Source: MiMoCode (packages/opencode)
Reliability: MEDIUM (reviewed TypeScript source, Bun/Effect monorepo, no decompilation)
Files examined:
  packages/opencode/src/memory/service.ts
  packages/opencode/src/memory/reconcile.ts
  packages/opencode/src/memory/paths.ts
  packages/opencode/src/memory/fts-query.ts
  packages/opencode/src/session/checkpoint.ts
  packages/opencode/src/session/auto-dream.ts
  packages/opencode/src/session/compaction.ts
  packages/opencode/src/session/overflow.ts
  packages/opencode/src/session/budgeted-read.ts
  packages/opencode/src/session/checkpoint-paths.ts
  packages/opencode/src/session/checkpoint-templates.ts
  packages/opencode/src/actor/spawn.ts
  packages/opencode/src/actor/registry.ts
  packages/opencode/src/skill/index.ts
  packages/opencode/src/skill/discovery.ts
  packages/opencode/src/session/goal.ts
  packages/opencode/src/session/instruction.ts
  packages/opencode/src/task/registry.ts
  packages/opencode/src/snapshot/index.ts
  packages/opencode/AGENTS.md
  README.md

---

OVERVIEW

MiMoCode is a Bun/TypeScript terminal coding agent forked from opencode. Its defining
additions over the upstream are: (1) a cross-session SQLite FTS5 memory layer,
(2) a checkpoint-writer subagent that snapshots session state on-demand,
(3) context rebuild/compaction that reconstitutes a trimmed window from checkpoint
artifacts, (4) auto-dream/distill for sleep-time knowledge consolidation and
skill extraction, and (5) a goal/judge system for autonomous stop-condition
evaluation.

Runtime model: Effect.ts (ZIO-style) functional effects. Every service is a
Context.Service singleton injected via Layer. All I/O is async Bun (Bun.file,
Bun.write). Database is Drizzle/libSQL (SQLite, Bun native). Build: bun + turbo.

---

Public API:

Memory.Service | @opencode/Memory | Effect-based service
  root() | () -> Effect<string> | returns absolute path to memory root dir
  reconcile() | () -> Effect<{indexed:number,pruned:number}> | walks disk+db, upserts/prunes FTS rows
  search(input) | {query,scope?,scope_id?,type?,limit?} -> Effect<SearchRow[]> | BM25 FTS5 search with relative score floor

SessionCheckpoint.Service | @opencode/SessionCheckpoint | Effect-based service
  tryStartCheckpointWriter(input) | TryStartCheckpointWriterInput -> Effect<"started"|"queued"|"skipped"> | forks writer subagent; 1-slot pending queue (F40); returns immediately
  waitForWriter(sessionID) | SessionID -> Effect<"success"|"failure"|"no-writer"> | blocks up to 300s for in-flight writer
  drainWriters(opts?) | {timeoutMs?} -> Effect<{drained,timedOut}> | wait for all writers on shutdown (default 120s timeout)
  hasCheckpoint(sessionID) | SessionID -> Effect<boolean> | file-existence check on checkpoint.md
  hasMemoryOrTasks(sessionID) | SessionID -> Effect<boolean> | checks memory dir entries + task DB rows
  loadLatest(sessionID) | SessionID -> Effect<string|undefined> | reads single checkpoint.md
  loadCheckpoints(sessionID,count) | SessionID,number -> Effect<string[]> | reads checkpoint.md (v5 single-file, count ignored)
  renderIndex(sessionID) | SessionID -> Effect<string> | human-readable index from checkpoint topic line
  renderRebuildContext(sessionID,opts?) | SessionID,{lastMessageInfo?,agentID?} -> Effect<string> | full rebuild injection: tasks ledger + checkpoint + project memory + global memory + session notes + memory keys index + seam framing + tail reminder
  lastBoundary(sessionID) | SessionID -> Effect<MessageID|undefined> | reads last_checkpoint_message_id from session DB row
  isWriterRunning(sessionID) | SessionID -> Effect<boolean> | checks in-memory writers Map
  insertRebuildBoundary(input) | {sessionID,boundary,lastMessageInfo?,agentID?,agent,model,boundaryCreatedAt?} -> Effect<boolean> | inserts synthetic user message with rebuild context + microcompacts tail tool results

Skill.Service | @opencode/Skill | Effect-based service
  get(name) | string -> Effect<Info|undefined> | lookup by skill name
  all() | () -> Effect<Info[]> | all loaded skills
  dirs() | () -> Effect<string[]> | directories where skills were found
  available(agent?) | Agent.Info? -> Effect<Info[]> | permission-filtered, sorted list

ActorRegistry.Service | @opencode/ActorRegistry | Effect-based service
  register(input) | Actor registration params -> Effect<Actor> | records actor in DB
  updateStatus | SessionID,actorID,{status,lastOutcome?,lastError?} -> Effect<void>
  get | SessionID,actorID -> Effect<Actor|undefined>
  listActive | () -> Effect<Actor[]>
  listByParent | SessionID,parentActorID -> Effect<Actor[]>
  renderForAgent | SessionID -> Effect<string> | one-line-per-actor summary for context injection
  isSystemSpawned | SessionID,actorID -> Effect<boolean>

AutoDream (module, not a Service)
  shouldAutoDream(cfg) | Config.Info -> Effect<boolean> | checks DB for last session titled "Auto Dream"; respects interval_days (default 7d) + project-age gate + 10s spawn gap
  shouldAutoDistill(cfg) | Config.Info -> Effect<boolean> | same pattern; interval_days default 30d

computeBoundary (exported fn)
  signature: (msgs: ReadonlyArray<{info:{id,role,finish?},parts[]}>) -> string
  returns: MessageID of first message to preserve (boundary); everything before is summarized

---

Behavioral contracts:

Memory.reconcile -> walks memory root dir recursively for .md files; for each file computes size+mtimeMs fingerprint; inserts/updates FTS row only on fingerprint change; prunes FTS rows whose disk path no longer exists. Side effect: writes to SQLite memory_fts + memory_fts_idx tables.

Memory.search -> lazy reconcile on each search call (unless memory_reconcile_on_search=false); builds FTS5 OR-query (phrase-quoted tokens); over-fetches 3x capped at 50; applies relative score floor (default 0.15 of top hit); returns up to limit results with score higher-is-better (FTS5 bm25 negated).

SessionCheckpoint.tryStartCheckpointWriter -> skips if session has zero messages; skips if system-spawned; computes token-budgeted boundary via computeBoundary + adjustBoundaryForApiInvariants; for fork:false (default) precomputes delta since last checkpoint and skips if empty; spawns writer as a child session via actor.spawn(mode="subagent", background=true); advances last_checkpoint_message_id in SessionTable on writer settle; 1-slot pending queue: newer request evicts older pending (newest is always superset). Writer timeout: 300s. Side effect: creates child session row, writes memory files via writer subagent.

SessionCheckpoint.renderRebuildContext -> waits up to 60s for in-flight writer before reading; reads checkpoint.md (cap: 11K tokens), MEMORY.md (cap: 10K), notes.md (cap: 6K), global MEMORY.md (cap: 6K); reads task DB; lazy reconciles FTS; assembles multi-section injection block ending with seam framing + tail reminder; returns "" for non-"main" agentIDs (subagents do not get rebuild context).

SessionCheckpoint.insertRebuildBoundary -> inserts synthetic user message in DB; microcompacts tool results for tools in COMPACTABLE_TOOL_NAMES (read/bash/grep/glob/webfetch/websearch/edit/write/multiedit/apply_patch/codesearch) that are strictly newer than boundary timestamp; never deletes DB messages.

SessionCompaction.isOverflow -> true when token count >= usable(cfg, model); usable = model.limit.input - reserved, where reserved = min(COMPACTION_BUFFER=20K, maxOutputTokens); output also capped at 20K. If compaction.auto=false or context=0, always false.

SessionCompaction.prune -> erases output of older tool call results to free context space; protects last 40K tokens (PRUNE_PROTECT) of tool calls; only "skill" tool results are never erased (PRUNE_PROTECTED_TOOLS). Requires compaction.prune=true in config.

computeBoundary algorithm: starts at lastAsstIdx-1; if tail >= TAIL_MAX_TOKENS (20K) returns as-is (soft ceiling, no truncation); if tail < TAIL_MIN_TOKENS (10K) or text-block messages < 5, walks backward until both floors met or TAIL_MAX_TOKENS hit; returns MessageID of boundary.

shouldAutoDream / shouldAutoDistill -> gate on: (1) enabled flag, (2) 10s spawn gap guard (module-level lastSpawnTime), (3) DB check for last session with title "Auto Dream"/"Auto Distill", (4) project age must exceed interval on first run. If interval not elapsed since last run: skip. Distill task prompt instructs agent to read DB + memory files and produce new skill assets; dream prompt consolidates durable facts into MEMORY.md.

Actor spawn (mode="subagent") -> creates child session; MAX_PRE_REACT=3 and MAX_POST_REACT=3 cap re-entries in preStop/postStop hooks; final message must start with "**Status**: success|partial|failed|blocked" header; writer writes checkpoint.md/MEMORY.md at parent session's paths even though it runs in a child session.

Skill discovery -> scans in priority order: (1) compose bundle (embedded in package, extracted to disk), (2) global external dirs (~/.claude, ~/.agents, ~/.codex, ~/.opencode) for skills/**/SKILL.md, (3) up-dirs from cwd matching same pattern, (4) config.directories(), (5) cfg.skills.paths[], (6) cfg.skills.urls[] (remote pull with HTTP cache). Duplicate skill names: last-writer-wins with a warn log.

---

Memory file layout on disk:

<data>/memory/global/MEMORY.md          -- global (cross-project) user preferences
<data>/memory/projects/<pid>/MEMORY.md  -- per-project durable knowledge (pid = sha256[:12] of repo abs path)
<data>/memory/sessions/<sid>/checkpoint.md  -- per-session single checkpoint (v5)
<data>/memory/sessions/<sid>/notes.md   -- per-session agent scratchpad
<data>/memory/sessions/<sid>/tasks/<tid>/progress.md  -- per-task journal

FTS index scopes: "global" | "projects" | "sessions" | "cc"
Memory types: "free" | "memory" | "checkpoint" | "progress" | "notes" | "feedback" | "project" | "reference" | "user"
CC types (Claude Code import): "feedback" | "project" | "reference" | "user" -- derived from YAML frontmatter metadata.type

Checkpoint template sections (11 sections, total ~11K token budget):
  §1 Active intent (500 tok) | §2 Next concrete action (1000) | §3 Directives (800)
  §4 Task tree (1000) | §5 Current work (2000) | §6 Files and code sections (1500)
  §7 Discovered knowledge (2000) | §8 Errors and fixes (1500) | §9 Live resources (1000)
  §10 Design decisions (3000) | §11 Open notes (800)

MEMORY.md sections: Project context (1000) | Rules (2000) | Architecture decisions (3000) | Discovered durable knowledge (4000)

---

Platform constraints:

Requires Bun runtime (Bun.file, Bun.write, bun.lock); not compatible with Node without shims.
SQLite (Drizzle/libSQL) is the only storage backend; all FTS is SQLite FTS5. No external vector DB.
Claude Code memory import: walks ~/.claude/projects/<slug>/memory/**/*.md; parses YAML frontmatter for type; opt-in via cfg.memory.cc_index = true.
Memory file path guards: assertSafeComponent rejects ".." segments and "/" prefix to prevent path traversal from caller-supplied scope_id/key.
Writer subagent tool whitelist: only [read, write, edit, apply_patch, glob, grep, task] — no bash, no websearch, no MCP tools.
Checkpoint fork mode: fork:false (default, cold-start, delta-only). fork:true preserves parent prefix-cache context (larger but cheaper on Anthropic prompt-cache providers).
FTS5 tokenizer: unicode61 via SQLite default. Query builder uses \p{L}\p{N}_ regex to extract tokens, phrase-wraps each, OR-joins. CJK letters included.
Memory reconcile is idempotent (fingerprint = size + mtimeMs). Concurrent sessions calling reconcile simultaneously will both read the disk and upsert the same rows without corruption (SQLite serializes writes).
Auto-dream/distill checks use a module-level process-singleton guard (lastDreamSpawnTime); restarting the process resets this.
Compaction pressureLevel thresholds: 0 (<50%), 1 (50-70%), 2 (70-85%), 3 (>=85%) of usable window.
COMPACTION_BUFFER = 20K tokens reserved headroom. OUTPUT_CAP = 20K hard cap on output reservation.
Goal judge: separate model call using generateObject/streamObject from the "ai" SDK; reads transcript, returns {ok, impossible, reason}. MAX_GOAL_REACT in prompt.ts caps re-entry loops.

---

Gotchas:

renderRebuildContext skips entirely for agentID !== "main" — subagents never get rebuild context, including writers; if agentID is missing (undefined) the check passes (undefined !== "main" is truthy — undefined passes through, see line 1021: `if (opts?.agentID && opts.agentID !== "main") return ""`).

Writer spawns a child session to isolate messages, but writes to PARENT session's file paths (checkpointPath(input.sessionID) not child.id). A bug that passes child.id to checkpointPath would silently produce an empty checkpoint.

FTS5 bm25() returns negative values (lower = better). Service inverts to higher-is-better. Any caller that compares raw bm25 output to a positive floor threshold will get inverted results.

computeBoundary TAIL_MAX_TOKENS is a soft ceiling: if a single turn legitimately exceeds 20K tokens, the tail will be >20K and that is by design. The check only gates backward expansion, not forward truncation.

Memory reconcile prunes paths not in EITHER walk; adding cc_index=true on an existing installation will NOT prune mimo rows (both walks combined before pruning). Disabling cc_index later WILL prune cc-indexed rows on next reconcile.

Skill name collision: if two SKILL.md files declare the same name field, second one wins silently (log warn only, no error surfaced to user).

Auto-dream interval check uses the session title "Auto Dream" (exact string). Any session titled identically by the user would reset the interval clock.

readBudgeted truncation uses a 0.95 safety factor: `Math.floor(fullText.length * ratio * 0.95)`. This means the effective budget is 5% smaller than configured, which can cause unexpected truncation of files close to the budget size.

Writer pending queue: if two checkpoint requests arrive while writer1 is running, only the LAST pending is kept (newest evicts older). A burst of 3+ checkpoint triggers will silently discard all but the most recent pending.

---

Critical assertions (testable):

# Memory search score floor: top hit always returned
results = memory_service.search(query="test", limit=5)
assert len(results) >= 1 or len(all_matching_fts_rows) == 0

# BM25 score inversion: all returned scores must be positive (service negates)
for r in results:
    assert r["score"] > 0, "service must negate bm25() which returns negative values"

# Checkpoint boundary never splits tool_use/tool_result pair
# (adjustBoundaryForApiInvariants guarantee)
boundary_idx = msgs.index(boundary_msg)
assert msgs[boundary_idx].role != "tool_result" or boundary_idx == 0

# Rebuild context excluded for subagents
rebuild = checkpoint_service.renderRebuildContext(session_id, agentID="some-subagent-id")
assert rebuild == ""

# Microcompact only clears tools newer than boundary (fail-closed: unknown timestamp = skip not wipe)
for part in session_messages_before_boundary:
    if part.type == "tool" and part.tool in COMPACTABLE_TOOL_NAMES:
        assert not part.state.time.compacted, "pre-boundary tool results must not be cleared"

# FTS path traversal guard
try:
    buildPath(root="/data/memory", scope="sessions", scope_id="../../etc/passwd", key="test")
    assert False, "must raise on path traversal"
except Exception:
    pass
