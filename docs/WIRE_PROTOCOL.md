# Wire Protocol v1

`WIRE_VERSION = 1`.

The machine-readable envelope is deterministic JSON:

```json
{
  "version": 1,
  "message": {
    "type": "hello",
    "replica_id": "...uuid..."
  }
}
```

Message variants:

- `hello { replica_id }`
- `summary { summary }`
- `operations { sender, operations }`
- `ack { summary }`

The current CLI does not open a network socket; the protocol is intentionally transport-independent and can be carried over files, stdin/stdout, or a future authenticated network transport.

## Bounds

- total encoded message: 16 MiB;
- operations per batch: 1024;
- causal-context entries per operation: 4096;
- decoded counter ranges: 16384;
- key: 1024 UTF-8 bytes;
- value: 1 MiB.

Oversize input is rejected before semantic application. Unsupported protocol versions fail closed.

## Trust

Wire checksums are not presently layered over JSON messages because corruption/authentication belong to the transport layer; every operation is still validated before durable state mutation. Authentication and encryption are not claimed in v0.1.
