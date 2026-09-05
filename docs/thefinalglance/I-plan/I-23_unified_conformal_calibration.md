# I-23: unified conformal calibration

- Track 15 — DETERMINISTIC GATES + ANTI-GAMING VERIFICATION | lever SMARTER-ORCH | grounding STRONG | nature EXTENSION | effort S-M | WAVE 1 | IC-13
- Premise: conformal_calibration.rs and calibration_bootstrap.rs are SHIPPED (IS-7: calibration_bootstrap.rs
  warm_start_tau:12 + conformal_calibration consumers across task_runner/agent_pool). they are consumed today by
  D-16 refusal and G-7 claim_score (planned) through separate call sites. if tau semantics drift between consumers
  (two calibration implementations, or one impl called with different warm-start assumptions), refusal thresholds and
  search uncertainty forks become structurally inconsistent. i-23 is UNIFICATION: one facade, three consumers, no
  new math. IC-13 is the binding correction: the calibration math already ships; i-23 is structural.

## Steal/build
CALIBRATION FACADE: a single module (calibration.rs or calibration/mod.rs) that re-exports the one
conformal_calibration + calibration_bootstrap implementation with a clean, typed interface: get_tau(CalibrationRequest)
-> TauResult, update_tau(ObservedOutcome), warm_start(). all three consumers call this facade; no consumer calls
conformal_calibration.rs directly after i-23 ships. the facade is a thin wrapper — no new math, no new algorithm.
THREE CONSUMERS:
  D-16 refusal (SHIPPED): migrates its conformal_calibration call site to call the facade. no change to refusal logic.
  G-7 claim_score (planned producer): routes through the facade (PLAN-asserted; when G-7 ships it writes to facade).
  i-4 search prior / uncertainty fork (NET-NEW): high epistemic uncertainty (wide TauResult CI) triggers an
  exploration fork in the PUCT tree instead of confident exploitation. the uncertainty-to-exploration rule is
  deterministic (threshold on CI width, measured on ELAI's own fixture corpus — ID-5).
DRIFT PREVENTION: the facade is the ONLY import path for calibration. a contract test asserts that no module in the
codebase imports conformal_calibration.rs directly (rg the direct import path — MUST be 0 after migration). this
structural test is what prevents tau-semantic drift between consumers.
UNCERTAINTY FORK: when the facade returns a TauResult whose CI width exceeds a measured threshold, the i-4 PUCT
expansion inserts an exploration fork (a new branch with a divergent hypothesis, not a repeat of the current one).
this is the mechanism by which calibrated uncertainty becomes a search-steering signal — explicitly without any new
calibration math.
IC-13 is the binding correction: the Consolidated draft said "implement Brier calibration" — wrong, conformal is
already shipped. i-23 is EXTENSION/UNIFICATION, not a new algorithm port.
EXCLUDED: any new calibration algorithm or mathematical variant; any design that retains multiple direct import
paths to conformal_calibration.rs after migration; any threshold for the uncertainty fork derived from a paper
constant without ELAI measurement (ID-5).

## Existing ground truth
- IS-7: calibration_bootstrap.rs warm_start_tau:12 (docstring) + conformal_calibration consumers across task_runner/agent_pool — SHIPPED; the substrate is real and functional
- IS-7: conformal_calibration.rs — SHIPPED; i-23 re-exports it, never re-implements it
- D-16 refusal substrate — SHIPPED; migrates its call site to the facade
- G-7 claim_score — planned producer; facade is its natural write path when it ships
- i-4 PUCT value function (WAVE 4, depends i-4): the uncertainty fork consumer; i-23 is WAVE 1 because the facade can ship before i-4 (the fork logic is gated on i-4's PUCT existing)
- IC-13 (binding correction): conformal_calibration ships; i-23 = unification; Brier calibration is NOT the task

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- calibration/mod.rs facade (re-exports one implementation, typed interface: get_tau / update_tau / warm_start)
- D-16 call-site migration to facade (do NOT change refusal logic)
- drift-prevention contract test (rg direct conformal_calibration.rs imports — MUST be 0)
- uncertainty fork in i-4 PUCT (threshold measured from fixture corpus, writes metrics/calibration_fork_threshold.toml — ID-5)
- NO new SSB action_type (I-0 table: "NO new action_type — reuse RefusalDecision/CalibrationUpdate (D-16) rows; unification is structural")
DEFER (Tier-2 / gated):
- G-7 claim_score call-site migration: G-7 owns its own call site when it ships; the facade is ready, the migration is G-7's responsibility
- multi-model tau divergence detection (separate taus for cloud vs local model families): a future enhancement; single tau ships first

## Integration
new module: crates/orchestrator/src/calibration/mod.rs (the facade — a thin wrapper, no new logic).
modify: D-16 refusal call site (one import path change + test confirmation that D-16 behavior is identical).
modify: i-4 search/ladder.rs or the PUCT expansion site (add uncertainty-fork logic conditioned on facade output —
PLAN-asserted; executor confirms the exact seam at EXECUTING, W-7, as i-23 is WAVE 1 and i-4 is WAVE 4; the fork
logic in i-23 is scaffolded as a conditional that fires only when PUCT is live).
NO new SSB action_type — reuse RefusalDecision / CalibrationUpdate (D-16) rows per I-0 SSB delta table (GD-1).
frontend_pair: N/A — this is an internal unification; the existing refusal surface is unchanged; W-6 justified
(no new dashboard signal; the existing calibration tiles remain on the existing D-16 surface). Justified.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/calibration/mod.rs::CalibrationFacade (get_tau / update_tau / warm_start typed interface)
  - crates/orchestrator/src/calibration/mod.rs::UncertaintyFork (threshold + fork trigger rule for the i-4 PUCT seam)
  - metrics/calibration_fork_threshold.toml (ID-5 baseline measurement artifact for the uncertainty-fork threshold)
consumes_symbols:
  - crates/orchestrator/src/conformal_calibration.rs (IS-7 — why: the facade re-exports this; GD-1 do NOT re-implement calibration math)
  - crates/orchestrator/src/calibration_bootstrap.rs warm_start_tau (IS-7 — why: the facade exposes the warm-start; GD-1)
  - D-16 refusal (why: migrates its call site to the facade; GD-1 — no change to refusal logic itself)
  - i-4 PUCT expansion seam (why: UncertaintyFork is consumed here; PLAN-asserted; re-grep at EXECUTING, W-7)
  - G-7 claim_score (planned: will call the facade when it ships; facade is the ready path; i-23 does not block on G-7)
reachable_from:
  - D-16 refusal path in task_runner/agent_pool (via the facade — the facade is the ONLY import path; this is an existing production caller migrated)  [PRODUCTION refusal path]
  - i-4 PUCT expansion (the uncertainty fork consumer — a new production caller)  [PRODUCTION search PUCT]
  (contracts/ callers never satisfy R7; the production refusal path + PUCT are the reachable_from chains)
stub_ban:
  - NO direct import of conformal_calibration.rs by any module after migration (the drift-prevention contract test enforces this — rg must return 0)
  - NO new calibration algorithm in this session (unification only — IC-13)
  - NO uncertainty-fork threshold derived from a paper constant (ID-5): threshold comes from metrics/calibration_fork_threshold.toml
  - NO new SSB action_type for calibration: reuse RefusalDecision/CalibrationUpdate (D-16) rows per I-0 table
  - NO .unwrap()/.expect() on production paths; NO fabricated tau values or CI widths
frontend_pair:
  N/A — internal unification; the existing refusal and calibration surfaces on the D-16 dashboard are unchanged.
  the uncertainty fork in i-4 is surfaced via the existing branch board (i-4's surface). W-6 justified: no new
  dashboard component is needed for a structural unification. the existing calibration tiles keep working. Justified.
observable_effect:
  trigger: "elai run --task-fixture fixtures/calibration_trace.toml --emit-ssb (fixture that triggers both a refusal decision and a search uncertainty fork in the same run)"
  expect_ssb: "RefusalDecision + CalibrationUpdate rows (D-16 reuse — no new row type). the calibration_id field on both rows is the SAME value (proving they share one tau). journal join on calibration_id proves drift is structurally impossible."
  expect_artifact: "a --split-calibrations broken arm (patches the facade to maintain two independent tau values, one for refusal and one for search) lets the two drift and FAILS the new contract test (crates/contracts/tests/i23_unified_calibration_contract.rs asserting same calibration_id on the RefusalDecision and search-prior CalibrationUpdate rows — R2). D-16's existing observable effects keep passing untouched (regression guard: the test suite for D-16 is re-run verbatim)."

## Grounding
- IS-7 (calibration_bootstrap.rs:12 + conformal_calibration consumers): the SHIPPED substrate; IC-13 is the binding correction
- IC-13 (binding correction): conformal_calibration SHIPS; i-23 = structural unification not a new algorithm; Brier calibration NOT the task
- ID-5 (local-baseline-before-paper-threshold): the uncertainty-fork threshold is measured on ELAI's corpus; no paper value is wired
- D-16 refusal substrate: SHIPPED; migrates its call site only (GD-1)
- G-7 claim_score: planned; facade is ready; no blocking dependency on G-7
- Consolidated Part IV + review 1.1 "unification of calibration across refusal, claim score, search prior"
- SSB: NO new action_type — reuse RefusalDecision/CalibrationUpdate (D-16) rows; unification is structural (per I-0 delta table). baseline 221/532 as of 2026-06-10 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9).

## COMPOSES-WITH
D-16 conformal_calibration call site (migrates to facade — do NOT change refusal logic, only the import path); G-7
claim_score (will call facade when it ships — do NOT absorb G-7 logic); H-39 abstention scoring (the credited
outcome upstream of calibration — do NOT re-implement); i-4 PUCT uncertainty fork (the new consumer — do NOT
inline fork logic into the facade). the facade itself is the net-new artifact; everything else is composition.

End of I-23.
