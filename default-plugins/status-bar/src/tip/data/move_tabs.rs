use ansi_term::{
    unstyled_len, ANSIString, ANSIStrings,
    Color::{Fixed, RGB},
    Style,
};

use zellij_tile::prelude::*;
use zellij_tile_utils::palette_match;

use crate::LinePart;

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

pub fn move_tabs_full(help: &ModeInfo) -> LinePart {
    // 提示：标签页顺序不对？你可以使用以下方式左右移动它们：
    //  Alt + i（左）和 Alt + o（右）
    let green_color = palette_match!(help.style.colors.text_unselected.emphasis_2);

    let bits = vec![
        Style::new().paint(" Tip: "),
        Style::new().paint("Wrong order of tabs? You can move them to left and right with: "),
        Style::new().fg(green_color).bold().paint("Alt + i"),
        Style::new().paint(" (left) and "),
        Style::new().fg(green_color).bold().paint("Alt + o"),
        Style::new().paint(" (right)"),
    ];
    strings!(&bits)
}

pub fn move_tabs_medium(help: &ModeInfo) -> LinePart {
    // 提示：你可以使用以下方式左右移动标签页：
    //  Alt + i（左）和 Alt + o（右）
    let green_color = palette_match!(help.style.colors.text_unselected.emphasis_2);

    let bits = vec![
        Style::new().paint(" Tip: "),
        Style::new().paint("You can move tabs to left and right with: "),
        Style::new().fg(green_color).bold().paint("Alt + i"),
        Style::new().paint(" (left) and "),
        Style::new().fg(green_color).bold().paint("Alt + o"),
        Style::new().paint(" (right)"),
    ];
    strings!(&bits)
}

pub fn move_tabs_short(help: &ModeInfo) -> LinePart {
    // 移动标签页：Alt + i（左）和 Alt + o（右）
    let green_color = palette_match!(help.style.colors.text_unselected.emphasis_2);

    let bits = vec![
        Style::new().paint(" Move tabs with: "),
        Style::new().fg(green_color).bold().paint("Alt + i"),
        Style::new().paint(" (left) and "),
        Style::new().fg(green_color).bold().paint("Alt + o"),
        Style::new().paint(" (right)"),
    ];
    strings!(&bits)
}
