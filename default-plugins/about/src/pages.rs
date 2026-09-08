use zellij_tile::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

use zellij_tile::prelude::actions::Action;

use crate::active_component::{ActiveComponent, ClickAction};
use crate::keybindings::{
    apply_missing_binds, ApplyStatus, ExpectedBind, Feature, KeybindingState,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PageKind {
    MainScreen,
    NestedSessions,
    PaneFocus,
    ScrollByCommand,
    UpdateKeybindings,
    Other,
}

#[derive(Debug)]
pub struct Page {
    title: Option<Text>,
    components_to_render: Vec<RenderedComponent>,
    has_hover: bool,
    hovering_over_link: bool,
    menu_item_is_selected: bool,
    pub is_main_screen: bool,
    pub kind: PageKind,
}

impl Page {
    pub fn new_main_screen(
        link_executable: Rc<RefCell<String>>,
        zellij_version: String,
        base_mode: Rc<RefCell<InputMode>>,
        is_release_notes: bool,
        keybinding_state: Rc<RefCell<KeybindingState>>,
    ) -> Self {
        let has_missing_binds = keybinding_state.borrow().has_missing_binds();
        let main_screen_builder: Rc<dyn Fn() -> Page> = Rc::new({
            let link_executable = link_executable.clone();
            let zellij_version = zellij_version.clone();
            let base_mode = base_mode.clone();
            let keybinding_state = keybinding_state.clone();
            move || {
                Page::new_main_screen(
                    link_executable.clone(),
                    zellij_version.clone(),
                    base_mode.clone(),
                    is_release_notes,
                    keybinding_state.clone(),
                )
            }
        });
        let page = Page::new()
            .main_screen()
            .with_kind(PageKind::MainScreen)
            .with_title(main_screen_title(zellij_version.clone(), is_release_notes))
            .with_bulletin_list(BulletinList::new(whats_new_title()).with_items(vec![
                    ActiveComponent::new(TextOrCustomRender::Text(main_menu_item(
                        "嵌套会话",
                    )))
                    .with_hover(TextOrCustomRender::Text(
                        main_menu_item("嵌套会话").selected(),
                    ))
                    .with_left_click_action(ClickAction::new_change_page({
                        let keybinding_state = keybinding_state.clone();
                        let main_screen_builder = main_screen_builder.clone();
                        move || Page::new_nested_sessions(keybinding_state, main_screen_builder)
                    })),
                    ActiveComponent::new(TextOrCustomRender::Text(main_menu_item(
                        "Kitty 图形协议",
                    )))
                    .with_hover(TextOrCustomRender::Text(
                        main_menu_item("Kitty 图形协议").selected(),
                    ))
                    .with_left_click_action(ClickAction::new_change_page({
                        move || Page::new_kitty_graphics()
                    })),
                    ActiveComponent::new(TextOrCustomRender::Text(main_menu_item(
                        "按命令滚动",
                    )))
                    .with_hover(TextOrCustomRender::Text(
                        main_menu_item("按命令滚动").selected(),
                    ))
                    .with_left_click_action(ClickAction::new_change_page({
                        let keybinding_state = keybinding_state.clone();
                        let main_screen_builder = main_screen_builder.clone();
                        move || Page::new_scroll_by_command(keybinding_state, main_screen_builder)
                    })),
                    ActiveComponent::new(TextOrCustomRender::Text(main_menu_item(
                        "移动端 Web 界面",
                    )))
                    .with_hover(TextOrCustomRender::Text(
                        main_menu_item("移动端 Web 界面").selected(),
                    ))
                    .with_left_click_action(ClickAction::new_change_page({
                        let link_executable = link_executable.clone();
                        move || Page::new_mobile_web_ui(link_executable)
                    })),
                    ActiveComponent::new(TextOrCustomRender::Text(main_menu_item("新界面")))
                    .with_hover(TextOrCustomRender::Text(
                        main_menu_item("新界面").selected(),
                    ))
                    .with_left_click_action(ClickAction::new_change_page({
                        move || Page::new_ui()
                    })),
                    ActiveComponent::new(TextOrCustomRender::Text(main_menu_item(
                        "按客户端调整标签页大小",
                    )))
                    .with_hover(TextOrCustomRender::Text(
                        main_menu_item("按客户端调整标签页大小").selected(),
                    ))
                    .with_left_click_action(ClickAction::new_change_page({
                        move || Page::new_per_client_tab_sizes()
                    })),
                    ActiveComponent::new(TextOrCustomRender::Text(main_menu_item(
                        "聚焦上一个窗格与全屏浮动窗格",
                    )))
                    .with_hover(TextOrCustomRender::Text(
                        main_menu_item("聚焦上一个窗格与全屏浮动窗格").selected(),
                    ))
                    .with_left_click_action(ClickAction::new_change_page({
                        let keybinding_state = keybinding_state.clone();
                        let main_screen_builder = main_screen_builder.clone();
                        move || Page::new_pane_focus(keybinding_state, main_screen_builder)
                    })),
                ]));
        page.with_paragraph(vec![ComponentLine::new(vec![
            ActiveComponent::new(TextOrCustomRender::Text(Text::new("完整更新日志： "))),
            ActiveComponent::new(TextOrCustomRender::Text(changelog_link_unselected(
                zellij_version.clone(),
            )))
            .with_hover(TextOrCustomRender::CustomRender(
                Box::new(changelog_link_selected(zellij_version.clone())),
                Box::new(changelog_link_selected_len(zellij_version.clone())),
            ))
            .with_left_click_action(ClickAction::new_open_link(
                format!(
                    "https://github.com/zellij-org/zellij/releases/tag/v{}",
                    zellij_version.clone()
                ),
                link_executable.clone(),
            )),
        ])])
        .with_paragraph(vec![ComponentLine::new(vec![
            ActiveComponent::new(TextOrCustomRender::Text(support_the_developer_text())),
            ActiveComponent::new(TextOrCustomRender::Text(sponsors_link_text_unselected()))
                .with_hover(TextOrCustomRender::CustomRender(
                    Box::new(sponsors_link_text_selected),
                    Box::new(sponsors_link_text_selected_len),
                ))
                .with_left_click_action(ClickAction::new_open_link(
                    "https://github.com/sponsors/imsnif".to_owned(),
                    link_executable.clone(),
                )),
        ])])
        .with_help(if is_release_notes {
            Box::new(move |hovering_over_link, menu_item_is_selected| {
                release_notes_main_help(
                    hovering_over_link,
                    menu_item_is_selected,
                    has_missing_binds,
                )
            })
        } else {
            Box::new(move |hovering_over_link, menu_item_is_selected| {
                main_screen_help_text(hovering_over_link, menu_item_is_selected, has_missing_binds)
            })
        })
    }
    pub fn new_nested_sessions(
        keybinding_state: Rc<RefCell<KeybindingState>>,
        main_screen_builder: Rc<dyn Fn() -> Page>,
    ) -> Page {
        let base_mode_name = keybinding_state.borrow().base_mode_name();
        let mut option_lines = vec![
            option_title_line("1. 放大并控制此会话"),
            option_text_line(
                "   该会话将占据整个屏幕，并可随时切换开关。",
            ),
        ];
        option_lines.extend(bind_lines(
            &keybinding_state,
            &[Action::ToggleHostFullscreen],
            "   ",
        ));
        option_lines.push(option_title_line("2. 聚焦时控制此会话"));
        option_lines.push(option_text_line(
            "   当此窗格获得焦点时，键位绑定将发送到此会话。",
        ));
        option_lines.push(option_text_line(
            "   然后你可以返回当前会话。",
        ));
        option_lines.extend(bind_lines(
            &keybinding_state,
            &[Action::FocusHostSession, Action::FocusGuestSession],
            "   ",
        ));
        let mut page = Page::new()
            .with_kind(PageKind::NestedSessions)
            .with_title(Text::new("嵌套会话").color_range(0, ..))
            .with_paragraph(vec![
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new(
                        "Zellij 现在可以检测到它是在另一个 Zellij 会话中启动的。",
                    ),
                ))]),
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new(format!(
                        "让你在（SESSION 模式，返回 {}）之间做出选择：",
                        base_mode_name
                    ))
                    .color_substring(3, "SESSION")
                    .color_substring(3, &base_mode_name),
                ))]),
            ])
            .with_paragraph(option_lines);
        if let Some(missing_binds_note) = missing_binds_note(
            &keybinding_state,
            &main_screen_builder,
            Feature::NestedSessions,
        ) {
            page = page.with_paragraph(missing_binds_note);
        }
        page.with_help(Box::new(|_hovering_over_link, _menu_item_is_selected| {
            esc_to_go_back_help()
        }))
    }
    pub fn new_pane_focus(
        keybinding_state: Rc<RefCell<KeybindingState>>,
        main_screen_builder: Rc<dyn Fn() -> Page>,
    ) -> Page {
        let mut page = Page::new()
            .with_kind(PageKind::PaneFocus)
            .with_title(
                Text::new("聚焦上一个窗格与全屏浮动窗格").color_range(0, ..),
            )
            .with_paragraph(vec![
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("新操作会将焦点返回到此前聚焦的窗格，"),
                ))]),
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("即当前窗格之前的那一个，这样两个窗格可以用一次"),
                ))]),
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("按键来回切换。"),
                ))]),
            ])
            .with_paragraph(bind_lines(&keybinding_state, &[Action::FocusLastPane], ""))
            .with_paragraph(vec![
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("浮动窗格现在也可以像平铺窗格一样全屏显示。"),
                ))]),
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("聚焦的浮动窗格会扩展到整个视口，或在隐藏界面时扩展到"),
                ))]),
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("整个屏幕。"),
                ))]),
            ]);
        if let Some(missing_binds_note) =
            missing_binds_note(&keybinding_state, &main_screen_builder, Feature::PaneFocus)
        {
            page = page.with_paragraph(missing_binds_note);
        }
        page.with_help(Box::new(|_hovering_over_link, _menu_item_is_selected| {
            esc_to_go_back_help()
        }))
    }
    pub fn new_scroll_by_command(
        keybinding_state: Rc<RefCell<KeybindingState>>,
        main_screen_builder: Rc<dyn Fn() -> Page>,
    ) -> Page {
        let mut command_bind_lines = merged_bind_line(
            &keybinding_state,
            &[Action::ScrollToPreviousPrompt, Action::ScrollToNextPrompt],
            "滚动到上一个 / 下一个命令",
        );
        command_bind_lines.extend(bind_lines(
            &keybinding_state,
            &[
                Action::SelectCommandAtScrollPosition,
                Action::CopyLastCommandOutput,
            ],
            "",
        ));
        let mut page = Page::new()
            .with_kind(PageKind::ScrollByCommand)
            .with_title(Text::new("按命令滚动").color_range(0, ..))
            .with_paragraph(vec![ComponentLine::new(vec![ActiveComponent::new(
                TextOrCustomRender::Text(
                    Text::new(
                        "Zellij 现在可以导航 shell（OSC 133）标记的命令：",
                    )
                    .color_substring(2, "OSC 133"),
                ),
            )])])
            .with_paragraph(command_bind_lines)
            .with_paragraph(vec![
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("- 按住 <Alt> 并滚动鼠标滚轮可在命令间滚动。")
                        .color_substring(3, "<Alt>"),
                ))]),
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("- 三次单击可选择整个命令。")
                        .color_substring(2, "三次单击"),
                ))]),
                opt_out_line("三次单击选择", "osc133_command_selection false"),
            ]);
        if let Some(missing_binds_note) = missing_binds_note(
            &keybinding_state,
            &main_screen_builder,
            Feature::ScrollByCommand,
        ) {
            page = page.with_paragraph(missing_binds_note);
        }
        page.with_help(Box::new(|_hovering_over_link, _menu_item_is_selected| {
            esc_to_go_back_help()
        }))
    }
    pub fn new_update_keybindings(
        keybinding_state: Rc<RefCell<KeybindingState>>,
        main_screen_builder: Rc<dyn Fn() -> Page>,
    ) -> Page {
        let base_mode_name = keybinding_state.borrow().base_mode_name();
        let keybind_items: Vec<ActiveComponent> = keybinding_state
            .borrow()
            .binds_to_add()
            .iter()
            .map(|bind| {
                let mode_name = bind.mode_name();
                let bind_text = if bind.returns_to_base_mode {
                    format!(
                        "{} 模式 + {} - {}，返回 {}",
                        mode_name,
                        bind.key_text(),
                        bind.description,
                        base_mode_name
                    )
                } else {
                    format!(
                        "{} mode + {} - {}",
                        mode_name,
                        bind.key_text(),
                        bind.description
                    )
                };
                ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new(bind_text)
                        .color_substring(3, &mode_name)
                        .color_substring(3, &bind.key_text())
                        .color_substring(3, &base_mode_name),
                ))
            })
            .collect();
        let conflicting_binds: Vec<ComponentLine> = keybinding_state
            .borrow()
            .conflicting_binds()
            .iter()
            .map(|(bind, currently_bound_to)| {
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new(format!(
                        "{} 当前绑定到 {}，将被覆盖",
                        bind.key_text(),
                        currently_bound_to
                    ))
                    .color_substring(3, &bind.key_text())
                    .color_substring(0, currently_bound_to),
                ))])
            })
            .collect();
        let status = keybinding_state.borrow().status().clone();
        let mut page = Page::new()
            .with_kind(PageKind::UpdateKeybindings)
            .with_title(Text::new("更新键位绑定").color_range(0, ..))
            .with_paragraph(vec![ComponentLine::new(feature_sentence(
                &keybinding_state,
                &main_screen_builder,
            ))])
            .with_bulletin_list(
                BulletinList::new(Text::new("新键位绑定：").color_range(2, ..))
                    .with_items(keybind_items),
            );
        if !conflicting_binds.is_empty() {
            page = page.with_paragraph(conflicting_binds);
        }
        match status {
            ApplyStatus::NotApplied => page
                .with_paragraph(vec![ComponentLine::new(vec![
                    ActiveComponent::new(TextOrCustomRender::Text(Text::new(
                        "将这些键位绑定添加到你的配置文件中？(",
                    ))),
                    ActiveComponent::new(TextOrCustomRender::Text(confirm_text()))
                        .with_hover(TextOrCustomRender::CustomRender(
                            Box::new(confirm_key_selected),
                            Box::new(single_character_len),
                        ))
                        .with_left_click_action(ClickAction::new_change_page({
                            let keybinding_state = keybinding_state.clone();
                            let main_screen_builder = main_screen_builder.clone();
                            move || {
                                apply_missing_binds(&keybinding_state);
                                Page::new_update_keybindings(keybinding_state, main_screen_builder)
                            }
                        })),
                    ActiveComponent::new(TextOrCustomRender::Text(Text::new("/"))),
                    ActiveComponent::new(TextOrCustomRender::Text(cancel_text()))
                        .with_hover(TextOrCustomRender::CustomRender(
                            Box::new(cancel_key_selected),
                            Box::new(single_character_len),
                        ))
                        .with_left_click_action(ClickAction::new_change_page({
                            let main_screen_builder = main_screen_builder.clone();
                            move || main_screen_builder()
                        })),
                    ActiveComponent::new(TextOrCustomRender::Text(Text::new(")"))),
                ])])
                .with_help(Box::new(|_hovering_over_link, _menu_item_is_selected| {
                    Text::new("帮助：<y> - 添加键位绑定，<n> - 取消，<ESC> - 返回")
                        .color_substring(1, "<y>")
                        .color_substring(1, "<n>")
                        .color_substring(1, "<ESC>")
                })),
            status => page
                .with_paragraph(vec![ComponentLine::new(vec![ActiveComponent::new(
                    TextOrCustomRender::Text(keybinding_status_text(&status)),
                )])])
                .with_help(Box::new(|_hovering_over_link, _menu_item_is_selected| {
                    esc_to_go_back_help()
                })),
        }
    }
    fn new_kitty_graphics() -> Page {
        Page::new()
            .with_title(Text::new("Kitty 图形协议").color_range(0, ..))
            .with_paragraph(vec![
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("Zellij now implements the Kitty graphics protocol.")
                        .color_substring(2, "Kitty 图形协议"),
                ))]),
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("图像查看器、绘图库和"),
                ))]),
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("文档工具显示的图像会在窗格内渲染，并在窗格滚动、移动、"),
                ))]),
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("调整大小或堆叠时保持工作。"),
                ))]),
            ])
            .with_paragraph(vec![ComponentLine::new(vec![ActiveComponent::new(
                TextOrCustomRender::Text(Text::new(
                    "宿主终端也需要支持该协议。",
                )),
            )])])
            .with_help(Box::new(|_hovering_over_link, _menu_item_is_selected| {
                esc_to_go_back_help()
            }))
    }
    fn new_mobile_web_ui(link_executable: Rc<RefCell<String>>) -> Page {
        Page::new()
            .with_title(Text::new("移动端 Web 界面").color_range(0, ..))
            .with_paragraph(vec![
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("网页客户端现在有专门的移动端界面。"),
                ))]),
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("它提供触控操作和适配小屏幕的布局，")
                        .color_substring(1, "触控操作"),
                ))]),
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("让你可以在手机或平板上使用会话。"),
                ))]),
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("可以直接在此界面中切换会话和窗格。")
                        .color_substring(1, "会话和窗格"),
                ))]),
            ])
            .with_paragraph(vec![
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("网页客户端还可以作为独立应用（PWA）安装，")
                        .color_substring(1, "standalone app (PWA)"),
                ))]),
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("直接从浏览器安装。"),
                ))]),
            ])
            .with_paragraph(vec![ComponentLine::new(vec![
                ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("了解更多： ").color_range(2, ..),
                )),
                ActiveComponent::new(TextOrCustomRender::Text(Text::new(
                    "https://zellij.dev/tutorials/web-client/",
                )))
                .with_hover(TextOrCustomRender::CustomRender(
                    Box::new(web_client_link_selected),
                    Box::new(web_client_link_selected_len),
                ))
                .with_left_click_action(ClickAction::new_open_link(
                    "https://zellij.dev/tutorials/web-client/".to_owned(),
                    link_executable.clone(),
                )),
            ])])
            .with_help(Box::new(|hovering_over_link, menu_item_is_selected| {
                esc_go_back_plus_link_hover(hovering_over_link, menu_item_is_selected)
            }))
    }
    fn new_ui() -> Page {
        Page::new()
            .with_title(Text::new("新界面").color_range(0, ..))
            .with_paragraph(vec![ComponentLine::new(vec![ActiveComponent::new(
                TextOrCustomRender::Text(Text::new("Zellij 界面已经重新设计。")),
            )])])
            .with_paragraph(vec![
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("1. 标题框：窗格边框现在默认关闭，只保留")
                        .color_substring(2, "标题框"),
                ))]),
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("   标签页中有多个窗格时的标题行。对于"),
                ))]),
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("   单个窗格，标签页的标题将是窗格的标题。"),
                ))]),
                config_option_line("pane_frame_style \"full\""),
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("2. 堆叠列表：窗格堆叠已重新设计，以紧凑列表形式显示在")
                        .color_substring(2, "堆叠列表"),
                ))]),
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("   整个堆叠上方，让完整列表"),
                ))]),
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("   在一个地方即可看到所有窗格，而不是同时显示在展开窗格的"),
                ))]),
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("   上方和下方。"),
                ))]),
                config_option_line("stacked_pane_list false"),
            ])
            .with_help(Box::new(|_hovering_over_link, _menu_item_is_selected| {
                esc_to_go_back_help()
            }))
    }
    fn new_per_client_tab_sizes() -> Page {
        Page::new()
            .with_title(Text::new("按客户端调整标签页大小").color_range(0, ..))
            .with_paragraph(vec![
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("标签页现在可以为不同客户端设置不同大小。"),
                ))]),
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("当多个客户端连接到同一会话并"),
                ))]),
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("聚焦于不同标签页时，每个标签页将按各自客户端的大小调整。"),
                ))]),
            ])
            .with_paragraph(vec![
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("以前，所有标签页共享最小客户端的大小。"),
                ))]),
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("只有当客户端聚焦于同一标签页时，标签页才会缩小。"),
                ))]),
            ])
            .with_help(Box::new(|_hovering_over_link, _menu_item_is_selected| {
                esc_to_go_back_help()
            }))
    }
}

fn keybinding_status_text(status: &ApplyStatus) -> Text {
    match status {
        ApplyStatus::Applying => Text::new("正在添加键位绑定...").color_range(2, ..),
        ApplyStatus::Applied => {
            Text::new("键位绑定已添加并保存到你的配置文件中。").color_range(2, ..)
        },
        ApplyStatus::Failed(Some(config_file)) => Text::new(format!(
            "键位绑定已添加到本会话，但无法写入 {}。",
            config_file
        ))
        .color_range(3, ..),
        ApplyStatus::Failed(None) => Text::new(
            "键位绑定已添加到本会话，但无法写入配置文件。",
        )
        .color_range(3, ..),
        ApplyStatus::NotApplied => Text::new(""),
    }
}

fn missing_binds_note(
    keybinding_state: &Rc<RefCell<KeybindingState>>,
    main_screen_builder: &Rc<dyn Fn() -> Page>,
    feature: Feature,
) -> Option<Vec<ComponentLine>> {
    let status = keybinding_state.borrow().status().clone();
    let has_missing_binds = keybinding_state.borrow().has_missing_binds_for(feature);
    let has_conflicts = keybinding_state.borrow().has_conflicts();
    match status {
        ApplyStatus::NotApplied if has_missing_binds => Some(vec![ComponentLine::new(vec![
            ActiveComponent::new(TextOrCustomRender::Text(
                Text::new("注意：这些键位绑定不在你的配置文件中。添加它们？(")
                    .color_substring(2, "注意："),
            )),
            ActiveComponent::new(TextOrCustomRender::Text(confirm_text()))
                .with_hover(TextOrCustomRender::CustomRender(
                    Box::new(confirm_key_selected),
                    Box::new(single_character_len),
                ))
                .with_left_click_action(ClickAction::new_change_page({
                    let keybinding_state = keybinding_state.clone();
                    let main_screen_builder = main_screen_builder.clone();
                    move || {
                        if has_conflicts {
                            Page::new_update_keybindings(keybinding_state, main_screen_builder)
                        } else {
                            apply_missing_binds(&keybinding_state);
                            feature_page(feature, keybinding_state, main_screen_builder)
                        }
                    }
                })),
            ActiveComponent::new(TextOrCustomRender::Text(Text::new("/"))),
            ActiveComponent::new(TextOrCustomRender::Text(cancel_text()))
                .with_hover(TextOrCustomRender::CustomRender(
                    Box::new(cancel_key_selected),
                    Box::new(single_character_len),
                ))
                .with_left_click_action(ClickAction::new_change_page({
                    let main_screen_builder = main_screen_builder.clone();
                    move || main_screen_builder()
                })),
            ActiveComponent::new(TextOrCustomRender::Text(Text::new(")"))),
        ])]),
        ApplyStatus::NotApplied => None,
        status => Some(vec![ComponentLine::new(vec![ActiveComponent::new(
            TextOrCustomRender::Text(keybinding_status_text(&status)),
        )])]),
    }
}

fn feature_page(
    feature: Feature,
    keybinding_state: Rc<RefCell<KeybindingState>>,
    main_screen_builder: Rc<dyn Fn() -> Page>,
) -> Page {
    match feature {
        Feature::NestedSessions => Page::new_nested_sessions(keybinding_state, main_screen_builder),
        Feature::PaneFocus => Page::new_pane_focus(keybinding_state, main_screen_builder),
        Feature::ScrollByCommand => {
            Page::new_scroll_by_command(keybinding_state, main_screen_builder)
        },
    }
}

fn option_title_line(title: &str) -> ComponentLine {
    let title_without_number = title.trim_start_matches(|character: char| {
        character.is_ascii_digit() || character == '.' || character == ' '
    });
    ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
        Text::new(title).color_substring(2, title_without_number),
    ))])
}

fn opt_out_line(what: &str, config_option: &str) -> ComponentLine {
    ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
        Text::new(format!("  ↳ 退出 {} 的方式：{}", what, config_option))
            .color_substring(1, "退出")
            .color_substring(3, config_option),
    ))])
}

fn config_option_line(config_option: &str) -> ComponentLine {
    ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
        Text::new(format!("   退出方式：{}", config_option))
            .color_substring(1, "退出方式")
            .color_substring(3, config_option),
    ))])
}

fn option_text_line(text: &str) -> ComponentLine {
    ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
        Text::new(text),
    ))])
}

fn bind_lines(
    keybinding_state: &Rc<RefCell<KeybindingState>>,
    actions: &[Action],
    indent: &str,
) -> Vec<ComponentLine> {
    actions
        .iter()
        .filter_map(|action| {
            keybinding_state
                .borrow()
                .bind_for_action(action)
                .map(|bind| {
                    let mode_name = format!("{:?}", bind.mode).to_uppercase();
                    ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                        Text::new(format!(
                            "{}{} mode + {} - {}",
                            indent,
                            mode_name,
                            bind.key_text(),
                            bind.description
                        ))
                        .color_substring(3, &mode_name)
                        .color_substring(3, &bind.key_text()),
                    ))])
                })
        })
        .collect()
}

fn merged_bind_line(
    keybinding_state: &Rc<RefCell<KeybindingState>>,
    actions: &[Action],
    description: &str,
) -> Vec<ComponentLine> {
    let keybinding_state = keybinding_state.borrow();
    let binds: Vec<&ExpectedBind> = actions
        .iter()
        .filter_map(|action| keybinding_state.bind_for_action(action))
        .collect();
    let Some(first_bind) = binds.first() else {
        return vec![];
    };
    let mode_name = first_bind.mode_name();
    let key_texts: Vec<String> = binds.iter().map(|bind| bind.key_text()).collect();
    let mut text = Text::new(format!(
        "{} mode + {} - {}",
        mode_name,
        key_texts.join(" / "),
        description
    ))
    .color_substring(3, &mode_name);
    for key_text in &key_texts {
        text = text.color_substring(3, key_text);
    }
    vec![ComponentLine::new(vec![ActiveComponent::new(
        TextOrCustomRender::Text(text),
    )])]
}

fn confirm_text() -> Text {
    Text::new("y").color_range(3, ..)
}

fn cancel_text() -> Text {
    Text::new("n").color_range(3, ..)
}

fn confirm_key_selected(x: usize, y: usize) -> usize {
    print!("\u{1b}[{};{}H\u{1b}[m\u{1b}[1;4my", y + 1, x + 1);
    1
}

fn cancel_key_selected(x: usize, y: usize) -> usize {
    print!("\u{1b}[{};{}H\u{1b}[m\u{1b}[1;4mn", y + 1, x + 1);
    1
}

fn single_character_len() -> usize {
    1
}

fn feature_link(label: &'static str, target_page: ClickAction) -> ActiveComponent {
    let hover_label = label.to_owned();
    ActiveComponent::new(TextOrCustomRender::Text(
        Text::new(label).color_range(3, ..),
    ))
    .with_hover(TextOrCustomRender::CustomRender(
        Box::new(move |x, y| {
            print!(
                "\u{1b}[{};{}H\u{1b}[m\u{1b}[1;4m{}",
                y + 1,
                x + 1,
                hover_label
            );
            hover_label.chars().count()
        }),
        Box::new(move || label.chars().count()),
    ))
    .with_left_click_action(target_page)
    .keyboard_selectable()
}

fn feature_sentence(
    keybinding_state: &Rc<RefCell<KeybindingState>>,
    main_screen_builder: &Rc<dyn Fn() -> Page>,
) -> Vec<ActiveComponent> {
    let features = keybinding_state.borrow().features_to_add();
    let mut sentence = vec![ActiveComponent::new(TextOrCustomRender::Text(Text::new(
        "此版本为以下功能添加了新的键位绑定：",
    )))];
    for (feature_index, feature) in features.iter().enumerate() {
        if feature_index > 0 {
            let separator = if feature_index == features.len().saturating_sub(1) {
                " 和 "
            } else {
                ", "
            };
            sentence.push(ActiveComponent::new(TextOrCustomRender::Text(Text::new(
                separator,
            ))));
        }
        let keybinding_state = keybinding_state.clone();
        let main_screen_builder = main_screen_builder.clone();
        let feature = *feature;
        sentence.push(feature_link(
            feature.label(),
            ClickAction::new_change_page(move || {
                feature_page(feature, keybinding_state, main_screen_builder)
            }),
        ));
    }
    sentence.push(ActiveComponent::new(TextOrCustomRender::Text(Text::new(
        ".",
    ))));
    sentence
}

impl Page {
    pub fn new() -> Self {
        Page {
            title: None,
            components_to_render: vec![],
            has_hover: false,
            hovering_over_link: false,
            menu_item_is_selected: false,
            is_main_screen: false,
            kind: PageKind::Other,
        }
    }
    pub fn main_screen(mut self) -> Self {
        self.is_main_screen = true;
        self
    }
    pub fn with_kind(mut self, kind: PageKind) -> Self {
        self.kind = kind;
        self
    }
    pub fn with_title(mut self, title: Text) -> Self {
        self.title = Some(title);
        self
    }
    pub fn with_bulletin_list(mut self, bulletin_list: BulletinList) -> Self {
        self.components_to_render
            .push(RenderedComponent::BulletinList(bulletin_list));
        self
    }
    pub fn with_paragraph(mut self, paragraph: Vec<ComponentLine>) -> Self {
        self.components_to_render
            .push(RenderedComponent::Paragraph(paragraph));
        self
    }
    pub fn with_help(mut self, help_text_fn: Box<dyn Fn(bool, bool) -> Text>) -> Self {
        self.components_to_render
            .push(RenderedComponent::HelpText(help_text_fn));
        self
    }
    pub fn handle_key(&mut self, key: KeyWithModifier) -> bool {
        let mut should_render = false;
        if key.bare_key == BareKey::Down && key.has_no_modifiers() {
            self.move_selection_down();
            should_render = true;
        } else if key.bare_key == BareKey::Up && key.has_no_modifiers() {
            self.move_selection_up();
            should_render = true;
        }
        should_render
    }
    pub fn handle_mouse_left_click(&mut self, x: usize, y: usize) -> Option<Page> {
        for rendered_component in &mut self.components_to_render {
            match rendered_component {
                RenderedComponent::BulletinList(bulletin_list) => {
                    let page_to_render = bulletin_list.handle_left_click_at_position(x, y);
                    if page_to_render.is_some() {
                        return page_to_render;
                    }
                },
                RenderedComponent::Paragraph(paragraph) => {
                    for component_line in paragraph {
                        let page_to_render = component_line.handle_left_click_at_position(x, y);
                        if page_to_render.is_some() {
                            return page_to_render;
                        }
                    }
                },
                _ => {},
            }
        }
        None
    }
    pub fn handle_selection(&mut self) -> Option<Page> {
        for component in self.selectable_components() {
            if component.is_active {
                let page_to_render = component.handle_selection();
                if page_to_render.is_some() {
                    return page_to_render;
                }
            }
        }
        None
    }
    pub fn handle_mouse_hover(&mut self, x: usize, y: usize) -> bool {
        let hover_cleared = self.clear_hover(); // TODO: 如果同一个组件在上一次移动中已被悬停，需做正确处理
                                                // （上一次移动）
        for rendered_component in &mut self.components_to_render {
            match rendered_component {
                RenderedComponent::BulletinList(bulletin_list) => {
                    let should_render = bulletin_list.handle_hover_at_position(x, y);
                    if should_render {
                        self.has_hover = true;
                        self.menu_item_is_selected = true;
                        return should_render;
                    }
                },
                RenderedComponent::Paragraph(paragraph) => {
                    for component_line in paragraph {
                        if let Some(hovered_component_is_selectable) =
                            component_line.handle_hover_at_position(x, y)
                        {
                            self.has_hover = true;
                            if hovered_component_is_selectable {
                                self.menu_item_is_selected = true;
                            } else {
                                self.hovering_over_link = true;
                            }
                            return true;
                        }
                    }
                },
                _ => {},
            }
        }
        hover_cleared
    }
    fn move_selection_up(&mut self) {
        match self.position_of_active_selectable() {
            Some(position_of_active_selectable) if position_of_active_selectable > 0 => {
                self.clear_active_selectables();
                self.set_active_selectable(position_of_active_selectable.saturating_sub(1));
            },
            Some(0) => {
                self.clear_active_selectables();
            },
            _ => {
                self.clear_active_selectables();
                self.set_last_active_selectable();
            },
        }
    }
    fn move_selection_down(&mut self) {
        match self.position_of_active_selectable() {
            Some(position_of_active_selectable) => {
                self.clear_active_selectables();
                self.set_active_selectable(position_of_active_selectable + 1);
            },
            None => {
                self.set_active_selectable(0);
            },
        }
    }
    fn selectable_components(&mut self) -> Vec<&mut ActiveComponent> {
        let mut selectable_components = vec![];
        for rendered_component in &mut self.components_to_render {
            match rendered_component {
                RenderedComponent::BulletinList(bulletin_list) => {
                    selectable_components.append(&mut bulletin_list.items_mut());
                },
                RenderedComponent::Paragraph(paragraph) => {
                    for component_line in paragraph {
                        selectable_components
                            .append(&mut component_line.keyboard_selectable_components_mut());
                    }
                },
                _ => {},
            }
        }
        selectable_components
    }
    fn position_of_active_selectable(&mut self) -> Option<usize> {
        self.selectable_components()
            .iter()
            .position(|component| component.is_active)
    }
    fn clear_active_selectables(&mut self) {
        for component in self.selectable_components() {
            component.is_active = false;
        }
    }
    fn set_active_selectable(&mut self, position: usize) {
        if let Some(component) = self.selectable_components().get_mut(position) {
            component.is_active = true;
        }
    }
    fn set_last_active_selectable(&mut self) {
        if let Some(component) = self.selectable_components().last_mut() {
            component.is_active = true;
        }
    }
    fn clear_hover(&mut self) -> bool {
        let had_hover = self.has_hover;
        self.menu_item_is_selected = false;
        self.hovering_over_link = false;
        for rendered_component in &mut self.components_to_render {
            match rendered_component {
                RenderedComponent::BulletinList(bulletin_list) => {
                    bulletin_list.clear_hover();
                },
                RenderedComponent::Paragraph(paragraph) => {
                    for active_component in paragraph {
                        active_component.clear_hover();
                    }
                },
                _ => {},
            }
        }
        self.has_hover = false;
        had_hover
    }
    pub fn ui_column_count(&mut self) -> usize {
        let mut column_count = 0;
        for rendered_component in &self.components_to_render {
            match rendered_component {
                RenderedComponent::BulletinList(bulletin_list) => {
                    column_count = std::cmp::max(column_count, bulletin_list.column_count());
                },
                RenderedComponent::Paragraph(paragraph) => {
                    for active_component in paragraph {
                        column_count = std::cmp::max(column_count, active_component.column_count());
                    }
                },
                RenderedComponent::HelpText(_text) => {}, // 我们在列数计算中忽略帮助文本
                                                          // 因为它始终左对齐
            }
        }
        column_count
    }
    pub fn ui_row_count(&mut self) -> usize {
        let mut row_count = 0;
        if self.title.is_some() {
            row_count += 1;
        }
        for rendered_component in &self.components_to_render {
            match rendered_component {
                RenderedComponent::BulletinList(bulletin_list) => {
                    row_count += bulletin_list.len();
                },
                RenderedComponent::Paragraph(paragraph) => {
                    row_count += paragraph.len();
                },
                RenderedComponent::HelpText(_text) => {}, // 我们忽略帮助文本，因为它在
                                                          // 界面容器之外
            }
        }
        row_count += self.components_to_render.len();
        row_count
    }
    pub fn render(&mut self, rows: usize, columns: usize, error: &Option<String>) {
        let base_x = columns.saturating_sub(self.ui_column_count()) / 2;
        let base_y = rows.saturating_sub(self.ui_row_count()) / 2;
        let mut current_y = base_y;
        if let Some(title) = &self.title {
            print_text_with_coordinates(
                title.clone(),
                base_x,
                current_y,
                Some(columns),
                Some(rows),
            );
            current_y += 2;
        }
        for rendered_component in &mut self.components_to_render {
            let is_help = match rendered_component {
                RenderedComponent::HelpText(_) => true,
                _ => false,
            };
            if is_help {
                if let Some(error) = error {
                    render_error(error, rows);
                    continue;
                }
            }
            let y = if is_help { rows } else { current_y };
            let columns = if is_help {
                columns
            } else {
                columns.saturating_sub(base_x * 2)
            };
            let rendered_rows = rendered_component.render(
                base_x,
                y,
                rows,
                columns,
                self.hovering_over_link,
                self.menu_item_is_selected,
            );
            current_y += rendered_rows + 1; // 1 表示组件之间的行间距
        }
    }
}

fn render_error(error: &str, y: usize) {
    print_text_with_coordinates(
        Text::new(format!("错误：{}", error)).color_range(3, ..),
        0,
        y,
        None,
        None,
    );
}

fn changelog_link_unselected(version: String) -> Text {
    let full_changelog_text = format!(
        "https://github.com/zellij-org/zellij/releases/tag/v{}",
        version
    );
    Text::new(full_changelog_text)
}

fn changelog_link_selected(version: String) -> Box<dyn Fn(usize, usize) -> usize> {
    Box::new(move |x, y| {
        print!(
            "\u{1b}[{};{}H\u{1b}[m\u{1b}[1;4mhttps://github.com/zellij-org/zellij/releases/tag/v{}",
            y + 1,
            x + 1,
            version
        );
        51 + version.chars().count()
    })
}

fn changelog_link_selected_len(version: String) -> Box<dyn Fn() -> usize> {
    Box::new(move || 51 + version.chars().count())
}

fn sponsors_link_text_unselected() -> Text {
    Text::new("https://github.com/sponsors/imsnif")
}

fn sponsors_link_text_selected(x: usize, y: usize) -> usize {
    print!(
        "\u{1b}[{};{}H\u{1b}[m\u{1b}[1;4mhttps://github.com/sponsors/imsnif",
        y + 1,
        x + 1
    );
    34
}

fn sponsors_link_text_selected_len() -> usize {
    34
}

fn web_client_link_selected(x: usize, y: usize) -> usize {
    print!(
        "\u{1b}[{};{}H\u{1b}[m\u{1b}[1;4mhttps://zellij.dev/tutorials/web-client/",
        y + 1,
        x + 1
    );
    40
}

fn web_client_link_selected_len() -> usize {
    40
}

// 文本组件
fn whats_new_title() -> Text {
    Text::new("有什么新内容？")
}

fn main_screen_title(version: String, is_release_notes: bool) -> Text {
    if is_release_notes {
        let title_text = format!("你好，欢迎使用 Zellij {}！", &version);
        Text::new(title_text).color_range(2, 7..=12 + version.chars().count())
    } else {
        let title_text = format!("Zellij {}", &version);
        Text::new(title_text).color_range(2, ..)
    }
}

fn main_screen_help_text(
    hovering_over_link: bool,
    menu_item_is_selected: bool,
    has_missing_binds: bool,
) -> Text {
    if hovering_over_link {
        return link_hover_help();
    }
    let mut help_text = String::from("帮助：<↓↑> - 导航");
    if menu_item_is_selected {
        help_text.push_str("，<ENTER> - 了解更多");
    }
    if has_missing_binds {
        help_text.push_str("，<u> - 更新键位绑定");
    }
    help_text.push_str("，<ESC> - 关闭");
    if !menu_item_is_selected {
        help_text.push_str("，<?> - 使用提示");
    }
    color_help_keys(help_text)
}

fn release_notes_main_help(
    hovering_over_link: bool,
    menu_item_is_selected: bool,
    has_missing_binds: bool,
) -> Text {
    if hovering_over_link {
        return link_hover_help();
    }
    let mut help_text = String::from("帮助：<↓↑> - 导航");
    if menu_item_is_selected {
        help_text.push_str("，<ENTER> - 了解更多");
    }
    if has_missing_binds {
        help_text.push_str("，<u> - 更新键位绑定");
    }
    help_text.push_str("，<ESC> - 关闭");
    color_help_keys(help_text)
}

fn color_help_keys(help_text: String) -> Text {
    Text::new(help_text)
        .color_substring(1, "<↓↑>")
        .color_substring(1, "<ENTER>")
        .color_substring(2, "更新键位绑定")
        .color_substring(1, "<u>")
        .color_substring(1, "<ESC>")
        .color_substring(1, "<?>")
}

fn link_hover_help() -> Text {
    Text::new("帮助：点击或 Shift-Click 在浏览器中打开")
        .color_range(3, 3..=4)
        .color_range(3, 7..=17)
}

fn esc_go_back_plus_link_hover(hovering_over_link: bool, _menu_item_is_selected: bool) -> Text {
    if hovering_over_link {
        let help_text = format!("帮助：点击或 Shift-Click 在浏览器中打开");
        Text::new(help_text)
            .color_range(3, 3..=4)
            .color_range(3, 7..=17)
    } else {
        let help_text = format!("帮助：<ESC> - 返回");
        Text::new(help_text).color_range(1, 3..=7)
    }
}

fn esc_to_go_back_help() -> Text {
    let help_text = format!("Help: <ESC> - Go back");
    Text::new(help_text).color_range(1, 6..=10)
}

fn main_menu_item(item_name: &str) -> Text {
    Text::new(item_name).color_range(0, ..)
}

fn support_the_developer_text() -> Text {
    let support_text = format!("请支持 Zellij 开发者 <3： ");
    Text::new(support_text).color_range(3, ..)
}

pub enum TextOrCustomRender {
    Text(Text),
    CustomRender(
        Box<dyn Fn(usize, usize) -> usize>, // (行数, 列数) -> 文本长度（渲染函数）
        Box<dyn Fn() -> usize>,             // 已渲染组件的长度
    ),
}

impl TextOrCustomRender {
    pub fn len(&self) -> usize {
        match self {
            TextOrCustomRender::Text(text) => text.len(),
            TextOrCustomRender::CustomRender(_render_fn, len_fn) => len_fn(),
        }
    }
    pub fn render(&mut self, x: usize, y: usize, rows: usize, columns: usize) -> usize {
        match self {
            TextOrCustomRender::Text(text) => {
                print_text_with_coordinates(text.clone(), x, y, Some(columns), Some(rows));
                text.len()
            },
            TextOrCustomRender::CustomRender(render_fn, _len_fn) => render_fn(x, y),
        }
    }
}

impl std::fmt::Debug for TextOrCustomRender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextOrCustomRender::Text(text) => write!(f, "Text {{ {:?} }}", text),
            TextOrCustomRender::CustomRender(..) => write!(f, "CustomRender"),
        }
    }
}

enum RenderedComponent {
    HelpText(Box<dyn Fn(bool, bool) -> Text>),
    BulletinList(BulletinList),
    Paragraph(Vec<ComponentLine>),
}

impl std::fmt::Debug for RenderedComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderedComponent::HelpText(_) => write!(f, "HelpText"),
            RenderedComponent::BulletinList(bulletinlist) => write!(f, "{:?}", bulletinlist),
            RenderedComponent::Paragraph(component_list) => write!(f, "{:?}", component_list),
        }
    }
}

impl RenderedComponent {
    pub fn render(
        &mut self,
        x: usize,
        y: usize,
        rows: usize,
        columns: usize,
        hovering_over_link: bool,
        menu_item_is_selected: bool,
    ) -> usize {
        let mut rendered_rows = 0;
        match self {
            RenderedComponent::HelpText(text) => {
                rendered_rows += 1;
                print_text_with_coordinates(
                    text(hovering_over_link, menu_item_is_selected),
                    0,
                    y,
                    Some(columns),
                    Some(rows),
                );
            },
            RenderedComponent::BulletinList(bulletin_list) => {
                rendered_rows += bulletin_list.len();
                bulletin_list.render(x, y, rows, columns);
            },
            RenderedComponent::Paragraph(paragraph) => {
                let mut paragraph_rendered_rows = 0;
                for component_line in paragraph {
                    component_line.render(
                        x,
                        y + paragraph_rendered_rows,
                        rows.saturating_sub(paragraph_rendered_rows),
                        columns,
                    );
                    rendered_rows += 1;
                    paragraph_rendered_rows += 1;
                }
            },
        }
        rendered_rows
    }
}

#[derive(Debug)]
pub struct BulletinList {
    title: Text,
    items: Vec<ActiveComponent>,
}

impl BulletinList {
    pub fn new(title: Text) -> Self {
        BulletinList {
            title,
            items: vec![],
        }
    }
    pub fn with_items(mut self, items: Vec<ActiveComponent>) -> Self {
        self.items = items;
        self
    }
    pub fn len(&self) -> usize {
        self.items.len() + 1 // 1 表示标题
    }
    pub fn column_count(&self) -> usize {
        let mut column_count = 0;
        for (item_index, item) in self.items.iter().enumerate() {
            let item_bulletin_len = format!("{}. ", item_index + 1).chars().count();
            column_count = std::cmp::max(column_count, item.column_count() + item_bulletin_len);
        }
        column_count
    }
    pub fn handle_left_click_at_position(&mut self, x: usize, y: usize) -> Option<Page> {
        for component in &mut self.items {
            let page_to_render = component.handle_left_click_at_position(x, y);
            if page_to_render.is_some() {
                return page_to_render;
            }
        }
        None
    }
    pub fn items_mut(&mut self) -> Vec<&mut ActiveComponent> {
        self.items.iter_mut().collect()
    }
    pub fn handle_hover_at_position(&mut self, x: usize, y: usize) -> bool {
        for component in &mut self.items {
            let should_render = component.handle_hover_at_position(x, y);
            if should_render {
                return should_render;
            }
        }
        false
    }
    pub fn clear_hover(&mut self) {
        for component in &mut self.items {
            component.clear_hover();
        }
    }
    pub fn render(&mut self, x: usize, y: usize, rows: usize, columns: usize) {
        print_text_with_coordinates(self.title.clone(), x, y, Some(columns), Some(rows));
        let mut item_bulletin = 1;
        let mut running_y = y + 1;
        for item in &mut self.items {
            let mut item_bulletin_text = Text::new(format!("{}. ", item_bulletin));
            if item.is_active {
                item_bulletin_text = item_bulletin_text.selected();
            }
            let item_bulletin_text_len = item_bulletin_text.len();
            print_text_with_coordinates(
                item_bulletin_text,
                x,
                running_y,
                Some(item_bulletin_text_len),
                Some(rows),
            );
            let extend_hit_area_to_end_of_line = true;
            item.render(
                x + item_bulletin_text_len,
                running_y,
                rows,
                columns.saturating_sub(item_bulletin_text_len),
                extend_hit_area_to_end_of_line,
            );
            running_y += 1;
            item_bulletin += 1;
        }
    }
}

#[derive(Debug)]
pub struct ComponentLine {
    components: Vec<ActiveComponent>,
}

impl ComponentLine {
    pub fn handle_left_click_at_position(&mut self, x: usize, y: usize) -> Option<Page> {
        for active_component in &mut self.components {
            let page_to_render = active_component.handle_left_click_at_position(x, y);
            if page_to_render.is_some() {
                return page_to_render;
            }
        }
        None
    }
    pub fn handle_hover_at_position(&mut self, x: usize, y: usize) -> Option<bool> {
        for active_component in &mut self.components {
            if active_component.handle_hover_at_position(x, y) {
                return Some(active_component.is_keyboard_selectable());
            }
        }
        None
    }
    pub fn keyboard_selectable_components_mut(&mut self) -> Vec<&mut ActiveComponent> {
        self.components
            .iter_mut()
            .filter(|component| component.is_keyboard_selectable())
            .collect()
    }
    pub fn clear_hover(&mut self) {
        for active_component in &mut self.components {
            active_component.clear_hover();
        }
    }
    pub fn column_count(&self) -> usize {
        let mut column_count = 0;
        for active_component in &self.components {
            column_count += active_component.column_count()
        }
        column_count
    }
    pub fn render(&mut self, x: usize, y: usize, rows: usize, columns: usize) {
        let mut current_x = x;
        let mut columns_left = columns;
        let last_component_index = self.components.len().saturating_sub(1);
        for (component_index, component) in self.components.iter_mut().enumerate() {
            let extend_hit_area_to_end_of_line = component_index == last_component_index;
            let component_len = component.render(
                current_x,
                y,
                rows,
                columns_left,
                extend_hit_area_to_end_of_line,
            );
            current_x += component_len;
            columns_left = columns_left.saturating_sub(component_len);
        }
    }
}

impl ComponentLine {
    pub fn new(components: Vec<ActiveComponent>) -> Self {
        ComponentLine { components }
    }
}
