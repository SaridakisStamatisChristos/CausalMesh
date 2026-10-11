use causalmesh::{sync_bidirectional, Replica};
use tempfile::TempDir;

fn replica() -> (TempDir, Replica) {
    let dir = TempDir::new().unwrap();
    let replica = Replica::init(dir.path()).unwrap();
    (dir, replica)
}

#[test]
fn concurrent_same_key_writes_survive() {
    let (_da, mut a) = replica();
    let (_db, mut b) = replica();
    a.put("k", b"alice".to_vec()).unwrap();
    b.put("k", b"bob".to_vec()).unwrap();
    sync_bidirectional(&mut a, &mut b).unwrap();
    assert_eq!(a.logical_state(), b.logical_state());
    let mut values = a.get("k");
    values.sort();
    assert_eq!(values, vec![b"alice".to_vec(), b"bob".to_vec()]);
}

#[test]
fn resolving_write_dominates_observed_concurrency() {
    let (_da, mut a) = replica();
    let (_db, mut b) = replica();
    a.put("k", b"a".to_vec()).unwrap();
    b.put("k", b"b".to_vec()).unwrap();
    sync_bidirectional(&mut a, &mut b).unwrap();
    a.put("k", b"resolved".to_vec()).unwrap();
    sync_bidirectional(&mut a, &mut b).unwrap();
    assert_eq!(a.get("k"), vec![b"resolved".to_vec()]);
    assert_eq!(a.logical_state(), b.logical_state());
}

#[test]
fn offline_delete_does_not_erase_concurrent_write() {
    let (_da, mut a) = replica();
    let (_db, mut b) = replica();
    a.put("k", b"old".to_vec()).unwrap();
    sync_bidirectional(&mut a, &mut b).unwrap();
    a.delete("k").unwrap();
    b.put("k", b"concurrent".to_vec()).unwrap();
    sync_bidirectional(&mut a, &mut b).unwrap();
    assert_eq!(a.get("k"), vec![b"concurrent".to_vec()]);
    assert_eq!(a.logical_state(), b.logical_state());
}

#[test]
fn stale_peer_cannot_resurrect_deleted_value() {
    let (_da, mut a) = replica();
    let (_db, mut b) = replica();
    a.put("k", b"old".to_vec()).unwrap();
    sync_bidirectional(&mut a, &mut b).unwrap();
    a.delete("k").unwrap();
    sync_bidirectional(&mut a, &mut b).unwrap();
    assert!(a.get("k").is_empty());
    assert!(b.get("k").is_empty());
    let ancient = b.operations().into_iter().find(|op| matches!(op.kind, causalmesh::OpKind::Put(_))).unwrap();
    assert_eq!(a.receive_batch(&[ancient]).unwrap(), 0);
    assert!(a.get("k").is_empty());
}

#[test]
fn three_way_partition_converges_under_pairwise_gossip() {
    let (_da, mut a) = replica();
    let (_db, mut b) = replica();
    let (_dc, mut c) = replica();
    a.put("k", b"a".to_vec()).unwrap();
    b.put("k", b"b".to_vec()).unwrap();
    c.put("x", b"c".to_vec()).unwrap();
    sync_bidirectional(&mut a, &mut b).unwrap();
    sync_bidirectional(&mut b, &mut c).unwrap();
    sync_bidirectional(&mut a, &mut c).unwrap();
    sync_bidirectional(&mut a, &mut b).unwrap();
    assert_eq!(a.logical_state(), b.logical_state());
    assert_eq!(b.logical_state(), c.logical_state());
}

#[test]
fn repeated_sync_is_idempotent() {
    let (_da, mut a) = replica();
    let (_db, mut b) = replica();
    a.put("k", b"v".to_vec()).unwrap();
    let first = sync_bidirectional(&mut a, &mut b).unwrap();
    assert_eq!(first.a_to_b, 1);
    let count = a.operations().len() + b.operations().len();
    let second = sync_bidirectional(&mut a, &mut b).unwrap();
    assert_eq!(second.a_to_b + second.b_to_a, 0);
    assert_eq!(count, a.operations().len() + b.operations().len());
}

#[test]
fn new_replica_bootstrap_preserves_causality() {
    let (_da, mut a) = replica();
    a.put("k", b"v1".to_vec()).unwrap();
    a.put("k", b"v2".to_vec()).unwrap();
    let (_db, mut b) = replica();
    sync_bidirectional(&mut a, &mut b).unwrap();
    assert_eq!(b.get("k"), vec![b"v2".to_vec()]);
}
