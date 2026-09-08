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

pub fn zellij_setup_check_full(help: &ModeInfo) -> LinePart {
    // 提示：Zellij 有问题？尝试运行 "zellij setup --check"
    let orange_color = palette_match!(help.style.colors.text_unselected.emphasis_0);

    strings!(&[
        Style::new().paint(" 提示： "),
        Style::new().paint("Zellij 遇到问题？试试运行 "),
        Style::new()
            .fg(orange_color)
            .bold()
            .paint("zellij setup --check"),
    ])
}

pub fn zellij_setup_check_medium(help: &ModeInfo) -> LinePart {
    // 提示：运行 "zellij setup --check" 查找问题
    let orange_color = palette_match!(help.style.colors.text_unselected.emphasis_0);

    strings!(&[
        Style::new().paint(" 提示： "),
        Style::new().paint("运行 "),
        Style::new()
            .fg(orange_color)
            .bold()
            .paint("zellij setup --check"),
        Style::new().paint(" 来查找问题"),
    ])
}

pub fn zellij_setup_check_short(help: &ModeInfo) -> LinePart {
    // 运行 "zellij setup --check" 查找问题
    let orange_color = palette_match!(help.style.colors.text_unselected.emphasis_0);

    strings!(&[
        Style::new().paint(" 运行 "),
        Style::new()
            .fg(orange_color)
            .bold()
            .paint("zellij setup --check"),
        Style::new().paint(" 来查找问题"),
    ])
}
