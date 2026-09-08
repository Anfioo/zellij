//! 本模块提供 InputParser 结构体，用于帮助解析从终端接收的输入。
use crate::vendored::termwiz::keymap::{Found, KeyMap};
use crate::vendored::termwiz::readbuf::ReadBuffer;
use bitflags::bitflags;
use std::fmt::Write;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Modifiers: u16 {
        const NONE = 0;
        const SHIFT = 1 << 1;
        const ALT = 1 << 2;
        const CTRL = 1 << 3;
        const SUPER = 1 << 4;
        const LEFT_ALT = 1 << 5;
        const RIGHT_ALT = 1 << 6;
        const LEADER = 1 << 7;
        const LEFT_CTRL = 1 << 8;
        const RIGHT_CTRL = 1 << 9;
        const LEFT_SHIFT = 1 << 10;
        const RIGHT_SHIFT = 1 << 11;
        const ENHANCED_KEY = 1 << 12;
    }
}

impl Modifiers {
    pub fn encode_xterm(self) -> u8 {
        let mut number = 0;
        if self.contains(Self::SHIFT) {
            number |= 1;
        }
        if self.contains(Self::ALT) {
            number |= 2;
        }
        if self.contains(Self::CTRL) {
            number |= 4;
        }
        number
    }

    pub fn remove_positional_mods(self) -> Self {
        self - (Self::LEFT_ALT
            | Self::RIGHT_ALT
            | Self::LEFT_CTRL
            | Self::RIGHT_CTRL
            | Self::LEFT_SHIFT
            | Self::RIGHT_SHIFT
            | Self::ENHANCED_KEY)
    }
}

bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub struct KittyKeyboardFlags: u16 {
        const NONE = 0;
        const DISAMBIGUATE_ESCAPE_CODES = 1;
        const REPORT_EVENT_TYPES = 2;
        const REPORT_ALTERNATE_KEYS = 4;
        const REPORT_ALL_KEYS_AS_ESCAPE_CODES = 8;
        const REPORT_ASSOCIATED_TEXT = 16;
    }
}

pub fn ctrl_mapping(c: char) -> Option<char> {
    Some(match c {
        '@' | '`' | ' ' | '2' => '\x00',
        'A' | 'a' => '\x01',
        'B' | 'b' => '\x02',
        'C' | 'c' => '\x03',
        'D' | 'd' => '\x04',
        'E' | 'e' => '\x05',
        'F' | 'f' => '\x06',
        'G' | 'g' => '\x07',
        'H' | 'h' => '\x08',
        'I' | 'i' => '\x09',
        'J' | 'j' => '\x0a',
        'K' | 'k' => '\x0b',
        'L' | 'l' => '\x0c',
        'M' | 'm' => '\x0d',
        'N' | 'n' => '\x0e',
        'O' | 'o' => '\x0f',
        'P' | 'p' => '\x10',
        'Q' | 'q' => '\x11',
        'R' | 'r' => '\x12',
        'S' | 's' => '\x13',
        'T' | 't' => '\x14',
        'U' | 'u' => '\x15',
        'V' | 'v' => '\x16',
        'W' | 'w' => '\x17',
        'X' | 'x' => '\x18',
        'Y' | 'y' => '\x19',
        'Z' | 'z' => '\x1a',
        '[' | '3' | '{' => '\x1b',
        '\\' | '4' | '|' => '\x1c',
        ']' | '5' | '}' => '\x1d',
        '^' | '6' | '~' => '\x1e',
        '_' | '7' | '/' => '\x1f',
        '8' | '?' => '\x7f',
        _ => return None,
    })
}

bitflags! {
    #[derive(Debug, Default, Clone, PartialEq, Eq)]
    pub struct MouseButtons: u8 {
        const NONE = 0;
        const LEFT = 1<<1;
        const RIGHT = 1<<2;
        const MIDDLE = 1<<3;
        const VERT_WHEEL = 1<<4;
        const HORZ_WHEEL = 1<<5;
        /// 如果设置，则滚轮移动方向为正方向，否则为负方向
        const WHEEL_POSITIVE = 1<<6;
    }
}

pub const CSI: &str = "\x1b[";
pub const SS3: &str = "\x1bO";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    PixelMouse(PixelMouseEvent),
    /// 检测到用户已调整终端大小
    Resized {
        cols: usize,
        rows: usize,
    },
    /// 对于支持 Bracketed Paste 模式的终端，粘贴内容会被收集并以此变体报告。
    Paste(String),
    /// 程序已唤醒输入线程。
    Wake,
    /// 接收到操作系统命令序列。包含 \x1b] 与终止符之间的原始载荷。
    OperatingSystemCommand(Vec<u8>),
    /// 由宿主终端发出的基于 CSI 的设备控制/状态报告回复（非键盘事件）。
    /// 此变体仅针对刻意收窄的最终字节白名单产生——`t`（像素尺寸回复）、
    /// `y`（DECRPM 回复）、`c`（Primary-DA 回复）和 `n`（DSR 回复）。
    /// raw 字段包含原始报告的确切字节序列（包括前导 ESC），因此可以
    /// 逐字转发而无需重新序列化。
    DeviceControlReply {
        intermediates: Vec<u8>,
        params: Vec<u8>,
        final_byte: u8,
        raw: Vec<u8>,
    },
    FocusGained,
    FocusLost,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MouseEvent {
    pub x: u16,
    pub y: u16,
    pub mouse_buttons: MouseButtons,
    pub modifiers: Modifiers,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PixelMouseEvent {
    pub x_pixels: u16,
    pub y_pixels: u16,
    pub mouse_buttons: MouseButtons,
    pub modifiers: Modifiers,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyEvent {
    /// 按下了哪个键
    pub key: KeyCode,
    /// 按下了哪些修饰键
    pub modifiers: Modifiers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardEncoding {
    Xterm,
    /// <http://www.leonerd.org.uk/hacks/fixterms/>
    CsiU,
    /// <https://github.com/microsoft/terminal/blob/main/doc/specs/%234999%20-%20Improved%20keyboard%20handling%20in%20Conpty.md>
    Win32,
    /// <https://sw.kovidgoyal.net/kitty/keyboard-protocol/>
    Kitty(KittyKeyboardFlags),
}

/// 指定可能影响 KeyCode 通过 pty 发送给应用程序时编码方式的终端模式/配置。
#[derive(Debug, Clone, Copy)]
pub struct KeyCodeEncodeModes {
    pub encoding: KeyboardEncoding,
    pub application_cursor_keys: bool,
    pub newline_mode: bool,
    pub modify_other_keys: Option<i64>,
}

/// 按下了哪个键。并非所有这些键都可能在大多数系统上出现。
/// 此列表的大部分是 @wez 翻阅文档并为首次遍历中可能出现的情况创建条目。
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum KeyCode {
    /// 解码后的 unicode 字符
    Char(char),

    Hyper,
    Super,
    Meta,

    /// Windows 上的 Ctrl-break
    Cancel,
    Backspace,
    Tab,
    Clear,
    Enter,
    Shift,
    Escape,
    LeftShift,
    RightShift,
    Control,
    LeftControl,
    RightControl,
    Alt,
    LeftAlt,
    RightAlt,
    Menu,
    LeftMenu,
    RightMenu,
    Pause,
    CapsLock,
    PageUp,
    PageDown,
    End,
    Home,
    LeftArrow,
    RightArrow,
    UpArrow,
    DownArrow,
    Select,
    Print,
    Execute,
    PrintScreen,
    Insert,
    Delete,
    Help,
    LeftWindows,
    RightWindows,
    Applications,
    Sleep,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    Multiply,
    Add,
    Separator,
    Subtract,
    Decimal,
    Divide,
    /// F1-F24 均有可能
    Function(u8),
    NumLock,
    ScrollLock,
    Copy,
    Cut,
    Paste,
    BrowserBack,
    BrowserForward,
    BrowserRefresh,
    BrowserStop,
    BrowserSearch,
    BrowserFavorites,
    BrowserHome,
    VolumeMute,
    VolumeDown,
    VolumeUp,
    MediaNextTrack,
    MediaPrevTrack,
    MediaStop,
    MediaPlayPause,
    ApplicationLeftArrow,
    ApplicationRightArrow,
    ApplicationUpArrow,
    ApplicationDownArrow,
    KeyPadHome,
    KeyPadEnd,
    KeyPadPageUp,
    KeyPadPageDown,
    KeyPadBegin,

    #[doc(hidden)]
    InternalPasteStart,
    #[doc(hidden)]
    InternalPasteEnd,
}

impl KeyCode {
    /// 如果按住 SHIFT 且我们有 KeyCode::Char('c')，我们希望将该键码规范化为
    /// KeyCode::Char('C')；这就是此函数的作用。
    pub fn normalize_shift_to_upper_case(self, modifiers: Modifiers) -> KeyCode {
        if modifiers.contains(Modifiers::SHIFT) {
            match self {
                KeyCode::Char(c) if c.is_ascii_lowercase() => KeyCode::Char(c.to_ascii_uppercase()),
                _ => self,
            }
        } else {
            self
        }
    }

    /// 如果该键表示修饰键，返回 true。
    pub fn is_modifier(self) -> bool {
        matches!(
            self,
            Self::Hyper
                | Self::Super
                | Self::Meta
                | Self::Shift
                | Self::LeftShift
                | Self::RightShift
                | Self::Control
                | Self::LeftControl
                | Self::RightControl
                | Self::Alt
                | Self::LeftAlt
                | Self::RightAlt
                | Self::LeftWindows
                | Self::RightWindows
        )
    }

    /// 返回表示此 KeyCode 与 Modifier 组合的字节序列。
    pub fn encode(
        &self,
        mods: Modifiers,
        modes: KeyCodeEncodeModes,
        is_down: bool,
    ) -> Result<String> {
        if !is_down {
            // 我们只想要按下事件
            return Ok(String::new());
        }
        // 我们将键编码为 xterm 兼容序列，该序列不支持位置修饰键。
        let mods = mods.remove_positional_mods();

        use KeyCode::*;

        let key = self.normalize_shift_to_upper_case(mods);
        // 对大写 Char 的修饰键状态进行规范化；移除 SHIFT 修饰键以减少下面的歧义
        let mods = match key {
            Char(c)
                if (c.is_ascii_punctuation() || c.is_ascii_uppercase())
                    && mods.contains(Modifiers::SHIFT) =>
            {
                mods & !Modifiers::SHIFT
            },
            _ => mods,
        };

        // 规范化 Backspace 和 Delete
        let key = match key {
            Char('\x7f') => Delete,
            Char('\x08') => Backspace,
            c => c,
        };

        let mut buf = String::new();

        // TODO: 同时遵循 self.application_keypad

        match key {
            Char(c)
                if is_ambiguous_ascii_ctrl(c)
                    && mods.contains(Modifiers::CTRL)
                    && modes.encoding == KeyboardEncoding::CsiU =>
            {
                csi_u_encode(&mut buf, c, mods, &modes)?;
            },
            Char(c) if c.is_ascii_uppercase() && mods.contains(Modifiers::CTRL) => {
                csi_u_encode(&mut buf, c, mods, &modes)?;
            },

            Char(c) if mods.contains(Modifiers::CTRL) && modes.modify_other_keys == Some(2) => {
                csi_u_encode(&mut buf, c, mods, &modes)?;
            },
            Char(c) if mods.contains(Modifiers::CTRL) && ctrl_mapping(c).is_some() => {
                let c = ctrl_mapping(c).unwrap();
                if mods.contains(Modifiers::ALT) {
                    buf.push(0x1b as char);
                }
                buf.push(c);
            },

            // 按下 alt 时，先发送 escape 以向对端指示按下了 ALT。
            // 我们仅对 ascii 字母数字字符这样做，因为例如：在 macOS 上会生成
            // altgr 风格字形并将 ALT 键保留在修饰键集中。这会使例如 zsh 困惑，
            // 然后仅显示 <fffffffff> 作为输入，因此我们希望避免这种情况。
            Char(c)
                if (c.is_ascii_alphanumeric() || c.is_ascii_punctuation())
                    && mods.contains(Modifiers::ALT) =>
            {
                buf.push(0x1b as char);
                buf.push(c);
            },

            Backspace => {
                // Backspace 发送默认的 VERASE，令人困惑的是它是 DEL ascii 码点
                // 而非 BS。我们仅在按住 CTRL 时发送 BS。
                if mods.contains(Modifiers::CTRL) {
                    csi_u_encode(&mut buf, '\x08', mods, &modes)?;
                } else if mods.contains(Modifiers::SHIFT) {
                    csi_u_encode(&mut buf, '\x7f', mods, &modes)?;
                } else {
                    if mods.contains(Modifiers::ALT) {
                        buf.push(0x1b as char);
                    }
                    buf.push('\x7f');
                }
            },

            Enter | Escape => {
                let c = match key {
                    Enter => '\r',
                    Escape => '\x1b',
                    _ => unreachable!(),
                };
                if mods.contains(Modifiers::SHIFT) || mods.contains(Modifiers::CTRL) {
                    csi_u_encode(&mut buf, c, mods, &modes)?;
                } else {
                    if mods.contains(Modifiers::ALT) {
                        buf.push(0x1b as char);
                    }
                    buf.push(c);
                    if modes.newline_mode && key == Enter {
                        buf.push(0x0a as char);
                    }
                }
            },

            Tab if !mods.is_empty() && modes.modify_other_keys.is_some() => {
                csi_u_encode(&mut buf, '\t', mods, &modes)?;
            },

            Tab => {
                if mods.contains(Modifiers::ALT) {
                    buf.push(0x1b as char);
                }
                let mods = mods & !Modifiers::ALT;
                if mods == Modifiers::CTRL {
                    buf.push_str("\x1b[9;5u");
                } else if mods == Modifiers::CTRL | Modifiers::SHIFT {
                    buf.push_str("\x1b[1;5Z");
                } else if mods == Modifiers::SHIFT {
                    buf.push_str("\x1b[Z");
                } else {
                    buf.push('\t');
                }
            },

            Char(c) => {
                if mods.is_empty() {
                    buf.push(c);
                } else {
                    csi_u_encode(&mut buf, c, mods, &modes)?;
                }
            },

            Home
            | KeyPadHome
            | End
            | KeyPadEnd
            | UpArrow
            | DownArrow
            | RightArrow
            | LeftArrow
            | ApplicationUpArrow
            | ApplicationDownArrow
            | ApplicationRightArrow
            | ApplicationLeftArrow => {
                let (force_app, c) = match key {
                    UpArrow => (false, 'A'),
                    DownArrow => (false, 'B'),
                    RightArrow => (false, 'C'),
                    LeftArrow => (false, 'D'),
                    KeyPadHome | Home => (false, 'H'),
                    End | KeyPadEnd => (false, 'F'),
                    ApplicationUpArrow => (true, 'A'),
                    ApplicationDownArrow => (true, 'B'),
                    ApplicationRightArrow => (true, 'C'),
                    ApplicationLeftArrow => (true, 'D'),
                    _ => unreachable!(),
                };

                let csi_or_ss3 = if force_app || modes.application_cursor_keys {
                    // 在应用程序模式下使用 SS3
                    SS3
                } else {
                    // 否则使用常规 CSI
                    CSI
                };

                if mods.contains(Modifiers::ALT)
                    || mods.contains(Modifiers::SHIFT)
                    || mods.contains(Modifiers::CTRL)
                {
                    write!(buf, "{}1;{}{}", CSI, 1 + mods.encode_xterm(), c)?;
                } else {
                    write!(buf, "{}{}", csi_or_ss3, c)?;
                }
            },

            PageUp | PageDown | KeyPadPageUp | KeyPadPageDown | Insert | Delete => {
                let c = match key {
                    Insert => 2,
                    Delete => 3,
                    KeyPadPageUp | PageUp => 5,
                    KeyPadPageDown | PageDown => 6,
                    _ => unreachable!(),
                };

                if mods.contains(Modifiers::ALT)
                    || mods.contains(Modifiers::SHIFT)
                    || mods.contains(Modifiers::CTRL)
                {
                    write!(buf, "\x1b[{};{}~", c, 1 + mods.encode_xterm())?;
                } else {
                    write!(buf, "\x1b[{}~", c)?;
                }
            },

            Function(n) => {
                if mods.is_empty() && n < 5 {
                    // 如果没有修饰键，F1-F4 使用 SS3 编码
                    write!(
                        buf,
                        "{}",
                        match n {
                            1 => "\x1bOP",
                            2 => "\x1bOQ",
                            3 => "\x1bOR",
                            4 => "\x1bOS",
                            _ => unreachable!("wat?"),
                        }
                    )?;
                } else if n < 5 {
                    // 带修饰键的 F1-F4 的特殊情况
                    let code = match n {
                        1 => 'P',
                        2 => 'Q',
                        3 => 'R',
                        4 => 'S',
                        _ => unreachable!("wat?"),
                    };
                    write!(buf, "\x1b[1;{}{code}", 1 + mods.encode_xterm())?;
                } else {
                    // 编号更高的功能键使用 CSI 而非 SS3。
                    let intro = match n {
                        1 => "\x1b[11",
                        2 => "\x1b[12",
                        3 => "\x1b[13",
                        4 => "\x1b[14",
                        5 => "\x1b[15",
                        6 => "\x1b[17",
                        7 => "\x1b[18",
                        8 => "\x1b[19",
                        9 => "\x1b[20",
                        10 => "\x1b[21",
                        11 => "\x1b[23",
                        12 => "\x1b[24",
                        13 => "\x1b[25",
                        14 => "\x1b[26",
                        15 => "\x1b[28",
                        16 => "\x1b[29",
                        17 => "\x1b[31",
                        18 => "\x1b[32",
                        19 => "\x1b[33",
                        20 => "\x1b[34",
                        21 => "\x1b[42",
                        22 => "\x1b[43",
                        23 => "\x1b[44",
                        24 => "\x1b[45",
                        _ => return Err(format!("unhandled fkey number {}", n).into()),
                    };
                    let encoded_mods = mods.encode_xterm();
                    if encoded_mods == 0 {
                        // 如果没有按住修饰键，不要发送修饰键序列，
                        // 因为修饰键编码是 CSI-u 扩展。
                        write!(buf, "{}~", intro)?;
                    } else {
                        write!(buf, "{};{}~", intro, 1 + encoded_mods)?;
                    }
                }
            },

            Numpad0 | Numpad3 | Numpad9 | Decimal => {
                let intro = match key {
                    Numpad0 => "\x1b[2",
                    Numpad3 => "\x1b[6",
                    Numpad9 => "\x1b[6",
                    Decimal => "\x1b[3",
                    _ => unreachable!(),
                };

                let encoded_mods = mods.encode_xterm();
                if encoded_mods == 0 {
                    write!(buf, "{}~", intro)?;
                } else {
                    write!(buf, "{};{}~", intro, 1 + encoded_mods)?;
                }
            },

            Numpad1 | Numpad2 | Numpad4 | Numpad5 | KeyPadBegin | Numpad6 | Numpad7 | Numpad8 => {
                let c = match key {
                    Numpad1 => "F",
                    Numpad2 => "B",
                    Numpad4 => "D",
                    KeyPadBegin | Numpad5 => "E",
                    Numpad6 => "C",
                    Numpad7 => "H",
                    Numpad8 => "A",
                    _ => unreachable!(),
                };

                let encoded_mods = mods.encode_xterm();
                if encoded_mods == 0 {
                    write!(buf, "{}{}", CSI, c)?;
                } else {
                    write!(buf, "{}1;{}{}", CSI, 1 + encoded_mods, c)?;
                }
            },

            Multiply | Add | Separator | Subtract | Divide => {},

            // 单独按下的修饰键不会扩展为任何内容
            Control | LeftControl | RightControl | Alt | LeftAlt | RightAlt | Menu | LeftMenu
            | RightMenu | Super | Hyper | Shift | LeftShift | RightShift | Meta | LeftWindows
            | RightWindows | NumLock | ScrollLock | Cancel | Clear | Pause | CapsLock | Select
            | Print | PrintScreen | Execute | Help | Applications | Sleep | Copy | Cut | Paste
            | BrowserBack | BrowserForward | BrowserRefresh | BrowserStop | BrowserSearch
            | BrowserFavorites | BrowserHome | VolumeMute | VolumeDown | VolumeUp
            | MediaNextTrack | MediaPrevTrack | MediaStop | MediaPlayPause | InternalPasteStart
            | InternalPasteEnd => {},
        };

        Ok(buf)
    }
}

/// 当被 CTRL 掩码时可能是 ascii 控制字符，也可能是用户合法希望在其
/// 终端应用程序中处理的键的字符
fn is_ambiguous_ascii_ctrl(c: char) -> bool {
    matches!(c, 'i' | 'I' | 'm' | 'M' | '[' | '{' | '@')
}

fn is_ascii(c: char) -> bool {
    (c as u32) < 0x80
}

fn csi_u_encode(
    buf: &mut String,
    c: char,
    mods: Modifiers,
    modes: &KeyCodeEncodeModes,
) -> Result<()> {
    if modes.encoding == KeyboardEncoding::CsiU && is_ascii(c) {
        write!(buf, "\x1b[{};{}u", c as u32, 1 + mods.encode_xterm())?;
        return Ok(());
    }

    // <https://invisible-island.net/xterm/modified-keys.html>
    match (c, modes.modify_other_keys) {
        ('c' | 'd' | '\x1b' | '\x7f' | '\x08', Some(1)) => {
            // 从 modifyOtherKeys 模式 1 中排除知名键
        },
        (c, Some(_)) => {
            write!(buf, "\x1b[27;{};{}~", 1 + mods.encode_xterm(), c as u32)?;
            return Ok(());
        },
        _ => {},
    }

    let c = if mods.contains(Modifiers::CTRL) && ctrl_mapping(c).is_some() {
        ctrl_mapping(c).unwrap()
    } else {
        c
    };
    if mods.contains(Modifiers::ALT) {
        buf.push(0x1b as char);
    }
    write!(buf, "{}", c)?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MouseButton {
    Button1Press,
    Button1Release,
    Button1Drag,
    Button2Press,
    Button2Release,
    Button2Drag,
    Button3Press,
    Button3Release,
    Button3Drag,
    Button4Press,
    Button4Release,
    Button5Press,
    Button5Release,
    Button6Press,
    Button6Release,
    Button7Press,
    Button7Release,
    None,
}

fn decode_mouse_button(control: u8, p0: i64) -> Option<MouseButton> {
    match (control, p0 & 0b110_0011) {
        (b'M', 0) => Some(MouseButton::Button1Press),
        (b'm', 0) => Some(MouseButton::Button1Release),
        (b'M', 1) => Some(MouseButton::Button2Press),
        (b'm', 1) => Some(MouseButton::Button2Release),
        (b'M', 2) => Some(MouseButton::Button3Press),
        (b'm', 2) => Some(MouseButton::Button3Release),
        (b'M', 64) => Some(MouseButton::Button4Press),
        (b'm', 64) => Some(MouseButton::Button4Release),
        (b'M', 65) => Some(MouseButton::Button5Press),
        (b'm', 65) => Some(MouseButton::Button5Release),
        (b'M', 66) => Some(MouseButton::Button6Press),
        (b'm', 66) => Some(MouseButton::Button6Release),
        (b'M', 67) => Some(MouseButton::Button7Press),
        (b'm', 67) => Some(MouseButton::Button7Release),
        (b'M', 32) => Some(MouseButton::Button1Drag),
        (b'M', 33) => Some(MouseButton::Button2Drag),
        (b'M', 34) => Some(MouseButton::Button3Drag),
        (b'M', 35) | (b'm', 35) | (b'M', 3) | (b'm', 3) => Some(MouseButton::None),
        _ => ::core::option::Option::None,
    }
}

impl From<MouseButton> for MouseButtons {
    fn from(button: MouseButton) -> MouseButtons {
        match button {
            MouseButton::Button1Press | MouseButton::Button1Drag => MouseButtons::LEFT,
            MouseButton::Button2Press | MouseButton::Button2Drag => MouseButtons::MIDDLE,
            MouseButton::Button3Press | MouseButton::Button3Drag => MouseButtons::RIGHT,
            MouseButton::Button4Press => MouseButtons::VERT_WHEEL | MouseButtons::WHEEL_POSITIVE,
            MouseButton::Button5Press => MouseButtons::VERT_WHEEL,
            MouseButton::Button6Press => MouseButtons::HORZ_WHEEL | MouseButtons::WHEEL_POSITIVE,
            MouseButton::Button7Press => MouseButtons::HORZ_WHEEL,
            _ => MouseButtons::NONE,
        }
    }
}

fn decode_mouse_modifiers(p0: i64) -> Modifiers {
    let mut modifiers = Modifiers::NONE;
    if p0 & 4 != 0 {
        modifiers |= Modifiers::SHIFT;
    }
    if p0 & 8 != 0 {
        modifiers |= Modifiers::ALT;
    }
    if p0 & 16 != 0 {
        modifiers |= Modifiers::CTRL;
    }
    modifiers
}

/// 尝试从缓冲区解析 SGR 鼠标序列。
/// 成功时返回 Some((InputEvent, bytes_consumed))。
/// 如果缓冲区不包含完整的 SGR 鼠标序列，返回 None。
fn parse_sgr_mouse(buf: &[u8]) -> Option<(InputEvent, usize)> {
    // 必须以 \x1b[< 开头
    if buf.len() < 6 || !buf.starts_with(b"\x1b[<") {
        return None;
    }
    let rest = &buf[3..]; // 跳过 \x1b[<

    // 查找终止符 M 或 m
    let term_pos = rest.iter().position(|&b| b == b'M' || b == b'm')?;
    let control = rest[term_pos];
    let params_str = std::str::from_utf8(&rest[..term_pos]).ok()?;

    // 解析三个以分号分隔的整数
    let mut parts = params_str.splitn(3, ';');
    let p0: i64 = parts.next()?.parse().ok()?;
    let p1: i64 = parts.next()?.parse().ok()?;
    let p2: i64 = parts.next()?.parse().ok()?;

    let button = decode_mouse_button(control, p0)?;
    let modifiers = decode_mouse_modifiers(p0);
    let mouse_buttons: MouseButtons = button.into();

    let consumed = 3 + term_pos + 1; // \x1b[< + 参数 + M/m

    Some((
        InputEvent::Mouse(MouseEvent {
            x: p1 as u16,
            y: p2 as u16,
            mouse_buttons,
            modifiers,
        }),
        consumed,
    ))
}

/// 尝试从缓冲区解析 OSC（操作系统命令）序列。
/// 如果找到完整的 OSC 序列，返回 `Some((InputEvent::OperatingSystemCommand(payload), len))`，
/// 其中 `payload` 是 `\x1b]` 与终止符之间的字节，`len` 是消耗的总字节数。
/// 如果缓冲区不以 `\x1b]` 开头或序列不完整，返回 `None`。
/// 尝试从 `buf` 开头解析基于 CSI 的宿主终端报告（设备属性响应、
/// DSR 回复、DECRPM、像素尺寸回复等）。
///
/// 仅识别收窄的最终字节白名单：`t`、`y`、`c`、`n`。任何其他最终字节
/// 返回 `None`，以便字节传递给常规 CSI 键映射机制。
///
/// 完全匹配时返回 `Some((event, len))`，如果字节看起来不像白名单 CSI 报告
/// （调用者应尝试下一个解析器）返回 `None`，并保留对以 `ESC [` 开头的缓冲区
/// 返回 None 的两种不同情况——此处目前不做区分：
/// - 真正格式错误/不支持的序列，或
/// - 输入不完整；调用者通过 `maybe_more` 处理不完整性。
/// 如果 `buf` 以结构上完整的 CSI 序列开头（根据 ECMA-48 §5.4，
/// `\x1b[ <params>* <intermediates>* <final>`），返回 `Some(len)`，
/// 无论最终字节是否是我们有用的。`process_bytes` 使用它来跳过键映射不识别的
/// CSI 序列——最重要的是 Kitty 键盘协议事件 `\x1b[<keycode>;<mods>u`。
/// 如果没有这个，键映射会返回 `Found::NeedData`（它将字节视为更长的已注册键
/// 的可能前缀），解析器会卡住，持有永远不会扩展为任何内容的字节。
///
/// 如果缓冲区不以 `\x1b[` 开头、包含非 CSI 字节，或尚未接收到最终字节，
/// 返回 `None`。
fn complete_csi_len(buf: &[u8]) -> Option<usize> {
    if buf.get(0) != Some(&0x1b) || buf.get(1) != Some(&b'[') {
        return None;
    }
    let mut i = 2;
    let max_scan = buf.len().min(256);
    while i < max_scan {
        let b = buf[i];
        match b {
            // 参数（数字、;、:、?、<、=、>）和中间字节（space..'/'）。
            0x30..=0x3F | 0x20..=0x2F => i += 1,
            // 最终字节范围内的任何字节都会终止 CSI 序列。
            0x40..=0x7E => return Some(i + 1),
            // 任何其他内容意味着这不是格式良好的 CSI。
            _ => return None,
        }
    }
    None
}

fn parse_csi_report(buf: &[u8]) -> Option<(InputEvent, usize)> {
    if buf.get(0) != Some(&0x1b) || buf.get(1) != Some(&b'[') {
        return None;
    }
    // 向前扫描查找白名单中的最终字节，如果遇到明显不是 CSI 报告的内容
    // （已知最终字节以外的不可打印字节）则放弃。
    let mut i = 2;
    let mut intermediates: Vec<u8> = Vec::new();
    let mut params: Vec<u8> = Vec::new();
    // 参数（0x30..=0x3F）在前，然后是中间字节（0x20..=0x2F），
    // 然后是最终字节（0x40..=0x7E）。我们仅扫描到合理长度以避免病态缓冲区。
    let max_scan = buf.len().min(256);
    while i < max_scan {
        let b = buf[i];
        match b {
            // 参数：数字、`;`、`:`、`?`、`<`、`=`、`>`
            0x30..=0x3F => {
                params.push(b);
                i += 1;
            },
            // 中间字节：空格、`!`、`"`、... `/`
            0x20..=0x2F => {
                intermediates.push(b);
                i += 1;
            },
            // 最终字节（0x40..=0x7E）：必须是白名单字节之一。
            b't' | b'y' | b'c' | b'n' => {
                let raw = buf[0..=i].to_vec();
                return Some((
                    InputEvent::DeviceControlReply {
                        intermediates,
                        params,
                        final_byte: b,
                        raw,
                    },
                    i + 1,
                ));
            },
            0x40..=0x7E => {
                // 最终字节在白名单之外——不是我们的。
                return None;
            },
            _ => {
                // CSI 内部出现意外内容——放弃。
                return None;
            },
        }
    }
    None
}

fn parse_osc(buf: &[u8]) -> Option<(InputEvent, usize)> {
    // OSC 序列以 ESC ]（0x1b 0x5d）开头
    if buf.get(0) != Some(&0x1b) || buf.get(1) != Some(&b']') {
        return None;
    }
    let mut i = 2;
    while i < buf.len() {
        match buf.get(i) {
            Some(&0x07) => {
                // BEL 终止符
                let payload = buf.get(2..i).unwrap_or_default().to_vec();
                return Some((InputEvent::OperatingSystemCommand(payload), i + 1));
            },
            Some(&0x1b) => {
                // 可能的 ST 终止符（ESC \）
                if buf.get(i + 1) == Some(&b'\\') {
                    let payload = buf.get(2..i).unwrap_or_default().to_vec();
                    return Some((InputEvent::OperatingSystemCommand(payload), i + 2));
                }
                // OSC 内部的裸 ESC——格式错误，但不继续消耗
                return None;
            },
            Some(_) => {
                i += 1;
            },
            None => {
                // 由于 i < buf.len()，不应发生，但优雅处理
                return None;
            },
        }
    }
    None // 不完整——尚未找到终止符
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputState {
    Normal,
    EscapeMaybeAlt,
    Pasting(usize),
}

#[derive(Debug)]
pub struct InputParser {
    key_map: KeyMap<InputEvent>,
    buf: ReadBuffer,
    state: InputState,
}

#[cfg(windows)]
mod windows {
    use super::*;
    use std;
    use winapi::um::wincon::{
        INPUT_RECORD, KEY_EVENT, KEY_EVENT_RECORD, MOUSE_EVENT, MOUSE_EVENT_RECORD,
        WINDOW_BUFFER_SIZE_EVENT, WINDOW_BUFFER_SIZE_RECORD,
    };
    use winapi::um::winuser;

    fn modifiers_from_ctrl_key_state(state: u32) -> Modifiers {
        use winapi::um::wincon::*;

        let mut mods = Modifiers::NONE;

        if (state & (LEFT_ALT_PRESSED | RIGHT_ALT_PRESSED)) != 0 {
            mods |= Modifiers::ALT;
        }

        if (state & (LEFT_CTRL_PRESSED | RIGHT_CTRL_PRESSED)) != 0 {
            mods |= Modifiers::CTRL;
        }

        if (state & SHIFT_PRESSED) != 0 {
            mods |= Modifiers::SHIFT;
        }

        mods
    }

    impl InputParser {
        fn decode_key_record<F: FnMut(InputEvent)>(
            &mut self,
            event: &KEY_EVENT_RECORD,
            callback: &mut F,
        ) {
            if event.bKeyDown == 0 {
                return;
            }

            let key_code = match std::char::from_u32(*unsafe { event.uChar.UnicodeChar() } as u32) {
                Some(unicode) if unicode > '\x00' => {
                    let mut buf = [0u8; 4];
                    self.buf
                        .extend_with(unicode.encode_utf8(&mut buf).as_bytes());
                    self.process_bytes(|e, _consumed| callback(e), true);
                    return;
                },
                _ => match event.wVirtualKeyCode as i32 {
                    winuser::VK_CANCEL => KeyCode::Cancel,
                    winuser::VK_BACK => KeyCode::Backspace,
                    winuser::VK_TAB => KeyCode::Tab,
                    winuser::VK_CLEAR => KeyCode::Clear,
                    winuser::VK_RETURN => KeyCode::Enter,
                    winuser::VK_SHIFT => KeyCode::Shift,
                    winuser::VK_CONTROL => KeyCode::Control,
                    winuser::VK_MENU => KeyCode::Menu,
                    winuser::VK_PAUSE => KeyCode::Pause,
                    winuser::VK_CAPITAL => KeyCode::CapsLock,
                    winuser::VK_ESCAPE => KeyCode::Escape,
                    winuser::VK_PRIOR => KeyCode::PageUp,
                    winuser::VK_NEXT => KeyCode::PageDown,
                    winuser::VK_END => KeyCode::End,
                    winuser::VK_HOME => KeyCode::Home,
                    winuser::VK_LEFT => KeyCode::LeftArrow,
                    winuser::VK_RIGHT => KeyCode::RightArrow,
                    winuser::VK_UP => KeyCode::UpArrow,
                    winuser::VK_DOWN => KeyCode::DownArrow,
                    winuser::VK_SELECT => KeyCode::Select,
                    winuser::VK_PRINT => KeyCode::Print,
                    winuser::VK_EXECUTE => KeyCode::Execute,
                    winuser::VK_SNAPSHOT => KeyCode::PrintScreen,
                    winuser::VK_INSERT => KeyCode::Insert,
                    winuser::VK_DELETE => KeyCode::Delete,
                    winuser::VK_HELP => KeyCode::Help,
                    winuser::VK_LWIN => KeyCode::LeftWindows,
                    winuser::VK_RWIN => KeyCode::RightWindows,
                    winuser::VK_APPS => KeyCode::Applications,
                    winuser::VK_SLEEP => KeyCode::Sleep,
                    winuser::VK_NUMPAD0 => KeyCode::Numpad0,
                    winuser::VK_NUMPAD1 => KeyCode::Numpad1,
                    winuser::VK_NUMPAD2 => KeyCode::Numpad2,
                    winuser::VK_NUMPAD3 => KeyCode::Numpad3,
                    winuser::VK_NUMPAD4 => KeyCode::Numpad4,
                    winuser::VK_NUMPAD5 => KeyCode::Numpad5,
                    winuser::VK_NUMPAD6 => KeyCode::Numpad6,
                    winuser::VK_NUMPAD7 => KeyCode::Numpad7,
                    winuser::VK_NUMPAD8 => KeyCode::Numpad8,
                    winuser::VK_NUMPAD9 => KeyCode::Numpad9,
                    winuser::VK_MULTIPLY => KeyCode::Multiply,
                    winuser::VK_ADD => KeyCode::Add,
                    winuser::VK_SEPARATOR => KeyCode::Separator,
                    winuser::VK_SUBTRACT => KeyCode::Subtract,
                    winuser::VK_DECIMAL => KeyCode::Decimal,
                    winuser::VK_DIVIDE => KeyCode::Divide,
                    winuser::VK_F1 => KeyCode::Function(1),
                    winuser::VK_F2 => KeyCode::Function(2),
                    winuser::VK_F3 => KeyCode::Function(3),
                    winuser::VK_F4 => KeyCode::Function(4),
                    winuser::VK_F5 => KeyCode::Function(5),
                    winuser::VK_F6 => KeyCode::Function(6),
                    winuser::VK_F7 => KeyCode::Function(7),
                    winuser::VK_F8 => KeyCode::Function(8),
                    winuser::VK_F9 => KeyCode::Function(9),
                    winuser::VK_F10 => KeyCode::Function(10),
                    winuser::VK_F11 => KeyCode::Function(11),
                    winuser::VK_F12 => KeyCode::Function(12),
                    winuser::VK_F13 => KeyCode::Function(13),
                    winuser::VK_F14 => KeyCode::Function(14),
                    winuser::VK_F15 => KeyCode::Function(15),
                    winuser::VK_F16 => KeyCode::Function(16),
                    winuser::VK_F17 => KeyCode::Function(17),
                    winuser::VK_F18 => KeyCode::Function(18),
                    winuser::VK_F19 => KeyCode::Function(19),
                    winuser::VK_F20 => KeyCode::Function(20),
                    winuser::VK_F21 => KeyCode::Function(21),
                    winuser::VK_F22 => KeyCode::Function(22),
                    winuser::VK_F23 => KeyCode::Function(23),
                    winuser::VK_F24 => KeyCode::Function(24),
                    winuser::VK_NUMLOCK => KeyCode::NumLock,
                    winuser::VK_SCROLL => KeyCode::ScrollLock,
                    winuser::VK_LSHIFT => KeyCode::LeftShift,
                    winuser::VK_RSHIFT => KeyCode::RightShift,
                    winuser::VK_LCONTROL => KeyCode::LeftControl,
                    winuser::VK_RCONTROL => KeyCode::RightControl,
                    winuser::VK_LMENU => KeyCode::LeftMenu,
                    winuser::VK_RMENU => KeyCode::RightMenu,
                    winuser::VK_BROWSER_BACK => KeyCode::BrowserBack,
                    winuser::VK_BROWSER_FORWARD => KeyCode::BrowserForward,
                    winuser::VK_BROWSER_REFRESH => KeyCode::BrowserRefresh,
                    winuser::VK_BROWSER_STOP => KeyCode::BrowserStop,
                    winuser::VK_BROWSER_SEARCH => KeyCode::BrowserSearch,
                    winuser::VK_BROWSER_FAVORITES => KeyCode::BrowserFavorites,
                    winuser::VK_BROWSER_HOME => KeyCode::BrowserHome,
                    winuser::VK_VOLUME_MUTE => KeyCode::VolumeMute,
                    winuser::VK_VOLUME_DOWN => KeyCode::VolumeDown,
                    winuser::VK_VOLUME_UP => KeyCode::VolumeUp,
                    winuser::VK_MEDIA_NEXT_TRACK => KeyCode::MediaNextTrack,
                    winuser::VK_MEDIA_PREV_TRACK => KeyCode::MediaPrevTrack,
                    winuser::VK_MEDIA_STOP => KeyCode::MediaStop,
                    winuser::VK_MEDIA_PLAY_PAUSE => KeyCode::MediaPlayPause,
                    _ => return,
                },
            };
            let mut modifiers = modifiers_from_ctrl_key_state(event.dwControlKeyState);

            let key_code = key_code.normalize_shift_to_upper_case(modifiers);
            if let KeyCode::Char(c) = key_code {
                if c.is_ascii_uppercase() {
                    modifiers.remove(Modifiers::SHIFT);
                }
            }

            let input_event = InputEvent::Key(KeyEvent {
                key: key_code,
                modifiers,
            });
            for _ in 0..event.wRepeatCount {
                callback(input_event.clone());
            }
        }

        fn decode_mouse_record<F: FnMut(InputEvent)>(
            &self,
            event: &MOUSE_EVENT_RECORD,
            callback: &mut F,
        ) {
            use winapi::um::wincon::*;
            let mut buttons = MouseButtons::NONE;

            if (event.dwButtonState & FROM_LEFT_1ST_BUTTON_PRESSED) != 0 {
                buttons |= MouseButtons::LEFT;
            }
            if (event.dwButtonState & RIGHTMOST_BUTTON_PRESSED) != 0 {
                buttons |= MouseButtons::RIGHT;
            }
            if (event.dwButtonState & FROM_LEFT_2ND_BUTTON_PRESSED) != 0 {
                buttons |= MouseButtons::MIDDLE;
            }

            let modifiers = modifiers_from_ctrl_key_state(event.dwControlKeyState);

            if (event.dwEventFlags & MOUSE_WHEELED) != 0 {
                buttons |= MouseButtons::VERT_WHEEL;
                if (event.dwButtonState >> 8) != 0 {
                    buttons |= MouseButtons::WHEEL_POSITIVE;
                }
            } else if (event.dwEventFlags & MOUSE_HWHEELED) != 0 {
                buttons |= MouseButtons::HORZ_WHEEL;
                if (event.dwButtonState >> 8) != 0 {
                    buttons |= MouseButtons::WHEEL_POSITIVE;
                }
            }

            let mouse = InputEvent::Mouse(MouseEvent {
                x: event.dwMousePosition.X as u16,
                y: event.dwMousePosition.Y as u16,
                mouse_buttons: buttons,
                modifiers,
            });

            if (event.dwEventFlags & DOUBLE_CLICK) != 0 {
                callback(mouse.clone());
            }
            callback(mouse);
        }

        fn decode_resize_record<F: FnMut(InputEvent)>(
            &self,
            event: &WINDOW_BUFFER_SIZE_RECORD,
            callback: &mut F,
        ) {
            callback(InputEvent::Resized {
                rows: event.dwSize.Y as usize,
                cols: event.dwSize.X as usize,
            });
        }

        pub fn decode_input_records<F: FnMut(InputEvent)>(
            &mut self,
            records: &[INPUT_RECORD],
            callback: &mut F,
        ) {
            for record in records {
                match record.EventType {
                    KEY_EVENT => {
                        self.decode_key_record(unsafe { record.Event.KeyEvent() }, callback)
                    },
                    MOUSE_EVENT => {
                        self.decode_mouse_record(unsafe { record.Event.MouseEvent() }, callback)
                    },
                    WINDOW_BUFFER_SIZE_EVENT => self.decode_resize_record(
                        unsafe { record.Event.WindowBufferSizeEvent() },
                        callback,
                    ),
                    _ => {},
                }
            }
            self.process_bytes(|e, _consumed| callback(e), false);
        }
    }
}

impl Default for InputParser {
    fn default() -> Self {
        Self::new()
    }
}

impl InputParser {
    pub fn new() -> Self {
        Self {
            key_map: Self::build_basic_key_map(),
            buf: ReadBuffer::new(),
            state: InputState::Normal,
        }
    }

    fn build_basic_key_map() -> KeyMap<InputEvent> {
        let mut map = KeyMap::new();

        let modifier_combos = &[
            ("", Modifiers::NONE),
            (";1", Modifiers::NONE),
            (";2", Modifiers::SHIFT),
            (";3", Modifiers::ALT),
            (";4", Modifiers::ALT | Modifiers::SHIFT),
            (";5", Modifiers::CTRL),
            (";6", Modifiers::CTRL | Modifiers::SHIFT),
            (";7", Modifiers::CTRL | Modifiers::ALT),
            (";8", Modifiers::CTRL | Modifiers::ALT | Modifiers::SHIFT),
        ];
        let meta = Modifiers::ALT;
        let meta_modifier_combos = &[
            (";9", meta),
            (";10", meta | Modifiers::SHIFT),
            (";11", meta | Modifiers::ALT),
            (";12", meta | Modifiers::ALT | Modifiers::SHIFT),
            (";13", meta | Modifiers::CTRL),
            (";14", meta | Modifiers::CTRL | Modifiers::SHIFT),
            (";15", meta | Modifiers::CTRL | Modifiers::ALT),
            (
                ";16",
                meta | Modifiers::CTRL | Modifiers::ALT | Modifiers::SHIFT,
            ),
        ];

        let modifier_combos_including_meta =
            || modifier_combos.iter().chain(meta_modifier_combos.iter());

        for alpha in b'A'..=b'Z' {
            // Ctrl-[A..=Z] 以 1..=26 发送
            let ctrl = [alpha & 0x1f];
            map.insert(
                &ctrl,
                InputEvent::Key(KeyEvent {
                    key: KeyCode::Char((alpha as char).to_ascii_lowercase()),
                    modifiers: Modifiers::CTRL,
                }),
            );

            // ALT A-Z 通常以前导 ESC 发送
            let alt = [0x1b, alpha];
            map.insert(
                &alt,
                InputEvent::Key(KeyEvent {
                    key: KeyCode::Char(alpha as char),
                    modifiers: Modifiers::ALT,
                }),
            );
        }

        for c in 0..=0x7fu8 {
            for (suffix, modifiers) in modifier_combos {
                // ascii 范围的 `CSI u` 编码；
                // 参见 http://www.leonerd.org.uk/hacks/fixterms/
                let key = format!("\x1b[{}{}u", c, suffix);
                map.insert(
                    key,
                    InputEvent::Key(KeyEvent {
                        key: KeyCode::Char(c as char),
                        modifiers: *modifiers,
                    }),
                );

                if !suffix.is_empty() {
                    // xterm modifyOtherKeys 序列
                    let key = format!("\x1b[27{};{}~", suffix, c);
                    map.insert(
                        key,
                        InputEvent::Key(KeyEvent {
                            key: match c {
                                8 | 0x7f => KeyCode::Backspace,
                                0x1b => KeyCode::Escape,
                                9 => KeyCode::Tab,
                                10 | 13 => KeyCode::Enter,
                                _ => KeyCode::Char(c as char),
                            },
                            modifiers: *modifiers,
                        }),
                    );
                }
            }
        }

        // 常见方向键
        for (keycode, dir) in &[
            (KeyCode::UpArrow, b'A'),
            (KeyCode::DownArrow, b'B'),
            (KeyCode::RightArrow, b'C'),
            (KeyCode::LeftArrow, b'D'),
            (KeyCode::Home, b'H'),
            (KeyCode::End, b'F'),
        ] {
            // 正常模式下的方向键使用 CSI 编码
            let arrow = [0x1b, b'[', *dir];
            map.insert(
                &arrow,
                InputEvent::Key(KeyEvent {
                    key: *keycode,
                    modifiers: Modifiers::NONE,
                }),
            );
            for (suffix, modifiers) in modifier_combos_including_meta() {
                let key = format!("\x1b[1{}{}", suffix, *dir as char);
                map.insert(
                    key,
                    InputEvent::Key(KeyEvent {
                        key: *keycode,
                        modifiers: *modifiers,
                    }),
                );
            }
        }
        for &(keycode, dir) in &[
            (KeyCode::UpArrow, b'a'),
            (KeyCode::DownArrow, b'b'),
            (KeyCode::RightArrow, b'c'),
            (KeyCode::LeftArrow, b'd'),
        ] {
            // rxvt 特有的带修饰键方向键。
            for &(seq, mods) in &[
                ([0x1b, b'[', dir], Modifiers::SHIFT),
                ([0x1b, b'O', dir], Modifiers::CTRL),
            ] {
                map.insert(
                    &seq,
                    InputEvent::Key(KeyEvent {
                        key: keycode,
                        modifiers: mods,
                    }),
                );
            }
        }

        for (keycode, dir) in &[
            (KeyCode::ApplicationUpArrow, b'A'),
            (KeyCode::ApplicationDownArrow, b'B'),
            (KeyCode::ApplicationRightArrow, b'C'),
            (KeyCode::ApplicationLeftArrow, b'D'),
        ] {
            // 应用程序光标模式下的方向键使用 SS3 编码
            let app = [0x1b, b'O', *dir];
            map.insert(
                &app,
                InputEvent::Key(KeyEvent {
                    key: *keycode,
                    modifiers: Modifiers::NONE,
                }),
            );
            for (suffix, modifiers) in modifier_combos {
                let key = format!("\x1bO1{}{}", suffix, *dir as char);
                map.insert(
                    key,
                    InputEvent::Key(KeyEvent {
                        key: *keycode,
                        modifiers: *modifiers,
                    }),
                );
            }
        }

        // 无修饰键的功能键 1-4 使用 SS3 编码
        for (keycode, c) in &[
            (KeyCode::Function(1), b'P'),
            (KeyCode::Function(2), b'Q'),
            (KeyCode::Function(3), b'R'),
            (KeyCode::Function(4), b'S'),
        ] {
            let key = [0x1b, b'O', *c];
            map.insert(
                &key,
                InputEvent::Key(KeyEvent {
                    key: *keycode,
                    modifiers: Modifiers::NONE,
                }),
            );
        }

        // 带修饰键的功能键 1-4
        for (keycode, c) in &[
            (KeyCode::Function(1), b'P'),
            (KeyCode::Function(2), b'Q'),
            (KeyCode::Function(3), b'R'),
            (KeyCode::Function(4), b'S'),
        ] {
            for (suffix, modifiers) in modifier_combos_including_meta() {
                let key = format!("\x1b[1{suffix}{code}", code = *c as char, suffix = suffix);
                map.insert(
                    key,
                    InputEvent::Key(KeyEvent {
                        key: *keycode,
                        modifiers: *modifiers,
                    }),
                );
            }
        }

        // 带修饰键的功能键使用 CSI 编码。
        // http://aperiodic.net/phil/archives/Geekery/term-function-keys.html
        for (range, offset) in &[
            // F1-F5 编码为 11-15
            (1..=5, 10),
            // F6-F10 编码为 17-21
            (6..=10, 11),
            // F11-F14 编码为 23-26
            (11..=14, 12),
            // F15-F16 编码为 28-29
            (15..=16, 13),
            // F17-F20 编码为 31-34
            (17..=20, 14),
        ] {
            for n in range.clone() {
                for (suffix, modifiers) in modifier_combos_including_meta() {
                    let key = format!("\x1b[{code}{suffix}~", code = n + offset, suffix = suffix);
                    map.insert(
                        key,
                        InputEvent::Key(KeyEvent {
                            key: KeyCode::Function(n),
                            modifiers: *modifiers,
                        }),
                    );
                }
            }
        }

        for (keycode, c) in &[
            (KeyCode::Insert, b'2'),
            (KeyCode::Delete, b'3'),
            (KeyCode::Home, b'1'),
            (KeyCode::End, b'4'),
            (KeyCode::PageUp, b'5'),
            (KeyCode::PageDown, b'6'),
            // rxvt
            (KeyCode::Home, b'7'),
            (KeyCode::End, b'8'),
        ] {
            for (suffix, modifiers) in &[
                (b'~', Modifiers::NONE),
                (b'$', Modifiers::SHIFT),
                (b'^', Modifiers::CTRL),
                (b'@', Modifiers::SHIFT | Modifiers::CTRL),
            ] {
                let key = [0x1b, b'[', *c, *suffix];
                map.insert(
                    key,
                    InputEvent::Key(KeyEvent {
                        key: *keycode,
                        modifiers: *modifiers,
                    }),
                );
            }
        }

        map.insert(
            &[0x7f],
            InputEvent::Key(KeyEvent {
                key: KeyCode::Backspace,
                modifiers: Modifiers::NONE,
            }),
        );

        map.insert(
            &[0x8],
            InputEvent::Key(KeyEvent {
                key: KeyCode::Backspace,
                modifiers: Modifiers::NONE,
            }),
        );

        map.insert(
            &[0x1b],
            InputEvent::Key(KeyEvent {
                key: KeyCode::Escape,
                modifiers: Modifiers::NONE,
            }),
        );

        map.insert(
            &[b'\t'],
            InputEvent::Key(KeyEvent {
                key: KeyCode::Tab,
                modifiers: Modifiers::NONE,
            }),
        );
        map.insert(
            b"\x1b[Z",
            InputEvent::Key(KeyEvent {
                key: KeyCode::Tab,
                modifiers: Modifiers::SHIFT,
            }),
        );

        map.insert(
            &[b'\r'],
            InputEvent::Key(KeyEvent {
                key: KeyCode::Enter,
                modifiers: Modifiers::NONE,
            }),
        );
        map.insert(
            &[b'\n'],
            InputEvent::Key(KeyEvent {
                key: KeyCode::Enter,
                modifiers: Modifiers::NONE,
            }),
        );

        map.insert(
            b"\x1b[200~",
            InputEvent::Key(KeyEvent {
                key: KeyCode::InternalPasteStart,
                modifiers: Modifiers::NONE,
            }),
        );
        map.insert(
            b"\x1b[201~",
            InputEvent::Key(KeyEvent {
                key: KeyCode::InternalPasteEnd,
                modifiers: Modifiers::NONE,
            }),
        );
        map.insert(b"\x1b[I", InputEvent::FocusGained);
        map.insert(b"\x1b[O", InputEvent::FocusLost);

        map.insert(
            b"\x1b[",
            InputEvent::Key(KeyEvent {
                key: KeyCode::Char('['),
                modifiers: Modifiers::ALT,
            }),
        );

        map
    }

    /// 返回 str 中的第一个字符以及该字符的长度（以 *字节* 为单位）。
    fn first_char_and_len(s: &str) -> (char, usize) {
        let mut iter = s.chars();
        let c = iter.next().unwrap();
        (c, c.len_utf8())
    }

    /// 这是一个糟糕的函数，用于从字节序列中提取第一个 unicode 字符，
    /// 并返回它和剩余的切片。
    fn decode_one_char(bytes: &[u8]) -> Option<(char, usize)> {
        let bytes = &bytes[..bytes.len().min(4)];
        match std::str::from_utf8(bytes) {
            Ok(s) => {
                let (c, len) = Self::first_char_and_len(s);
                Some((c, len))
            },
            Err(err) => {
                let (valid, _after_valid) = bytes.split_at(err.valid_up_to());
                if !valid.is_empty() {
                    let s = unsafe { std::str::from_utf8_unchecked(valid) };
                    let (c, len) = Self::first_char_and_len(s);
                    Some((c, len))
                } else {
                    None
                }
            },
        }
    }

    fn dispatch_callback<F: FnMut(InputEvent, usize)>(
        &mut self,
        mut callback: F,
        event: InputEvent,
    ) {
        // `self.buf` 已经前进过此事件，因此 `self.buf.len()` 是
        // `parse_with_consumed` 计算每个事件字节数时的剩余量。
        match (self.state, &event) {
            (
                InputState::Normal,
                InputEvent::Key(KeyEvent {
                    key: KeyCode::InternalPasteStart,
                    ..
                }),
            ) => {
                self.state = InputState::Pasting(0);
            },
            (
                InputState::EscapeMaybeAlt,
                InputEvent::Key(KeyEvent {
                    key: KeyCode::InternalPasteStart,
                    ..
                }),
            ) => {
                // 之前的 ESC 不是 ALT 序列的一部分，因此在开始收集粘贴内容之前先发出它。
                callback(
                    InputEvent::Key(KeyEvent {
                        key: KeyCode::Escape,
                        modifiers: Modifiers::NONE,
                    }),
                    self.buf.len(),
                );
                self.state = InputState::Pasting(0);
            },
            (InputState::EscapeMaybeAlt, InputEvent::Key(KeyEvent { key, modifiers })) => {
                // 将此视为 ALT 键
                let key = *key;
                let modifiers = *modifiers;
                self.state = InputState::Normal;
                callback(
                    InputEvent::Key(KeyEvent {
                        key,
                        modifiers: modifiers | Modifiers::ALT,
                    }),
                    self.buf.len(),
                );
            },
            (InputState::EscapeMaybeAlt, _) => {
                // 之前的 ESC 不是 ALT 序列的一部分，因此同时发出它和当前事件
                callback(
                    InputEvent::Key(KeyEvent {
                        key: KeyCode::Escape,
                        modifiers: Modifiers::NONE,
                    }),
                    self.buf.len(),
                );
                callback(event, self.buf.len());
            },
            (_, _) => callback(event, self.buf.len()),
        }
    }

    /// 如果停放的 ESC 当前保存在 `EscapeMaybeAlt` 中，将其作为真正的
    /// `Esc` 击键发出并返回 `Normal`。在分派即将到来的字节匹配的任何结构化序列
    /// （SGR 鼠标、OSC、白名单 CSI 宿主回复）之前，从 `process_bytes` 调用——
    /// 这些序列是自主的宿主事件，不能与停放的 ESC 进行 ALT 组合，因此必须在
    /// 发出序列之前刷新 ESC。
    fn flush_parked_esc_if_held<F: FnMut(InputEvent, usize)>(&mut self, callback: &mut F) {
        if self.state == InputState::EscapeMaybeAlt {
            callback(
                InputEvent::Key(KeyEvent {
                    key: KeyCode::Escape,
                    modifiers: Modifiers::NONE,
                }),
                self.buf.len(),
            );
            self.state = InputState::Normal;
        }
    }

    fn process_bytes<F: FnMut(InputEvent, usize)>(&mut self, mut callback: F, maybe_more: bool) {
        while !self.buf.is_empty() {
            match self.state {
                InputState::Pasting(offset) => {
                    let end_paste = b"\x1b[201~";
                    if let Some(idx) = self.buf.find_subsequence(offset, end_paste) {
                        let pasted =
                            String::from_utf8_lossy(&self.buf.as_slice()[0..idx]).to_string();
                        self.buf.advance(pasted.len() + end_paste.len());
                        callback(InputEvent::Paste(pasted), self.buf.len());
                        self.state = InputState::Normal;
                    } else {
                        self.state =
                            InputState::Pasting(self.buf.len().saturating_sub(end_paste.len()));
                        return;
                    }
                },
                InputState::EscapeMaybeAlt | InputState::Normal => {
                    // 结构化终端序列——SGR 鼠标、OSC、白名单 CSI 宿主回复——
                    // 是自主的宿主事件，不能与前导 Esc 击键进行 ALT 组合。
                    // 在 Normal 和 EscapeMaybeAlt 中都运行这些检查：如果我们正持有
                    // 停放的 ESC（EscapeMaybeAlt）且即将到来的字节匹配这些模式之一，
                    // 则 ESC 必须是真正的 Esc 击键，因此在分派序列之前刷新它。
                    // 否则，停放的 ESC 紧跟 `\x1b[<...M`（xterm 单独刷新 Esc，
                    // 然后在下一次读取中产生鼠标移动）会被分派为虚假的 ALT+`[`，
                    // 因为键映射将 `\x1b[` 注册为 Alt+`[`。
                    if self.buf.as_slice().get(0) == Some(&b'\x1b') {
                        if let Some((event, len)) = parse_sgr_mouse(self.buf.as_slice()) {
                            self.flush_parked_esc_if_held(&mut callback);
                            self.buf.advance(len);
                            callback(event, self.buf.len());
                            continue;
                        }

                        // OSC 序列检查——必须在不完整 SGR 鼠标的提前返回之前
                        if let Some((event, len)) = parse_osc(self.buf.as_slice()) {
                            self.flush_parked_esc_if_held(&mut callback);
                            self.buf.advance(len);
                            callback(event, self.buf.len());
                            continue;
                        }

                        // 不完整 OSC——缓冲并等待更多数据
                        if maybe_more && self.buf.as_slice().starts_with(b"\x1b]") {
                            self.flush_parked_esc_if_held(&mut callback);
                            return;
                        }

                        if maybe_more && self.buf.as_slice().starts_with(b"\x1b[<") {
                            self.flush_parked_esc_if_held(&mut callback);
                            return;
                        }

                        // 基于 CSI 的宿主终端报告（像素尺寸回复、DECRPM、DSR、
                        // Primary-DA）。必须在常规 CSI 键映射机制之前，否则该机制
                        // 会将 "\x1b[" 匹配为转义前缀并将字节作为键盘输入传递。
                        if let Some((event, len)) = parse_csi_report(self.buf.as_slice()) {
                            self.flush_parked_esc_if_held(&mut callback);
                            self.buf.advance(len);
                            callback(event, self.buf.len());
                            continue;
                        }

                        // 不完整 CSI ?... 报告（DECRPM、DSR 997 等）——等待更多数据，
                        // 以便报告分类路径可以匹配完整序列，而不是让键映射将前导字节
                        // 作为单独的键分派。
                        if maybe_more && self.buf.as_slice().starts_with(b"\x1b[?") {
                            self.flush_parked_esc_if_held(&mut callback);
                            return;
                        }
                    }

                    match (
                        self.key_map.lookup(self.buf.as_slice(), maybe_more),
                        maybe_more,
                    ) {
                        // 如果我们得到明确的 ESC 并且有更多数据跟随，那么这很可能是
                        // 后续按键的 Meta 版本。缓冲 escape 键并从输入中消耗它。
                        // dispatch_callback() 将发出 ESC 或 ALT 修饰的后续键。
                        (
                            Found::Exact(
                                len,
                                InputEvent::Key(KeyEvent {
                                    key: KeyCode::Escape,
                                    modifiers: Modifiers::NONE,
                                }),
                            ),
                            _,
                        ) if self.state == InputState::Normal && self.buf.len() > len => {
                            self.state = InputState::EscapeMaybeAlt;
                            self.buf.advance(len);
                        },
                        (Found::Exact(len, event), _) | (Found::Ambiguous(len, event), false) => {
                            // 在分派之前前进，以便 `dispatch_callback` 内部的
                            // `self.buf.len()` 已经反映此键的消耗。
                            self.buf.advance(len);
                            self.dispatch_callback(&mut callback, event.clone());
                        },
                        (Found::Ambiguous(_, _), true) | (Found::NeedData, true) => {
                            // 键映射发出信号"此缓冲区仍可能增长为已注册键，
                            // 给我更多字节。"当缓冲区已经持有结构上完整的 CSI 序列
                            // 且其最终字节不在键映射中时，该判断是错误的——最重要的是
                            // Kitty 键盘协议事件 `\x1b[<keycode>;<mods>u`，它永远不会
                            // 增长为键映射知道的任何内容。在此处返回会无限期卡住
                            // `self.buf`，吞掉其后到达的每个宿主回复（其中包括转发的
                            // `OSC 11;?` 查询的 OSC + DA1 字节），并在会话退出前
                            // 停滞宿主颜色转发。
                            //
                            // `Ambiguous(_, true)` 和 `NeedData(true)` 在实践中都会
                            // 到达此点：对于 `\x1b[<digits>;<digits>u`，trie 报告
                            // `Ambiguous(1, Escape)`（它将 ESC 单独作为匹配，将 ESC[…]
                            // 作为更长前缀），因此修复必须覆盖两种判断。
                            //
                            // 跳过未识别的 CSI 而不发出事件；需要键盘分派的调用者
                            // （kitty_parser、从 `StdinAnsiParser` 接收残差的单独
                            // `input_parser` 实例）通过 `strip_replies` 看到相同的字节，
                            // 该函数已将非白名单最终字节的 CSI 视为 `Malformed` 并推送通过。
                            if let Some(len) = complete_csi_len(self.buf.as_slice()) {
                                self.buf.advance(len);
                                continue;
                            }
                            return;
                        },
                        (Found::None, _) | (Found::NeedData, false) => {
                            // 没有预定义的键，因此提取一个 unicode 字符
                            if let Some((c, len)) = Self::decode_one_char(self.buf.as_slice()) {
                                self.buf.advance(len);
                                self.dispatch_callback(
                                    &mut callback,
                                    InputEvent::Key(KeyEvent {
                                        key: KeyCode::Char(c),
                                        modifiers: Modifiers::NONE,
                                    }),
                                );
                            } else {
                                // 我们需要更多数据来识别输入，因此让出切片的剩余部分
                                return;
                            }
                        },
                    }
                },
            }
        }
    }

    /// 将字节序列推入解析器。
    /// 每次识别到输入时，提供的 `callback` 将被传递解码后的 `InputEvent`。
    /// 如果没有足够的数据来完全解码序列，剩余数据将被缓冲直到下一次调用。
    /// `maybe_more` 标志控制如何处理歧义的部分序列。其意图是，如果你认为
    /// 马上能够提供更多数据，应将 `maybe_more` 设置为 true。这将导致解析器
    /// 推迟对部分前缀匹配的判断。你应尝试读取并在之后立即传入新数据。
    /// 如果你已尝试读取且立即可用的数据为空，则应随后使用空切片和
    /// `maybe_more=false` 调用 parse，以允许识别和处理部分数据。
    pub fn parse<F: FnMut(InputEvent)>(&mut self, bytes: &[u8], callback: F, maybe_more: bool) {
        // 重新绑定（而非 `mut callback: F`）以保持上游签名不变
        let mut callback = callback;
        self.parse_with_consumed(bytes, |event, _consumed| callback(event), maybe_more);
    }

    /// 类似于 [`InputParser::parse`]，但回调还接收产生每个事件所消耗的
    /// 输入字节数。这使得同时转发原始字节和解码事件的调用者能够在单个输入块
    /// 解码为多个事件时，将产生该事件的确切字节归因于每个事件。
    pub fn parse_with_consumed<F: FnMut(InputEvent, usize)>(
        &mut self,
        bytes: &[u8],
        mut callback: F,
        maybe_more: bool,
    ) {
        self.buf.extend_with(bytes);
        // `process_bytes` 报告每个事件后仍缓冲的字节；连续剩余量之间的
        // 下降量就是该事件消耗的字节数。
        let mut prev_remaining = self.buf.len();
        self.process_bytes(
            |event, remaining| {
                let consumed = prev_remaining.saturating_sub(remaining);
                prev_remaining = remaining;
                callback(event, consumed);
            },
            maybe_more,
        );
    }

    /// 解析器内部缓冲区中仍未处理的字节数。单独镜像此环形缓冲区的调用者
    /// （例如，与解码事件同时转发原始字节）可以将自己的缓冲区调整到完全相同的
    /// 长度，使两者永远不会偏离。
    pub fn buffered_len(&self) -> usize {
        self.buf.len()
    }

    pub fn parse_as_vec(&mut self, bytes: &[u8], maybe_more: bool) -> Vec<InputEvent> {
        let mut result = Vec::new();
        self.parse(bytes, |event| result.push(event), maybe_more);
        result
    }

    #[cfg(windows)]
    pub fn decode_input_records_as_vec(
        &mut self,
        records: &[winapi::um::wincon::INPUT_RECORD],
    ) -> Vec<InputEvent> {
        let mut result = Vec::new();
        self.decode_input_records(records, &mut |event| result.push(event));
        result
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const NO_MORE: bool = false;
    const MAYBE_MORE: bool = true;

    #[test]
    fn simple() {
        let mut p = InputParser::new();
        let inputs = p.parse_as_vec(b"hello", NO_MORE);
        assert_eq!(
            vec![
                InputEvent::Key(KeyEvent {
                    modifiers: Modifiers::NONE,
                    key: KeyCode::Char('h'),
                }),
                InputEvent::Key(KeyEvent {
                    modifiers: Modifiers::NONE,
                    key: KeyCode::Char('e'),
                }),
                InputEvent::Key(KeyEvent {
                    modifiers: Modifiers::NONE,
                    key: KeyCode::Char('l'),
                }),
                InputEvent::Key(KeyEvent {
                    modifiers: Modifiers::NONE,
                    key: KeyCode::Char('l'),
                }),
                InputEvent::Key(KeyEvent {
                    modifiers: Modifiers::NONE,
                    key: KeyCode::Char('o'),
                }),
            ],
            inputs
        );
    }

    #[test]
    fn control_characters() {
        let mut p = InputParser::new();
        let inputs = p.parse_as_vec(b"\x03\x1bJ\x7f", NO_MORE);
        assert_eq!(
            vec![
                InputEvent::Key(KeyEvent {
                    modifiers: Modifiers::CTRL,
                    key: KeyCode::Char('c'),
                }),
                InputEvent::Key(KeyEvent {
                    modifiers: Modifiers::ALT,
                    key: KeyCode::Char('J'),
                }),
                InputEvent::Key(KeyEvent {
                    modifiers: Modifiers::NONE,
                    key: KeyCode::Backspace,
                }),
            ],
            inputs
        );
    }

    #[test]
    fn arrow_keys() {
        let mut p = InputParser::new();
        let inputs = p.parse_as_vec(b"\x1bOA\x1bOB\x1bOC\x1bOD", NO_MORE);
        assert_eq!(
            vec![
                InputEvent::Key(KeyEvent {
                    modifiers: Modifiers::NONE,
                    key: KeyCode::ApplicationUpArrow,
                }),
                InputEvent::Key(KeyEvent {
                    modifiers: Modifiers::NONE,
                    key: KeyCode::ApplicationDownArrow,
                }),
                InputEvent::Key(KeyEvent {
                    modifiers: Modifiers::NONE,
                    key: KeyCode::ApplicationRightArrow,
                }),
                InputEvent::Key(KeyEvent {
                    modifiers: Modifiers::NONE,
                    key: KeyCode::ApplicationLeftArrow,
                }),
            ],
            inputs
        );
    }

    /// 解析 `bytes` 并将每个事件与其消耗的原始字节配对，
    /// 以与客户端标准输入相同的方式从输入副本中排出
    /// loop attributes raw bytes to events.
    fn parse_with_raw_bytes(bytes: &[u8], maybe_more: bool) -> Vec<(InputEvent, Vec<u8>)> {
        let mut p = InputParser::new();
        let mut collected: Vec<(InputEvent, usize)> = Vec::new();
        p.parse_with_consumed(bytes, |ev, n| collected.push((ev, n)), maybe_more);
        let mut buffer: Vec<u8> = bytes.to_vec();
        collected
            .into_iter()
            .map(|(ev, n)| {
                let take = n.min(buffer.len());
                let raw: Vec<u8> = buffer.drain(..take).collect();
                (ev, raw)
            })
            .collect()
    }

    #[test]
    fn typed_char_keeps_only_its_own_bytes_before_mouse_reports() {
        // A keystroke and two mouse reports arrive in one read: the key must be
        // paired with only its own byte and each report with its own bytes.
        let events = parse_with_raw_bytes(b"a\x1b[<35;52;16M\x1b[<35;49;16M", MAYBE_MORE);
        assert_eq!(
            events.len(),
            3,
            "expected key + 2 mouse events, got {:?}",
            events
        );
        assert!(
            matches!(
                events[0].0,
                InputEvent::Key(KeyEvent {
                    key: KeyCode::Char('a'),
                    ..
                })
            ),
            "first event should be the typed key, got {:?}",
            events[0].0
        );
        assert_eq!(
            events[0].1, b"a",
            "the keystroke must not carry the trailing mouse bytes"
        );
        assert!(matches!(events[1].0, InputEvent::Mouse(_)));
        assert_eq!(events[1].1, b"\x1b[<35;52;16M");
        assert!(matches!(events[2].0, InputEvent::Mouse(_)));
        assert_eq!(events[2].1, b"\x1b[<35;49;16M");
    }

    #[test]
    fn typed_char_keeps_only_its_own_bytes_after_mouse_reports() {
        // A mouse report precedes the keystroke in the read; the key must still
        // be paired with only its own byte.
        let events = parse_with_raw_bytes(b"\x1b[<35;52;16Ma", MAYBE_MORE);
        assert_eq!(events.len(), 2, "got {:?}", events);
        assert!(matches!(events[0].0, InputEvent::Mouse(_)));
        assert_eq!(events[0].1, b"\x1b[<35;52;16M");
        assert!(matches!(
            events[1].0,
            InputEvent::Key(KeyEvent {
                key: KeyCode::Char('a'),
                ..
            })
        ));
        assert_eq!(events[1].1, b"a");
    }

    #[test]
    fn consecutive_chars_before_mouse_each_keep_one_byte() {
        // Consecutive keystrokes in one read are each paired with their own byte.
        let events = parse_with_raw_bytes(b"ab\x1b[<35;52;16M", MAYBE_MORE);
        assert_eq!(events.len(), 3, "got {:?}", events);
        assert_eq!(events[0].1, b"a");
        assert_eq!(events[1].1, b"b");
        assert_eq!(events[2].1, b"\x1b[<35;52;16M");
    }

    #[test]
    fn single_event_keeps_all_its_bytes() {
        // A read that decodes into a single event pairs it with all of the
        // read's bytes, including a multi-byte sequence (`\x1bOA`).
        let events = parse_with_raw_bytes(b"\x1bOA", NO_MORE);
        assert_eq!(events.len(), 1, "got {:?}", events);
        assert!(matches!(
            events[0].0,
            InputEvent::Key(KeyEvent {
                key: KeyCode::ApplicationUpArrow,
                ..
            })
        ));
        assert_eq!(events[0].1, b"\x1bOA");
    }

    #[test]
    fn lone_esc_batch_then_mouse_report_batch() {
        // A lone ESC arrives in one batch and a complete mouse report in the
        // next. The ESC is held until the following batch disambiguates it;
        // both events are then emitted, each paired with its own bytes.
        let mut p = InputParser::new();
        let mut events: Vec<(InputEvent, usize)> = Vec::new();
        let mut buffer: Vec<u8> = Vec::new();

        buffer.extend_from_slice(b"\x1b");
        p.parse_with_consumed(b"\x1b", |ev, n| events.push((ev, n)), MAYBE_MORE);
        assert!(
            events.is_empty(),
            "a lone ESC with more data possibly coming must be held, got {:?}",
            events
        );

        buffer.extend_from_slice(b"\x1b[<35;62;16M");
        p.parse_with_consumed(b"\x1b[<35;62;16M", |ev, n| events.push((ev, n)), MAYBE_MORE);
        assert_eq!(events.len(), 2, "got {:?}", events);
        assert!(matches!(
            events[0].0,
            InputEvent::Key(KeyEvent {
                key: KeyCode::Escape,
                ..
            })
        ));
        assert_eq!(events[0].1, 1, "the ESC consumed its single byte");
        assert!(matches!(events[1].0, InputEvent::Mouse(_)));
        assert_eq!(events[1].1, 12, "the mouse report consumed its 12 bytes");

        // Draining the accumulated bytes per event, the way the client's
        // stdin loop does, pairs each event with its own raw bytes.
        let esc_bytes: Vec<u8> = buffer.drain(..events[0].1).collect();
        let mouse_bytes: Vec<u8> = buffer.drain(..events[1].1).collect();
        assert_eq!(esc_bytes, b"\x1b");
        assert_eq!(mouse_bytes, b"\x1b[<35;62;16M");
        assert!(buffer.is_empty());
    }

    #[test]
    fn paste_start_alone_is_consumed_silently_and_not_buffered() {
        let mut p = InputParser::new();
        let mut events: Vec<(InputEvent, usize)> = Vec::new();
        p.parse_with_consumed(b"\x1b[200~", |ev, n| events.push((ev, n)), MAYBE_MORE);
        assert!(
            events.is_empty(),
            "a lone paste-start marker must produce no events, got {:?}",
            events
        );
        assert_eq!(
            p.buffered_len(),
            0,
            "the paste-start bytes are consumed out of the parser buffer without any event reporting them"
        );
    }

    #[test]
    fn paste_start_with_partial_payload_buffers_only_the_payload() {
        let mut p = InputParser::new();
        let mut events: Vec<(InputEvent, usize)> = Vec::new();
        p.parse_with_consumed(b"\x1b[200~hel", |ev, n| events.push((ev, n)), MAYBE_MORE);
        assert!(events.is_empty(), "got {:?}", events);
        assert_eq!(
            p.buffered_len(),
            3,
            "only the pending paste payload remains buffered; the 6 marker bytes were consumed silently"
        );
    }

    #[test]
    fn parked_esc_before_partial_utf8_is_consumed_out_of_the_buffer() {
        let mut p = InputParser::new();
        let mut events: Vec<(InputEvent, usize)> = Vec::new();
        p.parse_with_consumed(b"\x1b\xc3", |ev, n| events.push((ev, n)), MAYBE_MORE);
        assert!(events.is_empty(), "got {:?}", events);
        assert_eq!(
            p.buffered_len(),
            1,
            "the parked ESC is held in parser state, not in the buffer; only the partial UTF-8 byte remains"
        );
    }

    #[test]
    fn newline_then_carriage_return_are_two_enter_events_with_their_own_bytes() {
        // In the legacy encoding a terminal sends `\r` for the Enter key and
        // `\n` for a control-j style newline; the keymap decodes both to
        // Enter. Arriving together they are two Enter events, each paired
        // with its own byte.
        let events = parse_with_raw_bytes(b"\n\r", MAYBE_MORE);
        assert_eq!(events.len(), 2, "got {:?}", events);
        for (event, raw) in &events {
            assert!(
                matches!(
                    event,
                    InputEvent::Key(KeyEvent {
                        key: KeyCode::Enter,
                        ..
                    })
                ),
                "expected an Enter key event, got {:?}",
                event
            );
            assert_eq!(raw.len(), 1, "each Enter is paired with a single byte");
        }
        assert_eq!(events[0].1, b"\n");
        assert_eq!(events[1].1, b"\r");
    }

    #[test]
    fn partial() {
        let mut p = InputParser::new();
        let mut inputs = Vec::new();
        // Fragment this F-key sequence across two different pushes
        p.parse(b"\x1b[11", |evt| inputs.push(evt), true);
        p.parse(b"~", |evt| inputs.push(evt), true);
        // make sure we recognize it as just the F-key
        assert_eq!(
            vec![InputEvent::Key(KeyEvent {
                modifiers: Modifiers::NONE,
                key: KeyCode::Function(1),
            })],
            inputs
        );
    }

    #[test]
    fn partial_ambig() {
        let mut p = InputParser::new();

        assert_eq!(
            vec![InputEvent::Key(KeyEvent {
                key: KeyCode::Escape,
                modifiers: Modifiers::NONE,
            })],
            p.parse_as_vec(b"\x1b", false)
        );

        let mut inputs = Vec::new();
        // An incomplete F-key sequence fragmented across two different pushes
        p.parse(b"\x1b[11", |evt| inputs.push(evt), MAYBE_MORE);
        p.parse(b"", |evt| inputs.push(evt), NO_MORE);
        // since we finish with maybe_more false (NO_MORE), the results should be the longest matching
        // parts of said f-key sequence
        assert_eq!(
            vec![
                InputEvent::Key(KeyEvent {
                    modifiers: Modifiers::ALT,
                    key: KeyCode::Char('['),
                }),
                InputEvent::Key(KeyEvent {
                    modifiers: Modifiers::NONE,
                    key: KeyCode::Char('1'),
                }),
                InputEvent::Key(KeyEvent {
                    modifiers: Modifiers::NONE,
                    key: KeyCode::Char('1'),
                }),
            ],
            inputs
        );
    }

    #[test]
    fn partial_mouse() {
        let mut p = InputParser::new();
        let mut inputs = Vec::new();
        // Fragment this mouse sequence across two different pushes
        p.parse(b"\x1b[<0;0;0", |evt| inputs.push(evt), true);
        p.parse(b"M", |evt| inputs.push(evt), true);
        // make sure we recognize it as just the mouse event
        assert_eq!(
            vec![InputEvent::Mouse(MouseEvent {
                x: 0,
                y: 0,
                mouse_buttons: MouseButtons::LEFT,
                modifiers: Modifiers::NONE,
            })],
            inputs
        );
    }

    #[test]
    fn partial_mouse_ambig() {
        let mut p = InputParser::new();
        let mut inputs = Vec::new();
        // Fragment this mouse sequence across two different pushes
        p.parse(b"\x1b[<", |evt| inputs.push(evt), MAYBE_MORE);
        p.parse(b"0;0;0", |evt| inputs.push(evt), NO_MORE);
        // since we finish with maybe_more false (NO_MORE), the results should be the longest matching
        // parts of said mouse sequence
        assert_eq!(
            vec![
                InputEvent::Key(KeyEvent {
                    modifiers: Modifiers::ALT,
                    key: KeyCode::Char('['),
                }),
                InputEvent::Key(KeyEvent {
                    modifiers: Modifiers::NONE,
                    key: KeyCode::Char('<'),
                }),
                InputEvent::Key(KeyEvent {
                    modifiers: Modifiers::NONE,
                    key: KeyCode::Char('0'),
                }),
                InputEvent::Key(KeyEvent {
                    modifiers: Modifiers::NONE,
                    key: KeyCode::Char(';'),
                }),
                InputEvent::Key(KeyEvent {
                    modifiers: Modifiers::NONE,
                    key: KeyCode::Char('0'),
                }),
                InputEvent::Key(KeyEvent {
                    modifiers: Modifiers::NONE,
                    key: KeyCode::Char(';'),
                }),
                InputEvent::Key(KeyEvent {
                    modifiers: Modifiers::NONE,
                    key: KeyCode::Char('0'),
                }),
            ],
            inputs
        );
    }

    #[test]
    fn alt_left_bracket() {
        // tests that `Alt` + `[` is recognized as a single
        // event rather than two events (one `Esc` the second `Char('[')`)
        let mut p = InputParser::new();

        let mut inputs = Vec::new();
        p.parse(b"\x1b[", |evt| inputs.push(evt), false);

        assert_eq!(
            vec![InputEvent::Key(KeyEvent {
                modifiers: Modifiers::ALT,
                key: KeyCode::Char('['),
            }),],
            inputs
        );
    }

    #[test]
    fn modify_other_keys_parse() {
        let mut p = InputParser::new();
        let inputs = p.parse_as_vec(
            b"\x1b[27;5;13~\x1b[27;5;9~\x1b[27;6;8~\x1b[27;2;127~\x1b[27;6;27~",
            NO_MORE,
        );
        assert_eq!(
            vec![
                InputEvent::Key(KeyEvent {
                    key: KeyCode::Enter,
                    modifiers: Modifiers::CTRL,
                }),
                InputEvent::Key(KeyEvent {
                    key: KeyCode::Tab,
                    modifiers: Modifiers::CTRL,
                }),
                InputEvent::Key(KeyEvent {
                    key: KeyCode::Backspace,
                    modifiers: Modifiers::CTRL | Modifiers::SHIFT,
                }),
                InputEvent::Key(KeyEvent {
                    key: KeyCode::Backspace,
                    modifiers: Modifiers::SHIFT,
                }),
                InputEvent::Key(KeyEvent {
                    key: KeyCode::Escape,
                    modifiers: Modifiers::CTRL | Modifiers::SHIFT,
                }),
            ],
            inputs
        );
    }

    #[test]
    fn modify_other_keys_encode() {
        let mode = KeyCodeEncodeModes {
            encoding: KeyboardEncoding::Xterm,
            newline_mode: false,
            application_cursor_keys: false,
            modify_other_keys: None,
        };
        let mode_1 = KeyCodeEncodeModes {
            encoding: KeyboardEncoding::Xterm,
            newline_mode: false,
            application_cursor_keys: false,
            modify_other_keys: Some(1),
        };
        let mode_2 = KeyCodeEncodeModes {
            encoding: KeyboardEncoding::Xterm,
            newline_mode: false,
            application_cursor_keys: false,
            modify_other_keys: Some(2),
        };

        assert_eq!(
            KeyCode::Enter.encode(Modifiers::CTRL, mode, true).unwrap(),
            "\r".to_string()
        );
        assert_eq!(
            KeyCode::Enter
                .encode(Modifiers::CTRL, mode_1, true)
                .unwrap(),
            "\x1b[27;5;13~".to_string()
        );
        assert_eq!(
            KeyCode::Enter
                .encode(Modifiers::CTRL | Modifiers::SHIFT, mode_1, true)
                .unwrap(),
            "\x1b[27;6;13~".to_string()
        );

        // This case is not conformant with xterm!
        // xterm just returns tab for CTRL-Tab when modify_other_keys
        // is not set.
        assert_eq!(
            KeyCode::Tab.encode(Modifiers::CTRL, mode, true).unwrap(),
            "\x1b[9;5u".to_string()
        );
        assert_eq!(
            KeyCode::Tab.encode(Modifiers::CTRL, mode_1, true).unwrap(),
            "\x1b[27;5;9~".to_string()
        );
        assert_eq!(
            KeyCode::Tab
                .encode(Modifiers::CTRL | Modifiers::SHIFT, mode_1, true)
                .unwrap(),
            "\x1b[27;6;9~".to_string()
        );

        assert_eq!(
            KeyCode::Char('c')
                .encode(Modifiers::CTRL, mode, true)
                .unwrap(),
            "\x03".to_string()
        );
        assert_eq!(
            KeyCode::Char('c')
                .encode(Modifiers::CTRL, mode_1, true)
                .unwrap(),
            "\x03".to_string()
        );
        assert_eq!(
            KeyCode::Char('c')
                .encode(Modifiers::CTRL, mode_2, true)
                .unwrap(),
            "\x1b[27;5;99~".to_string()
        );

        assert_eq!(
            KeyCode::Char('1')
                .encode(Modifiers::CTRL, mode, true)
                .unwrap(),
            "1".to_string()
        );
        assert_eq!(
            KeyCode::Char('1')
                .encode(Modifiers::CTRL, mode_2, true)
                .unwrap(),
            "\x1b[27;5;49~".to_string()
        );

        assert_eq!(
            KeyCode::Char(',')
                .encode(Modifiers::CTRL, mode, true)
                .unwrap(),
            ",".to_string()
        );
        assert_eq!(
            KeyCode::Char(',')
                .encode(Modifiers::CTRL, mode_2, true)
                .unwrap(),
            "\x1b[27;5;44~".to_string()
        );
    }

    #[test]
    fn encode_issue_892() {
        let mode = KeyCodeEncodeModes {
            encoding: KeyboardEncoding::Xterm,
            newline_mode: false,
            application_cursor_keys: false,
            modify_other_keys: None,
        };

        assert_eq!(
            KeyCode::LeftArrow
                .encode(Modifiers::NONE, mode, true)
                .unwrap(),
            "\x1b[D".to_string()
        );
        assert_eq!(
            KeyCode::LeftArrow
                .encode(Modifiers::ALT, mode, true)
                .unwrap(),
            "\x1b[1;3D".to_string()
        );
        assert_eq!(
            KeyCode::Home.encode(Modifiers::NONE, mode, true).unwrap(),
            "\x1b[H".to_string()
        );
        assert_eq!(
            KeyCode::Home.encode(Modifiers::ALT, mode, true).unwrap(),
            "\x1b[1;3H".to_string()
        );
        assert_eq!(
            KeyCode::End.encode(Modifiers::NONE, mode, true).unwrap(),
            "\x1b[F".to_string()
        );
        assert_eq!(
            KeyCode::End.encode(Modifiers::ALT, mode, true).unwrap(),
            "\x1b[1;3F".to_string()
        );
        assert_eq!(
            KeyCode::Tab.encode(Modifiers::ALT, mode, true).unwrap(),
            "\x1b\t".to_string()
        );
        assert_eq!(
            KeyCode::PageUp.encode(Modifiers::ALT, mode, true).unwrap(),
            "\x1b[5;3~".to_string()
        );
        assert_eq!(
            KeyCode::Function(1)
                .encode(Modifiers::NONE, mode, true)
                .unwrap(),
            "\x1bOP".to_string()
        );
    }

    #[test]
    fn partial_bracketed_paste() {
        let mut p = InputParser::new();

        let input = b"\x1b[200~1234";
        let input2 = b"5678\x1b[201~";

        let mut inputs = vec![];

        p.parse(input, |e| inputs.push(e), false);
        p.parse(input2, |e| inputs.push(e), false);

        assert_eq!(vec![InputEvent::Paste("12345678".to_owned())], inputs)
    }

    #[test]
    fn mouse_horizontal_scroll() {
        let mut p = InputParser::new();

        let input = b"\x1b[<66;42;12M\x1b[<67;42;12M";
        let res = p.parse_as_vec(input, MAYBE_MORE);

        assert_eq!(
            vec![
                InputEvent::Mouse(MouseEvent {
                    x: 42,
                    y: 12,
                    mouse_buttons: MouseButtons::HORZ_WHEEL | MouseButtons::WHEEL_POSITIVE,
                    modifiers: Modifiers::NONE,
                }),
                InputEvent::Mouse(MouseEvent {
                    x: 42,
                    y: 12,
                    mouse_buttons: MouseButtons::HORZ_WHEEL,
                    modifiers: Modifiers::NONE,
                })
            ],
            res
        );
    }

    #[test]
    fn encode_issue_3478_xterm() {
        let mode = KeyCodeEncodeModes {
            encoding: KeyboardEncoding::Xterm,
            newline_mode: false,
            application_cursor_keys: false,
            modify_other_keys: None,
        };

        assert_eq!(
            KeyCode::Numpad0
                .encode(Modifiers::NONE, mode, true)
                .unwrap(),
            "\u{1b}[2~".to_string()
        );
        assert_eq!(
            KeyCode::Numpad0
                .encode(Modifiers::SHIFT, mode, true)
                .unwrap(),
            "\u{1b}[2;2~".to_string()
        );

        assert_eq!(
            KeyCode::Numpad1
                .encode(Modifiers::NONE, mode, true)
                .unwrap(),
            "\u{1b}[F".to_string()
        );
        assert_eq!(
            KeyCode::Numpad1
                .encode(Modifiers::NONE | Modifiers::SHIFT, mode, true)
                .unwrap(),
            "\u{1b}[1;2F".to_string()
        );
    }

    #[test]
    fn encode_tab_with_modifiers() {
        let mode = KeyCodeEncodeModes {
            encoding: KeyboardEncoding::Xterm,
            newline_mode: false,
            application_cursor_keys: false,
            modify_other_keys: None,
        };

        let mods_to_result = [
            (Modifiers::SHIFT, "\u{1b}[Z"),
            (Modifiers::SHIFT | Modifiers::LEFT_SHIFT, "\u{1b}[Z"),
            (Modifiers::SHIFT | Modifiers::RIGHT_SHIFT, "\u{1b}[Z"),
            (Modifiers::CTRL, "\u{1b}[9;5u"),
            (Modifiers::CTRL | Modifiers::LEFT_CTRL, "\u{1b}[9;5u"),
            (Modifiers::CTRL | Modifiers::RIGHT_CTRL, "\u{1b}[9;5u"),
            (
                Modifiers::SHIFT | Modifiers::CTRL | Modifiers::LEFT_CTRL | Modifiers::LEFT_SHIFT,
                "\u{1b}[1;5Z",
            ),
        ];
        for (mods, result) in mods_to_result {
            assert_eq!(
                KeyCode::Tab.encode(mods, mode, true).unwrap(),
                result,
                "{:?}",
                mods
            );
        }
    }

    #[test]
    fn mouse_button1_press() {
        let mut p = InputParser::new();
        let res = p.parse_as_vec(b"\x1b[<0;42;12M", true);
        assert_eq!(
            res,
            vec![InputEvent::Mouse(MouseEvent {
                x: 42,
                y: 12,
                mouse_buttons: MouseButtons::LEFT,
                modifiers: Modifiers::NONE,
            })]
        );
    }

    #[test]
    fn mouse_button1_release() {
        let mut p = InputParser::new();
        let res = p.parse_as_vec(b"\x1b[<0;42;12m", true);
        assert_eq!(
            res,
            vec![InputEvent::Mouse(MouseEvent {
                x: 42,
                y: 12,
                mouse_buttons: MouseButtons::NONE,
                modifiers: Modifiers::NONE,
            })]
        );
    }

    #[test]
    fn mouse_button3_with_shift() {
        let mut p = InputParser::new();
        // button 2 (right) = 2, SHIFT adds 4 to p0 -> 6
        let res = p.parse_as_vec(b"\x1b[<6;10;20M", true);
        assert_eq!(
            res,
            vec![InputEvent::Mouse(MouseEvent {
                x: 10,
                y: 20,
                mouse_buttons: MouseButtons::RIGHT,
                modifiers: Modifiers::SHIFT,
            })]
        );
    }

    #[test]
    fn mouse_drag() {
        let mut p = InputParser::new();
        // button1 drag = 32
        let res = p.parse_as_vec(b"\x1b[<32;5;5M", true);
        assert_eq!(
            res,
            vec![InputEvent::Mouse(MouseEvent {
                x: 5,
                y: 5,
                mouse_buttons: MouseButtons::LEFT,
                modifiers: Modifiers::NONE,
            })]
        );
    }

    #[test]
    fn mouse_vertical_scroll_up() {
        let mut p = InputParser::new();
        // button4 press = 64
        let res = p.parse_as_vec(b"\x1b[<64;1;1M", true);
        assert_eq!(
            res,
            vec![InputEvent::Mouse(MouseEvent {
                x: 1,
                y: 1,
                mouse_buttons: MouseButtons::VERT_WHEEL | MouseButtons::WHEEL_POSITIVE,
                modifiers: Modifiers::NONE,
            })]
        );
    }

    #[test]
    fn mouse_vertical_scroll_down() {
        let mut p = InputParser::new();
        // button5 press = 65
        let res = p.parse_as_vec(b"\x1b[<65;1;1M", true);
        assert_eq!(
            res,
            vec![InputEvent::Mouse(MouseEvent {
                x: 1,
                y: 1,
                mouse_buttons: MouseButtons::VERT_WHEEL,
                modifiers: Modifiers::NONE,
            })]
        );
    }

    #[test]
    fn mouse_motion_no_buttons() {
        let mut p = InputParser::new();
        // motion with no buttons = 35
        let res = p.parse_as_vec(b"\x1b[<35;10;10M", true);
        assert_eq!(
            res,
            vec![InputEvent::Mouse(MouseEvent {
                x: 10,
                y: 10,
                mouse_buttons: MouseButtons::NONE,
                modifiers: Modifiers::NONE,
            })]
        );
    }

    #[test]
    fn mouse_with_ctrl_alt() {
        let mut p = InputParser::new();
        // button1 press = 0, ALT=8, CTRL=16 -> 0+8+16=24
        let res = p.parse_as_vec(b"\x1b[<24;1;1M", true);
        assert_eq!(
            res,
            vec![InputEvent::Mouse(MouseEvent {
                x: 1,
                y: 1,
                mouse_buttons: MouseButtons::LEFT,
                modifiers: Modifiers::ALT | Modifiers::CTRL,
            })]
        );
    }

    #[test]
    fn mouse_large_coordinates() {
        let mut p = InputParser::new();
        let res = p.parse_as_vec(b"\x1b[<0;999;999M", true);
        assert_eq!(
            res,
            vec![InputEvent::Mouse(MouseEvent {
                x: 999,
                y: 999,
                mouse_buttons: MouseButtons::LEFT,
                modifiers: Modifiers::NONE,
            })]
        );
    }

    #[test]
    fn mouse_followed_by_key() {
        let mut p = InputParser::new();
        let res = p.parse_as_vec(b"\x1b[<0;1;1Mhello", false);
        assert_eq!(res.len(), 6); // 1 mouse + 5 chars
        assert!(matches!(res[0], InputEvent::Mouse(_)));
        assert!(matches!(res[1], InputEvent::Key(_)));
    }

    #[test]
    fn two_mouse_events_back_to_back() {
        let mut p = InputParser::new();
        let res = p.parse_as_vec(b"\x1b[<0;1;1M\x1b[<0;2;2M", true);
        assert_eq!(res.len(), 2);
    }

    /// Regression for the xterm Esc-during-mouse-drag bug:
    /// xterm flushes a real Esc keypress as a single `\x1b` byte. If a mouse
    /// motion arrives in the next stdin read, upstream `StdinAnsiParser` may
    /// concatenate them into `\x1b\x1b[<...M`. Termwiz must parse this as
    /// two events (Esc then Mouse), not as Alt+`[` (which would happen if
    /// the keymap's `\x1b[`=Alt+`[` registration short-circuits the SGR
    /// mouse parser while in `EscapeMaybeAlt` state).
    #[test]
    fn esc_then_sgr_mouse_emits_esc_and_mouse() {
        let mut p = InputParser::new();
        let res = p.parse_as_vec(b"\x1b\x1b[<35;42;12M", MAYBE_MORE);
        assert_eq!(
            res,
            vec![
                InputEvent::Key(KeyEvent {
                    key: KeyCode::Escape,
                    modifiers: Modifiers::NONE,
                }),
                InputEvent::Mouse(MouseEvent {
                    x: 42,
                    y: 12,
                    mouse_buttons: MouseButtons::NONE,
                    modifiers: Modifiers::NONE,
                }),
            ]
        );
    }

    /// Same regression but for the cross-`parse()` case where the parked
    /// ESC is in `EscapeMaybeAlt` state from a prior call. The SGR mouse
    /// sequence arrives in a subsequent call.
    #[test]
    fn esc_then_sgr_mouse_across_parse_calls() {
        let mut p = InputParser::new();

        // First call: lone ESC byte. Termwiz parks no state because the
        // first arm only fires when there are bytes after the ESC; with
        // `MAYBE_MORE` it leaves the ESC pending in its internal buf and
        // emits nothing yet.
        let mut res = p.parse_as_vec(b"\x1b", MAYBE_MORE);
        assert!(
            res.is_empty(),
            "lone ESC should not emit yet under MAYBE_MORE"
        );

        // Second call: the mouse sequence arrives. The buffered ESC plus
        // these bytes form `\x1b\x1b[<...M` (the inner buf already has the
        // ESC; this call's bytes start with another ESC because that's
        // what xterm sends for the mouse sequence). Result must still be
        // Esc + Mouse, not Alt+`[`.
        res = p.parse_as_vec(b"\x1b[<35;42;12M", MAYBE_MORE);
        assert_eq!(
            res,
            vec![
                InputEvent::Key(KeyEvent {
                    key: KeyCode::Escape,
                    modifiers: Modifiers::NONE,
                }),
                InputEvent::Mouse(MouseEvent {
                    x: 42,
                    y: 12,
                    mouse_buttons: MouseButtons::NONE,
                    modifiers: Modifiers::NONE,
                }),
            ]
        );
    }

    /// Real Alt+Esc keystroke (`\x1b\x1b` with no further bytes) must
    /// still be recognised as Alt+Esc — the fix above must not regress
    /// this convention.
    #[test]
    fn alt_esc_still_recognized() {
        let mut p = InputParser::new();
        let res = p.parse_as_vec(b"\x1b\x1b", NO_MORE);
        assert_eq!(
            res,
            vec![InputEvent::Key(KeyEvent {
                key: KeyCode::Escape,
                modifiers: Modifiers::ALT,
            })]
        );
    }

    /// Esc keystroke followed by an OSC host reply (e.g. an OSC 11 color
    /// query response that arrives concatenated after a stray Esc byte
    /// the user pressed) must emit Esc and the OSC, not Alt-modify the
    /// OSC bytes.
    #[test]
    fn esc_then_osc_emits_esc_and_osc() {
        let mut p = InputParser::new();
        let res = p.parse_as_vec(b"\x1b\x1b]11;rgb:ffff/ffff/ffff\x1b\\", MAYBE_MORE);
        assert_eq!(
            res,
            vec![
                InputEvent::Key(KeyEvent {
                    key: KeyCode::Escape,
                    modifiers: Modifiers::NONE,
                }),
                InputEvent::OperatingSystemCommand(b"11;rgb:ffff/ffff/ffff".to_vec()),
            ]
        );
    }

    /// Esc followed by a CSI host-reply (whitelisted final byte). Must
    /// emit Esc and the report, never Alt+`[`.
    #[test]
    fn esc_then_csi_report_emits_esc_and_report() {
        let mut p = InputParser::new();
        // \x1b[?2026;0$y is a DECRPM reply for synchronised output mode.
        // Wrapped behind a stray Esc keystroke prefix.
        let res = p.parse_as_vec(b"\x1b\x1b[?2026;0$y", MAYBE_MORE);
        assert!(
            !res.is_empty(),
            "expected at least one event from Esc + CSI report"
        );
        assert!(
            matches!(
                res[0],
                InputEvent::Key(KeyEvent {
                    key: KeyCode::Escape,
                    modifiers: Modifiers::NONE,
                })
            ),
            "first event must be a bare Esc keystroke, got {:?}",
            res[0]
        );
        // The CSI report dispatches as DeviceControlReply via the
        // `parse_csi_report` whitelist. Anything but Alt+`[` is acceptable
        // for the second event; what we are guarding against is the
        // spurious Alt+`[` dispatch.
        for ev in &res {
            if let InputEvent::Key(KeyEvent { key, modifiers }) = ev {
                assert!(
                    !(matches!(key, KeyCode::Char('[')) && modifiers.contains(Modifiers::ALT)),
                    "must not emit Alt+`[`; got {:?}",
                    ev
                );
            }
        }
    }

    #[test]
    fn invalid_sgr_mouse_falls_through() {
        let mut p = InputParser::new();
        // Invalid: missing terminator, not enough params
        let res = p.parse_as_vec(b"\x1b[<0;1M", false);
        // Should NOT parse as mouse - falls through to keymap
        assert!(res.iter().all(|e| matches!(e, InputEvent::Key(_))));
    }

    #[test]
    fn osc_bel_terminated() {
        // Complete OSC sequence with BEL terminator
        let mut p = InputParser::new();
        let inputs = p.parse_as_vec(b"\x1b]99;i=test:p=title;Hello\x07", NO_MORE);
        assert_eq!(
            vec![InputEvent::OperatingSystemCommand(
                b"99;i=test:p=title;Hello".to_vec()
            )],
            inputs
        );
    }

    #[test]
    fn osc_st_terminated() {
        // Complete OSC sequence with ST terminator (ESC \)
        let mut p = InputParser::new();
        let inputs = p.parse_as_vec(b"\x1b]99;i=test:p=title;Hello\x1b\\", NO_MORE);
        assert_eq!(
            vec![InputEvent::OperatingSystemCommand(
                b"99;i=test:p=title;Hello".to_vec()
            )],
            inputs
        );
    }

    #[test]
    fn osc_partial_across_reads() {
        // OSC sequence split across two reads — must buffer first part
        let mut p = InputParser::new();
        let mut inputs = Vec::new();
        p.parse(
            b"\x1b]99;i=test:p=title;Hel",
            |evt| inputs.push(evt),
            MAYBE_MORE,
        );
        assert!(inputs.is_empty(), "no events yet - sequence incomplete");
        p.parse(b"lo\x1b\\", |evt| inputs.push(evt), MAYBE_MORE);
        assert_eq!(
            vec![InputEvent::OperatingSystemCommand(
                b"99;i=test:p=title;Hello".to_vec()
            )],
            inputs
        );
    }

    #[test]
    fn osc_followed_by_keypress() {
        // OSC sequence then regular key in same buffer
        let mut p = InputParser::new();
        let inputs = p.parse_as_vec(b"\x1b]99;i=test;clicked\x07x", NO_MORE);
        assert_eq!(
            vec![
                InputEvent::OperatingSystemCommand(b"99;i=test;clicked".to_vec()),
                InputEvent::Key(KeyEvent {
                    modifiers: Modifiers::NONE,
                    key: KeyCode::Char('x'),
                }),
            ],
            inputs
        );
    }

    #[test]
    fn keypress_followed_by_osc() {
        // Regular key then OSC sequence in same buffer
        let mut p = InputParser::new();
        let inputs = p.parse_as_vec(b"x\x1b]99;i=test;clicked\x07", NO_MORE);
        assert_eq!(
            vec![
                InputEvent::Key(KeyEvent {
                    modifiers: Modifiers::NONE,
                    key: KeyCode::Char('x'),
                }),
                InputEvent::OperatingSystemCommand(b"99;i=test;clicked".to_vec()),
            ],
            inputs
        );
    }

    #[test]
    fn osc_incomplete_degrades_to_keys() {
        // Incomplete OSC that never gets a terminator — when finalized with
        // maybe_more=false, must degrade to individual key events (not hang)
        let mut p = InputParser::new();
        let mut inputs = Vec::new();
        p.parse(b"\x1b]99;no-terminator", |evt| inputs.push(evt), MAYBE_MORE);
        assert!(inputs.is_empty(), "buffered while maybe_more=true");
        p.parse(b"", |evt| inputs.push(evt), NO_MORE);
        assert!(!inputs.is_empty(), "must emit something on finalization");
    }

    #[test]
    fn osc_non_99_code() {
        // Non-99 OSC codes are also captured as OperatingSystemCommand
        let mut p = InputParser::new();
        let inputs = p.parse_as_vec(b"\x1b]11;rgb:0000/0000/0000\x1b\\", NO_MORE);
        assert_eq!(
            vec![InputEvent::OperatingSystemCommand(
                b"11;rgb:0000/0000/0000".to_vec()
            )],
            inputs
        );
    }

    #[test]
    fn osc_empty_payload() {
        // Edge case: OSC with no payload between \x1b] and terminator
        let mut p = InputParser::new();
        let inputs = p.parse_as_vec(b"\x1b]\x07", NO_MORE);
        assert_eq!(
            vec![InputEvent::OperatingSystemCommand(b"".to_vec())],
            inputs
        );
    }

    #[test]
    fn csi_not_captured_as_osc() {
        // ESC [ (CSI) must NOT be captured as an OSC sequence.
        // This validates that only ESC ] triggers OSC parsing.
        let mut p = InputParser::new();
        let inputs = p.parse_as_vec(b"\x1b[A", NO_MORE);
        assert_eq!(
            vec![InputEvent::Key(KeyEvent {
                modifiers: Modifiers::NONE,
                key: KeyCode::UpArrow,
            })],
            inputs
        );
    }

    // =====================================================================
    // parse_csi_report (CSI report whitelist for host-reply forwarding)
    // =====================================================================

    fn csi_reply(intermediates: &[u8], params: &[u8], final_byte: u8, raw: &[u8]) -> InputEvent {
        InputEvent::DeviceControlReply {
            intermediates: intermediates.to_vec(),
            params: params.to_vec(),
            final_byte,
            raw: raw.to_vec(),
        }
    }

    #[test]
    fn csi_report_recognises_each_whitelisted_final_byte() {
        // `t` — pixel-dimension reply form `\x1b[4;H;Wt`.
        let bytes = b"\x1b[4;600;800t";
        let (evt, consumed) = parse_csi_report(bytes).expect("t accepted");
        assert_eq!(consumed, bytes.len());
        assert_eq!(evt, csi_reply(b"", b"4;600;800", b't', bytes));

        // `y` — DECRPM, e.g. sync-output support. Intermediate `$`.
        let bytes = b"\x1b[?2026;1$y";
        let (evt, consumed) = parse_csi_report(bytes).expect("y accepted");
        assert_eq!(consumed, bytes.len());
        assert_eq!(evt, csi_reply(b"$", b"?2026;1", b'y', bytes));

        // `c` — Primary-DA reply (barrier).
        let bytes = b"\x1b[?62;1;6c";
        let (evt, consumed) = parse_csi_report(bytes).expect("c accepted");
        assert_eq!(consumed, bytes.len());
        assert_eq!(evt, csi_reply(b"", b"?62;1;6", b'c', bytes));

        // `n` — DSR reply (used for theme notifications).
        let bytes = b"\x1b[?997;1n";
        let (evt, consumed) = parse_csi_report(bytes).expect("n accepted");
        assert_eq!(consumed, bytes.len());
        assert_eq!(evt, csi_reply(b"", b"?997;1", b'n', bytes));
    }

    #[test]
    fn csi_report_preserves_intermediates() {
        // DECRPM uses `$` as its intermediate byte — it must land in
        // `intermediates`, not `params`.
        let bytes = b"\x1b[?2026;2$y";
        let (evt, _len) = parse_csi_report(bytes).expect("DECRPM accepted");
        let InputEvent::DeviceControlReply {
            intermediates,
            params,
            final_byte,
            raw,
        } = evt
        else {
            panic!("expected DeviceControlReply, got {:?}", evt);
        };
        assert_eq!(intermediates, b"$");
        assert_eq!(params, b"?2026;2");
        assert_eq!(final_byte, b'y');
        assert_eq!(raw, bytes);
    }

    #[test]
    fn csi_report_rejects_non_whitelisted_final_bytes() {
        // `A` = cursor-up (keyboard input, not a report).
        assert!(parse_csi_report(b"\x1b[A").is_none());
        // `R` = cursor-position report — not whitelisted; must pass
        // through to the keyboard path.
        assert!(parse_csi_report(b"\x1b[24;80R").is_none());
        // `m` = SGR; appears in render streams but should never reach
        // stdin as a report.
        assert!(parse_csi_report(b"\x1b[0m").is_none());
    }

    #[test]
    fn csi_report_returns_none_on_truncated_input() {
        // No final byte within the supplied slice → caller should wait
        // for more bytes; `parse_csi_report` must not "commit" to a
        // partial parse.
        assert!(parse_csi_report(b"\x1b[4;600;800").is_none());
        // Only the lead-in; parameters haven't started.
        assert!(parse_csi_report(b"\x1b[").is_none());
        // Empty input — zero bytes to consume.
        assert!(parse_csi_report(b"").is_none());
    }

    #[test]
    fn csi_report_raw_preserves_input_byte_for_byte() {
        // `raw` must include the leading ESC through the final byte
        // inclusive, without adding or dropping any byte — the
        // forwarding path writes it verbatim to the pane's pty.
        let bytes = b"\x1b[4;16;8t";
        let (evt, consumed) = parse_csi_report(bytes).expect("accepted");
        assert_eq!(consumed, bytes.len());
        let InputEvent::DeviceControlReply { raw, .. } = evt else {
            panic!("wrong variant");
        };
        assert_eq!(&raw[..], bytes, "raw must be byte-identical to input");
    }

    #[test]
    fn focus_reports_decode_as_focus_events() {
        let mut p = InputParser::new();
        assert_eq!(
            p.parse_as_vec(b"\x1b[I", MAYBE_MORE),
            vec![InputEvent::FocusGained],
        );
        assert_eq!(
            p.parse_as_vec(b"\x1b[O", MAYBE_MORE),
            vec![InputEvent::FocusLost],
        );
    }

    #[test]
    fn focus_report_split_across_reads_still_decodes_as_one_event() {
        let mut p = InputParser::new();
        assert_eq!(p.parse_as_vec(b"\x1b[", MAYBE_MORE), vec![]);
        assert_eq!(
            p.parse_as_vec(b"I", MAYBE_MORE),
            vec![InputEvent::FocusGained],
        );
    }

    #[test]
    fn focus_reports_never_degrade_into_literal_characters() {
        let mut p = InputParser::new();
        let events = p.parse_as_vec(b"\x1b[O\x1b[I", MAYBE_MORE);
        assert_eq!(
            events,
            vec![InputEvent::FocusLost, InputEvent::FocusGained],
            "a focus report must not decode as Alt+[ plus a literal I/O keystroke"
        );
    }

    #[test]
    fn alt_bracket_is_still_recognized_for_other_following_bytes() {
        let mut p = InputParser::new();
        assert_eq!(
            p.parse_as_vec(b"\x1b[x", MAYBE_MORE),
            vec![
                InputEvent::Key(KeyEvent {
                    key: KeyCode::Char('['),
                    modifiers: Modifiers::ALT,
                }),
                InputEvent::Key(KeyEvent {
                    key: KeyCode::Char('x'),
                    modifiers: Modifiers::NONE,
                }),
            ],
        );
    }
}
