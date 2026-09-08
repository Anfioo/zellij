use crate::keyboard_parser::{KittyKeyboardParser, KittyParseOutcome};
use crate::os_input_output::ClientOsApi;
#[cfg(windows)]
use crate::os_input_output_windows::use_vt_path;
use crate::stdin_ansi_parser::{HostReply, PendingPartial, StdinAnsiParser};
#[cfg(windows)]
use crate::stdin_handler_windows::enable_vt_input;
use crate::InputInstruction;
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

const LONE_ESC_FLUSH_INTERVAL: Duration = Duration::from_millis(50);
const PARTIAL_REPLY_FLUSH_GUARD: Duration = Duration::from_millis(1000);
use zellij_utils::{
    channels::SenderWithContext,
    vendored::termwiz::input::{InputEvent, InputParser},
};

pub(crate) fn stdin_loop(
    mut os_input: Box<dyn ClientOsApi>,
    send_input_instructions: SenderWithContext<InputInstruction>,
    stdin_ansi_parser: Arc<Mutex<StdinAnsiParser>>,
    explicitly_disable_kitty_keyboard_protocol: bool,
    support_kitty_graphics_protocol: bool,
    resize_sender: Option<std::sync::mpsc::Sender<()>>,
) {
    // 在 Windows 上，我们在下面的启动 ANSI 查询之前，尽早在 VT 字节路径
    // （termwiz/kitty 解析）和原生控制台路径（crossterm INPUT_RECORD）之间选择。
    // 触发条件参见 `use_vt_path()`。
    #[cfg(windows)]
    let use_vt_reader = use_vt_path() && enable_vt_input();

    // 发送启动主机查询字符串，以便主机终端回复其实时像素尺寸、前景/背景、
    // 同步输出支持和调色板寄存器。这些回复将在到达时由连续解析器分类，
    // 并通过 `InputInstruction::AnsiStdinInstructions` 路由 — 无截止时间、
    // 无缓存、无加载门控。
    {
        // 在 Windows 原生控制台上，crossterm event::read() 循环通过
        // ReadConsoleInput 读取 INPUT_RECORD — 不是原始字节 — 因此
        // ANSI 查询响应永远无法在该路径上读取。
        #[cfg(windows)]
        let can_query_terminal = use_vt_reader;
        #[cfg(not(windows))]
        let can_query_terminal = true;

        if can_query_terminal {
            let query_string = build_startup_query_string(support_kitty_graphics_protocol);
            let _ = os_input
                .get_stdout_writer()
                .write(query_string.as_bytes())
                .unwrap();
            if support_kitty_graphics_protocol {
                stdin_ansi_parser.lock().unwrap().expect_kitty_probe_reply();
            } else {
                let _ =
                    send_input_instructions.send(InputInstruction::AnsiStdinInstructions(vec![
                        HostReply::KittyGraphicsSupport(false),
                    ]));
            }
        } else {
            let _ = send_input_instructions.send(InputInstruction::AnsiStdinInstructions(vec![
                HostReply::KittyGraphicsSupport(false),
                HostReply::SixelSupport(false),
            ]));
        }
    }

    #[cfg(windows)]
    if !use_vt_reader {
        crate::stdin_handler_windows::native_console_stdin_loop(
            send_input_instructions,
            resize_sender,
        );
        return;
    }

    // 丢弃调整大小发送器，以便信号处理程序线程回退到轮询。
    // 只有 Windows 原生控制台路径（上面）保持它存活；
    // VT 读取器路径和 Unix 不产生 crossterm 调整大小事件。
    drop(resize_sender);

    // 字节读取器 + termwiz/kitty 解析器路径。
    // 始终在 Unix 上使用，在 Windows 上的终端模拟器（Alacritty 等）中
    // 启用 ENABLE_VIRTUAL_TERMINAL_INPUT 时使用，以便标准输入传递原始 VT
    // 字节序列。
    let mut input_parser = InputParser::new();
    // Kitty 键盘解析器是长生命周期的，因此跨标准输入读取拆分的 Kitty CSI
    // 序列仍能在后续数据块上解析，而不是静默降级为旧版 CSI 形式
    // （并丢失修饰符元数据）。
    let mut kitty_parser = KittyKeyboardParser::new();
    let mut current_buffer = vec![];
    let (stdin_tx, stdin_rx) = mpsc::sync_channel(32);
    let _stdin_pump = std::thread::Builder::new()
        .name("stdin_pump".to_string())
        .spawn({
            move || loop {
                match os_input.read_from_stdin() {
                    Ok(buf) => {
                        if stdin_tx.send(Ok(buf)).is_err() {
                            break; // 接收器已丢弃
                        }
                    },
                    Err(e) => {
                        let _ = stdin_tx.send(Err(e));
                        break;
                    },
                }
            }
        });
    let mut needs_finalization = false;
    let mut reply_in_progress_since: Option<Instant> = None;
    'stdin: loop {
        match if needs_finalization {
            stdin_rx.recv_timeout(LONE_ESC_FLUSH_INTERVAL)
        } else {
            stdin_rx
                .recv()
                .map_err(|_| mpsc::RecvTimeoutError::Disconnected)
        } {
            Ok(result) => {
                match result {
                    Ok(buf) => {
                        // 连续剥离 + 分类任何主机回复序列。
                        // 残余是键盘解析器应看到的字节流。
                        let parse_output = {
                            let mut p = stdin_ansi_parser.lock().unwrap();
                            p.feed(&buf)
                        };
                        if !parse_output.replies.is_empty() {
                            let _ = send_input_instructions.send(
                                InputInstruction::AnsiStdinInstructions(parse_output.replies),
                            );
                        }
                        if let Some((token, reply_bytes)) = parse_output.completed_forward {
                            let _ = send_input_instructions.send(
                                InputInstruction::ForwardedReplyFromHostComplete {
                                    token,
                                    reply_bytes,
                                },
                            );
                        }
                        if let Some((token, reply_bytes)) = parse_output.completed_clipboard_forward
                        {
                            let _ = send_input_instructions.send(
                                InputInstruction::ForwardedReplyFromHostComplete {
                                    token,
                                    reply_bytes,
                                },
                            );
                        }
                        for payload in parse_output.desktop_notifications {
                            let _ = send_input_instructions
                                .send(InputInstruction::DesktopNotificationResponse(payload));
                        }
                        for payload_bytes in parse_output.nested_frames {
                            let _ = send_input_instructions
                                .send(InputInstruction::NestedSessionFrameFromHost(payload_bytes));
                        }
                        let (residue, focus_changes) = extract_focus_reports(parse_output.residue);
                        for focused in focus_changes {
                            let _ = send_input_instructions
                                .send(InputInstruction::HostTerminalFocusChanged(focused));
                        }
                        if residue.is_empty() {
                            schedule_finalization(
                                &stdin_ansi_parser,
                                false,
                                &mut needs_finalization,
                                &mut reply_in_progress_since,
                            );
                            continue;
                        }
                        current_buffer.append(&mut residue.clone());

                        if !explicitly_disable_kitty_keyboard_protocol {
                            // 首先我们尝试用 KittyKeyboardParser 解析
                            // 如果失败，我们尝试正常解析。
                            // Incomplete 和 NoMatch 都回退到下面的 termwiz 解析器；
                            // 在 Incomplete 时，Kitty 解析器保持其状态，
                            // 以便下一个数据块的继续完成序列。
                            match kitty_parser.feed(&residue) {
                                KittyParseOutcome::Complete(key_with_modifier) => {
                                    if send_input_instructions
                                        .send(InputInstruction::KeyWithModifierEvent(
                                            key_with_modifier,
                                            current_buffer.drain(..).collect(),
                                            true,
                                        ))
                                        .is_err()
                                    {
                                        break 'stdin;
                                    }
                                    schedule_finalization(
                                        &stdin_ansi_parser,
                                        false,
                                        &mut needs_finalization,
                                        &mut reply_in_progress_since,
                                    );
                                    continue;
                                },
                                KittyParseOutcome::Incomplete | KittyParseOutcome::NoMatch => {},
                            }
                        }

                        // 使用 maybe_more = true 解析 - 完整事件立即发送
                        //
                        // 模糊事件（如果有）仅在 50ms 没有新输入时才最终确定
                        let maybe_more = true;
                        let mut events: Vec<(InputEvent, usize)> = vec![];
                        input_parser.parse_with_consumed(
                            &residue,
                            |input_event: InputEvent, consumed: usize| {
                                events.push((input_event, consumed));
                            },
                            maybe_more,
                        );

                        // 残余不包含 OSC 或白名单 CSI 报告 —
                        // `StdinAnsiParser::feed` 在键盘解析器看到字节之前剥离两者。
                        // 每个 termwiz 事件都是键/鼠标/粘贴等。
                        // 每个事件都以恰好产生它的字节转发，永远不会属于
                        // 从同一次读取解码的其他事件的字节。
                        for (input_event, consumed) in events.into_iter() {
                            let take = consumed.min(current_buffer.len());
                            let raw_bytes: Vec<u8> = current_buffer.drain(..take).collect();
                            if send_input_instructions
                                .send(InputInstruction::KeyEvent(input_event, raw_bytes))
                                .is_err()
                            {
                                break 'stdin;
                            }
                        }
                        realign_current_buffer(&mut current_buffer, &input_parser);

                        schedule_finalization(
                            &stdin_ansi_parser,
                            true,
                            &mut needs_finalization,
                            &mut reply_in_progress_since,
                        );
                    },
                    Err(e) => {
                        if e == "Session ended" {
                            log::debug!("Switched sessions, signing this thread off...");
                        } else {
                            log::error!("Failed to read from STDIN: {}", e);
                        }
                        let _ = send_input_instructions.send(InputInstruction::Exit);
                        break;
                    },
                }
            },
            Err(mpsc::RecvTimeoutError::Timeout) => {
                let pending = stdin_ansi_parser.lock().unwrap().pending_partial();
                if pending.uses_reply_flush_guard() {
                    let elapsed = reply_in_progress_since
                        .map(|since| since.elapsed())
                        .unwrap_or_default();
                    if elapsed >= PARTIAL_REPLY_FLUSH_GUARD {
                        let drained = stdin_ansi_parser.lock().unwrap().finalize_force();
                        drain_partial_to_keyboard(
                            &mut input_parser,
                            &mut current_buffer,
                            send_input_instructions.clone(),
                            drained,
                        );
                        needs_finalization = false;
                        reply_in_progress_since = None;
                    } else {
                        needs_finalization = true;
                    }
                } else {
                    let drained = stdin_ansi_parser.lock().unwrap().finalize_fast_partial();
                    drain_partial_to_keyboard(
                        &mut input_parser,
                        &mut current_buffer,
                        send_input_instructions.clone(),
                        drained,
                    );
                    needs_finalization = false;
                    reply_in_progress_since = None;
                }
            },
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                log::debug!("STDIN pump disconnected");
                let _ = send_input_instructions.send(InputInstruction::Exit);
                break;
            },
        }
    }
}

fn schedule_finalization(
    stdin_ansi_parser: &Arc<Mutex<StdinAnsiParser>>,
    fed_termwiz: bool,
    needs_finalization: &mut bool,
    reply_in_progress_since: &mut Option<Instant>,
) {
    let pending = stdin_ansi_parser.lock().unwrap().pending_partial();
    if fed_termwiz || pending != PendingPartial::None {
        *needs_finalization = true;
    }
    if pending.uses_reply_flush_guard() {
        if reply_in_progress_since.is_none() {
            *reply_in_progress_since = Some(Instant::now());
        }
    } else {
        *reply_in_progress_since = None;
    }
}

fn drain_partial_to_keyboard(
    input_parser: &mut InputParser,
    current_buffer: &mut Vec<u8>,
    send_input_instructions: SenderWithContext<InputInstruction>,
    drained: Vec<u8>,
) {
    if !drained.is_empty() {
        current_buffer.extend_from_slice(&drained);
    }

    let mut events: Vec<(InputEvent, usize)> = vec![];
    input_parser.parse_with_consumed(
        &drained,
        |input_event: InputEvent, consumed: usize| {
            events.push((input_event, consumed));
        },
        false,
    );
    for (input_event, consumed) in events {
        let take = consumed.min(current_buffer.len());
        let raw_bytes: Vec<u8> = current_buffer.drain(..take).collect();
        send_input_instructions
            .send(InputInstruction::KeyEvent(input_event, raw_bytes))
            .unwrap();
    }
    realign_current_buffer(current_buffer, input_parser);
}

fn extract_focus_reports(residue: Vec<u8>) -> (Vec<u8>, Vec<bool>) {
    const FOCUS_GAINED: &[u8] = b"\x1b[I";
    const FOCUS_LOST: &[u8] = b"\x1b[O";
    if residue.len() < FOCUS_GAINED.len() {
        return (residue, vec![]);
    }
    let mut remaining = Vec::with_capacity(residue.len());
    let mut focus_changes = vec![];
    let mut index = 0;
    while index < residue.len() {
        let rest = &residue[index..];
        if rest.starts_with(FOCUS_GAINED) {
            focus_changes.push(true);
            index += FOCUS_GAINED.len();
        } else if rest.starts_with(FOCUS_LOST) {
            focus_changes.push(false);
            index += FOCUS_LOST.len();
        } else {
            remaining.push(residue[index]);
            index += 1;
        }
    }
    (remaining, focus_changes)
}

/// 将 `current_buffer` 修剪为解析器自身的缓冲长度，以便它永远不会
/// 偏离解析器的内部状态：尾随的不完整序列由解析器持有（并在此处镜像），
/// 直到下一次读取完成它，而已解码为事件的字节被丢弃。
fn realign_current_buffer(current_buffer: &mut Vec<u8>, input_parser: &InputParser) {
    let buffered = input_parser.buffered_len();
    let excess = current_buffer.len().saturating_sub(buffered);
    if excess > 0 {
        current_buffer.drain(..excess);
    }
}

/// 构建客户端启动时发送的即发即忘主机查询批次。
/// 主机的回复在到达时异步优化 `Screen` 的缓存状态；界面不会阻塞它们。
fn build_startup_query_string(support_kitty_graphics_protocol: bool) -> String {
    // <ESC>[14t => 获取文本区域的像素尺寸，
    // <ESC>[16t => 获取字符单元的像素尺寸
    // <ESC>]11;?<ESC>\ => 获取背景颜色
    // <ESC>]10;?<ESC>\ => 获取前景颜色
    // <ESC>[?2026$p => 获取同步输出模式
    // <ESC>_Ga=q,...<ESC>\ => 探测 kitty 图形支持（协议禁用时省略），
    // 仅由有能力的终端回答；尾随的主 DA 是屏障，当探测未被回答时
    // 负面地解决它
    let kitty_graphics_probe = if support_kitty_graphics_protocol {
        "\u{1b}_Ga=q,i=31,s=1,v=1,t=d,f=24;AAAA\u{1b}\u{5c}"
    } else {
        ""
    };
    format!(
        "{}\u{1b}]11;?\u{1b}\u{5c}\u{1b}]10;?\u{1b}\u{5c}\u{1b}[?2026$p{}\u{1b}[c",
        PIXEL_SIZE_QUERY, kitty_graphics_probe
    )
}

pub(crate) const PIXEL_SIZE_QUERY: &str = "\u{1b}[14t\u{1b}[16t";

#[cfg(test)]
mod tests {
    use super::{
        build_startup_query_string, extract_focus_reports, realign_current_buffer, InputParser,
        PIXEL_SIZE_QUERY,
    };

    #[test]
    fn realign_after_lone_paste_start_empties_the_buffer() {
        let mut parser = InputParser::new();
        parser.parse_with_consumed(b"\x1b[200~", |_, _| {}, true);
        let mut current_buffer = b"\x1b[200~".to_vec();
        realign_current_buffer(&mut current_buffer, &parser);
        assert!(
            current_buffer.is_empty(),
            "the silently consumed paste-start bytes must not linger: {:?}",
            current_buffer
        );
    }

    #[test]
    fn realign_drops_stale_bytes_from_the_front_and_keeps_the_pending_tail() {
        let mut parser = InputParser::new();
        parser.parse_with_consumed(b"\x1b[200~hel", |_, _| {}, true);
        let mut current_buffer = b"\x1b[200~hel".to_vec();
        realign_current_buffer(&mut current_buffer, &parser);
        assert_eq!(
            current_buffer, b"hel",
            "the retained bytes must be the parser's pending tail, not the stale front"
        );
    }

    #[test]
    fn realign_is_a_no_op_when_nothing_was_consumed() {
        let mut parser = InputParser::new();
        parser.parse_with_consumed(b"\x1b[1;2", |_, _| {}, true);
        let mut current_buffer = b"\x1b[1;2".to_vec();
        realign_current_buffer(&mut current_buffer, &parser);
        assert_eq!(current_buffer, b"\x1b[1;2");
    }

    #[test]
    fn realign_tolerates_a_shorter_mirror_buffer() {
        let mut parser = InputParser::new();
        parser.parse_with_consumed(b"\x1b[1;2", |_, _| {}, true);
        let mut current_buffer = b";2".to_vec();
        realign_current_buffer(&mut current_buffer, &parser);
        assert_eq!(current_buffer, b";2");
    }

    #[test]
    fn pixel_size_query_probes_text_area_and_character_cell() {
        assert_eq!(PIXEL_SIZE_QUERY, "\u{1b}[14t\u{1b}[16t");
    }

    #[test]
    fn startup_query_has_no_palette_register_loop() {
        let query = build_startup_query_string(true);
        assert_eq!(
            query,
            "\u{1b}[14t\u{1b}[16t\u{1b}]11;?\u{1b}\u{5c}\u{1b}]10;?\u{1b}\u{5c}\u{1b}[?2026$p\u{1b}_Ga=q,i=31,s=1,v=1,t=d,f=24;AAAA\u{1b}\u{5c}\u{1b}[c"
        );
        assert!(
            !query.contains("\u{1b}]4;"),
            "startup query must not contain OSC 4 palette-register probes: {:?}",
            query
        );
    }

    #[test]
    fn focus_reports_are_extracted_from_the_byte_stream() {
        let (residue, focus_changes) = extract_focus_reports(b"a\x1b[Ib\x1b[Oc".to_vec());
        assert_eq!(residue, b"abc".to_vec());
        assert_eq!(focus_changes, vec![true, false]);
    }

    #[test]
    fn a_stream_without_focus_reports_is_left_untouched() {
        let (residue, focus_changes) = extract_focus_reports(b"\x1b[A\x1b[B".to_vec());
        assert_eq!(residue, b"\x1b[A\x1b[B".to_vec());
        assert!(focus_changes.is_empty());
    }

    #[test]
    fn startup_query_contains_kitty_probe_before_barrier() {
        let query = build_startup_query_string(true);
        assert!(query.contains("\u{1b}_Ga=q,i=31,s=1,v=1,t=d,f=24;AAAA\u{1b}\u{5c}"));
        let probe_pos = query.find("\u{1b}_Ga=q,i=31,").unwrap();
        let barrier_pos = query.find("\u{1b}[c").unwrap();
        assert!(probe_pos < barrier_pos);
    }

    #[test]
    fn startup_query_omits_kitty_probe_when_the_protocol_is_disabled() {
        let query = build_startup_query_string(false);
        assert!(!query.contains("\u{1b}_G"));
        assert!(query.ends_with("\u{1b}[c"));
        assert!(query.starts_with("\u{1b}[14t\u{1b}[16t"));
    }
}
