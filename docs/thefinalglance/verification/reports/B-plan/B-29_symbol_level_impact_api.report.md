# Verification Report: B-29_symbol_level_impact_api.md
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-29_symbol_level_impact_api.md
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
B-29 is substantially implemented. All Phase 1-3 symbols (schema, types, sharding, extraction, resolution, cache, traversal) exist in the knowledge crate and match the spec. Phase 4 (IndexingService full-build call) is wired. Phase 5 (SSB SymbolQuery, IMPACT_SUMMARY PostToolUse, dashboard panel) is implemented. Two gaps remain: (1) the four dispatcher tool entries landed in production_agent_rpc.rs rather than dispatcher.rs and are NOT exposed in build_tool_catalog (so agents cannot self-select them), and (2) the incremental update hook (update_changed_files_symbol_graph) is not wired into indexing_service.rs — it exists in the knowledge crate but the orchestrator watcher never calls it. The CLI also does not expose /symbol_impact as a REPL command.

## Claims verified DONE

- Schema tables (symbol_name_index, symbol_reverse_index, symbol_file_payload, symbol_graph_meta) defined in migration_v3.sql — evidence: .elai_cc\crates\knowledge\src\symbol_graph\migration_v3.sql:8-42
- Schema migration idempotent (CREATE TABLE IF NOT EXISTS) — evidence: migration_v3.sql:6
- blake3 used for reverse_shard_bucket (plan said SHA1; correct deviation documented) — evidence: .elai_cc\crates\knowledge\src\symbol_graph\sharding.rs:9-61
- SymbolKind enum (9 variants), Confidence enum (4 tiers), SymbolNode, SymbolEdge, SymbolRef, EntryPoint, SymbolGraphMeta all defined — evidence: .elai_cc\crates\knowledge\src\symbol_graph\types.rs:23-158
- name_shard_key, reverse_shard_bucket, all_name_shard_keys implemented — evidence: sharding.rs:44-75
- Constants SYMBOL_NAME_SHARDS=27, SYMBOL_REVERSE_SHARDS=256, SYMBOL_FILE_LRU_SIZE=500, MAX_IMPACT_DEPTH=10, MAX_FLOW_DEPTH=10 — evidence: sharding.rs:18-30
- extract_symbols_and_calls implemented with <module> synthetic symbol — evidence: .elai_cc\crates\knowledge\src\symbol_graph\extraction.rs:1-47
- resolve_call_sites (4-tier: Local/Imported/Re-export/Unresolved) — evidence: .elai_cc\crates\knowledge\src\symbol_graph\resolution.rs:1-55
- SymbolGraphCache with hand-rolled LRU (BTreeMap-based, capacity 500) — evidence: .elai_cc\crates\knowledge\src\symbol_graph\cache.rs:52-188
- All four traversal functions: compute_impact_radius, compute_call_flow, compute_symbol_context, list_symbols — evidence: .elai_cc\crates\knowledge\src\symbol_graph\traversal.rs:149, 267, 417, 523
- ImpactResult.truncated flag, FlowTruncationReason enum — evidence: traversal.rs:34-94
- looks_like_file_path polymorphic target detection — evidence: traversal.rs:121-131
- build_symbol_graph called from IndexingService.build_index_resumable after DKG build — evidence: .elai_cc\crates\orchestrator\src\indexing_service.rs:574-588
- INCREMENTAL_SYMBOL_THRESHOLD=50 constant defined in build_pipeline.rs — evidence: .elai_cc\crates\knowledge\src\symbol_graph\build_pipeline.rs:44-46
- update_changed_files_symbol_graph implemented in knowledge crate — evidence: build_pipeline.rs:242
- SsbActionType::SymbolQuery variant, SsbPayload::SymbolQuery, SymbolQueryPayload{tool, target, hop_count, total_files, latency_ms} — evidence: .elai_cc\crates\orchestrator\src\ssb.rs:814,3465-3467,SCHEMA_VERSION 37
- PostToolUse IMPACT_SUMMARY block written to auditor briefing log — evidence: .elai_cc\crates\orchestrator\src\task_runner.rs:5806-5808, 8526-8794
- compute_impact_radius called from write_impact_summary_to_log at depth=2 — evidence: task_runner.rs:8764
- Dashboard KnowledgeDrawer IMPACT ANALYSIS panel with hop band table — evidence: .elai_cc\crates\dashboard-v2\src\lib\drawers\KnowledgeDrawer.svelte:289-340
- get_impact_analysis Tauri command — evidence: .elai_cc\crates\dashboard-v2\src-tauri\src\commands.rs:967
- knowledge crate exports symbol_graph module — evidence: .elai_cc\crates\knowledge\src\lib.rs:39
- Tests: symbol_graph_schema_migration.rs, symbol_graph_sharding.rs, symbol_graph_types_roundtrip.rs, symbol_graph_extraction.rs, symbol_graph_resolution.rs, symbol_graph_cache_tests.rs, symbol_graph_traversal_tests.rs, symbol_graph_store.rs, symbol_graph_build_pipeline.rs — evidence: .elai_cc\crates\knowledge\tests\
- Contract tests: symbol_graph_b29_phase1_contract.rs through phase5_contract.rs, symbol_graph_contract.rs — evidence: .elai_cc\crates\contracts\tests\
- symbol_graph store.rs (msgpack/postcard persistence) — evidence: .elai_cc\crates\knowledge\src\symbol_graph\store.rs
- B-29-decisions.md written — evidence: F:\.ELAI_workflow\workflow\reasoning\B-29-decisions.md

## Claims PARTIAL

- Four dispatcher tool entries registered (plan: crates/orchestrator/src/dispatcher.rs; actual: production_agent_rpc.rs)
  - exists: .elai_cc\crates\orchestrator\src\production_agent_rpc.rs:807-971 (symbol_impact, symbol_flow, symbol_context, symbol_list handled in translate_tool_request)
  - gap: NOT added to build_tool_catalog (production_agent_rpc.rs:312 returns only 6 tools: edit, file_read, validate, compile_check, classify, git_commit). Agents receive no ToolDefinition for symbol_* tools so the LLM cannot self-select them. Also landed in production_agent_rpc.rs, not dispatcher.rs as specified.

- Dashboard "Run /symbol_impact target=..." input form
  - exists: KnowledgeDrawer.svelte:289-340 (panel exists, shows target/total_files/hop_bands/unresolved_pct)
  - gap: No submit_task input form or submit_task Tauri command wiring found in the panel. The plan (Task 14) explicitly required "Run /symbol_impact target=..." input form delegating to submit_task.

- Incremental watcher update hook (Task 10: update_changed_files_symbol_graph called from update_project_index)
  - exists: .elai_cc\crates\knowledge\src\symbol_graph\build_pipeline.rs:242 (function implemented and tested)
  - gap: .elai_cc\crates\orchestrator\src\indexing_service.rs has no call to update_changed_files_symbol_graph. The watcher/incremental path is unconnected in the orchestrator.

## Claims MISSING

- crates/orchestrator/tests/symbol_query_dispatch.rs — searched: Glob **/*.rs in orchestrator/tests; no match. The plan required an end-to-end IPC dispatcher test.

## Claims UNVERIFIABLE

- SymbolGraphMeta.unresolved_edge_pct < 50.0 on live ELAI repo build — reason: runtime data; requires sqlite3 query against a populated dkg.db which is not present in the source tree.
- Auditor IMPACT_SUMMARY block contains >= 5 dependent files for compute_with_profile rename — reason: runtime/log data; not verifiable from source.
- DETERMINISTIC_CHECK items (sqlite3 table counts, CLI /symbol_impact stdout output) — reason: all require a live indexed database and running binary.

## What must be done (action list)

- [ ] Add ToolDefinition entries for symbol_impact, symbol_flow, symbol_context, symbol_list to build_tool_catalog in production_agent_rpc.rs:312 (or re-export via the canonical dispatcher.rs path per spec)
- [ ] Wire update_changed_files_symbol_graph into orchestrator/src/indexing_service.rs incremental-update path (when changed_file_count <= INCREMENTAL_SYMBOL_THRESHOLD, call it; otherwise full rebuild)
- [ ] Add "Run /symbol_impact target=..." input form to KnowledgeDrawer.svelte IMPACT ANALYSIS panel, wired to submit_task Tauri command
- [ ] Create crates/orchestrator/tests/symbol_query_dispatch.rs — end-to-end IPC dispatcher test for all four symbol_ tools
- [ ] Run DETERMINISTIC_CHECK items 1-7 against a live build to validate runtime invariants (unresolved_edge_pct, hop band output, LRU bound)

## Notes

blake3 substitution for SHA1 in reverse_shard_bucket is correct per CLAUDE.md anti-stale rule and is self-documented in sharding.rs. postcard is used instead of rmp-serde (msgpack); column names retain _msgpack suffix per spec but encoding is postcard — this is documented in migration_v3.sql:4. The orchestrator symbol_graph module (production_agent_rpc.rs imports) is the B-29 knowledge crate path, not the B-2 elai.map-based module at orchestrator/src/symbol_graph.rs.
