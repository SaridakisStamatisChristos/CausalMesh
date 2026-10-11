use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::Serialize;
use uuid::Uuid;

use crate::causal::{CausalContext, Dot};
use crate::error::{Error, Result};
use crate::model::{OpKind, Operation, OperationSet};
use crate::storage::Store;
use crate::{MAX_BATCH_OPS, MAX_MESSAGE_BYTES};

#[derive(Debug)]
pub struct Replica { store: Store, engine: OperationSet }

#[derive(Debug, Serialize)]
pub struct Status {
    pub replica_id: Uuid,
    pub reserved_counter: u64,
    pub operation_count: usize,
    pub known_replica_count: usize,
    pub unresolved_concurrent_registers: usize,
    pub tombstone_count: usize,
    pub log_bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct SyncReport { pub a_to_b: usize, pub b_to_a: usize }

impl Replica {
    pub fn init(path: impl AsRef<Path>) -> Result<Self> {
        let store = Store::init(path)?;
        Ok(Self { store, engine: OperationSet::default() })
    }
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let (store, operations) = Store::open(path)?;
        let engine = OperationSet::from_operations(operations)?;
        Ok(Self { store, engine })
    }
    pub fn replica_id(&self) -> Uuid { self.store.replica_id() }
    pub fn put(&mut self, key: impl Into<String>, value: impl Into<Vec<u8>>) -> Result<Dot> {
        self.local_mutation(key.into(), OpKind::Put(value.into()))
    }
    pub fn delete(&mut self, key: impl Into<String>) -> Result<Dot> { self.local_mutation(key.into(), OpKind::Delete) }
    pub fn get(&self, key: &str) -> Vec<Vec<u8>> { self.engine.get(key) }
    pub fn summary(&self) -> CausalContext { self.engine.seen().clone() }
    pub fn export_missing(&self, peer: &CausalContext) -> Vec<Operation> { self.engine.missing_for(peer) }

    pub fn receive_batch(&mut self, operations: &[Operation]) -> Result<usize> {
        if operations.len() > MAX_BATCH_OPS { return Err(Error::Bounds(format!("batch exceeds {MAX_BATCH_OPS} operations"))); }
        let mut bytes = 0usize;
        let mut batch_ids: BTreeMap<Dot, &Operation> = BTreeMap::new();
        let mut missing = Vec::new();
        for op in operations {
            op.validate()?;
            bytes = bytes.checked_add(serde_json::to_vec(op)?.len()).ok_or_else(|| Error::Bounds("batch byte count overflow".into()))?;
            if bytes > MAX_MESSAGE_BYTES { return Err(Error::Bounds(format!("batch exceeds {MAX_MESSAGE_BYTES} bytes"))); }
            if let Some(prior) = batch_ids.insert(op.id, op) {
                if prior != op { return Err(Error::ConflictingOperation(op.id.to_string())); }
            }
            if let Some(existing) = self.engine.operations().get(&op.id) {
                if existing != op { return Err(Error::ConflictingOperation(op.id.to_string())); }
            } else { missing.push(op.clone()); }
        }
        self.store.append_operations(&missing)?;
        for op in &missing {
            let inserted = self.engine.insert(op.clone())?;
            debug_assert!(inserted);
        }
        Ok(missing.len())
    }

    pub fn status(&self) -> Result<Status> {
        let known: BTreeSet<Uuid> = self.engine.operations().keys().map(|dot| dot.replica).collect();
        let log_bytes = std::fs::metadata(self.store.root().join("ops.log"))?.len();
        Ok(Status {
            replica_id: self.replica_id(), reserved_counter: self.store.reserved_counter(), operation_count: self.engine.operations().len(),
            known_replica_count: known.len(), unresolved_concurrent_registers: self.engine.unresolved_registers(), tombstone_count: self.engine.tombstone_count(), log_bytes,
        })
    }
    pub fn logical_state(&self) -> BTreeMap<String, Vec<Vec<u8>>> { self.engine.logical_state() }
    pub fn operations(&self) -> Vec<Operation> { self.engine.operations().values().cloned().collect() }
    pub fn snapshot(&self) -> Result<PathBuf> { self.store.write_snapshot(&self.operations()) }

    fn local_mutation(&mut self, key: String, kind: OpKind) -> Result<Dot> {
        let counter = self.store.reserve_counter()?;
        let id = Dot::new(self.replica_id(), counter)?;
        let op = Operation { id, context: self.engine.seen().clone(), key, kind };
        op.validate()?;
        self.store.append_operation(&op)?;
        self.engine.insert(op)?;
        Ok(id)
    }
}

pub fn sync_bidirectional(a: &mut Replica, b: &mut Replica) -> Result<SyncReport> {
    let a_summary = a.summary();
    let b_summary = b.summary();
    let a_to_b = a.export_missing(&b_summary);
    let b_to_a = b.export_missing(&a_summary);
    let a_to_b_count = b.receive_batch(&a_to_b)?;
    let b_to_a_count = a.receive_batch(&b_to_a)?;
    Ok(SyncReport { a_to_b: a_to_b_count, b_to_a: b_to_a_count })
}
