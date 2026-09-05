C-PLAN WIRE-REMEDIATION DEFER DECISIONS
session: s-wire-remediation-2026-06-05 (resume)
date: 2026-06-05
context: live concurrent H-plan session shares the tree and owns ssb.rs / action_type_lock golden / shadow_exec.rs. User instructed: pause SSB-adding work, limit to disjoint non-SSB files, leave shadow_exec for H-stream. This doc records what C-plan landed vs what is deferred and why.

LANDED THIS SESSION (C-plan Round 1, non-SSB + SSB-already-written, audited PASS 0.95)
  C-3   Tauri knowledge surface: 3 new Svelte consumers (get_feasibility_log/get_research_budget/get_source_tier_distribution) + SsbViewer::query_tier_stats real SQL; freshness_state WIRE-DARK[C-3] honest known:false.
  C-6   KnowledgeDrawer font 11px->10px.
  C-23  SessionsDrawer branch-event banner (consumes existing get_session_branches, no new cmd).
  C-26  3 EBNF grammars (import_block/function_signature/error_type).
  C-10  [stagnation] section in phase_4G.toml (config only) + C-10-decisions.md (already existed).
  C-24  graph_export_dispatch.rs (4 real assertions). Dashboard blast-radius surface DEFERRED (design decision).
  C-21  append_constraints_from_rules_db wired (task_runner.rs:5667); SteeringVectorRefreshTriggered SSB (already written into shared ssb.rs, reconciled to 234/511).
  C-31  record_lines_written wired (task_runner.rs:9389); language_limit_overrides seeded (deterministic_auditor.rs:190).
  C-44  BackendCostAccrued emit (runtime_orchestrator.rs:332); citation arXiv:2603.24477 WebSearch-verified.
  C-36  emit_mucoco_inconsistency/skip helpers (metamorphic_testing.rs:667/709) WIRE-DARK[C-36], detector GPU-deferred.
  C-2   3 contract tests in crates/contracts/tests (ssb_c2_*); Layer-4 real-emit assertions.
  Verify: cargo check lib exit 0 (cli/orchestrator/knowledge/providers); svelte-check 0 errors; contracts compile exit 0. No fabrication.

DEFERRED — R9 ALLOWLIST ROWS (blocked by live H-session; allowlist is hot SSB-registration surface)
  These new default-off flags shipped WITHOUT a wire-dark-allowlist.toml row this session to avoid clobbering H's concurrent allowlist edits. Add when SSB work resumes (H-session done):
  mucoco_inconsistency_detected (C-36), mucoco_skip_reason (C-36), steering_vector_refresh_triggered (C-21),
  stagnation_spinning_detected / stagnation_oscillation_detected / lateral_thinking_activated / goal_drift_warning / watchdog_timeout (C-10/C-2),
  plus the never-started allowlist-only flags: ELAI_EXPERIENCE_DISTILL_ENABLE (C-49), ELAI_DISABLE_FEASIBILITY_PROBE (C-22),
  empire cargo feature (C-34), ELAI_C42_MCP_AUTHZ_EMIT_ENABLE (C-42), ELAI_APPLICATION_SKILLS_ENABLE (C-41),
  credit_attribution (C-45), case_base (C-47), C-50 distill, github (C-40), rrk_reranker (C-58), BackendCostPanel (C-44).

DEFERRED — GPU/RUNPOD PILOT TAILS (code half may be landed; the run is RunPod 2x3090 / sidecar)
  C-37 per-role decoding pilot (PerRoleDecodingApplied emit + apply_role_default_decoding child-startup wire) ; C-45 cantante 450-rollout pilot ;
  C-47 case-base 50-session replay pilot (emitter wire is WIRE-DARK[C-47], structural prereqs in non-owned files: ElaiRuntimeConfig.tool_use field + DefaultMcpBridge attach at dispatch/mcp.rs:308) ;
  C-50 RL pipeline + Greenfield-5 ; C-17 scale fixture run ; C-19 cargo bench elai-plugins ; C-26 sidecar grammar bench ;
  C-36 mucoco sidecar detector pilot ; C-42 500K-sequence fuzz ; C-46 SelectiveDelegator pilot (WIRE-DARK[RUNPOD-PENDING] routing_cmd.rs:23).

DEFERRED — NET-NEW-ARCH (feature-plan work, not remediation wire-ups)
  C-34 Empire bodies (47 EMPIRE-PENDING, blocked on external Empire codebase; only allowlist row is remediation) ;
  C-24 blast-radius dashboard surface (BlastRadiusDrawer G-3 vs new panel = design decision post orbit-redesign) ;
  C-40 RestApiAdapter GitHub PR (needs reqwest dep approval + GITHUB_TOKEN) ;
  C-14 phase0 dead-code strip (compact.rs/oauth.rs/prompt.rs, 30+ callers, needs PRODUCT SIGN-OFF).

DEFERRED — NET cloud-smoke (local half may be landed; assertion needs a live cloud Sonnet run)
  C-9 autonomous-infra (gate_decision AutoAdvance SSB) ; C-15 determinism smoke ; C-23 fork/merge smoke ; C-28 mutation-runner cloud smoke.

DEFERRED — 9 NOT_STARTED EXTERNAL HARNESSES (dated-defer; new crates + vendored corpora + RunPod; belong to a benchmark plan, not remediation)
  C-43 phase-4H scope-freeze (new crates/self_mod/ + macros + ablation_harness) ;
  C-51 Aider Polyglot harness (new crates/task_runner/ + PolyglotAdapter + 6 sandbox shims) ;
  C-53 NF4 kernel-watch in-context counter (benches/nf4_e2e.rs + 30-task fixture) ;
  C-54 SWE-bench Verified 3-config (500-task corpus + docker eval) ;
  C-55 Aider Polyglot leaderboard submit (depends C-51+C-40+C-44; DRAFT PR to Aider-AI/aider) ;
  C-56 codebase-memory DKG 31-repo eval (CodebaseMemoryAdapter + Sonnet judge) ;
  C-57 OpenHands-Index/SWE-Cycle/SWE-Atlas (depends C-54; 3 adapters + 3 corpora) ;
  C-58 AgentForge/RRK reranker head-to-head (AutoCodeAI vendor + rrk.rs reranker) ;
  C-59 cross-config benchmark master report (capstone, depends C-54..C-58).
  Recommendation: fold C-54..C-59 + C-37/C-45/C-50 pilots into ONE RunPod batch session (queue item 7, alongside G-6 SAFEDELTA), do G-6 first.

SKIPPED — already SHIPPED_WIRED (19): C-1,4,5,8,11,12,13,16,18,20,25,27,29,30,32,33,35,38,39.
