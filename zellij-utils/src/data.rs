use crate::home::default_layout_dir;
use crate::input::actions::{Action, RunCommandAction};
use crate::input::config::{ConversionError, KdlError};
use crate::input::keybinds::Keybinds;
use crate::input::layout::{
    Layout, PercentOrFixed, Run, RunPlugin, RunPluginLocation, RunPluginOrAlias,
};
pub use crate::input::options::PaneFrameStyle;
use crate::pane_size::PaneGeom;
use crate::position::Position;
use crate::shared::{colors as default_colors, eightbit_to_rgb};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt;
use std::fs::Metadata;
use std::hash::{Hash, Hasher};
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::str::{self, FromStr};
use std::time::Duration;
use strum_macros::{Display, EnumDiscriminants, EnumIter, EnumString};
use unicode_width::UnicodeWidthChar;

#[cfg(not(target_family = "wasm"))]
use crate::vendored::termwiz::{
    input::KittyKeyboardFlags,
    input::{KeyCode, KeyCodeEncodeModes, KeyboardEncoding, Modifiers},
};

pub type ClientId = u16; // TODO: 与 crate 类型合并？

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnblockCondition {
    /// 仅在退出状态为 0（成功）时解除阻塞
    OnExitSuccess,
    /// 仅在退出状态非零（失败）时解除阻塞
    OnExitFailure,
    /// 在任何退出时解除阻塞（成功或失败）
    OnAnyExit,
}

impl UnblockCondition {
    /// 检查给定退出状态是否满足条件
    pub fn is_met(&self, exit_status: i32) -> bool {
        match self {
            UnblockCondition::OnExitSuccess => exit_status == 0,
            UnblockCondition::OnExitFailure => exit_status != 0,
            UnblockCondition::OnAnyExit => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandOrPlugin {
    Command(RunCommandAction),
    Plugin(RunPluginOrAlias),
    File(FileToOpen), // 在配置的编辑器中打开文件
}

impl CommandOrPlugin {
    pub fn new_command(command: Vec<String>) -> Self {
        CommandOrPlugin::Command(RunCommandAction::new(command))
    }
}

pub fn client_id_to_colors(
    client_id: ClientId,
    colors: MultiplayerColors,
) -> Option<(PaletteColor, PaletteColor)> {
    //（主色，辅色）
    let black = PaletteColor::EightBit(default_colors::BLACK);
    match client_id {
        1 => Some((colors.player_1, black)),
        2 => Some((colors.player_2, black)),
        3 => Some((colors.player_3, black)),
        4 => Some((colors.player_4, black)),
        5 => Some((colors.player_5, black)),
        6 => Some((colors.player_6, black)),
        7 => Some((colors.player_7, black)),
        8 => Some((colors.player_8, black)),
        9 => Some((colors.player_9, black)),
        10 => Some((colors.player_10, black)),
        _ => None,
    }
}

pub fn single_client_color(colors: Palette) -> (PaletteColor, PaletteColor) {
    (colors.green, colors.black)
}

impl FromStr for KeyWithModifier {
    type Err = Box<dyn std::error::Error>;
    fn from_str(key_str: &str) -> Result<Self, Self::Err> {
        let mut key_string_parts: Vec<&str> = key_str.split_ascii_whitespace().collect();
        let bare_key: BareKey = BareKey::from_str(key_string_parts.pop().ok_or("empty key")?)?;
        let mut key_modifiers: BTreeSet<KeyModifier> = BTreeSet::new();
        for stringified_modifier in key_string_parts {
            key_modifiers.insert(KeyModifier::from_str(stringified_modifier)?);
        }
        Ok(KeyWithModifier {
            bare_key,
            key_modifiers,
        })
    }
}

#[derive(Debug, Clone, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct KeyWithModifier {
    pub bare_key: BareKey,
    pub key_modifiers: BTreeSet<KeyModifier>,
}

impl PartialEq for KeyWithModifier {
    fn eq(&self, other: &Self) -> bool {
        match (self.bare_key, other.bare_key) {
            (BareKey::Char(self_char), BareKey::Char(other_char))
                if self_char.to_ascii_lowercase() == other_char.to_ascii_lowercase() =>
            {
                let mut self_cloned = self.clone();
                let mut other_cloned = other.clone();
                if self_char.is_ascii_uppercase() {
                    self_cloned.bare_key = BareKey::Char(self_char.to_ascii_lowercase());
                    self_cloned.key_modifiers.insert(KeyModifier::Shift);
                }
                if other_char.is_ascii_uppercase() {
                    other_cloned.bare_key = BareKey::Char(self_char.to_ascii_lowercase());
                    other_cloned.key_modifiers.insert(KeyModifier::Shift);
                }
                self_cloned.bare_key == other_cloned.bare_key
                    && self_cloned.key_modifiers == other_cloned.key_modifiers
            },
            _ => self.bare_key == other.bare_key && self.key_modifiers == other.key_modifiers,
        }
    }
}

impl Hash for KeyWithModifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self.bare_key {
            BareKey::Char(character) if character.is_ascii_uppercase() => {
                let mut to_hash = self.clone();
                to_hash.bare_key = BareKey::Char(character.to_ascii_lowercase());
                to_hash.key_modifiers.insert(KeyModifier::Shift);
                to_hash.bare_key.hash(state);
                to_hash.key_modifiers.hash(state);
            },
            _ => {
                self.bare_key.hash(state);
                self.key_modifiers.hash(state);
            },
        }
    }
}

impl fmt::Display for KeyWithModifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.key_modifiers.is_empty() {
            write!(f, "{}", self.bare_key)
        } else {
            write!(
                f,
                "{} {}",
                self.key_modifiers
                    .iter()
                    .map(|m| m.to_string())
                    .collect::<Vec<_>>()
                    .join(" "),
                self.bare_key
            )
        }
    }
}

#[cfg(not(target_family = "wasm"))]
impl Into<Modifiers> for &KeyModifier {
    fn into(self) -> Modifiers {
        match self {
            KeyModifier::Shift => Modifiers::SHIFT,
            KeyModifier::Alt => Modifiers::ALT,
            KeyModifier::Ctrl => Modifiers::CTRL,
            KeyModifier::Super => Modifiers::SUPER,
        }
    }
}

#[derive(Eq, Clone, Copy, Debug, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub enum BareKey {
    PageDown,
    PageUp,
    Left,
    Down,
    Up,
    Right,
    Home,
    End,
    Backspace,
    Delete,
    Insert,
    F(u8),
    Char(char),
    Tab,
    Esc,
    Enter,
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    Menu,
}

impl fmt::Display for BareKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BareKey::PageDown => write!(f, "PgDn"),
            BareKey::PageUp => write!(f, "PgUp"),
            BareKey::Left => write!(f, "←"),
            BareKey::Down => write!(f, "↓"),
            BareKey::Up => write!(f, "↑"),
            BareKey::Right => write!(f, "→"),
            BareKey::Home => write!(f, "HOME"),
            BareKey::End => write!(f, "END"),
            BareKey::Backspace => write!(f, "BACKSPACE"),
            BareKey::Delete => write!(f, "DEL"),
            BareKey::Insert => write!(f, "INS"),
            BareKey::F(index) => write!(f, "F{}", index),
            BareKey::Char(' ') => write!(f, "SPACE"),
            BareKey::Char(character) => write!(f, "{}", character),
            BareKey::Tab => write!(f, "TAB"),
            BareKey::Esc => write!(f, "ESC"),
            BareKey::Enter => write!(f, "ENTER"),
            BareKey::CapsLock => write!(f, "CAPSlOCK"),
            BareKey::ScrollLock => write!(f, "SCROLLlOCK"),
            BareKey::NumLock => write!(f, "NUMLOCK"),
            BareKey::PrintScreen => write!(f, "PRINTSCREEN"),
            BareKey::Pause => write!(f, "PAUSE"),
            BareKey::Menu => write!(f, "MENU"),
        }
    }
}

impl FromStr for BareKey {
    type Err = Box<dyn std::error::Error>;
    fn from_str(key_str: &str) -> Result<Self, Self::Err> {
        match key_str.to_ascii_lowercase().as_str() {
            "pagedown" => Ok(BareKey::PageDown),
            "pageup" => Ok(BareKey::PageUp),
            "left" => Ok(BareKey::Left),
            "down" => Ok(BareKey::Down),
            "up" => Ok(BareKey::Up),
            "right" => Ok(BareKey::Right),
            "home" => Ok(BareKey::Home),
            "end" => Ok(BareKey::End),
            "backspace" => Ok(BareKey::Backspace),
            "delete" => Ok(BareKey::Delete),
            "del" => Ok(BareKey::Delete),
            "insert" => Ok(BareKey::Insert),
            "f1" => Ok(BareKey::F(1)),
            "f2" => Ok(BareKey::F(2)),
            "f3" => Ok(BareKey::F(3)),
            "f4" => Ok(BareKey::F(4)),
            "f5" => Ok(BareKey::F(5)),
            "f6" => Ok(BareKey::F(6)),
            "f7" => Ok(BareKey::F(7)),
            "f8" => Ok(BareKey::F(8)),
            "f9" => Ok(BareKey::F(9)),
            "f10" => Ok(BareKey::F(10)),
            "f11" => Ok(BareKey::F(11)),
            "f12" => Ok(BareKey::F(12)),
            "tab" => Ok(BareKey::Tab),
            "esc" => Ok(BareKey::Esc),
            "enter" => Ok(BareKey::Enter),
            "capslock" => Ok(BareKey::CapsLock),
            "scrolllock" => Ok(BareKey::ScrollLock),
            "numlock" => Ok(BareKey::NumLock),
            "printscreen" => Ok(BareKey::PrintScreen),
            "pause" => Ok(BareKey::Pause),
            "menu" => Ok(BareKey::Menu),
            "space" => Ok(BareKey::Char(' ')),
            _ => {
                if key_str.chars().count() == 1 {
                    if let Some(character) = key_str.chars().next() {
                        return Ok(BareKey::Char(character));
                    }
                }
                Err("unsupported key".into())
            },
        }
    }
}

#[derive(
    Eq, Clone, Copy, Debug, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord, Display,
)]
pub enum KeyModifier {
    Ctrl,
    Alt,
    Shift,
    Super,
}

impl FromStr for KeyModifier {
    type Err = Box<dyn std::error::Error>;
    fn from_str(key_str: &str) -> Result<Self, Self::Err> {
        match key_str.to_ascii_lowercase().as_str() {
            "shift" => Ok(KeyModifier::Shift),
            "alt" => Ok(KeyModifier::Alt),
            "ctrl" => Ok(KeyModifier::Ctrl),
            "super" => Ok(KeyModifier::Super),
            _ => Err("unsupported modifier".into()),
        }
    }
}

impl BareKey {
    pub fn from_bytes_with_u(bytes: &[u8]) -> Option<Self> {
        match str::from_utf8(bytes) {
            Ok("27") => Some(BareKey::Esc),
            Ok("13") => Some(BareKey::Enter),
            Ok("9") => Some(BareKey::Tab),
            Ok("127") => Some(BareKey::Backspace),
            Ok("57358") => Some(BareKey::CapsLock),
            Ok("57359") => Some(BareKey::ScrollLock),
            Ok("57360") => Some(BareKey::NumLock),
            Ok("57361") => Some(BareKey::PrintScreen),
            Ok("57362") => Some(BareKey::Pause),
            Ok("57363") => Some(BareKey::Menu),
            Ok("57399") => Some(BareKey::Char('0')),
            Ok("57400") => Some(BareKey::Char('1')),
            Ok("57401") => Some(BareKey::Char('2')),
            Ok("57402") => Some(BareKey::Char('3')),
            Ok("57403") => Some(BareKey::Char('4')),
            Ok("57404") => Some(BareKey::Char('5')),
            Ok("57405") => Some(BareKey::Char('6')),
            Ok("57406") => Some(BareKey::Char('7')),
            Ok("57407") => Some(BareKey::Char('8')),
            Ok("57408") => Some(BareKey::Char('9')),
            Ok("57409") => Some(BareKey::Char('.')),
            Ok("57410") => Some(BareKey::Char('/')),
            Ok("57411") => Some(BareKey::Char('*')),
            Ok("57412") => Some(BareKey::Char('-')),
            Ok("57413") => Some(BareKey::Char('+')),
            Ok("57414") => Some(BareKey::Enter),
            Ok("57415") => Some(BareKey::Char('=')),
            Ok("57417") => Some(BareKey::Left),
            Ok("57418") => Some(BareKey::Right),
            Ok("57419") => Some(BareKey::Up),
            Ok("57420") => Some(BareKey::Down),
            Ok("57421") => Some(BareKey::PageUp),
            Ok("57422") => Some(BareKey::PageDown),
            Ok("57423") => Some(BareKey::Home),
            Ok("57424") => Some(BareKey::End),
            Ok("57425") => Some(BareKey::Insert),
            Ok("57426") => Some(BareKey::Delete),
            Ok(num) => u32::from_str_radix(num, 10)
                .ok()
                .and_then(char::from_u32)
                .map(BareKey::Char),
            _ => None,
        }
    }
    pub fn from_bytes_with_tilde(bytes: &[u8]) -> Option<Self> {
        match str::from_utf8(bytes) {
            Ok("2") => Some(BareKey::Insert),
            Ok("3") => Some(BareKey::Delete),
            Ok("5") => Some(BareKey::PageUp),
            Ok("6") => Some(BareKey::PageDown),
            Ok("7") => Some(BareKey::Home),
            Ok("8") => Some(BareKey::End),
            Ok("11") => Some(BareKey::F(1)),
            Ok("12") => Some(BareKey::F(2)),
            Ok("13") => Some(BareKey::F(3)),
            Ok("14") => Some(BareKey::F(4)),
            Ok("15") => Some(BareKey::F(5)),
            Ok("17") => Some(BareKey::F(6)),
            Ok("18") => Some(BareKey::F(7)),
            Ok("19") => Some(BareKey::F(8)),
            Ok("20") => Some(BareKey::F(9)),
            Ok("21") => Some(BareKey::F(10)),
            Ok("23") => Some(BareKey::F(11)),
            Ok("24") => Some(BareKey::F(12)),
            _ => None,
        }
    }
    pub fn from_bytes_with_no_ending_byte(bytes: &[u8]) -> Option<Self> {
        match str::from_utf8(bytes) {
            Ok("1D") | Ok("D") => Some(BareKey::Left),
            Ok("1C") | Ok("C") => Some(BareKey::Right),
            Ok("1A") | Ok("A") => Some(BareKey::Up),
            Ok("1B") | Ok("B") => Some(BareKey::Down),
            Ok("1H") | Ok("H") => Some(BareKey::Home),
            Ok("1F") | Ok("F") => Some(BareKey::End),
            Ok("1P") | Ok("P") => Some(BareKey::F(1)),
            Ok("1Q") | Ok("Q") => Some(BareKey::F(2)),
            Ok("1S") | Ok("S") => Some(BareKey::F(4)),
            _ => None,
        }
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct ModifierFlags: u8 {
        const SHIFT   = 0b0000_0001;
        const ALT     = 0b0000_0010;
        const CONTROL = 0b0000_0100;
        const SUPER   = 0b0000_1000;
        // 我们实际上没有使用下面的内容，保留在这里是为了在需要添加时保持完整性
        // 稍后
        const HYPER = 0b0001_0000;
        const META = 0b0010_0000;
        const CAPS_LOCK = 0b0100_0000;
        const NUM_LOCK = 0b1000_0000;
    }
}

impl KeyModifier {
    pub fn from_bytes(bytes: &[u8]) -> BTreeSet<KeyModifier> {
        let modifier_flags = str::from_utf8(bytes)
            .ok() // 转换为字符串：（例如 "16"）
            .and_then(|s| u8::from_str_radix(&s, 10).ok()) // 转换为 u8：（例如 16）
            .map(|s| s.saturating_sub(1)) // 减 1：（例如 15）
            .and_then(|b| ModifierFlags::from_bits(b)); // 位标志：（0b0000_1111：Shift、Alt、Control、Super）
        let mut key_modifiers = BTreeSet::new();
        if let Some(modifier_flags) = modifier_flags {
            for name in modifier_flags.iter() {
                match name {
                    ModifierFlags::SHIFT => key_modifiers.insert(KeyModifier::Shift),
                    ModifierFlags::ALT => key_modifiers.insert(KeyModifier::Alt),
                    ModifierFlags::CONTROL => key_modifiers.insert(KeyModifier::Ctrl),
                    ModifierFlags::SUPER => key_modifiers.insert(KeyModifier::Super),
                    _ => false,
                };
            }
        }
        key_modifiers
    }
}

impl KeyWithModifier {
    pub fn new(bare_key: BareKey) -> Self {
        KeyWithModifier {
            bare_key,
            key_modifiers: BTreeSet::new(),
        }
    }
    pub fn new_with_modifiers(bare_key: BareKey, key_modifiers: BTreeSet<KeyModifier>) -> Self {
        KeyWithModifier {
            bare_key,
            key_modifiers,
        }
    }
    pub fn with_shift_modifier(mut self) -> Self {
        self.key_modifiers.insert(KeyModifier::Shift);
        self
    }
    pub fn with_alt_modifier(mut self) -> Self {
        self.key_modifiers.insert(KeyModifier::Alt);
        self
    }
    pub fn with_ctrl_modifier(mut self) -> Self {
        self.key_modifiers.insert(KeyModifier::Ctrl);
        self
    }
    pub fn with_super_modifier(mut self) -> Self {
        self.key_modifiers.insert(KeyModifier::Super);
        self
    }
    pub fn from_bytes_with_u(number_bytes: &[u8], modifier_bytes: &[u8]) -> Option<Self> {
        // CSI number ; modifiers u
        let bare_key = BareKey::from_bytes_with_u(number_bytes);
        match bare_key {
            Some(bare_key) => {
                let key_modifiers = KeyModifier::from_bytes(modifier_bytes);
                Some(KeyWithModifier {
                    bare_key,
                    key_modifiers,
                })
            },
            _ => None,
        }
    }
    pub fn from_bytes_with_tilde(number_bytes: &[u8], modifier_bytes: &[u8]) -> Option<Self> {
        // CSI number ; modifiers ~
        let bare_key = BareKey::from_bytes_with_tilde(number_bytes);
        match bare_key {
            Some(bare_key) => {
                let key_modifiers = KeyModifier::from_bytes(modifier_bytes);
                Some(KeyWithModifier {
                    bare_key,
                    key_modifiers,
                })
            },
            _ => None,
        }
    }
    pub fn from_bytes_with_no_ending_byte(
        number_bytes: &[u8],
        modifier_bytes: &[u8],
    ) -> Option<Self> {
        // CSI 1; modifiers [ABCDEFHPQS]
        let bare_key = BareKey::from_bytes_with_no_ending_byte(number_bytes);
        match bare_key {
            Some(bare_key) => {
                let key_modifiers = KeyModifier::from_bytes(modifier_bytes);
                Some(KeyWithModifier {
                    bare_key,
                    key_modifiers,
                })
            },
            _ => None,
        }
    }
    pub fn strip_common_modifiers(&self, common_modifiers: &Vec<KeyModifier>) -> Self {
        let common_modifiers: BTreeSet<&KeyModifier> = common_modifiers.into_iter().collect();
        KeyWithModifier {
            bare_key: self.bare_key.clone(),
            key_modifiers: self
                .key_modifiers
                .iter()
                .filter(|m| !common_modifiers.contains(m))
                .cloned()
                .collect(),
        }
    }
    pub fn is_key_without_modifier(&self, key: BareKey) -> bool {
        self.bare_key == key && self.key_modifiers.is_empty()
    }
    pub fn is_key_with_ctrl_modifier(&self, key: BareKey) -> bool {
        self.bare_key == key && self.key_modifiers.contains(&KeyModifier::Ctrl)
    }
    pub fn is_key_with_alt_modifier(&self, key: BareKey) -> bool {
        self.bare_key == key && self.key_modifiers.contains(&KeyModifier::Alt)
    }
    pub fn is_key_with_shift_modifier(&self, key: BareKey) -> bool {
        self.bare_key == key && self.key_modifiers.contains(&KeyModifier::Shift)
    }
    pub fn is_key_with_super_modifier(&self, key: BareKey) -> bool {
        self.bare_key == key && self.key_modifiers.contains(&KeyModifier::Super)
    }
    pub fn is_cancel_key(&self) -> bool {
        // self.bare_key == BareKey::Esc || self.is_key_with_ctrl_modifier(BareKey::Char('c'))
        self.bare_key == BareKey::Esc
    }
    #[cfg(not(target_family = "wasm"))]
    pub fn to_termwiz_modifiers(&self) -> Modifiers {
        let mut modifiers = Modifiers::empty();
        for modifier in &self.key_modifiers {
            modifiers.set(modifier.into(), true);
        }
        modifiers
    }
    #[cfg(not(target_family = "wasm"))]
    pub fn to_termwiz_keycode(&self) -> KeyCode {
        match self.bare_key {
            BareKey::PageDown => KeyCode::PageDown,
            BareKey::PageUp => KeyCode::PageUp,
            BareKey::Left => KeyCode::LeftArrow,
            BareKey::Down => KeyCode::DownArrow,
            BareKey::Up => KeyCode::UpArrow,
            BareKey::Right => KeyCode::RightArrow,
            BareKey::Home => KeyCode::Home,
            BareKey::End => KeyCode::End,
            BareKey::Backspace => KeyCode::Backspace,
            BareKey::Delete => KeyCode::Delete,
            BareKey::Insert => KeyCode::Insert,
            BareKey::F(index) => KeyCode::Function(index),
            BareKey::Char(character) => KeyCode::Char(character),
            BareKey::Tab => KeyCode::Tab,
            BareKey::Esc => KeyCode::Escape,
            BareKey::Enter => KeyCode::Enter,
            BareKey::CapsLock => KeyCode::CapsLock,
            BareKey::ScrollLock => KeyCode::ScrollLock,
            BareKey::NumLock => KeyCode::NumLock,
            BareKey::PrintScreen => KeyCode::PrintScreen,
            BareKey::Pause => KeyCode::Pause,
            BareKey::Menu => KeyCode::Menu,
        }
    }
    #[cfg(not(target_family = "wasm"))]
    pub fn serialize_non_kitty(&self) -> Option<String> {
        let modifiers = self.to_termwiz_modifiers();
        let key_code_encode_modes = KeyCodeEncodeModes {
            encoding: KeyboardEncoding::Xterm,
            // 所有这些标志都为 false，因为它们在此序列化之前已经处理过
            // 序列化
            application_cursor_keys: false,
            newline_mode: false,
            modify_other_keys: None,
        };
        self.to_termwiz_keycode()
            .encode(modifiers, key_code_encode_modes, true)
            .ok()
    }
    #[cfg(not(target_family = "wasm"))]
    pub fn serialize_kitty(&self) -> Option<String> {
        let modifiers = self.to_termwiz_modifiers();
        let key_code_encode_modes = KeyCodeEncodeModes {
            encoding: KeyboardEncoding::Kitty(KittyKeyboardFlags::DISAMBIGUATE_ESCAPE_CODES),
            // 所有这些标志都为 false，因为它们在此序列化之前已经处理过
            // 序列化
            application_cursor_keys: false,
            newline_mode: false,
            modify_other_keys: None,
        };
        self.to_termwiz_keycode()
            .encode(modifiers, key_code_encode_modes, true)
            .ok()
    }
    pub fn has_no_modifiers(&self) -> bool {
        self.key_modifiers.is_empty()
    }
    pub fn has_modifiers(&self, modifiers: &[KeyModifier]) -> bool {
        for modifier in modifiers {
            if !self.key_modifiers.contains(modifier) {
                return false;
            }
        }
        true
    }
    pub fn has_only_modifiers(&self, modifiers: &[KeyModifier]) -> bool {
        for modifier in modifiers {
            if !self.key_modifiers.contains(modifier) {
                return false;
            }
        }
        if self.key_modifiers.len() != modifiers.len() {
            return false;
        }
        true
    }
}

#[derive(Eq, Clone, Copy, Debug, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Default for Direction {
    fn default() -> Self {
        Direction::Left
    }
}

impl Direction {
    pub fn invert(&self) -> Direction {
        match *self {
            Direction::Left => Direction::Right,
            Direction::Down => Direction::Up,
            Direction::Up => Direction::Down,
            Direction::Right => Direction::Left,
        }
    }

    pub fn is_horizontal(&self) -> bool {
        matches!(self, Direction::Left | Direction::Right)
    }

    pub fn is_vertical(&self) -> bool {
        matches!(self, Direction::Down | Direction::Up)
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Direction::Left => write!(f, "←"),
            Direction::Right => write!(f, "→"),
            Direction::Up => write!(f, "↑"),
            Direction::Down => write!(f, "↓"),
        }
    }
}

impl FromStr for Direction {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Left" | "left" => Ok(Direction::Left),
            "Right" | "right" => Ok(Direction::Right),
            "Up" | "up" => Ok(Direction::Up),
            "Down" | "down" => Ok(Direction::Down),
            _ => Err(format!(
                "解析方向失败。未知的方向：{}",
                s
            )),
        }
    }
}

/// 要执行的调整大小操作。
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub enum Resize {
    Increase,
    Decrease,
}

impl Default for Resize {
    fn default() -> Self {
        Resize::Increase
    }
}

impl Resize {
    pub fn invert(&self) -> Self {
        match self {
            Resize::Increase => Resize::Decrease,
            Resize::Decrease => Resize::Increase,
        }
    }
}

impl fmt::Display for Resize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Resize::Increase => write!(f, "+"),
            Resize::Decrease => write!(f, "-"),
        }
    }
}

impl FromStr for Resize {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Increase" | "increase" | "+" => Ok(Resize::Increase),
            "Decrease" | "decrease" | "-" => Ok(Resize::Decrease),
            _ => Err(format!(
                "解析调整大小类型失败。未知的指示符 '{}'",
                s
            )),
        }
    }
}

/// 完整描述调整大小操作的容器类型。
///
/// 最好这样理解：
///
/// - `resize` 表示在此调整大小操作中窗格的总*面积*将如何变化
///   操作。
/// - `direction` 有两个含义：
///     - `None` 表示均匀调整所有边框
///     - 其他值表示移动指定的边框以实现面积变化
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct ResizeStrategy {
    /// 是增加还是减少总面积
    pub resize: Resize,
    /// 使用哪个边框（如果有）来改变面积
    pub direction: Option<Direction>,
    /// 如果设置为 true（默认值），向视口边框方向增加大小将被反转。
    /// 例如这样的场景（"向右增加"）：
    ///
    /// ```text
    /// +---+---+
    /// |   | X |->
    /// +---+---+
    /// ```
    ///
    /// 会变成这样（"向左减少"）：
    ///
    /// ```text
    /// +---+---+
    /// |   |-> |
    /// +---+---+
    /// ```
    pub invert_on_boundaries: bool,
}

impl From<Direction> for ResizeStrategy {
    fn from(direction: Direction) -> Self {
        ResizeStrategy::new(Resize::Increase, Some(direction))
    }
}

impl From<Resize> for ResizeStrategy {
    fn from(resize: Resize) -> Self {
        ResizeStrategy::new(resize, None)
    }
}

impl ResizeStrategy {
    pub fn new(resize: Resize, direction: Option<Direction>) -> Self {
        ResizeStrategy {
            resize,
            direction,
            invert_on_boundaries: true,
        }
    }

    pub fn invert(&self) -> ResizeStrategy {
        let resize = match self.resize {
            Resize::Increase => Resize::Decrease,
            Resize::Decrease => Resize::Increase,
        };
        let direction = match self.direction {
            Some(direction) => Some(direction.invert()),
            None => None,
        };

        ResizeStrategy::new(resize, direction)
    }

    pub fn resize_type(&self) -> Resize {
        self.resize
    }

    pub fn direction(&self) -> Option<Direction> {
        self.direction
    }

    pub fn direction_horizontal(&self) -> bool {
        matches!(
            self.direction,
            Some(Direction::Left) | Some(Direction::Right)
        )
    }

    pub fn direction_vertical(&self) -> bool {
        matches!(self.direction, Some(Direction::Up) | Some(Direction::Down))
    }

    pub fn resize_increase(&self) -> bool {
        self.resize == Resize::Increase
    }

    pub fn resize_decrease(&self) -> bool {
        self.resize == Resize::Decrease
    }

    pub fn move_left_border_left(&self) -> bool {
        (self.resize == Resize::Increase) && (self.direction == Some(Direction::Left))
    }

    pub fn move_left_border_right(&self) -> bool {
        (self.resize == Resize::Decrease) && (self.direction == Some(Direction::Left))
    }

    pub fn move_lower_border_down(&self) -> bool {
        (self.resize == Resize::Increase) && (self.direction == Some(Direction::Down))
    }

    pub fn move_lower_border_up(&self) -> bool {
        (self.resize == Resize::Decrease) && (self.direction == Some(Direction::Down))
    }

    pub fn move_upper_border_up(&self) -> bool {
        (self.resize == Resize::Increase) && (self.direction == Some(Direction::Up))
    }

    pub fn move_upper_border_down(&self) -> bool {
        (self.resize == Resize::Decrease) && (self.direction == Some(Direction::Up))
    }

    pub fn move_right_border_right(&self) -> bool {
        (self.resize == Resize::Increase) && (self.direction == Some(Direction::Right))
    }

    pub fn move_right_border_left(&self) -> bool {
        (self.resize == Resize::Decrease) && (self.direction == Some(Direction::Right))
    }

    pub fn move_all_borders_out(&self) -> bool {
        (self.resize == Resize::Increase) && (self.direction == None)
    }

    pub fn move_all_borders_in(&self) -> bool {
        (self.resize == Resize::Decrease) && (self.direction == None)
    }
}

impl fmt::Display for ResizeStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let resize = match self.resize {
            Resize::Increase => "increase",
            Resize::Decrease => "decrease",
        };
        let border = match self.direction {
            Some(Direction::Left) => "left",
            Some(Direction::Down) => "bottom",
            Some(Direction::Up) => "top",
            Some(Direction::Right) => "right",
            None => "every",
        };

        write!(f, "{} size on {} border", resize, border)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
// FIXME: 这应该扩展以处理不同的按钮点击（不仅仅是
// 左键点击），并且 `ScrollUp` 和 `ScrollDown` 事件可能可以
// 合并为单个 `Scroll(isize)` 事件。
pub enum Mouse {
    ScrollUp(usize),   // 行数
    ScrollDown(usize), // 行数
    ScrollLeft(usize),
    ScrollRight(usize),
    LeftClick(isize, usize),  // 行和列
    RightClick(isize, usize), // 行和列
    Hold(isize, usize),       // 行和列
    Release(isize, usize),    // 行和列
    Hover(isize, usize),      // 行和列
}

impl Mouse {
    pub fn position(&self) -> Option<(usize, usize)> {
        //（行，列）
        match self {
            Mouse::LeftClick(line, column) => Some((*line as usize, *column as usize)),
            Mouse::RightClick(line, column) => Some((*line as usize, *column as usize)),
            Mouse::Hold(line, column) => Some((*line as usize, *column as usize)),
            Mouse::Release(line, column) => Some((*line as usize, *column as usize)),
            Mouse::Hover(line, column) => Some((*line as usize, *column as usize)),
            _ => None,
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FileMetadata {
    pub is_dir: bool,
    pub is_file: bool,
    pub is_symlink: bool,
    pub len: u64,
}

impl From<Metadata> for FileMetadata {
    fn from(metadata: Metadata) -> Self {
        FileMetadata {
            is_dir: metadata.is_dir(),
            is_file: metadata.is_file(),
            is_symlink: metadata.is_symlink(),
            len: metadata.len(),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StyledText {
    pub text: String,
    pub indices: Vec<Vec<usize>>,
}

/// 这些事件可以通过 `zellij-tile` 导出的 subscribe 方法订阅。
/// 订阅后，它们将触发 `ZellijPlugin` trait 的 `update` 方法。
#[derive(Debug, Clone, PartialEq, EnumDiscriminants, Display, Serialize, Deserialize)]
#[strum_discriminants(derive(EnumString, Hash, Serialize, Deserialize))]
#[strum_discriminants(name(EventType))]
#[non_exhaustive]
pub enum Event {
    ModeUpdate(ModeInfo),
    TabUpdate(Vec<TabInfo>),
    PaneUpdate(PaneManifest),
    /// 当用户聚焦在此插件的窗格上时按下了一个按键
    Key(KeyWithModifier),
    /// 当用户聚焦在此插件的窗格上时发生了鼠标事件
    Mouse(Mouse),
    /// 由 `zellij-tile` 导出的 `set_timeout` 方法设置的计时器已过期。
    Timer(f64),
    /// 应用中任意位置将文本复制到了剪贴板
    CopyToClipboard(CopyDestination),
    /// 应用中任意位置复制文本到剪贴板失败
    SystemClipboardFailure,
    /// 应用中任意位置接收到了输入
    InputReceived,
    /// 此插件变为可见或不可见
    Visible(bool),
    /// 来自插件某个 worker 的消息
    CustomMessage(
        String, // 消息
        String, // 载荷
    ),
    /// 在 Zellij 当前工作目录文件夹中的某处创建了文件
    FileSystemCreate(Vec<(PathBuf, Option<FileMetadata>)>),
    /// 在 Zellij 当前工作目录文件夹中的某处访问了文件
    FileSystemRead(Vec<(PathBuf, Option<FileMetadata>)>),
    /// 在 Zellij 当前工作目录文件夹中的某处修改了文件
    FileSystemUpdate(Vec<(PathBuf, Option<FileMetadata>)>),
    /// 在 Zellij 当前工作目录文件夹中的某处删除了文件
    FileSystemDelete(Vec<(PathBuf, Option<FileMetadata>)>),
    /// 插件权限请求的结果
    PermissionRequestResult(PermissionStatus),
    SessionUpdate(
        Vec<SessionInfo>,
        Vec<(String, Duration)>, // 可复活的会话
    ),
    RunCommandResult(Option<i32>, Vec<u8>, Vec<u8>, BTreeMap<String, String>), // exit_code、STDOUT、STDERR、
    // 上下文
    WebRequestResult(
        u16,
        BTreeMap<String, String>,
        Vec<u8>,
        BTreeMap<String, String>,
    ), // 状态码、
    // 请求头、
    // 响应体、
    // 上下文
    CommandPaneOpened(u32, Context), // u32 - terminal_pane_id
    CommandPaneExited(u32, Option<i32>, Context), // u32 - terminal_pane_id，Option<i32> -
    // exit_code
    PaneClosed(PaneId),
    EditPaneOpened(u32, Context),              // u32 - terminal_pane_id
    EditPaneExited(u32, Option<i32>, Context), // u32 - terminal_pane_id，Option<i32> - 退出码
    CommandPaneReRun(u32, Context),            // u32 - terminal_pane_id，Option<i32> -
    FailedToWriteConfigToDisk(Option<String>), // String -> 写入失败的文件路径
    ListClients(Vec<ClientInfo>),
    HostFolderChanged(PathBuf),               // PathBuf -> 新的宿主文件夹
    FailedToChangeHostFolder(Option<String>), // String -> 更改时遇到的错误
    PastedText(String),
    ConfigWasWrittenToDisk,
    WebServerStatus(WebServerStatus),
    FailedToStartWebServer(String),
    BeforeClose,
    InterceptedKeyPress(KeyWithModifier),
    /// 用户执行了一个操作（需要 InterceptInput 权限）
    UserAction(Action, ClientId, Option<u32>, Option<ClientId>), // Action、client_id、terminal_id、cli_client_id
    PaneRenderReport(HashMap<PaneId, PaneContents>),
    ActionComplete(Action, Option<PaneId>, BTreeMap<String, String>), // Action、pane_id、context
    CwdChanged(PaneId, PathBuf, Vec<ClientId>), // pane_id、cwd、focused_client_ids
    CommandChanged(PaneId, Vec<String>, bool, Vec<ClientId>), // pane_id、command、is_foreground、focused_client_ids
    AvailableLayoutInfo(Vec<LayoutInfo>, Vec<LayoutWithError>),
    PluginConfigurationChanged(BTreeMap<String, String>),
    HighlightClicked {
        pane_id: PaneId,
        pattern: String,
        matched_string: String,
        context: BTreeMap<String, String>,
    },
    /// 在插件加载和重新配置时发送一次的初始快捷键绑定。
    /// 订阅此事件的插件表示它们缓存快捷键绑定
    /// 并且可以处理不带快捷键绑定的轻量级 ModeUpdate 事件。
    InitialKeybinds(KeybindsVec),
    /// 宿主终端指示了其调色板主题模式（CSI 2031 / DSR 997）。
    HostTerminalThemeChanged(HostTerminalThemeMode),
    SoftKeyboardVisibilityChanged(bool),
    HintText(BTreeMap<usize, StyledText>),
    ActivePaneScroll(Option<(usize, usize)>),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HostTerminalThemeMode {
    Dark,
    Light,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumDiscriminants, Display, Serialize, Deserialize)]
pub enum WebServerStatus {
    Online(String), // String -> 基础 URL
    Offline,
    DifferentVersion(String), // 版本
}

#[derive(
    Debug,
    PartialEq,
    Eq,
    Hash,
    Copy,
    Clone,
    EnumDiscriminants,
    Display,
    Serialize,
    Deserialize,
    PartialOrd,
    Ord,
)]
#[strum_discriminants(derive(EnumString, Hash, Serialize, Deserialize, Display, PartialOrd, Ord))]
#[strum_discriminants(name(PermissionType))]
#[non_exhaustive]
pub enum Permission {
    ReadApplicationState,
    ChangeApplicationState,
    OpenFiles,
    RunCommands,
    OpenTerminalsOrPlugins,
    WriteToStdin,
    WebAccess,
    ReadCliPipes,
    MessageAndLaunchOtherPlugins,
    Reconfigure,
    FullHdAccess,
    StartWebServer,
    InterceptInput,
    ReadPaneContents,
    RunActionsAsUser,
    WriteToClipboard,
    ReadSessionEnvironmentVariables,
}

impl PermissionType {
    pub fn display_name(&self) -> String {
        match self {
            PermissionType::ReadApplicationState => {
                "Access Zellij state (Panes, Tabs and UI)".to_owned()
            },
            PermissionType::ChangeApplicationState => {
                "Change Zellij state (Panes, Tabs and UI) and run commands".to_owned()
            },
            PermissionType::OpenFiles => "Open files (eg. for editing)".to_owned(),
            PermissionType::RunCommands => "Run commands".to_owned(),
            PermissionType::OpenTerminalsOrPlugins => "Start new terminals and plugins".to_owned(),
            PermissionType::WriteToStdin => "Write to standard input (STDIN)".to_owned(),
            PermissionType::WebAccess => "Make web requests".to_owned(),
            PermissionType::ReadCliPipes => "Control command line pipes and output".to_owned(),
            PermissionType::MessageAndLaunchOtherPlugins => {
                "Send messages to and launch other plugins".to_owned()
            },
            PermissionType::Reconfigure => "Change Zellij runtime configuration".to_owned(),
            PermissionType::FullHdAccess => "Full access to the hard-drive".to_owned(),
            PermissionType::StartWebServer => {
                "Start a local web server to serve Zellij sessions".to_owned()
            },
            PermissionType::InterceptInput => "Intercept Input (keyboard & mouse)".to_owned(),
            PermissionType::ReadPaneContents => {
                "Read pane contents (viewport and selection)".to_owned()
            },
            PermissionType::RunActionsAsUser => "Execute actions as the user".to_owned(),
            PermissionType::WriteToClipboard => "Write to clipboard".to_owned(),
            PermissionType::ReadSessionEnvironmentVariables => {
                "Read environment variables present upon session creation".to_owned()
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct PluginPermission {
    pub name: String,
    pub permissions: Vec<PermissionType>,
}

impl PluginPermission {
    pub fn new(name: String, permissions: Vec<PermissionType>) -> Self {
        PluginPermission { name, permissions }
    }
}

/// 描述不同的输入模式，这些模式会改变按键的解释方式。
#[derive(
    Debug,
    PartialEq,
    Eq,
    Hash,
    Copy,
    Clone,
    EnumIter,
    Serialize,
    Deserialize,
    ValueEnum,
    PartialOrd,
    Ord,
)]
pub enum InputMode {
    /// 在 `Normal` 模式下，输入总是写入终端，除了通向
    /// 其他模式的快捷键
    #[serde(alias = "normal")]
    Normal,
    /// 在 `Locked` 模式下，输入总是写入终端，所有快捷键都被禁用
    /// 除了返回普通模式的快捷键
    #[serde(alias = "locked")]
    Locked,
    /// `Resize` 模式允许调整不同现有窗格的大小。
    #[serde(alias = "resize")]
    Resize,
    /// `Pane` 模式允许创建和关闭窗格，以及在窗格之间移动。
    #[serde(alias = "pane")]
    Pane,
    /// `Tab` 模式允许创建和关闭标签页，以及在标签页之间移动。
    #[serde(alias = "tab")]
    Tab,
    /// `Scroll` 模式允许在窗格内上下滚动。
    #[serde(alias = "scroll")]
    Scroll,
    /// `EnterSearch` 模式允许在窗格的回滚缓冲区中输入搜索关键词。
    #[serde(alias = "entersearch")]
    EnterSearch,
    /// `Search` 模式允许在窗格中搜索术语（`Scroll` 的超集）。
    #[serde(alias = "search")]
    Search,
    /// `RenameTab` 模式允许为标签页分配新名称。
    #[serde(alias = "renametab")]
    RenameTab,
    /// `RenamePane` 模式允许为窗格分配新名称。
    #[serde(alias = "renamepane")]
    RenamePane,
    /// `Session` 模式允许分离会话
    #[serde(alias = "session")]
    Session,
    /// `Move` 模式允许在标签页内移动不同的现有窗格
    #[serde(alias = "move")]
    Move,
    /// `Prompt` 模式允许与活动提示交互。
    #[serde(alias = "prompt")]
    Prompt,
    /// `Tmux` 模式允许基本的 tmux 快捷键绑定功能
    #[serde(alias = "tmux")]
    Tmux,
}

impl Default for InputMode {
    fn default() -> InputMode {
        InputMode::Normal
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, ValueEnum)]
pub enum ThemeHue {
    #[serde(alias = "light")]
    Light,
    #[serde(alias = "dark")]
    Dark,
}
impl Default for ThemeHue {
    fn default() -> ThemeHue {
        ThemeHue::Dark
    }
}

impl FromStr for ThemeHue {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "light" => Ok(ThemeHue::Light),
            "dark" => Ok(ThemeHue::Dark),
            e => Err(format!(
                "未知的主题色相：'{}'（期望 'dark' 或 'light'）",
                e
            )),
        }
    }
}

impl fmt::Display for ThemeHue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThemeHue::Light => write!(f, "light"),
            ThemeHue::Dark => write!(f, "dark"),
        }
    }
}

impl From<ThemeHue> for HostTerminalThemeMode {
    fn from(hue: ThemeHue) -> Self {
        match hue {
            ThemeHue::Light => HostTerminalThemeMode::Light,
            ThemeHue::Dark => HostTerminalThemeMode::Dark,
        }
    }
}

impl From<HostTerminalThemeMode> for ThemeHue {
    fn from(mode: HostTerminalThemeMode) -> Self {
        match mode {
            HostTerminalThemeMode::Light => ThemeHue::Light,
            HostTerminalThemeMode::Dark => ThemeHue::Dark,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PaletteColor {
    Rgb((u8, u8, u8)),
    EightBit(u8),
}
impl Default for PaletteColor {
    fn default() -> PaletteColor {
        PaletteColor::EightBit(0)
    }
}

/// 插件提供的正则表达式高亮的优先级层。
/// 较高优先级的层在视觉上优先于较低优先级的层
/// 当高亮重叠时。内置高亮（鼠标选择、
/// 搜索结果）始终优先于所有插件层。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HighlightLayer {
    Hint,           // 最低：纯模式匹配（路径、URL、IP）
    Tool,           // 中等：由运行时领域知识支持（git、docker、k8s）
    ActionFeedback, // 最高：显式用户操作的结果（搜索、书签）
}

impl Default for HighlightLayer {
    fn default() -> Self {
        HighlightLayer::Hint
    }
}

/// 插件提供的正则表达式高亮的样式。
/// 基于主题的变体引用 `style.colors.text_unselected.*`。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HighlightStyle {
    None,      // 无颜色覆盖 — 与 bold/italic/underline 一起用于仅样式高亮
    Emphasis0, // fg = emphasis_0，无 bg 覆盖
    Emphasis1, // fg = emphasis_1，无 bg 覆盖
    Emphasis2, // fg = emphasis_2，无 bg 覆盖
    Emphasis3, // fg = emphasis_3，无 bg 覆盖
    BackgroundEmphasis0, // bg = emphasis_0，fg = background
    BackgroundEmphasis1, // bg = emphasis_1，fg = background
    BackgroundEmphasis2, // bg = emphasis_2，fg = background
    BackgroundEmphasis3, // bg = emphasis_3，fg = background
    CustomRgb {
        fg: Option<(u8, u8, u8)>,
        bg: Option<(u8, u8, u8)>,
    },
    CustomIndex {
        fg: Option<u8>,
        bg: Option<u8>,
    },
}

/// 插件发送的一个模式 + 样式对。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegexHighlight {
    pub pattern: String, // 用于 upsert 的键；也是正则表达式源
    pub style: HighlightStyle,
    pub layer: HighlightLayer,
    pub context: BTreeMap<String, String>, // 点击时原样回显的任意数据
    pub on_hover: bool, // 如果为 true，仅在光标与此匹配项重叠时渲染
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub tooltip_text: Option<String>, // 悬停在匹配项上时显示在窗格框架底部
}

// 这些用于 Web 客户端
impl PaletteColor {
    pub fn as_rgb_str(&self) -> String {
        let (r, g, b) = match *self {
            Self::Rgb((r, g, b)) => (r, g, b),
            Self::EightBit(c) => eightbit_to_rgb(c),
        };
        format!("rgb({}, {}, {})", r, g, b)
    }
    pub fn from_rgb_str(rgb_str: &str) -> Self {
        let trimmed = rgb_str.trim();

        if !trimmed.starts_with("rgb(") || !trimmed.ends_with(')') {
            return Self::default();
        }

        let inner = trimmed
            .strip_prefix("rgb(")
            .and_then(|s| s.strip_suffix(')'))
            .unwrap_or("");

        let parts: Vec<&str> = inner.split(',').collect();

        if parts.len() != 3 {
            return Self::default();
        }

        let mut rgb_values = [0u8; 3];
        for (i, part) in parts.iter().enumerate() {
            if let Some(rgb_val) = rgb_values.get_mut(i) {
                if let Ok(parsed) = part.trim().parse::<u8>() {
                    *rgb_val = parsed;
                } else {
                    return Self::default();
                }
            }
        }

        Self::Rgb((rgb_values[0], rgb_values[1], rgb_values[2]))
    }
}

impl FromStr for InputMode {
    type Err = ConversionError;

    fn from_str(s: &str) -> Result<Self, ConversionError> {
        match s {
            "normal" | "Normal" => Ok(InputMode::Normal),
            "locked" | "Locked" => Ok(InputMode::Locked),
            "resize" | "Resize" => Ok(InputMode::Resize),
            "pane" | "Pane" => Ok(InputMode::Pane),
            "tab" | "Tab" => Ok(InputMode::Tab),
            "search" | "Search" => Ok(InputMode::Search),
            "scroll" | "Scroll" => Ok(InputMode::Scroll),
            "renametab" | "RenameTab" => Ok(InputMode::RenameTab),
            "renamepane" | "RenamePane" => Ok(InputMode::RenamePane),
            "session" | "Session" => Ok(InputMode::Session),
            "move" | "Move" => Ok(InputMode::Move),
            "prompt" | "Prompt" => Ok(InputMode::Prompt),
            "tmux" | "Tmux" => Ok(InputMode::Tmux),
            "entersearch" | "Entersearch" | "EnterSearch" => Ok(InputMode::EnterSearch),
            e => Err(ConversionError::UnknownInputMode(e.into())),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PaletteSource {
    Default,
    Xresources,
}
impl Default for PaletteSource {
    fn default() -> PaletteSource {
        PaletteSource::Default
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub struct Palette {
    pub source: PaletteSource,
    pub theme_hue: ThemeHue,
    pub fg: PaletteColor,
    pub bg: PaletteColor,
    pub black: PaletteColor,
    pub red: PaletteColor,
    pub green: PaletteColor,
    pub yellow: PaletteColor,
    pub blue: PaletteColor,
    pub magenta: PaletteColor,
    pub cyan: PaletteColor,
    pub white: PaletteColor,
    pub orange: PaletteColor,
    pub gray: PaletteColor,
    pub purple: PaletteColor,
    pub gold: PaletteColor,
    pub silver: PaletteColor,
    pub pink: PaletteColor,
    pub brown: PaletteColor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Style {
    pub colors: Styling,
    pub rounded_corners: bool,
    pub hide_session_name: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Coloration {
    NoStyling,
    Styled(StyleDeclaration),
}

impl Coloration {
    pub fn with_fallback(&self, fallback: StyleDeclaration) -> StyleDeclaration {
        match &self {
            Coloration::NoStyling => fallback,
            Coloration::Styled(style) => *style,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Styling {
    pub text_unselected: StyleDeclaration,
    pub text_selected: StyleDeclaration,
    pub ribbon_unselected: StyleDeclaration,
    pub ribbon_selected: StyleDeclaration,
    pub table_title: StyleDeclaration,
    pub table_cell_unselected: StyleDeclaration,
    pub table_cell_selected: StyleDeclaration,
    pub list_unselected: StyleDeclaration,
    pub list_selected: StyleDeclaration,
    pub frame_unselected: Option<StyleDeclaration>,
    pub frame_selected: StyleDeclaration,
    pub frame_highlight: StyleDeclaration,
    pub exit_code_success: StyleDeclaration,
    pub exit_code_error: StyleDeclaration,
    pub multiplayer_user_colors: MultiplayerColors,
}

#[derive(Debug, Copy, Default, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct StyleDeclaration {
    pub base: PaletteColor,
    pub background: PaletteColor,
    pub emphasis_0: PaletteColor,
    pub emphasis_1: PaletteColor,
    pub emphasis_2: PaletteColor,
    pub emphasis_3: PaletteColor,
}

#[derive(Debug, Copy, Default, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct MultiplayerColors {
    pub player_1: PaletteColor,
    pub player_2: PaletteColor,
    pub player_3: PaletteColor,
    pub player_4: PaletteColor,
    pub player_5: PaletteColor,
    pub player_6: PaletteColor,
    pub player_7: PaletteColor,
    pub player_8: PaletteColor,
    pub player_9: PaletteColor,
    pub player_10: PaletteColor,
}

pub const DEFAULT_STYLES: Styling = Styling {
    text_unselected: StyleDeclaration {
        base: PaletteColor::EightBit(default_colors::BRIGHT_GRAY),
        emphasis_0: PaletteColor::EightBit(default_colors::ORANGE),
        emphasis_1: PaletteColor::EightBit(default_colors::CYAN),
        emphasis_2: PaletteColor::EightBit(default_colors::GREEN),
        emphasis_3: PaletteColor::EightBit(default_colors::MAGENTA),
        background: PaletteColor::EightBit(default_colors::GRAY),
    },
    text_selected: StyleDeclaration {
        base: PaletteColor::EightBit(default_colors::BRIGHT_GRAY),
        emphasis_0: PaletteColor::EightBit(default_colors::ORANGE),
        emphasis_1: PaletteColor::EightBit(default_colors::CYAN),
        emphasis_2: PaletteColor::EightBit(default_colors::GREEN),
        emphasis_3: PaletteColor::EightBit(default_colors::MAGENTA),
        background: PaletteColor::EightBit(default_colors::GRAY),
    },
    ribbon_unselected: StyleDeclaration {
        base: PaletteColor::EightBit(default_colors::BLACK),
        emphasis_0: PaletteColor::EightBit(default_colors::RED),
        emphasis_1: PaletteColor::EightBit(default_colors::WHITE),
        emphasis_2: PaletteColor::EightBit(default_colors::BLUE),
        emphasis_3: PaletteColor::EightBit(default_colors::MAGENTA),
        background: PaletteColor::EightBit(default_colors::GRAY),
    },
    ribbon_selected: StyleDeclaration {
        base: PaletteColor::EightBit(default_colors::BLACK),
        emphasis_0: PaletteColor::EightBit(default_colors::RED),
        emphasis_1: PaletteColor::EightBit(default_colors::ORANGE),
        emphasis_2: PaletteColor::EightBit(default_colors::MAGENTA),
        emphasis_3: PaletteColor::EightBit(default_colors::BLUE),
        background: PaletteColor::EightBit(default_colors::GREEN),
    },
    exit_code_success: StyleDeclaration {
        base: PaletteColor::EightBit(default_colors::GREEN),
        emphasis_0: PaletteColor::EightBit(default_colors::CYAN),
        emphasis_1: PaletteColor::EightBit(default_colors::BLACK),
        emphasis_2: PaletteColor::EightBit(default_colors::MAGENTA),
        emphasis_3: PaletteColor::EightBit(default_colors::BLUE),
        background: PaletteColor::EightBit(default_colors::GRAY),
    },
    exit_code_error: StyleDeclaration {
        base: PaletteColor::EightBit(default_colors::RED),
        emphasis_0: PaletteColor::EightBit(default_colors::YELLOW),
        emphasis_1: PaletteColor::EightBit(default_colors::GOLD),
        emphasis_2: PaletteColor::EightBit(default_colors::SILVER),
        emphasis_3: PaletteColor::EightBit(default_colors::PURPLE),
        background: PaletteColor::EightBit(default_colors::GRAY),
    },
    frame_unselected: None,
    frame_selected: StyleDeclaration {
        base: PaletteColor::EightBit(default_colors::GREEN),
        emphasis_0: PaletteColor::EightBit(default_colors::ORANGE),
        emphasis_1: PaletteColor::EightBit(default_colors::CYAN),
        emphasis_2: PaletteColor::EightBit(default_colors::MAGENTA),
        emphasis_3: PaletteColor::EightBit(default_colors::BROWN),
        background: PaletteColor::EightBit(default_colors::GRAY),
    },
    frame_highlight: StyleDeclaration {
        base: PaletteColor::EightBit(default_colors::ORANGE),
        emphasis_0: PaletteColor::EightBit(default_colors::MAGENTA),
        emphasis_1: PaletteColor::EightBit(default_colors::PURPLE),
        emphasis_2: PaletteColor::EightBit(default_colors::GREEN),
        emphasis_3: PaletteColor::EightBit(default_colors::GREEN),
        background: PaletteColor::EightBit(default_colors::GREEN),
    },
    table_title: StyleDeclaration {
        base: PaletteColor::EightBit(default_colors::GREEN),
        emphasis_0: PaletteColor::EightBit(default_colors::ORANGE),
        emphasis_1: PaletteColor::EightBit(default_colors::CYAN),
        emphasis_2: PaletteColor::EightBit(default_colors::GREEN),
        emphasis_3: PaletteColor::EightBit(default_colors::MAGENTA),
        background: PaletteColor::EightBit(default_colors::GRAY),
    },
    table_cell_unselected: StyleDeclaration {
        base: PaletteColor::EightBit(default_colors::BRIGHT_GRAY),
        emphasis_0: PaletteColor::EightBit(default_colors::ORANGE),
        emphasis_1: PaletteColor::EightBit(default_colors::CYAN),
        emphasis_2: PaletteColor::EightBit(default_colors::GREEN),
        emphasis_3: PaletteColor::EightBit(default_colors::MAGENTA),
        background: PaletteColor::EightBit(default_colors::GRAY),
    },
    table_cell_selected: StyleDeclaration {
        base: PaletteColor::EightBit(default_colors::GREEN),
        emphasis_0: PaletteColor::EightBit(default_colors::ORANGE),
        emphasis_1: PaletteColor::EightBit(default_colors::CYAN),
        emphasis_2: PaletteColor::EightBit(default_colors::RED),
        emphasis_3: PaletteColor::EightBit(default_colors::MAGENTA),
        background: PaletteColor::EightBit(default_colors::GRAY),
    },
    list_unselected: StyleDeclaration {
        base: PaletteColor::EightBit(default_colors::BRIGHT_GRAY),
        emphasis_0: PaletteColor::EightBit(default_colors::ORANGE),
        emphasis_1: PaletteColor::EightBit(default_colors::CYAN),
        emphasis_2: PaletteColor::EightBit(default_colors::GREEN),
        emphasis_3: PaletteColor::EightBit(default_colors::MAGENTA),
        background: PaletteColor::EightBit(default_colors::GRAY),
    },
    list_selected: StyleDeclaration {
        base: PaletteColor::EightBit(default_colors::GREEN),
        emphasis_0: PaletteColor::EightBit(default_colors::ORANGE),
        emphasis_1: PaletteColor::EightBit(default_colors::CYAN),
        emphasis_2: PaletteColor::EightBit(default_colors::RED),
        emphasis_3: PaletteColor::EightBit(default_colors::MAGENTA),
        background: PaletteColor::EightBit(default_colors::GRAY),
    },
    multiplayer_user_colors: MultiplayerColors {
        player_1: PaletteColor::EightBit(default_colors::MAGENTA),
        player_2: PaletteColor::EightBit(default_colors::BLUE),
        player_3: PaletteColor::EightBit(default_colors::PURPLE),
        player_4: PaletteColor::EightBit(default_colors::YELLOW),
        player_5: PaletteColor::EightBit(default_colors::CYAN),
        player_6: PaletteColor::EightBit(default_colors::GOLD),
        player_7: PaletteColor::EightBit(default_colors::RED),
        player_8: PaletteColor::EightBit(default_colors::SILVER),
        player_9: PaletteColor::EightBit(default_colors::PINK),
        player_10: PaletteColor::EightBit(default_colors::BROWN),
    },
};

impl Default for Styling {
    fn default() -> Self {
        DEFAULT_STYLES
    }
}

impl From<Styling> for Palette {
    fn from(styling: Styling) -> Self {
        Palette {
            theme_hue: ThemeHue::Dark,
            source: PaletteSource::Default,
            fg: styling.ribbon_unselected.background,
            bg: styling.text_unselected.background,
            red: styling.exit_code_error.base,
            green: styling.text_unselected.emphasis_2,
            yellow: styling.exit_code_error.emphasis_0,
            blue: styling.ribbon_unselected.emphasis_2,
            magenta: styling.text_unselected.emphasis_3,
            orange: styling.text_unselected.emphasis_0,
            cyan: styling.text_unselected.emphasis_1,
            black: styling.ribbon_unselected.base,
            white: styling.ribbon_unselected.emphasis_1,
            gray: styling.list_unselected.background,
            purple: styling.multiplayer_user_colors.player_3,
            gold: styling.multiplayer_user_colors.player_6,
            silver: styling.multiplayer_user_colors.player_8,
            pink: styling.multiplayer_user_colors.player_9,
            brown: styling.multiplayer_user_colors.player_10,
        }
    }
}

impl From<Palette> for Styling {
    fn from(palette: Palette) -> Self {
        let (fg, bg) = match palette.theme_hue {
            ThemeHue::Light => (palette.black, palette.white),
            ThemeHue::Dark => (palette.white, palette.black),
        };
        Styling {
            text_unselected: StyleDeclaration {
                base: fg,
                emphasis_0: palette.orange,
                emphasis_1: palette.cyan,
                emphasis_2: palette.green,
                emphasis_3: palette.magenta,
                background: bg,
            },
            text_selected: StyleDeclaration {
                base: fg,
                emphasis_0: palette.orange,
                emphasis_1: palette.cyan,
                emphasis_2: palette.green,
                emphasis_3: palette.magenta,
                background: palette.bg,
            },
            ribbon_unselected: StyleDeclaration {
                base: palette.black,
                emphasis_0: palette.red,
                emphasis_1: palette.white,
                emphasis_2: palette.blue,
                emphasis_3: palette.magenta,
                background: palette.fg,
            },
            ribbon_selected: StyleDeclaration {
                base: palette.black,
                emphasis_0: palette.red,
                emphasis_1: palette.orange,
                emphasis_2: palette.magenta,
                emphasis_3: palette.blue,
                background: palette.green,
            },
            exit_code_success: StyleDeclaration {
                base: palette.green,
                emphasis_0: palette.cyan,
                emphasis_1: palette.black,
                emphasis_2: palette.magenta,
                emphasis_3: palette.blue,
                background: Default::default(),
            },
            exit_code_error: StyleDeclaration {
                base: palette.red,
                emphasis_0: palette.yellow,
                emphasis_1: palette.gold,
                emphasis_2: palette.silver,
                emphasis_3: palette.purple,
                background: Default::default(),
            },
            frame_unselected: None,
            frame_selected: StyleDeclaration {
                base: palette.green,
                emphasis_0: palette.orange,
                emphasis_1: palette.cyan,
                emphasis_2: palette.magenta,
                emphasis_3: palette.brown,
                background: Default::default(),
            },
            frame_highlight: StyleDeclaration {
                base: palette.orange,
                emphasis_0: palette.magenta,
                emphasis_1: palette.purple,
                emphasis_2: palette.orange,
                emphasis_3: palette.orange,
                background: Default::default(),
            },
            table_title: StyleDeclaration {
                base: palette.green,
                emphasis_0: palette.orange,
                emphasis_1: palette.cyan,
                emphasis_2: palette.green,
                emphasis_3: palette.magenta,
                background: palette.gray,
            },
            table_cell_unselected: StyleDeclaration {
                base: fg,
                emphasis_0: palette.orange,
                emphasis_1: palette.cyan,
                emphasis_2: palette.green,
                emphasis_3: palette.magenta,
                background: palette.black,
            },
            table_cell_selected: StyleDeclaration {
                base: fg,
                emphasis_0: palette.orange,
                emphasis_1: palette.cyan,
                emphasis_2: palette.green,
                emphasis_3: palette.magenta,
                background: palette.bg,
            },
            list_unselected: StyleDeclaration {
                base: palette.white,
                emphasis_0: palette.orange,
                emphasis_1: palette.cyan,
                emphasis_2: palette.green,
                emphasis_3: palette.magenta,
                background: palette.black,
            },
            list_selected: StyleDeclaration {
                base: palette.white,
                emphasis_0: palette.orange,
                emphasis_1: palette.cyan,
                emphasis_2: palette.green,
                emphasis_3: palette.magenta,
                background: palette.bg,
            },
            multiplayer_user_colors: MultiplayerColors {
                player_1: palette.magenta,
                player_2: palette.blue,
                player_3: palette.purple,
                player_4: palette.yellow,
                player_5: palette.cyan,
                player_6: palette.gold,
                player_7: palette.red,
                player_8: palette.silver,
                player_9: palette.pink,
                player_10: palette.brown,
            },
        }
    }
}

// FIXME: 可怜的开发者哈希表，因为 HashTable 无法派生 `Default`...
pub type KeybindsVec = Vec<(InputMode, Vec<(KeyWithModifier, Vec<Action>)>)>;

/// 提供有助于为 UI 栏渲染 Zellij 控件的信息
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModeInfo {
    pub mode: InputMode,
    pub base_mode: Option<InputMode>,
    pub keybinds: KeybindsVec,
    pub style: Style,
    pub capabilities: PluginCapabilities,
    pub session_name: Option<String>,
    pub editor: Option<PathBuf>,
    pub shell: Option<PathBuf>,
    pub web_clients_allowed: Option<bool>,
    pub web_sharing: Option<WebSharing>,
    pub currently_marking_pane_group: Option<bool>,
    pub is_web_client: Option<bool>,
    // 注意：这些只是配置的 ip/port，仅在服务端启动时才会绑定
    pub web_server_ip: Option<IpAddr>,
    pub web_server_port: Option<u16>,
    pub web_server_capability: Option<bool>,
    pub pane_frame_style: Option<PaneFrameStyle>,
    pub session_dimmed: Option<bool>,
    pub session_ancestry: Vec<String>,
    pub host_fullscreen: Option<bool>,
    pub nested_ascend_keys: Vec<KeyWithModifier>,
    pub session_ascended: Option<bool>,
    pub nested_descend_keys: Vec<KeyWithModifier>,
}

impl ModeInfo {
    pub fn get_mode_keybinds(&self) -> Vec<(KeyWithModifier, Vec<Action>)> {
        self.get_keybinds_for_mode(self.mode)
    }

    pub fn get_keybinds_for_mode(&self, mode: InputMode) -> Vec<(KeyWithModifier, Vec<Action>)> {
        for (vec_mode, map) in &self.keybinds {
            if mode == *vec_mode {
                return map.to_vec();
            }
        }
        vec![]
    }
    pub fn update_keybinds(&mut self, keybinds: Keybinds) {
        self.keybinds = keybinds.to_keybinds_vec();
    }
    pub fn update_default_mode(&mut self, new_default_mode: InputMode) {
        self.base_mode = Some(new_default_mode);
    }
    pub fn update_theme(&mut self, theme: Styling) {
        self.style.colors = theme.into();
    }
    pub fn update_rounded_corners(&mut self, rounded_corners: bool) {
        self.style.rounded_corners = rounded_corners;
    }
    pub fn update_arrow_fonts(&mut self, should_support_arrow_fonts: bool) {
        // 老实说，我很困惑 "arrow_fonts: false" 怎么能表示 "我支持箭头
        // 字体"，但既然这是一个公共 API... ¯\_(ツ)_/¯
        self.capabilities.arrow_fonts = !should_support_arrow_fonts;
    }
    pub fn update_hide_session_name(&mut self, hide_session_name: bool) {
        self.style.hide_session_name = hide_session_name;
    }
    pub fn change_to_default_mode(&mut self) {
        if let Some(base_mode) = self.base_mode {
            self.mode = base_mode;
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct SessionInfo {
    pub name: String,
    pub tabs: Vec<TabInfo>,
    pub panes: PaneManifest,
    pub connected_clients: usize,
    pub is_current_session: bool,
    pub available_layouts: Vec<LayoutInfo>,
    pub plugins: BTreeMap<u32, PluginInfo>,
    pub web_clients_allowed: bool,
    pub web_client_count: usize,
    pub tab_history: BTreeMap<ClientId, Vec<usize>>,
    pub pane_history: BTreeMap<ClientId, Vec<PaneId>>,
    pub creation_time: Duration,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct PluginInfo {
    pub location: String,
    pub configuration: BTreeMap<String, String>,
}

impl From<RunPlugin> for PluginInfo {
    fn from(run_plugin: RunPlugin) -> Self {
        PluginInfo {
            location: run_plugin.location.display(),
            configuration: run_plugin.configuration.inner().clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum LayoutInfo {
    BuiltIn(String),
    File(String, LayoutMetadata),
    Url(String),
    Stringified(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct LayoutWithError {
    pub layout_name: String,
    pub error: LayoutParsingError,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum LayoutParsingError {
    KdlError {
        kdl_error: KdlError,
        file_name: String,
        source_code: String,
    },
    SyntaxError,
}

impl AsRef<LayoutInfo> for LayoutInfo {
    fn as_ref(&self) -> &LayoutInfo {
        self
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct LayoutMetadata {
    pub tabs: Vec<TabMetadata>,
    pub creation_time: String,
    pub update_time: String,
}

impl From<&PathBuf> for LayoutMetadata {
    fn from(path: &PathBuf) -> LayoutMetadata {
        match Layout::stringified_from_path(path) {
            Ok((path_str, stringified_layout, _swap_layouts)) => {
                match Layout::from_kdl(&stringified_layout, Some(path_str), None, None) {
                    Ok(layout) => {
                        let layout_tabs = layout.tabs();
                        let tabs = if layout_tabs.is_empty() {
                            let (tiled_pane_layout, floating_pane_layout) = layout.new_tab();
                            vec![TabMetadata::from(&(
                                None,
                                tiled_pane_layout,
                                floating_pane_layout,
                            ))]
                        } else {
                            layout
                                .tabs()
                                .into_iter()
                                .map(|tab| TabMetadata::from(&tab))
                                .collect()
                        };

                        // 获取文件元数据以获取创建和修改时间（Unix 时间戳）
                        let (creation_time, update_time) =
                            LayoutMetadata::creation_and_update_times(&path);

                        LayoutMetadata {
                            tabs,
                            creation_time,
                            update_time,
                        }
                    },
                    Err(e) => {
                        log::error!("解析布局失败：{}", e);
                        LayoutMetadata::default()
                    },
                }
            },
            Err(e) => {
                log::error!("读取布局文件失败：{}", e);
                LayoutMetadata::default()
            },
        }
    }
}

impl LayoutMetadata {
    fn creation_and_update_times(path: &PathBuf) -> (String, String) {
        //（creation_time, update_time）返回字符串化的 unix 时间戳
        match std::fs::metadata(path) {
            Ok(metadata) => {
                let creation_time = metadata
                    .created()
                    .ok()
                    .and_then(|t| {
                        t.duration_since(std::time::UNIX_EPOCH)
                            .ok()
                            .map(|d| d.as_secs().to_string())
                    })
                    .unwrap_or_default();

                let update_time = metadata
                    .modified()
                    .ok()
                    .and_then(|t| {
                        t.duration_since(std::time::UNIX_EPOCH)
                            .ok()
                            .map(|d| d.as_secs().to_string())
                    })
                    .unwrap_or_default();

                (creation_time, update_time)
            },
            Err(_) => (String::new(), String::new()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct TabMetadata {
    pub panes: Vec<PaneMetadata>,
    pub name: Option<String>,
}

impl
    From<&(
        Option<String>,
        crate::input::layout::TiledPaneLayout,
        Vec<crate::input::layout::FloatingPaneLayout>,
    )> for TabMetadata
{
    fn from(
        tab: &(
            Option<String>,
            crate::input::layout::TiledPaneLayout,
            Vec<crate::input::layout::FloatingPaneLayout>,
        ),
    ) -> Self {
        let (tab_name, tiled_pane_layout, floating_panes) = tab;

        // 从平铺布局中收集窗格（只有叶节点是真实窗格）
        let mut panes = Vec::new();
        collect_leaf_panes(&tiled_pane_layout, &mut panes);

        // 从浮动窗格中收集窗格
        for floating_pane in floating_panes {
            panes.push(PaneMetadata::from(floating_pane));
        }

        TabMetadata {
            panes,
            name: tab_name.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct PaneMetadata {
    pub name: Option<String>,
    pub is_plugin: bool,
    pub is_builtin_plugin: bool,
}

impl From<&crate::input::layout::TiledPaneLayout> for PaneMetadata {
    fn from(pane: &crate::input::layout::TiledPaneLayout) -> Self {
        let mut is_plugin = false;
        let mut is_builtin_plugin = false;

        // 首先尝试从窗格的 name 字段获取名称
        let name = if let Some(ref name) = pane.name {
            Some(name.clone())
        } else if let Some(ref run) = pane.run {
            // 如果没有显式名称，则从运行配置中提取
            match run {
                Run::Command(cmd) => {
                    // 使用命令名称
                    Some(cmd.command.to_string_lossy().to_string())
                },
                Run::EditFile(path, _line, _cwd) => {
                    // 使用文件名
                    path.file_name().map(|n| n.to_string_lossy().to_string())
                },
                Run::Plugin(plugin) => {
                    is_plugin = true;
                    is_builtin_plugin = plugin.is_builtin_plugin();
                    Some(plugin.location_string())
                },
                Run::Cwd(_) => None,
            }
        } else {
            None
        };

        PaneMetadata {
            name,
            is_plugin,
            is_builtin_plugin,
        }
    }
}

impl From<&crate::input::layout::FloatingPaneLayout> for PaneMetadata {
    fn from(pane: &crate::input::layout::FloatingPaneLayout) -> Self {
        let mut is_plugin = false;
        let mut is_builtin_plugin = false;

        // 首先尝试从窗格的 name 字段获取名称
        let name = if let Some(ref name) = pane.name {
            Some(name.clone())
        } else if let Some(ref run) = pane.run {
            // 如果没有显式名称，则从运行配置中提取
            match run {
                Run::Command(cmd) => {
                    // 使用命令名称
                    Some(cmd.command.to_string_lossy().to_string())
                },
                Run::EditFile(path, _line, _cwd) => {
                    // 使用文件名
                    path.file_name().map(|n| n.to_string_lossy().to_string())
                },
                Run::Plugin(plugin) => {
                    is_plugin = true;
                    is_builtin_plugin = match plugin {
                        crate::input::layout::RunPluginOrAlias::RunPlugin(run_plugin) => {
                            matches!(run_plugin.location, RunPluginLocation::Zellij(_))
                        },
                        crate::input::layout::RunPluginOrAlias::Alias(_) => false,
                    };
                    // 使用插件位置字符串
                    Some(plugin.location_string())
                },
                Run::Cwd(_) => None,
            }
        } else {
            None
        };

        PaneMetadata {
            name,
            is_plugin,
            is_builtin_plugin,
        }
    }
}

// 从 TiledPaneLayout 递归收集叶窗格的辅助函数
fn collect_leaf_panes(
    pane: &crate::input::layout::TiledPaneLayout,
    result: &mut Vec<PaneMetadata>,
) {
    if pane.children.is_empty() {
        // 这是一个叶节点（实际窗格）
        result.push(PaneMetadata::from(pane));
    } else {
        // 这是一个容器，递归进入子节点
        for child in &pane.children {
            collect_leaf_panes(child, result);
        }
    }
}

impl LayoutInfo {
    pub fn name(&self) -> &str {
        match self {
            LayoutInfo::BuiltIn(name) => &name,
            LayoutInfo::File(name, _) => &name,
            LayoutInfo::Url(url) => &url,
            LayoutInfo::Stringified(layout) => &layout,
        }
    }
    pub fn is_builtin(&self) -> bool {
        match self {
            LayoutInfo::BuiltIn(_name) => true,
            LayoutInfo::File(_name, _) => false,
            LayoutInfo::Url(_url) => false,
            LayoutInfo::Stringified(_stringified) => false,
        }
    }
    pub fn from_cli(
        layout_dir: &Option<PathBuf>,
        maybe_layout_path: &Option<PathBuf>,
        cwd: PathBuf,
    ) -> Option<Self> {
        // 如果没有给定布局路径，则回退到 "default"。由于我们无法提前
        // 判断用户的布局目录中是否有名为 "default.kdl" 的布局，我们
        // 不能盲目假设这确实是内置默认布局。布局
        // 下面的解析将正确处理这种情况。
        // 文档承诺了此行为，因此我们必须遵守：
        // <https://zellij.dev/documentation/layouts.html#layout-default-directory>
        let layout_path = maybe_layout_path
            .clone()
            .unwrap_or(PathBuf::from("default"));

        if layout_path.starts_with("http://") || layout_path.starts_with("https://") {
            Some(LayoutInfo::Url(layout_path.display().to_string()))
        } else if layout_path.extension().is_some() || layout_path.components().count() > 1 {
            let layout_dir = cwd;
            let file_path = layout_dir.join(layout_path);
            Some(LayoutInfo::File(
                // layout_dir.join(layout_path).display().to_string(),
                file_path.display().to_string(),
                LayoutMetadata::from(&file_path),
            ))
        } else {
            // 尝试将布局解释为布局应用程序目录中的裸布局名称
            // 目录。这在文档中有描述：
            // <https://zellij.dev/documentation/layouts.html#layout-default-directory>
            if let Some(layout_dir) = layout_dir
                .as_ref()
                .map(|l| l.clone())
                .or_else(default_layout_dir)
            {
                let file_path = layout_dir.join(&layout_path);
                if file_path.exists() {
                    return Some(LayoutInfo::File(
                        file_path.display().to_string(),
                        LayoutMetadata::from(&file_path),
                    ));
                }
                let file_path_with_ext = file_path.with_extension("kdl");
                if file_path_with_ext.exists() {
                    return Some(LayoutInfo::File(
                        file_path_with_ext.display().to_string(),
                        LayoutMetadata::from(&file_path_with_ext),
                    ));
                }
            }
            // 默认假设为内置布局
            Some(LayoutInfo::BuiltIn(layout_path.display().to_string()))
        }
    }
    pub fn from_config(
        layout_dir: &Option<PathBuf>,
        maybe_layout_path: &Option<PathBuf>,
    ) -> Option<Self> {
        // 如果没有给定布局路径，则回退到 "default"。由于我们无法提前
        // 判断用户的布局目录中是否有名为 "default.kdl" 的布局，我们
        // 不能盲目假设这确实是内置默认布局。布局
        // 下面的解析将正确处理这种情况。
        // 文档承诺了此行为，因此我们必须遵守：
        // <https://zellij.dev/documentation/layouts.html#layout-default-directory>
        let layout_path = maybe_layout_path
            .clone()
            .unwrap_or(PathBuf::from("default"));

        if layout_path.starts_with("http://") || layout_path.starts_with("https://") {
            Some(LayoutInfo::Url(layout_path.display().to_string()))
        } else if layout_path.extension().is_some() || layout_path.components().count() > 1 {
            let Some(layout_dir) = layout_dir
                .as_ref()
                .map(|l| l.clone())
                .or_else(default_layout_dir)
            else {
                return None;
            };
            let file_path = layout_dir.join(layout_path);
            Some(LayoutInfo::File(
                // layout_dir.join(layout_path).display().to_string(),
                file_path.display().to_string(),
                LayoutMetadata::from(&file_path),
            ))
        } else {
            // 尝试将布局解释为布局应用程序目录中的裸布局名称
            // 目录。这在文档中有描述：
            // <https://zellij.dev/documentation/layouts.html#layout-default-directory>
            if let Some(layout_dir) = layout_dir
                .as_ref()
                .map(|l| l.clone())
                .or_else(default_layout_dir)
            {
                let file_path = layout_dir.join(&layout_path);
                if file_path.exists() {
                    return Some(LayoutInfo::File(
                        file_path.display().to_string(),
                        LayoutMetadata::from(&file_path),
                    ));
                }
                let file_path_with_ext = file_path.with_extension("kdl");
                if file_path_with_ext.exists() {
                    return Some(LayoutInfo::File(
                        file_path_with_ext.display().to_string(),
                        LayoutMetadata::from(&file_path_with_ext),
                    ));
                }
            }
            // 默认假设为内置布局
            Some(LayoutInfo::BuiltIn(layout_path.display().to_string()))
        }
    }
}

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for SessionInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl SessionInfo {
    pub fn new(name: String) -> Self {
        SessionInfo {
            name,
            ..Default::default()
        }
    }
    pub fn update_tab_info(&mut self, new_tab_info: Vec<TabInfo>) {
        self.tabs = new_tab_info;
    }
    pub fn update_pane_info(&mut self, new_pane_info: PaneManifest) {
        self.panes = new_pane_info;
    }
    pub fn update_connected_clients(&mut self, new_connected_clients: usize) {
        self.connected_clients = new_connected_clients;
    }
    pub fn populate_plugin_list(&mut self, plugins: BTreeMap<u32, RunPlugin>) {
        // u32 - plugin_id
        let mut plugin_list = BTreeMap::new();
        for (plugin_id, run_plugin) in plugins {
            plugin_list.insert(plugin_id, run_plugin.into());
        }
        self.plugins = plugin_list;
    }
}

/// 包含当前打开的标签页的所有信息。
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct TabInfo {
    /// 标签页的 0 索引位置
    pub position: usize,
    /// 标签页在界面中显示的名称（如果有足够空间）
    pub name: String,
    /// 此标签页是否被聚焦
    pub active: bool,
    /// 此标签页拥有的被抑制窗格数量
    pub panes_to_hide: usize,
    /// 此标签页上是否有一个窗格占据整个显示区域
    pub is_fullscreen_active: bool,
    /// 发送到此标签页的输入是否将同步到其中的所有窗格
    pub is_sync_panes_active: bool,
    pub are_floating_panes_visible: bool,
    pub other_focused_clients: Vec<ClientId>,
    pub active_swap_layout_name: Option<String>,
    /// 用户是否手动更改了布局，从而退出交换布局方案
    pub is_swap_layout_dirty: bool,
    /// 视口中的行数（包括所有非 UI 窗格，例如将排除状态栏）
    pub viewport_rows: usize,
    /// 视口中的列数（包括所有非 UI 窗格，例如将排除状态栏）
    pub viewport_columns: usize,
    /// 显示区域中的行数（包括所有窗格，通常大于视口）
    /// 视口）
    pub display_area_rows: usize,
    /// 显示区域中的列数（包括所有窗格，通常大于视口）
    /// 视口）
    pub display_area_columns: usize,
    /// 当前此标签页中可选择的（例如非 UI 栏）平铺窗格数量
    pub selectable_tiled_panes_count: usize,
    /// 当前此标签页中可选择的（例如非 UI 栏）浮动窗格数量
    pub selectable_floating_panes_count: usize,
    /// 此标签页的稳定标识符
    pub tab_id: usize,
    /// 此标签页是否有活动的（持久的）铃声通知
    pub has_bell_notification: bool,
    /// 此标签页当前是否正在闪烁铃声（瞬态 400ms 状态）
    pub is_flashing_bell: bool,
}

/// `PaneManifest` 包含一个窗格字典，按标签页位置（0 索引）索引。
/// 窗格包括相关标签页中的所有窗格，包括 `tiled` 窗格、`floating` 窗格和
/// `suppressed` 窗格。
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct PaneManifest {
    pub panes: HashMap<usize, Vec<PaneInfo>>, // usize 是标签页位置
}

/// 包含当前打开的窗格的所有信息
///
/// # 坐标/大小与内容坐标/大小之间的区别
///
/// 窗格的基本坐标和大小（例如 `pane_x` 或 `pane_columns`）是此窗格占据的整个空间
/// — 如果有边框，包括其框架和标题。
///
/// 窗格内容的坐标和大小（例如 `pane_content_x` 或 `pane_content_columns`）
/// 表示窗格内容占据的区域，如果有边框，则不包括其框架和标题。
/// 边框。
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct PaneInfo {
    /// 窗格的 id，对此类型的所有窗格唯一（例如终端中的 id 或窗格中的 id）
    pub id: u32,
    /// 此窗格是插件（`true`）还是终端（`false`），与 `id` 一起使用可以表示跨运行会话的唯一窗格 ID
    /// 运行中的会话
    pub is_plugin: bool,
    /// 窗格在其层（平铺或浮动）中是否被聚焦
    pub is_focused: bool,
    pub is_fullscreen: bool,
    /// 窗格是浮动的还是平铺的（嵌入的）
    pub is_floating: bool,
    /// 窗格是否被抑制 — 被抑制的窗格对用户不可见，但仍在后台运行
    /// 在后台
    pub is_suppressed: bool,
    /// 窗格在界面中显示的完整标题（如果有足够空间）
    pub title: String,
    /// 窗格是否已退出，请注意大多数窗格在设置此标志之前会自行关闭，
    /// 因此这仅与命令窗格相关
    pub exited: bool,
    /// 如果窗格已退出且仍在界面中，则为其退出状态
    pub exit_status: Option<i32>,
    /// "held" 窗格是一个暂停的窗格，正在等待用户输入（例如已退出并等待重新运行或关闭的命令窗格）
    /// 已退出并等待重新运行或关闭）
    pub is_held: bool,
    pub pane_x: usize,
    pub pane_content_x: usize,
    pub pane_y: usize,
    pub pane_content_y: usize,
    pub pane_rows: usize,
    pub pane_content_rows: usize,
    pub pane_columns: usize,
    pub pane_content_columns: usize,
    /// 光标的坐标 — 如果此窗格被聚焦 — 相对于窗格的坐标
    /// 坐标
    pub cursor_coordinates_in_pane: Option<(usize, usize)>, // x, y 如果光标可见
    /// 如果这是命令窗格，这将显示命令及其参数的字符串化版本
    /// 参数
    pub terminal_command: Option<String>,
    /// 加载此插件的 URL（例如内置 `strider` 插件的 `zellij:strider`
    /// 或本地插件的 `file:/path/to/my/plugin.wasm`）
    pub plugin_url: Option<String>,
    /// 不可选择的窗格通常用于没有直接用户交互的 UI 元素
    ///（例如默认的 `status-bar` 或 `tab-bar`）。
    pub is_selectable: bool,
    /// 已分组的窗格（通常通过显式用户操作），已暂存以执行批量操作
    /// 跟踪索引以保持窗格组的顺序
    pub index_in_pane_group: BTreeMap<ClientId, usize>,
    /// 此窗格的默认前景色（如果已设置）（例如 "#00e000"）
    pub default_fg: Option<String>,
    /// 此窗格的默认背景色（如果已设置）（例如 "#001a3a"）
    pub default_bg: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct PaneListEntry {
    #[serde(flatten)]
    pub pane_info: PaneInfo,
    pub tab_id: usize,
    pub tab_position: usize,
    pub tab_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pane_command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pane_cwd: Option<String>,
}

pub type ListPanesResponse = Vec<PaneListEntry>;
pub type ListTabsResponse = Vec<TabInfo>;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ClientInfo {
    pub client_id: ClientId,
    pub pane_id: PaneId,
    pub running_command: String,
    pub is_current_client: bool,
}

impl ClientInfo {
    pub fn new(
        client_id: ClientId,
        pane_id: PaneId,
        running_command: String,
        is_current_client: bool,
    ) -> Self {
        ClientInfo {
            client_id,
            pane_id,
            running_command,
            is_current_client,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PaneRenderReport {
    pub all_pane_contents: HashMap<ClientId, HashMap<PaneId, PaneContents>>,
    pub all_pane_contents_with_ansi: HashMap<ClientId, HashMap<PaneId, PaneContents>>,
}

impl PaneRenderReport {
    pub fn add_pane_contents(
        &mut self,
        client_ids: &[ClientId],
        pane_id: PaneId,
        pane_contents: PaneContents,
    ) {
        for client_id in client_ids {
            let p = self
                .all_pane_contents
                .entry(*client_id)
                .or_insert_with(|| HashMap::new());
            p.insert(pane_id, pane_contents.clone());
        }
    }
    pub fn add_pane_contents_with_ansi(
        &mut self,
        client_ids: &[ClientId],
        pane_id: PaneId,
        pane_contents: PaneContents,
    ) {
        for client_id in client_ids {
            let p = self
                .all_pane_contents_with_ansi
                .entry(*client_id)
                .or_insert_with(|| HashMap::new());
            p.insert(pane_id, pane_contents.clone());
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PaneContents {
    // 注意：lines_above_viewport 和 lines_below_viewport 仅在显式请求时填充
    //（例如在插件命令中设置 get_full_scrollback 为 true），这是出于性能原因
    // 原因
    pub lines_above_viewport: Vec<String>,
    pub lines_below_viewport: Vec<String>,
    pub viewport: Vec<String>,
    pub selected_text: Option<SelectedText>,
    pub cursor: Option<(usize, usize)>,
}

/// 从一行中两个列位置之间提取文本，考虑宽字符
fn extract_text_by_columns(line: &str, start_col: usize, end_col: usize) -> String {
    let mut current_col = 0;
    let mut result = String::new();
    let mut capturing = false;

    for ch in line.chars() {
        let char_width = ch.width().unwrap_or(0);

        // 到达 start_col 时开始捕获
        if current_col >= start_col && !capturing {
            capturing = true;
        }

        // 如果已到达或超过 end_col 则停止
        if current_col >= end_col {
            break;
        }

        // 如果在范围内则捕获字符
        if capturing {
            result.push(ch);
        }

        current_col += char_width;
    }

    result
}

/// 从一行中从某个列位置开始提取文本，考虑宽字符
fn extract_text_from_column(line: &str, start_col: usize) -> String {
    let mut current_col = 0;
    let mut result = String::new();
    let mut capturing = false;

    for ch in line.chars() {
        let char_width = ch.width().unwrap_or(0);

        if current_col >= start_col {
            capturing = true;
        }

        if capturing {
            result.push(ch);
        }

        current_col += char_width;
    }

    result
}

/// 从一行中提取到某个列位置为止的文本，考虑宽字符
fn extract_text_to_column(line: &str, end_col: usize) -> String {
    let mut current_col = 0;
    let mut result = String::new();

    for ch in line.chars() {
        let char_width = ch.width().unwrap_or(0);

        if current_col >= end_col {
            break;
        }

        result.push(ch);
        current_col += char_width;
    }

    result
}

impl PaneContents {
    pub fn new(viewport: Vec<String>, selection_start: Position, selection_end: Position) -> Self {
        PaneContents {
            viewport,
            selected_text: SelectedText::from_positions(selection_start, selection_end),
            ..Default::default()
        }
    }
    pub fn new_with_scrollback(
        viewport: Vec<String>,
        selection_start: Position,
        selection_end: Position,
        lines_above_viewport: Vec<String>,
        lines_below_viewport: Vec<String>,
    ) -> Self {
        PaneContents {
            viewport,
            selected_text: SelectedText::from_positions(selection_start, selection_end),
            lines_above_viewport,
            lines_below_viewport,
            cursor: None,
        }
    }

    /// 返回选区的实际文本内容（如果存在）。
    /// 选区仅在视口内发生。
    pub fn get_selected_text(&self) -> Option<String> {
        let selected_text = self.selected_text?;

        let start_line = selected_text.start.line() as usize;
        let start_col = selected_text.start.column();
        let end_line = selected_text.end.line() as usize;
        let end_col = selected_text.end.column();

        // 处理越界
        if start_line >= self.viewport.len() || end_line >= self.viewport.len() {
            return None;
        }

        if start_line == end_line {
            // 单行选择
            let line = &self.viewport[start_line];
            Some(extract_text_by_columns(line, start_col, end_col))
        } else {
            // 多行选择
            let mut result = String::new();

            // 第一行 — 从起始列到行尾
            let first_line = &self.viewport[start_line];
            result.push_str(&extract_text_from_column(first_line, start_col));
            result.push('\n');

            // 中间行 — 完整行
            for i in (start_line + 1)..end_line {
                result.push_str(&self.viewport[i]);
                result.push('\n');
            }

            // 最后一行 — 从起始列到结束列
            let last_line = &self.viewport[end_line];
            result.push_str(&extract_text_to_column(last_line, end_col));

            Some(result)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaneScrollbackResponse {
    Ok(PaneContents),
    Err(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GetPanePidResponse {
    Ok(i32),
    Err(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GetPaneRunningCommandResponse {
    Ok(Vec<String>),
    Err(String),
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionListSnapshot {
    pub live_sessions: Vec<SessionInfo>,
    pub resurrectable_sessions: Vec<(String, Duration)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GetSessionListResponse {
    Ok(SessionListSnapshot),
    Err(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KillSessionsResponse {
    Ok,
    Err(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeleteDeadSessionResponse {
    Ok,
    Err(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeleteAllDeadSessionsResponse {
    Ok,
    Err(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GetPaneCwdResponse {
    Ok(PathBuf),
    Err(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum GetFocusedPaneInfoResponse {
    Ok { tab_index: usize, pane_id: PaneId },
    Err(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SaveLayoutResponse {
    Ok(()),
    Err(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeleteLayoutResponse {
    Ok(()),
    Err(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenameLayoutResponse {
    Ok(()),
    Err(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditLayoutResponse {
    Ok(()),
    Err(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectedText {
    pub start: Position,
    pub end: Position,
}

impl SelectedText {
    pub fn new(start: Position, end: Position) -> Self {
        // 规范化：确保 start <= end
        let (normalized_start, normalized_end) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };

        // 将负行值规范化为 0
        //（column 已经是 usize，所以不能为负）
        let normalized_start = Position::new(
            normalized_start.line().max(0) as i32,
            normalized_start.column() as u16,
        );
        let normalized_end = Position::new(
            normalized_end.line().max(0) as i32,
            normalized_end.column() as u16,
        );

        SelectedText {
            start: normalized_start,
            end: normalized_end,
        }
    }

    pub fn from_positions(start: Position, end: Position) -> Option<Self> {
        if start == end {
            None
        } else {
            Some(Self::new(start, end))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct PluginIds {
    pub plugin_id: u32,
    pub zellij_pid: u32,
    pub initial_cwd: PathBuf,
    pub client_id: ClientId,
}

/// 用于在布局和配置 KDL 文件中标识插件的标签
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct PluginTag(String);

impl PluginTag {
    pub fn new(url: impl Into<String>) -> Self {
        PluginTag(url.into())
    }
}

impl From<PluginTag> for String {
    fn from(tag: PluginTag) -> Self {
        tag.0
    }
}

impl fmt::Display for PluginTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct PluginCapabilities {
    pub arrow_fonts: bool,
}

impl Default for PluginCapabilities {
    fn default() -> PluginCapabilities {
        PluginCapabilities { arrow_fonts: true }
    }
}

/// 表示剪贴板类型
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum CopyDestination {
    Command,
    Primary,
    System,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum PermissionStatus {
    Granted,
    Denied,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileToOpen {
    pub path: PathBuf,
    pub line_number: Option<usize>,
    pub cwd: Option<PathBuf>,
}

impl FileToOpen {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        FileToOpen {
            path: path.as_ref().to_path_buf(),
            ..Default::default()
        }
    }
    pub fn with_line_number(mut self, line_number: usize) -> Self {
        self.line_number = Some(line_number);
        self
    }
    pub fn with_cwd(mut self, cwd: PathBuf) -> Self {
        self.cwd = Some(cwd);
        self
    }
}

#[derive(Debug, Default, Clone)]
pub struct CommandToRun {
    pub path: PathBuf,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
}

impl CommandToRun {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        CommandToRun {
            path: path.as_ref().to_path_buf(),
            ..Default::default()
        }
    }
    pub fn new_with_args<P: AsRef<Path>, A: AsRef<str>>(path: P, args: Vec<A>) -> Self {
        CommandToRun {
            path: path.as_ref().to_path_buf(),
            args: args.into_iter().map(|a| a.as_ref().to_owned()).collect(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct MessageToPlugin {
    pub plugin_url: Option<String>,
    pub destination_plugin_id: Option<u32>,
    pub plugin_config: BTreeMap<String, String>,
    pub message_name: String,
    pub message_payload: Option<String>,
    pub message_args: BTreeMap<String, String>,
    /// 这些仅在需要启动新插件来发送此消息时使用，
    /// 因为没有正在运行的插件
    pub new_plugin_args: Option<NewPluginArgs>,
    pub floating_pane_coordinates: Option<FloatingPaneCoordinates>,
}

#[derive(Debug, Default, Clone)]
pub struct NewPluginArgs {
    pub should_float: Option<bool>,
    pub pane_id_to_replace: Option<PaneId>,
    pub pane_title: Option<String>,
    pub cwd: Option<PathBuf>,
    pub skip_cache: bool,
    pub should_focus: Option<bool>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PaneId {
    Terminal(u32),
    Plugin(u32),
}

impl Default for PaneId {
    fn default() -> Self {
        PaneId::Terminal(0)
    }
}

impl FromStr for PaneId {
    type Err = Box<dyn std::error::Error>;
    fn from_str(stringified_pane_id: &str) -> Result<Self, Self::Err> {
        if let Some(terminal_stringified_pane_id) = stringified_pane_id.strip_prefix("terminal_") {
            u32::from_str_radix(terminal_stringified_pane_id, 10)
                .map(|id| PaneId::Terminal(id))
                .map_err(|e| e.into())
        } else if let Some(plugin_pane_id) = stringified_pane_id.strip_prefix("plugin_") {
            u32::from_str_radix(plugin_pane_id, 10)
                .map(|id| PaneId::Plugin(id))
                .map_err(|e| e.into())
        } else {
            u32::from_str_radix(&stringified_pane_id, 10)
                .map(|id| PaneId::Terminal(id))
                .map_err(|e| e.into())
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

impl MessageToPlugin {
    pub fn new(message_name: impl Into<String>) -> Self {
        MessageToPlugin {
            message_name: message_name.into(),
            ..Default::default()
        }
    }
    pub fn with_plugin_url(mut self, url: impl Into<String>) -> Self {
        self.plugin_url = Some(url.into());
        self
    }
    pub fn with_destination_plugin_id(mut self, destination_plugin_id: u32) -> Self {
        self.destination_plugin_id = Some(destination_plugin_id);
        self
    }
    pub fn with_plugin_config(mut self, plugin_config: BTreeMap<String, String>) -> Self {
        self.plugin_config = plugin_config;
        self
    }
    pub fn with_payload(mut self, payload: impl Into<String>) -> Self {
        self.message_payload = Some(payload.into());
        self
    }
    pub fn with_args(mut self, args: BTreeMap<String, String>) -> Self {
        self.message_args = args;
        self
    }
    pub fn with_floating_pane_coordinates(
        mut self,
        floating_pane_coordinates: FloatingPaneCoordinates,
    ) -> Self {
        self.floating_pane_coordinates = Some(floating_pane_coordinates);
        self
    }
    pub fn new_plugin_instance_should_float(mut self, should_float: bool) -> Self {
        let new_plugin_args = self.new_plugin_args.get_or_insert_with(Default::default);
        new_plugin_args.should_float = Some(should_float);
        self
    }
    pub fn new_plugin_instance_should_replace_pane(mut self, pane_id: PaneId) -> Self {
        let new_plugin_args = self.new_plugin_args.get_or_insert_with(Default::default);
        new_plugin_args.pane_id_to_replace = Some(pane_id);
        self
    }
    pub fn new_plugin_instance_should_have_pane_title(
        mut self,
        pane_title: impl Into<String>,
    ) -> Self {
        let new_plugin_args = self.new_plugin_args.get_or_insert_with(Default::default);
        new_plugin_args.pane_title = Some(pane_title.into());
        self
    }
    pub fn new_plugin_instance_should_have_cwd(mut self, cwd: PathBuf) -> Self {
        let new_plugin_args = self.new_plugin_args.get_or_insert_with(Default::default);
        new_plugin_args.cwd = Some(cwd);
        self
    }
    pub fn new_plugin_instance_should_skip_cache(mut self) -> Self {
        let new_plugin_args = self.new_plugin_args.get_or_insert_with(Default::default);
        new_plugin_args.skip_cache = true;
        self
    }
    pub fn new_plugin_instance_should_be_focused(mut self) -> Self {
        let new_plugin_args = self.new_plugin_args.get_or_insert_with(Default::default);
        new_plugin_args.should_focus = Some(true);
        self
    }
    pub fn has_cwd(&self) -> bool {
        self.new_plugin_args
            .as_ref()
            .map(|n| n.cwd.is_some())
            .unwrap_or(false)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConnectToSession {
    pub name: Option<String>,
    pub tab_position: Option<usize>,
    pub pane_id: Option<(u32, bool)>, //（id, is_plugin）
    pub layout: Option<LayoutInfo>,
    pub cwd: Option<PathBuf>,
}

impl ConnectToSession {
    pub fn apply_layout_dir(&mut self, layout_dir: &PathBuf) {
        if let Some(LayoutInfo::File(file_path, _layout_metadata)) = self.layout.as_mut() {
            *file_path = Path::join(layout_dir, &file_path)
                .to_string_lossy()
                .to_string();
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct PluginMessage {
    pub name: String,
    pub payload: String,
    pub worker_name: Option<String>,
}

impl PluginMessage {
    pub fn new_to_worker(worker_name: &str, message: &str, payload: &str) -> Self {
        PluginMessage {
            name: message.to_owned(),
            payload: payload.to_owned(),
            worker_name: Some(worker_name.to_owned()),
        }
    }
    pub fn new_to_plugin(message: &str, payload: &str) -> Self {
        PluginMessage {
            name: message.to_owned(),
            payload: payload.to_owned(),
            worker_name: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpVerb {
    Get,
    Post,
    Put,
    Delete,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PipeSource {
    Cli(String), // String 是 CLI 管道的 pipe_id（用于阻塞/解除阻塞）
    Plugin(u32), // u32 是插件 id
    Keybind,     // TODO: 考虑在这里包含实际的快捷键绑定？
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PipeMessage {
    pub source: PipeSource,
    pub name: String,
    pub payload: Option<String>,
    pub args: BTreeMap<String, String>,
    pub is_private: bool,
}

impl PipeMessage {
    pub fn new(
        source: PipeSource,
        name: impl Into<String>,
        payload: &Option<String>,
        args: &Option<BTreeMap<String, String>>,
        is_private: bool,
    ) -> Self {
        PipeMessage {
            source,
            name: name.into(),
            payload: payload.clone(),
            args: args.clone().unwrap_or_else(|| Default::default()),
            is_private,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct FloatingPaneCoordinates {
    pub x: Option<PercentOrFixed>,
    pub y: Option<PercentOrFixed>,
    pub width: Option<PercentOrFixed>,
    pub height: Option<PercentOrFixed>,
    pub pinned: Option<bool>,
    pub borderless: Option<bool>,
}

impl FloatingPaneCoordinates {
    pub fn new(
        x: Option<String>,
        y: Option<String>,
        width: Option<String>,
        height: Option<String>,
        pinned: Option<bool>,
        borderless: Option<bool>,
    ) -> Option<Self> {
        // 解析 x/y 坐标 — 允许 0% 或 0
        let x = x.and_then(|x| PercentOrFixed::from_str(&x).ok());
        let y = y.and_then(|y| PercentOrFixed::from_str(&y).ok());

        // 解析宽度/高度 — 拒绝 0% 或 0
        let width = width.and_then(|w| {
            PercentOrFixed::from_str(&w)
                .ok()
                .and_then(|size| match size {
                    PercentOrFixed::Percent(0) => None,
                    PercentOrFixed::Fixed(0) => None,
                    _ => Some(size),
                })
        });
        let height = height.and_then(|h| {
            PercentOrFixed::from_str(&h)
                .ok()
                .and_then(|size| match size {
                    PercentOrFixed::Percent(0) => None,
                    PercentOrFixed::Fixed(0) => None,
                    _ => Some(size),
                })
        });

        if x.is_none()
            && y.is_none()
            && width.is_none()
            && height.is_none()
            && pinned.is_none()
            && borderless.is_none()
        {
            None
        } else {
            Some(FloatingPaneCoordinates {
                x,
                y,
                width,
                height,
                pinned,
                borderless,
            })
        }
    }
    pub fn with_x_fixed(mut self, x: usize) -> Self {
        self.x = Some(PercentOrFixed::Fixed(x));
        self
    }
    pub fn with_x_percent(mut self, x: usize) -> Self {
        if x > 100 {
            eprintln!("x 必须在 0 到 100 之间");
            return self;
        }
        self.x = Some(PercentOrFixed::Percent(x));
        self
    }
    pub fn with_y_fixed(mut self, y: usize) -> Self {
        self.y = Some(PercentOrFixed::Fixed(y));
        self
    }
    pub fn with_y_percent(mut self, y: usize) -> Self {
        if y > 100 {
            eprintln!("y 必须在 0 到 100 之间");
            return self;
        }
        self.y = Some(PercentOrFixed::Percent(y));
        self
    }
    pub fn with_width_fixed(mut self, width: usize) -> Self {
        self.width = Some(PercentOrFixed::Fixed(width));
        self
    }
    pub fn with_width_percent(mut self, width: usize) -> Self {
        if width > 100 {
            eprintln!("宽度必须在 0 到 100 之间");
            return self;
        }
        self.width = Some(PercentOrFixed::Percent(width));
        self
    }
    pub fn with_height_fixed(mut self, height: usize) -> Self {
        self.height = Some(PercentOrFixed::Fixed(height));
        self
    }
    pub fn with_height_percent(mut self, height: usize) -> Self {
        if height > 100 {
            eprintln!("高度必须在 0 到 100 之间");
            return self;
        }
        self.height = Some(PercentOrFixed::Percent(height));
        self
    }
}

impl From<PaneGeom> for FloatingPaneCoordinates {
    fn from(pane_geom: PaneGeom) -> Self {
        FloatingPaneCoordinates {
            x: Some(PercentOrFixed::Fixed(pane_geom.x)),
            y: Some(PercentOrFixed::Fixed(pane_geom.y)),
            width: Some(PercentOrFixed::Fixed(pane_geom.cols.as_usize())),
            height: Some(PercentOrFixed::Fixed(pane_geom.rows.as_usize())),
            pinned: Some(pane_geom.is_pinned),
            borderless: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OriginatingPlugin {
    pub plugin_id: u32,
    pub client_id: ClientId,
    pub context: Context,
}

impl OriginatingPlugin {
    pub fn new(plugin_id: u32, client_id: ClientId, context: Context) -> Self {
        OriginatingPlugin {
            plugin_id,
            client_id,
            context,
        }
    }
}

#[derive(ValueEnum, Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebSharing {
    #[serde(alias = "on")]
    On,
    #[serde(alias = "off")]
    Off,
    #[serde(alias = "disabled")]
    Disabled,
}

impl Default for WebSharing {
    fn default() -> Self {
        Self::Off
    }
}

impl WebSharing {
    pub fn is_on(&self) -> bool {
        match self {
            WebSharing::On => true,
            _ => false,
        }
    }
    pub fn web_clients_allowed(&self) -> bool {
        match self {
            WebSharing::On => true,
            _ => false,
        }
    }
    pub fn sharing_is_disabled(&self) -> bool {
        match self {
            WebSharing::Disabled => true,
            _ => false,
        }
    }
    pub fn set_sharing(&mut self) -> bool {
        // 如果成功设置共享则返回 true
        match self {
            WebSharing::On => true,
            WebSharing::Off => {
                *self = WebSharing::On;
                true
            },
            WebSharing::Disabled => false,
        }
    }
    pub fn set_not_sharing(&mut self) -> bool {
        // 如果成功设置为不共享则返回 true
        match self {
            WebSharing::On => {
                *self = WebSharing::Off;
                true
            },
            WebSharing::Off => true,
            WebSharing::Disabled => false,
        }
    }
}

impl FromStr for WebSharing {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "On" | "on" => Ok(Self::On),
            "Off" | "off" => Ok(Self::Off),
            "Disabled" | "disabled" => Ok(Self::Disabled),
            _ => Err(format!("No such option: {}", s)),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum NewPanePlacement {
    NoPreference {
        borderless: Option<bool>,
    },
    Tiled {
        direction: Option<Direction>,
        borderless: Option<bool>,
    },
    Floating(Option<FloatingPaneCoordinates>),
    InPlace {
        pane_id_to_replace: Option<PaneId>,
        close_replaced_pane: bool,
        borderless: Option<bool>,
    },
    Stacked {
        pane_id_to_stack_under: Option<PaneId>,
        borderless: Option<bool>,
    },
}

impl Default for NewPanePlacement {
    fn default() -> Self {
        NewPanePlacement::NoPreference { borderless: None }
    }
}

impl NewPanePlacement {
    pub fn with_floating_pane_coordinates(
        floating_pane_coordinates: Option<FloatingPaneCoordinates>,
    ) -> Self {
        NewPanePlacement::Floating(floating_pane_coordinates)
    }
    pub fn with_should_be_in_place(
        self,
        should_be_in_place: bool,
        close_replaced_pane: bool,
    ) -> Self {
        if should_be_in_place {
            NewPanePlacement::InPlace {
                pane_id_to_replace: None,
                close_replaced_pane,
                borderless: None,
            }
        } else {
            self
        }
    }
    pub fn with_pane_id_to_replace(
        pane_id_to_replace: Option<PaneId>,
        close_replaced_pane: bool,
    ) -> Self {
        NewPanePlacement::InPlace {
            pane_id_to_replace,
            close_replaced_pane,
            borderless: None,
        }
    }
    pub fn should_float(&self) -> Option<bool> {
        match self {
            NewPanePlacement::Floating(_) => Some(true),
            NewPanePlacement::Tiled { .. } => Some(false),
            _ => None,
        }
    }
    pub fn floating_pane_coordinates(&self) -> Option<FloatingPaneCoordinates> {
        match self {
            NewPanePlacement::Floating(floating_pane_coordinates) => {
                floating_pane_coordinates.clone()
            },
            _ => None,
        }
    }
    pub fn should_stack(&self) -> bool {
        match self {
            NewPanePlacement::Stacked { .. } => true,
            _ => false,
        }
    }
    pub fn id_of_stack_root(&self) -> Option<PaneId> {
        match self {
            NewPanePlacement::Stacked {
                pane_id_to_stack_under,
                ..
            } => *pane_id_to_stack_under,
            _ => None,
        }
    }
    pub fn get_borderless(&self) -> Option<bool> {
        match self {
            NewPanePlacement::NoPreference { borderless } => *borderless,
            NewPanePlacement::Tiled { borderless, .. } => *borderless,
            NewPanePlacement::Floating(coords) => coords.as_ref().and_then(|c| c.borderless),
            NewPanePlacement::InPlace { borderless, .. } => *borderless,
            NewPanePlacement::Stacked { borderless, .. } => *borderless,
        }
    }
}

type Context = BTreeMap<String, String>;

#[derive(Debug, Clone, EnumDiscriminants, Display)]
#[strum_discriminants(derive(EnumString, Hash, Serialize, Deserialize))]
#[strum_discriminants(name(CommandType))]
pub enum PluginCommand {
    Subscribe(HashSet<EventType>),
    Unsubscribe(HashSet<EventType>),
    SetSelectable(bool),
    ShowCursor(Option<(usize, usize)>),
    GetPluginIds,
    GetZellijVersion,
    OpenFile(FileToOpen, Context),
    OpenFileFloating(FileToOpen, Option<FloatingPaneCoordinates>, Context),
    OpenTerminal(FileToOpen), // 仅将路径用作 cwd
    OpenTerminalFloating(FileToOpen, Option<FloatingPaneCoordinates>), // 仅将路径用作 cwd
    OpenCommandPane(CommandToRun, Context),
    OpenCommandPaneFloating(CommandToRun, Option<FloatingPaneCoordinates>, Context),
    SwitchTabTo(u32), // 标签页索引
    SetTimeout(f64),  // 秒
    ExecCmd(Vec<String>),
    PostMessageTo(PluginMessage),
    PostMessageToPlugin(PluginMessage),
    HideSelf,
    ShowSelf(bool), // bool - 如果隐藏则应浮动
    SwitchToMode(InputMode),
    NewTabsWithLayout(String), // 原始 KDL 布局
    NewTab {
        name: Option<String>,
        cwd: Option<String>,
    },
    NewTabUnfocused {
        name: Option<String>,
        cwd: Option<String>,
    },
    NewTiledPaneInTab {
        tab_position: usize,
    },
    ToggleFloatingPanes {
        tab_id: Option<u64>,
    },
    NewPane,
    GoToNextTab,
    GoToPreviousTab,
    Resize(Resize),
    ResizeWithDirection(ResizeStrategy),
    FocusNextPane,
    FocusPreviousPane,
    FocusLastPane,
    MoveFocus(Direction),
    MoveFocusOrTab(Direction),
    Detach,
    EditScrollback,
    Write(Vec<u8>), // 字节
    WriteChars(String),
    ToggleTab,
    MovePane,
    MovePaneWithDirection(Direction),
    ClearScreen,
    ScrollUp,
    ScrollDown,
    ScrollToTop,
    ScrollToBottom,
    PageScrollUp,
    PageScrollDown,
    ToggleFocusFullscreen,
    ToggleFocusNoUiFullscreen,
    TogglePaneFrames,
    SetPaneFrameStyle(PaneFrameStyle),
    TogglePaneEmbedOrEject,
    UndoRenamePane,
    CloseFocus,
    ToggleActiveTabSync,
    CloseFocusedTab,
    UndoRenameTab,
    QuitZellij,
    PreviousSwapLayout,
    NextSwapLayout,
    GoToTabName(String),
    FocusOrCreateTab(String),
    GoToTab(u32),                       // 标签页索引
    StartOrReloadPlugin(String),        // 插件 URL（例如 file:/path/to/plugin.wasm）
    CloseTerminalPane(u32),             // 终端窗格 id
    ClosePluginPane(u32),               // 插件窗格 id
    FocusTerminalPane(u32, bool, bool), // 终端窗格 id，should_float_if_hidden，should_be_in_place_if_hidden
    FocusPluginPane(u32, bool, bool), // 插件窗格 id，should_float_if_hidden，should_be_in_place_if_hidden
    RenameTerminalPane(u32, String),  // 终端窗格 id，新名称
    RenamePluginPane(u32, String),    // 插件窗格 id，新名称
    RenameTab(u32, String),           // 标签页索引，新名称
    ReportPanic(String),              // 字符串化的 panic
    RequestPluginPermissions(Vec<PermissionType>),
    SwitchSession(ConnectToSession),
    DeleteDeadSession(String),       // String -> 会话名称
    DeleteAllDeadSessions,           // String -> 会话名称
    OpenTerminalInPlace(FileToOpen), // 仅将路径用作 cwd
    OpenFileInPlace(FileToOpen, Context),
    OpenCommandPaneInPlace(CommandToRun, Context),
    RunCommand(
        Vec<String>,              // 命令
        BTreeMap<String, String>, // env_variables
        PathBuf,                  // cwd
        BTreeMap<String, String>, // 上下文
    ),
    WebRequest(
        String, // url
        HttpVerb,
        BTreeMap<String, String>, // headers
        Vec<u8>,                  // body
        BTreeMap<String, String>, // 上下文
    ),
    RenameSession(String),         // String -> 新会话名称
    UnblockCliPipeInput(String),   // String => 管道名称
    BlockCliPipeInput(String),     // String => 管道名称
    CliPipeOutput(String, String), // String => 管道名称，String => 输出
    MessageToPlugin(MessageToPlugin),
    DisconnectOtherClients,
    KillSessions(Vec<String>), // 一个或多个会话名称
    ScanHostFolder(PathBuf),   // TODO: 重命名为 ScanHostFolder
    WatchFilesystem,
    DumpSessionLayout {
        tab_index: Option<usize>,
    },
    CloseSelf,
    NewTabsWithLayoutInfo(LayoutInfo),
    Reconfigure(String, bool), // String -> 字符串化配置，bool -> 保存配置
    // 文件到磁盘
    HidePaneWithId(PaneId),
    ShowPaneWithId(PaneId, bool, bool), // bools -> should_float_if_hidden，should_focus_pane
    OpenCommandPaneBackground(CommandToRun, Context),
    RerunCommandPane(u32), // u32 - 终端窗格 id
    ResizePaneIdWithDirection(ResizeStrategy, PaneId),
    EditScrollbackForPaneWithId(PaneId),
    GetPaneScrollback {
        pane_id: PaneId,
        get_full_scrollback: bool,
    },
    WriteToPaneId(Vec<u8>, PaneId),
    WriteCharsToPaneId(String, PaneId),
    SendSigintToPaneId(PaneId),
    SendSigkillToPaneId(PaneId),
    GetPanePid {
        pane_id: PaneId,
    },
    GetPaneRunningCommand {
        pane_id: PaneId,
    },
    GetPaneCwd {
        pane_id: PaneId,
    },
    MovePaneWithPaneId(PaneId),
    MovePaneWithPaneIdInDirection(PaneId, Direction),
    ClearScreenForPaneId(PaneId),
    ScrollUpInPaneId(PaneId),
    ScrollDownInPaneId(PaneId),
    ScrollToTopInPaneId(PaneId),
    ScrollToBottomInPaneId(PaneId),
    PageScrollUpInPaneId(PaneId),
    PageScrollDownInPaneId(PaneId),
    TogglePaneIdFullscreen(PaneId),
    TogglePaneEmbedOrEjectForPaneId(PaneId),
    CloseTabWithIndex(usize), // usize - tab_index
    BreakPanesToNewTab(Vec<PaneId>, Option<String>, bool), // bool -
    // should_change_focus_to_new_tab，
    // Option<String> - 可选名称，用于
    // 新标签页
    BreakPanesToTabWithIndex(Vec<PaneId>, usize, bool), // usize - tab_index，bool -
    // should_change_focus_to_new_tab
    SwitchTabToId(u64),                            // u64 - tab_id
    GoToTabWithId(u64),                            // u64 - tab_id
    CloseTabWithId(u64),                           // u64 - tab_id
    RenameTabWithId(u64, String),                  // u64 - tab_id，String - 新名称
    BreakPanesToTabWithId(Vec<PaneId>, u64, bool), // u64 - tab_id，bool -
    // should_change_focus_to_target_tab
    ReloadPlugin(u32), // u32 - 插件窗格 id
    LoadNewPlugin {
        url: String,
        config: BTreeMap<String, String>,
        load_in_background: bool,
        skip_plugin_cache: bool,
    },
    RebindKeys {
        keys_to_rebind: Vec<(InputMode, KeyWithModifier, Vec<Action>)>,
        keys_to_unbind: Vec<(InputMode, KeyWithModifier)>,
        write_config_to_disk: bool,
    },
    ListClients,
    ChangeHostFolder(PathBuf),
    SetFloatingPanePinned(PaneId, bool), // bool -> 应该被固定
    StackPanes(Vec<PaneId>),
    ChangeFloatingPanesCoordinates(Vec<(PaneId, FloatingPaneCoordinates)>),
    TogglePaneBorderless(PaneId),
    SetPaneBorderless(PaneId, bool),
    OpenCommandPaneNearPlugin(CommandToRun, Context),
    OpenTerminalNearPlugin(FileToOpen),
    OpenTerminalFloatingNearPlugin(FileToOpen, Option<FloatingPaneCoordinates>),
    OpenTerminalInPlaceOfPlugin(FileToOpen, bool), // bool -> close_plugin_after_replace
    OpenCommandPaneFloatingNearPlugin(CommandToRun, Option<FloatingPaneCoordinates>, Context),
    OpenCommandPaneInPlaceOfPlugin(CommandToRun, bool, Context), // bool ->
    // close_plugin_after_replace
    OpenFileNearPlugin(FileToOpen, Context),
    OpenFileFloatingNearPlugin(FileToOpen, Option<FloatingPaneCoordinates>, Context),
    StartWebServer,
    StopWebServer,
    ShareCurrentSession,
    StopSharingCurrentSession,
    OpenFileInPlaceOfPlugin(FileToOpen, bool, Context), // bool -> close_plugin_after_replace
    GroupAndUngroupPanes(Vec<PaneId>, Vec<PaneId>, bool), // 要分组的窗格，要取消分组的窗格，
    // bool -> 适用于所有客户端
    HighlightAndUnhighlightPanes(Vec<PaneId>, Vec<PaneId>), // 要高亮的窗格，要
    // 取消高亮的窗格
    CloseMultiplePanes(Vec<PaneId>),
    FloatMultiplePanes(Vec<PaneId>),
    EmbedMultiplePanes(Vec<PaneId>),
    QueryWebServerStatus,
    SetSelfMouseSelectionSupport(bool),
    GenerateWebLoginToken(Option<String>, bool), //（token_label, read_only）
    RevokeWebLoginToken(String), // String -> token id（提供的名称或生成的 id）
    ListWebLoginTokens,
    RevokeAllWebLoginTokens,
    RenameWebLoginToken(String, String), //（original_name, new_name）
    InterceptKeyPresses,
    ClearKeyPressesIntercepts,
    ReplacePaneWithExistingPane(PaneId, PaneId, bool), //（要替换的窗格 id，现有窗格的 id，
    // suppress_replaced_pane）
    RunAction(Action, BTreeMap<String, String>),
    CopyToClipboard(String), // 要复制的文本
    OverrideLayout(
        LayoutInfo,
        bool,                     // retain_existing_terminal_panes
        bool,                     // retain_existing_plugin_panes
        bool,                     // apply_only_to_active_tab，
        BTreeMap<String, String>, // 上下文
    ),
    SaveLayout {
        layout_name: String,
        layout_kdl: String,
        overwrite: bool,
    },
    DeleteLayout {
        layout_name: String,
    },
    RenameLayout {
        old_layout_name: String,
        new_layout_name: String,
    },
    EditLayout {
        layout_name: String,
        context: Context,
    },
    GenerateRandomName,
    DumpLayout(String),
    ParseLayout(String), // String 包含原始 KDL 布局
    GetLayoutDir,
    GetFocusedPaneInfo,
    SaveSession,
    CurrentSessionLastSavedTime,
    GetPaneInfo(PaneId),
    GetTabInfo(usize), // tab_id
    GetSessionEnvironmentVariables,
    OpenCommandPaneInNewTab(CommandToRun, Context),
    OpenPluginPaneInNewTab {
        plugin_url: String,
        configuration: BTreeMap<String, String>,
        context: Context,
    },
    OpenEditorPaneInNewTab(FileToOpen, Context),
    OpenCommandPaneInPlaceOfPaneId(PaneId, CommandToRun, bool, Context), // bool = close_replaced_pane
    OpenTerminalPaneInPlaceOfPaneId(PaneId, FileToOpen, bool),
    OpenEditPaneInPlaceOfPaneId(PaneId, FileToOpen, bool, Context),
    HideFloatingPanes {
        tab_id: Option<usize>,
    },
    ShowFloatingPanes {
        tab_id: Option<usize>,
    },
    SetPaneColor(PaneId, Option<String>, Option<String>), //（pane_id, fg, bg）
    SetPaneRegexHighlights(PaneId, Vec<RegexHighlight>),
    ClearPaneHighlights(PaneId),
    OpenPluginPaneFloating {
        plugin_url: String,
        configuration: BTreeMap<String, String>,
        floating_pane_coordinates: Option<FloatingPaneCoordinates>,
        context: BTreeMap<String, String>,
    },
    ListWindowsVolumes,
    GetSessionList,
    KillSessionsAndReply(Vec<String>), // 一个或多个会话名称；发送响应回来
    DeleteDeadSessionAndReply(String), // 会话名称；发送响应回来
    DeleteAllDeadSessionsAndReply,     // 无载荷；发送响应回来
    SetSoftKeyboard(bool),
    FocusHostSession,
}

// 在新标签页中打开窗格的插件 API 方法的响应类型
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenPaneInNewTabResponse {
    pub tab_id: Option<usize>,
    pub pane_id: Option<PaneId>,
}

// 创建标签页的插件 API 方法的响应类型
pub type NewTabResponse = Option<usize>;
pub type NewTabUnfocusedResponse = Option<usize>;
pub type NewTabsResponse = Vec<usize>;
pub type FocusOrCreateTabResponse = Option<usize>;
pub type BreakPanesToNewTabResponse = Option<usize>;
pub type BreakPanesToTabWithIndexResponse = Option<usize>;
pub type BreakPanesToTabWithIdResponse = Option<usize>;

// 创建窗格的插件 API 方法的响应类型
pub type OpenFileResponse = Option<PaneId>;
pub type OpenFileFloatingResponse = Option<PaneId>;
pub type OpenFileInPlaceResponse = Option<PaneId>;
pub type OpenFileNearPluginResponse = Option<PaneId>;
pub type OpenFileFloatingNearPluginResponse = Option<PaneId>;
pub type OpenFileInPlaceOfPluginResponse = Option<PaneId>;

pub type OpenTerminalResponse = Option<PaneId>;
pub type OpenTerminalFloatingResponse = Option<PaneId>;
pub type OpenTerminalInPlaceResponse = Option<PaneId>;
pub type OpenTerminalNearPluginResponse = Option<PaneId>;
pub type OpenTerminalFloatingNearPluginResponse = Option<PaneId>;
pub type OpenTerminalInPlaceOfPluginResponse = Option<PaneId>;
pub type NewTiledPaneInTabResponse = Option<PaneId>;

pub type OpenCommandPaneResponse = Option<PaneId>;
pub type OpenCommandPaneFloatingResponse = Option<PaneId>;
pub type OpenCommandPaneInPlaceResponse = Option<PaneId>;
pub type OpenCommandPaneNearPluginResponse = Option<PaneId>;
pub type OpenCommandPaneFloatingNearPluginResponse = Option<PaneId>;
pub type OpenCommandPaneInPlaceOfPluginResponse = Option<PaneId>;
pub type OpenCommandPaneBackgroundResponse = Option<PaneId>;
pub type OpenCommandPaneInPlaceOfPaneIdResponse = Option<PaneId>;
pub type OpenTerminalPaneInPlaceOfPaneIdResponse = Option<PaneId>;
pub type OpenEditPaneInPlaceOfPaneIdResponse = Option<PaneId>;
pub type OpenPluginPaneFloatingResponse = Option<PaneId>;

#[test]
pub fn can_parse_unicode_bare_keys() {
    let key = "1087"; // п
    assert_eq!(
        BareKey::from_bytes_with_u(&key.as_bytes()),
        Some(BareKey::Char('п')),
        "Can parse a bare 'п' keypress"
    );
    let key = "1255"; // ӧ
    assert_eq!(
        BareKey::from_bytes_with_u(&key.as_bytes()),
        Some(BareKey::Char('ӧ')),
        "Can parse a bare 'ӧ' keypress"
    );
    let key = "1098"; // ъ
    assert_eq!(
        BareKey::from_bytes_with_u(&key.as_bytes()),
        Some(BareKey::Char('ъ')),
        "Can parse a bare 'ъ' keypress"
    );
}
