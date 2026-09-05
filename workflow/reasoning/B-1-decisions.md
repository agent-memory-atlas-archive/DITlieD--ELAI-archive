B-1 Path Unification — Decision Log

Session: s-1368208
Phase: B-1
Date: 2026-04-25

DECISION 1 — Helper home

Question: do path helpers live in crates/core/src/lib.rs directly or in
a dedicated paths.rs module?

Choice: dedicated module (crates/core/src/paths.rs) with `pub mod paths`
exported from lib.rs.

Rationale:
- token-economy: keeps lib.rs free of layout boilerplate.
- discoverability: every consumer writes
  `use elai_core::paths::dkg_db_path` (single namespace).
- locality: B-1 doc-block + layout diagram + 8 constants live next to
  the 7 helpers; Anti-Stale-Training "named constants" rule is satisfied
  in one file.
- Testing: contract tests at `crates/core/tests/paths.rs` import the
  module verbatim with no re-export juggling.

DECISION 2 — Migration policy for pre-existing .elai/ directories

Question: do we auto-migrate or auto-delete `.elai/`, or leave it alone?

Choice: leave alone, log a one-time `tracing::warn` at CLI startup
when both `.elai/` and `.elai_cc/` exist.

Rationale:
- B-1 spec L94-98 explicit: "Do NOT auto-delete `.elai/` (user may have
  other tools using it)."
- destructive-op gate (CLAUDE.md): auto-delete is destructive; user
  must opt in. Manual cleanup is safe.
- the canonical reader is now `core::paths`; the legacy directory is
  inert from ELAI's perspective once the refactor lands.

DECISION 3 — Test fixture handling

Question: how do contract tests reference the canonical layout when
their fixture roots are tempdirs?

Choice: tests construct `project_root.join(ELAI_DATA_DIR_NAME).join(
INDEXES_DIR_NAME)` exactly the same way production does, then assert
equality with `core::paths::dkg_db_path(project_root)`.

Rationale:
- proves the helper agrees with the constants on every host platform
  (Windows MAIN_SEPARATOR + Unix slash both pass).
- `paths` helpers are pure path joins, so tempdir paths flow through
  unchanged — no mocking, no IO, no flake risk.

DECISION 4 — Severing the rule-engine per-session DKG binding

Question: cli/main.rs:7181-7188 (legacy expression
`session.persistence_path().parent()/dkg.db`) used to bind the rule
engine's DKG querier to the SESSION lifecycle, resolving to
`.elai/sessions/dkg.db`. Phase-3J L141-146 confirms this expression
NEVER hit a populated database — it silently fell through to
EmptyDkgQuerier.

Choice: replace the expression with
`core::paths::dkg_db_path(project_root)` (project-scoped). The
per-session DKG binding is intentionally severed.

Rationale:
- per-session DKG was never wired upstream — IndexingService writes
  one project-scoped DKG, not one per session.
- the existing fallback to `temp_dir/elai-dkg-{session_id}.db` was
  cosmetic; that path was never written either.
- collapsing to project-scoped restores the 3J.WIRE.STUBS contract:
  `DkgQuerierAdapter::new` now finds a real database and the rule
  engine consults real graph nodes.
- consequence: rule predicates that reference per-session DKG state
  (none currently exist) would need a different lifecycle hook; flag
  for B-3 audit.

DECISION 5 — models_dir_path layout

Question: spec Tasks L88-89 says callers expect `.elai_cc/models/onnx/`
but CLAUDE.md "Model location (primary)" places the active model at
`.elai_cc/models/Qwen2.5-Coder-7B-Instruct-abliterated/`. ONNX vector
models (graphcodebert, granite-embed) are NOT staged anywhere yet.

Choice: `models_dir_path(project_root) -> <root>/.elai_cc/models`.
Callers append `onnx/<name>.onnx` themselves (existing cli/main.rs
sites at the GraphCodeBERT, Granite, cross-encoder, SPLADE call sites
all want `.elai_cc/models/<file>.onnx`, not nested under `onnx/`).

Rationale:
- matches reality (Qwen tree at `.elai_cc/models/Qwen…/`).
- caller flexibility: cross-encoder may want a flat layout while
  SPLADE wants `splade/model.onnx` — the helper does not pre-judge.
- contract test
  `models_dir_path_layout_does_not_bake_onnx_subdir` enforces this
  invariant going forward.

DECISION 6 — Startup assertion firing order (Task #9)

Question: hard-assert `.elai_cc/` exists at CLI startup vs. fire AFTER
IndexingService had a chance to create it?

Choice: emit warnings AFTER `IndexingService::run_full_index`
completes. Do NOT panic.

Rationale:
- dashboard/lib.rs:884-888 graceful-degradation pattern: first run
  has no `.elai_cc/` until IndexingService runs. Hard-asserting at
  startup would break first-run UX.
- non-fatal warning makes the previously-silent empty-DKG failure
  visible without changing the contract that the CLI degrades
  gracefully.
- `tracing::info` on success establishes a positive log signal for
  the deterministic check.

DECISION 7 — Test file location for path_unification.rs

Question: spec lists `tests/contracts/path_unification.rs` but the
top-level `.elai_cc/tests/contracts/` directory has no Cargo target
(orphan dir; existing `bm25_contract.rs` there is dead code).

Choice: place the regression test at
`crates/contracts/tests/path_unification.rs` so it actually compiles
and runs under the existing `elai_cc_contracts` test target. The
orphan top-level directory is left untouched.

Rationale:
- only way to make `cargo test --test path_unification` succeed.
- consistent with all other in-tree contract tests (e.g.
  `task_runner_indexing_service_contract.rs`).
- documented here so the auditor can locate the test without grep.

OUT OF SCOPE (explicitly NOT touched in B-1)

- crates/cli/src/init.rs:12,85 — gitignore template `.elai/` entries.
- crates/cli/src/commands.rs:3402-3458 — help-text strings.
- crates/cli/src/main.rs:3640-3641 — legacy schema upgrade reader.
- crates/cli/src/main.rs:3892 — steering cache subdir.
- crates/cli/src/main.rs:5365 — sessions listing under `.elai/sessions`.
- crates/orchestrator/src/ssb.rs SSB_SESSIONS_SUBDIR.
- crates/orchestrator/src/prompt.rs / session_control.rs incidental
  hits.
- crates/tools/src/lib.rs Linux home defaults.
- pre-existing `.elai/` directories on user machines (warning only).

These are tracked in `crates/contracts/tests/path_unification.rs`
WHITELIST and intentionally NOT migrated by B-1. A separate B-step (or
a follow-up patch) handles the legacy-path eviction.
