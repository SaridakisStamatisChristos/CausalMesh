use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt;
use uuid::Uuid;

pub const STATE_VERSION: u16 = 1;
pub const LOG_VERSION: u16 = 1;
pub const WIRE_VERSION: u16 = 1;
pub const SNAPSHOT_VERSION: u16 = 1;
pub const COUNTER_VERSION: u16 = 1;

pub const MAX_KEY_BYTES: usize = 1024;
pub const MAX_VALUE_BYTES: usize = 1024 * 1024;
pub const MAX_CONTEXT_ENTRIES: usize = 4096;
pub const MAX_BATCH_OPS: usize = 1024;
pub const MAX_RECORD_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_MESSAGE_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_DECODED_RANGES: usize = 16_384;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ReplicaId(pub String);

impl ReplicaId {
    pub fn random() -> Self { Self(Uuid::new_v4().to_string()) }
    pub fn parse(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        Uuid::parse_str(&value).map_err(|_| Error::InvalidReplicaId(value.clone()))?;
        Ok(Self(value))
    }
    pub fn validate(&self) -> Result<()> {
        Uuid::parse_str(&self.0).map(|_| ()).map_err(|_| Error::InvalidReplicaId(self.0.clone()))
    }
}

impl fmt::Display for ReplicaId { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(&self.0) } }

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Dot { pub replica: ReplicaId, pub counter: u64 }
impl fmt::Display for Dot { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}:{}", self.replica, self.counter) } }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CausalRelation { Before, Equal, After, Concurrent }

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VersionVector(pub BTreeMap<ReplicaId, u64>);
impl VersionVector {
    pub fn get(&self, replica: &ReplicaId) -> u64 { self.0.get(replica).copied().unwrap_or(0) }
    pub fn observe(&mut self, dot: &Dot) { self.0.entry(dot.replica.clone()).and_modify(|v| *v = (*v).max(dot.counter)).or_insert(dot.counter); }
    pub fn join(&mut self, other: &VersionVector) { for (r, c) in &other.0 { self.0.entry(r.clone()).and_modify(|v| *v = (*v).max(*c)).or_insert(*c); } }
    pub fn dominates_dot(&self, dot: &Dot) -> bool { self.get(&dot.replica) >= dot.counter }
    pub fn relation(&self, other: &VersionVector) -> CausalRelation {
        let a = all_ge(self, other); let b = all_ge(other, self);
        match (a,b) { (true,true)=>CausalRelation::Equal, (true,false)=>CausalRelation::After, (false,true)=>CausalRelation::Before, (false,false)=>CausalRelation::Concurrent }
    }
    pub fn validate(&self) -> Result<()> {
        if self.0.len() > MAX_CONTEXT_ENTRIES { return Err(Error::Bounds(format!("causal context has {} entries; max is {}", self.0.len(), MAX_CONTEXT_ENTRIES))); }
        for r in self.0.keys() { r.validate()?; }
        Ok(())
    }
}
fn all_ge(left: &VersionVector, right: &VersionVector) -> bool { right.0.iter().all(|(r,c)| left.get(r) >= *c) }

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OperationKind { Put { key: String, value: Vec<u8> }, Delete { key: String } }
impl OperationKind { pub fn key(&self) -> &str { match self { Self::Put{key,..}|Self::Delete{key} => key } } }

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Operation { pub id: Dot, pub context: VersionVector, pub kind: OperationKind }
impl Operation {
    pub fn validate(&self) -> Result<()> {
        self.id.replica.validate()?;
        if self.id.counter == 0 { return Err(Error::InvalidOperation("operation counters are 1-based".into())); }
        self.context.validate()?;
        let own = self.context.get(&self.id.replica);
        if own != self.id.counter - 1 { return Err(Error::InvalidOperation(format!("dot {} requires origin context {}, found {}", self.id, self.id.counter - 1, own))); }
        let key_len = self.kind.key().as_bytes().len();
        if key_len > MAX_KEY_BYTES { return Err(Error::Bounds(format!("key is {key_len} bytes; max is {MAX_KEY_BYTES}"))); }
        if let OperationKind::Put{value,..}=&self.kind { if value.len() > MAX_VALUE_BYTES { return Err(Error::Bounds(format!("value is {} bytes; max is {}", value.len(), MAX_VALUE_BYTES))); } }
        Ok(())
    }
    pub fn canonical_bytes(&self) -> Result<Vec<u8>> { Ok(serde_json::to_vec(self)?) }
}
pub fn compare_operation_payloads(left:&Operation,right:&Operation)->Result<Ordering>{Ok(left.canonical_bytes()?.cmp(&right.canonical_bytes()?))}
