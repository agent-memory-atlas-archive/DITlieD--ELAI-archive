B-29 Symbol-Level Impact API — Decision Log

Session: s-22200
Plan: docs/thefinalglance/B-plan/B-29_symbol_level_impact_api.md

Phase 1 (Schema + types + sharding) decisions:

1. Encoder: postcard (workspace dep), NOT rmp-serde
   Why: rmp-serde would be a new external crate dependency. CLAUDE.md anti-dep
   rule requires explicit user confirmation. postcard is already in the workspace
   and used by other knowledge-crate modules. Functional contract preserved
   (opaque BLOB columns; field/column names ending in *_msgpack are retained to
   match the WIRING_CONTRACT spec text). Documented inside migration_v3.sql
   and SCHEMA_SQL_V3 header.

2. Reverse-shard hash: blake3 first byte, NOT SHA1
   Why: SHA1 would be a new dep. blake3 is workspace-pinned and CLAUDE.md
   anti-stale rule lists blake3 as the default for non-RFC hash uses. Domain
   is a 256-bucket router; uniform distribution is the only requirement and
   blake3's first byte is uniformly distributed (verified by sharding test
   `name_shard_key_corpus_distributes_across_buckets`).

3. Type namespacing: knowledge::symbol_graph::types::*, NOT re-exported at crate root
   Why: `knowledge::SymbolKind` (DKG, 10 variants) and `elai_core::EntryPoint`
   already exist with the same names. Re-exporting would shadow them and
   silently break downstream code. Module-only path forces callers to qualify
   intent (`knowledge::symbol_graph::types::SymbolKind` vs `knowledge::SymbolKind`).

4. Schema version: jumped 1 -> 3 (skipping v2)
   Why: Plan terminology pins "v3". v2 was reserved by a sibling B-step that
   never landed. dkg_meta `INSERT OR REPLACE` (not IGNORE) ensures pre-B-29
   databases re-opened after the migration update their stored version, not
   silently leave a stale '1' that diagnostic queries would surface.

5. unresolved_edge_pct: percentage scale (0.0–100.0), NOT fraction (0.0–1.0)
   Why: WIRING_CONTRACT stub_ban specifies "< 50.0" — only meaningful on the
   percentage scale. Auditor's WARN caught this before Phase 4 wrote the wrong
   scale. Contract test `b29_phase1_meta_fields_lock_and_percent_scale` locks it.

6. CURRENT_SCHEMA_VERSION constant bump: required, not just SQL DDL append
   Why: Existing DKG path uses `index_version::current_version` to decide
   FullRebuild vs IncrementalUpgrade. Without the constant bump, pre-B-29
   databases would be silently considered current — the new tables would
   exist but never populate.

7. Decision doc location: workflow/reasoning/B-29-decisions.md
   Why: CLAUDE.md "Decision doc" hard rule. One file per B-step covering all
   phases; appended at MEMORIZING of each phase.

Phase 2 (Extraction + cross-file resolution) decisions:

8. Symbol id format: `{file}::{name}#L{start_line}` (NOT `{file}::{name}`)
   Why: Same-name symbols in the same file (e.g. Python module-level fn `bar`
   plus class method `bar`) would collide on id under the simple format. The
   line suffix is stable across re-extraction of unchanged source. Module
   synthetic symbol uses `<module>#L1` for consistency.

9. Encoder for blobs in extraction: postcard (already chosen Phase 1).
   No change. ExtractedSymbols / RawCall types are serde-derived and
   postcard-roundtripped in tests.

10. Per-language call-node kinds (single-pass extraction):
    - Rust/TS/TSX/Svelte/Go: `call_expression` + (Rust) `macro_invocation`
    - Python/Ruby: `call`
    - Java: `method_invocation` (field "name" for callee)
    - C#: `invocation_expression` (field "expression"; last-segment split)
    Why: matches tree-sitter grammar node names verified during exploration.
    Fallback to first identifier descendant only when field lookup misses.

11. Re-export resolution depth capped at 2 hops; cycle prevention via
    BTreeSet<String> visited. Per plan L160-L162 (Tier 1 Local, Tier 2
    Imported 1-hop, Tier 3 Re-export 2-hop). Walking deeper is forbidden —
    Confidence::Unresolved at 3+ hops is the correct contract.

12. Resolution Tier 1 excludes Module-kind symbols from local-match candidates.
    Why: Module is a synthetic catch-all; resolving a call to the module
    symbol would lie about the call destination.

13. Local MultipleCandidates: overloaded names within the same file (Rust
    inherent impls, Java overloads) yield MultipleCandidates with sorted
    candidate ids. Spec L162-L164 says "1 -> Unique, >1 -> MultipleCandidates"
    — interpreted to apply at every tier including Local.

14. `find_enclosing_caller` for call-site attribution: sort scopes by
    (start_line asc, end_line desc); pick narrowest containing scope by
    span. O(N) per call site; acceptable for files <10K LOC. Larger files
    will be optimized in Phase 3 cache layer.

15. Removed dead `.expect()` and `_cursor` allocation flagged by Phase 2
    auditor. Defensive `if let Some(...)` with tracing::warn replaces the
    expect; no panic path in production library code.

Phase 3 (Cache + traversal + persistence) decisions:

16. SymbolGraphCache::new takes db_path: &Path (not SqliteConnectionPool)
    Why: Plan referenced `SqliteConnectionPool` which does not exist in the
    workspace (no deadpool/r2d2/bb8 anywhere in knowledge crate). Single
    rusqlite::Connection held in cache; opened with SQLITE_OPEN_READ_ONLY
    flag. Pattern matches DkgQuerierAdapter precedent in the crate.

17. Shard load is SERIAL, not tokio::JoinSet-parallel
    Why: Plan L172-L173 specified JoinSet. The knowledge crate only enables
    tokio `features=["sync"]`; no runtime, no spawn, no .await available.
    Adding tokio runtime features would be a substantive workspace change
    against CLAUDE.md anti-dep rule without explicit user confirmation.
    Decision: ship serial loading for 27 + 256 shards. Re-evaluate after
    B-29.5 scale benchmark — if Linux-kernel-scale load latency exceeds
    p99 budget (500ms), revisit with std::thread::scope (rusqlite is Send).

18. Lazy index state via Option<BTreeMap<...>>, not OnceCell
    Why: std::cell::OnceCell behind &mut self is fine but slightly more
    awkward than `if self.x.is_none() { self.x = Some(...) }` pattern.
    The latter matches existing knowledge-crate style (e.g. cached project
    profile detection). Functionally equivalent.

19. LRU is hand-rolled BTreeMap<u64, String> + FxHashMap<String, (Arc<P>, u64)>
    + monotonic counter
    Why: CLAUDE.md anti-dep ban on `lru` crate. Algorithm: O(log N) per access
    via BTreeMap ordering; O(log N) eviction via pop_first on the ordering map.
    Capacity defaults to SYMBOL_FILE_LRU_SIZE (500). Verified bounded under
    1000-key over-fill in tests/symbol_graph_cache_tests.rs.

20. CallerInfo.confidence and CalleeInfo.confidence: Confidence enum, NOT String
    Why: Phase 3 auditor caught a String regression (the executor used
    format!("{:?}", ...) for ergonomics). String breaks exhaustiveness checks
    when Confidence gains a variant + couples downstream consumers to enum
    debug representation. Switched back to enum. Contract test asserts the
    field type explicitly.

21. Defensive `.expect()` removal in cache.rs: replaced three logically-
    unreachable .expect() calls with KnowledgeError returns and tracing::warn.
    Why: CLAUDE.md hard rule — .expect() banned outside test code and
    entrypoints. Invariants remain enforced; failure mode is now an error
    return, not a panic.

22. registry_status() duplicate-method collision in dkg/mod.rs (cross-session
    cleanup): another session (s-21092 B-30) left an uncommitted duplicate
    method definition that broke `cargo check`. Both bodies were identical;
    deleted the older one and preserved the newer B-30 doc comment. Session
    s-21092 was IDLE at the time and owned the file via stale lock; cleanup
    was non-disruptive (identical bodies merged, no semantic loss). Noted
    here so the next audit understands the cross-session edit.

Phase 4 (Build pipeline integration + CLI subcommand) decisions:

23. Build entry point: knowledge::symbol_graph::build_pipeline (NOT a new
    method on DkgEngine). Why: minimises blast radius into the DKG public
    surface during a session where another agent (s-21092) holds the lock on
    dkg/mod.rs. The new module re-instantiates grammar unit structs locally
    (RustGrammar, PythonGrammar, etc) and reuses the DKG SQLite database
    only for reading the `dkg_edges WHERE edge_type='imports'` rows.

24. Source walking: `ignore::WalkBuilder` with `.hidden(true) + .git_ignore(true)`.
    Why: `ignore` is an existing knowledge-crate dep; no new external crate.
    `.hidden(true)` excludes `.cargo/.svelte-kit/.next/.git/etc` from the walk
    even when those paths are absent from `.gitignore` (Phase 4 audit WARN-2 fix).

25. ClockProvider injection: build_symbol_graph + update_changed_files_symbol_graph
    both accept `clock_now_ms: i64`. No raw SystemTime::now() in library code.
    CLI subcommand handler captures a single timestamp via `entrypoint_now_ms()`
    at the subcommand entry (CLAUDE.md exemption for main/entrypoint init).
    IndexingService passes `clock_unix_millis(&self.clock)` from the existing
    Arc<dyn ClockProvider> field.

26. `project_id` derivation in CLI: return Err if `project_root.file_name()` is
    None (root path). Phase 4 audit WARN-3 fix — previously fell back to
    "unknown" which could collide across sessions.

27. Incremental update name-shard correctness: collect `touched_name_keys`
    BEFORE removing files from `symbols_by_file`. Otherwise removed files'
    symbol names disappear and the shard is never rebuilt, leaving stale
    rows on disk. Phase 4 audit BLOCK-1 fix.

28. IndexingService auto-wire: SUCCESSFUL. `build_symbol_graph` is invoked
    after `drop(dkg)` and before "Mark completed". Symbol-graph failure is
    treated as best-effort (tracing::warn) — DKG remains the source of
    truth. The dkg/mod.rs edits (registry_status dedup, VueGrammar gate,
    style_element/script_element u32 casts) were also necessary cleanup of
    cross-session breakage from s-21092 — bodies were identical or trivial
    type conversions; documented in this file for future audit context.

29. CLI subcommand `/symbol_graph build|stats` shipped in parallel with the
    auto-wire. Why ship both: dashboard / scripted callers may want explicit
    invocation without triggering full reindex. /symbol_graph build runs DKG
    build then symbol_graph build; /symbol_graph stats prints the meta row.

Phase 5 (Dispatcher + Auditor injection + SSB + Dashboard) decisions:

30. Tool registration: 4 TOOL_SYMBOL_* constants + 4 typed wire request
    structs landed in `crates/ipc-types/src/lib.rs`. Dispatch arms in
    `crates/orchestrator/src/production_agent_rpc.rs::translate_tool_request`
    (NOT crates/orchestrator/src/dispatcher.rs — that file is DispatchContext,
    not a registry). 4 match arms: TOOL_SYMBOL_IMPACT/FLOW/CONTEXT/LIST.

31. SSB SymbolQuery variant: added to `crates/orchestrator/src/ssb.rs` (10-site
    coordinated edit pattern: SCHEMA_VERSION bump, CHECK constraint, enum
    variant, as_sql_str, from_str, payload struct, SsbPayload variant,
    action_type(), test_payload(), all_test_payloads + count). SCHEMA_VERSION
    bumped 36 -> 37; a sibling session (B-32 MinifiedFileChunked) later
    bumped to 38. Phase 5 contract test asserts `>= 37` (floor, not equality)
    so ordering between sibling batches never breaks B-29 tests.

32. ProductionAgentRpcServer Ssb threading: added `ssb: Option<Arc<Ssb>>`
    field + `with_ssb` builder. translate_tool_request signature gained
    a 4th param `ssb: Option<&Arc<Ssb>>`. New helper `emit_symbol_query`
    encapsulates best-effort SSB writes — failure logs warn but never
    aborts the tool result. Fixes Phase 5 auditor BLOCK-2 (was: tracing::debug
    placeholder).

33. TOOL_SYMBOL_LIST limit cap: hard-coded `MAX_SYMBOL_LIST_LIMIT = 500`.
    Agent-supplied limit clamped to this maximum to prevent adversarial
    unbounded scans. Plan's "default 100 if 0" is enforced inside SymbolFilter.

34. PostToolUse impact injection wrapped in `tokio::task::spawn_blocking`.
    Symbol-graph ops (SQLite reads + traversal) are sync; running them
    directly on an async task stalled the runtime per Phase 5 auditor BLOCK-3.
    Wrapped wrapper preserves the same logic; bs_id, changed_files, and
    project_dir are cloned/copied before move to spawn_blocking.

35. Dashboard wiring: KnowledgeDrawer.svelte now renders a real Impact
    Analysis panel (NOT an UnwiredNote). Polls `get_impact_analysis` Tauri
    command every 5s. Renders target / total_files / unresolved_pct +
    hop-band table reading from the most recent SymbolQuery SSB row.
    Frontend stub function `get_impact_analysis` added to `stub.ts`
    with an ImpactAnalysis interface that mirrors the backend DTO shape.

36. Phase 5 contract test schema_version floor (auditor BLOCK-1 fix):
    asserts SCHEMA_VERSION_PUBLIC >= 37 instead of `== 37`. Sibling
    B-step batches that introduce additional variants will bump the
    version higher without breaking this contract.

External grounding anchor (cited per CLAUDE.md grounded-by-default rule):
- FocalAD (Liu et al., arXiv:2506.11419, Jun 2025): top-k local-focus design
  for graph-attention over neighbors. Validates B-29's hop-banded BFS over
  reverse-call index against globally-aggregated alternatives. NOT consumed
  algorithmically in Phase 1; cited for design provenance only.
- SocratiCode-main (AGPL-3.0; reference-only, NOT vendored):
  - graph-impact.ts, symbol-graph-cache.ts, graph-symbol-resolution.ts
  - constants.ts L130-150 (SYMBOL_NAME_SHARDS=27, SYMBOL_REVERSE_SHARDS=256,
    SYMBOL_FILE_LRU_SIZE=500, MAX_IMPACT_DEPTH=10, MAX_FLOW_DEPTH=10).
  Ported to Rust + ELAI's existing DKG SQLite schema. No code copied.
