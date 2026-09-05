B-22 Phase 4G — Autonomous Mode decision doc
============================================
session: s-26716
date: 2026-05-09
plan: docs/thefinalglance/B-plan/B-22_phase_4G_autonomous_mode.md
spec: docs/specs/guide/steps/phase-4G.txt

WHAT
----
Phase 4G ships:
1. The /mode slash command (interactive | autonomous) wired through the CLI
   parser to the existing AutonomousGate::set_mode runtime hook + ModeChanged
   SSB row.
2. Real evidence wiring inside task_runner: the six AutonomousEvidence fields
   are derived from live SsbViewer queries instead of the Batch-3B all-true
   placeholder.
3. Real AutoAdvanceStats aggregation inside dashboard-v2's
   get_auto_advance_stats command (driven by GateDecision SSB rows) plus the
   matching panel inside MonitorWindow.svelte.
4. AgentPool::get_role(uuid) for the dashboard's per-agent role inspector and
   the eventual /agents view.
5. Eighteen extended contract tests that pin the per-pipeline-level,
   per-evidence-field, risk-level, and mode-switch behaviours against
   regression. (Existing 14 + 4 contracts in autonomous_gate_contract.rs and
   mode_switch_contract.rs stay GREEN unchanged.)
6. metrics/phase_4G.toml capturing the seven spec-required categories.

KEY DECISIONS
-------------
A. /mode parser lives only in cli/commands.rs.
   The contract for the parser cannot live in crates/contracts because the cli
   crate is binary-only (no [lib]). Adding a [lib] target purely to support
   contract tests would expand the public surface for no operational gain.
   The /mode parser is therefore exercised by unit tests in commands.rs
   (alongside the existing 70+ slash-command parser tests).

B. SlashCommand::Mode { mode: String }.
   String-typed payload (lower-cased downstream by GateMode::from_str) lets the
   parser stay token-validated without taking a dependency on
   elai-orchestrator types from the parser layer. Validation of the actual
   mode value happens at the handler boundary inside dashboard-v2's
   set_gate_mode (already in place).

C. Evidence wiring in task_runner.rs.
   The six fields are computed from the SsbViewer attached to the runtime
   instead of the live FSM/contracts subsystems. Rationale: the runtime
   already holds an SsbViewer and the SSB is the canonical event stream that
   the gate decision is supposed to reflect. Querying live FSM state would
   create circular ownership between the gate and the runner. Evidence
   derivation rules:
   - knowledge_readiness    -> at least one DkgReady row exists
   - zero_ungrounded        -> no Finding rows with severity="warn" and
                               category containing "ungrounded" within the
                               last 1000 rows
   - tests_fail_correctly   -> at least one TestOutcome row with passed=false
                               in the last 1000 rows OR no TestOutcome rows
                               at all (true by vacuous-default in early
                               sessions; mirrors the Batch-3B placeholder
                               semantics for cold sessions)
   - test_flipped           -> at least one TestOutcome row with passed=true
                               in the last 1000 rows OR no TestOutcome rows
                               at all (vacuous-default identical to above)
   - zero_blocks            -> no Finding rows with severity="error" in the
                               last 1000 rows
   - dkg_and_changelog_present
                            -> at least one DkgReady row AND at least one
                               PhaseTransition row in the last 1000 rows
   The 1000-row sliding window matches the SsbViewer's existing per-query
   rate-limited scan ceiling. Earlier sessions with fewer rows fall through
   to the all-true placeholder so the gate keeps Batch-3B behaviour in
   the cold-start path.

D. get_auto_advance_stats aggregates GateDecision rows.
   Algorithm: iterate the last 5000 SSB rows newest-first. For every
   GateDecision row decode the GateDecisionPayload and bucket by
   decision == "auto_advance" vs anything else. last_24h_rate uses
   created_at >= now_ms - 24*60*60*1000. Hard-cap at 5000 rows so the
   query runs in tens of ms even on heavily-active sessions; older history
   is irrelevant to the "did the gate auto-advance recently" signal.

E. AgentPool::get_role default impl.
   Added as a trait method with no default body so all AgentPool
   implementations must provide it. The InProcessAgentPool::get_role looks
   up the AgentHandle by Uuid and converts AgentRole to its canonical
   snake_case via the same mapping as bsp/engine.rs::agent_role_str (kept
   inline rather than re-exported to avoid a cross-module dep on the bsp
   submodule from agent_pool.rs).

F. MonitorWindow panel placement.
   The new "auto-advance" section must show even when fsmSnap == null /
   !isRunning so historical totals stay visible across idle gaps. Section
   sits outside the {#if !isRunning}…{:else}…{/if} switch, immediately
   after the .monitor__inner block, with reused .monitor-section styling
   for visual consistency.

EXPECTED TEST OUTCOME
---------------------
- crates/contracts/tests/phase_4g_extended_contract.rs: 18/18 pass.
- crates/contracts/tests/autonomous_gate_contract.rs: 14/14 pass (unchanged).
- crates/contracts/tests/mode_switch_contract.rs: 4/4 pass (unchanged).
- crates/cli/src/commands.rs unit tests: existing 70+ pass + 3 new /mode
  parser tests.
- crates/orchestrator/src/agent_pool.rs unit tests: existing pass + 2 new
  get_role tests.
- cargo check --workspace: clean.
- cargo clippy: zero new warnings (SCHEMA_VERSION lock unaffected).

UNKNOWNS / DEFERRED
-------------------
- last_24h_rate uses wall-clock filtering through the dashboard-v2 clock
  ClockProvider; does not yet honour a SeededClock under deterministic
  replay. Logged in handoff for a follow-up patch.
- task_runner.rs evidence reads are best-effort: the SsbViewer for the
  runner is constructed lazily from the same SSB the writer feeds, but
  cold-start sessions or cross-session SSB swaps fall back to the
  Batch-3B placeholder semantics (see Decision C above).
- AutoAdvanceStats panel polling is reused from the existing 5s monitor
  poll; per-stat realtime push (e.g. listening to GateDecision SSB events)
  is deferred to a later batch.

RISK
----
Low. Existing 14+4 contracts pin the gate behaviour and stay GREEN; the
18 new contracts only EXTEND the surface without altering signatures.
get_role is additive on the AgentPool trait but has no default body so
every implementor must provide it; only InProcessAgentPool exists in
production code and is updated in this batch.

patch-4G.1 — dashboard wiring fixes
=====================================
session: s-26716
date: 2026-05-09
parent: Phase 4G above

WHAT
----
Follow-up patch (UI-only, 5 Svelte/TS files, no Rust changes) fixing four
dashboard gaps discovered after Phase 4G landed:

1. CommandPalette IPC fix: set_gate_mode call was pointed at a wrong stub
   path. Wired to stub.set_gate_mode(next) with .catch() fire-and-forget;
   local gateMode state updated synchronously before the IPC call so UI
   reactivity is never blocked by backend latency.

2. StatusBar gate badge promotion: gate-indicator element moved out of the
   .verbose conditional block into the always-visible .cluster section so
   the gate-mode badge renders on all screen sizes without toggling verbose
   mode.

3. MonitorWindow gate mini-badge: auto-advance__mode CSS class applied in
   the auto-advance panel; modifier rules (.gate-label--autonomous /
   .gate-label--interactive) added so the panel reflects live gate mode
   colour.

4. session.reconcileGateMode: new exported function in session.svelte.ts
   that calls get_gate_status() on boot, normalises to upper-case, sets
   gateMode, and calls applyGateModeLayout only when the value changed.
   Workspace.svelte calls reconcileGateMode() in onMount before FSM/plan/
   activity bootstrap.

KEY DECISIONS
-------------
A. Cold-start gate-mode default remains INTERACTIVE without localStorage
   persistence. No existing localStorage key for gateMode; adding one is
   out of scope for this patch and would require a migration strategy for
   stale values. The gate toggle is always accessible via CommandPalette so
   the UX cost is one toggle per cold start if the backend is unreachable.
   Documented as intentional in response to auditor WARN-2.

B. IPC ordering: session.setGateMode(next) fires synchronously to update
   local UI state; stub.set_gate_mode(next) fires after as a non-blocking
   promise. IPC failure leaves the local UI updated while backend
   persistence fails — fire-and-forget is the stated intent per the
   existing architecture of all other set_* commands in this dashboard.

AUDITOR VERDICT
---------------
0 BLOCKs. 2 WARNs:
- WARN-1: decision doc gap for patch-4G.1 (resolved by this append).
- WARN-2: no localStorage fallback for gate-mode on cold-start IPC failure
  (documented as intentional — see Decision A above).

cargo check --workspace: PASS (0 errors).
svelte-check: PASS (0 new errors; 9 pre-existing CSS warnings unchanged).
