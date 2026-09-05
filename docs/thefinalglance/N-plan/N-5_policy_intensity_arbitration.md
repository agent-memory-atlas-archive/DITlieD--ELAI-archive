# N-5: policy / intensity arbitration plane

- Track: TIER 2 (HIGH) | lever: explainable-non-emergent-routing | nature: BUILD | WAVE 4
- Status: AMEND REQUIRED; N-5 remains dark on current main.
- Dependency: N-1 is a partial prerequisite, not complete. Its measurement spine, two bench families, `spine_entries`, `dark_families`, and `render_spine_table` are landed; bootstrap, outcome labels, and the broader bench coverage remain prerequisite work. Do not claim the stale ten-bench N-1 card is complete.

## Objective and bounded scope

Implement only the Tier-1 backend residual: one deterministic arbitration decision at the live `task_runner` route seam, plus one deterministic five-axis intensity resolver. The slice must explain the selected route and provide an auditable intensity measurement. It must not add a router, rewrite router logic, add the deferred J-66 monitor, or implement a frontend consumer. N-7 canvas/RRA acceptance is deferred until its consumer exists.

The precedence is frozen data, not model-settable policy:
`safety-gates > cost-ceiling > abstention > escalation > modality > serving-tier`.
The implementation orders outputs from the existing E-39, C-46, K-5, J-48, and J-48-adjacent task-runner seams (and the authoring-time inventory must re-grep the named J-48/task-runner source before coding). The full historical router list is an inventory target, not permission to invent absent seams: D-10, J-84, J-55, H-7, E-20, J-48, J-54, L-6, J-32, and J-73 are consulted only where current source exposes them. Reuse their outputs; do not reimplement them.

## Authoring-time reconnaissance and reuse inventory

Before implementation, record source evidence for each claimed producer and its caller:

- E-39: `crates/orchestrator/src/cost/cost_router.rs` and its live `task_runner.rs` call. Preserve its difficulty/budget/criticality semantics; do not assume it supplies all N-5 axes.
- C-46: `crates/orchestrator/src/routing/mod.rs`; reuse its difficulty/delegation output rather than making a second delegation policy.
- K-5: `crates/orchestrator/src/gate_intensity.rs` and `deterministic_audit_gate.rs`; reuse the table-backed risk signal where its contract matches, and explicitly handle its current parse fallback rather than introducing a production `unwrap`/`expect`.
- J-48: locate and cite the actual serving-tier producer and its task-runner call; if the claimed seam is absent, mark that input unavailable and apply the contract's deterministic missing-input rule.
- task runner: the production route seam is `crates/orchestrator/src/task_runner.rs:9903-9954` (line numbers are reconnaissance anchors, not API contracts). The smoke must drive this route, not a tracing-only cloud-mode helper.
- N-1: consume only the landed measurement-spine interface verified above; do not broaden this slice into the unlanded ten-bench spine.

## Five-axis input contract

The resolver accepts exactly these typed axes, with one canonical value per axis:

1. `risk`: K-5 gate-risk / `RiskSignalVector`-derived frozen risk tier;
2. `model_tier`: the selected serving/model tier (J-48 or the actual current serving-tier producer);
3. `level`: the pipeline/task level from the existing level producer;
4. `difficulty`: E-39/C-46 difficulty output;
5. `delegation`: C-46 delegation output.

The implementation must document the concrete enum/value mapping and provenance for every field in its input struct. No current E-39 call may be treated as supplying `model_tier`, `level`, or `delegation` merely because it supplies difficulty, budget, and hardcoded `Criticality::Medium`.

Missing input behavior is deterministic and non-configurable: a required axis is not silently fabricated, inferred from unrelated fields, or read from wall-clock state. The resolver returns a typed `MissingInput(axis)` result (or the repository-equivalent explicit error) and emits no load-bearing intensity selection. The caller records a deterministic rejected/insufficient-input outcome and does not route on a partial five-axis vector. If the existing production contract makes one axis genuinely optional, the plan executor must document that fact and use one named frozen sentinel value in the schema and tests; absent that evidence, treat all five axes as required. No production `unwrap` or `expect` is allowed.

K-5 remains observe-only: its existing selector and the new resolver are telemetry/measurement unless an explicitly approved follow-on changes this contract. They must not alter route selection, budget enforcement, or gate behavior. Bind K-4 gate-tax measurement to this observe-only choice and record the resolver result as the single policy measurement.

## Closed arbitration wire contract

Extend the existing `RouteDecisionEventPayload` envelope only if the action-type registry and golden/schema review confirm it is the compatible contract; otherwise add a separately registered action type. In either case the wire kind is closed and exact: `route_arbitrated` is the sole new arbitration kind, and unknown kinds are rejected by schema validation. The payload contains the existing envelope identity/timestamp fields plus:

- `routers_consulted`: non-empty, deterministic, sorted/stable list of router IDs;
- `winner`: exactly one selected route/router outcome;
- `reason`: stable machine-readable reason code, not free-form model text;
- `precedence_rank`: frozen numeric rank corresponding to the precedence table;
- `intensity`: the observe-only five-axis result or the explicit `missing_input` outcome;
- `schema_version`: the locked version.

The row is emitted exactly once when at least two router outputs conflict, names every consulted router, names the winner, and records the precedence rank. A single-router/no-conflict path does not pretend arbitration occurred. Update the action-type lock, payload golden, and schema validation in the implementation slice; do not weaken validation or permit model-supplied precedence. The selected winner must be the route actually taken.

## PRODUCTION_HOOK / WIRING_CONTRACT

PRODUCTION_HOOK: `crates/orchestrator/src/task_runner.rs`, the live route producer at approximately lines 9903-9954; the implementation must call arbitration from this real task-runner path and emit `route_arbitrated` there. The tracing-only cloud-mode helper is not a production caller. The smoke must prove the call and route effect through this hook.

WIRING_CONTRACT:
- produces: frozen precedence table, closed `route_arbitrated` row emitter, five-axis intensity resolver;
- consumes: existing E-39, C-46, K-5, J-48/task-runner outputs and only source-verified router outputs; J-92 is a structural pattern, not a new runtime dependency;
- reachable_from: live task-runner route with two conflicting router outputs; a test-only caller never satisfies wiring;
- forbidden: reimplemented routers, model-settable precedence, partial-vector routing, production `unwrap`/`expect`, dark public symbols;
- N-7 frontend consumer and J-66 monitor are deferred, not silently stubbed.

## Acceptance and observable behavior

EXPECTED_OBSERVATION: On a hard-input live task-runner smoke with two conflicting, source-backed router outputs, exactly one `route_arbitrated` row is emitted; it names both consulted routers, the frozen winning precedence/rank, and the five-axis intensity result, and the route actually selected equals the recorded winner. A missing required axis yields the deterministic `MissingInput(axis)` outcome and does not select a route from a partial vector. K-5 remains observe-only. No N-7 frontend assertion is made.

ON_FAIL: If any expected observation is absent, duplicated, schema-invalid, mismatched with the selected route, dependent on a fabricated/missing axis, or changes K-5 behavior, FAIL the slice and write the failing command/output to `/home/user/teikoku/conductor/opus_walk/evidence/N-5/verify-failure.txt`; do not claim completion. If the real task-runner seam or a named source producer cannot be verified, stop as BLOCKED and write the evidence and the exact missing seam to `/home/user/teikoku/conductor/opus_walk/evidence/N-5/blocker.txt`.

RECON_NEEDED: `/home/user/teikoku/conductor/opus_walk/evidence/N-5/recon.md` (typed verdict AMEND). Re-run the authoring inventory against current source and preserve its cited E-39, C-46, K-5, J-48, task-runner, N-1, and N-7 deltas. A stale assertion that all N units are dark or that all five axes already arrive together is invalid.

ABORT CONDITIONS:
- the task requires edits outside the Tier-1 backend residual, the allowed route seam, or its directly required schema/golden registration;
- the real task-runner production hook cannot be identified or wired;
- any required axis has no source-backed producer and no explicit deterministic missing-input result can be represented;
- arbitration would require changing a router's internal logic, making K-5 load-bearing, or inventing a public symbol with no production caller;
- schema/golden compatibility cannot be established without weakening the closed wire contract;
- N-7 frontend work or J-66 monitoring becomes a prerequisite for this backend slice.

## Off-agent verification

VERIFY-1 (recon and seam):
`cd /home/user/teikoku/elai && python .claude/tools/production_caller_check.py --files .elai_cc/crates/orchestrator/src/task_runner.rs > /home/user/teikoku/conductor/opus_walk/evidence/N-5/production-caller.txt`
PASS artifact: `/home/user/teikoku/conductor/opus_walk/evidence/N-5/production-caller.txt`; it must show the new producer is reached from the live task-runner path and no DARK symbol is present. On failure: `ON_FAIL` above.

VERIFY-2 (wire/schema):
`cd /home/user/teikoku/elai && grep -R -n 'route_arbitrated\|precedence_rank\|routers_consulted\|MissingInput' .elai_cc/crates/orchestrator/src > /home/user/teikoku/conductor/opus_walk/evidence/N-5/wire-contract.txt`
PASS artifact: `/home/user/teikoku/conductor/opus_walk/evidence/N-5/wire-contract.txt`; it must identify the closed action registration, payload fields, and missing-input path. On failure: `ON_FAIL` above.

VERIFY-3 (negative smoke):
`cd /home/user/teikoku/elai && python .claude/tools/plan_gate_check.py docs/thefinalglance/N-plan/N-5_policy_intensity_arbitration.md > /home/user/teikoku/conductor/opus_walk/evidence/N-5/plan-gate.txt`
PASS artifact: `/home/user/teikoku/conductor/opus_walk/evidence/N-5/plan-gate.txt`; it must pass the frozen plan gate, including matched EXPECTED_OBSERVATION/ON_FAIL blocks, evidence, abort, wiring, red-team, and smoke sections. On failure: `ON_FAIL` above.

INTEGRATION_SMOKE: The implementation verifier, off-agent, drives the actual task-runner route with two conflicting router outputs and a hard input, then checks the emitted row and selected route. It also runs the missing-axis case and asserts no partial-vector route. Save output at `/home/user/teikoku/conductor/opus_walk/evidence/N-5/integration-smoke.txt`. This is not discharged by unit tests or by the tracing-only helper.

## REDTEAM

Attack the seam with (a) two conflicting outputs whose call order differs, (b) a model-supplied precedence/rank, (c) a missing model tier or delegation field, (d) duplicate router IDs, (e) an unknown wire kind/schema version, and (f) K-5 configured to influence routing. Expected defenses are frozen precedence independent of call order, ignored/rejected model precedence, deterministic `MissingInput`, stable deduplication/rejection, closed-schema rejection, and unchanged observe-only K-5 behavior. Record results at `/home/user/teikoku/conductor/opus_walk/evidence/N-5/redteam.txt`; any failed defense follows `ON_FAIL` and blocks certification.

## Deferred / non-goals

N-7 canvas/RRA explain-row acceptance is explicitly deferred until the N-7 consumer exists. J-66 routing-order monitoring is Tier-2. N-1's remaining bootstrap/outcome-label/bench expansion is a prerequisite dependency, not work to smuggle into N-5. No router or decider is reimplemented.

FORK: If the authoring inventory finds a required source producer or the real task_runner hook absent, do not invent a seam; set the unit aside for a seam-design decision or amend the plan to the verified residual.
