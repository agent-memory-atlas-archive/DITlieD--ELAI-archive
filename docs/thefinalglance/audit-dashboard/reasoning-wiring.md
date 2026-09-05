# Reasoning Engine Dashboard Wiring Audit
Date: 2026-04-25
Auditor stance: HOSTILE
Scope: Phase 2A (GRC state machine), 2B (grounding verification),
       2C (per-agent GRC config), 2D (CSQ + inline feasibility)
       — surfaced through Tauri dashboard backend (commands.rs / lib.rs)
       and Svelte frontend (src-frontend/src/**).

================================================================
OVERALL VERDICT: MOSTLY INVISIBLE
================================================================

Of the four phases, exactly ONE end-to-end signal reaches the dashboard:
GRC budgets (2A) — and even that is pseudo-data manufactured by
`ssb_viewer.grc_budgets` from `reasoning_trace` SSB rows that nothing
in production ever fills with real GRC engine output. The reasoning
crate IS production-instantiated, but its `think()` / `verify_claim()` /
`ground_plan()` / `generate_csq()` / `check_inline_feasibility()` methods
have ZERO production callers. Every "reasoning trace" in SSB today is
the task_runner's `emit_reasoning` helper hard-coding
`agent_id="task_runner"`, `topic="task_runner"`, `budget_remaining=0`
(task_runner.rs:3985-3997). The ReasoningDrawer renders that
hard-coded synthetic data and labels it "GRC budgets". It is not.

The user's "doesn't feel or look like I wanted" instinct lines up:
the reasoning engine is a parallel-running but disconnected subsystem,
and the dashboard surface for it is a 121-line drawer that hides behind
a click on the ACTIVITY window's drawer rail (ActivityWindow.svelte:46).

================================================================
PHASE 2A — GRC state machine
================================================================

Backend signals available in principle:
  - GRC ThinkStep traces (per-agent, per-topic, per-step)
  - Per-topic budgets (max_calls, max_chars, calls_used, chars_used,
    remaining_calls, must_converge)
  - GroundingLevel (Verified/High/Uncertain) per step
  - State machine transitions (Observe→Hypothesize→…→Converge)
  - Warnings on uncertain grounding

Backend signal production reality:
  - `emit_reasoning` (task_runner.rs:3985-4004) writes the ONE
    production `ReasoningTracePayload`. It hard-codes:
      agent_id        = RUNNER_AGENT_ID ("task_runner")
      topic           = "task_runner"
      grounding_source= "task_runner"
      grounding_level = "medium" (string, not GroundingLevel enum)
      confidence      = 0.9 literal
      elapsed_ms      = 0 literal
      budget_remaining= 0 literal
    The fields the dashboard needs to surface real GRC state
    (per-agent topic, real budget_remaining, real step_index from
    the engine's state) are stubbed to zeros and runner-scoped.
  - `BasicGrcEngine::think` is invoked only inside the reasoning
    crate's tests and `orchestrator/src/grc.rs:81` (which itself
    is only exercised by tests at grc.rs:217). No FSM transition,
    auditor agent, executor agent, or pipeline call site invokes
    `engine.think()` and pushes its `ThinkResult` into SSB.

Dashboard wiring:
  - Tauri command `get_grc_budgets` registered at lib.rs:1474,
    implemented at commands.rs:3164-3172. Calls
    `SsbViewer::grc_budgets` (ssb_viewer.rs:1614-1671).
  - The viewer GROUPS reasoning_trace rows by (agent_id, topic),
    counting rows for `calls_used` and pulling the latest payload
    for `budget_remaining`. Because all production rows are
    `("task_runner", "task_runner", budget_remaining=0)`, the
    drawer always shows ONE row reading "task_runner/task_runner —
    N calls · 0 left" — a synthetic count, not a real GRC budget.
  - Frontend: `ReasoningDrawer.svelte:46` invokes
    `get_grc_budgets({limit: 10})` only on first expand
    (lazy-load, no polling, no live update). Renders into a
    collapsed drawer titled "reasoning · N topics" inside
    ActivityWindow (ActivityWindow.svelte:46). No state-transition
    visualisation; no per-topic budget gauge; no Observe→…→Converge
    flow indicator.

Verdict: PARTIAL → effectively INVISIBLE.
Type-level wiring exists (command, viewer, drawer); semantic content
is fake. The user clicks the drawer, sees "task_runner/task_runner ·
N calls · 0 left", and learns nothing about GRC.

================================================================
PHASE 2B — Grounding verification
================================================================

Backend signals available in principle:
  - VerificationResult per claim (passes, evidence, method_used)
  - PlanGroundingResult (claims, all_grounded, ungrounded,
    verify_needed)
  - VerificationMethod enum (FileCheck, SourceCheck, TestCheck,
    WebCheck, ApiCheck, ApiRegistryCheck) with
    requires_network/requires_tools predicates
  - GroundingSweepGate Pass/Block outcome

Backend signal production reality (from phase-2B audit P-2):
  - `verify_claim` and `ground_plan` exist on `PipelineGrc`
    (orchestrator/src/grc.rs:92-110) but have ZERO production
    callers. Only `crates/reasoning/benches/verification_bench.rs`,
    `crates/reasoning/tests/verification_tests.rs`, and
    `crates/orchestrator/src/grc.rs:264-276` (test mod) reach them.
  - GroundingSweepGate (evidence.rs:191-218) consumes
    `TransitionContext.ungrounded_claims` (Option<Vec<String>>).
    Nothing populates that field — the gate vacuously passes.
  - There is NO SsbActionType::GroundingSweep, no
    GroundingSweepPayload, no `emit_grounding_sweep` helper.
    The signal cannot reach SSB at all.

Dashboard wiring:
  - ReasoningDrawer.svelte comment at line 3 explicitly says:
    "Ghost command get_grounding_sweep is dropped." That is a
    silent admission that the surface for 2B was abandoned.
  - Grep `dashboard/src/**` for `verify_claim|ground_plan|grounding|
    GroundingResult|PlanGrounding` → ZERO matches. There is no
    Tauri command, no Rust struct, no viewer method exposing 2B
    results to the frontend.
  - Grep `dashboard/src-frontend/**` for the same → ZERO matches.
  - The closest related item is `feasibility_report` rendering in
    ActivityFeed.svelte (lines 88-97, 216-261) — that's Phase 4F
    feasibility, NOT Phase 2B grounding.

Verdict: INVISIBLE.
2B has no SSB action_type, no Tauri command, no Svelte component,
and no UI affordance whatsoever. The user has zero way to see "this
claim was verified against tests/source/file" or "the plan was swept
and 3 claims are ungrounded."

================================================================
PHASE 2C — Per-agent GRC config
================================================================

Backend signals available in principle:
  - AgentGrcConfig per agent role (8 roles; budget, required_steps,
    grounding_level, csq_required, allowed_methods,
    transition_overrides, require_scope_enumeration)
  - Anti-rubber-stamp enforcement outcome for auditor (FINDING:
    prefix counts; engine.rs:297-315)
  - Per-role config validation result
  - get_agent_config() on the engine

Backend signal production reality:
  - AgentGrcConfig IS attached to every agent handle at spawn
    (agent_pool.rs:365-371; default_grc_config at lines 467-592).
  - Per phase-2C audit G2, two sources of truth disagree
    (orchestrator/src/agent_pool.rs:467 vs
    reasoning/src/config.rs::TomlConfigLoader). ContextCompiler
    (line 563-566) and MediaValidator (line 587-590) inherit
    `AgentGrcConfig::default()` (5-step chain, Uncertain
    grounding, all methods) — losing the spec-required
    observe-only / restricted-methods enforcement.
  - The TOML loader at reasoning/src/config.rs is a stub: it does
    NOT read elai_cc.toml. There IS no TOML→config flow.
  - Anti-rubber-stamp enforcement: lives in `engine.think()`. Since
    no production code calls `engine.think()` (see 2A above), the
    enforcement never fires in production despite being correctly
    implemented.

Dashboard wiring:
  - Grep `dashboard/src/**` for `AgentGrcConfig|grc_config|
    get_agent_config|required_steps|csq_required|transition_overrides`
    → ZERO matches.
  - Grep `dashboard/src-frontend/**` for the same → ZERO matches.
    (`commands.rs` `role:` matches at lines 1566/1592/1636 are
    *steering vector* role / model role, unrelated to GRC.)
  - There is no per-agent panel. The closest UI is
    OrchestrationDrawer.svelte (which lists agent checkpoints, not
    config) and PlanTree.svelte:114 (which shows agent_id label
    only). Nowhere can the user see "auditor budget 10/1200,
    grounding=Verified, csq_required=true" or "context_compiler
    rejected hypothesize step".
  - The two-way discrepancy between orchestrator and reasoning
    config tables is therefore also invisible — the user has no
    way to detect that ContextCompiler is shipping with the wrong
    config.

Verdict: INVISIBLE.
2C is one of the most user-meaningful phases (it defines per-agent
behavior the user is supposed to trust) and the dashboard exposes
NONE of it. No command, no view, no per-agent reasoning panel.

================================================================
PHASE 2D — CSQ + inline feasibility
================================================================

Backend signals available in principle:
  - CsqResult per sensitive function (checks: Vec<CsqCheck> with
    severity Critical/Warning, guard_added bool, passed())
  - InlineFeasibilityResult per claim (Pass/Fail/Blocked) with
    method=SourceMatch
  - InlineFeasibilityState (checks_used / max_checks=2)
  - BudgetExhausted on third feasibility check

Backend signal production reality (from phase-2D audit Wiring):
  - `PipelineGrc::generate_csq` and `check_inline_feasibility`
    exist (grc.rs:122-149) and are tested (grc.rs:222-256).
  - ZERO production callers. file_ops.rs, dispatcher.rs, fsm.rs,
    mcp_tool_bridge.rs, permissions.rs, permission_enforcer.rs,
    gate.rs, autonomous_gate.rs, deterministic_audit_gate.rs —
    none invoke either method.
  - No SsbActionType for CSQ or inline feasibility outcomes. No
    payload struct. No emit helper.
  - InlineFeasibilityState.reset() between phases is also never
    called in production.

Dashboard wiring:
  - Grep `dashboard/src/**` for `csq|CsqResult|CsqCheck|
    inline_feasibility|FeasibilityOutcome` → ZERO matches.
  - Grep `dashboard/src-frontend/**` for the same → ZERO matches.
    (`feasibility` matches in ActivityFeed/ActivityEntry are
    Phase 4F task feasibility, not 2D inline feasibility.)
  - There is NO UI affordance to show "CSQ blocked this Write
    because guard missing on `auth` op" or "inline feasibility
    Blocked: replan recommended". Critical CSQ failures and
    Blocked feasibility outcomes — both designed as user-facing
    safety signals — are completely silent.

Verdict: INVISIBLE.
2D is the safety layer the user implicitly trusts to "stop the agent
before it does the wrong destructive thing." The dashboard cannot
show the user when (or whether) that protection fired. As a result,
silence cannot be distinguished from "no enforcement happened" — a
classic safety-UX failure.

================================================================
CROSS-CUTTING WIRING GAPS (REASONING)
================================================================

CG-1. ZERO production caller for the GRC engine. PipelineGrc exists,
       BasicGrcEngine is instantiated per agent_pool handle
       (agent_pool.rs:60-87), but NO orchestrator code path calls
       `engine.think()`, `verify_claim()`, `ground_plan()`,
       `generate_csq()`, or `check_inline_feasibility()` against
       real plans / real tool calls. Every wired-but-unused method
       is dead in production. Implementation audits 2A through 2D
       all flag this; the dashboard audit confirms nothing
       downstream consumes it either.

CG-2. ReasoningTrace SSB payload is hard-coded by `emit_reasoning`
       (task_runner.rs:3985-3997). topic/budget_remaining/
       agent_id/grounding_level/confidence/elapsed_ms are literal
       constants. The dashboard's ONLY GRC surface
       (ReasoningDrawer / get_grc_budgets) renders that synthetic
       data and labels it "GRC budgets — N topics". This is
       arguably worse than having no surface: it gives the user
       false confidence that GRC is running and reporting state.

CG-3. No SSB action types for the 2B/2D-specific outcomes.
       Required-but-absent action_types (per phase-2 spec):
         - grounding_sweep (PlanGroundingResult per phase entry)
         - claim_verified (VerificationResult per claim)
         - csq_check (CsqResult per sensitive op)
         - inline_feasibility (Pass/Fail/Blocked outcome)
       Without these, the viewer can't query, the command can't
       wrap, and the drawer can't render — the dashboard gap is
       structural, not cosmetic.

CG-4. AgentGrcConfig has no Tauri command or frontend surface.
       The dashboard cannot show the user which agent runs with
       which budget / grounding floor / csq policy / observe-only
       restriction. The 2C divergence (ContextCompiler shipped
       with default 5-step chain instead of observe-only) is
       therefore undetectable from the UI.

CG-5. ReasoningDrawer is lazy-load only (line 41-58). Once opened,
       it does not poll or refresh. New reasoning_trace rows
       arriving while the drawer is open are invisible. The
       activity feed has 500ms polling but the reasoning drawer
       has zero. This breaks the "live engine" feel.

CG-6. ReasoningDrawer comment at line 5 admits that "mistake book,
       GG signals, DAIRA trace are fetched lazily on their sub-tab
       click" — but the rendered drawer (lines 70-104) has NO
       sub-tabs. Only GRC budgets and the latest failure taxonomy
       row are shown. Mistake book, GG signals, and DAIRA trace
       are dead code from the user's perspective: backend commands
       exist (commands.rs:3174-3212) but no UI path reaches them.

CG-7. Grounding sweep is explicitly DROPPED. Drawer comment at
       line 3: "Ghost command get_grounding_sweep is dropped." So
       the dashboard team consciously gave up on Phase 2B
       surfacing. This decision is undocumented anywhere outside
       the drawer comment.

CG-8. ActivityEntry rendering for `reasoning_trace` rows
       (ActivityEntry.svelte:240-246) only assigns a colour
       (purple). It does NOT render topic, budget_remaining,
       step_index, grounding_level, or confidence in any
       feed-row decoration. Even when reasoning_trace eventually
       carries real data, the existing entry renderer drops it
       on the floor.

CG-9. The 2A grounding-validation warning surface is invisible.
       `ThinkResult.warnings` is a real field; nothing in the
       SSB payload carries warnings; the drawer cannot show them.

================================================================
RECOMMENDED DASHBOARD SURFACES
================================================================

R-1. PER-AGENT GRC PANEL (addresses 2C invisibility).
     New Tauri command `get_agent_grc_config(agent_id)` returning
     AgentGrcConfig serialized. New panel inside OrchestrationDrawer
     or a new ReasoningWindow tab listing each agent_id with:
       role · budget(max/used) · grounding_level · csq_required ·
       required_steps chain · transition_overrides
     Highlight rows where orchestrator config diverges from
     reasoning::TomlConfigLoader (the 2C audit G2 divergence).

R-2. GROUNDING SWEEP STRIP (addresses 2B invisibility).
     Add SsbActionType::GroundingSweep with
     GroundingSweepPayload { phase_id, all_grounded, ungrounded:
     Vec<String>, verify_needed: Vec<String>, source: String }.
     Wire `engine.ground_plan()` into the
     PLANNING→TESTING_FIRST transition handler so the gate
     populates real data. New Tauri command
     `get_latest_grounding_sweep(session_id)`. New strip on
     ActivityFeed (analogous to feasibility-strip at
     ActivityFeed.svelte:216-261) that renders red when
     `all_grounded=false` with a click-to-expand list of
     ungrounded claims.

R-3. CSQ BLOCK ALERT (addresses 2D invisibility, half).
     Add SsbActionType::CsqCheck with CsqCheckPayload
     { function_id, severity, assumption, consequence,
     guard_added, blocked: bool }. Wire `generate_csq()` into a
     pre-Write/Edit/Bash hook. ActivityEntry adds a "csq" case
     rendering Critical-no-guard rows in red with the
     consequence text inline.

R-4. INLINE FEASIBILITY OUTCOME (addresses 2D invisibility, half).
     Add SsbActionType::InlineFeasibility with
     InlineFeasibilityPayload { claim, outcome (Pass/Fail/Blocked),
     method, sources_checked }. Wire `check_inline_feasibility()`
     into the EXECUTING handler. Render Blocked outcomes with a
     replan suggestion link.

R-5. REAL ReasoningTrace EMISSION (addresses CG-2).
     Replace the hard-coded `emit_reasoning` constants in
     task_runner.rs:3985 with actual call:
       let result = handle.grc_engine.think(step)?;
       let trace = ReasoningTracePayload {
         agent_id: handle.agent_id.clone(),
         step_index: result.step.index,
         step_type: format!("{:?}", result.step.state),
         content_summary: result.step.content.clone(),
         grounding_source: result.step.grounding.unwrap_or_default(),
         grounding_level: format!("{:?}", result.grounding_level),
         confidence: result.confidence,
         elapsed_ms: t0.elapsed().as_millis() as u64,
         topic: result.step.topic.clone(),
         budget_remaining: result.budget_remaining.calls,
       };
     Then `get_grc_budgets` and the drawer become meaningful.

R-6. STATE-MACHINE FLOW VISUALISATION.
     ReasoningDrawer should render the GRC chain
     Observe→Hypothesize→PlanAction→Act→Verify→Converge per topic
     as a 6-dot row with the active dot highlighted, derived from
     the latest `ReasoningTracePayload.step_type` for that
     (agent_id, topic). Cheap; high feel-impact.

R-7. POLLING + LIVE PULSE (addresses CG-5).
     ReasoningDrawer should poll every 1s (or subscribe via
     Tauri event) like ActivityFeed does (POLL_MS=500). Pulse
     animation when a new `reasoning_trace` row lands, mirroring
     the ActivityWindow `is-executing` pulse
     (ActivityWindow.svelte:71-74).

R-8. ANTI-RUBBER-STAMP COUNTER FOR AUDITOR.
     Tauri command exposing
     `auditor_findings_count_in_last_converge`. Drawer line:
     "auditor: 3 findings since last Converge — pass".
     Critical user-facing safety signal that 2C wired but
     silently buried.

R-9. RETIRE MISLEADING DRAWER LABELS.
     Until R-5 lands, the drawer's "reasoning · N topics" label
     is a lie (it's task_runner emission counts). Either rewire
     to real GRC engine output (R-5) or rename the drawer to
     "task_runner traces" so the user knows what they are
     looking at.

================================================================
SUMMARY
================================================================

Phase 2A: PARTIAL surface (drawer + command exist), but renders
          synthetic task_runner data. Effectively INVISIBLE to the
          user as actual GRC state. The "reasoning" drawer label
          is misleading.
Phase 2B: INVISIBLE. No action_type, no command, no UI. Drawer
          comment explicitly says "ghost command dropped".
          GroundingSweepGate vacuously passes because no producer
          populates ungrounded_claims.
Phase 2C: INVISIBLE. AgentGrcConfig is attached at spawn but the
          dashboard exposes ZERO commands or views. The 2C
          ContextCompiler/MediaValidator divergence is
          undetectable from the UI.
Phase 2D: INVISIBLE. CSQ and inline feasibility have ZERO Tauri
          commands, ZERO frontend code, and ZERO SSB action_types.
          Critical safety signals are silent.

Cross-cutting: The reasoning engine subsystem is the largest
internal investment in the codebase that does not surface to
the user at all. Of ~2200 LOC of reasoning crate + 590 LOC of
PipelineGrc + agent_pool wiring, the dashboard exposes 1 drawer
(121 LOC) backed by 5 Tauri commands (commands.rs:3163-3212),
4 of which (mistake_book, gg_signals, failure_taxonomy_last,
dair_trace_last) are not connected to any rendered sub-tab in
the drawer.

Does ReasoningDrawer render REAL data? NO — it renders
hard-coded task_runner emission counts dressed up as "GRC
budgets". The failure_taxonomy_last row IS real data (when
populated by `emit_failure_classified`, task_runner.rs:4380),
but the budget row is not. From the user's perspective, the
entire reasoning engine is a phantom.

The user's "doesn't feel or look like I wanted" is correct
about the reasoning surface specifically: there is essentially
no reasoning surface. To reach the spec's vision, R-2 / R-3 /
R-4 / R-5 are mandatory; R-1 / R-6 / R-7 / R-8 / R-9 are the
fixes that make the existing drawer worth opening.
