// 此文件由 prost-build 生成。
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Resize {
    #[prost(enumeration="ResizeAction", tag="1")]
    pub resize_action: i32,
    #[prost(enumeration="ResizeDirection", optional, tag="2")]
    pub direction: ::core::option::Option<i32>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MoveDirection {
    #[prost(enumeration="ResizeDirection", tag="1")]
    pub direction: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ResizeAction {
    Increase = 0,
    Decrease = 1,
}
impl ResizeAction {
    /// ProtoBuf 定义中使用的枚举字段名的字符串值。
    ///
    /// 这些值不以任何方式进行转换，因此被认为是稳定的
    ///（如果 ProtoBuf 定义不变）且可供编程安全使用。
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ResizeAction::Increase => "Increase",
            ResizeAction::Decrease => "Decrease",
        }
    }
    /// 从 ProtoBuf 定义中使用的字段名创建枚举。
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "Increase" => Some(Self::Increase),
            "Decrease" => Some(Self::Decrease),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ResizeDirection {
    Left = 0,
    Right = 1,
    Up = 2,
    Down = 3,
}
impl ResizeDirection {
    /// ProtoBuf 定义中使用的枚举字段名的字符串值。
    ///
    /// 这些值不以任何方式进行转换，因此被认为是稳定的
    ///（如果 ProtoBuf 定义不变）且可供编程安全使用。
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ResizeDirection::Left => "Left",
            ResizeDirection::Right => "Right",
            ResizeDirection::Up => "Up",
            ResizeDirection::Down => "Down",
        }
    }
    /// 从 ProtoBuf 定义中使用的字段名创建枚举。
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "Left" => Some(Self::Left),
            "Right" => Some(Self::Right),
            "Up" => Some(Self::Up),
            "Down" => Some(Self::Down),
            _ => None,
        }
    }
}
