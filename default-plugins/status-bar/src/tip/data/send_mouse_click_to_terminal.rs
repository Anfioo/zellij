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

pub fn mouse_click_to_terminal_full(help: &ModeInfo) -> LinePart {
    // 提示：SHIFT + <鼠标点击> 绕过 Zellij，直接将鼠标点击发送到终端
    let green_color = palette_match!(help.style.colors.text_unselected.emphasis_2);
    let orange_color = palette_match!(help.style.colors.text_unselected.emphasis_0);

    strings!(&[
        Style::new().paint(" 提示： "),
        Style::new().fg(orange_color).bold().paint("Shift"),
        Style::new().paint(" + <"),
        Style::new().fg(green_color).bold().paint("mouse-click"),
        Style::new().paint("> 绕过 Zellij 并将鼠标点击直接发送到终端。"),
    ])
}

pub fn mouse_click_to_terminal_medium(help: &ModeInfo) -> LinePart {
    // 提示：SHIFT + <鼠标点击> 直接将点击发送到终端
    let green_color = palette_match!(help.style.colors.text_unselected.emphasis_2);
    let orange_color = palette_match!(help.style.colors.text_unselected.emphasis_0);
    strings!(&[
        Style::new().paint(" 提示： "),
        Style::new().fg(orange_color).bold().paint("Shift"),
        Style::new().paint(" + <"),
        Style::new().fg(green_color).bold().paint("mouse-click"),
        Style::new().paint("> 将点击直接发送到终端。"),
    ])
}

pub fn mouse_click_to_terminal_short(help: &ModeInfo) -> LinePart {
    // 提示：SHIFT + <鼠标点击> => 将点击发送到终端。
    let green_color = palette_match!(help.style.colors.text_unselected.emphasis_2);
    let orange_color = palette_match!(help.style.colors.text_unselected.emphasis_0);

    strings!(&[
        Style::new().paint(" 提示： "),
        Style::new().fg(orange_color).bold().paint("Shift"),
        Style::new().paint(" + <"),
        Style::new().fg(green_color).bold().paint("mouse-click"),
        Style::new().paint("> => 将点击发送到终端。"),
    ])
}
