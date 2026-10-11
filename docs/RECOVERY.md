# Recovery

Opening a replica requires all three authoritative files: `identity.json`, `counter.log`, and `ops.log`.

Recovery verifies identity checksum/version, scans counter reservations in strict sequence, scans framed operation records, validates every operation, reconstructs the CRDT operation set, and verifies that the maximum local operation counter does not exceed the durable reservation counter.

A truncated final frame is classified as a torn tail and truncated to the last complete verified frame. A complete frame with bad checksum is structural corruption and fails closed.

The durable local mutation order is:

1. append next counter reservation;
2. `sync_all(counter.log)`;
3. construct/validate operation;
4. append operation frame;
5. `sync_all(ops.log)`;
6. publish operation in memory;
7. return success.

A crash may consume a counter without creating an operation. This is safe. Once success is returned, the operation append has completed its declared sync boundary.
