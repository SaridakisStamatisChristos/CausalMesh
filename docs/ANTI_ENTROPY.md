# Anti-Entropy

A synchronization round is transport-independent:

1. exchange replica identity and protocol version;
2. exchange exact dotted causal summaries;
3. select local operations whose dots are absent from the peer summary;
4. transfer bounded operation batches;
5. validate the whole batch, including same-dot conflicts, before persistent mutation;
6. durably append missing operations;
7. publish them to the in-memory CRDT;
8. acknowledge with an updated summary.

The library convenience helper executes these semantics in process. `WireMessage` provides machine-readable hello, summary, operations, and ack messages for external transports.

No FIFO delivery assumption exists. Exact summaries preserve sparse out-of-order dots. Duplicate and overlapping batches are safe. If a receiver durably persists a batch and crashes before ack, retry delivers the same immutable IDs and becomes an idempotent no-op.

Completeness assumption: if no new writes occur and valid required messages are eventually delivered in repeated sync rounds, every missing dot remains discoverable until transferred; therefore healthy replicas converge to an equal operation set and logical state.
