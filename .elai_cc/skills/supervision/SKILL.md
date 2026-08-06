# Supervision skill

Use these tools only for external CLIs and long-running loops whose progress is not
reported by ELAI Agent completion notifications. On each supervision cycle, call
`supervision_wedge.py check --path LOG --state STATE --n 3`; repeated unchanged
output is a deterministic WEDGE. Do not apply this check to ELAI Agent workers.

On supervisor startup or restart, first inspect the durable wake set with
`wake_queue.py list`, then deliver ready entries with `wake_queue.py dequeue`.
Enqueue SESSION_HANDOFF records with `handoff_topic`, `plan_id`, `reason`, and
`enqueued_ts`; repeated keys within the 30-second grace period are coalesced.

A WEDGE is an escalation through the existing T4 sweep-fail/fixer surface, not
an LLM stop decision. Operators must retain the existing kill switches:
`ELAI_SUPERVISION_WEDGE_DISABLE=1` disables wedge supervision and
`ELAI_WAKE_QUEUE_DISABLE=1` disables wake-queue supervision; unset means enabled.
The caller owns checking these switches before invoking the tools.

DO-NOT-COPY: do not add secondmates, use dangerous permission flags, or run a hard git reset (forbidden).
Both tools are stdlib-only and use deterministic JSON output / atomic queue writes.
