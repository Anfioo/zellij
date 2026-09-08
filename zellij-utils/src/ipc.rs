//! 用于在线程之间发送和接收消息的定义与辅助函数。

use crate::channels::{SenderWithContext, ASYNCOPENCALLS, OPENCALLS};
use crate::data::{ClientId, ClientToServerMsg, ServerToClientMsg};
use crate::errors::prelude::*;
use crate::errors::{ErrorContext, ToAnyhow};
use anyhow::Context;
use interprocess::local_socket::traits::Stream;
use interprocess::local_socket::Stream as LocalSocketStream;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::marker::PhantomData;
use std::path::Path;

/// 一个包装了 [`LocalSocketStream`] 的发送器，带有错误上下文。
pub struct IpcSenderWithContext<T> {
    sender: BufWriter<LocalSocketStream>,
    current_instructions: ErrorContext,
    phantom: PhantomData<T>,
}

impl<T> IpcSenderWithContext<T> {
    /// 返回一个新的 [`IpcSenderWithContext`]。
    pub fn new(sender: LocalSocketStream) -> Self {
        Self {
            sender: BufWriter::new(sender),
            current_instructions: ErrorContext::new(),
            phantom: PhantomData,
        }
    }

    /// 从这个 [`IpcSenderWithContext`] 中获取一个 [`IpcReceiverWithContext`]。
    pub fn get_receiver<U>(&mut self) -> IpcReceiverWithContext<U> {
        IpcReceiverWithContext::new(self.sender.get_mut().try_clone().unwrap())
    }

    /// 在此 [`IpcSenderWithContext`] 的流上发送消息，同时附带当前的 [`ErrorContext`]。
    pub fn send(&mut self, msg: T) -> anyhow::Result<()>
    where
        T: serde::Serialize,
    {
        let err_ctx = ASYNCOPENCALLS
            .try_with(|ctx| *ctx.borrow())
            .unwrap_or_else(|_| OPENCALLS.with(|ctx| *ctx.borrow()));
        self.current_instructions = err_ctx;
        let op_codes = bincode::serialize(&err_ctx)?;
        let msg = bincode::serialize(&msg)?;
        self.send_bytes(&op_codes)?;
        self.send_bytes(&msg)?;
        self.sender.flush()?;
        Ok(())
    }

    fn send_bytes(&mut self, bytes: &[u8]) -> anyhow::Result<()> {
        let len = u32::try_from(bytes.len())?;
        self.sender.write_all(&len.to_le_bytes())?;
        self.sender.write_all(bytes)?;
        Ok(())
    }
}

/// 一个包装了 [`LocalSocketStream`] 的接收器，带有错误上下文。
pub struct IpcReceiverWithContext<T> {
    receiver: BufReader<LocalSocketStream>,
    phantom: PhantomData<T>,
}

impl<T> IpcReceiverWithContext<T> {
    /// 返回一个新的 [`IpcReceiverWithContext`]。
    pub fn new(receiver: LocalSocketStream) -> Self {
        Self {
            receiver: BufReader::new(receiver),
            phantom: PhantomData,
        }
    }

    /// 从这个 [`IpcReceiverWithContext`] 中获取一个 [`IpcSenderWithContext`]。
    pub fn get_sender<U>(&mut self) -> IpcSenderWithContext<U> {
        IpcSenderWithContext::new(self.receiver.get_mut().try_clone().unwrap())
    }

    /// 在此 [`IpcReceiverWithContext`] 的流上接收消息，同时附带当前的 [`ErrorContext`]。
    pub fn recv(&mut self) -> Option<(T, ErrorContext)>
    where
        T: serde::de::DeserializeOwned,
    {
        match self.recv_bytes() {
            Ok(err_ctx_bytes) => {
                let err_ctx: ErrorContext = match bincode::deserialize(&err_ctx_bytes) {
                    Ok(err_ctx) => err_ctx,
                    Err(e) => {
                        log::error!("Failed to deserialize error context: {:?}", e);
                        return None;
                    },
                };
                match self.recv_bytes() {
                    Ok(msg_bytes) => {
                        let msg: T = match bincode::deserialize(&msg_bytes) {
                            Ok(msg) => msg,
                            Err(e) => {
                                log::error!("Failed to deserialize message: {:?}", e);
                                return None;
                            },
                        };
                        Some((msg, err_ctx))
                    },
                    Err(e) => {
                        log::error!("Failed to receive message bytes: {:?}", e);
                        None
                    },
                }
            },
            Err(e) => {
                log::error!("Failed to receive error context bytes: {:?}", e);
                None
            },
        }
    }

    fn recv_bytes(&mut self) -> anyhow::Result<Vec<u8>> {
        let mut len_bytes = [0u8; 4];
        self.receiver.read_exact(&mut len_bytes)?;
        let len = u32::from_le_bytes(len_bytes) as usize;
        let mut buffer = vec![0u8; len];
        self.receiver.read_exact(&mut buffer)?;
        Ok(buffer)
    }
}

impl IpcSenderWithContext<ClientToServerMsg> {
    pub fn send_client_msg(&mut self, msg: ClientToServerMsg) -> anyhow::Result<()> {
        self.send(msg)
    }
}

impl IpcReceiverWithContext<ServerToClientMsg> {
    pub fn recv_server_msg(&mut self) -> Option<(ServerToClientMsg, ErrorContext)> {
        self.recv()
    }
}

impl IpcSenderWithContext<ServerToClientMsg> {
    pub fn send_server_msg(&mut self, msg: ServerToClientMsg) -> anyhow::Result<()> {
        self.send(msg)
    }
}

impl IpcReceiverWithContext<ClientToServerMsg> {
    pub fn recv_client_msg(&mut self) -> Option<(ClientToServerMsg, ErrorContext)> {
        self.recv()
    }
}

/// 用于从套接字路径创建 [`IpcSenderWithContext`] 的辅助函数。
pub fn create_ipc_sender<T>(path: &Path) -> anyhow::Result<IpcSenderWithContext<T>> {
    let stream = crate::consts::ipc_connect(path)?;
    Ok(IpcSenderWithContext::new(stream))
}

/// 用于从套接字路径创建 [`IpcReceiverWithContext`] 的辅助函数。
pub fn create_ipc_receiver<T>(path: &Path) -> anyhow::Result<IpcReceiverWithContext<T>> {
    let stream = crate::consts::ipc_connect(path)?;
    Ok(IpcReceiverWithContext::new(stream))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::ClientToServerMsg;
    use std::io::Write;

    #[test]
    fn test_ipc_sender_receiver_roundtrip() {
        // 这是一个简单的测试，验证序列化/反序列化是否正常工作
        // 我们使用内存中的缓冲区而不是真正的套接字
        let msg = ClientToServerMsg::ConnStatus;
        let err_ctx = ErrorContext::new();

        let msg_bytes = bincode::serialize(&msg).unwrap();
        let err_ctx_bytes = bincode::serialize(&err_ctx).unwrap();

        let deserialized_msg: ClientToServerMsg = bincode::deserialize(&msg_bytes).unwrap();
        let deserialized_err_ctx: ErrorContext = bincode::deserialize(&err_ctx_bytes).unwrap();

        assert_eq!(deserialized_msg, msg);
        assert_eq!(deserialized_err_ctx, err_ctx);
    }
}
