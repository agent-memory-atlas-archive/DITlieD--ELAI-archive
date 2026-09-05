Source: gnhf — "good night, have fun" (github.com/kunchenguid/gnhf)
Reliability: MEDIUM (reviewed public source, depth-1 clone, v0.1.42)
Files examined:
  F:\.ELAI_workflow\docs\.source\gnhf\src\core\orchestrator.ts
  F:\.ELAI_workflow\docs\.source\gnhf\src\core\agents\types.ts
  F:\.ELAI_workflow\docs\.source\gnhf\src\core\agents\claude.ts
  F:\.ELAI_workflow\docs\.source\gnhf\src\core\run.ts
  F:\.ELAI_workflow\docs\.source\gnhf\src\core\git.ts
  F:\.ELAI_workflow\docs\.source\gnhf\src\cli.ts
  F:\.ELAI_workflow\docs\.source\gnhf\package.json
  F:\.ELAI_workflow\docs\.source\gnhf\README.md
  F:\.ELAI_workflow\docs\.source\gnhf\skills\gnhf\SKILL.md

Language: TypeScript (ESM), compiled via tsdown; ships as dist/cli.mjs
Runtime: Node >= 20 (not bun; package.json engines.node ">=20")
Package manager: pnpm 11.1.1
Entry: gnhf (dist/cli.mjs via commander ^14.0.3)

Public API surface:

Orchestrator class (src/core/orchestrator.ts):

```typescript
// orchestrator.ts:93-153
export class Orchestrator extends EventEmitter<OrchestratorEvents> {
  constructor(
    config: Config,
    agent: Agent,
    runInfo: RunInfo,
    prompt: string,
    cwd: string,
    startIteration = 0,
    limits: RunLimits = {},
  )
  getState(): OrchestratorState
  requestGracefulStop(): void  // first Ctrl+C: lets current iteration finish
  handleInterrupt(): InterruptDisposition
  stop(): void                 // second Ctrl+C / SIGTERM: force-stops immediately
  async start(): Promise<void> // the main loop
}
```

OrchestratorState (orchestrator.ts:43-64):
  status: "running" | "waiting" | "aborted" | "stopped"
  gracefulStopRequested: boolean
  currentIteration: number
  totalInputTokens / totalOutputTokens: number (cumulative across iterations)
  tokensEstimated: boolean (sticky; true if ANY iteration used estimated tokens)
  commitCount: number
  consecutiveFailures / consecutiveErrors: number
  waitingUntil: Date | null (set during exponential backoff)
  hasPendingCommitFailure: boolean

Agent interface (src/core/agents/types.ts:156-163):

```typescript
export interface Agent {
  name: string;
  close?(): Promise<void> | void;
  run(
    prompt: string,
    cwd: string,
    options?: AgentRunOptions,
  ): Promise<AgentResult>;
}
```

AgentOutput required fields (types.ts:1-7):
  success: boolean
  summary: string
  key_changes_made: unknown (coerced to string[])
  key_learnings: unknown (coerced to string[])
  should_fully_stop?: boolean (optional; controls --stop-when exit)

TokenUsage (types.ts:119-128):
  inputTokens, outputTokens, cacheReadTokens, cacheCreationTokens: number
  estimated?: boolean (ACP adapters that emit no usage_update use heuristic estimates)

RunLimits (orchestrator.ts:74-79):
  maxIterations?: number   (abort BEFORE starting next iteration)
  maxTokens?: number       (can abort MID-iteration once reported usage crosses threshold)
  stopWhen?: string        (natural-language stop condition; persisted per run)
  push?: boolean           (push after each successful commit)

Core loop algorithm (orchestrator.ts:244-427):

```typescript
// orchestrator.ts:264-399 (start() inner while loop, condensed)
while (!this.stopRequested) {
  // 1. Pre-iteration abort checks (maxIterations, maxTokens)
  const preAbort = this.getPreIterationAbortReason();
  if (preAbort) { this.abort(preAbort); break; }
  if (this.stopForGracefulShutdown()) break;

  // 2. Build iteration prompt (inject notes.md context + stopWhen condition)
  //    If pendingCommitFailure: prepend commit-repair instructions
  const prompt = this.pendingCommitFailure
    ? this.buildCommitRepairPrompt(basePrompt)
    : basePrompt;

  // 3. Run agent (AbortController-wrapped; token cap can abort mid-iteration)
  const result = await this.runIteration(prompt);

  // 4. Handle result:
  //    success -> commitAll() -> appendNotes() -> reset consecutiveFailures/Errors
  //    commit-fail -> record pendingCommitFailure, ask next iteration to repair
  //    reported-fail -> reset() (git reset --hard), consecutiveFailures++, consecutiveErrors=0
  //    error-fail -> reset(), consecutiveFailures++, consecutiveErrors++ -> exponential backoff
  //    permanent-error -> abort immediately (e.g. Claude low credit)

  // 5. Backoff on consecutiveErrors > 0: sleep 60s * 2^(consecutiveErrors-1)
  //    Backoff is interruptible (Ctrl+C or graceful stop drains it immediately)

  // 6. Post-iteration abort checks (maxIterations, maxTokens, stopWhen, consecutiveFailures >= cap)
}
```

Failure handling (orchestrator.ts):
- agent-reported failure (success=false): resetHard() immediately, consecutiveErrors=0 (no backoff)
- hard agent error (threw): resetHard(), consecutiveErrors++ (triggers backoff)
- PermanentAgentError (e.g. Claude low credit): abort() immediately, no next iteration
- CommitFailedError: pendingCommitFailure set, uncommitted work preserved, repair prompt injected
- 3 consecutive failures (default maxConsecutiveFailures=3): abort()
- Complete no-op iteration (no git changes): treated as failure

Commit behavior:
Every successful iteration: commitAll() with message "gnhf <n>: <summary>" (or conventional preset)
Each commit is unsigned (no GPG/SSH signing to avoid blocking unattended runs)
Failed commit: preserved as pending, not reset; next iteration gets repair prompt

Cross-iteration shared memory:
.gnhf/runs/<runId>/notes.md: appended after each iteration (summary + key_changes + key_learnings)
Each agent sees accumulated notes via buildIterationPrompt (notes.md injected as context)
.gnhf/runs/<runId>/prompt.md: saved objective for resume
.gnhf/runs/<runId>/gnhf.log: JSONL debug log (lifecycle events, timings, error cause chains)
.gnhf/runs/<runId>/iteration-<n>.jsonl: per-iteration agent streaming output

Worktree mode (--worktree):
Creates <repo>-gnhf-worktrees/<run-slug>/ sibling directory via git worktree add
Multiple concurrent agents possible (run in background &)
Preserved if has commits; auto-removed if empty

Stop conditions:
maxIterations: checked before each iteration (pre-gate)
maxTokens: checked both pre-gate AND mid-iteration via onUsage callback; AbortController cancels agent
stopWhen: natural-language; agent must set should_fully_stop=true in output; persists across resume
Graceful stop: first Ctrl+C -> requestGracefulStop() -> loop exits after current iteration
Force stop: second Ctrl+C or SIGTERM -> stop() -> 250ms grace -> agent.close() -> resetHard()

Supported agents (6 native + ACP):
claude (Anthropic Claude Code CLI, non-interactive mode)
codex (OpenAI Codex CLI, codex exec mode)
rovodev (Atlassian acli, acli rovodev serve)
opencode (opencode serve --hostname 127.0.0.1 --port <port>)
copilot (GitHub Copilot CLI, JSONL mode)
pi (pi CLI, JSON mode, --no-session)
acp:<target> (Agent Communication Protocol via bundled acpx ^0.6.1)

Dependencies (package.json, verified):
commander ^14.0.3        (CLI framework)
js-yaml ^4.1.1           (config file parsing ~/.gnhf/config.yml)
acpx ^0.6.1              (ACP runtime, bundled, devDependency)
vitest ^4.1.2            (test runner)
tsdown ^0.21.7           (bundler)
typescript ^5.8.0

Tests:
src/core/orchestrator.test.ts: vitest unit tests with vi.mock for git/run/debug-log/iteration-prompt
src/core/agents/*.test.ts: per-agent unit tests
e2e/*.test.ts: e2e tests against a mock opencode server (e2e/fixtures/mock-opencode-server.mjs)
e2e/fixtures/acp-traces/: recorded ACP traces for claude.jsonl, codex.jsonl, opencode.jsonl

Platform constraints:
Node >= 20 (ESM, AbortController, EventEmitter)
Sleep prevention: caffeinate (macOS), systemd-inhibit (Linux), PowerShell SetThreadExecutionState (Windows)
Git required in PATH; runs git reset --hard on rollback
Config at ~/.gnhf/config.yml (YAML via js-yaml)

Gotchas:
tokensEstimated is sticky: once true, stays true for the rest of the run (displayed as "~N tokens")
commitAll() is NOT async: it calls child_process.execSync equivalent; a slow commit blocks the loop
pendingCommitFailure is NOT reset by a subsequent hard error; only explicit success or stop() clears it
--worktree and --current-branch are mutually exclusive (error on combine)
ACP adapters do not support agentPathOverride or agentArgsOverride
Telemetry: anonymous, sent to kunchenguid's self-hosted analytics; GNHF_TELEMETRY=0 disables

Critical assertions (testable):
assert consecutiveErrors resets to 0 on agent-reported failure (not a backoff trigger)
assert consecutiveErrors increments on hard throw (backoff trigger)
assert pendingCommitFailure survives the next iteration's reset (uncommitted work preserved)
assert maxTokens check fires both before iteration start AND during onUsage callback mid-iteration
assert should_fully_stop only triggers exit when stopWhen is set (ignored otherwise)
