C-17 Knowledge Indexing Resilience — Decisions

Session: s-32952  Phase: C-17 EXECUTING  Date: 2026-05-18

Scope: close MISSING/PARTIAL gaps from B-30, B-31, B-31.5, B-34, phase-3K — Kotlin
grammar, ChunkKind Heading/Paragraph, CLI auto_resume + signal handlers,
LockScope::Watch consumer, real ONNX dot-product (preserved from C-8), Oracle
pre-seeding, scale benchmark TOML population, indexing resilience contract.

==================================================================
DECISIONS
==================================================================

DECISION-1 (DRIFT-H) — Kotlin grammar via tree-sitter-kotlin-ng = "1.1"
  Anti-slopsquatting verified 2026-05-18 via `cargo search tree-sitter-kotlin`:
  the canonical crate from the official tree-sitter-grammars GitHub org is
  published as `tree-sitter-kotlin-ng v1.1.0`. The older `tree-sitter-kotlin`
  v0.3 crate is a different lineage; the -ng variant is the actively
  maintained one with current upstream parity. Chose the -ng crate rather
  than the JavaGrammar-fallback option because (a) Kotlin-specific node
  kinds (`function_declaration`, `primary_constructor`, `object_declaration`,
  `property_declaration`, `enum_entry`) carry information the Java grammar
  cannot represent, and (b) the resolver layer at `dkg/mod.rs:3279`
  already special-cases "kotlin" in `clean_import_name`, indicating the
  prior implementation contemplated a first-class Kotlin grammar all
  along.

DECISION-2 (DRIFT-A) — ChunkKind round-trip test extended
  Added `ChunkKind::Heading` and `ChunkKind::Paragraph` variants to the
  enum, plus `as_str` / `from_str` arms. The round-trip test in
  `bm25.rs:1063-1078` was extended to include the new variants. Without
  this, the assertion would silently lose either variant (they would
  decode as `File` via the default arm).

DECISION-3 (DRIFT-B) — ONNX dot-product preferred; norm-proxy logged-fallback
  C-8 already shipped the `Some(embedder)` dot-product path. The C-17 task
  was framed as "remove the norm proxy" but the C-8 decisions explicitly
  keep the proxy as a documented fallback for two degraded states (no
  embedder installed; embed() returned Err). Per orchestrator decision B,
  the proxy remains as a logged fallback. The arag.rs doc comment was
  updated to make this explicit: dot-product is the production path; the
  proxy is reserved for degraded states with a `tracing::warn!`. The
  WIRING_CONTRACT stub_ban in the C-17 plan ("norm proxy is forbidden")
  is interpreted as "proxy is forbidden as the default code path when an
  embedder is present", which holds.

DECISION-4 (DRIFT-C) — graceful_shutdown signature unchanged
  The dashboard caller pattern at `dashboard-v2/src-tauri/src/lib.rs:787`
  is the canonical example: `(indexes_root, deadline, clock)`. The C-17
  CLI wiring (`c17_lifecycle.rs::install_lifecycle`) mirrors the same
  shape using `SHUTDOWN_DEADLINE` for the deadline argument.

DECISION-5 (DRIFT-D + DRIFT-E) — Per-arm auto_resume + process-wide signal observer
  Per orchestrator decision D/E: auto_resume + signal handlers wired inside
  CliAction::Run / ::Repl / ::Prompt arms, NOT in main(). Each arm calls
  `c17_lifecycle::install_lifecycle(&project_root)` which (a) runs
  `auto_resume_indexed_projects` synchronously via a transient
  current-thread tokio runtime, and (b) installs a process-wide
  SIGINT/SIGTERM observer exactly once (`OnceLock` guard). The observer
  runs in a dedicated thread with its own runtime so it does not compete
  with foreground runtimes for the reactor.

  Per-scope `ctrl_c` handlers in `run_subcommand.rs:376` and
  `live_cli.rs:726` continue to observe the same OS signal — tokio installs
  a multiplexed listener so all consumers wake on a single SIGINT. The
  process-wide observer's only job is to flip the `SHUTTING_DOWN` flag and
  invoke `graceful_shutdown` so the metadata DB cancellation pass runs.

  Short-lived actions (Version, Help, DumpManifests, etc.) deliberately
  skip the lifecycle helper — they have no indexing state and no scope
  long enough to graceful-shutdown.

DECISION-6 (DRIFT-F) — FileWatcherService minimal lifecycle skeleton
  `crates/orchestrator/src/watcher_service.rs` is the first production
  consumer of `LockScope::Watch`. The current implementation is a lock
  lifecycle skeleton — acquires the Watch-scope lock on `start`, holds it
  for the lifetime of the value, releases it on Drop. Future revisions
  (C-18+) can layer inotify / ReadDirectoryChangesW change notification
  on top of this guard without disturbing the lock-acquire seam pinned
  by `c17_indexing_resilience_contract.rs`.

  Greenfield wiring lives at the post-resume point in `startup.rs`:
  after `auto_resume_indexed_projects` finishes, we iterate every
  metadata DB found, derive the project_root, and call
  `FileWatcherService::start(..)`. The guards are stashed in a
  process-global `Mutex<Vec<..>>` so the lock holds for the lifetime of
  the process. `graceful_shutdown`'s existing `force_release` loop at
  `startup.rs:620` already iterates both `LockScope::Index` and
  `LockScope::Watch`, so cleanup on shutdown is already structurally
  ready.

DECISION-7 (DRIFT-G) — Heading/Paragraph splitter between AST and line-block
  Added `is_prose_extension(path)` helper matching .md / .markdown / .txt
  / .rst / .adoc. In both `chunk_file` and `make_chunks_from_content`,
  when `try_ast_chunk` returns empty AND the file matches a prose
  extension, we route through `heading_paragraph_chunks` before falling
  back to the 50-line `line_block_chunks` path. The minified-first
  short-circuit (B-32 D1) still fires first; the prose splitter is a
  middle layer between AST and line-block paths.

  The splitter detects ATX-style (`#`, `##`, ..., `######`) AND
  Setext-style (`===`, `---` underlines) headings. When at least one
  heading is found, every section between heading boundaries becomes a
  `Heading`-kind chunk. When no headings are found, blank-line-delimited
  paragraphs become `Paragraph`-kind chunks. The existing
  `make_chunks_dispatches_correctly` test was updated to assert the new
  prose-routing contract: .txt files now produce Paragraph or Heading
  chunks instead of LineBlock chunks for short files.

DECISION-8 (DRIFT-I) — IndexCheckpointPayload.batch_duration_ms as Option<u64>
  Added `batch_duration_ms: Option<u64>` to `IndexCheckpointPayload` with
  `#[serde(default, skip_serializing_if = "Option::is_none")]` so the
  field is back-compat with pre-C-17 emitters. Older payloads that lack
  the field decode as `None` automatically via the existing rmp-serde
  named encoding.

  Per orchestrator decision I, NO SCHEMA_VERSION bump is required for
  this additive optional field — rmp-serde's named-map encoding tolerates
  added/removed fields when readers are tolerant via `#[serde(default)]`.
  The existing `IndexCheckpoint` action_type enum stays at SCHEMA_VERSION
  39 with no change.

  Per-batch wall-clock duration is measured via `clock.instant_now()` at
  batch loop entry and again at the checkpoint emit site
  (`indexing_service.rs:524`). The injected `Arc<dyn ClockProvider>`
  routes through OsClock by default so the entropy quarantine (C-15)
  stays intact. The contract test
  (`c17_indexing_resilience_contract.rs::index_checkpoint_payload_batch_duration_optional`)
  pins both the forward path (encode + decode preserves the field) and
  the back-compat path (None encodes/decodes as None).

DECISION-9 (DRIFT-J) — Dashboard retarget to IndexingProgressPanel + KnowledgeDrawer
  The plan named `ActivityFeed.svelte` as the destination, but no such
  file exists. Per orchestrator decision J:
  * `IndexingProgressPanel.svelte` gained a RESILIENCE sub-block that
    renders auto_resume toggle status, Watch-scope lock count, last
    IndexResume banner (when resumed_from_batch > 0), LockContested
    banner (when count > 0), and the C-17 batch_duration_ms surface.
  * `KnowledgeDrawer.svelte` embeds the panel (it already did); a
    comment annotation was added to document the new resilience role.
  * A new tauri command `get_indexing_resilience` was added to
    `dashboard-v2/src-tauri/src/commands.rs`. It probes the
    project-metadata DB for `last_checkpoint_at`, counts on-disk
    `watch.lock` files for the Watch-scope lock count, and queries the
    SSB viewer for the most recent IndexResume / LockContested /
    IndexCheckpoint rows. The handler also surfaces
    `SHUTDOWN_DEADLINE.as_secs()` so the dashboard stays in lockstep
    with the orchestrator constant.

DECISION-10 (DRIFT-K + DRIFT-M) — Scale TOMLs populated with measured-in:C-20 markers
  Production-scale fixture runs (Linux v6.7, VS Code 1.94.2, optional
  Chromium / Postgres) require HARDWARE-HEAVY local execution per
  CLAUDE.md. Per orchestrator decision M, every tier TOML now carries
  a `[c17_followup]` table with:
    * `measured_in_phase = "C-20"` — explicit deferred-measurement marker
    * `production_scale_fixture = "linux-v6.7"` — the required fixture
  The synthetic 200-file baseline stays in place as the regression
  baseline. `run_tier_a.sh` was updated to parse real per-batch p50/p99
  from `indexing_service` `batch_complete` tracing lines (RUST_LOG=info)
  rather than hardcoded zeros.

DECISION-11 (DRIFT-L) — SHUTDOWN_DEADLINE const at startup.rs
  Per orchestrator decision L, added
  `pub const SHUTDOWN_DEADLINE: Duration = Duration::from_secs(30)` so the
  contract test can grep / pin it. The pre-existing
  `STARTUP_SHUTDOWN_DEADLINE_SECS = 60` stays in place for the dashboard
  caller — both constants serve different surfaces:
    * `SHUTDOWN_DEADLINE` (C-17): the contract-pinned canonical deadline
      that the CLI signal observer uses. 30s value.
    * `STARTUP_SHUTDOWN_DEADLINE_SECS` (B-31): the dashboard's literal,
      passed in as a Duration argument to graceful_shutdown. 60s value.

DECISION-12 (DRIFT-N + DRIFT-O) — Oracle pre-seeding via existing EntryPointsDetected variant
  The orchestrator decision O instructed "add EntryPointsDetected SSB
  variant (SCHEMA 53->54)". However, on read of `ssb.rs:1042`, the
  `EntryPointsDetected` variant ALREADY exists from B-34
  (SCHEMA_VERSION 42), with its payload at `ssb.rs:2870`. The variant
  was added by B-34 as a one-row-per-tool-call beacon for the
  `/entry_points` slash command. The C-17 work simply needed to:
    1. Wire `detect_entry_points` into the `/research` arm (DRIFT-N).
    2. Emit an `EntryPointsDetected` row from the new call site (DRIFT-O
       — variant reused, no schema bump).
  No SCHEMA bump was performed. The 9-tuple test_payloads count stays
  at 107. This is the cleanest path: the variant exists, the payload
  shape is the same (project_id + count + by_reason), and reusing the
  variant means dashboard consumers do not need to be updated to handle
  a duplicate variant.

  The implementation lives in `live_cli.rs::c17_research_oracle_prefix`
  at the bottom of the file. It opens the SymbolGraphCache, runs
  detect_entry_points, caps the prepended block at
  `C17_MAX_ENTRY_POINTS_IN_BLOCK = 25` so the augmented prompt stays
  under ~1 KB, and emits the SSB row with the full per-reason buckets
  (not just the displayed subset).

==================================================================
CONTRACT
==================================================================

`crates/contracts/tests/c17_indexing_resilience_contract.rs` (6 tests):
  (a) `IndexResult` variant inventory exhaustive match.
  (b) `IndexLock::try_acquire` requires LockScope parameter (compile-time
      fn-pointer assertion).
  (c) `auto_resume_indexed_projects` returns within 5s on empty input.
  (d) `SHUTDOWN_DEADLINE` is a Duration in [30, 120] seconds.
  (e) `ChunkKind::Heading` and `ChunkKind::Paragraph` round-trip via
      as_str / from_str (DRIFT-A regression guard).
  (f) `IndexCheckpointPayload.batch_duration_ms` round-trips both
      Some(N) and None values (DRIFT-I additive field).

All 6 tests passed on first run.

==================================================================
NEW DEPENDENCY
==================================================================

`tree-sitter-kotlin-ng = "1.1"` added to
`crates/knowledge/Cargo.toml`. Anti-slopsquatting verification recorded
in DECISION-1 above. Source: github.com/tree-sitter-grammars/tree-sitter-kotlin
(official tree-sitter-grammars org).

==================================================================
TEST RESULTS
==================================================================

* `cargo check --workspace` — clean, 0 errors, 0 warnings beyond
  pre-existing dead-code lints unrelated to C-17.
* `cargo test -p elai_cc_contracts --test c17_indexing_resilience_contract`
  — 6/6 PASS.
* `cargo test -p elai-knowledge --lib bm25::` — 14/14 PASS (includes
  the two new heading/paragraph splitter tests and the updated
  make_chunks_dispatches_correctly assertion).
* `cargo test -p elai-orchestrator --lib lock::` — 11/11 PASS.
* `cargo test -p elai-orchestrator --lib startup::` — 4/4 PASS.
* `cargo test -p elai-orchestrator --lib watcher_service::` — 1/1 PASS.
* `cargo test -p elai-orchestrator --lib indexing_service::` — 4/4 PASS.
* `cargo test -p elai-orchestrator --lib ssb::tests::test_typed_payloads`
  — PASS (107 variants round-trip).
* `cargo test -p elai-orchestrator --test ssb_b31_variants` — 6/6 PASS.

Knowledge crate has 4 pre-existing unrelated lib-test failures
(reranker ORT non-determinism, pipeline canary) — confirmed by
running them in isolation against the pre-C-17 file set; they are
NOT caused by C-17 changes.

==================================================================
FILES TOUCHED
==================================================================

ADDED:
  .elai_cc/crates/orchestrator/src/watcher_service.rs
  .elai_cc/crates/cli/src/c17_lifecycle.rs
  .elai_cc/crates/contracts/tests/c17_indexing_resilience_contract.rs
  workflow/reasoning/C-17-decisions.md (this file)

MODIFIED:
  .elai_cc/crates/knowledge/Cargo.toml (tree-sitter-kotlin-ng = "1.1")
  .elai_cc/crates/knowledge/src/dkg/mod.rs (KotlinGrammar + ext_to_lang_id + registry)
  .elai_cc/crates/knowledge/src/bm25.rs (ChunkKind::Heading + ::Paragraph + splitter + tests)
  .elai_cc/crates/knowledge/src/arag.rs (doc comment update; C-8 dot-product retained)
  .elai_cc/crates/orchestrator/src/lib.rs (pub mod watcher_service)
  .elai_cc/crates/orchestrator/src/startup.rs (SHUTDOWN_DEADLINE + watcher start)
  .elai_cc/crates/orchestrator/src/indexing_service.rs (per-batch wall-clock + batch_duration_ms emit)
  .elai_cc/crates/orchestrator/src/ssb.rs (IndexCheckpointPayload.batch_duration_ms field + test fixtures)
  .elai_cc/crates/orchestrator/tests/ssb_b31_variants.rs (IndexCheckpointPayload literal update)
  .elai_cc/crates/cli/src/main.rs (mod c17_lifecycle + install_lifecycle in Run/Repl/Prompt arms)
  .elai_cc/crates/cli/src/live_cli.rs (Oracle pre-seeding in /research arm)
  .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs (get_indexing_resilience handler)
  .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs (handler registration)
  .elai_cc/crates/dashboard-v2/src/lib/components/IndexingProgressPanel.svelte (resilience surface)
  .elai_cc/crates/dashboard-v2/src/lib/drawers/KnowledgeDrawer.svelte (resilience embed annotation)
  .elai_cc/scripts/scale_test/run_tier_a.sh (real per-batch p50/p99 parsing)
  .elai_cc/metrics/phase_b31_5_tier_a.toml (measured-in:C-20)
  .elai_cc/metrics/phase_b31_5_tier_b.toml (measured-in:C-20)
  .elai_cc/metrics/phase_b31_5_tier_c.toml (measured-in:C-20)
  .elai_cc/metrics/phase_b31_5_tier_d.toml (measured-in:C-20)
  .elai_cc/metrics/phase_b31_5_tier_e.toml (measured-in:C-20)
  .elai_cc/metrics/phase_b31_5_scale.toml ([c17_followup] block)

==================================================================
DEFERRED
==================================================================

* Production-scale fixture runs (Linux v6.7, VS Code 1.94.2, Chromium,
  Postgres) — HARDWARE-HEAVY, deferred to C-20 per orchestrator
  decision M and DRIFT-K. Synthetic 200-file baseline remains as the
  regression baseline. `measured_in_phase = "C-20"` markers on every
  tier TOML.

* svelte_extraction.rs test — the plan's task #3 mentioned creating
  `crates/knowledge/tests/svelte_extraction.rs`. The svelte grammar
  (SvelteGrammar at dkg/mod.rs:1591-1620) is exercised by the existing
  knowledge lib tests and produces the qualified `<file>.svelte::`
  prefix correctly. C-17 closed every higher-priority MISSING/PARTIAL
  gap; the svelte test is the only task #3 item not landed in this
  session. Schedule for a follow-up patch when the broader test
  coverage pass runs.

==================================================================
END
