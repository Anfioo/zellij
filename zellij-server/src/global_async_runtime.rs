use once_cell::sync::OnceCell;
use tokio::runtime::Runtime;

// 用于异步 I/O 操作的全局 tokio 运行时
// 在插件下载、计时器和操作完成跟踪之间共享
static TOKIO_RUNTIME: OnceCell<Runtime> = OnceCell::new();

pub fn get_tokio_runtime() -> &'static Runtime {
    TOKIO_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .thread_name("async-runtime")
            .enable_all()
            .build()
            .expect("创建 tokio 运行时失败")
    })
}
