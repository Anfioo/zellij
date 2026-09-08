use zellij_tile::prelude::actions::Action;
use zellij_tile::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ActionType {
    MoveFocus,
    MovePaneWithDirection,
    MovePaneWithoutDirection,
    ResizeIncrease,
    ResizeDecrease,
    ResizeAny,
    Search,
    NewPaneWithDirection,
    NewPaneWithoutDirection,
    BreakPaneLeftOrRight,
    GoToAdjacentTab,
    Scroll,
    PageScroll,
    HalfPageScroll,
    SessionManager,
    Configuration,
    PluginManager,
    About,
    SwitchToMode(InputMode),
    TogglePaneEmbedOrFloating,
    ToggleFocusFullscreen,
    ToggleFloatingPanes,
    CloseFocus,
    CloseTab,
    ToggleActiveSyncTab,
    ToggleTab,
    BreakPane,
    EditScrollback,
    NewTab,
    Detach,
    Quit,
    NewStackedPane,
    Other(String), //  未处理操作的回退
}

impl ActionType {
    pub fn description(&self) -> String {
        match self {
            ActionType::MoveFocus => "移动焦点".to_string(),
            ActionType::MovePaneWithDirection => "移动窗格".to_string(),
            ActionType::MovePaneWithoutDirection => "移动窗格".to_string(),
            ActionType::ResizeIncrease => "按方向增大尺寸".to_string(),
            ActionType::ResizeDecrease => "按方向减小尺寸".to_string(),
            ActionType::ResizeAny => "增大或减小尺寸".to_string(),
            ActionType::Search => "搜索".to_string(),
            ActionType::NewPaneWithDirection => "向右/向下拆分".to_string(),
            ActionType::NewPaneWithoutDirection => "新建窗格".to_string(),
            ActionType::BreakPaneLeftOrRight => "将窗格移动到相邻标签页".to_string(),
            ActionType::GoToAdjacentTab => "移动标签页焦点".to_string(),
            ActionType::Scroll => "滚动".to_string(),
            ActionType::PageScroll => "整页滚动".to_string(),
            ActionType::HalfPageScroll => "半页滚动".to_string(),
            ActionType::SessionManager => "Session manager".to_string(),
            ActionType::PluginManager => "Plugin manager".to_string(),
            ActionType::Configuration => "Configuration".to_string(),
            ActionType::About => "About Zellij".to_string(),
            ActionType::SwitchToMode(input_mode) if input_mode == &InputMode::RenamePane => {
                "Rename pane".to_string()
            },
            ActionType::SwitchToMode(input_mode) if input_mode == &InputMode::RenameTab => {
                "Rename tab".to_string()
            },
            ActionType::SwitchToMode(input_mode) if input_mode == &InputMode::EnterSearch => {
                "搜索".to_string()
            },
            ActionType::SwitchToMode(input_mode) if input_mode == &InputMode::Locked => {
                "Lock".to_string()
            },
            ActionType::SwitchToMode(input_mode) if input_mode == &InputMode::Normal => {
                "Unlock".to_string()
            },
            ActionType::SwitchToMode(input_mode) => format!("{:?}", input_mode),
            ActionType::TogglePaneEmbedOrFloating => "Float or embed".to_string(),
            ActionType::NewStackedPane => "New stacked pane".to_string(),
            ActionType::ToggleFocusFullscreen => "Toggle fullscreen".to_string(),
            ActionType::ToggleFloatingPanes => "Show/hide floating panes".to_string(),
            ActionType::CloseFocus => "Close pane".to_string(),
            ActionType::CloseTab => "Close tab".to_string(),
            ActionType::ToggleActiveSyncTab => "Sync panes in tab".to_string(),
            ActionType::ToggleTab => "Circle tab focus".to_string(),
            ActionType::BreakPane => "Break pane to new tab".to_string(),
            ActionType::EditScrollback => "Open pane scrollback in editor".to_string(),
            ActionType::NewTab => "New tab".to_string(),
            ActionType::Detach => "Detach".to_string(),
            ActionType::Quit => "Quit".to_string(),
            ActionType::Other(_) => "Other action".to_string(),
        }
    }

    pub fn from_action(action: &Action) -> Self {
        match action {
            Action::MoveFocus { .. } => ActionType::MoveFocus,
            Action::MovePane { direction: Some(_) } => ActionType::MovePaneWithDirection,
            Action::MovePane { direction: None } => ActionType::MovePaneWithoutDirection,
            Action::Resize {
                resize: Resize::Increase,
                direction: Some(_),
            } => ActionType::ResizeIncrease,
            Action::Resize {
                resize: Resize::Decrease,
                direction: Some(_),
            } => ActionType::ResizeDecrease,
            Action::Resize {
                resize: _,
                direction: None,
            } => ActionType::ResizeAny,
            Action::Search { .. } => ActionType::Search,
            Action::NewPane {
                direction: Some(_), ..
            } => ActionType::NewPaneWithDirection,
            Action::NewPane {
                direction: None, ..
            } => ActionType::NewPaneWithoutDirection,
            Action::NewStackedPane { .. } => ActionType::NewStackedPane,
            Action::BreakPaneLeft | Action::BreakPaneRight => ActionType::BreakPaneLeftOrRight,
            Action::GoToPreviousTab | Action::GoToNextTab => ActionType::GoToAdjacentTab,
            Action::ScrollUp | Action::ScrollDown => ActionType::Scroll,
            Action::PageScrollUp | Action::PageScrollDown => ActionType::PageScroll,
            Action::HalfPageScrollUp | Action::HalfPageScrollDown => ActionType::HalfPageScroll,
            Action::SwitchToMode { input_mode } => ActionType::SwitchToMode(*input_mode),
            Action::TogglePaneEmbedOrFloating => ActionType::TogglePaneEmbedOrFloating,
            Action::ToggleFocusFullscreen => ActionType::ToggleFocusFullscreen,
            Action::ToggleFloatingPanes => ActionType::ToggleFloatingPanes,
            Action::CloseFocus => ActionType::CloseFocus,
            Action::CloseTab => ActionType::CloseTab,
            Action::ToggleActiveSyncTab => ActionType::ToggleActiveSyncTab,
            Action::ToggleTab => ActionType::ToggleTab,
            Action::BreakPane => ActionType::BreakPane,
            Action::EditScrollback { .. } => ActionType::EditScrollback,
            Action::Detach => ActionType::Detach,
            Action::Quit => ActionType::Quit,
            action if action.launches_plugin("session-manager") => ActionType::SessionManager,
            action if action.launches_plugin("configuration") => ActionType::Configuration,
            action if action.launches_plugin("plugin-manager") => ActionType::PluginManager,
            action if action.launches_plugin("zellij:about") => ActionType::About,
            action if matches!(action, Action::NewTab { .. }) => ActionType::NewTab,
            _ => ActionType::Other(format!("{:?}", action)),
        }
    }
}
