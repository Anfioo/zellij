use isahc::prelude::*;
use isahc::{config::RedirectPolicy, HttpClient, Request};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tokio::{io::AsyncWriteExt as _, sync::Mutex};
use tokio_stream::StreamExt as _;
use tokio_util::compat::FuturesAsyncReadCompatExt as _;
use tokio_util::io::ReaderStream;
use url::Url;

const STREAM_BUFFER_SIZE_BYTES: usize = 65535;

#[derive(Error, Debug)]
pub enum DownloaderError {
    #[error("RequestError: {0}")]
    Request(#[from] isahc::Error),
    #[error("HttpError: {0}")]
    HttpError(#[from] isahc::http::Error),
    #[error("IoError: {0}")]
    Io(#[source] std::io::Error),
    #[error("StdIoError: {0}")]
    StdIoError(#[from] std::io::Error),
    #[error("File name cannot be found in URL: {0}")]
    NotFoundFileName(String),
    #[error("Failed to parse URL body: {0}")]
    InvalidUrlBody(String),
}

#[derive(Debug, Clone)]
pub struct Downloader {
    client: Option<HttpClient>,
    location: PathBuf,
    // 整体是 Arc/Mutex，因此 Downloader 是线程安全的，而
    // HashMap 的各个值也是 Arc/Mutex（Mutexi?），表示各个下载不应
    // 并发发生
    download_locks: Arc<Mutex<HashMap<String, Arc<Mutex<()>>>>>,
}

impl Default for Downloader {
    fn default() -> Self {
        Self {
            client: HttpClient::builder()
                // TODO: 超时？
                .redirect_policy(RedirectPolicy::Follow)
                .build()
                .ok(),
            location: PathBuf::from(""),
            download_locks: Default::default(),
        }
    }
}

impl Downloader {
    pub fn new(location: PathBuf) -> Self {
        Self {
            client: HttpClient::builder()
                // TODO: 超时？
                .redirect_policy(RedirectPolicy::Follow)
                .build()
                .ok(),
            location,
            download_locks: Default::default(),
        }
    }

    pub async fn download(
        &self,
        url: &str,
        file_name: Option<&str>,
    ) -> Result<(), DownloaderError> {
        let Some(client) = &self.client else {
            log::error!("未找到 HTTP 客户端，无法执行请求——这很可能是 isahc::HttpClient 配置错误");
            return Ok(());
        };
        let file_name = match file_name {
            Some(name) => name.to_string(),
            None => self.parse_name(url)?,
        };

        // 我们这样做是为了确保同一时刻只有一个特定 url 的下载在进行，
        // 否则下载会互相破坏（并且我们浪费大量系统资源）
        let download_lock = self.acquire_download_lock(&file_name).await;
        // 重要的是 _lock 保持在作用域内，否则它会被丢弃、锁会在
        // 下载完成之前被释放
        let _lock = download_lock.lock().await;

        let file_path = self.location.join(file_name.as_str());
        if file_path.exists() {
            log::debug!("文件已存在：{:?}", file_path);
            return Ok(());
        }
        let file_part_path = self.location.join(format!("{}.part", file_name));
        let (mut target, file_part_size) = {
            if file_part_path.exists() {
                let file_part = tokio::fs::OpenOptions::new()
                    .append(true)
                    .write(true)
                    .open(&file_part_path)
                    .await
                    .map_err(|e| DownloaderError::Io(e))?;

                let file_part_size = file_part
                    .metadata()
                    .await
                    .map_err(|e| DownloaderError::Io(e))?
                    .len();

                log::debug!("从 {} 字节处继续下载", file_part_size);

                (file_part, file_part_size)
            } else {
                let file_part = tokio::fs::File::create(&file_part_path)
                    .await
                    .map_err(|e| DownloaderError::Io(e))?;

                (file_part, 0)
            }
        };
        let request = Request::get(url)
            .header("Content-Type", "application/octet-stream")
            .header("Range", format!("bytes={}-", file_part_size))
            .body(())?;
        let mut res = client.send_async(request).await?;
        let body = res.body_mut();
        let mut stream = ReaderStream::with_capacity(body.compat(), STREAM_BUFFER_SIZE_BYTES);
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(DownloaderError::Io)?;
            target
                .write_all(&chunk)
                .await
                .map_err(DownloaderError::Io)?;
        }

        log::debug!("Download complete: {:?}", file_part_path);

        tokio::fs::rename(file_part_path, file_path)
            .await
            .map_err(|e| DownloaderError::Io(e))?;

        Ok(())
    }
    pub async fn download_without_cache(url: &str) -> Result<String, DownloaderError> {
        let request = Request::get(url)
            .header("Content-Type", "application/octet-stream")
            .body(())?;
        let client = HttpClient::builder()
            // TODO: 超时？
            .redirect_policy(RedirectPolicy::Follow)
            .build()?;

        let mut res = client.send_async(request).await?;

        let mut downloaded_bytes: Vec<u8> = vec![];
        let body = res.body_mut();
        let mut stream = ReaderStream::with_capacity(body.compat(), STREAM_BUFFER_SIZE_BYTES);
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(DownloaderError::Io)?;
            downloaded_bytes.extend_from_slice(&*chunk);
        }

        log::debug!("Download complete");
        let stringified = String::from_utf8(downloaded_bytes)
            .map_err(|e| DownloaderError::InvalidUrlBody(format!("{}", e)))?;

        Ok(stringified)
    }

    /// 下载 URL 的内容并阻塞等待结果。
    ///
    /// 包装对 [`download_without_cache`] 的 `async` 调用，使其可以用于同步
    /// 代码。这通过以下两种方式之一实现：
    ///
    /// 1. 若当前线程中已存在 async 运行时则复用之，或
    /// 2. 在当前线程上启动一个新的 async 运行时
    ///
    /// 如果两者都不可行，则返回错误。
    ///
    /// # 注意
    ///
    /// 目前，此函数仅用于弥合 async 的
    /// [`Downloader`] 实现与最终调用此函数的同步 [`Layout`] 代码之间的鸿沟。之所以
    /// 需要它，是因为 Layout 代码无法在不进行大量
    /// 重构的情况下轻易改为 `async`，而 Downloader 在许多其他使用 async 代码的地方被使用，且无法
    /// 合理地改为同步。也许将来这里更多的代码变成 async 后，我们可以去掉
    /// 这个函数。
    pub fn download_without_cache_blocking(url: &str) -> Result<String, DownloaderError> {
        let runtime_handle = match tokio::runtime::Handle::try_current() {
            Ok(handle) => handle.clone(),
            Err(e) if e.is_missing_context() => {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .thread_name("ephemeral runtime for downloader implementation")
                    .build()
                    .map_err(DownloaderError::Io)?;
                runtime.handle().clone()
            },
            _ => {
                return Err(DownloaderError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "failed to spawn runtime for download task",
                )))
            },
        };
        runtime_handle.block_on(async move { Downloader::download_without_cache(url).await })
    }

    fn parse_name(&self, url: &str) -> Result<String, DownloaderError> {
        Url::parse(url)
            .map_err(|_| DownloaderError::NotFoundFileName(url.to_string()))?
            .path_segments()
            .ok_or_else(|| DownloaderError::NotFoundFileName(url.to_string()))?
            .last()
            .ok_or_else(|| DownloaderError::NotFoundFileName(url.to_string()))
            .map(|s| s.to_string())
    }
    async fn acquire_download_lock(&self, file_name: &String) -> Arc<Mutex<()>> {
        let mut lock_dict = self.download_locks.lock().await;
        let download_lock = lock_dict
            .entry(file_name.clone())
            .or_insert_with(|| Default::default());
        download_lock.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::tempdir;

    #[ignore]
    #[tokio::test]
    async fn test_download_ok() {
        let location = tempdir().expect("创建临时目录失败");
        let location_path = location.path();

        let downloader = Downloader::new(location_path.to_path_buf());
        let result = downloader
            .download(
                "https://github.com/imsnif/monocle/releases/download/0.39.0/monocle.wasm",
                Some("monocle.wasm"),
            )
            .await
            .is_ok();

        assert!(result);
        assert!(location_path.join("monocle.wasm").exists());

        location.close().expect("关闭临时目录失败");
    }

    #[ignore]
    #[tokio::test]
    async fn test_download_without_file_name() {
        let location = tempdir().expect("创建临时目录失败");
        let location_path = location.path();

        let downloader = Downloader::new(location_path.to_path_buf());
        let result = downloader
            .download(
                "https://github.com/imsnif/multitask/releases/download/0.38.2v2/multitask.wasm",
                None,
            )
            .await
            .is_ok();

        assert!(result);
        assert!(location_path.join("multitask.wasm").exists());

        location.close().expect("关闭临时目录失败");
    }
}
