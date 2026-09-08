use ansi_term::{
    unstyled_len, ANSIString, ANSIStrings,
    Color::{Fixed, RGB},
    Style,
};
use zellij_tile::prelude::actions::Action;
use zellij_tile::prelude::*;
use zellij_tile_utils::palette_match;

use crate::{
    action_key, action_key_group, style_key_with_modifier,
    tip::{data::TIPS, TipFn},
    LinePart, MORE_MSG, TO_NORMAL,
};

fn full_length_shortcut(
    is_first_shortcut: bool,
    key: Vec<KeyWithModifier>,
    action: &str,
    palette: Styling,
) -> LinePart {
    if key.is_empty() {
        return LinePart::default();
    }

    let text_color = palette_match!(palette.text_unselected.base);

    let separator = if is_first_shortcut { " " } else { " / " };
    let mut bits: Vec<ANSIString> = vec![Style::new().fg(text_color).paint(separator)];
    bits.extend(style_key_with_modifier(&key, &palette, None));
    bits.push(
        Style::new()
            .fg(text_color)
            .bold()
            .paint(format!(" {}", action)),
    );
    let part = ANSIStrings(&bits);

    LinePart {
        part: part.to_string(),
        len: unstyled_len(&part),
    }
}

fn locked_interface_indication(palette: Styling) -> LinePart {
    let locked_text = " -- INTERFACE LOCKED -- ";
    let locked_text_len = locked_text.chars().count();
    let text_color = palette_match!(palette.text_unselected.base);
    let locked_styled_text = Style::new().fg(text_color).bold().paint(locked_text);
    LinePart {
        part: locked_styled_text.to_string(),
        len: locked_text_len,
    }
}

fn add_shortcut(
    help: &ModeInfo,
    linepart: &LinePart,
    text: &str,
    keys: Vec<KeyWithModifier>,
) -> LinePart {
    let shortcut = if linepart.len == 0 {
        full_length_shortcut(true, keys, text, help.style.colors)
    } else {
        full_length_shortcut(false, keys, text, help.style.colors)
    };

    let mut new_linepart = LinePart::default();
    new_linepart.len += linepart.len + shortcut.len;
    new_linepart.part = format!("{}{}", linepart.part, shortcut);
    new_linepart
}

fn full_shortcut_list_nonstandard_mode(help: &ModeInfo) -> LinePart {
    let mut line_part = LinePart::default();
    let keys_and_hints = get_keys_and_hints(help);

    for (long, _short, keys) in keys_and_hints.into_iter() {
        line_part = add_shortcut(help, &line_part, &long, keys.to_vec());
    }
    line_part
}

///  收集所有要显示的相关快捷键绑定和提示。
///
///创建一个包含以下条目的元组向量：
///
///- 当没有大小限制时，为此快捷键绑定显示的 String，
///- 如果整个第二行变得太长时显示的缩短 String（在合理情况下），
///- 映射到此按键提示的按键的 `Vec<Key>`
///
///此向量通过遍历当前 [`InputMode`] 的快捷键绑定并
///存储所有匹配预定义 `Action` 模式的快捷键绑定。例如，
///`InputMode::Pane` 输入模式通过以下方式确定为 "Move focus" 提示显示哪些按键：
///在快捷键绑定中搜索匹配 `Action::MoveFocus(_)` 操作的内容。由于默认
///默认情况下多个快捷键绑定映射到某些操作模式（例如 `Action::MoveFocus(_)` 绑定到
///"hjkl"、方向键和 "Alt + <hjkl>"），我们对所有快捷键绑定的向量进行去重
///在处理之前。
///
///因此，我们按当前 keymap 的 [`Key`] 排序，并通过绑定到按键的 `Vec<Action>` 操作向量对生成的排序
///向量去重。这样，当多个按键映射
///到相同的操作序列时，将显示在 [`Key`] 结构中首先出现的按键
///显示。
//  请不要让 rustfmt 调整格式。它会将函数拉伸到大约
//  三倍的长度，我们生成的所有快捷键绑定向量将变得几乎不可读
//  对人类来说。
#[rustfmt::skip]
fn get_keys_and_hints(mi: &ModeInfo) -> Vec<(String, String, Vec<KeyWithModifier>)> {
    use Action as A;
    use InputMode as IM;
    use Direction as Dir;
    use actions::SearchDirection as SDir;
    use actions::SearchOption as SOpt;

    let mut old_keymap = mi.get_mode_keybinds();
    let s = |string: &str| string.to_string();

    //  查找返回 "Normal" 输入模式的快捷键绑定。在这种情况下，我们优先选择 '\n' 而非其他
    //  选项。在下面对 keymap 去重之前在此处执行！
    let to_normal_keys = action_key(&old_keymap, &[TO_NORMAL]);
    let to_normal_key = if to_normal_keys.contains(&KeyWithModifier::new(BareKey::Enter)) {
        vec![KeyWithModifier::new(BareKey::Enter)]
    } else {
        //  如果 `to_normal_keys` 至少有一个按键，则产生 `vec![key]`，否则产生空 vec。
        to_normal_keys.into_iter().take(1).collect()
    };

    //  首先对快捷键绑定进行排序和去重。我们按 `Key` 排序，并按
    //  其 `Action` 向量去重。这里不稳定排序是可以的，因为如果用户将任何内容再次映射到
    //  同一个键，什么都可能发生...
    old_keymap.sort_unstable_by(|(keya, _), (keyb, _)| keya.partial_cmp(keyb).unwrap());

    let mut known_actions: Vec<Vec<Action>> = vec![];
    let mut km = vec![];
    for (key, acvec) in old_keymap {
        if known_actions.contains(&acvec) {
            //  此操作已知
            continue;
        } else {
            known_actions.push(acvec.to_vec());
            km.push((key, acvec));
        }
    }

    if mi.mode == IM::Pane { vec![
        (s("新建"), s("新建"), action_key(&km, &[A::NewPane{direction: None, pane_name: None, start_suppressed: false}, TO_NORMAL])),
        (s("更改焦点"), s("移动"),
            action_key_group(&km, &[&[A::MoveFocus{direction: Dir::Left}], &[A::MoveFocus{direction: Dir::Down}],
                &[A::MoveFocus{direction: Dir::Up}], &[A::MoveFocus{direction: Dir::Right}]])),
        (s("关闭"), s("关闭"), action_key(&km, &[A::CloseFocus, TO_NORMAL])),
        (s("重命名"), s("重命名"),
            action_key(&km, &[A::SwitchToMode{input_mode: IM::RenamePane}, A::PaneNameInput{input: vec![0]}])),
        (s("切换全屏"), s("全屏"), action_key(&km, &[A::ToggleFocusFullscreen, TO_NORMAL])),
        (s("切换浮动"), s("浮动"),
            action_key(&km, &[A::ToggleFloatingPanes, TO_NORMAL])),
        (s("切换嵌入"), s("嵌入"), action_key(&km, &[A::TogglePaneEmbedOrFloating, TO_NORMAL])),
        (s("选择窗格"), s("选择"), to_normal_key),
    ]} else if mi.mode == IM::Tab {
        // 使用默认绑定时，标签页的 "Move focus" 很棘手：它绑定所有方向键
        // 到移动标签页焦点（左/上向左，右/下向右）。由于我们对按键排序
        // 然后根据操作去重，我们最终会得到 LeftArrow 用于
        // "left" 和 DownArrow 用于 "right"。我们真正期望看到的是 LeftArrow 和
        // RightArrow.
        //  FIXME: 由于没有更好的办法，我们在这里手动检查这种情况。
        let old_keymap = mi.get_mode_keybinds();
        let focus_keys_full: Vec<KeyWithModifier> = action_key_group(&old_keymap,
            &[&[A::GoToPreviousTab], &[A::GoToNextTab]]);
        let focus_keys = if focus_keys_full.contains(&KeyWithModifier::new(BareKey::Left))
            && focus_keys_full.contains(&KeyWithModifier::new(BareKey::Right)) {
            vec![KeyWithModifier::new(BareKey::Left), KeyWithModifier::new(BareKey::Right)]
        } else {
            action_key_group(&km, &[&[A::GoToPreviousTab], &[A::GoToNextTab]])
        };

        vec![
        (s("新建"), s("新建"), action_key(&km, &[A::NewTab{
            tiled_layout: None,
            floating_layouts: vec![],
            swap_tiled_layouts: None,
            swap_floating_layouts: None,
            tab_name: None,
            should_change_focus_to_new_tab: true,
            cwd: None,
            initial_panes: None,
            first_pane_unblock_condition: None,
        }, TO_NORMAL])),
        (s("更改焦点"), s("移动"), focus_keys),
        (s("关闭"), s("关闭"), action_key(&km, &[A::CloseTab, TO_NORMAL])),
        (s("重命名"), s("重命名"),
            action_key(&km, &[A::SwitchToMode{input_mode: IM::RenameTab}, A::TabNameInput{input: vec![0]}])),
        (s("同步"), s("同步"), action_key(&km, &[A::ToggleActiveSyncTab, TO_NORMAL])),
        (s("将窗格移到新标签页"), s("移出"), action_key(&km, &[A::BreakPane, TO_NORMAL])),
        (s("将窗格左右拆分"), s("拆分"), action_key_group(&km, &[
            &[Action::BreakPaneLeft, TO_NORMAL],
            &[Action::BreakPaneRight, TO_NORMAL],
        ])),
        (s("切换"), s("切换"), action_key(&km, &[A::ToggleTab])),
        (s("选择窗格"), s("选择"), to_normal_key),
    ]} else if mi.mode == IM::Resize { vec![
        (s("增大/减小尺寸"), s("增大/减小"),
            action_key_group(&km, &[
                &[A::Resize{resize: Resize::Increase, direction: None}],
                &[A::Resize{resize: Resize::Decrease, direction: None}]
            ])),
        (s("增大到"), s("增大"), action_key_group(&km, &[
            &[A::Resize{resize: Resize::Increase, direction: Some(Dir::Left)}],
            &[A::Resize{resize: Resize::Increase, direction: Some(Dir::Down)}],
            &[A::Resize{resize: Resize::Increase, direction: Some(Dir::Up)}],
            &[A::Resize{resize: Resize::Increase, direction: Some(Dir::Right)}]
            ])),
        (s("从当前减小"), s("减小"), action_key_group(&km, &[
            &[A::Resize{resize: Resize::Decrease, direction: Some(Dir::Left)}],
            &[A::Resize{resize: Resize::Decrease, direction: Some(Dir::Down)}],
            &[A::Resize{resize: Resize::Decrease, direction: Some(Dir::Up)}],
            &[A::Resize{resize: Resize::Decrease, direction: Some(Dir::Right)}]
            ])),
        (s("选择窗格"), s("选择"), to_normal_key),
    ]} else if mi.mode == IM::Move { vec![
        (s("切换位置"), s("移动"), action_key_group(&km, &[
            &[Action::MovePane{direction: Some(Dir::Left)}], &[Action::MovePane{direction: Some(Dir::Down)}],
            &[Action::MovePane{direction: Some(Dir::Up)}], &[Action::MovePane{direction: Some(Dir::Right)}]])),
    ]} else if mi.mode == IM::Scroll { vec![
        (s("输入搜索词"), s("搜索"),
            action_key(&km, &[A::SwitchToMode{input_mode: IM::EnterSearch}, A::SearchInput{input: vec![0]}])),
        (s("滚动"), s("滚动"),
            action_key_group(&km, &[&[Action::ScrollDown], &[Action::ScrollUp]])),
        (s("整页滚动"), s("滚动"),
            action_key_group(&km, &[&[Action::PageScrollDown], &[Action::PageScrollUp]])),
        (s("半页滚动"), s("滚动"),
            action_key_group(&km, &[&[Action::HalfPageScrollDown], &[Action::HalfPageScrollUp]])),
        (s("在默认编辑器中编辑回滚缓冲区"), s("编辑"),
            action_key(&km, &[Action::EditScrollback { ansi: false }, TO_NORMAL])),
        (s("选择窗格"), s("选择"), to_normal_key),
    ]} else if mi.mode == IM::EnterSearch { vec![
        (s("完成后"), s("完成"), action_key(&km, &[A::SwitchToMode{input_mode: IM::Search}])),
        (s("取消"), s("取消"),
            action_key(&km, &[A::SearchInput{input: vec![27]}, A::SwitchToMode{input_mode: IM::Scroll}])),
    ]} else if mi.mode == IM::Search { vec![
        (s("输入搜索词"), s("搜索"),
            action_key(&km, &[A::SwitchToMode{input_mode: IM::EnterSearch}, A::SearchInput{input: vec![0]}])),
        (s("滚动"), s("滚动"),
            action_key_group(&km, &[&[Action::ScrollDown], &[Action::ScrollUp]])),
        (s("整页滚动"), s("滚动"),
            action_key_group(&km, &[&[Action::PageScrollDown], &[Action::PageScrollUp]])),
        (s("半页滚动"), s("滚动"),
            action_key_group(&km, &[&[Action::HalfPageScrollDown], &[Action::HalfPageScrollUp]])),
        (s("向下搜索"), s("下"), action_key(&km, &[A::Search{direction: SDir::Down}])),
        (s("向上搜索"), s("上"), action_key(&km, &[A::Search{direction: SDir::Up}])),
        (s("区分大小写"), s("区分"),
            action_key(&km, &[A::SearchToggleOption{option: SOpt::CaseSensitivity}])),
        (s("循环"), s("循环"),
            action_key(&km, &[A::SearchToggleOption{option: SOpt::Wrap}])),
        (s("整词匹配"), s("整词"),
            action_key(&km, &[A::SearchToggleOption{option: SOpt::WholeWord}])),
    ]} else if mi.mode == IM::Session { vec![
        (s("分离"), s("分离"), action_key(&km, &[Action::Detach])),
        (s("会话管理器"), s("管理器"), action_key(&km, &[A::LaunchOrFocusPlugin{plugin: Default::default(), should_float: true, move_to_focused_tab: true, should_open_in_place: false, close_replaced_pane: false, skip_cache: false, tab_id: None}, TO_NORMAL])), //  不完全准确
        (s("选择窗格"), s("选择"), to_normal_key),
    ]} else if mi.mode == IM::Tmux { vec![
        (s("移动焦点"), s("移动"), action_key_group(&km, &[
            &[A::MoveFocus{direction: Dir::Left}], &[A::MoveFocus{direction: Dir::Down}],
            &[A::MoveFocus{direction: Dir::Up}], &[A::MoveFocus{direction: Dir::Right}]])),
        (s("向下拆分"), s("下"), action_key(&km, &[A::NewPane{direction: Some(Dir::Down), pane_name: None, start_suppressed: false}, TO_NORMAL])),
        (s("向右拆分"), s("右"), action_key(&km, &[A::NewPane{direction: Some(Dir::Right), pane_name: None, start_suppressed: false}, TO_NORMAL])),
        (s("全屏"), s("全屏"), action_key(&km, &[A::ToggleFocusFullscreen, TO_NORMAL])),
        (s("New tab"), s("New"), action_key(&km, &[A::NewTab{
            tiled_layout: None,
            floating_layouts: vec![],
            swap_tiled_layouts: None,
            swap_floating_layouts: None,
            tab_name: None,
            should_change_focus_to_new_tab: true,
            cwd: None,
            initial_panes: None,
            first_pane_unblock_condition: None,
        }, TO_NORMAL])),
        (s("重命名标签页"), s("重命名"),
            action_key(&km, &[A::SwitchToMode{input_mode: IM::RenameTab}, A::TabNameInput{input: vec![0]}])),
        (s("上一个标签页"), s("上一个"), action_key(&km, &[A::GoToPreviousTab, TO_NORMAL])),
        (s("下一个标签页"), s("下一个"), action_key(&km, &[A::GoToNextTab, TO_NORMAL])),
        (s("选择窗格"), s("选择"), to_normal_key),
    ]} else if matches!(mi.mode, IM::RenamePane | IM::RenameTab) { vec![
        (s("完成后"), s("完成"), to_normal_key),
        (s("选择窗格"), s("选择"), action_key_group(&km, &[
            &[A::MoveFocus{direction: Dir::Left}], &[A::MoveFocus{direction: Dir::Down}],
            &[A::MoveFocus{direction: Dir::Up}], &[A::MoveFocus{direction: Dir::Right}]])),
    ]} else { vec![] }
}

fn full_shortcut_list(help: &ModeInfo, tip: TipFn) -> LinePart {
    match help.mode {
        InputMode::Normal => tip(help),
        InputMode::Locked => locked_interface_indication(help.style.colors),
        _ => full_shortcut_list_nonstandard_mode(help),
    }
}

fn shortened_shortcut_list_nonstandard_mode(help: &ModeInfo) -> LinePart {
    let mut line_part = LinePart::default();
    let keys_and_hints = get_keys_and_hints(help);

    for (_, short, keys) in keys_and_hints.into_iter() {
        line_part = add_shortcut(help, &line_part, &short, keys.to_vec());
    }
    line_part
}

fn shortened_shortcut_list(help: &ModeInfo, tip: TipFn) -> LinePart {
    match help.mode {
        InputMode::Normal => tip(help),
        InputMode::Locked => locked_interface_indication(help.style.colors),
        _ => shortened_shortcut_list_nonstandard_mode(help),
    }
}

fn best_effort_shortcut_list_nonstandard_mode(help: &ModeInfo, max_len: usize) -> LinePart {
    let mut line_part = LinePart::default();
    let keys_and_hints = get_keys_and_hints(help);

    for (_, short, keys) in keys_and_hints.into_iter() {
        let new_line_part = add_shortcut(help, &line_part, &short, keys.to_vec());
        if new_line_part.len + MORE_MSG.chars().count() > max_len {
            line_part.part = format!("{}{}", line_part.part, MORE_MSG);
            line_part.len += MORE_MSG.chars().count();
            break;
        }
        line_part = new_line_part;
    }
    line_part
}

fn best_effort_shortcut_list(help: &ModeInfo, tip: TipFn, max_len: usize) -> LinePart {
    match help.mode {
        InputMode::Normal => {
            let line_part = tip(help);
            if line_part.len <= max_len {
                line_part
            } else {
                LinePart::default()
            }
        },
        InputMode::Locked => {
            let line_part = locked_interface_indication(help.style.colors);
            if line_part.len <= max_len {
                line_part
            } else {
                LinePart::default()
            }
        },
        _ => best_effort_shortcut_list_nonstandard_mode(help, max_len),
    }
}

pub fn keybinds(help: &ModeInfo, tip_name: &str, max_width: usize) -> LinePart {
    //  假设 TIPS HashMap 中至少有一个 TIP 数据。
    let tip_body = TIPS
        .get(tip_name)
        .unwrap_or_else(|| TIPS.get("quicknav").unwrap());

    let full_shortcut_list = full_shortcut_list(help, tip_body.full);
    if full_shortcut_list.len <= max_width {
        return full_shortcut_list;
    }
    let shortened_shortcut_list = shortened_shortcut_list(help, tip_body.medium);
    if shortened_shortcut_list.len <= max_width {
        return shortened_shortcut_list;
    }
    best_effort_shortcut_list(help, tip_body.short, max_width)
}

pub fn descended_into_nested_session_hint(help: &ModeInfo, max_len: usize) -> LinePart {
    nested_session_status_hint(help, "上升： ", &help.nested_ascend_keys, max_len)
}

pub fn ascended_to_host_session_hint(help: &ModeInfo, max_len: usize) -> LinePart {
    nested_session_status_hint(help, "下降： ", &help.nested_descend_keys, max_len)
}

fn nested_session_status_hint(
    help: &ModeInfo,
    label: &str,
    keys: &[KeyWithModifier],
    max_len: usize,
) -> LinePart {
    let palette = help.style.colors;
    let text_color = palette_match!(palette.text_unselected.base);
    let styled_label = Style::new().dimmed().italic().paint(format!(" {}", label));
    let mut key_bits: Vec<ANSIString> = vec![];
    if keys.is_empty() {
        key_bits.push(Style::new().dimmed().italic().paint("<unbound>"));
    } else {
        for (i, key) in keys.iter().enumerate() {
            if i > 0 {
                key_bits.push(Style::new().fg(text_color).paint(" + "));
            }
            key_bits.extend(style_key_with_modifier(
                std::slice::from_ref(key),
                &palette,
                None,
            ));
        }
    }
    let mut bits: Vec<ANSIString> = vec![styled_label];
    bits.extend(key_bits.clone());
    let part = ANSIStrings(&bits);
    let len = unstyled_len(&part);
    if len <= max_len {
        return LinePart {
            part: part.to_string(),
            len,
        };
    }
    let part = ANSIStrings(&key_bits);
    let len = unstyled_len(&part);
    if len <= max_len {
        LinePart {
            part: part.to_string(),
            len,
        }
    } else {
        LinePart::default()
    }
}

pub fn text_copied_hint(copy_destination: CopyDestination) -> LinePart {
    let hint = match copy_destination {
        CopyDestination::Command => "Text piped to external command",
        #[cfg(not(target_os = "macos"))]
        CopyDestination::Primary => "Text copied to system primary selection",
        #[cfg(target_os = "macos")] //  macOS 上不存在主选区
        CopyDestination::Primary => "Text copied to system clipboard",
        CopyDestination::System => "Text copied to system clipboard",
    };
    LinePart {
        part: serialize_text(&Text::new(&hint).color_range(2, ..).opaque()),
        len: hint.len(),
    }
}

pub fn system_clipboard_error(palette: &Styling) -> LinePart {
    let hint = " Error using the system clipboard.";
    let red_color = palette_match!(palette.text_unselected.emphasis_3);
    LinePart {
        part: Style::new().fg(red_color).bold().paint(hint).to_string(),
        len: hint.len(),
    }
}

pub fn fullscreen_panes_to_hide(palette: &Styling, panes_to_hide: usize) -> LinePart {
    let text_color = palette_match!(palette.text_unselected.base);
    let green_color = palette_match!(palette.text_unselected.emphasis_2);
    let orange_color = palette_match!(palette.text_unselected.emphasis_0);
    let shortcut_left_separator = Style::new().fg(text_color).bold().paint(" (");
    let shortcut_right_separator = Style::new().fg(text_color).bold().paint("): ");
    let fullscreen = "FULLSCREEN";
    let puls = "+ ";
    let panes = panes_to_hide.to_string();
    let hide = " hidden panes";
    let len = fullscreen.chars().count()
        + puls.chars().count()
        + panes.chars().count()
        + hide.chars().count()
        + 5; //  3 用于快捷键周围的 ():，2 用于空格
    LinePart {
        part: format!(
            "{}{}{}{}{}{}",
            shortcut_left_separator,
            Style::new().fg(orange_color).bold().paint(fullscreen),
            shortcut_right_separator,
            Style::new().fg(text_color).bold().paint(puls),
            Style::new().fg(green_color).bold().paint(panes),
            Style::new().fg(text_color).bold().paint(hide)
        ),
        len,
    }
}

pub fn floating_panes_are_visible(mode_info: &ModeInfo) -> LinePart {
    let palette = mode_info.style.colors;
    let km = &mode_info.get_mode_keybinds();
    let white_color = palette_match!(palette.text_unselected.base);
    let green_color = palette_match!(palette.text_unselected.emphasis_2);
    let orange_color = palette_match!(palette.text_unselected.emphasis_0);
    let shortcut_left_separator = Style::new().fg(white_color).bold().paint(" (");
    let shortcut_right_separator = Style::new().fg(white_color).bold().paint("): ");
    let floating_panes = "FLOATING PANES VISIBLE";
    let press = "Press ";
    let pane_mode = format!(
        "{}",
        action_key(
            km,
            &[Action::SwitchToMode {
                input_mode: InputMode::Pane
            }]
        )
        .first()
        .unwrap_or(&KeyWithModifier::new(BareKey::Char('?')))
    );
    let plus = ", ";
    let p_left_separator = "<";
    let p = format!(
        "{}",
        action_key(
            &mode_info.get_keybinds_for_mode(InputMode::Pane),
            &[Action::ToggleFloatingPanes, TO_NORMAL]
        )
        .first()
        .unwrap_or(&KeyWithModifier::new(BareKey::Char('?')))
    );
    let p_right_separator = "> ";
    let to_hide = "to hide.";

    let len = floating_panes.chars().count()
        + press.chars().count()
        + pane_mode.chars().count()
        + plus.chars().count()
        + p_left_separator.chars().count()
        + p.chars().count()
        + p_right_separator.chars().count()
        + to_hide.chars().count()
        + 5; //  3 用于 floating_panes 周围的 ():，2 用于空格
    LinePart {
        part: format!(
            "{}{}{}{}{}{}{}{}{}{}",
            shortcut_left_separator,
            Style::new().fg(orange_color).bold().paint(floating_panes),
            shortcut_right_separator,
            Style::new().fg(white_color).bold().paint(press),
            Style::new().fg(green_color).bold().paint(pane_mode),
            Style::new().fg(white_color).bold().paint(plus),
            Style::new().fg(white_color).bold().paint(p_left_separator),
            Style::new().fg(green_color).bold().paint(p),
            Style::new().fg(white_color).bold().paint(p_right_separator),
            Style::new().fg(white_color).bold().paint(to_hide),
        ),
        len,
    }
}

pub fn locked_fullscreen_panes_to_hide(palette: &Styling, panes_to_hide: usize) -> LinePart {
    let text_color = palette_match!(palette.text_unselected.base);
    let green_color = palette_match!(palette.text_unselected.emphasis_2);
    let orange_color = palette_match!(palette.text_unselected.emphasis_0);
    let locked_text = " -- INTERFACE LOCKED -- ";
    let shortcut_left_separator = Style::new().fg(text_color).bold().paint(" (");
    let shortcut_right_separator = Style::new().fg(text_color).bold().paint("): ");
    let fullscreen = "FULLSCREEN";
    let puls = "+ ";
    let panes = panes_to_hide.to_string();
    let hide = " hidden panes";
    let len = locked_text.chars().count()
        + fullscreen.chars().count()
        + puls.chars().count()
        + panes.chars().count()
        + hide.chars().count()
        + 5; //  3 用于快捷键周围的 ():，2 用于空格
    LinePart {
        part: format!(
            "{}{}{}{}{}{}{}",
            Style::new().fg(text_color).bold().paint(locked_text),
            shortcut_left_separator,
            Style::new().fg(orange_color).bold().paint(fullscreen),
            shortcut_right_separator,
            Style::new().fg(text_color).bold().paint(puls),
            Style::new().fg(green_color).bold().paint(panes),
            Style::new().fg(text_color).bold().paint(hide)
        ),
        len,
    }
}

pub fn locked_floating_panes_are_visible(palette: &Styling) -> LinePart {
    let white_color = palette_match!(palette.text_unselected.base);
    let orange_color = palette_match!(palette.text_unselected.emphasis_0);
    let shortcut_left_separator = Style::new().fg(white_color).bold().paint(" (");
    let shortcut_right_separator = Style::new().fg(white_color).bold().paint(")");
    let locked_text = " -- INTERFACE LOCKED -- ";
    let floating_panes = "FLOATING PANES VISIBLE";

    let len = locked_text.chars().count() + floating_panes.chars().count();
    LinePart {
        part: format!(
            "{}{}{}{}",
            Style::new().fg(white_color).bold().paint(locked_text),
            shortcut_left_separator,
            Style::new().fg(orange_color).bold().paint(floating_panes),
            shortcut_right_separator,
        ),
        len,
    }
}

#[cfg(test)]
///  单元测试。
///
///注意我们在这里取了一点巧，因为可能想要测试的东西数量是无穷无尽的，
///而为所有这些测试用例手动创建 [`ModeInfo`] 的 Mockup 简直是
///一种折磨。因此，我们彻底测试最原子的单元（[`full_length_shortcut`]），然后测试
///公共 API（[`keybinds`]）以确保正确运行。
mod tests {
    use super::*;

    //  从 `LinePart` 中剥离样式信息并返回原始 String
    fn unstyle(line_part: LinePart) -> String {
        let string = line_part.to_string();

        let re = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
        let string = re.replace_all(&string, "".to_string());

        string.to_string()
    }

    #[test]
    fn full_length_shortcut_with_key() {
        let keyvec = vec![KeyWithModifier::new(BareKey::Char('a'))];
        let palette = Styling::default();

        let ret = full_length_shortcut(false, keyvec, "Foobar", palette);
        let ret = unstyle(ret);

        assert_eq!(ret, " / <a> Foobar");
    }

    #[test]
    fn full_length_shortcut_with_key_first_element() {
        let keyvec = vec![KeyWithModifier::new(BareKey::Char('a'))];
        let palette = Styling::default();

        let ret = full_length_shortcut(true, keyvec, "Foobar", palette);
        let ret = unstyle(ret);

        assert_eq!(ret, " <a> Foobar");
    }

    #[test]
    //  当没有绑定时，我们也不打印快捷方式
    fn full_length_shortcut_without_key() {
        let keyvec = vec![];
        let palette = Styling::default();

        let ret = full_length_shortcut(false, keyvec, "Foobar", palette);
        let ret = unstyle(ret);

        assert_eq!(ret, "");
    }

    #[test]
    fn full_length_shortcut_with_key_unprintable_1() {
        let keyvec = vec![KeyWithModifier::new(BareKey::Enter)];
        let palette = Styling::default();

        let ret = full_length_shortcut(false, keyvec, "Foobar", palette);
        let ret = unstyle(ret);

        assert_eq!(ret, " / <ENTER> Foobar");
    }

    #[test]
    fn full_length_shortcut_with_key_unprintable_2() {
        let keyvec = vec![KeyWithModifier::new(BareKey::Backspace)];
        let palette = Styling::default();

        let ret = full_length_shortcut(false, keyvec, "Foobar", palette);
        let ret = unstyle(ret);

        assert_eq!(ret, " / <BACKSPACE> Foobar");
    }

    #[test]
    fn full_length_shortcut_with_ctrl_key() {
        let keyvec = vec![KeyWithModifier::new(BareKey::Char('a')).with_ctrl_modifier()];
        let palette = Styling::default();

        let ret = full_length_shortcut(false, keyvec, "Foobar", palette);
        let ret = unstyle(ret);

        assert_eq!(ret, " / Ctrl + <a> Foobar");
    }

    #[test]
    fn full_length_shortcut_with_alt_key() {
        let keyvec = vec![KeyWithModifier::new(BareKey::Char('a')).with_alt_modifier()];
        let palette = Styling::default();

        let ret = full_length_shortcut(false, keyvec, "Foobar", palette);
        let ret = unstyle(ret);

        assert_eq!(ret, " / Alt + <a> Foobar");
    }

    #[test]
    fn full_length_shortcut_with_homogenous_key_group() {
        let keyvec = vec![
            KeyWithModifier::new(BareKey::Char('a')),
            KeyWithModifier::new(BareKey::Char('b')),
            KeyWithModifier::new(BareKey::Char('c')),
        ];
        let palette = Styling::default();

        let ret = full_length_shortcut(false, keyvec, "Foobar", palette);
        let ret = unstyle(ret);

        assert_eq!(ret, " / <a|b|c> Foobar");
    }

    #[test]
    fn full_length_shortcut_with_heterogenous_key_group() {
        let keyvec = vec![
            KeyWithModifier::new(BareKey::Char('a')),
            KeyWithModifier::new(BareKey::Char('b')).with_ctrl_modifier(),
            KeyWithModifier::new(BareKey::Enter),
        ];
        let palette = Styling::default();

        let ret = full_length_shortcut(false, keyvec, "Foobar", palette);
        let ret = unstyle(ret);

        assert_eq!(ret, " / <a|Ctrl b|ENTER> Foobar");
    }

    #[test]
    fn full_length_shortcut_with_key_group_shared_ctrl_modifier() {
        let keyvec = vec![
            KeyWithModifier::new(BareKey::Char('a')).with_ctrl_modifier(),
            KeyWithModifier::new(BareKey::Char('b')).with_ctrl_modifier(),
            KeyWithModifier::new(BareKey::Char('c')).with_ctrl_modifier(),
        ];
        let palette = Styling::default();

        let ret = full_length_shortcut(false, keyvec, "Foobar", palette);
        let ret = unstyle(ret);

        assert_eq!(ret, " / Ctrl + <a|b|c> Foobar");
    }
    //pub fn keybinds(help: &ModeInfo, tip_name: &str, max_width: usize) -> LinePart {

    #[test]
    //  注意它如何省略不存在的元素！
    fn keybinds_wide() {
        let mode_info = ModeInfo {
            mode: InputMode::Pane,
            keybinds: vec![(
                InputMode::Pane,
                vec![
                    (
                        KeyWithModifier::new(BareKey::Left),
                        vec![Action::MoveFocus {
                            direction: Direction::Left,
                        }],
                    ),
                    (
                        KeyWithModifier::new(BareKey::Down),
                        vec![Action::MoveFocus {
                            direction: Direction::Down,
                        }],
                    ),
                    (
                        KeyWithModifier::new(BareKey::Up),
                        vec![Action::MoveFocus {
                            direction: Direction::Up,
                        }],
                    ),
                    (
                        KeyWithModifier::new(BareKey::Right),
                        vec![Action::MoveFocus {
                            direction: Direction::Right,
                        }],
                    ),
                    (
                        KeyWithModifier::new(BareKey::Char('n')),
                        vec![
                            Action::NewPane {
                                direction: None,
                                pane_name: None,
                                start_suppressed: false,
                            },
                            TO_NORMAL,
                        ],
                    ),
                    (
                        KeyWithModifier::new(BareKey::Char('x')),
                        vec![Action::CloseFocus, TO_NORMAL],
                    ),
                    (
                        KeyWithModifier::new(BareKey::Char('f')),
                        vec![Action::ToggleFocusFullscreen, TO_NORMAL],
                    ),
                ],
            )],
            ..ModeInfo::default()
        };

        let ret = keybinds(&mode_info, "quicknav", 500);
        let ret = unstyle(ret);

        assert_eq!(
            ret,
            " <n> New / <←↓↑→> Change Focus / <x> Close / <f> Toggle Fullscreen",
        );
    }

    #[test]
    //  注意 "Move focus" 如何变成 "移动"
    fn keybinds_tight_width() {
        let mode_info = ModeInfo {
            mode: InputMode::Pane,
            keybinds: vec![(
                InputMode::Pane,
                vec![
                    (
                        KeyWithModifier::new(BareKey::Left),
                        vec![Action::MoveFocus {
                            direction: Direction::Left,
                        }],
                    ),
                    (
                        KeyWithModifier::new(BareKey::Down),
                        vec![Action::MoveFocus {
                            direction: Direction::Down,
                        }],
                    ),
                    (
                        KeyWithModifier::new(BareKey::Up),
                        vec![Action::MoveFocus {
                            direction: Direction::Up,
                        }],
                    ),
                    (
                        KeyWithModifier::new(BareKey::Right),
                        vec![Action::MoveFocus {
                            direction: Direction::Right,
                        }],
                    ),
                    (
                        KeyWithModifier::new(BareKey::Char('n')),
                        vec![
                            Action::NewPane {
                                direction: None,
                                pane_name: None,
                                start_suppressed: false,
                            },
                            TO_NORMAL,
                        ],
                    ),
                    (
                        KeyWithModifier::new(BareKey::Char('x')),
                        vec![Action::CloseFocus, TO_NORMAL],
                    ),
                    (
                        KeyWithModifier::new(BareKey::Char('f')),
                        vec![Action::ToggleFocusFullscreen, TO_NORMAL],
                    ),
                ],
            )],
            ..ModeInfo::default()
        };

        let ret = keybinds(&mode_info, "quicknav", 35);
        let ret = unstyle(ret);

        assert_eq!(ret, " <n> New / <←↓↑→> Move ... ");
    }

    #[test]
    fn keybinds_wide_weird_keys() {
        let mode_info = ModeInfo {
            mode: InputMode::Pane,
            keybinds: vec![(
                InputMode::Pane,
                vec![
                    (
                        KeyWithModifier::new(BareKey::Char('a')).with_ctrl_modifier(),
                        vec![Action::MoveFocus {
                            direction: Direction::Left,
                        }],
                    ),
                    (
                        KeyWithModifier::new(BareKey::Enter).with_ctrl_modifier(),
                        vec![Action::MoveFocus {
                            direction: Direction::Down,
                        }],
                    ),
                    (
                        KeyWithModifier::new(BareKey::Char('1')).with_ctrl_modifier(),
                        vec![Action::MoveFocus {
                            direction: Direction::Up,
                        }],
                    ),
                    (
                        KeyWithModifier::new(BareKey::Char(' ')).with_ctrl_modifier(),
                        vec![Action::MoveFocus {
                            direction: Direction::Right,
                        }],
                    ),
                    (
                        KeyWithModifier::new(BareKey::Backspace),
                        vec![
                            Action::NewPane {
                                direction: None,
                                pane_name: None,
                                start_suppressed: false,
                            },
                            TO_NORMAL,
                        ],
                    ),
                    (
                        KeyWithModifier::new(BareKey::Esc),
                        vec![Action::CloseFocus, TO_NORMAL],
                    ),
                    (
                        KeyWithModifier::new(BareKey::End),
                        vec![Action::ToggleFocusFullscreen, TO_NORMAL],
                    ),
                ],
            )],
            ..ModeInfo::default()
        };

        let ret = keybinds(&mode_info, "quicknav", 500);
        let ret = unstyle(ret);

        assert_eq!(ret, " <BACKSPACE> New / Ctrl + <a|ENTER|1|SPACE> Change Focus / <ESC> Close / <END> Toggle Fullscreen");
    }
}
