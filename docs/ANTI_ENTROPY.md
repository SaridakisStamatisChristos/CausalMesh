# Anti-Entropy

A synchronization round is transport-independent:

1. exchange protocol version and replica identity;
2. reject a peer that presents the local replica identity;
3. exchange exact dotted causal summaries;
4. select local operations whose dots are absent from the peer summary;
5. transfer bounded operation batches;
6. validate the whole batch, including same-dot conflicts and intra-batch duplicates, before persistent mutation;
7. durably append missing operations;
8. publish them to the in-memory CRDT;
9. acknowledge with an updated summary.

The library convenience helper executes these semantics in process. `WireMessage` provides machine-readable hello, summary, operations, and ack messages for external transports; external transports must call the same peer-identity validation before accepting a summary.

No FIFO delivery assumption exists. Exact summaries preserve sparse out-of-order dots. Duplicate and overlapping batches are safe. If a receiver durably persists a batch and crashes before ack, retry delivers the same immutable IDs and becomes an idempotent no-op. A sender that crashes after transmit but before learning the outcome can reopen and retry for the same reason.

Completeness assumption: if no new writes occur and valid required messages are eventually delivered in repeated sync rounds, every existing missing dot remains discoverable until transferred; therefore healthy replicas converge to an equal operation set and logical state. Reserved-but-never-committed local counters can create permanent holes in a causal summary, but no nonexistent operation is required for convergence.
