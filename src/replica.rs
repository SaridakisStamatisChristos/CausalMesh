use crate::crdt::{CrdtState, LogicalState, RegisterEntry};
use crate::error::{Error, Result};
use crate::protocol::Summary;
use crate::storage::{SnapshotData, Storage};
use crate::types::{Dot, Operation, OperationKind, ReplicaId, MAX_BATCH_OPS};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Status {
    pub replica_id: ReplicaId,
    pub local_counter: u64,
    pub known_replica_count: usize,
    pub operation_count: usize,
    pub snapshot_generation: u64,
    pub unresolved_concurrent_registers: usize,
    pub tombstone_count: usize,
    pub log_bytes: u64,
}

#[derive(Debug)]
pub struct Replica {
    storage: Storage,
    state: CrdtState,
    replica_id: ReplicaId,
    local_counter: u64,
    poisoned: bool,
}

impl Replica {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let (storage, recovery) = Storage::open(path)?;
        let mut state = CrdtState::default();
        for batch in &recovery.batches {
            if batch.len() > MAX_BATCH_OPS {
                return Err(Error::Bounds(format!(
                    "persisted batch has {} operations; max is {}",
                    batch.len(),
                    MAX_BATCH_OPS
                )));
            }
            for op in batch {
                state.apply(op.clone())?;
            }
        }

        let max_local_op = state
            .operations()
            .filter(|op| op.id.replica == recovery.replica_id)
            .map(|op| op.id.counter)
            .max()
            .unwrap_or(0);

        if recovery.durable_counter > max_local_op {
            return Err(Error::CounterInvariant(format!(
                "counter log is at {}, but highest durable local operation is {}",
                recovery.durable_counter, max_local_op
            )));
        }
        let local_counter = recovery.durable_counter.max(max_local_op);
        if recovery.durable_counter < max_local_op {
            storage.append_counter(local_counter)?;
        }

        if state.clock().get(&recovery.replica_id) > local_counter {
            return Err(Error::CounterInvariant(
                "causal clock claims unseen future local operations".to_string(),
            ));
        }

        let _ = (recovery.log_torn_tail, recovery.counter_torn_tail);
        Ok(Self {
            storage,
            state,
            replica_id: recovery.replica_id,
            local_counter,
            poisoned: false,
        })
    }

    pub fn id(&self) -> &ReplicaId {
        &self.replica_id
    }

    pub fn path(&self) -> &Path {
        self.storage.path()
    }

    pub fn put(&mut self, key: impl Into<String>, value: impl Into<Vec<u8>>) -> Result<Dot> {
        self.local_mutation(OperationKind::Put {
            key: key.into(),
            value: value.into(),
        })
    }

    pub fn delete(&mut self, key: impl Into<String>) -> Result<Dot> {
        self.local_mutation(OperationKind::Delete { key: key.into() })
    }

    pub fn get(&self, key: &str) -> Result<Vec<Vec<u8>>> {
        self.ensure_live()?;
        Ok(self.state.get(key))
    }

    pub fn entries(&self, key: &str) -> Result<Vec<RegisterEntry>> {
        self.ensure_live()?;
        Ok(self.state.entries(key))
    }

    pub fn summary(&self) -> Result<Summary> {
        self.ensure_live()?;
        Ok(Summary::from_operations(
            self.replica_id.clone(),
            self.state.operations(),
        ))
    }

    pub fn operations_missing_from(&self, peer: &Summary) -> Result<Vec<Operation>> {
        self.ensure_live()?;
        peer.validate()?;
        Ok(self
            .state
            .operations()
            .filter(|op| !peer.contains(&op.id))
            .cloned()
            .collect())
    }

    pub fn receive_operations(&mut self, operations: &[Operation]) -> Result<usize> {
        self.ensure_live()?;
        if operations.is_empty() {
            return Ok(0);
        }
        if operations.len() > MAX_BATCH_OPS {
            return Err(Error::Bounds(format!(
                "incoming batch has {} operations; max is {}",
                operations.len(),
                MAX_BATCH_OPS
            )));
        }
        let preview = self.preview_receive(operations)?;
        if let Err(err) = self.storage.append_operations(operations) {
            self.poisoned = true;
            return Err(err);
        }
        let before = self.state.operation_count();
        self.state = preview;
        Ok(self.state.operation_count() - before)
    }

    pub fn verify(&self) -> Result<()> {
        self.ensure_live()?;
        self.storage.verify_files()?;
        let canonical_a = self.state.canonical_state_bytes()?;
        let canonical_b = serde_json::to_vec(&self.state.export())?;
        if canonical_a != canonical_b {
            return Err(Error::Corrupt {
                surface: "logical state",
                detail: "canonical serialization is unstable".to_string(),
            });
        }
        Ok(())
    }

    pub fn snapshot(&self) -> Result<SnapshotData> {
        self.ensure_live()?;
        let snapshot = SnapshotData {
            generation: self.storage.snapshot_generation()?.checked_add(1).ok_or_else(|| {
                Error::CounterInvariant("snapshot generation overflow".to_string())
            })?,
            replica_id: self.replica_id.clone(),
            local_counter: self.local_counter,
            state: self.state.export(),
            summary: self.summary()?,
        };
        self.storage.write_snapshot(&snapshot)?;
        Ok(snapshot)
    }

    pub fn status(&self) -> Result<Status> {
        self.ensure_live()?;
        Ok(Status {
            replica_id: self.replica_id.clone(),
            local_counter: self.local_counter,
            known_replica_count: self.state.clock().0.len(),
            operation_count: self.state.operation_count(),
            snapshot_generation: self.storage.snapshot_generation()?,
            unresolved_concurrent_registers: self.state.unresolved_registers(),
            tombstone_count: self.state.tombstone_count(),
            log_bytes: self.storage.log_bytes()?,
        })
    }

    pub fn logical_state(&self) -> Result<LogicalState> {
        self.ensure_live()?;
        Ok(self.state.export())
    }

    pub fn operations(&self) -> Result<Vec<Operation>> {
        self.ensure_live()?;
        Ok(self.state.operations().cloned().collect())
    }

    pub fn compact_safely(&self) -> Result<SnapshotData> {
        let snapshot = self.snapshot()?;
        // The snapshot is useful for inspection/recovery research, but the operation log is
        // deliberately retained because v0.1 has no membership/retirement proof that would
        // make deleting history safe for an arbitrarily stale valid replica.
        Ok(snapshot)
    }

    fn local_mutation(&mut self, kind: OperationKind) -> Result<Dot> {
        self.ensure_live()?;
        let next = self
            .local_counter
            .checked_add(1)
            .ok_or_else(|| Error::CounterInvariant("local counter overflow".to_string()))?;
        let id = Dot {
            replica: self.replica_id.clone(),
            counter: next,
        };
        let op = Operation {
            id: id.clone(),
            context: self.state.clock().clone(),
            kind,
        };
        let mut preview = self.state.clone();
        preview.apply(op.clone())?;

        if let Err(err) = self.storage.append_operations(std::slice::from_ref(&op)) {
            self.poisoned = true;
            return Err(err);
        }
        if let Err(err) = self.storage.append_counter(next) {
            self.poisoned = true;
            return Err(err);
        }
        self.state = preview;
        self.local_counter = next;
        Ok(id)
    }

    fn preview_receive(&self, operations: &[Operation]) -> Result<CrdtState> {
        let mut preview = self.state.clone();
        for op in operations {
            op.validate()?;
            if op.context.get(&self.replica_id) > self.local_counter {
                return Err(Error::CloneIdentity);
            }
            if op.id.replica == self.replica_id && !self.state.has_operation(&op.id) {
                return Err(Error::CloneIdentity);
            }
            preview.apply(op.clone())?;
        }
        Ok(preview)
    }

    fn ensure_live(&self) -> Result<()> {
        if self.poisoned {
            Err(Error::Poisoned)
        } else {
            Ok(())
        }
    }
}
