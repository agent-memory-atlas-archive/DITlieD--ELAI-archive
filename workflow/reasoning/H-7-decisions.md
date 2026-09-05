H-7 DECISION DOC — calibrated routing (architect/editor/verifier)
session s-66424 | 2026-06-04 | orchestrate H-plan WAVE 1 | composes-with B-37D RuntimeOrchestrator, E-31 RouteDecisionEvent

WHAT
The per-role RuntimeOrchestrator/PerRoleRuntime are built but UNWIRED; RuntimeRoutingPolicy::evaluate is inert;
RouteDecisionEvent has full SSB machinery but NO production emitter (the live leg is only CloudModeRouter
cap-enforcement). H-7 lights the routing TELEMETRY on the production path + documents the heavy behavior changes.

GROUNDED FINDINGS (2 explorers, live-verified; G run shifted lines)
- RouteDecisionEvent FULLY exists wire-dark (payload ssb.rs:25046 { kind: RouteDecisionEventKind(CloudRouteChosen|
  TierModelSwapped|LocalRouteKept), route, score:f32, ms:u64, reason, session_id, details_json, ts_ms }; all
  lockstep sites present). NO schema bump. LIVE SCHEMA 211/488 (G's churn; do NOT touch).
- GAP-1 (load-bearing): the dark emitter emit_route_decision_log (cloud_mode.rs:459-466) is tracing-only AND lives
  on route() (cloud_mode.rs:298) which has ZERO production callers. The PROD path is route_with_spend, called from
  the drive_task cap block (task_runner.rs:7503-7628) — which already emits RouteBlocked (:7573) + EscalateToCloud
  (:7617) via writer.write_payload(...).await (writer:&Arc<Ssb>, RUNNER_AGENT_ID, RUNNER_SUPERSTEP, now_ms all in
  scope). So the emit MUST be on the prod cap path, not route().
- The gate ELAI_ROUTE_DECISION_SSB_ENABLE (route_decision_ssb_enabled(), cloud_mode.rs:472) is ALREADY in
  wire-dark-allowlist.toml:55-60 (R9 satisfied) — NO allowlist edit needed.
- GAP-2: the allowlist's referenced smoke test crates/orchestrator/tests/route_decision_ssb_on_path.rs DOES NOT
  EXIST (only e31_route_decision_contract.rs). Per R8 / role-boundary the SMOKE TEST is AUDITOR-authored — flagged
  to the auditor, not written by this (feature) executor.

KEY DECISIONS
D1 (clean core — the deliverable): emit RouteDecisionEvent from the prod cap block (task_runner.rs, after the
   EscalateToCloud emit ~:7628), gated on crate::cloud_mode::route_decision_ssb_enabled(). Kind mapping:
   ExecMode::Local => LocalRouteKept; (Cloud|Auto) & cap_reached => LocalRouteKept (cloud requested, cap-blocked,
   kept local); (Cloud|Auto) & !cap_reached => CloudRouteChosen. reason from cap_period/exec_mode. Same
   writer.write_payload(...).await idiom + best-effort tracing::warn on Err (non-fatal — telemetry never blocks
   routing). ts_ms = now_ms (OsClock-derived, in scope). score=0.0 + ms=0 are OBSERVATIONAL PLACEHOLDERS — the
   calibrated score is HEAVY-1. This lights the dark RouteDecisionEvent producer on the REAL path (fixes GAP-1).
D2 (DEFERRED — heavy behavior changes, documented):
   - HEAVY-1 calibrated layer_failed: task_runner.rs:7613 layer_failed:1 is a hardcoded literal; the calibrated
     RouteLLM/AutoMix replacement primitive EXISTS (cost/automix_gate.rs:133 verify_before_escalate) but is wired
     only into bench/ablation, never the escalation path. GD-4: over-escalation must be bounded on >=3 runs before
     default-ON. Deferred (a net-new escalation seam + the GD-4 measurement).
   - HEAVY-2 per-role dispatch: RuntimeOrchestrator (runtime_orchestrator.rs:122) + PerRoleRuntime
     (per_role_runtime.rs:104) are built with ZERO task_runner dispatch refs. Wiring architect(cloud)/editor(7B)/
     verifier(auditor) = thread Arc<RuntimeOrchestrator> onto ResolvedConfig + a for_role call at the executor
     seam — a net-new production seam. Deferred.
D3 (gate posture): the RouteDecisionEvent emit fires only when ELAI_ROUTE_DECISION_SSB_ENABLE is on (allowlisted,
   E-31 promotion plan). This is OBSERVATIONAL telemetry (not behavior-changing), so default-OFF-allowlisted is
   correct until the calibrated routing (HEAVY-1/2) is built + GD-4-bounded; then E-31 promotes it.

DONE-GATE (Tier-1)
crates/orchestrator/tests/h7_route_decision_wiring.rs (source-wiring; placed in orchestrator/tests NOT the
G-blocked contracts crate — verifiable): task_runner.rs emits SsbPayload::RouteDecisionEvent on the prod cap path,
gated by route_decision_ssb_enabled(). cargo check -p elai-orchestrator clean (NOT G-blocked). Auditor authors
the route_decision_ssb_on_path.rs smoke (GAP-2). Test exec + live route capture + the calibrated routing = Tier-2.

RISK
R1 route_decision_ssb_enabled() visibility: if it is private (not pub(crate)) in cloud_mode.rs, promote it to
   pub(crate) (1-line) so task_runner can call it. (Auditor confirms; alternative: inline the env check.)
R2 RouteDecisionEventKind/Payload paths (crate::ssb::...) — confirmed pub + lockstep-present.
R3 noise: one RouteDecisionEvent per baby-step routing decision, gated default-OFF — acceptable telemetry volume.

DEFERRED: HEAVY-1 (calibrated layer_failed via automix_gate), HEAVY-2 (per-role dispatch), GAP-2 (auditor smoke
test), the calibrated score/ms, the GD-4 over-escalation bound, the default-ON promotion. NO commit.
