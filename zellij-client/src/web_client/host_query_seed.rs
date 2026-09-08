//! 构建在连接时（以及后续配置重载时）用 web 客户端数据植入
//! Screen 的主机终端查询缓存的 IPC 消息。
//!
//! 原生客户端通过转发其真实终端报告的内容来填充
//! `Screen.terminal_emulator_colors`、`Screen.pixel_dimensions` 和
//! `Screen.terminal_emulator_color_codes`。web 客户端没有"真实终端" —
//! 其显示表面是浏览器中的 xterm.js — 因此我们从两个来源合成等效状态：
//!
//! 1. 驱动 `SetConfigPayload` 的同一个 `Config`，浏览器最终用它渲染。
//!    fg/bg 保证与 xterm.js 绘制的内容匹配。
//! 2. 索引 16-255 的规范 xterm-256 调色板公式（在 xterm.js 的可配置主题中
//!    不可覆盖，因此该公式是权威的）。
//!
//! 像素尺寸不在此处理 — 它们需要浏览器端测量，并通过 `TerminalMetrics`
//! 控制消息流动。

use zellij_utils::{
    data::PaletteColor,
    input::{config::Config, options::Options},
    ipc::{ClientToServerMsg, ColorRegister},
};

/// 构建在连接后立即发送到服务端的植入消息（以及每次 `SetConfig` 更新时）。
/// 如果无法从配置解析出主题信息，可能返回空 vec — 在这种情况下，服务端的缓存
/// 保持其默认值，`synthesize_cached_reply` 返回空字节，与既有行为一致。
pub fn build_host_query_seed_msgs(
    config: &Config,
    config_options: &Options,
) -> Vec<ClientToServerMsg> {
    let mut msgs = Vec::new();
    let resolved = resolve_theme(config, config_options);

    if let Some(fg) = resolved
        .foreground
        .as_ref()
        .and_then(|s| css_rgb_to_xparse(s))
    {
        msgs.push(ClientToServerMsg::ForegroundColor { color: fg });
    }
    if let Some(bg) = resolved
        .background
        .as_ref()
        .and_then(|s| css_rgb_to_xparse(s))
    {
        msgs.push(ClientToServerMsg::BackgroundColor { color: bg });
    }

    let registers = build_color_registers(&resolved);
    if !registers.is_empty() {
        msgs.push(ClientToServerMsg::ColorRegisters {
            color_registers: registers,
        });
    }

    msgs
}

#[derive(Default, Debug, Clone)]
struct ResolvedTheme {
    foreground: Option<String>,
    background: Option<String>,
    indexed: [Option<String>; 16],
}

/// 使用与 `SetConfigPayload::from(&Config)` 相同的优先级解析 web 客户端主题，
/// 以便服务端合成与浏览器中 xterm.js 绘制的颜色匹配。
fn resolve_theme(config: &Config, config_options: &Options) -> ResolvedTheme {
    let mut out = ResolvedTheme::default();

    let palette = config.theme_config(config_options.theme.as_ref());
    let web_client_theme = config.web_client.theme.as_ref();

    out.foreground = web_client_theme
        .and_then(|t| t.foreground.clone())
        .or_else(|| palette.map(|p| p.text_unselected.base.as_rgb_str()));
    out.background = web_client_theme
        .and_then(|t| t.background.clone())
        .or_else(|| palette.map(|p| p.text_unselected.background.as_rgb_str()));

    if let Some(t) = web_client_theme {
        out.indexed[0] = t.black.clone();
        out.indexed[1] = t.red.clone();
        out.indexed[2] = t.green.clone();
        out.indexed[3] = t.yellow.clone();
        out.indexed[4] = t.blue.clone();
        out.indexed[5] = t.magenta.clone();
        out.indexed[6] = t.cyan.clone();
        out.indexed[7] = t.white.clone();
        out.indexed[8] = t.bright_black.clone();
        out.indexed[9] = t.bright_red.clone();
        out.indexed[10] = t.bright_green.clone();
        out.indexed[11] = t.bright_yellow.clone();
        out.indexed[12] = t.bright_blue.clone();
        out.indexed[13] = t.bright_magenta.clone();
        out.indexed[14] = t.bright_cyan.clone();
        out.indexed[15] = t.bright_white.clone();
    }

    out
}

fn build_color_registers(resolved: &ResolvedTheme) -> Vec<ColorRegister> {
    let mut registers = Vec::with_capacity(256);

    // 索引 0-15：仅植入在 `web_client.theme.*` 中有显式覆盖的条目。
    // 未植入的索引回退到空合成（既有语义）— 应用程序将使用自己的默认值。
    for (i, slot) in resolved.indexed.iter().enumerate() {
        if let Some(css) = slot {
            if let Some(color) = css_rgb_to_xparse_color(css) {
                registers.push(ColorRegister { index: i, color });
            }
        }
    }

    // 索引 16-255：规范 xterm-256 调色板。xterm.js 不允许通过其主题选项覆盖这些，
    // Zellij 的配置也不暴露它们，因此该公式是事实来源。
    for index in 16u8..=255 {
        let (r, g, b) = xterm_256_rgb(index);
        registers.push(ColorRegister {
            index: index as usize,
            color: rgb_to_xparse_color(r, g, b),
        });
    }

    registers
}

/// 将 CSS `rgb(R, G, B)` 字符串（`PaletteColor::as_rgb_str` 产生的格式，
/// 也是 `SetConfigPayload` 发送给 xterm.js 的格式）转换为 `xparse_color` 接受的
/// `rgb:RRRR/GGGG/BBBB` 形式，为 `ClientToServerMsg::{Foreground,Background}Color`
/// 的消费者添加前缀。
fn css_rgb_to_xparse(css: &str) -> Option<String> {
    css_rgb_to_xparse_color(css)
}

/// 与 `css_rgb_to_xparse` 相同。保留为单独的别名，以便在调用点明确
/// OSC 10/11（进入 screen 的 terminal_emulator_colors 的 xparse 格式字符串）与
/// OSC 4（进入 terminal_emulator_color_codes 的颜色字符串）之间的区别 —
/// 目前两个消费者都想要相同的 `rgb:RRRR/GGGG/BBBB` 形状。
fn css_rgb_to_xparse_color(css: &str) -> Option<String> {
    let PaletteColor::Rgb((r, g, b)) = PaletteColor::from_rgb_str(css) else {
        return None;
    };
    Some(rgb_to_xparse_color(r, g, b))
}

fn rgb_to_xparse_color(r: u8, g: u8, b: u8) -> String {
    format!(
        "rgb:{:04x}/{:04x}/{:04x}",
        (r as u16) * 0x0101,
        (g as u16) * 0x0101,
        (b as u16) * 0x0101,
    )
}

/// 索引 16..=255 的规范 xterm-256 调色板。索引 0..=15 故意不在此处理 —
/// 那些是主题相关的，当存在覆盖时从 `WebClientTheme` 单独植入。
fn xterm_256_rgb(index: u8) -> (u8, u8, u8) {
    match index {
        0..=15 => (0, 0, 0), // 未使用；调用方限制为 16..=255
        16..=231 => {
            let idx = (index - 16) as u32;
            let level = |x: u32| if x == 0 { 0u8 } else { (55 + x * 40) as u8 };
            (level(idx / 36), level((idx / 6) % 6), level(idx % 6))
        },
        232..=255 => {
            let v = 8u8 + 10 * (index - 232);
            (v, v, v)
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zellij_utils::input::web_client::{WebClientConfig, WebClientTheme};

    fn config_with_web_theme(theme: WebClientTheme) -> Config {
        let mut config = Config::default();
        config.web_client = WebClientConfig {
            theme: Some(theme),
            ..WebClientConfig::default()
        };
        config
    }

    /// 定位植入构建器产生的 `ColorRegisters` 消息中与 `index` 匹配的
    /// 单个 `ColorRegister`。
    fn find_register(msgs: &[ClientToServerMsg], index: usize) -> Option<String> {
        for msg in msgs {
            if let ClientToServerMsg::ColorRegisters { color_registers } = msg {
                if let Some(reg) = color_registers.iter().find(|r| r.index == index) {
                    return Some(reg.color.clone());
                }
            }
        }
        None
    }

    fn fg_color(msgs: &[ClientToServerMsg]) -> Option<String> {
        msgs.iter().find_map(|m| match m {
            ClientToServerMsg::ForegroundColor { color } => Some(color.clone()),
            _ => None,
        })
    }

    fn bg_color(msgs: &[ClientToServerMsg]) -> Option<String> {
        msgs.iter().find_map(|m| match m {
            ClientToServerMsg::BackgroundColor { color } => Some(color.clone()),
            _ => None,
        })
    }

    fn registers_msg(msgs: &[ClientToServerMsg]) -> Option<usize> {
        msgs.iter().find_map(|m| match m {
            ClientToServerMsg::ColorRegisters { color_registers } => Some(color_registers.len()),
            _ => None,
        })
    }

    #[test]
    fn xterm_256_first_cube_entry_is_black() {
        assert_eq!(xterm_256_rgb(16), (0, 0, 0));
    }

    #[test]
    fn xterm_256_last_cube_entry_is_white() {
        // 5,5,5 — 最大级别的角落
        assert_eq!(xterm_256_rgb(231), (255, 255, 255));
    }

    #[test]
    fn xterm_256_greyscale_endpoints() {
        assert_eq!(xterm_256_rgb(232), (8, 8, 8));
        assert_eq!(xterm_256_rgb(255), (238, 238, 238));
    }

    #[test]
    fn cube_level_formula_matches_xterm_table() {
        // 索引 17: r=0, g=0, b=1  => (0,0,95)
        assert_eq!(xterm_256_rgb(17), (0, 0, 95));
        // 索引 21: r=0, g=0, b=5  => (0,0,255)
        assert_eq!(xterm_256_rgb(21), (0, 0, 255));
        // 索引 196: 196-16=180; r=180/36=5, g=(180/6)%6=0, b=180%6=0 => (255,0,0)
        assert_eq!(xterm_256_rgb(196), (255, 0, 0));
    }

    #[test]
    fn rgb_to_xparse_round_trip() {
        assert_eq!(rgb_to_xparse_color(0x12, 0x34, 0x56), "rgb:1212/3434/5656");
        assert_eq!(rgb_to_xparse_color(0, 0, 0), "rgb:0000/0000/0000");
        assert_eq!(rgb_to_xparse_color(255, 255, 255), "rgb:ffff/ffff/ffff");
    }

    #[test]
    fn css_rgb_str_parses_through_palette_color() {
        // PaletteColor::from_rgb_str 接受 "rgb(R, G, B)"。
        assert_eq!(
            css_rgb_to_xparse_color("rgb(255, 128, 0)").as_deref(),
            Some("rgb:ffff/8080/0000"),
        );
    }

    #[test]
    fn invalid_css_rgb_yields_no_conversion() {
        // PaletteColor::from_rgb_str 对无法解析的字符串返回默认变体；
        // 桥接必须拒绝这些，而不是发出垃圾 xparse 字符串。
        assert!(css_rgb_to_xparse_color("#ff8800").is_none());
        assert!(css_rgb_to_xparse_color("not a color").is_none());
        assert!(css_rgb_to_xparse_color("").is_none());
    }

    #[test]
    fn seed_with_explicit_fg_bg_emits_both_messages() {
        // 用户已显式设置 web_client.theme.foreground 和 .background —
        // 植入构建器必须将它们作为 xparse 格式的 ForegroundColor /
        // BackgroundColor 消息传播。
        let mut theme = WebClientTheme::default();
        theme.foreground = Some("rgb(200, 200, 200)".to_string());
        theme.background = Some("rgb(20, 20, 20)".to_string());
        let config = config_with_web_theme(theme);
        let opts = Options::default();

        let msgs = build_host_query_seed_msgs(&config, &opts);

        assert_eq!(fg_color(&msgs).as_deref(), Some("rgb:c8c8/c8c8/c8c8"));
        assert_eq!(bg_color(&msgs).as_deref(), Some("rgb:1414/1414/1414"));
    }

    #[test]
    fn seed_with_indexed_overrides_populates_those_registers() {
        // WebClientTheme 中的每索引覆盖必须出现在 ColorRegisters 有效载荷中
        // 正确的 ANSI 索引处。
        let mut theme = WebClientTheme::default();
        theme.red = Some("rgb(204, 0, 0)".to_string()); // 索引 1
        theme.bright_blue = Some("rgb(50, 100, 200)".to_string()); // 索引 12
        let config = config_with_web_theme(theme);
        let opts = Options::default();

        let msgs = build_host_query_seed_msgs(&config, &opts);

        assert_eq!(
            find_register(&msgs, 1).as_deref(),
            Some("rgb:cccc/0000/0000"),
        );
        assert_eq!(
            find_register(&msgs, 12).as_deref(),
            Some("rgb:3232/6464/c8c8"),
        );
    }

    #[test]
    fn seed_omits_unset_low_indices() {
        // 没有显式覆盖的索引 0-15 应该不在 ColorRegisters 有效载荷中 —
        // 应用程序将回退到自己的默认值，而不是读取我们编造的错误值。
        let mut theme = WebClientTheme::default();
        theme.red = Some("rgb(255, 0, 0)".to_string());
        let config = config_with_web_theme(theme);
        let opts = Options::default();

        let msgs = build_host_query_seed_msgs(&config, &opts);

        // 只有 `red`（索引 1）被设置。
        assert_eq!(find_register(&msgs, 0), None);
        assert_eq!(find_register(&msgs, 2), None);
        assert!(find_register(&msgs, 1).is_some());
    }

    #[test]
    fn seed_always_populates_extended_palette_16_to_255() {
        // 16-255 范围是规范的 xterm-256 调色板，与主题配置无关。
        // 验证计数（240 个条目）和几个抽查值。
        let config = Config::default();
        let opts = Options::default();

        let msgs = build_host_query_seed_msgs(&config, &opts);

        // 默认配置没有每索引覆盖，因此 ColorRegisters 有效载荷恰好包含
        // 240 个条目（16..=255）。
        assert_eq!(registers_msg(&msgs), Some(240));

        // 索引 196 是 6x6x6 立方体中的规范纯红色。
        assert_eq!(
            find_register(&msgs, 196).as_deref(),
            Some("rgb:ffff/0000/0000"),
        );
        // 索引 232 是第一个灰度步骤（rgb 8/8/8）。
        assert_eq!(
            find_register(&msgs, 232).as_deref(),
            Some("rgb:0808/0808/0808"),
        );
        // 索引 255 是最后一个灰度步骤（rgb 238/238/238）。
        assert_eq!(
            find_register(&msgs, 255).as_deref(),
            Some("rgb:eeee/eeee/eeee"),
        );
    }

    #[test]
    fn seed_with_overrides_sums_to_240_plus_overrides() {
        // 添加每索引覆盖应该增加 ColorRegisters 有效载荷 —
        // 240 个用于扩展范围，加上 0..=15 中每个覆盖一个条目。
        let mut theme = WebClientTheme::default();
        theme.red = Some("rgb(255, 0, 0)".to_string());
        theme.green = Some("rgb(0, 255, 0)".to_string());
        theme.blue = Some("rgb(0, 0, 255)".to_string());
        let config = config_with_web_theme(theme);
        let opts = Options::default();

        let msgs = build_host_query_seed_msgs(&config, &opts);

        assert_eq!(registers_msg(&msgs), Some(240 + 3));
    }

    #[test]
    fn seed_skips_color_messages_when_no_theme_resolvable() {
        // 主题解析不产生任何内容的 Config（没有 web_client.theme，没有匹配的主主题）
        // 应该不产生 ForegroundColor / BackgroundColor 消息 — 服务端的缓存
        // 保持其默认值，而不是被破坏。
        let mut config = Config::default();
        // 通过也清除主题名称来强制主题解析失败。确切的内部细节取决于
        // Config::theme_config()，因此此测试仅断言：无论解析器决定什么，
        // 植入构建器都不能发出 "Some(空字符串)" 垃圾。
        config.web_client = WebClientConfig::default();
        let opts = Options::default();

        let msgs = build_host_query_seed_msgs(&config, &opts);

        // 无论解析器选择什么 fg/bg，桥接都必须产生格式良好的 xparse 字符串
        // 或完全跳过消息。
        for msg in &msgs {
            match msg {
                ClientToServerMsg::ForegroundColor { color }
                | ClientToServerMsg::BackgroundColor { color } => {
                    assert!(
                        color.starts_with("rgb:"),
                        "expected xparse-format string, got: {:?}",
                        color
                    );
                },
                _ => {},
            }
        }
    }

    #[test]
    fn seed_color_registers_always_emit_xparse_format() {
        // ColorRegisters 有效载荷中的每个条目都必须使用与
        // synthesize_cached_reply 原样重新发出的相同 `rgb:RRRR/GGGG/BBBB` 形状 —
        // 没有 `rgb(R, G, B)` CSS 字符串泄漏通过。
        let mut theme = WebClientTheme::default();
        theme.black = Some("rgb(0, 0, 0)".to_string());
        theme.bright_white = Some("rgb(255, 255, 255)".to_string());
        let config = config_with_web_theme(theme);
        let opts = Options::default();

        let msgs = build_host_query_seed_msgs(&config, &opts);

        let registers = msgs
            .iter()
            .find_map(|m| match m {
                ClientToServerMsg::ColorRegisters { color_registers } => Some(color_registers),
                _ => None,
            })
            .expect("ColorRegisters message missing");
        for reg in registers {
            assert!(
                reg.color.starts_with("rgb:") && reg.color.len() == 4 + 4 + 1 + 4 + 1 + 4,
                "register {} has malformed color string: {:?}",
                reg.index,
                reg.color
            );
        }
    }
}
