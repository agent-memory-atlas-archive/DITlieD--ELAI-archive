part9-followups.md

PART9 dashboard-finish auditor cycle 2 PASS (0 BLOCKs). 3 non-blocking WARNs deferred to a future micro-patch. Do not fix in PART9 commit; record only.

Session: s-40940
Phase: part9-dashboard-finish
Recorded: 2026-05-08


W-followup-1
File: .elai_cc/crates/dashboard-v2/src/App.svelte:8-12
Issue: comment says listener "registers BEFORE any ProjectSelector.openProject call", but Svelte onMount is bottom-up (child onMount fires before parent).
Risk: misleading comment only. Practical correctness intact — listener is registered before any user interaction can fire reindex_now (user input cannot arrive synchronously during the mount phase).
Fix: rephrase to "listener registers before any user interaction can trigger reindex_now, because user input cannot arrive synchronously during the mount phase."


W-followup-2
File: .elai_cc/crates/dashboard-v2/src/lib/state/reindex.svelte.ts:77-91
Issue: under HMR or test re-mount, bootstrap() resetting cancelled=false while a previous listen() Promise is still outstanding could leak the prior listener.
Risk: production unaffected — App.svelte never re-mounts. Affects HMR iteration loops only.
Fix: add a per-bootstrap generation counter; the cancellation check compares the captured generation against the current one before treating the listener as live.


W-followup-3
File: .elai_cc/crates/dashboard-v2/src/lib/state/reindex.svelte.ts:63
Issue: fade callback sets status='idle' but does not clear counts. reindex.counts therefore exposes stale data when status=idle.
Risk: StatusBar correctly gates on status !== 'idle' so no display bug. Future consumer reading counts without status check would see stale data.
Fix: add `counts = null;` in the fade setTimeout callback.
