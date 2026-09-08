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
    // 提示：使用默认的 $EDITOR 搜索回滚缓冲区，快捷键为
    //  Ctrl + <s> + <e>
    let green_color = palette_match!(help.style.colors.text_unselected.emphasis_0);

    let mut bits = vec![
        Style::new().paint(" 提示： "),
        Style::new().paint("使用默认的 "),
        Style::new().fg(green_color).bold().paint("$EDITOR"),
        Style::new().paint(" 与以下键位："),
    ];
    bits.extend(add_keybinds(help));
    strings!(&bits)
}

pub fn edit_scrollbuffer_medium(help: &ModeInfo) -> LinePart {
    // 提示：使用你的 $EDITOR 搜索回滚缓冲区，快捷键为
    //  Ctrl + <s> + <e>
    let green_color = palette_match!(help.style.colors.text_unselected.emphasis_0);

    let mut bits = vec![
        Style::new().paint(" 提示： "),
        Style::new().paint("使用你的 "),
        Style::new().fg(green_color).bold().paint("$EDITOR"),
        Style::new().paint(" 与以下键位："),
    ];
    bits.extend(add_keybinds(help));
    strings!(&bits)
}

pub fn edit_scrollbuffer_short(help: &ModeInfo) -> LinePart {
    // 使用 $EDITOR 搜索，快捷键为
    //  Ctrl + <s> + <e>
    let green_color = palette_match!(help.style.colors.text_unselected.emphasis_0);

    let mut bits = vec![
        Style::new().paint(" 使用 "),
        Style::new().fg(green_color).bold().paint("$EDITOR"),
        Style::new().paint(" 与以下键位："),
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
        return vec![Style::new().bold().paint("UNBOUND")];
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
