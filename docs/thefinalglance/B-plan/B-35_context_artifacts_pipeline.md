# B-35: Context Artifacts Pipeline (full second indexing pipeline)
Objective: ship a parallel indexing pipeline alongside the code DKG/BM25/
vector pipeline that ingests project-typed non-code knowledge — database
schemas, OpenAPI specs, k8s manifests, ADRs, domain glossaries, Terraform
configs — so the Designer agent grounds tech choices in actual project
conventions and the Researcher answers "what tables exist?" / "what
endpoints are defined?" without hallucinating.
Estimated effort: 2 sessions (~10-14 hours).
Pre-requisites: B-1 (path unification), B-31 (lock primitives — artifact
indexing must coexist with code indexing), B-32 (chunker reuse). Recommended
sibling: B-29 (artifacts feed Designer context, which also consumes
SymbolGraphCache).
Parallel-safe with: B-29, B-30, B-33, B-34.

## Context

Per the SocratiCode investigation (DETAILED_SUMMARY_03 §3 item B-35
notes), I initially scoped this as "a config block in elai_cc.toml that
declares trusted file paths" — substantially under-scoped. The
agent investigation surfaced that
`src/services/context-artifacts.ts` is 575 LOC: a complete second indexing
pipeline parallel to the code pipeline. It has its own Qdrant collection
(`context_{projectId}`), its own chunking (line-based for non-code
text), its own embedding (same provider, same model, separate index),
its own staleness detection via SHA-256 hashing, and its own search
contract. Auto-indexed alongside code on `codebase_index`; auto-staleness
triggers re-index transparently.

Why this matters for ELAI: Designer's Recursive Decomposition can ground
"add a `last_login` column to users" by reading the actual SQL schema's
naming conventions (snake_case vs camelCase, `updated_at` triggers,
`created_at` defaults). When the agent invents a column name without
this grounding, it's just guessing. Designer's TechStackDecision::
ExistingProject path is supposed to ground in real project state — but
without artifact awareness, it can ground only in code, missing the
half of project state that lives in `docs/`, `migrations/`,
`openapi.yaml`, `k8s/`, `terraform/`, `.tf`, ADR markdown.

ELAI today has `[research.trusted_domains]` in `elai_cc.toml` (100+
domains tagged tier1) but no equivalent for project-internal non-code
knowledge. The Designer cannot tell the agent "before adding an
endpoint, search the OpenAPI spec for existing patterns".

This B-step ports the SocratiCode pipeline to ELAI's stack:
- Artifact manifest in `elai_cc.toml` (or sibling
  `.elai_artifacts.json` for compatibility)
- Separate vector index collection in ELAI's vector store (already exists
  per Phase 1C — add a per-project `context_*` table)
- Reuse B-32 chunker for non-code text
- Auto-staleness via blake3 (CLAUDE.md anti-stale: blake3, not sha256)
- New retrieval stage `ArtifactStage` in QueryPipeline between CatRAG
  and A-RAG
- PipelineGrc grounding integration: `verify_claim(artifact:database-schema, ...)`
  becomes a real check, not just file existence

Outcome: when Designer says "add `last_login` to users table", the GRC
sweep verifies "users table" against `database-schema` artifact;
verification returns the actual column list including the
`updated_at`/`created_at` convention; Designer's plan reflects it.

## Reads (mandatory before coding)

- F:\.ELAI_workflow\docs\.source\SocratiCode-main\SocratiCode-main\src\services\context-artifacts.ts
  (FULL — 575 LOC; the entire pipeline is here)
- F:\.ELAI_workflow\docs\.source\SocratiCode-main\SocratiCode-main\src\tools\context-tools.ts
  (FULL — MCP tool surface for users)
- F:\.ELAI_workflow\docs\.source\SocratiCode-main\SocratiCode-main\.socraticodecontextartifacts.json.example
  (REFERENCE — the user-facing manifest format)
- F:\.ELAI_workflow\docs\.source\SocratiCode-main\SocratiCode-main\README.md
  search for "Context Artifacts" section (1500+ word user-facing doc
  with motivation + workflow examples — DDD glossary, schema, API spec)
- F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\vector\
  (existing vector index — reuse for context collection)
- F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\bm25.rs
  (existing BM25 — reuse for context BM25)
- F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\pipeline.rs
  (QueryPipeline — add ArtifactStage)
- F:\.ELAI_workflow\.elai_cc\crates\reasoning\src\grounding\verify.rs
  (PipelineGrc::verify_claim — extend with artifact dispatch)
- F:\.ELAI_workflow\docs\specs\guide\steps\phase-2B.txt
  (grounding verification — A-44 already produced verify_claim trait
  with FILE_CHECK / SOURCE_CHECK / TEST_CHECK / WEB_CHECK / API_CHECK /
  API_REGISTRY_CHECK; add ARTIFACT_CHECK as 7th method)

## Tasks

### Phase 1 — Manifest format + loader (Session 1, half)

1. Define artifact manifest format. Two equivalent forms — CHOOSE ONE
   (decision doc):
   - Option A: `[knowledge.artifacts]` block in `elai_cc.toml`. Pros:
     fewer files, fits ELAI's existing config pattern. Cons: TOML doesn't
     handle long descriptions as well as JSON.
   - Option B: standalone `.elai_artifacts.json` at project root. Pros:
     parity with SocratiCode `.socraticodecontextartifacts.json`,
     standalone tooling can consume. Cons: another file.
   Recommend Option A in decision doc; support both with TOML primary.

   TOML format:
   ```toml
   [[knowledge.artifacts]]
   name = "database-schema"
   path = "./docs/schema.sql"
   description = "PostgreSQL schema — all tables, columns, types, indexes, foreign keys, triggers. Check this before writing migrations to match snake_case naming convention and existing updated_at trigger pattern."

   [[knowledge.artifacts]]
   name = "api-spec"
   path = "./docs/openapi.yaml"
   description = "OpenAPI 3.0 spec — all endpoints, request/response schemas, Bearer auth, cursor/limit pagination. Check this before adding endpoints to match { data, meta } wrapper convention."

   [[knowledge.artifacts]]
   name = "ubiquitous-language"
   path = "./docs/ubiquitous-language.md"
   description = "Domain glossary — bounded context terms, definitions, relationships. Always check before naming entities, events, commands."

   [[knowledge.artifacts]]
   name = "k8s-manifests"
   path = "./deploy/k8s/"
   description = "Kubernetes deployment manifests. Shows how services are deployed, scaled, networked. Check before adding env vars or resource limits."
   ```
   Each artifact has: `name` (id, used in tool filtering), `path` (file
   or dir), `description` (steers LLM — written as "check this before
   X").

2. Implement `ArtifactManifest::load(project_root: &Path) -> Result<Self>`
   in `crates/knowledge/src/artifacts/manifest.rs`:
   - Parse from elai_cc.toml `[[knowledge.artifacts]]` blocks
   - Validate `name` matches `[a-zA-Z0-9_-]+` (used as collection
     suffix)
   - Resolve `path` relative to project_root; reject if path resolves
     outside project_root (security: prevent reading arbitrary system
     files)
   - For directories, expand to file list during indexing (recursive,
     applies project's existing `.elai_ignore` rules)
   - Return `Vec<ArtifactSpec>`

3. Add `Artifact` types in `crates/knowledge/src/artifacts/types.rs`:
   ```rust
   pub struct ArtifactSpec {
       pub name: String,
       pub path: PathBuf,
       pub description: String,
       pub is_dir: bool,
   }
   pub struct ArtifactChunk {
       pub id: String,                 // blake3 hash of (name, path, byte_offset)
       pub artifact_name: String,
       pub artifact_description: String,
       pub relative_path: String,      // path within artifact (file in dir)
       pub content: String,
       pub start_line: u32,
       pub end_line: u32,
       pub content_hash: String,       // blake3 of whole-file content
       pub embedding: Option<Vec<f32>>, // populated by indexer
   }
   pub struct ArtifactIndexState {
       pub name: String,
       pub last_indexed_at: i64,
       pub file_hashes: BTreeMap<String, String>, // file → blake3
       pub chunk_count: usize,
       pub status: ArtifactStatus,
   }
   pub enum ArtifactStatus {
       NotIndexed,
       Indexed,
       Stale { changed_files: Vec<String> },
       Indexing,
   }
   ```

### Phase 2 — Indexing pipeline (Session 1, half)

4. Implement `ArtifactIndexer` in
   `crates/knowledge/src/artifacts/indexer.rs`:
   - `index_artifact(project_id, spec, embedding_provider) -> Result<ArtifactIndexState>`
     - For each file in spec (single file or expand dir):
       - Read content
       - Compute blake3 hash
       - If hash matches existing record → skip (staleness check)
       - Else:
         - Chunk content (line-based for `.md`, `.yaml`, `.json`,
           `.tf`, `.toml`; reuse B-32's `chunk_by_lines` with
           CHUNK_SIZE=50; for SQL: chunk at `CREATE TABLE`/`CREATE
           VIEW`/`CREATE INDEX` boundaries via simple regex)
         - Apply `apply_char_cap` from B-32
         - Embed each chunk via existing embedding_provider
         - Build `ArtifactChunk` records with `prepareDocumentText`-
           equivalent prefix (`"context_artifact: {artifact_name}\n{content}"`)
         - Upsert to context vector index (separate collection per project)
       - Update `file_hashes` map for this file
     - Persist `ArtifactIndexState` to project metadata store

5. Add separate vector index collection for artifacts:
   - SQLite table `context_chunks` with columns: `id, artifact_name,
     relative_path, content, start_line, end_line, embedding,
     content_hash, indexed_at`
   - HNSW index over `embedding` for vector search
   - tantivy index for BM25 over `content`
   - Per-project: collection name `context_{project_id}` (mirror
     SocratiCode's contextCollectionName)

6. Hook into `IndexingService`:
   - After code indexing completes, scan for artifact manifest
   - For each artifact: load existing state, detect stale files,
     re-index changed files only
   - Logged as `ArtifactsIndexed` SSB action with summary
   - Failure non-fatal: log warn, continue. Code index integrity is
     never compromised by artifact failures.

7. Cross-process lock interaction: artifact indexing uses
   `LockScope::Index` (B-31) via shared scope with code indexing.
   Artifacts can't index simultaneously with code (avoids embedding
   provider rate-limit collision). Or use a sub-scope `LockScope::
   Artifacts` if separation matters for performance — decide in
   decision doc.

### Phase 3 — Retrieval and search (Session 2, half)

8. Implement `ArtifactStage` in QueryPipeline
   (`crates/knowledge/src/pipeline.rs`):
   - Insert between CatRAG and A-RAG stages
   - Query: same as code search (semantic via embedding + BM25 RRF
     fusion)
   - Returns `Vec<ArtifactSearchResult>` with score + content + artifact
     metadata
   - Tier-routing: ArtifactStage runs in STANDARD + DEEP tiers
     (Phase 3K). FAST tier = code-only.

9. Implement `search_artifacts` orchestrator dispatcher tool:
   ```rust
   "search_artifacts" => {
       let project_id = args.project_id;
       let query = args.query;
       let artifact_name = args.artifact_name; // optional filter
       let limit = args.limit.unwrap_or(10);
       let results = artifact_search(project_id, &query,
           artifact_name.as_deref(), limit).await?;
       Ok(json!({ "results": results }))
   }
   ```
   Plus `list_artifacts` and `index_artifacts` and `remove_artifacts`
   tools mirroring SocratiCode's 4-tool set
   (`src/tools/context-tools.ts`):
   - `list_artifacts` → returns manifest + per-artifact status
   - `index_artifacts` → force-reindex (usually auto-triggered)
   - `search_artifacts` → semantic search (above)
   - `remove_artifacts` → drop the context_* collection

### Phase 4 — Grounding integration (Session 2, half)

10. Add 7th grounding method `ARTIFACT_CHECK` to GRC trait
    (`crates/reasoning/src/grounding/verify.rs`):
    ```rust
    pub enum GroundingMethod {
        FileCheck,
        SourceCheck,
        TestCheck,
        WebCheck,
        ApiCheck,
        ApiRegistryCheck,
        ArtifactCheck { artifact_name: String, query: String },
    }
    ```
    Implementation: dispatches `search_artifacts` tool with
    `artifact_name` filter and `query`. Returns `Verified` if any
    result has `score > 0.5`; `Unverified` otherwise.
    Failure mode: if artifact `name` doesn't exist in manifest,
    return `MalformedGrounding { reason: "artifact not declared" }`.

11. Wire ArtifactCheck into Designer's plan grounding sweep
    (Phase 2B):
    - Designer's plan claims like "use snake_case for column names"
      should reference `artifact:database-schema` if it exists.
    - GroundingSweep gate uses ArtifactCheck to verify.
    - Block transition if a project-relevant claim has no artifact
      backing AND no other grounding source.

12. Auto-inject artifact descriptions into Designer briefing:
    - When task is in TechStackDecision::ExistingProject, inject
      `AVAILABLE_ARTIFACTS` block with `[name, description]` list.
    - Designer prompt: "Before making decisions about [database |
      API | infrastructure | domain], search the corresponding
      artifact via search_artifacts."

### Phase 5 — Dashboard surface (Session 2, half)

13. Add `ArtifactsPanel.svelte` to dashboard. Displays:
    - List of declared artifacts (name, path, description, status,
      chunk count, last indexed)
    - Status badges: `NotIndexed` / `Indexed` / `Stale` / `Indexing`
    - "Re-index" button per artifact (calls index_artifacts)
    - "Search" inline form (calls search_artifacts with that artifact
      filter)
    - Show first 200 chars of description per artifact (steering text
      visible to user)

14. Add Tauri commands:
    - `list_project_artifacts(project_id) -> Vec<ArtifactWithStatus>`
    - `search_project_artifact(project_id, name, query) -> Vec<ArtifactSearchResult>`
    - `reindex_project_artifact(project_id, name) -> Result<()>`
    - `remove_project_artifacts(project_id) -> Result<()>`

15. SSB action types:
    - `ArtifactsIndexed { project_id, indexed: Vec<String>, up_to_date: Vec<String>, total_chunks: usize, latency_ms: u64 }`
    - `ArtifactSearched { project_id, artifact_name: Option<String>, query, result_count, latency_ms }`
    - `ArtifactStale { project_id, artifact_name, changed_files: Vec<String> }`

16. ActivityFeed surfaces these for visibility.

### Phase 6 — Documentation + examples (Session 2, half)

17. Add example artifact manifest to `docs/`:
    - `docs/examples/elai_cc.example.toml` showing
      `[[knowledge.artifacts]]` blocks for SQL schema, OpenAPI, k8s,
      Terraform, ADR markdown, domain glossary
    - Inline comments showing "description" field convention: write as
      "check this before doing X"

18. Update `docs/specs/guide/dashboard-design.txt` (if mutable) with
    `ArtifactsPanel` mockup.

## WIRING_CONTRACT

produces_symbols:
  - knowledge::artifacts::manifest::ArtifactManifest::load
  - knowledge::artifacts::types::{ArtifactSpec, ArtifactChunk,
    ArtifactIndexState, ArtifactStatus}
  - knowledge::artifacts::indexer::{index_artifact, index_all_artifacts,
    detect_staleness}
  - knowledge::artifacts::search::artifact_search
  - knowledge::pipeline::stages::ArtifactStage
  - reasoning::grounding::GroundingMethod::ArtifactCheck (new variant)
  - core::ssb::SsbActionType::{ArtifactsIndexed, ArtifactSearched,
    ArtifactStale} (3 new variants)
  - dispatcher tool entries: list_artifacts, index_artifacts,
    search_artifacts, remove_artifacts
  - dashboard::commands::{list_project_artifacts,
    search_project_artifact, reindex_project_artifact,
    remove_project_artifacts}
  - Svelte component: ArtifactsPanel.svelte
consumes_symbols:
  - orchestrator::indexing_service::IndexingService::build_index → calls
    index_all_artifacts after code index completes
  - orchestrator::indexing_service::IndexingService::update_index → calls
    detect_staleness + per-artifact re-index
  - knowledge::pipeline::query_pipeline → ArtifactStage between CatRAG
    and A-RAG in STANDARD + DEEP tiers
  - reasoning::grounding::verify_claim → dispatches ArtifactCheck via
    search_artifacts tool
  - orchestrator::task_runner::run_designer_briefing → injects
    AVAILABLE_ARTIFACTS block when TechStackDecision is ExistingProject
reachable_from:
  - cli::main::handle_index_command (auto-trigger on /index)
  - cli::main::handle_research_command (auto-trigger on /research)
  - dashboard::commands::submit_task (manual trigger via UI)
  - orchestrator::indexing_service::IndexingService::build_index
stub_ban:
  - ArtifactStatus is always one of NotIndexed | Indexed | Stale {
    changed_files } | Indexing (no None / Unknown)
  - Description text is non-empty when artifact is declared (validation
    at manifest load time)
  - context_{project_id} collection exists when artifacts are indexed
    (verified via SQLite check)
  - GroundingMethod::ArtifactCheck.artifact_name validates against
    declared manifest before dispatch (no silent ungrounded result)
frontend_pair: B-29 (artifacts feed Designer briefing alongside symbol
  cache)
observable_effect:
  trigger: declare an artifact in elai_cc.toml referencing a real .sql
    file; run "elai.bat repl '/index'"
  expect_ssb: ArtifactsIndexed action with the artifact name in indexed
    list and total_chunks > 0
  expect_artifact: subsequent "elai.bat repl '/search_artifacts users
    --artifact_name database-schema'" returns chunks containing CREATE
    TABLE users from the .sql file

## DETERMINISTIC_CHECK

```bash
# 1. Manifest parses
elai.bat repl "/list_artifacts"
# Expected: lists declared artifacts with NotIndexed status before first
# /index.

# 2. Indexing populates context collection
elai.bat repl "/index"  # auto-triggers artifact index
sqlite3 .elai_cc/.elai_cc/indexes/context_<project_id>.db \
  "SELECT artifact_name, COUNT(*) FROM context_chunks GROUP BY artifact_name;"
# Expected: per-artifact chunk counts > 0.

# 3. Staleness detection
echo "-- new comment" >> docs/schema.sql
elai.bat repl "/index"
# Expected SSB: ArtifactStale event for database-schema, then
# ArtifactsIndexed re-running for that artifact only.

# 4. Search returns relevant chunks
elai.bat repl "/search_artifacts users --artifact_name database-schema"
# Expected: returns chunks containing CREATE TABLE users.

# 5. ArtifactCheck grounding works
# Trigger Designer in ExistingProject mode:
elai.bat repl "/task 'add a deleted_at column to the users table'"
# Designer's plan grounding sweep should issue ArtifactCheck against
# database-schema with query "users table". Verify SSB log:
sqlite3 .elai_cc/.elai/sessions/ssb.db \
  "SELECT * FROM ssb WHERE action_type='ArtifactSearched' ORDER BY ts DESC LIMIT 5;"

# 6. Dashboard panel renders
# Open dashboard → Artifacts tab. Verify list of declared artifacts with
# correct status and chunk counts. Click "Search" inline; verify
# results display.

# 7. Remove cleanup
elai.bat repl "/remove_artifacts"
sqlite3 .elai_cc/.elai_cc/indexes/context_<project_id>.db ".tables"
# Expected: context_chunks table empty or dropped.

# 8. Failure mode: missing artifact path
# Add an artifact to elai_cc.toml pointing to /tmp/does-not-exist
elai.bat repl "/index"
# Expected: warn log, ArtifactsIndexed action with that artifact name
# in failed list, no panic.

# 9. Security: path escape rejected
# Add an artifact with path "../../etc/passwd"
elai.bat repl "/index"
# Expected: manifest load fails with "path resolves outside project root".
```

## ACCEPTANCE

- [ ] `[[knowledge.artifacts]]` block parsed from elai_cc.toml
- [ ] ArtifactSpec with name regex `[a-zA-Z0-9_-]+`, path bounds-checked
- [ ] Chunking reuses B-32 chunker; SQL gets CREATE-statement-boundary
      chunking
- [ ] blake3 staleness detection (CLAUDE.md anti-stale rule)
- [ ] Per-project context_* SQLite collection created on first
      artifact index
- [ ] ArtifactStage in QueryPipeline (STANDARD + DEEP tiers)
- [ ] 4 dispatcher tools: list / index / search / remove artifacts
- [ ] 4 Tauri commands wired to ArtifactsPanel.svelte
- [ ] 3 SSB action types appended; visible in ActivityFeed
- [ ] GroundingMethod::ArtifactCheck implemented; Designer plan sweep
      uses it for project-relevant claims
- [ ] Designer briefing includes AVAILABLE_ARTIFACTS block when
      ExistingProject decomposition fires
- [ ] Auto-staleness on code update path: changed artifact files
      re-index transparently
- [ ] Failure non-fatal: artifact errors never block code indexing
- [ ] Path security: paths outside project_root rejected at manifest
      load
- [ ] Lock interaction: artifacts coexist with code indexing without
      collision
- [ ] Example manifest committed at `docs/examples/elai_cc.example.toml`
- [ ] Zero `.unwrap()` in non-test source per CLAUDE.md hard rule
- [ ] No new external crates without explicit user confirmation

## Tests to add

- `crates/knowledge/tests/artifact_manifest.rs`:
  - parse valid manifest with 5+ artifacts
  - reject path resolving outside project_root (security)
  - reject invalid name regex
  - support both single-file and directory paths
- `crates/knowledge/tests/artifact_indexer.rs`:
  - SQL file → chunked at CREATE TABLE boundaries
  - YAML / JSON file → line-window chunked
  - Markdown file → line-window chunked
  - Each chunk has artifact_name + description fields populated
- `crates/knowledge/tests/artifact_staleness.rs`:
  - First index: all files indexed
  - No-op re-index: nothing changed, all skipped
  - Modify one file: only that file re-indexed
  - Add new file in directory artifact: only new file indexed
  - Remove file from directory: chunks for that file dropped
- `crates/knowledge/tests/artifact_search.rs`:
  - search across all artifacts (no filter)
  - search filtered to single artifact_name
  - search returns score + chunk content + artifact metadata
- `crates/reasoning/tests/grounding_artifact_check.rs`:
  - ArtifactCheck verifies when search returns results above threshold
  - returns MalformedGrounding when artifact_name not declared
  - returns Unverified when no results above threshold
- `crates/orchestrator/tests/designer_artifact_briefing.rs`:
  - ExistingProject decomposition injects AVAILABLE_ARTIFACTS block
  - Greenfield decomposition skips it
- `crates/dashboard-v2/src-tauri/tests/artifacts_panel_e2e.rs` (V2 path
  — was V1 `crates/dashboard/tests/`):
  - ArtifactsPanel renders manifest + status
  - "Re-index" button triggers indexing, status updates live
- `tests/contracts/artifact_pipeline_contract.rs`:
  - regression: code indexing must not be blocked by artifact failures
  - regression: context_* collection always isolated from
    codebase_/codegraph_ collections (project boundary preserved)

## Files touched

- CREATE: `crates/knowledge/src/artifacts/mod.rs`
- CREATE: `crates/knowledge/src/artifacts/manifest.rs`
- CREATE: `crates/knowledge/src/artifacts/types.rs`
- CREATE: `crates/knowledge/src/artifacts/indexer.rs`
- CREATE: `crates/knowledge/src/artifacts/search.rs`
- CREATE: `crates/knowledge/src/artifacts/staleness.rs`
- CREATE: `crates/knowledge/src/artifacts/store.rs` (SQLite collection
  ops)
- CREATE: `crates/knowledge/src/pipeline/stages/artifact_stage.rs`
- MODIFY: `crates/knowledge/src/pipeline.rs` (insert ArtifactStage in
  STANDARD + DEEP tier dispatch)
- MODIFY: `crates/knowledge/src/lib.rs` (export `artifacts` module)
- MODIFY: `crates/orchestrator/src/indexing_service.rs` (call
  index_all_artifacts after code index; per-watcher staleness)
- MODIFY: `crates/orchestrator/src/dispatcher.rs` (4 new tool entries)
- MODIFY: `crates/orchestrator/src/task_runner.rs`
  (AVAILABLE_ARTIFACTS Designer briefing block)
- MODIFY: `crates/reasoning/src/grounding/verify.rs`
  (GroundingMethod::ArtifactCheck variant + dispatch)
- MODIFY: `crates/reasoning/src/grounding/sweep.rs` (Designer plan
  sweep uses ArtifactCheck for project-typed claims)
- MODIFY: `crates/cli/src/main.rs` (4 new slash commands:
  /list_artifacts, /index_artifacts, /search_artifacts,
  /remove_artifacts)
- MODIFY: `crates/dashboard-v2/src-tauri/src/lib.rs` (V2 — was V1
  `crates/dashboard/src/commands.rs`; 4 new Tauri commands:
  list_artifacts / index_artifacts / search_artifacts / remove_artifacts.
  B-36 swaps the corresponding stub.ts entries to live invokes)
- CREATE: `crates/dashboard-v2/src/lib/components/ArtifactsPanel.svelte`
  (V2 path — was V1 `crates/dashboard/svelte/src/panels/`)
- MODIFY: `crates/dashboard-v2/src/lib/screens/Workspace.svelte` (V2 —
  was V1 `crates/dashboard/svelte/src/Workspace.svelte`; mount
  ArtifactsPanel in left sidebar or KnowledgeDrawer.svelte)
- MODIFY: `crates/core/src/ssb/action_types.rs` (3 new variants)
- CREATE: `docs/examples/elai_cc.example.toml`
- MODIFY: `docs/specs/guide/dashboard-design.txt` (add
  ArtifactsPanel section if file is mutable; otherwise skip)
- CREATE: `workflow/reasoning/B-35-decisions.md`

## Decision doc

Write `workflow/reasoning/B-35-decisions.md` documenting:
- Manifest format choice (TOML vs JSON). Recommend TOML primary, allow
  `.elai_artifacts.json` fallback for SocratiCode-import compatibility
- Whether to import SocratiCode-style `.socraticodecontextartifacts.json`
  if user has one (transparent compatibility — yes/no?). Default no
  (avoid name conflict during file watching)
- Chunk strategy for SQL files: CREATE-statement-boundary regex vs
  generic line-window. Recommend regex for SQL specifically since the
  CREATE blocks are the natural retrieval unit
- Lock scope: shared `LockScope::Index` (artifacts can't index alongside
  code) vs new `LockScope::Artifacts` (parallel allowed). Recommend
  shared — embedding provider rate limit shared anyway
- Whether ArtifactStage runs in FAST tier. Recommend NO — FAST is
  BM25-only, already constrained. STANDARD + DEEP only
- ArtifactCheck threshold: score > 0.5 vs configurable per-artifact.
  Default 0.5; allow per-artifact override in manifest if needed
- Whether artifacts should produce DKG nodes too (for cross-artifact
  graph). Recommend NOT — keep DKG code-only; artifacts have their own
  search surface
- How auto-staleness interacts with B-31's checkpoint pattern: artifact
  state is per-artifact, not per-batch (artifacts are typically <100
  files; whole-artifact rebuild is fine)
- Default artifact set for elai_cc.toml.example: SQL schema, OpenAPI,
  domain glossary, k8s manifests, Terraform, ADR — pick 5 examples that
  cover the highest-value cases for typical projects

## Side benefits unlocked

- Designer's TechStackDecision::ExistingProject finally has data
  beyond code
- /research command can ground questions about database / API /
  infrastructure with authoritative answers
- PipelineGrc grounding verification gains a 7th method that catches
  project-internal hallucination (Designer claims "users table has
  email column" — ArtifactCheck against database-schema verifies)
- DDD projects benefit massively — ubiquitous-language artifact prevents
  Designer from inventing terms outside the bounded context vocabulary
- Empire integration (B-25) gains a path: Empire's brain protocol specs
  can be declared as artifacts; ELAI's PipelineGrc verifies brain
  configurations against the spec
- Self-modification (B-23) Layer 2 proposals can use artifact data —
  e.g. propose a new rule based on observed schema changes
- The dashboard ArtifactsPanel provides immediate visible value to
  users: "I can see what knowledge ELAI has about my project"
- Foundation for compliance-aware Designer: SOC2/HIPAA/GDPR docs
  declared as artifacts; Designer's plan sweep verifies compliance
  claims

## Next

B-29 (Symbol-Level Impact API) — independent; both feed Designer
  context.
B-31 (Resumable Indexing) — required dependency for lock primitives.
B-34 (Entry-Point Detection) — independent; complementary Designer
  context source.
B-23 (Self-Modification 4H) — could later use artifact data as Layer 2
  proposal fuel.
