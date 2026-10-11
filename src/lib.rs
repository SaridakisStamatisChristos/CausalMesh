//! CausalMesh: deterministic offline-first replicated state with explicit causal metadata.

pub mod causal;
pub mod error;
pub mod model;
pub mod protocol;
pub mod replica;
pub mod storage;

pub use causal::{CausalContext, Dot};
pub use error::{Error, Result};
pub use model::{OpKind, Operation};
pub use protocol::WireMessage;
pub use replica::{sync_bidirectional, FaultPoint, Replica, Status, SyncReport};

pub const STATE_VERSION: u16 = 1;
pub const LOG_VERSION: u16 = 1;
pub const WIRE_VERSION: u16 = 1;
pub const SNAPSHOT_VERSION: u16 = 1;

pub const MAX_KEY_BYTES: usize = 4 * 1024;
pub const MAX_VALUE_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_MESSAGE_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_BATCH_OPS: usize = 10_000;
pub const MAX_CAUSAL_ENTRIES: usize = 100_000;
pub const MAX_RECORD_BYTES: usize = MAX_MESSAGE_BYTES;
