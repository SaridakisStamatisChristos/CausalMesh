use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crc32fast::Hasher as Crc32;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::model::Operation;
use crate::{LOG_VERSION, MAX_RECORD_BYTES, SNAPSHOT_VERSION, STATE_VERSION};

const OP_MAGIC: [u8; 4] = *b"CMOP";
const COUNTER_MAGIC: [u8; 4] = *b"CMCT";
const HEADER_LEN: u64 = 10;
const CRC_LEN: u64 = 4;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IdentityPayload {
    version: u16,
    replica_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IdentityEnvelope {
    payload: IdentityPayload,
    checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotPayload {
    pub version: u16,
    pub operations: Vec<Operation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SnapshotEnvelope {
    payload: SnapshotPayload,
    checksum: String,
}

#[derive(Debug)]
pub struct Store {
    root: PathBuf,
    replica_id: Uuid,
    reserved_counter: u64,
}

impl Store {
    pub fn init(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root)?;
        let identity_path = root.join("identity.json");
        let counter_path = root.join("counter.log");
        let ops_path = root.join("ops.log");
        if identity_path.exists() || counter_path.exists() || ops_path.exists() {
            return Err(Error::AlreadyInitialized(root));
        }
        let replica_id = Uuid::new_v4();
        write_identity(&identity_path, replica_id)?;
        create_synced_file(&counter_path)?;
        create_synced_file(&ops_path)?;
        sync_directory(&root)?;
        Ok(Self {
            root,
            replica_id,
            reserved_counter: 0,
        })
    }

    pub fn open(root: impl AsRef<Path>) -> Result<(Self, Vec<Operation>)> {
        let root = root.as_ref().to_path_buf();
        let identity_path = root.join("identity.json");
        let counter_path = root.join("counter.log");
        let ops_path = root.join("ops.log");
        if !identity_path.exists() || !counter_path.exists() || !ops_path.exists() {
            return Err(Error::NotInitialized(root));
        }
        let replica_id = read_identity(&identity_path)?;
        let counters = scan_frames(&counter_path, COUNTER_MAGIC, LOG_VERSION)?;
        let mut reserved_counter = 0u64;
        for payload in counters {
            if payload.len() != 8 {
                return Err(corruption(&counter_path, "counter record is not 8 bytes"));
            }
            let value = u64::from_le_bytes(payload.try_into().expect("length checked"));
            if value
                != reserved_counter
                    .checked_add(1)
                    .ok_or_else(|| Error::Invariant("counter overflow".into()))?
            {
                return Err(corruption(
                    &counter_path,
                    format!("counter transition {reserved_counter} -> {value} is impossible"),
                ));
            }
            reserved_counter = value;
        }
        let records = scan_frames(&ops_path, OP_MAGIC, LOG_VERSION)?;
        let mut operations = Vec::with_capacity(records.len());
        for payload in records {
            let op: Operation = serde_json::from_slice(&payload)?;
            op.validate()?;
            operations.push(op);
        }
        let max_local = operations
            .iter()
            .filter(|op| op.id.replica == replica_id)
            .map(|op| op.id.counter)
            .max()
            .unwrap_or(0);
        if max_local > reserved_counter {
            return Err(corruption(&counter_path, format!("local operation counter {max_local} exceeds durable reservation {reserved_counter}")));
        }
        Ok((
            Self {
                root,
                replica_id,
                reserved_counter,
            },
            operations,
        ))
    }

    pub fn replica_id(&self) -> Uuid {
        self.replica_id
    }
    pub fn reserved_counter(&self) -> u64 {
        self.reserved_counter
    }

    pub fn reserve_counter(&mut self) -> Result<u64> {
        let next = self
            .reserved_counter
            .checked_add(1)
            .ok_or_else(|| Error::Invariant("local counter exhausted".into()))?;
        append_frame(
            &self.root.join("counter.log"),
            COUNTER_MAGIC,
            LOG_VERSION,
            &next.to_le_bytes(),
            true,
        )?;
        self.reserved_counter = next;
        Ok(next)
    }

    pub fn append_operation(&self, op: &Operation) -> Result<()> {
        let payload = serde_json::to_vec(op)?;
        append_frame(
            &self.root.join("ops.log"),
            OP_MAGIC,
            LOG_VERSION,
            &payload,
            true,
        )
    }

    pub fn append_operations(&self, operations: &[Operation]) -> Result<()> {
        if operations.is_empty() {
            return Ok(());
        }
        let path = self.root.join("ops.log");
        let mut file = OpenOptions::new().append(true).open(&path)?;
        for op in operations {
            let payload = serde_json::to_vec(op)?;
            write_frame(&mut file, OP_MAGIC, LOG_VERSION, &payload)?;
        }
        file.sync_all()?;
        Ok(())
    }

    pub fn write_snapshot(&self, operations: &[Operation]) -> Result<PathBuf> {
        let payload = SnapshotPayload {
            version: SNAPSHOT_VERSION,
            operations: operations.to_vec(),
        };
        let payload_bytes = serde_json::to_vec(&payload)?;
        let checksum = hex::encode(Sha256::digest(&payload_bytes));
        let envelope = SnapshotEnvelope {
            payload,
            checksum: checksum.clone(),
        };
        let bytes = serde_json::to_vec(&envelope)?;
        if bytes.len() > MAX_RECORD_BYTES {
            return Err(Error::Bounds(
                "snapshot exceeds maximum supported size".into(),
            ));
        }
        let final_path = self.root.join(format!("snapshot-{}.json", &checksum[..16]));
        if final_path.exists() {
            let _ = read_snapshot(&final_path)?;
            return Ok(final_path);
        }
        let temp_path = self.root.join(format!(".snapshot-{}.tmp", Uuid::new_v4()));
        {
            let mut file = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&temp_path)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
        }
        fs::rename(&temp_path, &final_path)?;
        sync_directory(&self.root)?;
        Ok(final_path)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

pub fn read_snapshot(path: impl AsRef<Path>) -> Result<SnapshotPayload> {
    let path = path.as_ref();
    if fs::metadata(path)?.len() > MAX_RECORD_BYTES as u64 {
        return Err(Error::Bounds(
            "snapshot exceeds maximum supported size".into(),
        ));
    }
    let bytes = fs::read(path)?;
    let envelope: SnapshotEnvelope = serde_json::from_slice(&bytes)?;
    if envelope.payload.version != SNAPSHOT_VERSION {
        return Err(Error::UnsupportedVersion {
            kind: "snapshot",
            found: envelope.payload.version,
            expected: SNAPSHOT_VERSION,
        });
    }
    let payload_bytes = serde_json::to_vec(&envelope.payload)?;
    let actual = hex::encode(Sha256::digest(&payload_bytes));
    if actual != envelope.checksum {
        return Err(corruption(path, "snapshot checksum mismatch"));
    }
    for op in &envelope.payload.operations {
        op.validate()?;
    }
    Ok(envelope.payload)
}

fn write_identity(path: &Path, replica_id: Uuid) -> Result<()> {
    let payload = IdentityPayload {
        version: STATE_VERSION,
        replica_id,
    };
    let payload_bytes = serde_json::to_vec(&payload)?;
    let checksum = hex::encode(Sha256::digest(&payload_bytes));
    let envelope = IdentityEnvelope { payload, checksum };
    let bytes = serde_json::to_vec(&envelope)?;
    let mut file = OpenOptions::new().create_new(true).write(true).open(path)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    Ok(())
}

fn read_identity(path: &Path) -> Result<Uuid> {
    if fs::metadata(path)?.len() > 64 * 1024 {
        return Err(Error::Bounds("identity file too large".into()));
    }
    let bytes = fs::read(path)?;
    let envelope: IdentityEnvelope = serde_json::from_slice(&bytes)?;
    if envelope.payload.version != STATE_VERSION {
        return Err(Error::UnsupportedVersion {
            kind: "state",
            found: envelope.payload.version,
            expected: STATE_VERSION,
        });
    }
    let payload_bytes = serde_json::to_vec(&envelope.payload)?;
    let actual = hex::encode(Sha256::digest(&payload_bytes));
    if actual != envelope.checksum {
        return Err(corruption(path, "identity checksum mismatch"));
    }
    Ok(envelope.payload.replica_id)
}

fn create_synced_file(path: &Path) -> Result<()> {
    OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)?
        .sync_all()?;
    Ok(())
}

fn append_frame(
    path: &Path,
    magic: [u8; 4],
    version: u16,
    payload: &[u8],
    sync: bool,
) -> Result<()> {
    let mut file = OpenOptions::new().append(true).open(path)?;
    write_frame(&mut file, magic, version, payload)?;
    if sync {
        file.sync_all()?;
    }
    Ok(())
}

fn write_frame(file: &mut File, magic: [u8; 4], version: u16, payload: &[u8]) -> Result<()> {
    if payload.len() > MAX_RECORD_BYTES {
        return Err(Error::Bounds(format!(
            "record exceeds {MAX_RECORD_BYTES} bytes"
        )));
    }
    let len = u32::try_from(payload.len())
        .map_err(|_| Error::Bounds("record length exceeds u32".into()))?;
    let mut crc = Crc32::new();
    crc.update(&version.to_le_bytes());
    crc.update(&len.to_le_bytes());
    crc.update(payload);
    let checksum = crc.finalize();
    file.write_all(&magic)?;
    file.write_all(&version.to_le_bytes())?;
    file.write_all(&len.to_le_bytes())?;
    file.write_all(payload)?;
    file.write_all(&checksum.to_le_bytes())?;
    Ok(())
}

fn scan_frames(path: &Path, magic: [u8; 4], expected_version: u16) -> Result<Vec<Vec<u8>>> {
    let mut file = OpenOptions::new().read(true).write(true).open(path)?;
    let file_len = file.metadata()?.len();
    let mut position = 0u64;
    let mut out = Vec::new();
    while position < file_len {
        let remaining = file_len - position;
        if remaining < HEADER_LEN + CRC_LEN {
            file.seek(SeekFrom::Start(position))?;
            let mut tail = vec![0u8; remaining as usize];
            file.read_exact(&mut tail)?;
            if !recognizable_torn_prefix(&tail, magic, expected_version) {
                return Err(corruption(
                    path,
                    format!("unrecognized short tail at offset {position}"),
                ));
            }
            file.set_len(position)?;
            file.sync_all()?;
            break;
        }
        file.seek(SeekFrom::Start(position))?;
        let mut header = [0u8; HEADER_LEN as usize];
        file.read_exact(&mut header)?;
        if header[0..4] != magic {
            return Err(corruption(
                path,
                format!("bad record magic at offset {position}"),
            ));
        }
        let version = u16::from_le_bytes([header[4], header[5]]);
        if version != expected_version {
            return Err(Error::UnsupportedVersion {
                kind: "log",
                found: version,
                expected: expected_version,
            });
        }
        let len = u32::from_le_bytes([header[6], header[7], header[8], header[9]]) as usize;
        if len > MAX_RECORD_BYTES {
            return Err(Error::Bounds(format!(
                "declared record length {len} exceeds {MAX_RECORD_BYTES}"
            )));
        }
        let record_end = position
            .checked_add(HEADER_LEN)
            .and_then(|v| v.checked_add(len as u64))
            .and_then(|v| v.checked_add(CRC_LEN))
            .ok_or_else(|| Error::Bounds("record offset overflow".into()))?;
        if record_end > file_len {
            file.set_len(position)?;
            file.sync_all()?;
            break;
        }
        let mut payload = vec![0u8; len];
        file.read_exact(&mut payload)?;
        let mut crc_bytes = [0u8; 4];
        file.read_exact(&mut crc_bytes)?;
        let stored_crc = u32::from_le_bytes(crc_bytes);
        let mut crc = Crc32::new();
        crc.update(&version.to_le_bytes());
        crc.update(&(len as u32).to_le_bytes());
        crc.update(&payload);
        if crc.finalize() != stored_crc {
            return Err(corruption(
                path,
                format!("record checksum mismatch at offset {position}"),
            ));
        }
        out.push(payload);
        position = record_end;
    }
    Ok(out)
}

fn recognizable_torn_prefix(tail: &[u8], magic: [u8; 4], expected_version: u16) -> bool {
    if tail.is_empty() {
        return true;
    }
    let magic_prefix = tail.len().min(magic.len());
    if tail[..magic_prefix] != magic[..magic_prefix] {
        return false;
    }
    if tail.len() <= magic.len() {
        return true;
    }
    if tail.len() == 5 {
        return tail[4] == expected_version.to_le_bytes()[0];
    }
    let version = u16::from_le_bytes([tail[4], tail[5]]);
    version == expected_version
}

fn corruption(path: &Path, detail: impl Into<String>) -> Error {
    Error::Corruption {
        path: path.to_path_buf(),
        detail: detail.into(),
    }
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<()> {
    File::open(path)?.sync_all()?;
    Ok(())
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) -> Result<()> {
    Ok(())
}
