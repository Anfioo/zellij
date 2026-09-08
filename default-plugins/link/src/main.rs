use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use zellij_tile::prelude::*;

const FILE_PATH_REGEX: &str = r#"(?:^|\s)((?:(?:\./|\.\./|/)[A-Za-z0-9_./\-+@%,#=~!\$\{\}\[\]]+|~/[A-Za-z0-9_./\-+@%,#=~!\$\{\}\[\]]+|\$\{?[A-Za-z_][A-Za-z0-9_]*\}?/[A-Za-z0-9_./\-+@%,#=~!\$\{\}\[\]]+)(?::\d+(?::\d+)?)?)(?::|\s|$)"#;

const CWD_CONTEXT_KEY: &str = "cwd";

#[derive(Default)]
struct State {
    known_terminal_panes: HashSet<PaneId>,
    /// 跟踪每个终端窗格的当前工作目录（CWD）。
    pane_cwds: HashMap<PaneId, PathBuf>,
    /// 跟踪每个窗格中高亮的目录条目名称，
    /// 以便在 CWD 变化时可以移除它们。
    pane_dir_entries: HashMap<PaneId, Vec<String>>,
    /// 会话环境变量，在加载时获取一次。
    /// 用于在点击的路径中展开 `~` 和 `$VAR`。
    env_vars: BTreeMap<String, String>,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        subscribe(&[
            EventType::PaneUpdate,
            EventType::HighlightClicked,
            EventType::CwdChanged,
        ]);
        // 将主机文件夹设置为 "/"，以便 /host 映射到真实的文件系统根目录，
        // 从而允许在 /host/<absolute_path> 上执行 std::fs 操作。
        change_host_folder(PathBuf::from("/"));
        self.env_vars = get_session_environment_variables();
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::PaneUpdate(pane_manifest) => {
                self.handle_pane_update(pane_manifest);
            },
            Event::HighlightClicked {
                pane_id: _,
                pattern: _,
                matched_string,
                context,
            } => {
                self.handle_highlight_clicked(matched_string, context);
            },
            Event::CwdChanged(pane_id, new_cwd, _focused_client_ids) => {
                self.handle_cwd_changed(pane_id, new_cwd);
            },
            _ => {},
        }
        false // 从不渲染 —— 仅后台运行的插件
    }

    fn render(&mut self, _rows: usize, _cols: usize) {
        // 仅后台运行的插件。从不渲染。故意留空。
    }
}

impl State {
    fn handle_pane_update(&mut self, pane_manifest: PaneManifest) {
        let mut current_panes: HashSet<PaneId> = HashSet::new();

        for (_tab_index, panes) in &pane_manifest.panes {
            for pane_info in panes {
                if !pane_info.is_plugin {
                    let pane_id = PaneId::Terminal(pane_info.id);
                    current_panes.insert(pane_id);
                }
            }
        }

        // 在新出现的终端窗格上设置高亮
        for &pane_id in &current_panes {
            if !self.known_terminal_panes.contains(&pane_id) {
                // 获取窗格的当前 CWD 并扫描其目录
                if let Ok(cwd) = get_pane_cwd(pane_id) {
                    self.scan_and_store_dir_entries(pane_id, &cwd);
                    self.pane_cwds.insert(pane_id, cwd);
                }
                self.set_all_highlights_for_pane(pane_id);
            }
        }

        // 清理已不存在的窗格的跟踪状态
        for &pane_id in &self.known_terminal_panes {
            if !current_panes.contains(&pane_id) {
                self.pane_cwds.remove(&pane_id);
                self.pane_dir_entries.remove(&pane_id);
            }
        }

        self.known_terminal_panes = current_panes;
    }

    fn handle_cwd_changed(&mut self, pane_id: PaneId, new_cwd: PathBuf) {
        let old_cwd = self.pane_cwds.get(&pane_id);
        if old_cwd == Some(&new_cwd) {
            return;
        }

        self.pane_cwds.insert(pane_id, new_cwd.clone());
        self.scan_and_store_dir_entries(pane_id, &new_cwd);

        // clear_pane_highlights 移除所有高亮，然后重新设置全部
        // （文件路径正则 + 目录条目模式）
        clear_pane_highlights(pane_id);
        self.set_all_highlights_for_pane(pane_id);
    }

    fn handle_highlight_clicked(&self, matched_string: String, context: BTreeMap<String, String>) {
        let (path_str, line_number) = parse_path_and_line(&matched_string);
        let path_str = path_str.trim();
        let expanded = expand_path(path_str, &self.env_vars);
        let path_str = expanded.as_str();

        // 解析为完全限定路径：如果是相对路径，则与高亮上下文中存储的
        // 窗格 CWD 拼接。
        let absolute_path = if path_str.starts_with('/') {
            PathBuf::from(path_str)
        } else if let Some(cwd) = context.get(CWD_CONTEXT_KEY) {
            PathBuf::from(cwd).join(path_str)
        } else {
            PathBuf::from(path_str)
        };

        // 通过加载时建立的 /host/ 文件系统映射验证路径是否存在。
        // 这可以防止正则表达式误匹配终端输出中的非路径文本。
        let host_path =
            Path::new("/host").join(absolute_path.strip_prefix("/").unwrap_or(&absolute_path));
        let metadata = match std::fs::metadata(&host_path) {
            Ok(m) => m,
            Err(_) => return, // 路径不存在 —— 静默忽略
        };

        if metadata.is_dir() {
            let mut args = BTreeMap::new();
            let mut configuration = BTreeMap::new();
            args.insert("open_directly".to_owned(), "true".to_owned());
            configuration.insert("caller_cwd".to_owned(), absolute_path.display().to_string());
            pipe_message_to_plugin(
                MessageToPlugin::new("filepicker")
                    .with_plugin_url("filepicker")
                    .new_plugin_instance_should_have_pane_title(&format!(
                        "Browse: {}",
                        absolute_path.display()
                    ))
                    .new_plugin_instance_should_be_focused()
                    .new_plugin_instance_should_have_cwd(absolute_path)
                    .with_args(args)
                    .with_plugin_config(configuration),
            );
        } else {
            let mut file_to_open = FileToOpen::new(&absolute_path);
            if let Some(line) = line_number {
                file_to_open = file_to_open.with_line_number(line);
            }
            open_file_floating(file_to_open, None, BTreeMap::new());
        }
    }

    /// （重新）设置窗格的所有正则高亮：通用文件路径正则
    /// 加上从窗格 CWD 派生的任何目录条目模式。
    fn set_all_highlights_for_pane(&self, pane_id: PaneId) {
        let mut highlights = Vec::new();

        // 构建包含窗格 CWD 的上下文映射（点击时回传）
        let context = self.cwd_context_for_pane(pane_id);

        // 通用文件路径正则（始终存在）
        highlights.push(RegexHighlight {
            pattern: FILE_PATH_REGEX.to_owned(),
            style: HighlightStyle::None,
            layer: HighlightLayer::Hint,
            context: context.clone(),
            on_hover: true,
            bold: false,
            italic: true,
            underline: true,
            tooltip_text: Some("Open".to_string()),
        });

        // 窗格当前 CWD 的目录条目模式
        if let Some(entries) = self.pane_dir_entries.get(&pane_id) {
            for entry_name in entries {
                let path_chars = r#"[A-Za-z0-9_./\-+@%,#=~!\$\{\}\[\]]"#;
                let pattern = format!(
                    "(?:^|\\s)({}(?:/{path_chars}+)?(?::\\d+(?::\\d+)?)?)(?::|\\s|$)",
                    regex_escape(entry_name),
                );
                highlights.push(RegexHighlight {
                    pattern,
                    style: HighlightStyle::None,
                    layer: HighlightLayer::Hint,
                    context: context.clone(),
                    on_hover: true,
                    bold: false,
                    italic: true,
                    underline: true,
                    tooltip_text: Some("Open".to_string()),
                });
            }
        }

        set_pane_regex_highlights(pane_id, highlights);
    }

    fn scan_and_store_dir_entries(&mut self, pane_id: PaneId, cwd: &Path) {
        let host_path = Path::new("/host").join(cwd.strip_prefix("/").unwrap_or(cwd));
        let dir_entries = scan_directory(&host_path);
        self.pane_dir_entries.insert(pane_id, dir_entries);
    }

    fn cwd_context_for_pane(&self, pane_id: PaneId) -> BTreeMap<String, String> {
        let mut context = BTreeMap::new();
        if let Some(cwd) = self.pane_cwds.get(&pane_id) {
            context.insert(CWD_CONTEXT_KEY.to_owned(), cwd.display().to_string());
        }
        context
    }
}

/// 扫描目录中的一级文件和文件夹名称。
/// 任何错误下返回空 vec。
/// 要扫描的最大目录条目数。条目数超过此值的目录将被完全跳过，
/// 以避免过多的正则模式数量。
const MAX_DIR_ENTRIES: usize = 500;

fn scan_directory(path: &Path) -> Vec<String> {
    let mut entries = Vec::new();
    let read_dir = match std::fs::read_dir(path) {
        Ok(rd) => rd,
        Err(_) => return entries,
    };
    for entry in read_dir {
        if let Ok(entry) = entry {
            if let Some(name) = entry.file_name().to_str() {
                entries.push(name.to_owned());
                if entries.len() > MAX_DIR_ENTRIES {
                    return Vec::new();
                }
            }
        }
    }
    entries
}

/// 转义字符串，使其在正则模式中被视为字面量。
fn regex_escape(s: &str) -> String {
    let mut escaped = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '\\' | '.' | '+' | '*' | '?' | '(' | ')' | '|' | '[' | ']' | '{' | '}' | '^' | '$' => {
                escaped.push('\\');
                escaped.push(c);
            },
            _ => escaped.push(c),
        }
    }
    escaped
}

/// 展开路径字符串中的 `~` 和 `$VAR` / `${VAR}` 引用。
///
/// - 开头的 `~`（后跟 `/` 或在字符串末尾）将被替换为
///   `env_vars` 中 `HOME` 的值。
/// - 字符串中任意位置的 `$VARNAME` 和 `${VARNAME}` 将被替换为
///   `env_vars` 中的对应值。
/// - 无法识别的变量和缺失的 `HOME` 保持原样。
fn expand_path(path: &str, env_vars: &BTreeMap<String, String>) -> String {
    // 步骤 1：波浪号展开（仅前导 ~）
    let after_tilde = if path == "~" {
        match env_vars.get("HOME") {
            Some(home) => home.clone(),
            None => path.to_owned(),
        }
    } else if let Some(rest) = path.strip_prefix("~/") {
        match env_vars.get("HOME") {
            Some(home) => format!("{}/{}", home, rest),
            None => path.to_owned(),
        }
    } else {
        path.to_owned()
    };

    // 步骤 2：环境变量展开（$VAR 和 ${VAR}）
    let bytes = after_tilde.as_bytes();
    let len = bytes.len();
    let mut result = String::with_capacity(len);
    let mut i = 0;

    while i < len {
        if bytes[i] == b'$' && i + 1 < len {
            let (var_name, end_idx) = if bytes[i + 1] == b'{' {
                // ${VAR} 形式
                if let Some(close) = after_tilde[i + 2..].find('}') {
                    let name = &after_tilde[i + 2..i + 2 + close];
                    (name, i + 2 + close + 1)
                } else {
                    // 没有闭合大括号 —— 不是有效的变量引用
                    result.push('$');
                    i += 1;
                    continue;
                }
            } else {
                // $VAR 形式 —— 变量名为 [A-Za-z_][A-Za-z0-9_]*
                let start = i + 1;
                if start < len && (bytes[start].is_ascii_alphabetic() || bytes[start] == b'_') {
                    let mut end = start + 1;
                    while end < len && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
                        end += 1;
                    }
                    (&after_tilde[start..end], end)
                } else {
                    result.push('$');
                    i += 1;
                    continue;
                }
            };

            if let Some(value) = env_vars.get(var_name) {
                result.push_str(value);
            } else {
                // 未知变量 —— 保留原始文本
                result.push_str(&after_tilde[i..end_idx]);
            }
            i = end_idx;
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }

    result
}

fn parse_path_and_line(matched_string: &str) -> (&str, Option<usize>) {
    let mut end = matched_string.len();
    let mut numeric_segments: Vec<(usize, &str)> = Vec::new();

    loop {
        if end == 0 {
            break;
        }
        let search_region = &matched_string[..end];
        if let Some(colon_pos) = search_region.rfind(':') {
            let segment = &matched_string[colon_pos + 1..end];
            if !segment.is_empty() && segment.chars().all(|c| c.is_ascii_digit()) {
                numeric_segments.push((colon_pos, segment));
                end = colon_pos;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    numeric_segments.reverse();

    match numeric_segments.first() {
        None => (matched_string, None),
        Some(&(colon_pos, line_str)) => {
            let path = &matched_string[..colon_pos];
            (path, line_str.parse::<usize>().ok())
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- parse_path_and_line 测试 ---

    #[test]
    fn parse_path_and_line_simple_path() {
        let (path, line) = parse_path_and_line("src/main.rs");
        assert_eq!(path, "src/main.rs");
        assert_eq!(line, None);
    }

    #[test]
    fn parse_path_and_line_with_line_number() {
        let (path, line) = parse_path_and_line("src/main.rs:42");
        assert_eq!(path, "src/main.rs");
        assert_eq!(line, Some(42));
    }

    #[test]
    fn parse_path_and_line_with_line_and_col() {
        let (path, line) = parse_path_and_line("src/main.rs:42:10");
        assert_eq!(path, "src/main.rs");
        assert_eq!(line, Some(42));
    }

    #[test]
    fn parse_path_and_line_no_trailing_number() {
        let (path, line) = parse_path_and_line("src/main.rs:");
        assert_eq!(path, "src/main.rs:");
        assert_eq!(line, None);
    }

    // --- expand_path 测试 ---

    #[test]
    fn expand_path_tilde() {
        let mut env = BTreeMap::new();
        env.insert("HOME".into(), "/home/user".into());
        assert_eq!(expand_path("~/foo/bar", &env), "/home/user/foo/bar");
    }

    #[test]
    fn expand_path_env_var() {
        let mut env = BTreeMap::new();
        env.insert("HOME".into(), "/home/user".into());
        assert_eq!(expand_path("$HOME/foo", &env), "/home/user/foo");
    }

    #[test]
    fn expand_path_braced_var() {
        let mut env = BTreeMap::new();
        env.insert("HOME".into(), "/home/user".into());
        assert_eq!(expand_path("${HOME}/foo", &env), "/home/user/foo");
    }

    #[test]
    fn expand_path_unknown_var_preserved() {
        let env = BTreeMap::new();
        assert_eq!(expand_path("$UNKNOWN/foo", &env), "$UNKNOWN/foo");
    }

    #[test]
    fn expand_path_no_expansion_needed() {
        let env = BTreeMap::new();
        assert_eq!(expand_path("/absolute/path", &env), "/absolute/path");
    }

    // --- regex_escape 测试 ---

    #[test]
    fn regex_escape_special_chars() {
        assert_eq!(regex_escape("file.txt"), r"file\.txt");
        assert_eq!(regex_escape("a+b*c?"), r"a\+b\*c\?");
        assert_eq!(regex_escape("(foo)[bar]{baz}"), r"\(foo\)\[bar\]\{baz\}");
    }

    #[test]
    fn regex_escape_no_special_chars() {
        assert_eq!(regex_escape("foobar"), "foobar");
        assert_eq!(regex_escape("hello_world"), "hello_world");
    }

    // --- scan_directory 测试 ---

    #[test]
    fn scan_directory_returns_entries() {
        use std::fs;
        let dir = std::env::temp_dir().join("zellij_test_scan_dir");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("file1.txt"), "").unwrap();
        fs::write(dir.join("file2.rs"), "").unwrap();
        fs::create_dir(dir.join("subdir")).unwrap();
        let mut entries = scan_directory(&dir);
        entries.sort();
        assert_eq!(entries, vec!["file1.txt", "file2.rs", "subdir"]);
        let _ = fs::remove_dir_all(&dir);
    }
}
