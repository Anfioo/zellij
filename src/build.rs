fn main() {
    // 由 clap 派生的 `augment_subcommands`（约 70 个变体）
    // 在 debug 模式下会产生 >1 MB 的栈帧，超出 Windows 默认的
    // 1 MB 主线程栈。将其增加到 8 MB 以与 Linux 保持一致。
    // Release 构建会优化掉该栈帧，因此仅非 release 配置需要此设置。
    if cfg!(target_os = "windows") && std::env::var("PROFILE").unwrap_or_default() != "release" {
        println!("cargo:rustc-link-arg=/STACK:8388608");
    }

    // 将应用程序图标嵌入 Windows 可执行文件。
    #[cfg(target_os = "windows")]
    let _ = embed_resource::compile("assets/zellij.rc", embed_resource::NONE);
}
