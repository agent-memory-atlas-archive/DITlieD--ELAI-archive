E-11 — Idea-to-SaaS scope clarity / prior art — decisions
session: s-orch-e11 (campaign continuation)
plan: F:\.ELAI_workflow\docs\thefinalglance\E-plan\E-11_idea_to_saas_prior_art.md

(a) SCOPE — TIER-A ONLY (12-of-12 campaign drift confirmed)

E-11 returned PLAN_DRIFTED_REQUIRES_IMPROVEMENT from both dispatched agents. Drift inventory:

  DRIFT-1 (HIGH): plan WIRING_CONTRACT cites `IdeaCriticVerdict` type — does NOT exist. Real types in orchestrator/src/idea_framing/ are FramedIdea / CriticFinding / CriticRound. Plan needs reconciliation.

  DRIFT-2 (HIGH): plan task 9 references `LevelAuditVerdict.outcome` field — does NOT exist. Real struct has `pub verdict: Verdict`; `mark_cascade` is a free fn at L384, not a field.

  DRIFT-3 (HIGH): plan task 5 references `EscalationTrigger::HumanIntervention` in `anti_spiral_extended.rs` — WRONG FILE. EscalationTrigger lives at autonomous_gate.rs:400. (Similar to E-31 DRIFT-1's fabricated symbol pattern.)

  DRIFT-4 (HIGH): plan task 4 (per-level critic at Step+BabyStep) depends on C-31 FILE_PLAN which is PLANNED, not shipped. Cannot ship until C-31 lands.

  DRIFT-5 (HIGH): plan task 2 + 9 (15-item production checklist Tier-C sandbox runner) requires crates/orchestrator/src/sandbox/ — in this run's AVOID list. Plan's own Open Questions flag this.

  DRIFT-6 (HIGH): plan adds 6 new SsbActionType variants (FramingHitlBlocked / ScopeClarityScored / ProductionAuditFailed / DemoVsDeployChasmDetected / PackageVerifierBlocked / EscalationTriggerFired) — 42+ atomic touchpoints + 288-golden alphabetical regen + SCHEMA 97 → 98. Grey-zone orchestrator/{root} edit; defer entirely.

  DRIFT-7 (HIGH): plan task 6 IdeaFramerSecurityPanel.svelte mount target — OrchestrationDrawer.svelte path needs verification, and lib/drawers/ exists today (E-29.A + E-21.A confirmed). The plan's mount path needs explicit dashboard-explorer reconciliation.

  DRIFT-8 (MEDIUM): plan introduces SaasArchetype enum + workflow/archetypes/ directory — neither exists. Tier-A this run creates the workflow/archetypes/README.md scaffold (mirroring E-36 vision_rag pattern); the enum itself ships in Tier-B alongside the producer.

  DRIFT-9 (MEDIUM): plan task 4 EditFormat enum (Unified | SearchReplace | FullFile) — net-new, zero existing callers. Should pair with the producer (Aider-style diff routing) so the enum has consumers; do not co-stage without consumers.

  DRIFT-10 (MEDIUM): plan cites 9 prior-art products (v0.dev / bolt.new / replit / cursor / lovable / horizon / fly.io / vercel + 1 more). Integration explorer verified ZERO of them appear in code or elai.map — no slopsquatting risk in EXECUTING but cite verification still needed in Tier-B's prior_art/ docs.

  DRIFT-11 (LOW): plan's [research_presets.production_saas] config block needs to land in a config file the existing research presets surface consumes. Config ownership unclear.

  DRIFT-12 (LOW): IdeaFramerSecurityPanel test fixtures — none exist; would need to be authored alongside the producer.

12-of-12 plans confirmed PLAN_DRIFTED_REQUIRES_IMPROVEMENT.

(b) TIER-A SHIPPED

  - crates/contracts/tests/e11_idea_to_saas_contracts.rs (~180 LOC, 5 tests):
      • no_parallel_critic_verdict_type — bans IdeaCriticVerdict / CriticVerdictV2 / CriticRoundOutcome enum. Canonical type is CriticRound (DRIFT-1 reconciliation).
      • no_demo_grade_scaffold_emit — bans emit_demo_scaffold / build_demo_grade_app / for_show_only_scaffold / SCAFFOLD_DEMO_GRADE. Forces explicit demo-class opt-in.
      • no_parallel_package_verifier — bans PackageVerifierV2 / BespokePackageVerifier / QuickPackageCheck / skip_package_verification. Slopsquatting gate must be the single safety::PackageVerifier path.
      • no_quick_audit_mode_bypassing_15_item_checklist — bans quick_audit_mode / skip_production_checklist / AuditorMode::DemoOnly / SkipChecklist / audit_without_checklist. AuditorMode::ProductionGrade (Tier-B) cannot be bypassed via a parallel quick path.
      • workspace_walker_finds_at_least_one_rs_file — sanity mirror.
    Walker cross-skips all 4 sibling contracts (e11 + e25 + e38 + e48).

  - workflow/archetypes/README.md (NEW) — scaffolding doc, mirrors workflow/vision_rag/ pre-E-36 pattern.

  - workflow/reasoning/E-11-decisions.md (this file)

  - 3 sibling contract test files (e25 / e38 / e48) — extended their walker cross-skip lists to include e11_idea_to_saas_contracts.rs. 4-way cross-skip now in place; the campaign-pattern note in E-25.A about factoring to a shared workspace_walker becomes more urgent as the skip-list grows.

(c) TIER-B DEFERRED

  Task 1 (scope_clarity_score field on CriticRound) — defer reasons: requires extending the shipped struct + SSB variant + dashboard wiring. Grey-zone.

  Task 2 (15-item production checklist runner — Tier-C sandbox) — defer reasons: requires orchestrator/src/sandbox/ (AVOID-zone).

  Task 3 (PackageVerifier in crates/safety) — defer reasons: belongs in safety crate (ownership unclear; may be grey-zone-adjacent). Anti-slopsquatting policy enforcement.

  Task 4 (per-level critic at Step + BabyStep) — defer reasons: depends on C-31 FILE_PLAN (PLANNED, not shipped).

  Task 5 (EscalationTrigger::HumanIntervention + ANTI_SPIRAL_HARD_ABORT) — defer reasons: cited in WRONG FILE (DRIFT-3); reconciliation needed. autonomous_gate.rs:400 is the real location.

  Task 6 (IdeaFramerSecurityPanel.svelte + 2 Tauri commands) — defer reasons: graceful-empty over real producer pattern — producers (Tasks 1-5) must exist first.

  Task 7 ([research_presets.production_saas] config block) — defer reasons: config-file ownership unclear.

  Task 8 (EditFormat enum) — defer reasons: zero existing callers; co-stage with the Aider-style diff routing producer.

  Task 9 (LevelAuditVerdict.outcome — actually .verdict + mark_cascade fn) — defer reasons: DRIFT-2 reconciliation.

  Task 10 (6 SsbActionType variants) — defer reasons: 42+ touchpoint surgery + golden regen + SCHEMA bump. Co-stage with producers.

  Task 11 (workflow/archetypes/ population — 5 archetypes × 4 subdirs) — defer reasons: depends on Task 1-3 producers. Tier-A shipped the directory + README only.

  Task 12 (prior-art doc set) — defer reasons: writes go in workflow/archetypes/prior_art/; need cite verification (DRIFT-10).

(d) DESIGN RATIONALE

The four negative-space guards encode the architectural decisions E-11 will need when Tier-B lands:

  - One canonical CriticRound (no IdeaCriticVerdict fork) — keeps the critic loop wire shape stable.
  - No demo-grade scaffolds emit silently — the gap between "demo" and "production" is the load-bearing concern of the whole E-11; the contract forces explicit demo-class opt-in.
  - One canonical PackageVerifier — slopsquatting gate is single-path or bypassable.
  - No quick-audit bypass — AuditorMode::ProductionGrade's 15-item checklist must not have a "skip it for this PR" door.

(e) WHY workflow/archetypes/ HAS A README WITHOUT TEMPLATES

The directory must exist on disk so the Tier-B work has a target. Empty directories aren't tracked by git, so a README.md is the minimal artefact. This mirrors workflow/vision_rag/README.md from E-36.A — same pattern, same rationale. Tier-B will populate templates/, checklists/, prior_art/, decisions/.

(f) WHY 4-WAY CROSS-SKIP IS GETTING UNWIELDY

After E-11.A we have FOUR sibling contract files (e11 + e25 + e38 + e48), each needing the other three in its walker skip list. The next negative-space contract addition will require 5 edits (4 sibling updates + the new file's own list). The systemPatterns.md note from E-25.A flagged this; E-11.A confirms the trajectory. Tier-B should factor to a shared workspace_walker helper in a tests/common/mod.rs or a workspace-tests crate. Concrete proposal:

  - crates/contracts/tests/common/walker.rs — single canonical workspace_walker() returning iterator of .rs paths with the standard skip list AND auto-detection of "is this a sibling negative-space contract" via file-basename pattern match (`e[0-9]+_*_contracts.rs`).
  - Each contract file imports from `mod common;`.
  - Adding a 5th contract becomes 1 edit (new file only).

Defer the refactor to a Tier-B housekeeping pass; this run is already committed to shipping 4-of-4 cross-skip.

(g) CAMPAIGN OBSERVATION

12-of-12 plans drifted. Tier-A discipline has now shipped 11 commits (E-33, E-31.A, E-29.A, E-21.A, E-36.A, E-52.A, E-42.A, E-38.A, E-48.A, E-25.A, E-11.A pending). 2 plans deferred without Tier-A scope (E-44, E-16) due to scope-ownership conflicts that even Tier-A couldn't navigate cleanly.

The pattern is: agents identify drift + recommend Tier-A → main LLM ships Tier-A + commits + dispatches memory-writer → repeat. The cost per E-N is ~2 agent dispatches + 1 audit + 1 memory-writer + ~30 minutes of main LLM Edit time. Throughput is roughly 1 E-N per cycle.

END E-11-decisions.md (Tier-A close-out)

=============================================================================

E-11 Tier-B catalog+recommender slice — decisions (session 3/4, s-25120)

Session: s-25120 completion-drive session 3/4 (E-cluster batch)
Plan: docs/thefinalglance/E-plan/E-11_idea_to_saas_prior_art.md
Cross-check: workflow/sessions/s-25120/cross-check-E/E-11_idea_to_saas_prior_art-report.md (2026-05-22 OVERALL=MISSING)
Context: workflow/sessions/s-25120/completion-drive/contexts/E-11_idea_to_saas_prior_art-context.md
Exploration: workflow/sessions/s-25120/completion-drive/exploration/E-11_idea_to_saas_prior_art-exploration.md
Date: 2026-05-28
Status: SHIPPED (Tier-B catalog+recommender slice)

SUMMARY
Tier-B ships the catalog + recommender substrate on top of the E-11.A + E-50.A Tier-A guards. NEW orchestrator::architecture subtree (5 files) carries:
  * SaasArchetype 5-archetype taxonomy enum
  * ArchetypeCatalog per-archetype const data (auth scheme + multi-tenancy + framework profile stem + SecurityCheckId subset + scope_clarity profile)
  * recommend_archetype(ProjectClass, &str) -> ArchetypeRecommendation with RecommendationConfidence 3-arm enum (NO f32)
  * PRIOR_ART 15-entry registry (9 tools + 6 reports)
  * ELAI_SAAS_PRIOR_ART_ENABLE wire-dark gate
Plus SSB cascade: SaasPriorArtEvent + PriorArtEventKind 2-arm discriminator (21st Pattern-2 application). SCHEMA 168 -> 169; TOTAL_VARIANTS 441 -> 442.

GROUNDED DECISIONS

DECISION-B1: 5-archetype taxonomy locked to workflow/archetypes/README.md
Grounding: e50.A guard #1; workflow/archetypes/README.md.
Outcome: SaasArchetype { SaasDashboard, B2bMarketplace, InternalTooling, DeveloperApi, ConsumerSocial }. `Saas` casing (not SaaS) for Rust-idiomatic single uppercase. snake_case wire labels match ProjectClass::SaaSDashboard's existing "saas_dashboard" discriminant verbatim — cross-enum round-trips clean.

DECISION-B2: AuditorMode::ProductionGrade CONSUMED, not re-implemented
Grounding: cross-check report STALE; reality verified at auditor_mode.rs:90 + map_to_agent_role at :100 (E-12.B Tier-B shipped).
Outcome: Catalog feeds per-archetype SecurityCheckId subset into the EXISTING ProductionGrade variant via `ArchetypeCatalog::production_checklist_subset_for(arch)`. No parallel variant. e11_idea_to_saas guard #4 stays green.

DECISION-B3: EscalationTrigger NOT touched
Grounding: file grep confirmed EscalationTrigger lives at autonomous_gate.rs:400+ (NOT anti_spiral_extended.rs as plan path drift implied), with 6 variants {AntiSpiral, HighRisk, ReplanMajor, CircuitBreaker, SubstituteModelFailed, SplitTaskFailed}. HumanIntervention ABSENT.
Outcome: ANTI_SPIRAL_HARD_ABORT + HumanIntervention is a SIBLING slice, not this one. The catalog+recommender substrate has zero EscalationTrigger dependency.

DECISION-B4: IdeaCriticVerdict forbidden — sibling-slice extension via CriticFinding
Grounding: e11_idea_to_saas guard #1 forbids "struct IdeaCriticVerdict", "enum IdeaCriticVerdict", "struct CriticVerdictV2", "enum CriticRoundOutcome". File grep confirms IdeaCriticVerdict does not exist; existing surface is CriticFinding (idea_framing.rs:448) + CriticRound + CriticCategory.
Outcome: scope_clarity_score is SIBLING slice. Recommender returns ArchetypeRecommendation; the call site computes scope_clarity from clarifying-question count + confidence arm.

DECISION-B5: RecommendationConfidence enum, NO f32
Grounding: integration-explorer GAP-3 + E-19 Pattern-2 + ProbeKind discriminator precedent.
Outcome: 3-arm enum {High, Medium, Low}. snake_case wire labels SSB-payload-compatible. Min clarifying questions per arm: High=0, Medium=2, Low=3.

DECISION-B6: 5-keyword-class brief heuristic (GAP-4)
Grounding: integration-explorer GAP-4 + 2024-2026 survey.
Outcome: KEYWORD_BIAS_TABLE in recommender.rs maps 5 keyword classes (auth / billing / tenant / api / feed / internal) to archetypes. First-keyword-hit wins (deterministic tie-break). Ambiguous ProjectClass (WebApp / DesktopApp / GameMod / Other) calls heuristic before falling back to per-class defaults.

DECISION-B7: Per-archetype SecurityCheckId subset (GAP-2)
Grounding: integration-explorer GAP-2 + production_checklist::SecurityCheckId 15-variant enum (framework_profile/profile.rs:28).
Outcome:
  SaasDashboard       -> 13/15 (omit Lighthouse, omit CargoMachete)
  B2bMarketplace      -> 15/15 (strictest)
  InternalTooling     -> 11/15 (omit Lighthouse + Sqlite3Rls + OTel + Sentry)
  DeveloperApi        -> 13/15 (omit Lighthouse, keep CargoMachete)
  ConsumerSocial      -> 11/15 (keep Lighthouse, omit Sqlite3Rls + CargoAudit + CargoMachete + Clippy)
Each subset = const `&'static [SecurityCheckId]` slice.

DECISION-B8: 15-entry prior-art registry (9 tools + 6 reports)
Grounding: context Q4 + 2024-2026 survey citation set.
Outcome: PRIOR_ART carries exactly 15 entries. Tools: v0.dev, bolt.new, lovable, replit-agent-3, cursor-2, devin, codex-cli, aider, horizon. Reports: Tenzai CSA Dec 2025, Veracode 2025, Escape.tech / Baytech 2025, Help Net 2025 slopsquatting, Answer.AI Devin Jan 2025, Cognition Devin 2025. Each entry archetype-tagged.

DECISION-B9: No inline framework version pins (e12.A guard 4)
Grounding: e12_production_codegen_contracts.rs guard 4.
Outcome: ArchetypeEntry.default_framework_profile = TOML stem (nextjs15 / svelte5 / fastapi-modern). Version pins live in workflow/archetypes/<stem>.toml ONLY. Contract test enforces.

DECISION-B10: SSB Pattern-2 21st application
Grounding: E-19 + E-55 Pattern-2 precedent.
Outcome: 1 variant + PriorArtEventKind 2-arm discriminator (ArchetypeRecommended / PriorArtConsulted). SaasPriorArtEventPayload mirrors RewardLadderEventPayload shape. Wire-dark gate ELAI_SAAS_PRIOR_ART_ENABLE.

DECISION-B11: NO dashboard surface in this slice
Grounding: prompt explicit "NO dashboard"; context Q2 deferred dashboard pair.
Outcome: NO SaasArchetypeRecommenderPanel.svelte. NO get_archetype_recommendation Tauri command. Rust-only surface; CLI / dashboard sibling slice.

DECISION-B12: NO new dependencies
Outcome: serde + serde_json only (already pinned). The architecture/ subtree adds zero crates.

OPEN QUESTIONS / RUNPOD-PENDING

OQ-B1: Live wiring of catalog::production_checklist_subset_for into auditor_mode dispatch site is a downstream consumption point (sibling slice). 15 external CLIs needed by production_checklist runners are RUNPOD-PENDING.

OQ-B2: SaasArchetypeRecommenderPanel.svelte + Tauri command DEFERRED.

OQ-B3: workflow/archetypes/ TOML manifests cover 4 frameworks (fastapi-modern / hono / nextjs15 / svelte5); the 5 archetypes each reference one as default. 5th TOML may be added later.

WORKFLOW
- SCHEMA_VERSION + TOTAL_VARIANTS read FRESH from ssb.rs:1982 (=168) and action_type_lock.rs:133 (=441) BEFORE any edit.
- E-19 + E-55 Pattern-2 mirrors read for SSB cascade shape.
- 10-site SSB cascade applied (variant + as_sql_str + FromStr + SsbPayload + action_type() + 4 exhaustive arms + payload struct + SQL CHECK + ALL_VARIANTS append + exhaustive_match_guard arm + golden regen).
- action_type_lock.golden regenerated via PowerShell helper (python3 not on PATH); saas_prior_art_event lands at ordinal 0332 (between runtime_health_changed 0331 and safety_critical_blocked 0333). All subsequent ordinals shifted +1.
- Negative-space guards (e11_idea_to_saas_contracts + e50_saas_archetype_contracts) re-run — both GREEN after this slice's additions.
- Per-test cargo run: saas_archetype_taxonomy 7/7 + archetype_recommender 17/17 + prior_art_registry 9/9 + e11_b_saas_prior_art_catalog_contract 21/21 + action_type_lock 2/2 + ssb::tests::test_typed_payloads 1/1. Total 57/57 GREEN.

TESTS_ADDED
- crates/orchestrator/tests/saas_archetype_taxonomy.rs (7 tests)
- crates/orchestrator/tests/archetype_recommender.rs (17 tests)
- crates/orchestrator/tests/prior_art_registry.rs (9 tests)
- crates/contracts/tests/e11_b_saas_prior_art_catalog_contract.rs (21 tests)
Total: 54 new tests.

FILES_TOUCHED
NEW:
- crates/orchestrator/src/architecture/mod.rs
- crates/orchestrator/src/architecture/saas_archetype.rs
- crates/orchestrator/src/architecture/prior_art.rs
- crates/orchestrator/src/architecture/catalog.rs
- crates/orchestrator/src/architecture/recommender.rs
- crates/orchestrator/tests/saas_archetype_taxonomy.rs
- crates/orchestrator/tests/archetype_recommender.rs
- crates/orchestrator/tests/prior_art_registry.rs
- crates/contracts/tests/e11_b_saas_prior_art_catalog_contract.rs

MODIFIED:
- crates/orchestrator/src/lib.rs (pub mod architecture + header comment)
- crates/orchestrator/src/ssb.rs (10-site SSB cascade; SCHEMA 168 -> 169; test_typed_payloads count 379 -> 380)
- crates/orchestrator/tests/action_type_lock.rs (TOTAL 441 -> 442 + ALL_VARIANTS + exhaustive_match_guard arm)
- crates/orchestrator/tests/action_type_lock.golden (regenerated; +1 row saas_prior_art_event at 0332)

CITATIONS / GROUNDING
- workflow/improvement-sweep-2026-05-19/reports/11-idea-to-saas-prior-art.md (2024-2026 survey)
- Veracode 2025 GenAI Code Security (45% fail-rate)
- Tenzai CSA Dec 2025 (100% CSRF / headers missing across 15 vibe-coded apps)
- Escape.tech / Baytech 2025 (1,400 apps, 65% with security issues + 400+ secrets)
- Help Net Security 2025 slopsquatting (19.7%)
- Answer.AI Devin retrospective Jan 2025 (3/20 success)
- Cognition Devin 2025 review (ambiguity handling)
- arXiv:2509.16941 SWE-Bench Pro (RAG-indexed cursor-2 reference)
- arXiv:2510.10460 Planner-Coder Gap (recommender clarifying-questions motivation)

END E-11-decisions.md (Tier-B catalog+recommender close-out)
