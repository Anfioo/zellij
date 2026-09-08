use std::sync::OnceLock;

use crossterm::event::{self, Event, KeyEventKind};
use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
use windows_sys::Win32::System::Console::{
    GetConsoleMode, GetStdHandle, SetConsoleMode, ENABLE_EXTENDED_FLAGS, ENABLE_MOUSE_INPUT,
    ENABLE_VIRTUAL_TERMINAL_INPUT, ENABLE_WINDOW_INPUT, STD_INPUT_HANDLE,
};

use crate::InputInstruction;
use zellij_utils::channels::SenderWithContext;
use zellij_utils::input::{cast_crossterm_key, from_crossterm_mouse};
use zellij_utils::vendored::termwiz::input::InputEvent;

/// `enable_vt_input()` 修改之前保存的控制台输入模式。
/// 由 `restore_vt_input()` 用于将控制台恢复到 shell 离开时的状态，
/// 清除 crossterm 的 disable_raw_mode() 不会处理的 ENABLE_MOUSE_INPUT 等标志。
static ORIGINAL_CONSOLE_MODE: OnceLock<u32> = OnceLock::new();

/// 为原始 VT 输入设置标准输入控制台模式。
///
/// 我们不是简单地在当前模式之上 OR 入 ENABLE_VIRTUAL_TERMINAL_INPUT，
/// 而是显式设置我们需要的确切模式。这避免了与 crossterm 的 EnableMouseCapture
/// （它也执行 GetConsoleMode/SetConsoleMode）的 TOCTOU 竞态，并确保
/// ENABLE_QUICK_EDIT_MODE 等标志始终被清除 — 该标志会在控制台级别拦截
/// 鼠标事件，破坏应用程序的鼠标支持。
pub(crate) fn enable_vt_input() -> bool {
    unsafe {
        let handle = GetStdHandle(STD_INPUT_HANDLE);
        if handle.is_null() || handle == INVALID_HANDLE_VALUE {
            return false;
        }
        let mut mode: u32 = 0;
        if GetConsoleMode(handle, &mut mode) == 0 {
            return false;
        }
        // 保存原始模式，以便我们可以在退出时恢复它。
        let _ = ORIGINAL_CONSOLE_MODE.set(mode);
        // 显式设置我们需要的模式，而不是读-改-写。
        // 这消除了与 crossterm 的 EnableMouseCapture 的竞态，后者也并发调用
        // GetConsoleMode/SetConsoleMode。
        //
        // 我们设置的标志：
        //   ENABLE_WINDOW_INPUT           (0x0008) - 接收窗口调整大小事件
        //   ENABLE_MOUSE_INPUT            (0x0010) - 接收鼠标事件；在 ConPTY 上
        //                                            这会通知终端模拟器捕获并转发
        //                                            鼠标输入
        //   ENABLE_EXTENDED_FLAGS         (0x0080) - 清除 QUICK_EDIT 所必需
        //   ENABLE_VIRTUAL_TERMINAL_INPUT (0x0200) - 标准输入返回原始 VT 字节
        //
        // 我们故意清除的标志：
        //   ENABLE_PROCESSED_INPUT  (0x0001) - 让 VT 序列原始通过
        //   ENABLE_LINE_INPUT       (0x0002) - 无行缓冲
        //   ENABLE_ECHO_INPUT       (0x0004) - 无回显
        //   ENABLE_QUICK_EDIT_MODE  (0x0040) - 会拦截鼠标事件
        let new_mode = ENABLE_WINDOW_INPUT
            | ENABLE_MOUSE_INPUT
            | ENABLE_EXTENDED_FLAGS
            | ENABLE_VIRTUAL_TERMINAL_INPUT;
        if SetConsoleMode(handle, new_mode) == 0 {
            return false;
        }
        true
    }
}

/// 恢复由 `enable_vt_input()` 保存的控制台输入模式。
///
/// `crossterm::terminal::disable_raw_mode()` 仅添加回 LINE_INPUT、
/// ECHO_INPUT 和 PROCESSED_INPUT — 它从不清除 ENABLE_MOUSE_INPUT 或
/// ENABLE_VIRTUAL_TERMINAL_INPUT。如果 Zellij 退出后这些标志保持设置，
/// ConPTY 会继续将鼠标事件作为 VT 转义序列传递到 shell 的标准输入，
/// 导致可见的乱码，如 `[555;99;32M`。
pub(crate) fn restore_vt_input() {
    if let Some(&original_mode) = ORIGINAL_CONSOLE_MODE.get() {
        unsafe {
            let handle = GetStdHandle(STD_INPUT_HANDLE);
            if !handle.is_null() && handle != INVALID_HANDLE_VALUE {
                SetConsoleMode(handle, original_mode);
            }
        }
    }
}

/// Windows 原生控制台事件循环。
///
/// 使用 crossterm 的 `event::read()`，它通过 ReadConsoleInput 读取 INPUT_RECORD。
/// 在 cmd.exe、PowerShell 和 Windows Terminal 中工作，其中 ALT 被报告为修饰符标志。
///
/// 调整大小事件通过 `resize_sender` 转发到信号处理程序线程。
pub(crate) fn native_console_stdin_loop(
    send_input_instructions: SenderWithContext<InputInstruction>,
    resize_sender: Option<std::sync::mpsc::Sender<()>>,
) {
    loop {
        match event::read() {
            Ok(Event::Key(key_event)) => {
                if key_event.kind != KeyEventKind::Press {
                    continue;
                }
                if let Some((key, bytes)) = cast_crossterm_key(key_event) {
                    if send_input_instructions
                        .send(InputInstruction::KeyWithModifierEvent(key, bytes, false))
                        .is_err()
                    {
                        break;
                    }
                }
            },
            Ok(Event::Mouse(mouse_event)) => {
                let mouse_event = from_crossterm_mouse(mouse_event);
                if send_input_instructions
                    .send(InputInstruction::MouseEvent(mouse_event))
                    .is_err()
                {
                    break;
                }
            },
            Ok(Event::Paste(text)) => {
                let raw_bytes = text.as_bytes().to_vec();
                let paste_event = InputEvent::Paste(text);
                if send_input_instructions
                    .send(InputInstruction::KeyEvent(paste_event, raw_bytes))
                    .is_err()
                {
                    break;
                }
            },
            Ok(Event::Resize(..)) => {
                if let Some(ref tx) = resize_sender {
                    let _ = tx.send(());
                }
            },
            Ok(_) => {},
            Err(e) => {
                log::error!("Failed to read crossterm event: {}", e);
                let _ = send_input_instructions.send(InputInstruction::Exit);
                break;
            },
        }
    }
}
