use crate::error::{Error, Result};
use crate::types::{
    Operation, COUNTER_VERSION, LOG_VERSION, MAX_RECORD_BYTES, SNAPSHOT_VERSION,
};
use crc32fast::Hasher;
use serde::{de::DeserializeOwned, Serialize};

const LOG_MAGIC: [u8; 4] = *b"CMOP";
const SNAPSHOT_MAGIC: [u8; 4] = *b"CMSN";
const COUNTER_MAGIC: [u8; 4] = *b"CMCT";
const FRAME_HEADER: usize = 10;
const FRAME_FOOTER: usize = 4;
const COUNTER_FRAME_LEN: usize = 4 + 2 + 8 + 4;

#[derive(Debug)]
pub struct DecodeLog {
    pub batches: Vec<Vec<Operation>>,
    pub torn_tail: bool,
}

pub fn encode_log_batch(batch: &[Operation]) -> Result<Vec<u8>> {
    let payload = serde_json::to_vec(batch)?;
    encode_frame(LOG_MAGIC, LOG_VERSION, &payload)
}

pub fn decode_log(bytes: &[u8]) -> Result<DecodeLog> {
    let mut offset = 0usize;
    let mut batches = Vec::new();
    let mut torn_tail = false;

    while offset < bytes.len() {
        let remaining = &bytes[offset..];
        if remaining.len() < FRAME_HEADER {
            let magic_prefix_len = remaining.len().min(LOG_MAGIC.len());
            if remaining[..magic_prefix_len] == LOG_MAGIC[..magic_prefix_len] {
                torn_tail = true;
                break;
            }
            return Err(Error::Corrupt {
                surface: "operation log",
                detail: format!("unrecognized {}-byte tail", remaining.len()),
            });
        }
        if remaining[..4] != LOG_MAGIC {
            return Err(Error::Corrupt {
                surface: "operation log",
                detail: format!("bad magic at byte {offset}"),
            });
        }
        let version = u16::from_le_bytes([remaining[4], remaining[5]]);
        if version != LOG_VERSION {
            return Err(Error::UnsupportedVersion {
                surface: "operation log",
                version,
            });
        }
        let payload_len =
            u32::from_le_bytes([remaining[6], remaining[7], remaining[8], remaining[9]]) as usize;
        if payload_len > MAX_RECORD_BYTES {
            return Err(Error::Bounds(format!(
                "operation-log record declares {payload_len} bytes; max is {MAX_RECORD_BYTES}"
            )));
        }
        let total = FRAME_HEADER
            .checked_add(payload_len)
            .and_then(|v| v.checked_add(FRAME_FOOTER))
            .ok_or_else(|| Error::Bounds("operation-log record length overflow".to_string()))?;
        if remaining.len() < total {
            torn_tail = true;
            break;
        }
        let payload = &remaining[FRAME_HEADER..FRAME_HEADER + payload_len];
        verify_crc(
            payload,
            &remaining[FRAME_HEADER + payload_len..total],
            "operation log",
        )?;
        let batch: Vec<Operation> = serde_json::from_slice(payload)?;
        batches.push(batch);
        offset += total;
    }

    Ok(DecodeLog { batches, torn_tail })
}

pub fn encode_snapshot<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let payload = serde_json::to_vec(value)?;
    encode_frame(SNAPSHOT_MAGIC, SNAPSHOT_VERSION, &payload)
}

pub fn decode_snapshot<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    let payload = decode_exact_frame(bytes, SNAPSHOT_MAGIC, SNAPSHOT_VERSION, "snapshot")?;
    Ok(serde_json::from_slice(payload)?)
}

pub fn encode_counter(counter: u64) -> Vec<u8> {
    let mut out = Vec::with_capacity(COUNTER_FRAME_LEN);
    out.extend_from_slice(&COUNTER_MAGIC);
    out.extend_from_slice(&COUNTER_VERSION.to_le_bytes());
    let payload = counter.to_le_bytes();
    out.extend_from_slice(&payload);
    let mut hasher = Hasher::new();
    hasher.update(&payload);
    out.extend_from_slice(&hasher.finalize().to_le_bytes());
    out
}

pub fn decode_counters(bytes: &[u8]) -> Result<(Vec<u64>, bool)> {
    let mut offset = 0usize;
    let mut counters = Vec::new();
    let mut torn_tail = false;
    while offset < bytes.len() {
        let remaining = &bytes[offset..];
        if remaining.len() < COUNTER_FRAME_LEN {
            let magic_prefix_len = remaining.len().min(COUNTER_MAGIC.len());
            if remaining[..magic_prefix_len] == COUNTER_MAGIC[..magic_prefix_len] {
                torn_tail = true;
                break;
            }
            return Err(Error::Corrupt {
                surface: "counter log",
                detail: format!("unrecognized {}-byte tail", remaining.len()),
            });
        }
        if remaining[..4] != COUNTER_MAGIC {
            return Err(Error::Corrupt {
                surface: "counter log",
                detail: format!("bad magic at byte {offset}"),
            });
        }
        let version = u16::from_le_bytes([remaining[4], remaining[5]]);
        if version != COUNTER_VERSION {
            return Err(Error::UnsupportedVersion {
                surface: "counter log",
                version,
            });
        }
        let payload = &remaining[6..14];
        verify_crc(payload, &remaining[14..18], "counter log")?;
        let counter = u64::from_le_bytes(payload.try_into().expect("fixed 8-byte slice"));
        counters.push(counter);
        offset += COUNTER_FRAME_LEN;
    }
    Ok((counters, torn_tail))
}

fn encode_frame(magic: [u8; 4], version: u16, payload: &[u8]) -> Result<Vec<u8>> {
    if payload.len() > MAX_RECORD_BYTES {
        return Err(Error::Bounds(format!(
            "record is {} bytes; max is {}",
            payload.len(),
            MAX_RECORD_BYTES
        )));
    }
    let len = u32::try_from(payload.len())
        .map_err(|_| Error::Bounds("record length does not fit u32".to_string()))?;
    let mut out = Vec::with_capacity(FRAME_HEADER + payload.len() + FRAME_FOOTER);
    out.extend_from_slice(&magic);
    out.extend_from_slice(&version.to_le_bytes());
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(payload);
    let mut hasher = Hasher::new();
    hasher.update(payload);
    out.extend_from_slice(&hasher.finalize().to_le_bytes());
    Ok(out)
}

fn decode_exact_frame<'a>(
    bytes: &'a [u8],
    magic: [u8; 4],
    expected_version: u16,
    surface: &'static str,
) -> Result<&'a [u8]> {
    if bytes.len() < FRAME_HEADER + FRAME_FOOTER {
        return Err(Error::Corrupt {
            surface,
            detail: "truncated frame".to_string(),
        });
    }
    if bytes[..4] != magic {
        return Err(Error::Corrupt {
            surface,
            detail: "bad magic".to_string(),
        });
    }
    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    if version != expected_version {
        return Err(Error::UnsupportedVersion { surface, version });
    }
    let payload_len = u32::from_le_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]) as usize;
    if payload_len > MAX_RECORD_BYTES {
        return Err(Error::Bounds(format!(
            "{surface} declares {payload_len} bytes; max is {MAX_RECORD_BYTES}"
        )));
    }
    let expected = FRAME_HEADER
        .checked_add(payload_len)
        .and_then(|v| v.checked_add(FRAME_FOOTER))
        .ok_or_else(|| Error::Bounds(format!("{surface} length overflow")))?;
    if bytes.len() != expected {
        return Err(Error::Corrupt {
            surface,
            detail: format!("frame length {}, expected {expected}", bytes.len()),
        });
    }
    let payload = &bytes[FRAME_HEADER..FRAME_HEADER + payload_len];
    verify_crc(payload, &bytes[FRAME_HEADER + payload_len..], surface)?;
    Ok(payload)
}

fn verify_crc(payload: &[u8], crc_bytes: &[u8], surface: &'static str) -> Result<()> {
    if crc_bytes.len() != 4 {
        return Err(Error::Corrupt {
            surface,
            detail: "truncated checksum".to_string(),
        });
    }
    let expected = u32::from_le_bytes(crc_bytes.try_into().expect("fixed 4-byte slice"));
    let mut hasher = Hasher::new();
    hasher.update(payload);
    let observed = hasher.finalize();
    if expected != observed {
        return Err(Error::Corrupt {
            surface,
            detail: format!("checksum mismatch: expected {expected:08x}, observed {observed:08x}"),
        });
    }
    Ok(())
}
