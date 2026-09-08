use crate::{os_input_output::AsyncReader, screen::ScreenInstruction, thread_bus::ThreadSenders};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tokio::task;
use zellij_utils::{
    errors::{get_current_ctx, prelude::*, ContextType},
    logging::debug_to_file,
};

pub(crate) struct TerminalBytes {
    terminal_id: u32,
    senders: ThreadSenders,
    async_reader: Box<dyn AsyncReader>,
    debug: bool,
    activity_flag: Arc<AtomicBool>,
}

impl TerminalBytes {
    pub fn new(
        terminal_id: u32,
        async_reader: Box<dyn AsyncReader>,
        senders: ThreadSenders,
        debug: bool,
        activity_flag: Arc<AtomicBool>,
    ) -> Self {
        TerminalBytes {
            terminal_id,
            senders,
            debug,
            async_reader,
            activity_flag,
        }
    }
    pub async fn listen(&mut self) -> Result<()> {
        // 此函数从 pty 读取字节，然后将它们作为 ScreenInstruction::PtyBytes 发送到 screen 以在那里解析
        // 我们还向 Screen 发送单独的指令以作为 ScreenInstruction::Render 进行渲染
        //
        // 我们努力在发送字节进行解析后立即向 screen 发送 Render 指令 — 这样渲染快速且流畅。然而，如果 screen 积压，这可能会导致延迟。因此，如果我们检测到发送渲染指令所需时间出现峰值，我们假设 screen 线程已积压，因此只少量发送渲染指令，给 screen 时间来处理字节和渲染，同时仍允许用户看到事情正在发生的指示（少量渲染指令）
        let err_context = || "failed to listen for bytes from PTY".to_string();

        let mut err_ctx = get_current_ctx();
        err_ctx.add_call(ContextType::AsyncTask);
        let mut buf = [0u8; 65536];
        loop {
            match self.async_reader.read(&mut buf).await {
                Ok(0) => break, // EOF
                Err(err) => {
                    log::error!("{}", err);
                    break;
                },
                Ok(n_bytes) => {
                    self.activity_flag.store(true, Ordering::Relaxed);
                    let bytes = &buf[..n_bytes];
                    if self.debug {
                        let _ = debug_to_file(bytes, self.terminal_id as i32);
                    }
                    self.async_send_to_screen(ScreenInstruction::PtyBytes(
                        self.terminal_id,
                        bytes.to_vec(),
                    ))
                    .await
                    .with_context(err_context)?;
                },
            }
        }

        // 忽略这里发生的任何错误。
        // 我们只在窗格退出时离开上面的循环。这可能以多种方式发生，但最有问题的是使用 `Ctrl+q` 退出 zellij 时。这是因为 `Screen` 的通道已经退出，所以此发送*将*失败。这本身不是问题，因为应用程序无论如何都会终止，但它会为我们退出应用程序时仍处于活动状态的每个窗格在日志中打印一条冗长的错误消息。
        // 这：
        //
        // 1. 使日志变得相当无意义，因为即使应用程序 "正常" 退出，里面也会有错误
        // 2. 给人留下我们代码中有 bug 且无法正确终止的印象
        //
        // FIXME: 理想情况下，我们检测应用程序是否正在退出，并且只在那种特定情况下忽略错误？
        let _ = self.async_send_to_screen(ScreenInstruction::Render).await;

        Ok(())
    }
    async fn async_send_to_screen(
        &self,
        screen_instruction: ScreenInstruction,
    ) -> Result<Duration> {
        // 返回它阻塞线程的时间
        let sent_at = Instant::now();
        let senders = self.senders.clone();
        task::spawn_blocking(move || senders.send_to_screen(screen_instruction))
            .await
            .context("failed to async-send to screen")?
            .context("failed to block on sending message to screen")?;
        Ok(sent_at.elapsed())
    }
}
