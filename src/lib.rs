//! CausalMesh: a deterministic, offline-first replicated-state engine.
//!
//! The v0.1 data model is `Map<Key, MVRegister<Value>>`. Causality is represented by
//! version vectors and operation dots `(replica_id, counter)`. Synchronization exchanges
//! exact received counter ranges so out-of-order delivery cannot hide holes.

pub mod codec;
pub mod crdt;
pub mod error;
pub mod protocol;
pub mod replica;
pub mod storage;
pub mod sync;
pub mod types;

pub use error::{Error, Result};
pub use protocol::{decode_wire, encode_wire, CounterRange, Summary, WireEnvelope, WireMessage};
pub use replica::{Replica, Status};
pub use sync::{sync_pair, SyncReport};
pub use types::{Dot, Operation, OperationKind, ReplicaId, VersionVector};
