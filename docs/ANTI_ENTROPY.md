# Anti-Entropy

CausalMesh anti-entropy is transport-independent. The implemented `sync_pair` helper uses the same summary semantics in-process; wire envelopes make Hello, Summary, Operations, and Ack machine-readable for future transports.

## Round

1. exchange stable replica identities;
2. exchange exact received-range summaries;
3. compute operations absent from each peer's ranges;
4. validate both logical incoming batches before mutating either peer;
5. append each incoming batch as one checksummed durable log frame;
6. publish merged in-memory state only after the frame is synced;
7. compare resulting logical states for convenience reporting.

A single round between two replicas is complete because each summary covers every operation the replica currently stores, including operations learned from third parties.

## Delivery assumptions

No FIFO ordering is assumed. Duplicates, overlap, delayed old operations, reverse order, and repeated rounds are safe.

Eventual convergence requires that valid missing operations are eventually delivered and that replicas remain within declared resource/version bounds.

## Unknown outcome

If receiver B persists A's batch and crashes before A sees an acknowledgment, A may resend the same operation IDs. B recognizes identical duplicates as already known, so the retry creates no new logical mutation.
