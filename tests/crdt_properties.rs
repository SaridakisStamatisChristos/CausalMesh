use causalmesh::Replica;
use proptest::prelude::*;
use tempfile::TempDir;

proptest! {
    #[test]
    fn delivery_permutation_and_duplication_do_not_change_state(seed in any::<u64>()) {
        let da = TempDir::new().unwrap();
        let db = TempDir::new().unwrap();
        let mut a = Replica::init(da.path()).unwrap();
        let mut b = Replica::init(db.path()).unwrap();
        a.put("k", format!("a-{seed}").into_bytes()).unwrap();
        b.put("k", format!("b-{seed}").into_bytes()).unwrap();
        a.put("x", b"x".to_vec()).unwrap();
        b.delete("y").unwrap();
        let mut ops = a.operations();
        ops.extend(b.operations());
        let mut reversed = ops.clone();
        reversed.reverse();
        let d1 = TempDir::new().unwrap();
        let d2 = TempDir::new().unwrap();
        let mut r1 = Replica::init(d1.path()).unwrap();
        let mut r2 = Replica::init(d2.path()).unwrap();
        r1.receive_batch(&ops).unwrap();
        r1.receive_batch(&ops).unwrap();
        r2.receive_batch(&reversed).unwrap();
        prop_assert_eq!(r1.logical_state(), r2.logical_state());
    }
}
