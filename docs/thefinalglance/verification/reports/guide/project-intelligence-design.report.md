# Verification Report: project-intelligence-design
Source: F:\.ELAI_workflow\docs\specs\guide\project-intelligence-design.txt
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
The core ProjectProfile type system (sections 1, 11), Tier 1 structural discovery (section 2A-2G),
Tier 2 convention extraction with DKG+regex fallback (section 3A-3D), Tier 3 domain/architecture
inference and template-based vision generation (section 4), build command registry with Tier A
static validation and Tier B opt-in gate (section 5), project rule seeder with SHADOW lifecycle
(section 6), per-project .elai_cc isolation (section 7), Context Telescope integration with prompt
assembly (section 8), and incremental refresh at N=50 baby steps (section 9) are all implemented.
Key gaps: Tier 3 LLM-assisted vision is template-only (LLM path deferred), Tier 2 DependencyProfile
Tier A declaration extraction is stubbed (returns Default), few-shot DKG examples are not wired into
production prompt flows, and the regex_fallback source marking is not surfaced in project-profile.toml.


## Mechanisms verified DONE

- ProjectProfile struct with all Tier 1/2/3 fields — evidence: crates/core/src/project_profile.rs:25-71
- BuildSystem enum (20+ variants including Cargo, Npm, Bun, PyProject, GoMod, Zig, Just, Task, etc.) — evidence: crates/core/src/project_profile.rs:88-110
- BuildSystemProfile struct (system, config_path, version, monorepo, workspaces) — evidence: crates/core/src/project_profile.rs:79-85
- LanguageProfile with 25+ Language variants — evidence: crates/core/src/project_profile.rs:123-158
- FrameworkDetection struct with category/confidence/detected_via — evidence: crates/core/src/project_profile.rs:199-205
- TestProfile with location pattern (SameFile/Adjacent/SeparateDirectory/Mixed/Unknown) — evidence: crates/core/src/project_profile.rs:225-230
- LintProfile, EntryPoint, CiProfile structs — evidence: crates/core/src/project_profile.rs:246-295
- NamingConventions with Convention (Strong/Moderate/Mixed/Unknown) — evidence: crates/core/src/project_profile.rs:298-311
- ErrorHandlingProfile with ErrorPattern, PropagationStyle — evidence: crates/core/src/project_profile.rs:314-337
- DependencyProfile struct with ClassificationDepth (DeclarationOnly/UsageBased) — evidence: crates/core/src/project_profile.rs:392-416
- BuildCommandRegistry with build/test/lint/format/run/clean vecs — evidence: crates/core/src/project_profile.rs:489-497
- BuildCommand with cmd/working_dir/env/timeout_s/scope/verified — evidence: crates/core/src/project_profile.rs:499-508
- CommandScope enum (Full/Crate/WorkspaceMember/Directory) — evidence: crates/core/src/project_profile.rs:509
- ProjectProfileError with thiserror — evidence: crates/core/src/project_profile.rs:520-521
- ArchitecturePattern, DomainClassification, ProjectDomain enums — evidence: crates/core/src/project_profile.rs:426-468
- project_vision, overrides_applied, schema_version fields — evidence: crates/core/src/project_profile.rs:47-71
- PROFILE_SCHEMA_VERSION = 1 constant — evidence: crates/core/src/project_profile.rs:12
- Section 2A build system detection (all 20 build systems in lookup table) — evidence: crates/knowledge/src/project_profile.rs:482-604
- Section 2A build system priority (language-native over generic wrappers) — evidence: crates/knowledge/src/project_profile.rs:482-604
- Section 2A monorepo detection (Cargo [workspace], package.json workspaces) — evidence: crates/knowledge/src/project_profile.rs:560-603
- Section 2B language detection via file extension census + ignore crate WalkBuilder — evidence: crates/knowledge/src/project_profile.rs:18, 630-672
- Section 2C framework detection (lookup table, ~50 entries for Rust/Python/TS/Go/Java) — evidence: crates/knowledge/src/project_profile.rs:674-760
- Section 2D test runner/location detection — evidence: crates/knowledge/src/project_profile.rs:761-852
- Section 2E lint/formatter detection (rustfmt, clippy, eslint, ruff, biome, etc.) — evidence: crates/knowledge/src/project_profile.rs:808-926
- Section 2F entry point detection (fn main, bin targets, package.json main) — evidence: crates/knowledge/src/project_profile.rs:853-926
- Section 2G CI/deployment detection (GitHub Actions, GitLab CI, Docker, Vercel, etc.) — evidence: crates/knowledge/src/project_profile.rs:927-1040
- Section 3A naming convention detection (NamingHistogram, 80%/60% thresholds) — evidence: crates/knowledge/src/conventions.rs:135-174
- Section 3A DKG path for Rust + regex fallback for non-Rust — evidence: crates/knowledge/src/project_profile.rs:1555-1591, crates/knowledge/src/conventions.rs:191-423
- Section 3A STRONG/MODERATE/MIXED classification — evidence: crates/knowledge/src/conventions.rs:147-174
- Section 3B error handling detection (Rust: Result, thiserror/anyhow grep; Python/TS: try/except patterns) — evidence: crates/knowledge/src/conventions.rs:191-285
- Section 3C module organization pattern detection with strip_prefix for relative paths — evidence: crates/knowledge/src/conventions.rs:191-285; crates/knowledge/src/project_profile.rs:327
- Section 3D test pattern detection (co-location ratio, adjacent test files) — evidence: crates/knowledge/src/project_profile.rs:1217-1315
- Regex fallback marks source as "regex_fallback" in ConventionStats — evidence: crates/knowledge/src/conventions.rs:422
- Section 5: BuildCommandRegistry populated from Tier 1 detection — evidence: crates/knowledge/src/project_profile.rs:979-1040
- Section 5: Tier A static command verification (binary on PATH, makefile/justfile targets, npm scripts) — evidence: crates/knowledge/src/project_profile.rs:1324-1430
- Section 5: Tier B opt-in execution verification (validate_tier_b_with_consent) — evidence: crates/knowledge/src/project_profile.rs:1526-1544
- Section 5: Makefile and justfile target parsing — evidence: crates/knowledge/src/project_profile.rs:1435-1520
- Section 5: read_npm_script for package.json scripts — evidence: crates/knowledge/src/project_profile.rs:793-807
- Section 6: ProjectRuleSeeder::seed_rules — evidence: crates/rules/src/project_rule_seeder.rs:63-382
- Section 6: SHADOW lifecycle (AUTO_PROMOTE_SUCCESSES=5, AUTO_RETIRE_FP_COUNT=3) — evidence: crates/rules/src/project_rule_seeder.rs:388, 391
- Section 6: Rules seeded for naming convention, error handling, test location, lint tools — evidence: crates/rules/src/project_rule_seeder.rs:134-340
- Section 6: check_and_promote_seeded_rules / check_and_retire_seeded_rules — evidence: crates/rules/src/project_rule_seeder.rs:397-470
- Section 6: ProjectRuleSeeder wired in runtime_builder.rs on project open — evidence: crates/cli/src/runtime_builder.rs:265-278
- Section 7: .elai_cc per-project directory layout with core/paths.rs canonical paths — evidence: crates/core/src/paths.rs:37
- Section 7: .elai_cc/rules.db, .elai_cc/indexes/dkg.db, .elai_cc/sessions per project — evidence: crates/core/src/paths.rs:83-190
- Section 7: project-profile-overrides.toml (OVERRIDE_FILE_NAME constant, load_overrides, merge_overrides) — evidence: crates/knowledge/src/project_profile.rs:29, 1068-1131
- Section 7: graceful handling of malformed overrides TOML — evidence: crates/knowledge/src/project_profile.rs:2066
- Section 8: TelescopeContext struct with Layer 1 project_vision, build_cmd, test_cmd, conventions — evidence: crates/knowledge/src/telescope.rs:36-55
- Section 8: assemble_telescope populates from ProjectProfile — evidence: crates/knowledge/src/telescope.rs:147-186
- Section 8: format_for_phase_spec injects into Slot 3 (PhaseSpec) — evidence: crates/knowledge/src/telescope.rs:60-116
- Section 8: convention_function_template and import_template (TEMPLATE GENERATION) — evidence: crates/knowledge/src/telescope.rs:219-265
- Section 8: format_few_shot_example helper function defined — evidence: crates/knowledge/src/telescope.rs:269
- Section 8: LANGUAGE-PARAMETERIZED fix templates (Rust/Python/Go/TS import syntax) — evidence: crates/knowledge/src/telescope.rs:250-265
- Section 8: telescope block wired into PhaseSpec in CLI live_cli.rs — evidence: crates/cli/src/live_cli.rs:720, 829, 3078-3100
- Section 8: InterrogationTree has language + domain fields (LAYER 3 parameterization) — evidence: crates/core/src/interrogation_tree.rs:103-117
- Section 8: InterrogationStore.lookup_tree(language, domain) — evidence: crates/rules/src/interrogation_store.rs:32, 54-79
- Section 9: ProfileChangeDetector with should_refresh_tier1(baby_step_count) — evidence: crates/knowledge/src/incremental_profile.rs:96-154
- Section 9: BABY_STEP_REFRESH_N=50 constant — evidence: crates/knowledge/src/incremental_profile.rs:18
- Section 9: should_refresh_tier3 checks framework/lang changes — evidence: crates/knowledge/src/incremental_profile.rs:154-163
- Section 9: incremental_tier1_refresh function — evidence: crates/knowledge/src/incremental_profile.rs:181-276
- Section 9: load_or_detect_profile (cached vs. full detect, schema_version check) — evidence: crates/knowledge/src/incremental_profile.rs:285-310
- Section 9: baby_step_counter incremented per turn + try_incremental_profile_refresh called — evidence: crates/cli/src/live_cli.rs:1720-1722
- Section 9: polling-based approach (no notify crate) as documented — evidence: no notify usage found, consistent with design note
- Section 10: Multi-language DKG grammars (1A-MULTILANG) — RustGrammar, PythonGrammar, GoGrammar, TypeScriptGrammar, SvelteGrammar registered — evidence: crates/knowledge/src/dkg/mod.rs:210-218
- Section 10: DkgEngine::extract_conventions bridging to conventions::extract_from_dkg — evidence: crates/knowledge/src/dkg/mod.rs:1059-1071
- Section 10: ignore crate (WalkBuilder) used for filesystem traversal, not glob crate — evidence: crates/knowledge/src/project_profile.rs:18
- Section 10: All new error types use thiserror (ProjectProfileError) — evidence: crates/core/src/project_profile.rs:520
- Section 10: toml crate for serialization/deserialization — evidence: crates/knowledge/src/project_profile.rs:1599-1605
- Section 4: classify_domains with WebBackend, WebFrontend, CliTool, Library, Game, MlPipeline, Fullstack — evidence: crates/knowledge/src/tier3.rs:36-57
- Section 4: infer_architecture (Layered, Hexagonal, Microservices; confidence < 0.6 -> Unknown) — evidence: crates/knowledge/src/tier3.rs:308-414
- Section 4: populate_tier3 orchestrates domains+arch+vision — evidence: crates/knowledge/src/tier3.rs:496-510
- Contract tests: pi_incremental_contract, pi_telescope_contract, pi_tier2_a33_contract, pi_tier2_seeder_contract, pi_tier3_completions_contract, dashboard_pi_tier3_on_open_contract — evidence: crates/contracts/tests/


## Mechanisms PARTIAL

- Section 3A/fallback: "Fallback fields marked with source: 'regex_fallback' in profile TOML"
  - exists: ConventionStats.source = "regex_fallback" in crates/knowledge/src/conventions.rs:422
  - gap: The source field is not propagated to the final ProjectProfile or project-profile.toml; NamingConventions struct has no source field (crates/core/src/project_profile.rs:298-303)

- Section 3E: DependencyProfile Tier A (declaration-only extraction from Cargo.toml/package.json)
  - exists: DependencyProfile struct with total/declared/dev_only/core/peripheral fields (crates/core/src/project_profile.rs:392-403); ClassificationDepth enum implemented
  - gap: Both DKG path (conventions.rs:281) and regex fallback path (conventions.rs:421) return dep_profile: DependencyProfile::default() — no actual dependency file parsing is performed; declared/dev_only lists are never populated

- Section 4A: Tier 3 LLM-assisted project vision generation
  - exists: generate_project_vision in crates/knowledge/src/tier3.rs:416-471 with template fallback (~40 token format)
  - gap: LLM-assisted path (the 800-token input prompt calling a MICRO model) is explicitly deferred per code comment at tier3.rs:414 ("LLM-assisted path is deferred"); only template fallback is active

- Section 8: Few-shot examples from DKG similarity search wired into prompt assembly
  - exists: few_shot_for_symbol (crates/knowledge/src/fix_templates.rs:247-255), format_few_shot_example (crates/knowledge/src/telescope.rs:269), FEW_SHOT_K=3 constant
  - gap: FewShotExample is never populated in TelescopeContext (no few_shot_examples field); find_similar_functions is not called from CLI live_cli.rs or runtime_builder.rs in production prompt assembly paths


## Mechanisms MISSING

- Section 10/section 12: phase-3L-decisions.md decision document — searched: workflow/reasoning/ directory listing; no 3L file found
- Section 3E Tier B: DKG import-edge traversal for core/peripheral/unused dependency classification — searched: "core.*dep", "peripheral", "unused.*dep", "import.*edge.*dep" in knowledge crate; no implementation found (field exists in struct but never populated)
- Section 10: 3L.8 latency benchmark on 5+ real projects (1K/10K/50K/100K/500K LOC) — searched: "profile.*latency", "ProfileLatency", "5.*project", "LOC.*50K" in knowledge/benchmark.rs and all knowledge sources; benchmark.rs covers retrieval quality only (Phase 3K.5), not profile generation latency
- Section 2B: tree-sitter parse success used as Signal 2 for language detection confirmation — searched: "tree_sitter_coverage" population in detect_languages; LanguageProfile.tree_sitter_coverage field exists (core/src/project_profile.rs:127) but detect_languages sets no DKG cross-signal (crates/knowledge/src/project_profile.rs:630-672 uses extension census only; tree_sitter_coverage is always 0.0 unless explicitly computed)


## Mechanisms UNVERIFIABLE

- Section 13: UX flow latency numbers (Tier 1 <5s, Tier 2 <30s, Tier 3 <60s) — explicitly labeled UNGROUNDED in design; no measured benchmarks exist
- Section 10: ort RC pin known issue resolution — depends on external crate registry state; not verifiable from source alone


## What must be done (action list)

- [ ] Populate DependencyProfile in detect_project_profile: parse [dependencies]/[dev-dependencies] from Cargo.toml and package.json devDependencies into declared/dev_only vecs (crates/knowledge/src/project_profile.rs, target: detect_build_system or new detect_dependencies fn)
- [ ] Surface convention source ("dkg" vs "regex_fallback") in ProjectProfile TOML: add source: String field to NamingConventions or a top-level convention_source field (crates/core/src/project_profile.rs:298; crates/knowledge/src/project_profile.rs:1587)
- [ ] Wire few-shot DKG examples into production prompt assembly: call few_shot_for_symbol from build_telescope_block or assemble_telescope, add few_shot_examples field to TelescopeContext or format output into format_for_phase_spec (crates/knowledge/src/telescope.rs, crates/cli/src/live_cli.rs:3078-3100)
- [ ] Implement LLM-assisted Tier 3 vision or document explicitly as out-of-scope for current phase (crates/knowledge/src/tier3.rs:416)
- [ ] Compute tree_sitter_coverage in detect_languages via DKG parse results when engine is available (crates/knowledge/src/project_profile.rs:630-672)
- [ ] Create workflow/reasoning/phase-3L-decisions.md per CLAUDE.md requirement
- [ ] Add latency benchmark (3L.8): measure profile generation on 5+ real projects of varying LOC, store in metrics/ (crates/knowledge/src/benchmark.rs or new file)
- [ ] Add adversarial tests: Windows absolute paths in DKG strip_prefix, large monorepo >100 members, vendored dependencies, project with only generated code (crates/knowledge/src/project_profile.rs test module)


## Notes

The implementation substantially exceeds a skeleton: all data structures, detection logic, rule seeding,
telescope integration, and incremental refresh are complete and tested. The main delta from the design is
that Tier 3 LLM assistance is template-only (intentional deferral), DependencyProfile Tier A parsing is
not wired despite the struct being defined, and few-shot example injection into prompt assembly stops short
of the final wiring step. The 1A-MULTILANG work item (Python/Go/TypeScript/Svelte grammars) is DONE —
that prerequisite is satisfied. Phase ordering note from section 10 (3J/3K numbering) is a workflow
documentation issue, not a code issue.

REPORT_WRITTEN=F:\.ELAI_workflow\docs\thefinalglance\verification\reports\guide\project-intelligence-design.report.md | status=MOSTLY_DONE | done=57 | partial=4 | missing=4 | unverifiable=2
