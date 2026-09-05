# H-3: Output-token stack — emit diffs on the cloud path + activate streaming early-abort (the 5x lever)

STATUS: PLANNED (WAVE 1; in-process; BUILD + a one-line ACTIVATE; the most-neglected lever)

Objective: Output bills 5x input and prompt caching never touches it (HD-3), so the output-token stack is where a
saved token is worth 5x the most. Three gaps (grep-confirmed 2026-06-03): (1) FocusTunnel DIFF mode NEVER emits a
diff in production (C6) — task_runner.rs:7871 always passes editor_fs_override=None to run_diff_step_with_provider,
which always returns UnsupportedMode and falls back to whole-output STRUCTURED; (2) streaming early-abort is DARK on
ALL paths (C5, premise "sidecar-only" FALSE) — the on_token hook sits in the SINGLE shared streaming loop serving
both cloud + sidecar but streaming_validator is hardcoded None at runtime_builder.rs:306 with no setter, and the
50-token on_chunk gate has ZERO production callers; (3) StreamingEarlyTerminated has NO production SSB emitter (only
tracing::warn). H-3 threads a real BufferEditor so DIFF emits diffs on the cloud path, relaxes ModeSelector Rule 1
(which forces Cloud->Structured), constructs Some(validator) at runtime_builder.rs:306 + calls on_chunk every ~50
tokens, and writes the StreamingEarlyTerminated SSB producer. The B-20 3-tier matcher (try_edit_tiers) IS shipped
via the STRUCTURED path — REUSE it, do not rebuild.

Estimated effort: ~2 sessions (~8-12 h). Pure in-process; no network at gate time (W-10 Tier-1 whole).
Pre-requisites: B-28 focus-tunnel SHIPPED (orchestrator.rs); B-20 edit-matching SHIPPED (try_edit_tiers dispatcher.rs:292).
Parallel-safe with: H-5 (batch, disjoint); coordinate with H-4 on production_agent_rpc.rs only if both touch dispatch.
Source: ELAI-EFFICIENCY-TRUST-DIRECTION-2026-06-03.md §10.3 (LEVER L3); _grounding/H-GROUNDING-MAP.md C5 + C6 + the H-3 table. Doctrine: H-0_INDEX.md.

## Context

Output is 5x input and caching never touches it (§10.0). Diff/SEARCH-REPLACE edits emit only changed lines instead
of whole files: Aider's unified-diff prompting cut "laziness" 20%->61% / 26%->59% AND improved quality [HOLDS];
SEARCH/REPLACE is best for larger models (Diff-XYZ arXiv:2510.12487; the prior "udiff-for-small" claim is REFUTED).
ELAI's edge vs Cursor (which deliberately uses FULL-FILE rewrites + speculative decoding, structurally output-heavy):
ELAI emitting diffs + FIM on a deterministic file plan is a defensible output-cost edge. Streaming early-abort is a
latency + downstream-token win (an aborted stream stops decoding before the rest of a wrong answer is generated) —
but the $ saving on aborted-stream BILLING is UNVERIFIED conjecture (GD-5: claim latency + downstream-token avoidance,
NOT a $ cut, until Anthropic billing on aborted streams is confirmed). HD-3 binds it: never force-JSON / constrained-
decode the coder/planner roles (arXiv:2408.02442 — constrained decoding degrades reasoning); constrained decoding is
for op-emission roles only (H-13 canvas ops).

## Existing state (B/C/D/E/F/G coverage)
- run_diff_step_with_provider (orchestrator/src/focus_tunnel/orchestrator.rs:266) — WIRE-DARK. task_runner.rs:7871
  passes editor_fs_override=None -> always UnsupportedMode (:306-310) -> STRUCTURED fallback + StructuredFallback SSB.
- run_fim_step_with_provider (focus_tunnel/orchestrator.rs:150) — WIRE-DARK. Caller task_runner.rs:7821 only when
  ELAI_FOCUS_TUNNEL_EXECUTE=1 (default-OFF) AND mode==Fim AND spec has target_file+edit_region.
- replay_hunks (focus_tunnel/diff.rs:613) — WIRE-DARK. 3-tier BufferEditor replay reached only when editor_fs_override=Some.
- StreamingValidator on_token/on_chunk (focus_tunnel/streaming.rs:548) — WIRE-DARK. on_token invoked runtime_client.rs:511
  inside `if let Some(validator)` but validator hardcoded None at runtime_builder.rs:306 (no setter); on_chunk (50-token
  gate: tree-sitter ERROR debounce + deprecated-import scan) has ZERO production callers.
- ModeSelector::select (focus_tunnel/mode.rs:175) — SHIPPED. select() at task_runner.rs:7248 every baby step; Rule 1
  forces Cloud->Structured (mode.rs:184). The decision side is live; the diff EXECUTION is dark.
- try_edit_tiers (tools/src/dispatcher.rs:292) — SHIPPED. EditMatchTier {Exact,WhitespaceNormalized,Ellipsis,NotMatched};
  consumed by dispatch_edit (:789); prod chain production_agent_rpc.rs:1184 (TOOL_EDIT). The STRUCTURED edit path — REUSE.
- StreamingEarlyTerminated — NO production SSB emit site (early_term only tracing::warn at runtime_client.rs:523-531).

## Reads (mandatory before coding)
- _grounding/H-GROUNDING-MAP.md C5 + C6 + the H-3 symbol table.
- ELAI-EFFICIENCY-TRUST-DIRECTION-2026-06-03.md §10.3 (L3) + §15 (output-reduction ledger; the aborted-stream-billing UNVERIFIED caveat).
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\focus_tunnel\orchestrator.rs:117-320 (run_diff/run_fim + UnsupportedMode)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\focus_tunnel\mode.rs:175-200 (ModeSelector + Rule 1)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\focus_tunnel\streaming.rs:540-600 (StreamingValidator on_token/on_chunk)
- F:\.ELAI_workflow\.elai_cc\crates\cli\src\runtime_client.rs:450-540 (the shared streaming loop + the early-abort hook) + runtime_builder.rs:300-310 (the None decision point)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\task_runner.rs:7240-7960 (the focus-tunnel pre-executor branch + the editor=None site)
- CLAUDE.md HARD RULES + HD-3 (output-is-5x / decoding-is-sacred).

## Tasks (WAVE 1 — build the cloud diff path + activate streaming)

1. **Thread a real BufferEditor so DIFF emits on the cloud path (~90 LOC).** Add an editor slot (Arc<dyn BufferEditor>)
   to ResolvedConfig / BabyStepExecutionCtx and supply it at task_runner.rs:7871 instead of None, so
   run_diff_step_with_provider gets editor_fs_override=Some and replay_hunks fires (the 3-tier replay). The cloud model's
   edit lands as a diff, not a whole-file STRUCTURED rewrite.

2. **Relax ModeSelector Rule 1 (~40 LOC).** Remove/relax the Cloud->Structured force (mode.rs:184) so a cloud baby step
   can select a diff mode when the spec has a clear edit region. Keep STRUCTURED as the fallback when the diff path is
   genuinely unsuitable (large refactor, new file).

3. **Activate streaming early-abort (the one-line + the gate) (~60 LOC).** Construct Some(StreamingValidator) at
   runtime_builder.rs:306 (the single decision point; today None, no setter) so the on_token hook in the shared loop
   (runtime_client.rs:508-531) is live on BOTH cloud + sidecar; add the on_chunk(accum) call every ~50 tokens
   (today ZERO production callers) so the tree-sitter ERROR debounce + deprecated-import scan actually run.

4. **Write the StreamingEarlyTerminated SSB producer (~50 LOC).** On EarlyTerm, emit an SSB row (today only
   tracing::warn at runtime_client.rs:523-531). If the SsbActionType::StreamingEarlyTerminated variant already exists
   (grep ssb.rs), ACTIVATE the emitter (no bump); if absent, +streaming_early_terminated (Pattern-2) baseline+1 (then
   also bump e45_cross_language_drift_contract.rs:294).

5. **Promote ELAI_FOCUS_TUNNEL_EXECUTE per W-2 (~15 LOC).** A default-OFF flag is not a terminal state: either flip
   it default-ON (with the diff path now real) or register it in wire-dark-allowlist.toml with a promotion plan + a
   smoke test exercising the ON path.

6. **Decision doc workflow/reasoning/H-3-decisions.md (~100 LOC).** Records: editor=None is why DIFF never emits;
   Rule 1 forces Structured; streaming dark on ALL paths (not sidecar-only); the B-20 matcher is reused via STRUCTURED
   (not rebuilt); aborted-stream billing is UNVERIFIED (latency + downstream-token claim only); HD-3 no-force-JSON-on-coder.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/task_runner.rs (a real Arc<dyn BufferEditor> threaded into the diff step instead of None)
  - crates/orchestrator/src/focus_tunnel/mode.rs (relaxed Rule 1 so Cloud can select a diff mode)
  - crates/cli/src/runtime_builder.rs (Some(StreamingValidator) at the :306 decision point)
  - crates/cli/src/runtime_client.rs (the on_chunk ~50-token gate call + the StreamingEarlyTerminated SSB emit)
  - (conditional) crates/orchestrator/src/ssb.rs::SsbActionType::StreamingEarlyTerminated (if absent)
consumes_symbols:
  - crates/orchestrator/src/focus_tunnel/orchestrator.rs::run_diff_step_with_provider
  - crates/orchestrator/src/focus_tunnel/diff.rs::replay_hunks
  - crates/tools/src/dispatcher.rs::try_edit_tiers (SHIPPED — REUSE via the STRUCTURED path)
  - crates/orchestrator/src/focus_tunnel/streaming.rs::StreamingValidator (on_token/on_chunk)
  - crates/orchestrator/src/focus_tunnel/mode.rs::ModeSelector::select
reachable_from:
  - crates/orchestrator/src/task_runner.rs:7762/7871 (the focus-tunnel pre-executor branch) -> run_diff_step_with_provider(editor=Some) -> diff emitted  [PRODUCTION cloud edit path]
  - crates/cli/src/runtime_client.rs:508-531 (the shared streaming loop) -> on_token/on_chunk -> EarlyTerm -> StreamingEarlyTerminated SSB  [PRODUCTION streaming path, cloud + sidecar]
stub_ban:
  - NO re-implementation of the B-20 3-tier matcher (try_edit_tiers is shipped — REUSE, GD-1 P0)
  - NO force-JSON / constrained-decode on the coder/planner role (HD-3)
  - NO default-OFF ELAI_FOCUS_TUNNEL_EXECUTE as a terminal state (W-2: promote or allowlist with a plan)
  - NO $ claim on aborted-stream billing (UNVERIFIED — latency + downstream-token only, GD-5)
  - NO silent free-form on a grammar/mode miss; blake3; no .unwrap()/.expect() in prod
frontend_pair:
  GenerationModeSelected + StreamingEarlyTerminated payloads mirrored in dashboard-v2 TS (api/types.ts, CloudPanel.svelte)
  — the wire form is locked; activation is Rust-side. cross_lang_diff.py PASS.
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- <run a real cloud-mode edit task on an existing file with a clear
    edit region, NO env override; plus a streaming run that hits an early-abort condition (e.g. a tree-sitter ERROR)>"
  expect_ssb: "a StreamingEarlyTerminated SSB row lands on the early-abort run; the edit task's output-token count
    (journal-recorded) is materially lower than the pre-H-3 whole-file STRUCTURED baseline"
  expect_artifact: "the cloud edit lands on disk as a DIFF / SEARCH-REPLACE (changed lines only), verified by the
    journal showing a diff op not a whole-file rewrite; the early-abort run stops decoding mid-stream"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-3_output_token_diff_fim_streaming.md --run-trigger
rg "editor_fs_override|BufferEditor" .elai_cc/crates/orchestrator/src/task_runner.rs   # editor now Some on the diff step
rg "streaming_validator: Some|StreamingValidator::new" .elai_cc/crates/cli/src/runtime_builder.rs  # validator constructed (was None)
rg "on_chunk" .elai_cc/crates/cli/src/runtime_client.rs                                 # the 50-token gate now called in prod
rg "StreamingEarlyTerminated" .elai_cc/crates/orchestrator/src/ssb.rs .elai_cc/crates/cli/src/runtime_client.rs  # producer wired
python .claude/tools/cross_lang_diff.py
test -f workflow/reasoning/H-3-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: --run-trigger PASS (the cloud edit emits a diff; the early-abort emits a StreamingEarlyTerminated row).
- [ ] W-2: ELAI_FOCUS_TUNNEL_EXECUTE promoted (default-ON or allowlisted with a promotion plan + smoke test); streaming validator default-ON.
- [ ] W-3: the diff path fires from the live task_runner focus-tunnel branch; the streaming hook from the live runtime_client loop; not test-only.
- [ ] W-4: GenerationModeSelected + StreamingEarlyTerminated cross-lang mirrored; cross_lang_diff PASS.
- [ ] HD-3: no force-JSON on the coder role; output-token reduction MEASURED (not multiplied with caching).
- [ ] GD-5: no $ claim on aborted-stream billing (latency + downstream-token avoidance only).
- [ ] GD-1: try_edit_tiers is REUSED via STRUCTURED, not re-implemented.
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-orchestrator -p elai-cli -p elai-tools.
- [ ] No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE 1; the threaded BufferEditor (produced) drives run_diff_step (consumed) on the live branch (W-3, W-8).
- W-2: ELAI_FOCUS_TUNNEL_EXECUTE promoted; the streaming validator is constructed by default.
- W-3: task_runner.rs:7762/7871 (focus-tunnel branch) + runtime_client.rs:508-531 (streaming loop); not test.
- W-4: dashboard TS mirror for the two payloads; cross_lang_diff PASS.
- W-5: a runnable cloud edit + an early-abort run -> a real diff on disk + a real SSB row; in-process.
- W-6: the GenerationModeSelected/EarlyTerminated tiles via dashboard-smoker (or N/A justified if CloudPanel suffices).
- W-7: run_diff_step / replay_hunks / StreamingValidator / mode.rs Rule 1 / try_edit_tiers grep-confirmed 2026-06-03 (C5/C6).
- W-8: the BufferEditor (produced) is consumed by run_diff_step; the on_chunk call (produced) consumes StreamingValidator.
- W-9: conditional StreamingEarlyTerminated bump (baseline+1 only if absent; then bump e45:294).
- W-10: Tier-1 in-process.
- HD-3: no constrained decoding on the coder role; the win is output tokens, counted 5x, measured not multiplied.

## Tests to add
- crates/contracts/tests/h3_output_stack_contract.rs (~180 LOC): (a) a cloud edit with editor=Some emits a diff op
  (not StructuredFallback); (b) ModeSelector permits a diff mode for Cloud with a clear edit region (Rule 1 relaxed);
  (c) the streaming validator is Some by default and on_token/on_chunk fire; (d) an early-abort condition emits a
  StreamingEarlyTerminated SSB row + stops decoding; (e) the B-20 try_edit_tiers path is reused (no second matcher);
  (f) no constrained-decoding grammar is applied to the coder role (HD-3).

## Files touched
- MODIFY: crates/orchestrator/src/task_runner.rs (~90 LOC: thread Arc<dyn BufferEditor> into the diff step)
- MODIFY: crates/orchestrator/src/focus_tunnel/mode.rs (~40 LOC: relax Rule 1)
- MODIFY: crates/cli/src/runtime_builder.rs (~15 LOC: Some(StreamingValidator))
- MODIFY: crates/cli/src/runtime_client.rs (~70 LOC: on_chunk gate call + StreamingEarlyTerminated emit)
- MODIFY: crates/dashboard-v2/src/lib/api/types.ts + CloudPanel.svelte (~10 LOC: the EarlyTerminated payload mirror)
- CREATE: crates/contracts/tests/h3_output_stack_contract.rs (~180 LOC)
- CREATE: workflow/reasoning/H-3-decisions.md (~100 LOC)
- (conditional) MODIFY: crates/orchestrator/src/ssb.rs + action_type_lock.rs + e45 contract (only if StreamingEarlyTerminated absent)

## Citations / Grounding
- ELAI-EFFICIENCY-TRUST-DIRECTION-2026-06-03.md §10.3 (L3): Aider udiff laziness 20->61% / 26->59% [HOLDS];
  Diff-XYZ arXiv:2510.12487 SEARCH/REPLACE best for larger models [HOLDS] (udiff-for-small REFUTED); aborted-stream
  billing UNVERIFIED [DESIGN]; FIM Hidden-Cost-of-Readability arXiv:2508.13666 24.5% input cut [CORRECTED].
- ELAI ground truth (C5/C6, grep-confirmed 2026-06-03): run_diff_step orchestrator.rs:266 (editor=None -> UnsupportedMode);
  StreamingValidator streaming.rs:548 (validator None at runtime_builder.rs:306); ModeSelector Rule 1 mode.rs:184;
  try_edit_tiers dispatcher.rs:292 (shipped); StreamingEarlyTerminated only tracing::warn runtime_client.rs:523-531.
- CLAUDE.md HARD RULES + HD-3.

## Open questions / risks
- Diff edits fail when the model miscounts lines (Aider's 9x error increase without flexible patching); ELAI's 3-tier
  edit matching (B-20 try_edit_tiers) is the mitigation and must be on the diff path, not just the STRUCTURED path.
- Relaxing Rule 1 risks Cloud picking a diff mode for an unsuitable change (large refactor); keep STRUCTURED as the
  fallback when the spec lacks a clear edit region; measure the fallback rate.
- Aborted-stream billing remains UNVERIFIED — the gate measures output-token reduction on the DIFF path (certain),
  not a streaming $ saving (unconfirmed). Confirm Anthropic aborted-stream billing before any $ claim (GD-5).

## Next
H-9 (cost cockpit) surfaces the output-token reduction as the input:output ratio tile. H-2's 1h TTL + H-3's diffs
compose: cheaper repeated input + cheaper output on the same task. H-13 (canvas ops) is where constrained decoding
IS admissible (op-emission, not the coder role) — H-3's HD-3 boundary keeps them separate.

End of H-3.
