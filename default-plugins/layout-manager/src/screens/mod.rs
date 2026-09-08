mod import_layout;
mod layout_list;
mod new_layout_from_session;
mod rename_layout;

use zellij_tile::prelude::LayoutMetadata;

pub use import_layout::ImportLayoutScreen;
pub use layout_list::LayoutListScreen;
pub use new_layout_from_session::NewLayoutFromCurrentSessionScreen;
pub use rename_layout::RenameLayoutScreen;

// 为方便起见，从 errors 模块重新导出错误类型
pub use crate::errors::{ErrorDetailScreen, ErrorScreen};

#[derive(Clone)]
pub enum Screen {
    LayoutList(LayoutListScreen),
    NewLayoutFromSession(NewLayoutFromCurrentSessionScreen),
    ImportLayout(ImportLayoutScreen),
    RenameLayout(RenameLayoutScreen),
    Error(ErrorScreen),
    ErrorDetail(ErrorDetailScreen),
}

impl Default for Screen {
    fn default() -> Self {
        Screen::LayoutList(LayoutListScreen::default())
    }
}

///在 Zellij 确认之前应用的乐观状态更新
#[derive(Clone, Debug)]
pub enum OptimisticUpdate {
    Delete(String), // file_name
    Rename {
        old_name: String,
        new_name: String,
    },
    Add {
        name: String,
        metadata: LayoutMetadata,
    },
}

///  来自屏幕按键处理器的响应
#[derive(Default)]
pub struct KeyResponse {
    pub should_render: bool,
    pub new_screen: Option<Screen>,
    pub optimistic_update: Option<OptimisticUpdate>,
}

impl KeyResponse {
    pub fn render() -> Self {
        KeyResponse {
            should_render: true,
            new_screen: None,
            optimistic_update: None,
        }
    }

    pub fn new_screen(screen: Screen) -> Self {
        KeyResponse {
            should_render: true,
            new_screen: Some(screen),
            optimistic_update: None,
        }
    }

    pub fn with_optimistic(mut self, update: OptimisticUpdate) -> Self {
        self.optimistic_update = Some(update);
        self
    }

    pub fn none() -> Self {
        KeyResponse::default()
    }
}
