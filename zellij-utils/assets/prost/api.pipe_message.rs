// 此文件由 prost-build 生成。
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PipeMessage {
    #[prost(enumeration="PipeSource", tag="1")]
    pub source: i32,
    #[prost(string, optional, tag="2")]
    pub cli_source_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(uint32, optional, tag="3")]
    pub plugin_source_id: ::core::option::Option<u32>,
    #[prost(string, tag="4")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, optional, tag="5")]
    pub payload: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, repeated, tag="6")]
    pub args: ::prost::alloc::vec::Vec<Arg>,
    #[prost(bool, tag="7")]
    pub is_private: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Arg {
    #[prost(string, tag="1")]
    pub key: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub value: ::prost::alloc::string::String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum PipeSource {
    Cli = 0,
    Plugin = 1,
    Keybind = 2,
}
impl PipeSource {
    /// ProtoBuf 定义中使用的枚举字段名的字符串值。
    ///
    /// 这些值不以任何方式进行转换，因此被认为是稳定的
    ///（如果 ProtoBuf 定义不变）且可供编程安全使用。
    pub fn as_str_name(&self) -> &'static str {
        match self {
            PipeSource::Cli => "Cli",
            PipeSource::Plugin => "Plugin",
            PipeSource::Keybind => "Keybind",
        }
    }
    /// 从 ProtoBuf 定义中使用的字段名创建枚举。
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "Cli" => Some(Self::Cli),
            "Plugin" => Some(Self::Plugin),
            "Keybind" => Some(Self::Keybind),
            _ => None,
        }
    }
}
