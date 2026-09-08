use ansi_term::{
    unstyled_len, ANSIString, ANSIStrings,
    Color::{Fixed, RGB},
    Style,
};

use crate::LinePart;
use zellij_tile::prelude::*;
use zellij_tile_utils::palette_match;

macro_rules! strings {
    ($ANSIStrings:expr) => {{
        let strings: &[ANSIString] = $ANSIStrings;

        let ansi_strings = ANSIStrings(strings);

        LinePart {
            part: format!("{}", ansi_strings),
            len: unstyled_len(&ansi_strings),
        }
    }};
}

pub fn use_mouse_full(help: &ModeInfo) -> LinePart {
    // 提示：使用鼠标切换窗格焦点、滚动窗格
    // 回滚缓冲区、切换或滚动标签页
    let green_color = palette_match!(help.style.colors.text_unselected.emphasis_2);

    strings!(&[
        Style::new().paint(" Tip: "),
        Style::new().fg(green_color).bold().paint("Use the mouse"),
        Style::new().paint(" to switch pane focus, scroll through the pane scrollbuffer, switch or scroll through the tabs."),
    ])
}

pub fn use_mouse_medium(help: &ModeInfo) -> LinePart {
    // 提示：使用鼠标切换窗格/标签页或滚动窗格
    // 回滚缓冲区
    let green_color = palette_match!(help.style.colors.text_unselected.emphasis_2);

    strings!(&[
        Style::new().paint(" Tip: "),
        Style::new().fg(green_color).bold().paint("Use the mouse"),
        Style::new().paint(" to switch pane/tabs or scroll through the pane scrollbuffer."),
    ])
}

pub fn use_mouse_short(help: &ModeInfo) -> LinePart {
    // 提示：使用鼠标切换窗格/标签页或滚动
    let green_color = palette_match!(help.style.colors.text_unselected.emphasis_2);

    strings!(&[
        Style::new().fg(green_color).bold().paint(" Use the mouse"),
        Style::new().paint(" to switch pane/tabs or scroll."),
    ])
}
