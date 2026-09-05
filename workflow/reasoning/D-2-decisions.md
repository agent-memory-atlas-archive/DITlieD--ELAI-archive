D-2 Decisions
=============

Session: s-62376 (host orchestrate run)
Plan: docs/thefinalglance/D-plan/D-2_ppm_sidecar_wiring.md
Sub-phase: PPM sidecar Rust-side adapter (Qwen2.5-Math-PRM-7B separable verifier)

Atomic SCHEMA bump: 65 -> 66 (D-2 contribution; later restacked to 67 by W2 E-1 in same wave).
Two new SSB variants land in one patch (9-tuple lockstep: SCHEMA_VERSION,
CHECK constraint, SsbActionType enum, as_sql_str arms,
ssb_action_type_from_str arms, SsbPayload variant, action_type() match arm,
test_payload arm, all_test_payloads list). Variant count 142 -> 144.
Variants: PpmScored, PpmGatingFailed.

Decisions
---------

DECISION-1 (native async fn in trait, no async-trait crate): SidecarPpmHook
declares its methods as `async fn` directly on the trait (Rust 1.75+
stable since Dec 2023, workspace-canonical per CLAUDE.md anti-stale rule).
The async-trait crate is REJECTED at the substrate layer for the same
reason it is rejected on the B-37 RuntimeAdapter trait surface and the
E-35 SidecarXGrammarHook trait surface: native async fn carries the
return type's Future shape on the trait method, the async-trait crate
forces a Pin<Box<dyn Future + Send>> heap allocation per call that we do
not need, and the workspace pins blake3 / rand>=0.9 / native async fn
together as the canonical "current Rust" toolbelt. Trade-off: native
async fn in trait makes the trait NOT dyn-compatible (cannot be used as
`Box<dyn SidecarPpmHook>`) because each impl's Future is a unique opaque
type — this drives DECISION-2.

DECISION-2 (generic PpmClient<H: SidecarPpmHook>, not dyn-dispatched):
because native async fn in trait disables dyn compatibility, PpmClient
takes the hook as a generic type parameter `PpmClient<H: SidecarPpmHook>`
with monomorphisation at compile time, NOT `PpmClient { hook: Box<dyn
SidecarPpmHook> }`. The test surface (MockSidecarPpmHook) and the future
production surface (HttpSidecarPpmHook, deferred follow-up) both fit the
generic without runtime dispatch overhead. Same pattern as E-35
SidecarXGrammarHook generic-over-hook structure. Trade-off: generic
parameter means PpmClient cannot be stored in a heterogeneous
collection of PpmClient<DifferentHook> values; this is acceptable
because the orchestrator holds exactly one PpmClient per task_runner
hot path and the hook impl is selected once at boot time via
RuntimeBackend::Ppm dispatcher (deferred follow-up).

DECISION-3 (no same-model fallback on Unavailable; propagate error):
when SidecarPpmHook::score_step returns PpmHookError::Unavailable
(remote sidecar down, port 8101 unreachable, model not loaded),
PpmClient propagates the error UP rather than falling back to scoring
with the same Qwen2.5-Coder model that drives the main reasoning path.
Rationale: the PPM (Qwen2.5-Math-PRM-7B) is a SEPARABLE verifier — its
value over self-grading is precisely that it is a different model
trained for the process-reward task; same-model fallback would silently
degrade the verifier into a self-grader and defeat the separation
invariant. The Unavailable error propagates to apply_ppm_gate which
emits PpmGatingFailed SSB row and lets the orchestrator decide whether
to skip the gate or abort the step (operator-visible failure beats
silent degradation). Trade-off: when the PPM sidecar is down, PPM
gating produces no signal at all; this is acceptable because PpmGatingFailed
is operator-visible AND the follow-up ELAI_PPM_ENABLE opt-in gate
lets operators run without PPM until the second sidecar is provisioned.

Architectural deviation flagged in audit
----------------------------------------

The auditor logged PASS-WITH-WARNINGS because PpmClient does NOT implement
RuntimeAdapter. The RuntimeAdapter trait (B-37) is the workspace-canonical
shape for runtime backends that produce LLM completions. PPM is NOT a
completion producer — it is a step-scoring verifier — so the
RuntimeAdapter shape (complete + stream + health_check returning
CompletionResponse / StreamChunk) does not fit. PpmClient defines a
narrower hook (SidecarPpmHook::score_step returning PpmScore) appropriate
to the verifier role. The follow-up `providers/src/runtime/sidecar.rs
PpmAdapter impl RuntimeAdapter` work is for the dashboard-discoverable
runtime listing surface (so `/runtime list` enumerates the PPM backend
alongside the completion backends), NOT for routing scoring calls through
RuntimeAdapter::complete.

5-slot wiring confirmation
---------------------------

All 5 wiring slots hit per the WIRING_CONTRACT block in the plan:
1. produces: PpmScored + PpmGatingFailed SsbActionType variants emitted
   via emit_ppm_score + apply_ppm_gate;
2. consumes: SidecarPpmHook::score_step request via PpmScoreRequest;
3. reachable: PpmClient::score_step + apply_ppm_gate called from
   orchestrator/src/sidecar_ppm_hook.rs::run_ppm_verifier_check;
4. stub_ban: zero TODO / placeholder / unimplemented! / .unwrap() in
   production paths (MockSidecarPpmHook is #[cfg(test)] only);
5. observable_effect: get_ppm_recent_scores Tauri command + PpmScoresPanel
   Svelte component renders live PPM score history with mock:false flag.

Tests: 20 PASS (sidecar_ppm_hook 8 + ppm_client 6 + ppm_observable_effect 6).
cargo check --workspace clean.

Known un-admitted gaps (logged as followups)
--------------------------------------------

1. workflow/reasoning/D-2-decisions.md was missing at audit time (this
   file resolves it post-audit).
2. workflow/papers/ paper-ingestor outputs missing for arXiv:2501.04519
   (rstar-math), arXiv:2504.16760 (latent-verifiers), arXiv:2501.03124
   (PRMBench). Spawn paper-ingestor in follow-up session before EXECUTING
   any PPM-scoring algorithm change that quotes effect-size numbers.

Followups (deferred to follow-up sessions)
------------------------------------------

- production HttpSidecarPpmHook (real HTTP impl against sidecar /score endpoint)
- python/elai_inference/server.py PPM worker per wire contract (PpmScoreRequest in, PpmScoreResponse out, port 8101)
- per_role_runtime.rs RuntimeBackend::Ppm variant + dispatcher
- providers/src/runtime/sidecar.rs PpmAdapter impl RuntimeAdapter (dashboard discovery surface only, not routing)
- elai ppm health CLI subcommand
- workflow/remote-setup/ start-ppm-sitecar.sh + dual-port tunnel (8100 reasoning + 8101 PPM)
- ELAI_PPM_ENABLE opt-in gate (env var; defaults off until second sidecar provisioned)

Pinned config (commented in elai_cc.toml):
- port: 8101
- verifier_id: "ppm_qwen25_math_prm_7b"

Addendum (D-2 continuation session — disk-blocked recovery)
-----------------------------------------------------------

DECISION-1 (continuation): SCHEMA_VERSION stays at 103 for this D-2
continuation slice. Plan §7 (PPM SSB action types) + context-package §9
LOW both endorse deferring the PpmUnavailable SSB variant atomic bump to
D-3, which will batch multiple variant additions (consensus-rule SSB rows
plus the PPM-side telemetry) into a single 9-tuple lockstep migration.
Until that lands, operators see PPM-offline state via the
`tracing::warn!("ppm_model_not_loaded", ...)` line emitted on
PpmHookError::Unavailable -- structured logs reach the operator without
an SSB row, and no regression risk exists because no consumer is wired
to a PpmUnavailable variant yet. The Rust HttpSidecarPpmHook returns
PpmHookError::Unavailable on closed-port connect failures and the
generic Transport(_) on the structured 503 path; both surface in
tracing in this slice.

DECISION-2 (continuation): PpmAdapter remains MISSING-BY-DESIGN per the
original s-62376 D-2-decisions DECISION-3 above. RuntimeBackend::Ppm
exists only for the `/runtime list` discovery surface; create_runtime
returns the NotImplemented sentinel that the runtime_adapter_contract
test now asserts. PerRoleRuntime::default() does NOT bind any AgentRole
to RuntimeBackend::Ppm -- the verifier role hits PpmClient directly via
the SidecarPpmHook trait, bypassing the RuntimeAdapter shape entirely
(see the original D-2-decisions architectural-deviation paragraph). This
shape preserves the verifier-only invariant while keeping PPM
operator-visible in the runtime listing.

DECISION-3 (continuation): PpmRuntimeConfig added in
orchestrator/src/runtime_config.rs with ELAI_PPM_ENABLE_ENV gating
(`ELAI_PPM_ENABLE=1` opts in). Off by default. When on, the config
requires ELAI_PPM_ENDPOINT or falls back to the canonical
http://127.0.0.1:8101 -- same default the
http_sidecar_ppm_hook::PPM_DEFAULT_BASE_URL constant pins. No new
external dependency is introduced -- reqwest is already a non-optional
workspace dep (Cargo.toml L36) and the HttpSidecarPpmHook construction
reuses it. This keeps the slopsquatting surface area at zero per
CLAUDE.md anti-slopsquatting rule.

Python sidecar /v1/ppm/score endpoint (handle_ppm_score in server.py)
-------------------------------------------------------------------

Inserted directly before the /search endpoint per the orchestration
instruction. Honours the wire contract from sidecar_ppm_hook.rs
(clauses C1-C7) and emits typed error envelopes the Rust WireErrorBody
maps to PpmHookError variants without lossy stringification:
  - empty_input -> PpmHookError::EmptyInput
  - invalid_mode -> PpmHookError::InvalidMode
  - parse_error -> PpmHookError::Parse
  - invalid_score -> PpmHookError::InvalidScore
  - shape_mismatch -> PpmHookError::ShapeMismatch
  - ppm_model_not_loaded (HTTP 503) -> classified as
    PpmHookError::Unavailable on the Rust side via the HTTP-status
    fallback when no structured tag matches.

Scoring math: per-step score = sigmoid(logit('+') - logit('-')) at the
step-separator token position, computed in one forward pass under
torch.inference_mode() (NOT torch.no_grad() per CLAUDE.md anti-stale
rule). Aggregation = arithmetic mean (latent-verifiers §A.2 ablation)
+ population variance. Each per-step score is clamped into [0, 1]
defensively (sigmoid float-rounding boundary).

Same-model fallback is BANNED: when the loaded model_name does not
contain any of the {"prm", "math-prm", "process-reward", "ppm"} hint
substrings, the handler returns 503 with error="ppm_model_not_loaded"
+ model_id rather than scoring with the policy SLM (Qwen2.5-Coder).
This enforces the heterogeneous-verifier invariant from
arxiv:2504.16760 T1 + CLAUDE.md stub_ban for D-2.

Integration test (tests/http_sidecar_ppm_hook_integration.rs)
-------------------------------------------------------------

Six tests against an in-process tokio TcpListener mock that speaks the
/v1/ppm/score wire on an ephemeral port:
  1. happy_path_round_trip_returns_well_formed_score
  2. server_503_with_unstructured_body_surfaces_as_transport_error
  3. structured_error_body_maps_to_typed_variant (422 + WireErrorBody
     -> InvalidScore round-trip)
  4. malformed_response_len_mismatch_returns_shape_mismatch (defensive
     C2 enforcement on the client side)
  5. retry_on_502_then_200_succeeds (retry counter asserted == 2)
  6. out_of_range_score_response_caught_client_side (defensive C1)

Mock uses tokio::sync::Mutex (NOT std::sync::Mutex) per the
anti-stale-training rule for tokio-driven shared state.
