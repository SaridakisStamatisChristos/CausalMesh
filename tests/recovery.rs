use std::fs::OpenOptions;
use std::io::Write;

use causalmesh::{storage::read_snapshot, Replica};
use tempfile::TempDir;

#[test]
fn committed_local_mutation_survives_reopen() {
    let dir = TempDir::new().unwrap();
    let id;
    {
        let mut r = Replica::init(dir.path()).unwrap();
        id = r.put("k", b"v".to_vec()).unwrap();
    }
    let r = Replica::open(dir.path()).unwrap();
    assert_eq!(r.get("k"), vec![b"v".to_vec()]);
    assert!(r.summary().contains(id));
}

#[test]
fn counter_never_moves_backward_across_restart() {
    let dir = TempDir::new().unwrap();
    let first = { let mut r = Replica::init(dir.path()).unwrap(); r.put("a", b"1".to_vec()).unwrap() };
    let second = { let mut r = Replica::open(dir.path()).unwrap(); r.put("b", b"2".to_vec()).unwrap() };
    assert_eq!(first.replica, second.replica);
    assert!(second.counter > first.counter);
}

#[test]
fn recognizable_torn_final_log_record_is_truncated() {
    let dir = TempDir::new().unwrap();
    { let mut r = Replica::init(dir.path()).unwrap(); r.put("k", b"v".to_vec()).unwrap(); }
    let path = dir.path().join("ops.log");
    let good_len = std::fs::metadata(&path).unwrap().len();
    let mut file = OpenOptions::new().append(true).open(&path).unwrap();
    file.write_all(b"CMOP\x01").unwrap();
    file.sync_all().unwrap();
    drop(file);
    let r = Replica::open(dir.path()).unwrap();
    assert_eq!(r.get("k"), vec![b"v".to_vec()]);
    assert_eq!(std::fs::metadata(path).unwrap().len(), good_len);
}

#[test]
fn complete_corrupt_record_fails_closed() {
    let dir = TempDir::new().unwrap();
    { let mut r = Replica::init(dir.path()).unwrap(); r.put("k", b"v".to_vec()).unwrap(); }
    let path = dir.path().join("ops.log");
    let mut bytes = std::fs::read(&path).unwrap();
    let idx = bytes.len() - 1;
    bytes[idx] ^= 0xFF;
    std::fs::write(&path, bytes).unwrap();
    assert!(Replica::open(dir.path()).is_err());
}

#[test]
fn snapshot_round_trip_preserves_operations() {
    let dir = TempDir::new().unwrap();
    let mut r = Replica::init(dir.path()).unwrap();
    r.put("k", b"v".to_vec()).unwrap();
    r.delete("gone").unwrap();
    let path = r.snapshot().unwrap();
    let snap = read_snapshot(path).unwrap();
    assert_eq!(snap.operations, r.operations());
}
