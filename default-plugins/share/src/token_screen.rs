use zellij_tile::prelude::*;

// 文本内容常量
const TOKEN_LABEL_LONG: &str = "新登录令牌： ";
const TOKEN_LABEL_SHORT: &str = "令牌： ";
const EXPLANATION_1_LONG: &str = "使用此令牌从浏览器登录。";
const EXPLANATION_1_SHORT: &str = "用于从浏览器登录。";
const EXPLANATION_2_LONG: &str =
    "请复制此令牌，因为它不会被保存且无法找回。";
const EXPLANATION_2_SHORT: &str = "它不会被保存且无法找回。";
const EXPLANATION_3_LONG: &str = "如果丢失，可以随时撤销并生成新的令牌。";
const EXPLANATION_3_SHORT: &str = "它可以随时撤销并重新生成。";
const ESC_INSTRUCTION: &str = "<Esc> - 返回";

// 界面布局常量
const SCREEN_HEIGHT: usize = 7;
const TOKEN_Y_OFFSET: usize = 0;
const EXPLANATION_1_Y_OFFSET: usize = 2;
const EXPLANATION_2_Y_OFFSET: usize = 4;
const EXPLANATION_3_Y_OFFSET: usize = 5;
const ESC_Y_OFFSET: usize = 7;
const ERROR_Y_OFFSET: usize = 8;

struct TextVariant {
    long: &'static str,
    short: &'static str,
}

impl TextVariant {
    fn select(&self, cols: usize) -> &'static str {
        if cols >= self.long.chars().count() {
            self.long
        } else {
            self.short
        }
    }
}

pub struct TokenScreen {
    token: String,
    web_server_error: Option<String>,
    rows: usize,
    cols: usize,
}

impl TokenScreen {
    pub fn new(token: String, web_server_error: Option<String>, rows: usize, cols: usize) -> Self {
        Self {
            token,
            web_server_error,
            rows,
            cols,
        }
    }

    pub fn render(&self) {
        let elements = self.prepare_screen_elements();
        let width = self.calculate_max_width(&elements);
        let (base_x, base_y) = self.calculate_base_position(width);

        self.render_elements(&elements, base_x, base_y);
        self.render_error_if_present(base_x, base_y);
    }

    fn prepare_screen_elements(&self) -> ScreenElements {
        let token_variant = TextVariant {
            long: TOKEN_LABEL_LONG,
            short: TOKEN_LABEL_SHORT,
        };

        let explanation_variants = [
            TextVariant {
                long: EXPLANATION_1_LONG,
                short: EXPLANATION_1_SHORT,
            },
            TextVariant {
                long: EXPLANATION_2_LONG,
                short: EXPLANATION_2_SHORT,
            },
            TextVariant {
                long: EXPLANATION_3_LONG,
                short: EXPLANATION_3_SHORT,
            },
        ];

        let token_label = token_variant.select(
            self.cols
                .saturating_sub(self.token.chars().count().saturating_sub(1)),
        );
        let token_text = format!("{}{}", token_label, self.token);
        let token_element = self.create_token_text_element(&token_text, token_label);

        let explanation_texts: Vec<&str> = explanation_variants
            .iter()
            .map(|variant| variant.select(self.cols))
            .collect();

        let explanation_elements: Vec<Text> = explanation_texts
            .iter()
            .enumerate()
            .map(|(i, &text)| {
                if i == 0 {
                    Text::new(text).color_range(0, ..)
                } else {
                    Text::new(text)
                }
            })
            .collect();

        let esc_element = Text::new(ESC_INSTRUCTION).color_range(3, ..=4);

        ScreenElements {
            token: token_element,
            token_text,
            explanation_texts,
            explanations: explanation_elements,
            esc: esc_element,
        }
    }

    fn create_token_text_element(&self, token_text: &str, token_label: &str) -> Text {
        Text::new(token_text).color_range(2, ..token_label.chars().count())
    }

    fn calculate_max_width(&self, elements: &ScreenElements) -> usize {
        let token_width = elements.token_text.chars().count();
        let explanation_widths = elements
            .explanation_texts
            .iter()
            .map(|text| text.chars().count());
        let esc_width = ESC_INSTRUCTION.chars().count();

        [token_width, esc_width]
            .into_iter()
            .chain(explanation_widths)
            .max()
            .unwrap_or(0)
    }

    fn calculate_base_position(&self, width: usize) -> (usize, usize) {
        let base_x = self.cols.saturating_sub(width) / 2;
        let base_y = self.rows.saturating_sub(SCREEN_HEIGHT) / 2;
        (base_x, base_y)
    }

    fn render_elements(&self, elements: &ScreenElements, base_x: usize, base_y: usize) {
        print_text_with_coordinates(
            elements.token.clone(),
            base_x,
            base_y + TOKEN_Y_OFFSET,
            None,
            None,
        );

        let y_offsets = [
            EXPLANATION_1_Y_OFFSET,
            EXPLANATION_2_Y_OFFSET,
            EXPLANATION_3_Y_OFFSET,
        ];
        for (explanation, &y_offset) in elements.explanations.iter().zip(y_offsets.iter()) {
            print_text_with_coordinates(explanation.clone(), base_x, base_y + y_offset, None, None);
        }

        print_text_with_coordinates(
            elements.esc.clone(),
            base_x,
            base_y + ESC_Y_OFFSET,
            None,
            None,
        );
    }

    fn render_error_if_present(&self, base_x: usize, base_y: usize) {
        if let Some(error) = &self.web_server_error {
            print_text_with_coordinates(
                Text::new(error).color_range(3, ..),
                base_x,
                base_y + ERROR_Y_OFFSET,
                None,
                None,
            );
        }
    }
}

struct ScreenElements {
    token: Text,
    token_text: String,
    explanation_texts: Vec<&'static str>,
    explanations: Vec<Text>,
    esc: Text,
}
