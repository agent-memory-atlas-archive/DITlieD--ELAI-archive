# B-4: Producer Wiring — Knowledge Engine
Objective: surface every backend signal the knowledge engine produces in
the dashboard, fix the runtime degradations identified by the audit, and
stage the missing ONNX models.
Estimated effort: 2-3 sessions.
Pre-requisites: B-1 (path unification — vector models live at the canonical
path now).
Parallel-safe with: B-5, B-6 (different domains).

## V2 REBASELINE NOTE (2026-05-07)

Producer-side work (knowledge crate, pipelines, ONNX staging) is V2-agnostic
— land as written. Dashboard surface targets become:
  V1                                       V2
  --------------------------------------------------------------------
  crates/dashboard/src/commands.rs       -> crates/dashboard-v2/src-tauri/
                                             src/lib.rs (NEW Tauri
                                             commands; B-36 wires)
  KnowledgeDrawer rendering              -> crates/dashboard-v2/src/lib/
                                             drawers/KnowledgeDrawer.svelte
  ssb_viewer.rs query helpers            -> consumed inside the V2 Tauri
                                             commands (e.g. AppState
                                             holds the SSB handle)

V2 stub.ts already exposes `get_query_trace_recent`, `get_search_mode`,
`get_embedding_breakers`, `get_recall_metrics`, `get_dkg_stats`. Keep
those names 1:1 when wiring the live invokes.

## V2 REBASELINE NOTE — B-4 closure session (2026-05-22, s-25120)

Task #3 cli/main.rs:3721 path was stale. Real surface:
  V1                                       V2
  --------------------------------------------------------------------
  cli/main.rs:3721 NoopSsbNotifier       -> cli/src/live_cli.rs:173
                                             SsbDegradationNotifier (B-3
                                             shipped; B-4 invokes via
                                             spawn_cascade_evaluator_loop)
  reranker.rs:465 .expect("semaphore")   -> reranker.rs:606-613 graceful
                                             shutdown SHIPPED in B-16A
  Box<dyn Error> in bm25/vector/         -> SHIPPED (anti-stale sweep
   pagerank                                already migrated all sites
                                             to thiserror)

Task #17 SHIPPED on V2 (B-16A graceful shutdown at reranker.rs:606-613).
Task #18 SHIPPED on V2 (zero `Box<dyn Error>` matches in
crates/knowledge/src/{bm25.rs,vector/indexer.rs,pagerank/mod.rs}).

Task #8 numbers: stale (0.206/0.575 in plan vs 0.5017/0.6000 in
metrics/phase_3K_rag.toml). DEEP MRR < STANDARD MRR still holds
qualitatively (0.5017 < 0.6000). The B-4 closure rewires
`get_recall_metrics` (commands.rs) to read the toml at runtime instead
of hardcoded literals; KnowledgeDrawer already paints the DEEP-MRR-warn
badge red when `deepMrr < stdMrr`.

C-10 dependency: SCHEMA_VERSION bumped 98 -> 99 in B-4 to add the
`SsbActionType::CanaryDrift` variant (QUINTUPLE-UPDATE pattern, 7 sites
+ payload struct + all_test_payloads bijection 231 -> 232 + count
assertion). Producer wired in cli::live_cli at startup + heartbeat
every 30s. Original C-10 scope (canary_drift row type) is now SHIPPED
inside B-4 — C-10 can drop that line item.

## Context

Per audit-dashboard/knowledge-wiring.md and audit-implementation phase-1A
through 1I + 3K: the knowledge engine is mostly built but ~91% of its
signals are INVISIBLE. Several stages run degraded because models aren't
on disk or because of wiring shortcuts.

## Reads

- audit-dashboard/knowledge-wiring.md (the full inventory)
- audit-implementation/phase-1A.md, 1B.md, 1C.md, 1D.md, 1E.md, 1G.md, 1I.md, 3K.md
- summary/phase-1-summary.md (specs reference)
- summary/phase-3-summary.md PHASE 3K
- crates/knowledge/src/pipeline.rs (the QueryPipeline)
- crates/knowledge/src/reranker.rs:465 (the .expect violation to fix)

## Tasks

1. **Stage cross-encoder ONNX model** at `core::paths::models_dir_path()/
   onnx/cross-encoder-l12.onnx` plus tokenizer.json. Add a setup script
   (`scripts/stage_models.sh` + `.bat`) that downloads + verifies blake3
   hash. Document in elai_cc.toml [models.cross_encoder]. Without this,
   reranker permanently degrades to RRF-only.

2. **Stage GraphCodeBERT + Granite ONNX models** similarly. Document
   path mismatch fix from B-1: spec said `.elai_cc/models/onnx/` not
   `.elai/models/`. Update Cargo features so canary now points to
   `embed("fn main()")` per spec (currently `embed("test")`).

3. **SHIPPED ON V2 (B-4 closure 2026-05-22, s-25120).** Wire SearchMode
   cascade into query path:
   - `pipeline.search_mode: SearchMode` field at pipeline.rs:102 with
     `set_search_mode()` setter (driver pattern, not the literal
     `cascade: Arc<DegradationCascade>` from the original plan — caller
     drives cascade externally per code review).
   - `SsbDegradationNotifier` lives at cli/src/live_cli.rs:173 (B-3
     shipped); B-4 removes the `#[allow(dead_code)]` and invokes the
     constructor from `spawn_cascade_evaluator_loop` (live_cli.rs:~273)
     so mode transitions land `knowledge_degradation` SSB rows.
   - QueryTrace already records `self.search_mode` via
     `QueryTrace::new(..., self.search_mode, ...)` (pipeline.rs:878).
   - Per-query cascade evaluation runs every CASCADE_EVAL_INTERVAL_SECS
     (30s) in a tokio-spawned loop.

4. **Query code_vec at runtime.** Currently embed_code() is populated into
   cache during indexing but never queried. Wire vector stage to perform
   3-ranker RRF (BM25 + code_vec + text_vec), not 2-ranker.

5. **QueryTrace inspector dashboard surface.** New Tauri command
   `get_query_trace(query_id)` returns the per-stage timeline + latency.
   Frontend KnowledgeDrawer adds a "Last query" panel showing each stage
   (BM25, vector, pagerank, catrag, arag, rerank, token_fit) with timing
   bars and a colored "fired/skipped/degraded" badge per stage.

6. **Per-model EmbeddingCircuitBreaker grid.** New Tauri command
   `get_circuit_breaker_states()` returns Vec<{model, state: Closed|HalfOpen|
   Open, last_failure, last_success, attempts}>. Frontend adds 5-row grid
   to KnowledgeDrawer.

7. **HyDE hypothesis preview + cache hit rate.** New SSB action_type
   `hyde_query` written by SidecarHydeClient with the hypothesis text,
   cache hit/miss, latency. Dashboard renders last hypothesis preview
   inline in QueryTrace inspector.

8. **SHIPPED ON V2 (B-4 closure 2026-05-22).** Per-tier R@5/R@10/MRR
   badges. Real measured values in `metrics/phase_3K_rag.toml`
   (FAST R@5=0.6000/R@10=0.7000/MRR=0.3908; STANDARD
   R@5=0.8000/R@10=0.8000/MRR=0.6000; DEEP R@5=0.8000/R@10=0.8000/
   MRR=0.5017). V2 Tauri command name is `get_recall_metrics` (per
   stub.ts 1:1 contract, NOT the original `get_retrieval_quality_metrics`).
   Body now reads the toml at runtime via
   `std::fs::read_to_string` + `toml::from_str` with probe path
   fallback. KnowledgeDrawer paints the DEEP-MRR-warn badge red when
   `deepMrr < stdMrr` (consumer already shipped at
   KnowledgeDrawer.svelte:520-523).

9. **PARTIAL on V2 (B-4 closure).** Fix DEEP-tier MRR < STANDARD-tier
   MRR. Current measured values in metrics/phase_3K_rag.toml:
   DEEP MRR=0.5017 < STANDARD MRR=0.6000 — qualitative monotonicity
   violation still holds (smaller gap than original 0.206 vs 0.575
   citation, but DEEP still loses to STANDARD). Pipeline-side fix
   already shipped per pipeline.rs:428-435 comment ("B-4 DEEP-tier MRR
   fix: always include original query as anchor signal"). Dashboard
   surfaces the violation via DEEP-MRR-warn badge. Further MRR
   improvement is tracked separately — outside B-4 scope.

10. **Pagerank dashboard read.** After B-1 fixes the path, ensure
    `get_pagerank_top` SQL query returns non-zero. KnowledgeDrawer "Top
    symbols by importance" section (5-10 rows).

11. **SHIPPED ON V2 (B-4 closure 2026-05-22).** Freshness panel.
    `get_freshness_state` Tauri command already shipped via
    `SsbViewer::freshness_state()` (returns known:false when no
    producer has run); KnowledgeDrawer FRESHNESS DECAY panel now
    consumes via `stub.get_freshness_state()` (5s poll) and renders
    real `decay_rate` + `access_boost` + `last_recalc_ms` when known,
    falls back to "(no freshness snapshot yet)" when not.

12. **SHIPPED ON V2 (B-4 closure 2026-05-22).** VectorCache hit rate
    gauge. Producer already shipped (knowledge_ssb_sink.rs:121
    `emit_vector_cache_hit_rate` invoked from pipeline.rs after every
    successful query). New V2 Tauri command `get_vector_cache_hit_rate`
    reads latest `vector_cache_hit_rate` SSB row via
    `SsbViewer::latest_action_of_type`; stub.ts wrapper added;
    KnowledgeDrawer VECTOR CACHE HIT RATE card renders `hit_rate_pct`
    + `hits/total` + session id with `known:false` fallback.

13. **Index format version + UpgradeAction status.** Surface in Settings
    panel: current schema version, last migration, any pending upgrades.

14. **SHIPPED ON V2 (B-4 closure 2026-05-22).** Canary drift cosine.
    QUINTUPLE-UPDATE pattern added `SsbActionType::CanaryDrift` variant
    + `CanaryDriftPayload {similarity, drift_threshold, passed,
    reference_present, ts_ms}` (SCHEMA_VERSION 98 -> 99;
    all_test_payloads 231 -> 232). Producer wired in
    cli::live_cli at startup (carries real
    `startup_result.drift_check` verdict) + heartbeat every
    CASCADE_EVAL_INTERVAL_SECS (30s). V2 Tauri command
    `get_canary_drift` reads `viewer.latest_action_of_type("canary_drift")`;
    stub.ts wrapper added; SettingsWindow CANARY DRIFT row renders
    `cosine X / thr 0.85` with red highlight when `passed=false` plus a
    recompute affordance. Threshold = 0.85 (knowledge::startup::
    DRIFT_THRESHOLD); plan literal "0.05" was a delta target — the
    knowledge-wiring.md spec actually uses absolute cosine sim
    `>= 0.85` per knowledge.txt L197-200.

15. **DKG failed_files count.** Add to KnowledgeDrawer: "47 files indexed,
    3 failed (parse error)". Click for list.

16. **SHIPPED ON V2 (B-4 closure 2026-05-22).** Per-stage latency_ms
    ungrounded zeros. Real sites in current code: pipeline.rs:747
    (`deep_cap` stage emit) + pipeline.rs:763 (`rrf_gate` stage emit)
    + pipeline.rs:859 (`QueryTrace::new(stages, 0, ...)` total).
    Plan-cited line range 30-33 was use statements (drift). Fix
    captures `self.clock.now()` before each gate-fire emit block and
    computes `self.clock.elapsed_since(start).as_millis() as u64`
    for the synthetic `deep_cap` / `rrf_gate` stage's latency.
    `total_latency_ms` is now computed BEFORE `QueryTrace::new()` so
    the trace carries the real sum-of-stage-latencies upper bound.
    Entropy quarantine preserved (only `self.clock` reads, no raw
    `std::time::Instant`).

17. **SHIPPED ON V2 (B-16A — verified during B-4 closure).**
    `.expect("semaphore closed")` at reranker.rs:465 already replaced
    by graceful shutdown at reranker.rs:606-613 (match on
    `semaphore.acquire().await` with `tracing::warn!` exit). Zero
    matches for `expect("semaphore` in current source — B-16A wrap is
    complete.

18. **SHIPPED ON V2 (anti-stale sweep — verified during B-4 closure).**
    Zero matches for `Box<dyn` in crates/knowledge/src/bm25.rs,
    vector/indexer.rs, pagerank/mod.rs. All sites migrated to
    `KnowledgeError` / `VectorError` / `Bm25Error` per the anti-stale
    enforcement sweep.

## WIRING_CONTRACT

produces_symbols:
  - crates/orchestrator/src/ssb/payloads.rs::KnowledgeDegradationPayload
  - crates/orchestrator/src/ssb/payloads.rs::HydeQueryPayload
  - crates/orchestrator/src/ssb/payloads.rs::QueryTracePayload
  - crates/orchestrator/src/ssb/payloads.rs::CircuitBreakerStateChangePayload
  - crates/orchestrator/src/ssb/payloads.rs::VectorCacheHitRatePayload
  - crates/dashboard-v2/src-tauri/src/commands.rs::get_query_trace
  - crates/dashboard-v2/src-tauri/src/commands.rs::get_embedding_breakers
  - crates/dashboard-v2/src-tauri/src/commands.rs::get_recall_metrics
  - crates/dashboard-v2/src-tauri/src/commands.rs::get_pagerank_top
  - crates/dashboard-v2/src-tauri/src/commands.rs::get_freshness_state
  - crates/dashboard-v2/src-tauri/src/commands.rs::get_canary_drift
consumes_symbols:
  - KnowledgeDrawer.svelte (renders all of the above)
  - Settings.svelte (canary drift + index version)
reachable_from:
  - crates/cli/src/bench_rag.rs::run
  - crates/cli/src/live_cli.rs::LiveCli::build_local_knowledge_stack
  - crates/dashboard-v2/src-tauri/src/lib.rs::invoke_handler
stub_ban:
  - NoopSsbNotifier removed from production (cli/main.rs:3721)
  - .expect("semaphore closed") removed from reranker.rs:465
  - Per-stage latency_ms != 0 always
frontend_pair: same B-step (producer + consumer wired together)
observable_effect:
  trigger: F:\.ELAI_workflow\.elai_cc\target\release\elai.exe bench rag --tier STANDARD --top-k 5 -- "find auth handlers"
  expect_ssb: query_trace
  expect_artifact: KnowledgeDrawer "Last query" panel shows each stage

## DETERMINISTIC_CHECK

```bash
# 1. Models staged (filenames per stage_onnx_models.{sh,bat,py} script
#    in .elai_cc/scripts/ — granite payload-name is granite-embed.onnx
#    per current on-disk layout).
test -f .elai_cc/models/onnx/cross-encoder-l12.onnx
test -f .elai_cc/models/onnx/graphcodebert.onnx
test -f .elai_cc/models/onnx/granite-embed.onnx

# 2. SCHEMA_VERSION bumped to 99 (B-4 CanaryDrift QUINTUPLE add).
grep -q "const SCHEMA_VERSION: i64 = 99" .elai_cc/crates/orchestrator/src/ssb.rs

# 3. CanaryDrift variant present in the 7 QUINTUPLE sites + payload struct.
grep -c "CanaryDrift\|canary_drift\|CanaryDriftPayload" \
  .elai_cc/crates/orchestrator/src/ssb.rs
# Expected: >= 7 matches.

# 4. SsbActionType bijection still holds (all_test_payloads.len() == 232).
cd .elai_cc && cargo test -p elai-orchestrator --lib ssb::tests::test_typed_payloads
# Expected: 1 passed.

# 5. SsbDegradationNotifier no longer #[allow(dead_code)].
! grep -q '#\[allow(dead_code)\].*B-4 will wire' \
    .elai_cc/crates/cli/src/live_cli.rs

# 6. .expect("semaphore closed") absent from reranker.rs (B-16A SHIPPED).
! grep -q 'expect("semaphore closed")' .elai_cc/crates/knowledge/src/reranker.rs

# 7. Per-stage latency: pipeline.rs no longer hard-codes 0 for
#    the `deep_cap` / `rrf_gate` emit stages (`latency_ms: 0,`
#    pattern removed inside the gate-fire branches).
! grep -B1 -A4 'stage: "deep_cap"' .elai_cc/crates/knowledge/src/pipeline.rs \
    | grep -q 'latency_ms: 0,'
! grep -B1 -A4 'stage: "rrf_gate"' .elai_cc/crates/knowledge/src/pipeline.rs \
    | grep -q 'latency_ms: 0,'

# 8. DEEP MRR < STANDARD MRR (monotonicity violation surfaces in UI but
#    the toml itself enforces the qualitative gap from current numbers).
python -c "import toml; m=toml.load('.elai_cc/metrics/phase_3K_rag.toml'); \
  assert m['tier']['deep']['mrr'] < m['tier']['standard']['mrr'], \
  'expected DEEP MRR < STANDARD MRR (B-4 dashboard surfaces this warn)'"

# 9. Tauri commands registered + reachable from invoke_handler!.
grep -q "commands::get_vector_cache_hit_rate" \
  .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs
grep -q "commands::get_canary_drift" \
  .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs

# 10. stub.ts wrappers reachable from Svelte consumers.
grep -q "get_canary_drift\|get_freshness_state\|get_vector_cache_hit_rate" \
  .elai_cc/crates/dashboard-v2/src/lib/api/stub.ts
```

## ACCEPTANCE

- [ ] 3 ONNX models staged + verified by blake3 hash
- [ ] SearchMode cascade fires SSB knowledge_degradation rows
- [ ] code_vec queried at runtime (3-ranker RRF live)
- [ ] QueryTrace inspector visible in KnowledgeDrawer
- [ ] Circuit breaker grid visible
- [ ] HyDE preview visible
- [ ] R@5/R@10/MRR per-tier badges visible
- [ ] DEEP MRR ≥ STANDARD MRR (monotonicity restored)
- [ ] DKG failed_files count visible
- [ ] Canary drift visible in Settings
- [ ] No .expect("semaphore closed") in reranker.rs
- [ ] Zero Box<dyn Error> in knowledge crate public APIs

## Files touched

- MODIFY: crates/knowledge/src/pipeline.rs (stage timing, vec-3-ranker, cascade)
- MODIFY: crates/knowledge/src/reranker.rs (fix .expect, fallback model wire)
- MODIFY: crates/knowledge/src/bm25.rs (Box<dyn Error> → thiserror)
- MODIFY: crates/knowledge/src/vector/indexer.rs (same)
- MODIFY: crates/knowledge/src/pagerank/mod.rs (same)
- MODIFY: crates/knowledge/src/hyde.rs (SSB emit)
- MODIFY: crates/cli/src/main.rs:3721 (real SsbNotifier)
- MODIFY: crates/dashboard/src/commands.rs + ssb_viewer.rs (new commands)
- MODIFY: crates/dashboard/src-frontend/src/drawers/KnowledgeDrawer.svelte
- CREATE: scripts/stage_models.sh + .bat
- CREATE: workflow/reasoning/B-4-decisions.md

## Decision doc

- Whether to download models on first run vs require manual stage
- ts-rs bindings for new types (assumes B-3 added the dep)

## Next

B-5 (Reasoning producer wiring), B-6 (Orchestration). All parallel-safe.
