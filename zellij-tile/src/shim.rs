use serde::{de::DeserializeOwned, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::{
    io,
    path::{Path, PathBuf},
};
use zellij_utils::data::*;
use zellij_utils::errors::prelude::*;
use zellij_utils::input::actions::Action;
pub use zellij_utils::plugin_api;
use zellij_utils::plugin_api::event::ProtobufPaneScrollbackResponse;
use zellij_utils::plugin_api::generated_api::api::plugin_command::{
    hide_floating_panes_response, save_session_response, show_floating_panes_response,
};
use zellij_utils::plugin_api::plugin_command::{
    dump_layout_response, dump_session_layout_response, get_focused_pane_info_response,
    get_pane_cwd_response, get_pane_running_command_response, get_session_list_response,
    parse_layout_response, CreateTokenResponse, ListTokensResponse,
    ProtobufBreakPanesToNewTabResponse, ProtobufBreakPanesToTabWithIdResponse,
    ProtobufBreakPanesToTabWithIndexResponse, ProtobufCurrentSessionLastSavedTimeResponse,
    ProtobufDeleteAllDeadSessionsResponse, ProtobufDeleteDeadSessionResponse,
    ProtobufDeleteLayoutResponse, ProtobufDumpLayoutResponse, ProtobufDumpSessionLayoutResponse,
    ProtobufEditLayoutResponse, ProtobufFocusOrCreateTabResponse,
    ProtobufGenerateRandomNameResponse, ProtobufGetFocusedPaneInfoResponse,
    ProtobufGetLayoutDirResponse, ProtobufGetPaneCwdResponse, ProtobufGetPaneInfoResponse,
    ProtobufGetPanePidResponse, ProtobufGetPaneRunningCommandResponse,
    ProtobufGetSessionEnvironmentVariablesResponse, ProtobufGetSessionListResponse,
    ProtobufGetTabInfoResponse, ProtobufHideFloatingPanesResponse, ProtobufKillSessionsResponse,
    ProtobufNewTabResponse, ProtobufNewTabUnfocusedResponse, ProtobufNewTabsResponse,
    ProtobufNewTiledPaneInTabResponse, ProtobufOpenCommandPaneBackgroundResponse,
    ProtobufOpenCommandPaneFloatingNearPluginResponse, ProtobufOpenCommandPaneFloatingResponse,
    ProtobufOpenCommandPaneInPlaceOfPaneIdResponse, ProtobufOpenCommandPaneInPlaceOfPluginResponse,
    ProtobufOpenCommandPaneInPlaceResponse, ProtobufOpenCommandPaneNearPluginResponse,
    ProtobufOpenCommandPaneResponse, ProtobufOpenEditPaneInPlaceOfPaneIdResponse,
    ProtobufOpenFileFloatingNearPluginResponse, ProtobufOpenFileFloatingResponse,
    ProtobufOpenFileInPlaceOfPluginResponse, ProtobufOpenFileInPlaceResponse,
    ProtobufOpenFileNearPluginResponse, ProtobufOpenFileResponse, ProtobufOpenPaneInNewTabResponse,
    ProtobufOpenPluginPaneFloatingResponse, ProtobufOpenTerminalFloatingNearPluginResponse,
    ProtobufOpenTerminalFloatingResponse, ProtobufOpenTerminalInPlaceOfPluginResponse,
    ProtobufOpenTerminalInPlaceResponse, ProtobufOpenTerminalNearPluginResponse,
    ProtobufOpenTerminalPaneInPlaceOfPaneIdResponse, ProtobufOpenTerminalResponse,
    ProtobufParseLayoutResponse, ProtobufPluginCommand, ProtobufRenameLayoutResponse,
    ProtobufSaveLayoutResponse, ProtobufSaveSessionResponse, ProtobufShowFloatingPanesResponse,
    RenameWebTokenResponse, RevokeAllWebTokensResponse, RevokeTokenResponse,
};
use zellij_utils::plugin_api::plugin_ids::{ProtobufPluginIds, ProtobufZellijVersion};

pub use super::ui_components::*;
pub use prost::{self, *};

// 订阅处理

/// 订阅由 [`EventType`] 表示的 [`事件`](Event) 列表，这些事件随后将触发 `update` 方法
pub fn subscribe(event_types: &[EventType]) {
    let event_types: HashSet<EventType> = event_types.iter().cloned().collect();
    let plugin_command = PluginCommand::Subscribe(event_types);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 取消订阅由 [`EventType`] 表示的 [`事件`](Event) 列表。
pub fn unsubscribe(event_types: &[EventType]) {
    let event_types: HashSet<EventType> = event_types.iter().cloned().collect();
    let plugin_command = PluginCommand::Unsubscribe(event_types);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

// 插件设置

/// 将插件设置为用户可选或不可选。当插件不接受用户输入时，可能需要将其设为不可选。
pub fn set_selectable(selectable: bool) {
    let plugin_command = PluginCommand::SetSelectable(selectable);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 在指定坐标处显示光标或隐藏光标
///
/// # 参数
/// * `cursor_position` - None 表示隐藏光标，Some((x, y)) 表示在指定坐标显示
pub fn show_cursor(cursor_position: Option<(usize, usize)>) {
    let plugin_command = PluginCommand::ShowCursor(cursor_position);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

pub fn request_permission(permissions: &[PermissionType]) {
    let plugin_command = PluginCommand::RequestPluginPermissions(permissions.into());
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

// 查询函数
/// 返回插件的唯一 Zellij 窗格 ID 以及 Zellij 进程 ID。
pub fn get_plugin_ids() -> PluginIds {
    let plugin_command = PluginCommand::GetPluginIds;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
    let protobuf_plugin_ids =
        ProtobufPluginIds::decode(bytes_from_stdin().unwrap().as_slice()).unwrap();
    PluginIds::try_from(protobuf_plugin_ids).unwrap()
}

/// 返回正在运行的 Zellij 实例的版本——可用于检查插件兼容性
pub fn get_zellij_version() -> String {
    let plugin_command = PluginCommand::GetZellijVersion;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
    let protobuf_zellij_version =
        ProtobufZellijVersion::decode(bytes_from_stdin().unwrap().as_slice()).unwrap();
    protobuf_zellij_version.version
}

/// 使用 Zellij 精选的词表生成一个随机的、人类可读的名称。
/// 返回格式为 AdjectiveNoun 的名称（例如 "BraveRustacean"、"ZippyWeasel"）。
///
/// 它使用与会话名称生成相同的词表，提供
/// 约 4,096 种唯一组合。
pub fn generate_random_name() -> String {
    let plugin_command = PluginCommand::GenerateRandomName;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
    let response =
        ProtobufGenerateRandomNameResponse::decode(bytes_from_stdin().unwrap().as_slice()).unwrap();
    response.name
}

/// 按名称导出布局并以字符串形式返回其 KDL 内容
///
/// 支持内置布局（例如 "default"、"compact"、"welcome"）
/// 以及插件布局目录中的自定义布局。
pub fn dump_layout(layout_name: &str) -> Result<String, String> {
    // 使用布局名称创建插件命令
    let plugin_command = PluginCommand::DumpLayout(layout_name.to_string());

    // 转换为 protobuf 并编码
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());

    // 调用宿主函数（阻塞直到收到响应）
    unsafe { host_run_plugin_command() };

    // 读取并解码响应
    let response_bytes =
        bytes_from_stdin().map_err(|e| format!("从标准输入读取响应失败：{:?}", e))?;

    let protobuf_response = ProtobufDumpLayoutResponse::decode(response_bytes.as_slice())
        .map_err(|e| format!("解码 protobuf 响应失败：{}", e))?;

    // 从 oneof 字段中提取结果
    match protobuf_response.result {
        Some(dump_layout_response::Result::LayoutContent(content)) => Ok(content),
        Some(dump_layout_response::Result::Error(error)) => Err(error),
        None => Err("服务器返回了空响应".to_string()),
    }
}

/// 返回布局目录的路径。
///
/// 这是 Zellij 查找布局文件的目录。它可以是：
/// - 通过 CLI `--layout-dir` 标志指定的目录
/// - 配置文件中指定的目录
/// - 通过 ZELLIJ_LAYOUT_DIR 环境变量指定的目录
/// - 默认值：`~/.config/zellij/layouts`
///
/// # 返回值
/// 包含布局目录绝对路径的字符串。
/// 如果无法确定布局目录则返回空字符串（罕见的边界情况）。
pub fn get_layout_dir() -> String {
    let plugin_command = PluginCommand::GetLayoutDir;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
    let response =
        ProtobufGetLayoutDirResponse::decode(bytes_from_stdin().unwrap().as_slice()).unwrap();
    response.layout_dir
}

pub fn get_session_environment_variables() -> BTreeMap<String, String> {
    let plugin_command = PluginCommand::GetSessionEnvironmentVariables;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();

    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response = ProtobufGetSessionEnvironmentVariablesResponse::decode(
        bytes_from_stdin().unwrap().as_slice(),
    )
    .unwrap();

    response
        .env_vars
        .into_iter()
        .map(|env_var| (env_var.name, env_var.value))
        .collect()
}

/// 返回与此插件关联的客户端的焦点窗格 ID 和标签页索引。
pub fn get_focused_pane_info() -> Result<(usize, PaneId), String> {
    let plugin_command = PluginCommand::GetFocusedPaneInfo;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let protobuf_response =
        ProtobufGetFocusedPaneInfoResponse::decode(bytes_from_stdin().unwrap().as_slice()).unwrap();

    match protobuf_response.result {
        Some(get_focused_pane_info_response::Result::FocusedPaneInfo(info)) => {
            let tab_index = info.focused_tab_index as usize;
            match info.focused_pane_id {
                Some(pb_pane_id) => match pb_pane_id.try_into() {
                    Ok(pane_id) => Ok((tab_index, pane_id)),
                    Err(_) => Err("响应中的 pane_id 无效".to_string()),
                },
                None => Err("响应中缺少 pane_id".to_string()),
            }
        },
        Some(get_focused_pane_info_response::Result::Error(err)) => Err(err),
        None => Err("主机返回了空响应".to_string()),
    }
}

/// 通过 PaneId 查询特定窗格的信息。
///
/// 这会同步向 Zellij 查询具有给定 ID 的窗格的详细信息，
/// 包括其位置、大小、状态和其他元数据。
///
/// # 参数
///
/// - `pane_id`：要查询的窗格 ID
///
/// # Returns
///
/// - 如果窗格存在且信息成功获取，返回 `Some(PaneInfo)`
/// - 如果窗格不存在或无法找到，返回 `None`
///
/// # 示例
///
/// ```no_run
/// use zellij_tile::prelude::*;
///
/// // 查询特定窗格的信息
/// let pane_id = PaneId::Terminal(1);
/// match get_pane_info(pane_id) {
///     Some(info) => {
///         println!("Pane title: {}", info.title);
///         println!("Pane is focused: {}", info.is_focused);
///         println!("Pane position: ({}, {})", info.pane_x, info.pane_y);
///     },
///     None => println!("Pane not found"),
/// }
/// ```
pub fn get_pane_info(pane_id: PaneId) -> Option<PaneInfo> {
    let plugin_command = PluginCommand::GetPaneInfo(pane_id);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let protobuf_response =
        ProtobufGetPaneInfoResponse::decode(bytes_from_stdin().unwrap().as_slice()).unwrap();

    protobuf_response
        .pane_info
        .and_then(|pb_pane_info| pb_pane_info.try_into().ok())
}

/// 通过标签页 ID 查询特定标签页的信息。
///
/// 这会同步向 Zellij 查询具有给定 ID 的标签页的详细信息，
/// 包括其名称、位置、活动状态、窗格数量和其他元数据。
///
/// # Parameters
///
/// - `tab_id`：要查询的标签页的稳定 ID
///
/// # Returns
///
/// - 如果标签页存在且信息成功获取，返回 `Some(TabInfo)`
/// - 如果标签页不存在或无法找到，返回 `None`
///
/// # Example
///
/// ```no_run
/// use zellij_tile::prelude::*;
///
/// // 查询特定标签页的信息
/// let tab_id = 3;
/// match get_tab_info(tab_id) {
///     Some(info) => {
///         println!("Tab name: {}", info.name);
///         println!("Tab position: {}", info.position);
///         println!("Tab is active: {}", info.active);
///         println!("Tiled panes: {}", info.selectable_tiled_panes_count);
///     },
///     None => println!("Tab not found"),
/// }
/// ```
pub fn get_tab_info(tab_id: usize) -> Option<TabInfo> {
    let plugin_command = PluginCommand::GetTabInfo(tab_id);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let protobuf_response =
        ProtobufGetTabInfoResponse::decode(bytes_from_stdin().unwrap().as_slice()).unwrap();

    protobuf_response
        .tab_info
        .and_then(|pb_tab_info| pb_tab_info.try_into().ok())
}

/// 立即将当前会话状态保存到磁盘。
///
/// 这会触发立即将当前会话元数据和布局写入
/// 到会话缓存目录（~/.cache/zellij/contract_version_1/session_info/<session_name>/）。
///
/// # Returns
///
/// - 如果保存请求成功发送，返回 `Ok(())`
/// - 如果发送请求时出错，返回 `Err(String)`
///
/// # Example
///
/// ```no_run
/// use zellij_tile::prelude::*;
///
/// // 保存当前会话
/// match save_session() {
///     Ok(()) => println!("Session saved successfully"),
///     Err(e) => eprintln!("Failed to save session: {}", e),
/// }
/// ```
pub fn save_session() -> Result<(), String> {
    let plugin_command = PluginCommand::SaveSession;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());

    unsafe { host_run_plugin_command() };

    let response_bytes =
        bytes_from_stdin().map_err(|e| format!("读取响应失败：{:?}", e))?;
    let protobuf_response = ProtobufSaveSessionResponse::decode(response_bytes.as_slice())
        .map_err(|e| format!("解码响应失败：{}", e))?;

    match protobuf_response.result {
        Some(save_session_response::Result::Success(_)) => Ok(()),
        Some(save_session_response::Result::Error(error)) => Err(error),
        None => Err("服务器返回了空响应".to_string()),
    }
}

/// 返回自当前会话状态上次保存到磁盘以来经过的时间（毫秒）。
///
/// 如果会话在此期间从未保存过，返回 `None`。
/// 返回值是自上次保存以来经过的毫秒数，而非 Unix 时间戳。
///
/// # Example
///
/// ```no_run
/// use zellij_tile::prelude::*;
///
/// if let Some(elapsed_millis) = current_session_last_saved_time() {
///     println!("Session was last saved {} ms ago", elapsed_millis);
/// } else {
///     println!("Session has not been saved yet");
/// }
/// ```
pub fn current_session_last_saved_time() -> Option<u64> {
    let plugin_command = PluginCommand::CurrentSessionLastSavedTime;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let protobuf_response =
        ProtobufCurrentSessionLastSavedTimeResponse::decode(bytes_from_stdin().unwrap().as_slice())
            .unwrap();

    protobuf_response.timestamp_millis
}

// 宿主函数

/// 在用户默认的 `$EDITOR` 中打开文件，位于新窗格中
pub fn open_file(file_to_open: FileToOpen, context: BTreeMap<String, String>) -> Option<PaneId> {
    let plugin_command = PluginCommand::OpenFile(file_to_open, context);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response = match bytes_from_stdin() {
        Ok(bytes_from_stdin) => ProtobufOpenFileResponse::decode(bytes_from_stdin.as_slice()).ok(),
        Err(e) => {
            eprintln!("{}", e);
            None
        },
    };
    response.and_then(|r| OpenFileResponse::try_from(r).ok().flatten())
}

/// 在用户默认的 `$EDITOR` 中打开文件，位于新浮动窗格中
pub fn open_file_floating(
    file_to_open: FileToOpen,
    coordinates: Option<FloatingPaneCoordinates>,
    context: BTreeMap<String, String>,
) -> Option<PaneId> {
    let plugin_command = PluginCommand::OpenFileFloating(file_to_open, coordinates, context);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response =
        ProtobufOpenFileFloatingResponse::decode(bytes_from_stdin().unwrap().as_slice()).unwrap();
    OpenFileFloatingResponse::try_from(response).unwrap()
}

/// 在用户默认的 `$EDITOR` 中打开文件，替换焦点窗格
pub fn open_file_in_place(
    file_to_open: FileToOpen,
    context: BTreeMap<String, String>,
) -> Option<PaneId> {
    let plugin_command = PluginCommand::OpenFileInPlace(file_to_open, context);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response =
        ProtobufOpenFileInPlaceResponse::decode(bytes_from_stdin().unwrap().as_slice()).unwrap();
    OpenFileInPlaceResponse::try_from(response).unwrap()
}

/// 在用户默认的 `$EDITOR` 中打开文件，位于插件附近的新窗格中
pub fn open_file_near_plugin(
    file_to_open: FileToOpen,
    context: BTreeMap<String, String>,
) -> Option<PaneId> {
    let plugin_command = PluginCommand::OpenFileNearPlugin(file_to_open, context);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response =
        ProtobufOpenFileNearPluginResponse::decode(bytes_from_stdin().unwrap().as_slice()).unwrap();
    OpenFileNearPluginResponse::try_from(response).unwrap()
}

/// 在用户默认的 `$EDITOR` 中打开文件，位于插件附近的新浮动窗格中
pub fn open_file_floating_near_plugin(
    file_to_open: FileToOpen,
    coordinates: Option<FloatingPaneCoordinates>,
    context: BTreeMap<String, String>,
) -> Option<PaneId> {
    let plugin_command =
        PluginCommand::OpenFileFloatingNearPlugin(file_to_open, coordinates, context);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response =
        ProtobufOpenFileFloatingNearPluginResponse::decode(bytes_from_stdin().unwrap().as_slice())
            .unwrap();
    OpenFileFloatingNearPluginResponse::try_from(response).unwrap()
}

/// 在用户默认的 `$EDITOR` 中打开文件，替换插件窗格
pub fn open_file_in_place_of_plugin(
    file_to_open: FileToOpen,
    close_plugin_after_replace: bool,
    context: BTreeMap<String, String>,
) -> Option<PaneId> {
    let plugin_command =
        PluginCommand::OpenFileInPlaceOfPlugin(file_to_open, close_plugin_after_replace, context);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response =
        ProtobufOpenFileInPlaceOfPluginResponse::decode(bytes_from_stdin().unwrap().as_slice())
            .unwrap();
    OpenFileInPlaceOfPluginResponse::try_from(response).unwrap()
}
/// 在宿主文件系统的指定位置打开新的终端窗格
pub fn open_terminal<P: AsRef<Path>>(path: P) -> Option<PaneId> {
    let file_to_open = FileToOpen::new(path.as_ref().to_path_buf());
    let plugin_command = PluginCommand::OpenTerminal(file_to_open);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response =
        ProtobufOpenTerminalResponse::decode(bytes_from_stdin().unwrap().as_slice()).unwrap();
    OpenTerminalResponse::try_from(response).unwrap()
}

/// 在宿主文件系统的指定位置打开新的终端窗格
/// 此变体与 open_terminal 相同，只是它会在插件附近打开，无论
/// 用户是否正聚焦于该插件
pub fn open_terminal_near_plugin<P: AsRef<Path>>(path: P) -> Option<PaneId> {
    let file_to_open = FileToOpen::new(path.as_ref().to_path_buf());
    let plugin_command = PluginCommand::OpenTerminalNearPlugin(file_to_open);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response =
        ProtobufOpenTerminalNearPluginResponse::decode(bytes_from_stdin().unwrap().as_slice())
            .unwrap();
    OpenTerminalNearPluginResponse::try_from(response).unwrap()
}

/// 在宿主文件系统的指定位置打开新的浮动终端窗格
pub fn open_terminal_floating<P: AsRef<Path>>(
    path: P,
    coordinates: Option<FloatingPaneCoordinates>,
) -> Option<PaneId> {
    let file_to_open = FileToOpen::new(path.as_ref().to_path_buf());
    let plugin_command = PluginCommand::OpenTerminalFloating(file_to_open, coordinates);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response =
        ProtobufOpenTerminalFloatingResponse::decode(bytes_from_stdin().unwrap().as_slice())
            .unwrap();
    OpenTerminalFloatingResponse::try_from(response).unwrap()
}

/// 在宿主文件系统的指定位置打开新的浮动终端窗格
/// 此变体与 open_terminal_floating 相同，只是它会在插件附近打开，无论
/// whether the user was focused on it or not
pub fn open_terminal_floating_near_plugin<P: AsRef<Path>>(
    path: P,
    coordinates: Option<FloatingPaneCoordinates>,
) -> Option<PaneId> {
    let file_to_open = FileToOpen::new(path.as_ref().to_path_buf());
    let plugin_command = PluginCommand::OpenTerminalFloatingNearPlugin(file_to_open, coordinates);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response = ProtobufOpenTerminalFloatingNearPluginResponse::decode(
        bytes_from_stdin().unwrap().as_slice(),
    )
    .unwrap();
    OpenTerminalFloatingNearPluginResponse::try_from(response).unwrap()
}

/// 在宿主文件系统的指定位置打开新的终端窗格，临时
/// 替换焦点窗格
pub fn open_terminal_in_place<P: AsRef<Path>>(path: P) -> Option<PaneId> {
    let file_to_open = FileToOpen::new(path.as_ref().to_path_buf());
    let plugin_command = PluginCommand::OpenTerminalInPlace(file_to_open);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response =
        ProtobufOpenTerminalInPlaceResponse::decode(bytes_from_stdin().unwrap().as_slice())
            .unwrap();
    OpenTerminalInPlaceResponse::try_from(response).unwrap()
}

/// 在宿主文件系统的指定位置打开新的终端窗格, temporarily
/// 替换插件窗格
pub fn open_terminal_in_place_of_plugin<P: AsRef<Path>>(
    path: P,
    close_plugin_after_replace: bool,
) -> Option<PaneId> {
    let file_to_open = FileToOpen::new(path.as_ref().to_path_buf());
    let plugin_command =
        PluginCommand::OpenTerminalInPlaceOfPlugin(file_to_open, close_plugin_after_replace);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response =
        ProtobufOpenTerminalInPlaceOfPluginResponse::decode(bytes_from_stdin().unwrap().as_slice())
            .unwrap();
    OpenTerminalInPlaceOfPluginResponse::try_from(response).unwrap()
}

/// 使用指定的命令和参数打开新的命令窗格（此类窗格允许用户通过 Zellij 界面控制命令、重新运行并查看其退出状态）。
pub fn open_command_pane(
    command_to_run: CommandToRun,
    context: BTreeMap<String, String>,
) -> Option<PaneId> {
    let plugin_command = PluginCommand::OpenCommandPane(command_to_run, context);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response =
        ProtobufOpenCommandPaneResponse::decode(bytes_from_stdin().unwrap().as_slice()).unwrap();
    OpenCommandPaneResponse::try_from(response).unwrap()
}

/// 使用指定的命令和参数打开新的命令窗格（此类窗格允许用户通过 Zellij 界面控制命令、重新运行并查看其退出状态）。
/// 此变体与 `open_command_pane` 相同，只是它会在与插件相同的标签页中打开窗格，
/// 无论用户是否正聚焦于该插件
pub fn open_command_pane_near_plugin(
    command_to_run: CommandToRun,
    context: BTreeMap<String, String>,
) -> Option<PaneId> {
    let plugin_command = PluginCommand::OpenCommandPaneNearPlugin(command_to_run, context);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response =
        ProtobufOpenCommandPaneNearPluginResponse::decode(bytes_from_stdin().unwrap().as_slice())
            .unwrap();
    OpenCommandPaneNearPluginResponse::try_from(response).unwrap()
}

/// 使用指定的命令和参数打开新的浮动命令窗格（此类窗格允许用户通过 Zellij 界面控制命令、重新运行并查看其退出状态）。
pub fn open_command_pane_floating(
    command_to_run: CommandToRun,
    coordinates: Option<FloatingPaneCoordinates>,
    context: BTreeMap<String, String>,
) -> Option<PaneId> {
    let plugin_command =
        PluginCommand::OpenCommandPaneFloating(command_to_run, coordinates, context);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response =
        ProtobufOpenCommandPaneFloatingResponse::decode(bytes_from_stdin().unwrap().as_slice())
            .unwrap();
    OpenCommandPaneFloatingResponse::try_from(response).unwrap()
}

/// 使用指定的命令和参数打开新的浮动命令窗格（此类窗格允许用户通过 Zellij 界面控制命令、重新运行并查看其退出状态）。
/// 此变体与 `open_command_pane_floating` 相同，只是它会在与插件相同的标签页中打开窗格，
/// plugin regardless of whether the user is focused on it
pub fn open_command_pane_floating_near_plugin(
    command_to_run: CommandToRun,
    coordinates: Option<FloatingPaneCoordinates>,
    context: BTreeMap<String, String>,
) -> Option<PaneId> {
    let plugin_command =
        PluginCommand::OpenCommandPaneFloatingNearPlugin(command_to_run, coordinates, context);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response = ProtobufOpenCommandPaneFloatingNearPluginResponse::decode(
        bytes_from_stdin().unwrap().as_slice(),
    )
    .unwrap();
    OpenCommandPaneFloatingNearPluginResponse::try_from(response).unwrap()
}

/// 使用指定的命令和参数打开新的原位命令窗格（此类窗格允许用户通过 Zellij 界面控制命令、重新运行并查看其退出状态）。
pub fn open_command_pane_in_place(
    command_to_run: CommandToRun,
    context: BTreeMap<String, String>,
) -> Option<PaneId> {
    let plugin_command = PluginCommand::OpenCommandPaneInPlace(command_to_run, context);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response =
        ProtobufOpenCommandPaneInPlaceResponse::decode(bytes_from_stdin().unwrap().as_slice())
            .unwrap();
    OpenCommandPaneInPlaceResponse::try_from(response).unwrap()
}

/// 使用指定的命令和参数打开新的原位命令窗格（此类窗格允许用户通过 Zellij 界面控制命令、重新运行并查看其退出状态）。
/// 此变体与 open_command_pane_in_place 相同，只是它始终替换
/// 插件窗格，而非用户当前聚焦的任何窗格
pub fn open_command_pane_in_place_of_plugin(
    command_to_run: CommandToRun,
    close_plugin_after_replace: bool,
    context: BTreeMap<String, String>,
) -> Option<PaneId> {
    let plugin_command = PluginCommand::OpenCommandPaneInPlaceOfPlugin(
        command_to_run,
        close_plugin_after_replace,
        context,
    );
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response = ProtobufOpenCommandPaneInPlaceOfPluginResponse::decode(
        bytes_from_stdin().unwrap().as_slice(),
    )
    .unwrap();
    OpenCommandPaneInPlaceOfPluginResponse::try_from(response).unwrap()
}

/// 打开一个命令窗格，替换由 `pane_id` 标识的窗格。
/// 与 `open_command_pane_in_place` 不同，此函数通过 ID 定位任意窗格，而非
/// 焦点窗格，且不会改变焦点。如果 `close_replaced_pane` 为 false，被替换的
/// 窗格将被抑制，并在新窗格关闭时恢复；如果为 true，则被永久关闭。
/// 返回新打开窗格的 `PaneId`，如果操作失败则返回 `None`。
pub fn open_command_pane_in_place_of_pane_id(
    pane_id: PaneId,
    command_to_run: CommandToRun,
    close_replaced_pane: bool,
    context: BTreeMap<String, String>,
) -> Option<PaneId> {
    let plugin_command = PluginCommand::OpenCommandPaneInPlaceOfPaneId(
        pane_id,
        command_to_run,
        close_replaced_pane,
        context,
    );
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response = ProtobufOpenCommandPaneInPlaceOfPaneIdResponse::decode(
        bytes_from_stdin().unwrap().as_slice(),
    )
    .unwrap();
    OpenCommandPaneInPlaceOfPaneIdResponse::try_from(response).unwrap()
}

/// 打开一个终端窗格，替换由 `pane_id` 标识的窗格。
/// 与 `open_terminal_in_place` 不同，此函数通过 ID 定位任意窗格，而非
/// focused pane, and does not change focus. If `close_replaced_pane` is false, the replaced
/// pane is suppressed and restored when the new pane closes; if true, it is permanently closed.
/// `cwd` 设置新终端的工作目录。返回新打开
/// 窗格的 `PaneId`，如果操作失败则返回 `None`。
pub fn open_terminal_pane_in_place_of_pane_id<P: AsRef<Path>>(
    pane_id: PaneId,
    cwd: P,
    close_replaced_pane: bool,
) -> Option<PaneId> {
    let file_to_open = FileToOpen {
        path: cwd.as_ref().to_path_buf(),
        ..Default::default()
    };
    let plugin_command =
        PluginCommand::OpenTerminalPaneInPlaceOfPaneId(pane_id, file_to_open, close_replaced_pane);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response = ProtobufOpenTerminalPaneInPlaceOfPaneIdResponse::decode(
        bytes_from_stdin().unwrap().as_slice(),
    )
    .unwrap();
    OpenTerminalPaneInPlaceOfPaneIdResponse::try_from(response).unwrap()
}

/// 打开一个编辑器窗格，替换由 `pane_id` 标识的窗格。
/// 与 `open_file_in_place` 不同，此函数通过 ID 定位任意窗格，而非
/// focused pane, and does not change focus. If `close_replaced_pane` is false, the replaced
/// pane is suppressed and restored when the new pane closes; if true, it is permanently closed.
/// 返回新打开窗格的 `PaneId`，如果操作失败则返回 `None`。
pub fn open_edit_pane_in_place_of_pane_id(
    pane_id: PaneId,
    file_to_open: FileToOpen,
    close_replaced_pane: bool,
    context: BTreeMap<String, String>,
) -> Option<PaneId> {
    let plugin_command = PluginCommand::OpenEditPaneInPlaceOfPaneId(
        pane_id,
        file_to_open,
        close_replaced_pane,
        context,
    );
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response =
        ProtobufOpenEditPaneInPlaceOfPaneIdResponse::decode(bytes_from_stdin().unwrap().as_slice())
            .unwrap();
    OpenEditPaneInPlaceOfPaneIdResponse::try_from(response).unwrap()
}

/// 使用指定的命令和参数打开新的隐藏（后台）命令窗格（此类窗格允许用户通过 Zellij 界面控制命令、重新运行并查看其退出状态）。
pub fn open_command_pane_background(
    command_to_run: CommandToRun,
    context: BTreeMap<String, String>,
) -> Option<PaneId> {
    let plugin_command = PluginCommand::OpenCommandPaneBackground(command_to_run, context);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response =
        ProtobufOpenCommandPaneBackgroundResponse::decode(bytes_from_stdin().unwrap().as_slice())
            .unwrap();
    OpenCommandPaneBackgroundResponse::try_from(response).unwrap()
}

/// 将焦点标签页切换到指定索引（与默认标签页名称对应，从 `1` 开始，`0` 将被视为 `1`）。
pub fn switch_tab_to(tab_idx: u32) {
    let plugin_command = PluginCommand::SwitchTabTo(tab_idx);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 设置以秒（或其分数）为单位的超时，超时后将以 [`Timer`](./plugin-api-events.md#timer) 事件调用插件的 [update](./plugin-api-events#update) 方法。
pub fn set_timeout(secs: f64) {
    let plugin_command = PluginCommand::SetTimeout(secs);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

#[doc(hidden)]
pub fn exec_cmd(cmd: &[&str]) {
    let plugin_command =
        PluginCommand::ExecCmd(cmd.iter().cloned().map(|s| s.to_owned()).collect());
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 在宿主机器上后台运行此命令，如果订阅了 `RunCommandResult` 事件，
/// 可选择性地收到其输出通知
pub fn run_command(cmd: &[&str], context: BTreeMap<String, String>) {
    let plugin_command = PluginCommand::RunCommand(
        cmd.iter().cloned().map(|s| s.to_owned()).collect(),
        BTreeMap::new(),
        PathBuf::from("."),
        context,
    );
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 在宿主机器上后台运行此命令，提供环境变量和
/// 工作目录。如果订阅了 `RunCommandResult` 事件，可选择性地收到其输出通知
pub fn run_command_with_env_variables_and_cwd(
    cmd: &[&str],
    env_variables: BTreeMap<String, String>,
    cwd: PathBuf,
    context: BTreeMap<String, String>,
) {
    let plugin_command = PluginCommand::RunCommand(
        cmd.iter().cloned().map(|s| s.to_owned()).collect(),
        env_variables,
        cwd,
        context,
    );
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 发起网络请求，如果订阅了 `WebRequestResult` 事件，
/// 可选择性地收到其输出通知，context 将在此事件中原样返回，
/// 可用于例如标记 request_id
pub fn web_request<S: AsRef<str>>(
    url: S,
    verb: HttpVerb,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
    context: BTreeMap<String, String>,
) where
    S: ToString,
{
    let plugin_command = PluginCommand::WebRequest(url.to_string(), verb, headers, body, context);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 从界面中隐藏插件窗格（抑制它）
pub fn hide_self() {
    let plugin_command = PluginCommand::HideSelf;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 从界面中隐藏具有指定 [PaneId] 的窗格（抑制它）
pub fn hide_pane_with_id(pane_id: PaneId) {
    let plugin_command = PluginCommand::HidePaneWithId(pane_id);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 显示插件窗格（如果被抑制则取消抑制），聚焦它并切换到其标签页
pub fn show_self(should_float_if_hidden: bool) {
    let plugin_command = PluginCommand::ShowSelf(should_float_if_hidden);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 显示具有指定 [PaneId] 的窗格（如果被抑制则取消抑制），聚焦它并切换到其标签页
pub fn show_pane_with_id(pane_id: PaneId, should_float_if_hidden: bool, should_focus_pane: bool) {
    let plugin_command =
        PluginCommand::ShowPaneWithId(pane_id, should_float_if_hidden, should_focus_pane);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 关闭此插件窗格
pub fn close_self() {
    let plugin_command = PluginCommand::CloseSelf;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 切换到指定的输入模式（例如 `Normal`、`Tab`、`Pane`）
pub fn switch_to_input_mode(mode: &InputMode) {
    let plugin_command = PluginCommand::SwitchToMode(*mode);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 提供字符串化的 [`布局`](https://zellij.dev/documentation/layouts.html) 以应用于当前会话。如果布局包含多个标签页，它们都将被打开。
pub fn new_tabs_with_layout(layout: &str) -> Vec<usize> {
    let plugin_command = PluginCommand::NewTabsWithLayout(layout.to_owned());
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response = ProtobufNewTabsResponse::decode(bytes_from_stdin().unwrap().as_slice()).unwrap();
    NewTabsResponse::try_from(response).unwrap()
}

/// 提供 LayoutInfo 以在新标签页中应用于当前会话。如果布局包含多个标签页，它们都将被打开。
pub fn new_tabs_with_layout_info<L: AsRef<LayoutInfo>>(layout_info: L) -> Vec<usize> {
    let plugin_command = PluginCommand::NewTabsWithLayoutInfo(layout_info.as_ref().clone());
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response = ProtobufNewTabsResponse::decode(bytes_from_stdin().unwrap().as_slice()).unwrap();
    NewTabsResponse::try_from(response).unwrap()
}

/// 使用默认布局打开新标签页
pub fn new_tab<S: AsRef<str>>(name: Option<S>, cwd: Option<S>) -> Option<usize>
where
    S: ToString,
{
    let name = name.map(|s| s.to_string());
    let cwd = cwd.map(|s| s.to_string());
    let plugin_command = PluginCommand::NewTab { name, cwd };
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response = ProtobufNewTabResponse::decode(bytes_from_stdin().unwrap().as_slice()).unwrap();
    NewTabResponse::try_from(response).unwrap()
}

pub fn new_tab_unfocused<S: AsRef<str>>(name: Option<S>, cwd: Option<S>) -> Option<usize>
where
    S: ToString,
{
    let name = name.map(|s| s.to_string());
    let cwd = cwd.map(|s| s.to_string());
    let plugin_command = PluginCommand::NewTabUnfocused { name, cwd };
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response =
        ProtobufNewTabUnfocusedResponse::decode(bytes_from_stdin().unwrap().as_slice()).unwrap();
    NewTabUnfocusedResponse::try_from(response).unwrap()
}

pub fn new_tiled_pane_in_tab(tab_position: usize) -> Option<PaneId> {
    let plugin_command = PluginCommand::NewTiledPaneInTab { tab_position };
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response =
        ProtobufNewTiledPaneInTabResponse::decode(bytes_from_stdin().unwrap().as_slice()).unwrap();
    NewTiledPaneInTabResponse::try_from(response).unwrap()
}

/// 打开一个新标签页，其中包含运行 `command_to_run` 的命令窗格。
/// 返回所创建标签页和窗格的 `(tab_id, pane_id)`，如果不可用则返回 `None`。
pub fn open_command_pane_in_new_tab(
    command_to_run: CommandToRun,
    context: BTreeMap<String, String>,
) -> (Option<usize>, Option<PaneId>) {
    let plugin_command = PluginCommand::OpenCommandPaneInNewTab(command_to_run, context);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response =
        ProtobufOpenPaneInNewTabResponse::decode(bytes_from_stdin().unwrap().as_slice()).unwrap();
    let result = OpenPaneInNewTabResponse::try_from(response).unwrap();
    (result.tab_id, result.pane_id)
}

/// 打开一个新标签页，其中包含从 `plugin_url` 加载的插件窗格。
/// `plugin_url` 可以是路径（`file:/path/to/plugin.wasm`）或命名别名。
/// 返回所创建标签页和窗格的 `(tab_id, pane_id)`。
pub fn open_plugin_pane_in_new_tab(
    plugin_url: impl ToString,
    configuration: BTreeMap<String, String>,
    context: BTreeMap<String, String>,
) -> (Option<usize>, Option<PaneId>) {
    let plugin_command = PluginCommand::OpenPluginPaneInNewTab {
        plugin_url: plugin_url.to_string(),
        configuration,
        context,
    };
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response =
        ProtobufOpenPaneInNewTabResponse::decode(bytes_from_stdin().unwrap().as_slice()).unwrap();
    let result = OpenPaneInNewTabResponse::try_from(response).unwrap();
    (result.tab_id, result.pane_id)
}

/// 使用指定的插件 URL 和配置打开新的浮动插件窗格。
/// 如果成功，返回新创建插件窗格的窗格 ID。
pub fn open_plugin_pane_floating(
    plugin_url: &str,
    configuration: BTreeMap<String, String>,
    coordinates: Option<FloatingPaneCoordinates>,
    context: BTreeMap<String, String>,
) -> Option<PaneId> {
    let plugin_command = PluginCommand::OpenPluginPaneFloating {
        plugin_url: plugin_url.to_owned(),
        configuration,
        floating_pane_coordinates: coordinates,
        context,
    };
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response =
        ProtobufOpenPluginPaneFloatingResponse::decode(bytes_from_stdin().unwrap().as_slice())
            .unwrap();
    OpenPluginPaneFloatingResponse::try_from(response).unwrap()
}

/// 打开一个新标签页，其中包含用于 `file_to_open` 的编辑器窗格。
/// 返回所创建标签页和窗格的 `(tab_id, pane_id)`。
pub fn open_editor_pane_in_new_tab(
    file_to_open: FileToOpen,
    context: BTreeMap<String, String>,
) -> (Option<usize>, Option<PaneId>) {
    let plugin_command = PluginCommand::OpenEditorPaneInNewTab(file_to_open, context);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response =
        ProtobufOpenPaneInNewTabResponse::decode(bytes_from_stdin().unwrap().as_slice()).unwrap();
    let result = OpenPaneInNewTabResponse::try_from(response).unwrap();
    (result.tab_id, result.pane_id)
}

/// 将焦点切换到下一个标签页，或循环回到第一个
pub fn go_to_next_tab() {
    let plugin_command = PluginCommand::GoToNextTab;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 将焦点切换到上一个标签页，或循环回到最后一个
pub fn go_to_previous_tab() {
    let plugin_command = PluginCommand::GoToPreviousTab;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

pub fn report_panic(info: &std::panic::PanicHookInfo) {
    let panic_payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
        format!("{}", s)
    } else {
        format!("<NO PAYLOAD>")
    };
    let panic_stringified = format!("{}\n\r{:#?}", panic_payload, info).replace("\n", "\r\n");
    let plugin_command = PluginCommand::ReportPanic(panic_stringified);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 增大或减小焦点窗格的大小
pub fn resize_focused_pane(resize: Resize) {
    let plugin_command = PluginCommand::Resize(resize);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 沿指定方向增大或减小焦点窗格的大小（例如 `Left`、`Right`、`Up`、`Down`）。
pub fn resize_focused_pane_with_direction(resize: Resize, direction: Direction) {
    let resize_strategy = ResizeStrategy {
        resize,
        direction: Some(direction),
        invert_on_boundaries: false,
    };
    let plugin_command = PluginCommand::ResizeWithDirection(resize_strategy);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 按时间顺序将焦点切换到下一个窗格
pub fn focus_next_pane() {
    let plugin_command = PluginCommand::FocusNextPane;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 按时间顺序将焦点切换到上一个窗格
pub fn focus_previous_pane() {
    let plugin_command = PluginCommand::FocusPreviousPane;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 将焦点切换到之前聚焦的窗格
pub fn focus_last_pane() {
    let plugin_command = PluginCommand::FocusLastPane;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 沿指定方向切换焦点窗格
pub fn move_focus(direction: Direction) {
    let plugin_command = PluginCommand::MoveFocus(direction);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 沿指定方向切换焦点窗格，如果窗格位于屏幕边缘，则聚焦下一个标签页（右边缘则下一个，左边缘则上一个）。
pub fn move_focus_or_tab(direction: Direction) {
    let plugin_command = PluginCommand::MoveFocusOrTab(direction);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 将用户从活动会话中分离
pub fn detach() {
    let plugin_command = PluginCommand::Detach;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 在用户默认的 `$EDITOR` 中编辑焦点窗格的回滚缓冲区
pub fn edit_scrollback() {
    let plugin_command = PluginCommand::EditScrollback;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 将字节写入焦点窗格的 `STDIN`
pub fn write(bytes: Vec<u8>) {
    let plugin_command = PluginCommand::Write(bytes);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 将字符写入焦点窗格的 `STDIN`
pub fn write_chars(chars: &str) {
    let plugin_command = PluginCommand::WriteChars(chars.to_owned());
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 将任意文本复制到用户的剪贴板
///
/// 遵循用户配置的剪贴板目标（系统剪贴板或主选区）。
/// 需要 WriteToClipboard 权限。
pub fn copy_to_clipboard(text: impl Into<String>) {
    let plugin_command = PluginCommand::CopyToClipboard(text.into());
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 聚焦之前聚焦的标签页（无论标签页位置如何）
pub fn toggle_tab() {
    let plugin_command = PluginCommand::ToggleTab;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 将焦点窗格与另一个窗格交换位置
pub fn move_pane() {
    let plugin_command = PluginCommand::MovePane;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 沿指定方向将焦点窗格与另一个窗格交换位置（例如 `Down`、`Up`、`Left`、`Right`）。
pub fn move_pane_with_direction(direction: Direction) {
    let plugin_command = PluginCommand::MovePaneWithDirection(direction);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 清除焦点窗格的滚动缓冲区
pub fn clear_screen() {
    let plugin_command = PluginCommand::ClearScreen;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 将焦点窗格向上滚动 1 行
pub fn scroll_up() {
    let plugin_command = PluginCommand::ScrollUp;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 将焦点窗格向下滚动 1 行
pub fn scroll_down() {
    let plugin_command = PluginCommand::ScrollDown;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 将焦点窗格一直滚动到滚动缓冲区顶部
pub fn scroll_to_top() {
    let plugin_command = PluginCommand::ScrollToTop;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 将焦点窗格一直滚动到滚动缓冲区底部
pub fn scroll_to_bottom() {
    let plugin_command = PluginCommand::ScrollToBottom;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 将焦点窗格向上滚动一页
pub fn page_scroll_up() {
    let plugin_command = PluginCommand::PageScrollUp;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 将焦点窗格向下滚动一页
pub fn page_scroll_down() {
    let plugin_command = PluginCommand::PageScrollDown;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 切换焦点窗格为全屏或正常大小
pub fn toggle_focus_fullscreen() {
    let plugin_command = PluginCommand::ToggleFocusFullscreen;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

pub fn toggle_focus_no_ui_fullscreen() {
    let plugin_command = PluginCommand::ToggleFocusNoUiFullscreen;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

pub fn focus_host_session() {
    let plugin_command = PluginCommand::FocusHostSession;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 切换界面窗格边框的开启或关闭
pub fn toggle_pane_frames() {
    let plugin_command = PluginCommand::TogglePaneFrames;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

pub fn set_pane_frame_style(pane_frame_style: PaneFrameStyle) {
    let plugin_command = PluginCommand::SetPaneFrameStyle(pane_frame_style);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 嵌入当前焦点窗格（使其停止浮动），如果它不是浮动窗格则将其变为浮动窗格
pub fn toggle_pane_embed_or_eject() {
    let plugin_command = PluginCommand::TogglePaneEmbedOrEject;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

pub fn undo_rename_pane() {
    let plugin_command = PluginCommand::UndoRenamePane;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 关闭焦点窗格
pub fn close_focus() {
    let plugin_command = PluginCommand::CloseFocus;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

pub fn new_pane() {
    let plugin_command = PluginCommand::NewPane;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

pub fn toggle_floating_panes(tab_id: Option<u64>) {
    let plugin_command = PluginCommand::ToggleFloatingPanes { tab_id };
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 开启或关闭当前标签页的 `STDIN` 同步
pub fn toggle_active_tab_sync() {
    let plugin_command = PluginCommand::ToggleActiveTabSync;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 关闭焦点标签页
pub fn close_focused_tab() {
    let plugin_command = PluginCommand::CloseFocusedTab;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

pub fn undo_rename_tab() {
    let plugin_command = PluginCommand::UndoRenameTab;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 完全退出此客户端及所有其他已连接客户端的 Zellij
pub fn quit_zellij() {
    let plugin_command = PluginCommand::QuitZellij;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 切换到上一个 [交换布局](https://zellij.dev/documentation/swap-layouts.html)
pub fn previous_swap_layout() {
    let plugin_command = PluginCommand::PreviousSwapLayout;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 切换到下一个 [交换布局](https://zellij.dev/documentation/swap-layouts.html)
pub fn next_swap_layout() {
    let plugin_command = PluginCommand::NextSwapLayout;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 将焦点切换到具有指定名称的标签页
pub fn go_to_tab_name(tab_name: &str) {
    let plugin_command = PluginCommand::GoToTabName(tab_name.to_owned());
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 将焦点切换到具有指定名称的标签页，如果不存在则创建它
pub fn focus_or_create_tab(tab_name: &str) -> Option<usize> {
    let plugin_command = PluginCommand::FocusOrCreateTab(tab_name.to_owned());
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response =
        ProtobufFocusOrCreateTabResponse::decode(bytes_from_stdin().unwrap().as_slice()).unwrap();
    FocusOrCreateTabResponse::try_from(response).unwrap()
}

pub fn go_to_tab(tab_index: u32) {
    let plugin_command = PluginCommand::GoToTab(tab_index);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

pub fn start_or_reload_plugin(url: &str) {
    let plugin_command = PluginCommand::StartOrReloadPlugin(url.to_owned());
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 关闭具有指定 ID 的终端窗格
pub fn close_terminal_pane(terminal_pane_id: u32) {
    let plugin_command = PluginCommand::CloseTerminalPane(terminal_pane_id);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 关闭具有指定 ID 的插件窗格
pub fn close_plugin_pane(plugin_pane_id: u32) {
    let plugin_command = PluginCommand::ClosePluginPane(plugin_pane_id);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 将焦点切换到具有指定 ID 的终端窗格，如果它被抑制则取消抑制，并切换到其标签页和层级（例如 floating/tiled）。
pub fn focus_terminal_pane(
    terminal_pane_id: u32,
    should_float_if_hidden: bool,
    should_be_in_place_if_hidden: bool,
) {
    let plugin_command = PluginCommand::FocusTerminalPane(
        terminal_pane_id,
        should_float_if_hidden,
        should_be_in_place_if_hidden,
    );
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 将焦点切换到具有指定 ID 的插件窗格，如果它被抑制则取消抑制，并切换到其标签页和层级（例如 floating/tiled）。
pub fn focus_plugin_pane(
    plugin_pane_id: u32,
    should_float_if_hidden: bool,
    should_be_in_place_if_hidden: bool,
) {
    let plugin_command = PluginCommand::FocusPluginPane(
        plugin_pane_id,
        should_float_if_hidden,
        should_be_in_place_if_hidden,
    );
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 修改具有指定 ID 的终端窗格的名称（显示在界面中的标题）。
pub fn rename_terminal_pane<S: AsRef<str>>(terminal_pane_id: u32, new_name: S)
where
    S: ToString,
{
    let plugin_command = PluginCommand::RenameTerminalPane(terminal_pane_id, new_name.to_string());
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 修改具有指定 ID 的插件窗格的名称（显示在界面中的标题）。
pub fn rename_plugin_pane<S: AsRef<str>>(plugin_pane_id: u32, new_name: S)
where
    S: ToString,
{
    let plugin_command = PluginCommand::RenamePluginPane(plugin_pane_id, new_name.to_string());
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 修改具有指定位置的标签页的名称（显示在界面中的标题）。
pub fn rename_tab<S: AsRef<str>>(tab_position: u32, new_name: S)
where
    S: ToString,
{
    let plugin_command = PluginCommand::RenameTab(tab_position, new_name.to_string());
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 修改具有指定 ID 的标签页的名称（显示在界面中的标题）。
pub fn rename_tab_with_id<S: AsRef<str>>(tab_id: u64, new_name: S)
where
    S: ToString,
{
    let plugin_command = PluginCommand::RenameTabWithId(tab_id, new_name.to_string());
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 切换到具有给定名称的会话，如果未提供名称则创建一个
pub fn switch_session(name: Option<&str>) {
    let plugin_command = PluginCommand::SwitchSession(ConnectToSession {
        name: name.map(|n| n.to_string()),
        ..Default::default()
    });
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 切换到具有给定名称的会话，如果未提供名称则创建一个
pub fn switch_session_with_layout(name: Option<&str>, layout: LayoutInfo, cwd: Option<PathBuf>) {
    let plugin_command = PluginCommand::SwitchSession(ConnectToSession {
        name: name.map(|n| n.to_string()),
        layout: Some(layout),
        cwd,
        ..Default::default()
    });
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 切换到具有给定名称的会话，如果未提供名称则创建一个
pub fn switch_session_with_cwd(name: Option<&str>, cwd: Option<PathBuf>) {
    let plugin_command = PluginCommand::SwitchSession(ConnectToSession {
        name: name.map(|n| n.to_string()),
        cwd,
        ..Default::default()
    });
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 切换到具有给定名称的会话，聚焦提供的 pane_id 或提供的
/// 标签页位置（按此优先级）
pub fn switch_session_with_focus(
    name: &str,
    tab_position: Option<usize>,
    pane_id: Option<(u32, bool)>,
) {
    let plugin_command = PluginCommand::SwitchSession(ConnectToSession {
        name: Some(name.to_owned()),
        tab_position,
        pane_id,
        ..Default::default()
    });
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 永久删除具有给定名称的可恢复会话。
///
/// 如果会话的缓存目录已删除，返回 `Ok(())`；`Err(...)`
/// 如果底层 `remove_dir_all` 失败（例如权限不足、路径不存在）。
pub fn delete_dead_session(name: &str) -> Result<(), String> {
    let plugin_command = PluginCommand::DeleteDeadSessionAndReply(name.to_owned());
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response_bytes = bytes_from_stdin()
        .map_err(|e| format!("读取 DeleteDeadSession 响应失败：{}", e))?;
    let protobuf_response = ProtobufDeleteDeadSessionResponse::decode(response_bytes.as_slice())
        .map_err(|e| format!("格式错误的 DeleteDeadSession 响应：{}", e))?;
    match protobuf_response.error {
        Some(err) => Err(err),
        None => Ok(()),
    }
}

/// 永久删除此机器上的所有可恢复会话。
///
/// 受宿主端卡死超时限制（与 kill-all 预算一致），因此
/// 异常的文件系统不会冻结调用插件。
pub fn delete_all_dead_sessions() -> Result<(), String> {
    let plugin_command = PluginCommand::DeleteAllDeadSessionsAndReply;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response_bytes = bytes_from_stdin()
        .map_err(|e| format!("读取 DeleteAllDeadSessions 响应失败：{}", e))?;
    let protobuf_response =
        ProtobufDeleteAllDeadSessionsResponse::decode(response_bytes.as_slice())
            .map_err(|e| format!("格式错误的 DeleteAllDeadSessions 响应：{}", e))?;
    match protobuf_response.error {
        Some(err) => Err(err),
        None => Ok(()),
    }
}

/// 重命名当前会话
pub fn rename_session(name: &str) {
    let plugin_command = PluginCommand::RenameSession(name.to_owned());
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 解除管道输入端的阻塞，如果有下一条消息则请求发送
pub fn unblock_cli_pipe_input(pipe_name: &str) {
    let plugin_command = PluginCommand::UnblockCliPipeInput(pipe_name.to_owned());
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 阻塞管道的输入端，只有在此插件或另一个插件解除阻塞后才会释放
pub fn block_cli_pipe_input(pipe_name: &str) {
    let plugin_command = PluginCommand::BlockCliPipeInput(pipe_name.to_owned());
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 向管道的输出端发送输出，这不会影响同一管道的输入端
pub fn cli_pipe_output(pipe_name: &str, output: &str) {
    let plugin_command = PluginCommand::CliPipeOutput(pipe_name.to_owned(), output.to_owned());
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 向插件发送消息，如果插件尚未运行则将其启动
pub fn pipe_message_to_plugin(message_to_plugin: MessageToPlugin) {
    let plugin_command = PluginCommand::MessageToPlugin(message_to_plugin);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 断开所有其他客户端与当前会话的连接
pub fn disconnect_other_clients() {
    let plugin_command = PluginCommand::DisconnectOtherClients;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 终止列表中的所有 Zellij 会话。
///
/// 等待每个对等会话确认终止（通过其自己的 IPC
/// 套接字——服务端处理器会读回对等方
/// 作为关闭一部分发送的 `Exit` 消息，或者如果对等方先死亡则读取流关闭）。
/// 在所有终止都已确认或其中一个失败后返回。卡死的
/// 对等方受短暂的服务端卡死超时限制。
pub fn kill_sessions<S: AsRef<str>>(session_names: &[S]) -> Result<(), String>
where
    S: ToString,
{
    let plugin_command = PluginCommand::KillSessionsAndReply(
        session_names.into_iter().map(|s| s.to_string()).collect(),
    );
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response_bytes =
        bytes_from_stdin().map_err(|e| format!("读取 KillSessions 响应失败：{}", e))?;
    let protobuf_response = ProtobufKillSessionsResponse::decode(response_bytes.as_slice())
        .map_err(|e| format!("格式错误的 KillSessions 响应：{}", e))?;
    match protobuf_response.error {
        Some(err) => Err(err),
        None => Ok(()),
    }
}

/// 列出 Windows 卷（驱动器和 WSL 发行版）。
/// 结果通过 `FileSystemUpdate` 事件返回。
/// 此命令仅在 Windows 上受支持，需要 FullHdAccess 权限。
pub fn list_windows_volumes() {
    let plugin_command = PluginCommand::ListWindowsVolumes;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 扫描宿主文件系统中的特定文件夹（这是针对某些 WASI 运行时性能
/// 问题的变通方案），不会跟随符号链接
pub fn scan_host_folder<S: AsRef<Path>>(folder_to_scan: &S) {
    let plugin_command = PluginCommand::ScanHostFolder(folder_to_scan.as_ref().to_path_buf());
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

pub fn set_soft_keyboard(on: bool) {
    let plugin_command = PluginCommand::SetSoftKeyboard(on);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 开始监视宿主文件夹的文件系统更改（注意：目前
/// 有些不稳定）
pub fn watch_filesystem() {
    let plugin_command = PluginCommand::WatchFilesystem;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 同步获取 KDL 格式的序列化会话布局
/// 注意：这会从导出的布局中移除发起请求的插件
pub fn dump_session_layout() -> Result<(String, Option<LayoutMetadata>), String> {
    dump_session_layout_impl(None)
}

/// 同步获取特定标签页的 KDL 格式序列化布局
/// note: this removes the requesting plugin from the dumped layout
pub fn dump_session_layout_for_tab(
    tab_index: usize,
) -> Result<(String, Option<LayoutMetadata>), String> {
    dump_session_layout_impl(Some(tab_index))
}

fn dump_session_layout_impl(
    tab_index: Option<usize>,
) -> Result<(String, Option<LayoutMetadata>), String> {
    let plugin_command = PluginCommand::DumpSessionLayout { tab_index };
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());

    unsafe { host_run_plugin_command() };

    let response_bytes =
        bytes_from_stdin().map_err(|e| format!("从标准输入读取响应失败：{:?}", e))?;
    let protobuf_response = ProtobufDumpSessionLayoutResponse::decode(response_bytes.as_slice())
        .map_err(|e| format!("解码 protobuf 响应失败：{}", e))?;

    // 如果存在则提取元数据
    let metadata = protobuf_response
        .metadata
        .and_then(|pb_metadata| pb_metadata.try_into().ok());

    match protobuf_response.result {
        Some(dump_session_layout_response::Result::LayoutContent(content)) => {
            Ok((content, metadata))
        },
        Some(dump_session_layout_response::Result::Error(error)) => Err(error),
        None => Err("服务器返回了空响应".to_string()),
    }
}

/// 解析 KDL 布局字符串并返回 LayoutMetadata
pub fn parse_layout(layout_string: &str) -> Result<LayoutMetadata, LayoutParsingError> {
    let plugin_command = PluginCommand::ParseLayout(layout_string.to_string());
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());

    unsafe { host_run_plugin_command() };

    let response_bytes = bytes_from_stdin().map_err(|_| LayoutParsingError::SyntaxError)?;

    let protobuf_response = ProtobufParseLayoutResponse::decode(response_bytes.as_slice())
        .map_err(|_| LayoutParsingError::SyntaxError)?;

    match protobuf_response.result {
        Some(parse_layout_response::Result::Metadata(metadata)) => metadata
            .try_into()
            .map_err(|_| LayoutParsingError::SyntaxError),
        Some(parse_layout_response::Result::Error(error)) => Err(error
            .try_into()
            .map_err(|_| LayoutParsingError::SyntaxError)?),
        None => Err(LayoutParsingError::SyntaxError),
    }
}

/// 获取客户端列表、其焦点窗格和正在运行的命令或焦点插件，以
/// Event::ListClients 形式返回（注意：必须订阅此事件）
pub fn list_clients() {
    let plugin_command = PluginCommand::ListClients;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 更改当前用户的配置
pub fn reconfigure(new_config: String, save_configuration_file: bool) {
    let plugin_command = PluginCommand::Reconfigure(new_config, save_configuration_file);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 在窗格中重新运行命令
pub fn rerun_command_pane(terminal_pane_id: u32) {
    let plugin_command = PluginCommand::RerunCommandPane(terminal_pane_id);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// close_terminal_pane 和 close_plugin_pane 的语法糖
pub fn close_pane_with_id(pane_id: PaneId) {
    let plugin_command = match pane_id {
        PaneId::Terminal(terminal_pane_id) => PluginCommand::CloseTerminalPane(terminal_pane_id),
        PaneId::Plugin(plugin_pane_id) => PluginCommand::ClosePluginPane(plugin_pane_id),
    };
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 使用可选方向（左/右/上/下）调整指定窗格的大小（增大/减小）
pub fn resize_pane_with_id(resize_strategy: ResizeStrategy, pane_id: PaneId) {
    let plugin_command = PluginCommand::ResizePaneIdWithDirection(resize_strategy, pane_id);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 将焦点切换到具有指定 ID 的窗格，如果它被抑制则取消抑制，并切换到其标签页和层级（例如 floating/tiled）。
pub fn focus_pane_with_id(
    pane_id: PaneId,
    should_float_if_hidden: bool,
    should_be_in_place_if_hidden: bool,
) {
    let plugin_command = match pane_id {
        PaneId::Terminal(terminal_pane_id) => PluginCommand::FocusTerminalPane(
            terminal_pane_id,
            should_float_if_hidden,
            should_be_in_place_if_hidden,
        ),
        PaneId::Plugin(plugin_pane_id) => PluginCommand::FocusPluginPane(
            plugin_pane_id,
            should_float_if_hidden,
            should_be_in_place_if_hidden,
        ),
    };
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 在用户默认的 `$EDITOR` 中编辑指定窗格的回滚缓冲区（目前仅
/// 适用于终端窗格）
pub fn edit_scrollback_for_pane_with_id(pane_id: PaneId) {
    let plugin_command = PluginCommand::EditScrollbackForPaneWithId(pane_id);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 从指定窗格检索回滚缓冲区内容
///
/// # Arguments
/// * `pane_id` - 要获取回滚缓冲区的窗格 ID
/// * `get_full_scrollback` - 是否检索完整的回滚缓冲区（包括视口上方和下方的行）
///
/// # Returns
/// * `Ok(PaneContents)` - 成功时返回窗格内容
/// * `Err(String)` - 如果窗格未找到、超时或发生其他错误时的错误消息
pub fn get_pane_scrollback(
    pane_id: PaneId,
    get_full_scrollback: bool,
) -> Result<PaneContents, String> {
    let plugin_command = PluginCommand::GetPaneScrollback {
        pane_id,
        get_full_scrollback,
    };
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    // 从 stdin 读取响应
    let response_bytes =
        bytes_from_stdin().map_err(|e| format!("从标准输入读取响应失败：{:?}", e))?;

    // 解码 protobuf 响应
    let protobuf_response = ProtobufPaneScrollbackResponse::decode(response_bytes.as_slice())
        .map_err(|e| format!("解码 protobuf 响应失败：{}", e))?;

    // 转换为 Rust 类型
    let response = PaneScrollbackResponse::try_from(protobuf_response)
        .map_err(|e| format!("转换 protobuf 响应失败：{}", e))?;

    // 将 Result 枚举转换为实际的 Result 类型
    match response {
        PaneScrollbackResponse::Ok(contents) => Ok(contents),
        PaneScrollbackResponse::Err(error_msg) => Err(error_msg),
    }
}

/// 将字节写入指定窗格的 `STDIN`
pub fn write_to_pane_id(bytes: Vec<u8>, pane_id: PaneId) {
    let plugin_command = PluginCommand::WriteToPaneId(bytes, pane_id);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 将字符写入指定窗格的 `STDIN`
pub fn write_chars_to_pane_id(chars: &str, pane_id: PaneId) {
    let plugin_command = PluginCommand::WriteCharsToPaneId(chars.to_owned(), pane_id);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 向由此 PaneId 标识的终端窗格内运行的进程发送 SIGINT
pub fn send_sigint_to_pane_id(pane_id: PaneId) {
    let plugin_command = PluginCommand::SendSigintToPaneId(pane_id);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 向由此 PaneId 标识的终端窗格内运行的进程发送 SIGKILL
pub fn send_sigkill_to_pane_id(pane_id: PaneId) {
    let plugin_command = PluginCommand::SendSigkillToPaneId(pane_id);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 获取终端窗格内运行进程的 PID
pub fn get_pane_pid(pane_id: PaneId) -> Result<i32, String> {
    let plugin_command = PluginCommand::GetPanePid { pane_id };
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    // Read response from stdin
    let response_bytes =
        bytes_from_stdin().map_err(|e| format!("从标准输入读取响应失败：{:?}", e))?;

    // Decode protobuf response
    let protobuf_response = ProtobufGetPanePidResponse::decode(response_bytes.as_slice())
        .map_err(|e| format!("解码 protobuf 响应失败：{}", e))?;

    // Convert to Rust type
    let response = GetPanePidResponse::try_from(protobuf_response)
        .map_err(|e| format!("转换 protobuf 响应失败：{}", e))?;

    // Convert Result enum to actual Result type
    match response {
        GetPanePidResponse::Ok(pid) => Ok(pid),
        GetPanePidResponse::Err(error_msg) => Err(error_msg),
    }
}

/// 通过 ID 获取特定窗格当前正在运行的命令。
///
/// 这会向操作系统查询**当前**正在运行的命令，
/// 而非启动窗格时使用的初始命令。命令以
/// 字符串向量（argv 风格）返回，其中第一个元素是
/// 可执行文件，后续元素是参数。
///
/// # Arguments
/// * `pane_id` - 要查询的窗格 ID
///
/// # Returns
/// * `Ok(Vec<String>)` - 命令和参数作为独立字符串
/// * `Err(String)` - 以下情况的错误消息：
///   - 窗格是插件（只有终端窗格有命令）
///   - 窗格不存在
///   - 操作系统查询失败
///
/// # 所需权限
/// * `ReadApplicationState`
///
/// # Example
/// ```no_run
/// use zellij_tile::prelude::*;
///
/// let pane_id = PaneId::Terminal(1);
/// match get_pane_running_command(pane_id) {
///     Ok(cmd) => eprintln!("Running: {} {}", cmd[0], cmd[1..].join(" ")),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn get_pane_running_command(pane_id: PaneId) -> Result<Vec<String>, String> {
    let plugin_command = PluginCommand::GetPaneRunningCommand { pane_id };
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let protobuf_response =
        ProtobufGetPaneRunningCommandResponse::decode(bytes_from_stdin().unwrap().as_slice())
            .unwrap();

    match protobuf_response.result {
        Some(get_pane_running_command_response::Result::Command(cmd)) => Ok(cmd.args),
        Some(get_pane_running_command_response::Result::Error(err)) => Err(err),
        None => Err("服务器返回了空响应".to_string()),
    }
}

/// 获取此机器上所有活动和可恢复会话的最新快照。
///
/// # 权限 Required
/// * `ReadApplicationState`
pub fn get_session_list() -> Result<SessionListSnapshot, String> {
    let plugin_command = PluginCommand::GetSessionList;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let protobuf_response =
        ProtobufGetSessionListResponse::decode(bytes_from_stdin().unwrap().as_slice()).unwrap();

    match protobuf_response.result {
        Some(get_session_list_response::Result::Snapshot(snapshot)) => {
            let mut live_sessions = Vec::new();
            for manifest in snapshot.live_sessions {
                match SessionInfo::try_from(manifest) {
                    Ok(si) => live_sessions.push(si),
                    Err(e) => return Err(format!("格式错误的会话清单：{}", e)),
                }
            }
            let resurrectable_sessions = snapshot
                .resurrectable_sessions
                .into_iter()
                .map(<(String, std::time::Duration)>::from)
                .collect();
            Ok(SessionListSnapshot {
                live_sessions,
                resurrectable_sessions,
            })
        },
        Some(get_session_list_response::Result::Error(err)) => Err(err),
        None => Err("服务器返回了空响应".to_string()),
    }
}

/// 通过 ID 获取特定窗格的当前工作目录。
///
/// 这会向操作系统查询窗格中运行进程的**当前**工作目录。
/// CWD 可能会随着用户
/// 在终端中导航目录而改变。
///
/// # Arguments
/// * `pane_id` - The ID of the pane to query
///
/// # Returns
/// * `Ok(PathBuf)` - 当前工作目录
/// * `Err(String)` - Error message if:
///   - 窗格是插件（只有终端窗格有 CWD）
///   - Pane doesn't exist
///   - 操作系统查询失败（进程可能已退出）
///   - CWD 不可访问（权限不足、目录已删除）
///
/// # Permissions Required
/// * `ReadApplicationState`
///
/// # Example
/// ```no_run
/// use zellij_tile::prelude::*;
///
/// let pane_id = PaneId::Terminal(1);
/// match get_pane_cwd(pane_id) {
///     Ok(cwd) => eprintln!("CWD: {}", cwd.display()),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn get_pane_cwd(pane_id: PaneId) -> Result<PathBuf, String> {
    let plugin_command = PluginCommand::GetPaneCwd { pane_id };
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let protobuf_response =
        ProtobufGetPaneCwdResponse::decode(bytes_from_stdin().unwrap().as_slice()).unwrap();

    match protobuf_response.result {
        Some(get_pane_cwd_response::Result::Cwd(cwd_str)) => Ok(PathBuf::from(cwd_str)),
        Some(get_pane_cwd_response::Result::Error(err)) => Err(err),
        None => Err("服务器返回了空响应".to_string()),
    }
}

/// 将布局保存到用户的布局目录
///
/// # Arguments
/// * `layout_name` - 布局文件的名称（不含 .kdl 扩展名）
/// * `layout_kdl` - 表示布局的 KDL 字符串
/// * `overwrite` - 如果文件已存在是否覆盖
///
/// # Returns
/// * `Ok(())` - 布局成功验证并保存
/// * `Err(String)` - 错误消息（解析错误、I/O 错误、文件已存在等）
pub fn save_layout<S: AsRef<str>>(
    layout_name: S,
    layout_kdl: S,
    overwrite: bool,
) -> Result<(), String> {
    let plugin_command = PluginCommand::SaveLayout {
        layout_name: layout_name.as_ref().to_owned(),
        layout_kdl: layout_kdl.as_ref().to_owned(),
        overwrite,
    };
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    // Read response from stdin
    let response_bytes =
        bytes_from_stdin().map_err(|e| format!("从标准输入读取响应失败：{:?}", e))?;

    // Decode protobuf response
    let protobuf_response = ProtobufSaveLayoutResponse::decode(response_bytes.as_slice())
        .map_err(|e| format!("解码 protobuf 响应失败：{}", e))?;

    // Convert to Rust type
    let response = SaveLayoutResponse::try_from(protobuf_response)
        .map_err(|e| format!("转换 protobuf 响应失败：{}", e))?;

    // Convert Result enum to actual Result type
    match response {
        SaveLayoutResponse::Ok(_) => Ok(()),
        SaveLayoutResponse::Err(error_msg) => Err(error_msg),
    }
}

/// 从用户的布局目录中删除布局。
///
/// # Arguments
///
/// * `layout_name` - 要删除的布局文件的名称（不含 `.kdl` 扩展名）。
///                   布局名称在服务端经过清理，以防止目录遍历。
///
/// # Returns
///
/// * `Ok(())` - 布局成功删除
/// * `Err(String)` - 发生错误并附带描述性消息（例如文件未找到、权限被拒绝）
///
/// # Permissions
///
/// 需要 `ChangeApplicationState` 权限。
///
/// # 示例
///
/// ```no_run
/// use zellij_tile::prelude::*;
///
/// // 删除名为 "my-layout" 的布局
/// match delete_layout("my-layout") {
///     Ok(_) => eprintln!("Layout deleted successfully"),
///     Err(e) => eprintln!("Failed to delete layout: {}", e),
/// }
/// ```
pub fn delete_layout<S: AsRef<str>>(layout_name: S) -> Result<(), String> {
    let plugin_command = PluginCommand::DeleteLayout {
        layout_name: layout_name.as_ref().to_owned(),
    };
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    // Read response from stdin
    let response_bytes =
        bytes_from_stdin().map_err(|e| format!("从标准输入读取响应失败：{:?}", e))?;

    // Decode protobuf response
    let protobuf_response = ProtobufDeleteLayoutResponse::decode(response_bytes.as_slice())
        .map_err(|e| format!("解码 protobuf 响应失败：{}", e))?;

    // Convert to Rust type
    let response = DeleteLayoutResponse::try_from(protobuf_response)
        .map_err(|e| format!("转换 protobuf 响应失败：{}", e))?;

    // Convert Result enum to actual Result type
    match response {
        DeleteLayoutResponse::Ok(_) => Ok(()),
        DeleteLayoutResponse::Err(error_msg) => Err(error_msg),
    }
}

/// 重命名用户布局目录中的布局文件
///
/// # Arguments
/// * `old_layout_name` - 布局的当前名称（不含 .kdl 扩展名）
/// * `new_layout_name` - 布局的新名称（不含 .kdl 扩展名）
///
/// # Returns
/// * `Ok(())` - 布局成功重命名
/// * `Err(String)` - 描述出错原因的错误消息
///
/// # 错误情况
/// * 旧布局名称无效（为空、包含路径分隔符等）
/// * 新布局名称无效
/// * 源布局文件不存在
/// * 目标布局文件已存在（不覆盖）
/// * 布局目录未找到
/// * 重命名操作期间的文件系统错误
///
/// # Permissions
///
/// 需要 `ChangeApplicationState` 权限。
///
/// # Examples
///
/// ```no_run
/// use zellij_tile::prelude::*;
///
/// // 将布局从 "old-name" 重命名为 "new-name"
/// match rename_layout("old-name", "new-name") {
///     Ok(_) => eprintln!("Layout renamed successfully"),
///     Err(e) => eprintln!("Failed to rename layout: {}", e),
/// }
/// ```
pub fn rename_layout(
    old_layout_name: impl Into<String>,
    new_layout_name: impl Into<String>,
) -> Result<(), String> {
    let plugin_command = PluginCommand::RenameLayout {
        old_layout_name: old_layout_name.into(),
        new_layout_name: new_layout_name.into(),
    };
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    // Read response from stdin
    let response_bytes =
        bytes_from_stdin().map_err(|e| format!("从标准输入读取响应失败：{:?}", e))?;

    // Decode protobuf response
    let protobuf_response = ProtobufRenameLayoutResponse::decode(response_bytes.as_slice())
        .map_err(|e| format!("解码 protobuf 响应失败：{}", e))?;

    // 转换为原生响应类型
    let response: RenameLayoutResponse = protobuf_response
        .try_into()
        .map_err(|e| format!("转换 protobuf 响应失败：{}", e))?;

    match response {
        RenameLayoutResponse::Ok(_) => Ok(()),
        RenameLayoutResponse::Err(error_msg) => Err(error_msg),
    }
}

/// 在用户默认的 `$EDITOR` 中打开布局文件
pub fn edit_layout<S: AsRef<str>>(
    layout_name: S,
    context: BTreeMap<String, String>,
) -> Result<(), String> {
    let layout_name = layout_name.as_ref().to_owned();
    let plugin_command = PluginCommand::EditLayout {
        layout_name,
        context,
    };
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    // Read response from stdin
    let response_bytes =
        bytes_from_stdin().map_err(|e| format!("从标准输入读取响应失败：{:?}", e))?;

    // Decode protobuf response
    let protobuf_response = ProtobufEditLayoutResponse::decode(response_bytes.as_slice())
        .map_err(|e| format!("解码 protobuf 响应失败：{}", e))?;

    // Convert to Rust type
    let response = EditLayoutResponse::try_from(protobuf_response)
        .map_err(|e| format!("转换 protobuf 响应失败：{}", e))?;

    // Convert Result enum to actual Result type
    match response {
        EditLayoutResponse::Ok(_) => Ok(()),
        EditLayoutResponse::Err(error_msg) => Err(error_msg),
    }
}

/// 将具有此 ID 的窗格与另一个窗格交换位置
pub fn move_pane_with_pane_id(pane_id: PaneId) {
    let plugin_command = PluginCommand::MovePaneWithPaneId(pane_id);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 沿指定方向将具有此 ID 的窗格与另一个窗格交换位置（例如 `Down`、`Up`、`Left`、`Right`）。
pub fn move_pane_with_pane_id_in_direction(pane_id: PaneId, direction: Direction) {
    let plugin_command = PluginCommand::MovePaneWithPaneIdInDirection(pane_id, direction);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 清除指定窗格的滚动缓冲区
pub fn clear_screen_for_pane_id(pane_id: PaneId) {
    let plugin_command = PluginCommand::ClearScreenForPaneId(pane_id);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 将指定窗格向上滚动 1 行
pub fn scroll_up_in_pane_id(pane_id: PaneId) {
    let plugin_command = PluginCommand::ScrollUpInPaneId(pane_id);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 将指定窗格向下滚动 1 行
pub fn scroll_down_in_pane_id(pane_id: PaneId) {
    let plugin_command = PluginCommand::ScrollDownInPaneId(pane_id);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 将指定窗格一直滚动到滚动缓冲区顶部
pub fn scroll_to_top_in_pane_id(pane_id: PaneId) {
    let plugin_command = PluginCommand::ScrollToTopInPaneId(pane_id);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 将指定窗格一直滚动到滚动缓冲区底部
pub fn scroll_to_bottom_in_pane_id(pane_id: PaneId) {
    let plugin_command = PluginCommand::ScrollToBottomInPaneId(pane_id);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 将指定窗格向上滚动一页
pub fn page_scroll_up_in_pane_id(pane_id: PaneId) {
    let plugin_command = PluginCommand::PageScrollUpInPaneId(pane_id);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 将指定窗格向下滚动一页
pub fn page_scroll_down_in_pane_id(pane_id: PaneId) {
    let plugin_command = PluginCommand::PageScrollDownInPaneId(pane_id);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 切换指定窗格为全屏或正常大小
pub fn toggle_pane_id_fullscreen(pane_id: PaneId) {
    let plugin_command = PluginCommand::TogglePaneIdFullscreen(pane_id);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 嵌入指定窗格（使其停止浮动），如果它不是浮动窗格则将其变为浮动窗格
pub fn toggle_pane_embed_or_eject_for_pane_id(pane_id: PaneId) {
    let plugin_command = PluginCommand::TogglePaneEmbedOrEjectForPaneId(pane_id);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 关闭焦点标签页
pub fn close_tab_with_index(tab_index: usize) {
    let plugin_command = PluginCommand::CloseTabWithIndex(tab_index);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 关闭具有给定稳定 ID 的标签页。
///
/// 与 `close_tab_with_index` 不同，此函数通过标签页的稳定
/// `tab_id` 而非显示位置来标识标签页，显示位置可能会随着标签页的移动
/// 或关闭而改变。tab_id 可从 `TabInfo.tab_id` 获取。
pub fn close_tab_with_id(tab_id: u64) {
    let plugin_command = PluginCommand::CloseTabWithId(tab_id);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 重命名指定窗格
pub fn rename_pane_with_id<S: AsRef<str>>(pane_id: PaneId, new_name: S)
where
    S: ToString,
{
    let plugin_command = match pane_id {
        PaneId::Terminal(terminal_pane_id) => {
            PluginCommand::RenameTerminalPane(terminal_pane_id, new_name.to_string())
        },
        PaneId::Plugin(plugin_pane_id) => {
            PluginCommand::RenamePluginPane(plugin_pane_id, new_name.to_string())
        },
    };
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 创建一个包含指定窗格 ID 的新标签页
pub fn break_panes_to_new_tab(
    pane_ids: &[PaneId],
    new_tab_name: Option<String>,
    should_change_focus_to_new_tab: bool,
) -> Option<usize> {
    let plugin_command = PluginCommand::BreakPanesToNewTab(
        pane_ids.to_vec(),
        new_tab_name,
        should_change_focus_to_new_tab,
    );
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response =
        ProtobufBreakPanesToNewTabResponse::decode(bytes_from_stdin().unwrap().as_slice()).unwrap();
    BreakPanesToNewTabResponse::try_from(response).unwrap()
}

/// 将窗格 ID 移动到具有指定索引的标签页
pub fn break_panes_to_tab_with_index(
    pane_ids: &[PaneId],
    tab_index: usize,
    should_change_focus_to_new_tab: bool,
) -> Option<usize> {
    let plugin_command = PluginCommand::BreakPanesToTabWithIndex(
        pane_ids.to_vec(),
        tab_index,
        should_change_focus_to_new_tab,
    );
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response =
        ProtobufBreakPanesToTabWithIndexResponse::decode(bytes_from_stdin().unwrap().as_slice())
            .unwrap();
    BreakPanesToTabWithIndexResponse::try_from(response).unwrap()
}

/// 将窗格 ID 移动到具有指定 ID 的标签页
pub fn break_panes_to_tab_with_id(
    pane_ids: &[PaneId],
    tab_id: usize,
    should_change_focus_to_target_tab: bool,
) -> Option<usize> {
    let plugin_command = PluginCommand::BreakPanesToTabWithId(
        pane_ids.to_vec(),
        tab_id as u64,
        should_change_focus_to_target_tab,
    );
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };

    let response =
        ProtobufBreakPanesToTabWithIdResponse::decode(bytes_from_stdin().unwrap().as_slice())
            .unwrap();
    BreakPanesToTabWithIdResponse::try_from(response).unwrap()
}

/// 重新加载此会话中已在运行的插件，可选择性跳过缓存
pub fn reload_plugin_with_id(plugin_id: u32) {
    let plugin_command = PluginCommand::ReloadPlugin(plugin_id);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 重新加载此会话中已在运行的插件，可选择性跳过缓存
pub fn load_new_plugin<S: AsRef<str>>(
    url: S,
    config: BTreeMap<String, String>,
    load_in_background: bool,
    skip_plugin_cache: bool,
) where
    S: ToString,
{
    let plugin_command = PluginCommand::LoadNewPlugin {
        url: url.to_string(),
        config,
        load_in_background,
        skip_plugin_cache,
    };
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 为当前用户重新绑定按键
pub fn rebind_keys(
    keys_to_unbind: Vec<(InputMode, KeyWithModifier)>,
    keys_to_rebind: Vec<(InputMode, KeyWithModifier, Vec<Action>)>,
    write_config_to_disk: bool,
) {
    let plugin_command = PluginCommand::RebindKeys {
        keys_to_rebind,
        keys_to_unbind,
        write_config_to_disk,
    };
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

pub fn change_host_folder(new_host_folder: PathBuf) {
    let plugin_command = PluginCommand::ChangeHostFolder(new_host_folder);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

pub fn set_floating_pane_pinned(pane_id: PaneId, should_be_pinned: bool) {
    let plugin_command = PluginCommand::SetFloatingPanePinned(pane_id, should_be_pinned);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

pub fn stack_panes(pane_ids: Vec<PaneId>) {
    let plugin_command = PluginCommand::StackPanes(pane_ids);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

pub fn change_floating_panes_coordinates(
    pane_ids_and_coordinates: Vec<(PaneId, FloatingPaneCoordinates)>,
) {
    let plugin_command = PluginCommand::ChangeFloatingPanesCoordinates(pane_ids_and_coordinates);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 切换由 pane_id 标识的窗格的无边框状态
///
/// # Arguments
/// * `pane_id` - 要切换的窗格 ID（PaneId::Terminal 或 PaneId::Plugin）
pub fn toggle_pane_borderless(pane_id: PaneId) {
    let plugin_command = PluginCommand::TogglePaneBorderless(pane_id);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 显式设置窗格的无边框状态
///
/// # Arguments
/// * `pane_id` - 窗格 ID（PaneId::Terminal 或 PaneId::Plugin）
/// * `borderless` - true 表示无边框，false 表示有边框
pub fn set_pane_borderless(pane_id: PaneId, borderless: bool) {
    let plugin_command = PluginCommand::SetPaneBorderless(pane_id, borderless);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 设置窗格的默认前景色和/或背景色
///
/// # Arguments
/// * `pane_id` - The ID of the pane (PaneId::Terminal or PaneId::Plugin)
/// * `fg` - 可选的前景色字符串（例如 "#00e000"），None 表示不更改
/// * `bg` - 可选的背景色字符串（例如 "#001a3a"），None 表示不更改
pub fn set_pane_color(pane_id: PaneId, fg: Option<String>, bg: Option<String>) {
    let plugin_command = PluginCommand::SetPaneColor(pane_id, fg, bg);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

pub fn start_web_server() {
    let plugin_command = PluginCommand::StartWebServer;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

pub fn stop_web_server() {
    let plugin_command = PluginCommand::StopWebServer;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

pub fn query_web_server_status() {
    let plugin_command = PluginCommand::QueryWebServerStatus;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

pub fn share_current_session() {
    let plugin_command = PluginCommand::ShareCurrentSession;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

pub fn stop_sharing_current_session() {
    let plugin_command = PluginCommand::StopSharingCurrentSession;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

pub fn group_and_ungroup_panes(
    pane_ids_to_group: Vec<PaneId>,
    pane_ids_to_ungroup: Vec<PaneId>,
    for_all_clients: bool,
) {
    let plugin_command = PluginCommand::GroupAndUngroupPanes(
        pane_ids_to_group,
        pane_ids_to_ungroup,
        for_all_clients,
    );
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

pub fn highlight_and_unhighlight_panes(
    pane_ids_to_highlight: Vec<PaneId>,
    pane_ids_to_unhighlight: Vec<PaneId>,
) {
    let plugin_command =
        PluginCommand::HighlightAndUnhighlightPanes(pane_ids_to_highlight, pane_ids_to_unhighlight);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

pub fn close_multiple_panes(pane_ids: Vec<PaneId>) {
    let plugin_command = PluginCommand::CloseMultiplePanes(pane_ids);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

pub fn float_multiple_panes(pane_ids: Vec<PaneId>) {
    let plugin_command = PluginCommand::FloatMultiplePanes(pane_ids);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

pub fn embed_multiple_panes(pane_ids: Vec<PaneId>) {
    let plugin_command = PluginCommand::EmbedMultiplePanes(pane_ids);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

pub fn set_self_mouse_selection_support(selection_support: bool) {
    let plugin_command = PluginCommand::SetSelfMouseSelectionSupport(selection_support);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

pub fn generate_web_login_token(
    token_label: Option<String>,
    read_only: bool,
) -> Result<String, String> {
    let plugin_command = PluginCommand::GenerateWebLoginToken(token_label, read_only);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
    let create_token_response =
        CreateTokenResponse::decode(bytes_from_stdin().unwrap().as_slice()).unwrap();
    if let Some(error) = create_token_response.error {
        Err(error)
    } else if let Some(token) = create_token_response.token {
        Ok(token)
    } else {
        Err("收到空响应".to_owned())
    }
}

pub fn revoke_web_login_token(token_label: &str) -> Result<(), String> {
    let plugin_command = PluginCommand::RevokeWebLoginToken(token_label.to_owned());
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
    let revoke_token_response =
        RevokeTokenResponse::decode(bytes_from_stdin().unwrap().as_slice()).unwrap();
    if let Some(error) = revoke_token_response.error {
        Err(error)
    } else {
        Ok(())
    }
}

pub fn list_web_login_tokens() -> Result<Vec<(String, String, bool)>, String> {
    // (name, created_at, read_only)
    let plugin_command = PluginCommand::ListWebLoginTokens;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
    let list_tokens_response =
        ListTokensResponse::decode(bytes_from_stdin().unwrap().as_slice()).unwrap();

    if let Some(error) = list_tokens_response.error {
        Err(error)
    } else {
        let tokens_with_info = list_tokens_response
            .tokens
            .iter()
            .zip(list_tokens_response.creation_times.iter())
            .zip(list_tokens_response.read_only_flags.iter())
            .map(|((name, created_at), read_only)| (name.clone(), created_at.clone(), *read_only))
            .collect();
        Ok(tokens_with_info)
    }
}

pub fn revoke_all_web_tokens() -> Result<(), String> {
    let plugin_command = PluginCommand::RevokeAllWebLoginTokens;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
    let revoke_all_web_tokens_response =
        RevokeAllWebTokensResponse::decode(bytes_from_stdin().unwrap().as_slice()).unwrap();
    if let Some(error) = revoke_all_web_tokens_response.error {
        Err(error)
    } else {
        Ok(())
    }
}

pub fn rename_web_token(old_name: &str, new_name: &str) -> Result<(), String> {
    let plugin_command =
        PluginCommand::RenameWebLoginToken(old_name.to_owned(), new_name.to_owned());
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
    let rename_web_token_response =
        RenameWebTokenResponse::decode(bytes_from_stdin().unwrap().as_slice()).unwrap();
    if let Some(error) = rename_web_token_response.error {
        Err(error)
    } else {
        Ok(())
    }
}

pub fn intercept_key_presses() {
    let plugin_command = PluginCommand::InterceptKeyPresses;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

pub fn clear_key_presses_intercepts() {
    let plugin_command = PluginCommand::ClearKeyPressesIntercepts;
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

pub fn replace_pane_with_existing_pane(
    pane_id_to_replace: PaneId,
    existing_pane_id: PaneId,
    suppress_replaced_pane: bool,
) {
    let plugin_command = PluginCommand::ReplacePaneWithExistingPane(
        pane_id_to_replace,
        existing_pane_id,
        suppress_replaced_pane,
    );
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

// 工具函数

#[allow(unused)]
/// 返回与当前活动标签页对应的 `TabInfo`
pub fn get_focused_tab(tab_infos: &Vec<TabInfo>) -> Option<TabInfo> {
    for tab_info in tab_infos {
        if tab_info.active {
            return Some(tab_info.clone());
        }
    }
    return None;
}

#[allow(unused)]
/// 返回与当前活动窗格对应的 `PaneInfo`（忽略插件）
pub fn get_focused_pane(tab_position: usize, pane_manifest: &PaneManifest) -> Option<PaneInfo> {
    let panes = pane_manifest.panes.get(&tab_position);
    if let Some(panes) = panes {
        for pane in panes {
            if pane.is_focused & !pane.is_plugin {
                return Some(pane.clone());
            }
        }
    }
    None
}

pub fn override_layout<L: AsRef<LayoutInfo>>(
    layout_info: L,
    retain_existing_terminal_panes: bool,
    retain_existing_plugin_panes: bool,
    apply_only_to_active_tab: bool,
    context: BTreeMap<String, String>,
) {
    let plugin_command = PluginCommand::OverrideLayout(
        layout_info.as_ref().clone(),
        retain_existing_terminal_panes,
        retain_existing_plugin_panes,
        apply_only_to_active_tab,
        context,
    );
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

// 内部函数

#[doc(hidden)]
pub fn object_from_stdin<T: DeserializeOwned>() -> Result<T> {
    let err_context = || "failed to deserialize object from stdin".to_string();

    let mut json = String::new();
    io::stdin().read_line(&mut json).with_context(err_context)?;
    serde_json::from_str(&json).with_context(err_context)
}

#[doc(hidden)]
pub fn bytes_from_stdin() -> Result<Vec<u8>> {
    let err_context = || "failed to deserialize bytes from stdin".to_string();
    let mut json = String::new();
    io::stdin().read_line(&mut json).with_context(err_context)?;
    serde_json::from_str(&json).with_context(err_context)
}

#[doc(hidden)]
pub fn object_to_stdout(object: &impl Serialize) {
    // TODO: no crashy
    println!("{}", serde_json::to_string(object).unwrap());
}

/// 向此插件的工作器发送消息，更多信息请参阅 [插件工作器](https://zellij.dev/documentation/plugin-api-workers.md)
pub fn post_message_to(plugin_message: PluginMessage) {
    let plugin_command = PluginCommand::PostMessageTo(plugin_message);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 向此插件发送消息，更多信息请参阅 [插件工作器](https://zellij.dev/documentation/plugin-api-workers.md)
pub fn post_message_to_plugin(plugin_message: PluginMessage) {
    let plugin_command = PluginCommand::PostMessageToPlugin(plugin_message);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

pub fn run_action(action: Action, context: BTreeMap<String, String>) {
    // TODO: also accept reference
    let plugin_command = PluginCommand::RunAction(action, context);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 显示指定标签页中的所有浮动窗格，如果 `tab_id` 为 `None` 则显示活动标签页。
///
/// 阻塞直到服务端操作完成。
///
/// 如果浮动窗格已变为可见（状态改变），返回 `Ok(true)`，
/// 如果浮动窗格已经可见（无变化），返回 `Ok(false)`，
/// 如果指定的标签页未找到，返回 `Err(String)`。
pub fn show_floating_panes(tab_id: Option<usize>) -> Result<bool, String> {
    let plugin_command = PluginCommand::ShowFloatingPanes { tab_id };
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
    let response_bytes =
        bytes_from_stdin().map_err(|e| format!("读取响应失败：{:?}", e))?;
    let response = ProtobufShowFloatingPanesResponse::decode(response_bytes.as_slice())
        .map_err(|e| format!("解码响应失败：{}", e))?;
    match response.result {
        Some(show_floating_panes_response::Result::Success(changed)) => Ok(changed),
        Some(show_floating_panes_response::Result::Error(e)) => Err(e),
        None => Err("Empty response".to_string()),
    }
}

/// 隐藏指定标签页中的所有浮动窗格，如果 `tab_id` 为 `None` 则隐藏活动标签页。
///
/// 阻塞直到服务端操作完成。
///
/// 如果浮动窗格已被隐藏（状态改变），返回 `Ok(true)`，
/// 如果浮动窗格已经隐藏（无变化），返回 `Ok(false)`，
/// or `Err(String)` if the specified tab was not found.
pub fn hide_floating_panes(tab_id: Option<usize>) -> Result<bool, String> {
    let plugin_command = PluginCommand::HideFloatingPanes { tab_id };
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
    let response_bytes =
        bytes_from_stdin().map_err(|e| format!("读取响应失败：{:?}", e))?;
    let response = ProtobufHideFloatingPanesResponse::decode(response_bytes.as_slice())
        .map_err(|e| format!("解码响应失败：{}", e))?;
    match response.result {
        Some(hide_floating_panes_response::Result::Success(changed)) => Ok(changed),
        Some(hide_floating_panes_response::Result::Error(e)) => Err(e),
        None => Err("Empty response".to_string()),
    }
}

/// 为窗格设置或更新基于正则表达式的内容高亮。
///
/// `highlights` 中的每个条目都以其 `pattern` 字符串为键。再次
/// 使用相同的模式调用此函数会更新其样式；新的模式将被
/// 添加；本次调用中未出现的模式将被保留。要移除此插件在
/// 该窗格上的所有高亮，请使用 `clear_pane_highlights`。
///
/// 模式匹配由服务端在渲染时针对当前视口执行。
/// 插件从不处理坐标，因此不存在内容更改与高亮应用之间的
/// 竞态条件。
///
/// 当 `on_hover` 为 `true` 且 `tooltip_text` 为 `Some(...)` 时，工具提示
/// 文本显示在窗格边框底部（格式为
/// `" Alt <Click> - {tooltip_text} "`），每当鼠标光标悬停在
/// 高亮区域上方时。
///
/// 需要 `ChangeApplicationState` 权限。
pub fn set_pane_regex_highlights(pane_id: PaneId, highlights: Vec<RegexHighlight>) {
    let plugin_command = PluginCommand::SetPaneRegexHighlights(pane_id, highlights);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

/// 移除此插件在给定窗格上设置的所有正则高亮。
///
/// 同一窗格上其他插件的高亮不受影响。
///
/// 需要 `ChangeApplicationState` 权限。
pub fn clear_pane_highlights(pane_id: PaneId) {
    let plugin_command = PluginCommand::ClearPaneHighlights(pane_id);
    let protobuf_plugin_command: ProtobufPluginCommand = plugin_command.try_into().unwrap();
    object_to_stdout(&protobuf_plugin_command.encode_to_vec());
    unsafe { host_run_plugin_command() };
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "zellij")]
extern "C" {
    fn host_run_plugin_command();
}

#[cfg(not(target_arch = "wasm32"))]
unsafe fn host_run_plugin_command() {}
