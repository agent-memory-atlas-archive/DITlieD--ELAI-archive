# MiMoCode Session Partition (Part 2) — Comprehensive Analysis

## Overview

The `packages/opencode/src/session/` partition (40 TypeScript files, ~150K tokens) is the core orchestration and context-management layer of MiMoCode's agent-loop infrastructure. It handles session lifecycle, message protocol, context lifecycle (checkpoint/compaction), autonomous loop control, and the LLM request/response cycle.

---

## Per-File Analysis

### Core Session Architecture

#### **session.ts** (500+ lines)
**Purpose:** Session metadata, CRUD, and lifecycle management.
**Mechanism:** Database-backed session records (SessionTable) with slug, directory, parent-child hierarchy, version tracking, and optional contextFrom/contextWatermark pointers. Exports `fromRow()` / `toRow()` for serialization. Defines the `Info` and `Snapshot` types.
**Key constants:** parentTitlePrefix, childTitlePrefix, default title generation.
**Cross-refs:** Storage (database), ProjectID, SessionID schema, MessageV2.

#### **index.ts** (1 line)
**Purpose:** Public export barrel.
**Mechanism:** Re-exports session module as `Session`.

#### **schema.ts** (implied, not read)
**Purpose:** SessionID, MessageID, PartID branded types.
**Mechanism:** Zod schema definitions for strong typing.

#### **session.sql.ts** (implied, not read)
**Purpose:** Drizzle ORM table definitions (SessionTable, MessageTable, PartTable).
**Mechanism:** Database schema codegen from TypeScript types.

---

### Message Protocol & Rendering

#### **message-v2.ts** (300+ lines)
**Purpose:** Polymorphic message part system for assistant/user/system roles.
**Mechanism:** Discriminated-union types: TextPart, ReasoningPart, ToolPart, SnapshotPart, PatchPart, CompactionPart, etc. Error shape definitions (OutputLengthError, ContextOverflowError, APIError, StructuredOutputError, etc.). Converts native AI SDK ModelMessage to MiMo MessageV2 for persistence and reconstruction.
**Key exports:** OutputFormat (text | json_schema), MessageV2.{User, Assistant, WithParts}, part discriminators.
**Notable:** `convertToModelMessages()` (from ai SDK) integrates with Anthropic API message reconstruction.
**Cross-refs:** Storage, LanguageModelUsage from ai SDK, Snapshot, Sync events, LSP.

#### **message.ts** (implied, supplementary)
**Purpose:** Legacy or complementary message handling.
**Mechanism:** Unknown (not read).

#### **claude-import.ts** (200+ lines)
**Purpose:** Migrate Claude Code .claude-conversation JSON export format into MiMoCode.
**Mechanism:** `parse()` reconstructs session from JSONL entries (type: user|assistant, timestamp, cwd, version, message content). Splits model string (provider/model), recovers tool_use/tool_result pairing, builds MessageV2 parts from raw content blocks.
**Notable:** `tokensFrom()` extracts token counts from usage metadata; `stringifyToolContent()` handles union tool results.
**Cross-refs:** MessageV2, SessionID, ProjectID, ProviderID/ModelID.

#### **claude-import.sql.ts** (implied, schema def)
**Purpose:** ClaudeImportTable schema for deduplication/audit.

---

### Context Management & Compaction

#### **checkpoint.ts** (500+ lines)
**Purpose:** Rebuild-time checkpoint writing (subagent spawn, template init, message boundary decisions).
**Mechanism:** On demand or at rebuild entry, spawns the `checkpoint-writer` subagent with injected context (MessageV2.WithParts[], prior discoveries, memory state). Ensures checkpoint template exists (section §1–§11). Reads budgeted file content (checkpoint.md, memory.md). Manages tail preservation (TAIL_MIN_TOKENS=10K, TAIL_MAX_TOKENS=20K), rebuild-time microcompact. Calls `adjustBoundaryForApiInvariants()` to preserve tool_use/tool_result pairing and thinking-block message-id alignment.
**Key constants:** TAIL_MIN_TOKENS=10_000, TAIL_MAX_TOKENS=20_000, TAIL_MIN_TEXT_BLOCK_MESSAGES=5.
**Reminders:** `autonomousLoopReminder()`, `stopReminder()`, `toolResultContinueReminder()` injected to guide agent behavior across rebuilds.
**Cross-refs:** MessageV2, checkpoint-paths, budgeted-read, boundary, checkpoint-align, checkpoint-templates, checkpoint-validator, goal, last-message-info, Memory, Bus, Storage, Actor registry, spawnRef.

#### **compaction.ts** (300+ lines)
**Purpose:** Overflow detection, automatic pruning, and compaction triggering.
**Mechanism:** `isOverflow()` checks if token count exceeds usable window (context - outputReserve - compactionBuffer). `process()` spawns compaction agent or returns "continue"/"stop"/"overflow". `usable()` calculates: max(0, input - reserved) where reserved = min(COMPACTION_BUFFER, maxOutputTokens). Prune rules: PRUNE_MINIMUM=20K, PRUNE_PROTECT=40K, protected tools=["skill"], DEFAULT_TAIL_TURNS=2, MIN_PRESERVE=2K, MAX_PRESERVE=8K.
**Mechanism detail:** Turns are reconstructed from message boundaries (user message = turn start, next user = turn end). Recent budget defaults to 25% of usable if unconfigured. Pressure level (0–3) scales linearly 50%/70%/85% of limit.
**Cross-refs:** Config, Provider, MessageV2, Session, Agent, Plugin, SessionProcessor, overflow module, Bus.

#### **compaction-overflow.ts** (aka overflow.ts, 54 lines)
**Purpose:** Simple computed properties for context pressure.
**Mechanism:** `usable()` = context - outputReserve - reserved. `isOverflow()` boolean check. `pressureLevel()` returns 0|1|2|3 based on token ratio.
**Constants:** COMPACTION_BUFFER=20K, OUTPUT_CAP=20K.
**Cross-refs:** Config, Provider.Model.

#### **boundary.ts** (77 lines)
**Purpose:** Adjust compaction/summary boundary to respect API invariants.
**Mechanism:** Walk backward from candidate boundary to find tool_use/tool_result pairs (orphaned tool_results = bail backward to include their tool_use). Then walk backward to include sibling messages sharing message.id (thinking blocks). Pure function.
**Algorithm:** Collect tail tool_results/uses, find orphans, backtrack to include matching uses, then backtrack for shared message.id siblings.
**Cross-refs:** None (pure utility).

#### **checkpoint-align.ts** (implied, not read)
**Purpose:** Align compaction/checkpoint boundary to non-tool-result/user message pair.
**Mechanism:** Unknown.

#### **checkpoint-paths.ts** (87 lines)
**Purpose:** Path helpers for memory hierarchy.
**Mechanism:** `metaDir(sessionID)` = `<data>/memory/sessions/<sid>`, `checkpointPath()` = `<sid>/checkpoint.md` (v5 single-file), `memoryPath(projectID)` = `<data>/memory/projects/<pid>/MEMORY.md`, `globalMemoryPath()` = `<data>/memory/global/MEMORY.md`, `progressPath(sessionID, taskID)` = `<sid>/tasks/<TID>/progress.md`. Also: `migrateProjectMemory()` one-shot rename of lowercase memory.md to MEMORY.md (atomic, idempotent).
**Cross-refs:** Global.Path.

#### **checkpoint-context.ts** (37 lines)
**Purpose:** Transient in-memory store for checkpoint writer context (priorTitles, expectedRevisions).
**Mechanism:** Map<key, CheckpointContext> where key = `${sessionID}:${actorID}`. Set/get/remove/clear for test-only.
**Use case:** Avoid re-parsing prior checkpoint.md on each writer invocation.

#### **checkpoint-templates.ts** (80 lines)
**Purpose:** Markdown template strings for checkpoint.md and MEMORY.md.
**Mechanism:** CHECKPOINT_TEMPLATE has 11 sections (§1 Active intent, §2 Next action, §3 Directives, §4 Task tree, §5 Current work, §6 Files, §7 Discoveries, §8 Errors, §9 Live resources, §10 Design decisions, §11 Open notes). MEMORY_TEMPLATE has project context, rules, architecture decisions, discovered knowledge. CHECKPOINT_SECTION_BUDGETS (not shown, implied) define per-section token limits.
**Cross-refs:** None (pure data).

#### **checkpoint-validator.ts** (implied, not read)
**Purpose:** Validate checkpoint/memory markdown for completeness, duplication, section structure.
**Mechanism:** Unknown, but used by checkpoint-retry to validate writer output before acceptance.

#### **checkpoint-retry.ts** (70+ lines)
**Purpose:** Post-write validation and deduplication guard for checkpoint writer.
**Mechanism:** `loadPriorDiscoveredTitles()` parses checkpoint.md to extract prior §7 entries (dedup guard). `runValidatorsForCkpt()` runs validators (snapshot, learning, budget, revisions). Detects violations (role, severity, detail).
**Use case:** Prevents checkpoint writer from duplicating discoveries, violating section budgets, or creating incomplete snapshots.
**Cross-refs:** checkpoint-paths, checkpoint-validator, checkpoint-templates.

#### **checkpoint-progress-reconcile.ts** (100 lines)
**Purpose:** Integrate per-task progress.md into main checkpoint.md.
**Mechanism:** `buildProgressDiffItems()` scans tasks/<TID>/progress.md, compares written-at timestamp vs. prior reconcile mark in main checkpoint. Returns NEW / CHANGED items. `renderProgressDiffBlock()` formats markdown injection block for checkpoint writer with instructions (Read each task, integrate §4 verbatim into §5, update reconcile marker).
**Algorithm:** Negative-lookbehind regex to avoid matching marker inside backtick code span.
**Cross-refs:** checkpoint-paths.

---

### Session Loop Control

#### **processor.ts** (300+ lines)
**Purpose:** Main session loop orchestrator: handle LLM responses, classify outcomes, execute tool calls, manage overflow.
**Mechanism:** `SessionProcessor.Handle` wraps message state with `updateToolCall()` / `completeToolCall()` / `process()`. `Result` = "overflow" | "stop" | "continue". `ProposedToolCall` (from max-mode) captures parsed tool args before execution. `ReplayInput` reconstructs winning candidate from max-mode ensemble (includes reasoning, text, tool calls, selection metadata, overhead cost).
**Classification:** Calls `classifyAssistantStep()` to determine terminal vs. continue.
**Doom-loop:** DOOM_LOOP_THRESHOLD=3 repeated failures → hard error.
**Cross-refs:** LLM, MessageV2, Agent, Bus, Config, Permission, Plugin, Snapshot, SessionStatus, SessionRetry, SessionSummary, Question, isOverflow check.

#### **classify.ts** (92 lines)
**Purpose:** Pure classifier for assistant step terminal-ness.
**Mechanism:** `classifyAssistantStep()` checks (in order):
  1. Pending non-providerExecuted client tool part (re-loop core guarantee).
  2. Finish reason (if not set, continue).
  3. Provider-executed-only tool step (tool-calls → continue).
  4. Stale assistant (predates user turn → skip).
  5. Errored step (failed).
  6. Structured / summary (final).
  7. Safety finish (content-filter, error).
  8. Stop / length / other → check text/reasoning content; "other" finish with text → degraded.
  9. Empty output → invalid.
**Return types:** final | continue | filtered | think-only | invalid | failed.
**Cross-refs:** MessageV2.

#### **llm.ts** (300+ lines)
**Purpose:** LLM request/response streaming and retry logic.
**Mechanism:** `streamText()` from ai SDK with provider wrapping. Persistent retry schedule: exponential backoff 500ms × 2, max 5min per delay, 10 attempts. Transient error detector (`isRetryableTransientError()`): 429/5xx/529 HTTP, ECONNRESET/EPIPE/ETIMEDOUT, "SSE read timed out". Auth/client errors not retried. Builds memory-system instructions appended to system prompt (MEMORY.md governance, Active recall protocol, subagent return contract).
**Constants:** OUTPUT_TOKEN_MAX, persistentRetrySchedule = exponential with modifyDelay cap.
**Reminders:** Memory-system instructions inject governance over checkpoint.md write permissions (writer exclusive), task progress integration, subagent return format contract.
**Cross-refs:** Provider, Config, InstanceState, Auth, SystemPrompt, Memory, ActorRegistry, Plugin, Flag, Bus, Permission, Gateway, Installation.

#### **goal.ts** (200+ lines)
**Purpose:** Per-session stop-condition judge (autonomous loop goal gating).
**Mechanism:** User sets `/goal: condition`. Main loop reads goal.get(sessionID). Before stopping, calls goal.evaluate(condition, messages, model) with a separate judge model. Judge sees system prompt (evaluate stop-condition logic) + conversation transcript + closing question about condition. Returns Verdict: {ok: boolean, impossible?: boolean, reason: string}. Broadcasts Goal.Updated event on verdict. Re-entry counter bumped until MAX_GOAL_REACT (higher than spawned actors' 3, to allow longer main-session efforts).
**Judge logic:** Independent confirmation (not trusting assistant self-report), impossible only when genuinely unachievable (self-contradictory, unavailable resource, exhausted), not just slow.
**Cross-refs:** InstanceState, Provider, Auth, Config, Bus, MessageV2, SessionID, generateObject from ai SDK.

#### **max-mode.ts** (200+ lines)
**Purpose:** Ensemble mode: N parallel LLM candidates + judge selection.
**Mechanism:** `DEFAULT_CANDIDATES=5`. `toSchemaOnlyTools()` strips execute closure for propose-only streaming. Each candidate collected into `Candidate` type (index, reasoning, text, toolCalls, finishReason, usage). Judge selects winner (0-based index). `ReplayInput` reconstructs winner with selection metadata and overhead cost (losing candidates + judge not charged to message tokens, only cost).
**Mechanism detail:** Candidates run propose-only (no tool execution), judge model picks best by reasoning, winner replayed with full tool execution.
**Cross-refs:** LLM, SessionProcessor, Session, Provider, Agent, MessageV2, Permission.

#### **retry.ts** (implied, not read)
**Purpose:** Transient error detection and retry strategies.
**Mechanism:** `isRetryableTransientError()` (exported by llm.ts), retry circuit, doom-loop breaking.

#### **prune.ts** (implied, not read)
**Purpose:** Aggressive message pruning when compaction is insufficient.

#### **revert.ts** (implied, not read)
**Purpose:** Undo recent assistant turns or tool states.

#### **summary.ts** (80+ lines)
**Purpose:** Compute and cache session git diff (file additions/deletions/modifications).
**Mechanism:** `summarize()` triggers snapshot diff compute. `diff()` reads from Snapshot service, caches per messageID. `computeDiff()` walks MessageV2.WithParts for SnapshotPart and PatchPart, diffs against prior snapshots.
**Tool utility:** `unquoteGitPath()` handles octal escapes in git quoted paths.
**Cross-refs:** Session, MessageV2, SessionID, Bus, Snapshot, Storage.

#### **status.ts** (80+ lines)
**Purpose:** Per-session run status tracking (idle/busy/retry).
**Mechanism:** `Info` union type: {type: "idle"} | {type: "busy", message?: string} | {type: "retry", attempt: number, message: string, next: number}. Broadcasts Status event on set. State stored in InstanceState map.
**Cross-refs:** Bus, BusEvent, InstanceState, SessionID.

#### **run-state.ts** (100+ lines)
**Purpose:** Per-session / per-agent runner management (concurrent work coordination).
**Mechanism:** InstanceState-backed map of runners (key = `${sessionID}:${agentID}`). Runner is a cancellable Effect.Effect container. Main runner isMain=true, status=idle/busy broadcast. Subagent runners auto-clean on idle. `ensureRunning()` starts work or returns error if busy. `startShell()` separate entry for shell mode.
**Cross-refs:** SessionStatus, InstanceState, Runner, MessageV2, Session.BusyError.

#### **instruction.ts** (implied, not read)
**Purpose:** Session-level instruction injection.

#### **system.ts** (80+ lines)
**Purpose:** Model-conditional system prompt selection.
**Mechanism:** `provider(model)` selects prompt variant by model name (gpt-4/o1/o3 → BEAST, gpt → GPT, gemini → GEMINI, claude → ANTHROPIC, trinity → TRINITY, kimi → KIMI, default → DEFAULT). Also returns CODEX for gpt-codex. Interface also exposes `skills()` which composes available skills into system block.
**Prompt files:** /prompt/{anthropic,beast,default,gemini,gpt,kimi,codex,trinity,compose,build-switch,max-steps}.txt.
**Cross-refs:** Provider, Skill, Agent, Permission, Instance.

---

### Session Loop & Automation

#### **auto-dream.ts** (110+ lines)
**Purpose:** Sleep-time session consolidation (dream) and pattern extraction (distill).
**Mechanism:** Interval-based triggers: DEFAULT_DREAM_INTERVAL_DAYS=7, DEFAULT_DISTILL_INTERVAL_DAYS=30. `shouldAutoDream()` / `shouldAutoDistill()` query Database for prior Auto Dream/Distill sessions, check elapsed time, check project age (no run on brand-new projects). Spawn limit: MIN_SPAWN_GAP_MS=10s between spawns to prevent duplicate triggers.
**Task specs:** DREAM_TASK (read memory + DB, consolidate durable facts) and DISTILL_TASK (review past month, identify repeated workflows, package missing skills).
**Cross-refs:** Config, SessionTable, Database.

#### **run-state.ts** (covered above)

#### **prompt.ts** (500+ lines)
**Purpose:** Session prompt assembly, tool resolution, artifact injection.
**Mechanism:** Loads prompt templates from disk, injects system block (environment, skills, memory instructions). Assembles tool registry from Permission ruleset. Composes LLM request prefix (rebuild context, checkpoints, memory). Handles max-mode candidates + judge. Manages task gates (MAX_TASK_GATE_MAIN_REACT=MAX_GOAL_REACT from goal.ts). Auto-dream/distill trigger checks at loop entry. Injects recallHintLines for Active recall protocol.
**Build-switch:** BUILD_SWITCH.txt conditional logic for model-specific behavior. MAX_STEPS.txt limits.
**Skill composition:** `composeSkillsBlock()` from skill/compose/extract.
**Cross-refs:** Almost everything: LLM, MessageV2, MaxMode, SessionProcessor, Agent, Config, Permission, Plugin, Tool, MCP, LSP, Flag, Bus, Goal, TaskGate, TaskRegistry, SystemPrompt, auto-dream, Metrics.

#### **last-message-info.ts** (implied, not read)
**Purpose:** Extract salient facts from final user message (e.g., explicit directives, implicit context).

#### **llm-request-prefix.ts** (implied, not read)
**Purpose:** Build the compact rebuild prefix injected at each step.

#### **prefix-capture-ref.ts** (implied, not read)
**Purpose:** Actor spawn ref for prefix-capture (rebuild context collection).

#### **projectors.ts** (implied, not read)
**Purpose:** Transform message data for TUI rendering.

#### **todo.ts** (implied, not read)
**Purpose:** Todo item tracking within session.

---

### Data Model & I/O

#### **budgeted-read.ts** (119 lines)
**Purpose:** Token-budgeted file reading (for checkpoint/memory reconstruction).
**Mechanism:** `readBudgeted(filePath, budgetTokens)` reads full file, checks token count, truncates if needed (preserving final newline, appending truncation hint with file:line resumption). `readBudgetedSectionAware()` parses markdown sections (## headers, _italic_ subtitles, - See index lines), preserves all section headers + indexes, fills body space up to budget, truncates gracefully.
**Return:** {text, truncated: bool, totalTokens}.
**Algorithm:** Ratio-based truncation (95% guard factor), section-aware fill-order (headers/indexes always included).
**Cross-refs:** Token.estimate.

#### **overflow.ts** (covered under compaction)

#### **schema.ts** / **session.sql.ts** (mentioned, not read)

---

## Cross-Partition Dependencies (Subsystems Called)

### Internal Subsystems

1. **Bus** (`@/bus`): Publish Goal.Updated, Compacted, Status events.
2. **Storage** (`@/storage`, Drizzle ORM): Query SessionTable, MessageTable, PartTable.
3. **MessageV2** (local): Core protocol for parts (text, tool, reasoning, snapshot, patch, compaction).
4. **Config** (`@/config`): Model limits, compaction settings, goal react caps.
5. **Permission** (`@/permission`): Tool/skill access control per agent.
6. **Provider** (`@/provider`): Model metadata, token limits, API transform, transient error classification.
7. **Agent** (`@/agent`): Agent metadata, SYSTEM_SPAWNED_AGENT_TYPES list.
8. **Memory** (`@/memory`): Project/session/global memory file access, FTS index.
9. **Plugin** (`@/plugin`): Custom tool / skill discovery.
10. **Auth** (`@/auth`): Provider credentials.
11. **Global** (`@/global`): Data path, project instance state.
12. **InstanceState** (`@/effect`): Per-instance transient storage (goals, run-state).
13. **Snapshot** (`@/snapshot`): File diff computation.
14. **Sync** (`@/sync`): SyncEvent broadcast.
15. **LSP** (`@/lsp`): Language server integration for definition/reference lookup.
16. **Flag** (`@/flag`): Feature flags.
17. **ActorRegistry** (`@/actor/registry`): Subagent spawn registry.
18. **TaskRegistry** (`@/task/registry`): Task CRUD.
19. **EffectLogger** (`@/effect`): Effect-based logging.

### External Libraries

1. **ai** (Vercel SDK): `streamText`, `generateObject`, `convertToModelMessages`, `tool`, `jsonSchema`, ModelMessage.
2. **effect** (Effect-TS): Effect monad, Context, Layer, Scope, Stream, Schedule.
3. **zod**: Schema validation.
4. **bun**: `Bun.file`, `Bun.write`, `fs/promises`.
5. **path**, **os**: File system utilities.
6. **ulid**: Unique ID generation.

---

## PORTABLE MECHANISMS — Top 5–10 Designs Worth Stealing for Rust Orchestrator

### 1. **Checkpoint Template + Structured State Injection** (checkpoint-templates.ts, checkpoint.ts)
**Why steal:** Rebuilding agent context is hard; MiMo nails it with a stable 11-section markdown template that survives rewrites. The checkpoint writer (subagent) knows exactly what to update and what to preserve. Sections lock in order: intent, action, directives, task tree, current work, files, discoveries, errors, live resources, design decisions, open notes.
**Mechanism:** Template has "(none yet)" / "(none)" placeholders. Writer overwrites file in place, preserving structure. Validator enforces no dups in §7 (discoveries) via prior-titles dedup.
**File refs:** checkpoint-templates.ts (1–80), checkpoint.ts (checkpoint-builder / ensureCheckpointTemplate), checkpoint-validator.ts (not read, but called).
**For Rust:** Port the 11 sections, enforce via serde + Markdown parser (markdown = plaintext resilience over JSON/TOML).

### 2. **Budgeted-Read with Section-Aware Truncation** (budgeted-read.ts)
**Why steal:** Checkpoint/memory files grow unbounded. MiMo prevents rebuild bloat by truncating near budget boundary while preserving all section headers + index lines (the skeleton). Fallback for massive files: headers-only + hint-to-read-full.
**Mechanism:** `readBudgetedSectionAware()` parses markdown sections, fills to budget in section order (headers/indexes always included, body fills remaining space). Graceful tail-off.
**File refs:** budgeted-read.ts (65–118), Token.estimate() for token count.
**For Rust:** Use pulldown_cmark or similar for section parsing, maintain token budget via language-agnostic token estimator (or LLM API tokenizer).

### 3. **Automatic Compaction with Overflow Pressure Levels** (compaction.ts, overflow.ts)
**Why steal:** Manual "hey run compaction" is pain. MiMo auto-triggers on pressure: 0 (safe) / 1 (50% full) / 2 (70% full) / 3 (85% full). Thresholds are configurable per model.
**Mechanism:** `usable()` = context - outputReserve - compactionBuffer. `isOverflow()` boolean, `pressureLevel()` 0–3 scale. Prune rules: PRUNE_MINIMUM=20K, PRUNE_PROTECT=40K, protected tools=["skill"]. Recent-turn preservation (defaults to 25% of usable).
**File refs:** compaction.ts (45–92), overflow.ts.
**For Rust:** Same thresholds, make outputReserve per-model config in orchestrator.toml.

### 4. **Tool/Tool-Result Pairing Boundary Guard** (boundary.ts)
**Why steal:** Compaction / summary boundary can orphan tool_use calls from their tool_result responses, confusing the model. MiMo walks backward to un-orphan them, then walks backward for shared message.id (thinking blocks).
**Mechanism:** Pure function `adjustBoundaryForApiInvariants()`: collect tail tool_results/uses, find orphans, backtrack to include matching uses. Then backtrack for shared message.id. Cheap to run, huge safety win.
**File refs:** boundary.ts (34–77).
**For Rust:** Port directly, call before every summary boundary decision.

### 5. **Per-Task Progress Reconciliation** (checkpoint-progress-reconcile.ts, checkpoint.ts)
**Why steal:** Subagent progress (tasks/<TID>/progress.md) can lag behind main checkpoint. MiMo detects NEW / CHANGED entries via written-at timestamp, injects reconcile block into checkpoint writer with explicit merge instructions (read task, integrate §4 verbatim, update marker).
**Mechanism:** `buildProgressDiffItems()` scans tasks/*/progress.md, compares written-at vs. prior reconcile marker. `renderProgressDiffBlock()` formats markdown injection. Writer integrates per instructions.
**File refs:** checkpoint-progress-reconcile.ts.
**For Rust:** Same approach: timestamps in frontmatter, reconcile markers in main checkpoint, explicit merge block for writer.

### 6. **Auto-Dream Interval Gating** (auto-dream.ts)
**Why steal:** Consolidation tasks (dream/distill) need circuit-breaker: don't spawn too often, don't run on brand-new projects. MiMo checks elapsed time vs. last run + project age (minimum DEFAULT_DREAM_INTERVAL_DAYS=7).
**Mechanism:** `shouldAutoDream()` queries SessionTable for prior Auto Dream session, computes elapsed, checks project age (earliest top-level session), skips if too young or too recent. MIN_SPAWN_GAP_MS=10s prevents dupe triggers within same turn.
**File refs:** auto-dream.ts (38–110).
**For Rust:** Same queries, configurable intervals per project type (web vs. agent vs. firmware).

### 7. **Stop-Condition Judge (Goal Loop)** (goal.ts)
**Why steal:** User sets `/goal: write a feature flag system` and expects the agent to stop when done, not when the user asks. MiMo runs a separate judge model over the transcript, confirming the condition is satisfied (or impossible). Judge sees system prompt teaching it "when is this condition satisfied?", then reads full transcript + closing question.
**Mechanism:** `evaluate()` calls generateObject with JUDGE_SYSTEM prompt + conversation + condition. Returns {ok, impossible?, reason}. Main loop reads verdict before accepting stop.
**File refs:** goal.ts (62–79, 93–97).
**For Rust:** Same judge pattern, call before honoring user/autonomous stop.

### 8. **Max-Mode Ensemble + Judge Selection** (max-mode.ts, processor.ts)
**Why steal:** Deterministic selection of best LLM output among N candidates (default 5). Candidates run propose-only (no tool execution), separate judge picks winner by reasoning, winner replayed with full execution. Overhead cost (losing candidates + judge) tracked separately, not charged to message token budget.
**Mechanism:** `toSchemaOnlyTools()` strips execute closure for propose-only. Each candidate streams reasoning + text + tool proposals. Judge selects best. ReplayInput reconstructs with selection metadata + overhead cost.
**File refs:** max-mode.ts (78–80+), processor.ts (ReplayInput).
**For Rust:** Same shape, useful when output quality matters more than latency (e.g., high-stakes decisions).

### 9. **Message Classification (Terminal vs. Continue)** (classify.ts)
**Why steal:** Pure, order-dependent classification of "should the loop continue?" Checks 9 signals in priority order: pending client tools (highest), finish reason, safety filters, text content, empty content. Degraded flag for abnormal-but-usable finishes.
**Mechanism:** `classifyAssistantStep()` = pure function, no I/O. Returns StepClassification union: final | continue | filtered | think-only | invalid | failed. Beats the usual "parse finish reason and guess" pattern.
**File refs:** classify.ts (31–92).
**For Rust:** Port directly, call before loop termination decision.

### 10. **Rebuild-Time Memory Hierarchy + Migration** (checkpoint-paths.ts, claude-import.ts)
**Why steal:** Session has 3 memory levels: global (\<data>/memory/global/MEMORY.md), project (\<data>/memory/projects/<pid>/MEMORY.md), session (\<data>/memory/sessions/<sid>/checkpoint.md + tasks/<TID>/progress.md). MiMo auto-migrates legacy lowercase memory.md to uppercase MEMORY.md (one-shot, atomic). Path helpers keep hierarchy consistent.
**Mechanism:** `migrateProjectMemory()` atomic rename, idempotent (no-op if uppercase exists or if both missing). Path functions expose hierarchy without hardcoding.
**File refs:** checkpoint-paths.ts (14–60).
**For Rust:** Same hierarchy, migrate on first read, path helpers in a dedicated module.

---

## Session Partition Design Philosophy

The session partition embodies **three core principles:**

1. **Checkpoint-as-source-of-truth:** Rebuild from markdown checkpoint + memory files, never binary state. Writer subagent curates, validator ensures completeness, progress-reconcile integrates subagent work.

2. **Overflow prevention as a service:** Automatic pressure tracking (0–3 levels), early signals (pressureLevel), on-demand compaction, and boundary guards (tool pairing, thinking blocks) prevent the silent "ran out of context" failure.

3. **Goal-driven autonomous loops:** User sets `/goal: condition`, judge model independently confirms when done. Separates intent (user) from evidence (judge), preventing agent self-serving declarations.

---

## Token Budget Summary

Partition total: ~150K tokens.
- Core session CRUD: ~10K (session.ts, schema.ts, session.sql.ts).
- Message protocol: ~30K (message-v2.ts, checkpoint.ts, compaction.ts).
- Context management: ~40K (checkpoint-*.ts suite, budgeted-read.ts, boundary.ts).
- Loop control: ~50K (processor.ts, classify.ts, llm.ts, goal.ts, max-mode.ts, prompt.ts).
- Automation: ~20K (auto-dream.ts, run-state.ts, status.ts, summary.ts, system.ts).

---

## Metrics & Thresholds (Instrument These)

- **TAIL_MIN_TOKENS=10_000, TAIL_MAX_TOKENS=20_000** — empirical sweet spot for recent-message anchor.
- **PRUNE_MINIMUM=20_000, PRUNE_PROTECT=40_000** — thresholds for aggressive pruning.
- **OUTPUT_CAP=20_000** — compaction summary token ceiling (covers >99.99% of real outputs).
- **DEFAULT_DREAM_INTERVAL_DAYS=7** — consolidation frequency.
- **MAX_GOAL_REACT** — re-entry cap for goal-driven loops (higher than spawned actors' 3).
- **Pressure levels 0–3** at 50%, 70%, 85% of usable window.

---

## Next Steps for ELAI Integration

1. **Rust Checkpoint Writer:** Port the checkpoint.txt prompt + validator logic as a native Rust task.
2. **Budgeted Read for Rust:** Adapt budgeted-read for Rust file I/O + token estimation.
3. **Goal Judge:** Integrate with ELAI's existing auditor/judge infrastructure.
4. **Max-Mode:** If ELAI doesn't already support candidate ensembles, port max-mode.
5. **Auto-Dream / Auto-Distill:** Condition these on ELAI's phase/plan completion, not just interval.

---

**Analysis completed.** This partition is a self-contained, well-designed orchestration layer ready for cross-framework adaptation.
