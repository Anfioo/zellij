//! Zellij 日志工具函数。

use std::{
    fs,
    io::{self, prelude::*},
    path::{Path, PathBuf},
};

use log::LevelFilter;

use log4rs::append::rolling_file::{
    policy::compound::{
        roll::fixed_window::FixedWindowRoller, trigger::size::SizeTrigger, CompoundPolicy,
    },
    RollingFileAppender,
};
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;

use crate::consts::{ZELLIJ_TMP_DIR, ZELLIJ_TMP_LOG_DIR, ZELLIJ_TMP_LOG_FILE};
use crate::shared::set_permissions;

const LOG_MAX_BYTES: u64 = 1024 * 1024 * 16; // 每个日志 16 MiB

pub fn configure_logger() {
    atomic_create_dir(&*ZELLIJ_TMP_DIR).unwrap();
    atomic_create_dir(&*ZELLIJ_TMP_LOG_DIR).unwrap();
    atomic_create_file(&*ZELLIJ_TMP_LOG_FILE).unwrap();

    let trigger = SizeTrigger::new(LOG_MAX_BYTES);
    let roller = FixedWindowRoller::builder()
        .build(
            ZELLIJ_TMP_LOG_DIR
                .join("zellij.log.old.{}")
                .to_str()
                .unwrap(),
            1,
        )
        .unwrap();

    // {n} 表示平台相关的换行符
    // module 被填充到恰好 25 字节，thread 被填充到 10 到 15 字节之间。
    let file_pattern = "{highlight({level:<6})} |{module:<25.25}| {date(%Y-%m-%d %H:%M:%S.%3f)} [{thread:<10.15}] {file}:{line}: {message} {n}";

    // 默认的 zellij 日志追加器，应在大部分代码库中使用。
    let log_file = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(file_pattern)))
        .build(
            &*ZELLIJ_TMP_LOG_FILE,
            Box::new(CompoundPolicy::new(
                Box::new(trigger),
                Box::new(roller.clone()),
            )),
        )
        .unwrap();

    // 插件日志追加器。用于 logging_pipe 转发插件的 stderr 输出。
    // 我们在 logging_pipe 中做了一些格式化，将插件名称打印为 'module'，将 plugin_id 打印为 thread。
    let log_plugin = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{highlight({level:<6})} {message} {n}",
        )))
        .build(
            &*ZELLIJ_TMP_LOG_FILE,
            Box::new(CompoundPolicy::new(Box::new(trigger), Box::new(roller))),
        )
        .unwrap();

    // 将默认日志级别设置为 "info"，并记录到 zellij.log 文件
    // 降低 `wasmtime_wasi` 模块的详细程度，因为它有很多无用的 info 日志
    // 对于 `zellij_server::logging_pipe`，我们使用自定义格式，因为使用日志宏来转发插件的 stderr 输出
    let config = Config::builder()
        .appender(Appender::builder().build("logFile", Box::new(log_file)))
        .appender(Appender::builder().build("logPlugin", Box::new(log_plugin)))
        // 降低 isahc 的详细程度，否则它会在每次失败的 web 请求时记录日志
        .logger(
            Logger::builder()
                .appender("logFile")
                .build("isahc", LevelFilter::Error),
        )
        .logger(
            Logger::builder()
                .appender("logPlugin")
                .build("wasmtime_wasi", LevelFilter::Warn),
        )
        .logger(
            Logger::builder()
                .appender("logPlugin")
                .additive(false)
                .build("zellij_server::logging_pipe", LevelFilter::Trace),
        )
        .build(Root::builder().appender("logFile").build(LevelFilter::Info))
        .unwrap();

    let _ = log4rs::init_config(config).unwrap();
}

pub fn atomic_create_file(file_name: &Path) -> io::Result<()> {
    let _ = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(file_name)?;
    set_permissions(file_name, 0o600)
}

pub fn atomic_create_dir(dir_name: &Path) -> io::Result<()> {
    let result = if let Err(e) = fs::create_dir(dir_name) {
        if e.kind() == std::io::ErrorKind::AlreadyExists {
            Ok(())
        } else {
            Err(e)
        }
    } else {
        Ok(())
    };
    if result.is_ok() {
        set_permissions(dir_name, 0o700)?;
    }
    result
}

pub fn debug_to_file(message: &[u8], terminal_id: i32) -> io::Result<()> {
    let mut path = PathBuf::new();
    path.push(&*ZELLIJ_TMP_LOG_DIR);
    path.push(format!("zellij-{}.log", terminal_id));

    let mut file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&path)?;
    set_permissions(&path, 0o600)?;
    file.write_all(message)
}
