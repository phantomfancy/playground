use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use quick_xml::de::from_str;
use serde::Deserialize;
use serde_json::Value;

use crate::{
    error::{AppError, Result},
    model::{ProjectInfo, WorkspaceInfo},
};

#[derive(Debug, Deserialize)]
struct WorkspaceXml {
    #[serde(rename = "@Name")]
    name: String,
    #[serde(rename = "Project", default)]
    projects: Vec<WorkspaceProjectXml>,
    #[serde(rename = "BuildMatrix")]
    build_matrix: Option<BuildMatrixXml>,
}

#[derive(Debug, Deserialize)]
struct WorkspaceProjectXml {
    #[serde(rename = "@Name")]
    name: String,
    #[serde(rename = "@Path")]
    path: PathBuf,
    #[serde(rename = "@Active")]
    active: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BuildMatrixXml {
    #[serde(rename = "WorkspaceConfiguration", default)]
    configurations: Vec<WorkspaceConfigurationXml>,
}

#[derive(Debug, Deserialize)]
struct WorkspaceConfigurationXml {
    #[serde(rename = "@Selected")]
    selected: Option<String>,
    #[serde(rename = "Project", default)]
    projects: Vec<MatrixProjectXml>,
}

#[derive(Debug, Deserialize)]
struct MatrixProjectXml {
    #[serde(rename = "@Name")]
    name: String,
    #[serde(rename = "@ConfigName")]
    config_name: String,
}

#[derive(Debug, Deserialize)]
struct ProjectXml {
    #[serde(rename = "@Name")]
    name: String,
    #[serde(rename = "@Language")]
    language: Option<String>,
    #[serde(rename = "@Type")]
    project_type: Option<String>,
    #[serde(rename = "BuildConfigs")]
    build_configs: Option<BuildConfigsXml>,
}

#[derive(Debug, Deserialize)]
struct BuildConfigsXml {
    #[serde(rename = "BuildConfig", default)]
    configs: Vec<BuildConfigXml>,
}

#[derive(Debug, Deserialize)]
struct BuildConfigXml {
    #[serde(rename = "@Name")]
    name: String,
}

#[derive(Clone, Debug)]
pub enum XmlBackend {
    Native,
    Yq(PathBuf),
}

pub fn parse_workspace(path: &Path, backend: &XmlBackend) -> Result<WorkspaceInfo> {
    match backend {
        XmlBackend::Native => parse_workspace_native(path),
        XmlBackend::Yq(yq) => parse_workspace_yq(path, yq),
    }
}

pub fn parse_project(
    path: &Path,
    default_build_config: Option<String>,
    backend: &XmlBackend,
) -> Result<ProjectInfo> {
    let mut project = match backend {
        XmlBackend::Native => parse_project_native(path)?,
        XmlBackend::Yq(yq) => parse_project_yq(path, yq)?,
    };
    project.path = path.to_path_buf();
    project.default_build_config = default_build_config
        .filter(|candidate| project.build_configs.contains(candidate))
        .or_else(|| (project.build_configs.len() == 1).then(|| project.build_configs[0].clone()));
    Ok(project)
}

fn parse_workspace_native(path: &Path) -> Result<WorkspaceInfo> {
    let xml: WorkspaceXml = from_str(&read(path)?).map_err(|error| AppError::Parse {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;
    workspace_from_parts(
        path,
        xml.name,
        xml.projects
            .into_iter()
            .map(|project| (project.name, project.path, project.active))
            .collect(),
        xml.build_matrix
            .map(|matrix| {
                matrix
                    .configurations
                    .into_iter()
                    .map(|configuration| {
                        (
                            configuration.selected,
                            configuration
                                .projects
                                .into_iter()
                                .map(|project| (project.name, project.config_name))
                                .collect(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default(),
        &XmlBackend::Native,
    )
}

fn parse_project_native(path: &Path) -> Result<ProjectInfo> {
    let xml: ProjectXml = from_str(&read(path)?).map_err(|error| AppError::Parse {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;
    Ok(ProjectInfo {
        name: xml.name,
        path: path.to_path_buf(),
        language: xml.language,
        project_type: xml.project_type,
        build_configs: xml
            .build_configs
            .map(|configs| {
                configs
                    .configs
                    .into_iter()
                    .map(|config| config.name)
                    .collect()
            })
            .unwrap_or_default(),
        default_build_config: None,
    })
}

fn parse_workspace_yq(path: &Path, yq: &Path) -> Result<WorkspaceInfo> {
    let json = yq_json(path, yq)?;
    let root = json
        .get("CDK_Workspace")
        .ok_or_else(|| parse_message(path, "缺少 CDK_Workspace 根元素"))?;
    let projects = values(root.get("Project"))
        .into_iter()
        .map(|project| {
            Ok((
                attr(project, "Name", path)?.to_owned(),
                PathBuf::from(attr(project, "Path", path)?),
                attr_optional(project, "Active").map(str::to_owned),
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    let mut matrix = Vec::new();
    if let Some(build_matrix) = root.get("BuildMatrix") {
        for configuration in values(build_matrix.get("WorkspaceConfiguration")) {
            let mappings = values(configuration.get("Project"))
                .into_iter()
                .map(|project| {
                    Ok((
                        attr(project, "Name", path)?.to_owned(),
                        attr(project, "ConfigName", path)?.to_owned(),
                    ))
                })
                .collect::<Result<Vec<_>>>()?;
            matrix.push((
                attr_optional(configuration, "Selected").map(str::to_owned),
                mappings,
            ));
        }
    }
    workspace_from_parts(
        path,
        attr(root, "Name", path)?.to_owned(),
        projects,
        matrix,
        &XmlBackend::Yq(yq.to_path_buf()),
    )
}

fn parse_project_yq(path: &Path, yq: &Path) -> Result<ProjectInfo> {
    let json = yq_json(path, yq)?;
    let root = json
        .get("Project")
        .ok_or_else(|| parse_message(path, "缺少 Project 根元素"))?;
    let build_configs = root
        .get("BuildConfigs")
        .map(|configs| {
            values(configs.get("BuildConfig"))
                .into_iter()
                .map(|config| attr(config, "Name", path).map(str::to_owned))
                .collect()
        })
        .transpose()?
        .unwrap_or_default();
    Ok(ProjectInfo {
        name: attr(root, "Name", path)?.to_owned(),
        path: path.to_path_buf(),
        language: attr_optional(root, "Language").map(str::to_owned),
        project_type: attr_optional(root, "Type").map(str::to_owned),
        build_configs,
        default_build_config: None,
    })
}

type WorkspaceProjectParts = (String, PathBuf, Option<String>);
type MatrixParts = (Option<String>, Vec<(String, String)>);

fn workspace_from_parts(
    path: &Path,
    name: String,
    projects: Vec<WorkspaceProjectParts>,
    matrix: Vec<MatrixParts>,
    backend: &XmlBackend,
) -> Result<WorkspaceInfo> {
    let defaults: HashMap<String, String> = matrix
        .iter()
        .find(|(selected, _)| is_yes(selected.as_deref()))
        .or_else(|| matrix.first())
        .map(|(_, mappings)| mappings.iter().cloned().collect())
        .unwrap_or_default();
    let parent = path
        .parent()
        .ok_or_else(|| parse_message(path, "workspace 路径没有父目录"))?;
    let active_project = projects
        .iter()
        .find(|(_, _, active)| is_yes(active.as_deref()))
        .map(|(name, _, _)| name.clone());
    let mut parsed_projects = Vec::with_capacity(projects.len());
    for (project_name, project_path, _) in projects {
        let joined_path = parent.join(project_path);
        let full_path = dunce::canonicalize(&joined_path).unwrap_or(joined_path);
        if !full_path.is_file() {
            return Err(parse_message(
                path,
                format!("引用的项目不存在: {}", full_path.display()),
            ));
        }
        let project = parse_project(&full_path, defaults.get(&project_name).cloned(), backend)?;
        if project.name != project_name {
            return Err(parse_message(
                path,
                format!(
                    "workspace 项目名 {project_name} 与项目文件中的名称 {} 不一致",
                    project.name
                ),
            ));
        }
        parsed_projects.push(project);
    }
    Ok(WorkspaceInfo {
        name,
        path: path.to_path_buf(),
        active_project,
        projects: parsed_projects,
    })
}

fn yq_json(path: &Path, yq: &Path) -> Result<Value> {
    let output = Command::new(yq)
        .args(["-p=xml", "-o=json", "."])
        .arg(path)
        .output()
        .map_err(|error| AppError::Tool {
            tool: yq.display().to_string(),
            message: error.to_string(),
        })?;
    if !output.status.success() {
        return Err(AppError::Tool {
            tool: yq.display().to_string(),
            message: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        });
    }
    serde_json::from_slice(&output.stdout).map_err(|error| AppError::Parse {
        path: path.to_path_buf(),
        message: format!("yq 输出不是有效 JSON: {error}"),
    })
}

fn values(value: Option<&Value>) -> Vec<&Value> {
    match value {
        Some(Value::Array(values)) => values.iter().collect(),
        Some(value) => vec![value],
        None => Vec::new(),
    }
}

fn attr<'a>(value: &'a Value, name: &str, path: &Path) -> Result<&'a str> {
    attr_optional(value, name).ok_or_else(|| parse_message(path, format!("缺少 XML 属性 {name}")))
}

fn attr_optional<'a>(value: &'a Value, name: &str) -> Option<&'a str> {
    value.get(format!("+@{name}"))?.as_str()
}

fn is_yes(value: Option<&str>) -> bool {
    value.is_some_and(|value| value.eq_ignore_ascii_case("yes"))
}

fn read(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|source| AppError::Read {
        path: path.to_path_buf(),
        source,
    })
}

fn parse_message(path: &Path, message: impl Into<String>) -> AppError {
    AppError::Parse {
        path: path.to_path_buf(),
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn parses_workspace_projects_and_build_matrix_defaults() {
        let temp = tempdir().unwrap();
        let project_dir = temp.path().join("app");
        fs::create_dir(&project_dir).unwrap();
        fs::write(
            project_dir.join("app.cdkproj"),
            r#"<Project Name="app" Language="C" Type="Application">
                <BuildConfigs>
                  <BuildConfig Name="Debug"/>
                  <BuildConfig Name="Release"/>
                </BuildConfigs>
              </Project>"#,
        )
        .unwrap();
        let workspace = temp.path().join("sample.cdkws");
        fs::write(
            &workspace,
            r#"<CDK_Workspace Name="sample">
                <Project Name="app" Path="app/app.cdkproj" Active="Yes"/>
                <BuildMatrix>
                  <WorkspaceConfiguration Name="Debug" Selected="yes">
                    <Project Name="app" ConfigName="Release"/>
                  </WorkspaceConfiguration>
                </BuildMatrix>
              </CDK_Workspace>"#,
        )
        .unwrap();

        let parsed = parse_workspace(&workspace, &XmlBackend::Native).unwrap();

        assert_eq!(parsed.name, "sample");
        assert_eq!(parsed.active_project.as_deref(), Some("app"));
        assert_eq!(parsed.projects[0].build_configs, ["Debug", "Release"]);
        assert_eq!(
            parsed.projects[0].default_build_config.as_deref(),
            Some("Release")
        );
    }
}
