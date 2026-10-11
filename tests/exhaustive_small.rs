use causalmesh::Replica;
use tempfile::TempDir;

fn permutations<T: Clone>(items: &[T]) -> Vec<Vec<T>> {
    if items.is_empty() {
        return vec![Vec::new()];
    }
    let mut out = Vec::new();
    for i in 0..items.len() {
        let mut rest = items.to_vec();
        let head = rest.remove(i);
        for mut tail in permutations(&rest) {
            let mut p = vec![head.clone()];
            p.append(&mut tail);
            out.push(p);
        }
    }
    out
}

#[test]
fn two_replicas_two_keys_four_operations_all_delivery_orders_converge() {
    let da = TempDir::new().unwrap();
    let db = TempDir::new().unwrap();
    let mut a = Replica::init(da.path()).unwrap();
    let mut b = Replica::init(db.path()).unwrap();
    a.put("k0", b"a".to_vec()).unwrap();
    a.put("k1", b"b".to_vec()).unwrap();
    b.put("k0", b"b".to_vec()).unwrap();
    b.delete("k1").unwrap();
    let mut ops = a.operations();
    ops.extend(b.operations());
    assert_eq!(ops.len(), 4);

    let mut expected = None;
    for order in permutations(&ops) {
        let dir = TempDir::new().unwrap();
        let mut replica = Replica::init(dir.path()).unwrap();
        for op in order {
            replica.receive_batch(&[op]).unwrap();
        }
        match &expected {
            Some(state) => assert_eq!(&replica.logical_state(), state),
            None => expected = Some(replica.logical_state()),
        }
    }
}
