B-16A Anti-Stale Enforcement — execution decisions

Phase: B-16A (subset of B-16, deferred to B-16B/C/D: .unwrap audit, log->tracing,
magic numbers, narrowing as-casts).
Session: s-28972.
Date: 2026-04-26.

DECISION-A — ort version pin

Decision: keep `ort = "=2.0.0-rc.12"` in crates/knowledge/Cargo.toml.
Rationale:
  - WebSearch verification (2026-04-26) confirmed ort 2.0.0 STABLE has not
    been published on crates.io. Latest is 2.0.0-rc.12 (released
    2026-03-05 per pykeio/ort GitHub releases). Stable release "planned but
    not yet published" per pyke.io docs.
  - The Anti-Stale-Training rule "ort stable (never -rc)" is currently
    UNSATISFIABLE. Forcing ">= 2.0.0" or "2" would break the build the
    moment a stable release ships with API changes (ort 2.x has had
    multiple breaking rc-to-rc changes already).
  - 3 production files use the rc.12 surface identically: reranker.rs L4-6,
    splade.rs L12-14, vector/ort_session.rs L1-2. A single coordinated bump
    is cleaner than incremental migration.
Waiver: this file constitutes the explicit waiver. The lint script
  `scripts/check_anti_stale.sh` checks for this waiver file's presence
  before failing on `^ort\s*=\s*"[^"]*-rc` matches.
Bump trigger (TODO post-ort-2.0.0-stable):
  1. Run `cargo search ort --limit 1` and verify the latest published
     version is `2.0.0` (no `-rc`/`-pre`/`-beta` suffix).
  2. Read pykeio/ort's CHANGELOG between rc.12 and stable for breaking
     changes — pay attention to `Session::run` signature, `Tensor`
     constructors, and `inputs!` macro semantics.
  3. Bump Cargo.toml to `ort = "2"`. Run `cargo check -p elai-knowledge`.
  4. Update reranker.rs / splade.rs / vector/ort_session.rs in lockstep.
  5. Delete this DECISION-A waiver section after the bump lands.

DECISION-B — reranker.rs:465 .expect fix (verify -> fix)

Decision: replace `.expect("semaphore closed")` with a match-Ok-Err
pattern that returns gracefully via `tracing::warn!`.
Rationale:
  - Context-package validated that B-4's claim of fixing this site was
    incorrect — the .expect was still in place at L465 as of B-16A entry.
  - The current code: `let _permit = semaphore.acquire().await.expect("semaphore closed");`
    panics whenever `Semaphore::close()` is called (legitimate during test
    teardown or supervisor shutdown).
  - Tokio's canonical pattern for shared semaphore use is to treat
    `AcquireError` as "shutdown signaled" and exit the task cleanly.
Pattern landed at reranker.rs:465-475:
    let _permit = match semaphore.acquire().await {
        Ok(p) => p,
        Err(_) => {
            tracing::warn!("reranker semaphore closed; exiting batch worker");
            return;
        }
    };
Test coverage: tests/error_propagation.rs::semaphore_closed_returns_error_not_panic
exercises the contract directly via tokio::sync::Semaphore::close().
Other reranker .expect/.unwrap occurrences (L768, L770, L809, L819, L831,
L842, L887, L909, L1005, L1020, L1022, L1037, L1045, L1065, L1073-1075,
L1078, L1081) are all inside #[cfg(test)] modules — acceptable per
CLAUDE.md and not touched in B-16A.

DECISION-C — error enum design (single KnowledgeError vs per-module enums)

Decision: extend the existing single `KnowledgeError` thiserror enum at
crates/knowledge/src/error.rs.
Rationale:
  - dkg/mod.rs, arag.rs, catrag.rs, freshness.rs, fix_templates.rs already
    return Result<_, KnowledgeError> per the A-35 migration. Public
    re-export at lib.rs:265 already exists.
  - Per-module Bm25Error/VectorError/PagerankError would force every
    caller (cli/main.rs, orchestrator/indexing_service.rs,
    pipeline.rs, contracts) to handle three new error kinds — pure churn.
  - Spec wording in B-16 allows either approach; less-churn wins.
Variants ADDED (4 new):
  - `Tantivy(#[from] tantivy::TantivyError)` for bm25.rs writer/searcher
    errors.
  - `Json(#[from] serde_json::Error)` for vector/indexer.rs meta serde.
  - `SchemaMismatch { path: PathBuf, field: String }` for the
    bm25.rs:178 schema-validation case (was a stringly-typed format!).
  - `Index(String)` for BruteForceIndex's IndexError, bridged via a
    manual `From<crate::vector::hnsw_index::IndexError>` impl that
    avoids leaking IndexError onto the KnowledgeError public surface.
Variants kept unchanged: Io, Sqlite, QueryValidation, Vector, Bm25, Graph,
Pipeline.
Bridge impls added (manual From, not #[from]):
  - From<IndexError>           -> Index(String)
  - From<QueryParserError>     -> Tantivy(InvalidArgument(String))

DECISION-D — anti_stale lint mode (REPORT vs FAIL)

Decision: STRICT mode for `Box<dyn .*Error`, `use log::`, and
`ort = "...-rc..."` (without waiver doc); REPORT-ONLY for `.unwrap()`
and narrowing as-casts.
Rationale:
  - B-16A's scope explicitly defers .unwrap audit (Task 8 -> B-16B),
    log->tracing migration (Task 10 -> B-16B/C), and as-casts ->
    try_into (Task 12 -> B-16D).
  - Going FAIL on those today would block the build immediately —
    counterproductive given the deferred scope.
  - STRICT for the three patterns that B-16A actually fixes (Box<dyn
    Error> via thiserror, log::->tracing convention not yet violated in
    knowledge crate, and the rc.12 waiver).
Gradual-escalation plan (recorded for future phases):
  - B-16B: tighten .unwrap() to STRICT (exempt main.rs/test/const).
  - B-16C: tighten use log:: to STRICT across all crates after final
    audit confirms no log macro consumers remain.
  - B-16D: tighten narrowing as-cast pattern to STRICT (exempt widening).

DECISION-E — blake3 spec amendment scope

Decision: amend ONLY the narrative claims about current implementation
state. PRESERVE all lines that name "blake2" as the deprecated_pattern
fixture itself.
Rationale:
  - Spec files at docs/specs/guide/ are gitignored per CLAUDE.md project
    boundary rule (only .elai_cc/ is tracked). Amendment lands locally.
  - The anti-staleness-enforcement.txt spec encodes blake2 as a seed for
    its detection rule pack and few-shot training fixtures (L111, L150,
    L177, L305, L737-739). Removing those would destroy the test data
    the spec is teaching the agent to generate.
Lines amended (narrative-only):
  - phase-3C.txt L399 — "Hash = blake2s of normalized..." -> blake3
  - phase-3C.txt L442 — describes obsolete state of project hashing
  - anti-staleness-enforcement.txt L12 — model favors blake2 narrative
  - anti-staleness-enforcement.txt L14 — same paragraph continuation
  - anti-staleness-enforcement.txt L25 — 9 critical findings list
  - anti-staleness-enforcement.txt L70 — "blake2 used hundreds of times"
    obsolete claim
  - anti-staleness-enforcement.txt L347 — negative-lookahead example
    (rephrase to use a different stale pattern as the example)
Lines preserved unchanged (deprecated_pattern fixtures): L111, L150,
L177, L305, L737-739.

DECISION-F — pipeline.rs caller migration

Decision: pipeline.rs already returns Result<_, KnowledgeError> on its
public methods (per A-35 work). Internal calls to BM25/Vector/PageRank
that previously consumed Box<dyn Error> via `.map_err(|e| ...e.to_string())`
now work uniformly because KnowledgeError implements Display+Error and the
enclosing match arms in pipeline.rs already extract `e.to_string()`.
Verification: `cargo check -p elai-knowledge` and `cargo check --workspace`
both pass clean (only 2 pre-existing warnings unrelated to B-16A).
No further pipeline.rs edits required for the migration to compile.

14 Box<dyn Error> production sites migrated

bm25.rs:155 Bm25Index::new            -> Result<Self, KnowledgeError>
bm25.rs:217 index_file                -> Result<usize, KnowledgeError>
bm25.rs:243 index_directory           -> Result<IndexStats, KnowledgeError>
bm25.rs:262 query                     -> Result<Vec<BM25Hit>, KnowledgeError>
bm25.rs:314 delete_file               -> Result<usize, KnowledgeError>
bm25.rs:329 update_file               -> Result<usize, KnowledgeError>
bm25.rs:362 chunk_file (private)      -> Result<Vec<BM25Chunk>, KnowledgeError>
bm25.rs:403 walk_dir (private)        -> Result<(), KnowledgeError>
vector/indexer.rs:57 VectorIndexer::load        -> Result<Self, KnowledgeError>
vector/indexer.rs:71 VectorIndexer::save        -> Result<(), KnowledgeError>
vector/indexer.rs:89 index_file                  -> Result<IndexStats, KnowledgeError>
vector/indexer.rs:194 search                     -> Result<Vec<(ChunkMeta,f32)>, KnowledgeError>
vector/indexer.rs:218 chunk_file (free fn)       -> Result<Vec<BM25Chunk>, KnowledgeError>
pagerank/mod.rs:28 DkgSnapshot::from_engine     -> Result<DkgSnapshot, KnowledgeError>

ort waiver — STATUS

Active. Trigger to re-evaluate: see DECISION-A bump trigger steps.
Last verified: 2026-04-26 (WebSearch confirmed rc.12 still latest).
Re-verified: 2026-05-09 — WebSearch (crates.io/crates/ort, docs.rs/crate/ort/latest)
  confirmed ort 2.0.0 stable STILL not published. Latest is 2.0.0-rc.12 (2026-03-05).
  Waiver remains active. No bump action taken.
