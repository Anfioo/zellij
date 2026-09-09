use crate::os_input_output::ClientOsApi;
use crate::web_client::control_message::{SetConfigPayload, WebServerToWebClientControlMessage};
use crate::web_client::host_query_seed::build_host_query_seed_msgs;
use crate::web_client::session_management::{
    build_initial_connection, create_first_message, create_ipc_pipe,
};
use crate::web_client::types::{
    take_pending_welcome_session, ClientConnectionBus, ConnectionTable, PendingWelcomeSessions,
    SessionManager,
};
use crate::web_client::utils::terminal_init_messages;

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use zellij_utils::{
    cli::CliArgs,
    data::Style,
    input::{config::Config, options::Options},
    ipc::{ClientToServerMsg, ExitReason, PixelDimensions, ServerToClientMsg},
    pane_size::{Size, SizeInPixels},
    sessions::generate_unique_session_name,
    setup::Setup,
};

pub fn zellij_server_listener(
    os_input: Box<dyn ClientOsApi>,
    connection_table: Arc<Mutex<ConnectionTable>>,
    session_name: Option<String>,
    mut config: Config,
    mut config_options: Options,
    config_file_path: Option<PathBuf>,
    web_client_id: String,
    session_manager: Arc<dyn SessionManager>,
    attachment_complete_tx: Option<tokio::sync::oneshot::Sender<()>>,
    client_size: Option<Size>,
    client_pixel_dims: Option<SizeInPixels>,
    pending_welcome_sessions: PendingWelcomeSessions,
) {
    let _server_listener_thread = std::thread::Builder::new()
        .name("server_listener".to_string())
        .spawn({
            move || {
                let mut client_connection_bus =
                    ClientConnectionBus::new(&web_client_id, &connection_table);
                let is_welcome_session = session_name
                    .as_ref()
                    .map(|name| take_pending_welcome_session(&pending_welcome_sessions, name))
                    .unwrap_or(true);
                let mut reconnect_to_session =
                    match build_initial_connection(session_name, is_welcome_session, &config) {
                        Ok(initial_session_connection) => initial_session_connection,
                        Err(e) => {
                            log::error!("{}", e);
                            return;
                        },
                    };
                let mut attachment_complete_tx = attachment_complete_tx;
                let mut host_query_seed_msgs: Option<Vec<ClientToServerMsg>> = None;
                let mut switched_from_previous_session = false;
                'reconnect_loop: loop {
                    let reconnect_info = reconnect_to_session.take();
                    let initial_layout = reconnect_info.as_ref().and_then(|r| r.layout.clone());
                    let path = {
                        let Some(session_name) = reconnect_info
                            .as_ref()
                            .and_then(|r| r.name.clone())
                            .or_else(generate_unique_session_name)
                        else {
                            log::error!("无法生成唯一的会话名称，正在放弃。");
                            client_connection_bus.close_connection();
                            return;
                        };
                        let mut sock_dir = zellij_utils::consts::ZELLIJ_SOCK_DIR.clone();
                        if let Err(e) = zellij_utils::sessions::validate_session_name(&session_name) {
                            log::error!("无效的会话名称：{}", e);
                            client_connection_bus.close_connection();
                            return;
                        }
                        sock_dir.push(session_name.clone());
                        sock_dir.to_str().unwrap().to_owned()
                    };

                    reload_config_from_disk(&mut config, &mut config_options, &config_file_path);

                    let full_screen_ws =
                        client_size.unwrap_or_else(|| os_input.get_terminal_size());
                    let mut sent_init_messages = false;

                    let palette = config
                        .theme_config(config_options.theme.as_ref())
                        .unwrap_or_else(|| os_input.load_palette().into());
                    let client_attributes = zellij_utils::ipc::ClientAttributes {
                        size: full_screen_ws,
                        style: Style {
                            colors: palette,
                            rounded_corners: config.ui.pane_frames.rounded_corners,
                            hide_session_name: config.ui.pane_frames.hide_session_name,
                        },
                    };

                    let session_name = PathBuf::from(path.clone())
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_owned();

                    // 从连接表中查找只读状态
                    let is_read_only = connection_table
                        .lock()
                        .unwrap()
                        .is_client_read_only(&web_client_id);


                    let session_exists = session_manager.session_exists(&session_name).unwrap_or(false);

                    if is_read_only && !session_exists {
                        log::error!("只读令牌无法创建新会话。");
                        client_connection_bus.close_connection();
                        return;
                    }

                    let should_create_new_session = !session_exists;
                    let first_message = create_first_message(is_read_only, config_file_path.clone(), client_attributes.clone(), config_options.clone(), should_create_new_session, &session_name, initial_layout);
                    let zellij_ipc_pipe = create_ipc_pipe(&session_name);

                    session_manager.spawn_session_if_needed(
                        &session_name,
                        os_input.clone(),
                        session_exists,
                        &zellij_ipc_pipe,
                        first_message,
                    );

                    if let Some(pixel_dims) = client_pixel_dims {
                        os_input.send_to_server(ClientToServerMsg::TerminalPixelDimensions {
                            pixel_dimensions: PixelDimensions {
                                text_area_size: client_size.map(|size| SizeInPixels {
                                    width: size.cols * pixel_dims.width,
                                    height: size.rows * pixel_dims.height,
                                }),
                                character_cell_size: Some(pixel_dims),
                            },
                        });
                    }

                    // 用从 Config 派生的 web 客户端状态（fg/bg/palette）植入
                    // 服务端的主机终端查询缓存。没有这个，来自应用程序的
                    // OSC 10/11/4 查询会在 1s 服务端转发超时上停滞，然后返回空。
                    // 像素尺寸在浏览器报告后通过 TerminalMetrics 控制消息单独植入。
                    let seeds = host_query_seed_msgs
                        .get_or_insert_with(|| build_host_query_seed_msgs(&config, &config_options));
                    for seed in seeds.iter().cloned() {
                        os_input.send_to_server(seed);
                    }

                    if let Some(tx) = attachment_complete_tx.take() {
                        let _ = tx.send(());
                    }

                    if switched_from_previous_session {
                        client_connection_bus.send_control(
                            WebServerToWebClientControlMessage::SwitchedSession {
                                new_session_name: session_name.clone(),
                            },
                        );
                    }

                    let mut unknown_message_count = 0;
                    loop {
                        let msg = os_input.recv_from_server();
                        if msg.is_some() {
                            unknown_message_count = 0;
                        } else {
                            unknown_message_count += 1;
                        }
                        match msg.map(|m| m.0) {
                            Some(ServerToClientMsg::UnblockInputThread) => {},
                            Some(ServerToClientMsg::Connected) => {},
                            Some(ServerToClientMsg::CliPipeOutput { .. } ) => {},
                            Some(ServerToClientMsg::UnblockCliPipeInput { .. } ) => {},
                            Some(ServerToClientMsg::StartWebServer { .. } ) => {},
                            Some(ServerToClientMsg::Exit{exit_reason}) => {
                                handle_exit_reason(&mut client_connection_bus, exit_reason);
                                os_input.send_to_server(ClientToServerMsg::ClientExited);
                                break;
                            },
                            Some(ServerToClientMsg::Render{content: bytes}) => {
                                if !sent_init_messages {
                                    client_connection_bus
                                        .send_stdout(terminal_init_messages().concat());
                                    sent_init_messages = true;
                                }
                                client_connection_bus.send_stdout(bytes);
                            },
                            Some(ServerToClientMsg::SwitchSession{connect_to_session}) => {
                                reconnect_to_session = Some(connect_to_session);
                                switched_from_previous_session = true;
                                continue 'reconnect_loop;
                            },
                            Some(ServerToClientMsg::QueryTerminalSize) => {
                                client_connection_bus.send_control(
                                    WebServerToWebClientControlMessage::QueryTerminalSize,
                                );
                            },
                            Some(ServerToClientMsg::SetSoftKeyboard{on}) => {
                                client_connection_bus.send_control(
                                    WebServerToWebClientControlMessage::SetSoftKeyboard { on },
                                );
                            },
                            Some(ServerToClientMsg::MobileState{payload}) => {
                                client_connection_bus.send_control(
                                    WebServerToWebClientControlMessage::MobileState { payload },
                                );
                            },
                            Some(ServerToClientMsg::Log{lines}) => {
                                client_connection_bus.send_control(
                                    WebServerToWebClientControlMessage::Log { lines },
                                );
                            },
                            Some(ServerToClientMsg::LogError{lines}) => {
                                client_connection_bus.send_control(
                                    WebServerToWebClientControlMessage::LogError { lines },
                                );
                            },
                            Some(ServerToClientMsg::RenamedSession{name: new_session_name}) => {
                                client_connection_bus.send_control(
                                    WebServerToWebClientControlMessage::SwitchedSession {
                                        new_session_name,
                                    },
                                );
                            },
                            Some(ServerToClientMsg::ConfigFileUpdated) => {

                                if let Some(config_file_path) = &config_file_path {
                                    if let Ok(new_config) = Config::from_path(&config_file_path, Some(config.clone())) {
                                        // 为此客户端重新植入主机查询缓存，
                                        // 以便 OSC 10/11/4 回复遵循新主题。
                                        for seed in build_host_query_seed_msgs(&new_config, &config_options) {
                                            os_input.send_to_server(seed);
                                        }
                                        let set_config_payload = SetConfigPayload::from(&new_config);
                                        client_connection_bus.send_control(
                                            WebServerToWebClientControlMessage::SetConfig(set_config_payload),
                                        );
                                    }
                                }
                            },
                            // 仅订阅消息 — 与 web 客户端无关
                            Some(ServerToClientMsg::PaneRenderUpdate { .. }) => {},
                            Some(ServerToClientMsg::SubscribedPaneClosed { .. }) => {},
                            Some(ServerToClientMsg::EmitNestedSessionFrame { .. }) => {},
                            Some(ServerToClientMsg::ForwardQueryToHost { token, .. }) => {
                                // 立即用空 reply_bytes 回复。
                                // 这是现有的约定，表示"没有可用的主机回复 —
                                // 请从缓存状态合成"。服务端的
                                // synthesize_cached_reply 路径将使用我们已经
                                // 从浏览器/配置植入的像素尺寸和颜色，返回
                                // 真实答案，而不是等待 1000ms 转发超时。
                                os_input.send_to_server(
                                    ClientToServerMsg::ForwardedReplyFromHost {
                                        token,
                                        reply_bytes: Vec::new(),
                                    },
                                );
                            },
                            None => {
                                if unknown_message_count >= 1000 {
                                    log::error!("错误：连续收到超过 1000 条未知的服务器消息，正在断开连接。");
                                    // 这可能意味着我们处于无限循环中，让我们断开连接
                                    // 以免导致 100% CPU
                                    break;
                                }
                            },
                        }
                    }
                    if reconnect_to_session.is_none() {
                        break;
                    }
                }
            }
        });
}

fn handle_exit_reason(client_connection_bus: &mut ClientConnectionBus, exit_reason: ExitReason) {
    match exit_reason {
        ExitReason::KickedByHost => {
            client_connection_bus.close_connection_kicked();
            return;
        },
        ExitReason::WebClientsForbidden => {
            client_connection_bus.send_stdout(format!(
                "\u{1b}[2J\n Web Clients are not allowed to attach to this session."
            ));
        },
        ExitReason::Error(e) => {
            let goto_start_of_last_line = format!("\u{1b}[{};{}H", 1, 1);
            let clear_client_terminal_attributes = "\u{1b}[?1l\u{1b}=\u{1b}[r\u{1b}[?1000l\u{1b}[?1002l\u{1b}[?1003l\u{1b}[?1005l\u{1b}[?1006l\u{1b}[?12l";
            let disable_mouse = "\u{1b}[?1006l\u{1b}[?1015l\u{1b}[?1003l\u{1b}[?1002l\u{1b}[?1000l";
            let error = format!(
                "{}{}\n{}{}\n",
                disable_mouse,
                clear_client_terminal_attributes,
                goto_start_of_last_line,
                e.to_string().replace("\n", "\n\r")
            );
            client_connection_bus.send_stdout(format!("\u{1b}[2J\n{}", error));
        },
        _ => {},
    }
    client_connection_bus.close_connection();
}

fn reload_config_from_disk(
    config_without_layout: &mut Config,
    config_options_without_layout: &mut Options,
    config_file_path: &Option<PathBuf>,
) {
    let mut cli_args = CliArgs::default();
    cli_args.config = config_file_path.clone();
    match Setup::from_cli_args(&cli_args) {
        Ok((_, _, _, reloaded_config_without_layout, reloaded_config_options_without_layout)) => {
            *config_without_layout = reloaded_config_without_layout;
            *config_options_without_layout = reloaded_config_options_without_layout;
        },
        Err(e) => {
            log::error!("重新加载配置失败：{}", e);
        },
    };
}
