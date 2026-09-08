use std::collections::{HashMap, VecDeque};
use std::time::Duration;

use zellij_utils::channels;
use zellij_utils::errors::{prelude::*, ContextType, PtyWriteContext};

use crate::route::NotificationEnd;
use crate::thread_bus::Bus;

// 我们将这些指令分离到不同的线程，因为某些程序在你从其 STDOUT 读取时写入其 STDIN 会发生死锁（我说的就是你，vim）
// 虽然在调整大小时尚未观察到同样的情况，但可以想象会发生，而且我们反正已经有了这个，所以
#[derive(Debug, Clone)]
pub enum PtyWriteInstruction {
    Write(Vec<u8>, u32, Option<NotificationEnd>),
    ResizePty(u32, u16, u16, Option<u16>, Option<u16>),
    StartCachingResizes,
    ApplyCachedResizes,
    Exit,
}

impl From<&PtyWriteInstruction> for PtyWriteContext {
    fn from(tty_write_instruction: &PtyWriteInstruction) -> Self {
        match *tty_write_instruction {
            PtyWriteInstruction::Write(..) => PtyWriteContext::Write,
            PtyWriteInstruction::ResizePty(..) => PtyWriteContext::ResizePty,
            PtyWriteInstruction::ApplyCachedResizes => PtyWriteContext::ApplyCachedResizes,
            PtyWriteInstruction::StartCachingResizes => PtyWriteContext::StartCachingResizes,
            PtyWriteInstruction::Exit => PtyWriteContext::Exit,
        }
    }
}

/// 每个终端在丢弃缓冲区之前的最大待处理字节数。
///
/// 暴露（但不是 `pub`）以便此模块中的测试可以引用它。
#[allow(dead_code)]
const MAX_PENDING_BYTES: usize = 10 * 1024 * 1024;

/// 当存在待处理写入时，在重试排空之前以此超时轮询新指令。保持线程对新指令的响应性，同时仍重试卡住的终端。
const PENDING_DRAIN_TIMEOUT: Duration = Duration::from_millis(10);

/// 等待写入终端标准输入的一块字节。
struct PendingWrite {
    bytes: Vec<u8>,
    offset: usize,
    _completion: Option<NotificationEnd>,
}

pub(crate) fn pty_writer_main(bus: Bus<PtyWriteInstruction>) -> Result<()> {
    let err_context = || "failed to write to pty".to_string();
    let mut pending: HashMap<u32, VecDeque<PendingWrite>> = HashMap::new();

    loop {
        // 如果有待处理写入，使用短超时以便我们可以继续排空。否则，阻塞直到新指令到达。
        let has_pending = pending.values().any(|q| !q.is_empty());
        let event = if has_pending {
            match bus.recv_timeout(PENDING_DRAIN_TIMEOUT) {
                Ok(pair) => Some(pair),
                Err(channels::RecvTimeoutError::Timeout) => None,
                Err(channels::RecvTimeoutError::Disconnected) => return Ok(()),
            }
        } else {
            Some(bus.recv().with_context(err_context)?)
        };

        let mut os_input = bus
            .os_input
            .clone()
            .context("no OS input API found")
            .with_context(err_context)?;

        // 如果收到了指令，则处理它
        if let Some((event, mut err_ctx)) = event {
            err_ctx.add_call(ContextType::PtyWrite((&event).into()));
            match event {
                PtyWriteInstruction::Write(bytes, terminal_id, completion) => {
                    let queue = pending.entry(terminal_id).or_default();
                    let queued: usize = queue
                        .iter()
                        .map(|w| w.bytes.len().saturating_sub(w.offset))
                        .sum();
                    if queued + bytes.len() > MAX_PENDING_BYTES {
                        log::error!(
                            "dropping write buffer for terminal {} \
                             ({} + {} bytes exceeds {} limit)",
                            terminal_id,
                            queued,
                            bytes.len(),
                            MAX_PENDING_BYTES,
                        );
                        queue.clear();
                    } else {
                        queue.push_back(PendingWrite {
                            bytes,
                            offset: 0,
                            _completion: completion,
                        });
                    }
                },
                PtyWriteInstruction::ResizePty(
                    terminal_id,
                    columns,
                    rows,
                    width_in_pixels,
                    height_in_pixels,
                ) => {
                    os_input
                        .set_terminal_size_using_terminal_id(
                            terminal_id,
                            columns,
                            rows,
                            width_in_pixels,
                            height_in_pixels,
                        )
                        .with_context(err_context)
                        .non_fatal();
                },
                PtyWriteInstruction::StartCachingResizes => {
                    // 我们这样做是因为 screen/tab/layout 代码中有一些逻辑陷阱导致多个调整大小被发送到 pty — 虽然最后一个总是正确的，但许多程序和 shell 会对这些进行去抖（我猜是由于处理控制终端窗口 GUI 调整大小的创伤），这然后会导致故障和缺失的重绘
                    // 所以我们这样做是为了友好，始终只向每个窗格发送最后的调整大小指令
                    // 此逻辑发生在主 Screen 事件循环中
                    os_input.cache_resizes();
                },
                PtyWriteInstruction::ApplyCachedResizes => {
                    os_input.apply_cached_resizes();
                },
                PtyWriteInstruction::Exit => {
                    return Ok(());
                },
            }
        }

        // 排空待处理写入 — 在所有终端上进行一次遍历。
        // 对于每个终端，写入内核在不阻塞的情况下能接受的尽可能多的数据，然后移至下一个终端。
        let terminal_ids: Vec<u32> = pending.keys().copied().collect();
        for tid in terminal_ids {
            let queue = match pending.get_mut(&tid) {
                Some(q) if !q.is_empty() => q,
                _ => continue,
            };

            while let Some(front) = queue.front_mut() {
                let remaining = front.bytes.get(front.offset..).unwrap_or_default();
                if remaining.is_empty() {
                    queue.pop_front();
                    continue;
                }
                match os_input.write_to_tty_stdin(tid, remaining) {
                    Ok(0) => break, // EAGAIN — 移至下一个终端
                    Ok(n) => {
                        front.offset = front.offset.saturating_add(n);
                        if front.offset >= front.bytes.len() {
                            queue.pop_front();
                        }
                    },
                    Err(e) => {
                        // 终端错误 (EBADF, EIO 等) — 清除其队列
                        Err::<(), _>(e).with_context(err_context).non_fatal();
                        queue.clear();
                        break;
                    },
                }
            }
        }
        pending.retain(|_, q| !q.is_empty());
    }
}

#[cfg(test)]
#[path = "./unit/pty_writer_tests.rs"]
mod pty_writer_tests;
