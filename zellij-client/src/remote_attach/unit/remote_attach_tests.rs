use super::super::*;
use crate::RemoteClientError;
use serial_test::serial;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use zellij_utils::remote_session_tokens;

// 模拟服务器基础设施
#[cfg(feature = "web_server_capability")]
mod mock_server {
    use super::*;
    use axum::{
        extract::State,
        http::StatusCode,
        response::Response,
        routing::{get, post},
        Json, Router,
    };
    use axum_extra::extract::cookie::{Cookie, CookieJar};
    use serde::Deserialize;
    use serde_json::json;
    use tokio::net::TcpListener;
    use uuid::Uuid;

    #[derive(Clone)]
    pub struct MockRemoteServerState {
        pub valid_auth_tokens: Arc<Mutex<HashMap<String, ()>>>,
        pub session_tokens: Arc<Mutex<HashMap<String, String>>>, // token -> web_client_id
        pub endpoints_called: Arc<Mutex<Vec<String>>>,
        pub query_strings: Arc<Mutex<HashMap<String, String>>>,
    }

    impl MockRemoteServerState {
        pub fn new() -> Self {
            Self {
                valid_auth_tokens: Arc::new(Mutex::new(HashMap::new())),
                session_tokens: Arc::new(Mutex::new(HashMap::new())),
                endpoints_called: Arc::new(Mutex::new(Vec::new())),
                query_strings: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        fn record_query(&self, endpoint: &str, query: Option<&str>) {
            self.query_strings
                .lock()
                .unwrap()
                .insert(endpoint.to_string(), query.unwrap_or("").to_string());
        }

        pub fn get_query_string(&self, endpoint: &str) -> Option<String> {
            self.query_strings.lock().unwrap().get(endpoint).cloned()
        }

        pub fn add_valid_token(&self, token: &str) {
            self.valid_auth_tokens
                .lock()
                .unwrap()
                .insert(token.to_string(), ());
        }

        fn record_endpoint(&self, endpoint: &str) {
            self.endpoints_called
                .lock()
                .unwrap()
                .push(endpoint.to_string());
        }

        pub fn get_endpoints_called(&self) -> Vec<String> {
            self.endpoints_called.lock().unwrap().clone()
        }
    }

    #[derive(Deserialize)]
    pub struct LoginRequest {
        pub auth_token: String,
    }

    pub async fn handle_login(
        State(state): State<MockRemoteServerState>,
        jar: CookieJar,
        Json(payload): Json<LoginRequest>,
    ) -> Result<(CookieJar, Json<serde_json::Value>), StatusCode> {
        state.record_endpoint("/command/login");

        let valid_tokens = state.valid_auth_tokens.lock().unwrap();
        if !valid_tokens.contains_key(&payload.auth_token) {
            return Err(StatusCode::UNAUTHORIZED);
        }
        drop(valid_tokens);

        // 始终创建会话令牌（始终设置 cookie）
        let session_token = Uuid::new_v4().to_string();
        let web_client_id = Uuid::new_v4().to_string();

        state
            .session_tokens
            .lock()
            .unwrap()
            .insert(session_token.clone(), web_client_id);

        let cookie = Cookie::build(("session_token", session_token))
            .path("/")
            .http_only(true)
            .build();
        let jar = jar.add(cookie);

        Ok((
            jar,
            Json(json!({
                "success": true,
                "message": "Login successful"
            })),
        ))
    }

    pub async fn handle_session(
        State(state): State<MockRemoteServerState>,
        uri: axum::http::Uri,
        jar: CookieJar,
    ) -> Result<Json<serde_json::Value>, StatusCode> {
        state.record_endpoint("/session");
        state.record_query("/session", uri.query());

        let session_token = jar
            .get("session_token")
            .map(|c| c.value().to_string())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let session_tokens = state.session_tokens.lock().unwrap();
        let web_client_id = session_tokens
            .get(&session_token)
            .ok_or(StatusCode::UNAUTHORIZED)?
            .clone();
        drop(session_tokens);

        Ok(Json(json!({
            "web_client_id": web_client_id,
            "is_read_only": false,
            "session_name": "session-name",
            "config": {
                "font": "Monospace",
                "theme": {},
                "cursor_blink": false,
                "mac_option_is_meta": false
            }
        })))
    }

    pub async fn handle_ws_terminal(
        ws: axum::extract::ws::WebSocketUpgrade,
        State(state): State<MockRemoteServerState>,
        uri: axum::http::Uri,
        jar: CookieJar,
    ) -> Result<Response, StatusCode> {
        state.record_endpoint("/ws/terminal");
        state.record_query("/ws/terminal", uri.query());

        // 验证会话令牌
        let session_token = jar
            .get("session_token")
            .map(|c| c.value().to_string())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let session_tokens = state.session_tokens.lock().unwrap();
        if !session_tokens.contains_key(&session_token) {
            return Err(StatusCode::UNAUTHORIZED);
        }
        drop(session_tokens);

        Ok(ws.on_upgrade(|socket| async move {
            // 基本的回显 WebSocket 处理器
            use axum::extract::ws::Message;
            use futures_util::{SinkExt, StreamExt};
            let (mut sender, mut receiver) = socket.split();

            while let Some(Ok(msg)) = receiver.next().await {
                if let Message::Text(text) = msg {
                    let _ = sender.send(Message::Text(text)).await;
                }
            }
        }))
    }

    pub async fn handle_ws_control(
        ws: axum::extract::ws::WebSocketUpgrade,
        State(state): State<MockRemoteServerState>,
        uri: axum::http::Uri,
        jar: CookieJar,
    ) -> Result<Response, StatusCode> {
        state.record_endpoint("/ws/control");
        state.record_query("/ws/control", uri.query());

        // 验证会话令牌
        let session_token = jar
            .get("session_token")
            .map(|c| c.value().to_string())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let session_tokens = state.session_tokens.lock().unwrap();
        if !session_tokens.contains_key(&session_token) {
            return Err(StatusCode::UNAUTHORIZED);
        }
        drop(session_tokens);

        Ok(ws.on_upgrade(|socket| async move {
            // 基本的回显 WebSocket 处理器
            use axum::extract::ws::Message;
            use futures_util::{SinkExt, StreamExt};
            let (mut sender, mut receiver) = socket.split();

            while let Some(Ok(msg)) = receiver.next().await {
                if let Message::Text(text) = msg {
                    let _ = sender.send(Message::Text(text)).await;
                }
            }
        }))
    }

    pub async fn start_mock_server(
        state: MockRemoteServerState,
    ) -> (u16, tokio::task::JoinHandle<()>) {
        let app = Router::new()
            .route("/command/login", post(handle_login))
            .route("/session", post(handle_session))
            .route("/ws/terminal", get(handle_ws_terminal))
            .route("/ws/terminal/{session_name}", get(handle_ws_terminal))
            .route("/ws/control", get(handle_ws_control))
            .with_state(state);

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let server_handle = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        // 等待服务器就绪
        tokio::time::sleep(Duration::from_millis(100)).await;

        (port, server_handle)
    }
}

#[cfg(feature = "web_server_capability")]
mod tls_mock_server {
    use super::mock_server::MockRemoteServerState;
    use axum::routing::{get, post};
    use axum::Router;
    use axum_server::tls_rustls::RustlsConfig;
    use axum_server::Handle;
    use std::net::SocketAddr;
    use std::path::PathBuf;
    use std::time::Duration;

    pub struct TlsTestCerts {
        pub ca_cert_path: PathBuf,
        _ca_cert_file: tempfile::NamedTempFile,
        _server_cert_file: tempfile::NamedTempFile,
        _server_key_file: tempfile::NamedTempFile,
        server_cert_path: PathBuf,
        server_key_path: PathBuf,
    }

    pub fn generate_test_certs() -> TlsTestCerts {
        // 创建具有正确密钥用法的 CA
        let ca_key = rcgen::KeyPair::generate().unwrap();
        let mut ca_params = rcgen::CertificateParams::new(Vec::<String>::new()).unwrap();
        ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        ca_params.key_usages = vec![
            rcgen::KeyUsagePurpose::KeyCertSign,
            rcgen::KeyUsagePurpose::CrlSign,
        ];
        let ca = ca_params.self_signed(&ca_key).unwrap();

        // 仅使用 IP SAN 创建服务器证书（IP 地址无 DNS 名称）
        let mut server_params = rcgen::CertificateParams::new(Vec::<String>::new()).unwrap();
        server_params.subject_alt_names = vec![rcgen::SanType::IpAddress(std::net::IpAddr::V4(
            std::net::Ipv4Addr::LOCALHOST,
        ))];
        server_params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ServerAuth];
        let server_key = rcgen::KeyPair::generate().unwrap();
        let ca_issuer = rcgen::Issuer::from_params(&ca_params, &ca_key);
        let server_cert = server_params.signed_by(&server_key, &ca_issuer).unwrap();

        // 写入临时文件
        let ca_cert_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(ca_cert_file.path(), ca.pem()).unwrap();

        let server_cert_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(server_cert_file.path(), server_cert.pem()).unwrap();

        let server_key_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(server_key_file.path(), server_key.serialize_pem()).unwrap();

        TlsTestCerts {
            ca_cert_path: ca_cert_file.path().to_path_buf(),
            server_cert_path: server_cert_file.path().to_path_buf(),
            server_key_path: server_key_file.path().to_path_buf(),
            _ca_cert_file: ca_cert_file,
            _server_cert_file: server_cert_file,
            _server_key_file: server_key_file,
        }
    }

    pub async fn start_tls_mock_server(
        state: MockRemoteServerState,
        certs: &TlsTestCerts,
    ) -> (u16, Handle<SocketAddr>, tokio::task::JoinHandle<()>) {
        let app = Router::new()
            .route("/command/login", post(super::mock_server::handle_login))
            .route("/session", post(super::mock_server::handle_session))
            .route("/ws/terminal", get(super::mock_server::handle_ws_terminal))
            .route(
                "/ws/terminal/{session_name}",
                get(super::mock_server::handle_ws_terminal),
            )
            .route("/ws/control", get(super::mock_server::handle_ws_control))
            .with_state(state);

        let rustls_config =
            RustlsConfig::from_pem_file(&certs.server_cert_path, &certs.server_key_path)
                .await
                .expect("Failed to load test TLS config");

        let listener =
            std::net::TcpListener::bind("127.0.0.1:0").expect("Failed to bind test TLS server");
        listener
            .set_nonblocking(true)
            .expect("Failed to set test TLS server listener to non-blocking");
        let port = listener.local_addr().unwrap().port();

        let handle = Handle::new();
        let server_handle = handle.clone();

        let server_task = tokio::spawn(async move {
            axum_server::from_tcp_rustls(listener, rustls_config)
                .unwrap()
                .handle(server_handle)
                .serve(app.into_make_service())
                .await
                .unwrap();
        });

        // 等待服务器开始监听（确定性，无睡眠）
        handle.listening().await;

        (port, handle, server_task)
    }

    pub async fn shutdown_server(
        handle: Handle<SocketAddr>,
        server_task: tokio::task::JoinHandle<()>,
    ) {
        handle.graceful_shutdown(Some(Duration::from_secs(1)));
        let _ = server_task.await;
    }
}

// 数据库测试辅助函数
fn setup_test_db(server_url: &str) {
    let _ = remote_session_tokens::delete_session_token(server_url);
}

fn cleanup_test_db(server_url: &str) {
    let _ = remote_session_tokens::delete_session_token(server_url);
}

// 用于测试的模拟 ClientOsApi
#[derive(Debug, Clone)]
struct MockClientOsApi;

impl crate::os_input_output::ClientOsApi for MockClientOsApi {
    fn get_terminal_size(&self) -> zellij_utils::pane_size::Size {
        zellij_utils::pane_size::Size { rows: 24, cols: 80 }
    }

    fn set_raw_mode(&mut self) {}

    fn unset_raw_mode(&self) -> Result<(), std::io::Error> {
        Ok(())
    }

    fn box_clone(&self) -> Box<dyn crate::os_input_output::ClientOsApi> {
        Box::new(MockClientOsApi)
    }

    fn read_from_stdin(&mut self) -> Result<Vec<u8>, &'static str> {
        Ok(Vec::new())
    }

    fn get_stdin_reader(&self) -> Box<dyn std::io::BufRead> {
        Box::new(std::io::BufReader::new(std::io::empty()))
    }

    fn get_stdout_writer(&self) -> Box<dyn std::io::Write> {
        Box::new(std::io::sink())
    }

    fn update_session_name(&mut self, _new_session_name: String) {}

    fn send_to_server(&self, _msg: zellij_utils::ipc::ClientToServerMsg) {}

    fn recv_from_server(
        &self,
    ) -> Option<(
        zellij_utils::ipc::ServerToClientMsg,
        zellij_utils::errors::ErrorContext,
    )> {
        None
    }

    fn handle_signals(
        &self,
        _sigwinch_cb: Box<dyn Fn()>,
        _quit_cb: Box<dyn Fn()>,
        _resize_receiver: Option<std::sync::mpsc::Receiver<()>>,
    ) {
    }

    fn connect_to_server(&self, _path: &std::path::Path) {}

    fn load_palette(&self) -> zellij_utils::data::Palette {
        zellij_utils::shared::default_palette()
    }

    fn enable_mouse(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn disable_mouse(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

// 测试
#[cfg(feature = "web_server_capability")]
mod tests {
    use super::mock_server::*;
    use super::*;

    // 从异步上下文调用 attach_to_remote_session 的辅助函数
    async fn call_attach_to_remote_session(
        remote_session_url: String,
        token: Option<String>,
        remember: bool,
        forget: bool,
    ) -> Result<WebSocketConnections, RemoteClientError> {
        tokio::task::spawn_blocking(move || {
            let runtime = crate::async_runtime(None);
            let os_input: Box<dyn crate::os_input_output::ClientOsApi> = Box::new(MockClientOsApi);
            attach_to_remote_session(
                runtime,
                os_input,
                &remote_session_url,
                token,
                remember,
                forget,
                None,
                true, // 测试时使用 insecure
            )
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    #[serial]
    async fn test_successful_authentication_with_valid_token() {
        let server_state = MockRemoteServerState::new();
        let auth_token = "test-auth-token-123";
        server_state.add_valid_token(auth_token);

        let (port, server_handle) = start_mock_server(server_state.clone()).await;
        let server_url = format!("http://127.0.0.1:{}/session-name", port);

        setup_test_db(&format!("http://127.0.0.1:{}", port));

        let result =
            call_attach_to_remote_session(server_url, Some(auth_token.to_string()), false, false)
                .await;

        assert!(
            result.is_ok(),
            "Should successfully authenticate: {:?}",
            result.err()
        );

        let endpoints = server_state.get_endpoints_called();
        assert!(
            endpoints.contains(&"/command/login".to_string()),
            "Should call login endpoint"
        );
        assert!(
            endpoints.contains(&"/session".to_string()),
            "Should call session endpoint"
        );
        assert!(
            endpoints.contains(&"/ws/terminal".to_string()),
            "Should establish terminal WebSocket"
        );
        assert!(
            endpoints.contains(&"/ws/control".to_string()),
            "Should establish control WebSocket"
        );

        let session_query = server_state
            .get_query_string("/session")
            .expect("session endpoint query recorded");
        assert_eq!(
            session_query, "session=session-name",
            "session endpoint must carry the requested session name"
        );

        let terminal_query = server_state
            .get_query_string("/ws/terminal")
            .expect("terminal socket query recorded");
        assert!(
            terminal_query.contains("web_client_id=")
                && terminal_query.contains("rows=")
                && terminal_query.contains("cols="),
            "terminal socket must carry the local terminal size, got: {}",
            terminal_query
        );

        let control_query = server_state
            .get_query_string("/ws/control")
            .expect("control socket query recorded");
        assert!(
            control_query.contains("web_client_id="),
            "control socket must carry the web_client_id, got: {}",
            control_query
        );

        server_handle.abort();
        cleanup_test_db(&format!("http://127.0.0.1:{}", port));
    }

    #[tokio::test]
    #[serial]
    async fn test_failed_authentication_with_invalid_token() {
        let server_state = MockRemoteServerState::new();
        // 不要将令牌添加到有效令牌中 — 服务器会拒绝它

        let (port, server_handle) = start_mock_server(server_state.clone()).await;
        let server_url = format!("http://127.0.0.1:{}/session-name", port);

        setup_test_db(&format!("http://127.0.0.1:{}", port));

        let result = call_attach_to_remote_session(
            server_url,
            Some("invalid-token".to_string()),
            false,
            false,
        )
        .await;

        assert!(result.is_err(), "Should fail with invalid token");
        assert!(
            matches!(result.unwrap_err(), RemoteClientError::InvalidAuthToken),
            "Should return InvalidAuthToken error"
        );

        server_handle.abort();
        cleanup_test_db(&format!("http://127.0.0.1:{}", port));
    }

    #[tokio::test]
    #[serial]
    async fn test_save_session_token_with_remember_true() {
        let server_state = MockRemoteServerState::new();
        let auth_token = "test-token-remember";
        server_state.add_valid_token(auth_token);

        let (port, server_handle) = start_mock_server(server_state.clone()).await;
        let server_url = format!("http://127.0.0.1:{}/session-name", port);
        let base_url = format!("http://127.0.0.1:{}", port);

        setup_test_db(&base_url);

        let result = call_attach_to_remote_session(
            server_url,
            Some(auth_token.to_string()),
            true, // remember = true
            false,
        )
        .await;

        assert!(result.is_ok(), "Connection should succeed");

        // 验证令牌已保存
        let saved_token = remote_session_tokens::get_session_token(&base_url);
        assert!(saved_token.is_ok());
        assert!(
            saved_token.unwrap().is_some(),
            "Session token should be saved"
        );

        server_handle.abort();
        cleanup_test_db(&base_url);
    }

    #[tokio::test]
    #[serial]
    async fn test_dont_save_token_with_remember_false() {
        let server_state = MockRemoteServerState::new();
        let auth_token = "test-token-no-remember";
        server_state.add_valid_token(auth_token);

        let (port, server_handle) = start_mock_server(server_state.clone()).await;
        let server_url = format!("http://127.0.0.1:{}/session-name", port);
        let base_url = format!("http://127.0.0.1:{}", port);

        setup_test_db(&base_url);

        let result = call_attach_to_remote_session(
            server_url,
            Some(auth_token.to_string()),
            false, // remember = false
            false,
        )
        .await;

        assert!(result.is_ok(), "Connection should succeed");

        // 验证令牌未保存
        let saved_token = remote_session_tokens::get_session_token(&base_url);
        assert!(saved_token.is_ok());
        assert!(
            saved_token.unwrap().is_none(),
            "Session token should NOT be saved"
        );

        server_handle.abort();
        cleanup_test_db(&base_url);
    }

    #[tokio::test]
    #[serial]
    async fn test_load_and_use_saved_session_token() {
        let server_state = MockRemoteServerState::new();

        // 预创建会话令牌
        let session_token = uuid::Uuid::new_v4().to_string();
        let web_client_id = uuid::Uuid::new_v4().to_string();
        server_state
            .session_tokens
            .lock()
            .unwrap()
            .insert(session_token.clone(), web_client_id);

        let (port, server_handle) = start_mock_server(server_state.clone()).await;
        let server_url = format!("http://127.0.0.1:{}/session-name", port);
        let base_url = format!("http://127.0.0.1:{}", port);

        setup_test_db(&base_url);

        // 保存会话令牌
        remote_session_tokens::save_session_token(&base_url, &session_token).unwrap();

        let result = call_attach_to_remote_session(
            server_url, None, // 未提供认证令牌
            false, false,
        )
        .await;

        assert!(result.is_ok(), "Should successfully use saved token");

        // 验证我们没有调用 login 端点（直接使用了保存的令牌）
        let endpoints = server_state.get_endpoints_called();
        assert!(
            !endpoints.contains(&"/command/login".to_string()),
            "Should NOT call login endpoint"
        );
        assert!(
            endpoints.contains(&"/session".to_string()),
            "Should call session endpoint"
        );

        server_handle.abort();
        cleanup_test_db(&base_url);
    }

    #[tokio::test]
    #[serial]
    async fn test_token_flag_deletes_saved_token() {
        let server_state = MockRemoteServerState::new();
        let auth_token = "new-auth-token";
        server_state.add_valid_token(auth_token);

        let (port, server_handle) = start_mock_server(server_state.clone()).await;
        let server_url = format!("http://127.0.0.1:{}/session-name", port);
        let base_url = format!("http://127.0.0.1:{}", port);

        setup_test_db(&base_url);

        // 预保存旧令牌
        remote_session_tokens::save_session_token(&base_url, "old-token").unwrap();

        let result = call_attach_to_remote_session(
            server_url,
            Some(auth_token.to_string()), // 提供新令牌
            false,
            false,
        )
        .await;

        assert!(result.is_ok(), "Should succeed with new token");

        // 在使用新令牌之前，旧令牌应该已被删除
        // （因为 remember=false，新令牌不会被保存）
        // 通过检查 session 端点被调用（未使用保存的令牌）来验证
        let endpoints = server_state.get_endpoints_called();
        assert!(
            endpoints.contains(&"/command/login".to_string()),
            "Should use new auth token, not saved token"
        );

        server_handle.abort();
        cleanup_test_db(&base_url);
    }

    #[tokio::test]
    #[serial]
    async fn test_successful_websocket_establishment() {
        let server_state = MockRemoteServerState::new();
        let auth_token = "test-ws-token";
        server_state.add_valid_token(auth_token);

        let (port, server_handle) = start_mock_server(server_state.clone()).await;
        let server_url = format!("http://127.0.0.1:{}/test-session", port);
        let base_url = format!("http://127.0.0.1:{}", port);

        setup_test_db(&base_url);

        let result =
            call_attach_to_remote_session(server_url, Some(auth_token.to_string()), false, false)
                .await;

        assert!(
            result.is_ok(),
            "WebSocket connections should be established"
        );

        let connections = result.unwrap();
        assert!(
            !connections.web_client_id.is_empty(),
            "Should have web_client_id"
        );

        // 验证两个 WebSocket 端点都被调用
        let endpoints = server_state.get_endpoints_called();
        assert!(
            endpoints.contains(&"/ws/terminal".to_string()),
            "Terminal WebSocket should be established"
        );
        assert!(
            endpoints.contains(&"/ws/control".to_string()),
            "Control WebSocket should be established"
        );

        server_handle.abort();
        cleanup_test_db(&base_url);
    }

    #[tokio::test]
    async fn test_url_parsing_for_session_name() {
        // 测试各种 URL 格式
        let test_cases = vec![
            ("https://example.com/my-session", "my-session"),
            ("https://example.com/", ""),
            ("https://example.com/path/to/session", "path/to/session"),
            ("http://localhost:8080/test", "test"),
        ];

        for (url, expected_name) in test_cases {
            let result = extract_session_name(url);
            assert!(result.is_ok(), "Failed to parse URL: {}", url);
            assert_eq!(
                result.unwrap(),
                expected_name,
                "Wrong session name for URL: {}",
                url
            );
        }
    }

    #[tokio::test]
    async fn test_server_url_extraction() {
        // 测试各种 URL 格式
        let test_cases = vec![
            (
                "https://example.com:8080/session?foo=bar",
                "https://example.com:8080",
            ),
            ("http://localhost/test", "http://localhost"),
            (
                "https://example.com/path/to/session#anchor",
                "https://example.com",
            ),
        ];

        for (url, expected_base) in test_cases {
            let result = extract_server_url(url);
            assert!(result.is_ok(), "Failed to extract server URL: {}", url);
            assert_eq!(
                result.unwrap(),
                expected_base,
                "Wrong base URL for: {}",
                url
            );
        }
    }

    #[tokio::test]
    async fn test_invalid_url_format() {
        let result = call_attach_to_remote_session(
            "not-a-valid-url".to_string(),
            Some("token".to_string()),
            false,
            false,
        )
        .await;

        assert!(result.is_err(), "Should fail with malformed URL");
        assert!(matches!(
            result.unwrap_err(),
            RemoteClientError::UrlParseError(_)
        ));
    }

    // -- TLS 测试 ------------------------------------------------------------
    //
    // 这些测试测试由 native-tls → rustls 迁移添加的 rustls WebSocket TLS 代码路径。
    // 它们直接调用 establish_websocket_connections，而不是通过 attach_to_remote_session，
    // 因为：
    //
    // 1. HTTP 认证步骤（isahc/curl）使用单独的 TLS 栈，该栈未被此迁移更改 —
    //    它由上面的非 TLS 测试测试。
    // 2. attach_to_remote_session 打开 SQLite 会话令牌数据库，
    //    这可能导致与在同一目录中使用不同 SQLite 数据库的 web_client 测试的 I/O 争用。
    //
    // 每个测试直接在模拟服务器状态中植入会话，并预填充 HTTP 客户端 cookie，
    // 然后通过 wss:// 连接。

    /// 辅助函数：创建带有预植入会话 cookie 的 HTTP 客户端，
    /// 并在模拟服务器状态中注册会话。返回 (web_client_id, http_client)。
    fn seed_mock_session(
        server_state: &MockRemoteServerState,
    ) -> (
        String,
        crate::remote_attach::http_client::HttpClientWithCookies,
    ) {
        let session_token = uuid::Uuid::new_v4().to_string();
        let web_client_id = uuid::Uuid::new_v4().to_string();
        server_state
            .session_tokens
            .lock()
            .unwrap()
            .insert(session_token.clone(), web_client_id.clone());

        // HTTP 客户端仅用于其 cookie jar（WebSocket 升级发送会话 cookie）。
        // 此客户端的 TLS 无关紧要，因为它在这些测试中从不发出 HTTP 请求。
        let http_client =
            crate::remote_attach::http_client::HttpClientWithCookies::new(None, true).unwrap();
        http_client.set_cookie("session_token".to_string(), session_token);

        (web_client_id, http_client)
    }

    #[tokio::test]
    #[serial]
    async fn test_tls_insecure_mode() {
        use crate::remote_attach::websockets;

        let certs = tls_mock_server::generate_test_certs();
        let server_state = MockRemoteServerState::new();

        let (port, handle, server_task) =
            tls_mock_server::start_tls_mock_server(server_state.clone(), &certs).await;

        let (web_client_id, http_client) = seed_mock_session(&server_state);
        let server_base_url = format!("https://127.0.0.1:{}", port);

        let result = tokio::time::timeout(
            Duration::from_secs(10),
            websockets::establish_websocket_connections(
                &web_client_id,
                &http_client,
                &server_base_url,
                "test-session",
                None,
                true, // insecure — exercises NoVerifier
            ),
        )
        .await
        .expect("Test timed out");

        assert!(
            result.is_ok(),
            "TLS insecure mode should connect successfully: {:?}",
            result.err()
        );

        let connections = result.unwrap();
        assert!(!connections.web_client_id.is_empty());

        let endpoints = server_state.get_endpoints_called();
        assert!(endpoints.contains(&"/ws/terminal".to_string()));
        assert!(endpoints.contains(&"/ws/control".to_string()));

        tls_mock_server::shutdown_server(handle, server_task).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_tls_ca_cert_mode() {
        use crate::remote_attach::websockets;

        let certs = tls_mock_server::generate_test_certs();
        let server_state = MockRemoteServerState::new();

        let (port, handle, server_task) =
            tls_mock_server::start_tls_mock_server(server_state.clone(), &certs).await;

        let (web_client_id, http_client) = seed_mock_session(&server_state);
        let server_base_url = format!("https://127.0.0.1:{}", port);

        let result = tokio::time::timeout(
            Duration::from_secs(10),
            websockets::establish_websocket_connections(
                &web_client_id,
                &http_client,
                &server_base_url,
                "test-session",
                Some(certs.ca_cert_path.as_path()),
                false, // not insecure — verify against CA cert
            ),
        )
        .await
        .expect("Test timed out");

        assert!(
            result.is_ok(),
            "WebSocket TLS with CA cert should connect successfully: {:?}",
            result.err()
        );

        let connections = result.unwrap();
        assert!(!connections.web_client_id.is_empty());

        let endpoints = server_state.get_endpoints_called();
        assert!(endpoints.contains(&"/ws/terminal".to_string()));
        assert!(endpoints.contains(&"/ws/control".to_string()));

        tls_mock_server::shutdown_server(handle, server_task).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_tls_rejects_untrusted_cert() {
        use crate::remote_attach::websockets;

        let certs = tls_mock_server::generate_test_certs();
        let server_state = MockRemoteServerState::new();

        let (port, handle, server_task) =
            tls_mock_server::start_tls_mock_server(server_state.clone(), &certs).await;

        let (web_client_id, http_client) = seed_mock_session(&server_state);
        let server_base_url = format!("https://127.0.0.1:{}", port);

        let result = tokio::time::timeout(
            Duration::from_secs(10),
            websockets::establish_websocket_connections(
                &web_client_id,
                &http_client,
                &server_base_url,
                "test-session",
                None,  // no CA cert
                false, // not insecure — should reject self-signed
            ),
        )
        .await
        .expect("Test timed out");

        assert!(
            result.is_err(),
            "TLS without CA cert should reject self-signed server"
        );

        tls_mock_server::shutdown_server(handle, server_task).await;
    }
}

// Tests that don't require the web_server_capability feature
#[cfg(not(feature = "web_server_capability"))]
mod tests {
    use super::*;

    #[test]
    fn test_url_parsing_without_server() {
        // Basic URL parsing tests that don't require a server
        let result = extract_session_name("https://example.com/my-session");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "my-session");

        let result = extract_server_url("https://example.com:8080/session?foo=bar");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://example.com:8080");
    }
}
