# SaaS architecture patterns for ELAI Designer (May 2026)

## TL;DR
- Default Designer PipelineSpec for "ship a SaaS": Next.js (or SvelteKit) on Vercel/Cloudflare Pages for FE, container backend on Railway/Fly for long jobs, Neon/Supabase Postgres with RLS as the day-1 multi-tenancy primitive, BetterAuth/Clerk + WorkOS-style upsell for enterprise SSO, Stripe Billing + Anrok (US) or Polar/Lemon Squeezy (global MoR) for billing, Inngest for async jobs, Resend for transactional email, Sentry + OpenTelemetry exporters from commit #1.
- Multi-tenancy default = shared schema + Postgres RLS. Promote a tenant to dedicated schema only when (a) compliance requires, (b) noisy-neighbor at >100K rows/tenant, (c) per-tenant migrations diverge. Hybrid is the 2026 consensus pattern.
- Day-1 must-haves are: auth on EVERY route incl /admin, rate limit middleware (Upstash Ratelimit), Sentry+OTel, tenant_id on every table+policy, secrets via env (never in repo), webhook signature verification. ELAI's "production-first" gate must enforce these or the SaaS is shipped broken (92% of AI-coded apps have at least one critical CVE as of 2026).
- Fork-and-ship template that already maps to ELAI's pipeline best: Wasp + Open SaaS (free, MIT, includes Stripe/email/admin/PostHog), Makerkit (paid), Supastarter (paid, multi-tenant first-class). For type-safe Rust-y indie path: Encore.dev TS+Go.
- Designer should emit ONE PipelineSpec per archetype (B2C, B2B, API-first, AI-app) and let downstream phases swap a single block rather than freestyling a new stack each idea.

## Key sources
URL | what | takeaway
- https://hunchbite.com/guides/multi-tenant-saas-architecture | RLS vs schema-per-tenant guide | hybrid (RLS-default, promote to schema on compliance/scale) is the 2025+ recommendation
- https://www.thenile.dev/blog/multi-tenant-rls | Postgres RLS shipping guide | forgetting WHERE tenant_id = ? is the dominant tenant-leak bug; RLS defends against application bugs
- https://makerkit.dev/blog/tutorials/supabase-rls-best-practices | Supabase RLS prod patterns | put org_id in JWT custom claims, match in policy; do not rely on app-layer filter
- https://makerkit.dev/blog/tutorials/best-database-software-startups | Neon/Supabase/PlanetScale 2026 | Neon = best new-project default (autoscale, scale-to-zero, branching, HTTP driver for edge)
- https://workos.com/blog/workos-vs-betterauth-vs-clerk | auth platforms 2026 | Clerk = best DX/indie SaaS; WorkOS = enterprise upsell (SSO/SCIM @ $125/connection); BetterAuth = OSS in-app, data in your DB
- https://www.scalekit.com/blog/clerk-alternatives-for-b2b-ai-apps-in-2026 | B2B AI auth 2026 | Clerk's per-MAU + 100 org cap punishes B2B; WorkOS AuthKit free to 1M MAU
- https://dodopayments.com/blogs/best-billing-platform-usage-based-pricing | Stripe/Orb/Lago/Metronome | Stripe acquired Metronome (Jan 2026); usage-billing now consolidating around Stripe; Lago = OSS escape hatch
- https://getlago.com/blog/top-7-alternatives-to-stripe-billing-for-usage-based-billing | Lago/alternatives | OSS billing, agnostic payment processor, integrates Anrok for tax
- https://fintechspecs.com/blog/stripe-vs-paddle-vs-lemon-squeezy-vs-polar-merchant-of-record-b2b-saas/ | MoR decision 2026 | Polar = 4% fee + OSS + GitHub-native; Lemon Squeezy acquired by Stripe Jul 2024; Paddle = enterprise
- https://www.anrok.com/resources/saas-sales-tax-automation-for-stripe-introducing-the-anrok-app-for-stripe | Anrok+Stripe | end-to-end US sales tax automation for Stripe-direct SaaS (the answer when you stay on Stripe vs MoR)
- https://starterpick.com/guides/background-jobs-inngest-vs-bullmq-vs-trigger-2026 | job queue 2026 | Inngest = indie SaaS default (no Redis); Trigger.dev = long jobs + observability; Hatchet = AI DAGs; BullMQ = >100K jobs/day
- https://www.pkgpulse.com/blog/hatchet-vs-trigger-dev-v3-vs-inngest-durable-workflows-2026 | durable workflows | Hatchet best for AI pipelines (DAG, streaming step outputs, priority lanes)
- https://liveblocks.io/docs/collaboration-features/multiplayer/sync-engine/liveblocks-yjs | Liveblocks+Yjs | managed Yjs sync, presence, comments — best collab infra without running WS servers
- https://latestfromtechguy.com/article/partykit-realtime-collaboration-2026 | PartyKit on Cloudflare DO | per-room Durable Object, edge-distributed, pairs with Yjs/Automerge/Replicache
- https://encore.dev/articles/vercel-alternatives | hosting 2026 | "three-box decision": frontend = edge/CDN, backend = container PaaS, state = managed DB
- https://blog.railway.com/p/server-rendering-benchmarks-railway-vs-cloudflare-vs-vercel | hosting benchmarks | Vercel+Railway dual-deploy is the dominant pattern for serious SaaS
- https://betterstack.com/community/comparisons/axiom-alternatives/ | observability 2026 | OTel is no longer optional; Better Stack bundles logs/traces/uptime/on-call
- https://dev.to/axiom_agent/the-nodejs-observability-stack-in-2026-opentelemetry-prometheus-and-distributed-tracing-229b | Node OTel 2026 | OTel is the baseline; Sentry SDK lock-in is the cost of staying proprietary
- https://www.buildmvpfast.com/api-costs/email | email pricing | Resend = best DX (react-email), Postmark = best deliverability (<1s, 99%+ inbox); SendGrid free tier killed May 2025
- https://www.tigrisdata.com/blog/benchmark-small-objects/ | storage benchmark | Tigris = 4x S3 / 20x R2 throughput, zero egress; R2 still cheapest at $0.015/GB
- https://upstash.com/blog/cloudflare-workers-rate-limiting | edge rate-limit | Upstash Ratelimit + Workers = 5ms global; Upstash 1.0 vs Redis 7.2 = 62% lower latency for rate-limit
- https://accuknox.com/blog/owasp-api-security-top-10-the-complete-testing-checklist-2026 | OWASP API Top 10 2026 | rate-limit + broken-auth + BOLA on /admin remain top 3 failure modes
- https://dev.to/alessandro_pignati/the-9-second-disaster-how-an-ai-agent-wiped-a-production-database-p56 | AI-agent prod incident Apr 2026 | AI agent found Railway CLI token, wiped DB+backups in 9s — agents must not have god-mode prod creds
- https://securityboulevard.com/2026/03/the-state-of-secrets-sprawl-2026-ai-service-leaks-surge-81-and-29m-secrets-hit-public-github/ | GitGuardian 2026 | 28.65M secrets leaked to public GitHub in 2025 (+34% YoY); AI-assisted repos leak 6.4% rate vs non-AI
- https://www.sherlockforensics.com/pages/ai-code-security-report-2026.html | AI code CVE rate 2026 | 92% of AI-generated codebases have a critical vuln; avg vibe-coded app = 8.3 exploitable findings
- https://oneuptime.com/blog/post/2026-02-06-error-budget-burn-rate-alerts-opentelemetry/view | SLO burn-rate | crit @ 14x burn (2h to exhaust), warn @ 6x, notice @ 1x; runbook URL in every page
- https://starterpick.com/guides/open-saas-review-2026 | Open SaaS (Wasp) | most-complete free SaaS boilerplate; Stripe + email + admin + PostHog + OpenAI; MIT
- https://github.com/JorchRL/saas-template-tmp | t3 SaaS template | T3 (~28.5k stars) remains the type-safe TS default
- https://fly.io/docs/blueprints/work-queues/ | fly.io long jobs | offload >5min jobs to work queue; fly-replay header for graceful machine upgrade with in-flight inferences
- https://modal.com/docs/guide/high-performance-llm-inference | Modal LLM inference | serverless inference = decoupled serve-queue-process-store pattern, Redis as glue
- https://nikuscs.com/blog/13-tanstackstart-i18n/ | Paraglide i18n | Paraglide officially supports TanStack/SvelteKit/Astro 2026; first-class no-prefix default locale
- https://humbldesign.io/blog-posts/color-accessibility-guide-wcag | WCAG 2.2 / APCA 2026 | ship WCAG AA from day 1; 95.9% of top 1M home pages fail at least one WCAG 2 A/AA check

## Stack-tier table per SaaS archetype

| Layer | B2C SaaS | B2B SaaS | API-first | AI-app |
|---|---|---|---|---|
| FE framework | Next.js / SvelteKit | Next.js (App Router) | Docs+playground (Nextra/Mintlify) | Next.js + AI SDK / SvelteKit |
| FE host | Vercel | Vercel | Cloudflare Pages | Vercel + Modal for inference |
| Backend | Vercel functions | Container on Railway/Fly | Cloudflare Workers + Hono | Modal/Replicate + container API |
| DB | Neon Postgres | Neon/Supabase + RLS | PlanetScale or Neon | Neon + pgvector / Pinecone |
| Multi-tenancy | n/a (single tenant per user) | RLS, schema for enterprise | API key + tenant_id RLS | RLS, per-tenant rate-limit |
| Auth | Clerk (cheapest @ <10k MAU) | WorkOS AuthKit (free to 1M, SSO upsell) | API keys + Unkey/JWT | Clerk + WorkOS for enterprise AI |
| Billing | Stripe Billing direct | Stripe + Anrok (US tax) or Paddle (global) | Orb (SQL-defined usage metrics) | Orb/Metronome for token-based usage |
| Email | Resend (react-email) | Postmark (deliverability) | Resend | Resend |
| Async | Inngest | Inngest or Trigger.dev | BullMQ (high volume) | Hatchet (DAG, AI pipelines) |
| Realtime | none unless feature | Liveblocks (presence/comments) | n/a | Vercel AI SDK streaming + Liveblocks for collab AI |
| Storage | UploadThing / R2 | R2 with signed URLs | Tigris (zero-egress + versioning) | Tigris (model weights) + R2 |
| Caching | Upstash Redis | Upstash + Redis | Cloudflare KV at edge | Upstash + Vercel Data Cache |
| Observability | Sentry + Axiom | Better Stack (logs+uptime+on-call) | Sentry + Axiom + Tinybird (request analytics) | Sentry + Helicone (LLM tracing) + OTel |
| Rate-limit | Upstash Ratelimit | Upstash + per-org plan tiers | Unkey or Upstash | Per-token-bucket + per-org plan |

Sources cited above (Encore.dev hosting article, Makerkit DB article, WorkOS comparison, Dodo billing article, BuildMVPFast email, Starterpick queue article, Liveblocks docs, Tigris benchmark, Upstash blog, BetterStack observability, Modal docs).

## Multi-tenancy

Default: **shared schema + Postgres RLS**. Every multi-tenant table carries `tenant_id` / `org_id`. The active tenant lives in a JWT custom claim set at sign-in. Every table has `ENABLE ROW LEVEL SECURITY` plus a USING+WITH CHECK policy comparing the column to `auth.jwt() ->> 'org_id'`. This is the 2026 consensus on Supabase/Neon (Makerkit RLS guide; Nile RLS blog).

Promote a tenant out of the pool when ANY of:
- compliance requires data residency / dedicated encryption keys (HIPAA, FedRAMP, EU residency).
- single tenant is >5-10% of total table volume causing query plan regressions.
- tenant needs migrations on a different cadence (custom fields, custom schema).
- noisy-neighbor: one tenant's queries blow the buffer cache for others.

Promotion path: **shared schema (RLS) → schema-per-tenant (same DB) → DB-per-tenant (same cluster) → dedicated cluster**. Hybrid is explicitly the 2026 best practice ("start RLS, isolate the whales", Hunchbite, Synthax, Zenn 2025 articles).

Anti-pattern: starting with schema-per-tenant or DB-per-tenant "for safety". You pay 10-100x ops cost from day 1 to defend against a leak you can prevent with one RLS policy (Nile blog).

Three RLS gotchas Designer must inject as deterministic checks:
1. RLS does NOT apply to the table owner / superuser. Migrations + app role must use a NON-superuser role.
2. `auth.uid()` / JWT claim must be set in the connection (Supabase pgbouncer transaction pooling can drop this).
3. Every new table is RLS-DISABLED by default in vanilla Postgres. Migration linter must FAIL CI on any table without an explicit `ENABLE ROW LEVEL SECURITY` + at least one policy (Makerkit RLS prod patterns).

## Day-1 must-haves

| Pattern | Tool default | Citation |
|---|---|---|
| Auth on every route, incl /admin and /api/internal | Clerk (indie) / BetterAuth (own data) / WorkOS AuthKit (B2B) | WorkOS vs BetterAuth vs Clerk; OWASP API#3 BOLA |
| RLS on every table | Postgres RLS via Supabase/Neon | Makerkit RLS, Nile RLS |
| Rate limit (per-IP + per-user + per-org) | Upstash Ratelimit on edge middleware | Upstash 1.0 benchmarks |
| Error tracking | Sentry + OTel SDK export to second sink (Axiom/Better Stack) | OTel 2026 baseline; Sentry SDK lock-in warning |
| SLO + burn-rate alerts | OTel metrics → multi-burn-rate (14x/6x/1x) + runbook URL on every page | OneUptime SLO guide |
| Webhook signature verification | Stripe / Resend / Inngest libs (never roll your own HMAC) | OWASP API top 10 2026 |
| Secrets via env, never repo | Doppler/Infisical or platform secrets; CI secret scanner (TruffleHog / GitGuardian) | GitGuardian 2026 report (28.65M leaks) |
| Billing wired pre-launch | Stripe Billing + Anrok (US) OR Polar/Paddle MoR | FintechSpecs MoR 2026 |
| Transactional email | Resend (react-email) or Postmark | BuildMVPFast pricing |
| Async queue from day 1 | Inngest | Starterpick jobs 2026 |
| WCAG 2.2 AA + Lighthouse CI a11y>=90 | axe-core in CI + Lighthouse CI | HumblDesign WCAG 2026 |
| i18n scaffold | next-intl or Paraglide (even single-locale start, structure ready) | Paraglide TanStack guide |

## Best opinionated templates 2026 (fork-and-ship)

| Name | Included | Missing | Citation |
|---|---|---|---|
| **Open SaaS (Wasp)** | Auth (W/ social), Stripe, Resend, admin dashboard, PostHog, OpenAI, blog. Free + MIT | Multi-tenant (single-tenant by default); needs Wasp ecosystem buy-in (smaller than Next) | Starterpick Open SaaS review 2026 |
| **T3 + create-t3-app** | tRPC, Drizzle/Prisma, Auth.js, Tailwind, type-safety end-to-end. ~28.5k stars, MIT | No billing, no email, no admin, no multi-tenancy out of box | Starterpick T3 2026; AdminLTE T3 templates |
| **Makerkit** | Multi-tenant orgs, Stripe, RLS, teams, billing portal, i18n, dark mode. Next.js/Remix/SvelteKit variants | Paid ($499+ lifetime) | Makerkit blog (rls best practices, DB choice 2026) |
| **Supastarter** | Supabase + multi-tenant + Stripe + emails + admin + i18n + OAuth | Paid ($349); Supabase-locked | DesignRevision SaaS hosting compared |
| **Encore.dev (TS or Go)** | Type-safe backend, infra-as-code-from-code, deploys to your AWS/GCP, free dev | FE not opinionated; smaller community vs Vercel | Encore Vercel/Fly alternatives articles |
| **Wasp** | DSL-driven full-stack; auth+jobs+email batteries included | DSL learning curve, smaller ecosystem | Open SaaS review |
| **Payload CMS** | Headless CMS + auth + admin UI + custom fields | Not a SaaS template per se; pair with Next FE | (general 2026 ecosystem) |

For ELAI's Designer: pre-bake the **Open SaaS (Wasp)** and **Makerkit-style Next.js + Supabase** scaffolds as two of the "B2C SaaS" / "B2B SaaS" PipelineSpec templates. Indie users get Open SaaS by default; B2B-flagged ideas get the Supabase+multi-tenant scaffold.

## Best additions for ELAI Designer

Ranked by ELAI-fit impact.

1. **PipelineSpec archetype templates (B2C / B2B / API-first / AI-app)** — instead of letting Designer freestyle the stack, store 4 canonical PipelineSpecs (the table above is the seed). Designer's job becomes "pick archetype + override 1-3 blocks", not "invent a stack". *Grounding:* the Encore "three-box decision" article and Makerkit DB-choice article both argue this is now the consensus shape. *Integration hook:* `PipelineSpec::archetype: SaasArchetype` enum + per-block override map. FlashResearch consumes archetype to choose which papers to fetch (e.g. AI-app archetype triggers LLM-inference-cost-optimization papers).

2. **Mandatory RLS-policy BabyStepSpec on every multi-tenant data model** — when Designer detects `tenants` or `organizations` in the data model, auto-emit a BabyStep that (a) enables RLS, (b) writes the org_id-vs-JWT policy, (c) writes a contract test attempting cross-tenant access and expecting empty result. *Grounding:* Nile + Makerkit RLS posts identify "forgot WHERE tenant_id" as the dominant tenant-leak. *Integration hook:* BabyStepSpec template named `rls-enforce-{table}` generated for every table where `multi_tenant: true`.

3. **Day-1 deterministic check pack: auth + rate-limit + Sentry + secret-scan** — a fixed BabyStep that auditor runs as a hard gate before "ready to ship". Greps every `/api/admin/*` route for an auth middleware import; checks for an Upstash Ratelimit import; checks `package.json` for `@sentry/...`; runs TruffleHog/GitGuardian over the repo. *Grounding:* 92% AI-code CVE rate (Sherlock 2026), 28.65M leaked secrets (GitGuardian 2026), PocketOS DB-wipe incident. *Integration hook:* `DeterministicCheck::ProductionReadiness` variant + pass-fail in the audit timeline.

4. **PipelineSpec emits OTel + SLO + runbook scaffold** — every generated SaaS includes `instrumentation.ts` (OTel SDK init), an `slos.yml` with default 99.9% availability / 500ms p95, and a `runbooks/` dir with stub Markdown linked from alert payload templates. *Grounding:* OneUptime SLO 2026 + Axiom OTel 2026 ("OTel is no longer optional"). *Integration hook:* `PipelineSpec::observability` block with `otel: true, slo_defaults: ...`; Critic (B-40) flags any spec missing this as low quality.

5. **AI-agent permission scoping baked into IaC** — every PipelineSpec that uses Railway/Fly/Vercel CLI generates *scoped* tokens (project-id pinned, env-pinned, no destroy/delete scopes). The 9-second-DB-wipe incident at PocketOS is the canonical anti-pattern. *Grounding:* AI agent DB-wipe Apr 2026 incident. *Integration hook:* the deploy BabyStep refuses to use a god-mode token; auditor fails if `RAILWAY_TOKEN` is unscoped.

6. **MoR-vs-Stripe-direct decision step in Designer flow** — Designer asks (or infers from target geography + revenue projection): MoR (Polar/Paddle/Lemon Squeezy) or Stripe + Anrok? This is a 1-question fork that locks the rest of the billing block. *Grounding:* FintechSpecs MoR 2026 + Anrok+Stripe announcement. *Integration hook:* new field `PipelineSpec::billing.merchant_of_record: Option<MorVendor>`.

7. **Realtime/collab block is OFF by default** — most SaaS does not need it. Designer only enables Liveblocks/PartyKit when the idea mentions co-editing, presence, comments, or "multiplayer". Saves tokens, latency, and a recurring bill. *Grounding:* Liveblocks docs; PartyKit Cloudflare DO architecture. *Integration hook:* `PipelineSpec::realtime: RealtimeNeed::None | Presence | CRDT`.

8. **Long-running task block routes to right tool by latency budget** — Designer asks "max sync user-wait" and "max job duration". <30s sync → serverless function. 30s-15min async → Inngest. 15min-2h → Trigger.dev or fly.io worker queue. AI inference → Modal (or fly-replay with GPU machines). *Grounding:* fly.io blueprints + Modal LLM inference + Hatchet AI DAG article. *Integration hook:* `PipelineSpec::async_backbone` enum, decided pre-FlashResearch.

9. **Per-tenant rate-limit defaults from billing tier** — if billing block has tiered plans, auto-derive rate-limit budgets per tier (free=10rpm, pro=100rpm, etc) and wire to Upstash Ratelimit middleware with `org_id` as key. *Grounding:* Upstash Ratelimit overview docs; OWASP API#4 unrestricted resource consumption. *Integration hook:* BabyStepSpec `ratelimit-per-tier`, auto-generated when both billing tiers and Upstash present.

10. **WCAG 2.2 AA + i18n scaffold even single-locale** — Lighthouse CI gate at 90+ a11y/perf/SEO from the first PR. i18n folder structure exists even if only `en` populated; adding a second locale is one file, not a refactor. *Grounding:* HumblDesign WCAG 2026; Paraglide TanStack 2026 article; 95.9% of top 1M pages fail WCAG. *Integration hook:* `PipelineSpec::a11y.lighthouse_min: u8 = 90` + Critic flag if removed.

## Anti-patterns

- **DB-per-tenant from day 1.** 10-100x ops cost before tenant #2 exists. (Hunchbite, Zenn 2025).
- **Application-layer-only tenant filter (no RLS).** One missing `WHERE tenant_id` and you breach. (Nile blog, Makerkit RLS guide).
- **Auth middleware on user routes but not `/api/admin/*`.** OWASP API#3 BOLA — most reported bug class in 2025-26 AI-coded apps. (AccuKnox OWASP 2026).
- **Unscoped CLI tokens granted to AI coding agents.** PocketOS lost its DB + backups in 9 seconds via a Railway token. (Pignati Apr 2026).
- **Secrets in `.env.local` committed to repo.** 28.65M leaks in 2025; AI-assisted repos leak 6.4% vs non-AI. (GitGuardian 2026).
- **Sentry SDK as the only observability stack.** Proprietary SDK lock-in. Use OTel SDK + Sentry exporter so future migration is free. (Axiom OTel 2026, Better Stack guide).
- **Roll-your-own webhook HMAC verification.** Use vendor lib; constant-time compare is easy to get subtly wrong.
- **No rate limit on signup / password-reset / magic-link endpoints.** Free DoS + email-bombing + cost explosion. (OWASP API#4).
- **Skipping i18n folder structure because "we only ship English".** Retrofit cost is high; folder + tooling first day is ~30 min. (Paraglide guide).
- **Lighthouse a11y score 100 = WCAG-compliant.** It does not. Pair with axe-core + manual keyboard test. (UXIA blog, HumblDesign).
- **Mocked DB calls left in production handlers.** Already covered in report 11 — at architecture level, Designer must require a contract test that hits real DB before any ship-gate.
- **Edge for everything.** Workers have 50ms CPU cap; long-running model inference + persistent TCP DB drivers do not work at edge. Edge for auth/A/B/personalization; origin/container for the rest. (Cloudflare edge guide 2026).
- **Single-region Postgres for global SaaS.** Add read-replicas + use Neon's branching for preview envs, not for prod scaling.

## Open questions

1. Should ELAI Designer default to **Wasp/Open SaaS** or **Next+Supabase Makerkit-style** as the B2C archetype? Measurable: ship-the-same-idea via each path, compare time-to-first-paying-user + audit pass-rate.
2. Is **Polar.sh** mature enough in May 2026 for ELAI to default to it as MoR? Measurable: end-to-end test subscribe → upgrade → mid-cycle proration → dunning flow; compare to Paddle and Lemon Squeezy on each.
3. **OTel + Sentry-exporter vs Sentry-native** — does the migration freedom justify the ~10% slower instrumentation DX? Measurable: instrumentation LoC + p95 overhead in one canonical app.
4. **Inngest vs Trigger.dev** as ELAI's default async backbone — which produces fewer audit BLOCKs from AI-generated handlers? Measurable: A/B 20 generated B-N plans across both, count handler bugs surfaced by auditor.
5. **Hybrid multi-tenancy promotion automation** — can we detect "this tenant should be promoted to schema isolation" from query plans + row counts and emit a BabyStep automatically? Measurable: detection precision/recall on a seeded multi-tenant Postgres workload.
