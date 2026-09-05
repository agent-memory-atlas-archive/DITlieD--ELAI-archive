# Verification Report: B-34_entry_point_detection.md
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-34_entry_point_detection.md
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
The four-heuristic entry-point detection algorithm is fully implemented in the knowledge crate with all required types, helper functions, and unit tests. The orchestrator dispatcher tool is wired through production_agent_rpc.rs (not dispatcher.rs as the plan specified), SSB logging is operational, and the Designer briefing injection via build_existing_project_context exists. Two items are missing: (1) the /research Oracle pre-seeding with PROJECT_STARTING_POINTS is absent from the CLI research handler, and (2) the performance and payload-selectivity test files were not created.

## Claims verified DONE

- detect_entry_points implemented (all four heuristics: Orphan, WellKnownName, FrameworkRoute, Test) — evidence: crates/knowledge/src/symbol_graph/entry_points/detect.rs:142-266
- find_symbol_at returns deepest enclosing symbol — evidence: crates/knowledge/src/symbol_graph/entry_points/detect.rs:104-124
- ENTRY_POINT_NAMES table covers 14 languages with Python __main__ special-case — evidence: crates/knowledge/src/symbol_graph/entry_points/names.rs:16-53
- Deduplication keyed on (id, reason_key) via BTreeSet — evidence: crates/knowledge/src/symbol_graph/entry_points/detect.rs:147-157
- Types: DetectedEntryPoint and EntryPointReason enum with Orphan/WellKnownName/FrameworkRoute/Test — evidence: crates/knowledge/src/symbol_graph/entry_points/types.rs:16-62
- HTTP verbs (get/post/put/delete/patch/head/options/all/use) + route mapped to framework reason strings — evidence: crates/knowledge/src/symbol_graph/entry_points/detect.rs:30-61
- Test-function name patterns ^test[_A-Z] and ^Test[A-Z_] and "it"/"describe" matched char-by-char without regex crate — evidence: crates/knowledge/src/symbol_graph/entry_points/detect.rs:72-92
- Dispatcher tool TOOL_ENTRY_POINTS registered and callable from IPC — evidence: crates/orchestrator/src/production_agent_rpc.rs:1110 (TOOL_ENTRY_POINTS arm) + crates/ipc-types/src/lib.rs:627 (EntryPointsRequest)
- EntryPointsDetected SSB action variant emitted per tool call — evidence: crates/orchestrator/src/ssb.rs:899, crates/orchestrator/src/production_agent_rpc.rs:1228
- Designer briefing auto-injection (EXISTING_ENTRY_POINTS block) when project_dir is present — evidence: crates/orchestrator/src/task_runner.rs:4342-4354 + build_existing_project_context at line 8594
- /entry_points REPL slash command implemented in live_cli.rs — evidence: crates/cli/src/live_cli.rs:2394-2461
- Dashboard list_entry_points Tauri command (dashboard-v2) — evidence: crates/dashboard-v2/src-tauri/src/commands.rs:6429 + stub.ts:2251
- Contract test: entry_points_dispatcher_contract.rs — evidence: crates/contracts/tests/entry_points_dispatcher_contract.rs (confirmed by elai.map lines 48946-48952)
- Contract test: artifact_ssb_contract.rs covers EntryPointsDetected roundtrip — evidence: crates/contracts/tests/artifact_ssb_contract.rs:66
- Unit tests: entry_points_orphan.rs, entry_points_well_known.rs, entry_points_framework.rs, entry_points_test_names.rs, entry_points_dedup.rs — evidence: crates/knowledge/tests/ (all five files confirmed)
- Decision doc created — evidence: workflow/reasoning/B-34-decisions.md (confirmed by ls)
- Zero .unwrap() rule: types.rs:80 uses .unwrap_or(u32::MAX), production code is clean — evidence: crates/knowledge/src/symbol_graph/entry_points/types.rs:80
- include_tests filter on EntryPointsRequest supported — evidence: crates/orchestrator/src/production_agent_rpc.rs:1180-1182

## Claims PARTIAL

- Dispatcher tool exposed via crates/orchestrator/src/dispatcher.rs
  - exists: The tool arm is in crates/orchestrator/src/production_agent_rpc.rs:1110 (TOOL_ENTRY_POINTS), not in dispatcher.rs
  - gap: dispatcher.rs has zero entry_points references; the plan's target file differs from actual implementation location. Functionally equivalent but file path in plan is wrong.

- Designer auto-injection condition: "when decision_source == TechStackDecision::ExistingProject"
  - exists: crates/orchestrator/src/task_runner.rs:4346 — build_existing_project_context called when project_dir.is_some()
  - gap: Injection is guarded by project_dir presence, not by TechStackDecision::ExistingProject enum match. TechStackDecision/ExistingProject symbols do not appear in the orchestrator codebase. The injection is functionally active but uses a simpler project_dir != None guard rather than the plan's exact enum check.

- EntryPointsDetected SSB payload location: plan says crates/core/src/ssb/action_types.rs
  - exists: crates/orchestrator/src/ssb.rs:899 (SsbActionType::EntryPointsDetected)
  - gap: Implemented in orchestrator/ssb.rs, not core/ssb/action_types.rs. No core/ssb/ module path exists in the codebase.

- Payload selectivity: only orphan files + conventional filenames loaded (not all files)
  - exists: Selectivity present via CODE_EXTENSIONS filter (crates/orchestrator/src/production_agent_rpc.rs:1145-1158)
  - gap: The plan required loading only orphan-candidate files + conventional filename matches. Actual implementation loads all code-extension files, with a comment at line 1141-1144 noting that the stricter orphan+filename filter was reverted (DECISION-B34-9) because it broke Heuristic 3. The DETERMINISTIC_CHECK item 6 (ELAI_DEBUG_PAYLOAD_LOAD env var + <<10% selectivity assertion) is not implemented.

## Claims MISSING

- Researcher's /research Oracle pre-seeding with PROJECT_STARTING_POINTS — searched: "PROJECT_STARTING_POINTS", "entry_points.*oracle", "research.*entry_points" in crates/cli/src/live_cli.rs and crates/cli/src/main.rs; no evidence. The /research handler at live_cli.rs:2203 calls run_research_pipeline with no entry-point injection.
- entry_points_performance.rs test — searched: "entry_points_performance" across .elai_cc; no file found. Only five of the seven planned test files were created.
- entry_points_payload_selectivity.rs test — searched: "entry_points_payload_selectivity" across .elai_cc; no file found.
- ELAI_DEBUG_PAYLOAD_LOAD env var for selectivity logging — searched: "ELAI_DEBUG_PAYLOAD_LOAD" across .elai_cc; no evidence.

## Claims UNVERIFIABLE

- Performance: <500ms p99 on 10K-file project, <100ms on ELAI repo — reason: no performance test file exists; runtime measurement not possible from static analysis.
- "PROVEN end-to-end" integration observable_effect (EntryPointsDetected SSB emitted on /research) — reason: /research does not call entry_points tool; observable_effect trigger does not match actual code path.

## What must be done (action list)

- [ ] Implement /research Oracle pre-seeding: in live_cli.rs SlashCommand::Research arm, call detect_entry_points and prepend PROJECT_STARTING_POINTS block to the query before run_research_pipeline (crates/cli/src/live_cli.rs ~line 2195)
- [ ] Create crates/knowledge/tests/entry_points_performance.rs with synthetic 10K-file fixture and <500ms wall assertion
- [ ] Create crates/knowledge/tests/entry_points_payload_selectivity.rs with mock SymbolGraphCache recording load_payload calls
- [ ] Add ELAI_DEBUG_PAYLOAD_LOAD=1 logging in TOOL_ENTRY_POINTS arm (crates/orchestrator/src/production_agent_rpc.rs ~line 1173) to report payload count vs total files
- [ ] Update plan file to reflect actual implementation location (production_agent_rpc.rs, not dispatcher.rs) and actual SSB location (orchestrator/ssb.rs, not core/ssb/action_types.rs)

## Notes

The plan's struct name "EntryPoint" was implemented as "DetectedEntryPoint" to avoid collision with the pre-existing B-29 EntryPoint type in the same crate (documented in types.rs:3-5). This is a deliberate divergence, not an oversight — the types.rs comment explains the rationale. The IPC wire types (EntryPointDto, EntryPointsRequest, EntryPointsResponse) are in crates/ipc-types/src/lib.rs as the plan intended.

REPORT_WRITTEN=F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-34_entry_point_detection.report.md | status=MOSTLY_DONE | done=18 | partial=4 | missing=4 | unverifiable=2
