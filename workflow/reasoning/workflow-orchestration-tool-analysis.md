WORKFLOW ORCHESTRATION TOOL — ANALYSIS + ELAI FIT
Authored 2026-06-01. Source: direct inspection of the Claude Code install on this machine
(anthropic.claude-code-2.1.159) + the persisted artifacts of run wf_e2948fab-3a8 + two read-only
Explore agents (Claude Code impl location; ELAI orchestration mapping). Every file:line below was
observed, not assumed. Implementation of the tool itself is closed (see section 2); everything
else is grounded in on-disk artifacts.

==============================================================================
0. ONE-LINE VERDICT
==============================================================================
The Claude Code "Workflow" tool is a deterministic, AUTHORABLE multi-agent fan-out engine: the
caller writes a small JavaScript program that spawns sub-agents in parallel/pipeline, enforces a
JSON schema per agent, journals every event for replay, and tracks a token budget. ELAI already
believes in this exact philosophy ("the code orchestrates, the model is a worker") and already has
the low-level PRIMITIVES (BSP supersteps, channels, AgentPool, JoinSet, agent roles, SSB journal,
ELAI_SEED determinism) — but they are scaffold-only / wire-dark, and ELAI has NO authorable
orchestration layer. The Workflow tool is, in effect, a clean reference implementation of what
ELAI's orchestrator + pipeline-factory is trying to become. Adopting its ERGONOMICS (an executable
orchestration spec the deterministic engine runs) + wiring the BSP engine is a high-leverage,
on-thesis move.

==============================================================================
1. WHAT IT IS (NOT a skill, NOT a rule)
==============================================================================
It is a BUILT-IN tool of the Claude Code harness, in the same category as Bash / Read / Edit /
Agent. There is no .claude/skills or .claude/rules file that defines it; it ships inside the
Claude Code runtime. So there is no project-local path that "is" the tool.

Trigger to use it (observed this session): the harness "Ultracode" mode was ON (a runtime setting),
which is a standing instruction to use Workflow on substantive tasks. Typing the literal word
"workflow" in a user message is also an opt-in. By default it requires explicit opt-in because a
single run can spawn dozens of agents and burn large token volume (the ELAI-direction run used
~940K sub-agent tokens across 14 agents).

==============================================================================
2. WHERE IT LIVES + WHAT IS / ISN'T INSPECTABLE
==============================================================================
Install (this machine):
  C:\Users\user\.vscode\extensions\anthropic.claude-code-2.1.159-win32-x64\
    extension.js            (~381K lines, MINIFIED/bundled — not human-readable)
    resources\native-binary\claude.exe   (the runtime)
    webview\index.js        (UI)
    claude-code-settings.schema.json      (harness settings only; NOT the workflow API)

INSPECTABLE (audit-friendly by design):
  - The orchestration script the caller wrote (plain JS):
      <project>\<session-uuid>\workflows\scripts\<name>-<runId>.js
  - Run metadata + full results:
      <project>\<session-uuid>\workflows\<runId>.json   (runId, taskId, full script source, result)
  - Execution journal (event log, JSONL, one object per line):
      <...>\subagents\workflows\<runId>\journal.jsonl
      events: {"type":"started", key, agentId} and {"type":"result", key, agentId, result}
      "key" is a deterministic content hash (same prompt+opts => same key) — enables caching/resume.
  - Per-sub-agent transcript (full conversation incl. every tool call + result):
      <...>\subagents\workflows\<runId>\agent-<id>.jsonl   (one per spawned agent)
      <...>\agent-<id>.meta.json   (currently just {"agentType": "..."}, extensible)

NOT INSPECTABLE:
  - The tool implementation (bundled in extension.js / native claude.exe — closed/obfuscated).
  - A formal published schema for the workflow script API (learned from usage, not a doc file).

Takeaway for ELAI: the persistence layer is the interesting part and it is fully observable. The
journal + per-agent transcript + deterministic key is exactly the audit/replay model ELAI's SSB +
session-handoff + ELAI_SEED determinism is built around. ELAI does not need the closed runtime; it
needs the PATTERN, which is fully visible on disk.

==============================================================================
3. HOW IT WORKS (execution model + API, observed from run wf_e2948fab-3a8)
==============================================================================
The caller authors a JS module. First statement MUST be a pure-literal `export const meta = {name,
description, phases:[...]}`. The body is async; standard JS built-ins available EXCEPT Date.now /
Math.random / argless new Date (banned so runs are reproducible — same determinism stance as ELAI's
entropy quarantine). Hooks available in the script body:

  agent(prompt, opts?) -> Promise<result>
     opts: { label, phase, schema (JSON Schema; forces a StructuredOutput tool-call and validates,
             retrying on mismatch), model, agentType (e.g. 'Explore', 'general-purpose'),
             isolation:'worktree' }
     Without schema returns final text; WITH schema returns the validated object. Returns null if
     skipped (so .filter(Boolean)).
  parallel(thunks[]) -> Promise<results[]>   BARRIER: awaits all; a throwing thunk -> null.
  pipeline(items, stage1, stage2, ...) -> per-item independent multi-stage, NO barrier between
     stages (item A can be in stage 3 while item B is in stage 1). This is the DEFAULT shape.
  phase(title)  groups subsequent agents in the progress UI.
  log(msg)      narrator line to the user.
  budget        { total, spent(), remaining() } — the turn's token target; agent() throws once the
                hard ceiling is hit. Enables "loop until budget" dynamic depth.
  workflow(name|{scriptPath}, args) — run another workflow inline (one level deep).

Concurrency: capped at min(16, cores-2) concurrent agents; lifetime cap 1000. Excess queue.
Resume/caching: relaunch with {scriptPath, resumeFromRunId} — unchanged (prompt,opts) agent() calls
return cached results from the journal instantly; only edited/new calls re-run. Same script+args =
100% cache hit. This is why Date.now/random are banned (they'd break the cache key).

Canonical shape (what the ELAI-direction run used): a find -> verify pipeline, or a parallel fan-out
of independent research streams + a parallel pass of adversarial verifiers, each returning a
schema-validated object, then the MAIN model synthesizes (the script returns raw data; the main
model does the reasoning).

==============================================================================
4. DOES ELAI ALREADY DO THIS? (grounded mapping)
==============================================================================
Short answer: ELAI shares the PHILOSOPHY and has the PRIMITIVES, but not the authorable ergonomic,
and the parallel-execution primitives are wire-dark.

A. PHILOSOPHY MATCH (strong).
   DETAILED_SUMMARY_01 section 4: "the Rust binary controls the workflow. The model is a worker that
   receives a structured prompt, produces output, and the code validates it." That is the Workflow
   tool's thesis verbatim. ELAI's whole "intelligence in the infrastructure, not the model" stance is
   the same bet as "a deterministic script orchestrates; agents are GC'd workers." The Workflow tool
   is external validation that ELAI's core architecture choice is correct.

B. PRIMITIVES PRESENT (but scaffold/wire-dark).
   - Parallel agent execution: AgentPool::spawn_parallel exists
     (.elai_cc/crates/orchestrator/src/agent_pool.rs:224, impl 685-697) BUT the production
     InProcessAgentPool::spawn_parallel just LOOPS spawn() sequentially (agent_pool.rs:690-696). Real
     concurrency lives in BspEngine::execute_tasks_with_timeout via tokio::JoinSet
     (bsp/engine.rs:607-629).
   - BSP superstep engine = ELAI's barrier-synchronized fan-out = the Rust analogue of parallel() /
     pipeline() barriers, with typed channels (LastValue / Topic / EphemeralValue / UntrackedValue).
     BUT: BspEngine has NO production caller — every BspEngine::new() is inside test modules
     (bsp/engine.rs:1180-1683). It is scaffold-only.
   - SupervisionManager (restart/recovery on top of agents) is NOT wired into task_runner
     (supervision.rs:1-11 self-admits).
   - WIRED today: AgentPoolBabyStepExecutor (runtime_setup.rs:627-638, task_runner.rs:1256) +
     start_task_runner (task_runner.rs:4086) — the per-baby-step execution path runs.

C. ERGONOMIC MISSING (the key gap).
   ELAI has NO equivalent of "author a small orchestration program the system executes." What it has:
   - Hardcoded FSM-cycle SKILLS: b-plan-runner (SKILL.md: "a state machine encoded as a sequence,
     follow top to bottom") and orchestrate (fixed per-sub-phase loop, parallel cap=3). The SEQUENCE
     of agent dispatches is baked into the .md; the LLM cannot author a new fan-out shape at runtime.
   - Static plan FILES (B-N markdown): metadata (sub-phases, tasks, WIRING_CONTRACT) that the
     orchestrate skill READS, not an executable orchestration.
   - The Pipeline Factory (Phase 4D/4E) DOES aim at this: ExecutablePipeline + DecompositionTree +
     TreeNavigator is "the primary execution artifact — a typed Rust struct, not prompts." That is
     ELAI's intended deterministic orchestration object. It IS wired into task_runner (per
     DETAILED_SUMMARY: 12/12 sub-steps wired, drives task_runner.rs:3143). But it executes a fixed
     decomposition tree, not an arbitrary author-supplied fan-out/pipeline graph, and it does not run
     agents in true parallel (BSP unwired).

D. THE IRONY WORTH NOTING.
   ELAI's own DEVELOPMENT (the completion-drive + night-run sessions) used THIS Claude Code
   Agent/Workflow pattern — the main LLM dispatching parallel general-purpose agents
   (context-compiler + integration-explorer, cap=3). So the harness ELAI is built WITH already
   provides the orchestration ELAI's runtime is trying to build. ELAI's runtime has the parts on the
   bench; the harness has them assembled.

==============================================================================
5. WOULD IT WORK ON ELAI / HOW TO ADOPT IT
==============================================================================
Yes, and it maps cleanly onto on-thesis, already-planned ELAI surfaces. Concrete adoption, in
leverage order:

(1) WIRE THE BSP ENGINE (close the biggest scaffold gap).
    BspEngine + execute_tasks_with_timeout (JoinSet) already implement parallel + barrier. Give it a
    production caller from task_runner / pipeline executor so spawn_parallel is actually parallel
    (today it is sequential). This is the single change that turns ELAI's "parallel agents" from
    aspiration into fact. Low risk: the code exists and is tested.

(2) MAKE THE ORCHESTRATION AUTHORABLE (the ergonomic ELAI lacks).
    The Workflow tool's win is that a fan-out/pipeline is DATA you write, not a hardcoded skill.
    ELAI's analogue already exists in spirit: ExecutablePipeline / DecompositionTree are typed
    orchestration objects. Extend them so a pipeline can express "run these N agent steps in
    parallel" and "pipeline these stages" as first-class, not just a fixed concept->phase->step->baby
    tree. The Designer/PipelineBuilder (4D/4E) should be able to EMIT a parallel/pipeline graph, and
    the executor should run it over BSP. This is "ELAI authors its own workflow scripts as typed Rust
    PipelineSpec" — strictly stronger than JS because it is typed + deterministic + entropy-quarantined.

(3) ADOPT THE PER-AGENT SCHEMA-ENFORCEMENT PATTERN.
    Workflow's `schema` option forces each agent to emit a validated JSON object (retry on mismatch).
    ELAI already has Tier-2 structured output (GRAMMAR / INSTRUCTOR modes, constrained decoding,
    E-35 XGrammar). Wire that as the default contract on every internal agent dispatch so agent
    outputs are schema-validated structs, not free text the orchestrator must parse. This directly
    serves the trust thesis (typed, checkable agent IO).

(4) THE JOURNAL/RESUME MODEL IS ALREADY ELAI'S — UNIFY IT.
    Workflow's journal.jsonl (started/result per agent, deterministic content key, replay/resume) is
    functionally ELAI's SSB + session-handoff + checkpoint + ELAI_SEED determinism. ELAI's version is
    arguably richer (74 SSB action types, BSP checkpoints, ELAI_SEED byte-identical runs). The lesson
    is ergonomic: expose a "resume this orchestration from its journal" path the way Workflow does
    (resumeFromRunId), so a killed autonomous session resumes mid-fan-out from cached agent results
    rather than re-running. ELAI has the substrate; it needs the resume-from-journal entry point
    wired (E-44 idempotency/SAGA + session_dag are the right homes).

(5) BUDGET HOOK = ELAI'S TOKEN/SPEND GATES.
    Workflow's budget.remaining()/spent() loop-until-budget pattern is ELAI's CloudSpendTracker /
    cost_gate / hard-ceiling (E-39). Same idea; ELAI already has the harder version (hard ceiling
    that BLOCKS, per-tool gates). No new work needed conceptually; just confirm the autonomous loop
    consults it the way a Workflow loop consults budget.

NON-FIT / CAUTION:
    Do NOT adopt JS-script authoring inside ELAI's runtime. ELAI's whole differentiator is
    typed/deterministic Rust orchestration; a JS layer would reintroduce the non-determinism ELAI
    spent Phase 0.9 / 3I eliminating. The thing to copy is the SHAPE (authorable parallel/pipeline
    fan-out + schema-per-agent + journal-resume + budget), expressed in ELAI's existing typed
    PipelineSpec/DecompositionTree, executed over the (now-wired) BSP engine.

==============================================================================
6. STRATEGIC READ
==============================================================================
The Workflow tool is the clearest external proof that ELAI's central architectural bet — code
orchestrates, model is a worker, every step journaled and replayable — is the right one; a frontier
lab built exactly this into its own harness. ELAI's risk was never the idea; it is that the idea is
~60% assembled (philosophy + primitives present, parallel execution + authorable layer not wired).
"Wire BSP + make the pipeline authorable + schema-per-agent + resume-from-journal" is a tight,
on-thesis tranche that converts ELAI's scaffold into the thing the harness already demonstrates
works. It also reinforces the trust positioning: a fully journaled, deterministic, schema-validated,
resumable agent fan-out is precisely "an orchestration you can audit and reproduce" — the trust
story told at the architecture layer.

Cross-refs: DETAILED_SUMMARY_01 sections 3-4 (pipeline factory + prompt architecture); phase-3D
(BSP), phase-4E (ExecutablePipeline/DecompositionTree/TreeNavigator), E-44 (durability/resume),
E-39 (cost gate), E-35 (structured output). See also memory project_market_positioning_2026.
