# I-52: checkpoint/rewind agent tool — expose the shipped E-25 edit-checkpoint substrate as an agent-callable speculative-exploration tool

- Track 20 | lever CONCURRENCY-SAFETY | grounding STRONG (SL-6) | nature ACTIVATION | effort S-M |
  WAVE 3

- Premise: STEAL-LIST B2, corrected by SLC-1: gajae ships an agent-callable checkpoint/rewind tool for
  speculative exploration (try an approach, rewind cheaply, no worktree overhead). ELAI ALREADY OWNS the
  snapshot machinery — architecture/edit_checkpoints.rs + checkpoint_cmd.rs + two e25 contract tests are
  shipped (SL-6) — but no AGENT can call it: it is CLI/orchestrator-internal. I-52 is activation: a
  3-arm tool surface (save/rewind/diff) over the existing store, so a worker can speculate and roll back
  inside one task without burning a worktree (I-2's heavier isolation stays the right tool for PARALLEL
  branches; this is the in-branch, single-agent lever).

## Steal/build
- Steal the tool ergonomics only (gajae packages/coding-agent/src/tools/checkpoint.ts; SL-10 read rule).
- Build: a checkpoint tool in the agent tool catalog (the E-35/IS-14 3-arm catalog pattern): save
  (labelled snapshot via edit_checkpoints), rewind (restore to label, refusing if files changed outside
  the checkpointed set — divergence guard), diff (what changed since label). Scope-bound: an agent can
  only rewind ITS OWN session's checkpoints (no cross-session restore).
- H-49 COORDINATION CLAUSE (SLC-1): H-49 (PLANNED, Track 8) builds the full CAS/undo-tree over this same
  substrate. If H-49 ships first, I-52 re-scopes to the tool surface over H-49's store and REUSES its
  EditHistoryEvent row instead of adding one (decide at edit; prefer reuse — the I-0 delta table marks
  this conditional). I-52 must not build a second snapshot store either way.
- EXCLUDED: no redo/branching semantics (H-49's job); no git worktree integration (I-2's job); no
  cross-session time-travel; no UI beyond existing drawers.

## Existing ground truth
- SL-6: crates/orchestrator/src/architecture/edit_checkpoints.rs + crates/cli/src/checkpoint_cmd.rs +
  e25_edit_checkpoints_contract.rs + e25_edit_checkpoints_parallel_contract.rs all exist (grep/glob
  2026-06-10). H-plan Track 8 notes the substrate is test-proven but gated OFF and pre-images were being
  discarded at file_ops seams — the executor re-checks the current gate state (W-7) and, if still OFF,
  the activation includes the gate flip per W-2 (default-ON behind ELAI_*_DISABLE, or an allowlist row).

## REVISED scope (W-10 tiered)
- SHIP NOW: the 3-arm tool + session scoping + divergence-refusal + contract test, in-process.
- DEFER: H-49 store migration (the coordination clause), auto-checkpoint-before-risky-edit policy.

## Integration
- The agent tool catalog (3-arm add, E-35 pattern) + edit_checkpoints.rs (consumed, extended only with
  label/scope lookups). frontend_pair: checkpoint rows in the existing session/transcript drawer (no new
  component).

## WIRING_CONTRACT
- produces_symbols: the checkpoint agent tool (save/rewind/diff arms) + session-scope guard +
  CheckpointToolEvent SSB rows (CONDITIONAL: reuse H-49 EditHistoryEvent if shipped first — SLC-1).
- consumes_symbols: orchestrator::architecture::edit_checkpoints (the store — never re-implemented),
  the tool catalog registration path, file_ops pre-image capture seams.
- reachable_from: the production agent tool dispatch (a live worker calling the tool mid-task). A
  contracts-only caller does not satisfy (R7).
- stub_ban: NO second snapshot store; NO cross-session rewind; NO silent rewind over diverged files
  (refuse with the divergence set named); NO unwrap/expect on prod paths; NO leaving the substrate gated
  OFF while claiming the tool works (W-2 — the activation includes the gate posture).
- observable_effect:
  trigger: "a live agent task: save checkpoint A, edit two files, rewind to A"
  expect_ssb: "CheckpointToolEvent(Saved) then (Rewound) with the file set; a diverged-file fixture lands
  a REFUSAL row naming the diverged path"
  expect_artifact: "post-rewind file bytes blake3-match the checkpoint A pre-images; the NEGATIVE arm —
  rewinding another session's label — is REFUSED; a rewind that leaves a diverged file silently
  overwritten is the broken arm and FAILS (R2)."

## Grounding
- STEAL-LIST B2 (Tier 2); SL-6, SLC-1, SL-10.
- CLAUDE.md no-reimplementation (GD-1 — the store exists; this is a surface).

## COMPOSES-WITH
- E-25 edit_checkpoints (the substrate — activated), H-49 reversibility (the coordination clause; RV-1
  divergence-guard semantics inherited when it ships), I-2 workspace isolation (the PARALLEL sibling —
  disjoint scope), I-8 abort (rollback-to-trunk uses orchestrator paths, not this tool). Restates none.
  SSB: +CheckpointToolEvent (Kind: Saved/Rewound/Diffed/RefusedDiverged). baseline+1 from LIVE 221/532
  (SL-1) UNLESS the H-49 reuse clause fires; re-read at edit (W-9).

End of I-52.
