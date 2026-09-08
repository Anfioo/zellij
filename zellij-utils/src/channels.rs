//! 用于在线程之间发送和接收消息的定义与辅助函数。

use std::cell::RefCell;

use crate::errors::{get_current_ctx, ErrorContext};
pub use crossbeam::channel::{
    bounded, unbounded, Receiver, RecvError, RecvTimeoutError, Select, SendError, Sender,
    TrySendError,
};

/// 一个带有额外错误上下文的 [MPSC](mpsc) 异步通道。
pub type ChannelWithContext<T> = (Sender<(T, ErrorContext)>, Receiver<(T, ErrorContext)>);

/// 在 [MPSC](std::sync::mpsc) 通道上发送消息，同时附带一个 [`ErrorContext`]，
/// 根据底层的 [`SenderType`] 决定是同步还是异步发送。
#[derive(Clone)]
pub struct SenderWithContext<T> {
    sender: Sender<(T, ErrorContext)>,
}

impl<T: Clone> SenderWithContext<T> {
    pub fn new(sender: Sender<(T, ErrorContext)>) -> Self {
        Self { sender }
    }

    /// 在这个 [`SenderWithContext`] 的通道上发送一个事件，同时附带当前的 [`ErrorContext`]。
    pub fn send(&self, event: T) -> Result<(), SendError<(T, ErrorContext)>> {
        let err_ctx = get_current_ctx();
        self.sender.send((event, err_ctx))
    }
}

thread_local!(
    /// 指向某个线程局部存储（TLS）的键，该存储以 [`ErrorContext`] 的形式保存线程调用栈的表示。
    pub static OPENCALLS: RefCell<ErrorContext> = RefCell::default()
);

tokio::task_local! {
    /// 指向某个任务局部存储的键，该存储以 [`ErrorContext`] 的形式保存任务调用栈的表示。
    pub static ASYNCOPENCALLS: RefCell<ErrorContext>;
}
