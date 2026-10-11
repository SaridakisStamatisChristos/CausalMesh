use causalmesh::{sync_pair, Replica};
use tempfile::TempDir;

#[test]
fn two_replica_concurrent_write_then_resolution_converges() {
    let da = TempDir::new().unwrap();
    let db = TempDir::new().unwrap();
    let mut a = Replica::open(da.path()).unwrap();
    let mut b = Replica::open(db.path()).unwrap();

    a.put("k", b"alice".to_vec()).unwrap();
    b.put("k", b"bob".to_vec()).unwrap();
    let first = sync_pair(&mut a, &mut b).unwrap();
    assert!(first.converged);
    assert_eq!(a.get("k").unwrap().len(), 2);

    a.put("k", b"resolved".to_vec()).unwrap();
    let second = sync_pair(&mut a, &mut b).unwrap();
    assert!(second.converged);
    assert_eq!(b.get("k").unwrap(), vec![b"resolved".to_vec()]);
}

#[test]
fn offline_delete_prevents_stale_value_resurrection() {
    let da = TempDir::new().unwrap();
    let db = TempDir::new().unwrap();
    let mut a = Replica::open(da.path()).unwrap();
    let mut b = Replica::open(db.path()).unwrap();

    a.put("k", b"old".to_vec()).unwrap();
    sync_pair(&mut a, &mut b).unwrap();
    a.delete("k").unwrap();

    drop(b);
    let mut stale_b = Replica::open(db.path()).unwrap();
    let report = sync_pair(&mut a, &mut stale_b).unwrap();
    assert!(report.converged);
    assert!(stale_b.get("k").unwrap().is_empty());
}

#[test]
fn three_way_partition_and_changing_reconnect_order_converges() {
    let da = TempDir::new().unwrap();
    let db = TempDir::new().unwrap();
    let dc = TempDir::new().unwrap();
    let mut a = Replica::open(da.path()).unwrap();
    let mut b = Replica::open(db.path()).unwrap();
    let mut c = Replica::open(dc.path()).unwrap();

    a.put("shared", b"a".to_vec()).unwrap();
    b.put("shared", b"b".to_vec()).unwrap();
    c.put("other", b"c".to_vec()).unwrap();

    sync_pair(&mut a, &mut b).unwrap();
    sync_pair(&mut b, &mut c).unwrap();
    sync_pair(&mut a, &mut c).unwrap();

    assert_eq!(a.logical_state().unwrap(), b.logical_state().unwrap());
    assert_eq!(b.logical_state().unwrap(), c.logical_state().unwrap());
}

#[test]
fn repeated_sync_is_idempotent_and_creates_no_new_operations() {
    let da = TempDir::new().unwrap();
    let db = TempDir::new().unwrap();
    let mut a = Replica::open(da.path()).unwrap();
    let mut b = Replica::open(db.path()).unwrap();
    a.put("k", b"v".to_vec()).unwrap();
    sync_pair(&mut a, &mut b).unwrap();
    let ca = a.status().unwrap().operation_count;
    let cb = b.status().unwrap().operation_count;
    let report = sync_pair(&mut a, &mut b).unwrap();
    assert_eq!(report.sent_a_to_b, 0);
    assert_eq!(report.sent_b_to_a, 0);
    assert_eq!(a.status().unwrap().operation_count, ca);
    assert_eq!(b.status().unwrap().operation_count, cb);
}

#[test]
fn receive_persist_then_lost_ack_can_be_retried_safely() {
    let da = TempDir::new().unwrap();
    let db = TempDir::new().unwrap();
    let mut a = Replica::open(da.path()).unwrap();
    let mut b = Replica::open(db.path()).unwrap();
    a.put("k", b"v".to_vec()).unwrap();

    let batch = a.operations_missing_from(&b.summary().unwrap()).unwrap();
    assert_eq!(b.receive_operations(&batch).unwrap(), 1);
    drop(b); // model receiver crash after persistence and before acknowledgment

    let mut recovered_b = Replica::open(db.path()).unwrap();
    assert_eq!(recovered_b.receive_operations(&batch).unwrap(), 0);
    assert_eq!(recovered_b.get("k").unwrap(), vec![b"v".to_vec()]);
}
