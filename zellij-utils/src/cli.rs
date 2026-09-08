//! 用于解析命令行参数的结构和枚举。

use crate::input::options::Options;
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Default, Clone, Parser, Serialize, Deserialize)]
#[clap(version, name = "zellij")]
/// 一个具有布局和插件的终端多路复用器
pub struct CliArgs {
    /// 要加载的配置文件（绝对路径或相对于当前工作目录）
    #[clap(short, long, value_parser)]
    pub config: Option<PathBuf>,

    /// 要加载的配置目录（绝对路径或相对于当前工作目录）
    #[clap(long, value_parser)]
    pub config_dir: Option<PathBuf>,

    /// 数据目录（绝对路径或相对于当前工作目录）
    #[clap(long, value_parser)]
    pub data_dir: Option<PathBuf>,

    /// 最大回滚缓冲区大小（行数）
    #[clap(short, long, value_parser)]
    pub max_panes: Option<usize>,

    /// 调试模式 - 所有错误都会打印完整的错误上下文
    #[clap(long, value_parser)]
    pub debug: bool,

    /// 打印会话名称并退出
    #[clap(long, value_parser)]
    pub print_session_name: bool,

    /// 以字符串形式加载布局（例如 'layout { pane; }'）
    #[clap(short, long, value_parser)]
    pub layout_string: Option<String>,

    /// 启动时要加载的布局（绝对路径或相对于当前工作目录，或布局目录中的名称）
    #[clap(short, long, value_parser)]
    pub layout: Option<PathBuf>,

    /// 启动时要使用的会话名称
    #[clap(short, long, value_parser)]
    pub session: Option<String>,

    /// 附加到最近创建的会话
    #[clap(short, long, value_parser)]
    pub attach: bool,

    /// 创建会话后立即分离
    #[clap(short = 'd', long, value_parser)]
    pub detached: bool,

    /// 要运行的子命令
    #[clap(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Clone, Subcommand, Serialize, Deserialize)]
pub enum Command {
    /// 列出当前正在运行的会话
    ListSessions(ListSessions),
    /// 附加到一个会话
    Sessions(Sessions),
    /// 杀掉特定的会话
    KillSession(KillSession),
    /// 删除特定的会话（包括其布局和历史记录）
    DeleteSession(DeleteSession),
    /// 杀掉所有会话
    KillAllSessions,
    /// 配置选项
    Options(Options),
    /// 生成 shell 补全
    Setup(Setup),
    /// 编辑 zellij 配置
    Edit(Edit),
    /// 保存当前会话
    Save(Save),
    /// 恢复之前保存的会话
    Restore(Restore),
    /// 插件管理
    Plugin(Plugin),
    /// 布局管理
    Layout(Layout),
    /// 主题管理
    Theme(Theme),
    /// 启动 Web 服务器
    StartServer(StartServer),
    /// 停止 Web 服务器
    StopServer,
    /// 列出 Web 会话
    ListWebSessions,
    /// 共享当前会话
    Share(Share),
    /// 取消共享当前会话
    Unshare,
    /// 运行命令并在新窗格中显示输出
    Run(Run),
    /// 管道命令输出到新窗格
    Pipe(Pipe),
}

#[derive(Debug, Default, Clone, Args, Serialize, Deserialize)]
pub struct ListSessions {
    /// 不格式化输出
    #[clap(long, value_parser)]
    pub no_formatting: bool,
    /// 仅打印会话名称
    #[clap(short, long, value_parser)]
    pub short: bool,
    /// 反转排序顺序
    #[clap(short, long, value_parser)]
    pub reverse: bool,
}

#[derive(Debug, Clone, Subcommand, Serialize, Deserialize)]
pub enum Sessions {
    /// 附加到一个会话
    Attach {
        /// 要附加到的会话名称
        #[clap(value_parser)]
        name: Option<String>,
        /// 附加时使用的选项
        #[clap(flatten)]
        options: Option<Box<SessionCommand>>,
        /// 创建会话后立即分离
        #[clap(short = 'd', long, value_parser)]
        detached: bool,
        /// 强制创建新会话（如果不存在）
        #[clap(short, long, value_parser)]
        create: bool,
        /// 如果不存在则创建新会话并使用指定名称
        #[clap(short = 'c', long, value_parser)]
        create_with_name: Option<String>,
    },
    /// 分离当前会话
    Detach,
    /// 重命名当前会话
    Rename {
        /// 新的会话名称
        #[clap(value_parser)]
        name: String,
    },
}

#[derive(Debug, Clone, Subcommand, Serialize, Deserialize)]
pub enum SessionCommand {
    /// 附加时使用的选项
    Options(Options),
}

#[derive(Debug, Default, Clone, Args, Serialize, Deserialize)]
pub struct KillSession {
    /// 要杀掉的会话名称
    #[clap(value_parser)]
    pub name: String,
}

#[derive(Debug, Default, Clone, Args, Serialize, Deserialize)]
pub struct DeleteSession {
    /// 要删除的会话名称
    #[clap(value_parser)]
    pub name: String,
    /// 强制删除（即使会话正在运行）
    #[clap(short, long, value_parser)]
    pub force: bool,
}

#[derive(Debug, Default, Clone, Args, Serialize, Deserialize)]
pub struct Setup {
    /// 将默认配置文件转储到标准输出
    #[clap(long, value_parser)]
    pub dump_config: bool,
    /// 禁用在默认位置加载配置文件
    #[clap(long, value_parser)]
    pub clean: bool,
    /// 检查 zellij 的配置并显示当前使用的目录
    #[clap(long, value_parser)]
    pub check: bool,
    /// 将指定的布局转储到标准输出
    #[clap(long, value_parser)]
    pub dump_layout: Option<String>,
    /// 将指定的交换布局文件转储到标准输出
    #[clap(long, value_parser)]
    pub dump_swap_layout: Option<String>,
    /// 将内置插件转储到 DIR，若未指定则转储到 "DATA DIR"
    #[clap(long, value_name = "DIR", value_parser, exclusive = true, num_args(0..=1))]
    pub dump_plugins: Option<Option<PathBuf>>,
    /// 为指定的 shell 生成补全
    #[clap(long, value_name = "SHELL", value_parser)]
    pub generate_completion: Option<String>,
    /// 为指定的 shell 生成自动启动脚本
    #[clap(long, value_name = "SHELL", value_parser)]
    pub generate_auto_start: Option<String>,
}

#[derive(Debug, Default, Clone, Args, Serialize, Deserialize)]
pub struct Edit {
    /// 要编辑的配置文件类型
    #[clap(value_enum)]
    pub file: Option<EditConfigFile>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
pub enum EditConfigFile {
    /// 编辑主配置文件
    Config,
    /// 编辑布局文件
    Layout,
    /// 编辑主题文件
    Theme,
}

#[derive(Debug, Default, Clone, Args, Serialize, Deserialize)]
pub struct Save {
    /// 保存时使用的会话名称（默认为当前会话名称）
    #[clap(value_parser)]
    pub name: Option<String>,
    /// 覆盖已存在的保存
    #[clap(short, long, value_parser)]
    pub force: bool,
}

#[derive(Debug, Default, Clone, Args, Serialize, Deserialize)]
pub struct Restore {
    /// 要恢复的会话名称
    #[clap(value_parser)]
    pub name: Option<String>,
    /// 恢复后立即分离
    #[clap(short = 'd', long, value_parser)]
    pub detached: bool,
}

#[derive(Debug, Clone, Subcommand, Serialize, Deserialize)]
pub enum Plugin {
    /// 列出已安装的插件
    List,
    /// 安装插件
    Install(PluginInstall),
    /// 卸载插件
    Uninstall(PluginUninstall),
    /// 重新加载插件
    Reload(PluginReload),
}

#[derive(Debug, Default, Clone, Args, Serialize, Deserialize)]
pub struct PluginInstall {
    /// 插件 URL 或路径
    #[clap(value_parser)]
    pub url: String,
    /// 插件名称（覆盖默认名称）
    #[clap(short, long, value_parser)]
    pub name: Option<String>,
}

#[derive(Debug, Default, Clone, Args, Serialize, Deserialize)]
pub struct PluginUninstall {
    /// 要卸载的插件名称
    #[clap(value_parser)]
    pub name: String,
}

#[derive(Debug, Default, Clone, Args, Serialize, Deserialize)]
pub struct PluginReload {
    /// 要重新加载的插件名称
    #[clap(value_parser)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Subcommand, Serialize, Deserialize)]
pub enum Layout {
    /// 列出可用的布局
    List,
    /// 保存当前布局
    Save(LayoutSave),
    /// 删除布局
    Delete(LayoutDelete),
    /// 编辑布局
    Edit(LayoutEdit),
}

#[derive(Debug, Default, Clone, Args, Serialize, Deserialize)]
pub struct LayoutSave {
    /// 布局名称
    #[clap(value_parser)]
    pub name: String,
    /// 覆盖已存在的布局
    #[clap(short, long, value_parser)]
    pub force: bool,
}

#[derive(Debug, Default, Clone, Args, Serialize, Deserialize)]
pub struct LayoutDelete {
    /// 要删除的布局名称
    #[clap(value_parser)]
    pub name: String,
}

#[derive(Debug, Default, Clone, Args, Serialize, Deserialize)]
pub struct LayoutEdit {
    /// 要编辑的布局名称
    #[clap(value_parser)]
    pub name: String,
}

#[derive(Debug, Clone, Subcommand, Serialize, Deserialize)]
pub enum Theme {
    /// 列出可用的主题
    List,
    /// 切换主题
    Switch(ThemeSwitch),
}

#[derive(Debug, Default, Clone, Args, Serialize, Deserialize)]
pub struct ThemeSwitch {
    /// 要切换到的主题名称
    #[clap(value_parser)]
    pub name: String,
}

#[derive(Debug, Default, Clone, Args, Serialize, Deserialize)]
pub struct StartServer {
    /// Web 服务器 IP 地址
    #[clap(long, value_parser)]
    pub ip: Option<String>,
    /// Web 服务器端口
    #[clap(short, long, value_parser)]
    pub port: Option<u16>,
    /// 启用 HTTPS
    #[clap(long, value_parser)]
    pub https: bool,
    /// 证书文件路径
    #[clap(long, value_parser)]
    pub cert: Option<PathBuf>,
    /// 密钥文件路径
    #[clap(long, value_parser)]
    pub key: Option<PathBuf>,
}

#[derive(Debug, Default, Clone, Args, Serialize, Deserialize)]
pub struct Share {
    /// 共享时使用的名称
    #[clap(value_parser)]
    pub name: Option<String>,
}

#[derive(Debug, Default, Clone, Args, Serialize, Deserialize)]
pub struct Run {
    /// 要运行的命令
    #[clap(trailing_var_arg = true, value_parser)]
    pub command: Vec<String>,
    /// 在新标签页中运行
    #[clap(short, long, value_parser)]
    pub new_tab: bool,
    /// 在浮动窗格中运行
    #[clap(short, long, value_parser)]
    pub floating: bool,
    /// 窗格名称
    #[clap(short, long, value_parser)]
    pub name: Option<String>,
    /// 工作目录
    #[clap(short, long, value_parser)]
    pub cwd: Option<PathBuf>,
    /// 关闭后保持窗格打开
    #[clap(long, value_parser)]
    pub hold_on_close: bool,
    /// 启动后立即关闭
    #[clap(long, value_parser)]
    pub close_on_exit: bool,
    /// 启动后立即全屏
    #[clap(long, value_parser)]
    pub start_fullscreen: bool,
}

#[derive(Debug, Default, Clone, Args, Serialize, Deserialize)]
pub struct Pipe {
    /// 要管道的命令
    #[clap(trailing_var_arg = true, value_parser)]
    pub command: Vec<String>,
    /// 管道名称
    #[clap(short, long, value_parser)]
    pub name: Option<String>,
    /// 会话名称
    #[clap(short, long, value_parser)]
    pub session: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_args_debug() {
        let args = CliArgs::try_parse_from(["zellij", "--debug"]).unwrap();
        assert!(args.debug);
    }

    #[test]
    fn test_cli_args_session() {
        let args = CliArgs::try_parse_from(["zellij", "--session", "test"]).unwrap();
        assert_eq!(args.session, Some("test".to_string()));
    }

    #[test]
    fn test_cli_args_layout() {
        let args = CliArgs::try_parse_from(["zellij", "--layout", "default"]).unwrap();
        assert_eq!(args.layout, Some(PathBuf::from("default")));
    }

    #[test]
    fn test_cli_args_config() {
        let args = CliArgs::try_parse_from(["zellij", "--config", "/tmp/config.kdl"]).unwrap();
        assert_eq!(args.config, Some(PathBuf::from("/tmp/config.kdl")));
    }

    #[test]
    fn test_command_list_sessions() {
        let args = CliArgs::try_parse_from(["zellij", "list-sessions"]).unwrap();
        assert!(matches!(args.command, Some(Command::ListSessions(_))));
    }

    #[test]
    fn test_command_kill_session() {
        let args = CliArgs::try_parse_from(["zellij", "kill-session", "test"]).unwrap();
        assert!(matches!(args.command, Some(Command::KillSession(_))));
    }

    #[test]
    fn test_command_options() {
        let args = CliArgs::try_parse_from(["zellij", "options", "--simplified-ui"]).unwrap();
        assert!(matches!(args.command, Some(Command::Options(_))));
    }

    #[test]
    fn test_command_setup() {
        let args = CliArgs::try_parse_from(["zellij", "setup", "--check"]).unwrap();
        assert!(matches!(args.command, Some(Command::Setup(_))));
    }

    #[test]
    fn test_command_attach() {
        let args = CliArgs::try_parse_from(["zellij", "sessions", "attach", "test"]).unwrap();
        assert!(matches!(args.command, Some(Command::Sessions(Sessions::Attach { .. }))));
    }

    #[test]
    fn test_command_detach() {
        let args = CliArgs::try_parse_from(["zellij", "sessions", "detach"]).unwrap();
        assert!(matches!(args.command, Some(Command::Sessions(Sessions::Detach))));
    }

    #[test]
    fn test_command_rename_session() {
        let args = CliArgs::try_parse_from(["zellij", "sessions", "rename", "new-name"]).unwrap();
        assert!(matches!(args.command, Some(Command::Sessions(Sessions::Rename { .. }))));
    }

    #[test]
    fn test_command_run() {
        let args = CliArgs::try_parse_from(["zellij", "run", "ls", "-la"]).unwrap();
        assert!(matches!(args.command, Some(Command::Run(_))));
    }

    #[test]
    fn test_command_pipe() {
        let args = CliArgs::try_parse_from(["zellij", "pipe", "echo", "hello"]).unwrap();
        assert!(matches!(args.command, Some(Command::Pipe(_))));
    }

    #[test]
    fn test_cli_args_verify() {
        CliArgs::command().debug_assert();
    }
}
