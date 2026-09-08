use super::config::{LOGIN_ENDPOINT, SESSION_ENDPOINT};
use super::http_client::HttpClientWithCookies;
use crate::RemoteClientError;
use isahc::{AsyncReadResponseExt, Request};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct LoginRequest {
    auth_token: String,
    remember_me: bool,
}

#[derive(Deserialize)]
pub struct SessionResponse {
    pub web_client_id: String,
}

fn session_url(server_base_url: &str, session_name: &str) -> String {
    if session_name.is_empty() {
        format!("{}{}", server_base_url, SESSION_ENDPOINT)
    } else {
        format!(
            "{}{}?session={}",
            server_base_url,
            SESSION_ENDPOINT,
            urlencoding::encode(session_name)
        )
    }
}

pub async fn authenticate(
    server_base_url: &str,
    auth_token: &str,
    remember_me: bool,
    session_name: &str,
    ca_cert: Option<&std::path::Path>,
    insecure: bool,
) -> Result<(String, HttpClientWithCookies, Option<String>), RemoteClientError> {
    let http_client = HttpClientWithCookies::new(ca_cert, insecure)
        .map_err(|e| RemoteClientError::Other(Box::new(e)))?;

    // 步骤 1：使用认证令牌登录
    let login_url = format!("{}{}", server_base_url, LOGIN_ENDPOINT);

    let login_request = LoginRequest {
        auth_token: auth_token.to_string(),
        remember_me,
    };

    let response = http_client
        .send_with_cookies(
            Request::post(login_url)
                .header("Content-Type", "application/json")
                .header("User-Agent", "http-terminal-client/1.0")
                .header("Accept", "application/json")
                .body(
                    serde_json::to_vec(&login_request)
                        .map_err(|e| RemoteClientError::Other(Box::new(e)))?,
                )
                .map_err(|e| RemoteClientError::Other(Box::new(e)))?,
        )
        .await
        .map_err(|e| RemoteClientError::ConnectionFailed(e.to_string()))?;

    // 处理 HTTP 状态码
    match response.status().as_u16() {
        401 => return Err(RemoteClientError::InvalidAuthToken),
        status if !response.status().is_success() => {
            return Err(RemoteClientError::ConnectionFailed(format!(
                "Server returned status {}",
                status
            )));
        },
        _ => {},
    }

    // 步骤 2：获取会话/客户端 ID
    let session_url = session_url(server_base_url, session_name);

    let mut session_response = http_client
        .send_with_cookies(
            Request::post(session_url)
                .header("Content-Type", "application/json")
                .header("User-Agent", "http-terminal-client/1.0")
                .header("Accept", "application/json")
                .body("{}".as_bytes().to_vec())
                .map_err(|e| RemoteClientError::Other(Box::new(e)))?,
        )
        .await
        .map_err(|e| RemoteClientError::ConnectionFailed(e.to_string()))?;

    // 处理会话响应
    match session_response.status().as_u16() {
        401 => return Err(RemoteClientError::Unauthorized),
        status if !session_response.status().is_success() => {
            return Err(RemoteClientError::ConnectionFailed(format!(
                "Server returned status {}",
                status
            )));
        },
        _ => {},
    }

    let response_body = session_response
        .text()
        .await
        .map_err(|e| RemoteClientError::Other(Box::new(e)))?;
    let session_data: SessionResponse =
        serde_json::from_str(&response_body).map_err(|e| RemoteClientError::Other(Box::new(e)))?;

    // 如果 remember_me 为 true，提取 session_token
    let session_token = if remember_me {
        http_client.get_cookie("session_token")
    } else {
        None
    };

    Ok((session_data.web_client_id, http_client, session_token))
}

pub async fn validate_session_token(
    server_base_url: &str,
    session_token: &str,
    session_name: &str,
    ca_cert: Option<&std::path::Path>,
    insecure: bool,
) -> Result<(String, HttpClientWithCookies), RemoteClientError> {
    let http_client = HttpClientWithCookies::new(ca_cert, insecure)
        .map_err(|e| RemoteClientError::Other(Box::new(e)))?;

    // 预填充 session_token cookie
    http_client.set_cookie("session_token".to_string(), session_token.to_string());

    // 跳过 /login，直接转到 /session 端点
    let session_url = session_url(server_base_url, session_name);

    let mut session_response = http_client
        .send_with_cookies(
            Request::post(session_url)
                .header("Content-Type", "application/json")
                .header("User-Agent", "http-terminal-client/1.0")
                .header("Accept", "application/json")
                .body("{}".as_bytes().to_vec())
                .map_err(|e| RemoteClientError::Other(Box::new(e)))?,
        )
        .await
        .map_err(|e| RemoteClientError::ConnectionFailed(e.to_string()))?;

    match session_response.status().as_u16() {
        401 => Err(RemoteClientError::SessionTokenExpired),
        status if !session_response.status().is_success() => Err(
            RemoteClientError::ConnectionFailed(format!("Server returned status {}", status)),
        ),
        _ => {
            let response_body = session_response
                .text()
                .await
                .map_err(|e| RemoteClientError::Other(Box::new(e)))?;
            let session_data: SessionResponse = serde_json::from_str(&response_body)
                .map_err(|e| RemoteClientError::Other(Box::new(e)))?;
            Ok((session_data.web_client_id, http_client))
        },
    }
}
