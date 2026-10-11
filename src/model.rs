use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::causal::{CausalContext, Dot};
use crate::error::{Error, Result};
use crate::{MAX_KEY_BYTES, MAX_VALUE_BYTES};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum OpKind {
    Put(Vec<u8>),
    Delete,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Operation {
    pub id: Dot,
    pub context: CausalContext,
    pub key: String,
    pub kind: OpKind,
}

impl Operation {
    pub fn validate(&self) -> Result<()> {
        if self.id.counter == 0 {
            return Err(Error::InvalidOperation(
                "operation counter must be positive".into(),
            ));
        }
        if self.key.is_empty() {
            return Err(Error::InvalidOperation("key must not be empty".into()));
        }
        if self.key.len() > MAX_KEY_BYTES {
            return Err(Error::Bounds(format!("key exceeds {MAX_KEY_BYTES} bytes")));
        }
        if let OpKind::Put(value) = &self.kind {
            if value.len() > MAX_VALUE_BYTES {
                return Err(Error::Bounds(format!(
                    "value exceeds {MAX_VALUE_BYTES} bytes"
                )));
            }
        }
        self.context.validate_canonical()?;
        if self.context.contains(self.id) {
            return Err(Error::InvalidOperation(
                "operation context contains its own dot".into(),
            ));
        }
        if self
            .context
            .base()
            .get(&self.id.replica)
            .copied()
            .unwrap_or(0)
            >= self.id.counter
        {
            return Err(Error::InvalidOperation(
                "operation context contains a future/self local counter".into(),
            ));
        }
        if self
            .context
            .cloud()
            .iter()
            .any(|d| d.replica == self.id.replica && d.counter >= self.id.counter)
        {
            return Err(Error::InvalidOperation(
                "operation context contains future local dots".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct OperationSet {
    ops: BTreeMap<Dot, Operation>,
    seen: CausalContext,
}

impl OperationSet {
    pub fn from_operations<I>(ops: I) -> Result<Self>
    where
        I: IntoIterator<Item = Operation>,
    {
        let mut set = Self::default();
        for op in ops {
            set.insert(op)?;
        }
        Ok(set)
    }

    pub fn insert(&mut self, op: Operation) -> Result<bool> {
        op.validate()?;
        if let Some(existing) = self.ops.get(&op.id) {
            if existing == &op {
                return Ok(false);
            }
            return Err(Error::ConflictingOperation(op.id.to_string()));
        }
        self.seen.observe(op.id)?;
        self.ops.insert(op.id, op);
        Ok(true)
    }

    pub fn get(&self, key: &str) -> Vec<Vec<u8>> {
        self.maximal_for_key(key)
            .into_iter()
            .filter_map(|op| match &op.kind {
                OpKind::Put(value) => Some(value.clone()),
                OpKind::Delete => None,
            })
            .collect()
    }

    pub fn maximal_for_key(&self, key: &str) -> Vec<&Operation> {
        let candidates: Vec<&Operation> = self.ops.values().filter(|op| op.key == key).collect();
        candidates
            .iter()
            .copied()
            .filter(|candidate| {
                !candidates
                    .iter()
                    .any(|other| other.id != candidate.id && other.context.contains(candidate.id))
            })
            .collect()
    }

    pub fn logical_state(&self) -> BTreeMap<String, Vec<Vec<u8>>> {
        let keys: BTreeSet<String> = self.ops.values().map(|op| op.key.clone()).collect();
        keys.into_iter()
            .filter_map(|key| {
                let values = self.get(&key);
                (!values.is_empty()).then_some((key, values))
            })
            .collect()
    }

    pub fn operations(&self) -> &BTreeMap<Dot, Operation> {
        &self.ops
    }
    pub fn seen(&self) -> &CausalContext {
        &self.seen
    }
    pub fn missing_for(&self, peer: &CausalContext) -> Vec<Operation> {
        self.ops
            .values()
            .filter(|op| !peer.contains(op.id))
            .cloned()
            .collect()
    }
    pub fn unresolved_registers(&self) -> usize {
        let keys: BTreeSet<&str> = self.ops.values().map(|op| op.key.as_str()).collect();
        keys.into_iter()
            .filter(|key| self.maximal_for_key(key).len() > 1)
            .count()
    }
    pub fn tombstone_count(&self) -> usize {
        self.ops
            .values()
            .filter(|op| matches!(op.kind, OpKind::Delete))
            .count()
    }
}
