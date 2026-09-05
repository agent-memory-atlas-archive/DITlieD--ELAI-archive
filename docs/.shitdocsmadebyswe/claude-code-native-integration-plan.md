plan: Unified Claude Code Native Engine & Workflow Integration
created: 2026-04-01
status: pending

Goal: Transform the agent from an external participant into the "Engine Master" by natively extending the leaked Claude Code orchestration layer. This integrates the DKB (Topological Ingestor), POS-Control (6-Tier Pipeline), and FSM (Tool Gating) into the actual TypeScript runtime.


1. NATIVE TOPOLOGICAL CACHING (DKB-INTEGRATED)
Source: 01-fundamentals.md, 03-orchestration.md, SOTA Mapping

Problem: Current "context-package.md" is a static blob that causes token bloat and reasoning decay.

Solution: Extend the leaked `FileStateCache` and `QueryEngine` with "Graph-Aware Skeletonization."
- DKB-Lite Integration: Inject a Tree-sitter parser into the `context.ts` layer.
- Skeleton Cache: For every file read, generate a `filename.ts.skeleton` containing only imports, types, and exported signatures.
- Context Strategy:
    - Target Files: Load full code into `messages`.
    - Dependencies (1-hop): Load skeletons into `system_prompt` as a "Reference Map."
    - Remote Dependencies (2+ hops): Represent as a DOT graph or Mermaid diagram in a dedicated `mcp_instructions_delta` block to preserve prefix caching.


2. RUNTIME FSM & TOOL-GATING (QUERYENGINE HOOKS)
Source: 02-tool-system.md, 03-orchestration.md, CLAUDE.md

Problem: FSM is currently "soft" (prompt-based) and easily ignored.

Solution: Modify `src/QueryEngine.ts` to implement a runtime `FSMValidator`.
- State Persistence: The engine reads `.fsm-state.json` on every turn.
- Hard Gating: Modify the `executeTool` function:
    - If `state === AUDITING` and `tool === Edit|Write`, the engine throws a `NativePermissionError` and returns a "Verification-Only Mode" message to the LLM.
    - If `state === TESTING_FIRST`, the `Edit` tool is restricted to the `/tests` directory only.
- Validation Reward: The `transition(MEMORIZING)` function is only callable if the `run_pipeline.py` exit code is `0`.


3. AUTOMATED GIT-GOVERNANCE (LIFECYCLE HOOKS)
Source: Leak Discoveries (main.tsx), gitcommitinworkflow.txt

Problem: Commits are inconsistent or happen before code is verified.

Solution: Wire Git operations into the `main.tsx` startup and shutdown hooks.
- Commit on Success: In the `OnStateTransition(MEMORIZING)` hook, the engine triggers `git commit -am "[state] session: [id]"`.
- Auto-Gitignore: A `PostFileCreate` hook in `FileWriteTool` automatically checks file size and type. Files >5MB or binary types are instantly appended to `.gitignore` and excluded from the `FileTree` context.
- History Grep: If the POS-pipeline fails, the `Auditor` agent triggers a hidden `git_diff_history` tool that returns a 3-way diff (Known Good vs. Current Slop vs. Proposed Fix).


4. ADVANCED "CRISPY" SUB-AGENT ORCHESTRATION
Source: 04-advanced-patterns.md, 05-exercises.md

Problem: The "Instruction Budget" is consumed by housekeeping tasks (research, mapping).

Solution: Multi-Agent Parallelization via `AgentTool`.
- Episodic Research (Paper Ingestor): Spawned as a "Read-Only Researcher" with a restricted toolset. It distills papers into a single `mcp_instructions_delta` block.
- Topological Ingestor (The Mapper): Runs as a background "Teammate" (TeamCreateTool). It continuously updates the SQLite DKB without interrupting the main agent's flow.
- The Auditor: Runs in a dedicated "Audit Sandbox" (using the leaked sandbox logic in `BashTool.ts`) to prevent implementation side-effects.


5. PRODUCTION HARDENING (COMMUNITY PATCHES)
Source: Claude Code Leak Technical Discoveries.txt

Problem: Memory leaks (70GB+) and caching failures (cch=00000).

Solution: Implement community-sourced surgical patches.
- Prefix Cache Stability: Apply the `A.attachment.type==="mcp_instructions_delta"` patch to `src/services/api/client.ts` to stop the 99% cache drop.
- Runtime Migration: Execute the patched source directly via `node cli.js` (unmodified Node.js) instead of the buggy Bun fork.
- RSS Watcher: Add a `process.memoryUsage()` check in `QueryEngine.ts`. If RSS > 12GB, trigger an automatic `mcp_fsm_save` and force a process restart to clear the leak.


IMPLEMENTATION PHASES

Phase 1: Native FSM & Tool Gating (The "Master" Layer)
- Modify `src/QueryEngine.ts` and `src/Tool.ts` to enforce state-based permissions.

Phase 2: Topological Ingestor & Skeleton Cache (The "Brain" Layer)
- Implement AST-based skeletonization and the DKB SQLite graph.

Phase 3: Lifecycle Governance & Git (The "Memory" Layer)
- Implement transition-based commits and auto-gitignore hooks.

Phase 4: Stability & Community Patches (The "Hardware" Layer)
- Apply caching and memory leak patches.
