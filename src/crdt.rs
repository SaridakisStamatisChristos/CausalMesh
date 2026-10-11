use crate::error::{Error, Result};
use crate::types::{compare_operation_payloads, Dot, Operation, OperationKind, VersionVector};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RegisterEntry {
    pub dot: Dot,
    pub context: VersionVector,
    pub value: Option<Vec<u8>>,
}

impl RegisterEntry {
    fn from_operation(op: &Operation) -> Self {
        let value = match &op.kind {
            OperationKind::Put { value, .. } => Some(value.clone()),
            OperationKind::Delete { .. } => None,
        };
        Self {
            dot: op.id.clone(),
            context: op.context.clone(),
            value,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct LogicalState {
    pub clock: VersionVector,
    pub registers: BTreeMap<String, Vec<RegisterEntry>>,
}

#[derive(Clone, Debug, Default)]
pub struct CrdtState {
    clock: VersionVector,
    registers: BTreeMap<String, Vec<Operation>>,
    operations: BTreeMap<Dot, Operation>,
}

impl CrdtState {
    pub fn apply(&mut self, op: Operation) -> Result<bool> {
        op.validate()?;
        if let Some(existing) = self.operations.get(&op.id) {
            if existing == &op {
                return Ok(false);
            }
            return Err(Error::DuplicateOperationConflict(op.id));
        }

        let key = op.kind.key().to_string();
        let entries = self.registers.entry(key).or_default();
        let dominated_by_existing = entries
            .iter()
            .any(|existing| existing.context.dominates_dot(&op.id));

        if !dominated_by_existing {
            entries.retain(|existing| !op.context.dominates_dot(&existing.id));
            entries.push(op.clone());
            entries.sort_by(|left, right| {
                left.id.cmp(&right.id).then_with(|| {
                    compare_operation_payloads(left, right).unwrap_or(std::cmp::Ordering::Equal)
                })
            });
        }

        self.clock.join(&op.context);
        self.clock.observe(&op.id);
        self.operations.insert(op.id.clone(), op);
        Ok(true)
    }

    pub fn merge(&self, other: &Self) -> Result<Self> {
        let mut merged = self.clone();
        for op in other.operations.values() {
            merged.apply(op.clone())?;
        }
        Ok(merged)
    }

    pub fn get(&self, key: &str) -> Vec<Vec<u8>> {
        self.registers
            .get(key)
            .into_iter()
            .flat_map(|entries| entries.iter())
            .filter_map(|op| match &op.kind {
                OperationKind::Put { value, .. } => Some(value.clone()),
                OperationKind::Delete { .. } => None,
            })
            .collect()
    }

    pub fn entries(&self, key: &str) -> Vec<RegisterEntry> {
        self.registers
            .get(key)
            .into_iter()
            .flat_map(|entries| entries.iter())
            .map(RegisterEntry::from_operation)
            .collect()
    }

    pub fn clock(&self) -> &VersionVector {
        &self.clock
    }

    pub fn has_operation(&self, dot: &Dot) -> bool {
        self.operations.contains_key(dot)
    }

    pub fn operation(&self, dot: &Dot) -> Option<&Operation> {
        self.operations.get(dot)
    }

    pub fn operations(&self) -> impl Iterator<Item = &Operation> {
        self.operations.values()
    }

    pub fn operation_count(&self) -> usize {
        self.operations.len()
    }

    pub fn unresolved_registers(&self) -> usize {
        self.registers
            .values()
            .filter(|entries| entries.len() > 1)
            .count()
    }

    pub fn tombstone_count(&self) -> usize {
        self.registers
            .values()
            .flat_map(|entries| entries.iter())
            .filter(|op| matches!(&op.kind, OperationKind::Delete { .. }))
            .count()
    }

    pub fn export(&self) -> LogicalState {
        let registers = self
            .registers
            .iter()
            .map(|(key, entries)| {
                (
                    key.clone(),
                    entries.iter().map(RegisterEntry::from_operation).collect(),
                )
            })
            .collect();
        LogicalState {
            clock: self.clock.clone(),
            registers,
        }
    }

    pub fn canonical_state_bytes(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(&self.export())?)
    }
}
