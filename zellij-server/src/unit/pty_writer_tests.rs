use super::*;
use crate::os_input_output::{AsyncReader, ServerOsApi};
use crate::panes::PaneId;
use crate::thread_bus::Bus;
use interprocess::local_socket::Stream as LocalSocketStream;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use zellij_utils::channels::SenderWithContext;
use zellij_utils::data::Palette;

use zellij_utils::input::command::RunCommand;
use zellij_utils::ipc::{ClientToServerMsg, IpcReceiverWithContext, ServerToClientMsg};

use crate::ClientId;

/// 模拟对象的每终端写入行为。
#[derive(Clone)]
enum WriteBehavior {
    /// 接受所有字节（像健康的终端一样）。
    AcceptAll,
    /// 每次写入调用最多接受 `n` 个字节（模拟短写入）。
    AcceptAtMost(usize),
    /// 接受零字节（模拟 EAGAIN / 缓冲区满）。
    AcceptNone,
    /// 返回错误（模拟 EBADF、EIO 等）。
    Error,
}

#[derive(Clone)]
struct MockServerOsApi {
    /// 控制每个终端在写入时的行为。
    write_behavior: Arc<Mutex<HashMap<u32, WriteBehavior>>>,
    /// 每次成功写入的 (terminal_id, bytes) 日志。
    write_log: Arc<Mutex<Vec<(u32, Vec<u8>)>>>,
}

impl MockServerOsApi {
    fn new() -> Self {
        MockServerOsApi {
            write_behavior: Arc::new(Mutex::new(HashMap::new())),
            write_log: Arc::new(Mutex::new(Vec::new())),
        }
    }
    fn set_behavior(&self, terminal_id: u32, behavior: WriteBehavior) {
        self.write_behavior
            .lock()
            .unwrap()
            .insert(terminal_id, behavior);
    }
    fn get_written_bytes(&self, terminal_id: u32) -> Vec<u8> {
        self.write_log
            .lock()
            .unwrap()
            .iter()
            .filter(|(tid, _)| *tid == terminal_id)
            .flat_map(|(_, bytes)| bytes.clone())
            .collect()
    }
}

impl ServerOsApi for MockServerOsApi {
    fn set_terminal_size_using_terminal_id(
        &self,
        _id: u32,
        _cols: u16,
        _rows: u16,
        _width_in_pixels: Option<u16>,
        _height_in_pixels: Option<u16>,
    ) -> Result<()> {
        Ok(())
    }
    fn spawn_terminal(
        &self,
        _terminal_action: zellij_utils::input::command::TerminalAction,
        _quit_cb: Box<dyn Fn(PaneId, Option<i32>, RunCommand) + Send>,
        _default_editor: Option<PathBuf>,
    ) -> Result<(u32, Box<dyn AsyncReader>, Option<u32>)> {
        unimplemented!()
    }
    fn write_to_tty_stdin(&self, terminal_id: u32, buf: &[u8]) -> Result<usize> {
        let behavior = self
            .write_behavior
            .lock()
            .unwrap()
            .get(&terminal_id)
            .cloned()
            .unwrap_or(WriteBehavior::AcceptAll);
        match behavior {
            WriteBehavior::AcceptAll => {
                self.write_log
                    .lock()
                    .unwrap()
                    .push((terminal_id, buf.to_vec()));
                Ok(buf.len())
            },
            WriteBehavior::AcceptAtMost(max) => {
                let n = buf.len().min(max);
                if n > 0 {
                    self.write_log
                        .lock()
                        .unwrap()
                        .push((terminal_id, buf[..n].to_vec()));
                }
                Ok(n)
            },
            WriteBehavior::AcceptNone => Ok(0),
            WriteBehavior::Error => Err(anyhow::anyhow!(
                "simulated write error for terminal {}",
                terminal_id
            )),
        }
    }
    fn tcdrain(&self, _id: u32) -> Result<()> {
        Ok(())
    }
    fn kill(&self, _pid: u32) -> Result<()> {
        Ok(())
    }
    fn force_kill(&self, _pid: u32) -> Result<()> {
        Ok(())
    }
    fn send_sigint(&self, _pid: u32) -> Result<()> {
        Ok(())
    }
    fn box_clone(&self) -> Box<dyn ServerOsApi> {
        Box::new((*self).clone())
    }
    fn send_to_client(&self, _client_id: ClientId, _msg: ServerToClientMsg) -> Result<()> {
        Ok(())
    }
    fn new_client(
        &mut self,
        _client_id: ClientId,
        _stream: LocalSocketStream,
    ) -> Result<IpcReceiverWithContext<ClientToServerMsg>> {
        unimplemented!()
    }
    fn new_client_with_reply(
        &mut self,
        _client_id: ClientId,
        _stream: LocalSocketStream,
        _reply_stream: LocalSocketStream,
    ) -> Result<IpcReceiverWithContext<ClientToServerMsg>> {
        unimplemented!()
    }
    fn remove_client(&mut self, _client_id: ClientId) -> Result<()> {
        Ok(())
    }
    fn load_palette(&self) -> Palette {
        Palette::default()
    }
    fn get_cwd(&self, _pid: u32) -> Option<PathBuf> {
        None
    }
    fn write_to_file(&mut self, _buf: String, _file: Option<String>) -> Result<()> {
        Ok(())
    }
    fn re_run_command_in_terminal(
        &self,
        _terminal_id: u32,
        _run_command: RunCommand,
        _quit_cb: Box<dyn Fn(PaneId, Option<i32>, RunCommand) + Send>,
    ) -> Result<(Box<dyn AsyncReader>, Option<u32>)> {
        unimplemented!()
    }
    fn clear_terminal_id(&self, _terminal_id: u32) -> Result<()> {
        Ok(())
    }
}

/// 辅助函数：使用给定的模拟对象为 PtyWriteInstruction 创建 Bus 和发送器。
fn make_test_bus(
    mock: MockServerOsApi,
) -> (
    Bus<PtyWriteInstruction>,
    SenderWithContext<PtyWriteInstruction>,
) {
    let (sender, receiver) = channels::unbounded();
    let sender_with_ctx = SenderWithContext::new(sender);
    let bus = Bus::new(
        vec![receiver],
        None,
        None,
        None,
        None,
        None,
        None,
        Some(Box::new(mock)),
    );
    (bus, sender_with_ctx)
}

/// 辅助函数：发送指令然后 Exit，运行 pty_writer_main，完成后返回。
fn run_pty_writer(
    bus: Bus<PtyWriteInstruction>,
    sender: SenderWithContext<PtyWriteInstruction>,
    instructions: Vec<PtyWriteInstruction>,
) {
    // 发送所有指令，然后发送 Exit
    for instruction in instructions {
        sender.send(instruction).unwrap();
    }
    sender.send(PtyWriteInstruction::Exit).unwrap();

    pty_writer_main(bus).unwrap();
}

#[test]
fn full_write_completes_immediately() {
    let mock = MockServerOsApi::new();
    mock.set_behavior(1, WriteBehavior::AcceptAll);
    let mock_clone = mock.clone();

    let (bus, sender) = make_test_bus(mock);
    run_pty_writer(
        bus,
        sender,
        vec![PtyWriteInstruction::Write(b"hello world".to_vec(), 1, None)],
    );

    let written = mock_clone.get_written_bytes(1);
    assert_eq!(written, b"hello world");
}

#[test]
fn partial_write_is_buffered_and_drained() {
    let mock = MockServerOsApi::new();
    // 一次只接受 3 个字节
    mock.set_behavior(1, WriteBehavior::AcceptAtMost(3));
    let mock_clone = mock.clone();

    let (bus, sender) = make_test_bus(mock);
    run_pty_writer(
        bus,
        sender,
        vec![PtyWriteInstruction::Write(b"abcdefghij".to_vec(), 1, None)],
    );

    let written = mock_clone.get_written_bytes(1);
    assert_eq!(
        written, b"abcdefghij",
        "all bytes should eventually be written"
    );
}

#[test]
fn eagain_on_one_terminal_does_not_block_another() {
    let mock = MockServerOsApi::new();
    // 终端 1 卡住了 (EAGAIN)
    mock.set_behavior(1, WriteBehavior::AcceptNone);
    // 终端 2 接受一切
    mock.set_behavior(2, WriteBehavior::AcceptAll);
    let mock_clone = mock.clone();

    let (bus, sender) = make_test_bus(mock);
    run_pty_writer(
        bus,
        sender,
        vec![
            PtyWriteInstruction::Write(b"stuck data".to_vec(), 1, None),
            PtyWriteInstruction::Write(b"good data".to_vec(), 2, None),
        ],
    );

    // 尽管终端 1 卡住了，终端 2 应该已收到其字节
    let written_2 = mock_clone.get_written_bytes(2);
    assert_eq!(written_2, b"good data");

    // 终端 1 数据丢失（卡住直到 Exit，然后被丢弃）
    // — 这是预期行为；不对终端 1 做断言
}

#[test]
fn writes_to_same_terminal_preserve_order() {
    let mock = MockServerOsApi::new();
    // 一次只接受 4 个字节 — 强制缓冲
    mock.set_behavior(1, WriteBehavior::AcceptAtMost(4));
    let mock_clone = mock.clone();

    let (bus, sender) = make_test_bus(mock);
    run_pty_writer(
        bus,
        sender,
        vec![
            PtyWriteInstruction::Write(b"AAAA".to_vec(), 1, None),
            PtyWriteInstruction::Write(b"BBBB".to_vec(), 1, None),
            PtyWriteInstruction::Write(b"CCCC".to_vec(), 1, None),
        ],
    );

    let written = mock_clone.get_written_bytes(1);
    assert_eq!(
        written, b"AAAABBBBCCCC",
        "writes must be delivered in order"
    );
}

#[test]
fn memory_cap_drops_buffer_when_exceeded() {
    let mock = MockServerOsApi::new();
    // 终端卡住了 — 没有任何东西被排空
    mock.set_behavior(1, WriteBehavior::AcceptNone);
    let mock_clone = mock.clone();

    let (bus, sender) = make_test_bus(mock);

    // 发送足够多的数据以超过 MAX_PENDING_BYTES
    let chunk_size = 1024 * 1024; // 1 MB
    let mut instructions = Vec::new();
    for _ in 0..(MAX_PENDING_BYTES / chunk_size + 1) {
        instructions.push(PtyWriteInstruction::Write(vec![0x42; chunk_size], 1, None));
    }
    // 超过上限后，向终端 2 发送一次写入以验证写入线程仍然正常工作
    instructions.push(PtyWriteInstruction::Write(b"still alive".to_vec(), 2, None));
    mock_clone.set_behavior(2, WriteBehavior::AcceptAll);

    run_pty_writer(bus, sender, instructions);

    let written_2 = mock_clone.get_written_bytes(2);
    assert_eq!(
        written_2, b"still alive",
        "writer thread should continue functioning after dropping a buffer"
    );
}

#[test]
fn write_error_clears_terminal_queue() {
    let mock = MockServerOsApi::new();
    mock.set_behavior(1, WriteBehavior::Error);
    mock.set_behavior(2, WriteBehavior::AcceptAll);
    let mock_clone = mock.clone();

    let (bus, sender) = make_test_bus(mock);
    run_pty_writer(
        bus,
        sender,
        vec![
            PtyWriteInstruction::Write(b"will fail".to_vec(), 1, None),
            PtyWriteInstruction::Write(b"also queued".to_vec(), 1, None),
            PtyWriteInstruction::Write(b"should work".to_vec(), 2, None),
        ],
    );

    // 终端 1 应该没有写入任何内容（错误清除了队列）
    let written_1 = mock_clone.get_written_bytes(1);
    assert!(
        written_1.is_empty(),
        "errored terminal should have no written bytes"
    );

    // 终端 2 应该不受影响
    let written_2 = mock_clone.get_written_bytes(2);
    assert_eq!(written_2, b"should work");
}
