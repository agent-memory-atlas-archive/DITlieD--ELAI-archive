# Verification Report: B-35_context_artifacts_pipeline.md
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-35_context_artifacts_pipeline.md
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
The core artifact pipeline infrastructure is shipped: all source files in
crates/knowledge/src/artifacts/ exist (manifest, types, indexer, store, search,
staleness, stage, mod), ArtifactStage is inserted into QueryPipeline at
STANDARD+DEEP tiers, VerificationMethod::ArtifactCheck is fully implemented in
the reasoning crate, AVAILABLE_ARTIFACTS injection fires in task_runner.rs, the
3 SSB action types exist, ArtifactsPanel.svelte is wired into KnowledgeDrawer,
and 4 Tauri commands are in commands.rs. Three gaps remain: (1) IndexingService
does not call index_all_artifacts after code indexing; (2) the plan-specified
tests crates/reasoning/tests/grounding_artifact_check.rs and
crates/orchestrator/tests/designer_artifact_briefing.rs are absent; (3)
docs/examples/elai_cc.example.toml does not exist.

## Claims verified DONE

- ArtifactManifest::load in crates/knowledge/src/artifacts/manifest.rs — evidence: crates/knowledge/src/artifacts/manifest.rs:112
- ArtifactSpec, ArtifactChunk, ArtifactIndexState, ArtifactStatus types — evidence: crates/knowledge/src/artifacts/types.rs (imported at staleness.rs:16, indexer.rs:90)
- ArtifactIndexer::index_artifact, index_all_artifacts, index_single_file, chunk_file — evidence: crates/knowledge/src/artifacts/indexer.rs:85, 201, 229, 313
- ArtifactStore (SQLite context_* collection) — evidence: crates/knowledge/src/artifacts/store.rs:90; CONTEXT_DB_FILENAME_PREFIX used in contract test artifact_pipeline_contract.rs:46
- detect_staleness + staleness.rs — evidence: crates/knowledge/src/artifacts/staleness.rs; ArtifactIndexState imported at staleness.rs:16
- artifact_search function — evidence: crates/knowledge/src/artifacts/search.rs:46
- ArtifactStage in QueryPipeline (STANDARD+DEEP only) — evidence: crates/knowledge/src/pipeline.rs:533; crates/knowledge/src/artifacts/stage.rs:107
- ArtifactStage exported from crates/knowledge/src/artifacts/mod.rs — evidence: crates/knowledge/src/artifacts/mod.rs:37
- VerificationMethod::ArtifactCheck variant with artifact_name + query — evidence: crates/reasoning/src/verification.rs:350-379; crates/reasoning/src/types.rs:247, 387
- ArtifactCheck dispatch: no lookup -> MalformedGrounding; lookup returning true -> passes — evidence: crates/reasoning/src/verification.rs:354-379
- requires_artifact_store() / is_artifact_check() predicate — evidence: crates/reasoning/src/types.rs:387
- 3 SSB action types: ArtifactsIndexed, ArtifactSearched, ArtifactStale — evidence: crates/orchestrator/src/ssb.rs:903, 1110, 2545, 2561, 2578
- SSB payload structs: ArtifactsIndexedPayload, ArtifactSearchedPayload, ArtifactStalePayload — evidence: crates/orchestrator/src/ssb.rs:2539, 2557, 2575
- 4 Tauri commands: list_artifacts, search_artifacts, index_artifacts, remove_artifacts — evidence: crates/dashboard-v2/src-tauri/src/commands.rs:6265, 6297, 6367, 6399
- ArtifactsPanel.svelte component — evidence: crates/dashboard-v2/src/lib/components/ArtifactsPanel.svelte:75
- ArtifactsPanel mounted in KnowledgeDrawer — evidence: crates/dashboard-v2/src/lib/drawers/KnowledgeDrawer.svelte:7
- AVAILABLE_ARTIFACTS block injected in Designer briefing (build_existing_project_context) — evidence: crates/orchestrator/src/task_runner.rs:8588, 8678-8696
- Manifest TOML path primary, .elai_artifacts.json fallback — evidence: crates/knowledge/src/artifacts/manifest.rs:153, 169
- blake3 staleness hashing — evidence: crates/knowledge/src/artifacts/staleness.rs:155 (hash.to_hex)
- Path bounds-check (paths outside project_root rejected) — evidence: crates/knowledge/src/artifacts/manifest.rs:222 (canonical_root.join; resolve_spec)
- Name regex validation — evidence: crates/knowledge/src/artifacts/manifest.rs:193
- Non-empty description validation — evidence: crates/knowledge/src/artifacts/manifest.rs:134, 137
- ipc-types tool constants: TOOL_LIST_ARTIFACTS, TOOL_INDEX_ARTIFACTS, TOOL_SEARCH_ARTIFACTS, TOOL_REMOVE_ARTIFACTS — evidence: crates/contracts/tests/artifact_dispatcher_contract.rs:24-27
- Wire structs (ListArtifactsRequest, SearchArtifactsRequest, IndexArtifactsRequest, RemoveArtifactsRequest) with serde+postcard roundtrip — evidence: crates/contracts/tests/artifact_dispatcher_contract.rs:47-200
- Contract tests for isolation and non-panic on missing config — evidence: crates/contracts/tests/artifact_pipeline_contract.rs:23-74
- SSB roundtrip contract tests — evidence: crates/contracts/tests/artifact_ssb_contract.rs:42, 47, 52
- knowledge unit tests: artifact_manifest.rs, artifact_search.rs, artifact_staleness.rs, artifact_indexer.rs — evidence: crates/knowledge/tests/artifact_manifest.rs:73, crates/knowledge/tests/artifact_staleness.rs:15
- Failure non-fatal: missing config returns empty manifest, not Err — evidence: crates/contracts/tests/artifact_pipeline_contract.rs:23-36

## Claims PARTIAL

- dispatcher tool entries (list_artifacts, search_artifacts, remove_artifacts working; index_artifacts is a STUB)
  - exists: crates/orchestrator/src/production_agent_rpc.rs:988 (list_artifacts), 1021 (search_artifacts), 1008 (index_artifacts stub)
  - gap: index_artifacts returns Err("full indexing deferred to Phase C") at production_agent_rpc.rs:1014-1018; agents cannot invoke it; CLI indexing is the only path

- IndexingService::build_index hook calls index_all_artifacts after code indexing
  - exists: index_all_artifacts defined at crates/knowledge/src/artifacts/indexer.rs:201
  - gap: crates/orchestrator/src/indexing_service.rs contains zero references to artifacts, index_all_artifacts, or ArtifactIndexer; the hook is not wired

- ActivityFeed surfaces ArtifactsIndexed / ArtifactSearched / ArtifactStale SSB events
  - exists: SSB payloads + action types defined (ssb.rs:2539, 2561, 2578); ArtifactStatus type referenced in stub.ts:16
  - gap: no match for these action type strings in dashboard-v2/src rendering code; feed display not verified

## Claims MISSING

- crates/reasoning/tests/grounding_artifact_check.rs — searched: grounding_artifact_check, ArtifactCheck in reasoning/tests; no file found; tests for ArtifactCheck verifies/MalformedGrounding/Unverified paths are absent
- crates/orchestrator/tests/designer_artifact_briefing.rs — searched: designer_artifact_briefing in orchestrator/tests; no file found; ExistingProject injection test absent
- docs/examples/elai_cc.example.toml — searched: elai_cc.example.toml, example.*toml in elai.map and filesystem; Test-Path returned False
- tests/contracts/artifact_pipeline_contract.rs (plan reference) — note: a file named artifact_pipeline_contract.rs exists at crates/contracts/tests/ (covers isolation + non-panic only); the plan's "regression: code indexing must not be blocked" item is covered but the "context_* always isolated from codebase_/codegraph_ collections" is only partially tested (filename prefix, not live concurrent run)

## Claims UNVERIFIABLE

- Lock interaction: artifacts coexist with code indexing without collision — reason: IndexingService hook is not wired (see PARTIAL above); cannot verify LockScope::Index coordination for artifact indexing since artifact indexing is not triggered from indexing_service.rs; LockScope enum only has Index and Watch variants, no Artifacts sub-scope exists
- Failure non-fatal for code indexing when artifact path is invalid — reason: IndexingService hook is absent so the "artifact errors never block code indexing" path at the service level cannot be exercised end-to-end; the manifest-level non-panic is verified but service-level isolation is untested

## What must be done (action list)

- [ ] Wire index_all_artifacts into IndexingService::build_index_resumable after code phase completes (crates/orchestrator/src/indexing_service.rs); emit ArtifactsIndexed SSB on completion and ArtifactStale when detect_staleness fires
- [ ] Implement TOOL_INDEX_ARTIFACTS in production_agent_rpc.rs (currently returns Err stub at line 1014); Phase C referenced in comment
- [ ] Create crates/reasoning/tests/grounding_artifact_check.rs with tests: ArtifactCheck verifies when search returns results above threshold, MalformedGrounding when artifact_name not declared, Unverified when no results above threshold
- [ ] Create crates/orchestrator/tests/designer_artifact_briefing.rs with tests: ExistingProject decomposition injects AVAILABLE_ARTIFACTS block, Greenfield skips it
- [ ] Create docs/examples/elai_cc.example.toml with [[knowledge.artifacts]] examples (SQL schema, OpenAPI, k8s, Terraform, ADR, domain glossary)
- [ ] Verify ActivityFeed in dashboard renders ArtifactsIndexed/ArtifactSearched/ArtifactStale events (no evidence in dashboard-v2/src rendering code)

## Notes

The plan refers to GroundingMethod::ArtifactCheck as a 7th method; the actual implementation uses VerificationMethod::ArtifactCheck (in elai-reasoning crate) not GroundingMethod (in elai-orchestrator ipc_gateway). The ipc_gateway GroundingMethod enum has FileCheck, WebCheck, SourceCheck variants only. This is a naming discrepancy vs the plan but the reasoning-level verification is correctly implemented. The ArtifactsPanel.svelte shows ~200-char content excerpts, matching the plan's "200 chars of description" spec. The plan's TechStackDecision::ExistingProject conditional was not found as an enum variant gating the AVAILABLE_ARTIFACTS injection; the injection function build_existing_project_context fires unconditionally when a project_root is available and the manifest has items.
