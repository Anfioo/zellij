// 此文件由 prost-build 生成。
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InputModeMessage {
    #[prost(enumeration="InputMode", tag="1")]
    pub input_mode: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum InputMode {
    /// / 在 `Normal` 模式下，输入始终写入终端，除了导致
    /// / 到其他模式
    Normal = 0,
    /// / 在 `Locked` 模式下，输入始终写入终端且所有快捷键被禁用
    /// / 除了返回正常模式的那个
    Locked = 1,
    /// / `Resize` 模式允许调整不同现有窗格的大小。
    Resize = 2,
    /// / `Pane` 模式允许创建和关闭窗格，以及在它们之间移动。
    Pane = 3,
    /// / `Tab` 模式允许创建和关闭标签页，以及在它们之间移动。
    Tab = 4,
    /// / `Scroll` 模式允许在窗格内上下滚动。
    Scroll = 5,
    /// / `EnterSearch` 模式允许在窗格的回滚缓冲区中输入搜索关键词。
    EnterSearch = 6,
    /// / `Search` 模式允许在窗格中搜索术语（`Scroll` 的超集）。
    Search = 7,
    /// / `RenameTab` 模式允许为标签页分配新名称。
    RenameTab = 8,
    /// / `RenamePane` 模式允许为窗格分配新名称。
    RenamePane = 9,
    /// / `Session` 模式允许分离会话
    Session = 10,
    /// / `Move` 模式允许在标签页内移动不同的现有窗格
    Move = 11,
    /// / `Prompt` 模式允许与活动提示交互。
    Prompt = 12,
    /// / `Tmux` 模式允许基本的 tmux 快捷键绑定功能
    Tmux = 13,
}
impl InputMode {
    /// ProtoBuf 定义中使用的枚举字段名的字符串值。
    ///
    /// 这些值不以任何方式进行转换，因此被认为是稳定的
    ///（如果 ProtoBuf 定义不变）且可供编程安全使用。
    pub fn as_str_name(&self) -> &'static str {
        match self {
            InputMode::Normal => "Normal",
            InputMode::Locked => "Locked",
            InputMode::Resize => "Resize",
            InputMode::Pane => "Pane",
            InputMode::Tab => "Tab",
            InputMode::Scroll => "Scroll",
            InputMode::EnterSearch => "EnterSearch",
            InputMode::Search => "Search",
            InputMode::RenameTab => "RenameTab",
            InputMode::RenamePane => "RenamePane",
            InputMode::Session => "Session",
            InputMode::Move => "Move",
            InputMode::Prompt => "Prompt",
            InputMode::Tmux => "Tmux",
        }
    }
    /// 从 ProtoBuf 定义中使用的字段名创建枚举。
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "Normal" => Some(Self::Normal),
            "Locked" => Some(Self::Locked),
            "Resize" => Some(Self::Resize),
            "Pane" => Some(Self::Pane),
            "Tab" => Some(Self::Tab),
            "Scroll" => Some(Self::Scroll),
            "EnterSearch" => Some(Self::EnterSearch),
            "Search" => Some(Self::Search),
            "RenameTab" => Some(Self::RenameTab),
            "RenamePane" => Some(Self::RenamePane),
            "Session" => Some(Self::Session),
            "Move" => Some(Self::Move),
            "Prompt" => Some(Self::Prompt),
            "Tmux" => Some(Self::Tmux),
            _ => None,
        }
    }
}
