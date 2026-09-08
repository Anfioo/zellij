use super::{
    is_too_wide, parse_disabled, parse_indices, parse_opaque, parse_selected, Coordinates,
};
use crate::panes::terminal_character::RESET_STYLES;
use crate::panes::{terminal_character::CharacterStyles, AnsiCode};
use zellij_utils::{
    data::{PaletteColor, Style, StyleDeclaration},
    shared::ansi_len,
};

use unicode_width::UnicodeWidthChar;
use zellij_utils::errors::prelude::*;

pub fn text(content: Text, style: &Style, component_coordinates: Option<Coordinates>) -> Vec<u8> {
    if content.disabled {
        let declaration = style.colors.text_unselected;
        let mut base_text_style = RESET_STYLES
            .foreground(Some(declaration.base.into()))
            .italic(Some(AnsiCode::On));
        if content.opaque || content.selected {
            base_text_style = base_text_style.background(Some(declaration.background.into()));
        }
        let disabled_content = content.into_disabled();
        let (text, _text_width) = stringify_text(
            &disabled_content,
            None,
            &component_coordinates,
            &declaration,
            &style.colors,
            base_text_style,
        );
        return match component_coordinates {
            Some(component_coordinates) => {
                format!("{}{}{}", component_coordinates, base_text_style, text)
                    .as_bytes()
                    .to_vec()
            },
            None => format!("{}{}", base_text_style, text).as_bytes().to_vec(),
        };
    }
    let declaration = if content.selected {
        style.colors.text_selected
    } else {
        style.colors.text_unselected
    };

    // 从声明中获取基础样式开始
    let base_text_style = CharacterStyles::from(declaration).bold(Some(AnsiCode::On));

    let (text, _text_width) = stringify_text(
        &content,
        None,
        &component_coordinates,
        &declaration,
        &style.colors,
        base_text_style,
    );
    match component_coordinates {
        Some(component_coordinates) => {
            format!("{}{}{}", component_coordinates, base_text_style, text)
                .as_bytes()
                .to_vec()
        },
        None => format!("{}{}", base_text_style, text).as_bytes().to_vec(),
    }
}

pub fn stringify_text(
    text: &Text,
    left_padding: Option<usize>,
    coordinates: &Option<Coordinates>,
    style: &StyleDeclaration,
    styling: &zellij_utils::data::Styling,
    component_text_style: CharacterStyles,
) -> (String, usize) {
    let mut text_width = 0;
    let mut stringified = String::new();
    let base_text_style = if text.opaque || text.selected {
        component_text_style.background(Some(style.background.into()))
    } else {
        component_text_style
    };
    stringified.push_str(&format!("{}", base_text_style));
    for (i, character) in text.text.chars().enumerate() {
        let character_width = character.width().unwrap_or(0);
        if is_too_wide(
            character_width,
            left_padding.unwrap_or(0) + text_width,
            &coordinates,
        ) {
            break;
        }
        text_width += character_width;

        if text.selected || text.opaque {
            // 我们这样做是为了让选中的文本即使没有颜色索引也显示为选中状态
            stringified.push_str(&format!("{}", base_text_style));
        }

        if !text.indices.is_empty() || text.selected || text.opaque {
            let character_with_styling =
                color_index_character(character, i, &text, style, styling, base_text_style);
            stringified.push_str(&character_with_styling);
        } else {
            stringified.push(character)
        }
    }
    let coordinates_width = coordinates.as_ref().and_then(|c| c.width);
    match (coordinates_width, base_text_style.background) {
        (Some(coordinates_width), Some(_background_style)) => {
            let text_width_with_left_padding = text_width + left_padding.unwrap_or(0);
            let background_padding_length =
                coordinates_width.saturating_sub(text_width_with_left_padding);
            if text_width_with_left_padding < coordinates_width {
                // 这里我们用空白填充字符串直到末尾，以便背景样式将应用于坐标的整个长度
                stringified.push_str(&format!(
                    "{:width$}",
                    " ",
                    width = background_padding_length
                ));
            }
            text_width += background_padding_length;
        },
        _ => {},
    }
    (stringified, text_width)
}

pub fn color_index_character(
    character: char,
    index: usize,
    text: &Text,
    declaration: &StyleDeclaration,
    styling: &zellij_utils::data::Styling,
    base_text_style: CharacterStyles,
) -> String {
    let mut character_style = text
        .style_of_index(index, declaration, styling)
        .map(|foreground_style| base_text_style.foreground(Some(foreground_style.into())))
        .unwrap_or(base_text_style);

    // 根据索引级别4和5对每个字符应用变暗和取消粗体
    if text.is_unbold_at(index) {
        // 移除此字符的粗体
        character_style = character_style.bold(Some(AnsiCode::Reset));
    } else if text.is_dimmed_at(index) {
        // 对此字符应用变暗
        character_style = character_style
            .foreground(Some(AnsiCode::Reset)) // 某些终端（例如 alacritty）不支持对非16种颜色进行变暗处理，
            // 所以我们这里必须使用终端的默认值
            .dim(Some(AnsiCode::On));
    } else {
        character_style = character_style
            .bold(Some(AnsiCode::On))
            .dim(Some(AnsiCode::Reset)); // 默认值，用于重置之前索引中可能的变暗/粗体值
    }

    format!("{}{}{}", character_style, character, base_text_style)
}

pub fn parse_text_params<'a>(params_iter: impl Iterator<Item = &'a mut String>) -> Vec<Text> {
    params_iter
        .flat_map(|mut stringified| {
            let selected = parse_selected(&mut stringified);
            let opaque = parse_opaque(&mut stringified);
            let disabled = parse_disabled(&mut stringified);
            let indices = parse_indices(&mut stringified);
            let text = parse_text(&mut stringified).map_err(|e| e.to_string())?;
            Ok::<Text, String>(Text {
                text,
                opaque,
                selected,
                disabled,
                indices,
            })
        })
        .collect::<Vec<Text>>()
}

#[derive(Debug, Clone)]
pub struct Text {
    pub text: String,
    pub selected: bool,
    pub opaque: bool,
    pub disabled: bool,
    pub indices: Vec<Vec<usize>>,
}

impl Text {
    pub fn into_disabled(mut self) -> Self {
        self.selected = false;
        self.opaque = false;
        self.disabled = true;
        self.indices = vec![];
        self
    }

    pub fn pad_text(&mut self, max_column_width: usize) {
        for _ in ansi_len(&self.text)..max_column_width {
            self.text.push(' ');
        }
    }

    pub fn is_dimmed_at(&self, index: usize) -> bool {
        const DIM_LEVEL: usize = 4;
        self.indices
            .get(DIM_LEVEL)
            .map(|indices| indices.contains(&index))
            .unwrap_or(false)
    }

    pub fn is_unbold_at(&self, index: usize) -> bool {
        const UNBOLD_LEVEL: usize = 5;
        self.indices
            .get(UNBOLD_LEVEL)
            .map(|indices| indices.contains(&index))
            .unwrap_or(false)
    }

    pub fn style_of_index(
        &self,
        index: usize,
        style: &StyleDeclaration,
        styling: &zellij_utils::data::Styling,
    ) -> Option<PaletteColor> {
        const ERROR_COLOR_LEVEL: usize = 6;
        const SUCCESS_COLOR_LEVEL: usize = 7;

        // 首先检查错误颜色（最高优先级）
        if let Some(indices) = self.indices.get(ERROR_COLOR_LEVEL) {
            if indices.contains(&index) {
                return Some(styling.exit_code_error.base);
            }
        }

        // 检查成功颜色（第二高优先级）
        if let Some(indices) = self.indices.get(SUCCESS_COLOR_LEVEL) {
            if indices.contains(&index) {
                return Some(styling.exit_code_success.base);
            }
        }

        // 检查常规强调级别（现有代码）
        let index_variant_styles = [
            style.emphasis_0,
            style.emphasis_1,
            style.emphasis_2,
            style.emphasis_3,
        ];
        for i in (0..=3).rev() {
            // 我们反向执行此操作，以便优先考虑最后应用的样式
            if let Some(indices) = self.indices.get(i) {
                if indices.contains(&index) {
                    return Some(index_variant_styles[i]);
                }
            }
        }
        Some(style.base)
    }
}

pub fn parse_text(stringified: &mut String) -> Result<String> {
    let mut utf8 = vec![];
    for stringified_character in stringified.split(',') {
        utf8.push(
            stringified_character
                .to_string()
                .parse::<u8>()
                .with_context(|| format!("Failed to parse utf8"))?,
        );
    }
    Ok(String::from_utf8_lossy(&utf8).to_string())
}
