//! 主要输入逻辑。
use crate::{
    nested_reannounce::NestedReannounce, os_input_output::ClientOsApi,
    stdin_ansi_parser::AnsiStdinInstruction, ClientId, ClientInstruction, CommandIsExecuting,
    InputInstruction,
};
use zellij_utils::{
    channels::{Receiver, SenderWithContext, OPENCALLS},
    data::{InputMode, KeyWithModifier},
    errors::{ContextType, ErrorContext, FatalError},
    input::{
        actions::Action,
        cast_termwiz_key,
        config::Config,
        mouse::{MouseEvent, MouseEventType},
        options::Options,
    },
    ipc::{ClientToServerMsg, ExitReason},
    nested_session::{self, NestedSessionMessage},
    position::Position,
    vendored::termwiz::input::{
        InputEvent, Modifiers, MouseButtons, MouseEvent as TermwizMouseEvent,
    },
};

/// 根据当前的 [InputMode] 处理 [Action] 的分发，并跟踪当前的 [InputMode]。
struct InputHandler {
    /// 当前输入模式
    mode: InputMode,
    os_input: Box<dyn ClientOsApi>,
    config: Config,
    options: Options,
    command_is_executing: CommandIsExecuting,
    send_client_instructions: SenderWithContext<ClientInstruction>,
    should_exit: bool,
    receive_input_instructions: Receiver<(InputInstruction, ErrorContext)>,
    mouse_old_event: MouseEvent,
    mouse_mode_active: bool,
    nested_reannounce: NestedReannounce,
}

fn termwiz_mouse_convert(original_event: &mut MouseEvent, event: &TermwizMouseEvent) {
    let button_bits = &event.mouse_buttons;
    original_event.left = button_bits.contains(MouseButtons::LEFT);
    original_event.right = button_bits.contains(MouseButtons::RIGHT);
    original_event.middle = button_bits.contains(MouseButtons::MIDDLE);
    original_event.wheel_up = button_bits.contains(MouseButtons::VERT_WHEEL)
        && button_bits.contains(MouseButtons::WHEEL_POSITIVE);
    original_event.wheel_down = button_bits.contains(MouseButtons::VERT_WHEEL)
        && !button_bits.contains(MouseButtons::WHEEL_POSITIVE);
    original_event.wheel_right = button_bits.contains(MouseButtons::HORZ_WHEEL)
        && button_bits.contains(MouseButtons::WHEEL_POSITIVE);
    original_event.wheel_left = button_bits.contains(MouseButtons::HORZ_WHEEL)
        && !button_bits.contains(MouseButtons::WHEEL_POSITIVE);

    let mods = &event.modifiers;
    original_event.shift = mods.contains(Modifiers::SHIFT);
    original_event.alt = mods.contains(Modifiers::ALT);
    original_event.ctrl = mods.contains(Modifiers::CTRL);
}

pub fn from_termwiz(old_event: &mut MouseEvent, event: TermwizMouseEvent) -> MouseEvent {
    // 我们使用 old_event 与 new_event 的状态来确定此事件是 Press、Release 还是 Motion。
    // 这是 pre-SGR-encoded X10 鼠标协议设计的一个不幸副作用，在该设计中 release 事件
    // 不携带关于释放了哪个按钮的信息，因此我们必须在事件之间维护一点状态。
    //
    // 请注意，只有 Left、Right 和 Middle 会在调用之间保存。WheelUp/WheelDown 通常
    // 不会生成 Release 事件。
    let mut new_event = MouseEvent::new();
    termwiz_mouse_convert(&mut new_event, &event);
    new_event.position = Position::new(event.y.saturating_sub(1) as i32, event.x.saturating_sub(1));

    if (new_event.left && !old_event.left)
        || (new_event.right && !old_event.right)
        || (new_event.middle && !old_event.middle)
        || new_event.wheel_up
        || new_event.wheel_down
        || new_event.wheel_left
        || new_event.wheel_right
    {
        // 这是一个鼠标 Press 事件。
        new_event.event_type = MouseEventType::Press;

        // 保留按钮状态。
        *old_event = new_event;
    } else if event.mouse_buttons.is_empty()
        && !old_event.left
        && !old_event.right
        && !old_event.middle
    {
        // 这是一个鼠标 Motion 事件（没有按钮按下）。
        new_event.event_type = MouseEventType::Motion;

        // 保留按钮状态。
        *old_event = new_event;
    } else if event.mouse_buttons.is_empty()
        && (old_event.left || old_event.right || old_event.middle)
    {
        // 这是一个鼠标 Release 事件。请注意，我们将 old_event.{button} 设置为 false
        // （以释放），但在向上发送事件之前，仅将被释放的 new_event 设置为 true。
        if old_event.left {
            old_event.left = false;
            new_event.left = true;
        }
        if old_event.right {
            old_event.right = false;
            new_event.right = true;
        }
        if old_event.middle {
            old_event.middle = false;
            new_event.middle = true;
        }
        new_event.event_type = MouseEventType::Release;
    } else {
        // 按住某个按钮拖动。将其作为 Motion 事件返回，并保留按钮状态。
        new_event.event_type = MouseEventType::Motion;
        *old_event = new_event;
    }

    new_event
}

impl InputHandler {
    /// 返回一个具有指定参数属性的新 [InputHandler]。
    fn new(
        os_input: Box<dyn ClientOsApi>,
        command_is_executing: CommandIsExecuting,
        config: Config,
        options: Options,
        send_client_instructions: SenderWithContext<ClientInstruction>,
        mode: InputMode, // TODO: 既然我们在服务端跟踪它，现在可能可以去掉这个了
        receive_input_instructions: Receiver<(InputInstruction, ErrorContext)>,
        nested_reannounce: NestedReannounce,
    ) -> Self {
        InputHandler {
            mode,
            os_input,
            config,
            options,
            command_is_executing,
            send_client_instructions,
            should_exit: false,
            receive_input_instructions,
            mouse_old_event: MouseEvent::new(),
            mouse_mode_active: false,
            nested_reannounce,
        }
    }

    /// 主要输入事件循环。根据当前的 [InputMode] 将终端事件解释为 [Action]，并分发这些操作。
    fn handle_input(&mut self) {
        let mut err_ctx = OPENCALLS.with(|ctx| *ctx.borrow());
        err_ctx.add_call(ContextType::StdinHandler);
        let bracketed_paste_start = vec![27, 91, 50, 48, 48, 126]; // \u{1b}[200~
        let bracketed_paste_end = vec![27, 91, 50, 48, 49, 126]; // \u{1b}[201~
        if self.options.mouse_mode.unwrap_or(true) {
            self.os_input.enable_mouse().non_fatal();
            self.mouse_mode_active = true;
        }
        loop {
            if self.should_exit {
                break;
            }
            match self.receive_input_instructions.recv() {
                Ok((InputInstruction::KeyEvent(input_event, raw_bytes), _error_context)) => {
                    match input_event {
                        InputEvent::Key(key_event) => {
                            let key = cast_termwiz_key(
                                key_event,
                                &raw_bytes,
                                Some((&self.config.keybinds, &self.mode)),
                            );
                            self.handle_key(&key, raw_bytes, false);
                        },
                        InputEvent::Mouse(mouse_event) => {
                            let mouse_event = from_termwiz(&mut self.mouse_old_event, mouse_event);
                            self.handle_mouse_event(&mouse_event);
                        },
                        InputEvent::FocusGained => {
                            self.os_input.send_to_server(
                                ClientToServerMsg::HostTerminalFocusChanged { focused: true },
                            );
                        },
                        InputEvent::FocusLost => {
                            self.os_input.send_to_server(
                                ClientToServerMsg::HostTerminalFocusChanged { focused: false },
                            );
                        },
                        InputEvent::Paste(pasted_text) => {
                            if self.mode == InputMode::Normal || self.mode == InputMode::Locked {
                                self.dispatch_action(
                                    Action::Write {
                                        key_with_modifier: None,
                                        bytes: bracketed_paste_start.clone(),
                                        is_kitty_keyboard_protocol: false,
                                    },
                                    None,
                                );
                                self.dispatch_action(
                                    Action::Write {
                                        key_with_modifier: None,
                                        bytes: pasted_text.as_bytes().to_vec(),
                                        is_kitty_keyboard_protocol: false,
                                    },
                                    None,
                                );
                                self.dispatch_action(
                                    Action::Write {
                                        key_with_modifier: None,
                                        bytes: bracketed_paste_end.clone(),
                                        is_kitty_keyboard_protocol: false,
                                    },
                                    None,
                                );
                            }
                            if self.mode == InputMode::EnterSearch {
                                self.dispatch_action(
                                    Action::SearchInput {
                                        input: pasted_text.as_bytes().to_vec(),
                                    },
                                    None,
                                );
                            }
                            if self.mode == InputMode::RenameTab {
                                self.dispatch_action(
                                    Action::TabNameInput {
                                        input: pasted_text.as_bytes().to_vec(),
                                    },
                                    None,
                                );
                            }
                            if self.mode == InputMode::RenamePane {
                                self.dispatch_action(
                                    Action::PaneNameInput {
                                        input: pasted_text.as_bytes().to_vec(),
                                    },
                                    None,
                                );
                            }
                        },
                        _ => {},
                    }
                },
                Ok((
                    InputInstruction::KeyWithModifierEvent(key_with_modifier, raw_bytes, is_kitty),
                    _error_context,
                )) => {
                    self.handle_key(&key_with_modifier, raw_bytes, is_kitty);
                },
                Ok((InputInstruction::MouseEvent(mouse_event), _error_context)) => {
                    self.handle_mouse_event(&mouse_event);
                },
                Ok((
                    InputInstruction::AnsiStdinInstructions(ansi_stdin_instructions),
                    _error_context,
                )) => {
                    for ansi_instruction in ansi_stdin_instructions {
                        self.handle_stdin_ansi_instruction(ansi_instruction);
                    }
                },
                Ok((InputInstruction::DesktopNotificationResponse(raw_bytes), _error_context)) => {
                    self.os_input
                        .send_to_server(ClientToServerMsg::DesktopNotificationResponse {
                            raw_bytes,
                        });
                },
                Ok((
                    InputInstruction::ForwardedReplyFromHostComplete { token, reply_bytes },
                    _error_context,
                )) => {
                    self.os_input
                        .send_to_server(ClientToServerMsg::ForwardedReplyFromHost {
                            token,
                            reply_bytes,
                        });
                },
                Ok((
                    InputInstruction::NestedSessionFrameFromHost(payload_bytes),
                    _error_context,
                )) => {
                    self.handle_nested_session_frame_from_host(payload_bytes);
                },
                Ok((InputInstruction::HostTerminalFocusChanged(focused), _error_context)) => {
                    self.os_input
                        .send_to_server(ClientToServerMsg::HostTerminalFocusChanged { focused });
                },
                Ok((InputInstruction::Exit, _error_context)) => {
                    self.should_exit = true;
                },
                Err(err) => panic!("Encountered read error: {:?}", err),
            }
        }
    }
    fn handle_key(
        &mut self,
        key: &KeyWithModifier,
        raw_bytes: Vec<u8>,
        is_kitty_keyboard_protocol: bool,
    ) {
        // 我们在服务端将按键解释为操作，这样我们就可以在运行时更改快捷键绑定
        self.os_input.send_to_server(ClientToServerMsg::Key {
            key: key.clone(),
            raw_bytes,
            is_kitty_keyboard_protocol,
        });
    }
    fn handle_stdin_ansi_instruction(&mut self, ansi_stdin_instructions: AnsiStdinInstruction) {
        match ansi_stdin_instructions {
            AnsiStdinInstruction::PixelDimensions(pixel_dimensions) => {
                self.os_input
                    .send_to_server(ClientToServerMsg::TerminalPixelDimensions {
                        pixel_dimensions,
                    });
            },
            AnsiStdinInstruction::BackgroundColor(background_color_instruction) => {
                self.os_input
                    .send_to_server(ClientToServerMsg::BackgroundColor {
                        color: background_color_instruction,
                    });
            },
            AnsiStdinInstruction::ForegroundColor(foreground_color_instruction) => {
                self.os_input
                    .send_to_server(ClientToServerMsg::ForegroundColor {
                        color: foreground_color_instruction,
                    });
            },
            AnsiStdinInstruction::ColorRegisters(color_registers) => {
                let color_registers: Vec<_> = color_registers
                    .into_iter()
                    .map(|(index, color)| zellij_utils::ipc::ColorRegister { index, color })
                    .collect();
                self.os_input
                    .send_to_server(ClientToServerMsg::ColorRegisters { color_registers });
            },
            AnsiStdinInstruction::SynchronizedOutput(enabled) => {
                self.send_client_instructions
                    .send(ClientInstruction::SetSynchronizedOutput(enabled))
                    .unwrap();
            },
            AnsiStdinInstruction::HostTerminalThemeChanged(mode) => {
                self.os_input
                    .send_to_server(ClientToServerMsg::HostTerminalThemeChanged { mode });
            },
            AnsiStdinInstruction::KittyGraphicsSupport(supported) => {
                self.os_input
                    .send_to_server(ClientToServerMsg::KittyGraphicsSupport { supported });
            },
            AnsiStdinInstruction::SixelSupport(supported) => {
                self.os_input
                    .send_to_server(ClientToServerMsg::SixelSupport { supported });
            },
        }
    }
    fn handle_nested_session_frame_from_host(&mut self, payload_bytes: Vec<u8>) {
        self.nested_reannounce.note_host_contact();
        match nested_session::decode_payload(&payload_bytes) {
            Some(NestedSessionMessage::Ping) => {
                self.send_client_instructions
                    .send(ClientInstruction::EmitNestedSessionFrame(
                        nested_session::encode_payload(&NestedSessionMessage::Pong),
                    ))
                    .unwrap();
            },
            Some(_) => {
                self.os_input
                    .send_to_server(ClientToServerMsg::NestedSessionFrameFromHost {
                        payload_bytes,
                    });
            },
            None => {
                log::debug!("dropping undecodable nested session frame from host");
            },
        }
    }
    fn handle_mouse_event(&mut self, mouse_event: &MouseEvent) {
        self.dispatch_action(
            Action::MouseEvent {
                event: *mouse_event,
            },
            None,
        );
    }
    /// 分发一个 [Action]。
    ///
    /// 此函数的主体决定了每个 [Action] 在分发时实际执行的操作。
    ///
    /// # 返回值
    /// 目前，此函数返回一个布尔值，指示在分发此操作后 [Self::handle_input()] 是否应该中断。
    /// 这是一个临时措施，仅由于框架的工作方式而必要，一旦测试框架修订后就不再需要了。
    /// 参见 issue#183 (https://github.com/zellij-org/zellij/issues/183)。
    fn dispatch_action(&mut self, action: Action, client_id: Option<ClientId>) -> bool {
        let mut should_break = false;

        match action {
            Action::NoOp => {},
            Action::Quit => {
                self.os_input.send_to_server(ClientToServerMsg::Action {
                    action,
                    terminal_id: None,
                    client_id,
                    is_cli_client: false,
                });
                self.exit(ExitReason::Normal);
                should_break = true;
            },
            Action::Detach => {
                self.os_input.send_to_server(ClientToServerMsg::Action {
                    action,
                    terminal_id: None,
                    client_id,
                    is_cli_client: false,
                });
                self.exit(ExitReason::NormalDetached);
                should_break = true;
            },
            Action::SwitchSession { .. } => {
                self.os_input.send_to_server(ClientToServerMsg::Action {
                    action,
                    terminal_id: None,
                    client_id,
                    is_cli_client: false,
                });
                self.exit(ExitReason::NormalDetached);
                should_break = true;
            },
            Action::CloseFocus
            | Action::SwitchToMode { .. }
            | Action::ClearScreen
            | Action::NewPane { .. }
            | Action::Run { .. }
            | Action::NewTiledPane { .. }
            | Action::NewFloatingPane { .. }
            | Action::ToggleFloatingPanes
            | Action::TogglePaneEmbedOrFloating
            | Action::NewTab { .. }
            | Action::GoToNextTab
            | Action::GoToPreviousTab
            | Action::CloseTab
            | Action::GoToTab { .. }
            | Action::MoveTab { .. }
            | Action::GoToTabName { .. }
            | Action::ToggleTab
            | Action::MoveFocusOrTab { .. } => {
                self.command_is_executing.blocking_input_thread();
                self.os_input.send_to_server(ClientToServerMsg::Action {
                    action,
                    terminal_id: None,
                    client_id,
                    is_cli_client: false,
                });
                self.command_is_executing
                    .wait_until_input_thread_is_unblocked();
            },
            Action::ToggleMouseMode => {
                if self.mouse_mode_active {
                    self.os_input.disable_mouse().non_fatal();
                    self.mouse_mode_active = false;
                } else {
                    self.os_input.enable_mouse().non_fatal();
                    self.mouse_mode_active = true;
                }
            },
            _ => self.os_input.send_to_server(ClientToServerMsg::Action {
                action,
                terminal_id: None,
                client_id,
                is_cli_client: false,
            }),
        }

        should_break
    }

    /// 输入处理器退出时要调用的例程（目前这与退出 Zellij 相同）。
    fn exit(&mut self, reason: ExitReason) {
        self.send_client_instructions
            .send(ClientInstruction::Exit(reason))
            .unwrap();
    }
}

/// 模块的入口点。实例化一个 [InputHandler] 并启动其 [InputHandler::handle_input()] 循环。
pub(crate) fn input_loop(
    os_input: Box<dyn ClientOsApi>,
    config: Config,
    options: Options,
    command_is_executing: CommandIsExecuting,
    send_client_instructions: SenderWithContext<ClientInstruction>,
    default_mode: InputMode,
    receive_input_instructions: Receiver<(InputInstruction, ErrorContext)>,
    nested_reannounce: NestedReannounce,
) {
    let _handler = InputHandler::new(
        os_input,
        command_is_executing,
        config,
        options,
        send_client_instructions,
        default_mode,
        receive_input_instructions,
        nested_reannounce,
    )
    .handle_input();
}
