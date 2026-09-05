# I-25: ledger-trained router — train the route decision on own verifier outcomes, not proxy signals

- Track 16 — ROUTING + BUDGETS + MEASUREMENT | lever CHEAPER-SPEED | grounding STRONG | nature EXTENSION | effort M | WAVE 4 | binds ID-5/ID-6
- Premise: routing products such as RouteLLM train on human proxy labels; ELAI's journal already holds
  ground-truth verifier outcomes per (task features, lane, cost). A small classifier trained on those receipts
  routes without speculation. Execution-cost profiling pre-routes known-expensive AST shapes to the deep lane
  before a cheap-lane failure burns tokens. Router promotions are ledger-gated — the router is a learned
  artifact and therefore subject to ID-6.

## Steal/build
the distinguishing claim versus commodity routers (RouteLLM, ours-at-H-7) is the label source: H-7 is a
cost-calibrated heuristic; I-25 trains on verifier-outcome labels from the journal (W-1-grade, not
proxy). features: task class (AST shape / DKG centrality / file count), historical cost shape per class,
recent pass-rate per lane. a SMALL classifier (logistic or gradient-boosted — no neural overhead; neural
scoring deferred behind the W-10 gate). offline trainer runs on ELAI_SEED-sealed journal exports. ID-5
(local-baseline-before-paper-threshold): the router's over-escalation rate is measured on ELAI's own mixed
batch, never a RouteLLM paper number; a router that does NOT beat the H-7 heuristic baseline is NOT
promoted (ID-6: the promotion row must cite the ledger comparison). execution-cost profiling layer: before
routing, classify the task's shape against the historically-expensive-shape fingerprint set; if match,
pre-route to the deep lane unconditionally. the emitted SSB row is the E-31 RouteDecisionEvent (NO new
variant — reuse LIVE emission; feature_vec + predicted_lane + reason extend details_json). EXCLUDED: Shapley
attribution for router explanation (research-grade, same reason as I-28 DECLINED); neural router as the first
increment (deferred Tier-2 — no VRAM budget allocated).

## Existing ground truth (ratified 2026-08-08 @ HEAD after I-16)
- IS-19: wiring-contract tooling + wire-dark-allowlist.toml live; R8 frozen-gate discipline in force.
- H-7 RouteDecisionEvent is LIVE (not dark): task_runner + runtime_orchestrator emit behind
  ELAI_ROUTE_DECISION_SSB_ENABLE. Reuse details_json; NO new action_type (FORK-1 lean).
- PRIMARY pre-route production hook: task_runner.rs `run_cost_router_preclassify` (called on pre-cloud
  dispatch). Do NOT treat crates/orchestrator/src/escalation_policy.rs as the hook — that file is E-32
  handoff policy. Heuristic fallback compose target is providers/src/runtime/escalation_policy.rs
  RuntimeRoutingPolicy::evaluate (dark production callers today; not Slice-1 sole hook).
- C-46 selective delegation lives under crates/orchestrator/src/routing/ (budget_controller, difficulty,
  policy, selective). Name cost_profile.rs is free. CostProfiler ABSENT — genuine BUILD gap.
- IS-16: trial_ledger.rs / deflation_gate.rs / holdout_budget.rs / gate_self_validation.rs exist.
  Consume only; never edit (R8). ID-6 binds router promotion to TrialLedger::register_trial.

## REVISED scope (W-10 tiered + seam proposal slices)
SHIP NOW (Tier-1):
- SLICE 1: CostProfiler + live pre-route hook (Rust). CREATE
  crates/orchestrator/src/routing/cost_profile.rs (~80 LOC: deterministic task-shape fingerprint
  classifier, BTreeMap cache, no unwrap/expect). WIRE into run_cost_router_preclassify before
  route_for_difficulty_and_budget; emit RouteDecisionEvent with details_json.reason=
  cost_profile_expensive_shape + predicted_lane + feature_vec. Contract test
  crates/contracts/tests/i25_ledger_trained_router_contract.rs (in-process, deterministic).
- SLICE 2: offline trainer (Python). CREATE workflow/training/router/train_router.py (~150 LOC
  stdlib: sqlite3 read-only over ssb.db; join RouteDecisionEvent to verifier + UsageTurnEvent;
  logistic classifier; write model artifact TOML + eval_report.toml). Insufficient labels => REFUSED.
- SLICE 3: ledger-gated artifact load. Load artifact from elai_cc.toml ONLY when a trial-ledger
  row records the win; loser never loaded. Fallback remains H-7/E-39 heuristics.

DEFER (Tier-2 / gated):
- neural router variant (RunPod gated — VRAM budget required)
- continuous online retraining (depends I-4 high-volume trajectory stream)
- RuntimeRoutingPolicy::evaluate production wire (H-7 Tier-2 debt, not I-25 Slice-1 primary)
- dashboard/cockpit route-explanation UI (owner-ui-rework-hold — backend events only)

## Integration
- CREATE: crates/orchestrator/src/routing/cost_profile.rs::CostProfiler
- MODIFY: crates/orchestrator/src/task_runner.rs (run_cost_router_preclassify primary hook)
- MODIFY: crates/orchestrator/src/routing/mod.rs (pub mod cost_profile)
- CREATE: crates/contracts/tests/i25_ledger_trained_router_contract.rs
- CREATE: workflow/training/router/train_router.py::RouterTrainer (Slice2)
- ABORT if touched: trial_ledger.rs / deflation_gate.rs / holdout_budget.rs / gate_self_validation.rs
- seams: task_runner preclassify + E-31 RouteDecisionEvent details_json + IS-16 consume-only

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/routing/cost_profile.rs::CostProfiler (shape classifier, pre-route verdict)
  - workflow/training/router/train_router.py::RouterTrainer (offline trainer; does NOT write ledger rows)
  - task_runner.rs run_cost_router_preclassify (modified): CostProfiler consulted before E-39 route

consumes_symbols:
  - task_runner.rs::run_cost_router_preclassify (PRIMARY production hook) — FORK-2 lean
  - cost/cost_router.rs::route_for_difficulty_and_budget (E-39; keep when profiler non-match / default-off)
  - ssb RouteDecisionEventPayload.details_json (feature_vec + predicted_lane + reason keys) — FORK-1 lean
  - providers/src/runtime/escalation_policy.rs::RuntimeRoutingPolicy (fallback compose later; not Slice-1 sole)
  - IS-16 trial_ledger.rs::register_trial / deflation_gate / holdout_budget (promotion gates, consume-only) — ID-6

reachable_from:
  - `elai run <task>` -> task_runner pre-cloud path -> run_cost_router_preclassify -> CostProfiler
  - RouteDecisionEvent SSB row is the off-agent artifact (default-OFF via ELAI_ROUTE_DECISION_SSB_ENABLE)

stub_ban:
  - NO promotion of a router artifact without a ledger win (IS-16, ID-6)
  - NO new RouteDecisionEvent action_type / schema bump for cost profile (use details_json)
  - NO fabricated measurements or paper RouteLLM numbers as thresholds (ID-5)
  - NO .unwrap()/.expect() on production routing paths; use ? / .ok_or()
  - NO edit to frozen trial_ledger / deflation_gate / holdout_budget / gate_self_validation
  - NO primary hook solely on RuntimeRoutingPolicy::evaluate (zero production callers = dark ship)

frontend_pair:
  DEFERRED under owner-ui-rework-hold. Backend emits details_json only; no dashboard/Svelte work this unit.

observable_effect:
  trigger: "cargo test -p elai-contracts i25_ledger_trained_router_contract" and later "elai run --task fixture"
  expect_ssb: "RouteDecisionEvent with details_json.reason=cost_profile_expensive_shape on expensive fingerprint; non-match keeps E-39 verdict"
  expect_artifact: "workflow/training/router/eval_report.toml (Slice2) with ELAI-measured rates + seed + blake3; loser artifact path absent when promotion blocked"

## Grounding
- Seam proposal: conductor/opus_walk/evidence/I-25/seam-design-proposal.md (BUILD-WITH-SEAM ratified).
- IS-19 (wiring tooling live), IS-16 (H-gate files consume-only), IS-4 config_archive promotion precedent.
- Track 16 anchor: live pre-route = run_cost_router_preclassify; E-31 RouteDecisionEvent LIVE.
- ID-5: all threshold and accuracy claims are ELAI-measured; paper RouteLLM numbers never targets.
- ID-6: router artifact promotes only through IS-16 ledger gates; evaluator outside write boundary (R8).
- HD-1: efficiency numbers measured per config, never multiplied.
- SSB: NO new action_type — re-read LIVE action_type integers at edit time (W-9); do not invent SCHEMA_VERSION.
- arXiv IDs: none claimed for RouteDecisionEvent or router classifier math (first-party journal labels).

## COMPOSES-WITH
- E-39 cost router (route_for_difficulty_and_budget) — keep as non-match / default-off path.
- H-7 calibrated routing RuntimeRoutingPolicy — compose as fallback later; do not re-implement.
- E-31 RouteDecisionEvent — extend details_json only; do not add a new action_type.
- C-46 selective delegation — pre-route filter in routing/; do not re-implement.
- E-20 privacy routing — pre-route filter; do not re-implement.
- IS-16 gate files — promotion uses these directly; do not re-implement or edit.

## J-11 AMENDMENT — S-8b DIFFICULTY FUSION AS ROUTER FEATURE (source-cited)
Grounding: DeNovoSWE paper `2606.10728.html` §4.1 (S4.SS1), Eq.4-5 (S4.E4/S4.E5), Table 2 (S4.T2), read directly per W-3.
DeNovoSWE's difficulty fusion is a working existence proof of a multi-signal difficulty score that I-25's
router can consume as a feature. The fusion formula (Eq.4, `S4.E4`):
  d_i = w_s · d_i^struct + w_g · ℓ_i^(g) + w_q · ℓ_i^(q)
subject to (Eq.5, `S4.E5`): w_s, w_g, w_q ≥ 0, w_s + w_g + w_q = 1 (2-simplex Δ²).
Three signals (`S4.SS1.SSS0.Px1`):
- Structural (`S4.I1.i1.p1.1`): total executable Python lines (e_i ∈ Z≥0), log(1+·) transformed + min-max
  normalized against 5th/95th percentiles (`S4.I2.i1.p1.5`).
- LLM judge g (`S4.I1.i2.p1.3`): 5-level qualitative difficulty from annotator g.
- LLM judge q (`S4.I1.i2.p1.3`): independent 5-level from annotator q.
Weight optimization (`S4.SS1.SSS0.Px4.p1.5`, Eq.5): w* = arg max_{w ∈ Δ²} |ρ(d(w), s)| where ρ = Pearson
correlation, s = empirical pass rate. Solved via exhaustive 2D grid sweep over (w_g, w_q) on a uniform dense
mesh (`S4.SS1.SSS0.Px4.p1.10`). Winning weights NOT published — ELAI calibrates on its OWN rollouts.
For I-25: the structural signal ("executable Python lines") needs a domain substitute — ELAI is Rust+TS+Python,
not Python-only. Candidate substitutes: SCIP function count (from the DKG symbol graph), cross-language
bindings touched, SSB action_type variants added per task. The fusion score d_i is a router FEATURE input,
not a gate — I-25's ledger gates remain authoritative (ID-6). The +1.2pp figure (0.488→0.500 on
BeyondSWE-Doc2Repo, `S5.SS3.p3.1`) is design rationale for the banding approach, never an ELAI target (HD-1/ID-5).

## Wargame / acceptance

EXPECTED_OBSERVATION: fixture task matching an expensive CostProfiler fingerprint yields a
  RouteDecisionEvent row with kind=CloudRouteChosen and details_json.reason=
  cost_profile_expensive_shape (plus predicted_lane and feature_vec keys). A non-matching task
  keeps the E-39 cost-router verdict unchanged. Default-off config leaves E-39 behavior alone.
  Slice2: eval_report.toml exists with ELAI-measured over-escalation and mis-route rates plus seed and
  input blake3; insufficient labeled rows yields an explicit REFUSED report (never a silent model).
  Slice3: loser artifact is never loaded (promotion_blocked: path absent in live binary).

ON_FAIL: missing RouteDecisionEvent or wrong reason string or E-39 regression when profiler
  default-off -> CAUSE CostProfiler not wired into run_cost_router_preclassify or details_json
  not filled -> COUNTER-MOVE re-read task_runner.rs run_cost_router_preclassify plus H-7 emission
  sites plus i25 contract test; re-run cargo test -p elai-contracts i25_ledger_trained_router_contract.

FORK: FORK-1 payload = details_json keys only (lean; no schema bump; no new action_type).
  FORK-2 hook = run_cost_router_preclassify primary (lean live caller); RuntimeRoutingPolicy::evaluate
  is composed fallback later, not Slice-1 sole hook. FORK-3 ledger row writer stays on Rust
  TrialLedger::register_trial (lean); Python trainer does not write ledger rows directly.

ABORT: any edit to trial_ledger.rs or deflation_gate.rs or holdout_budget.rs or
  gate_self_validation.rs (frozen R8). STOP if LIVE SSB action_type integers cannot be re-read
  (do not invent SCHEMA_VERSION bumps). STOP if a worker invents a new RouteDecisionEvent
  variant instead of details_json keys. Never promote a router artifact without a ledger win (ID-6).

VERIFY (off-agent; controller/auditor owns these runs):
  - python3 .claude/tools/plan_gate_check.py --file docs/thefinalglance/I-plan/I-25_ledger_trained_router.md
  - cargo test -p elai-contracts i25_ledger_trained_router_contract
  - cargo check -p elai-orchestrator --all-targets
  - production_caller_check on CostProfiler and new pub symbols in changed .rs files
  - python3 -m py_compile workflow/training/router/train_router.py (Slice2) plus python_wire_check

REDTEAM: attack "promote router without ledger row / silent adopt of losing artifact" must FAIL
  closed. attack "new action_type for cost profile instead of details_json" is banned (GD-1/FORK-1).
  attack "hook only RuntimeRoutingPolicy::evaluate with zero production callers" is banned for
  Slice-1 primary (ships dark).

INTEGRATION_SMOKE: cargo test -p elai-contracts i25_ledger_trained_router_contract (in-process
  deterministic) proves expensive fingerprint maps to RouteDecisionEvent reason=cost_profile_expensive_shape
  and non-match leaves E-39 path. No dashboard UI surface (UI-hold).

RECON: none open — seams re-read at ratification (task_runner run_cost_router_preclassify, routing/
  module inventory, RouteDecisionEvent LIVE, TrialLedger::register_trial).

End of I-25.
