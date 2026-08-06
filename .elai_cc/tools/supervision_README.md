# Supervision harness tools

`supervision_wedge.py` records output signatures and reports `WEDGE` after repeated
unchanged cycles. `wake_queue.py` stores crash-safe SESSION_HANDOFF wake items,
coalescing repeated keys for 30 seconds. Both have `--help` CLIs and use only the
Python standard library. Callers should honor `ELAI_SUPERVISION_WEDGE_DISABLE=1`
and `ELAI_WAKE_QUEUE_DISABLE=1` before invoking them.
