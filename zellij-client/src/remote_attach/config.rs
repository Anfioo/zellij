use std::time::Duration;

// API 端点
pub const LOGIN_ENDPOINT: &str = "/command/login";
pub const SESSION_ENDPOINT: &str = "/session";
pub const WS_TERMINAL_ENDPOINT: &str = "/ws/terminal";
pub const WS_CONTROL_ENDPOINT: &str = "/ws/control";

// 连接设置
pub const CONNECTION_TIMEOUT_SECS: u64 = 30;

pub fn connection_timeout() -> Duration {
    Duration::from_secs(CONNECTION_TIMEOUT_SECS)
}
