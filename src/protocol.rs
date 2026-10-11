use crate::error::{Error, Result};
use crate::types::{
    Dot, Operation, ReplicaId, MAX_BATCH_OPS, MAX_DECODED_RANGES, MAX_MESSAGE_BYTES, WIRE_VERSION,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CounterRange {
    pub start: u64,
    pub end: u64,
}

impl CounterRange {
    fn contains(&self, counter: u64) -> bool {
        self.start <= counter && counter <= self.end
    }

    fn validate(&self) -> Result<()> {
        if self.start == 0 || self.start > self.end {
            return Err(Error::InvalidOperation(format!(
                "invalid counter range {}..={}",
                self.start, self.end
            )));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Summary {
    pub replica_id: ReplicaId,
    pub ranges: BTreeMap<ReplicaId, Vec<CounterRange>>,
}

impl Summary {
    pub fn from_operations<'a>(
        replica_id: ReplicaId,
        operations: impl Iterator<Item = &'a Operation>,
    ) -> Self {
        let mut counters: BTreeMap<ReplicaId, Vec<u64>> = BTreeMap::new();
        for op in operations {
            counters
                .entry(op.id.replica.clone())
                .or_default()
                .push(op.id.counter);
        }
        let ranges = counters
            .into_iter()
            .map(|(replica, mut values)| {
                values.sort_unstable();
                values.dedup();
                let mut out = Vec::new();
                for value in values {
                    match out.last_mut() {
                        Some(CounterRange { end, .. }) if end.checked_add(1) == Some(value) => {
                            *end = value;
                        }
                        _ => out.push(CounterRange {
                            start: value,
                            end: value,
                        }),
                    }
                }
                (replica, out)
            })
            .collect();
        Self { replica_id, ranges }
    }

    pub fn contains(&self, dot: &Dot) -> bool {
        self.ranges
            .get(&dot.replica)
            .map(|ranges| ranges.iter().any(|range| range.contains(dot.counter)))
            .unwrap_or(false)
    }

    pub fn validate(&self) -> Result<()> {
        self.replica_id.validate()?;
        let total_ranges: usize = self.ranges.values().map(Vec::len).sum();
        if total_ranges > MAX_DECODED_RANGES {
            return Err(Error::Bounds(format!(
                "summary has {total_ranges} ranges; max is {MAX_DECODED_RANGES}"
            )));
        }
        for (replica, ranges) in &self.ranges {
            replica.validate()?;
            let mut previous_end = 0u64;
            for range in ranges {
                range.validate()?;
                if range.start <= previous_end {
                    return Err(Error::InvalidOperation(
                        "summary ranges overlap or are unsorted".to_string(),
                    ));
                }
                previous_end = range.end;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WireMessage {
    Hello { replica_id: ReplicaId },
    Summary { summary: Summary },
    Operations { sender: ReplicaId, operations: Vec<Operation> },
    Ack { summary: Summary },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WireEnvelope {
    pub version: u16,
    pub message: WireMessage,
}

impl WireEnvelope {
    pub fn new(message: WireMessage) -> Self {
        Self {
            version: WIRE_VERSION,
            message,
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.version != WIRE_VERSION {
            return Err(Error::UnsupportedVersion {
                surface: "wire protocol",
                version: self.version,
            });
        }
        match &self.message {
            WireMessage::Hello { replica_id } => replica_id.validate(),
            WireMessage::Summary { summary } | WireMessage::Ack { summary } => summary.validate(),
            WireMessage::Operations { sender, operations } => {
                sender.validate()?;
                if operations.len() > MAX_BATCH_OPS {
                    return Err(Error::Bounds(format!(
                        "wire batch has {} operations; max is {}",
                        operations.len(),
                        MAX_BATCH_OPS
                    )));
                }
                for op in operations {
                    op.validate()?;
                }
                Ok(())
            }
        }
    }
}

pub fn encode_wire(envelope: &WireEnvelope) -> Result<Vec<u8>> {
    envelope.validate()?;
    let bytes = serde_json::to_vec(envelope)?;
    if bytes.len() > MAX_MESSAGE_BYTES {
        return Err(Error::Bounds(format!(
            "wire message is {} bytes; max is {}",
            bytes.len(),
            MAX_MESSAGE_BYTES
        )));
    }
    Ok(bytes)
}

pub fn decode_wire(bytes: &[u8]) -> Result<WireEnvelope> {
    if bytes.len() > MAX_MESSAGE_BYTES {
        return Err(Error::Bounds(format!(
            "wire message is {} bytes; max is {}",
            bytes.len(),
            MAX_MESSAGE_BYTES
        )));
    }
    let envelope: WireEnvelope = serde_json::from_slice(bytes)?;
    envelope.validate()?;
    Ok(envelope)
}
