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
            ActionType::SessionManager => "会话管理器".to_string(),
            ActionType::PluginManager => "插件管理器".to_string(),
            ActionType::Configuration => "配置".to_string(),
            ActionType::About => "关于 Zellij".to_string(),
            ActionType::SwitchToMode(input_mode) if input_mode == &InputMode::RenamePane => {
                "重命名窗格".to_string()
            },
            ActionType::SwitchToMode(input_mode) if input_mode == &InputMode::RenameTab => {
                "重命名标签页".to_string()
            },
            ActionType::SwitchToMode(input_mode) if input_mode == &InputMode::EnterSearch => {
                "搜索".to_string()
            },
            ActionType::SwitchToMode(input_mode) if input_mode == &InputMode::Locked => {
                "锁定".to_string()
            },
            ActionType::SwitchToMode(input_mode) if input_mode == &InputMode::Normal => {
                "解锁".to_string()
            },
            ActionType::SwitchToMode(input_mode) => format!("{:?}", input_mode),
            ActionType::TogglePaneEmbedOrFloating => "浮动或嵌入".to_string(),
            ActionType::NewStackedPane => "新建堆叠窗格".to_string(),
            ActionType::ToggleFocusFullscreen => "切换全屏".to_string(),
            ActionType::ToggleFloatingPanes => "显示/隐藏浮动窗格".to_string(),
            ActionType::CloseFocus => "关闭窗格".to_string(),
            ActionType::CloseTab => "关闭标签页".to_string(),
            ActionType::ToggleActiveSyncTab => "同步标签页中的窗格".to_string(),
            ActionType::ToggleTab => "循环切换标签页焦点".to_string(),
            ActionType::BreakPane => "将窗格移到新标签页".to_string(),
            ActionType::EditScrollback => "在编辑器中打开窗格回滚缓冲区".to_string(),
            ActionType::NewTab => "新建标签页".to_string(),
            ActionType::Detach => "分离".to_string(),
            ActionType::Quit => "退出".to_string(),
            ActionType::Other(_) => "其他操作".to_string(),
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
