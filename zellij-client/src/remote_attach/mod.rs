mod auth;
mod config;
pub mod http_client;
pub mod websockets;

#[cfg(test)]
mod unit;

pub use websockets::WebSocketConnections;

use crate::os_input_output::ClientOsApi;
use crate::RemoteClientError;
use tokio::runtime::Handle;
use zellij_utils::remote_session_tokens;

// 在测试中，仅尝试一次（无重试）以避免交互式提示
// 在生产环境中，允许多达 3 次尝试（初始 + 2 次重试）
#[cfg(test)]
const MAX_AUTH_ATTEMPTS: u32 = 1;

#[cfg(not(test))]
const MAX_AUTH_ATTEMPTS: u32 = 3;

/// 通过 HTTP(S) 连接到远程 Zellij 会话
///
/// 此函数处理完整的认证流程，包括：
/// - URL 验证
/// - 会话令牌管理（--forget、--token 标志）
/// - 尝试保存的会话令牌
/// - 带重试逻辑的交互式认证
/// - 使用 --remember 时保存会话令牌
///
/// 成功时返回 WebSocketConnections
pub fn attach_to_remote_session(
    runtime: Handle,
    _os_input: Box<dyn ClientOsApi>,
    remote_session_url: &str,
    token: Option<String>,
    remember: bool,
    forget: bool,
    ca_cert: Option<&std::path::Path>,
    insecure: bool,
) -> Result<WebSocketConnections, RemoteClientError> {
    // 提取服务器 URL 用于令牌管理
    let server_url = extract_server_url(remote_session_url)?;

    // 处理 --forget 标志
    if forget {
        let _ = remote_session_tokens::delete_session_token(&server_url);
    }

    // 如果提供了 --token，删除保存的会话令牌
    if token.is_some() {
        let _ = remote_session_tokens::delete_session_token(&server_url);
    }

    if token.is_none() {
        if let Some(connections) = try_to_connect_with_saved_session_token(
            runtime.clone(),
            remote_session_url,
            &server_url,
            ca_cert,
            insecure,
        )? {
            return Ok(connections);
        }
    }

    // 带重试逻辑的常规认证流程
    authenticate_with_retry(
        runtime,
        remote_session_url,
        token,
        remember,
        ca_cert,
        insecure,
    )
}

/// 尝试使用保存的会话令牌连接
/// 成功时返回 Ok(Some(connections))，如果应使用认证重试则返回 Ok(None)
fn try_to_connect_with_saved_session_token(
    runtime: Handle,
    remote_session_url: &str,
    server_url: &str,
    ca_cert: Option<&std::path::Path>,
    insecure: bool,
) -> Result<Option<WebSocketConnections>, RemoteClientError> {
    if let Ok(Some(saved_session_token)) = remote_session_tokens::get_session_token(server_url) {
        // 我们有一个保存的会话令牌，让我们尝试用它进行认证
        let ca_cert_owned = ca_cert.map(|p| p.to_path_buf());
        match runtime.block_on(async move {
            remote_attach_with_session_token(
                remote_session_url,
                &saved_session_token,
                ca_cert_owned.as_deref(),
                insecure,
            )
            .await
        }) {
            Ok(connections) => {
                return Ok(Some(connections));
            },
            Err(RemoteClientError::SessionTokenExpired) => {
                // 会话已过期 — 删除并返回以重试
                let _ = remote_session_tokens::delete_session_token(server_url);
                eprintln!("会话已过期，请重新认证");
                return Ok(None);
            },
            Err(e) => {
                return Err(e);
            },
        }
    }
    Ok(None)
}

fn dialoguer_error_to_client_error(error: dialoguer::Error) -> RemoteClientError {
    match error {
        dialoguer::Error::IO(error) => RemoteClientError::IoError(error),
    }
}

fn authenticate_with_retry(
    runtime: Handle,
    remote_session_url: &str,
    initial_token: Option<String>,
    remember: bool,
    ca_cert: Option<&std::path::Path>,
    insecure: bool,
) -> Result<WebSocketConnections, RemoteClientError> {
    use dialoguer::{Confirm, Password};

    let mut attempt = 0;
    let mut current_token = initial_token;

    loop {
        attempt += 1;

        let auth_token = match &current_token {
            Some(t) => t.clone(),
            None => Password::new()
                .with_prompt("Enter authentication token")
                .interact()
                .map_err(dialoguer_error_to_client_error)?,
        };

        let ca_cert_owned = ca_cert.map(|p| p.to_path_buf());
        match runtime.block_on(async move {
            remote_attach(
                remote_session_url,
                &auth_token,
                remember,
                ca_cert_owned.as_deref(),
                insecure,
            )
            .await
        }) {
            Ok((connections, session_token_opt)) => {
                // 如果获得了会话令牌，保存它
                if let Some(session_token) = session_token_opt {
                    let server_url = extract_server_url(remote_session_url)?;
                    let _ = remote_session_tokens::save_session_token(&server_url, &session_token);
                }
                return Ok(connections);
            },
            Err(RemoteClientError::InvalidAuthToken) => {
                eprintln!("无效的身份验证令牌");

                if attempt >= MAX_AUTH_ATTEMPTS {
                    eprintln!(
                        "超过最大认证尝试次数（{}）。",
                        MAX_AUTH_ATTEMPTS
                    );
                    return Err(RemoteClientError::InvalidAuthToken);
                }

                match Confirm::new()
                    .with_prompt("Try again?")
                    .default(true)
                    .interact()
                {
                    Ok(true) => {
                        current_token = None;
                        continue;
                    },
                    Ok(false) => {
                        return Err(RemoteClientError::InvalidAuthToken);
                    },
                    Err(e) => {
                        return Err(dialoguer_error_to_client_error(e));
                    },
                }
            },
            Err(e) => {
                return Err(e);
            },
        }
    }
}

async fn remote_attach(
    server_url: &str,
    auth_token: &str,
    remember_me: bool,
    ca_cert: Option<&std::path::Path>,
    insecure: bool,
) -> Result<(websockets::WebSocketConnections, Option<String>), RemoteClientError> {
    let server_base_url = extract_server_url(server_url)?;
    let session_name = extract_session_name(server_url)?;
    let (web_client_id, http_client, session_token) = auth::authenticate(
        &server_base_url,
        auth_token,
        remember_me,
        &session_name,
        ca_cert,
        insecure,
    )
    .await?;
    let connections = websockets::establish_websocket_connections(
        &web_client_id,
        &http_client,
        &server_base_url,
        &session_name,
        ca_cert,
        insecure,
    )
    .await
    .map_err(|e| RemoteClientError::ConnectionFailed(e.to_string()))?;
    Ok((connections, session_token))
}

async fn remote_attach_with_session_token(
    server_url: &str,
    session_token: &str,
    ca_cert: Option<&std::path::Path>,
    insecure: bool,
) -> Result<websockets::WebSocketConnections, RemoteClientError> {
    let server_base_url = extract_server_url(server_url)?;
    let session_name = extract_session_name(server_url)?;
    let (web_client_id, http_client) = auth::validate_session_token(
        &server_base_url,
        session_token,
        &session_name,
        ca_cert,
        insecure,
    )
    .await?;
    let connections = websockets::establish_websocket_connections(
        &web_client_id,
        &http_client,
        &server_base_url,
        &session_name,
        ca_cert,
        insecure,
    )
    .await
    .map_err(|e| RemoteClientError::ConnectionFailed(e.to_string()))?;
    Ok(connections)
}

pub fn extract_server_url(full_url: &str) -> Result<String, RemoteClientError> {
    let parsed = url::Url::parse(full_url)?;
    let mut base_url = parsed.clone();
    base_url.set_path("");
    base_url.set_query(None);
    base_url.set_fragment(None);
    Ok(base_url.to_string().trim_end_matches('/').to_string())
}

fn extract_session_name(server_url: &str) -> Result<String, RemoteClientError> {
    let parsed_url = url::Url::parse(server_url)?;
    let path = parsed_url.path();
    // 从路径中提取会话名称（第一个 / 之后的所有内容）
    if path.len() > 1 && path.starts_with('/') {
        Ok(path[1..].trim_end_matches('/').to_string())
    } else {
        Ok(String::new())
    }
}
