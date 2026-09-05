E-12 Tier-B — production fullstack codegen, decisions log
=========================================================

Session: s-25120 session 2/4 (concurrent with s-31708 + s-37420)
Tier-A predecessor: workflow/reasoning/E-12-decisions.md (2026-05-22 ship)
Companion contract test (Tier-A): .elai_cc/crates/contracts/tests/e12_production_codegen_contracts.rs (5/5 PASS)
Companion contract test (Tier-B): .elai_cc/crates/contracts/tests/e12_production_fullstack_codegen_tier_b_contract.rs (shipped this session)

DECISION-1 — E-11 prereq fulfilled INSIDE E-12
----------------------------------------------
Cross-check report flagged E-11 (AuditorMode::ProductionGrade + IdeaFramerSecurityPanel)
as a hard blocker for E-12. Three options were considered:
  (a) Defer E-12 Tier-B entirely until E-11.B ships.
  (b) Stub `AuditorMode::ProductionGrade` as a feature-gated TODO and ship
      E-12 Tier-B against the stub.
  (c) Ship both the ProductionGrade variant AND IdeaFramerSecurityPanel
      AS PART OF E-12 Tier-B, making E-11.B's surface a no-op for these
      two artifacts.

Decision: (c). Rationale:
  - The plan §Tasks #12 explicitly requires extending IdeaFramerSecurityPanel
    from E-11. Shipping a stub now and a real variant in E-11.B would
    duplicate the change.
  - AuditorMode::ProductionGrade is a single enum variant + two arm matches
    + 1 AgentRole variant; trivially additive (no cross-crate breakage).
  - Tier-A guard 1's banned-token list explicitly bans `ProductionGradeV2 /
    EnhancedProductionGrade` etc. — shipping the canonical name now
    preempts a future rename war.

DECISION-2 — Fold framework profiles INTO workflow/archetypes/
--------------------------------------------------------------
Plan §Tasks #1 names `.claude/tools/framework-conventions/` as the manifest
home. The context-package (D9) flagged this as a parallel dual-source to
the E-11.A-shipped `workflow/archetypes/` directory.

Decision: workflow/archetypes/ is canonical; `.claude/tools/framework-conventions/`
is NOT created. The contract test
`workflow_archetypes_is_canonical_no_parallel_framework_conventions`
locks this invariant.

Four representative archetype manifests ship in this commit:
  - nextjs15.toml (SaaSDashboard target)
  - svelte5.toml (WebApp target)
  - fastapi-modern.toml (ResearchTool target)
  - hono.toml (CLITool target)

The remaining 12+ framework manifests (Wasp / Encore / RedwoodSDK / T3
/ NextAppRouter / NextPagesRouter / SveltekitMinimal / RemixV2 / Astro /
HonoCloudflare / FastApiAsync / DjangoNinja / etc.) are deferred to a
follow-on patch-N once paper-ingestor mode=source has source-grounded
the official docs.

DECISION-3 — Wire-dark posture for 15 external CLIs
---------------------------------------------------
gitleaks / trufflehog / semgrep / npm-audit / pip-audit / cargo-audit /
lighthouse-ci / biome / ruff / clippy / sqlite3 / cargo-machete /
ast-grep / lsof — none are installed on the local dev host (F:).

Per memory:wire-dark-posture (5th application after C-49 / C-50 / C-52 /
phase-4J empire + B-16 ORT-PENDING), the runners ship the FULL surface:
  - tokio::process::Command shellout
  - 30s / 60s tokio::time::timeout per runner
  - structured thiserror enum (not Box<dyn Error>, not Result<_, String>)
  - explicit ToolchainMissing branch DISTINCT from Error
  - `// TOOLCHAIN-PENDING-<TOOL>: <install command>` seam marker per runner
  - blake3-prefix hashes for sensitive output (anti-stale HARD RULE: NEVER sha2)

ToolchainMissing is NOT a stub. It is a stable wire variant that the
dashboard renders as a one-line "missing: <tool> — install via X"
chip alongside Ok and Error results. The contract test
`fifteen_production_checklist_runners_exist` verifies every runner
ships the ToolchainMissing branch + the PENDING marker.

RUNPOD-PENDING: install all 15 CLIs on the runpod host (per
memory:project_vram_tier_plan — 2x3090 host on the way), run the
checklist against a representative app (Next.js 15 SaaS or FastAPI
backend), capture findings + render in IdeaFramerSecurityPanel.

DECISION-4 — Anti-slopsquatting deferral via manifest indirection
-----------------------------------------------------------------
Plan §Citations names 16 framework version literals (Next.js 15, React 19,
Svelte 5, Tailwind v4, Biome 2.3, FastAPI 0.115, etc). Tier-A guard 4
`no_hardcoded_unpinned_framework_versions_in_source` BANS source-inline
literals regardless of WebSearch verification status.

Decision: version pins live in `workflow/archetypes/<name>.toml`. The
4 manifests authored in this session carry the version_pin field, but
the pins themselves are CANDIDATES — they require WebSearch
verification against npm view / pypi / GitHub releases / etc. before
they should be trusted for codegen.

Per CLAUDE.md anti-slopsquatting HARD RULE: verify name + repo URL +
version + publish date BEFORE the manifest is consulted by a real codegen
pipeline. This verification is RUNPOD-PENDING.

DECISION-5 — SCHEMA 127 -> 128 atomic, 2-variant cascade (NOT QUINTUPLE)
------------------------------------------------------------------------
Concurrent session 3 (s-31708) was warned in the session prompt about
shipping SCHEMA 126/127. As of edit time, SCHEMA head was 127 with
CalibrationCorpusUpdated as the last variant (D-16 closure). E-12 Tier-B
cascades atomically to SCHEMA_VERSION = 128 with two new variants:

  - SsbActionType::FrameworkProfileSelected
    payload: FrameworkProfileSelectedPayload { profile_name_hash,
             version_pin_hash, language_hash, ts_ms } — all u64
    blake3-prefix.
  - SsbActionType::LegacyConfigStripped
    payload: LegacyConfigStrippedPayload { config_path_hash,
             lint_engine_hash, ts_ms } — all u64 blake3-prefix.

TOTAL_VARIANTS 352 -> 354 (+2). action_type_lock.golden regenerated
deterministically (Python helper); the +2 entries sort alphabetically
to ordinals 0124 (framework_profile_selected, between
fitm_calibration_applied and fsm_transition) and 0172
(legacy_config_stripped, between lateral_thinking_succeeded and
llamacpp_ep_detected).

DECISION-6 — FrameworkAdapter trait MIRRORS ScaffoldApplier
-----------------------------------------------------------
Tier-A guard 1 `no_parallel_framework_adapter_trait` bans
FrameworkAdapterV2 / BespokeFrameworkAdapter / FullstackEmitterV2 /
AdHocScaffoldEmitter. The canonical trait shape is
`crate::task_runner::ScaffoldApplier` at task_runner.rs:2173:

  pub trait ScaffoldApplier: Send + Sync {
      fn apply(&self, template_name: &str, target_dir: &Path, task_text: &str) -> Result<(), String>;
  }

FrameworkAdapter mirrors this signature EXACTLY (sync, three args, same
parameter names) with two additive methods:
  - production_checklist() -> Vec<SecurityCheckId>
  - apply_lint_defaults(&FilePlan) -> Vec<PathBuf>

The contract test `framework_adapter_trait_mirrors_scaffold_applier`
locks this invariant against future drift.

DECISION-7 — PROJECT_CLASS_TO_FRAMEWORK is Vec, not BTreeMap
------------------------------------------------------------
The B-40 `ProjectClass` enum does NOT derive `Ord` (idea_framing.rs:87).
Adding `Ord` would be a cross-plan change outside E-12 scope. Two
alternatives:
  (a) Add `Ord` derive to ProjectClass — touches a B-40 invariant.
  (b) Use Vec<(ProjectClass, &'static str)> in declaration order.

Decision: (b). `PROJECT_CLASS_TO_FRAMEWORK()` returns the mapping in
declaration order via the canonical `framework_for_project_class`
const fn. The contract test `project_class_table_covers_all_9_variants`
locks the 9-entry coverage invariant.

DECISION-8 — Wire-dark Tauri command pair (NOT mock data)
---------------------------------------------------------
Per memory:feedback_verify_semantics — "test passed" without semantic
output is NOT proof. The IdeaFramerSecurityPanel Tauri command pair
(`get_framework_profile` + `run_production_checklist`) returns the
ACTUAL FrameworkProfile (loaded from workflow/archetypes/ on disk) and
the ACTUAL runner results (ToolchainMissing / Ok / Error). No mock
data, no synthetic checklist scores. When the runpod CLIs are missing
the panel surfaces ToolchainMissing badges; this IS the production
truth, not a stub.

DECISION-9 — Skipped scope (Tier-B follow-on)
----------------------------------------------
The plan declares 12 dashboard surface items + an installer extension.
Deferred to future patch-N:
  - install_pre_commit_hook.sh gitleaks/trufflehog injection (Tier-B
    of E-12 Phase 2 — needs hook test fixtures).
  - C-32 SkeletonGenerator SERVER_ENTRY_SEEDS const map (depends on
    FIM markers per D8; deferred until paper-ingestor source-grounding
    on the OTel + Sentry SDKs lands).
  - 5 new FailureCategory variants in core::mistake_book (RlsOff /
    SecretInRepo / LegacyLinterDuplicate / LocalStorageJwt /
    DemoCredsLeft) — depends on the auditor consumer rewiring per D5.
  - LevelAuditVerdict.findings category extension to 15 entries —
    depends on D11 resolution of the open enum vs string-key debate.

These items are explicitly out of scope for the Tier-B session and
have NO orphan code in tree; they remain entirely as planning text.

KNOWN-RESIDUAL
--------------
1. 15 external CLIs absent on dev host → wire-dark posture per D3.
2. 16 framework version literals not anti-slopsquatting verified → D4.
3. paper-ingestor mode=source on Wasp/Encore/Supabase docs not run →
   D2 (only 4 representative manifests ship; remainder deferred).
4. install_pre_commit_hook.sh not extended (deferred per D9).
5. SkeletonGenerator SERVER_ENTRY_SEEDS not added (deferred per D9).
6. core::mistake_book FailureCategory not extended (deferred per D9).
7. LevelAuditVerdict.findings category not extended (deferred per D9).

RUNPOD-PENDING: 1 — 15-tool toolchain install + live production
checklist eval on representative app (Next.js 15 SaaS + FastAPI
backend), per DECISION-3.
TOOLCHAIN-PENDING: 15 — one per external CLI runner; markers grep-able
via `grep -r "TOOLCHAIN-PENDING-"
.elai_cc/crates/orchestrator/src/production_checklist/`.
