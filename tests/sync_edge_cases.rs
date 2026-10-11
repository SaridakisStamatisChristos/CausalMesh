use causalmesh::{sync_bidirectional, Replica};
use tempfile::TempDir;

#[test]
fn identical_duplicates_inside_one_batch_are_coalesced() {
    let source_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();
    let mut source = Replica::init(source_dir.path()).unwrap();
    let mut target = Replica::init(target_dir.path()).unwrap();
    source.put("k", b"v".to_vec()).unwrap();
    let op = source.operations().remove(0);

    assert_eq!(
        target.receive_batch(&[op.clone(), op.clone(), op]).unwrap(),
        1
    );
    drop(target);

    let reopened = Replica::open(target_dir.path()).unwrap();
    assert_eq!(reopened.operations().len(), 1);
    assert_eq!(reopened.get("k"), vec![b"v".to_vec()]);
}

#[test]
fn sender_restart_after_transmit_before_ack_is_retry_safe() {
    let sender_dir = TempDir::new().unwrap();
    let receiver_dir = TempDir::new().unwrap();
    let mut sender = Replica::init(sender_dir.path()).unwrap();
    let mut receiver = Replica::init(receiver_dir.path()).unwrap();
    sender.put("k", b"v".to_vec()).unwrap();

    let batch = sender.export_missing(&receiver.summary());
    assert_eq!(receiver.receive_batch(&batch).unwrap(), 1);
    drop(sender);

    let mut sender = Replica::open(sender_dir.path()).unwrap();
    let report = sync_bidirectional(&mut sender, &mut receiver).unwrap();
    assert_eq!(report.a_to_b + report.b_to_a, 0);
    assert_eq!(sender.logical_state(), receiver.logical_state());
}

#[test]
fn receiver_restart_mid_transfer_preserves_partial_progress() {
    let sender_dir = TempDir::new().unwrap();
    let receiver_dir = TempDir::new().unwrap();
    let mut sender = Replica::init(sender_dir.path()).unwrap();
    let mut receiver = Replica::init(receiver_dir.path()).unwrap();
    for i in 0..6 {
        sender.put(format!("k:{i}"), vec![i]).unwrap();
    }
    let operations = sender.operations();

    assert_eq!(receiver.receive_batch(&operations[..3]).unwrap(), 3);
    drop(receiver);
    let mut receiver = Replica::open(receiver_dir.path()).unwrap();
    assert_eq!(receiver.receive_batch(&operations[3..]).unwrap(), 3);
    assert_eq!(sender.logical_state(), receiver.logical_state());
}

#[test]
fn seeded_random_pairwise_gossip_then_eventual_delivery_converges() {
    let dirs: Vec<TempDir> = (0..6).map(|_| TempDir::new().unwrap()).collect();
    let mut replicas: Vec<Replica> = dirs
        .iter()
        .map(|dir| Replica::init(dir.path()).unwrap())
        .collect();
    for (index, replica) in replicas.iter_mut().enumerate() {
        replica
            .put(format!("replica:{index}"), vec![index as u8])
            .unwrap();
    }

    let mut state = 0x9e37_79b9_7f4a_7c15u64;
    for _ in 0..128 {
        state ^= state << 7;
        state ^= state >> 9;
        state ^= state << 8;
        let a = (state as usize) % replicas.len();
        state = state.rotate_left(17) ^ 0xa076_1d64_78bd_642f;
        let mut b = (state as usize) % replicas.len();
        if a == b {
            b = (b + 1) % replicas.len();
        }
        sync_pair(&mut replicas, a, b);
    }

    // Explicit eventual-delivery closure: every pair synchronizes after the
    // randomized partition/reconnect schedule.
    for a in 0..replicas.len() {
        for b in (a + 1)..replicas.len() {
            sync_pair(&mut replicas, a, b);
        }
    }

    let expected = replicas[0].logical_state();
    for replica in &replicas {
        assert_eq!(replica.logical_state(), expected);
    }
}

fn sync_pair(replicas: &mut [Replica], a: usize, b: usize) {
    assert_ne!(a, b);
    let (low, high) = if a < b { (a, b) } else { (b, a) };
    let (left, right) = replicas.split_at_mut(high);
    sync_bidirectional(&mut left[low], &mut right[0]).unwrap();
}
