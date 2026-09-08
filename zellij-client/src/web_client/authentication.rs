use crate::web_client::utils::parse_cookies;
use axum::body::Body;
use axum::http::header::SET_COOKIE;
use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use axum_extra::extract::cookie::{Cookie, SameSite};
use zellij_utils::web_authentication_tokens::{
    hash_token, is_session_token_read_only, validate_session_token,
};

#[derive(Clone)]
pub struct SessionTokenHash(pub String);

#[derive(Clone, Copy)]
pub struct IsReadOnly(pub bool);

pub async fn auth_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    let cookies = parse_cookies(&request);

    let session_token = match cookies.get("session_token") {
        Some(token) => token.clone(),
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    match validate_session_token(&session_token) {
        Ok(true) => {
            // 检查这是否是只读令牌
            let is_read_only = is_session_token_read_only(&session_token).unwrap_or(true);

            // 计算会话令牌哈希用于客户端所有权验证
            let session_token_hash = hash_token(&session_token);

            // 存储在请求扩展中供下游处理程序使用
            let mut request = request;
            request.extensions_mut().insert(IsReadOnly(is_read_only));
            request
                .extensions_mut()
                .insert(SessionTokenHash(session_token_hash));

            let response = next.run(request).await;
            Ok(response)
        },
        Ok(false) | Err(_) => {
            // 撤销 session_token，因为如果它存在，它不再有效
            let mut response = Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::empty())
                .unwrap();

            // 清除安全和非安全版本
            // 以防用户之前在 http 上，现在在 https 上
            // 或反之亦然
            let clear_cookies = [
                Cookie::build(("session_token", ""))
                    .http_only(true)
                    .secure(false)
                    .same_site(SameSite::Strict)
                    .path("/")
                    .max_age(time::Duration::seconds(0))
                    .build(),
                Cookie::build(("session_token", ""))
                    .http_only(true)
                    .secure(true)
                    .same_site(SameSite::Strict)
                    .path("/")
                    .max_age(time::Duration::seconds(0))
                    .build(),
            ];

            for cookie in clear_cookies {
                response
                    .headers_mut()
                    .append(SET_COOKIE, cookie.to_string().parse().unwrap());
            }

            Ok(response)
        },
    }
}
