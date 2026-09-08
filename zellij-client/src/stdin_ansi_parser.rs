//! 到达标准输入的主机终端回复的连续解析器。
//!
//! 此解析器通过私有的 `termwiz::InputParser` 路由标准输入字节，
//! 将 OSC / CSI 报告事件分类为 `HostReply` 变体，并让所有其他字节
//! （键盘输入）作为残余字节序列通过，调用方将其馈送到普通键盘解析器。

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use zellij_utils::{
    data::HostTerminalThemeMode,
    ipc::PixelDimensions,
    nested_session,
    pane_size::SizeInPixels,
    vendored::termwiz::input::{InputEvent, InputParser},
};

/// 描述同步输出的终端实现
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SyncOutput {
    DCS,
    CSI,
}

impl SyncOutput {
    pub fn start_seq(&self) -> &'static [u8] {
        static CSI_BSU_SEQ: &'static [u8] = "\u{1b}[?2026h".as_bytes();
        static DCS_BSU_SEQ: &'static [u8] = "\u{1b}P=1s\u{1b}".as_bytes();
        match self {
            SyncOutput::DCS => DCS_BSU_SEQ,
            SyncOutput::CSI => CSI_BSU_SEQ,
        }
    }

    pub fn end_seq(&self) -> &'static [u8] {
        static CSI_ESU_SEQ: &'static [u8] = "\u{1b}[?2026l".as_bytes();
        static DCS_ESU_SEQ: &'static [u8] = "\u{1b}P=2s\u{1b}".as_bytes();
        match self {
            SyncOutput::DCS => DCS_ESU_SEQ,
            SyncOutput::CSI => CSI_ESU_SEQ,
        }
    }
}

/// 在标准输入上接收的已分类主机终端回复。
///
/// 变体跟踪 Zellij 为其自身同步渲染热路径（像素尺寸、背景/前景、调色板寄存器、
/// 同步输出支持）消费的回复类型。累积的转发查询字节流走单独的路径：
/// 它们通过专用的输入指令通道承载 `ParseOutput::completed_forward`，
/// 以便不会与语义类型化的状态更新混合在一起。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HostReply {
    PixelDimensions(PixelDimensions),
    BackgroundColor(String),
    ForegroundColor(String),
    ColorRegisters(Vec<(usize, String)>),
    SynchronizedOutput(Option<SyncOutput>),
    /// DSR 997 回复 / 报告主机终端调色板主题模式的主动通知（CSI 2031）。
    HostTerminalThemeChanged(HostTerminalThemeMode),
    KittyGraphicsSupport(bool),
    SixelSupport(bool),
}

/// 客户端中其他模块使用的重构前类型名称的保留别名。
/// 新代码应优先使用 `HostReply`；该别名在迁移期间保持现有的
/// `InputInstruction::AnsiStdinInstructions(Vec<...>)` 管道稳定。
pub type AnsiStdinInstruction = HostReply;

impl HostReply {
    /// 将 OSC 有效载荷（`ESC ]` 前缀与 ST/BEL 终止符之间的字节）分类为
    /// 已知的 `HostReply`（如果可能）。
    pub fn from_osc_payload(payload: &[u8]) -> Option<HostReply> {
        lazy_static! {
            // OSC 10（前景）/ OSC 11（背景）回复形式：
            //   OSC 10 ; <color> ST        例如 "10;rgb:ffff/ffff/ffff"
            //   OSC 11 ; <color> ST
            static ref FG_RE: Regex = Regex::new(r"^10;(.*)$").unwrap();
            static ref BG_RE: Regex = Regex::new(r"^11;(.*)$").unwrap();
            // OSC 4 ; N ; <color> — 调色板寄存器回复。
            static ref COLOR_REGISTER_RE: Regex = Regex::new(r"^4;(\d+);(.*)$").unwrap();
        }
        let s = std::str::from_utf8(payload).ok()?;
        if let Some(caps) = BG_RE.captures(s) {
            return Some(HostReply::BackgroundColor(caps[1].to_string()));
        }
        if let Some(caps) = FG_RE.captures(s) {
            return Some(HostReply::ForegroundColor(caps[1].to_string()));
        }
        if let Some(caps) = COLOR_REGISTER_RE.captures(s) {
            let index: usize = caps[1].parse().ok()?;
            let color = caps[2].to_string();
            return Some(HostReply::ColorRegisters(vec![(index, color)]));
        }
        None
    }

    /// 将基于 CSI 的报告（包括前导 `ESC [` 的完整原始序列）分类为
    /// `HostReply`（如果可能）。
    ///
    /// 识别的终止字节：`t`（像素尺寸回复，`CSI 4/6 ; H ; W t`），
    /// `y`（对 `CSI ?2026$p` 的 DECRPM 回复 — 同步输出支持通告）。
    pub fn from_csi_report(raw: &[u8]) -> Option<HostReply> {
        let s = std::str::from_utf8(raw).ok()?;
        lazy_static! {
            // <ESC>[4;H;Wt 或 <ESC>[6;H;Wt
            static ref PIX_RE: Regex = Regex::new(r"^\u{1b}\[(\d+);(\d+);(\d+)t$").unwrap();
            // <ESC>[?2026;Ny — 同步输出的 DECRPM 回复（VT 模式 2026）
            static ref SYNC_RE: Regex = Regex::new(r"^\u{1b}\[\?2026;([0-4])\$y$").unwrap();
            // <ESC>[?997;1n（深色）/ <ESC>[?997;2n（浅色）— 对 CSI ?996n 的 DSR 997 回复，
            // 或启用 CSI ?2031h 时的主动主机主题通知。
            static ref THEME_RE: Regex = Regex::new(r"^\u{1b}\[\?997;([12])n$").unwrap();
        }
        if let Some(caps) = PIX_RE.captures(s) {
            let which: usize = caps[1].parse().ok()?;
            let first: usize = caps[2].parse().ok()?;
            let second: usize = caps[3].parse().ok()?;
            return match which {
                4 => Some(HostReply::PixelDimensions(PixelDimensions {
                    character_cell_size: None,
                    text_area_size: Some(SizeInPixels {
                        height: first,
                        width: second,
                    }),
                })),
                6 => Some(HostReply::PixelDimensions(PixelDimensions {
                    character_cell_size: Some(SizeInPixels {
                        height: first,
                        width: second,
                    }),
                    text_area_size: None,
                })),
                _ => None,
            };
        }
        if let Some(caps) = SYNC_RE.captures(s) {
            let code: usize = caps[1].parse().ok()?;
            return match code {
                1 | 2 | 3 => Some(HostReply::SynchronizedOutput(Some(SyncOutput::CSI))),
                _ => Some(HostReply::SynchronizedOutput(None)),
            };
        }
        if let Some(caps) = THEME_RE.captures(s) {
            let mode = match &caps[1] {
                "1" => HostTerminalThemeMode::Dark,
                "2" => HostTerminalThemeMode::Light,
                _ => return None,
            };
            return Some(HostReply::HostTerminalThemeChanged(mode));
        }
        None
    }

    /// 将主设备属性回复（`CSI ? Ps ; Ps ... c`）分类为 Sixel 能力通告。
    /// 回复中的属性 `4` 表示主机终端支持 Sixel 图形。
    ///
    /// 不是主 DA 形式的回复（例如次 DA `CSI > ... c`）产生 `None`，
    /// 以便它们不会破坏主机能力状态。
    pub fn sixel_support_from_primary_da(raw: &[u8]) -> Option<HostReply> {
        lazy_static! {
            static ref PRIMARY_DA_RE: Regex = Regex::new(r"^\u{1b}\[\?([0-9;]*)c$").unwrap();
        }
        let s = std::str::from_utf8(raw).ok()?;
        let caps = PRIMARY_DA_RE.captures(s)?;
        let supports_sixel = caps[1].split(';').any(|attribute| attribute == "4");
        Some(HostReply::SixelSupport(supports_sixel))
    }
}

/// 当前正在向主机终端进行的单个转发查询的"槽"跟踪状态。
/// 解析器将原始回复字节累积到 `reply_bytes` 中，直到看到主 DA（`c`）回复，
/// 该回复充当序列化屏障。强制执行 500ms 截止时间的计时器位于
/// forward-timeout 运行时上，并拥有自己的挂钟 — 解析器本身与截止时间无关。
#[derive(Debug, Clone)]
pub struct ForwardSlot {
    pub token: u32,
    pub reply_bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ClipboardForwardSlot {
    pub token: u32,
}

pub const CLIENT_CLIPBOARD_FORWARD_TIMEOUT_MS: u64 = 30_000;

/// `feed()` 的返回值。
#[derive(Debug, Clone, Default)]
pub struct ParseOutput {
    /// 已分类的主机回复（零个或多个）。
    pub replies: Vec<HostReply>,
    /// 已完成的转发回复（看到主 DA 屏障），准备发送到服务端。
    /// 每次 feed 调用至多一个；单次 feed 中超过一个表示主机发出了两个屏障，
    /// 在这种情况下仅遵循第一个。
    pub completed_forward: Option<(u32, Vec<u8>)>,
    pub completed_clipboard_forward: Option<(u32, Vec<u8>)>,
    /// OSC 99 通知响应有效载荷（在数据块中找到的每个 OSC 99 一个）。
    /// 由调用方作为 `InputInstruction::DesktopNotificationResponse` 路由。
    /// 位于此处，而不是键盘解析器路径，因为残余清理器在键盘解析器看到
    /// 所有 OSC 字节之前就将其剥离。
    pub desktop_notifications: Vec<Vec<u8>>,
    /// 未被分类为主机回复的残余字节。这些是调用方应馈送到键盘解析器的字节。
    pub residue: Vec<u8>,
    pub nested_frames: Vec<Vec<u8>>,
    /// 当解析器仍持有缓冲的部分 OSC/CSI 字节，需要空闲刷新来释放时为 `true`。
    /// 调用方使用它来调度完成滴答，即使当前数据块没有产生残余
    /// （这样单独的尾随 ESC 不会无限期滞留）。参见 `StdinAnsiParser::finalize`。
    pub has_partial_state: bool,
}

fn is_user_input(event: &InputEvent) -> bool {
    match event {
        InputEvent::Key(_) | InputEvent::Paste(_) => true,
        InputEvent::Mouse(mouse_event) => !mouse_event.mouse_buttons.is_empty(),
        InputEvent::PixelMouse(mouse_event) => !mouse_event.mouse_buttons.is_empty(),
        _ => false,
    }
}

/// 进行中的部分 OSC/CSI 缓冲区大小上限。大小设定为能通过合法的 OSC 52
/// 剪贴板有效载荷，这些有效载荷携带整个剪贴板的 base64 编码，且没有协议级限制
/// （图像、多 MB 文本转储等）。超过此限制我们假设是失控或格式错误的序列，
/// 并将缓冲的字节刷新回残余 — 与今天对未终止 OSC 的可观察行为相同，只是有界。
const PARTIAL_BUFFER_CAP_BYTES: usize = 100 * 1024 * 1024;

const PASTE_START_MARKER: &[u8] = b"\x1b[200~";
const PASTE_END_MARKER: &[u8] = b"\x1b[201~";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingPartial {
    None,
    LoneEsc,
    BareIntroducer,
    ReplyInProgress,
}

impl PendingPartial {
    pub fn uses_reply_flush_guard(&self) -> bool {
        matches!(self, PendingPartial::ReplyInProgress)
    }
}

#[derive(Debug, Clone, Copy)]
enum MarkerMatch {
    Complete,
    Prefix,
    No,
}

fn marker_match(buf: &[u8], marker: &[u8]) -> MarkerMatch {
    if buf.len() >= marker.len() {
        if &buf[..marker.len()] == marker {
            MarkerMatch::Complete
        } else {
            MarkerMatch::No
        }
    } else if marker.starts_with(buf) {
        MarkerMatch::Prefix
    } else {
        MarkerMatch::No
    }
}

/// 单次 OSC/CSI 遍历字节缓冲区的结果。区分"需要更多字节"与"格式错误"
/// 是让残余清理器跨 `feed()` 调用缓冲部分序列，而不是将其字节泄漏到
/// 键盘残余中的关键。
#[derive(Debug, Clone, Copy)]
enum SeqStatus {
    /// 序列已完成；从 buf 头部消耗 `len` 字节。
    Complete(usize),
    /// 序列是有效的前缀；调用方应缓冲这些字节并将它们前置到下一个数据块。
    NeedMore,
    /// 序列格式错误（有效载荷中间的裸 ESC、非白名单终止字节、达到长度上限）。
    /// 调用方应回退到将前导字节作为残余发出。
    Malformed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StartupKittyProbe {
    NotSent,
    AwaitingReply,
    Resolved,
}

/// 连续主机回复解析器。在整个客户端会话期间存活。
pub struct StdinAnsiParser {
    inner: InputParser,
    /// 活动转发槽：转发查询进行中时为 `Some`，否则为 `None`。
    active_forward: Option<ForwardSlot>,
    active_clipboard_forward: Option<ClipboardForwardSlot>,
    /// 终止符尚未到达的 OSC 序列的字节。
    /// 跨 feed() 调用携带，以便下一个数据块可以完成它。
    partial_osc: Vec<u8>,
    /// CSI 设备控制报告同理。
    partial_csi: Vec<u8>,
    partial_paste: Vec<u8>,
    nested_frame_extractor: nested_session::NestedFrameExtractor,
    in_bracketed_paste: bool,
    startup_kitty_probe: StartupKittyProbe,
    partial_apc: Vec<u8>,
}

impl std::fmt::Debug for StdinAnsiParser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StdinAnsiParser")
            .field("active_forward", &self.active_forward)
            .field("partial_osc_len", &self.partial_osc.len())
            .field("partial_csi_len", &self.partial_csi.len())
            .finish()
    }
}

impl StdinAnsiParser {
    pub fn new() -> Self {
        StdinAnsiParser {
            inner: InputParser::new(),
            active_forward: None,
            active_clipboard_forward: None,
            partial_osc: Vec::new(),
            partial_csi: Vec::new(),
            partial_paste: Vec::new(),
            nested_frame_extractor: nested_session::NestedFrameExtractor::new(),
            in_bracketed_paste: false,
            startup_kitty_probe: StartupKittyProbe::NotSent,
            partial_apc: Vec::new(),
        }
    }

    pub fn expect_kitty_probe_reply(&mut self) {
        self.startup_kitty_probe = StartupKittyProbe::AwaitingReply;
    }

    /// 为 `token` 打开转发窗口。在主 DA 屏障之前到达的后续回复事件将被累积到
    /// 槽的 `reply_bytes` 中，此外还会作为正常的已分类 `HostReply` 事件分发。
    ///
    /// 服务端全局序列化转发查询（`Screen` 上的 `forward_in_flight`），但其自身的
    /// 回退超时可能会释放该槽并在此客户端的每槽计时器运行之前分派下一个查询，
    /// 因此调用方首先用 `take_active_forward` 移交仍打开的槽。下面的守卫捕获
    /// 跳过该移交的调用方：调试版本会 panic 以便错误在测试期间暴露，发布版本
    /// 记录并破坏前一个槽（其累积的字节否则会静默泄漏）。
    pub fn open_forward(&mut self, token: u32) {
        debug_assert!(
            self.active_forward.is_none(),
            "open_forward({}) called while slot for token {:?} is still active",
            token,
            self.active_forward.as_ref().map(|s| s.token),
        );
        if let Some(existing) = self.active_forward.as_ref() {
            log::warn!(
                "open_forward({}) re-entered with existing slot token={} ({} accumulated bytes \
                 will be dropped); server serialization should have prevented this",
                token,
                existing.token,
                existing.reply_bytes.len(),
            );
        }
        self.active_forward = Some(ForwardSlot {
            token,
            reply_bytes: Vec::new(),
        });
    }

    /// 在没有屏障的情况下关闭活动转发窗口（超时路径）。
    /// 返回累积的回复字节和令牌（如果有）。
    pub fn close_forward_on_timeout(&mut self, token: u32) -> Option<(u32, Vec<u8>)> {
        match &self.active_forward {
            Some(slot) if slot.token == token => {
                let slot = self.active_forward.take().unwrap();
                Some((slot.token, slot.reply_bytes))
            },
            _ => None,
        }
    }

    /// 关闭当前打开的任何转发窗口，无论它属于哪个令牌，并返回其令牌以及
    /// 它累积的字节。用于将服务端已经放弃的槽移交给替换它的转发，而不是破坏它。
    pub fn take_active_forward(&mut self) -> Option<(u32, Vec<u8>)> {
        self.active_forward
            .take()
            .map(|slot| (slot.token, slot.reply_bytes))
    }

    pub fn open_clipboard_forward(&mut self, token: u32) {
        debug_assert!(
            self.active_clipboard_forward.is_none(),
            "open_clipboard_forward({}) called while slot for token {:?} is still active",
            token,
            self.active_clipboard_forward.as_ref().map(|s| s.token),
        );
        self.active_clipboard_forward = Some(ClipboardForwardSlot { token });
    }

    pub fn close_clipboard_forward_on_timeout(&mut self, token: u32) -> Option<(u32, Vec<u8>)> {
        match &self.active_clipboard_forward {
            Some(slot) if slot.token == token => {
                let slot = self.active_clipboard_forward.take().unwrap();
                Some((slot.token, Vec::new()))
            },
            _ => None,
        }
    }

    pub fn take_active_clipboard_forward(&mut self) -> Option<(u32, Vec<u8>)> {
        self.active_clipboard_forward
            .take()
            .map(|slot| (slot.token, Vec::new()))
    }

    #[cfg(test)]
    pub fn active_clipboard_forward_token(&self) -> Option<u32> {
        self.active_clipboard_forward.as_ref().map(|s| s.token)
    }

    /// 当前打开的槽的令牌（如果有）。仅测试检查器；
    /// 生产代码直接通过 `open_forward`、`close_forward_on_timeout` 和 `feed()`
    /// 驱动槽生命周期。
    #[cfg(test)]
    pub fn active_forward_token(&self) -> Option<u32> {
        self.active_forward.as_ref().map(|s| s.token)
    }

    /// 消费一块原始标准输入字节。返回已分类的主机回复（将分发到服务端的
    /// 缓存状态消费者）、至多一个已完成的转发回复（屏障关闭了窗口），以及
    /// 不属于任何已分类序列的残余字节 — 这些是调用方应馈送到键盘解析器的字节。
    pub fn feed(&mut self, bytes: &[u8]) -> ParseOutput {
        let mut out = ParseOutput::default();
        let (bytes, nested_frames) = self.nested_frame_extractor.extract(bytes);
        let bytes = &bytes[..];
        out.nested_frames = nested_frames;
        let sanitized = self.extract_kitty_probe_reply(bytes, &mut out.replies);
        // 首先收集事件（在回调和后处理突变之间借用拆分 InputParser）。
        let mut events = Vec::new();
        let mut residue = Vec::new();
        self.inner.parse(
            &sanitized,
            |event| {
                events.push(event);
            },
            true, // maybe_more — 典型的流用法
        );
        for event in events {
            match event {
                InputEvent::OperatingSystemCommand(payload) => {
                    // OSC 99（桌面通知响应）在此处路由，而不是从键盘解析器路由，
                    // 因为残余清理器在键盘解析器运行之前移除了所有 OSC 字节。
                    // 其他 OSC 被分类为 `HostReply` 用于缓存状态优化。
                    if payload.starts_with(b"99;") {
                        out.desktop_notifications
                            .push(payload.get(3..).unwrap_or_default().to_vec());
                        continue;
                    } else if let Some(reply) = HostReply::from_osc_payload(&payload) {
                        out.replies.push(reply);
                    }
                    if payload.starts_with(b"52;") {
                        if let Some(slot) = self.active_clipboard_forward.take() {
                            let mut bytes = b"\x1b]".to_vec();
                            bytes.extend_from_slice(&payload);
                            bytes.extend_from_slice(b"\x1b\\");
                            out.completed_clipboard_forward = Some((slot.token, bytes));
                            continue;
                        }
                    }
                    if let Some(slot) = self.active_forward.as_mut() {
                        // 重新序列化，以便窗格的 PTY 看到合法的 OSC。
                        // 终止符因主机而异；ST（ESC \）始终安全。
                        slot.reply_bytes.extend_from_slice(b"\x1b]");
                        slot.reply_bytes.extend_from_slice(&payload);
                        slot.reply_bytes.extend_from_slice(b"\x1b\\");
                    }
                },
                InputEvent::DeviceControlReply {
                    params,
                    final_byte,
                    raw,
                    ..
                } => {
                    match final_byte {
                        b'c' => {
                            if self.startup_kitty_probe == StartupKittyProbe::AwaitingReply {
                                self.startup_kitty_probe = StartupKittyProbe::Resolved;
                                out.replies.push(HostReply::KittyGraphicsSupport(false));
                            }
                            if let Some(reply) = HostReply::sixel_support_from_primary_da(&raw) {
                                out.replies.push(reply);
                            }
                            // 主 DA — 屏障。关闭槽并在活动时发出已完成的转发回复。
                            if let Some(slot) = self.active_forward.take() {
                                out.completed_forward = Some((slot.token, slot.reply_bytes));
                            }
                            // 主 DA 不会被双重分发 — 它没有缓存状态对应物。
                        },
                        _ => {
                            if let Some(reply) = HostReply::from_csi_report(&raw) {
                                out.replies.push(reply);
                            }
                            if let Some(slot) = self.active_forward.as_mut() {
                                slot.reply_bytes.extend_from_slice(&raw);
                            }
                            // 抑制 params 的未使用变量警告。
                            let _ = params;
                        },
                    }
                },
                // 其他所有内容都是键盘 / 鼠标 / 粘贴 / 唤醒输入；
                // 我们需要这些字节到达键盘解析器。我们无法在此处从已解析的事件
                // 重建确切的字节，因此我们依赖调用方自己对键盘解析器的第二次遍历：
                // 残余是所有不属于已分类回复的输入字节的串联。为了确定性地
                // 产生该残余，我们在下面第二次重新扫描缓冲区。
                other => {
                    if self.active_clipboard_forward.is_some()
                        && is_user_input(&other)
                        && out.completed_clipboard_forward.is_none()
                    {
                        if let Some(slot) = self.active_clipboard_forward.take() {
                            out.completed_clipboard_forward = Some((slot.token, Vec::new()));
                        }
                    }
                },
            }
        }
        // 产生残余：通过临时解析器重放输入，该解析器剥离 OSC 有效载荷和
        // 白名单 CSI 报告。所有其他字节原样通过。遍历是有状态的，因此
        // 跨 `feed()` 调用拆分的 OSC/CSI 序列被缓冲，而不是泄漏到残余中。
        residue.extend(self.strip_replies(&sanitized));
        out.residue = residue;
        out.has_partial_state = !self.partial_osc.is_empty()
            || !self.partial_csi.is_empty()
            || !self.partial_paste.is_empty()
            || !self.nested_frame_extractor.partial_bytes().is_empty()
            || !self.partial_apc.is_empty();
        out
    }

    pub fn pending_partial(&self) -> PendingPartial {
        if !self.partial_paste.is_empty() || !self.partial_apc.is_empty() {
            PendingPartial::ReplyInProgress
        } else if self.partial_csi.is_empty()
            && self.partial_osc.is_empty()
            && self.nested_frame_extractor.partial_bytes() == [0x1b]
        {
            PendingPartial::LoneEsc
        } else if !self.nested_frame_extractor.partial_bytes().is_empty() {
            PendingPartial::ReplyInProgress
        } else if self.partial_csi.is_empty() && self.partial_osc.is_empty() {
            PendingPartial::None
        } else if self.partial_csi.is_empty() && self.partial_osc == [0x1b] {
            PendingPartial::LoneEsc
        } else if self.partial_csi.is_empty() && self.partial_osc == [0x1b, b']'] {
            PendingPartial::BareIntroducer
        } else if self.partial_osc.is_empty() && self.partial_csi == [0x1b, b'['] {
            PendingPartial::BareIntroducer
        } else {
            PendingPartial::ReplyInProgress
        }
    }

    pub fn finalize_fast_partial(&mut self) -> Vec<u8> {
        match self.pending_partial() {
            PendingPartial::LoneEsc | PendingPartial::BareIntroducer => {
                if !self.nested_frame_extractor.partial_bytes().is_empty() {
                    self.nested_frame_extractor.take_partial()
                } else if !self.partial_osc.is_empty() {
                    std::mem::take(&mut self.partial_osc)
                } else {
                    std::mem::take(&mut self.partial_csi)
                }
            },
            PendingPartial::None | PendingPartial::ReplyInProgress => Vec::new(),
        }
    }

    pub fn finalize_force(&mut self) -> Vec<u8> {
        self.in_bracketed_paste = false;
        let mut partial_nested_frame = self.nested_frame_extractor.take_partial();
        let mut out = Vec::with_capacity(
            self.partial_osc.len()
                + self.partial_csi.len()
                + self.partial_paste.len()
                + partial_nested_frame.len()
                + self.partial_apc.len(),
        );
        out.append(&mut self.partial_osc);
        out.append(&mut self.partial_csi);
        out.append(&mut self.partial_paste);
        out.append(&mut partial_nested_frame);
        out.append(&mut self.partial_apc);
        out
    }

    fn extract_kitty_probe_reply(&mut self, bytes: &[u8], replies: &mut Vec<HostReply>) -> Vec<u8> {
        let mut working = std::mem::take(&mut self.partial_apc);
        if working.is_empty() && self.partial_osc == [0x1b] && bytes.first() == Some(&b'_') {
            working.append(&mut self.partial_osc);
        }
        working.extend_from_slice(bytes);
        let mut out = Vec::with_capacity(working.len());
        let mut i = 0;
        while i < working.len() {
            let rest = &working[i..];
            if rest.len() >= 2 && rest[0] == 0x1b && rest[1] == b'_' {
                match rest.get(2) {
                    Some(&b'G') => {},
                    Some(_) => {
                        out.push(working[i]);
                        i += 1;
                        continue;
                    },
                    None => {
                        self.partial_apc = rest.to_vec();
                        return out;
                    },
                }
                match apc_status(rest) {
                    SeqStatus::Complete(len) => {
                        let payload = &rest[2..len - 2];
                        if self.startup_kitty_probe == StartupKittyProbe::AwaitingReply
                            && payload.first() == Some(&b'G')
                            && contains_subslice(payload, b"i=31")
                        {
                            replies.push(HostReply::KittyGraphicsSupport(contains_subslice(
                                payload, b"i=31;OK",
                            )));
                            self.startup_kitty_probe = StartupKittyProbe::Resolved;
                        }
                        i += len;
                        continue;
                    },
                    SeqStatus::NeedMore => {
                        let tail = rest.to_vec();
                        if tail.len() > PARTIAL_BUFFER_CAP_BYTES {
                            out.extend_from_slice(&tail);
                        } else {
                            self.partial_apc = tail;
                        }
                        return out;
                    },
                    SeqStatus::Malformed => {
                        out.push(working[i]);
                        i += 1;
                        continue;
                    },
                }
            }
            out.push(working[i]);
            i += 1;
        }
        out
    }

    /// 遍历 `bytes`（前置任何挂起的部分缓冲区）并丢弃任何 OSC/白名单 CSI
    /// 序列，原样返回剩余字节（键盘残余）。这是一个字节级清理器 —
    /// 它不产生事件，只产生字节。
    ///
    /// 如果数据块在序列中间结束，未终止的尾部保存在 `self.partial_osc`
    /// 或 `self.partial_csi` 中，并前置到下一次调用的输入 — 因此相应的字节
    /// 在等待序列其余部分时永远不会到达残余（也永远不会作为虚假按键出现）。
    fn strip_replies(&mut self, bytes: &[u8]) -> Vec<u8> {
        // 前置任何挂起的部分。(partial_osc, partial_csi) 中至多一个在任何时候
        // 非空 — 前一次遍历要么完成了所有序列，要么停在恰好一个未终止的尾部。
        let mut working: Vec<u8> = Vec::with_capacity(
            self.partial_osc.len()
                + self.partial_csi.len()
                + self.partial_paste.len()
                + bytes.len(),
        );
        working.append(&mut self.partial_osc);
        working.append(&mut self.partial_csi);
        working.append(&mut self.partial_paste);
        working.extend_from_slice(bytes);

        let mut out = Vec::with_capacity(working.len());
        let mut i = 0;
        while i < working.len() {
            let rest = &working[i..];
            if self.in_bracketed_paste {
                if rest[0] == 0x1b {
                    match marker_match(rest, PASTE_END_MARKER) {
                        MarkerMatch::Complete => {
                            out.extend_from_slice(PASTE_END_MARKER);
                            self.in_bracketed_paste = false;
                            i += PASTE_END_MARKER.len();
                            continue;
                        },
                        MarkerMatch::Prefix => {
                            let tail = rest.to_vec();
                            if tail.len() > PARTIAL_BUFFER_CAP_BYTES {
                                out.extend_from_slice(&tail);
                            } else {
                                self.partial_paste = tail;
                            }
                            return out;
                        },
                        MarkerMatch::No => {},
                    }
                }
                out.push(working[i]);
                i += 1;
                continue;
            }
            if rest.len() >= 2 && rest[0] == 0x1b && rest[1] == b'[' {
                if let MarkerMatch::Complete = marker_match(rest, PASTE_START_MARKER) {
                    out.extend_from_slice(PASTE_START_MARKER);
                    self.in_bracketed_paste = true;
                    i += PASTE_START_MARKER.len();
                    continue;
                }
            }
            // OSC: ESC ] ... (BEL | ESC \)
            if rest.len() >= 2 && rest[0] == 0x1b && rest[1] == b']' {
                match osc_status(rest) {
                    SeqStatus::Complete(len) => {
                        i += len;
                        continue;
                    },
                    SeqStatus::NeedMore => {
                        let tail = rest.to_vec();
                        if tail.len() > PARTIAL_BUFFER_CAP_BYTES {
                            // 超过上限：将缓冲的字节刷新到残余并重置，
                            // 保留未终止字节不被静默吞掉的语义。
                            out.extend_from_slice(&tail);
                        } else {
                            self.partial_osc = tail;
                        }
                        return out;
                    },
                    SeqStatus::Malformed => {
                        out.push(working[i]);
                        i += 1;
                        continue;
                    },
                }
            }
            // 白名单 CSI 报告: ESC [ <params>* <intermediates>* <final>
            if rest.len() >= 2 && rest[0] == 0x1b && rest[1] == b'[' {
                match csi_status(rest) {
                    SeqStatus::Complete(len) => {
                        i += len;
                        continue;
                    },
                    SeqStatus::NeedMore => {
                        let tail = rest.to_vec();
                        if tail.len() > PARTIAL_BUFFER_CAP_BYTES {
                            out.extend_from_slice(&tail);
                        } else {
                            self.partial_csi = tail;
                        }
                        return out;
                    },
                    SeqStatus::Malformed => {
                        out.push(working[i]);
                        i += 1;
                        continue;
                    },
                }
            }
            // 尾部的单独尾随 ESC — 可能是 OSC 或 CSI 的开始；下一个字节将消除歧义。
            // 按约定将其缓冲在 partial_osc 下；下一次调用的遍历器根据实际的
            // 第二个字节重新路由。
            if rest.len() == 1 && rest[0] == 0x1b {
                self.partial_osc = vec![0x1b];
                return out;
            }
            out.push(working[i]);
            i += 1;
        }
        out
    }
}

/// 遍历从 `buf` 头部开始的 OSC 序列。返回序列是已完成、需要更多字节还是格式错误。
fn osc_status(buf: &[u8]) -> SeqStatus {
    if buf.get(0) != Some(&0x1b) || buf.get(1) != Some(&b']') {
        return SeqStatus::Malformed;
    }
    let mut i = 2;
    while i < buf.len() {
        match buf[i] {
            0x07 => return SeqStatus::Complete(i + 1),
            0x1b => match buf.get(i + 1) {
                Some(&b'\\') => return SeqStatus::Complete(i + 2),
                // 后跟 `\` 以外内容的裸 ESC —
                // 在我们接受的仅 ST 终止下格式错误。
                Some(_) => return SeqStatus::Malformed,
                // 最尾部的 ESC；下一个数据块可能带来 `\` 并完成序列。
                None => return SeqStatus::NeedMore,
            },
            _ => i += 1,
        }
    }
    SeqStatus::NeedMore
}

fn apc_status(buf: &[u8]) -> SeqStatus {
    if buf.get(0) != Some(&0x1b) || buf.get(1) != Some(&b'_') {
        return SeqStatus::Malformed;
    }
    let mut i = 2;
    while i < buf.len() {
        match buf[i] {
            0x1b => match buf.get(i + 1) {
                Some(&b'\\') => return SeqStatus::Complete(i + 2),
                Some(_) => return SeqStatus::Malformed,
                None => return SeqStatus::NeedMore,
            },
            _ => i += 1,
        }
    }
    SeqStatus::NeedMore
}

fn contains_subslice(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

/// 遍历从 `buf` 头部开始的白名单 CSI 报告。
fn csi_status(buf: &[u8]) -> SeqStatus {
    if buf.get(0) != Some(&0x1b) || buf.get(1) != Some(&b'[') {
        return SeqStatus::Malformed;
    }
    let mut i = 2;
    let max = 256;
    while i < buf.len() && i < max {
        let b = buf[i];
        match b {
            0x30..=0x3F | 0x20..=0x2F => i += 1,
            b't' | b'y' | b'c' | b'n' => return SeqStatus::Complete(i + 1),
            0x40..=0x7E => return SeqStatus::Malformed, // 非白名单终止符
            _ => return SeqStatus::Malformed,
        }
    }
    if i >= max {
        // CSI 超过上限而未终止 — 视为格式错误，以便前导字节回退到残余，
        // 并从下一个位置恢复解析。
        SeqStatus::Malformed
    } else {
        SeqStatus::NeedMore
    }
}

// =====================================================================
// 转发槽超时基础设施
// =====================================================================

use std::sync::{Arc, Mutex, OnceLock};

/// 用于驱动转发槽超时的专用、延迟初始化的运行时。
/// 单个当前线程执行器在其自己的 OS 线程上运行；计时器任务从同步的
/// `ClientInstruction::ForwardQueryToHost` 处理器 `spawn` 到它上面。
/// 单线程模型是因为计时器任务不做 CPU 工作 — 它们只是睡眠并在唤醒时
/// 执行毫秒级的互斥锁检查。
static FORWARD_TIMEOUT_RUNTIME: OnceLock<Arc<tokio::runtime::Runtime>> = OnceLock::new();

pub fn forward_timeout_runtime() -> &'static Arc<tokio::runtime::Runtime> {
    FORWARD_TIMEOUT_RUNTIME.get_or_init(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .expect("failed to build forward-timeout runtime");
        let rt = Arc::new(rt);
        let rt_for_driver = rt.clone();
        // `block_on(pending())` 在此线程上永远保持执行器循环存活；
        // 生成的计时器任务在它们就绪时（生成时、从时间驱动程序唤醒时）被轮询。
        std::thread::Builder::new()
            .name("zellij-client-forward-timeout".into())
            .spawn(move || {
                rt_for_driver.block_on(std::future::pending::<()>());
            })
            .expect("failed to spawn forward-timeout driver thread");
        rt
    })
}

/// 生成一个计时器任务，在 `deadline` 后关闭转发槽，并使用槽累积的任何内容
/// 调用 `on_timeout(token, reply_bytes)`。令牌守卫幂等：如果屏障（或替换转发）
/// 在计时器唤醒时已经清除了槽，`close_forward_on_timeout(token)` 返回 `None`，
/// 且 `on_timeout` 永远不会被调用 — 不需要显式取消路径。
///
/// 提取为自由函数，以便测试可以针对 `tokio::time::pause()` 支持的暂停运行时
/// 驱动它，而无需实例化完整客户端。
pub fn schedule_forward_timeout<F>(
    runtime: &tokio::runtime::Handle,
    parser: Arc<Mutex<StdinAnsiParser>>,
    token: u32,
    deadline: std::time::Duration,
    on_timeout: F,
) where
    F: FnOnce(u32, Vec<u8>) + Send + 'static,
{
    runtime.spawn(async move {
        tokio::time::sleep(deadline).await;
        let payload = parser.lock().unwrap().close_forward_on_timeout(token);
        if let Some((t, bytes)) = payload {
            on_timeout(t, bytes);
        }
    });
}

pub fn schedule_clipboard_forward_timeout<F>(
    runtime: &tokio::runtime::Handle,
    parser: Arc<Mutex<StdinAnsiParser>>,
    token: u32,
    deadline: std::time::Duration,
    on_timeout: F,
) where
    F: FnOnce(u32, Vec<u8>) + Send + 'static,
{
    runtime.spawn(async move {
        tokio::time::sleep(deadline).await;
        let payload = parser
            .lock()
            .unwrap()
            .close_clipboard_forward_on_timeout(token);
        if let Some((t, bytes)) = payload {
            on_timeout(t, bytes);
        }
    });
}

#[cfg(test)]
#[path = "stdin_ansi_parser_tests.rs"]
mod tests;
