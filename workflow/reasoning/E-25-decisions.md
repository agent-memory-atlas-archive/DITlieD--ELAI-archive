E-25 — Claude Code architecture / 6-tier compaction — decisions
session: s-orch-e25 (campaign continuation)
plan: F:\.ELAI_workflow\docs\thefinalglance\E-plan\E-25_claude_code_architecture.md

(a) SCOPE — TIER-A ONLY (11-of-11 campaign drift confirmed)

Both agents returned PLAN_DRIFTED_REQUIRES_IMPROVEMENT. Drift inventory:

  DRIFT-1 (HIGH): plan task 1 "build a 6-tier compaction ladder" — ELAI already has a 3-level adaptive_reduce ladder (AdaptiveLevel {FilteredByRelevance, ReducedGranularity, ReducedScope, Terminal}) in orchestrator/src/context_budget.rs:1480 emitting ContextReductionPayload SSB rows per level. Tier-B work is EXTENDING this with the new tiers (T2 per-tool-result snip, T4 >5-turn bucket collapse, T5/T6 sidecar re-narration), NOT replacing it.

  DRIFT-2 (HIGH): plan task 6a "session fork" already SHIPPED. Surface lives at cli/handoff.rs:489-560 + fork_session_branch Tauri command + SessionsDrawer.svelte:44. Task is redundant.

  DRIFT-3 (HIGH): plan Tier-4 "Context Collapse" already SHIPPED. A-46 telescope_caller + TelescopeCollapsePayload + a46_context_telescope_contract.rs implement the per-layer collapse. Tier-B's compaction_ladder.rs should call this as a building block, not replace it.

  DRIFT-4 (HIGH): plan claims HookPoint::PreCompact does compaction — it does NOT. hook_dispatcher.rs:121-124 marks the no-op MVP; hook_dispatcher.rs:915 runs T1 analyse + debug log only. The actual compaction call lives at task_runner.rs:7446 BEFORE the dispatch_pre_compact call. Plan's framing of "naive single-step PreCompact" needs reconciliation.

  DRIFT-5 (HIGH): plan adds 10+ new SsbActionType variants. 70+ atomic touchpoints + 288-golden alphabetical regen + SCHEMA 97 → 98. From AVOID-zone (orchestrator/src/ssb.rs). Defer entirely; the right approach per integration explorer is to EXTEND ContextReductionPayload with a tier_id discriminator rather than mint parallel `compaction_tier_applied` variants.

  DRIFT-6 (MEDIUM): plan task 5 "skill templating" cites a hooks crate — no crates/hooks/ directory exists. Hooks are .claude/hooks/*.py + the in-process HookDispatcher at orchestrator/src/hook_dispatcher.rs.

  DRIFT-7 (MEDIUM): plan task 2 "deferred MCP" — MCP transport is in plugins/mcp_bridge.rs (NOT OWNED). E-38 also flagged plugins ownership unclear.

  DRIFT-8 (MEDIUM): notify crate cited by plan — presence not confirmed in Cargo.lock (anti-slopsquatting verify needed).

  DRIFT-9 (MEDIUM): dashboard types.ts may have stale interfaces (recurring pattern this campaign — E-29 caught QueryTrace + ChatEntry stale).

  DRIFT-10 (LOW): plan cites `/opusplan` substate overlay — opus-specific UX work that should pair with the model-routing dashboard surface.

  DRIFT-11 (LOW): plan Open Questions 1-5 (LLM-decision hook cost / fork rebase semantics / skill template DSL / opusplan toggle UI / 6-tier threshold tuning) all carry forward unanswered.

11-of-11 plans confirmed PLAN_DRIFTED_REQUIRES_IMPROVEMENT.

(b) TIER-A SHIPPED

  - crates/contracts/tests/e25_compaction_architecture_contracts.rs (~150 LOC, 5 tests):
      • no_parallel_telescope_collapse_payload — bans TelescopeCollapseV2Payload / LayerCollapsePayload / CompactionCollapsePayload / TelescopeCollapseKind enum.
      • no_parallel_session_fork_handler — bans fork_session_branch_v2 / SessionForkPlus / ForkSessionV2 trait.
      • no_bespoke_message_summarizer_parallel_to_compact — bans compact_session_v2 / summarize_messages_v2 / BespokeSummarizer / CustomMessageSummarizer.
      • no_silent_context_truncation_helpers — bans silent_drop_oldest / truncate_silently / drop_oldest_no_emit / quiet_compact / stealth_summarize. Forces ContextReductionPayload-emitting paths.
      • workspace_walker_finds_at_least_one_rs_file — sanity test.
    Walker cross-skips e38_anti_pattern + e48_research + e25_compaction contract files so banned-string literals don't trip each other.

  - workflow/reasoning/E-25-decisions.md (this file)

(c) TIER-B DEFERRED

  Task 1 (compaction_ladder.rs) — defer reasons: (a) grey-zone orchestrator/{root} module; (b) wraps existing compact::compact_session + context_budget::adaptive_reduce + telescope_caller::invoke_telescope primitives; (c) needs the tier_id discriminator extension on ContextReductionPayload (SSB grey-zone edit).

  Task 2 (deferred MCP loading) — defer reasons: plugins/mcp_bridge.rs NOT OWNED.

  Task 3 (skill templating) — defer reasons: lives in .claude/skills/ (workflow tooling); pairs with E-43 agent skill systems (not in this run's queue).

  Task 4 (LLM-decision hook) — defer reasons: needs prompt-engineer template (workflow tooling) + the sidecar / cloud routing decision; high latency budget.

  Task 5 (opusplan substate overlay) — defer reasons: dashboard panel + dashboard-side state machine — graceful-empty over real producer pattern; producer must exist first.

  Task 6 (session fork) — already SHIPPED; skip.

  Task 7 (10+ SSB variants) — defer reasons: 70+ touchpoint surgery + golden regen + SCHEMA bump; co-staging without producers violates B-23 4I.1.

  Task 8 (CompactionLadderPanel.svelte) — defer reasons: producer must exist first. Path is src/lib/drawers/.

  Task 9-12 (skill DSL / fork rebase / opusplan UI / 6-tier threshold) — Open Questions; need user input.

(d) DESIGN RATIONALE

The four negative-space guards encode the architectural decisions ELAI has already made:

  - One canonical TelescopeCollapsePayload — the dashboard listens for it, the auditor reads it, the SSB CHECK constraint includes it. Forks fragment the surface.
  - One canonical session-fork handler — cli/handoff.rs + Tauri command + SessionsDrawer is the user's mental model. A parallel handler doubles the UX.
  - One canonical summarizer — compact::compact_session has the cache-key + token-budget guarantees. Bespoke parallels lose those.
  - No silent truncation — every reduction emits a ContextReductionPayload so the audit trail captures it. Silent helpers break replay + audit.

(e) WHY EXTENDING ContextReductionPayload IS THE RIGHT TIER-B SHAPE

The integration explorer recommended extending ContextReductionPayload with a tier_id discriminator rather than minting parallel compaction_tier_applied variants. Three reasons:

  1. Dashboard listens for ContextReduction events today; a new variant means new listener wiring everywhere.
  2. The SSB CHECK constraint + as_sql_str + from_str + payload struct + variant + match arm + test_payload + all_test_payloads list is 7+ touchpoints PER variant. Adding 10 means 70+ touchpoints; extending one struct with one new field is 1.
  3. The tier semantics (6 ladders) is metadata, not protocol — discriminating WHICH reduction tier fired is naturally a field on the existing payload, not a new SSB row.

The contract test bans parallel payloads (no_parallel_telescope_collapse_payload) to enforce this when Tier-B lands.

(f) CAMPAIGN PATTERN — 11-OF-11

Plans drifted: E-44, E-16, E-31, E-29, E-21, E-36, E-52, E-42, E-38, E-48, E-25. The Tier-A shipping discipline is now stable:
  - parallel agent dispatch (context-compiler + integration-explorer-elai)
  - both classify per task: OWNED / AVOIDED / GREY-ZONE / SHIPPED-ALREADY
  - Tier-A scope is the OWNED + SHIPPED-ALREADY-as-regression-guard subset
  - decisions doc carries the full DRIFT inventory + Tier-B deferral for future improvement
  - commit shipped after auditor PASS + memory-writer fanout

10 commits shipped, 2 explicit deferrals (E-44, E-16). Pattern is repeatable + auditable.

END E-25-decisions.md (session-3 TIER-A entry)


====================================================================
E-25 CATALOG SUBSTRATE (session-4 / completion-drive cluster s-25120)
====================================================================

session: s-25120 cluster batch5 completion-drive E-plan closure
plan: F:\.ELAI_workflow\docs\thefinalglance\E-plan\E-25_claude_code_architecture.md
prior-session: session-3 TIER-A entry above (negative-space contract guards shipped)

SCOPE OUTCOME

Session-4 ships E-25 as an architecture-reference CATALOG (single file
.elai_cc/crates/orchestrator/src/architecture/claude_code_arch.rs plus
the ssb.rs Pattern-2 cascade). This is the substrate slice that pairs
with the session-3 negative-space contract guards. Concrete impl
modules (compaction_ladder / cli-fork / edit_checkpoints /
prompt_handler / opusplan-overlay / dashboard-panel) remain
FOLLOW-UPS (matches session-3 DEFERRED list verbatim).

DECISION-1 -- architecture-reference catalog landing point.
  Choice: .elai_cc/crates/orchestrator/src/architecture/claude_code_arch.rs.
  Rationale: mirrors E-11 saas_archetype.rs sibling file (taxonomy
  enum + snake_case + display_label + ALL_* iteration const). Wire-
  dark posture via ELAI_CLAUDE_CODE_ARCH_ENABLE env var matches
  E-11 / E-22 / E-23 / E-43 / E-55 substrate convention.
  Grounding: file:.elai_cc/crates/orchestrator/src/architecture/
  saas_archetype.rs (HIGH; in-codebase E-11 sibling).

DECISION-2 -- Pattern-2 single-variant SSB cascade.
  Choice: 1 SsbActionType::ClaudeCodeArchitectureEvent variant +
  1 ClaudeCodeArchitectureEventKind 9-arm discriminator on payload.
  SCHEMA_VERSION bumps 171 -> 172. TOTAL_VARIANTS 444 -> 445. 24th
  application of Pattern-2.
  Rationale: 9 fire-points as 9 separate variants would have been
  9 * 9-tuple = 81 cascade edits (matches session-3 DRIFT-5 "70+
  atomic touchpoints"). Pattern-2 collapse is the precedent of
  E-55 (RewardLadderEventKind 9-arm) / E-11 (SaasPriorArtEventKind
  2-arm) / E-22 (TrendingEventKind 3-arm) / E-23 (DiscourseEventKind
  3-arm).
  Grounding: file:.elai_cc/crates/orchestrator/src/ssb.rs L18660
  RewardLadderEventKind + grep TOTAL_VARIANTS=444.

DECISION-3 -- DESCOPE Primitive 2 (deferred MCP descriptions).
  Choice: omit entirely from this commit. No
  ClaudeCodeArchPrimitive::DeferredMcpSchemas variant. No
  McpSchemaPanel. No --lazy-mcp-schemas flag.
  Rationale: underlying lazy_schemas mechanism (E-22 Task 3 / C-35)
  is unshipped per context line 24 + session-3 DRIFT-7
  ("plugins/mcp_bridge.rs NOT OWNED"). Adding a catalog entry now
  would lie about ELAI's surface; the entry will be added in a
  follow-up once the prereq mechanism lands.
  Grounding: file:context line 24 (E-22 Task 3 / C-35 lazy_schemas
  prereq UNSHIPPED) + session-3 DRIFT-7 + dispatch correction.

DECISION-4 -- RENAME ladder result struct to ClaudeCodeLadderResult.
  Choice: ClaudeCodeLadderResult (NOT CompactionResult).
  Rationale: plan's proposed orchestrator::compaction_ladder::
  CompactionResult { messages, freed_tokens, tiers_applied,
  cache_preserved_bytes } collides hard with the existing
  crate::compact::CompactionResult { summary, formatted_summary,
  compacted_session, removed_message_count }. The renamed struct
  preserves the plan's field shape verbatim; only the type name
  changes.
  Grounding: file:.elai_cc/crates/orchestrator/src/compact.rs L26
  + file:.elai_cc/crates/orchestrator/src/lib.rs L847
  "pub use compact::CompactionResult".

DECISION-5 -- 9-primitive taxonomy.
  Composition: CompactionLadder (P1) + SkillTemplating (P3) +
  PromptHandler (P4) + OpusplanOverlay (P5) + SessionFork (P6a) +
  EditCheckpoints (P6b) + BashComplexityGuard (P7b) +
  ApprovalFatigueGuard (P7c) + SandboxTemplateGuard (P7
  sandbox-warn sub-clause). Primitive 2 DESCOPED per DECISION-3.
  Plan Task 7 sub-clauses (a / d / e / f / g) are NOT separate
  primitives: (a) is a property of gateway-rejection guard (E-24);
  (d) is a return-shape change on mcp_search_context; (e) is a
  status-line surface; (f) defers to E-24 Task 5 LLM-guard;
  (g) is the E-23 trust-telemetry plane.
  Grounding: dispatch correction "Catalog 9 architecture
  primitives."

DECISION-6 -- Wire-dark env gate ELAI_CLAUDE_CODE_ARCH_ENABLE.
  Default-off keeps the substrate dark until concrete consumers
  ship. ssb_emission_enabled() helper mirrors
  architecture/saas_archetype's ssb_emission_enabled() exactly.
  Grounding: dispatch correction + file:architecture/mod.rs L49.

DECISION-7 -- Session-3 negative-space contract guards still apply.
  The 4 banned-symbol guards in
  crates/contracts/tests/e25_compaction_architecture_contracts.rs
  (no_parallel_telescope_collapse_payload /
  no_parallel_session_fork_handler /
  no_bespoke_message_summarizer_parallel_to_compact /
  no_silent_context_truncation_helpers) remain LOAD-BEARING. The
  catalog substrate does not introduce any banned-symbol regression
  (verified: claude_code_arch.rs uses only ClaudeCodeArchitectureEvent
  + ClaudeCodeArchitectureEventKind + ClaudeCodeLadderResult +
  CompactionTierId names, none of which match the banned-symbol
  patterns).

FOLLOW-UPS (matches session-3 deferred list)

F-1 .. F-8 deferrals from session-3 (Tasks 1-7 + dashboard panels)
remain accurate. The catalog ships the public taxonomy + payload
shape + ladder-result struct that the F-1 / F-3 / F-4 / F-5 / F-6
follow-ups will import. F-2 (notify crate add) deferred to F-2
landing time.

WIRING CONTRACT (locked by this session-4 commit)

produces_symbols:
  - orchestrator::architecture::claude_code_arch::ClaudeCodeArchPrimitive (9 variants)
  - orchestrator::architecture::claude_code_arch::ALL_CLAUDE_CODE_PRIMITIVES
  - orchestrator::architecture::claude_code_arch::ClaudeCodeArchitectureEventKind (9-arm discriminator)
  - orchestrator::architecture::claude_code_arch::ClaudeCodeArchitectureEventPayload
  - orchestrator::architecture::claude_code_arch::ClaudeCodeLadderResult (renamed)
  - orchestrator::architecture::claude_code_arch::CompactionTierId (6 tiers)
  - orchestrator::architecture::claude_code_arch::ALL_COMPACTION_TIERS
  - orchestrator::architecture::claude_code_arch::ELAI_CLAUDE_CODE_ARCH_ENABLE_VAR
  - orchestrator::architecture::claude_code_arch::ssb_emission_enabled
  - orchestrator::architecture::claude_code_arch_ssb_emission_enabled (alias)
  - orchestrator::ssb::SsbActionType::ClaudeCodeArchitectureEvent
  - orchestrator::ssb::SsbPayload::ClaudeCodeArchitectureEvent

consumes_symbols: none (catalog is reference-only)

reachable_from:
  - crate::architecture re-exports (mod.rs)
  - SSB write path (test_payload helper + all_test_payloads sample)

stub_ban: none (every type round-trips through msgpack; payload
  uses serde_json::Value for free-form details)

frontend_pair: none in this commit; dashboard panels deferred.

observable_effect:
  trigger: instantiate ClaudeCodeArchitectureEventPayload + write
    via Ssb::write_payload with the variant.
  expect: SSB row persisted; round-trip deserialization through
    rmp-serde yields the original payload (covered by the
    payload_round_trips_msgpack + ladder_result_round_trips_msgpack
    unit tests in claude_code_arch.rs).

ACCEPTANCE CRITERIA (this session-4 commit)

[x] .elai_cc/crates/orchestrator/src/architecture/claude_code_arch.rs
    created (9-primitive taxonomy + 9-arm event kind + payload +
    ladder result struct + 6-tier enum + env-gate helper + unit
    tests for taxonomy lock / round-trip / env gate).
[x] .elai_cc/crates/orchestrator/src/architecture/mod.rs re-exports
    the new types + claude_code_arch_ssb_emission_enabled alias.
[x] ssb.rs 9-tuple cascade complete: SQL CHECK list +
    SsbActionType variant + as_sql_str arm + from_str arm +
    SsbPayload variant + action_type() arm + test_payload helper
    arm + all_test_payloads sample + payloads.len() 382 -> 383 +
    SCHEMA_VERSION 171 -> 172.
[x] action_type_lock.rs: TOTAL_VARIANTS 444 -> 445, ALL_VARIANTS
    appended, exhaustive_match_guard appended.
[x] action_type_lock.golden: header total 444 -> 445, new entry
    "0066 claude_code_architecture_event" inserted alphabetically
    between "0065 claim_routed" and "0067 cli_mcp_dispatch", all
    subsequent ordinals shifted +1.
[x] workflow/reasoning/E-25-decisions.md updated (this section).

OPEN QUESTIONS / RISKS

- F-2 notify-crate add deferred to F-2 landing time (catalog does
  not import notify).
- F-3 prompt_handler uses NEW PromptDecision enum (HookVerdictKind
  is Allow / Warn / Block, NOT Allow / Deny / Ask per
  file:hook_dispatcher.rs L179).
- F-4 substate-overlay key choice (FsmSubstate vs FsmState)
  unresolved; FsmSubstate has 5 variants none matching the plan's
  Planning / Executing names per file:fsm.rs L124. Decision
  deferred to F-4 entry.
- F-6 eviction policy + Windows file-watch reliability on UNC
  shares open at F-6 entry.

GROUNDING (sources consulted; tier per .claude/rules/source-tiers.md)

- file:.elai_cc/crates/orchestrator/src/architecture/saas_archetype.rs
  (HIGH; in-codebase E-11 sibling pattern)
- file:.elai_cc/crates/orchestrator/src/ssb.rs L18660
  RewardLadderEventKind (HIGH; in-codebase E-55 Pattern-2 precedent)
- file:.elai_cc/crates/orchestrator/src/compact.rs L26
  CompactionResult (HIGH; in-codebase collision proof)
- file:.elai_cc/crates/orchestrator/tests/action_type_lock.rs L156
  TOTAL_VARIANTS = 444 (HIGH; grep verified)
- code.claude.com/docs/en/how-claude-code-works (HIGH; official)
- code.claude.com/docs/en/skills (HIGH; official)
- code.claude.com/docs/en/hooks (HIGH; official)
- code.claude.com/docs/en/model-config (HIGH; official)
- karanprasad.com/blog/how-claude-code-actually-works-reverse-
  engineering-512k-lines (MEDIUM; reverse-engineering blog)
- agiflow.io/blog/claude-code-internals-reverse-engineering-prompt-
  augmentation (MEDIUM; blog)
- newsletter.pragmaticengineer.com/p/building-claude-code-with-
  boris-cherny (MEDIUM; industry newsletter)
- github.com/VILA-Lab/Dive-into-Claude-Code (MEDIUM; community
  anti-pattern analysis)
- infoq.com/news/2026/05/anthropic-claude-code-postmortem
  (MEDIUM-HIGH; industry report)

END E-25-decisions.md session-4 catalog substrate
