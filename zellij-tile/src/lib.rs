//! zellij-tile crate 是用于开发 Zellij 插件的 Rust API。
//!
//! 了解更多关于 Zellij 插件的信息：
//! [https://zellij.dev/documentation/plugins](https://zellij.dev/documentation/plugins)
//!
//! ### 本库中的重要内容：
//! - 用于实现插件的 [`ZellijPlugin`] trait，配合
//! [`register_plugin!`](register_plugin) 宏来注册它们。
//! - 代表插件可执行操作的 [命令](shim) 列表。
//! - 插件可订阅的 [`事件`](prelude::Event) 列表
//! - 用于实现后台工作器的 [`ZellijWorker`] trait，配合
//! [`register_worker!`](register_worker) 宏来注册它们
//!
//! ### 完整示例与开发环境
//! 有关可运行的插件示例以及开发环境，请参阅：
//! [https://github.com/zellij-org/rust-plugin-example](https://github.com/zellij-org/rust-plugin-example)
//!
pub mod prelude;
pub mod shim;
pub mod ui_components;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use zellij_utils::data::{Event, PipeMessage};

// use zellij_tile::shim::plugin_api::event::ProtobufEvent;

/// 每个插件应在一个结构体（通常代表插件状态）上实现一次此 trait。该结构体随后应通过
/// [`register_plugin!`](register_plugin) 宏注册。
#[allow(unused_variables)]
pub trait ZellijPlugin: Default {
    /// 插件加载时调用，这是 [`subscribe`](shim::subscribe) 订阅该插件感兴趣事件的好地方。
    fn load(&mut self, configuration: BTreeMap<String, String>) {}
    /// 如果插件订阅了某个 [`事件`](prelude::Event)，将以该事件为参数调用此方法。
    /// 如果插件从此函数返回 `true`，Zellij 将知道需要渲染该插件并调用其 `render` 函数。
    fn update(&mut self, event: Event) -> bool {
        false
    } // 如果需要渲染则返回 true
    /// 当数据通过管道传输到插件时调用，PipeMessage.payload 为 None 表示管道
    /// 已结束
    /// 如果插件从此函数返回 `true`，Zellij 将知道需要渲染该插件并调用其 `render` 函数。
    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        false
    } // 如果需要渲染则返回 true
    /// 在请求渲染的 `update` 之后调用，或在插件需要重新渲染时调用（例如启动时或插件大小改变时）。
    /// `rows` 和 `cols` 值代表插件的"内容大小"（如果用户启用了窗格边框，则不包括周围的边框）。
    fn render(&mut self, rows: usize, cols: usize) {}
}

/// 此 trait 用于创建工作器。插件可使用工作器运行耗时较长的
/// 后台任务，而不会阻塞自身的渲染（例如在等待任务完成时
/// 按需在界面的一部分显示某种加载指示）。
///
/// ## 在插件加载时启动工作器
/// 在结构体（通常代表工作器状态）上实现此 trait，并通过
/// [`register_worker!`](register_worker) 宏注册。
///
/// ## 向工作器发送消息以及回传插件
/// 使用 [`post_message_to`](shim::post_message_to) 方法向工作器发送消息。
/// 使用以下方法将消息从工作器回传到插件：
/// [`post_message_to_plugin`](shim::post_message_to_plugin) 方法（但请确保插件已
/// [`subscribe`](shim::subscribe) 了 [`CustomMessage`](prelude::Event::CustomMessage) 事件
/// ！
#[allow(unused_variables)]
pub trait ZellijWorker<'de>: Default + Serialize + Deserialize<'de> {
    /// 每当插件使用 [`post_message_to`](shim::post_message_to) 方法向工作器发送消息时触发。
    /// 
    fn on_message(&mut self, message: String, payload: String) {}
}

pub const PLUGIN_MISMATCH: &str =
    "插件在从 zellij 接收事件时发生错误。这意味着
插件与当前 zellij 版本不兼容。

最可能的原因是您在运行
自行编译的 zellij 或插件版本。请确保在开发时
同时重新构建插件，以便拾取插件代码的更改。

更多信息请参阅文档：
    https://github.com/zellij-org/zellij/blob/main/CONTRIBUTING.md#building
";

/// 用于注册实现了 [`ZellijPlugin`] trait 的插件。
///
/// 例如：
/// ```rust
/// use zellij_tile::prelude::*;
///
/// #[derive(Default)]
/// pub struct MyPlugin {}
///
/// impl ZellijPlugin for MyPlugin {
///    // ...
/// }
///
/// register_plugin!(MyPlugin);
/// ```
#[macro_export]
macro_rules! register_plugin {
    ($t:ty) => {
        thread_local! {
            static STATE: std::cell::RefCell<$t> = std::cell::RefCell::new(Default::default());
        }

        fn main() {
            // 注册自定义 panic 处理器
            std::panic::set_hook(Box::new(|info| {
                report_panic(info);
            }));
        }

        #[no_mangle]
        fn load() {
            STATE.with(|state| {
                use std::collections::BTreeMap;
                use std::convert::TryFrom;
                use std::convert::TryInto;
                use zellij_tile::shim::plugin_api::action::ProtobufPluginConfiguration;
                use zellij_tile::shim::prost::Message;
                let protobuf_bytes: Vec<u8> = $crate::shim::object_from_stdin().unwrap();
                let protobuf_configuration: ProtobufPluginConfiguration =
                    ProtobufPluginConfiguration::decode(protobuf_bytes.as_slice()).unwrap();
                let plugin_configuration: BTreeMap<String, String> =
                    BTreeMap::try_from(&protobuf_configuration).unwrap();
                state.borrow_mut().load(plugin_configuration);
            });
        }

        #[no_mangle]
        pub fn update() -> bool {
            let err_context = "反序列化事件失败";
            use std::convert::TryInto;
            use zellij_tile::shim::plugin_api::event::ProtobufEvent;
            use zellij_tile::shim::prost::Message;
            STATE.with(|state| {
                let protobuf_bytes: Vec<u8> = $crate::shim::object_from_stdin().unwrap();
                let protobuf_event: ProtobufEvent =
                    ProtobufEvent::decode(protobuf_bytes.as_slice()).unwrap();
                let event = protobuf_event.try_into().unwrap();
                state.borrow_mut().update(event)
            })
        }

        #[no_mangle]
        pub fn pipe() -> bool {
            let err_context = "反序列化管道消息失败";
            use std::convert::TryInto;
            use zellij_tile::shim::plugin_api::pipe_message::ProtobufPipeMessage;
            use zellij_tile::shim::prost::Message;
            STATE.with(|state| {
                let protobuf_bytes: Vec<u8> = $crate::shim::object_from_stdin().unwrap();
                let protobuf_pipe_message: ProtobufPipeMessage =
                    ProtobufPipeMessage::decode(protobuf_bytes.as_slice()).unwrap();
                let pipe_message = protobuf_pipe_message.try_into().unwrap();
                state.borrow_mut().pipe(pipe_message)
            })
        }

        #[no_mangle]
        pub fn render(rows: i32, cols: i32) {
            STATE.with(|state| {
                state.borrow_mut().render(rows as usize, cols as usize);
            });
        }

        #[no_mangle]
        pub fn plugin_version() {
            println!("{}", $crate::prelude::VERSION);
        }
    };
}

/// 用于注册实现了 [`ZellijWorker`] trait 的插件工作器。
///
/// eg.
/// ```rust
/// use zellij_tile::prelude::*;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Default, Serialize, Deserialize)]
/// pub struct FileSearchWorker {}
///
/// impl ZellijWorker<'_> for FileSearchWorker {
///     fn on_message(&mut self, message: String, payload: String) {
///         // ...
///     }
/// }
///
/// register_worker!(
///     FileSearchWorker,
///     file_search_worker, // 将工作器注册为命名空间 "file_search"
///     FILE_SEARCH_WORKER  // 展开为一个保存工作器状态的静态变量
/// );
/// ```
#[macro_export]
macro_rules! register_worker {
    ($worker:ty, $worker_name:ident, $worker_static_name:ident) => {
        // 在静态变量中将工作器状态持久化到内存中
        thread_local! {
            static $worker_static_name: std::cell::RefCell<$worker> = std::cell::RefCell::new(Default::default());
        }
        #[no_mangle]
        pub fn $worker_name() {
            use zellij_tile::shim::plugin_api::message::ProtobufMessage;
            use zellij_tile::shim::prost::Message;
            let worker_display_name = std::stringify!($worker_name);
            let protobuf_bytes: Vec<u8> = $crate::shim::object_from_stdin()
                .unwrap();
            let protobuf_message: ProtobufMessage = ProtobufMessage::decode(protobuf_bytes.as_slice())
                .unwrap();
            let message = protobuf_message.name;
            let payload = protobuf_message.payload;
            $worker_static_name.with(|worker_instance| {
                let mut worker_instance = worker_instance.borrow_mut();
                worker_instance.on_message(message, payload);
            });
         }
    };
}
