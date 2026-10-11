mod common;

use causalmesh::storage::{FailPoint, Storage};
use causalmesh::{Replica, VersionVector};
use common::put;
use std::fs;
use tempfile::TempDir;

#[test]
fn torn_final_operation_record_is_ignored_and_recovery_continues() {
    let dir = TempDir::new().unwrap();
    let (storage, recovery) = Storage::open(dir.path()).unwrap();
    let op = put(recovery.replica_id.clone(), 1, VersionVector::default(), "k", "v");
    let err = storage
        .append_operations_with_failpoint(&[op], Some(FailPoint::MidRecord))
        .unwrap_err();
    assert!(err.to_string().contains("mid_record"));
    drop(storage);

    let reopened = Replica::open(dir.path()).unwrap();
    assert_eq!(reopened.status().unwrap().operation_count, 0);
    assert_eq!(reopened.status().unwrap().local_counter, 0);
}

#[test]
fn durable_operation_with_missing_counter_metadata_repairs_monotonically() {
    let dir = TempDir::new().unwrap();
    let (storage, recovery) = Storage::open(dir.path()).unwrap();
    let op = put(recovery.replica_id.clone(), 1, VersionVector::default(), "k", "v");
    storage.append_operations(&[op]).unwrap();
    drop(storage);

    let mut reopened = Replica::open(dir.path()).unwrap();
    assert_eq!(reopened.status().unwrap().local_counter, 1);
    let dot = reopened.put("k2", b"v2".to_vec()).unwrap();
    assert_eq!(dot.counter, 2);
}

#[test]
fn counter_metadata_ahead_of_operation_history_fails_closed() {
    let dir = TempDir::new().unwrap();
    let (storage, _) = Storage::open(dir.path()).unwrap();
    storage.append_counter(1).unwrap();
    drop(storage);
    let err = Replica::open(dir.path()).unwrap_err();
    assert!(err.to_string().contains("counter log"));
}

#[test]
fn missing_identity_with_nonempty_history_fails_closed() {
    let dir = TempDir::new().unwrap();
    let mut replica = Replica::open(dir.path()).unwrap();
    replica.put("k", b"v".to_vec()).unwrap();
    drop(replica);
    fs::remove_file(dir.path().join("identity.json")).unwrap();
    let err = Replica::open(dir.path()).unwrap_err();
    assert!(err.to_string().contains("identity is missing"));
}

#[test]
fn corrupted_snapshot_fails_closed() {
    let dir = TempDir::new().unwrap();
    let mut replica = Replica::open(dir.path()).unwrap();
    replica.put("k", b"v".to_vec()).unwrap();
    replica.snapshot().unwrap();
    drop(replica);

    let path = dir.path().join("snapshot.bin");
    let mut bytes = fs::read(&path).unwrap();
    let idx = bytes.len() / 2;
    bytes[idx] ^= 0x80;
    fs::write(&path, bytes).unwrap();
    let err = Replica::open(dir.path()).unwrap_err();
    assert!(err.to_string().contains("snapshot") || err.to_string().contains("serialization"));
}
