Agent spawning (HARD RULE)

Loaded by: ALWAYS (any task that may delegate to an agent).

Core protocol

NEVER read .claude/agents/*.md yourself. NEVER ad-lib agent instructions. Pass the agent file path ({project_root}/.claude/agents/<name>.md) as the prompt; the agent reads its own definition. Only provide: task, session_id, required parameters.

All agents run on model=sonnet by default. Override only if the agent file's frontmatter specifies otherwise (e.g. memory-writer.md uses haiku).

Canonical agent list (do NOT guess names — every name below maps to a real file at .claude/agents/<name>.md)

  auditor.md             — adversarial code/plan reviewer; sole test runner via xcodebuild + XCTest in AUDITING/RUN_TESTS sub-state
  context-compiler.md    — reads memory-bank, writes context-package.md with blast radius + interface map
  scope-planner.md       — partitions a HEAVY task into N integration subsystems; emits exploration-plan.md
  integration-explorer.md — runs in parallel (one per subsystem); writes per-subsystem wiring file under workflow/sessions/{id}/exploration/
  synthesizer.md         — aggregates N exploration files into a WIRING section appended to context-package.md
  prompt-engineer.md     — turns raw user task + context-package.md into a structured implementation prompt
  paper-ingestor.md      — ingests papers (mode=paper) or source/SDKs (mode=source) into RAG; emits interface map
  memory-writer.md       — sole writer of memory-bank/ files; runs in MEMORIZING; uses mcp_log_change / mcp_memorize / mcp_update_context
  designer.md            — RETIRED. Replaced by idea-intake skill orchestrating the chain of specialized agents below. The file is kept as a redirect; do not spawn it.
  idea-framer.md         — Layer A step 1. Frames the idea, identifies high-level concepts (count emerges from the idea, ceiling 12). Modes: intake | rescope | rescope-from-critic. Writes workflow/sessions/{id}/idea-frame.md.
  idea-critic.md         — Layer A step 1b. Adversarial review of the idea-framer's output. Categories: stale stack, slopsquatting risk, App Store policy, technical infeasibility, market saturation, scope mismatch, internal contradictions, hard D-rule violations, vague framing, missing required scope. Returns BLOCK / WARN / INFO. Loops with idea-framer up to 3 rounds; round 3 BLOCK escalates to user. Writes workflow/sessions/{id}/idea-critique-r{round}.md.
  concept-decomposer.md  — Layer A step 4 (per concept). Decomposes a concept into phases (count emerges from concept shape, ceiling 12). Reads idea-frame + concept's research report. ONE invocation PER CONCEPT, dispatched in parallel.
  phase-decomposer.md    — Layer A step 8 (per phase). Decomposes a phase into steps (count emerges from phase shape, ceiling 10). ONE invocation PER PHASE, parallel within concept, sequential across concepts.
  step-decomposer.md     — Layer A step 9 (per step). Decomposes a step into baby steps (count emerges from step shape, ceiling 8). ONE invocation PER STEP, massively parallel.
  babystep-author.md     — Layer A step 10 (per baby step). Writes the full 10-block detail file per BABYSTEP-TEMPLATE.md. ONE invocation PER BABY STEP, massively parallel (batches of 4). The densest agent in the chain.
  researcher.md          — tree-structured grounded research with mandatory per-mode search floors. mode=concept (deepest), mode=phase (medium), mode=oracle (deep + human-readable for /oracle skill), mode=feasibility (light, for inline checks). Source tier classification, paper-ingestor + source-fetcher integration, package registry verification, contradiction surfacing, stale-approach detection. Writes workflow/sessions/{id}/research/{node_id}.md OR workflow/sessions/{id}/oracle/{slug}.md.

If a task requires an agent NOT on this list, ask the user before proceeding. Do not invent.

When to use the Layer A intake chain (idea-framer + idea-critic + concept/phase/step/babystep decomposers + per-level auditors + researcher)

  IDEA-tier task (concept / app brief / hirer description / "build me X"): USE the idea-intake skill. See Sequencing rules > IDEA flow below for the full order.
  HEAVY-fuzzy task on an existing app (complex feature add, multi-system refactor): same chain, scoped to the feature.
  HEAVY-spec'd or FOCUSED tasks: SKIP idea-intake. Use the legacy single-plan flow in planning-workflow SKILL.

  Researcher can ALSO be spawned standalone:
  - by planning-workflow during BubbleUp ResearchAlternative actions (mode=phase or mode=feasibility)
  - by inline feasibility checks during EXECUTING (mode=feasibility, max 2 per phase)
  - by Oracle skill for /oracle and /ask commands (mode=oracle, deep + human-readable)

When to use Oracle skill

  User typed /oracle <question> or /ask <question>: USE Oracle.
  Question requires up-to-date factual grounding (current API, library state, framework version, App Store rules, recent ecosystem changes): USE Oracle.
  Main LLM uncertain about a technical claim and about to make a recommendation: USE Oracle BEFORE answering with confidence.
  Conversational / local code / workflow conventions / prior conversation context: SKIP Oracle. Answer directly.

When to use prompt-engineer (WHEN-clause)

  HEAVY-fuzzy task without idea-intake (user wants planning but not full decomposition): USE prompt-engineer BEFORE planning.
  IDEA-tier task: SKIP. The intake chain does its own briefing; per-baby-step detail files are already structured prompts.
  HEAVY-spec'd task (user provided deliverables + paths + gotchas): SKIP.
  FOCUSED-tier task: SKIP.
  SIMPLE-tier task: skip everything (no agents needed).

Judge whether prompt re-formatting would actually improve agent output. Do not over-spawn.

When to use scope-planner + integration-explorer + synthesizer (wiring discovery batch)

scope-planner runs after context-compiler in HEAVY flows. Decides EXPLORE (>=3 integration subsystems) or SKIP (<=2). On EXPLORE, dispatch N parallel integration-explorer agents (hard cap: 4, reduced from 8 for 5-hour rate limit headroom), then spawn synthesizer to aggregate. Full pattern: .claude/rules/integration-exploration.md.

Sequencing rules

  IDEA flow (Layer A, idea-intake skill orchestrates):
    regen-map.mjs (1 process)
      -> idea-framer (1 agent)
      -> idea-critic + idea-framer LOOP (max 3 rounds; loop exits on CRITIQUE PASSED or WARN-ONLY; round-3 BLOCK escalates to user)
      -> plan-auditor mode=concepts (1 agent, 1 cycle; BLOCK > re-spawn idea-framer with audit findings ONCE, no re-audit)
      -> USER CONCEPT-LIST CHECKPOINT
      -> researcher per concept (PARALLEL, batch of 4)
      -> concept-decomposer per concept (PARALLEL, batch of 4)
      -> plan-auditor mode=phases per concept (PARALLEL, batch of 4, 1 cycle each; BLOCK > re-spawn the failing concept-decomposer ONCE, no re-audit)
      -> USER CONCEPT-TREE CHECKPOINT (option to end-session-and-resume)
      -> researcher per phase where needed (PARALLEL, conditional, batch of 4)
      -> phase-decomposer per phase (PARALLEL within concept, SEQUENTIAL across concepts, batch of 4)
      -> plan-auditor mode=steps per phase (PARALLEL, batch of 4, 1 cycle each; BLOCK > re-spawn failing phase-decomposer ONCE, no re-audit)
      -> step-decomposer per step (PARALLEL, batches of 4)
      -> plan-auditor mode=babysteps per step (PARALLEL, batches of 4, 1 cycle each; BLOCK > re-spawn failing step-decomposer ONCE, no re-audit)
      -> babystep-author per baby step (MASSIVELY PARALLEL, batches of 4)
      -> plan-auditor mode=babystep-detail per baby step (PARALLEL, batches of 4, 1 cycle each; BLOCK > re-spawn failing babystep-author ONCE, no re-audit)
      -> assemble decomposition-tree.md (main LLM, no agent)
      -> auditor mode=plan (1 agent, 1 cycle, whole-tree final sanity check)
      -> USER FINAL APPROVAL
      -> hand-off to planning-workflow

  Hard caps:
    - max 12 concepts per idea (was 7; soft target 2-9, ceiling 12)
    - max 4 agents in any single parallel batch (was 8; reduced for 5-hour rate limit headroom)
    - max 200 baby steps total per plan (was 50; soft target whatever the idea decomposes to, ceiling 200 as a never-hit safety net — split into sequential plans if exceeded)
    - max 3 critic-framer loop rounds (round 3 BLOCK escalates to user via the idea-intake skill)
    - max 1 audit cycle per per-level audit invocation (no re-audit after BLOCK fix; cascades caught at the next level's audit or at the final mode=plan whole-tree audit)
    - max 2 USER CHECKPOINT rounds (concept-list, concept-tree, final = 3 fixed checkpoints; no extra)

  HEAVY flow without idea-intake (legacy):
    context-compiler -> scope-planner -> [integration-explorer ... in parallel] -> synthesizer -> prompt-engineer
    Adjacent agents SEQUENTIAL (each consumes the previous output file). integration-explorer agents within the explorer-batch are PARALLEL (one main-LLM message).

  Per-baby-step EXECUTION (Layer B, planning-workflow skill drives):
    [for each baby step in tree] auditor (pre-flight contract tests) -> executor (main LLM under FSM) -> auditor (full audit + UI check) -> memory-writer (only on concept-end roll-up)
    BubbleUp can re-spawn researcher (mode=feasibility or mode=phase) mid-cycle for ResearchAlternative actions.

  Oracle flow (interactive, any FSM state):
    /oracle <question> -> Oracle skill -> researcher (mode=oracle) -> formatted answer. NO file writes other than oracle/ artifact + cache.

  Auditor runs once per baby step (no auto-retry; BLOCKs trigger BubbleUp fix or escalation).
  Memory-writer runs at concept boundary, NOT every baby step (per-baby-step memorize is lightweight, no agent needed).
  Researcher runs at multiple stages with different modes — see researcher.md for mode-specific search floors.

Audit cycles

1 audit cycle per agent / baby step (previous 2-cycle default dropped). If audit fails after cycle 1, BubbleUp decides next action (RetryBabyStep / RedecomposeBabyStep / etc per .claude/rules/bubble-up.md). Main LLM does NOT loop audits manually — BubbleUp is the only retry pathway.

Legacy flow (FOCUSED-tier, no decomposition tree): 1 audit per phase, surface to user with remediation options on failure.

IDEA-tier flow (Layer A intake): 1 audit cycle per per-level audit invocation (mode=concepts / mode=phases / mode=steps / mode=babysteps / mode=babystep-detail). On BLOCK, the idea-intake skill re-spawns the level-N decomposer ONCE with audit findings folded in, no re-audit at this level. Cascades caught at the next level's audit, the final mode=plan whole-tree audit, or CHECKPOINT 3 final approval. Exception: the critic-framer pair at IDEA level loops up to 3 rounds before plan-auditor mode=concepts fires, because the framing layer benefits from iteration. Reference: .claude/rules/bubble-up.md PLANNING-TIME BUBBLEUP section.

Pod batching at phase boundaries (HARD RULE for asset-gen cost control)

When walking a decomposition tree, BEFORE starting the first baby step of any phase, check if the phase contains ASSET_SPEC sub-blocks. If 1+ asset-gen baby steps in the phase:
- pod_start gated by user approval (first time per session) OR auto if pod already RUNNING
- reorder phase baby steps so asset-gen baby steps run CONSECUTIVELY (burst), respecting depends-on
- pod_idle_check + pod_stop after the LAST asset baby step IF the next phase has zero ASSET_SPEC baby steps within 30 minutes
- log batching decisions to workflow/sessions/{id}/pod-batching.md

Full spec: .claude/skills/planning-workflow/SKILL.md SECTION 1 PHASE-ENTRY POD BATCHING. This rule cross-references; the orchestrator owns the implementation. Cost savings: 50%+ on pod hours for asset-heavy plans (example-scale intake with paywall + onboarding + icon + empty-states + screenshots ships ~15-25 asset-gen baby steps across 5 phases; naive walk leaves pod up ~6h, burst pattern <2h).


Pipeline next-step prep (context-compiler in background, preferred at execution time)

When the FSM is mid-tree-walk and the next baby step is known via workflow/sessions/{id}/tree-cursor.txt: spawn context-compiler for baby step N+1 with run_in_background=true WHILE the executor implements baby step N. By the time N completes AUDITING + MEMORIZING, the context-package for N+1 is already on disk. Drops wall-clock per baby step by ~20-40%, especially for HEAVY-flow baby steps that would otherwise serially wait on context-compiler at TESTING_FIRST entry.

Trigger conditions:
- HEAVY-flow execution (planning-workflow driving the tree)
- baby step N is in AUDITING or MEMORIZING (implementation done; main LLM mostly waiting)
- baby step N+1 identified per tree-cursor (no ambiguity)
- no BubbleUp event has fired since last regen (else N+1 may be stale)

When NOT to trigger:
- FOCUSED or SIMPLE tier (no pre-compute benefit)
- N+1 depends on an output from N that the executor has not yet emitted (context-package would be wrong)
- about to swap to a different concept entirely (spawn fresh at the boundary)


Context delegation rule (when main LLM CAN read directly instead of spawning an agent)

PREFER agents for plan files / memory-bank / context-package reads, especially for large multi-file or HEAVY-tier flows. Main LLM CAN read directly when:
  (a) file is small (<25k tokens) AND single-file scope
  (b) user explicitly says to read directly
  (c) verifying / debugging a specific claim
  (d) discoverable file path was just surfaced and a one-shot read avoids spawning an agent for trivial lookup

Use judgment. Do not blindly delegate or refuse.
