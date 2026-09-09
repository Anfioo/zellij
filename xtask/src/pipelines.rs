//! Composite pipelines for the build system.
//!
//! Defines multiple "pipelines" that run specific individual steps in sequence.
use crate::{build, format, metadata, test};
use crate::{flags, WorkspaceMember};
use anyhow::Context;
use xshell::{cmd, Shell};

/// Perform a default build.
///
/// Runs the following steps in sequence:
///
/// - format
/// - build
/// - test
pub fn make(sh: &Shell, flags: flags::Make) -> anyhow::Result<()> {
    let err_context = || format!("failed to run pipeline 'make' with args {flags:?}");

    if flags.clean {
        crate::cargo()
            .and_then(|cargo| cmd!(sh, "{cargo} clean").run().map_err(anyhow::Error::new))
            .with_context(err_context)?;
    }

    format::format(sh, flags::Format { check: false })
        .and_then(|_| {
            build::build(
                sh,
                flags::Build {
                    release: flags.release,
                    no_plugins: false,
                    plugins_only: false,
                    no_web: flags.no_web,
                    args: vec![],
                },
            )
        })
        .and_then(|_| {
            test::test(
                sh,
                flags::Test {
                    args: vec![],
                    no_web: flags.no_web,
                },
            )
        })
        .with_context(err_context)
}

/// Generate a runnable executable.
///
/// Runs the following steps in sequence:
///
/// - [`build`](build::build) (release, plugins only)
/// - [`build`](build::build) (release, without plugins)
/// - Copy the executable to [target file](flags::Install::destination)
pub fn install(sh: &Shell, flags: flags::Install) -> anyhow::Result<()> {
    let err_context = || format!("failed to run pipeline 'install' with args {flags:?}");

    // Build and optimize plugins
    build::build(
        sh,
        flags::Build {
            release: true,
            no_plugins: false,
            plugins_only: true,
            no_web: flags.no_web,
            args: vec![],
        },
    )
    .and_then(|_| {
        // Build the main executable
        build::build(
            sh,
            flags::Build {
                release: true,
                no_plugins: true,
                plugins_only: false,
                no_web: flags.no_web,
                args: flags.args.clone(),
            },
        )
    })
    .with_context(err_context)?;

    // Copy binary to destination
    let destination = if flags.destination.is_absolute() {
        flags.destination.clone()
    } else {
        std::env::current_dir()
            .context("Can't determine current working directory")?
            .join(&flags.destination)
    };
    sh.change_dir(crate::project_root());
    sh.copy_file(
        crate::target_dir().join("release").join("zellij"),
        &destination,
    )
    .with_context(err_context)
}

/// Run zellij debug build.
pub fn run(sh: &Shell, mut flags: flags::Run) -> anyhow::Result<()> {
    let err_context =
        |flags: &flags::Run| format!("failed to run pipeline 'run' with args {:?}", flags);

    if flags.quick_run {
        if flags.data_dir.is_some() {
            eprintln!("不能同时使用 '--data-dir' 和 '--quick-run'！");
            std::process::exit(1);
        }
        flags.data_dir.replace(crate::asset_dir());
    }

    let profile = if flags.disable_deps_optimize {
        "dev"
    } else {
        "dev-opt"
    };

    if let Some(ref data_dir) = flags.data_dir {
        let data_dir = sh.current_dir().join(data_dir);
        let features = if flags.no_web {
            "disable_automatic_asset_installation"
        } else {
            "disable_automatic_asset_installation web_server_capability"
        };

        build::ensure_plugin_assets(sh).with_context(|| err_context(&flags))?;

        crate::cargo()
            .and_then(|cargo| {
                cmd!(sh, "{cargo} run")
                    .args(["--package", "zellij"])
                    .arg("--no-default-features")
                    .args(["--features", features])
                    .args(["--profile", profile])
                    .args(["--", "--data-dir", &format!("{}", data_dir.display())])
                    .args(&flags.args)
                    .run()
                    .map_err(anyhow::Error::new)
            })
            .with_context(|| err_context(&flags))
    } else {
        build::build(
            sh,
            flags::Build {
                release: false,
                no_plugins: false,
                plugins_only: true,
                no_web: flags.no_web,
                args: vec![],
            },
        )
        .and_then(|_| crate::cargo())
        .and_then(|cargo| {
            if flags.no_web {
                // Use dynamic metadata approach to get the correct features
                match metadata::get_no_web_features(sh, ".")
                    .context("Failed to check web features for main crate")?
                {
                    Some(features) => {
                        let mut cmd = cmd!(sh, "{cargo} run").args(["--no-default-features"]);

                        if !features.is_empty() {
                            cmd = cmd.args(["--features", &features]);
                        }

                        cmd.args(["--profile", profile])
                            .args(["--"])
                            .args(&flags.args)
                            .run()
                            .map_err(anyhow::Error::new)
                    },
                    None => {
                        // Main crate doesn't have web_server_capability, run normally
                        cmd!(sh, "{cargo} run")
                            .args(["--profile", profile])
                            .args(["--"])
                            .args(&flags.args)
                            .run()
                            .map_err(anyhow::Error::new)
                    },
                }
            } else {
                cmd!(sh, "{cargo} run")
                    .args(["--profile", profile])
                    .args(["--"])
                    .args(&flags.args)
                    .run()
                    .map_err(anyhow::Error::new)
            }
        })
        .with_context(|| err_context(&flags))
    }
}

/// Actions for the user to choose from to resolve publishing errors/conflicts.
enum UserAction {
    Retry,
    Abort,
    Ignore,
}

/// Make a zellij release and publish all crates.
pub fn publish(sh: &Shell, flags: flags::Publish) -> anyhow::Result<()> {
    let err_context = "failed to publish zellij";

    // Process flags
    let dry_run = if flags.dry_run {
        Some("--dry-run")
    } else {
        None
    };
    let remote = flags.git_remote.unwrap_or("origin".into());
    let registry = if let Some(ref registry) = flags.cargo_registry {
        Some(format!(
            "--registry={}",
            registry
                .clone()
                .into_string()
                .map_err(|registry| anyhow::Error::msg(format!(
                    "将 '{:?}' 转换为有效的注册表名称失败",
                    registry
                )))
                .context(err_context)?
        ))
    } else {
        None
    };
    let registry = registry.as_ref();
    if flags.no_push && flags.cargo_registry.is_none() {
        anyhow::bail!("标志 '--no-push' 只能与 '--cargo-registry' 一起使用");
    }

    sh.change_dir(crate::project_root());
    let cargo = crate::cargo().context(err_context)?;
    let project_dir = crate::project_root();
    let manifest = sh
        .read_file(project_dir.join("Cargo.toml"))
        .context(err_context)?
        .parse::<toml::Table>()
        .context(err_context)?;
    // Version of the core crate
    let version = manifest
        .get("workspace")
        .and_then(|workspace| workspace.get("package"))
        .and_then(|package| package["version"].as_str())
        .context("failed to read package version from manifest")
        .context(err_context)?;

    let mut skip_build = false;
    if cmd!(sh, "git tag -l")
        .read()
        .context(err_context)?
        .contains(version)
    {
        println!();
        println!("Git 标签 'v{version}' 已存在。");
        println!("如果这是误操作，请用以下命令删除：git tag -d 'v{version}'");
        println!("跳过构建阶段并继续发布？[y/n]");

        let stdin = std::io::stdin();
        loop {
            let mut buffer = String::new();
            stdin.read_line(&mut buffer).context(err_context)?;
            match buffer.trim_end() {
                "y" | "Y" => {
                    skip_build = true;
                    break;
                },
                "n" | "N" => {
                    skip_build = false;
                    break;
                },
                _ => {
                    println!(" --> 未知输入 '{buffer}'，正在忽略……");
                    println!();
                    println!("跳过构建阶段并继续发布？[y/n]");
                },
            }
        }
    }

    if !skip_build {
        // Clean project
        cmd!(sh, "{cargo} clean").run().context(err_context)?;

        // Build plugins
        build::build(
            sh,
            flags::Build {
                release: true,
                no_plugins: false,
                plugins_only: true,
                no_web: false,
                args: vec![],
            },
        )
        .context(err_context)?;

        // Update default config
        sh.copy_file(
            project_dir
                .join("zellij-utils")
                .join("assets")
                .join("config")
                .join("default.kdl"),
            project_dir.join("example").join("default.kdl"),
        )
        .context(err_context)?;

        // Commit changes
        cmd!(sh, "git commit -aem")
            .arg(format!("chore(release): v{}", version))
            .run()
            .context(err_context)?;

        // Tag release
        cmd!(sh, "git tag --annotate --message")
            .arg(format!("Version {}", version))
            .arg(format!("v{}", version))
            .run()
            .context(err_context)?;
    }

    let closure = || -> anyhow::Result<()> {
        // Push commit and tag
        if flags.dry_run {
            println!("由于试运行（dry-run）而跳过推送");
        } else if flags.no_push {
            println!("由于 no-push 而跳过推送");
        } else {
            let branch = cmd!(sh, "git rev-parse --abbrev-ref HEAD")
                .read()
                .context(err_context)?;
            cmd!(sh, "git push --atomic {remote} {branch} v{version}")
                .run()
                .context(err_context)?;
        }

        // Publish all the crates
        for WorkspaceMember { crate_name, .. } in crate::workspace_members().iter() {
            if crate_name.contains("plugin") || crate_name.contains("xtask") {
                continue;
            }

            let _pd = sh.push_dir(project_dir.join(crate_name));
            loop {
                let msg = format!(">> Publishing '{crate_name}'");
                crate::status(&msg);
                println!("{}", msg);

                let more_args = match *crate_name {
                    // This is needed for zellij to pick up the plugins from the assets included in
                    // the released zellij-utils binary
                    "." => Some("--no-default-features"),
                    _ => None,
                };

                if let Err(err) = cmd!(
                    sh,
                    "{cargo} publish --locked {registry...} {more_args...} {dry_run...}"
                )
                .run()
                .context(err_context)
                {
                    println!();
                    println!("发布 crate '{crate_name}' 失败，错误：");
                    println!("{:?}", err);
                    println!();
                    println!("请选择要执行的操作：[r]重试/[a]中止/[i]忽略");

                    let stdin = std::io::stdin();
                    let action;

                    loop {
                        let mut buffer = String::new();
                        stdin.read_line(&mut buffer).context(err_context)?;
                        match buffer.trim_end() {
                            "r" | "R" => {
                                action = UserAction::Retry;
                                break;
                            },
                            "a" | "A" => {
                                action = UserAction::Abort;
                                break;
                            },
                            "i" | "I" => {
                                action = UserAction::Ignore;
                                break;
                            },
                            _ => {
                                println!(" --> 未知输入 '{buffer}'，正在忽略……");
                                println!();
                                println!("请选择要执行的操作：[r]重试/[a]中止/[i]忽略");
                            },
                        }
                    }

                    match action {
                        UserAction::Retry => continue,
                        UserAction::Ignore => break,
                        UserAction::Abort => {
                            eprintln!("正在中止发布 crate '{crate_name}'");
                            return Err::<(), _>(err);
                        },
                    }
                } else {
                    // publish successful, continue to next crate
                    break;
                }
            }
        }

        println!();
        println!(" +-----------------------------------------------+");
        println!(" | 赞美开发者们，我们发布了新的 ZELLIJ |");
        println!(" +-----------------------------------------------+");
        Ok(())
    };

    // We run this in a closure so that a failure in any of the commands doesn't abort the whole
    // program. When dry-running we need to undo the release commit first!
    let result = closure();

    if flags.dry_run && !skip_build {
        cmd!(sh, "git reset --hard HEAD~1")
            .run()
            .context(err_context)?;
    }

    result
}
