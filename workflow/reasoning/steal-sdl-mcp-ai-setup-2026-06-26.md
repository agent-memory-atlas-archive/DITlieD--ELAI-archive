Steal summary: sdl-mcp + ai-setup (Caliber)
Authored: 2026-06-26
Source: clones at F:\.ELAI_workflow\docs\.source\sdl-mcp and F:\.ELAI_workflow\docs\.source\ai-setup
Reliability: HIGH for both (clean source reads from successful shallow clones)

---

REPO 1: sdl-mcp

REPO
  name:            sdl-mcp
  url:             https://github.com/GlitterKill/sdl-mcp
  license:         Custom "FREE PERSONAL USE LICENSE" (SDL-MCP Community License v1.0,
                   2026-02-11, Gary Mullins). Free for internal/personal use; paid
                   commercial license required to distribute or embed as a product.
  primary lang:    TypeScript (ESM, Node.js 24+)
  last-commit:     d66ced4a  2026-06-25  "Use TS fallback for Linux sync memory indexing"
  reliability:     HIGH (full source clone, all source files readable)

WHAT IT IS
  sdl-mcp is a Symbol Delta Ledger served as an MCP server. It parses every symbol in a
  polyglot repository via tree-sitter (11 language adapters: TS, Python, Go, Java, Rust,
  C#, C++, C, Kotlin, PHP, Shell) and stores compact Symbol Cards (~100 tokens each) in a
  KuzuDB property graph (npm:@ladybugdb/core@0.16.0, optional). Agents access this graph
  through a four-rung "Iris Gate Ladder" that escalates from Symbol Card -> Skeleton IR ->
  Hot-Path Excerpt -> Code Window, requiring proof-of-need before returning raw source.
  The core value proposition is 4-20x token reduction compared to naive file reads.

KEY MECHANISMS
  - Symbol Cards (src/indexer/indexer.ts, src/services/card-builder.ts):
    Each fn/class/type becomes a ~100-token record with name, signature, deps (imports +
    calls with confidence scores), metrics (fanIn, fanOut, churn30d, testRefs), community
    cluster membership, and an ETag for conditional-get caching. The pass-2 resolver
    (src/indexer/indexer-pass2.ts) traces imports/aliases/barrel-reexports to produce
    accurate cross-file dependency edges with confidence scores.

  - Iris Gate Ladder / Context Ladder (src/CONTEXT.md, src/policy/code-access.ts):
    Four progressive rungs: (1) Symbol Card, (2) Skeleton IR, (3) Hot-Path Excerpt, (4)
    Code Window. Each rung costs more tokens. Code Window is gated behind
    decideCodeAccess() (src/policy/code-access.ts) which returns approve/downgrade/deny.
    Code Window also runs enforceCodeWindow() (src/code/enforce.ts) to verify the
    requested identifiers are actually present in real file text before returning.

  - Beam-Search Graph Slice (src/graph/slice/beam-search-engine.ts:beamSearch):
    Given start nodes, beam search walks the KuzuDB symbol graph to assemble a token-
    budgeted "slice" -- a dependency frontier. Uses adaptive min-confidence thresholds,
    dynamic card-cap adjustments (shouldTightenDynamicCardCap), parallel scorer workers
    (ParallelScorerPool), and MinHeap priority queue. Slice results are cached
    (src/graph/sliceCache.ts) and served with spillover pagination.

  - Delta Pack + Blast Radius (src/delta/blastRadius.ts:runGovernorLoop,
    src/mcp/tools/delta.ts:handleDeltaGet):
    Computes which symbols changed between two ledger versions and ranks affected
    downstream symbols by shortest-path distance, fan-in, and test proximity
    (batchComputeFanInOut, shortestPath from src/db/ladybug-algorithms.ts). Cache TTL 60s,
    max 50 entries. Exposed as delta.get MCP tool.

  - Policy Engine (src/policy/engine.ts, src/policy/code-access.ts:decideCodeAccessLegacy):
    Pure decision layer (no I/O): given a PolicyRequestContext, emits approve/downgrade/deny
    for each of six artifact types: Code Window, Skeleton IR, Hot-Path Excerpt, Symbol Card,
    Graph Slice, Delta Pack. PolicyConfig is user-configurable and cached in an LRU
    ENGINE_CACHE_MAX=32 map. Downgrade paths steer agents toward cheaper rungs.

  - Live Index + Overlay Store (src/live-index/coordinator.ts, src/live-index/idle-monitor.ts):
    Watchman or Chokidar file watcher detects source changes; IdleMonitor fires
    checkpoint writes (DEFAULT_IDLE_CHECKPOINT_INTERVAL_MS) so the in-memory overlay
    is drained to durable state during idle periods. Overlay store is queried first
    (searchSymbolsWithOverlay in src/live-index/overlay-reader.ts) before the base graph,
    so recent edits are immediately visible without a full re-index.

  - Agent Memory Layer (src/mcp/tools/memory.ts, src/memory/surface.ts,
    src/memory/file-sync.ts):
    Opt-in (memory.enabled config flag). Agents store typed memory entries via memory.store;
    memory.surface returns relevant entries ranked by centrality signals
    (loadCentralitySignals from the symbol graph). Memories are also persisted as files
    in the workspace for human inspection (writeMemoryFile in src/memory/file-sync.ts).

  - Runtime Execution (src/mcp/tools/runtime.ts, src/runtime/executor.ts:execute):
    sdl.runtime.execute runs repo-local commands inside a sandbox environment
    (buildScrubbedEnv strips dangerous env vars, resolveAndValidateCwd enforces cwd).
    Output is persisted as artifacts, queryable later via runtimeQueryOutput with focused
    queryTerms. Policy gate (decideRuntimeLegacy in src/policy/runtime.ts) controls
    which executables are allowed per config.

  - Structured Agent Workflow Instructions (src/mcp/server-instructions.ts,
    src/SDL.md):
    Server-level system prompt injected at MCP session start defines the canonical 8-step
    workflow: repo.status check -> sdl.context -> symbolSearch/symbolGetCard -> slice.build
    -> codeSkeleton/codeHotPath -> codeNeedWindow -> file.read (non-indexed only) ->
    runtimeExecute. Agents are steered away from native file reads on indexed source.

  - Token Accumulator + Savings Meter (src/mcp/token-accumulator.ts,
    src/mcp/savings-meter.ts):
    Tracks raw-equivalent tokens that would have been consumed by file reads vs. actual
    tokens consumed via SDL. Reported in usageStats at session end.

  - sdl-mcp-agent-workflow SKILL (referenced in src/mcp/server-instructions.ts):
    A companion SKILL.md delivered to MCP-capable clients so the workflow instructions
    survive context compaction (the SKILL is always-loaded; SDL.md is the in-repo fallback).

HOW AGENTS USE IT
  Agents connect via stdio MCP transport (sdl-mcp serve --stdio) or any MCP client.
  The server exposes ~17 tool categories (see src/mcp/tools/ directory):
  repo.status, index.refresh, sdl.context, symbolSearch, symbolGetCard, slice.build,
  delta.get, codeSkeleton, codeHotPath, codeNeedWindow, file.read, file.write,
  symbol.edit, search.edit, runtimeExecute, runtimeQueryOutput, memory.{store,query,
  surface,remove}, usageStats, agentFeedback, policy.{get,set}.
  The server also injects a mandatory workflow instruction into the MCP session
  (SDL_MCP_SERVER_INSTRUCTIONS in src/mcp/server-instructions.ts) that binds the agent
  to the escalation ladder instead of naive file reads.

PORTABLE IDEAS
  - Graduated context retrieval (Symbol Card -> Skeleton -> Hot-Path -> Code Window):
    a harness can apply the same four-rung gate to any code intelligence source to prevent
    agents from burning tokens on unnecessary file reads.
  - Policy-gated code window with proof-of-need: requiring the agent to name specific
    identifiers it expects to find before returning raw source is a concrete anti-slop gate;
    enforceCodeWindow verifies presence after the fact.
  - Beam-search over a symbol graph to build token-budgeted context slices is a direct
    replacement for ad-hoc "read these N files" context strategies.
  - Delta Pack blast-radius ranking (fanIn + shortest-path + test proximity) is a
    concrete algorithm for impact analysis that beats naive grep-based approaches.
  - Agent memory ranked by graph centrality signals ties episodic memory relevance to
    architectural importance of the code the agent worked on.
  - Token savings metering (src/mcp/savings-meter.ts) as a first-class session metric
    gives empirical evidence for context-economy claims rather than vibe-based claims.
  - The idleMonitor + overlay-store pattern (checkpoint on idle, overlay before base) is
    a lightweight way to make a persistent index live-updated without a full rebuild per edit.
  - Injecting mandatory workflow instructions at MCP session level (server-instructions.ts)
    rather than relying on CLAUDE.md alone is more reliable because it fires even when the
    project lacks CLAUDE.md.

---

REPO 2: ai-setup (Caliber)

REPO
  name:            ai-setup (published as @rely-ai/caliber)
  url:             https://github.com/caliber-ai-org/ai-setup
  license:         MIT (Copyright 2025 Caliber AI)
  primary lang:    TypeScript (ESM, Node.js >=20)
  last-commit:     d59ae5bc  2026-05-20  "Merge pull request #219 from caliber-ai-org/baghdad"
  reliability:     HIGH (full source clone, all source files readable)

WHAT IT IS
  Caliber is a CLI and GitHub Action that generates and continuously syncs AI agent
  configuration files -- CLAUDE.md, .cursor/rules/*.mdc, AGENTS.md,
  .github/copilot-instructions.md, .claude/skills/, .agents/skills/ -- across Claude Code,
  Cursor, Codex, OpenCode, and GitHub Copilot. It solves config drift: a hand-written
  CLAUDE.md goes stale the moment the codebase evolves; Caliber re-derives it from a
  live fingerprint of the project on every commit via a pre-commit hook. Its scoring
  engine is entirely deterministic (no LLM, no API calls) and cross-references config
  content against the actual project filesystem.

KEY MECHANISMS
  - Deterministic Scoring Engine (src/scoring/index.ts:computeLocalScore,
    src/scoring/checks/):
    Six check categories: existence (src/scoring/checks/existence.ts), quality
    (src/scoring/checks/quality.ts), grounding (src/scoring/checks/grounding.ts),
    accuracy (src/scoring/checks/accuracy.ts), freshness (src/scoring/checks/freshness.ts),
    bonus (src/scoring/checks/bonus.ts). Each category returns Check[] with id, maxPoints,
    earnedPoints, passed, detail, suggestion, and a fix object describing the exact
    remediation action. Grounding check (checkGrounding) measures what fraction of real
    project dirs/files are mentioned in the config; freshness check cross-references
    against git log. Total score 0-100; grade A-F. No LLM involved.

  - Project Fingerprint (src/fingerprint/index.ts:collectFingerprint):
    Collects gitRemoteUrl, file tree, existing config content, package name, and runs
    LLM-driven stack detection (src/ai/detect.ts:detectProjectStack) to infer languages,
    frameworks, and tools with no hardcoded mappings. Result is cached in a fingerprint
    cache (src/fingerprint/cache.ts) keyed by file-tree hash to avoid redundant LLM calls.

  - LLM-Driven Config Generation + Refresh (src/ai/generate.ts:generateSetup,
    src/ai/refresh.ts:refreshDocs):
    generateSetup takes the fingerprint + failing score checks and produces initial CLAUDE.md,
    skills, rules, copilot instructions in one LLM call. refreshDocs takes the git diff
    since last commit (src/lib/git-diff.ts:collectDiff) and re-derives only the sections
    that need updating. Both functions use the shared ROLE_AND_CONTEXT + AUDIT_CHECKLIST +
    SKILL_FORMAT_RULES system prompt (src/ai/prompts.ts) which encodes the OpenSkills spec.

  - Multi-Platform Writer Layer (src/writers/):
    Per-platform writers (claude/index.ts, cursor/index.ts, codex/index.ts,
    github-copilot/index.ts) materialize config files from the LLM response. Staging
    (src/writers/staging.ts:stageFiles) writes to a tmp dir first; backup
    (src/writers/backup.ts) saves originals to .caliber/backups/ before any overwrite;
    undoSetup restores from backup. Manifest tracks which files are Caliber-managed.

  - Pre-Commit Hook Integration (src/lib/hooks.ts:installPreCommitHook,
    src/lib/learning-hooks.ts):
    caliber hooks --install writes a git pre-commit hook that runs caliber refresh on every
    commit. A PostToolUse stop hook (installStopHook) and SessionStart hook
    (installSessionStartHook) wire into Claude Code's hook system to capture tool events
    for session learning (src/learner/storage.ts:appendEvent, appendPromptEvent).

  - Session Learning Loop (src/commands/learn.ts, src/ai/learn.ts:analyzeEvents,
    src/learner/attribution.ts):
    During a Claude Code session, the stop hook captures every tool event and user prompt
    (hook_event_name: PostToolUse | UserPromptSubmit). After the session, caliber learn
    finalize passes these events to analyzeEvents() which calls an LLM to extract learned
    patterns, corrections, and new skills (AnalysisResult: claudeMdLearnedSection + skills[]
    + tasks[]). Corrections are appended to CALIBER_LEARNINGS.md. Attribution
    (matchLearningsToFailures, semanticMatchFallback) measures which past learnings prevented
    failures in this session to compute an ROI score. Threshold: MIN_EVENTS_FOR_ANALYSIS=25.

  - Multi-Provider LLM Abstraction (src/llm/index.ts:createProvider):
    Supports anthropic (via @anthropic-ai/sdk ^0.78.0), vertex (via @anthropic-ai/vertex-sdk
    ^0.14.4), openai (via openai ^6.29.0), minimax, cursor (via Cursor Agent CLI),
    claude-cli (via Claude Code CLI without API key), opencode. Provider is selected from
    .caliber/config.json or interactively via runInteractiveProviderSetup. No API key
    needed if Claude Code CLI or Cursor is already logged in.

  - Bootstrap + Skill Installation (src/commands/bootstrap.ts:bootstrapCommand,
    src/lib/builtin-skills.ts):
    caliber bootstrap installs BUILTIN_SKILLS (setup-caliber, find-skills, save-learning,
    writers-pattern, scoring-checks, caliber-testing, adding-a-command, llm-provider as
    found in skills/ directory) into .claude/skills/, .cursor/skills/, .agents/skills/
    depending on detected platforms (detectPlatforms scans ~/.claude, ~/.cursor, ~/.codex,
    ~/.config/opencode). After bootstrap, the agent runs /setup-caliber to complete setup.

  - /setup-caliber Skill (skills/setup-caliber/SKILL.md):
    A SKILL.md that the agent loads and executes. Five steps: check caliber install ->
    check pre-commit hook -> detect agents and check if configs exist -> score check ->
    team vs. solo setup. Each step runs deterministic bash checks first; the agent only
    acts if something is missing. Skill also handles GitHub Action installation for team
    setups.

  - Score Comparison Against Git Baseline (src/commands/score.ts:scoreBaseRef):
    caliber score --compare main checks out the config files from the git base ref into a
    tmp dir, runs computeLocalScore on them, and diffs against the current score to show
    what the branch changed. No LLM; uses git show + execFileSync.

HOW AGENTS USE IT
  Agents interact with Caliber through two surfaces:
  (1) SKILL.md files installed by caliber bootstrap -- /setup-caliber is the primary
      entry point; the agent reads the skill and executes numbered bash steps, only
      taking action when the deterministic check shows a gap.
  (2) Pre-commit and session hooks that run caliber refresh/learn automatically so the
      agent never has to explicitly invoke Caliber during a coding session.
  The LLM abstraction (src/llm/index.ts) also lets Caliber itself call the agent's
  backing model (Claude, Cursor, OpenAI, etc.) for config generation and session analysis,
  making Caliber an LLM-driving tool as well as an LLM-driven config.

PORTABLE IDEAS
  - Deterministic config scoring (no LLM, no API) as a pre-commit gate: a scoring engine
    that measures grounding ratio (how many real project paths are cited in CLAUDE.md) and
    freshness (git log since last sync) is a cheap, always-on quality gate for harness rules.
  - The "grounding check" pattern (checkGrounding: count real project dirs/files mentioned
    in config / total dirs; fail if ratio < 0.2) is directly applicable to validating that
    ELAI's CLAUDE.md and memory-bank files stay grounded in real code paths.
  - Git-diff-driven incremental refresh (collectDiff -> refreshDocs with only changed files
    in scope) is much cheaper than full config regeneration and worth adopting in any
    harness where a CLAUDE.md-equivalent is LLM-maintained.
  - PostToolUse hook capture (appendEvent stores every tool call and its response into
    a session file) is a concrete implementation of the "session transcript as learning
    signal" pattern; ELAI's existing session/hook system could feed the same data to an
    offline learning pass.
  - Attribution scoring (matchLearningsToFailures: what fraction of past learnings
    prevented failures this session) gives empirical ROI for the memory system; applying
    this to ELAI's MEMORIZING state would make memory-bank quality measurable.
  - Provider-agnostic LLM layer (LLMProvider interface from src/llm/types.ts with six
    concrete impls) that piggy-backs on already-authenticated CLI tools (Claude Code CLI,
    Cursor Agent CLI) without requiring a separate API key is a zero-setup pattern for
    harness utilities that need occasional LLM access.
  - Staged write + backup + undo pattern (stageFiles -> backup -> apply -> undoSetup
    restores from .caliber/backups/) as a safe config-write primitive that any harness
    rule-file updater should copy.

---

CROSS-CUTTING THEMES

Both repos address the same structural problem from opposite directions: AI coding agents
operating on codebases with stale or token-wasteful context.

sdl-mcp attacks the runtime problem: the agent's context window during a task is bloated
by naive file reads. Its solution is a persistent symbol graph + gated escalation ladder
that makes "read exactly what you need" the default instead of "read the whole file."
The key design move is the policy engine as a pure decision layer that can downgrade or
deny requests without I/O, keeping the enforcement path cheap and auditable.

Caliber attacks the configuration problem: the harness files that shape agent behavior
(CLAUDE.md, skills, rules) go stale as the codebase evolves. Its solution is a
deterministic scoring engine that detects drift without LLM calls, plus an LLM-driven
refresh triggered on every commit. The key design move is separating the "is it stale?"
question (deterministic, instant) from the "what should it say?" question (LLM, expensive).

Both converge on: structured escalation (ask cheap first, escalate with justification),
off-agent deterministic verification (the policy gate, the scoring engine), and session-
level telemetry (token savings meter, ROI attribution) as first-class metrics rather than
afterthoughts. Neither system trusts the agent to self-regulate -- both impose external
constraints that fail closed.
