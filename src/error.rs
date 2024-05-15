use thiserror::Error;

#[derive(Error, Debug)]
pub enum BingoError {
    #[error("file not found: {0}")]
    FileNotFound(String),
    #[error("duplicate executable name: {0}")]
    DuplicateExecutableName(String),
    #[error("config file not found: {0}")]
    ConfigFileNotFound(String),
    #[error("config file error: {0}")]
    ConfigFileError(String),
    #[error("executable must be a file: {0}")]
    ExecutableNotFile(String),
    #[error("executable cannot be executed: {0}")]
    ExecutableNotExecutable(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

pub type BingoResult<T> = Result<T, BingoError>;
