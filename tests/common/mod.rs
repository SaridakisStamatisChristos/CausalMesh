#![allow(dead_code)]

use causalmesh::{Dot, Operation, OperationKind, ReplicaId, VersionVector};

pub fn rid(n: u128) -> ReplicaId {
    let s = format!("{n:032x}");
    let uuid = format!(
        "{}-{}-{}-{}-{}",
        &s[0..8],
        &s[8..12],
        &s[12..16],
        &s[16..20],
        &s[20..32]
    );
    ReplicaId::parse(uuid).unwrap()
}

pub fn vv(entries: &[(ReplicaId, u64)]) -> VersionVector {
    let mut vv = VersionVector::default();
    for (replica, counter) in entries {
        vv.0.insert(replica.clone(), *counter);
    }
    vv
}

pub fn put(
    replica: ReplicaId,
    counter: u64,
    context: VersionVector,
    key: &str,
    value: &str,
) -> Operation {
    Operation {
        id: Dot { replica, counter },
        context,
        kind: OperationKind::Put {
            key: key.to_string(),
            value: value.as_bytes().to_vec(),
        },
    }
}

pub fn delete(replica: ReplicaId, counter: u64, context: VersionVector, key: &str) -> Operation {
    Operation {
        id: Dot { replica, counter },
        context,
        kind: OperationKind::Delete {
            key: key.to_string(),
        },
    }
}
