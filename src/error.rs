use crate::types::Dot;
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid replica id: {0}")]
    InvalidReplicaId(String),
    #[error("unsupported {surface} version: {version}")]
    UnsupportedVersion { surface: &'static str, version: u16 },
    #[error("corrupt {surface}: {detail}")]
    Corrupt {
        surface: &'static str,
        detail: String,
    },
    #[error("resource bound exceeded: {0}")]
    Bounds(String),
    #[error("duplicate operation id {0} has a different payload")]
    DuplicateOperationConflict(Dot),
    #[error("invalid operation: {0}")]
    InvalidOperation(String),
    #[error("replica identity is missing while durable state exists")]
    IdentityMissing,
    #[error("replica identity file is invalid: {0}")]
    IdentityCorrupt(String),
    #[error("counter invariant violated: {0}")]
    CounterInvariant(String),
    #[error("peer appears to share this replica identity; refusing ambiguous history")]
    CloneIdentity,
    #[error("replica must be reopened after an interrupted durable write")]
    Poisoned,
    #[error("destructive compaction is unsafe without replica retirement/peer-liveness proof")]
    UnsafeCompaction,
    #[error("deterministic fault injection at {0}")]
    InjectedFault(&'static str),
}

pub type Result<T> = std::result::Result<T, Error>;
