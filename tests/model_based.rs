use std::collections::{BTreeMap, BTreeSet};

use causalmesh::{sync_bidirectional, OpKind, Operation, Replica};
use proptest::prelude::*;
use tempfile::TempDir;

fn reference_visible(ops: &[Operation]) -> BTreeMap<String, Vec<Vec<u8>>> {
    let keys: BTreeSet<&str> = ops.iter().map(|op| op.key.as_str()).collect();
    let mut out = BTreeMap::new();
    for key in keys {
        let same_key: Vec<&Operation> = ops.iter().filter(|op| op.key == key).collect();
        let maximal: Vec<&Operation> = same_key
            .iter()
            .copied()
            .filter(|candidate| {
                !same_key
                    .iter()
                    .any(|other| other.id != candidate.id && other.context.contains(candidate.id))
            })
            .collect();
        let mut values: Vec<Vec<u8>> = maximal
            .into_iter()
            .filter_map(|op| match &op.kind {
                OpKind::Put(value) => Some(value.clone()),
                OpKind::Delete => None,
            })
            .collect();
        values.sort();
        if !values.is_empty() {
            out.insert(key.to_string(), values);
        }
    }
    out
}

fn sync_pair(replicas: &mut [Option<Replica>], a: usize, b: usize) {
    if a == b {
        return;
    }
    let (low, high) = if a < b { (a, b) } else { (b, a) };
    let (left, right) = replicas.split_at_mut(high);
    let low_replica = left[low].as_mut().unwrap();
    let high_replica = right[0].as_mut().unwrap();
    sync_bidirectional(low_replica, high_replica).unwrap();
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn generated_multi_replica_histories_converge(
        actions in prop::collection::vec(
            (0u8..10, 0u8..3, 0u8..3, 0u8..3, any::<u8>()),
            1..40,
        )
    ) {
        let dirs: Vec<TempDir> = (0..3).map(|_| TempDir::new().unwrap()).collect();
        let mut replicas: Vec<Option<Replica>> = dirs
            .iter()
            .map(|dir| Some(Replica::init(dir.path()).unwrap()))
            .collect();

        for (kind, a_raw, b_raw, key_raw, value) in actions {
            let a = a_raw as usize % replicas.len();
            let b = b_raw as usize % replicas.len();
            let key = format!("k:{}", key_raw % 3);
            match kind {
                0 | 1 => {
                    replicas[a]
                        .as_mut()
                        .unwrap()
                        .put(key, vec![value])
                        .unwrap();
                }
                2 => {
                    replicas[a].as_mut().unwrap().delete(key).unwrap();
                }
                3 => sync_pair(&mut replicas, a, b),
                4 if a != b => {
                    let target_summary = replicas[b].as_ref().unwrap().summary();
                    let mut missing = replicas[a]
                        .as_ref()
                        .unwrap()
                        .export_missing(&target_summary);
                    missing.reverse();
                    replicas[b].as_mut().unwrap().receive_batch(&missing).unwrap();
                }
                5 if a != b => {
                    let target_summary = replicas[b].as_ref().unwrap().summary();
                    if let Some(op) = replicas[a]
                        .as_ref()
                        .unwrap()
                        .export_missing(&target_summary)
                        .into_iter()
                        .next()
                    {
                        replicas[b]
                            .as_mut()
                            .unwrap()
                            .receive_batch(&[op.clone(), op])
                            .unwrap();
                    }
                }
                6 => {
                    let old = replicas[a].take().unwrap();
                    drop(old);
                    replicas[a] = Some(Replica::open(dirs[a].path()).unwrap());
                }
                7 => {
                    replicas[a].as_ref().unwrap().snapshot().unwrap();
                }
                8 => {
                    // Deliberate partition/delay step: no delivery occurs.
                }
                _ => sync_pair(&mut replicas, a, b),
            }
        }

        // Eventual-delivery closure after arbitrary delay/reorder/restart history.
        for _ in 0..2 {
            for a in 0..replicas.len() {
                for b in (a + 1)..replicas.len() {
                    sync_pair(&mut replicas, a, b);
                }
            }
        }

        let expected_ops = replicas[0].as_ref().unwrap().operations();
        let mut reference = reference_visible(&expected_ops);
        for values in reference.values_mut() {
            values.sort();
        }

        for replica in &replicas {
            let replica = replica.as_ref().unwrap();
            let mut actual = replica.logical_state();
            for values in actual.values_mut() {
                values.sort();
            }
            prop_assert_eq!(&actual, &reference);
            prop_assert_eq!(replica.operations().len(), expected_ops.len());
        }
    }
}
