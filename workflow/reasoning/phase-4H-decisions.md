B-23 PHASE 4H DECISIONS

ORCHESTRATION
- 7 mega-groups, each runs FSM cycle (TESTING_FIRST -> EXECUTING -> AUDITING -> MEMORIZING).
- Per group: prep wave (context-compiler + integration-explorer + impact-mapper, max 3 parallel) -> main model executes -> auditor (1 cycle) -> memory-writer.
- Commit at end after all groups complete.

GROUP A (4H.1 SKELETON + 4H.2 VERIFY)
What: shared types/enums/structs for Phase 4H Self-Modification
Files modified:
  ADD: .elai_cc/crates/orchestrator/src/self_mod_types.rs (new module, all new types)
  MODIFY: .elai_cc/crates/orchestrator/src/self_mod_layer1a.rs (add Oracle variant to PipelineMode; AllowedConfigKey::Parameter stays as String for now, will refactor in Group B)
  MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs (bump SCHEMA_VERSION 27->28; add 6 new SsbActionType variants per B-23 spec: DataTableGrew, L3Proposal, ConvergenceStagnant, ConfigOscillation, RegressionDetected, EvolutionWithheld; new payload structs; new SsbPayload arms)
  MODIFY: .elai_cc/crates/orchestrator/src/lib.rs (add `pub mod self_mod_types;` + re-exports)
  MODIFY: .elai_cc/crates/contracts/tests/activity_entry_coverage_contract.rs (extend ALL_VARIANTS by 6; assertion >=72 -> >=78)
  MODIFY: .elai_cc/crates/contracts/tests/ssb_4k3_4k4_4k5_contract.rs (EXPECTED_SCHEMA 25 -> 28, skipping intermediate)
  ADD: .elai_cc/crates/contracts/tests/self_mod_types_contract.rs (compile-level contract for new types)

Key decisions:
1. Spec PipelineOutcome (17 fields) name collides with existing 6-field PipelineOutcome in self_mod_layer1a.rs. RESOLUTION: name the new struct `FullPipelineOutcome`. The MVP `PipelineOutcome` stays as-is for Layer 1A backward compat.
2. PipelineMode currently has Surgical+Greenfield. RESOLUTION: add Oracle variant in self_mod_layer1a.rs (adding a variant is additive; no exhaustive matches without `_` arm exist on this enum per integration-map findings).
3. AllowedConfigKey currently has DataTable(DataTable) + Parameter(String). Spec needs 12+ typed Parameter variants. RESOLUTION: defer typed-parameter refactor to Group B (or wherever staged-eval landing requires it). Group A keeps the existing variants and adds a new `parameter_key()` accessor. This is a known decision; if Group D needs typed enums, it will refactor.
4. New SSB action types use spec/B-23 names: data_table_grew, l3_proposal, convergence_stagnant, config_oscillation, regression_detected, evolution_withheld (NOT context-package's suggested alt names).
5. Cross-language risk for ActivityEntry.svelte (only in .bkp_old): activity_entry_coverage_contract.rs scans the archived path. We update ALL_VARIANTS+assertion (mandatory for compile) but do NOT touch the dashboard's active components (no MSI/Proposals window in dashboard-v2/src/lib/windows/ yet — that's Group F).
6. Pre-existing stale EXPECTED_SCHEMA=25 in ssb_4k3_4k4_4k5_contract.rs: bump directly to 28 (atomic), do not intermediate to 27.

Expected test outcome: cargo check -p elai_orchestrator clean; cargo test -p elai_contracts -- self_mod_types_contract passes after types added; cargo check --workspace clean.

Unknowns/risks:
- AllowedConfigKey::Parameter(String) may need to become typed enum in Group D when staged-eval lands the 12 parameter keys. Defer; flag in Group D decision doc.
- SsbActionDetail mapping may need extension for the 6 new variants if SsbViewer enumerates them. Verify in Group A audit.

GROUNDING
- Source: F:\.ELAI_workflow\workflow\papers\dgm-interface.md (parent selection formula, deceptive dip)
- Source: F:\.ELAI_workflow\workflow\papers\ouroboros-interface.md (similarity formula, oscillation, stagnation, regression, evolution gate)
- Source: F:\.ELAI_workflow\workflow\papers\swe-world-interface.md (TestOutcomeReport 4-category)
- Source: F:\.ELAI_workflow\docs\specs\guide\steps\phase-4H.txt:75-362 (4H.1 SKELETON)
- Source: F:\.ELAI_workflow\workflow\sessions\s-37672\context-package.md (full type list)
- Source: F:\.ELAI_workflow\workflow\sessions\s-37672\integration-map-groupA.md (existing types + integration points)
- Source: F:\.ELAI_workflow\workflow\sessions\s-37672\blast-radius.md (collision warnings)

GROUPS B-G PLANNED (preview)
GROUP B: 4H.3 + 4H.4 — config_archive.rs + log-driven proposals + SQLite config_variants table + parent selection algorithm
GROUP C: 4H.3B Layer 1A enhancements — scope enforcement + FPR validation full surface + reasoning_trace_evidence wiring
GROUP D: 4H.5 + 4H.6 — staged eval (10/50/200) + 4-layer safety + AllowedConfigKey typed parameters + Layer 3 stub gate
GROUP E: 4H.7 + 4H.7B/C/D — MSI per-mode (full impl) + OSC + REG + EVO ouroboros ports
GROUP F: 4H.8 — wiring (anti-staleness/reasoning/token-efficiency) + dashboard ProposalsPanel + MsiPanel inside dashboard-v2 windows
GROUP G: 4H.9 — metrics measurement + final cargo workspace test
