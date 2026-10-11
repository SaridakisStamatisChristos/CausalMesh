use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("unsupported {kind} version {found}; expected {expected}")]
    UnsupportedVersion {
        kind: &'static str,
        found: u16,
        expected: u16,
    },
    #[error("corruption detected in {path}: {detail}")]
    Corruption { path: PathBuf, detail: String },
    #[error("resource bound exceeded: {0}")]
    Bounds(String),
    #[error("invalid operation: {0}")]
    InvalidOperation(String),
    #[error("operation id {0} already exists with a different payload")]
    ConflictingOperation(String),
    #[error("replica store is not initialized: {0}")]
    NotInitialized(PathBuf),
    #[error("replica store already exists: {0}")]
    AlreadyInitialized(PathBuf),
    #[error("invariant violation: {0}")]
    Invariant(String),
    #[error("injected fault at {0}")]
    InjectedFault(&'static str),
}

pub type Result<T> = std::result::Result<T, Error>;
