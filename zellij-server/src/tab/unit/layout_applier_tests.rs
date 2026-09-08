use crate::os_input_output::AsyncReader;
use crate::panes::kitty_graphics::KittyImageStore;
use crate::panes::sixel::SixelImageStore;
use crate::panes::{FloatingPanes, TiledPanes};
use crate::panes::{LinkHandler, PaneId};
use crate::plugins::PluginInstruction;
use crate::pty::PtyInstruction;
use crate::tab::layout_applier::LayoutApplier;
use crate::{os_input_output::ServerOsApi, thread_bus::ThreadSenders, ClientId};
use insta::assert_snapshot;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::path::PathBuf;
use std::rc::Rc;
use zellij_utils::input::options::PaneFrameStyle;

use interprocess::local_socket::Stream as LocalSocketStream;
use zellij_utils::{
    channels::{self, ChannelWithContext, Receiver, SenderWithContext},
    data::{ModeInfo, Palette, Style},
    errors::prelude::*,
    input::command::{RunCommand, TerminalAction},
    input::layout::RunPluginOrAlias,
    input::layout::{FloatingPaneLayout, Layout, Run, TiledPaneLayout},
    ipc::{ClientToServerMsg, IpcReceiverWithContext, ServerToClientMsg},
    pane_size::{Size, SizeInPixels, Viewport},
};

#[derive(Clone)]
struct FakeInputOutput {}

impl ServerOsApi for FakeInputOutput {
    fn set_terminal_size_using_terminal_id(
        &self,
        _id: u32,
        _cols: u16,
        _rows: u16,
        _width_in_pixels: Option<u16>,
        _height_in_pixels: Option<u16>,
    ) -> Result<()> {
        Ok(())
    }

    fn spawn_terminal(
        &self,
        _file_to_open: TerminalAction,
        _quit_cb: Box<dyn Fn(PaneId, Option<i32>, RunCommand) + Send>,
        _default_editor: Option<PathBuf>,
    ) -> Result<(u32, Box<dyn AsyncReader>, Option<u32>)> {
        unimplemented!()
    }

    fn write_to_tty_stdin(&self, _id: u32, _buf: &[u8]) -> Result<usize> {
        unimplemented!()
    }

    fn tcdrain(&self, _id: u32) -> Result<()> {
        unimplemented!()
    }

    fn kill(&self, _pid: u32) -> Result<()> {
        unimplemented!()
    }

    fn force_kill(&self, _pid: u32) -> Result<()> {
        unimplemented!()
    }

    fn box_clone(&self) -> Box<dyn ServerOsApi> {
        Box::new((*self).clone())
    }

    fn send_to_client(&self, _client_id: ClientId, _msg: ServerToClientMsg) -> Result<()> {
        unimplemented!()
    }

    fn new_client(
        &mut self,
        _client_id: ClientId,
        _stream: LocalSocketStream,
    ) -> Result<IpcReceiverWithContext<ClientToServerMsg>> {
        unimplemented!()
    }
    fn new_client_with_reply(
        &mut self,
        _client_id: ClientId,
        _stream: LocalSocketStream,
        _reply_stream: LocalSocketStream,
    ) -> Result<IpcReceiverWithContext<ClientToServerMsg>> {
        unimplemented!()
    }

    fn remove_client(&mut self, _client_id: ClientId) -> Result<()> {
        unimplemented!()
    }

    fn load_palette(&self) -> Palette {
        unimplemented!()
    }

    fn get_cwd(&self, _pid: u32) -> Option<PathBuf> {
        unimplemented!()
    }

    fn write_to_file(&mut self, _buf: String, _name: Option<String>) -> Result<()> {
        unimplemented!()
    }

    fn re_run_command_in_terminal(
        &self,
        _terminal_id: u32,
        _run_command: RunCommand,
        _quit_cb: Box<dyn Fn(PaneId, Option<i32>, RunCommand) + Send>,
    ) -> Result<(Box<dyn AsyncReader>, Option<u32>)> {
        unimplemented!()
    }

    fn clear_terminal_id(&self, _terminal_id: u32) -> Result<()> {
        unimplemented!()
    }

    fn send_sigint(&self, _pid: u32) -> Result<()> {
        unimplemented!()
    }
}

/// 解析 KDL 布局 string and 提取 平铺 and 浮动 布局
fn parse_kdl_layout(kdl_str: &str) -> (TiledPaneLayout, Vec<FloatingPaneLayout>) {
    let layout = Layout::from_kdl(kdl_str, Some("test_layout".into()), None, None)
        .expect("Failed to parse KDL layout");
    layout.new_tab()
}

/// Creates all the fixtures needed for LayoutApplier 测试
#[allow(clippy::type_complexity)]
fn create_layout_applier_fixtures(
    size: Size,
) -> (
    Rc<RefCell<Viewport>>,
    ThreadSenders,
    Rc<RefCell<SixelImageStore>>,
    Rc<RefCell<LinkHandler>>,
    Rc<RefCell<Palette>>,
    Rc<RefCell<HashMap<usize, String>>>,
    Rc<RefCell<Option<SizeInPixels>>>,
    Rc<RefCell<HashMap<ClientId, bool>>>,
    Style,
    Rc<RefCell<Size>>,
    TiledPanes,
    FloatingPanes,
    PaneFrameStyle,
    Option<PaneId>,
    Box<dyn ServerOsApi>,
    bool,
    bool,
    bool,
    bool,
    bool,
) {
    let viewport = Rc::new(RefCell::new(Viewport {
        x: 0,
        y: 0,
        rows: size.rows,
        cols: size.cols,
    }));

    let (mock_plugin_sender, _mock_plugin_receiver) = channels::unbounded();
    let mut senders = ThreadSenders::default().silently_fail_on_send();
    senders.replace_to_plugin(SenderWithContext::new(mock_plugin_sender));
    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    let terminal_emulator_colors = Rc::new(RefCell::new(Palette::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let character_cell_size = Rc::new(RefCell::new(None));

    let client_id = 1;
    let mut connected_clients_map = HashMap::new();
    connected_clients_map.insert(client_id, false);
    let connected_clients = Rc::new(RefCell::new(connected_clients_map));

    let style = Style::default();
    let display_area = Rc::new(RefCell::new(size));

    let os_api = Box::new(FakeInputOutput {});

    // 创建 TiledPanes
    let connected_clients_set = Rc::new(RefCell::new(HashSet::from([client_id])));
    let mode_info = Rc::new(RefCell::new(HashMap::new()));
    let stacked_resize = Rc::new(RefCell::new(false));
    let reserved_top_rows = Rc::new(RefCell::new(HashMap::new()));
    let fullscreen_covers_ui = Rc::new(RefCell::new(false));
    let session_is_mirrored = true;
    let draw_pane_frames = PaneFrameStyle::Full;
    let default_mode_info = ModeInfo::default();

    let tiled_panes = TiledPanes::new(
        display_area.clone(),
        viewport.clone(),
        connected_clients_set.clone(),
        connected_clients.clone(),
        mode_info.clone(),
        character_cell_size.clone(),
        stacked_resize,
        reserved_top_rows,
        fullscreen_covers_ui.clone(),
        session_is_mirrored,
        draw_pane_frames,
        default_mode_info.clone(),
        style.clone(),
        os_api.box_clone(),
        senders.clone(),
    );

    // 创建 FloatingPanes
    let floating_panes = FloatingPanes::new(
        display_area.clone(),
        viewport.clone(),
        connected_clients_set,
        connected_clients.clone(),
        mode_info,
        character_cell_size.clone(),
        fullscreen_covers_ui,
        draw_pane_frames,
        session_is_mirrored,
        default_mode_info,
        style.clone(),
        os_api.box_clone(),
        senders.clone(),
    );

    let focus_pane_id = None;
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;

    (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        tiled_panes,
        floating_panes,
        draw_pane_frames,
        focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    )
}

/// Creates fixtures with 接收者 for 验证 消息 sent to pty and 插件 线程
#[allow(clippy::type_complexity)]
fn create_layout_applier_fixtures_with_receivers(
    size: Size,
) -> (
    Rc<RefCell<Viewport>>,
    ThreadSenders,
    Rc<RefCell<SixelImageStore>>,
    Rc<RefCell<LinkHandler>>,
    Rc<RefCell<Palette>>,
    Rc<RefCell<HashMap<usize, String>>>,
    Rc<RefCell<Option<SizeInPixels>>>,
    Rc<RefCell<HashMap<ClientId, bool>>>,
    Style,
    Rc<RefCell<Size>>,
    TiledPanes,
    FloatingPanes,
    PaneFrameStyle,
    Option<PaneId>,
    Box<dyn ServerOsApi>,
    bool,
    bool,
    bool,
    bool,
    bool,
    Receiver<(PtyInstruction, zellij_utils::errors::ErrorContext)>,
    Receiver<(PluginInstruction, zellij_utils::errors::ErrorContext)>,
) {
    let viewport = Rc::new(RefCell::new(Viewport {
        x: 0,
        y: 0,
        rows: size.rows,
        cols: size.cols,
    }));

    let (mock_pty_sender, mock_pty_receiver): ChannelWithContext<PtyInstruction> =
        channels::unbounded();
    let (mock_plugin_sender, mock_plugin_receiver): ChannelWithContext<PluginInstruction> =
        channels::unbounded();

    let mut senders = ThreadSenders::default().silently_fail_on_send();
    senders.replace_to_pty(SenderWithContext::new(mock_pty_sender));
    senders.replace_to_plugin(SenderWithContext::new(mock_plugin_sender));

    let sixel_image_store = Rc::new(RefCell::new(SixelImageStore::default()));
    let link_handler = Rc::new(RefCell::new(LinkHandler::new()));
    let terminal_emulator_colors = Rc::new(RefCell::new(Palette::default()));
    let terminal_emulator_color_codes = Rc::new(RefCell::new(HashMap::new()));
    let character_cell_size = Rc::new(RefCell::new(None));

    let client_id = 1;
    let mut connected_clients_map = HashMap::new();
    connected_clients_map.insert(client_id, false);
    let connected_clients = Rc::new(RefCell::new(connected_clients_map));

    let style = Style::default();
    let display_area = Rc::new(RefCell::new(size));

    let os_api = Box::new(FakeInputOutput {});

    // 创建 TiledPanes
    let connected_clients_set = Rc::new(RefCell::new(HashSet::from([client_id])));
    let mode_info = Rc::new(RefCell::new(HashMap::new()));
    let stacked_resize = Rc::new(RefCell::new(false));
    let reserved_top_rows = Rc::new(RefCell::new(HashMap::new()));
    let fullscreen_covers_ui = Rc::new(RefCell::new(false));
    let session_is_mirrored = true;
    let draw_pane_frames = PaneFrameStyle::Full;
    let default_mode_info = ModeInfo::default();

    let tiled_panes = TiledPanes::new(
        display_area.clone(),
        viewport.clone(),
        connected_clients_set.clone(),
        connected_clients.clone(),
        mode_info.clone(),
        character_cell_size.clone(),
        stacked_resize,
        reserved_top_rows,
        fullscreen_covers_ui.clone(),
        session_is_mirrored,
        draw_pane_frames,
        default_mode_info.clone(),
        style.clone(),
        os_api.box_clone(),
        senders.clone(),
    );

    // 创建 FloatingPanes
    let floating_panes = FloatingPanes::new(
        display_area.clone(),
        viewport.clone(),
        connected_clients_set,
        connected_clients.clone(),
        mode_info,
        character_cell_size.clone(),
        fullscreen_covers_ui,
        draw_pane_frames,
        session_is_mirrored,
        default_mode_info,
        style.clone(),
        os_api.box_clone(),
        senders.clone(),
    );

    let focus_pane_id = None;
    let debug = false;
    let arrow_fonts = true;
    let styled_underlines = true;
    let osc8_hyperlinks = true;
    let explicitly_disable_kitty_keyboard_protocol = false;

    (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        tiled_panes,
        floating_panes,
        draw_pane_frames,
        focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        mock_pty_receiver,
        mock_plugin_receiver,
    )
}

/// Takes a 快照 of the 当前 窗格 状态 for 断言
fn take_pane_state_snapshot(
    tiled_panes: &TiledPanes,
    floating_panes: &FloatingPanes,
    focus_pane_id: &Option<PaneId>,
    viewport: &Rc<RefCell<Viewport>>,
    display_area: &Rc<RefCell<Size>>,
) -> String {
    let mut output = String::new();

    // 视口 info
    let viewport_state = viewport.borrow();
    writeln!(
        &mut output,
        "VIEWPORT: x={}, y={}, cols={}, rows={}",
        viewport_state.x, viewport_state.y, viewport_state.cols, viewport_state.rows
    )
    .unwrap();

    let display_state = display_area.borrow();
    writeln!(
        &mut output,
        "DISPLAY: cols={}, rows={}",
        display_state.cols, display_state.rows
    )
    .unwrap();

    // 焦点 状态
    writeln!(&mut output, "FOCUS: {:?}", focus_pane_id).unwrap();

    writeln!(&mut output).unwrap();

    // 平铺 窗格
    writeln!(&mut output, "TILED PANES ({})", tiled_panes.panes.len()).unwrap();
    let mut tiled_list: Vec<_> = tiled_panes.get_panes().collect();
    tiled_list.sort_by_key(|(id, _)| **id);

    for (pane_id, pane) in tiled_list {
        let geom = pane.position_and_size();
        let run = pane.invoked_with();
        let selectable = pane.selectable();

        writeln!(&mut output, "  {:?}:", pane_id).unwrap();
        writeln!(
            &mut output,
            "    geom: x={}, y={}, cols={}, rows={}",
            geom.x,
            geom.y,
            geom.cols.as_usize(),
            geom.rows.as_usize()
        )
        .unwrap();

        if let Some(logical_pos) = geom.logical_position {
            writeln!(&mut output, "    logical_position: {}", logical_pos).unwrap();
        }

        if let Some(stack_id) = geom.stacked {
            writeln!(&mut output, "    stacked: {}", stack_id).unwrap();
        }

        writeln!(&mut output, "    run: {}", format_run_instruction(run)).unwrap();

        writeln!(&mut output, "    selectable: {}", selectable).unwrap();
        writeln!(&mut output, "    title: {}", pane.current_title()).unwrap();
        writeln!(&mut output, "    borderless: {}", pane.borderless()).unwrap();

        writeln!(&mut output).unwrap();
    }

    // 浮动 窗格
    if floating_panes.pane_ids().count() > 0 {
        writeln!(
            &mut output,
            "FLOATING PANES ({})",
            floating_panes.pane_ids().count()
        )
        .unwrap();
        let mut floating_list: Vec<_> = floating_panes.get_panes().collect();
        floating_list.sort_by_key(|(id, _)| **id);

        for (pane_id, pane) in floating_list {
            let geom = pane.position_and_size();
            let run = pane.invoked_with();

            writeln!(&mut output, "  {:?}:", pane_id).unwrap();
            writeln!(
                &mut output,
                "    geom: x={}, y={}, cols={}, rows={}",
                geom.x,
                geom.y,
                geom.cols.as_usize(),
                geom.rows.as_usize()
            )
            .unwrap();

            if let Some(logical_pos) = geom.logical_position {
                writeln!(&mut output, "    logical_position: {}", logical_pos).unwrap();
            }

            writeln!(&mut output, "    run: {}", format_run_instruction(run)).unwrap();
            writeln!(&mut output, "    pinned: {}", geom.is_pinned).unwrap();
            writeln!(&mut output, "    selectable: {}", pane.selectable()).unwrap();
            writeln!(&mut output, "    title: {}", pane.current_title()).unwrap();
            writeln!(&mut output, "    borderless: {}", pane.borderless()).unwrap();
            writeln!(&mut output).unwrap();
        }
    }

    output
}

/// 格式 a Run instruction as a human-readable string
fn format_run_instruction(run: &Option<Run>) -> String {
    match run {
        None => "None".to_string(),
        Some(Run::Command(cmd)) => {
            let mut s = format!("Command({})", cmd.command.display());
            if !cmd.args.is_empty() {
                s.push_str(&format!(" args={:?}", cmd.args));
            }
            if let Some(cwd) = &cmd.cwd {
                s.push_str(&format!(" cwd={:?}", cwd));
            }
            s
        },
        Some(Run::Plugin(plugin)) => {
            format!("Plugin({})", plugin.location_string())
        },
        Some(Run::Cwd(path)) => format!("Cwd({:?})", path),
        Some(Run::EditFile(path, line, cwd)) => {
            format!("EditFile({:?}, line={:?}, cwd={:?})", path, line, cwd)
        },
    }
}

/// 收集 all 关闭 窗格 消息 from the pty 接收者
fn collect_close_pane_messages(
    pty_receiver: &Receiver<(PtyInstruction, zellij_utils::errors::ErrorContext)>,
) -> Vec<PaneId> {
    let mut closed_panes = Vec::new();
    while let Ok((instruction, _)) = pty_receiver.try_recv() {
        if let PtyInstruction::ClosePane(pane_id, _) = instruction {
            closed_panes.push(pane_id);
        }
    }
    closed_panes
}

/// 收集 all 卸载 插件 消息 from the 插件 接收者
fn collect_unload_plugin_messages(
    plugin_receiver: &Receiver<(PluginInstruction, zellij_utils::errors::ErrorContext)>,
) -> Vec<u32> {
    let mut unloaded_plugins = Vec::new();
    while let Ok((instruction, _)) = plugin_receiver.try_recv() {
        if let PluginInstruction::Unload(plugin_id) = instruction {
            unloaded_plugins.push(plugin_id);
        }
    }
    unloaded_plugins
}

#[test]
fn test_apply_empty_layout() {
    let kdl_layout = r#"
        layout {
        }
    "#;

    let (tiled_layout, floating_layout) = parse_kdl_layout(kdl_layout);
    let terminal_ids = vec![];

    let size = Size {
        cols: 100,
        rows: 50,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    ) = create_layout_applier_fixtures(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None, // blocking_terminal
    );

    let result = applier.apply_layout(
        tiled_layout,
        floating_layout,
        terminal_ids,
        vec![],         // new_floating_terminal_ids
        HashMap::new(), // new_plugin_ids
        1,              // client_id
    );

    assert!(result.is_ok());

    let snapshot = take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    );

    assert_snapshot!(snapshot);
}

#[test]
fn test_apply_simple_two_pane_layout() {
    let kdl_layout = r#"
        layout {
            pane
            pane
        }
    "#;

    let (tiled_layout, floating_layout) = parse_kdl_layout(kdl_layout);
    let terminal_ids = vec![(1, None), (2, None)];

    let size = Size {
        cols: 100,
        rows: 50,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    ) = create_layout_applier_fixtures(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            tiled_layout,
            floating_layout,
            terminal_ids,
            vec![],
            HashMap::new(),
            1,
        )
        .unwrap();

    let snapshot = take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    );

    assert_snapshot!(snapshot);
}

#[test]
fn test_apply_three_pane_layout() {
    let kdl_layout = r#"
        layout {
            pane
            pane
            pane
        }
    "#;

    let (tiled_layout, floating_layout) = parse_kdl_layout(kdl_layout);
    let terminal_ids = vec![(1, None), (2, None), (3, None)];

    let size = Size {
        cols: 100,
        rows: 50,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    ) = create_layout_applier_fixtures(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            tiled_layout,
            floating_layout,
            terminal_ids,
            vec![],
            HashMap::new(),
            1,
        )
        .unwrap();

    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_apply_horizontal_split_with_sizes() {
    let kdl_layout = r#"
        layout {
            pane split_direction="Horizontal" {
                pane size="30%"
                pane size="70%"
            }
        }
    "#;

    let (tiled_layout, floating_layout) = parse_kdl_layout(kdl_layout);
    let terminal_ids = vec![(1, None), (2, None)];

    let size = Size {
        cols: 100,
        rows: 50,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    ) = create_layout_applier_fixtures(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            tiled_layout,
            floating_layout,
            terminal_ids,
            vec![],
            HashMap::new(),
            1,
        )
        .unwrap();

    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_apply_vertical_split_with_sizes() {
    let kdl_layout = r#"
        layout {
            pane split_direction="Vertical" {
                pane size="60%"
                pane size="40%"
            }
        }
    "#;

    let (tiled_layout, floating_layout) = parse_kdl_layout(kdl_layout);
    let terminal_ids = vec![(1, None), (2, None)];

    let size = Size {
        cols: 100,
        rows: 50,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    ) = create_layout_applier_fixtures(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            tiled_layout,
            floating_layout,
            terminal_ids,
            vec![],
            HashMap::new(),
            1,
        )
        .unwrap();

    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_apply_nested_layout() {
    let kdl_layout = r#"
        layout {
            pane split_direction="Vertical" {
                pane size="60%"
                pane size="40%" split_direction="Horizontal" {
                    pane
                    pane
                }
            }
        }
    "#;

    let (tiled_layout, floating_layout) = parse_kdl_layout(kdl_layout);
    let terminal_ids = vec![(1, None), (2, None), (3, None)];

    let size = Size {
        cols: 120,
        rows: 40,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    ) = create_layout_applier_fixtures(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            tiled_layout,
            floating_layout,
            terminal_ids,
            vec![],
            HashMap::new(),
            1,
        )
        .unwrap();

    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_apply_layout_with_focus() {
    let kdl_layout = r#"
        layout {
            pane
            pane focus=true
            pane
        }
    "#;

    let (tiled_layout, floating_layout) = parse_kdl_layout(kdl_layout);
    let terminal_ids = vec![(1, None), (2, None), (3, None)];

    let size = Size {
        cols: 100,
        rows: 50,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    ) = create_layout_applier_fixtures(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            tiled_layout,
            floating_layout,
            terminal_ids,
            vec![],
            HashMap::new(),
            1,
        )
        .unwrap();

    // 快照 should show 焦点: Some(终端(2))
    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_apply_layout_with_commands() {
    let kdl_layout = r#"
        layout {
            pane command="htop"
            pane command="tail" {
                args "-f" "/var/log/syslog"
            }
            pane
        }
    "#;

    let (tiled_layout, floating_layout) = parse_kdl_layout(kdl_layout);
    let terminal_ids = vec![(1, None), (2, None), (3, None)];

    let size = Size {
        cols: 100,
        rows: 50,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    ) = create_layout_applier_fixtures(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            tiled_layout,
            floating_layout,
            terminal_ids,
            vec![],
            HashMap::new(),
            1,
        )
        .unwrap();

    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_apply_layout_with_named_panes() {
    let kdl_layout = r#"
        layout {
            pane name="editor"
            pane name="terminal"
            pane name="logs"
        }
    "#;

    let (tiled_layout, floating_layout) = parse_kdl_layout(kdl_layout);
    let terminal_ids = vec![(1, None), (2, None), (3, None)];

    let size = Size {
        cols: 100,
        rows: 50,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    ) = create_layout_applier_fixtures(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            tiled_layout,
            floating_layout,
            terminal_ids,
            vec![],
            HashMap::new(),
            1,
        )
        .unwrap();

    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_apply_layout_with_borderless_panes() {
    let kdl_layout = r#"
        layout {
            pane borderless=true
            pane
            pane borderless=true
        }
    "#;

    let (tiled_layout, floating_layout) = parse_kdl_layout(kdl_layout);
    let terminal_ids = vec![(1, None), (2, None), (3, None)];

    let size = Size {
        cols: 100,
        rows: 50,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    ) = create_layout_applier_fixtures(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            tiled_layout,
            floating_layout,
            terminal_ids,
            vec![],
            HashMap::new(),
            1,
        )
        .unwrap();

    // 快照 should show 视口 adjusted for borderless 窗格
    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_apply_layout_with_floating_panes() {
    let kdl_layout = r#"
        layout {
            pane
            pane
            floating_panes {
                pane {
                    x 10
                    y 10
                    width 40
                    height 20
                }
            }
        }
    "#;

    let (tiled_layout, floating_layout) = parse_kdl_layout(kdl_layout);
    let terminal_ids = vec![(1, None), (2, None)];
    let floating_terminal_ids = vec![(3, None)];

    let size = Size {
        cols: 100,
        rows: 50,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    ) = create_layout_applier_fixtures(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    let should_show_floating = applier
        .apply_layout(
            tiled_layout,
            floating_layout,
            terminal_ids,
            floating_terminal_ids,
            HashMap::new(),
            1,
        )
        .unwrap();

    assert_eq!(should_show_floating, true);

    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_apply_layout_with_floating_pane_with_command() {
    let kdl_layout = r#"
        layout {
            pane
            floating_panes {
                pane {
                    x 10
                    y 10
                    width 50
                    height 25
                    command "htop"
                }
            }
        }
    "#;

    let (tiled_layout, floating_layout) = parse_kdl_layout(kdl_layout);
    let terminal_ids = vec![(1, None)];
    let floating_terminal_ids = vec![(2, None)];

    let size = Size {
        cols: 100,
        rows: 50,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    ) = create_layout_applier_fixtures(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            tiled_layout,
            floating_layout,
            terminal_ids,
            floating_terminal_ids,
            HashMap::new(),
            1,
        )
        .unwrap();

    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_apply_layout_with_mixed_tiled_and_floating_panes() {
    let kdl_layout = r#"
        layout {
            pane split_direction="Vertical" {
                pane size="60%" command="vim"
                pane size="40%" split_direction="Horizontal" {
                    pane name="terminal" focus=true
                    pane command="tail" {
                        args "-f" "/var/log/syslog"
                    }
                }
            }
            floating_panes {
                pane {
                    x 5
                    y 5
                    width 40
                    height 15
                    command "htop"
                    name "monitor"
                }
                pane {
                    x "50%"
                    y 10
                    width 45
                    height 20
                    name "notes"
                }
            }
        }
    "#;

    let (tiled_layout, floating_layout) = parse_kdl_layout(kdl_layout);
    let terminal_ids = vec![(1, None), (2, None), (3, None)];
    let floating_terminal_ids = vec![(4, None), (5, None)];

    let size = Size {
        cols: 150,
        rows: 50,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    ) = create_layout_applier_fixtures(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    let should_show_floating = applier
        .apply_layout(
            tiled_layout,
            floating_layout,
            terminal_ids,
            floating_terminal_ids,
            HashMap::new(),
            1,
        )
        .unwrap();

    assert_eq!(should_show_floating, true);

    // 快照 should show:
    // - 3 平铺 窗格 with correct geometries, 命令, and names
    // - 2 浮动 窗格 with correct positions and properties
    // - 焦点 on 终端 窗格 (终端(2))
    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_reapply_layout_exact_match() {
    // First apply 初始 布局
    let initial_kdl = r#"
        layout {
            pane command="htop"
            pane command="vim"
            pane
        }
    "#;

    let (tiled_layout, floating_layout) = parse_kdl_layout(initial_kdl);
    let terminal_ids = vec![(1, None), (2, None), (3, None)];

    let size = Size {
        cols: 120,
        rows: 40,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    ) = create_layout_applier_fixtures(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            tiled_layout,
            floating_layout,
            terminal_ids,
            vec![],
            HashMap::new(),
            1,
        )
        .unwrap();

    // Now reapply with 命令 in different positions
    let new_kdl = r#"
        layout {
            pane
            pane command="htop"
            pane command="vim"
        }
    "#;

    let (new_layout, _) = parse_kdl_layout(new_kdl);

    applier
        .apply_tiled_panes_layout_to_existing_panes(&new_layout)
        .unwrap();

    let snapshot = take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    );

    // 快照 will show 窗格 matched by 命令 and repositioned
    assert_snapshot!(snapshot);
}

#[test]
fn test_reapply_layout_logical_position_match() {
    // Apply 初始 布局 - 3 窗格 in horizontal 分割
    let initial_kdl = r#"
        layout {
            pane
            pane
            pane
        }
    "#;

    let (tiled_layout, floating_layout) = parse_kdl_layout(initial_kdl);
    let terminal_ids = vec![(1, None), (2, None), (3, None)];

    let size = Size {
        cols: 120,
        rows: 40,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    ) = create_layout_applier_fixtures(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            tiled_layout,
            floating_layout,
            terminal_ids,
            vec![],
            HashMap::new(),
            1,
        )
        .unwrap();

    // Reapply DIFFERENT 布局 - still 3 窗格 but with different 分割
    // This 测试 logical position matching (position 0, 1, 2) without exact 命令 匹配
    let new_kdl = r#"
        layout {
            pane split_direction="Vertical" {
                pane size="50%"
                pane size="50%"
            }
            pane
        }
    "#;

    let (new_layout, _) = parse_kdl_layout(new_kdl);

    applier
        .apply_tiled_panes_layout_to_existing_panes(&new_layout)
        .unwrap();

    // 窗格 should be repositioned according to new 布局 structure
    // while being matched by their logical positions (0, 1, 2)
    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_reapply_layout_with_more_positions() {
    // Apply 初始 布局 with 2 窗格
    let initial_kdl = r#"
        layout {
            pane
            pane
        }
    "#;

    let (tiled_layout, floating_layout) = parse_kdl_layout(initial_kdl);
    let terminal_ids = vec![(1, None), (2, None)];

    let size = Size {
        cols: 100,
        rows: 50,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    ) = create_layout_applier_fixtures(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            tiled_layout,
            floating_layout,
            terminal_ids,
            vec![],
            HashMap::new(),
            1,
        )
        .unwrap();

    // Reapply with 4 positions (but we only have 2 窗格)
    let new_kdl = r#"
        layout {
            pane
            pane
            pane
            pane
        }
    "#;

    let (new_layout, _) = parse_kdl_layout(new_kdl);

    applier
        .apply_tiled_panes_layout_to_existing_panes(&new_layout)
        .unwrap();

    // Should show 2 窗格 filling first 2 positions, remaining positions empty
    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_reapply_floating_pane_layout() {
    // Apply 初始 布局
    let initial_kdl = r#"
        layout {
            pane
            floating_panes {
                pane {
                    x 10
                    y 10
                    width 40
                    height 20
                }
            }
        }
    "#;

    let (tiled_layout, floating_layout) = parse_kdl_layout(initial_kdl);
    let terminal_ids = vec![(1, None)];
    let floating_terminal_ids = vec![(2, None)];

    let size = Size {
        cols: 100,
        rows: 50,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    ) = create_layout_applier_fixtures(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            tiled_layout,
            floating_layout,
            terminal_ids,
            floating_terminal_ids,
            HashMap::new(),
            1,
        )
        .unwrap();

    // Reapply with different position
    let new_kdl = r#"
        layout {
            pane
            floating_panes {
                pane {
                    x 20
                    y 20
                    width 50
                    height 25
                }
            }
        }
    "#;

    let (_, new_floating_layout) = parse_kdl_layout(new_kdl);

    applier
        .apply_floating_panes_layout_to_existing_panes(&new_floating_layout)
        .unwrap();

    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_apply_complex_nested_layout() {
    let kdl_layout = r#"
        layout {
            pane split_direction="Vertical" {
                pane size="25%"
                pane size="50%" split_direction="Horizontal" {
                    pane size="60%"
                    pane size="40%" split_direction="Vertical" {
                        pane
                        pane
                    }
                }
                pane size="25%"
            }
        }
    "#;

    let (tiled_layout, floating_layout) = parse_kdl_layout(kdl_layout);
    let terminal_ids = vec![(1, None), (2, None), (3, None), (4, None), (5, None)];

    let size = Size {
        cols: 200,
        rows: 60,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    ) = create_layout_applier_fixtures(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            tiled_layout,
            floating_layout,
            terminal_ids,
            vec![],
            HashMap::new(),
            1,
        )
        .unwrap();

    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_apply_layout_with_stacked_panes() {
    let kdl_layout = r#"
        layout {
            pane split_direction="Vertical" {
                pane size="70%" stacked=true {
                    pane name="editor-1"
                    pane name="editor-2"
                    pane name="editor-3"
                }
                pane size="30%"
            }
        }
    "#;

    let (tiled_layout, floating_layout) = parse_kdl_layout(kdl_layout);
    let terminal_ids = vec![(1, None), (2, None), (3, None), (4, None)];

    let size = Size {
        cols: 120,
        rows: 40,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    ) = create_layout_applier_fixtures(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            tiled_layout,
            floating_layout,
            terminal_ids,
            vec![],
            HashMap::new(),
            1,
        )
        .unwrap();

    // 快照 should show:
    // - 4 窗格 total (3 in 栈 + 1 regular)
    // - 栈 窗格 should have 堆叠 字段 set with same stack_id
    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_apply_layout_with_multiple_stacks() {
    let kdl_layout = r#"
        layout {
            pane split_direction="Vertical" {
                pane size="50%" stacked=true {
                    pane name="left-1"
                    pane name="left-2"
                }
                pane size="50%" stacked=true {
                    pane name="right-1"
                    pane name="right-2"
                    pane name="right-3"
                }
            }
        }
    "#;

    let (tiled_layout, floating_layout) = parse_kdl_layout(kdl_layout);
    let terminal_ids = vec![(1, None), (2, None), (3, None), (4, None), (5, None)];

    let size = Size {
        cols: 150,
        rows: 40,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    ) = create_layout_applier_fixtures(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            tiled_layout,
            floating_layout,
            terminal_ids,
            vec![],
            HashMap::new(),
            1,
        )
        .unwrap();

    // 快照 should show:
    // - 5 窗格 in 2 different stacks (different stack_ids)
    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_apply_layout_with_plugin_panes() {
    let kdl_layout = r#"
        layout {
            pane
            pane {
                plugin location="zellij:tab-bar"
            }
            pane {
                plugin location="zellij:status-bar"
            }
        }
    "#;

    let (tiled_layout, floating_layout) = parse_kdl_layout(kdl_layout);
    let terminal_ids = vec![(1, None)];

    // 创建 插件 IDs - need to 匹配 the RunPluginOrAlias from the 布局

    let mut new_plugin_ids = HashMap::new();

    // 创建 插件 aliases that 匹配 the 布局
    let tab_bar_plugin = RunPluginOrAlias::from_url("zellij:tab-bar", &None, None, None).unwrap();
    let status_bar_plugin =
        RunPluginOrAlias::from_url("zellij:status-bar", &None, None, None).unwrap();

    new_plugin_ids.insert(tab_bar_plugin, vec![100]);
    new_plugin_ids.insert(status_bar_plugin, vec![101]);

    let size = Size {
        cols: 120,
        rows: 40,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    ) = create_layout_applier_fixtures(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            tiled_layout,
            floating_layout,
            terminal_ids,
            vec![],
            new_plugin_ids,
            1,
        )
        .unwrap();

    // 快照 should show:
    // - 1 终端 窗格
    // - 2 插件 窗格 with correct 插件 locations
    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_apply_layout_with_mixed_plugin_and_terminal_panes() {
    let kdl_layout = r#"
        layout {
            pane split_direction="Vertical" {
                pane size="20%" {
                    plugin location="file:///path/to/filebrowser.wasm"
                }
                pane size="60%" split_direction="Horizontal" {
                    pane command="vim"
                    pane command="cargo" {
                        args "watch" "-x" "test"
                    }
                }
                pane size="20%" {
                    plugin location="zellij:compact-bar"
                }
            }
        }
    "#;

    let (tiled_layout, floating_layout) = parse_kdl_layout(kdl_layout);
    let terminal_ids = vec![(1, None), (2, None)];

    let mut new_plugin_ids = HashMap::new();

    let filebrowser_plugin =
        RunPluginOrAlias::from_url("file:///path/to/filebrowser.wasm", &None, None, None).unwrap();
    let compact_bar_plugin =
        RunPluginOrAlias::from_url("zellij:compact-bar", &None, None, None).unwrap();

    new_plugin_ids.insert(filebrowser_plugin, vec![102]);
    new_plugin_ids.insert(compact_bar_plugin, vec![103]);

    let size = Size {
        cols: 200,
        rows: 50,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    ) = create_layout_applier_fixtures(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            tiled_layout,
            floating_layout,
            terminal_ids,
            vec![],
            new_plugin_ids,
            1,
        )
        .unwrap();

    // 快照 should show:
    // - 2 终端 窗格 with 命令
    // - 2 插件 窗格 with different locations
    // - Correct size distribution (20%, 60%, 20%)
    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_apply_layout_with_missing_plugin_ids() {
    let kdl_layout = r#"
        layout {
            pane
            pane {
                plugin location="zellij:tab-bar"
            }
        }
    "#;

    let (tiled_layout, floating_layout) = parse_kdl_layout(kdl_layout);
    let terminal_ids = vec![(1, None)];
    // Don't provide 插件 IDs - empty HashMap
    let new_plugin_ids = HashMap::new();

    let size = Size {
        cols: 100,
        rows: 50,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    ) = create_layout_applier_fixtures(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    let result = applier.apply_layout(
        tiled_layout,
        floating_layout,
        terminal_ids,
        vec![],
        new_plugin_ids,
        1,
    );

    // This should 返回 an 错误 - missing 插件 ID
    assert!(result.is_err());
}

#[test]
fn test_apply_layout_with_excess_terminal_ids() {
    let kdl_layout = r#"
        layout {
            pane
            pane
        }
    "#;

    let (tiled_layout, floating_layout) = parse_kdl_layout(kdl_layout);
    // Provide more 终端 IDs than needed
    let terminal_ids = vec![(1, None), (2, None), (3, None), (4, None)];

    let size = Size {
        cols: 100,
        rows: 50,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    ) = create_layout_applier_fixtures(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    let result = applier.apply_layout(
        tiled_layout,
        floating_layout,
        terminal_ids,
        vec![],
        HashMap::new(),
        1,
    );

    assert!(result.is_ok());

    // 快照 should show only 2 窗格 创建的
    // Excess IDs should be 关闭的 by the applier
    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_override_layout_basic_with_both_tiled_and_floating() {
    // 设置: Apply 初始 布局 with 2 平铺 窗格 + 1 浮动 窗格
    let initial_kdl = r#"
        layout {
            pane command="htop"
            pane command="vim"
            floating_panes {
                pane {
                    x 10
                    y 10
                    width 40
                    height 20
                    command "tail"
                    args "-f" "/var/log/syslog"
                }
            }
        }
    "#;

    let (initial_tiled, initial_floating) = parse_kdl_layout(initial_kdl);
    let terminal_ids = vec![(1, None), (2, None)];
    let floating_terminal_ids = vec![(3, None)];

    let size = Size {
        cols: 120,
        rows: 40,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        pty_receiver,
        plugin_receiver,
    ) = create_layout_applier_fixtures_with_receivers(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            initial_tiled,
            initial_floating,
            terminal_ids,
            floating_terminal_ids,
            HashMap::new(),
            1,
        )
        .unwrap();

    // Now override with different 布局 (2 平铺 + 1 浮动)
    let override_kdl = r#"
        layout {
            pane command="top"
            pane command="htop"
            floating_panes {
                pane {
                    x 20
                    y 20
                    width 50
                    height 25
                    command "watch"
                    args "df" "-h"
                }
            }
        }
    "#;

    let (override_tiled, override_floating) = parse_kdl_layout(override_kdl);
    let new_terminal_ids = vec![(4, None)];
    let new_floating_terminal_ids = vec![(5, None)];

    let retain_existing_terminal_panes = false;
    let retain_existing_plugin_panes = false;
    let should_show_floating = applier
        .override_layout(
            override_tiled,
            override_floating,
            new_terminal_ids,
            new_floating_terminal_ids,
            HashMap::new(),
            retain_existing_terminal_panes,
            retain_existing_plugin_panes,
            1,
        )
        .unwrap();

    // Should show 浮动 窗格
    assert_eq!(should_show_floating, true);

    // 验证 关闭 消息 were sent for vim (终端(2)) and tail (终端(3))
    let closed_panes = collect_close_pane_messages(&pty_receiver);
    assert_eq!(closed_panes.len(), 2);
    assert!(closed_panes.contains(&PaneId::Terminal(2))); // vim
    assert!(closed_panes.contains(&PaneId::Terminal(3))); // tail

    // No 插件 should be 卸载的
    let unloaded_plugins = collect_unload_plugin_messages(&plugin_receiver);
    assert!(unloaded_plugins.is_empty());

    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_override_layout_hide_floating_panes_true() {
    // 设置: 初始 布局 with 浮动 窗格
    let initial_kdl = r#"
        layout {
            pane
            floating_panes {
                pane {
                    x 10
                    y 10
                    width 40
                    height 20
                }
            }
        }
    "#;

    let (initial_tiled, initial_floating) = parse_kdl_layout(initial_kdl);
    let terminal_ids = vec![(1, None)];
    let floating_terminal_ids = vec![(2, None)];

    let size = Size {
        cols: 100,
        rows: 50,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        pty_receiver,
        plugin_receiver,
    ) = create_layout_applier_fixtures_with_receivers(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            initial_tiled,
            initial_floating,
            terminal_ids,
            floating_terminal_ids,
            HashMap::new(),
            1,
        )
        .unwrap();

    // Override with 布局 that has hide_floating_panes true
    let override_kdl = r#"
        layout {
            hide_floating_panes true
            pane
            pane
            floating_panes {
                pane {
                    x 15
                    y 15
                    width 45
                    height 22
                }
            }
        }
    "#;

    let (override_tiled, override_floating) = parse_kdl_layout(override_kdl);
    let new_terminal_ids = vec![(3, None)];
    let new_floating_terminal_ids = vec![(4, None)];

    let retain_existing_terminal_panes = false;
    let retain_existing_plugin_panes = false;
    let should_show_floating = applier
        .override_layout(
            override_tiled,
            override_floating,
            new_terminal_ids,
            new_floating_terminal_ids,
            HashMap::new(),
            retain_existing_terminal_panes,
            retain_existing_plugin_panes,
            1,
        )
        .unwrap();

    // Should NOT show 浮动 窗格 because of hide_floating_panes
    assert_eq!(should_show_floating, false);

    let closed_panes = collect_close_pane_messages(&pty_receiver);
    assert_eq!(closed_panes.len(), 0);

    // No 插件 should be 卸载的
    let unloaded_plugins = collect_unload_plugin_messages(&plugin_receiver);
    assert!(unloaded_plugins.is_empty());

    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_override_layout_show_floating_panes() {
    // 设置: 初始 布局
    let initial_kdl = r#"
        layout {
            pane
            pane
        }
    "#;

    let (initial_tiled, initial_floating) = parse_kdl_layout(initial_kdl);
    let terminal_ids = vec![(1, None), (2, None)];

    let size = Size {
        cols: 100,
        rows: 50,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        pty_receiver,
        plugin_receiver,
    ) = create_layout_applier_fixtures_with_receivers(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            initial_tiled,
            initial_floating,
            terminal_ids,
            vec![],
            HashMap::new(),
            1,
        )
        .unwrap();

    // Override with 布局 containing 浮动 窗格
    let override_kdl = r#"
        layout {
            pane
            floating_panes {
                pane {
                    x 20
                    y 10
                    width 50
                    height 30
                }
            }
        }
    "#;

    let (override_tiled, override_floating) = parse_kdl_layout(override_kdl);
    let new_floating_terminal_ids = vec![(3, None)];

    let retain_existing_terminal_panes = false;
    let retain_existing_plugin_panes = false;
    let should_show_floating = applier
        .override_layout(
            override_tiled,
            override_floating,
            vec![],
            new_floating_terminal_ids,
            HashMap::new(),
            retain_existing_terminal_panes,
            retain_existing_plugin_panes,
            1,
        )
        .unwrap();

    // Should show 浮动 窗格
    assert_eq!(should_show_floating, true);

    // 验证 关闭 消息 was sent for one 平铺 窗格 (终端(2))
    let closed_panes = collect_close_pane_messages(&pty_receiver);
    assert_eq!(closed_panes.len(), 1);
    assert!(closed_panes.contains(&PaneId::Terminal(2)));

    // No 插件 should be 卸载的
    let unloaded_plugins = collect_unload_plugin_messages(&plugin_receiver);
    assert!(unloaded_plugins.is_empty());

    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

// ============================================================================
// Suite 2: override_tiled_panes_layout_for_existing_panes 测试
// ============================================================================

#[test]
fn test_override_tiled_exact_match_preservation_commands() {
    // 设置: Apply 初始 布局 with 3 窗格 running different 命令
    let initial_kdl = r#"
        layout {
            pane command="htop"
            pane command="vim"
            pane command="tail" {
                args "-f" "/var/log/syslog"
            }
        }
    "#;

    let (initial_tiled, initial_floating) = parse_kdl_layout(initial_kdl);
    let terminal_ids = vec![(1, None), (2, None), (3, None)];

    let size = Size {
        cols: 120,
        rows: 40,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        pty_receiver,
        plugin_receiver,
    ) = create_layout_applier_fixtures_with_receivers(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            initial_tiled,
            initial_floating,
            terminal_ids,
            vec![],
            HashMap::new(),
            1,
        )
        .unwrap();

    // Override: New 布局 with only htop and vim in different positions
    let override_kdl = r#"
        layout {
            pane command="vim"
            pane command="htop"
        }
    "#;

    let (override_tiled, _) = parse_kdl_layout(override_kdl);

    applier
        .override_tiled_panes_layout_for_existing_panes(
            &override_tiled,
            vec![],
            &mut HashMap::new(),
            false,
            false,
            1,
        )
        .unwrap();

    // htop and vim 窗格 should be 保存的 (same PaneIds: 终端(1) and 终端(2))
    // tail 窗格 should be 关闭的
    // 窗格 should be repositioned to new 布局 positions

    // 验证 关闭 消息 was sent for tail 窗格 (终端(3))
    let closed_panes = collect_close_pane_messages(&pty_receiver);
    assert_eq!(closed_panes.len(), 1);
    assert!(closed_panes.contains(&PaneId::Terminal(3))); // tail

    // No 插件 should be 卸载的
    let unloaded_plugins = collect_unload_plugin_messages(&plugin_receiver);
    assert!(unloaded_plugins.is_empty());

    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_override_tiled_exact_match_preservation_plugins() {
    // 设置: 初始 布局 with 2 终端 窗格 + 1 插件 窗格
    let initial_kdl = r#"
        layout {
            pane
            pane
            pane {
                plugin location="zellij:tab-bar"
            }
        }
    "#;

    let (initial_tiled, initial_floating) = parse_kdl_layout(initial_kdl);
    let terminal_ids = vec![(1, None), (2, None)];

    let mut initial_plugin_ids = HashMap::new();
    let tab_bar_plugin = RunPluginOrAlias::from_url("zellij:tab-bar", &None, None, None).unwrap();
    initial_plugin_ids.insert(tab_bar_plugin.clone(), vec![100]);

    let size = Size {
        cols: 120,
        rows: 40,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        pty_receiver,
        plugin_receiver,
    ) = create_layout_applier_fixtures_with_receivers(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            initial_tiled,
            initial_floating,
            terminal_ids,
            vec![],
            initial_plugin_ids,
            1,
        )
        .unwrap();

    // Override: New 布局 with only the 插件 窗格
    let override_kdl = r#"
        layout {
            pane {
                plugin location="zellij:tab-bar"
            }
        }
    "#;

    let (override_tiled, _) = parse_kdl_layout(override_kdl);

    applier
        .override_tiled_panes_layout_for_existing_panes(
            &override_tiled,
            vec![],
            &mut HashMap::new(),
            false,
            false,
            1,
        )
        .unwrap();

    // 插件 窗格 should be 保存的
    // 终端 窗格 should be 关闭的
    // Total 窗格 count is 1

    // 验证 关闭 消息 were sent for both 终端 窗格 (终端(1) and 终端(2))
    let closed_panes = collect_close_pane_messages(&pty_receiver);
    assert_eq!(closed_panes.len(), 2);
    assert!(closed_panes.contains(&PaneId::Terminal(1)));
    assert!(closed_panes.contains(&PaneId::Terminal(2)));

    // No 插件 should be 卸载的 (插件 is 保存的)
    let unloaded_plugins = collect_unload_plugin_messages(&plugin_receiver);
    assert!(unloaded_plugins.is_empty());

    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_override_tiled_all_panes_closed_no_matches() {
    // 设置: 3 窗格 running htop, vim, tail
    let initial_kdl = r#"
        layout {
            pane command="htop"
            pane command="vim"
            pane command="tail" {
                args "-f" "/var/log/syslog"
            }
        }
    "#;

    let (initial_tiled, initial_floating) = parse_kdl_layout(initial_kdl);
    let terminal_ids = vec![(1, None), (2, None), (3, None)];

    let size = Size {
        cols: 120,
        rows: 40,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        pty_receiver,
        plugin_receiver,
    ) = create_layout_applier_fixtures_with_receivers(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            initial_tiled,
            initial_floating,
            terminal_ids,
            vec![],
            HashMap::new(),
            1,
        )
        .unwrap();

    // Override: New 布局 with 3 completely different 命令
    let override_kdl = r#"
        layout {
            pane command="cargo" {
                args "watch"
            }
            pane command="npm" {
                args "start"
            }
            pane command="python" {
                args "-m" "http.server"
            }
        }
    "#;

    let (override_tiled, _) = parse_kdl_layout(override_kdl);
    let new_terminal_ids = vec![(4, None), (5, None), (6, None)];

    applier
        .override_tiled_panes_layout_for_existing_panes(
            &override_tiled,
            new_terminal_ids,
            &mut HashMap::new(),
            false,
            false,
            1,
        )
        .unwrap();

    // All 原始 窗格 IDs gone (1, 2, 3 should not be present)
    // 3 new 窗格 with new IDs (4, 5, 6)
    // Total 窗格 count is 3

    // 验证 关闭 消息 were sent for all 原始 窗格 (终端(1), 终端(2), 终端(3))
    let closed_panes = collect_close_pane_messages(&pty_receiver);
    assert_eq!(closed_panes.len(), 3);
    assert!(closed_panes.contains(&PaneId::Terminal(1))); // htop
    assert!(closed_panes.contains(&PaneId::Terminal(2))); // vim
    assert!(closed_panes.contains(&PaneId::Terminal(3))); // tail

    // No 插件 should be 卸载的
    let unloaded_plugins = collect_unload_plugin_messages(&plugin_receiver);
    assert!(unloaded_plugins.is_empty());

    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_override_tiled_mixed_some_matches_some_new() {
    // 设置: 2 窗格 - one running htop, one 泛型 shell (no 命令)
    let initial_kdl = r#"
        layout {
            pane command="htop"
            pane
        }
    "#;

    let (initial_tiled, initial_floating) = parse_kdl_layout(initial_kdl);
    let terminal_ids = vec![(1, None), (2, None)];

    let size = Size {
        cols: 120,
        rows: 40,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        pty_receiver,
        plugin_receiver,
    ) = create_layout_applier_fixtures_with_receivers(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            initial_tiled,
            initial_floating,
            terminal_ids,
            vec![],
            HashMap::new(),
            1,
        )
        .unwrap();

    // Override: 布局 with htop, vim, and 泛型 shell
    let override_kdl = r#"
        layout {
            pane command="htop"
            pane command="vim"
            pane
        }
    "#;

    let (override_tiled, _) = parse_kdl_layout(override_kdl);
    let new_terminal_ids = vec![(3, None), (4, None)];

    applier
        .override_tiled_panes_layout_for_existing_panes(
            &override_tiled,
            new_terminal_ids,
            &mut HashMap::new(),
            false,
            false,
            1,
        )
        .unwrap();

    // htop 保存的 with same ID (终端(1))
    // 原始 shell 窗格 关闭的 (泛型 shells are NOT exact matches)
    // 2 new 窗格 创建的 (vim and new shell)
    // Total 窗格 count is 3

    // 验证 关闭 消息 was not sent for 原始 shell 窗格 (终端(2))
    let closed_panes = collect_close_pane_messages(&pty_receiver);
    assert_eq!(closed_panes.len(), 0);

    // No 插件 should be 卸载的
    let unloaded_plugins = collect_unload_plugin_messages(&plugin_receiver);
    assert!(unloaded_plugins.is_empty());

    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_override_tiled_new_panes_for_unmatched_positions() {
    // 设置: 1 窗格 running htop
    let initial_kdl = r#"
        layout {
            pane command="htop"
        }
    "#;

    let (initial_tiled, initial_floating) = parse_kdl_layout(initial_kdl);
    let terminal_ids = vec![(1, None)];

    let size = Size {
        cols: 150,
        rows: 50,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    ) = create_layout_applier_fixtures(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            initial_tiled,
            initial_floating,
            terminal_ids,
            vec![],
            HashMap::new(),
            1,
        )
        .unwrap();

    // Override: 布局 with 4 positions
    let override_kdl = r#"
        layout {
            pane command="htop"
            pane command="vim"
            pane
            pane
        }
    "#;

    let (override_tiled, _) = parse_kdl_layout(override_kdl);
    let new_terminal_ids = vec![(2, None), (3, None), (4, None)];

    applier
        .override_tiled_panes_layout_for_existing_panes(
            &override_tiled,
            new_terminal_ids,
            &mut HashMap::new(),
            false,
            false,
            1,
        )
        .unwrap();

    // 1 原始 htop 窗格 保存的 (终端(1))
    // 3 new 窗格 创建的 (终端(2), 终端(3), 终端(4))
    // Total 4 窗格
    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_override_tiled_focus_on_new_pane() {
    // 设置: 2 窗格, first one 聚焦的
    let initial_kdl = r#"
        layout {
            pane focus=true
            pane
        }
    "#;

    let (initial_tiled, initial_floating) = parse_kdl_layout(initial_kdl);
    let terminal_ids = vec![(1, None), (2, None)];

    let size = Size {
        cols: 100,
        rows: 50,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    ) = create_layout_applier_fixtures(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            initial_tiled,
            initial_floating,
            terminal_ids,
            vec![],
            HashMap::new(),
            1,
        )
        .unwrap();

    // Override: 布局 with 3 窗格 where second 窗格 has 焦点=true
    let override_kdl = r#"
        layout {
            pane
            pane focus=true
            pane
        }
    "#;

    let (override_tiled, _) = parse_kdl_layout(override_kdl);
    let new_terminal_ids = vec![(3, None)];

    applier
        .override_tiled_panes_layout_for_existing_panes(
            &override_tiled,
            new_terminal_ids,
            &mut HashMap::new(),
            false,
            false,
            1,
        )
        .unwrap();

    // focus_pane_id should point to the newly 创建的 middle 窗格 (终端(3))
    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_override_tiled_focus_when_focused_pane_closed() {
    // 设置: 3 窗格, middle one 聚焦的 running vim
    let initial_kdl = r#"
        layout {
            pane command="htop"
            pane command="vim" focus=true
            pane command="tail" {
                args "-f" "/var/log/syslog"
            }
        }
    "#;

    let (initial_tiled, initial_floating) = parse_kdl_layout(initial_kdl);
    let terminal_ids = vec![(1, None), (2, None), (3, None)];

    let size = Size {
        cols: 120,
        rows: 40,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        pty_receiver,
        plugin_receiver,
    ) = create_layout_applier_fixtures_with_receivers(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            initial_tiled,
            initial_floating,
            terminal_ids,
            vec![],
            HashMap::new(),
            1,
        )
        .unwrap();

    // Override: 布局 with 2 窗格 running htop and cargo (not vim)
    let override_kdl = r#"
        layout {
            pane command="htop"
            pane command="cargo" {
                args "check"
            }
        }
    "#;

    let (override_tiled, _) = parse_kdl_layout(override_kdl);
    let new_terminal_ids = vec![(4, None)];

    applier
        .override_tiled_panes_layout_for_existing_panes(
            &override_tiled,
            new_terminal_ids,
            &mut HashMap::new(),
            false,
            false,
            1,
        )
        .unwrap();

    // 聚焦的 窗格 (vim) no longer exists
    // 焦点 should be 移动 to one of the remaining 窗格

    // 验证 关闭 消息 were sent for vim (终端(2)) and tail (终端(3))
    let closed_panes = collect_close_pane_messages(&pty_receiver);
    assert_eq!(closed_panes.len(), 2);
    assert!(closed_panes.contains(&PaneId::Terminal(2))); // vim (聚焦的)
    assert!(closed_panes.contains(&PaneId::Terminal(3))); // tail

    // No 插件 should be 卸载的
    let unloaded_plugins = collect_unload_plugin_messages(&plugin_receiver);
    assert!(unloaded_plugins.is_empty());

    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_override_tiled_empty_layout_closes_all() {
    // 设置: 3 窗格 running various 命令
    let initial_kdl = r#"
        layout {
            pane command="htop"
            pane command="vim"
            pane command="tail" {
                args "-f" "/var/log/syslog"
            }
        }
    "#;

    let (initial_tiled, initial_floating) = parse_kdl_layout(initial_kdl);
    let terminal_ids = vec![(1, None), (2, None), (3, None)];

    let size = Size {
        cols: 120,
        rows: 40,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        pty_receiver,
        plugin_receiver,
    ) = create_layout_applier_fixtures_with_receivers(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            initial_tiled,
            initial_floating,
            terminal_ids,
            vec![],
            HashMap::new(),
            1,
        )
        .unwrap();

    // Override: Empty 布局
    let override_kdl = r#"
        layout {
        }
    "#;

    let (override_tiled, _) = parse_kdl_layout(override_kdl);

    applier
        .override_tiled_panes_layout_for_existing_panes(
            &override_tiled,
            vec![],
            &mut HashMap::new(),
            false,
            false,
            1,
        )
        .unwrap();

    // No 窗格 in 快照
    // 窗格 count is 0

    // 验证 关闭 消息 were sent for all 窗格 (终端(1), 终端(2), 终端(3))
    let closed_panes = collect_close_pane_messages(&pty_receiver);
    assert_eq!(closed_panes.len(), 3);
    assert!(closed_panes.contains(&PaneId::Terminal(1))); // htop
    assert!(closed_panes.contains(&PaneId::Terminal(2))); // vim
    assert!(closed_panes.contains(&PaneId::Terminal(3))); // tail

    // No 插件 should be 卸载的
    let unloaded_plugins = collect_unload_plugin_messages(&plugin_receiver);
    assert!(unloaded_plugins.is_empty());

    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

// ============================================================================
// Suite 3: override_floating_panes_layout_for_existing_panes 测试
// ============================================================================

#[test]
fn test_override_floating_exact_match_preservation() {
    // 设置: 2 浮动 窗格 running htop and vim
    let initial_kdl = r#"
        layout {
            pane
            floating_panes {
                pane {
                    x 10
                    y 10
                    width 40
                    height 20
                    command "htop"
                }
                pane {
                    x 20
                    y 20
                    width 50
                    height 25
                    command "vim"
                }
            }
        }
    "#;

    let (initial_tiled, initial_floating) = parse_kdl_layout(initial_kdl);
    let terminal_ids = vec![(1, None)];
    let floating_terminal_ids = vec![(2, None), (3, None)];

    let size = Size {
        cols: 120,
        rows: 40,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        pty_receiver,
        plugin_receiver,
    ) = create_layout_applier_fixtures_with_receivers(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            initial_tiled,
            initial_floating,
            terminal_ids,
            floating_terminal_ids,
            HashMap::new(),
            1,
        )
        .unwrap();

    // Override: 布局 with htop at different x/y position
    let override_kdl = r#"
        layout {
            pane
            floating_panes {
                pane {
                    x 50
                    y 30
                    width 45
                    height 22
                    command "htop"
                }
            }
        }
    "#;

    let (_, override_floating) = parse_kdl_layout(override_kdl);

    applier
        .override_floating_panes_layout_for_existing_panes(
            &override_floating,
            vec![],
            &mut HashMap::new(),
            false,
            false,
        )
        .unwrap();

    // htop 窗格 保存的 (终端(2)), repositioned
    // vim 窗格 关闭的 (终端(3))
    // Total 浮动 窗格 count is 1

    // 验证 关闭 消息 was sent for vim 窗格 (终端(3))
    let closed_panes = collect_close_pane_messages(&pty_receiver);
    assert_eq!(closed_panes.len(), 1);
    assert!(closed_panes.contains(&PaneId::Terminal(3))); // vim

    // No 插件 should be 卸载的
    let unloaded_plugins = collect_unload_plugin_messages(&plugin_receiver);
    assert!(unloaded_plugins.is_empty());

    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_override_floating_all_closed_no_matches() {
    // 设置: 2 浮动 窗格 with 特定 命令
    let initial_kdl = r#"
        layout {
            pane
            floating_panes {
                pane {
                    x 10
                    y 10
                    width 40
                    height 20
                    command "htop"
                }
                pane {
                    x 20
                    y 20
                    width 50
                    height 25
                    command "vim"
                }
            }
        }
    "#;

    let (initial_tiled, initial_floating) = parse_kdl_layout(initial_kdl);
    let terminal_ids = vec![(1, None)];
    let floating_terminal_ids = vec![(2, None), (3, None)];

    let size = Size {
        cols: 120,
        rows: 40,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        pty_receiver,
        plugin_receiver,
    ) = create_layout_applier_fixtures_with_receivers(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            initial_tiled,
            initial_floating,
            terminal_ids,
            floating_terminal_ids,
            HashMap::new(),
            1,
        )
        .unwrap();

    // Override: 布局 with different 命令
    let override_kdl = r#"
        layout {
            pane
            floating_panes {
                pane {
                    x 15
                    y 15
                    width 45
                    height 22
                    command "top"
                }
                pane {
                    x 25
                    y 25
                    width 55
                    height 27
                    command "emacs"
                }
            }
        }
    "#;

    let (_, override_floating) = parse_kdl_layout(override_kdl);
    let new_floating_terminal_ids = vec![(4, None), (5, None)];

    applier
        .override_floating_panes_layout_for_existing_panes(
            &override_floating,
            new_floating_terminal_ids,
            &mut HashMap::new(),
            false,
            false,
        )
        .unwrap();

    // Both 原始 窗格 关闭的 (IDs 2, 3 gone)
    // New 窗格 创建的 with new IDs (4, 5)
    // 窗格 count matches new 布局 (2 浮动 窗格)

    // 验证 关闭 消息 were sent for both 浮动 窗格 (终端(2), 终端(3))
    let closed_panes = collect_close_pane_messages(&pty_receiver);
    assert_eq!(closed_panes.len(), 2);
    assert!(closed_panes.contains(&PaneId::Terminal(2))); // htop
    assert!(closed_panes.contains(&PaneId::Terminal(3))); // vim

    // No 插件 should be 卸载的
    let unloaded_plugins = collect_unload_plugin_messages(&plugin_receiver);
    assert!(unloaded_plugins.is_empty());

    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_override_floating_new_panes_created() {
    // 设置: 1 浮动 窗格 running htop
    let initial_kdl = r#"
        layout {
            pane
            floating_panes {
                pane {
                    x 10
                    y 10
                    width 40
                    height 20
                    command "htop"
                }
            }
        }
    "#;

    let (initial_tiled, initial_floating) = parse_kdl_layout(initial_kdl);
    let terminal_ids = vec![(1, None)];
    let floating_terminal_ids = vec![(2, None)];

    let size = Size {
        cols: 120,
        rows: 40,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    ) = create_layout_applier_fixtures(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            initial_tiled,
            initial_floating,
            terminal_ids,
            floating_terminal_ids,
            HashMap::new(),
            1,
        )
        .unwrap();

    // Override: 布局 with 3 浮动 窗格: htop, vim, and 泛型 shell
    let override_kdl = r#"
        layout {
            pane
            floating_panes {
                pane {
                    x 10
                    y 10
                    width 40
                    height 20
                    command "htop"
                }
                pane {
                    x 20
                    y 20
                    width 50
                    height 25
                    command "vim"
                }
                pane {
                    x 30
                    y 30
                    width 45
                    height 22
                }
            }
        }
    "#;

    let (_, override_floating) = parse_kdl_layout(override_kdl);
    let new_floating_terminal_ids = vec![(3, None), (4, None)];

    applier
        .override_floating_panes_layout_for_existing_panes(
            &override_floating,
            new_floating_terminal_ids,
            &mut HashMap::new(),
            false,
            false,
        )
        .unwrap();

    // 原始 htop 保存的 (终端(2))
    // 2 new 浮动 窗格 创建的 (终端(3), 终端(4))
    // Total 3 浮动 窗格
    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_override_floating_focus_handling() {
    // 设置: 2 浮动 窗格, one 聚焦的
    let initial_kdl = r#"
        layout {
            pane
            floating_panes {
                pane {
                    x 10
                    y 10
                    width 40
                    height 20
                    focus true
                }
                pane {
                    x 20
                    y 20
                    width 50
                    height 25
                }
            }
        }
    "#;

    let (initial_tiled, initial_floating) = parse_kdl_layout(initial_kdl);
    let terminal_ids = vec![(1, None)];
    let floating_terminal_ids = vec![(2, None), (3, None)];

    let size = Size {
        cols: 120,
        rows: 40,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        pty_receiver,
        plugin_receiver,
    ) = create_layout_applier_fixtures_with_receivers(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            initial_tiled,
            initial_floating,
            terminal_ids,
            floating_terminal_ids,
            HashMap::new(),
            1,
        )
        .unwrap();

    // Override: 布局 with 1 new 窗格 that has 焦点=true
    let override_kdl = r#"
        layout {
            pane
            floating_panes {
                pane {
                    x 30
                    y 30
                    width 60
                    height 30
                    focus true
                }
            }
        }
    "#;

    let (_, override_floating) = parse_kdl_layout(override_kdl);
    let new_floating_terminal_ids = vec![(4, None)];

    applier
        .override_floating_panes_layout_for_existing_panes(
            &override_floating,
            new_floating_terminal_ids,
            &mut HashMap::new(),
            false,
            false,
        )
        .unwrap();

    // 焦点 should be set on newly 创建的 窗格 (终端(4))

    // 验证 关闭 消息 were sent for 终端(3)
    let closed_panes = collect_close_pane_messages(&pty_receiver);
    assert_eq!(closed_panes.len(), 1);
    assert!(closed_panes.contains(&PaneId::Terminal(3)));

    // No 插件 should be 卸载的
    let unloaded_plugins = collect_unload_plugin_messages(&plugin_receiver);
    assert!(unloaded_plugins.is_empty());

    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_override_floating_position_and_size_update() {
    // 设置: 1 浮动 窗格 running htop at 特定 position
    let initial_kdl = r#"
        layout {
            pane
            floating_panes {
                pane {
                    x 10
                    y 10
                    width 40
                    height 20
                    command "htop"
                }
            }
        }
    "#;

    let (initial_tiled, initial_floating) = parse_kdl_layout(initial_kdl);
    let terminal_ids = vec![(1, None)];
    let floating_terminal_ids = vec![(2, None)];

    let size = Size {
        cols: 120,
        rows: 50,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    ) = create_layout_applier_fixtures(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            initial_tiled,
            initial_floating,
            terminal_ids,
            floating_terminal_ids,
            HashMap::new(),
            1,
        )
        .unwrap();

    // Override: 布局 with htop at different position and size
    let override_kdl = r#"
        layout {
            pane
            floating_panes {
                pane {
                    x 50
                    y 30
                    width 60
                    height 30
                    command "htop"
                }
            }
        }
    "#;

    let (_, override_floating) = parse_kdl_layout(override_kdl);

    applier
        .override_floating_panes_layout_for_existing_panes(
            &override_floating,
            vec![],
            &mut HashMap::new(),
            false,
            false,
        )
        .unwrap();

    // Same 窗格 ID 保存的 (终端(2))
    // 几何 updated: x=50, y=30, cols=60, 行=30
    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_override_floating_return_value_has_panes() {
    // 设置: Empty 浮动 窗格
    let initial_kdl = r#"
        layout {
            pane
        }
    "#;

    let (initial_tiled, initial_floating) = parse_kdl_layout(initial_kdl);
    let terminal_ids = vec![(1, None)];

    let size = Size {
        cols: 100,
        rows: 50,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
    ) = create_layout_applier_fixtures(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            initial_tiled,
            initial_floating,
            terminal_ids,
            vec![],
            HashMap::new(),
            1,
        )
        .unwrap();

    // Override: 布局 with 1 浮动 窗格
    let override_kdl = r#"
        layout {
            pane
            floating_panes {
                pane {
                    x 20
                    y 10
                    width 50
                    height 30
                }
            }
        }
    "#;

    let (_, override_floating) = parse_kdl_layout(override_kdl);
    let new_floating_terminal_ids = vec![(2, None)];

    let has_floating_panes = applier
        .override_floating_panes_layout_for_existing_panes(
            &override_floating,
            new_floating_terminal_ids,
            &mut HashMap::new(),
            false,
            false,
        )
        .unwrap();

    // 函数 should 返回 true because 布局 has 浮动 窗格
    assert_eq!(has_floating_panes, true);

    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_override_floating_return_value_no_panes() {
    // 设置: 1 浮动 窗格
    let initial_kdl = r#"
        layout {
            pane
            floating_panes {
                pane {
                    x 10
                    y 10
                    width 40
                    height 20
                }
            }
        }
    "#;

    let (initial_tiled, initial_floating) = parse_kdl_layout(initial_kdl);
    let terminal_ids = vec![(1, None)];
    let floating_terminal_ids = vec![(2, None)];

    let size = Size {
        cols: 100,
        rows: 50,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        pty_receiver,
        plugin_receiver,
    ) = create_layout_applier_fixtures_with_receivers(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            initial_tiled,
            initial_floating,
            terminal_ids,
            floating_terminal_ids,
            HashMap::new(),
            1,
        )
        .unwrap();

    // Override: Empty 浮动 布局 (no floating_panes block)
    let override_kdl = r#"
        layout {
            pane
        }
    "#;

    let (_, override_floating) = parse_kdl_layout(override_kdl);

    let has_floating_panes = applier
        .override_floating_panes_layout_for_existing_panes(
            &override_floating,
            vec![],
            &mut HashMap::new(),
            false,
            false,
        )
        .unwrap();

    // 函数 should 返回 false because 布局 has no 浮动 窗格
    assert_eq!(has_floating_panes, false);

    // 验证 关闭 消息 was sent for 浮动 窗格 (终端(2))
    let closed_panes = collect_close_pane_messages(&pty_receiver);
    assert_eq!(closed_panes.len(), 1);
    assert!(closed_panes.contains(&PaneId::Terminal(2)));

    // No 插件 should be 卸载的
    let unloaded_plugins = collect_unload_plugin_messages(&plugin_receiver);
    assert!(unloaded_plugins.is_empty());

    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

// ============================================================================
// Suite 4: Integration 测试
// ============================================================================

#[test]
fn test_override_full_tiled_and_floating_together() {
    // 设置: 初始 布局 with 3 平铺 窗格 + 2 浮动 窗格
    let initial_kdl = r#"
        layout {
            pane command="htop"
            pane command="vim"
            pane
            floating_panes {
                pane {
                    x 10
                    y 10
                    width 40
                    height 20
                    command "cargo"
                    args "watch"
                }
                pane {
                    x 20
                    y 20
                    width 50
                    height 25
                    command "tail"
                    args "-f" "/var/log/syslog"
                }
            }
        }
    "#;

    let (initial_tiled, initial_floating) = parse_kdl_layout(initial_kdl);
    let terminal_ids = vec![(1, None), (2, None), (3, None)];
    let floating_terminal_ids = vec![(4, None), (5, None)];

    let size = Size {
        cols: 150,
        rows: 50,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        pty_receiver,
        plugin_receiver,
    ) = create_layout_applier_fixtures_with_receivers(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            initial_tiled,
            initial_floating,
            terminal_ids,
            floating_terminal_ids,
            HashMap::new(),
            1,
        )
        .unwrap();

    // Override: 布局 with 2 平铺 (htop, npm 启动) + 1 浮动 (cargo 监视)
    let override_kdl = r#"
        layout {
            pane command="htop"
            pane command="npm" {
                args "start"
            }
            floating_panes {
                pane {
                    x 30
                    y 30
                    width 60
                    height 30
                    command "cargo"
                    args "watch"
                }
            }
        }
    "#;

    let (override_tiled, override_floating) = parse_kdl_layout(override_kdl);
    let new_terminal_ids = vec![(6, None)];

    applier
        .override_tiled_panes_layout_for_existing_panes(
            &override_tiled,
            new_terminal_ids,
            &mut HashMap::new(),
            false,
            false,
            1,
        )
        .unwrap();

    applier
        .override_floating_panes_layout_for_existing_panes(
            &override_floating,
            vec![],
            &mut HashMap::new(),
            false,
            false,
        )
        .unwrap();

    // 平铺: htop 保存的 (终端(1)), vim and shell 关闭的, npm 启动 创建的 (终端(6))
    // 浮动: cargo 监视 保存的 (终端(4)), tail 关闭的
    // Total: 2 平铺 + 1 浮动

    // 验证 关闭 消息 were sent for vim (终端(2)), shell (终端(3)), and tail (终端(5))
    let closed_panes = collect_close_pane_messages(&pty_receiver);
    assert_eq!(closed_panes.len(), 3);
    assert!(closed_panes.contains(&PaneId::Terminal(2))); // vim
    assert!(closed_panes.contains(&PaneId::Terminal(3))); // shell
    assert!(closed_panes.contains(&PaneId::Terminal(5))); // tail

    // No 插件 should be 卸载的
    let unloaded_plugins = collect_unload_plugin_messages(&plugin_receiver);
    assert!(unloaded_plugins.is_empty());

    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_override_viewport_adjustment_with_borderless() {
    // 设置: 初始 布局 with borderless 窗格
    let initial_kdl = r#"
        layout {
            pane borderless=true
            pane
            pane borderless=true
        }
    "#;

    let (initial_tiled, initial_floating) = parse_kdl_layout(initial_kdl);
    let terminal_ids = vec![(1, None), (2, None), (3, None)];

    let size = Size {
        cols: 120,
        rows: 40,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        pty_receiver,
        plugin_receiver,
    ) = create_layout_applier_fixtures_with_receivers(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            initial_tiled,
            initial_floating,
            terminal_ids,
            vec![],
            HashMap::new(),
            1,
        )
        .unwrap();

    // Override: 布局 with different borderless 配置
    let override_kdl = r#"
        layout {
            pane
            pane borderless=true
        }
    "#;

    let (override_tiled, _) = parse_kdl_layout(override_kdl);

    applier
        .override_tiled_panes_layout_for_existing_panes(
            &override_tiled,
            vec![],
            &mut HashMap::new(),
            false,
            false,
            1,
        )
        .unwrap();

    // 视口 dimensions should be correctly adjusted for borderless 窗格

    // 验证 关闭 消息 was sent for at least the extra 窗格 (终端(3))
    // 泛型 窗格 without 命令 don't 匹配 exactly, so all 3 may be 关闭的
    let closed_panes = collect_close_pane_messages(&pty_receiver);
    assert!(closed_panes.len() >= 1);
    assert!(closed_panes.contains(&PaneId::Terminal(3)));

    // No 插件 should be 卸载的
    let unloaded_plugins = collect_unload_plugin_messages(&plugin_receiver);
    assert!(unloaded_plugins.is_empty());

    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_override_tiled_retain_terminal_panes_partial_match() {
    // 测试 that when retain_existing_terminal_panes is true, 终端 窗格 that don't 匹配
    // the new 布局 are 保留的 instead of being 关闭的.
    // 设置: Apply 初始 布局 with 3 窗格 running different 命令
    let initial_kdl = r#"
        layout {
            pane command="htop"
            pane command="vim"
            pane command="tail" {
                args "-f" "/var/log/syslog"
            }
        }
    "#;

    let (initial_tiled, initial_floating) = parse_kdl_layout(initial_kdl);
    let terminal_ids = vec![(1, None), (2, None), (3, None)];

    let size = Size {
        cols: 120,
        rows: 40,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        pty_receiver,
        plugin_receiver,
    ) = create_layout_applier_fixtures_with_receivers(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            initial_tiled,
            initial_floating,
            terminal_ids,
            vec![],
            HashMap::new(),
            1,
        )
        .unwrap();

    // Override: New 布局 with only vim and htop (tail is not in the new 布局)
    let override_kdl = r#"
        layout {
            pane command="vim"
            pane command="htop"
        }
    "#;

    let (override_tiled, _) = parse_kdl_layout(override_kdl);

    // Override with retain_existing_terminal_panes = true
    applier
        .override_tiled_panes_layout_for_existing_panes(
            &override_tiled,
            vec![],
            &mut HashMap::new(),
            true, // retain_existing_terminal_panes
            false,
            1,
        )
        .unwrap();

    // With retain_existing_terminal_panes = true:
    // - NO 终端 窗格 should be 关闭的 (not even tail)
    // - All 3 原始 终端 (终端(1), 终端(2), 终端(3)) should still exist
    // - vim and htop 窗格 should 匹配 the new 布局 positions
    // - tail 窗格 should be 保留的 and added after the matched 窗格

    // 验证 NO 关闭 消息 were sent
    let closed_panes = collect_close_pane_messages(&pty_receiver);
    assert_eq!(
        closed_panes.len(),
        0,
        "No terminal panes should be closed when retain_existing_terminal_panes is true"
    );

    // No 插件 should be 卸载的
    let unloaded_plugins = collect_unload_plugin_messages(&plugin_receiver);
    assert!(unloaded_plugins.is_empty());

    // All 3 原始 终端 should still exist
    assert_eq!(
        tiled_panes.visible_panes_count(),
        3,
        "All 3 terminal panes should be retained"
    );

    // we're not 断言 a 快照 here because adding 窗格 uses unstable sorting and so the
    // 测试 would be flaky
}

#[test]
fn test_override_tiled_retain_terminal_panes_no_matches() {
    // 测试 that when retain_existing_terminal_panes is true and NO 窗格 匹配,
    // all 原始 终端 are 保留的 AND new 终端 are 创建的.
    // 设置: Apply 初始 布局 with 3 窗格
    let initial_kdl = r#"
        layout {
            pane command="htop"
            pane command="vim"
            pane command="tail" {
                args "-f" "/var/log/syslog"
            }
        }
    "#;

    let (initial_tiled, initial_floating) = parse_kdl_layout(initial_kdl);
    let terminal_ids = vec![(1, None), (2, None), (3, None)];

    let size = Size {
        cols: 120,
        rows: 40,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        pty_receiver,
        plugin_receiver,
    ) = create_layout_applier_fixtures_with_receivers(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            initial_tiled,
            initial_floating,
            terminal_ids,
            vec![],
            HashMap::new(),
            1,
        )
        .unwrap();

    // Override: New 布局 with completely different 命令 (no matches)
    let override_kdl = r#"
        layout {
            pane command="cargo"
            pane command="npm"
            pane command="python"
        }
    "#;

    let (override_tiled, _) = parse_kdl_layout(override_kdl);
    let new_terminal_ids = vec![(4, None), (5, None), (6, None)];

    // Override with retain_existing_terminal_panes = true
    applier
        .override_tiled_panes_layout_for_existing_panes(
            &override_tiled,
            new_terminal_ids,
            &mut HashMap::new(),
            true, // retain_existing_terminal_panes
            false,
            1,
        )
        .unwrap();

    // With retain_existing_terminal_panes = true and no matches:
    // - NO 终端 窗格 should be 关闭的
    // - All 3 原始 终端 (终端(1), 终端(2), 终端(3)) should still exist
    // - 3 NEW 终端 (终端(4), 终端(5), 终端(6)) should be 创建的
    // - Total: 6 终端 窗格

    // 验证 NO 关闭 消息 were sent
    let closed_panes = collect_close_pane_messages(&pty_receiver);
    assert_eq!(
        closed_panes.len(),
        0,
        "No terminal panes should be closed when retain_existing_terminal_panes is true"
    );

    // No 插件 should be 卸载的
    let unloaded_plugins = collect_unload_plugin_messages(&plugin_receiver);
    assert!(unloaded_plugins.is_empty());

    // Should have 6 total 窗格 (3 原始 + 3 new)
    assert_eq!(
        tiled_panes.visible_panes_count(),
        6,
        "Should have 6 terminal panes (3 original + 3 new)"
    );

    // we're not 断言 a 快照 here because adding 窗格 uses unstable sorting and so the
    // 测试 would be flaky
}

#[test]
fn test_override_floating_retain_terminal_panes_partial_match() {
    // 测试 that when retain_existing_terminal_panes is true, 浮动 终端 窗格
    // that don't 匹配 the new 布局 are 保留的 instead of being 关闭的.
    // 设置: 1 平铺 窗格 + 2 浮动 窗格 running htop and vim
    let initial_kdl = r#"
        layout {
            pane
            floating_panes {
                pane {
                    x 10
                    y 10
                    width 40
                    height 20
                    command "htop"
                }
                pane {
                    x 20
                    y 20
                    width 50
                    height 25
                    command "vim"
                }
            }
        }
    "#;

    let (initial_tiled, initial_floating) = parse_kdl_layout(initial_kdl);
    let terminal_ids = vec![(1, None)];
    let floating_terminal_ids = vec![(2, None), (3, None)];

    let size = Size {
        cols: 120,
        rows: 40,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        pty_receiver,
        plugin_receiver,
    ) = create_layout_applier_fixtures_with_receivers(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            initial_tiled,
            initial_floating,
            terminal_ids,
            floating_terminal_ids,
            HashMap::new(),
            1,
        )
        .unwrap();

    // Override: 布局 with only htop (vim is not in the new 布局)
    let override_kdl = r#"
        layout {
            pane
            floating_panes {
                pane {
                    x 50
                    y 30
                    width 45
                    height 22
                    command "htop"
                }
            }
        }
    "#;

    let (_, override_floating) = parse_kdl_layout(override_kdl);

    // Override with retain_existing_terminal_panes = true
    applier
        .override_floating_panes_layout_for_existing_panes(
            &override_floating,
            vec![],
            &mut HashMap::new(),
            true, // retain_existing_terminal_panes
            false,
        )
        .unwrap();

    // With retain_existing_terminal_panes = true:
    // - NO 终端 窗格 should be 关闭的 (not even vim)
    // - Both 原始 浮动 终端 (终端(2), 终端(3)) should still exist
    // - htop 窗格 should 匹配 the new 布局 position
    // - vim 窗格 should be 保留的 as a 浮动 窗格

    // 验证 NO 关闭 消息 were sent
    let closed_panes = collect_close_pane_messages(&pty_receiver);
    assert_eq!(
        closed_panes.len(),
        0,
        "No terminal panes should be closed when retain_existing_terminal_panes is true"
    );

    // No 插件 should be 卸载的
    let unloaded_plugins = collect_unload_plugin_messages(&plugin_receiver);
    assert!(unloaded_plugins.is_empty());

    // Both 浮动 窗格 should still exist
    assert_eq!(
        floating_panes.visible_panes_count(),
        2,
        "Both floating terminal panes should be retained"
    );

    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_override_floating_retain_terminal_panes_no_matches() {
    // 测试 that when retain_existing_terminal_panes is true and NO 浮动 窗格 匹配,
    // all 原始 浮动 终端 are 保留的 AND new 浮动 终端 are 创建的.
    // 设置: 1 平铺 窗格 + 2 浮动 窗格 running htop and vim
    let initial_kdl = r#"
        layout {
            pane
            floating_panes {
                pane {
                    x 10
                    y 10
                    width 40
                    height 20
                    command "htop"
                }
                pane {
                    x 20
                    y 20
                    width 50
                    height 25
                    command "vim"
                }
            }
        }
    "#;

    let (initial_tiled, initial_floating) = parse_kdl_layout(initial_kdl);
    let terminal_ids = vec![(1, None)];
    let floating_terminal_ids = vec![(2, None), (3, None)];

    let size = Size {
        cols: 120,
        rows: 40,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        pty_receiver,
        plugin_receiver,
    ) = create_layout_applier_fixtures_with_receivers(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            initial_tiled,
            initial_floating,
            terminal_ids,
            floating_terminal_ids,
            HashMap::new(),
            1,
        )
        .unwrap();

    // Override: 布局 with 2 different 浮动 窗格 (top and emacs)
    let override_kdl = r#"
        layout {
            pane
            floating_panes {
                pane {
                    x 30
                    y 30
                    width 40
                    height 20
                    command "top"
                }
                pane {
                    x 40
                    y 40
                    width 50
                    height 25
                    command "emacs"
                }
            }
        }
    "#;

    let (_, override_floating) = parse_kdl_layout(override_kdl);
    let new_floating_terminal_ids = vec![(4, None), (5, None)];

    // Override with retain_existing_terminal_panes = true
    applier
        .override_floating_panes_layout_for_existing_panes(
            &override_floating,
            new_floating_terminal_ids,
            &mut HashMap::new(),
            true, // retain_existing_terminal_panes
            false,
        )
        .unwrap();

    // With retain_existing_terminal_panes = true and no matches:
    // - NO 终端 窗格 should be 关闭的
    // - Both 原始 浮动 终端 (终端(2), 终端(3)) should still exist
    // - 2 NEW 浮动 终端 (终端(4), 终端(5)) should be 创建的
    // - Total: 4 浮动 窗格

    // 验证 NO 关闭 消息 were sent
    let closed_panes = collect_close_pane_messages(&pty_receiver);
    assert_eq!(
        closed_panes.len(),
        0,
        "No terminal panes should be closed when retain_existing_terminal_panes is true"
    );

    // No 插件 should be 卸载的
    let unloaded_plugins = collect_unload_plugin_messages(&plugin_receiver);
    assert!(unloaded_plugins.is_empty());

    // Should have 4 浮动 窗格 (2 原始 + 2 new)
    assert_eq!(
        floating_panes.visible_panes_count(),
        4,
        "Should have 4 floating panes (2 original + 2 new)"
    );

    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_override_mixed_retain_terminal_panes_both_tiled_and_floating() {
    // 测试 that when retain_existing_terminal_panes is true, both 平铺 and 浮动
    // 终端 窗格 that don't 匹配 the new 布局 are 保留的.
    // 设置: Apply 初始 布局 with 3 平铺 窗格 + 1 浮动 窗格
    let initial_kdl = r#"
        layout {
            pane command="htop"
            pane command="vim"
            pane command="tail" {
                args "-f" "/var/log/syslog"
            }
            floating_panes {
                pane {
                    x 10
                    y 10
                    width 40
                    height 20
                    command "watch"
                    args "df" "-h"
                }
            }
        }
    "#;

    let (initial_tiled, initial_floating) = parse_kdl_layout(initial_kdl);
    let terminal_ids = vec![(1, None), (2, None), (3, None)];
    let floating_terminal_ids = vec![(4, None)];

    let size = Size {
        cols: 120,
        rows: 40,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        pty_receiver,
        plugin_receiver,
    ) = create_layout_applier_fixtures_with_receivers(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            initial_tiled,
            initial_floating,
            terminal_ids,
            floating_terminal_ids,
            HashMap::new(),
            1,
        )
        .unwrap();

    // Override: 2 平铺 (top, htop) + 1 浮动 (different 监视 命令)
    let override_kdl = r#"
        layout {
            pane command="top"
            pane command="htop"
            floating_panes {
                pane {
                    x 20
                    y 20
                    width 50
                    height 25
                    command "watch"
                    args "free" "-h"
                }
            }
        }
    "#;

    let (override_tiled, override_floating) = parse_kdl_layout(override_kdl);
    let new_terminal_ids = vec![(5, None)];
    let new_floating_terminal_ids = vec![(6, None)];

    let retain_existing_terminal_panes = true;
    let retain_existing_plugin_panes = false;
    applier
        .override_layout(
            override_tiled,
            override_floating,
            new_terminal_ids,
            new_floating_terminal_ids,
            HashMap::new(),
            retain_existing_terminal_panes,
            retain_existing_plugin_panes,
            1,
        )
        .unwrap();

    // With retain_existing_terminal_panes = true:
    // - NO 终端 窗格 should be 关闭的 (neither 平铺 nor 浮动)
    // - 原始 平铺 终端: 终端(1), 终端(2), 终端(3) 保留的
    //   (htop matches, so it's 重用的; vim and tail are 保留的)
    // - 原始 浮动 终端: 终端(4) 保留的
    // - New 平铺 终端: 终端(5) 创建的 (for top, htop matches)
    // - New 浮动 终端: 终端(6) 创建的 (different 监视 命令)

    // 验证 NO 关闭 消息 were sent
    let closed_panes = collect_close_pane_messages(&pty_receiver);
    assert_eq!(
        closed_panes.len(),
        0,
        "No terminal panes should be closed when retain_existing_terminal_panes is true"
    );

    // No 插件 should be 卸载的
    let unloaded_plugins = collect_unload_plugin_messages(&plugin_receiver);
    assert!(unloaded_plugins.is_empty());

    // 检查 平铺 窗格 count (3 原始 + 1 new = 4)
    assert_eq!(
        tiled_panes.visible_panes_count(),
        4,
        "Should have 4 tiled panes (3 original + 1 new)"
    );

    // 检查 浮动 窗格 count (1 原始 + 1 new = 2)
    assert_eq!(
        floating_panes.visible_panes_count(),
        2,
        "Should have 2 floating panes (1 original + 1 new)"
    );

    // we're not 断言 a 快照 here because adding 窗格 uses unstable sorting and so the
    // 测试 would be flaky
}

#[test]
fn test_override_retain_terminal_but_close_plugin_panes() {
    // 测试 that when retain_existing_terminal_panes is true, the 标志 ONLY affects
    // 终端 窗格 and 插件 窗格 are still 关闭的 as normal.
    // 设置: 初始 布局 with 2 终端 窗格 + 1 插件 窗格
    let initial_kdl = r#"
        layout {
            pane command="htop"
            pane command="vim"
            pane {
                plugin location="zellij:tab-bar"
            }
        }
    "#;

    let (initial_tiled, initial_floating) = parse_kdl_layout(initial_kdl);
    let terminal_ids = vec![(1, None), (2, None)];

    let mut initial_plugin_ids = HashMap::new();
    let tab_bar_plugin = RunPluginOrAlias::from_url("zellij:tab-bar", &None, None, None).unwrap();
    initial_plugin_ids.insert(tab_bar_plugin.clone(), vec![100]);

    let size = Size {
        cols: 120,
        rows: 40,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        pty_receiver,
        plugin_receiver,
    ) = create_layout_applier_fixtures_with_receivers(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            initial_tiled,
            initial_floating,
            terminal_ids,
            vec![],
            initial_plugin_ids,
            1,
        )
        .unwrap();

    // Override: New 布局 with only htop (vim and 标签页-bar 插件 not in new 布局)
    let override_kdl = r#"
        layout {
            pane command="htop"
        }
    "#;

    let (override_tiled, _) = parse_kdl_layout(override_kdl);

    // Override with retain_existing_terminal_panes = true
    applier
        .override_tiled_panes_layout_for_existing_panes(
            &override_tiled,
            vec![],
            &mut HashMap::new(),
            true, // retain_existing_terminal_panes
            false,
            1,
        )
        .unwrap();

    // With retain_existing_terminal_panes = true:
    // - 终端 窗格 NOT 关闭的: 终端(1) matched (htop), 终端(2) 保留的 (vim)
    // - 插件 窗格 IS 关闭的: 插件(100) 卸载的 (标签页-bar)
    // - Both 终端 should exist, but 插件 should be gone

    // No 终端 窗格 should be 关闭的
    let closed_panes = collect_close_pane_messages(&pty_receiver);
    assert_eq!(
        closed_panes.len(),
        0,
        "No terminal panes should be closed when retain_existing_terminal_panes is true"
    );

    // 插件 窗格 should be 卸载的
    let unloaded_plugins = collect_unload_plugin_messages(&plugin_receiver);
    assert_eq!(
        unloaded_plugins.len(),
        1,
        "Plugin pane should be unloaded even with retain_existing_terminal_panes"
    );
    assert!(
        unloaded_plugins.contains(&100),
        "Tab-bar plugin (100) should be unloaded"
    );

    // Both 终端 should exist
    assert_eq!(
        tiled_panes.visible_panes_count(),
        2,
        "Both terminal panes should be retained"
    );

    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}

#[test]
fn test_override_tiled_retain_plugin_panes_partial_match() {
    // 验证 that when retain_existing_plugin_panes = true, 插件 窗格 that don't 匹配
    // the new 布局 are 保留的 instead of being 关闭的
    let initial_kdl = r#"
        layout {
            pane
            pane {
                plugin location="zellij:tab-bar"
            }
            pane {
                plugin location="zellij:status-bar"
            }
            pane {
                plugin location="file:///path/to/custom.wasm"
            }
        }
    "#;

    let (initial_tiled, initial_floating) = parse_kdl_layout(initial_kdl);

    let terminal_ids = vec![(1, None)];

    let mut initial_plugin_ids = HashMap::new();
    let tab_bar = RunPluginOrAlias::from_url("zellij:tab-bar", &None, None, None).unwrap();
    let status_bar = RunPluginOrAlias::from_url("zellij:status-bar", &None, None, None).unwrap();
    let custom =
        RunPluginOrAlias::from_url("file:///path/to/custom.wasm", &None, None, None).unwrap();
    initial_plugin_ids.insert(tab_bar, vec![100]);
    initial_plugin_ids.insert(status_bar, vec![101]);
    initial_plugin_ids.insert(custom, vec![102]);

    let size = Size {
        cols: 120,
        rows: 40,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        pty_receiver,
        plugin_receiver,
    ) = create_layout_applier_fixtures_with_receivers(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            initial_tiled,
            initial_floating,
            terminal_ids,
            vec![],
            initial_plugin_ids,
            1,
        )
        .unwrap();

    let override_kdl = r#"
        layout {
            pane
            pane {
                plugin location="zellij:tab-bar"
            }
            pane {
                plugin location="zellij:status-bar"
            }
        }
    "#;

    let (override_tiled, _) = parse_kdl_layout(override_kdl);

    applier
        .override_tiled_panes_layout_for_existing_panes(
            &override_tiled,
            vec![],
            &mut HashMap::new(),
            false,
            true,
            1,
        )
        .unwrap();

    // 验证 NO 插件 窗格 were 卸载的
    let unloaded_plugins = collect_unload_plugin_messages(&plugin_receiver);
    assert_eq!(
        unloaded_plugins.len(),
        0,
        "No plugin panes should be unloaded when retain_existing_plugin_panes is true"
    );

    // No 终端 should be 关闭的
    let closed_panes = collect_close_pane_messages(&pty_receiver);
    assert!(closed_panes.is_empty());

    // All 4 窗格 should exist (1 终端 + 3 插件)
    assert_eq!(
        tiled_panes.visible_panes_count(),
        4,
        "All plugin panes should be retained"
    );
}

#[test]
fn test_override_tiled_retain_plugin_panes_no_matches() {
    // When NO 插件 匹配 the new 布局, all 原始 插件 are 保留的 AND new 插件 are 创建的
    let initial_kdl = r#"
        layout {
            pane
            pane {
                plugin location="zellij:tab-bar"
            }
            pane {
                plugin location="zellij:status-bar"
            }
            pane {
                plugin location="zellij:compact-bar"
            }
        }
    "#;

    let (initial_tiled, initial_floating) = parse_kdl_layout(initial_kdl);

    let terminal_ids = vec![(1, None)];

    let mut initial_plugin_ids = HashMap::new();
    let tab_bar = RunPluginOrAlias::from_url("zellij:tab-bar", &None, None, None).unwrap();
    let status_bar = RunPluginOrAlias::from_url("zellij:status-bar", &None, None, None).unwrap();
    let compact_bar = RunPluginOrAlias::from_url("zellij:compact-bar", &None, None, None).unwrap();
    initial_plugin_ids.insert(tab_bar, vec![100]);
    initial_plugin_ids.insert(status_bar, vec![101]);
    initial_plugin_ids.insert(compact_bar, vec![102]);

    let size = Size {
        cols: 120,
        rows: 40,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        _pty_receiver,
        plugin_receiver,
    ) = create_layout_applier_fixtures_with_receivers(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            initial_tiled,
            initial_floating,
            terminal_ids,
            vec![],
            initial_plugin_ids,
            1,
        )
        .unwrap();

    let override_kdl = r#"
        layout {
            pane
            pane {
                plugin location="file:///path/to/plugin1.wasm"
            }
            pane {
                plugin location="file:///path/to/plugin2.wasm"
            }
            pane {
                plugin location="file:///path/to/plugin3.wasm"
            }
        }
    "#;

    let (override_tiled, _) = parse_kdl_layout(override_kdl);

    let mut override_plugin_ids = HashMap::new();
    let plugin1 =
        RunPluginOrAlias::from_url("file:///path/to/plugin1.wasm", &None, None, None).unwrap();
    let plugin2 =
        RunPluginOrAlias::from_url("file:///path/to/plugin2.wasm", &None, None, None).unwrap();
    let plugin3 =
        RunPluginOrAlias::from_url("file:///path/to/plugin3.wasm", &None, None, None).unwrap();
    override_plugin_ids.insert(plugin1, vec![103]);
    override_plugin_ids.insert(plugin2, vec![104]);
    override_plugin_ids.insert(plugin3, vec![105]);

    applier
        .override_tiled_panes_layout_for_existing_panes(
            &override_tiled,
            vec![],
            &mut override_plugin_ids,
            false,
            true,
            1,
        )
        .unwrap();

    // 验证 NO 插件 窗格 were 卸载的
    let unloaded_plugins = collect_unload_plugin_messages(&plugin_receiver);
    assert_eq!(
        unloaded_plugins.len(),
        0,
        "No plugin panes should be unloaded when retain_existing_plugin_panes is true"
    );

    // Total: 1 终端 + 6 插件 (3 原始 + 3 new)
    assert_eq!(
        tiled_panes.visible_panes_count(),
        7,
        "All original plugins retained and new plugins created"
    );
}

#[test]
fn test_override_floating_retain_plugin_panes_partial_match() {
    // 浮动 插件 窗格 that don't 匹配 the new 布局 are 保留的
    let initial_kdl = r#"
        layout {
            pane
            floating_panes {
                pane {
                    x 10
                    y 10
                    width 40
                    height 20
                    plugin location="zellij:tab-bar"
                }
                pane {
                    x 20
                    y 20
                    width 50
                    height 25
                    plugin location="zellij:status-bar"
                }
            }
        }
    "#;

    let (initial_tiled, initial_floating) = parse_kdl_layout(initial_kdl);

    let terminal_ids = vec![(1, None)];

    let mut initial_plugin_ids = HashMap::new();
    let tab_bar = RunPluginOrAlias::from_url("zellij:tab-bar", &None, None, None).unwrap();
    let status_bar = RunPluginOrAlias::from_url("zellij:status-bar", &None, None, None).unwrap();
    initial_plugin_ids.insert(tab_bar, vec![100]);
    initial_plugin_ids.insert(status_bar, vec![101]);

    let size = Size {
        cols: 120,
        rows: 40,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        pty_receiver,
        plugin_receiver,
    ) = create_layout_applier_fixtures_with_receivers(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            initial_tiled,
            initial_floating,
            terminal_ids,
            vec![],
            initial_plugin_ids,
            1,
        )
        .unwrap();

    let override_kdl = r#"
        layout {
            pane
            floating_panes {
                pane {
                    x 15
                    y 15
                    width 45
                    height 22
                    plugin location="zellij:tab-bar"
                }
            }
        }
    "#;

    let (_, override_floating) = parse_kdl_layout(override_kdl);

    applier
        .override_floating_panes_layout_for_existing_panes(
            &override_floating,
            vec![],
            &mut HashMap::new(),
            false,
            true,
        )
        .unwrap();

    // 验证 NO 插件 窗格 were 卸载的
    let unloaded_plugins = collect_unload_plugin_messages(&plugin_receiver);
    assert_eq!(
        unloaded_plugins.len(),
        0,
        "No floating plugin panes should be unloaded when retain_existing_plugin_panes is true"
    );

    // No 终端 should be 关闭的
    let closed_panes = collect_close_pane_messages(&pty_receiver);
    assert!(closed_panes.is_empty());

    // Both 浮动 插件 should still exist
    assert_eq!(
        floating_panes.visible_panes_count(),
        2,
        "Both floating plugin panes should be retained"
    );

    // 验证 平铺 窗格 exists
    assert_eq!(tiled_panes.visible_panes_count(), 1);
}

#[test]
fn test_override_floating_retain_plugin_panes_no_matches() {
    // All 原始 浮动 插件 are 保留的 AND new 浮动 插件 are 创建的 when there are no matches
    let initial_kdl = r#"
        layout {
            pane
            floating_panes {
                pane {
                    x 10
                    y 10
                    width 40
                    height 20
                    plugin location="zellij:tab-bar"
                }
                pane {
                    x 20
                    y 20
                    width 50
                    height 25
                    plugin location="zellij:status-bar"
                }
            }
        }
    "#;

    let (initial_tiled, initial_floating) = parse_kdl_layout(initial_kdl);

    let terminal_ids = vec![(1, None)];

    let mut initial_plugin_ids = HashMap::new();
    let tab_bar = RunPluginOrAlias::from_url("zellij:tab-bar", &None, None, None).unwrap();
    let status_bar = RunPluginOrAlias::from_url("zellij:status-bar", &None, None, None).unwrap();
    initial_plugin_ids.insert(tab_bar, vec![100]);
    initial_plugin_ids.insert(status_bar, vec![101]);

    let size = Size {
        cols: 120,
        rows: 40,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        _pty_receiver,
        plugin_receiver,
    ) = create_layout_applier_fixtures_with_receivers(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            initial_tiled,
            initial_floating,
            terminal_ids,
            vec![],
            initial_plugin_ids,
            1,
        )
        .unwrap();

    let override_kdl = r#"
        layout {
            pane
            floating_panes {
                pane {
                    x 15
                    y 15
                    width 45
                    height 22
                    plugin location="file:///path/to/plugin1.wasm"
                }
                pane {
                    x 25
                    y 25
                    width 55
                    height 27
                    plugin location="file:///path/to/plugin2.wasm"
                }
            }
        }
    "#;

    let (_, override_floating) = parse_kdl_layout(override_kdl);

    let mut override_plugin_ids = HashMap::new();
    let plugin1 =
        RunPluginOrAlias::from_url("file:///path/to/plugin1.wasm", &None, None, None).unwrap();
    let plugin2 =
        RunPluginOrAlias::from_url("file:///path/to/plugin2.wasm", &None, None, None).unwrap();
    override_plugin_ids.insert(plugin1, vec![102]);
    override_plugin_ids.insert(plugin2, vec![103]);

    applier
        .override_floating_panes_layout_for_existing_panes(
            &override_floating,
            vec![],
            &mut override_plugin_ids,
            false,
            true,
        )
        .unwrap();

    // 验证 NO 插件 窗格 were 卸载的
    let unloaded_plugins = collect_unload_plugin_messages(&plugin_receiver);
    assert_eq!(
        unloaded_plugins.len(),
        0,
        "No floating plugin panes should be unloaded when retain_existing_plugin_panes is true"
    );

    // Total: 4 浮动 插件 (2 原始 + 2 new)
    assert_eq!(
        floating_panes.visible_panes_count(),
        4,
        "All original floating plugins retained and new plugins created"
    );

    // 1 平铺 终端 窗格
    assert_eq!(tiled_panes.visible_panes_count(), 1);
}

#[test]
fn test_override_mixed_retain_plugin_panes_both_tiled_and_floating() {
    // Both 平铺 and 浮动 插件 窗格 are 保留的 when the 标志 is true
    let initial_kdl = r#"
        layout {
            pane command="htop"
            pane command="vim"
            pane {
                plugin location="zellij:tab-bar"
            }
            floating_panes {
                pane {
                    x 10
                    y 10
                    width 40
                    height 20
                    plugin location="zellij:status-bar"
                }
            }
        }
    "#;

    let (initial_tiled, initial_floating) = parse_kdl_layout(initial_kdl);

    let terminal_ids = vec![(1, None), (2, None)];

    let mut initial_plugin_ids = HashMap::new();
    let tab_bar = RunPluginOrAlias::from_url("zellij:tab-bar", &None, None, None).unwrap();
    let status_bar = RunPluginOrAlias::from_url("zellij:status-bar", &None, None, None).unwrap();
    initial_plugin_ids.insert(tab_bar, vec![100]);
    initial_plugin_ids.insert(status_bar, vec![101]);

    let size = Size {
        cols: 120,
        rows: 40,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        pty_receiver,
        plugin_receiver,
    ) = create_layout_applier_fixtures_with_receivers(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            initial_tiled,
            initial_floating,
            terminal_ids,
            vec![],
            initial_plugin_ids,
            1,
        )
        .unwrap();

    let override_kdl = r#"
        layout {
            pane command="htop"
            pane {
                plugin location="zellij:tab-bar"
            }
            pane {
                plugin location="zellij:compact-bar"
            }
            floating_panes {
                pane {
                    x 20
                    y 20
                    width 50
                    height 25
                    plugin location="file:///path/to/custom.wasm"
                }
            }
        }
    "#;

    let (override_tiled, override_floating) = parse_kdl_layout(override_kdl);

    let mut override_plugin_ids = HashMap::new();
    let compact_bar = RunPluginOrAlias::from_url("zellij:compact-bar", &None, None, None).unwrap();
    let custom =
        RunPluginOrAlias::from_url("file:///path/to/custom.wasm", &None, None, None).unwrap();
    override_plugin_ids.insert(compact_bar, vec![102]);
    override_plugin_ids.insert(custom, vec![103]);

    let retain_existing_terminal_panes = false;
    let retain_existing_plugin_panes = true;
    applier
        .override_layout(
            override_tiled,
            override_floating,
            vec![],
            vec![],
            override_plugin_ids,
            retain_existing_terminal_panes,
            retain_existing_plugin_panes,
            1,
        )
        .unwrap();

    // 验证 NO 插件 窗格 were 卸载的
    let unloaded_plugins = collect_unload_plugin_messages(&plugin_receiver);
    assert_eq!(
        unloaded_plugins.len(),
        0,
        "No plugin panes should be unloaded when retain_existing_plugin_panes is true"
    );

    // vim 终端 should be 关闭的 (doesn't 匹配 布局)
    let closed_panes = collect_close_pane_messages(&pty_receiver);
    assert_eq!(closed_panes.len(), 1, "vim terminal should be closed");
    assert!(
        closed_panes.contains(&PaneId::Terminal(2)),
        "Terminal(2) (vim) should be closed"
    );

    // 验证 插件 exist (exact counts may vary based on where 保留的 窗格 land)
    assert!(
        tiled_panes.visible_panes_count() >= 2,
        "At least htop and matched plugins"
    );
    assert!(
        floating_panes.visible_panes_count() >= 1,
        "At least one floating plugin"
    );
}

#[test]
fn test_override_retain_plugin_but_close_terminal_panes() {
    // 验证 that retain_existing_plugin_panes = true ONLY affects 插件 窗格; 终端 are still 关闭的 normally
    let initial_kdl = r#"
        layout {
            pane command="htop"
            pane command="vim"
            pane {
                plugin location="zellij:tab-bar"
            }
        }
    "#;

    let (initial_tiled, initial_floating) = parse_kdl_layout(initial_kdl);

    let terminal_ids = vec![(1, None), (2, None)];

    let mut initial_plugin_ids = HashMap::new();
    let tab_bar = RunPluginOrAlias::from_url("zellij:tab-bar", &None, None, None).unwrap();
    initial_plugin_ids.insert(tab_bar, vec![100]);

    let size = Size {
        cols: 120,
        rows: 40,
    };
    let (
        viewport,
        senders,
        sixel_image_store,
        link_handler,
        terminal_emulator_colors,
        terminal_emulator_color_codes,
        character_cell_size,
        connected_clients,
        style,
        display_area,
        mut tiled_panes,
        mut floating_panes,
        draw_pane_frames,
        mut focus_pane_id,
        os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        pty_receiver,
        plugin_receiver,
    ) = create_layout_applier_fixtures_with_receivers(size);

    let mut applier = LayoutApplier::new(
        &viewport,
        &senders,
        &sixel_image_store,
        &Rc::new(RefCell::new(KittyImageStore::default())),
        &link_handler,
        &terminal_emulator_colors,
        &terminal_emulator_color_codes,
        &character_cell_size,
        &connected_clients,
        &style,
        &display_area,
        &mut tiled_panes,
        &mut floating_panes,
        draw_pane_frames,
        &mut focus_pane_id,
        &os_api,
        debug,
        arrow_fonts,
        styled_underlines,
        osc8_hyperlinks,
        explicitly_disable_kitty_keyboard_protocol,
        None,
    );

    applier
        .apply_layout(
            initial_tiled,
            initial_floating,
            terminal_ids,
            vec![],
            initial_plugin_ids,
            1,
        )
        .unwrap();

    let override_kdl = r#"
        layout {
            pane command="htop"
        }
    "#;

    let (override_tiled, _) = parse_kdl_layout(override_kdl);

    applier
        .override_tiled_panes_layout_for_existing_panes(
            &override_tiled,
            vec![],
            &mut HashMap::new(),
            false,
            true,
            1,
        )
        .unwrap();

    // vim 终端 should be 关闭的
    let closed_panes = collect_close_pane_messages(&pty_receiver);
    assert_eq!(
        closed_panes.len(),
        1,
        "Terminal pane should be closed even with retain_existing_plugin_panes"
    );
    assert!(
        closed_panes.contains(&PaneId::Terminal(2)),
        "vim (Terminal(2)) should be closed"
    );

    // 插件 should NOT be 卸载的
    let unloaded_plugins = collect_unload_plugin_messages(&plugin_receiver);
    assert_eq!(
        unloaded_plugins.len(),
        0,
        "No plugin panes should be unloaded"
    );

    // 最终 窗格: htop + 标签页-bar = 2
    assert_eq!(
        tiled_panes.visible_panes_count(),
        2,
        "htop and tab-bar should remain"
    );

    assert_snapshot!(take_pane_state_snapshot(
        &tiled_panes,
        &floating_panes,
        &focus_pane_id,
        &viewport,
        &display_area,
    ));
}
