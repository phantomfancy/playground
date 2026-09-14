use std::{io::Error, path::PathBuf, result};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Message(String),
    #[error("无法读取 {path}: {source}")]
    Read { path: PathBuf, source: Error },
    #[error("无法解析 {path}: {message}")]
    Parse { path: PathBuf, message: String },
    #[error("无法写入 {path}: {source}")]
    Write { path: PathBuf, source: Error },
    #[error("外部工具 {tool} 执行失败: {message}")]
    Tool { tool: String, message: String },
}

pub type Result<T> = result::Result<T, AppError>;

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Read { .. } => "io_read",
            Self::Parse { .. } => "parse_error",
            Self::Write { .. } => "io_write",
            Self::Tool { .. } => "external_tool",
            Self::Message(_) => "invalid_request",
        }
    }
}
