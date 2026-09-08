use ansi_term::{
    unstyled_len, ANSIString, ANSIStrings,
    Color::{Fixed, RGB},
    Style,
};

use crate::{action_key, style_key_with_modifier, LinePart};
use zellij_tile::prelude::{actions::Action, *};
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

pub fn edit_scrollbuffer_full(help: &ModeInfo) -> LinePart {
    // Tip: Search through the scrollbuffer using your default $EDITOR with
    // Ctrl + <s> + <e>
    let green_color = palette_match!(help.style.colors.text_unselected.emphasis_0);

    let mut bits = vec![
        Style::new().paint(" 提示："),
        Style::new().paint("用默认 "),
        Style::new().fg(green_color).bold().paint("$EDITOR"),
        Style::new().paint(" 搜索滚动缓冲，按键"),
    ];
    bits.extend(add_keybinds(help));
    strings!(&bits)
}

pub fn edit_scrollbuffer_medium(help: &ModeInfo) -> LinePart {
    // Tip: Search the scrollbuffer using your $EDITOR with
    // Ctrl + <s> + <e>
    let green_color = palette_match!(help.style.colors.text_unselected.emphasis_0);

    let mut bits = vec![
        Style::new().paint(" 提示："),
        Style::new().paint("用 "),
        Style::new().fg(green_color).bold().paint("$EDITOR"),
        Style::new().paint(" 搜索滚动缓冲，按键"),
    ];
    bits.extend(add_keybinds(help));
    strings!(&bits)
}

pub fn edit_scrollbuffer_short(help: &ModeInfo) -> LinePart {
    // Search using $EDITOR with
    // Ctrl + <s> + <e>
    let green_color = palette_match!(help.style.colors.text_unselected.emphasis_0);

    let mut bits = vec![
        Style::new().paint(" 用 "),
        Style::new().fg(green_color).bold().paint("$EDITOR"),
        Style::new().paint(" 搜索，按键"),
    ];
    bits.extend(add_keybinds(help));
    strings!(&bits)
}

fn add_keybinds<'a>(help: &'a ModeInfo) -> Vec<ANSIString<'a>> {
    let to_pane = action_key(
        &help.get_mode_keybinds(),
        &[Action::SwitchToMode {
            input_mode: InputMode::Scroll,
        }],
    );
    let edit_buffer = action_key(
        &help.get_keybinds_for_mode(InputMode::Scroll),
        &[
            Action::EditScrollback { ansi: false },
            Action::SwitchToMode {
                input_mode: InputMode::Normal,
            },
        ],
    );

    if edit_buffer.is_empty() {
        return vec![Style::new().bold().paint("未绑定")];
    }

    let mut bits = vec![];
    bits.extend(style_key_with_modifier(&to_pane, &help.style.colors, None));
    bits.push(Style::new().paint(", "));
    bits.extend(style_key_with_modifier(
        &edit_buffer,
        &help.style.colors,
        None,
    ));
    bits
}
