# ELAI Time-Travel: Reversible LLM Edits + Lossless Session History — Design

Status: DESIGN (grounded, pre-decomposition). Authored 2026-06-05.
Scope: reconcile and complete H-35 + E-25 (Task 6a/6b) + E-29 + H-6 + C-23 + E-44 into one
coherent "undo everything the LLM did, redo it, at any moment, and never lose the start of a
long session" feature.

Grounding sources (strongest first):
- Code read this session via subagents (file:line throughout). The two load-bearing facts:
  (1) `tools/src/file_ops.rs` already captures the pre-edit bytes (`original_file`) before every
      overwrite and then discards them; (2) `orchestrator/src/architecture/edit_checkpoints.rs`
      already does byte-exact snapshot+restore (test `rewind_restores_file_content` passes) but is
      gated OFF and finds the edited path via a lossy heuristic.
- Existing plans: H-35, E-25, E-29, H-6, C-23, E-44, F-13, G-5, B-9 (DONE), phase-3H.txt.
- Prior art verified via WebSearch 2026-06-05 (citations at end): Cursor checkpoints, Cline/Roo
  shadow-git checkpoints, Aider git `/undo`, Vim/Emacs undo-tree.

This document does not change code. It is a blueprint for a future plan track.

---

## 1. Goal (restated from the request)

1. Every change the LLM makes to a file is reversible. Before any agent write/edit, the file's
   prior content is durably backed up inside the session.
2. Every session is saved; every file, every backup, every LLM action is revertible at any moment.
3. Redo: "if reverted, revert the reverted forward" — undoing is itself undoable, and re-applying
   an undone change is a first-class operation, not a destructive overwrite.
4. The big-session-viewing problem (the Claude Code complaint): when a session gets large you can
   only see the tail, never the start. ELAI must let you scroll to the very first message of an
   arbitrarily long session, always.

The unifying primitive for (1)-(3) is a **branching history tree with a movable HEAD** (an
undo-tree), backed by a **content-addressed file-snapshot store**. The unifying primitive for (4)
is **lossless append-only transcript storage decoupled from the model's context window**, plus a
**paged / virtualized viewer**.

---

## 2. What already exists (the substrate — do not re-invent)

Verified by codebase search this session. Status legend: LIVE / WIRE-DARK (real but gated off or
test-only reachable) / METADATA-ONLY / ABSENT.

- `tools/src/file_ops.rs:259-331` — `write_file` and `edit_file` both `fs::read_to_string(path)`
  into `original_file` BEFORE overwriting, compute `structured_patch` (`make_patch` at :551) and a
  `git_diff`, and return them in `WriteFileOutput`/`EditFileOutput`. LIVE. The pre-image is already
  in hand at the tool boundary — it is just never persisted. This is the highest-leverage hook in
  the whole system.
- `orchestrator/src/architecture/edit_checkpoints.rs` — `snapshot_pre_edit` (:241) writes the FULL
  pre-edit file to `.elai_cc/.session-checkpoints/{sid}/{seq}_{nanos}/{blake3}/{path}`; `rewind_to`
  (:350) restores byte-exact (highest-seq-first so a crash mid-rewind leaves newest-restored on
  disk). `SESSION_CHECKPOINT_ROOT` at :54. WIRE-DARK: auto-snapshot call site
  `task_runner.rs:6861-6909` is gated by `ELAI_E25_EDIT_CHECKPOINTS_ENABLE` (default OFF, :94) AND
  only fires when `extract_edit_target_from_task_summary` heuristically scrapes a path from the
  baby-step `task_summary` string (BabyStep carries no structured target_file field). Storage is
  full copies (no dedup) — E-25 flagged this can "balloon to GB-scale."
- CLI `checkpoint list` / `checkpoint rewind [--dry-run] [--steps N]` —
  `cli/src/checkpoint_cmd.rs:86-137`, dispatched from `CheckpointSub` (`args.rs:1267`). LIVE
  entrypoint, but only restores whatever the gated snapshotter happened to capture.
- `orchestrator/src/ssb.rs` — append-only single-writer WAL event log. `SsbActionType::FileWrite`
  (:5016) / `FileRead` (:5017); `FileWritePayload { path, size_bytes, hash }` (:10644). METADATA-ONLY
  and effectively ABSENT in production: no live agent path emits FileWrite rows, and the payload has
  no content/diff. The doc-comment at ssb.rs:17172 claiming "the live diff remains carried by
  FileWrite" is stale — there is no diff field. SSB cannot reconstruct a file.
- `orchestrator/src/session_dag.rs` (~3,600 LOC, LIVE) — Phase 3H SessionDag. `snapshots` table
  with self-FK `parent_id` (:213) makes it a real DAG; `snapshot_context` per-message rows
  (:232-242, msgpack BLOB content). `save_snapshot`/`load_snapshot`, `create_branch`,
  `MergeResult::{Winner,Conflict}` (:120), `parent_chain` walk. `create_handoff`/`load_handoff`
  wired to `/continue` (activation test `continue_uses_load_handoff_not_jsonl`). Stores conversation
  messages + a `code_changes` summary BLOB — NOT file bytes.
- `orchestrator/src/durability/` — `saga.rs` typestate saga with `compensate_tool:"undo"` (reverse
  action logged before each irreversible side effect; E-44). `resume_validator.rs` checkpoint
  validation (msgpack + blake3, but plain `fs::write`, no fsync+rename). LIVE/E-44 PLANNED.
- `dashboard-v2/.../TranscriptDrawer.svelte:29` — `ENTRY_LIMIT = 50`; polls `get_chat_history(50)`.
  Backend `ssb_viewer.rs get_chat_history` does `ORDER BY ... DESC LIMIT 50`. No pagination UI.
- `orchestrator/src/handoff_payload.rs:100-177` — 3-pass trim. `VERBATIM_TAIL_MESSAGES = 4`; older
  messages truncated to a char ceiling then summarized; if still over budget the oldest are DROPPED
  with a "[handoff: N prior messages elided]" sentinel. So compaction to a smaller window is LOSSY
  unless the originals are archived elsewhere.
- `recover_stubs` (`session_dag.rs:1507`) exists but has ZERO production callers; the live read-back
  path (`live_cli.rs:1252-1264`) substitutes a hardcoded "[stubbed — content archived in SSB]"
  placeholder. H-6 is the plan to activate it.

Already-planned but unshipped pieces that overlap this design:
- H-35 `rewind_over_ssb_journal` — `elai rewind <target> [--scope conversation|code|both]` +
  "Rewind to here" dashboard control + `rewind_session` Tauri cmd. Drives existing primitives,
  emits a `session_rewound` SSB beacon. Redo and branch-fork are EXPLICITLY DEFERRED (H-35:32).
- E-25 Task 6b — the `.session-checkpoints/` snapshot store (became `edit_checkpoints.rs`).
- E-25 Task 6a — `elai resume --fork <new_sid>` session-level fork with `forked_from:` lineage.
- E-29 — `TranscriptDrawer` + `ssb_transcript(session_id, filter)` (chat-shaped, filters,
  free-text search). No virtual-scroll / cursor pagination of the transcript itself.
- H-6 — activates `recover_stubs` for real read-back.
- C-23 — completes B-9 branch UI (real fork/merge/close Tauri cmds; `/continue`→`load_handoff`).
- E-44 — saga compensation log + Continue-As-New rollover + idempotency.
- F-13 — session archive schema bump.

---

## 3. Gaps (what the feature actually needs and the substrate lacks)

G1. No durable, default-ON, reliable per-edit byte store. `edit_checkpoints` is the right mechanism
    but is off, heuristic, and unwired from the tool's already-captured `original_file`.
G2. No redo / forward navigation anywhere. Both H-35 and `edit_checkpoints` are strictly backward.
    The request's "revert the reverted forward" has zero coverage today.
G3. Two unreconciled undo mechanisms: H-35 (SSB-journal deterministic replay) vs `edit_checkpoints`
    (byte snapshots). Two competing commands (`elai rewind` vs `elai checkpoint rewind`), no
    arbitration in the docs.
G4. No content addressing → snapshots are full copies → storage balloons → forces the feature to
    stay off. (E-25's own open question.)
G5. No divergence guard. Restoring blindly overwrites the working tree; a user's manual edits would
    be clobbered (the exact Cursor footgun — see prior art).
G6. Large-session viewing: `ENTRY_LIMIT=50` + no pagination + handoff drops old messages + CLI
    never replays transcript on resume + `recover_stubs` is dead. The start of a long session is
    currently unreachable for viewing.

---

## 4. Design principles (each grounded in prior art; citations in §13)

P1. Snapshot the bytes; do not rely on replay. Cursor and Cline/Roo both snapshot file state
    rather than replay the model. Replay (H-35) is fragile on the production cloud lane because
    cloud-model output is non-reproducible (H-35 itself downgrades that case to VerifyDivergence /
    best-effort, H-35:32). Snapshot-restore is model-agnostic and already proven by a passing test.
    => snapshot-restore is load-bearing; deterministic replay is an optional audit overlay only.
P2. History is a tree, not a stack. Vim/Emacs undo-tree: when you undo and then edit, you branch;
    nothing is ever lost; you can walk to any state. This is the only model that satisfies both
    "redo" and "at any moment" without data loss. ELAI already has a DAG (`SessionDag.parent_id`).
P3. Decouple scopes. Cline separates "Restore Files Only" from conversation restore. ELAI should
    let `--scope code | conversation | both` be independent, restorable axes over the same node.
P4. Never clobber the human. Aider commits the user's uncommitted work BEFORE editing, so it is
    never lost. ELAI's analog: a divergence guard that forks-before-restore when the working tree
    has drifted from what HEAD expects.
P5. Capture at the boundary you already pay for. `file_ops.rs` already reads `original_file`. Persist
    that, instead of re-reading via a heuristic. Deterministic, lossless, free of the task_summary
    guess.
P6. Lossless storage, lossy view. What the model sees (trimmed/stubbed) must be fully decoupled
    from what is stored (append-only, never deleted). Compaction is a view concern, not a storage
    concern.

---

## 5. Architecture

### 5.1 Core model — content-addressed snapshot store + HEAD-over-SessionDag undo-tree

Two cooperating layers:

(a) Object store (content-addressed, git-like). A blob = zstd-compressed file content keyed by
    `blake3(content)` (blake3 is already the repo standard per CLAUDE.md anti-stale rule). A
    `FileTree` manifest = an ordered map `{ project_relative_path -> blake3 }` describing the full
    set of agent-touched files at a point in time. Manifests are themselves blobs (hash of the
    serialized map), so identical trees dedup. Storage root reuses
    `.elai_cc/.session-checkpoints/{sid}/` but becomes `objects/{blake3[..2]}/{blake3}` +
    `manifests/{blake3}` instead of per-edit full-copy directories.

    Dedup is the property that makes "back up every change" affordable and therefore default-ON:
    an unchanged file across 500 edits is stored once; reverting a file to a prior content costs
    zero new blobs. This directly retires G4 and E-25's GB-balloon open question.

(b) History tree (the undo-tree). Reuse `SessionDag` as the spine — it already has `parent_id`,
    branching, and per-message context. Add to each snapshot node a `file_manifest: blake3` column
    (one `ALTER TABLE snapshots ADD COLUMN file_manifest` — the codebase already uses lazy
    `ensure_*_column` ALTER migrations, e.g. `ensure_raw_blocks_column`). A node therefore pins
    BOTH the conversation context (existing `snapshot_context`) AND the file-tree manifest (new).
    A movable `HEAD` ref per session points at the current node.

Operations:
- Capture (new edit): create a child node of HEAD whose manifest = HEAD's manifest with the edited
  path(s) repointed to the new blob(s); advance HEAD to it.
- Undo: `HEAD := HEAD.parent`; materialize that node's manifest into the working tree (only paths
  in the manifest; see divergence guard §5.4).
- Redo: `HEAD := chosen child of HEAD` (the child along the last-followed edge by default; if HEAD
  has multiple children — because you branched — prompt or pick most-recent). Materialize.
- Undo then new edit: the new edit creates a NEW child, so the old child (the "redo" timeline) is
  retained as a sibling branch. Nothing is lost. This is the literal "revert, then revert the
  revert forward, and also keep going a new way" behavior the request asks for.
- Jump to any node: materialize an arbitrary node's manifest. "At any moment" = any node is
  addressable; undo/redo are just the parent/child special cases.

Because undo/redo are pointer moves over an append-only tree, and a snapshot is taken of the
CURRENT state before any destructive materialize (§5.4), undo is itself undoable — redo is not a
fragile separate stack but simply the forward edge that still exists.

### 5.2 Two approaches considered (decision: CAS store; alternative: shadow-git)

APPROACH A (CHOSEN): ELAI-native CAS store over SessionDag, as in §5.1.
  + Works with no git repo present (Cline/Roo's checkpointing silently DISABLES when the workspace
    is not a git repo — GitHub Roo-Code #8164; a non-starter for ELAI's many non-git targets).
  + No nested-git footguns (Roo-Code #7765: nested git checkpoint deleted task contents).
  + Integrates with the existing SessionDag, SSB determinism (ELAI_SEED + ClockProvider), and the
    blake3 standard. One tree for conversation + files.
  + Reuses `edit_checkpoints.rs` (already tested) as the restore engine; reuses `file_ops`
    `original_file` as the capture source.
  - More code than shelling out to git plumbing.

APPROACH B (ALTERNATIVE, rejected as primary): shadow git repo, the Cline/Roo design — a separate
  `.git` with `core.worktree` pointed at the project; snapshot = `git add` + `git commit`, restore =
  `git clean` + `git reset --hard`.
  + Least code; git does the content-addressing, dedup, and packing for free.
  + Battle-tested object model.
  - Hard dependency on a working git + the workspace being (or being safely shadowable as) a repo;
    the documented failure modes above.
  - A second `.git` interacting with the user's real repo is a sharp edge (ignore rules, hooks,
    submodules, LFS, huge binaries).
  - Restore is whole-worktree (`reset --hard`) — coarser than per-path materialize; mixing with
    the user's own uncommitted work needs care.
  Verdict: keep shadow-git as a documented fallback engine behind the same `RestoreEngine` trait so
  a future config can select it, but ship the CAS store as default. (The git object model and the
  CAS model are the same idea; choosing our own store is about avoiding the integration hazards,
  not the data structure.)

### 5.3 Capture path (the write/edit hook)

1. Agent calls `write_file`/`edit_file` (`file_ops.rs`). These already produce `original_file`
   (pre-image), the new `content`/`updated` (post-image), and `structured_patch`.
2. A new orchestrator-side step (PostToolUse hook, the spot where the SSB FileWrite row is built)
   does, for each written path:
   - `pre_hash = blake3(original_file)` (or "absent" for a created file);
   - `post_hash = blake3(post_image)`; persist any not-yet-stored blob to the object store
     (zstd + write `objects/..`); idempotent (skip if hash present).
   - record an edit entry `{ path, pre_hash, post_hash, tool, ssb_seq, turn_id }`.
3. At the natural grouping boundary (default: per agent turn / baby step — see §5.5), create the
   child snapshot node with the updated `file_manifest` and advance HEAD.
4. Emit the off-agent observability rows (§7) — this is the wiring-enforcement artifact the agent
   did not author.

Note this replaces the heuristic entirely: capture is driven by the actual tool result, not by
scraping `task_summary`. It also fixes the "FileWrite carries no content" gap without bloating SSB
— SSB keeps `{path, size, hash}` (or gains `pre_hash`+`post_hash`), and the bytes live in the CAS,
referenced by hash. SSB stays a lean index; the object store holds content.

### 5.4 Restore path + divergence guard

Before materializing target node T:
1. Snapshot-current: if the working tree differs from HEAD's manifest (any tracked path's on-disk
   `blake3` != HEAD's expected hash, OR an untracked-by-us change), first create a child snapshot
   of HEAD capturing the current on-disk bytes, tagged `auto:pre-restore`. This guarantees the
   state you are leaving is always reachable (redo/forward), and is the Aider "never lose the
   human's work" guard against the Cursor "manual edits clobbered" footgun.
2. Compute the path delta between HEAD's manifest and T's manifest. For each differing path:
   - in T and changed/added: write T's blob to disk (atomic: temp file + fsync + rename — fixes the
     current non-atomic `fs::write`);
   - in HEAD but not in T (file was created after T): the file did not exist at T. Default: move to
     a `.elai_cc/.trash/{sid}/` rather than hard-delete (recoverable); configurable.
3. Move HEAD to T. Emit `session_rewound` (reuse H-35's beacon) with `{from, to, scope, paths_n}`.

Scopes (P3): `--scope code` materializes only the file manifest; `--scope conversation` moves only
the conversation view (existing `load_handoff` / context restore + `recover_stubs`); `--scope both`
does both atomically. Code and conversation HEADs may be the same node but are restorable
independently.

Honesty boundary (grounded, not optional): file edits are sound to undo. External, already-emitted
side effects (network calls, deletes outside the project, spend, git push, deploys, sent messages)
are NOT undone by this system — they are surfaced from the E-44 saga `compensation_actions` log as
"these N irreversible actions happened after the point you are rewinding to; here is the logged
compensation, run at your own risk." Never claim a deploy or a sent email is undoable.

### 5.5 Granularity and grouping

Capture at the finest grain (per file-edit, content-addressed so it is cheap). Expose navigation at
three grains, all backed by the same tree:
- Edit-level: each `write_file`/`edit_file`.
- Turn-level (default undo unit): all edits in one user-message→assistant turn collapse to one
  snapshot node. Matches Cursor/Cline "restore to this message."
- Phase / baby-step level: snapshots already emitted per GRC step / tool call / phase transition
  (B-9) tag the tree so you can rewind to "before this baby step."
Grain is a view over the tree (which ancestor you jump to), not separate storage.

---

## 6. Large-session viewing (the "can't see the start" fix)

Three coordinated changes; the first is the one that actually matters.

6.1 Lossless append-only transcript, decoupled from the model window (P6). Every user/assistant/
    tool message is written once to a durable per-session store and NEVER deleted. The 3-pass trim
    (`handoff_payload.rs`) and stubbing operate on a PROJECTION used to build the model prompt; they
    must not be the system of record. Concretely: when the trim drops/stubs a message for the model,
    the original is guaranteed already archived (SSB row or `snapshot_context`), and the stub keeps
    `stub_ssb_seq` so it is recoverable. Today this is half-true (the sentinel "[handoff: N prior
    messages elided]" implies loss); make archival a precondition of trimming, fail-closed.

6.2 Wire `recover_stubs` (H-6). The viewer and `--scope conversation` restore must be able to
    rehydrate a stubbed message from its `stub_ssb_seq` (blake3-verified). Today `recover_stubs`
    has zero production callers and the read path returns a hardcoded placeholder — that is the bug
    that makes the start of a session unviewable even when the data exists. H-6 + this design share
    the same fix; do it once.

6.3 Paged + virtualized transcript viewer. Replace `TranscriptDrawer`'s `ENTRY_LIMIT=50` with
    cursor pagination + windowed rendering:
    - Backend: extend E-29's `ssb_transcript` (and `get_chat_history`) with `(before_seq|after_seq,
      limit)` cursors and a `total_count` + `first_seq`. The data layer already supports offset+limit
      (phase-4K: "all list queries support offset + limit"); this exposes it as scroll-back.
    - Frontend: virtual scroll (render only the visible window of rows; Svelte 5 runes, no new dep
      required — a fixed-height windowing list is ~80 LOC). "Load older" triggers on scroll-up;
      a "Jump to start" control seeks to `first_seq`; a "Jump to message N / turn N" seeks by index.
    - This is exactly the Claude Code gap: there the displayed history is truncated and the start is
      unreachable; here the data is lossless (6.1) and paged in on demand (6.3), so the first message
      is always reachable.

6.4 CLI parity. On `/continue`/`/resume`, the CLI loads the full session (it already does —
    `resume.rs:74-102`) but prints only a banner. Add `elai history [--from N] [--grep ...]
    [--scope ...]` to page the transcript in the terminal, and a `--replay` flag to echo it.

---

## 7. Surfaces

CLI (unify the two competing commands under one model; keep `checkpoint` as the low-level alias):
- `elai undo [--steps N] [--scope code|conversation|both] [--dry-run]` — HEAD→ancestor.
- `elai redo [--steps N] [--scope ...] [--dry-run]` — HEAD→descendant (prompts on branch ambiguity).
- `elai history` / `elai timeline` — render the undo-tree (current node marked, branches shown);
  `elai jump <node-id|turn N|"before baby step X">`.
- `elai rewind <target> ...` — keep as H-35's alias for `undo` to a labeled target; `elai checkpoint
  list|rewind` becomes a thin wrapper over the same engine (resolves G3).
- `--dry-run` on every mutate prints the path delta and the irreversible-side-effect warning, never
  touches disk.

Dashboard:
- "Rewind to here" per-turn control (H-35) + a new "Redo" affordance and a small branch indicator
  when a node has multiple children.
- A Timeline / undo-tree view (the SessionDag branch tree from C-23, extended to show file-manifest
  diffs per node). Reuse the existing BranchTree.svelte component (shipped in B-9).
- The paged transcript viewer (§6.3) in TranscriptDrawer.
- "Restore Files Only" vs "Restore Files + Conversation" buttons (Cline parity, P3).

---

## 8. SSB action types + observability (off-agent artifacts)

Add typed, append-only rows (the gate evidence the agent does not author — wiring-enforcement R4):
- `file_snapshot_captured { path, pre_hash, post_hash, blob_stored: bool, turn_id, node_id }`
- `snapshot_node_created { node_id, parent_id, file_manifest, scope }`
- `session_rewound { from_node, to_node, scope, paths_changed, irreversible_warned: n }` (reuse
  H-35's beacon name)
- `redo_applied { from_node, to_node, scope }`
- `restore_diverged { node_id, drifted_paths, forked_to_node }` (divergence guard fired)
Object-store GC events: `objects_gc { freed_blobs, freed_bytes }`.

These also feed the dashboard timeline and make the feature auditable: "show me every file the
agent changed in turn 7 and let me revert exactly those."

---

## 9. Wiring-enforcement compliance (.claude/rules/wiring-enforcement.md)

This feature is where the project's own anti-wire-dark doctrine must be honored, because the current
state (edit_checkpoints default-OFF, heuristic, untested in prod) is the canonical wire-dark
failure.
- R9 / default-off graveyard: the capture path graduates from `ELAI_E25_EDIT_CHECKPOINTS_ENABLE`
  (default OFF) to DEFAULT-ON, with a kill switch `ELAI_TIMETRAVEL_DISABLE` registered in
  `wire-dark-allowlist.toml` (type=kill_switch, owner, expiry, promotion_plan_id) plus a smoke test
  exercising the ON path. A flag with no promotion path is wire-dark by design.
- R4 off-agent artifact: the done-gate asserts the SSB `file_snapshot_captured` / `session_rewound`
  rows (which the executor does not author) landed after driving a REAL entrypoint (`elai undo` /
  the PostToolUse hook), not a unit test calling `rewind_to` directly.
- R1/R2 missing=FAIL: if a smoke run edits a file and no `file_snapshot_captured` row exists, the
  gate FAILS (not SKIP). The object blob must be readable back and blake3-verify.
- R6/R7: the snapshot capture has a production caller (the PostToolUse hook), not only the CLI/test.
- Walking-skeleton PRE_GATE: each unit ships producer + consumer + smoke together — e.g. "add
  manifest column AND write it on capture AND assert `elai undo` reads it AND a smoke test rewinds a
  real edit."

---

## 10. Reconciliation with existing plans

- H-35 (rewind): keep its CLI surface and `session_rewound` beacon; replace its "deterministic
  replay reconstructs files" assumption with CAS snapshot-restore (P1). Its replay path becomes an
  optional `--verify` audit overlay, not the restore mechanism. Its DEFERRED redo + branch-fork are
  DELIVERED here (the undo-tree gives both for free).
- E-25 Task 6b (`edit_checkpoints`): becomes the restore engine; upgrade storage from full-copy to
  CAS (retires its eviction open-question), wire capture to `file_ops.original_file`, flip default-ON.
- E-25 Task 6a (`resume --fork`): subsumed — a fork is creating a new branch from any node; expose
  as `elai fork [<node>]`.
- E-29 (TranscriptDrawer / ssb_transcript): extend with cursor pagination + virtual scroll (§6.3).
- H-6 (recover_stubs activation): shared dependency for §6.1/6.2 — implement once, both consume.
- C-23 (branch UI): the Timeline/undo-tree view extends C-23's BranchTree.
- E-44 (saga compensation): the source of the irreversible-side-effect warnings in §5.4; not a file
  undo, do not conflate.
- F-13 (session archive schema bump) + G-5 (FileWrite payload): align the schema bump with the new
  `file_manifest` column and the `pre_hash/post_hash` extension; G-5's review-density use of FileWrite
  is compatible (it reads counts, not content).

---

## 11. Risks and edge cases

- Manual edits between agent edits (Cursor footgun): handled by the §5.4 divergence guard
  (fork-before-restore). Also snapshot user-visible-but-untracked changes opportunistically.
- Irreversible side effects: explicitly out of file-undo scope; surfaced from E-44 saga log with a
  warning. Do not oversell.
- Cloud lane: snapshot-restore is model-agnostic, so unlike H-35 replay it works identically for
  cloud Sonnet (the actual production lane). No VerifyDivergence weakness for the file scope.
- Storage growth: CAS dedup + zstd bounds it; add GC — keep all blobs reachable from any retained
  node; prune nodes by policy (keep last K turns + all user-tagged + all branch tips; never auto-
  prune below the session start). GC is the only place data is deleted, and it is reachability-based.
- Large / binary files: cap per-blob size (config, e.g. 5 MB) above which store a pointer + skip
  content (note it as "not snapshotted, too large"); never silently pretend it was captured (R1).
- Concurrent sessions on the same files (cross-session file locks already exist in the PreToolUse
  hook): capture is per-session; a restore that would cross another session's lock must refuse, not
  clobber.
- Determinism: all hashing blake3, all timestamps via ClockProvider, all ordering by SSB seq, so the
  tree and GC are reproducible under ELAI_SEED (consistent with phase-3H/3I determinism gates).
- Atomicity: materialize uses temp+fsync+rename per file (fixes the current plain `fs::write`); the
  highest-seq-first restore order from `edit_checkpoints.rs` already gives crash-safe partial restore.

---

## 12. Suggested decomposition (vertical slices, each ships producer+consumer+smoke)

1. Object store + manifest types + `file_manifest` column ALTER; CAS write/read with zstd+blake3;
   unit + smoke (write blob, read back, verify hash).
2. Capture hook: persist `original_file`/post-image from `file_ops` at the PostToolUse point; emit
   `file_snapshot_captured`; default-ON + kill switch + allowlist row. Smoke: real edit → SSB row +
   blob present.
3. HEAD ref + undo/redo over SessionDag; `elai undo`/`elai redo` driving `rewind_to`/forward
   materialize; divergence guard + fork-before-restore. Smoke: edit, `elai undo`, file is prior
   bytes; `elai redo`, file is new bytes; edit-after-undo creates a branch (assert sibling node).
4. Scopes (code/conversation/both) + reconcile H-35 `rewind` + `checkpoint` aliases.
5. Lossless transcript precondition + `recover_stubs` wiring (with H-6).
6. Paged + virtualized transcript viewer (with E-29) + `elai history`.
7. Dashboard timeline/undo-tree view + Restore-Files-Only vs Both (with C-23 component).
8. GC + retention policy + `objects_gc` row.
9. Irreversible-side-effect surfacing from E-44 saga log in `--dry-run`/restore warnings.

---

## 13. Prior-art citations (verified 2026-06-05)

- Cursor checkpoints — snapshot pre-change state before every agent edit, local hidden store
  separate from git, per-message restore, "continue from reverted state by writing a new message";
  tracks ONLY AI edits (manual edits not captured). https://cursor.com/docs/agent/chat/checkpoints
- Cline checkpoints — shadow git repo via `core.worktree`, snapshot = add+commit, restore = clean +
  reset --hard, "Restore Files Only" vs conversation restore. https://docs.cline.bot/features/checkpoints
- Roo Code checkpoints — same shadow-git design; documented failure when workspace is not a git repo
  (Roo-Code #8164) and nested-git data loss (#7765). https://docs.roocode.com/features/checkpoints
- Aider git integration — commit-per-edit; `/undo` reverts aider's last commit; commits the user's
  pre-existing uncommitted work first so it is never lost. https://aider.chat/docs/git.html
- Emacs undo-tree (Toby Cubitt) — undo history as a branching tree; never lose history; persistent
  to disk. https://www.dr-qubit.org/undo-tree.html
- Vim undotree (mbbill) — visualizes the branching undo tree; Vim 7 stores undo as a tree, g-/g+
  walk all states. https://github.com/mbbill/undotree
- Checkpoint/restore systems survey for AI agents (context). https://eunomia.dev/blog/2025/05/11/checkpointrestore-systems-evolution-techniques-and-applications-in-ai-agents/

---

## 14. Open questions

- Default undo grain: turn-level (Cursor/Cline parity) vs edit-level. Recommend turn-level default
  with edit-level drill-down. Needs a user decision.
- Conversation-scope restore semantics when branching: does `elai undo --scope conversation` also
  move the file HEAD? Recommend independent HEADs with a "linked" default and an `--unlink` escape.
- GC retention numbers (K turns, blob size cap) — pick after measuring real session sizes.
- Whether to expose the shadow-git engine (Approach B) as a selectable `RestoreEngine` for users who
  want their checkpoints inside a real git they already trust.
