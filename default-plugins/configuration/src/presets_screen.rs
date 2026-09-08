use zellij_tile::prelude::*;

use crate::ui_components::info_line;

use crate::rebind_leaders_screen::RebindLeadersScreen;
use std::collections::BTreeSet;

use crate::presets::{default_keybinds, unlock_first_keybinds};

#[derive(Debug)]
pub struct PresetsScreen {
    selected_index: Option<usize>,
    latest_mode_info: Option<ModeInfo>,
    notification: Option<String>,
    primary_modifier: BTreeSet<KeyModifier>,
    secondary_modifier: BTreeSet<KeyModifier>,
    rebind_leaders_screen: Option<RebindLeadersScreen>,
}

impl Default for PresetsScreen {
    fn default() -> Self {
        let mut primary_modifier = BTreeSet::new();
        primary_modifier.insert(KeyModifier::Ctrl);
        let mut secondary_modifier = BTreeSet::new();
        secondary_modifier.insert(KeyModifier::Alt);
        PresetsScreen {
            primary_modifier,
            secondary_modifier,
            selected_index: None,
            latest_mode_info: None,
            notification: None,
            rebind_leaders_screen: None,
        }
    }
}

impl PresetsScreen {
    pub fn new(selected_index: Option<usize>) -> Self {
        PresetsScreen {
            selected_index,
            ..Default::default()
        }
    }
    pub fn rebinding_leaders(&self) -> bool {
        self.rebind_leaders_screen.is_some()
    }
    pub fn handle_presets_key(&mut self, key: KeyWithModifier) -> bool {
        if let Some(rebind_leaders_screen) = self.rebind_leaders_screen.as_mut() {
            match key.bare_key {
                BareKey::Esc if key.has_no_modifiers() => {
                    // consume screen without applying its modifiers
                    drop(self.rebind_leaders_screen.take());
                    return true;
                },
                BareKey::Enter if key.has_no_modifiers() => {
                    // consume screen and apply its modifiers
                    let (primary_modifier, secondary_modifier) =
                        rebind_leaders_screen.primary_and_secondary_modifiers();
                    self.primary_modifier = primary_modifier;
                    self.secondary_modifier = secondary_modifier;
                    drop(self.rebind_leaders_screen.take());
                    return true;
                },
                _ => {
                    return rebind_leaders_screen.handle_key(key);
                },
            }
        }
        let mut should_render = false;
        if self.notification.is_some() {
            self.notification = None;
            should_render = true;
        } else if key.bare_key == BareKey::Down && key.has_no_modifiers() {
            self.move_selected_index_down();
            should_render = true;
        } else if key.bare_key == BareKey::Up && key.has_no_modifiers() {
            self.move_selected_index_up();
            should_render = true;
        } else if key.bare_key == BareKey::Enter && key.has_no_modifiers() {
            if let Some(selected_index) = self.selected_index.take() {
                let write_to_disk = false;
                self.reconfigure(selected_index, write_to_disk);
                self.notification = Some("配置已应用到当前会话。".to_owned());
            } else {
                self.reset_selected_index();
            }
            should_render = true;
        } else if key.bare_key == BareKey::Char('a') && key.has_modifiers(&[KeyModifier::Ctrl]) {
            if let Some(selected_index) = self.take_selected_index() {
                let write_to_disk = true;
                self.reconfigure(selected_index, write_to_disk);
                self.notification = Some("配置已应用并保存到磁盘。".to_owned());
                should_render = true;
            }
        } else if key.bare_key == BareKey::Char('l') && key.has_no_modifiers() {
            // for the time being this screen has been disabled because it was deemed too confusing
            // and its use-cases are very limited (it's possible to achieve the same results by
            // applying a preset and then rebinding the leader keys)
            //
            // the code is left here in case someone feels strongly about implementing this on
            // their own, and because at the time of writing I'm a little ambiguous about this
            // decision. At some point it should be refactored away
            //             self.rebind_leaders_screen = Some(
            //                 RebindLeadersScreen::default()
            //                     .with_rebinding_for_presets()
            //                     .with_mode_info(self.latest_mode_info.clone()),
            //             );
            //            should_render = true;
        } else if (key.bare_key == BareKey::Esc && key.has_no_modifiers())
            || key.is_key_with_ctrl_modifier(BareKey::Char('c'))
        {
            close_self();
            should_render = true;
        }
        should_render
    }
    pub fn handle_setup_wizard_key(&mut self, key: KeyWithModifier) -> bool {
        if let Some(rebind_leaders_screen) = self.rebind_leaders_screen.as_mut() {
            match key.bare_key {
                BareKey::Esc if key.has_no_modifiers() => {
                    // consume screen without applying its modifiers
                    drop(self.rebind_leaders_screen.take());
                    return true;
                },
                BareKey::Enter if key.has_no_modifiers() => {
                    // consume screen and apply its modifiers
                    let (primary_modifier, secondary_modifier) =
                        rebind_leaders_screen.primary_and_secondary_modifiers();
                    self.primary_modifier = primary_modifier;
                    self.secondary_modifier = secondary_modifier;
                    drop(self.rebind_leaders_screen.take());
                    return true;
                },
                _ => {
                    return rebind_leaders_screen.handle_key(key);
                },
            }
        }
        let mut should_render = false;
        if self.notification.is_some() {
            self.notification = None;
            should_render = true;
        } else if key.bare_key == BareKey::Down && key.has_no_modifiers() {
            self.move_selected_index_down();
            should_render = true;
        } else if key.bare_key == BareKey::Up && key.has_no_modifiers() {
            self.move_selected_index_up();
            should_render = true;
        } else if key.bare_key == BareKey::Enter && key.has_no_modifiers() {
            if let Some(selected_index) = self.take_selected_index() {
                let write_to_disk = true;
                self.reconfigure(selected_index, write_to_disk);
                close_self();
            } else {
                self.reset_selected_index();
                should_render = true;
            }
        } else if key.bare_key == BareKey::Char('l') && key.has_no_modifiers() {
            // for the time being this screen has been disabled because it was deemed too confusing
            // and its use-cases are very limited (it's possible to achieve the same results by
            // applying a preset and then rebinding the leader keys)
            //
            // the code is left here in case someone feels strongly about implementing this on
            // their own, and because at the time of writing I'm a little ambiguous about this
            // decision. At some point it should be refactored away
            //             self.rebind_leaders_screen =
            //                 Some(RebindLeadersScreen::default().with_rebinding_for_presets());
            //            should_render = true;
        } else if (key.bare_key == BareKey::Esc && key.has_no_modifiers())
            || key.is_key_with_ctrl_modifier(BareKey::Char('c'))
        {
            close_self();
            should_render = true;
        }
        should_render
    }
    pub fn update_mode_info(&mut self, mode_info: ModeInfo) {
        if let Some(rebind_leaders_screen) = self.rebind_leaders_screen.as_mut() {
            rebind_leaders_screen.update_mode_info(mode_info.clone());
        }
        self.latest_mode_info = Some(mode_info);
    }
    pub fn move_selected_index_down(&mut self) {
        if self.selected_index.is_none() {
            self.selected_index = Some(0);
        } else if self.selected_index < Some(1) {
            self.selected_index = Some(1);
        } else {
            self.selected_index = None;
        }
    }
    pub fn move_selected_index_up(&mut self) {
        if self.selected_index.is_none() {
            self.selected_index = Some(1);
        } else if self.selected_index == Some(1) {
            self.selected_index = Some(0);
        } else {
            self.selected_index = None;
        }
    }
    pub fn take_selected_index(&mut self) -> Option<usize> {
        self.selected_index.take()
    }
    pub fn reset_selected_index(&mut self) {
        self.selected_index = Some(0);
    }
    pub fn drain_notification(&mut self) -> Option<String> {
        self.notification.take()
    }
    pub fn set_notification(&mut self, notification: Option<String>) {
        self.notification = notification;
    }
    fn reconfigure(&self, selected: usize, write_to_disk: bool) {
        if selected == 0 {
            // TODO: these should be part of a "transaction" when they are
            // implemented
            reconfigure(
                default_keybinds(
                    self.primary_modifier
                        .iter()
                        .map(|m| m.to_string())
                        .collect::<Vec<_>>()
                        .join(" "),
                    self.secondary_modifier
                        .iter()
                        .map(|m| m.to_string())
                        .collect::<Vec<_>>()
                        .join(" "),
                ),
                write_to_disk,
            );
            switch_to_input_mode(&InputMode::Normal);
        } else if selected == 1 {
            // TODO: these should be part of a "transaction" when they are
            // implemented
            reconfigure(
                unlock_first_keybinds(
                    self.primary_modifier
                        .iter()
                        .map(|m| m.to_string())
                        .collect::<Vec<_>>()
                        .join(" "),
                    self.secondary_modifier
                        .iter()
                        .map(|m| m.to_string())
                        .collect::<Vec<_>>()
                        .join(" "),
                ),
                write_to_disk,
            );
            switch_to_input_mode(&InputMode::Locked);
        }
    }
    pub fn render_setup_wizard_screen(
        &mut self,
        rows: usize,
        cols: usize,
        ui_size: usize,
        notification: &Option<String>,
    ) {
        if let Some(rebind_leaders_screen) = self.rebind_leaders_screen.as_mut() {
            return rebind_leaders_screen.render(rows, cols, ui_size, notification);
        }
        let primary_modifier_key_text = self.primary_modifier_text();
        let secondary_modifier_key_text = self.secondary_modifier_text();
        self.render_setup_wizard_title(rows, cols, &primary_modifier_key_text, ui_size);
        self.render_first_bulletin(rows + 8, cols, &primary_modifier_key_text, ui_size);
        self.render_second_bulletin(rows + 8, cols, &primary_modifier_key_text, ui_size);
        self.render_leader_keys_indication(
            rows + 8,
            cols,
            &primary_modifier_key_text,
            &secondary_modifier_key_text,
            ui_size,
        );
        info_line(
            rows + 8,
            cols,
            ui_size,
            &notification,
            &self.warning_text(cols),
            Some(self.main_screen_widths(&primary_modifier_key_text)),
        );
        // self.render_info_line(rows + 8, cols);
        self.render_help_text_setup_wizard(rows + 8, cols);
    }
    pub fn render_reset_keybindings_screen(
        &mut self,
        rows: usize,
        cols: usize,
        ui_size: usize,
        notification: &Option<String>,
    ) {
        if let Some(rebind_leaders_screen) = self.rebind_leaders_screen.as_mut() {
            return rebind_leaders_screen.render(rows, cols, ui_size, notification);
        }
        let primary_modifier_key_text = self.primary_modifier_text();
        let secondary_modifier_key_text = self.secondary_modifier_text();
        self.render_override_title(rows, cols, &primary_modifier_key_text, ui_size);
        self.render_first_bulletin(rows, cols, &primary_modifier_key_text, ui_size);
        self.render_second_bulletin(rows, cols, &primary_modifier_key_text, ui_size);
        self.render_leader_keys_indication(
            rows,
            cols,
            &primary_modifier_key_text,
            &secondary_modifier_key_text,
            ui_size,
        );
        let notification = notification.clone().or_else(|| self.notification.clone());
        let warning_text = self.warning_text(cols);
        info_line(
            rows,
            cols,
            ui_size,
            &notification,
            &warning_text,
            Some(self.main_screen_widths(&primary_modifier_key_text)),
        );
        self.render_help_text_main(rows, cols);
    }
    fn primary_modifier_text(&self) -> String {
        if self.primary_modifier.is_empty() {
            "<UNBOUND>".to_owned()
        } else {
            self.primary_modifier
                .iter()
                .map(|m| m.to_string())
                .collect::<Vec<_>>()
                .join("-")
        }
    }
    fn secondary_modifier_text(&self) -> String {
        if self.secondary_modifier.is_empty() {
            "<UNBOUND>".to_owned()
        } else {
            self.secondary_modifier
                .iter()
                .map(|m| m.to_string())
                .collect::<Vec<_>>()
                .join("-")
        }
    }
    fn render_setup_wizard_title(
        &self,
        rows: usize,
        cols: usize,
        primary_modifier_key_text: &str,
        ui_size: usize,
    ) {
        let widths = self.main_screen_widths(primary_modifier_key_text);
        if cols >= widths.0 {
            let title_text_1 = "你好！你想如何与 Zellij 交互？";
            let title_text_2 = "不确定？按 <ENTER> 选择默认。";
            let title_text_3 = "一切都可以在以后更改。";
            let title_text_4 = "提示会显示在屏幕上 - 你不需要记住任何东西。";
            let left_padding = cols.saturating_sub(widths.0) / 2;
            let first_row_coords = (rows.saturating_sub(ui_size) / 2).saturating_sub(1);
            print_text_with_coordinates(
                Text::new(title_text_1).color_range(2, ..),
                left_padding,
                first_row_coords,
                None,
                None,
            );
            print_text_with_coordinates(
                Text::new(title_text_2)
                    .color_range(0, ..4)
                    .color_range(2, 6..13)
                    .color_range(1, 16..18),
                left_padding,
                first_row_coords + 2,
                None,
                None,
            );
            print_text_with_coordinates(
                Text::new(title_text_3),
                left_padding,
                first_row_coords + 4,
                None,
                None,
            );
            print_text_with_coordinates(
                Text::new(title_text_4),
                left_padding,
                first_row_coords + 5,
                None,
                None,
            );
        } else {
            let title_text_1 = "你好！你更喜欢哪一个？";
            let title_text_2 = "不确定？按 <ENTER>";
            let title_text_3 = "以后可以更改。提示会显示";
            let title_text_4 = "在屏幕上 - 无需记住";
            let left_padding = if cols >= widths.1 {
                cols.saturating_sub(widths.1) / 2
            } else {
                cols.saturating_sub(widths.2) / 2
            };
            let first_row_coords = (rows.saturating_sub(ui_size) / 2).saturating_sub(1);
            print_text_with_coordinates(
                Text::new(title_text_1).color_range(2, ..),
                left_padding,
                first_row_coords,
                None,
                None,
            );
            print_text_with_coordinates(
                Text::new(title_text_2)
                    .color_range(0, ..4)
                    .color_range(2, 6..13),
                left_padding,
                first_row_coords + 2,
                None,
                None,
            );
            print_text_with_coordinates(
                Text::new(title_text_3),
                left_padding,
                first_row_coords + 4,
                None,
                None,
            );
            print_text_with_coordinates(
                Text::new(title_text_4),
                left_padding,
                first_row_coords + 5,
                None,
                None,
            );
        }
    }
    fn main_screen_widths(&self, primary_modifier_text: &str) -> (usize, usize, usize) {
        let primary_modifier_key_text_len = primary_modifier_text.chars().count();
        let full_width = 61 + primary_modifier_key_text_len;
        let mid_width = 36 + primary_modifier_key_text_len;
        let min_width = 26 + primary_modifier_key_text_len;
        (full_width, mid_width, min_width)
    }
    fn render_first_bulletin(
        &self,
        rows: usize,
        cols: usize,
        primary_modifier_key_text: &str,
        ui_size: usize,
    ) {
        let widths = self.main_screen_widths(primary_modifier_key_text);
        let primary_modifier_key_text_len = primary_modifier_key_text.chars().count();
        let default_text = "1. 默认";
        let (mut list_items, max_width) = if cols >= widths.0 {
            let list_items = vec![
                NestedListItem::new(default_text).color_range(1, ..),
                NestedListItem::new("所有模式都可直接从基础模式进入，例如：")
                    .indent(1),
                NestedListItem::new(format!(
                    "{} p - 进入 PANE 模式",
                    primary_modifier_key_text
                ))
                .color_range(3, ..primary_modifier_key_text_len + 3)
                .color_range(
                    2,
                    primary_modifier_key_text_len + 8..primary_modifier_key_text_len + 12,
                )
                .indent(1),
                NestedListItem::new(format!(
                    "{} t - 进入 TAB 模式",
                    primary_modifier_key_text
                ))
                .color_range(3, ..primary_modifier_key_text_len + 3)
                .color_range(
                    2,
                    primary_modifier_key_text_len + 8..primary_modifier_key_text_len + 11,
                )
                .indent(1),
            ];
            let max_width = widths.0;
            (list_items, max_width)
        } else if cols >= widths.1 {
            let list_items = vec![
                NestedListItem::new(default_text).color_range(1, ..),
                NestedListItem::new("模式可直接进入，例如：").indent(1),
                NestedListItem::new(format!(
                    "{} p - 进入 PANE 模式",
                    primary_modifier_key_text
                ))
                .indent(1)
                .color_range(3, ..primary_modifier_key_text_len + 3)
                .color_range(
                    2,
                    primary_modifier_key_text_len + 8..primary_modifier_key_text_len + 12,
                ),
                NestedListItem::new(format!(
                    "{} t - 进入 TAB 模式",
                    primary_modifier_key_text
                ))
                .indent(1)
                .color_range(3, ..primary_modifier_key_text_len + 3)
                .color_range(
                    2,
                    primary_modifier_key_text_len + 8..primary_modifier_key_text_len + 11,
                ),
            ];
            let max_width = widths.1;
            (list_items, max_width)
        } else {
            let list_items = vec![
                NestedListItem::new(default_text).color_range(1, ..),
                NestedListItem::new("直接，例如：").indent(1),
                NestedListItem::new(format!("{} p - PANE 模式", primary_modifier_key_text))
                    .color_range(3, ..primary_modifier_key_text_len + 3)
                    .color_range(
                        2,
                        primary_modifier_key_text_len + 5..primary_modifier_key_text_len + 10,
                    )
                    .indent(1),
                NestedListItem::new(format!("{} t - TAB 模式", primary_modifier_key_text))
                    .color_range(3, ..primary_modifier_key_text_len + 3)
                    .color_range(
                        2,
                        primary_modifier_key_text_len + 5..primary_modifier_key_text_len + 9,
                    )
                    .indent(1),
            ];
            let max_width = widths.2;
            (list_items, max_width)
        };
        if self.selected_index == Some(0) {
            list_items = list_items.drain(..).map(|i| i.selected()).collect();
        }
        let left_padding = cols.saturating_sub(max_width) / 2;
        let top_coordinates = if rows > 14 {
            (rows.saturating_sub(ui_size) / 2) + 3
        } else {
            (rows.saturating_sub(ui_size) / 2) + 2
        };
        print_nested_list_with_coordinates(list_items, left_padding, top_coordinates, None, None);
    }
    fn render_second_bulletin(
        &self,
        rows: usize,
        cols: usize,
        primary_modifier_key_text: &str,
        ui_size: usize,
    ) {
        let unlock_first_text = "2. 先解锁（无冲突）";
        let widths = self.main_screen_widths(primary_modifier_key_text);
        let primary_modifier_key_text_len = primary_modifier_key_text.chars().count();
        let (mut list_items, max_width) = if cols >= widths.0 {
            let list_items = vec![
                NestedListItem::new(unlock_first_text).color_range(1, ..),
                NestedListItem::new(format!(
                    "用 {} g 解锁后可使用单键模式，例如：",
                    primary_modifier_key_text
                ))
                .indent(1),
                NestedListItem::new(format!(
                    "{} g + p 进入 PANE 模式",
                    primary_modifier_key_text
                ))
                .indent(1)
                .color_range(3, ..primary_modifier_key_text_len + 3)
                .color_range(
                    3,
                    primary_modifier_key_text_len + 5..primary_modifier_key_text_len + 7,
                )
                .color_range(
                    2,
                    primary_modifier_key_text_len + 10..primary_modifier_key_text_len + 14,
                ),
                NestedListItem::new(format!(
                    "{} g + t 进入 TAB 模式",
                    primary_modifier_key_text
                ))
                .indent(1)
                .color_range(3, ..primary_modifier_key_text_len + 3)
                .color_range(
                    3,
                    primary_modifier_key_text_len + 5..primary_modifier_key_text_len + 7,
                )
                .color_range(
                    2,
                    primary_modifier_key_text_len + 10..primary_modifier_key_text_len + 13,
                ),
            ];
            let max_width = widths.0;
            (list_items, max_width)
        } else if cols >= widths.1 {
            let list_items = vec![
                NestedListItem::new(unlock_first_text).color_range(1, ..),
                NestedListItem::new(format!(
                    "{} g 后单键模式，例如：",
                    primary_modifier_key_text
                ))
                .indent(1),
                NestedListItem::new(format!(
                    "{} g + p 进入 PANE 模式",
                    primary_modifier_key_text
                ))
                .color_range(3, ..primary_modifier_key_text_len + 3)
                .color_range(
                    3,
                    primary_modifier_key_text_len + 5..primary_modifier_key_text_len + 7,
                )
                .color_range(
                    2,
                    primary_modifier_key_text_len + 10..primary_modifier_key_text_len + 14,
                )
                .indent(1),
                NestedListItem::new(format!(
                    "{} g + t 进入 TAB 模式",
                    primary_modifier_key_text
                ))
                .color_range(3, ..primary_modifier_key_text_len + 3)
                .color_range(
                    3,
                    primary_modifier_key_text_len + 5..primary_modifier_key_text_len + 7,
                )
                .color_range(
                    2,
                    primary_modifier_key_text_len + 10..primary_modifier_key_text_len + 13,
                )
                .indent(1),
            ];
            let max_width = widths.1;
            (list_items, max_width)
        } else {
            let list_items = vec![
                NestedListItem::new("2. 先解锁").color_range(1, ..),
                NestedListItem::new(format!(
                    "{} g + 单键，例如：",
                    primary_modifier_key_text
                ))
                .indent(1),
                NestedListItem::new(format!("{} g + p PANE 模式", primary_modifier_key_text))
                    .color_range(3, ..primary_modifier_key_text_len + 3)
                    .color_range(
                        3,
                        primary_modifier_key_text_len + 5..primary_modifier_key_text_len + 7,
                    )
                    .color_range(
                        2,
                        primary_modifier_key_text_len + 7..primary_modifier_key_text_len + 11,
                    )
                    .indent(1),
                NestedListItem::new(format!("{} g + t TAB 模式", primary_modifier_key_text))
                    .color_range(3, ..primary_modifier_key_text_len + 3)
                    .color_range(
                        3,
                        primary_modifier_key_text_len + 5..primary_modifier_key_text_len + 7,
                    )
                    .color_range(
                        2,
                        primary_modifier_key_text_len + 7..primary_modifier_key_text_len + 10,
                    )
                    .indent(1),
            ];
            let max_width = widths.2;
            (list_items, max_width)
        };
        if self.selected_index == Some(1) {
            list_items = list_items.drain(..).map(|i| i.selected()).collect();
        }
        let left_padding = cols.saturating_sub(max_width) / 2;
        let top_coordinates = if rows > 14 {
            (rows.saturating_sub(ui_size) / 2) + 8
        } else {
            (rows.saturating_sub(ui_size) / 2) + 6
        };
        print_nested_list_with_coordinates(list_items, left_padding, top_coordinates, None, None);
    }
    fn render_leader_keys_indication(
        &self,
        rows: usize,
        cols: usize,
        primary_modifier_key_text: &str,
        secondary_modifier_key_text: &str,
        ui_size: usize,
    ) {
        let widths = self.main_screen_widths(primary_modifier_key_text);
        let primary_modifier_key_text_len = primary_modifier_key_text.chars().count();
        let secondary_modifier_key_text_len = secondary_modifier_key_text.chars().count();
        let top_coordinates = if rows > 14 {
            (rows.saturating_sub(ui_size) / 2) + 13
        } else {
            (rows.saturating_sub(ui_size) / 2) + 10
        };

        if cols >= widths.0 {
            let leader_key_text = format!(
                "前导键: {} - 模式, {} - 快速导航和快捷键",
                primary_modifier_key_text, secondary_modifier_key_text
            );
            let left_padding = cols.saturating_sub(widths.0) / 2;
            print_text_with_coordinates(
                Text::new(leader_key_text)
                    .color_range(2, ..3)
                    .color_range(3, 5..primary_modifier_key_text_len + 6)
                    .color_range(
                        0,
                        primary_modifier_key_text_len + 12
                            ..primary_modifier_key_text_len + 12 + secondary_modifier_key_text_len,
                    ),
                left_padding,
                top_coordinates,
                None,
                None,
            )
        } else {
            let leader_key_text = format!(
                "前导键: {}, {}",
                primary_modifier_key_text, secondary_modifier_key_text
            );
            let left_padding = if cols >= widths.1 {
                cols.saturating_sub(widths.1) / 2
            } else {
                cols.saturating_sub(widths.2) / 2
            };
            print_text_with_coordinates(
                Text::new(leader_key_text)
                    .color_range(2, ..3)
                    .color_range(3, 5..primary_modifier_key_text_len + 6)
                    .color_range(
                        0,
                        primary_modifier_key_text_len + 7
                            ..primary_modifier_key_text_len + 8 + secondary_modifier_key_text_len,
                    ),
                left_padding,
                top_coordinates,
                None,
                None,
            )
        };
    }
    fn render_help_text_setup_wizard(&self, rows: usize, cols: usize) {
        let full_help_text = "帮助: <↓↑> - 导航, <ENTER> - 应用并保存, <ESC> - 关闭";
        let short_help_text = "帮助: <↓↑> / <ENTER> / <ESC>";
        if cols >= full_help_text.chars().count() {
            print_text_with_coordinates(
                Text::new(full_help_text)
                    .color_range(2, 4..8)
                    .color_range(2, 15..22)
                    .color_range(2, 32..=35),
                0,
                rows,
                None,
                None,
            );
        } else {
            print_text_with_coordinates(
                Text::new(short_help_text)
                    .color_range(2, 4..8)
                    .color_range(2, 11..18)
                    .color_range(2, 21..=24),
                0,
                rows,
                None,
                None,
            );
        }
    }
    fn render_override_title(
        &self,
        rows: usize,
        cols: usize,
        primary_modifier_key_text: &str,
        ui_size: usize,
    ) {
        let widths = self.main_screen_widths(primary_modifier_key_text);
        if cols >= widths.0 {
            let title_text = "使用以下预设之一覆盖按键绑定：";
            let left_padding = cols.saturating_sub(widths.0) / 2;
            print_text_with_coordinates(
                Text::new(title_text).color_range(2, ..),
                left_padding,
                (rows.saturating_sub(ui_size) / 2) + 1,
                None,
                None,
            );
        } else {
            let title_text = "覆盖按键绑定：";
            let left_padding = if cols >= widths.1 {
                cols.saturating_sub(widths.1) / 2
            } else {
                cols.saturating_sub(widths.2) / 2
            };
            print_text_with_coordinates(
                Text::new(title_text).color_range(2, ..),
                left_padding,
                (rows.saturating_sub(ui_size) / 2) + 1,
                None,
                None,
            );
        }
    }
    fn render_help_text_main(&self, rows: usize, cols: usize) {
        let full_help_text =
            "帮助: <↓↑> - 导航, <ENTER> - 应用, <Ctrl a> - 应用并保存, <ESC> - 关闭";
        let short_help_text = "帮助: <↓↑> / <ENTER> / <Ctrl a> / <ESC>";
        if cols >= full_help_text.chars().count() {
            print_text_with_coordinates(
                Text::new(full_help_text)
                    .color_range(2, 4..8)
                    .color_range(2, 15..22)
                    .color_range(2, 29..37)
                    .color_range(2, 47..=51),
                0,
                rows,
                None,
                None,
            );
        } else {
            print_text_with_coordinates(
                Text::new(short_help_text)
                    .color_range(2, 4..8)
                    .color_range(2, 11..18)
                    .color_range(2, 21..29)
                    .color_range(2, 32..=36),
                0,
                rows,
                None,
                None,
            );
        }
    }
    fn warning_text(&self, max_width: usize) -> Option<String> {
        if self.needs_kitty_support() {
            if max_width >= 38 {
                Some(String::from("警告: 需要支持的终端。"))
            } else {
                Some(String::from("需要支持的终端"))
            }
        } else if self.primary_modifier.is_empty() && self.secondary_modifier.is_empty() {
            if max_width >= 49 {
                Some(String::from(
                    "警告: 未定义前导键。UI 将被禁用。",
                ))
            } else {
                Some(String::from("无前导键。UI 将无法使用。"))
            }
        } else {
            None
        }
    }
    fn needs_kitty_support(&self) -> bool {
        self.primary_modifier.len() > 1
            || self.secondary_modifier.len() > 1
            || self.primary_modifier.contains(&KeyModifier::Super)
            || self.secondary_modifier.contains(&KeyModifier::Super)
    }
}
