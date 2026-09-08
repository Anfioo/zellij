use crate::output::{CharacterChunk, KittyImageChunk, SixelImageChunk};
use crate::panes::kitty_graphics::{
    InterceptorResult, KittyApcInterceptor, KittyHostSupport, KittyImageStore,
};
use crate::panes::sixel::SixelImageStore;
use crate::panes::LinkHandler;
use crate::panes::{
    grid::{Grid, PendingNotification},
    nested_session_modal::GuestModalShortcuts,
    terminal_character::{render_first_run_banner, TerminalCharacter, EMPTY_TERMINAL_CHARACTER},
};
use crate::pty::VteBytes;
use crate::route::NotificationEnd;
use crate::tab::{AdjustedInput, GuestChoiceIndicator, Pane};
use crate::ClientId;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::rc::Rc;
use std::time::{self, Instant};
use vte;
use zellij_utils::data::PaneContents;
use zellij_utils::input::command::RunCommand;
use zellij_utils::input::mouse::{MouseEvent, MouseEventType};
use zellij_utils::pane_size::Offset;
use zellij_utils::{
    data::{
        BareKey, InputMode, KeyWithModifier, Palette, PaletteColor, PaneId as ZellijUtilsPaneId,
        RegexHighlight, Style, Styling,
    },
    errors::prelude::*,
    input::layout::Run,
    nested_session::NestedSessionMessage,
    pane_size::PaneGeom,
    pane_size::SizeInPixels,
    position::Position,
    shared::make_terminal_title,
};

use crate::ui::pane_boundaries_frame::{FrameParams, PaneFrame};

pub const SELECTION_SCROLL_INTERVAL_MS: u64 = 10;

// 以不同格式存在但在代码中使用的部分按键
const LEFT_ARROW: &[u8] = &[27, 91, 68];
const RIGHT_ARROW: &[u8] = &[27, 91, 67];
const UP_ARROW: &[u8] = &[27, 91, 65];
const DOWN_ARROW: &[u8] = &[27, 91, 66];
const HOME_KEY: &[u8] = &[27, 91, 72];
const END_KEY: &[u8] = &[27, 91, 70];
pub const BRACKETED_PASTE_BEGIN: &[u8] = &[27, 91, 50, 48, 48, 126];
pub const BRACKETED_PASTE_END: &[u8] = &[27, 91, 50, 48, 49, 126];
const ENTER_NEWLINE: &[u8] = &[10];
const ESC: &[u8] = &[27];
const ENTER_CARRIAGE_RETURN: &[u8] = &[13];
const SPACE: &[u8] = &[32];
const CTRL_C: &[u8] = &[3]; // TODO: 检查此键位是否适用于所有类型的 CTRL_C（包括 Mac 等）
const TERMINATING_STRING: &str = "\0";
const DELETE_KEY: &str = "\u{007F}";
const BACKSPACE_KEY: &str = "\u{0008}";

/// 某些按键的 ANSI 编码
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum AnsiEncoding {
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
}

impl AnsiEncoding {
    /// 返回这些条目的 ANSI 表示。
    /// 注意：字符串开头有一个 ANSI 转义码（27），
    ///       部分编辑器不会显示它
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Left => "OD".as_bytes(),
            Self::Right => "OC".as_bytes(),
            Self::Up => "OA".as_bytes(),
            Self::Down => "OB".as_bytes(),
            Self::Home => &[27, 79, 72], // ESC O H
            Self::End => &[27, 79, 70],  // ESC O F
        }
    }

    pub fn as_vec_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

#[derive(PartialEq, Eq, Ord, PartialOrd, Hash, Clone, Copy, Debug)]
pub enum PaneId {
    Terminal(u32),
    Plugin(u32), // FIXME：去掉 trait 对象，把它变成这个结构体的包装器？
}

// 因为 crate 架构等原因……
impl From<ZellijUtilsPaneId> for PaneId {
    fn from(zellij_utils_pane_id: ZellijUtilsPaneId) -> Self {
        match zellij_utils_pane_id {
            ZellijUtilsPaneId::Terminal(id) => PaneId::Terminal(id),
            ZellijUtilsPaneId::Plugin(id) => PaneId::Plugin(id),
        }
    }
}

impl Into<ZellijUtilsPaneId> for PaneId {
    fn into(self) -> ZellijUtilsPaneId {
        match self {
            PaneId::Terminal(id) => ZellijUtilsPaneId::Terminal(id),
            PaneId::Plugin(id) => ZellijUtilsPaneId::Plugin(id),
        }
    }
}

impl std::fmt::Display for PaneId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaneId::Terminal(id) => write!(f, "terminal_{}", id),
            PaneId::Plugin(id) => write!(f, "plugin_{}", id),
        }
    }
}

type IsFirstRun = bool;

// FIXME：这里应持有一个 os_api 句柄，以便终端窗格可以通过其
// `reflow_lines()` 方法中的 FD 设置自身大小。把 Box<dyn ServerOsApi> 放在这里的某处。
#[allow(clippy::too_many_arguments)]
pub struct TerminalPane {
    pub grid: Grid,
    pub pid: u32,
    pub selectable: bool,
    pub geom: PaneGeom,
    pub geom_override: Option<PaneGeom>,
    pub active_at: Instant,
    pub style: Style,
    vte_parser: vte::Parser,
    selection_scrolled_at: time::Instant,
    content_offset: Offset,
    pane_title: String,
    pane_name: String,
    prev_pane_name: String,
    frame: HashMap<ClientId, PaneFrame>,
    borderless: bool,
    exclude_from_sync: bool,
    fake_cursor_locations: HashSet<(usize, usize)>, // (x, y) - 这里记录了需要在渲染时清除的先前虚拟光标位置
    search_term: String,
    is_held: Option<(Option<i32>, IsFirstRun, RunCommand)>, // "挂起"窗格意味着它的命令要么已退出、窗格正在等待可能的
    // 用户指令以重新运行，要么该命令尚未运行
    banner: Option<String>, // 一个要在该 TerminalPane 内部渲染的横幅，用于
    // 启动时被挂起的窗格，也可用于显示某些错误
    pane_frame_color_override: Option<(PaletteColor, Option<String>)>,
    has_bell_notification: bool,
    invoked_with: Option<Run>,
    #[allow(dead_code)]
    arrow_fonts: bool,
    notification_end: Option<NotificationEnd>,
    /// 当该窗格发起的宿主终端转发尚
    /// 未完成时为 `true`。置位期间，对 `pending_pty_input` 的处理会
    /// 被挂起，以便异步宿主回复落在窗格
    /// stdin 上原始查询所占据的同一流位置。
    /// 当回复（或 500 毫秒缓存回退）到达时由 Tab 清除。
    forward_paused: bool,
    nested_guest: bool,
    guest_modal: HashMap<ClientId, usize>,
    guest_choice_indicators: HashMap<ClientId, GuestChoiceIndicator>,
    guest_session_name: Option<String>,
    guest_modal_shortcuts: GuestModalShortcuts,
    /// 尚未喂入 vte 的 PTY 字节。唯一的事实来源：
    /// `handle_pty_bytes` 总是追加到这里，而处理过程
    /// 一次弹出一个字节并推进 vte 解析器。处理
    /// 在 Grid 产生一个转发方向的查询时立即停止，将
    /// 队列中剩余的字节留待宿主回复
    /// 写入后再排空。
    pending_pty_input: VecDeque<u8>,
    kitty_interceptor: KittyApcInterceptor,
}

impl Pane for TerminalPane {
    fn x(&self) -> usize {
        self.get_x()
    }
    fn y(&self) -> usize {
        self.get_y()
    }
    fn rows(&self) -> usize {
        self.get_rows()
    }
    fn cols(&self) -> usize {
        self.get_columns()
    }
    fn get_content_x(&self) -> usize {
        self.get_x() + self.content_offset.left
    }
    fn get_content_y(&self) -> usize {
        self.get_y() + self.content_offset.top
    }
    fn get_content_columns(&self) -> usize {
        // 如果窗格有边框，内容列数可能与窗格列数不同——
        // 那种情况下会少 2
        self.get_columns()
            .saturating_sub(self.content_offset.left + self.content_offset.right)
    }
    fn get_content_rows(&self) -> usize {
        // 如果窗格有边框，内容行数可能与窗格行数不同——
        // 那种情况下会少 2
        self.get_rows()
            .saturating_sub(self.content_offset.top + self.content_offset.bottom)
    }
    fn reset_size_and_position_override(&mut self) {
        self.geom_override = None;
        self.reflow_lines();
    }
    fn set_geom(&mut self, position_and_size: PaneGeom) {
        let is_pinned = self.geom.is_pinned;
        self.geom = position_and_size;
        self.geom.is_pinned = is_pinned;
        self.reflow_lines();
        self.render_full_viewport();
    }
    fn set_geom_override(&mut self, pane_geom: PaneGeom) {
        self.geom_override = Some(pane_geom);
        self.reflow_lines();
    }
    fn handle_pty_bytes(&mut self, bytes: VteBytes) {
        self.set_should_render(true);
        if self.forward_paused {
            // 由该窗格发起的一次宿主转发尚未完成。
            // 缓冲这些字节；Tab 在恢复时会排空它们。
            self.pending_pty_input.extend(bytes);
            return;
        }
        let mut forwarded: Vec<u8> = Vec::with_capacity(bytes.len());
        let mut index = 0;
        let mut capture_started = false;
        while index < bytes.len() {
            let byte = bytes[index];
            index += 1;
            match self.kitty_interceptor.advance(byte) {
                InterceptorResult::Forward(fwd) => {
                    capture_started = false;
                    forwarded.extend_from_slice(fwd.as_slice());
                },
                InterceptorResult::Swallow => {
                    if !capture_started {
                        capture_started = true;
                        // 现在先把该序列之前的所有内容排空，这样本次冲刷
                        // 过程中的暂停绝不会使
                        // 已离开输入缓冲区的捕获数据滞留。
                        if !forwarded.is_empty() {
                            let consumed = self
                                .vte_parser
                                .advance_until_terminated(&mut self.grid, &forwarded);
                            if consumed < forwarded.len() {
                                self.pending_pty_input.extend(&forwarded[consumed..]);
                                self.pending_pty_input.extend(&bytes[index - 1..]);
                                self.kitty_interceptor.reset();
                                return;
                            }
                            forwarded.clear();
                        }
                    }
                },
                InterceptorResult::Captured(cmd) => {
                    capture_started = false;
                    if !forwarded.is_empty() {
                        let consumed = self
                            .vte_parser
                            .advance_until_terminated(&mut self.grid, &forwarded);
                        if consumed < forwarded.len() {
                            self.pending_pty_input.extend(&forwarded[consumed..]);
                            self.pending_pty_input.extend(b"\x1b_G");
                            self.pending_pty_input.extend(&cmd);
                            self.pending_pty_input.extend(b"\x1b\\");
                            self.pending_pty_input.extend(&bytes[index..]);
                            return;
                        }
                        forwarded.clear();
                    }
                    self.grid.handle_kitty_apc(&cmd);
                    if !self.grid.pending_forwarded_queries.is_empty() {
                        // Grid 产生了一个转发。停止喂入；将
                        // 尚未喂入的剩余部分排队，以便 Tab 在
                        // 回复之后重放。
                        self.pending_pty_input.extend(&bytes[index..]);
                        return;
                    }
                },
            }
        }
        let consumed = self
            .vte_parser
            .advance_until_terminated(&mut self.grid, &forwarded);
        if consumed < forwarded.len() {
            self.pending_pty_input.extend(&forwarded[consumed..]);
        }
    }
    fn cursor_coordinates(&self, client_id: Option<ClientId>) -> Option<(usize, usize, bool)> {
        // (x, y, is_visible)
        if let Some(client_id) = client_id {
            if self.guest_modal.contains_key(&client_id) {
                return None;
            }
        }
        if self.get_content_rows() < 1 || self.get_content_columns() < 1 {
            // 若没有空间则不渲染光标
            return None;
        }
        let Offset { top, left, .. } = self.content_offset;
        self.grid
            .cursor_coordinates()
            .map(|(x, y, is_visible)| (x + left, y + top, is_visible))
    }
    fn is_mid_frame(&self) -> bool {
        self.grid.is_mid_frame()
    }
    fn adjust_input_to_terminal(
        &mut self,
        key_with_modifier: &Option<KeyWithModifier>,
        raw_input_bytes: Vec<u8>,
        raw_input_bytes_are_kitty: bool,
        client_id: Option<ClientId>,
    ) -> Option<AdjustedInput> {
        // 在某些情况下，终端的当前状态意味着发送给它的输入
        // 需要被调整。
        // 这里我们匹配这些情况——如果需要，我们调整输入；如果不需要
        // 我们则原样送回输入

        self.reset_selection(client_id);
        if !self.grid.bracketed_paste_mode {
            // Zellij 本身运行在括号粘贴模式下，因此终端在粘贴输入时会发送这些
            // 指令（分别是括号粘贴开始和括号粘贴结束），
            // 我们只需要确保不将它们发送给
            // 不支持此模式的终端窗格
            match raw_input_bytes.as_slice() {
                BRACKETED_PASTE_BEGIN | BRACKETED_PASTE_END => {
                    return Some(AdjustedInput::WriteBytesToTerminal(vec![]))
                },
                _ => {},
            }
        }

        if let Some(selection) = client_id.and_then(|c| self.guest_modal.get(&c).copied()) {
            let client_id = client_id.expect("guest modal selection requires a client id");
            let is_up = key_with_modifier
                .as_ref()
                .map(|k| {
                    k.is_key_without_modifier(BareKey::Up)
                        || k.is_key_without_modifier(BareKey::Char('k'))
                })
                .unwrap_or(false)
                || raw_input_bytes.as_slice() == UP_ARROW;
            let is_down = key_with_modifier
                .as_ref()
                .map(|k| {
                    k.is_key_without_modifier(BareKey::Down)
                        || k.is_key_without_modifier(BareKey::Char('j'))
                })
                .unwrap_or(false)
                || raw_input_bytes.as_slice() == DOWN_ARROW;
            let is_enter = key_with_modifier
                .as_ref()
                .map(|k| k.is_key_without_modifier(BareKey::Enter))
                .unwrap_or(false)
                || matches!(
                    raw_input_bytes.as_slice(),
                    ENTER_CARRIAGE_RETURN | ENTER_NEWLINE
                );
            let is_esc = key_with_modifier
                .as_ref()
                .map(|k| k.is_key_without_modifier(BareKey::Esc))
                .unwrap_or(false)
                || raw_input_bytes.as_slice() == ESC;
            let digit = key_with_modifier
                .as_ref()
                .and_then(|k| match k.bare_key {
                    BareKey::Char(c @ '1'..='2') if k.key_modifiers.is_empty() => Some(c),
                    _ => None,
                })
                .or_else(|| match raw_input_bytes.as_slice() {
                    b"1" => Some('1'),
                    b"2" => Some('2'),
                    _ => None,
                });
            if is_up {
                self.guest_modal.insert(client_id, (selection + 1) % 2);
                self.set_should_render(true);
                Some(AdjustedInput::GuestModalSelectionChanged)
            } else if is_down {
                self.guest_modal.insert(client_id, (selection + 1) % 2);
                self.set_should_render(true);
                Some(AdjustedInput::GuestModalSelectionChanged)
            } else if let Some(digit) = digit {
                match digit {
                    '1' => Some(AdjustedInput::GuestModalZoom),
                    _ => Some(AdjustedInput::GuestModalDescend),
                }
            } else if is_enter {
                match selection {
                    0 => Some(AdjustedInput::GuestModalZoom),
                    _ => Some(AdjustedInput::GuestModalDescend),
                }
            } else if is_esc {
                Some(AdjustedInput::GuestModalDescend)
            } else {
                None
            }
        } else if self.is_held.is_some() {
            if key_with_modifier
                .as_ref()
                .map(|k| k.is_key_without_modifier(BareKey::Enter))
                .unwrap_or(false)
            {
                self.handle_held_run()
            } else if key_with_modifier
                .as_ref()
                .map(|k| k.is_key_without_modifier(BareKey::Esc))
                .unwrap_or(false)
            {
                self.handle_held_drop_to_shell()
            } else if key_with_modifier
                .as_ref()
                .map(|k| k.is_key_with_ctrl_modifier(BareKey::Char('c')))
                .unwrap_or(false)
            {
                Some(AdjustedInput::CloseThisPane)
            } else {
                match raw_input_bytes.as_slice() {
                    ENTER_CARRIAGE_RETURN | ENTER_NEWLINE | SPACE => self.handle_held_run(),
                    ESC => self.handle_held_drop_to_shell(),
                    CTRL_C => Some(AdjustedInput::CloseThisPane),
                    _ => None,
                }
            }
        } else {
            if self.grid.supports_kitty_keyboard_protocol {
                self.adjust_input_to_terminal_with_kitty_keyboard_protocol(
                    key_with_modifier,
                    raw_input_bytes,
                    raw_input_bytes_are_kitty,
                )
            } else {
                self.adjust_input_to_terminal_without_kitty_keyboard_protocol(
                    key_with_modifier,
                    raw_input_bytes,
                    raw_input_bytes_are_kitty,
                )
            }
        }
    }
    fn position_and_size(&self) -> PaneGeom {
        self.geom
    }
    fn current_geom(&self) -> PaneGeom {
        self.geom_override.unwrap_or(self.geom)
    }
    fn geom_override(&self) -> Option<PaneGeom> {
        self.geom_override
    }
    fn should_render(&self) -> bool {
        self.grid.should_render
    }
    fn set_should_render(&mut self, should_render: bool) {
        self.grid.should_render = should_render;
    }
    fn render_full_viewport(&mut self) {
        // 这将标记窗格进行完整重渲染，而不是像通常使用 OutputBuffer 那样只渲染
        // 差异部分
        self.frame.clear();
        self.grid.render_full_viewport();
    }
    fn selectable(&self) -> bool {
        self.selectable
    }
    fn set_selectable(&mut self, selectable: bool) {
        self.selectable = selectable;
    }
    fn set_pane_default_colors(&mut self, fg: Option<String>, bg: Option<String>) {
        self.grid.set_pane_default_colors(fg, bg);
        self.set_should_render(true);
    }
    fn get_pane_default_colors(&self) -> (Option<String>, Option<String>) {
        self.grid.get_pane_default_color_strings()
    }
    fn render(
        &mut self,
        _client_id: Option<ClientId>,
    ) -> Result<
        Option<(
            Vec<CharacterChunk>,
            Option<String>,
            Vec<SixelImageChunk>,
            Vec<KittyImageChunk>,
        )>,
    > {
        if self.should_render() {
            let content_x = self.get_content_x();
            let content_y = self.get_content_y();
            let rows = self.get_content_rows();
            let columns = self.get_content_columns();
            if rows < 1 || columns < 1 {
                return Ok(None);
            }
            match self.grid.render(content_x, content_y, &self.style) {
                Ok(rendered_assets) => {
                    self.set_should_render(false);
                    return Ok(rendered_assets);
                },
                e => return e,
            }
        } else {
            Ok(None)
        }
    }
    fn render_frame(
        &mut self,
        client_id: ClientId,
        mut frame_params: FrameParams,
        input_mode: InputMode,
    ) -> Result<Option<(Vec<CharacterChunk>, Option<String>)>> {
        let err_context = || format!("failed to render frame for client {client_id}");
        frame_params.omit_title = frame_params.omit_title
            && !(input_mode == InputMode::RenamePane && frame_params.is_main_client);
        // TODO: 从这里移除光标相关代码
        let normal_title = if self.pane_name.is_empty()
            && input_mode == InputMode::RenamePane
            && frame_params.is_main_client
        {
            String::from("Enter name...")
        } else if input_mode == InputMode::EnterSearch
            && frame_params.is_main_client
            && self.search_term.is_empty()
        {
            String::from("Enter search...")
        } else if (input_mode == InputMode::EnterSearch || input_mode == InputMode::Search)
            && !self.search_term.is_empty()
        {
            let mut modifier_text = String::new();
            if self.grid.search_results.has_modifiers_set() {
                let mut modifiers = Vec::new();
                modifier_text.push_str(" [");
                if self.grid.search_results.case_insensitive {
                    modifiers.push("c")
                }
                if self.grid.search_results.whole_word_only {
                    modifiers.push("o")
                }
                if self.grid.search_results.wrap_search {
                    modifiers.push("w")
                }
                modifier_text.push_str(&modifiers.join(", "));
                modifier_text.push(']');
            }
            format!("SEARCHING: {}{}", self.search_term, modifier_text)
        } else {
            self.current_title()
        };
        let pane_title = if frame_params.blank_title {
            String::new()
        } else if let Some(text_color_override) = self
            .pane_frame_color_override
            .as_ref()
            .and_then(|(_color, text)| text.as_ref())
        {
            text_color_override.into()
        } else {
            self.title_with_bell_indicator(normal_title)
        };

        let frame_geom = frame_params
            .frame_geom_override
            .unwrap_or_else(|| self.current_geom());
        let is_pinned = frame_geom.is_pinned;
        let mut frame = PaneFrame::new(
            frame_geom.into(),
            self.grid.scrollback_position_and_length(),
            pane_title,
            frame_params,
        )
        .is_pinned(is_pinned);
        if let Some((exit_status, is_first_run, _run_command)) = &self.is_held {
            if *is_first_run {
                frame.indicate_first_run();
            } else {
                frame.add_exit_status(exit_status.as_ref().copied());
            }
        }
        if let Some((frame_color_override, _text)) = self.pane_frame_color_override.as_ref() {
            frame.override_color(*frame_color_override);
        }

        let res = match self.frame.get(&client_id) {
            // TODO: 改用 and_then 或其他方式？
            Some(last_frame) => {
                if &frame != last_frame {
                    if !self.borderless {
                        let frame_output = frame.render().with_context(err_context)?;
                        self.frame.insert(client_id, frame);
                        Some(frame_output)
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            None => {
                if !self.borderless {
                    let frame_output = frame.render().with_context(err_context)?;
                    self.frame.insert(client_id, frame);
                    Some(frame_output)
                } else {
                    None
                }
            },
        };
        Ok(res)
    }
    fn render_fake_cursor(
        &mut self,
        cursor_color: PaletteColor,
        text_color: PaletteColor,
    ) -> Option<String> {
        let mut vte_output = None;
        if let Some((cursor_x, cursor_y, true)) = self.cursor_coordinates() {
            let mut character_under_cursor = self
                .grid
                .get_character_under_cursor()
                .unwrap_or(EMPTY_TERMINAL_CHARACTER);
            character_under_cursor.styles.update(|styles| {
                styles.background = Some(cursor_color.into());
                styles.foreground = Some(text_color.into());
            });
            // 我们跟踪这些以便稍后清除它们（见 render 函数）
            self.fake_cursor_locations.insert((cursor_y, cursor_x));
            let mut fake_cursor = format!(
                "\u{1b}[{};{}H\u{1b}[m{}",           // 跳转到指定行列并清除样式
                self.get_content_y() + cursor_y + 1, // + 1 因为 goto 从 1 开始索引
                self.get_content_x() + cursor_x + 1,
                &character_under_cursor.styles,
            );
            fake_cursor.push(character_under_cursor.character);
            vte_output = Some(fake_cursor);
        }
        vte_output
    }
    fn render_terminal_title(&mut self, input_mode: InputMode) -> String {
        let pane_title = if self.pane_name.is_empty() && input_mode == InputMode::RenamePane {
            "Enter name..."
        } else if self.pane_name.is_empty() {
            self.grid.title.as_deref().unwrap_or("")
        } else {
            &self.pane_name
        };
        make_terminal_title(pane_title)
    }
    fn update_name(&mut self, name: &str) {
        match name {
            TERMINATING_STRING => {
                self.pane_name = String::new();
            },
            DELETE_KEY | BACKSPACE_KEY => {
                self.pane_name.pop();
            },
            c => {
                self.pane_name.push_str(c);
            },
        }
        self.set_should_render(true);
    }
    fn pid(&self) -> PaneId {
        PaneId::Terminal(self.pid)
    }
    fn reduce_height(&mut self, percent: f64) {
        if let Some(p) = self.geom.rows.as_percent() {
            self.geom.rows.set_percent(p - percent);
            self.set_should_render(true);
        }
    }
    fn increase_height(&mut self, percent: f64) {
        if let Some(p) = self.geom.rows.as_percent() {
            self.geom.rows.set_percent(p + percent);
            self.set_should_render(true);
        }
    }
    fn reduce_width(&mut self, percent: f64) {
        if let Some(p) = self.geom.cols.as_percent() {
            self.geom.cols.set_percent(p - percent);
            self.set_should_render(true);
        }
    }
    fn increase_width(&mut self, percent: f64) {
        if let Some(p) = self.geom.cols.as_percent() {
            self.geom.cols.set_percent(p + percent);
            self.set_should_render(true);
        }
    }
    fn push_down(&mut self, count: usize) {
        self.geom.y += count;
        self.reflow_lines();
    }
    fn push_right(&mut self, count: usize) {
        self.geom.x += count;
        self.reflow_lines();
    }
    fn pull_left(&mut self, count: usize) {
        self.geom.x -= count;
        self.reflow_lines();
    }
    fn pull_up(&mut self, count: usize) {
        self.geom.y -= count;
        self.reflow_lines();
    }
    fn dump_screen(&self, full: bool, _client_id: Option<ClientId>) -> String {
        self.grid.dump_screen(full)
    }
    fn dump_screen_with_ansi(&self, full: bool, _client_id: Option<ClientId>) -> String {
        self.grid.dump_screen_with_ansi(full)
    }
    fn clear_screen(&mut self) {
        self.grid.clear_screen()
    }
    fn scroll_up(&mut self, count: usize, _client_id: ClientId) {
        self.grid.move_viewport_up(count);
        self.set_should_render(true);
    }
    fn scroll_down(&mut self, count: usize, _client_id: ClientId) {
        self.grid.move_viewport_down(count);
        self.set_should_render(true);
    }
    fn scroll_to_previous_prompt(&mut self, _client_id: ClientId) {
        if self.grid.scroll_to_previous_prompt() {
            self.set_should_render(true);
        }
    }
    fn scroll_to_next_prompt(&mut self, _client_id: ClientId) {
        if self.grid.scroll_to_next_prompt() {
            self.set_should_render(true);
        }
    }
    fn select_command_at_scroll_position(&mut self, _client_id: ClientId) {
        if self.grid.select_command_at_scroll_position() {
            self.set_should_render(true);
        }
    }
    fn copy_last_command_output(&mut self) -> Option<String> {
        let (output, start, end) = self.grid.last_completed_command_output()?;
        self.grid.set_command_output_flash(start, end);
        self.set_should_render(true);
        Some(output)
    }
    fn clear_command_output_flash(&mut self) {
        if self.grid.clear_command_output_flash() {
            self.set_should_render(true);
        }
    }
    fn clear_scroll(&mut self) {
        self.grid.reset_viewport();
        self.set_should_render(true);
    }
    fn is_scrolled(&self) -> bool {
        self.grid.is_scrolled
    }

    fn active_at(&self) -> Instant {
        self.active_at
    }

    fn set_active_at(&mut self, time: Instant) {
        self.active_at = time;
    }
    fn cursor_shape_csi(&self) -> String {
        self.grid.cursor_shape().get_csi_str().to_string()
    }
    fn drain_messages_to_pty(&mut self) -> Vec<Vec<u8>> {
        self.grid.pending_messages_to_pty.drain(..).collect()
    }

    fn drain_forwarded_queries(&mut self) -> Vec<crate::host_query::HostQuery> {
        self.grid.pending_forwarded_queries.drain(..).collect()
    }

    fn drain_nested_session_messages(&mut self) -> Vec<NestedSessionMessage> {
        self.grid
            .pending_nested_session_messages
            .drain(..)
            .collect()
    }

    fn is_nested_guest(&self) -> bool {
        self.nested_guest
    }

    fn set_is_nested_guest(&mut self, is_nested_guest: bool) {
        self.nested_guest = is_nested_guest;
    }

    fn set_guest_modal(&mut self, client_ids: &[ClientId]) {
        for client_id in client_ids {
            self.guest_modal.insert(*client_id, 0);
        }
        self.set_should_render(true);
    }

    fn clear_guest_modal(&mut self, client_id: ClientId) {
        if self.guest_modal.remove(&client_id).is_some() {
            self.render_full_viewport();
            self.set_should_render(true);
        }
    }

    fn clear_all_guest_modals(&mut self) {
        if !self.guest_modal.is_empty() {
            self.guest_modal.clear();
            self.render_full_viewport();
            self.set_should_render(true);
        }
    }

    fn guest_modal_selection(&self, client_id: ClientId) -> Option<usize> {
        self.guest_modal.get(&client_id).copied()
    }

    fn has_guest_modal_for_any_client(&self) -> bool {
        !self.guest_modal.is_empty()
    }

    fn set_guest_choice_indicator(
        &mut self,
        client_id: ClientId,
        indicator: Option<GuestChoiceIndicator>,
    ) {
        let previous = self.guest_choice_indicators.get(&client_id).copied();
        if previous == indicator {
            return;
        }
        match indicator {
            Some(indicator) => {
                self.guest_choice_indicators.insert(client_id, indicator);
            },
            None => {
                self.guest_choice_indicators.remove(&client_id);
            },
        }
        self.set_should_render(true);
    }

    fn guest_choice_indicator(&self, client_id: ClientId) -> Option<GuestChoiceIndicator> {
        self.guest_choice_indicators.get(&client_id).copied()
    }

    fn clear_all_guest_choice_indicators(&mut self) {
        if !self.guest_choice_indicators.is_empty() {
            self.guest_choice_indicators.clear();
            self.set_should_render(true);
        }
    }

    fn set_guest_session_name(&mut self, session_name: Option<String>) {
        self.guest_session_name = session_name;
    }

    fn guest_session_name(&self) -> Option<String> {
        self.guest_session_name.clone()
    }

    fn set_guest_modal_shortcuts(&mut self, shortcuts: GuestModalShortcuts) {
        self.guest_modal_shortcuts = shortcuts;
    }

    fn guest_modal_shortcuts(&self) -> GuestModalShortcuts {
        self.guest_modal_shortcuts.clone()
    }

    fn arm_forward_pause(&mut self) {
        self.forward_paused = true;
    }

    fn clear_forward_pause(&mut self) -> bool {
        let was_paused = self.forward_paused;
        self.forward_paused = false;
        was_paused
    }

    fn drain_pending_pty_input(&mut self) -> Vec<u8> {
        self.pending_pty_input.drain(..).collect()
    }

    fn is_forward_paused(&self) -> bool {
        self.forward_paused
    }

    fn push_color_palette_dsr(&mut self, mode: zellij_utils::data::HostTerminalThemeMode) {
        self.grid.push_color_palette_dsr(mode);
    }

    fn drain_clipboard_update(&mut self) -> Option<String> {
        self.grid.pending_clipboard_update.take()
    }

    fn drain_desktop_notifications(&mut self) -> Vec<PendingNotification> {
        self.grid.pending_desktop_notifications.drain(..).collect()
    }

    fn drain_osc7_cwd(&mut self) -> Option<std::path::PathBuf> {
        self.grid.pending_osc7_cwd.take()
    }

    fn set_selection_options(&mut self, osc133_command_selection: bool, word_separators: &str) {
        self.grid
            .set_selection_options(osc133_command_selection, word_separators);
    }

    fn start_selection(&mut self, start: &Position, _client_id: ClientId) {
        self.grid.start_selection(start);
        self.set_should_render(true);
    }

    fn update_selection(&mut self, to: &Position, _client_id: ClientId) {
        let should_scroll = self.selection_scrolled_at.elapsed()
            >= time::Duration::from_millis(SELECTION_SCROLL_INTERVAL_MS);
        let cursor_at_the_bottom = to.line.0 < 0 && should_scroll;
        let cursor_at_the_top = to.line.0 as usize >= self.grid.height && should_scroll;
        let cursor_in_the_middle = to.line.0 >= 0 && (to.line.0 as usize) < self.grid.height;

        // TODO: 检查鼠标相对窗格的上下位置，以增加滚动行数？
        if cursor_at_the_bottom {
            self.grid.scroll_up_one_line();
            self.selection_scrolled_at = time::Instant::now();
            self.set_should_render(true);
        } else if cursor_at_the_top {
            self.grid.scroll_down_one_line();
            self.selection_scrolled_at = time::Instant::now();
            self.set_should_render(true);
        } else if cursor_in_the_middle {
            // 这里我们只在选择被更新时才渲染，而这将由
            // grid 处理
            self.grid.update_selection(to);
        }
    }

    fn end_selection(&mut self, end: &Position, _client_id: ClientId) {
        self.grid.end_selection(end);
        self.set_should_render(true);
    }

    fn reset_selection(&mut self, _client_id: Option<ClientId>) {
        self.grid.reset_selection();
    }

    fn get_selected_text(&self, _client_id: ClientId) -> Option<String> {
        self.grid.get_selected_text()
    }

    fn set_frame(&mut self, _frame: bool) {
        self.frame.clear();
    }

    fn set_content_offset(&mut self, offset: Offset) {
        self.content_offset = offset;
        self.reflow_lines();
    }

    fn get_content_offset(&self) -> Offset {
        self.content_offset
    }

    fn store_pane_name(&mut self) {
        if self.pane_name != self.prev_pane_name {
            self.prev_pane_name = self.pane_name.clone()
        }
    }
    fn load_pane_name(&mut self) {
        if self.pane_name != self.prev_pane_name {
            self.pane_name = self.prev_pane_name.clone()
        }
    }

    fn borderless(&self) -> bool {
        self.borderless
    }
    fn set_borderless(&mut self, should_be_borderless: bool) {
        self.borderless = should_be_borderless;
        if should_be_borderless {
            self.set_content_offset(Offset::default());
        } else {
            self.set_content_offset(Offset::frame(1));
        }
    }

    fn set_exclude_from_sync(&mut self, exclude_from_sync: bool) {
        self.exclude_from_sync = exclude_from_sync;
    }

    fn exclude_from_sync(&self) -> bool {
        self.exclude_from_sync
    }

    fn mouse_event(&self, event: &MouseEvent, _client_id: ClientId) -> Option<String> {
        self.grid.mouse_event_signal(event)
    }

    fn mouse_left_click(&self, position: &Position, is_held: bool) -> Option<String> {
        self.grid.mouse_left_click_signal(position, is_held)
    }
    fn mouse_left_click_release(&self, position: &Position) -> Option<String> {
        self.grid.mouse_left_click_release_signal(position)
    }
    fn mouse_right_click(&self, position: &Position, is_held: bool) -> Option<String> {
        self.grid.mouse_right_click_signal(position, is_held)
    }
    fn mouse_right_click_release(&self, position: &Position) -> Option<String> {
        self.grid.mouse_right_click_release_signal(position)
    }
    fn mouse_middle_click(&self, position: &Position, is_held: bool) -> Option<String> {
        self.grid.mouse_middle_click_signal(position, is_held)
    }
    fn mouse_middle_click_release(&self, position: &Position) -> Option<String> {
        self.grid.mouse_middle_click_release_signal(position)
    }
    fn mouse_scroll_up(&self, position: &Position) -> Option<String> {
        self.grid.mouse_scroll_up_signal(position)
    }
    fn mouse_scroll_down(&self, position: &Position) -> Option<String> {
        self.grid.mouse_scroll_down_signal(position)
    }
    fn focus_event(&self) -> Option<String> {
        self.grid.focus_event()
    }
    fn unfocus_event(&self) -> Option<String> {
        self.grid.unfocus_event()
    }
    fn get_line_number(&self) -> Option<usize> {
        // + 1 因为滚动缓冲中的绝对位置从 0 索引，而这里应为 1 索引
        Some(self.grid.absolute_position_in_scrollback() + 1)
    }

    fn update_search_term(&mut self, needle: &str) {
        match needle {
            TERMINATING_STRING => {
                self.search_term = String::new();
            },
            DELETE_KEY | BACKSPACE_KEY => {
                self.search_term.pop();
            },
            c => {
                self.search_term.push_str(c);
            },
        }
        self.grid.clear_search();
        if !self.search_term.is_empty() {
            self.grid.set_search_string(&self.search_term);
        }
        self.set_should_render(true);
    }
    fn search_down(&mut self) {
        if self.search_term.is_empty() {
            return; // 空操作
        }
        self.grid.search_down();
        self.set_should_render(true);
    }
    fn search_up(&mut self) {
        if self.search_term.is_empty() {
            return; // 空操作
        }
        self.grid.search_up();
        self.set_should_render(true);
    }
    fn toggle_search_case_sensitivity(&mut self) {
        self.grid.toggle_search_case_sensitivity();
        self.set_should_render(true);
    }
    fn toggle_search_whole_words(&mut self) {
        self.grid.toggle_search_whole_words();
        self.set_should_render(true);
    }
    fn toggle_search_wrap(&mut self) {
        self.grid.toggle_search_wrap();
    }
    fn clear_search(&mut self) {
        self.grid.clear_search();
        self.search_term.clear();
    }
    fn is_alternate_mode_active(&self) -> bool {
        self.grid.is_alternate_mode_active()
    }
    fn hold(&mut self, exit_status: Option<i32>, is_first_run: bool, run_command: RunCommand) {
        self.invoked_with = Some(Run::Command(run_command.clone()));
        self.is_held = Some((exit_status, is_first_run, run_command));
        if let Some(notification_end) = self.notification_end.as_mut() {
            if let Some(exit_status) = exit_status {
                notification_end.set_exit_status(exit_status);

                // 检查是否满足解除阻塞条件
                if let Some(condition) = notification_end.unblock_condition() {
                    if condition.is_met(exit_status) {
                        // 条件满足 - 现在丢弃 NotificationEnd 以解除阻塞
                        drop(self.notification_end.take());
                    }
                }
            }
        }
        if is_first_run {
            self.render_first_run_banner();
        }
        self.set_should_render(true);
    }
    fn has_bell(&self) -> bool {
        self.grid.ring_bell
    }
    fn consume_bell(&mut self) {
        self.grid.ring_bell = false;
    }
    fn set_bell_notification(&mut self, val: bool) {
        self.has_bell_notification = val;
    }
    fn get_bell_notification(&self) -> bool {
        self.has_bell_notification
    }
    fn add_red_pane_frame_color_override(&mut self, error_text: Option<String>) {
        self.pane_frame_color_override = Some((self.style.colors.exit_code_error.base, error_text));
    }
    fn add_highlight_pane_frame_color_override(
        &mut self,
        text: Option<String>,
        _client_id: Option<ClientId>,
    ) {
        // TODO: 如果我们有 client_id，应仅为此客户端高亮边框
        self.pane_frame_color_override = Some((self.style.colors.frame_highlight.emphasis_0, text));
    }
    fn clear_pane_frame_color_override(&mut self, _client_id: Option<ClientId>) {
        // TODO: 如果我们有 client_id，应仅清除此客户端的边框高亮
        self.pane_frame_color_override = None;
    }
    fn frame_color_override(&self) -> Option<PaletteColor> {
        self.pane_frame_color_override
            .as_ref()
            .map(|(color, _text)| *color)
    }
    fn invoked_with(&self) -> &Option<Run> {
        &self.invoked_with
    }
    fn set_title(&mut self, title: String) {
        self.pane_title = title;
    }
    fn current_title(&self) -> String {
        if self.pane_name.is_empty() {
            self.grid
                .title
                .as_deref()
                .unwrap_or(&self.pane_title)
                .into()
        } else {
            self.pane_name.to_owned()
        }
    }
    fn stack_list_entry_label(&self) -> String {
        self.title_with_bell_indicator(self.current_title())
    }
    fn custom_title(&self) -> Option<String> {
        if self.pane_name.is_empty() {
            None
        } else {
            Some(self.pane_name.clone())
        }
    }
    fn has_explicit_title(&self) -> bool {
        !self.pane_name.is_empty() || self.grid.title.is_some()
    }
    fn scroll_position(&self) -> (usize, usize) {
        self.grid.scrollback_position_and_length()
    }
    fn exit_status(&self) -> Option<i32> {
        self.is_held
            .as_ref()
            .and_then(|(exit_status, _, _)| *exit_status)
    }
    fn is_held(&self) -> bool {
        self.is_held.is_some()
    }
    fn exited(&self) -> bool {
        match self.is_held {
            Some((_, is_first_run, _)) => !is_first_run,
            None => false,
        }
    }
    fn rename(&mut self, buf: Vec<u8>) {
        self.pane_name = String::from_utf8_lossy(&buf).to_string();
        self.set_should_render(true);
    }
    fn serialize(&self, scrollback_lines_to_serialize: Option<usize>) -> Option<String> {
        self.grid.serialize(scrollback_lines_to_serialize)
    }
    fn rerun(&mut self) -> Option<RunCommand> {
        // 如果这是一个已退出或等待重新运行的命令窗格，将返回其
        // RunCommand，否则可以安全地认为这不是合适的窗格类型，或者它
        // 不处于正确的状态
        self.is_held.take().map(|(_, _, run_command)| {
            self.is_held = None;
            self.grid.reset_terminal_state();
            self.set_should_render(true);
            self.remove_banner();
            run_command.clone()
        })
    }
    fn update_theme(&mut self, theme: Styling) {
        self.style.colors = theme.clone();
        self.grid.update_theme(theme);
        if self.banner.is_some() {
            // 这样做是为了让横幅用新的主题颜色更新
            self.render_first_run_banner();
        }
    }
    fn update_arrow_fonts(&mut self, should_support_arrow_fonts: bool) {
        self.arrow_fonts = should_support_arrow_fonts;
        self.grid.update_arrow_fonts(should_support_arrow_fonts);
    }
    fn update_kitty_host_support(&mut self, supported: KittyHostSupport) {
        self.grid.update_kitty_host_support(supported);
    }
    fn update_sixel_host_support(&mut self, supported: bool) {
        self.grid.update_sixel_host_support(supported);
    }
    fn update_rounded_corners(&mut self, rounded_corners: bool) {
        self.style.rounded_corners = rounded_corners;
        self.frame.clear();
    }
    fn drain_fake_cursors(&mut self) -> Option<HashSet<(usize, usize)>> {
        if !self.fake_cursor_locations.is_empty() {
            for (y, _x) in &self.fake_cursor_locations {
                // 我们这样做是因为一旦这些 fake_cursor_locations
                // 被清空，我们必须确保渲染它们
                // 出现时所在的行，这样任何清除其位置的操作
                // 都不会留下空洞
                self.grid.update_line_for_rendering(*y);
            }
            Some(self.fake_cursor_locations.drain().collect())
        } else {
            None
        }
    }
    fn toggle_pinned(&mut self) {
        self.geom.is_pinned = !self.geom.is_pinned;
    }
    fn set_pinned(&mut self, should_be_pinned: bool) {
        self.geom.is_pinned = should_be_pinned;
    }
    fn intercept_left_mouse_click(&mut self, position: &Position, client_id: ClientId) -> bool {
        if self.position_is_on_frame(position) {
            let relative_position = self.relative_position(position);
            if let Some(client_frame) = self.frame.get_mut(&client_id) {
                if client_frame.clicked_on_pinned(relative_position) {
                    self.toggle_pinned();
                    return true;
                }
            }
        }
        false
    }
    fn intercept_mouse_event_on_frame(&mut self, event: &MouseEvent, client_id: ClientId) -> bool {
        if self.position_is_on_frame(&event.position) {
            let relative_position = self.relative_position(&event.position);
            if let MouseEventType::Press = event.event_type {
                if let Some(client_frame) = self.frame.get_mut(&client_id) {
                    if client_frame.clicked_on_pinned(relative_position) {
                        self.toggle_pinned();
                        return true;
                    }
                }
            }
        }
        false
    }
    fn reset_logical_position(&mut self) {
        self.geom.logical_position = None;
    }
    fn pane_contents(
        &self,
        _client_id: Option<ClientId>,
        get_full_scrollback: bool,
        max_scrollback_lines: Option<usize>,
    ) -> PaneContents {
        self.grid
            .pane_contents(get_full_scrollback, max_scrollback_lines)
    }
    fn pane_contents_with_ansi(
        &self,
        _client_id: Option<ClientId>,
        get_full_scrollback: bool,
        max_scrollback_lines: Option<usize>,
    ) -> PaneContents {
        self.grid
            .pane_contents_with_ansi(get_full_scrollback, max_scrollback_lines)
    }
    fn update_exit_status(&mut self, exit_status: i32) {
        if let Some(notification_end) = self.notification_end.as_mut() {
            notification_end.set_exit_status(exit_status);
            // 检查是否满足解除阻塞条件
            if let Some(condition) = notification_end.unblock_condition() {
                if condition.is_met(exit_status) {
                    // 条件满足 - 现在丢弃 NotificationEnd 以解除阻塞
                    drop(self.notification_end.take());
                }
            }
        }
    }
    fn set_plugin_regex_highlights(
        &mut self,
        plugin_id: u32,
        highlights: Vec<RegexHighlight>,
        style: &Style,
    ) {
        self.grid
            .set_plugin_regex_highlights(plugin_id, highlights, style);
        self.set_should_render(true);
    }
    fn clear_plugin_highlights(&mut self, plugin_id: u32) {
        self.grid.clear_plugin_highlights(plugin_id);
        self.set_should_render(true);
    }
    fn set_hover_position(&mut self, position: Option<Position>) -> bool {
        let changed = self.grid.set_hover_position(position);
        if changed {
            self.set_should_render(true);
        }
        changed
    }
    fn cached_hover_tooltip(&self) -> Option<String> {
        self.grid.cached_hover_tooltip.clone()
    }
    fn plugin_highlight_at(
        &self,
        position: &Position,
    ) -> Option<(
        u32,
        String,
        String,
        std::collections::BTreeMap<String, String>,
    )> {
        self.grid.plugin_highlight_at(position)
    }
    fn terminal_emulator_wants_mouse(&self) -> bool {
        self.grid.mouse_tracking != crate::panes::grid::MouseTracking::Off
    }
}

impl TerminalPane {
    fn title_with_bell_indicator(&self, title: String) -> String {
        if self.has_bell_notification {
            format!("{} [!]", title)
        } else {
            title
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pid: u32,
        position_and_size: PaneGeom,
        style: Style,
        pane_index: usize,
        pane_name: String,
        link_handler: Rc<RefCell<LinkHandler>>,
        character_cell_size: Rc<RefCell<Option<SizeInPixels>>>,
        sixel_image_store: Rc<RefCell<SixelImageStore>>,
        kitty_image_store: Rc<RefCell<KittyImageStore>>,
        terminal_emulator_colors: Rc<RefCell<Palette>>,
        terminal_emulator_color_codes: Rc<RefCell<HashMap<usize, String>>>,
        initial_pane_title: Option<String>,
        invoked_with: Option<Run>,
        debug: bool,
        arrow_fonts: bool,
        styled_underlines: bool,
        osc8_hyperlinks: bool,
        explicitly_disable_keyboard_protocol: bool,
        mut notification_end: Option<NotificationEnd>,
    ) -> TerminalPane {
        let initial_pane_title =
            initial_pane_title.unwrap_or_else(|| format!("Pane #{}", pane_index));
        let grid = Grid::new(
            position_and_size.rows.as_usize(),
            position_and_size.cols.as_usize(),
            terminal_emulator_colors,
            terminal_emulator_color_codes,
            link_handler,
            character_cell_size,
            sixel_image_store,
            kitty_image_store,
            style.clone(),
            debug,
            arrow_fonts,
            styled_underlines,
            osc8_hyperlinks,
            explicitly_disable_keyboard_protocol,
        );
        if let Some(notification_end) = notification_end.as_mut() {
            notification_end.set_affected_pane_id(PaneId::Terminal(pid));
        }
        TerminalPane {
            frame: HashMap::new(),
            content_offset: Offset::default(),
            pid,
            grid,
            selectable: true,
            geom: position_and_size,
            geom_override: None,
            vte_parser: vte::Parser::new(),
            active_at: Instant::now(),
            style,
            selection_scrolled_at: time::Instant::now(),
            pane_title: initial_pane_title,
            pane_name: pane_name.clone(),
            prev_pane_name: pane_name,
            borderless: false,
            exclude_from_sync: false,
            fake_cursor_locations: HashSet::new(),
            search_term: String::new(),
            is_held: None,
            banner: None,
            pane_frame_color_override: None,
            has_bell_notification: false,
            invoked_with,
            arrow_fonts,
            notification_end,
            forward_paused: false,
            nested_guest: false,
            guest_modal: HashMap::new(),
            guest_choice_indicators: HashMap::new(),
            guest_session_name: None,
            guest_modal_shortcuts: GuestModalShortcuts::default(),
            pending_pty_input: VecDeque::new(),
            kitty_interceptor: KittyApcInterceptor::new(),
        }
    }
    pub fn get_x(&self) -> usize {
        match self.geom_override {
            Some(position_and_size_override) => position_and_size_override.x,
            None => self.geom.x,
        }
    }
    pub fn get_y(&self) -> usize {
        match self.geom_override {
            Some(position_and_size_override) => position_and_size_override.y,
            None => self.geom.y,
        }
    }
    pub fn get_columns(&self) -> usize {
        match self.geom_override {
            Some(position_and_size_override) => position_and_size_override.cols.as_usize(),
            None => self.geom.cols.as_usize(),
        }
    }
    pub fn get_rows(&self) -> usize {
        match self.geom_override {
            Some(position_and_size_override) => position_and_size_override.rows.as_usize(),
            None => self.geom.rows.as_usize(),
        }
    }
    fn reflow_lines(&mut self) {
        let rows = self.get_content_rows();
        let cols = self.get_content_columns();
        self.grid.force_change_size(rows, cols);
        if self.banner.is_some() {
            self.grid.reset_terminal_state();
            self.render_first_run_banner();
        }
        self.set_should_render(true);
    }
    pub fn read_buffer_as_lines(&self) -> Vec<Vec<TerminalCharacter>> {
        self.grid.as_character_lines()
    }
    pub fn cursor_coordinates(&self) -> Option<(usize, usize, bool)> {
        // (x, y, is_visible)
        if self.get_content_rows() < 1 || self.get_content_columns() < 1 {
            // 若没有空间则不渲染光标
            return None;
        }
        self.grid.cursor_coordinates()
    }
    fn render_first_run_banner(&mut self) {
        let columns = self.get_content_columns();
        let rows = self.get_content_rows();
        let banner = match &self.is_held {
            Some((_exit_status, _is_first_run, run_command)) => {
                render_first_run_banner(columns, rows, &self.style, Some(run_command))
            },
            None => render_first_run_banner(columns, rows, &self.style, None),
        };
        self.banner = Some(banner.clone());
        self.handle_pty_bytes(banner.as_bytes().to_vec());
    }
    fn remove_banner(&mut self) {
        if self.banner.is_some() {
            self.grid.reset_terminal_state();
            self.set_should_render(true);
            self.banner = None;
        }
    }
    fn adjust_input_to_terminal_with_kitty_keyboard_protocol(
        &self,
        key: &Option<KeyWithModifier>,
        raw_input_bytes: Vec<u8>,
        raw_input_bytes_are_kitty: bool,
    ) -> Option<AdjustedInput> {
        if raw_input_bytes_are_kitty || key.is_none() {
            Some(AdjustedInput::WriteBytesToTerminal(raw_input_bytes))
        } else {
            // 这里的情况是：宿主终端运行在非 "kitty keys" 模式下，但
            // 该终端窗格*确实*运行在 "kitty keys" 模式下——因此我们需要把 "non kitty"
            // 按键序列化为 "kitty key"
            key.as_ref()
                .and_then(|k| k.serialize_kitty())
                .map(|s| AdjustedInput::WriteBytesToTerminal(s.as_bytes().to_vec()))
        }
    }
    fn adjust_input_to_terminal_without_kitty_keyboard_protocol(
        &self,
        key: &Option<KeyWithModifier>,
        raw_input_bytes: Vec<u8>,
        raw_input_bytes_are_kitty: bool,
    ) -> Option<AdjustedInput> {
        if self.grid.new_line_mode {
            let key_is_enter = raw_input_bytes.as_slice() == &[13]
                || key
                    .as_ref()
                    .map(|k| k.is_key_without_modifier(BareKey::Enter))
                    .unwrap_or(false);
            if key_is_enter {
                // LNM - 回车后跟随换行
                return Some(AdjustedInput::WriteBytesToTerminal(
                    "\u{0d}\u{0a}".as_bytes().to_vec(),
                ));
            };
        }
        if self.grid.cursor_key_mode {
            let key_is_left_arrow = raw_input_bytes.as_slice() == LEFT_ARROW
                || key
                    .as_ref()
                    .map(|k| k.is_key_without_modifier(BareKey::Left))
                    .unwrap_or(false);
            let key_is_right_arrow = raw_input_bytes.as_slice() == RIGHT_ARROW
                || key
                    .as_ref()
                    .map(|k| k.is_key_without_modifier(BareKey::Right))
                    .unwrap_or(false);
            let key_is_up_arrow = raw_input_bytes.as_slice() == UP_ARROW
                || key
                    .as_ref()
                    .map(|k| k.is_key_without_modifier(BareKey::Up))
                    .unwrap_or(false);
            let key_is_down_arrow = raw_input_bytes.as_slice() == DOWN_ARROW
                || key
                    .as_ref()
                    .map(|k| k.is_key_without_modifier(BareKey::Down))
                    .unwrap_or(false);
            let key_is_home_key = raw_input_bytes.as_slice() == HOME_KEY
                || key
                    .as_ref()
                    .map(|k| k.is_key_without_modifier(BareKey::Home))
                    .unwrap_or(false);
            let key_is_end_key = raw_input_bytes.as_slice() == END_KEY
                || key
                    .as_ref()
                    .map(|k| k.is_key_without_modifier(BareKey::End))
                    .unwrap_or(false);
            if key_is_left_arrow {
                return Some(AdjustedInput::WriteBytesToTerminal(
                    AnsiEncoding::Left.as_vec_bytes(),
                ));
            } else if key_is_right_arrow {
                return Some(AdjustedInput::WriteBytesToTerminal(
                    AnsiEncoding::Right.as_vec_bytes(),
                ));
            } else if key_is_up_arrow {
                return Some(AdjustedInput::WriteBytesToTerminal(
                    AnsiEncoding::Up.as_vec_bytes(),
                ));
            } else if key_is_down_arrow {
                return Some(AdjustedInput::WriteBytesToTerminal(
                    AnsiEncoding::Down.as_vec_bytes(),
                ));
            } else if key_is_home_key {
                return Some(AdjustedInput::WriteBytesToTerminal(
                    AnsiEncoding::Home.as_vec_bytes(),
                ));
            } else if key_is_end_key {
                return Some(AdjustedInput::WriteBytesToTerminal(
                    AnsiEncoding::End.as_vec_bytes(),
                ));
            }
        }
        if raw_input_bytes_are_kitty {
            // 这里的情况是：宿主终端运行在 "kitty keys" 模式下，但
            // 该终端窗格不是——因此我们需要将 kitty 按键序列化为 "non kitty"，如果
            // 做不到（例如带有多个修饰键），我们将在此返回 None
            // 并且不向终端窗格写入任何内容
            key.as_ref()
                .and_then(|k| k.serialize_non_kitty())
                .map(|s| AdjustedInput::WriteBytesToTerminal(s.as_bytes().to_vec()))
        } else {
            Some(AdjustedInput::WriteBytesToTerminal(raw_input_bytes))
        }
    }
    fn handle_held_run(&mut self) -> Option<AdjustedInput> {
        self.is_held.take().map(|(_, _, run_command)| {
            self.is_held = None;
            self.grid.reset_terminal_state();
            self.set_should_render(true);
            self.remove_banner();
            AdjustedInput::ReRunCommandInThisPane(run_command.clone())
        })
    }
    fn handle_held_drop_to_shell(&mut self) -> Option<AdjustedInput> {
        self.is_held.take().map(|(_, _, run_command)| {
            // 在命令运行的同一工作目录中进入 shell
            let working_dir = run_command.cwd.clone();
            self.is_held = None;
            self.grid.reset_terminal_state();
            self.set_should_render(true);
            self.remove_banner();
            AdjustedInput::DropToShellInThisPane { working_dir }
        })
    }
}

#[cfg(test)]
#[path = "./unit/terminal_pane_tests.rs"]
mod grid_tests;

#[cfg(test)]
#[path = "./unit/search_in_pane_tests.rs"]
mod search_tests;
