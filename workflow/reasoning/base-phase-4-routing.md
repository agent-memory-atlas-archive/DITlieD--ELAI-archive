BASE PHASE 4 ROUTING + DECISIONS
date: 2026-06-05
session: base-phase-4 executor (s-plan-audit-2026-06-04 follow-on)
source punch list: workflow/sessions/s-plan-audit-2026-06-04/FINISH/base-phase-4-FINISH.md

This doc records (1) the two base-4 DECISIONS that are documentation-only, and
(2) the full ROUTE list: items NOT implemented in base-4 because an upcoming
B/C/E letter plan owns them. Items in the DO set were implemented in code and
are reported separately (not here).


DECISIONS (base-4-owned)

4D DESIGNER-DISPATCH (decided: decompose-output equivalence suffices)
  Question: does AgentRole::Designer require agent-pool dispatch, or does
  decompose-output equivalence suffice?
  Grounding (code read this session):
    - AgentRole::Designer has a real role entry + GRC config in the pool role
      table (agent_pool.rs:761 label, :849 grc config, :3920-3947 canonical
      Designer chain ending in Converge, no Act; "Designer emits a PipelineSpec,
      it does not Act").
    - There is NO production call site that does pool.spawn(AgentRole::Designer).
      The production decompose/pipeline path constructs a PipelineSpec inline
      (research_tree::PipelineSpec) and feeds it straight to PipelineBuilder
      (pipeline_builder.rs:425-429 PipelineSpec -> DependencyGraph -> validate ->
      repair -> ExecutablePipeline). The Designer role config is reserved for the
      agent-FSM path, not the inline decompose path.
  Decision: decompose-output equivalence SUFFICES for base-4. The Designer role's
    output contract (a PipelineSpec) is already what the production path produces
    and consumes; spawning a separate Designer agent would add a round-trip with
    no behavioural delta on the current inline path. No code change in base-4.
    If/when the agent-FSM path needs a spawned Designer agent (distinct from the
    inline PipelineSpec producer), that is a NEW capability and belongs to the
    plan that introduces agent-FSM-driven design (not a base-4 reconciliation
    item). Lower priority per the punch list — recorded, no action.

4D REGISTRYCHECK (BLOCKED_ARCH -> route to D-11; NOT faked)
  Intended: wire research_tree::verify_packages_with_emitter (research_tree.rs
    :2576) to call CheckerKind::RegistryCheck via the checker_registry runner
    instead of the shape-only is_plausible_package_name (:2631).
  Confirmed exist (grep): CheckerKind::RegistryCheck (checker_registry.rs:94) and
    the CheckerRegistry runner (CheckerRegistry::run_all_applicable, :319).
  Why it cannot be wired in base-4 without faking (all three DO-named blockers
    fire at once):
    1. run_all_applicable is `async` and the per-checker Checker::check is `async`
       (checker_registry.rs:319, :333). verify_packages_with_emitter is a SYNC
       free function (`pub fn ... -> Result<...>`) with no tokio runtime/handle in
       scope and sync-only callers (verify_packages wrapper + sync test callers;
       grep found no async production caller).
    2. The runner needs a &CheckerRegistry instance (constructed once at boot via
       CheckerRegistry::load_from_toml(workspace_root), :979). That handle is
       None / absent at this sync call site — no registry is threaded in.
    3. RegistryCheck is a ScriptChecker that shells out to
       .elai_cc/checkers/registry/registry_check.py doing a LIVE PyPI/npm network
       call. The DO item explicitly forbids running live network checks here.
    Blocking on a network subprocess inside this sync, hot decompose path would
    also violate the "no block_on in sync hot path" posture.
  Action: ROUTE to D-11 (owner of RegistryCheck + checker_registry, per
    D-11-decisions.md DECISION-5). The real wiring belongs in the async
    verification path where a &CheckerRegistry handle and a tokio runtime exist
    (the producer that constructs the registry and calls run_all_applicable on a
    Claim per package). is_plausible_package_name stays as the sync shape-only
    pre-filter; it is NOT a substitute for the network check and is left
    unchanged (no fake-success swap). The 4 B-7 emitter plug-ins (which would
    supply the production SsbPackageRefVerifiedEmitter) remain routed to B-7.


ROUTE LIST (NOT implemented in base-4 — owned by the named plan)

phase-4A
  - allowed_paths -> B-18 : wire AgentHandle.allowed_paths into tools/src/lib.rs
    tool-dispatch gate (replace static is_hidden_path with per-agent path check).
  - InProcessToolDispatcher deletion -> B-20 (unblocked by C-11+C-13).
  - expect_ssb row on phase-4A WIRING_CONTRACT -> B-18 (punch item 4).

phase-4B
  - edit_match_failed SSB row (SsbPayload::EditMatchFailed on NotMatched in
    dispatcher.rs) -> B-20 (B-18B-gated).
  - integration_checkpoint_success SSB rows with timing -> B-20 (task 8).
  - WIP commit squash on phase boundary -> B-20 (task 9).
  - InProcessToolDispatcher deletion (after B-18B IPC accept loop ships) -> B-20
    (DEFERRED-TO-B-18B).

phase-4C
  - build_production_ipc_hook_dispatcher wiring into run_subcommand.rs +
    task_cmd.rs + dashboard lib.rs (B-19 DECISION-A) -> B-19.
  - ELAI_HOOKS_DISABLED allowlist row -> B-19.
  (NOTE: 4C ESCALATE + 4C PATH-TRAVERSAL were base-4-owned and were IMPLEMENTED
   this session — see report, not routed.)

phase-4D
  - 4 B-7 emitter plug-ins -> B-7.
  (RegistryCheck wiring -> D-11, see DECISIONS above. Designer dispatch decided
   above, no route needed.)

phase-4F
  - get_feasibility_log TS/Svelte consumer (Tauri command -> TS wrapper ->
    Svelte) -> C-22.
  - live-cloud rate metrics (replace 3 synthetic-proxy rate metrics via a live
    cloud smoke) -> C-22 (BLOCKED_MEASUREMENT until the live run).
  (NOTE: ELAI_DISABLE_FEASIBILITY_PROBE allowlist row was base-4-owned and was
   ADDED this session — see report.)

phase-4G
  - ELAI_DISABLE_AUTONOMOUS_GATE allowlist row -> B-22.
  - ELAI_GATE_MODE allowlist row -> B-22.
  - workflow/reasoning/B-22-decisions.md (GateDecision.decision string vs spec
    discrete SSB variant naming) -> B-22.
  - SSB action_type reconcile (add auto_advance_decision / autonomy_escalation /
    split_task_initiated variants OR canonicalize string encoding) -> B-22.

phase-4H
  - dashboard-v2 "self-modification" cargo feature allowlist row
    (type=experiment, owner=B-3, promotion=B-23, smoke=task_runner_chokepoint_
    contract.rs) -> B-3.
  - ELAI_LAYER1A_AUTO_APPLY allowlist row (type=experiment, owner/promotion B-23)
    -> B-23.

phase-4I
  - ELAI_VLM_IMAGE_VALIDATION_ENABLE allowlist row (promotion=E-16) -> E-16.
  - axtree::AxTreeAdapter into render_adapters/web.rs production path -> E-16.
  - HybridDiff + VlmConstrainedClient + StagehandWebAdapter + SaasJourneyRunner
    (implement missing components) -> E-14.

phase-4J
  - 47 EMPIRE-PENDING live-integration bodies (FactoryLineTransport bodies, MCP
    HTTP handler, EventFilter+RateLimiter, EmpireRepairPipeline, 4J.9 perf
    targets) -> C-34 (Empire-codebase-access gated).
  (NOTE: the `empire` cargo-feature allowlist row was base-4-owned and was ADDED
   this session — see report.)

phase-4K
  - 187 off-spec font-size instances (canvas/* + components/*) -> B-11.
  - line-height (base.css 1.5 -> 1.6 body / 1.4 code) -> B-11.
  - KnowledgeDrawer.svelte:791 residual 11px -> C-6.
  - visual.spec.ts Playwright snapshot suite -> C-7 / B-11.
  - FirstRunWizard.svelte (tier-preference prompt) -> B-12.
  - Sidebar icon count (7 actual vs 5 spec) -> B-12.
  - ELAI_DASHBOARD_OFFLINE allowlist row -> B-36.
  - docs/specs/track-d-v2-spec.md + B-plan-delta doc + per-command contract tests
    (dashboard_v2_*_contract.rs) -> B-36.
  - canonical live-activity surface decision (V2 orbit starmap vs V1 ActivityFeed
    auto-scroll/inline-preview-link) -> B-36.


GATE-FREEZE NOTE (R8)
  wire-dark-allowlist.toml is in .claude/gate-manifest.txt (line 14), so the three
  rows added this session (ELAI_SANDBOX_ENABLE, ELAI_DISABLE_FEASIBILITY_PROBE,
  empire) trip gate_freeze_check.py until an OUT-OF-BAND re-lock:
    python .claude/tools/gate_freeze_check.py --lock
  This is a sanctioned, human-reviewed flag registration (mirrors the existing
  "added by an executing run" rows already in the file). The executing run does
  NOT re-lock its own gate (R8).
