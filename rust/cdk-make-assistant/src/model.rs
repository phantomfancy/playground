use std::{
    fmt::{self, Display, Formatter},
    path::PathBuf,
};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, ValueEnum, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BuildAction {
    #[default]
    Build,
    Clean,
    Rebuild,
}

impl BuildAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Build => "build",
            Self::Clean => "clean",
            Self::Rebuild => "rebuild",
        }
    }
}

impl Display for BuildAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInfo {
    pub name: String,
    pub path: PathBuf,
    pub language: Option<String>,
    pub project_type: Option<String>,
    pub build_configs: Vec<String>,
    pub default_build_config: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceInfo {
    pub name: String,
    pub path: PathBuf,
    pub active_project: Option<String>,
    pub projects: Vec<ProjectInfo>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryReport {
    pub root: PathBuf,
    pub workspaces: Vec<WorkspaceInfo>,
    pub standalone_projects: Vec<ProjectInfo>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Envelope<T: Serialize> {
    pub schema_version: u32,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorBody>,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
}

impl<T: Serialize> Envelope<T> {
    pub fn success(data: T) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            ok: true,
            data: Some(data),
            error: None,
        }
    }
}

impl Envelope<()> {
    pub fn failure(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            ok: false,
            data: None,
            error: Some(ErrorBody {
                code: code.into(),
                message: message.into(),
            }),
        }
    }
}
