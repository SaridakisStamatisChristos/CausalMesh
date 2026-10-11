# Operations

Use `causalmesh status <DIR>` for deterministic JSON including replica ID, reserved counter, operation count, known replica count, unresolved concurrent-register count, tombstone count, and operation-log bytes.

`causalmesh verify <DIR>` performs the same fail-closed open/recovery path used by the library and emits status only after validation succeeds.

Back up all authoritative files together while the process is quiescent. Do not copy an active replica directory and run both copies as independent replicas; clone identity is detectable only when conflicting dots meet, not prevented at startup.

For unknown sync outcomes, retry. Operation IDs are stable and duplicate receipt is idempotent.
