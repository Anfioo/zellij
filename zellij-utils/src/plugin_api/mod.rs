pub mod action;
pub mod command;
pub mod event;
pub mod file;
pub mod input_mode;
pub mod key;
pub mod message;
pub mod pane_frame_style;
pub mod pipe_message;
pub mod plugin_command;
pub mod plugin_ids;
pub mod plugin_permission;
pub mod resize;
pub mod style;
// 注意：此代码目前顺序不正常。
// 请参考 [引入此更改的 PR][1] 了解更多原因。
// 简而言之：运行 `cargo release --dry-run` 时，zellij-utils 中的 build-script 由于未知原因
//        未被执行，导致编译失败。为了在此期间能够发布新版本，我们决定暂时静态包含
//        protobuf 插件 API 定义。
//
// [1]: https://github.com/zellij-org/zellij/pull/2711#issuecomment-1695015818
//pub mod generated_api {
//    include!(concat!(env!("OUT_DIR"), "/generated_plugin_api.rs"));
//}
pub mod generated_api {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/prost/generated_plugin_api.rs"
    ));
}
