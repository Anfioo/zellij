// 此文件由 prost-build 生成。
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InstructionForWebServer {
    #[prost(oneof="instruction_for_web_server::Instruction", tags="1, 2")]
    pub instruction: ::core::option::Option<instruction_for_web_server::Instruction>,
}
/// `InstructionForWebServer` 中的嵌套消息和枚举类型。
pub mod instruction_for_web_server {
    #[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Instruction {
        #[prost(message, tag="1")]
        ShutdownWebServer(super::ShutdownWebServerMsg),
        /// 未来的命令可以添加到这里
        /// RestartWebServerMsg restart_web_server = 3;
        /// ReloadConfigMsg reload_config = 4;
        #[prost(message, tag="2")]
        QueryVersion(super::QueryVersionMsg),
    }
}
/// 目前为空，但允许未来添加如优雅超时等参数
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ShutdownWebServerMsg {
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryVersionMsg {
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WebServerResponse {
    #[prost(oneof="web_server_response::Response", tags="1")]
    pub response: ::core::option::Option<web_server_response::Response>,
}
/// `WebServerResponse` 中的嵌套消息和枚举类型。
pub mod web_server_response {
    #[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Response {
        #[prost(message, tag="1")]
        Version(super::VersionResponseMsg),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VersionResponseMsg {
    #[prost(string, tag="1")]
    pub version: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub ip: ::prost::alloc::string::String,
    #[prost(uint32, tag="3")]
    pub port: u32,
}
