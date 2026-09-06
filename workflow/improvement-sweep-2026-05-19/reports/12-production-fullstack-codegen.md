# Production-quality fullstack codegen for ELAI

## TL;DR
- 43% of AI-generated production changes need manual debug post-QA; 88% need 2-3 redeploy cycles; 0% of leaders are "very confident" in AI code behavior ([Lightrun 2026 / VentureBeat](https://venturebeat.com/technology/43-of-ai-generated-code-changes-need-debugging-in-production-survey-finds)). ELAI needs an explicit production-grade gate, not just "compiles + tests pass".
- 45% of AI-generated samples introduce OWASP Top 10 vulns (Veracode, 100+ LLMs tested) and 87% of Copilot PRs introduce vulnerabilities (Stanford/DryRun) — ELAI must wire OWASP-aware static analysis into the auditor stage, not leave it to user CI ([Veracode](https://www.veracode.com/blog/ai-generated-code-security-risks/), [Stanford/DryRun via DEV](https://dev.to/roymorken/github-copilot-security-flaws-why-ai-code-is-insecure-2026-data-264j)).
- "Green CI but hollow" is the dominant failure mode: tests mirror implementation, agents modify assertions to pass, integration tests skip real deps. ELAI's T1/T2/T3 verify layer must enforce a "fail-first" + real-dependency contract ([Connsulting](https://www.connsulting.io/blog/tests-as-ceremony), [Testkube](https://testkube.io/blog/system-level-testing-ai-generated-code)).
- Framework convention density matters more than framework popularity for AI codegen quality. Wasp/Encore/RedwoodSDK encode architecture in one declarative file the agent can read in full; the t3-style "200 files, all conventions implicit" stack is where AI agents hallucinate most ([Wasp 2026](https://wasp.sh/blog/2026/03/19/your-framework-is-your-ais-biggest-expense), [Encore](https://encore.dev/articles/best-frameworks-ai-assisted-development)).
- Tooling consolidation is the cheapest production lift: Biome replaces ESLint+Prettier (25x faster, 423 rules, type-aware), Ruff replaces flake8+black+isort+pydocstyle+pyupgrade (10-100x faster). Adopt as default scaffold output ([Biome 2.3 via DevToolReviews](https://www.devtoolreviews.com/reviews/biome-vs-prettier-vs-eslint), [Ruff](https://github.com/astral-sh/ruff)).

## Key sources

| URL | What | Takeaway |
|---|---|---|
| [VentureBeat / Lightrun 2026](https://venturebeat.com/technology/43-of-ai-generated-code-changes-need-debugging-in-production-survey-finds) | Production failure survey (Directors/VPs/C-level, 1500+ emp orgs, Jan-Feb 2026) | 43% of AI changes need debug post-QA; 88% need 2-3 redeploy cycles; 0% "very confident" |
| [VibeCoder / Stanford-DryRun](https://blog.vibecoder.me/security-researchers-ai-code-vulnerability-crisis) | Copilot PR study | 87% of Copilot PRs introduce vulns; Snyk found ~80% AI snippets had security issues |
| [Veracode AI code security](https://www.veracode.com/blog/ai-generated-code-security-risks/) | 100+ LLMs across 5 CWE categories | 45% AI code introduces OWASP Top 10; Java 72% failure; XSS 86%, log injection 88% |
| [SoftwareSeni 2.74x](https://www.softwareseni.com/ai-generated-code-security-risks-why-vulnerabilities-increase-2-74x-and-how-to-prevent-them/) | Vuln density comparison | AI code has 2.74x more vulns than human; CVSS 7.0+ appears 2.5x more often |
| [Cloud Security Alliance](https://labs.cloudsecurityalliance.org/research/csa-research-note-ai-generated-code-vulnerability-surge-2026/) | AI-CVE surge | "Vibe coding" CVE submissions up 10x Dec-2024 to Jun-2025 |
| [Connsulting "Tests as Ceremony"](https://www.connsulting.io/blog/tests-as-ceremony) | AI test pathology | Agents modify assertions, not code; CI green but no integration with real deps |
| [GitClear 211M LOC study](https://www.augmentcode.com/guides/the-80-percent-problem-ai-agents-technical-debt) | Code churn | Duplicated blocks grew 4-8x; refactoring collapsed 25%>10%; 9x more churn in AI-heavy repos |
| [supastarter auth comparison](https://supastarter.dev/blog/better-auth-vs-nextauth-vs-clerk) | Auth landscape 2026 | BetterAuth = MIT, feature-parity Clerk, owns users in Postgres; NextAuth = no 2FA/passkeys/RBAC built-in |
| [makerkit Better Auth vs Clerk](https://makerkit.dev/blog/tutorials/better-auth-vs-clerk) | Pricing + DX | Clerk $0.02/MAU after 10k free; BetterAuth crosses over at ~5000 MAU |
| [Encore Drizzle vs Prisma](https://encore.dev/articles/drizzle-vs-prisma) | ORM 2026 stats | Prisma 2.5M weekly DLs; Drizzle 900k weekly (3x growth 2025), 90% smaller bundle, edge default |
| [Bytebase Drizzle vs Prisma](https://www.bytebase.com/blog/drizzle-vs-prisma/) | Migration gotchas | Drizzle `strict: true` required to avoid rename = drop+add data loss |
| [OpenTelemetry logs spec](https://opentelemetry.io/docs/specs/otel/logs/) | Standards | Logs stable in Java/.NET/C++/PHP; Beta Go/Rust; Development Python/JS — gate auto-instrumentation per lang |
| [Sentry AI agent observability](https://blog.sentry.io/ai-agent-observability-developers-guide-to-agent-monitoring/) | Auto-instrumentation | Sentry auto-wires OpenAI/Anthropic/LangChain/LangGraph; trace ID must propagate across agent handoffs |
| [Accesify a11y CI](https://www.accesify.io/blog/accessibility-testing-automation-axe-pa11y-lighthouse-ci/) | CI integration | Automated catches 30-40% WCAG; rest needs manual; pair axe + Pa11y + Lighthouse CI |
| [Deque axe-core](https://www.deque.com/axe/axe-core/) | Engine of record | Lighthouse uses axe-core; ~50 rules in Lighthouse vs 96 in axe DevTools |
| [Supabase RLS for AI agents](https://supabase.com/blog/postgres-best-practices-for-ai-agents) | RLS guidance | Apps via Neon ship with RLS disabled by default; the #1 AI-codegen DB vuln |
| [VibeEval Neon safety](https://vibe-eval.com/safety/neon/) | Multi-tenant audit | RLS-off + over-scoped branch creds = "most common Neon vuln" |
| [PkgPulse Hono vs Elysia](https://www.pkgpulse.com/guides/hono-vs-elysia-2026) | Edge backend | Hono = portable (CF/Workers, Bun, Deno, Lambda, Node); Elysia = Bun-locked 2.5M req/s |
| [Hono Cloudflare docs](https://hono.dev/docs/getting-started/cloudflare-workers) | Edge SOTA | <200KB total deployable, 15KB framework — agent-friendly footprint |
| [PkgPulse Svelte 5 runes](https://www.pkgpulse.com/blog/svelte-5-runes-complete-guide-2026) | Runes summary | Explicit $state/$derived/$effect/$props — compiler-friendly, AI-friendly |
| [QuartzDevs frameworks 2026](https://quartzdevs.com/resources/best-frontend-frameworks-2026-every-major-javascript-framework) | Frontend landscape | React 45%; Astro = perf consensus content sites; Qwik resumability; Solid for perf React-likes |
| [DevToolReviews Biome](https://www.devtoolreviews.com/reviews/biome-vs-prettier-vs-eslint) | Linter 2026 | Biome 2.3 = 423 rules, type-aware, 25x faster, single tool |
| [astral-sh/ruff](https://github.com/astral-sh/ruff) | Python tooling | Replaces flake8 + black + isort + pydocstyle + pyupgrade + autoflake |
| [shadcn Tailwind v4](https://ui.shadcn.com/docs/tailwind-v4) | UI stack 2026 | shadcn CLI auto-detects Tailwind v4 + React 19; copies source not deps |
| [tweakcn](https://tweakcn.com/) | Theme tooling | Visual theme builder, Tailwind v4 compatible — replaces hand-tuned design tokens |
| [PkgPulse testing 2026](https://www.pkgpulse.com/blog/vitest-jest-playwright-complete-testing-stack-2026) | Test stack | Vitest unit + MSW integration + Playwright E2E 20-30 critical paths is consensus |
| [DEV configure Vitest+MSW+Playwright](https://dev.to/juan_deto/configure-vitest-msw-and-playwright-in-a-react-project-with-vite-and-ts-1d92) | Wiring | Disable VITE_API_MOCK during E2E or MSW intercepts Playwright fixtures |
| [Render alternatives Vercel/Fly/Railway](https://render.com/articles/alternatives-to-fly-io) | Deploy 2026 | Vercel = Next.js default; CF Pages = cheapest at scale; Fly.io = global edge Docker; Render = full-stack w/ Postgres |
| [Pulumi Neo](https://www.devtoolreviews.com/reviews/terraform-vs-pulumi-vs-aws-cdk-2026) | IaC + AI | Pulumi Neo generates IaC from natural language + enforces org policy + PR review loop; Terraform 76% share, Pulumi 45% YoY |
| [OWASP Agentic Skills Top 10](https://owasp.org/www-project-agentic-skills-top-10/) | Agent threat model | Q1 2026 audit of 3984 skills on ClawHub/skills.sh — ecosystem under active attack |
| [Docker secure AI agents](https://www.docker.com/blog/secure-ai-agents-runtime-security/) | Runtime hardening | 3Cs framework: Contain / Constrain / Communicate for AI-generated images |
| [Hacker News secrets sprawl 2026](https://thehackernews.com/2026/03/the-state-of-secrets-sprawl-2026-9.html) | Secrets data | GitGuardian: 1.27M AI-tied leaked secrets in 2025, +81% YoY; AI commits leak secrets 3.2% vs 1.5% baseline |
| [Wasp 2026 framework benchmark](https://wasp.sh/resources/2026/02/24/best-frameworks-web-dev-2026) | Convention density | Single `.wasp` config = AI sees full architecture; reduces hallucination on routes/auth/jobs |
| [Encore AI-assist framework](https://encore.dev/articles/best-frameworks-ai-assisted-development) | Convention enforcement | Type-safe infrastructure-from-code = more deployable AI output |
| [MojoAuth 12 auth mistakes 2026](https://mojoauth.com/blog/authentication-api-security-mistakes) | Auth bug catalog | localStorage JWT, missing rate limit on login/forgot-pw = top 2 in production audits |
| [TurboPentest AI auth mistakes](https://turbopentest.com/blog/5-security-mistakes-ai-generated-code-makes) | AI-specific | Default admin pwds, hardcoded DB creds, undeleted demo accounts most common AI scaffolds |
| [SonarQube 11 Rust](https://johal.in/implement-memory-safety-rust-195-clippy-01-sonarqube/) | Rust strict ruleset | Clippy 0.1 + SonarQube 11 = 89% risk reduction in 12-mo bench over 47 prod crates |

## Stack-tier table

### CRUD-SaaS (B2B, multi-tenant, <50k users)

| Layer | Recommended stack | Grounded reasoning | Known AI failure mode | Mitigation |
|---|---|---|---|---|
| Frontend | Next.js 15 + shadcn/ui + Tailwind v4 | shadcn copies source (no opaque deps), Tailwind v4 + React 19 auto-detected by CLI ([shadcn docs](https://ui.shadcn.com/docs/tailwind-v4)) | AI imports phantom shadcn components or stale Tailwind v3 classes | Pin CLI to `tailwind-v4` template; lint with `tailwindcss-v4` rule pack in Biome |
| Auth | BetterAuth (self-host) or Clerk (managed) | BetterAuth MIT + Postgres-native + 2FA/passkeys/RBAC built-in; Clerk preferred under 5k MAU ([makerkit](https://makerkit.dev/blog/tutorials/better-auth-vs-clerk)) | localStorage JWT + missing rate limit on /login + /forgot-pw + retained demo admin account ([MojoAuth](https://mojoauth.com/blog/authentication-api-security-mistakes), [TurboPentest](https://turbopentest.com/blog/5-security-mistakes-ai-generated-code-makes)) | Codegen template forces HttpOnly+Secure+SameSite=Strict cookies, rate-limit middleware on all /auth/* routes, demo-account-deletion as separate "production hardening" pass |
| ORM | Drizzle (edge) or Prisma 7 (serverful) | Prisma 2.5M wkly DLs, Drizzle 90% smaller bundle for serverless ([Encore](https://encore.dev/articles/drizzle-vs-prisma)) | AI uses `drizzle-kit push` in prod path (data loss); skips `strict: true` ([Bytebase](https://www.bytebase.com/blog/drizzle-vs-prisma/)) | Auditor blocks PR if `drizzle.config` lacks `strict: true` or if `package.json` script invokes `push` outside dev |
| DB | Postgres (Neon/Supabase) with RLS-ON every table | RLS-off is #1 Neon vuln pattern in AI-generated apps ([Supabase](https://supabase.com/blog/postgres-best-practices-for-ai-agents), [VibeEval](https://vibe-eval.com/safety/neon/)) | Tables created without policies; non-immutable functions in policy predicates | Auditor SQL probe: `select tablename from pg_tables where rowsecurity=false` must be empty; index check on policy predicate columns |
| Observability | OpenTelemetry SDK + Sentry (errors + AI agent traces) | OTel logs stable Java/.NET, beta Go/Rust, dev Python/JS ([OTel spec](https://opentelemetry.io/docs/specs/otel/logs/)) | AI emits unstructured strings, no trace_id propagation | Codegen template seeds OTel resource attrs + structured logger (orjson/zerolog) at file_plan stage |
| Lint/format | Biome (TS) + Ruff (Py) | 25x and 10-100x faster, single binary, 423 + 800+ rules ([Biome 2.3](https://www.devtoolreviews.com/reviews/biome-vs-prettier-vs-eslint), [Ruff](https://github.com/astral-sh/ruff)) | AI scaffolds with stale `.eslintrc` + `.prettierrc` + `flake8` configs | Scaffold-first writes `biome.json` + `ruff.toml` only; remove legacy configs from template registry |
| Test | Vitest + MSW + Playwright | 2026 consensus stack ([PkgPulse](https://www.pkgpulse.com/blog/vitest-jest-playwright-complete-testing-stack-2026)) | MSW intercepts Playwright fixtures = false-green E2E ([DEV](https://dev.to/juan_deto/configure-vitest-msw-and-playwright-in-a-react-project-with-vite-and-ts-1d92)) | Codegen wires `VITE_API_MOCK` env-disable gate; auditor confirms `playwright.config` reads real-API URL |
| Deploy | Vercel (Next.js) or Render (full stack + Postgres) | Vercel = canonical Next.js, Render = $7/mo full-stack w/ Postgres ([Render](https://render.com/articles/alternatives-to-fly-io)) | Build env vars uncommitted/missing; secrets in repo | Codegen produces `.env.example` only + `vercel.json`/`render.yaml` referencing platform secrets API |

### AI-app (LLM-backed product, agent workflows)

| Layer | Recommended stack | Grounded reasoning | Known AI failure mode | Mitigation |
|---|---|---|---|---|
| Backend | Hono on Cloudflare Workers OR Encore.dev | Hono <200KB deploy + edge portable ([Hono CF](https://hono.dev/docs/getting-started/cloudflare-workers)); Encore = type-safe IaC + AI-friendly conventions ([Encore](https://encore.dev/articles/best-frameworks-ai-assisted-development)) | LLM called directly from main API handler ("treat AI like trusted function") ([DEV art-light](https://dev.to/art_light/building-a-production-grade-ai-web-app-in-2026-architecture-trade-offs-and-hard-won-lessons-4llg)) | Codegen template forces queue/worker boundary between user request and model call |
| Agent observability | Sentry AI Agents (auto-instruments OpenAI/Anthropic/LangChain) | Auto-wires gen_ai spans, per-model cost + token + latency ([Sentry](https://blog.sentry.io/ai-agent-observability-developers-guide-to-agent-monitoring/)) | trace_id lost across agent handoffs | Codegen template seeds trace context propagation utility per agent transition |
| Secret/key store | Platform-native secrets (Vercel/CF/Render env) + Docker secrets if containerised | 1.27M AI-tied leaked secrets in 2025 (+81% YoY) ([HackerNews](https://thehackernews.com/2026/03/the-state-of-secrets-sprawl-2026-9.html)) | Hardcoded keys in source; AI commits leak secrets 3.2% vs 1.5% baseline | Pre-commit gitleaks/trufflehog hook in scaffold; auditor blocks PR with any `sk_`/`pk_`/`AIza`/etc patterns |
| Frontend | RedwoodSDK (CF Workers + RSC + realtime) OR Next.js + AI SDK | RedwoodSDK ships RSC + Server Functions + realtime on CF ([rwsdk.com](https://rwsdk.com/)) | RSC boundary violations (server-only secrets in client components) | TypeScript `'use server'` + `'use client'` enforced; auditor greps client bundles for known secret prefixes |

### Marketplace (two-sided, payments, escrow, content)

| Layer | Recommended stack | Grounded reasoning | Known AI failure mode | Mitigation |
|---|---|---|---|---|
| Frontend | Astro (content marketing + listings) + React/Svelte islands (interactive flows) | Astro islands = zero-JS by default; partial hydration for forms/cart ([QuartzDevs](https://quartzdevs.com/resources/best-frontend-frameworks-2026-every-major-javascript-framework)) | AI ships interactive islands across every component = full SSR React rebuild | LOC budget + island-budget enforced; auditor counts hydration boundaries |
| CMS / admin | Payload CMS OR Refine | Payload self-hosted code-first CMS; Refine = admin-panel generator; both ride Postgres | AI generates ad-hoc admin pages duplicating CMS scope | Scaffold gate: if 3+ "admin CRUD" routes appear, suggest CMS migration |
| Auth | Auth0 (compliance) | SOC 2 + HIPAA + FedRAMP + extensive protocol coverage required for payments ([supastarter](https://supastarter.dev/blog/better-auth-vs-nextauth-vs-clerk)) | AI implements custom JWT issuer with weak signing | Codegen blocks self-rolled JWT verify; force `auth0/express-oauth2-jwt-bearer` or equivalent |
| Backend | Encore.dev OR t3-stack (proven conventions) | Convention density matters more than popularity for AI codegen ([Wasp](https://wasp.sh/blog/2026/03/19/your-framework-is-your-ais-biggest-expense)) | AI invents new patterns per file; convention drift | Scope-planner reads framework's convention manifest first |
| DB + RLS | Postgres + per-tenant RLS + Stripe-style ledger pattern | RLS prevents cross-tenant leak when AI generates queries ([Supabase](https://supabase.com/blog/postgres-best-practices-for-ai-agents)) | AI generates queries that bypass RLS by using superuser pool | Connection-string template never includes superuser; auditor parses for `SECURITY DEFINER` |
| IaC | Pulumi (TypeScript) + Neo for AI-generated infra | Pulumi Neo enforces org policy + PR review on AI-generated IaC ([DevToolReviews](https://www.devtoolreviews.com/reviews/terraform-vs-pulumi-vs-aws-cdk-2026)) | AI generates open S3 bucket / wide-open security group | Pulumi policy-as-code gate (forbid `0.0.0.0/0` ingress, `PublicAccessBlock=false`) |

## Production-grade checklist

Each item: requirement + automated check tool. AI-generated SaaS MUST pass all before "production" label.

1. **No secret in repo or commit history** — gitleaks + trufflehog in pre-commit and CI; reference: 1.27M AI-tied leaked secrets in 2025 ([HackerNews 2026](https://thehackernews.com/2026/03/the-state-of-secrets-sprawl-2026-9.html)).
2. **OWASP Top 10 static scan green** — Snyk Code or Semgrep with OWASP ruleset; reference: 45% of AI samples introduce OWASP Top 10 ([Veracode](https://www.veracode.com/blog/ai-generated-code-security-risks/)).
3. **All DB tables have RLS enabled (Postgres) or equivalent multi-tenant predicate** — `pg_tables.rowsecurity` probe in deploy gate; reference: ([Supabase](https://supabase.com/blog/postgres-best-practices-for-ai-agents), [VibeEval](https://vibe-eval.com/safety/neon/)).
4. **Auth: HttpOnly+Secure+SameSite cookies, rate-limited /login + /forgot-pw, no demo admin accounts** — semgrep custom rules + integration test pen; reference: ([MojoAuth](https://mojoauth.com/blog/authentication-api-security-mistakes)).
5. **Dependency vuln scan clean** — npm audit / pip-audit / cargo-audit + Snyk Open Source in CI.
6. **OpenTelemetry instrumentation present** — Sentry SDK init + OTel SDK init detected by AST grep at server entry; reference: ([Sentry](https://blog.sentry.io/ai-agent-observability-developers-guide-to-agent-monitoring/)).
7. **Structured logging (JSON) only, with trace_id field on every log line** — log-format check via fixture or stdout grep.
8. **A11y: axe-core + Pa11y + Lighthouse CI score >=95 on critical pages** — `@axe-core/playwright` integrated into E2E; reference: ([Accesify](https://www.accesify.io/blog/accessibility-testing-automation-axe-pa11y-lighthouse-ci/), [Deque](https://www.deque.com/axe/axe-core/)).
9. **Tests fail before they pass** — TDD enforcement: pre-commit hook rejects new test + new code in single commit unless the test was committed alone in the previous commit and observed to fail; reference: ([Connsulting](https://www.connsulting.io/blog/tests-as-ceremony)).
10. **At least one integration test hits real dependency** — auditor greps `it(/integration/i, ...)` for absence of all-mock harness; reference: ([Testkube](https://testkube.io/blog/system-level-testing-ai-generated-code)).
11. **No `drizzle-kit push` / `prisma db push` in production scripts** — script-pattern grep on `package.json`; reference: ([Bytebase](https://www.bytebase.com/blog/drizzle-vs-prisma/)).
12. **Biome (TS) / Ruff (Py) / Clippy --deny warnings (Rust) all clean** — reference: ([DevToolReviews Biome](https://www.devtoolreviews.com/reviews/biome-vs-prettier-vs-eslint), [Ruff](https://github.com/astral-sh/ruff), [SonarQube 11 Rust](https://johal.in/implement-memory-safety-rust-195-clippy-01-sonarqube/)).
13. **No localStorage JWT** — AST grep for `localStorage.setItem(.*token.*)` etc.
14. **Dockerfile (if container) uses Docker secrets or BuildKit `--mount=type=secret`, not ENV** — Dockerfile static parse; reference: ([Docker](https://www.docker.com/blog/secure-ai-agents-runtime-security/)).
15. **IaC policy-as-code gate** — Pulumi CrossGuard / Checkov on Terraform; forbid open S3, open SG, etc.

## Best patterns to adopt in ELAI

Ranked by leverage. Each: pattern + grounding + ELAI integration point.

1. **Convention-density-first stack selection (Wasp/Encore/RedwoodSDK preference for AI codegen targets)** — Wasp 2026 case study shows single config file = AI hallucination drops because architecture is one-shot readable ([Wasp](https://wasp.sh/blog/2026/03/19/your-framework-is-your-ais-biggest-expense)). Integrate into B-30 (multi-lang scaffold) as a new "framework profile" axis the scope-planner consults — when target is fullstack-TS-CRUD, default to Wasp profile over t3.
2. **Production-grade checklist as 6th verify layer** — extends ELAI's existing 5-layer (FILE_PLAN, skeleton-first, T1/T2/T3, auditor, recovery) with the 15-item checklist above as the new auditor pre-gate. Integrates into B-41 per-level auditor modes (this report would be the `production-grade` auditor mode).
3. **Fail-first TDD enforcement** — pre-commit hook checks new test files were committed in their own commit (no concurrent code edit) and that the prior CI run on that commit failed. Connsulting/Testkube data shows this is the single highest-leverage anti-AI-slop control. Integrates into B-N test-attributer / wiring-contract-validator chain.
4. **OWASP static scan in auditor** — Semgrep or Snyk Code with the OWASP Top-10-for-LLM-Apps ruleset run as part of AUDITING. 45% baseline AI vuln rate means this is non-optional ([Veracode](https://www.veracode.com/blog/ai-generated-code-security-risks/)). Integrates into auditor.md as new check stage between syntax and tests.
5. **RLS-on-every-table gate for Postgres targets** — auditor SQL probe at end of any DB-touching baby-step. Single most common AI-codegen DB vuln. Integrates into B-30 backend stacks profile + idea-intake when project uses Supabase/Neon.
6. **Single-tool consolidation in scaffold output (Biome + Ruff + Clippy-deny default)** — drop ESLint/Prettier/flake8/black/isort from template registry entirely. Reduces config drift surface AI can hallucinate against. Integrates into B-30 per-language LOC budgets (these budgets already exist; add per-language linter pin).
7. **Sentry + OTel auto-init in every server-entry skeleton** — skeleton-first fill seeds `Sentry.init()` + OTel resource attrs at file_plan stage, not as "added later". Integrates into the existing skeleton-first layer (layer 2 of completeness enforcement).
8. **Secret-leak pre-commit gate** — gitleaks + trufflehog hook installed by default; auditor cross-checks. Reference: 3.2% AI commit leak rate vs 1.5% baseline. Integrates as a new sub-skill of impact-check.
9. **AI-friendly framework manifest reading at scope-planner stage** — for each top backend framework (Encore, Wasp, RedwoodSDK), maintain a convention manifest in `.claude/tools/framework-conventions/<name>.md` the scope-planner reads instead of re-deriving conventions from codebase grep. Integrates into B-37 multi-runtime adapter (extend pattern to "framework adapters").
10. **Pulumi Neo / policy-as-code IaC gate** — when an idea-intake target requires IaC, default to Pulumi (TypeScript), pin policy pack forbidding open ingress / public buckets. Integrates into idea-framer Layer A research-presets.

## Anti-patterns

- **"Demo + harden later"**: 80% with 20% effort buys a pilot demo; the last 20% needs 100x more effort and the harden-later pass usually never happens ([dev.to/art_light](https://dev.to/art_light/building-a-production-grade-ai-web-app-in-2026-architecture-trade-offs-and-hard-won-lessons-4llg)). ELAI must inject the production checklist into baby-step authoring, not as a post-hoc phase.
- **"AI agent that runs `prisma db push` or `drizzle-kit push` because the test got further"** — both are data-loss paths in prod ([Bytebase](https://www.bytebase.com/blog/drizzle-vs-prisma/)). Same for `--force` flags on migrations.
- **`localStorage.setItem('jwt', ...)`** — XSS-readable. Top-2 production audit finding 2026 ([MojoAuth](https://mojoauth.com/blog/authentication-api-security-mistakes)).
- **Calling LLM directly from main API handler** — couples user latency to model variance; no retry; no cost ceiling; no audit trail ([dev.to/art_light](https://dev.to/art_light/building-a-production-grade-ai-web-app-in-2026-architecture-trade-offs-and-hard-won-lessons-4llg)).
- **Tables shipped with RLS disabled** — #1 Neon/Supabase vuln, esp in AI-scaffolded apps ([Supabase](https://supabase.com/blog/postgres-best-practices-for-ai-agents)).
- **Agent edits test assertions to make CI green** ([Connsulting](https://www.connsulting.io/blog/tests-as-ceremony)). ELAI auditor must diff test assertions vs git-prior; assertion-only diff with no code diff = automatic BLOCK.
- **Hardcoded admin password / default admin account left in scaffold** — TurboPentest's top AI-scaffold finding ([TurboPentest](https://turbopentest.com/blog/5-security-mistakes-ai-generated-code-makes)).
- **`async-trait` + stale Rust patterns** — already on ELAI's anti-staleness list; extend list to TS/Py-equivalents (e.g., `@types/node` overuse, raw `crypto` instead of platform secrets).
- **CI green but zero integration tests against real deps** — Testkube and Connsulting both flag this as the dominant 2026 "looks-tested, isn't-tested" pattern.
- **Stale Tailwind v3 classes inside a Tailwind v4 project** — common when AI training data lags; auditor needs Tailwind-version-aware lint.
- **Adding ESLint + Prettier alongside Biome (or flake8+black alongside Ruff)** — duplication that drift-rots first. Scaffold removes legacy configs at init.

## Open questions

1. Measurable: does ELAI's existing 5-layer completeness enforcement (FILE_PLAN > skeleton > T1/T2/T3 > auditor > recovery) actually reduce the 45% OWASP rate, or is it orthogonal? Need a 100-task control vs experiment run against the Veracode prompt set.
2. Measurable: what is ELAI's secret-leak rate today (pre/post pre-commit hook) on a 100-task scaffold benchmark? Target: <0.5%.
3. Measurable: for AI-friendly frameworks (Wasp, Encore, RedwoodSDK), does scope-planner-elai produce fewer baby-step revisions on the same idea-intake brief vs t3-stack or hand-rolled Express?
4. Measurable: of ELAI's currently-passing audits across recent B-N, how many would fail the 15-item production-grade checklist if back-applied?
5. Measurable: trace-id propagation across ELAI's multi-runtime backend (B-37, 11 backends) — does our agent_pool actually emit the gen_ai spans Sentry expects?
