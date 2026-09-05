# Reasoning Quality Hooks (E-19)

- **ELAI ids:** E-19 / E-001/DEEP-E001-E19-001..E19-023 / SUMMARY/E-19.F01..F12
- **Spec status (as designed):** DARK (`ELAI_REASONING_QUALITY_PROBE_ENABLE`); original plan PLANNED; H-42 treats E-19 self-certainty + micro-reflection as SHIPPED substrate (RS-11) — see §16
- **Crate / module (spec):** `core/reasoning/{self_certainty,typed_cot,ttc_budget}.rs`; `orchestrator/src/reasoning/{prm_lite,ltot,l6_verifier_diversity}.rs`; gate inserts **before** `reasoning/src/chat_reflect.rs` candidate sampling; L2 site `micro_reflection_emitter.rs`
- **Purpose of this file:** enough architecture to rebuild the six-layer reasoning hooks — pre-L2 confidence gate, PRM-lite, LToT lateral split, Typed-CoT, L6 diversity, difficulty TTC. Not a status ledger.

## 1. What it is

**Reasoning quality hooks** harden ELAI’s **6-layer** reasoning pipeline (L1 deterministic 60-70%, L2 micro-reflection 20-25%, L3 interrogation 5-8%, L4 AB-MCTS 3-5%, L5 cloud diagnostic, L6 cloud generation) against four documented danger zones: confirmatory 7B L2, intrinsic-only L4 value, L4 breadth saturation, same-pipeline L6 verification (`E-plan/E-19_reasoning_quality.md:1-2`, `:10`). The organ is a **gate + scoring + routing** layer on shipped B-39 `ChatReflectPipeline`, B-41 auditor, MistakeBook, `InterrogationTree`, and (when present) B-27 HYBRID L6 — not a new reasoner.

## 2. Why it exists

Literature the sweep flags: First Try Matters / ARLCP — reflections on a 7B are confirmatory and rarely flip wrong→right; inverse-scaling-in-TTC — more CoT is not always better; AB-MCTS paper assumes **external** feedback while ELAI uses noisy `mean_logprob+cosine` (Lanham); LToT — standard ToT saturates breadth and myopically prunes consistent-but-low-utility branches; L6 verified by the same local pipeline that already failed at L1–L4 is the same noisy signal (`E-plan/E-19_reasoning_quality.md:10-18`). If hooks are missing, L2 always fires, L4 is GG-intrinsic, N=3 prunes laterals, L6 rubber-stamps. If faked (prompt “reflect well”), the four danger zones remain. Anti-pattern cited: “just ask the model to think more” (`:245`).

## 3. How it works (runtime)

1. **Who calls it.** `task_runner.rs::reasoning_phase` and `agent_role_dispatcher.rs::dispatch` (`E-plan/E-19_reasoning_quality.md:97-99`). Confidence gate inserts **before** B-39 candidate sampling in `ChatReflectPipeline::run` (`:34-36`, `:91`).
2. **L1 self-certainty (deterministic over N=4 distribution).** Reuse B-39 N=4 sampling. `core::reasoning::self_certainty::compute_score` / `ConfidenceGate` from L1 candidate logits/margins (`:47`, `:70-71`).
3. **Route table (deterministic).** (`:51`, exhaustive E-001/DEEP-E001-E19-004; **corrects** summary “hard task runs L2”):
   - L1 score `≥ L2_CONFIDENCE_THRESHOLD` (initial `0.80`) → skip **both** L2 and L3; emit `L2SkippedHighConfidence` + `L2SelfCertaintyScore`.
   - score in `[0.40, 0.80)` → L2.
   - score `< 0.40` → **L3 directly** (skip L2); emit `L1ToL3DirectRoute`.
4. **No-CoT class override.** FailureTaxonomy categories `implicit-statistical-learning` and `pattern-with-exceptions` → L1→L3, skip L2; emit `L2SkippedNoCotClass` (`:58`). Precedence vs the numeric route: **UNCERTAIN** (§16).
5. **L2 = re-sample with prior, not “reflection.”** `ReSampleWithPrior`: surface MistakeBook as “previous attempt’s **forbidden-diff list**.” Never “reflect on your mistake.” Hard `L2_MAX_TOKENS = 240`; emit `L2TokenCapHit` when the cap binds (`:49`, stub_ban `:104`).
6. **Typed-CoT lite (L2 output).** XGrammar JSON `{rule_applied, types_in, types_out}` (`TypedCotTriple`). Cite 50.3% GSM8K only as **wiring-quality rationale**, not an ELAI math-eval claim. Emit `L2TypedCotEmitted` (`:57`).
7. **L4 PRM-lite.** Auditor in deterministic syntactic-verifier mode returns `{pass/fail, rule_applied}` per AB-MCTS node. Weighted score: `auditor_weight=0.6`, `mean_logprob_weight=0.2`, `cosine_weight=0.2`. Emit `L4PrmLiteScore` per node. After D-2 PPM sidecar ships, **swap** auditor-deterministic for PPM (`:53`, `:90`). Stub_ban: no L4 scoring on `mean_logprob+cosine` alone (`:102`).
8. **L4 LToT split.** Keep `N_MAINLINE = 3` highest-utility + `N_LATERAL = 2` low-utility-but-**logically-consistent** (auditor syntactic-pass) one extra ply only. Emit `L4LateralKept` (`:55`). If 7B+NF4 latency tightens, drop to `N_LATERAL=1` (`:243`).
9. **Difficulty-conditional TTC.** `DifficultyEstimator`: FailureTaxonomy × historical retry count. easy `(depth=2, N=2)`; medium `(depth=3, N=3)`; hard `(depth=3, N=5+lateral)`. Emit `L4TtcBudgetChosen` (`:61`).
10. **L6 verifier diversity.** After L6 generates: (a) **fresh-seeded** L1 (different RNG, same model) + (b) **different** L3 question set (rotated MCQ pool from `rules.db`). **Both must agree** before promotion. Emit `L6VerifierDiversityPass` / `L6VerifierDiversityDisagree`. Stub_ban: no L6 promotion without both (`:59`, `:103`). Latency: default on for HEAVY, off for FOCUSED if budget breaks (`:244`).
11. **Calibration (task 10).** 200-trace holdout at `.elai_cc/eval/reasoning/`; ROC Youden’s-J retunes `L2_CONFIDENCE_THRESHOLD`; persist `[reasoning] l2_threshold` + `calibrated_at` (`:63`).
12. **Task 11 CONDITIONAL — not committed wiring.** L4 `recombine_segments` (SE-Agent operator only, **not** its LLM-judge reward): cloud-lane + `difficulty=hard` only; keep a recombined candidate only if it **strictly** beats every parent PRM-lite score; drop if no lift vs BestOfN on ≥30-task ablation. **Out of** the committed `WIRING_CONTRACT` (`:220-237`).

The model may generate L1/L2/L4/L6 **candidates**. It may **not** decide the confidence gate, L4 weights, lateral keep, or L6 promotion.

## 4. Types, enums, objects

Copy from `E-plan/E-19_reasoning_quality.md:69-89`. Do not invent fields.

```
ConfidenceGate + compute_score                    // core::reasoning::self_certainty
ReSampleWithPrior                                 // replaces L2 "reflection" framing
l3_direct_route                                   // orchestrator::reasoning
AuditorVerifierScore { pass/fail, rule_applied }  // prm_lite
lateral_branch                                    // ltot
TypedCotTriple { rule_applied, types_in, types_out }
l6_verifier_diversity
DifficultyEstimator + TtcBudget
  easy:    depth=2, N=2
  medium:  depth=3, N=3
  hard:    depth=3, N=5+lateral

L2_CONFIDENCE_THRESHOLD = 0.80                    // initial; ROC-tuned
L2_MAX_TOKENS           = 240
N_MAINLINE              = 3
N_LATERAL               = 2
weights                 = 0.6 / 0.2 / 0.2

SsbActionType:
  L2SkippedHighConfidence, L2SelfCertaintyScore, L2TokenCapHit,
  L1ToL3DirectRoute, L4PrmLiteScore, L4LateralKept,
  L2TypedCotEmitted, L6VerifierDiversityPass, L6VerifierDiversityDisagree,
  L4TtcBudgetChosen, L2SkippedNoCotClass
  // Task 11 only: l4_recombination_applied
```

Six layers L1–L6 as named in `:10`. Prevalence percentages are **rationale**, not implementation results (exhaustive E-001/DEEP-E001-E19-002).

## 5. Control flow / state machine

**Owner:** orchestrator reasoning_phase wrapping B-39/B-41 surfaces.

```
L1 N=4 + compute_score
  if no-CoT FailureTaxonomy class -> L3 (skip L2)     // override; precedence UNCERTAIN
  else if score >= 0.80 -> skip L2 and L3
  else if score in [0.40, 0.80) -> L2 ReSampleWithPrior (cap 240, Typed-CoT)
  else score < 0.40 -> L3 InterrogationTree directly
L4 AB-MCTS (if escalated): PRM-lite weights + LToT 3+2; TTC from DifficultyEstimator
L6 (HYBRID/cloud): generate then fresh-seed L1 ∧ rotated L3; disagree -> no promotion
```

**Forced skips:** unconditional L2 after task 1 is banned (`:101`). **Forced reject:** L6 without both diversity checks (`:103`).

**Neighbor, do not author:** J-85 overthinking guard (chain **DRIFT**). E-19 gates **when to spend L2 tokens**, it does not detect answer oscillation.

## 6. Grounding, receipts, gates

**Receipts:** SSB rows listed in §4; auditor syntactic pass/fail; MistakeBook query; calibration.toml. PRM-lite is **syntactic** (compile + rules), not semantic step-validity (`:241-242`).

**Never allowed as L4 value / L6 verify:** `mean_logprob+cosine` alone; same-seed same-pipeline L6 self-check; SE-Agent LLM-judge reward (`:227-228`); “reflect on your mistake” as evidence of correction (`:104`).

**D-2 upgrade:** when PPM sidecar lands, swap auditor-deterministic for PPM in the same `AuditorVerifierScore` slot (`:53`, `:170`).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| L2 confidence skip | `≥ 0.80` initial | per L1 distribution |
| L2 band | `[0.40, 0.80)` | per L1 |
| L3-direct | `< 0.40` | per L1 |
| L2 token cap | 240 | per L2 call |
| L4 weights | 0.6 / 0.2 / 0.2 | per node |
| LToT | N=3 mainline + 2 lateral × 1 extra ply | per expansion |
| TTC easy/med/hard | (2,2) / (3,3) / (3, 5+lateral) | per task |
| L6 diversity | on HEAVY default; FOCUSED may disable | config |
| calibration | 200 traces; Youden-J | holdout |
| L2 retire probe | flip wrong→right `< 2%` → retire L2 | post-ship (`:254`) |
| autonomous full-task | wait L6 diversity pass-rate `≥ 90%` holdout | (`:255`) |

H-42 later **scales** this reflection budget by serial depth (RS-11); E-19 defines the **base**.

## 8. Roles and neighbors

**Consumes:** `ChatReflectPipeline::run` (B-39), `emit_micro_reflection`, `mistake_book::query_by_baby_step`, `InterrogationTree`, B-41 `dispatch_audit` as PRM-lite, `failure_taxonomy::classify` (`:90-96`). **Produces:** symbols in §4. **Frontend:** `ReasoningDrawer.svelte` (`:106`).

**Composes:** D-2 supersedes PRM-lite; D-13 cross-family is the other L6 axis; D-3 consensus consumes PRM-lite + L6 verdicts (`:250-252`). E-18 XGrammar / E-35 structured-output are Typed-CoT substrate. E-46 `process_reward_score` is a **downstream consumer** of a scalar, not this organ’s value function.

**Must not fork:** always-reflect; intrinsic-only AB-MCTS; importing SE-Agent into the DGM loop (`:221-222`).

## 9. Config / flags / defaults

- Feature-map: **DARK** `ELAI_REASONING_QUALITY_PROBE_ENABLE`; “zero production callers” (`ELAI-FEATURE-MAP.md:615-618`).
- `[reasoning] l2_threshold` + `calibrated_at` after task 10 (`E-plan/E-19_reasoning_quality.md:63`).
- `[reasoning] l6_verifier_diversity_enabled` — default on HEAVY, off FOCUSED if latency breaks (`:244`).
- Threshold 0.80 is **initial**; ROC may move it (e.g. 0.65) (`:240`). Compile-time vs config override after tune: **UNCERTAIN** (§16).
- Decision doc: `workflow/reasoning/E-19-decisions.md` plus D-2 upgrade path (`:200`, `:170`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `core/src/reasoning/self_certainty.rs` | `E-plan/E-19_reasoning_quality.md:187` |
| `typed_cot.rs` / `ttc_budget.rs` | `:188-189` |
| `orchestrator/src/reasoning/prm_lite.rs` | `:190` |
| `ltot.rs` / `l6_verifier_diversity.rs` | `:191-192` |
| MODIFY `reasoning/src/chat_reflect.rs` (not `orchestrator/.../chat_reflect.rs`) | `:193`; exhaustive DEL-015 |
| MODIFY `micro_reflection_emitter.rs`, `task_runner.rs`, `ssb.rs` | `:194-196` |
| `ReasoningDrawer.svelte` | `:197` |
| `.elai_cc/eval/reasoning/{traces,calibration}.toml` | `:198-199` |
| tests listed `:175-183` | nine files + contracts round-trip |

Summary path `orchestrator/src/chat_reflect.rs` is **wrong**; original is `crates/reasoning/src/chat_reflect.rs` (E-001/DEEP-E001-DEL-015). Rebuild the original.

## 11. Journal / SSB events

Eleven committed variants (`:79-89`). Summary names `prm_lite_score` / `l2_confidence_gate` are **not** the original symbols (DEL-015). Rebuild the granular list in §4.

Task 11 only: `l4_recombination_applied` `{parents, segment_spans, score_delta}` (`:234`).

## 12. Tests and observables

Named tests (`:175-183`): `self_certainty_gate_contract`, `l2_token_cap`, `l3_direct_route` (no L2 SSB), `prm_lite_node_score` (weights), `ltot_lateral_branch` (3+2 one extra ply), `typed_cot_triples`, `l6_verifier_diversity`, `difficulty_estimator`, `reasoning_gates_contract` SSB round-trip.

**Conflict (do not paper over):** acceptance requires fresh-seed L1 **plus** rotated L3; the named L6 test text says same-seed agrees / different-seed disagrees and **omits** rotated L3 (`:181` vs `:59`; exhaustive CON-003). Rebuild the **acceptance** (both diversity axes).

**Observable effect** (`:107-110`): trivial `add(i32,i32)` local task → `l2_skipped_high_confidence` and `l4_ttc_budget_chosen=easy`; calibration.toml exists; ReasoningDrawer shows new fields.

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- Six-layer pipeline with a **pre-L2** self-certainty gate; L2 not unconditional.
- Route: `≥0.80` skip L2+L3; `[0.40,0.80)` L2; `<0.40` L3-direct; no-CoT classes skip L2.
- L2 is `ReSampleWithPrior` + MistakeBook forbidden-diff; cap 240; never “reflect on your mistake.”
- L4 node score = 0.6 auditor-deterministic + 0.2 logprob + 0.2 cosine; D-2 swap documented.
- LToT keeps 3 mainline + 2 consistent laterals one extra ply.
- Typed-CoT triples via XGrammar.
- L6 promotion requires **fresh-seed L1 and rotated L3** agreement.
- TTC easy/medium/hard mapping above; calibration.toml with threshold + date.
- SE-Agent recombination **not** in default wiring; no LLM-judge reward.

Always-on L2, GG-intrinsic AB-MCTS, or “think more” CoT-by-default is a cousin.

## 14. Non-goals / banned designs

- Unconditional L2 / always-reflect (`:101`, `:171`).
- L4 without PRM-lite after task 4 (`:102`).
- L6 promotion without both diversity checks (`:103`).
- Importing SE-Agent **reward** or placing SE-Agent in the DGM/archive loop (`:221-228`).
- Recombination on the 7B local default path (`:230-231`).
- Citing README-only 80% Claude-4 as a result (`:224`).
- Claiming Typed-CoT’s GSM8K 50.3% as an ELAI measured lift (`:57`).
- Authoring J-85.

## 15. Source citations

Primary: `docs/thefinalglance/E-plan/E-19_reasoning_quality.md` (full file). Index: `E-plan/E-SUMMARY_INDEX.md:174-180` (summary **wrong** on “hard → L2” — use original route). Feature map: `ELAI-FEATURE-MAP.md:610-618`. Exhaustive: `ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md` E-19 ~14181-14230. H-42 consume: `H-REASONING-GROUNDING-MAP.md:59-61`. Papers listed `E-plan/E-19_reasoning_quality.md:202-218`.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Feature-map DARK + “zero production callers” vs H-42 RS-11 “E-19 SHIPPED” self-certainty/micro-reflection (`ELAI-FEATURE-MAP.md:615-616` vs `H-REASONING-GROUNDING-MAP.md:59-61`). Rebuild: E-19 **hooks** (gate/PRM-lite/LToT/L6) may still be DARK while B-39/micro-reflection substrate exists.
- No-CoT class vs numeric confidence precedence (E-001/DEEP-E001-INF-004, AMB-009).
- Whether tuned Youden-J **replaces** the compile-time `0.80` or config overrides it (AMB-008).
- Whether L3-direct later reaches L4 (AMB-009).
- L6 test vs acceptance (CON-003) — acceptance wins.
- `l6_verifier_diversity` default when HEAVY/FOCUSED unset (`:244`).
- NF4 vs W4A16 on L2 accuracy — flagged, not specified (`:246`).
