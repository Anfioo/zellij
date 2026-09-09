mod component_coordinates;
mod nested_list;
mod ribbon;
mod table;
mod text;

use crate::panes::grid::Grid;
use lazy_static::lazy_static;
use regex::Regex;
use vte;
use zellij_utils::data::Style;
use zellij_utils::errors::prelude::*;

use component_coordinates::{is_too_high, is_too_wide, Coordinates};
use nested_list::{nested_list, parse_nested_list_items};
use ribbon::ribbon;
use table::table;
use text::{parse_text, parse_text_params, stringify_text, text, Text};

macro_rules! parse_next_param {
    ($next_param:expr, $type:ident, $component_name:expr, $item_name:expr) => {{
        $next_param
            .and_then(|stringified_param| stringified_param.parse::<$type>().ok())
            .with_context(|| format!("{} must have {}", $component_name, $item_name))?
    }};
}

macro_rules! parse_vte_bytes {
    ($self:expr, $encoded_component:expr) => {{
        let mut vte_parser = vte::Parser::new();
        vte_parser.advance($self.grid, &$encoded_component);
    }};
}

#[derive(Debug)]
pub struct UiComponentParser<'a> {
    grid: &'a mut Grid,
    style: Style,
    arrow_fonts: bool,
}

impl<'a> UiComponentParser<'a> {
    pub fn new(grid: &'a mut Grid, style: Style, arrow_fonts: bool) -> Self {
        UiComponentParser {
            grid,
            style,
            arrow_fonts,
        }
    }
    pub fn parse(&mut self, bytes: Vec<u8>) -> Result<()> {
        // 解析的阶段：
        // 1. 我们将字节解码为utf8，得到类似（作为String）的内容：`component_name;111;222;333`
        // 2. 我们用`;`分割这个字符串以获取参数本身
        // 3. 我们提取组件名称，然后根据组件进行相应处理
        // 4. 某些组件将其参数解释为字节，因此还有另一层utf8解码，其他组件则直接使用，
        //    有些组件会根据它们的位置来执行操作（例如`table`组件将前两个参数视为整数，
        //    用于表格的列/行，然后将组件的其余部分视为utf8编码的字节，每个字节代表表格中的一个单元格）
        // 5. 每个组件解析其参数，创建一个自己的ANSI指令字符串，表示创建组件的指令
        // 6. 最后，我们获取这个字符串，将其编码回字节，并通过ANSI解析器（我们的`Grid`）传回，
        //    以便在屏幕上创建它的表示
        let mut params: Vec<String> = String::from_utf8_lossy(&bytes)
            .to_string()
            .split(';')
            .map(|c| c.to_owned())
            .collect();
        let mut params_iter = params.iter_mut().peekable();
        let component_name = params_iter
            .next()
            .with_context(|| format!("UI 组件必须具有名称"))?;

        // 解析坐标
        let mut component_coordinates = None;
        if let Some(coordinates) = params_iter.peek() {
            component_coordinates = self.parse_coordinates(coordinates)?;
            if component_coordinates.is_some() {
                let _ = params_iter.next(); // 我们刚刚peek了，现在让我们消费坐标
            }
        }

        if component_name == &"table" {
            let columns = parse_next_param!(params_iter.next(), usize, "table", "columns");
            let rows = parse_next_param!(params_iter.next(), usize, "table", "rows");
            let stringified_params = parse_text_params(params_iter);
            let encoded_table = table(
                columns,
                rows,
                stringified_params,
                &self.style,
                component_coordinates,
            );
            parse_vte_bytes!(self, encoded_table);
            Ok(())
        } else if component_name == &"ribbon" {
            let stringified_params = parse_text_params(params_iter)
                .into_iter()
                .next()
                .with_context(|| format!("丝带组件必须具有文本"))?;
            let encoded_text = ribbon(
                stringified_params,
                &self.style,
                self.arrow_fonts,
                component_coordinates,
            );
            parse_vte_bytes!(self, encoded_text);
            Ok(())
        } else if component_name == &"nested_list" {
            let nested_list_items = parse_nested_list_items(params_iter);
            let encoded_nested_list =
                nested_list(nested_list_items, &self.style, component_coordinates);
            parse_vte_bytes!(self, encoded_nested_list);
            Ok(())
        } else if component_name == &"text" {
            let stringified_params = parse_text_params(params_iter)
                .into_iter()
                .next()
                .with_context(|| format!("文本组件必须具有……文本……"))?;
            let encoded_text = text(stringified_params, &self.style, component_coordinates);
            parse_vte_bytes!(self, encoded_text);
            Ok(())
        } else {
            Err(anyhow!("Unknown component: {}", component_name))
        }
    }
    fn parse_coordinates(&self, coordinates: &str) -> Result<Option<Coordinates>> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(\d*)/(\d*)/(\d*)/(\d*)").unwrap();
        }
        if let Some(captures) = RE.captures_iter(&coordinates).next() {
            let x = captures[1].parse::<usize>().with_context(|| {
                format!(
                    "解析字符串 {:?} 的 x 坐标失败",
                    coordinates
                )
            })?;
            let y = captures[2].parse::<usize>().with_context(|| {
                format!(
                    "解析字符串 {:?} 的 y 坐标失败",
                    coordinates
                )
            })?;
            let width = captures[3].parse::<usize>().ok();
            let height = captures[4].parse::<usize>().ok();
            Ok(Some(Coordinates {
                x,
                y,
                width,
                height,
            }))
        } else {
            Ok(None)
        }
    }
}

fn parse_flag(stringified: &mut String, flag: char) -> bool {
    const FLAG_CHARS: [char; 3] = ['x', 'z', 'd'];
    let prefix_len = stringified
        .chars()
        .take_while(|c| FLAG_CHARS.contains(c))
        .count();
    if let Some(position) = stringified[..prefix_len].find(flag) {
        stringified.remove(position);
        true
    } else {
        false
    }
}

fn parse_selected(stringified: &mut String) -> bool {
    parse_flag(stringified, 'x')
}

fn parse_opaque(stringified: &mut String) -> bool {
    parse_flag(stringified, 'z')
}

fn parse_disabled(stringified: &mut String) -> bool {
    parse_flag(stringified, 'd')
}

fn parse_indices(stringified: &mut String) -> Vec<Vec<usize>> {
    stringified
        .chars()
        .collect::<Vec<_>>()
        .iter()
        .rposition(|c| c == &'$')
        .map(|last_position| stringified.drain(0..=last_position).collect::<String>())
        .map(|indices_string| {
            let mut all_indices = vec![];
            let raw_indices_for_each_variant = indices_string.split('$');
            for index_string in raw_indices_for_each_variant {
                let indices_for_variant = index_string
                    .split(',')
                    .filter_map(|s| s.parse::<usize>().ok())
                    .collect();
                all_indices.push(indices_for_variant)
            }
            all_indices
        })
        .unwrap_or_default()
}
