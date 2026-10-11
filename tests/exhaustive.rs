use causalmesh::Replica;
use tempfile::TempDir;

fn permutations<T: Clone>(items: &[T]) -> Vec<Vec<T>> {
    if items.is_empty() { return vec![Vec::new()]; }
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
fn all_delivery_orders_of_three_concurrent_operations_converge() {
    let source_dirs: Vec<TempDir> = (0..3).map(|_| TempDir::new().unwrap()).collect();
    let mut sources: Vec<Replica> = source_dirs.iter().map(|d| Replica::init(d.path()).unwrap()).collect();
    sources[0].put("k", b"a".to_vec()).unwrap();
    sources[1].put("k", b"b".to_vec()).unwrap();
    sources[2].delete("k").unwrap();
    let ops = sources.iter().flat_map(|r| r.operations()).collect::<Vec<_>>();
    let mut expected = None;
    for order in permutations(&ops) {
        let dir = TempDir::new().unwrap();
        let mut r = Replica::init(dir.path()).unwrap();
        for op in order { r.receive_batch(&[op]).unwrap(); }
        match &expected { Some(state) => assert_eq!(&r.logical_state(), state), None => expected = Some(r.logical_state()) }
    }
}
