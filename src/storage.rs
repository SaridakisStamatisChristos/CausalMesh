use crate::codec::{
    decode_counters, decode_log, decode_snapshot, encode_counter, encode_log_batch, encode_snapshot,
};
use crate::crdt::LogicalState;
use crate::error::{Error, Result};
use crate::protocol::Summary;
use crate::types::{Operation, ReplicaId, STATE_VERSION};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

const IDENTITY_FILE: &str = "identity.json";
const LOG_FILE: &str = "operations.log";
const COUNTER_FILE: &str = "counter.log";
const SNAPSHOT_FILE: &str = "snapshot.bin";

#[derive(Clone, Debug, Serialize, Deserialize)]
struct IdentityFile {
    state_version: u16,
    replica_id: ReplicaId,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SnapshotData {
    pub generation: u64,
    pub replica_id: ReplicaId,
    pub local_counter: u64,
    pub state: LogicalState,
    pub summary: Summary,
}

#[derive(Debug)]
pub struct RecoveryData {
    pub replica_id: ReplicaId,
    pub batches: Vec<Vec<Operation>>,
    pub durable_counter: u64,
    pub log_torn_tail: bool,
    pub counter_torn_tail: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailPoint {
    BeforeAppend,
    MidRecord,
    AfterAppendBeforeSync,
    AfterSyncBeforePublish,
}

impl FailPoint {
    fn label(self) -> &'static str {
        match self {
            Self::BeforeAppend => "before_append",
            Self::MidRecord => "mid_record",
            Self::AfterAppendBeforeSync => "after_append_before_sync",
            Self::AfterSyncBeforePublish => "after_sync_before_publish",
        }
    }
}

#[derive(Debug)]
pub struct Storage {
    dir: PathBuf,
}

impl Storage {
    pub fn open(path: impl AsRef<Path>) -> Result<(Self, RecoveryData)> {
        let dir = path.as_ref().to_path_buf();
        fs::create_dir_all(&dir)?;
        let storage = Self { dir };
        let identity_path = storage.identity_path();
        if !identity_path.exists() {
            if storage.has_durable_state()? {
                return Err(Error::IdentityMissing);
            }
            storage.create_identity()?;
        }
        let identity = storage.read_identity()?;
        let log_bytes = read_if_exists(&storage.log_path())?;
        let decoded_log = decode_log(&log_bytes)?;
        let counter_bytes = read_if_exists(&storage.counter_path())?;
        let (counters, counter_torn_tail) = decode_counters(&counter_bytes)?;
        let durable_counter = counters.last().copied().unwrap_or(0);

        if let Some(snapshot) = storage.read_snapshot()? {
            if snapshot.replica_id != identity.replica_id {
                return Err(Error::Corrupt {
                    surface: "snapshot",
                    detail: "snapshot replica identity does not match identity file".to_string(),
                });
            }
        }

        Ok((
            storage,
            RecoveryData {
                replica_id: identity.replica_id,
                batches: decoded_log.batches,
                durable_counter,
                log_torn_tail: decoded_log.torn_tail,
                counter_torn_tail,
            },
        ))
    }

    pub fn append_operations(&self, batch: &[Operation]) -> Result<()> {
        self.append_operations_with_failpoint(batch, None)
    }

    pub fn append_operations_with_failpoint(
        &self,
        batch: &[Operation],
        failpoint: Option<FailPoint>,
    ) -> Result<()> {
        let record = encode_log_batch(batch)?;
        append_record(&self.log_path(), &record, failpoint)
    }

    pub fn append_counter(&self, counter: u64) -> Result<()> {
        self.append_counter_with_failpoint(counter, None)
    }

    pub fn append_counter_with_failpoint(
        &self,
        counter: u64,
        failpoint: Option<FailPoint>,
    ) -> Result<()> {
        let record = encode_counter(counter);
        append_record(&self.counter_path(), &record, failpoint)
    }

    pub fn write_snapshot(&self, snapshot: &SnapshotData) -> Result<()> {
        let bytes = encode_snapshot(snapshot)?;
        let tmp = self.dir.join("snapshot.bin.tmp");
        {
            let mut file = File::create(&tmp)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
        }
        replace_file_portable(&tmp, &self.snapshot_path())?;
        sync_directory_best_effort(&self.dir)?;
        Ok(())
    }

    pub fn read_snapshot(&self) -> Result<Option<SnapshotData>> {
        let path = self.snapshot_path();
        if !path.exists() {
            return Ok(None);
        }
        let bytes = fs::read(path)?;
        Ok(Some(decode_snapshot(&bytes)?))
    }

    pub fn snapshot_generation(&self) -> Result<u64> {
        Ok(self.read_snapshot()?.map(|s| s.generation).unwrap_or(0))
    }

    pub fn log_bytes(&self) -> Result<u64> {
        match fs::metadata(self.log_path()) {
            Ok(meta) => Ok(meta.len()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(0),
            Err(err) => Err(err.into()),
        }
    }

    pub fn verify_files(&self) -> Result<()> {
        let _ = self.read_identity()?;
        let _ = decode_log(&read_if_exists(&self.log_path())?)?;
        let _ = decode_counters(&read_if_exists(&self.counter_path())?)?;
        let _ = self.read_snapshot()?;
        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.dir
    }

    fn create_identity(&self) -> Result<()> {
        let identity = IdentityFile {
            state_version: STATE_VERSION,
            replica_id: ReplicaId::random(),
        };
        let bytes = serde_json::to_vec_pretty(&identity)?;
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        let mut file = options.open(self.identity_path())?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        sync_directory_best_effort(&self.dir)?;
        Ok(())
    }

    fn read_identity(&self) -> Result<IdentityFile> {
        let bytes = fs::read(self.identity_path())?;
        let identity: IdentityFile = serde_json::from_slice(&bytes)
            .map_err(|err| Error::IdentityCorrupt(err.to_string()))?;
        if identity.state_version != STATE_VERSION {
            return Err(Error::UnsupportedVersion {
                surface: "state identity",
                version: identity.state_version,
            });
        }
        identity.replica_id.validate()?;
        Ok(identity)
    }

    fn has_durable_state(&self) -> Result<bool> {
        for path in [self.log_path(), self.counter_path(), self.snapshot_path()] {
            match fs::metadata(path) {
                Ok(meta) if meta.len() > 0 => return Ok(true),
                Ok(_) => {}
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                Err(err) => return Err(err.into()),
            }
        }
        Ok(false)
    }

    fn identity_path(&self) -> PathBuf {
        self.dir.join(IDENTITY_FILE)
    }
    fn log_path(&self) -> PathBuf {
        self.dir.join(LOG_FILE)
    }
    fn counter_path(&self) -> PathBuf {
        self.dir.join(COUNTER_FILE)
    }
    fn snapshot_path(&self) -> PathBuf {
        self.dir.join(SNAPSHOT_FILE)
    }
}

fn append_record(path: &Path, bytes: &[u8], failpoint: Option<FailPoint>) -> Result<()> {
    if failpoint == Some(FailPoint::BeforeAppend) {
        return Err(Error::InjectedFault(FailPoint::BeforeAppend.label()));
    }
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    if failpoint == Some(FailPoint::MidRecord) {
        let cut = (bytes.len() / 2).max(1);
        file.write_all(&bytes[..cut])?;
        file.sync_all()?;
        return Err(Error::InjectedFault(FailPoint::MidRecord.label()));
    }
    file.write_all(bytes)?;
    if failpoint == Some(FailPoint::AfterAppendBeforeSync) {
        return Err(Error::InjectedFault(
            FailPoint::AfterAppendBeforeSync.label(),
        ));
    }
    file.sync_all()?;
    if failpoint == Some(FailPoint::AfterSyncBeforePublish) {
        return Err(Error::InjectedFault(
            FailPoint::AfterSyncBeforePublish.label(),
        ));
    }
    Ok(())
}

fn read_if_exists(path: &Path) -> Result<Vec<u8>> {
    match fs::read(path) {
        Ok(bytes) => Ok(bytes),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(err) => Err(err.into()),
    }
}

fn replace_file_portable(tmp: &Path, dest: &Path) -> Result<()> {
    #[cfg(not(windows))]
    {
        fs::rename(tmp, dest)?;
        Ok(())
    }
    #[cfg(windows)]
    {
        let backup = dest.with_extension("bin.bak");
        if backup.exists() {
            fs::remove_file(&backup)?;
        }
        if dest.exists() {
            fs::rename(dest, &backup)?;
        }
        match fs::rename(tmp, dest) {
            Ok(()) => {
                if backup.exists() {
                    fs::remove_file(backup)?;
                }
                Ok(())
            }
            Err(err) => {
                if backup.exists() && !dest.exists() {
                    let _ = fs::rename(&backup, dest);
                }
                Err(err.into())
            }
        }
    }
}

fn sync_directory_best_effort(dir: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        File::open(dir)?.sync_all()?;
    }
    #[cfg(not(unix))]
    {
        let _ = dir;
    }
    Ok(())
}
