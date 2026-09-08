// 此文件由 prost-build 生成。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum PermissionType {
    ReadApplicationState = 0,
    ChangeApplicationState = 1,
    OpenFiles = 2,
    RunCommands = 3,
    OpenTerminalsOrPlugins = 4,
    WriteToStdin = 5,
    WebAccess = 6,
    ReadCliPipes = 7,
    MessageAndLaunchOtherPlugins = 8,
    Reconfigure = 9,
    FullHdAccess = 10,
    StartWebServer = 11,
    InterceptInput = 12,
    ReadPaneContents = 13,
    RunActionsAsUser = 14,
    WriteToClipboard = 15,
    ReadSessionEnvironmentVariables = 16,
}
impl PermissionType {
    /// ProtoBuf 定义中使用的枚举字段名的字符串值。
    ///
    /// 这些值不以任何方式进行转换，因此被认为是稳定的
    ///（如果 ProtoBuf 定义不变）且可供编程安全使用。
    pub fn as_str_name(&self) -> &'static str {
        match self {
            PermissionType::ReadApplicationState => "ReadApplicationState",
            PermissionType::ChangeApplicationState => "ChangeApplicationState",
            PermissionType::OpenFiles => "OpenFiles",
            PermissionType::RunCommands => "RunCommands",
            PermissionType::OpenTerminalsOrPlugins => "OpenTerminalsOrPlugins",
            PermissionType::WriteToStdin => "WriteToStdin",
            PermissionType::WebAccess => "WebAccess",
            PermissionType::ReadCliPipes => "ReadCliPipes",
            PermissionType::MessageAndLaunchOtherPlugins => "MessageAndLaunchOtherPlugins",
            PermissionType::Reconfigure => "Reconfigure",
            PermissionType::FullHdAccess => "FullHdAccess",
            PermissionType::StartWebServer => "StartWebServer",
            PermissionType::InterceptInput => "InterceptInput",
            PermissionType::ReadPaneContents => "ReadPaneContents",
            PermissionType::RunActionsAsUser => "RunActionsAsUser",
            PermissionType::WriteToClipboard => "WriteToClipboard",
            PermissionType::ReadSessionEnvironmentVariables => "ReadSessionEnvironmentVariables",
        }
    }
    /// 从 ProtoBuf 定义中使用的字段名创建枚举。
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "ReadApplicationState" => Some(Self::ReadApplicationState),
            "ChangeApplicationState" => Some(Self::ChangeApplicationState),
            "OpenFiles" => Some(Self::OpenFiles),
            "RunCommands" => Some(Self::RunCommands),
            "OpenTerminalsOrPlugins" => Some(Self::OpenTerminalsOrPlugins),
            "WriteToStdin" => Some(Self::WriteToStdin),
            "WebAccess" => Some(Self::WebAccess),
            "ReadCliPipes" => Some(Self::ReadCliPipes),
            "MessageAndLaunchOtherPlugins" => Some(Self::MessageAndLaunchOtherPlugins),
            "Reconfigure" => Some(Self::Reconfigure),
            "FullHdAccess" => Some(Self::FullHdAccess),
            "StartWebServer" => Some(Self::StartWebServer),
            "InterceptInput" => Some(Self::InterceptInput),
            "ReadPaneContents" => Some(Self::ReadPaneContents),
            "RunActionsAsUser" => Some(Self::RunActionsAsUser),
            "WriteToClipboard" => Some(Self::WriteToClipboard),
            "ReadSessionEnvironmentVariables" => Some(Self::ReadSessionEnvironmentVariables),
            _ => None,
        }
    }
}
