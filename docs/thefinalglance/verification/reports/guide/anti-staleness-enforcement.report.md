# Verification Report: anti-staleness-enforcement
Source: F:\.ELAI_workflow\docs\specs\guide\anti-staleness-enforcement.txt
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
The anti-staleness design specifies four layers of defense (pre-generation, during-generation,
post-generation, maintenance). Layers 1 and 3 are substantially implemented: freshness scoring,
VERSION_GATE predicate, T1/T2 hook wiring with deprecated-import checking, the deterministic
auditor, and the dep-audit CLI subcommand are all present. Layer 2 (ETC LogitsProcessor,
freshness steering vector compute script, ContrastPairs struct) is also present. Key gaps are:
the constitutional constraint injection is plumbed but NOT auto-populated from rules.db
(constraints come from TOML override only); modern_api.npz vector artifact is absent; no
periodic automation wires dep-audit or vector refresh to a scheduler; and constrained decoding
for imports (section 2.3) targets generic JSON-schema paths rather than the anti-staleness
import-allowlist described in the spec.

## Mechanisms verified DONE

- FreshnessConfig + exponential recency decay formula (half-life 180d, floor 0.05) —
  evidence: crates/knowledge/src/freshness.rs:51-64 (struct + default), :333 (select_half_life),
  tests at :431-500 verify 30d=89%, 365d=25%, floor clamp
- FreshnessDb SQLite (freshness_meta.db) with upsert_mtime + access_count —
  evidence: crates/knowledge/src/freshness.rs:97-285
- freshness_multiplier wired into pipeline.rs RRF rerank —
  evidence: crates/knowledge/src/pipeline.rs:653 (extract_file_path call + doc comment),
  :1015 (FreshnessConfig/FreshnessDb import), :1061 (with_freshness_db), test
  rrf_rerank_with_freshness_boosts_fresh_files at :1029
- Doc-specific half-life (90d) via select_half_life —
  evidence: crates/knowledge/src/freshness.rs:488-506 tests confirm .md/.txt -> 90d, .rs -> 180d
- deprecated_patterns table schema in rules.db (ecosystem, deprecated_name, replacement_name,
  severity, reason, deprecated_since) —
  evidence: crates/rules/src/schema.rs:747 (seed test), :900-931 (query functions with
  BLOCK/WARN filters); seed data loaded from DEPRECATED_PATTERNS_SEED constant
- VERSION_GATE predicate type in PredicateType enum —
  evidence: crates/rules/src/predicate.rs:158 (VersionGate variant), :236 (from_json),
  :850-942 (round-trip + validation tests)
- VERSION_GATE in rule_engine CompiledPredicate + evaluator —
  evidence: crates/rules/src/rule_engine.rs:123, :320-325, :675 (evaluation), :868-875
- ViolationRecord.replacement_name: Option<String> for VERSION_GATE self-discovery —
  evidence: crates/rules/src/self_discovery.rs:73 (field def); contract tests at
  crates/contracts/tests/rules_self_discovery_contract.rs:422 (Some("blake3") path)
- T1Hook loads rules from rules.db at construction (not empty stub) —
  evidence: crates/plugins/src/tiered_analysis.rs:336-363 (with_rules constructor, info log)
- T2Hook AST deprecated-import extraction via tree-sitter —
  evidence: crates/plugins/src/tiered_analysis.rs:605-651 (with_deprecated_imports, set),
  :782-891 (check_deprecated_ast_imports using import_extractor)
- import_extractor: language-parameterized regex extraction (Rust, Python, TypeScript, Go,
  C#, Ruby) + check_deprecated_imports —
  evidence: crates/plugins/src/import_extractor.rs:81 (tree-sitter dispatch), :403-547 (per-lang
  extract functions), :474+597+606 (check_deprecated_imports tests)
- T2 wired to check deprecated imports against deprecated_patterns in streaming path —
  evidence: crates/orchestrator/src/focus_tunnel/streaming.rs:706 (check_deprecated_imports call)
- ConstraintsBlock with 200-token budget + truncation + "DO NOT USE" format —
  evidence: crates/orchestrator/src/context_budget.rs:1426-1462 (MAX_CONSTRAINT_TOKENS=200,
  from_entries, truncation), :2433 (assert contains "DO NOT USE blake2")
- PhaseSlot.constraints field wired into build_dispatch_context assembler —
  evidence: crates/orchestrator/src/dispatcher.rs:44 (field def), :259-281 (assemble_phase_content
  renders CONSTRAINTS: block), :267-273 (per-constraint bullet)
- Constitutional constraints plumbed into task_runner dispatch —
  evidence: crates/orchestrator/src/task_runner.rs:4149-4157 (constraints loaded from
  detect_project_profile, threaded into PhaseSlot)
- Contract tests confirm constraints block renders in phase content —
  evidence: crates/contracts/tests/fu_constraints_injection_contract.rs:56-85 (rendered block
  test), crates/contracts/tests/fu_build_dispatch_context_phase_slot_contract.rs:77
  (assert contains "DO NOT USE blake2. Use blake3.")
- ETCLogitsProcessor class implemented as HuggingFace LogitsProcessor subclass —
  evidence: python/elai_inference/server.py:952-956 (class def at L956), :1477-1488
  (instantiated in engine), :22003 (SCIP_DEF __call__)
- model.generate() (not manual token loop) drives decode —
  evidence: python/elai_inference/server.py:1376 (docstring states "Uses model.generate()"),
  server.py:7-8 (module docstring confirms LogitsProcessor + model.generate() architecture)
- ETCLogitsProcessor disabled by default (threshold=infinity) —
  evidence: crates/providers/src/sidecar_client.rs:263 (etc_threshold.is_infinite() guard);
  crates/orchestrator/src/self_mod_safety.rs:254 (!is_locked("etc_threshold"))
- calibrate_etc_threshold.py calibration script present —
  evidence: metrics/calibrate_etc_threshold.py:1 (SCIP_DEF, confirmed exists)
- SearchTriggerPayload (ETC -> Rust bridge) implemented —
  evidence: crates/providers/src/sidecar_client.rs:140 (struct def), :304 (returned from
  generate_completions), :355 (last_search_trigger accessor)
- ContrastPairs struct for steering vector batch mode —
  evidence: crates/providers/src/layer_config/steering.rs:872-889 (ContrastPairs struct,
  positive/negative fields)
- compute_steering_vector.py updated for batch mode with equal-length template pairs —
  evidence: metrics/compute_steering_vector.py:7-8 (module docstring), :51-61 (template
  "The recommended {domain} library is {name}", blake3/blake2 pair), :40 (modern_api.npz path)
- modern_api_preference reason string in provider signals —
  evidence: crates/orchestrator/src/provider_signals.rs:321 (reason = "modern_api_preference_vector_unavailable"),
  crates/contracts/tests/provider_signals_contract.rs:287
- DeterministicAuditor implemented (build check + VERSION_GATE eval + import check + lazy patterns) —
  evidence: crates/orchestrator/src/deterministic_auditor.rs:449-1098 (check methods),
  crates/contracts/tests/task_runner_deterministic_auditor_contract.rs:44 (contract test
  with deprecated_patterns field)
- elai dep-audit CLI subcommand implemented (multi-ecosystem registry queries, TOML report) —
  evidence: crates/cli/src/dep_audit.rs:38 (USER_AGENT), :629-790 (dep_audit function),
  :487 (arg dispatch), metrics/dep_audit_2026-04-29.toml (real run artifact exists)
- AntiStalenessDrawer dashboard UI with recompute_steering_vector trigger —
  evidence: crates/dashboard-v2/src/lib/drawers/AntiStalenessDrawer.svelte:8+45,
  crates/dashboard/src/commands.rs (get_constitutional_constraints + recompute_steering_vector
  Tauri commands at SCIP_DEF lines 1377, 1655)

## Mechanisms PARTIAL

- Constitutional spec injection auto-populated from rules.db (section 1.3)
  - exists: crates/orchestrator/src/task_runner.rs:4149-4157 (constraints loaded from
    detect_project_profile, field at crates/core/src/project_profile.rs:66)
  - gap: constraints field on ProjectProfile is populated from TOML override only
    (knowledge/src/project_profile.rs:1117-1126). There is no code path that queries
    deprecated_patterns WHERE severity='BLOCK' and auto-injects them into the PhaseSlot at
    dispatch time. The ConstraintsBlock infrastructure exists but is not wired to rules.db.

- Freshness decay recalculation / pruning (section 4.2)
  - exists: crates/knowledge/src/freshness.rs implements FreshnessDb with upsert_mtime and
    min_decay floor (0.05) applied at read time
  - gap: no periodic batch-recalculation or prune_entries function found in freshness.rs
    or any scheduler. Decay is computed on-demand per-request; there is no offline scan +
    prune below min_decay or log of pruned count.

- Constrained decoding for imports (section 2.3, vLLM/positive allowlist)
  - exists: python/elai_inference/server.py:63 (build_transformers_prefix_allowed_tokens_fn
    import), :1566-1738 (prefix_allowed_tokens_fn construction from JSON schema),
    :117 (_ERR_CONSTRAINED_DECODING_UNAVAILABLE constant)
  - gap: the constrained decoding uses caller-supplied JSON schema (generic C-B tracking),
    not an auto-generated import allowlist from deprecated_patterns. No XGrammar/vLLM
    guided_json path, no import-context detection gate, no EBNF grammar over allowed crates.
    The spec mechanism (auto-generate schema from deprecated_patterns.replacement_name) is absent.

- Freshness steering vector auto-update on deprecated_patterns change (section 4.3)
  - exists: metrics/compute_steering_vector.py (batch mode, pair list, modern_api.npz path),
    dashboard recompute_steering_vector Tauri command, crates/cli/src/sidecar.rs:168
    (precompute_role_steering CLI trigger)
  - gap: modern_api.npz artifact is absent (Glob returned no file). No automated trigger wires
    deprecated_patterns table change to recompute; the dashboard button is manual. The sidecar
    "lazy-load on next request" behavior (section 4.3 step 4) is unverified.

## Mechanisms MISSING

- Nightly / scheduled dep-audit automation (section 4.1 schedule trigger)
  - searched: "nightly.*dep.*audit", "dep.*audit.*cron", "periodic.*dep.*audit",
    "task_scheduler", "4H.*dep.*audit"; no evidence
  - dep-audit runs on-demand only (CLI subcommand). No OS scheduler integration, no Phase 4H
    self-modification loop trigger, no cron wiring found.

- Reasoning architecture integration: Layer 1 fix templates using deprecated_patterns
  (section "REASONING ARCHITECTURE INTEGRATION" point 1)
  - searched: "MISSING_IMPORT.*deprecated", "fix.*template.*deprecated",
    "Layer.*1.*version_gate", "MissingImport.*deprecated"; only bare MISSING_IMPORT taxonomy
    enum found (crates/tools/tests/failure_taxonomy_9cat.rs:91). No code path queries
    deprecated_patterns before generating a fix template for import errors.

## Mechanisms UNVERIFIABLE

- ETC threshold calibration run completed (section 2.1 calibration procedure)
  - reason: calibrate_etc_threshold.py exists and etc_threshold config key is present in
    self_mod_safety; but whether calibration was actually executed and threshold stored in
    elai_cc.toml cannot be determined from static analysis. Default is infinity (disabled).

- ETC + GG Signal 3 integration (section "REASONING ARCHITECTURE INTEGRATION" point 2)
  - reason: GgSignals struct exists (provider_signals_contract.rs:263 entropy_max field),
    and SearchTriggerPayload is returned from generate_completions. Whether the orchestrator
    actually uses ETC trigger data as AB-MCTS GG Signal 3 input cannot be confirmed without
    reading the full AB-MCTS path, which is out of scope.

- Steering vector DoLa stacking guard on <32B models (section 2.1 warning)
  - reason: server.py shows DoLaSteeringProcessor and ETCLogitsProcessor as separate classes.
    The spec mandates they must NOT be stacked on 7B/14B. The guard logic (if any) would
    be in the engine init; not greppable without full method read.

## What must be done (action list)

- [ ] Wire ConstraintsBlock auto-population from rules.db: query deprecated_patterns
  WHERE severity='BLOCK', convert to ConstraintEntry list, call ConstraintsBlock::from_entries,
  populate ProjectProfile.constraints (or inject directly into PhaseSlot at task_runner.rs:4149).
  Target: crates/orchestrator/src/task_runner.rs or crates/knowledge/src/project_profile.rs.
- [ ] Implement periodic dep-audit automation: add Phase 4H trigger or OS scheduler entry
  that calls `elai dep-audit` once per 24 hours and updates deprecated_patterns table.
  Target: crates/cli/src/dep_audit.rs + Phase 4H self-mod loop.
- [ ] Compute and commit modern_api.npz: run metrics/compute_steering_vector.py in batch mode
  to generate models/steering_vectors/modern_api.npz; add auto-trigger when deprecated_patterns
  changes (not just manual dashboard button).
- [ ] Implement pruning for freshness_meta.db: add prune_below_min_decay() method to FreshnessDb
  that removes entries older than floor threshold and logs pruned count. Call from dep-audit
  periodic trigger. Target: crates/knowledge/src/freshness.rs.
- [ ] Implement import-context constrained decoding allowlist (section 2.3): auto-generate
  allowed-crate EBNF or JSON Schema from deprecated_patterns.replacement_name; gate
  prefix_allowed_tokens_fn to import-context tokens only. Target: python/elai_inference/server.py.
- [ ] Implement Layer 1 reasoning fix templates using deprecated_patterns (reasoning-architecture
  integration point 1): when FailureTaxonomy::MissingImport is triggered, query deprecated_patterns
  before generating fix instruction; substitute replacement_name if match found.

## Notes
- Priority 0 prerequisite stated in spec (T1/T2 stubs fix, C-04 model.generate migration) are
  all resolved: T1Hook loads rules, T2Hook does AST extraction, server.py uses model.generate().
- The dep_audit_2026-04-29.toml artifact in metrics/ confirms dep-audit has been run at least
  once manually (on 2026-04-29), satisfying the report-writing requirement.
- The constrained decoding path (section 2.3) is marked vLLM-only in the spec but the
  implementation uses lm-format-enforcer (prefix_allowed_tokens_fn), which is HuggingFace-
  compatible — this is an acceptable alternative but covers a different use case (caller JSON
  schema, not the import allowlist).
