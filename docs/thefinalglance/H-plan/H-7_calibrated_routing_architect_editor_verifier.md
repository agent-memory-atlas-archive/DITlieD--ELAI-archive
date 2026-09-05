# H-7: Calibrated routing — architect / editor / verifier on the live cap-enforcement leg

STATUS: PLANNED (WAVE 1; in-process; BUILD — the heaviest Wave-1 session)

Objective: Smarter ORCHESTRATION, not a smarter model. The architect/editor split is +9.6pp on Aider (o1-mini
reasoner + GPT-4o editor), positive across every pairing. ELAI's per-role RuntimeOrchestrator + PerRoleRuntime are
BUILT but UNWIRED (C8, grep-confirmed 2026-06-03): zero task_runner dispatch references (deferred to B-37D);
RuntimeRoutingPolicy::evaluate (escalation_policy.rs:261) is doubly inert (default-OFF + explicitly observational —
it does NOT change the routed provider); RouteDecisionEvent has full machinery but NO production emitter (only
tracing::info); the only LIVE leg is CloudModeRouter cap-enforcement + EscalateToCloud. H-7 wires architect (cloud
flagship, reasons/plans) + editor (cheap 7B, cheaply-checkable edits) + independent-context verifier (the auditor)
on top of the live cap leg, replaces the hardcoded layer_failed:1 with a CALIBRATED external signal (RouteLLM/AutoMix-
style, NEVER the 7B's self-confidence), feeds evaluate a REAL RoutingContext, and emits RouteDecisionEvent. Strictly
cost-AND-correctness gated; the over-escalation rate is bounded on >=3 runs before default-on (GD-4).

Estimated effort: ~2 sessions (~10-14 h). In-process at gate time (the route decision + the emit); the cheap-worker
quality is measured over runs. W-10 Tier-1 whole (the routing decision is deterministic given the context).
Pre-requisites: B-37D RuntimeOrchestrator/PerRoleRuntime BUILT (per_role_runtime.rs:182, runtime_orchestrator.rs);
  E-31 RouteDecisionEvent SSB SHIPPED wire-dark (ssb.rs:8153); the CloudModeRouter cap leg LIVE (runtime_setup.rs:533-536).
Parallel-safe with: H-5/H-6/H-8 (disjoint). NOT parallel-safe with any plan editing task_runner routing or cloud_mode.rs.
Source: ELAI-EFFICIENCY-TRUST-DIRECTION-2026-06-03.md §10.7 (LEVER L7) + C1/C5/§2 (the orchestration brain); _grounding/H-GROUNDING-MAP.md C8 + the H-7 table. Doctrine: H-0_INDEX.md.

## Context

Route by an EXTERNAL CALIBRATED signal, never the 7B's self-confidence (the documented anti-pattern — an over-eager
editor that fails checks burns BOTH the cheap call and the cloud retry, net negative). RouteLLM (arXiv:2406.18665,
ICLR 2025) = 3.66x cost ratio (~73% cut) at 95% GPT-4 quality on MT-Bench, transfers across pairs without retraining;
AutoMix (arXiv:2310.12963) >50% with a POMDP that absorbs noisy self-verification. The cheap worker only saves cloud
tokens on steps it can ACTUALLY do (Model-Cascading-for-Code excludes any model <10% accuracy from a cascade; the
SWE-Protege 7B->42.4% gain is the cloud EXPERT's, not a free 7B's). Single-strong-agent + INDEPENDENT-context verifier
beats parallel code-writers (Science of Scaling: negative returns above 45% single-agent accuracy) — so do NOT wire
BSP for parallel code generation; the second cloud call is a VERIFIER (the auditor, separate context), not a parallel
writer. Strictly cost-AND-correctness gated: a cheap edit that fails an off-agent check is not cheaper.

## Existing state (B/C/D/E/F/G coverage)
- PerRoleRuntime (per_role_runtime.rs:182) — WIRE-DARK for dispatch. Held by GepaTemplateEvolution / RlaifJudge /
  runtime_config; ZERO task_runner refs; deferred to B-37D.
- RuntimeOrchestrator (runtime_orchestrator.rs ~120/140/255-373) — WIRE-DARK. Real fallback logic; every new() caller is
  non-dispatch (runtime_cmd inspector, measure bench, dashboard panel). NOT invoked by task_runner to route.
- RuntimeRoutingPolicy::evaluate (escalation_policy.rs:261) — WIRE-DARK. One prod caller cloud_mode.rs:448 (in
  emit_route_decision_log from CloudModeRouter::route :298); early-returns unless ELAI_ROUTE_DECISION_SSB_ENABLE=1 (:472);
  observational only (does NOT change RoutedProvider, 410-412); RoutingContext partly hardcoded (prior_audit_failures:0 :443,
  budget_remaining_usd:25.0 :444); DRIFT vram_detected_gb now real (detect_vram_gb :442, F13).
- emit_route_decision_log (cloud_mode.rs:416) — WIRE-DARK. Emits only tracing::info (459-466); no SSB row written.
- CloudModeRouter::route / route_with_spend (runtime_setup.rs:533-536) — SHIPPED. Cap-enforcement + EscalateToCloud LIVE
  (consumed task_runner.rs:7326/7371/7432/7496); the TIER-selection leg (evaluate) on the same call is the dark part.
- task_runner.rs:7606 — the hardcoded layer_failed:1 (EscalateToCloudPayload literal); the surrounding EscalateToCloud
  probe (:7496-7535) is wired and fires on real local-failure.
- HybridEscalationGate (cost/cloud_route.rs:292) + confidence::compute_score (:189) — TEST-ONLY (zero non-test callers).
- RouteDecisionEvent SSB (ssb.rs:8153, Pattern-2 43rd; 3-arm CloudRouteChosen/TierModelSwapped/LocalRouteKept) — WIRE-DARK. NO production write_payload.

## Reads (mandatory before coding)
- _grounding/H-GROUNDING-MAP.md C8 + the H-7 symbol table.
- ELAI-EFFICIENCY-TRUST-DIRECTION-2026-06-03.md §10.7 (L7) + C1/C5 (cloud reasons, cheap worker executes checkable steps;
  single>=multi; the verifier not a parallel writer) + the DO-NOT list (no 7B self-confidence, no BSP-for-code).
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\runtime_orchestrator.rs (the unwired orchestrator) + per_role_runtime.rs:182
- F:\.ELAI_workflow\.elai_cc\crates\providers\src\runtime\escalation_policy.rs:255-480 (evaluate + RoutingContext)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\cloud_mode.rs:290-470 (route + emit_route_decision_log)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\runtime_setup.rs:525-540 (the live CloudModeRouter wiring) + task_runner.rs:7490-7610 (the EscalateToCloud probe + layer_failed:1)
- CLAUDE.md HARD RULES + GD-4 (FP/over-escalation gate) + HD-1.

## Tasks (WAVE 1 — build calibrated role dispatch on the live cap leg)

1. **Wire role dispatch into task_runner (~120 LOC).** Have task_runner consult PerRoleRuntime/RuntimeOrchestrator to
   dispatch the architect (cloud flagship, reasons/plans the baby step), the editor (cheap tier / 7B for cheaply-checkable
   edits), and the verifier (the auditor, a SEPARATE context). The editor's output is gated by an OFF-AGENT check (the
   auditor / a test), not the editor's own confidence.

2. **Replace the hardcoded layer_failed:1 with a calibrated signal (~70 LOC).** At task_runner.rs:7606, replace the
   literal with a calibrated router signal (RouteLLM/AutoMix-style external calibrated layer). Either wire the existing
   HybridEscalationGate::evaluate (cost/cloud_route.rs:292, test-only today) or build the calibrated decision; route by
   an EXTERNAL signal, NEVER the 7B self-confidence.

3. **Feed evaluate a REAL RoutingContext + let it change the routed provider (~60 LOC).** Replace the hardcoded
   prior_audit_failures:0 / budget_remaining_usd:25.0 (escalation_policy.rs:443-444) with live values; let RuntimeRoutingPolicy::evaluate's
   RoutingDecision actually CHANGE the dispatched provider (today observational, :410-412), not just log.

4. **Emit RouteDecisionEvent (~50 LOC).** Make emit_route_decision_log (cloud_mode.rs:416) call Ssb::write_payload(RouteDecisionEvent)
   (today only tracing::info), and promote ELAI_ROUTE_DECISION_SSB_ENABLE off default-OFF (W-2). The route decision becomes an off-agent artifact.

5. **GD-4 over-escalation measurement (~30 LOC + a harness).** Measure the over-escalation rate (cheap-edit-then-cloud-retry
   churn) on >=3 real orchestrate runs before flipping default-on; record it. Gate on measured per-step-class pass-rate.

6. **Decision doc workflow/reasoning/H-7-decisions.md (~110 LOC).** Records: calibrated external signal not 7B
   self-confidence; NO BSP-for-code (verifier not parallel writer); cost-AND-correctness gating; the layer_failed:1 fix;
   the RoutingContext realness; the over-escalation gate; HybridEscalationGate wire-or-replace decision.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/task_runner.rs (role dispatch consulting PerRoleRuntime/RuntimeOrchestrator + the calibrated layer_failed replacement)
  - crates/orchestrator/src/cloud_mode.rs (emit_route_decision_log -> Ssb::write_payload(RouteDecisionEvent))
  - crates/providers/src/runtime/escalation_policy.rs (a real RoutingContext + a provider-changing RoutingDecision)
consumes_symbols:
  - crates/orchestrator/src/runtime_setup.rs::CloudModeRouter (the live cap-enforcement leg to build on)
  - crates/providers/src/runtime/escalation_policy.rs::RuntimeRoutingPolicy::evaluate
  - crates/orchestrator/src/ssb.rs::RouteDecisionEvent (the wire-dark SSB — emit it; do NOT re-define, GD-1)
  - crates/orchestrator/src/per_role_runtime.rs::PerRoleRuntime + crates/orchestrator/src/runtime_orchestrator.rs::RuntimeOrchestrator
  - crates/orchestrator/src/cost/cloud_route.rs::HybridEscalationGate (wire or replace the layer_failed:1)
reachable_from:
  - crates/orchestrator/src/task_runner.rs:7496-7535 (the live EscalateToCloud probe) -> calibrated decision -> role dispatch -> RouteDecisionEvent SSB  [PRODUCTION routing]
stub_ban:
  - NO routing on the 7B self-confidence (external calibrated signal only — the documented anti-pattern)
  - NO BSP / parallel code-writers (the second cloud call is a VERIFIER, not a parallel writer — refuted for coding)
  - NO observational-only evaluate (the RoutingDecision MUST change the dispatched provider)
  - NO hardcoded layer_failed:1 / prior_audit_failures:0 / budget_remaining:25.0 (live context)
  - NO re-definition of RouteDecisionEvent (it exists wire-dark — emit it, GD-1 P0)
  - NO default-OFF ELAI_ROUTE_DECISION_SSB_ENABLE as a terminal state (W-2)
  - blake3; no .unwrap()/.expect() in prod
frontend_pair:
  a route-decision drawer rendering RouteDecisionEvent rows (the 3 Kind arms). cross_lang_diff PASS.
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- <orchestrate a real task with a cheaply-checkable edit step AND a
    reasoning/plan step, NO env override>"
  expect_ssb: "a RouteDecisionEvent row (kind=CloudRouteChosen for the architect step, LocalRouteKept/editor for the
    cheap-checkable edit) lands in the live ssb.db, recomputable from the journal under ELAI_SEED (GD-2); the route
    decision reflects the REAL RoutingContext (not the 0/25.0 literals)"
  expect_artifact: "the cheap-checkable edit dispatches to the editor (cheap tier) and is gated by the auditor's
    off-agent check; the reasoning step dispatches to the architect (cloud); the verifier runs in a separate context;
    the over-escalation rate is recorded over >=3 runs"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-7_calibrated_routing_architect_editor_verifier.md --run-trigger
rg "PerRoleRuntime|RuntimeOrchestrator" .elai_cc/crates/orchestrator/src/task_runner.rs   # now referenced for dispatch (was zero)
rg "write_payload.*RouteDecision|RouteDecisionEvent" .elai_cc/crates/orchestrator/src/cloud_mode.rs  # the emitter is wired
rg "layer_failed:\s*1" .elai_cc/crates/orchestrator/src/task_runner.rs                     # the hardcoded literal is GONE
rg "prior_audit_failures:\s*0|budget_remaining_usd:\s*25" .elai_cc/crates/providers/src/runtime/escalation_policy.rs  # MUST be 0 (real context)
python .claude/tools/cross_lang_diff.py
test -f workflow/reasoning/H-7-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: --run-trigger PASS (role dispatch fires; a RouteDecisionEvent row lands; the decision uses real context).
- [ ] W-2: routing active by default; ELAI_ROUTE_DECISION_SSB_ENABLE promoted (kill-switch only).
- [ ] W-3: role dispatch + the RouteDecisionEvent emit fire from the live task_runner EscalateToCloud probe; not test-only.
- [ ] W-4: RouteDecisionEvent drawer; cross_lang_diff PASS.
- [ ] GD-4: the over-escalation rate is MEASURED + bounded on >=3 real runs before default-on.
- [ ] GD-1: RouteDecisionEvent is EMITTED, not re-defined; PerRoleRuntime/RuntimeOrchestrator are wired, not rebuilt.
- [ ] No BSP / parallel code-writers; the second cloud call is a verifier.
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-orchestrator -p elai-providers.
- [ ] No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE 1; the RouteDecisionEvent emit (produced) is reached by the live EscalateToCloud probe (W-3); the role dispatch (produced) consumes PerRoleRuntime (W-8).
- W-2: routing default-on; the SSB-enable gate promoted.
- W-3: task_runner.rs:7496-7535 (live probe); not test.
- W-4: the route-decision drawer; cross_lang_diff PASS.
- W-5: a runnable orchestrate task -> a real RouteDecisionEvent row + real role dispatch; in-process.
- W-6: the drawer via dashboard-smoker.
- W-7: PerRoleRuntime / evaluate / RouteDecisionEvent / layer_failed:1 / the hardcoded context grep-confirmed 2026-06-03 (C8).
- W-8: the role dispatch (produced) consumes PerRoleRuntime/RuntimeOrchestrator; the emitter (produced) consumes RouteDecisionEvent.
- W-9: NO new action_type (RouteDecisionEvent exists — emit it).
- W-10: Tier-1 in-process (the routing decision is deterministic given the context).
- GD-4: the over-escalation gate before default-on; HD-1 cost-AND-correctness, measured.

## Tests to add
- crates/contracts/tests/h7_calibrated_routing_contract.rs (~190 LOC): (a) task_runner dispatches a cheaply-checkable
  edit to the editor (cheap tier) and a reasoning step to the architect (cloud); (b) the editor's output is gated by an
  off-agent check, not its own confidence; (c) RuntimeRoutingPolicy::evaluate's RoutingDecision CHANGES the dispatched
  provider (not observational); (d) emit_route_decision_log writes a RouteDecisionEvent SSB row (not just a log); (e)
  the RoutingContext uses live prior_audit_failures + budget (not 0/25.0); (f) no BSP / parallel writers are spawned.

## Files touched
- MODIFY: crates/orchestrator/src/task_runner.rs (~190 LOC: role dispatch + the calibrated layer_failed replacement)
- MODIFY: crates/orchestrator/src/cloud_mode.rs (~50 LOC: emit_route_decision_log -> write_payload + promote the gate)
- MODIFY: crates/providers/src/runtime/escalation_policy.rs (~60 LOC: real RoutingContext + provider-changing decision)
- MODIFY: crates/orchestrator/src/cost/cloud_route.rs (~40 LOC: wire HybridEscalationGate or the calibrated signal)
- MODIFY: crates/dashboard-v2/src/lib/... (~30 LOC: the RouteDecisionEvent drawer)
- CREATE: crates/contracts/tests/h7_calibrated_routing_contract.rs (~190 LOC)
- CREATE: workflow/reasoning/H-7-decisions.md (~110 LOC)

## Citations / Grounding
- ELAI-EFFICIENCY-TRUST-DIRECTION-2026-06-03.md §10.7 (L7): RouteLLM arXiv:2406.18665 (3.66x/~73%, transfers — the
  ~85% is the LMSYS blog, not the paper body); AutoMix arXiv:2310.12963 (>50%); Architect/Editor +9.6pp (Aider);
  Model-Cascading-for-Code <10% exclusion; SWE-Protege (the gain is the cloud expert's, not a free 7B). DO-NOT:
  7B self-confidence, BSP-for-code (Science of Scaling negative returns above 45%).
- ELAI ground truth (C8): PerRoleRuntime per_role_runtime.rs:182 (unwired); evaluate escalation_policy.rs:261 (observational,
  default-OFF, hardcoded context); RouteDecisionEvent ssb.rs:8153 (no emitter); CloudModeRouter live runtime_setup.rs:533-536; layer_failed:1 task_runner.rs:7606.
- CLAUDE.md HARD RULES + GD-4 + HD-1.

## Open questions / risks
- TOP risk (GD-4): an over-eager editor that fails off-agent checks burns BOTH the cheap call and the cloud retry —
  net negative. MITIGATION: gate on measured per-step-class pass-rate; bound the over-escalation rate on >=3 runs before default-on.
- The calibrated signal needs a calibration target; RouteLLM transfers across pairs without retraining, but ELAI's
  cheap-tier-vs-cloud boundary is its own pairing — measure the cheap-tier pass-rate per step class, do not assume.
- HybridEscalationGate vs a fresh calibrated layer is a design fork; HybridEscalationGate is test-only and may not carry
  the right signal — decide whether to wire it or replace the layer_failed:1 with a new calibrated decision.

## Next
H-9 (cost cockpit) surfaces the route mix ($ saved by editor-routed steps). This is the SMARTER-ORCHESTRATION pillar:
the architect/editor/verifier split, the unused per-role orchestrator finally on the dispatch path, the calibrated
router replacing the hardcoded escalation. It pairs with the G-7 confidence producer (the abstention layer) downstream.

## J-15 AMENDMENT — recorded-escalation-hops idiom (source-cited)
Grounding: determinism triage ledger item 5
(workflow/reasoning/steal-determinism-repos-2026-06-11.md); Balchandar/intentusnet WAL-recorded
fallback HOPS pattern (agent det-repos-S1 verdict).

### The steal: every attempted+failed backend hop lands its own journal row
H-7's fallback chain (cheap-tier -> cloud escalation -> alternate cloud) MUST journal EVERY
attempted+failed backend hop as its own SSB row, not just the final `RouteDecisionEvent`. The
intentusnet WAL pattern: each hop in a fallback chain is a first-class journal entry with
`(from_backend, to_backend, reason, timestamp)`, not a discarded intermediate that only the
final success/failure row captures. Rationale: without per-hop rows, the cost cockpit (H-9)
cannot attribute spend to the hop that actually consumed tokens, the escalation policy cannot
learn which hops fail most often, and the RRA (G-2) cannot replay a multi-hop chain faithfully
(the intermediate hops are invisible).

### Implementation: prefer-reuse existing RouteDecisionEvent kinds
The H-7 executor should prefer-reuse the existing `RouteDecisionEvent` kinds (Pattern-2 with
a `Kind` discriminator) rather than adding a new SSB action_type (GD-1, W-9). The existing
kinds already name routing decisions; a `Kind::EscalationHopFailed` (or similar — decide at
edit after reading the live `RouteDecisionEvent` variant enum) carries the per-hop failure
record. The final successful route still lands the existing `RouteDecisionEvent` success row.
The hop rows are ADDITIVE — they do not replace the final row, they supplement it. If the
live variant enum does not have a suitable hop-failure kind, add ONE new kind to the existing
Pattern-2 variant (not a new action_type — the 9-tuple lockstep applies to the kind addition
within the existing variant, not a full new variant).

### Decision: H-7 over E-31 as the hop-recording home
Both H-7 and E-31 touch routing. H-7 is the calibrated-routing plan with `RouteDecisionEvent`,
`escalation_policy.rs`, and `CloudModeRouter` in its write set — the natural home for
hop-recording. E-31 is the coding-model-leaderboard/tier-matrix plan; its routing surface is
the tier-matrix recommendation, not the live fallback chain. The hop-recording idiom is about
LIVE fallback-chain journaling, not tier-matrix data. H-7's executor lands first (H-7 is
PARTIAL with live code; E-31 is a leaderboard doc). Decision recorded in
workflow/reasoning/J-15-decisions.md.

End of H-7.
