use std::{
    iter::once,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};

use crate::{
    error::{AppError, Result},
    model::BuildAction,
};

#[derive(Clone, Debug)]
pub enum BuildTarget {
    Workspace {
        path: PathBuf,
        project: Option<String>,
        build_config: Option<String>,
        all: bool,
    },
    Project {
        path: PathBuf,
        build_config: String,
    },
}

#[derive(Clone, Debug)]
pub struct Invocation {
    pub executable: PathBuf,
    pub action: BuildAction,
    pub target: BuildTarget,
    pub verbose: bool,
    pub extra: Vec<String>,
}

impl Invocation {
    pub fn args(&self) -> Result<Vec<String>> {
        let mut args = Vec::new();
        match &self.target {
            BuildTarget::Workspace {
                path,
                project,
                build_config,
                all,
            } => {
                args.extend(["-w".into(), path_string(path)]);
                if *all {
                    if project.is_some() || build_config.is_some() {
                        return Err(AppError::Message(
                            "--all 不能与 --project 或 --config 同时使用".into(),
                        ));
                    }
                    args.push("-a".into());
                } else {
                    if let Some(project) = project {
                        args.extend(["-p".into(), project.clone()]);
                    }
                    let config = build_config.as_ref().ok_or_else(|| {
                        AppError::Message("workspace 单项目构建必须指定 BuildSet".into())
                    })?;
                    args.extend(["-c".into(), config.clone()]);
                }
            }
            BuildTarget::Project { path, build_config } => {
                args.extend(["-p".into(), path_string(path)]);
                args.extend(["-c".into(), build_config.clone()]);
            }
        }
        args.extend(["-d".into(), self.action.as_str().into()]);
        if self.verbose {
            args.push("-v".into());
        }
        args.extend(self.extra.clone());
        Ok(args)
    }

    pub fn execute(&self, dry_run: bool) -> Result<Option<ExitStatus>> {
        let args = self.args()?;
        if dry_run {
            println!("{}", display_command(&self.executable, &args));
            return Ok(None);
        }
        Command::new(&self.executable)
            .args(&args)
            .status()
            .map(Some)
            .map_err(|error| AppError::Tool {
                tool: self.executable.display().to_string(),
                message: error.to_string(),
            })
    }
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn display_command(executable: &Path, args: &[String]) -> String {
    once(executable.to_string_lossy().into_owned())
        .chain(args.iter().cloned())
        .map(|arg| {
            if arg.contains([' ', '\t', '"']) {
                format!("\"{}\"", arg.replace('"', "\\\""))
            } else {
                arg
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_workspace_build_to_cdk_make_arguments() {
        let invocation = Invocation {
            executable: PathBuf::from("cdk-make.exe"),
            action: BuildAction::Rebuild,
            target: BuildTarget::Workspace {
                path: PathBuf::from(r"C:\work space\sample.cdkws"),
                project: Some("app".into()),
                build_config: Some("BuildSet".into()),
                all: false,
            },
            verbose: true,
            extra: Vec::new(),
        };

        assert_eq!(
            invocation.args().unwrap(),
            [
                "-w",
                r"C:\work space\sample.cdkws",
                "-p",
                "app",
                "-c",
                "BuildSet",
                "-d",
                "rebuild",
                "-v"
            ]
        );
    }

    #[test]
    fn all_rejects_project_specific_arguments() {
        let invocation = Invocation {
            executable: PathBuf::from("cdk-make.exe"),
            action: BuildAction::Build,
            target: BuildTarget::Workspace {
                path: PathBuf::from("sample.cdkws"),
                project: Some("app".into()),
                build_config: None,
                all: true,
            },
            verbose: false,
            extra: Vec::new(),
        };

        assert!(invocation.args().is_err());
    }
}
