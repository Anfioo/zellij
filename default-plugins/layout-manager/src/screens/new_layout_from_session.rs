use super::{ErrorScreen, KeyResponse, LayoutListScreen, OptimisticUpdate, Screen};
use crate::text_input::{InputAction, TextInput};
use crate::ui::{truncate_with_ellipsis_start, LayoutDetail};
use crate::DisplayLayout;
use zellij_tile::prelude::*;

#[derive(Clone)]
pub struct NewLayoutFromCurrentSessionScreen {
    pub name_input: TextInput,
    pub session_layout: String,
    pub current_layout_metadata: LayoutMetadata,
    pub editing_name: bool,
    pub save_current_tab_only: bool,
}

impl NewLayoutFromCurrentSessionScreen {
    pub fn handle_key(&mut self, key: KeyWithModifier) -> KeyResponse {
        // Special Esc handling based on edit state
        if key.bare_key == BareKey::Esc && key.has_no_modifiers() {
            return self.handle_escape_key();
        }

        if self.editing_name {
            self.handle_editing_mode_key(key)
        } else {
            self.handle_non_editing_mode_key(key)
        }
    }

    fn handle_escape_key(&mut self) -> KeyResponse {
        if self.editing_name {
            self.editing_name = false;
            KeyResponse::render()
        } else {
            KeyResponse::new_screen(self.cancel_save())
        }
    }

    fn handle_editing_mode_key(&mut self, key: KeyWithModifier) -> KeyResponse {
        let action = self.name_input.handle_key(key);

        match action {
            InputAction::Continue => KeyResponse::render(),
            InputAction::Submit => self.exit_editing_mode(),
            InputAction::Cancel => self.exit_editing_mode(),
            InputAction::Complete => KeyResponse::none(),
            InputAction::NoAction => KeyResponse::none(),
        }
    }

    fn handle_non_editing_mode_key(&mut self, key: KeyWithModifier) -> KeyResponse {
        match key.bare_key {
            BareKey::Enter if key.has_no_modifiers() => self.attempt_save_current_session(),
            BareKey::Tab if key.has_no_modifiers() => self.handle_tab_toggle(),
            BareKey::Char('u') if key.has_no_modifiers() => self.handle_update_layout(),
            BareKey::Char('r') if key.has_no_modifiers() => self.enter_editing_mode(),
            _ => KeyResponse::none(),
        }
    }

    fn exit_editing_mode(&mut self) -> KeyResponse {
        self.editing_name = false;
        KeyResponse::render()
    }

    fn enter_editing_mode(&mut self) -> KeyResponse {
        self.editing_name = true;
        KeyResponse::render()
    }

    fn handle_tab_toggle(&mut self) -> KeyResponse {
        self.toggle_tab_target();
        if let Some(new_screen) = self.update_layout_from_session() {
            KeyResponse::new_screen(new_screen)
        } else {
            KeyResponse::render()
        }
    }

    fn handle_update_layout(&mut self) -> KeyResponse {
        if let Some(new_screen) = self.update_layout_from_session() {
            KeyResponse::new_screen(new_screen)
        } else {
            KeyResponse::render()
        }
    }

    fn update_layout_from_session(&mut self) -> Option<Screen> {
        let Ok(focused_pane_info) = get_focused_pane_info() else {
            eprintln!("Cannot retrieve focused tab info");
            return None;
        };
        let focused_tab_index = focused_pane_info.0;
        if self.save_current_tab_only {
            match dump_session_layout_for_tab(focused_tab_index) {
                Ok((session_layout, Some(session_layout_metadata))) => {
                    self.session_layout = session_layout;
                    self.current_layout_metadata = session_layout_metadata;
                    None
                },
                Ok((_, None)) => Some(Screen::Error(super::ErrorScreen {
                    message: "无法获取会话布局元数据".to_string(),
                    return_to_screen: Box::new(Screen::LayoutList(Default::default())),
                })),
                Err(error_msg) => Some(Screen::Error(super::ErrorScreen {
                    message: format!("无法导出会话布局：{}", error_msg),
                    return_to_screen: Box::new(Default::default()),
                })),
            }
        } else {
            match dump_session_layout() {
                Ok((session_layout, Some(session_layout_metadata))) => {
                    self.session_layout = session_layout;
                    self.current_layout_metadata = session_layout_metadata;
                    None
                },
                Ok((_, None)) => Some(Screen::Error(super::ErrorScreen {
                    message: "无法获取会话布局元数据".to_string(),
                    return_to_screen: Box::new(Screen::LayoutList(Default::default())),
                })),
                Err(error_msg) => Some(Screen::Error(super::ErrorScreen {
                    message: format!("无法导出会话布局：{}", error_msg),
                    return_to_screen: Box::new(Default::default()),
                })),
            }
        }
    }

    fn cancel_save(&self) -> Screen {
        Screen::LayoutList(LayoutListScreen::default())
    }

    fn toggle_tab_target(&mut self) {
        self.save_current_tab_only = !self.save_current_tab_only;
    }

    fn attempt_save_current_session(&self) -> KeyResponse {
        let layout_name = self.determine_layout_name();

        // Capture optimistic update BEFORE the API call
        let optimistic = OptimisticUpdate::Add {
            name: layout_name.clone(),
            metadata: self.current_layout_metadata.clone(),
        };

        match self.save_session_layout(&layout_name) {
            Ok(()) => KeyResponse::new_screen(Screen::LayoutList(LayoutListScreen::default()))
                .with_optimistic(optimistic),
            Err(error_msg) => KeyResponse::new_screen(self.create_error_screen(&error_msg)),
        }
    }

    fn create_error_screen(&self, message: &str) -> Screen {
        Screen::Error(ErrorScreen {
            message: message.to_string(),
            return_to_screen: Box::new(Screen::NewLayoutFromSession(self.clone())),
        })
    }

    fn determine_layout_name(&self) -> String {
        let input_text = self.name_input.get_text();
        if input_text.is_empty() {
            generate_random_name()
        } else {
            input_text.to_string()
        }
    }

    fn save_session_layout(&self, layout_name: &str) -> Result<(), String> {
        save_layout(layout_name.to_owned(), self.session_layout.clone(), false)
            .map_err(|err| format!("无法保存布局 '{}'：{}", layout_name, err))?;

        eprintln!(
            "已成功将当前会话保存为布局：{}",
            layout_name
        );
        Ok(())
    }

    pub fn render(&self, rows: usize, cols: usize) {
        let display_layout = DisplayLayout::Valid(LayoutInfo::File(
            "current".to_string(),
            self.current_layout_metadata.clone(),
        ));

        let layout_detail = LayoutDetail::new(&display_layout);

        let desired_ui_width = std::cmp::max(
            self.help_text_full().0.chars().count(),
            self.max_description_width(),
        );

        let actual_ui_width = std::cmp::min(desired_ui_width, cols);

        let available_cols_for_details = actual_ui_width.saturating_sub(2); // Account for indentation
        let desired_details_height =
            layout_detail.calculate_required_height(available_cols_for_details);

        let desired_ui_height = 12 + desired_details_height;
        let base_y = rows.saturating_sub(desired_ui_height) / 2;
        let base_x = cols.saturating_sub(actual_ui_width) / 2;

        let save_as_line_y = base_y + 5;
        self.update_cursor_position(base_x, save_as_line_y, actual_ui_width); // NOTE: must be before any render
                                                                              // happens

        self.render_title(base_x, base_y, actual_ui_width);
        self.render_description(base_x, base_y + 2, actual_ui_width);
        self.render_save_as_line(base_x, save_as_line_y, actual_ui_width);
        self.render_tab_toggle(base_x, base_y + 7, actual_ui_width);

        let details_y = base_y + 9;

        let actual_details_height = if rows < desired_ui_height {
            rows.saturating_sub(details_y + 2)
        } else {
            desired_details_height
        };

        layout_detail.render(
            base_x + 2,
            details_y,
            actual_details_height,
            available_cols_for_details,
        );

        let help_y = details_y + actual_details_height + 1;
        self.render_help_text(base_x, help_y, actual_ui_width);
    }

    fn render_title(&self, x: usize, y: usize, width: usize) {
        let title = Text::new("保存当前会话的布局").color_all(2);
        print_text_with_coordinates(title, x, y, Some(width), None);
    }

    fn description_text_full(&self) -> (&str, &str) {
        (
            "此布局基于当前会话创建。",
            "保存后可稍后重建会话或与他人分享。",
        )
    }
    fn description_text_short(&self) -> (&str, &str) {
        (
            "来自当前会话的布局。",
            "保存后可重建或稍后分享。",
        )
    }
    fn render_description(&self, x: usize, y: usize, width: usize) {
        let (line1, line2) = if width >= self.max_description_width() {
            self.description_text_full()
        } else {
            self.description_text_short()
        };
        print_text_with_coordinates(Text::new(line1), x, y, None, None);
        print_text_with_coordinates(Text::new(line2), x, y + 1, None, None);
    }
    fn max_description_width(&self) -> usize {
        let (line1, line2) = self.description_text_full();
        std::cmp::max(line1.chars().count(), line2.chars().count())
    }

    fn save_as_line_text(&self, width: usize) -> (String, usize) {
        // (text, cursor_position_in_line)
        let input_text = self.name_input.get_text();
        let cursor_pos = self.name_input.get_cursor_position();

        let display_name = if input_text.is_empty() && !self.editing_name {
            "RANDOM"
        } else {
            input_text
        };
        let mut text = format!("保存为：{}（<r> 重命名）", display_name);
        let mut cursor_position_in_line = 5 + cursor_pos;
        if text.chars().count() > width {
            let truncated_display_name = truncate_with_ellipsis_start(
                display_name,
                width.saturating_sub(15), // 文本中除显示名称外的部分大小
            );
            text = format!("保存为：{}（<r> 重命名）", truncated_display_name);
            let truncated_len = truncated_display_name.chars().count();
            cursor_position_in_line = 5 + cursor_pos.min(truncated_len);
        }
        (text, cursor_position_in_line)
    }
    fn render_save_as_line(&self, x: usize, y: usize, width: usize) {
        let (text, _) = self.save_as_line_text(width);
        let colored = Text::new(&text).color_substring(3, "<r>");

        print_text_with_coordinates(colored, x, y, None, None);
    }

    fn render_tab_toggle(&self, x: usize, y: usize, width: usize) {
        let text = if self.save_current_tab_only {
            "<Tab>  所有标签  | [仅当前标签]"
        } else {
            "<Tab> [所有标签] |  仅当前标签"
        };

        let short_text = if self.save_current_tab_only {
            "<Tab>  全部  | [当前标签]"
        } else {
            "<Tab> [全部] |  当前标签"
        };
        let text = if text.chars().count() > width {
            short_text
        } else {
            text
        };
        let colored = Text::new(text)
            .color_substring(3, "<Tab>")
            .color_substring(0, "[所有标签]")
            .color_substring(0, "[全部]")
            .color_substring(0, "[仅当前标签]")
            .color_substring(0, "[当前标签]");

        print_text_with_coordinates(colored, x, y, None, None);
    }

    fn help_text_full(&self) -> (&str, &[&str]) {
        (
            "<Enter> - 保存，<u> - 更新布局，<Esc> - 返回",
            &["<Enter>", "<u>", "<Esc>"],
        )
    }
    fn help_text_short(&self) -> (&str, &[&str]) {
        (
            "<Enter> - 保存，<u> - 更新，<Esc> - 返回",
            &["<Enter>", "<u>", "<Esc>"],
        )
    }
    fn render_help_text(&self, x: usize, y: usize, width: usize) {
        let (text, items_to_color) = if width < self.help_text_full().0.chars().count() {
            self.help_text_short()
        } else {
            self.help_text_full()
        };
        let mut text = Text::new(text);
        for item in items_to_color {
            text = text.color_substring(3, item)
        }
        print_text_with_coordinates(text, x, y, None, None);
    }

    pub fn update_cursor_position(&self, base_x: usize, cursor_y: usize, max_width: usize) {
        if self.editing_name {
            let (_, cursor_position_in_line) = self.save_as_line_text(max_width);
            show_cursor(Some((base_x + cursor_position_in_line, cursor_y)))
        } else {
            show_cursor(None)
        }
    }
}
