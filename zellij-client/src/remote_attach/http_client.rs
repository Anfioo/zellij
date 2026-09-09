use super::config::connection_timeout;
use isahc::prelude::*;
use isahc::{config::RedirectPolicy, AsyncBody, HttpClient, Request, Response};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub fn create_http_client(
    ca_cert: Option<&Path>,
    insecure: bool,
) -> Result<HttpClient, isahc::Error> {
    let mut builder = HttpClient::builder()
        .redirect_policy(RedirectPolicy::Follow)
        .timeout(connection_timeout());

    if insecure {
        eprintln!(
            "警告：TLS 证书验证已禁用。此连接不安全。"
        );
        builder = builder.ssl_options(
            isahc::config::SslOption::DANGER_ACCEPT_INVALID_CERTS
                | isahc::config::SslOption::DANGER_ACCEPT_INVALID_HOSTS,
        );
    } else if let Some(ca_path) = ca_cert {
        builder = builder.ssl_ca_certificate(isahc::config::CaCertificate::file(ca_path));
    }

    builder.build()
}

pub struct HttpClientWithCookies {
    client: HttpClient,
    cookies: Arc<Mutex<HashMap<String, String>>>,
}

impl HttpClientWithCookies {
    pub fn new(ca_cert: Option<&Path>, insecure: bool) -> Result<Self, isahc::Error> {
        Ok(Self {
            client: create_http_client(ca_cert, insecure)?,
            cookies: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub async fn send_with_cookies<T: Into<Request<Vec<u8>>>>(
        &self,
        request: T,
    ) -> Result<Response<AsyncBody>, isahc::Error> {
        let mut req = request.into();

        // 向请求添加 cookie
        if let Ok(cookies) = self.cookies.lock() {
            if !cookies.is_empty() {
                let cookie_header = cookies
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("; ");
                req.headers_mut()
                    .insert("cookie", cookie_header.parse().unwrap());
            }
        }

        let response = self.client.send_async(req).await?;

        // 从响应中提取并存储 cookie
        if let Some(set_cookie_headers) = response.headers().get_all("set-cookie").iter().next() {
            if let Ok(cookie_str) = set_cookie_headers.to_str() {
                self.parse_and_store_cookies(cookie_str);
            }
        }

        Ok(response)
    }

    fn parse_and_store_cookies(&self, cookie_header: &str) {
        if let Ok(mut cookies) = self.cookies.lock() {
            // 简单的 cookie 解析 — 仅提取 name=value 对
            for cookie_part in cookie_header.split(';') {
                let cookie_part = cookie_part.trim();
                if let Some((name, value)) = cookie_part.split_once('=') {
                    // 跳过 cookie 属性，如 Path、Domain、HttpOnly 等
                    if ![
                        "path", "domain", "httponly", "secure", "samesite", "expires", "max-age",
                    ]
                    .contains(&name.to_lowercase().as_str())
                    {
                        cookies.insert(name.trim().to_string(), value.trim().to_string());
                    }
                }
            }
        }
    }

    pub fn get_cookie_header(&self) -> Option<String> {
        if let Ok(cookies) = self.cookies.lock() {
            if !cookies.is_empty() {
                let cookie_header = cookies
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("; ");
                return Some(cookie_header);
            }
        }
        None
    }

    /// 提取特定的 cookie 值
    pub fn get_cookie(&self, name: &str) -> Option<String> {
        if let Ok(cookies) = self.cookies.lock() {
            return cookies.get(name).cloned();
        }
        None
    }

    /// 预填充 cookie（用于保存的会话令牌）
    pub fn set_cookie(&self, name: String, value: String) {
        if let Ok(mut cookies) = self.cookies.lock() {
            cookies.insert(name, value);
        }
    }
}
