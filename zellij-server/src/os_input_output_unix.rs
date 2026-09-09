use crate::os_input_output::{command_exists, AsyncReader};
use crate::panes::PaneId;

use nix::{
    fcntl::{fcntl, FcntlArg, OFlag},
    pty::{openpty, OpenptyResult, Winsize},
    sys::{
        signal::{kill, Signal},
        termios,
    },
    unistd,
};
use tokio::io::unix::AsyncFd;

use libc::{self, ioctl, TIOCSWINSZ};
use signal_hook;
use signal_hook::consts::*;

use std::{
    collections::BTreeMap,
    fs::File,
    io,
    os::fd::{BorrowedFd, FromRawFd, IntoRawFd},
    os::unix::{
        io::{AsRawFd, RawFd},
        process::CommandExt,
    },
    process::{Child, Command},
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use zellij_utils::{errors::prelude::*, input::command::RunCommand};

pub use async_trait::async_trait;

/// 使用 `AsyncFd` 通过 epoll 包装 `RawFd` 的 `AsyncReader`。
///
/// 构造时设置 O_NONBLOCK，但将 `AsyncFd` 注册推迟到第一次 `read()` 调用，因为 `AsyncFd::new()` 需要一个活动的 Tokio 反应器，而 `spawn_terminal` 在普通的 PTY 线程上运行（在运行时之外）。
struct RawFdAsyncReader {
    /// 在反应器注册之前保存文件；提升后为 `None`。
    pending: Option<File>,
    /// 在 Tokio 运行时内的第一次 `read()` 时填充。
    async_fd: Option<AsyncFd<File>>,
}

impl RawFdAsyncReader {
    fn new(fd: RawFd) -> io::Result<Self> {
        // 设置 O_NONBLOCK 以便 AsyncFd 能正确使用 epoll
        let borrowed_fd = unsafe { BorrowedFd::borrow_raw(fd) };
        let flags = fcntl(borrowed_fd, FcntlArg::F_GETFL)
            .map_err(|e| io::Error::from_raw_os_error(e as i32))?;
        let mut oflags = OFlag::from_bits_truncate(flags);
        oflags.insert(OFlag::O_NONBLOCK);
        fcntl(borrowed_fd, FcntlArg::F_SETFL(oflags))
            .map_err(|e| io::Error::from_raw_os_error(e as i32))?;

        let file = unsafe { File::from_raw_fd(fd) };
        Ok(Self {
            pending: Some(file),
            async_fd: None,
        })
    }

    /// 在首次使用时惰性注册到 Tokio 反应器。
    fn get_async_fd(&mut self) -> io::Result<&mut AsyncFd<File>> {
        if self.async_fd.is_none() {
            let file = self
                .pending
                .take()
                .expect("RawFdAsyncReader 在初始化后被使用");
            self.async_fd = Some(AsyncFd::new(file)?);
        }
        Ok(self.async_fd.as_mut().unwrap())
    }
}

#[async_trait]
impl AsyncReader for RawFdAsyncReader {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        let async_fd = self.get_async_fd()?;
        loop {
            let mut guard = async_fd.readable().await?;
            match guard.try_io(|inner| {
                let fd = inner.get_ref().as_raw_fd();
                let ret =
                    unsafe { libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
                if ret < 0 {
                    Err(io::Error::last_os_error())
                } else {
                    Ok(ret as usize)
                }
            }) {
                Ok(result) => return result,
                Err(_would_block) => continue,
            }
        }
    }
}

fn set_terminal_size_using_fd(
    fd: RawFd,
    columns: u16,
    rows: u16,
    width_in_pixels: Option<u16>,
    height_in_pixels: Option<u16>,
) {
    // TODO: 使用 nix ioctl 来做这件事
    let ws_xpixel = width_in_pixels.unwrap_or(0);
    let ws_ypixel = height_in_pixels.unwrap_or(0);
    let winsize = Winsize {
        ws_col: columns,
        ws_row: rows,
        ws_xpixel,
        ws_ypixel,
    };
    // TIOCGWINSZ 是 u32，但在某些平台上 ioctl 的第二个参数是 u64。在 Linux 上检查时，clippy 会抱怨无用的转换。
    #[allow(clippy::useless_conversion)]
    unsafe {
        ioctl(fd, TIOCSWINSZ.into(), &winsize)
    };
}

/// 处理子进程的一些信号。这将循环直到子进程退出。
fn handle_command_exit(mut child: Child) -> Result<Option<i32>> {
    let id = child.id();
    let err_context = || {
        format!(
            "failed to handle signals and command exit for child process pid {}",
            id
        )
    };

    // 返回退出状态（如果有）
    let mut should_exit = false;
    let mut attempts = 3;
    let mut signals =
        signal_hook::iterator::Signals::new(&[SIGINT, SIGTERM]).with_context(err_context)?;
    'handle_exit: loop {
        // 测试子进程是否已退出
        match child.try_wait() {
            Ok(Some(status)) => {
                // 如果子进程已退出，跳出循环并退出此函数
                // TODO: 处理错误？
                break 'handle_exit Ok(status.code());
            },
            Ok(None) => {
                thread::sleep(Duration::from_millis(10));
            },
            Err(e) => panic!("等待时出错：{}", e),
        }

        if !should_exit {
            for signal in signals.pending() {
                if signal == SIGINT || signal == SIGTERM {
                    should_exit = true;
                }
            }
        } else if attempts > 0 {
            // 我们先友好地尝试一下...
            attempts -= 1;
            kill(
                unistd::Pid::from_raw(child.id() as i32),
                Some(Signal::SIGTERM),
            )
            .with_context(err_context)?;
            continue;
        } else {
            // 当我说停的时候，我是说真的停！
            let _ = child.kill();
            break 'handle_exit Ok(None);
        }
    }
}

fn handle_openpty(
    open_pty_res: OpenptyResult,
    cmd: RunCommand,
    quit_cb: Box<dyn Fn(PaneId, Option<i32>, RunCommand) + Send>,
    terminal_id: u32,
) -> Result<(RawFd, RawFd)> {
    let err_context = |cmd: &RunCommand| {
        format!(
            "failed to open PTY for command '{}'",
            cmd.command.to_string_lossy().to_string()
        )
    };

    // pty 的主端和子进程 fd
    let pid_primary = open_pty_res.master.into_raw_fd();
    let pid_secondary = open_pty_res.slave.into_raw_fd();

    if !command_exists(&cmd) {
        return Err(ZellijError::CommandNotFound {
            terminal_id,
            command: cmd.command.to_string_lossy().to_string(),
        })
        .with_context(|| err_context(&cmd));
    }

    let mut child = unsafe {
        let cmd = cmd.clone();
        let command = &mut Command::new(cmd.command);
        if let Some(current_dir) = cmd.cwd {
            if current_dir.exists() && current_dir.is_dir() {
                command.current_dir(current_dir);
            } else {
                log::error!(
                    "为新窗格设置工作目录失败。'{}' 不存在或不是文件夹",
                    current_dir.display()
                );
            }
        }
        command
            .args(&cmd.args)
            .env("ZELLIJ_PANE_ID", &format!("{}", terminal_id))
            .pre_exec(move || -> io::Result<()> {
                if libc::login_tty(pid_secondary) != 0 {
                    panic!("设置控制终端失败");
                }
                close_fds::close_open_fds(3, &[]);
                Ok(())
            })
            .spawn()
            .expect("failed to spawn")
    };

    let child_id = child.id();
    thread::spawn(move || {
        child.wait().with_context(|| err_context(&cmd)).fatal();
        let exit_status = handle_command_exit(child)
            .with_context(|| err_context(&cmd))
            .fatal();
        let _ = unistd::close(pid_secondary);
        quit_cb(PaneId::Terminal(terminal_id), exit_status, cmd);
    });

    Ok((pid_primary, child_id as RawFd))
}

/// 使用 [`termios`](termios::Termios) `orig_termios` 从父终端生成新终端。
fn handle_terminal(
    cmd: RunCommand,
    failover_cmd: Option<RunCommand>,
    orig_termios: Option<termios::Termios>,
    quit_cb: Box<dyn Fn(PaneId, Option<i32>, RunCommand) + Send>,
    terminal_id: u32,
) -> Result<(RawFd, RawFd)> {
    let err_context = || "failed to spawn child terminal".to_string();

    // 创建一个管道，允许子进程将 shell 的 pid 传达给其父进程。
    match openpty(None, &orig_termios) {
        Ok(open_pty_res) => handle_openpty(open_pty_res, cmd, quit_cb, terminal_id),
        Err(e) => match failover_cmd {
            Some(failover_cmd) => {
                handle_terminal(failover_cmd, None, orig_termios, quit_cb, terminal_id)
                    .with_context(err_context)
            },
            None => Err::<(i32, i32), _>(e)
                .context("failed to start pty")
                .with_context(err_context)
                .to_log(),
        },
    }
}

/// Unix PTY 后端。管理原生 PTY 文件描述符和信号。
#[derive(Clone)]
pub(crate) struct UnixPtyBackend {
    orig_termios: Arc<Mutex<Option<termios::Termios>>>,
    terminal_id_to_raw_fd: Arc<Mutex<BTreeMap<u32, Option<RawFd>>>>,
    next_terminal_id_counter: Arc<AtomicU32>,
}

/// 尝试在不阻塞的情况下将 `buf` 中尽可能多的字节写入 `fd`。
///
/// 在成功的短写入和 EINTR 上循环，以耗尽内核能接受的尽可能多的字节。在 EAGAIN（fd 缓冲区满）时，停止并返回到目前为止写入了多少字节（可能为 0）。调用者应重新排队任何未写入的剩余部分。
fn try_write_to_fd(fd: RawFd, buf: &[u8]) -> Result<usize> {
    let mut written = 0;
    while written < buf.len() {
        match unistd::write(unsafe { BorrowedFd::borrow_raw(fd) }, &buf[written..]) {
            Ok(0) => break, // fd 在非空 buf 上返回 0；视为 EAGAIN
            Ok(n) => written += n,
            Err(nix::errno::Errno::EINTR) => continue,
            Err(nix::errno::Errno::EAGAIN) => break,
            Err(e) => return Err(e.into()),
        }
    }
    Ok(written)
}

impl UnixPtyBackend {
    pub fn new() -> Result<Self, io::Error> {
        let current_termios = termios::tcgetattr(io::stdin()).ok();
        if current_termios.is_none() {
            log::warn!("正在无控制终端的情况下启动服务器，使用默认 termios 配置。");
        }
        Ok(Self {
            orig_termios: Arc::new(Mutex::new(current_termios)),
            terminal_id_to_raw_fd: Arc::new(Mutex::new(BTreeMap::new())),
            next_terminal_id_counter: Arc::new(AtomicU32::new(0)),
        })
    }

    pub fn spawn_terminal(
        &self,
        cmd: RunCommand,
        failover_cmd: Option<RunCommand>,
        quit_cb: Box<dyn Fn(PaneId, Option<i32>, RunCommand) + Send>,
        terminal_id: u32,
    ) -> Result<(Box<dyn AsyncReader>, RawFd)> {
        let orig_termios = self
            .orig_termios
            .lock()
            .to_anyhow()
            .context("failed to lock orig_termios")?;
        let (pid_primary, child_fd) = handle_terminal(
            cmd,
            failover_cmd,
            orig_termios.clone(),
            quit_cb,
            terminal_id,
        )?;
        self.terminal_id_to_raw_fd
            .lock()
            .to_anyhow()?
            .insert(terminal_id, Some(pid_primary));
        let async_reader = Box::new(
            RawFdAsyncReader::new(pid_primary)
                .map_err(|e| anyhow::anyhow!("创建异步读取器失败：{}", e))?,
        ) as Box<dyn AsyncReader>;
        Ok((async_reader, child_fd))
    }

    pub fn set_terminal_size(
        &self,
        terminal_id: u32,
        cols: u16,
        rows: u16,
        width_in_pixels: Option<u16>,
        height_in_pixels: Option<u16>,
    ) -> Result<()> {
        let err_context = || {
            format!(
                "failed to set terminal id {} to size ({}, {})",
                terminal_id, rows, cols
            )
        };
        match self
            .terminal_id_to_raw_fd
            .lock()
            .to_anyhow()
            .with_context(err_context)?
            .get(&terminal_id)
        {
            Some(Some(fd)) => {
                if cols > 0 && rows > 0 {
                    set_terminal_size_using_fd(*fd, cols, rows, width_in_pixels, height_in_pixels);
                }
            },
            _ => {
                Err::<(), _>(anyhow!("找不到 ID 为 {terminal_id} 的终端文件描述符"))
                    .with_context(err_context)
                    .non_fatal();
            },
        }
        Ok(())
    }

    pub fn write_to_tty_stdin(&self, terminal_id: u32, buf: &[u8]) -> Result<usize> {
        let err_context = || format!("failed to write to stdin of TTY ID {}", terminal_id);

        let fd = match self
            .terminal_id_to_raw_fd
            .lock()
            .to_anyhow()
            .with_context(err_context)?
            .get(&terminal_id)
        {
            Some(Some(fd)) => *fd,
            _ => {
                return Err(anyhow!("找不到原始文件描述符")).with_context(err_context)
            },
        };

        try_write_to_fd(fd, buf).with_context(err_context)
    }

    pub fn tcdrain(&self, terminal_id: u32) -> Result<()> {
        let err_context = || format!("failed to tcdrain to TTY ID {}", terminal_id);

        match self
            .terminal_id_to_raw_fd
            .lock()
            .to_anyhow()
            .with_context(err_context)?
            .get(&terminal_id)
        {
            Some(Some(fd)) => {
                termios::tcdrain(unsafe { BorrowedFd::borrow_raw(*fd) }).with_context(err_context)
            },
            _ => Err(anyhow!("找不到原始文件描述符")).with_context(err_context),
        }
    }

    pub fn tcgetpgrp(&self, terminal_id: u32) -> Option<i32> {
        match self.terminal_id_to_raw_fd.lock().ok()?.get(&terminal_id) {
            Some(Some(fd)) => unistd::tcgetpgrp(unsafe { BorrowedFd::borrow_raw(*fd) })
                .ok()
                .map(|pgid| pgid.as_raw()),
            _ => None,
        }
    }

    pub fn kill(&self, pid: u32) -> Result<()> {
        let _ = kill(unistd::Pid::from_raw(pid as i32), Some(Signal::SIGHUP));
        Ok(())
    }

    pub fn force_kill(&self, pid: u32) -> Result<()> {
        let _ = kill(unistd::Pid::from_raw(pid as i32), Some(Signal::SIGKILL));
        Ok(())
    }

    pub fn send_sigint(&self, pid: u32) -> Result<()> {
        let _ = kill(unistd::Pid::from_raw(pid as i32), Some(Signal::SIGINT));
        Ok(())
    }

    pub fn reserve_terminal_id(&self, terminal_id: u32) {
        self.terminal_id_to_raw_fd
            .lock()
            .unwrap()
            .insert(terminal_id, None);
    }

    pub fn clear_terminal_id(&self, terminal_id: u32) {
        self.terminal_id_to_raw_fd
            .lock()
            .unwrap()
            .remove(&terminal_id);
    }

    pub fn next_terminal_id(&self) -> Option<u32> {
        Some(
            self.next_terminal_id_counter
                .fetch_add(1, Ordering::Relaxed),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nix::fcntl::{fcntl, FcntlArg, OFlag};
    use nix::sys::termios;
    use std::io::Read;

    /// 验证 `try_write_to_fd` 在一次遍历中写入内核能接受的尽可能多的字节，并在 PTY 缓冲区填满时返回部分计数（而不是错误）。
    ///
    /// 并发读取器排空从端，以便接受一些字节。关键断言：函数返回 Ok(n)，其中 n <= buf.len()，调用者 (PtyWriter) 负责重新排队其余部分。
    #[test]
    fn try_write_to_fd_returns_partial_on_full_buffer() {
        let pty = openpty(None, &None).expect("openpty failed");
        let master_fd = pty.master.into_raw_fd();
        let slave_fd = pty.slave.into_raw_fd();
        let borrowed_master = unsafe { BorrowedFd::borrow_raw(master_fd) };
        let borrowed_slave = unsafe { BorrowedFd::borrow_raw(slave_fd) };

        let mut attrs = termios::tcgetattr(borrowed_slave).expect("tcgetattr failed");
        termios::cfmakeraw(&mut attrs);
        termios::tcsetattr(borrowed_slave, termios::SetArg::TCSANOW, &attrs)
            .expect("tcsetattr failed");

        // O_NONBLOCK 使 write() 返回 EAGAIN 而不是阻塞
        let flags = fcntl(borrowed_master, FcntlArg::F_GETFL).expect("F_GETFL");
        let mut oflags = OFlag::from_bits_truncate(flags);
        oflags.insert(OFlag::O_NONBLOCK);
        fcntl(borrowed_master, FcntlArg::F_SETFL(oflags)).expect("F_SETFL");

        // 填满大部分缓冲区，留一些空间
        let chunk = vec![0x42u8; 1024];
        let mut total_filled = 0;
        loop {
            match super::try_write_to_fd(master_fd, &chunk) {
                Ok(0) => break,
                Ok(n) => total_filled += n,
                Err(e) => panic!("填充缓冲区时出现意外错误：{e}"),
            }
        }
        assert!(
            total_filled > 0,
            "should have written some bytes to fill buffer"
        );

        // 从从端读取少量数据以释放部分空间
        let mut drain = vec![0u8; 512];
        let slave_file = unsafe { std::fs::File::from_raw_fd(slave_fd) };
        let mut slave_reader = std::io::BufReader::new(&slave_file);
        let drained = slave_reader.read(&mut drain).expect("从端读取失败");
        assert!(drained > 0, "本应已排空部分字节");
        // 防止 File 关闭从端 fd — 我们在下面手动关闭它
        std::mem::forget(slave_file);

        // 现在写入比释放空间更多的数据 — 应该得到部分写入
        let size = 128 * 1024;
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        let written = super::try_write_to_fd(master_fd, &data)
            .expect("try_write_to_fd 不应在 EAGAIN 上报错");

        assert!(
            written > 0 && written < size,
            "expected partial write, got {written}/{size}",
        );

        unsafe {
            libc::close(master_fd);
            libc::close(slave_fd);
        }
    }

    /// 验证当 fd 完全满且无法接受任何字节时，`try_write_to_fd` 返回 Ok(0) — 而不是错误。
    #[test]
    fn try_write_to_fd_returns_zero_on_stuck_pty() {
        let pty = openpty(None, &None).expect("openpty failed");
        let master_fd = pty.master.into_raw_fd();
        let slave_fd = pty.slave.into_raw_fd();
        let borrowed_master = unsafe { BorrowedFd::borrow_raw(master_fd) };
        let borrowed_slave = unsafe { BorrowedFd::borrow_raw(slave_fd) };

        let mut attrs = termios::tcgetattr(borrowed_slave).expect("tcgetattr failed");
        termios::cfmakeraw(&mut attrs);
        termios::tcsetattr(borrowed_slave, termios::SetArg::TCSANOW, &attrs)
            .expect("tcsetattr failed");

        let flags = fcntl(borrowed_master, FcntlArg::F_GETFL).expect("F_GETFL");
        let mut oflags = OFlag::from_bits_truncate(flags);
        oflags.insert(OFlag::O_NONBLOCK);
        fcntl(borrowed_master, FcntlArg::F_SETFL(oflags)).expect("F_SETFL");

        // 完全填满缓冲区 — 继续写入直到得到 Ok(0)
        let fill = vec![0x42u8; 1024];
        loop {
            match super::try_write_to_fd(master_fd, &fill) {
                Ok(0) => break,
                Ok(_) => continue,
                Err(e) => panic!("填充缓冲区时出现意外错误：{e}"),
            }
        }

        // 现在缓冲区已满 — 下一次写入应返回 Ok(0)
        let written = super::try_write_to_fd(master_fd, &[0x01, 0x02, 0x03])
            .expect("try_write_to_fd 不应在 EAGAIN 上报错");

        assert_eq!(written, 0, "expected zero bytes written on full buffer");

        unsafe {
            libc::close(master_fd);
            libc::close(slave_fd);
        }
    }
}
