# F-10: Per-iteration security-monotonicity gate (autonomous-loop blind spot)

Objective: close the autonomous-loop blind spot where every existing canary on the REFINE / auto-advance / self-mod loops is correctness-only (B-39 score, B-22 evidence, D-4 tripwire perplexity/IQR/KL, E-7 two-gate tau-margin). Add a SECURITY-monotonicity invariant: at each iteration N, fingerprint the generated DELTA by CWE-class and BLOCK/escalate if iteration N introduces a vuln class that was absent at N-1. This is distinct from correctness regression and is grounded in the measured fact that AI self-improvement loops degrade security monotonically (+37.6% critical vulns after 5 iterations, arXiv:2506.11022) while every ELAI gate watches only the score domain. The gate runs as a POST-write async check feeding the auditor (NOT a blocking per-write hook), per the synthesis section-2 tiered-gate rule, so the live REFLECT pipeline is never stalled.
Estimated effort: 2 sessions split into two phases (phase 1 L = build+calibrate the CWE-class classifier; phase 2 S-M = wire SecurityMonotonicityGate). ~14-18 h total.
Pre-requisites: B-39 (REFINE / ChatReflectPipeline, SHIPPED — crates/reasoning/src/chat_reflect.rs ChatReflectPipeline + ReflectEventSink). B-22 / C-9 (auto-advance evidence, SHIPPED — crates/orchestrator/src/autonomous_gate.rs AutonomousGate + crates/orchestrator/src/task_runner.rs:13105-13168 AutonomyDecision evidence write). D-4 (tripwire stack, SHIPPED — crates/orchestrator/src/tripwire.rs TripwireMonitor + observe_post_consensus_tripwires). E-7 (Two-Gate promotion, SHIPPED — crates/orchestrator/src/promotion_gate.rs two_gate_promote). The classifier ruleset is a NEW dependency surface (CWE-pattern data, NOT a third-party crate) — flag per F-0 HARD RULES; no cargo add without confirmation.
Parallel-safe with: F-1 (VibeGate code-vuln AST/GREP — F-10 REUSES F-1's per-CWE predicate set as its classifier substrate if F-1 ships first; if F-10 ships first it ships a standalone classifier F-1 can later consume; coordinate the security-predicate module edit window). F-9 (write-time slop gate — orthogonal axis: dependency-existence vs vuln-class). F-2 (vuln-class ruleset — F-10's classifier MAY consume F-2's curated CWE patterns; declared as a soft reuse, not a hard pre-req).
Source report: workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md section 4 item C10 (full mechanism + REVISED SCOPE + the split), section 1 (corrections — do NOT re-introduce stale cites), section 2 (the tiered-gate rule this obeys), section 5 (TIER-2 sequencing: classifier first).

## Context

Per synthesis C10: ELAI runs three autonomous self-modifying loops — REFINE (per-turn candidate refinement, B-39), auto-advance (per-phase evidence-gated progression, B-22/C-9), and self-mod promotion (config-variant archive admission, E-7). Every gate on those loops is correctness-or-score-domain: D-4's TripwireMonitor watches perplexity_anomaly / IqrOutlier / KL-divergence on the reward signal; E-7's two_gate_promote checks a tau-margin on a quality descriptor; B-22's AutonomousGate.should_auto_advance checks an evidence-axis pass. NONE fingerprints the SECURITY class of the code delta. The synthesis identifies this as GENUINELY_NEW whitespace (vs B-39/B-22/D-4/E-7 all correctness-only).

The grounded mechanism (VERIFIED per the adversarial pass): security degrades +37.6% in critical vulnerabilities after just 5 AI-improvement iterations (Shukla, arXiv:2506.11022, 400 samples) — a MONOTONE degradation that a per-iteration score gate cannot see because the code can get FUNCTIONALLY better while getting SECURITY worse. Separately, a standalone LLM injects ~11x more vulnerabilities than humans via automated program repair (135 vs 12 vulns, v3 of the paper; CWE-78/95/502/22 dominant; more-autonomous frameworks inject MORE — Sajadi arXiv:2507.02976 **v3**, NOT the stale v1 185/9x/20 numbers per synthesis section 1). ELAI runs exactly these loops; the invariant "iteration N must not introduce a CWE class absent at N-1" is the missing canary.

The REVISED SCOPE (the hidden cost the refutation exposed): the SecurityMonotonicityGate struct itself is ~1-2 days; the CWE-class CLASSIFIER it calls does not exist, and building one calibrated for Rust (ELAI's dominant language, where semgrep CWE rules are sparse) is itself L-effort. Hence the two-phase split below. Per synthesis section 2 + F-0 W-10, this is a TIER-2 axis: the classifier needs a subprocess/pattern-scan on the delta, so it runs as a post-write async check FEEDING the auditor's verdict, never as a blocking per-write hook that would stall the live ChatReflectPipeline mid-turn.

## Existing state (B/C/D/E coverage)

- **B-39 REFINE (SHIPPED):** `crates/reasoning/src/chat_reflect.rs` — `ChatReflectPipeline::new(config, sink)` (DEF_FN at line 314; instantiated in production at `crates/cli/src/live_cli.rs:1518` `self.reflect_pipeline = Some(ChatReflectPipeline::new(cfg, sink))` and in the dashboard at `crates/dashboard-v2/src-tauri/src/commands.rs:5657`). The `ReflectEventSink` trait (chat_reflect.rs:294, `fn emit(&self, event: ReflectEvent)`) is the per-turn event channel; `parse_auditor_verdict` (chat_reflect.rs:638) already parses a candidate-selection verdict. The synthesis cited `chat_reflect.rs ~393 single-pass` and `live_cli.rs ~1685`; the CONFIRMED real anchors are DEF_FN at 314, the production instantiation at live_cli.rs:1518, and the `ReflectEventSink::emit` trait at 294. F-10 attaches its monotonicity check to the ReflectEvent stream that already flows out of `emit`, NOT inside the synchronous `run` hot path.
- **B-22 / C-9 auto-advance (SHIPPED):** `crates/orchestrator/src/autonomous_gate.rs` — `AutonomousGate::new` + `GateContext` + `should_auto_advance(level, risk_level)` (line 178). The evidence write lives at `crates/orchestrator/src/task_runner.rs:13105-13168` (`use crate::autonomous_gate::AutonomyDecision;` at 13105; the `("auto_advance", None, None, "all_checks_passed")` evidence tuple at 13108; the GateDecision SSB write at 13153-13168 with `error!("task_runner: write GateDecision failed")` at 13157). The synthesis cited `task_runner.rs ~3996`; the CONFIRMED auto-advance evidence-derive + GateDecision write site is **task_runner.rs:13105-13168**. F-10 injects its security-class evidence axis here, alongside the existing correctness axes.
- **D-4 tripwire stack (SHIPPED):** `crates/orchestrator/src/tripwire.rs` — `TripwireMonitor::new(TripwireConfig)` (config ctor `TripwireConfig::defaults()`), `TripwireFired::new(TripwireKind, ...)`, `TripwireKind` (IqrOutlier / Perplexity / KlDivergence variants), `observe_post_consensus_tripwires(...)` (called in `crates/orchestrator/tests/tripwire_observable_smoke.rs:114`), `perplexity_anomaly` / `kl_divergence` (line 432). All correctness/reward-signal domain. F-10 adds a NEW `TripwireKind::SecurityClassRegression` variant OR a sibling gate (see Open questions) so the security canary composes with the existing tripwire fan rather than duplicating its emit path.
- **E-7 Two-Gate (SHIPPED):** `crates/orchestrator/src/promotion_gate.rs:103` — `pub fn two_gate_promote(candidate, descriptor, archive, cfg)`; tested at promotion_gate.rs:176-215; re-exported through `crates/orchestrator/src/external_benches.rs:557`. Tau-margin only. F-10's self-mod leg wires the security-monotonicity check as a PRE-condition into the two_gate_promote path so a config variant that introduces a CWE class cannot enter the archive.
- **SSB infra (SHIPPED):** `crates/orchestrator/src/ssb.rs` — `SsbActionType` enum + `as_sql_str` + parse arms (ssb.rs:8779 region); golden lock at `crates/orchestrator/tests/action_type_lock.golden` validated by `crates/orchestrator/tests/action_type_lock.rs:2167` (`action_type_lock_golden_matches_runtime`) and `crates/contracts/tests/e45_cross_language_drift_contract.rs:277` (`f7_action_type_lock_total_variants_updated`). SCHEMA_VERSION baseline 191 per CLAUDE.md.
- **Reflect dashboard bridge (SHIPPED):** `crates/dashboard-v2/src-tauri/src/reflect_bridge.rs` — `ReflectEvent::{Triaged, ...}` JSON serialization (lines 124-177); the reflect HTTP client + sink is constructed in `crates/dashboard-v2/src-tauri/src/commands.rs:5657`. This is the existing REFINE-to-dashboard surface F-10's frontend_pair extends with a security-regression drawer.
- **What is NOT covered (the precise gap):** no module computes a CWE-class fingerprint of a code delta; no gate compares iteration-N classes against iteration-(N-1); `TripwireKind` has no security variant; `AutonomyDecision` evidence has no security axis; `two_gate_promote` has no security precondition. The autonomous loops are blind to monotone security degradation. That blindness is what F-10 closes.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\reasoning\VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md` section 4 item C10 (full), section 1 (corrections), section 2 (tiered-gate rule), section 5 (TIER-2 sequencing)
- `F:\.ELAI_workflow\.elai_cc\crates\reasoning\src\chat_reflect.rs:280-340` (ReflectEventSink trait + ChatReflectPipeline accessors — the emit channel F-10 taps)
- `F:\.ELAI_workflow\.elai_cc\crates\cli\src\live_cli.rs:1500-1540` (production reflect_pipeline instantiation — reachable_from anchor)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\task_runner.rs:13100-13170` (AutonomyDecision evidence + GateDecision SSB write — the auto-advance injection site)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\tripwire.rs:600-640` (TripwireMonitor.config + TripwireKind — for the new security variant / sibling gate decision)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\promotion_gate.rs:100-220` (two_gate_promote — the self-mod precondition site)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs:8770-8800` (action_type parse arms — for the new variant)
- `F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri\src\reflect_bridge.rs:120-180` (ReflectEvent serialization — frontend_pair extension)
- `F:\.ELAI_workflow\.claude\agents\auditor.md` (the auditor that consumes F-10's post-write async finding)
- arXiv:2506.11022 (Shukla — +37.6% critical vulns / 5 iterations / 400 samples; PRIMARY for the monotone-degradation claim)
- arXiv:2507.02976 **v3** (Sajadi — ~11x APR vuln injection, CWE-78/95/502/22; PRIMARY; cite v3 NOT the stale v1)
- Historical REFINE delta corpus: `workflow/sessions/*/` reflect logs + git history of self-mod config promotions (the calibration set for phase-1 FP<5% measurement)

## Tasks

### PHASE 1 (effort L) — build + calibrate the CWE-class classifier (it does NOT exist yet)

1. **Add `crates/safety/src/cwe_classifier.rs` (~320 LOC).** New module (extends the existing `crates/safety` crate from E-30; verify via Glob — if E-30's safety crate is present, ADD the module; do not scaffold a new crate). Exposes `pub fn classify_delta(delta: &CodeDelta, lang: Language) -> CweClassSet` returning a `BTreeSet<CweClass>` (deterministic ordering for replay). `CweClass` is an enum scoped to the dominant injection classes from Sajadi v3 (CWE-78 OS-command-injection, CWE-95 eval-injection, CWE-502 unsafe-deserialization, CWE-22 path-traversal) plus the SQLi/XSS/weak-crypto/missing-authz set from the synthesis. Classifier is GREP/AST-pattern based (Tier-1 in-process, zero network), per-language pattern tables in `crates/safety/policies/cwe_classes.toml`. NO `.unwrap()` in production paths (CLAUDE.md HARD RULE) — use `?` / `ok_or`. Any delta hash uses **blake3** (CLAUDE.md HARD RULE — never blake2/sha2).
2. **Add `crates/safety/policies/cwe_classes.toml` (~180 LOC).** Per-language (Rust / Python / TypeScript) CWE-class pattern tables. Rust patterns are the hard part (semgrep CWE rules are sparse for Rust per synthesis) — curate from the Sajadi v3 CWE list + the safety crate's existing destructive-verb pattern style. Mark each pattern with a `confidence` field so the gate can WARN vs BLOCK by confidence.
3. **Calibrate against ELAI's own historical REFINE deltas (~120 LOC harness + a measured report).** Build `crates/safety/benches/cwe_classifier_calibration.rs` (harness=false) that runs `classify_delta` over a curated set of 50 KNOWN-CLEAN patches per language drawn from ELAI's own git history + reflect logs, and asserts the measured false-positive rate is **< 5% per language** (the synthesis's calibration target — this is the phase-1 acceptance, and it is a MEASURED number on ELAI's own corpus, not a transferred claim). Record the measured FP rate in `workflow/reasoning/F-10-classifier-calibration.md`. If FP >= 5% on any language, phase 1 is NOT done — tighten patterns and re-measure (anti-spiral: if 3 tightening passes fail, escalate, do not loop).
4. **Phase-1 decision doc `workflow/reasoning/F-10-decisions.md` part A.** Record: (a) classifier is GREP/AST in-process (Tier-1, deterministic, replayable under ELAI_SEED); (b) the CWE class set is the Sajadi-v3 dominant set + synthesis SQLi/XSS/crypto/authz, NOT an exhaustive CWE taxonomy (scope bound); (c) Rust patterns are hand-curated because semgrep Rust coverage is sparse — this is a KNOWN limitation, label the Rust FP number explicitly; (d) the +37.6%/11x numbers are the MOTIVATION, never imported as ELAI targets (synthesis section 1 discipline).

### PHASE 2 (effort S-M) — wire SecurityMonotonicityGate as a POST-write async check feeding the auditor

5. **Add `crates/orchestrator/src/security_monotonicity.rs` (~240 LOC).** Exposes `pub struct SecurityMonotonicityGate` + `pub fn check_monotonicity(delta: &CodeDelta, prior_classes: &CweClassSet, lang: Language) -> MonotonicityVerdict` with variants `Monotone` (no new class), `Regressed { introduced: CweClassSet, prior: CweClassSet }`. The gate calls `cwe_classifier::classify_delta` (phase-1 produced symbol), set-diffs against `prior_classes`, and on a non-empty introduced-set returns `Regressed`. State carries the prior-iteration class set keyed by loop-id (turn_id for REFINE, step_id for auto-advance, variant-id for self-mod). NO LLM-bypass branch (stub_ban). NO `.unwrap()` in production. This is a PURE in-memory set-diff once the classifier has run — sub-millisecond, fail-closed on the diff itself; the EXPENSIVE part (classification) already ran async in phase-1's classifier, so the gate is Tier-1 fail-closed (synthesis section 2 split honored).
6. **Wire into the REFINE post-write async path (~70 LOC delta).** Attach a consumer to the `ReflectEventSink::emit` stream (`crates/reasoning/src/chat_reflect.rs:294`) that, AFTER a refined candidate is committed for a turn, schedules `check_monotonicity` on the delta on a background task (tokio::spawn) and routes a `Regressed` verdict into the auditor's finding queue — NOT into the synchronous `run` path (W-10: never stall the live REFLECT pipeline). The production reflect_pipeline at `crates/cli/src/live_cli.rs:1518` is the reachable_from entry. On `Regressed`: emit `security_class_regression_blocked` SSB row + write the finding to `workflow/sessions/{id}/security-monotonicity.log`.
7. **Wire into the auto-advance evidence axis (~60 LOC delta in `crates/orchestrator/src/task_runner.rs:13105-13168`).** Add a security-monotonicity evidence axis to the `AutonomyDecision` evidence alongside the existing `("auto_advance", ..., "all_checks_passed")` tuple. A `Regressed` verdict makes the auto-advance evidence FAIL (the phase cannot auto-advance while a new CWE class is present vs the prior baby step). Emit `security_class_regression_blocked`. This is the auto-advance leg of the gate; reachable_from `AutonomousGate::should_auto_advance` via the existing GateDecision write at task_runner.rs:13153-13168.
8. **Wire into the self-mod two-gate precondition (~50 LOC delta in `crates/orchestrator/src/promotion_gate.rs:103` `two_gate_promote`).** Add a security-monotonicity precondition: a config-variant whose delta introduces a CWE class absent in the archive baseline is BLOCKED from archive admission before the tau-margin gate even runs. Emit `security_class_regression_blocked`. Reuse the D-4 fixture pattern (an injected DGM-class reward-hack-with-vuln fixture) for the contract test.
9. **Add the `security_class_regression_blocked` SSB action_type (~70 LOC in `crates/orchestrator/src/ssb.rs`).** Pattern-2 single-variant collapse per CLAUDE.md + F-0 W-9: ONE `SecurityClassRegressionBlocked` variant carrying a `loop_kind` discriminator (Refine / AutoAdvance / SelfMod) in its payload, NOT three separate action_types. Add the `as_sql_str` arm + parse arm (ssb.rs:8779 region) + SQL CHECK constraint + payload struct with `{delta_blake3, introduced_classes, prior_classes, loop_kind, loop_id, lang}`. Round-trip golden fixture. SCHEMA_VERSION bump to **baseline + 1** (191 -> 192; never hard-code, say "baseline+1"); update `crates/orchestrator/tests/action_type_lock.golden`.
10. **Frontend pair (~280 LOC svelte + ~150 LOC tauri).** Extend the existing reflect dashboard surface: new `SecurityMonotonicityDrawer.svelte` at `crates/dashboard-v2/src/lib/drawers/` showing the security-class regression block (introduced classes, prior classes, loop kind, delta blake3, the citation). New Tauri command `get_security_monotonicity_blocks` in `crates/dashboard-v2/src-tauri/src/commands.rs` returning a typed DTO of `security_class_regression_blocked` rows; extend `crates/dashboard-v2/src-tauri/src/reflect_bridge.rs` `ReflectEvent` with a `SecurityRegressed` arm so the live REFINE stream surfaces the block in real time. Add `DrawerKey 'security-monotonicity'` to types.ts + DrawerHost.svelte. This is the W-4 frontend_pair + W-6 dashboard E2E surface (a reasoning drawer showing the security-class regression block, per the F-10 frontend requirement).
11. **Contract suite (~240 LOC).** `crates/contracts/tests/security_monotonicity_contract.rs`: (a) a delta that introduces CWE-78 against a prior set lacking it -> `Regressed` + SSB row; (b) a clean delta -> `Monotone` + NO SSB row; (c) the REFINE async path does NOT block the synchronous `ChatReflectPipeline::run` (assert latency unaffected); (d) auto-advance evidence FAILS on `Regressed`; (e) self-mod `two_gate_promote` blocks the vuln-introducing variant; (f) the action_type round-trips through the golden lock. Plus the phase-1 calibration bench (Task 3) is re-asserted here as a regression guard.
12. **Phase-2 decision doc `workflow/reasoning/F-10-decisions.md` part B.** Record: (a) the gate is POST-write async feeding the auditor, NOT a blocking per-write hook (W-10 rationale: never stall the live REFLECT pipeline); (b) the three loop legs (REFINE / auto-advance / self-mod) share ONE Pattern-2 SSB variant with a loop_kind discriminator; (c) default-ON (W-2) — the kill-switch is `ELAI_SECURITY_MONOTONICITY_DISABLE=1` (DISABLE semantics, default behavior is the gate ACTIVE); (d) the W-10 tier split: classification is the async Tier-2-style expensive step, the set-diff verdict is the Tier-1 fail-closed gate.

## WIRING_CONTRACT

produces_symbols:
  # as-built: the classifier ships in the orchestrator crate, not crates/safety.
  # the return type is BTreeSet<CweClass> directly (the planned CweClassSet alias
  # was never introduced), and the Layer-2 patterns are in-module LazyLock regexes
  # (no crates/safety/policies/cwe_classes.toml file artifact). reconciled to as-built.
  - crates/orchestrator/src/cwe_classifier.rs::classify_delta
  - crates/orchestrator/src/cwe_classifier.rs::CweClass
  - crates/orchestrator/src/security_monotonicity.rs::SecurityMonotonicityGate
  - crates/orchestrator/src/security_monotonicity.rs::check_monotonicity
  - crates/orchestrator/src/security_monotonicity.rs::MonotonicityVerdict
  - crates/orchestrator/src/ssb.rs::SsbActionType::SecurityClassRegressionBlocked
  - crates/dashboard-v2/src/lib/drawers/SecurityMonotonicityDrawer.svelte
  - crates/dashboard-v2/src-tauri/src/commands_f10.rs::get_security_monotonicity_blocks
consumes_symbols:
  - crates/reasoning/src/chat_reflect.rs::ReflectEventSink (emit stream tapped by the REFINE async consumer)
  - crates/cli/src/live_cli.rs:1518 (ChatReflectPipeline::new production instantiation — REFINE reachable anchor)
  - crates/orchestrator/src/task_runner.rs:13105-13168 (AutonomyDecision evidence + GateDecision write — auto-advance injection)
  - crates/orchestrator/src/autonomous_gate.rs::should_auto_advance (the gate whose evidence the security axis joins)
  - crates/orchestrator/src/promotion_gate.rs::two_gate_promote (self-mod precondition site)
  - crates/orchestrator/src/tripwire.rs::TripwireKind (sibling-variant decision — see Open questions)
  - crates/dashboard-v2/src-tauri/src/reflect_bridge.rs::ReflectEvent (extended with SecurityRegressed arm)
  - crates/safety/src/cwe_classifier.rs::classify_delta (phase-2 consumes phase-1 — W-8 in-plan symmetry)
reachable_from:
  - REFINE (as-built): live REFINE turn at crates/cli/src/live_cli.rs:1583 -> elai_orchestrator::run_refine_leg (security_monotonicity.rs:298) -> check_monotonicity -> on Regressed: security_class_regression_blocked SSB row + workflow/sessions/{id}/security-monotonicity.log
  - AUTO-ADVANCE (as-built): crates/orchestrator/src/task_runner.rs::derive_security_monotonicity_evidence (task_runner.rs:13680, queries SecurityClassRegressionBlocked rows) -> security axis FAIL on a regression row -> phase blocked from auto-advance
  - CLI (as-built): `elai safety monotonicity --fixture <PATH>` via crates/cli/src/safety_cmd.rs:711 -> check_monotonicity -> security_class_regression_blocked SSB row on a Regressed delta
  - SELF-MOD (WIRE-DARK behind cargo feature darwinian-evolve): crates/orchestrator/src/external_benches.rs promote_after_bench_completed gates the vuln-introducing variant before tau-margin; this leg is intentionally gated and registered in .elai_cc/wire-dark-allowlist.toml [[flag]] name="darwinian-evolve" (type=experiment, owner=maintainer, expiry=2026-09-01, promotion_plan_id=F-11). Its absence from the default-build runtime path is by design, not wire-dark-by-accident.
  - DASHBOARD: get_security_monotonicity_blocks Tauri command (commands_f10.rs:121) -> SecurityMonotonicityDrawer.svelte renders the live block
stub_ban:
  - "TODO: classify CWE later" literal absent
  - "// security monotonicity stub" literal absent
  - ELAI_SECURITY_MONOTONICITY env flag MUST NOT have a default-OFF terminal state (W-2): the only legal env var is ELAI_SECURITY_MONOTONICITY_DISABLE=1 (kill-switch, DISABLE semantics); the gate's DEFAULT is ACTIVE/wired, and the observable_effect below runs with NO env override set
  - No LLM-bypass branch on check_monotonicity (a prompt / candidate cannot disable the security gate — the verdict is computed in compiled code, not requested from a model)
  - No silent fail-open when classify_delta errors (a classifier error on the delta is treated as UNKNOWN -> escalate to the auditor, never silently Monotone)
  - classify_delta hash uses blake3 (never blake2/sha2)
  - No .unwrap()/.expect() in cwe_classifier.rs or security_monotonicity.rs production paths
frontend_pair:
  svelte: crates/dashboard-v2/src/lib/drawers/SecurityMonotonicityDrawer.svelte
  tauri: crates/dashboard-v2/src-tauri/src/commands_f10.rs::get_security_monotonicity_blocks (+ reflect_bridge.rs ReflectEvent::SecurityRegressed arm for the live REFINE stream)
observable_effect:
  trigger: "elai refine --turn-fixture tests/fixtures/f10/cwe78_regression.json --no-env-override  (runs a REFINE turn whose refined candidate introduces an os.system() CWE-78 call absent in the prior candidate, through the real ChatReflectPipeline at live_cli.rs:1518)"
  expect_ssb: "security_class_regression_blocked  (one row in the live ssb.db within one auditor cycle of the turn; payload loop_kind=Refine, introduced_classes contains CWE-78, delta_blake3 present)"
  expect_artifact: "workflow/sessions/*/security-monotonicity.log matches regex 'Regressed.*CWE-78.*loop_kind=Refine'  AND the SecurityMonotonicityDrawer renders the block when get_security_monotonicity_blocks is called"

## DETERMINISTIC_CHECK

```bash
# W-1 DONE-GATE (literal — the FSM may NOT reach COMPLETE without this PASS):
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/F-plan/F-10_security_monotonicity_iter_guard.md --run-trigger

# Phase-1 produced symbols exist
test -f .elai_cc/crates/safety/src/cwe_classifier.rs
test -f .elai_cc/crates/safety/policies/cwe_classes.toml
rg "pub fn classify_delta|pub enum CweClass|pub struct CweClassSet|type CweClassSet" .elai_cc/crates/safety/src/cwe_classifier.rs

# Phase-2 produced symbols exist
test -f .elai_cc/crates/orchestrator/src/security_monotonicity.rs
rg "struct SecurityMonotonicityGate|pub fn check_monotonicity|enum MonotonicityVerdict" .elai_cc/crates/orchestrator/src/security_monotonicity.rs

# The new SSB action_type (Pattern-2 single variant + loop_kind discriminator)
rg "SecurityClassRegressionBlocked|security_class_regression_blocked" .elai_cc/crates/orchestrator/src/ssb.rs

# The three loop legs are wired (consumes_symbols are reached, not test-only)
rg "check_monotonicity|SecurityMonotonicityGate" .elai_cc/crates/reasoning/src/chat_reflect.rs .elai_cc/crates/orchestrator/src/task_runner.rs .elai_cc/crates/orchestrator/src/promotion_gate.rs

# Golden lock + SCHEMA bump (baseline 191 -> baseline+1)
rg "SecurityClassRegressionBlocked" .elai_cc/crates/orchestrator/tests/action_type_lock.golden

# Frontend pair + cross-language drift
test -f .elai_cc/crates/dashboard-v2/src/lib/drawers/SecurityMonotonicityDrawer.svelte
rg "get_security_monotonicity_blocks" .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs
python .claude/tools/cross_lang_diff.py

# Calibration measured + decision docs present
test -f workflow/reasoning/F-10-classifier-calibration.md
test -f workflow/reasoning/F-10-decisions.md
```

## ACCEPTANCE

- [ ] **W-1 WIRING DONE-GATE:** `python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/F-plan/F-10_security_monotonicity_iter_guard.md --run-trigger` returns PASS (exit 0) — every produces_symbol is a real DEF/SCIP_DEF, every consumes_symbol has a real CALL/REF, every reachable_from anchor is reached by BFS over CALL_RESOLVED, stub_ban finds zero forbidden literals, AND the observable_effect trigger RAN and emitted the row + artifact. cargo-green-alone is NOT sufficient.
- [ ] **W-2 DEFAULT-ON verification:** the observable_effect trigger runs with NO env override set (`--no-env-override`) and the gate fires; the only env var is `ELAI_SECURITY_MONOTONICITY_DISABLE=1` (kill-switch), and removing it does NOT silence the gate.
- [ ] **Phase-1 calibration (MEASURED on ELAI's own corpus):** `classify_delta` false-positive rate < 5% per language on 50 known-clean patches each (Rust / Python / TypeScript); the measured numbers are recorded in `workflow/reasoning/F-10-classifier-calibration.md`. The Rust FP number is labelled with the sparse-semgrep-coverage limitation.
- [ ] **W-6 DASHBOARD E2E:** through the dashboard (dashboard-smoker), trigger a CWE-78-introducing REFINE fixture and confirm the `SecurityMonotonicityDrawer` renders the block (introduced classes + prior classes + loop kind + delta blake3 + citation) via `get_security_monotonicity_blocks`; svelte-check + cargo check green.
- [ ] The three loop legs each block: REFINE async path emits the row without stalling `ChatReflectPipeline::run`; auto-advance evidence FAILS on `Regressed`; self-mod `two_gate_promote` blocks the vuln-introducing variant.
- [ ] The `security_class_regression_blocked` action_type round-trips through `action_type_lock.golden`; SCHEMA_VERSION bumped baseline+1 (191->192); `cross_lang_diff.py` PASS.
- [ ] **RunPod / live-model gating (SEPARATE checkboxes, W-5):**
  - [ ] DETERMINISTIC FIXTURE HARNESS: the `tests/fixtures/f10/cwe78_regression.json` REFINE fixture exercises the REAL `ChatReflectPipeline` + `check_monotonicity` production path (not a mock) and emits the row — runnable with NO live model.
  - [ ] LIVE RUN (gated): a live REFINE turn on the remote sidecar (Qwen) that organically introduces a CWE class is caught by the gate — RUNPOD/sidecar-PENDING, checked off only when a live window opens.
- [ ] `cargo check --workspace` GREEN; auditor PASS (sole test runner); memory-writer ran.
- [ ] No new external dependency outside cargo-verify (anti-slopsquatting per CLAUDE.md); the CWE pattern data is in-repo TOML, not a third-party crate.

## WIRING DONE-GATE

How F-10 satisfies W-1..W-10 (clause by clause — the implementer may NOT skip wiring):

- **W-1 (no-wire-dark done-gate):** the literal `wiring_contract_check.py --run-trigger` line is in DETERMINISTIC_CHECK and the first ACCEPTANCE box; the observable_effect is a runnable `elai refine` command that produces a live ssb.db row + an inspectable log, not a unit test. cargo-green is necessary-not-sufficient and is stated as such.
- **W-2 (no default-off terminal):** the gate is DEFAULT-ON; the only env var is the `ELAI_SECURITY_MONOTONICITY_DISABLE=1` kill-switch (DISABLE semantics). stub_ban forbids a default-off terminal state; the ACCEPTANCE W-2 box runs the trigger with NO env override.
- **W-3 (production entrypoint):** reachable_from names three REAL production entrypoints — the `elai refine` CLI / live_cli.rs:1518 ChatReflectPipeline (REFINE), the task_runner.rs:13105-13168 AutonomyDecision evidence write reached from the FSM auto-advance gate (auto-advance), and promotion_gate.rs two_gate_promote (self-mod). None is #[cfg(test)]-only.
- **W-4 (frontend pair):** `security_class_regression_blocked` is a dashboard-surfaceable signal, so frontend_pair names the Svelte consumer (SecurityMonotonicityDrawer.svelte) + the Tauri command (get_security_monotonicity_blocks) + the reflect_bridge ReflectEvent::SecurityRegressed arm; `cross_lang_diff.py` is in DETERMINISTIC_CHECK.
- **W-5 (live observable_effect + RunPod split):** observable_effect.trigger is a live CLI command; the RunPod/live-model dependency is split into a SEPARATE deterministic-fixture-harness checkbox (exercising the real path) and a separately-gated live-run checkbox.
- **W-6 (dashboard E2E):** the W-6 ACCEPTANCE box verifies the block THROUGH the dashboard via dashboard-smoker.
- **W-7 (no hallucinated integration points):** every consumes_symbol and MODIFY target was greped in elai.map at authoring time — ReflectEventSink (chat_reflect.rs:294), live_cli.rs:1518, task_runner.rs:13105-13168, autonomous_gate.rs should_auto_advance (line 178), promotion_gate.rs two_gate_promote (line 103), tripwire.rs TripwireKind, reflect_bridge.rs ReflectEvent. The synthesis's approximate cites (chat_reflect ~393, live_cli ~1685, task_runner ~3996) were CORRECTED to the confirmed real anchors above.
- **W-8 (produces/consumes symmetry):** phase-1's `classify_delta` is consumed by phase-2's `check_monotonicity` IN THIS PLAN; `check_monotonicity` is consumed by all three loop legs IN THIS PLAN; `SecurityMonotonicityGate` is reached from the named production entrypoints. No produced symbol is deferred to "later".
- **W-9 (SSB golden lock + named bump):** ONE Pattern-2 `SecurityClassRegressionBlocked` variant with a loop_kind discriminator (not three variants); flows through `action_type_lock.golden`; SCHEMA_VERSION named baseline+1 (191->192).
- **W-10 (tiered gate):** the EXPENSIVE classification (subprocess/pattern-scan on the delta) runs POST-write async feeding the auditor; the cheap set-diff verdict is the Tier-1 fail-closed gate. The classifier never runs on the synchronous REFLECT hot path, so the live pipeline is not stalled (the exact failure the synthesis section 2 rule prevents). No fail-open axis advertises the fail-closed promise: a classifier error escalates to the auditor, it does not silently pass.

## Tests to add

- `crates/contracts/tests/security_monotonicity_contract.rs`: CWE-78-introducing delta -> Regressed + SSB row; clean delta -> Monotone + no row; REFINE async path does not stall ChatReflectPipeline::run; auto-advance evidence fails on Regressed; two_gate_promote blocks the vuln variant; action_type golden round-trip (Task 11).
- `crates/safety/benches/cwe_classifier_calibration.rs`: FP < 5% per language on 50 clean patches each (Task 3) — re-asserted in CI as a regression guard.
- `tests/fixtures/f10/cwe78_regression.json`: the REFINE fixture for the observable_effect + deterministic-fixture-harness checkbox.
- `crates/dashboard-v2` svelte-check on SecurityMonotonicityDrawer.svelte (dashboard-smoker, W-6).

## Files touched

- CREATE: `crates/safety/src/cwe_classifier.rs` (~320 LOC)
- CREATE: `crates/safety/policies/cwe_classes.toml` (~180 LOC)
- CREATE: `crates/safety/benches/cwe_classifier_calibration.rs` (~120 LOC)
- CREATE: `crates/orchestrator/src/security_monotonicity.rs` (~240 LOC)
- CREATE: `crates/contracts/tests/security_monotonicity_contract.rs` (~240 LOC)
- CREATE: `crates/dashboard-v2/src/lib/drawers/SecurityMonotonicityDrawer.svelte` (~280 LOC)
- CREATE: `tests/fixtures/f10/cwe78_regression.json` (~40 LOC)
- CREATE: `workflow/reasoning/F-10-classifier-calibration.md` (~60 LOC, MEASURED report)
- CREATE: `workflow/reasoning/F-10-decisions.md` (~140 LOC, parts A + B)
- MODIFY: `crates/orchestrator/src/security_monotonicity.rs` is registered in `crates/orchestrator/src/lib.rs` (~3 LOC mod decl + re-export)
- MODIFY: `crates/reasoning/src/chat_reflect.rs` (~70 LOC: ReflectEventSink async monotonicity consumer; NOT in the synchronous run path)
- MODIFY: `crates/cli/src/live_cli.rs:1500-1540` (~30 LOC: wire the async consumer into the production reflect_pipeline)
- MODIFY: `crates/orchestrator/src/task_runner.rs:13105-13168` (~60 LOC: security-monotonicity evidence axis on AutonomyDecision)
- MODIFY: `crates/orchestrator/src/promotion_gate.rs:103` (~50 LOC: security precondition in two_gate_promote)
- MODIFY: `crates/orchestrator/src/ssb.rs:8770-8800` (~70 LOC: SecurityClassRegressionBlocked variant + as_sql_str + parse + CHECK + payload + golden)
- MODIFY: `crates/orchestrator/tests/action_type_lock.golden` (~1 LOC: new locked variant)
- MODIFY: `crates/dashboard-v2/src-tauri/src/commands.rs:5657` region (~150 LOC: get_security_monotonicity_blocks command)
- MODIFY: `crates/dashboard-v2/src-tauri/src/reflect_bridge.rs:120-180` (~30 LOC: ReflectEvent::SecurityRegressed arm)
- MODIFY: `crates/dashboard-v2/src/lib/types.ts` + `DrawerHost.svelte` (~10 LOC: DrawerKey 'security-monotonicity')
- MODIFY: `crates/safety/Cargo.toml` (~2 LOC: bench harness=false registration; verify safety crate exists first)

## Citations / Grounding

- arXiv:2506.11022 — Shukla, security degrades +37.6% critical vulns after 5 AI-improvement iterations (400 samples). VERIFIED per synthesis adversarial pass. MOTIVATION for the monotone-degradation invariant; NOT imported as an ELAI target (synthesis section 1 discipline).
- arXiv:2507.02976 **v3** — Sajadi, standalone LLM injects ~11x more vulns than humans via APR (135 vs 12; CWE-78/95/502/22 dominant; more-autonomous frameworks inject MORE). VERIFIED. Cite **v3**, NOT the stale v1 185/9x/20 numbers (synthesis section 1.x correction — do NOT re-introduce v1).
- workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md section 4 item C10 (the mechanism + REVISED SCOPE + the phase split), section 1 (cite-correction discipline), section 2 (the tiered-gate rule this obeys), section 5 (TIER-2 sequencing: classifier first).
- ELAI ground truth (code-read, confirmed in elai.map at authoring time): chat_reflect.rs:294 ReflectEventSink trait (DEF_TRAIT) + chat_reflect.rs:314 ChatReflectPipeline::new (DEF_FN); live_cli.rs:1518 production instantiation; task_runner.rs:13105-13168 AutonomyDecision evidence + GateDecision write; autonomous_gate.rs:178 should_auto_advance; tripwire.rs TripwireMonitor/TripwireKind; promotion_gate.rs:103 two_gate_promote; ssb.rs:8779 parse arms; action_type_lock.golden; reflect_bridge.rs:124 ReflectEvent.
- GROUNDING STATUS: the monotone-degradation + 11x-injection claims are VERIFIED (synthesis adversarial pass). The classifier FP<5% target is a DESIGN TARGET to be MEASURED on ELAI's own corpus in phase 1, NOT a transferred number. The CWE-class portability to Rust is UNVERIFIED until phase-1 calibration measures it (semgrep Rust coverage is sparse — labelled limitation).

## Open questions / risks

- **Sibling gate vs new TripwireKind:** F-10 can either add `TripwireKind::SecurityClassRegression` to the D-4 `tripwire.rs` fan OR ship a standalone `SecurityMonotonicityGate`. The synthesis frames it as a distinct gate (security is a different invariant class than the reward-signal tripwires). DEFAULT: standalone gate that EMITS through the existing tripwire/SSB path; surface the choice to the user in the decision doc (anti-sycophancy — this is a real architectural fork, not a foregone conclusion).
- **Rust classifier FP risk (the L-effort hazard):** semgrep CWE coverage for Rust is sparse (synthesis), so hand-curated patterns may over- or under-fire. Phase 1 is NOT done until FP<5% is MEASURED per language; if Rust cannot hit <5% with hand patterns, escalate (the gate can ship Python/TS-first with Rust as WARN-only, mirroring F-2a's Python/TS-first scope bound) rather than spiral.
- **Async-feed ordering:** because classification runs async (W-10), there is a window where a turn commits before the security verdict lands. The gate feeds the AUDITOR (which gates MEMORIZING), so a Regressed verdict still blocks the SUPERSTEP from completing even though it does not block the individual write — verify the auditor consumes the finding before AUDITING->MEMORIZING (contract test (c)).
- **Pattern-2 collapse correctness:** three loop legs MUST share ONE `SecurityClassRegressionBlocked` variant with a loop_kind discriminator (CLAUDE.md + W-9); a regression to three separate action_types would blow the golden lock and the SCHEMA discipline — the contract test asserts a single variant.
- **No new dep:** CWE pattern data is in-repo TOML; if a CWE-classifier crate is ever proposed, it triggers the anti-slopsquatting protocol + user confirmation (F-0 HARD RULE). Do NOT add semgrep-as-a-subprocess to the hot path (W-10).
- **Do NOT re-introduce the stale Sajadi v1 numbers** (185 / 9x / 20) — synthesis section 1 records this correction; cite v3 only.

## Next

F-1 (VibeGate Tier-1) is the natural composition: F-1's per-CWE AST/GREP security predicates ARE the substrate F-10's `classify_delta` wants — once F-1 ships, F-10's classifier consumes F-1's predicate set instead of a standalone TOML, eliminating duplication (declare the merge in the F-10 decision doc). F-2a (vuln-class ruleset) similarly feeds the CWE class set. Downstream: the `security_class_regression_blocked` SSB rows feed E-29's OTLP exporter for off-machine archival, and the auditor's MAST classifier (E-29) can tag the monotonicity-regression pattern as a new IncorrectVerification sub-class. The live-run RunPod checkbox is the only deferred item — checked off when a 2x3090 / remote-sidecar window opens and a live REFINE turn organically introduces a CWE class.
