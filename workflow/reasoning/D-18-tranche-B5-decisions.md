D-18 tranche-B5 decisions (ObservableEffect REAL executor wire)

Session: s-22300
Sub-phase: tranche-B5 (D-18 ObservableEffect EffectSet REAL executor)
Predecessor: s-21636 (D-18 closure shipped smoke-beacon stub)
Predecessor: workflow/reasoning/D-18-decisions.md (DECISION-10 deferred real wire)

Scope
Replace the smoke beacon at task_runner.rs::dispatch_observable_effect_hook
(s-21636 shipped fixed-cardinality 2-row pack with effect_id="smoke-d18-effect")
with the real EffectSet executor wire: ObservableEffectDispatched + N
observable_effect_verified (via verify_effect_batch) + M goodhart_risk_flag
rows. NO new SsbActionType variant, NO SCHEMA_VERSION bump.

Decisions

DECISION-1 — F3 follow-up correction (handoff text inaccurate)
The s-21636 handoff F3 text ("D-18 Goodhart-flag SSB row emission — needs
new Pattern-2 variant in future plan") is INACCURATE per exploration
tranche-B5-D18 DRIFT row 2. The GoodhartRiskFlag SsbActionType variant +
GoodhartRiskFlagPayload + as_sql_str + from_str + write-encode + read-decode
are ALL already shipped at ssb.rs:6184, 8336, 9065, 12259, 13038, 18924,
26384, 30075. What was missing is the PRODUCER CALL that writes the row.
This tranche adds the producer; ssb.rs is untouched. SCHEMA_VERSION stays
at 193 and TOTAL_VARIANTS stays at 466.

DECISION-2 — singleton-flag policy via check_goodhart_risk_with_min
The D-18 spec §2 requires "min_effects_per_claim = 2" and the spec
acceptance test calls for "probe with 1 effect -> GoodhartRiskFlag SSB
row". The shipped check_goodhart_risk only detects duplicate (trigger.kind,
observation.kind) pairs — singletons and empty sets pass orthogonality
vacuously per observable_effect.rs:989. To meet spec while preserving the
existing observable_effect_integration.rs assertions, this tranche adds
check_goodhart_risk_with_min(set, expected_min) as a SEPARATE overload.
The existing fn stays unchanged for back-compat; the new producer at
dispatch_observable_effect_hook calls the overload with expected_min=2.
Risk kind for the new flag is "below_min_effects"; evidence text cites
the observed N and the expected_min. When the set is empty the synthetic
effect_id is "<empty_effect_set>" so the dashboard's per-effect join
stays stable.

DECISION-3 — NoOpSandboxExecutor as production default
A real SandboxExecutor implementor was missing from production code paths
(MockSandboxExecutor is test-only fixture-keyed). This tranche adds
NoOpSandboxExecutor in observable_effect.rs (pub, Default, Copy). The
no-op returns SandboxExecutorError::UnsupportedTrigger for every trigger
kind, which verify_observable_effect maps to VerificationOutcome::Skipped
via the existing run_and_compare error path (observable_effect.rs:625-637).
This keeps the wire end-to-end with zero production-side side effects.
Real container/process-isolated executors land in a follow-up tranche.

DECISION-4 — smoke beacon migration window (one release cycle)
The s-21636 smoke beacon used effect_id="smoke-d18-effect" for both the
ObservableEffectDispatched row and the SandboxObservation row. The real
wire uses sentinel effect_id "d18-executor-sentinel-seq{seq}-step0"
derived from seq + a fixed step_index (per-task EffectSet sourcing via
ResolvedConfig is DEFERRED — see DECISION-7). To avoid breaking
downstream queries / dashboards keyed off the legacy id, the legacy
ObservableEffectDispatched row + the legacy SandboxObservation effect_id
are retained behind a new env gate: ELAI_OBSERVABLE_EFFECT_SMOKE_LEGACY_ENABLE
(default OFF; truthy set {1, true, yes, on}, case-sensitive, no-trim).
The gate ships for ONE release cycle and is then removed; the stub registry
entry D18-smoke-beacon-legacy carries DEFERRED-FOR-REMOVAL until then.

DECISION-5 — env-gate matcher discipline (case-sensitive, no-trim)
Both new producer env gates (ELAI_OBSERVABLE_EFFECT_EXECUTOR_ENABLE — pre-
existing; ELAI_OBSERVABLE_EFFECT_SMOKE_LEGACY_ENABLE — new) use the same
strict matcher pattern as observable_effect.rs::effect_set_executor_enabled
(matches!(v.as_str(), "1" | "true" | "yes" | "on")). The matcher is
intentionally case-sensitive and does not trim whitespace; "TRUE" / " 1"
are NOT truthy. The contract test pins this to prevent accidental
widening of the truthy set in future patches.

DECISION-6 — gate retention (both env gates kept, not collapsed)
ELAI_OBSERVABLE_EFFECT_ENABLE gates the hook entry (no-op when off);
ELAI_OBSERVABLE_EFFECT_EXECUTOR_ENABLE gates the executor body (verify_
effect_batch + check_goodhart_risk_with_min). When the entry gate is on
but the executor gate is off, the producer still emits the
ObservableEffectDispatched row + the SandboxObservation row (taxonomy
continuity), but skips per-effect verification and goodhart emission.
Collapse to a single gate is DEFERRED to a follow-up plan once CI proves
the executor wire stable across enough sessions.

DECISION-7 — DEFER per-task EffectSet plumbing through ResolvedConfig
The exploration GAP-2 noted that dispatch_observable_effect_hook has no
EffectSet at the call site (task layer carries no claim_id/effect
declarations today). Two options were considered:
  (a) hardcode a sentinel EffectSet inline (chosen)
  (b) plumb a per-task EffectSet through ResolvedConfig (heavier change)
Option (a) is chosen for this tranche because the sentinel deliberately
triggers below_min_effects (N=1, expected_min=2), which proves the wire
end-to-end fires the goodhart_risk_flag row in gated CI runs. Option (b)
is DEFERRED to a follow-up plan that adds the [observable_effect.effects]
section to elai_cc.toml and threads it through ResolvedConfig.

DECISION-8 — DEFER new TriggerSpec / TypedObservation variants
The task brief mentioned file_write / subprocess_run / network_call /
env_set as candidate "effect types". Exploration GAP-6 concluded these
are OUT OF SCOPE for D-18: spec D-18 says nothing about adding them, and
each new TriggerSpec variant requires a new TypedObservation variant +
compare_observation arm + SandboxExecutor impl. The current closed
taxonomies (5 TriggerSpec variants + 5 TypedObservation variants) are
sufficient for the executor smoke + real wire. New trigger kinds land in
a follow-up tranche if they prove necessary.

DECISION-9 — entropy quarantine + 3-gate ordering preserved
The producer keeps the existing 3-gate ordering (resolved.observable_
effect_runtime.as_ref()? then ObservableEffectRuntimeConfig::is_enabled()
then runtime.runtime_config.enable). agent_id / superstep / now_ms remain
caller-injected at task_runner; observable_effect.rs never reads
SystemTime. ts_ms on GoodhartRiskFlagPayload is constructed at the
producer site (DECISION-1 of D-18-decisions.md restated).

Files touched
  .elai_cc/crates/orchestrator/src/observable_effect.rs (+ check_goodhart
    _risk_with_min, + NoOpSandboxExecutor)
  .elai_cc/crates/orchestrator/src/task_runner.rs (dispatch_observable_
    effect_hook body replaced)
  .elai_cc/crates/contracts/tests/d18_observable_effect_real_contract.rs
    (NEW, 6 tests)
  workflow/sessions/s-22300/stubs.md (D18-smoke-beacon-legacy added)

Env vars
  ELAI_OBSERVABLE_EFFECT_ENABLE — pre-existing; hook entry gate
  ELAI_OBSERVABLE_EFFECT_EXECUTOR_ENABLE — pre-existing; executor body gate
  ELAI_OBSERVABLE_EFFECT_SMOKE_LEGACY_ENABLE — NEW; smoke-beacon migration
    window gate (one release cycle then remove)

Schema invariants (verified, unchanged)
  SCHEMA_VERSION = 193 (ssb.rs:2661)
  SCHEMA_VERSION_PUBLIC = 193 (ssb.rs:2666)
  TOTAL_VARIANTS = 466 (action_type_lock.rs:453)
  No new SsbActionType variant; no new SsbPayload arm.

Open follow-ups
  * Per-task EffectSet plumbing through ResolvedConfig (DECISION-7).
  * Collapse ELAI_OBSERVABLE_EFFECT_ENABLE + ELAI_OBSERVABLE_EFFECT_EXECUTOR
    _ENABLE to a single gate (DECISION-6).
  * Remove ELAI_OBSERVABLE_EFFECT_SMOKE_LEGACY_ENABLE + the legacy emit
    branches after one release cycle (DECISION-4).
  * Real container/process-isolated SandboxExecutor implementors
    (DECISION-3).
