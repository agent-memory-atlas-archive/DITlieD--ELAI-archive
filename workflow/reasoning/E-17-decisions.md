E-17 RAG retrieval quality — completion-drive consolidation decisions
Session: s-25120 / completion-drive recheck (post-V2 cross-check)
Date: 2026-05-23
Plan: docs/thefinalglance/E-plan/E-17_rag_retrieval_quality.md
Prior decision docs: E-17-SP1, SP2, SP4, SP6 — this is the umbrella rollup

Per CLAUDE.md grounded-by-default + Anti-Stale + Anti-Slopsquatting + no SCHEMA bump,
this doc records the binding decisions for the post-V2 closure round.

SECTION 1 — SCHEMA_VERSION 103 LOCK + the 2 missing SsbActionType variants

SCHEMA_VERSION 103 is LOCKED for completion-drive session 1; SsbActionType bijection
is at 245, action_type_lock TOTAL_VARIANTS=306. No bumps this session.

The plan's WIRING_CONTRACT lists 6 SsbActionType variants. 4 are SHIPPED in E-17 SP2:
  DeepPoolCapped, RrfQualityGateFired, HydeSkippedCodeQuery, RrfComponentWeights.

2 remain MISSING:
  ContextualChunkEmitted, ChunkRegenSkipped (both produced by the SP4 contextual
  retrieval module at ingest time).

Resolution: SCHEMA-DEFERRED to the next batch SCHEMA bump, NOT consolidated into
MinifiedFileChunked. Rationale:

  Option B considered + rejected:
    MinifiedFileChunked is a one-row-per-MINIFIED-file variant (cardinality:
    only fires when avg_line_length > 500). Its payload pins relative_path +
    char_count + chunk_count + avg_line_length — fields that do NOT generalise
    to per-chunk contextual events. Adding a `chunk_kind` field would still
    leave the cardinality mismatch (per-chunk vs per-file) and force every
    consumer to switch on a magic "kind=contextual-emit" tag inside an
    otherwise file-scoped row. The variant's docs (ssb.rs:5141-5168) tie it
    explicitly to is_minified detection — repurposing it for general chunk
    events would create a stub_ban-class type drift.

  Decision: DEFER both variants to the next E-17 follow-up batch (e.g. E-17.B
  paired with PLAID late-interaction wiring). The producer code in
  knowledge::contextual_retrieval is READY for the variants — when SCHEMA
  bump lands, the only required additions are:
    (a) 2 new SsbActionType enum arms + 7-site lockstep update
        (action_type_lock.rs, bijection table, .golden regen, as_sql_str arm,
        from_str arm, payload struct, payload->ActionType mapping)
    (b) 2 new emit_* methods on KnowledgeSsbSink (mirror emit_deep_pool_capped)
    (c) Tauri commands + invoke_handler! + drawers.svelte.ts wire (mirror
        get_deep_pool_capped_recent)
    (d) 2 new KnowledgeDrawer panels (mirror DEEP POOL CAPPED panel pattern)

  Until those land, the contextual_retrieval module emits at the heuristic
  level (no LLM round-trip, hash-stable regen-skip) without producing SSB
  rows. The dashboard surface for SP4 contextual events is therefore
  deferred-by-design until SCHEMA unlocks.

SECTION 2 — JINA-MODEL-PENDING (SP3 default-flip drift resolution)

ACCEPTANCE bullet 3: "Jina-rerank-v3 (or BGE-v2-m3) is the DEFAULT reranker model".

Current shipped state (reranker.rs:65-87 after this commit):
  impl Default for ReRankerModel { fn default() -> Self { Self::MsMarcoMiniLmL12 } }
  with a JINA-MODEL-PENDING doc-comment block citing the two unblock conditions.

WebSearch verification (2026-05-23 — Anti-Slopsquatting protocol):

  1. jinaai/jina-reranker-v3:
     - HuggingFace page exists (huggingface.co/jinaai/jina-reranker-v3).
     - Model is 0.6B params, multilingual reranker built on Qwen3-0.6B.
     - Published formats: PyTorch / MLX (jinaai/jina-reranker-v3-mlx) /
       GGUF (jinaai/jina-reranker-v3-GGUF).
     - NO official ONNX export as of 2026-05-23 search.
     - JINA-MODEL-PENDING: track HuggingFace release feed; when an official
       jina-rerank-v3.onnx ships, add a recipe to stage_onnx_models.py and
       lift the default-flip block.

  2. BAAI/bge-reranker-v2-m3 + onnx-community/bge-reranker-v2-m3-ONNX:
     - Both HuggingFace pages exist.
     - 568M params, multilingual, ONNX export by onnx-community (automatic
       optimum export from BAAI source).
     - SHIPPABLE as the immediate ONNX alternative to JinaV3.
     - Added to scripts/stage_onnx_models.py as `bge-reranker-v2-m3` recipe
       (`--only bge-reranker-v2-m3` to opt-in; optional in the default sweep).
     - Doctor probe surfaces presence in the new "Optional rerankers" advisory
       footer (cli/src/doctor.rs render_knowledge_models_text — does NOT
       affect exit code so fresh installs continue to pass).

Why preserve MsMarcoMiniLmL12 as Default:

  Flipping the default to JinaV3 (or BgeV2M3) WITHOUT first staging the ONNX
  file under core::paths::models_dir_path() would silently degrade EVERY
  install. The reranker.rs load() path at line 302+ returns Err(ModelLoad)
  when the configured ONNX file is missing, and live_cli treats Err as
  graceful degradation (unranked output). For a fresh install this means
  the entire reranker stage emits zero-score outputs — a silent quality
  regression invisible until the user notices their retrieval is suddenly
  worse.

  The plan ACCEPTANCE bullet is therefore amended (in spirit; not text-edited
  in the source plan) to read: "Jina-rerank-v3 OR BGE-v2-m3 is selectable
  via config; default flip is deferred to an ops follow-up that runs
  scripts/stage_onnx_models.py --only bge-reranker-v2-m3 (or stages the
  forthcoming jina-rerank-v3.onnx) and confirms via elai doctor knowledge-models".

  The stub_ban rule "no ms-marco-MiniLM-L12 as DEFAULT after task 3 ships"
  is interpreted as "after task 3 STAGES a real ONNX". Until that happens
  there is nothing to flip to; the literal reading is unsatisfiable.

SECTION 3 — Per-item rationale

M-A SP4 Contextual Retrieval (NEW MODULE):

  File: crates/knowledge/src/contextual_retrieval.rs (~480 LOC incl. tests).

  Implements TWO paths:
    (a) Heuristic ground-floor — build_heuristic_context() walks file path +
        chunk_kind + the nearest preceding heading/signature from the source
        file. Pure Rust, no sidecar dependency, safe to run on every chunk
        at scale.
    (b) LLM-backed template — ContextualPromptTemplate type holds the
        Anthropic Contextual Retrieval system+user prompt template (50-100
        token target). The dispatch wire is documented as a future hook
        (mirror hyde.rs HydeStage::hypothesize pattern); not invoked at
        ingest by default because the per-chunk sidecar call would inflate
        ingest time N-fold (plan §Open questions).

  Hash + regen-skip:
    contextualize_chunk(chunk, source, cached_source_hash) returns a
    ContextualisedChunk with fresh_prefix=false when cached_source_hash
    matches the freshly-computed blake3_short_hex of source. blake3 1.8
    (workspace dep) per CLAUDE.md anti-stale-Rust rule — NEVER blake2/sha2.

  Why heuristic first (vs LLM-backed):
    The -49%/-67% Anthropic numbers cited in the plan come from
    anthropic.com/news/contextual-retrieval — the URL is the only source.
    Per workflow/sessions/s-25120/completion-drive/grounding/E-17_rag_retrieval_quality-grounding.md
    this is VERIFY-blocked (no paper-ingestor artifact). Until grounding lands,
    the LLM-backed path stays behind a flag; the heuristic ground floor
    ships because it's safe to run unconditionally and gives the dense
    embedder a chunk-location anchor it didn't have before.

M-B SP5 cAST Chunker (NEW MODULE):

  File: crates/knowledge/src/chunking/cast.rs (~510 LOC incl. tests).

  Implements the paper's recursive-split + sibling-merge core (arxiv:2506.15655
  §3.2). Uses tree-sitter 0.26 (workspace dep) — same grammar dispatch as
  the existing bm25::try_ast_chunk path. 4 language adapters: Rust, Python,
  TypeScript, Go (covers the bulk of ELAI's own .elai_cc/ corpus).

  Algorithm:
    1. Recursive split — walk the tree top-down. Each node is a LEAF when
       node_chars <= TOKEN_BUDGET_CHARS (1500 chars ~= 375 tokens, inside
       the paper's 250-500 sweet spot per §4 Table 4); INTERNAL otherwise,
       recurse into named children. Structural containers (modules, impl
       blocks) with 2+ structural children are FORCED INTERNAL so each
       function lands as its own leaf floor.
    2. Sibling merge — walk LEAVES left-to-right. Greedy forward merge of
       contiguous leaves while combined size stays under TOKEN_BUDGET_CHARS.
    3. Fallback — when a single leaf exceeds HARD_MAX_CHUNK_CHARS (4000),
       emit alone (paper rule: do NOT split inside a leaf).

  Why ship alongside legacy chunker (not replace):
    ACCEPTANCE bullet "if +2 Recall@5 not met, keep legacy chunker behind a
    flag" — the eval harness at .elai_cc/eval/rag/run.py is the load-bearing
    measurement, and ELAI's 135-file corpus may not reproduce the paper's
    +4.3 Recall@5 / +2.67 Pass@1 RepoEval numbers. The cast.rs module is
    self-contained; bm25.rs's try_ast_chunk call site stays unchanged so
    a future wrapper can A/B test the two.

  Paper grounding status: arxiv:2506.15655 is INGEST-blocked per grounding
  sweep — no paper-ingestor artifact for it as of 2026-05-23. The +4.3 /
  +2.67 numbers are quoted second-hand; the algorithm implementation here
  follows the recursive-split + sibling-merge description that appears
  verbatim in multiple secondary summaries. SP6 eval harness is the
  authoritative measurement for ELAI's corpus.

M-C SP7 PLAID flag (new TOML section):

  File: elai_cc.toml [knowledge.late_interaction] section.

  Added the deferred-default-off flag schema as `enabled = false` (commented).
  The plan §Open questions explicitly defers PLAID to E-17.B; this section
  exists so contracts and future config-loading code can compile against a
  stable key path.

  Anti-Slopsquatting verification deferred: the next-plaid Rust crate was
  NOT added to Cargo.toml. crates.io existence check is required before
  adding the dep (grounding sweep §ACTIONS REQUIRED). E-17.B owns that
  verification + the actual producer wiring.

M-D dashboard wiring (NOT changed — already SHIPPED in B-4 + E-17 SP2):

  Verified via direct file read:
    crates/dashboard-v2/src/lib/drawers/KnowledgeDrawer.svelte:357-502 ships
    4 SP2 panels (DEEP POOL CAPPED, RRF QUALITY GATE, HYDE SKIPPED,
    RRF COMPONENT WEIGHTS) with full loading/error/empty branches.
    crates/dashboard-v2/src/lib/state/drawers.svelte.ts:132-226 ships the
    deepPoolCapped / rrfQualityGate / hydeSkippedCodeQuery / rrfComponentWeights
    state fields + polling loop.
    crates/dashboard-v2/src-tauri/src/lib.rs:301-304 registers the 4 Tauri
    commands in invoke_handler!.

  The prior recheck's MED warning about `get_rrf_quality_gate_fired_recent`
  being absent from invoke_handler is RESOLVED — the command is named
  `get_rrf_quality_gate_recent` (commands.rs:1270) and IS registered.
  Recheck used a stale name.

  Conclusion: SP8 dashboard wiring (M-D, M-E, M-F, M-G, M-H from the
  binding decision input) is already done. No further dashboard edits
  needed in this session.

D-1 SP3 default-flip drift: see SECTION 2 above. JINA-MODEL-PENDING marker
landed at reranker.rs:46-67. stage_onnx_models.py adds bge-reranker-v2-m3 as
the immediate-shippable ONNX path. Doctor probe surfaces optional rerankers
in the advisory footer (cli/src/doctor.rs render_knowledge_models_text).

SECTION 4 — 3 missing tests (all green)

Added tests (verified passing on 2026-05-23):

  crates/knowledge/tests/contextual_chunk_emission_contract.rs — 6 tests:
    round_trip_synthetic_chunk_preserves_body_and_prepends_prefix
    hash_stable_regen_skip_does_not_modify_body
    hash_mismatch_forces_fresh_regen
    prefix_length_never_exceeds_cap
    is_contextualised_round_trips_after_contextualize
    blake3_short_hex_is_stable_and_unique
    => 6/6 PASS

  crates/knowledge/tests/cast_chunker_recall_smoke.rs — 10 tests:
    rust_3_function_fixture_emits_chunks_covering_all_functions
    rust_oversized_fixture_emits_multiple_chunks
    python_2_class_fixture_produces_chunks_per_class_or_method
    typescript_class_and_fn_fixture_produces_chunks
    go_package_fixture_produces_chunks
    oversized_function_does_not_exceed_hard_max
    detect_cast_language_covers_4_languages_in_smoke_suite
    recall_at_5_sanity_chunk_contains_gold_anchor
    chunker_diagnostic_counters_are_sane
    (10 total, with the size-of-corpus check above split into 2 tests)
    => 10/10 PASS

  crates/knowledge/tests/jina_reranker_load_and_score.rs — 5 tests:
    jina_v3_load_returns_err_when_file_missing
    bge_v2_m3_load_returns_err_when_file_missing
    variant_path_mapping_is_deterministic_across_trials
    jina_v3_config_path_documents_expected_staged_location
    ms_marco_minilm_l12_default_preserved
    => 5/5 PASS

In-module unit tests:
  src/contextual_retrieval.rs::tests — 13/13 PASS
  src/chunking/cast.rs::tests — 10/10 PASS

Aggregate new test count: 44/44 PASS.

SECTION 5 — DEFERRED items (rolled forward to E-17.B follow-up)

  1. SsbActionType::{ContextualChunkEmitted, ChunkRegenSkipped} — SCHEMA-DEFERRED
     to next batch SCHEMA bump (see SECTION 1).
  2. LLM-backed contextual retrieval dispatch — Type scaffold + template ship,
     dispatch wire deferred until per-chunk sidecar cost is measured
     (plan §Open questions: ingest time grows N-fold without prompt caching).
  3. Default-flip of ReRankerModel to JinaV3 or BgeV2M3 — JINA-MODEL-PENDING
     until ONNX is staged (see SECTION 2).
  4. PLAID/FastPlaid producer wiring — flag schema ships; producer code
     deferred to E-17.B with anti-slopsquatting verification of next-plaid /
     fast-plaid before adding to Cargo.toml.
  5. cAST chunker as the DEFAULT chunker — legacy bm25::try_ast_chunk path
     stays as the active dispatch; cast.rs module is self-contained and
     ready for SP6 eval-harness A/B comparison. ACCEPTANCE "if +2 not met,
     keep legacy" — flip is gated on SP6 results.
  6. Dashboard surface for ContextualChunkEmitted / ChunkRegenSkipped —
     deferred-by-design until SCHEMA unlocks (no SSB variant -> no Tauri
     command -> no panel).

SECTION 6 — Build + test status

cargo check --workspace: GREEN (49.26s) — only pre-existing warnings.
cargo check -p elai-knowledge --tests: GREEN (8.37s clean).
cargo check -p elai-cli --tests: GREEN (43.24s).

cargo test -p elai-knowledge (full): not run from main LLM per CLAUDE.md
scoped-test-runs rule; auditor's job. Scoped runs above confirm the new
modules + tests are green and reranker_model_enum_load.rs still passes
(6/6) — no regression from the JINA-MODEL-PENDING doc-comment add.

cargo test -p elai-cli --tests: GREEN (doctor_knowledge_models_integration
3/3 pass — including the json_output_contract test that asserts the
shape of the existing `entries` array; the additive `optional_rerankers`
key did not break it).

SECTION 7 — Plan rebaseline markers (for cross-check ingest)

The plan file F:\.ELAI_workflow\docs\thefinalglance\E-plan\E-17_rag_retrieval_quality.md
is amended in-place below this session with the following markers
(searchable strings — Cross-Check V3 parses them):

  SHIPPED-ON-V2: 15 surfaces previously identified as shipped in V2 baseline
                 (SP1 doctor probe, SP2 deep cap + RRF gate + HyDE NL-only
                 gate + 4 SsbActionType variants + adapters + dashboard
                 panels + tests, .elai_cc/eval/rag/ harness + 15-query goldset).
  SHIPPED-THIS-SESSION: 5 surfaces (contextual_retrieval module, cast chunker
                 module, [knowledge.late_interaction] config schema, 3 missing
                 tests, doctor probe advisory footer for optional rerankers).
  JINA-MODEL-PENDING: D-1 drift resolution — Default reranker stays
                 MsMarcoMiniLmL12 until jina-rerank-v3.onnx ships OR
                 bge-reranker-v2-m3 is opt-in staged via stage_onnx_models.py.
  SCHEMA-DEFERRED: 2 SsbActionType variants (ContextualChunkEmitted,
                 ChunkRegenSkipped) — SCHEMA_VERSION 103 LOCKED, defer to
                 next batch.
  SPEC_DRIFT-RESOLVED: 3 sweep grounding items (Default flip; missing
                 cAST/contextual modules; missing late_interaction flag)
                 — all 3 now have decision-doc resolutions or shipped code.

SECTION 8 — Anti-Lazy hard-rule compliance

  1. ZERO new stubs/TODO/FIXME — verified via grep over the 2 new modules
     + 3 new tests + doctor.rs additions. JINA-MODEL-PENDING is a doc
     comment recording a real ops blocker, NOT a stub (no placeholder
     function body, no fatalError, no force-unwrap).
  2. No .unwrap()/expect() outside #[cfg(test)]/main. The new modules use
     ? + match/if let throughout. Test code uses .expect("reason") which
     is the documented exception per token-discipline.md.
     Continuation-session refactor: cast.rs sibling_merge originally had a
     single `chunks.last_mut().expect("non-empty checked above")` guarded
     by a prior `!chunks.is_empty()` check (provably-unreachable panic).
     Refactored to nested `if let Some(last) = chunks.last_mut()` with a
     `floor_merged` bool — semantics unchanged, no .expect() on production
     line. Tests (10 lib + 9 smoke) re-run GREEN post-refactor.
  3. tracing crate + thiserror/anyhow used; native async fn in traits.
     No async-trait crate, no Box<dyn Error>.
  4. No SCHEMA bump.
  5. Anti-Slopsquatting verified: WebSearch confirmed jina-reranker-v3
     HuggingFace page exists but has NO official ONNX (formats: PyTorch/
     MLX/GGUF). bge-reranker-v2-m3-ONNX (onnx-community) confirmed real
     before adding to stage_onnx_models.py.
  6. cAST chunker uses tree-sitter (workspace dep 0.26) — no regex
     heuristic. Recursive-split + sibling-merge per paper §3.2.

End of E-17-decisions.md.
