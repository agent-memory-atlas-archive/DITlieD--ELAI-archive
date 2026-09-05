# E-50: SaaS PipelineSpec archetype catalog — 5 archetypes (SaasDashboard / B2bMarketplace / InternalTooling / DeveloperApi / ConsumerSocial) + RLS-mandatory + day-1 production-readiness gate + scoped-token enforcement + MoR/Stripe decision fork
Objective: Give ELAI's Designer a structured catalog of five opinionated SaaS PipelineSpec archetypes (SaasDashboard, B2bMarketplace, InternalTooling, DeveloperApi, ConsumerSocial) instead of freestyling a stack per idea. Each archetype seeds an `archetype_template.toml` covering the 13 stack layers (FE, host, backend, DB, multi-tenancy, auth, billing, email, async, realtime, storage, caching, observability, rate-limit). Add a mandatory RLS BabyStep on every multi-tenant data model, a hard day-1 production-readiness gate (auth + rate-limit + Sentry + secret-scan), scoped-token enforcement on every IaC step, and a Merchant-of-Record vs Stripe-direct decision fork.
Estimated effort: 3 sessions, ~12 h
Pre-requisites: E-11 SHIPPED (Idea-to-SaaS prior art catalog — provides the upstream knowledge layer this E-N consumes), E-12 SHIPPED (Production fullstack codegen — provides the codegen scaffolding the archetype templates feed), B-7 SHIPPED (Designer + PipelineSpec producer; `crates/orchestrator/src/pipeline_builder.rs::configure_agents(spec: &PipelineSpec)` per elai.map line 10395 + `crates/orchestrator/src/research_tree.rs::PipelineSpec` per line 289867), B-40 SHIPPED (Idea-Framer with project classification — produces the archetype-routing signal).
Parallel-safe with: E-14 (browser e2e — different surface), E-15 (sandboxed exec — coordinates on scoped-token rule), E-17 (RAG retrieval — different surface). EXTENDS E-11 (idea-to-SaaS prior art) and E-12 (production fullstack codegen) — E-50 is the catalog layer those two feed into.
Source report: workflow/improvement-sweep-2026-05-19/reports/50-saas-architecture-patterns.md

## Vocabulary dictionary (canonical 5-variant taxonomy, 2026-05-31 hygiene refresh)

This plan was authored with a legacy 4-bucket taxonomy (B2C / B2B / API-first / AI-app). The shipped canonical surface in `.elai_cc/crates/orchestrator/src/architecture/saas_archetype.rs` lines 33-67 is a 5-variant `SaasArchetype` enum:

- `SaasDashboard` (snake_case wire `saas_dashboard`) — single-tenant or multi-tenant B2B/B2C dashboard with first-party auth + tenant data; legacy `B2B` and most `B2C` map here when the product is a dashboard
- `B2bMarketplace` (snake_case `b2b_marketplace`) — two-sided marketplace (supply + demand) with payments + tenant isolation; legacy `B2B` maps here when the product is a two-sided marketplace, NOT a single-tenant dashboard
- `InternalTooling` (snake_case `internal_tooling`) — single-tenant internal-only tool, no public surface
- `DeveloperApi` (snake_case `developer_api`) — API-first SDK + OpenAPI product; legacy `API-first` maps here
- `ConsumerSocial` (snake_case `consumer_social`) — consumer feed / social-graph product; legacy `B2C` maps here when the product is a consumer social/feed app, not a dashboard

Legacy → canonical map (NOT 1:1; B2B forks on single-tenant vs two-sided; B2C forks on dashboard vs consumer-social; AI-app collapses into the matching shape):

- `B2C` → `SaasDashboard` (dashboard-shaped) OR `ConsumerSocial` (feed/social-shaped)
- `B2B` → `SaasDashboard` (single-tenant) OR `B2bMarketplace` (two-sided)
- `API-first` → `DeveloperApi`
- `AI-app` → whichever of the above 5 matches the product shape (typically `SaasDashboard` for B2B AI apps, `ConsumerSocial` for consumer AI apps, `DeveloperApi` for AI-API products); there is NO dedicated `AiApp` root in the canonical enum

`ALL_SAAS_ARCHETYPES` const at saas_archetype.rs:59-67 is the iteration surface used by tests + catalog traversal. The `as_snake_case` / `from_snake_case` impls on lines 73-96 are the wire-stable labels used in SSB payloads + per-archetype TOML manifest stems.

## Context

The May 2026 SaaS architecture survey establishes that May 2026 has a consensus opinionated default stack: Next.js or SvelteKit FE on Vercel/Cloudflare Pages, container backend on Railway/Fly for long jobs, Neon/Supabase Postgres with RLS as day-1 multi-tenancy primitive, BetterAuth/Clerk + WorkOS-style upsell for enterprise SSO, Stripe Billing + Anrok (US) or Polar/Lemon Squeezy (global MoR) for billing, Inngest for async jobs, Resend for transactional email, Sentry + OpenTelemetry exporters from commit #1. Multi-tenancy default = shared schema + Postgres RLS; promote to dedicated schema only on compliance, >100K rows/tenant noisy-neighbor, or migration divergence — hybrid is explicit 2026 consensus (Hunchbite, Synthax, Zenn 2025). Day-1 must-haves are auth on EVERY route incl /admin, rate limit middleware (Upstash Ratelimit), Sentry + OTel, tenant_id on every table+policy, secrets via env, webhook signature verification. The grounding here is hard: 92% of AI-coded apps have at least one critical CVE (Sherlock 2026); 28.65M secrets leaked to public GitHub in 2025 (+34% YoY, GitGuardian 2026); the PocketOS DB-wipe incident April 2026 lost DB+backups in 9 seconds via an unscoped Railway CLI token an AI agent found. OWASP API Top 10 2026 lists rate-limit + broken-auth + BOLA on /admin as top 3 failure modes. Designer must emit ONE PipelineSpec per archetype (SaasDashboard, B2bMarketplace, InternalTooling, DeveloperApi, ConsumerSocial) per .elai_cc/crates/orchestrator/src/architecture/saas_archetype.rs and let downstream phases swap a single block rather than freestyling each idea — this is the consensus framing in the Encore "three-box decision" article and Makerkit DB-choice guide.

ELAI's existing Designer produces `PipelineSpec` via `pipeline_builder::configure_agents` + research_tree dispatch. The PipelineSpec already carries `options` for ssr/ssg/client_only_spa and `shared_db/isolated_db_per_tenant` (per elai.map lines 1164, 1172). E-50 extends this with a typed archetype enum, a per-block override map, mandatory RLS BabySteps for multi-tenant data models, a day-1 readiness gate, scoped-token enforcement, MoR-vs-Stripe decision fork, and ten other archetype-specific blocks (realtime, observability, async backbone, etc).

## Existing state (B/C/D coverage)

- B-7 SHIPPED (Designer + PipelineSpec producer): `crates/orchestrator/src/research_tree.rs::PipelineSpec` (per elai.map line 289867); `crates/orchestrator/src/pipeline_builder.rs::configure_agents(spec: &PipelineSpec)` (per line 10395); existing options include `ssr/ssg/client_only_spa` and `shared_db/isolated_db_per_tenant` (per lines 1164, 1172). EXTENSION: E-50 adds `PipelineSpec::archetype: SaasArchetype` enum + `block_overrides: HashMap<StackLayer, OverrideVendor>` field; backward-compatible additive.
- B-40 SHIPPED (Idea-Framer): produces project-class classification with `idea_framing` types (per elai.map line 718). EXTENSION: E-50 maps Idea-Framer project_class onto SaasArchetype as the default archetype, with explicit override allowed.
- E-11 SHIPPED (Idea-to-SaaS prior art): provides the catalog of templates (Open SaaS, Makerkit, Supastarter, T3, Encore, Wasp, Payload). EXTENSION: E-50 consumes E-11's catalog as the archetype-template seed pool — `ConsumerSocial` archetype defaults to Open SaaS / Wasp (legacy B2C-consumer); `SaasDashboard` defaults to Makerkit-style Next + Supabase (legacy B2B-single-tenant + B2C-dashboard); `B2bMarketplace` defaults to Next + Supabase + Stripe Connect (legacy B2B two-sided); `DeveloperApi` defaults to Cloudflare Workers + Hono / Nextra+Mintlify docs (legacy API-first); legacy AI-app collapses into whichever of the 5 matches the product shape (typically `SaasDashboard` for B2B AI apps with Next + Modal + Vercel AI SDK overrides). See top-of-file dictionary block for the canonical 5-variant mapping rules.
- E-12 SHIPPED (Production fullstack codegen): provides the per-block codegen scaffolds. EXTENSION: E-50's archetype templates feed E-12's codegen — choosing an archetype + 1-3 block overrides drives E-12 to emit the corresponding scaffold.
- Existing PipelineSpec `multi_tenant: bool` flag (or its equivalent — verify via Grep `multi_tenant` in research_tree.rs / idea_framing.rs): E-50 augments to `multi_tenant: MultiTenancyMode` enum (`Disabled | SharedSchemaRls | SchemaPerTenant | DbPerTenant | Hybrid { whales: Vec<TenantId> }`). When SharedSchemaRls (default for SaasDashboard archetype, single-tenant B2B, and B2bMarketplace), auto-emit the RLS BabyStep per Task 2.
- B-22 SHIPPED (autonomous mode + StagnationDetector): the day-1 production-readiness gate (Task 3) consumes the same SSB infrastructure for tripwire emit.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\50-saas-architecture-patterns.md` (full)
- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\E-plan\E-11_idea_to_saas_prior_art.md` (E-50 consumes E-11 catalog; read for archetype-template alignment)
- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\E-plan\E-12_production_fullstack_codegen.md` (E-50 emits into E-12 codegen; read for output-pair alignment)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\research_tree.rs` lines 909-1180 (`PipelineSpec` struct; per elai.map line 289867)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\pipeline_builder.rs` lines 850-1000 (`configure_agents` + `generate(quality_spec, spec)`; per elai.map lines 10395, 344283)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\idea_framing.rs` lines 700-1180 (project_class classification + 4 research presets; per elai.map lines 718-720, 1164, 1172)
- `F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-PLAN-SUMMARY.md` lines 700-770 (Designer + AVAILABLE_ARTIFACTS auto-injection context)
- https://hunchbite.com/guides/multi-tenant-saas-architecture (WebFetch — RLS-default + hybrid promotion)
- https://www.thenile.dev/blog/multi-tenant-rls (WebFetch — Postgres RLS shipping guide; forgot-tenant_id is dominant leak)
- https://makerkit.dev/blog/tutorials/supabase-rls-best-practices (WebFetch — RLS in JWT custom claims)
- https://makerkit.dev/blog/tutorials/best-database-software-startups (WebFetch — Neon/Supabase/PlanetScale 2026)
- https://workos.com/blog/workos-vs-betterauth-vs-clerk (WebFetch — auth 2026 landscape)
- https://fintechspecs.com/blog/stripe-vs-paddle-vs-lemon-squeezy-vs-polar-merchant-of-record-b2b-saas/ (WebFetch — MoR decision; Polar 4% + OSS)
- https://www.anrok.com/resources/saas-sales-tax-automation-for-stripe-introducing-the-anrok-app-for-stripe (WebFetch — Stripe + Anrok answer)
- https://accuknox.com/blog/owasp-api-security-top-10-the-complete-testing-checklist-2026 (WebFetch — OWASP API 2026)
- https://dev.to/alessandro_pignati/the-9-second-disaster-how-an-ai-agent-wiped-a-production-database-p56 (WebFetch — PocketOS Apr 2026 incident)
- https://securityboulevard.com/2026/03/the-state-of-secrets-sprawl-2026-ai-service-leaks-surge-81-and-29m-secrets-hit-public-github/ (WebFetch — GitGuardian 2026)
- https://www.sherlockforensics.com/pages/ai-code-security-report-2026.html (WebFetch — 92% AI-code CVE rate)
- https://oneuptime.com/blog/post/2026-02-06-error-budget-burn-rate-alerts-opentelemetry/view (WebFetch — SLO burn-rate 14x/6x/1x)

## Tasks

1. SaaS archetype enum + PipelineSpec extension. New types in `.elai_cc/crates/orchestrator/src/architecture/saas_archetype.rs` (~280 LOC): `SaasArchetype::{SaasDashboard, B2bMarketplace, InternalTooling, DeveloperApi, ConsumerSocial}` (canonical 5-variant taxonomy; see top-of-file dictionary), `StackLayer::{FeFramework, FeHost, Backend, Db, MultiTenancy, Auth, Billing, Email, Async, Realtime, Storage, Caching, Observability, RateLimit}` (14 layers per the report's stack table), `OverrideVendor { layer: StackLayer, vendor: String, reason: String }`. Extend `PipelineSpec` with `archetype: SaasArchetype` (default `SaasDashboard`) + `block_overrides: Vec<OverrideVendor>` (default empty) — backward-compatible additive fields. Idea-Framer's project_class maps to default archetype (`saas_b2c_dashboard` → `SaasDashboard`, `saas_b2c_social` → `ConsumerSocial`, `saas_b2b_dashboard` → `SaasDashboard`, `saas_b2b_marketplace` → `B2bMarketplace`, `internal_tool` → `InternalTooling`, `developer_tool` → `DeveloperApi`, `ai_product` → archetype matching product shape); Designer can override. Emit `saas_archetype_selected` SSB row + `archetype_block_overridden` row per override.

2. Mandatory RLS BabyStep emitter on multi-tenant data models. New module `.elai_cc/crates/orchestrator/src/rls_baby_step_emitter.rs` (~200 LOC). When `PipelineSpec.multi_tenant == MultiTenancyMode::SharedSchemaRls` (default for B2B), Designer detects `tenants` / `organizations` / `workspaces` in the data model and auto-emits one BabyStepSpec per multi-tenant table with template name `rls-enforce-{table}`. The BabyStep authoring requirements: (a) enable `ROW LEVEL SECURITY` on the table, (b) write the org_id-vs-JWT policy using `auth.jwt() ->> 'org_id'` (per Makerkit best-practices), (c) write a contract test attempting cross-tenant access expecting empty result. Plus three deterministic checks injected into the BabyStep audit gate: (i) RLS does NOT apply to superuser → migrations use NON-superuser role; (ii) `auth.uid()` / JWT claim is set in connection (Supabase pgbouncer transaction pooling drops this); (iii) every new table without explicit `ENABLE ROW LEVEL SECURITY` + policy fails CI lint. Emit `rls_baby_step_emitted` SSB row + `rls_audit_check_passed` / `rls_audit_check_failed` rows. Reference: arXiv-ish — the Nile blog, Makerkit RLS guide; "forgot WHERE tenant_id" is dominant leak.

3. Day-1 production-readiness gate (auth + rate-limit + Sentry + secret-scan + RLS lint). New module `.elai_cc/crates/orchestrator/src/production_readiness_gate.rs` (~300 LOC). Runs as a hard auditor gate before any "ready to ship" verdict. Checks: (a) every `/api/admin/*` route AND every `/api/internal/*` route has an auth middleware import (grep + AST check via tree-sitter if available, regex fallback); (b) at least one `Upstash Ratelimit` or equivalent rate-limit middleware imported on the edge entry; (c) `package.json` (or equivalent — `Cargo.toml`, `pyproject.toml`) contains `@sentry/...` or OpenTelemetry SDK; (d) `TruffleHog` or `GitGuardian` CLI run over the repo returns zero findings (vendor pinned + SHA-locked); (e) every multi-tenant table has `ENABLE ROW LEVEL SECURITY` + at least one policy (RLS lint from Task 2); (f) all webhook handlers use vendor lib for signature verification (regex grep for `Stripe.webhooks.constructEvent` / `verifyHook` etc., NEVER hand-rolled HMAC). New `DeterministicCheck::ProductionReadiness` variant. SSB action_type `production_readiness_gate_blocked` per failing check. Pass-or-block in audit timeline.

4. Scoped-token enforcement for AI-agent IaC steps. New module `.elai_cc/crates/orchestrator/src/scoped_token_enforcer.rs` (~250 LOC). Every PipelineSpec deploy BabyStep that touches Railway / Fly / Vercel / AWS CLI MUST use scoped credentials: project-id pinned, env-pinned, no destroy/delete scopes. The enforcer parses environment-variable references in the deploy BabyStep diff and refuses with `unscoped_token_blocked` SSB row if `RAILWAY_TOKEN` / `VERCEL_TOKEN` / `FLY_API_TOKEN` etc are used without scope pinning (verifiable via the platform's API — e.g. Railway tokens have a `project_id` field on creation that the enforcer queries). Documented anti-pattern: PocketOS April 2026 9-second DB-wipe via unscoped Railway token an AI agent found. Refuses by default; opt-out requires `--allow-unscoped-token` flag with a confirmation prompt (auditor logs the opt-out).

5. PipelineSpec emits OTel + SLO + runbook scaffold. New scaffold templates in `.elai_cc/templates/saas/{archetype}/`: `instrumentation.ts` (OTel SDK init for Node-based stacks; equivalent for Python via `opentelemetry-instrumentation`), `slos.yml` (defaults 99.9% availability / 500ms p95 / 1% error rate; multi-burn-rate per OneUptime: crit @ 14x / warn @ 6x / notice @ 1x; runbook URL placeholder), `runbooks/` directory with stub Markdown (one per SLO). E-12 (production fullstack codegen) consumes these via the codegen scaffolding. Critic (B-40 critic loop) flags any generated SaaS missing this as low quality. Emit `otel_slo_runbook_scaffolded` SSB row.

6. Merchant-of-Record vs Stripe-direct decision fork. New field `PipelineSpec::billing.merchant_of_record: Option<MorVendor>` where `MorVendor::{Polar, Paddle, LemonSqueezy} | None (Stripe + Anrok direct)`. Designer infers from target geography + revenue projection (questions injected into FRAMING substate when B-40 Idea-Framer runs): "Primary geography? US-only / Global / EU-only?", "Estimated MRR Y1? <$10K / $10K-100K / >$100K?". US-only + <$100K → suggest Stripe + Anrok; Global + any → suggest Polar (4% + OSS + GitHub-native) or Paddle (enterprise). The decision is a one-question fork that locks the rest of the billing block. Per FintechSpecs 2026 + Anrok+Stripe announcement. Emit `billing_mor_decision_made` SSB row.

7. Realtime/collab block OFF by default. New field `PipelineSpec::realtime: RealtimeNeed::{None, Presence, Crdt}` (default `None`). Designer only sets `Presence` or `Crdt` when the idea text mentions co-editing, presence, comments, "multiplayer", "real-time collaboration". When set, defaults to Liveblocks (managed Yjs sync) for `Crdt` and PartyKit-on-Cloudflare-Durable-Objects for `Presence` per Liveblocks docs + latestfromtechguy.com PartyKit 2026 article. Saves tokens, latency, and recurring bill on the typical SaaS that doesn't need it. Emit `realtime_block_default_off` / `realtime_block_enabled` SSB row.

8. Long-running task block routing by latency budget. New field `PipelineSpec::async_backbone: AsyncBackbone` where AsyncBackbone routes by user-wait + max-job duration questions injected into FRAMING. <30s sync → serverless function (Vercel / Cloudflare Worker). 30s-15min async → Inngest (no Redis required). 15min-2h → Trigger.dev (long jobs + observability) or fly.io worker queue. AI inference → Modal (serverless GPU) or fly-replay with GPU machines or Hatchet (AI DAG, streaming step outputs, priority lanes per pkgpulse.com 2026). Emit `async_backbone_decided` SSB row with chosen vendor + latency budget rationale.

9. Per-tenant rate-limit defaults from billing tier. New module `.elai_cc/crates/orchestrator/src/per_tenant_rate_limit.rs` (~180 LOC). When PipelineSpec has tiered plans (`billing.tiers: Vec<BillingTier>` with rate-limit-relevant fields), auto-derive rate-limit budgets per tier (`free=10rpm, pro=100rpm, business=1000rpm` as default seeds) and wire to Upstash Ratelimit middleware with `org_id` as key. Auto-generates a BabyStepSpec `ratelimit-per-tier` that drops the middleware. Per Upstash Ratelimit overview docs; OWASP API#4 unrestricted resource consumption.

10. WCAG 2.2 AA + i18n scaffold even on single-locale starts. New field `PipelineSpec::a11y.lighthouse_min: u8 = 90`. Every generated SaaS includes axe-core in CI + Lighthouse CI a11y/perf/SEO gate at 90+. i18n folder structure exists from day 1 (even if only `en/` populated) via next-intl or Paraglide (verify which is current ELAI default; Paraglide TanStack 2026 article). Critic flags any PipelineSpec where `lighthouse_min` is lowered below 90. Per HumblDesign WCAG 2026 (95.9% of top 1M pages fail at least one WCAG 2 A/AA check); Paraglide TanStack guide.

11. Catalog the five canonical archetypes in `templates/saas/` as runnable PipelineSpec seeds. New files: `templates/saas/saas_dashboard.toml`, `templates/saas/b2b_marketplace.toml`, `templates/saas/internal_tooling.toml`, `templates/saas/developer_api.toml`, `templates/saas/consumer_social.toml` (snake_case stems matching `SaasArchetype::as_snake_case`). Each TOML is a complete `PipelineSpec` example with the 14 stack layers filled per the report's stack-tier table (e.g. SaasDashboard: Next.js + Vercel + Container on Railway + Neon Postgres + RLS + WorkOS AuthKit + Stripe+Anrok + Postmark + Inngest + Liveblocks + R2 + Upstash + Better Stack + Upstash Ratelimit). These are the seeds Designer mutates per idea — the catalog itself is read-only ground truth.

12. Dashboard panels. Five small Svelte panels: `ArchetypePicker.svelte` (~120 LOC, shows the 5 canonical archetypes — SaasDashboard / B2bMarketplace / InternalTooling / DeveloperApi / ConsumerSocial — plus default-stack preview), `RlsAuditPanel.svelte` (~140 LOC, status of every multi-tenant table's RLS state), `ProductionReadinessGate.svelte` (~140 LOC, the 6 day-1 checks pass/block status), `ScopedTokenAuditPanel.svelte` (~100 LOC, list of all deploy tokens + scope pinning status), `BillingDecisionPanel.svelte` (~100 LOC, MoR vs Stripe-direct decision + rationale). Five Tauri commands: `get_archetype_options`, `get_rls_audit_status`, `get_production_readiness`, `get_scoped_token_audit`, `get_billing_decision`. Mount under existing Designer drawer or new `SaasArchetypeDrawer.svelte`.

## WIRING_CONTRACT

produces_symbols:
  - orchestrator::architecture::saas_archetype::SaasArchetype::{SaasDashboard, B2bMarketplace, InternalTooling, DeveloperApi, ConsumerSocial}
  - orchestrator::architecture::saas_archetype::ALL_SAAS_ARCHETYPES (const iteration surface)
  - orchestrator::architecture::saas_archetype::StackLayer (14 layers per report stack-tier)
  - orchestrator::architecture::saas_archetype::OverrideVendor { layer, vendor, reason }
  - orchestrator::research_tree::PipelineSpec::archetype: SaasArchetype (extension)
  - orchestrator::research_tree::PipelineSpec::block_overrides: Vec<OverrideVendor> (extension)
  - orchestrator::research_tree::PipelineSpec::multi_tenant: MultiTenancyMode (extension; replaces bool)
  - orchestrator::research_tree::PipelineSpec::realtime: RealtimeNeed (extension)
  - orchestrator::research_tree::PipelineSpec::async_backbone: AsyncBackbone (extension)
  - orchestrator::research_tree::PipelineSpec::billing.merchant_of_record: Option<MorVendor> (extension)
  - orchestrator::research_tree::PipelineSpec::a11y.lighthouse_min: u8 (extension)
  - orchestrator::rls_baby_step_emitter::emit_for_table(table: &TableSpec) -> BabyStepSpec
  - orchestrator::production_readiness_gate::DeterministicCheck::ProductionReadiness (variant extension)
  - orchestrator::scoped_token_enforcer::ScopedTokenEnforcer::verify(diff: &Diff) -> EnforcementResult
  - orchestrator::per_tenant_rate_limit::derive_tier_limits(tiers: &[BillingTier]) -> Vec<RateLimit>
consumes_symbols:
  - crates/orchestrator/src/research_tree.rs::PipelineSpec (B-7 — extension surface; per elai.map line 289867)
  - crates/orchestrator/src/pipeline_builder.rs::configure_agents (B-7 — archetype-driven dispatch; per line 10395)
  - crates/orchestrator/src/idea_framing.rs (B-40 — project_class → archetype mapping; per lines 718-720)
  - E-11 catalog templates (consumed as archetype-template seed pool)
  - E-12 codegen scaffolds (E-50 archetype templates flow into E-12 codegen)
  - tree-sitter crate (verify presence; for auth-middleware AST check in Task 3)
  - blake3 (RLS-policy content hashing for audit-trail integrity)
reachable_from:
  - crates/orchestrator/src/idea_framing.rs (archetype default chosen here when project_class classified)
  - crates/orchestrator/src/pipeline_builder.rs::generate (production_readiness_gate runs here)
  - crates/orchestrator/src/auditor.rs (verify via Grep — gate consumed by AUDITING substate)
  - templates/saas/{saas_dashboard,b2b_marketplace,internal_tooling,developer_api,consumer_social}.toml (seed pool; archetype renderer reads these; snake_case stems match `SaasArchetype::as_snake_case` per saas_archetype.rs:73-82)
stub_ban:
  - no `unimplemented!()` in any new module
  - no `.unwrap()` outside `#[cfg(test)]`
  - production_readiness_gate MUST NOT pass on any of the 6 checks silently — every check produces an explicit pass/block SSB row with the check name in payload
  - scoped_token_enforcer's `--allow-unscoped-token` opt-out MUST require an interactive confirmation prompt OR an explicit `confirm_unscoped: true` config flag with timestamp logging
  - RLS BabyStep emit MUST NOT skip the contract test step (Task 2 (c)) — the cross-tenant access empty-result test is the load-bearing safety check
  - no DuckDB / SQLite-only RLS policy emit (Postgres RLS only; the report's grounded vendor list — Neon/Supabase/PlanetScale)
  - no `DROP DATABASE` / `DROP TABLE` instructions in any generated BabyStep deploy step (PocketOS-incident defense)
  - no MoR vendor decision without recording the question/answer trail in `saas_archetype.toml` decision log
  - no Sentry-only observability emit — OTel SDK + Sentry exporter MUST coexist per Task 5 + report anti-pattern (Sentry SDK lock-in)
  - no realtime block enabled-by-default — Task 7 requires explicit idea-text trigger
  - no rate-limit-skipping signup / password-reset / magic-link endpoints (OWASP API#4)
  - blake3 ONLY for RLS-policy content hashing; no blake2/sha2
  - existing `multi_tenant: bool` field migration: bool → MultiTenancyMode enum is a breaking change; provide a `From<bool>` impl that maps true→SharedSchemaRls, false→Disabled for back-compat
frontend_pair: 5 Svelte panels per Task 12 (ArchetypePicker.svelte, RlsAuditPanel.svelte, ProductionReadinessGate.svelte, ScopedTokenAuditPanel.svelte, BillingDecisionPanel.svelte)
observable_effect:
  trigger: synthetic idea "Build a multi-tenant B2B SaaS CRM" run through Designer with default Idea-Framer classification
  expect_ssb: ≥1 `saas_archetype_selected` SSB row with `SaasDashboard` (single-tenant B2B CRM maps to SaasDashboard per dictionary), ≥1 `rls_baby_step_emitted` row per multi-tenant table, ≥1 `production_readiness_gate_blocked` row on missing checks (test fixture), ≥1 `billing_mor_decision_made` row with US/EU question prompt
  expect_artifact: generated PipelineSpec contains `archetype: SaasDashboard`, `multi_tenant: SharedSchemaRls`, `realtime: None`, `async_backbone: Inngest`, `billing.merchant_of_record: None` (Stripe+Anrok) if US/<$100K, `a11y.lighthouse_min: 90`; `templates/saas/saas_dashboard.toml` seed values flow through; RLS BabyStep emitted with contract test

## DETERMINISTIC_CHECK

```bash
# 1. SaasArchetype enum with 5 canonical variants
rg "SaasArchetype::SaasDashboard|SaasArchetype::B2bMarketplace|SaasArchetype::InternalTooling|SaasArchetype::DeveloperApi|SaasArchetype::ConsumerSocial" .elai_cc/crates/orchestrator/src/architecture/saas_archetype.rs
# Expected: ≥5 hits

# 2. PipelineSpec extension fields present
rg "archetype.*SaasArchetype|block_overrides.*Vec<OverrideVendor>|multi_tenant.*MultiTenancyMode|realtime.*RealtimeNeed|async_backbone.*AsyncBackbone|merchant_of_record.*Option<MorVendor>" .elai_cc/crates/orchestrator/src/research_tree.rs
# Expected: ≥6 hits

# 3. Five archetype seed TOMLs exist (snake_case stems matching as_snake_case)
ls .elai_cc/templates/saas/saas_dashboard.toml .elai_cc/templates/saas/b2b_marketplace.toml .elai_cc/templates/saas/internal_tooling.toml .elai_cc/templates/saas/developer_api.toml .elai_cc/templates/saas/consumer_social.toml
# Expected: 5 files exist

# 4. RLS BabyStep emitter + 3 deterministic checks
rg "rls_baby_step_emitted|ENABLE ROW LEVEL SECURITY|auth\.jwt\(\) ->> 'org_id'" .elai_cc/crates/orchestrator/src/rls_baby_step_emitter.rs
# Expected: ≥3 hits

# 5. Production readiness gate covers 6 checks
rg "production_readiness_gate_blocked|admin_route_auth|rate_limit_present|sentry_or_otel|secret_scan|rls_lint|webhook_signature" .elai_cc/crates/orchestrator/src/production_readiness_gate.rs
# Expected: ≥6 hits

# 6. Scoped-token enforcer refuses unscoped + requires opt-out flag
rg "unscoped_token_blocked|allow-unscoped-token|confirm_unscoped" .elai_cc/crates/orchestrator/src/scoped_token_enforcer.rs
# Expected: ≥2 hits

# 7. OTel + SLO + runbook scaffold template exists per archetype
ls .elai_cc/templates/saas/saas_dashboard/slos.yml .elai_cc/templates/saas/b2b_marketplace/slos.yml .elai_cc/templates/saas/internal_tooling/slos.yml .elai_cc/templates/saas/developer_api/slos.yml .elai_cc/templates/saas/consumer_social/slos.yml
# Expected: 5 files exist (one per canonical archetype)
rg "14x|6x|1x" .elai_cc/templates/saas/b2b_marketplace/slos.yml
# Expected: ≥3 hits (multi-burn-rate per OneUptime)

# 8. MoR decision fork
rg "MorVendor::Polar|MorVendor::Paddle|MorVendor::LemonSqueezy|billing_mor_decision_made" .elai_cc/crates/orchestrator/src/saas_archetype.rs
# Expected: ≥3 hits

# 9. Realtime OFF by default
rg "RealtimeNeed::None.*default|realtime_block_default_off" .elai_cc/crates/orchestrator/src
# Expected: ≥1 hit

# 10. Per-tier rate-limit derivation
rg "free.*10rpm|pro.*100rpm|business.*1000rpm|derive_tier_limits" .elai_cc/crates/orchestrator/src/per_tenant_rate_limit.rs
# Expected: ≥2 hits

# 11. a11y default 90
rg "lighthouse_min.*90|a11y.*90" .elai_cc/crates/orchestrator/src/research_tree.rs
# Expected: ≥1 hit

# 12. blake3 for RLS-policy hashing
rg "blake3" .elai_cc/crates/orchestrator/src/rls_baby_step_emitter.rs
# Expected: ≥1 hit
rg "blake2|sha2" .elai_cc/crates/orchestrator/src/rls_baby_step_emitter.rs
# Expected: zero hits

# 13. No DROP DATABASE in any generated BabyStep template
rg -i "DROP DATABASE|DROP TABLE" .elai_cc/templates/saas/
# Expected: zero hits

# 14. Smoke: B2B-marketplace CRM idea fixture (canonical B2bMarketplace archetype)
cd .elai_cc && cargo test -p orchestrator b2b_marketplace_archetype_emits_rls_baby_step
# Expected: PASS
cargo test -p orchestrator production_readiness_gate_blocks_missing_admin_auth
# Expected: PASS
cargo test -p orchestrator scoped_token_enforcer_refuses_unscoped_railway
# Expected: PASS
```

## ACCEPTANCE

- [ ] SaasArchetype enum with 5 canonical variants (SaasDashboard / B2bMarketplace / InternalTooling / DeveloperApi / ConsumerSocial); mapped from Idea-Framer project_class per the top-of-file dictionary block
- [ ] PipelineSpec extended with archetype + block_overrides + MultiTenancyMode + RealtimeNeed + AsyncBackbone + merchant_of_record + lighthouse_min fields, backward-compatible
- [ ] Five archetype seed TOMLs in `templates/saas/`: saas_dashboard, b2b_marketplace, internal_tooling, developer_api, consumer_social
- [ ] RLS BabyStep emitted per multi-tenant table when SharedSchemaRls; 3 deterministic audit checks injected; cross-tenant contract test present
- [ ] Production-readiness gate covers 6 checks (admin auth, rate-limit, Sentry/OTel, secret-scan, RLS lint, webhook signature); each emits explicit pass/block SSB
- [ ] Scoped-token enforcer refuses unscoped Railway / Fly / Vercel tokens; opt-out via flag + confirmation prompt
- [ ] OTel + SLO + runbook scaffold per archetype with multi-burn-rate (14x/6x/1x) per OneUptime
- [ ] MoR vs Stripe-direct decision fork with geography + MRR-projection questions injected into FRAMING
- [ ] Realtime OFF by default; only enabled on co-editing / presence / multiplayer triggers
- [ ] Async backbone routed by latency budget (serverless / Inngest / Trigger.dev / Modal / Hatchet)
- [ ] Per-tier rate-limit derived from billing tiers when present
- [ ] a11y lighthouse_min default 90; i18n folder structure from day 1
- [ ] Five dashboard panels + five Tauri commands wired
- [ ] No regression on existing B-7 PipelineSpec consumers; existing `multi_tenant: bool` migrated via `From<bool>` impl
- [ ] All anti-patterns from report's anti-patterns list explicitly guarded in stub_ban + tests

## Tests to add

- `crates/orchestrator/tests/b2b_marketplace_archetype_emits_rls_baby_step.rs`: fixture B2bMarketplace idea + 3 tables; assert RLS BabyStep emitted per table
- `crates/orchestrator/tests/production_readiness_gate_blocks_missing_admin_auth.rs`: `/api/admin/users` route without auth import; assert BLOCK + SSB row
- `crates/orchestrator/tests/production_readiness_gate_blocks_missing_rate_limit.rs`: no Upstash Ratelimit import; assert BLOCK + SSB
- `crates/orchestrator/tests/scoped_token_enforcer_refuses_unscoped_railway.rs`: deploy step with raw `RAILWAY_TOKEN`; assert BLOCK
- `crates/orchestrator/tests/scoped_token_enforcer_accepts_scoped_railway.rs`: scoped token with project_id pinned; assert PASS
- `crates/orchestrator/tests/mor_decision_us_under_100k_picks_stripe_anrok.rs`: US + $50K projection → suggest Stripe+Anrok (`merchant_of_record: None`)
- `crates/orchestrator/tests/mor_decision_global_any_picks_polar.rs`: Global + any → suggest Polar
- `crates/orchestrator/tests/realtime_block_default_off.rs`: idea text "todo app" → `realtime: None`; idea text "multiplayer whiteboard" → `realtime: Crdt`
- `crates/orchestrator/tests/async_backbone_routes_by_latency.rs`: <30s → serverless; 30s-15min → Inngest; AI inference → Modal
- `crates/orchestrator/tests/per_tier_rate_limits_derived.rs`: 3 tiers free/pro/business → 3 RateLimit specs with seeded RPM
- `crates/orchestrator/tests/no_drop_database_in_generated_templates.rs`: grep all `templates/saas/` for forbidden strings; assert zero
- `crates/orchestrator/tests/sentry_alone_blocks_must_have_otel.rs`: PipelineSpec with Sentry-only observability → BLOCK
- `crates/contracts/tests/saas_archetype_contract.rs`, `rls_baby_step_contract.rs`, `production_readiness_gate_contract.rs`, `scoped_token_enforcer_contract.rs`, `mor_decision_contract.rs`: contract test per major surface

## Files touched

- CREATE: `crates/orchestrator/src/saas_archetype.rs` (~280 LOC)
- CREATE: `crates/orchestrator/src/rls_baby_step_emitter.rs` (~200 LOC)
- CREATE: `crates/orchestrator/src/production_readiness_gate.rs` (~300 LOC)
- CREATE: `crates/orchestrator/src/scoped_token_enforcer.rs` (~250 LOC)
- CREATE: `crates/orchestrator/src/per_tenant_rate_limit.rs` (~180 LOC)
- CREATE: `.elai_cc/templates/saas/saas_dashboard.toml` (PipelineSpec seed; ~100 LOC TOML)
- CREATE: `.elai_cc/templates/saas/b2b_marketplace.toml` (~100 LOC TOML)
- CREATE: `.elai_cc/templates/saas/internal_tooling.toml` (~100 LOC TOML)
- CREATE: `.elai_cc/templates/saas/developer_api.toml` (~100 LOC TOML)
- CREATE: `.elai_cc/templates/saas/consumer_social.toml` (~100 LOC TOML)
- CREATE: `.elai_cc/templates/saas/{saas_dashboard,b2b_marketplace,internal_tooling,developer_api,consumer_social}/slos.yml` (5 files, ~30 LOC each)
- CREATE: `.elai_cc/templates/saas/{saas_dashboard,b2b_marketplace,internal_tooling,developer_api,consumer_social}/instrumentation.ts` (5 files; sample OTel init)
- CREATE: `.elai_cc/templates/saas/{saas_dashboard,b2b_marketplace,internal_tooling,developer_api,consumer_social}/runbooks/README.md` (5 stubs)
- CREATE: `crates/dashboard-v2/src/lib/components/ArchetypePicker.svelte` (~120 LOC)
- CREATE: `crates/dashboard-v2/src/lib/components/RlsAuditPanel.svelte` (~140 LOC)
- CREATE: `crates/dashboard-v2/src/lib/components/ProductionReadinessGate.svelte` (~140 LOC)
- CREATE: `crates/dashboard-v2/src/lib/components/ScopedTokenAuditPanel.svelte` (~100 LOC)
- CREATE: `crates/dashboard-v2/src/lib/components/BillingDecisionPanel.svelte` (~100 LOC)
- MODIFY: `crates/orchestrator/src/research_tree.rs` (+~100 LOC for PipelineSpec extension fields + From<bool> back-compat; verify insertion at line 909 area per elai.map line 289867)
- MODIFY: `crates/orchestrator/src/pipeline_builder.rs` (+~50 LOC for archetype-driven configure_agents dispatch + production-readiness-gate invocation; verify line 850-1000 area)
- MODIFY: `crates/orchestrator/src/idea_framing.rs` (+~40 LOC for project_class → archetype mapping + MoR/realtime/async questions in FRAMING substate)
- MODIFY: `crates/dashboard-v2/src-tauri/src/commands.rs` (+~100 LOC for 5 new Tauri commands)
- CREATE: `workflow/reasoning/E-50-decisions.md`

## Citations / Grounding

- https://hunchbite.com/guides/multi-tenant-saas-architecture — RLS-default + hybrid promotion; grounds Task 1 + Task 2
- https://www.thenile.dev/blog/multi-tenant-rls — "forgot WHERE tenant_id" dominant leak; grounds Task 2 mandatory RLS BabyStep
- https://makerkit.dev/blog/tutorials/supabase-rls-best-practices — org_id in JWT custom claims; auth.jwt() ->> 'org_id'; grounds Task 2 policy template
- https://makerkit.dev/blog/tutorials/best-database-software-startups — Neon best new-project default 2026; grounds DB choice in archetype seeds
- https://workos.com/blog/workos-vs-betterauth-vs-clerk — Clerk indie / WorkOS B2B / BetterAuth OSS; grounds Task 11 auth choices
- https://www.scalekit.com/blog/clerk-alternatives-for-b2b-ai-apps-in-2026 — WorkOS AuthKit free to 1M MAU; grounds B2B archetype auth choice
- https://dodopayments.com/blogs/best-billing-platform-usage-based-pricing — Stripe acquired Metronome Jan 2026; grounds billing consolidation
- https://fintechspecs.com/blog/stripe-vs-paddle-vs-lemon-squeezy-vs-polar-merchant-of-record-b2b-saas/ — Polar 4% + OSS + GitHub-native; Lemon Squeezy acquired by Stripe Jul 2024; grounds Task 6 MoR fork
- https://www.anrok.com/resources/saas-sales-tax-automation-for-stripe-introducing-the-anrok-app-for-stripe — Stripe + Anrok US-only answer; grounds Task 6
- https://starterpick.com/guides/background-jobs-inngest-vs-bullmq-vs-trigger-2026 — Inngest indie default no-Redis; Trigger.dev long jobs; grounds Task 8
- https://www.pkgpulse.com/blog/hatchet-vs-trigger-dev-v3-vs-inngest-durable-workflows-2026 — Hatchet best for AI pipelines; grounds Task 8 AI-app routing
- https://liveblocks.io/docs/collaboration-features/multiplayer/sync-engine/liveblocks-yjs — managed Yjs sync; grounds Task 7 Crdt default
- https://latestfromtechguy.com/article/partykit-realtime-collaboration-2026 — PartyKit on Cloudflare DO; grounds Task 7 Presence
- https://encore.dev/articles/vercel-alternatives — "three-box decision" frontend/backend/state; grounds the archetype catalog framing
- https://accuknox.com/blog/owasp-api-security-top-10-the-complete-testing-checklist-2026 — rate-limit + broken-auth + BOLA on /admin top 3; grounds Task 3
- https://dev.to/alessandro_pignati/the-9-second-disaster-how-an-ai-agent-wiped-a-production-database-p56 — PocketOS April 2026; grounds Task 4
- https://securityboulevard.com/2026/03/the-state-of-secrets-sprawl-2026-ai-service-leaks-surge-81-and-29m-secrets-hit-public-github/ — 28.65M secrets leaked; AI-assisted repos leak 6.4% vs non-AI; grounds Task 3 secret-scan check
- https://www.sherlockforensics.com/pages/ai-code-security-report-2026.html — 92% AI-coded apps have critical CVE; grounds Task 3 mandatory gate
- https://oneuptime.com/blog/post/2026-02-06-error-budget-burn-rate-alerts-opentelemetry/view — crit @ 14x / warn @ 6x / notice @ 1x; grounds Task 5 SLO scaffold
- https://starterpick.com/guides/open-saas-review-2026 — Open SaaS (Wasp) MIT free; grounds B2C archetype default
- https://github.com/JorchRL/saas-template-tmp — T3 ~28.5k stars; grounds API-first archetype option
- https://upstash.com/blog/cloudflare-workers-rate-limiting — Upstash Ratelimit + Workers 5ms global; grounds Task 9
- https://nikuscs.com/blog/13-tanstackstart-i18n/ — Paraglide TanStack/SvelteKit/Astro 2026; grounds Task 10 i18n choice
- https://humbldesign.io/blog-posts/color-accessibility-guide-wcag — WCAG 2.2 + 95.9% top-1M-pages fail; grounds Task 10 a11y default
- E-11 (Idea-to-SaaS prior art) — provides catalog templates; explicitly consumed
- E-12 (Production fullstack codegen) — provides codegen scaffolds; explicitly emitted-into
- CLAUDE.md anti-stale-training — grounds blake3-only for RLS-policy content hashing

## Open questions / risks

- Open question 1 (from report): default B2C archetype to Wasp/Open SaaS or Next+Supabase Makerkit-style? Default proposed: B2C archetype seeds Open SaaS (free MIT, batteries included) but allows `block_overrides` to swap to Next+Supabase. Measurable: A/B same idea via each, compare time-to-first-paying-user.
- Open question 2 (from report): is Polar.sh mature enough to default for MoR? Default proposed: list Polar + Paddle + LemonSqueezy in `MorVendor` enum; current default suggestion for "Global + any MRR" → Polar (4% + OSS + GitHub-native) but mark `polar_maturity_risk: true` in decision rationale.
- Open question 3 (from report): OTel + Sentry-exporter vs Sentry-native — 10% slower instrumentation DX justifies migration freedom? Default proposed: ship OTel + Sentry-exporter as default; document the LoC + p95 overhead in `workflow/reasoning/E-50-decisions.md`.
- Open question 4 (from report): Inngest vs Trigger.dev default for async backbone — which produces fewer audit BLOCKs from AI-generated handlers? Default proposed: Inngest for indie/B2C (no Redis required); Trigger.dev for long-job B2B. Measurable: A/B 20 generated B-N plans across both.
- Open question 5 (from report): can ELAI auto-detect "promote tenant to schema isolation" from query plans + row counts? Out of scope for E-50; deferred to follow-up. E-50 ships `MultiTenancyMode::Hybrid { whales: Vec<TenantId> }` as the spec surface so the future detector has a target.
- The 14-layer StackLayer enum may need extending (CDN, A/B-testing, search, vector DB). Open question: ship 14 layers in v1 + extension protocol, or pre-bake all 18-20? Default proposed: 14 + `StackLayer::Custom(String)` escape hatch.
- Tree-sitter dependency for AST auth-middleware check (Task 3 (a)): verify presence via Grep before shipping; if absent, regex fallback shipped first + tree-sitter added as patch-50.1.
- Scoped-token enforcement (Task 4) on Vercel: Vercel tokens may not carry an explicit "scope" field the way Railway does. Verify per-platform via WebSearch; if a platform doesn't support scoped tokens, the enforcer emits `platform_unsupported_for_scope_check` SSB row + falls back to "block any token-related env var without an explicit `# scope-verified: <platform-issued-id>` comment".
- The 6 production-readiness checks (Task 3) are not exhaustive — Sherlock 2026 reports 92% AI-coded apps have critical CVE, suggesting more checks needed. Risk: E-50 ships an under-strict gate. Mitigation: gate is extensible via `[saas.production_readiness.checks]` config block; future patches add checks without re-architecting.
- MoR decision questions (Task 6) injected into FRAMING substate may not be answerable by Idea-Framer's auto-classification — they require user input. Risk: FRAMING blocks waiting for user. Mitigation: default to "US-only + <$100K → Stripe+Anrok" if user is silent for 60s; emit `mor_decision_defaulted_no_user_input` SSB row.
- Realtime-OFF-by-default (Task 7) trigger regex set ("multiplayer", "co-editing", "presence", "comments", "real-time collaboration"): risk of false negatives (an idea about "live updates" without the trigger words). Mitigation: regex set is configurable + Designer LLM-pass may up-vote realtime if it detects intent the regex missed; flag with `realtime_block_enabled_by_llm_detection` SSB row.
- `multi_tenant: bool` → `MultiTenancyMode` enum is a breaking change to existing PipelineSpec consumers. Mitigation: `From<bool>` impl + a session of grep-fix to migrate explicit `.multi_tenant = true` usages.

## Next

E-50 turns Designer from a freestyle stack-inventor into a four-archetype catalog picker, with mandatory RLS BabySteps + day-1 production-readiness gate + scoped-token enforcement + MoR decision fork. After E-50, every generated SaaS by ELAI ships with: auth on every admin route, rate-limit middleware on signup/password/magic-link, OTel + Sentry-exporter from commit #1, multi-tenant RLS on every multi-tenant table, scoped CLI tokens by default, lighthouse a11y/perf/SEO >= 90, i18n folder structure even single-locale, MoR-vs-Stripe-direct decision recorded. This closes the 92% AI-coded-app CVE rate finding the report surfaces, the PocketOS DB-wipe class of incidents, and the OWASP API Top 10 2026 top 3 failure modes. Downstream: E-12 (production fullstack codegen) consumes the archetype templates directly; E-11 (idea-to-SaaS prior art) feeds upstream knowledge; E-15 (sandboxed exec) coordinates with scoped-token enforcement; future patch-50.1 could add the tenant-promotion-automation per the report's Open Q5. The biggest remaining gap is the question-injection UX in FRAMING substate (Task 6) — captured in Open questions block above.
