use std::io::prelude::*;
use std::process::{Child, Command, Stdio};
use std::thread::JoinHandle;

use anyhow::{Context, Result};

pub struct CopyCommand {
    command: String,
    args: Vec<String>,
}

impl CopyCommand {
    pub fn new(command: String) -> Self {
        let mut command_with_args = command.split(' ').map(String::from);

        Self {
            command: command_with_args.next().expect("missing command"),
            args: command_with_args.collect(),
        }
    }
    pub fn set(&self, value: String) -> Result<()> {
        let mut process = Command::new(self.command.clone())
            .args(self.args.clone())
            .stdin(Stdio::piped())
            .spawn()
            .with_context(|| format!("couldn't spawn {}", self.command))?;
        process
            .stdin
            .take()
            .context("could not get stdin")?
            .write_all(value.as_bytes())
            .with_context(|| format!("couldn't write to {} stdin", self.command))?;

        reap(process);

        Ok(())
    }
}

/// Wait for the 复制 命令's 进程 in the background, so that it does not linger as a zombie
/// once it 退出.
///
/// The 进程 is deliberately not 杀死的 if it outlives the 复制 operation: X11 and Wayland
/// 剪贴板 helpers (eg. `xsel`, `xclip` or `wl-复制`) keep running for as long as they own the
/// 选择, and 杀死 them clears the 剪贴板. 复制 命令 that do 退出 by themselves (eg.
/// `pbcopy`) are reaped as soon as they do.
fn reap(mut process: Child) -> JoinHandle<()> {
    std::thread::spawn(move || {
        if let Err(e) = process.wait() {
            log::error!("Clipboard failure: {}", e);
        }
    })
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use nix::errno::Errno;
    use nix::sys::wait::{waitpid, WaitPidFlag};
    use nix::unistd::Pid;
    use std::path::Path;
    use std::time::{Duration, Instant};

    // stands in for an x11/wayland 剪贴板 helper: 消费 标准输入 and then keeps running the way
    // `xsel` or `xclip` do while they own the 选择
    fn fake_clipboard_helper(dir: &Path, received: &Path, still_alive: &Path) -> String {
        let script = dir.join("fake-clipboard-helper");
        std::fs::write(
            &script,
            format!(
                "cat > '{}'\nsleep 2\ntouch '{}'\n",
                received.display(),
                still_alive.display()
            ),
        )
        .unwrap();
        // run it through `sh` rather than making it executable, so that a concurrent fork/exec
        // elsewhere in the 测试 binary can't make this spawn fail with ETXTBSY
        format!("sh {}", script.display())
    }

    #[test]
    fn copy_command_is_not_killed_when_it_outlives_the_copy() {
        let dir = tempfile::tempdir().unwrap();
        let received = dir.path().join("received");
        let still_alive = dir.path().join("still-alive");
        let copy_command = fake_clipboard_helper(dir.path(), &received, &still_alive);

        CopyCommand::new(copy_command)
            .set("some copied text".to_owned())
            .unwrap();

        // 轮询 (bounded to ~15s) for the helper to outlive the 复制 operation
        let deadline = Instant::now() + Duration::from_secs(15);
        while !still_alive.exists() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(20));
        }

        assert_eq!(
            std::fs::read_to_string(&received).unwrap(),
            "some copied text",
            "the copy command should be fed the copied text on its stdin"
        );
        assert!(
            still_alive.exists(),
            "the copy command should not be killed for outliving the copy operation"
        );
    }

    #[test]
    fn copy_command_is_reaped_once_it_exits() {
        let process = Command::new("sh")
            .args(["-c", "exit 0"])
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();
        let pid = Pid::from_raw(process.id() as i32);

        reap(process).join().unwrap();

        // having been waited for, the 进程 is gone from the 进程 表格 rather than left
        // behind as a zombie
        assert_eq!(
            waitpid(pid, Some(WaitPidFlag::WNOHANG)).err(),
            Some(Errno::ECHILD),
            "the copy command's process should have been reaped"
        );
    }
}
