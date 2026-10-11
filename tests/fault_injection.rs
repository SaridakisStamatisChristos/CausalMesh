use causalmesh::{Error, FaultPoint, Replica};
use tempfile::TempDir;

#[test]
fn crash_after_counter_reservation_never_reuses_dot() {
    let dir = TempDir::new().unwrap();
    let mut replica = Replica::init(dir.path()).unwrap();
    replica.set_fault_point(Some(FaultPoint::AfterCounterReservation));
    assert!(matches!(
        replica.put("k", b"lost".to_vec()),
        Err(Error::InjectedFault(_))
    ));
    drop(replica);

    let mut recovered = Replica::open(dir.path()).unwrap();
    assert!(recovered.get("k").is_empty());
    assert_eq!(recovered.status().unwrap().reserved_counter, 1);
    let id = recovered.put("k", b"committed".to_vec()).unwrap();
    assert_eq!(id.counter, 2);
}

#[test]
fn crash_after_local_persistence_recovers_unknown_outcome() {
    let dir = TempDir::new().unwrap();
    let mut replica = Replica::init(dir.path()).unwrap();
    replica.set_fault_point(Some(FaultPoint::AfterLocalOperationPersistence));
    assert!(matches!(
        replica.put("k", b"v".to_vec()),
        Err(Error::InjectedFault(_))
    ));
    drop(replica);

    let recovered = Replica::open(dir.path()).unwrap();
    assert_eq!(recovered.get("k"), vec![b"v".to_vec()]);
    assert_eq!(recovered.status().unwrap().reserved_counter, 1);
}

#[test]
fn receiver_crash_after_persist_before_publication_is_retry_safe() {
    let sender_dir = TempDir::new().unwrap();
    let receiver_dir = TempDir::new().unwrap();
    let mut sender = Replica::init(sender_dir.path()).unwrap();
    let mut receiver = Replica::init(receiver_dir.path()).unwrap();
    sender.put("k", b"v".to_vec()).unwrap();
    let batch = sender.operations();

    receiver.set_fault_point(Some(FaultPoint::AfterReceivedBatchPersistence));
    assert!(matches!(
        receiver.receive_batch(&batch),
        Err(Error::InjectedFault(_))
    ));
    drop(receiver);

    let mut recovered = Replica::open(receiver_dir.path()).unwrap();
    assert_eq!(recovered.get("k"), vec![b"v".to_vec()]);
    assert_eq!(recovered.receive_batch(&batch).unwrap(), 0);
    assert_eq!(recovered.logical_state(), sender.logical_state());
}
