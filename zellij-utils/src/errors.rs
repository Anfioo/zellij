// 误报：thiserror 的 derive 宏会在结构体风格的枚举变体字段上触发 unused_assignments
#![allow(unused_assignments)]
//! 基于线程局部调用栈表示的错误上下文系统，该调用栈本身基于在线程之间发送的指令。
//!
//! # 寻求帮助
//!
//! 目前正在进行一项改进 zellij 错误处理状态的工作。目前，许多函数依赖于对 [`Result`] 进行 [`unwrap`]，
//! 而不是返回并因此传播潜在的错误。如果您有兴趣帮助为 zellij 添加错误处理，请随时与我们联系。
//! 更多信息可以在 [关于错误处理的文档](https://github.com/zellij-org/zellij/tree/main/docs/ERROR_HANDLING.md) 中找到。

use anyhow::Context;
use colored::*;
#[allow(unused_imports)] // 在 set_panic_handler 中使用；在 wasm 目标下可能显示为未使用
use log::error;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};
use std::path::PathBuf;

/// 常见错误处理代码的重新导出。
pub mod prelude {
    pub use super::FatalError;
    pub use super::LoggableError;
    #[cfg(not(target_family = "wasm"))]
    pub use super::ToAnyhow;
    pub use super::ZellijError;
    pub use anyhow::anyhow;
    pub use anyhow::bail;
    pub use anyhow::Context;
    pub use anyhow::Error as anyError;
    pub use anyhow::Result;
}

pub trait ErrorInstruction {
    fn error(err: String) -> Self;
}

/// 用于轻松记录错误类型的辅助 trait。
///
/// `print_error` 函数接受一个闭包，该闭包接受一个 `&str` 并根据需要处理它，
/// 以将错误记录到某个可用的位置。为方便起见，已经实现了记录到 stdout、stderr 和
/// `log::error!`。
///
/// 注意，trait 函数会原样传递错误，因此它们可以与通常的 [`std::result::Result`] 类型处理链接使用。
pub trait LoggableError<T>: Sized {
    /// 将从 `self` 派生的格式化错误消息传递给闭包 `fun`，以便适当地打印/记录。
    ///
    /// # 示例
    ///
    /// ```should_panic
    /// use anyhow;
    /// use zellij_utils::errors::LoggableError;
    ///
    /// let my_err: anyhow::Result<&str> = Err(anyhow::anyhow!("Test error"));
    /// my_err
    ///     .print_error(|msg| println!("{msg}"))
    ///     .unwrap();
    /// ```
    #[track_caller]
    fn print_error<F: Fn(&str)>(self, fun: F) -> Self;

    /// 便捷函数，调用 `print_error` 并将结果记录为错误。
    ///
    /// 这不是 `log::error!` 的包装器，因为 `log` crate 使用了大量来自 `std` 的编译时宏
    /// 来确定调用者位置/模块名称等。由于这些是在编写它们的位置在编译时解析的，
    /// 它们将始终解析到此函数中调用 `log::error!` 的位置，从而掩盖了真正的调用者位置。
    /// 因此，我们自己构建日志消息。这意味着我们丢失了关于调用模块的信息
    /// （因为它只能在编译时解析），但是调用者的文件和行号被保留了。
    #[track_caller]
    fn to_log(self) -> Self {
        let caller = std::panic::Location::caller();
        self.print_error(|msg| {
            // 手动构建日志条目
            // 注意：日志条目没有关联的模块路径。这是因为 `log`
            // 从 `std::module_path!()` 宏获取模块路径，该宏在编写它的位置在编译时被替换！
            log::logger().log(
                &log::Record::builder()
                    .level(log::Level::Error)
                    .args(format_args!("{}", msg))
                    .file(Some(caller.file()))
                    .line(Some(caller.line()))
                    .module_path(None)
                    .build(),
            );
        })
    }

    /// 便捷函数，使用闭包 `|msg| eprintln!("{}", msg)` 调用 `print_error`。
    fn to_stderr(self) -> Self {
        self.print_error(|msg| eprintln!("{}", msg))
    }

    /// 便捷函数，使用闭包 `|msg| println!("{}", msg)` 调用 `print_error`。
    fn to_stdout(self) -> Self {
        self.print_error(|msg| println!("{}", msg))
    }
}

impl<T> LoggableError<T> for anyhow::Result<T> {
    fn print_error<F: Fn(&str)>(self, fun: F) -> Self {
        if let Err(ref err) = self {
            fun(&format!("{:?}", err));
        }
        self
    }
}

/// 用于标记致命/非致命错误的特殊 trait。
///
/// 这与上面的 `LoggableError` 协同工作，旨在使阅读代码时更容易判断错误是否致命
/// （即可以忽略，或者至少不会使应用程序崩溃）。
///
/// 这实质上是将任何 `std::result::Result<(), _>` 降级为简单的 `()`。
pub trait FatalError<T> {
    /// 将结果标记为非致命。
    ///
    /// 如果结果是 `Err` 变体，这将 [将错误打印到日志][`to_log`]。
    /// 之后丢弃结果类型。
    ///
    /// [`to_log`]: LoggableError::to_log
    #[track_caller]
    fn non_fatal(self);

    /// 将结果标记为致命。
    ///
    /// 如果结果是 `Err` 变体，这将 unwrap 错误并使应用程序 panic。
    /// 如果结果是 `Ok` 变体，则 unwrap 内部值并返回它。
    ///
    /// # Panics
    ///
    /// 如果给定结果是 `Err` 变体。
    #[track_caller]
    fn fatal(self) -> T;
}

/// 用于消除 `#[warn(unused_must_use)]` cargo 警告的辅助函数。仅在 `FatalError::non_fatal` 中使用！
fn discard_result<T>(_arg: anyhow::Result<T>) {}

impl<T> FatalError<T> for anyhow::Result<T> {
    fn non_fatal(self) {
        if self.is_err() {
            discard_result(self.context("a non-fatal error occurred").to_log());
        }
    }

    fn fatal(self) -> T {
        if let Ok(val) = self {
            val
        } else {
            self.context("a fatal error occurred")
                .expect("Program terminates")
        }
    }
}

/// 构成 [`ErrorContext`] 调用栈的不同类型的调用。
///
/// 复杂变体存储相关枚举的一个变体，其变体可以从对应的 Zellij MSPC 指令枚举变体
/// （[`ScreenInstruction`]、[`PtyInstruction`]、[`ClientInstruction`] 等）构建。
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum ContextType {
    /// 与屏幕相关的调用。
    Screen(ScreenContext),
    /// 与 PTY 相关的调用。
    Pty(PtyContext),
    /// 与插件相关的调用。
    Plugin(PluginContext),
    /// 与应用相关的调用。
    Client(ClientContext),
    /// 与服务端相关的调用。
    IPCServer(ServerContext),
    StdinHandler,
    AsyncTask,
    PtyWrite(PtyWriteContext),
    BackgroundJob(BackgroundJobContext),
    /// 一个空的占位调用。这应该被认为表示根本没有调用。
    /// 填充了这些的调用栈表示是空调用栈的表示。
    Empty,
}

impl Display for ContextType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        if let Some((left, right)) = match *self {
            ContextType::Screen(c) => Some(("screen_thread:", format!("{:?}", c))),
            ContextType::Pty(c) => Some(("pty_thread:", format!("{:?}", c))),
            ContextType::Plugin(c) => Some(("plugin_thread:", format!("{:?}", c))),
            ContextType::Client(c) => Some(("main_thread:", format!("{:?}", c))),
            ContextType::IPCServer(c) => Some(("ipc_server:", format!("{:?}", c))),
            ContextType::StdinHandler => Some(("标准输入处理线程：", "AcceptInput".to_string())),
            ContextType::AsyncTask => Some(("stream_terminal_bytes:", "AsyncTask".to_string())),
            ContextType::PtyWrite(c) => Some(("PTY 写入线程：", format!("{:?}", c))),
            ContextType::BackgroundJob(c) => Some(("后台任务线程：", format!("{:?}", c))),
            ContextType::Empty => None,
        } {
            write!(f, "{} {}", left.purple(), right.green())
        } else {
            write!(f, "")
        }
    }
}

// FIXME: 只需从 strum 派生 EnumDiscriminants 就可以消除对这些的任何需求！！！
/// 对应于不同类型 [`ScreenInstruction`] 的栈调用表示。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ScreenContext {
    HandlePtyBytes,
    PluginBytes,
    Render,
    RenderToClients,
    NewPane,
    OpenInPlaceEditor,
    ToggleFloatingPanes,
    ShowFloatingPanes,
    HideFloatingPanes,
    AreFloatingPanesVisible,
    TogglePaneEmbedOrFloating,
    HorizontalSplit,
    VerticalSplit,
    WriteCharacter,
    ResizeIncreaseAll,
    ResizeIncreaseLeft,
    ResizeIncreaseDown,
    ResizeIncreaseUp,
    ResizeIncreaseRight,
    ResizeDecreaseAll,
    ResizeDecreaseLeft,
    ResizeDecreaseDown,
    ResizeDecreaseUp,
    ResizeDecreaseRight,
    ResizeLeft,
    ResizeRight,
    ResizeDown,
    ResizeUp,
    ResizeIncrease,
    ResizeDecrease,
    SwitchFocus,
    FocusNextPane,
    FocusPreviousPane,
    FocusLastPane,
    FocusPaneAt,
    MoveFocusLeft,
    MoveFocusLeftOrPreviousTab,
    MoveFocusDown,
    MoveFocusUp,
    MoveFocusRight,
    MoveFocusRightOrNextTab,
    MovePane,
    MovePaneBackwards,
    MovePaneDown,
    MovePaneUp,
    MovePaneRight,
    MovePaneLeft,
    Exit,
    ClearScreen,
    DumpScreen,
    DumpLayout,
    SaveSession,
    EditScrollback,
    GetPaneScrollback,
    ScrollUp,
    ScrollUpAt,
    ScrollDown,
    ScrollDownAt,
    ScrollToBottom,
    ScrollToTop,
    ScrollToPreviousPrompt,
    ScrollToNextPrompt,
    SelectCommandAtScrollPosition,
    CopyLastCommandOutput,
    ClearCommandOutputFlash,
    PageScrollUp,
    PageScrollDown,
    HalfPageScrollUp,
    HalfPageScrollDown,
    ClearScroll,
    CloseFocusedPane,
    ToggleActiveSyncTab,
    ToggleActiveTerminalFullscreen,
    ToggleActiveTerminalNoUiFullscreen,
    TogglePaneFrames,
    SetPaneFrameStyle,
    SetSelectable,
    ShowPluginCursor,
    SetInvisibleBorders,
    SetFixedHeight,
    SetFixedWidth,
    ClosePane,
    HoldPane,
    UpdatePaneName,
    UndoRenamePane,
    NewTab,
    ApplyLayout,
    SwitchTabNext,
    SwitchTabPrev,
    CloseTab,
    GoToTab,
    GoToTabName,
    UpdateTabName,
    UndoRenameTab,
    MoveTabLeft,
    MoveTabRight,
    GoToTabWithId,
    CloseTabWithId,
    RenameTabWithId,
    BreakPanesToTabWithId,
    RecomputeTabSize,
    TerminalPixelDimensions,
    TerminalBackgroundColor,
    TerminalForegroundColor,
    TerminalColorRegisters,
    SetKittyGraphicsSupport,
    SetSixelSupport,
    ForwardHostQuery,
    NestedSessionMessageFromPane,
    NestedGuestPingTick,
    NestedSessionMessageFromHost,
    GuestModalChoice,
    ForwardedReplyFromHost,
    ResumePaneAfterForward,
    HostTerminalThemeChanged,
    SetDarkTheme,
    SetLightTheme,
    ToggleTheme,
    ChangeMode,
    ChangeModeForAllClients,
    LeftClick,
    RightClick,
    MiddleClick,
    LeftMouseRelease,
    RightMouseRelease,
    MiddleMouseRelease,
    MouseEvent,
    Copy,
    ToggleTab,
    AddClient,
    RemoveClient,
    UpdateSearch,
    SearchDown,
    SearchUp,
    SearchToggleCaseSensitivity,
    SearchToggleWholeWord,
    SearchToggleWrap,
    AddRedPaneFrameColorOverride,
    ClearPaneFrameColorOverride,
    SetTabBellFlash,
    HostTerminalFocusChanged,
    SetClientHostTerminalEnv,
    ForwardDesktopNotifications,
    PreviousSwapLayout,
    NextSwapLayout,
    OverrideLayout,
    OverrideLayoutComplete,
    QueryTabNames,
    NewTiledPluginPane,
    StartOrReloadPluginPane,
    NewFloatingPluginPane,
    AddPlugin,
    UpdatePluginLoadingStage,
    ProgressPluginLoadingOffset,
    StartPluginLoadingIndication,
    RequestStateUpdateForPlugins,
    LaunchOrFocusPlugin,
    LaunchPlugin,
    SuppressPane,
    UnsuppressPane,
    UnsuppressOrExpandPane,
    FocusPaneWithId,
    RenamePane,
    RenameActivePane,
    RenameTab,
    RequestPluginPermissions,
    BreakPane,
    BreakPaneRight,
    BreakPaneLeft,
    UpdateSessionInfos,
    UpdateAvailableLayouts,
    ReplacePane,
    NewInPlacePluginPane,
    SerializeLayoutForResurrection,
    RenameSession,
    DumpLayoutToPlugin,
    GetFocusedPaneInfo,
    GetPaneInfo,
    GetTabInfo,
    ListClientsMetadata,
    ListPanes,
    ListTabs,
    GetCurrentTabInfo,
    Reconfigure,
    RerunCommandPane,
    ResizePaneWithId,
    EditScrollbackForPaneWithId,
    WriteToPaneId,
    Paste,
    SetPaneColor,
    WriteKeyToPaneId,
    CopyTextToClipboard,
    MovePaneWithPaneId,
    MovePaneWithPaneIdInDirection,
    ClearScreenForPaneId,
    ScrollUpInPaneId,
    ScrollDownInPaneId,
    ScrollToTopInPaneId,
    ScrollToBottomInPaneId,
    PageScrollUpInPaneId,
    PageScrollDownInPaneId,
    TogglePaneIdFullscreen,
    SetMobileRenderPreferences,
    TogglePaneEmbedOrEjectForPaneId,
    CloseTabWithIndex,
    BreakPanesToNewTab,
    BreakPanesToTabWithIndex,
    ListClientsToPlugin,
    TogglePanePinned,
    SetFloatingPanePinned,
    StackPanes,
    ChangeFloatingPanesCoordinates,
    TogglePaneBorderless,
    SetPaneBorderless,
    AddHighlightPaneFrameColorOverride,
    GroupAndUngroupPanes,
    HighlightAndUnhighlightPanes,
    FloatMultiplePanes,
    EmbedMultiplePanes,
    TogglePaneInGroup,
    ToggleGroupMarking,
    SessionSharingStatusChange,
    SetMouseSelectionSupport,
    InterceptKeyPresses,
    ClearKeyPressesIntercepts,
    ReplacePaneWithExistingPane,
    AddWatcherClient,
    RemoveWatcherClient,
    SetFollowedClient,
    WatcherTerminalResize,
    ClearMouseHelpText,
    SetPluginRegexHighlights,
    ClearPluginHighlights,
    DesktopNotificationResponse,
    SubscribeToPaneRenders,
    NotifyPaneClosedToSubscribers,
    // 以窗格为目标的 CLI 变体
    ScrollUpWithPaneId,
    ScrollDownWithPaneId,
    ScrollToTopWithPaneId,
    ScrollToBottomWithPaneId,
    PageScrollUpWithPaneId,
    PageScrollDownWithPaneId,
    HalfPageScrollUpWithPaneId,
    HalfPageScrollDownWithPaneId,
    ResizeWithPaneId,
    MovePaneWithPaneIdCli,
    MovePaneBackwardsWithPaneId,
    ClearScreenWithPaneId,
    EditScrollbackWithPaneId,
    ToggleFullscreenWithPaneId,
    ToggleNoUiFullscreenWithPaneId,
    TogglePaneEmbedOrFloatingWithPaneId,
    CloseFocusWithPaneId,
    RenamePaneWithPaneId,
    UndoRenamePaneWithPaneId,
    TogglePanePinnedWithPaneId,
    // 以标签页为目标的 CLI 变体
    UndoRenameTabWithTabId,
    ToggleActiveSyncTabWithTabId,
    ToggleFloatingPanesWithTabId,
    PreviousSwapLayoutWithTabId,
    NextSwapLayoutWithTabId,
    MoveTabWithTabId,
    UpdateBackgroundPluginSubscriptions,
    ClearHintTextCache,
    BroadcastModeUpdate,
    SetSoftKeyboard,
    FocusHostSession,
    FocusGuestSession,
    ToggleHostFullscreen,
}

/// 对应于不同类型 [`PtyInstruction`] 的栈调用表示。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PtyContext {
    SpawnTerminal,
    OpenInPlaceEditor,
    SpawnTerminalVertically,
    SpawnTerminalHorizontally,
    UpdateActivePane,
    GoToTab,
    NewTab,
    OverrideLayout,
    ClosePane,
    CloseTab,
    ReRunCommandInPane,
    DropToShellInPane,
    SpawnInPlaceTerminal,
    DumpLayout,
    LogLayoutToHd,
    SaveSessionToDisk,
    FillPluginCwd,
    DumpLayoutToPlugin,
    ListClientsMetadata,
    Reconfigure,
    ListClientsToPlugin,
    ReportPluginCwd,
    SendSigintToPaneId,
    SendSigkillToPaneId,
    GetPanePid,
    GetPaneRunningCommand,
    GetPaneCwd,
    UpdateAndReportCwds,
    NotifyCwdFromOsc7,
    Exit,
}

/// 对应于不同类型 [`PluginInstruction`] 的栈调用表示。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PluginContext {
    Load,
    LoadBackgroundPlugin,
    Update,
    Render,
    Unload,
    Reload,
    ReloadPluginWithId,
    Resize,
    Exit,
    AddClient,
    RemoveClient,
    NewTab,
    OverrideLayout,
    ApplyCachedEvents,
    ApplyCachedWorkerMessages,
    PostMessageToPluginWorker,
    PostMessageToPlugin,
    PluginSubscribedToEvents,
    PermissionRequestResult,
    DumpLayout,
    LogLayoutToHd,
    CliPipe,
    Message,
    CachePluginEvents,
    MessageFromPlugin,
    UnblockCliPipes,
    WatchFilesystem,
    KeybindPipe,
    DumpLayoutToPlugin,
    ListClientsMetadata,
    Reconfigure,
    FailedToWriteConfigToDisk,
    ListClientsToPlugin,
    ChangePluginHostDir,
    WebServerStarted,
    FailedToStartWebServer,
    PaneRenderReport,
    UserInput,
    LayoutListUpdate,
    RequestStateUpdateForPlugin,
    UpdateSessionSaveTime,
    GetLastSessionSaveTime,
    DetectPluginConfigChanges,
    HighlightClicked,
}

/// 对应于不同类型 [`ClientInstruction`] 的栈调用表示。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ClientContext {
    Exit,
    Error,
    UnblockInputThread,
    Render,
    ServerError,
    SwitchToMode,
    Connected,
    Log,
    LogError,
    OwnClientId,
    SwitchSession,
    SetSynchronisedOutput,
    UnblockCliPipeInput,
    CliPipeOutput,
    QueryTerminalSize,
    WriteConfigToDisk,
    StartWebServer,
    RenamedSession,
    ConfigFileUpdated,
    ForwardQueryToHost,
    EmitNestedSessionFrame,
}

/// 对应于不同类型 [`ServerInstruction`] 的栈调用表示。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ServerContext {
    NewClient,
    Render,
    UnblockInputThread,
    ClientExit,
    RemoveClient,
    Error,
    KillSession,
    DetachSession,
    AttachClient,
    ConnStatus,
    Log,
    LogError,
    SwitchSession,
    UnblockCliPipeInput,
    CliPipeOutput,
    AssociatePipeWithClient,
    DisconnectAllClientsExcept,
    ChangeMode,
    ChangeModeForAllClients,
    Reconfigure,
    ConfigWrittenToDisk,
    FailedToWriteConfigToDisk,
    RebindKeys,
    StartWebServer,
    ShareCurrentSession,
    StopSharingCurrentSession,
    WebServerStarted,
    FailedToStartWebServer,
    SendWebClientsForbidden,
    ClearMouseHelpText,
    ClearCommandOutputFlash,
    ForwardQueryToHost,
    KeyPassthroughChanged,
    EmitNestedSessionFrameToClient,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PtyWriteContext {
    Write,
    ResizePty,
    StartCachingResizes,
    ApplyCachedResizes,
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BackgroundJobContext {
    DisplayPaneError,
    AnimatePluginLoading,
    StopPluginLoadingAnimation,
    ReportSessionInfo,
    ReportLayoutInfo,
    RunCommand,
    WebRequest,
    ReportPluginList,
    ListWebSessions,
    RenderToClients,
    HighlightPanesWithMessage,
    QueryZellijWebServerStatus,
    ClearHelpText,
    ClearCommandOutputFlash,
    FlashPaneBell,
    StopFlashPaneBell,
    FlashTabBell,
    StopFlashTabBell,
    StartNestedGuestPing,
    StopNestedGuestPing,
    Exit,
}

use thiserror::Error;
#[derive(Debug, Error)]
pub enum ZellijError {
    #[error("could not find command '{command}' for terminal {terminal_id}")]
    CommandNotFound { terminal_id: u32, command: String },

    #[error("could not determine default editor")]
    NoEditorFound,

    #[error("failed to allocate another terminal id")]
    NoMoreTerminalIds,

    #[error("failed to start PTY")]
    FailedToStartPty,

    #[error(
        "This version of zellij was built to load the core plugins from
the globally configured plugin directory. However, a plugin wasn't found:

Plugin name: '{plugin_path}'
Plugin directory: '{plugin_dir}'

If you're a user:
    Please report this error to the distributor of your current zellij version

If you're a developer:
Either make sure to include the plugins with the application (See feature
'disable_automatic_asset_installation'), or make them available in the
plugin directory.

Possible fix for your problem:
    Place the builtin plugin '.wasm' files in the plugin directory shown above,
or in the 'plugins' folder of the system data directory. Both are visible in
the output of `zellij setup --check`. This build carries no bundled plugins,
so `zellij setup --dump-plugins` cannot provide them.
"
    )]
    BuiltinPluginMissing {
        plugin_path: PathBuf,
        plugin_dir: PathBuf,
        #[source]
        source: anyhow::Error,
    },

    #[error(
        "It seems you tried to load the following builtin plugin:

    Plugin name: '{plugin_path}'

This is not a builtin plugin known to this version of zellij. If you were using
a custom layout, please refer to the layout documentation at:

    https://zellij.dev/documentation/creating-a-layout.html#plugin

If you think this is a bug and the plugin is indeed an internal plugin, please
open an issue on GitHub:

    https://github.com/zellij-org/zellij/issues
"
    )]
    BuiltinPluginNonexistent {
        plugin_path: PathBuf,
        #[source]
        source: anyhow::Error,
    },

    // 这是一个临时的 hack，直到我们能够从各个 crate 内部合并自定义错误，
    // 而不必将它们的有效载荷类型移到这里
    #[error("Cannot resize fixed panes")]
    CantResizeFixedPanes { pane_ids: Vec<(u32, bool)> }, // bool: 0 => terminal_pane, 1 =>
    // plugin_pane
    #[error("Pane size remains unchanged")]
    PaneSizeUnchanged,

    #[error("an error occurred")]
    GenericError { source: anyhow::Error },

    #[error("Client {client_id} is too slow to handle incoming messages")]
    ClientTooSlow { client_id: u16 },

    #[error("The plugin does not exist")]
    PluginDoesNotExist,

    #[error("Ran out of room for spans")]
    RanOutOfRoomForSpans,
}

#[cfg(not(target_family = "wasm"))]
pub use not_wasm::*;

#[cfg(not(target_family = "wasm"))]
mod not_wasm {
    use super::*;
    use crate::channels::{SenderWithContext, ASYNCOPENCALLS, OPENCALLS};
    use miette::{Diagnostic, GraphicalReportHandler, GraphicalTheme, Report};
    use std::panic::PanicHookInfo;
    use thiserror::Error as ThisError;

    /// [`ErrorContext`] 在其栈表示中跟踪的最大调用数量。这是每个线程的最大值。
    const MAX_THREAD_CALL_STACK: usize = 6;

    #[derive(Debug, ThisError, Diagnostic)]
    #[error("{0}{backtrace}", backtrace = self.show_backtrace())]
    #[diagnostic(help("{}", self.show_help()))]
    struct Panic(String);

    impl Panic {
        // 我们已经在后台使用 `backtrace` crate 通过 `anyhow` 捕获了回溯。
        // 优点是这是真正错误来源的回溯（即我们第一次遇到错误并将其转换为 `anyhow::Error` 的地方），
        // 而这里记录的回溯是导致调用任何 `panic` 函数的回溯。由于现在我们在 `unwrap` 之前向上传播错误
        // （例如在 `zellij_server::screen::screen_thread_main` 中），前者才是我们真正想要诊断的。
        // 我们仍然保留第二个，以防第一个回溯没有意义或根本不存在（这真的不应该发生，但你永远不知道）。
        fn show_backtrace(&self) -> String {
            if let Ok(var) = std::env::var("RUST_BACKTRACE") {
                if !var.is_empty() && var != "0" {
                    return format!("\n\nPanic backtrace:\n{:?}", backtrace::Backtrace::new());
                }
            }
            "".into()
        }

        fn show_help(&self) -> String {
            format!(
                "If you are seeing this message, it means that something went wrong.

-> To get additional information, check the log at: {}
-> To see a backtrace next time, reproduce the error with: RUST_BACKTRACE=1 zellij [...]
-> To help us fix this, please open an issue: https://github.com/zellij-org/zellij/issues

",
                crate::consts::ZELLIJ_TMP_LOG_FILE.display().to_string()
            )
        }
    }

    /// 自定义 panic 处理器/钩子。打印 [`ErrorContext`]。
    pub fn handle_panic<T>(info: &PanicHookInfo<'_>, sender: Option<&SenderWithContext<T>>)
    where
        T: ErrorInstruction + Clone,
    {
        use std::{process, thread};
        let thread = thread::current();
        let thread = thread.name().unwrap_or("unnamed");

        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => Some(*s),
            None => info.payload().downcast_ref::<String>().map(|s| &**s),
        }
        .unwrap_or("An unexpected error occurred!");

        let err_ctx = OPENCALLS.with(|ctx| *ctx.borrow());

        let mut report: Report = Panic(format!("\u{1b}[0;31m{}\u{1b}[0;0m", msg)).into();

        let mut location_string = String::new();
        if let Some(location) = info.location() {
            location_string = format!(
                "At {}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            );
            report = report.wrap_err(location_string.clone());
        }

        if !err_ctx.is_empty() {
            report = report.wrap_err(format!("{}", err_ctx));
        }

        report = report.wrap_err(format!(
            "Thread '\u{1b}[0;31m{}\u{1b}[0;0m' panicked.",
            thread
        ));

        error!(
            "{}",
            format!(
                "Panic occurred:
             thread: {}
             location: {}
             message: {}",
                thread, location_string, msg
            )
        );

        if thread == "main" || sender.is_none() {
            // 这里我们只显示第一行，因为否则回溯不可读
            // 更好的解决方案是在这之前转义原始模式，但在这里获取 os_input 并不简单
            println!("\u{1b}[2J{}", fmt_report(report));
            process::exit(1);
        } else {
            let _ = sender.unwrap().send(T::error(fmt_report(report)));
        }
    }

    pub fn get_current_ctx() -> ErrorContext {
        ASYNCOPENCALLS
            .try_with(|ctx| *ctx.borrow())
            .unwrap_or_else(|_| OPENCALLS.with(|ctx| *ctx.borrow()))
    }

    fn fmt_report(diag: Report) -> String {
        let mut out = String::new();
        GraphicalReportHandler::new_themed(GraphicalTheme::unicode())
            .render_report(&mut out, diag.as_ref())
            .unwrap();
        out
    }

    /// 调用栈的表示。
    #[derive(Clone, Copy, Serialize, Deserialize, Debug)]
    pub struct ErrorContext {
        calls: [ContextType; MAX_THREAD_CALL_STACK],
    }

    impl ErrorContext {
        /// 返回一个新的、空白的 [`ErrorContext`]，仅包含 [`Empty`](ContextType::Empty) 调用。
        pub fn new() -> Self {
            Self {
                calls: [ContextType::Empty; MAX_THREAD_CALL_STACK],
            }
        }

        /// 如果所有调用都是 [`Empty`](ContextType::Empty) 调用，则返回 `true`。
        pub fn is_empty(&self) -> bool {
            self.calls.iter().all(|c| c == &ContextType::Empty)
        }

        /// 向此 [`ErrorContext`] 的调用栈表示中添加一个调用。
        pub fn add_call(&mut self, call: ContextType) {
            for ctx in &mut self.calls {
                if let ContextType::Empty = ctx {
                    *ctx = call;
                    break;
                }
            }
            self.update_thread_ctx()
        }

        /// 更新线程局部的 [`ErrorContext`]。
        pub fn update_thread_ctx(&self) {
            ASYNCOPENCALLS
                .try_with(|ctx| *ctx.borrow_mut() = *self)
                .unwrap_or_else(|_| OPENCALLS.with(|ctx| *ctx.borrow_mut() = *self));
        }
    }

    impl Default for ErrorContext {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Display for ErrorContext {
        fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
            writeln!(f, "Originating Thread(s)")?;
            for (index, ctx) in self.calls.iter().enumerate() {
                if *ctx == ContextType::Empty {
                    break;
                }
                writeln!(f, "\t\u{1b}[0;0m{}. {}", index + 1, ctx)?;
            }
            Ok(())
        }
    }

    /// 用于将不满足 `anyhow` trait 要求的错误类型转换为 anyhow 错误的辅助 trait。
    pub trait ToAnyhow<U> {
        fn to_anyhow(self) -> anyhow::Result<U>;
    }

    /// `SendError` 不满足 `anyhow` 的 trait 要求，因为 `T` 可能是 `PluginInstruction` 类型，
    /// 它包装了一个 `mpsc::Send` 且不是 `Sync`。因此，整个错误类型不是 `Sync`，
    /// 无法与 `anyhow`（或几乎任何其他错误处理 crate）一起使用。
    ///
    /// 接受 `SendError` 并创建一个带有已发送消息（格式化为字符串）的 `anyhow` 错误类型，
    /// 将 [`ErrorContext`] 作为 anyhow 上下文附加到它上面。
    impl<T: std::fmt::Debug, U> ToAnyhow<U>
        for Result<U, crate::channels::SendError<(T, ErrorContext)>>
    {
        fn to_anyhow(self) -> anyhow::Result<U> {
            match self {
                Ok(val) => anyhow::Ok(val),
                Err(e) => {
                    let (msg, context) = e.into_inner();
                    if *crate::consts::DEBUG_MODE.get().unwrap_or(&true) {
                        Err(anyhow::anyhow!(
                            "发送消息到通道失败：{:#?}",
                            msg
                        ))
                        .with_context(|| context.to_string())
                    } else {
                        Err(anyhow::anyhow!("发送消息到通道失败"))
                            .with_context(|| context.to_string())
                    }
                },
            }
        }
    }

    impl<U> ToAnyhow<U> for Result<U, std::sync::PoisonError<U>> {
        fn to_anyhow(self) -> anyhow::Result<U> {
            match self {
                Ok(val) => anyhow::Ok(val),
                Err(e) => {
                    if *crate::consts::DEBUG_MODE.get().unwrap_or(&true) {
                        Err(anyhow::anyhow!("无法获取已污染的锁 {e:#?}"))
                    } else {
                        Err(anyhow::anyhow!("无法获取已污染的锁"))
                    }
                },
            }
        }
    }
}
