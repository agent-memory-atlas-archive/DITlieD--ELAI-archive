E-50 — SaaS architecture pattern catalog — decisions
session: s-orch-e50 (campaign continuation — FINAL PLAN)
plan: F:\.ELAI_workflow\docs\thefinalglance\E-plan\E-50_saas_architecture_patterns.md

(a) SCOPE — TIER-A ONLY (14-of-14 campaign drift confirmed — FINAL)

E-50 returned PLAN_DRIFTED_REQUIRES_IMPROVEMENT from both dispatched agents. The final plan in the s-26556 queue locks the 14-of-14 campaign pattern.

DRIFT inventory:

  DRIFT-1 (HIGH): plan claims PipelineSpec at research_tree.rs:909 has fields `multi_tenant: bool`, `options.ssr/ssg/client_only_spa`, `shared_db/isolated_db_per_tenant`. Reality: PipelineSpec is a 5-field stub with name/steps/output_type/perf/media. Plan-claimed fields do not exist.

  DRIFT-2 (HIGH): plan's archetype mapping (saas_b2c / saas_b2b / developer_tool / ai_product) is FABRICATED. ProjectClass enum at idea_framing.rs:89 has 9 variants (ConsumerMobileApp / WebApp / SaaSDashboard / GameMod / ...). None of the plan's mapping strings exist. Either ProjectClass needs extension OR a translation classifier needs to be added.

  DRIFT-3 (HIGH): E-11.A workflow/archetypes/README.md declares 5 archetypes (SaaSDashboard / B2BMarketplace / InternalTooling / DeveloperAPI / ConsumerSocial); E-50 plan uses 4 different ones (B2C / B2B / ApiFirst / AiApp). Taxonomy reconciliation needed before Tier-B.

  DRIFT-4 (HIGH): plan adds 15+ new SsbActionType variants. 105+ atomic touchpoints + 288-golden alphabetical regen + SCHEMA 97 → 98 atomic bump. Wildly over soft cap.

  DRIFT-5 (HIGH): plan adds 5 net-new orchestrator modules (saas_archetype / rls_baby_step_emitter / production_readiness_gate / scoped_token_enforcer / per_tenant_rate_limit). All grey-zone orchestrator/{root}.

  DRIFT-6 (HIGH): plan elai.map line citations are stale: :10395 / :344283 / :1164 / :1172 all wrong. Real: configure_agents :1232, generate :1110. Lines 1164/1172 are MACRO records. Same drift class as E-44 / E-16 / E-31 / E-29 / E-21 / E-36 / E-52 / E-42 / E-38 / E-48 / E-25 / E-11 / E-12.

  DRIFT-7 (MEDIUM): plan templates/saas/ directory + 12 template files + 5 Svelte panels + 5 Tauri commands — all greenfield. Mostly Tier-B work.

  DRIFT-8 (MEDIUM): plan cites 25+ vendor URLs (Stripe / Clerk / Auth0 / Supabase / NextAuth / Lucia / Lemonsqueezy / Paddle / Postmark / Resend / OTel / Sentry / etc) without anti-slopsquatting verification.

  DRIFT-9 (MEDIUM): plan's workflow/compliance/ reference — directory does NOT exist (E-38.A deferred). Cross-plan dependency must be resolved before E-50 lands.

  DRIFT-10 (LOW): plan suggests adding a 7th AuditorMode variant — integration explorer flagged this as a DRIFT against the current 6-variant set. Reuse, don't extend.

  DRIFT-11 (LOW): existing wiring at crates/rules/src/contract_tests.rs:1603-1649 already enforces app.5/app.6 rules-as-data (RLS WHERE-only BLOCK + SET app.tenant_id ALLOW + DISCARD ALL on pooler reuse). E-50 builds on this; do not parallel-implement.

  DRIFT-12 (LOW): elai_cc.toml [research_presets.saas_dashboard] at line 635 already declares cohort/compliance/billing/feature_flag streams. Tier-B extends, not parallels.

  DRIFT-13 (LOW): templates/web/manifest.toml byte-level {{ident}} substitution is the canonical template engine — E-50 reuses it (avoids the template-engine-dep-pollution guard from E-12.A).

14-of-14 plans confirmed PLAN_DRIFTED_REQUIRES_IMPROVEMENT.

(b) WHAT TIER-A E-50 SHIPPED

  - crates/contracts/tests/e50_saas_archetype_contracts.rs (~115 LOC, 4 negative-space + walker sanity)
    Sixth sibling negative-space contract. Auto-skip via tests/common/mod.rs handled the sibling integration with ZERO edits to existing siblings (e11 + e12 + e25 + e38 + e48). The E-12.A refactor's value is now demonstrated.
    Guards:
      • no_parallel_saas_archetype_enum — bans SaasArchetypeV2 / SaaSArchetypeNext / ProductArchetype / BespokeArchetype. Canonical taxonomy is the one in workflow/archetypes/README.md.
      • no_silent_multi_tenancy_bypass — bans raw_db_query_no_tenant / bypass_rls_policy / skip_tenant_scope / untenanted_query / admin_skip_rls. Forces RLS gate.
      • no_destructive_ddl_helpers_in_codegen — bans emit_drop_database / emit_drop_table / emit_silent_table_clear / unscoped_delete_all / generate_destructive_migration. E-50 templates would be reused across tenants; destructive DDL emitters one silent merge away from disaster.
      • no_bespoke_billing_adapter_parallel_to_canonical — bans BespokeBillingAdapter / CustomBillingV2 / QuickPaymentAdapter / skip_billing_quota_check.
      • walker sanity.

  - workflow/reasoning/E-50-decisions.md (this file)

(c) TIER-B DEFERRED

  Task 1 (PipelineSpec extension with multi_tenant / ssr / db fields) — defer reasons: requires the plan rewrite per DRIFT-1; PipelineSpec is grey-zone orchestrator/{root}.

  Task 2 (ProjectClass extension OR SaasArchetypeClassifier overlay) — defer reasons: DRIFT-2 reconciliation. Decision needed at start of E-50.B.

  Task 3 (taxonomy reconciliation between E-11.A README + E-50 plan) — defer reasons: DRIFT-3. Workflow/archetypes/ taxonomy is canonical (E-11.A); E-50.B updates plan to match.

  Task 4 (5 net-new orchestrator modules) — defer reasons: grey-zone; pair with Task 1.

  Task 5 (templates/saas/ + 12 template files) — defer reasons: depends on Tasks 2 + 3.

  Task 6 (5 Svelte panels + 5 Tauri commands) — defer reasons: graceful-empty over real producer pattern.

  Task 7 (15 new SsbActionType variants) — defer reasons: 105+ atomic touchpoints + golden regen + SCHEMA bump. Co-stage with producers.

  Task 8 (25+ vendor anti-slopsquatting WebSearch verify) — defer reasons: workflow tooling pass; pair with Task 5.

  Task 9 (workflow/compliance/ resolution — E-38.A deferred) — defer reasons: cross-plan dependency.

  Task 10 (extension of crates/rules/src/contract_tests.rs app.5/app.6) — defer reasons: build on existing surface; reuse, don't fork.

(d) CAMPAIGN PATTERN — 14-OF-14 LOCKED, COMPLETE

The full s-26556 + s-orch-* campaign is now complete:

PLANS DRIFTED (14): E-44, E-16, E-31, E-29, E-21, E-36, E-52, E-42, E-38, E-48, E-25, E-11, E-12, E-50.

PLANS SHIPPED VIA TIER-A (13):
  - E-33 (full, not Tier-A — pre-protocol-rule first half + protocol-correct second half)
  - E-31.A — head_dim invariant contract
  - E-29.A — OTel observability (3 drawers over existing SSB producers)
  - E-21.A — pagerank cross-surface (panel + CLI)
  - E-36.A — vision artifact types
  - E-52.A — LSP diagnostic envelope types
  - E-42.A — grammar tier metadata
  - E-38.A — anti-pattern regression contracts (RuntimeAdapter / FA2 / agent-proto / screenshot)
  - E-48.A — web research contracts (search engines / blake2-sha2 / source tier / arxiv)
  - E-25.A — compaction architecture contracts (telescope / fork / summarizer / silent-drop)
  - E-11.A — idea-to-SaaS contracts (critic verdict / demo scaffold / package verifier / quick-audit)
  - E-12.A — fullstack codegen contracts + shared-walker refactor (370 LOC duplication removed)
  - E-50.A — SaaS archetype catalog contracts (archetype enum / multi-tenant / destructive DDL / billing)

PLANS DEFERRED WITHOUT TIER-A (2):
  - E-44 (SAGA durability — orchestrator-heavy, plan drift on SCHEMA 41-45 vs current 97, needs dedicated session)
  - E-16 (VLM image validation — Tier-2 stubs not actually shipped, MediaValidationDrawer absent, XGrammar has no supporting code)

COMMITS THIS SESSION (13): 00a1164 / 759fe1b / b604cde / be4d098 / 873b10b / 70ab25d / 6be878b / 8afa189 / d234aae / ad79220 / 7dffda5 / f7b2af8 / (E-50.A commit hash pending). + the in-band rule additions (.claude/rules/orchestrate-protocol.md + CLAUDE.md + SKILL.md updates) gitignored.

ARCHITECTURAL OUTPUTS:
  - 6 negative-space sibling contract test files at crates/contracts/tests/ (e11/e12/e25/e38/e48/e50) — 30/30 tests pass on current workspace.
  - 1 shared walker helper at crates/contracts/tests/common/mod.rs — auto-skips e<N>_*_contracts.rs siblings.
  - 4 types-first foundation modules in crates/knowledge/src/ (chunking::image_artifact, diagnostic, grammar_tier; plus the integration with existing modules) — 23 unit + contract tests across them.
  - 1 head_dim invariant contract (Qwen2 family pinned at 128).
  - 3 dashboard drawers (SpendDrawer, QueryDrawer, TranscriptDrawer) + PagerankPanel — all over existing Tauri command surfaces, zero new backend.
  - 5 new dashboard components from E-33 (ActivityRibbon, SkeletonList, ConfidenceTag, WindowFooterMode, SnackBarPermission) + motion store + Wong palette + reduced-motion 3-layer pattern.
  - 1 CLI subcommand (elai symbol_graph rank --top N).
  - 4 workflow scaffolding directories (workflow/archetypes/ + workflow/vision_rag/ — both with README; workflow/compliance/ still deferred to E-38.B).
  - 14 decisions docs (workflow/reasoning/E-*.md) capturing all DRIFT findings + Tier-B inventories.

PROTOCOL FIX:
  - .claude/rules/orchestrate-protocol.md HARD RULE H-1..H-10 (agent dispatch BLOCKING) added mid-session after the main LLM was caught skipping context-compiler + integration-explorer-elai on E-33's first half.
  - CLAUDE.md + .claude/skills/orchestrate/SKILL.md + .claude/rules/conditional-routing.md cross-linked to the new rule.

(e) WHAT NEXT SESSION INHERITS

  - All 14 E-plans from the Prompt 3 queue addressed (12 shipped Tier-A + 2 deferred).
  - Plan-improvement protocol per .claude/skills/planning-workflow/SKILL.md is MANDATORY before any Tier-B execution on the deferred items (carries E-N.B suffix in commit messages).
  - 6-contract shared-walker pattern is stable; adding a 7th contract = 1 edit.
  - Negative-space contract pattern is now the canonical Tier-A shape for any future drifted plan.

END E-50-decisions.md (FINAL plan in s-26556 campaign queue)

================================================================
ADDENDUM: E-50 completion-drive session 3/4 (s-25120) -- substrate ship
================================================================

Date: 2026-05-28. Session: s-25120 completion-drive 3/4.

Scope: ships the E-50 SaaS architecture pattern catalog substrate
on top of the e50.A negative-space contracts (already shipped). Lands
the type surface + deterministic decision functions + Pattern-2 SSB
plumbing per the 13 KEY DECISIONS below. Composes the existing
5-archetype taxonomy (E-11) with day-1 production-readiness gate +
scoped-token enforcement + MoR decision fork + realtime / async /
rate-limit / OTel-SLO scaffolding. No dashboard wiring (task instruction
"NO dashboard" override). Tier-B template/checklist population deferred
to a follow-up patch.

SCHEMA: 176 -> 177 (atomic +1).
TOTAL_VARIANTS: 449 -> 450.
LANDING: .elai_cc/crates/orchestrator/src/architecture/saas_patterns.rs
(sibling to E-11 catalog / recommender / prior_art / saas_archetype).
ENV gate: ELAI_SAAS_ARCH_PATTERNS_ENABLE=1 (default-off wire-dark).

KEY DECISIONS (substrate ship)

D1. Taxonomy: reuse locked 5-variant SaasArchetype, REJECT plan's
4-variant B2C/B2B/ApiFirst/AiApp fork. Per e50.A negative-space contract
and workflow/archetypes/README.md the 5-variant set {SaasDashboard,
B2bMarketplace, InternalTooling, DeveloperApi, ConsumerSocial} is the
single source of truth. Introducing a parallel enum would trip
no_parallel_saas_archetype_enum.

D2. Pattern-2 SSB extension via NEW SaasArchPatternEvent single variant
+ 11-arm SaasArchPatternEventKind discriminator. Cleaner than extending
E-11's PriorArtEventKind because E-11's payload schema (archetype +
confidence) does not generalize to the gate / scoped-token / MoR /
realtime / async fire-points. The 29th Pattern-2 application. Schema
177, TOTAL_VARIANTS 450. Plan's 15-new-variant explosion REJECTED.

D3. ProjectClass FROZEN at 9 variants -- did not invent
saas_b2c/saas_b2b/developer_tool/ai_product. E-50 reuses the existing
architecture::recommender::recommend_archetype function (shipped under
E-11). E-50 does NOT touch idea_framing.rs.

D4. MultiTenancyModel naming collision avoided. Plan proposed
MultiTenancyMode; existing enum is MultiTenancyModel. E-50 REUSES the
existing enum. Hybrid variant NOT added in this slice.

D5. SecurityCheckId NOT extended. Plan suggested adding 3 new variants;
E-50 instead introduces a SIBLING enum ProductionReadinessCheck
(6 variants) with composes_with() helper mapping 3 of the 6 onto
existing SecurityCheckIds. Reason: existing 15-variant set is contract-
locked across multiple crates; a 15->18 bump would cascade.

D6. templates/saas/ vs workflow/archetypes/ COLLISION resolved by NOT
creating templates in this slice. Future patch-50.1 can populate
workflow/archetypes/templates/ + checklists/ + decisions/ if needed.

D7. Dashboard wiring DROPPED per task instruction.

D8. Scoped-token enforcer surface: closed DeployPlatform 5-arm +
DeployVerb 4-arm + per-platform allowed-verb table. Destroy verb is
BANNED universally for agent-issued tokens (PocketOS Apr 2026 defense).

D9. MoR decision fork: 3-arm closed enums for TargetGeography +
MrrProjection + MorVendor. Decision matrix DETERMINISTIC per
(geography, mrr_projection). Rules: US-only -> Stripe + Anrok direct;
EU-only or Global default -> Polar; OverHundredK enterprise -> Paddle.

D10. RealtimeNeed default OFF + 12-entry REALTIME_TRIGGER_KEYWORDS
table. CRDT triggers take priority over Presence triggers. Liveblocks
for Crdt; PartyKit on Cloudflare Durable Objects for Presence.

D11. AsyncBackbone routing by 4-arm LatencyBudget -> 5-arm AsyncBackbone.
AI-inference bucket: DeveloperApi + DAG-keyword -> Hatchet; else -> Modal.

D12. Per-tier rate-limit derivation table. free/starter/hobby=10rpm;
pro/growth/standard=100rpm; business/team=1000rpm; enterprise/scale
=5000rpm; unknown=100rpm median fallback.

D13. SLO scaffold integer-bps storage avoids f32 round-trip drift.
Per-archetype overrides: B2bMarketplace p95=300ms; DeveloperApi
availability_bps=9995 + p95=250ms. Multi-burn-rate 14x/6x/1x per
OneUptime 2026.

ARTIFACTS SHIPPED THIS ADDENDUM:
  - .elai_cc/crates/orchestrator/src/architecture/saas_patterns.rs
    (~700 LOC including 17 unit tests for round-trip + decision
    invariants + PocketOS defense check)
  - .elai_cc/crates/orchestrator/src/architecture/mod.rs (+saas_patterns
    submodule export)
  - .elai_cc/crates/orchestrator/src/ssb.rs (SCHEMA 176->177, new
    SsbActionType::SaasArchPatternEvent, new SaasArchPatternEventKind
    11-arm discriminator, new SaasArchPatternEventPayload, new
    SsbPayload::SaasArchPatternEvent, sample-constructor in test_payload,
    all_test_payloads entry, payloads.len() 387->388, must-cover string
    extended)
  - .elai_cc/crates/orchestrator/tests/action_type_lock.rs
    (TOTAL_VARIANTS 449->450, ALL_VARIANTS entry,
    exhaustive_match_guard entry + CostTokenEvent arm restored)
  - .elai_cc/crates/orchestrator/tests/action_type_lock.golden
    (450 entries; saas_arch_pattern_event at ordinal 0339; subsequent
    entries shifted +1)
  - workflow/reasoning/E-50-decisions.md (this addendum)

TESTS PASSING (verified):
  - cargo check --workspace: GREEN (1m 03s)
  - cargo test -p elai-orchestrator --test action_type_lock: 2/2 PASS
    (1 ignored regen helper)
  - cargo test -p elai_cc_contracts --test e50_saas_archetype_contracts:
    5/5 PASS (negative-space guards still satisfied)

GROUNDING SOURCES (verified May 2026): hunchbite multi-tenant SaaS,
thenile.dev RLS, makerkit RLS best-practices, accuknox OWASP API 2026,
dev.to/alessandro_pignati PocketOS Apr 2026, securityboulevard 2026
secret sprawl, sherlockforensics AI-code 2026, oneuptime burn-rate-
alerts-opentelemetry 14x/6x/1x, fintechspecs Polar MoR 2026, anrok
Stripe US sales-tax, starterpick Inngest 2026, pkgpulse Hatchet 2026,
liveblocks Yjs, latestfromtechguy PartyKit 2026, upstash workers
rate-limiting.

DEFERRED (out-of-scope this addendum):
  - workflow/archetypes/templates/ + checklists/ + decisions/ Tier-B
    population.
  - Tree-sitter AST upgrade for admin-route-auth check (patch-50.1).
  - Live Designer / Auditor wiring of the new SaasArchPatternEvent
    emits (the substrate exposes the surfaces; wiring is per-callsite).
  - AWS scoped-token check (RUNPOD-PENDING).
  - 5 Svelte panels + 5 Tauri commands (task NO-DASHBOARD override).
  - MultiTenancyModel::Hybrid variant ("whales -> schema isolation").
  - contracts/tests/saas_arch_pattern_contracts.rs (extension of the
    existing 5-assert e50_saas_archetype_contracts.rs).

END E-50 substrate addendum

