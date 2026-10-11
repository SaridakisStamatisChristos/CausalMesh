use std::collections::{BTreeMap, BTreeSet};

use causalmesh::{sync_bidirectional, OpKind, Operation, Replica};
use tempfile::TempDir;

fn reference_visible(ops: &[Operation]) -> BTreeMap<String, Vec<Vec<u8>>> {
    let keys: BTreeSet<&str> = ops.iter().map(|o| o.key.as_str()).collect();
    let mut out = BTreeMap::new();
    for key in keys {
        let same: Vec<&Operation> = ops.iter().filter(|o| o.key == key).collect();
        let maximal: Vec<&Operation> = same.iter().copied().filter(|candidate| {
            !same.iter().any(|other| other.id != candidate.id && other.context.contains(candidate.id))
        }).collect();
        let mut values: Vec<Vec<u8>> = maximal.into_iter().filter_map(|o| match &o.kind {
            OpKind::Put(v) => Some(v.clone()), OpKind::Delete => None,
        }).collect();
        values.sort();
        if !values.is_empty() { out.insert(key.to_string(), values); }
    }
    out
}

#[test]
fn production_state_matches_independent_reference_model() {
    let da = TempDir::new().unwrap();
    let db = TempDir::new().unwrap();
    let mut a = Replica::init(da.path()).unwrap();
    let mut b = Replica::init(db.path()).unwrap();
    a.put("k", b"a".to_vec()).unwrap();
    b.put("k", b"b".to_vec()).unwrap();
    b.put("x", b"x".to_vec()).unwrap();
    sync_bidirectional(&mut a, &mut b).unwrap();
    a.delete("x").unwrap();
    sync_bidirectional(&mut a, &mut b).unwrap();
    let mut expected = reference_visible(&a.operations());
    for values in expected.values_mut() { values.sort(); }
    let mut actual = a.logical_state();
    for values in actual.values_mut() { values.sort(); }
    assert_eq!(actual, expected);
}
