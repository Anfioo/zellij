/*
 *
 * 注意：这些测试非常重，仅作为冒烟测试用于验证应用程序端到端正常工作。
 * 避免添加新的测试，优先使用 zellij-integration-tests 模块——
 * 它将应用程序作为一个整体进行测试，仅模拟操作系统交互部分
 *
*/
#![allow(unused)]

use insta::assert_snapshot;
use zellij_utils::{
    pane_size::Size,
    position::{Column, Line, Position},
};

use rand::RngExt;
use regex::Regex;

use std::fmt::Write;
use std::path::Path;

use super::remote_runner::{RemoteRunner, RemoteTerminal, Step};

pub const QUIT: [u8; 1] = [17]; // ctrl-q
pub const ESC: [u8; 1] = [27];
pub const ENTER: [u8; 1] = [13]; // '\r'
pub const SPACE: [u8; 1] = [32];
pub const LOCK_MODE: [u8; 1] = [7]; // ctrl-g

pub const MOVE_FOCUS_LEFT_IN_NORMAL_MODE: [u8; 2] = [27, 104]; // alt-h
pub const MOVE_FOCUS_RIGHT_IN_NORMAL_MODE: [u8; 2] = [27, 108]; // alt-l

pub const PANE_MODE: [u8; 1] = [16]; // ctrl-p
pub const TMUX_MODE: [u8; 1] = [2]; // ctrl-b
pub const SPAWN_TERMINAL_IN_PANE_MODE: [u8; 1] = [110]; // n
pub const MOVE_FOCUS_IN_PANE_MODE: [u8; 1] = [112]; // p
pub const SPLIT_DOWN_IN_PANE_MODE: [u8; 1] = [100]; // d
pub const SPLIT_RIGHT_IN_PANE_MODE: [u8; 1] = [114]; // r
pub const SPLIT_RIGHT_IN_TMUX_MODE: [u8; 1] = [37]; // %
pub const TOGGLE_ACTIVE_TERMINAL_FULLSCREEN_IN_PANE_MODE: [u8; 1] = [102]; // f
pub const TOGGLE_FLOATING_PANES: [u8; 1] = [119]; // w
pub const CLOSE_PANE_IN_PANE_MODE: [u8; 1] = [120]; // x
pub const MOVE_FOCUS_DOWN_IN_PANE_MODE: [u8; 1] = [106]; // j
pub const MOVE_FOCUS_UP_IN_PANE_MODE: [u8; 1] = [107]; // k
pub const MOVE_FOCUS_LEFT_IN_PANE_MODE: [u8; 1] = [104]; // h
pub const MOVE_FOCUS_RIGHT_IN_PANE_MODE: [u8; 1] = [108]; // l
pub const RENAME_PANE_MODE: [u8; 1] = [99]; // c

pub const SCROLL_MODE: [u8; 1] = [19]; // ctrl-s
pub const SCROLL_UP_IN_SCROLL_MODE: [u8; 1] = [107]; // k
pub const SCROLL_DOWN_IN_SCROLL_MODE: [u8; 1] = [106]; // j
pub const SCROLL_PAGE_UP_IN_SCROLL_MODE: [u8; 1] = [2]; // ctrl-b
pub const SCROLL_PAGE_DOWN_IN_SCROLL_MODE: [u8; 1] = [6]; // ctrl-f
pub const EDIT_SCROLLBACK: [u8; 1] = [101]; // e

pub const RESIZE_MODE: [u8; 1] = [14]; // ctrl-n
pub const RESIZE_DOWN_IN_RESIZE_MODE: [u8; 1] = [106]; // j
pub const RESIZE_UP_IN_RESIZE_MODE: [u8; 1] = [107]; // k
pub const RESIZE_LEFT_IN_RESIZE_MODE: [u8; 1] = [104]; // h
pub const RESIZE_RIGHT_IN_RESIZE_MODE: [u8; 1] = [108]; // l

pub const TAB_MODE: [u8; 1] = [20]; // ctrl-t
pub const NEW_TAB_IN_TAB_MODE: [u8; 1] = [110]; // n
pub const SWITCH_NEXT_TAB_IN_TAB_MODE: [u8; 1] = [108]; // l
pub const SWITCH_PREV_TAB_IN_TAB_MODE: [u8; 1] = [104]; // h
pub const CLOSE_TAB_IN_TAB_MODE: [u8; 1] = [120]; // x
pub const RENAME_TAB_MODE: [u8; 1] = [114]; // r

pub const MOVE_TAB_LEFT: [u8; 2] = [27, 105]; // Alt + i
pub const MOVE_TAB_RIGHT: [u8; 2] = [27, 111]; // Alt + o

pub const SESSION_MODE: [u8; 1] = [15]; // ctrl-o
pub const DETACH_IN_SESSION_MODE: [u8; 1] = [100]; // d

pub const BRACKETED_PASTE_START: [u8; 6] = [27, 91, 50, 48, 48, 126]; // \u{1b}[200~
pub const BRACKETED_PASTE_END: [u8; 6] = [27, 91, 50, 48, 49, 126]; // \u{1b}[201
pub const SLEEP: [u8; 0] = [];

pub const SECOND_TAB_CONTENT: [u8; 14] =
    [84, 97, 98, 32, 35, 50, 32, 99, 111, 110, 116, 101, 110, 116]; // Tab #2 content

// 我们在这里做的是针对各种竞态条件调整快照，希望这些调整是暂时的，
// 直到我们能够修复它们——在这里添加内容时，请添加详细注释
// 解释竞态条件以及解决它需要做什么
fn account_for_races_in_snapshot(snapshot: String) -> String {
    // 这些替换是必要的，因为插件在加载时会将自己设置为"不可选择"——
    // 由于它们是异步加载的，有时"BASE"指示（仅当有多个可选择窗格时才应出现）
    // 会被渲染，有时不会——这里将其完全移除
    //
    // 要修复此问题，我们应该在布局中（加载之前）将插件设置为不可选择，
    // 一旦完成，我们应该能够移除这个 hack（并调整此处不得不去掉的尾随空格的快照）
    let base_replace = Regex::new(r"Alt <\[\]>  BASE \s*\n").unwrap();
    let base_replace_tmux_mode_1 = Regex::new(r"Alt \[\|SPACE\|Alt \]  BASE \s*\n").unwrap();
    let base_replace_tmux_mode_2 = Regex::new(r"Alt \[\|Alt \]\|SPACE  BASE \s*\n").unwrap();
    let eol_arrow_replace = Regex::new(r"\s*\n").unwrap();
    let snapshot = base_replace.replace_all(&snapshot, "\n").to_string();
    let snapshot = base_replace_tmux_mode_1
        .replace_all(&snapshot, "\n")
        .to_string();
    let snapshot = base_replace_tmux_mode_2
        .replace_all(&snapshot, "\n")
        .to_string();
    let snapshot = eol_arrow_replace.replace_all(&snapshot, "\n").to_string();

    snapshot
}

// 所有 E2E 测试都标记为 "ignored"，以便可以与常规测试分开运行

#[test]
#[ignore]
pub fn starts_with_one_terminal() {
    let fake_win_size = Size {
        cols: 120,
        rows: 24,
    };
    let mut test_attempts = 10;
    let last_snapshot = loop {
        RemoteRunner::kill_running_sessions(fake_win_size);
        let mut runner = RemoteRunner::new(fake_win_size);
        let last_snapshot = runner.take_snapshot_after(Step {
            name: "Wait for app to load",
            instruction: |remote_terminal: RemoteTerminal| -> bool {
                let mut step_is_complete = false;
                if remote_terminal.status_bar_appears() && remote_terminal.cursor_position_is(2, 1)
                {
                    step_is_complete = true;
                }
                step_is_complete
            },
        });
        if runner.test_timed_out && test_attempts > 0 {
            test_attempts -= 1;
            continue;
        } else {
            break last_snapshot;
        }
    };

    let last_snapshot = account_for_races_in_snapshot(last_snapshot);
    assert_snapshot!(last_snapshot);
}

#[test]
#[ignore]
pub fn typing_exit_closes_pane() {
    let fake_win_size = Size {
        cols: 120,
        rows: 24,
    };
    let mut test_attempts = 10;
    let last_snapshot = loop {
        RemoteRunner::kill_running_sessions(fake_win_size);
        let mut runner = RemoteRunner::new(fake_win_size)
            .add_step(Step {
                name: "Split pane to the right",
                instruction: |mut remote_terminal: RemoteTerminal| -> bool {
                    let mut step_is_complete = false;
                    if remote_terminal.status_bar_appears()
                        && remote_terminal.cursor_position_is(2, 1)
                    {
                        remote_terminal.send_key(&PANE_MODE);
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        remote_terminal.send_key(&SPLIT_RIGHT_IN_PANE_MODE);
                        step_is_complete = true;
                    }
                    step_is_complete
                },
            })
            .add_step(Step {
                name: "Type exit",
                instruction: |mut remote_terminal: RemoteTerminal| -> bool {
                    let mut step_is_complete = false;
                    if remote_terminal.cursor_position_is(62, 2)
                        && remote_terminal.status_bar_appears()
                    {
                        remote_terminal.send_key("e".as_bytes());
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        remote_terminal.send_key("x".as_bytes());
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        remote_terminal.send_key("i".as_bytes());
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        remote_terminal.send_key("t".as_bytes());
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        remote_terminal.send_key("\n".as_bytes());
                        step_is_complete = true;
                    }
                    step_is_complete
                },
            });
        runner.run_all_steps();
        let last_snapshot = runner.take_snapshot_after(Step {
            name: "Wait for pane to close",
            instruction: |remote_terminal: RemoteTerminal| -> bool {
                let mut step_is_complete = false;
                if remote_terminal.cursor_position_is(2, 1) && remote_terminal.status_bar_appears()
                {
                    // 光标在原始窗格中
                    step_is_complete = true;
                }
                step_is_complete
            },
        });
        if runner.test_timed_out && test_attempts > 0 {
            test_attempts -= 1;
            continue;
        } else {
            break last_snapshot;
        }
    };
    let last_snapshot = account_for_races_in_snapshot(last_snapshot);
    assert_snapshot!(last_snapshot);
}

#[test]
#[ignore]
pub fn resize_terminal_window() {
    // 这检查整个终端窗口的大小调整（对 SIGWINCH 的反应），而不仅仅是单个窗格
    let fake_win_size = Size {
        cols: 120,
        rows: 24,
    };
    let mut test_attempts = 10;
    let last_snapshot = loop {
        RemoteRunner::kill_running_sessions(fake_win_size);
        let mut runner = RemoteRunner::new(fake_win_size)
            .add_step(Step {
                name: "Split pane to the right",
                instruction: |mut remote_terminal: RemoteTerminal| -> bool {
                    let mut step_is_complete = false;
                    if remote_terminal.status_bar_appears()
                        && remote_terminal.cursor_position_is(2, 1)
                    {
                        remote_terminal.send_key(&PANE_MODE);
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        remote_terminal.send_key(&SPLIT_RIGHT_IN_PANE_MODE);
                        step_is_complete = true;
                    }
                    step_is_complete
                },
            })
            .add_step(Step {
                name: "Change terminal window size",
                instruction: |mut remote_terminal: RemoteTerminal| -> bool {
                    let mut step_is_complete = false;
                    if remote_terminal.cursor_position_is(62, 2)
                        && remote_terminal.status_bar_appears()
                    {
                        // 新窗格已打开并获得焦点
                        remote_terminal.change_size(100, 24);
                        step_is_complete = true;
                    }
                    step_is_complete
                },
            });
        runner.run_all_steps();
        let last_snapshot = runner.take_snapshot_after(Step {
            name: "wait for terminal to be resized and app to be re-rendered",
            instruction: |remote_terminal: RemoteTerminal| -> bool {
                let mut step_is_complete = false;
                if remote_terminal.cursor_position_is(52, 2) && remote_terminal.ctrl_plus_appears()
                {
                    // 大小已更改
                    step_is_complete = true;
                }
                step_is_complete
            },
        });
        if runner.test_timed_out && test_attempts > 0 {
            test_attempts -= 1;
            continue;
        } else {
            break last_snapshot;
        }
    };
    let last_snapshot = account_for_races_in_snapshot(last_snapshot);
    assert_snapshot!(last_snapshot);
}

#[test]
#[ignore]
pub fn quit_and_resurrect_session() {
    let fake_win_size = Size {
        cols: 120,
        rows: 24,
    };
    let mut test_attempts = 10;
    let layout_name = "layout_for_resurrection.kdl";
    let last_snapshot = loop {
        RemoteRunner::kill_running_sessions(fake_win_size);
        let mut runner = RemoteRunner::new_mirrored_session_with_layout(fake_win_size, layout_name)
            .add_step(Step {
                name: "Wait for session to be serialized",
                instruction: |mut remote_terminal: RemoteTerminal| -> bool {
                    let mut step_is_complete = false;
                    if remote_terminal.snapshot_contains("Waiting to run: top") {
                        std::thread::sleep(std::time::Duration::from_millis(5000)); // 等待
                                                                                    // 序列化
                        remote_terminal.send_key(&QUIT);
                        step_is_complete = true;
                    }
                    step_is_complete
                },
            })
            .add_step(Step {
                name: "Resurrect session by attaching",
                instruction: |mut remote_terminal: RemoteTerminal| -> bool {
                    let mut step_is_complete = false;
                    if remote_terminal.snapshot_contains("Bye from Zellij!") {
                        remote_terminal.attach_to_original_session();
                        step_is_complete = true;
                    }
                    step_is_complete
                },
            });
        runner.run_all_steps();
        let last_snapshot = runner.take_snapshot_after(Step {
            name: "Wait for session to be resurrected",
            instruction: |remote_terminal: RemoteTerminal| -> bool {
                remote_terminal.snapshot_contains("(FLOATING PANES VISIBLE)")
                    && remote_terminal.status_bar_appears()
                    && remote_terminal.tab_bar_appears()
            },
        });
        if runner.test_timed_out && test_attempts > 0 {
            test_attempts -= 1;
            continue;
        } else {
            break last_snapshot;
        }
    };
    let last_snapshot = account_for_races_in_snapshot(last_snapshot);
    assert_snapshot!(last_snapshot);
}

#[test]
#[ignore]
pub fn send_blocking_command_through_the_cli() {
    // 在这里我们测试以下流程：
    // - 通过 cli 发送阻塞命令，使用 --blocking --floating --close-on-exit
    // - 命令休眠 2 秒（长于默认的 1 秒超时），然后以状态码 42 退出
    // - 验证 CLI 阻塞了完整持续时间（我们在 2 秒以上后检查）
    // - 验证浮动窗格在运行时出现，完成后消失
    // - 验证退出状态正确传播
    let fake_win_size = Size {
        cols: 150,
        rows: 24,
    };
    let mut test_attempts = 10;
    let last_snapshot = loop {
        RemoteRunner::kill_running_sessions(fake_win_size);
        let mut runner = RemoteRunner::new(fake_win_size)
            .add_step(Step {
                name: "Run blocking command through the cli",
                instruction: |mut remote_terminal: RemoteTerminal| -> bool {
                    let mut step_is_complete = false;
                    if remote_terminal.status_bar_appears()
                        && remote_terminal.cursor_position_is(2, 1)
                    {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        remote_terminal
                            .send_blocking_command_through_the_cli("bash -c 'sleep 2 && exit 42'");
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        remote_terminal.send_key(&ENTER);
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        step_is_complete = true;
                    }
                    step_is_complete
                },
            })
            .add_step(Step {
                name: "Wait for floating pane to appear",
                instruction: |remote_terminal: RemoteTerminal| -> bool {
                    let mut step_is_complete = false;
                    // 浮动窗格应随正在运行的命令一起出现
                    if remote_terminal.snapshot_contains("PIN [ ]") {
                        std::thread::sleep(std::time::Duration::from_millis(2000)); // 等待
                                                                                    // 命令
                                                                                    // 结束
                        step_is_complete = true
                    }
                    step_is_complete
                },
            })
            .add_step(Step {
                name: "Wait for command to complete and verify exit status",
                instruction: |mut remote_terminal: RemoteTerminal| -> bool {
                    let mut step_is_complete = false;
                    // 2 秒以上后，命令应完成，浮动窗格应关闭
                    // 等待浮动窗格消失且 shell 提示符返回后再
                    // 请求 $?，否则我们可能与阻塞 CLI 进程本身
                    // 返回 shell 发生竞态。
                    if !remote_terminal.snapshot_contains("PIN [ ]")
                        && remote_terminal.snapshot_contains("$ \u{2588}")
                        && remote_terminal.status_bar_appears()
                    {
                        remote_terminal.send_key("echo $?".as_bytes());
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        remote_terminal.send_key(&ENTER);
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        step_is_complete = true
                    }
                    step_is_complete
                },
            });
        runner.run_all_steps();

        let last_snapshot = runner.take_snapshot_after(Step {
            name: "Verify CLI returned with proper exit status after command completed",
            instruction: |remote_terminal: RemoteTerminal| -> bool {
                let mut step_is_complete = false;
                // 等待 echo $? 可见、退出状态已渲染、光标回到
                // 空白提示符，这意味着 shell 命令确实已执行
                if remote_terminal.snapshot_contains("echo $?")
                    && remote_terminal.snapshot_contains("42")
                    && remote_terminal.snapshot_contains("$ \u{2588}")
                    && remote_terminal.status_bar_appears()
                {
                    step_is_complete = true
                }
                step_is_complete
            },
        });

        if runner.test_timed_out && test_attempts > 0 {
            test_attempts -= 1;
            continue;
        } else {
            break last_snapshot;
        }
    };
    let last_snapshot = account_for_races_in_snapshot(last_snapshot);
    assert_snapshot!(last_snapshot);
}
