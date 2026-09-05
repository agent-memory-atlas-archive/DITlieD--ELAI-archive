B-7 triple tranche decisions — E-13 + E-12 + E-50

Session: s-22300
Date: 2026-05-31
Author: orchestrate executor (subagent dispatch under .claude/rules/orchestrate-protocol.md)
SCHEMA_VERSION_PUBLIC: 194 -> 195 (single coordinated bump)
TOTAL_VARIANTS: 470 -> 471 (+1 new top-level variant: TddVerificationEvent)
Pattern-2 application: 46th
action_type_lock.golden: +1 row (tdd_verification_event @ ordinal 0417, between tdd_test_passed @ 0416 and tech_stack_decided @ 0418)

Scope

This tranche lands a triple of small, coordinated extensions across three E-plans whose substrates rode the same SCHEMA bump. Per .claude/rules/orchestrate-protocol.md (H-3), no cargo test is executed by the main LLM / executor; the auditor runs all suites.

E-13 SHIPPABLE-NOW

DECISION B7-E13-1. survival_cache as a top-level module under crates/orchestrator/src/survival_cache.rs (NOT a tdd_verifier submodule). Rationale: the cache is the cross-cutting short-circuit substrate referenced by the mutant-hunter agent + the cargo-mutants subprocess wrapper + the dashboard. Hoisting it to the orchestrator top-level keeps `tdd_verifier` focused on the cycle FSM and avoids a circular-import risk (tdd_verifier::mutator -> survival_cache; survival_cache -> ssb::blake3_label_prefix already; no need to thread mutator through). Pure-fn surface + rusqlite Connection ownership.

DECISION B7-E13-2. rusqlite, NOT sqlx. The B-7 exploration corrected the context-package's stale "sqlx already in tree" claim; rusqlite.workspace = true is the canonical local-store dep on orchestrator. Switching to sqlx would have required a new dep -- HARD RULE block.

DECISION B7-E13-3. (mutation_id_hash, ts_ms) composite primary key. Repeated calls within the same millisecond collide with INSERT OR REPLACE -- callers SHOULD pass distinct ts_ms per run. Trade-off: trying to make the column UNIQUE-on-hash alone would require an UPSERT on observed_at, which is more complex for no observable win (we want the row history, not just the latest status).

DECISION B7-E13-4. SurvivorStatus 3-arm closed enum {LiveAndKilled, LiveAndSurvived, Skipped} + snake_case wire labels. The 3-arm shape mirrors the cargo-mutants outcome map without adding the "spawn-pending" tombstone state (we don't need it for the short-circuit path; the dashboard renders absence-of-row as "untested").

DECISION B7-E13-5. ELAI_E13_SURVIVAL_CACHE_ENABLE wire-dark gate. The survival cache's IO surface (in-memory or .elai_cc/cache/survival_cache.db) is gated by env var because the cache writer creates an on-disk SQLite file. Default OFF avoids cluttering ephemeral CI runs; the gate is the operator opt-in.

DECISION B7-E13-6. mutant-hunter agent file at workflow/tooling/mutant-hunter.md (NOT .claude/agents/). Rationale: cargo-mutants 27.x is RUNPOD-PENDING (E-13 DECISION-7), so the agent has no live tool surface yet. Per the CLAUDE.md project-boundary HARD RULE, workflow/tooling/ is the locally-authored documentation path for loop definitions whose runtime substrate is wire-dark. When the toolchain lands, the file migrates to .claude/agents/ and gains a planning-workflow auto-dispatch entry.

E-12 SHIPPABLE-NOW

DECISION B7-E12-1. Wasp + Encore + T3 manifests added to workflow/archetypes/. Plan-side §Tasks #1 had 7-set; Tier-B DECISION-2 collapsed to 4 representative; B-7 adds 3 of the next-highest-priority. RedwoodSDK stays excluded (lowest traffic per context-package; Tier-B already excluded).

DECISION B7-E12-2. version_pin = "UNVERIFIED-PENDING-CONFIRM" on all 3 new manifests. WebFetch on 2026-05-31 returned version tags (v0.23.0 Wasp, v1.57.5 Encore, v7.40.0 create-t3-app) and confirmed each GitHub org (wasp-lang, encoredev, t3-oss), but the release-date metadata returned by the fetched-page summary was flagged STALE by the executor (the model-fetched summary cited 2024 dates that are inconsistent with the project ship cadence vs the 2026-05-31 fetch time). Per the executor brief's "If WebFetch is uncertain on ANY framework: ship manifest with version: UNVERIFIED-PENDING-CONFIRM placeholder and add a TODO[E12-anti-slopsquat] block — DO NOT inline a guessed version", the policy is to surface the uncertainty rather than freeze a guess. Each manifest carries the canonical repo URL + org + a TODO[E12-anti-slopsquat] block pointing at the canonical doc surface for refresh.

DECISION B7-E12-3. SecurityCheckId restriction. All 3 manifests stay within the canonical 15-variant SecurityCheckId set (per crates/orchestrator/src/framework_profile/registry.rs:67-81). No enum extension. Wasp + T3 pick the 9-item SaasDashboard-adjacent set; Encore picks the 8-item DeveloperApi-adjacent set (no Lighthouse on server-only deploys at the default tier).

DECISION B7-E12-4. AssertionOnlyDiffGate DEFERRED for this tranche. Per exploration ("optional this tranche"). The substrate arm (TddVerificationEventKind::AssertionOnlyDiffBlock) ships so the producer can wire in a follow-on tranche without a SCHEMA bump. assertion_diff.rs scanner module + EvidenceGate slot 10 wiring are NOT shipped here; the contract test pins the wire label for the future producer.

E-50 SHIPPABLE-NOW

DECISION B7-E50-1. SaasArchPatternEventKind gains a 12th arm SaasArchetypeSelected (within-variant arm extension). NO new top-level SsbActionType variant; NO additional SCHEMA bump beyond the single E-13 bump 194 -> 195. Matches the E-50 addendum's "single SaasArchPatternEvent" design + closes the master-index "archetype_selected SSB absent" framing.

DECISION B7-E50-2. New env gate ELAI_E50_ARCHETYPE_SELECTED_EMIT_ENABLE_ENV (separate from the parent ELAI_SAAS_ARCH_PATTERNS_ENABLE_ENV) so the new arm can be rolled out independently of the rest of the SaasArchPatternEvent surface. Both gates default OFF; the contract test asserts both.

DECISION B7-E50-3. New ArchetypeSelectionSource 5-arm enum in architecture/recommender.rs covering 3 recommender-confidence arms + UserOverride + Default. confidence_bps is u16 in 0..=10_000 (basis points) to avoid f32 round-trip drift per the E-50 addendum D13 SLO precedent. The recommend_archetype_with_emit pure-fn surface returns (rec, Option<payload>) so the caller composes with the orchestrator's SSB writer (no IO inside the architecture subtree).

DECISION B7-E50-4. Plan-doc vocab refresh DEFERRED to Tranche D. Per exploration: 9 fabricated-vocab lines in docs/thefinalglance/E-plan/E-50_saas_architecture_patterns.md (L1, 10, 18, 20, 47, 67, 74, 117, 125) need hygiene edits replacing B2C/B2B/ApiFirst/AiApp with the canonical 5-variant labels. Tranche D ships this as a single plan-doc edit; the code stays canonical in B-7 (the contract test pins the 5-variant taxonomy as the source of truth).

Anti-slopsquatting outcomes per manifest

  * wasp.toml — REPO + ORG VERIFIED ("wasp-lang" via WebFetch 2026-05-31). VERSION UNVERIFIED (release-date staleness flag). TODO block in manifest.
  * encore.toml — REPO + ORG VERIFIED ("encoredev" via WebFetch 2026-05-31). VERSION UNVERIFIED (release-date staleness flag). TODO block in manifest.
  * t3.toml — REPO + ORG VERIFIED ("t3-oss" via WebFetch 2026-05-31). VERSION UNVERIFIED (release-date staleness flag). TODO block in manifest.

Wire-dark default-off env gates introduced/referenced this tranche

  * ELAI_E13_TDD_VERIFICATION_EMIT_ENABLE — gates the new SsbActionType::TddVerificationEvent emission.
  * ELAI_E13_SURVIVAL_CACHE_ENABLE — gates the survival_cache::SurvivalCache IO surface.
  * ELAI_E50_ARCHETYPE_SELECTED_EMIT_ENABLE — gates recommend_archetype_with_emit returning Some(payload).
  * ELAI_SAAS_ARCH_PATTERNS_ENABLE (pre-existing) — referenced for the default-off posture contract.

All four are asserted absent in test_default_off_posture (e13_e12_tdd_verification_contract.rs).

Files touched (B-7 triple)

NEW
  * F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\survival_cache.rs (~350 LOC incl. 6 tests)
  * F:\.ELAI_workflow\workflow\tooling\mutant-hunter.md (~130 LOC)
  * F:\.ELAI_workflow\workflow\archetypes\wasp.toml (~35 LOC)
  * F:\.ELAI_workflow\workflow\archetypes\encore.toml (~33 LOC)
  * F:\.ELAI_workflow\workflow\archetypes\t3.toml (~37 LOC)
  * F:\.ELAI_workflow\.elai_cc\crates\contracts\tests\e13_e12_tdd_verification_contract.rs (~245 LOC, 11 tests)
  * F:\.ELAI_workflow\workflow\reasoning\B7-triple-tranche-decisions.md (this file)

EDITED
  * F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs (SCHEMA bump 194->195; TddVerificationEvent declared in 8 lockstep sites; SaasArchPatternEventKind +1 arm SaasArchetypeSelected; new env consts; test_payload cascade + all_test_payloads round-trip; must-cover string suffix)
  * F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\lib.rs (pub mod survival_cache)
  * F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\architecture\recommender.rs (+ArchetypeSelectionSource +recommend_archetype_with_emit +build_archetype_selected_payload + 4 unit tests)
  * F:\.ELAI_workflow\.elai_cc\crates\orchestrator\tests\action_type_lock.rs (TOTAL_VARIANTS 470->471; +TddVerificationEvent entries in ALL_VARIANTS + variant test)
  * F:\.ELAI_workflow\.elai_cc\crates\orchestrator\tests\action_type_lock.golden (+1 row at ordinal 0417; ordinals 0418+ shifted by +1; header updated)
  * F:\.ELAI_workflow\.elai_cc\python\elai_inference\schemas\messages.py (SCHEMA_VERSION 194->195)

DEFERRED (NOT shipped in B-7)
  * assertion_diff.rs scanner module (E-12)
  * EvidenceGate AssertionOnlyDiffGate at slot 10 (E-12)
  * cargo-mutants 27.x toolchain pinning + producer wiring on survival_cache (E-13 DECISION-7 RUNPOD-PENDING)
  * mutant-hunter agent migration from workflow/tooling/ to .claude/agents/ (E-13; gated on toolchain landing)
  * E-50 plan-doc vocab refresh (Tranche D)
  * RedwoodSDK manifest (Tier-B exclusion holds)

Audit posture

Per .claude/rules/orchestrate-protocol.md, the auditor agent runs the full cargo check + the new contract test + the existing E-13 / E-50 contract tests after this executor batch returns. The main LLM does NOT run cargo / cargo test / svelte-check (per CLAUDE.md HARD RULE + this brief).

Grounding

  * arXiv:2404.09952 LLMorpheus §2 -- LLM-driven mutation testing motivation; canonical citation for the survival_cache short-circuit semantics.
  * arXiv:2504.16828 ThinkPRM §3 -- process reward model evidence; canonical citation for the "test flipped" verifier the TddVerificationEvent envelope carries.
  * E-13 plan: F:\.ELAI_workflow\docs\thefinalglance\E-plan\E-13_tdd_verification_agents.md
  * E-12 plan: F:\.ELAI_workflow\docs\thefinalglance\E-plan\E-12_production_fullstack_codegen.md
  * E-50 plan: F:\.ELAI_workflow\docs\thefinalglance\E-plan\E-50_saas_architecture_patterns.md
  * B-7 context package: F:\.ELAI_workflow\workflow\sessions\s-22300\context-package-tranche-B7-triple.md
  * B-7 exploration: F:\.ELAI_workflow\workflow\sessions\s-22300\exploration\tranche-B7-triple.md
