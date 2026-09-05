L-18 — PROJECTION COROLLARIES: losslessness + time-travel collapse
==================================================================
Track: L-T3 CONTEXT-CORE. Binds CC-1, CC-4.
Nature: BUILD-compose (collapses four separate mechanisms into parameterizations of the L-16 projection).
Lever: because the log is immutable and the view is a pure recomputable projection, lossless recovery and
  undo/redo/branch stop being their own subsystems and become "recompute the view at a chosen point/budget".
Wave: W2 | depends L-16.
Composes-with: K-12 compact.rs:545 (the critical-atom gate, re-pointed at the view), H-49 session_dag.rs
  undo-tree + CAS (now near-free), recover_stubs session_dag.rs:1539 (subsumed), the H-49 edit_checkpoints.
Authored 2026-06-30. Reference: CWD's immutable-log corollaries (nothing dropped -> recovery is recompute,
  time-travel is recompute-at-length).

BUILD
=====
Two corollaries of L-16, each replacing a mechanism:
1. LOSSLESSNESS becomes a VIEW assertion (CC-4). check_lossless_compaction (today: "are the to-be-dropped
   messages archived in SSB before we drop them", currently WARN) is re-pointed: assert the declared K-12
   critical atoms {kind, value, must_preserve} are PRESENT in the projected view (exact string/symbol presence,
   never semantic similarity). It can now be FAIL-CLOSED with no risk, because nothing is dropped from the log
   — a failed assert means the projection budget was too tight and must widen, not that data was lost. The
   K-12 R1 fail-closed flip is satisfied structurally.
2. TIME-TRAVEL becomes recompute-at-length. Undo = recompute the view from the log truncated to a chosen
   append index. Redo = move the index forward. Branch = fork the projection parameters at an index. The H-49
   undo-tree HEAD walks log indices, not stored diffs; edit_checkpoints and the CAS pre-image capture
   (file_ops.rs) are no longer needed for context recovery (they remain for FILE time-travel, a different
   grain). recover_stubs collapses to "project the log at the requested span with a larger budget".

COMPOSES-WITH (do NOT re-author)
================================
K-12: the critical-atom schema + the presence assert are reused verbatim, just evaluated over the view. H-49:
the undo-tree + branch grain are reused; this session changes what a node POINTS AT (a log index + projection
params) not the tree machinery. recover_stubs: kept for legacy DAG snapshots; the new path does not call it.

INTEGRATION
===========
crates/orchestrator/src/context_projection.rs (the project-at-index + the view-level atom assert) +
compact.rs (re-point check_lossless_compaction at the view, flip to FAIL-CLOSED per K-12 R1). SSB: prefer-reuse
the K-12 HandoffFaithfulnessGated / BeliefStateEvent rows + the H-49 EditHistoryEvent; net 0 expected. The
losslessness gate is auditor-owned + frozen in gate-manifest (off-agent, never self-run).

OBSERVABLE_EFFECT (done-gate)
=============================
A session compacted twice: every declared critical atom (a cited path, a hash, a port) is present verbatim in
the view at every later turn (CC-4, the assert is FAIL-CLOSED and green). An undo to an earlier turn recomputes
the exact earlier view (byte-identical to what was sent then, because fold is pure over the truncated log). A
branch forks without copying transcript bytes (CAS dedup). NEGATIVE: a projection budget too tight to hold a
must_preserve atom FAILS the gate and widens the budget, never ships a view missing the atom; the gate cannot
be passed by the producing model (auditor-owned, frozen).
