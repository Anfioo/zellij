use zellij_tile::prelude::*;

#[derive(Debug)]
struct ScreenContent {
    title: (String, Text),
    items: Vec<Vec<Text>>,
    help: (String, Text),
    status_message: Option<(String, Text)>,
    max_width: usize,
    new_token_line: Option<(String, Text)>,
}

#[derive(Debug)]
struct Layout {
    base_x: usize,
    base_y: usize,
    title_x: usize,
    new_token_y: usize,
    help_y: usize,
    status_y: usize,
}

#[derive(Debug)]
struct ScrollInfo {
    start_index: usize,
    end_index: usize,
    truncated_top: usize,
    truncated_bottom: usize,
}

#[derive(Debug)]
struct ColumnWidths {
    token: usize,
    date: usize,
    read_only: usize,
    controls: usize,
}

pub struct TokenManagementScreen<'a> {
    token_list: &'a Vec<(String, String, bool)>, // bool -> is_read_only
    selected_list_index: Option<usize>,
    renaming_token: &'a Option<String>,
    entering_new_token_name: &'a Option<String>,
    error: &'a Option<String>,
    info: &'a Option<String>,
    rows: usize,
    cols: usize,
}

impl<'a> TokenManagementScreen<'a> {
    pub fn new(
        token_list: &'a Vec<(String, String, bool)>,
        selected_list_index: Option<usize>,
        renaming_token: &'a Option<String>,
        entering_new_token_name: &'a Option<String>,
        error: &'a Option<String>,
        info: &'a Option<String>,
        rows: usize,
        cols: usize,
    ) -> Self {
        Self {
            token_list,
            selected_list_index,
            renaming_token,
            entering_new_token_name,
            error,
            info,
            rows,
            cols,
        }
    }
    pub fn render(&self) {
        let content = self.build_screen_content();
        let max_height = self.calculate_max_item_height();
        let scrolled_content = self.apply_scroll_truncation(content, max_height);
        let layout = self.calculate_layout(&scrolled_content);
        self.print_items_to_screen(scrolled_content, layout);
    }

    fn calculate_column_widths(&self) -> ColumnWidths {
        let max_table_width = self.cols;

        const MIN_TOKEN_WIDTH: usize = 10;
        const MIN_DATE_WIDTH: usize = 10; // 仅日期 "YYYY-MM-DD" 的最小值
        const MIN_READ_ONLY_WIDTH: usize = 5; // "RO/RW" 的最小值
        const MIN_CONTROLS_WIDTH: usize = 6; // "(<x>, <r>)" 的最小值
        const COLUMN_SPACING: usize = 5; // 列之间的间距（4 列含表格内边距）

        let min_total_width = MIN_TOKEN_WIDTH
            + MIN_DATE_WIDTH
            + MIN_READ_ONLY_WIDTH
            + MIN_CONTROLS_WIDTH
            + COLUMN_SPACING;

        if max_table_width <= min_total_width {
            return ColumnWidths {
                token: MIN_TOKEN_WIDTH,
                date: MIN_DATE_WIDTH,
                read_only: MIN_READ_ONLY_WIDTH,
                controls: MIN_CONTROLS_WIDTH,
            };
        }

        const PREFERRED_DATE_WIDTH: usize = 29; // "issued on YYYY-MM-DD HH:MM:SS"
        const PREFERRED_READ_ONLY_WIDTH: usize = 10; // "read-write"
        const PREFERRED_CONTROLS_WIDTH: usize = 24; // "(<x> revoke, <r> rename)"

        let available_width = max_table_width.saturating_sub(COLUMN_SPACING);
        let preferred_fixed_width =
            PREFERRED_DATE_WIDTH + PREFERRED_READ_ONLY_WIDTH + PREFERRED_CONTROLS_WIDTH;

        if available_width >= preferred_fixed_width + MIN_TOKEN_WIDTH {
            // 我们可以对日期、只读和控制列使用首选宽度
            ColumnWidths {
                token: available_width.saturating_sub(preferred_fixed_width),
                date: PREFERRED_DATE_WIDTH,
                read_only: PREFERRED_READ_ONLY_WIDTH,
                controls: PREFERRED_CONTROLS_WIDTH,
            }
        } else {
            // 需要在所有列之间平衡截断
            // 优先级：controls > read_only > date > token（token 获得剩余空间）
            let remaining_width = available_width
                .saturating_sub(MIN_TOKEN_WIDTH)
                .saturating_sub(MIN_DATE_WIDTH)
                .saturating_sub(MIN_READ_ONLY_WIDTH)
                .saturating_sub(MIN_CONTROLS_WIDTH);
            let extra_per_column = remaining_width / 4;

            ColumnWidths {
                token: MIN_TOKEN_WIDTH + extra_per_column,
                date: MIN_DATE_WIDTH + extra_per_column,
                read_only: MIN_READ_ONLY_WIDTH + extra_per_column,
                controls: MIN_CONTROLS_WIDTH + extra_per_column,
            }
        }
    }

    fn truncate_token_name(&self, token: &str, max_width: usize) -> String {
        if token.chars().count() <= max_width {
            return token.to_string();
        }

        if max_width <= 6 {
            // 太小，无法显示有意义的内容
            return "[...]".to_string();
        }

        let truncator = if max_width <= 10 { "[..]" } else { "[...]" };
        let truncator_len = truncator.chars().count();
        let remaining_chars = max_width.saturating_sub(truncator_len);
        let start_chars = remaining_chars / 2;
        let end_chars = remaining_chars.saturating_sub(start_chars);

        let token_chars: Vec<char> = token.chars().collect();
        let start_part: String = token_chars.iter().take(start_chars).collect();
        let end_part: String = token_chars
            .iter()
            .rev()
            .take(end_chars)
            .collect::<String>()
            .chars()
            .rev()
            .collect();

        format!("{}{}{}", start_part, truncator, end_part)
    }

    fn format_date(
        &self,
        created_at: &str,
        max_width: usize,
        include_issued_prefix: bool,
    ) -> String {
        let full_text = if include_issued_prefix {
            format!("issued on {}", created_at)
        } else {
            created_at.to_string()
        };

        if full_text.chars().count() <= max_width {
            return full_text;
        }

        // 如果放不下 "issued on"，则使用日期
        if !include_issued_prefix || created_at.chars().count() <= max_width {
            if created_at.chars().count() <= max_width {
                return created_at.to_string();
            }

            // 如有必要，截断日期本身
            let chars: Vec<char> = created_at.chars().collect();
            if max_width <= 3 {
                return "...".to_string();
            }
            let truncated: String = chars.iter().take(max_width - 3).collect();
            format!("{}...", truncated)
        } else {
            // 尝试不带 "issued on" 前缀
            self.format_date(created_at, max_width, false)
        }
    }

    fn format_read_only(&self, is_read_only: bool, max_width: usize) -> String {
        let full_text = if is_read_only {
            "read-only"
        } else {
            "read-write"
        };
        let short_text = if is_read_only { "RO" } else { "RW" };

        let text = if full_text.chars().count() <= max_width {
            full_text
        } else {
            short_text
        };

        // 在列中居中文本
        let text_len = text.chars().count();
        if text_len >= max_width {
            return text.to_string();
        }

        let padding = max_width - text_len;
        let left_padding = padding / 2;
        let right_padding = padding - left_padding;

        format!(
            "{}{}{}",
            " ".repeat(left_padding),
            text,
            " ".repeat(right_padding)
        )
    }

    fn format_controls(&self, max_width: usize, is_selected: bool) -> String {
        if !is_selected {
            return " ".repeat(max_width);
        }

        let full_controls = "(<x> revoke, <r> rename)";
        let short_controls = "(<x>, <r>)";

        if full_controls.chars().count() <= max_width {
            full_controls.to_string()
        } else if short_controls.chars().count() <= max_width {
            // 填充短控制文本以填满可用宽度
            let padding = max_width - short_controls.chars().count();
            format!("{}{}", short_controls, " ".repeat(padding))
        } else {
            // 空间非常受限
            " ".repeat(max_width)
        }
    }

    fn calculate_max_item_height(&self) -> usize {
        // 计算始终存在的固定界面元素：
        // - 1 行标题
        // - 1 行标题后间距（始终保留）
        // - 1 行 "创建新令牌" 行（始终可见）
        // - 1 行帮助前间距（始终保留）
        // - 1 行帮助文本（或状态消息 —— 它们互斥）

        let fixed_rows = 4; // 标题 + 间距 + 帮助/状态 + 帮助前间距
        let create_new_token_rows = 1; // "创建新令牌" 行

        let total_fixed_rows = fixed_rows + create_new_token_rows;

        // 计算令牌条目可用的行数
        let available_for_items = self.rows.saturating_sub(total_fixed_rows);

        // 至少返回 1 以避免问题，但这仅是令牌条目的最大高度
        available_for_items.max(1)
    }

    fn build_screen_content(&self) -> ScreenContent {
        let mut max_width = 0;
        let max_table_width = self.cols;
        let column_widths = self.calculate_column_widths();

        let title_text = "List of Login Tokens";
        let title = Text::new(title_text).color_range(2, ..);
        max_width = std::cmp::max(max_width, title_text.len());

        let mut items = vec![];
        for (i, (token, created_at, read_only)) in self.token_list.iter().enumerate() {
            let is_selected = Some(i) == self.selected_list_index;
            let (row_text, row_items) =
                self.create_token_item(token, created_at, *read_only, is_selected, &column_widths);
            max_width = std::cmp::max(max_width, row_text.chars().count());
            items.push(row_items);
        }

        let (new_token_text, new_token_line) = self.create_new_token_line();
        max_width = std::cmp::max(max_width, new_token_text.chars().count());

        let (help_text, help_line) = self.create_help_line();
        max_width = std::cmp::max(max_width, help_text.chars().count());

        let status_message = self.create_status_message();
        if let Some((ref text, _)) = status_message {
            max_width = std::cmp::max(max_width, text.chars().count());
        }

        max_width = std::cmp::min(max_width, max_table_width);

        ScreenContent {
            title: (title_text.to_string(), title),
            items,
            help: (help_text, help_line),
            status_message,
            max_width,
            new_token_line: Some((new_token_text, new_token_line)),
        }
    }

    fn apply_scroll_truncation(
        &self,
        mut content: ScreenContent,
        max_height: usize,
    ) -> ScreenContent {
        let total_token_items = content.items.len(); // 仅令牌条目，不包括 "创建新令牌"

        // 如果所有令牌条目都能放下，则无需截断
        if total_token_items <= max_height {
            return content;
        }

        let scroll_info = self.calculate_scroll_info(total_token_items, max_height);

        // 提取可见范围
        let mut visible_items: Vec<Vec<Text>> = content
            .items
            .into_iter()
            .skip(scroll_info.start_index)
            .take(
                scroll_info
                    .end_index
                    .saturating_sub(scroll_info.start_index),
            )
            .collect();

        // 添加截断指示器
        if scroll_info.truncated_top > 0 {
            self.add_truncation_indicator(&mut visible_items[0], scroll_info.truncated_top);
        }

        if scroll_info.truncated_bottom > 0 {
            let last_idx = visible_items.len().saturating_sub(1);
            self.add_truncation_indicator(
                &mut visible_items[last_idx],
                scroll_info.truncated_bottom,
            );
        }

        content.items = visible_items;
        content
    }

    fn calculate_scroll_info(&self, total_token_items: usize, max_height: usize) -> ScrollInfo {
        // 滚动时仅考虑令牌条目（不包括 "创建新令牌" 行）
        // "创建新令牌" 行始终可见并单独处理

        // 仅在令牌列表中查找选中的索引
        let selected_index = if let Some(idx) = self.selected_list_index {
            idx
        } else {
            // 如果选中了 "创建新令牌" 或没有选中项，
            // 我们不需要在令牌列表中居中任何内容
            0
        };

        // 计算选中条目上方和下方显示的条目数
        let items_above = max_height / 2;
        let items_below = max_height.saturating_sub(items_above).saturating_sub(1); // -1 表示选中条目本身

        // 计算起始和结束索引
        let start_index = if selected_index < items_above {
            0
        } else if selected_index + items_below >= total_token_items {
            total_token_items.saturating_sub(max_height)
        } else {
            selected_index.saturating_sub(items_above)
        };

        let end_index = std::cmp::min(start_index + max_height, total_token_items);

        ScrollInfo {
            start_index,
            end_index,
            truncated_top: start_index,
            truncated_bottom: total_token_items.saturating_sub(end_index),
        }
    }

    fn add_truncation_indicator(&self, row: &mut Vec<Text>, count: usize) {
        let indicator = format!("+[{}]", count);

        // 用截断指示器替换最后一个单元格（控制列）
        if let Some(last_cell) = row.last_mut() {
            *last_cell = Text::new(&indicator).color_range(1, ..);
        }
    }

    fn create_token_item(
        &self,
        token: &str,
        created_at: &str,
        is_read_only: bool,
        is_selected: bool,
        column_widths: &ColumnWidths,
    ) -> (String, Vec<Text>) {
        if is_selected {
            if let Some(new_name) = &self.renaming_token {
                self.create_renaming_item(new_name, created_at, is_read_only, column_widths)
            } else {
                self.create_selected_item(token, created_at, is_read_only, column_widths)
            }
        } else {
            self.create_regular_item(token, created_at, is_read_only, column_widths)
        }
    }

    fn create_renaming_item(
        &self,
        new_name: &str,
        created_at: &str,
        is_read_only: bool,
        column_widths: &ColumnWidths,
    ) -> (String, Vec<Text>) {
        let truncated_name =
            self.truncate_token_name(new_name, column_widths.token.saturating_sub(1)); // -1 用于光标
        let item_text = format!("{}_", truncated_name);
        let date_text = self.format_date(created_at, column_widths.date, true);
        let read_only_text = self.format_read_only(is_read_only, column_widths.read_only);
        let controls_text = " ".repeat(column_widths.controls);

        let token_end = truncated_name.chars().count();
        let items = vec![
            Text::new(&item_text)
                .color_range(0, ..token_end + 1)
                .selected(),
            Text::new(&date_text),
            Text::new(&read_only_text).color_all(1),
            Text::new(&controls_text),
        ];
        (
            format!(
                "{} {} {} {}",
                item_text, date_text, read_only_text, controls_text
            ),
            items,
        )
    }

    fn create_selected_item(
        &self,
        token: &str,
        created_at: &str,
        is_read_only: bool,
        column_widths: &ColumnWidths,
    ) -> (String, Vec<Text>) {
        let mut item_text = self.truncate_token_name(token, column_widths.token);
        if item_text.is_empty() {
            // 否则表格会乱掉
            item_text.push(' ');
        };
        let date_text = self.format_date(created_at, column_widths.date, true);
        let read_only_text = self.format_read_only(is_read_only, column_widths.read_only);
        let controls_text = self.format_controls(column_widths.controls, true);

        // 根据实际内容确定控制列的高亮范围
        let (x_range, r_range) = if controls_text.contains("revoke") {
            // 完整控制："(<x> revoke, <r> rename)"
            (1..=3, 13..=15)
        } else {
            // 简短控制："(<x>, <r>)"
            (1..=3, 6..=8)
        };

        let controls_colored = if controls_text.trim().is_empty() {
            Text::new(&controls_text).selected()
        } else {
            Text::new(&controls_text)
                .color_range(3, x_range)
                .color_range(3, r_range)
                .selected()
        };

        let items = vec![
            Text::new(&item_text).color_range(0, ..).selected(),
            Text::new(&date_text).selected(),
            Text::new(&read_only_text).color_all(1).selected(),
            controls_colored,
        ];

        (
            format!(
                "{} {} {} {}",
                item_text, date_text, read_only_text, controls_text
            ),
            items,
        )
    }

    fn create_regular_item(
        &self,
        token: &str,
        created_at: &str,
        is_read_only: bool,
        column_widths: &ColumnWidths,
    ) -> (String, Vec<Text>) {
        let mut item_text = self.truncate_token_name(token, column_widths.token);
        if item_text.is_empty() {
            // otherwise the table gets messed up
            item_text.push(' ');
        };
        let date_text = self.format_date(created_at, column_widths.date, true);
        let read_only_text = self.format_read_only(is_read_only, column_widths.read_only);
        let controls_text = " ".repeat(column_widths.controls);

        let items = vec![
            Text::new(&item_text).color_range(0, ..),
            Text::new(&date_text),
            Text::new(&read_only_text).color_all(1),
            Text::new(&controls_text),
        ];
        (
            format!(
                "{} {} {} {}",
                item_text, date_text, read_only_text, controls_text
            ),
            items,
        )
    }

    fn create_new_token_line(&self) -> (String, Text) {
        let full_create_text = "<n> - create new token, <o> - create read-only token".to_string();
        let medium_create_text = "<n> - new token, <o> - read-only".to_string();
        let short_create_text = "<n> - new, <o> - RO".to_string();

        if let Some(name) = &self.entering_new_token_name {
            let max_width = self.cols.saturating_sub(1); // 为光标留出空间
            let truncated_name = if name.chars().count() > max_width {
                let chars: Vec<char> = name.chars().take(max_width).collect();
                chars.into_iter().collect()
            } else {
                name.clone()
            };
            let text = format!("{}_", truncated_name);
            (text.clone(), Text::new(&text).color_range(3, ..))
        } else {
            // 检查哪个文本能放下
            let (text_to_use, n_range, o_range) = if full_create_text.chars().count() <= self.cols {
                (&full_create_text, 0..=2, 24..=26)
            } else if medium_create_text.chars().count() <= self.cols {
                (&medium_create_text, 0..=2, 16..=19)
            } else {
                (&short_create_text, 0..=2, 11..=14)
            };

            (
                text_to_use.to_string(),
                Text::new(text_to_use)
                    .color_range(3, n_range)
                    .color_range(3, o_range),
            )
        }
    }

    fn create_help_line(&self) -> (String, Text) {
        let (text, highlight_range) = if self.entering_new_token_name.is_some() {
            (
                "Help: Enter optional name for new token, <Enter> to submit",
                41..=47,
            )
        } else if self.renaming_token.is_some() {
            (
                "Help: Enter new name for this token, <Enter> to submit",
                39..=45,
            )
        } else {
            (
                "Help: <Ctrl x> - revoke all tokens, <Esc> - go back",
                6..=13,
            )
        };

        let mut help_line = Text::new(text).color_range(3, highlight_range);

        // 为返回选项添加第二个高亮
        if self.entering_new_token_name.is_none() && self.renaming_token.is_none() {
            help_line = help_line.color_range(3, 36..=40);
        }

        (text.to_string(), help_line)
    }

    fn create_status_message(&self) -> Option<(String, Text)> {
        if let Some(error) = &self.error {
            Some((error.clone(), Text::new(error).color_range(3, ..)))
        } else if let Some(info) = &self.info {
            Some((info.clone(), Text::new(info).color_range(1, ..)))
        } else {
            None
        }
    }

    fn calculate_layout(&self, content: &ScreenContent) -> Layout {
        // 计算必须始终存在的固定界面元素：
        // - 1 行标题
        // - 1 行标题后间距（始终保留）
        // - 令牌条目（可变，可能被截断）
        // - 1 行 "创建新令牌" 行
        // - 1 行帮助前间距（始终保留）
        // - 1 行帮助文本或状态消息（现在互斥）

        let fixed_ui_rows = 4; // 标题 + 标题后间距 + 帮助前间距 + 帮助/状态
        let create_new_token_rows = 1;
        let token_item_rows = content.items.len();

        let total_content_rows = fixed_ui_rows + create_new_token_rows + token_item_rows;

        // 仅在有额外空间时添加顶部/底部内边距
        let base_y = if total_content_rows < self.rows {
            // 有空间放内边距 —— 内容居中
            (self.rows.saturating_sub(total_content_rows)) / 2
        } else {
            // 没有空间放内边距 —— 从顶部开始
            0
        };

        // 计算相对于 base_y 的位置
        let item_start_y = base_y + 2; // 标题 + 标题后间距
        let new_token_y = item_start_y + token_item_rows;
        let help_y = new_token_y + 1 + 1; // 新令牌行 + 帮助前间距

        Layout {
            base_x: (self.cols.saturating_sub(content.max_width) as f64 / 2.0).floor() as usize,
            base_y,
            title_x: self.cols.saturating_sub(content.title.0.len()) / 2,
            new_token_y,
            help_y,
            status_y: help_y, // 状态消息与帮助使用相同位置
        }
    }

    fn print_items_to_screen(&self, content: ScreenContent, layout: Layout) {
        print_text_with_coordinates(content.title.1, layout.title_x, layout.base_y, None, None);

        let mut table = Table::new().add_row(vec![" ", " ", " ", " "]);
        for item in content.items.into_iter() {
            table = table.add_styled_row(item);
        }

        print_table_with_coordinates(table, layout.base_x, layout.base_y + 1, None, None);

        if let Some((_, new_token_text)) = content.new_token_line {
            print_text_with_coordinates(
                new_token_text,
                layout.base_x,
                layout.new_token_y,
                None,
                None,
            );
        }

        if let Some((_, status_text)) = content.status_message {
            print_text_with_coordinates(status_text, layout.base_x, layout.status_y, None, None);
        } else {
            print_text_with_coordinates(content.help.1, layout.base_x, layout.help_y, None, None);
        }
    }
}
