# H-6: Deterministic observation masking + memory read-back — cache-preserving context efficiency

STATUS: PLANNED (WAVE 1; in-process; ACTIVATION — three real-but-never-called symbols)

Objective: Deterministic observation masking HALVES cost while matching LLM-summarization solve-rate AND preserves
the cache prefix (it drops trailing observations rather than rewriting the prefix, so it compounds H-1/H-2 where
LLM re-summarization would bust the cache). Three real-but-dark symbols (C7, grep-confirmed 2026-06-03):
adaptive_reduce (context_budget.rs:1525) has ZERO non-test production callers — neither default-on nor gated, simply
never invoked; recover_stubs (session_dag.rs:1507) does REAL blake3-verified SSB content recovery and ERRORS on
missing SSB (it does NOT "fall back to a placeholder"), but the production read-back path (live_cli.rs:1252-1264)
substitutes a hardcoded "[stubbed — content archived in SSB]" placeholder by NOT calling it; get_relevant_facts
(memory/retrieval.rs:46) is trust-and-provenance-filtered memory read-back with zero non-test callers. H-6 makes
adaptive_reduce the default long-session context manager, un-gates the E-25 compaction ladder default-on
cheapest-first, and wires recover_stubs + get_relevant_facts into the live read paths.

Estimated effort: ~1-2 sessions (~6-10 h). Pure in-process; no network at gate time (W-10 Tier-1 whole).
Pre-requisites: E-25 ladder SHIPPED (compaction_ladder.rs, gated); F-5 memory read SHIPPED (get_relevant_facts);
  the ContextReduction SSB payload SHIPPED (ssb.rs:10134, write-active). H-6 activates; does not build the substrate.
Parallel-safe with: H-5/H-7/H-8. Coordinate with H-2 on the cache-preserving property (HD-2).
Source: ELAI-EFFICIENCY-TRUST-DIRECTION-2026-06-03.md §10.6 (LEVER L6) + §16/E3; _grounding/H-GROUNDING-MAP.md C7 + the H-6 table. Doctrine: H-0_INDEX.md.

## Context

"The Complexity Trap" (arXiv:2508.21433, NeurIPS25 workshop): deterministic observation masking halves cost
(50.9-56.1% per model) while MATCHING LLM-summarization solve-rate on SWE-bench Verified, and it BEATS summarization
on some models. The load-bearing property (HD-2): masking PRESERVES the cache prefix (it drops trailing observations)
whereas LLM re-summarization REWRITES the prefix and BUSTS the cache — so masking compounds H-1/H-2 and summarization
fights it. ELAI has NO LLM compaction to "replace" (CORRECTED §16/E3): compact_session (compact.rs:96) is already a
deterministic string formatter. The real situation is that the deterministic substrate (adaptive_reduce, the E-25
ladder, the memory read-backs) is BUILT but CALL-DARK. H-6 is activation, not invention. Context-rot (Chroma): a
bigger window degrades, so the move is a tight working set + a lossless journal backstop (the masked observation stays
re-fetchable from the SSB), not a larger context.

## Existing state (B/C/D/E/F/G coverage)
- adaptive_reduce (context_budget.rs:1525) — WIRE-DARK. 3 levels (FilteredByRelevance, ReducedGranularity, ReducedScope
  +Terminal); ZERO prod callers (only 2 contract-test files); write-active (ContextReduction SSB) but call-dark.
- run_compaction_ladder (compaction_ladder.rs:153) — WIRE-DARK (gate). Real prod caller task_runner.rs:9379 (PreCompact)
  but double-guarded by compaction_ladder_enabled() (:9354) + self-short-circuit (159-165); ELAI_E25_COMPACTION_LADDER_ENABLE
  default-OFF (:45). Per-tier reduction is a deterministic ESTIMATOR (ratios at :133), NOT real mutation (docstring :22-28).
- ALL_COMPACTION_TIERS (architecture/claude_code_arch.rs:419) — SHIPPED. 6 deterministic tiers cheapest-first.
- get_relevant_facts (memory/retrieval.rs:46) — WIRE-DARK. Trust+provenance-filtered read; pub-exported (lib.rs:91);
  every caller in crates/memory/tests/; bumps last_observed_at_ms on read (:7). Read-dark.
- recover_stubs (session_dag.rs:1507) + recover_single_stub (:1561) — WIRE-DARK. Real blake3-verified SSB recovery
  (msg.content=entry.payload :1551); ERRORS on missing/corrupt (StubRecoveryFailed); ZERO prod callers.
- create_handoff (session_dag.rs:1627) — SHIPPED (the stub PRODUCER; live_cli.rs:1241, 85% budget branch). Stubs created in prod, never read back.
- compact_session (compact.rs:96) — SHIPPED. Deterministic formatter; live /compact (commands.rs:4604) + --resume (resume.rs:510).
- live_cli.rs:1252-1264 — the hardcoded "[stubbed]" placeholder read-back (the ACTIVATE target for recover_stubs).

## Reads (mandatory before coding)
- _grounding/H-GROUNDING-MAP.md C7 + the H-6 symbol table.
- ELAI-EFFICIENCY-TRUST-DIRECTION-2026-06-03.md §10.6 (L6) + §16/E3 (there is NO LLM compaction to replace).
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\context_budget.rs:1520-1600 (adaptive_reduce + ContextReductionPayload)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\compaction_ladder.rs:20-170 (the gate + the estimator)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\session_dag.rs:1500-1640 (recover_stubs + create_handoff)
- F:\.ELAI_workflow\.elai_cc\crates\memory\src\retrieval.rs:40-90 (get_relevant_facts)
- F:\.ELAI_workflow\.elai_cc\crates\cli\src\live_cli.rs:1235-1270 (the placeholder read-back — the ACTIVATE target)
- CLAUDE.md HARD RULES + HD-2 (cache-preserving — masking drops trailing, never rewrites prefix).

## Tasks (WAVE 1 — activation)

1. **Make adaptive_reduce the default long-session context manager (~70 LOC).** Add a non-test production caller that
   invokes adaptive_reduce when a session exceeds a budget threshold (it has zero prod callers today). It must DROP
   TRAILING observations (cache-preserving, HD-2), never rewrite the stable prefix; the masked observation stays in the
   SSB journal (the lossless backstop, re-fetchable). Emit the existing ContextReduction SSB row (NO new action_type).

2. **Un-gate the E-25 ladder default-on cheapest-first + make masking REAL (~90 LOC).** Promote ELAI_E25_COMPACTION_LADDER_ENABLE
   off default-OFF (W-2: default-on or allowlisted with a promotion plan). The ladder per-tier is currently an ESTIMATOR
   (telemetry + estimated delta, does NOT mutate content — docstring :22-28); H-6 must make the cheapest applicable tier
   actually mutate the working set (drop trailing observations per the tier's policy) OR honestly scope what it does
   (GD-5: if a tier only estimates, say so; do not advertise a reduction it does not deliver).

3. **Wire recover_stubs into the live read-back path (~50 LOC).** Replace the hardcoded "[stubbed — content archived
   in SSB]" placeholder at live_cli.rs:1252-1264 with a real recover_stubs call so /continue rehydrates real content;
   recover_stubs ERRORS on missing SSB (fail-closed — correct), so handle the Err as a visible recovery failure, never a silent placeholder.

4. **Wire get_relevant_facts into the production memory read path (~40 LOC).** Add a non-test caller on a memory-relevant
   turn so the trust+provenance-filtered facts are actually read back (it bumps last_observed_at_ms on read).

5. **Decision doc workflow/reasoning/H-6-decisions.md (~100 LOC).** Records: there is NO LLM compaction to replace
   (compact_session is deterministic); the three call-dark symbols; HD-2 masking drops trailing only (cache-preserving);
   the ladder estimator-vs-real-mutation decision (GD-5); recover_stubs errors-on-missing (fail-closed).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/context_budget.rs (a production caller of adaptive_reduce on long sessions)
  - crates/orchestrator/src/compaction_ladder.rs (the default-on promotion + real-mutation cheapest-tier)
  - crates/cli/src/live_cli.rs (recover_stubs wired into the read-back, replacing the placeholder)
  - crates/memory/src/retrieval.rs OR its caller (a production caller of get_relevant_facts)
consumes_symbols:
  - crates/orchestrator/src/context_budget.rs::adaptive_reduce (+ ContextReductionPayload ssb.rs:10134, REUSE)
  - crates/orchestrator/src/compaction_ladder.rs::run_compaction_ladder (+ the ELAI_E25 gate)
  - crates/orchestrator/src/session_dag.rs::recover_stubs
  - crates/memory/src/retrieval.rs::get_relevant_facts
  - crates/orchestrator/src/compact.rs::compact_session (the live deterministic-formatter reference)
reachable_from:
  - crates/orchestrator/src/task_runner.rs:9379 (PreCompact branch) -> run_compaction_ladder (un-gated) -> ContextReduction SSB  [PRODUCTION compaction]
  - crates/cli/src/live_cli.rs:1252-1264 (the read-back) -> recover_stubs -> real content  [PRODUCTION /continue rehydration]
stub_ban:
  - NO LLM re-summarization that rewrites the cache prefix (HD-2 — masking drops trailing only)
  - NO hardcoded "[stubbed]" placeholder remaining at the read-back (recover_stubs replaces it)
  - NO ladder tier advertising a reduction it does not deliver (GD-5 — estimator vs real-mutation stated honestly)
  - NO default-OFF ELAI_E25_COMPACTION_LADDER_ENABLE as a terminal state (W-2)
  - NO silent loss: a masked observation MUST stay re-fetchable from the SSB journal
  - blake3 (recover_stubs verification); no .unwrap()/.expect() in prod
frontend_pair:
  the E-25 ladder state panel via get_compaction_ladder_state (commands_e25.rs:130, exists) — cross_lang_diff PASS.
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- <run a long real session that exceeds the budget threshold, then
    /continue in a fresh process, NO env override>"
  expect_ssb: "ContextReduction SSB rows land as trailing observations are masked; the masking is cache-preserving
    (the H-2 byte-prefix assert still holds across the masked turns)"
  expect_artifact: "the /continue rehydrates REAL content via recover_stubs (the journal shows the recovered payload,
    NOT the '[stubbed]' placeholder); a memory-relevant turn reads back facts via get_relevant_facts (last_observed_at_ms bumps);
    a masked observation is re-fetchable from the SSB journal"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-6_observation_masking_memory_readback.md --run-trigger
rg "adaptive_reduce" .elai_cc/crates/orchestrator/src --glob '!*test*' --glob '!*contract*'   # now a non-test prod caller
rg "recover_stubs" .elai_cc/crates/cli/src/live_cli.rs                                          # wired into the read-back
rg "\[stubbed" .elai_cc/crates/cli/src/live_cli.rs                                              # the hardcoded placeholder GONE (or only on the Err path)
rg "get_relevant_facts" .elai_cc/crates --glob '!*test*'                                        # now a non-test prod caller
rg -c "ELAI_E25_COMPACTION_LADDER_ENABLE" .elai_cc/crates/orchestrator/src/compaction_ladder.rs # gate promoted (kill-switch only)
python .claude/tools/cross_lang_diff.py
test -f workflow/reasoning/H-6-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: --run-trigger PASS (masking emits ContextReduction rows; /continue rehydrates real content via recover_stubs).
- [ ] W-2: adaptive_reduce default-on for long sessions; the E-25 ladder promoted off default-OFF.
- [ ] W-3: adaptive_reduce + recover_stubs fire from the live task_runner / live_cli paths; not test-only.
- [ ] HD-2: the masking is cache-preserving (the H-2 byte-prefix assert holds across masked turns); masking drops trailing, never rewrites prefix.
- [ ] GD-5: a ladder tier that only estimates is documented as such, not advertised as a reduction.
- [ ] GD-1: ContextReduction SSB is REUSED (no new action_type).
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-orchestrator -p elai-cli -p elai-memory.
- [ ] No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE 1; adaptive_reduce (consumed) is called on a live session (W-3, W-8); recover_stubs (consumed) replaces the placeholder.
- W-2: adaptive_reduce default-on; ELAI_E25 promoted (kill-switch only).
- W-3: task_runner.rs:9379 + live_cli.rs:1252-1264; not test.
- W-4: the E-25 ladder panel; cross_lang_diff PASS.
- W-5: a runnable long session + /continue -> real ContextReduction rows + real rehydration; in-process.
- W-6: the ladder panel via dashboard-smoker.
- W-7: adaptive_reduce / recover_stubs / get_relevant_facts / the placeholder site grep-confirmed 2026-06-03 (C7).
- W-8: the adaptive_reduce caller (produced) consumes adaptive_reduce; the read-back (produced) consumes recover_stubs.
- W-9: NO new action_type (reuse ContextReduction).
- W-10: Tier-1 in-process.
- HD-2: masking is cache-preserving (the load-bearing property — it compounds H-1/H-2).

## Tests to add
- crates/contracts/tests/h6_masking_readback_contract.rs (~170 LOC): (a) adaptive_reduce is invoked on a long session
  and emits ContextReduction rows; (b) masking drops TRAILING observations and leaves the stable prefix byte-identical
  (HD-2 — the H-2 assert holds); (c) a masked observation is re-fetchable from the SSB journal (lossless backstop);
  (d) recover_stubs rehydrates real content (not the placeholder) and ERRORS visibly on missing SSB; (e) get_relevant_facts
  is read on a memory-relevant turn and bumps last_observed_at_ms; (f) the ladder default-on cheapest tier actually mutates (or is documented as estimate-only).

## Files touched
- MODIFY: crates/orchestrator/src/context_budget.rs (~70 LOC: a prod caller of adaptive_reduce on long sessions)
- MODIFY: crates/orchestrator/src/compaction_ladder.rs (~90 LOC: default-on promotion + real-mutation cheapest tier)
- MODIFY: crates/cli/src/live_cli.rs (~50 LOC: recover_stubs replaces the placeholder)
- MODIFY: crates/memory/src/retrieval.rs OR its production caller (~40 LOC: a prod caller of get_relevant_facts)
- CREATE: crates/contracts/tests/h6_masking_readback_contract.rs (~170 LOC)
- CREATE: workflow/reasoning/H-6-decisions.md (~100 LOC)

## Citations / Grounding
- ELAI-EFFICIENCY-TRUST-DIRECTION-2026-06-03.md §10.6 (L6): Complexity Trap arXiv:2508.21433 (masking halves cost,
  matches summarization, PRESERVES the cache prefix); §16/E3 (NO LLM compaction to replace — compact_session is deterministic).
- ELAI ground truth (C7, grep-confirmed 2026-06-03): adaptive_reduce context_budget.rs:1525 (zero prod callers);
  recover_stubs session_dag.rs:1507 (zero prod callers; errors on missing SSB); get_relevant_facts retrieval.rs:46 (read-dark);
  the placeholder live_cli.rs:1252-1264; ladder estimator docstring compaction_ladder.rs:22-28.
- CLAUDE.md HARD RULES + HD-2.

## Open questions / risks
- Masking can drop an observation a later step needs; the SSB journal is the lossless backstop (re-fetchable) — the
  contract test MUST assert re-fetchability, not just that the row was masked.
- The ladder's per-tier estimator-vs-real-mutation gap (GD-5) is the honest scope question: making masking actually
  mutate the working set is the real win; if a tier only estimates, the plan must not claim its reduction. Decide per tier.
- recover_stubs errors-on-missing is fail-closed (correct), but the read-back must surface the error to the user, not
  silently fall back to the placeholder (that would re-introduce the dark path).

## Next
H-2's byte-stable prefix + H-6's cache-preserving masking compose: the prefix stays cached AND the volatile tail
shrinks. H-9 (cost cockpit) surfaces the masked-token reduction. This is the deterministic context-efficiency leg
the trust thesis prefers over LLM summarization (deterministic + reproducible + cache-preserving).

End of H-6.
