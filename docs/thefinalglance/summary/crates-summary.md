ELAI CRATE SPECS — LOSSLESS SUMMARY
Source: F:\.ELAI_workflow\docs\specs\crates\*.txt
Project code root: F:\.ELAI_workflow\.elai_cc\crates\
Audience: downstream audit agents, plan writers.

==========================================================================

## CRATE: core/

### 1. RESPONSIBILITY
Leaf crate providing config parsing (elai_cc.toml), cross-platform paths, tracing-based logging, serde (msgpack/postcard), heapless structures, platform/capability detection, deterministic entropy quarantine (ClockProvider/EntropySource), and HashMap quarantine.

### 2. DEPENDENCIES
None internal (leaf). External: serde, serde_json, crossterm (KEEP); ADD rmp-serde, postcard, heapless, tracing, tracing-subscriber, dirs, uuid, toml, rand (behind EntropySource trait only).

### 3. PUBLIC TRAITS
```
trait ClockProvider {
    fn now(&self) -> LogicalInstant;
    fn elapsed_since(&self, mark: LogicalInstant) -> Duration;
    fn system_now(&self) -> SystemTime;  // for cache expiry, HTTP headers
}
trait EntropySource {
    fn next_u64(&mut self) -> u64;
    fn fill_bytes(&mut self, buf: &mut [u8]);
}
```
Note: `&self` on ClockProvider (read-only); `&mut self` on EntropySource (stateful RNG).

Serialization wrapper traits: `MsgpackSerialize`, `MsgpackDeserialize` (rmp-serde), `PostcardSerialize`, `PostcardDeserialize`.

### 4. PUBLIC TYPES
- `Config` (parsed from elai_cc.toml): project settings, agent role configs, GRC budgets per role, rule sets, model assignments, provider configs, hardware detection settings.
- `SessionId` format: `{project_id}:{pid}:{timestamp_ms}` (timestamp eliminates Windows PID reuse).
- `RuntimeContext<C: ClockProvider, E: EntropySource> { clock: C, entropy: E }` — static dispatch, no vtable, no Mutex on entropy. Accessors: `clock(&self) -> &C`, `entropy_mut(&mut self) -> &mut E`.
- Type aliases: `ProductionContext = RuntimeContext<OsClock, OsEntropy>`, `TestContext = RuntimeContext<SeededClock, SeededRng>`.
- Implementations: `OsClock` (delegates to Instant), `OsRng` (thread_rng), `SeededClock` (logical, tick-based, monotonic), `SeededRng` (deterministic PRNG).
- `TokenUsage { input_tokens, output_tokens }` (replaces deleted UsageTracker/CompactionConfig/billing).
- `SandboxConfig`, `FilesystemIsolationMode` — moved here from runtime/sandbox.rs because config.rs imports them.
- ONNX model storage: canonical path `.elai_cc/models/onnx/`. Models pre-staged via setup script; missing = DEGRADED, not crash. Hidden from agents via `.elai_cc/models/**` in hidden_paths.

### 5. ERROR TYPES
Not enumerated in spec.

### 6. SQLITE SCHEMAS
None.

### 7. CONFIG KEYS
- `elai_cc.toml` is the canonical config file (root parsed by core).
- `[planning.sizing.default_avg_file_tokens]` default 150 (used by orchestrator SystemConstraints fallback).
- `[planning.sizing.default_avg_test_tokens]` default 80.
- All other agent role / GRC / rule / model / provider / hardware sections live within Config (referenced from other crates).

### 8. SECTIONS
S1 RESPONSIBILITIES; S2 CLAW-CODE MODULE MAPPING; S3 STRIP/REPLACE; S4 KEY TYPES (Config, SessionId, Serialization traits, Deterministic entropy traits, HashMap determinism quarantine, ONNX model storage, SandboxConfig); S5 DEPENDENCIES; S6 TESTABLE INVARIANTS.

### 9. PHASE OWNERSHIP
- Phase 0: entropy quarantine (ClockProvider/EntropySource traits + impls + CI grep enforcement).
- Phase 0.8: HashMap determinism quarantine extension (BTreeMap/IndexMap/FxHashMap; clippy.toml `disallowed-methods` for `HashMap::new`, `HashSet::new`, `HashMap::with_capacity`, `HashSet::with_capacity`).
- Phase 0.9: prompt_cache.rs (4 SystemTime::now), anthropic.rs (2 SystemTime::now) must use ClockProvider.

### 10. KNOWN GAPS / TODOs / DEFERRED
- FIND AND REMOVE: SYSTEM_PROMPT_DYNAMIC_BOUNDARY (L39), "You are an interactive agent" (L451), compact.rs + summary_compression.rs, telemetry crate (526 LOC), TokenUsage/UsageTracker/CompactionConfig/billing (61 occurrences across 5 files), compat-harness/, lsp_client.rs (747 LOC).
- NOTE: "sentinel", "anti_distill", "bias_inject" do NOT exist in claw-code (verified 2026-04-06).
- Production exception sites (ONLY raw time/entropy): `core/src/clock.rs` (OsClock), `core/src/entropy.rs` (OsRng).
- Enforced by CI grep script (not clippy deny — custom lints require dylint/nightly).
- Pre-restructure scope: 60 occurrences across 31 files (providers/prompt_cache.rs 4 SystemTime::now, tools/lib.rs 14 Instant::now, cli/main.rs 5, providers/anthropic.rs 2, orchestrator/session_control.rs 3).
- Source for HashMap quarantine: deterministic-simulation-testing.md section 8.1.
- Third-party crate internals exception (madsim patches SipHash seed).

### Testable invariants
Config round-trip; SessionId 10K loop zero collisions; cross-platform path equivalence; msgpack < json size; tracing captures every tool/file op; no direct Instant::now()/thread_rng() (clippy enforced); seeded clock+rng deterministic; no HashMap/HashSet (clippy enforced); deterministic map iteration.

==========================================================================

## CRATE: providers/

### 1. RESPONSIBILITY
LLM provider trait + implementations (Anthropic, OpenAI, Ollama, VllmLocal, LlamaCpp), per-model layer optimizations, ModelPool VRAM lifecycle (load/unload/predictive pre-load), node-level cache, assistant prefill continuation.

### 2. DEPENDENCIES
core/. External KEEP: reqwest, serde. ADD: nvml-wrapper or sysinfo, xxhash. Phase 2+: vLLM HTTP/gRPC client.

### 3. PUBLIC TRAITS
```
trait LlmProvider {
    fn complete(prompt, config) -> Stream<Token>;
    fn embed(text) -> Vec<f32>;
    fn token_count(text) -> usize;          // exact, real tokenizer
    fn token_count_estimate(text) -> usize; // sampling-based, O(lines/N)
}
```
Sampling estimation: every Nth line (N=10 default), real tokenizer × N. Accuracy 95%+ for files >100 lines. Exact for files ≤N lines.

### 4. PUBLIC TYPES
- `OllamaConfig`: temperature, top_p, grammar constraints (constrained decoding).
- `VllmConfig` (Phase 2+): DoLa, TurboQuant, steering vectors.
- `OnnxModelStatus { model_id, state: HEALTHY | DEGRADED | UNAVAILABLE }`.
- `CachePolicy { key_fn, ttl_seconds }`. `CacheKey`: xxhash after normalization. Float normalization: NaN → u64::MAX, -0.0 → 0.0, round to 6 decimals. In-memory `HashMap<CacheKey, (Output, Instant)>` per node, NOT checkpointed.
- ModelPool tracks: `{model_id -> ModelState(Loaded(gpu_id, vram_used) | Unloaded | Loading)}`.
- VRAM query: nvml-wrapper (NVIDIA) or sysinfo (cross-platform fallback).
- Priority levels: ACTIVE (never unloaded), PRELOAD (background load if VRAM allows), IDLE (unloaded first on pressure).
- SLO quality tiers (Murakkab arxiv 2508.18298): BEST (auditor, researcher), GOOD (executor, context-compiler), FAIR (prompt-engineer), BASIC (memory-writer). Stored in `[agents.{role}.slo_tier]`.

### 5. ERROR TYPES
Not enumerated in spec; uses provider-specific error mapping.

### 6. SQLITE SCHEMAS
None directly (uses cache file structures).

### 7. CONFIG KEYS
- `[providers.ollama.speculative]` `draft_model`, `num_draft_tokens` (default 5).
- `[agents.{role}.tier2_mode]`: `GRAMMAR | INSTRUCTOR | AUTO`.
  - Defaults: executor (Gemma 4) GRAMMAR; auditor (R1-Distill) INSTRUCTOR; designer (Gemma 4) GRAMMAR; prompt-engineer (Qwen) GRAMMAR.
- `[agents.{role}.slo_tier]`: BEST | GOOD | FAIR | BASIC.
- Ollama integration paths: `POST /api/generate` (load), `DELETE /api/delete` (unload). KEEP_ALIVE controls auto-unload. Dual-GPU: CUDA_VISIBLE_DEVICES per ollama instance or `--gpu` flag.

### 8. SECTIONS
S1 RESPONSIBILITIES; S2 LLMPROVIDER TRAIT; S3 LAYERCONFIG (Phase 0-1 ollama, Tier 2 enforcement modes GRAMMAR/INSTRUCTOR/AUTO, Phase 1+ speculative decoding, Phase 2+ vLLM); S4 MODELPOOL (priority, SLO tiers, GPU multiplexing, predictive pre-loading, workload profiling, ollama integration, swap protocol, vLLM integration); S5 NODE-LEVEL CACHE; S6 ASSISTANT PREFILL CONTINUATION; S7 CLAW-CODE MODULE MAPPING; S8 DEPENDENCIES; S9 ONNX MODEL HEALTH PROTOCOL; S10 TESTABLE INVARIANTS.

### 9. PHASE OWNERSHIP
- Phase 0-1: ollama only.
- Phase 0 dual-trait: old Provider trait kept #[deprecated]; OllamaProvider implements LlmProvider directly. AnthropicClient/OpenAiCompatClient continue old trait. Migration to LlmProvider in Phase 1+.
- Phase 0.9: prompt_cache.rs SystemTime::now → ClockProvider; anthropic.rs SystemTime::now → ClockProvider; sse.rs (api/ keeps 279 LOC version, strip runtime/ 158 LOC).
- Phase 1+: speculative decoding (EVALUATE: adopt if >1.3x throughput on RTX 3060; disabled by default for auditor).
- Phase 2+: vLLM integration; multiplexing on GPU1 for small models; MILP scheduling EVALUATE (coin-or/cbc or highs).

### 10. KNOWN GAPS / TODOs / DEFERRED
- DoLa/TurboQuant require vLLM or custom inference (CRITICAL: ollama is a black box).
- INSTRUCTOR mode worst case ~66s for one auditor response on RTX 3060 at 3 retries.
- GBNF limitations enforced at schema compile time: max 3 nesting levels (deeper = SIGSEGV in ollama GBNF), no recursive `$ref`, no uniqueItems/contains/not/if/then/else, regex requires `^` and `$` anchors, additionalProperties=false default. Source: llm-json-schema-generation.md section 2.2.
- TIDE pattern (temporal incremental draft) — 1.15x over static speculative decoding.
- vLLM MILP scheduling: implement only if heuristic insufficient in Phase 2 metrics.
- Tools/lib.rs imports `api::*` (line 6) → must update to `providers::` during restructure.

### Detail: Tier 2 enforcement (compact)
- GRAMMAR: GBNF/grammar at generation time. Use for Gemma 4 (delimiter tokens). NOT for DeepSeek-R1-Distill or any model requiring CoT traces.
- INSTRUCTOR: free generation + post-hoc JSON schema validation + retry. Max 3 retries. On exhaustion: ESCALATE to human with last valid JSON fragment or empty finding set + error description. Use for DeepSeek-R1-Distill.
- AUTO: requires_cot=true → INSTRUCTOR; else GRAMMAR.

### Detail: Swap protocol
1. Determine next model (informed by SLO tier + workload profile).
2. Multiplexing check → load in parallel if fits.
3. Otherwise wait for `phase_complete` SSB record (atomic durability marker, fsync to WAL).
4. Unload current, load new, health check inference. Failure → reload previous from disk (~15-30s).
5. Resume pipeline (spawn new agent process). Crash recovery: see orchestrator §7B.

### Testable invariants
token_count_estimate within 15% for files >100 lines; ModelPool never exceeds VRAM; swap only at phase boundaries; cache hit skips invocation; float normalization equality; prefill only on finish_reason=Length in EXECUTING; ONNX load failure marks UNAVAILABLE (no crash); ModelPool VRAM accounting includes ONNX; SLO tier mismatch → WARNING; multiplexed sum ≤ GPU_VRAM − 1GB; workload_profile at every phase boundary; no multiplexing on single-GPU Phase 0-1; speculative throughput logged per pair; draft model VRAM accounted; Tier 2 GRAMMAR rejects >3 nesting/$ref cycles; Tier 2 INSTRUCTOR max 3 retries → ESCALATE; AUTO routes correctly; R1-Distill never uses GRAMMAR.

==========================================================================

## CRATE: rules/

### 1. RESPONSIBILITY
Self-improving deterministic behavior enforcement engine. Rules are DATA in SQLite (not files). 6 predicate types, three-tier enforcement, phase/domain-aware activation, self-discovery from auditor patterns, contract-test-validated lifecycle PROPOSED → CANDIDATE → SHADOW → ACTIVE → RETIRED.

### 2. DEPENDENCIES
core/ only. AST evaluation injected via `Box<dyn AstEvaluator>` (knowledge/ implements). DKG injected via `Box<dyn DkgQuerier>` (knowledge/ implements). NO static dependency on knowledge/.

### 3. PUBLIC TRAITS
- `trait AstEvaluator { fn evaluate_query(lang, source, query) -> Result<Vec<AstMatch>> }` (knowledge/ implements; rules/ takes Box<dyn> at construction).
- `trait DkgQuerier { fn query_reachability(...) -> ... }` (knowledge/ implements; rules/ takes Box<dyn>).
- Both validated via `validate_query()` at insertion time.

### 4. PUBLIC TYPES
- 6 predicate types (DSL):
  - **AST_PATTERN** {language, query (sexp or ast-grep YAML), query_format ("sexp"|"ast-grep"), scope (file_path glob)}.
  - **GREP_PATTERN** {pattern (regex), scope, context_lines}.
  - **SCHEMA_REQUIRED** {schema (JSON Schema), output_type (signal/report/config/etc.)}.
  - **METRIC_GATE** {metric_name, operator (lt/gt/eq/between), threshold, source (file/config/runtime)}.
  - **COMPOSITE** {operator (AND/OR/NOT), children (predicate refs)}.
  - **GRAPH_REACHABILITY** {source_pattern, sink_pattern, must_traverse, must_not_exist, max_depth (default 5, max 7), edge_types}; query types: CHOKEPOINT, ISOLATION, BLAST_RADIUS, EFFECT_CHAIN.
- `Verdict { rule_id, result, evidence: String }`.
- Result variants: ALLOW, WARN, BLOCK, ESCALATE.
- `EscalationTarget`: Auditor (SHADOW rules), Human (mutable=0 promotion), NextTierRule(RuleId).

### 5. ERROR TYPES
Predicate validation rejected with specific error type at insertion (Regex::new failure, AstEvaluator::validate_query failure, JSON Schema compile failure, GRAPH_REACHABILITY DkgQuerier::validate_query failure, COMPOSITE depth limit 8 levels exceeded). Anti-backtracking AST_PATTERN rejection: unanchored unbounded `*`/`+` without terminal node; warn on alternations >3 levels.

### 6. SQLITE SCHEMAS

```sql
-- rules.db (.elai_cc/projects/{id}/rules.db)
CREATE TABLE rules (
  rule_id       TEXT PRIMARY KEY,
  domain        TEXT NOT NULL,
  tier          INTEGER NOT NULL CHECK(tier IN (1,2,3)),
  status        TEXT NOT NULL CHECK(status IN ('PROPOSED','CANDIDATE','SHADOW','ACTIVE','RETIRED')),
  title         TEXT NOT NULL,
  description   TEXT NOT NULL,
  predicate     TEXT NOT NULL,
  phase_mask    INTEGER NOT NULL DEFAULT 255,
  severity      TEXT NOT NULL CHECK(severity IN ('BLOCK','WARN','INFO')),
  mutable       INTEGER NOT NULL DEFAULT 1,
  created_at    INTEGER NOT NULL,
  promoted_at   INTEGER,
  version       INTEGER NOT NULL DEFAULT 1,
  source        TEXT NOT NULL CHECK(source IN ('SEED','AUDITOR','HUMAN','SELF_MOD'))
);

CREATE TABLE rule_metrics (
  rule_id       TEXT REFERENCES rules,
  window_start  INTEGER NOT NULL,
  evaluations   INTEGER NOT NULL DEFAULT 0,
  true_positives INTEGER NOT NULL DEFAULT 0,
  false_positives INTEGER NOT NULL DEFAULT 0,
  missed        INTEGER NOT NULL DEFAULT 0,
  -- additions:
  last_true_positive_at  INTEGER,
  confidence_score       REAL DEFAULT 1.0,
  causal_catches    INTEGER NOT NULL DEFAULT 0,
  spurious_catches  INTEGER NOT NULL DEFAULT 0,
  edct_pass  INTEGER NOT NULL DEFAULT 0,
  edct_fail  INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (rule_id, window_start)
);

CREATE TABLE contract_tests (
  test_id       TEXT PRIMARY KEY,
  rule_id       TEXT REFERENCES rules NOT NULL,
  test_type     TEXT NOT NULL CHECK(test_type IN ('BLOCK','ALLOW','EDGE')),
  input_code    TEXT NOT NULL,
  expected      TEXT NOT NULL CHECK(expected IN ('BLOCK','ALLOW','WARN')),
  language      TEXT NOT NULL DEFAULT 'python',
  created_at    INTEGER NOT NULL,
  source        TEXT NOT NULL CHECK(source IN ('SEED','GENERATED','HUMAN'))
);

CREATE TABLE violation_patterns (
  pattern_id    TEXT PRIMARY KEY,
  pattern_hash  BLOB NOT NULL UNIQUE,        -- blake2s of normalized pattern
  occurrences   INTEGER NOT NULL DEFAULT 1,
  first_seen    INTEGER NOT NULL,
  last_seen     INTEGER NOT NULL,
  proposed_rule TEXT,
  domain        TEXT,
  example_code  TEXT NOT NULL,
  score_clarity     INTEGER DEFAULT 0 CHECK(score_clarity BETWEEN 0 AND 5),
  score_frequency   INTEGER DEFAULT 0 CHECK(score_frequency BETWEEN 0 AND 5),
  score_precision   INTEGER DEFAULT 0 CHECK(score_precision BETWEEN 0 AND 5)
);
```
Global rules: `.elai_cc/global/rules_seed.db` (shipped, read-only). Project = global + project-specific (merged at load).

### 7. CONFIG KEYS
- `[rules.domain_mapping]` overrides for file-path domain detection.
- ast-grep query_format decision gate: Phase 3F SKELETON step.

### 8. SECTIONS
S1 RESPONSIBILITIES; S2 STORAGE SCHEMA (rules, rule_metrics, contract_tests, violation_patterns + 3-axis pattern scoring); S3 PREDICATE DSL (6 types); S3A IMPLEMENTATION NOTES (eval order, AST_PATTERN dependency injection, predicate validation); S4 THREE-TIER ENFORCEMENT; S5 RULE LIFECYCLE (PROPOSED→CANDIDATE→SHADOW→ACTIVE→RETIRED + time-aware decay + asymmetric harm + sustainability limit); S5A COUNTERFACTUAL SHADOW TESTING (CEL); S5B EDCT COUNTERFACTUAL METHOD; S6 SELF-DISCOVERY MECHANISM; S7 PHASE-AWARE ACTIVATION (phase_mask bitmask); S8 SEED DATA (subsections 8.1-8.7 by domain); S9 CONTRACT TEST SPECIFICATION; S9A PROPERTY-BASED CONTRACT TESTS (proptest); S10 SEED DATA LOADING; S11 SELF-IMPROVEMENT INTEGRATION; S12 CLAW-CODE MAPPING; S13 TESTABLE INVARIANTS.

### 9. PHASE OWNERSHIP
- Phase 1+: rules.db schema, predicate evaluator, Tier 1 evaluation, seed data load.
- Phase 3F: ast-grep query_format decision gate; GRAPH_REACHABILITY benchmarking at depth 6-7 (reduce to max 5 if >10ms); proptest contract tests alongside static contract tests; effect tagging in DKG.
- Phase 4+ (self-mod): EDCT + CEL counterfactual gates; auditor-source PROPOSED rules require human approval / second auditor verification.

### 10. KNOWN GAPS / TODOs / DEFERRED
- GRAPH_REACHABILITY depth 6-7 performance is an extrapolation that must be validated.
- ast-grep vs sexp decision deferred to Phase 3F SKELETON step.
- EDCT skipped for GREP_PATTERN (no AST transform).
- Theoretical sustainability limit: human review every 50 pipelines or 30 days (top 5 lowest-confidence ACTIVE + top 5 highest-composite PROPOSED).

### Lifecycle gates (deterministic arithmetic)
- PROPOSED → CANDIDATE: `violation_patterns.occurrences >= 3` AND ≥3 contract tests written AND all pass AND EDCT pass.
- CANDIDATE → SHADOW: contract tests pass AND human approves OR independent second-auditor verifies.
- SHADOW → ACTIVE: shadow period complete AND `false_positive_rate < 5%` AND `true_positive_rate > 80%` AND no safety regressions AND `counterfactual_causal_rate > 50%`.
- ACTIVE → RETIRED: `false_positive_rate > 30%` over 10 pipelines OR superseded OR human request OR `confidence_score < 0.1` over 5 pipelines.
- Confidence decay: 90-day half-life, resets on TP. <0.25 demotes ACTIVE→SHADOW. Asymmetric: each FP penalizes 4× vs TP reward.
- Safety/security rules `mutable=0`: cannot auto-retire/auto-adjust.

### Phase mask bitmask
bit 0 PLANNING; bit 1 FEASIBILITY_PROBE; bit 2 TESTING_FIRST; bit 3 EXECUTING; bit 4 AUDITING; bit 5 MEMORIZING; bit 6 CONFIGURING.

### Domain detection (file path → domain)
- "trading", "backtest", "signal", "pipeline" → trading.
- "dashboard", "component", "chart", "ui" → dashboard.
- "payment", "billing", "tenant", "saas" → application.
- "game", "mod", "entity", "spawn" → game.
- "ida", "ghidra", "disasm", "hook" → re.

### Seed rule counts (by domain)
- Trading: 40 rules (~14 AST, 11 GREP, 5 SCHEMA, 13 PROMPT).
- Dashboard: 16 rules (7 AST, 3 GREP, 1 METRIC, 5 PROMPT).
- Application/SaaS: 19 rules (5 AST, 6 GREP, 2 SCHEMA, 6 PROMPT).
- Game: 17 rules (8 AST, 2 GREP, 7 PROMPT).
- RE: 12 rules (5 GREP, 2 AST, 5 PROMPT).
- Engineering (cross-cutting always-active): ~13 Tier 1 (incl. 3 GRAPH_REACHABILITY).
- Security (cross-cutting, mutable=0): ~6 Tier 1 (eval/exec, shell=True, SQL string, deserialization, hardcoded creds, http:// non-localhost).
- Total minimum contract tests for seed rules: ~263.

### Trading rule examples (Tier 1 AST)
Rule 1: `df[].mean()/std()` without `.expanding()/.rolling()` + `.shift(1)`.
Rule 2: find_peaks/argrelextrema/argrelmin/argrelmax imports.
Rule 3: `train_test_split(shuffle=True)`, `KFold` without Purged/Temporal.
Rule 4: signal column without `.shift(1)` before execution price mapping.
Rule 5: `fillna('bfill'/'backfill')`, `scaler.fit(X_full)`, `PCA(X_full)`.
Rule 17: backtest report SCHEMA requires `{ sharpe, dsr, max_drawdown, trial_count }`.
Rule 21: report schema requires `{ trial_count }` (cannot omit).
Rule 29: signal output requires `{ opposing_thesis, risk_if_wrong }`.

### Engineering Tier 1 GRAPH_REACHABILITY rules
- no public API → DB_Write without Auth node on path (CHOKEPOINT).
- no Domain layer → Presentation layer edge (ISOLATION).
- file change → un-updated callers flagged WARN (BLAST_RADIUS).

### Self-discovery process
1. Orchestrator (NOT auditor) normalizes finding description → blake2s pattern hash.
2. Increment `violation_patterns.occurrences`.
3. When ≥3: orchestrator analyzes example_code for common AST structure → generate AST_PATTERN; fall back to GREP_PATTERN; else Tier 3 prompt-only. Auto-generate contract tests (BLOCK from each example, ALLOW counterexamples generated by orchestrator).
4. Insert as PROPOSED with source=AUDITOR. NEVER auto-promote — require human OR independent second auditor.

### Evaluation order (cheapest first, short-circuit on BLOCK)
1. METRIC_GATE (O(1)).
2. GREP_PATTERN (O(n)).
3. AST_PATTERN (tree-sitter parse, amortized cached).
4. COMPOSITE (recursive).
5. GRAPH_REACHABILITY (recursive CTE, <10ms at depth≤5).
Within type: severity DESC (BLOCK before WARN). First BLOCK stops.

### CEL counterfactual (S5A) and EDCT (S5B) — promotion-blocking gates
CEL: counterfactual_causal_rate > 50% required SHADOW → ACTIVE. EDCT runs at PROPOSED → CANDIDATE; spurious rule blocked.

==========================================================================

## CRATE: knowledge/

### 1. RESPONSIBILITY
Codebase understanding engine: DKG (tree-sitter → SQLite graph), BM25 (tantivy), dual-vector search (CODE GraphCodeBERT + TEXT Granite), SPLADE keyword expansion (evaluation), PageRank repo map, CatRAG, A-RAG, cross-encoder reranking, incremental updates.

### 2. DEPENDENCIES
core/, providers/ (for embeddings). External ADD: tree-sitter + grammars (rust/python/typescript/go), rusqlite, tantivy, ort, usearch v2.24.0 (decided Phase 1C 2026-04-06), blake2 (Blake2s256). EVALUATE: fastembed (use ort directly to avoid version conflict). EVALUATE Phase 1G: TurboQuant (arXiv:2504.19874, ICLR 2026) for vector quantization; Phase 1C stores raw f32.

### 3. PUBLIC TRAITS
- `KnowledgeEngine::index_stats() -> IndexStats`.
- Implements `DkgQuerier` (defined in rules/) — `Box<dyn DkgQuerier>` returned to rules/.
- Implements `AstEvaluator` (defined in rules/) — `Box<dyn AstEvaluator>` returned to rules/.

### 4. PUBLIC TYPES
- `IndexStats { total_files, total_chunks, avg_file_tokens, avg_test_tokens, total_dkg_nodes, total_dkg_edges, bm25_doc_count, vector_cache_size }`.
- `ReadinessStatus`: `READY` (all layers + drift check passed), `DEGRADED` (BM25_ONLY mode, ONNX failed but BM25 operational; invariant: implies SearchMode==BM25_ONLY), `UNAVAILABLE` (BM25 also failed; orchestrator blocks all FSM transitions).
- `SearchMode`: `HYBRID` (default, full pipeline), `SPARSE_ONLY` (BM25 + SPLADE), `BM25_ONLY` (last resort).
- `QueryTrace { query_id: UUID, stages: Vec<StageTrace>, total_latency_ms: u64, mode: SearchMode, result_count: usize }`.
- `StageTrace { stage: String, latency_ms: u64, input_count: usize, output_count: usize, error: Option<String>, circuit_breaker_state: Option<String> }`. Stages: bm25, splade, code_vector, text_vector, pagerank, catrag, arag, rerank, token_fit.
- DKG node has `concept TEXT` column (semantic topology, generated by context-compiler during MEMORIZING).
- Effect tagging on dkg_nodes (text column): `Effect::Unsafe`, `Effect::Filesystem`, `Effect::Network`, `Effect::Database`, `Effect::Auth`. Pattern-based (function name + import path).
- Vector cache table: `(content_blake2s BLOB PK, code_vec BLOB, text_vec BLOB, cached_at INTEGER)`.
- Circuit breaker states per ONNX model: CLOSED, OPEN, HALF_OPEN. Failure types: TRANSIENT (count toward threshold default 3 within 60s), PERMANENT (open after 1). Backoff: 30s → 60s → 120s cap. Deterministic jitter `hash(model_id) mod window`. HALF_OPEN: fresh InferenceSession, 2 consecutive successes → CLOSED.

### 5. ERROR TYPES
Query input validation: max 10,000 chars, no null bytes/control chars, no empty queries. Validation <1ms before model invocation.

### 6. SQLITE SCHEMAS
- DKG graph (dkg_nodes + dkg_edges); covering B-Tree index on `(source, target, edge_type)` required for GRAPH_REACHABILITY <10ms guarantee.
- Vector cache (above).
- BM25 via tantivy (per-project index, not shared).

### 7. CONFIG KEYS
- `[knowledge.effect_tags]` configurable effect tagging patterns.
- `[knowledge.exclude_paths]` excludes `.elai_cc/` from DKG/BM25/vectors.
- `index_version.toml` per-project at `.elai_cc/projects/{id}/index_version.toml`:
```
[index]
version = 1
dkg_schema_version = 1
vector_dimensions = { code = 768, text = 384 }
bm25_format = "tantivy_v21"
onnx_model_path = ".elai_cc/models/onnx/"
created_at = 1743811200
```
Plus `canary_reference_vec` field (drift check baseline).

### 8. SECTIONS
S1 RESPONSIBILITIES; S2 LAYERS (5 major + 2 sub: DKG, BM25, Vectors [+SPLADE 3.25, +PageRank 3.5], CatRAG, A-RAG); S3 QUERY PIPELINE (with cross-encoder); S4 DEPENDENCIES; S5 GRACEFUL DEGRADATION (HYBRID/SPARSE_ONLY/BM25_ONLY cascade + query input validation); S6 ONNX MODEL HEALTH AND CIRCUIT BREAKER; S7 STARTUP READINESS (canary query + drift check); S8 TREE-SITTER PARSE FAILURE FALLBACK; S9 CONCEPT ANNOTATION BATCHING; S10 INDEX VERSIONING AND MIGRATION; S10A INDEX STATISTICS EXPORT; S11 CROSS-ENCODER CONCURRENCY (tokio Semaphore=1, queue depth 4); S12 KNOWLEDGE PIPELINE EXECUTION TRACING; S13 TESTABLE INVARIANTS.

### 9. PHASE OWNERSHIP
- Phase 1A: DKG.
- Phase 1C: usearch v2.24.0 decided 2026-04-06.
- Phase 1G/1H: SPLADE KEEP/DROP decision; TurboQuant evaluation (Phase 1C stores raw f32; quantization deferred).
- Phase 3E+: auditor DKG blast radius injection in context budget Slot 5 (orchestrator §6).
- Phase 3F: effect tagging incremental addition (text column, no schema change).

### 10. KNOWN GAPS / TODOs / DEFERRED
- SPLADE EVALUATE: drop if <5% retrieval gain over BM25 alone.
- TurboQuant: paper ingested 2026-04-06 (workflow/papers/turboquant-interface.md). Verified: b=3 → MSE=0.030, ~10.7× compression vs f32, InProd error ~0.00023/dim for 768d. Requires unit-norm input + QR rotation (not Hadamard). Codebook precomputed per (b,d). Combine with HNSW. Reimplement in Rust.
- fastembed: crate slug is "fastembed" not "fastembed-rs"; wraps ort internally; version conflict risk; use ort directly.
- Cross-encoder: if contention >200ms p99 in production, evaluate ONNX session pool (2-3 instances).
- First indexing batching: 50 nodes/MEMORIZING phase max, ~10 batches for 200-node project, ~2-5 min at local model speed.

### Layer details
- **Layer 1 DKG**: 100% coverage of supported langs, <15s build, incremental, languages Rust/Python/TS/Go.
- **Layer 2 BM25**: tantivy, sub-ms queries, per-project.
- **Layer 3 Vectors**: GraphCodeBERT (microsoft/graphcodebert-base, 768d) + Granite (ibm-granite, 384d). Both via ort on GPU. HNSW index (usearch). Vector cache invalidated immediately on SSB action_type='file_write'.
- **Layer 3.25 SPLADE**: prithvida/Splade_PP_en_v1 (~500MB ONNX), sparse vector, fused via weighted RRF.
- **Layer 3.5 PageRank repo map**: 6-step pipeline (snapshot+self-edges 0.1, edge boost ×50 with sqrt scaling, personalization ×10, dangling=personalization, density guard 0.3, power iteration damping=0.85 max_iter=100 tol=1e-6, binary search token fit ±15%, signature truncation 100 chars). Special files (Cargo.toml, Dockerfile, .gitignore, README, ~100 items) always included.
- **Layer 4 CatRAG**: query-aware graph traversal.
- **Layer 5 A-RAG**: hierarchical retrieval, 55-87% token savings.
- **Cross-encoder**: cross-encoder/ms-marco-MiniLM-L-12-v2 (~120MB ONNX fp32, CPU-only). Applied only when candidate set > 10. <50ms for 50 candidates.

### Query pipeline (full)
`query → BM25 + SPLADE (RRF fused) → dual-vector rerank → PageRank importance filter → CatRAG expand → A-RAG filter → cross-encoder rerank → token-budget fit → context`.

### Fallback per ONNX model
- GraphCodeBERT fail → TEXT-only (Granite).
- Granite fail → CODE-only (GraphCodeBERT).
- Both fail → SPARSE_ONLY.
- SPLADE fail → BM25_ONLY.
- Cross-encoder fail → skip reranking, use vector similarity scores.
- Circuit breaker state NOT checkpointed; resets to CLOSED on process restart.

==========================================================================

## CRATE: tools/

### 1. RESPONSIBILITY
File ops (read with metadata/outline, write, edit), bash sandbox (per-agent isolated, timeout-controlled), git operations (git2 crate, auto-commit/undo/branch), 5-tier validation pipeline, DAIRA trace execution, ShortCoder AST-preserving simplification, 3-tier SEARCH/REPLACE edit matching, deterministic failure taxonomy, tool registry phase-filtering, visibility filter (per-agent allowlist; hidden paths return "not found").

### 2. DEPENDENCIES
core/, knowledge/ (for tree-sitter AST). ADD: git2, tree-sitter + grammars (shared with knowledge/). KEEP: existing bash execution from claw-code.

### 3. PUBLIC TRAITS
- `BashSandbox` trait: `fn execute(command, sandbox_config) -> BashResult`. Implementations: `HostBash` (Phase 0-1), `FirecrackerBash` (Phase 3+), `GvisorBash` (fallback).

### 4. PUBLIC TYPES
- `read_file(path, start_line?, end_line?, force_full: bool = false) -> FileReadResult`.
- `FileReadResult { path, total_lines, estimated_tokens, content: Full | Outline }`.
- Full: complete content with line numbers (files ≤300 lines or range specified).
- Outline: tree-sitter parsed sections (kind, signature, line range) for large files.
- Force_full=true bypasses outline threshold but NOT budget guard. Budget guard: if estimated_tokens > context_budget_remaining × 0.5 → WARN. INTERACTIVE: user decides. AUTONOMOUS: auto-downgrade to outline.
- `trace_execution(script, trace_config) -> TraceReport`. Markers: [CALL], [RETURN], [MUTATION], [BRANCH], [ERROR], [PANIC], [TIMEOUT], [FINDING], [PATTERN], [ANOMALY], [STAT:name], [DURATION:name], [MEMORY:name], [CHECKPOINT], [STEP:n].
- `MarkerReport { findings, anomalies, mutations, errors, stats, raw_line_count, marker_line_count }`.
- `AdaptiveTool<T: Tool>` wrapper with budget; max 3 reductions.
- `code_simplify(code, rules) -> SimplifiedCode`.
- `VisibilityFilter { allowed_paths: Vec<GlobPattern>, hidden_paths: Vec<GlobPattern> }`.
- `classify_failure(error_output) -> FailureClassification`.

### 5. ERROR TYPES
Failure taxonomy (deterministic regex, no LLM):
SYNTAX_ERROR, TYPE_ERROR, IMPORT_ERROR, ASSERTION_ERROR, TIMEOUT, EMPTY_OUTPUT, RUNTIME_PANIC, PERMISSION_DENIED, RESOURCE_EXHAUSTED, DEFERRED (unresolved import from unwritten module during incremental development). Each category has deterministic suggested_action. Auditor reads classification BEFORE raw output (reduces context 200-500 tokens). 3+ same-category failures in one phase → anti-spiral trigger.

Visibility hidden-path access: returns `Error::NotFound("path")` IDENTICAL to genuinely missing files (no separate variant, no log, no timing difference, filter <0.1ms).

### 6. SQLITE SCHEMAS
None.

### 7. CONFIG KEYS
- `[agents.{role}.sandbox]` per-agent sandbox config.
- Default hidden_paths (all agents): `.elai_cc/**`, `**/rules.db`, `**/rules_seed.db`, `**/elai_cc.toml`, `**/handoff.msgpack`, `**/ssb.db*`.
- Default allowed_paths per role:
  - executor: project source code, test files, agent scratch dir.
  - auditor: project source code, test files, test results, agent scratch dir.
  - context-compiler: project source code (read-only).
  - prompt-engineer: nothing (receives context via prompt).
  - memory-writer: project source code (read-only), knowledge output dir.

### 8. SECTIONS
S1 RESPONSIBILITIES; S2 FILE READING ENHANCEMENT (force_full + budget guard); S3 3-TIER EDIT MATCHING; S4 GIT INTEGRATION (auto-commit, branch, WIP commits); S5 VALIDATION PIPELINE (5 tiers); S6 DAIRA TRACE EXECUTION (markers + AdaptiveTool); S7 SHORTCODER (10 AST rules, 18-38% savings); S8 VISIBILITY FILTER (Tier A); S9 BASH SANDBOX (Tier B); S10 INCREMENTAL COMPILATION FEEDBACK (Phase 4B); S11 FAILURE TAXONOMY; S12 CLAW-CODE MAPPING; S13 DEPENDENCIES; S14 TESTABLE INVARIANTS.

### 9. PHASE OWNERSHIP
- Phase 0-1: HostBash sandbox (chroot project source, sanitized PATH, no HOME/no ELAI_CC paths/no API keys).
- Phase 3-4: Firecracker / gVisor sandbox (full spec in orchestrator §12 Tier B).
- Phase 4B: incremental compilation feedback (cargo check / mypy+ruff / tsc / go vet) async per file write during EXECUTING; max 3 retries per file (round 1 initial, round 2 retry, round 3 final); NEEDS_AUDIT after round 3 failure; NOT applied during TESTING_FIRST.

### 10. KNOWN GAPS / TODOs / DEFERRED
- Latency estimates (cargo check 2-5s, ruff <500ms, mypy <2s) are UNGROUNDED training-derived; measure Phase 4B.5.
- Expected impact: ~30% reduction in auditor BLOCK findings for syntax/type errors.
- ASYNC execution model: compile check runs in background while executor proceeds (risk-register R51).

### Edit matching tiers
Tier 1 exact string. Tier 2 whitespace-normalized (preserves original indentation on replace). Tier 3 ellipsis-elision ("..." matches any line sequence). On failure: SequenceMatcher similarity hint. Never silently applies wrong match.

### Validation pipeline tiers
1. Determinism (math-verified). 2. Quality (AST analysis, code smells, branch coverage). 3. Unit tests. 4. Integration/contract. 5. Stress/adversarial. All run by auditor; executor NEVER runs own tests.

### Git integration
Phase-level commits (not per-file). Every non-WIP commit on main has passing tests (enforced by evidence gate). Large EXECUTING (>5 file edits): intermediate WIP commits prefixed `WIP phase-{N}: {step_description}`. WIP squashed into phase commit on AUDITING success. Branch per plan: `plan-{pipeline_id}`. Auto-gitignore heavy data.

### ShortCoder rules (10, AST-preserving via tree-sitter, NOT regex)
R1: redundant variable assignments. R2: simplify boolean. R3: collapse single-use vars. R4: if/else→ternary (CONFIGURABLE). R5: merge sequential ops. R6: loops→comprehensions (CONFIGURABLE). R7: remove unnecessary parentheses. R8: simplify return. R9: remove dead branches. R10: normalize string formatting. Applied as POST-PROCESSING on executor output before auditor review.

### Error sanitization (applied to all tool results to agent)
Strip: rule_id patterns ("trading.22", "auto.127"), predicate type names (AST_PATTERN, GREP_PATTERN), SQLite error details on rules/ssb tables, hook names (PreToolUse, PostToolUse). Replace with generic NL description.

### Tool schema sanitization (swarm-derived)
Strip from agent-visible schemas: `_rule_context`, `_enforcement_tier`, `_session_lock`, `_fsm_state`, `_agent_id`, `_visibility_filter`. Source: swarm core.py `__CTX_VARS_NAME__` stripping. ~5-line filter in tool registry.

==========================================================================

## CRATE: plugins/

### 1. RESPONSIBILITY
Hook system (PreToolUse/PostToolUse/Stop/PreCompact), skill system (prompt-transform, planning-workflow, rag-tools), MCP bridge, tiered static analysis hook on SSB writes (T1 RULES <1ms, T2 AST <10ms, T3 PATTERN MATCH <5ms).

### 2. DEPENDENCIES
core/, rules/ (for hook enforcement).

### 3. PUBLIC TRAITS
Not enumerated; uses hook handler interface.

### 4. PUBLIC TYPES
Hook verdicts: ALLOW / BLOCK / WARN. BLOCK never leaks specifics — generic IPC error: "Tool call rejected by policy".

### 5. ERROR TYPES
Generic IPC rejection; no rule ID, tier, or pattern info exposed to agents.

### 6. SQLITE SCHEMAS
None directly (uses rules.db via rules/ crate).

### 7. CONFIG KEYS
None enumerated.

### 8. SECTIONS
S1 RESPONSIBILITIES; S2 HOOK SYSTEM (Tier C: hooks intercept IPC; PreToolUse fires on ToolRequest/BashRequest/WriteFileRequest BEFORE forward; PostToolUse fires after tool execution BEFORE ToolResult sent back; Stop on session termination; PreCompact on truncation checkpoint); S3 TIERED STATIC ANALYSIS HOOK (T1/T2/T3); S4 MCP BRIDGE; S5 CLAW-CODE MAPPING; S6 TESTABLE INVARIANTS.

### 9. PHASE OWNERSHIP
- Phase 4A+: hooks intercept IPC under Tier C.
- T1 RULES: every SSB write triggers <1ms.
- T2 AST: action_type='file_write' only; circuit breaker — files >5000 lines degrade to T1-only.
- T3 PATTERN MATCH: only when (WARN exists) AND (no BLOCK from any tier). If T1=BLOCK and T2=WARN, T3 does NOT fire. Pattern-match library; NO LLM. Examples: function >200 lines → WARN:complexity; `catch Exception: pass` → BLOCK:swallow.

### 10. KNOWN GAPS / TODOs / DEFERRED
- BLOCK/WARN pathway: SSB 'finding' entry → orchestrator normalizes finding to pattern hash (blake2s) → feeds violation_patterns. Circular trust boundary: orchestrator normalizes (NOT auditor).

### Claw-code mapping
- mcp_tool_bridge.rs (912 LOC) → MCP bridge.
- mcp_lifecycle_hardened.rs (839 LOC) → error recovery.
- plugin_lifecycle.rs (532 LOC) → plugin lifecycle.
- Hooks infrastructure from plugins/ → KEEP and extend.

### Testable invariants
T1 within 1ms; T2 only file_write; T2 circuit breaker >5000 lines; T3 only on WARN; T3 <5ms (no LLM); BLOCK as 'finding'; no LLM in hot path; Stop hook prevents non-IDLE termination; PreCompact triggers SESSION_HANDOFF; cross-session locks via canonical path comparison.

==========================================================================

## CRATE: reasoning/

### 1. RESPONSIBILITY
GRC engine (state machine replacing internal CoT), per-topic budgets, grounding verification (FILE/API/WEB/SOURCE/TEST/API_REGISTRY checks), plan grounding sweep, fact-based planning protocol, counterfactual safety (CSQ), per-agent GRC config, inline feasibility check, IPC-proxied verification under Tier C process isolation.

### 2. DEPENDENCIES
core/, providers/ (model invocation), tools/ (verification checks).

### 3. PUBLIC TRAITS
```
[2B] trait Verifier: Send + Sync {
    fn verify(&self, method: VerificationMethod, claim: &str, context: &str)
        -> Result<VerificationResult, GrcError>;
    fn supported_methods(&self) -> &[VerificationMethod];
}

[2B] trait TestRunner: Send + Sync {
    fn run_test(&self, name: &str, crate_name: &str) -> TestRunResult;
}
// Implementations: CargoTestRunner (BashSandbox::execute), MockTestRunner (tests).

[2B] GrcEngine trait updates (breaking from 2A stubs):
    fn verify_claim(&self, claim: &str, source: &str, context: &str) -> Result<VerificationResult, GrcError>
    fn ground_plan(&self, plan: &str) -> PlanGroundingResult
```

### 4. PUBLIC TYPES (formal Rust definitions, phase tags shown)
```
[2A] enum GrcState { Observe, Hypothesize, PlanAction, Act, Verify, Converge, Status }
    fn allowed_next(&self) -> &[GrcState]
    // Status free (no budget, no grounding). Converge from any state.

[2A] ThinkStep {
    step_type: GrcState,
    content: String,
    topic: String,
    grounding: Option<String>,
    conclusion: Option<String>,
    scope: Option<Vec<String>>   // require_scope_enumeration: Observe must enumerate components
}

[2A] GrcBudget { max_calls: u32, max_chars: u32, calls_used: u32, chars_used: u32 }
    fn remaining_calls(&self) -> u32
    fn remaining_chars(&self) -> u32
    fn is_exhausted(&self) -> bool
    fn must_converge(&self) -> bool   // remaining_calls <= 1

[2A] enum GroundingLevel { Verified, High, Uncertain }

[2A] enum GrcError {
    UngroundedStep { step_type, topic },
    BudgetExhausted { topic, calls_used, max_calls },
    InvalidTransition { from: GrcState, to: GrcState, allowed: Vec<GrcState> },
    MalformedGrounding { raw: String, reason: String },
}  // implements std::error::Error via thiserror

[2A] ThinkResult { step: ThinkStep, budget_remaining: GrcBudget,
                   grounding_level: Option<GroundingLevel>, warnings: Vec<String> }

[2B] enum VerificationMethod { FileCheck, SourceCheck, TestCheck, WebCheck, ApiCheck, ApiRegistryCheck }
    fn requires_network(&self) -> bool   // WebCheck, ApiCheck, ApiRegistryCheck = true
    fn requires_tools(&self) -> bool     // FileCheck, SourceCheck, TestCheck = true

[2B] GroundingResult { claim: String, method: VerificationMethod, level: GroundingLevel,
                       source: String, verified_at: u64 }   // verified_at from TimeSource trait

[2B] VerificationResult { claim: String, passes: bool, evidence: String,
                          method_used: VerificationMethod }

[2B] PlanGroundingResult { claims: Vec<GroundingResult>, all_grounded: bool,
                           ungrounded: Vec<String>, verify_needed: Vec<String> }
    // all_grounded = ungrounded.is_empty()

[2B] GrcError += VerificationError { method: VerificationMethod, claim: String, reason: String }

[2C] AgentGrcConfig { budget, required_steps, grounding_level, csq_required,
                      require_scope_enumeration: bool }   // default true for executor & designer

[2D] enum CsqSeverity { Critical, Warning }
[2D] CsqCheck { assumption: String, consequence: String, severity: CsqSeverity, guard_added: bool }
[2D] CsqResult { function_id: String, checks: Vec<CsqCheck> }
    fn passed(&self) -> bool   // all Critical checks have guard_added == true

[2D] InlineFeasibilityState { checks_used: u32, max_checks: u32 }
    fn is_exhausted(&self) -> bool   // checks_used >= max_checks
    fn record_check(&mut self)       // saturating_add
    fn reset(&mut self)              // checks_used = 0, called by orchestrator between phases
    // Default: max_checks = 2 (hardcoded)

[2D] enum FeasibilityMethod { SourceMatch }   // future: WebSearch, Grep, ApiRegistryCheck
[2D] enum FeasibilityOutcome { Pass, Fail, Blocked }
[2D] InlineFeasibilityResult { claim: String, method: FeasibilityMethod, outcome: FeasibilityOutcome }

[4A] GroundingRequest { method, claim, context }   // IPC message type for process isolation
```

### 5. ERROR TYPES
`GrcError` variants: UngroundedStep, BudgetExhausted, InvalidTransition, MalformedGrounding, VerificationError (added [2B]).

### 6. SQLITE SCHEMAS
None.

### 7. CONFIG KEYS
- Per-agent GRC config in `elai_cc.toml`. Per-topic max_calls (1-12), max_chars (200-1500). Global default configurable.
- Budget scales: simple=3/600, moderate=5/800, complex=8/1200.

### 8. SECTIONS
S1 RESPONSIBILITIES; S2 GRC STATE MACHINE (observe→hypothesize→plan_action→act→verify); S3 BUDGET SYSTEM; S4 GROUNDING VERIFICATION (FILE/API/WEB/SOURCE/TEST/API_REGISTRY); S5 FACT-BASED PLANNING PROTOCOL (7 steps); S6 COUNTERFACTUAL SAFETY (CSQ); S7 PER-AGENT GRC CONFIGURATION; S8 INLINE FEASIBILITY CHECK; S9 GRC VERIFICATION UNDER PROCESS ISOLATION (Tier C); S10 KEY TYPES (phase-tagged); S11 TESTABLE INVARIANTS.

### 9. PHASE OWNERSHIP
- [2A]: GrcState, ThinkStep, GrcBudget, GroundingLevel, GrcError, ThinkResult.
- [2B]: VerificationMethod, GroundingResult, VerificationResult, PlanGroundingResult, Verifier trait, TestRunner trait, breaking GrcEngine updates.
- [2C]: AgentGrcConfig, per-role configs (executor, auditor, designer, context-compiler, prompt-engineer, memory-writer).
- [2D]: CSQ types, InlineFeasibilityState, FeasibilityMethod/Outcome/Result.
- [3A+]: Blocked outcome triggers ReplanRequest(MAJOR) emission (orchestrator integration).
- [4A]: GroundingRequest IPC; orchestrator-counted budget enforcement.

### 10. KNOWN GAPS / TODOs / DEFERRED
- FeasibilityMethod future variants: WebSearch, Grep, ApiRegistryCheck (currently SourceMatch only).
- API_REGISTRY_CHECK sources: public-apis GitHub (~1400 free), RapidAPI free tier, package managers. INTERACTIVE: user approves selection. AUTONOMOUS (Phase 4G): only keyless/free APIs auto-selected. Auth credentials in `.elai_cc/projects/{id}/.env` (gitignored).

### Per-agent GRC configs (defaults)
- executor (Gemma 4 31B): budget 8/1200; steps [observe, hypothesize, plan_action, act, verify]; all steps require source citation; CSQ mandatory for file/network/money/auth; require_scope_enumeration: true.
- auditor (DeepSeek-R1-32B): budget 10/1200; steps [observe, hypothesize, verify, verify] (double verify); VERIFIED only (no HIGH/UNCERTAIN); must find ≥1 issue per review (anti-rubber-stamp); CSQ mandatory on every finding.
- designer (Gemma 4 31B): budget 8/1200; steps [observe, hypothesize, plan_action, verify, converge]; cite Research Report + verified sources; no CSQ; require_scope_enumeration: true.
- context-compiler: budget 3/600; steps [observe]; FILE_CHECK + SOURCE_CHECK only; no CSQ; require_scope_enumeration: false.
- prompt-engineer: budget 5/800; steps [observe, hypothesize, converge]; cite plan + context package.
- memory-writer: budget 2/400; steps [observe, converge]; cite diff + test results; require_scope_enumeration: false.
- Per-agent FSM sub-states: IDLE → LOADING → REASONING → ACTING → DONE (persisted in agent checkpoint).

### Inline feasibility (S8)
During EXECUTING, GRC verify detects NEW claim not in PipelineSpec/Research Report/DKG → quick check (max 2 GRC calls): web search OR grep OR API_REGISTRY_CHECK. PASS → grounded. FAIL → must use plan's original approach. Fundamentally different approach → EXECUTING → BLOCK → PLANNING.

### Tier C IPC verification proxy (S9)
- FILE_CHECK: orchestrator reads via visibility filter (all roles).
- SOURCE_CHECK: orchestrator reads + parses (all roles).
- TEST_CHECK: orchestrator runs in sandbox (executor, auditor).
- WEB_CHECK / API_CHECK / API_REGISTRY_CHECK: orchestrator (has network) (all roles).
- Agents NEVER perform WEB_CHECK/API_CHECK directly.
- Budget enforcement counts IPC GroundingRequest messages per topic against same per-topic GRC budget.

==========================================================================

## CRATE: plugins/ (already covered above — placeholder)

==========================================================================

## CRATE: tools/ (already covered above — placeholder)

==========================================================================

## CRATE: orchestrator/

### 1. RESPONSIBILITY
Largest crate. FSM with phase-aware tool/context loading; agent spawning/lifecycle/checkpointing/crash recovery; agent parallelism with resource-aware scheduling; SSB (single-writer SQLite WAL); BSP superstep model; channel types (LastValue/Topic/UntrackedValue/EphemeralValue); context budget allocator; session handoff (DAG, no summarization); project isolation namespace; dual execution modes; evidence gate protocol; FEASIBILITY_PROBE; dynamic fan-out (Send); deferred node execution; session-owned commit tracking; agent isolation tiers A/B/C; deterministic dispatch (meta-dispatch + plan-dispatch).

### 2. DEPENDENCIES
core/, providers/ (ModelPool), reasoning/ (GRC), rules/, tools/.

### 3. PUBLIC TRAITS
```
trait FeasibilityProbe {   // object-safe, all methods take &self
    fn probe_system(&self) -> Result<SystemProbeResult>;
    fn check_constraints(&self, probe, constraints) -> bool;
    fn run_stub(&self, step, alternative, timeout) -> Result<StubResult>;
    fn decide(&self, report, mode: ExecutionMode) -> FeasibilityDecision;
}

trait ChannelRegistry {
    fn read(id: ChannelId<T>) -> ChannelValue<T>;
    fn write(id: ChannelId<T>, value: T) -> Result<(), ChannelError>;
    fn snapshot_for_scheduling() -> ChannelSnapshot;
    fn clear_ephemeral() -> usize;   // cleared count, records in SSB
    fn force_clear_untracked(id) -> bool;
    fn deep_size_untracked(id) -> usize;
}
```
- `ChannelId<T>`: typed key ensuring compile-time channel/type match.

### 4. PUBLIC TYPES
- FSM states: `IDLE → PLANNING → FEASIBILITY_PROBE → TESTING_FIRST → EXECUTING → AUDITING → MEMORIZING → COMPLETE → IDLE`. Plus `CONFIGURING`.
- `ChannelState`: enum 3 variants (LastValue, Topic, Ephemeral) for checkpoint serialization. UntrackedValue excluded by design.
- Channel kinds:
  - `LastValue<T>`: single value, last writer wins (deterministic via SSB seq).
  - `Topic(accumulate: bool)`: append-only list. accumulate=true persists across supersteps. accumulate=false: cleared at boundary (two-phase reset).
  - `UntrackedValue<T: DeepSizeOf>`: in-memory only, NEVER checkpointed. guard=false multi-writer. Lost on crash.
  - `EphemeralValue<T>`: cleared to MISSING on zero-write superstep (per-channel write_count u32 saturating_add, reset at superstep start; clearing recorded in SSB action_type='checkpoint' ephemeral_clears field for replay).
  - `BinaryOp`: DROPPED.
- SSB action_types (17 total): `file_write, file_read, tool_result, test_outcome, finding, claim, checkpoint, model_swap, phase_transition, knowledge_degradation, isolation_metric, plan_update, replan_request, stub_reference, reasoning_trace, memory_monitoring, phase_complete`.
- `ReasoningTraceEntry { agent_id: UUID, step_index: u32, step_type: String ("observe"|"hypothesize"|"plan_action"|"act"|"verify"|"tool_call"|"grounding_check"|"convergence"), content_summary: String (max 200 chars), grounding_source: String, grounding_level: String (VERIFIED|HIGH|UNCERTAIN), confidence: f64 [0,1], elapsed_ms: u64, topic: String, budget_remaining: u32 }`.
- Feasibility types (in `orchestrator::feasibility`):
  - `SystemProbeResult { gpu_count, gpu_info: Vec<GpuInfo>, ram_total_bytes, ram_free_bytes, disk_free_bytes, os, is_supported }`.
  - `GpuInfo { index, name, total_bytes, free_bytes, driver_version, uuid }`.
  - `SystemConstraints { min_vram_bytes, min_ram_bytes, min_disk_bytes, required_gpu_count, model_max_context: usize, available_for_work: usize, avg_file_tokens: usize, avg_test_tokens: usize }`.
  - `StubResult { step_id, alternative_index (0-2), outcome: FeasibilityOutcome, elapsed_ms, stdout_hash: [u8;32], stderr_summary, resource_delta: ResourceDelta }`.
  - `ResourceDelta { vram_before_bytes, vram_after_bytes, ram_before_bytes, ram_after_bytes }`.
  - `ConstraintResult { resource, required, available, margin, passed }`.
  - `FeasibilityReport { probe, constraints, constraint_results: Vec<ConstraintResult>, stubs (passing novel only), skipped_steps, escalations }`.
  - `FeasibilityEscalation { step_id, reason, stubs_tried: Vec<StubResult> }`.
  - `enum FeasibilityDecision { Proceed, ProceedWithConstraints(Vec<String>), Escalate(Vec<FeasibilityEscalation>), Replan(Vec<FeasibilityEscalation>) }`.
- Dispatch types:
  - `AgentDispatchConfig { role: AgentRole, model_preference: ModelPreference, tool_filter: ToolFilter, rule_domains: Vec<Domain>, grc_config: AgentGrcConfig, sandbox_config: SandboxConfig }`.
  - `ToolFilter { allowed_tools: Vec<ToolId> }` — whitelist enforced at IPC layer.
  - `DispatchPipeline { stages: Vec<DispatchStage> }`.
  - `DispatchStage { stage_id: StageId, agent: AgentDispatchConfig, input_from: Option<StageId>, output_artifact: ArtifactType, gate: Option<GateCondition>, skip_condition: Option<SkipCondition> }`.
  - `enum ArtifactType { ResearchReport, PipelineSpec, QualitySpec, TransformedPrompt, FeasibilityReport, ExecutablePipeline }`.
  - `enum GateCondition { ArtifactExists(ArtifactType), QualityThreshold { metric, min }, EvidenceGate(FsmState) }`.
  - `enum SkipCondition { NotAnAgent, FeatureDisabled }`.
  - `MetaDispatchTable { planning: DispatchPipeline, configuring: DispatchPipeline }`.
  - `PlanDispatchTable { entries: Vec<PlanDispatchEntry> }`.
  - `PlanDispatchEntry { phase_id: PhaseId, fsm_state: FsmState, agent: AgentDispatchConfig, depends_on: Vec<PhaseId> }`.
  - `enum AgentRole { Executor, Auditor, Researcher, Designer, PromptEngineer, ContextCompiler, MemoryWriter, MediaValidator }`.
- Designer (NEW): role_prompt = "Design a phased execution plan from the research findings. Output: TOML PipelineSpec with phases, dependencies, tool requirements, resource estimates, and quality metrics. Do not implement. Plan only." GRC: 8/1200, steps [observe, hypothesize, plan_action, verify, converge]. Reuses executor model.
- IPC message enum (complete):
  `AgentRequest, AgentResponse, InferRequest, InferResponse, ToolRequest, ToolResult, BashRequest, BashResult, GroundingRequest, GroundingResult, ThinkStep, CheckpointSync, AgentHeartbeat, AgentKill, ReplanRequest`.
- `ReplanRequest` fields: `evidence (string, max 500 tokens), severity (MINOR|MAJOR)`. Bounds: max 1 per agent per phase. Logged as action_type='replan_request'.
- Agent death classification: CLEAN_EXIT, CRASH, TIMEOUT (heartbeat 15s missed → SIGKILL + restart), BUDGET_EXCEEDED.
- Supervision restart policy: `max_restarts: 3 per phase` (configurable `[agents.{role}.supervision]`); restart_window = current phase duration; on_max_restarts: ESCALATE (interactive Phase 3) or SUBSTITUTE_MODEL (autonomous Phase 4G).

### 5. ERROR TYPES
- `ChannelError` returned by ChannelRegistry::write.
- IPC: oversized frame rejected (MAX_FRAME_SIZE 10MB hard cap). Per-type limits: AgentHeartbeat <1KB, ThinkStep <64KB, ToolResult <10MB, AgentResponse <context_limit.

### 6. SQLITE SCHEMAS

```sql
-- SSB (per-project, single SQLite WAL)
CREATE TABLE ssb (
    seq INTEGER PRIMARY KEY,
    project_id TEXT,
    session_id TEXT,
    agent_id TEXT,
    superstep INTEGER,
    action_type TEXT CHECK(...),  -- 17 types listed in §3
    payload BLOB,
    created_at INTEGER
);
CREATE TABLE ssb_files (
    seq INTEGER REFERENCES ssb,
    file_path TEXT
);
-- Indexes: project_id, session_id, superstep, file_path, (project_id, superstep)

-- Conflict detection query (requires JOIN — ssb_files has only seq + file_path)
SELECT DISTINCT ssb.agent_id FROM ssb_files
JOIN ssb ON ssb_files.seq = ssb.seq
WHERE ssb_files.file_path IN (rarray(?1))
  AND ssb.project_id = ?2
  AND ssb.session_id != ?3
  AND ssb.superstep = ?4;

-- Session DAG (.elai_cc/sessions/{id}/session_dag.db, CMV-derived)
CREATE TABLE snapshots (
    snap_id TEXT PRIMARY KEY,
    parent_id TEXT,
    session_id TEXT,
    fsm_state TEXT,
    phase TEXT,
    step TEXT,
    findings BLOB,
    code_changes BLOB,
    tests_written BLOB,
    commit_hashes TEXT,
    created_at INTEGER,
    tag TEXT
);
CREATE TABLE snapshot_context (
    snap_id TEXT REFERENCES snapshots(snap_id),
    seq INTEGER,
    role TEXT CHECK(role IN ('system','user','assistant','tool')),
    content BLOB,
    is_stubbed INTEGER DEFAULT 0,
    stub_ssb_seq INTEGER,
    PRIMARY KEY(snap_id, seq)
);

-- Hierarchical plan management (in SSB action_type='plan_update')
-- File: .elai_cc/pipelines/{id}/skeleton.toml (~2-4k tokens, always loaded Slot 3)
[phase.3B]
goal = "SSB single-writer SQLite WAL"
inputs = ["AgentPool::write(action)", "FSM::phase_transition()"]
outputs = ["ssb::query(filter) -> Vec<SsbRow>"]
constraints = ["append-only within session", "conflict detection on file_path"]
depends_on = ["3A"]
depended_by = ["3C", "3D"]
version = 3
-- Phase detail: .elai_cc/pipelines/{id}/phases/{phase_id}.toml (~5-15k, on demand)
```
SSB throughput: ~50K inserts/sec ceiling. Agent rate: ~1-10/sec.

### 7. CONFIG KEYS
- `[fsm.meta_dispatch]` for meta-dispatch table override (else hardcoded defaults).
- `[agents.{role}.supervision]` max_restarts (default 3), restart_window.
- `[context_budget.auditor.subgraph_pct]` default 30%.
- `[planning.sizing.default_avg_file_tokens]` default 150.
- `[planning.sizing.default_avg_test_tokens]` default 80.
- `[knowledge.exclude_paths]` excludes .elai_cc/ from indexes.

### 8. SECTIONS
S1 RESPONSIBILITIES; S2 FSM STATES (phase-aware loading per state); S3 SSB; S4 BSP SUPERSTEPS; S5 CHANNEL TYPES; S6 CONTEXT BUDGET ALLOCATOR (fixed-slot ordering, cache breakpoints, auditor DKG blast radius split); S7 AGENT CHECKPOINTING (+ supervision tree); S7A STRUCTURED REASONING TRACE (verifiable audit trail); S7B PHASE COMPLETION AND MODEL SWAP PROTOCOL; S8 DYNAMIC FAN-OUT; S9 HIERARCHICAL PLAN MANAGEMENT; S10 SESSION HANDOFF (CMV-derived DAG, structurally lossless trimming); S11 PROJECT ISOLATION; S12 AGENT ISOLATION MODEL (Tier A invisible walls / Tier B sandbox / Tier C process separation); S13 EVIDENCE GATE PROTOCOL; S13A FEASIBILITY_PROBE STATE (Phase 4F); S14 DETERMINISTIC DISPATCH PROTOCOL (meta-dispatch + plan-dispatch); S15 CLAW-CODE MAPPING; S15 (duplicate label in source) TESTABLE INVARIANTS.

### 9. PHASE OWNERSHIP
- Phase 0-1: Tier A (invisible walls, filesystem view filtering, error sanitization, knowledge engine exclusion, tool registry opacity); HostBash sandbox.
- Phase 3: Tier B (Firecracker preferred <125ms boot 5MB overhead, gVisor fallback ~200ms; decision via measurement).
- Phase 3C: supervision tree pattern formalization.
- Phase 3D: memory_monitoring (UntrackedValue size tracking + force-clear).
- Phase 3E+: auditor DKG blast radius injection.
- Phase 4: Tier C process separation (each agent = elai_cc_agent binary; LLM proxied via IPC).
- Phase 4A+: hooks intercept IPC; orchestrator-counted GRC budget.
- Phase 4F: FEASIBILITY_PROBE state.
- Phase 4G: AUTONOMOUS mode; SUBSTITUTE_MODEL on max_restarts.

### 10. KNOWN GAPS / TODOs / DEFERRED
- IPC latency ~0.1ms per message (acceptable, agent turns are seconds).
- Process spawn ~50ms (Rust binary fork + namespace, not cold start).
- Memory overhead ~20MB per agent process.
- Tier B: measure Firecracker vs gVisor on target hardware in Phase 3, pick lower latency.
- Windows isolation explicitly WEAKER than Linux. Linux is production security target. Windows = best-effort dev. Fallback: restricted token + Job object. Known weakness: NTFS junction/reparse points may bypass ACL deny — documented, not hidden.
- Branch limit: max 3 active branches per session (combinatorial explosion prevention).
- Stub line limit: <50 lines (≥50 rejected).
- Stub timeout: 25s graceful shutdown IPC, 30s hard kill via explicit `child.kill().await` (tokio kill_on_drop is false by default).
- SSB branching: per S10, each alternative creates `branch:stub_{step_id}_{n}`. Max 3.
- FeasibilityOutcome reused from reasoning crate (no duplication).

### Phase-aware tool/context loading per FSM state
- PLANNING: rules + knowledge results + plan template. NO edit/bash tools.
- FEASIBILITY: stub write (<50 lines) + bash + web search. NO full edit/git.
- TESTING_FIRST: test framework + plan + rules. NO edit/bash tools.
- EXECUTING: edit tools + current phase spec + code context. NO planning tools.
- AUDITING: validation tools + test runner + code. NO edit/planning tools.
- MEMORIZING: memory-bank write tools only.
- CONFIGURING: edit tools restricted to config paths only (`.elai_cc/`, `elai_cc.toml`, `rules/`, `templates/`); workflow domain. NO project source editing. NO knowledge engine queries.

### BSP superstep loop (pseudocode)
```
loop {
    channel_snapshot = channels.snapshot_for_scheduling();
    tasks = prepare_next_tasks(checkpoint, channel_snapshot, nodes);
    if tasks.empty() { break; }
    channels.clear_non_accumulating_topics();   // after snapshot, before execute
    writes = execute_tasks_parallel(tasks, model_pool, ssb);
    apply_writes(checkpoint, channels, writes);
    // deferred node phase (design 31.2)
    for node in get_deferred_nodes(nodes) {
        if node.all_inputs_satisfied(channels) {
            deferred_writes = execute_single(node, channels);
            apply_writes(checkpoint, channels, deferred_writes);
        }
    }
    channels.clear_ephemeral();   // recorded in SSB for crash recovery replay
    if !evidence_gate_passes(checkpoint, channels) {
        attempt += 1;
        if attempt >= max_superstep_attempts { break; }
        continue;
    }
    save_checkpoint(checkpoint, channels);   // AFTER gate (prevents filename collision)
    superstep += 1;
    attempt = 0;
}
```

### Context budget allocator (Slot ordering + cache breakpoints)
- Slot 1: system_prompt (GRC structured prompt per role).
- Slot 2: rules (active for current phase + file context).
- Slot 3: phase_spec (from PipelineSpec, includes output_format + constraints).
- --- cache breakpoint 1 (Slots 1-3 stable, cacheable) ---
- Slot 4: repo_map (PageRank DKG, fitted to budget).
- Slot 5: code_context (BM25 + vector chunks).
- --- cache breakpoint 2 (Slots 4-5 stable within same files) ---
- Slot 6: ssb_entries (filtered by file overlap).
- Slot 7: grc_history (prior reasoning for current topic).
- Slot 8: [output_reserve] (2000 tokens minimum).
- Budget split after fixed: repo_map 15%, code_context 40%, ssb 25%, grc 20%.
- Floors: ssb≥200, grc≥500, output≥2000. Floor enforcement: steal from code_context first, then repo_map. Err if insufficient.
- File overlap score: `|intersection(entry.files, agent.files)| / |agent.files|`.
- Adaptive reduction (max 3): 1) filter by relevance; 2) reduce granularity; 3) reduce scope. Terminal: INTERACTIVE→ESCALATE (Phase 3); AUTONOMOUS→SPLIT_TASK (Phase 4G). Never silently truncate.
- Auditor (Phase 3E+): Slot 5 split — DKG blast radius subgraph 30% + BM25/vector chunks 70%. Configurable. Cache breakpoint 2 applies to BM25/vector portion only (subgraph volatile). Non-auditor: no split.

### Agent checkpoint
- Location: `.elai_cc/sessions/{sid}/agents/{agent_id}/`.
- Files: `state.msgpack`, `grc_history.msgpack`, `tool_results.msgpack`, `output_draft.msgpack`, `checkpoint_ts`.
- Triggers: after GRC step, tool call, file edit, before slow ops.
- Recovery: orchestrator detects stop/crash → reads checkpoint → spawns new instance → injects checkpoint as structured context → agent continues from last GRC step.
- Anti-spiral: same_step_count > 3 → checkpoint + escalate.
- UntrackedValue exclusion: checkpoints MUST NOT reference UntrackedValue channel IDs (validated by serializer).

### Phase completion / model swap (S7B atomic sequence)
a. Agent sends AgentResponse via IPC.
b. Orchestrator validates against evidence gate.
c. Orchestrator writes SSB `phase_complete` ATOMIC DURABILITY MARKER `{ phase_id, agent_id, fsm_state, evidence_artifacts, commit_hashes, test_outcomes }`. Everything before = committed; everything after = idempotent on replay.
d. Orchestrator advances FSM state.
e. ModelPool unloads current model from GPU. Precondition: phase_complete fsync to WAL confirmed.
f. ModelPool loads next model (predictive pre-load if VRAM allows).
g. Spawn new agent process with next phase's AgentRequest.
h. Health check inference. Failure → reload previous from disk (~15-30s).

Crash recovery semantics:
- Crash before (c): re-dispatch same phase; agent re-runs from last internal checkpoint; SSB side effects append-only no conflict.
- Crash between (c)-(e): phase_complete present → skip re-execution, proceed to (e).
- Crash between (e)-(g): FSM advanced, model unloaded → load next model per FSM state, spawn agent.
- Crash during (g)/(h): kill orphan processes (stale PID), re-spawn.

### Predictive pre-loading (FSM-driven)
EXECUTING → pre-load auditor; AUDITING → pre-load executor; PLANNING → pre-load prompt-engineer; TESTING_FIRST → pre-load executor.

### Project isolation (S11)
- `project_id`: UUID v4.
- Directory: `.elai_cc/projects/{project_id}/` (ssb.db, knowledge.db, index/, rules/, etc.).
- Global shared: `.elai_cc/global/` (model_registry, rule_templates, pipeline_templates, metrics).
- One project on GPU at a time. FIFO queue with priority override. Switch: ~30-60s.
- Query-level scoping: every knowledge engine query MUST include project_id (no global search). Exception: `project_id="global"`.

### Tier C IPC details (S12)
- Transport: unix domain socket (Linux) / named pipe (Windows).
- Wire format: 1-byte version + 4-byte length prefix + postcard payload.
- MAX_FRAME_SIZE: 10MB hard cap.
- Connection sequence: (1) orchestrator creates 0700 directory `/tmp/elai_cc_{session}/`; (2) creates socket inside, listen(); (3) spawns agent with socket path arg; (4) agent connects; orchestrator verifies SO_PEERCRED matches spawned PID (Linux) or impersonation level SecurityIdentification (Windows); (5) version handshake — mismatch = kill.
- Crash safety: SSB write synchronous with IPC ToolResult dispatch; orchestrator writes SSB BEFORE sending ToolResult.
- agent_id: UUID v4 at spawn. Socket path: `/tmp/elai_cc_{session}/{agent_uuid}.sock`. No two live agents share path.
- Heartbeat 5s interval. AgentKill grace 1s flush.

### Linux process restrictions
- `clone()` with `CLONE_NEWNS | CLONE_NEWPID | CLONE_NEWNET`.
- All capabilities dropped via `prctl(PR_SET_NO_NEW_PRIVS)` BEFORE exec(). Zero capabilities. Researcher network: veth pair + iptables allowlist (no caps).
- seccomp-bpf two profiles:
  - **IPC_BASELINE (all roles)**: read, write, open, openat, close, mmap, mmap2, munmap, mprotect, brk, exit_group, clock_gettime, clock_getres, getrandom, futex, sigaction, rt_sigreturn, rt_sigprocmask, nanosleep, poll, epoll_ctl, epoll_wait, epoll_create1, eventfd2, socket(AF_UNIX only — BPF arg check rejects AF_INET/AF_INET6), connect, sendmsg, recvmsg, sendto, recvfrom, shutdown, getsockname, getpeername, setsockopt, getsockopt, fstat, stat, lstat, access, readlink, getdents64, ioctl(FIONREAD only), getpid, gettid, set_tid_address, arch_prctl, set_robust_list. All unlisted → EPERM (silent, no audit visible to agent).
  - **NETWORK_EXTRA (researcher only)**: socket(AF_INET, AF_INET6 — TCP only, BPF SOCK_STREAM check), connect (allowlisted IPs via iptables, not seccomp).
- Mount namespace: bind-mount project source RO + scratch dir RW. `.elai_cc/` not mounted (does not exist in agent view). Self-modification agent: locked files/dirs RO additionally. /proc masked (hidepid=2 or not mounted). /sys not mounted.
- PID namespace: agent sees only own PID (PID 1 inside namespace).
- rlimit: RLIMIT_NPROC=1, RLIMIT_FSIZE=100MB, RLIMIT_AS=512MB (configurable).

### Windows process restrictions
- AppContainer preferred over ACL deny. Filesystem (explicit grants), network (no default; researcher: add network capability), registry (no access), process (isolated from other AppContainers).
- Job object: JOB_OBJECT_LIMIT_ACTIVE_PROCESS=1, JOB_OBJECT_LIMIT_PROCESS_MEMORY=512MB, JOB_OBJECT_LIMIT_JOB_TIME (configurable timeout).

### Agent environment (sanitized)
Pass: LANG, TERM, PATH (restricted), TMPDIR (scratch). NOT pass: HOME, USER, API keys, ELAI_CC_* variables, shell history paths.

### Agent isolation cannot
See other agents' sandboxes; see orchestrator process; see GPU devices (no /dev/nvidia*); access host network stack; read /proc or /sys (restricted/fake); know it is in a sandbox.

### Self-modification extra restrictions
- IPC: orchestrator rejects WriteFile/EditFile to locked paths (canonical realpath comparison, not string match — prevents normalization attacks).
- Filesystem: locked dirs mounted RO; even BashRequest cannot write to RO mount.
- Git: orchestrator filters GitCommit, rejects if diff touches locked path.
- Auditor self-discovery trust boundary: source=AUDITOR → PROPOSED status NEVER auto-promoted to CANDIDATE; require human approval gate; second auditor instance (fresh process, no shared state) validates predicate logic; mutable=0 rules protected from all automated modification.

### Evidence gate protocol (S13)
- IDLE → PLANNING: knowledge engine readiness gate. READY or DEGRADED required. UNAVAILABLE → block, user must fix.
- PLANNING → TESTING_FIRST: zero UNGROUNDED claims in grounding_sweep_report.
- TESTING_FIRST → EXECUTING: all tests fail for right reasons (not syntax errors).
- EXECUTING → AUDITING: at least 1 test flipped FAIL → PASS.
- EXECUTING → PLANNING (fast-track replan): ReplanRequest(MINOR), auditor validates evidence, approved → transition back for current sub-phase only. Rejected → executor continues. Max 1 per phase.
- EXECUTING → PLANNING (major replan): ReplanRequest(MAJOR), requires human approval in BOTH AUTONOMOUS and INTERACTIVE.
- AUDITING → MEMORIZING: zero BLOCK findings remaining.
- MEMORIZING → next: DKG reflects new code, changelog has entry.

### Deterministic dispatch (S14, two tables)

**A. META-DISPATCH TABLE** (static, hardcoded):
| FsmState | Dispatch Entry |
|---|---|
| PLANNING | DispatchPipeline (4 stages: research, design, validate, probe) |
| CONFIGURING | Single stage: executor, config paths only, workflow domain |
| IDLE | No dispatch |
| COMPLETE | No dispatch (cleanup, transition to IDLE) |

PLANNING DispatchPipeline 4 stages:
1. **RESEARCH**: { role: Researcher, grc: 10/1200, tools: [web_search] }. input_from: None. output_artifact: ResearchReport. Gate: ArtifactExists(ResearchReport) AND phi ≥ phi_min convergence.
2. **PLAN ASSEMBLY (L1 designer)**: { role: Designer, grc: 8/1200, tools: [none — context via prompt] }. input_from: Stage 1. output_artifact: PipelineSpec. Gate: ArtifactExists(PipelineSpec) AND zero ungrounded claims. Designer DISTINCT from Executor.
3. **STRUCTURAL VALIDATION (L2 builder)**: skip_condition: NotAnAgent (pure Rust in orchestrator). Kahn's cycle detection, 7-point validation, Phase Sizing. output_artifact: ExecutablePipeline.
4. **FEASIBILITY PROBE**: { role: Executor with config overrides, grc: 3/600, tools: [stub_write, bash, web_search], timeout: 25s/30s }. input_from: Stage 3 (novel_step_ids). output_artifact: FeasibilityReport. skip_condition: novel_steps.is_empty() AND all resources ≥20% margin. AgentDispatchConfig is authoritative, not AgentRole alone.

**B. PLAN-DISPATCH TABLE** (dynamic, built from PipelineSpec at L2):
| FsmState | Agent Role | Tool Access | GRC Config |
|---|---|---|---|
| TESTING_FIRST | Executor | test framework, plan | 5/800 |
| EXECUTING | Executor | edit, bash, git | 8/1200 |
| AUDITING | Auditor | validation, test runner | 10/1200 |
| MEMORIZING | MemoryWriter | memory write only | 2/400 |

Built once per pipeline. Immutable until ReplanRequest.

Dispatch sequence per phase:
1. Read FSM state. Meta-dispatch (PLANNING/CONFIGURING) or plan-dispatch.
2. Dispatch table returns DispatchStage or DispatchPipeline.
3. Per stage: (a) check skip_condition; if met, run Rust directly; (b) assemble prompt (role_prompt + tier_3_rules + code_context + task_spec); (c) spawn agent process (Tier C), AgentRequest via IPC; (d) agent works (GRC chain, tool/LLM calls via IPC); (e) AgentResponse via IPC; (f) check gate — pass→next stage, fail→retry/escalate.
4. All stages → evidence gate → FSM transition.

Key properties: no LLM in dispatch loop; two tables not one; agent has zero knowledge of dispatch; session_id+pipeline_id+phase_id uniquely determines dispatch; ToolFilter is defense-in-depth on top of PreToolUse hooks; meta-dispatch hardcoded (not user-configurable); plan-dispatch built programmatically.

### Hierarchical plan management (S9)
- Plan skeleton (~2-4k tokens, always Slot 3): `.elai_cc/pipelines/{id}/skeleton.toml` (one-liner per phase: goal, key decision, interface contracts, cross-refs; version per section, updated only when phase elaboration approved).
- Phase detail (~5-15k, on demand): `.elai_cc/pipelines/{id}/phases/{phase_id}.toml`.
- Contract extraction pre-elaboration: input contracts + output contracts + shared constraints prepended to phase detail. Model elaborates AGAINST contracts.
- Coherence enforcement: auditor checks phase satisfies all input/output contracts; no new interfaces; version incremented if changed; skeleton change → propagate flags to dependent phases.
- Phase Sizing split: preserve_contracts flag; split at contract boundaries first, then directory proximity.
- Storage in SSB action_type='plan_update'. NOT in RAG/vector. SQLite query: `SELECT * FROM plan_phases WHERE phase_id IN (?, dependencies_of(?), dependents_of(?))`.

### Session handoff (S10, CMV-derived, replaces flat handoff.msgpack)
- DAG-based; SQLite at `.elai_cc/sessions/{id}/session_dag.db` (schema in §6).
- Checkpoint trigger: after every GRC step, tool call, phase transition.
- Handoff trigger: token budget threshold (configurable, e.g. 85%).
- Structurally lossless trimming (3 streaming passes): (1) identify large tool results > stub_threshold tokens (default 500); (2) replace with stubs (type, size, summary hash) — preserve ALL user/assistant intact; (3) validate stub references (every stub points to SSB seq for recovery).
- Stubs recoverable: new session re-executes tool call or reads from SSB.
- Prompt cache aware: trimming preserves prefix tokens in Slots 1-3.
- Branch and explore: snapshot tag="branch:{name}". Multiple approaches from same checkpoint. Merge: pick winning, discard others (only winning branch's SSB kept). Branch limit max 3 active per session.
- Handoff: snapshot tag="handoff", run trimming. Load: read latest handoff snapshot. If branch exists, load branch tip not trunk. No summarization.
- Session-owned commits (design 31.11): `commit_hashes: Vec<String>` per session; safe undo only session-owned, working tree clean, not pushed, has parent.

### Cognitive isolation metric
`context_savings_from_isolation = sum(all_agent_grc_tokens_this_phase) / model.max_context`. Logged action_type='isolation_metric' at each phase transition. On 8k local model: 12k internal reasoning saves 150% context (makes task possible). On 32k cloud: 30-40%.

### Dynamic fan-out (S8)
- Agent emits Send(target_role, input_data) objects.
- Namespace: `"{parent_ns}|{subgraph_type}:{counter}"` (AtomicU64 per type).
- Counter checkpointed at superstep boundary.
- Resource-aware: checks ModelPool. Insufficient VRAM → serialize, not parallel.

### Hardware detection
nvml-wrapper (GPU VRAM via `device.memory_info()`); sysinfo (RAM via `system.total_memory()`, disk via `disk.available_space()`).

### Stub execution
Reuses Tier C agent process (S12) with FEASIBILITY tool restrictions. Output: `stub_results.msgpack` (FeasibilityReport via rmp-serde).

### Claw-code mapping
- worker_boot.rs (1,069 LOC) → AgentPool.
- policy_engine.rs (581 LOC) → rules::RuleEngine (shared with rules/).
- permission_enforcer.rs (546 LOC) → rules enforcement layer.
- green_contract.rs (4,061 LOC) → evidence gates + contract tests.
- mcp_lifecycle_hardened.rs (839 LOC) → plugins/ error recovery.
- recovery_recipes.rs (630 LOC) → crash recovery.
- trust_resolver.rs (299 LOC) → interactive gating.
- session_control.rs (461 LOC) → session management.
- task_registry.rs (506 LOC) → task tracking.
- lane_events.rs (241 LOC) → evaluate for SSB action_types.
- task_packet.rs (162 LOC) → structured task data.
- STRIP: compact.rs, summary_compression.rs (never compact).

==========================================================================

## CRATE: cli/

### 1. RESPONSIBILITY
Thin shell (most logic in other crates): REPL loop (read user input, route to orchestrator), first-message routing (slash-command dispatch + fallback classifier), output rendering (syntax highlighting, progress display), argument parsing, mode switching INTERACTIVE (default Phase 3) ↔ AUTONOMOUS (Phase 4G).

### 2. DEPENDENCIES
core/, orchestrator/, tools/, providers/. KEEP: rustyline (line editing), pulldown-cmark (markdown), syntect (syntax highlighting), crossterm (terminal control).

### 3. PUBLIC TRAITS
None enumerated.

### 4. PUBLIC TYPES
- Slash commands (parsed before orchestrator):
  - `/task <prompt>`: Normal workflow. IDLE → PLANNING → ... → IDLE.
  - `/config <prompt>`: Edit `.elai_cc/` config. IDLE → CONFIGURING. Targets `elai_cc.toml`, `rules/`, `templates/`.
  - `/workflow <prompt>`: Edit workflow tooling. IDLE → CONFIGURING. Targets `CLAUDE.md`, `.claude/`, hooks.
  - `/fix <prompt>`: Debugging. IDLE → TESTING_FIRST (skip PLANNING for known-broken state).
  - `/continue`: Resume from `handoff.msgpack`. Loads FSM state, continues from last checkpoint.
  - `/explore <prompt>`: Read-only query. Knowledge engine answers, no FSM change.
  - `/status`: Show current FSM state, active sessions, GPU status, plan progress.
  - `/register-command <name> <fsm_target> <description>`: extensible custom commands stored in `.elai_cc/commands.toml`.
- Fallback classifier (bare messages):
  - file paths + fix/bug/error keywords → `/fix`.
  - "change/modify/update" + config/workflow/rule paths → `/config`.
  - "?" + no action verbs → `/explore`.
  - Default → `/task`.
- CLI strips prefix, sets `initial_route` in TaskPacket, passes to orchestrator. Orchestrator reads initial_route to determine FSM entry state and tool/rule loading.

### 5. ERROR TYPES
None enumerated.

### 6. SQLITE SCHEMAS
None.

### 7. CONFIG KEYS
- `.elai_cc/commands.toml` for custom commands. Loaded at REPL startup.

### 8. SECTIONS
S1 RESPONSIBILITIES; S2 FIRST-MESSAGE ROUTING (slash-command dispatch); S3 CLAW-CODE MODULE MAPPING; S4 DEPENDENCIES; S5 TESTABLE INVARIANTS.

### 9. PHASE OWNERSHIP
- Phase 3: INTERACTIVE mode (default).
- Phase 4G: AUTONOMOUS mode (explicit switch). Mode switch takes effect on next phase boundary (not mid-phase).

### 10. KNOWN GAPS / TODOs / DEFERRED
- claw-code `rusty-claude-cli/main.rs` (7,749 LOC) → cli/ (strip to essentials). Most of main.rs is orchestration logic that belongs in orchestrator/. cli/ should be a thin shell.

### Testable invariants
- Slash-command prefix parsed before orchestrator dispatch.
- `/task` is default route when no prefix and classifier has no strong match.
- `/continue` loads handoff.msgpack and resumes exact FSM state.
- `/config` and `/workflow` restrict tool access to config-target paths only.
- `/explore` never triggers FSM state change (read-only).
- Custom commands from commands.toml load correctly at startup.
- CLI passes input to orchestrator without modification (after prefix stripping).
- INTERACTIVE is the default mode; AUTONOMOUS requires explicit switch (Phase 4G).
- Mode switch takes effect on next phase boundary.
- Rendering handles all output formats (code, markdown, progress, errors).

==========================================================================

## DEPENDENCY GRAPH

Leaf-to-root build order (one edge per line):

```
core/                                  (leaf — no internal deps)
providers/      -> core/
rules/          -> core/                   [Box<dyn AstEvaluator>, Box<dyn DkgQuerier> injected at construction]
knowledge/      -> core/
knowledge/      -> providers/              (for embeddings)
tools/          -> core/
tools/          -> knowledge/              (tree-sitter AST shared)
plugins/        -> core/
plugins/        -> rules/                  (hook enforcement)
reasoning/      -> core/
reasoning/      -> providers/              (model invocation)
reasoning/      -> tools/                  (verification checks)
orchestrator/   -> core/
orchestrator/   -> providers/              (ModelPool)
orchestrator/   -> reasoning/              (GRC)
orchestrator/   -> rules/
orchestrator/   -> tools/
cli/            -> core/
cli/            -> orchestrator/
cli/            -> tools/
cli/            -> providers/
```

Linearized: **core → providers/rules → knowledge/tools/plugins → reasoning → orchestrator → cli**.

Notes:
- knowledge/ implements `AstEvaluator` and `DkgQuerier` (defined in rules/), passed back into rules/ as `Box<dyn ...>` at construction time. NO static `rules/ → knowledge/` dependency edge.
- rules/ also implements no static dep on knowledge/; testable with `MockDkgQuerier` and mock `AstEvaluator` before knowledge/ is available.
- plugins/ depends on rules/ for hook enforcement (T1 RULES tier evaluator).
- orchestrator/ does NOT directly depend on knowledge/ (uses rules/ + reasoning/ which themselves consume knowledge via injected traits or providers).
- cli/ depends on tools/ AND providers/ in addition to orchestrator/ (per cli.txt L4).
- Tools/ imports `api::*` (line 6) → must update to `providers::` during restructure.
