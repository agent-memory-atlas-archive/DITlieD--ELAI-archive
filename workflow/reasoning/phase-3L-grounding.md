GROUNDING SWEEP -- phase-3L
Source: F:\.ELAI_workflow\docs\specs\guide\steps\phase-3L.txt
Date: 2026-05-22
FSM: unverified | Session: s-25120

SPEC REFERENCES
.elai_cc/crates/knowledge/src/conventions.rs | MISMATCH | ConventionStats L175 OK, but pub fn extract_conventions(corpus: &CorpusView) is not the real API -- actual: extract_from_dkg(&[DkgNodeInfo], &Path) at L191 plus method extract_conventions at dkg/mod.rs:1062. NamingConventions is re-export from elai_core::project_profile, not native here.
.elai_cc/crates/rules/src/project_rule_seeder.rs | VERIFY_NEEDED | ProjectRuleSeeder L61 OK; real method is seed_rules (not seed as WIRING_CONTRACT claims).
.elai_cc/crates/orchestrator/src/indexing_service.rs | MISMATCH | grep CorpusView|conventions = 0 hits in file; CorpusView type does not exist in repo.
.elai_cc/crates/core/src/telescope.rs | MISMATCH | grep pi_telescope_contract|pi_tier3|T1Hook|T2Hook|T3Hook = 0 hits; tier hooks live in plugins/src/tiered_analysis.rs.
.elai_cc/crates/knowledge/src/project_profile.rs | MISMATCH | file not present; canonical is core/src/project_profile.rs (L418 detect_project_profile, L443 ProjectProfile).
metrics/phase_3L.toml | MISSING | calibration output, expected post-execution.
.claude/tools/elai.map | GROUNDED | 2016 symbol hits for cited identifiers; map fresh.
docs/specs/guide/steps/phase-3K.txt | GROUNDED | exists but is RAG PRODUCTION HARDENING; mislabeled by 3L as "Tier0 prompt scaffold".

CLAIM CLASSIFICATION
extract_conventions free fn in knowledge/conventions.rs | UNGROUNDED | grep shows extract_from_dkg + method extract_conventions in dkg/mod.rs
NamingConventions struct native to knowledge::conventions | UNGROUNDED | re-export from elai_core::project_profile
T1Hook / T2Hook / T3Hook in knowledge/src | UNGROUNDED | actual: plugins/src/tiered_analysis.rs L468/L774/L1924
ProjectIntelligenceHook trait | UNGROUNDED | trait does not exist in repo
CorpusView type | UNGROUNDED | does not exist
pi_telescope_contract test | UNGROUNDED | does not exist
pi_tier3 priority ordering in core/src/telescope.rs | UNGROUNDED | symbol absent
ProjectProfile in knowledge/src/project_profile.rs | MISMATCH | wrong crate; lives in core
prompt_builder.rs in reasoning/src | UNGROUNDED | file does not exist
/profile slash command Profile variant | VERIFY | enum at commands.rs L21; needs scan
knowledge/src/migrations/conventions_*.sql | UNGROUNDED | dir absent; only metadata_migration_v2.sql + v3.sql at src/ root
metrics/calibrate_phase_3L.py | UNGROUNDED | script absent; Step 8 deliverable
p50 <=50ms / p99 <=100ms targets | INGEST | no source; project-internal aspirational
CONVENTIONS_SCHEMA_VERSION | UNGROUNDED | constant not authored
once_cell::sync::Lazy for regex | VERIFY | canonical Rust; current conventions.rs uses raw Regex (Step 1 retrofit)
rules.db schema versioning | VERIFY | rules.db only referenced in interrogation_store.rs
auto-import-rule kind | UNGROUNDED | introduce in Step 5 or collide check needed
4-language unit-test corpus | UNGROUNDED | tests not yet written
200-token cap + top-language truncation | UNGROUNDED | not implemented

ACTIONS REQUIRED
- MISMATCH conventions.rs API signature: spec extract_conventions(&CorpusView) vs real extract_from_dkg(&[DkgNodeInfo], &Path) + dkg method. Resolve before EXECUTING.
- MISMATCH tier-hook location: spec knowledge/src vs real plugins/src/tiered_analysis.rs (T1 L468, T2 L774, T3 L1924, TieredAnalysis L2090). Decide move vs extend.
- MISMATCH project_profile crate: spec knowledge/src/project_profile.rs vs real core/src/project_profile.rs. Fix WIRING_CONTRACT paths.
- MISMATCH indexing_service.rs claim "populates the conventions corpus per file" + CorpusView -- both unverifiable. Either author CorpusView or rebind to DkgNodeInfo.
- MISMATCH core/src/telescope.rs cited content vs reality (assemble_telescope only). Author pi_telescope_contract or rewrite spec.
- UNGROUNDED prompt_builder.rs path -- pick a real target in reasoning/src/ (engine.rs ?) before Step 4 wires CONVENTIONS block.
- UNGROUNDED ProjectIntelligenceHook trait -- decide: author trait, or drop trait abstraction and reuse TieredAnalysis facade.
- UNGROUNDED CONVENTIONS_SCHEMA_VERSION + migrations dir -- author both before Step 2 / Step 5.
- ingest source "plugins::tiered_analysis tier-hook architecture" via paper-ingestor mode=source paths .elai_cc/crates/plugins/src/tiered_analysis.rs. Required to keep T1/T2/T3 design coherent.
- ingest source "core::project_profile" via paper-ingestor mode=source paths .elai_cc/crates/core/src/project_profile.rs. Required for Step 6 aggregator.
- verify ProjectRuleSeeder::seed vs seed_rules with grep "fn seed" rules/src/project_rule_seeder.rs.
- verify Profile variant in cli/commands.rs (single Read of full enum).
- verify auto-import-rule absence with grep "auto-import-rule" .elai_cc before introducing.
- INGEST p50/p99 latency targets -- either cite source or downgrade to internal target.

VALIDATION WARNINGS
heavy spec-reality drift: 5 of 6 cited .elai_cc paths show MISMATCH or unverifiable content; treat phase-3L as PARTIAL with elevated churn risk -- spec rewrite recommended before TESTING_FIRST.
FSM state not verified in this dispatch; executor must call mcp_fsm_get_state.
phase-3K.txt is RAG PRODUCTION HARDENING, not "Tier0 prompt scaffold" -- predecessor label in Reads (mandatory) is wrong.
