// 此文件由 prost-build 生成。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum PaneFrameStyle {
    Full = 0,
    Titles = 1,
    None = 2,
}
impl PaneFrameStyle {
    /// ProtoBuf 定义中使用的枚举字段名的字符串值。
    ///
    /// 这些值不以任何方式进行转换，因此被认为是稳定的
    ///（如果 ProtoBuf 定义不变）且可供编程安全使用。
    pub fn as_str_name(&self) -> &'static str {
        match self {
            PaneFrameStyle::Full => "Full",
            PaneFrameStyle::Titles => "Titles",
            PaneFrameStyle::None => "None",
        }
    }
    /// 从 ProtoBuf 定义中使用的字段名创建枚举。
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "Full" => Some(Self::Full),
            "Titles" => Some(Self::Titles),
            "None" => Some(Self::None),
            _ => None,
        }
    }
}
