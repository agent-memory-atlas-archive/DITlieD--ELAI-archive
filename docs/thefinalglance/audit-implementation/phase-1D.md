Phase 1D audit — PageRank repo map
===================================

Auditor: hostile implementation audit
Date: 2026-04-24
Sources consulted:
  - F:\.ELAI_workflow\docs\thefinalglance\summary\phase-1-summary.md (L606-712)
  - F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\pagerank\mod.rs (913 lines)
  - F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\pipeline.rs (L380-445)
  - F:\.ELAI_workflow\.elai_cc\crates\cli\src\main.rs (L3752-3762, L5437-5446)
  - F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\indexing_service.rs (L15-18, L121-165)
  - F:\.ELAI_workflow\.elai_cc\metrics\phase_1D.toml
  - F:\.ELAI_workflow\.claude\tools\elai.map (grep evidence)
  - Filesystem verification: `.elai_cc\.elai_cc\indexes\dkg.db` (8.2 MB, populated) vs `.elai_cc\.elai\` (no dkg.db)

--------------------------------------------------------------------------------
Per-sub-step verdict
--------------------------------------------------------------------------------

1D.1 SKELETON — PASS
  Evidence:
    - `pub fn compute_repo_map(snapshot, task, active_files, config, token_estimator) -> RepoMap`
      at mod.rs:351.
    - `PageRankConfig { damping_factor, max_iterations, tolerance, density_threshold,
      max_map_tokens }` with Default (0.85/100/1e-6/0.3/2048) at mod.rs:89-107. Matches
      spec defaults exactly.
    - `RepoMap { ranked_symbols, token_count, density, used_pagerank, fit_iterations }`
      at mod.rs:118-125. Matches spec.
    - `RankedSymbol { node_id, symbol_name, file_path, rank, signature }` at mod.rs:109-116.
    - `DkgSnapshot { nodes: Vec<PageRankNodeInfo>, edges: Vec<(usize,usize,f64)> }` at
      mod.rs:21-25.
    - `DkgSnapshot::from_engine` at mod.rs:28. Note: spec says `DkgNodeInfo` — impl
      renamed to `PageRankNodeInfo`. Fields match exactly (node_id, symbol_name,
      file_path, signature, kind, ref_count). Rename is non-breaking but deviates from
      spec wording — DEVIATION (minor).
    - proptest = "1" in Cargo.toml dev-dependencies (L43).
    - Self-edges for orphans: from_engine adds `(i, i, 0.1)` for nodes with zero inbound
      AND zero outbound (mod.rs:77-82). Spec said "zero incoming edges" — impl uses
      stricter "no in AND no out" criterion. DEVIATION (minor — stricter than spec; a
      node with only inbound still gets no self-edge). T16 test explicitly bypasses
      from_engine and adds a self-edge manually to pass.

1D.2 VERIFY — UNVERIFIED-BY-AUDITOR
  Spec requires workspace cargo check 0. No cargo invocations run in this audit (main
  model does not run tests). pipeline.rs compiles against compute_repo_map per map
  evidence (SCIP_REF at pipeline.rs:14). Box<dyn Error> leak at mod.rs:28 crosses an
  API boundary but is not `Send+Sync` — this can still compile but makes callers that
  require `Send+Sync` error types (most orchestration layers) use `?` with coercion.
  Downstream pipeline.rs:400-403 catches with `tracing::debug!("DkgSnapshot::from_engine
  failed: {e}");` + `e.to_string()` so compilation is not blocked.

1D.3 TEST — PASS
  Evidence (mod.rs:493-912):
    - 21 `#[test]` attributes (T1..T18 + A1..A3). Verified by grep `^    #\[test\]` = 21.
    - 3 proptest cases inside `proptest!` block (P1 ranks_are_finite_and_non_negative,
      P2 ranks_sum_to_one, P3 token_count_within_budget) at mod.rs:836-910.
    - Helper `fn make_snapshot(node_count, edges)` present at mod.rs:497.
    - Helper `fn dummy_token_estimator(s) -> usize = s.len()/4 + 1` at mod.rs:518.
    - Spec required >=21 tests. Actual: 21 #[test] + 3 proptest = 24 test functions. MEETS
      threshold.

1D.4 IMPLEMENT — PASS with caveats
  Confirmed implementation details:
    (1) Adjacency build + sqrt(ref_count) scaling:
        mod.rs:162-201, scale = `(ref_count as f64).sqrt()` when ref_count>0. OK.
    (2) Edge-weight boost *= 50 when either endpoint file is in active_files:
        mod.rs:181-185. OK, but note: boost is applied to BOTH directions (from_active
        OR to_active) rather than strictly "edges touching active files". This matches
        spec intent.
    (3) Row normalization to stochastic matrix (mod.rs:191-198) — standard; not in
        spec but necessary for PageRank.
    (4) Personalization 10x + normalize sum=1.0: mod.rs:203-223. OK. If all
        `task_mentioned` are out of range (len==0 or indices>=n), the result reduces
        to uniform 1/N — matches spec fallback.
    (5) Density = edge_count / (n*(n-1)) at mod.rs:371-375. Density guard at
        mod.rs:378: `density > config.density_threshold` -> used_pagerank=false. Default
        threshold 0.3. OK. Note: guard uses strict `>`, so density == 0.3 will NOT
        trigger. Spec says ">0.3" — consistent.
    (6) Power iteration with dangling redistribution: mod.rs:234-288.
        `new_rank[i] = damping * (rank_contribution + dangling_sum * personalization[i])
         + (1-damping) * personalization[i]`.
        This is the correct NetworkX-style formulation. NaN/Inf clamp to 0.0 at
        mod.rs:268-270. OK.
    (7) Special files list has 13 entries (mod.rs:141-155) — matches spec exactly:
        Cargo.toml, Cargo.lock, README.md, README, LICENSE, .gitignore, Dockerfile,
        docker-compose.yml, build.rs, rust-toolchain.toml, clippy.toml, rustfmt.toml,
        .cargo/config.toml.
    (8) is_special_file uses `ends_with` substring match on the full path — OK
        for Cargo.toml/README.md but could false-positive on files like
        "fakeCargo.toml" or "mydir/README.md". Spec does not require basename-only
        matching. Minor concern.
    (9) Binary search fit (mod.rs:290-345):
        - Zero-budget short-circuit: returns (vec![], 0). OK (T13).
        - Initial guess `max_tokens/25` per aider adoption #4. OK.
        - Max 10 iterations. OK.
        - On overshoot: shrinks `hi` and halves. On undershoot: grows toward `hi`.
        - Truncation applied in estimator call via `truncate_signature(&s.signature,
          100)` — OK.
        ISSUE: After binary_search_fit, compute_repo_map re-computes token_count using
        `token_estimator(&s.signature)` WITHOUT re-applying truncation (mod.rs:393-395,
        475-478). But `s.signature` was already truncated when building ranked_symbols
        (mod.rs:442, 470). So the final token_count is truncated-equivalent. OK.
        SUBTLE ISSUE: the density-guard branch (mod.rs:379-404) truncates signatures
        in ranked_symbols but then calls estimator on `s.signature` (already
        truncated) — consistent. OK.
    (10) Line truncation: MAX_SIGNATURE_CHARS=100, `truncate_signature` slices first
         100 bytes (mod.rs:131-139). WARNING: `sig[..max_chars]` slices by byte offset,
         not char boundary. A multi-byte UTF-8 char at offset 99..100 will panic. Rust
         signatures are ASCII in practice so this has not surfaced, but it is a latent
         bug. Fix: use `sig.char_indices().nth(max_chars).map_or(sig.len(), |(i,_)| i)`
         or `sig.chars().take(max_chars).collect()`. FLAGGED for patch.

1D.5 MEASURE — PRESENT but sparse
  .elai_cc/metrics/phase_1D.toml exists (12 lines, 7 keys):
    test_count=24, compile_time_seconds=9.45, pagerank_loc=913,
    ranking_quality=true, token_budget_accuracy=15.0 (upper bound from P3, not
    measured), density_guard_correct=true, binary_search_max_iterations=10 (upper
    bound from T10, not measured).
  Spec ACCEPTANCE was "file with all 7 metrics" — satisfied literally. However,
  `token_budget_accuracy` and `binary_search_max_iterations` are upper-bound
  placeholders, not real measurements. ACCEPTABLE but weak.

--------------------------------------------------------------------------------
Box<dyn Error> count
--------------------------------------------------------------------------------

grep -c "Box<dyn" crates\knowledge\src\pagerank\mod.rs = 1.
Location: mod.rs:28 `DkgSnapshot::from_engine -> Result<DkgSnapshot, Box<dyn
std::error::Error>>`.

This is the same debt flagged in Phase 1A audit. Recommend migrating to
`KnowledgeError` (already used by catrag.rs, per map line 16337). Single remaining
leak in this module.

--------------------------------------------------------------------------------
CRITICAL wiring check — dual DB path bug
--------------------------------------------------------------------------------

VERDICT: BUG CONFIRMED. PageRank reads an empty/absent DB on real runs.

Writer side (IndexingService):
  - `indexing_service.rs:15`  `INDEXES_DIR_NAME = "indexes"`
  - `indexing_service.rs:18`  `ELAI_DATA_DIR_NAME = ".elai_cc"`
  - `indexing_service.rs:131` `dkg_db_path = indexes_dir.join(DKG_DB_FILENAME)`
  - `cli/main.rs:3834`        `indexes_dir = project_root.join(ELAI_DATA_DIR_NAME)
                               .join(INDEXES_DIR_NAME)` -> resolves to
                               `{root}/.elai_cc/indexes/dkg.db`.
  Filesystem evidence: `F:\.ELAI_workflow\.elai_cc\.elai_cc\indexes\dkg.db` is 8.2 MB
  and dated 2026-04-19 (real index artifact).

Reader side (PageRank + telescope):
  - `cli/main.rs:3752`  `let dkg_db_path = project_root.join(".elai").join("dkg.db");`
                         followed by `if dkg_db_path.exists() { ... pipeline.dkg_engine
                         = Some(engine); }`.
  - `cli/main.rs:5437`  same pattern for `build_telescope_block()` used by A-36
                         architecture inference.
  Filesystem evidence: `F:\.ELAI_workflow\.elai_cc\.elai` contains ONLY
  `sessions/`. NO `dkg.db` exists at that path.

Consequence:
  - `dkg_db_path.exists()` is FALSE at the CLI path, so `pipeline.dkg_engine` stays
    `None`.
  - In pipeline.rs:406-409, the `None` branch yields `DkgSnapshot { nodes: empty,
    edges: empty }` and NO pagerank results are produced.
  - `compute_repo_map` is never even called in the production CLI path, because the
    empty snapshot means there is nothing to rank. PageRank is silently no-op at
    runtime.
  - dashboard/commands.rs:125 DOCSTRING correctly states `.elai_cc/indexes/dkg.db`
    — the comment is right, the CLI code is wrong.

Note: the dashboard consumer (dashboard/src/lib.rs:826-829) CORRECTLY uses
`ELAI_DATA_DIR_NAME / INDEXES_DIR_NAME / "dkg.db"`. Only the CLI consumers
(main.rs:3752 and main.rs:5437) use the broken `.elai` literal. This is a
two-line fix but blocks the entire PageRank pipeline end-to-end.

--------------------------------------------------------------------------------
Wiring check — compute_repo_map call graph
--------------------------------------------------------------------------------

compute_repo_map is WIRED (but disabled by the DB-path bug above).

In-project callers (from elai.map + direct read):
  - `crates/knowledge/src/pipeline.rs:387` — called inside QueryPipeline::query /
    query_with_tier, stage "pagerank", under `if use_vector && dkg_engine.is_some()`.
  - `crates/knowledge/src/pagerank/mod.rs:527-830` — 21 test callsites.

QueryPipeline reaches the CLI through `elai_knowledge::pipeline::QueryPipeline` at
`cli/main.rs:69` + instantiation in the command path. `pipeline.dkg_engine` is
assigned at `cli/main.rs:3756` behind the broken `.elai/dkg.db` existence check.

Therefore:
  Pure code wiring: WIRED (pipeline -> compute_repo_map exists).
  Runtime wiring:   BROKEN (dkg_engine never gets set because CLI looks at wrong
                    path; pagerank stage returns empty results silently).

Status: WIRED-BUT-DEAD. The feature flag is effectively "off" in every production
invocation today, even though all the implementation code is present and tested.

--------------------------------------------------------------------------------
Divergences vs aider/repomap.py — checked
--------------------------------------------------------------------------------

Spec claims 3 deliberate divergences + 5 adopted improvements.

Adopted (spec AIDER ADOPTED 5):
  1. Dangling node redistribution via personalization — VERIFIED (mod.rs:253,
     265-266, T15).
  2. Self-edges weight=0.1 for orphans — VERIFIED in from_engine (mod.rs:80) and T16.
     Note impl is stricter than spec wording (see 1D.1).
  3. sqrt(num_refs) edge scaling — VERIFIED (mod.rs:174-178, T17).
  4. Binary search initial guess max_map_tokens/25 — VERIFIED (mod.rs:299, T10).
  5. Line truncation 100 chars — VERIFIED (mod.rs:131, MAX_SIGNATURE_CHARS=100, T18).

Divergences (spec 3):
  1. Symbol-level ranking (not file-level) — VERIFIED. RankedSymbol is per-node.
  2. Density guard at 0.3 — VERIFIED (mod.rs:378, T4, A1).
  3. Insertion-order fallback — VERIFIED. mod.rs:379-389 yields nodes in snapshot
     insertion order with uniform rank = 1/n when guard fires. No BM25 dependency in
     signature. Matches spec T4 acceptance.

--------------------------------------------------------------------------------
Summary
--------------------------------------------------------------------------------

Overall verdict: MOSTLY IMPLEMENTED, WIRED-BUT-DEAD.

The pagerank module itself is complete, correctly implemented, and well-tested
(21 unit + 3 proptest = 24 tests; all spec deliverables 1D.1-1D.5 present;
aider divergences and adopted improvements all verified). The module is
plumbed into QueryPipeline via `pipeline.rs:387`.

However, the Phase 1A audit finding is confirmed and more severe for 1D: the
CLI opens the DKG engine from `{root}/.elai/dkg.db` (cli/main.rs:3752, 5437),
but IndexingService writes to `{root}/.elai_cc/indexes/dkg.db`. The reader
path does not exist on disk; the writer path holds a populated 8.2 MB DB.
Net effect: `pipeline.dkg_engine` is always `None` in production, the
`pagerank` stage always returns empty, and `compute_repo_map` — though
correctly implemented — is dead code at runtime. Telescope A-36 architecture
inference (cli/main.rs:5437) suffers the same dead-read bug.

Outstanding debt:
  1. Box<dyn std::error::Error> at mod.rs:28 — 1 instance, inherited from 1A.
  2. Dual-path DB bug at cli/main.rs:3752 and 5437 — blocks all PageRank at
     runtime. Fix: replace `.join(".elai").join("dkg.db")` with `.join(".elai_cc")
     .join("indexes").join("dkg.db")` using the existing `ELAI_DATA_DIR_NAME` /
     `INDEXES_DIR_NAME` constants.
  3. Latent UTF-8 panic in truncate_signature (byte slice, not char slice) —
     mod.rs:137. Won't trigger on ASCII Rust signatures but is a correctness bug.
  4. Phase_1D.toml metrics `token_budget_accuracy` and
     `binary_search_max_iterations` are upper-bound placeholders, not measured.
  5. `PageRankNodeInfo` renamed from spec's `DkgNodeInfo` — cosmetic spec
     divergence; update spec or rename struct.

Test counts:
  - Unit + adversarial tests: 21
  - Proptest cases:            3
  - Total test functions:      24
  - Spec threshold (>=21):     MET

Reported DB path read by pagerank consumers: `{project_root}/.elai/dkg.db`
(WRONG — actual DB is at `{project_root}/.elai_cc/indexes/dkg.db`).
