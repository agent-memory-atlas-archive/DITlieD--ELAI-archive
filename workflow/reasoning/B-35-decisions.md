B-35 Phase A — Design Decisions

WHAT
Implement the context artifacts pipeline Phase A: manifest loader, types, indexer,
staleness detector, SQLite store, and in-memory cosine search.

KEY DECISIONS

1. Config format: TOML primary (.elai_cc.toml [[knowledge.artifacts]]) + JSON fallback
   (.elai_artifacts.json). Rationale: ELAI already uses elai_cc.toml for all other
   runtime config; adding a [knowledge.artifacts] section keeps the config surface
   unified. JSON fallback gives SocratiCode migration parity (.socraticodecontextartifacts.json
   → .elai_artifacts.json). Decision: option A from B-35 spec.

2. Hashing: blake3 throughout — chunk IDs, file hashes, content hashes.
   SocratiCode uses sha256; CLAUDE.md anti-stale rule mandates blake3.
   Chunk ID = first 32 hex chars of blake3(artifact_name:relative_path:start_line).

3. SQL CREATE boundary chunking via line state machine.
   Triggers on keyword "CREATE" followed by TABLE/VIEW/INDEX/FUNCTION/TRIGGER/TYPE.
   Implemented as a small bool-flag line scanner (no regex crate — regex is present
   in workspace but the scanner is trivially implementable with str::split_whitespace
   and avoids adding a heavy dependency to the hot path).

4. LockScope::Index reuse.
   integration-map confirms LockScope::Index is committed at lock.rs:266.
   Phase A artifact indexing runs inside the existing IndexingService flow which
   already holds LockScope::Index, so no new scope is needed.

5. In-memory cosine for Phase A search.
   HNSW and tantivy integration deferred to Phase B. Phase A loads stored
   embedding blobs from SQLite, computes cosine in-process, sorts, slices to limit.
   This is correct and safe for the expected artifact count (<100 chunks).

6. Embedding optional (Option<&dyn VectorModule>).
   Phase A stores embedding=None when no VectorModule is injected (test context,
   startup before model loads). Cosine search returns empty when embeddings absent.

7. ArtifactStore separate DB: context_{project_id}.db in indexes_dir.
   Never shares dkg.db or metadata.db (schema collision prevention, pitfall from
   context-package.md). Follows METADATA_DB_FILENAME pattern from metadata.rs.

8. v3 migration in metadata.db for ArtifactIndexState.
   Artifact state (name, last_indexed_at, file_hashes_json, chunk_count, status)
   stored in the same metadata.db via a new migration_v3.sql applied at
   ProjectMetadataStore::open. Keeps all project state in one DB, one backup path.
   Separate ArtifactStore (context_{project_id}.db) holds chunk content + embeddings.

9. HNSW in Phase B.
   Phase A search is O(n) cosine. Phase B wires ArtifactStore embeddings into
   BruteForceIndex / usearch HNSW and tantivy for hybrid BM25+vector artifact search.

EXPECTED TEST OUTCOME
artifact_manifest.rs: 4 tests GREEN
artifact_indexer.rs: 4 tests GREEN (tempfile-based, no live embedding)
artifact_staleness.rs: 5 tests GREEN
artifact_search.rs: 2 tests GREEN (in-memory cosine, mock embeddings)
contracts/artifact_pipeline_contract.rs: 2 tests GREEN

UNKNOWNS AT IMPLEMENTATION TIME
- indexing_service.rs run_full_index signature may have drifted from line 316 (last
  read 2026-05-15). Verified against integration-map before touching.
- B-29 may hold a session lock on indexing_service.rs. Phase A avoids touching it;
  Task 6 hook is deferred pending coordination.

---

B-35 Phase B — Design Decisions

WHAT
ArtifactStage + 4 dispatcher tools (embed/scan/index/search) + VerificationMethod::ArtifactCheck
+ parse_claim_prefix "artifact:NAME:QUERY" + discriminant-gated allowed_methods.

KEY DECISIONS

DECISION-B1: ArtifactStage builder pattern (vs central pipeline dispatch)
Rationale: cross-session conflict avoidance. Central pipeline dispatch (e.g. touching pipeline.rs
in the same edit as reasoning/engine.rs) triggers session lock conflicts when multiple sessions
own different regions. The builder pattern confines Phase B changes to stage.rs (knowledge) and
reasoning/{types,verification,engine}.rs independently — no shared lock between the two subsystems.
Rule: when two crates both need to change in a phase, isolate the seam via a builder/callback so
each crate can ship independently without requiring a shared mutable pipeline slot.

DECISION-B2: VerificationMethod Copy removal + discriminant-based gate
VerificationMethod gained ArtifactCheck(String, String) — a data-carrying variant.
Copy derive is incompatible with non-Copy heap data (String); removed Copy.
Vec::contains() / PartialEq compares all fields, so ArtifactCheck("","") would never match
ArtifactCheck("real","query"). Fix: std::mem::discriminant comparison in a supports() helper.
Rule: data-carrying enum variants require discriminant gates, never PartialEq gates. See gotcha
[B35_DATA_ENUM_DISCRIMINANT_VS_VEC_CONTAINS].

DECISION-B3: artifact_lookup callback injection (reasoning has no dep on knowledge)
reasoning::verification::LocalVerifier is injected with artifact_lookup: Option<Arc<dyn Fn(name,
query) -> Vec<ArtifactChunk> + Send + Sync>>. This keeps elai-reasoning free of an elai-knowledge
dep (crate isolation invariant: reasoning is a pure logic crate). The callback is constructed in
production_agent_rpc.rs where both knowledge (ArtifactStage) and reasoning (LocalVerifier) are
co-located. Rule: when two isolated crates need to cooperate at runtime, use a callback/closure
injection rather than adding a crate dep to the "lower" crate.

DECISION-B4: parse_claim_prefix "artifact:NAME:QUERY" syntax + strict unknown-prefix rejection
Syntax: "artifact:<artifact_name>:<query_string>" — colon-split, exactly 3 parts.
Strict: any prefix not in {"artifact", "bm25", "dkg", "web"} returns Err(UnknownPrefix).
Rationale: silent _ => continue was masking malformed citations from agents. Fast feedback on
unknown prefixes means agents catch typos ("artefact:...") immediately. See gotcha
[B35_PARSE_CLAIM_PREFIX_SILENT_CONTINUE].

DECISION-B5: Split embed(async)/scan(spawn_blocking) dispatcher pattern under !Send
VectorModule is !Send (holds raw pointers / non-Send ONNX state). An async arm cannot hold a
!Send reference across an await boundary in a multi-thread tokio runtime. Solution: split into
two arms — embed dispatches a Send-safe request without holding vm, scan wraps vm.scan() in
tokio::task::spawn_blocking(move || ...) to run on the blocking thread pool. Locked by
artifact_dispatcher_contract.rs 12/12. See gotcha [B35_SEND_GUEST_ASYNC_DISPATCHER].

DECISION-B6: index_artifacts dispatcher returns Err today (vm injection path Phase C)
The index_artifacts arm in production_agent_rpc.rs returns Err("vm not yet injected") rather
than silently performing an empty store operation. Design intent: never silently empty an artifact
store when vm is absent. Phase C will thread project_root + vm through the builder fields
(artifact_vm + artifact_store on ProductionAgentRpcServer) and replace the Err with a real
full-reindex dispatch. This mirrors the B4 ArtifactsError::ClockOverflow pattern — fail loud
rather than silently degrade.

EXPECTED TEST OUTCOME (Phase B)
elai-reasoning: 18/18 GREEN
artifact_grounding_contract: 5/5 GREEN
artifact_dispatcher_contract: 12/12 GREEN (incl 5 postcard roundtrip)
artifact_manifest+indexer+staleness+search: 15/15 GREEN
cargo check elai-knowledge/elai-reasoning/elai-orchestrator/elai-ipc-types/elai_cc_contracts: CLEAN

DEFERRED (NOT incomplete Phase B)
- Designer briefing AVAILABLE_ARTIFACTS auto-inject (task_runner.rs — s-10380 owns)
- SSB action_types ArtifactsIndexed/ArtifactSearched/ArtifactStale (ssb.rs — s-10380 owns)
- Researcher /research auto-inject PROJECT_STARTING_POINTS (B-34 territory; cli/src/main.rs)
- Dashboard ArtifactsPanel + Tauri commands (Phase C)
- index_artifacts full re-index path (project_root threading; Phase C wiring)
