E-12 — Production fullstack codegen — decisions
session: s-orch-e12 (campaign continuation)
plan: F:\.ELAI_workflow\docs\thefinalglance\E-plan\E-12_production_fullstack_codegen.md

(a) SCOPE — TIER-A ONLY (13-of-13 campaign drift confirmed)

E-12 returned PLAN_DRIFTED_REQUIRES_IMPROVEMENT from both dispatched agents. Drift inventory:

  DRIFT-1 (HIGH): plan inherits E-11's IdeaCriticVerdict + LevelAuditVerdict.outcome + EscalationTrigger location drifts. E-11.A documented these; E-12 carries them forward unresolved.

  DRIFT-2 (HIGH): plan cites FrameworkProfile + FrameworkAdapter + SERVER_ENTRY_SEEDS + production_checklist/ + AuditorMode::ProductionGrade + IdeaFramerSecurityPanel.svelte + 5 fabricated ProjectClass variants + get_framework_profile — ALL net-new, zero existing callers in workspace. Greenfield.

  DRIFT-3 (HIGH): plan cites orchestrator::file_plan path — real path is elai_core::file_plan (.elai_cc/crates/core/src/file_plan.rs).

  DRIFT-4 (HIGH): plan cites cli_common.rs:337 — real surface is types.rs:504 RuntimeAdapter trait. Same fabricated-line-number pattern as E-31 / E-52 / E-25.

  DRIFT-5 (HIGH): plan claims "extended from E-11" for IdeaFramerSecurityPanel.svelte — E-11.A shipped Tier-A only; the panel doesn't exist yet (deferred to E-11.B).

  DRIFT-6 (HIGH): plan adds 2 new SsbActionType variants + 5 new MistakeBook categories — coordinated bump across orchestrator/{root} schema + golden lock + 9-tuple lockstep. Grey-zone.

  DRIFT-7 (HIGH): no template engine in workspace (tera / handlebars / askama / minijinja / liquid all absent). Plan implies template-engine adoption; needs explicit user confirmation per anti-stale "No new dependencies without confirmation". The scaffold crate uses byte-level `{{ident}}` substitution (scaffold/substitute.rs) — adequate for SERVER_ENTRY_SEEDS without a new dep.

  DRIFT-8 (HIGH): 16 framework version literals cited (Next.js 15 / React 19 / Svelte 5 / Tailwind v4 / Biome 2.3 / Ruff / Prisma 7 / Drizzle / Hono / FastAPI 0.115 / Pulumi Neo / Wasp / Encore / RedwoodSDK / Astro / shadcn / Sentry SDK / OTel) — ZERO anti-slopsquatting WebSearch verification done. Mandatory before any version literal lands.

  DRIFT-9 (MEDIUM): existing scaffold substrate (orchestrator::task_runner::ScaffoldApplier trait at line 2173 + DashboardScaffoldApplier impl) is the natural FrameworkAdapter shape — Tier-B should mirror it, not fork.

  DRIFT-10 (MEDIUM): FIM markers in scaffold::fim.rs are the natural "MiniCritic-protected seed" mechanism for SERVER_ENTRY_SEEDS — Tier-B should reuse, not parallel.

  DRIFT-11 (MEDIUM): plan's .claude/tools/framework-conventions/ + workflow/archetypes/ create dual sources of truth for scope-planner. Reconciliation: workflow/archetypes/ (created by E-11.A) is canonical; .claude/tools/framework-conventions/ stays as workflow tooling reference if it lands at all.

  DRIFT-12 (LOW): plan's elai_cc.toml extensions need a schema migration; coordinate with config_archive infrastructure.

13-of-13 plans confirmed PLAN_DRIFTED_REQUIRES_IMPROVEMENT.

(b) TIER-A SHIPPED (BIG ARCHITECTURAL REFACTOR)

  - crates/contracts/tests/common/mod.rs (NEW, ~120 LOC) — factored shared workspace walker. Auto-skips ALL files matching `e<N>_*_contracts.rs` via `is_sibling_negative_space_contract()`. Adding a 6th contract requires ZERO edits to existing siblings (down from 5 edits with the 4-way cross-skip pattern).

  - 4 existing contract files refactored to use the shared mod:
      • crates/contracts/tests/e11_idea_to_saas_contracts.rs (~85 LOC down from ~180)
      • crates/contracts/tests/e25_compaction_architecture_contracts.rs (~95 LOC down from ~150)
      • crates/contracts/tests/e38_anti_pattern_contracts.rs (~85 LOC down from ~210)
      • crates/contracts/tests/e48_research_contracts.rs (~95 LOC down from ~190)
    Each now imports `mod common;` + uses `common::{collect_hits, format_hits, workspace_rs_files}` instead of duplicating walker code. Total reduction: ~370 LOC of duplication removed.

  - crates/contracts/tests/e12_production_codegen_contracts.rs (NEW, ~115 LOC, 4 negative-space + 1 sanity):
      • no_parallel_framework_adapter_trait — bans FrameworkAdapterV2 / BespokeFrameworkAdapter / FullstackEmitterV2 / AdHocScaffoldEmitter.
      • no_template_engine_dep_pollution — bans tera / handlebars / askama / minijinja / liquid imports. Forces scaffold byte-substitute pattern.
      • no_emit_without_production_checklist_gate — bans emit_scaffold_no_checklist / write_files_unaudited / skip_production_gate / unchecked_codegen / raw_emit_no_audit. Forces the production checklist gate.
      • no_hardcoded_unpinned_framework_versions_in_source — bans literal version strings (next@15.0.0, sveltekit@2.20, fastapi==0.115, axum = "0.8.0"/"0.9.0"). Pins go in elai_cc.toml / templates.
      • workspace_walker_finds_at_least_one_rs_file — sanity.

  - workflow/reasoning/E-12-decisions.md (this file)

(c) TIER-B DEFERRED

  Task 1 (FrameworkProfile + 5 concrete adapters) — defer reasons: grey-zone orchestrator/{root}; mirror ScaffoldApplier trait (DRIFT-9); requires explicit user confirmation on any new template-engine dep.

  Task 2 (production_checklist/ runner) — defer reasons: depends on E-11 Tier-B AuditorMode::ProductionGrade (DRIFT-5).

  Task 3 (SERVER_ENTRY_SEEDS const map + FIM markers) — defer reasons: extend scaffold::fim.rs (DRIFT-10); pure Rust const map, no new dep.

  Task 4 (5 fabricated ProjectClass variants) — defer reasons: ProjectClass currently has fewer variants than plan claims; need agent reconciliation pass.

  Task 5 (2 new SsbActionType + 5 new MistakeBook categories) — defer reasons: coordinated bump + golden regen.

  Task 6 (IdeaFramerSecurityPanel.svelte) — defer reasons: cited as "extended from E-11" but E-11.A shipped Tier-A only.

  Task 7 (16 framework version anti-slopsquatting WebSearch verify) — defer reasons: workflow-tooling pass; pair with workflow/archetypes/ template population.

  Task 8 (elai_cc.toml schema extensions) — defer reasons: needs producers from Tasks 1-3.

  Task 9 (.claude/tools/framework-conventions/) — defer reasons: workflow/archetypes/ is canonical (E-11.A); avoid dual sources of truth.

  Task 10 (ScaffoldEmitterPanel.svelte) — defer reasons: graceful-empty over real producer.

(d) DESIGN RATIONALE — THE SHARED-WALKER REFACTOR

The 4-way cross-skip list (e11 + e25 + e38 + e48) hit the threshold E-25.A + E-11.A flagged: each new negative-space contract was costing 5 edits (4 sibling updates + new file). The N*N edit cost makes the pattern unsustainable past 5 contracts.

The fix: factor the workspace walker into `tests/common/mod.rs` with an `is_sibling_negative_space_contract(name)` regex-shaped predicate matching `e<N>_*_contracts.rs`. Each contract file does `mod common;` + uses `common::workspace_rs_files()`. Adding a 6th contract = 1 edit (new file only).

Validation: 5/5 tests pass on e11 + e25 + e38 + e48 + e12 (25/25 total across the 5 contracts). The refactor is byte-for-byte semantically equivalent for the 4 existing contracts; only the e48 web_search-scope filter retains its file-path predicate (since "ban these strings inside ONE crate" is a different shape from "ban everywhere").

Net win: ~370 LOC of duplicated walker code removed. Pattern now scales linearly.

(e) WHY THE FOUR E-12 ANTI-PATTERNS

  - One canonical FrameworkAdapter trait — Tier-B needs single dispatch surface so the production checklist gate runs uniformly across all emitted stacks.
  - No template-engine dep pollution — anti-stale rule (CLAUDE.md "No new dependencies without confirmation"). scaffold byte-substitute is enough for SERVER_ENTRY_SEEDS.
  - Production checklist gate must run — the load-bearing demo-vs-deploy guard E-11 ships gets bypassed if any helper writes files without the gate.
  - Version pins in elai_cc.toml / templates — keeps version-bump cost low + prevents silent literal-mismatch between source and templates.

(f) CAMPAIGN PATTERN — 13-OF-13 LOCKED

E-44, E-16, E-31, E-29, E-21, E-36, E-52, E-42, E-38, E-48, E-25, E-11, E-12. 12 commits shipped, 2 explicit deferrals (E-44, E-16). Tier-A discipline + shared-walker refactor + negative-space contract pattern are now stable.

E-50 (the last plan) depends on E-11 + E-12 — both shipped Tier-A. E-50 is unblocked.

END E-12-decisions.md
