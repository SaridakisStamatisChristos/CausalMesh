use crate::error::{Error, Result};
use crate::replica::Replica;
use crate::types::MAX_BATCH_OPS;
use serde::{Deserialize, Serialize};

// A conservative internal chunk size keeps framed JSON batches far below the 16 MiB
// record bound even for large byte-array values and causal contexts. Wire peers may use
// larger batches as long as the encoded record remains within bounds.
const INTERNAL_SYNC_CHUNK_OPS: usize = 2;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncReport {
    pub sent_a_to_b: usize,
    pub sent_b_to_a: usize,
    pub applied_at_a: usize,
    pub applied_at_b: usize,
    pub converged: bool,
}

pub fn sync_pair(a: &mut Replica, b: &mut Replica) -> Result<SyncReport> {
    if a.id() == b.id() {
        return Err(Error::CloneIdentity);
    }
    let summary_a = a.summary()?;
    let summary_b = b.summary()?;
    let a_to_b = a.operations_missing_from(&summary_b)?;
    let b_to_a = b.operations_missing_from(&summary_a)?;

    // Validate the complete logical deltas before either peer is durably mutated. I/O failure
    // can still leave one peer ahead, which is safe because retry is operation-idempotent.
    let mut dry_a = ReplicaView::from(a)?;
    let mut dry_b = ReplicaView::from(b)?;
    dry_a.preview(&b_to_a)?;
    dry_b.preview(&a_to_b)?;

    let applied_at_b = receive_chunked(b, &a_to_b)?;
    let applied_at_a = receive_chunked(a, &b_to_a)?;
    let converged = a.logical_state()? == b.logical_state()?;

    Ok(SyncReport {
        sent_a_to_b: a_to_b.len(),
        sent_b_to_a: b_to_a.len(),
        applied_at_a,
        applied_at_b,
        converged,
    })
}

fn receive_chunked(replica: &mut Replica, operations: &[crate::types::Operation]) -> Result<usize> {
    let chunk_size = INTERNAL_SYNC_CHUNK_OPS.min(MAX_BATCH_OPS);
    let mut applied = 0usize;
    for chunk in operations.chunks(chunk_size) {
        applied += replica.receive_operations(chunk)?;
    }
    Ok(applied)
}

struct ReplicaView {
    local_id: crate::types::ReplicaId,
    local_counter: u64,
    state: crate::crdt::CrdtState,
}

impl ReplicaView {
    fn from(replica: &Replica) -> Result<Self> {
        let mut state = crate::crdt::CrdtState::default();
        for op in replica.operations()? {
            state.apply(op)?;
        }
        Ok(Self {
            local_id: replica.id().clone(),
            local_counter: replica.status()?.local_counter,
            state,
        })
    }

    fn preview(&mut self, operations: &[crate::types::Operation]) -> Result<()> {
        for op in operations {
            if op.context.get(&self.local_id) > self.local_counter {
                return Err(Error::CloneIdentity);
            }
            if op.id.replica == self.local_id && !self.state.has_operation(&op.id) {
                return Err(Error::CloneIdentity);
            }
            self.state.apply(op.clone())?;
        }
        Ok(())
    }
}
