use std::{collections::VecDeque, io::Write};

use crate::plugins::PluginId;
use log::{debug, error};
use zellij_utils::errors::prelude::*;

use serde::{Deserialize, Serialize};

// 16kB 日志缓冲区
const ZELLIJ_MAX_PIPE_BUFFER_SIZE: usize = 16_384;
#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingPipe {
    buffer: VecDeque<u8>,
    plugin_name: String,
    plugin_id: PluginId,
}

impl LoggingPipe {
    pub fn new(plugin_name: &str, plugin_id: PluginId) -> LoggingPipe {
        LoggingPipe {
            buffer: VecDeque::new(),
            plugin_name: String::from(plugin_name),
            plugin_id,
        }
    }

    fn log_message(&self, message: &str) {
        debug!(
            "|{:<25.25}| {} [{:<10.15}] {}",
            self.plugin_name,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S.%3f"),
            format!("id: {}", self.plugin_id),
            message
        );
    }
}

impl Write for LoggingPipe {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.buffer.len() + buf.len() > ZELLIJ_MAX_PIPE_BUFFER_SIZE {
            let error_msg =
                "Exceeded log buffer size. Make sure that your plugin calls flush on stderr on \
                valid UTF-8 symbol boundary. Additionally, make sure that your log message contains \
                endline \\n symbol.";
            error!("{}: {}", self.plugin_name, error_msg);
            self.buffer.clear();
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                error_msg,
            ));
        }

        self.buffer.extend(buf);
        self.flush()?;

        Ok(buf.len())
    }

    // 当我们刷新时，检查当前缓冲区是否为有效的 utf8 字符串，按 '\n' 分割并在此过程中截断缓冲区。
    // 我们假设最终会在有效的字符串边界上调用 flush（即 std::str::from_utf8(..).is_ok() 在某个时刻返回 true）。
    // 上述假设可能不成立，在这种情况下我们将不得不考虑它。目前先简化处理。
    fn flush(&mut self) -> std::io::Result<()> {
        self.buffer.make_contiguous();

        match std::str::from_utf8(self.buffer.as_slices().0) {
            Ok(converted_buffer) => {
                if converted_buffer.contains('\n') {
                    let mut consumed_bytes = 0;
                    let mut split_converted_buffer = converted_buffer.split('\n').peekable();

                    while let Some(msg) = split_converted_buffer.next() {
                        if split_converted_buffer.peek().is_none() {
                            // 仅当最后一个字符是换行符时才记录最后一块。否则不记录。
                            if converted_buffer.ends_with('\n') && !msg.is_empty() {
                                self.log_message(msg);
                                consumed_bytes += msg.len() + 1;
                            }
                        } else {
                            self.log_message(msg);
                            consumed_bytes += msg.len() + 1;
                        }
                    }
                    drop(self.buffer.drain(..consumed_bytes));
                }
            },
            Err(e) => Err::<(), _>(e)
                .context("刷新日志管道缓冲区失败")
                .non_fatal(),
        }

        Ok(())
    }
}

// 单元测试
#[cfg(test)]
mod logging_pipe_test {

    use super::*;

    #[test]
    fn write_without_endl_does_not_consume_buffer_after_flush() {
        let mut pipe = LoggingPipe::new("TestPipe", 0);

        let test_buffer = b"Testing write";

        pipe.write_all(test_buffer).expect("Err write");
        pipe.flush().expect("Err flush");

        assert_eq!(pipe.buffer.len(), test_buffer.len());
    }

    #[test]
    fn write_with_single_endl_at_the_end_consumes_whole_buffer_after_flush() {
        let mut pipe = LoggingPipe::new("TestPipe", 0);

        let test_buffer = b"Testing write \n";

        pipe.write_all(test_buffer).expect("Err write");
        pipe.flush().expect("Err flush");

        assert_eq!(pipe.buffer.len(), 0);
    }

    #[test]
    fn write_with_endl_in_the_middle_consumes_buffer_up_to_endl_after_flush() {
        let mut pipe = LoggingPipe::new("TestPipe", 0);

        let test_buffer = b"Testing write \n";
        let test_buffer2: &[_] = b"And the rest";

        pipe.write_all(
            [
                test_buffer,
                test_buffer,
                test_buffer,
                test_buffer,
                test_buffer2,
            ]
            .concat()
            .as_slice(),
        )
        .expect("Err write");
        pipe.flush().expect("Err flush");

        assert_eq!(pipe.buffer.len(), test_buffer2.len());
    }

    #[test]
    fn write_with_many_endl_consumes_whole_buffer_after_flush() {
        let mut pipe = LoggingPipe::new("TestPipe", 0);

        let test_buffer: &[_] = b"Testing write \n";

        pipe.write_all(
            [
                test_buffer,
                test_buffer,
                test_buffer,
                test_buffer,
                test_buffer,
            ]
            .concat()
            .as_slice(),
        )
        .expect("Err write");
        pipe.flush().expect("Err flush");

        assert_eq!(pipe.buffer.len(), 0);
    }

    #[test]
    fn write_with_incorrect_byte_boundary_does_not_crash() {
        let mut pipe = LoggingPipe::new("TestPipe", 0);

        let test_buffer = "😱".as_bytes();

        // 确保如果去掉最后一个符号，它不是有效的 utf8 字符串
        assert!(std::str::from_utf8(&test_buffer[..test_buffer.len() - 1]).is_err());

        pipe.write_all(&test_buffer[..test_buffer.len() - 1])
            .expect("Err write");
        pipe.flush().expect("Err flush");

        assert_eq!(pipe.buffer.len(), test_buffer.len() - 1);

        println!("len: {}, buf: {:?}", test_buffer.len(), test_buffer);
    }

    #[test]
    fn write_with_many_endls_consumes_everything_after_flush() {
        let mut pipe = LoggingPipe::new("TestPipe", 0);
        let test_buffer: &[_] = b"Testing write \n";

        pipe.write_all(
            [test_buffer, test_buffer, b"\n", b"\n", b"\n"]
                .concat()
                .as_slice(),
        )
        .expect("Err write");
        pipe.flush().expect("Err flush");

        assert_eq!(pipe.buffer.len(), 0);
    }
}
