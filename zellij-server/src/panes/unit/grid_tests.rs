use super::super::Grid;
use crate::panes::grid::SixelImageStore;
use crate::panes::kitty_graphics::KittyImageStore;
use crate::panes::link_handler::LinkHandler;
use base64::engine::general_purpose::STANDARD as BASE64_ENCODER;
use base64::engine::Engine as _;
use insta::assert_snapshot;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use vte;
use zellij_utils::consts::SCROLL_BUFFER_SIZE;
use zellij_utils::{
    data::{Palette, Style},
    input::options::DEFAULT_WORD_SEPARATORS,
    pane_size::SizeInPixels,
    position::Position,
};

use std::fmt::Write;

fn read_fixture(fixture_name: &str) -> Vec<u8> {
    let mut path_to_file = std::path::PathBuf::new();
    path_to_file.push("../src");
    path_to_file.push("tests");
    path_to_file.push("fixtures");
    path_to_file.push(fixture_name);
    std::fs::read(path_to_file)
        .unwrap_or_else(|_| panic!("could not read fixture {:?}", &fixture_name))
}

#[test]
fn vttest1_0() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest1-0";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest1_1() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest1-1";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest1_2() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest1-2";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest1_3() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest1-3";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest1_4() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest1-4";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest1_5() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest1-5";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest2_0() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest2-0";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest2_1() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest2-1";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest2_2() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest2-2";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest2_3() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest2-3";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest2_4() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest2-4";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest2_5() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest2-5";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest2_6() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest2-6";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest2_7() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest2-7";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest2_8() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest2-8";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest2_9() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest2-9";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest2_10() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest2-10";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest2_11() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest2-11";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest2_12() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest2-12";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest2_13() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest2-13";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest2_14() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest2-14";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest3_0() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest3-0";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest8_0() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest8-0";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest8_1() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest8-1";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest8_2() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest8-2";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest8_3() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest8-3";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest8_4() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest8-4";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn vttest8_5() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vttest8-5";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn csi_b() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "csi-b";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn csi_capital_i() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "csi-capital-i";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn csi_capital_z() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "csi-capital-z";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn terminal_reports() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "terminal_reports";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid.pending_messages_to_pty));
}

#[test]
fn wide_characters() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        104,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "wide_characters";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn wide_characters_line_wrap() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        104,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "wide_characters_line_wrap";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn insert_character_in_line_with_wide_character() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        104,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "wide_characters_middle_line_insert";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn delete_char_in_middle_of_line_with_widechar() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        104,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "wide-chars-delete-middle";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn delete_char_in_middle_of_line_with_multiple_widechars() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        104,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "wide-chars-delete-middle-after-multi";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn fish_wide_characters_override_clock() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        104,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "fish_wide_characters_override_clock";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn bash_delete_wide_characters() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        104,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "bash_delete_wide_characters";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn delete_wide_characters_before_cursor() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        104,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "delete_wide_characters_before_cursor";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn delete_wide_characters_before_cursor_when_cursor_is_on_wide_character() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        104,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "delete_wide_characters_before_cursor_when_cursor_is_on_wide_character";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn delete_wide_character_under_cursor() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        104,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "delete_wide_character_under_cursor";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn replace_wide_character_under_cursor() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        104,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "replace_wide_character_under_cursor";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn wrap_wide_characters() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        90,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "wide_characters_full";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn wrap_wide_characters_on_size_change() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        93,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "wide_characters_full";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    grid.change_size(21, 90);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn unwrap_wide_characters_on_size_change() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        93,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "wide_characters_full";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    grid.change_size(21, 90);
    grid.change_size(21, 93);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn wrap_wide_characters_in_the_middle_of_the_line() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        91,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "wide_characters_line_middle";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn wrap_wide_characters_at_the_end_of_the_line() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        90,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "wide_characters_line_end";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn copy_selected_text_from_viewport() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        27,
        125,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "grid_copy";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);

    grid.start_selection(&Position::new(23, 6));
    // 检查宽字符，📦 占据第 34、35 列，即使只选中第一列也会被选中
    grid.end_selection(&Position::new(25, 35));
    let text = grid.get_selected_text();
    assert_eq!(
        text.unwrap(),
        "mauris in aliquam sem fringilla.\n\nzellij on  mouse-support [?] is 📦"
    );
}

#[test]
fn copy_wrapped_selected_text_from_viewport() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        22,
        73,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "grid_copy_wrapped";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);

    grid.start_selection(&Position::new(5, 0));
    grid.end_selection(&Position::new(8, 42));
    let text = grid.get_selected_text();
    assert_eq!(
        text.unwrap(),
        "Lorem ipsum dolor sit amet,                                                                                                                          consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua."
    );
}

#[test]
fn copy_selected_text_from_lines_above() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        27,
        125,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "grid_copy";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);

    grid.start_selection(&Position::new(-2, 10));
    // 检查宽字符，📦 占据第 34、35 列，即使只选中第一列也会被选中
    grid.end_selection(&Position::new(2, 8));
    let text = grid.get_selected_text();
    assert_eq!(
        text.unwrap(),
        "eu scelerisque felis imperdiet proin fermentum leo.\nCursus risus at ultrices mi tempus.\nLaoreet id donec ultrices tincidunt arcu non sodales.\nAmet dictum sit amet justo donec enim.\nHac habi"
    );
}

#[test]
fn copy_selected_text_from_lines_below() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        27,
        125,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "grid_copy";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);

    grid.move_viewport_up(40);

    grid.start_selection(&Position::new(63, 6));
    // 检查宽字符，📦 占据第 34、35 列，即使只选中第一列也会被选中
    grid.end_selection(&Position::new(65, 35));
    let text = grid.get_selected_text();
    assert_eq!(
        text.unwrap(),
        "mauris in aliquam sem fringilla.\n\nzellij on  mouse-support [?] is 📦"
    );
}

/*
 * 以下测试是针对终端中运行的非平凡场景的通用兼容性测试。
 * 它们使用从这些场景复制的伪 TTY 输入。
 *
 */

#[test]
fn run_bandwhich_from_fish_shell() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        116,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "fish_and_bandwhich";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn fish_tab_completion_options() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        116,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "fish_tab_completion_options";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn fish_select_tab_completion_options() {
    // 此测试与上一个测试的区别在于，这里我们按了 <TAB>
    // 两次，这意味着选择在选项之间移动，命令行也随之
    // 变化。
    // 这在快照中不太明显，因为快照不包含样式，
    // 但我们可以看到命令行变化且光标保持在原位
    // terminal_emulator_color_codes,
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        116,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "fish_select_tab_completion_options";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn vim_scroll_region_down() {
    // 这里我们测试 vim 将滚动区域定义为小于屏幕行数的情况
    // 然后向下滚动
    // 这里 vim 将区域定义为 1-26（共 28 行）
    // 然后光标移动到第 26 行并添加新行
    // 应该发生的是滚动区域中的第一行（1）被删除
    // terminal_emulator_color_codes,
    // 并在滚动区域的最后一行（26）插入空行
    // 此测试之后还有其他步骤，用下一行填充该行
    // sixel_image_store,
    // 文件
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        116,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vim_scroll_region_down";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn vim_ctrl_d() {
    // 在 vim 中 ctrl-d 向下移动半页
    // 在这种情况下，它向终端发送 csi 'M' 指令，告诉它在滚动区域内删除 X（本例中为 13）
    // 行并将其他行上推
    // 这里发生的是删除了 13 行，取而代之的是在滚动区域末尾添加了 13 个空行
    // 滚动区域末尾
    // terminal_emulator_color_codes,
    // vim 确保用文件的其余部分填充这些空行
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        116,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vim_ctrl_d";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn vim_ctrl_u() {
    // 在 vim 中 ctrl-u 向上移动半页
    // 在这种情况下，它向终端发送 csi 'L' 指令，告诉它在光标处插入 X（本例中为 13）
    // 行，推开（删除）滚动区域中的最后一行
    // 这会产生向上滚动 X 行的效果（vim 用当前内容上方
    // 文件中的行替换这些行）
    // terminal_emulator_color_codes,
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        116,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vim_ctrl_u";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn htop() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        116,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "htop";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn htop_scrolling() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        116,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "htop_scrolling";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn htop_right_scrolling() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        116,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "htop_right_scrolling";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn vim_overwrite() {
    // 此测试测试 vim 覆盖消息
    // 复现步骤：
    // * 在 vim 中打开一个文件
    // * 在另一个窗口中打开同一个文件
    // * 在另一个窗口中修改文件并保存
    // terminal_emulator_color_codes,
    // * 在原始 vim 窗口中修改文件并保存
    // * 按 'y' 然后按 ENTER 确认你想要修改文件
    // sixel_image_store,
    // * 如果一切正常，此测试通过 :)
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        116,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "vim_overwrite";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn clear_scroll_region() {
    // 这实际上是对 1049h/l（备用缓冲区）的测试
    // @imsnif - 这个名字是为了纪念我当时没有完全理解这个机制的时光 :)
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        116,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "clear_scroll_region";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn display_tab_characters_properly() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        116,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "tab_characters";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn neovim_insert_mode() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        116,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "nvim_insert";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn bash_cursor_linewrap() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        116,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "bash_cursor_linewrap";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn fish_paste_multiline() {
    // 这里我们在 fish shell 中粘贴多行命令，确保我们支持它
    // 向上移动并更改我们换行粘贴文本的颜色
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        149,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "fish_paste_multiline";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn git_log() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        149,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "git_log";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn git_diff_scrollup() {
    // 此测试确保当我们有一个超出屏幕大小的 git diff 时
    // 我们能够向上滚动
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        28,
        149,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "git_diff_scrollup";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn emacs_longbuf() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        60,
        284,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "emacs_longbuf_tutorial";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn top_and_quit() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        56,
        235,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "top_and_quit";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn exa_plus_omf_theme() {
    // 此测试确保我们正确处理制表符分隔的表格
    // 而不覆盖之前的内容
    // 这是一个潜在的 bug，因为 \t 字符是一个跳转
    // 如果我们将其原样转发给终端，我们将会跳过
    // 屏幕上已有的内容而不删除它，所以我们必须
    // terminal_emulator_color_codes,
    // 将其转换为空格
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        56,
        235,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "exa_plus_omf_theme";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn scroll_up() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        10,
        50,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "scrolling";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    grid.scroll_up_one_line();
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn scroll_down() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        10,
        50,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "scrolling";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    grid.scroll_up_one_line();
    grid.scroll_down_one_line();
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn scroll_up_with_line_wraps() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        10,
        25,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "scrolling";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    grid.scroll_up_one_line();
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn scroll_down_with_line_wraps() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        10,
        25,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "scrolling";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    grid.scroll_up_one_line();
    grid.scroll_down_one_line();
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn scroll_up_decrease_width_and_scroll_down() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        10,
        50,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "scrolling";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    for _ in 0..10 {
        grid.scroll_up_one_line();
    }
    grid.change_size(10, 25);
    for _ in 0..10 {
        grid.scroll_down_one_line();
    }
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn scroll_up_increase_width_and_scroll_down() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        10,
        25,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "scrolling";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    for _ in 0..10 {
        grid.scroll_up_one_line();
    }
    grid.change_size(10, 50);
    for _ in 0..10 {
        grid.scroll_down_one_line();
    }
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn saved_cursor_across_resize() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        4,
        20,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let mut parse = |s, grid: &mut Grid| {
        vte_parser.advance(&mut *grid, &Vec::from(s));
    };
    let content = "
\rLine 1 >fill to 20_<
\rLine 2 >fill to 20_<
\rLine 3 >fill to 20_<
\rL\u{1b}[sine 4 >fill to 20_<";
    parse(content, &mut grid);
    // 将真实光标位置上移三行
    let content = "\u{1b}[3A";
    parse(content, &mut grid);
    // 截断终端顶部，重置光标（但不重置保存的光标）
    grid.change_size(3, 20);
    // 换行，再次重置光标（但不重置保存的光标）
    grid.change_size(3, 10);
    // 恢复保存的光标位置并写入 ZZZ
    let content = "\u{1b}[uZZZ";
    parse(content, &mut grid);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn saved_cursor_across_resize_longline() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        4,
        20,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let mut parse = |s, grid: &mut Grid| {
        vte_parser.advance(&mut *grid, &Vec::from(s));
    };
    let content = "
\rLine 1 >fill \u{1b}[sto 20_<";
    parse(content, &mut grid);
    // 每行精确换行到一半
    grid.change_size(4, 10);
    // 在末尾写入 'YY'（最终出现在新的换行行上），恢复到保存的光标
    // 并用 'ZZ' 覆盖 'to'
    let content = "YY\u{1b}[uZZ";
    parse(content, &mut grid);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn saved_cursor_across_resize_rewrap() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        4,
        4 * 8,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let mut parse = |s, grid: &mut Grid| {
        vte_parser.advance(&mut *grid, &Vec::from(s));
    };
    let content = "
\r12345678123456781234567\u{1b}[s812345678"; // 4*8 个字符
    parse(content, &mut grid);
    // 每行精确换行到一半，然后再次换行将它们减半
    grid.change_size(4, 16);
    grid.change_size(4, 8);
    // 在第 3 行末尾写入 'Z'
    let content = "\u{1b}[uZ";
    parse(content, &mut grid);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn move_cursor_below_scroll_region() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        34,
        114,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "move_cursor_below_scroll_region";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn insert_wide_characters_in_existing_line() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        21,
        86,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "chinese_characters_line_middle";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn full_screen_scroll_region_and_scroll_up() {
    // 此测试是针对一个 bug 的回归测试
    // 该 bug 中滚动区域会被设置为
    // 完整视口，然后向上滚动会导致
    // 行从视口中被删除，而不是
    // 移动到 "lines_above"
    // terminal_emulator_color_codes,
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        54,
        80,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "scroll_region_full_screen";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    grid.scroll_up_one_line();
    grid.scroll_up_one_line();
    grid.scroll_up_one_line();
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn ring_bell() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        134,
        64,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "ring_bell";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert!(grid.ring_bell);
}

#[test]
pub fn alternate_screen_change_size() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        20,
        20,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "alternate_screen_change_size";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    // 备用屏幕中没有回滚缓冲区
    assert_eq!(grid.scrollback_position_and_length(), (0, 0));
    grid.change_size(10, 10);
    assert_snapshot!(format!("{:?}", grid));
    assert_eq!(grid.scrollback_position_and_length(), (0, 0))
}

#[test]
pub fn fzf_fullscreen() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        112,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "fzf_fullscreen";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn replace_multiple_wide_characters_under_cursor() {
    // 此测试确保如果我们用非宽字符替换宽字符，它会
    // 在正确的位置正确填充多余的宽度（如果光标在宽字符的"中间"，则在被替换的非宽
    // 字符之前填充；如果光标在宽字符的"开头"，则在字符
    // 之后填充）
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        112,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "replace_multiple_wide_characters";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn replace_non_wide_characters_with_wide_characters() {
    // 此测试确保如果我们用非宽字符替换宽字符，它会
    // 在正确的位置正确填充多余的宽度（如果光标在宽字符的"中间"，则在被替换的非宽
    // 字符之前填充；如果光标在宽字符的"开头"，则在字符
    // 之后填充）
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        112,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "replace_non_wide_characters_with_wide_characters";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn scroll_down_ansi() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        112,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "scroll_down";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn ansi_capital_t() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        112,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let content = "foo\u{1b}[14Tbar".as_bytes();
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn ansi_capital_s() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        112,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let content = "\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\nfoo\u{1b}[14Sbar".as_bytes();
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn terminal_pixel_size_reports() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(Some(SizeInPixels {
            height: 21,
            width: 8,
        }))),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        osc8_hyperlinks,
        debug,
        arrow_fonts,
        styled_underlines,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "terminal_pixel_size_reports";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    // CSI 14t 和 CSI 16t 被转发给主机；Zellij 不再
    // 为这些指令从 character_cell_size 生成本地回复。
    assert!(grid.pending_messages_to_pty.is_empty());
    use crate::host_query::HostQuery;
    assert_eq!(
        grid.pending_forwarded_queries,
        vec![
            HostQuery::TextAreaPixelSize,
            HostQuery::CharacterCellPixelSize
        ],
    );
}

#[test]
fn terminal_pixel_size_reports_in_unsupported_terminals() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)), // 在不支持的终端中，我们没有此信息
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "terminal_pixel_size_reports";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    // 转发与 character_cell_size 是否可用无关 ——
    // 无论 Zellij 本地知道什么，主机终端对这些查询都是权威的
    // 。
    assert!(grid.pending_messages_to_pty.is_empty());
    use crate::host_query::HostQuery;
    assert_eq!(
        grid.pending_forwarded_queries,
        vec![
            HostQuery::TextAreaPixelSize,
            HostQuery::CharacterCellPixelSize
        ],
    );
}

#[test]
pub fn ansi_csi_at_sign() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        112,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let content = "foo\u{1b}[2D\u{1b}[2@".as_bytes();
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn sixel_images_are_reaped_when_scrolled_off() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let character_cell_size = Rc::new(RefCell::new(Some(SizeInPixels {
        width: 8,
        height: 21,
    })));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        112,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        character_cell_size,
        sixel_image_store.clone(),
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let pane_content = read_fixture("sixel-image-500px.six");
    vte_parser.advance(&mut grid, &pane_content);
    for _ in 0..10_051 {
        // 回滚缓冲区限制 + 视口高度
        grid.add_canonical_line();
    }
    let _ = grid.read_changes(0, 0); // 我们这样做是因为这里是回收图像的地方
    assert_eq!(
        sixel_image_store.borrow().image_count(),
        0,
        "all images were deleted from the store"
    );
}

#[test]
pub fn sixel_images_are_reaped_when_resetting() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let character_cell_size = Rc::new(RefCell::new(Some(SizeInPixels {
        width: 8,
        height: 21,
    })));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        112,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        character_cell_size,
        sixel_image_store.clone(),
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let pane_content = read_fixture("sixel-image-500px.six");
    vte_parser.advance(&mut grid, &pane_content);
    grid.reset_terminal_state();
    let _ = grid.read_changes(0, 0); // 我们这样做是因为这里是回收图像的地方
    assert_eq!(
        sixel_image_store.borrow().image_count(),
        0,
        "all images were deleted from the store"
    );
}

#[test]
pub fn sixel_image_in_alternate_buffer() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let character_cell_size = Rc::new(RefCell::new(Some(SizeInPixels {
        width: 8,
        height: 21,
    })));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        30,
        112,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        character_cell_size,
        sixel_image_store.clone(),
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );

    let move_to_alternate_screen = "\u{1b}[?1049h";
    vte_parser.advance(&mut grid, move_to_alternate_screen.as_bytes());

    let pane_content = read_fixture("sixel-image-500px.six");
    vte_parser.advance(&mut grid, &pane_content);
    assert_snapshot!(format!("{:?}", grid)); // 应包含图像
                                             //
    let move_away_from_alternate_screen = "\u{1b}[?1049l";
    vte_parser.advance(&mut grid, move_away_from_alternate_screen.as_bytes());
    assert_snapshot!(format!("{:?}", grid)); // 应不包含图像
    assert_eq!(
        sixel_image_store.borrow().image_count(),
        0,
        "all images were deleted from the store when we moved back from alternate screen"
    );
}

#[test]
pub fn sixel_with_image_scrolling_decsdm() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let character_cell_size = Rc::new(RefCell::new(Some(SizeInPixels {
        width: 8,
        height: 21,
    })));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        30,
        112,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        character_cell_size,
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );

    // 进入 DECSDM
    let move_to_decsdm = "\u{1b}[?80h";
    vte_parser.advance(&mut grid, move_to_decsdm.as_bytes());

    // 写入一些文本
    let mut text_to_fill_pane = String::new();
    for i in 0..10 {
        writeln!(&mut text_to_fill_pane, "\rline {}", i + 1).unwrap();
    }
    vte_parser.advance(&mut grid, text_to_fill_pane.as_bytes());

    // 渲染一个 sixel 图像（将出现在左上角并部分覆盖文本）
    let pane_content = read_fixture("sixel-image-100px.six");
    vte_parser.advance(&mut grid, &pane_content);
    // 图像应位于网格的左上角
    assert_snapshot!(format!("{:?}", grid));

    // 离开 DECSDM
    let move_away_from_decsdm = "\u{1b}[?80l";
    vte_parser.advance(&mut grid, move_away_from_decsdm.as_bytes());

    // 向下移动到下一行的开头
    let mut go_down_once = String::new();
    writeln!(&mut go_down_once, "\n\r").unwrap();
    vte_parser.advance(&mut grid, go_down_once.as_bytes());

    // 渲染另一个 sixel 图像，应出现在光标下方
    let pane_content = read_fixture("sixel-image-100px.six");
    vte_parser.advance(&mut grid, &pane_content);

    // 图像应出现在光标位置
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
pub fn osc_4_background_query() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let content = "\u{1b}]10;?\u{1b}\\";
    vte_parser.advance(&mut grid, content.as_bytes());
    // 重构后：OSC 10;? 被转发给主机，而不是由本地应答
    // 从 Zellij 的缓存调色板中应答。pending_messages_to_pty 必须保持
    // 为空。
    assert!(grid.pending_messages_to_pty.is_empty());
    let forwarded_string: String = grid
        .pending_forwarded_queries
        .iter()
        .map(|q| String::from_utf8(q.to_query_bytes()).unwrap())
        .collect();
    assert_eq!(forwarded_string, "\u{1b}]10;?\u{1b}\\");
}

#[test]
pub fn osc_4_foreground_query() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let content = "\u{1b}]11;?\u{1b}\\";
    vte_parser.advance(&mut grid, content.as_bytes());
    assert!(grid.pending_messages_to_pty.is_empty());
    let forwarded_string: String = grid
        .pending_forwarded_queries
        .iter()
        .map(|q| String::from_utf8(q.to_query_bytes()).unwrap())
        .collect();
    assert_eq!(forwarded_string, "\u{1b}]11;?\u{1b}\\");
}

#[test]
pub fn osc_4_color_query() {
    let mut color_codes = HashMap::new();
    color_codes.insert(222, String::from("rgb:ffff/d7d7/8787"));
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(color_codes));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let content = "\u{1b}]4;222;?\u{1b}\\";
    vte_parser.advance(&mut grid, content.as_bytes());
    // OSC 4;N;? 被转发给主机以获取真实的调色板值。
    assert!(grid.pending_messages_to_pty.is_empty());
    let forwarded_string: String = grid
        .pending_forwarded_queries
        .iter()
        .map(|q| String::from_utf8(q.to_query_bytes()).unwrap())
        .collect();
    assert_eq!(forwarded_string, "\u{1b}]4;222;?\u{1b}\\");
}

#[test]
pub fn xtsmgraphics_color_register_count() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let content = "\u{1b}[?1;1;S\u{1b}\\";
    vte_parser.advance(&mut grid, content.as_bytes());
    let message_string = grid
        .pending_messages_to_pty
        .iter()
        .map(|m| String::from_utf8_lossy(m))
        .fold(String::new(), |mut acc, s| {
            acc.push_str(&s);
            acc
        });
    assert_eq!(message_string, "\u{1b}[?1;0;65536S");
}

#[test]
pub fn xtsmgraphics_pixel_graphics_geometry() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let character_cell_size = Rc::new(RefCell::new(Some(SizeInPixels {
        width: 8,
        height: 21,
    })));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        51,
        97,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        character_cell_size,
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let content = "\u{1b}[?2;1;S\u{1b}\\";
    vte_parser.advance(&mut grid, content.as_bytes());
    let message_string = grid
        .pending_messages_to_pty
        .iter()
        .map(|m| String::from_utf8_lossy(m))
        .fold(String::new(), |mut acc, s| {
            acc.push_str(&s);
            acc
        });
    assert_eq!(message_string, "\u{1b}[?2;0;776;1071S");
}

#[test]
pub fn cursor_hide_persists_through_alternate_screen() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let character_cell_size = Rc::new(RefCell::new(Some(SizeInPixels {
        width: 8,
        height: 21,
    })));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        30,
        112,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        character_cell_size,
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );

    let hide_cursor = "\u{1b}[?25l";
    vte_parser.advance(&mut grid, hide_cursor.as_bytes());
    assert!(
        matches!(grid.cursor_coordinates(), Some((_, _, false))),
        "Cursor hidden properly"
    );

    let move_to_alternate_screen = "\u{1b}[?1049h";
    vte_parser.advance(&mut grid, move_to_alternate_screen.as_bytes());
    assert!(
        matches!(grid.cursor_coordinates(), Some((_, _, false))),
        "Cursor still hidden in alternate screen"
    );

    let show_cursor = "\u{1b}[?25h";
    vte_parser.advance(&mut grid, show_cursor.as_bytes());
    assert!(
        matches!(grid.cursor_coordinates(), Some((_, _, true))),
        "Cursor shown"
    );

    let move_away_from_alternate_screen = "\u{1b}[?1049l";
    vte_parser.advance(&mut grid, move_away_from_alternate_screen.as_bytes());
    assert!(
        matches!(grid.cursor_coordinates(), Some((_, _, true))),
        "Cursor still shown away from alternate screen"
    );
}

#[test]
fn table_ui_component() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "table-ui-component";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn table_ui_component_with_coordinates() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "table-ui-component-with-coordinates";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn ribbon_ui_component() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "ribbon-ui-component";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn ribbon_ui_component_with_coordinates() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "ribbon-ui-component-with-coordinates";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn nested_list_ui_component() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        120,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "nested-list-ui-component";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn nested_list_ui_component_with_coordinates() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        120,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "nested-list-ui-component-with-coordinates";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn text_ui_component() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        120,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "text-ui-component";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn text_ui_component_with_coordinates() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        120,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let fixture_name = "text-ui-component-with-coordinates";
    let content = read_fixture(fixture_name);
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn cannot_escape_scroll_region() {
    // 此测试测试一个 bug 的修复，该 bug 允许将滚动区域边界设置为超出
    // 窗格高度，从而允许超出滚动区域的 goto 指令逃逸
    // 窗格边界并在其他窗格上渲染内容
    //
    // 我们在这里做的是将滚动区域设置为超出终端边界（`<ESC>[1;42r` —— 而
    // 终端只有 41 行高），然后发出 goto 指令到第 42 行，即超出
    // 窗格和滚动区域边界一行（`<ESC>[42;1H`），然后打印文本 `Hi there!`。
    // 这应该打印在终端的最后一行（零索引 40）上，而不是超出它。
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        41,
        120,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );
    let content = "\u{1b}[1;42r\u{1b}[42;1HHi there!".as_bytes();
    vte_parser.advance(&mut grid, &content);
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn preserve_background_color_on_resize() {
    use crate::panes::terminal_character::{AnsiCode, EMPTY_TERMINAL_CHARACTER};

    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;
    let mut grid = Grid::new(
        10,
        20,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    );

    let mut parse = |s, grid: &mut Grid| {
        vte_parser.advance(&mut *grid, &Vec::from(s));
    };

    // 写入延伸到行尾的红色背景文本
    // ESC[41m = 红色背景
    // ESC[K = 清除到行尾（用当前背景填充）
    // ESC[0m = 重置
    let content = "test\x1b[41m\x1b[K\x1b[0m";
    parse(content, &mut grid);

    // 检查调整大小前 "test" 之后的字符是否有红色背景
    let first_row = &grid.viewport[0];
    let background_char_count_before = first_row
        .columns
        .iter()
        .enumerate()
        .filter(|(i, c)| *i >= 4 && c.styles.background != Some(AnsiCode::Reset))
        .count();
    assert!(
        background_char_count_before > 0,
        "Should have characters with background color before resize"
    );

    // 同时检查普通尾随空格是否被正确修剪（回归测试）
    let content2 = "\r\n\rplain text with spaces    ";
    parse(content2, &mut grid);

    // 调整网格大小
    grid.change_size(10, 30);

    // 检查调整大小后背景色是否保留
    let first_row = &grid.viewport[0];
    let background_char_count_after = first_row
        .columns
        .iter()
        .enumerate()
        .filter(|(i, c)| *i >= 4 && c.styles.background != Some(AnsiCode::Reset))
        .count();
    assert_eq!(
        background_char_count_before, background_char_count_after,
        "Background colored characters should be preserved after resize"
    );

    // 验证第二行没有过多的尾随空格
    //（因为它们是没有背景色的普通空格，所以应该被修剪）
    let second_row = &grid.viewport[1];
    let trailing_spaces = second_row
        .columns
        .iter()
        .rev()
        .take_while(|c| c.character == EMPTY_TERMINAL_CHARACTER.character)
        .count();
    // 所有尾随普通空格应被完全移除
    assert_eq!(
        trailing_spaces, 0,
        "Plain trailing spaces should be completely trimmed, but found {} trailing spaces",
        trailing_spaces
    );
}

fn create_grid_with_content(content: &str) -> Grid {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let mut grid = Grid::new(
        20,
        80,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        false,
        true,
        true,
        true,
        false,
    );
    vte_parser.advance(&mut grid, content.as_bytes());
    grid
}

fn select_with_click_count(grid: &mut Grid, position: Position, click_count: usize) -> String {
    for _ in 0..click_count {
        grid.start_selection(&position);
    }
    grid.end_selection(&position);
    grid.get_selected_text().unwrap()
}

fn viewport_position_of(grid: &Grid, needle: &str) -> Position {
    grid.viewport
        .iter()
        .enumerate()
        .find_map(|(line, row)| {
            let text: String = row.columns.iter().map(|c| c.character).collect();
            text.find(needle)
                .map(|column| Position::new(line as i32, column as u16))
        })
        .unwrap_or_else(|| panic!("{needle:?} not found in viewport"))
}

#[test]
fn osc133_a_b_c_d_selects_command_and_output_without_prompt() {
    let mut grid = create_grid_with_content(
        "\x1b]133;A\x07$ \x1b]133;B\x07echo hi\x1b]133;C\x07\r\nhi\x1b]133;D;0\x07 suffix",
    );

    assert_eq!(
        select_with_click_count(&mut grid, Position::new(1, 0), 3),
        "echo hi\nhi"
    );
}

#[test]
fn osc133_p_i_aliases_select_command_and_output_without_prompt() {
    let mut grid = create_grid_with_content(
        "\x1b]133;P\x07> \x1b]133;I\x07pwd\x1b]133;C\x07\r\n/tmp\x1b]133;D\x07",
    );

    let position = viewport_position_of(&grid, "/tmp");
    assert_eq!(select_with_click_count(&mut grid, position, 3), "pwd\n/tmp");
}

#[test]
fn osc133_input_marker_survives_ghostty_prompt_clear() {
    let mut grid = create_grid_with_content(
        "\x1b]133;A;cl=line\x07$ \x1b]133;B\x07\x1b[Kecho hi\r\n\x1b]133;C\x07hi\r\n\x1b]133;D;0\x07\r\x1b[J\x1b]133;A;cl=line\x07$ \x1b]133;B\x07\x1b[Knext",
    );

    assert_eq!(
        select_with_click_count(&mut grid, Position::new(1, 0), 3),
        "echo hi\nhi"
    );
}

#[test]
fn osc133_triple_click_outside_output_keeps_logical_line_selection() {
    let mut grid = create_grid_with_content(
        "\x1b]133;A\x07$ \x1b]133;B\x07echo hi\x1b]133;C\x07 output\x1b]133;D\x07",
    );

    assert_eq!(
        select_with_click_count(&mut grid, Position::new(0, 0), 3),
        "$ echo hi output"
    );
}

#[test]
fn osc133_double_click_keeps_word_selection() {
    let mut grid = create_grid_with_content(
        "\x1b]133;A\x07$ \x1b]133;B\x07echo\x1b]133;C\x07 result word\x1b]133;D\x07",
    );

    let position = viewport_position_of(&grid, "word");
    assert_eq!(select_with_click_count(&mut grid, position, 2), "word");
}

#[test]
fn osc133_wrapped_command_and_output_are_selected_together() {
    let mut grid = create_grid_with_size_and_raw(
        4,
        8,
        b"\x1b]133;A\x07$ \x1b]133;B\x07longcmd\x1b]133;C\x07-output\x1b]133;D\x07",
    );

    let position = viewport_position_of(&grid, "output");
    assert_eq!(
        select_with_click_count(&mut grid, position, 3),
        "longcmd-output"
    );
}

#[test]
fn osc133_running_command_without_end_boundary_falls_back_to_logical_line_selection() {
    let mut grid =
        create_grid_with_content("\x1b]133;A\x07$ \x1b]133;B\x07sleep\x1b]133;C\x07\r\npartial");

    let position = viewport_position_of(&grid, "partial");
    assert_eq!(select_with_click_count(&mut grid, position, 3), "partial");
}

#[test]
fn osc133_command_becomes_selectable_once_its_end_boundary_arrives() {
    let mut grid =
        create_grid_with_content("\x1b]133;A\x07$ \x1b]133;B\x07sleep\x1b]133;C\x07\r\npartial");
    let position = viewport_position_of(&grid, "partial");

    let mut vte_parser = vte::Parser::new();
    vte_parser.advance(&mut grid, b"\x1b]133;D;0\x07");

    assert_eq!(
        select_with_click_count(&mut grid, position, 3),
        "sleep\npartial"
    );
}

#[test]
fn osc133_missing_input_starts_selection_at_output_boundary() {
    let mut grid =
        create_grid_with_content("\x1b]133;A\x07$ hidden\x1b]133;C\x07visible\x1b]133;D\x07");

    let position = viewport_position_of(&grid, "visible");
    assert_eq!(select_with_click_count(&mut grid, position, 3), "visible");
}

#[test]
fn osc133_missing_end_stops_at_later_prompt() {
    let mut grid = create_grid_with_content(
        "\x1b]133;A\x07$ \x1b]133;B\x07one\x1b]133;C\x07:1\x1b]133;A\x07$ \x1b]133;B\x07two\x1b]133;C\x07:2",
    );

    assert_eq!(
        select_with_click_count(&mut grid, Position::new(0, 5), 3),
        "one:1"
    );
}

#[test]
fn osc133_multiple_commands_on_one_row_remain_distinct() {
    let content = "\x1b]133;A\x07$ \x1b]133;B\x07one\x1b]133;C\x07:1\x1b]133;D\x07\x1b]133;P\x07$ \x1b]133;I\x07two\x1b]133;C\x07:2\x1b]133;D\x07";
    let mut first = create_grid_with_content(content);
    let mut second = create_grid_with_content(content);

    assert_eq!(
        select_with_click_count(&mut first, Position::new(0, 5), 3),
        "one:1"
    );
    assert_eq!(
        select_with_click_count(&mut second, Position::new(0, 12), 3),
        "two:2"
    );
}

#[test]
fn osc133_markers_survive_scrollback() {
    let mut grid = create_grid_with_size_and_raw(
        3,
        20,
        b"\x1b]133;A\x07$ \x1b]133;B\x07cmd\x1b]133;C\x07\r\nRESULT\x1b]133;D\x07\r\ntail1\r\ntail2\r\ntail3\r\ntail4",
    );
    for _ in 0..3 {
        grid.scroll_up_one_line();
    }

    let position = viewport_position_of(&grid, "RESULT");
    assert_eq!(
        select_with_click_count(&mut grid, position, 3),
        "cmd\nRESULT"
    );
}

#[test]
fn osc133_markers_survive_resize_reflow() {
    let mut grid = create_grid_with_size_and_raw(
        5,
        20,
        b"\x1b]133;A\x07$ \x1b]133;B\x07echo hi\x1b]133;C\x07RESULT\x1b]133;D\x07",
    );
    grid.change_size(5, 6);

    let position = viewport_position_of(&grid, "RES");
    assert_eq!(
        select_with_click_count(&mut grid, position, 3),
        "echo hiRESULT"
    );
}

#[test]
fn osc133_cleared_markers_fall_back_to_logical_line_selection() {
    let mut grid = create_grid_with_content(
        "\x1b]133;A\x07$ \x1b]133;B\x07cmd\x1b]133;C\x07old\r\x1b[2Kunmarked",
    );

    assert_eq!(
        select_with_click_count(&mut grid, Position::new(0, 6), 3),
        "unmarked"
    );
}

#[test]
fn osc133_truncated_markers_fall_back_to_logical_line_selection() {
    let mut grid = create_grid_with_content(
        "\x1b]133;A\x07$ \x1b]133;B\x07cmd\x1b]133;C\x07old\r\x1b[2C\x1b[Jplain",
    );

    assert_eq!(
        select_with_click_count(&mut grid, Position::new(0, 3), 3),
        "$ plain"
    );
}

#[test]
fn osc133_overwritten_markers_fall_back_to_logical_line_selection() {
    let mut grid =
        create_grid_with_content("\x1b]133;A\x07$ \x1b]133;B\x07cmd\x1b]133;C\x07old\runmarked");

    assert_eq!(
        select_with_click_count(&mut grid, Position::new(0, 6), 3),
        "unmarked"
    );
}

#[test]
fn osc133_unknown_subcommand_is_ignored() {
    let mut grid = create_grid_with_content("plain\x1b]133;Z\x07 text");

    assert_eq!(
        select_with_click_count(&mut grid, Position::new(0, 7), 3),
        "plain text"
    );
}

#[test]
fn osc133_command_selection_disabled_falls_back_to_logical_line_selection() {
    let mut grid = create_grid_with_content(
        "\x1b]133;A\x07$ \x1b]133;B\x07echo hi\x1b]133;C\x07\r\nhi\x1b]133;D;0\x07",
    );
    grid.set_selection_options(false, DEFAULT_WORD_SEPARATORS);

    assert_eq!(
        select_with_click_count(&mut grid, Position::new(1, 0), 3),
        "hi"
    );
}

#[test]
fn osc133_command_selection_can_be_re_enabled_for_existing_markers() {
    let mut grid = create_grid_with_content(
        "\x1b]133;A\x07$ \x1b]133;B\x07echo hi\x1b]133;C\x07\r\nhi\x1b]133;D;0\x07",
    );
    grid.set_selection_options(false, DEFAULT_WORD_SEPARATORS);
    grid.set_selection_options(true, DEFAULT_WORD_SEPARATORS);

    assert_eq!(
        select_with_click_count(&mut grid, Position::new(1, 0), 3),
        "echo hi\nhi"
    );
}

#[test]
fn configured_word_separators_split_double_click_selection() {
    let mut grid = create_grid_with_content("foo:bar baz");
    grid.set_selection_options(true, ":");

    let position = viewport_position_of(&grid, "bar");
    assert_eq!(select_with_click_count(&mut grid, position, 2), "bar");
}

#[test]
fn characters_absent_from_word_separators_do_not_split_double_click_selection() {
    let mut grid = create_grid_with_content("foo:bar baz");
    grid.set_selection_options(true, DEFAULT_WORD_SEPARATORS);

    let position = viewport_position_of(&grid, "bar");
    assert_eq!(select_with_click_count(&mut grid, position, 2), "foo:bar");
}

#[test]
fn word_separators_not_configured_as_separators_are_part_of_the_word() {
    let mut grid = create_grid_with_content("foo(bar) baz");
    grid.set_selection_options(true, "");

    let position = viewport_position_of(&grid, "bar");
    assert_eq!(select_with_click_count(&mut grid, position, 2), "foo(bar)");
}

#[test]
fn default_word_separators_keep_bracket_boundaries() {
    let mut grid = create_grid_with_content("foo(bar) baz");
    grid.set_selection_options(true, DEFAULT_WORD_SEPARATORS);

    let position = viewport_position_of(&grid, "bar");
    assert_eq!(select_with_click_count(&mut grid, position, 2), "bar");
}

#[test]
fn osc133_repeated_output_markers_without_input_start_at_latest_output() {
    let mut grid = create_grid_with_content(
        "\x1b]133;A\x07$ first\x1b]133;C\x07skip\x1b]133;C\x07keep\x1b]133;D\x07",
    );

    let position = viewport_position_of(&grid, "keep");
    assert_eq!(select_with_click_count(&mut grid, position, 3), "keep");
}

#[test]
fn osc133_click_exactly_on_output_marker_column_selects_command_and_output() {
    let mut grid = create_grid_with_content(
        "\x1b]133;A\x07$ \x1b]133;B\x07cmd\x1b]133;C\x07OUT\x1b]133;D\x07",
    );

    let position = viewport_position_of(&grid, "OUT");
    assert_eq!(select_with_click_count(&mut grid, position, 3), "cmdOUT");
}

#[test]
fn osc133_backward_scan_finds_command_start_in_deep_scrollback() {
    let mut content = b"\x1b]133;A\x07$ \x1b]133;B\x07cmd\x1b]133;C\x07\r\nline00".to_vec();
    for i in 1..40 {
        content.extend_from_slice(format!("\r\nline{:02}", i).as_bytes());
    }
    content.extend_from_slice(b"\x1b]133;D\x07");
    let mut grid = create_grid_with_size_and_raw(10, 20, &content);

    let position = viewport_position_of(&grid, "line39");
    let expected = std::iter::once("cmd".to_string())
        .chain((0..40).map(|i| format!("line{:02}", i)))
        .collect::<Vec<String>>()
        .join("\n");
    assert_eq!(select_with_click_count(&mut grid, position, 3), expected);
}

#[test]
fn cursor_forward_over_unwritten_line_positions_character() {
    use crate::panes::terminal_character::AnsiCode;

    let content = "\u{1b}[1BABC \u{1b}[1B\r\u{1b}[3C\u{1b}[42mDEF\u{1b}[0m";
    let grid = create_grid_with_content(content);

    let row = &grid.viewport[2];

    assert_eq!(row.columns[0].character, ' ');
    assert_eq!(row.columns[1].character, ' ');
    assert_eq!(row.columns[2].character, ' ');
    assert_eq!(row.columns[3].character, 'D');
    assert_eq!(row.columns[4].character, 'E');
    assert_eq!(row.columns[5].character, 'F');

    let has_background = |c: &crate::panes::terminal_character::TerminalCharacter| {
        !matches!(c.styles.background, Some(AnsiCode::Reset) | None)
    };
    assert!(!has_background(&row.columns[0]));
    assert!(!has_background(&row.columns[1]));
    assert!(!has_background(&row.columns[2]));
    assert!(has_background(&row.columns[3]));
    assert!(has_background(&row.columns[4]));
    assert!(has_background(&row.columns[5]));
}

#[test]
fn double_click_selection_preserved_after_scroll() {
    let content = "line 0\nline 1\nline 2\nline 3\nline 4\nthis is a word test\nline 6\nline 7\nline 8\nline 9\nline 10\nline 11\nline 12\nline 13\nline 14\nline 15\nline 16\nline 17\nline 18\nline 19\n";
    let mut grid = create_grid_with_content(content);

    for _ in 0..20 {
        grid.add_canonical_line();
    }

    let word_position = Position::new(5, 10);
    grid.start_selection(&word_position);

    let selection_before = grid.get_selected_text();
    let word_start = grid.selection.start;
    let word_end = grid.selection.end;

    grid.end_selection(&word_position);

    grid.scroll_up_one_line();

    let selection_after_start = grid.selection.start;
    let selection_after_end = grid.selection.end;

    assert_eq!(selection_after_start.line.0, word_start.line.0 + 1);
    assert_eq!(selection_after_end.line.0, word_end.line.0 + 1);
    assert_eq!(selection_after_start.column, word_start.column);
    assert_eq!(selection_after_end.column, word_end.column);

    let text_after = grid.get_selected_text();
    assert_eq!(selection_before, text_after);
}

#[test]
fn triple_click_selection_preserved_after_scroll() {
    let content = "line 0\nline 1\nline 2\nline 3\nline 4\nthis is line five with some text\nline 6\nline 7\nline 8\nline 9\nline 10\nline 11\nline 12\nline 13\nline 14\nline 15\nline 16\nline 17\nline 18\nline 19\n";
    let mut grid = create_grid_with_content(content);

    for _ in 0..20 {
        grid.add_canonical_line();
    }

    let line_position = Position::new(5, 15);
    grid.start_selection(&line_position);
    grid.start_selection(&line_position);
    grid.start_selection(&line_position);

    let selection_before = grid.get_selected_text();
    let line_start = grid.selection.start;
    let line_end = grid.selection.end;

    grid.end_selection(&line_position);

    grid.scroll_up_one_line();

    let selection_after_start = grid.selection.start;
    let selection_after_end = grid.selection.end;

    assert_eq!(selection_after_start.line.0, line_start.line.0 + 1);
    assert_eq!(selection_after_end.line.0, line_end.line.0 + 1);
    assert_eq!(selection_after_start.column, line_start.column);
    assert_eq!(selection_after_end.column, line_end.column);

    let text_after = grid.get_selected_text();
    assert_eq!(selection_before, text_after);
}

#[test]
fn double_click_selection_moves_with_multiple_scrolls() {
    let content = "line 0\nline 1\nline 2\nline 3\nline 4\nthis is a word test\nline 6\nline 7\nline 8\nline 9\nline 10\nline 11\nline 12\nline 13\nline 14\nline 15\nline 16\nline 17\nline 18\nline 19\n";
    let mut grid = create_grid_with_content(content);

    for _ in 0..20 {
        grid.add_canonical_line();
    }

    let word_position = Position::new(5, 10);
    grid.start_selection(&word_position);

    let initial_start = grid.selection.start;
    let initial_end = grid.selection.end;

    grid.end_selection(&word_position);

    for _ in 0..5 {
        grid.scroll_up_one_line();
    }

    assert_eq!(grid.selection.start.line.0, initial_start.line.0 + 5);
    assert_eq!(grid.selection.end.line.0, initial_end.line.0 + 5);
    assert_eq!(grid.selection.start.column, initial_start.column);
    assert_eq!(grid.selection.end.column, initial_end.column);
}

#[test]
fn single_click_drag_selection_preserved_after_scroll() {
    let content = "line 0\nline 1\nline 2\nline 3\nline 4\nsome text here\nline 6\nline 7\nline 8\nline 9\nline 10\nline 11\nline 12\nline 13\nline 14\nline 15\nline 16\nline 17\nline 18\nline 19\n";
    let mut grid = create_grid_with_content(content);

    for _ in 0..20 {
        grid.add_canonical_line();
    }

    grid.start_selection(&Position::new(5, 5));
    grid.update_selection(&Position::new(5, 10));

    let start_before = grid.selection.start;
    let end_before = grid.selection.end;

    grid.end_selection(&Position::new(5, 10));

    grid.scroll_up_one_line();

    assert_eq!(grid.selection.start.line.0, start_before.line.0 + 1);
    assert_eq!(grid.selection.end.line.0, end_before.line.0 + 1);
    assert_eq!(grid.selection.start.column, start_before.column);
    assert_eq!(grid.selection.end.column, end_before.column);
}

#[test]
fn osc_11_set_and_query_pane_default_bg() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let mut grid = Grid::new(
        10,
        20,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        false,
        true,
        true,
        true,
        false,
    );

    // 通过 OSC 11 设置背景
    let set_bg = b"\x1b]11;#001a3a\x07";
    vte_parser.advance(&mut grid, set_bg);

    assert_eq!(grid.pane_default_bg, Some((0, 26, 58)));

    // 通过 OSC 11 查询背景 —— 因为窗格级覆盖已生效，
    // 查询被短路：窗格内的应用必须看到 Zellij 实际渲染的内容，
    // 而不是主机终端的背景。回复使用 xterm 的规范
    // `rgb:RRRR/GGGG/BBBB` 形式，每个 8 位通道通过
    // 重复扩展（0x00 → 0x0000，0x1a → 0x1a1a，0x3a → 0x3a3a）。
    // 重复扩展（0x00 → 0x0000，0x1a → 0x1a1a，0x3a → 0x3a3a）。
    let query_bg = b"\x1b]11;?\x07";
    vte_parser.advance(&mut grid, query_bg);

    assert!(
        grid.pending_forwarded_queries.is_empty(),
        "OSC 11 query must not be forwarded when a pane override is set"
    );
    assert_eq!(grid.pending_messages_to_pty.len(), 1);
    let reply = String::from_utf8(grid.pending_messages_to_pty[0].clone()).unwrap();
    assert_eq!(reply, "\u{1b}]11;rgb:0000/1a1a/3a3a\u{7}",);
}

#[test]
fn osc_10_set_and_query_pane_default_fg() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let mut grid = Grid::new(
        10,
        20,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        false,
        true,
        true,
        true,
        false,
    );

    // 通过 OSC 10 设置前景
    let set_fg = b"\x1b]10;#00e000\x07";
    vte_parser.advance(&mut grid, set_fg);

    assert_eq!(grid.pane_default_fg, Some((0, 224, 0)));

    // 通过 OSC 10 查询前景 —— 窗格级覆盖已生效，
    // 因此查询由本地应答（有关短路的原理，请参阅 OSC 11 等效测试）。
    //
    let query_fg = b"\x1b]10;?\x07";
    vte_parser.advance(&mut grid, query_fg);

    assert!(
        grid.pending_forwarded_queries.is_empty(),
        "OSC 10 query must not be forwarded when a pane override is set"
    );
    assert_eq!(grid.pending_messages_to_pty.len(), 1);
    let reply = String::from_utf8(grid.pending_messages_to_pty[0].clone()).unwrap();
    assert_eq!(reply, "\u{1b}]10;rgb:0000/e0e0/0000\u{7}",);
}

#[test]
fn osc_110_111_reset_pane_default_colors() {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let mut grid = Grid::new(
        10,
        20,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        false,
        true,
        true,
        true,
        false,
    );

    // 同时设置前景和背景
    let set_fg = b"\x1b]10;#00e000\x07";
    vte_parser.advance(&mut grid, set_fg);
    let set_bg = b"\x1b]11;#001a3a\x07";
    vte_parser.advance(&mut grid, set_bg);

    assert_eq!(grid.pane_default_fg, Some((0, 224, 0)));
    assert_eq!(grid.pane_default_bg, Some((0, 26, 58)));

    // 通过 OSC 110 重置前景
    let reset_fg = b"\x1b]110\x07";
    vte_parser.advance(&mut grid, reset_fg);
    assert_eq!(grid.pane_default_fg, None);
    assert_eq!(grid.pane_default_bg, Some((0, 26, 58)));

    // 通过 OSC 111 重置背景
    let reset_bg = b"\x1b]111\x07";
    vte_parser.advance(&mut grid, reset_bg);
    assert_eq!(grid.pane_default_fg, None);
    assert_eq!(grid.pane_default_bg, None);
}

#[test]
fn osc_11_set_bg_produces_ansi_in_render_output() {
    use crate::panes::terminal_character::AnsiCode;

    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let mut grid = Grid::new(
        5,
        10,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        false,
        true,
        true,
        true,
        false,
    );

    // 通过 OSC 11 设置背景
    let set_bg = b"\x1b]11;#001a3a\x07";
    vte_parser.advance(&mut grid, set_bg);

    assert_eq!(grid.pane_default_bg, Some((0, 26, 58)));

    // 渲染网格并检查窗格默认值是否印在块上
    let style = Style::default();
    let render_result = grid.render(0, 0, &style).unwrap();
    assert!(render_result.is_some(), "Expected render output");

    let (chunks, _, _, _) = render_result.unwrap();
    assert!(!chunks.is_empty(), "Expected at least one character chunk");

    // 所有块都应携带窗格默认背景
    for chunk in &chunks {
        assert_eq!(
            chunk.pane_default_bg,
            Some(AnsiCode::RgbCode((0, 26, 58))),
            "Chunk should carry pane default background"
        );
    }
}

// =====================================================================
// 插件高亮引擎测试
// =====================================================================

use crate::panes::grid::MouseTracking;
use crate::panes::terminal_character::AnsiCode;
use std::collections::BTreeMap;
use zellij_utils::data::{HighlightLayer, HighlightStyle, RegexHighlight};

fn create_highlight(
    pattern: &str,
    on_hover: bool,
    bold: bool,
    italic: bool,
    underline: bool,
    layer: HighlightLayer,
) -> RegexHighlight {
    RegexHighlight {
        pattern: pattern.to_string(),
        style: HighlightStyle::Emphasis0,
        layer,
        context: BTreeMap::new(),
        on_hover,
        bold,
        italic,
        underline,
        tooltip_text: None,
    }
}

#[test]
fn set_plugin_regex_highlights_basic_match() {
    let mut grid = create_grid_with_content("hello foo bar\n");
    let highlights = vec![RegexHighlight {
        pattern: "foo".into(),
        style: HighlightStyle::Emphasis0,
        layer: HighlightLayer::Hint,
        context: BTreeMap::new(),
        on_hover: false,
        bold: false,
        italic: false,
        underline: true,
        tooltip_text: None,
    }];
    grid.set_plugin_regex_highlights(1, highlights, &Style::default());
    let slot = grid.plugin_highlights.get(&1);
    assert!(slot.is_some());
    let entries = slot.unwrap();
    assert_eq!(entries.len(), 1);
    assert!(entries[0].1.regex.is_match("foo"));
}

#[test]
fn set_plugin_regex_highlights_no_match() {
    let mut grid = create_grid_with_content("hello world bar\n");
    let highlights = vec![create_highlight(
        "xyz123",
        false,
        false,
        false,
        false,
        HighlightLayer::Hint,
    )];
    grid.set_plugin_regex_highlights(1, highlights, &Style::default());

    // 视口中没有位置应匹配
    for col in 0..15 {
        assert!(grid.plugin_highlight_at(&Position::new(0, col)).is_none());
    }
}

#[test]
fn clear_plugin_highlights_removes_highlights() {
    let mut grid = create_grid_with_content("hello foo bar\n");
    let highlights = vec![create_highlight(
        "foo",
        false,
        false,
        false,
        true,
        HighlightLayer::Hint,
    )];
    grid.set_plugin_regex_highlights(1, highlights, &Style::default());
    assert!(grid.plugin_highlights.get(&1).is_some());

    grid.clear_plugin_highlights(1);
    assert!(grid.plugin_highlights.get(&1).is_none());
}

#[test]
fn multiple_plugins_highlights_independent() {
    let mut grid = create_grid_with_content("aaa bbb ccc\n");
    let h1 = vec![create_highlight(
        "aaa",
        false,
        false,
        false,
        false,
        HighlightLayer::Hint,
    )];
    let h2 = vec![create_highlight(
        "bbb",
        false,
        false,
        false,
        false,
        HighlightLayer::Hint,
    )];
    grid.set_plugin_regex_highlights(1, h1, &Style::default());
    grid.set_plugin_regex_highlights(2, h2, &Style::default());

    assert!(grid.plugin_highlights.get(&1).is_some());
    assert!(grid.plugin_highlights.get(&2).is_some());

    grid.clear_plugin_highlights(1);
    assert!(grid.plugin_highlights.get(&1).is_none());
    assert!(grid.plugin_highlights.get(&2).is_some());
}

#[test]
fn upsert_replaces_same_pattern() {
    let mut grid = create_grid_with_content("foo bar\n");
    let h1 = vec![RegexHighlight {
        pattern: "foo".into(),
        style: HighlightStyle::Emphasis0,
        layer: HighlightLayer::Hint,
        context: BTreeMap::new(),
        on_hover: false,
        bold: false,
        italic: false,
        underline: true,
        tooltip_text: None,
    }];
    grid.set_plugin_regex_highlights(1, h1, &Style::default());

    let h2 = vec![RegexHighlight {
        pattern: "foo".into(),
        style: HighlightStyle::Emphasis0,
        layer: HighlightLayer::Hint,
        context: BTreeMap::new(),
        on_hover: false,
        bold: false,
        italic: false,
        underline: false,
        tooltip_text: None,
    }];
    grid.set_plugin_regex_highlights(1, h2, &Style::default());

    let entries = grid.plugin_highlights.get(&1).unwrap();
    assert_eq!(entries.len(), 1);
    assert!(!entries[0].1.underline);
}

#[test]
fn invalid_regex_does_not_crash() {
    let mut grid = create_grid_with_content("hello\n");
    let highlights = vec![create_highlight(
        "[invalid",
        false,
        false,
        false,
        false,
        HighlightLayer::Hint,
    )];
    grid.set_plugin_regex_highlights(1, highlights, &Style::default());
    // 无效的正则表达式应被跳过
    let slot = grid.plugin_highlights.get(&1);
    match slot {
        None => {}, // 可接受
        Some(entries) => assert_eq!(entries.len(), 0),
    }
}

#[test]
fn plugin_highlight_at_returns_match() {
    let mut grid = create_grid_with_content("hello foo bar\n");
    let mut context = BTreeMap::new();
    context.insert("key".to_string(), "value".to_string());
    let highlights = vec![RegexHighlight {
        pattern: "foo".into(),
        style: HighlightStyle::Emphasis0,
        layer: HighlightLayer::Hint,
        context: context.clone(),
        on_hover: false,
        bold: false,
        italic: false,
        underline: true,
        tooltip_text: None,
    }];
    grid.set_plugin_regex_highlights(1, highlights, &Style::default());

    // "foo" 在 "hello foo bar" 中从第 6 列开始
    let result = grid.plugin_highlight_at(&Position::new(0, 6));
    assert!(result.is_some());
    let (plugin_id, pattern, matched_string, ctx) = result.unwrap();
    assert_eq!(plugin_id, 1);
    assert_eq!(pattern, "foo");
    assert_eq!(matched_string, "foo");
    assert_eq!(ctx.get("key").unwrap(), "value");
}

#[test]
fn plugin_highlight_at_returns_none_on_miss() {
    let mut grid = create_grid_with_content("hello foo bar\n");
    let highlights = vec![create_highlight(
        "foo",
        false,
        false,
        false,
        true,
        HighlightLayer::Hint,
    )];
    grid.set_plugin_regex_highlights(1, highlights, &Style::default());

    // 位置 0 在 "hello" 中，不是 "foo"
    let result = grid.plugin_highlight_at(&Position::new(0, 0));
    assert!(result.is_none());
}

#[test]
fn plugin_highlight_at_wrapped_line() {
    // 创建一个窄网格（10 列），以便长字符串换行
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let mut grid = Grid::new(
        5,
        10,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        false,
        true,
        true,
        true,
        false,
    );
    // 输入一个换行的长字符串："abcdefghij" 填满第 0 行，"klmnopqrst" 填满第 1 行
    let content = "abcdefghijklmnopqrst";
    let mut vte_parser = vte::Parser::new();
    vte_parser.advance(&mut grid, content.as_bytes());

    // 为跨越换行边界的 "jklm" 设置高亮
    let highlights = vec![create_highlight(
        "jklm",
        false,
        false,
        false,
        true,
        HighlightLayer::Hint,
    )];
    grid.set_plugin_regex_highlights(1, highlights, &Style::default());

    // "jklm" 跨越第 0 行第 9 列到第 1 行第 3 列
    // 换行部分中的位置（第 1 行，第 1 列 = 'k'）
    let result = grid.plugin_highlight_at(&Position::new(1, 1));
    assert!(result.is_some());
    let (plugin_id, _pattern, matched_string, _ctx) = result.unwrap();
    assert_eq!(plugin_id, 1);
    assert_eq!(matched_string, "jklm");
}

#[test]
fn hover_position_triggers_on_hover_highlight() {
    let mut grid = create_grid_with_content("hello link_text bar\n");
    let highlights = vec![create_highlight(
        "link_text",
        true,
        false,
        false,
        true,
        HighlightLayer::Hint,
    )];
    grid.set_plugin_regex_highlights(1, highlights, &Style::default());

    // 在 "link_text" 内设置悬停位置（从第 6 列开始）
    grid.set_hover_position(Some(Position::new(0, 8)));
    assert!(grid.hover_position.is_some());
    assert_eq!(grid.hover_position.unwrap(), Position::new(0, 8));

    // on_hover 条目应存在于 plugin_highlights 中
    let entries = grid.plugin_highlights.get(&1).unwrap();
    assert_eq!(entries.len(), 1);
    assert!(entries[0].1.on_hover);
}

#[test]
fn hover_suppressed_when_mouse_tracking_on() {
    let mut grid = create_grid_with_content("hello link_text bar\n");
    let highlights = vec![create_highlight(
        "link_text",
        true,
        false,
        false,
        true,
        HighlightLayer::Hint,
    )];
    grid.set_plugin_regex_highlights(1, highlights, &Style::default());

    // 启用鼠标跟踪 —— 渲染路径应跳过悬停高亮
    grid.mouse_tracking = MouseTracking::Normal;
    grid.set_hover_position(Some(Position::new(0, 8)));

    // 无论如何都会设置悬停位置（守卫在渲染路径中），
    // 但我们验证 mouse_tracking 不是 Off
    assert!(grid.hover_position.is_some());
    assert_ne!(grid.mouse_tracking, MouseTracking::Off);
}

#[test]
fn wide_char_display_column_mapping() {
    // CJK 字符："你好" = 2 个字符，每个 2 显示列宽，所以 "world" 从显示列 4 开始
    let mut grid = create_grid_with_content("你好world\n");
    let highlights = vec![create_highlight(
        "world",
        false,
        false,
        false,
        true,
        HighlightLayer::Hint,
    )];
    grid.set_plugin_regex_highlights(1, highlights, &Style::default());

    // "你好" 占据显示列 0-3，"world" 从显示列 4 开始
    let result = grid.plugin_highlight_at(&Position::new(0, 4));
    assert!(result.is_some());
    let (plugin_id, _pattern, matched_string, _ctx) = result.unwrap();
    assert_eq!(plugin_id, 1);
    assert_eq!(matched_string, "world");

    // 位置 2 应在 "你好" 内，不是 "world"
    let result_miss = grid.plugin_highlight_at(&Position::new(0, 2));
    assert!(result_miss.is_none());
}

#[test]
fn highlight_style_variants_resolve_colors() {
    use super::resolve_highlight_colors;

    let style = Style::default();

    // HighlightStyle::None 返回 (None, None)
    let (fg, bg): (Option<AnsiCode>, Option<AnsiCode>) =
        resolve_highlight_colors(&HighlightStyle::None, &style);
    assert!(fg.is_none());
    assert!(bg.is_none());

    // 仅带 fg 的 HighlightStyle::CustomRgb
    let (fg, bg): (Option<AnsiCode>, Option<AnsiCode>) = resolve_highlight_colors(
        &HighlightStyle::CustomRgb {
            fg: Some((255, 0, 0)),
            bg: None,
        },
        &style,
    );
    assert_eq!(fg, Some(AnsiCode::RgbCode((255, 0, 0))));
    assert!(bg.is_none());

    // 仅带 bg 的 HighlightStyle::CustomIndex
    let (fg, bg): (Option<AnsiCode>, Option<AnsiCode>) = resolve_highlight_colors(
        &HighlightStyle::CustomIndex {
            fg: None,
            bg: Some(42),
        },
        &style,
    );
    assert!(fg.is_none());
    assert_eq!(bg, Some(AnsiCode::ColorIndex(42)));

    // HighlightStyle::Emphasis0 应从调色板返回前景色
    let (fg, bg): (Option<AnsiCode>, Option<AnsiCode>) =
        resolve_highlight_colors(&HighlightStyle::Emphasis0, &style);
    assert!(fg.is_some());
    assert!(bg.is_none());
}

fn create_grid_with_scrollback() -> Grid {
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let mut grid = Grid::new(
        5,
        40,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        false,
        true,
        true,
        true,
        false,
    );
    let mut parser = vte::Parser::new();
    for i in 0..25 {
        let line = format!("scrollback line {}\r\n", i);
        parser.advance(&mut grid, line.as_bytes());
    }
    grid
}

#[test]
fn pane_contents_scrollback_no_truncation_when_max_none() {
    let grid = create_grid_with_scrollback();
    let result = grid.pane_contents(true, None);
    assert_eq!(
        result.lines_above_viewport.len(),
        21,
        "All scrollback lines should be returned when max is None"
    );
}

#[test]
fn pane_contents_scrollback_no_truncation_when_max_zero() {
    let grid = create_grid_with_scrollback();
    let result = grid.pane_contents(true, Some(0));
    assert_eq!(
        result.lines_above_viewport.len(),
        21,
        "Some(0) is sentinel for all scrollback — no truncation"
    );
}

#[test]
fn pane_contents_scrollback_truncates_to_last_n() {
    let grid = create_grid_with_scrollback();
    let result = grid.pane_contents(true, Some(5));
    assert_eq!(result.lines_above_viewport.len(), 5);
    let full = grid.pane_contents(true, None);
    let expected: Vec<String> = full
        .lines_above_viewport
        .iter()
        .rev()
        .take(5)
        .rev()
        .cloned()
        .collect();
    assert_eq!(result.lines_above_viewport, expected);
}

#[test]
fn pane_contents_scrollback_no_truncation_when_n_exceeds_total() {
    let grid = create_grid_with_scrollback();
    let result = grid.pane_contents(true, Some(100));
    assert_eq!(
        result.lines_above_viewport.len(),
        21,
        "No truncation when N exceeds total scrollback lines"
    );
}

#[test]
fn pane_contents_no_scrollback_when_flag_false() {
    let grid = create_grid_with_scrollback();
    let result = grid.pane_contents(false, Some(5));
    assert!(
        result.lines_above_viewport.is_empty(),
        "get_full_scrollback=false should never collect scrollback"
    );
}

// =====================================================================
// pane_contents_with_ansi 测试
// =====================================================================

fn create_grid_with_colored_scrollback() -> Grid {
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let mut grid = Grid::new(
        5,
        40,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        false,
        true,
        true,
        true,
        false,
    );
    let mut parser = vte::Parser::new();
    for i in 0..25 {
        let line = format!("\x1b[31mred line {}\x1b[0m\r\n", i);
        parser.advance(&mut grid, line.as_bytes());
    }
    grid
}

#[test]
fn pane_contents_with_ansi_preserves_escape_codes() {
    let grid = create_grid_with_colored_scrollback();
    let result = grid.pane_contents_with_ansi(false, None);
    let has_ansi = result.viewport.iter().any(|line| line.contains("\x1b["));
    assert!(
        has_ansi,
        "pane_contents_with_ansi should preserve ANSI escape codes in viewport. Lines: {:?}",
        result.viewport
    );
}

#[test]
fn pane_contents_strips_escape_codes() {
    let grid = create_grid_with_colored_scrollback();
    let result = grid.pane_contents(false, None);
    let has_ansi = result.viewport.iter().any(|line| line.contains("\x1b["));
    assert!(
        !has_ansi,
        "pane_contents should strip ANSI escape codes from viewport. Lines: {:?}",
        result.viewport
    );
}

#[test]
fn pane_contents_with_ansi_scrollback_preserves_escape_codes() {
    let grid = create_grid_with_colored_scrollback();
    let result = grid.pane_contents_with_ansi(true, None);
    let has_ansi = result
        .lines_above_viewport
        .iter()
        .any(|line| line.contains("\x1b["));
    assert!(
        has_ansi,
        "pane_contents_with_ansi should preserve ANSI escape codes in scrollback. Lines: {:?}",
        result.lines_above_viewport
    );
}

#[test]
fn pane_contents_with_ansi_scrollback_truncation() {
    let grid = create_grid_with_colored_scrollback();
    let result = grid.pane_contents_with_ansi(true, Some(3));
    assert_eq!(
        result.lines_above_viewport.len(),
        3,
        "Should truncate to 3 scrollback lines"
    );
    let all_have_ansi = result
        .lines_above_viewport
        .iter()
        .all(|line| line.contains("\x1b["));
    assert!(
        all_have_ansi,
        "All truncated scrollback lines should contain ANSI codes. Lines: {:?}",
        result.lines_above_viewport
    );
}

#[test]
fn pane_contents_with_ansi_no_scrollback_when_flag_false() {
    let grid = create_grid_with_colored_scrollback();
    let result = grid.pane_contents_with_ansi(false, Some(5));
    assert!(
        result.lines_above_viewport.is_empty(),
        "get_full_scrollback=false should never collect scrollback even with ansi"
    );
}

#[test]
fn pane_contents_with_ansi_and_without_have_same_text() {
    let grid = create_grid_with_colored_scrollback();
    let plain = grid.pane_contents(false, None);
    let ansi = grid.pane_contents_with_ansi(false, None);
    assert_eq!(
        plain.viewport.len(),
        ansi.viewport.len(),
        "Both should have the same number of viewport lines"
    );
    // 从 ansi 版本中剥离 ANSI 代码并比较纯文本内容
    let ansi_escape = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    for (plain_line, ansi_line) in plain.viewport.iter().zip(ansi.viewport.iter()) {
        let stripped = ansi_escape.replace_all(ansi_line, "").to_string();
        assert_eq!(
            *plain_line, stripped,
            "After stripping ANSI codes, text content should match"
        );
    }
}

// =====================================================================
// 高亮层优先级测试
// =====================================================================

#[test]
fn higher_layer_wins_plugin_highlight_at() {
    let mut grid = create_grid_with_content("hello foo bar\n");
    let h1 = vec![RegexHighlight {
        pattern: "foo".into(),
        style: HighlightStyle::Emphasis0,
        layer: HighlightLayer::Hint,
        context: BTreeMap::new(),
        on_hover: false,
        bold: false,
        italic: false,
        underline: false,
        tooltip_text: None,
    }];
    let h2 = vec![RegexHighlight {
        pattern: "foo".into(),
        style: HighlightStyle::Emphasis1,
        layer: HighlightLayer::Tool,
        context: BTreeMap::new(),
        on_hover: false,
        bold: false,
        italic: false,
        underline: false,
        tooltip_text: None,
    }];
    grid.set_plugin_regex_highlights(1, h1, &Style::default());
    grid.set_plugin_regex_highlights(2, h2, &Style::default());

    // "foo" 从第 6 列开始
    let result = grid.plugin_highlight_at(&Position::new(0, 6));
    assert!(result.is_some());
    let (plugin_id, _, _, _) = result.unwrap();
    assert_eq!(plugin_id, 2, "Tool layer plugin should win over Hint layer");
}

#[test]
fn same_layer_both_returned_deterministically() {
    let mut grid = create_grid_with_content("hello foo bar\n");
    let h1 = vec![RegexHighlight {
        pattern: "foo".into(),
        style: HighlightStyle::Emphasis0,
        layer: HighlightLayer::Hint,
        context: BTreeMap::new(),
        on_hover: false,
        bold: false,
        italic: false,
        underline: false,
        tooltip_text: None,
    }];
    let h2 = vec![RegexHighlight {
        pattern: "foo".into(),
        style: HighlightStyle::Emphasis1,
        layer: HighlightLayer::Hint,
        context: BTreeMap::new(),
        on_hover: false,
        bold: false,
        italic: false,
        underline: false,
        tooltip_text: None,
    }];
    grid.set_plugin_regex_highlights(1, h1, &Style::default());
    grid.set_plugin_regex_highlights(2, h2, &Style::default());

    let result = grid.plugin_highlight_at(&Position::new(0, 6));
    assert!(
        result.is_some(),
        "Same-layer conflicts should not cause errors"
    );
}

#[test]
fn lower_layer_wins_when_higher_layer_absent() {
    let mut grid = create_grid_with_content("foo bar\n");
    let h1 = vec![RegexHighlight {
        pattern: "foo".into(),
        style: HighlightStyle::Emphasis0,
        layer: HighlightLayer::Hint,
        context: BTreeMap::new(),
        on_hover: false,
        bold: false,
        italic: false,
        underline: false,
        tooltip_text: None,
    }];
    let h2 = vec![RegexHighlight {
        pattern: "bar".into(),
        style: HighlightStyle::Emphasis1,
        layer: HighlightLayer::ActionFeedback,
        context: BTreeMap::new(),
        on_hover: false,
        bold: false,
        italic: false,
        underline: false,
        tooltip_text: None,
    }];
    grid.set_plugin_regex_highlights(1, h1, &Style::default());
    grid.set_plugin_regex_highlights(2, h2, &Style::default());

    // 第 0 列的 "foo" —— 这里只有 Hint 层匹配
    let result_foo = grid.plugin_highlight_at(&Position::new(0, 0));
    assert!(result_foo.is_some());
    assert_eq!(result_foo.unwrap().0, 1);

    // 第 4 列的 "bar" —— 这里只有 ActionFeedback 层匹配
    let result_bar = grid.plugin_highlight_at(&Position::new(0, 4));
    assert!(result_bar.is_some());
    assert_eq!(result_bar.unwrap().0, 2);
}

#[test]
fn tooltip_from_higher_layer_wins() {
    let mut grid = create_grid_with_content("hello foo bar\n");
    let h1 = vec![RegexHighlight {
        pattern: "foo".into(),
        style: HighlightStyle::Emphasis0,
        layer: HighlightLayer::Hint,
        context: BTreeMap::new(),
        on_hover: true,
        bold: false,
        italic: false,
        underline: false,
        tooltip_text: Some("hint tooltip".to_string()),
    }];
    let h2 = vec![RegexHighlight {
        pattern: "foo".into(),
        style: HighlightStyle::Emphasis1,
        layer: HighlightLayer::Tool,
        context: BTreeMap::new(),
        on_hover: true,
        bold: false,
        italic: false,
        underline: false,
        tooltip_text: Some("tool tooltip".to_string()),
    }];
    grid.set_plugin_regex_highlights(1, h1, &Style::default());
    grid.set_plugin_regex_highlights(2, h2, &Style::default());

    // 在 "foo" 内设置悬停位置（从第 6 列开始）
    grid.set_hover_position(Some(Position::new(0, 6)));
    assert_eq!(
        grid.cached_hover_tooltip,
        Some("tool tooltip".to_string()),
        "Tool layer tooltip should win over Hint layer tooltip"
    );
}

#[test]
fn tooltip_from_lower_layer_when_higher_has_none() {
    let mut grid = create_grid_with_content("hello foo bar\n");
    let h1 = vec![RegexHighlight {
        pattern: "foo".into(),
        style: HighlightStyle::Emphasis0,
        layer: HighlightLayer::Hint,
        context: BTreeMap::new(),
        on_hover: true,
        bold: false,
        italic: false,
        underline: false,
        tooltip_text: Some("hint tooltip".to_string()),
    }];
    let h2 = vec![RegexHighlight {
        pattern: "foo".into(),
        style: HighlightStyle::Emphasis1,
        layer: HighlightLayer::Tool,
        context: BTreeMap::new(),
        on_hover: true,
        bold: false,
        italic: false,
        underline: false,
        tooltip_text: None,
    }];
    grid.set_plugin_regex_highlights(1, h1, &Style::default());
    grid.set_plugin_regex_highlights(2, h2, &Style::default());

    grid.set_hover_position(Some(Position::new(0, 6)));
    assert_eq!(
        grid.cached_hover_tooltip,
        Some("hint tooltip".to_string()),
        "When higher layer has no tooltip, lower layer tooltip should be used"
    );
}

#[test]
fn layer_field_stored_in_compiled_highlight() {
    let mut grid = create_grid_with_content("foo bar\n");
    let highlights = vec![RegexHighlight {
        pattern: "foo".into(),
        style: HighlightStyle::Emphasis0,
        layer: HighlightLayer::ActionFeedback,
        context: BTreeMap::new(),
        on_hover: false,
        bold: false,
        italic: false,
        underline: false,
        tooltip_text: None,
    }];
    grid.set_plugin_regex_highlights(1, highlights, &Style::default());
    assert_eq!(
        grid.plugin_highlights.get(&1).unwrap()[0].1.layer,
        HighlightLayer::ActionFeedback,
        "Layer field should be propagated through compilation"
    );
}

#[test]
fn default_layer_is_hint() {
    assert_eq!(HighlightLayer::default(), HighlightLayer::Hint);
}

#[test]
fn layer_ordering() {
    assert!(HighlightLayer::Hint < HighlightLayer::Tool);
    assert!(HighlightLayer::Tool < HighlightLayer::ActionFeedback);
}

fn row_text(row: &super::super::Row) -> String {
    row.columns
        .iter()
        .map(|c| c.character)
        .collect::<String>()
        .trim_end()
        .to_string()
}

fn scrollback_texts(grid: &Grid) -> Vec<String> {
    grid.lines_above.iter().map(|r| row_text(r)).collect()
}

fn viewport_texts(grid: &Grid) -> Vec<String> {
    grid.viewport.iter().map(|r| row_text(r)).collect()
}

fn create_grid_with_size_and_raw(rows: usize, cols: usize, content: &[u8]) -> Grid {
    let mut vte_parser = vte::Parser::new();
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let mut grid = Grid::new(
        rows,
        cols,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        false,
        true,
        true,
        true,
        false,
    );
    vte_parser.advance(&mut grid, &content);
    grid
}

fn feed_bytes(grid: &mut Grid, bytes: &[u8]) {
    let mut vte_parser = vte::Parser::new();
    vte_parser.advance(grid, bytes);
}

// 以下所有测试使用 10 行 40 列的网格，滚动区域为 1;8
//（0 基：行 0-7），行 8-9 在区域之外。
const PARTIAL_SR: &[u8] = b"\x1b[1;8r";
const FILL_8_LINES: &[u8] = b"AAA\r\nBBB\r\nCCC\r\nDDD\r\nEEE\r\nFFF\r\nGGG\r\nHHH";

#[test]
fn partial_scroll_region_newline_transfers_to_scrollback() {
    let mut content: Vec<u8> = Vec::new();
    content.extend_from_slice(PARTIAL_SR);
    content.extend_from_slice(FILL_8_LINES);
    content.extend_from_slice(b"\r\nIII\r\nJJJ");

    let grid = create_grid_with_size_and_raw(10, 40, &content);

    assert_eq!(scrollback_texts(&grid), vec!["AAA", "BBB"]);
    let vp = viewport_texts(&grid);
    assert_eq!(vp[0], "CCC");
    assert_eq!(vp[1], "DDD");
    assert_eq!(vp[6], "III");
    assert_eq!(vp[7], "JJJ");
}

#[test]
fn partial_scroll_region_csi_s_transfers_to_scrollback() {
    let mut content: Vec<u8> = Vec::new();
    content.extend_from_slice(PARTIAL_SR);
    content.extend_from_slice(FILL_8_LINES);
    content.extend_from_slice(b"\x1b[3S");

    let grid = create_grid_with_size_and_raw(10, 40, &content);

    assert_eq!(scrollback_texts(&grid), vec!["AAA", "BBB", "CCC"]);
    let vp = viewport_texts(&grid);
    assert_eq!(vp[0], "DDD");
    assert_eq!(vp[4], "HHH");
    assert_eq!(vp[5], "");
    assert_eq!(vp[7], "");
}

#[test]
fn partial_scroll_region_csi_m_at_top_transfers_to_scrollback() {
    let mut content: Vec<u8> = Vec::new();
    content.extend_from_slice(PARTIAL_SR);
    content.extend_from_slice(FILL_8_LINES);
    content.extend_from_slice(b"\x1b[1;1H\x1b[2M");

    let grid = create_grid_with_size_and_raw(10, 40, &content);

    assert_eq!(scrollback_texts(&grid), vec!["AAA", "BBB"]);
    let vp = viewport_texts(&grid);
    assert_eq!(vp[0], "CCC");
    assert_eq!(vp[5], "HHH");
    assert_eq!(vp[6], "");
    assert_eq!(vp[7], "");
}

#[test]
fn partial_scroll_region_csi_m_mid_region_does_not_transfer() {
    let mut content: Vec<u8> = Vec::new();
    content.extend_from_slice(PARTIAL_SR);
    content.extend_from_slice(FILL_8_LINES);
    // 光标到第 3 行，删除 2 行
    content.extend_from_slice(b"\x1b[4;1H\x1b[2M");

    let grid = create_grid_with_size_and_raw(10, 40, &content);

    assert_eq!(grid.lines_above.len(), 0);
    let vp = viewport_texts(&grid);
    assert_eq!(vp[0], "AAA");
    assert_eq!(vp[2], "CCC");
    assert_eq!(vp[3], "FFF");
    assert_eq!(vp[5], "HHH");
    assert_eq!(vp[6], "");
    assert_eq!(vp[7], "");
}

#[test]
fn partial_scroll_region_does_not_transfer_on_alternate_screen() {
    let mut content: Vec<u8> = Vec::new();
    content.extend_from_slice(b"\x1b[?1049h");
    content.extend_from_slice(PARTIAL_SR);
    content.extend_from_slice(FILL_8_LINES);
    content.extend_from_slice(b"\x1b[3S");

    let grid = create_grid_with_size_and_raw(10, 40, &content);

    assert_eq!(grid.lines_above.len(), 0);
    let vp = viewport_texts(&grid);
    assert_eq!(vp[0], "DDD");
    assert_eq!(vp[4], "HHH");
}

#[test]
fn partial_scroll_region_nonzero_top_does_not_transfer() {
    let mut content: Vec<u8> = Vec::new();
    content.extend_from_slice(b"AAA\r\nBBB\r\nCCC\r\nDDD\r\nEEE\r\nFFF");
    // 滚动区域为行 3-6（1 基），所以顶部是行 2，不是行 0
    content.extend_from_slice(b"\x1b[3;6r");
    content.extend_from_slice(b"\x1b[2S");

    let grid = create_grid_with_size_and_raw(10, 40, &content);

    assert_eq!(grid.lines_above.len(), 0);
    let vp = viewport_texts(&grid);
    assert_eq!(vp[0], "AAA");
    assert_eq!(vp[1], "BBB");
    assert_eq!(vp[2], "EEE");
    assert_eq!(vp[3], "FFF");
    assert_eq!(vp[4], "");
    assert_eq!(vp[5], "");
}

#[test]
fn partial_scroll_region_selection_adjusted_on_newline() {
    let mut grid = create_grid_with_size_and_raw(10, 40, FILL_8_LINES);
    feed_bytes(&mut grid, PARTIAL_SR);

    let pos = Position::new(3, 5);
    grid.start_selection(&pos);
    grid.end_selection(&pos);
    let start_before = grid.selection.start.line.0;
    let end_before = grid.selection.end.line.0;

    feed_bytes(&mut grid, b"\x1b[8;1H\r\nnew line");

    assert_eq!(grid.selection.start.line.0, start_before - 1);
    assert_eq!(grid.selection.end.line.0, end_before - 1);
}

#[test]
fn partial_scroll_region_selection_adjusted_on_csi_s() {
    let mut grid = create_grid_with_size_and_raw(10, 40, FILL_8_LINES);
    feed_bytes(&mut grid, PARTIAL_SR);

    let pos = Position::new(4, 2);
    grid.start_selection(&pos);
    grid.end_selection(&pos);
    let start_before = grid.selection.start.line.0;
    let end_before = grid.selection.end.line.0;

    feed_bytes(&mut grid, b"\x1b[2S");

    assert_eq!(grid.selection.start.line.0, start_before - 2);
    assert_eq!(grid.selection.end.line.0, end_before - 2);
}

#[test]
fn partial_scroll_region_selection_adjusted_on_csi_m() {
    let mut grid = create_grid_with_size_and_raw(10, 40, FILL_8_LINES);
    feed_bytes(&mut grid, PARTIAL_SR);

    let pos = Position::new(5, 2);
    grid.start_selection(&pos);
    grid.end_selection(&pos);
    let start_before = grid.selection.start.line.0;
    let end_before = grid.selection.end.line.0;

    feed_bytes(&mut grid, b"\x1b[3M");

    assert_eq!(grid.selection.start.line.0, start_before - 3);
    assert_eq!(grid.selection.end.line.0, end_before - 3);
}

#[test]
fn partial_scroll_region_all_three_mechanisms_transfer_in_order() {
    let mut content: Vec<u8> = Vec::new();
    content.extend_from_slice(PARTIAL_SR);
    content.extend_from_slice(FILL_8_LINES);
    // 换行将 AAA 滚出，CSI S 将 BBB 滚出，CSI M 将 CCC 滚出
    content.extend_from_slice(b"\r\nIII");
    content.extend_from_slice(b"\x1b[1S");
    content.extend_from_slice(b"\x1b[1;1H\x1b[1M");

    let grid = create_grid_with_size_and_raw(10, 40, &content);

    assert_eq!(scrollback_texts(&grid), vec!["AAA", "BBB", "CCC"]);
}

#[test]
fn wrap_at_bottom_of_partial_scroll_region_transfers_to_scrollback() {
    // 当光标在滚动区域的底行时，一个软换行的长行
    // 必须滚动区域，就像那里有换行一样
    let mut content: Vec<u8> = Vec::new();
    content.extend_from_slice(PARTIAL_SR);
    content.extend_from_slice(FILL_8_LINES);
    content.extend_from_slice(b"\r\n");
    content.extend_from_slice(&[b'I'; 90]); // 40 列网格上的 90 列：换行两次

    let grid = create_grid_with_size_and_raw(10, 40, &content);

    // 换行将 AAA 滚出，两次换行将 BBB 和 CCC 滚出
    assert_eq!(scrollback_texts(&grid), vec!["AAA", "BBB", "CCC"]);
    let vp = viewport_texts(&grid);
    assert_eq!(vp[0], "DDD");
    assert_eq!(vp[4], "HHH");
    assert_eq!(vp[5], "I".repeat(40));
    assert_eq!(vp[6], "I".repeat(40));
    assert_eq!(vp[7], "I".repeat(10));
    assert_eq!(vp.len(), 8, "rows below the region must be untouched");
    // 续行是换行行，所以调整大小会正确地重新换行它们
    assert!(grid.viewport[5].is_canonical);
    assert!(!grid.viewport[6].is_canonical);
    assert!(!grid.viewport[7].is_canonical);
}

#[test]
fn wrap_at_bottom_of_partial_scroll_region_keeps_cursor_inside_region() {
    // 修复前，光标会逃逸到区域底部边距之下
    // 在换行时，之后没有任何行能再滚动到回滚缓冲区中
    let mut content: Vec<u8> = Vec::new();
    content.extend_from_slice(PARTIAL_SR);
    content.extend_from_slice(FILL_8_LINES);
    content.extend_from_slice(b"\r\n");
    content.extend_from_slice(&[b'I'; 90]);

    let grid = create_grid_with_size_and_raw(10, 40, &content);

    // 区域为行 0-7；两次换行后光标必须仍在第 7 行
    assert_eq!(
        grid.cursor_coordinates().map(|(x, y, _)| (x, y)),
        Some((10, 7))
    );
}

#[test]
fn wrap_at_bottom_of_nonzero_top_scroll_region_does_not_transfer() {
    let mut content: Vec<u8> = Vec::new();
    content.extend_from_slice(b"AAA\r\nBBB\r\nCCC\r\nDDD\r\nEEE\r\nFFF");
    // 滚动区域为行 3-6（1 基），所以顶部是行 2，不是行 0
    content.extend_from_slice(b"\x1b[3;6r");
    content.extend_from_slice(b"\x1b[6;1H");
    content.extend_from_slice(&[b'X'; 50]); // 在区域底部换行一次

    let grid = create_grid_with_size_and_raw(10, 40, &content);

    // 未锚定在顶部的区域不保留被滚出的行
    assert_eq!(grid.lines_above.len(), 0);
    let vp = viewport_texts(&grid);
    assert_eq!(vp[0], "AAA", "rows above the region must be untouched");
    assert_eq!(vp[1], "BBB", "rows above the region must be untouched");
    assert_eq!(vp[2], "DDD", "the region's top row (CCC) is discarded");
    assert_eq!(vp[4], "X".repeat(40));
    assert_eq!(vp[5], "X".repeat(10));
}

#[test]
fn wrap_at_bottom_of_partial_scroll_region_does_not_transfer_on_alternate_screen() {
    let mut content: Vec<u8> = Vec::new();
    content.extend_from_slice(b"\x1b[?1049h");
    content.extend_from_slice(PARTIAL_SR);
    content.extend_from_slice(FILL_8_LINES);
    content.extend_from_slice(b"\r\n");
    content.extend_from_slice(&[b'I'; 90]);

    let grid = create_grid_with_size_and_raw(10, 40, &content);

    assert_eq!(grid.lines_above.len(), 0);
    let vp = viewport_texts(&grid);
    assert_eq!(vp[5], "I".repeat(40));
    assert_eq!(vp[6], "I".repeat(40));
    assert_eq!(vp[7], "I".repeat(10));
}

#[test]
fn wrap_at_bottom_of_explicit_full_screen_scroll_region_keeps_wrap_flag() {
    // 10 行网格上的 CSI 1;10r 覆盖整个屏幕；换行行为必须
    // 与完全没有设置区域相同（滚动 + 换行行）
    let mut content: Vec<u8> = Vec::new();
    content.extend_from_slice(b"\x1b[1;10r");
    for i in 1..=10u8 {
        if i > 1 {
            content.extend_from_slice(b"\r\n");
        }
        content.extend_from_slice(b"EARLY-");
        content.push(b'0' + i / 10);
        content.push(b'0' + i % 10);
    }
    content.extend_from_slice(b"\r\n");
    content.extend_from_slice(&[b'I'; 90]); // 在 40 列网格上换行两次

    let grid = create_grid_with_size_and_raw(10, 40, &content);

    // 换行和两次换行各将一行滚动到回滚缓冲区中
    assert_eq!(
        scrollback_texts(&grid),
        vec!["EARLY-01", "EARLY-02", "EARLY-03"]
    );
    let vp = viewport_texts(&grid);
    assert_eq!(vp[6], "EARLY-10");
    assert_eq!(vp[7], "I".repeat(40));
    assert_eq!(vp[8], "I".repeat(40));
    assert_eq!(vp[9], "I".repeat(10));
    assert!(grid.viewport[7].is_canonical);
    assert!(
        !grid.viewport[8].is_canonical,
        "continuation must stay a wrapped row"
    );
    assert!(
        !grid.viewport[9].is_canonical,
        "continuation must stay a wrapped row"
    );
}

#[test]
fn wrap_at_bottom_of_nonzero_top_scroll_region_keeps_hyperlink_tracking() {
    // 一个在未锚定在顶部的区域底部换行的纯文本 URL
    // ：自动链接器跟踪的位置必须跟随行
    // 随着区域滚动，否则 URL 将失去其可点击锚点
    let mut content: Vec<u8> = Vec::new();
    content.extend_from_slice(b"AAA\r\nBBB\r\nCCC\r\nDDD\r\nEEE\r\nFFF");
    content.extend_from_slice(b"\x1b[3;6r\x1b[6;1H");
    content.extend_from_slice(b"https://example.com/");
    content.extend_from_slice(&[b'a'; 30]); // 50 个字符：在区域底部换行
    content.extend_from_slice(b" ");

    let grid = create_grid_with_size_and_raw(10, 40, &content);

    let start_anchors_per_row: Vec<usize> = grid
        .viewport
        .iter()
        .map(|row| {
            row.columns
                .iter()
                .filter(|c| {
                    matches!(
                        c.styles.link_anchor,
                        Some(crate::panes::terminal_character::LinkAnchor::Start(_))
                    )
                })
                .count()
        })
        .collect();

    // 行 0-3：AAA、BBB、DDD、EEE（CCC 被滚出并丢弃）- 无锚点；
    // 第 4 行包含 URL 的前 40 列，第 5 行包含换行的剩余部分
    assert_eq!(start_anchors_per_row, vec![0, 0, 0, 0, 40, 10]);
}

#[test]
fn scroll_region_newline_sets_bg_color_on_new_row() {
    // 设置背景色，设置滚动区域 1-5，填充 5 行，然后换行滚动
    let content = b"\x1b[48;2;26;26;26m\x1b[1;5r\
        AAA\r\nBBB\r\nCCC\r\nDDD\r\nEEE\r\nFFF";
    let grid = create_grid_with_size_and_raw(10, 40, content);
    // 滚动区域底部的新行（行索引 4）应具有 bg_color
    let new_row = &grid.viewport[4];
    assert_eq!(
        new_row.bg_color,
        Some(AnsiCode::RgbCode((26, 26, 26))),
        "scroll-created row should carry the cursor background color"
    );
}

#[test]
fn scroll_region_newline_bg_color_used_for_trailing_padding() {
    // 设置背景，滚动区域 1-5，填充行，滚动，然后在新行上写短文本
    let content = b"\x1b[48;2;26;26;26m\x1b[1;5rAAA\r\nBBB\r\nCCC\r\nDDD\r\nEEE\r\nhi";
    let mut grid = create_grid_with_size_and_raw(10, 40, content);
    // read_changes 返回应用了填充的字符块
    let (chunks, _, _) = grid.read_changes(0, 0);
    // 找到第 4 行的块（滚动创建的带有 "hi" 的行）
    let row_4_chunk = chunks.iter().find(|c| c.y == 4).expect("row 4 chunk");
    // 尾随填充字符（最后一列）应具有该行的 bg_color
    let pad_char = &row_4_chunk.terminal_characters[39];
    assert_eq!(
        pad_char.styles.background,
        Some(AnsiCode::RgbCode((26, 26, 26))),
        "trailing padding should use the row's background color"
    );
}

#[test]
fn scroll_region_newline_bg_color_used_for_cursor_forward_gaps() {
    // 设置背景，滚动区域 1-5，填充行，滚动，然后光标前进并写入
    // 这模拟了 vim 在滚动创建的行上的 [12C 行为
    let content = b"\x1b[48;2;26;26;26m\x1b[1;5rAAA\r\nBBB\r\nCCC\r\nDDD\r\nEEE\
        \r\n\x1b[0m\x1b[10Cx";
    let grid = create_grid_with_size_and_raw(10, 40, content);
    let new_row = &grid.viewport[4];
    // 位置 5（在光标前进创建的间隙内）应具有 bg_color
    let gap_char = &new_row.columns[5];
    assert_eq!(
        gap_char.styles.background,
        Some(AnsiCode::RgbCode((26, 26, 26))),
        "cursor-forward gap should use the row's background color"
    );
}

#[test]
fn scroll_region_bg_color_does_not_override_explicit_background() {
    // 设置背景，滚动区域 1-5，填充行，滚动，然后用不同的背景写入
    let content = b"\x1b[48;2;26;26;26m\x1b[1;5rAAA\r\nBBB\r\nCCC\r\nDDD\r\nEEE\
        \r\n\x1b[48;2;255;0;0mRED";
    let grid = create_grid_with_size_and_raw(10, 40, content);
    let new_row = &grid.viewport[4];
    // 'R' 字符应具有显式设置的红色背景，而不是行背景
    let r_char = &new_row.columns[0];
    assert_eq!(
        r_char.styles.background,
        Some(AnsiCode::RgbCode((255, 0, 0))),
        "explicitly set background should not be overridden by row bg_color"
    );
}

#[test]
fn full_scroll_region_newline_sets_bg_color_on_new_row() {
    // 完整滚动区域（整个视口），设置背景，填充，然后滚动
    let content = b"\x1b[48;2;26;26;26m\x1b[1;10r\
        L1\r\nL2\r\nL3\r\nL4\r\nL5\r\nL6\r\nL7\r\nL8\r\nL9\r\nL10\r\nL11";
    let grid = create_grid_with_size_and_raw(10, 40, content);
    // 底部的新行（第 9 行）应具有 bg_color
    let new_row = &grid.viewport[9];
    assert_eq!(
        new_row.bg_color,
        Some(AnsiCode::RgbCode((26, 26, 26))),
        "full scroll region: new row should carry the cursor background color"
    );
}

#[test]
fn row_without_scroll_has_no_bg_color() {
    // 没有滚动的普通内容不应在行上设置 bg_color
    let content = b"\x1b[48;2;26;26;26mHello";
    let grid = create_grid_with_size_and_raw(10, 40, content);
    let row = &grid.viewport[0];
    assert_eq!(
        row.bg_color, None,
        "rows not created by scroll should have no bg_color"
    );
}

fn new_grid_for_forwarding_test() -> Grid {
    Grid::new(
        10,
        20,
        Rc::new(RefCell::new(Palette::default())),
        Rc::new(RefCell::new(HashMap::new())),
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(Some(SizeInPixels {
            width: 8,
            height: 16,
        }))),
        Rc::new(RefCell::new(SixelImageStore::default())),
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        false,
        true,
        true,
        true,
        false,
    )
}

#[test]
fn csi_14t_forwards_to_host_not_local() {
    // CSI 14t 过去会生成本地 "\x1b[4;H;Wt" 回复；重构后
    // 它必须改为转发给主机，以便应用观察到
    // 终端真实的窗口像素尺寸。
    let mut parser = vte::Parser::new();
    let mut grid = new_grid_for_forwarding_test();
    parser.advance(&mut grid, b"\x1b[14t");
    assert!(
        grid.pending_messages_to_pty.is_empty(),
        "local reply path must not fire for 14t"
    );
    assert_eq!(grid.pending_forwarded_queries.len(), 1);
    assert_eq!(
        grid.pending_forwarded_queries[0],
        crate::host_query::HostQuery::TextAreaPixelSize,
        "forwarded classification must be TextAreaPixelSize"
    );
}

#[test]
fn csi_16t_forwards_to_host_not_local() {
    let mut parser = vte::Parser::new();
    let mut grid = new_grid_for_forwarding_test();
    parser.advance(&mut grid, b"\x1b[16t");
    assert!(grid.pending_messages_to_pty.is_empty());
    assert_eq!(grid.pending_forwarded_queries.len(), 1);
    assert_eq!(
        grid.pending_forwarded_queries[0],
        crate::host_query::HostQuery::CharacterCellPixelSize,
    );
}

#[test]
fn csi_18t_still_answered_locally() {
    // 18 以单元格为单位报告 Zellij 自己的文本区域大小 —— Zellij 是
    // 此信息的权威来源，请勿转发。
    let mut parser = vte::Parser::new();
    let mut grid = new_grid_for_forwarding_test();
    parser.advance(&mut grid, b"\x1b[18t");
    assert_eq!(grid.pending_messages_to_pty.len(), 1);
    assert!(grid.pending_forwarded_queries.is_empty());
}

#[test]
fn osc_11_set_stays_local() {
    // OSC 11;<rgb>（设置窗格默认背景）必须保持本地 —— Zellij 需要
    // 为自己的渲染跟踪它。
    let mut parser = vte::Parser::new();
    let mut grid = new_grid_for_forwarding_test();
    parser.advance(&mut grid, b"\x1b]11;rgb:ffff/ffff/ffff\x07");
    assert!(grid.pending_messages_to_pty.is_empty());
    assert!(
        grid.pending_forwarded_queries.is_empty(),
        "set (not query) must not forward"
    );
    assert!(grid.pane_default_bg.is_some());
}

#[test]
fn osc_11_query_without_override_forwards_to_host() {
    // 当没有窗格本地覆盖生效时，查询仍必须
    // 被转发 —— 主机的实际背景就是应用所要求的。
    let mut parser = vte::Parser::new();
    let mut grid = new_grid_for_forwarding_test();
    assert!(grid.pane_default_bg.is_none());
    parser.advance(&mut grid, b"\x1b]11;?\x07");
    assert!(
        grid.pending_messages_to_pty.is_empty(),
        "no override → no local reply"
    );
    assert_eq!(grid.pending_forwarded_queries.len(), 1);
    assert!(matches!(
        grid.pending_forwarded_queries[0],
        crate::host_query::HostQuery::DefaultBackground { .. }
    ));
}

#[test]
fn osc_10_query_without_override_forwards_to_host() {
    let mut parser = vte::Parser::new();
    let mut grid = new_grid_for_forwarding_test();
    assert!(grid.pane_default_fg.is_none());
    parser.advance(&mut grid, b"\x1b]10;?\x07");
    assert!(grid.pending_messages_to_pty.is_empty());
    assert_eq!(grid.pending_forwarded_queries.len(), 1);
    assert!(matches!(
        grid.pending_forwarded_queries[0],
        crate::host_query::HostQuery::DefaultForeground { .. }
    ));
}

#[test]
fn set_pane_default_colors_short_circuits_osc_queries() {
    // CLI 路径（`zellij action set-pane-color`）落在
    // `Grid::set_pane_default_colors` 上。后续的 OSC 10/11 查询必须
    // 读取该覆盖，而不是被转发 —— CLI 覆盖的全部意义在于
    // 应用看到 Zellij 正在绘制的颜色，而不是底层主机的颜色。
    //
    let mut parser = vte::Parser::new();
    let mut grid = new_grid_for_forwarding_test();
    grid.set_pane_default_colors(Some("#ff8040".to_string()), Some("#102030".to_string()));

    parser.advance(&mut grid, b"\x1b]10;?\x07");
    parser.advance(&mut grid, b"\x1b]11;?\x07");

    assert!(
        grid.pending_forwarded_queries.is_empty(),
        "overrides set via set_pane_default_colors must suppress forwarding"
    );
    assert_eq!(grid.pending_messages_to_pty.len(), 2);
    let fg_reply = String::from_utf8(grid.pending_messages_to_pty[0].clone()).unwrap();
    let bg_reply = String::from_utf8(grid.pending_messages_to_pty[1].clone()).unwrap();
    assert_eq!(fg_reply, "\u{1b}]10;rgb:ffff/8080/4040\u{7}");
    assert_eq!(bg_reply, "\u{1b}]11;rgb:1010/2020/3030\u{7}");
}

#[test]
fn osc_11_override_short_circuits_only_the_overridden_channel() {
    // 仅设置背景不能短路前景查询 —— 每个通道的覆盖是独立的。
    // 如果只有 `pane_default_bg` 被填充，OSC 10 查询（前景）仍然发送到主机。
    //
    let mut parser = vte::Parser::new();
    let mut grid = new_grid_for_forwarding_test();
    parser.advance(&mut grid, b"\x1b]11;rgb:1010/2020/3030\x07");
    assert!(grid.pane_default_bg.is_some());
    assert!(grid.pane_default_fg.is_none());

    parser.advance(&mut grid, b"\x1b]10;?\x07");
    assert!(
        grid.pending_messages_to_pty.is_empty(),
        "fg has no override → must forward"
    );
    assert_eq!(grid.pending_forwarded_queries.len(), 1);
}

#[test]
fn csi_2026_dollar_p_stays_local() {
    // DECRQM 模式 2026（同步输出）由 Zellij 自己模拟 —— 从不转发。
    // 响应是 DECRPM 形式 `\x1b[?2026;2$y`（2 = "已重置但已识别"；
    // Zellij 用括号括起自己的帧，所以单个窗格被视为"未启用"）。
    //
    let mut parser = vte::Parser::new();
    let mut grid = new_grid_for_forwarding_test();
    parser.advance(&mut grid, b"\x1b[?2026$p");
    assert!(
        grid.pending_forwarded_queries.is_empty(),
        "CSI ?2026$p is emulated locally, must not forward"
    );
    assert_eq!(grid.pending_messages_to_pty.len(), 1);
    assert_eq!(
        grid.pending_messages_to_pty[0], b"\x1b[?2026;2$y",
        "DECRPM reply must use the `$y` suffix per spec"
    );
}

#[test]
fn csi_2031_dollar_p_when_disabled_replies_reset() {
    // DECRQM 模式 2031（应用主题报告）是由 Zellij 本地跟踪的
    // 每窗格状态。在没有先前的 `CSI ? 2031 h` 的情况下，模式
    // 被重置，因此 DECRPM 回复必须报告 value=2。
    let mut parser = vte::Parser::new();
    let mut grid = new_grid_for_forwarding_test();
    parser.advance(&mut grid, b"\x1b[?2031$p");
    assert!(
        grid.pending_forwarded_queries.is_empty(),
        "CSI ?2031$p is answered locally, must not forward"
    );
    assert_eq!(grid.pending_messages_to_pty.len(), 1);
    assert_eq!(
        grid.pending_messages_to_pty[0], b"\x1b[?2031;2$y",
        "DECRPM reply must report value=2 (reset) when 2031 is disabled"
    );
}

#[test]
fn csi_2031_dollar_p_when_enabled_replies_set() {
    // 在 `CSI ? 2031 h` 在此窗格上启用主题更改通知后，
    // DECRQM 探测必须报告 value=1（已设置）。
    let mut parser = vte::Parser::new();
    let mut grid = new_grid_for_forwarding_test();
    parser.advance(&mut grid, b"\x1b[?2031h\x1b[?2031$p");
    assert!(
        grid.pending_forwarded_queries.is_empty(),
        "CSI ?2031$p is answered locally, must not forward"
    );
    assert_eq!(grid.pending_messages_to_pty.len(), 1);
    assert_eq!(
        grid.pending_messages_to_pty[0], b"\x1b[?2031;1$y",
        "DECRPM reply must report value=1 (set) when 2031 is enabled"
    );
}

#[test]
fn csi_22t_and_23t_stay_local() {
    // CSI 22t / 23t 操作窗格的标题栈 —— Zellij 拥有该状态，
    // 因此转发会将应用的 push/pop 路由到主机不相关的标题栈，
    // 而不是可见的窗格标题。它们不产生回复；每次操作后两个队列必须保持为空。
    //
    let mut parser = vte::Parser::new();
    let mut grid = new_grid_for_forwarding_test();
    grid.set_title("hello".to_string());

    parser.advance(&mut grid, b"\x1b[22;0t");
    assert!(
        grid.pending_forwarded_queries.is_empty(),
        "22t is a stack op, not a query"
    );
    assert!(
        grid.pending_messages_to_pty.is_empty(),
        "22t produces no reply"
    );

    // push 实际上落在了 Zellij 的每窗格栈上：恢复它。
    grid.set_title("different".to_string());
    parser.advance(&mut grid, b"\x1b[23;0t");
    assert!(
        grid.pending_forwarded_queries.is_empty(),
        "23t is a stack op, not a query"
    );
    assert!(
        grid.pending_messages_to_pty.is_empty(),
        "23t produces no reply"
    );
    assert_eq!(
        grid.title.as_deref(),
        Some("hello"),
        "pop should restore the previously-pushed title"
    );
}

#[test]
fn osc_4_set_stays_local() {
    // OSC 4;<index>;<rgb> 写入 Zellij 的内存调色板；只有
    // 查询形式（`OSC 4;N;?`）才会转发给主机。
    let mut parser = vte::Parser::new();
    let mut grid = new_grid_for_forwarding_test();
    parser.advance(&mut grid, b"\x1b]4;5;rgb:ffff/0000/0000\x07");
    assert!(
        grid.pending_forwarded_queries.is_empty(),
        "OSC 4 set must not forward"
    );
    assert!(
        grid.pending_messages_to_pty.is_empty(),
        "OSC 4 set produces no reply"
    );
    let changed = grid
        .changed_colors
        .expect("changed_colors should be populated by OSC 4 set");
    assert!(changed[5].is_some(), "index 5 should have been written");
}

#[test]
fn decset_2031_enables_color_palette_notification() {
    let mut parser = vte::Parser::new();
    let mut grid = new_grid_for_forwarding_test();
    assert!(
        !grid.color_palette_notification_enabled,
        "default state must be disabled"
    );
    parser.advance(&mut grid, b"\x1b[?2031h");
    assert!(
        grid.color_palette_notification_enabled,
        "DECSET 2031 must enable the flag"
    );
    parser.advance(&mut grid, b"\x1b[?2031l");
    assert!(
        !grid.color_palette_notification_enabled,
        "DECRST 2031 must disable the flag"
    );
}

#[test]
fn push_color_palette_dsr_emits_when_enabled() {
    let mut grid = new_grid_for_forwarding_test();
    grid.color_palette_notification_enabled = true;
    grid.push_color_palette_dsr(zellij_utils::data::HostTerminalThemeMode::Dark);
    assert_eq!(grid.pending_messages_to_pty.len(), 1);
    assert_eq!(
        grid.pending_messages_to_pty[0],
        b"\x1b[?997;1n".to_vec(),
        "Dark mode emits ?997;1n"
    );
    grid.push_color_palette_dsr(zellij_utils::data::HostTerminalThemeMode::Light);
    assert_eq!(grid.pending_messages_to_pty.len(), 2);
    assert_eq!(
        grid.pending_messages_to_pty[1],
        b"\x1b[?997;2n".to_vec(),
        "Light mode emits ?997;2n"
    );
}

#[test]
fn push_color_palette_dsr_noop_when_disabled() {
    let mut grid = new_grid_for_forwarding_test();
    assert!(!grid.color_palette_notification_enabled);
    grid.push_color_palette_dsr(zellij_utils::data::HostTerminalThemeMode::Dark);
    assert!(
        grid.pending_messages_to_pty.is_empty(),
        "no DSR is queued when the app has not opted in via CSI ?2031h"
    );
}

#[test]
fn csi_996n_pushes_color_palette_mode_query_to_forwarded_queries() {
    use crate::host_query::HostQuery;
    let mut parser = vte::Parser::new();
    let mut grid = new_grid_for_forwarding_test();
    parser.advance(&mut grid, b"\x1b[?996n");
    assert_eq!(
        grid.pending_forwarded_queries,
        vec![HostQuery::ColorPaletteMode],
        "CSI ?996n must push HostQuery::ColorPaletteMode for Screen to short-circuit"
    );
    assert!(
        grid.pending_messages_to_pty.is_empty(),
        "Grid must NOT answer locally — Screen owns the host_terminal_theme_mode cache"
    );
}

#[test]
fn csi_5n_status_query_still_handled_locally() {
    let mut parser = vte::Parser::new();
    let mut grid = new_grid_for_forwarding_test();
    // 普通 DSR 5（没有 `?` 中间符）不是新的主题查询，
    // 必须继续在本地接收其 `\e[0n` "一切正常" 回复。
    parser.advance(&mut grid, b"\x1b[5n");
    assert!(
        grid.pending_forwarded_queries.is_empty(),
        "DSR 5 is not a forwarded query"
    );
    assert_eq!(
        grid.pending_messages_to_pty,
        vec![b"\x1b[0n".to_vec()],
        "DSR 5 must still produce its local 'all good' reply"
    );
}

fn new_grid_for_nested_frames() -> Grid {
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    Grid::new(
        41,
        110,
        Rc::new(RefCell::new(Palette::default())),
        terminal_emulator_color_codes,
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(None)),
        sixel_image_store,
        Rc::new(RefCell::new(KittyImageStore::default())),
        Style::default(),
        false,
        true,
        true,
        true,
        false,
    )
}

fn nested_announce_message() -> zellij_utils::nested_session::NestedSessionMessage {
    zellij_utils::nested_session::NestedSessionMessage::Announce {
        session_name: "guest-session".to_owned(),
        capabilities: vec![zellij_utils::nested_session::NestedSessionCapability::NestedControl],
    }
}

#[test]
fn nested_session_frame_is_staged_for_dispatch() {
    let mut vte_parser = vte::Parser::new();
    let mut grid = new_grid_for_nested_frames();
    let message = nested_announce_message();
    vte_parser.advance(
        &mut grid,
        &zellij_utils::nested_session::encode_frame(&message),
    );
    assert_eq!(grid.pending_nested_session_messages, vec![message]);
}

#[test]
fn nested_session_frame_split_across_feeds_is_staged() {
    let mut vte_parser = vte::Parser::new();
    let mut grid = new_grid_for_nested_frames();
    let message = nested_announce_message();
    let frame = zellij_utils::nested_session::encode_frame(&message);
    let (first_half, second_half) = frame.split_at(frame.len() / 2);
    vte_parser.advance(&mut grid, first_half);
    assert!(grid.pending_nested_session_messages.is_empty());
    vte_parser.advance(&mut grid, second_half);
    assert_eq!(grid.pending_nested_session_messages, vec![message]);
}

#[test]
fn dcs_with_wrong_magic_param_is_not_staged() {
    let mut vte_parser = vte::Parser::new();
    let mut grid = new_grid_for_nested_frames();
    vte_parser.advance(&mut grid, b"\x1bP26662nAAAA\x1b\\");
    assert!(grid.pending_nested_session_messages.is_empty());
}

#[test]
fn dcs_with_intermediates_is_not_staged() {
    let mut vte_parser = vte::Parser::new();
    let mut grid = new_grid_for_nested_frames();
    vte_parser.advance(&mut grid, b"\x1bP26661$nAAAA\x1b\\");
    assert!(grid.pending_nested_session_messages.is_empty());
}

#[test]
fn dcs_n_without_params_is_not_staged() {
    let mut vte_parser = vte::Parser::new();
    let mut grid = new_grid_for_nested_frames();
    vte_parser.advance(&mut grid, b"\x1bPnAAAA\x1b\\");
    assert!(grid.pending_nested_session_messages.is_empty());
}

#[test]
fn nested_session_frame_with_garbage_payload_is_dropped() {
    let mut vte_parser = vte::Parser::new();
    let mut grid = new_grid_for_nested_frames();
    vte_parser.advance(&mut grid, b"\x1bP26661n!!!not-base64!!!\x1b\\");
    assert!(grid.pending_nested_session_messages.is_empty());
}

fn serialize_text_bytes(text: &str) -> String {
    text.as_bytes()
        .iter()
        .map(|byte| byte.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn render_ui_component(component_name: &str, serialized_params: &str) -> Grid {
    let mut vte_parser = vte::Parser::new();
    let mut grid = new_grid_for_nested_frames();
    let dcs = format!("\u{1b}Pz{};{}\u{1b}\\", component_name, serialized_params);
    vte_parser.advance(&mut grid, dcs.as_bytes());
    grid
}

fn styled_cells_of(
    grid: &Grid,
    needle: char,
) -> Vec<crate::panes::terminal_character::CharacterStyles> {
    let mut styles = vec![];
    for line in grid.as_character_lines() {
        for terminal_character in line {
            if terminal_character.character == needle {
                styles.push(*terminal_character.styles);
            }
        }
    }
    styles
}

#[test]
fn disabled_ribbon_renders_italic_non_bold_unselected() {
    use crate::panes::terminal_character::AnsiCode;
    let grid = render_ui_component("ribbon", &format!("d{}", serialize_text_bytes("RES")));
    let letter_styles = styled_cells_of(&grid, 'R');
    assert!(!letter_styles.is_empty());
    for style in letter_styles {
        assert_eq!(style.italic, Some(AnsiCode::On));
        assert_ne!(style.bold, Some(AnsiCode::On));
    }
}

#[test]
fn disabled_text_with_opaque_flag_keeps_a_background() {
    use crate::panes::terminal_character::AnsiCode;
    let grid = render_ui_component("text", &format!("zd{}", serialize_text_bytes("RES")));
    let letter_styles = styled_cells_of(&grid, 'R');
    assert!(!letter_styles.is_empty());
    for style in letter_styles {
        assert_eq!(style.italic, Some(AnsiCode::On));
        assert_ne!(style.bold, Some(AnsiCode::On));
        assert!(style.background.is_some());
    }
}

#[test]
fn disabled_text_renders_italic_non_bold() {
    use crate::panes::terminal_character::AnsiCode;
    let grid = render_ui_component("text", &format!("d{}", serialize_text_bytes("RES")));
    let letter_styles = styled_cells_of(&grid, 'R');
    assert!(!letter_styles.is_empty());
    for style in letter_styles {
        assert_eq!(style.italic, Some(AnsiCode::On));
        assert_ne!(style.bold, Some(AnsiCode::On));
    }
}

#[test]
fn ui_component_flag_prefixes_parse_order_independently() {
    let baseline = render_ui_component("text", &format!("xzd{}", serialize_text_bytes("RES")));
    let baseline_styles = styled_cells_of(&baseline, 'R');
    for prefix in ["xdz", "zxd", "zdx", "dxz", "dzx"] {
        let grid = render_ui_component(
            "text",
            &format!("{}{}", prefix, serialize_text_bytes("RES")),
        );
        assert_eq!(
            styled_cells_of(&grid, 'R'),
            baseline_styles,
            "prefix {prefix} should parse the same flags as xzd"
        );
    }
}

use crate::panes::kitty_graphics::{InterceptorResult, KittyApcInterceptor, KittyHostSupport};
use crate::panes::sixel::PixelRect;

const KITTY_PNG_2X2: [u8; 75] = [
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02, 0x08, 0x06, 0x00, 0x00, 0x00, 0x72, 0xb6, 0x0d,
    0x24, 0x00, 0x00, 0x00, 0x12, 0x49, 0x44, 0x41, 0x54, 0x78, 0xda, 0x63, 0xf8, 0xcf, 0xc0, 0xf0,
    0x1f, 0x0c, 0x81, 0x34, 0x18, 0x00, 0x00, 0x49, 0xc8, 0x09, 0xf7, 0x03, 0xd9, 0x64, 0xf1, 0x00,
    0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
];

fn new_kitty_grid(rows: usize, columns: usize) -> (Grid, Rc<RefCell<KittyImageStore>>) {
    let kitty_image_store = Rc::new(RefCell::new(KittyImageStore::default()));
    let grid = Grid::new(
        rows,
        columns,
        Rc::new(RefCell::new(Palette::default())),
        Rc::new(RefCell::new(HashMap::new())),
        Rc::new(RefCell::new(LinkHandler::new())),
        Rc::new(RefCell::new(Some(SizeInPixels {
            width: 10,
            height: 20,
        }))),
        Rc::new(RefCell::new(SixelImageStore::default())),
        kitty_image_store.clone(),
        Style::default(),
        false,
        true,
        true,
        true,
        false,
    );
    (grid, kitty_image_store)
}

fn feed_kitty_bytes(
    grid: &mut Grid,
    vte_parser: &mut vte::Parser,
    interceptor: &mut KittyApcInterceptor,
    bytes: &[u8],
) {
    for byte in bytes {
        match interceptor.advance(*byte) {
            InterceptorResult::Forward(fwd) => {
                vte_parser.advance(grid, fwd.as_slice());
            },
            InterceptorResult::Swallow => {},
            InterceptorResult::Captured(cmd) => {
                grid.handle_kitty_apc(&cmd);
            },
        }
    }
}

fn kitty_apc(control: &str, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"\x1b_G");
    out.extend_from_slice(control.as_bytes());
    if !payload.is_empty() {
        out.push(b';');
        out.extend_from_slice(BASE64_ENCODER.encode(payload).as_bytes());
    }
    out.extend_from_slice(b"\x1b\\");
    out
}

fn rgb_raster(width: usize, height: usize) -> Vec<u8> {
    (0..width * height * 3).map(|i| (i % 251) as u8).collect()
}

fn rgba_raster(width: usize, height: usize) -> Vec<u8> {
    (0..width * height * 4).map(|i| (i % 251) as u8).collect()
}

fn kitty_placement_id_set(grid: &Grid) -> Vec<(u32, u32)> {
    let mut ids: Vec<(u32, u32)> = grid
        .kitty_placements()
        .iter()
        .map(|placement| (placement.image_id, placement.placement_id))
        .collect();
    ids.sort();
    ids
}

fn build_three_kitty_placements(
    grid: &mut Grid,
    vte_parser: &mut vte::Parser,
    interceptor: &mut KittyApcInterceptor,
) {
    feed_kitty_bytes(
        grid,
        vte_parser,
        interceptor,
        &kitty_apc("a=T,f=24,s=20,v=40,i=1,C=1", &rgb_raster(20, 40)),
    );
    feed_kitty_bytes(grid, vte_parser, interceptor, b"\x1b[6;11H");
    feed_kitty_bytes(
        grid,
        vte_parser,
        interceptor,
        &kitty_apc("a=T,f=24,s=20,v=40,i=2,z=1,C=1", &rgb_raster(20, 40)),
    );
    feed_kitty_bytes(grid, vte_parser, interceptor, b"\x1b[11;21H");
    feed_kitty_bytes(
        grid,
        vte_parser,
        interceptor,
        &kitty_apc("a=T,f=32,s=10,v=20,i=5,z=-1,C=1", &rgba_raster(10, 20)),
    );
    assert_eq!(grid.kitty_placement_count(), 3);
}

#[test]
fn kitty_transmit_and_display_raw_rgb() {
    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=20,v=40,i=1", &rgb_raster(20, 40)),
    );
    assert_eq!(kitty_image_store.borrow().image_count(), 1);
    assert_eq!(grid.kitty_placement_count(), 1);
    assert_eq!(
        grid.kitty_placements()[0].display_rect,
        PixelRect {
            x: 0,
            y: 0,
            width: 20,
            height: 40,
        }
    );
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn kitty_transmit_and_display_png() {
    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=100,i=1", &KITTY_PNG_2X2),
    );
    assert_eq!(kitty_image_store.borrow().image_count(), 1);
    assert_eq!(grid.kitty_placement_count(), 1);
    assert_eq!(
        grid.kitty_placements()[0].display_rect,
        PixelRect {
            x: 0,
            y: 0,
            width: 2,
            height: 2,
        }
    );
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn kitty_cursor_advances_after_display() {
    let (mut grid, _kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=20,v=40,i=1", &rgb_raster(20, 40)),
    );
    let (cursor_x, cursor_y, _) = grid.cursor_coordinates().unwrap();
    assert_eq!((cursor_x, cursor_y), (2, 1));
}

#[test]
fn kitty_cursor_unmoved_with_c1() {
    let (mut grid, _kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=20,v=40,i=1,C=1", &rgb_raster(20, 40)),
    );
    let (cursor_x, cursor_y, _) = grid.cursor_coordinates().unwrap();
    assert_eq!((cursor_x, cursor_y), (0, 0));
}

#[test]
fn kitty_placement_survives_text_overwrite_and_erase() {
    let (mut grid, _kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=20,v=40,i=1,C=1", &rgb_raster(20, 40)),
    );
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"XXXX");
    assert_eq!(grid.kitty_placement_count(), 1);
    for erase_sequence in [
        &b"\x1b[K"[..],
        &b"\x1b[1K"[..],
        &b"\x1b[0J"[..],
        &b"\x1b[1J"[..],
    ] {
        feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, erase_sequence);
        assert_eq!(grid.kitty_placement_count(), 1);
    }
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn kitty_replace_same_ids_moves_placement() {
    let (mut grid, _kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=20,v=40,i=1,p=1,C=1", &rgb_raster(20, 40)),
    );
    assert_snapshot!(format!("{:?}", grid));
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[10;5H");
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=p,i=1,p=1,C=1", b""),
    );
    assert_snapshot!(format!("{:?}", grid));
    assert_eq!(grid.kitty_placement_count(), 1);
    assert_eq!(
        grid.kitty_placements()[0].display_rect,
        PixelRect {
            x: 40,
            y: 180,
            width: 20,
            height: 40,
        }
    );
}

#[test]
fn kitty_delete_all_visible() {
    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    build_three_kitty_placements(&mut grid, &mut vte_parser, &mut interceptor);
    assert_eq!(kitty_image_store.borrow().total_bytes(), 7200);
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=d", b""),
    );
    assert_eq!(kitty_placement_id_set(&grid), vec![]);
    assert_eq!(kitty_image_store.borrow().total_bytes(), 7200);

    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    build_three_kitty_placements(&mut grid, &mut vte_parser, &mut interceptor);
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=d,d=A", b""),
    );
    assert_eq!(kitty_placement_id_set(&grid), vec![]);
    assert_eq!(kitty_image_store.borrow().image_count(), 0);
    assert_eq!(kitty_image_store.borrow().total_bytes(), 0);
}

#[test]
fn kitty_delete_by_image_id() {
    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    build_three_kitty_placements(&mut grid, &mut vte_parser, &mut interceptor);
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=d,d=i,i=2", b""),
    );
    assert_eq!(kitty_placement_id_set(&grid), vec![(1, 0), (5, 0)]);
    assert_eq!(kitty_image_store.borrow().total_bytes(), 7200);

    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=20,v=40,i=2,p=1,C=1", &rgb_raster(20, 40)),
    );
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=p,i=2,p=2,C=1", b""),
    );
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=d,d=i,i=2,p=1", b""),
    );
    assert_eq!(kitty_placement_id_set(&grid), vec![(2, 2)]);
    assert_eq!(kitty_image_store.borrow().total_bytes(), 3200);

    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    build_three_kitty_placements(&mut grid, &mut vte_parser, &mut interceptor);
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=d,d=I,i=2", b""),
    );
    assert_eq!(kitty_placement_id_set(&grid), vec![(1, 0), (5, 0)]);
    assert_eq!(kitty_image_store.borrow().image_count(), 2);
    assert_eq!(kitty_image_store.borrow().total_bytes(), 4000);
}

#[test]
fn kitty_delete_by_image_number() {
    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    build_three_kitty_placements(&mut grid, &mut vte_parser, &mut interceptor);
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[1;1H");
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=10,v=20,I=7,C=1", &rgb_raster(10, 20)),
    );
    assert_eq!(grid.kitty_placement_count(), 4);
    assert_eq!(kitty_image_store.borrow().total_bytes(), 8000);
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=d,d=n,I=7", b""),
    );
    assert_eq!(kitty_placement_id_set(&grid), vec![(1, 0), (2, 0), (5, 0)]);
    assert_eq!(kitty_image_store.borrow().total_bytes(), 8000);
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=d,d=N,I=7", b""),
    );
    assert_eq!(kitty_image_store.borrow().total_bytes(), 7200);
    assert_eq!(kitty_image_store.borrow().image_count(), 3);
}

#[test]
fn kitty_delete_cursor_intersecting() {
    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    build_three_kitty_placements(&mut grid, &mut vte_parser, &mut interceptor);
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=d,d=c", b""),
    );
    assert_eq!(kitty_placement_id_set(&grid), vec![(1, 0), (2, 0)]);
    assert_eq!(kitty_image_store.borrow().total_bytes(), 7200);

    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    build_three_kitty_placements(&mut grid, &mut vte_parser, &mut interceptor);
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=d,d=C", b""),
    );
    assert_eq!(kitty_placement_id_set(&grid), vec![(1, 0), (2, 0)]);
    assert_eq!(kitty_image_store.borrow().image_count(), 2);
    assert_eq!(kitty_image_store.borrow().total_bytes(), 6400);
}

#[test]
fn kitty_delete_cell_intersecting() {
    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    build_three_kitty_placements(&mut grid, &mut vte_parser, &mut interceptor);
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=d,d=p,x=11,y=6", b""),
    );
    assert_eq!(kitty_placement_id_set(&grid), vec![(1, 0), (5, 0)]);
    assert_eq!(kitty_image_store.borrow().total_bytes(), 7200);

    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    build_three_kitty_placements(&mut grid, &mut vte_parser, &mut interceptor);
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=d,d=P,x=11,y=6", b""),
    );
    assert_eq!(kitty_placement_id_set(&grid), vec![(1, 0), (5, 0)]);
    assert_eq!(kitty_image_store.borrow().total_bytes(), 4000);
}

#[test]
fn kitty_delete_cell_with_z() {
    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    build_three_kitty_placements(&mut grid, &mut vte_parser, &mut interceptor);
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=d,d=q,x=11,y=6,z=5", b""),
    );
    assert_eq!(kitty_placement_id_set(&grid), vec![(1, 0), (2, 0), (5, 0)]);
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=d,d=q,x=11,y=6,z=1", b""),
    );
    assert_eq!(kitty_placement_id_set(&grid), vec![(1, 0), (5, 0)]);
    assert_eq!(kitty_image_store.borrow().total_bytes(), 7200);

    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    build_three_kitty_placements(&mut grid, &mut vte_parser, &mut interceptor);
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=d,d=Q,x=11,y=6,z=1", b""),
    );
    assert_eq!(kitty_placement_id_set(&grid), vec![(1, 0), (5, 0)]);
    assert_eq!(kitty_image_store.borrow().total_bytes(), 4000);
}

#[test]
fn kitty_delete_column() {
    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    build_three_kitty_placements(&mut grid, &mut vte_parser, &mut interceptor);
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=d,d=x,x=1", b""),
    );
    assert_eq!(kitty_placement_id_set(&grid), vec![(2, 0), (5, 0)]);
    assert_eq!(kitty_image_store.borrow().total_bytes(), 7200);

    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    build_three_kitty_placements(&mut grid, &mut vte_parser, &mut interceptor);
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=d,d=X,x=1", b""),
    );
    assert_eq!(kitty_placement_id_set(&grid), vec![(2, 0), (5, 0)]);
    assert_eq!(kitty_image_store.borrow().total_bytes(), 4000);
}

#[test]
fn kitty_delete_row() {
    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    build_three_kitty_placements(&mut grid, &mut vte_parser, &mut interceptor);
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=d,d=y,y=6", b""),
    );
    assert_eq!(kitty_placement_id_set(&grid), vec![(1, 0), (5, 0)]);
    assert_eq!(kitty_image_store.borrow().total_bytes(), 7200);

    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    build_three_kitty_placements(&mut grid, &mut vte_parser, &mut interceptor);
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=d,d=Y,y=6", b""),
    );
    assert_eq!(kitty_placement_id_set(&grid), vec![(1, 0), (5, 0)]);
    assert_eq!(kitty_image_store.borrow().total_bytes(), 4000);
}

#[test]
fn kitty_delete_by_z_index() {
    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    build_three_kitty_placements(&mut grid, &mut vte_parser, &mut interceptor);
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=d,d=z,z=-1", b""),
    );
    assert_eq!(kitty_placement_id_set(&grid), vec![(1, 0), (2, 0)]);
    assert_eq!(kitty_image_store.borrow().total_bytes(), 7200);

    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    build_three_kitty_placements(&mut grid, &mut vte_parser, &mut interceptor);
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=d,d=Z,z=-1", b""),
    );
    assert_eq!(kitty_placement_id_set(&grid), vec![(1, 0), (2, 0)]);
    assert_eq!(kitty_image_store.borrow().total_bytes(), 6400);
}

#[test]
fn kitty_delete_image_id_range() {
    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    build_three_kitty_placements(&mut grid, &mut vte_parser, &mut interceptor);
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=d,d=r,x=1,y=2", b""),
    );
    assert_eq!(kitty_placement_id_set(&grid), vec![(5, 0)]);
    assert_eq!(kitty_image_store.borrow().total_bytes(), 7200);

    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    build_three_kitty_placements(&mut grid, &mut vte_parser, &mut interceptor);
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=d,d=R,x=1,y=2", b""),
    );
    assert_eq!(kitty_placement_id_set(&grid), vec![(5, 0)]);
    assert_eq!(kitty_image_store.borrow().image_count(), 1);
    assert_eq!(kitty_image_store.borrow().total_bytes(), 800);
}

#[test]
fn kitty_delete_image_id_range_with_omitted_lower_bound() {
    let (mut grid, _kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    build_three_kitty_placements(&mut grid, &mut vte_parser, &mut interceptor);
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=d,d=r,y=2", b""),
    );
    assert_eq!(kitty_placement_id_set(&grid), vec![(5, 0)]);
    let replies: Vec<String> = grid
        .pending_messages_to_pty
        .iter()
        .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
        .collect();
    assert!(
        !replies.iter().any(|reply| reply.contains("EINVAL")),
        "d=r with omitted lower bound must not emit EINVAL, got {:?}",
        replies
    );
}

#[test]
fn kitty_retransmit_to_existing_id_replaces_image() {
    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=32,s=4,v=4,i=1,C=1", &rgba_raster(4, 4)),
    );
    assert_eq!(grid.kitty_placement_count(), 1);
    assert_eq!(kitty_image_store.borrow().total_bytes(), 64);
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=t,f=32,s=2,v=2,i=1", &rgba_raster(2, 2)),
    );
    assert_eq!(grid.kitty_placement_count(), 0);
    assert_eq!(kitty_image_store.borrow().image_count(), 1);
    assert_eq!(kitty_image_store.borrow().total_bytes(), 16);
}

#[test]
fn kitty_c_r_scaling_produces_exact_cell_rect_and_variant() {
    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=10,v=10,c=3,r=2,i=1", &rgb_raster(10, 10)),
    );
    assert_eq!(grid.kitty_placement_count(), 1);
    let placement = &grid.kitty_placements()[0];
    assert_eq!(placement.dest_cells, (3, 2));
    assert_eq!(
        placement.display_rect,
        PixelRect {
            x: 0,
            y: 0,
            width: 30,
            height: 40,
        }
    );
    let internal_id = placement.internal_id;
    assert_eq!(
        kitty_image_store
            .borrow()
            .scaled_variant(internal_id, (3, 2))
            .unwrap()
            .len(),
        30 * 40 * 4
    );
    assert_snapshot!(format!("{:?}", grid));
}

#[test]
fn kitty_aspect_fit_single_axis() {
    let (mut grid, _kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=10,v=10,c=4,i=1", &rgb_raster(10, 10)),
    );
    assert_eq!(grid.kitty_placement_count(), 1);
    let placement = &grid.kitty_placements()[0];
    assert_eq!(placement.dest_cells, (4, 2));
    assert_eq!(
        placement.display_rect,
        PixelRect {
            x: 0,
            y: 0,
            width: 40,
            height: 40,
        }
    );
}

#[test]
fn kitty_yazi_kgpold_stream_roundtrip() {
    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    let raster = rgb_raster(2, 2);
    let full_b64 = BASE64_ENCODER.encode(&raster);
    let (b64_chunk_one, b64_chunk_two) = full_b64.split_at(8);
    let mut stream = Vec::new();
    stream.extend_from_slice(
        format!(
            "\x1b_Gq=2,a=T,z=-1,C=1,f=24,s=2,v=2,m=1;{}\x1b\\",
            b64_chunk_one
        )
        .as_bytes(),
    );
    stream.extend_from_slice(format!("\x1b_Gm=0;{}\x1b\\", b64_chunk_two).as_bytes());
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, &stream);
    assert_eq!(grid.kitty_placement_count(), 1);
    let placement = &grid.kitty_placements()[0];
    assert_eq!(placement.z_index, -1);
    assert_eq!(placement.image_id, 0);
    let (cursor_x, cursor_y, _) = grid.cursor_coordinates().unwrap();
    assert_eq!((cursor_x, cursor_y), (0, 0));
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        b"\x1b_Gq=2,a=d,d=A\x1b\\",
    );
    assert_eq!(grid.kitty_placement_count(), 0);
    assert_eq!(kitty_image_store.borrow().image_count(), 0);
}

#[test]
fn kitty_csi_2j_clears_placements() {
    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=20,v=40,i=1,C=1", &rgb_raster(20, 40)),
    );
    assert_eq!(grid.kitty_placement_count(), 1);
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[2J");
    assert_eq!(grid.kitty_placement_count(), 0);
    assert_eq!(kitty_image_store.borrow().image_count(), 0);
}

#[test]
fn kitty_ris_clears_placements_and_store() {
    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=20,v=40,i=1,C=1", &rgb_raster(20, 40)),
    );
    assert_eq!(grid.kitty_placement_count(), 1);
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1bc");
    assert_eq!(grid.kitty_placement_count(), 0);
    assert_eq!(kitty_image_store.borrow().image_count(), 0);
}

#[test]
fn kitty_image_in_alternate_buffer() {
    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=20,v=40,i=1,C=1", &rgb_raster(20, 40)),
    );
    assert_eq!(grid.kitty_placement_count(), 1);
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[?1049h");
    assert_eq!(grid.kitty_placement_count(), 0);
    assert_snapshot!(format!("{:?}", grid));
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=32,s=10,v=20,i=2,C=1", &rgba_raster(10, 20)),
    );
    assert_eq!(grid.kitty_placement_count(), 1);
    assert_eq!(kitty_image_store.borrow().image_count(), 2);
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[?1049l");
    assert_eq!(grid.kitty_placement_count(), 1);
    assert_snapshot!(format!("{:?}", grid));
    assert_eq!(
        kitty_image_store.borrow().image_count(),
        1,
        "alternate screen image was freed when leaving the alternate screen"
    );
}

#[test]
fn kitty_placements_are_reaped_when_scrolled_off() {
    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=20,v=40,i=1,C=1", &rgb_raster(20, 40)),
    );
    assert_eq!(grid.kitty_placement_count(), 1);
    for _ in 0..10_040 {
        grid.add_canonical_line();
    }
    assert_eq!(grid.kitty_placement_count(), 0);
    assert_eq!(kitty_image_store.borrow().image_count(), 0);
}

#[test]
fn kitty_scroll_region_moves_inside_placement_and_clips_straddler() {
    let (mut grid, _kitty_image_store) = new_kitty_grid(10, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[2;5r");
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[3;1H");
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=10,v=20,i=1,C=1", &rgb_raster(10, 20)),
    );
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[5;1H");
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=10,v=40,i=2,C=1", &rgb_raster(10, 40)),
    );
    assert_eq!(
        grid.kitty_placements()[0].display_rect,
        PixelRect {
            x: 0,
            y: 40,
            width: 10,
            height: 20,
        }
    );
    assert_eq!(
        grid.kitty_placements()[1].display_rect,
        PixelRect {
            x: 0,
            y: 80,
            width: 10,
            height: 40,
        }
    );
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\n");
    assert_eq!(grid.kitty_placement_count(), 2);
    let placement_a = &grid.kitty_placements()[0];
    assert_eq!(
        placement_a.display_rect,
        PixelRect {
            x: 0,
            y: 20,
            width: 10,
            height: 20,
        }
    );
    assert_eq!(placement_a.emit_y, 0);
    let placement_b = &grid.kitty_placements()[1];
    assert_eq!(
        placement_b.display_rect,
        PixelRect {
            x: 0,
            y: 100,
            width: 10,
            height: 20,
        }
    );
    assert_eq!(placement_b.emit_y, 20);
    assert_eq!(placement_b.emit_x, 0);
}

#[test]
fn kitty_reply_query_probe_exact_bytes() {
    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        b"\x1b_Ga=q,i=31,s=1,v=1,t=d,f=24;AAAA\x1b\\",
    );
    assert_eq!(
        grid.pending_messages_to_pty,
        vec![b"\x1b_Gi=31;OK\x1b\\".to_vec()]
    );
    assert_eq!(kitty_image_store.borrow().image_count(), 0);
    assert_eq!(grid.kitty_placement_count(), 0);
}

#[test]
fn kitty_reply_display_unknown_id_enoent() {
    let (mut grid, _kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=p,i=10", &[]),
    );
    assert_eq!(grid.pending_messages_to_pty.len(), 1);
    let reply = String::from_utf8(grid.pending_messages_to_pty[0].clone()).unwrap();
    assert!(reply.starts_with("\x1b_Gi=10;ENOENT"));
    assert!(reply.ends_with("\x1b\\"));
}

#[test]
fn kitty_reply_quiet_one_suppresses_ok_not_errors() {
    let (mut grid, _kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=2,v=2,i=1,q=1,C=1", &rgb_raster(2, 2)),
    );
    assert!(grid.pending_messages_to_pty.is_empty());
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=p,i=99,q=1", &[]),
    );
    assert_eq!(grid.pending_messages_to_pty.len(), 1);
    let reply = String::from_utf8(grid.pending_messages_to_pty[0].clone()).unwrap();
    assert!(reply.contains("i=99;ENOENT"));
}

#[test]
fn kitty_reply_quiet_two_suppresses_everything() {
    let (mut grid, _kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=2,v=2,i=1,q=2,C=1", &rgb_raster(2, 2)),
    );
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=p,i=99,q=2", &[]),
    );
    assert!(grid.pending_messages_to_pty.is_empty());
}

#[test]
fn kitty_reply_transmit_with_image_number_echoes_assigned_id() {
    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=t,I=5,f=24,s=2,v=2", &rgb_raster(2, 2)),
    );
    assert_eq!(
        grid.pending_messages_to_pty,
        vec![format!("\u{1b}_Gi={},I=5;OK\u{1b}\\", u32::MAX).into_bytes()]
    );
    assert_eq!(grid.kitty_placement_count(), 0);
    assert_eq!(kitty_image_store.borrow().image_count(), 1);
}

#[test]
fn kitty_reply_anonymous_transmit_and_display_is_silent() {
    let (mut grid, _kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=2,v=2,C=1", &rgb_raster(2, 2)),
    );
    assert!(grid.pending_messages_to_pty.is_empty());
    assert_eq!(grid.kitty_placement_count(), 1);
}

#[test]
fn kitty_reply_malformed_control_einval_without_ids() {
    let (mut grid, _kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        b"\x1b_Gnot-a-kv\x1b\\",
    );
    assert_eq!(
        grid.pending_messages_to_pty,
        vec![b"\x1b_G;EINVAL:malformed control data\x1b\\".to_vec()]
    );
}

#[test]
fn kitty_reply_icat_detection_sequence() {
    let (mut grid, _kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("icat-detect.bin");
    std::fs::write(&path, [1u8, 2, 3]).unwrap();
    let path_bytes = path.to_str().unwrap().as_bytes();
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        b"\x1b_Gt=d,a=q,i=1,s=1,v=1,f=24,S=3;MTIz\x1b\\",
    );
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        format!(
            "\x1b_Gt=t,a=q,i=2,s=1,v=1,f=24,S={};{}\x1b\\",
            path_bytes.len(),
            BASE64_ENCODER.encode(path_bytes)
        )
        .as_bytes(),
    );
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        format!(
            "\x1b_Gt=s,a=q,i=3,s=1,v=1,f=24,S=3;{}\x1b\\",
            BASE64_ENCODER.encode(b"/some-shm")
        )
        .as_bytes(),
    );
    assert_eq!(grid.pending_messages_to_pty.len(), 3);
    assert_eq!(
        grid.pending_messages_to_pty[0],
        b"\x1b_Gi=1;OK\x1b\\".to_vec()
    );
    assert_eq!(
        grid.pending_messages_to_pty[1],
        b"\x1b_Gi=2;OK\x1b\\".to_vec()
    );
    let third = String::from_utf8(grid.pending_messages_to_pty[2].clone()).unwrap();
    assert!(third.starts_with("\x1b_Gi=3;ENOTSUPPORTED"));
    assert!(third.ends_with("\x1b\\"));
    assert!(path.starts_with(std::env::temp_dir()));
    assert!(!path.exists());
}

#[test]
fn kitty_reply_query_when_host_unsupported() {
    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    grid.update_kitty_host_support(KittyHostSupport::Unsupported);
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        b"\x1b_Ga=q,i=31,s=1,v=1,t=d,f=24;AAAA\x1b\\",
    );
    assert_eq!(grid.pending_messages_to_pty.len(), 1);
    let reply = String::from_utf8(grid.pending_messages_to_pty[0].clone()).unwrap();
    assert!(reply.starts_with("\x1b_Gi=31;ENOTSUPPORTED"));
    assert!(reply.ends_with("\x1b\\"));
    assert_eq!(kitty_image_store.borrow().image_count(), 0);
}

#[test]
fn kitty_disabled_protocol_answers_queries_with_silence() {
    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    grid.update_kitty_host_support(KittyHostSupport::ProtocolDisabled);
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        b"\x1b_Ga=q,i=31,s=1,v=1,t=d,f=24;AAAA\x1b\\",
    );
    assert!(
        grid.pending_messages_to_pty.is_empty(),
        "a disabled protocol must not reply to queries at all"
    );
    assert_eq!(kitty_image_store.borrow().image_count(), 0);
}

#[test]
fn kitty_disabled_protocol_ignores_transmitted_images() {
    let (mut grid, kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    grid.update_kitty_host_support(KittyHostSupport::ProtocolDisabled);
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[3;1H");
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=20,v=20,i=1,C=1", &rgb_raster(20, 20)),
    );
    assert_eq!(grid.kitty_placement_count(), 0);
    assert_eq!(kitty_image_store.borrow().image_count(), 0);
    assert!(grid.pending_messages_to_pty.is_empty());
}

#[test]
fn kitty_disabled_protocol_does_not_leak_apc_bytes_into_the_grid() {
    let (mut grid, _kitty_image_store) = new_kitty_grid(20, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    grid.update_kitty_host_support(KittyHostSupport::ProtocolDisabled);
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"a");
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=20,v=20,i=1,C=1", &rgb_raster(20, 20)),
    );
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"b");
    assert_eq!(row_text(&grid.viewport[0]), "ab");
}

#[test]
fn kitty_conformance_image_put_cursor_positions() {
    let (mut grid, _kitty_image_store) = new_kitty_grid(5, 10);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=10,v=20,i=1", &rgb_raster(10, 20)),
    );
    assert_eq!(
        grid.cursor_coordinates().map(|(x, y, _)| (x, y)),
        Some((1, 0))
    );
}

#[test]
fn kitty_conformance_image_put_scaled_cursor() {
    let (mut grid, _kitty_image_store) = new_kitty_grid(5, 10);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=20,v=40,c=3,i=1", &rgb_raster(20, 40)),
    );
    assert_eq!(
        grid.cursor_coordinates().map(|(x, y, _)| (x, y)),
        Some((3, 2))
    );
    assert_eq!(grid.kitty_placements()[0].dest_cells, (3, 3));
}

#[test]
fn kitty_conformance_image_put_full_width_wraps() {
    let (mut grid, _kitty_image_store) = new_kitty_grid(5, 10);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=100,v=20,i=1", &rgb_raster(100, 20)),
    );
    assert_eq!(
        grid.cursor_coordinates().map(|(x, y, _)| (x, y)),
        Some((0, 1))
    );
}

#[test]
fn kitty_conformance_image_put_pixel_offsets_cursor() {
    let (mut grid, _kitty_image_store) = new_kitty_grid(5, 10);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=10,v=20,X=5,Y=5,i=1", &rgb_raster(10, 20)),
    );
    assert_eq!(
        grid.cursor_coordinates().map(|(x, y, _)| (x, y)),
        Some((2, 1))
    );
    assert_eq!(grid.kitty_placements()[0].dest_cells, (2, 2));
}

#[test]
fn kitty_conformance_bottom_row_placement_scrolls() {
    let (mut grid, _kitty_image_store) = new_kitty_grid(5, 10);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[5;1H");
    assert_eq!(grid.lines_above.len(), 0);
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=10,v=40,i=1", &rgb_raster(10, 40)),
    );
    assert_eq!(grid.lines_above.len(), 1);
    assert_eq!(
        grid.cursor_coordinates().map(|(x, y, _)| (x, y)),
        Some((1, 4))
    );
    assert_eq!(grid.kitty_placement_count(), 1);
    assert_eq!(
        grid.kitty_placements()[0].display_rect,
        PixelRect {
            x: 0,
            y: 80,
            width: 10,
            height: 40,
        }
    );
}

#[test]
fn kitty_conformance_bottom_row_placement_c1_does_not_scroll() {
    let (mut grid, _kitty_image_store) = new_kitty_grid(5, 10);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[5;1H");
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=10,v=40,i=1,C=1", &rgb_raster(10, 40)),
    );
    assert_eq!(grid.lines_above.len(), 0);
    assert_eq!(
        grid.cursor_coordinates().map(|(x, y, _)| (x, y)),
        Some((0, 4))
    );
    assert_eq!(
        grid.kitty_placements()[0].display_rect,
        PixelRect {
            x: 0,
            y: 80,
            width: 10,
            height: 40,
        }
    );
}

#[test]
fn kitty_conformance_full_screen_index_scroll_reaps_image() {
    let (mut grid, kitty_image_store) = new_kitty_grid(5, 10);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=10,v=20,i=1,C=1", &rgb_raster(10, 20)),
    );
    assert_eq!(kitty_image_store.borrow().image_count(), 1);
    let scroll_buffer_size = *SCROLL_BUFFER_SIZE.get().unwrap();
    for _ in 0..(scroll_buffer_size + grid.height) {
        grid.add_canonical_line();
    }
    let _ = grid.read_changes(0, 0);
    assert_eq!(grid.kitty_placement_count(), 0);
    assert_eq!(kitty_image_store.borrow().image_count(), 0);
}

#[test]
fn kitty_conformance_margin_scroll_leaves_outside_images_untouched() {
    let (mut grid, _kitty_image_store) = new_kitty_grid(5, 10);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[1;1H");
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=10,v=20,i=1,C=1", &rgb_raster(10, 20)),
    );
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[2;4r");
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[4;1H");
    let outside_before = grid.kitty_placements()[0].display_rect;
    for _ in 0..3 {
        feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\n");
    }
    assert_eq!(grid.kitty_placement_count(), 1);
    assert_eq!(grid.kitty_placements()[0].display_rect, outside_before);
    assert_eq!(grid.kitty_placements()[0].emit_y, 0);
}

#[test]
fn kitty_conformance_transmit_with_both_id_and_number_is_einval() {
    let (mut grid, kitty_image_store) = new_kitty_grid(5, 10);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=t,f=24,s=1,v=1,I=1,i=3", &rgb_raster(1, 1)),
    );
    assert_eq!(grid.kitty_placement_count(), 0);
    assert_eq!(kitty_image_store.borrow().image_count(), 0);
    assert_eq!(grid.pending_messages_to_pty.len(), 1);
    let reply = String::from_utf8(grid.pending_messages_to_pty[0].clone()).unwrap();
    assert!(
        reply.contains("EINVAL"),
        "expected EINVAL for transmit with both i and I, got {:?}",
        reply
    );
}

fn kitty_image_px_below_canonical_line(grid: &Grid, canonical_index: usize) -> isize {
    let placement = &grid.kitty_placements()[0];
    let cell_height = 20isize;
    let line_start = buffer_cell_row_of_canonical_line(grid, canonical_index);
    placement.display_rect.y - line_start * cell_height
}

#[test]
fn kitty_placement_keeps_screen_row_across_width_reflow() {
    let (mut grid, _kitty_image_store) = new_kitty_grid(6, 20);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    for line in 0..5 {
        let text = format!("row{}-{}", line, "x".repeat(30));
        feed_kitty_bytes(
            &mut grid,
            &mut vte_parser,
            &mut interceptor,
            text.as_bytes(),
        );
        feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\r\n");
    }
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[3;1H");
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=20,v=20,i=1,C=1", &rgb_raster(20, 20)),
    );
    let anchor_canonical_line = grid.kitty_placements()[0].vertical_anchor.canonical_line;
    let px_below_before = kitty_image_px_below_canonical_line(&grid, anchor_canonical_line);
    let lines_above_before = grid.lines_above.len();

    grid.change_size(6, 10);
    assert_ne!(
        grid.lines_above.len(),
        lines_above_before,
        "width reflow must change the scrollback row count for this test to be meaningful"
    );
    assert_eq!(grid.kitty_placement_count(), 1);
    assert_eq!(
        kitty_image_px_below_canonical_line(&grid, anchor_canonical_line),
        px_below_before,
        "image must keep the same pixel offset below its anchor canonical line (track the text) across a narrowing width reflow"
    );

    grid.change_size(6, 40);
    assert_eq!(grid.kitty_placement_count(), 1);
    assert_eq!(
        kitty_image_px_below_canonical_line(&grid, anchor_canonical_line),
        px_below_before,
        "image must keep the same pixel offset below its anchor canonical line (track the text) across a widening width reflow"
    );
}

fn kitty_image_absolute_cell_row(grid: &Grid) -> isize {
    let placement = &grid.kitty_placements()[0];
    let cell_height = 20isize;
    placement.display_rect.y.div_euclid(cell_height)
}

fn buffer_cell_row_of_canonical_line(grid: &Grid, canonical_index: usize) -> isize {
    let mut canonical_seen = 0usize;
    let mut wrapped_row = 0isize;
    for row in grid.lines_above.iter().chain(grid.viewport.iter()) {
        if row.is_canonical {
            if canonical_seen == canonical_index {
                return wrapped_row;
            }
            canonical_seen += 1;
        }
        wrapped_row += 1;
    }
    -1
}

#[test]
fn kitty_placement_follows_text_across_width_reflow() {
    let (mut grid, _kitty_image_store) = new_kitty_grid(10, 20);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    for line in 0..3 {
        let text = format!("r{}-{}", line, "x".repeat(34));
        feed_kitty_bytes(
            &mut grid,
            &mut vte_parser,
            &mut interceptor,
            text.as_bytes(),
        );
        feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\r\n");
    }
    let image_canonical_line = 3usize;
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[7;1H");
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=20,v=20,i=1,C=1", &rgb_raster(20, 20)),
    );

    let image_row_before = kitty_image_absolute_cell_row(&grid);
    let text_row_before = buffer_cell_row_of_canonical_line(&grid, image_canonical_line);
    assert_eq!(
        image_row_before, text_row_before,
        "image top must sit on the buffer cell row of its canonical line before reflow"
    );

    grid.change_size(10, 10);
    assert_eq!(grid.kitty_placement_count(), 1);
    let image_row_narrow = kitty_image_absolute_cell_row(&grid);
    let text_row_narrow = buffer_cell_row_of_canonical_line(&grid, image_canonical_line);
    assert_eq!(
        image_row_narrow, text_row_narrow,
        "image must track its canonical text line after narrowing reflow (lines above rewrap into more rows)"
    );
    assert!(
        text_row_narrow > text_row_before,
        "narrowing must push the image's canonical line further down the buffer for the test to be meaningful"
    );

    grid.change_size(10, 40);
    assert_eq!(grid.kitty_placement_count(), 1);
    let image_row_wide = kitty_image_absolute_cell_row(&grid);
    let text_row_wide = buffer_cell_row_of_canonical_line(&grid, image_canonical_line);
    assert_eq!(
        image_row_wide, text_row_wide,
        "image must track its canonical text line after widening reflow (lines above rewrap into fewer rows)"
    );
    assert!(
        text_row_wide < text_row_narrow,
        "widening must pull the image's canonical line back up for the test to be meaningful"
    );
}

#[test]
fn kitty_placement_moves_with_csi_insert_lines() {
    let (mut grid, _kitty_image_store) = new_kitty_grid(10, 20);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[6;1H");
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=20,v=20,i=1,C=1", &rgb_raster(20, 20)),
    );
    let row_before = kitty_image_absolute_cell_row(&grid);
    assert_eq!(row_before, 5);

    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[3;1H");
    let inserted = 2usize;
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[2L");

    assert_eq!(grid.kitty_placement_count(), 1);
    assert_eq!(
        kitty_image_absolute_cell_row(&grid),
        row_before + inserted as isize,
        "image below the insertion point must shift down by the inserted line count"
    );
}

#[test]
fn kitty_placement_moves_with_csi_delete_lines() {
    let (mut grid, _kitty_image_store) = new_kitty_grid(10, 20);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[6;1H");
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=20,v=20,i=1,C=1", &rgb_raster(20, 20)),
    );
    let row_before = kitty_image_absolute_cell_row(&grid);
    assert_eq!(row_before, 5);

    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[3;1H");
    let deleted = 2usize;
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[2M");

    assert_eq!(grid.kitty_placement_count(), 1);
    assert_eq!(
        kitty_image_absolute_cell_row(&grid),
        row_before - deleted as isize,
        "image below the deletion point must shift up by the deleted line count"
    );
}

#[test]
fn kitty_placement_in_deleted_range_is_removed() {
    let (mut grid, _kitty_image_store) = new_kitty_grid(10, 20);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[4;1H");
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=20,v=20,i=1,C=1", &rgb_raster(20, 20)),
    );
    assert_eq!(kitty_image_absolute_cell_row(&grid), 3);

    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[4;1H");
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[1M");

    assert_eq!(
        grid.kitty_placement_count(),
        0,
        "an image whose only row is deleted by CSI M must be removed"
    );
}

fn kitty_row_starts_the_marker_line(row: &super::super::Row) -> bool {
    row.is_canonical && row_text(row).starts_with("MARKER")
}

fn kitty_marker_is_below_the_viewport(grid: &Grid) -> bool {
    grid.lines_below
        .iter()
        .any(kitty_row_starts_the_marker_line)
}

fn kitty_marker_absolute_cell_row(grid: &Grid) -> Option<isize> {
    grid.lines_above
        .iter()
        .chain(grid.viewport.iter())
        .position(kitty_row_starts_the_marker_line)
        .map(|row| row as isize)
}

fn kitty_marker_extended_absolute_cell_row(grid: &Grid) -> Option<isize> {
    grid.lines_above
        .iter()
        .chain(grid.viewport.iter())
        .chain(grid.lines_below.iter())
        .position(kitty_row_starts_the_marker_line)
        .map(|row| row as isize)
}

fn kitty_scrollback_has_merged_wrapped_rows(grid: &Grid) -> bool {
    grid.lines_above.iter().any(|row| row.width() > grid.width)
}

fn kitty_marker_viewport_row(grid: &Grid) -> Option<usize> {
    let marker_row = kitty_marker_absolute_cell_row(grid)?;
    let viewport_row = marker_row - grid.lines_above.len() as isize;
    if viewport_row >= 0 && (viewport_row as usize) < grid.viewport.len() {
        Some(viewport_row as usize)
    } else {
        None
    }
}

#[test]
fn kitty_placement_tracks_text_when_a_wrapped_row_moves_into_the_scrollback() {
    let (mut grid, _kitty_image_store) = new_kitty_grid(5, 20);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        b"AAAAAAAAAAAAAAAAAAAAAAAAA\r\n",
    );
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"MARKER");
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[3;1H");
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=20,v=20,i=1,C=1", &rgb_raster(20, 20)),
    );
    assert_eq!(
        Some(kitty_image_absolute_cell_row(&grid)),
        kitty_marker_absolute_cell_row(&grid)
    );
    for _ in 0..5 {
        feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\r\nx");
    }
    assert!(kitty_scrollback_has_merged_wrapped_rows(&grid));
    assert_eq!(
        Some(kitty_image_absolute_cell_row(&grid)),
        kitty_marker_absolute_cell_row(&grid),
        "the image must stay on the row of its marker line after a wrapped row is merged into the scrollback"
    );
}

fn kitty_wrapped_scrollback_setup(
    wrapped_line_width: usize,
) -> (
    Grid,
    Rc<RefCell<KittyImageStore>>,
    vte::Parser,
    KittyApcInterceptor,
) {
    let (mut grid, kitty_image_store) = new_kitty_grid(5, 20);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    let wrapped_line = format!("{}\r\n", "A".repeat(wrapped_line_width));
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        wrapped_line.as_bytes(),
    );
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"MARKER");
    let marker_viewport_row = kitty_marker_viewport_row(&grid).expect("marker must be visible");
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        format!("\x1b[{};1H", marker_viewport_row + 1).as_bytes(),
    );
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=20,v=20,i=1,C=1", &rgb_raster(20, 20)),
    );
    assert_eq!(
        Some(kitty_image_absolute_cell_row(&grid)),
        kitty_marker_absolute_cell_row(&grid),
        "the image must start on the row of its marker line"
    );
    for _ in 0..5 {
        feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\r\nx");
    }
    assert!(
        kitty_scrollback_has_merged_wrapped_rows(&grid),
        "the wrapped line must have been merged into a single scrollback row"
    );
    (grid, kitty_image_store, vte_parser, interceptor)
}

#[test]
fn kitty_placement_tracks_text_when_a_wrapped_row_is_split_out_of_the_scrollback() {
    let (mut grid, _kitty_image_store, _vte_parser, _interceptor) =
        kitty_wrapped_scrollback_setup(25);
    for step in 0..2 {
        grid.scroll_up_one_line();
        assert_eq!(
            grid.kitty_placement_count(),
            1,
            "scrolling the scrollback must not free the image (step {})",
            step
        );
        assert_eq!(
            Some(kitty_image_absolute_cell_row(&grid)),
            kitty_marker_absolute_cell_row(&grid),
            "the image must stay on the row of its marker line after scrollback scroll up (step {})",
            step
        );
    }
    assert!(
        !grid.viewport[0].is_canonical,
        "the wrapped scrollback row must have been split back into the viewport"
    );
}

#[test]
fn kitty_placement_tracks_text_when_a_multiply_wrapped_row_is_split_out_of_the_scrollback() {
    let (mut grid, _kitty_image_store, _vte_parser, _interceptor) =
        kitty_wrapped_scrollback_setup(45);
    let mut non_canonical_viewport_rows = 0;
    for step in 0..4 {
        grid.scroll_up_one_line();
        assert_eq!(
            grid.kitty_placement_count(),
            1,
            "scrolling the scrollback must not free the image (step {})",
            step
        );
        assert_eq!(
            Some(kitty_image_absolute_cell_row(&grid)),
            kitty_marker_absolute_cell_row(&grid),
            "the image must stay on the row of its marker line after scrollback scroll up (step {})",
            step
        );
        non_canonical_viewport_rows = grid.viewport.iter().filter(|row| !row.is_canonical).count();
    }
    assert!(
        non_canonical_viewport_rows >= 2,
        "a canonical line wrapping into three rows must be split into at least two continuation rows in the viewport"
    );
}

#[test]
fn kitty_placement_geometry_survives_a_scrollback_scroll_round_trip() {
    let (mut grid, _kitty_image_store, _vte_parser, _interceptor) =
        kitty_wrapped_scrollback_setup(45);
    let display_rect_before = grid.kitty_placements()[0].display_rect;
    let anchor_before = grid.kitty_placements()[0].vertical_anchor;
    let lines_above_before = grid.lines_above.len();
    let marker_row_before = kitty_marker_absolute_cell_row(&grid);
    for _ in 0..4 {
        grid.scroll_up_one_line();
    }
    for _ in 0..4 {
        grid.scroll_down_one_line();
    }
    assert_eq!(grid.lines_above.len(), lines_above_before);
    assert_eq!(
        kitty_marker_absolute_cell_row(&grid),
        marker_row_before,
        "the round trip must restore the marker line row"
    );
    assert_eq!(
        grid.kitty_placements()[0].display_rect,
        display_rect_before,
        "the round trip must restore the placement pixel geometry"
    );
    assert_eq!(
        grid.kitty_placements()[0].vertical_anchor,
        anchor_before,
        "the round trip must restore the placement anchor"
    );
}

fn kitty_marker_canonical_line_index(grid: &Grid) -> Option<usize> {
    let mut canonical_lines_seen = 0usize;
    for row in grid.lines_above.iter().chain(grid.viewport.iter()) {
        if row.is_canonical {
            canonical_lines_seen += 1;
        }
        if row_text(row) == "MARKER" {
            return Some(canonical_lines_seen.saturating_sub(1));
        }
    }
    None
}

#[test]
fn kitty_placement_anchor_survives_scroll_buffer_front_eviction() {
    let (mut grid, _kitty_image_store) = new_kitty_grid(10, 20);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    let scroll_buffer_size = *SCROLL_BUFFER_SIZE.get().unwrap();
    for _ in 0..(scroll_buffer_size + grid.height) {
        grid.add_canonical_line();
    }
    assert_eq!(
        grid.lines_above.len(),
        scroll_buffer_size,
        "the scroll buffer must be full for this test to be meaningful"
    );
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[1;1H");
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"MARKER");
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[1;1H");
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=20,v=20,i=1,C=1", &rgb_raster(20, 20)),
    );
    assert_eq!(
        Some(kitty_image_absolute_cell_row(&grid)),
        kitty_marker_absolute_cell_row(&grid)
    );
    assert_eq!(
        grid.kitty_placements()[0].vertical_anchor.canonical_line,
        kitty_marker_canonical_line_index(&grid).expect("marker must exist")
    );
    let front_drops_before = grid.kitty_grid.front_drops();

    grid.change_size(5, 20);

    let evicted_lines = grid.kitty_grid.front_drops() - front_drops_before;
    assert!(
        evicted_lines >= 5,
        "shrinking the viewport must evict rows from the front of a full scroll buffer"
    );
    assert_eq!(
        grid.kitty_placement_count(),
        1,
        "a placement below the evicted rows must survive the eviction"
    );
    let marker_canonical_line =
        kitty_marker_canonical_line_index(&grid).expect("marker must still exist");
    assert_eq!(
        grid.kitty_placements()[0].vertical_anchor.canonical_line,
        marker_canonical_line,
        "the placement anchor must still resolve to its marker text line after front eviction"
    );
    assert_eq!(
        Some(kitty_image_absolute_cell_row(&grid)),
        kitty_marker_absolute_cell_row(&grid),
        "the image must stay on the row of its marker line across scroll buffer front eviction"
    );
}

fn kitty_marker_with_image_on_first_row(
    rows: usize,
    columns: usize,
) -> (
    Grid,
    Rc<RefCell<KittyImageStore>>,
    vte::Parser,
    KittyApcInterceptor,
) {
    let (mut grid, kitty_image_store) = new_kitty_grid(rows, columns);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[1;1H");
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"MARKER");
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[1;1H");
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=20,v=20,i=1,C=1", &rgb_raster(20, 20)),
    );
    assert_eq!(grid.kitty_placement_count(), 1);
    assert_eq!(kitty_marker_absolute_cell_row(&grid), Some(0));
    assert_eq!(kitty_image_absolute_cell_row(&grid), 0);
    (grid, kitty_image_store, vte_parser, interceptor)
}

#[test]
fn kitty_top_anchored_scroll_up_keeps_image_in_scrollback() {
    let (mut grid, kitty_image_store, mut vte_parser, mut interceptor) =
        kitty_marker_with_image_on_first_row(5, 20);
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[S");
    assert_eq!(
        grid.lines_above.len(),
        1,
        "a top anchored scroll region moves the top row into the scrollback"
    );
    assert_eq!(
        grid.kitty_placement_count(),
        1,
        "an image whose text was preserved into the scrollback must not be freed"
    );
    assert_eq!(kitty_image_store.borrow().image_count(), 1);
    assert_eq!(kitty_marker_absolute_cell_row(&grid), Some(0));
    assert_eq!(
        kitty_image_absolute_cell_row(&grid),
        0,
        "the image must follow its text into the scrollback"
    );
    assert_eq!(grid.kitty_placements()[0].display_rect.height, 20);
    assert_eq!(grid.kitty_placements()[0].emit_y, 0);

    grid.scroll_up_one_line();
    assert_eq!(grid.lines_above.len(), 0);
    assert_eq!(kitty_marker_absolute_cell_row(&grid), Some(0));
    assert_eq!(
        kitty_image_absolute_cell_row(&grid),
        0,
        "the image must still sit on its marker row when scrolled back into view"
    );
}

#[test]
fn kitty_top_anchored_delete_line_keeps_image_in_scrollback() {
    let (mut grid, kitty_image_store, mut vte_parser, mut interceptor) =
        kitty_marker_with_image_on_first_row(5, 20);
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[1;1H");
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[1M");
    assert_eq!(
        grid.lines_above.len(),
        1,
        "a top anchored line deletion moves the deleted row into the scrollback"
    );
    assert_eq!(
        grid.kitty_placement_count(),
        1,
        "an image whose text was preserved into the scrollback must not be freed"
    );
    assert_eq!(kitty_image_store.borrow().image_count(), 1);
    assert_eq!(kitty_marker_absolute_cell_row(&grid), Some(0));
    assert_eq!(
        kitty_image_absolute_cell_row(&grid),
        0,
        "the image must follow its text into the scrollback"
    );

    grid.scroll_up_one_line();
    assert_eq!(kitty_marker_absolute_cell_row(&grid), Some(0));
    assert_eq!(
        kitty_image_absolute_cell_row(&grid),
        0,
        "the image must still sit on its marker row when scrolled back into view"
    );
}

#[test]
fn kitty_non_top_anchored_region_scroll_still_frees_image() {
    let (mut grid, kitty_image_store) = new_kitty_grid(5, 20);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[2;5r");
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[2;1H");
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=20,v=20,i=1,C=1", &rgb_raster(20, 20)),
    );
    assert_eq!(kitty_image_absolute_cell_row(&grid), 1);
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[S");
    assert_eq!(
        grid.lines_above.len(),
        0,
        "a region that does not start at the top of the viewport discards its top row"
    );
    assert_eq!(
        grid.kitty_placement_count(),
        0,
        "an image scrolled out of a non top anchored region must be freed"
    );
    assert_eq!(kitty_image_store.borrow().image_count(), 0);
}

#[test]
fn kitty_alternate_screen_top_anchored_scroll_frees_image() {
    let (mut grid, kitty_image_store) = new_kitty_grid(5, 20);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[?1049h");
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[1;1H");
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=20,v=20,i=1,C=1", &rgb_raster(20, 20)),
    );
    assert_eq!(kitty_image_absolute_cell_row(&grid), 0);
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[S");
    assert_eq!(
        grid.lines_above.len(),
        0,
        "the alternate screen has no scrollback"
    );
    assert_eq!(
        grid.kitty_placement_count(),
        0,
        "an image scrolled off the alternate screen must be freed"
    );
    assert_eq!(kitty_image_store.borrow().image_count(), 0);
}

#[test]
fn kitty_placement_rejoins_its_text_after_editing_while_scrolled_back() {
    let (mut grid, _kitty_image_store) = new_kitty_grid(10, 40);
    let mut vte_parser = vte::Parser::new();
    let mut interceptor = KittyApcInterceptor::new();
    for line in 0..20 {
        feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[10;1H");
        let text = format!("\r\npad{}", line);
        feed_kitty_bytes(
            &mut grid,
            &mut vte_parser,
            &mut interceptor,
            text.as_bytes(),
        );
    }
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        b"\x1b[10;1H\r\nMARKER",
    );
    feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[10;1H");
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        &kitty_apc("a=T,f=24,s=20,v=20,i=1,C=1", &rgb_raster(20, 20)),
    );
    for line in 0..3 {
        feed_kitty_bytes(&mut grid, &mut vte_parser, &mut interceptor, b"\x1b[10;1H");
        let text = format!("\r\ntail{}", line);
        feed_kitty_bytes(
            &mut grid,
            &mut vte_parser,
            &mut interceptor,
            text.as_bytes(),
        );
    }
    let _ = grid.read_changes(0, 0);
    for _ in 0..8 {
        grid.scroll_up_one_line();
    }
    let _ = grid.read_changes(0, 0);
    assert!(
        kitty_marker_is_below_the_viewport(&grid),
        "the marker line must be scrolled below the viewport for this scenario"
    );
    feed_kitty_bytes(
        &mut grid,
        &mut vte_parser,
        &mut interceptor,
        b"\x1b[1;1H\x1b[1M",
    );
    let _ = grid.read_changes(0, 0);
    grid.reset_viewport();
    let _ = grid.read_changes(0, 0);
    assert!(
        !kitty_marker_is_below_the_viewport(&grid),
        "the marker line must be back inside the buffer after restoring the viewport"
    );
    let marker_row = kitty_marker_extended_absolute_cell_row(&grid)
        .expect("the marker line must survive the edit");
    assert_eq!(
        grid.kitty_placement_count(),
        1,
        "the image must survive an edit made while scrolled back"
    );
    assert_eq!(
        kitty_image_absolute_cell_row(&grid),
        marker_row,
        "the image must rejoin its marker line once the viewport is restored"
    );
}

#[test]
fn primary_device_attributes_advertise_sixel_when_host_supports_it() {
    let mut grid = create_grid_with_content("");
    let mut vte_parser = vte::Parser::new();
    grid.update_sixel_host_support(true);
    vte_parser.advance(&mut grid, b"\x1b[c");
    assert_eq!(
        grid.pending_messages_to_pty,
        vec![b"\x1b[?62;4;52c".to_vec()]
    );
}

#[test]
fn primary_device_attributes_omit_sixel_when_host_does_not_support_it() {
    let mut grid = create_grid_with_content("");
    let mut vte_parser = vte::Parser::new();
    grid.update_sixel_host_support(false);
    vte_parser.advance(&mut grid, b"\x1b[c");
    assert_eq!(grid.pending_messages_to_pty, vec![b"\x1b[?62;52c".to_vec()]);
}

#[test]
fn xtsmgraphics_color_registers_report_failure_when_host_does_not_support_sixel() {
    let mut grid = create_grid_with_content("");
    let mut vte_parser = vte::Parser::new();
    grid.update_sixel_host_support(false);
    vte_parser.advance(&mut grid, b"\x1b[?1;1S");
    assert_eq!(grid.pending_messages_to_pty, vec![b"\x1b[?1;3;0S".to_vec()]);
}

#[test]
fn xtsmgraphics_geometry_reports_failure_when_host_does_not_support_sixel() {
    let mut grid = create_grid_with_content("");
    let mut vte_parser = vte::Parser::new();
    grid.update_sixel_host_support(false);
    vte_parser.advance(&mut grid, b"\x1b[?2;1S");
    assert_eq!(grid.pending_messages_to_pty, vec![b"\x1b[?2;3;0S".to_vec()]);
}

#[test]
fn osc_52_read_is_forwarded_to_the_host() {
    let mut grid = new_grid_for_forwarding_test();
    let mut vte_parser = vte::Parser::new();
    vte_parser.advance(&mut grid, b"\x1b]52;c;?\x07");
    assert!(
        grid.pending_messages_to_pty.is_empty(),
        "clipboard reads must never be answered locally"
    );
    assert_eq!(
        grid.pending_forwarded_queries,
        vec![crate::host_query::HostQuery::ClipboardContent {
            selection: 'c',
            terminator: crate::host_query::OscTerminator::Bel,
        }]
    );
}

#[test]
fn osc_52_read_keeps_the_selection_and_terminator_of_the_query() {
    let mut grid = new_grid_for_forwarding_test();
    let mut vte_parser = vte::Parser::new();
    vte_parser.advance(&mut grid, b"\x1b]52;p;?\x1b\\");
    assert_eq!(
        grid.pending_forwarded_queries,
        vec![crate::host_query::HostQuery::ClipboardContent {
            selection: 'p',
            terminator: crate::host_query::OscTerminator::St,
        }]
    );
}

#[test]
fn osc_52_write_is_not_forwarded() {
    let mut grid = new_grid_for_forwarding_test();
    let mut vte_parser = vte::Parser::new();
    vte_parser.advance(&mut grid, b"\x1b]52;c;aGVsbG8=\x07");
    assert!(
        grid.pending_forwarded_queries.is_empty(),
        "copying to the clipboard must not go through the forward path"
    );
    assert_eq!(grid.pending_clipboard_update.as_deref(), Some("hello"));
}

#[test]
fn xtgettcap_answers_the_ms_capability() {
    let mut grid = create_grid_with_content("");
    let mut vte_parser = vte::Parser::new();
    vte_parser.advance(&mut grid, b"\x1bP+q4d73\x1b\\");
    assert_eq!(
        grid.pending_messages_to_pty,
        vec![b"\x1bP1+r4D73=1B5D35323B25703125733B257032257307\x1b\\".to_vec()]
    );
}

#[test]
fn xtgettcap_rejects_unknown_capabilities() {
    let mut grid = create_grid_with_content("");
    let mut vte_parser = vte::Parser::new();
    vte_parser.advance(&mut grid, b"\x1bP+q62656c\x1b\\");
    assert_eq!(
        grid.pending_messages_to_pty,
        vec![b"\x1bP0+r\x1b\\".to_vec()]
    );
}

#[test]
fn xtgettcap_answers_each_requested_capability() {
    let mut grid = create_grid_with_content("");
    let mut vte_parser = vte::Parser::new();
    vte_parser.advance(&mut grid, b"\x1bP+q62656c;4d73\x1b\\");
    assert_eq!(grid.pending_messages_to_pty.len(), 2);
    assert_eq!(grid.pending_messages_to_pty[0], b"\x1bP0+r\x1b\\".to_vec());
    assert!(String::from_utf8_lossy(&grid.pending_messages_to_pty[1]).starts_with("\x1bP1+r4D73="));
}

#[test]
fn xtgettcap_rejects_malformed_hex() {
    let mut grid = create_grid_with_content("");
    let mut vte_parser = vte::Parser::new();
    vte_parser.advance(&mut grid, b"\x1bP+q4d7\x1b\\");
    assert_eq!(
        grid.pending_messages_to_pty,
        vec![b"\x1bP0+r\x1b\\".to_vec()]
    );
}

#[test]
fn sixel_dcs_is_not_confused_with_xtgettcap() {
    let mut grid = new_grid_for_forwarding_test();
    let mut vte_parser = vte::Parser::new();
    vte_parser.advance(&mut grid, b"\x1bPq#0;2;0;0;0\x1b\\");
    assert!(
        grid.pending_messages_to_pty.is_empty(),
        "a sixel image must not produce an XTGETTCAP reply"
    );
}

#[test]
fn ech_over_wide_character_preserves_background_on_padding_cell() {
    use crate::panes::terminal_character::NamedColor;
    // 使用宽字符来测试宽单元格的替换。
    assert_eq!(crate::panes::TerminalCharacter::new('Ａ').width(), 2);

    let content = "\x1b[44mＡ\x1b[1;1H\x1b[1X".as_bytes();
    let grid = create_grid_with_size_and_raw(2, 4, content);

    let lines = grid.as_character_lines();

    assert_eq!(lines[0][0].character, ' ');
    assert_eq!(lines[0][1].character, ' ');

    assert_eq!(
        lines[0][0].styles.background,
        Some(crate::panes::terminal_character::AnsiCode::NamedColor(
            NamedColor::Blue
        ))
    );
    assert_eq!(
        lines[0][1].styles.background,
        Some(crate::panes::terminal_character::AnsiCode::NamedColor(
            NamedColor::Blue
        ))
    );
}

#[test]
fn osc_9_notification_is_parsed() {
    use crate::panes::grid::PendingNotification;
    let mut grid = new_grid_for_forwarding_test();
    let mut vte_parser = vte::Parser::new();
    vte_parser.advance(&mut grid, b"\x1b]9;the build finished\x07");
    assert_eq!(
        grid.pending_desktop_notifications,
        vec![PendingNotification::Osc9 {
            body: "the build finished".to_owned()
        }]
    );
}

#[test]
fn osc_9_notification_with_semicolons_keeps_the_whole_body() {
    use crate::panes::grid::PendingNotification;
    let mut grid = new_grid_for_forwarding_test();
    let mut vte_parser = vte::Parser::new();
    vte_parser.advance(&mut grid, b"\x1b]9;done: 1;2;3\x07");
    assert_eq!(
        grid.pending_desktop_notifications,
        vec![PendingNotification::Osc9 {
            body: "done: 1;2;3".to_owned()
        }]
    );
}

#[test]
fn an_empty_osc_9_notification_is_ignored() {
    let mut grid = new_grid_for_forwarding_test();
    let mut vte_parser = vte::Parser::new();
    vte_parser.advance(&mut grid, b"\x1b]9;\x07");
    assert!(grid.pending_desktop_notifications.is_empty());
}

#[test]
fn osc_777_notification_is_parsed() {
    use crate::panes::grid::PendingNotification;
    let mut grid = new_grid_for_forwarding_test();
    let mut vte_parser = vte::Parser::new();
    vte_parser.advance(&mut grid, b"\x1b]777;notify;the title;the body\x07");
    assert_eq!(
        grid.pending_desktop_notifications,
        vec![PendingNotification::Osc777 {
            title: "the title".to_owned(),
            body: "the body".to_owned()
        }]
    );
}

#[test]
fn an_osc_777_notification_body_may_contain_semicolons() {
    use crate::panes::grid::PendingNotification;
    let mut grid = new_grid_for_forwarding_test();
    let mut vte_parser = vte::Parser::new();
    vte_parser.advance(&mut grid, b"\x1b]777;notify;title;a;b;c\x07");
    assert_eq!(
        grid.pending_desktop_notifications,
        vec![PendingNotification::Osc777 {
            title: "title".to_owned(),
            body: "a;b;c".to_owned()
        }]
    );
}

#[test]
fn an_osc_777_non_notify_subcommand_is_ignored() {
    let mut grid = new_grid_for_forwarding_test();
    let mut vte_parser = vte::Parser::new();
    vte_parser.advance(&mut grid, b"\x1b]777;precmd;something\x07");
    assert!(grid.pending_desktop_notifications.is_empty());
}

#[test]
fn an_osc_777_notification_without_a_body_is_kept() {
    use crate::panes::grid::PendingNotification;
    let mut grid = new_grid_for_forwarding_test();
    let mut vte_parser = vte::Parser::new();
    vte_parser.advance(&mut grid, b"\x1b]777;notify;just a title\x07");
    assert_eq!(
        grid.pending_desktop_notifications,
        vec![PendingNotification::Osc777 {
            title: "just a title".to_owned(),
            body: String::new()
        }]
    );
}

#[test]
fn a_notification_carries_its_title_and_body_across_protocols() {
    use crate::panes::grid::PendingNotification;
    assert_eq!(
        PendingNotification::Osc9 {
            body: "body".to_owned()
        }
        .title_and_body(),
        (String::new(), "body".to_owned())
    );
    assert_eq!(
        PendingNotification::Osc777 {
            title: "title".to_owned(),
            body: "body".to_owned()
        }
        .title_and_body(),
        ("title".to_owned(), "body".to_owned())
    );
    assert_eq!(
        PendingNotification::Osc99 {
            payload: "i=1;the title".to_owned(),
            terminator: "\u{7}".to_owned(),
            wants_report: false,
            display: Some(("the title".to_owned(), "the body".to_owned())),
        }
        .title_and_body(),
        ("the title".to_owned(), "the body".to_owned())
    );
}

fn osc99_display(grid: &Grid, index: usize) -> Option<(String, String)> {
    use crate::panes::grid::PendingNotification;
    match grid.pending_desktop_notifications.get(index) {
        Some(PendingNotification::Osc99 { display, .. }) => display.clone(),
        other => panic!("expected an OSC 99 notification, got: {:?}", other),
    }
}

#[test]
fn an_osc_9_conemu_progress_report_is_not_a_notification() {
    let mut grid = new_grid_for_forwarding_test();
    let mut vte_parser = vte::Parser::new();
    vte_parser.advance(&mut grid, b"\x1b]9;4;0;0\x1b\\");
    vte_parser.advance(&mut grid, b"\x1b]9;4;1;50\x07");
    assert!(
        grid.pending_desktop_notifications.is_empty(),
        "progress reports are not desktop notifications, got: {:?}",
        grid.pending_desktop_notifications
    );
}

#[test]
fn other_osc_9_conemu_subcommands_are_not_notifications() {
    let mut grid = new_grid_for_forwarding_test();
    let mut vte_parser = vte::Parser::new();
    vte_parser.advance(&mut grid, b"\x1b]9;9;/home/user\x07");
    vte_parser.advance(&mut grid, b"\x1b]9;3;a tab title\x07");
    assert!(
        grid.pending_desktop_notifications.is_empty(),
        "ConEmu subcommands are not desktop notifications, got: {:?}",
        grid.pending_desktop_notifications
    );
}

#[test]
fn an_osc_9_notification_starting_with_a_number_is_still_a_notification() {
    use crate::panes::grid::PendingNotification;
    let mut grid = new_grid_for_forwarding_test();
    let mut vte_parser = vte::Parser::new();
    vte_parser.advance(&mut grid, b"\x1b]9;4 tests failed;in 2 crates\x07");
    assert_eq!(
        grid.pending_desktop_notifications,
        vec![PendingNotification::Osc9 {
            body: "4 tests failed;in 2 crates".to_owned()
        }]
    );
}

#[test]
fn an_osc_99_notification_is_assembled_from_its_chunks() {
    let mut grid = new_grid_for_forwarding_test();
    let mut vte_parser = vte::Parser::new();
    vte_parser.advance(&mut grid, b"\x1b]99;i=1:d=0;Hello world\x1b\\");
    vte_parser.advance(&mut grid, b"\x1b]99;i=1:p=body;This is cool\x1b\\");
    assert_eq!(
        osc99_display(&grid, 0),
        None,
        "an unfinished notification has nothing to show yet"
    );
    assert_eq!(
        osc99_display(&grid, 1),
        Some(("Hello world".to_owned(), "This is cool".to_owned())),
        "the finished notification carries all of its chunks"
    );
}

#[test]
fn an_osc_99_notification_payload_may_be_base64_encoded() {
    let mut grid = new_grid_for_forwarding_test();
    let mut vte_parser = vte::Parser::new();
    vte_parser.advance(&mut grid, b"\x1b]99;i=1:e=1;dGhlIGJ1aWxkIGZpbmlzaGVk\x1b\\");
    assert_eq!(
        osc99_display(&grid, 0),
        Some(("the build finished".to_owned(), String::new()))
    );
}

#[test]
fn osc_99_requests_that_display_nothing_are_not_downgraded() {
    let mut grid = new_grid_for_forwarding_test();
    let mut vte_parser = vte::Parser::new();
    vte_parser.advance(&mut grid, b"\x1b]99;i=1:p=close;\x1b\\");
    vte_parser.advance(&mut grid, b"\x1b]99;i=2:p=?;\x1b\\");
    vte_parser.advance(&mut grid, b"\x1b]99;i=3:p=alive;\x1b\\");
    assert_eq!(osc99_display(&grid, 0), None, "a close shows nothing");
    assert_eq!(osc99_display(&grid, 1), None, "a query shows nothing");
    assert_eq!(
        osc99_display(&grid, 2),
        None,
        "a liveness poll shows nothing"
    );
}

#[test]
fn the_report_intent_of_an_osc_99_notification_is_remembered() {
    use crate::panes::grid::PendingNotification;
    let mut grid = new_grid_for_forwarding_test();
    let mut vte_parser = vte::Parser::new();
    vte_parser.advance(&mut grid, b"\x1b]99;i=1:a=report;the build finished\x1b\\");
    vte_parser.advance(&mut grid, b"\x1b]99;i=1:p=close;\x1b\\");
    let wants_report: Vec<bool> = grid
        .pending_desktop_notifications
        .iter()
        .map(|notification| match notification {
            PendingNotification::Osc99 { wants_report, .. } => *wants_report,
            other => panic!("expected an OSC 99 notification, got: {:?}", other),
        })
        .collect();
    assert_eq!(
        wants_report,
        vec![true, true],
        "a close request inherits the intent of the notification it closes"
    );
}

#[test]
fn an_explicit_action_replaces_the_remembered_report_intent() {
    use crate::panes::grid::PendingNotification;
    let mut grid = new_grid_for_forwarding_test();
    let mut vte_parser = vte::Parser::new();
    vte_parser.advance(&mut grid, b"\x1b]99;i=1:a=report;the build started\x1b\\");
    vte_parser.advance(&mut grid, b"\x1b]99;i=1:a=focus;the build finished\x1b\\");
    vte_parser.advance(&mut grid, b"\x1b]99;i=1:p=close;\x1b\\");
    let wants_report: Vec<bool> = grid
        .pending_desktop_notifications
        .iter()
        .map(|notification| match notification {
            PendingNotification::Osc99 { wants_report, .. } => *wants_report,
            other => panic!("expected an OSC 99 notification, got: {:?}", other),
        })
        .collect();
    assert_eq!(
        wants_report,
        vec![true, false, false],
        "an escape that states its own action decides, and is remembered from then on"
    );
}

#[test]
fn an_osc_99_notification_is_finished_by_a_payload_it_cannot_show() {
    let mut grid = new_grid_for_forwarding_test();
    let mut vte_parser = vte::Parser::new();
    vte_parser.advance(&mut grid, b"\x1b]99;i=1:d=0;the build finished\x1b\\");
    vte_parser.advance(&mut grid, b"\x1b]99;i=1:p=buttons:d=1;retry\x1b\\");
    assert_eq!(
        osc99_display(&grid, 1),
        Some(("the build finished".to_owned(), String::new())),
        "the assembled text is shown even when the last chunk carries buttons"
    );
}

#[test]
fn the_notification_state_remembered_per_pane_is_bounded() {
    let mut grid = new_grid_for_forwarding_test();
    let mut vte_parser = vte::Parser::new();
    for index in 0..1000 {
        vte_parser.advance(
            &mut grid,
            format!("\x1b]99;i={}:a=report;notification\x1b\\", index).as_bytes(),
        );
    }
    assert!(
        grid.notification_tracker.known_ids.len() <= 256
            && grid.notification_tracker.wants_report.len() <= 256
            && grid.notification_tracker.assemblies.is_empty(),
        "an app sending endless notifications does not grow the pane's state without bound"
    );
}

#[test]
fn a_single_notification_being_assembled_is_bounded() {
    let mut grid = new_grid_for_forwarding_test();
    let mut vte_parser = vte::Parser::new();
    let chunk = "\u{5efa}".repeat(300);
    for _ in 0..100 {
        vte_parser.advance(
            &mut grid,
            format!("\x1b]99;i=1:d=0;{}\x1b\\", chunk).as_bytes(),
        );
    }

    let assembled_bytes = grid
        .notification_tracker
        .assemblies
        .get("1")
        .map(|assembly| assembly.title.len())
        .unwrap_or(0);
    assert!(
        assembled_bytes <= 4096,
        "an app streaming an endless single notification does not grow the pane's state without \
         bound, got {} bytes",
        assembled_bytes
    );

    vte_parser.advance(&mut grid, b"\x1b]99;i=1:d=1;\x1b\\");
    let last_notification = grid.pending_desktop_notifications.len() - 1;
    let (title, _body) =
        osc99_display(&grid, last_notification).expect("the truncated notification is still shown");
    assert_eq!(
        title,
        "\u{5efa}".repeat(1365),
        "the assembled text is truncated on a character boundary"
    );
}

fn rendered_row(grid: &Grid, row_index: usize) -> String {
    grid.viewport[row_index]
        .columns
        .iter()
        .map(|terminal_character| terminal_character.character)
        .collect()
}

fn cursor_position(grid: &Grid) -> Option<(usize, usize)> {
    grid.cursor_coordinates().map(|(x, y, _)| (x, y))
}

#[test]
fn cjk_characters_occupy_two_columns_each() {
    let grid = create_grid_with_content("\u{4f60}\u{597d}\u{4e16}\u{754c}");

    let row = &grid.viewport[0];
    assert_eq!(row.columns.len(), 4);
    assert!(row
        .columns
        .iter()
        .all(|terminal_character| terminal_character.width() == 2));
    assert_eq!(row.width(), 8);
    assert_eq!(cursor_position(&grid), Some((8, 0)));
}

#[test]
fn kana_and_hangul_syllables_occupy_two_columns_each() {
    let grid = create_grid_with_content("\u{30c6}\u{30ad}\u{d55c}\u{ad6d}");

    let row = &grid.viewport[0];
    assert_eq!(row.columns.len(), 4);
    assert_eq!(row.width(), 8);
    assert_eq!(cursor_position(&grid), Some((8, 0)));
}

#[test]
fn zwj_emoji_sequence_keeps_one_wide_cell_per_emoji() {
    let grid =
        create_grid_with_content("\u{1f468}\u{200d}\u{1f469}\u{200d}\u{1f467}\u{200d}\u{1f466}");

    let row = &grid.viewport[0];
    assert_eq!(
        rendered_row(&grid, 0),
        "\u{1f468}\u{1f469}\u{1f467}\u{1f466}"
    );
    assert_eq!(row.width(), 8);
    assert_eq!(cursor_position(&grid), Some((8, 0)));
}

#[test]
fn skin_tone_modifier_occupies_its_own_two_columns() {
    let grid = create_grid_with_content("\u{1f44b}\u{1f3fd}");

    let row = &grid.viewport[0];
    assert_eq!(row.columns.len(), 2);
    assert!(row
        .columns
        .iter()
        .all(|terminal_character| terminal_character.width() == 2));
    assert_eq!(row.width(), 4);
    assert_eq!(cursor_position(&grid), Some((4, 0)));
}

#[test]
fn variation_selector_and_zero_width_joiner_do_not_advance_the_cursor() {
    let grid = create_grid_with_content("\u{26a0}\u{fe0f}\u{200d}\u{200b}");

    assert_eq!(rendered_row(&grid, 0), "\u{26a0}");
    assert_eq!(grid.viewport[0].width(), 1);
    assert_eq!(cursor_position(&grid), Some((1, 0)));
}

#[test]
fn combining_marks_are_dropped_and_do_not_advance_the_cursor() {
    let grid = create_grid_with_content("e\u{301}a\u{300}\u{308}o\u{331}");

    assert_eq!(rendered_row(&grid, 0), "eao");
    assert_eq!(grid.viewport[0].width(), 3);
    assert_eq!(cursor_position(&grid), Some((3, 0)));
}

#[test]
fn precomposed_and_decomposed_forms_occupy_the_same_number_of_columns() {
    let precomposed = create_grid_with_content("\u{e9}cole");
    let decomposed = create_grid_with_content("e\u{301}cole");

    assert_eq!(precomposed.viewport[0].width(), 5);
    assert_eq!(decomposed.viewport[0].width(), 5);
    assert_eq!(cursor_position(&precomposed), cursor_position(&decomposed));
}

#[test]
fn ambiguous_width_characters_occupy_one_column_each() {
    let grid = create_grid_with_content("\u{b1}\u{b0}\u{2192}\u{3b1}\u{203b}\u{d7}\u{f7}");

    let row = &grid.viewport[0];
    assert_eq!(row.columns.len(), 7);
    assert!(row
        .columns
        .iter()
        .all(|terminal_character| terminal_character.width() == 1));
    assert_eq!(row.width(), 7);
    assert_eq!(cursor_position(&grid), Some((7, 0)));
}

#[test]
fn box_drawing_characters_occupy_one_column_each() {
    let grid = create_grid_with_content(
        "\u{250c}\u{2500}\u{252c}\u{2510}\u{2502}\u{251c}\u{253c}\u{2524}\u{2514}\u{2534}\u{2518}\u{2554}\u{2550}\u{2557}\u{2588}\u{2589}",
    );

    let row = &grid.viewport[0];
    assert_eq!(row.columns.len(), 16);
    assert!(row
        .columns
        .iter()
        .all(|terminal_character| terminal_character.width() == 1));
    assert_eq!(row.width(), 16);
    assert_eq!(cursor_position(&grid), Some((16, 0)));
}

#[test]
fn a_wide_character_that_does_not_fit_the_last_column_wraps_whole() {
    let grid = create_grid_with_size_and_raw(4, 6, "abcde\u{4e16}".as_bytes());

    assert_eq!(rendered_row(&grid, 0), "abcde");
    assert_eq!(grid.viewport[0].width(), 5);
    assert_eq!(rendered_row(&grid, 1), "\u{4e16}");
    assert_eq!(cursor_position(&grid), Some((2, 1)));
}

#[test]
fn a_wide_character_ending_exactly_on_the_last_column_stays_on_its_row() {
    let grid = create_grid_with_size_and_raw(4, 6, "abcd\u{4e16}".as_bytes());

    assert_eq!(rendered_row(&grid, 0), "abcd\u{4e16}");
    assert_eq!(grid.viewport[0].width(), 6);
    assert_eq!(grid.viewport.len(), 1);
    assert_eq!(cursor_position(&grid), None);
}

#[test]
fn ech_over_the_leading_half_of_a_wide_character_blanks_both_columns() {
    let grid =
        create_grid_with_size_and_raw(3, 10, "\u{4e16}\u{754c}abc\u{1b}[1;1H\u{1b}[1X".as_bytes());

    let lines = grid.as_character_lines();
    assert_eq!(lines[0][0].character, ' ');
    assert_eq!(lines[0][1].character, ' ');
    assert_eq!(lines[0][2].character, '\u{754c}');
    assert_eq!(lines[0][3].character, 'a');
    assert_eq!(lines[0][4].character, 'b');
    assert_eq!(lines[0][5].character, 'c');
}

#[test]
fn ech_over_a_wide_character_keeps_the_background_of_the_padding_cell() {
    use crate::panes::terminal_character::NamedColor;

    let grid = create_grid_with_size_and_raw(
        3,
        10,
        "\u{1b}[44m\u{4e16}\u{754c}\u{1b}[1;1H\u{1b}[1X".as_bytes(),
    );

    let lines = grid.as_character_lines();
    assert_eq!(
        lines[0][0].styles.background,
        Some(crate::panes::terminal_character::AnsiCode::NamedColor(
            NamedColor::Blue
        ))
    );
    assert_eq!(
        lines[0][1].styles.background,
        Some(crate::panes::terminal_character::AnsiCode::NamedColor(
            NamedColor::Blue
        ))
    );
}

#[test]
fn cursor_forward_past_content_pads_the_row_before_a_wide_character() {
    let grid =
        create_grid_with_size_and_raw(6, 20, "\u{1b}[2B\r\u{1b}[4C\u{4e16}\u{754c}".as_bytes());

    assert_eq!(rendered_row(&grid, 2), "    \u{4e16}\u{754c}");
    assert_eq!(grid.viewport[2].width(), 8);
    assert_eq!(cursor_position(&grid), Some((8, 2)));
}

#[test]
fn yijing_and_trigram_symbols_are_wide_under_the_new_width_tables() {
    let grid = create_grid_with_content("\u{2630}\u{2637}\u{268a}\u{4dc0}\u{4dff}");

    let row = &grid.viewport[0];
    assert_eq!(row.columns.len(), 5);
    assert!(row
        .columns
        .iter()
        .all(|terminal_character| terminal_character.width() == 2));
    assert_eq!(row.width(), 10);
    assert_eq!(cursor_position(&grid), Some((10, 0)));
}

#[test]
fn halfwidth_katakana_sound_marks_are_zero_width_under_the_new_width_tables() {
    let grid = create_grid_with_content("\u{ff76}\u{ff9e}\u{ff8a}\u{ff9f}");

    assert_eq!(rendered_row(&grid, 0), "\u{ff76}\u{ff8a}");
    assert_eq!(grid.viewport[0].width(), 2);
    assert_eq!(cursor_position(&grid), Some((2, 0)));
}

#[test]
fn soft_hyphen_and_hangul_filler_are_zero_width_under_the_new_width_tables() {
    let grid = create_grid_with_content("a\u{ad}b\u{3164}c");

    assert_eq!(rendered_row(&grid, 0), "abc");
    assert_eq!(grid.viewport[0].width(), 3);
    assert_eq!(cursor_position(&grid), Some((3, 0)));
}

#[test]
fn a_character_wider_than_two_columns_advances_the_cursor_by_its_full_width() {
    let grid = create_grid_with_content("\u{17d8}x");

    let row = &grid.viewport[0];
    assert_eq!(row.columns.len(), 2);
    assert_eq!(row.columns[0].width(), 3);
    assert_eq!(row.width(), 4);
    assert_eq!(cursor_position(&grid), Some((4, 0)));
}
