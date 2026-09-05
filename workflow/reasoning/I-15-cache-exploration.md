SUBSYSTEM: knowledge+cli+ssb-payload
TAGS: CROSS_BOUNDARY
INTEGRATION QUESTION: Map all insertion points, call chains, payload anchors, badge IPC surface, and confinement risks for I-15 semantic cache + blake3 fast path.
NOTE: written to workflow/reasoning/ because FSM EXECUTING state blocked workflow/sessions/s-107b022a/exploration/. Controller must copy or symlink to the canonical path.
EXPLORED FILES: 7 | arag.rs, oracle_cache.rs, ssb.rs, repl_command.rs, h8_oracle_persist_wiring.rs, commands.rs, stub.ts
ENTRY POINTS: knowledge/src/arag.rs, orchestrator/src/oracle_cache.rs, cli/src/repl_command.rs
CONFIDENCE: 0.96

INTEGRATION SURFACE

AragOrchestrator::semantic_search(&self, query:&str, top_k:usize)->Result<Vec<RetrievalResult>,KnowledgeError>
  private, called via execute_tool(RetrievalTool::SemanticSearch). ONNX path entry: arag.rs:298 match self.embedder.as_ref().
  INSERTION POINT: top of semantic_search, after is_empty() early-return at arag.rs:292, BEFORE line :298.
  This is the single chokepoint for all semantic search calls regardless of caller.

AragOrchestrator::query_pipeline(&self, query:&str, _snapshot:&DkgSnapshot)->Result<Vec<RetrievalResult>,KnowledgeError>
  public, arag.rs:359. Dispatches semantic via execute_tool(SemanticSearch) at arag.rs:379. Does NOT hold any cache reference today.

AragChunk: chunk_id:String, content:String, embedding:Vec<f32>
  materialized via add_chunk(arag.rs:153) and populate_corpus_from_dkg(arag.rs:560-583).
  chunk_id="node_{node_id}" for DKG chunks. content="{kind_str} {symbol_name} {signature}" (arag.rs:570-573).
  These ARE the cacheable intermediate artifacts — no separate summary generation exists in arag.rs.

OracleAnswerCache::get(&self, key:&CacheKey, now_secs:u64)->Option<CachedAnswer> | oracle_cache.rs:328
OracleAnswerCache::insert(&self, key:CacheKey, entry:CachedAnswer)->bool | oracle_cache.rs (inferred from map rows)
CacheKey::compute(query:&str, project_root:&Path)->CacheKey | oracle_cache.rs:408 — already O(1); I-15 reuses verbatim
OracleCacheHitPayload | ssb.rs:17394 | fields: query_hash_hex, source_tier, cached_age_secs — no semantic_cache/source field today
OracleCacheMissPayload | ssb.rs:17410 | fields: query_hash_hex, confidence — no semantic_cache/source field today
SsbPayload::OracleCacheHit(OracleCacheHitPayload) | ssb.rs:13407
SsbPayload::OracleCacheMiss(OracleCacheMissPayload) | ssb.rs:13408

SEAM ANALYSIS

OPTION A — seam at semantic_search (RECOMMENDED, cheapest):
  AragOrchestrator gains field cache:Option<Arc<OracleAnswerCache>> + builder with_cache(c)->Self
  (mirrors existing embedder:Option<Arc<OrtSession>> + with_embedder pattern at arag.rs:130/147).
  Guard block inserted at arag.rs:292 before match self.embedder.
  project_root baked at construction time (stored in AragOrchestrator, set via with_cache).
  Construction site: build_local_knowledge_stack (live_cli.rs) — same Arc threaded to both CLI-level oracle_cache and AragOrchestrator::with_cache().
  Plumbing cost: 3 new fields/builders in arag.rs + 1 construction site change in live_cli.rs + guard block.
  CRITICAL DEP-CYCLE RISK: knowledge crate must NOT already depend on orchestrator for this to work.
  If cycle: use trait AragExactCache in knowledge crate, implemented by OracleAnswerCache in orchestrator.

OPTION B — seam at query_pipeline (NOT RECOMMENDED):
  Threads cache through public query_pipeline signature. Forces knowledge->orchestrator dep or requires
  an injected trait. Higher blast radius. Reject unless Option A blocked by dep cycle.

OPTION C — CLI-only (NOT SUFFICIENT alone):
  repl_command.rs:315 already IS the query-level exact-match path (C-30). I-15 requires ONNX skip
  at the arag layer too (WIRING_CONTRACT stub_ban: no embedder.embed() before blake3 check).
  Option C alone does not satisfy the stub_ban invariant.

DEP CHECK (must be done by executor before editing):
  grep -r "elai-orchestrator" crates/knowledge/Cargo.toml
  If absent -> use trait abstraction. If present -> direct Arc<OracleAnswerCache> is safe.

CHUNK-SUMMARY CACHE VERDICT: QUERY-KEYED SESSION-STABLE IN-MEMORY ONLY

DKG chunk content (kind+symbol+signature) changes on reindex. oracle_cache has NO invalidation hook
(confirmed: no callback, no signal from IndexingService). Durable disk caching of chunk summaries
WOULD serve stale content post-reindex with no recovery path.
VERDICT: intermediate_summary_cache = in-memory FxHashMap<([u8;32], String), Vec<RetrievalResult>>
inside AragOrchestrator, keyed on (blake3(query_bytes), chunk_id), lifetime = AragOrchestrator instance.
NEVER inserted into OracleAnswerCache::persisted_default().
This is the safe, narrow scope. The disk-backed oracle_cache is for research ANSWERS only.

SSB PAYLOAD ANCHORS + LIVE SCHEMA

SCHEMA_VERSION: 232 (ssb.rs:3087, verified live 2026-06-12)
TOTAL_VARIANTS: 542 (action_type_lock.rs:671, verified live 2026-06-12)
(context-package 221/532 was stale by multiple sessions — use 232/542)

Field-add path (GD-1, cheapest): add pub source:Option<String> + #[serde(default)] to both
OracleCacheHitPayload (ssb.rs:17394) and OracleCacheMissPayload (ssb.rs:17410).
Values: "blake3_exact" | "semantic_vector" | None (existing C-30 path).
NO schema version bump, NO action_type_lock change, NO golden re-lock (payload-extension-only pattern).
Fallback: +SemanticCacheEvent new variant (+2 TV, bump 232->233, full 9-tuple lockstep).

EMIT SITES

OracleCacheHit emit: repl_command.rs:323 (existing hit branch from oracle_cache.get() at :315)
  I-15 blake3 exact hit at arag layer must surface source="blake3_exact" to this emit.
  Threading path: RetrievalResult.tool_used="blake3_cache_hit" (already a String field) propagates
  up through execute_tool -> query_pipeline -> QueryPipeline -> ResearchOrchestrator to repl_command.rs.
  Executor must verify QueryPipeline passes tool_used metadata through its result aggregation.
OracleCacheMiss emit: repl_command.rs:507 (after insert at :501)
  source="semantic_vector" when pipeline ran but no exact match.

BADGE LEG (cross-language pair)

ResearchDrawer.svelte: polls get_research_node_history (commands.rs:10703), get_research_budget (commands.rs:2008), get_source_tier_distribution.
NO oracle_cache_hit or source field display today.
Minimal path: extend get_research_node_history row projection in SsbViewer::research_node_stages to
surface oracle_cache_source from payload_json. No new Tauri command needed if SSB row carries full payload.
Alternative new command: get_research_cache_source(query_hash_hex)->String; requires commands.rs fn +
generate_handler! at lib.rs (pattern: lib.rs:643/706/912) + stub.ts export + ResearchDrawer.svelte chip.
Cross-lang pair: commands.rs Tauri fn + stub.ts tryInvoke + optional types.ts extension.

CONFINEMENT REGRESSION GUARDS

h8_oracle_persist_wiring.rs:28 oracle_cache_is_disk_backed: asserts oracle_cache.rs contains
"persisted", "persist_to_disk", "oracle-cache". I-15 must NOT remove any of these. SAFE under Option A.

h8_oracle_persist_wiring.rs:43 research_arm_uses_persisted_cache: asserts repl_command.rs contains
"persisted_default" or "persisted(". I-15 must NOT remove persisted_default() at repl_command.rs:302. SAFE.

CONFINEMENT RISK — SECOND DISK-BACKED CTOR: LOW.
Option A passes the same Arc<OracleAnswerCache> (constructed once in CLI) into AragOrchestrator via
with_cache(). The arag.rs code only calls get()/insert() on the handed-in Arc; it never calls
persisted_default() itself. Single-ctor confinement preserved IF the CLI threads the same Arc (not a new instance).
The in-memory chunk-summary FxHashMap never calls persisted_default() — no confinement risk there.
EXECUTOR MUST: verify live_cli.rs constructs ONE oracle_cache Arc and reuses it for both the
CLI-level check and the AragOrchestrator::with_cache() call. The current lazy get_or_insert_with at
repl_command.rs:302 creates the Arc per-dispatch; for AragOrchestrator the Arc must be available at
build_local_knowledge_stack time (before dispatch). This may require hoisting the oracle_cache
construction out of get_or_insert_with into a once-at-init pattern.

PATTERNS TO FOLLOW

blake3_32 hasher pattern | oracle_cache.rs:413-419 | blake3::Hasher::new()->update()->finalize()->copy_from_slice
Option<Arc<T>> with with_X() builder | arag.rs:130/147 (embedder) | cache field mirrors this shape
#[serde(default)] payload extension | ssb.rs payload structs | Option<String> source field, no variant bump
persisted_default() sole construction | repl_command.rs:302 | never construct a second disk-backed instance

EXISTING WIRING

cli/src/repl_command.rs:302 -> OracleAnswerCache::persisted_default() | sole disk-backed ctor lazy init
cli/src/repl_command.rs:308 -> CacheKey::compute(q, project_root) | oracle key before get()
cli/src/repl_command.rs:315 -> oracle_cache.get(&oracle_key, now_secs) | query-level hit; OracleCacheHit at :323
cli/src/repl_command.rs:501 -> oracle_cache.insert(oracle_key, cached_entry) | sole insert; OracleCacheMiss at :507
knowledge/src/arag.rs:292 | is_empty() early-return | INSERTION POINT just below this line
knowledge/src/arag.rs:298 | embedder.embed(query) | ONNX call to short-circuit
knowledge/src/arag.rs:560 | populate_corpus_from_dkg() | DKG->AragChunk materialization anchor
orchestrator/src/oracle_cache.rs:413 | blake3_32() helper | hash utility for cache key

SEE-ALSO

crates/knowledge/src/pipeline.rs:1037 | QueryPipeline downstream caller of query_pipeline; must thread cache Arc if Option B chosen | pipeline scope
crates/cli/src/live_cli.rs | build_local_knowledge_stack: sole AragOrchestrator production construction site; gains with_cache() call | cli scope
crates/knowledge/Cargo.toml | dep-cycle check before adding Arc<OracleAnswerCache> to arag.rs | executor must grep before editing

GAPS / RISKS

knowledge->orchestrator dep cycle | AragOrchestrator + Arc<OracleAnswerCache> requires knowledge to depend on orchestrator; may need trait abstraction | HIGH — verify first
DKG chunk-summary invalidation gap | oracle_cache has no invalidation hook; IndexingService emits no usable signal on reindex | confirmed absent — chunk summaries = session-stable only
source discriminator threading gap | no metadata field survives from arag.rs RetrievalResult up to repl_command.rs SSB emit sites; tool_used is the cheapest carrier | MEDIUM
oracle_cache Arc hoisting | repl_command.rs:302 lazy get_or_insert_with creates Arc per-dispatch; AragOrchestrator needs the Arc at construction time; may require build_local_knowledge_stack to construct oracle_cache eagerly | MEDIUM

DRIFT

context-package SCHEMA/TV "221/532" stale | live: 232/542 | HIGH — use 232/542 as baseline
no drift on OracleCacheHitPayload/OracleCacheMissPayload field absence (confirmed: no source/semantic_cache field in either struct)
