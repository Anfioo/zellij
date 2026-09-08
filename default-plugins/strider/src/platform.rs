use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Unix,
    Windows,
}

impl Default for Platform {
    fn default() -> Self {
        Platform::Unix
    }
}

impl Platform {
    /// 从 initial_cwd 字符串检测主机平台。
    /// 检查盘符（如 `C:\` 或 `C:/`）或 UNC 路径（`\\server\`）。
    pub fn detect(initial_cwd: &str) -> Self {
        let bytes = initial_cwd.as_bytes();
        // 盘符：X:\ 或 X:/
        let is_drive_letter = matches!(
            (bytes.get(0), bytes.get(1), bytes.get(2)),
            (Some(c), Some(b':'), Some(b'\\' | b'/')) if c.is_ascii_alphabetic()
        );
        if is_drive_letter {
            return Platform::Windows;
        }
        // UNC 路径：\\server\share
        let is_unc_path = matches!((bytes.get(0), bytes.get(1)), (Some(b'\\'), Some(b'\\')));
        if is_unc_path {
            return Platform::Windows;
        }
        Platform::Unix
    }

    /// 将 `\` 替换为 `/`，以便 WASM PathBuf 能正确解析路径。
    /// 对于不包含反斜杠的路径不做任何操作。
    pub fn normalize(path: &Path) -> PathBuf {
        let s = path.to_string_lossy();
        if s.contains('\\') {
            PathBuf::from(s.replace('\\', "/"))
        } else {
            path.to_path_buf()
        }
    }

    /// 将内部的正斜杠路径转换回 Windows 显示用的原生反斜杠。
    /// 在 Unix 上保持不变。
    pub fn to_host_display(path: &Path, platform: Platform) -> String {
        let s = path.to_string_lossy();
        match platform {
            Platform::Windows => s.replace('/', "\\"),
            Platform::Unix => s.into_owned(),
        }
    }

    /// 主机平台的路径分隔符字符。
    pub fn separator(self) -> char {
        match self {
            Platform::Windows => '\\',
            Platform::Unix => '/',
        }
    }

    /// 确保像 `C:` 这样的裸盘符变为 `C:/`。
    /// WASM PathBuf 的 `parent()` 会去掉尾部斜杠，但在 Windows 上
    /// `C:` 表示 "C 盘上的当前目录"，而不是盘符根目录。
    pub fn ensure_drive_root(path: PathBuf, platform: Platform) -> PathBuf {
        if platform == Platform::Windows {
            let s = path.to_string_lossy();
            let bytes = s.as_bytes();
            let is_bare_drive = s.len() == 2
                && matches!(bytes.get(0), Some(c) if c.is_ascii_alphabetic())
                && matches!(bytes.get(1), Some(b':'));
            if is_bare_drive {
                return PathBuf::from(format!("{}/", s));
            }
        }
        path
    }

    /// 虚拟根目录条目的显示名称。
    /// `C:/` → `C:\`，`//wsl.localhost/Ubuntu/` → `Ubuntu (WSL)`，`/` → `/`。
    pub fn virtual_root_display_name(path: &Path, platform: Platform) -> String {
        let s = path.to_string_lossy();
        match platform {
            Platform::Windows => {
                if s.starts_with("//wsl.localhost/") {
                    let rest = &s["//wsl.localhost/".len()..];
                    let distro = rest.trim_end_matches('/');
                    return format!("{} (WSL)", distro);
                }
                Platform::to_host_display(path, platform)
            },
            Platform::Unix => s.into_owned(),
        }
    }

    /// 检查路径是否为文件系统根目录。
    /// Unix：`/` 或空。
    /// Windows：`X:/` 或 `X:`（盘符根），或 `//server/share`（组件数 <= 4 的 UNC 根）。
    pub fn is_root(path: &Path, platform: Platform) -> bool {
        let s = path.to_string_lossy();
        match platform {
            Platform::Unix => s == "/" || s.is_empty(),
            Platform::Windows => {
                let bytes = s.as_bytes();
                // 盘符根："C:/" 或 "C:"
                let is_drive_root_slash = s.len() == 3
                    && matches!(bytes.get(0), Some(c) if c.is_ascii_alphabetic())
                    && matches!(bytes.get(1), Some(b':'))
                    && matches!(bytes.get(2), Some(b'/'));
                if is_drive_root_slash {
                    return true;
                }
                let is_bare_drive = s.len() == 2
                    && matches!(bytes.get(0), Some(c) if c.is_ascii_alphabetic())
                    && matches!(bytes.get(1), Some(b':'));
                if is_bare_drive {
                    return true;
                }
                // UNC 根：//server/share（最多 4 个路径组件）
                if s.starts_with("//") {
                    let without_prefix = &s[2..];
                    let parts: Vec<&str> = without_prefix
                        .split('/')
                        .filter(|p| !p.is_empty())
                        .collect();
                    return parts.len() <= 2;
                }
                s.is_empty()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_unix() {
        assert_eq!(Platform::detect("/home/user"), Platform::Unix);
        assert_eq!(Platform::detect("/"), Platform::Unix);
    }

    #[test]
    fn detect_windows_drive() {
        assert_eq!(Platform::detect("C:\\Users\\user"), Platform::Windows);
        assert_eq!(Platform::detect("D:/Projects"), Platform::Windows);
    }

    #[test]
    fn detect_windows_unc() {
        assert_eq!(
            Platform::detect("\\\\wsl.localhost\\Ubuntu"),
            Platform::Windows
        );
    }

    #[test]
    fn normalize_backslashes() {
        let p = PathBuf::from("C:\\Users\\user");
        assert_eq!(Platform::normalize(&p), PathBuf::from("C:/Users/user"));
    }

    #[test]
    fn normalize_noop_unix() {
        let p = PathBuf::from("/home/user");
        assert_eq!(Platform::normalize(&p), PathBuf::from("/home/user"));
    }

    #[test]
    fn to_host_display_windows() {
        let p = PathBuf::from("C:/Users/user");
        assert_eq!(
            Platform::to_host_display(&p, Platform::Windows),
            "C:\\Users\\user"
        );
    }

    #[test]
    fn to_host_display_unix() {
        let p = PathBuf::from("/home/user");
        assert_eq!(Platform::to_host_display(&p, Platform::Unix), "/home/user");
    }

    #[test]
    fn is_root_unix() {
        assert!(Platform::is_root(&PathBuf::from("/"), Platform::Unix));
        assert!(Platform::is_root(&PathBuf::from(""), Platform::Unix));
        assert!(!Platform::is_root(&PathBuf::from("/home"), Platform::Unix));
    }

    #[test]
    fn is_root_windows_drive() {
        assert!(Platform::is_root(&PathBuf::from("C:/"), Platform::Windows));
        assert!(Platform::is_root(&PathBuf::from("C:"), Platform::Windows));
        assert!(!Platform::is_root(
            &PathBuf::from("C:/Users"),
            Platform::Windows
        ));
    }

    #[test]
    fn ensure_drive_root_fixes_bare_drive() {
        assert_eq!(
            Platform::ensure_drive_root(PathBuf::from("C:"), Platform::Windows),
            PathBuf::from("C:/")
        );
    }

    #[test]
    fn ensure_drive_root_noop_with_slash() {
        assert_eq!(
            Platform::ensure_drive_root(PathBuf::from("C:/"), Platform::Windows),
            PathBuf::from("C:/")
        );
    }

    #[test]
    fn ensure_drive_root_noop_unix() {
        assert_eq!(
            Platform::ensure_drive_root(PathBuf::from("/"), Platform::Unix),
            PathBuf::from("/")
        );
    }

    #[test]
    fn is_root_windows_unc() {
        assert!(Platform::is_root(
            &PathBuf::from("//server/share"),
            Platform::Windows
        ));
        assert!(!Platform::is_root(
            &PathBuf::from("//server/share/folder"),
            Platform::Windows
        ));
    }
}
