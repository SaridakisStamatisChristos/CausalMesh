use causalmesh::{
    sync_bidirectional, CausalContext, Error, OpKind, Operation, Replica, WireMessage,
    MAX_KEY_BYTES, MAX_VALUE_BYTES, WIRE_VERSION,
};
use tempfile::TempDir;
use uuid::Uuid;

fn fresh() -> (TempDir, Replica) {
    let dir = TempDir::new().unwrap();
    let replica = Replica::init(dir.path()).unwrap();
    (dir, replica)
}

#[test]
fn exact_summary_preserves_missing_hole() {
    let (_da, mut a) = fresh();
    let (_db, mut b) = fresh();
    a.put("k", b"v1".to_vec()).unwrap();
    a.put("k", b"v2".to_vec()).unwrap();
    let ops = a.operations();
    b.receive_batch(&[ops[1].clone()]).unwrap();
    assert!(!b.summary().contains(ops[0].id));
    let missing = a.export_missing(&b.summary());
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0].id, ops[0].id);
    b.receive_batch(&missing).unwrap();
    assert_eq!(a.logical_state(), b.logical_state());
}

#[test]
fn reverse_order_delivery_of_causal_chain_converges() {
    let (_da, mut a) = fresh();
    let (_db, mut b) = fresh();
    a.put("k", b"old".to_vec()).unwrap();
    a.put("k", b"new".to_vec()).unwrap();
    let mut ops = a.operations();
    ops.reverse();
    for op in ops {
        b.receive_batch(&[op]).unwrap();
    }
    assert_eq!(b.get("k"), vec![b"new".to_vec()]);
    assert_eq!(a.logical_state(), b.logical_state());
}

#[test]
fn partially_overlapping_batches_are_idempotent() {
    let (_da, mut a) = fresh();
    let (_db, mut b) = fresh();
    for i in 0..4 {
        a.put(format!("k{i}"), vec![i]).unwrap();
    }
    let ops = a.operations();
    assert_eq!(b.receive_batch(&ops[..3]).unwrap(), 3);
    assert_eq!(b.receive_batch(&ops[1..]).unwrap(), 1);
    assert_eq!(b.receive_batch(&ops).unwrap(), 0);
    assert_eq!(a.logical_state(), b.logical_state());
}

#[test]
fn chain_topology_converges_after_repeated_anti_entropy() {
    let (_da, mut a) = fresh();
    let (_db, mut b) = fresh();
    let (_dc, mut c) = fresh();
    a.put("a", b"1".to_vec()).unwrap();
    b.put("b", b"2".to_vec()).unwrap();
    c.put("c", b"3".to_vec()).unwrap();
    for _ in 0..3 {
        sync_bidirectional(&mut a, &mut b).unwrap();
        sync_bidirectional(&mut b, &mut c).unwrap();
    }
    assert_eq!(a.logical_state(), b.logical_state());
    assert_eq!(b.logical_state(), c.logical_state());
}

#[test]
fn star_topology_converges() {
    let (_dc, mut center) = fresh();
    let leaf_dirs: Vec<TempDir> = (0..4).map(|_| TempDir::new().unwrap()).collect();
    let mut leaves: Vec<Replica> = leaf_dirs
        .iter()
        .map(|d| Replica::init(d.path()).unwrap())
        .collect();
    center.put("center", b"c".to_vec()).unwrap();
    for (i, leaf) in leaves.iter_mut().enumerate() {
        leaf.put(format!("leaf:{i}"), vec![i as u8]).unwrap();
    }
    for _ in 0..2 {
        for leaf in &mut leaves {
            sync_bidirectional(&mut center, leaf).unwrap();
        }
    }
    for leaf in &leaves {
        assert_eq!(center.logical_state(), leaf.logical_state());
    }
}

#[test]
fn deterministic_pairwise_gossip_converges_five_replicas() {
    let dirs: Vec<TempDir> = (0..5).map(|_| TempDir::new().unwrap()).collect();
    let mut replicas: Vec<Replica> = dirs
        .iter()
        .map(|d| Replica::init(d.path()).unwrap())
        .collect();
    for (i, replica) in replicas.iter_mut().enumerate() {
        replica.put(format!("k{i}"), vec![i as u8]).unwrap();
    }
    let schedule = [
        (0, 1),
        (2, 3),
        (1, 4),
        (0, 2),
        (3, 4),
        (1, 3),
        (0, 4),
        (2, 4),
        (0, 3),
        (1, 2),
    ];
    for _ in 0..3 {
        for &(i, j) in &schedule {
            let (left, right) = replicas.split_at_mut(j.max(i));
            if i < j {
                sync_bidirectional(&mut left[i], &mut right[0]).unwrap();
            } else {
                sync_bidirectional(&mut right[0], &mut left[j]).unwrap();
            }
        }
    }
    let state = replicas[0].logical_state();
    for replica in &replicas {
        assert_eq!(state, replica.logical_state());
    }
}

#[test]
fn conflicting_cloned_replica_identity_fails_closed() {
    let seed = TempDir::new().unwrap();
    let _seed_replica = Replica::init(seed.path()).unwrap();
    let a_dir = TempDir::new().unwrap();
    let b_dir = TempDir::new().unwrap();
    for name in ["identity.json", "counter.log", "ops.log"] {
        std::fs::copy(seed.path().join(name), a_dir.path().join(name)).unwrap();
        std::fs::copy(seed.path().join(name), b_dir.path().join(name)).unwrap();
    }
    let mut a = Replica::open(a_dir.path()).unwrap();
    let mut b = Replica::open(b_dir.path()).unwrap();
    let a_id = a.put("k", b"a".to_vec()).unwrap();
    let b_id = b.put("k", b"b".to_vec()).unwrap();
    assert_eq!(a_id, b_id);
    assert!(matches!(
        sync_bidirectional(&mut a, &mut b),
        Err(Error::ReplicaIdentityCollision(_))
    ));
}

#[test]
fn malformed_batch_failure_survives_reopen_without_partial_commit() {
    let (dir, mut r) = fresh();
    let remote = Uuid::new_v4();
    let good = Operation {
        id: causalmesh::Dot::new(remote, 1).unwrap(),
        context: CausalContext::default(),
        key: "ok".into(),
        kind: OpKind::Put(b"v".to_vec()),
    };
    let bad = Operation {
        id: causalmesh::Dot::new(remote, 2).unwrap(),
        context: CausalContext::default(),
        key: String::new(),
        kind: OpKind::Put(b"bad".to_vec()),
    };
    assert!(r.receive_batch(&[good, bad]).is_err());
    drop(r);
    let reopened = Replica::open(dir.path()).unwrap();
    assert!(reopened.get("ok").is_empty());
    assert!(reopened.operations().is_empty());
}

#[test]
fn deterministic_wire_encoding_ignores_observation_order() {
    let r1 = Uuid::new_v4();
    let r2 = Uuid::new_v4();
    let dots = [
        causalmesh::Dot::new(r1, 2).unwrap(),
        causalmesh::Dot::new(r2, 1).unwrap(),
        causalmesh::Dot::new(r1, 1).unwrap(),
    ];
    let mut a = CausalContext::default();
    let mut b = CausalContext::default();
    for dot in dots {
        a.observe(dot).unwrap();
    }
    for dot in dots.into_iter().rev() {
        b.observe(dot).unwrap();
    }
    let peer = Uuid::new_v4();
    let ma = WireMessage::Summary {
        version: WIRE_VERSION,
        replica_id: peer,
        seen: a,
    };
    let mb = WireMessage::Summary {
        version: WIRE_VERSION,
        replica_id: peer,
        seen: b,
    };
    assert_eq!(ma.encode().unwrap(), mb.encode().unwrap());
}

#[test]
fn declared_key_and_value_boundaries_are_enforced() {
    let (_dir, mut r) = fresh();
    let max_key = "κ".repeat(MAX_KEY_BYTES / "κ".len());
    assert!(max_key.len() <= MAX_KEY_BYTES);
    r.put(max_key, Vec::new()).unwrap();
    assert!(r.put("x".repeat(MAX_KEY_BYTES + 1), b"v".to_vec()).is_err());
    assert!(r.put("large", vec![0; MAX_VALUE_BYTES + 1]).is_err());
}
