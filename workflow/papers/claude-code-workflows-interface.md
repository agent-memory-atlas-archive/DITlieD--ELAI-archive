Source: Claude Code Dynamic Workflows (Anthropic official docs + blog)
Reliability: HIGH (vendor-official documentation, code.claude.com/docs/en/workflows, 2026-06-02 blog post)
Files examined: https://claude.com/blog/a-harness-for-every-task-dynamic-workflows-in-claude-code, https://code.claude.com/docs/en/workflows
Min version required: Claude Code v2.1.154+

WHAT IT IS

A dynamic workflow is a JavaScript script that the model authors on the fly, which a
background runtime then executes to orchestrate subagents at scale. The model writes the
script; it does not call subagents directly from the conversation turn. Intermediate results
live in script variables, not in the model's context window.

Trigger mechanisms (two opt-ins, no ambient activation):
  - User includes "ultracode" keyword (or natural language "use a workflow" / "run a workflow")
    in their prompt. Highlighted in the UI; user can dismiss with Option+W / Alt+W.
  - /effort ultracode session setting: combines xhigh reasoning with automatic workflow
    planning for every substantive task in the session.
  - Existing saved workflow command: /deep-research or user-saved /<name>.
  - claude -p / Agent SDK: never prompts, runs immediately.

SCRIPT RUNTIME ENVIRONMENT

Language: JavaScript (NOT TypeScript, NOT Python)
Standard globals available: JSON, Math, Array, and other standard JS built-ins
Special functions: spawn and coordinate subagents (exact API not published in docs)
No direct filesystem or shell access FROM the script itself; only agents do I/O
Script file location: written to ~/.claude/projects/<session-dir>/ per run
Isolation: script executes in a separate process from the conversation

WORKFLOW PATTERNS (named in blog post, not API primitives)

classify-and-act: route tasks by type classification
fan-out-and-synthesize: split into parallel steps then merge results
adversarial-verification: independent agents verify outputs against rubrics
generate-and-filter: create multiple options, filter by quality criteria
tournament: multiple agents compete via pairwise comparison
loop-until-done: spawn agents until stopping condition met
rule-verification: dedicated verifier agents check specific rules

COMPARISON TO OTHER PRIMITIVES

Subagents: Claude decides turn-by-turn, results land in context window, few delegated tasks/turn
Skills: Instructions Claude follows, context-window-bound
Agent teams: Lead agent supervises peers, handful of long-running sessions
Workflows: Script decides execution, results in variables, dozens-to-hundreds agents/run,
           resumable within same session

KEY BEHAVIORAL CONTRACTS

No mid-run user input: only agent permission prompts can pause
Subagents always run in acceptEdits mode regardless of session permission mode
File edits are auto-approved for subagents
Shell/web/MCP tools not in allowlist can still prompt mid-run
Up to 16 concurrent agents (bounded by CPU cores)
1000 agents total per run hard cap
Every agent uses session model UNLESS the script routes a stage to a different model
Resumable within same session: completed agents return cached results; paused run
  exit Claude Code = workflow starts fresh on next session

SAVE/REUSE MECHANISM

/workflows UI: list, inspect, save (press s), pause (p), stop (x), restart agent (r)
Save locations:
  .claude/workflows/<name>.js  (project-local, shared via git)
  ~/.claude/workflows/<name>.js  (user-global, all projects)
Saved workflows become slash commands: /<name>
Monorepo: closest .claude/workflows/ dir to cwd wins; project wins over personal on name clash
args parameter: script receives structured args as global named "args" (no parsing needed)

COST / TOKEN MODEL

A single run uses significantly more tokens than conversation-based approach
Counts toward plan usage and rate limits like any other session
/workflows view shows per-agent token usage live
Can stop mid-run without losing completed work (caching)
Caution: workflow-for-every-task (ultracode mode) applies to EVERY request in session

AVAILABILITY

Paid plans only (Pro, Max, Team, Enterprise), Anthropic API, Amazon Bedrock,
Google Cloud Vertex AI, Microsoft Foundry
Not available on free tier
CLI, Desktop app, IDE extensions, headless (claude -p), Agent SDK
Disable: /config toggle, "disableWorkflows": true in settings.json,
         CLAUDE_CODE_DISABLE_WORKFLOWS=1 env var
Org-level disable: managed settings or Claude.ai admin page

STATED FAILURE MODES / LIMITATIONS

agentic laziness: single-context Claude stops early after partial progress (workflow solves by codifying the loop in a script)
self-preferential bias: model prefers its own results when verifying (workflow solves via adversarial-verification pattern with independent agents)
goal drift: original objective lost through summarization (workflow solves by keeping intermediate results in script variables, not context)
token cost: "often use more tokens" - not suited for routine tasks
NOTE: the blog post claims the trigger keyword "workflow" worked before v2.1.160; from v2.1.160 natural language also works

VERDICT AGAINST ELAI'S ORCHESTRATE/BSP/FSM

ELAI's CLAUDE.md hard-bans the Workflow tool: "NEVER invoke the Workflow tool off the
harness's workflow keyword system-reminder. That trigger false-positives constantly."

The mechanism confirms the ban is correct: the ambient keyword "workflow" in any user
message previously (pre-v2.1.160) triggered the tool. The ELAI project root is literally
named .ELAI_workflow and CLAUDE.md + plan files use "workflow" constantly as domain vocab,
so the keyword would fire spuriously on EVERY session.

ELAI's orchestrate skill is a REPLACEMENT with explicit user opt-in ("orchestrate" command)
and the main LLM as controller dispatching well-defined agents, NOT a JS script authored
by the model mid-conversation. These are architecturally distinct:
  - Workflow tool: model writes JS script, runtime executes it, no turn-by-turn LLM control
  - ELAI orchestrate: LLM is the controller, dispatches typed agents, reads structured returns,
    FSM gates every transition

UNVERIFIED: exact JavaScript API surface (function signatures like agent(), parallel(),
pipeline()) not published in official docs; only patterns and behavior described.
UNVERIFIED: whether the script can call external HTTP APIs or is sandboxed to agent spawning only.
