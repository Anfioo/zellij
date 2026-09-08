//! Plugins configuration metadata
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

use serde::{Deserialize, Serialize};
use url::Url;

use super::layout::{PluginUserConfiguration, RunPlugin, RunPluginLocation};
#[cfg(not(target_family = "wasm"))]
use crate::consts::ASSET_MAP;
use crate::consts::BUILTIN_PLUGIN_NAMES;
pub use crate::data::PluginTag;
use crate::errors::prelude::*;

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct PluginAliases {
    pub aliases: BTreeMap<String, RunPlugin>,
}

impl PluginAliases {
    pub fn merge(&mut self, other: Self) {
        self.aliases.extend(other.aliases);
    }
    pub fn from_data(aliases: BTreeMap<String, RunPlugin>) -> Self {
        PluginAliases { aliases }
    }
    pub fn list(&self) -> Vec<String> {
        self.aliases.keys().cloned().collect()
    }
}

/// Plugin metadata
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct PluginConfig {
    /// Path of the plugin, see resolve_wasm_bytes for resolution semantics
    pub path: PathBuf,
    /// Allow command execution from plugin
    pub _allow_exec_host_cmd: bool,
    /// Original location of the
    pub location: RunPluginLocation,
    /// Custom configuration for this plugin
    pub initial_userspace_configuration: PluginUserConfiguration,
    /// plugin initial working directory
    pub initial_cwd: Option<PathBuf>,
}

impl PluginConfig {
    pub fn from_run_plugin(run_plugin: &RunPlugin) -> Option<PluginConfig> {
        match &run_plugin.location {
            RunPluginLocation::File(path) => Some(PluginConfig {
                path: path.clone(),
                _allow_exec_host_cmd: run_plugin._allow_exec_host_cmd,
                location: run_plugin.location.clone(),
                initial_userspace_configuration: run_plugin.configuration.clone(),
                initial_cwd: run_plugin.initial_cwd.clone(),
            }),
            RunPluginLocation::Zellij(tag) => {
                let tag = tag.to_string();
                if BUILTIN_PLUGIN_NAMES.contains(&tag.as_str()) {
                    Some(PluginConfig {
                        path: PathBuf::from(&tag),
                        _allow_exec_host_cmd: run_plugin._allow_exec_host_cmd,
                        location: RunPluginLocation::parse(&format!("zellij:{}", tag), None)
                            .ok()?,
                        initial_userspace_configuration: run_plugin.configuration.clone(),
                        initial_cwd: run_plugin.initial_cwd.clone(),
                    })
                } else {
                    None
                }
            },
            RunPluginLocation::Remote(_) => Some(PluginConfig {
                path: PathBuf::new(),
                _allow_exec_host_cmd: run_plugin._allow_exec_host_cmd,
                location: run_plugin.location.clone(),
                initial_userspace_configuration: run_plugin.configuration.clone(),
                initial_cwd: run_plugin.initial_cwd.clone(),
            }),
        }
    }
    /// Resolve wasm plugin bytes for the plugin path and given plugin directory.
    ///
    /// If zellij was built without the 'disable_automatic_asset_installation' feature, builtin
    /// plugins (Starting with 'zellij:' in the layout file) are loaded directly from the
    /// binary-internal asset map. Otherwise:
    ///
    /// Attempts to first resolve the plugin path as an absolute path, then adds a ".wasm"
    /// extension to the path and resolves that, then the plugin directory joined with the path
    /// with an appended ".wasm" extension, and finally the system data directory joined with
    /// "plugins" and the same file name. So if our path is "tab-bar" and the given plugin dir is
    /// "/home/bob/.local/share/zellij/plugins" the lookup chain will be this:
    ///
    /// ```bash
    ///   tab-bar
    ///   tab-bar.wasm
    ///   /home/bob/.local/share/zellij/plugins/tab-bar.wasm
    ///   /usr/share/zellij/plugins/tab-bar.wasm
    /// ```
    ///
    pub fn resolve_wasm_bytes(&self, plugin_dir: &Path) -> Result<Vec<u8>> {
        let err_context =
            |err: std::io::Error, path: &PathBuf| format!("{}: '{}'", err, path.display());

        // 我们检查有效插件的位置
        #[allow(unused_mut)]
        let mut paths: Vec<PathBuf> = vec![
            self.path.clone(),
            self.path.with_extension("wasm"),
            plugin_dir.join(&self.path).with_extension("wasm"),
        ];
        #[cfg(not(target_family = "wasm"))]
        paths.push(
            crate::home::system_data_dir()
                .join("plugins")
                .join(&self.path)
                .with_extension("wasm"),
        );
        // 去重，因为读到 zellij 多次检查同一个插件会令人困惑
        // 位置多次。不要在这里对向量排序，因为这会破坏查找！
        paths.dedup();

        // 这看起来很奇怪，通常我们会以不同方式处理这样的错误，但在这种
        // 情况下它对用户和开发者都有帮助。这样我们保留所有查找
        // 错误并可以全部报告回来。我们必须用某些东西初始化 `last_err`，
        // 而且由于用户只有在加载插件失败时才会看到它，我们不妨
        // spell it out right here.
        let mut last_err: Result<Vec<u8>> = Err(anyhow!("failed to load plugin from disk"));
        for path in paths {
            // 检查插件路径是否与资源映射表中的条目匹配。如果是，直接从内存加载，
            // 从内存，不用麻烦磁盘。
            #[cfg(not(target_family = "wasm"))]
            if !cfg!(feature = "disable_automatic_asset_installation") && self.is_builtin() {
                let asset_path = PathBuf::from("plugins").join(&path);
                if let Some(bytes) = ASSET_MAP.get(&asset_path) {
                    log::debug!("Loaded plugin '{}' from internal assets", path.display());

                    if plugin_dir.join(&path).with_extension("wasm").exists() {
                        log::info!(
                            "Plugin '{}' exists in the 'PLUGIN DIR' at '{}' but is being ignored",
                            path.display(),
                            plugin_dir.display()
                        );
                    }

                    return Ok(bytes.to_vec());
                }
            }

            // 尝试从磁盘读取
            match fs::read(&path) {
                Ok(val) => {
                    log::debug!("Loaded plugin '{}' from disk", path.display());
                    return Ok(val);
                },
                Err(err) => {
                    last_err = last_err.with_context(|| err_context(err, &path));
                },
            }
        }

        // 如果找到插件则不会到达！
        #[cfg(not(target_family = "wasm"))]
        if self.is_builtin() {
            // 布局请求了一个未找到的内置插件
            let plugin_path = self.path.with_extension("wasm");

            if cfg!(feature = "disable_automatic_asset_installation") && self.is_builtin_name() {
                return Err(ZellijError::BuiltinPluginMissing {
                    plugin_path,
                    plugin_dir: plugin_dir.to_owned(),
                    source: last_err.unwrap_err(),
                })
                .context("failed to load a plugin");
            } else {
                return Err(ZellijError::BuiltinPluginNonexistent {
                    plugin_path,
                    source: last_err.unwrap_err(),
                })
                .context("failed to load a plugin");
            }
        }

        return last_err;
    }

    pub fn is_builtin(&self) -> bool {
        matches!(self.location, RunPluginLocation::Zellij(_))
    }

    pub fn is_builtin_name(&self) -> bool {
        self.path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .map(|name| BUILTIN_PLUGIN_NAMES.contains(&name))
            .unwrap_or(false)
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum PluginsConfigError {
    #[error("Duplication in plugin tag names is not allowed: '{}'", String::from(.0.clone()))]
    DuplicatePlugins(PluginTag),
    #[error("Failed to parse url: {0:?}")]
    InvalidUrl(#[from] url::ParseError),
    #[error("Only 'file:', 'http(s):' and 'zellij:' url schemes are supported for plugin lookup. '{0}' does not match either.")]
    InvalidUrlScheme(Url),
    #[error("Could not find plugin at the path: '{0:?}'")]
    InvalidPluginLocation(PathBuf),
}
