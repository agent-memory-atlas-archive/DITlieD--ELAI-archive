I-34 CRANE grammar gate production wiring — decisions (STAGE5-EXEC-E)

SCOPE DECISION: Option-B (observe + emit in production; enforcement routing is a forward dep)

The controller pre-decided Option-B. This changeset hooks call_grammar_gate into the
real model-completion path (RuntimeOrchestrator::try_complete) so GrammarGateEvent rows
land in ssb.db on every grammar-eligible live dispatch. On PostParseRejected or
PinViolation the gate emits the SSB row and a tracing::warn, then returns Ok normally --
the completion still reaches the caller unchanged. Enforcement routing (returning Err from
try_complete and routing to dispatch_conformal_refusal at D-16 with claim_score=0.0) is
registered as a forward dep under WIRE-DARK[I-34-enforce] in wire-dark-allowlist.toml.

Rationale: the enforcement routing requires (a) threading a CalibrationCorpus reference
into try_complete, (b) a new RuntimeOrchestratorError variant, and (c) updating all
try_complete callers to handle the new Err arm. That is a multi-crate ripple best
sequenced AFTER the observe-only gate has run on live data and the false-positive rate
on PostParseRejected is bounded. Shipping observe-only first is the safe incremental
approach per the GRC plan action > verify protocol.


FIELD ADDITIONS (3 total, all append-only, all backwards-compatible)

1. CompletionConfig::grammar_gate_schema_json: Option<String>
   - Location: crates/providers/src/runtime/types.rs
   - Serde: #[serde(default, skip_serializing_if = "Option::is_none")]
   - Purpose: JSON Schema string for re_validate_output. When None, the grammar gate is
     skipped entirely even if the decoding mode is crane-adaptive Grammar. The None default
     means all existing callers compile and behave identically with no code changes required.
   - Added to Default impl as None.

2. RuntimeConfig::grammar_manifest_path: Option<PathBuf>
   - Location: crates/orchestrator/src/runtime_config.rs
   - No serde on RuntimeConfig (the struct is not Serialize/Deserialize; RuntimeConfigPolicyView
     is, but it does not include factory-level fields). None default in the Default impl.
   - Purpose: optional explicit manifest path for pin verification. When None, the
     orchestrator falls back to current_dir() + elai_providers::default_manifest_path
     (which yields .elai/grammar_pins/manifest.toml relative to the project root).

3. PathBuf import added to runtime_config.rs (was not previously imported).


SYNTHETIC SESSION_ID

Format: "runtime_orchestrator::{role:?}" -- matches the convention used by
BackendCostAccrued at runtime_orchestrator.rs where `write_payload("runtime_orchestrator", 0, ...)`
is the existing pattern. The role discriminant makes the SSB row queryable per role without
any structural change to GrammarGateEventPayload.


ELIGIBILITY GATING

The gate runs only when ALL of the following are true:
  (a) self.ssb is Some (no SSB handle = no point emitting; skipped silently)
  (b) config.decoding_config is Some and its mode is DecodingMode::Grammar { crane_adaptive: true, .. }
  (c) config.grammar_gate_schema_json is Some (else: tracing::warn + skip, never silent)

Non-grammar completions (FreeText, Regex, JsonSchema, Grammar { crane_adaptive: false })
skip the gate entirely with zero overhead.


HELPER EXTRACTION: run_grammar_gate_if_eligible

try_complete is already ~227 lines (line 235 to line 459 before this changeset). Adding
the gate logic inline would push it further. Per the maintainability rules (MR-3, >80 lines =
extract), the gate logic was extracted into a private async fn
RuntimeOrchestrator::run_grammar_gate_if_eligible taking:
  (&Option<Arc<Ssb>>, AgentRole, &CompletionConfig, Option<&PathBuf>, &str)
The helper is static-dispatch safe (no dyn in signature) and returns () (non-fatal by design).
The call site in try_complete is a single .await call after the BackendCostAccrued emission
and before the return Ok(CompleteOutcome{...}).


WIRE-DARK[I-34-enforce] FORWARD DEP

In-code marker: a // WIRE-DARK[I-34-enforce]: comment appears in the run_grammar_gate_if_eligible
body at the call site. Backing row in wire-dark-allowlist.toml:
  name = "WIRE-DARK[I-34-enforce]", type = experiment, owner = maintainer,
  expiry_date = 2027-06-01, promotion_plan_id = I-34-enforce,
  smoke_test = crates/contracts/tests/i34_grammar_gate_contract.rs::grammar_gate_call_emits_crane_passed_event

The promotion plan (I-34-enforce) will: thread CalibrationCorpus into try_complete, add
RuntimeOrchestratorError::GrammarGateRejected, update try_complete callers, and verify
false-positive rate on live PostParseRejected observations before enabling enforcement.


NON-FATAL POSTURE

Any Err from call_grammar_gate (SSB write failure, manifest I/O error) is warn-logged
and the function returns without blocking the completion. This is consistent with the
fail-soft pattern already used for BackendCostAccrued, UnifiedPreambleApplied, and
HandoffPayloadCompressed SSB emissions in the same function.
