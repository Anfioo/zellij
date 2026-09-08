use crate::{panes::PaneId, ClientId};

use interprocess::local_socket::Stream as LocalSocketStream;

#[cfg(not(windows))]
use crate::os_input_output_unix::UnixPtyBackend as PtyBackendImpl;
#[cfg(windows)]
use crate::os_input_output_windows::WindowsPtyBackend as PtyBackendImpl;

use interprocess;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
use tempfile::tempfile;
use zellij_utils::{
    channels,
    channels::TrySendError,
    data::Palette,
    errors::prelude::*,
    input::command::{RunCommand, TerminalAction},
    ipc::{
        ClientToServerMsg, ExitReason, IpcReceiverWithContext, IpcSenderWithContext,
        ServerToClientMsg,
    },
    shared::default_palette,
};

use std::{
    collections::{BTreeMap, HashMap},
    env,
    fs::File,
    io::{self, Write},
    path::PathBuf,
    process::Command,
    sync::{Arc, Mutex},
};

pub use async_trait::async_trait;

/// 检查候选路径是否指向可执行文件，考虑 Windows 上的 PATHEXT 扩展名（如 `.exe`、`.cmd`）。
///
/// 在 Windows 上，当候选文件没有扩展名时，我们会在裸匹配之前尝试每个 PATHEXT 变体，镜像 cmd.exe 的解析方式。像 Composer 这样的工具会同时安装 `composer`（Unix 启动器）和 `composer.bat`（Windows 启动器）；返回裸文件会将非 PE 二进制文件发送给 CreateProcessW 并因 ERROR_BAD_EXE_FORMAT 而失败。
fn find_executable(candidate: &std::path::Path) -> Option<PathBuf> {
    #[cfg(windows)]
    {
        if candidate.extension().is_none() {
            if let Some(pathext) = env::var_os("PATHEXT") {
                let pathext = pathext.to_string_lossy();
                for ext in pathext.split(';') {
                    let ext = ext.trim();
                    if ext.is_empty() {
                        continue;
                    }
                    let mut with_ext = candidate.as_os_str().to_os_string();
                    with_ext.push(ext);
                    let with_ext_path = PathBuf::from(with_ext);
                    if with_ext_path.exists() && with_ext_path.is_file() {
                        return Some(with_ext_path);
                    }
                }
            }
        }
    }
    if candidate.exists() && candidate.is_file() {
        return Some(candidate.to_path_buf());
    }
    None
}

/// 将命令解析为其绝对路径，先搜索工作目录，再搜索 PATH（Windows 上还包括 PATHEXT）。
pub(crate) fn resolve_command(cmd: &RunCommand) -> Option<PathBuf> {
    let command = &cmd.command;
    match cmd.cwd.as_ref() {
        Some(cwd) => {
            if let Some(resolved) = find_executable(&cwd.join(command)) {
                return Some(resolved);
            }
        },
        None => {
            if let Some(resolved) = find_executable(command) {
                return Some(resolved);
            }
        },
    }
    if let Some(paths) = env::var_os("PATH") {
        for path in env::split_paths(&paths) {
            if let Some(resolved) = find_executable(&path.join(command)) {
                return Some(resolved);
            }
        }
    }
    None
}

#[cfg(not(windows))]
pub(crate) fn command_exists(cmd: &RunCommand) -> bool {
    resolve_command(cmd).is_some()
}

// 这是一个实用方法，用于在将 pathbuf 转换为 Command 之前从中分离参数。例如 "/usr/bin/vim -e" ==> "/usr/bin/vim" + "-e"（后者将被推送到 args）
fn separate_command_arguments(command: &mut PathBuf, args: &mut Vec<String>) {
    let mut parts = vec![];
    let mut current_part = String::new();
    for part in command.display().to_string().split_ascii_whitespace() {
        current_part.push_str(part);
        if current_part.ends_with('\\') {
            let _ = current_part.pop();
            current_part.push(' ');
        } else {
            let current_part = std::mem::replace(&mut current_part, String::new());
            parts.push(current_part);
        }
    }
    if !parts.is_empty() {
        *command = PathBuf::from(parts.remove(0));
        args.append(&mut parts);
    }
}

/// 如果给定 [`TerminalAction::OpenFile(file)`]，将在新终端中启动由环境变量 `EDITOR` 指定的文本编辑器（如果未设置 `EDITOR` 则使用 `VISUAL`），并打开给定文件。
/// 如果给定 [`TerminalAction::RunCommand(RunCommand)`]，将在新终端中启动该命令。
/// 如果给定 None，将在新终端中启动由环境变量 `SHELL` 指定的 shell。
///
/// 返回 (cmd, failover_cmd)。
fn build_command(
    terminal_action: TerminalAction,
    default_editor: Option<PathBuf>,
) -> (RunCommand, Option<RunCommand>) {
    let mut failover_cmd_args = None;
    let cmd = match terminal_action {
        TerminalAction::OpenFile(mut payload) => {
            if payload.path.is_relative() {
                if let Some(cwd) = payload.cwd.as_ref() {
                    payload.path = cwd.join(payload.path);
                }
            }
            let mut command = default_editor.unwrap_or_else(|| {
                PathBuf::from(
                    env::var("EDITOR")
                        .unwrap_or_else(|_| env::var("VISUAL").unwrap_or_else(|_| "vi".into())),
                )
            });

            let mut args = vec![];

            if !command.is_dir() {
                separate_command_arguments(&mut command, &mut args);
            }
            let file_to_open = payload
                .path
                .into_os_string()
                .into_string()
                .expect("Not valid Utf8 Encoding");
            if let Some(line_number) = payload.line_number {
                if command.ends_with("vim")
                    || command.ends_with("nvim")
                    || command.ends_with("emacs")
                    || command.ends_with("nano")
                    || command.ends_with("kak")
                {
                    failover_cmd_args = Some(vec![file_to_open.clone()]);
                    args.push(format!("+{}", line_number));
                    args.push(file_to_open);
                } else if command.ends_with("hx") || command.ends_with("helix") {
                    // 在编写本文时，helix 仅支持此语法
                    // 而且无论如何将其保留在这里可能是个好主意
                    // 以继续支持旧版本
                    args.push(format!("{}:{}", file_to_open, line_number));
                } else {
                    args.push(file_to_open);
                }
            } else {
                args.push(file_to_open);
            }
            RunCommand {
                command,
                args,
                cwd: payload.cwd,
                hold_on_close: false,
                hold_on_start: false,
                ..Default::default()
            }
        },
        TerminalAction::RunCommand(command) => command,
    };
    let failover_cmd = if let Some(failover_cmd_args) = failover_cmd_args {
        let mut failover = cmd.clone();
        failover.args = failover_cmd_args;
        Some(failover)
    } else {
        None
    };
    (cmd, failover_cmd)
}

// ClientSender 负责在一个特殊线程上向客户端发送消息
// 这样做是为了在 unix socket 缓冲区已满时，不会阻塞整个路由器线程
// 当上述情况发生时，ClientSender 会缓冲消息，希望拥塞能被解除，直到我们耗尽缓冲区空间。
// 如果我们耗尽了缓冲区空间，会向上抛出错误，以便路由器线程放弃此客户端，我们将停止向其发送消息。
// 如果客户端再次变得有响应，我们会发送最后一条 "Buffer full" 消息，让它知道发生了什么。
#[derive(Clone)]
struct ClientSender {
    client_id: ClientId,
    client_buffer_sender: channels::Sender<ServerToClientMsg>,
}

impl ClientSender {
    pub fn new(client_id: ClientId, mut sender: IpcSenderWithContext<ServerToClientMsg>) -> Self {
        // FIXME(hartan): 此队列负责在服务端和客户端之间缓冲消息。如果它满了，客户端会因类似 "Buffer full" 的错误消息而断开连接。之前发现它太小（深度为 50），因此增加到了 5000。做出这个决定是因为发现深度为 5000 的队列不会导致 RAM 使用量明显增加，但除此之外没有其他原因。如果将来发现它又太快填满，可能值得进一步增加大小（或者更好的是，实现一个背压时重绘机制）。
        // 我们 zellij 维护者暂时决定不使用无界队列，因为我们希望防止例如整个会话仅仅因为单个客户端不响应而被杀死（由 OOM-killer 或其他机制）。
        let (client_buffer_sender, client_buffer_receiver) = channels::bounded(5000);
        std::thread::spawn(move || {
            let err_context = || format!("failed to send message to client {client_id}");
            for msg in client_buffer_receiver.iter() {
                sender
                    .send_server_msg(msg)
                    .with_context(err_context)
                    .non_fatal();
            }
            let _ = sender.send_server_msg(ServerToClientMsg::Exit {
                exit_reason: ExitReason::Disconnect,
            });
        });
        ClientSender {
            client_id,
            client_buffer_sender,
        }
    }
    pub fn send_or_buffer(&self, msg: ServerToClientMsg) -> Result<()> {
        let err_context = || {
            format!(
                "failed to send or buffer message for client {}",
                self.client_id
            )
        };

        self.client_buffer_sender
            .try_send(msg)
            .or_else(|err| {
                if let TrySendError::Full(_) = err {
                    log::warn!(
                        "client {} is processing server messages too slow",
                        self.client_id
                    );
                }
                Err(err)
            })
            .with_context(err_context)
    }
}

#[derive(Clone)]
pub struct ServerOsInputOutput {
    pty_backend: PtyBackendImpl,
    client_senders: Arc<Mutex<HashMap<ClientId, ClientSender>>>,
    cached_resizes: Arc<Mutex<Option<BTreeMap<u32, (u16, u16, Option<u16>, Option<u16>)>>>>,
}

/// 用于挂起窗格的空 `AsyncReader`（立即产生 EOF）。
pub(crate) struct NullAsyncReader;

// Rust 不支持 trait 中的 async fn，因此使用了 dtolnay 出色的 async_trait 宏。参见 https://smallcultfollowing.com/babysteps/blog/2019/10/26/async-fn-in-traits-are-hard/
#[async_trait]
pub trait AsyncReader: Send + Sync {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error>;
}

#[async_trait]
impl AsyncReader for NullAsyncReader {
    async fn read(&mut self, _buf: &mut [u8]) -> Result<usize, io::Error> {
        Ok(0) // EOF
    }
}

/// `ServerOsApi` trait 表示 Zellij 服务端所需的操作系统功能的抽象接口。
pub trait ServerOsApi: Send + Sync {
    fn set_terminal_size_using_terminal_id(
        &self,
        id: u32,
        cols: u16,
        rows: u16,
        width_in_pixels: Option<u16>,
        height_in_pixels: Option<u16>,
    ) -> Result<()>;
    /// 使用终端操作生成新终端。返回的元组包含：
    /// - terminal_id (u32)
    /// PTY 输出的异步读取器
    /// - 子进程 PID（如果可用）(Option<u32>)
    fn spawn_terminal(
        &self,
        terminal_action: TerminalAction,
        quit_cb: Box<dyn Fn(PaneId, Option<i32>, RunCommand) + Send>,
        default_editor: Option<PathBuf>,
    ) -> Result<(u32, Box<dyn AsyncReader>, Option<u32>)>;
    // 保留终端 ID 而不实际打开终端
    fn reserve_terminal_id(&self) -> Result<u32> {
        unimplemented!()
    }
    /// 将字节写入由 `terminal_id` 引用的虚拟终端的标准输入。
    fn write_to_tty_stdin(&self, terminal_id: u32, buf: &[u8]) -> Result<usize>;
    /// 等待直到写入终端的所有输出都已传输完毕。
    fn tcdrain(&self, terminal_id: u32) -> Result<()>;
    /// 终止进程 ID 为 `pid` 的进程。(SIGHUP)
    fn kill(&self, pid: u32) -> Result<()>;
    /// 终止进程 ID 为 `pid` 的进程。(SIGKILL)
    fn force_kill(&self, pid: u32) -> Result<()>;
    /// 向进程 ID 为 `pid` 的进程发送 SIGINT
    fn send_sigint(&self, pid: u32) -> Result<()>;
    /// 返回指向此 [`ServerOsApi`] 结构体的 [`Box`] 指针。
    fn box_clone(&self) -> Box<dyn ServerOsApi>;
    fn send_to_client(&self, client_id: ClientId, msg: ServerToClientMsg) -> Result<()>;
    fn new_client(
        &mut self,
        client_id: ClientId,
        stream: LocalSocketStream,
    ) -> Result<IpcReceiverWithContext<ClientToServerMsg>>;
    /// 使用单独的回复流创建新客户端（Windows 双管道 IPC）。
    fn new_client_with_reply(
        &mut self,
        client_id: ClientId,
        stream: LocalSocketStream,
        reply_stream: LocalSocketStream,
    ) -> Result<IpcReceiverWithContext<ClientToServerMsg>>;
    fn remove_client(&mut self, client_id: ClientId) -> Result<()>;
    fn load_palette(&self) -> Palette;
    /// 返回给定 pid 的当前工作目录
    fn get_cwd(&self, pid: u32) -> Option<PathBuf>;
    /// 返回多个 pid 的当前工作目录
    fn get_cwds(&self, _pids: Vec<u32>) -> (HashMap<u32, PathBuf>, HashMap<u32, Vec<String>>) {
        (HashMap::new(), HashMap::new())
    }
    /// 按父进程 ID 获取所有正在运行的命令的列表
    fn get_all_cmds_by_ppid(&self, _post_hook: &Option<String>) -> HashMap<String, Vec<String>> {
        HashMap::new()
    }
    /// 对于每个 `(terminal_id, shell_pid)` 窗格，返回在其控制终端中运行的前台命令，以 `terminal_id` 为键。
    fn get_foreground_cmds(
        &self,
        _panes: &[(u32, u32)],
        _post_hook: &Option<String>,
    ) -> HashMap<u32, Vec<String>> {
        HashMap::new()
    }
    /// 将给定缓冲区写入字符串
    fn write_to_file(&mut self, buf: String, file: Option<String>) -> Result<()>;

    fn re_run_command_in_terminal(
        &self,
        terminal_id: u32,
        run_command: RunCommand,
        quit_cb: Box<dyn Fn(PaneId, Option<i32>, RunCommand) + Send>,
    ) -> Result<(Box<dyn AsyncReader>, Option<u32>)>;
    fn clear_terminal_id(&self, terminal_id: u32) -> Result<()>;
    fn cache_resizes(&mut self) {}
    fn apply_cached_resizes(&mut self) {}
}

impl ServerOsApi for ServerOsInputOutput {
    fn set_terminal_size_using_terminal_id(
        &self,
        id: u32,
        cols: u16,
        rows: u16,
        width_in_pixels: Option<u16>,
        height_in_pixels: Option<u16>,
    ) -> Result<()> {
        if let Some(cached_resizes) = self.cached_resizes.lock().unwrap().as_mut() {
            cached_resizes.insert(id, (cols, rows, width_in_pixels, height_in_pixels));
            return Ok(());
        }
        self.pty_backend
            .set_terminal_size(id, cols, rows, width_in_pixels, height_in_pixels)
    }
    fn spawn_terminal(
        &self,
        terminal_action: TerminalAction,
        quit_cb: Box<dyn Fn(PaneId, Option<i32>, RunCommand) + Send>,
        default_editor: Option<PathBuf>,
    ) -> Result<(u32, Box<dyn AsyncReader>, Option<u32>)> {
        let err_context = || "failed to spawn terminal".to_string();

        let terminal_id = self
            .pty_backend
            .next_terminal_id()
            .context("no more terminal IDs left to allocate")?;

        self.pty_backend.reserve_terminal_id(terminal_id);

        let (cmd, failover_cmd) = build_command(terminal_action, default_editor);

        let (async_reader, child_fd) = self
            .pty_backend
            .spawn_terminal(cmd, failover_cmd, quit_cb, terminal_id)
            .with_context(err_context)?;

        Ok((terminal_id, async_reader, Some(child_fd as u32)))
    }
    fn reserve_terminal_id(&self) -> Result<u32> {
        let terminal_id = self
            .pty_backend
            .next_terminal_id()
            .context("no more terminal IDs available")?;
        self.pty_backend.reserve_terminal_id(terminal_id);
        Ok(terminal_id)
    }
    fn write_to_tty_stdin(&self, terminal_id: u32, buf: &[u8]) -> Result<usize> {
        self.pty_backend.write_to_tty_stdin(terminal_id, buf)
    }
    fn tcdrain(&self, terminal_id: u32) -> Result<()> {
        self.pty_backend.tcdrain(terminal_id)
    }
    fn box_clone(&self) -> Box<dyn ServerOsApi> {
        Box::new((*self).clone())
    }
    fn kill(&self, pid: u32) -> Result<()> {
        self.pty_backend.kill(pid)
    }
    fn force_kill(&self, pid: u32) -> Result<()> {
        self.pty_backend.force_kill(pid)
    }
    fn send_sigint(&self, pid: u32) -> Result<()> {
        self.pty_backend.send_sigint(pid)
    }
    fn send_to_client(&self, client_id: ClientId, msg: ServerToClientMsg) -> Result<()> {
        let err_context = || format!("failed to send message to client {client_id}");

        if let Some(sender) = self
            .client_senders
            .lock()
            .to_anyhow()
            .with_context(err_context)?
            .get_mut(&client_id)
        {
            sender.send_or_buffer(msg).with_context(err_context)
        } else {
            Ok(())
        }
    }

    fn new_client(
        &mut self,
        client_id: ClientId,
        stream: LocalSocketStream,
    ) -> Result<IpcReceiverWithContext<ClientToServerMsg>> {
        let receiver = IpcReceiverWithContext::new(stream);
        let sender = ClientSender::new(client_id, receiver.get_sender());
        self.client_senders
            .lock()
            .to_anyhow()
            .with_context(|| format!("failed to create new client {client_id}"))?
            .insert(client_id, sender);
        Ok(receiver)
    }

    fn new_client_with_reply(
        &mut self,
        client_id: ClientId,
        stream: LocalSocketStream,
        reply_stream: LocalSocketStream,
    ) -> Result<IpcReceiverWithContext<ClientToServerMsg>> {
        let receiver = IpcReceiverWithContext::new(stream);
        let sender = ClientSender::new(client_id, IpcSenderWithContext::new(reply_stream));
        self.client_senders
            .lock()
            .to_anyhow()
            .with_context(|| format!("failed to create new client {client_id}"))?
            .insert(client_id, sender);
        Ok(receiver)
    }

    fn remove_client(&mut self, client_id: ClientId) -> Result<()> {
        let mut client_senders = self
            .client_senders
            .lock()
            .to_anyhow()
            .with_context(|| format!("failed to remove client {client_id}"))?;
        if client_senders.contains_key(&client_id) {
            client_senders.remove(&client_id);
        }
        Ok(())
    }

    fn load_palette(&self) -> Palette {
        default_palette()
    }

    fn get_cwd(&self, pid: u32) -> Option<PathBuf> {
        let mut system_info = System::new();
        let sysinfo_pid = sysinfo::Pid::from_u32(pid);
        let refresh_kind = ProcessRefreshKind::nothing().with_cwd(UpdateKind::Always);
        system_info.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[sysinfo_pid]),
            false,
            refresh_kind,
        );

        if let Some(process) = system_info.process(sysinfo_pid) {
            if let Some(cwd) = process.cwd() {
                return Some(cwd.to_path_buf());
            }
        }
        None
    }

    fn get_cwds(&self, pids: Vec<u32>) -> (HashMap<u32, PathBuf>, HashMap<u32, Vec<String>>) {
        let mut system_info = System::new();
        let mut cwds = HashMap::new();
        let mut cmds = HashMap::new();

        let sysinfo_pids: Vec<sysinfo::Pid> =
            pids.iter().map(|&p| sysinfo::Pid::from_u32(p)).collect();
        let refresh_kind = ProcessRefreshKind::nothing()
            .with_cwd(UpdateKind::Always)
            .with_cmd(UpdateKind::Always);
        system_info.refresh_processes_specifics(
            ProcessesToUpdate::Some(&sysinfo_pids),
            false,
            refresh_kind,
        );

        for pid in pids {
            let sysinfo_pid = sysinfo::Pid::from_u32(pid);
            if let Some(process) = system_info.process(sysinfo_pid) {
                if let Some(cwd) = process.cwd() {
                    cwds.insert(pid, cwd.to_path_buf());
                }
                let cmd = process.cmd();
                if !cmd.is_empty() {
                    cmds.insert(
                        pid,
                        cmd.iter()
                            .map(|s| s.to_string_lossy().into_owned())
                            .collect(),
                    );
                }
            }
        }

        (cwds, cmds)
    }
    #[cfg(not(unix))]
    fn get_all_cmds_by_ppid(&self, post_hook: &Option<String>) -> HashMap<String, Vec<String>> {
        let mut system_info = System::new();
        let refresh_kind = ProcessRefreshKind::nothing().with_cmd(UpdateKind::Always);
        system_info.refresh_processes_specifics(ProcessesToUpdate::All, true, refresh_kind);
        let mut cmds = HashMap::new();
        for (_pid, process) in system_info.processes() {
            if let Some(parent_pid) = process.parent() {
                let ppid_str = format!("{}", parent_pid);
                let command: Vec<String> = process
                    .cmd()
                    .iter()
                    .map(|s| s.to_string_lossy().into_owned())
                    .collect();
                if command.is_empty() {
                    continue;
                }
                cmds.insert(ppid_str, apply_post_command_hook(command, post_hook));
            }
        }
        cmds
    }

    #[cfg(unix)]
    fn get_foreground_cmds(
        &self,
        panes: &[(u32, u32)],
        post_hook: &Option<String>,
    ) -> HashMap<u32, Vec<String>> {
        let mut terminal_to_fg_pid: HashMap<u32, u32> = HashMap::new();
        for &(terminal_id, shell_pid) in panes {
            if let Some(fpgid) = self.pty_backend.tcgetpgrp(terminal_id) {
                if fpgid > 0 && fpgid as u32 != shell_pid {
                    terminal_to_fg_pid.insert(terminal_id, fpgid as u32);
                }
            }
        }
        if terminal_to_fg_pid.is_empty() {
            return HashMap::new();
        }

        let sysinfo_pids: Vec<sysinfo::Pid> = terminal_to_fg_pid
            .values()
            .map(|&p| sysinfo::Pid::from_u32(p))
            .collect();
        let mut system_info = System::new();
        let refresh_kind = ProcessRefreshKind::nothing().with_cmd(UpdateKind::Always);
        system_info.refresh_processes_specifics(
            ProcessesToUpdate::Some(&sysinfo_pids),
            false,
            refresh_kind,
        );

        let mut cmds = HashMap::new();
        for (terminal_id, fg_pid) in terminal_to_fg_pid {
            let Some(process) = system_info.process(sysinfo::Pid::from_u32(fg_pid)) else {
                continue;
            };
            let command: Vec<String> = process
                .cmd()
                .iter()
                .map(|s| s.to_string_lossy().into_owned())
                .collect();
            if command.is_empty() {
                continue;
            }
            let command = apply_post_command_hook(command, post_hook);
            cmds.insert(terminal_id, command);
        }
        cmds
    }

    #[cfg(not(unix))]
    fn get_foreground_cmds(
        &self,
        panes: &[(u32, u32)],
        post_hook: &Option<String>,
    ) -> HashMap<u32, Vec<String>> {
        // 在 Windows 上使用基于 ppid 的发现，因为它没有控制终端前台组。
        let ppids_to_cmds = self.get_all_cmds_by_ppid(post_hook);
        let mut cmds = HashMap::new();
        for &(terminal_id, shell_pid) in panes {
            if let Some(cmd) = ppids_to_cmds.get(&shell_pid.to_string()) {
                cmds.insert(terminal_id, cmd.clone());
            }
        }
        cmds
    }

    fn write_to_file(&mut self, buf: String, name: Option<String>) -> Result<()> {
        let err_context = || "failed to write to file".to_string();

        let mut f: File = match name {
            Some(x) => File::create(x).with_context(err_context)?,
            None => tempfile().with_context(err_context)?,
        };
        write!(f, "{}", buf).with_context(err_context)
    }

    fn re_run_command_in_terminal(
        &self,
        terminal_id: u32,
        run_command: RunCommand,
        quit_cb: Box<dyn Fn(PaneId, Option<i32>, RunCommand) + Send>,
    ) -> Result<(Box<dyn AsyncReader>, Option<u32>)> {
        let (async_reader, child_fd) =
            self.pty_backend
                .spawn_terminal(run_command, None, quit_cb, terminal_id)?;
        Ok((async_reader, Some(child_fd as u32)))
    }
    fn clear_terminal_id(&self, terminal_id: u32) -> Result<()> {
        self.pty_backend.clear_terminal_id(terminal_id);
        Ok(())
    }
    fn cache_resizes(&mut self) {
        if self.cached_resizes.lock().unwrap().is_none() {
            *self.cached_resizes.lock().unwrap() = Some(BTreeMap::new());
        }
    }
    fn apply_cached_resizes(&mut self) {
        let mut cached_resizes = self.cached_resizes.lock().unwrap().take();
        if let Some(cached_resizes) = cached_resizes.as_mut() {
            for (terminal_id, (cols, rows, width_in_pixels, height_in_pixels)) in
                cached_resizes.iter()
            {
                let _ = self.set_terminal_size_using_terminal_id(
                    *terminal_id,
                    *cols,
                    *rows,
                    width_in_pixels.clone(),
                    height_in_pixels.clone(),
                );
            }
        }
    }
}

impl Clone for Box<dyn ServerOsApi> {
    fn clone(&self) -> Box<dyn ServerOsApi> {
        self.box_clone()
    }
}

pub fn get_server_os_input() -> Result<ServerOsInputOutput, std::io::Error> {
    Ok(ServerOsInputOutput {
        pty_backend: PtyBackendImpl::new()?,
        client_senders: Arc::new(Mutex::new(HashMap::new())),
        cached_resizes: Arc::new(Mutex::new(None)),
    })
}

use crate::pty_writer::PtyWriteInstruction;
use crate::thread_bus::ThreadSenders;

pub struct ResizeCache {
    senders: ThreadSenders,
}

impl ResizeCache {
    pub fn new(senders: ThreadSenders) -> Self {
        senders
            .send_to_pty_writer(PtyWriteInstruction::StartCachingResizes)
            .unwrap_or_else(|e| {
                log::error!("Failed to cache resizes: {}", e);
            });
        ResizeCache { senders }
    }
}

impl Drop for ResizeCache {
    fn drop(&mut self) {
        self.senders
            .send_to_pty_writer(PtyWriteInstruction::ApplyCachedResizes)
            .unwrap_or_else(|e| {
                log::error!("Failed to apply cached resizes: {}", e);
            });
    }
}

fn apply_post_command_hook(command: Vec<String>, post_hook: &Option<String>) -> Vec<String> {
    let Some(post_hook) = post_hook else {
        return command;
    };
    let stringified = command.join(" ");
    let cmd = match run_command_hook(&stringified, post_hook) {
        Ok(command) => command,
        Err(e) => {
            Err::<(), _>(anyhow!("post command discovery hook failed to run: {e}")).non_fatal();
            stringified
        },
    };
    cmd.trim()
        .split_ascii_whitespace()
        .map(|p| p.to_owned())
        .collect()
}

#[cfg(not(windows))]
fn run_command_hook(
    original_command: &str,
    hook_script: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(hook_script)
        .env("RESURRECT_COMMAND", original_command)
        .output()?;

    if !output.status.success() {
        return Err(format!("Hook failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

#[cfg(windows)]
fn run_command_hook(
    original_command: &str,
    hook_script: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("cmd")
        .arg("/C")
        .arg(hook_script)
        .env("RESURRECT_COMMAND", original_command)
        .output()?;
    if !output.status.success() {
        return Err(format!("Hook failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

#[cfg(test)]
#[path = "./unit/os_input_output_tests.rs"]
mod os_input_output_tests;
