use std::{
    io::{self, IsTerminal},
    path::{Path, PathBuf},
    process::Command as ProcessCommand,
};

use crate::{
    config::{self, GlobalConfig, LocalConfig},
    discovery::{DiscoveryBackend, discover},
    error::{AppError, Result},
    model::{BuildAction, DiscoveryReport, Envelope, ProjectInfo, WorkspaceInfo},
    parser::XmlBackend,
    runner::{BuildTarget, Invocation},
};
use clap::{Args, Parser, Subcommand, ValueEnum};
use dialoguer::{Select, theme::ColorfulTheme};

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    #[arg(long, global = true, value_name = "PATH")]
    cdk_make: Option<PathBuf>,
    #[arg(long, global = true, value_name = "PATH")]
    rg: Option<PathBuf>,
    #[arg(long, global = true, value_name = "PATH")]
    yq: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Discover CDK workspaces and projects.
    Inspect {
        #[arg(default_value = ".")]
        root: PathBuf,
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
    /// Select, persist, and optionally run a build configuration.
    Configure(ConfigureArgs),
    /// Build the selected project.
    Build(RunArgs),
    /// Clean the selected project.
    Clean(RunArgs),
    /// Rebuild the selected project.
    Rebuild(RunArgs),
    /// Read or update global tool configuration.
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Validate configured executables and the current CDK project.
    Doctor {
        #[arg(default_value = ".")]
        root: PathBuf,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Args, Clone, Default)]
struct RunArgs {
    #[arg(default_value = ".")]
    root: PathBuf,
    #[arg(short = 'w', long, value_name = "PATH")]
    workspace: Option<PathBuf>,
    #[arg(short = 'p', long, value_name = "NAME")]
    project: Option<String>,
    #[arg(long, value_name = "PATH", conflicts_with = "workspace")]
    project_file: Option<PathBuf>,
    #[arg(short = 'c', long = "config", value_name = "BUILD_SET")]
    build_config: Option<String>,
    #[arg(short = 'a', long)]
    all: bool,
    #[arg(short = 'v', long)]
    verbose: bool,
    #[arg(long)]
    dry_run: bool,
    #[arg(last = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[derive(Debug, Args)]
struct ConfigureArgs {
    #[command(flatten)]
    run: RunArgs,
    #[arg(long, value_enum)]
    action: Option<BuildAction>,
    #[arg(long)]
    no_run: bool,
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    Show {
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
    Set {
        #[arg(value_enum)]
        key: ConfigKey,
        value: PathBuf,
    },
    Unset {
        #[arg(value_enum)]
        key: ConfigKey,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ConfigKey {
    CdkMake,
    Rg,
    Yq,
}

#[derive(Debug)]
struct Context {
    global: GlobalConfig,
    discovery_backend: DiscoveryBackend,
    xml_backend: XmlBackend,
}

#[derive(Debug)]
struct Prepared {
    invocation: Invocation,
    local_root: PathBuf,
    local: LocalConfig,
}

pub fn run(cli: Cli) -> Result<i32> {
    let json_mode = matches!(
        &cli.command,
        Some(Commands::Inspect {
            format: OutputFormat::Json,
            ..
        }) | Some(Commands::Config {
            command: ConfigCommand::Show {
                format: OutputFormat::Json
            }
        })
    );
    match dispatch(cli) {
        Ok(code) => Ok(code),
        Err(error) if json_mode => {
            println!(
                "{}",
                serde_json::to_string(&Envelope::failure(error.code(), error.to_string()))
                    .expect("serializing an error envelope cannot fail")
            );
            Ok(2)
        }
        Err(error) => Err(error),
    }
}

fn dispatch(cli: Cli) -> Result<i32> {
    if let Some(Commands::Config { command }) = cli.command {
        return handle_config(command);
    }

    let context = context(cli.rg, cli.yq)?;
    match cli.command {
        Some(Commands::Inspect { root, format }) => {
            let report = discover(&root, &context.discovery_backend, &context.xml_backend)?;
            print_report(&report, format)?;
            Ok(0)
        }
        Some(Commands::Configure(args)) => {
            let action = args.action.unwrap_or(BuildAction::Build);
            let prepared = prepare(&args.run, action, &context, cli.cdk_make, true)?;
            let path = config::save_local(&prepared.local_root, &prepared.local)?;
            eprintln!("已保存项目配置: {}", path.display());
            if args.no_run {
                Ok(0)
            } else {
                execute(prepared.invocation, args.run.dry_run)
            }
        }
        Some(Commands::Build(args)) => {
            execute_prepared(args, BuildAction::Build, context, cli.cdk_make)
        }
        Some(Commands::Clean(args)) => {
            execute_prepared(args, BuildAction::Clean, context, cli.cdk_make)
        }
        Some(Commands::Rebuild(args)) => {
            execute_prepared(args, BuildAction::Rebuild, context, cli.cdk_make)
        }
        Some(Commands::Doctor { root }) => doctor(&root, &context, cli.cdk_make),
        Some(Commands::Config { .. }) => unreachable!(),
        None => {
            let args = RunArgs::default();
            let action = config::load_local(Path::new("."))?
                .and_then(|(_, local)| local.action)
                .unwrap_or(BuildAction::Build);
            execute_prepared(args, action, context, cli.cdk_make)
        }
    }
}

fn context(rg: Option<PathBuf>, yq: Option<PathBuf>) -> Result<Context> {
    let global = config::load_global()?;
    let discovery_backend = rg
        .or_else(|| global.rg_path.clone())
        .map(DiscoveryBackend::Ripgrep)
        .unwrap_or(DiscoveryBackend::Native);
    let xml_backend = yq
        .or_else(|| global.yq_path.clone())
        .map(XmlBackend::Yq)
        .unwrap_or(XmlBackend::Native);
    Ok(Context {
        global,
        discovery_backend,
        xml_backend,
    })
}

fn execute_prepared(
    args: RunArgs,
    action: BuildAction,
    context: Context,
    cdk_make: Option<PathBuf>,
) -> Result<i32> {
    let prepared = prepare(&args, action, &context, cdk_make, false)?;
    execute(prepared.invocation, args.dry_run)
}

fn execute(invocation: Invocation, dry_run: bool) -> Result<i32> {
    match invocation.execute(dry_run)? {
        Some(status) => Ok(status.code().unwrap_or(1)),
        None => Ok(0),
    }
}

fn prepare(
    args: &RunArgs,
    action: BuildAction,
    context: &Context,
    cdk_make: Option<PathBuf>,
    force_choose: bool,
) -> Result<Prepared> {
    let start = absolute(&args.root)?;
    let loaded = config::load_local(&start)?;
    let (configured_root, existing) = loaded
        .map(|(root, local)| (Some(root), local))
        .unwrap_or_default();
    let local_root = configured_root.unwrap_or_else(|| start.clone());
    let report = discover(
        &local_root,
        &context.discovery_backend,
        &context.xml_backend,
    )?;
    let executable = config::resolve_cdk_make(cdk_make, &context.global)?;

    let explicit_workspace = args
        .workspace
        .as_ref()
        .map(|path| resolve_relative(&local_root, path));
    let configured_workspace = existing
        .workspace
        .as_ref()
        .map(|path| resolve_relative(&local_root, path));
    let explicit_project_file = args
        .project_file
        .as_ref()
        .map(|path| resolve_relative(&local_root, path));
    let configured_project_file = existing
        .project_file
        .as_ref()
        .map(|path| resolve_relative(&local_root, path));

    let (target, local) = if explicit_project_file.is_some()
        || (explicit_workspace.is_none()
            && configured_workspace.is_none()
            && configured_project_file.is_some())
        || (report.workspaces.is_empty() && !report.standalone_projects.is_empty())
    {
        let project = choose_standalone(
            &report,
            explicit_project_file.or(configured_project_file),
            force_choose,
        )?;
        let build_config = choose_build_config(
            project,
            args.build_config
                .as_ref()
                .or(existing.build_config.as_ref()),
            force_choose,
        )?;
        (
            BuildTarget::Project {
                path: project.path.clone(),
                build_config: build_config.clone(),
            },
            LocalConfig {
                schema_version: 1,
                project_file: Some(relative(&local_root, &project.path)),
                build_config: Some(build_config.clone()),
                action: Some(action),
                ..LocalConfig::default()
            },
        )
    } else {
        let workspace = choose_workspace(
            &report,
            explicit_workspace.or(configured_workspace),
            force_choose,
        )?;
        let build_all = args.all
            || (args.project.is_none()
                && args.build_config.is_none()
                && !force_choose
                && existing.all);
        if build_all {
            (
                BuildTarget::Workspace {
                    path: workspace.path.clone(),
                    project: None,
                    build_config: None,
                    all: true,
                },
                LocalConfig {
                    schema_version: 1,
                    workspace: Some(relative(&local_root, &workspace.path)),
                    action: Some(action),
                    all: true,
                    ..LocalConfig::default()
                },
            )
        } else {
            let project = choose_project(
                workspace,
                args.project.as_ref().or(existing.project.as_ref()),
                force_choose,
            )?;
            let build_config = choose_build_config(
                project,
                args.build_config
                    .as_ref()
                    .or(existing.build_config.as_ref()),
                force_choose,
            )?;
            (
                BuildTarget::Workspace {
                    path: workspace.path.clone(),
                    project: Some(project.name.clone()),
                    build_config: Some(build_config.clone()),
                    all: false,
                },
                LocalConfig {
                    schema_version: 1,
                    workspace: Some(relative(&local_root, &workspace.path)),
                    project: Some(project.name.clone()),
                    build_config: Some(build_config.clone()),
                    action: Some(action),
                    all: false,
                    ..LocalConfig::default()
                },
            )
        }
    };

    Ok(Prepared {
        invocation: Invocation {
            executable,
            action,
            target,
            verbose: args.verbose,
            extra: args.extra.clone(),
        },
        local_root,
        local,
    })
}

fn choose_workspace(
    report: &DiscoveryReport,
    selected: Option<PathBuf>,
    force_choose: bool,
) -> Result<&WorkspaceInfo> {
    if let Some(path) = selected {
        let canonical = dunce::canonicalize(&path).unwrap_or(path);
        return report
            .workspaces
            .iter()
            .find(|workspace| workspace.path == canonical)
            .ok_or_else(|| {
                AppError::Message(format!(
                    "配置的 workspace 不在发现结果中: {}",
                    canonical.display()
                ))
            });
    }
    choose(
        &report.workspaces,
        "请选择 CDK Workspace",
        |workspace| {
            relative(&report.root, &workspace.path)
                .display()
                .to_string()
        },
        force_choose,
    )
}

fn choose_project<'a>(
    workspace: &'a WorkspaceInfo,
    selected: Option<&String>,
    force_choose: bool,
) -> Result<&'a ProjectInfo> {
    if let Some(name) = selected {
        return workspace
            .projects
            .iter()
            .find(|project| project.name == *name)
            .ok_or_else(|| AppError::Message(format!("workspace 中不存在项目: {name}")));
    }
    if !force_choose
        && let Some(active) = workspace.active_project.as_ref()
        && let Some(project) = workspace
            .projects
            .iter()
            .find(|project| &project.name == active)
    {
        return Ok(project);
    }
    choose(
        &workspace.projects,
        "请选择 CDK 项目",
        |project| project.name.clone(),
        force_choose,
    )
}

fn choose_standalone(
    report: &DiscoveryReport,
    selected: Option<PathBuf>,
    force_choose: bool,
) -> Result<&ProjectInfo> {
    if let Some(path) = selected {
        let canonical = dunce::canonicalize(&path).unwrap_or(path);
        return report
            .standalone_projects
            .iter()
            .find(|project| project.path == canonical)
            .ok_or_else(|| {
                AppError::Message(format!("配置的项目不在发现结果中: {}", canonical.display()))
            });
    }
    choose(
        &report.standalone_projects,
        "请选择 CDK 项目",
        |project| relative(&report.root, &project.path).display().to_string(),
        force_choose,
    )
}

fn choose_build_config<'a>(
    project: &'a ProjectInfo,
    selected: Option<&String>,
    force_choose: bool,
) -> Result<&'a String> {
    if let Some(name) = selected {
        return project
            .build_configs
            .iter()
            .find(|config| *config == name)
            .ok_or_else(|| {
                AppError::Message(format!("项目 {} 中不存在 BuildSet: {name}", project.name))
            });
    }
    if !force_choose && let Some(default) = project.default_build_config.as_ref() {
        return Ok(default);
    }
    choose(
        &project.build_configs,
        "请选择 BuildSet",
        Clone::clone,
        force_choose,
    )
}

fn choose<'a, T, F>(values: &'a [T], prompt: &str, label: F, force_choose: bool) -> Result<&'a T>
where
    F: Fn(&T) -> String,
{
    match values {
        [] => Err(AppError::Message(format!("{prompt}: 没有可用项"))),
        [only] if !force_choose => Ok(only),
        _ if io::stdin().is_terminal() => {
            let labels = values.iter().map(label).collect::<Vec<_>>();
            let index = Select::with_theme(&ColorfulTheme::default())
                .with_prompt(prompt)
                .items(&labels)
                .default(0)
                .interact()
                .map_err(|error| AppError::Message(error.to_string()))?;
            Ok(&values[index])
        }
        _ => Err(AppError::Message(format!(
            "{prompt}: 存在多个候选项，非交互模式下必须显式指定"
        ))),
    }
}

fn print_report(report: &DiscoveryReport, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&Envelope::success(report))
                .map_err(|error| AppError::Message(error.to_string()))?
        ),
        OutputFormat::Text => {
            println!("Root: {}", report.root.display());
            for workspace in &report.workspaces {
                println!(
                    "Workspace: {} ({})",
                    workspace.name,
                    workspace.path.display()
                );
                for project in &workspace.projects {
                    println!(
                        "  Project: {} [{}]",
                        project.name,
                        project.build_configs.join(", ")
                    );
                }
            }
            for project in &report.standalone_projects {
                println!(
                    "Project: {} ({}) [{}]",
                    project.name,
                    project.path.display(),
                    project.build_configs.join(", ")
                );
            }
        }
    }
    Ok(())
}

fn handle_config(command: ConfigCommand) -> Result<i32> {
    let mut global = config::load_global()?;
    match command {
        ConfigCommand::Show { format } => match format {
            OutputFormat::Json => println!(
                "{}",
                serde_json::to_string_pretty(&Envelope::success(global))
                    .map_err(|error| AppError::Message(error.to_string()))?
            ),
            OutputFormat::Text => {
                let path = config::global_config_path()?;
                println!("Config: {}", path.display());
                if !path.is_file() {
                    println!("Status: not created; built-in defaults are active");
                }
                print!(
                    "{}",
                    toml::to_string_pretty(&global)
                        .map_err(|error| AppError::Message(error.to_string()))?
                );
            }
        },
        ConfigCommand::Set { key, value } => {
            if !value.is_file() {
                return Err(AppError::Message(format!(
                    "配置路径不是文件: {}",
                    value.display()
                )));
            }
            set_key(&mut global, key, Some(absolute(&value)?));
            let path = config::save_global(&global)?;
            println!("已更新: {}", path.display());
        }
        ConfigCommand::Unset { key } => {
            set_key(&mut global, key, None);
            let path = config::save_global(&global)?;
            println!("已更新: {}", path.display());
        }
    }
    Ok(0)
}

fn set_key(config: &mut GlobalConfig, key: ConfigKey, value: Option<PathBuf>) {
    match key {
        ConfigKey::CdkMake => config.cdk_make_path = value,
        ConfigKey::Rg => config.rg_path = value,
        ConfigKey::Yq => config.yq_path = value,
    }
}

fn doctor(root: &Path, context: &Context, cdk_make: Option<PathBuf>) -> Result<i32> {
    let executable = config::resolve_cdk_make(cdk_make, &context.global)?;
    println!("OK cdk-make: {}", executable.display());
    check_optional_tool("rg", &context.discovery_backend)?;
    check_yq(&context.xml_backend)?;
    let report = discover(root, &context.discovery_backend, &context.xml_backend)?;
    println!(
        "OK discovery: {} workspace(s), {} standalone project(s)",
        report.workspaces.len(),
        report.standalone_projects.len()
    );
    Ok(0)
}

fn check_optional_tool(name: &str, backend: &DiscoveryBackend) -> Result<()> {
    if let DiscoveryBackend::Ripgrep(path) = backend {
        let output = ProcessCommand::new(path)
            .arg("--version")
            .output()
            .map_err(|error| AppError::Tool {
                tool: name.into(),
                message: error.to_string(),
            })?;
        if !output.status.success() {
            return Err(AppError::Tool {
                tool: name.into(),
                message: String::from_utf8_lossy(&output.stderr).into_owned(),
            });
        }
        println!("OK rg: {}", path.display());
    }
    Ok(())
}

fn check_yq(backend: &XmlBackend) -> Result<()> {
    if let XmlBackend::Yq(path) = backend {
        let output = ProcessCommand::new(path)
            .arg("--version")
            .output()
            .map_err(|error| AppError::Tool {
                tool: "yq".into(),
                message: error.to_string(),
            })?;
        let version = String::from_utf8_lossy(&output.stdout);
        if !output.status.success()
            || !version.contains("mikefarah")
            || !version.contains("version v4")
        {
            return Err(AppError::Tool {
                tool: "yq".into(),
                message: "需要 Mike Farah yq v4".into(),
            });
        }
        println!("OK yq: {}", path.display());
    }
    Ok(())
}

fn absolute(path: &Path) -> Result<PathBuf> {
    dunce::canonicalize(path).map_err(|source| AppError::Read {
        path: path.to_path_buf(),
        source,
    })
}

fn resolve_relative(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn relative(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}

pub fn parse() -> Cli {
    Cli::parse()
}
