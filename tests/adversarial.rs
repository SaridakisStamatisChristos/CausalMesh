use causalmesh::{CausalContext, Dot, Error, OpKind, Operation, Replica, WireMessage, WIRE_VERSION};
use tempfile::TempDir;
use uuid::Uuid;

#[test]
fn duplicate_delivery_is_safe() {
    let a_dir = TempDir::new().unwrap();
    let b_dir = TempDir::new().unwrap();
    let mut a = Replica::init(a_dir.path()).unwrap();
    let mut b = Replica::init(b_dir.path()).unwrap();
    a.put("k", b"v".to_vec()).unwrap();
    let ops = a.operations();
    assert_eq!(b.receive_batch(&ops).unwrap(), 1);
    assert_eq!(b.receive_batch(&ops).unwrap(), 0);
    assert_eq!(b.get("k"), vec![b"v".to_vec()]);
}

#[test]
fn conflicting_duplicate_id_fails_before_mutation() {
    let a_dir = TempDir::new().unwrap();
    let b_dir = TempDir::new().unwrap();
    let mut a = Replica::init(a_dir.path()).unwrap();
    let mut b = Replica::init(b_dir.path()).unwrap();
    a.put("k", b"v".to_vec()).unwrap();
    let mut op = a.operations().remove(0);
    b.receive_batch(&[op.clone()]).unwrap();
    let before = b.logical_state();
    op.kind = OpKind::Put(b"evil".to_vec());
    assert!(matches!(b.receive_batch(&[op]), Err(Error::ConflictingOperation(_))));
    assert_eq!(before, b.logical_state());
}

#[test]
fn malformed_batch_does_not_partially_commit() {
    let dir = TempDir::new().unwrap();
    let mut r = Replica::init(dir.path()).unwrap();
    let remote = Uuid::new_v4();
    let good = Operation { id: Dot::new(remote, 1).unwrap(), context: CausalContext::default(), key: "ok".into(), kind: OpKind::Put(b"v".to_vec()) };
    let bad = Operation { id: Dot::new(remote, 2).unwrap(), context: CausalContext::default(), key: String::new(), kind: OpKind::Put(b"bad".to_vec()) };
    assert!(r.receive_batch(&[good, bad]).is_err());
    assert!(r.get("ok").is_empty());
    assert!(r.operations().is_empty());
}

#[test]
fn unsupported_wire_version_fails_closed() {
    let msg = format!("{{\"message\":\"hello\",\"version\":{},\"replica_id\":\"{}\"}}", WIRE_VERSION + 1, Uuid::new_v4());
    assert!(WireMessage::decode(msg.as_bytes()).is_err());
}

#[test]
fn oversized_wire_payload_is_rejected_before_decode() {
    let bytes = vec![b'x'; causalmesh::MAX_MESSAGE_BYTES + 1];
    assert!(matches!(WireMessage::decode(&bytes), Err(Error::Bounds(_))));
}

#[test]
fn empty_key_is_rejected() {
    let dir = TempDir::new().unwrap();
    let mut r = Replica::init(dir.path()).unwrap();
    assert!(r.put("", b"v".to_vec()).is_err());
}
