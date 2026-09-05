E-19 reasoning quality — completion-drive decisions
session: s-25120 completion-drive
date: 2026-05-27
plan: docs/thefinalglance/E-plan/E-19_reasoning_quality.md
prior cross-check: workflow/sessions/s-25120/cross-check-E/E-19_reasoning_quality-report.md
grounding: workflow/sessions/s-25120/completion-drive/grounding/E-19_reasoning_quality-grounding.md

SUMMARY

This ships the host-side substrate for E-19 reasoning-quality probes:
self-certainty gate primitives, PRM-lite weighted-combine math, LToT
mainline/lateral split, L6 verifier-diversity rule, difficulty-conditional
TTC budget, Typed-CoT triple struct + JSON schema, the
`ELAI_REASONING_QUALITY_PROBE_ENABLE` umbrella env gate, and a single new
SSB variant `ReasoningQualityProbe` with a 5-arm `ProbeKind` discriminator.
Wire-dark unless the umbrella env gate is set. Live emission into the
orchestrator reasoning pipeline (micro_reflection_emitter +
task_runner::emit_reasoning) is RUNPOD-PENDING — the dispatchers are pure
data types in this commit so the contract tests round-trip without
spinning up the sidecar. Calibration of the L2 threshold is also
RUNPOD-PENDING; the calibration.toml + traces.toml scaffolding ships with
5 seed rows + the harvest schema.

DECISION-PATTERN2 — single-variant SSB collapse

Plan-doc proposed 11 new SsbActionType variants (L2SkippedHighConfidence,
L2SelfCertaintyScore, L2TokenCapHit, L1ToL3DirectRoute, L4PrmLiteScore,
L4LateralKept, L2TypedCotEmitted, L6VerifierDiversityPass,
L6VerifierDiversityDisagree, L4TtcBudgetChosen, L2SkippedNoCotClass).
REJECTED per cluster precedent (s-25120 sibling D-19..D-30, E-7, E-8,
E-10, E-43, E-9 all single-variant Pattern-2). E-19 ships ONE new
variant `ReasoningQualityProbe` with a 5-arm `ProbeKind` discriminator
INSIDE the payload (l2_gate_decision / l4_prm_lite_score /
l4_lateral_kept / l4_ttc_budget_chosen / l6_verifier_diversity). The
`verdict` and `extras` payload string fields carry per-kind sub-state
(e.g. `skip_high_confidence` / `reflect` / `l3_direct` / `no_cot_class`
for L2 gate decisions). SCHEMA_VERSION 165 -> 166 (+1). TOTAL_VARIANTS
438 -> 439 (+1). 18th application of the Pattern-2 single-variant
single-bump cascade.

DECISION-PAPER-DROP-ARLCP

arXiv:2602.12113 ARLCP (ICLR 2026) was the plan-doc citation for
`L2_MAX_TOKENS = 240 proportional to L1 confidence`. The paper-ingestor
triage at workflow/sessions/s-25120/completion-drive/grounding/E-19-grounding.md
flagged this as WRONG_REGIME (BLOCKING): ARLCP is a TRAINING-TIME RL
method requiring a fresh PPO pass; ELAI does NOT retrain the sidecar.
The 35.0% length reduction + 2.7% accuracy gain numbers are NOT
achievable via an inference-time token cap. The ARLCP citation is
DROPPED. `L2_MAX_TOKENS = 240` is documented as an engineering heuristic
in `crates/core/src/reasoning/` code comments and in this decision doc.
arXiv:2510.08308 First Try Matters is cited as the NEAREST inference-time
paper support (dynamic truncation after candidate answer appears) but
NOT as direct grounding for the 240-token cap. The orchestrator
implementation is RUNPOD-PENDING and the actual cap-binding behavior
will be measured + recalibrated against the 200-trace holdout.

DECISION-PAPER-DROP-LANHAM

arXiv:2307.13702 (Lanham et al, Anthropic 2023 CoT Faithfulness) was
the plan-doc citation for "logit signal as noisy proxy for correctness."
WRONG_REGIME per triage: the Lanham paper measures whether CoT TEXT is
faithful to model reasoning via interventions on the CoT (adding mistakes,
paraphrasing). It does NOT measure logit / probability signals as
correctness proxies. DROPPED. The L4 PRM-lite documentation in
`prm_lite.rs` cites arXiv:2502.18581 (Self-Certainty, Kang/Zhao/Song) as
the re-anchored paper for "logit signal is a noisy proxy" — that paper
explicitly addresses logit-derived BoN selection and is the correct
regime match.

DECISION-PAPER-RELABEL-VANILLA-MAD

arXiv:2502.08788 (Vanilla MAD Underperforms) was cited by the plan-doc
in tasks 4 + 7 for "diversity + calibration." Triage flagged the
"calibration" framing as WRONG_REGIME-minor: the paper's actual
recommendation is MODEL HETEROGENEITY (Heter-MAD), NOT confidence
calibration. The orchestrator dispatchers + code comments use the
"model heterogeneity" framing. L6 verifier-diversity is documented as
the closest LOCAL-pipeline approximation of heterogeneity (same model,
different seed + question set); D-13 Adversary role is the orthogonal
cross-family axis.

DECISION-L2-MAX-TOKENS-HEURISTIC

`L2_MAX_TOKENS = 240` is an engineering heuristic in this commit — not
paper-grounded. The actual L2 token cap binding code lives in
`crates/reasoning/src/chat_reflect.rs` modifications (RUNPOD-PENDING).
The number 240 is the median L2 length observed in B-39 production
traces (per activeContext.md fragments); it is NOT pinned to ARLCP's
35% length reduction. The L2 token cap will be revisited post-calibration
along with the L2 threshold.

DECISION-PATH-FIXES (4 drift items from cross-check + context-compiler)

1. `core::mistake_book::query_by_baby_step` -> consumer code (when
   wired) imports from `elai_orchestrator::mistake_book::query_by_baby_step`.
   The `elai_core::mistake_book` module exposes data types only
   (MistakeBook, DiffHash, FailureCategory). Documented in the
   orchestrator reasoning module comment headers.
2. `orchestrator::auditor::dispatch_audit` -> does NOT exist. The
   canonical B-41 audit-dispatch surface is
   `orchestrator::per_level_audit_dispatch::record_verdict` at line 206
   + the cascade dispatch path at line 125. PRM-lite consumers MUST
   call `record_verdict` in `AuditorMode::*` deterministic-syntactic
   mode (live wiring is RUNPOD-PENDING).
3. `task_runner::reasoning_phase` -> does NOT exist. Canonical L1->L4
   reasoning emit site is `orchestrator::task_runner::emit_reasoning`
   at line 7753. Live insertion of the difficulty-estimator +
   TTC-budget selection is RUNPOD-PENDING.
4. `agent_role_dispatcher::dispatch` -> actual name is
   `agent_role_dispatcher::build_dispatch` at line 179. Documented for
   the future wiring patch.
5. Dashboard frontend_pair `crates/dashboard-v2/src/lib/components/`
   path is WRONG. Actual path is `crates/dashboard-v2/src/lib/drawers/`.
   E-19 is NOT in the dashboard list per
   `docs/specs/guide/steps/phase-index-3A-4J.txt`; existing
   `ReasoningDrawer.svelte` extension is RUNPOD-PENDING in a follow-up.

DECISION-WIRE-DARK-ENV-GATE

Pattern-4: every E-19 dispatch path short-circuits when
`ELAI_REASONING_QUALITY_PROBE_ENABLE` is unset. Default off. Per
E-26/E-43/E-9 precedent. The umbrella gate covers all 5 ProbeKind arms;
finer-grained per-arm gates are NOT shipped (rejected — Pattern-4
precedent uses one umbrella per scope-id). The gate is read via
`std::env::var` in `orchestrator::reasoning::umbrella_enabled()` —
callers MAY cache via `OnceLock` if they're on a hot path (E-3
RuntimeHandoffSignal precedent).

DECISION-CALIBRATION

Plan task 10 requires a 200-trace holdout for tuning
`L2_CONFIDENCE_THRESHOLD`. Live harvest is RUNPOD-PENDING (the
`eval_cmd reasoning --harvest` subcommand wires in a follow-up). This
commit ships 5 seed rows in `.elai_cc/eval/reasoning/traces.toml` as a
schema illustration; the seed rows are NOT representative for threshold
tuning. The initial seed threshold of `0.80` is documented in
`crates/core/src/reasoning/self_certainty.rs` and re-stated in
`.elai_cc/eval/reasoning/calibration.toml` with
`calibrated_at = ""` and `youden_j_max = 0.0`. The dashboard surfaces
an "uncalibrated" badge when `calibrated_at` is empty (RUNPOD-PENDING
follow-up wiring).

DECISION-LTOT-SIMPLIFICATION

Paper arXiv:2510.01500 LToT introduces Lateral Racing with Short-
Circuit (LR-SC) — a budgeted race with aggressive culling. ELAI ships
the SIMPLIFIED version per cluster pragmatism + 7B latency budget:
keep top `N_MAINLINE = 3` by utility, keep up to `N_LATERAL = 2`
low-utility-but-logically-consistent candidates one extra ply. The
simplification is documented in `crates/orchestrator/src/reasoning/ltot.rs`
module docs.

DECISION-PRM-LITE-D2-UPGRADE-PATH

PRM-lite via the auditor is a stopgap. The auditor is deterministic-
syntactic only; it cannot score "is the reasoning step semantically
valid" — only "does the resulting code compile + pass syntactic rules."
When D-2 (PPM sidecar at port 8101 with Qwen2.5-Math-PRM-7B) ships, the
PRM-lite `auditor_pass` boolean is replaced with the PPM scalar score;
the weighted-combine math
(`AUDITOR_WEIGHT=0.6, MEAN_LOGPROB_WEIGHT=0.2, COSINE_WEIGHT=0.2`) is
unchanged. D-2 sequencing: AFTER E-19 main.

DECISION-NO-NEW-DASHBOARD-PANEL

E-19 is NOT in the dashboard list per
`docs/specs/guide/steps/phase-index-3A-4J.txt`. Plan-doc's reference
to `crates/dashboard-v2/src/lib/components/ReasoningDrawer.svelte`
modification is RUNPOD-PENDING (the existing
`ReasoningDrawer.svelte` lives at `drawers/`, not `components/`); the
+80-LOC extension for the new ProbeKind fields ships in a follow-up
when the dashboard is in scope. The dashboard subscribes to the
existing `ssb_events` IPC v2 stream filtered by `action_type` — no new
Tauri command is required.

DECISION-NO-UNWRAP

Per CLAUDE.md anti-unwrap rule, every new `core/src/reasoning/*.rs` +
`orchestrator/src/reasoning/*.rs` file uses `?` / `.ok_or()` /
`.map()` / `.unwrap_or(0.0)` / saturating-arithmetic. `.unwrap()` /
`.expect()` appear ONLY in `#[cfg(test)]` blocks.

FILES SHIPPED THIS COMMIT

CREATE:
  .elai_cc/crates/core/src/reasoning/mod.rs
  .elai_cc/crates/core/src/reasoning/self_certainty.rs (~240 LOC)
  .elai_cc/crates/core/src/reasoning/typed_cot.rs (~135 LOC)
  .elai_cc/crates/core/src/reasoning/ttc_budget.rs (~195 LOC)
  .elai_cc/crates/orchestrator/src/reasoning/mod.rs (~50 LOC)
  .elai_cc/crates/orchestrator/src/reasoning/prm_lite.rs (~175 LOC)
  .elai_cc/crates/orchestrator/src/reasoning/ltot.rs (~165 LOC)
  .elai_cc/crates/orchestrator/src/reasoning/l6_verifier_diversity.rs (~135 LOC)
  .elai_cc/crates/orchestrator/src/reasoning/l3_direct.rs (~95 LOC)
  .elai_cc/crates/contracts/tests/e19_reasoning_quality_probe_ssb_contract.rs (~140 LOC)
  .elai_cc/eval/reasoning/calibration.toml (seed)
  .elai_cc/eval/reasoning/traces.toml (5 seed rows + schema header)
  workflow/reasoning/E-19-decisions.md (this file)
MODIFY:
  .elai_cc/crates/core/src/lib.rs (+pub mod reasoning declaration)
  .elai_cc/crates/orchestrator/src/lib.rs (+pub mod reasoning declaration)
  .elai_cc/crates/orchestrator/src/ssb.rs (+ReasoningQualityProbe variant
    + ProbeKind enum + ReasoningQualityProbePayload struct + SCHEMA bump
    165 -> 166 + CHECK + as_sql_str arm + from_str arm + SsbPayload arm
    + action_type arm + test_payload arm)
  .elai_cc/crates/orchestrator/tests/action_type_lock.rs (TOTAL_VARIANTS
    438 -> 439; ALL_VARIANTS append; exhaustive guard arm)
  .elai_cc/crates/orchestrator/tests/action_type_lock.golden (regenerated
    via in-place sort; +1 row reasoning_quality_probe at ordinal 0303;
    subsequent ordinals +1)

RUNPOD-PENDING (NOT shipped this commit)

  - Live micro_reflection_emitter.rs confidence-gate dispatch wiring
  - Live task_runner.rs::emit_reasoning difficulty-estimator + TTC dispatch
  - Live chat_reflect.rs L2_MAX_TOKENS + re-sample-with-prior framing
  - Live per_level_audit_dispatch.rs PRM-lite consumer
  - 200-trace harvest + Youden's-J calibration sweep
  - Dashboard ReasoningDrawer.svelte +80 LOC ProbeKind fields panel
  - paper-ingestor RAG re-ingest for the 6 plan-cited papers that timed
    out (workflow/papers/ entries exist as written but RAG index pending)

ACCEPTANCE STATUS (vs plan-doc list)

[x] Self-certainty gate primitive (ConfidenceGate + compute_score)
[x] L2 token cap heuristic (L2_MAX_TOKENS const, paper-citation dropped)
[x] L3 direct route helper (l3_direct_route fn + per-category routing)
[x] PRM-lite L4 node scoring math (AuditorVerifierScore + weighted_score)
[x] LToT mainline/lateral split (lateral_branch fn + N_MAINLINE/N_LATERAL)
[x] Typed-CoT triple struct + XGrammar JSON schema
[x] L6 verifier diversity rule (verifier_diversity + DiversityProbe)
[x] Difficulty estimator + TTC budget (DifficultyEstimator + TtcBudget)
[x] Calibration scaffolding (traces.toml + calibration.toml)
[x] D-2 upgrade path documented
[x] CLAUDE.md anti-stale: no async-trait, no log crate, no HashMap,
    no .unwrap() in library code
[~] Live wiring into reasoning pipeline (RUNPOD-PENDING — substrate
    only this commit)
[~] Dashboard ReasoningDrawer extension (RUNPOD-PENDING — not in
    dashboard scope per phase-index)
[~] 200-trace calibration (RUNPOD-PENDING — seed rows + schema only)
