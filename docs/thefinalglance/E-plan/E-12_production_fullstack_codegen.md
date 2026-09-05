# E-12: Production fullstack codegen — framework profiles + 15-item checklist + RLS / secrets / OWASP gates
Objective: ship the production-grade auditor mode introduced by E-11 with concrete framework profiles (Wasp, Encore, RedwoodSDK, t3, Hono), RLS-on-by-default Postgres gates, gitleaks pre-commit, OWASP semgrep ruleset, OTel + Sentry init in skeletons, and Biome/Ruff/Clippy-deny default scaffold. Closes the demo-vs-deploy chasm by making the executor and scope-planner consume convention-density manifests rather than re-derive conventions per file.
Estimated effort: 3 sessions (~12 h).
Pre-requisites: B-30 PLANNED (Multi-Language DKG Hardening — language registry), B-37 SHIPPED (Multi-Runtime Backend Abstraction — adapter pattern), C-21 PLANNED (MistakeBook), C-31 PLANNED (FilePlan), C-32 PLANNED (Skeleton-first), E-11 (Production-grade auditor mode + Package verifier).
Parallel-safe with: E-13 (TDD gates), E-14 (browser E2E), B-28 Focus Tunnel (PLANNED).
Source report: workflow/improvement-sweep-2026-05-19/reports/12-production-fullstack-codegen.md

## Context

VentureBeat / Lightrun 2026 measured 43% of AI-generated production changes needing post-QA debug, 88% needing 2-3 redeploy cycles, and 0% of leaders "very confident" in AI code behavior. Veracode (45% OWASP-Top-10 baseline) and the Stanford/DryRun Copilot study (87% of Copilot PRs introduce vulns) converge on the same conclusion: ELAI's existing "compiles + tests pass" gate is not a production gate. The Wasp / Encore / RedwoodSDK 2026 case studies measured that single-file declarative-convention frameworks reduce AI hallucination on routes/auth/jobs because the agent sees the full architecture one-shot. The 2026 tooling consolidation wins (Biome 25x faster than ESLint+Prettier, 423 rules; Ruff 10-100x faster than flake8+black+isort+pydocstyle+pyupgrade) are cheap to adopt and remove the entire class of "stale .eslintrc + .prettierrc + flake8 alongside Biome" drift bugs that AI scaffolds introduce. Supabase's 2026 "Postgres best practices for AI agents" calls out RLS-off as the #1 Neon/Supabase vulnerability pattern in AI-generated apps, with VibeEval's safety audit confirming it is the most-common Neon vuln. ELAI's B-30 language-registry surface and B-37 runtime adapter pattern provide the right substrate for framework profiles; this E-N adds the production-readiness checklist runners + framework-convention manifests on top of E-11's ProductionGrade auditor mode.

## Existing state (B/C/D coverage)

- EXTENSION of E-11 ProductionGrade auditor mode — extends the 15-item checklist into a per-framework superset (12 base + 3-6 per-framework).
- EXTENSION of B-37 RuntimeAdapter pattern (SHIPPED, .elai_cc/crates/providers/src/runtime/cli_common.rs:337) — adds a FrameworkAdapter parallel layer for codegen target profiles.
- AUGMENTS B-30 (PLANNED) Multi-Language DKG Hardening — framework profiles register their language preferences against B-30's LanguageRegistry.
- AUGMENTS C-31 FilePlan + C-32 Skeleton-first — Wasp/Encore manifests are read at scope-planner stage as a convention source for the FilePlan generator.
- AUGMENTS C-21 MistakeBook — adds new categories `RlsOff`, `SecretInRepo`, `LegacyLinterDuplicate`, `LocalStorageJwt`, `DemoCredsLeft`.
- DELTA from B-28 Focus Tunnel (PLANNED, FIM / DIFF / STRUCTURED modes) — framework profiles set per-framework FIM eligibility (Wasp .wasp file = STRUCTURED only; React component = FIM-eligible).
- NET-NEW: framework-convention manifests at `.claude/tools/framework-conventions/{wasp,encore,redwoodsdk,t3,hono,fastapi-fastify,nextjs-app-router}.md`, read by scope-planner-elai (mandatory before scope-decompose).

## Reads (mandatory before coding)

- workflow/improvement-sweep-2026-05-19/reports/12-production-fullstack-codegen.md
- .elai_cc/crates/orchestrator/src/auditor_mode.rs (E-11 ProductionGrade variant — must exist or be staged)
- .elai_cc/crates/providers/src/runtime/cli_common.rs:337 (TierCSpawn adapter shape — model FrameworkAdapter on this)
- .elai_cc/crates/knowledge/src/cosmos_worker.rs (cosmos manifest pattern; framework manifests will live alongside)
- .elai_cc/elai_cc.toml (existing [research_presets.*] + [runtime] tables)
- .claude/agents/scope-planner-elai.md (which conventions the planner currently reads; new section to add)
- .claude/agents/auditor.md (where ProductionGrade MODE DISPATCH block lives — per E-11 task #2)
- workflow/papers/dgm-interface.md
- Wasp 2026 framework manifest spec (via paper-ingestor mode=source, target Wasp official docs single-config-file pattern)
- Supabase 2026 Postgres-RLS-for-AI-agents post (via paper-ingestor mode=source)

## Tasks

1. Author 5 framework-convention manifests at `.claude/tools/framework-conventions/` per .md file: wasp.md, encore.md, redwoodsdk.md, t3.md, hono.md, fastapi.md, nextjs-app-router.md. Each manifest is plain text plus a fenced YAML block per the existing rules-file convention (no markdown headers, every token meaningful per output-formatting rule). YAML block lists: root_config_file, route_convention, auth_convention, db_convention, env_convention, scaffold_command, linter_default, formatter_default, test_runner_default, dependency_manifest_paths, banned_patterns (stale Tailwind v3 classes for Tailwind v4 projects, localStorage.setItem JWT, prisma db push in prod scripts, drizzle-kit push outside dev, hardcoded admin creds). The manifests are token-compact (<200 lines each) so the scope-planner can read them in full.

2. Add `FrameworkProfile` enum + `FrameworkAdapter` trait in new file `.elai_cc/crates/orchestrator/src/framework_adapter.rs`. Variants: Wasp, EncoreDev, RedwoodSdk, T3Stack, HonoEdge, FastApi, NextAppRouter, Generic. `FrameworkAdapter` trait methods: `scaffold(spec: &ProjectSpec) -> Result<Vec<FileSpec>>` (returns initial file plan), `production_checklist() -> Vec<ChecklistItem>` (framework-specific items added to the 12-base), `apply_lint_defaults(file_plan: &mut FilePlan)` (forces biome.json / ruff.toml / clippy.toml emission and strips legacy configs). Wire dispatch from IdeaFramer's ProjectClass output (B-40) into FrameworkProfile selection via a deterministic table `PROJECT_CLASS_TO_FRAMEWORK` in elai_cc.toml.

3. Implement the 12-item base checklist as deterministic Tier-C runners in `.elai_cc/crates/orchestrator/src/production_checklist/` (sub-module per check): `secrets_scan.rs` (gitleaks + trufflehog subprocess; 30s timeout), `owasp_static.rs` (semgrep --config=p/owasp-top-ten, 60s), `rls_probe.rs` (sqlite3 + pg_dump parser detecting `CREATE TABLE` without `ENABLE ROW LEVEL SECURITY`), `auth_hardening.rs` (semgrep custom ruleset for HttpOnly/Secure/SameSite cookies, rate-limit on /auth/* routes, no demo admin account), `dep_audit.rs` (npm audit / pip-audit / cargo-audit with --json), `otel_present.rs` (AST grep on server-entry for `Sentry.init` + OTel resource attrs), `structured_logging.rs` (grep + fixture sanity check on log format), `axe_lighthouse.rs` (lighthouse-ci subprocess; >=95 on critical pages; gated by frontend_pair non-null), `tdd_fail_first.rs` (git log analysis: new test file's first commit must be alone, second-commit must add code), `integration_real_dep.rs` (AST grep for `it(/integration/i, ...)` against the all-mock harness), `migration_push_ban.rs` (script-pattern grep on package.json for prod-path `drizzle-kit push` / `prisma db push`), `linter_clean.rs` (biome / ruff / clippy --deny warnings subprocess).

4. Add the 3 remaining checks: `localstorage_jwt.rs` (AST grep `localStorage.setItem(.*token.*)` etc.), `dockerfile_secrets.rs` (Dockerfile static parse rejecting ENV for secrets; require BuildKit `--mount=type=secret`), `iac_policy_as_code.rs` (Pulumi CrossGuard / Checkov subprocess for Terraform; forbid 0.0.0.0/0 ingress, PublicAccessBlock=false, open S3 buckets). All checks emit findings into the LevelAuditVerdict.findings Vec with category enum extended to {CSRF_MISSING, SECRET_LEAK, RLS_OFF, OWASP_TOP10, PACKAGE_HALLUCINATED, AUTH_WEAK, LINTER_DUPLICATE, LOCALSTORAGE_JWT, DEMO_CREDS, MIGRATION_PUSH, DOCKERFILE_ENV_SECRET, IAC_OPEN_INGRESS, AXE_FAIL, OTEL_MISSING, INTEGRATION_ALL_MOCK}.

5. Wire FrameworkAdapter into scope-planner-elai's flow. Add a new section in `.claude/agents/scope-planner-elai.md` "Framework profile selection" that mandates: (a) read elai_cc.toml [research_presets.{class}] → pick profile, (b) Read .claude/tools/framework-conventions/{profile}.md in full, (c) emit decomposition with file_plan that respects the manifest's route_convention / auth_convention / db_convention, (d) ban patterns from the manifest's banned_patterns list go into MistakeBook as INTERACTIVE-block on first occurrence.

6. Add Biome / Ruff / Clippy-deny default scaffold output. When FrameworkAdapter::scaffold runs, it emits exactly one linter+formatter config per language with the legacy configs explicitly removed from FilePlan.modified_files (delete .eslintrc, .prettierrc, .flake8, setup.cfg's flake8 section, pyproject.toml's black + isort sections). Emit `legacy_config_stripped` SSB row per file removed. Wire into C-32's flush_to_disk so the strip happens in a single atomic write per file.

7. Seed OTel + Sentry init in every server-entry skeleton. New const table `SERVER_ENTRY_SEEDS` in framework_adapter.rs keyed by FrameworkProfile; values are token-budgeted code stubs (15-30 LOC each) that include `Sentry.init({ dsn: process.env.SENTRY_DSN, ... })`, OTel resource attrs, and a structured logger (orjson in Python, zerolog in Go, pino in Node — only declared if dependency_manifest_paths confirms the dep can be added). The seed is part of the skeleton emission at C-32's SkeletonGenerator::generate; the BLANK_N fill cannot remove the seed (MiniCritic flags removal as BLOCK).

8. Add `[framework_profiles]` table to elai_cc.toml with entries: default_profile_per_class (SaaS=WaspOrEncore, ApiBackend=Hono, AiApp=HonoCloudflare, Marketplace=Encore, ContentSite=Astro). Add `[secrets_scan]` table: `enabled_default = true`, `pre_commit_hook = true`, `block_on_match = true`. Add `[rls_probe]` table: `block_on_disabled_table = true`, `audit_superuser_pool = true`. Add `[linter_consolidation]` table: `strip_legacy_on_scaffold = true`, `enforced_linter_per_lang = { ts = "biome", py = "ruff", rs = "clippy_deny" }`.

9. Install gitleaks pre-commit hook by default. Add `.elai_cc/.git/hooks/pre-commit` template installed by the existing `install_pre_commit_hook.sh` (per C-21 schedule) extended with a gitleaks call; on first hit, BLOCK and emit MistakeBook category `SecretInRepo`. The hook also runs trufflehog as a second-source verifier; if either reports a finding, BLOCK. The hook supports `GITLEAKS_ALLOWLIST` env for known false-positive paths.

10. Wire the AI-friendly framework convention reading at scope-planner stage into the existing per-level audit dispatch (B-41) and per_level_critic (E-11). At Phase + Step level, the critic re-reads the framework manifest's "banned_patterns" list and emits findings for any banned pattern observed in the FILE_PLAN. This is the Wasp/Encore "single config file = AI sees full architecture" pattern applied to ELAI's per-level critic.

11. Add `[testing.assertions]` table to elai_cc.toml + AST grep that diffs test assertions vs git-prior. If a commit changes assertions but does not change implementation (the "agent edits tests to make CI green" anti-pattern, Connsulting 2026), the auditor emits BLOCK with category `ASSERTION_ONLY_DIFF`. This complements E-13's fail-first TDD gate but ships independently because the AST grep is cheap and orthogonal.

12. Dashboard: extend `IdeaFramerSecurityPanel.svelte` (E-11) with a "Framework profile" section showing the selected FrameworkProfile + the manifest summary + the 15 (or 18 with framework-specifics) checklist rows. New Tauri command `get_framework_profile` reads from the new SsbActionType::FrameworkProfileSelected payload {profile, manifest_path, scaffold_paths[]}. Emit SSB row on scope-planner exit when profile is picked.

## WIRING_CONTRACT
produces_symbols:
  - orchestrator::framework_adapter::FrameworkProfile
  - orchestrator::framework_adapter::FrameworkAdapter (trait)
  - orchestrator::framework_adapter::SERVER_ENTRY_SEEDS
  - orchestrator::production_checklist::* (12 runners + 3 net-new = 15)
  - .claude/tools/framework-conventions/{wasp,encore,redwoodsdk,t3,hono,fastapi,nextjs-app-router}.md
  - dashboard-v2::commands::get_framework_profile
consumes_symbols:
  - orchestrator::auditor_mode::AuditorMode::ProductionGrade (E-11) — consumes checklist findings
  - orchestrator::idea_framing::ProjectClass (B-40) — drives profile selection
  - orchestrator::file_plan::FilePlan (C-31) — receives scaffold output
  - orchestrator::skeleton_generator::SkeletonGenerator (C-32) — receives SERVER_ENTRY_SEEDS
  - orchestrator::mistake_book (C-21) — receives 5 new categories
reachable_from:
  - crates/cli/src/main.rs::main → task_runner::drive_task → scope_planner → framework_adapter
  - crates/dashboard-v2/src-tauri/src/lib.rs::submit_task
stub_ban:
  - todo!() / unimplemented!() in production_checklist/*
  - silent error swallow in any of the 15 checklist runners (use FrameworkCheckError thiserror enum)
  - hardcoded paths in checklist runners — use core::paths helpers
  - Box<dyn Error> in framework_adapter.rs
frontend_pair: crates/dashboard-v2/src/lib/components/IdeaFramerSecurityPanel.svelte (extended from E-11)
observable_effect:
  trigger: "elai /task --mode cloud 'build a production SaaS with auth, billing, and a Postgres backend using a framework you choose'"
  expect_ssb: "framework_profile_selected within 30s of FramingConverged, manifest_path is .claude/tools/framework-conventions/{wasp,encore}.md, legacy_config_stripped rows for any pre-existing .eslintrc/.prettierrc/.flake8 in scaffold target dir"
  expect_artifact: ".elai_cc/sessions/{id}/audit/production_grade_checklist.toml has 15+ rows, RLS_OFF row absent if framework_profile.db_convention enforces RLS-on, OTEL_MISSING absent because skeleton seeded Sentry.init"

## DETERMINISTIC_CHECK
```bash
# 1. All 7 framework manifests exist
ls .claude/tools/framework-conventions/ | wc -l
# expect: >=7 .md files

# 2. FrameworkProfile + FrameworkAdapter present
rg "pub enum FrameworkProfile|pub trait FrameworkAdapter" .elai_cc/crates/orchestrator/src/framework_adapter.rs
# expect: 2 hits

# 3. 15 checklist runners present
ls .elai_cc/crates/orchestrator/src/production_checklist/*.rs | wc -l
# expect: >=15 .rs files

# 4. SERVER_ENTRY_SEEDS keyed by all framework profiles
rg "SERVER_ENTRY_SEEDS" .elai_cc/crates --type rust
# expect: >=2 hits (def + use in skeleton_generator)

# 5. gitleaks pre-commit hook installer extended
rg "gitleaks|trufflehog" .elai_cc/scripts/install_pre_commit_hook.sh
# expect: >=2 hits

# 6. legacy_config_stripped SSB action_type registered
rg "legacy_config_stripped|framework_profile_selected" .elai_cc/crates/orchestrator/src/ssb.rs
# expect: >=2 hits

# 7. Smoke: production-grade audit on a known-RLS-off fixture blocks
cargo test -p elai-orchestrator --test production_checklist_rls_probe blocks_on_rls_disabled_table
# expect: pass

# 8. Smoke: scaffold strips legacy configs deterministically
cargo test -p elai-orchestrator --test framework_adapter_scaffold strips_eslintrc_when_biome_chosen
# expect: pass
```

## ACCEPTANCE
- [ ] 7 framework manifests authored at .claude/tools/framework-conventions/ each <=200 lines, plain text + YAML block
- [ ] FrameworkProfile enum + FrameworkAdapter trait wired; PROJECT_CLASS_TO_FRAMEWORK table in elai_cc.toml dispatches deterministically
- [ ] 15 production_checklist runners under .elai_cc/crates/orchestrator/src/production_checklist/, each its own file, each its own subprocess timeout
- [ ] LevelAuditVerdict.findings category extended to 15 entries; serde-compatible with B-41 shipped payloads
- [ ] SERVER_ENTRY_SEEDS per FrameworkProfile; SkeletonGenerator emits seed; MiniCritic blocks removal
- [ ] [framework_profiles] + [secrets_scan] + [rls_probe] + [linter_consolidation] + [testing.assertions] tables in elai_cc.toml
- [ ] gitleaks + trufflehog pre-commit hook auto-installed; MistakeBook records SecretInRepo on hit
- [ ] legacy_config_stripped + framework_profile_selected SSB action_types registered
- [ ] IdeaFramerSecurityPanel.svelte extended with framework-profile section; get_framework_profile Tauri command wired
- [ ] No NoopSsbNotifier in any new module; no Box<dyn Error>; no .unwrap() outside #[cfg(test)]

## Tests to add
- `.elai_cc/crates/orchestrator/tests/framework_adapter_scaffold.rs`: 7 cases (one per FrameworkProfile)
- `.elai_cc/crates/orchestrator/tests/production_checklist_rls_probe.rs`: 4 cases (rls-on / rls-off / mixed / no-tables)
- `.elai_cc/crates/orchestrator/tests/production_checklist_secrets_scan.rs`: 5 cases (clean / sk_prefix / pk_prefix / AKIA / mixed)
- `.elai_cc/crates/orchestrator/tests/production_checklist_owasp.rs`: 6 cases (XSS / SQLi / SSRF / log-injection / open-redirect / clean)
- `.elai_cc/crates/orchestrator/tests/production_checklist_localstorage_jwt.rs`: 3 cases
- `.elai_cc/crates/orchestrator/tests/skeleton_seed_otel.rs`: 4 cases (skeleton has seed / fill removes seed → BLOCK / fill keeps seed / framework-specific seed correct)
- `.elai_cc/crates/orchestrator/tests/assertion_only_diff.rs`: 3 cases
- `.elai_cc/crates/contracts/tests/framework_manifest_link_check.rs`: parses each .md manifest; verifies YAML keys present

## Files touched
- CREATE: `.claude/tools/framework-conventions/wasp.md` (~180 LOC)
- CREATE: `.claude/tools/framework-conventions/encore.md` (~180 LOC)
- CREATE: `.claude/tools/framework-conventions/redwoodsdk.md` (~180 LOC)
- CREATE: `.claude/tools/framework-conventions/t3.md` (~180 LOC)
- CREATE: `.claude/tools/framework-conventions/hono.md` (~150 LOC)
- CREATE: `.claude/tools/framework-conventions/fastapi.md` (~150 LOC)
- CREATE: `.claude/tools/framework-conventions/nextjs-app-router.md` (~180 LOC)
- CREATE: `.elai_cc/crates/orchestrator/src/framework_adapter.rs` (~420 LOC)
- CREATE: `.elai_cc/crates/orchestrator/src/production_checklist/mod.rs` + 15 sub-files (~120 LOC each, ~1800 LOC total)
- MODIFY: `.elai_cc/crates/orchestrator/src/auditor_mode.rs` (+~80 LOC; consume FrameworkProfile-specific checklist items)
- MODIFY: `.elai_cc/crates/orchestrator/src/skeleton_generator.rs` (+~60 LOC; seed integration) — file authored by C-32
- MODIFY: `.elai_cc/elai_cc.toml` (+~80 LOC; 5 new tables)
- MODIFY: `.elai_cc/scripts/install_pre_commit_hook.sh` (+~25 LOC; gitleaks + trufflehog)
- MODIFY: `.elai_cc/crates/orchestrator/src/ssb.rs` (+~30 LOC; 2 new SsbActionType variants)
- MODIFY: `.claude/agents/scope-planner-elai.md` (+~25 LOC; Framework profile selection section)
- MODIFY: `.elai_cc/crates/dashboard-v2/src/lib/components/IdeaFramerSecurityPanel.svelte` (+~80 LOC; framework section)
- MODIFY: `.elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs` (+~30 LOC; get_framework_profile)

## Citations / Grounding
- VentureBeat / Lightrun 2026 (Jan-Feb 2026 survey) — 43% / 88% / 0% confidence stats; ungrounded confidence number is the report's headline, treat as directional
- Veracode 2025 100-LLM study — 45% OWASP-Top-10 baseline; checklist item #2 grounding
- Wasp 2026 "your-framework-is-your-ais-biggest-expense" — single-config-file hallucination reduction; framework manifest grounding
- Encore.dev "best-frameworks-ai-assisted-development" — convention density as AI-codegen lever
- Supabase 2026 "Postgres best practices for AI agents" — RLS-off as #1 Neon vuln; checklist item #3 grounding
- VibeEval 2026 safety/neon — RLS-off multi-tenant audit
- Biome 2.3 DevToolReviews — 25x faster, 423 rules, type-aware; linter-consolidation grounding
- Astral Ruff README — 10-100x faster than flake8+black+isort+pydocstyle+pyupgrade
- MojoAuth 2026 12-auth-mistakes — localStorage JWT + missing rate-limit on /login + /forgot-pw
- TurboPentest 2026 — default admin pwd / hardcoded DB creds / undeleted demo accounts
- GitGuardian 2026 secrets sprawl report — 1.27M AI-tied leaked secrets in 2025 (+81% YoY); secrets-scan grounding
- Connsulting "Tests as Ceremony" — agent edits assertions not code; ASSERTION_ONLY_DIFF grounding
- Testkube 2026 system-level testing — CI-green-no-real-deps anti-pattern; integration_real_dep grounding
- Pulumi CrossGuard + DevToolReviews 2026 IaC review — policy-as-code IaC gate grounding

## Open questions / risks
- Ungrounded: framework-profile decision table (PROJECT_CLASS_TO_FRAMEWORK) is a starting heuristic from the report's stack-tier tables; needs empirical baseline once 20+ idea-framing sessions have produced ProjectClass labels.
- Wasp 2026 framework manifest spec must be source-grounded via paper-ingestor before the manifest is authored; if the public docs do not expose the full convention surface, file-stub the manifest with TODO[E-12]: pull from <url> and treat as advisory until verified.
- gitleaks + trufflehog as default pre-commit hook adds ~200ms per commit on a clean repo; the cost is acceptable per Hacker News 2026 secrets-sprawl data but must be measurable — auditor reports commit-time delta in metrics/last_measure_run.toml.
- The 15-item checklist depends on Tier C subprocess availability (B-18 / C-11). If Tier C is not active, the checklist degrades to a warning-only mode and emits `production_checklist_degraded_no_sandbox`; production_grade verdict downgrades to advisory.
- The OWASP semgrep ruleset version pin is volatile (rules update monthly); pin a specific commit in elai_cc.toml [secrets_scan] semgrep_ruleset_pin and refresh quarterly via paper-ingestor mode=source.
- The localStorage JWT AST grep is a pattern match; React Native / mobile contexts may use SecureStore legitimately. Add a per-FrameworkProfile allowlist mechanism for environments where the rule does not apply.

## Next
- E-13 (TDD) plugs into the production_checklist's `tdd_fail_first.rs` and `integration_real_dep.rs` runners and provides the mutation-survival deterministic signal.
- E-14 (browser E2E) extends `axe_lighthouse.rs` with Playwright + AXTree-first checks.
- E-11's scope_ceiling table receives a new entry for any framework where the manifest declares "external_auth_complex = true" (Replit Agent 3's documented failure mode).
- Open Q4 from the report (back-apply checklist to historical audits) becomes a one-shot retroactive run as a follow-up patch-N after E-12 ships.
