use ansi_term::{unstyled_len, ANSIStrings};
use zellij_tile::prelude::actions::Action;
use zellij_tile::prelude::*;

use crate::color_elements;
use crate::{
    action_key, action_key_group, get_common_modifiers, style_key_with_modifier, TO_NORMAL,
};
use crate::{ColoredElements, LinePart};

#[derive(Debug)]
pub struct KeyShortcut {
    pub mode: KeyMode,
    pub action: KeyAction,
    pub key: Option<KeyWithModifier>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum KeyAction {
    Unlock,
    Lock,
    Pane,
    Tab,
    Resize,
    Search,
    Quit,
    Session,
    Move,
    Tmux,
}

#[derive(Debug, Copy, Clone)]
pub enum KeyMode {
    Unselected,
    UnselectedAlternate,
    Selected,
    Disabled,
}

impl KeyShortcut {
    pub fn new(mode: KeyMode, action: KeyAction, key: Option<KeyWithModifier>) -> Self {
        KeyShortcut { mode, action, key }
    }

    pub fn full_text(&self) -> String {
        match self.action {
            KeyAction::Lock => String::from("LOCK"),
            KeyAction::Unlock => String::from("UNLOCK"),
            KeyAction::Pane => String::from("PANE"),
            KeyAction::Tab => String::from("TAB"),
            KeyAction::Resize => String::from("RESIZE"),
            KeyAction::Search => String::from("SEARCH"),
            KeyAction::Quit => String::from("QUIT"),
            KeyAction::Session => String::from("SESSION"),
            KeyAction::Move => String::from("MOVE"),
            KeyAction::Tmux => String::from("TMUX"),
        }
    }
    pub fn with_shortened_modifiers(&self, common_modifiers: &Vec<KeyModifier>) -> String {
        let key = match &self.key {
            Some(k) => k.strip_common_modifiers(common_modifiers),
            None => return String::from("?"),
        };
        let shortened_modifiers = key
            .key_modifiers
            .iter()
            .map(|m| match m {
                KeyModifier::Ctrl => "^C",
                KeyModifier::Alt => "^A",
                KeyModifier::Super => "^Su",
                KeyModifier::Shift => "^Sh",
            })
            .collect::<Vec<_>>()
            .join("-");
        if shortened_modifiers.is_empty() {
            format!("{}", key)
        } else {
            format!("{} {}", shortened_modifiers, key.bare_key)
        }
    }
    pub fn letter_shortcut(&self, common_modifiers: &Vec<KeyModifier>) -> String {
        let key = match &self.key {
            Some(k) => k.strip_common_modifiers(common_modifiers),
            None => return String::from("?"),
        };
        format!("{}", key)
    }
    pub fn get_key(&self) -> Option<KeyWithModifier> {
        self.key.clone()
    }
    pub fn get_mode(&self) -> KeyMode {
        self.mode
    }
    pub fn get_action(&self) -> KeyAction {
        self.action
    }
    pub fn is_selected(&self) -> bool {
        match self.mode {
            KeyMode::Selected => true,
            _ => false,
        }
    }
    pub fn short_text(&self) -> String {
        match self.action {
            KeyAction::Lock => String::from("Lo"),
            KeyAction::Unlock => String::from("Un"),
            KeyAction::Pane => String::from("Pa"),
            KeyAction::Tab => String::from("Ta"),
            KeyAction::Resize => String::from("Re"),
            KeyAction::Search => String::from("Se"),
            KeyAction::Quit => String::from("Qu"),
            KeyAction::Session => String::from("Se"),
            KeyAction::Move => String::from("Mo"),
            KeyAction::Tmux => String::from("Tm"),
        }
    }
}

///  生成长模式快捷方式块。
///
///长模式快捷方式块由前导和尾随 `separator`、包裹在
///`<>` 中以及旁边以大写字母显示的模式名称组成。例如，
///例如，"Locked" 模式的默认长模式快捷方式块为：` <g> LOCK `。
///
///# 参数
///
///- `key`：一个 [`KeyShortcut`]，定义块的显示方式（激活/禁用/...）、所属
///操作（大致等同于 [`InputMode`]）以及触发此操作的快捷键绑定。
///此操作。
///- `palette`：保存样式信息的结构。
///- `separator`：在模式快捷方式块前后打印的分隔符。默认为
///箭头形状的分隔符。
///- `shared_super`：如果设为 true，所有模式快捷方式快捷键绑定共享一个通用修饰键（参见
///[`get_common_modifier`]），并且属于该快捷键绑定的修饰键**不会**打印在
///快捷方式块中。
///- `first_tile`：如果设为 true，将省略此块的前导分隔符，这样屏幕上不会出现间隙
///出现在屏幕上。
fn long_mode_shortcut(
    key: &KeyShortcut,
    palette: ColoredElements,
    separator: &str,
    common_modifiers: &Vec<KeyModifier>,
    first_tile: bool,
) -> LinePart {
    let key_hint = key.full_text();
    let has_common_modifiers = !common_modifiers.is_empty();
    let key_binding = match (&key.mode, &key.key) {
        (KeyMode::Disabled, None) => "".to_string(),
        (_, None) => return LinePart::default(),
        (_, Some(_)) => key.letter_shortcut(common_modifiers),
    };

    let colors = match key.mode {
        KeyMode::Unselected => palette.unselected,
        KeyMode::UnselectedAlternate => palette.unselected_alternate,
        KeyMode::Selected => palette.selected,
        KeyMode::Disabled => palette.disabled,
    };
    let start_separator = if !has_common_modifiers && first_tile {
        ""
    } else {
        separator
    };
    let prefix_separator = colors.prefix_separator.paint(start_separator);
    let char_left_separator = colors.char_left_separator.paint(" <".to_string());
    let char_shortcut = colors.char_shortcut.paint(key_binding.to_string());
    let char_right_separator = colors.char_right_separator.paint("> ".to_string());
    let styled_text = colors.styled_text.paint(format!("{} ", key_hint));
    let suffix_separator = colors.suffix_separator.paint(separator);
    LinePart {
        part: ANSIStrings(&[
            prefix_separator,
            char_left_separator,
            char_shortcut,
            char_right_separator,
            styled_text,
            suffix_separator,
        ])
        .to_string(),
        len: start_separator.chars().count() //  分隔符
            + 2                              //  " <"
            + key_binding.chars().count()    //  快捷键绑定
            + 2                              //  "> "
            + key_hint.chars().count()       //  按键提示（模式）
            + 1                              //  " "
            + separator.chars().count(), //  分隔符
    }
}

fn shortened_modifier_shortcut(
    key: &KeyShortcut,
    palette: ColoredElements,
    separator: &str,
    common_modifiers: &Vec<KeyModifier>,
    first_tile: bool,
) -> LinePart {
    let key_hint = key.full_text();
    let has_common_modifiers = !common_modifiers.is_empty();
    let key_binding = match (&key.mode, &key.key) {
        (KeyMode::Disabled, None) => "".to_string(),
        (_, None) => return LinePart::default(),
        (_, Some(_)) => key.with_shortened_modifiers(common_modifiers),
    };

    let colors = match key.mode {
        KeyMode::Unselected => palette.unselected,
        KeyMode::UnselectedAlternate => palette.unselected_alternate,
        KeyMode::Selected => palette.selected,
        KeyMode::Disabled => palette.disabled,
    };
    let start_separator = if !has_common_modifiers && first_tile {
        ""
    } else {
        separator
    };
    let prefix_separator = colors.prefix_separator.paint(start_separator);
    let char_left_separator = colors.char_left_separator.paint(" <".to_string());
    let char_shortcut = colors.char_shortcut.paint(key_binding.to_string());
    let char_right_separator = colors.char_right_separator.paint("> ".to_string());
    let styled_text = colors.styled_text.paint(format!("{} ", key_hint));
    let suffix_separator = colors.suffix_separator.paint(separator);
    LinePart {
        part: ANSIStrings(&[
            prefix_separator,
            char_left_separator,
            char_shortcut,
            char_right_separator,
            styled_text,
            suffix_separator,
        ])
        .to_string(),
        len: start_separator.chars().count() //  分隔符
            + 2                              //  " <"
            + key_binding.chars().count()    //  快捷键绑定
            + 2                              //  "> "
            + key_hint.chars().count()       //  按键提示（模式）
            + 1                              //  " "
            + separator.chars().count(), //  分隔符
    }
}

///  生成短模式快捷方式块。
///
///短模式快捷方式块由前导和尾随 `separator` 以及一个快捷键绑定组成。
///例如，"Locked" 模式的默认短模式快捷方式块为：` g `。
///
///# 参数
///
///- `key`：一个 [`KeyShortcut`]，定义块的显示方式（激活/禁用/...）、所属
///操作（大致等同于 [`InputMode`]）以及触发此操作的快捷键绑定。
///此操作。
///- `palette`：保存样式信息的结构。
///- `separator`：在模式快捷方式块前后打印的分隔符。默认为
///箭头形状的分隔符。
///- `shared_super`：如果设为 true，所有模式快捷方式快捷键绑定共享一个通用修饰键（参见
///[`get_common_modifier`]），并且属于该快捷键绑定的修饰键**不会**打印在
///快捷方式块中。
///- `first_tile`：如果设为 true，将省略此块的前导分隔符，这样屏幕上不会出现间隙
///出现在屏幕上。
fn short_mode_shortcut(
    key: &KeyShortcut,
    palette: ColoredElements,
    separator: &str,
    common_modifiers: &Vec<KeyModifier>,
    first_tile: bool,
) -> LinePart {
    let has_common_modifiers = !common_modifiers.is_empty();
    let key_binding = match (&key.mode, &key.key) {
        (KeyMode::Disabled, None) => "".to_string(),
        (_, None) => return LinePart::default(),
        (_, Some(_)) => key.letter_shortcut(common_modifiers),
    };

    let colors = match key.mode {
        KeyMode::Unselected => palette.unselected,
        KeyMode::UnselectedAlternate => palette.unselected_alternate,
        KeyMode::Selected => palette.selected,
        KeyMode::Disabled => palette.disabled,
    };
    let start_separator = if !has_common_modifiers && first_tile {
        ""
    } else {
        separator
    };
    let prefix_separator = colors.prefix_separator.paint(start_separator);
    let char_shortcut = colors.char_shortcut.paint(format!(" {} ", key_binding));
    let suffix_separator = colors.suffix_separator.paint(separator);
    LinePart {
        part: ANSIStrings(&[prefix_separator, char_shortcut, suffix_separator]).to_string(),
        len: separator.chars().count()      //  分隔符
            + 1                             //  " "
            + key_binding.chars().count()   //  快捷键绑定
            + 1                             //  " "
            + separator.chars().count(), //  分隔符
    }
}

fn key_indicators(
    max_len: usize,
    keys: &[KeyShortcut],
    palette: ColoredElements,
    separator: &str,
    mode_info: &ModeInfo,
) -> LinePart {
    //  打印全宽提示
    let (shared_modifiers, mut line_part) = superkey(palette, separator, mode_info);
    for key in keys {
        let line_empty = line_part.len == 0;
        let key = long_mode_shortcut(key, palette, separator, &shared_modifiers, line_empty);
        line_part.part = format!("{}{}", line_part.part, key.part);
        line_part.len += key.len;
    }
    if line_part.len < max_len {
        return line_part;
    }

    //  全宽不适用，尝试缩短修饰键（例如用 "^C" 代替 "Ctrl"）
    line_part = superkey(palette, separator, mode_info).1;
    for key in keys {
        let line_empty = line_part.len == 0;
        let key =
            shortened_modifier_shortcut(key, palette, separator, &shared_modifiers, line_empty);
        line_part.part = format!("{}{}", line_part.part, key.part);
        line_part.len += key.len;
    }
    if line_part.len < max_len {
        return line_part;
    }

    //  全宽不适用，尝试缩短提示（仅快捷键绑定，无含义/操作）
    line_part = superkey(palette, separator, mode_info).1;
    for key in keys {
        let line_empty = line_part.len == 0;
        let key = short_mode_shortcut(key, palette, separator, &shared_modifiers, line_empty);
        line_part.part = format!("{}{}", line_part.part, key.part);
        line_part.len += key.len;
    }
    if line_part.len < max_len {
        return line_part;
    }

    //  缩短版也不适用，不打印任何内容
    line_part = LinePart::default();
    line_part
}

fn swap_layout_keycode(mode_info: &ModeInfo) -> LinePart {
    let mode_keybinds = mode_info.get_mode_keybinds();
    let prev_next_keys = action_key_group(
        &mode_keybinds,
        &[&[Action::PreviousSwapLayout], &[Action::NextSwapLayout]],
    );
    let prev_next_keys_indicator = style_key_with_modifier(
        &prev_next_keys,
        &mode_info.style.colors,
        Some(mode_info.style.colors.text_unselected.background),
    );
    let keycode = ANSIStrings(&prev_next_keys_indicator);
    let len = unstyled_len(&keycode);
    let part = keycode.to_string();
    LinePart { part, len }
}

fn swap_layout_status(
    max_len: usize,
    swap_layout_name: &Option<String>,
    is_swap_layout_damaged: bool,
    mode_info: &ModeInfo,
    colored_elements: ColoredElements,
    separator: &str,
) -> Option<LinePart> {
    match swap_layout_name {
        Some(swap_layout_name) => {
            let mut swap_layout_name = format!(" {} ", swap_layout_name);
            swap_layout_name.make_ascii_uppercase();
            let keycode = swap_layout_keycode(mode_info);
            let swap_layout_name_len = swap_layout_name.len() + 3; //  2 用于箭头分隔符，1 用于屏幕末端缓冲区
                                                                   //
            macro_rules! style_swap_layout_indicator {
                ($style_name:ident) => {{
                    (
                        colored_elements
                            .$style_name
                            .prefix_separator
                            .paint(separator),
                        colored_elements
                            .$style_name
                            .styled_text
                            .paint(&swap_layout_name),
                        colored_elements
                            .$style_name
                            .suffix_separator
                            .paint(separator),
                    )
                }};
            }
            let (prefix_separator, swap_layout_name, suffix_separator) =
                if mode_info.mode == InputMode::Locked {
                    style_swap_layout_indicator!(disabled)
                } else if is_swap_layout_damaged {
                    style_swap_layout_indicator!(unselected)
                } else {
                    style_swap_layout_indicator!(selected)
                };
            let swap_layout_indicator = format!(
                "{}{}{}",
                prefix_separator, swap_layout_name, suffix_separator
            );
            let (part, full_len) = if mode_info.mode == InputMode::Locked {
                (
                    format!("{}", swap_layout_indicator),
                    swap_layout_name_len, //  1 是之间的空格
                )
            } else {
                (
                    format!(
                        "{}{}{}{}",
                        keycode,
                        colored_elements.superkey_prefix.paint(" "),
                        swap_layout_indicator,
                        colored_elements.superkey_prefix.paint(" ")
                    ),
                    keycode.len + swap_layout_name_len + 1, //  1 是之间的空格
                )
            };
            let short_len = swap_layout_name_len + 1; //  1 是之间的空格
            if full_len <= max_len {
                Some(LinePart {
                    part,
                    len: full_len,
                })
            } else if short_len <= max_len && mode_info.mode != InputMode::Locked {
                Some(LinePart {
                    part: swap_layout_indicator,
                    len: short_len,
                })
            } else {
                None
            }
        },
        None => None,
    }
}

///  获取状态栏中可见的切换 `InputMode` 和 `Quit` 的快捷键绑定。
///
///返回一个 `Key` 向量，其中每个 `Key` 是切换到某个 `InputMode` 或退出的快捷方式
///zellij。鉴于用户可以在 zellij 配置中配置大量内容，此
///函数有一些需要注意的限制：
///
///- 向量不会去重：如果切换到某个 `InputMode` 绑定了多个
///`Key`，所有这些绑定都将成为返回向量的一部分。也没有
///保证的排序顺序。在这种情况下，哪个键最终出现在状态栏中是未定义的。
///- 向量**不会**包含 ' '、'\n' 和 'Esc' 键：这些是默认绑定
///从任何输入模式返回普通模式，但在搜索时它们并不重要
///用于超级键。如果对于任何输入模式，用户仅将这些键绑定为切换回
///到 `InputMode::Normal`，则将显示 '?' 作为快捷键绑定。
pub fn mode_switch_keys(mode_info: &ModeInfo) -> Vec<KeyWithModifier> {
    mode_info
        .get_mode_keybinds()
        .iter()
        .filter_map(|(key, vac)| match vac.first() {
            //  未定义操作，忽略
            None => None,
            Some(vac) => {
                //  我们忽略某些切换回普通 InputMode 的"默认"快捷键绑定。
                //  包括：' '、'\n'、'Esc'
                if matches!(
                    key,
                    KeyWithModifier {
                        bare_key: BareKey::Char(' '),
                        ..
                    } | KeyWithModifier {
                        bare_key: BareKey::Enter,
                        ..
                    } | KeyWithModifier {
                        bare_key: BareKey::Esc,
                        ..
                    }
                ) {
                    return None;
                }
                if let actions::Action::SwitchToMode { input_mode: mode } = vac {
                    return match mode {
                        //  存储切换到显示模式的键
                        InputMode::Normal
                        | InputMode::Locked
                        | InputMode::Pane
                        | InputMode::Tab
                        | InputMode::Resize
                        | InputMode::Move
                        | InputMode::Scroll
                        | InputMode::Session => Some(key.clone()),
                        _ => None,
                    };
                }
                if let actions::Action::Quit = vac {
                    return Some(key.clone());
                }
                //  不是 `SwitchToMode` 或 `Quit` 操作，忽略
                None
            },
        })
        .collect()
}

pub fn superkey(
    palette: ColoredElements,
    separator: &str,
    mode_info: &ModeInfo,
) -> (Vec<KeyModifier>, LinePart) {
    //  查找通用修饰键（如果有）
    let common_modifiers = get_common_modifiers(mode_switch_keys(mode_info).iter().collect());
    if common_modifiers.is_empty() {
        return (common_modifiers, LinePart::default());
    }

    let prefix_text = if mode_info.capabilities.arrow_fonts {
        //  在简化界面中添加额外空格
        format!(
            " {} + ",
            common_modifiers
                .iter()
                .map(|m| m.to_string())
                .collect::<Vec<_>>()
                .join("-")
        )
    } else {
        format!(
            " {} +",
            common_modifiers
                .iter()
                .map(|m| m.to_string())
                .collect::<Vec<_>>()
                .join("-")
        )
    };

    let prefix = palette.superkey_prefix.paint(&prefix_text);
    let suffix_separator = palette.superkey_suffix_separator.paint(separator);
    (
        common_modifiers,
        LinePart {
            part: ANSIStrings(&[prefix, suffix_separator]).to_string(),
            len: prefix_text.chars().count() + separator.chars().count(),
        },
    )
}

pub fn to_char(kv: Vec<KeyWithModifier>) -> Option<KeyWithModifier> {
    let key = kv
        .iter()
        .filter(|key| {
            //  这些是返回普通模式的通用"快捷键绑定"，在这里不感兴趣。
            !matches!(
                key,
                KeyWithModifier {
                    bare_key: BareKey::Enter,
                    ..
                } | KeyWithModifier {
                    bare_key: BareKey::Char(' '),
                    ..
                } | KeyWithModifier {
                    bare_key: BareKey::Esc,
                    ..
                }
            )
        })
        .collect::<Vec<&KeyWithModifier>>()
        .into_iter()
        .next();
    //  也许用户绑定了被忽略的键之一？
    if key.is_none() {
        return kv.first().cloned();
    }
    key.cloned()
}

///  获取特定 [`InputMode`] 的 [`KeyShortcut`]。
///
///遍历 `shortcuts` 的内容以查找 [`KeyAction`] 匹配的 [`KeyShortcut`]
///匹配 [`InputMode`]。如果找到匹配项，返回对 `shortcuts` 中条目的可变引用，
///如果找到匹配项，否则为 `None`。
///
///如果 `shortcuts` 中有多个条目匹配 `mode`（这不应该发生），则返回第一个匹配项
///被返回。
fn get_key_shortcut_for_mode<'a>(
    shortcuts: &'a mut [KeyShortcut],
    mode: &InputMode,
) -> Option<&'a mut KeyShortcut> {
    let key_action = match mode {
        InputMode::Normal | InputMode::Prompt | InputMode::Tmux => return None,
        InputMode::Locked => KeyAction::Lock,
        InputMode::Pane | InputMode::RenamePane => KeyAction::Pane,
        InputMode::Tab | InputMode::RenameTab => KeyAction::Tab,
        InputMode::Resize => KeyAction::Resize,
        InputMode::Move => KeyAction::Move,
        InputMode::Scroll | InputMode::Search | InputMode::EnterSearch => KeyAction::Search,
        InputMode::Session => KeyAction::Session,
    };
    for shortcut in shortcuts.iter_mut() {
        if shortcut.action == key_action {
            return Some(shortcut);
        }
    }
    None
}

pub fn first_line(
    help: &ModeInfo,
    tab_info: Option<&TabInfo>,
    max_len: usize,
    separator: &str,
) -> LinePart {
    let supports_arrow_fonts = !help.capabilities.arrow_fonts;
    let colored_elements = color_elements(help.style.colors, !supports_arrow_fonts, false);
    let binds = &help.get_mode_keybinds();
    //  默认全部取消选中
    let mut default_keys = vec![
        KeyShortcut::new(
            KeyMode::Unselected,
            KeyAction::Lock,
            to_char(action_key(
                binds,
                &[Action::SwitchToMode {
                    input_mode: InputMode::Locked,
                }],
            )),
        ),
        KeyShortcut::new(
            KeyMode::UnselectedAlternate,
            KeyAction::Pane,
            to_char(action_key(
                binds,
                &[Action::SwitchToMode {
                    input_mode: InputMode::Pane,
                }],
            )),
        ),
        KeyShortcut::new(
            KeyMode::Unselected,
            KeyAction::Tab,
            to_char(action_key(
                binds,
                &[Action::SwitchToMode {
                    input_mode: InputMode::Tab,
                }],
            )),
        ),
        KeyShortcut::new(
            KeyMode::UnselectedAlternate,
            KeyAction::Resize,
            to_char(action_key(
                binds,
                &[Action::SwitchToMode {
                    input_mode: InputMode::Resize,
                }],
            )),
        ),
        KeyShortcut::new(
            KeyMode::Unselected,
            KeyAction::Move,
            to_char(action_key(
                binds,
                &[Action::SwitchToMode {
                    input_mode: InputMode::Move,
                }],
            )),
        ),
        KeyShortcut::new(
            KeyMode::UnselectedAlternate,
            KeyAction::Search,
            to_char(action_key(
                binds,
                &[Action::SwitchToMode {
                    input_mode: InputMode::Scroll,
                }],
            )),
        ),
        KeyShortcut::new(
            KeyMode::Unselected,
            KeyAction::Session,
            to_char(action_key(
                binds,
                &[Action::SwitchToMode {
                    input_mode: InputMode::Session,
                }],
            )),
        ),
        KeyShortcut::new(
            KeyMode::UnselectedAlternate,
            KeyAction::Quit,
            to_char(action_key(binds, &[Action::Quit])),
        ),
    ];

    if let Some(key_shortcut) = get_key_shortcut_for_mode(&mut default_keys, &help.mode) {
        key_shortcut.mode = KeyMode::Selected;
        key_shortcut.key = to_char(action_key(binds, &[TO_NORMAL]));
    }

    //  在锁定模式下，我们必须禁用所有其他模式快捷键绑定
    if help.mode == InputMode::Locked {
        for key in default_keys.iter_mut().skip(1) {
            key.mode = KeyMode::Disabled;
        }
    }

    if help.mode == InputMode::Tmux {
        //  Tmux 块默认隐藏
        default_keys.push(KeyShortcut::new(
            KeyMode::Selected,
            KeyAction::Tmux,
            to_char(action_key(binds, &[TO_NORMAL])),
        ));
    }

    let mut key_indicators =
        key_indicators(max_len, &default_keys, colored_elements, separator, help);
    if key_indicators.len < max_len {
        if let Some(tab_info) = tab_info {
            let mut remaining_space = max_len - key_indicators.len;
            if let Some(swap_layout_status) = swap_layout_status(
                remaining_space,
                &tab_info.active_swap_layout_name,
                tab_info.is_swap_layout_dirty,
                help,
                colored_elements,
                separator,
            ) {
                remaining_space -= swap_layout_status.len;
                for _ in 0..remaining_space {
                    key_indicators.part.push_str(
                        &ANSIStrings(&[colored_elements.superkey_prefix.paint(" ")]).to_string(),
                    );
                    key_indicators.len += 1;
                }
                key_indicators.append(&swap_layout_status);
            }
        }
    }
    key_indicators
}

#[cfg(test)]
///  单元测试。
///
///注意我们在这里取了一点巧，因为可能想要测试的东西数量是无穷无尽的，
///而为所有这些测试用例手动创建 [`ModeInfo`] 的 Mockup 简直是
///一种折磨。因此，我们彻底测试最原子的单元（[`long_mode_shortcut`] 和
///[`short_mode_shortcut`]），然后测试公共 API（[`first_line`]）以确保正确
///操作。
mod tests {
    use super::*;

    fn colored_elements() -> ColoredElements {
        let palette = Styling::default();
        color_elements(palette, false, false)
    }

    //  从 `LinePart` 中剥离样式信息并返回原始 String
    fn unstyle(line_part: LinePart) -> String {
        let string = line_part.to_string();

        let re = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
        let string = re.replace_all(&string, "".to_string());

        string.to_string()
    }

    #[test]
    fn long_mode_shortcut_selected_with_binding() {
        let key = KeyShortcut::new(
            KeyMode::Selected,
            KeyAction::Session,
            Some(KeyWithModifier::new(BareKey::Char('0'))),
        );
        let color = colored_elements();

        let ret = long_mode_shortcut(&key, color, "+", &vec![], false);
        let ret = unstyle(ret);

        assert_eq!(ret, "+ <0> SESSION +".to_string());
    }

    #[test]
    //  显示效果类似 selected(alternate)，但样式不同
    fn long_mode_shortcut_unselected_with_binding() {
        let key = KeyShortcut::new(
            KeyMode::Unselected,
            KeyAction::Session,
            Some(KeyWithModifier::new(BareKey::Char('0'))),
        );
        let color = colored_elements();

        let ret = long_mode_shortcut(&key, color, "+", &vec![], false);
        let ret = unstyle(ret);

        assert_eq!(ret, "+ <0> SESSION +".to_string());
    }

    #[test]
    //  完全按照 "unselected" 变体处理
    fn long_mode_shortcut_unselected_alternate_with_binding() {
        let key = KeyShortcut::new(
            KeyMode::UnselectedAlternate,
            KeyAction::Session,
            Some(KeyWithModifier::new(BareKey::Char('0'))),
        );
        let color = colored_elements();

        let ret = long_mode_shortcut(&key, color, "+", &vec![], false);
        let ret = unstyle(ret);

        assert_eq!(ret, "+ <0> SESSION +".to_string());
    }

    #[test]
    //  没有绑定的 KeyShortcut 仅在 "disabled" 时显示（用于锁定模式指示）
    fn long_mode_shortcut_selected_without_binding() {
        let key = KeyShortcut::new(KeyMode::Selected, KeyAction::Session, None);
        let color = colored_elements();

        let ret = long_mode_shortcut(&key, color, "+", &vec![], false);
        let ret = unstyle(ret);

        assert_eq!(ret, "".to_string());
    }

    #[test]
    //  第一个块不打印起始分隔符
    fn long_mode_shortcut_selected_with_binding_first_tile() {
        let key = KeyShortcut::new(
            KeyMode::Selected,
            KeyAction::Session,
            Some(KeyWithModifier::new(BareKey::Char('0'))),
        );
        let color = colored_elements();

        let ret = long_mode_shortcut(&key, color, "+", &vec![], true);
        let ret = unstyle(ret);

        assert_eq!(ret, " <0> SESSION +".to_string());
    }

    #[test]
    //  修饰键是超级键，不应出现在尖括号中
    fn long_mode_shortcut_selected_with_ctrl_binding_shared_superkey() {
        let key = KeyShortcut::new(
            KeyMode::Selected,
            KeyAction::Session,
            Some(KeyWithModifier::new(BareKey::Char('0')).with_ctrl_modifier()),
        );
        let color = colored_elements();

        let ret = long_mode_shortcut(&key, color, "+", &vec![KeyModifier::Ctrl], false);
        let ret = unstyle(ret);

        assert_eq!(ret, "+ <0> SESSION +".to_string());
    }

    #[test]
    //  修饰键必须在尖括号中
    fn long_mode_shortcut_selected_with_ctrl_binding_no_shared_superkey() {
        let key = KeyShortcut::new(
            KeyMode::Selected,
            KeyAction::Session,
            Some(KeyWithModifier::new(BareKey::Char('0')).with_ctrl_modifier()),
        );
        let color = colored_elements();

        let ret = long_mode_shortcut(&key, color, "+", &vec![], false);
        let ret = unstyle(ret);

        assert_eq!(ret, "+ <Ctrl 0> SESSION +".to_string());
    }

    #[test]
    //  必须照常显示，但样式为灰色，我们在此不测试
    fn long_mode_shortcut_disabled_with_binding() {
        let key = KeyShortcut::new(
            KeyMode::Disabled,
            KeyAction::Session,
            Some(KeyWithModifier::new(BareKey::Char('0'))),
        );
        let color = colored_elements();

        let ret = long_mode_shortcut(&key, color, "+", &vec![], false);
        let ret = unstyle(ret);

        assert_eq!(ret, "+ <0> SESSION +".to_string());
    }

    #[test]
    //  必须显示但没有快捷键绑定
    fn long_mode_shortcut_disabled_without_binding() {
        let key = KeyShortcut::new(KeyMode::Disabled, KeyAction::Session, None);
        let color = colored_elements();

        let ret = long_mode_shortcut(&key, color, "+", &vec![], false);
        let ret = unstyle(ret);

        assert_eq!(ret, "+ <> SESSION +".to_string());
    }

    #[test]
    //  一次性测试全部
    // 注意，当 "shared_super" 为 true 时，块**不能**是行上的第一个，因此我们
    // 在这里忽略 **first**。
    fn long_mode_shortcut_selected_with_ctrl_binding_and_shared_super_and_first_tile() {
        let key = KeyShortcut::new(
            KeyMode::Selected,
            KeyAction::Session,
            Some(KeyWithModifier::new(BareKey::Char('0')).with_ctrl_modifier()),
        );
        let color = colored_elements();

        let ret = long_mode_shortcut(&key, color, "+", &vec![KeyModifier::Ctrl], true);
        let ret = unstyle(ret);

        assert_eq!(ret, "+ <0> SESSION +".to_string());
    }

    #[test]
    fn short_mode_shortcut_selected_with_binding() {
        let key = KeyShortcut::new(
            KeyMode::Selected,
            KeyAction::Session,
            Some(KeyWithModifier::new(BareKey::Char('0'))),
        );
        let color = colored_elements();

        let ret = short_mode_shortcut(&key, color, "+", &vec![], false);
        let ret = unstyle(ret);

        assert_eq!(ret, "+ 0 +".to_string());
    }

    #[test]
    fn short_mode_shortcut_selected_with_ctrl_binding_no_shared_super() {
        let key = KeyShortcut::new(
            KeyMode::Selected,
            KeyAction::Session,
            Some(KeyWithModifier::new(BareKey::Char('0')).with_ctrl_modifier()),
        );
        let color = colored_elements();

        let ret = short_mode_shortcut(&key, color, "+", &vec![], false);
        let ret = unstyle(ret);

        assert_eq!(ret, "+ Ctrl 0 +".to_string());
    }

    #[test]
    fn short_mode_shortcut_selected_with_ctrl_binding_shared_super() {
        let key = KeyShortcut::new(
            KeyMode::Selected,
            KeyAction::Session,
            Some(KeyWithModifier::new(BareKey::Char('0')).with_ctrl_modifier()),
        );
        let color = colored_elements();

        let ret = short_mode_shortcut(&key, color, "+", &vec![KeyModifier::Ctrl], false);
        let ret = unstyle(ret);

        assert_eq!(ret, "+ 0 +".to_string());
    }

    #[test]
    fn short_mode_shortcut_selected_with_binding_first_tile() {
        let key = KeyShortcut::new(
            KeyMode::Selected,
            KeyAction::Session,
            Some(KeyWithModifier::new(BareKey::Char('0'))),
        );
        let color = colored_elements();

        let ret = short_mode_shortcut(&key, color, "+", &vec![], true);
        let ret = unstyle(ret);

        assert_eq!(ret, " 0 +".to_string());
    }

    #[test]
    fn short_mode_shortcut_unselected_with_binding() {
        let key = KeyShortcut::new(
            KeyMode::Unselected,
            KeyAction::Session,
            Some(KeyWithModifier::new(BareKey::Char('0'))),
        );
        let color = colored_elements();

        let ret = short_mode_shortcut(&key, color, "+", &vec![], false);
        let ret = unstyle(ret);

        assert_eq!(ret, "+ 0 +".to_string());
    }

    #[test]
    fn short_mode_shortcut_unselected_alternate_with_binding() {
        let key = KeyShortcut::new(
            KeyMode::UnselectedAlternate,
            KeyAction::Session,
            Some(KeyWithModifier::new(BareKey::Char('0'))),
        );
        let color = colored_elements();

        let ret = short_mode_shortcut(&key, color, "+", &vec![], false);
        let ret = unstyle(ret);

        assert_eq!(ret, "+ 0 +".to_string());
    }

    #[test]
    fn short_mode_shortcut_disabled_with_binding() {
        let key = KeyShortcut::new(
            KeyMode::Selected,
            KeyAction::Session,
            Some(KeyWithModifier::new(BareKey::Char('0'))),
        );
        let color = colored_elements();

        let ret = short_mode_shortcut(&key, color, "+", &vec![], false);
        let ret = unstyle(ret);

        assert_eq!(ret, "+ 0 +".to_string());
    }

    #[test]
    fn short_mode_shortcut_selected_without_binding() {
        let key = KeyShortcut::new(KeyMode::Selected, KeyAction::Session, None);
        let color = colored_elements();

        let ret = short_mode_shortcut(&key, color, "+", &vec![], false);
        let ret = unstyle(ret);

        assert_eq!(ret, "".to_string());
    }

    #[test]
    fn short_mode_shortcut_unselected_without_binding() {
        let key = KeyShortcut::new(KeyMode::Unselected, KeyAction::Session, None);
        let color = colored_elements();

        let ret = short_mode_shortcut(&key, color, "+", &vec![], false);
        let ret = unstyle(ret);

        assert_eq!(ret, "".to_string());
    }

    #[test]
    fn short_mode_shortcut_unselected_alternate_without_binding() {
        let key = KeyShortcut::new(KeyMode::UnselectedAlternate, KeyAction::Session, None);
        let color = colored_elements();

        let ret = short_mode_shortcut(&key, color, "+", &vec![], false);
        let ret = unstyle(ret);

        assert_eq!(ret, "".to_string());
    }

    #[test]
    fn short_mode_shortcut_disabled_without_binding() {
        let key = KeyShortcut::new(KeyMode::Selected, KeyAction::Session, None);
        let color = colored_elements();

        let ret = short_mode_shortcut(&key, color, "+", &vec![], false);
        let ret = unstyle(ret);

        assert_eq!(ret, "".to_string());
    }

    #[test]
    //  观察：中间缺失的模式不会显示！
    fn first_line_default_layout_shared_super() {
        #[rustfmt::skip]
        let mode_info = ModeInfo{
            mode: InputMode::Normal,
            keybinds : vec![
                (InputMode::Normal, vec![
                    (KeyWithModifier::new(BareKey::Char('a')).with_ctrl_modifier(), vec![Action::SwitchToMode{input_mode: InputMode::Pane}]),
                    (KeyWithModifier::new(BareKey::Char('b')).with_ctrl_modifier(), vec![Action::SwitchToMode{input_mode: InputMode::Resize}]),
                    (KeyWithModifier::new(BareKey::Char('c')).with_ctrl_modifier(), vec![Action::SwitchToMode{input_mode: InputMode::Move}]),
                ]),
            ],
            ..ModeInfo::default()
        };

        let ret = first_line(&mode_info, None, 500, ">");
        let ret = unstyle(ret);

        assert_eq!(
            ret,
            " Ctrl + >> <a> PANE >> <b> RESIZE >> <c> MOVE >".to_string()
        );
    }

    #[test]
    fn first_line_default_layout_no_shared_super() {
        #[rustfmt::skip]
        let mode_info = ModeInfo{
            mode: InputMode::Normal,
            keybinds : vec![
                (InputMode::Normal, vec![
                    (KeyWithModifier::new(BareKey::Char('a')).with_ctrl_modifier(), vec![Action::SwitchToMode{input_mode: InputMode::Pane}]),
                    (KeyWithModifier::new(BareKey::Char('b')).with_ctrl_modifier(), vec![Action::SwitchToMode{input_mode: InputMode::Resize}]),
                    (KeyWithModifier::new(BareKey::Char('c')), vec![Action::SwitchToMode{input_mode: InputMode::Move}]),
                ]),
            ],
            ..ModeInfo::default()
        };

        let ret = first_line(&mode_info, None, 500, ">");
        let ret = unstyle(ret);

        assert_eq!(
            ret,
            " <Ctrl a> PANE >> <Ctrl b> RESIZE >> <c> MOVE >".to_string()
        );
    }

    #[test]
    fn first_line_default_layout_unprintables() {
        #[rustfmt::skip]
        let mode_info = ModeInfo{
            mode: InputMode::Normal,
            keybinds : vec![
                (InputMode::Normal, vec![
                    (KeyWithModifier::new(BareKey::Char('a')).with_ctrl_modifier(), vec![Action::SwitchToMode{input_mode: InputMode::Locked}]),
                    (KeyWithModifier::new(BareKey::Backspace), vec![Action::SwitchToMode{input_mode: InputMode::Pane}]),
                    (KeyWithModifier::new(BareKey::Enter), vec![Action::SwitchToMode{input_mode: InputMode::Tab}]),
                    (KeyWithModifier::new(BareKey::Tab), vec![Action::SwitchToMode{input_mode: InputMode::Resize}]),
                    (KeyWithModifier::new(BareKey::Left), vec![Action::SwitchToMode{input_mode: InputMode::Move}]),
                ]),
            ],
            ..ModeInfo::default()
        };

        let ret = first_line(&mode_info, None, 500, ">");
        let ret = unstyle(ret);

        assert_eq!(
            ret,
            " <Ctrl a> LOCK >> <BACKSPACE> PANE >> <ENTER> TAB >> <TAB> RESIZE >> <←> MOVE >"
                .to_string()
        );
    }

    #[test]
    fn first_line_short_layout_shared_super() {
        #[rustfmt::skip]
        let mode_info = ModeInfo{
            mode: InputMode::Normal,
            keybinds : vec![
                (InputMode::Normal, vec![
                    (KeyWithModifier::new(BareKey::Char('a')).with_ctrl_modifier(), vec![Action::SwitchToMode{input_mode: InputMode::Locked}]),
                    (KeyWithModifier::new(BareKey::Char('b')).with_ctrl_modifier(), vec![Action::SwitchToMode{input_mode: InputMode::Pane}]),
                    (KeyWithModifier::new(BareKey::Char('c')).with_ctrl_modifier(), vec![Action::SwitchToMode{input_mode: InputMode::Tab}]),
                    (KeyWithModifier::new(BareKey::Char('d')).with_ctrl_modifier(), vec![Action::SwitchToMode{input_mode: InputMode::Resize}]),
                    (KeyWithModifier::new(BareKey::Char('e')).with_ctrl_modifier(), vec![Action::SwitchToMode{input_mode: InputMode::Move}]),
                ]),
            ],
            ..ModeInfo::default()
        };

        let ret = first_line(&mode_info, None, 50, ">");
        let ret = unstyle(ret);

        assert_eq!(ret, " Ctrl + >> a >> b >> c >> d >> e >".to_string());
    }

    #[test]
    fn first_line_short_simplified_ui_shared_super() {
        #[rustfmt::skip]
        let mode_info = ModeInfo{
            mode: InputMode::Normal,
            keybinds : vec![
                (InputMode::Normal, vec![
                    (KeyWithModifier::new(BareKey::Char('a')).with_ctrl_modifier(), vec![Action::SwitchToMode{input_mode: InputMode::Pane}]),
                    (KeyWithModifier::new(BareKey::Char('b')).with_ctrl_modifier(), vec![Action::SwitchToMode{input_mode: InputMode::Resize}]),
                    (KeyWithModifier::new(BareKey::Char('c')).with_ctrl_modifier(), vec![Action::SwitchToMode{input_mode: InputMode::Move}]),
                ]),
            ],
            ..ModeInfo::default()
        };

        let ret = first_line(&mode_info, None, 30, "");
        let ret = unstyle(ret);

        assert_eq!(ret, " Ctrl +  a  b  c ".to_string());
    }
}
