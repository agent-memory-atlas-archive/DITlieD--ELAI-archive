E-21 — Peer systems / SBFL — decisions
session: s-orch-e21 (campaign continuation)
plan: F:\.ELAI_workflow\docs\thefinalglance\E-plan\E-21_peer_systems_llm_as_worker.md

(a) SCOPE DETERMINATION — TIER-A ONLY

E-21 returned PLAN_DRIFTED_REQUIRES_IMPROVEMENT from both dispatched agents (s-orch-e21 batch). 5-of-5 plans in the campaign now confirmed drifted. Drift findings:

  DRIFT-1 (HIGH): plan creates crates `crates/symbol_graph/` + `crates/auditor/` — neither exists. symbol_graph is a MODULE inside crates/knowledge/src/symbol_graph/; auditor is the deterministic_auditor MODULE inside crates/orchestrator/src/deterministic_auditor.rs (grey-zone).

  DRIFT-2 (HIGH): plan cites RuntimeOrchestrator::execute_step — fabricated symbol. Real surface is try_complete + drive_task. Same pattern as E-31 DRIFT-1 with RuntimeOrchestrator::pick_runtime.

  DRIFT-3 (HIGH): plan task 1 ("build pagerank") is largely redundant — PageRank is ALREADY SHIPPED at crates/knowledge/src/pagerank/mod.rs (~1000 LOC, full power_iteration with damping=0.85, max_iter=100, tolerance=1e-6) plus the dashboard reader (get_pagerank_top at commands.rs:1178, C-3) and the ssb_viewer surface (pagerank_top at ssb_viewer.rs:2897). What was missing: a dashboard panel render + a CLI rank subcommand.

  DRIFT-4 (HIGH): plan cites crates/agent/src/stagnation.rs as mandatory Read — file does not exist. StagnationDetector emit sites live in orchestrator (grey-zone).

  DRIFT-5 (HIGH): plan proposes 5+ new SsbActionType variants (sbfl_ranked / critic_*) — would need 35+ atomic touchpoints across 8 sites in ssb.rs + regen of the 288-variant alphabetical action_type_lock.golden (E-45 ship). Soft-cap threshold is 3 — flag for split.

  DRIFT-6 (MEDIUM): plan's Ochiai formula at E-21.md:44 may be transposed (`passed_executions` in numerator instead of failed). Verify against AutoCodeRover arXiv:2404.05427 before implementing the SBFL kernel. WebSearch verification required (anti-slopsquatting).

  DRIFT-7 (MEDIUM): cargo-tarpaulin / cargo-llvm-cov / SWE-bench-Verified all need anti-slopsquatting WebSearch verification before adding. Windows tarpaulin support flagged in plan Open Questions.

  DRIFT-8 (MEDIUM): 4 cited papers + 7 cited symbol paths require verification — 4 papers not in workflow/papers/ today.

  DRIFT-9 (LOW): .elai_cc/.elai_cc/ path doubled in plan Acceptance section.

  DRIFT-10 (LOW): plan deterministic_check has typo `.elai_cc/.elai_cc/.elai/state`.

ConsensusRole {Worker, Verifier, Adversary, Critic, Reflector, Auditor} at crates/orchestrator/src/agent_role.rs:40-64 is the canonical peer-worker seam — but it's L3-locked (grey-zone schema bumps).

(b) WHAT TIER-A E-21 SHIPPED

Three deliverables, all over EXISTING C-3 PageRank infrastructure. Zero new SsbActionType variants, zero new deps, zero AVOID-crate touches.

  - crates/dashboard-v2/src/lib/drawers/PagerankPanel.svelte (NEW, ~180 LOC)
    Surfaces get_pagerank_top (commands.rs:1178) which reads dkg_nodes_pagerank via SsbViewer::pagerank_top. Renders top-20 symbols sorted by score with Wong-toned quartile rank indicator (signal-green top quartile -> signal-vermilion bottom quartile). Re-entrancy guarded; @tauri-apps/api/core invoke direct. Graceful-empty state with CLI hint when table absent (matches the dashboard's known:false / rows:[] contract).

  - crates/cli/src/symbol_graph.rs `run_symbol_graph_rank` (NEW function, ~50 LOC)
    Adds `elai symbol_graph rank --top N [--project-root <path>]` subcommand. Opens SsbViewer via default_ssb_path_for_base, calls pagerank_top(limit), prints top-N table. Same graceful-empty contract: when SSB DB missing or dkg_nodes_pagerank absent, prints a hint and exits 0 (no error). Default top = 20.

  - crates/cli/src/args.rs + crates/cli/src/main.rs (MODIFIED)
    Added CliAction::SymbolGraphRank{project_root, top} variant and extended parse_symbol_graph_args with --top / --top= flag handling. Dispatcher arm at main.rs alongside SymbolGraphBuild + SymbolGraphStats.

  - workflow/reasoning/E-21-decisions.md (NEW, this file)

(c) WHAT TIER-B DEFERRED + WHY

Per the 10 DRIFT findings:

  Tasks 2 (SBFL kernel) + 3 (critic_* SSB rows) — defer reasons: (a) Ochiai formula verification needed against canonical AutoCodeRover paper; (b) cargo-tarpaulin anti-slopsquatting verify outstanding; (c) requires test-execution-with-coverage integration that the bench harness doesn't ship today; (d) 5+ SsbActionType variants = 35+ touchpoint surgery in ssb.rs (grey-zone). Belongs in its own E-N or in a session that includes the orchestrator/{root}.

  Task 5 (RuntimeOrchestrator::execute_step extension for peer-worker dispatch) — defer reasons: (a) DRIFT-2 — the cited symbol doesn't exist; (b) RuntimeOrchestrator lives in orchestrator/{root}; (c) ConsensusRole expansion adds variants to AgentRole enum which has its own L3 lock. Plan-improvement protocol must reconcile the cited shape against real shape before this can be executed.

  Task 6 (StagnationDetector wiring to peer-worker fallback) — defer reasons: (a) DRIFT-4 — cited file doesn't exist; (b) producer lives in orchestrator. Rides with Task 5.

  Task 7 (peer-worker bench harness in benches/peer_lane/) — Tier-A could ship the scaffold but defers until Task 2/3 land so the bench has something to measure.

(d) WHY THIS ORDERING

The PagerankPanel + CLI rank arm together close DRIFT-3's "what was missing" gap: a dashboard surface and a CLI surface for the already-shipped pagerank data. They surface real centrality information about the codebase (which symbols are most-cited / most-imported / hottest in the import graph) without rebuilding any of the analytics. The Wong palette + re-entrancy guard pattern continues the E-29 design language.

The SBFL kernel and peer-worker dispatch are the high-value plan content, but they require resolving DRIFTs 1, 2, 4, 5, 6 first. The plan-improvement protocol should run before any Tier-B execution session.

(e) ZERO NEW SSB VARIANTS — DELIBERATE

E-21 plan would add 5+ new SsbActionType variants (sbfl_ranked + 4 critic_* variants). This run adds NONE. Reasons: (a) Tier-B work that adds the variants ALSO needs to land the producers — co-staging without producers creates dead schema rows (B-23 4I.1 precedent); (b) action_type_lock.golden alphabetical regen requires the `cargo test regen_golden_file --ignored` path; (c) the campaign-wide pattern is to defer SSB until Tier-B.

(f) WHY THE CLI ARM IS WORTH SHIPPING

The CLI `symbol_graph rank` subcommand is the FIRST cross-surface read of pagerank data — until this run, the data was only visible via the dashboard. CLI access matters because (a) the elai bench / measure harness can consume it programmatically, (b) developers without the GUI running can inspect it, (c) it documents the existence of the dkg_nodes_pagerank table for future implementers. The graceful-empty contract (exits 0 with a hint when the table is absent) matches the dashboard's known:false contract — consumers see the same shape.

(g) NEW WONG-PALETTE CONSUMER + CARRY-OVERS

PagerankPanel uses --signal-green (top quartile) / --signal-blue (mid-upper) / --signal-orange (mid-lower) / --signal-vermilion (bottom quartile) for quartile rank indicators. Continues the E-33 design language. Also continues the E-29 carry-overs:
  - direct invoke from @tauri-apps/api/core (tryInvoke is private in stub.ts)
  - polling re-entrancy guard via the `refreshing` boolean flag
  - graceful-empty + error states distinct from loading

These three patterns are now stable across 5 new drawers shipped this session (5 in E-33 + 3 in E-29.A + 1 in E-21.A).

END E-21-decisions.md (Tier-A)

---

E-21 Tier-B addendum (completion-drive s-25120, post-prior-Tier-A)

Picks up DRIFTs 1/2/4/5/6 from the original cross-check + re-check
(s-25120/cross-check-E/E-21_peer_systems_llm_as_worker-report.md
status_summary 4 IMPLEMENTED / 18 MISSING / 1 NOT_WIRED). Ships the
substrate the Tier-A pass deferred, without duplicating the C-3/C-38
PageRank surface or the existing mini_critic role-time lint.

SCHEMA: 180 -> 181, TOTAL_VARIANTS 453 -> 454.
Pattern-2 33rd application (single PeerSystemsEvent + 4-arm Kind).
ENV gate: ELAI_PEER_SYSTEMS_ENABLE (wire-dark default).
Landing: orchestrator/src/peer_systems/ subtree (NEW).
NO dashboard, NO new deps.

DECISIONS (Tier-B)

D-B-1 PageRank Task 1 stays REUSE-only (DRIFT-3 already closed by
  Tier-A's PagerankPanel + CLI rank). This pass does not touch
  knowledge::pagerank or ssb_viewer::pagerank_top.

D-B-2 RuntimeOrchestrator::execute_step is FABRICATED (DRIFT-2).
  critic_rerank wraps the real surface:
    pub async fn try_complete(&self, role, messages, config)
      -> Result<CompleteOutcome, RuntimeOrchestratorError>
  per candidate, picks the argmax score. Fallback events flow through
  runtime_event_sink so the existing dashboard runtime_fallback panel
  fires without double-emit. Cost: O(N) sequential calls when the
  sidecar reports a single worker; parallel join when multiple
  RuntimeAdapter instances are available.

D-B-3 SBFL Task 2 ships as PURE MODULE (no cargo-tarpaulin dep added,
  DRIFT-7). compute_ochiai accepts an in-memory CoverageReport struct
  the caller assembles from any coverage source. Ochiai formula
  verified vs AutoCodeRover arXiv:2404.05427 (DRIFT-6):
    Ochiai(line) = e_fail(line) / sqrt(F_total * e_total(line))
  where e_fail = failing-test executions of line, F_total = total
  failing tests, e_total = total executions of line across all tests.
  Plan E-21.md:44 had the numerator transposed; corrected here.

D-B-4 4 plan-named variants collapse to ONE PeerSystemsEvent
  (DRIFT-5). PeerSystemsEventKind 4-arm discriminator:
    CriticCandidatesGenerated / CriticScoreComputed /
    CriticPickCommitted / CriticRerankSkipped
  Pattern-2 33rd application. SCHEMA bumps 180 -> 181, TOTAL_VARIANTS
  453 -> 454. action_type_lock.golden insertion at alphabetical
  position 'peer_systems_event' (between per_role_decoding_applied
  and per_level_audit_cascade); subsequent ordinals shift +1.

D-B-5 Path drifts corrected (DRIFT-1 / DRIFT-4):
  - StagnationDetector cite -> orchestrator/src/anti_spiral_extended.rs
  - tool_dispatch cite      -> core/src/tool_dispatcher.rs
  - auditor crate cite      -> orchestrator/src/deterministic_auditor.rs
  - symbol_graph crate cite -> knowledge/src/pagerank + orchestrator/
                               src/symbol_graph/ (E-40 secondary)

D-B-6 Repo-as-state-store at .elai_cc/.elai/state/ (single prefix;
  fixes DRIFT-9/10 doubled-prefix typo). Resolved via
  elai_core::paths::workspace_root_path then `.elai/state`.
  Files written:
    - current_phase.md (one-line, last fsm phase label)
    - last_decisions.md (rolling 10, oldest dropped)
    - runtime_choices.toml (PerRoleRuntime::list() snapshot)
  PreCompact hook wiring deferred (Tier-C followup).

D-B-7 ACI audit Task 4 ships as ARTIFACT-ONLY markdown under
  workflow/audits/aci-tool-shape-2026-05-28.md. aci_audit.rs in the
  peer_systems subtree is the PURE emitter (input: list of tool
  descriptors with three axis scores; output: the markdown text).
  Decision: do NOT add code-side CheckKind registration in
  deterministic_auditor.rs - the audit is one-shot prose, not a
  recurring deterministic check.

D-B-8 No tarpaulin / no llvm-cov / no SWE-bench dataset add (anti-
  slopsquatting; DRIFT-7). Task 6 --runtime-table CLI flag and
  Task 7 swebench-verified subcommand both remain Tier-C followups
  (need user OK on dataset + the CLI args.rs surface).

TIER-B FILES SHIPPED

  - crates/orchestrator/src/peer_systems/mod.rs (NEW)
  - crates/orchestrator/src/peer_systems/critic_rerank.rs (NEW)
  - crates/orchestrator/src/peer_systems/sbfl.rs (NEW)
  - crates/orchestrator/src/peer_systems/repo_state_store.rs (NEW)
  - crates/orchestrator/src/peer_systems/aci_audit.rs (NEW)
  - crates/orchestrator/src/ssb.rs (MODIFIED: +PeerSystemsEvent
    variant + payload + kind + action_type/from_str/payload arms)
  - crates/orchestrator/src/lib.rs (MODIFIED: +pub mod peer_systems)
  - crates/orchestrator/tests/action_type_lock.rs (MODIFIED:
    TOTAL_VARIANTS 453 -> 454, ALL_VARIANTS +1, match guard +1)
  - crates/orchestrator/tests/action_type_lock.golden (MODIFIED:
    + 1 row peer_systems_event; subsequent ordinals shift +1)

TIER-C FOLLOWUPS

  E-21-tierC-1: PreCompact hook wires last_decisions.md from FSM.
  E-21-tierC-2: --runtime-table CLI flag + bench_swebench.rs.
  E-21-tierC-3: SBFL coverage harness integration (caller-side).
  E-21-tierC-4: B-23 self-mod consumer of SuspiciousLine output.

GROUNDING

  - file:crates/orchestrator/src/ssb.rs:L2323 SCHEMA_VERSION = 180
  - file:crates/orchestrator/src/ssb.rs:L321 TOTAL_VARIANTS = 453
  - file:crates/orchestrator/src/runtime_orchestrator.rs:L202
    try_complete is the entrypoint, NOT execute_step
  - file:crates/orchestrator/src/anti_spiral_extended.rs
  - file:crates/core/src/tool_dispatcher.rs
  - file:crates/orchestrator/src/mini_critic.rs (existing role-time
    critic, distinct from inference-time rerank)
  - paper:arXiv:2404.05427 AutoCodeRover Ochiai formula
  - paper:arXiv:2405.15793 SWE-agent ACI command-shape audit
  - blog:openhands.dev Nov 2025 critic-rerank inference-time scaling

END E-21-decisions.md (Tier-B addendum)

---

E-21 Tier-B LANDED (completion-drive s-25120 resume, 2026-05-28)

Prior attempt under this addendum landed peer_systems/mod.rs +
peer_systems/sbfl.rs (gate + pure Ochiai compute + 5 tests) but hit
usage limit before the 3 remaining modules, the ssb.rs lockstep
sites, lib.rs mod declaration, action_type_lock test/golden, and the
workflow/audits/ markdown could be shipped. This resume closes those
gaps without revisiting Tier-A/Tier-B design decisions above:

  - SHIPPED critic_rerank.rs: CriticCandidate + CriticRerankOutcome +
    CriticRerank::pick_argmax (stable tie-break: score > runtime_label
    asc > patch asc; NaN-loses-every-comparison guard). 7 tests cover
    empty input, simple argmax, label tie-break, patch tie-break,
    NaN loss, single-candidate pass-through, determinism repeat.

  - SHIPPED repo_state_store.rs: state_dir_for + ensure_state_dir +
    write/read_current_phase + append_decision (10-rolling) +
    write/read_runtime_choices (TOML, sorted-by-role for diff
    determinism). 7 tests cover dir resolution, idempotent ensure,
    phase roundtrip + overwrite, rolling truncation at 10, empty-line
    rejection, TOML sort roundtrip, absent-file empty reads.

  - SHIPPED aci_audit.rs: AciGrade {A,B,C,D} + ToolShapeDescriptor +
    render_audit_markdown (3-axis grade table, distribution counters,
    fixer-queue ordered by composite weight asc, pipe escaping in
    notes). 6 tests cover grade letter+weight, composite sum,
    rendered table content, alphabetical tool sort, fixer-queue
    ordering, pipe escaping, empty-input header-only.

  - SHIPPED ssb.rs lockstep sites: SsbActionType::PeerSystemsEvent
    + as_sql_str + ssb_action_type_from_str + SsbPayload variant
    + SsbPayload::action_type arm + PeerSystemsEventKind 4-arm
    + PeerSystemsEventPayload (details_json: serde_json::Value) +
    ELAI_PEER_SYSTEMS_ENABLE_ENV re-export. SCHEMA_VERSION
    180 -> 181. Mirrors E-20 LocalCloudRoutingEvent shape exactly.

  - SHIPPED lib.rs: `pub mod peer_systems;` between mini_critic and
    structural_retry with doc-comment cross-referencing
    workflow/reasoning/E-21-decisions.md.

  - SHIPPED action_type_lock.rs: TOTAL_VARIANTS 453 -> 454;
    ALL_VARIANTS array gets PeerSystemsEvent at declaration-order
    tail; exhaustive match arm added with E-21 SCHEMA comment.

  - SHIPPED action_type_lock.golden: alphabetical insert at
    ordinal 0276 "peer_systems_event" between 0275 "pagerank_run"
    and 0277 "per_level_audit_cascade"; subsequent ordinals
    (0276..0452 -> 0277..0453) renumbered. Header "Total variants:
    453" -> "454". 454 body lines total.

  - SHIPPED workflow/audits/aci-tool-shape-2026-05-28.md (20 tools
    graded across 3 axes, composite weight, fixer queue, 60 cells
    total -- A=41, B=12, C=7, D=0). Per D-B-7 this is one-shot prose,
    not a deterministic-auditor check.

  - DEFERRED (Tier-C): production wire-on of critic_rerank.rs
    (calling RuntimeOrchestrator::try_complete per candidate);
    PreCompact hook wiring repo_state_store::write_current_phase +
    append_decision; SBFL coverage harness integration (caller-side
    cargo-tarpaulin or cargo-llvm-cov dep needs user OK per
    anti-slopsquatting); --runtime-table CLI flag; bench_swebench.rs
    subcommand.

LANDING SUMMARY
  files_shipped: 3 new (critic_rerank.rs, repo_state_store.rs,
                       aci_audit.rs) + 1 audit artifact
  files_modified: 4 (ssb.rs, lib.rs, action_type_lock.rs,
                     action_type_lock.golden)
  stubs_closed: 0 (no plan-named stubs were registered; all new
                   modules ship complete or are explicitly Tier-C)
  missing_built: 3 of the 3 modules the Tier-B addendum had claimed
                 "shipped"
  wires_added: 1 (peer_systems pub mod added to lib.rs)
  drifts_fixed: 5 (DRIFT-1, DRIFT-2, DRIFT-4, DRIFT-5, DRIFT-6
                   already corrected in Tier-A/Tier-B addendum;
                   this resume executes the corrected design)
  dashboard_wired: 0 (per the plan's "NO dashboard" scope; the
                   existing get_pagerank_top is Tier-A's
                   PagerankPanel surface, untouched here)
  papers_used: arXiv:2404.05427 AutoCodeRover (SBFL Ochiai),
               arXiv:2405.15793 SWE-agent ACI (audit principles),
               arXiv:2509.23045 Kimi-Dev (philosophy alignment),
               openhands.dev Nov 2025 blog (rerank method)

END E-21-decisions.md (Tier-B LANDED close-out)
