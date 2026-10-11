use std::fs::OpenOptions;
use std::io::Write;

use causalmesh::{storage::read_snapshot, Error, Replica};
use tempfile::TempDir;

#[test]
fn unrecognized_short_tail_is_corruption_not_torn_record() {
    let dir = TempDir::new().unwrap();
    let mut r = Replica::init(dir.path()).unwrap();
    r.put("k", b"v".to_vec()).unwrap();
    drop(r);
    let path = dir.path().join("ops.log");
    let mut f = OpenOptions::new().append(true).open(&path).unwrap();
    f.write_all(b"NO").unwrap();
    f.sync_all().unwrap();
    assert!(matches!(
        Replica::open(dir.path()),
        Err(Error::Corruption { .. })
    ));
}

#[test]
fn unsupported_log_version_fails_closed() {
    let dir = TempDir::new().unwrap();
    let r = Replica::init(dir.path()).unwrap();
    drop(r);
    let path = dir.path().join("ops.log");
    let mut f = OpenOptions::new().append(true).open(&path).unwrap();
    f.write_all(b"CMOP").unwrap();
    f.write_all(&2u16.to_le_bytes()).unwrap();
    f.write_all(&0u32.to_le_bytes()).unwrap();
    f.write_all(&0u32.to_le_bytes()).unwrap();
    f.sync_all().unwrap();
    assert!(matches!(
        Replica::open(dir.path()),
        Err(Error::UnsupportedVersion { .. })
    ));
}

#[test]
fn counter_rollback_relative_to_committed_operation_fails_closed() {
    let dir = TempDir::new().unwrap();
    let mut r = Replica::init(dir.path()).unwrap();
    r.put("k", b"v".to_vec()).unwrap();
    drop(r);
    std::fs::write(dir.path().join("counter.log"), []).unwrap();
    assert!(matches!(
        Replica::open(dir.path()),
        Err(Error::Corruption { .. })
    ));
}

#[test]
fn missing_identity_with_existing_state_fails_closed() {
    let dir = TempDir::new().unwrap();
    let mut r = Replica::init(dir.path()).unwrap();
    r.put("k", b"v".to_vec()).unwrap();
    drop(r);
    std::fs::remove_file(dir.path().join("identity.json")).unwrap();
    assert!(matches!(
        Replica::open(dir.path()),
        Err(Error::NotInitialized(_))
    ));
}

#[test]
fn oversized_identity_is_rejected_before_file_read() {
    let dir = TempDir::new().unwrap();
    let r = Replica::init(dir.path()).unwrap();
    drop(r);
    let path = dir.path().join("identity.json");
    OpenOptions::new()
        .write(true)
        .open(&path)
        .unwrap()
        .set_len(64 * 1024 + 1)
        .unwrap();
    assert!(matches!(Replica::open(dir.path()), Err(Error::Bounds(_))));
}

#[test]
fn oversized_snapshot_is_rejected_before_file_read() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("oversized.snapshot");
    OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)
        .unwrap()
        .set_len(causalmesh::MAX_RECORD_BYTES as u64 + 1)
        .unwrap();
    assert!(matches!(read_snapshot(path), Err(Error::Bounds(_))));
}

#[test]
fn snapshot_corruption_fails_closed() {
    let dir = TempDir::new().unwrap();
    let mut r = Replica::init(dir.path()).unwrap();
    r.put("k", b"v".to_vec()).unwrap();
    let path = r.snapshot().unwrap();
    let mut bytes = std::fs::read(&path).unwrap();
    let index = bytes.len() / 2;
    bytes[index] ^= 1;
    std::fs::write(&path, bytes).unwrap();
    assert!(read_snapshot(path).is_err());
}
