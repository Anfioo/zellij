//! `[cli_client]` 用于连接到正在运行的服务端会话并分发通过命令行指定的操作。
use std::collections::{BTreeMap, HashSet};
use std::io::{self, BufRead, Write};
use std::process;
use std::str::FromStr;
use std::{fs, path::PathBuf};

use crate::os_input_output::ClientOsApi;
use uuid::Uuid;
use zellij_utils::{
    cli::{SubscribeCli, SubscribeFormat},
    data::PaneId,
    errors::prelude::*,
    input::actions::Action,
    ipc::{ClientToServerMsg, ExitReason, ServerToClientMsg},
};

pub fn start_cli_client(
    mut os_input: Box<dyn ClientOsApi>,
    session_name: &str,
    actions: Vec<Action>,
) -> i32 {
    let zellij_ipc_pipe: PathBuf = {
        let mut sock_dir = zellij_utils::consts::ZELLIJ_SOCK_DIR.clone();
        fs::create_dir_all(&sock_dir).unwrap();
        zellij_utils::shared::set_permissions(&sock_dir, 0o700).unwrap();
        sock_dir.push(session_name);
        sock_dir
    };
    crate::check_ipc_pipe_length(&zellij_ipc_pipe);
    os_input.connect_to_server(&*zellij_ipc_pipe);
    let pane_id = os_input
        .env_variable("ZELLIJ_PANE_ID")
        .and_then(|e| e.trim().parse().ok());

    for action in actions {
        match action {
            Action::CliPipe {
                pipe_id,
                name,
                payload,
                plugin,
                args,
                configuration,
                launch_new,
                skip_cache,
                floating,
                in_place,
                cwd,
                pane_title,
            } => {
                pipe_client(
                    &mut os_input,
                    pipe_id,
                    name,
                    payload,
                    plugin,
                    args,
                    configuration,
                    launch_new,
                    skip_cache,
                    floating,
                    in_place,
                    pane_id,
                    cwd,
                    pane_title,
                );
            },
            action => {
                if let Some(exit_status) =
                    individual_messages_client(&mut os_input, action, pane_id)
                {
                    return exit_status;
                }
            },
        }
    }
    os_input.send_to_server(ClientToServerMsg::ClientExited);
    0
}

fn pipe_client(
    os_input: &mut Box<dyn ClientOsApi>,
    pipe_id: String,
    mut name: Option<String>,
    mut payload: Option<String>,
    plugin: Option<String>,
    args: Option<BTreeMap<String, String>>,
    mut configuration: Option<BTreeMap<String, String>>,
    launch_new: bool,
    skip_cache: bool,
    floating: Option<bool>,
    in_place: Option<bool>,
    pane_id: Option<u32>,
    cwd: Option<PathBuf>,
    pane_title: Option<String>,
) {
    let mut stdin = os_input.get_stdin_reader();
    let name = name
        // 首先尝试获取显式提供的消息名称
        .take()
        // 然后使用 plugin，以便于使用别名
        .or_else(|| plugin.clone())
        // 最后使用 uuid，至少为该消息提供某种标识符
        .or_else(|| Some(Uuid::new_v4().to_string()));
    if launch_new {
        // 这样做是为了确保插件是唯一的（具有唯一的配置参数），以便会启动一个新插件，
        // 但我们仍然会将其发送到同一个实例，而不是在循环的每次迭代中都启动一个新实例
        configuration
            .get_or_insert_with(BTreeMap::new)
            .insert("_zellij_id".to_owned(), Uuid::new_v4().to_string());
    }
    let create_msg = |payload: Option<String>| -> ClientToServerMsg {
        ClientToServerMsg::Action {
            action: Action::CliPipe {
                pipe_id: pipe_id.clone(),
                name: name.clone(),
                payload,
                args: args.clone(),
                plugin: plugin.clone(),
                configuration: configuration.clone(),
                floating,
                in_place,
                launch_new,
                skip_cache,
                cwd: cwd.clone(),
                pane_title: pane_title.clone(),
            },
            terminal_id: pane_id,
            client_id: None,
            is_cli_client: true,
        }
    };
    let is_piped = !os_input.stdin_is_terminal();
    loop {
        if let Some(payload) = payload.take() {
            let msg = create_msg(Some(payload));
            os_input.send_to_server(msg);
        } else if !is_piped {
            // 这里我们发送一条空消息来触发插件，因为我们没有更多数据了
            let msg = create_msg(None);
            os_input.send_to_server(msg);
        } else {
            // 我们没有从命令行获取 payload，这意味着我们在监听标准输入，因为这表示用户
            // 即将通过管道传入更多数据（例如 cat my-large-file | zellij pipe ...）
            let mut buffer = String::new();
            let _ = stdin.read_line(&mut buffer);
            if buffer.is_empty() {
                let msg = create_msg(None);
                os_input.send_to_server(msg);
                break;
            } else {
                // 我们收到数据了！将其发送到管道（最常见的情况）
                let msg = create_msg(Some(buffer));
                os_input.send_to_server(msg);
            }
        }
        loop {
            // 等待响应并采取相应行动
            match os_input.recv_from_server() {
                Some((ServerToClientMsg::UnblockCliPipeInput { pipe_name }, _)) => {
                    // 解除此管道的阻塞，意味着我们需要停止等待响应并再次从标准输入读取
                    if pipe_name == pipe_id {
                        if !is_piped {
                            // 如果此客户端未通过管道连接，我们需要完全退出进程，而不是等待更多数据
                            process::exit(0);
                        } else {
                            break;
                        }
                    }
                },
                Some((ServerToClientMsg::CliPipeOutput { pipe_name, output }, _)) => {
                    // 将数据发送到标准输出，这*并不*意味着我们需要解除输入的阻塞
                    let err_context = "Failed to write to stdout";
                    if pipe_name == pipe_id {
                        let mut stdout = os_input.get_stdout_writer();
                        stdout
                            .write_all(output.as_bytes())
                            .context(err_context)
                            .non_fatal();
                        stdout.flush().context(err_context).non_fatal();
                    }
                },
                Some((ServerToClientMsg::Log { lines: log_lines }, _)) => {
                    log_lines.iter().for_each(|line| println!("{line}"));
                    process::exit(0);
                },
                Some((ServerToClientMsg::LogError { lines: log_lines }, _)) => {
                    log_lines.iter().for_each(|line| eprintln!("{line}"));
                    process::exit(2);
                },
                Some((ServerToClientMsg::Exit { exit_reason }, _)) => match exit_reason {
                    ExitReason::Error(e) => {
                        eprintln!("{}", e);
                        process::exit(2);
                    },
                    _ => {
                        process::exit(0);
                    },
                },
                _ => {},
            }
        }
    }
}

fn individual_messages_client(
    os_input: &mut Box<dyn ClientOsApi>,
    action: Action,
    pane_id: Option<u32>,
) -> Option<i32> {
    let is_blocking = matches!(
        &action,
        Action::NewBlockingPane {
            unblock_condition: Some(_),
            ..
        }
    );
    let msg = ClientToServerMsg::Action {
        action,
        terminal_id: pane_id,
        client_id: None,
        is_cli_client: true,
    };
    os_input.send_to_server(msg);
    loop {
        match os_input.recv_from_server() {
            Some((ServerToClientMsg::UnblockInputThread, _)) if !is_blocking => {
                return None;
            },
            Some((ServerToClientMsg::Log { lines: log_lines }, _)) => {
                log_lines.iter().for_each(|line| println!("{line}"));
                return None;
            },
            Some((ServerToClientMsg::LogError { lines: log_lines }, _)) => {
                log_lines.iter().for_each(|line| eprintln!("{line}"));
                return Some(2);
            },
            Some((ServerToClientMsg::Exit { exit_reason }, _)) => match exit_reason {
                ExitReason::Error(e) => {
                    eprintln!("{}", e);
                    return Some(2);
                },
                ExitReason::CustomExitStatus(exit_status) => {
                    return Some(exit_status);
                },
                _ => {
                    return None;
                },
            },
            _ => {},
        }
    }
}

pub fn start_subscribe_client(
    os_input: Box<dyn ClientOsApi>,
    session_name: &str,
    subscribe_cli: SubscribeCli,
) {
    let zellij_ipc_pipe: PathBuf = {
        let mut sock_dir = zellij_utils::consts::ZELLIJ_SOCK_DIR.clone();
        fs::create_dir_all(&sock_dir).unwrap();
        zellij_utils::shared::set_permissions(&sock_dir, 0o700).unwrap();
        sock_dir.push(session_name);
        sock_dir
    };
    crate::check_ipc_pipe_length(&zellij_ipc_pipe);
    os_input.connect_to_server(&*zellij_ipc_pipe);

    // 解析窗格 ID
    let pane_ids: Vec<PaneId> = subscribe_cli
        .pane_id
        .iter()
        .map(|s| {
            PaneId::from_str(s).unwrap_or_else(|e| {
                eprintln!("Invalid pane ID '{}': {}", s, e);
                process::exit(2);
            })
        })
        .collect();

    // 发送订阅消息
    os_input.send_to_server(ClientToServerMsg::SubscribeToPaneRenders {
        pane_ids: pane_ids.clone(),
        scrollback: subscribe_cli.scrollback,
        ansi: subscribe_cli.ansi,
    });

    // 跟踪剩余窗格，以便在所有窗格关闭时退出
    let mut remaining_panes: HashSet<PaneId> = pane_ids.into_iter().collect();

    // 流式接收循环
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    loop {
        match os_input.recv_from_server() {
            Some((
                ServerToClientMsg::PaneRenderUpdate {
                    pane_id,
                    viewport,
                    scrollback,
                    is_initial,
                },
                _,
            )) => match subscribe_cli.format {
                SubscribeFormat::Raw => {
                    if let Some(ref scrollback_lines) = scrollback {
                        for line in scrollback_lines {
                            let _ = writeln!(stdout, "{}", line);
                        }
                    }
                    for line in &viewport {
                        let _ = writeln!(stdout, "{}", line);
                    }
                    let _ = stdout.flush();
                },
                SubscribeFormat::Json => {
                    let json = serde_json::json!({
                        "event": "pane_update",
                        "pane_id": pane_id.to_string(),
                        "viewport": viewport,
                        "scrollback": scrollback,
                        "is_initial": is_initial,
                    });
                    let _ = writeln!(stdout, "{}", json);
                    let _ = stdout.flush();
                },
            },
            Some((ServerToClientMsg::SubscribedPaneClosed { pane_id }, _)) => {
                remaining_panes.remove(&pane_id);
                match subscribe_cli.format {
                    SubscribeFormat::Raw => {},
                    SubscribeFormat::Json => {
                        let json = serde_json::json!({
                            "event": "pane_closed",
                            "pane_id": pane_id.to_string(),
                        });
                        let _ = writeln!(stdout, "{}", json);
                        let _ = stdout.flush();
                    },
                }
                if remaining_panes.is_empty() {
                    break;
                }
            },
            Some((ServerToClientMsg::Exit { .. }, _)) => break,
            Some((ServerToClientMsg::LogError { lines }, _)) => {
                for line in lines {
                    eprintln!("{}", line);
                }
                process::exit(2);
            },
            None => break,
            _ => {},
        }
    }

    os_input.send_to_server(ClientToServerMsg::ClientExited);
}
