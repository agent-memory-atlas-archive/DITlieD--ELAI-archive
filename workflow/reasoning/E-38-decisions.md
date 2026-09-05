E-38 — AI roadmap 2026 signals + EU AI Act compliance gate — decisions
session: s-orch-e38 (campaign continuation)
plan: F:\.ELAI_workflow\docs\thefinalglance\E-plan\E-38_ai_roadmap_2026_signals.md

(a) SCOPE — TIER-A ONLY (9-of-9 campaign drift confirmed)

E-38 returned PLAN_DRIFTED_REQUIRES_IMPROVEMENT from both dispatched agents. Drift inventory:

  DRIFT-1 (HIGH): plan claims "C-15 PLANNED (ClockProvider injection)" — ClockProvider is shipped TODAY at crates/core/src/clock.rs (182 LOC, 198 referencing files). The plan's Task 2a needs to drop "PLANNED" and tighten to "no bare SystemTime::now/Instant::now outside ClockProvider-derived helpers" (per integration explorer DRIFT-MEDIUM observation).

  DRIFT-2 (HIGH): plan Task 5 RuntimeAdapter arch-agnostic claim — already TRUE. Trait at providers/src/runtime/types.rs:504-531 has 5 methods (backend / capabilities / health_check / complete / stream); zero is_dense/is_moe/requires_dense methods exist anywhere (grep verified). The contract test ships as a REGRESSION GUARD, not a fix.

  DRIFT-3 (HIGH): plan Task 8c audit_trail_liveness — already GREEN. fsm.rs:733-752 emits SsbPayload::PhaseTransition on every successful transition. Contract is a regression guard.

  DRIFT-4 (HIGH): plan adds 5 new SsbActionType variants (EuAiActComplianceBlocked / EuAiActComplianceVerified / McpTransportVersionLogged / MrcrUnverified / PostureDocStale). 7+ touchpoint × 5 = 35+ atomic edits + 288-golden alphabetical regen + SCHEMA 97 → 98 atomic bump. Grey-zone surgery; defer to Tier-B.

  DRIFT-5 (HIGH): plan modifies orchestrator/fsm.rs:622-631 (compliance pre-gate hook) — grey-zone. Pairs with Task 2 + DRIFT-4 SSB variants; ship atomically together in Tier-B.

  DRIFT-6 (HIGH): plan modifies plugins/mcp_bridge.rs — NOT OWNED. Plugins crate ownership is unclear; treat as AVOIDED until clarified.

  DRIFT-7 (MEDIUM): workflow/compliance/ + workflow/archetypes/ directories DO NOT EXIST. Plan's OWNED claim is aspirational. (workflow/vision_rag/ was also non-existent before E-36.A created its README.)

  DRIFT-8 (MEDIUM): EU AI Act / Digital Omnibus dates cited from plan text — needs WebFetch verification at exec time. Per memory:feedback_paper_citation_verify, 2 prior bad cites were caught this campaign. According to integration explorer's verification: GPAI Aug 2 2025 (live), transparency Aug 2 2026, high-risk Annex III POSTPONED Aug 2 2026 → Dec 2 2027 per Digital Omnibus May 7 2026 (artificialintelligenceact.eu/implementation-timeline).

  DRIFT-9 (LOW): MRCR v2 arXiv ID cited in plan needs anti-slopsquatting WebSearch (2 bad cites already this campaign).

  DRIFT-10 (LOW): plan task 1 + 6 imply strategy docs in docs/strategy/ — that directory does not exist; would need creation.

9-of-9 plans confirmed PLAN_DRIFTED_REQUIRES_IMPROVEMENT.

(b) WHAT TIER-A E-38 SHIPPED

  - crates/contracts/tests/e38_anti_pattern_contracts.rs (NEW, ~210 LOC)
    Four grep-style regression guards + one sanity test:
      • runtime_adapter_trait_stays_arch_agnostic — bans fn requires_dense / is_moe / DenseRuntime / MoeRuntime etc.
      • no_hardcoded_flash_attention_2_assumption — bans require_flash_attention_2 / REQUIRES_FA2 etc.
      • no_bespoke_agent_protocol_parallel_to_mcp — bans trait ElaiAgentProtocol / struct CustomAgentProto etc.
      • no_operator_screenshot_polling_loop — bans screenshot_loop / browser_screenshot_poll etc.
      • workspace_walker_finds_at_least_one_rs_file — sanity test ensuring the recursive walker actually scans files (so the other four can't false-pass on a broken walker).
    Walker skips target/, .git/, node_modules/, .elai/, indexes/, scip_indices/, models/. Self-skips this contract file so its own mention of the banned strings doesn't trip the assertion.

  - workflow/reasoning/E-38-decisions.md (this file)

(c) WHAT TIER-B DEFERRED + WHY

  Task 1 + 6 (strategy docs ELAI-2026-POSTURE.md + MoE-on-Ampere-tier.md in docs/strategy/) — defer reasons: (a) docs/strategy/ directory does not exist (DRIFT-10); (b) workflow tooling, gitignored — would not be in the commit payload anyway; (c) strategy docs should pair with the compliance gate + posture-staleness SSB row (PostureDocStale) in Tier-B for a complete story.

  Task 2 (EU AI Act compliance pre-gate at orchestrator/fsm.rs:622) — defer reasons: (a) grey-zone fsm.rs edit; (b) needs the 5 new SSB variants from DRIFT-4; (c) ~150 LOC at crates/orchestrator/src/compliance.rs (NEW grey-zone module) + ~30 LOC hook in fsm.rs. Belongs in a session that includes orchestrator/{root}.

  Task 3 (ClockProvider injection audit across the workspace) — defer reasons: ClockProvider shipped (DRIFT-1); the residual work is a sweep to migrate bare SystemTime::now() in non-entrypoint code. Mechanical refactor, deserves its own pass.

  Task 4 (MCP transport version SSB beacon — McpTransportVersionLogged) — defer reasons: plugins/mcp_bridge.rs is NOT OWNED (DRIFT-6); needs the SSB variant from DRIFT-4.

  Task 7 (audit_trail_liveness contract test in contracts crate) — defer reasons: would be a thin regression guard verifying fsm.rs:733-752 still emits PhaseTransition; ship in Tier-B alongside the EU AI Act gate's own contract.

  Task 8 (context_advertised contract — agent runtime advertises its context-window via MCP transport metadata) — defer reasons: needs the MCP transport version logging (Task 4) + ClockProvider audit (Task 3) for the time-decay semantics. Composite Tier-B item.

  Task 9 (.elai_cc.toml schema for compliance config — risk_class, jurisdiction, audit_retention_days) — defer reasons: needs the compliance gate (Task 2) to consume the schema.

  Task 10 (ComplianceDrawer.svelte) — defer reasons: graceful-empty over real producer pattern — producers (Tasks 2 + 4) must exist first. Path is src/lib/drawers/.

(d) WHY GREP-STYLE CONTRACT TESTS

Three of the four contract tests assert NEGATIVE space: a string MUST NOT appear in the workspace. This is the canonical "anti-pattern regression guard" shape. The advantages:

  - Cheap to write (no production code authored).
  - Cheap to run (file-walk + string-contains in <100ms for the workspace).
  - Loud failure mode: a future PR adding an `is_moe` method to RuntimeAdapter would fail the contract immediately, with the offending file path printed.
  - Captures social engineering deltas: "let's add a quick is_dense check" looks innocuous in a code review; the contract makes the rationale explicit.

The sanity test (workspace_walker_finds_at_least_one_rs_file) guards against a walker-implementation bug masking the other four — if the walker silently breaks, the bans become false-positive PASSes.

(e) WHY THE SIDECAR SAFETY-NET INVARIANT MATTERS

The no_hardcoded_flash_attention_2_assumption test exists because the sidecar (python/elai_inference/server.py — AVOIDED this run) defaults to SDPA with FA2 as opt-in (ELAI_FORCE_SDPA=1 + ELAI_TRY_FLASH_ATTN_2). A future "we always have FA2 in 2026" assumption would silently degrade older Ampere boxes + non-CUDA hosts. The contract spans the whole workspace including the (non-Rust) sidecar's potential future Rust extensions — even though the sidecar is AVOIDED, anyone adding a Rust crate that hard-codes FA2 fails the contract.

(f) WHY MCP IS THE STANDARD + WHY THE CONTRACT GUARDS IT

ELAI uses Model Context Protocol (MCP) for tool / agent communication (per CLAUDE.md + memory entries on MCP bridge / MCP tool calls). The no_bespoke_agent_protocol_parallel_to_mcp test bans trait/struct/enum names like ElaiAgentProtocol / BespokeAgentProtocol / CustomAgentProto. Anyone proposing a custom protocol must delete this contract first — which forces the proposal into a visible RFC rather than landing as a "quick refactor".

(g) WHY SCREENSHOT POLLING IS BANNED

Operator-style automation (OpenAI Operator, Claude Computer Use) takes browser screenshots every N seconds and feeds them to a vision-LLM. Three failure modes: (1) $/screenshot dominates cost; (2) per-screenshot latency stacks into minutes; (3) every screenshot is a privacy footgun (PII / session tokens / etc visible in the frame). ELAI uses LSP + structured tool output for automation (E-52.A diagnostic envelope was the foundation for that path). The contract makes the choice load-bearing.

(h) TIER-B SHIPPED (s-25120 completion-drive session 4/4 E-cluster)

Tier-A had deferred 8 of the 10 plan tasks. Tier-B closes Tasks 1 + 2 + 4 + 6 + 9 + 10 + the strategy docs. Concretely:

  - SCHEMA bump 174 -> 175. SsbActionType::AiRoadmapSignalEvent variant added (Pattern-2 single-variant + 5-arm RoadmapSignalKind discriminator: compliance_blocked / compliance_verified / mcp_transport_version_logged / mrcr_unverified / posture_doc_stale). TOTAL_VARIANTS 447 -> 448; payloads count 385 -> 386. SQL CHECK constraint + as_sql_str + from_str + SsbPayload variant + action_type discriminant arms + test_payload + all_test_payloads round-trip arms all wired. See `crates/orchestrator/src/ssb.rs`.

  - DRIFT-4 RESOLVED via Pattern-2: instead of the plan's 5 separate variants (which would have forced SCHEMA 174 -> 179 + 5 SQL-CHECK entries + 5 from_str arms + 5 atomic payload-variant edits), the session prompt's KEY CORRECTION directs Pattern-2 application (canonical encoding for new SSB additions in this codebase, mirrors E-29 Tier-B AgenticObservabilityEvent + E-31 CodingModelLeaderboardEvent precedent). Saves 4 schema bumps + 4 round-trip fixture edits.

  - crates/orchestrator/src/compliance.rs (NEW, ~360 LOC incl. unit tests): pre_gate_planning_to_testing_first(&PlanContext) -> ComplianceVerdict. 4 assertions (ClockProviderIntact / DeterministicSeed / TemplateHashChain / AuditTrailLive). TemplateHashChain degrades to Skipped (transition allowed) when template_archive_present=false per plan Open Question 1 (D-1 PLANNED not yet shipped). build_block_payload + build_verified_payload helpers shape AiRoadmapSignalEventPayload rows for the SSB. 7 unit tests inside the module.

  - crates/orchestrator/src/roadmap_signals.rs (NEW, ~170 LOC incl. tests): roadmap_signals_enabled() truthiness parser for the env-gate; build_mrcr_unverified_payload helper; check_posture_doc_age probe (returns None for missing/fresh, Some(payload) when mtime > 90 days). 3 unit tests.

  - crates/orchestrator/src/fsm.rs: FsmError::ComplianceBlocked variant + Display impl ("PLANNING -> TESTING_FIRST refused: EU AI Act assertion ... failed; ...; see docs/strategy/ELAI-2026-POSTURE.md"). ComplianceProvider trait + Fsm.compliance_provider: Option<Arc<dyn ComplianceProvider>> field + with_compliance_provider / set_compliance_provider builder + setter. Pre-gate hook inserted between the AuditingBlocked guard and the AUDITING gate, fires when target==TestingFirst AND source in {Planning, FeasibilityProbe}. Block path emits ComplianceBlocked row + returns Err. Pass path emits ComplianceVerified row + falls through. SSB emission gated by ELAI_AI_ROADMAP_SIGNALS_ENABLE; the GATE itself runs unconditionally when a provider is installed (env-gate controls observability, not policy).

  - crates/orchestrator/src/dispatch/mcp.rs: McpClientDispatch::new now emits one AiRoadmapSignalEvent::McpTransportVersionLogged SSB row per process boot via emit_transport_version_once() (idempotency via AtomicBool MCP_TRANSPORT_VERSION_LOGGED + compare_exchange). Wire-dark unless ELAI_AI_ROADMAP_SIGNALS_ENABLE=1. Constants MCP_TRANSPORT_CURRENT_DEFAULT = "streamable_http" (aligned with plugins/src/mcp_bridge.rs::McpTransportMode::Default = StreamableHttp) + MCP_TRANSPORT_ROADMAP_TARGET = "stateless+tasks+registry-q4-2026+saml-oidc-pkce-q2-2026".

  - DRIFT-6 ROUTED AROUND: mcp_bridge.rs is in the plugins crate; orchestrator depends on plugins, not the reverse, so the emission was placed at the orchestrator-side McpClientDispatch::new (already a known wiring site). No plugins/mcp_bridge.rs edit needed. Future cleanup: hoist the constants closer to the bridge if/when the plugins crate gets an SSB emitter trait.

  - Dashboard surface (DASHBOARD REQUIRED per session prompt override against plan's frontend_pair: null):
    - crates/dashboard-v2/src-tauri/src/commands_e38.rs (NEW, ~220 LOC): get_roadmap_signals_state Tauri command. Reads ai_roadmap_signal_event SSB rows, projects to RoadmapSignalsState { last_compliance, mcp_transport, posture_stale, mrcr_unverified_count, grand_total }. Tile dedup keeps the most-recent row per kind by ts_ms; mrcr_backends collected into a BTreeSet so the count is distinct-backends, not row-count.
    - Registered in dashboard-v2/src-tauri/src/lib.rs mod + invoke_handler.
    - crates/dashboard-v2/src/lib/components/RoadmapPanel.svelte (NEW, ~120 LOC): 4-tile renderer (compliance / MCP transport / posture-doc-stale / MRCR-unverified). Invoke<RoadmapSignalsState> on onMount + 5s setInterval (mirrors CriticDecisionPanel.svelte scaffold).
    - Mounted in dashboard-v2/src/lib/windows/MonitorWindow.svelte alongside CriticDecisionPanel + EvalCorpusPanel.

  - IPC v2 UNCHANGED per memory:project_b40_shipped. The RoadmapSignalsState DTO lives in dashboard-v2/src-tauri/src/commands_e38.rs (not in crates/ipc-types/); read-only SSB projection does not require an IPC variant bump.

  - Contract tests (3 NEW under crates/contracts/tests/):
    - e38_ai_roadmap_signals_ssb_contract.rs -- SCHEMA bump assertion, action_type round-trip, RoadmapSignalKind 5-arm coverage, env-gate constant name, COMPLIANCE_BIT_* powers-of-two, threshold constants, SsbPayload discriminant mapping, msgpack round-trip for each of the 5 kinds, action_type negative-lookup guard.
    - e38_compliance_pre_gate_contract.rs -- 4 failing fixtures (one per ComplianceAssertion arm) + 1 passing fixture (all assertions set) + 1 D-1-absent Skipped fixture. block_payload + verified_payload helper field-coverage tests. AUDIT_TRAIL_MIN_SSB_ROWS >= 1 invariant.
    - e38_mcp_transport_version_contract.rs -- transport constants (current default + roadmap target milestone-string coverage), roadmap_signals_enabled wire-dark default, build_mrcr_unverified_payload field-coverage, check_posture_doc_age None for missing/fresh files. Note: testing the >90-day stale path needs filetime crate which is NOT a workspace dep; per anti-slopsquatting + no-new-deps rules we keep the test fresh-file-negative + rely on the orchestrator-side roadmap_signals.rs unit tests for the rest.

  - Strategy docs (Tier-A had deferred per DRIFT-10):
    - docs/strategy/ELAI-2026-POSTURE.md (NEW, ~140 lines): 7 sections covering 5 signals + 7 bets + per-lab + hardware + regulatory + insurance + refresh discipline. Every claim has a citation; every source line is WebSearch-verified May 2026.
    - docs/strategy/MoE-on-Ampere-tier.md (NEW, ~80 lines): expert-offload design space. Cites arXiv:2510.12357 (MoBiLE), arXiv:2512.16473 (CPU-GPU collab), arXiv:2509.08342 (MoEpic), arXiv:2602.03495 (DALI). All four IDs WebSearch-verified May 2026 (resolves plan task 6 "do not fabricate" requirement).

  - CLAUDE.md HARD RULE added under "Hard rules" section: "E-38 2026 posture reference" pointing to docs/strategy/ELAI-2026-POSTURE.md + the 90-day staleness check + the compliance pre-gate location.

  - .claude/rulebook/session-protocol.md: appended "E-38 posture-doc-stale check" rule. At session start, check the posture doc mtime; surface a `posture_doc_stale` warning if > 90 days old.

(i) DRIFT INVENTORY CLOSURE

  DRIFT-1 (C-15 ClockProvider PLANNED claim) -- handled by the compliance gate's ClockProviderIntact assertion treating ClockProvider as expected-present (the PlanContext.has_clock_provider bool); the gate refuses transitions where the field is false. Future audit-sweep to migrate bare SystemTime::now() in non-entrypoint code is still deferred (Task 3 mechanical refactor; not E-38 scope).

  DRIFT-2 (RuntimeAdapter arch-blind) -- Tier-A already shipped the regression-guard contract test (`e38_anti_pattern_contracts.rs`). Tier-B does NOT re-author it.

  DRIFT-3 (audit_trail_liveness already GREEN) -- Tier-A regression-guard test exists. The compliance pre-gate's AuditTrailLive assertion adds a SECOND guard at the FSM-edge level (verifies SSB row count threshold before TESTING_FIRST entry).

  DRIFT-4 (5 SSB variants atomic surgery) -- RESOLVED via Pattern-2 (1 variant + 5-arm discriminator = 1 atomic schema bump).

  DRIFT-5 (fsm.rs grey-zone) -- handled atomically with DRIFT-4 in this Tier-B drop.

  DRIFT-6 (plugins/mcp_bridge.rs not owned) -- ROUTED around via dispatch/mcp.rs (orchestrator-side, owned). The plugins crate is not touched.

  DRIFT-7 (workflow/compliance/ + workflow/archetypes/ dirs missing) -- not applicable: this Tier-B uses crates/orchestrator/src/compliance.rs (code path) + docs/strategy/ (docs path). workflow/* dirs were aspirational and not used.

  DRIFT-8 (EU AI Act dates verification) -- verified May 2026 via WebSearch + posture doc Section 5. GPAI live Aug 2 2025; transparency Aug 2 2026; Annex III postponed Dec 2 2027.

  DRIFT-9 (MRCR v2 arXiv) -- WebSearch verified arXiv:2409.12640.

  DRIFT-10 (docs/strategy/ dir missing) -- created. Two strategy docs authored.

(j) WHAT REMAINS DEFERRED OUT-OF-SCOPE FOR E-38

  - Task 3 (ClockProvider audit-sweep across non-entrypoint code paths). Mechanical refactor; spawn as a patch-N or a small B-N once user signals readiness.

  - Task 8 (context_advertised contract over MCP transport metadata + ClockProvider time-decay semantics). Composite item; needs Task 3 sweep + the MRCR scoring loop to be fully wired before the contract can assert anything beyond the existing mrcr_unverified producer.

  - Live emission RUNPOD-PENDING -- the env-gate ELAI_AI_ROADMAP_SIGNALS_ENABLE stays default-off. The substrate (producers + SSB variant + dashboard tile + contract tests) is complete; flipping the env-gate live is a separate operational step.

  - filetime crate dev-dep for the >90-day stale-mtime contract test. Anti-slopsquatting + no-new-deps rule blocked; the orchestrator-side roadmap_signals.rs unit test covers the missing path (None for missing file), and a follow-up patch-N can add filetime if needed.

END E-38-decisions.md
