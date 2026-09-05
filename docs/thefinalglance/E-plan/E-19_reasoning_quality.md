# E-19: Reasoning quality — pre-L2 confidence gate, PRM-lite L4 scoring, LToT mainline/lateral
Objective: harden ELAI's 6-layer reasoning pipeline against the four documented danger zones the sweep flagged: (1) L2 Micro-Reflection on a 7B is the weakest layer per "First Try Matters" + ARLCP — gate it with answer-confidence to avoid confirmatory reflection that rarely flips wrong→right; (2) L4 AB-MCTS uses `mean_logprob+cosine` (GG intrinsic) as its value function while the paper assumes external feedback — augment with a PRM-lite signal; (3) L4 N=3 risks breadth saturation per LToT — add mainline+lateral split keeping low-utility-but-consistent branches one ply longer; (4) L6 Cloud Generation validates via the same local pipeline that failed at L1-L4 — add verifier diversity (fresh-seed L1 + different L3 question set).
Estimated effort: 2-3 sessions, ~10-14 h
Pre-requisites: B-39 SHIPPED (chat REFLECT pipeline; ChatReflectPipeline + ReflectEventSink in reasoning/), B-27 PLANNED (Cloud Mode L5/L6), B-41 SHIPPED (per-level auditor modes), D-2 PLANNED (PPM sidecar — supersedes the PRM-lite quick win once shipped).
Parallel-safe with: E-16, E-17, E-18, E-20.
Source report: workflow/improvement-sweep-2026-05-19/reports/19-reasoning-quality.md

## Context

ELAI's 6-layer reasoning architecture (L1 deterministic 60-70%, L2 micro-reflection 20-25%, L3 interrogation 5-8%, L4 AB-MCTS 3-5%, L5 cloud diagnostic, L6 cloud generation) is well-aligned with the broad literature on extrinsic-verifier reasoning (arxiv:2507.02778 Self-Correction Bench, arxiv:2310.01798 DeepMind "LLMs Cannot Self-Correct Reasoning Yet"). Four specific failure modes are flagged by the sweep:

L2 danger zone (arxiv:2510.08308 First Try Matters, arxiv:2602.12113 ARLCP, arxiv:2410.21333 Mind-Your-Step): on 8 reasoning models / 5 math datasets, reflections are predominantly confirmatory and rarely flip wrong→right; ARLCP measures DeepSeek-R1-Distill-Qwen-7B getting 35.0% length reduction + 2.7% acc gain when reflection is penalized — reflection tokens negatively correlate with accuracy on a 7B. ELAI's L2 fires unconditionally whenever L1 doesn't accept; the inverse-scaling-in-TTC paper (arxiv:2507.14417) is the strongest argument for not always escalating.

L4 value-function risk (arxiv:2503.04412 AB-MCTS, arxiv:2502.18581 self-certainty, arxiv:2510.01500 LToT): AB-MCTS paper uses external feedback signals + Bayesian posteriors. ELAI uses `mean_logprob+cosine` which Lanham (arxiv:2307.13702) measured as a noisy proxy for correctness. PRMBench (acl-long.1230, ACL 2025) shows Qwen2.5-Math-PRM-7B as the SOTA open PRM — D-2 (PPM sidecar) is the long-term fix; E-19 adds a PRM-lite quick-win via the existing `auditor` agent as a deterministic syntactic verifier for L4 node scoring.

L4 breadth saturation (arxiv:2510.01500 LToT): standard ToT suffers breadth saturation + depth myopia; LToT separates utility from logical consistency and keeps "low-utility but consistent" candidates one extra ply. ELAI's N=3 at depth 3 (max 15 branches) risks pruning consistent-but-low-utility candidates too aggressively.

L6 verifier diversity (arxiv:2502.08788 + arxiv:2502.18581): "If the same local pipeline that failed at L1-L4 verifies L6, the verifier signal is the same noisy signal." Fresh-seed L1 + different L3 question set is the cheap diversity addition.

Cited grounding (PDFs in workflow/papers/ where ingested): arxiv:2510.08308, arxiv:2602.12113, arxiv:2410.21333, arxiv:2507.14417, arxiv:2503.04412, arxiv:2502.18581, arxiv:2510.01500, arxiv:2307.13702, arxiv:2507.02778, arxiv:2310.01798, arxiv:2504.04823 (NF4 reasoning-accuracy boundary), arxiv:2510.01069 (Typed CoT).

## Existing state (B/C/D coverage)

- B-39 (DONE s-30584, B-39C s-34648): chat REFLECT pipeline. `crates/reasoning/src/chat_reflect.rs` (~530 LOC) implements ChatReflectPipeline + ChatInferenceClient + ReflectEventSink (avoids circular deps). EXTENSION — E-19 adds a confidence-gate path BEFORE the reflection stage.
- B-39 N=4 candidates per Liu arxiv:2502.06703 inflection (3B-7B class). The candidate-sampling primitive is reusable for E-19's L4 self-certainty BoN selection.
- B-41 (SHIPPED): per-level auditor modes; `auditor_mode + per_level_audit_dispatch + AgentRole +5 + Evidence +5 + AuditTimelineWindow`. EXTENSION — E-19 uses the auditor agent as a deterministic syntactic verifier for L4 node scoring.
- B-27 (PLANNED): Cloud Mode (D-1..D-4) with LOCAL/CLOUD/HYBRID. E-19 augments the HYBRID path's L6 validation with verifier diversity.
- D-2 (PLANNED — Wave 1 D-plan): PPM sidecar at port 8101 with Qwen2.5-Math-PRM-7B. E-19 ships a PRM-lite quick-win using the auditor as a deterministic syntactic verifier BEFORE D-2 ships; D-2 then upgrades the signal to a real PRM.
- D-13 Adversary role (PLANNED): cross-family adversarial probe. E-19's L6 verifier diversity is the local-pipeline-side version; D-13 is the cross-family side. They compose — both should be wired before retiring HumanReview.
- MistakeBook + micro_reflection_emitter exist at `crates/core/src/mistake_book.rs` + `crates/orchestrator/src/micro_reflection_emitter.rs`; the pre-L2 gate consumes the MistakeBook prior already.
- InterrogationTree exists at `crates/core/src/interrogation_tree.rs`; E-19 promotes L3 ahead of L2 when L1 confidence is low (sweep recommendation #8).

## Reads (mandatory before coding)

- workflow/improvement-sweep-2026-05-19/reports/19-reasoning-quality.md (full)
- F:\.ELAI_workflow\.elai_cc\crates\reasoning\src\chat_reflect.rs (existing pipeline; lines 264-559 cover ReflectEventSink trait, ChatReflectConfig, run loop — confidence-gate inserts BEFORE candidate sampling)
- F:\.ELAI_workflow\.elai_cc\crates\reasoning\src\engine.rs (if present; B-1 audit refs emit_reasoning at task_runner.rs:3985-4004 and reasoning/engine.rs:340 for AuditorRubberStampRejected)
- F:\.ELAI_workflow\.elai_cc\crates\core\src\mistake_book.rs (lines 116-427; MistakeBook prior for re-sampling)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\micro_reflection_emitter.rs (lines 137-229; L2 invocation site)
- F:\.ELAI_workflow\.elai_cc\crates\core\src\micro_reflection.rs (lines 253-360; response validation incl. MAX_ROOT_CAUSE_WORDS — extend with confidence-gate predicate)
- F:\.ELAI_workflow\.elai_cc\crates\core\src\interrogation_tree.rs (line 648 serde; L3 dispatch surface)
- F:\.ELAI_workflow\docs\specs\guide\reasoning-architecture-design.txt (6-layer + AB-MCTS spec)
- arxiv:2510.08308 + arxiv:2602.12113 + arxiv:2503.04412 + arxiv:2502.18581 + arxiv:2510.01500 PDFs (cite in tests + decision doc)

## Tasks

1. Add a pre-L2 confidence gate using self-certainty (arxiv:2502.18581). At the L1→L2 boundary, compute a logit-margin / self-certainty score from L1's candidate distribution; skip L2 entirely if score ≥ `L2_CONFIDENCE_THRESHOLD = 0.80` (initial; tune via task 10 holdout calibration). Use the existing N=4 candidate sampling primitive from B-39 to derive the self-certainty distribution. Emit `l2_skipped_high_confidence` and `l2_self_certainty_score` SSB rows. This is the cheapest single change with the strongest grounding (inverse-scaling-in-TTC + First-Try-Matters).

2. Reframe L2 from "reflection" to "re-sample with prior" in the prompt. The MistakeBook (existing) IS the prior — surface it explicitly to the model as "previous attempt's forbidden-diff list" rather than "reflect on your mistake." Cap L2 output tokens hard via `L2_MAX_TOKENS = 240` (proportional to L1 confidence per ARLCP arxiv:2602.12113 35.0% length reduction finding). Emit `l2_token_cap_hit` row when the cap binds.

3. Promote L3 Interrogation Tree ahead of L2 when L1 confidence is low (sweep recommendation #8). The routing rule: L1 confidence ≥ 0.80 → skip both; L1 confidence in [0.40, 0.80) → L2; L1 confidence < 0.40 → L3 directly (extrinsic verification per arxiv:2507.02778 Self-Correction Bench). Add `l1_to_l3_direct_route` SSB row.

4. Add PRM-lite L4 node scoring via the existing `auditor` agent in deterministic syntactic-verifier mode. The auditor returns a pass/fail + rule_applied tuple for each AB-MCTS candidate node; combine with `mean_logprob+cosine` as a weighted score (default `auditor_weight = 0.6, mean_logprob_weight = 0.2, cosine_weight = 0.2`). Cite PRMBench (acl-long.1230). Document the D-2 upgrade path: when PPM sidecar lands, swap the auditor-deterministic signal for the PPM score. Emit `l4_prm_lite_score` row per node.

5. Add LToT mainline/lateral split to L4 expansion policy (arxiv:2510.01500). Within L4: keep `N_MAINLINE = 3` highest-utility candidates + `N_LATERAL = 2` "low-utility but logically consistent" candidates; prune lateral candidates after one extra ply only. Logical consistency = auditor syntactic-pass from task 4. Emit `l4_lateral_kept` row when a low-utility-consistent branch is retained.

6. Implement Typed-CoT lite for L2 output (arxiv:2510.01069). Emit `{rule_applied, types_in, types_out}` triples in L2 output via XGrammar JSON schema (E-18's primitive). Each triple becomes audit input for wiring-contract-validator. Cite "50.3% improvement on GSM8K with same sample budget" but document as a wiring-quality gain on ELAI's stack, not a math-eval claim. Emit `l2_typed_cot_emitted` row.

7. Add verifier diversity for L6 validation. After L6 generates, run: (a) fresh-seeded L1 pass (different RNG seed, same model) + (b) different L3 question set (rotated MCQ pool from `rules.db`). Both must agree before promotion. Cite arxiv:2502.08788 (vanilla MAD underperforms; diversity + calibration matter). Emit `l6_verifier_diversity_pass` / `l6_verifier_diversity_disagree` rows.

8. Add a difficulty estimator + difficulty-conditional TTC budget (arxiv:2502.10954 + arxiv:2408.03314). Signal = FailureTaxonomy category × historical retry count (existing primitives). Easy → smaller depth/N; hard → larger. Default mapping: easy(depth=2, N=2), medium(depth=3, N=3), hard(depth=3, N=5+lateral). Emit `l4_ttc_budget_chosen` row.

9. Add no-CoT routing for known no-CoT-helps task classes (arxiv:2410.21333 — implicit statistical learning, pattern-with-exceptions). Route L1→L3 directly, skip L2 entirely. Class identification via FailureTaxonomy category. Emit `l2_skipped_no_cot_class` row.

10. Build a held-out calibration set at `.elai_cc/eval/reasoning/` with 200 production-trace samples (L1 candidate distribution → labeled L2-needed / L2-not-needed). Tune `L2_CONFIDENCE_THRESHOLD` via ROC; target Youden's-J max. Surface the threshold + calibration date in `[reasoning] l2_threshold = ..., calibrated_at = ...` config.

## WIRING_CONTRACT

produces_symbols:
  - core::reasoning::self_certainty::compute_score (new)
  - core::reasoning::self_certainty::ConfidenceGate (new struct)
  - reasoning::chat_reflect::ReSampleWithPrior (new struct replacing the "reflection" framing in L2)
  - orchestrator::reasoning::l3_direct_route (new fn)
  - orchestrator::reasoning::prm_lite::AuditorVerifierScore (new struct)
  - orchestrator::reasoning::ltot::lateral_branch (new fn)
  - core::reasoning::typed_cot::TypedCotTriple (new struct)
  - orchestrator::reasoning::l6_verifier_diversity (new fn)
  - core::reasoning::ttc_budget::DifficultyEstimator (new struct)
  - SsbActionType::L2SkippedHighConfidence (new)
  - SsbActionType::L2SelfCertaintyScore (new)
  - SsbActionType::L2TokenCapHit (new)
  - SsbActionType::L1ToL3DirectRoute (new)
  - SsbActionType::L4PrmLiteScore (new)
  - SsbActionType::L4LateralKept (new)
  - SsbActionType::L2TypedCotEmitted (new)
  - SsbActionType::L6VerifierDiversityPass (new)
  - SsbActionType::L6VerifierDiversityDisagree (new)
  - SsbActionType::L4TtcBudgetChosen (new)
  - SsbActionType::L2SkippedNoCotClass (new)
consumes_symbols:
  - reasoning::chat_reflect::ChatReflectPipeline::run (B-39 entry; gate inserts before this)
  - orchestrator::micro_reflection_emitter::emit_micro_reflection (existing L2 invocation)
  - core::mistake_book::query_by_baby_step (existing prior)
  - core::interrogation_tree::InterrogationTree (existing L3 surface)
  - orchestrator::auditor::dispatch_audit (B-41 per-level auditor; reused as PRM-lite verifier)
  - failure_taxonomy::classify (existing 9-category taxonomy; difficulty estimator input)
reachable_from:
  - crates/orchestrator/src/task_runner.rs::reasoning_phase
  - crates/orchestrator/src/agent_role_dispatcher.rs::dispatch (per-role reasoning entry)
stub_ban:
  - no unconditional L2 invocation after task 1 (gate enforced)
  - no L4 AB-MCTS scoring without PRM-lite signal after task 4 (mean_logprob+cosine alone is insufficient per arxiv:2503.04412 paper rationale)
  - no L6 promotion without fresh-seed L1 + rotated L3 agreement after task 7
  - no "reflect on your mistake" framing in L2 prompts (replaced by re-sample-with-prior per ARLCP risk)
  - CLAUDE.md anti-unwrap: no `.unwrap()` in reasoning paths
frontend_pair: crates/dashboard-v2/src/lib/components/ReasoningDrawer.svelte
observable_effect:
  trigger: "elai /task --mode local 'implement a pure-Rust add(a: i32, b: i32) -> i32 function in math.rs with doc-comment and unit test'"
  expect_ssb: "l2_skipped_high_confidence row present (trivial task → high L1 confidence); l4_ttc_budget_chosen=easy"
  expect_artifact: ".elai_cc/eval/reasoning/calibration.toml exists with l2_threshold and calibrated_at populated; ReasoningDrawer shows new fields"

## DETERMINISTIC_CHECK

```bash
# 1. Confidence gate present
rg "L2_CONFIDENCE_THRESHOLD|ConfidenceGate|compute_self_certainty" .elai_cc/crates/reasoning .elai_cc/crates/core --type rust
# Expected: ≥3 hits.

# 2. L2 token cap enforced
rg "L2_MAX_TOKENS|l2_token_cap_hit" .elai_cc/crates/reasoning .elai_cc/crates/orchestrator --type rust
# Expected: const + SSB variant + at least one dispatch reference.

# 3. L3 direct route from L1 low-confidence
rg "l1_to_l3_direct_route|low_confidence_l3" .elai_cc/crates/orchestrator --type rust
# Expected: ≥2 hits.

# 4. PRM-lite L4 wiring
rg "AuditorVerifierScore|l4_prm_lite_score|prm_lite" .elai_cc/crates/orchestrator --type rust
# Expected: ≥3 hits.

# 5. LToT lateral split
rg "N_MAINLINE|N_LATERAL|lateral_branch|l4_lateral_kept" .elai_cc/crates/orchestrator --type rust
# Expected: ≥4 hits (const + fn + SSB + dispatch).

# 6. Typed CoT triples emitted
rg "TypedCotTriple|l2_typed_cot_emitted" .elai_cc/crates/core .elai_cc/crates/reasoning --type rust
# Expected: ≥2 hits.

# 7. L6 verifier diversity
rg "l6_verifier_diversity|fresh_seed_l1|rotated_l3" .elai_cc/crates/orchestrator --type rust
# Expected: ≥2 hits.

# 8. Difficulty-conditional TTC budget
rg "DifficultyEstimator|l4_ttc_budget_chosen|TtcBudget" .elai_cc/crates --type rust
# Expected: ≥3 hits.

# 9. Calibration set + threshold
ls .elai_cc/eval/reasoning/calibration.toml .elai_cc/eval/reasoning/traces.toml
# Expected: both exist.

# 10. Smoke: trivial task skips L2
elai /task --mode local "implement a pure-Rust add function with doc and unit test"
sqlite3 .elai_cc/.elai_cc/ssb.db "SELECT action_type, COUNT(*) FROM ssb_events WHERE action_type IN ('l2_skipped_high_confidence','l4_ttc_budget_chosen') GROUP BY action_type;"
# Expected: l2_skipped_high_confidence >= 1; l4_ttc_budget_chosen >= 1 with payload mode=easy.
```

## ACCEPTANCE

- [ ] L2 skipped when L1 self-certainty ≥ 0.80 (threshold tuned on 200-trace holdout in task 10)
- [ ] L2 prompt reframed as "re-sample with prior" (MistakeBook explicit); never says "reflect on your mistake"
- [ ] L2 token cap binds at 240 tokens; cap_hit row emitted when bound
- [ ] L3 fires directly from L1 when confidence < 0.40 (skipping L2 entirely)
- [ ] L4 node scoring uses PRM-lite (auditor-deterministic) + weighted with mean_logprob + cosine
- [ ] L4 keeps N_LATERAL=2 logically-consistent low-utility branches one extra ply
- [ ] L2 emits Typed-CoT triples via XGrammar JSON schema (E-18 primitive); auditor sees `{rule_applied, types_in, types_out}`
- [ ] L6 validation requires fresh-seed L1 + rotated-L3 agreement (verifier diversity)
- [ ] Difficulty estimator routes easy/medium/hard tasks to (N=2,depth=2)/(N=3,depth=3)/(N=5+lateral,depth=3)
- [ ] No-CoT task classes (FailureTaxonomy categories: implicit-statistical-learning, pattern-with-exceptions) skip L2 directly
- [ ] Calibration set written; threshold value documented with calibrated_at timestamp
- [ ] D-2 (PPM sidecar) upgrade path documented in decision doc — when D-2 ships, swap auditor-deterministic for PPM score in PRM-lite
- [ ] CLAUDE.md anti-stale: never re-introduce "always-reflect" pattern (inverse-scaling-in-TTC anti-pattern)

## Tests to add

- `.elai_cc/crates/reasoning/tests/self_certainty_gate_contract.rs`: synthesize candidate distributions at varying margins; assert gate fires above threshold, fires below threshold.
- `.elai_cc/crates/orchestrator/tests/l2_token_cap.rs`: feed a long-output mock; assert cap binds + SSB row.
- `.elai_cc/crates/orchestrator/tests/l3_direct_route.rs`: low-confidence L1 candidate → route hits L3 first; assert no L2 SSB row.
- `.elai_cc/crates/orchestrator/tests/prm_lite_node_score.rs`: synthetic 3-candidate AB-MCTS expansion; assert weighted score combines auditor + mean_logprob + cosine with documented weights.
- `.elai_cc/crates/orchestrator/tests/ltot_lateral_branch.rs`: 5-candidate expansion (3 high-utility + 2 consistent-low-utility); assert lateral branches retained one extra ply.
- `.elai_cc/crates/reasoning/tests/typed_cot_triples.rs`: assert XGrammar-constrained JSON output matches `TypedCotTriple` schema.
- `.elai_cc/crates/orchestrator/tests/l6_verifier_diversity.rs`: same-seed re-run agrees → pass; different-seed disagrees → disagree row.
- `.elai_cc/crates/orchestrator/tests/difficulty_estimator.rs`: FailureTaxonomy easy/medium/hard inputs → correct (N, depth) outputs.
- `.elai_cc/crates/contracts/tests/reasoning_gates_contract.rs`: round-trip all new SSB variants.

## Files touched

- CREATE: `.elai_cc/crates/core/src/reasoning/self_certainty.rs` (~160 LOC)
- CREATE: `.elai_cc/crates/core/src/reasoning/typed_cot.rs` (~100 LOC)
- CREATE: `.elai_cc/crates/core/src/reasoning/ttc_budget.rs` (~140 LOC)
- CREATE: `.elai_cc/crates/orchestrator/src/reasoning/prm_lite.rs` (~180 LOC)
- CREATE: `.elai_cc/crates/orchestrator/src/reasoning/ltot.rs` (~200 LOC)
- CREATE: `.elai_cc/crates/orchestrator/src/reasoning/l6_verifier_diversity.rs` (~160 LOC)
- MODIFY: `.elai_cc/crates/reasoning/src/chat_reflect.rs` (+~60 LOC re-sample-with-prior framing)
- MODIFY: `.elai_cc/crates/orchestrator/src/micro_reflection_emitter.rs` (+~50 LOC confidence-gate dispatch)
- MODIFY: `.elai_cc/crates/orchestrator/src/task_runner.rs` (+~40 LOC L1→L3 direct route)
- MODIFY: `.elai_cc/crates/orchestrator/src/ssb.rs` (+11 SsbActionType variants)
- MODIFY: `.elai_cc/crates/dashboard-v2/src/lib/components/ReasoningDrawer.svelte` (+~80 LOC fields)
- CREATE: `.elai_cc/eval/reasoning/traces.toml` (200-trace holdout)
- CREATE: `.elai_cc/eval/reasoning/calibration.toml` (threshold + calibrated_at)
- CREATE: `workflow/reasoning/E-19-decisions.md`

## Citations / Grounding

- arXiv:2510.08308 (cited in source) — First Try Matters: reflections confirmatory, rarely flip wrong→right
- arXiv:2602.12113 (cited in source) — ARLCP ICLR 2026: DeepSeek-R1-Distill-Qwen-7B reflection harms accuracy
- arXiv:2410.21333 (cited in source) — Mind-Your-Step: CoT reduces accuracy up to 36.3% on implicit-statistical/pattern-with-exceptions
- arXiv:2507.14417 (cited in source) — Inverse Scaling in Test-Time Compute TMLR 2025: 4 task categories degrade with more CoT
- arXiv:2503.04412 (cited in source) — AB-MCTS (Misaki/Sakana 2025): Bayesian posterior decides wider vs deeper
- arXiv:2502.18581 (cited in source) — Self-Certainty: replaces external verifier with logit-derived score for BoN
- arXiv:2510.01500 (cited in source) — Lateral ToT (LToT): mainline+lateral split, keep consistent-low-utility candidates
- arXiv:2307.13702 (cited in source) — Lanham et al CoT Faithfulness Anthropic 2023: logit signal not clean proxy
- arXiv:2507.02778 (cited in source) — Self-Correction Bench: 64.5% blind-spot rate; LLMs correct external errors, not own
- arXiv:2310.01798 (cited in source) — DeepMind: intrinsic self-correct degrades reasoning unless external feedback
- arXiv:2502.08788 (cited in source) — Vanilla MAD underperforms; diversity + calibrated confidence required
- arXiv:2504.04823 (cited in source) — Quantization Hurts Reasoning: NF4 borderline; W8A8/W4A16 lossless
- arXiv:2510.01069 (cited in source) — Typed CoT / TFCs: 50.3% improvement on GSM8K with same sample budget
- aclanthology:2025.acl-long.1230 (cited in source) — PRMBench ACL 2025: Qwen2.5-Math-PRM-7B SOTA open PRM
- arXiv:2508.02085 (added 2026-06-05, re-scope — NOT in source report; ingestor-verified at workflow/papers/se-agent-interface.md) — SE-Agent NeurIPS 2025 (github.com/JARVIS-Xs/SE-Agent, 274★ MIT): per-task trajectory Revision/Recombination/Refinement; SWE-bench Verified pass@1 61.2% Claude-3.7-Sonnet (+51% vs SWE-Agent, ~+30% vs SWE-Search/MCTS), 38.8% Qwen-2.5-72b; selection reward is hybrid AutoEval+λ·LLM-judge — do NOT import (single-judge collapse per the self-verification dossier A.0)

## SE-Agent recombination operator (arXiv:2508.02085) — L4 prior-art extension [CONDITIONAL, cloud-lane]

Added 2026-06-05 as a deliberate re-scope of the self-verification dossier's rejection (workflow/reasoning/ELAI-SELF-VERIFICATION-RESEARCH-DESIGN-2026-06-05.md — Dossier 1 REJECTED SE-Agent as "out-of-lineage: trajectory-optimization, NOT self-code/archive editing"). That rejection is CORRECT for the DGM self-improvement loop: SE-Agent never mutates the agent/scaffold, keeps no archive, and is stateless across tasks (paper-ingestor-confirmed, workflow/papers/se-agent-interface.md). It is NOT a self-improvement substrate. It IS a per-task multi-sample SEARCH strategy — which is exactly what L4 is. So it belongs on the L4 shelf, not in the DGM loop. This section relocates it; it does NOT reopen the DGM rejection.

What it is (ingestor-verified): SE-Agent (NeurIPS 2025) evolves a POPULATION OF TRAJECTORIES for ONE task across K cycles of Revision (self-reflect -> re-execute) / Recombination (crossover-merge segments + transfer sub-steps + restructure) / Refinement (elite+diversity selection), scores them, keeps elites, emits the winner, discards the pool. Measured SWE-bench Verified pass@1: Claude-3.7-Sonnet 61.2% (+51% rel vs SWE-Agent), GPT-4o 40.4%, Qwen-2.5-72b 38.8%, Llama-3.1-70b 32.6%; ~+30% vs SWE-Search (MCTS — same family as L4 AB-MCTS). (README-only 80% Claude-4-Sonnet is NOT in the manuscript — never cite as a result.)

The net-new mechanism (the only liftable part): RECOMBINATION — crossover that merges high-performing SEGMENTS across different candidate trajectories. ELAI's L4 today only SELECTS: run_best_of_n (crates/orchestrator/src/structural_retry.rs:277) scores each candidate (+2 compile, +2 critic-pass, +1 lines) and returns the max; AB-MCTS (ab_mcts_max_branches at crates/core/src/vram_tier.rs:72; the crates/orchestrator/src/reasoning/prm_lite.rs scorer) expands+scores branches. Neither merges segments across branches — a 2026-06-05 elai.map grep found zero recombination/crossover symbol. SE-Agent's Recombination is the missing operator; its epsilon-convergence stop (halt when max(Reward) gain < epsilon for consecutive cycles) is a free early-stop add-on for the Task-5 LToT/AB-MCTS expansion.

HARD CAUTION 1 — lift the operator, NOT the reward. SE-Agent's selection reward folds in an LLM-judge (Reward = w1*TaskCompletion + w2*ReasoningQuality + w3*Efficiency; TaskCompletion = AutoEval + lambda*ExpertEval; reward-hacking via ExpertEval unaddressed by the paper). That is the single-judge proxy the dossier's A.0 meta-finding says DIVERGES (Co-rewarding 40-pt self-judge collapse; Skalse unhackability arXiv:2209.13085). DO NOT import SE-Agent's reward. Score recombined candidates with E-19 Task 4's PRM-lite auditor-deterministic signal — or, once it exists, the dossier GAP-1 execution-grounded fitness (CodeRM arXiv:2501.01054) / F-4 stdout_hash. The operator is only safe BEHIND an off-agent score.

HARD CAUTION 2 — cloud-lane, hard-tasks-only, never default. Gains are measured on cloud flagships + 70B-class; the abliterated 7B local lane is unproven and per-task sample-multiplication is worst there. Recombination multiplies OUTPUT tokens — in direct tension with H-plan Track 1's "provably cheaper on the same model." Gate to difficulty=hard (Task 8's hard branch, already N=5+lateral) on the cloud/architect route (H-7); never the local default path.

Composition / sequencing: meaningful ONLY once an off-agent fitness exists (dossier GAP-1, RUNPOD-PENDING Tier-2 bench). Until then it runs on the deterministic fixture harness against the Task-4 auditor signal — same constraint as SAFEDELTA C1 / F-2b. Sequence AFTER E-19 main AND after GAP-1 fitness; a follow-on, NOT committed E-19 scope (kept OUT of the committed WIRING_CONTRACT above for that reason).

Task 11 (CONDITIONAL — not in the committed 10-task scope): add an L4 Recombination operator `orchestrator::reasoning::ltot::recombine_segments`. Given the scored AB-MCTS/BestOfN candidate pool (>=2 candidates carrying a Task-4 PRM-lite score), splice the highest-scoring contiguous segments across candidates into <=2 recombined candidates, re-score with the SAME Task-4 auditor-deterministic signal, and KEEP a recombined candidate only if it STRICTLY beats every parent score (no-regression, mirrors run_best_of_n's max-preserving select at structural_retry.rs:277). Cloud-lane + difficulty=hard gate. Emit `l4_recombination_applied` (parents, segment_spans, score_delta) SSB row.
  observable_effect: a hard cloud task where two candidates each pass DISJOINT sub-checks -> a recombined candidate passing BOTH lands one l4_recombination_applied row with score strictly above both parents; a case where no recombination beats its parents emits NO row (the no-regression guard is the off-agent artifact).
  DETERMINISTIC_CHECK: `rg "recombine_segments|l4_recombination_applied" .elai_cc/crates/orchestrator --type rust` -> >=2 hits.
  ACCEPTANCE (gate before any default-on): recombination must beat plain BestOfN selection on a >=30-task hard-split gate-ON-vs-gate-OFF ablation (GD-3 framing); if the ablation shows no lift over selection, DROP the operator — selection is cheaper and recombination's only justification is measured lift.

## Open questions / risks

- L2 threshold 0.80 is initial; task 10 calibration on 200-trace holdout will tune. If Youden's-J max sits at e.g. 0.65, L2 fires more often — that's fine if confirmatory-reflection rate is also measured (does L2 flip wrong→right ever, per report Q2). If not, retire L2 entirely for non-RESEARCH task classes.
- PRM-lite via auditor is a stopgap. The auditor is deterministic-syntactic; it cannot score "is the reasoning step semantically valid" — only "does the resulting code compile + pass syntactic rules." When D-2 (PPM sidecar) ships, swap. Document D-2 dependency in decision doc.
- LToT N_LATERAL=2 may double L4 compute cost; if 7B+NF4 latency budget tightens, drop to N_LATERAL=1.
- L6 verifier diversity adds latency: fresh-seed L1 + rotated-L3 each cost a sidecar call. For HEAVY tasks the cost is amortized; for FOCUSED it's marginal. If latency budget breaks, gate verifier diversity behind a `[reasoning] l6_verifier_diversity_enabled = true` config (default on for HEAVY, off for FOCUSED).
- Anti-pattern (cited): "Just ask the model to think more" — never re-introduce CoT-by-default; the inverse-scaling paper is unambiguous.
- NF4 quantization on the reasoning steps is in the "accuracy at risk" zone (arxiv:2504.04823). E-19 doesn't change quant config but the open Q5 (NF4 vs W4A16 boundary on L2 reasoning) is flagged for measurement.
- Difficulty estimator on FailureTaxonomy × retry count is heuristic; if signal is too weak, augment with cosine-similarity-to-historical-hard-tasks from L3 MCQ pool.

## Next

- D-2 (PPM sidecar) supersedes the PRM-lite auditor signal once shipped; sequence D-2 after E-19 main.
- D-13 Adversary role provides cross-family verifier; composes with E-19's L6 verifier diversity to add a second axis of independence.
- D-3 (VerificationResult aggregator) consumes E-19's PRM-lite scores + L6 diversity verdicts as inputs to the multi-verifier consensus rule.
- Open ablation per report Q2: does L2 reflection on a 7B EVER flip wrong→right in ELAI logs? Instrument + measure post-ship; if rate < 2%, retire L2 entirely.
- Verifier-diversity at L6 is a precondition for autonomous-mode confidence; gate "autonomous full-task" feature behind L6 diversity pass-rate measured at ≥ 90% on holdout.
- SE-Agent (arXiv:2508.02085) L4 recombination operator is a CONDITIONAL prior-art extension (see the dedicated section above) — a per-task SEARCH refinement, NOT a DGM-loop item; gated on the dossier GAP-1 off-agent fitness + a gate-ON-vs-OFF ablation proving recombination beats plain BestOfN selection; cloud/hard-lane only, never the 7B local default.
