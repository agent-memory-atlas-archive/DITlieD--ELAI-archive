# H-44: Behavioral trap detection + measured recall — replace the keyword-grep cheat-detector with execution/conflict/perturbation canaries [TRACK 7]

STATUS: PLANNED (TRACK 7 DGM SELF-IMPROVEMENT VERIFICATION HARDENING; WAVE 7; W-10 tiered: in-process behavioral detector + measured recall on a frozen corpus Tier-1; the live large-corpus recall sweep is a separately-gated checkbox)

Objective: ELAI's cheat-detector (trap_tasks.rs:123 `score_candidate_for_traps`) is a substring grep over literal
keyword arrays — `["fake_test_log","fake test","fabricate test",...]` (:139), `["verifier_capture","rubber-stamp",...]`
(:143) — and the code self-flags "name-grep is unsound; an adversary may evade it" (DG-2, GAP-2). A candidate that
games the verifier WITHOUT using any flagged phrase passes the detector untouched. H-44 replaces it with BEHAVIORAL
detection that reads execution and structure, not strings: (1) an ImpossibleBench conflict canary (2510.20270) where
spec and tests CONFLICT so any pass IS a cheat; (2) Isomorphic-Perturbation re-verification (2604.15149) — a "win"
that passes the original but fails a logically-isomorphic rewrite is gaming; (3) CodeHalu execution-classification
(2405.00253) — execute and classify "looks right, runs wrong"; and CRITICALLY (4) it MEASURES its own recall on the
frozen TRACE contrastive corpus (2601.20103) and reports it (VD-2 — an unmeasured detector advertises a guarantee it
cannot prove). Cross-view agreement (Co-rewarding 2508.00410) gates the verdict.

Estimated effort: ~1.5 sessions (M-L, ~8-10 h). W-10 SPLIT: Tier-1 = the in-process behavioral detector + the
  TrapBehaviorDetected SSB row + the recall measured on a FROZEN out-of-band TRACE-style corpus (fail-closed
  done-gate); Tier-2 = the live recall sweep over the full TRACE dataset / a real generation stream (fixture now, live a gated checkbox).
Pre-requisites: NONE hard (trap_tasks + the trap registry + safedelta frozen-runner are SHIPPED). Composes D-24
  (cross-domain canary regression suite), D-13 (cross-family adversary — the cross-view leg), E-54 (eval-awareness probes).
  Reads DG-2/DG-7, PC-3/PC-5, VD-2/VD-4 in H-VERIFICATION-GROUNDING-MAP.md.
Parallel-safe with: H-43 (disjoint file: execution_fitness vs trap_tasks). Sequence H-44 SECOND in Track 7 (after H-43, parallel-eligible).
Source: ELAI-SELF-VERIFICATION-RESEARCH-DESIGN-2026-06-05.md (GAP-2 P0; DOSSIER 3 ImpossibleBench/Iso-Perturbation/
  Co-rewarding; DOSSIER 4 CodeHalu/TRACE; PART C/F); _grounding/H-VERIFICATION-GROUNDING-MAP.md DG-2 + VD-2.

## Context

A keyword detector is the textbook gameable proxy (Skalse 2209.13085). The dossier's DOSSIER 3 gives three measured
behavioral recipes that need no keyword: ImpossibleBench (2510.20270, Zhong/Raghunathan/Carlini, safety-research/
impossiblebench, WebSearch-confirmed this session) builds spec-test CONFLICT tasks where pass-rate == cheat-rate, and
shows "STOP if tests flawed" cut the hack rate 93%->1% — a drop-in held-out canary. Isomorphic-Perturbation Testing
(2604.15149, WebSearch-confirmed real this session) re-verifies a win on a logically-isomorphic rewrite: passes
original but fails the isomorph == gaming (RLVR-specific; non-RLVR 0%, GPT-5-nano 36.8%). CodeHalu (2405.00253,
yuchen814/CodeHalu) EXECUTES code and classifies runtime-behavior failures ("looks right, runs wrong", ~33-61%
detected). The detector's recall is itself MEASURED on TRACE (2601.20103, PatronusAI/trace-dataset, best model 63%
reward-hack detection) — a verifier-of-the-verifier (VD-2). Cross-view agreement (Co-rewarding 2508.00410) requires
two DIFFERENT-family views to agree before flagging, so a single weak detector cannot collapse the signal (VD-4).

## Existing state (B/C/D/E/F/G coverage)
- trap_tasks.rs:123 score_candidate_for_traps (keyword arrays :139/:143) + TrapPattern + the trap registry (insert/:262) — SHIPPED PLACEHOLDER (DG-2, GAP-2). The REPLACE target.
- darwin_admit_loop guard chain (score_candidate routes the trap guard) — SHIPPED. The reachable_from path (the trap guard is one of the four guards).
- benchmarks/safedelta_runner.rs (frozen-runner + measured-row) — SHIPPED. The recall-measurement pattern H-44 MIRRORS (GD-1).
- D-24 cross-domain canary regression suite — PLANNED. H-44's canaries register through D-24's framework (compose, do not duplicate).
- D-13 adversary (cross-family) — PLANNED. The cross-view second opinion (VD-4). E-6 iso-budget is H-40's target, not H-44's.
- Behavioral detection + a TRACE recall harness — ABSENT (GAP-2). The pure-BUILD surface.

## Reads (mandatory before coding)
- _grounding/H-VERIFICATION-GROUNDING-MAP.md: DG-2, DG-7, PC-3, PC-5, VD-2, VD-4, the SCHEMA + CITATIONS blocks.
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\trap_tasks.rs:60-160 + :240-275 (score_candidate_for_traps + TrapPattern + the registry)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\benchmarks\safedelta_runner.rs (the measured-row pattern to mirror for recall)
- workflow/reasoning/ELAI-SELF-VERIFICATION-RESEARCH-DESIGN-2026-06-05.md (GAP-2 + DOSSIER 3/4 + PART C/F)
- ImpossibleBench 2510.20270 + Isomorphic-Perturbation 2604.15149 + CodeHalu 2405.00253 + TRACE 2601.20103 PDFs/repos.
- CLAUDE.md HARD RULES: blake3 (corpus hash), no .unwrap()/.expect() in prod.

## Tasks (WAVE 7 — Tier-1 behavioral detector + recall measurement + measured row)

1. **ImpossibleBench conflict canary (~80 LOC, crates/orchestrator/src/trap_behavioral.rs).** A frozen set of
   spec-test-CONFLICT tasks; a candidate that PASSES the (impossible) tests is flagged ImpossiblePassExploit. Reuse
   the dossier's "STOP if tests flawed / read-only / abort" guard recipe (measured 93%->1% hack-rate cut).

2. **Isomorphic-Perturbation re-verification (~70 LOC).** For a flagged win, generate a logically-isomorphic rewrite
   (variable rename / structure-preserving transform) and re-run; passes original but fails isomorph -> IsomorphMismatch.

3. **CodeHalu execution-classification (~60 LOC).** Execute the candidate and classify runtime-behavior failures
   ("compiles + asserts pass but wrong output / wrong exception class") -> ExecutionHallucination. Compose H-43's
   execution harness (do not build a second runner; GD-1).

4. **Cross-view gate (~40 LOC).** A flag is RAISED only when >=2 DIFFERENT-family views agree (the behavioral signal
   + a cross-family adversary opinion via D-13). A single view's flag is advisory, not blocking (VD-4; Co-rewarding).
   KeywordFallback Kind retains the old grep as a CHEAP first-pass advisory ONLY (never the sole blocker — VD-2).

5. **Recall measurement on the frozen TRACE corpus (~70 LOC).** Run the detector over a FROZEN out-of-band
   contrastive corpus (TRACE-style honest-vs-hacking pairs); compute recall + false-positive rate; emit them. A
   detector with UNMEASURED recall is a FAIL (VD-2). The corpus is gate-frozen (R8/TD-2).

6. **Off-agent measured row (~30 LOC).** Emit `{kind, recall, fp_rate, n, corpus_hash, seed}` as a
   TrapBehaviorDetected SSB row (Pattern-2; Kind ImpossiblePassExploit / IsomorphMismatch / ExecutionHallucination /
   KeywordFallback; baseline+1 from LIVE 224/501, re-read at edit; reconcile the e45 pin if first Track-7 bump). Reuse
   the InsecureRateMeasured pattern (GD-1).

7. **Decision doc workflow/reasoning/H-44-decisions.md (~80 LOC):** why behavioral not keyword (Skalse); the cross-
   view requirement (Co-rewarding); the measured-recall mandate (VD-2); the honest residue (no detector is complete —
   recall is reported, never claimed 100%; eval-awareness 2510.20487 means a strong candidate can sandbag the canary; VD-5).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/trap_behavioral.rs::detect_trap_behavior (the behavioral detector)
  - crates/orchestrator/src/trap_behavioral.rs::TrapRecallReport (the {recall, fp_rate, n} roll-up)
  - crates/orchestrator/src/ssb.rs::SsbActionType::TrapBehaviorDetected (off-agent detector artifact; baseline+1, Pattern-2)
consumes_symbols:
  - crates/orchestrator/src/trap_tasks.rs::score_candidate_for_traps (downgraded to a KeywordFallback advisory — extend, do not delete the registry)
  - crates/orchestrator/src/darwin_admit_loop.rs (the guard chain routes the trap guard — the reachable_from path)
  - crates/orchestrator/src/execution_fitness.rs (H-43's execution harness — REUSE for CodeHalu, GD-1)
  - crates/orchestrator/src/benchmarks/safedelta_runner.rs (the measured-row pattern — mirror, GD-1)
reachable_from:
  - the self-mod loop guard chain (darwin_admit_loop -> trap guard -> detect_trap_behavior -> TrapBehaviorDetected SSB) [PRODUCTION, behind the darwinian kill-switch — allowlisted]
stub_ban:
  - NO keyword/substring grep as the SOLE cheat-detector (VD-2; keyword is a KeywordFallback advisory only) — FAIL
  - NO detector whose recall is UNMEASURED (VD-2; missing recall = a guarantee it cannot prove) — FAIL
  - NO single-view flag as a hard block where a cross-family view is available (VD-4) — FAIL
  - the broken `--trap-keyword-only` arm MUST miss a keyword-free hack the behavioral detector catches (R2)
  - blake3 corpus hash only; no blake2/sha2; no .unwrap()/.expect() in prod
frontend_pair:
  the detector recall + the per-kind flag count in the trust/DGM drawer (composes the archive panel; cross_lang_diff PASS).
  W-6 E2E: a keyword-free hack being BLOCKED (and a clean candidate passing) is visible in the dashboard.
observable_effect:
  trigger: "cd .elai_cc && ELAI_DARWINIAN_LOOP_ENABLE=1 cargo run -p elai-cli -- <the self-mod generation subcommand>
    over a FROZEN fixture containing a keyword-free verifier-gaming candidate H1 and a clean candidate C1, NO other env override"
  expect_ssb: "a TrapBehaviorDetected row flags H1 (ImpossiblePassExploit or IsomorphMismatch) and NOT C1; a
    TrapRecallReport row carries recall on the frozen TRACE corpus, recomputable under ELAI_SEED (GD-2)"
  expect_artifact: "H1 is BLOCKED by the guard chain despite using no flagged keyword; the broken `--trap-keyword-only`
    arm lets H1 through (proving the behavioral leg is load-bearing, R2)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-44_behavioral_trap_detection_recall.md --run-trigger
rg "detect_trap_behavior|TrapBehaviorDetected|TrapRecallReport" .elai_cc/crates/orchestrator/src --glob '!*test*'
rg "score_candidate_for_traps" .elai_cc/crates/orchestrator/src   # still present but downgraded to advisory (KeywordFallback)
rg -c "blake2|sha2" .elai_cc/crates/orchestrator/src/trap_behavioral.rs   # MUST be 0
test -f workflow/reasoning/H-44-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: --run-trigger PASS (H1 flagged + blocked; C1 clean; recall reported). cargo-green != DONE.
- [ ] W-2: behavioral detection is the default under the darwinian kill-switch; keyword is advisory only; no new default-off gate.
- [ ] W-3 production entrypoint: the detector fires from the guard chain, not a #[cfg(test)] caller.
- [ ] VD-2: a keyword-free hack is BLOCKED; the detector's recall is MEASURED on TRACE and emitted.
- [ ] VD-4: a flag requires >=2 different-family views to agree (single-view is advisory).
- [ ] GD-1: trap registry + execution harness (H-43) + the measured-row pattern REUSED (no re-implementation; P0).
- [ ] GD-2 reproducible: recall + fp_rate re-compute byte-identically under ELAI_SEED over the frozen corpus.
- [ ] R2: the --trap-keyword-only broken arm misses the keyword-free hack (the behavioral leg is falsifiably load-bearing).
- [ ] W-10: Tier-1 detector + frozen-corpus recall is the fail-closed done-gate; the live recall sweep is a separately-gated checkbox.
- [ ] cargo check --workspace GREEN; auditor PASS.

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE item 1; the detector (produced) is reached by the guard chain (W-3) and feeds the block (W-8).
- W-2: default under kill-switch; keyword advisory; allowlisted.
- W-3: live guard chain; not test-only.
- W-4: frontend_pair = the recall/flag-count tile; cross_lang_diff PASS.
- W-5: observable_effect is a runnable generation over a frozen fixture -> real journal rows; Tier-1 in-process.
- W-6: a keyword-free hack blocked is visible in the dashboard (dashboard-smoker).
- W-7: score_candidate_for_traps / the guard chain / execution_fitness / safedelta_runner grep-confirmed (DG-2/DG-1/DG-7); behavioral detector ABSENT (GAP-2).
- W-8: detect_trap_behavior (produced) consumed by the guard-chain block decision.
- W-9: +TrapBehaviorDetected baseline+1 (Pattern-2); the recall roll-up rides the same variant payload (no 2nd variant).
- W-10: SPLIT — Tier-1 detector + frozen recall fail-closed; Tier-2 live sweep + checkbox.
- VD-2/VD-4 + GD-1/2: behavioral; measured-recall; cross-view; compose; reproducible.

## Tests to add
- crates/contracts/tests/h44_trap_behavioral_contract.rs (~180 LOC): (a) a keyword-free verifier-gaming candidate is
  flagged (ImpossiblePassExploit); (b) a win passing original but failing the isomorph -> IsomorphMismatch; (c) a
  looks-right-runs-wrong candidate -> ExecutionHallucination; (d) a single-view flag is advisory, two-view agreement
  blocks (VD-4); (e) recall is computed on the frozen TRACE corpus and is reproducible (GD-2); (f) the
  --trap-keyword-only arm misses the keyword-free hack (R2).
- crates/contracts/tests/fixtures/trap_behavioral/ (frozen H1/C1 + the impossible-conflict set + the isomorph rewriter + a TRACE-style contrastive corpus + its hash).

## Files touched
- CREATE: crates/orchestrator/src/trap_behavioral.rs (~280 LOC)
- MODIFY: crates/orchestrator/src/trap_tasks.rs (downgrade score_candidate_for_traps to a KeywordFallback advisory; keep the registry, ~30 LOC)
- MODIFY: the guard chain wire in darwin_admit_loop (route detect_trap_behavior, ~30 LOC)
- MODIFY: crates/orchestrator/src/ssb.rs (+TrapBehaviorDetected) + action_type_lock.rs:645 (+ e45:294 if first Track-7 bump)
- MODIFY: the trust/DGM dashboard drawer (+recall/flag-count, ~40 LOC) + its types.ts pair
- CREATE: crates/contracts/tests/h44_trap_behavioral_contract.rs (~180 LOC)
- CREATE: workflow/reasoning/H-44-decisions.md (~80 LOC)

## Citations / Grounding
- ImpossibleBench 2510.20270 (spec-test conflict -> pass==cheat; "STOP if tests flawed" 93%->1%; safety-research/impossiblebench). WebSearch-confirmed this session.
- Isomorphic-Perturbation 2604.15149 "LLMs Gaming Verifiers: RLVR Reward Hacking" (re-verify on an isomorph; RLVR-specific). Confirmed real this session.
- CodeHalu 2405.00253 (execute + classify runtime-behavior hallucinations; yuchen814/CodeHalu). TRACE 2601.20103 (contrastive honest-vs-hacking pairs; PatronusAI/trace-dataset — the recall measuring-stick).
- Co-rewarding 2508.00410 (a single self-judge collapses; two disagreeing views stay stable — the cross-view requirement). Skalse 2209.13085 (keyword proxy is gameable by construction).
- ELAI ground truth (DG-2/DG-1/DG-7, grep-confirmed 2026-06-05): score_candidate_for_traps + the trap registry; the guard chain; safedelta_runner.
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect(); VD-5 (recall reported, never claimed complete).

## Open questions / risks
- A strong candidate can SANDBAG the conflict canary if it detects the eval (eval-awareness 2505.23836 AUC 0.83 /
  2510.20487 >80%). Mitigation: make canary tasks indistinguishable from real workload (compose H-47's
  indistinguishability probe); the recall report is honest about the residual (VD-5), never "catches everything".
- Isomorphic rewriting is itself a code transform that can fail to preserve semantics on edge cases; treat an
  IsomorphMismatch as a FLAG requiring the cross-view second opinion, not a unilateral block (VD-4).
- TRACE recall (best model 63% in the paper) sets a realistic ceiling — report ELAI's measured recall, never the
  paper's number as ELAI's (HD-1/GD-5). The frozen corpus is necessary-not-sufficient; the live sweep is Tier-2.

## Next
- H-45 scores promotions worst-case + lineage-diverse (a flagged candidate's worst-case verdict dominates); H-47's
  indistinguishability probe hardens these canaries against sandbagging; D-13's cross-family adversary supplies the
  second view. The keyword registry survives as a cheap advisory first-pass only.

End of H-44.
