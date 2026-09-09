use std::path::PathBuf;

pub(crate) fn home_config_dir() -> Option<PathBuf> {
    // 在 Windows 上没有 ~/.config 约定。
    // 直接返回 ProjectDirs 的 config_dir（Roaming AppData）。
    Some(crate::home::xdg_config_dir())
}

pub(crate) fn try_create_home_config_dir() {
    let config_dir = crate::home::xdg_config_dir();
    if let Err(e) = std::fs::create_dir_all(config_dir) {
        log::error!("创建配置目录失败：{:?}", e);
    }
}

/// 系统级数据目录（`C:\ProgramData\Zellij\data`）。
pub(crate) fn system_data_dir() -> PathBuf {
    use crate::consts::SYSTEM_DEFAULT_DATA_DIR_PREFIX;
    std::path::Path::new(SYSTEM_DEFAULT_DATA_DIR_PREFIX).join("data")
}
