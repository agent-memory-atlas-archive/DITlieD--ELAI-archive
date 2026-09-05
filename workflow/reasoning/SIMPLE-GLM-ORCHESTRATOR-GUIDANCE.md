SIMPLE GLM ORCHESTRATOR — BUILD GUIDANCE (for the next session)

READ THIS FIRST: this file is GUIDANCE, not rules. It is the intent + reasoning + the
moving parts, written so the session that builds this does not have to rediscover what
we already learned. Nothing here is a hard constraint except where it quotes an existing
project rule. You (the builder) design the actual prompts and the launcher. Deviate
wherever you find a better path — just keep the spirit: prompt-driven orchestration of
GLM-5.2 Claude Code instances, walking the plans in order, with a real wired/done
pre-check, the full agent roster, one executor at a time, parallel checks, a polish lane,
and a real frontend tester whose findings jump the queue.

WHAT WE ARE BUILDING (one paragraph)
A simple orchestrator that walks every plan in plan order and, per plan, runs an extended
version of the existing orchestrate / b-plan-runner FSM — but the workers are GLM-5.2
Claude Code instances (claude -p configured to talk to GLM-5.2), driven entirely by
PROMPTS that point each agent at "task T of plan P." The ONLY deterministic code is the
launcher that (a) spawns the claude-code-with-GLM instance and (b) substitutes the plan
/ task / phase identifiers into the prompt template correctly. Everything else — the
work, the checks, the decisions — is done by our existing .claude/agents/*.md prompts.

WHY THIS SHAPE (grounded lessons from the conductor, so you don't repeat them)
- GLM-5.2 is NOT a weak model (#3 globally; free only on promo). The slop we saw from the
  old conductor (per-task duplicate module copies, tautological tests, empty diffs) was
  NOT a capability floor. It was an ORCHESTRATION / CONTEXT failure: workers were not
  pointed at the existing canonical module and told "EDIT this, do not recreate it," so a
  strong model with weak context defaulted to writing fresh files. Root cause confirmed in
  workflow/handoff/conductor-quality-gap.md ("workers create per-task COPIES of canonical
  modules instead of editing them"). FIX THE CONTEXT, not the model.
- Because of that, the old conductor over-invested in a heavy 6-layer gate to FILTER slop,
  and most of one long night was spent debugging the gate (cold --workspace cargo_check
  kills, broad cargo_test hangs/flakes, a 6-day-stale baseline, per_task_review nuking
  legitimate doc/rule wiring tests). The cheaper win is to NOT PRODUCE the slop: good
  context + an explicit "edit, don't recreate" instruction + a focused wiring-enforcement
  done-gate. You need far less gate when the input context is right.
- The single most important line in every executor prompt: name the EXACT existing file(s)
  to edit and forbid recreating modules that already exist. The context-compiler /
  integration-explorer step is what supplies this; do not skip it.

CORE PRINCIPLES (the spirit to keep)
- PROMPT-DRIVEN. No deterministic decision scripts. The agents reason; the launcher only
  spawns instances and templates identifiers.
- ONE deterministic piece only: the launcher + prompt-templating (plan/task/phase number
  substitution). Get the substitution right — a wrong task number silently runs the wrong
  work.
- PLAN ORDER, not dependency order. Walk phases 0 -> 1(A..I) -> 2 -> 3 -> 4, then B -> C
  -> D -> E -> F -> G -> H -> I -> J -> K -> L. EXCLUDE marketing / GTM / market-positioning
  material (it is not implementation work).
- ONE EXECUTOR AT A TIME. Execution is serial (a single executor agent running, so two
  instances never fight over the same files / build). Wiring + integration CHECKS may run
  in PARALLEL (they are read-only / verification, no write conflict). A POLISH agent runs
  on the already-done parts.
- REUSE the agent files. Use the whole roster in .claude/agents/ (list below). Do NOT
  reuse the old conductor's DAG/scheduler — plan order replaces it.

PLAN SOURCES (where the tasks live)
- Phases 0-4: docs/specs/guide/steps/phase-{0,1A..1I,2*,3*,4*}.txt (the per-step files;
  phase-index-*.txt are summaries).
- B..L plans: docs/thefinalglance/{B,C,D,E,F,G,H,I,J,K,L}-plan/ . ALL-PLANS-MASTER-SUMMARY.md
  is the catalog. Each plan file decomposes into tasks/steps the prompts will reference.

THE EXTENDED FSM (per plan — this is the "more complex" orchestrate flow you asked for)

  PHASE A — WIRED/DONE PRE-CHECK (before any execution)
    An agent assesses: is plan P already wired and done? It reads the plan, then checks the
    live code (codegraph-elai + elai.map + the wire checkers production_caller_check.py /
    python_wire_check.py + a real-entrypoint smoke where cheap). It REPORTS BACK, and that
    report becomes a stored "wired: yes | partial | no" value somewhere durable (a per-plan
    status record — e.g. workflow/<orchestrator-state>/plan-status/{P}.json, or one status
    file keyed by plan; you choose). This is the fix for the old "already-done work loops
    forever" problem: we decide ONCE, up front, what state each plan is in.
    Agents to use: wiring-contract-validator (is the contract satisfied), auditor (does it
    build + tests green), integration-explorer-elai (does it reach a real entrypoint).
    This pre-check can run in PARALLEL across plans (read-only).

  PHASE B — ROUTE on the pre-check value
    - wired=yes (already done)      -> POLISH lane only (polish agent tidies / hardens the
                                       existing implementation; no re-build from zero).
    - wired=partial / no            -> full EXECUTION FSM (Phase C).
    Record the routing decision next to the wired value.

  PHASE C — EXECUTION FSM (the orchestrate / b-plan-runner cycle, one executor at a time)
    Mirror .claude/skills/b-plan-runner/SKILL.md and .claude/skills/orchestrate, but with
    GLM-5.2 instances as the workers. The canonical order there is:
      context-compiler -> scope-planner-elai -> (integration-explorer-elai x N in PARALLEL)
      -> synthesizer-elai -> spec-grounder -> prompt-engineer -> impact-mapper -> EXECUTOR
      (ONE at a time) -> auditor -> fixer (only if auditor BLOCKs) -> wiring-contract-validator
      -> memory-writer.
    Keep that backbone. The parallelizable steps (the integration-explorers, and the
    wiring/integration validation) run concurrently; the EXECUTOR step is strictly serial.
    The executor prompt MUST carry the curated context packet that NAMES the existing files
    to edit (see "WHY THIS SHAPE" — this is what kills duplication).

  PHASE D — DONE-GATE (wiring enforcement is the real gate)
    Use .claude/rules/wiring-enforcement.md + production-caller-gate.md as the done
    definition: a real entrypoint ran the new code and left an artifact the worker did not
    author; missing evidence is FAIL. This focused gate replaces the old conductor's heavy
    stack. Keep it; it is the thing that makes autonomous output trustable.

THE TESTER (real, drives the live app — add this, it did not exist properly before)
- A real end-to-end tester RUNS elai (F:\.ELAI_workflow\.elai_cc\elai.bat) and DRIVES THE
  FRONTEND (dashboard-v2) like a user, exercising every feature, not a toy happy path.
- Building blocks already on disk: .claude/agents/dashboard-smoker.md, the visual UI loop
  tooling in .claude/rules/visual-ui-loop.md (CDP capture, ax-scaffold, ui-diff), and the
  conductor's tester_vision / CDP harness (see workflow/handoff/e2e-tester.md for what was
  proven: vision computer-use + AX-tree + SSB-journal observation).
- The tester scores features against a rubric (ELAI-TEST-CHECKPOINT-MAP.json was the old
  rubric; reuse or refresh it). Issues it finds ENTER THE QUEUE WITH PRIORITY OVER PLANS,
  and each issue follows the SAME extended FSM (pre-check -> route -> execute/polish ->
  wiring done-gate). So the tester is a continuous source of high-priority, real-defect
  work that interleaves with the plan walk.

PROMPT TEMPLATING (the one deterministic concern)
- The launcher substitutes identifiers into prompt templates: plan id (P), task/step id
  (T), phase id, the plan file path, the agent file path, and the per-plan status (the
  wired value from Phase A). Get the mapping plan->file->task exact; an off-by-one task
  number runs the wrong work silently.
- Each agent is invoked by pointing it at its own .claude/agents/<name>.md and giving it
  the task: "read task T of plan P (file: <path>) and <do your role>." The agents read
  their own definitions — do not paraphrase agent instructions into the launcher.
- Follow the dispatch-preamble convention: every agent prompt should begin with the pointer
  line from .claude/rules/agent-dispatch-preamble.md (worker git read-only, scope is a
  contract, BLOCKED = handoff entry). Sub-agents do not reliably load CLAUDE.md.

DASHBOARD WORK -> DESIGN.md (hard pointer)
- Any plan/task that builds or changes the dashboard / frontend / a visualization MUST be
  given F:\.ELAI_workflow\workflow\design\DESIGN.md as design context (the look/UX source
  of truth). Bake a reference to it into the prompt for dashboard-touching tasks. Pair with
  .claude/rules/visual-ui-loop.md (screenshot-compare loop) and design-taste.md for the
  judging lens.

AGENT ROSTER (.claude/agents/, and the role each plays here)
- context-compiler — curates the per-task context packet (THE anti-duplication step: names
  existing files to edit).
- scope-planner-elai / scope-planner — decides explore vs skip, partitions exploration.
- integration-explorer-elai / integration-explorer — finds the real entrypoint seam; names
  the PRODUCTION_HOOK. Parallelizable.
- synthesizer-elai / synthesizer — merges explorer outputs into one plan.
- spec-grounder — verifies the task's claims against the spec.
- prompt-engineer — shapes the executor prompt (budget, focus).
- impact-mapper — pre-edit blast radius.
- executor — THE implementer (one at a time). GLM-5.2.
- auditor — runs tests / the gate; sole test runner; hostile.
- fixer — targeted repair when the auditor BLOCKs.
- wiring-contract-validator — the wired/done verification (Phase A pre-check + Phase D gate).
- memory-writer — records to memory-bank after a plan passes.
- dashboard-smoker — dashboard build + svelte-check + manual launch (feeds the tester).
- designer / researcher / idea-framer / idea-critic / concept-/phase-/step-decomposer /
  babystep-author / paper-ingestor / sidecar-health / test-attributer / adversary /
  persona-* — available; use where the task calls for them (decomposition, grounding,
  anti-spiral, cross-family adversarial checks).

REUSE vs BUILD-FRESH
- REUSE: all .claude/agents/*.md; .claude/skills/orchestrate + b-plan-runner (as the FSM
  reference); .claude/rules/wiring-enforcement.md + production-caller-gate.md (the done
  gate); .claude/rules/agent-dispatch-preamble.md; the visual-ui-loop + CDP tester tooling;
  the e2e tester learnings in workflow/handoff/e2e-tester.md; DESIGN.md for dashboard work.
- BUILD FRESH: the plan-order walker (replaces the conductor DAG/scheduler — do NOT reuse
  that); the GLM-5.2 claude-code launcher + prompt templater; the per-plan wired-status
  store; the priority queue that lets tester-found issues jump ahead of plans.
- DO NOT REUSE: the conductor's scheduler, leases, merge_lane, the heavy multi-gate stack.
  The focused wiring-enforcement gate is enough when context is right.

OPEN DECISIONS FOR YOU (the builder) — not prescribed here
- HOW claude code talks to GLM-5.2 (ANTHROPIC_BASE_URL at a GLM-compatible endpoint, or the
  devin-CLI wrapper the conductor used with --model glm-5-2; confirm which actually drives a
  claude-code instance vs a devin instance, and pick the one that loads our agent files).
- WHERE the wired-or-not value lives and its schema (per-plan json vs one status file).
- The tester rubric (refresh ELAI-TEST-CHECKPOINT-MAP.json or author a new feature list).
- Commit cadence (per-plan commit vs per-phase; the house rule is one changeset per shipped
  unit — see .claude/rules/commit-cadence.md, maintainer voice, no AI trailers).
- Concurrency cap for the parallel check/explore agents (the existing house cap is 3
  parallel; >2 agents in one go is normally a confirm-first, but for this autonomous build
  the user has opted into the orchestrated fan-out — keep it sane, ~3).
- How a tester-found issue is expressed as a "task" the same FSM can consume.

NORTH STAR
The conductor failed not because orchestration is hard but because it filtered slop instead
of preventing it, and lost the plumbing in a heavy gate. This system prevents slop with good
context + "edit don't recreate" + a focused wiring gate, walks the plans in human order,
checks done-ness once up front, polishes what's already there, and lets a real tester drive
the queue. Keep it prompt-driven and simple; the agents already know how to do the work.
