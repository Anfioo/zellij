pub mod cli;
pub mod client_server_contract;
pub mod consts;
pub mod data;
pub mod envs;
pub mod errors;
pub mod home;
#[cfg(not(windows))]
mod home_unix;
#[cfg(windows)]
mod home_windows;
pub mod input;
pub mod kdl;
pub mod nested_session_contract;
pub mod pane_size;
pub mod plugin_api;
pub mod position;
pub mod session_serialization;
pub mod setup;
pub mod shared;

// 以下模块在目标为 wasm 时无法使用
#[cfg(not(target_family = "wasm"))]
pub mod channels; // 需要 tokio
#[cfg(not(target_family = "wasm"))]
pub mod common_path;
#[cfg(not(target_family = "wasm"))]
pub mod downloader; // 需要 tokio
#[cfg(not(target_family = "wasm"))]
pub mod ipc; // 需要 interprocess
#[cfg(not(target_family = "wasm"))]
pub mod logging; // 需要 log4rs
#[cfg(not(target_family = "wasm"))]
pub mod nested_session;
#[cfg(all(not(target_family = "wasm"), feature = "web_server_capability"))]
pub mod remote_session_tokens;
#[cfg(not(target_family = "wasm"))]
pub mod sessions;
#[cfg(all(not(target_family = "wasm"), feature = "web_server_capability"))]
pub mod web_authentication_tokens;
#[cfg(all(not(target_family = "wasm"), feature = "web_server_capability"))]
pub mod web_server_commands;
#[cfg(all(not(target_family = "wasm"), feature = "web_server_capability"))]
pub mod web_server_contract;

// TODO(hartan): 在下一个次版本中移除此重新导出。
pub use ::prost;

// 内置库
pub mod vendored;
