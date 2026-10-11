# Operations

Minimum commands:

```bash
causalmesh init <DIR>
causalmesh put <DIR> <KEY> <VALUE>
causalmesh get <DIR> <KEY>
causalmesh delete <DIR> <KEY>
causalmesh status <DIR>
causalmesh verify <DIR>
causalmesh export-summary <DIR>
causalmesh sync <DIR-A> <DIR-B>
```

Inspection/snapshot commands:

```bash
causalmesh dump-log <DIR>
causalmesh snapshot <DIR>
causalmesh compact <DIR>
causalmesh doctor <DIR>
```

`verify` revalidates durable files and canonical logical serialization. `doctor` additionally reports the replica ID and the clone-detection limitation.

For synchronization, make backups before manually copying or editing replica directories. Never duplicate a live replica directory and then allow both copies to mutate under the same identity.
