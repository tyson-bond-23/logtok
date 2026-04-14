use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TokeniserError {
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },

    #[error("Cannot read file: {path}: {source}")]
    FileReadError {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Invalid block size: {size} (must be between 1024 and 104857600)")]
    InvalidBlockSize { size: usize },

    #[error("JSON parse error at line {line}: {source}")]
    JsonParseError {
        line: usize,
        source: serde_json::Error,
    },

    #[error("Write error: {0}")]
    WriteError(#[from] std::io::Error),
}
