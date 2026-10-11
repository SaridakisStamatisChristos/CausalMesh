mod common;

use causalmesh::crdt::CrdtState;
use causalmesh::protocol::{decode_wire, CounterRange, Summary, WireEnvelope, WireMessage};
use causalmesh::{sync_pair, Dot, Error, Operation, OperationKind, Replica, VersionVector};
use common::{put, rid, vv};
use std::collections::BTreeMap;
use std::fs;
use tempfile::TempDir;

#[test]
fn conflicting_duplicate_operation_id_fails_closed() {
    let a = rid(1);
    let mut state = CrdtState::default();
    let first = put(a.clone(), 1, vv(&[]), "k", "one");
    let second = put(a, 1, vv(&[]), "k", "two");
    state.apply(first).unwrap();
    assert!(matches!(
        state.apply(second).unwrap_err(),
        Error::DuplicateOperationConflict(_)
    ));
}

#[test]
fn invalid_batch_does_not_partially_commit() {
    let da = TempDir::new().unwrap();
    let db = TempDir::new().unwrap();
    let mut a = Replica::open(da.path()).unwrap();
    let mut b = Replica::open(db.path()).unwrap();
    a.put("k", b"good".to_vec()).unwrap();
    let mut batch = a.operations().unwrap();
    batch.push(Operation {
        id: Dot {
            replica: rid(99),
            counter: 2,
        },
        context: VersionVector::default(),
        kind: OperationKind::Put {
            key: "bad".to_string(),
            value: b"bad".to_vec(),
        },
    });
    assert!(b.receive_operations(&batch).is_err());
    assert_eq!(b.status().unwrap().operation_count, 0);
    drop(b);
    assert_eq!(Replica::open(db.path()).unwrap().status().unwrap().operation_count, 0);
}

#[test]
fn unsupported_wire_version_is_rejected() {
    let bytes = serde_json::to_vec(&WireEnvelope {
        version: 99,
        message: WireMessage::Hello { replica_id: rid(1) },
    })
    .unwrap();
    assert!(matches!(
        decode_wire(&bytes).unwrap_err(),
        Error::UnsupportedVersion { .. }
    ));
}

#[test]
fn excessive_summary_ranges_are_rejected() {
    let mut ranges = Vec::new();
    for n in 0..16_385u64 {
        let c = 1 + n * 2;
        ranges.push(CounterRange { start: c, end: c });
    }
    let mut map = BTreeMap::new();
    map.insert(rid(2), ranges);
    let summary = Summary {
        replica_id: rid(1),
        ranges: map,
    };
    assert!(matches!(summary.validate().unwrap_err(), Error::Bounds(_)));
}

#[test]
fn cloned_replica_identity_is_detected_before_sync_mutation() {
    let original = TempDir::new().unwrap();
    let clone = TempDir::new().unwrap();
    let mut a = Replica::open(original.path()).unwrap();
    a.put("seed", b"x".to_vec()).unwrap();
    drop(a);

    for name in ["identity.json", "operations.log", "counter.log"] {
        fs::copy(original.path().join(name), clone.path().join(name)).unwrap();
    }
    let mut a = Replica::open(original.path()).unwrap();
    let mut b = Replica::open(clone.path()).unwrap();
    a.put("k", b"from-a".to_vec()).unwrap();
    b.put("k", b"from-b".to_vec()).unwrap();
    assert!(matches!(sync_pair(&mut a, &mut b), Err(Error::CloneIdentity)));
}

#[test]
fn origin_counter_transition_must_be_exact() {
    let op = Operation {
        id: Dot {
            replica: rid(1),
            counter: 3,
        },
        context: vv(&[(rid(1), 1)]),
        kind: OperationKind::Delete {
            key: "k".to_string(),
        },
    };
    assert!(matches!(op.validate().unwrap_err(), Error::InvalidOperation(_)));
}
