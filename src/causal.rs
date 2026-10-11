use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::MAX_CAUSAL_ENTRIES;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Dot {
    pub replica: Uuid,
    pub counter: u64,
}

impl Dot {
    pub fn new(replica: Uuid, counter: u64) -> Result<Self> {
        if counter == 0 {
            return Err(Error::InvalidOperation(
                "dot counter must be positive".into(),
            ));
        }
        Ok(Self { replica, counter })
    }
}

impl fmt::Display for Dot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.replica, self.counter)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct CausalContext {
    base: BTreeMap<Uuid, u64>,
    cloud: BTreeSet<Dot>,
}

impl CausalContext {
    pub fn contains(&self, dot: Dot) -> bool {
        self.base.get(&dot.replica).copied().unwrap_or(0) >= dot.counter
            || self.cloud.contains(&dot)
    }

    pub fn observe(&mut self, dot: Dot) -> Result<()> {
        if dot.counter == 0 {
            return Err(Error::InvalidOperation(
                "cannot observe counter zero".into(),
            ));
        }
        let current = self.base.get(&dot.replica).copied().unwrap_or(0);
        if dot.counter <= current {
            return Ok(());
        }
        if dot.counter == current + 1 {
            self.base.insert(dot.replica, dot.counter);
            self.absorb(dot.replica);
        } else {
            self.cloud.insert(dot);
        }
        self.enforce_bound()
    }

    pub fn merge(&mut self, other: &Self) -> Result<()> {
        for (replica, counter) in &other.base {
            let entry = self.base.entry(*replica).or_insert(0);
            *entry = (*entry).max(*counter);
        }
        self.cloud.extend(other.cloud.iter().copied());
        self.normalize();
        self.enforce_bound()
    }

    pub fn validate_canonical(&self) -> Result<()> {
        self.enforce_bound()?;
        for (replica, counter) in &self.base {
            if *counter == 0 {
                return Err(Error::InvalidOperation(format!(
                    "zero base entry for {replica}"
                )));
            }
        }
        for dot in &self.cloud {
            if dot.counter == 0 {
                return Err(Error::InvalidOperation("zero cloud counter".into()));
            }
            let base = self.base.get(&dot.replica).copied().unwrap_or(0);
            if dot.counter <= base + 1 {
                return Err(Error::InvalidOperation(format!(
                    "non-canonical cloud dot {dot}; base is {base}"
                )));
            }
        }
        Ok(())
    }

    pub fn base(&self) -> &BTreeMap<Uuid, u64> {
        &self.base
    }
    pub fn cloud(&self) -> &BTreeSet<Dot> {
        &self.cloud
    }
    pub fn entry_count(&self) -> usize {
        self.base.len() + self.cloud.len()
    }

    fn normalize(&mut self) {
        self.cloud
            .retain(|dot| dot.counter > self.base.get(&dot.replica).copied().unwrap_or(0));
        let replicas: Vec<Uuid> = self.cloud.iter().map(|d| d.replica).collect();
        for replica in replicas {
            self.absorb(replica);
        }
        self.base.retain(|_, counter| *counter != 0);
    }

    fn absorb(&mut self, replica: Uuid) {
        loop {
            let current = self.base.get(&replica).copied().unwrap_or(0);
            let next = match current.checked_add(1) {
                Some(v) => v,
                None => break,
            };
            let dot = Dot {
                replica,
                counter: next,
            };
            if self.cloud.remove(&dot) {
                self.base.insert(replica, next);
            } else {
                break;
            }
        }
    }

    fn enforce_bound(&self) -> Result<()> {
        if self.entry_count() > MAX_CAUSAL_ENTRIES {
            return Err(Error::Bounds(format!(
                "causal context has {} entries; maximum is {}",
                self.entry_count(),
                MAX_CAUSAL_ENTRIES
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn holes_are_preserved_and_absorbed() {
        let id = Uuid::new_v4();
        let mut c = CausalContext::default();
        c.observe(Dot::new(id, 2).unwrap()).unwrap();
        assert!(!c.contains(Dot::new(id, 1).unwrap()));
        assert!(c.contains(Dot::new(id, 2).unwrap()));
        c.observe(Dot::new(id, 1).unwrap()).unwrap();
        assert_eq!(c.base().get(&id), Some(&2));
        assert!(c.cloud().is_empty());
    }
}
