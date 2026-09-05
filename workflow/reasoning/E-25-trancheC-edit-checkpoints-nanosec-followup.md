E-25 Tranche C -- edit_checkpoints next_seq race fix (s-22600 Tranche G G.2 followup)

Date: 2026-06-01
Source: workflow/sessions/s-22600/context-package-trancheG.md item G.2 (audit WARN-2 followup)
File touched: .elai_cc/crates/orchestrator/src/architecture/edit_checkpoints.rs
Contract test added: .elai_cc/crates/contracts/tests/e25_edit_checkpoints_parallel_contract.rs

Race window (pre-G.2)

Two concurrent calls to `snapshot_pre_edit` on the same session both observe `max_seen = N`, both compute `seq = N+1`, and both create `{N+1:06}/{blake3_prefix}/...`. The first writer's snapshot lives under one blake3 subdir; the second writer's lives under (potentially) a different blake3 subdir of the SAME `{N+1:06}` parent. The on-disk file paths do not collide directly, but `list_session_checkpoints` returns two entries with `seq = N+1` in non-deterministic order, breaking rewind discoverability ("which one am I rewinding past ?"). The pre-G.2 docstring acknowledged this ("the worst case is two snapshots claim the same seq, in which case the second loses its rewind discoverability -- acceptable per Claude Code session-single-writer model"). G.2 hardens against the case where the session-single-writer assumption breaks (multi-tool dispatch in parallel sub-agents).

Option chosen: B (nanosec suffix on seq filename)

Three options weighed:
  A. Lockfile (homegrown CreateFile-based exclusive open or fs2 crate)
  B. Nanosec suffix on the seq directory name (`{seq:06}_{nanos:09}/`)
  C. Lock + nanosec

B chosen because:
  - Zero new deps. fs2 / file_lock would add a Cargo.toml entry violating CLAUDE.md no-new-deps-without-confirm. Homegrown CreateFile-based exclusive open on Windows adds a platform branch.
  - Zero extra IO. Nanosec read is a single SystemTime::now call; the lock option adds an OpenOptions::new().create_new(true).open() round-trip per snapshot.
  - Preserves SSB wire shape. `build_snapshot_payload` reads `cp.seq + cp.blake3_prefix + cp.path` -- the nanos suffix lives only in the on-disk directory name + the new `EditCheckpoint.nanos: u64` field. `details_json.nanos` is additive (Pattern-2 free-form details).
  - Preserves lex sort. The `{seq:06}` prefix keeps the lex-sort that `list_session_checkpoints` relies on; the `_{nanos:09}` suffix only breaks ties within the same seq.
  - Preserves rewind contract. Rewind keys on `seq`; nanos is cosmetic for the rewind operator's perspective.

C rejected as overkill -- the nanos suffix alone eliminates the on-disk collision; a lock adds belt-and-suspenders without measurable benefit at the documented load (single-session multi-tool dispatch).

Legacy compatibility (read-side)

`parse_seq_dirname` accepts both `{seq:06}` (legacy) and `{seq:06}_{nanos:09}` (new). Legacy directories parse with `nanos = 0`. `next_seq` still computes the right next value; `list_session_checkpoints` emits legacy entries with `nanos = 0` (cosmetic only); `rewind_to` and `evict_oldest` operate on `seq` regardless. No data migration required; new and old snapshots coexist on disk.

Schema impact

ZERO. SSB consumer side reads `details_json.nanos` as Option (Pattern-2 free-form). The 9-arm `ClaudeCodeArchitectureEventKind` enum LOCKED per session-4 DECISION-5 stays untouched. SCHEMA_VERSION 195 stable.

Risk

MEDIUM at edit time (pub-struct field add). Grep across `.elai_cc/crates` confirmed ZERO external constructors of `EditCheckpoint` (only the in-module tests at L513 + the existing E-25 contract). The one in-module test fixture was updated inline; `cargo check -p elai-orchestrator --tests` GREEN. `cargo check --workspace --tests` GREEN (no other crate breaks).

Verification

- Unit tests added to in-module `tests` mod: `parse_seq_dirname_round_trips_new_format`, `parse_seq_dirname_accepts_legacy_bare_seq`, `parse_seq_dirname_rejects_non_numeric_prefix`, `snapshot_unique_under_parallel_writers` (8 threads, distinct on-disk paths + distinct `(seq, nanos)` pairs).
- Contract test: `e25_edit_checkpoints_parallel_contract.rs` -- standalone cross-crate test pinning the same invariant from outside the orchestrator crate (so a future refactor that breaks the wire is caught at the contract boundary).
- `cargo check --workspace --tests` 2026-06-01: zero errors.
