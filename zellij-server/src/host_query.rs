//! Zellij 代表窗格内运行的应用程序转发的主机终端查询的分类表示。
//!
//! 网格调度器已经通过 `vte` 解析了传入的 OSC / CSI 序列；我们不在回复合成阶段存储原始字节并重新解析它们，而是在拦截时将分类捕获到 [`HostQuery`] 中。从 Grid → Tab → Screen 的流水线携带此枚举；Screen 的缓存回退合成直接匹配变体（无需字节级正则表达式）。当我们必须真正在线路上发送字节时（服务端 → 客户端 → 主机终端），[`HostQuery::to_query_bytes`] 会从枚举重新派生它们。

/// 常用的两种 OSC 字符串终止符。使用其中一种表述查询的应用程序期望回复能镜像它。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OscTerminator {
    /// `ESC \`（字符串终止符，xterm 风格）。
    St,
    /// `BEL`（`\x07`）。
    Bel,
}

impl OscTerminator {
    pub fn from_bell_terminated(bell_terminated: bool) -> Self {
        if bell_terminated {
            OscTerminator::Bel
        } else {
            OscTerminator::St
        }
    }

    pub fn as_bytes(self) -> &'static [u8] {
        match self {
            OscTerminator::St => b"\x1b\\",
            OscTerminator::Bel => b"\x07",
        }
    }
}

/// 白名单中的主机终端查询。变体与 Grid 拦截并推送到 `pending_forwarded_queries` 上的条目一一对应。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostQuery {
    /// `CSI 14 t` — 文本区域像素尺寸。
    TextAreaPixelSize,
    /// `CSI 16 t` — 字符单元格像素尺寸。
    CharacterCellPixelSize,
    /// `OSC 10 ; ? <term>` — 默认前景色。
    DefaultForeground { terminator: OscTerminator },
    /// `OSC 11 ; ? <term>` — 默认背景色。
    DefaultBackground { terminator: OscTerminator },
    /// `OSC 4 ; N ; ? <term>` — 调色板寄存器 `N`。
    PaletteRegister {
        index: u8,
        terminator: OscTerminator,
    },
    ClipboardContent {
        selection: char,
        terminator: OscTerminator,
    },
    /// `CSI ? 996 n` — 查询主机终端的调色板主题模式（浅色或深色）。不转发给主机。Zellij 在启动时已经通过客户端的 `\e[?996n` 写入查询过主机一次，并在 `\e[?2031h` 启用期间跟踪未经请求的 DSR 997 更新，因此可以从缓存中回答。Screen 处理程序会短路此变体：它不向线路写入，而是直接将 `\e[?997;{0|1|2}n` 合成到发起窗格的 pty 中。
    ColorPaletteMode,
}

impl HostQuery {
    /// 将此查询重新序列化为主机终端期望的字节序列。客户端在将查询写入其标准输出时使用，断言线路形状的测试也使用。
    pub fn to_query_bytes(&self) -> Vec<u8> {
        match self {
            HostQuery::TextAreaPixelSize => b"\x1b[14t".to_vec(),
            HostQuery::CharacterCellPixelSize => b"\x1b[16t".to_vec(),
            HostQuery::DefaultForeground { terminator } => {
                let mut v = b"\x1b]10;?".to_vec();
                v.extend_from_slice(terminator.as_bytes());
                v
            },
            HostQuery::DefaultBackground { terminator } => {
                let mut v = b"\x1b]11;?".to_vec();
                v.extend_from_slice(terminator.as_bytes());
                v
            },
            HostQuery::PaletteRegister { index, terminator } => {
                let mut v = format!("\x1b]4;{};?", index).into_bytes();
                v.extend_from_slice(terminator.as_bytes());
                v
            },
            HostQuery::ClipboardContent {
                selection,
                terminator,
            } => {
                let mut v = format!("\x1b]52;{};?", selection).into_bytes();
                v.extend_from_slice(terminator.as_bytes());
                v
            },
            // `ColorPaletteMode` 由 Zellij 本地回答，永远不会在线路上发送。返回空字节可以保持调用总数不变而不污染线路格式。绕过 `Screen::forward_host_query` 处理此变体的调用者根本不应该对它调用 `to_query_bytes`。
            HostQuery::ColorPaletteMode => Vec::new(),
        }
    }

    pub fn empty_reply_bytes(&self) -> Vec<u8> {
        match self {
            HostQuery::ClipboardContent {
                selection,
                terminator,
            } => {
                let mut v = format!("\x1b]52;{};", selection).into_bytes();
                v.extend_from_slice(terminator.as_bytes());
                v
            },
            _ => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_area_pixel_size_serializes_as_csi_14t() {
        assert_eq!(HostQuery::TextAreaPixelSize.to_query_bytes(), b"\x1b[14t");
    }

    #[test]
    fn character_cell_pixel_size_serializes_as_csi_16t() {
        assert_eq!(
            HostQuery::CharacterCellPixelSize.to_query_bytes(),
            b"\x1b[16t"
        );
    }

    #[test]
    fn default_foreground_mirrors_terminator() {
        assert_eq!(
            HostQuery::DefaultForeground {
                terminator: OscTerminator::St,
            }
            .to_query_bytes(),
            b"\x1b]10;?\x1b\\",
        );
        assert_eq!(
            HostQuery::DefaultForeground {
                terminator: OscTerminator::Bel,
            }
            .to_query_bytes(),
            b"\x1b]10;?\x07",
        );
    }

    #[test]
    fn default_background_serializes_correctly() {
        assert_eq!(
            HostQuery::DefaultBackground {
                terminator: OscTerminator::St,
            }
            .to_query_bytes(),
            b"\x1b]11;?\x1b\\",
        );
    }

    #[test]
    fn palette_register_carries_index_and_terminator() {
        assert_eq!(
            HostQuery::PaletteRegister {
                index: 5,
                terminator: OscTerminator::Bel,
            }
            .to_query_bytes(),
            b"\x1b]4;5;?\x07",
        );
        assert_eq!(
            HostQuery::PaletteRegister {
                index: 255,
                terminator: OscTerminator::St,
            }
            .to_query_bytes(),
            b"\x1b]4;255;?\x1b\\",
        );
    }

    #[test]
    fn clipboard_content_carries_selection_and_terminator() {
        assert_eq!(
            HostQuery::ClipboardContent {
                selection: 'c',
                terminator: OscTerminator::Bel,
            }
            .to_query_bytes(),
            b"\x1b]52;c;?\x07",
        );
        assert_eq!(
            HostQuery::ClipboardContent {
                selection: 'p',
                terminator: OscTerminator::St,
            }
            .to_query_bytes(),
            b"\x1b]52;p;?\x1b\\",
        );
    }

    #[test]
    fn clipboard_content_empty_reply_mirrors_the_query_shape() {
        assert_eq!(
            HostQuery::ClipboardContent {
                selection: 'c',
                terminator: OscTerminator::Bel,
            }
            .empty_reply_bytes(),
            b"\x1b]52;c;\x07",
        );
        assert_eq!(
            HostQuery::ClipboardContent {
                selection: 'p',
                terminator: OscTerminator::St,
            }
            .empty_reply_bytes(),
            b"\x1b]52;p;\x1b\\",
        );
    }

    #[test]
    fn non_clipboard_queries_have_no_empty_reply() {
        assert!(HostQuery::TextAreaPixelSize.empty_reply_bytes().is_empty());
        assert!(HostQuery::DefaultBackground {
            terminator: OscTerminator::St,
        }
        .empty_reply_bytes()
        .is_empty());
    }

    #[test]
    fn from_bell_terminated_maps_bool_to_variant() {
        assert_eq!(
            OscTerminator::from_bell_terminated(true),
            OscTerminator::Bel
        );
        assert_eq!(
            OscTerminator::from_bell_terminated(false),
            OscTerminator::St
        );
    }
}
