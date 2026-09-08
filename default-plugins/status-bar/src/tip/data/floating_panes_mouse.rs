use ansi_term::{unstyled_len, ANSIString, ANSIStrings, Style};

use crate::{action_key, style_key_with_modifier, LinePart};
use zellij_tile::prelude::{actions::Action, *};

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

pub fn floating_panes_mouse_full(help: &ModeInfo) -> LinePart {
    // 提示：使用 Ctrl + <p> + <w> 切换浮动窗格，并用键盘或鼠标移动它们
    let mut bits = vec![
        Style::new().paint(" 提示： "),
        Style::new().paint("使用以下键位切换浮动窗格："),
    ];
    bits.extend(add_keybinds(help));
    bits.push(Style::new().paint("，并用键盘或鼠标移动它们"));
    strings!(&bits)
}

pub fn floating_panes_mouse_medium(help: &ModeInfo) -> LinePart {
    // 提示：使用 Ctrl + <p> + <w> 切换浮动窗格
    let mut bits = vec![
        Style::new().paint(" 提示： "),
        Style::new().paint("使用以下键位切换浮动窗格："),
    ];
    bits.extend(add_keybinds(help));
    strings!(&bits)
}

pub fn floating_panes_mouse_short(help: &ModeInfo) -> LinePart {
    //  Ctrl + <p> + <w> => 浮动窗格
    let mut bits = add_keybinds(help);
    bits.push(Style::new().paint(" => 浮动窗格"));
    strings!(&bits)
}

fn add_keybinds<'a>(help: &'a ModeInfo) -> Vec<ANSIString<'a>> {
    let to_pane = action_key(
        &help.get_mode_keybinds(),
        &[Action::SwitchToMode {
            input_mode: InputMode::Pane,
        }],
    );
    let floating_toggle = action_key(
        &help.get_keybinds_for_mode(InputMode::Pane),
        &[
            Action::ToggleFloatingPanes,
            Action::SwitchToMode {
                input_mode: InputMode::Normal,
            },
        ],
    );

    if floating_toggle.is_empty() {
        return vec![Style::new().bold().paint("UNBOUND")];
    }

    let mut bits = vec![];
    bits.extend(style_key_with_modifier(&to_pane, &help.style.colors, None));
    bits.push(Style::new().paint(", "));
    bits.extend(style_key_with_modifier(
        &floating_toggle,
        &help.style.colors,
        None,
    ));
    bits
}
