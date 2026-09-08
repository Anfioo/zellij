//!
//! # 本模块包含访问本地系统路径所需的一切
//! 包括配置和布局

use crate::consts::{SYSTEM_DEFAULT_CONFIG_DIR, ZELLIJ_PROJ_DIR};

use std::{path::Path, path::PathBuf};

#[cfg(not(windows))]
use crate::home_unix as platform;
#[cfg(windows)]
use crate::home_windows as platform;

#[cfg(not(test))]
/// 遍历预定义列表，检查是否已存在配置目录，返回第一个匹配项
pub fn find_default_config_dir() -> Option<PathBuf> {
    default_config_dirs()
        .into_iter()
        .filter(|p| p.is_some())
        .find(|p| p.clone().unwrap().exists())
        .flatten()
}

#[cfg(test)]
pub fn find_default_config_dir() -> Option<PathBuf> {
    None
}

/// 检查配置目录的顺序
pub(crate) fn default_config_dirs() -> Vec<Option<PathBuf>> {
    vec![
        home_config_dir(),
        Some(xdg_config_dir()),
        Some(Path::new(SYSTEM_DEFAULT_CONFIG_DIR).to_path_buf()),
    ]
}

/// 查找已存在的目录并使用它，否则返回符合配置规范的目录。
pub fn get_default_data_dir() -> PathBuf {
    [xdg_data_dir(), platform::system_data_dir()]
        .into_iter()
        .find(|p| p.exists())
        .unwrap_or_else(xdg_data_dir)
}

pub fn xdg_config_dir() -> PathBuf {
    ZELLIJ_PROJ_DIR.config_dir().to_owned()
}

pub fn xdg_data_dir() -> PathBuf {
    ZELLIJ_PROJ_DIR.data_dir().to_owned()
}

pub fn home_config_dir() -> Option<PathBuf> {
    platform::home_config_dir()
}

pub fn try_create_home_config_dir() {
    platform::try_create_home_config_dir()
}

pub fn system_data_dir() -> PathBuf {
    platform::system_data_dir()
}

pub fn get_layout_dir(config_dir: Option<PathBuf>) -> Option<PathBuf> {
    config_dir.map(|dir| dir.join("layouts"))
}

pub fn default_layout_dir() -> Option<PathBuf> {
    find_default_config_dir().map(|dir| dir.join("layouts"))
}

pub fn get_theme_dir(config_dir: Option<PathBuf>) -> Option<PathBuf> {
    config_dir.map(|dir| dir.join("themes"))
}

pub fn default_theme_dir() -> Option<PathBuf> {
    find_default_config_dir().map(|dir| dir.join("themes"))
}
