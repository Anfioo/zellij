pub mod os_input_output;

#[cfg(not(windows))]
#[path = "os_input_output_unix.rs"]
mod os_input_output_unix;
#[cfg(windows)]
#[path = "os_input_output_windows.rs"]
mod os_input_output_windows;

pub mod cli_client;
mod command_is_executing;
mod input_handler;
mod keyboard_parser;
mod nested_reannounce;
#[cfg(feature = "web_server_capability")]
pub mod remote_attach;
mod stdin_ansi_parser;
mod stdin_handler;
#[cfg(windows)]
mod stdin_handler_windows;
#[cfg(feature = "web_server_capability")]
pub mod web_client;

use log::info;
use std::env::current_exe;
use std::io::{self, Write};
use std::net::{IpAddr, Ipv4Addr};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use zellij_utils::errors::FatalError;
use zellij_utils::shared::web_server_base_url;

#[cfg(feature = "web_server_capability")]
use futures_util::{SinkExt, StreamExt};
#[cfg(feature = "web_server_capability")]
use tokio_tungstenite::tungstenite::Message;

#[cfg(feature = "web_server_capability")]
use crate::web_client::control_message::{
    WebClientToWebServerControlMessage, WebClientToWebServerControlMessagePayload,
    WebServerToWebClientControlMessage,
};

#[cfg(feature = "web_server_capability")]
static ASYNC_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
#[cfg(feature = "web_server_capability")]
use std::sync::OnceLock;

const ENTER_ALTERNATE_SCREEN: &str = "\u{1b}[?1049h";
const EXIT_ALTERNATE_SCREEN: &str = "\u{1b}[?1049l";
const ENABLE_BRACKETED_PASTE: &str = "\u{1b}[?2004h";
const ENABLE_FOCUS_REPORTING: &str = "\u{1b}[?1004h";
const DISABLE_FOCUS_REPORTING: &str = "\u{1b}[?1004l";
const RESET_STYLE: &str = "\u{1b}[m";
const SHOW_CURSOR: &str = "\u{1b}[?25h";
const ENTER_KITTY_KEYBOARD_MODE: &str = "\u{1b}[>1u";
const EXIT_KITTY_KEYBOARD_MODE: &str = "\u{1b}[<1u";
const CLEAR_CLIENT_TERMINAL_ATTRIBUTES: &str = "\u{1b}[?1l\u{1b}=\u{1b}[r\u{1b}[?1000l\u{1b}[?1002l\u{1b}[?1003l\u{1b}[?1005l\u{1b}[?1006l\u{1b}[?12l";
/// 订阅主机调色板主题通知（CSI 2031）。支持它的主机在发送此命令后，
/// 会在主题更改时开始发出主动的 DSR 997 报告。
const ENABLE_HOST_THEME_NOTIFY: &str = "\u{1b}[?2031h";
/// 取消 CSI 2031 订阅（在分离/关闭时发送，以免让主机向空处发出 DSR 997）。
const DISABLE_HOST_THEME_NOTIFY: &str = "\u{1b}[?2031l";
/// 主动查询当前主机主题（DSR 996）。回复以与主动通知相同的
/// `CSI ? 997 ; {1|2} n` 形式到达，因此标准输入解析器统一处理两者。
const QUERY_HOST_THEME: &str = "\u{1b}[?996n";

/// 为此客户端实例生成一个异步运行时。
///
/// 工作线程数可以配置为任何非零值。传入零或 `None` 将为当前机器上的
/// 每个物理 CPU 生成一个工作线程。
#[cfg(feature = "web_server_capability")]
pub(crate) fn async_runtime(maybe_number_of_workers: Option<usize>) -> tokio::runtime::Handle {
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => handle.clone(),
        _ => {
            let number_of_workers = match maybe_number_of_workers {
                Some(value) if value > 0 => {
                    log::debug!(
                        "Creating client async runtime with {} tasks based on user request",
                        value
                    );
                    value
                },
                _ => {
                    let cpus = num_cpus::get_physical();
                    log::debug!(
                        "Creating client async runtime with {} tasks based on CPU count",
                        cpus
                    );
                    cpus
                },
            };
            let runtime = ASYNC_RUNTIME.get_or_init(|| {
                tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(number_of_workers)
                    .thread_name("zellij client async-runtime")
                    .enable_all()
                    .build()
                    .expect("Failed to create tokio runtime")
            });
            runtime.handle().clone()
        },
    }
}

#[derive(Debug)]
pub enum RemoteClientError {
    InvalidAuthToken,
    SessionTokenExpired,
    Unauthorized,
    ConnectionFailed(String),
    UrlParseError(url::ParseError),
    IoError(std::io::Error),
    Other(Box<dyn std::error::Error + Send + Sync>),
}

impl std::fmt::Display for RemoteClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RemoteClientError::InvalidAuthToken => write!(f, "Invalid authentication token"),
            RemoteClientError::SessionTokenExpired => write!(f, "Session token expired"),
            RemoteClientError::Unauthorized => write!(f, "Unauthorized"),
            RemoteClientError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            RemoteClientError::UrlParseError(e) => write!(f, "Invalid URL: {}", e),
            RemoteClientError::IoError(e) => write!(f, "IO error: {}", e),
            RemoteClientError::Other(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for RemoteClientError {}

impl From<url::ParseError> for RemoteClientError {
    fn from(error: url::ParseError) -> Self {
        RemoteClientError::UrlParseError(error)
    }
}

impl From<std::io::Error> for RemoteClientError {
    fn from(error: std::io::Error) -> Self {
        RemoteClientError::IoError(error)
    }
}

use crate::stdin_ansi_parser::{AnsiStdinInstruction, StdinAnsiParser, SyncOutput};
use crate::{
    command_is_executing::CommandIsExecuting, input_handler::input_loop,
    os_input_output::ClientOsApi, stdin_handler::stdin_loop,
};
use zellij_utils::cli::CliArgs;
use zellij_utils::{
    channels::{self, ChannelWithContext, SenderWithContext},
    consts::{set_permissions, ZELLIJ_SOCK_DIR},
    data::{
        ClientId, CommandOrPlugin, ConnectToSession, KeyWithModifier, LayoutInfo, LayoutMetadata,
    },
    envs,
    errors::{ClientContext, ContextType, ErrorInstruction},
    input::{
        cli_assets::{host_terminal_env, CliAssets},
        config::Config,
        options::Options,
    },
    ipc::{ClientToServerMsg, ExitReason, IpcReceiveError, ServerToClientMsg},
    nested_session,
    pane_size::Size,
    vendored::termwiz::input::InputEvent,
};

/// 与客户端应用程序相关的指令
#[derive(Debug, Clone)]
pub(crate) enum ClientInstruction {
    Error(String),
    Render(String),
    UnblockInputThread,
    Exit(ExitReason),
    Connected,
    Log(Vec<String>),
    LogError(Vec<String>),
    SwitchSession(ConnectToSession),
    SetSynchronizedOutput(Option<SyncOutput>),
    UnblockCliPipeInput(()), // String -> 管道名称
    CliPipeOutput((), ()),   // String -> 管道名称, String -> 输出
    QueryTerminalSize,
    StartWebServer,
    #[allow(dead_code)] // 即使我们目前没有使用会话名称，这里也需要它
    RenamedSession(String), // String -> 新会话名称
    ConfigFileUpdated,
    /// 服务端要求我们将 `query_bytes` 转发到主机终端，
    /// 并将回复字节收集到由 `token` 标识的窗口中。
    ForwardQueryToHost {
        token: u32,
        query_bytes: Vec<u8>,
        resolve_async: bool,
    },
    EmitNestedSessionFrame(Vec<u8>),
}

impl From<ServerToClientMsg> for ClientInstruction {
    fn from(instruction: ServerToClientMsg) -> Self {
        match instruction {
            ServerToClientMsg::Exit { exit_reason } => ClientInstruction::Exit(exit_reason),
            ServerToClientMsg::Render { content } => ClientInstruction::Render(content),
            ServerToClientMsg::UnblockInputThread => ClientInstruction::UnblockInputThread,
            ServerToClientMsg::Connected => ClientInstruction::Connected,
            ServerToClientMsg::Log { lines } => ClientInstruction::Log(lines),
            ServerToClientMsg::LogError { lines } => ClientInstruction::LogError(lines),
            ServerToClientMsg::SwitchSession { connect_to_session } => {
                ClientInstruction::SwitchSession(connect_to_session)
            },
            ServerToClientMsg::UnblockCliPipeInput { .. } => {
                ClientInstruction::UnblockCliPipeInput(())
            },
            ServerToClientMsg::CliPipeOutput { .. } => ClientInstruction::CliPipeOutput((), ()),
            ServerToClientMsg::QueryTerminalSize => ClientInstruction::QueryTerminalSize,
            ServerToClientMsg::StartWebServer => ClientInstruction::StartWebServer,
            ServerToClientMsg::RenamedSession { name } => ClientInstruction::RenamedSession(name),
            ServerToClientMsg::ConfigFileUpdated => ClientInstruction::ConfigFileUpdated,
            ServerToClientMsg::ForwardQueryToHost {
                token,
                query_bytes,
                resolve_async,
            } => ClientInstruction::ForwardQueryToHost {
                token,
                query_bytes,
                resolve_async,
            },
            ServerToClientMsg::EmitNestedSessionFrame { payload_bytes } => {
                ClientInstruction::EmitNestedSessionFrame(payload_bytes)
            },
            // 仅订阅消息 — 常规交互式客户端不处理
            ServerToClientMsg::PaneRenderUpdate { .. } => ClientInstruction::UnblockInputThread,
            ServerToClientMsg::SubscribedPaneClosed { .. } => ClientInstruction::UnblockInputThread,
            ServerToClientMsg::SetSoftKeyboard { .. } => ClientInstruction::UnblockInputThread,
            ServerToClientMsg::MobileState { .. } => ClientInstruction::UnblockInputThread,
        }
    }
}

impl From<&ClientInstruction> for ClientContext {
    fn from(client_instruction: &ClientInstruction) -> Self {
        match *client_instruction {
            ClientInstruction::Exit(_) => ClientContext::Exit,
            ClientInstruction::Error(_) => ClientContext::Error,
            ClientInstruction::Render(_) => ClientContext::Render,
            ClientInstruction::UnblockInputThread => ClientContext::UnblockInputThread,
            ClientInstruction::Connected => ClientContext::Connected,
            ClientInstruction::Log(_) => ClientContext::Log,
            ClientInstruction::LogError(_) => ClientContext::LogError,
            ClientInstruction::SwitchSession(..) => ClientContext::SwitchSession,
            ClientInstruction::SetSynchronizedOutput(..) => ClientContext::SetSynchronisedOutput,
            ClientInstruction::UnblockCliPipeInput(..) => ClientContext::UnblockCliPipeInput,
            ClientInstruction::CliPipeOutput(..) => ClientContext::CliPipeOutput,
            ClientInstruction::QueryTerminalSize => ClientContext::QueryTerminalSize,
            ClientInstruction::StartWebServer => ClientContext::StartWebServer,
            ClientInstruction::RenamedSession(..) => ClientContext::RenamedSession,
            ClientInstruction::ConfigFileUpdated => ClientContext::ConfigFileUpdated,
            ClientInstruction::ForwardQueryToHost { .. } => ClientContext::ForwardQueryToHost,
            ClientInstruction::EmitNestedSessionFrame(..) => ClientContext::EmitNestedSessionFrame,
        }
    }
}

impl ErrorInstruction for ClientInstruction {
    fn error(err: String) -> Self {
        ClientInstruction::Error(err)
    }
}

#[cfg(all(feature = "web_server_capability", not(windows)))]
fn spawn_web_server(cli_args: &CliArgs) -> Result<String, String> {
    let mut cmd = Command::new(current_exe().map_err(|e| e.to_string())?);
    if let Some(config_file_path) = Config::config_file_path(cli_args) {
        let config_file_path_exists = Path::new(&config_file_path).exists();
        if !config_file_path_exists {
            return Err(format!(
                "Config file: {} does not exist",
                config_file_path.display()
            ));
        }
        // 这样做是为了如果 Zellij 本身是用不同的配置文件启动的，我们将使用它
        // 来启动 web 服务器
        cmd.arg("--config");
        cmd.arg(format!("{}", config_file_path.display()));
    }
    cmd.arg("web");
    cmd.arg("-d");
    let output = cmd.output();
    match output {
        Ok(output) => {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err(String::from_utf8_lossy(&output.stderr).to_string())
            }
        },
        Err(e) => Err(e.to_string()),
    }
}

/// 在 Windows 上，cmd.output() 为 stdout/stderr 创建管道句柄。子进程
/// (zellij web -d) 会生成一个孙进程（web 服务器），它继承这些管道句柄。
/// cmd.output() 等待管道上的 EOF，但长生命周期的孙进程会保持它们打开 — 永远挂起。
///
/// 将孙进程的 stdio 重定向到 null 是不够的：在 Windows 上，
/// bInheritHandles=TRUE 的 CreateProcess 会继承所有可继承句柄，
/// 而不仅仅是 STARTUPINFO 中指定的 stdio 句柄。无论孙进程的 stdio 配置如何，
/// 管道句柄都会泄漏。
///
/// 改用 cmd.status()：不创建管道，因此没有什么可挂起的。
#[cfg(all(feature = "web_server_capability", windows))]
fn spawn_web_server(cli_args: &CliArgs) -> Result<String, String> {
    let mut cmd = Command::new(current_exe().map_err(|e| e.to_string())?);
    if let Some(config_file_path) = Config::config_file_path(cli_args) {
        let config_file_path_exists = Path::new(&config_file_path).exists();
        if !config_file_path_exists {
            return Err(format!(
                "Config file: {} does not exist",
                config_file_path.display()
            ));
        }
        cmd.arg("--config");
        cmd.arg(format!("{}", config_file_path.display()));
    }
    cmd.arg("web");
    cmd.arg("-d");
    match cmd.status() {
        Ok(status) => {
            if status.success() {
                Ok(String::new())
            } else {
                Err(format!(
                    "Web server process exited with code: {}",
                    status.code().unwrap_or(-1)
                ))
            }
        },
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(not(feature = "web_server_capability"))]
fn spawn_web_server(_cli_args: &CliArgs) -> Result<String, String> {
    log::error!(
        "This version of Zellij was compiled without web server support, cannot run web server!"
    );
    Ok("".to_owned())
}

fn check_ipc_pipe_length(ipc_pipe: &Path) {
    use zellij_utils::consts::ZELLIJ_SOCK_MAX_LENGTH;
    let path_len = ipc_pipe.as_os_str().len();
    if path_len >= ZELLIJ_SOCK_MAX_LENGTH {
        eprintln!(
            "Error: the IPC socket path is too long ({} bytes, max {}):\n  {}\n\n\
             This is usually caused by a long $TMPDIR path.\n\
             To fix this, set a shorter socket directory, eg.:\n  \
             ZELLIJ_SOCKET_DIR=/tmp/zellij zellij",
            path_len,
            ZELLIJ_SOCK_MAX_LENGTH - 1,
            ipc_pipe.display()
        );
        std::process::exit(1);
    }
}

#[derive(Clone, Copy)]
struct TerminalTeardown {
    include_kitty_exit: bool,
}

fn exit_after_startup_error(teardown: Option<TerminalTeardown>, message: String) -> ! {
    log::error!("{}", message);
    match teardown {
        Some(teardown) => {
            let kitty_exit = if teardown.include_kitty_exit {
                EXIT_KITTY_KEYBOARD_MODE
            } else {
                ""
            };
            let rendered = format!(
                "{}{}{}{}{}{}\r\n{}\n",
                kitty_exit,
                DISABLE_HOST_THEME_NOTIFY,
                DISABLE_FOCUS_REPORTING,
                EXIT_ALTERNATE_SCREEN,
                RESET_STYLE,
                SHOW_CURSOR,
                message
            );
            let mut stdout = io::stdout();
            let _ = stdout.write_all(rendered.as_bytes());
            let _ = stdout.flush();
        },
        None => {
            eprintln!("{}", message);
        },
    }
    std::process::exit(1);
}

fn spawn_server_error_message(e: io::Error) -> String {
    format!(
        "Error: failed to start the Zellij server process:\n\n\
         Reason: {}\n\n\
         This can happen if the Zellij binary cannot be executed, or if the server \
         could not create its session socket - for example due to a permission issue \
         in the socket directory.\n\
         To fix a socket directory issue, set a writable socket directory, eg.:\n  \
         ZELLIJ_SOCKET_DIR=/tmp/zellij-$USER zellij",
        e
    )
}

fn create_ipc_pipe(teardown: Option<TerminalTeardown>) -> PathBuf {
    let mut sock_dir = ZELLIJ_SOCK_DIR.clone();
    if let Err(e) = std::fs::create_dir_all(&sock_dir) {
        exit_after_startup_error(
            teardown,
            format!(
                "Error: failed to create the Zellij socket directory:\n  {}\n\n\
                 Reason: {}\n\n\
                 This usually means the directory (or one of its parents) is owned by \
                 another user or is not writable - for example if Zellij was previously \
                 run with `sudo`, or if $XDG_RUNTIME_DIR points to a directory you do not \
                 own.\nTo fix this, remove or correct the offending directory, or set a \
                 writable socket directory, eg.:\n  ZELLIJ_SOCKET_DIR=/tmp/zellij-$USER zellij",
                sock_dir.display(),
                e
            ),
        );
    }
    if let Err(e) = set_permissions(&sock_dir, 0o700) {
        exit_after_startup_error(
            teardown,
            format!(
                "Error: failed to set permissions (0700) on the Zellij socket directory:\n  {}\n\n\
                 Reason: {}\n\n\
                 This usually means the directory is owned by another user.\n\
                 To fix this, remove or correct the offending directory, or set a writable \
                 socket directory, eg.:\n  ZELLIJ_SOCKET_DIR=/tmp/zellij-$USER zellij",
                sock_dir.display(),
                e
            ),
        );
    }
    sock_dir.push(envs::get_session_name().unwrap());
    check_ipc_pipe_length(&sock_dir);
    sock_dir
}

/// 生成 Zellij 服务端进程。
///
/// 在 Unix 上，服务端在 start_server() 内部进行守护进程化（双 fork），因此
/// 中间子进程立即退出，`cmd.status()` 返回。
#[cfg(not(windows))]
pub fn spawn_server(socket_path: &Path, debug: bool) -> io::Result<()> {
    let mut cmd = Command::new(current_exe()?);
    cmd.arg("--server").arg(socket_path);
    if debug {
        cmd.arg("--debug");
    }
    let status = cmd.status()?;
    if status.success() {
        Ok(())
    } else {
        let msg = "Process returned non-zero exit code";
        let err_msg = match status.code() {
            Some(c) => format!("{}: {}", msg, c),
            None => msg.to_string(),
        };
        Err(io::Error::new(io::ErrorKind::Other, err_msg))
    }
}

/// 生成 Zellij 服务端进程。
///
/// 在 Windows 上没有守护进程化 — 我们将服务端作为带有隐藏控制台的后台进程启动。
/// 我们使用 CREATE_NO_WINDOW（而不是 DETACHED_PROCESS），以便服务端获得有效的标准句柄；
/// DETACHED_PROCESS 会将 stdin/stdout/stderr 保留为 NULL，这会破坏 PTY 创建、
/// WASM 插件加载和日志记录。
#[cfg(windows)]
pub fn spawn_server(socket_path: &Path, debug: bool) -> io::Result<()> {
    use std::os::windows::process::CommandExt;
    let mut cmd = Command::new(current_exe()?);
    cmd.arg("--server").arg(socket_path);
    if debug {
        cmd.arg("--debug");
    }
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
    cmd.creation_flags(CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP);
    cmd.spawn()?;
    Ok(())
}

#[derive(Debug, Clone)]
pub enum ClientInfo {
    Attach(String, Options),
    New(
        String,
        Option<LayoutInfo>,
        Option<PathBuf>,
        Option<Vec<CommandOrPlugin>>,
    ), // PathBuf -> explicit cwd
    Resurrect(String, PathBuf, bool, Option<PathBuf>), // (名称, 布局路径, 强制运行命令, cwd)
    Watch(String, Options),                            // 监视模式（只读）
}

impl ClientInfo {
    pub fn get_session_name(&self) -> &str {
        match self {
            Self::Attach(ref name, _) => name,
            Self::New(ref name, _layout_info, _layout_cwd, _initial_panes) => name,
            Self::Resurrect(ref name, _, _, _) => name,
            Self::Watch(ref name, _) => name,
        }
    }
    pub fn set_layout_info(&mut self, new_layout_info: LayoutInfo) {
        match self {
            ClientInfo::New(_, layout_info, _, _) => *layout_info = Some(new_layout_info),
            _ => {},
        }
    }
    pub fn set_cwd(&mut self, new_cwd: PathBuf) {
        match self {
            ClientInfo::New(_, _, cwd, _) => *cwd = Some(new_cwd),
            ClientInfo::Resurrect(_, _, _, cwd) => *cwd = Some(new_cwd),
            _ => {},
        }
    }
    pub fn set_initial_panes(&mut self, new_initial_panes: Vec<CommandOrPlugin>) {
        match self {
            ClientInfo::New(_, _, _, initial_panes) => *initial_panes = Some(new_initial_panes),
            _ => {},
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum InputInstruction {
    KeyEvent(InputEvent, Vec<u8>),
    KeyWithModifierEvent(KeyWithModifier, Vec<u8>, bool), // bool = is_kitty_keyboard_protocol
    #[allow(dead_code)] // 在 stdin_handler_windows.rs 中构造（仅 Windows）
    MouseEvent(zellij_utils::input::mouse::MouseEvent),
    AnsiStdinInstructions(Vec<AnsiStdinInstruction>),
    DesktopNotificationResponse(Vec<u8>),
    /// 连续的主机回复解析器关闭了一个转发窗口（看到屏障回复或触发超时）。
    /// 有效载荷是要发送到服务端的累积原始字节。
    ForwardedReplyFromHostComplete {
        token: u32,
        reply_bytes: Vec<u8>,
    },
    NestedSessionFrameFromHost(Vec<u8>),
    HostTerminalFocusChanged(bool),
    Exit,
}

#[cfg(feature = "web_server_capability")]
pub async fn run_remote_client_terminal_loop(
    os_input: Box<dyn ClientOsApi>,
    mut connections: remote_attach::WebSocketConnections,
    nested_session_name: Option<String>,
    host_contacted: Arc<std::sync::atomic::AtomicBool>,
) -> Result<Option<ConnectToSession>, RemoteClientError> {
    use crate::os_input_output::{AsyncSignals, AsyncStdin};

    let synchronised_output = match os_input.env_variable("TERM").as_deref() {
        Some("alacritty") => Some(SyncOutput::DCS),
        _ => None,
    };

    let mut async_stdin: Box<dyn AsyncStdin> = os_input.get_async_stdin_reader();
    let mut async_signals: Box<dyn AsyncSignals> = os_input
        .get_async_signal_listener()
        .map_err(|e| RemoteClientError::IoError(e))?;

    let create_resize_message = |size: Size| {
        Message::Text(
            serde_json::to_string(&WebClientToWebServerControlMessage {
                web_client_id: connections.web_client_id.clone(),
                payload: WebClientToWebServerControlMessagePayload::TerminalResize(size),
            })
            .unwrap()
            .into(),
        )
    };

    // 启动时发送大小
    let new_size = os_input.get_terminal_size();
    if let Err(e) = connections
        .control_ws
        .send(create_resize_message(new_size))
        .await
    {
        log::error!("Failed to send resize message: {}", e);
    }

    let mut nested_frame_extractor = nested_session::NestedFrameExtractor::new();
    let mut reannounce_scheduler =
        nested_session::ReannounceScheduler::new(std::time::Instant::now());
    let mut reannounce_check = tokio::time::interval(std::time::Duration::from_millis(
        nested_session::reannounce_check_interval_ms(),
    ));
    reannounce_check.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            // 处理标准输入
            result = async_stdin.read() => {
                match result {
                    Ok(buf) if !buf.is_empty() => {
                        let (cleaned, nested_frames) = nested_frame_extractor.extract(&buf);
                        for payload_bytes in nested_frames {
                            reannounce_scheduler.note_host_contact(std::time::Instant::now());
                            host_contacted.store(true, std::sync::atomic::Ordering::Relaxed);
                            match nested_session::decode_payload(&payload_bytes) {
                                Some(nested_session::NestedSessionMessage::Ping) => {
                                    let mut stdout = os_input.get_stdout_writer();
                                    let _ = stdout.write_all(&nested_session::encode_frame(
                                        &nested_session::NestedSessionMessage::Pong,
                                    ));
                                    let _ = stdout.flush();
                                },
                                Some(_) => {
                                    let control_msg = Message::Text(
                                        serde_json::to_string(&WebClientToWebServerControlMessage {
                                            web_client_id: connections.web_client_id.clone(),
                                            payload: WebClientToWebServerControlMessagePayload::NestedSessionFrameFromHost {
                                                payload_bytes,
                                            },
                                        })
                                        .unwrap()
                                        .into(),
                                    );
                                    if let Err(e) = connections.control_ws.send(control_msg).await {
                                        log::error!("Failed to forward nested session frame over control WebSocket: {}", e);
                                    }
                                },
                                None => {},
                            }
                        }
                        if !cleaned.is_empty() {
                            if let Err(e) = connections.terminal_ws.send(Message::Binary(cleaned.into())).await {
                                log::error!("Failed to send stdin to terminal WebSocket: {}", e);
                                break;
                            }
                        }
                    }
                    Ok(_) => {
                        // 空缓冲区意味着 EOF
                        break;
                    }
                    Err(e) => {
                        log::error!("Error reading from stdin: {}", e);
                        break;
                    }
                }
            }

            _ = reannounce_check.tick() => {
                if let Some(session_name) = &nested_session_name {
                    if reannounce_scheduler.on_tick(std::time::Instant::now()) {
                        let announce = nested_session::NestedSessionMessage::Announce {
                            session_name: session_name.clone(),
                            capabilities: vec![nested_session::NestedSessionCapability::NestedControl],
                        };
                        let mut stdout = os_input.get_stdout_writer();
                        if stdout
                            .write_all(&nested_session::encode_frame(&announce))
                            .is_ok()
                        {
                            let _ = stdout.flush();
                        }
                    }
                }
            }

            // 处理信号
            Some(signal) = async_signals.recv() => {
                match signal {
                    crate::os_input_output::SignalEvent::Resize => {
                        let new_size = os_input.get_terminal_size();
                        if let Err(e) = connections.control_ws.send(create_resize_message(new_size)).await {
                            log::error!("Failed to send resize message: {}", e);
                            break;
                        }
                    }
                    crate::os_input_output::SignalEvent::Quit => {
                        break;
                    }
                }
            }

            // 处理终端消息
            terminal_msg = connections.terminal_ws.next() => {
                match terminal_msg {
                    Some(Ok(Message::Text(text))) => {
                        let mut stdout = os_input.get_stdout_writer();
                        if let Some(sync) = synchronised_output {
                            stdout
                                .write_all(sync.start_seq())
                                .expect("cannot write to stdout");
                        }
                        stdout
                            .write_all(text.as_bytes())
                            .expect("cannot write to stdout");
                        if let Some(sync) = synchronised_output {
                            stdout
                                .write_all(sync.end_seq())
                                .expect("cannot write to stdout");
                        }
                        stdout.flush().expect("could not flush");
                    }
                    Some(Ok(Message::Binary(data))) => {
                        let mut stdout = os_input.get_stdout_writer();
                        if let Some(sync) = synchronised_output {
                            stdout
                                .write_all(sync.start_seq())
                                .expect("cannot write to stdout");
                        }
                        stdout
                            .write_all(&data)
                            .expect("cannot write to stdout");
                        if let Some(sync) = synchronised_output {
                            stdout
                                .write_all(sync.end_seq())
                                .expect("cannot write to stdout");
                        }
                        stdout.flush().expect("could not flush");
                    }
                    Some(Ok(Message::Close(_))) => {
                        break;
                    }
                    Some(Err(e)) => {
                        log::error!("Error: {}", e);
                        break;
                    }
                    None => {
                        log::error!("Received empty message from web server");
                        break;
                    }
                    _ => {}
                }
            }

            control_msg = connections.control_ws.next() => {
                match control_msg {
                    Some(Ok(Message::Text(msg))) => {
                        let deserialized_msg: Result<WebServerToWebClientControlMessage, _> =
                            serde_json::from_str(&msg);
                        match deserialized_msg {
                            Ok(WebServerToWebClientControlMessage::SetConfig(..)) => {
                                // 空操作
                            }
                            Ok(WebServerToWebClientControlMessage::QueryTerminalSize) => {
                                let new_size = os_input.get_terminal_size();
                                if let Err(e) = connections.control_ws.send(create_resize_message(new_size)).await {
                                    log::error!("Failed to send resize message: {}", e);
                                }
                            }
                            Ok(WebServerToWebClientControlMessage::Log { lines }) => {
                                for line in lines {
                                    log::info!("{}", line);
                                }
                            }
                            Ok(WebServerToWebClientControlMessage::LogError { lines }) => {
                                for line in lines {
                                    log::error!("{}", line);
                                }
                            }
                            Ok(WebServerToWebClientControlMessage::SwitchedSession{ .. }) => {
                                // 空操作
                            }
                            Ok(WebServerToWebClientControlMessage::SetSoftKeyboard{ .. }) => {
                                // 空操作
                            }
                            Ok(WebServerToWebClientControlMessage::MobileState{ .. }) => {
                                // 空操作
                            }
                            Err(e) => {
                                log::debug!("Ignoring unrecognized control message: {}", e);
                            }
                        }

                    }
                    Some(Ok(Message::Close(_))) => {
                        break;
                    }
                    Some(Err(e)) => {
                        log::error!("{}", e);
                        break;
                    }
                    None => break,
                    _ => {}
                }
            }

        }
    }

    Ok(None)
}

#[cfg(feature = "web_server_capability")]
pub fn start_remote_client(
    mut os_input: Box<dyn ClientOsApi>,
    remote_session_url: &str,
    token: Option<String>,
    remember: bool,
    forget: bool,
    ca_cert: Option<std::path::PathBuf>,
    insecure: bool,
    async_worker_tasks: Option<usize>,
) -> Result<Option<ConnectToSession>, RemoteClientError> {
    info!("Starting Zellij client!");

    let remote_session_name = remote_session_url
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or_default()
        .to_owned();

    let runtime = crate::async_runtime(async_worker_tasks);

    let connections = remote_attach::attach_to_remote_session(
        runtime.clone(),
        os_input.clone(),
        remote_session_url,
        token,
        remember,
        forget,
        ca_cert.as_deref(),
        insecure,
    )?;

    let reconnect_to_session = None;
    os_input.unset_raw_mode().unwrap();

    let mut stdout = os_input.get_stdout_writer();
    stdout.write_all(ENTER_ALTERNATE_SCREEN.as_bytes()).unwrap();
    stdout
        .write_all(CLEAR_CLIENT_TERMINAL_ATTRIBUTES.as_bytes())
        .unwrap();
    stdout
        .write_all(ENTER_KITTY_KEYBOARD_MODE.as_bytes())
        .unwrap();
    stdout
        .write_all(ENABLE_HOST_THEME_NOTIFY.as_bytes())
        .unwrap();
    stdout.write_all(QUERY_HOST_THEME.as_bytes()).unwrap();

    envs::set_zellij("0".to_string());

    let full_screen_ws = os_input.get_terminal_size();

    os_input.set_raw_mode();
    stdout.write_all(ENABLE_BRACKETED_PASTE.as_bytes()).unwrap();
    stdout.write_all(ENABLE_FOCUS_REPORTING.as_bytes()).unwrap();
    let announce = nested_session::NestedSessionMessage::Announce {
        session_name: remote_session_name.clone(),
        capabilities: vec![nested_session::NestedSessionCapability::NestedControl],
    };
    stdout
        .write_all(&nested_session::encode_frame(&announce))
        .unwrap();
    let _ = stdout.flush();
    let host_contacted = Arc::new(std::sync::atomic::AtomicBool::new(false));

    std::panic::set_hook({
        use zellij_utils::errors::handle_panic;
        let os_input = os_input.clone();
        Box::new(move |info| {
            os_input.disable_mouse().non_fatal();
            os_input.restore_console_mode();
            if let Ok(()) = os_input.unset_raw_mode() {
                handle_panic::<ClientInstruction>(info, None);
            }
        })
    });

    let reset_controlling_terminal_state = |e: String, exit_status: i32| {
        os_input.disable_mouse().non_fatal();
        os_input.unset_raw_mode().unwrap();
        os_input.restore_console_mode();
        let error = terminal_teardown_message(&e, full_screen_ws.rows, true);
        let mut stdout = os_input.get_stdout_writer();
        stdout.write_all(error.as_bytes()).unwrap();
        stdout.flush().unwrap();
        if exit_status == 0 {
            log::info!("{}", e);
        } else {
            log::error!("{}", e);
        };
        std::process::exit(exit_status);
    };

    runtime.block_on(run_remote_client_terminal_loop(
        os_input.clone(),
        connections,
        Some(remote_session_name.clone()),
        host_contacted.clone(),
    ))?;

    if host_contacted.load(std::sync::atomic::Ordering::Relaxed) {
        let mut stdout = os_input.get_stdout_writer();
        let _ = stdout.write_all(&nested_session::encode_frame(
            &nested_session::NestedSessionMessage::Bye,
        ));
        let _ = stdout.flush();
    }

    let exit_msg = String::from("Bye from Zellij!");

    if reconnect_to_session.is_none() {
        reset_controlling_terminal_state(exit_msg, 0);
        std::process::exit(0);
    } else {
        let clear_screen = "\u{1b}[2J";
        let mut stdout = os_input.get_stdout_writer();
        stdout.write_all(clear_screen.as_bytes()).unwrap();
        stdout.flush().unwrap();
    }

    Ok(reconnect_to_session)
}

pub fn start_client(
    mut os_input: Box<dyn ClientOsApi>,
    cli_args: CliArgs,
    config: Config,          // 保存到磁盘（或默认？）
    config_options: Options, // CLI 选项合并到（优先级高于）保存的配置选项
    info: ClientInfo,
    tab_position_to_focus: Option<usize>,
    pane_id_to_focus: Option<(u32, bool)>, // (pane_id, is_plugin)
    is_a_reconnect: bool,
    start_detached_and_exit: bool,
) -> Option<ConnectToSession> {
    if start_detached_and_exit {
        start_server_detached(os_input, cli_args, config, config_options, info);
        return None;
    }
    info!("Starting Zellij client!");

    let own_session_name = info.get_session_name().to_owned();

    let explicitly_disable_kitty_keyboard_protocol = config_options
        .support_kitty_keyboard_protocol
        .map(|e| !e)
        .unwrap_or(false);
    let support_kitty_graphics_protocol = config_options
        .support_kitty_graphics_protocol
        .unwrap_or(true);
    let should_start_web_server = config_options.web_server.map(|w| w).unwrap_or(false);
    let mut reconnect_to_session = None;
    os_input.unset_raw_mode().unwrap();

    if !is_a_reconnect {
        // 我们不对重连执行此操作，因为我们的控制终端已经具有我们想要的属性，
        // 而且某些终端不会原子地处理这些属性（说的就是你 Windows Terminal...）
        let mut stdout = os_input.get_stdout_writer();
        stdout.write_all(ENTER_ALTERNATE_SCREEN.as_bytes()).unwrap();
        stdout
            .write_all(CLEAR_CLIENT_TERMINAL_ATTRIBUTES.as_bytes())
            .unwrap();
        if !explicitly_disable_kitty_keyboard_protocol {
            stdout
                .write_all(ENTER_KITTY_KEYBOARD_MODE.as_bytes())
                .unwrap();
        }
        // 订阅主机 CSI 2031 主题通知并查询当前模式。
        // 紧跟在 CLEAR_CLIENT_TERMINAL_ATTRIBUTES 之后发送，以便不存在主机未订阅的窗口。
        // 不支持 2031 的主机会忽略这两个序列。
        stdout
            .write_all(ENABLE_HOST_THEME_NOTIFY.as_bytes())
            .unwrap();
        stdout.write_all(QUERY_HOST_THEME.as_bytes()).unwrap();
    }
    envs::set_zellij("0".to_string());
    config.env.set_vars();

    let full_screen_ws = os_input.get_terminal_size();

    let web_server_ip = config_options
        .web_server_ip
        .unwrap_or_else(|| IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    let web_server_port = config_options.web_server_port.unwrap_or_else(|| 8082);
    let has_certificate =
        config_options.web_server_cert.is_some() && config_options.web_server_key.is_some();
    let enforce_https_for_localhost = config_options.enforce_https_for_localhost.unwrap_or(false);

    let terminal_teardown = TerminalTeardown {
        include_kitty_exit: !explicitly_disable_kitty_keyboard_protocol,
    };

    let (first_msg, ipc_pipe) = match info {
        ClientInfo::Attach(name, config_options) => {
            envs::set_session_name(name.clone());
            os_input.update_session_name(name);
            let ipc_pipe = create_ipc_pipe(Some(terminal_teardown));
            let is_web_client = false;

            let cli_assets = CliAssets {
                config_file_path: Config::config_file_path(&cli_args),
                config_dir: cli_args.config_dir.clone(),
                should_ignore_config: cli_args.is_setup_clean(),
                configuration_options: Some(config_options.clone()),
                layout: if let Some(layout_string) = &cli_args.layout_string {
                    Some(LayoutInfo::Stringified(layout_string.clone()))
                } else {
                    cli_args
                        .layout
                        .as_ref()
                        .and_then(|l| {
                            LayoutInfo::from_cli(
                                &config_options.layout_dir,
                                &Some(l.clone()),
                                std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
                            )
                        })
                        .or_else(|| {
                            LayoutInfo::from_config(
                                &config_options.layout_dir,
                                &config_options.default_layout,
                            )
                        })
                },
                terminal_window_size: full_screen_ws,
                data_dir: cli_args.data_dir.clone(),
                is_debug: cli_args.debug,
                max_panes: cli_args.max_panes,
                force_run_layout_commands: false,
                cwd: None,
                host_terminal_env: host_terminal_env(),
                initial_panes: None,
            };
            (
                ClientToServerMsg::AttachClient {
                    cli_assets,
                    tab_position_to_focus,
                    pane_to_focus: pane_id_to_focus.map(|(pane_id, is_plugin)| {
                        zellij_utils::ipc::PaneReference { pane_id, is_plugin }
                    }),
                    is_web_client,
                },
                ipc_pipe,
            )
        },
        ClientInfo::Watch(name, _config_options) => {
            envs::set_session_name(name.clone());
            os_input.update_session_name(name);
            let ipc_pipe = create_ipc_pipe(Some(terminal_teardown));
            let is_web_client = false;

            (
                ClientToServerMsg::AttachWatcherClient {
                    terminal_size: full_screen_ws,
                    is_web_client,
                },
                ipc_pipe,
            )
        },
        ClientInfo::Resurrect(name, path_to_layout, force_run_commands, cwd) => {
            envs::set_session_name(name.clone());

            let cli_assets = CliAssets {
                config_file_path: Config::config_file_path(&cli_args),
                config_dir: cli_args.config_dir.clone(),
                should_ignore_config: cli_args.is_setup_clean(),
                configuration_options: Some(config_options.clone()),
                layout: Some(LayoutInfo::File(
                    path_to_layout.display().to_string(),
                    LayoutMetadata::default(),
                )),
                terminal_window_size: full_screen_ws,
                data_dir: cli_args.data_dir.clone(),
                is_debug: cli_args.debug,
                max_panes: cli_args.max_panes,
                force_run_layout_commands: force_run_commands,
                cwd,
                host_terminal_env: host_terminal_env(),
                initial_panes: None,
            };

            os_input.update_session_name(name);
            let ipc_pipe = create_ipc_pipe(Some(terminal_teardown));

            if let Err(e) = os_input.spawn_server(&*ipc_pipe, cli_args.debug) {
                exit_after_startup_error(Some(terminal_teardown), spawn_server_error_message(e));
            }
            if should_start_web_server {
                if let Err(e) = spawn_web_server(&cli_args) {
                    log::error!("Failed to start web server: {}", e);
                }
            }

            let is_web_client = false;

            (
                ClientToServerMsg::FirstClientConnected {
                    cli_assets,
                    is_web_client,
                },
                ipc_pipe,
            )
        },
        ClientInfo::New(name, layout_info, layout_cwd, initial_panes) => {
            envs::set_session_name(name.clone());

            let cli_assets = CliAssets {
                config_file_path: Config::config_file_path(&cli_args),
                config_dir: cli_args.config_dir.clone(),
                should_ignore_config: cli_args.is_setup_clean(),
                configuration_options: Some(config_options.clone()),
                layout: layout_info.or_else(|| {
                    cli_args
                        .layout
                        .as_ref()
                        .and_then(|l| {
                            LayoutInfo::from_cli(
                                &config_options.layout_dir,
                                &Some(l.clone()),
                                std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
                            )
                        })
                        .or_else(|| {
                            LayoutInfo::from_config(
                                &config_options.layout_dir,
                                &config_options.default_layout,
                            )
                        })
                }),
                terminal_window_size: full_screen_ws,
                data_dir: cli_args.data_dir.clone(),
                is_debug: cli_args.debug,
                max_panes: cli_args.max_panes,
                force_run_layout_commands: false,
                cwd: layout_cwd,
                host_terminal_env: host_terminal_env(),
                initial_panes,
            };

            os_input.update_session_name(name);
            let ipc_pipe = create_ipc_pipe(Some(terminal_teardown));

            if let Err(e) = os_input.spawn_server(&*ipc_pipe, cli_args.debug) {
                exit_after_startup_error(Some(terminal_teardown), spawn_server_error_message(e));
            }
            if should_start_web_server {
                if let Err(e) = spawn_web_server(&cli_args) {
                    log::error!("Failed to start web server: {}", e);
                }
            }

            let is_web_client = false;

            (
                ClientToServerMsg::FirstClientConnected {
                    cli_assets,
                    is_web_client,
                },
                ipc_pipe,
            )
        },
    };

    os_input.connect_to_server(&*ipc_pipe);
    os_input.send_to_server(first_msg);

    let mut command_is_executing = CommandIsExecuting::new();

    os_input.set_raw_mode();
    let mut stdout = os_input.get_stdout_writer();
    stdout.write_all(ENABLE_BRACKETED_PASTE.as_bytes()).unwrap();
    stdout.write_all(ENABLE_FOCUS_REPORTING.as_bytes()).unwrap();
    let announce = nested_session::NestedSessionMessage::Announce {
        session_name: own_session_name.clone(),
        capabilities: vec![nested_session::NestedSessionCapability::NestedControl],
    };
    stdout
        .write_all(&nested_session::encode_frame(&announce))
        .unwrap();
    let _ = stdout.flush();
    let nested_reannounce = crate::nested_reannounce::NestedReannounce::spawn(
        os_input.clone(),
        own_session_name.clone(),
    );

    let (send_client_instructions, receive_client_instructions): ChannelWithContext<
        ClientInstruction,
    > = channels::bounded(50);
    let send_client_instructions = SenderWithContext::new(send_client_instructions);

    let (send_input_instructions, receive_input_instructions): ChannelWithContext<
        InputInstruction,
    > = channels::bounded(50);
    let send_input_instructions = SenderWithContext::new(send_input_instructions);

    if os_input.should_install_panic_hook() {
        std::panic::set_hook({
            use zellij_utils::errors::handle_panic;
            let send_client_instructions = send_client_instructions.clone();
            let os_input = os_input.clone();
            Box::new(move |info| {
                os_input.disable_mouse().non_fatal();
                os_input.restore_console_mode();
                if let Ok(()) = os_input.unset_raw_mode() {
                    handle_panic(info, Some(&send_client_instructions));
                }
            })
        });
    }

    let on_force_close = config_options.on_force_close.unwrap_or_default();
    let stdin_ansi_parser = Arc::new(Mutex::new(StdinAnsiParser::new()));

    let (resize_sender, resize_receiver) = std::sync::mpsc::channel::<()>();

    let _stdin_thread = thread::Builder::new()
        .name("stdin_handler".to_string())
        .spawn({
            let os_input = os_input.clone();
            let send_input_instructions = send_input_instructions.clone();
            let stdin_ansi_parser = stdin_ansi_parser.clone();
            move || {
                stdin_loop(
                    os_input,
                    send_input_instructions,
                    stdin_ansi_parser,
                    explicitly_disable_kitty_keyboard_protocol,
                    support_kitty_graphics_protocol,
                    Some(resize_sender),
                )
            }
        });

    // 在 Zellij 窗格内运行的应用程序可以向主机终端发出一组白名单查询
    // （背景/前景颜色、调色板寄存器、窗口像素尺寸）。每个查询在客户端上
    // 打开一个"转发槽"：我们将查询 + Primary-DA 屏障写入标准输出，然后收集
    // 到达标准输入的任何回复字节，直到屏障回复关闭该槽。
    // 发起查询的窗格会将捕获的字节通过管道传输到其 PTY。
    //
    // 如果主机从不回复，我们无论如何都必须关闭该槽，以便服务端可以分派
    // 下一个排队的转发。每个槽的计时器任务强制执行该截止时间：打开转发会在
    // `forward_timeout_runtime()` 上生成一个异步睡眠；唤醒时它会尝试关闭该特定
    // 令牌的槽。如果屏障（或后续转发）先关闭了槽，计时器的关闭调用就是空操作 —
    // 令牌保护使隐式取消。生成位置：下面的
    // `ClientInstruction::ForwardQueryToHost` 处理器。

    let _input_thread = thread::Builder::new()
        .name("input_handler".to_string())
        .spawn({
            let send_client_instructions = send_client_instructions.clone();
            let command_is_executing = command_is_executing.clone();
            let os_input = os_input.clone();
            let default_mode = config_options.default_mode.unwrap_or_default();
            let nested_reannounce = nested_reannounce.clone();
            move || {
                input_loop(
                    os_input,
                    config,
                    config_options,
                    command_is_executing,
                    send_client_instructions,
                    default_mode,
                    receive_input_instructions,
                    nested_reannounce,
                )
            }
        });

    let _signal_thread = thread::Builder::new()
        .name("signal_listener".to_string())
        .spawn({
            let os_input = os_input.clone();
            move || {
                os_input.handle_signals(
                    Box::new({
                        let os_api = os_input.clone();
                        move || {
                            os_api.send_to_server(ClientToServerMsg::TerminalResize {
                                new_size: os_api.get_terminal_size(),
                            });
                            #[cfg(not(windows))]
                            let _ = os_api
                                .get_stdout_writer()
                                .write(crate::stdin_handler::PIXEL_SIZE_QUERY.as_bytes());
                        }
                    }),
                    Box::new({
                        let os_api = os_input.clone();
                        move || {
                            os_api.send_to_server(ClientToServerMsg::Action {
                                action: on_force_close.into(),
                                terminal_id: None,
                                client_id: None,
                                is_cli_client: false,
                            });
                        }
                    }),
                    Some(resize_receiver),
                );
            }
        })
        .unwrap();

    let router_thread = thread::Builder::new()
        .name("router".to_string())
        .spawn({
            let os_input = os_input.clone();
            let mut should_break = false;
            let mut consecutive_unknown_messages_received = 0;
            move || loop {
                match os_input.try_recv_from_server() {
                    Ok((instruction, err_ctx)) => {
                        consecutive_unknown_messages_received = 0;
                        err_ctx.update_thread_ctx();
                        if let ServerToClientMsg::Exit { .. } = instruction {
                            should_break = true;
                        }
                        send_client_instructions.send(instruction.into()).unwrap();
                        if should_break {
                            break;
                        }
                    },
                    Err(IpcReceiveError::Disconnected) => {
                        log::error!("Lost connection to the Zellij server");
                        send_client_instructions
                            .send(ClientInstruction::UnblockInputThread)
                            .unwrap();
                        send_client_instructions
                            .send(ClientInstruction::Error(
                                "Lost connection to the Zellij server".to_string(),
                            ))
                            .unwrap();
                        break;
                    },
                    Err(IpcReceiveError::Undecodable) => {
                        consecutive_unknown_messages_received += 1;
                        send_client_instructions
                            .send(ClientInstruction::UnblockInputThread)
                            .unwrap();
                        if consecutive_unknown_messages_received == 1 {
                            log::error!("Received unknown message from server");
                        }
                        if consecutive_unknown_messages_received >= 1000 {
                            send_client_instructions
                                .send(ClientInstruction::Error(
                                    "Received empty unknown from server".to_string(),
                                ))
                                .unwrap();
                            break;
                        }
                    },
                }
            }
        })
        .unwrap();

    let handle_error = |backtrace: String| {
        os_input.disable_mouse().non_fatal();
        os_input.unset_raw_mode().unwrap();
        os_input.restore_console_mode();
        let error = terminal_teardown_message(
            &backtrace,
            full_screen_ws.rows,
            !explicitly_disable_kitty_keyboard_protocol,
        );
        let mut stdout = os_input.get_stdout_writer();
        stdout.write_all(error.as_bytes()).unwrap();
        stdout.flush().unwrap();
        std::process::exit(1);
    };

    let mut exit_msg = String::new();
    let mut synchronised_output = match os_input.env_variable("TERM").as_deref() {
        Some("alacritty") => Some(SyncOutput::DCS),
        _ => None,
    };

    loop {
        let (client_instruction, mut err_ctx) = receive_client_instructions
            .recv()
            .expect("failed to receive app instruction on channel");

        err_ctx.add_call(ContextType::Client((&client_instruction).into()));

        match client_instruction {
            ClientInstruction::Exit(reason) => {
                os_input.send_to_server(ClientToServerMsg::ClientExited);

                if let ExitReason::Error(_) = reason {
                    handle_error(reason.to_string());
                }
                exit_msg = reason.to_string();
                break;
            },
            ClientInstruction::Error(backtrace) => {
                handle_error(backtrace);
            },
            ClientInstruction::Render(output) => {
                let mut stdout = os_input.get_stdout_writer();
                if let Some(sync) = synchronised_output {
                    stdout
                        .write_all(sync.start_seq())
                        .expect("cannot write to stdout");
                }
                stdout
                    .write_all(output.as_bytes())
                    .expect("cannot write to stdout");
                if let Some(sync) = synchronised_output {
                    stdout
                        .write_all(sync.end_seq())
                        .expect("cannot write to stdout");
                }
                stdout.flush().expect("could not flush");
            },
            ClientInstruction::UnblockInputThread => {
                command_is_executing.unblock_input_thread();
            },
            ClientInstruction::Log(lines_to_log) => {
                for line in lines_to_log {
                    log::info!("{line}");
                }
            },
            ClientInstruction::LogError(lines_to_log) => {
                for line in lines_to_log {
                    log::error!("{line}");
                }
            },
            ClientInstruction::SwitchSession(connect_to_session) => {
                reconnect_to_session = Some(connect_to_session);
                os_input.send_to_server(ClientToServerMsg::ClientExited);
                break;
            },
            ClientInstruction::SetSynchronizedOutput(enabled) => {
                synchronised_output = enabled;
            },
            ClientInstruction::QueryTerminalSize => {
                os_input.send_to_server(ClientToServerMsg::TerminalResize {
                    new_size: os_input.get_terminal_size(),
                });
            },
            ClientInstruction::StartWebServer => {
                let web_server_base_url = web_server_base_url(
                    web_server_ip,
                    web_server_port,
                    has_certificate,
                    enforce_https_for_localhost,
                );
                match spawn_web_server(&cli_args) {
                    Ok(_) => {
                        let _ = os_input.send_to_server(ClientToServerMsg::WebServerStarted {
                            base_url: web_server_base_url,
                        });
                    },
                    Err(e) => {
                        log::error!("Failed to start web_server: {}", e);
                        let _ = os_input
                            .send_to_server(ClientToServerMsg::FailedToStartWebServer { error: e });
                    },
                }
            },
            ClientInstruction::ForwardQueryToHost {
                token,
                query_bytes,
                resolve_async: true,
            } => {
                {
                    let mut stdin_ansi_parser = stdin_ansi_parser.lock().unwrap();
                    if let Some((stale_token, stale_reply_bytes)) =
                        stdin_ansi_parser.take_active_clipboard_forward()
                    {
                        log::warn!(
                            "clipboard forward slot for token {} was still open when token {} was \
                             dispatched ({} accumulated bytes); closing it out",
                            stale_token,
                            token,
                            stale_reply_bytes.len(),
                        );
                        let _ = send_input_instructions.send(
                            InputInstruction::ForwardedReplyFromHostComplete {
                                token: stale_token,
                                reply_bytes: stale_reply_bytes,
                            },
                        );
                    }
                    stdin_ansi_parser.open_clipboard_forward(token);
                }
                let runtime = stdin_ansi_parser::forward_timeout_runtime();
                let parser_for_timer = stdin_ansi_parser.clone();
                let sender_for_timer = send_input_instructions.clone();
                stdin_ansi_parser::schedule_clipboard_forward_timeout(
                    runtime.handle(),
                    parser_for_timer,
                    token,
                    std::time::Duration::from_millis(
                        stdin_ansi_parser::CLIENT_CLIPBOARD_FORWARD_TIMEOUT_MS,
                    ),
                    move |token, reply_bytes| {
                        let _ = sender_for_timer.send(
                            InputInstruction::ForwardedReplyFromHostComplete { token, reply_bytes },
                        );
                    },
                );
                let mut out = os_input.get_stdout_writer();
                let _ = out.write_all(&query_bytes);
                let _ = out.flush();
            },
            ClientInstruction::ForwardQueryToHost {
                token, query_bytes, ..
            } => {
                // 1. 在解析器上打开一个转发窗口，以便在屏障之前到达的任何回复
                //    事件都被捕获。这里可能仍有一个槽处于打开状态：服务端的回退
                //    超时可能会放弃前一个令牌，并在此客户端的每槽计时器运行之前
                //    分派此令牌。移交该槽而不是破坏它。
                let stale_forward = {
                    let mut stdin_ansi_parser = stdin_ansi_parser.lock().unwrap();
                    let stale_forward = stdin_ansi_parser.take_active_forward();
                    stdin_ansi_parser.open_forward(token);
                    stale_forward
                };
                if let Some((stale_token, stale_reply_bytes)) = stale_forward {
                    log::warn!(
                        "forward slot for token {} was still open when token {} was dispatched \
                         ({} accumulated bytes); closing it out",
                        stale_token,
                        token,
                        stale_reply_bytes.len(),
                    );
                    let _ = send_input_instructions.send(
                        InputInstruction::ForwardedReplyFromHostComplete {
                            token: stale_token,
                            reply_bytes: stale_reply_bytes,
                        },
                    );
                }
                // 2. 在专用异步运行时上生成每转发计时器。当截止时间触发时，
                //    任务关闭该槽（如果它仍为此令牌打开）并中继
                //    `ForwardedReplyFromHostComplete`，以便服务端释放
                //    `forward_in_flight` 并分派下一个排队的转发。
                let runtime = stdin_ansi_parser::forward_timeout_runtime();
                let parser_for_timer = stdin_ansi_parser.clone();
                let sender_for_timer = send_input_instructions.clone();
                stdin_ansi_parser::schedule_forward_timeout(
                    runtime.handle(),
                    parser_for_timer,
                    token,
                    std::time::Duration::from_millis(500),
                    move |token, reply_bytes| {
                        let _ = sender_for_timer.send(
                            InputInstruction::ForwardedReplyFromHostComplete { token, reply_bytes },
                        );
                    },
                );
                // 3. 在单个 write_all 中写入查询 + Primary-DA 屏障。
                //    当屏障的回复到达时，它会关闭解析器端的窗口 — 计时器任务
                //    最终唤醒时会发现此令牌的空槽并执行空操作。
                let mut blob = query_bytes;
                blob.extend_from_slice(b"\x1b[c");
                let mut out = os_input.get_stdout_writer();
                let _ = out.write_all(&blob);
                let _ = out.flush();
            },
            ClientInstruction::EmitNestedSessionFrame(payload_bytes) => {
                if nested_reannounce.host_contacted() {
                    let frame = nested_session::encode_frame_from_payload(&payload_bytes);
                    let mut out = os_input.get_stdout_writer();
                    let _ = out.write_all(&frame);
                    let _ = out.flush();
                }
            },
            _ => {},
        }
    }

    nested_reannounce.stop();

    router_thread.join().unwrap();

    if nested_reannounce.host_contacted() {
        let mut stdout = os_input.get_stdout_writer();
        let _ = stdout.write_all(&nested_session::encode_frame(
            &nested_session::NestedSessionMessage::Bye,
        ));
        let _ = stdout.flush();
    }

    if reconnect_to_session.is_none() {
        let goodbye_message = terminal_teardown_message(
            &exit_msg,
            full_screen_ws.rows,
            !explicitly_disable_kitty_keyboard_protocol,
        );

        os_input.disable_mouse().non_fatal();
        info!("{}", exit_msg);
        os_input.unset_raw_mode().unwrap();
        os_input.restore_console_mode();
        let mut stdout = os_input.get_stdout_writer();
        stdout.write_all(goodbye_message.as_bytes()).unwrap();
        stdout.flush().unwrap();
    } else {
        let clear_screen = "\u{1b}[2J";
        let mut stdout = os_input.get_stdout_writer();
        stdout.write_all(clear_screen.as_bytes()).unwrap();
        stdout.flush().unwrap();
    }

    let _ = send_input_instructions.send(InputInstruction::Exit);

    reconnect_to_session
}

pub fn start_server_detached(
    mut os_input: Box<dyn ClientOsApi>,
    cli_args: CliArgs,
    config: Config,
    config_options: Options,
    info: ClientInfo,
) {
    envs::set_zellij("0".to_string());
    config.env.set_vars();

    let should_start_web_server = config_options.web_server.map(|w| w).unwrap_or(false);

    let (first_msg, ipc_pipe) = match info {
        ClientInfo::Resurrect(name, path_to_layout, force_run_commands, cwd) => {
            envs::set_session_name(name.clone());

            let cli_assets = CliAssets {
                config_file_path: Config::config_file_path(&cli_args),
                config_dir: cli_args.config_dir.clone(),
                should_ignore_config: cli_args.is_setup_clean(),
                configuration_options: Some(config_options.clone()),
                layout: Some(LayoutInfo::File(
                    path_to_layout.display().to_string(),
                    LayoutMetadata::default(),
                )),
                terminal_window_size: Size { cols: 50, rows: 50 }, // 静态数字，直到
                // 客户端连接
                data_dir: cli_args.data_dir.clone(),
                is_debug: cli_args.debug,
                max_panes: cli_args.max_panes,
                force_run_layout_commands: force_run_commands,
                cwd,
                host_terminal_env: host_terminal_env(),
                initial_panes: None,
            };

            os_input.update_session_name(name);
            let ipc_pipe = create_ipc_pipe(None);

            if let Err(e) = os_input.spawn_server(&*ipc_pipe, cli_args.debug) {
                exit_after_startup_error(None, spawn_server_error_message(e));
            }
            if should_start_web_server {
                if let Err(e) = spawn_web_server(&cli_args) {
                    log::error!("Failed to start web server: {}", e);
                }
            }

            let is_web_client = false;

            (
                ClientToServerMsg::FirstClientConnected {
                    cli_assets,
                    is_web_client,
                },
                ipc_pipe,
            )
        },
        ClientInfo::New(name, layout_info, layout_cwd, initial_panes) => {
            envs::set_session_name(name.clone());

            let cli_assets = CliAssets {
                config_file_path: Config::config_file_path(&cli_args),
                config_dir: cli_args.config_dir.clone(),
                should_ignore_config: cli_args.is_setup_clean(),
                configuration_options: cli_args.options(),
                layout: layout_info.or_else(|| {
                    cli_args
                        .layout
                        .as_ref()
                        .and_then(|l| {
                            LayoutInfo::from_cli(
                                &config_options.layout_dir,
                                &Some(l.clone()),
                                std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
                            )
                        })
                        .or_else(|| {
                            LayoutInfo::from_config(
                                &config_options.layout_dir,
                                &config_options.default_layout,
                            )
                        })
                }),
                terminal_window_size: Size { cols: 50, rows: 50 }, // 静态数字，直到
                // 客户端连接
                data_dir: cli_args.data_dir.clone(),
                is_debug: cli_args.debug,
                max_panes: cli_args.max_panes,
                force_run_layout_commands: false,
                cwd: layout_cwd,
                host_terminal_env: host_terminal_env(),
                initial_panes,
            };

            os_input.update_session_name(name);
            let ipc_pipe = create_ipc_pipe(None);

            if let Err(e) = os_input.spawn_server(&*ipc_pipe, cli_args.debug) {
                exit_after_startup_error(None, spawn_server_error_message(e));
            }
            if should_start_web_server {
                if let Err(e) = spawn_web_server(&cli_args) {
                    log::error!("Failed to start web server: {}", e);
                }
            }
            let is_web_client = false;

            (
                ClientToServerMsg::FirstClientConnected {
                    cli_assets,
                    is_web_client,
                },
                ipc_pipe,
            )
        },
        _ => {
            eprintln!("Session already exists");
            std::process::exit(1);
        },
    };

    os_input.connect_to_server(&*ipc_pipe);
    os_input.send_to_server(first_msg);
}

fn terminal_teardown_message(message: &str, rows: usize, include_kitty_exit: bool) -> String {
    let goto_start_of_last_line = format!("\u{1b}[{};{}H", rows, 1);
    let kitty_exit = if include_kitty_exit {
        EXIT_KITTY_KEYBOARD_MODE
    } else {
        ""
    };
    format!(
        "{}{}{}{}{}{}{}{}\n",
        kitty_exit,
        DISABLE_HOST_THEME_NOTIFY,
        DISABLE_FOCUS_REPORTING,
        EXIT_ALTERNATE_SCREEN,
        RESET_STYLE,
        SHOW_CURSOR,
        goto_start_of_last_line,
        message
    )
}

#[cfg(test)]
mod unit;
