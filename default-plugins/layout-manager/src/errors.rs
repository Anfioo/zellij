use crate::screens::{KeyResponse, Screen};
use crate::ui::{truncate_line_with_ansi, wrap_text_to_width, ErrorMessage, MultiLineErrorMessage};
use zellij_tile::prelude::*;

///  将布局解析错误格式化为详细的错误字符串
pub fn format_kdl_error(error: LayoutParsingError) -> String {
    match error {
        LayoutParsingError::KdlError {
            mut kdl_error,
            file_name,
            source_code,
        } => {
            use miette::{GraphicalReportHandler, NamedSource, Report};

            kdl_error.help_message =
                Some("https://zellij.dev/documentation/creating-a-layout.html".to_owned());
            let report: Report = kdl_error.into();
            let report = report.with_source_code(NamedSource::new(file_name, source_code));

            let handler = GraphicalReportHandler::new();
            let mut output = String::new();
            handler.render_report(&mut output, report.as_ref()).unwrap();
            output
        },
        LayoutParsingError::SyntaxError => {
            format!("无法反序列化 KDL 节点。\n可能的原因：\n{}\n{}\n{}\n{}",
            "- 节点名称后缺少 `;`，例如 { node; another_node; }",
            "- 参数节点周围缺少引号 (\")，例如 { first_node \"argument_node\"; }",
            "- 标题行上的节点参数之间缺少等号 (=)。例如 argument=\"value\"",
            "- 节点子参数与其值之间发现多余的等号 (=)。例如 { argument=\"value\" }")
        },
    }
}

#[derive(Clone)]
pub struct ErrorScreen {
    pub message: String,
    pub return_to_screen: Box<Screen>,
}

impl ErrorScreen {
    pub fn handle_key(&mut self, _key: KeyWithModifier) -> KeyResponse {
        KeyResponse::new_screen((*self.return_to_screen).clone())
    }

    pub fn render(&self, rows: usize, cols: usize) {
        if self.message.chars().count() > cols.saturating_sub(4) {
            let max_width = cols.saturating_sub(4);
            let max_rows = rows.saturating_sub(4);
            let lines = wrap_text_to_width(&self.message, max_width);
            let base_x = 2;
            let base_y = rows.saturating_sub(4 + lines.len()) / 2;
            MultiLineErrorMessage::new(lines).render(base_x, base_y, max_rows);
        } else {
            let desired_width = self.message.chars().count();
            let base_y = rows.saturating_sub(5) / 2;
            let base_x = cols.saturating_sub(desired_width) / 2;
            ErrorMessage::new(&self.message).render(base_x, base_y);
        }
    }
}

#[derive(Clone)]
pub struct ErrorDetailScreen {
    pub layout_name: String,
    pub detailed_error: String,
    pub return_to_screen: Box<Screen>,
}

impl ErrorDetailScreen {
    pub fn new(layout_name: String, detailed_error: String, return_to_screen: Box<Screen>) -> Self {
        Self {
            layout_name,
            detailed_error,
            return_to_screen,
        }
    }

    pub fn handle_key(&mut self, _key: KeyWithModifier) -> KeyResponse {
        KeyResponse::new_screen((*self.return_to_screen).clone())
    }

    pub fn render(&self, rows: usize, cols: usize) {
        // 表头：显示布局名称
        let header = format!("布局错误：{}", self.layout_name);
        let header_text = Text::new(&header).error_color_all();
        print_text_with_coordinates(header_text, 1, 0, None, None);

        //  计算错误内容的可用空间
        let header_height = 2; // 表头 + 间距
        let available_rows = rows.saturating_sub(header_height);
        let available_cols = cols.saturating_sub(2); // 两侧内边距

        // 渲染带中间截断的错误行
        let error_lines: Vec<&str> = self.detailed_error.lines().collect();
        let total_lines = error_lines.len();

        if total_lines <= available_rows {
            //  所有行都适合，全部显示
            for (i, line) in error_lines.iter().enumerate() {
                let truncated = truncate_line_with_ansi(line, available_cols);
                print!("\u{1b}[{};{}H{}", header_height + i + 1, 2, truncated);
            }
        } else {
            //  需要截断中间部分 - 显示开头、指示器和结尾
            let omitted_indicator_lines = 1; //  为 "... X lines omitted ..." 预留 1 行
            let lines_for_content = available_rows.saturating_sub(omitted_indicator_lines);

            //  分割内容空间：60% 用于开头，40% 用于结尾
            let beginning_lines = (lines_for_content as f32 * 0.6).ceil() as usize;
            let end_lines = lines_for_content.saturating_sub(beginning_lines);

            let omitted_count = total_lines.saturating_sub(beginning_lines + end_lines);

            let mut current_row = 0;

            //  渲染开头行
            for line in error_lines.iter().take(beginning_lines) {
                let truncated = truncate_line_with_ansi(line, available_cols);
                print!(
                    "\u{1b}[{};{}H{}",
                    header_height + current_row + 1,
                    2,
                    truncated
                );
                current_row += 1;
            }

            //  渲染省略指示器
            let indicator = format!("……省略了 {} 行……", omitted_count);
            let indicator_text = Text::new(&indicator).color_range(0, ..);
            print_text_with_coordinates(
                indicator_text,
                (cols.saturating_sub(indicator.chars().count())) / 2,
                header_height + current_row,
                None,
                None,
            );
            current_row += 1;

            //  渲染末尾行
            let start_index = total_lines.saturating_sub(end_lines);
            for line in error_lines.iter().skip(start_index) {
                let truncated = truncate_line_with_ansi(line, available_cols);
                print!(
                    "\u{1b}[{};{}H{}",
                    header_height + current_row + 1,
                    2,
                    truncated
                );
                current_row += 1;
            }
        }
    }
}
