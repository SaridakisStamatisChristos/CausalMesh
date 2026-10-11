# Wire Protocol

`WIRE_VERSION = 1`.

The reference encoding is deterministic JSON over Rust structs using ordered causal collections. Messages are tagged as `hello`, `summary`, `operations`, or `ack`.

All decoders enforce the maximum message byte length before JSON decoding. Operation batches have an explicit maximum count, every operation is validated, causal contexts are bounded, and unsupported versions fail closed.

Checksums in disk records detect accidental corruption. They are not authentication. Version 0.1 does not authenticate peers or encrypt transport.
