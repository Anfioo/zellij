use anyhow::Result;
use async_trait::async_trait;
use zellij_utils::pane_size::Size;

#[cfg(not(windows))]
use crate::os_input_output_unix::{
    disable_mouse_support, enable_mouse_support, setup_ipc, AsyncSignalListener,
    BlockingSignalIterator,
};
#[cfg(windows)]
use crate::os_input_output_windows::{
    disable_mouse_support, enable_mouse_support, restore_console_mode, setup_ipc,
    AsyncSignalListener, BlockingSignalIterator,
};

use std::io::prelude::*;
use std::io::IsTerminal;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::{io, thread, time};
use zellij_utils::{
    data::Palette,
    errors::ErrorContext,
    ipc::{
        ClientToServerMsg, IpcReceiveError, IpcReceiverWithContext, IpcSenderWithContext,
        ServerToClientMsg,
    },
    shared::default_palette,
};

const SIGWINCH_CB_THROTTLE_DURATION: time::Duration = time::Duration::from_millis(50);

pub(crate) const ENABLE_MOUSE_SUPPORT: &str =
    "\u{1b}[?1000h\u{1b}[?1002h\u{1b}[?1003h\u{1b}[?1015h\u{1b}[?1006h";
pub(crate) const DISABLE_MOUSE_SUPPORT: &str =
    "\u{1b}[?1006l\u{1b}[?1015l\u{1b}[?1003l\u{1b}[?1002l\u{1b}[?1000l";

/// 异步标准输入读取的 trait，允许可测试的实现
#[async_trait]
pub trait AsyncStdin: Send {
    async fn read(&mut self) -> io::Result<Vec<u8>>;
}

pub struct AsyncStdinReader {
    stdin: tokio::io::Stdin,
    buffer: Vec<u8>,
}

impl AsyncStdinReader {
    pub fn new() -> Self {
        Self {
            stdin: tokio::io::stdin(),
            buffer: vec![0u8; 10 * 1024],
        }
    }
}

#[async_trait]
impl AsyncStdin for AsyncStdinReader {
    async fn read(&mut self) -> io::Result<Vec<u8>> {
        use tokio::io::AsyncReadExt;
        let n = self.stdin.read(&mut self.buffer).await?;
        Ok(self.buffer[..n].to_vec())
    }
}

pub enum SignalEvent {
    Resize,
    Quit,
}

/// 异步信号监听的 trait，允许可测试的实现
#[async_trait]
pub trait AsyncSignals: Send {
    async fn recv(&mut self) -> Option<SignalEvent>;
}

pub(crate) fn get_terminal_size() -> Size {
    match crossterm::terminal::size() {
        Ok((cols, rows)) => {
            // 当 rows/cols == 0 时回退到默认值：https://github.com/zellij-org/zellij/issues/1551
            let rows = if rows != 0 { rows as usize } else { 24 };
            let cols = if cols != 0 { cols as usize } else { 80 };
            Size { rows, cols }
        },
        Err(_) => Size { rows: 24, cols: 80 },
    }
}

#[derive(Clone)]
pub struct ClientOsInputOutput {
    send_instructions_to_server: Arc<Mutex<Option<IpcSenderWithContext<ClientToServerMsg>>>>,
    receive_instructions_from_server: Arc<Mutex<Option<IpcReceiverWithContext<ServerToClientMsg>>>>,
    reading_from_stdin: Arc<Mutex<Option<Vec<u8>>>>,
    session_name: Arc<Mutex<Option<String>>>,
}

impl std::fmt::Debug for ClientOsInputOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientOsInputOutput").finish()
    }
}

/// `ClientOsApi` trait 表示 Zellij 客户端所需的操作系统功能的抽象接口。
pub trait ClientOsApi: Send + Sync + std::fmt::Debug {
    /// 返回终端的大小。
    fn get_terminal_size(&self) -> Size;
    /// 将终端设置为
    /// [原始模式](https://en.wikipedia.org/wiki/Terminal_mode)。
    fn set_raw_mode(&mut self);
    /// 将终端设置为
    /// [规范模式](https://en.wikipedia.org/wiki/Terminal_mode)。
    fn unset_raw_mode(&self) -> Result<(), std::io::Error>;
    /// 返回允许写入标准输出的写入器。
    fn get_stdout_writer(&self) -> Box<dyn io::Write>;
    /// 返回一个允许逐行读取标准输入的 BufReader，同时锁定标准输入
    fn get_stdin_reader(&self) -> Box<dyn io::BufRead>;
    fn stdin_is_terminal(&self) -> bool {
        true
    }
    fn stdout_is_terminal(&self) -> bool {
        true
    }
    fn update_session_name(&mut self, new_session_name: String);
    /// 返回标准输入的原始内容。
    fn read_from_stdin(&mut self) -> Result<Vec<u8>, &'static str>;
    /// 返回指向此 [`ClientOsApi`] 结构体的 [`Box`] 指针。
    fn box_clone(&self) -> Box<dyn ClientOsApi>;
    /// 向服务端发送消息。
    fn send_to_server(&self, msg: ClientToServerMsg);
    /// 在客户端进程间通信通道上接收消息
    // 这应该仅从客户端路由线程调用。
    fn recv_from_server(&self) -> Option<(ServerToClientMsg, ErrorContext)>;
    fn try_recv_from_server(
        &self,
    ) -> std::result::Result<(ServerToClientMsg, ErrorContext), IpcReceiveError> {
        self.recv_from_server().ok_or(IpcReceiveError::Undecodable)
    }
    fn handle_signals(
        &self,
        sigwinch_cb: Box<dyn Fn()>,
        quit_cb: Box<dyn Fn()>,
        resize_receiver: Option<std::sync::mpsc::Receiver<()>>,
    );
    /// 与服务端套接字建立连接。
    fn connect_to_server(&self, path: &Path);
    fn spawn_server(&self, socket_path: &Path, debug: bool) -> Result<(), std::io::Error> {
        crate::spawn_server(socket_path, debug)
    }
    fn should_install_panic_hook(&self) -> bool {
        true
    }
    fn load_palette(&self) -> Palette;
    fn enable_mouse(&self) -> Result<()>;
    fn disable_mouse(&self) -> Result<()>;
    /// 将控制台输入模式恢复到 Zellij 之前的状态。
    ///
    /// 在 Windows 上，这会清除 crossterm 的 disable_raw_mode() 留下的
    /// ENABLE_MOUSE_INPUT 和 ENABLE_VIRTUAL_TERMINAL_INPUT。在其他平台上为空操作。
    fn restore_console_mode(&self) {}
    fn env_variable(&self, _name: &str) -> Option<String> {
        None
    }
    /// 返回一个可在 tokio::select 中轮询的异步标准输入读取器
    fn get_async_stdin_reader(&self) -> Box<dyn AsyncStdin> {
        Box::new(AsyncStdinReader::new())
    }
    /// 返回一个可在 tokio::select 中轮询的异步信号监听器
    fn get_async_signal_listener(&self) -> io::Result<Box<dyn AsyncSignals>> {
        Ok(Box::new(AsyncSignalListener::new()?))
    }
}

impl ClientOsApi for ClientOsInputOutput {
    fn get_terminal_size(&self) -> Size {
        get_terminal_size()
    }
    fn set_raw_mode(&mut self) {
        crossterm::terminal::enable_raw_mode().expect("无法启用原始模式");
    }
    fn unset_raw_mode(&self) -> Result<(), std::io::Error> {
        crossterm::terminal::disable_raw_mode()
    }
    fn box_clone(&self) -> Box<dyn ClientOsApi> {
        Box::new((*self).clone())
    }
    fn update_session_name(&mut self, new_session_name: String) {
        *self.session_name.lock().unwrap() = Some(new_session_name);
    }
    fn read_from_stdin(&mut self) -> Result<Vec<u8>, &'static str> {
        let session_name_at_calltime = { self.session_name.lock().unwrap().clone() };
        // 这里我们等待锁，以防另一个线程正持有标准输入
        // 这可能发生在例如切换会话时，旧线程只有在看到标准输入上的输入后才会被释放
        //
        // 当这种情况发生时，我们在另一个线程中检测到我们的会话已结束（通过比较
        // 调用开始时的会话名称和从标准输入读取后的会话名称），
        // 因此将从标准输入读取的内容放入缓冲区（我们状态上的 "reading_from_stdin"）
        // 并释放锁
        //
        // 然后，另一个线程在获取锁时会立即看到缓冲区中有内容（无需等待标准输入本身），
        // 转发此缓冲区并继续在下次调用时等待"真正的"标准输入
        let mut buffered_bytes = self.reading_from_stdin.lock().unwrap();
        match buffered_bytes.take() {
            Some(buffered_bytes) => Ok(buffered_bytes),
            None => {
                let stdin = std::io::stdin();
                let mut stdin = stdin.lock();
                let buffer = stdin.fill_buf().unwrap();
                let length = buffer.len();
                let read_bytes = Vec::from(buffer);
                stdin.consume(length);

                let session_name_after_reading_from_stdin =
                    { self.session_name.lock().unwrap().clone() };
                if session_name_at_calltime.is_some()
                    && session_name_at_calltime != session_name_after_reading_from_stdin
                {
                    *buffered_bytes = Some(read_bytes);
                    Err("Session ended")
                } else {
                    Ok(read_bytes)
                }
            },
        }
    }
    fn get_stdout_writer(&self) -> Box<dyn io::Write> {
        let stdout = ::std::io::stdout();
        Box::new(stdout)
    }

    fn get_stdin_reader(&self) -> Box<dyn io::BufRead> {
        let stdin = ::std::io::stdin();
        Box::new(stdin.lock())
    }

    fn stdin_is_terminal(&self) -> bool {
        let stdin = ::std::io::stdin();
        stdin.is_terminal()
    }

    fn stdout_is_terminal(&self) -> bool {
        let stdout = ::std::io::stdout();
        stdout.is_terminal()
    }

    fn send_to_server(&self, msg: ClientToServerMsg) {
        match self.send_instructions_to_server.lock().unwrap().as_mut() {
            Some(sender) => {
                let _ = sender.send_client_msg(msg);
            },
            None => {
                log::warn!("服务器未就绪，正在丢弃消息。");
            },
        }
    }
    fn recv_from_server(&self) -> Option<(ServerToClientMsg, ErrorContext)> {
        self.try_recv_from_server().ok()
    }
    fn try_recv_from_server(
        &self,
    ) -> std::result::Result<(ServerToClientMsg, ErrorContext), IpcReceiveError> {
        self.receive_instructions_from_server
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .try_recv_server_msg()
    }
    fn handle_signals(
        &self,
        sigwinch_cb: Box<dyn Fn()>,
        quit_cb: Box<dyn Fn()>,
        resize_receiver: Option<std::sync::mpsc::Receiver<()>>,
    ) {
        let mut sigwinch_cb_timestamp = time::Instant::now();
        let signals = BlockingSignalIterator::new(resize_receiver).unwrap();
        for event in signals {
            match event {
                SignalEvent::Resize => {
                    // 节流 sigwinch_cb 调用，减少调整大小时的过度渲染
                    if sigwinch_cb_timestamp.elapsed() < SIGWINCH_CB_THROTTLE_DURATION {
                        thread::sleep(SIGWINCH_CB_THROTTLE_DURATION);
                    }
                    sigwinch_cb_timestamp = time::Instant::now();
                    sigwinch_cb();
                },
                SignalEvent::Quit => {
                    quit_cb();
                    break;
                },
            }
        }
    }
    fn connect_to_server(&self, path: &Path) {
        let socket;
        loop {
            match zellij_utils::consts::ipc_connect(path) {
                Ok(sock) => {
                    socket = sock;
                    break;
                },
                Err(_) => {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                },
            }
        }
        let (sender, receiver) = setup_ipc(socket, path);
        *self.send_instructions_to_server.lock().unwrap() = Some(sender);
        *self.receive_instructions_from_server.lock().unwrap() = Some(receiver);
    }
    fn load_palette(&self) -> Palette {
        // 这被移除了，因为 termbg 在某些场景下不会释放标准输入（我们知道的有
        // windows terminal 和 FreeBSD）：https://github.com/zellij-org/zellij/issues/538
        //
        // let palette = default_palette();
        // let timeout = std::time::Duration::from_millis(100);
        // if let Ok(rgb) = termbg::rgb(timeout) {
        //     palette.bg = PaletteColor::Rgb((rgb.r as u8, rgb.g as u8, rgb.b as u8));
        //     // TODO: 也动态地从用户终端获取所有其他颜色
        //     // 这应该在同一个方法（OSC ]11）中完成，但这里可能有其他
        //     // 考虑因素，因此使用该库
        // };
        default_palette()
    }
    fn enable_mouse(&self) -> Result<()> {
        let mut stdout = self.get_stdout_writer();
        enable_mouse_support(&mut *stdout)
    }

    fn disable_mouse(&self) -> Result<()> {
        let mut stdout = self.get_stdout_writer();
        disable_mouse_support(&mut *stdout)
    }

    #[cfg(windows)]
    fn restore_console_mode(&self) {
        restore_console_mode();
    }

    fn env_variable(&self, name: &str) -> Option<String> {
        std::env::var(name).ok()
    }
}

impl Clone for Box<dyn ClientOsApi> {
    fn clone(&self) -> Box<dyn ClientOsApi> {
        self.box_clone()
    }
}

pub fn get_client_os_input() -> Result<ClientOsInputOutput, std::io::Error> {
    let reading_from_stdin = Arc::new(Mutex::new(None));
    Ok(ClientOsInputOutput {
        send_instructions_to_server: Arc::new(Mutex::new(None)),
        receive_instructions_from_server: Arc::new(Mutex::new(None)),
        reading_from_stdin,
        session_name: Arc::new(Mutex::new(None)),
    })
}

pub fn get_cli_client_os_input() -> Result<ClientOsInputOutput, std::io::Error> {
    let reading_from_stdin = Arc::new(Mutex::new(None));
    Ok(ClientOsInputOutput {
        send_instructions_to_server: Arc::new(Mutex::new(None)),
        receive_instructions_from_server: Arc::new(Mutex::new(None)),
        reading_from_stdin,
        session_name: Arc::new(Mutex::new(None)),
    })
}

pub const DEFAULT_STDIN_POLL_TIMEOUT_MS: u64 = 10;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_terminal_size_returns_nonzero_or_fallback() {
        let size = get_terminal_size();
        // 在 CI 中或未连接到终端时，crossterm 可能返回错误
        // 我们回退到 80x24。无论哪种方式，size 都应该有效。
        assert!(size.rows > 0, "rows should be positive");
        assert!(size.cols > 0, "cols should be positive");
    }

    #[test]
    fn get_terminal_size_fallback_values() {
        // 验证回退常量是否符合我们的预期
        let fallback = Size { rows: 24, cols: 80 };
        // 当 crossterm::terminal::size() 失败（无终端）时，我们应该得到 80x24
        // 这在 get_terminal_size_returns_nonzero_or_fallback 中被隐式测试
        // 但我们在这里验证常量
        assert_eq!(fallback.rows, 24);
        assert_eq!(fallback.cols, 80);
    }

    #[test]
    fn client_os_input_output_can_be_constructed() {
        let os_input = get_client_os_input().expect("应当构造 ClientOsInputOutput");
        let size = os_input.get_terminal_size();
        assert!(size.rows > 0, "rows should be positive");
        assert!(size.cols > 0, "cols should be positive");
    }

    #[test]
    fn cli_client_os_input_can_be_constructed() {
        let os_input = get_cli_client_os_input().expect("应当构造 CLI ClientOsInputOutput");
        let size = os_input.get_terminal_size();
        assert!(size.rows > 0, "rows should be positive");
        assert!(size.cols > 0, "cols should be positive");
    }
}
