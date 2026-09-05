# Verification Report: phase-3F.txt
Source: F:\.ELAI_workflow\docs\specs\guide\steps\phase-3F.txt
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
The rules crate is substantially implemented: all 4 required SQLite tables with CHECK constraints and versioned migration exist, all 5 predicate types plus 2 extension types (GRAPH_REACHABILITY, VERSION_GATE) are defined, the RuleEngine trait is object-safe with correct evaluation ordering, the full 5-state lifecycle machine is implemented with all transition guards, self-discovery fires at occurrences >= 3 with blake3 hashing and compile-time validation, and 263+ static contract tests plus proptest suites with De Morgan/associativity invariants are present. Two gaps remain: (1) criterion benchmarks and phase_3F.toml metrics file are completely absent, and (2) the global rules_seed.db with >= 33 engineering/security/trading ACTIVE SEED rules and post-tool completeness GREP rules (per spec 3F.8) has no evidence of implementation -- only project-convention SHADOW seeds exist.

## Deliverables verified DONE

- 3F.1 SQLite schema: tables rules/rule_metrics/contract_tests/violation_patterns — evidence: crates/rules/src/schema.rs:33-98
- 3F.1 schema_version table and versioned migration (idempotent, currently v6) — evidence: crates/rules/src/schema.rs:21, 342-510
- 3F.1 CHECK constraints on tier (1-3), status (5 states), source (4 values), severity (BLOCK/WARN/INFO) — evidence: crates/rules/src/schema.rs:37-52
- 3F.1 violation_patterns score columns (score_clarity, score_frequency, score_precision) — evidence: crates/rules/src/schema.rs:95-97
- 3F.1 rule_metrics counterfactual columns (causal_catches, spurious_catches) — evidence: crates/rules/src/schema.rs:63-64
- 3F.2 All 5 spec predicate types: AST_PATTERN, GREP_PATTERN, SCHEMA_REQUIRED, METRIC_GATE, COMPOSITE — evidence: crates/rules/src/predicate.rs:101-141 (plus 2 extensions: GraphReachability, VersionGate)
- 3F.2 AstEvaluator trait: evaluate_query + validate_query — evidence: crates/rules/src/predicate.rs:343-351
- 3F.2 VerdictResult enum: Allow/Warn/Block/Escalate(EscalationTarget) — evidence: crates/rules/src/predicate.rs:299-303
- 3F.2 EscalationTarget: Auditor/Human/NextTierRule(String) — evidence: crates/rules/src/predicate.rs:290-293
- 3F.2 Round-trip serde for all predicate types — evidence: crates/rules/src/predicate.rs:468-557 (tests)
- 3F.3 RuleEngine trait object-safe with evaluate/get_active_rules/compile_predicate — evidence: crates/rules/src/rule_engine.rs:263-290
- 3F.3 compile_predicate validates regex, JSON schema, tree-sitter query at insertion time — evidence: crates/rules/src/rule_engine.rs:451, 830
- 3F.3 Evaluation order: METRIC_GATE(0) -> VERSION_GATE/GREP_PATTERN(1) -> AST_PATTERN/SCHEMA_REQUIRED(2) -> COMPOSITE(3) -> GRAPH_REACHABILITY(4) — evidence: crates/rules/src/rule_engine.rs:321-331
- 3F.3 Short-circuit on first BLOCK verdict; WARNs accumulated before BLOCK — evidence: crates/rules/src/rule_engine.rs:401-416
- 3F.4 Lifecycle states PROPOSED/CANDIDATE/SHADOW/ACTIVE/RETIRED — evidence: crates/rules/src/lifecycle.rs:57-73
- 3F.4 PROPOSED->CANDIDATE: requires >= 3 contract tests AND all pass — evidence: crates/rules/src/lifecycle.rs:395, with constant MIN_CONTRACT_TESTS
- 3F.4 CANDIDATE->SHADOW: FindingSourceAuditor rejected (circular trust) — evidence: crates/rules/src/lifecycle.rs:432-455
- 3F.4 SHADOW->ACTIVE guards: FPR < 5%, TPR > 80%, CCR > 50%, no safety regression — evidence: crates/rules/src/lifecycle.rs:23-31
- 3F.4 mutable=0 rules reject auto-retirement — evidence: crates/rules/src/lifecycle.rs:686 (test)
- 3F.4 Division-by-zero safe (CCR returns None when denominator=0) — evidence: crates/rules/src/lifecycle.rs:374-376
- 3F.4 Transition log written to rule_transitions for every successful transition — evidence: crates/rules/src/lifecycle.rs:186-209
- 3F.5 Rust interpreter for all 5+ predicate types in DefaultRuleEngine — evidence: crates/rules/src/rule_engine.rs:467-720
- 3F.5 COMPOSITE depth limited to MAX_COMPOSITE_DEPTH=8 — evidence: crates/rules/src/predicate.rs:93, 187-189
- 3F.5 ESCALATE routing: Auditor/Human/NextTierRule — evidence: crates/rules/src/rule_engine.rs:697-702
- 3F.6 phase_mask bitmask filtering in get_active_rules — evidence: crates/rules/src/rule_engine.rs:425-451
- 3F.6 Domain detection from file paths; engineering/security always included — evidence: crates/rules/src/rule_engine.rs:187, 275-276
- 3F.6 Case-insensitive domain detection test — evidence: crates/rules/src/rule_engine.rs:1202 (test)
- 3F.6 phase_mask=0 rule never returned by get_active_rules — evidence: crates/rules/src/rule_engine.rs:1271 (test pattern)
- 3F.7 blake3 pattern hashing (not blake2s as spec says) — evidence: crates/rules/src/self_discovery.rs:10, 225-227
- 3F.7 Occurrence counting; auto-proposal at DEFAULT_PROPOSAL_THRESHOLD=3 — evidence: crates/rules/src/self_discovery.rs:25, 403
- 3F.7 compile_predicate validation before insertion; invalid predicates logged and discarded — evidence: crates/rules/src/self_discovery.rs:447-504
- 3F.7 Auto-generated contract tests (BLOCK type) from violation examples — evidence: crates/rules/src/self_discovery.rs:447-504, contract_tests.rs:3362-3381
- 3F.7 Trust boundary: PROPOSED rules from source=AUDITOR require independent approval — evidence: crates/rules/src/self_discovery.rs:5, lifecycle.rs:432-455
- 3F.7 Violation pattern scoring: 3-axis composite (clarity/frequency/precision) — evidence: crates/rules/src/self_discovery.rs:238-283
- 3F.7 Normalization idempotent for whitespace/comment differences — evidence: crates/rules/src/self_discovery.rs:882-922 (tests)
- 3F.8 Seed TOML files present in crates/rules/seed/ — evidence: crates/rules/seed/{app_store,design_hard,risk_levels,slopsquatting,source_tiers}.toml (92 rule_id entries total)
- 3F.9 Static contract test suite: 263+ tests documented — evidence: crates/rules/src/contract_tests.rs:1, 2988
- 3F.9 proptest suite with De Morgan, associativity, double negation, depth coverage — evidence: crates/rules/src/contract_tests.rs:3141-3230
- 3F.9 proptest configured for 256 cases per property — evidence: crates/rules/src/contract_tests.rs:3142, 3231
- 3F.9 arb_composite uses prop_recursive at MAX_COMPOSITE_DEPTH=8 — evidence: crates/rules/src/contract_tests.rs:3117-3118
- 3F.9 Contract test READ-LOCK enforced during self-modification — evidence: crates/rules/src/contract_tests.rs:3341-3389
- proptest crate in dev-dependencies — evidence: crates/rules/Cargo.toml:22

## Deliverables PARTIAL

- 3F.8 Seed rules with status=ACTIVE and source=SEED (spec requirement)
  - exists: crates/rules/src/project_rule_seeder.rs:68 (seed_rules fn), seed/*.toml files
  - gap: project_rule_seeder inserts seeds with status=SHADOW (not ACTIVE per spec 3F.8). Seed files in seed/ directory cover app_store/design_hard/risk_levels/slopsquatting/source_tiers domains but no evidence of a global rules_seed.db loader that inserts with status=ACTIVE. Spec requires "Seed rules enter as status=ACTIVE, source=SEED, skip lifecycle."

- 3F.8 >= 33 seed rules including engineering/security/trading and post-tool completeness GREP rules
  - exists: crates/rules/seed/ has 92 rule_id entries across 5 domain TOMLs
  - gap: No engineering domain seed file, no security domain seed file, and no post-tool completeness GREP rules (web_fetch img/table tag patterns, file_read TODO/FIXME) found anywhere in rules/. Seed domains are app-store/design/risk/slopsquatting/source-tiers, not the engineering+security+trading combination required. The 3 PostToolUse GREP rules (Concern 3 Mechanism 2 from spec) are absent.

- 3F.8 Merge logic: global seeds + project-specific rules; mutable=0 cannot be overridden
  - exists: schema supports mutable column with CHECK; project_rule_seeder.rs:102 skips existing rule_ids
  - gap: No global-to-project merge function found. The idempotent skip in project_rule_seeder only prevents duplicate inserts of the same rule_id; it does not implement the spec's "project rules override by rule_id" merge with mutable=0 protection.

- 3F.6 Domain detection configurable via elai_cc.toml [rules.domain_mapping]
  - exists: crates/rules/src/rule_engine.rs:211-212 (detect_domains function hardcoded)
  - gap: No [rules.domain_mapping] configuration key in elai_cc.toml found; domain detection is entirely hardcoded in rule_engine.rs with no config layer.

- 3F.7 Spec says blake2s for pattern hashing; implementation uses blake3
  - exists: crates/rules/src/self_discovery.rs:10, 226-227 (blake3::hash)
  - gap: Spec 3F.7 says "Pattern hashing: blake2s of normalized pattern text." Implementation uses blake3. The CLAUDE.md hard rule also mandates blake3 over blake2 -- this is a spec/CLAUDE.md conflict. blake3 is correct per CLAUDE.md but diverges from spec text.

## Deliverables MISSING

- 3F.10 criterion benchmarks for rule evaluation latency — searched: benches/ directory, criterion in Cargo.toml, eval_latency; crates/rules/benches/ does not exist, criterion not in Cargo.toml
- 3F.10 Benchmark targets: GREP < 1ms, METRIC < 0.1ms, 10-rule mixed < 10ms — searched: benches/, criterion, eval_latency; no benchmark harness found
- 3F.10 Metrics file D:/.elai_cc/metrics/phase_3F.toml — searched: metrics/ directory listing; phase_3F.toml absent (metrics/ has phase_1A..4M but not 3F)

## Deliverables UNVERIFIABLE

- 3F.9 Shrunk counter-examples auto-inserted into contract_tests table as source=GENERATED from proptest failures — reason: proptest auto-insertion of shrunk counter-examples requires a custom proptest reporter hook; the contract_tests.rs shows source=GENERATED insertion in self_discovery context but proptest failure auto-insert is not verifiable without running the suite
- 3F.9 Total proptest lines ~1400 across 7 domains — reason: spec claims ~200 per domain but domain breakdown in contract_tests.rs covers proptest composite/invariant tests without explicit per-domain partitioning; line count not verified without running wc -l
- 3F.8 Seed rule count >= 33 with correct status=ACTIVE — reason: seed/*.toml files have 92 rule_id entries but no loader that inserts them with status=ACTIVE was found; the project_rule_seeder inserts SHADOW seeds only; whether a separate global seeder exists at runtime is not verifiable from static analysis alone

## What must be done (action list)

- [ ] 3F.8: Create a global seed loader (e.g. `load_global_seeds(conn: &Connection)`) that reads from seed/*.toml and inserts rules with status=ACTIVE, source=SEED into the rules table, skipping if already present
- [ ] 3F.8: Add engineering domain seed TOML (e.g. seed/engineering.toml) with >= 10 rules covering no-.unwrap(), tracing, etc.
- [ ] 3F.8: Add security domain seed TOML (e.g. seed/security.toml) with mutable=0 rules
- [ ] 3F.8: Add 3 post-tool completeness GREP rules to engineering seed: web_fetch img-tag WARN, web_fetch table-tag WARN, file_read TODO/FIXME INFO (per spec 3F.8 and technical-analysis-v3.txt Concern 3 Mechanism 2)
- [ ] 3F.8: Implement merge logic protecting mutable=0 global seed rules from project-level override (project rule with same rule_id + mutable=1 must be rejected if global seed has mutable=0)
- [ ] 3F.6: Add [rules.domain_mapping] config key to elai_cc.toml and wire into detect_domains function
- [ ] 3F.10: Add criterion to crates/rules/Cargo.toml dev-dependencies
- [ ] 3F.10: Create crates/rules/benches/eval_latency.rs with 4 benchmark groups: single GREP on 100-line file, single AST (mock) on 100-line file, single METRIC, 10 mixed rules on 100-line file
- [ ] 3F.10: Create D:/.elai_cc/metrics/phase_3F.toml with benchmark results, contract test pass rate, seed rule count, lifecycle transition count

## Notes

- The predicate type count is 7 (not 5 as spec states): spec defines AST_PATTERN/GREP_PATTERN/SCHEMA_REQUIRED/METRIC_GATE/COMPOSITE; implementation adds GRAPH_REACHABILITY and VERSION_GATE. This is an upward extension, not a gap.
- Schema severity CHECK allows BLOCK/WARN/INFO but not ESCALATE -- ESCALATE is handled as a severity string in rule rows (crates/rules/src/rule_engine.rs:697) and routed to VerdictResult::Escalate. The schema CHECK constraint allows ESCALATE as severity per rule_engine test at line 1641 (make_rule with severity="ESCALATE"). Verify schema is updated to include ESCALATE in CHECK if that test passes -- the DDL at schema.rs:45 shows only BLOCK/WARN/INFO which would make ESCALATE severity rules fail CHECK. This is a latent bug.
- The spec says blake2s for hashing; CLAUDE.md mandates blake3. Implementation correctly uses blake3. Spec text is stale.
- proptest dev-dependency is present in Cargo.toml; criterion is absent.
REPORT_WRITTEN=F:\.ELAI_workflow\docs\thefinalglance\verification\reports\steps\phase-3F.report.md | status=MOSTLY_DONE | done=44 | partial=5 | missing=3 | unverifiable=3
