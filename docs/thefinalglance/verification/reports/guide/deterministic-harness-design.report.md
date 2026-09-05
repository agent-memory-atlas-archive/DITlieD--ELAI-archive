# Verification Report: deterministic-harness-design
Source: F:\.ELAI_workflow\docs\specs\guide\deterministic-harness-design.txt
Date: 2026-05-17
Overall status: PARTIAL
Confidence: HIGH

## Summary
The design proposes a three-tier deterministic rule enforcement harness (Tier 1 regex/grep, Tier 2 AST, Tier 3 semantic-only in prompt) replacing .md-based rules, with LINTER_GATE and CONVENTION_GATE predicate types, a domain rule registry, a .md-to-SQLite migration CLI, and context budget reduction for the Rules slot. The codebase has implemented most of the foundational infrastructure (rules engine with 7 predicates, PostToolUse tiered analysis, ProjectRuleSeeder seeding CONVENTION-equivalent rules from ProjectProfile, DeterministicAuditor with linter dispatch, TelescopeContext/Layer 1 context assembly) but the two named predicate types (LINTER_GATE, CONVENTION_GATE) are absent, the context budget rules slot is not reduced to the target token counts, the domain rule registry and its 5 built-in packs are not implemented, and the CLI migration tool (elai rules import) does not exist.

## Mechanisms verified DONE

- Verdict enum (ALLOW/WARN/BLOCK/ESCALATE) — evidence: crates/rules/src/predicate.rs:101 (VerdictResult enum with Block/Warn/Allow/Escalate variants)
- GREP_PATTERN predicate type — evidence: crates/rules/src/predicate.rs:109 (GrepPattern variant); crates/rules/src/self_discovery.rs:55 (PREDICATE_GREP_PATTERN const)
- AST_PATTERN predicate type — evidence: crates/rules/src/predicate.rs:102 (AstPattern variant with tree-sitter query support)
- VERSION_GATE predicate type — evidence: crates/rules/src/predicate.rs:158 (VersionGate variant); dashboard mock-data.ts:466 (VERSION_GATE string in mock rules)
- GRAPH_REACHABILITY predicate type — evidence: crates/rules/src/predicate.rs:144 (GraphReachability variant)
- METRIC_GATE predicate type — evidence: crates/rules/src/predicate.rs:126 (MetricGate variant)
- rules.db (SQLite-stored rules with lifecycle) — evidence: crates/rules/src/schema.rs:35 (CREATE TABLE with status IN PROPOSED/CANDIDATE/SHADOW/ACTIVE/RETIRED)
- Phase 3F rules engine — evidence: crates/rules/src/rule_engine.rs:90 (CompiledPredicate enum); crates/rules/src/predicate.rs:1-4 (7 predicate variants)
- SHADOW lifecycle state for seeded rules (design called it CANDIDATE) — evidence: crates/rules/src/project_rule_seeder.rs:35 (SEEDED_STATUS = "SHADOW"); lifecycle.rs:1 (PROPOSED->CANDIDATE->SHADOW->ACTIVE->RETIRED machine)
- Auto-promotion lifecycle (SHADOW -> ACTIVE after metrics thresholds) — evidence: crates/rules/src/project_rule_seeder.rs:393 (check_and_promote_seeded_rules); crates/rules/src/lifecycle.rs:23-29 (promotion thresholds)
- ProjectProfile struct (language, build_system, conventions, frameworks, domains) — evidence: crates/core/src/project_profile.rs:31,48 (build_system, architecture fields); crates/knowledge/src/project_profile.rs:420 (detect function)
- ProjectRuleSeeder: convention-derived rule auto-seeding from ProjectProfile — evidence: crates/rules/src/project_rule_seeder.rs:63-380 (naming_rules, error_handling_rules, test_location_rules, lint_rules all implemented)
- Lint tool seed rules (clippy, ruff, eslint) via METRIC_GATE — evidence: crates/rules/src/project_rule_seeder.rs:305-379 (lint_rules() with clippy/ruff/eslint cases)
- ProjectRuleSeeder wired into runtime build — evidence: crates/cli/src/runtime_builder.rs:266 (ProjectRuleSeeder::seed_rules called at startup)
- Tiered analysis PostToolUse hooks (T1/T2/T3) — evidence: crates/plugins/src/tiered_analysis.rs:1-8 (module doc: T1 <1ms, T2 <10ms, T3 <5ms); crates/orchestrator/src/hook_dispatcher.rs:393-404 (InProcessHookDispatcher drives TieredAnalysis)
- T1 predicate evaluation from rules.db on SSB write — evidence: crates/plugins/src/tiered_analysis.rs:4 ("T1 (RULES, <1ms): Predicate evaluation from rules.db on every SSB write")
- T2 AST parse (tree-sitter) for file_write actions — evidence: crates/plugins/src/tiered_analysis.rs:5 ("T2 (AST, <10ms): tree-sitter parse for file_write actions only")
- DeterministicAuditGate (FSM gate blocking AUDITING->MEMORIZING on Block findings) — evidence: crates/orchestrator/src/deterministic_audit_gate.rs:62-74 (DeterministicAuditGate trait + FSM integration)
- DeterministicAuditor with linter dispatch (run_all, per-language build/lint cmds) — evidence: crates/orchestrator/src/deterministic_auditor.rs:100-178 (DeterministicAuditorConfig with lint_cmds; default_lint_cmd for 7 languages)
- Ruff PostToolUse runner wired in InProcessHookDispatcher — evidence: crates/orchestrator/src/hook_dispatcher.rs:407-412,595-634 (ruff runner fires on .py/.pyi file writes)
- Deterministic-first auditor mode (linters before LLM) — evidence: crates/orchestrator/src/deterministic_auditor.rs:101 (enabled: true; runs before LLM audit); deterministic_audit_gate.rs:91-133 (run_all dispatched at AUDITING FSM transition)
- TelescopeContext / Layer 1 project vision assembly — evidence: crates/knowledge/src/telescope.rs:36-55 (TelescopeContext struct); assemble_telescope():147 (builds from ProjectProfile)
- load_or_detect_profile (incremental ProjectProfile detection) — evidence: crates/knowledge/src/incremental_profile.rs:285 (load_or_detect_profile function)
- Context budget slots (MICRO/LARGE rules token targets) — evidence: crates/orchestrator/src/context_budget.rs:22-31 (MICRO_SLOT_RULES=150, LARGE_SLOT_RULES=500)
- SelfModBridge / SelfModOrchestrator (self-discovery proposes new rules from violations) — evidence: crates/orchestrator/src/self_mod_bridge.rs; crates/rules/src/self_discovery.rs:469-813

## Mechanisms PARTIAL

- Linter rules as seed rules (design: LINTER_GATE predicate; actual: METRIC_GATE wrapping metric_name like "clippy_warnings")
  - exists: crates/rules/src/project_rule_seeder.rs:309-379 (lint_rules seeds clippy/ruff/eslint as METRIC_GATE)
  - gap: LINTER_GATE is not a real predicate type — actual enforcement delegates to METRIC_GATE on a named metric ("clippy_warnings", "ruff_violations"). There is no linter subprocess spawn or cached JSON parse inside the predicate engine itself. The design's linter_cmd/output_format/cache_enabled fields do not exist.

- Convention-derived rules (design: CONVENTION_GATE predicate type; actual: GREP_PATTERN predicates seeded from ProjectProfile.naming)
  - exists: crates/rules/src/project_rule_seeder.rs:76-88 (naming_rules, error_handling_rules, test_location_rules seeded as GREP_PATTERN); crates/rules/src/predicate.rs:109 (GrepPattern variant)
  - gap: CONVENTION_GATE is not a predicate type. Convention enforcement uses GREP_PATTERN (and METRIC_GATE for linters). No dedicated CONVENTION_GATE variant exists.

- Context budget Slot 2 (Rules) reduction (design: MICRO=50-80 tokens; actual: MICRO_SLOT_RULES=150)
  - exists: crates/orchestrator/src/context_budget.rs:22 (MICRO_SLOT_RULES = 150)
  - gap: Design targets 50-80 tokens (MICRO) and 200 tokens (LARGE) for Tier 3-only rules. Actual values are 150 (MICRO) and 500 (LARGE). No Tier-3-only filtering of the rules slot content was found.

- Seeded rules lifecycle state (design says CANDIDATE initial state; actual: SHADOW)
  - exists: crates/rules/src/project_rule_seeder.rs:35 (SEEDED_STATUS = "SHADOW"); schema.rs:39 (CANDIDATE state exists in DB schema)
  - gap: Design specifies lifecycle state CANDIDATE by default for auto-seeded rules; code inserts them as SHADOW. CANDIDATE state exists in the schema but is not used by the seeder.

## Mechanisms MISSING

- LINTER_GATE predicate type — searched: LINTER_GATE, LinterGate, linter_gate, linter_cmd in elai.map and crates/rules/src/; no evidence
- CONVENTION_GATE predicate type — searched: CONVENTION_GATE, ConventionGate, convention_gate in elai.map and crates/rules/; no evidence
- lint_cache.db (separate SQLite cache for linter results keyed by content hash) — searched: lint_cache, LintCache, lintcache in elai.map and full .elai_cc/; no evidence
- Incremental linting (single-file re-lint with JSON output parsing per PostToolUse) — searched: linter subprocess spawn with --message-format=json in predicate/hook path; ruff runner exists for Python but no clippy/eslint/golangci-lint JSON parse path in predicate engine
- domain_rule_packs table (SQLite table with pack_id, domain, version, rules_count) — searched: domain_rule_packs, DomainRulePack in elai.map and .elai_cc/; no evidence
- 5 built-in domain rule packs (trading.rules.db, gamedev.rules.db, security.rules.db, ml.rules.db, web.rules.db) — searched: trading.rules, gamedev.rules, ml.rules, security.rules, web.rules; no evidence
- CLI command: elai rules import (md-to-SQLite migration tool) — searched: "elai rules import", md_to_sqlite, RulesMigration, rules import in elai.map and .elai_cc/; no evidence
- CLI command: elai rules install (domain pack installer) — searched: "rules install", "install rules", pack install in .elai_cc/; no evidence
- Project-level linter config discovery (use project's .eslintrc.js / clippy.toml for rules) — searched in deterministic_auditor.rs; config is derived from ProjectProfile but no config-file discovery path confirmed beyond profile.commands

## Mechanisms UNVERIFIABLE

- Phase 4H (self-modification loop) wired to linter output for new rule proposals — reason: self_mod_bridge and self_discovery exist (crates/orchestrator/src/self_mod_bridge.rs, crates/rules/src/self_discovery.rs) but tracing the full pipeline from linter JSON output -> self_mod proposal requires runtime execution; static analysis inconclusive on whether linter finding categories trigger self-discovery vs only build/test failure taxonomy categories.

## What must be done (action list)

- [ ] Add LINTER_GATE variant to Predicate enum in crates/rules/src/predicate.rs with fields: linter_cmd, output_format, cache_enabled
- [ ] Add CONVENTION_GATE variant to Predicate enum in crates/rules/src/predicate.rs with fields: convention_type, expected_pattern
- [ ] Create lint_cache.db schema and evaluation logic (blake3-keyed per file, separate SQLite from rules.db)
- [ ] Implement incremental linter subprocess dispatch in tiered analysis or predicate evaluator (cargo clippy --message-format=json, ruff --output-format=json, eslint --format=json per file)
- [ ] Create domain_rule_packs table in rules.db schema (crates/rules/src/schema.rs)
- [ ] Implement 5 built-in domain rule packs: trading, gamedev, security, ml, web (as SQLite or TOML files loaded at startup)
- [ ] Implement CLI command: elai rules import <path> (parse .md numbered rules, classify tiers, generate predicates, insert into rules.db)
- [ ] Implement CLI command: elai rules install <pack_name> (load domain pack into rules.db)
- [ ] Reduce MICRO_SLOT_RULES from 150 to target 50-80 tokens (crates/orchestrator/src/context_budget.rs:22) and add Tier-3-only filtering logic
- [ ] Reduce LARGE_SLOT_RULES from 500 to target 200 tokens (crates/orchestrator/src/context_budget.rs:31)
- [ ] Change SEEDED_STATUS in project_rule_seeder.rs from "SHADOW" to "CANDIDATE" (or document the intentional deviation)

## Notes
The core enforcement infrastructure is well-implemented: the rules engine (7 predicate types), PostToolUse tiered hooks, ProjectRuleSeeder seeding convention-based rules, DeterministicAuditor with per-language linter dispatch, DeterministicAuditGate gating FSM transitions, and TelescopeContext (Layer 1 project vision) are all present and wired. The main gaps are the two named predicate types (LINTER_GATE, CONVENTION_GATE), the lint result cache, the domain rule registry infrastructure, and the .md-to-SQLite migration CLI — which are the Phase 3L-specific deliverables that the design flags as new work requiring 800-1200 LOC. The context budget slot token targets have also not been reduced to the design's specified values.
