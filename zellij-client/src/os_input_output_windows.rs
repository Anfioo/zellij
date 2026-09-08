use crate::os_input_output::SignalEvent;
use crate::stdin_handler_windows::restore_vt_input;

use anyhow::{Context, Result};
use async_trait::async_trait;

use std::io;
use std::io::Write;
use std::path::Path;
use zellij_utils::ipc::{IpcReceiverWithContext, IpcSenderWithContext};

/// zellij 是否应在 Windows 上使用 VT 字节路径（通过 `ReadFile` 的原始标准输入 +
/// termwiz/kitty 解析），而不是原生控制台路径（crossterm `INPUT_RECORD`）。
///
/// 当设置了以下任一环境变量时为 true：
///
/// * `TERM` — 由暴露 Unix 风格环境的终端模拟器自动设置（Alacritty、WezTerm、
///   继承 bash 环境的 WSL 会话），或由希望 zellij 使用 VT 路径的用户手动设置
///   （例如在 Windows Terminal / PowerShell 配置文件中添加 `TERM=xterm-256color`）。
/// * `WT_SESSION` — 由 Windows Terminal 本身按会话设置。检测到它意味着默认的 WT
///   安装无需用户设置 `TERM` 即可使用 VT 路径，因此他们可以获得括号粘贴、kitty 键盘、
///   SGR 鼠标以及其他依赖原始 VT 输入的功能。
///
/// 每个 VT 路径条件代码站点（标准输入处理器、鼠标模式设置、未来的输入相关代码）
/// 都应调用此函数，以便门控不会漂移 — 当它们漂移时，使用
/// `crossterm::EnableMouseCapture` 的鼠标设置会破坏标准输入路径刚刚设置的
/// `ENABLE_VIRTUAL_TERMINAL_INPUT` 标志，并破坏来自 WT 的所有输入。
pub(crate) fn use_vt_path() -> bool {
    std::env::var("TERM").is_ok() || std::env::var("WT_SESSION").is_ok()
}

/// Windows 异步信号监听器。
///
/// 以 100ms 间隔轮询 `crossterm::terminal::size()` 以获取调整大小事件，
/// 并监听 `tokio::signal::windows` 以获取 ctrl_c/ctrl_break/ctrl_close。
pub(crate) struct AsyncSignalListener {
    interval: tokio::time::Interval,
    last_size: (u16, u16),
    ctrl_c: tokio::signal::windows::CtrlC,
    ctrl_break: tokio::signal::windows::CtrlBreak,
    ctrl_close: tokio::signal::windows::CtrlClose,
}

impl AsyncSignalListener {
    pub fn new() -> io::Result<Self> {
        let size = crossterm::terminal::size().unwrap_or((80, 24));
        Ok(Self {
            interval: tokio::time::interval(std::time::Duration::from_millis(100)),
            last_size: size,
            ctrl_c: tokio::signal::windows::ctrl_c()?,
            ctrl_break: tokio::signal::windows::ctrl_break()?,
            ctrl_close: tokio::signal::windows::ctrl_close()?,
        })
    }
}

#[async_trait]
impl crate::os_input_output::AsyncSignals for AsyncSignalListener {
    async fn recv(&mut self) -> Option<SignalEvent> {
        loop {
            tokio::select! {
                _ = self.interval.tick() => {
                    if let Ok(new_size) = crossterm::terminal::size() {
                        if new_size != self.last_size {
                            self.last_size = new_size;
                            return Some(SignalEvent::Resize);
                        }
                    }
                }
                result = self.ctrl_c.recv() => {
                    return result.map(|_| SignalEvent::Quit);
                }
                result = self.ctrl_break.recv() => {
                    return result.map(|_| SignalEvent::Quit);
                }
                result = self.ctrl_close.recv() => {
                    return result.map(|_| SignalEvent::Quit);
                }
            }
        }
    }
}

/// Windows 阻塞信号迭代器。
///
/// 将 `SetConsoleCtrlHandler` 与 `AtomicBool` 一起用于退出信号。
/// 对于调整大小检测，以两种模式运行：
/// - **通道模式**：接收从标准输入线程转发的调整大小通知（该线程从 crossterm 获取
///   `Event::Resize`）。比轮询响应更快。
/// - **轮询回退**：以 50ms 间隔轮询 `crossterm::terminal::size()`。
///   在未提供接收器或发送器被丢弃时使用（VT 读取器路径）。
pub(crate) struct BlockingSignalIterator {
    last_size: (u16, u16),
    resize_receiver: Option<std::sync::mpsc::Receiver<()>>,
}

mod win_ctrl_handler {
    use std::sync::atomic::{AtomicBool, Ordering};

    use windows_sys::Win32::Foundation::BOOL;
    use windows_sys::Win32::System::Console::{CTRL_BREAK_EVENT, CTRL_CLOSE_EVENT, CTRL_C_EVENT};

    pub static CTRL_QUIT_RECEIVED: AtomicBool = AtomicBool::new(false);

    pub unsafe extern "system" fn ctrl_handler(ctrl_type: u32) -> BOOL {
        match ctrl_type {
            CTRL_C_EVENT | CTRL_BREAK_EVENT | CTRL_CLOSE_EVENT => {
                CTRL_QUIT_RECEIVED.store(true, Ordering::SeqCst);
                1 // TRUE — 已处理
            },
            _ => 0, // FALSE — 未处理
        }
    }
}

impl BlockingSignalIterator {
    pub fn new(resize_receiver: Option<std::sync::mpsc::Receiver<()>>) -> io::Result<Self> {
        use windows_sys::Win32::System::Console::SetConsoleCtrlHandler;

        win_ctrl_handler::CTRL_QUIT_RECEIVED.store(false, std::sync::atomic::Ordering::SeqCst);

        let ok = unsafe { SetConsoleCtrlHandler(Some(win_ctrl_handler::ctrl_handler), 1) };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }

        let size = crossterm::terminal::size().unwrap_or((80, 24));
        Ok(Self {
            last_size: size,
            resize_receiver,
        })
    }
}

impl Iterator for BlockingSignalIterator {
    type Item = SignalEvent;

    fn next(&mut self) -> Option<SignalEvent> {
        use std::sync::mpsc::RecvTimeoutError;
        use std::time::Duration;

        loop {
            if win_ctrl_handler::CTRL_QUIT_RECEIVED.load(std::sync::atomic::Ordering::SeqCst) {
                return Some(SignalEvent::Quit);
            }

            if let Some(ref rx) = self.resize_receiver {
                // 通道模式：原生控制台标准输入循环通过此通道发送调整大小通知。
                // 带超时阻塞，以便我们可以定期检查上面的退出标志。
                match rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(()) => return Some(SignalEvent::Resize),
                    Err(RecvTimeoutError::Timeout) => continue,
                    Err(RecvTimeoutError::Disconnected) => {
                        // 发送器被丢弃（VT 读取器路径）— 切换到轮询模式。
                        self.resize_receiver = None;
                        continue;
                    },
                }
            } else {
                // 轮询模式：在 VT 读取器路径上使用，其中不使用 crossterm 的
                // event::read()，因此调整大小事件不会通过通道传来。改为定期比较终端大小。
                if let Ok(new_size) = crossterm::terminal::size() {
                    if new_size != self.last_size {
                        self.last_size = new_size;
                        return Some(SignalEvent::Resize);
                    }
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }
}

/// 从已连接的套接字设置客户端进程间通信通道。
///
/// 在 Windows 上，我们使用两个独立的命名管道来避免 DuplicateHandle 死锁：
/// 命令管道（套接字）用于客户端→服务端，回复管道用于服务端→客户端。
pub(crate) fn setup_ipc(
    socket: interprocess::local_socket::Stream,
    path: &Path,
) -> (
    IpcSenderWithContext<zellij_utils::ipc::ClientToServerMsg>,
    IpcReceiverWithContext<zellij_utils::ipc::ServerToClientMsg>,
) {
    let reply_socket;
    loop {
        match zellij_utils::consts::ipc_connect_reply(path) {
            Ok(sock) => {
                reply_socket = sock;
                break;
            },
            Err(_) => {
                std::thread::sleep(std::time::Duration::from_millis(50));
            },
        }
    }
    let sender = IpcSenderWithContext::new(socket);
    let receiver = IpcReceiverWithContext::new(reply_socket);
    (sender, receiver)
}

/// 在标准输出上启用 ENABLE_VIRTUAL_TERMINAL_PROCESSING，以便 ConPTY 进入
/// 透传模式并将 DEC 私有模式序列（如鼠标启用）转发到终端模拟器。
/// 使用 crossterm 的安全包装器，它在内部处理 GetConsoleMode/SetConsoleMode。
fn enable_vt_processing_on_stdout() {
    crossterm::ansi_support::supports_ansi();
}

/// 在 Windows 上启用鼠标支持。
///
/// 当设置了 TERM 时，我们在 VT 输入路径上（通过 ConPTY 的终端模拟器如 Alacritty）。
/// 我们不能使用 crossterm 的 EnableMouseCapture，因为它会执行完整的 SetConsoleMode()，
/// 这会覆盖 enable_vt_input() 设置的模式，破坏 ENABLE_VIRTUAL_TERMINAL_INPUT。
///
/// 相反，我们在标准输出上启用 ENABLE_VIRTUAL_TERMINAL_PROCESSING，以便 ConPTY
/// 进入透传模式，然后写入 ANSI 鼠标启用序列。
///
/// 当未设置 TERM 时，我们在原生控制台（cmd、PowerShell、Windows Terminal）中，
/// 使用 crossterm 的 Console API 方法。
pub(crate) fn enable_mouse_support(stdout: &mut dyn Write) -> Result<()> {
    let err_context = "failed to enable mouse mode";
    if use_vt_path() {
        enable_vt_processing_on_stdout();
        stdout
            .write_all(super::os_input_output::ENABLE_MOUSE_SUPPORT.as_bytes())
            .context(err_context)?;
        stdout.flush().context(err_context)?;
    } else {
        // crossterm::execute! 需要 Sized，因此我们直接使用 std::io::stdout()
        // 而不是 trait 对象写入器。
        crossterm::execute!(std::io::stdout(), crossterm::event::EnableMouseCapture)
            .context(err_context)?;
    }
    Ok(())
}

/// 将控制台输入模式恢复到 Zellij 之前的状态。
///
/// 在 VT 路径上，`enable_vt_input()` 在控制台句柄上设置 ENABLE_MOUSE_INPUT 和
/// ENABLE_VIRTUAL_TERMINAL_INPUT，但 crossterm 的 `disable_raw_mode()` 从不清除它们。
/// 此函数恢复在设置这些标志之前保存的原始控制台模式。
pub(crate) fn restore_console_mode() {
    restore_vt_input();
}

/// 在 Windows 上禁用鼠标支持。
///
/// 有关 VT 与 Console API 路径的基本原理，请参见 `enable_mouse_support()`。
pub(crate) fn disable_mouse_support(stdout: &mut dyn Write) -> Result<()> {
    let err_context = "failed to disable mouse mode";
    if use_vt_path() {
        stdout
            .write_all(super::os_input_output::DISABLE_MOUSE_SUPPORT.as_bytes())
            .context(err_context)?;
        stdout.flush().context(err_context)?;
    } else {
        crossterm::execute!(std::io::stdout(), crossterm::event::DisableMouseCapture)
            .context(err_context)?;
    }
    Ok(())
}
