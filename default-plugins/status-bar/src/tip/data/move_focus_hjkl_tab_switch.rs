use ansi_term::{unstyled_len, ANSIString, ANSIStrings, Style};

use crate::{action_key_group, style_key_with_modifier, LinePart};
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

pub fn move_focus_hjkl_tab_switch_full(help: &ModeInfo) -> LinePart {
    // 提示：使用 Alt + <←↓↑→> 改变焦点时，移出屏幕左/右边缘会聚焦下一个标签页。
    let mut bits = vec![
        Style::new().paint(" Tip: "),
        Style::new().paint("When changing focus with "),
    ];
    bits.extend(add_keybinds(help));
    bits.push(Style::new().paint(" moving off screen left/right focuses the next tab."));
    strings!(&bits)
}

pub fn move_focus_hjkl_tab_switch_medium(help: &ModeInfo) -> LinePart {
    // 提示：使用 Alt + <←↓↑→> 改变焦点时移出屏幕会聚焦下一个标签页。
    let mut bits = vec![
        Style::new().paint(" Tip: "),
        Style::new().paint("Changing focus with "),
    ];
    bits.extend(add_keybinds(help));
    bits.push(Style::new().paint(" off screen focuses the next tab."));
    strings!(&bits)
}

pub fn move_focus_hjkl_tab_switch_short(help: &ModeInfo) -> LinePart {
    //  Alt + <←↓↑→> 离开屏幕边缘时聚焦下一个标签页。
    let mut bits = add_keybinds(help);
    bits.push(Style::new().paint(" off screen edge focuses next tab."));
    strings!(&bits)
}

fn add_keybinds<'a>(help: &'a ModeInfo) -> Vec<ANSIString<'a>> {
    let pane_keymap = help.get_keybinds_for_mode(InputMode::Pane);
    let move_focus_keys = action_key_group(
        &pane_keymap,
        &[
            &[Action::MoveFocusOrTab {
                direction: Direction::Left,
            }],
            &[Action::MoveFocusOrTab {
                direction: Direction::Right,
            }],
        ],
    );

    // 让我们看看这里是否有一些共同的漂亮分组
    let mut arrows = vec![];
    let mut letters = vec![];
    for key in move_focus_keys.into_iter() {
        let key_str = key.to_string();
        if key_str.contains('←')
            || key_str.contains('↓')
            || key_str.contains('↑')
            || key_str.contains('→')
        {
            arrows.push(key);
        } else {
            letters.push(key);
        }
    }
    let arrows = style_key_with_modifier(&arrows, &help.style.colors, None);
    let letters = style_key_with_modifier(&letters, &help.style.colors, None);
    if arrows.is_empty() && letters.is_empty() {
        vec![Style::new().bold().paint("UNBOUND")]
    } else if arrows.is_empty() || letters.is_empty() {
        arrows.into_iter().chain(letters.into_iter()).collect()
    } else {
        arrows
            .into_iter()
            .chain(vec![Style::new().paint(" or ")].into_iter())
            .chain(letters.into_iter())
            .collect()
    }
}
