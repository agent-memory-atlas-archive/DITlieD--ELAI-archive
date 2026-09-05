# MiMoCode Part 7: Memory, History, Snapshot, Storage, Project, Account, Task, Worktree, Git

## Per-File Summaries

### memory/index.ts
Barrel export of the Memory service from `./service`. Entry point for the memory subsystem.

### memory/fts.sql.ts
**Purpose:** Define Drizzle ORM table schema for full-text search over markdown memory files.

**Key exports:** `MemoryFtsTable` — SQLite table with FTS5 virtual index.

**Mechanism:** Stores memory documents with content-addressable metadata:
- `path`: absolute file path (unique constraint)
- `scope` + `scope_id`: hierarchical scoping (e.g., project-id, phase-id)
- `type`: document category (derived from MIME or frontmatter)
- `body`: markdown text
- `fingerprint`: `{size}-{mtimeMs}` for change detection
- `last_indexed_at`: reconciliation timestamp
- Dual indexes on (scope, scope_id) and (type) for filtered search

### memory/service.ts
**Purpose:** Provide search interface over memory files via FTS5 with semantic ranking.

**Key exports:** `Service` (Effect service), `Interface` (search + reconcile methods).

**Mechanism:**
1. **Reconciliation:** On-demand walk of `{root}/memory/**/*.md` (mimo) and `~/.claude/projects/*/memory/**/*.md` (Claude Code projects); compute fingerprints to detect updates; reindex only changed files.
2. **Search:** Token-level FTS5 query (`buildFtsQuery`): split user input into alphanumeric tokens, wrap each in phrase quotes, OR-join (high recall, BM25 ranks by term rarity). Over-fetch 3x, apply relative score floor (keep top hit + results >= `floorRatio * topScore`, default 0.15). Returns `{path, snippet, score, scope, scope_id, type}`.
3. **Snippet generation:** FTS5 `snippet()` with context window = 32 chars, markers `<< >>`.
4. **Config-driven:** `config.checkpoint?.memory_reconcile_on_search` auto-reconciles before each search; `memory_search_score_floor` configures the relative threshold.

### memory/reconcile.ts
**Purpose:** Scan on-disk memory trees, compute fingerprints, detect changes, upsert DB.

**Key exports:** `walkMemoryDir`, `walkCcRoot`, `indexFromDisk`.

**Mechanism:**
- Walk directory tree collecting all `.md` files.
- For each file: read stat, hash as `{size}-{mtimeMs}`.
- Compare to stored fingerprint; skip if unchanged.
- Parse frontmatter (CC projects) or derive type from path (MiMoCode); read body.
- Upsert into `MemoryFtsTable` with ON CONFLICT DO UPDATE.
- Return `{indexed, pruned}` counts.

### memory/paths.ts
(Not fully read, but referenced) Parses memory file paths to extract `scope`, `scope_id`, and `type`.

### memory/fts-query.ts
**Purpose:** Build FTS5-safe MATCH expressions from free-form user queries.

**Key exports:** `buildFtsQuery(raw: string): string | null`.

**Mechanism:**
1. Extract alphanumeric token runs via regex `/[\p{L}\p{N}_]+/gu` (includes CJK).
2. Wrap each token in phrase quotes: `"token"` (prevents FTS5 special-char crashes).
3. OR-join: `"token1" OR "token2" OR ...` (high recall; BM25 ranks rare tokens higher).
4. Returns `null` for empty queries.

**Design note:** AND would require all tokens present, failing on minor omissions. OR + score floor (`memory_search_score_floor`) trades noise for recall.

### history/index.ts
Barrel export; declares that `History`, `WriterService`, and `BackfillService` must be activated at bootstrap (not auto-started by Layer.merge).

### history/service.ts
**Purpose:** Search conversation history (session messages) using FTS5 with temporal + scope filtering.

**Key exports:** `Service` (search + around methods), default layer composition with writer + backfill.

**Mechanism:**
1. **Search:** Token-level FTS5 (same as memory), scoped to project or global; optional filters on session_id, kind (type of message), tool_name, time range. Returns `{part_id, session_id, message_id, project_id, kind, tool_name, snippet, score, time_created}`.
2. **Around:** Fetch contextual messages before/after a target message_id (e.g., 5 before, 3 after), including all parts of each message.
3. **Structure:** HistoryFtsTable stores message parts (session conversation snapshots), indexed on (session_id, time_created), (project_id, time_created), (message_id).

### history/fts.sql.ts
**Purpose:** Define Drizzle ORM table schema for conversation history.

**Key exports:** `HistoryFtsTable` — SQLite table.

**Schema:**
- `part_id`: primary key (unique part identifier)
- `session_id`, `message_id`, `project_id`: foreign references
- `kind`: part type (e.g., text, tool_use)
- `tool_name`: name of tool invoked (if applicable)
- `body`: part content (text or tool invocation)
- `time_created`: message timestamp
- Indexes on (session, time), (project, time), (message_id)

### history/writer.ts
**Purpose:** Listen to message bus events and index new conversation parts into HistoryFtsTable.

**Key exports:** `Service` (init method), layer.

**Mechanism:**
1. Subscribe to `MessageV2.Event.PartUpdated` and `PartRemoved` on the bus.
2. Enqueue upsert/delete jobs into an unbounded queue.
3. Async background worker processes queue: extract part content (via `extract.ts`), resolve references (via `resolve.ts`), upsert into HistoryFtsTable.
4. Fail open (log warning on errors) so transient DB issues don't trap the agent.

### history/extract.ts
(Not fully read) Extracts searchable content from message parts; defines `Kind` enum for part types.

### history/backfill.ts
(Not fully read) Scans existing message tables at startup and populates HistoryFtsTable.

### snapshot/index.ts
**Purpose:** Manage git-based file snapshots (point-in-time captures) for undo/diff/restore.

**Key exports:** `Service` (interface with track, patch, restore, revert, diff, diffFull), `Patch` (Zod schema).

**Mechanism:**
1. **Initialization:** Create a separate git directory (`.git-dir = {data}/snapshot/{projectID}/{hash(worktree)}`).
2. **Tracking:** `track()` stages all files (filtered by git ignore rules), writes a tree object, returns its hash. Called before major operations to capture state.
3. **Patching:** `patch(hash)` diffs current tree vs. a past snapshot; returns list of changed files.
4. **Restore:** `restore(snapshot)` reads a past tree and checks it out.
5. **Revert:** `revert(patches)` restores specific files from past snapshots (batches to 100 files per git command).
6. **Diff:** `diff(hash)` returns unified patch. `diffFull(from, to)` returns structured `FileDiff[]` with per-file diffs.

**Optimizations:**
- Semaphore locks per git-dir (per-project serialization).
- Batch checkout/revert to 100 files (avoid command line length limits).
- Filter ignored files from output (reconcile with `.gitignore` + project-level excludes).
- Cleanup loop (hourly, spawned forkScoped) runs `git gc --prune=7.days`.
- Large-file detection: files >2 MB are excluded from staging.

### storage/schema.ts
**Purpose:** Central export point for all Drizzle ORM table schemas.

**Key exports:** Re-exports all `*Table` definitions from subordinate modules.

### storage/storage.ts
**Purpose:** Abstract file-based persistence layer for JSON documents.

**Key exports:** `Service` (read, write, update, remove, list), `Interface`.

**Mechanism:**
1. **Keys:** Documents addressed by string array paths (e.g., `["project", "P123", "config"]` → `{dir}/project/P123/config.json`).
2. **Operations:**
   - `read<T>(key)`: Load JSON, parse via Effect.Schema, fail on ENOENT.
   - `write<T>(key, content)`: Serialize and write.
   - `update<T>(key, fn)`: Read, apply mutation, write back (CRUD atomicity).
   - `list(prefix)`: Glob all files under a prefix.
   - `remove(key)`: Delete file.
3. **Migrations:** Run a sequence of schema migrations at startup (versioned).
4. **Concurrency:** RcMap + TxReentrantLock per root for write serialization.

### project/project.sql.ts
**Purpose:** Drizzle ORM schema for project metadata.

**Key exports:** `ProjectTable`.

**Schema:**
- `id`: project ID (primary key)
- `worktree`: root directory path
- `vcs`: version control system ("git", etc.)
- `name`, `icon_url`, `icon_color`: display metadata
- `time_initialized`: bootstrap timestamp
- `sandboxes`: JSON array of sandbox worktree paths
- `commands`: JSON object with `start` command (optional)
- `created_at`, `updated_at`: timestamps

### project/schema.ts
**Purpose:** Effect.Schema and Zod type definitions for project IDs.

**Key exports:** `ProjectID` (branded string, with statics `global` and `zod`).

**Mechanism:** Brands a string to prevent accidental type mixing; provides a `.zod` export for Zod integration.

### account/account.sql.ts
**Purpose:** Drizzle ORM schemas for user authentication and account state.

**Key exports:** `AccountTable`, `AccountStateTable`, `ControlAccountTable` (legacy).

**Schemas:**
- `AccountTable`: OAuth tokens, account info
  - `id`: account ID (primary key)
  - `email`, `url`: provider info
  - `access_token`, `refresh_token`: branded strings
  - `token_expiry`: unix timestamp
- `AccountStateTable`: Current active account + org
  - `active_account_id`: foreign key to `AccountTable.id`
  - `active_org_id`: current organization (nullable)
- `ControlAccountTable`: Legacy; separate account storage (deprecated)

### account/schema.ts
**Purpose:** Effect.Schema type definitions for account, organization, and auth workflow.

**Key exports:** 
- Branded types: `AccountID`, `OrgID`, `AccessToken`, `RefreshToken`, `DeviceCode`, `UserCode`
- Classes: `Info` (account details), `Org` (org details), errors (`AccountRepoError`, `ServiceError`, `TransportError`)
- Poll results: `PollSuccess`, `PollPending`, `PollSlow`, `PollExpired`, `PollDenied`, `PollError`

### account/repo.ts
(Not fully read) Database operations for account CRUD.

### account/url.ts
(Not fully read) URL parsing for provider endpoints.

### task/schema.ts
**Purpose:** Zod schemas for task management (decomposition tree nodes).

**Key exports:** `TaskID`, `TaskStatus`, `Task`, `TaskEventKind`, `TaskEvent`.

**Schemas:**
- `Task`: `id` (Tn or Tn.m...), `session_id`, `parent_task_id`, `status`, `summary`, `owner`, timestamps, `cleanup_after`.
- `TaskEvent`: `id`, `task_id`, `at`, `kind` (created/started/blocked/done/abandoned/renamed), optional summary.

### task/gate.ts
**Purpose:** Stop-gate decision logic for task completion at agent exit.

**Key exports:** `decide(input: DecideInput): Effect.Effect<Decision>`.

**Mechanism:**
1. Query `TaskRegistry` for non-terminal tasks (open + in_progress, exclude blocked).
2. If list is empty: return `{ needReentry: false }` (allow stop).
3. If `reactCount >= maxReact`: return `{ capExceeded: true, incompleteTasks }` (prevent infinite loop).
4. Otherwise: return `{ needReentry: true, reentryText, incompleteTasks }` (inject nudge system-reminder).
5. **Mode semantics:** subagent gate lists tasks the agent owns; main gate lists all session tasks (safety net for orphaned work).

### task/registry.ts
(Not fully read) CRUD for task table; list/create/update/delete/transition operations.

### task/events.ts
(Not fully read) Task event emission and bus integration.

### task/gate-state.ts
(Not fully read) Persistent gate re-entry counter state.

### task/index.ts
Barrel export.

### worktree/index.ts
**Purpose:** Multi-worktree git management for isolated task sandboxes.

**Key exports:** `Service` (interface with makeWorktreeInfo, create, remove, reset, head, isPristine).

**Mechanism:**
1. **Creation:** Generate unique worktree names (slugified input + random suffix if needed), create via `git worktree add -b {branch}`, bootstrap (populate from main, run start scripts).
2. **Structure:** Each worktree is a separate checkout with its own branch (`mimocode/{name}`), managed in `{data}/worktree/{projectID}/`.
3. **Reset:** `git reset --hard {base}`, clean untracked files, update submodules, verify pristine (no local changes).
4. **Removal:** `git worktree remove --force`, delete branch, clean directory.
5. **Pristine check:** `git status --porcelain` must be empty and HEAD must match base.
6. **Integration:** Reads project start command from ProjectTable and executes it during bootstrap.

### git/index.ts
**Purpose:** Minimal git wrapper with Effect.ts async process spawning.

**Key exports:** `Service` (run, branch, prefix, defaultBranch, hasHead, mergeBase, show, status, diff, stats).

**Mechanism:**
1. **run():** Spawn `git {cfg-flags} {args}`, collect stdout/stderr, return `{ exitCode, text(), stdout, stderr }`.
2. **Config flags:** `--no-optional-locks`, `core.autocrlf=false`, `core.fsmonitor=false`, `core.longpaths=true`, `core.symlinks=true`, `core.quotepath=false`.
3. **Helpers:**
   - `branch(cwd)`: return current branch name
   - `defaultBranch(cwd)`: return `{ name, ref }` of default (main/master)
   - `status(cwd)`: return `{ file, code, status }` for working tree changes
   - `diff(cwd, ref)`: return `{ file, code, status }` for ref vs. HEAD
   - `stats(cwd, ref)`: return `{ file, additions, deletions }` for ref vs. HEAD
   - `show(cwd, ref, file, prefix?)`: retrieve file content at a ref
   - `mergeBase(cwd, base, head?)`: find common ancestor

---

## Portable Mechanisms for Rust Orchestrator

### M1: Two-Tier FTS Search with Score Thresholding
**File citations:** memory/service.ts:52-133, memory/fts-query.ts

**Design:** Decouple query parsing (token safety) from ranking (semantic relevance):
1. Tokenize user input into phrase-quoted literals, OR-join (high recall).
2. Execute FTS5 BM25 ranking.
3. Apply *relative* score floor: keep top hit + all results >= `floorRatio * topScore`. Relative (not absolute) because BM25 scores scale with corpus size; a fixed floor wrongly prunes small corpora.
4. Return results ordered by BM25 (lower score = better in FTS5; negate for user-facing API).

**Why steal it:** Prevents query crashes from special chars, avoids per-word thresholding drift, handles CJK and underscore naturally.

**ELAI applicability:** Memory-bank search (activeContext.md, changelog.md) already runs FTS but lacks the score floor. Import the `buildFtsQuery` logic + the relative thresholding strategy.

---

### M2: Content-Addressable Memory with Fingerprint-Based Reconciliation
**File citations:** memory/reconcile.ts:46-87, memory/fts.sql.ts

**Design:** Track on-disk memory files without polling or inotify:
1. Store a fingerprint `{size}-{mtimeMs}` per file in DB.
2. On reconcile, walk filesystem, compute fingerprints, compare to stored values.
3. Upsert only changed files (unchanged files skip I/O and index updates).
4. Dual discovery: local (MiMoCode) + remote (Claude Code projects under ~/.claude/projects/*/memory).

**Why steal it:** Zero inotify overhead, handles file moves (rehashing on access), cross-workspace indexing via path conventions.

**ELAI applicability:** Memory-bank changelog.md is currently regenerated on every tool call. Import fingerprint-based detection to skip reindex when unchanged.

---

### M3: Async Message Bus + Unbounded Queue Worker Pattern
**File citations:** history/writer.ts:22-71

**Design:** Decouple event publication from indexing:
1. Subscribe to bus events (e.g., PartUpdated, PartRemoved).
2. Enqueue jobs (upsert / delete) into unbounded queue.
3. Spawn async background worker (`Effect.forever` + `Queue.take`).
4. **Fail open:** catch errors and log; don't trap the bus or agent.
5. Use `subscribeCallback` (synchronous PubSub) not `Stream.subscribe` (lazy) to guarantee subscriptions live before init returns.

**Why steal it:** Decouples hot-path (agent loop) from cold-path (DB indexing); queue buffer absorbs burst writes; fail-open prevents transient DB errors from cascading.

**ELAI applicability:** Session journal indexing (workflow/sessions/{id}/orchestrate-run.md) could use this pattern instead of direct writes, improving latency on the agent loop.

---

### M4: Git-Backed File Snapshots with Atomic Tree Objects
**File citations:** snapshot/index.ts:86-726

**Design:** Leverage git's tree objects for point-in-time file captures:
1. Initialize a separate git directory (`{data}/snapshot/{projectID}/{hash(worktree)}`).
2. On snapshot: `git add` files (filtered by .gitignore), `git write-tree` to capture a tree hash, return hash.
3. On diff: `git diff --cached {hash}` returns changed files; `git show {hash}:{file}` retrieves content.
4. On restore: `git read-tree {hash}` + `git checkout-index -a -f`.
5. **Optimizations:** Semaphore locks per git-dir; batch operations to 100 files; 2 MB file size limit (exclude large binaries); hourly GC with 7-day prune.

**Why steal it:** Git tree objects provide content-addressable snapshots; no custom storage overhead. Batching avoids command-line length limits. Atomic writes (write-tree returns hash only when successful).

**ELAI applicability:** Crates/orchestrator/src/artifacts.rs (session snapshots) could use git trees instead of JSON snapshots, enabling fine-grained diff/restore per file.

---

### M5: Scope-Based Multi-Tenant Indexing with Hierarchical Filters
**File citations:** memory/fts.sql.ts, memory/service.ts:86-99, history/fts.sql.ts

**Design:** Enable scoped search without duplicate indexes:
1. Store `scope` + `scope_id` as regular columns (not FTS5 virtual index).
2. Define separate indexes on (scope, scope_id) and (type) for fast filtering.
3. At query time, apply WHERE clauses before FTS5 MATCH (e.g., `memory_fts.scope = ? AND memory_fts MATCH ?`).
4. For CC projects: scope="cc_project", scope_id="{slug}"; for mimo: scope="mimo", scope_id="".

**Why steal it:** Avoids schema explosion (no per-scope table), enables progressive filtering (narrow by scope, then full-text).

**ELAI applicability:** Handoff-protocol.md appends span multiple topics; indexed search with scope="handoff", scope_id="{topic-slug}" would enable topic-scoped history without separate tables.

---

### M6: Dual-Mode Transport (Memory + History) with Lazy Activation
**File citations:** history/index.ts, history/writer.ts:24

**Design:** Layer composition + explicit initialization:
1. Define a layer that merges the main service + worker services (WriterService, BackfillService).
2. **Do NOT auto-start workers** when the layer is provided. Require explicit `Service.init()` call at bootstrap.
3. This allows the harness to decide _when_ indexing starts (after config load, after FSM state restore, etc.).

**Why steal it:** Separates layer composition (late-bound dependency) from activation (eager startup); enables deferred initialization without nested Effect.gen recursion.

**ELAI applicability:** Currently, mcp_memorize is synchronous (blocks on write). Decouple into a write + async indexing loop using this pattern.

---

### M7: Stop-Gate Nudging with React Cap and Owner-Aware Task Filtering
**File citations:** task/gate.ts:80-101

**Design:** Prevent infinite stop-gate loops without hard-failing on incomplete work:
1. Query task registry for non-terminal tasks (open + in_progress; exclude blocked as unactionable).
2. If empty, allow stop. If count >= `maxReact`, cap exceeded (return error + list). Else, nudge (inject system-reminder with task list).
3. **Subagent vs. main semantics:** Subagent's nudge is "you own these"; main's nudge is "session still has these" (catches orphaned work from subagents).
4. Re-entry cap is lower for subagents (2) than main (3) because main sessions are longer-lived.

**Why steal it:** Avoids infinite nudging (cap prevents loops) while maintaining visibility (nudge still surfaces incomplete work). Owner-aware filtering prevents false positives (subagent doesn't own main's tasks).

**ELAI applicability:** Orchestrator FSM gate (orchestrate skill step 8.5 EXECUTING -> 8.6 AUDITING) could use this pattern to catch incomplete baby-steps before declaring a phase done.

---

### M8: Worktree Name Generation with Collision Avoidance
**File citations:** worktree/index.ts:201-231

**Design:** Generate unique sandboxed environment names without distributed ID service:
1. User-provided base (optional): slugify input, try 26 attempts with random suffixes.
2. On each attempt, check filesystem + git branch namespace for collisions.
3. Return first collision-free candidate (name + branch + directory).
4. **Slug function:** lowercase, alphanumeric + dash, strip leading/trailing dashes.

**Why steal it:** Pure deterministic generation (no external service), handles user intent (prefer user-provided base), bounded loop (fail after 26 attempts, not infinite).

**ELAI applicability:** Worktree sandbox naming already uses this. For session ID generation (orchestrate-run.md), import the collision-avoidance loop logic (check existing sessions before assigning).

---

## Cross-File Dependencies

- **Memory + History:** Both use `buildFtsQuery` (token normalization), follow the same scoring pattern (BM25 + relative floor).
- **Snapshot + Git:** Snapshot wraps git; git service is a reusable abstraction over `child_process` spawning.
- **Storage + Project/Account:** Both use JSON persistence; storage is the abstract layer, concrete tables are defined in schema files.
- **Task + Worktree:** Task gate can nudge about incomplete worktree setup/cleanup tasks.
- **All:** Connect via Effect layers; no circular dependencies; InstanceState provides per-project context binding.

---

## Summary

MiMoCode Part 7 implements a full-stack agent memory + conversation history + file snapshot + project/account lifecycle + task decomposition + multi-worktree orchestration system. The standout mechanisms are:

1. **Two-tier FTS** with relative scoring (recall-optimized, user-query-safe).
2. **Content-addressable snapshots** via git trees (no custom storage overhead).
3. **Async message bus** worker pattern (fail-open, burst-tolerant).
4. **Fingerprint-based reconciliation** (zero-polling, cross-workspace discovery).
5. **Scope-based multi-tenancy** (progressive filtering, no schema explosion).
6. **Stop-gate nudging** (prevents loops, owner-aware filtering).

All exportable to Rust via careful Effect.ts → Tokio translation; see PORTABLE MECHANISMS M1–M8 for per-mechanism citations and ELAI adoption paths.
