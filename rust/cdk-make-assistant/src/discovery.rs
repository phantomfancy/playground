use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use ignore::{DirEntry, WalkBuilder};

use crate::{
    error::{AppError, Result},
    model::DiscoveryReport,
    parser::{XmlBackend, parse_project, parse_workspace},
};

#[derive(Clone, Debug)]
pub enum DiscoveryBackend {
    Native,
    Ripgrep(PathBuf),
}

pub fn discover(
    root: &Path,
    discovery_backend: &DiscoveryBackend,
    xml_backend: &XmlBackend,
) -> Result<DiscoveryReport> {
    let root = dunce::canonicalize(root).map_err(|source| AppError::Read {
        path: root.to_path_buf(),
        source,
    })?;
    let immediate = immediate_files(&root, "cdkws")?;
    let workspace_paths = if immediate.is_empty() {
        find_files(&root, "cdkws", discovery_backend)?
    } else {
        immediate
    };
    let mut workspaces = workspace_paths
        .iter()
        .map(|path| parse_workspace(path, xml_backend))
        .collect::<Result<Vec<_>>>()?;
    workspaces.sort_by(|a, b| a.path.cmp(&b.path));

    let standalone_projects = if workspaces.is_empty() {
        let immediate = immediate_files(&root, "cdkproj")?;
        let paths = if immediate.is_empty() {
            find_files(&root, "cdkproj", discovery_backend)?
        } else {
            immediate
        };
        paths
            .iter()
            .map(|path| parse_project(path, None, xml_backend))
            .collect::<Result<Vec<_>>>()?
    } else {
        Vec::new()
    };

    Ok(DiscoveryReport {
        root,
        workspaces,
        standalone_projects,
    })
}

fn immediate_files(root: &Path, extension: &str) -> Result<Vec<PathBuf>> {
    let mut paths = fs::read_dir(root)
        .map_err(|source| AppError::Read {
            path: root.to_path_buf(),
            source,
        })?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| is_extension(path, extension))
        .collect::<Vec<_>>();
    paths.sort();
    Ok(paths)
}

fn find_files(root: &Path, extension: &str, backend: &DiscoveryBackend) -> Result<Vec<PathBuf>> {
    let mut paths: Vec<PathBuf> = match backend {
        DiscoveryBackend::Native => WalkBuilder::new(root)
            .hidden(false)
            .git_ignore(true)
            .filter_entry(include_entry)
            .build()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.into_path())
            .filter(|path| is_extension(path, extension))
            .collect(),
        DiscoveryBackend::Ripgrep(rg) => {
            let glob = format!("*.{extension}");
            let output = Command::new(rg)
                .current_dir(root)
                .args(["--files", ".", "-g", &glob])
                .output()
                .map_err(|error| AppError::Tool {
                    tool: rg.display().to_string(),
                    message: error.to_string(),
                })?;
            if !output.status.success() && output.status.code() != Some(1) {
                return Err(AppError::Tool {
                    tool: rg.display().to_string(),
                    message: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
                });
            }
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| root.join(line))
                .collect()
        }
    };
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn include_entry(entry: &DirEntry) -> bool {
    if entry.depth() == 0 {
        return true;
    }
    !matches!(
        entry.file_name().to_string_lossy().as_ref(),
        ".git" | "node_modules" | "target" | "dist" | "out"
    )
}

fn is_extension(path: &Path, extension: &str) -> bool {
    path.is_file()
        && path
            .extension()
            .is_some_and(|value| value.eq_ignore_ascii_case(extension))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn immediate_workspace_takes_precedence_over_nested_workspace() {
        let temp = tempdir().unwrap();
        fs::write(temp.path().join("root.cdkws"), "").unwrap();
        fs::create_dir(temp.path().join("nested")).unwrap();
        fs::write(temp.path().join("nested/nested.cdkws"), "").unwrap();

        let found = immediate_files(temp.path(), "cdkws").unwrap();

        assert_eq!(found, vec![temp.path().join("root.cdkws")]);
    }
}
