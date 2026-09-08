use crate::LinePart;
use zellij_tile::prelude::*;

pub fn text_copied_hint(copy_destination: CopyDestination) -> LinePart {
    let hint = match copy_destination {
        CopyDestination::Command => "文本已通过管道发送到外部命令",
        #[cfg(not(target_os = "macos"))]
        CopyDestination::Primary => "文本已复制到系统主选区",
        #[cfg(target_os = "macos")] // primary selection does not exist on macos
        CopyDestination::Primary => "文本已复制到系统剪贴板",
        CopyDestination::System => "文本已复制到系统剪贴板",
    };
    LinePart {
        part: serialize_text(&Text::new(&hint).color_range(2, ..).opaque()),
        len: hint.len(),
        tab_index: None,
    }
}

pub fn system_clipboard_error() -> LinePart {
    let hint = " Error using the system clipboard.";
    LinePart {
        part: serialize_text(&Text::new(&hint).color_range(2, ..).opaque()),
        len: hint.len(),
        tab_index: None,
    }
}
