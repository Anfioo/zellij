use crate::data::{Direction, InputMode, Resize, UnblockCondition};
use crate::setup::Setup;
use crate::{
    consts::{ZELLIJ_CONFIG_DIR_ENV, ZELLIJ_CONFIG_FILE_ENV},
    input::{
        layout::PluginUserConfiguration,
        options::{Options, PaneFrameStyle},
    },
};
use clap::builder::styling::{AnsiColor, Color, Style, Styles};
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::path::PathBuf;
use url::Url;

const fn ansi(color: AnsiColor) -> Style {
    Style::new().fg_color(Some(Color::Ansi(color)))
}

const CLI_STYLES: Styles = Styles::styled()
    .header(ansi(AnsiColor::Yellow))
    .usage(ansi(AnsiColor::Yellow))
    .literal(ansi(AnsiColor::Green))
    .placeholder(Style::new())
    .error(ansi(AnsiColor::Red))
    .valid(ansi(AnsiColor::Green))
    .invalid(ansi(AnsiColor::Yellow));

fn validate_session(name: &str) -> Result<String, String> {
    #[cfg(unix)]
    {
        use crate::consts::ZELLIJ_SOCK_MAX_LENGTH;

        let mut socket_path = crate::consts::ZELLIJ_SOCK_DIR.clone();
        socket_path.push(name);

        if socket_path.as_os_str().len() >= ZELLIJ_SOCK_MAX_LENGTH {
            // socket 路径必须小于 108 字节
            let available_length = ZELLIJ_SOCK_MAX_LENGTH
                .saturating_sub(socket_path.as_os_str().len())
                .saturating_sub(1);

            return Err(format!(
                "session name must be less than {} characters",
                available_length
            ));
        };
    };

    Ok(name.to_owned())
}

#[derive(Parser, Default, Debug, Clone, Serialize, Deserialize)]
#[clap(
    version,
    name = "zellij",
    about = "A terminal workspace with batteries included",
    styles = CLI_STYLES,
    args_override_self = true
)]
pub struct CliArgs {
    /// 屏幕上最多显示的窗格数，注意：打开更多窗格会关闭旧窗格
    #[clap(long, value_parser)]
    pub max_panes: Option<usize>,

    /// 更改 zellij 查找插件的位置
    #[clap(long, value_parser, overrides_with = "data_dir")]
    pub data_dir: Option<PathBuf>,

    /// 运行服务端并监听指定的 socket 路径
    #[clap(long, value_parser, hide = true, overrides_with = "server")]
    pub server: Option<PathBuf>,

    /// 指定新会话的名称
    #[clap(long, short, overrides_with = "session", value_parser = validate_session)]
    pub session: Option<String>,

    /// 布局目录中预定义布局的名称，或布局文件的路径
    /// 如果处于会话中（或使用 --session 标志），将作为新标签页添加到该会话，
    /// 否则将启动一个新会话
    #[clap(short, long, value_parser, overrides_with = "layout")]
    pub layout: Option<PathBuf>,

    /// 直接使用的原始 KDL 布局字符串（而不是文件路径）
    /// 如果处于会话中（或使用 --session 标志），将作为新标签页添加到该会话，
    /// 否则将启动一个新会话
    #[clap(long, value_parser, conflicts_with_all = &["layout", "new_session_with_layout"])]
    pub layout_string: Option<String>,

    /// 布局目录中预定义布局的名称，或布局文件的路径
    /// 即使处于已有会话中，也始终启动一个新会话
    #[clap(short, long, value_parser, overrides_with = "new_session_with_layout")]
    pub new_session_with_layout: Option<PathBuf>,

    /// 更改 zellij 查找配置文件的位置
    #[clap(short, long, overrides_with = "config", env = ZELLIJ_CONFIG_FILE_ENV, value_parser)]
    pub config: Option<PathBuf>,

    /// 更改 zellij 查找配置目录的位置
    #[clap(long, overrides_with = "config_dir", env = ZELLIJ_CONFIG_DIR_ENV, value_parser)]
    pub config_dir: Option<PathBuf>,

    #[clap(subcommand)]
    pub command: Option<Command>,

    /// 指定输出额外的调试信息
    #[clap(short, long, value_parser)]
    pub debug: bool,
}

impl CliArgs {
    pub fn is_setup_clean(&self) -> bool {
        if let Some(Command::Setup(ref setup)) = &self.command {
            if setup.clean {
                return true;
            }
        }
        false
    }
    pub fn options(&self) -> Option<Options> {
        if let Some(Command::Options(options)) = &self.command {
            return Some(options.clone());
        }
        None
    }
}

#[derive(Debug, Subcommand, Clone, Serialize, Deserialize)]
pub enum Command {
    /// 更改 zellij 的行为
    #[clap(name = "options", value_parser)]
    Options(Options),

    /// 设置 zellij 并检查其配置
    #[clap(name = "setup", value_parser)]
    Setup(Setup),

    /// 运行 Web 服务器以提供终端会话服务
    #[clap(name = "web", value_parser)]
    Web(WebCli),

    /// 向特定会话发送操作
    #[clap(visible_alias = "ac")]
    #[clap(subcommand)]
    Action(Box<CliAction>),

    /// 查看现有 zellij 会话
    #[clap(flatten)]
    Sessions(Sessions),

    /// 订阅窗格渲染更新（视口和滚动缓冲）
    #[clap(override_usage(
        "zellij [--session <OTHER SESSION NAME>] subscribe [OPTIONS] --pane-id..."
    ))]
    Subscribe(SubscribeCli),
}

#[derive(Debug, Parser, Clone, Serialize, Deserialize)]
pub struct SubscribeCli {
    /// 要订阅的窗格 ID（例如 terminal_1、plugin_2，或如 1 这样的裸数字）
    #[clap(
        short,
        long,
        required = true,
        num_args(1..)
    )]
    pub pane_id: Vec<String>,

    /// 在初始交付中包含回滚缓冲行。
    /// 裸 --scrollback = 全部回滚，--scrollback N = 最后 N 行。
    #[clap(
        short,
        long,
        default_missing_value = "0",
        num_args(0..=1)
    )]
    pub scrollback: Option<usize>,

    /// 输出格式
    #[clap(short, long, default_value = "raw", value_enum)]
    pub format: SubscribeFormat,

    /// 在输出中保留 ANSI 样式
    #[clap(long)]
    pub ansi: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum)]
pub enum SubscribeFormat {
    Raw,
    Json,
}

#[derive(Debug, Clone, Args, Serialize, Deserialize)]
pub struct WebCli {
    /// 启动服务端（除非指定了其他参数，否则为默认操作）
    #[clap(long, value_parser, display_order = 1)]
    pub start: bool,

    /// 停止服务端
    #[clap(long, value_parser, exclusive(true), display_order = 2)]
    pub stop: bool,

    /// 获取服务端状态
    #[clap(long, value_parser, conflicts_with("start"), display_order = 3)]
    pub status: bool,

    /// 状态检查的超时秒数（默认：30）
    #[clap(long, value_parser, requires = "status", display_order = 4)]
    pub timeout: Option<u64>,

    /// 在后台运行服务端
    #[clap(
        short,
        long,
        value_parser,
        conflicts_with_all(&["stop", "status", "create_token", "revoke_token", "revoke_all_tokens"]),
        display_order = 5
    )]
    pub daemonize: bool,
    /// 等待服务器启动的超时秒数（默认：10）。
    /// 仅在 Windows 上使用，此时守护化的服务器通过 TCP 轮询。
    /// 在 Unix 上，启动信令使用管道，此选项会被忽略。
    #[clap(long, value_parser, display_order = 6)]
    pub server_startup_timeout: Option<u64>,
    /// 为 Web 界面创建一个登录令牌，只会显示一次，之后无法
    /// 再获取。返回令牌名称和令牌本身。
    #[clap(long, value_parser, exclusive(true), display_order = 7)]
    pub create_token: bool,
    /// 令牌的可选名称
    #[clap(long, value_parser, value_name = "TOKEN_NAME", display_order = 8)]
    pub token_name: Option<String>,
    /// 创建只读登录令牌（只能以观察者身份附加到现有会话）
    #[clap(long, value_parser, exclusive(true), display_order = 9)]
    pub create_read_only_token: bool,
    /// 按名称撤销登录令牌
    #[clap(
        long,
        value_parser,
        exclusive(true),
        value_name = "TOKEN NAME",
        display_order = 10
    )]
    pub revoke_token: Option<String>,
    /// 撤销所有登录令牌
    #[clap(long, value_parser, exclusive(true), display_order = 11)]
    pub revoke_all_tokens: bool,
    /// 列出令牌名称及其创建日期（无法显示实际令牌）
    #[clap(long, value_parser, exclusive(true), display_order = 12)]
    pub list_tokens: bool,
    /// 本地监听的 IP 地址（默认为 127.0.0.1）
    #[clap(
        long,
        value_parser,
        conflicts_with_all(&["stop", "create_token", "revoke_token", "revoke_all_tokens"]),
        display_order = 13
    )]
    pub ip: Option<IpAddr>,
    /// 本地监听的端口（默认为 8082）
    #[clap(
        long,
        value_parser,
        conflicts_with_all(&["stop", "create_token", "revoke_token", "revoke_all_tokens"]),
        display_order = 14
    )]
    pub port: Option<u16>,
    /// SSL 证书的路径（若未监听 127.0.0.1 则为必填）
    #[clap(
        long,
        value_parser,
        conflicts_with_all(&["stop", "status", "create_token", "revoke_token", "revoke_all_tokens"]),
        display_order = 15
    )]
    pub cert: Option<PathBuf>,
    /// SSL 密钥的路径（若未监听 127.0.0.1 则为必填）
    #[clap(
        long,
        value_parser,
        conflicts_with_all(&["stop", "status", "create_token", "revoke_token", "revoke_all_tokens"]),
        display_order = 16
    )]
    pub key: Option<PathBuf>,
}

impl WebCli {
    pub fn get_start(&self) -> bool {
        self.start
            || !(self.stop
                || self.status
                || self.create_token
                || self.create_read_only_token
                || self.revoke_token.is_some()
                || self.revoke_all_tokens
                || self.list_tokens)
    }
}

#[derive(Debug, Subcommand, Clone, Serialize, Deserialize)]
pub enum SessionCommand {
    /// 更改 zellij 的行为
    #[clap(name = "options")]
    Options(Options),
}

#[derive(Debug, Subcommand, Clone, Serialize, Deserialize)]
pub enum Sessions {
    /// 列出活动会话
    #[clap(visible_alias = "ls")]
    ListSessions {
        /// 不给列表添加颜色和格式（便于解析）
        #[clap(short, long)]
        no_formatting: bool,

        /// 仅打印会话名称
        #[clap(short, long)]
        short: bool,

        /// 按相反顺序列出会话（默认为升序）
        #[clap(short, long)]
        reverse: bool,
    },
    /// 列出现有插件别名
    #[clap(visible_alias = "la")]
    ListAliases,
    /// 附加到会话
    #[clap(visible_alias = "a")]
    Attach {
        /// 要附加到的会话名称。
        #[clap(value_parser)]
        session_name: Option<String>,

        /// 若会话不存在则创建它。
        #[clap(short, long, value_parser)]
        create: bool,

        /// 若会话不存在，则在后台创建一个分离的会话
        #[clap(short('b'), long, value_parser)]
        create_background: bool,

        /// 活动会话按创建日期排序后的索引编号。
        #[clap(long, value_parser)]
        index: Option<usize>,

        /// 更改 zellij 的行为
        #[clap(subcommand, name = "options")]
        options: Option<Box<SessionCommand>>,

        /// 若恢复已死亡的会话，在启动时立即运行其所有命令
        #[clap(short, long)]
        force_run_commands: bool,

        /// 远程会话的认证令牌
        #[clap(short('t'), long, value_parser)]
        token: Option<String>,

        /// 保存会话以便自动重新认证（4 周）
        #[clap(short('r'), long, value_parser)]
        remember: bool,

        /// 在连接前删除已保存的会话
        #[clap(long, value_parser)]
        forget: bool,

        /// 用于验证远程服务器的自定义 CA 证书（PEM 格式）路径
        #[clap(long, value_name = "FILE", value_parser)]
        ca_cert: Option<PathBuf>,

        /// 跳过 TLS 证书校验（危险——仅用于开发）
        #[clap(long, value_parser)]
        insecure: bool,

        /// 会话创建时在第一个窗格中运行的命令
        #[clap(value_parser, last(true))]
        initial_command: Vec<String>,

        /// 初始命令退出时立即关闭其窗格
        #[clap(long, requires("initial_command"))]
        close_on_exit: bool,

        /// 以挂起状态启动初始命令，仅在你首次按回车后才运行
        #[clap(long, requires("initial_command"))]
        start_suspended: bool,
    },

    /// 观察会话（只读）
    #[clap(visible_alias = "w")]
    Watch {
        /// 要观察的会话名称
        #[clap(value_parser)]
        session_name: Option<String>,
    },

    /// 终止指定会话
    #[clap(visible_alias = "k")]
    KillSession {
        /// 目标会话名称
        #[clap(value_parser)]
        target_session: Option<String>,
    },

    /// 删除指定会话
    #[clap(visible_alias = "d")]
    DeleteSession {
        /// 目标会话名称
        #[clap(value_parser)]
        target_session: Option<String>,
        /// 删除前若会话正在运行则先终止它
        #[clap(short, long)]
        force: bool,
    },

    /// 终止所有会话
    #[clap(visible_alias = "ka")]
    KillAllSessions {
        /// 对提示自动回答是
        #[clap(short, long, value_parser)]
        yes: bool,
    },

    /// 删除所有会话
    #[clap(visible_alias = "da")]
    DeleteAllSessions {
        /// 对提示自动回答是
        #[clap(short, long, value_parser)]
        yes: bool,
        /// 删除前若会话正在运行则先终止它们
        #[clap(short, long)]
        force: bool,
    },

    /// 在新窗格中运行命令
    /// 返回：创建的窗格 ID（格式：terminal_<id>）
    #[clap(visible_alias = "r")]
    Run {
        /// 要运行的命令
        #[clap(last(true), required(true))]
        command: Vec<String>,

        /// 打开新窗格的方向
        #[clap(short, long, value_parser, conflicts_with("floating"))]
        direction: Option<Direction>,

        /// 更改新窗格的工作目录
        #[clap(long, value_parser)]
        cwd: Option<PathBuf>,

        /// 以浮动模式打开新窗格
        #[clap(short, long)]
        floating: bool,

        /// 在当前窗格位置打开新窗格，并暂时挂起当前窗格
        #[clap(short, long, conflicts_with("floating"), conflicts_with("direction"))]
        in_place: bool,

        /// 关闭被替换的窗格而不是挂起它（仅与 --in-place 一起使用有效）
        #[clap(long, requires("in_place"))]
        close_replaced_pane: bool,

        /// 新窗格名称
        #[clap(short, long, value_parser)]
        name: Option<String>,

        /// 命令退出时立即关闭窗格
        #[clap(short, long)]
        close_on_exit: bool,

        /// 以挂起状态启动命令，仅在你首次按回车后才运行
        #[clap(short, long)]
        start_suspended: bool,

        /// 窗格浮动时的 x 坐标，可以是裸整数（如 1）或百分比（如 10%）
        #[clap(short, long, requires("floating"))]
        x: Option<String>,
        /// 窗格浮动时的 y 坐标，可以是裸整数（如 1）或百分比（如 10%）
        #[clap(short, long, requires("floating"))]
        y: Option<String>,
        /// 窗格浮动时的宽度，可以是裸整数（如 1）或百分比（如 10%）
        #[clap(long, requires("floating"))]
        width: Option<String>,
        /// 窗格浮动时的高度，可以是裸整数（如 1）或百分比（如 10%）
        #[clap(long, requires("floating"))]
        height: Option<String>,
        /// 是否固定浮动窗格使其始终置顶
        #[clap(long, requires("floating"))]
        pinned: Option<bool>,
        #[clap(long, conflicts_with("floating"), conflicts_with("direction"))]
        stacked: bool,
        /// 阻塞直到命令结束且其窗格已关闭
        #[clap(long)]
        blocking: bool,

        /// 阻塞直到命令成功退出（退出状态 0）或其窗格已关闭
        #[clap(
            long,
            conflicts_with("blocking"),
            conflicts_with("block_until_exit_failure"),
            conflicts_with("block_until_exit")
        )]
        block_until_exit_success: bool,

        /// 阻塞直到命令以失败退出（非零退出状态）或其窗格已被
        /// 关闭
        #[clap(
            long,
            conflicts_with("blocking"),
            conflicts_with("block_until_exit_success"),
            conflicts_with("block_until_exit")
        )]
        block_until_exit_failure: bool,

        /// 阻塞直到命令退出（无论退出状态如何）或其窗格已关闭
        #[clap(
            long,
            conflicts_with("blocking"),
            conflicts_with("block_until_exit_success"),
            conflicts_with("block_until_exit_failure")
        )]
        block_until_exit: bool,
        /// 若设置，将在当前窗格附近打开新窗格，而不是跟随用户的焦点
        #[clap(long)]
        near_current_pane: bool,
        #[clap(
            long,
            help = "if set, will open the pane without changing the focus of any client, placing it relative to the pane the command was issued from"
        )]
        no_focus: bool,
        /// 以无边框方式启动此窗格（警告：将无法用
        /// 鼠标移动）
        #[clap(short, long, value_parser)]
        borderless: Option<bool>,
        /// 按 ID 定位指定标签页
        #[clap(
            long,
            value_parser,
            conflicts_with("near_current_pane"),
            conflicts_with("in_place")
        )]
        tab_id: Option<usize>,
    },
    /// 加载插件
    /// 返回：创建的窗格 ID（格式：plugin_<id>）
    #[clap(visible_alias = "p")]
    Plugin {
        /// 插件 URL，可以以 http(s)、file: 或 zellij: 开头
        #[clap(last(true), required(true))]
        url: String,

        /// 插件配置
        #[clap(short, long, value_parser)]
        configuration: Option<PluginUserConfiguration>,

        /// 以浮动模式打开新窗格
        #[clap(short, long)]
        floating: bool,

        /// 在当前窗格位置打开新窗格，并暂时挂起当前窗格
        #[clap(short, long, conflicts_with("floating"))]
        in_place: bool,

        /// 关闭被替换的窗格而不是挂起它（仅与 --in-place 一起使用有效）
        #[clap(long, requires("in_place"))]
        close_replaced_pane: bool,

        /// 跳过内存和硬盘缓存并强制重新编译插件（适合开发）
        #[clap(short, long)]
        skip_plugin_cache: bool,
        /// 窗格浮动时的 x 坐标，可以是裸整数（如 1）或百分比（如 10%）
        #[clap(short, long, requires("floating"))]
        x: Option<String>,
        /// 窗格浮动时的 y 坐标，可以是裸整数（如 1）或百分比（如 10%）
        #[clap(short, long, requires("floating"))]
        y: Option<String>,
        /// 窗格浮动时的宽度，可以是裸整数（如 1）或百分比（如 10%）
        #[clap(long, requires("floating"))]
        width: Option<String>,
        /// 窗格浮动时的高度，可以是裸整数（如 1）或百分比（如 10%）
        #[clap(long, requires("floating"))]
        height: Option<String>,
        /// 是否固定浮动窗格使其始终置顶
        #[clap(long, requires("floating"))]
        pinned: Option<bool>,
        #[clap(
            long,
            help = "if set, will open the plugin pane without changing the focus of any client, placing it relative to the pane the command was issued from"
        )]
        no_focus: bool,
        /// 以无边框方式启动此窗格（警告：将无法用
        /// 鼠标移动）
        #[clap(short, long, value_parser)]
        borderless: Option<bool>,
        /// 按 ID 定位指定标签页
        #[clap(long, value_parser, conflicts_with("in_place"))]
        tab_id: Option<usize>,
    },
    /// 使用默认 $EDITOR / $VISUAL 编辑文件
    /// 返回：创建的窗格 ID（格式：terminal_<id>）
    #[clap(visible_alias = "e")]
    Edit {
        file: PathBuf,

        /// 在指定的行号处打开文件
        #[clap(short, long, value_parser)]
        line_number: Option<usize>,

        /// 打开新窗格的方向
        #[clap(short, long, value_parser, conflicts_with("floating"))]
        direction: Option<Direction>,

        /// 在当前窗格位置打开新窗格，并暂时挂起当前窗格
        #[clap(short, long, conflicts_with("floating"), conflicts_with("direction"))]
        in_place: bool,

        /// 关闭被替换的窗格而不是挂起它（仅与 --in-place 一起使用有效）
        #[clap(long, requires("in_place"))]
        close_replaced_pane: bool,

        /// 以浮动模式打开新窗格
        #[clap(short, long)]
        floating: bool,

        /// 更改编辑器的工作目录
        #[clap(long, value_parser)]
        cwd: Option<PathBuf>,
        /// 窗格浮动时的 x 坐标，可以是裸整数（如 1）或百分比（如 10%）
        #[clap(short, long, requires("floating"))]
        x: Option<String>,
        /// 窗格浮动时的 y 坐标，可以是裸整数（如 1）或百分比（如 10%）
        #[clap(short, long, requires("floating"))]
        y: Option<String>,
        /// 窗格浮动时的宽度，可以是裸整数（如 1）或百分比（如 10%）
        #[clap(long, requires("floating"))]
        width: Option<String>,
        /// 窗格浮动时的高度，可以是裸整数（如 1）或百分比（如 10%）
        #[clap(long, requires("floating"))]
        height: Option<String>,
        /// 是否固定浮动窗格使其始终置顶
        #[clap(long, requires("floating"))]
        pinned: Option<bool>,
        /// 若设置，将在当前窗格附近打开新窗格，而不是跟随用户的焦点
        #[clap(long)]
        near_current_pane: bool,
        #[clap(
            long,
            help = "if set, will open the pane without changing the focus of any client, placing it relative to the pane the command was issued from"
        )]
        no_focus: bool,
        /// 以无边框方式启动此窗格（警告：将无法用
        /// 鼠标移动）
        #[clap(short, long, value_parser)]
        borderless: Option<bool>,
        /// 按 ID 定位指定标签页
        #[clap(
            long,
            value_parser,
            conflicts_with("near_current_pane"),
            conflicts_with("in_place")
        )]
        tab_id: Option<usize>,
    },
    /// 向一个或多个插件发送数据，若未运行则启动它们。
    #[clap(override_usage(
r#"
zellij pipe [OPTIONS] [--] <PAYLOAD>

* Send data to a specific plugin:

zellij pipe --plugin file:/path/to/my/plugin.wasm --name my_pipe_name -- my_arbitrary_data

* To all running plugins (that are listening):

zellij pipe --name my_pipe_name -- my_arbitrary_data

* Pipe data into this command's STDIN and get output from the plugin on this command's STDOUT

tail -f /tmp/my-live-logfile | zellij pipe --name logs --plugin https://example.com/my-plugin.wasm | wc -l
"#))]
    Pipe {
        /// 管道名称
        #[clap(short, long, value_parser, display_order(1))]
        name: Option<String>,
        /// 通过此管道发送的数据（若为空，将监听 STDIN）
        payload: Option<String>,

        #[clap(short, long, value_parser, display_order(2))]
        /// 管道的参数
        args: Option<PluginUserConfiguration>, // TODO：我们可能不想重复使用
        // PluginUserConfiguration
        /// 该管道指向的插件 url（例如 file:/tmp/my-plugin.wasm）；若未指定，
        /// 将发送给所有插件；若已指定但未运行，插件将被启动
        #[clap(short, long, value_parser, display_order(3))]
        plugin: Option<String>,
        /// 插件配置（注意：在确定管道目的地时，配置不同的同一插件会被视为
        /// 不同的插件）
        #[clap(short('c'), long, value_parser, display_order(4))]
        plugin_configuration: Option<PluginUserConfiguration>,
    },
}

#[derive(Debug, Subcommand, Clone, Serialize, Deserialize)]
pub enum CliAction {
    /// 向终端写入字节。
    Write {
        bytes: Vec<u8>,
        /// 窗格的 pane_id，例如 terminal_1、plugin_2 或 3（等价于 terminal_3）
        #[clap(short, long, value_parser)]
        pane_id: Option<String>,
    },
    /// 向终端写入字符。
    WriteChars {
        chars: String,
        /// 窗格的 pane_id，例如 terminal_1、plugin_2 或 3（等价于 terminal_3）
        #[clap(short, long, value_parser)]
        pane_id: Option<String>,
    },
    /// 向终端粘贴文本（使用括号粘贴模式）。
    Paste {
        chars: String,
        /// 窗格的 pane_id，例如 terminal_1、plugin_2 或 3（等价于 terminal_3）
        #[clap(short, long, value_parser)]
        pane_id: Option<String>,
    },
    /// 向终端发送一个或多个按键（例如 "Ctrl a"、"F1"、"Alt Shift b"）
    SendKeys {
        /// 以空格分隔的字符串形式发送按键
        #[clap(value_parser, required = true)]
        keys: Vec<String>,

        /// 窗格的 pane_id，例如 terminal_1、plugin_2 或 3（等价于 terminal_3）
        #[clap(short, long, value_parser)]
        pane_id: Option<String>,
    },
    /// 在 [左|下|上|右] 边界 [增加|减少] 聚焦窗格区域。
    Resize {
        resize: Resize,
        direction: Option<Direction>,
        /// 按 ID 定位指定窗格（例如 terminal_1、plugin_2 或 3）
        #[clap(short, long, value_parser)]
        pane_id: Option<String>,
    },
    /// 将焦点切换到下一个窗格
    FocusNextPane,
    /// 将焦点切换到上一个窗格
    FocusPreviousPane,
    /// 按 ID 聚焦指定窗格
    FocusPaneId {
        /// 窗格的 pane_id，例如 terminal_1、plugin_2 或 3
        pane_id: String,
    },
    /// 将焦点切换到上一个聚焦的框架
    FocusLastPane,
    /// 沿指定方向移动聚焦窗格。[right|left|up|down]
    MoveFocus {
        direction: Direction,
    },
    /// 按指定方向将焦点移动到窗格或标签页（如果在屏幕边缘）
    /// [right|left|up|down]
    MoveFocusOrTab {
        direction: Direction,
    },
    /// 按指定方向改变聚焦窗格的位置，或向前旋转
    /// [right|left|up|down]
    MovePane {
        direction: Option<Direction>,
        /// 按 ID 定位指定窗格（例如 terminal_1、plugin_2 或 3）
        #[clap(short, long, value_parser)]
        pane_id: Option<String>,
    },
    /// 向后旋转上一个窗格的位置
    MovePaneBackwards {
        /// 按 ID 定位指定窗格（例如 terminal_1、plugin_2 或 3）
        #[clap(short, long, value_parser)]
        pane_id: Option<String>,
    },
    /// 清除聚焦窗格的所有缓冲区
    Clear {
        /// 按 ID 定位指定窗格（例如 terminal_1、plugin_2 或 3）
        #[clap(short, long, value_parser)]
        pane_id: Option<String>,
    },
    /// 将窗格的视口及可选的滚动缓冲转储到文件或 STDOUT
    DumpScreen {
        /// 转储窗格内容的文件路径。若省略，则打印到 STDOUT。
        #[clap(long, value_parser)]
        path: Option<PathBuf>,

        /// 转储窗格的全部滚动缓冲
        #[clap(short, long)]
        full: bool,

        /// 窗格的 pane_id，例如 terminal_1、plugin_2 或 3（等价于 terminal_3）。若未指定，则转储聚焦窗格。
        #[clap(short, long, value_parser)]
        pane_id: Option<String>,

        /// 在转储输出中保留 ANSI 样式
        #[clap(short, long)]
        ansi: bool,
    },
    /// 将当前布局转储到 stdout
    DumpLayout,
    /// 立即将当前会话状态保存到磁盘
    SaveSession,
    /// 在默认编辑器中打开窗格的滚动缓冲
    EditScrollback {
        /// 按 ID 定位指定窗格（例如 terminal_1、plugin_2 或 3）
        #[clap(short, long, value_parser)]
        pane_id: Option<String>,

        /// 在滚动缓冲转储中保留 ANSI 样式
        #[clap(short, long)]
        ansi: bool,
    },
    /// 在聚焦窗格中向上滚动
    ScrollUp {
        /// 按 ID 定位指定窗格（例如 terminal_1、plugin_2 或 3）
        #[clap(short, long, value_parser)]
        pane_id: Option<String>,
    },
    /// 在聚焦窗格中向下滚动。
    ScrollDown {
        /// 按 ID 定位指定窗格（例如 terminal_1、plugin_2 或 3）
        #[clap(short, long, value_parser)]
        pane_id: Option<String>,
    },
    /// 在聚焦窗格中向下滚动到底部。
    ScrollToBottom {
        /// 按 ID 定位指定窗格（例如 terminal_1、plugin_2 或 3）
        #[clap(short, long, value_parser)]
        pane_id: Option<String>,
    },
    /// 在聚焦窗格中向上滚动到顶部。
    ScrollToTop {
        /// 按 ID 定位指定窗格（例如 terminal_1、plugin_2 或 3）
        #[clap(short, long, value_parser)]
        pane_id: Option<String>,
    },
    /// 在聚焦窗格中向上滚动一页。
    PageScrollUp {
        /// 按 ID 定位指定窗格（例如 terminal_1、plugin_2 或 3）
        #[clap(short, long, value_parser)]
        pane_id: Option<String>,
    },
    /// 在聚焦窗格中向下滚动一页。
    PageScrollDown {
        /// 按 ID 定位指定窗格（例如 terminal_1、plugin_2 或 3）
        #[clap(short, long, value_parser)]
        pane_id: Option<String>,
    },
    /// 在聚焦窗格中向上滚动半页。
    HalfPageScrollUp {
        /// 按 ID 定位指定窗格（例如 terminal_1、plugin_2 或 3）
        #[clap(short, long, value_parser)]
        pane_id: Option<String>,
    },
    /// 在聚焦窗格中向下滚动半页。
    HalfPageScrollDown {
        /// 按 ID 定位指定窗格（例如 terminal_1、plugin_2 或 3）
        #[clap(short, long, value_parser)]
        pane_id: Option<String>,
    },
    /// 在全屏聚焦窗格和正常布局之间切换。
    ToggleFullscreen {
        /// 按 ID 定位指定窗格（例如 terminal_1、plugin_2 或 3）
        #[clap(short, long, value_parser)]
        pane_id: Option<String>,
    },
    #[clap(
        about = "Toggle between fullscreen over the entire display (including the UI bars) and normal layout"
    )]
    ToggleNoUiFullscreen {
        #[clap(
            short,
            long,
            value_parser,
            help = "Target a specific pane by ID (eg. terminal_1, plugin_2, or 3)"
        )]
        pane_id: Option<String>,
    },
    /// 切换 UI 中窗格周围的框架
    TogglePaneFrames,
    SetPaneFrameStyle {
        #[clap(value_enum, value_parser)]
        style: PaneFrameStyle,
    },
    /// 在向当前标签页所有窗格发送文本命令与正常模式之间切换。
    ToggleActiveSyncTab {
        /// 按 ID 定位指定标签页
        #[clap(short, long, value_parser)]
        tab_id: Option<usize>,
    },
    /// 在指定方向 [right|down] 打开新窗格
    /// 若未指定方向，将尝试使用最大的可用空间。
    /// 返回：创建的窗格 ID（格式：terminal_<id> 或 plugin_<id>）
    NewPane {
        /// 打开新窗格的方向
        #[clap(short, long, value_parser, conflicts_with("floating"))]
        direction: Option<Direction>,

        #[clap(last(true))]
        command: Vec<String>,

        #[clap(short, long, conflicts_with("command"), conflicts_with("direction"))]
        plugin: Option<String>,

        /// 更改新窗格的工作目录
        #[clap(long, value_parser)]
        cwd: Option<PathBuf>,

        /// 以浮动模式打开新窗格
        #[clap(short, long)]
        floating: bool,

        /// 在当前窗格位置打开新窗格，并暂时挂起当前窗格
        #[clap(short, long, conflicts_with("floating"), conflicts_with("direction"))]
        in_place: bool,

        /// 关闭被替换的窗格而不是挂起它（仅与 --in-place 一起使用有效）
        #[clap(long, requires("in_place"))]
        close_replaced_pane: bool,

        /// 原地打开时要替换的窗格，例如 terminal_1、plugin_2 或 3（仅
        /// 与 --in-place 搭配生效；默认为聚焦窗格）
        #[clap(
            long,
            value_parser,
            requires("in_place"),
            conflicts_with("near_current_pane")
        )]
        pane_id: Option<String>,

        /// 新窗格名称
        #[clap(short, long, value_parser)]
        name: Option<String>,

        /// 命令退出时立即关闭窗格
        #[clap(short, long, requires("command"))]
        close_on_exit: bool,
        /// 以挂起状态启动命令，仅在你首次按回车后才运行
        #[clap(short, long, requires("command"))]
        start_suspended: bool,
        #[clap(long, value_parser)]
        configuration: Option<PluginUserConfiguration>,
        #[clap(long, value_parser)]
        skip_plugin_cache: bool,
        /// 窗格浮动时的 x 坐标，可以是裸整数（如 1）或百分比（如 10%）
        #[clap(short, long, requires("floating"))]
        x: Option<String>,
        /// 窗格浮动时的 y 坐标，可以是裸整数（如 1）或百分比（如 10%）
        #[clap(short, long, requires("floating"))]
        y: Option<String>,
        /// 窗格浮动时的宽度，可以是裸整数（如 1）或百分比（如 10%）
        #[clap(long, requires("floating"))]
        width: Option<String>,
        /// 窗格浮动时的高度，可以是裸整数（如 1）或百分比（如 10%）
        #[clap(long, requires("floating"))]
        height: Option<String>,
        /// 是否固定浮动窗格使其始终置顶
        #[clap(long, requires("floating"))]
        pinned: Option<bool>,
        #[clap(long, conflicts_with("floating"), conflicts_with("direction"))]
        stacked: bool,
        /// 阻塞直到命令结束且其窗格已关闭
        #[clap(short, long)]
        blocking: bool,

        /// 阻塞直到命令成功退出（退出状态 0）或其窗格已关闭
        #[clap(
            long,
            conflicts_with("blocking"),
            conflicts_with("block_until_exit_failure"),
            conflicts_with("block_until_exit")
        )]
        block_until_exit_success: bool,

        /// 阻塞直到命令以失败退出（非零退出状态）或其窗格已被
        /// 关闭
        #[clap(
            long,
            conflicts_with("blocking"),
            conflicts_with("block_until_exit_success"),
            conflicts_with("block_until_exit")
        )]
        block_until_exit_failure: bool,

        /// 阻塞直到命令退出（无论退出状态如何）或其窗格已关闭
        #[clap(
            long,
            conflicts_with("blocking"),
            conflicts_with("block_until_exit_success"),
            conflicts_with("block_until_exit_failure")
        )]
        block_until_exit: bool,

        #[clap(skip)]
        unblock_condition: Option<UnblockCondition>,

        /// 若设置，将在当前窗格附近打开新窗格，而不是跟随用户的焦点
        #[clap(long)]
        near_current_pane: bool,
        #[clap(
            long,
            help = "if set, will open the pane without changing the focus of any client, placing it relative to the pane the command was issued from"
        )]
        no_focus: bool,
        /// 以无边框方式启动此窗格（警告：将无法用
        /// 鼠标移动）
        #[clap(long, value_parser)]
        borderless: Option<bool>,
        /// 按 ID 定位指定标签页
        #[clap(
            long,
            value_parser,
            conflicts_with("near_current_pane"),
            conflicts_with("in_place")
        )]
        tab_id: Option<usize>,
    },
    /// 使用默认 EDITOR 在 zellij 新窗格中打开指定文件
    /// 返回：创建的窗格 ID（格式：terminal_<id>）
    Edit {
        file: PathBuf,

        /// 打开新窗格的方向
        #[clap(short, long, value_parser, conflicts_with("floating"))]
        direction: Option<Direction>,

        /// 在指定的行号处打开文件
        #[clap(short, long, value_parser)]
        line_number: Option<usize>,

        /// 以浮动模式打开新窗格
        #[clap(short, long)]
        floating: bool,

        /// 在当前窗格位置打开新窗格，并暂时挂起当前窗格
        #[clap(short, long, conflicts_with("floating"), conflicts_with("direction"))]
        in_place: bool,

        /// 关闭被替换的窗格而不是挂起它（仅与 --in-place 一起使用有效）
        #[clap(long, requires("in_place"))]
        close_replaced_pane: bool,

        /// 更改编辑器的工作目录
        #[clap(long, value_parser)]
        cwd: Option<PathBuf>,
        /// 窗格浮动时的 x 坐标，可以是裸整数（如 1）或百分比（如 10%）
        #[clap(short, long, requires("floating"))]
        x: Option<String>,
        /// 窗格浮动时的 y 坐标，可以是裸整数（如 1）或百分比（如 10%）
        #[clap(short, long, requires("floating"))]
        y: Option<String>,
        /// 窗格浮动时的宽度，可以是裸整数（如 1）或百分比（如 10%）
        #[clap(long, requires("floating"))]
        width: Option<String>,
        /// 窗格浮动时的高度，可以是裸整数（如 1）或百分比（如 10%）
        #[clap(long, requires("floating"))]
        height: Option<String>,
        /// 是否固定浮动窗格使其始终置顶
        #[clap(long, requires("floating"))]
        pinned: Option<bool>,
        /// 若设置，将在当前窗格附近打开新窗格，而不是跟随用户的焦点
        #[clap(long)]
        near_current_pane: bool,
        #[clap(
            long,
            help = "if set, will open the pane without changing the focus of any client, placing it relative to the pane the command was issued from"
        )]
        no_focus: bool,
        /// 以无边框方式启动此窗格（警告：将无法用
        /// 鼠标移动）
        #[clap(short, long, value_parser)]
        borderless: Option<bool>,
        /// 按 ID 定位指定标签页
        #[clap(
            long,
            value_parser,
            conflicts_with("near_current_pane"),
            conflicts_with("in_place")
        )]
        tab_id: Option<usize>,
    },
    /// 切换所有已连接客户端的输入模式 [locked|pane|tab|resize|move|search|session]
    SwitchMode {
        input_mode: InputMode,
    },
    /// 若聚焦窗格是浮动窗格则嵌入，若是嵌入窗格则浮动
    TogglePaneEmbedOrFloating {
        /// 按 ID 定位指定窗格（例如 terminal_1、plugin_2 或 3）
        #[clap(short, long, value_parser)]
        pane_id: Option<String>,
    },
    /// 切换当前标签页中所有浮动窗格的可见性，若不存在则打开一个
    ToggleFloatingPanes {
        /// 按 ID 定位指定标签页
        #[clap(short, long, value_parser)]
        tab_id: Option<usize>,
    },
    /// 显示指定标签页（若未提供 tab_id 则为活动标签页）中的所有浮动窗格。
    ///
    /// 状态被改变时返回退出码 0，已可见时返回 2，标签页未找到时返回 1。
    ShowFloatingPanes {
        #[clap(short, long, value_parser)]
        tab_id: Option<usize>,
    },
    /// 隐藏指定标签页（若未提供 tab_id 则为活动标签页）中的所有浮动窗格。
    ///
    /// 状态被改变时返回退出码 0，已隐藏时返回 2，标签页未找到时返回 1。
    HideFloatingPanes {
        #[clap(short, long, value_parser)]
        tab_id: Option<usize>,
    },
    /// 检查指定标签页（或活动标签页）中的浮动窗格是否可见。
    ///
    /// 若可见，向 stdout 输出 "true" 并以 0 退出。
    /// 若不可见，向 stdout 输出 "false" 并以 1 退出。
    AreFloatingPanesVisible {
        #[clap(short, long, value_parser)]
        tab_id: Option<usize>,
    },
    /// 关闭聚焦窗格。
    ClosePane {
        /// 按 ID 定位指定窗格（例如 terminal_1、plugin_2 或 3）
        #[clap(short, long, value_parser)]
        pane_id: Option<String>,
    },
    /// 重命名聚焦窗格
    RenamePane {
        name: String,
        /// 按 ID 定位指定窗格（例如 terminal_1、plugin_2 或 3）
        #[clap(short, long, value_parser)]
        pane_id: Option<String>,
    },
    /// 移除先前设置的窗格名称
    UndoRenamePane {
        /// 按 ID 定位指定窗格（例如 terminal_1、plugin_2 或 3）
        #[clap(short, long, value_parser)]
        pane_id: Option<String>,
    },
    /// 转到下一个标签页。
    GoToNextTab,
    /// 转到上一个标签页。
    GoToPreviousTab,
    /// 关闭当前标签页。
    CloseTab {
        /// 按 ID 定位指定标签页
        #[clap(short, long, value_parser)]
        tab_id: Option<usize>,
    },
    /// 转到索引为 [index] 的标签页
    GoToTab {
        index: u32,
    },
    /// 转到名为 [name] 的标签页
    ///
    /// 返回：当使用 --create 且标签页被创建时，以单个数字输出标签页 ID
    GoToTabName {
        name: String,
        /// 若标签页不存在则创建它。
        #[clap(short, long, value_parser)]
        create: bool,
    },
    /// 重命名聚焦窗格
    RenameTab {
        name: String,
        /// 按 ID 定位指定标签页
        #[clap(short, long, value_parser)]
        tab_id: Option<usize>,
    },
    /// 移除先前设置的标签页名称
    UndoRenameTab {
        /// 按 ID 定位指定标签页
        #[clap(short, long, value_parser)]
        tab_id: Option<usize>,
    },
    /// 转到具有稳定 ID 的标签页
    GoToTabById {
        id: u64,
    },
    /// 关闭具有稳定 ID 的标签页
    CloseTabById {
        id: u64,
    },
    /// 按稳定 ID 重命名标签页
    RenameTabById {
        id: u64,
        name: String,
    },
    /// 创建一个新标签页，可选指定标签页布局和名称
    ///
    /// 返回：在 stdout 上以单个数字输出所创建标签页的 ID
    NewTab {
        /// 新标签页使用的布局
        #[clap(short, long, value_parser, conflicts_with = "layout_string")]
        layout: Option<PathBuf>,

        /// 直接使用的原始 KDL 布局字符串（而不是布局文件路径）
        #[clap(long, value_parser, conflicts_with = "layout")]
        layout_string: Option<String>,

        /// 查找布局的默认文件夹
        #[clap(long, value_parser, requires("layout"))]
        layout_dir: Option<PathBuf>,

        /// 新标签页名称
        #[clap(short, long, value_parser)]
        name: Option<String>,

        /// 更改新标签页的工作目录
        #[clap(short, long, value_parser)]
        cwd: Option<PathBuf>,

        /// 新标签页中可选的初始运行命令
        #[clap(value_parser, conflicts_with("initial_plugin"), last(true))]
        initial_command: Vec<String>,

        /// 新标签页中加载的初始插件
        #[clap(long, value_parser, conflicts_with("initial_command"))]
        initial_plugin: Option<String>,

        /// 命令退出时立即关闭窗格
        #[clap(long, requires("initial_command"))]
        close_on_exit: bool,

        /// 以挂起状态启动命令，仅在你首次按回车后才运行
        #[clap(long, requires("initial_command"))]
        start_suspended: bool,

        /// 阻塞直到命令成功退出（退出状态 0）或其窗格已关闭
        #[clap(
            long,
            requires("initial_command"),
            conflicts_with("block_until_exit_failure"),
            conflicts_with("block_until_exit")
        )]
        block_until_exit_success: bool,

        /// 阻塞直到命令以失败退出（非零退出状态）或其窗格已被关闭
        #[clap(
            long,
            requires("initial_command"),
            conflicts_with("block_until_exit_success"),
            conflicts_with("block_until_exit")
        )]
        block_until_exit_failure: bool,

        /// 阻塞直到命令退出（无论退出状态如何）或其窗格已关闭
        #[clap(
            long,
            requires("initial_command"),
            conflicts_with("block_until_exit_success"),
            conflicts_with("block_until_exit_failure")
        )]
        block_until_exit: bool,

        #[clap(
            long,
            help = "if set, will create the tab without changing the focus of any client"
        )]
        no_focus: bool,
    },
    /// 沿指定方向移动聚焦标签页。[right|left]
    MoveTab {
        direction: Direction,
        /// 按 ID 定位指定标签页
        #[clap(short, long, value_parser)]
        tab_id: Option<usize>,
    },
    PreviousSwapLayout {
        /// 按 ID 定位指定标签页
        #[clap(short, long, value_parser)]
        tab_id: Option<usize>,
    },
    NextSwapLayout {
        /// 按 ID 定位指定标签页
        #[clap(short, long, value_parser)]
        tab_id: Option<usize>,
    },
    /// 覆盖活动标签页的布局
    OverrideLayout {
        /// 布局文件的路径
        #[clap(
            value_parser,
            required_unless_present = "layout_string",
            conflicts_with = "layout_string"
        )]
        layout: Option<PathBuf>,

        /// 直接使用的原始 KDL 布局字符串（而不是布局文件路径）
        #[clap(long, value_parser, conflicts_with = "layout")]
        layout_string: Option<String>,

        /// 查找布局的默认文件夹
        #[clap(long, value_parser)]
        layout_dir: Option<PathBuf>,

        /// 保留不符合布局的现有终端窗格（默认：false）
        #[clap(long)]
        retain_existing_terminal_panes: bool,

        /// 保留不符合布局的现有插件窗格（默认：false）
        #[clap(long)]
        retain_existing_plugin_panes: bool,

        /// 仅将布局应用于活动标签页（若布局有多个标签页则
        /// 只使用第一个）
        #[clap(long)]
        apply_only_to_active_tab: bool,
    },
    /// 查询所有标签页名称
    QueryTabNames,
    StartOrReloadPlugin {
        url: String,
        #[clap(short, long, value_parser)]
        configuration: Option<PluginUserConfiguration>,
    },
    /// 返回：创建或聚焦插件时的插件窗格 ID（格式：plugin_<id>）
    LaunchOrFocusPlugin {
        #[clap(short, long, value_parser)]
        floating: bool,
        #[clap(short, long, value_parser)]
        in_place: bool,
        /// 关闭被替换的窗格而不是挂起它（仅与 --in-place 一起使用有效）
        #[clap(long, requires("in_place"))]
        close_replaced_pane: bool,
        #[clap(short, long, value_parser)]
        move_to_focused_tab: bool,
        url: String,
        #[clap(short, long, value_parser)]
        configuration: Option<PluginUserConfiguration>,
        #[clap(short, long, value_parser)]
        skip_plugin_cache: bool,
        /// 按 ID 定位指定标签页
        #[clap(long, value_parser, conflicts_with("in_place"))]
        tab_id: Option<usize>,
    },
    /// 返回：插件窗格 ID（格式：plugin_<id>）
    LaunchPlugin {
        #[clap(short, long, value_parser)]
        floating: bool,
        #[clap(short, long, value_parser)]
        in_place: bool,
        /// 关闭被替换的窗格而不是挂起它（仅与 --in-place 一起使用有效）
        #[clap(long, requires("in_place"))]
        close_replaced_pane: bool,
        url: Url,
        #[clap(short, long, value_parser)]
        configuration: Option<PluginUserConfiguration>,
        #[clap(short, long, value_parser)]
        skip_plugin_cache: bool,
        #[clap(
            long,
            help = "if set, will open the plugin pane without changing the focus of any client"
        )]
        no_focus: bool,
        /// 按 ID 定位指定标签页
        #[clap(long, value_parser, conflicts_with("in_place"))]
        tab_id: Option<usize>,
    },
    RenameSession {
        name: String,
    },
    /// 向一个或多个插件发送数据，若未运行则启动它们。
    #[clap(override_usage(
r#"
zellij action pipe [OPTIONS] [--] <PAYLOAD>

* Send data to a specific plugin:

zellij action pipe --plugin file:/path/to/my/plugin.wasm --name my_pipe_name -- my_arbitrary_data

* To all running plugins (that are listening):

zellij action pipe --name my_pipe_name -- my_arbitrary_data

* Pipe data into this command's STDIN and get output from the plugin on this command's STDOUT

tail -f /tmp/my-live-logfile | zellij action pipe --name logs --plugin https://example.com/my-plugin.wasm | wc -l
"#))]
    Pipe {
        /// 管道名称
        #[clap(short, long, value_parser, display_order(1))]
        name: Option<String>,
        /// 通过此管道发送的数据（若为空，将监听 STDIN）
        payload: Option<String>,

        #[clap(short, long, value_parser, display_order(2))]
        /// 管道的参数
        args: Option<PluginUserConfiguration>, // TODO：我们可能不想重复使用
        // PluginUserConfiguration
        /// 该管道指向的插件 url（例如 file:/tmp/my-plugin.wasm）；若未指定，
        /// 将发送给所有插件；若已指定但未运行，插件将被启动
        #[clap(short, long, value_parser, display_order(3))]
        plugin: Option<String>,
        /// 插件配置（注意：在确定管道目的地时，配置不同的同一插件会被视为
        /// 不同的插件）
        #[clap(short('c'), long, value_parser, display_order(4))]
        plugin_configuration: Option<PluginUserConfiguration>,
        /// 即使已有插件在运行，也启动一个新的插件
        #[clap(short('l'), long, display_order(5))]
        force_launch_plugin: bool,
        /// 若启动新插件，跳过缓存并强制编译插件
        #[clap(short('s'), long, display_order(6))]
        skip_plugin_cache: bool,
        /// 若启动插件，是否浮动，默认为浮动
        #[clap(short('f'), long, value_parser, display_order(7))]
        floating_plugin: Option<bool>,
        /// 若启动插件，原地启动它（在当前窗格之上）
        #[clap(
            short('i'),
            long,
            value_parser,
            conflicts_with("floating_plugin"),
            display_order(8)
        )]
        in_place_plugin: Option<bool>,
        /// 若启动插件，指定其工作目录
        #[clap(short('w'), long, value_parser, display_order(9))]
        plugin_cwd: Option<PathBuf>,
        /// 若启动插件，指定其窗格标题
        #[clap(short('t'), long, value_parser, display_order(10))]
        plugin_title: Option<String>,
    },
    ListClients,
    /// 列出当前会话中的所有窗格
    ///
    /// 返回：以表格或 JSON 格式将格式化后的窗格列表输出到 stdout
    ListPanes {
        /// 包含标签页信息（名称、位置、ID）
        #[clap(short, long, value_parser)]
        tab: bool,

        /// 包含正在运行的命令信息
        #[clap(short, long, value_parser)]
        command: bool,

        /// 包含窗格状态（聚焦、浮动、已退出等）
        #[clap(short, long, value_parser)]
        state: bool,

        /// 包含几何信息（位置、大小）
        #[clap(short, long, value_parser)]
        geometry: bool,

        /// 包含所有可用字段
        #[clap(short, long, value_parser)]
        all: bool,

        /// 以 JSON 格式输出
        #[clap(short, long, value_parser)]
        json: bool,
    },
    /// 列出所有标签页及其信息
    ///
    /// 返回：以表格或 JSON 格式返回标签页信息
    ListTabs {
        /// 包含状态信息（活动、全屏、同步、浮动可见性）
        #[clap(short, long, value_parser)]
        state: bool,

        /// 包含尺寸信息（视口、显示区域）
        #[clap(short, long, value_parser)]
        dimensions: bool,

        /// 包含窗格数量
        #[clap(short, long, value_parser)]
        panes: bool,

        /// 包含布局信息（交换布局名称和脏状态）
        #[clap(short, long, value_parser)]
        layout: bool,

        /// 包含所有可用字段
        #[clap(short, long, value_parser)]
        all: bool,

        /// 以 JSON 格式输出
        #[clap(short, long, value_parser)]
        json: bool,
    },
    /// 获取当前活动标签页的信息
    ///
    /// 返回：默认返回标签页名称和 ID，或以 JSON 格式返回完整信息
    CurrentTabInfo {
        /// 以 JSON 输出完整的 TabInfo
        #[clap(short, long, value_parser)]
        json: bool,
    },
    TogglePanePinned {
        /// 按 ID 定位指定窗格（例如 terminal_1、plugin_2 或 3）
        #[clap(short, long, value_parser)]
        pane_id: Option<String>,
    },
    /// 堆叠窗格 id
    /// id 是以空格分隔的窗格 id 列表。
    /// 它们既可以是 `terminal_<int>`（例如 terminal_1）、`plugin_<int>`（例如
    /// plugin_1）形式，也可以是裸整数，此时会被视为终端窗格（例如 1
    /// 等同于 terminal_1）
    ///
    /// 示例：zellij action stack-panes -- terminal_1 plugin_2 3
    StackPanes {
        #[clap(last(true), required(true))]
        pane_ids: Vec<String>,
    },
    ChangeFloatingPaneCoordinates {
        /// 浮动窗格的 pane_id，例如 terminal_1、plugin_2 或 3（等同于
        /// terminal_3）
        #[clap(short, long, value_parser)]
        pane_id: String,
        /// 窗格浮动时的 x 坐标，可以是裸整数（如 1）或百分比（如 10%）
        #[clap(short, long)]
        x: Option<String>,
        /// 窗格浮动时的 y 坐标，可以是裸整数（如 1）或百分比（如 10%）
        #[clap(short, long)]
        y: Option<String>,
        /// 窗格浮动时的宽度，可以是裸整数（如 1）或百分比（如 10%）
        #[clap(long)]
        width: Option<String>,
        /// 窗格浮动时的高度，可以是裸整数（如 1）或百分比（如 10%）
        #[clap(long)]
        height: Option<String>,
        /// 是否固定浮动窗格使其始终置顶
        #[clap(long)]
        pinned: Option<bool>,
        /// 将该窗格切换为带/不带边框（警告：不带边框时将无法用
        /// 鼠标移动）
        #[clap(short, long, value_parser)]
        borderless: Option<bool>,
    },
    TogglePaneBorderless {
        /// 窗格的 pane_id，例如 terminal_1、plugin_2 或 3（等价于 terminal_3）
        #[clap(short, long, value_parser)]
        pane_id: String,
    },
    SetPaneBorderless {
        /// 窗格的 pane_id，例如 terminal_1、plugin_2 或 3（等价于 terminal_3）
        #[clap(short, long, value_parser)]
        pane_id: String,
        /// 窗格应为无边框（有标志）还是有边框（无标志）
        #[clap(short, long, value_parser)]
        borderless: bool,
    },
    /// 从当前会话分离
    Detach,
    /// 切换到深色主题（使用配置的 `theme_dark`）。
    SetDarkTheme,
    /// 切换到浅色主题（使用配置的 `theme_light`）。
    SetLightTheme,
    /// 在深色和浅色主题之间切换（使用配置的 `theme_dark` 和 `theme_light`）
    ToggleTheme,
    /// 切换到不同的会话
    SwitchSession {
        /// 要切换到的会话名称
        name: String,
        /// 可选的聚焦标签页位置
        #[clap(long)]
        tab_position: Option<usize>,
        /// 可选的聚焦窗格 ID（例如 id 为 1 的终端窗格 "terminal_1"，或 id 为 2 的插件窗格 "plugin_2"）
        #[clap(long)]
        pane_id: Option<String>,
        /// 切换到会话时应用的布局（相对路径从 layout-dir 开始）
        #[clap(short, long, value_parser, conflicts_with = "layout_string")]
        layout: Option<PathBuf>,
        /// 直接使用的原始 KDL 布局字符串
        #[clap(long, value_parser, conflicts_with = "layout")]
        layout_string: Option<String>,
        /// 查找布局的默认文件夹
        #[clap(long, value_parser, requires("layout"))]
        layout_dir: Option<PathBuf>,
        /// 切换时更改工作目录
        #[clap(short, long, value_parser)]
        cwd: Option<PathBuf>,
    },
    /// 设置窗格的默认前景/背景颜色
    SetPaneColor {
        /// 窗格的 pane_id，例如 terminal_1、plugin_2 或 3（等同于 terminal_3）。
        /// 未提供时默认为 $ZELLIJ_PANE_ID。
        #[clap(short, long, value_parser)]
        pane_id: Option<String>,
        /// 前景颜色（例如 "#00e000"、"rgb:00/e0/00"）
        #[clap(long, value_parser)]
        fg: Option<String>,
        /// 背景颜色（例如 "#001a3a"、"rgb:00/1a/3a"）
        #[clap(long, value_parser)]
        bg: Option<String>,
        /// 将窗格颜色重置为终端默认值
        #[clap(long, value_parser, conflicts_with_all(&["fg", "bg"]))]
        reset: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn parse_subscribe(args: &[&str]) -> SubscribeCli {
        let mut full_args = vec!["zellij"];
        full_args.extend_from_slice(args);
        let cli = CliArgs::try_parse_from(full_args).unwrap();
        match cli.command {
            Some(Command::Subscribe(s)) => s,
            other => panic!("Expected Subscribe, got {:?}", other),
        }
    }

    #[test]
    fn subscribe_scrollback_bare_flag() {
        let s = parse_subscribe(&["subscribe", "--pane-id", "terminal_1", "--scrollback"]);
        assert_eq!(s.scrollback, Some(0));
    }

    #[test]
    fn subscribe_scrollback_with_value() {
        let s = parse_subscribe(&[
            "subscribe",
            "--pane-id",
            "terminal_1",
            "--scrollback",
            "100",
        ]);
        assert_eq!(s.scrollback, Some(100));
    }

    #[test]
    fn subscribe_scrollback_absent() {
        let s = parse_subscribe(&["subscribe", "--pane-id", "terminal_1"]);
        assert_eq!(s.scrollback, None);
    }

    #[test]
    fn subscribe_format_json() {
        let s = parse_subscribe(&["subscribe", "--pane-id", "terminal_1", "--format", "json"]);
        assert!(matches!(s.format, SubscribeFormat::Json));
    }

    #[test]
    fn subscribe_format_default_raw() {
        let s = parse_subscribe(&["subscribe", "--pane-id", "terminal_1"]);
        assert!(matches!(s.format, SubscribeFormat::Raw));
    }

    #[test]
    fn subscribe_multiple_pane_ids() {
        let s = parse_subscribe(&[
            "subscribe",
            "--pane-id",
            "terminal_1",
            "--pane-id",
            "plugin_2",
        ]);
        assert_eq!(
            s.pane_id,
            vec!["terminal_1".to_string(), "plugin_2".to_string()]
        );
    }

    #[test]
    fn subscribe_requires_pane_id() {
        let result = CliArgs::try_parse_from(["zellij", "subscribe"]);
        assert!(result.is_err());
    }
}
