mod common;

use causalmesh::crdt::CrdtState;
use causalmesh::{Operation, OperationKind};
use common::{delete, put, rid, vv};
use std::collections::BTreeMap;

fn independent_frontier(
    ops: &[Operation],
) -> BTreeMap<String, Vec<(causalmesh::Dot, Option<Vec<u8>>)>> {
    let mut by_key: BTreeMap<String, Vec<&Operation>> = BTreeMap::new();
    for op in ops {
        by_key
            .entry(op.kind.key().to_string())
            .or_default()
            .push(op);
    }
    by_key
        .into_iter()
        .map(|(key, candidates)| {
            let mut frontier = Vec::new();
            for candidate in &candidates {
                let dominated = candidates.iter().any(|other| {
                    other.id != candidate.id && other.context.dominates_dot(&candidate.id)
                });
                if !dominated {
                    let value = match &candidate.kind {
                        OperationKind::Put { value, .. } => Some(value.clone()),
                        OperationKind::Delete { .. } => None,
                    };
                    frontier.push((candidate.id.clone(), value));
                }
            }
            frontier.sort_by(|a, b| a.0.cmp(&b.0));
            (key, frontier)
        })
        .collect()
}

#[test]
fn production_incremental_semantics_match_independent_frontier_oracle() {
    let a = rid(1);
    let b = rid(2);
    let c = rid(3);
    let ops = vec![
        put(a.clone(), 1, vv(&[]), "k", "a1"),
        put(b.clone(), 1, vv(&[]), "k", "b1"),
        delete(c.clone(), 1, vv(&[]), "k"),
        put(
            a.clone(),
            2,
            vv(&[(a.clone(), 1), (b.clone(), 1)]),
            "k",
            "a2",
        ),
        put(c, 2, vv(&[(rid(3), 1)]), "x", "cx"),
    ];

    let expected = independent_frontier(&ops);
    let mut production = CrdtState::default();
    for op in ops.iter().rev() {
        production.apply(op.clone()).unwrap();
    }

    for (key, expected_entries) in expected {
        let actual: Vec<_> = production
            .entries(&key)
            .into_iter()
            .map(|entry| (entry.dot, entry.value))
            .collect();
        assert_eq!(actual, expected_entries, "mismatch at key {key}");
    }
}
