mod common;

use causalmesh::crdt::CrdtState;
use causalmesh::Operation;
use common::{delete, put, rid, vv};

fn permutations<T: Clone>(items: &[T]) -> Vec<Vec<T>> {
    if items.is_empty() {
        return vec![Vec::new()];
    }
    let mut out = Vec::new();
    for i in 0..items.len() {
        let mut rest = items.to_vec();
        let head = rest.remove(i);
        for mut tail in permutations(&rest) {
            let mut perm = vec![head.clone()];
            perm.append(&mut tail);
            out.push(perm);
        }
    }
    out
}

#[test]
fn all_delivery_permutations_for_four_operation_tiny_universe_converge() {
    let a = rid(1);
    let b = rid(2);
    let ops: Vec<Operation> = vec![
        put(a.clone(), 1, vv(&[]), "k0", "v0"),
        put(b.clone(), 1, vv(&[]), "k0", "v1"),
        put(a.clone(), 2, vv(&[(a.clone(), 1)]), "k1", "v1"),
        delete(b.clone(), 2, vv(&[(b.clone(), 1)]), "k1"),
    ];

    let mut expected = None;
    let mut count = 0usize;
    for schedule in permutations(&ops) {
        let mut state = CrdtState::default();
        for op in schedule {
            state.apply(op).unwrap();
        }
        let exported = state.export();
        if let Some(reference) = &expected {
            assert_eq!(&exported, reference);
        } else {
            expected = Some(exported);
        }
        count += 1;
    }
    assert_eq!(count, 24);
}
