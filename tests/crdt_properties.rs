mod common;

use causalmesh::crdt::CrdtState;
use common::{delete, put, rid, vv};

#[test]
fn concurrent_writes_are_preserved_and_resolved_by_causal_write() {
    let a = rid(1);
    let b = rid(2);
    let a1 = put(a.clone(), 1, vv(&[]), "k", "alice");
    let b1 = put(b.clone(), 1, vv(&[]), "k", "bob");

    let mut state = CrdtState::default();
    state.apply(a1.clone()).unwrap();
    state.apply(b1.clone()).unwrap();
    assert_eq!(state.get("k").len(), 2);

    let a2 = put(
        a.clone(),
        2,
        vv(&[(a.clone(), 1), (b.clone(), 1)]),
        "k",
        "resolved",
    );
    state.apply(a2).unwrap();
    assert_eq!(state.get("k"), vec![b"resolved".to_vec()]);
}

#[test]
fn causally_old_value_cannot_resurrect_when_delivered_after_delete() {
    let a = rid(1);
    let a1 = put(a.clone(), 1, vv(&[]), "k", "old");
    let a2 = delete(a.clone(), 2, vv(&[(a.clone(), 1)]), "k");

    let mut state = CrdtState::default();
    state.apply(a2).unwrap();
    state.apply(a1).unwrap();
    assert!(state.get("k").is_empty());
    assert_eq!(state.entries("k").len(), 1);
    assert!(state.entries("k")[0].value.is_none());
}

#[test]
fn concurrent_delete_does_not_silently_erase_concurrent_write() {
    let a = rid(1);
    let b = rid(2);
    let mut state = CrdtState::default();
    state.apply(delete(a, 1, vv(&[]), "k")).unwrap();
    state.apply(put(b, 1, vv(&[]), "k", "survives")).unwrap();
    assert_eq!(state.entries("k").len(), 2);
    assert_eq!(state.get("k"), vec![b"survives".to_vec()]);
}

#[test]
fn merge_is_idempotent_commutative_and_associative() {
    let a = rid(1);
    let b = rid(2);
    let c = rid(3);

    let mut sa = CrdtState::default();
    sa.apply(put(a, 1, vv(&[]), "k", "a")).unwrap();
    let mut sb = CrdtState::default();
    sb.apply(put(b, 1, vv(&[]), "k", "b")).unwrap();
    let mut sc = CrdtState::default();
    sc.apply(put(c, 1, vv(&[]), "x", "c")).unwrap();

    assert_eq!(sa.merge(&sa).unwrap().export(), sa.export());
    assert_eq!(sa.merge(&sb).unwrap().export(), sb.merge(&sa).unwrap().export());

    let left = sa.merge(&sb).unwrap().merge(&sc).unwrap();
    let right = sa.merge(&sb.merge(&sc).unwrap()).unwrap();
    assert_eq!(left.export(), right.export());
}

#[test]
fn equivalent_histories_have_identical_canonical_bytes() {
    let a = rid(1);
    let b = rid(2);
    let ops = vec![
        put(a, 1, vv(&[]), "z", "1"),
        put(b, 1, vv(&[]), "a", "2"),
    ];
    let mut left = CrdtState::default();
    for op in &ops {
        left.apply(op.clone()).unwrap();
    }
    let mut right = CrdtState::default();
    for op in ops.iter().rev() {
        right.apply(op.clone()).unwrap();
    }
    assert_eq!(left.export(), right.export());
    assert_eq!(
        left.canonical_state_bytes().unwrap(),
        right.canonical_state_bytes().unwrap()
    );
}
