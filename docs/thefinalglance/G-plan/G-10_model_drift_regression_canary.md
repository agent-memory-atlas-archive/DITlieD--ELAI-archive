# G-10: Model-Drift Regression Canary — the exogenous model-version behavioral-regression gate

STATUS: PLANNED (Post-G-9 extension, added 2026-06-02; WAVE 3 — depends G-2; reuses D-7 canary.rs)

Objective: close the one trust seam the whole G/F substrate does NOT cover — ELAI today only watches its
OWN changes (self-mod templates, GEPA prompts, per-refine CWE-class, goal-drift between steps); NOTHING
watches whether the model UNDERNEATH it changed. When a provider silently ships a new Sonnet/Qwen
checkpoint (or a local weights swap lands), a prompt/grounding path that worked last week can regress with
zero changelog. G-10 adds a model-IDENTITY-keyed behavioral canary: pin the (model, fingerprint, lane)
tuple, hold a small fixed canary set with baseline scores keyed to that identity, and on a detected
identity change (or schedule) re-run the canary, compute a per-family regression delta against the
last-good baseline for that identity, and on regression FLAG `model_drift_suspected` + route the affected
ReasoningTemplates to review. SURFACE, never auto-swap by default (GD-4). HONEST SCOPE (GD-5): on the local
lane the identity is ground-truth (weights/version) so drift is FingerprintConfirmable; on the cloud lane
Anthropic's system_fingerprint is opaque (per G-2 §6) so the signal is BEHAVIORAL-drift-suspected, never
fingerprint-confirmed — state this, do not over-claim.
Estimated effort: ~1 session (~6-10 h). In-process; reuses shipped canary machinery + G-2's identity pins.

Provenance (why this file is a Post-G-9 extension, not an R1..R9 roadmap item): derived 2026-06-02 from a
gap analysis corroborated by external practitioner discourse (r/ClaudeAI "production-ready AI is not sexy
work" — the unsexy-90% list explicitly names "prompt version control so you can roll back when a model
update breaks your use case"; tier-5 sentiment, corroboration NOT a requirement) and CONFIRMED by an
elai.map grep: there is NO model-version pin type, all regression/canary/drift symbols key on ENDOGENOUS
change, substitute_model() fires on max_restarts/VRAM (not regression), and the `ModelSwap` / `TierModelSwap`
SSB variants are themselves WIRE-DARK ("variants without a current production emitter", ssb.rs:12285). The
seam is real, small, and a COMPOSITION gap — the detection substrate already exists or is planned.

Pre-requisites: G-2 SHIPPED or co-shipped (RunArtifact carries `system_fingerprint` + `model_pin` — the
identity-capture input G-10 reads). D-7 canary machinery SHIPPED (crates/orchestrator/src/canary.rs —
CanaryFamily + FamilyEvalResult + the regression-delta + sample-count/threshold logic; reused, NOT
re-implemented — GD-1). held_out fixtures SHIPPED (crate::held_out::TaskId — canary.rs:699). F-4 SHIPPED
per memory F-plan Tier-1 (TestOutcomePayload.stdout_hash / deterministic pass-rate — the malformed-output
signal G-10 consumes). reasoning_template module present (reasoning_template::Domain — canary_contract.rs:65)
— the revert/review target (D-1 ReasoningTemplate archive).
Parallel-safe with: G-5, G-8, G-9 (all read the SSB/RRA, no write-path collision). NOT parallel-safe with
G-2 in the same window (consumes RunArtifact identity fields) or any plan editing canary.rs.
Source: this session's gap analysis + elai.map grep (2026-06-02); model-drift prior art (Citations below).
Doctrine: G-0_INDEX.md (W-1..W-10 + GD-1..GD-5) — INHERITED VERBATIM.

## Context

ELAI's regression apparatus is rich but ENDOGENOUS-only (grep-confirmed 2026-06-02):
- `security_monotonicity.rs` MonotonicityVerdict::Regressed = F-10 per-refine CWE-class.
- `staged_eval.rs` / `self_mod_*` PASS_TO_FAIL "regressed" = DGM self-mod candidate grading.
- `optimizer_admit.rs::assert_no_cross_domain_regression` = D-7 canary on a GEPA-OPTIMIZED template.
- `canary.rs` CanaryFamily / `regressed_domain` = cross-domain transfer of a TEMPLATE mutation.
- `goal_drift_canary.rs` GoalDriftCanary = goal-Jaccard between AGENT steps.
Every one answers "did ELAI's OWN change regress?". None answers "did the provider's model change under me?"
— the literal practitioner complaint. `autonomous_gate.rs::substitute_model()` swaps from a fallback chain
but is triggered by `max_restarts_exhausted` / VRAM, not by a measured behavioral regression. And the
`ModelSwap` + `TierModelSwap` SSB action types have NO production emitter (wire-dark) — even the telemetry
for a model change is dark today.

The fix is a COMPOSITION, not an invention (GD-1): G-2 already captures the (model_pin, system_fingerprint,
lane) tuple per run — that is the detection INPUT; D-7's canary.rs already computes a per-family
regression delta with a sample-count / non-determinism band — that is the MEASUREMENT; D-1's
ReasoningTemplate archive is the REVERT/review target. G-10 wires them into a model-identity-keyed gate
and adds the one missing thing: a baseline keyed to the MODEL IDENTITY (not a template/self-mod candidate).

THE HONEST GUARANTEE (GD-5): (1) LOCAL lane — model identity = weights/version hash = ground truth; a
fingerprint change is reliably detectable, drift is FingerprintConfirmed + CanaryRegressed. (2) CLOUD lane —
Anthropic's system_fingerprint is opaque (G-2 §6), so a same-named alias can change behavior invisibly;
G-10 therefore relies on the BEHAVIORAL canary (fixed tasks, scored, compared) and labels the signal
BehavioralDriftSuspected, never FingerprintConfirmed. (3) The gate SURFACES (advisory row + template-review
queue); it does NOT auto-swap or auto-revert the model by default — a false drift alarm that auto-swapped
the model would be strictly worse than the drift (GD-4). Auto-pin-to-last-good is a separately-gated,
wire-dark-allowlisted ACTION with a promotion plan, not a terminal default-off graveyard (W-2 / R9).

## Existing state (B/C/D/E/F/G coverage)
- crates/orchestrator/src/canary.rs — SHIPPED. CanaryFamily (CodeFix/MathProof/...), FamilyEvalResult,
  regressed_domain, sample-count + threshold + non-determinism band, fixture loading via held_out. REUSED
  for the per-family delta — G-10 re-implements NONE of it (GD-1).
- crates/orchestrator/src/rra.rs::RunArtifact — PLANNED (G-2). Carries `system_fingerprint: String` +
  `model_pin: String` — the identity tuple G-10 reads. HARD dependency on G-2.
- crates/orchestrator/src/ssb.rs `ModelSwap` + `TierModelSwap` — SHIPPED but WIRE-DARK (no production
  emitter, ssb.rs:12285 / ssb.rs:8115 "no live emit in 6A"). G-10 does NOT emit these (a drift event is
  not a swap); it adds its own `model_drift_suspected`. A recommended swap is an ADVISORY field, not an act.
- crates/orchestrator/src/autonomous_gate.rs::substitute_model — SHIPPED. Restart/VRAM-triggered fallback,
  NOT regression-triggered. G-10's optional auto-pin action MAY later feed this; not in the done-gate.
- reasoning_template module (reasoning_template::Domain — canary_contract.rs:65) — present. The D-1
  template archive is the review/revert target on a confirmed drift.
- crates/orchestrator/src/ssb.rs::TestOutcomePayload (F-4 stdout_hash / deterministic pass-rate) — SHIPPED
  per memory F-plan Tier-1. The malformed-output-spike signal G-10 consumes (COMPOSES-WITH F-4, GD-1).
- crate::held_out::TaskId (canary.rs:699) — SHIPPED. The fixed canary task fixtures, reused.

## Reads (mandatory before coding)
- this file's Provenance + the 2026-06-02 gap analysis; the prior-art papers in Citations (paper-ingestor
  if effect sizes are to be quoted — page-anchor required per memory feedback_paper_citation_verify).
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\canary.rs (CanaryFamily, FamilyEvalResult, the
  delta + sample-count + non-determinism band — the machinery to REUSE, confirm the exact API at authoring).
- F:\.ELAI_workflow\docs\thefinalglance\G-plan\G-2_reproducible_run_artifact.md (RunArtifact identity
  fields + the cloud-lane system_fingerprint-opacity caveat G-10 inherits).
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs (write_payload; the ModelSwap/TierModelSwap
  wire-dark variants; TestOutcomePayload.stdout_hash; action_type golden lock).
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\runtime_setup.rs (build_production_hook_dispatcher —
  the run-end hook where an identity-change check fires).
- F:\.ELAI_workflow\.elai_cc\crates\cli\src\main.rs (CLI dispatch for the `model-drift` subcommand).
- CLAUDE.md HARD RULES: blake3 (canary-output hashing), no .unwrap()/.expect() in prod, anti-slopsquatting.

## Tasks (WAVE 3)

1. **Add crates/orchestrator/src/model_drift.rs (~240 LOC) — the identity + baseline types.**
   `pub struct ModelIdentity { model_pin: String, system_fingerprint: Option<String>, provider: String,
   lane: Lane }` (`pub enum Lane { Local, Cloud }`); `identity_key()` = blake3 over the tuple.
   `pub struct DriftBaseline { identity_key: String, per_family_scores: Vec<(CanaryFamily, f64)>,
   captured_at: u64, output_blake3: Vec<String> }`. `pub enum DriftSignal { FingerprintChanged,
   CanaryRegressed, MalformedOutputSpike }`. `pub enum DriftVerdict { NoDrift, BehavioralDriftSuspected {
   signals: Vec<DriftSignal>, per_family_delta: Vec<(CanaryFamily, f64)>, affected_template_ids: Vec<String>,
   confirmed_by_fingerprint: bool }, BaselineCaptured }`. blake3 ONLY. NO .unwrap()/.expect().

2. **Add `pub fn record_baseline(ssb, identity, canary_set) -> DriftBaseline` + `pub fn check_model_drift(
   ssb, identity, last_baseline, canary_set) -> DriftVerdict` (~160 LOC).** record_baseline runs the canary
   set under the current model + ELAI_SEED, scores via the REUSED canary.rs FamilyEvalResult path, stores
   the per-family scores + output blake3 keyed to identity_key. check_model_drift: (a) if identity_key
   changed vs the baseline -> DriftSignal::FingerprintChanged (local lane) / set `confirmed_by_fingerprint`;
   (b) re-run the canary set, compute per-family delta vs the last-good baseline REUSING canary.rs's
   sample-count + non-determinism band (a delta WITHIN the band is NOT drift — the GD-4 FP guard);
   (c) read the F-4 deterministic pass-rate for the window — a malformed-output spike -> MalformedOutputSpike;
   (d) on any signal beyond the band -> BehavioralDriftSuspected with the affected ReasoningTemplate ids
   (templates whose domain matches a regressed family). Cloud lane NEVER sets confirmed_by_fingerprint=true.

3. **Wire the run-end identity check from the production entrypoint (~50 LOC in runtime_setup.rs).**
   When a run completes, read the RunArtifact identity tuple (G-2); if identity_key differs from the stored
   last-good baseline for this provider+lane, call check_model_drift; on BehavioralDriftSuspected emit ONE
   `model_drift_suspected` SSB row (signals + per_family_delta + affected_template_ids + confirmed_by_
   fingerprint). DETECTION default-ON, kill-switch ELAI_MODEL_DRIFT_DISABLE only (W-2); the AUTO-PIN action
   stays OFF (Task 7).

4. **CLI surface (~110 LOC across crates/cli/src/main.rs + a new model_drift_cmd.rs).**
   `elai model-drift baseline` (capture/refresh the baseline for the current model identity),
   `elai model-drift check` (run check_model_drift now, print NoDrift / BehavioralDriftSuspected{deltas,
   affected templates, fingerprint-confirmed?}), `elai model-drift show` (the stored baselines + last
   verdict). `--json` per harness convention. These are the W-3 entrypoints + W-5 observable_effect triggers.

5. **Add the `model_drift_suspected` SSB action_type (~60 LOC in ssb.rs, W-9).**
   `SsbActionType::ModelDriftSuspected` (Pattern-2 single-variant; the DriftSignal Kind is the discriminator:
   FingerprintChanged / CanaryRegressed / MalformedOutputSpike) + `ModelDriftSuspectedPayload { identity_key,
   signals: Vec<String>, per_family_delta: Vec<(String, f64)>, affected_template_ids: Vec<String>,
   confirmed_by_fingerprint: bool, baseline_captured_at: u64 }`. sql_str + parse + CHECK + round-trip golden
   + action_type_lock. SCHEMA bump named baseline+1 (NEVER hard-coded — see G-0 SCHEMA bump-sites).

6. **Frontend pair: ModelDriftPanel.svelte (~200 LOC svelte + ~110 LOC tauri) (W-4).**
   New ModelDriftPanel.svelte + Tauri `get_model_drift_status()` rendering: current model identity, the
   per-family baseline-vs-current bars, the active signals, the affected-template review queue, and a
   fingerprint-confirmed / behavioral-suspected badge (the honest cloud-lane label). cross_lang_diff.py PASS.

7. **(OPTIONAL, gated — NOT in the done-gate) Auto-pin-to-last-good action.**
   On a confirmed drift, recommend pinning the model to the last-good identity (feeding autonomous_gate's
   substitute_model path) — wire-dark-allowlisted (R9: type=experiment, owner, expiry, promotion_plan_id,
   smoke), default OFF, human-gated, NOT advertised as fail-closed. Surfacing is the done-gate; acting is the
   follow-on once the GD-4 FP rate is bounded on >=3 real runs.

8. **Decision doc workflow/reasoning/G-10-decisions.md (~100 LOC).** Records: (a) the local-vs-cloud
   guarantee asymmetry (GD-5; system_fingerprint opacity inherited from G-2 §6); (b) why DETECTION is
   default-ON but ACTION is human-gated (GD-4 FP friction; a false auto-swap > the drift); (c) the reuse of
   canary.rs's non-determinism band as the FP guard (genuine regression vs normal LLM non-determinism);
   (d) why this is COMPOSE-NOT-RESTATE of D-7/G-2/F-4 (GD-1), not a new regression engine.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/model_drift.rs::ModelIdentity
  - crates/orchestrator/src/model_drift.rs::DriftBaseline
  - crates/orchestrator/src/model_drift.rs::DriftSignal
  - crates/orchestrator/src/model_drift.rs::DriftVerdict
  - crates/orchestrator/src/model_drift.rs::record_baseline
  - crates/orchestrator/src/model_drift.rs::check_model_drift
  - crates/orchestrator/src/ssb.rs::SsbActionType::ModelDriftSuspected
  - crates/orchestrator/src/ssb.rs::ModelDriftSuspectedPayload
  - crates/cli/src/model_drift_cmd.rs::run
  - crates/dashboard-v2/src-tauri/src/commands_g10.rs::get_model_drift_status
  - crates/dashboard-v2/src/lib/windows/ModelDriftPanel.svelte
consumes_symbols:
  - crates/orchestrator/src/canary.rs::CanaryFamily (REUSE families + per-family delta — D-7, GD-1)
  - crates/orchestrator/src/canary.rs::FamilyEvalResult (REUSE the score + non-determinism band — GD-4 FP guard)
  - crates/orchestrator/src/held_out.rs::TaskId (REUSE the fixed canary fixtures)
  - crates/orchestrator/src/rra.rs::RunArtifact (G-2 system_fingerprint + model_pin — the identity input; G-2 dep)
  - crates/orchestrator/src/ssb.rs::Ssb (write_payload; baseline + delta rows recomputable under ELAI_SEED, GD-2)
  - crates/orchestrator/src/ssb.rs::TestOutcomePayload (F-4 stdout_hash / deterministic pass-rate — malformed-spike signal, GD-1 COMPOSES-WITH F-4)
  - crates/orchestrator/src/reasoning_template.rs (D-1 ReasoningTemplate — the review/revert target)
  - crates/orchestrator/src/runtime_setup.rs::build_production_hook_dispatcher (the run-end hook to tap)
  - crates/cli/src/main.rs (CLI dispatch)
reachable_from:
  - crates/cli/src/main.rs -> `elai model-drift check` -> check_model_drift -> DriftVerdict + (on drift) model_drift_suspected row
  - crates/cli/src/main.rs -> `elai model-drift baseline` -> record_baseline keyed to ModelIdentity
  - crates/orchestrator/src/runtime_setup.rs (run-end) -> identity-change check -> check_model_drift [PRODUCTION]
  - crates/dashboard-v2/src-tauri/src/commands_g10.rs::get_model_drift_status -> ModelDriftPanel.svelte [dashboard]
stub_ban:
  - "TODO: drift later" / "// canary stub" literals absent
  - NO fabricated FingerprintConfirmed for the cloud lane: a cloud-lane drift is BehavioralDriftSuspected
    with confirmed_by_fingerprint=false (GD-5; system_fingerprint opaque per G-2 §6)
  - NO auto-swap / auto-revert in the done-gate path (surface advisory only; Task 7 is wire-dark-allowlisted, human-gated — GD-4)
  - NO default-off ENABLE flag on DETECTION (kill-switch ELAI_MODEL_DRIFT_DISABLE only; default ON, W-2)
  - NO re-implementation of canary.rs's family/delta/band logic (REUSE it — GD-1; re-stating D-7 is a P0)
  - blake3 only for identity_key + canary-output hashes (no blake2/sha2)
  - NO .unwrap()/.expect() in model_drift.rs production paths
  - NO new dependency (canary.rs + ssb + rmp-serde all in-tree)
frontend_pair:
  svelte: crates/dashboard-v2/src/lib/windows/ModelDriftPanel.svelte
  tauri: crates/dashboard-v2/src-tauri/src/commands_g10.rs::get_model_drift_status
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- model-drift baseline   (under fixture model A) ; then cargo run -p elai-cli -- model-drift check   (under a fixture mock-provider model B returning a regressed canary output)"
  expect_ssb: "a model_drift_suspected row carrying signals (CanaryRegressed; +FingerprintChanged on the local lane), per_family_delta, affected_template_ids, confirmed_by_fingerprint in the live ssb.db"
  expect_artifact: "`model-drift check` prints BehavioralDriftSuspected with the per-family deltas + affected templates; a CLEAN re-check under model A (delta within the canary non-determinism band) prints NoDrift and emits NO row (the GD-4 FP guard); ModelDriftPanel shows the before/after canary scores + the fingerprint-confirmed/behavioral badge; the per-family delta re-computes byte-identically from the SSB stream under the pinned ELAI_SEED (GD-2)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/G-plan/G-10_model_drift_regression_canary.md --run-trigger
rg "pub fn check_model_drift|pub fn record_baseline|pub enum DriftVerdict|pub enum DriftSignal|pub struct ModelIdentity" .elai_cc/crates/orchestrator/src/model_drift.rs
rg "ModelDriftSuspected|model_drift_suspected" .elai_cc/crates/orchestrator/src/ssb.rs
rg "model_drift_suspected" .elai_cc/crates/orchestrator/tests/action_type_lock.golden
rg "CanaryFamily|FamilyEvalResult" .elai_cc/crates/orchestrator/src/model_drift.rs   # MUST be >0 (reuse, GD-1)
rg "check_model_drift|record_baseline" .elai_cc/crates/cli/src/model_drift_cmd.rs .elai_cc/crates/orchestrator/src/runtime_setup.rs
rg -c "blake2|sha2" .elai_cc/crates/orchestrator/src/model_drift.rs   # MUST be 0
rg -c "ELAI_MODEL_DRIFT_ENABLE" .elai_cc/crates/orchestrator/src/model_drift.rs   # MUST be 0 (kill-switch DISABLE only)
test -f .elai_cc/crates/dashboard-v2/src/lib/windows/ModelDriftPanel.svelte
python .claude/tools/cross_lang_diff.py
test -f workflow/reasoning/G-10-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: wiring_contract_check.py --run-trigger PASS; the observable_effect emitted a real
  model_drift_suspected row AND a clean re-check emitted none (the FP guard fired). cargo-green != DONE.
- [ ] W-2 default-ON detection: a run with a changed identity checks drift with NO env override;
  ELAI_MODEL_DRIFT_DISABLE=1 suppresses; no ENABLE flag. (Task 7 auto-pin is the only OFF surface, R9-allowlisted.)
- [ ] W-3 production entrypoint: the check fires from the build_production_hook_dispatcher run path AND from
  the `elai model-drift check` CLI (not test-only).
- [ ] W-5/GD-5 HONEST GUARANTEE: a local-lane fixture reports FingerprintChanged + CanaryRegressed
  (confirmed_by_fingerprint=true); a cloud-lane fixture reports BehavioralDriftSuspected with
  confirmed_by_fingerprint=false (NEVER a fabricated fingerprint-confirmed).
- [ ] W-6 dashboard E2E: ModelDriftPanel renders the deltas + signals + the honest badge via dashboard-smoker.
- [ ] GD-1 COMPOSES-WITH: model_drift.rs REUSES canary.rs CanaryFamily/FamilyEvalResult + reads G-2
  RunArtifact pins + F-4 stdout_hash — re-implements NONE of D-7 / G-2 / F-4 (rg proves the reuse calls).
- [ ] GD-2 REPRODUCIBLE-METRIC: the per-family delta re-computes byte-identically from the SSB stream under the pinned seed.
- [ ] GD-4 FP-GUARD: a re-check whose delta is within the canary non-determinism band returns NoDrift (no row);
  the FP rate is recorded on >=3 real orchestrate runs before any auto-pin action is enabled.
- [ ] model_drift_suspected round-trips + action_type_lock golden; SCHEMA bump named baseline+1.
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-orchestrator -p elai-cli -p contracts.
- [ ] No new dependency (canary.rs + ssb + rmp-serde + blake3 all in-tree).

## WIRING DONE-GATE (clause by clause)
- W-1: wiring_contract_check --run-trigger is ACCEPTANCE 1; check_model_drift (produced) is consumed by the
  run-end hook + the CLI (W-8), reached by the live run path (W-3); the FP-guard clean-run is part of the trigger.
- W-2: only ELAI_MODEL_DRIFT_DISABLE; detection default-ON; observable_effect runs with no override.
- W-3: build_production_hook_dispatcher run-end identity check + the model_drift_cmd CLI; not test-only.
- W-4: model_drift_suspected is human-surfaceable -> ModelDriftPanel + get_model_drift_status; cross_lang_diff PASS.
- W-5: observable_effect is runnable CLI -> a real SSB row + a live verdict + the FP-guard clean run; in-process, no RunPod.
- W-6: ACCEPTANCE verifies the deltas + badge THROUGH ModelDriftPanel via dashboard-smoker.
- W-7: canary.rs (CanaryFamily/FamilyEvalResult), held_out::TaskId, ssb ModelSwap/TierModelSwap wire-dark,
  autonomous_gate::substitute_model, reasoning_template::Domain ALL grep-confirmed 2026-06-02; rra.rs::RunArtifact
  is the G-2 dependency (confirm at authoring once G-2 lands); the validator re-greps.
- W-8: record_baseline/check_model_drift (produced) called by the run hook + CLI; ModelDriftSuspectedPayload
  emitted (Task 3) + consumed by the Tauri command (Task 6). No produced symbol caller-less.
- W-9: ONE ModelDriftSuspected variant (Pattern-2, DriftSignal Kind) through the golden lock; SCHEMA bump named baseline+1.
- W-10: entirely Tier-1 in-process (run the in-tree canary, score, compare, emit) — no network/subprocess/RunPod
  at gate time. The auto-pin ACTION (Task 7) is the SEPARATE wire-dark-allowlisted sub-phase, not in the done-gate.
- GD-1: COMPOSES-WITH D-7 (canary.rs), G-2 (RunArtifact pins), F-4 (stdout_hash); re-implements none.
  GD-3: a drift flag means "behavior shifted vs baseline," NOT "the new model is worse in absolute terms" —
  the delta is baseline-relative, the headline is never a model-capability rank. GD-4: the non-determinism
  band is the FP guard; auto-pin gated on a bounded FP rate. GD-5: local=fingerprint-confirmed,
  cloud=behavioral-suspected, never a fabricated fingerprint claim.

## Tests to add
- crates/contracts/tests/g10_drift_verdict_contract.rs (~150 LOC): (a) identity unchanged + scores within
  band -> NoDrift; (b) local-lane identity change + a regressed family -> BehavioralDriftSuspected with
  confirmed_by_fingerprint=true + the affected template ids; (c) cloud-lane regressed family ->
  BehavioralDriftSuspected confirmed_by_fingerprint=false (GD-5); (d) a malformed-output spike from a
  seeded F-4 pass-rate -> MalformedOutputSpike signal.
- crates/contracts/tests/g10_fp_guard_contract.rs (~110 LOC): a re-run whose per-family delta sits inside
  the canary non-determinism band returns NoDrift and emits NO model_drift_suspected row (the GD-4 guard);
  the delta re-computes identically from the seeded SSB stream (GD-2).
- crates/orchestrator/tests/g10_drift_emit_smoke.rs (~90 LOC): a run through the real run-end hook with a
  swapped fixture identity writes the model_drift_suspected row (the W-5 deterministic harness over the REAL path).

## Files touched
- CREATE: crates/orchestrator/src/model_drift.rs (~400 LOC: ModelIdentity + DriftBaseline + DriftSignal + DriftVerdict + record_baseline/check_model_drift)
- CREATE: crates/cli/src/model_drift_cmd.rs (~110 LOC: model-drift baseline / check / show)
- CREATE: crates/dashboard-v2/src-tauri/src/commands_g10.rs (~110 LOC: get_model_drift_status)
- CREATE: crates/dashboard-v2/src/lib/windows/ModelDriftPanel.svelte (~200 LOC)
- CREATE: crates/contracts/tests/g10_drift_verdict_contract.rs (~150 LOC)
- CREATE: crates/contracts/tests/g10_fp_guard_contract.rs (~110 LOC)
- CREATE: crates/orchestrator/tests/g10_drift_emit_smoke.rs (~90 LOC)
- CREATE: workflow/reasoning/G-10-decisions.md (~100 LOC)
- MODIFY: crates/orchestrator/src/ssb.rs (~60 LOC: ModelDriftSuspected variant + payload + sql_str + parse + CHECK + golden)
- MODIFY: crates/orchestrator/src/lib.rs (~2 LOC: pub mod model_drift;)
- MODIFY: crates/orchestrator/src/runtime_setup.rs (~50 LOC: run-end identity-change check)
- MODIFY: crates/cli/src/main.rs (~40 LOC: model-drift subcommand dispatch)
- MODIFY: crates/orchestrator/tests/action_type_lock.golden (~1 row)
- MODIFY: crates/dashboard-v2/src/lib/api/types.ts + a window registration (~10 LOC)
- (IF Task 7) MODIFY: wire-dark-allowlist.toml (~1 row: type=experiment, owner, expiry_date, promotion_plan_id)

## Citations / Grounding
- ELAI ground truth (grep-confirmed 2026-06-02 on elai.map): NO model-version pin type exists; all
  regression/canary/drift symbols (security_monotonicity.rs, staged_eval.rs, optimizer_admit.rs, canary.rs,
  goal_drift_canary.rs) key on ENDOGENOUS change; autonomous_gate.rs::substitute_model is restart/VRAM-
  triggered; ssb.rs ModelSwap/TierModelSwap are wire-dark (no production emitter). This is the seam G-10 fills.
- External prior art (arXiv IDs verified present via WebSearch 2026-06-02; effect-size numbers are MOTIVATION
  ONLY, NOT ELAI targets — GD-3, and require page-anchored quotes per memory feedback_paper_citation_verify
  before any headline use):
  - arXiv:2311.11123 "(Why) Is My Prompt Getting Worse? Rethinking Regression Testing for Evolving LLM APIs"
    — the canonical source; GPT-3.5-family case study: silent API upgrades cause significant performance
    deterioration; the PROMPT is a key factor in the behavioral change. (page-anchor pending ingest)
  - arXiv:2409.03928 "RETAIN: Interactive Tool for Regression Testing Guided LLM Migration" — data-slicing to
    identify behavioral differences ACROSS models + prompt refinement to address regressions.
  - arXiv:2601.22025 "When 'Better' Prompts Hurt: Evaluation-Driven Iteration for LLM Applications" — CANARY
    test cases detect a model personality shift; run the eval suite after ANY model update, even minor.
  - arXiv:2603.02601 "AgentAssay: Token-Efficient Regression Testing for Non-Deterministic AI Agent Workflows"
    — reports a silent provider update dropping agent routing accuracy 93% -> 71% (reported abstract figure;
    NOT an ELAI target).
  - arXiv:2505.02854 "Ensuring Reproducibility in Generative AI Systems: A Framework for Regression Testing
    and Open Datasets" (GPR-bench) — regression-test harness + open dataset precedent.
  - Industry observation (DEV/Tianpan practitioner writeups, InsightFinder-attributed; SECONDARY, weak):
    silent behavioral drift within ~90 days of deployment with a multi-week detection lag is reported as the
    norm for externally-hosted LLMs. Cite as motivation, never as a measured ELAI claim.
- Cloud-lane caveat inherited from G-2 §6: Anthropic system_fingerprint opacity -> cloud-lane signal is
  BEHAVIORAL, never fingerprint-confirmed (GD-5).
- CLAUDE.md HARD RULES: blake3 (identity_key + output hashes), no .unwrap()/.expect() in prod, anti-slopsquatting.

## Open questions / risks
- FP RATE (GD-4, the central risk): distinguishing a genuine model regression from normal LLM
  non-determinism requires the canary non-determinism band to be calibrated; an over-tight band cries drift
  on noise (the CodeGate-death failure mode). The band is REUSED from canary.rs (D-7) and must be bounded on
  >=3 real runs BEFORE the auto-pin action (Task 7) is enabled. Detection-only ships first.
- CLOUD-LANE DETECTABILITY: with system_fingerprint opaque, cloud drift is ONLY visible behaviorally — a
  small/cheap-to-run canary set is required so the check is affordable on a schedule; scope the canary set
  size in the decision doc (cost vs sensitivity tradeoff).
- BASELINE BOOTSTRAP: the first run for a given model identity has no baseline -> BaselineCaptured, not
  drift. A model identity seen for the first time must NOT flag drift (cold-start guard in check_model_drift).
- G-2 DEPENDENCY: RunArtifact's system_fingerprint/model_pin are the identity input; if G-2 has not landed,
  G-10 must read the pins from the determinism module directly (confirm the path at authoring, W-7) — but
  co-shipping after G-2 is cleaner.
- SCOPE DISCIPLINE: G-10 is DETECTION + SURFACING. It is NOT a model-selection/routing engine (that is
  E-31 / cloud_mode.rs) and NOT an auto-remediation loop — keep the action human-gated (GD-4).

## Next
G-10 closes the exogenous-drift seam left open after G-9 and composes cleanly with the spine: G-2 supplies
the identity pins it reads; D-7's canary.rs supplies the per-family delta it reuses; F-4 supplies the
malformed-output signal; D-1's template archive is the review/revert target. The natural follow-on is Task 7
(auto-pin-to-last-good via autonomous_gate::substitute_model — which would also give the long-wire-dark
ModelSwap/TierModelSwap SSB variants their first production emitter), gated on a bounded FP rate.

End of G-10.
