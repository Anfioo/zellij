use super::super::TerminalPane;
use crate::panes::kitty_graphics::KittyImageStore;
use crate::panes::sixel::SixelImageStore;
use crate::panes::LinkHandler;
use crate::tab::Pane;
use insta::assert_snapshot;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use zellij_utils::data::{Palette, Style};
use zellij_utils::pane_size::PaneGeom;

fn read_fixture() -> Vec<u8> {
    let mut path_to_file = std::path::PathBuf::new();
    path_to_file.push("../src");
    path_to_file.push("tests");
    path_to_file.push("fixtures");
    path_to_file.push("grid_copy");
    std::fs::read(path_to_file)
        .unwrap_or_else(|_| panic!("could not read fixture ../src/tests/fixtures/grid_copy"))
}

fn create_pane() -> TerminalPane {
    let mut fake_win_size = PaneGeom::default();
    fake_win_size.cols.set_inner(121);
    fake_win_size.rows.set_inner(20);

    let pid = 1;
    let style = Style::default();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut terminal_pane = TerminalPane::new(
        pid,
        fake_win_size,
        style,
        0,
        String::new(),
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        None,
        None,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    ); // 0 是窗格索引
    let content = read_fixture();
    terminal_pane.handle_pty_bytes(content);
    terminal_pane
}

#[test]
pub fn searching_inside_a_viewport() {
    let mut terminal_pane = create_pane();
    terminal_pane.update_search_term("tortor");
    assert_snapshot!(
        "grid_copy_tortor_highlighted",
        format!("{:?}", terminal_pane.grid)
    );
    terminal_pane.search_up();
    // 快照大小优化：我们在这里使用命名快照来去重
    assert_snapshot!(
        "grid_copy_search_cursor_at_bottom",
        format!("{:?}", terminal_pane.grid)
    );
    terminal_pane.search_up();
    assert_snapshot!(
        "grid_copy_search_cursor_at_second",
        format!("{:?}", terminal_pane.grid)
    );
}

#[test]
pub fn searching_scroll_viewport() {
    let mut terminal_pane = create_pane();
    terminal_pane.update_search_term("tortor");
    terminal_pane.search_up();
    // 快照大小优化：我们在这里使用命名快照来去重
    assert_snapshot!(
        "grid_copy_search_cursor_at_bottom",
        format!("{:?}", terminal_pane.grid)
    );
    terminal_pane.search_up();
    assert_snapshot!(
        "grid_copy_search_cursor_at_second",
        format!("{:?}", terminal_pane.grid)
    );
    // 滚动离开
    terminal_pane.search_up();
    assert_snapshot!(
        "grid_copy_search_scrolled_up",
        format!("{:?}", terminal_pane.grid)
    );
}

#[test]
pub fn searching_with_wrap() {
    let mut terminal_pane = create_pane();
    // 搜索 "tortor"
    terminal_pane.update_search_term("tortor");
    // 选择 tortor 最后一次出现的位置
    terminal_pane.search_up();
    assert_snapshot!(
        "grid_copy_search_cursor_at_bottom",
        format!("{:?}", terminal_pane.grid)
    );
    // 再次向后搜索
    terminal_pane.search_up();
    assert_snapshot!(
        "grid_copy_search_cursor_at_second",
        format!("{:?}", terminal_pane.grid)
    );
    terminal_pane.search_down();
    assert_snapshot!(
        "grid_copy_search_cursor_at_bottom",
        format!("{:?}", terminal_pane.grid)
    );
    // 再次向前搜索在这里应该没有效果
    terminal_pane.search_down();
    assert_snapshot!(
        "grid_copy_search_cursor_at_bottom",
        format!("{:?}", terminal_pane.grid)
    );
    // 只有在循环搜索激活后，我们才会实际在回滚缓冲区中跳转
    terminal_pane.toggle_search_wrap();
    terminal_pane.search_down();
    assert_snapshot!(
        "grid_copy_search_cursor_at_top",
        format!("{:?}", terminal_pane.grid)
    );

    // 再次停用循环
    terminal_pane.toggle_search_wrap();
    // 应该再次是空操作
    terminal_pane.search_up();
    assert_snapshot!(
        "grid_copy_search_cursor_at_top",
        format!("{:?}", terminal_pane.grid)
    );

    // 再次激活循环
    terminal_pane.toggle_search_wrap();
    terminal_pane.search_up();
    assert_snapshot!(
        "grid_copy_search_cursor_at_bottom",
        format!("{:?}", terminal_pane.grid)
    );
}

#[test]
pub fn searching_case_insensitive() {
    let mut terminal_pane = create_pane();
    terminal_pane.update_search_term("quam");
    assert_snapshot!(
        "grid_copy_quam_highlighted",
        format!("{:?}", terminal_pane.grid)
    );

    // 关闭大小写敏感
    terminal_pane.toggle_search_case_sensitivity();

    assert_snapshot!(
        "grid_copy_quam_insensitive_highlighted",
        format!("{:?}", terminal_pane.grid)
    );

    // 开启大小写敏感
    terminal_pane.toggle_search_case_sensitivity();

    assert_snapshot!(
        "grid_copy_quam_highlighted",
        format!("{:?}", terminal_pane.grid)
    );

    // 选择一个并检查我们保持当前选中，
    // 如果它不是消失的那个
    terminal_pane.search_up();
    assert_snapshot!(
        "grid_copy_quam_highlighted_cursor_bottom",
        format!("{:?}", terminal_pane.grid)
    );

    // 关闭大小写敏感
    terminal_pane.toggle_search_case_sensitivity();

    assert_snapshot!(
        "grid_copy_quam_insensitive_cursor_bottom",
        format!("{:?}", terminal_pane.grid)
    );

    // 开启大小写敏感
    terminal_pane.toggle_search_case_sensitivity();

    assert_snapshot!(
        "grid_copy_quam_highlighted_cursor_bottom",
        format!("{:?}", terminal_pane.grid)
    );

    // 关闭大小写敏感
    terminal_pane.toggle_search_case_sensitivity();

    // 选择不区分大小写的结果
    terminal_pane.search_up();
    terminal_pane.search_up();
    terminal_pane.search_up();
    terminal_pane.search_up();
    terminal_pane.search_up();
    assert_snapshot!(
        "grid_copy_quam_insensitive_selection",
        format!("{:?}", terminal_pane.grid)
    );

    // 开启大小写敏感
    terminal_pane.toggle_search_case_sensitivity();
    // 现在选中的结果消失了，我们应该回到
    // 开头
    assert_snapshot!(
        "grid_copy_quam_highlighted",
        format!("{:?}", terminal_pane.grid)
    );
    terminal_pane.search_up();
    assert_snapshot!(
        "grid_copy_quam_highlighted_cursor_bottom",
        format!("{:?}", terminal_pane.grid)
    );
}

#[test]
pub fn searching_inside_and_scroll() {
    let fake_client_id = 1;
    let mut terminal_pane = create_pane();
    terminal_pane.update_search_term("quam");
    terminal_pane.search_up();
    assert_snapshot!(
        "grid_copy_quam_highlighted_cursor_bottom",
        format!("{:?}", terminal_pane.grid)
    );
    assert_eq!(
        terminal_pane.grid.search_results.active.as_ref(),
        terminal_pane.grid.search_results.selections.last()
    );
    // 向上滚动直到出现新的搜索结果
    terminal_pane.scroll_up(4, fake_client_id);

    // 向下滚动回应该给出与之前相同的结果
    terminal_pane.scroll_down(4, fake_client_id);
    assert_eq!(
        terminal_pane.grid.search_results.active.as_ref(),
        terminal_pane.grid.search_results.selections.last()
    );
    assert_snapshot!(
        "grid_copy_quam_highlighted_cursor_bottom",
        format!("{:?}", terminal_pane.grid)
    );

    // 向上滚动直到活动标记移出视图
    terminal_pane.scroll_up(5, fake_client_id);
    assert_eq!(terminal_pane.grid.search_results.active, None);

    terminal_pane.scroll_down(5, fake_client_id);
    assert_eq!(terminal_pane.grid.search_results.active, None);
    assert_snapshot!(
        "grid_copy_quam_highlighted",
        format!("{:?}", terminal_pane.grid)
    );
}

#[test]
pub fn searching_and_resize() {
    let mut terminal_pane = create_pane();
    terminal_pane.update_search_term("tortor");
    assert_snapshot!(
        "grid_copy_tortor_highlighted",
        format!("{:?}", terminal_pane.grid)
    );

    // 如果窗格调整大小，高亮应该仍然存在
    terminal_pane.grid.change_size(20, 150);
    assert_snapshot!(
        "grid_copy_tortor_highlighted_wide",
        format!("{:?}", terminal_pane.grid)
    );

    terminal_pane.grid.change_size(20, 80);
    assert_snapshot!(
        "grid_copy_tortor_highlighted_narrow",
        format!("{:?}", terminal_pane.grid)
    );
}

#[test]
pub fn searching_across_line_wrap() {
    let mut terminal_pane = create_pane();
    terminal_pane.update_search_term("aliquam sem fringilla");
    // 分布在两行
    terminal_pane.grid.change_size(30, 60);
    assert_snapshot!(
        "grid_copy_multiline_highlighted",
        format!("{:?}", terminal_pane.grid)
    );

    // 分布在 4 行
    terminal_pane.grid.change_size(40, 4);
    assert_snapshot!(
        "grid_copy_multiline_highlighted_narrow",
        format!("{:?}", terminal_pane.grid)
    );

    terminal_pane.search_up();
    assert_snapshot!(
        "grid_copy_multiline_selected_narrow",
        format!("{:?}", terminal_pane.grid)
    );

    // 开启循环
    terminal_pane.toggle_search_wrap();
    terminal_pane.search_down();
    assert_snapshot!(
        "grid_copy_multiline_selected_wrap_narrow",
        format!("{:?}", terminal_pane.grid)
    );

    // 关闭循环
    terminal_pane.toggle_search_wrap();
    // 不要忘记当前选中
    terminal_pane.search_up();
    assert_snapshot!(
        "grid_copy_multiline_selected_wrap_narrow",
        format!("{:?}", terminal_pane.grid)
    );

    // 开启循环
    terminal_pane.toggle_search_wrap();
    terminal_pane.search_up();
    assert_snapshot!(
        "grid_copy_multiline_selected_narrow",
        format!("{:?}", terminal_pane.grid)
    );
}

#[test]
pub fn searching_whole_word() {
    let mut terminal_pane = create_pane();
    terminal_pane.update_search_term("quam");
    assert_snapshot!(
        "grid_copy_quam_highlighted",
        format!("{:?}", terminal_pane.grid)
    );

    terminal_pane.toggle_search_whole_words();
    assert_snapshot!(
        "grid_copy_quam_whole_word_only",
        format!("{:?}", terminal_pane.grid)
    );

    terminal_pane.toggle_search_whole_words();
    assert_snapshot!(
        "grid_copy_quam_highlighted",
        format!("{:?}", terminal_pane.grid)
    );
}

#[test]
pub fn searching_whole_word_across_line_wrap() {
    let mut terminal_pane = create_pane();
    terminal_pane.handle_pty_bytes(
        "a:--:aaaaaaaaa:--:--:--:aaaaaaaaaaa:--: :--: :--: aaa :--::--: aaa"
            .as_bytes()
            .to_vec(),
    );
    terminal_pane.grid.change_size(20, 5);
    terminal_pane.update_search_term(":--:");
    assert_snapshot!(
        "grid_copy_multiline_not_whole_word",
        format!("{:?}", terminal_pane.grid)
    );

    terminal_pane.toggle_search_whole_words();
    assert_snapshot!(
        "grid_copy_multiline_whole_word",
        format!("{:?}", terminal_pane.grid)
    );
}

#[test]
pub fn searching_whole_word_case_insensitive() {
    let mut terminal_pane = create_pane();
    terminal_pane.update_search_term("quam");
    assert_snapshot!(
        "grid_copy_quam_highlighted",
        format!("{:?}", terminal_pane.grid)
    );

    terminal_pane.toggle_search_whole_words();
    assert_snapshot!(
        "grid_copy_quam_whole_word_only",
        format!("{:?}", terminal_pane.grid)
    );

    terminal_pane.toggle_search_case_sensitivity();
    assert_snapshot!(
        "grid_copy_quam_whole_word_case_insensitive",
        format!("{:?}", terminal_pane.grid)
    );

    terminal_pane.toggle_search_whole_words();
    assert_snapshot!(
        "grid_copy_quam_insensitive_highlighted",
        format!("{:?}", terminal_pane.grid)
    );

    terminal_pane.toggle_search_case_sensitivity();
    assert_snapshot!(
        "grid_copy_quam_highlighted",
        format!("{:?}", terminal_pane.grid)
    );
}
