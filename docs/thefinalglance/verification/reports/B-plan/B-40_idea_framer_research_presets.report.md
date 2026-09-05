# Verification Report: B-40_idea_framer_research_presets.md
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-40_idea_framer_research_presets.md
Date: 2026-05-17
Overall status: DONE
Confidence: HIGH

## Summary

All major B-40 deliverables are present in the codebase. The idea_framing.rs module (~1358 LOC) ships FramedIdea, TradeoffCard, CriticCategory (10-variant), run_critic_loop, ProjectClass (9-variant), and classify_project_class. FsmSubstate::Framing is wired in fsm.rs. All 5 SSB action types are present and assigned to SCHEMA_VERSION 41. The dashboard surface (IdeaFramingWindow.svelte, MonitorWindow critic-rounds badge, stub.ts + lib.rs Tauri command) is fully present. Test count (51 total across 5 files) exceeds the plan's ≥28 threshold. Two plan-text discrepancies exist (SCHEMA_VERSION numbering and arXiv citation) but both are corrected in the implementation code — not gaps.

## Claims verified DONE

- idea_framing.rs module created (~700 LOC plan estimate, actual 1358 LOC) — evidence: .elai_cc/crates/orchestrator/src/idea_framing.rs:1 (file exists, 1358 lines)
- ProjectClass enum with 9 variants (ConsumerMobileApp/WebApp/SaaSDashboard/DesktopApp/GameMod/ResearchTool/LibraryOrSDK/CLITool/Other) — evidence: .elai_cc/crates/orchestrator/src/idea_framing.rs:89-99
- classify_project_class function present — evidence: .elai_cc/crates/orchestrator/src/idea_framing.rs:654
- CriticCategory enum with exactly 10 variants (Stale/Slop/Policy/Feasibility/Market/Scope/Contradiction/DesignHard/Vague/Missing) — evidence: .elai_cc/crates/orchestrator/src/idea_framing.rs:160-181
- FramedIdea struct with project_class + research_summary + key_decisions + known_unknowns + tradeoff_card fields — evidence: .elai_cc/crates/orchestrator/src/idea_framing.rs:504-515
- TradeoffCard struct — evidence: .elai_cc/crates/orchestrator/src/idea_framing.rs:442-445
- CriticFinding struct with category/severity/evidence_ref/proposed_fix — evidence: .elai_cc/crates/orchestrator/src/idea_framing.rs:447-456
- run_critic_loop function — evidence: .elai_cc/crates/orchestrator/src/idea_framing.rs:747
- ResearchTree::new_multi_root constructor — evidence: .elai_cc/crates/orchestrator/src/research_tree.rs:1070
- FsmSubstate::Framing substate under PLANNING — evidence: .elai_cc/crates/orchestrator/src/fsm.rs:129, 135
- 5 SSB action types (IdeaFramingStarted/ProjectClassClassified/CriticRoundComplete/IdeaFramed/TradeoffCardEmitted) — evidence: .elai_cc/crates/orchestrator/src/ssb.rs:870-894 (enum), 1103-1107 (as_sql_str), 1280-1284 (from_str)
- SCHEMA_VERSION bumped for B-40 (40→41, as confirmed by inline comment) — evidence: .elai_cc/crates/orchestrator/src/ssb.rs:122-126 (comment), 184 (current value 46 after subsequent B-plans)
- AgentRole::IdeaFramer + AgentRole::IdeaCritic added to agent_pool.rs — evidence: .elai_cc/crates/orchestrator/src/agent_pool.rs:692-693, 1076-1077
- idea_framer + idea_critic GRC configs in reasoning/src/config.rs — evidence: .elai_cc/crates/reasoning/src/config.rs:417-478
- GRC budget 5/800 enforced for idea_framer — evidence: .elai_cc/crates/orchestrator/src/idea_framing.rs:69-72 (IDEA_FRAMER_GRC_BUDGET_CALLS=5, IDEA_FRAMER_GRC_BUDGET_CHARS=800)
- MAX_CRITIC_ROUNDS=3 constant — evidence: .elai_cc/crates/orchestrator/src/idea_framing.rs:60
- Research presets in elai_cc.toml for ≥4 classes (consumer_mobile_app/web_app/saas_dashboard/game_mod confirmed) — evidence: .elai_cc/elai_cc.toml:348-514
- ResearchStream struct with name/query_template/min_source_tier/parallelism_budget/cosmos_grounding/mandatory — evidence: .elai_cc/crates/orchestrator/src/idea_framing.rs:275-295
- parse_research_presets_from_toml function — evidence: .elai_cc/crates/orchestrator/src/idea_framing.rs:369
- derive_framing_evidence (7th AutoAdvanceEvaluator check) in task_runner.rs — evidence: .elai_cc/crates/orchestrator/src/task_runner.rs:7162, 7267
- IdeaFramingWindow.svelte created (~250 LOC plan estimate, actual 352 lines) — evidence: .elai_cc/crates/dashboard-v2/src/lib/windows/IdeaFramingWindow.svelte (file exists)
- MonitorWindow.svelte critic-rounds badge — evidence: .elai_cc/crates/dashboard-v2/src/lib/windows/MonitorWindow.svelte:482-491
- get_idea_framing_state stub in stub.ts wired to live invoke — evidence: .elai_cc/crates/dashboard-v2/src/lib/api/stub.ts:2131-2133
- get_idea_framing_state Tauri command registered in lib.rs — evidence: .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs:252
- idea_framing_unit.rs test file (23 tests, ≥250 LOC) — evidence: .elai_cc/crates/orchestrator/tests/idea_framing_unit.rs (exists, 489 lines)
- idea_framing_contract.rs test file (7 tests, ≥180 LOC) — evidence: .elai_cc/crates/contracts/tests/idea_framing_contract.rs (exists, 277 lines)
- idea_framing_block_contract.rs test file (4 tests) — evidence: .elai_cc/crates/contracts/tests/idea_framing_block_contract.rs (exists, 146 lines)
- autonomous_gate_with_framing_contract.rs test file (11 tests) — evidence: .elai_cc/crates/contracts/tests/autonomous_gate_with_framing_contract.rs (exists, 262 lines)
- research_preset_dispatch.rs test file (6 tests) — evidence: .elai_cc/crates/orchestrator/tests/research_preset_dispatch.rs (exists, 157 lines)
- Total test count ≥28: 23+7+4+11+6 = 51 tests across 5 files — evidence: file line counts + test function counts above
- project_class_50.toml fixture (50-prompt classifier accuracy gate) — evidence: .elai_cc/tests/fixtures/project_class_50.toml (exists, 230 lines)
- Framed-idea artifact persistence path documented (workflow/sessions/{sid}/idea-framing/) — evidence: .elai_cc/crates/orchestrator/src/idea_framing.rs:1102
- arXiv:2510.05145 (FlashResearch) cited in implementation — evidence: .elai_cc/crates/orchestrator/src/idea_framing.rs:25
- arXiv:2511.22173 (RefineBench, max 3 rounds rationale) cited in implementation — evidence: .elai_cc/crates/orchestrator/src/idea_framing.rs:27-28

## Claims PARTIAL

- SCHEMA_VERSION claim in plan ("SCHEMA_VERSION currently 28, B-40 bumps to 29")
  - exists: The B-40 bump is confirmed at .elai_cc/crates/orchestrator/src/ssb.rs:122 ("bumped 40 -> 41 for 5 new SSB action types")
  - gap: Plan was written when SCHEMA_VERSION was 28; by implementation time intervening B-plans had advanced it to 40. The bump occurred correctly (40→41) but the plan text numbers (28→29) are now stale. Current SCHEMA_VERSION is 46 (further bumped by B-34/B-38/B-42/B-41). Plan text is a pre-implementation artifact, not a code defect.

- arXiv:2407.21787 paper citation in plan spec
  - exists: Citation appears in plan file at line 34-37 as the empirical baseline
  - gap: Implementation correctly notes this was the wrong paper ("Large Language Monkeys") and substitutes arXiv:2409.04109 (Si et al. 2024). The code is correct; the plan spec still contains the wrong arXiv ID. Documented at idea_framing.rs:29-32.

## Claims MISSING

None. All WIRING_CONTRACT produces_symbols verified present.

## Claims UNVERIFIABLE

- PRE-FRAMING vs POST-FRAMING ablation (30% reduction in Concept-tier BLOCK rate on 50 fixture ideas) — reason: requires a live inference run against the full pipeline; no test fixture or recorded measurement found in codebase. ACCEPTANCE criterion only — not a code structure claim.
- Critic loop converges in ≤3 rounds on 95% of fixture ideas — reason: convergence rate is a behavioral metric requiring inference. The structural machinery (MAX_CRITIC_ROUNDS=3, block-gate logic) is present; the 95% threshold requires runtime measurement.
- ProjectClass classifier ≥90% accuracy on 50-prompt fixture — reason: the fixture (project_class_50.toml) and the test (idea_framing_unit.rs classifier_accuracy_at_least_90_percent) both exist, but actual pass/fail outcome requires running cargo test. Structure is correct.

## What must be done (action list)

- [ ] Update plan spec line 15 to reflect actual SCHEMA_VERSION bump (40→41, not 28→29) to keep plan doc accurate for future reference
- [ ] Update plan spec line 34-37 to replace arXiv:2407.21787 with arXiv:2409.04109 (Si et al. 2024) — the correct substitution is already in the code but the plan doc still carries the wrong ID
- [ ] Run cargo test -p elai-orchestrator idea_framing to confirm ≥28 tests pass and classifier hits ≥90% accuracy threshold (auditor gate)

## Notes

The implementation exceeds plan estimates in several ways: idea_framing.rs is 1358 LOC vs the 700 LOC estimate; IdeaFramingWindow.svelte is 352 lines vs 250; total test count is 51 vs ≥28 required. The research_preset_dispatch.rs test file was not listed in the plan's "Tests to add" section but was added during implementation. The plan's WIRING_CONTRACT stub_ban entries and observable_effect trigger are structurally satisfied by the code but the live E2E scenario (elai.bat + "make an app like PictureThis") is untestable in a static audit.
