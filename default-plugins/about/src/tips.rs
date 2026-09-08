use zellij_tile::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

use crate::active_component::{ActiveComponent, ClickAction};
use crate::pages::{BulletinList, ComponentLine, Page, TextOrCustomRender};

pub const MAX_TIP_INDEX: usize = 11;

impl Page {
    pub fn new_tip_screen(
        link_executable: Rc<RefCell<String>>,
        base_mode: Rc<RefCell<InputMode>>,
        tip_index: usize,
    ) -> Self {
        if tip_index == 0 {
            Page::tip_1(link_executable)
        } else if tip_index == 1 {
            Page::tip_2(link_executable, base_mode)
        } else if tip_index == 2 {
            Page::tip_3(link_executable)
        } else if tip_index == 3 {
            Page::tip_4(link_executable, base_mode)
        } else if tip_index == 4 {
            Page::tip_5(link_executable)
        } else if tip_index == 5 {
            Page::tip_6(link_executable, base_mode)
        } else if tip_index == 6 {
            Page::tip_7(link_executable)
        } else if tip_index == 7 {
            Page::tip_8(link_executable)
        } else if tip_index == 8 {
            Page::tip_9(link_executable)
        } else if tip_index == 9 {
            Page::tip_10(link_executable, base_mode)
        } else if tip_index == 10 {
            Page::tip_11(link_executable)
        } else if tip_index == 11 {
            Page::tip_12(link_executable, base_mode)
        } else {
            Page::tip_1(link_executable)
        }
    }
    pub fn tip_1(link_executable: Rc<RefCell<String>>) -> Self {
        Page::new()
            .main_screen()
            .with_title(Text::new("Zellij 提示 #1").color_range(0, ..))
            .with_paragraph(vec![
                ComponentLine::new(vec![
                    ActiveComponent::new(TextOrCustomRender::Text(
                            Text::new("观看 Zellij 的屏幕录像/教程，学习如何更好地利用")
                    ))
                ]),
                ComponentLine::new(vec![
                    ActiveComponent::new(TextOrCustomRender::Text(
                            Text::new("Zellij 的所有功能。了解基本用法、布局、会话等更多内容！")
                    ))
                ])
            ])
            .with_paragraph(vec![ComponentLine::new(vec![
                ActiveComponent::new(TextOrCustomRender::Text(Text::new("点击此链接： ").color_range(2, ..))),
                ActiveComponent::new(TextOrCustomRender::Text(Text::new("https://zellij.dev/screencasts")))
                .with_hover(TextOrCustomRender::CustomRender(
                    Box::new(screencasts_link_selected()),
                    Box::new(screencasts_link_selected_len()),
                ))
                .with_left_click_action(ClickAction::new_open_link(
                    format!("https://zellij.dev/screencasts"),
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
            .with_help(Box::new(|hovering_over_link, _menu_item_is_selected| {
                tips_help_text(hovering_over_link)
            }))
    }
    pub fn tip_2(link_executable: Rc<RefCell<String>>, base_mode: Rc<RefCell<InputMode>>) -> Self {
        Page::new()
            .main_screen()
            .with_title(Text::new("Zellij 提示 #2").color_range(0, ..))
            .with_paragraph(vec![
                ComponentLine::new(vec![
                    ActiveComponent::new(TextOrCustomRender::Text(
                            Text::new("你可以在你的 $EDITOR 中打开终端内容，以便搜索")
                                .color_range(2, 7..=13)
                    ))
                ]),
                ComponentLine::new(vec![
                    ActiveComponent::new(TextOrCustomRender::Text(
                            Text::new("在其中搜索、复制到剪贴板，甚至保存起来以备后用。")
                    ))
                ])
            ])
            .with_paragraph(vec![ComponentLine::new(vec![
                match *base_mode.borrow() {
                    InputMode::Locked => {
                        ActiveComponent::new(TextOrCustomRender::Text(
                            Text::new("在终端窗格聚焦时：Ctrl g + s + e")
                                .color_range(0, 3..=8)
                                .color_indices(0, vec![22])
                        ))
                    },
                    _ => {
                        ActiveComponent::new(TextOrCustomRender::Text(
                            Text::new("在终端窗格聚焦时：Ctrl s + e")
                                .color_range(0, 3..=8)
                                .color_indices(0, vec![18])
                        ))
                    }
                }
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
            .with_help(Box::new(|hovering_over_link, _menu_item_is_selected| {
                tips_help_text(hovering_over_link)
            }))
    }
    pub fn tip_3(link_executable: Rc<RefCell<String>>) -> Self {
        Page::new()
            .main_screen()
            .with_title(Text::new("Zellij 提示 #3").color_range(0, ..))
            .with_paragraph(vec![
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("想放大你的浮动窗格吗？"),
                ))]),
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new(
                        "You can switch to the ENLARGED layout with Alt ] while focused on it.",
                    )
                    .color_range(2, 19..=26)
                    .color_range(0, 9..=13),
                ))]),
            ])
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
            .with_help(Box::new(|hovering_over_link, _menu_item_is_selected| {
                tips_help_text(hovering_over_link)
            }))
    }
    fn tip_4(link_executable: Rc<RefCell<String>>, base_mode: Rc<RefCell<InputMode>>) -> Page {
        Page::new()
            .main_screen()
            .with_title(Text::new("Zellij 提示 #4").color_range(0, ..))
            .with_paragraph(vec![
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("可以“固定”一个浮动窗格，使它始终"),
                ))]),
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("即使浮动窗格被隐藏也保持可见。"),
                ))]),
            ])
            .with_bulletin_list(
                BulletinList::new(
                    Text::new(format!("浮动窗格可以被“固定”： ")).color_range(2, ..),
                )
                .with_items(vec![
                    ActiveComponent::new(TextOrCustomRender::Text(
                        Text::new(format!("用鼠标点击其右上角"))
                            
                    )),
                    ActiveComponent::new(TextOrCustomRender::Text(match *base_mode.borrow() {
                        InputMode::Locked => Text::new(format!("使用 Ctrl g + p + i"))
                            .color_range(3, 3..=8)
                            .color_range(3, 12..=12)
                            .color_range(3, 16..=16),
                        _ => Text::new("使用 Ctrl p + i")
                            .color_range(3, 3..=8)
                            .color_range(3, 12..=12),
                    })),
                ]),
            )
            .with_paragraph(vec![
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new("这些功能的一个绝佳用途是跟踪日志文件，或在"),
                ))]),
                ComponentLine::new(vec![ActiveComponent::new(TextOrCustomRender::Text(
                    Text::new(format!(
                        "其他窗格中工作时显示实时的编译器输出。"
                    )),
                ))]),
            ])
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
            .with_help(Box::new(|hovering_over_link, _menu_item_is_selected| {
                tips_help_text(hovering_over_link)
            }))
    }
    pub fn tip_5(link_executable: Rc<RefCell<String>>) -> Page {
        Page::new()
            .main_screen()
            .with_title(Text::new("Zellij 提示 #5").color_range(0, ..))
            .with_paragraph(vec![
                ComponentLine::new(vec![
                    ActiveComponent::new(TextOrCustomRender::Text(Text::new("窗格可以调整大小并堆叠成栈，以便更容易管理。"))),
                ]),
            ])
            .with_bulletin_list(BulletinList::new(Text::new("试试看：").color_range(2, ..))
                .with_items(vec![
                    ActiveComponent::new(TextOrCustomRender::Text(
                            Text::new("用 Alt f 隐藏此窗格（再次按 Alt f 可将其恢复）")
                                .color_range(3, 2..=6)
                                .color_range(3, 18..=22)
                    )),
                    ActiveComponent::new(TextOrCustomRender::Text(
                            Text::new("用 Alt n 打开 4-5 个窗格")
                                .color_range(3, 2..=6)
                    )),
                    ActiveComponent::new(TextOrCustomRender::Text(
                            Text::new("按 Alt + 直到达到全屏")
                                
                    )),
                    ActiveComponent::new(TextOrCustomRender::Text(
                            Text::new("按 Alt - 直到恢复到原始状态")
                                
                    )),
                    ActiveComponent::new(TextOrCustomRender::Text(
                            Text::new("你可以随时用 Alt <[]> 快速切回内置的交换布局")
                                .color_range(3, 7..=9)
                                .color_range(3, 11..=14)
                    )),
                ])
            )
            .with_paragraph(vec![
                ComponentLine::new(vec![
                    ActiveComponent::new(TextOrCustomRender::Text(
                            Text::new("要禁用此行为，请在 Zellij 配置中添加 stacked_resize false")
                                .color_range(3, 23..=42)
                    )),
                ])
            ])
            .with_paragraph(vec![
                ComponentLine::new(vec![
                    ActiveComponent::new(TextOrCustomRender::Text(
                        Text::new("了解更多详情，请参见： ")
                            .color_range(2, ..)
                    )),
                    ActiveComponent::new(TextOrCustomRender::Text(Text::new("https://zellij.dev/tutorials/stacked-resize")))
                        .with_hover(TextOrCustomRender::CustomRender(Box::new(stacked_resize_screencast_link_selected), Box::new(stacked_resize_screencast_link_selected_len)))
                        .with_left_click_action(ClickAction::new_open_link("https://zellij.dev/tutorials/stacked-resize".to_owned(), link_executable.clone()))
                ])
            ])
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
            .with_help(Box::new(|hovering_over_link, _menu_item_is_selected| {
                tips_help_text(hovering_over_link)
            }))
    }
    pub fn tip_6(link_executable: Rc<RefCell<String>>, base_mode: Rc<RefCell<InputMode>>) -> Page {
        Page::new()
            .main_screen()
            .with_title(Text::new("Zellij 提示 #6").color_range(0, ..))
            .with_paragraph(vec![
                ComponentLine::new(vec![
                    ActiveComponent::new(TextOrCustomRender::Text(Text::new("Zellij 的键位绑定是否与其他应用程序冲突？")))
                ]),
            ])
            .with_bulletin_list(BulletinList::new(Text::new("看看无冲突键位绑定预设："))
                .with_items(vec![
                    ActiveComponent::new(TextOrCustomRender::Text(
                            match *base_mode.borrow() {
                                InputMode::Locked => {
                                    Text::new("使用 Ctrl g + o + c 打开 Zellij 配置")
                                        .color_range(3, 3..=8)
                                        .color_indices(3, vec![12, 16])
                                },
                                _ => {
                                    Text::new("使用 Ctrl o + c 打开 Zellij 配置")
                                        .color_range(3, 3..=8)
                                        .color_indices(3, vec![12])
                                }
                            }
                    )),
                    ActiveComponent::new(TextOrCustomRender::Text(
                            Text::new("按 TAB 进入“更改模式行为”")
                                .color_range(3, 2..=4)
                    )),
                    ActiveComponent::new(TextOrCustomRender::Text(
                            Text::new("按 ENTER 临时选择无冲突，或按 Ctrl a 永久选择")
                                .color_range(3, 2..=6)
                                .color_range(3, 19..=24)
                    )),
                ])
            )
            .with_paragraph(vec![
                ComponentLine::new(vec![
                    ActiveComponent::new(TextOrCustomRender::Text(
                        Text::new("了解更多详情，请参见： ")
                            .color_range(2, ..)
                    )),
                    ActiveComponent::new(TextOrCustomRender::Text(Text::new("https://zellij.dev/tutorials/colliding-keybindings")))
                        .with_hover(TextOrCustomRender::CustomRender(Box::new(colliding_keybindings_link_selected), Box::new(colliding_keybindings_link_selected_len)))
                        .with_left_click_action(ClickAction::new_open_link("https://zellij.dev/tutorials/colliding-keybindings".to_owned(), link_executable.clone()))
                ])
            ])
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
            .with_help(Box::new(|hovering_over_link, _menu_item_is_selected| {
                tips_help_text(hovering_over_link)
            }))
    }
    pub fn tip_7(link_executable: Rc<RefCell<String>>) -> Page {
        Page::new()
            .main_screen()
            .with_title(Text::new("Zellij 提示 #7").color_range(0, ..))
            .with_paragraph(vec![ComponentLine::new(vec![ActiveComponent::new(
                TextOrCustomRender::Text(Text::new(
                    "Want to customize the appearance and colors of Zellij?",
                )),
            )])])
            .with_paragraph(vec![
                ComponentLine::new(vec![
                    ActiveComponent::new(TextOrCustomRender::Text(
                        Text::new("查看内置主题： ").color_range(2, ..),
                    )),
                    ActiveComponent::new(TextOrCustomRender::Text(Text::new(
                        "https://zellij.dev/documentation/theme-list",
                    )))
                    .with_hover(TextOrCustomRender::CustomRender(
                        Box::new(theme_list_selected),
                        Box::new(theme_list_selected_len),
                    ))
                    .with_left_click_action(ClickAction::new_open_link(
                        "https://zellij.dev/documentation/theme-list".to_owned(),
                        link_executable.clone(),
                    )),
                ]),
                ComponentLine::new(vec![
                    ActiveComponent::new(TextOrCustomRender::Text(
                        Text::new("或创建你自己的主题： ").color_range(2, ..),
                    )),
                    ActiveComponent::new(TextOrCustomRender::Text(Text::new(
                        "https://zellij.dev/documentation/themes",
                    )))
                    .with_hover(TextOrCustomRender::CustomRender(
                        Box::new(theme_link_selected),
                        Box::new(theme_link_selected_len),
                    ))
                    .with_left_click_action(ClickAction::new_open_link(
                        "https://zellij.dev/documentation/themes".to_owned(),
                        link_executable.clone(),
                    )),
                ]),
            ])
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
            .with_help(Box::new(|hovering_over_link, _menu_item_is_selected| {
                tips_help_text(hovering_over_link)
            }))
    }
    pub fn tip_8(link_executable: Rc<RefCell<String>>) -> Page {
        Page::new()
            .main_screen()
            .with_title(Text::new("Zellij 提示 #8").color_range(0, ..))
            .with_paragraph(vec![
                ComponentLine::new(vec![
                    ActiveComponent::new(TextOrCustomRender::Text(
                        Text::new("如果你用 Alt + <←↓↑→> 或 Alt + <hjkl> 将窗格焦点移过")
                            .color_range(0, 5..=7)
                            .color_range(2, 11..=16)
                            .color_range(0, 20..=22)
                            .color_range(2, 26..=31)
                    ))
                ]),
                ComponentLine::new(vec![
                    ActiveComponent::new(TextOrCustomRender::Text(Text::new("屏幕的右侧或左侧，则会聚焦下一个或上一个标签页。")))
                ]),
            ])
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
            .with_help(Box::new(|hovering_over_link, _menu_item_is_selected| {
                tips_help_text(hovering_over_link)
            }))
    }
    pub fn tip_9(link_executable: Rc<RefCell<String>>) -> Page {
        Page::new()
            .main_screen()
            .with_title(Text::new("Zellij 提示 #9").color_range(0, ..))
            .with_paragraph(vec![
                ComponentLine::new(vec![
                    ActiveComponent::new(TextOrCustomRender::Text(
                        Text::new("想了解社区创建的插件、集成和教程，请查看")
                    ))
                ]),
                ComponentLine::new(vec![
                    ActiveComponent::new(TextOrCustomRender::Text(
                        Text::new("Awesome-zellij 仓库： ")
                            .color_range(2, ..)
                    )),
                    ActiveComponent::new(TextOrCustomRender::Text(Text::new("https://github.com/zellij-org/awesome-zellij")))
                        .with_hover(TextOrCustomRender::CustomRender(
                            Box::new(awesome_zellij_link_text_selected),
                            Box::new(awesome_zellij_link_text_selected_len),
                        ))
                        .with_left_click_action(ClickAction::new_open_link(
                            "https://github.com/zellij-org/awesome-zellij".to_owned(),
                            link_executable.clone(),
                        )),
                ]),
            ])
            .with_paragraph(vec![
                ComponentLine::new(vec![
                    ActiveComponent::new(TextOrCustomRender::Text(
                        Text::new("社区与支持：")
                            .color_range(2, ..)
                    ))
                ]),
                ComponentLine::new(vec![
                    ActiveComponent::new(TextOrCustomRender::Text(Text::new("Discord： "))),
                    ActiveComponent::new(TextOrCustomRender::Text(Text::new("https://discord.com/invite/CrUAFH3")))
                        .with_hover(TextOrCustomRender::CustomRender(
                            Box::new(discord_link_text_selected),
                            Box::new(discord_link_text_selected_len),
                        ))
                        .with_left_click_action(ClickAction::new_open_link(
                            "https://discord.com/invite/CrUAFH3".to_owned(),
                            link_executable.clone(),
                        )),
                ]),
                ComponentLine::new(vec![
                    ActiveComponent::new(TextOrCustomRender::Text(Text::new("Matrix： "))),
                    ActiveComponent::new(TextOrCustomRender::Text(Text::new("https://matrix.to/#/#zellij_general:matrix.org")))
                        .with_hover(TextOrCustomRender::CustomRender(
                            Box::new(matrix_link_text_selected),
                            Box::new(matrix_link_text_selected_len),
                        ))
                        .with_left_click_action(ClickAction::new_open_link(
                            "https://matrix.to/#/#zellij_general:matrix.org".to_owned(),
                            link_executable.clone(),
                        )),
                ])
            ])
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
            .with_help(Box::new(|hovering_over_link, _menu_item_is_selected| {
                tips_help_text(hovering_over_link)
            }))
    }
    pub fn tip_10(link_executable: Rc<RefCell<String>>, base_mode: Rc<RefCell<InputMode>>) -> Page {
        Page::new()
            .main_screen()
            .with_title(Text::new("Zellij 提示 #10").color_range(0, ..))
            .with_bulletin_list(
                BulletinList::new(
                    Text::new("Zellij 会话管理器可以：").color_range(2, 7..=12),
                )
                .with_items(vec![
                    ActiveComponent::new(TextOrCustomRender::Text(Text::new(
                        "创建新会话",
                    ))),
                    ActiveComponent::new(TextOrCustomRender::Text(Text::new(
                        "在现有会话之间切换",
                    ))),
                    ActiveComponent::new(TextOrCustomRender::Text(Text::new(
                        "恢复已退出的会话",
                    ))),
                    ActiveComponent::new(TextOrCustomRender::Text(Text::new(
                        "更改会话名称",
                    ))),
                    ActiveComponent::new(TextOrCustomRender::Text(Text::new(
                        "断开当前会话中的其他用户",
                    ))),
                ]),
            )
            .with_paragraph(vec![ComponentLine::new(vec![ActiveComponent::new(
                TextOrCustomRender::Text(match *base_mode.borrow() {
                    InputMode::Locked => Text::new("使用 Ctrl g + o + w 查看")
                        .color_range(3, 3..=8)
                        .color_indices(3, vec![12, 16]),
                    _ => Text::new("使用 Ctrl o + w 查看")
                        .color_range(3, 3..=8)
                        .color_indices(3, vec![12]),
                }),
            )])])
            .with_paragraph(vec![ComponentLine::new(vec![ActiveComponent::new(
                TextOrCustomRender::Text(
                    Text::new("你也可以把它当作欢迎界面使用：zellij -l welcome")
                        .color_range(0, 16..=32),
                ),
            )])])
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
            .with_help(Box::new(|hovering_over_link, _menu_item_is_selected| {
                tips_help_text(hovering_over_link)
            }))
    }
    pub fn tip_11(link_executable: Rc<RefCell<String>>) -> Page {
        Page::new()
            .main_screen()
            .with_title(Text::new("Zellij 提示 #11").color_range(0, ..))
            .with_paragraph(vec![
                ComponentLine::new(vec![
                    ActiveComponent::new(TextOrCustomRender::Text(
                        Text::new("你可以用 Alt + [] 改变屏幕上窗格的排列方式")
                            .color_range(0, 5..=7)
                            .color_range(2, 11..=12)
                    )),
                ]),
                ComponentLine::new(vec![
                    ActiveComponent::new(TextOrCustomRender::Text(
                        Text::new("这对平铺窗格或浮动窗格都有效，取决于哪个可见。")
                    ))
                ])
            ])
            .with_paragraph(vec![
                ComponentLine::new(vec![
                    ActiveComponent::new(TextOrCustomRender::Text(
                        Text::new("调整大小或拆分窗格会打破这种排列。此时仍可")
                    )),
                ]),
                ComponentLine::new(vec![
                    ActiveComponent::new(TextOrCustomRender::Text(
                        Text::new("再次按 Alt + [] 快速切回。此状态可显示在")
                            .color_range(0, 4..=6)
                            .color_range(2, 10..=11)
                    )),
                ]),
                ComponentLine::new(vec![
                    ActiveComponent::new(TextOrCustomRender::Text(
                        Text::new("屏幕的右上角。")
                    )),
                ]),
            ])
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
            .with_help(Box::new(|hovering_over_link, _menu_item_is_selected| {
                tips_help_text(hovering_over_link)
            }))
    }
    pub fn tip_12(link_executable: Rc<RefCell<String>>, base_mode: Rc<RefCell<InputMode>>) -> Page {
        Page::new()
            .main_screen()
            .with_title(Text::new("Zellij 提示 #12").color_range(0, ..))
            .with_paragraph(vec![
                ComponentLine::new(vec![
                    ActiveComponent::new(TextOrCustomRender::Text(
                        Text::new("Zellij 插件可以从插件管理器中加载、重新加载和跟踪。")
                    )),
                ]),
                ComponentLine::new(vec![
                    ActiveComponent::new(TextOrCustomRender::Text(
                            match *base_mode.borrow() {
                                InputMode::Locked => {
                                    Text::new("使用 Ctrl g + o + p 查看")
                                        .color_range(3, 3..=8)
                                        .color_indices(3, vec![12, 16])
                                },
                                _ => {
                                    Text::new("使用 Ctrl o + p 查看")
                                        .color_range(3, 3..=8)
                                        .color_indices(3, vec![12])
                                }
                            }
                    )),
                ]),
            ])
            .with_paragraph(vec![ComponentLine::new(vec![
                ActiveComponent::new(TextOrCustomRender::Text(Text::new("了解关于插件的更多信息： ").color_range(2, ..))),
                ActiveComponent::new(TextOrCustomRender::Text(Text::new("https://zellij.dev/documentation/plugins")))
                    .with_hover(TextOrCustomRender::CustomRender(
                        Box::new(plugin_docs_link_text_selected),
                        Box::new(plugin_docs_link_text_selected_len),
                    ))
                    .with_left_click_action(ClickAction::new_open_link(
                        "https://zellij.dev/documentation/plugins".to_owned(),
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
            .with_help(Box::new(|hovering_over_link, _menu_item_is_selected| {
                tips_help_text(hovering_over_link)
            }))
    }
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

fn plugin_docs_link_text_selected(x: usize, y: usize) -> usize {
    print!(
        "\u{1b}[{};{}H\u{1b}[m\u{1b}[1;4mhttps://zellij.dev/documentation/plugins",
        y + 1,
        x + 1
    );
    40
}

fn plugin_docs_link_text_selected_len() -> usize {
    40
}

fn awesome_zellij_link_text_selected(x: usize, y: usize) -> usize {
    print!(
        "\u{1b}[{};{}H\u{1b}[m\u{1b}[1;4mhttps://github.com/zellij-org/awesome-zellij",
        y + 1,
        x + 1
    );
    44
}

fn awesome_zellij_link_text_selected_len() -> usize {
    44
}

fn discord_link_text_selected(x: usize, y: usize) -> usize {
    print!(
        "\u{1b}[{};{}H\u{1b}[m\u{1b}[1;4mhttps://discord.com/invite/CrUAFH3",
        y + 1,
        x + 1
    );
    34
}

fn discord_link_text_selected_len() -> usize {
    34
}

fn matrix_link_text_selected(x: usize, y: usize) -> usize {
    print!(
        "\u{1b}[{};{}H\u{1b}[m\u{1b}[1;4mhttps://matrix.to/#/#zellij_general:matrix.org",
        y + 1,
        x + 1
    );
    46
}

fn matrix_link_text_selected_len() -> usize {
    46
}

fn stacked_resize_screencast_link_selected(x: usize, y: usize) -> usize {
    print!(
        "\u{1b}[{};{}H\u{1b}[m\u{1b}[1;4mhttps://zellij.dev/tutorials/stacked-resize",
        y + 1,
        x + 1
    );
    45
}

fn stacked_resize_screencast_link_selected_len() -> usize {
    45
}

fn colliding_keybindings_link_selected(x: usize, y: usize) -> usize {
    print!(
        "\u{1b}[{};{}H\u{1b}[m\u{1b}[1;4mhttps://zellij.dev/tutorials/colliding-keybindings",
        y + 1,
        x + 1
    );
    51
}

fn colliding_keybindings_link_selected_len() -> usize {
    51
}

fn theme_link_selected(x: usize, y: usize) -> usize {
    print!(
        "\u{1b}[{};{}H\u{1b}[m\u{1b}[1;4mhttps://zellij.dev/documentation/themes",
        y + 1,
        x + 1
    );
    39
}
fn theme_link_selected_len() -> usize {
    39
}

fn theme_list_selected(x: usize, y: usize) -> usize {
    print!(
        "\u{1b}[{};{}H\u{1b}[m\u{1b}[1;4mhttps://zellij.dev/documentation/theme-list",
        y + 1,
        x + 1
    );
    43
}
fn theme_list_selected_len() -> usize {
    43
}

fn support_the_developer_text() -> Text {
    let support_text = format!("请支持 Zellij 开发者 <3： ");
    Text::new(support_text).color_range(3, ..)
}

fn screencasts_link_selected() -> Box<dyn Fn(usize, usize) -> usize> {
    Box::new(move |x, y| {
        print!(
            "\u{1b}[{};{}H\u{1b}[m\u{1b}[1;4mhttps://zellij.dev/screencasts",
            y + 1,
            x + 1,
        );
        30
    })
}

fn screencasts_link_selected_len() -> Box<dyn Fn() -> usize> {
    Box::new(move || 30)
}

fn tips_help_text(hovering_over_link: bool) -> Text {
    if hovering_over_link {
        let help_text = format!("帮助：点击或 Shift-Click 在浏览器中打开");
        Text::new(help_text)
            
            .color_range(3, 7..=17)
    } else {
        let help_text = format!(
            "帮助：<ESC> - 关闭，<↓↑> - 浏览提示，<Ctrl c> - 启动时不显示提示"
        );
        Text::new(help_text)
            .color_range(1, 3..=7)
            .color_range(1, 14..=17)
            .color_range(1, 26..=33)
    }
}
