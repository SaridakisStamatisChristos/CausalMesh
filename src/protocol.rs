use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::causal::CausalContext;
use crate::error::{Error, Result};
use crate::model::Operation;
use crate::{MAX_BATCH_OPS, MAX_MESSAGE_BYTES, WIRE_VERSION};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "message", rename_all = "snake_case")]
pub enum WireMessage {
    Hello {
        version: u16,
        replica_id: Uuid,
    },
    Summary {
        version: u16,
        replica_id: Uuid,
        seen: CausalContext,
    },
    Operations {
        version: u16,
        from: Uuid,
        operations: Vec<Operation>,
    },
    Ack {
        version: u16,
        replica_id: Uuid,
        seen: CausalContext,
    },
}

impl WireMessage {
    pub fn encode(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let bytes = serde_json::to_vec(self)?;
        if bytes.len() > MAX_MESSAGE_BYTES {
            return Err(Error::Bounds(format!(
                "wire message exceeds {MAX_MESSAGE_BYTES} bytes"
            )));
        }
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_MESSAGE_BYTES {
            return Err(Error::Bounds(format!(
                "wire message exceeds {MAX_MESSAGE_BYTES} bytes"
            )));
        }
        let message: Self = serde_json::from_slice(bytes)?;
        message.validate()?;
        Ok(message)
    }

    pub fn validate(&self) -> Result<()> {
        let version = match self {
            Self::Hello { version, .. }
            | Self::Summary { version, .. }
            | Self::Operations { version, .. }
            | Self::Ack { version, .. } => *version,
        };
        if version != WIRE_VERSION {
            return Err(Error::UnsupportedVersion {
                kind: "wire",
                found: version,
                expected: WIRE_VERSION,
            });
        }
        match self {
            Self::Summary { seen, .. } | Self::Ack { seen, .. } => seen.validate_canonical(),
            Self::Operations { operations, .. } => {
                if operations.len() > MAX_BATCH_OPS {
                    return Err(Error::Bounds(format!(
                        "batch exceeds {MAX_BATCH_OPS} operations"
                    )));
                }
                for op in operations {
                    op.validate()?;
                }
                Ok(())
            }
            Self::Hello { .. } => Ok(()),
        }
    }
}
