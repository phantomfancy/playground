use std::{
    env, fs,
    path::{Path, PathBuf},
};

use directories::BaseDirs;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{
    error::{AppError, Result},
    model::BuildAction,
};

pub const LOCAL_CONFIG_NAME: &str = ".cdk-make-assistant.toml";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GlobalConfig {
    #[serde(default = "schema_version")]
    pub schema_version: u32,
    pub cdk_make_path: Option<PathBuf>,
    pub rg_path: Option<PathBuf>,
    pub yq_path: Option<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LocalConfig {
    #[serde(default = "schema_version")]
    pub schema_version: u32,
    pub workspace: Option<PathBuf>,
    pub project_file: Option<PathBuf>,
    pub project: Option<String>,
    pub build_config: Option<String>,
    pub action: Option<BuildAction>,
    #[serde(default)]
    pub all: bool,
}

fn schema_version() -> u32 {
    1
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            schema_version: schema_version(),
            cdk_make_path: None,
            rg_path: None,
            yq_path: None,
        }
    }
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self {
            schema_version: schema_version(),
            workspace: None,
            project_file: None,
            project: None,
            build_config: None,
            action: None,
            all: false,
        }
    }
}

pub fn global_config_path() -> Result<PathBuf> {
    BaseDirs::new()
        .map(|dirs| {
            dirs.config_dir()
                .join("cdk-make-assistant")
                .join("config.toml")
        })
        .ok_or_else(|| AppError::Message("无法确定用户配置目录".into()))
}

pub fn load_global() -> Result<GlobalConfig> {
    let path = global_config_path()?;
    if path.is_file() {
        return load_toml(&path);
    }
    Ok(GlobalConfig::default())
}

pub fn save_global(config: &GlobalConfig) -> Result<PathBuf> {
    let path = global_config_path()?;
    save_toml(&path, config)?;
    Ok(path)
}

pub fn local_config_path(root: &Path) -> PathBuf {
    root.join(LOCAL_CONFIG_NAME)
}

fn find_local(start: &Path) -> Option<(PathBuf, PathBuf)> {
    let mut current = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start.to_path_buf()
    };
    loop {
        let candidate = local_config_path(&current);
        if candidate.is_file() {
            return Some((current, candidate));
        }
        if !current.pop() {
            return None;
        }
    }
}

pub fn load_local(start: &Path) -> Result<Option<(PathBuf, LocalConfig)>> {
    let Some((root, path)) = find_local(start) else {
        return Ok(None);
    };
    Ok(Some((root, load_toml(&path)?)))
}

pub fn save_local(root: &Path, config: &LocalConfig) -> Result<PathBuf> {
    let path = local_config_path(root);
    save_toml(&path, config)?;
    Ok(path)
}

pub fn resolve_cdk_make(explicit: Option<PathBuf>, global: &GlobalConfig) -> Result<PathBuf> {
    let path = explicit
        .or_else(|| env::var_os("CDK_MAKE_PATH").map(PathBuf::from))
        .or_else(|| global.cdk_make_path.clone())
        .or_else(|| {
            let default = PathBuf::from(r"C:\Program Files\C-Sky\CDK\cdk-make.exe");
            default.is_file().then_some(default)
        })
        .ok_or_else(|| {
            AppError::Message(
                "未配置 cdk-make.exe；请运行 `cdk-make-assistant config set cdk-make <PATH>`"
                    .into(),
            )
        })?;
    if !path.is_file() {
        return Err(AppError::Message(format!(
            "cdk-make.exe 不存在: {}",
            path.display()
        )));
    }
    Ok(path)
}

fn load_toml<T>(path: &Path) -> Result<T>
where
    T: DeserializeOwned + Default,
{
    if !path.is_file() {
        return Ok(T::default());
    }
    let text = fs::read_to_string(path).map_err(|source| AppError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    toml::from_str(&text).map_err(|error| AppError::Parse {
        path: path.to_path_buf(),
        message: error.to_string(),
    })
}

fn save_toml<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| AppError::Write {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let text = toml::to_string_pretty(value).map_err(|error| {
        AppError::Message(format!("无法序列化配置 {}: {error}", path.display()))
    })?;
    let temp = path.with_extension("toml.tmp");
    fs::write(&temp, text).map_err(|source| AppError::Write {
        path: temp.clone(),
        source,
    })?;
    fs::rename(&temp, path).map_err(|source| AppError::Write {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_and_finds_local_config_at_project_root() {
        let temp = tempfile::tempdir().unwrap();
        let nested = temp.path().join("src").join("module");
        fs::create_dir_all(&nested).unwrap();
        let config = LocalConfig {
            project: Some("app".into()),
            build_config: Some("BuildSet".into()),
            ..LocalConfig::default()
        };

        let saved = save_local(temp.path(), &config).unwrap();
        let (root, loaded) = load_local(&nested).unwrap().unwrap();

        assert_eq!(saved, temp.path().join(".cdk-make-assistant.toml"));
        assert_eq!(root, temp.path());
        assert_eq!(loaded.project.as_deref(), Some("app"));
    }
}
