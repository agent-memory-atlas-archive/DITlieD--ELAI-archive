TEIKOKU — Agent-Roster Trim + Session-Orientation Substrate (redesign plan)

name: AOR (Agent-Orientation Redesign) — a focused TEIKOKU work item
status: v1.1 plan, 2026-06-22, authored from the owner's redesign brief + a cross-read of TEIKOKU-MASTER-PLAN.md, teikoku-build.md (P0-P5 as-built), teikoku-native-dashboard.md, the live ELAI .claude agent/skill roster, and the deterministic tools in .claude/tools. v1.1 addendum (owner-requested 2026-06-22): added WAVE 0 cheap ELAI ports (U-0a..U-0d) + the §4 verified-present block; all four ports confirmed on disk before authoring (codebase_mapper.py + elai.map ~205MB + elai_map_freshness.py + skill-audit.py + the rag/hooks PostToolUse change-recorder + anti-spiral SKILL.md). v1.2 addendum (owner-requested 2026-06-22): TEIKOKU becomes the best Unity agent workflow via a UNITY FLAGSHIP PACK selected by a "unity game" new-session toggle; AOR-D7..D10 + Section 9 added; pack contents grounded on the liteGAME .claude Unity ecosystem (read + inventoried this conversation) and the as-built TEIKOKU dashboard (teikoku-build.md / teikoku-native-dashboard.md). No section 0-8 decision reopened; Section 9 only ADDS the pack + the toggle.
purpose: collapse the execution agent fleet to THREE roles + the orchestrator, and give every executor full context (broad AND task-specific) that does NOT drift, by fusing git + codegraph + the event-log RAG into one deterministic Session Orientation Brief. Project-agnostic (TEIKOKU is the multi-project harness; ELAI is just the first consumer).
how to run: hand this file to a fresh Opus orchestrate session against the TEIKOKU distro (~/teikoku). Another session implements; this session only plans.
relationship to the master plan: this REFINES TEIKOKU C-02 (the per-sub-phase cycle), EXTENDS Phase 2 (the memory substrate already shipped: memory_store.py / memory_semantic.py / handoff_indexer.py / Memory.svelte), and EXTENDS Phase 3.3 (the off-agent per-pack gates). It does NOT re-open Section 0 of the master plan; where it tightens a locked decision (C-02 max-parallel), that is called out and is owner-confirmed in this conversation.

================================================================================
0. LOCKED DECISIONS (settled with the owner in the 2026-06-22 redesign session)
================================================================================

AOR-D1  THE EXECUTION CYCLE IS EXACTLY THREE AGENT ROLES + THE ORCHESTRATOR.
        ORIENTER (Opus-1M)  -> EXECUTOR (Opus-1M)  -> AUDITOR (Sonnet).
        The main LLM is the ORCHESTRATOR (never an implementer) and owns memory.
        Concurrency CAP = 3 (never more than 3 agents at once). This SUPERSEDES the
        master plan C-02 "MAX 2 in parallel" for the execution cycle (owner override,
        this conversation). The cycle is sequential per slice; the cap matters only
        when fanning out disjoint executor slices.

AOR-D2  NO MEMORY-WRITER AGENT. After an audit passes, the ORCHESTRATOR itself appends
        the change record to the topic handoff AND writes ONE event-log row per cycle;
        the already-shipped handoff_indexer auto-indexes it into the RAG (master plan
        D-08). Rationale (owner): the orchestrator already holds everything dispatched +
        returned, so it is the natural memory author; handing that to a separate agent
        is wasted indirection.

AOR-D3  THE ORIENTER IS ALSO THE PROMPT-ENGINEER, AND READS THE PRIOR HANDOFF.
        One Opus-1M agent eats a large context and emits BOTH (a) the curated broad
        orientation and (b) the structured executor prompt (files-to-read list, blast
        radius, applicable rules, acceptance). If a prior-session handoff exists for the
        topic, the orienter reads it as a first-class input (the resume spec).

AOR-D4  FROZEN GATES STAY OFF-AGENT SCRIPTS; THE AUDITOR IS A SLIM SONNET DIFF-REVIEWER.
        The deterministic gates (production_caller_check.py, python_wire_check.py,
        maintainability_check.py, wiring_contract_check.py, traced_test_assert_check.py,
        per-pack gate set) remain frozen, off-agent, fail-closed (master plan D-06,
        wiring-enforcement R8). The ORCHESTRATOR runs them and produces a verdict
        artifact. The AUDITOR (Sonnet) reads ONLY the executor's diff + that verdict
        artifact and emits a qualitative review in the vibe-coder-audit shape
        (PROGRESS SUMMARY line + BLOCK / WARN / INFO + FINAL VERDICT). The auditor does
        NOT run the heavy gates and does NOT review the whole repo.

AOR-D5  CONTEXT GATHERING IS REDUCED TO DETERMINISTIC SCRIPTS. The orienter does not
        hand-hunt files. A script ("teikoku-orient") fuses git + codegraph + the
        event-log RAG + the last gate artifact into a Session Orientation Brief; the
        orienter CURATES and synthesizes that brief into the prompt. Judgment stays with
        the agent; gathering is deterministic and cheap.

AOR-D6  PROJECT-AGNOSTIC + ANTI-DRIFT BY REGENERATION. Every DERIVED brief field is
        recomputed each run from ground truth (git / graph / event-log) and can never go
        stale. The only HAND-AUTHORED, slow-changing fields (product goal, architecture
        intent, conventions, never-change list, done-definition) live in ONE place — the
        per-project descriptor in the registry — versioned, not scattered. Migrating to a
        new project = one registry row + its graph + its event-log; zero agent rewrites.

AOR-D7  UNITY IS THE FLAGSHIP PACK + "unity game" IS A NEW-SESSION PRESET (owner-confirmed
        2026-06-22). Checking "unity game" on a new chat applies the FULL Unity stack as a
        preset over the existing per-session Skills/MCP/Rulebook machinery (NewSessionMenu +
        context_catalog + session_toggles, already confirmed honored by base-cc). One click
        = the beast; unchecked = the lean default. TOPOLOGY: a Unity game runs against its
        WINDOWS-NATIVE path, NOT copied into the ext4 distro (Unity owns file-watching;
        \\wsl.localhost over 9p is the same slow/flaky path the dashboard build already hit
        with cargo). base-cc in WSL edits the game over /mnt/<drive> and reaches the Windows
        Unity Editor + MCPForUnity bridge + the codegraph daemon over FORWARDED LOCALHOST
        (same pattern as conductor :8787). This is a deliberate carve-out from D-12 "projects
        copied into ext4"; justified, see 9.7.

AOR-D8  TWO-LAYER STACK/PROJECT SPLIT. The UNITY PACK holds STACK-level concerns reusable
        across every Unity game (rulebook trigger-set, MCP set, gate set, orient extensions,
        executor + auditor profiles). The PROJECT DESCRIPTOR holds GAME-level concerns
        (folder-ownership subtree + branch, artDirection lock pointer, per-game gotchas, the
        four static AOR-D6 fields). The toggle picks the PACK; the project picks the GAME.
        Realizes AOR-D6 with zero Unity hardcode in the engine.

AOR-D9  THE UNITY EXECUTOR NEVER HAND-WRITES SCENE/PREFAB/ASSET YAML. It mutates via
        unityMCP (manage_prefabs / manage_gameobject / manage_components /
        manage_scriptable_object / manage_material) or a Procedural Builder script; UI is
        UI Toolkit text-first (UXML/USS via Edit or manage_ui); after EVERY .cs write it
        read_console + polls isCompiling (the domain-reload barrier) before proceeding;
        verify-before-write discipline holds (clear a verify-tier deny via
        unity_reflect/unity_docs). Encoded in the orienter's Unity executor prompt (9.5).
        Hand-editing .prefab/.unity/.asset YAML is a BLOCK (fileID/GUID hallucination).

AOR-D10 THE UNITY WIRING/RUNTIME LEG IS run_tests, NOT a coverage-instrumented binary.
        Unity has no LLVM-instrumentable CLI, so the wiring-enforcement R4 "off-agent
        artifact the agent did not author" is the EditMode+PlayMode NUnit result XML from an
        AUDITOR-driven run_tests (+ a RuntimeStateExporter/AgentLogBridge dump from
        com.taporbit.agentdebug for runtime bugs, + the visual-ui-loop verdict for UI work).
        The auditor stays the sole test runner. Static legs = maintainability_check.py +
        the TOBAN001 banned-API Roslyn analyzer + the verify-before-write census + a
        MonoBehaviour-attachment wire census (a script attached to no GameObject/asset and
        referenced by nothing is wire-dark, the Unity analog of python_wire_check). The
        Unity gate pack is FROZEN per R8 so a Unity session cannot weaken its own gate.

================================================================================
1. THE PROBLEM (grounded, not hypothetical)
================================================================================

P-1  Fleet sprawl. The live ELAI harness ships 32 agent files; TEIKOKU's P1 merge
     collapsed the generic-vs-"-elai" duplicates (scope-planner / integration-explorer /
     synthesizer) but the roster still LISTS context-compiler / scope-planner /
     integration-explorer / synthesizer / memory-writer (teikoku-build.md:896). The
     owner ends up typing "use X agent" to pick among them.

P-2  Executors get raw objectives. teikoku-build.md:896 (verbatim grounded gap): "GAP
     found: chat_orchestrator was a python classifier BYPASSING the skills; workers got
     raw objectives." An executor without orientation produces shallow or wire-dark work
     — the dominant failure class this codebase already documents (wiring-enforcement;
     production-caller-gate D-030, six occurrences in one stretch).

P-3  Context drift. "What changed / what is pending / what file touches another" is today
     spread across hand-maintained markdown (changelog.md, activeContext.md,
     ELAI-FEATURE-MAP.md) that drifts the moment someone forgets to update it. TEIKOKU
     already retired the markdown memory-bank in favor of the event-log + RAG (master
     plan D-08, shipped P2), but there is no single ORIENTATION view that fuses the code
     graph with that memory and with git into the brief an executor actually needs.

P-4  Gather-by-LLM is expensive and non-reproducible. The context-compiler reads 6+
     files and greps by hand every run; two runs on the same state can disagree. The
     deterministic tools to replace most of that already exist (see §4) and are unused as
     a single orientation pipeline.

================================================================================
2. THE SESSION ORIENTATION BRIEF (the core new artifact)
================================================================================

A single generated document the orienter consumes and (curated) injects into the
executor. It realizes the owner's "Session Orientation" template, with every field
mapped to a deterministic source so it cannot drift. TWO layers, because the executor
needs both: BROAD (what is this project / where are we) and TASK (the exact change +
its blast radius).

FIELD                         LAYER   SOURCE (how it is generated, deterministic unless noted)
----------------------------- ------- --------------------------------------------------------
PRODUCT / SYSTEM GOAL         broad   registry descriptor (hand-authored once, per project)
ARCHITECTURE MAP              broad   codegraph_files + import/boundary edges + entry points
  (apps/packages, data flows,         from the graph (query_graph.py / codegraph MCP). Static
  service boundaries, entries)        "architecture intent" prose from the registry descriptor.
IMPLEMENTATION RULES          broad   registry/pack (coding + error-handling + testing
  (conventions, never-change)         conventions, never-change-casually list) — per active pack.
BRANCH STATE                  broad   git: branch, base, HEAD sha+subject, diff --stat vs base,
  (branch/base/HEAD/changed)          uncommitted+untracked set.
RECENT DECISIONS              broad   event-log + handoff RAG (memory_store / memory_semantic):
  (choices made, rejected,            decisions, rejected approaches, open TODOs. grep+semantic.
  TODOs)
WHAT CHANGED SINCE LAST       broad   event-log delta + git log since the last orientation/commit.
PENDING WORK                  broad   union of: uncommitted files (git) + in-flight/blocked tasks
  (what files are pending)            (event-log) + unshipped plan items (bn_status_report-style).
KNOWN FAILING                 broad   last gate/test verdict artifact, classified by
  (tests / type errors)               failure_taxonomy.py.
CURRENT AREA OF WORK          task    the files/symbols the task names, intersected with the graph.
  (subsystem, files, symbols)
BLAST RADIUS / TOUCHES        task    impact_radius.py + graph_reconcile.py + codegraph_impact:
  (file -> file, who calls/             downstream consumers, upstream deps, cross-language pairs
  consumes, cross-lang)               (cross_lang_diff.py). THIS is "if a file touches another one."
NEARBY PITFALLS               task    gotchas/event-log grep scoped to the touched subsystem.
ACCEPTANCE / DONE             task    the active-pack done-definition + the task's own checks.

ANTI-DRIFT INVARIANT: only the four registry fields (product goal, architecture intent,
conventions/never-change, done-definition) are hand-authored. Everything else is
RECOMPUTED from ground truth on every orient call. There is no hand-maintained
"current state" file to fall out of sync — the brief IS regenerated, never edited.

OUTPUT: a plain-text brief written to the session dir
(workflow/sessions/<sid>/orientation-<task-id>.md). The orienter reads it + the prior
handoff + the task; the orchestrator never reads it (token economy — same discipline as
the current context-package.md).

================================================================================
3. THE THREE-ROLE CYCLE (replaces the per-sub-phase fleet)
================================================================================

Every non-trivial task — a feature, a bug, "fix this", "is X wired" — runs ONE path.
No manual agent selection. (! -prefixed and pure yes/no still skip, as today.)

  1. ORCHESTRATOR runs `teikoku-orient` (deterministic) -> the Brief.
  2. ORIENTER (Opus-1M) reads Brief + prior handoff + task
        -> emits the EXECUTOR PROMPT: a BROAD ORIENTATION block (curated from the brief)
           + a TASK block (objective, files-to-read list, blast radius, applicable rules,
           acceptance). This single agent absorbs the old context-compiler +
           prompt-engineer + scope-planner + integration-explorer + synthesizer +
           impact-mapper + spec-grounder.
  3. EXECUTOR (Opus-1M) consumes that prompt -> writes code, wires the production hook in
        the same changeset, adds the unit test. It has full context: broad + task.
  4. ORCHESTRATOR runs the FROZEN OFF-AGENT GATE SCRIPTS over the diff (per active pack)
        -> a verdict artifact.
  5. AUDITOR (Sonnet) reads the diff + the verdict artifact -> qualitative review
        (PROGRESS SUMMARY + BLOCK/WARN/INFO + FINAL VERDICT, vibe-coder-audit shape).
        BLOCK -> orchestrator loops to step 3 (one fix pass). PASS -> step 6.
  6. ORCHESTRATOR writes memory itself: append the change record to the topic handoff +
        one event-log row; handoff_indexer auto-indexes (D-08). No memory-writer agent.

CONCURRENCY: cap 3. The cycle is sequential per slice; fan out only when slices have
disjoint write-sets, and never beyond 3 agents live at once.

MODELS: orienter = Opus-1M (it eats the whole brief + handoff + graph slices), executor
= Opus-1M (it must hold the broad orientation + the task + all named files), auditor =
Sonnet (diff + verdict only — a bounded input, no need for Opus).

================================================================================
4. THE DETERMINISTIC ORIENT SCRIPTS (reuse first, build the thin glue)
================================================================================

The owner's "reduce the context model to deterministic scripts, combining codegraph,
rag etc" is mostly an ASSEMBLY job — the pieces exist:

ALREADY PRESENT (reuse, generalize beyond ELAI paths):
- codegraph MCP (codegraph-elai) — live call graph: explore/callers/callees/impact.
- query_graph.py / query_bm25.py / find_bm25_new.py — graph + BM25 queries.
- impact_radius.py — blast radius via the map (--symbols/--files, --depth, --json).
- graph_reconcile.py — union codegraph + map, confidence-tagged; --mode affected.
- cross_lang_diff.py — flag changed files lacking their cross-language pair.
- failure_taxonomy.py — classify a build/test verdict into 13 categories.
- bn_status_report.py — unshipped plan-item punch list (generalize to any backlog).
- memory_store.py / memory_semantic.py — the TEIKOKU event-log + RAG (shipped P2).
- handoff_indexer.py — auto-indexes handoffs into the RAG (shipped P2, D-08).
- the per-pack gate scripts + run_gates (shipped P3.3) + gate_freeze_check.py (R8).

TO BUILD (thin, project-agnostic):
- teikoku-orient — the single brief generator. Inputs: project_id, task_text, optional
  changed-files/symbols. Internally calls git + the tools above + the registry; emits the
  §2 brief schema. NO new graph/RAG engine — only the fusion + formatting.
- registry descriptor extension — add the four static orientation fields (product_goal,
  arch_intent, conventions+never_change, done_def [already in ProjectSpec]) to
  projects.toml / plan_schema.ProjectSpec so the brief's hand-authored layer is per-project.
- gate-verdict artifact contract — formalize the off-agent gate output into one file the
  auditor reads (so the auditor never re-runs gates). Mostly a serialization of run_gates.

WAVE 0 — CHEAP ELAI PORTS (pure-script, no new design, no agent; ship before/alongside
U-1). All four artifacts were VERIFIED PRESENT on disk 2026-06-22 before this addendum was
written (anti-slopsquat: no recommended tool is hypothetical). The port cost is
PARAMETERIZATION of ELAI's hardcoded paths, never redesign — consistent with AOR-D6 (one
registry row per project). These strengthen the SAME brief / memory / security surfaces AOR
already plans; they are not new scope, they are the deterministic mechanisms those surfaces
were going to need anyway.
- GREP-MAP + FRESHNESS  (.claude/tools/codebase_mapper.py -> elai.map ~205MB +
  elai_map_freshness.py, exit 0 fresh / 1 stale / 2 missing). A grep-optimal, SCIP-enriched,
  6-col TSV (DEF/CALL/USE/IMPL_TRAIT/TYPE_REF/SCIP_REF) over rs/py/ts/svelte. This is a
  STATIC, restart-proof complement to the live codegraph MCP: it feeds the brief's
  ARCHITECTURE MAP + BLAST RADIUS deterministically AND is greppable by an agent with no live
  process running. Regenerate per project keyed by registry project_id; the freshness check
  gates the brief (stale map -> regenerate before orient).
- POSTTOOLUSE CHANGE-RECORDER  (ELAI rag/hooks/post_tool_check.py + record_evidence.py;
  CLAUDE.md "PostToolUse ... change recording to session state"). The deterministic source
  for the brief's WHAT CHANGED / PENDING fields and the handoff FILES TOUCHED — replaces LLM
  recollection. This is already the master plan J-3/J-4 intent (machine-derived FILES TOUCHED
  from PostToolUse, never recollection); WAVE 0 names the concrete hook to port and binds it
  to AOR-D2 (the orchestrator writes memory from recorded facts, not memory).
- SKILL-AUDIT GATE  (.claude/tools/skill-audit.py = the skillspector whole-directory scanner
  + skill-import-policy). HIGHER value in TEIKOKU than in ELAI because TEIKOKU imports skills
  across MANY projects. Standalone Python; near-zero port cost; run on any external SKILL.md
  import, log the row, BLOCK on HIGH/CRITICAL.
- ANTI-SPIRAL DETECTOR  (.claude/skills/anti-spiral: the 4-pattern detector
  Spinning/Oscillation/NoDrift/DiminishingReturns). Port the DETECTOR half as a deterministic
  script the orchestrator runs over the event-log (cheap, no agent); the persona-switch stays
  the single re-orient CAPABILITY (resolves O-3). The detector is the load-bearing half; the
  5 standing persona agents are exactly the sprawl this redesign kills.
ALSO STEAL (schema, not a script): the .claude/rules/conditional-routing.md trigger->file
table format becomes the per-pack manifest schema (load a rule only when its trigger is true)
— the same lazy-load discipline that keeps the brief small; folded into U-2.

================================================================================
5. AGENT ROSTER: KEEP / MERGE / CUT / SCRIPT-IFY
================================================================================

CORE EXECUTION CYCLE (the only standing execution agents — 3):
  ORIENTER  (Opus-1M)  <= MERGE of context-compiler + prompt-engineer + scope-planner(-elai)
                           + integration-explorer(-elai) + synthesizer(-elai) + impact-mapper
                           + spec-grounder. Backed by teikoku-orient.
  EXECUTOR  (Opus-1M)  <= NEW clean role. NOT the existing executor.md (that one is the
                           conductor/Devin-lane worker per the owner; do not reuse it here).
  AUDITOR   (Sonnet)   <= auditor.md SLIMMED to qualitative diff review over diff+verdict.

ABSORBED BY THE ORCHESTRATOR (main LLM, no agent):
  memory-writer            -> CUT. Orchestrator appends handoff + event row; handoff_indexer
                              auto-indexes (AOR-D2 / D-08).
  wiring-contract-validator-> SCRIPT (wiring_contract_check.py, run off-agent).
  test-attributer          -> SCRIPT (failure_taxonomy.py + the verdict artifact).
  dashboard-smoker         -> SCRIPT / pack gate step (a smoke, not an agent).
  sidecar-health           -> SCRIPT (a health check; and N-01 — TEIKOKU does not own the sidecar).

SEPARATE SUBSYSTEM (kept, but NOT in the cap-3 execution cycle):
  idea-intake pipeline     -> idea-framer/idea-critic, concept/phase/step-decomposer,
                              babystep-author, researcher, designer. This is PLAN AUTHORING
                              (master plan D-11), not the orient->execute->audit cycle. Left
                              intact here; flagged for a later, separate consolidation pass.
  paper-ingestor           -> grounding/research path; invoked on demand, not in the cycle.

ON-DEMAND CAPABILITIES (not standing agents):
  persona-* (5 anti-spiral)-> FOLD into one "fresh-eyes re-orient" capability the
                              orchestrator invokes on a stall (same trigger as the anti-spiral
                              skill). Not 5 standing agents.
  adversary                -> optional cross-family grounding capability; off the critical path.

NET: the execution fleet is 3 standing agents; the orchestrator owns memory; ~9 former
agents become scripts, capabilities, or move to the plan-authoring subsystem.

================================================================================
6. BUILD UNITS (what the implementing session ships; vertical slices per wiring R10)
================================================================================

WAVE 0 (cheap ELAI ports; pure-script, no agent, no new design; ship before/alongside U-1.
Each generalizes an existing ELAI artifact's paths per AOR-D6 — see §4 WAVE 0 for provenance.)

U-0a teikoku-map + freshness: port codebase_mapper.py -> a per-project grep-map keyed by
     registry project_id + the freshness check (exit 0/1/2). Producer (the map) AND its first
     consumer (teikoku-orient reads it for ARCHITECTURE MAP + BLAST RADIUS) in the same unit.
     Smoke: generate the map for ELAI and for the non-ELAI demo pack; the freshness check
     reports STALE after a tracked edit and FRESH after regen; no ELAI path reachable on the
     demo pack (generality gate). Brief gate: orient regenerates the map if freshness != 0.
U-0b PostToolUse change-recorder: port the rag/hooks change-recording hook to write every
     write into the TEIKOKU session event-log. Consumer in the same unit: teikoku-orient's
     WHAT CHANGED / PENDING fields and the handoff FILES TOUCHED read ONLY from recorded rows,
     never from recollection (binds AOR-D2, master plan J-3/J-4). Smoke: edit two files, run
     orient; both appear in CHANGED with zero hand-editing; the handoff FILES TOUCHED matches.
U-0c skill-audit gate: port skill-audit.py (skillspector) + the import-policy row + BLOCK on
     HIGH/CRITICAL. Smoke: a benign demo skill scans ALLOWED and logs a row; a seeded
     tests/-dir-payload skill scans BLOCKED and is refused promotion (whole-directory scan,
     not SKILL.md-only).
U-0d anti-spiral detector script: port the 4-pattern detector (Spinning / Oscillation /
     NoDrift / DiminishingReturns) as a deterministic script over the event-log; the
     orchestrator invokes the single re-orient capability when it fires (resolves O-3). Smoke:
     a seeded 3-identical-edit event sequence trips Spinning; a fail->pass->fail sequence trips
     Oscillation; no standing persona agent exists in the roster.

U-1  teikoku-orient script + the brief schema (§2, §4). Producer AND its first consumer
     (the orienter dispatch) in the same unit. Smoke: run it on a real ELAI task and a
     non-ELAI demo-pack task; both emit a complete brief, no missing field, no ELAI
     hardcode reachable on the demo pack (the master plan generality gate).
U-2  registry descriptor extension (the 4 static fields) + plan_schema validator. Smoke:
     a project with no descriptor fails closed with a clear message, not a blank brief.
     ALSO: adopt the .claude/rules/conditional-routing.md trigger->file table format as the
     per-pack manifest schema (a rule/pack-doc loads only when its trigger predicate is true),
     so the brief's IMPLEMENTATION RULES layer stays lazy-loaded and small.
U-3  ORIENTER agent definition (Opus-1M) consuming brief+handoff+task -> executor prompt
     (broad + task blocks). Smoke: the emitted prompt contains a concrete files-to-read
     list + blast radius + acceptance derived from the brief, not restated from the task.
U-4  EXECUTOR agent definition (Opus-1M) consuming the orienter prompt. Smoke: a real one-
     file change ships code + production hook + unit test; production_caller_check passes.
U-5  AUDITOR slim (Sonnet) consuming diff+verdict only -> vibe-coder-audit-shaped review.
     Smoke: a seeded wire-dark diff yields a BLOCK with the PROGRESS SUMMARY line.
U-6  gate-verdict artifact contract (§4) — orchestrator runs the frozen per-pack gates and
     serializes one verdict file the auditor reads. Freeze-check (R8) still green.
U-7  orchestrator memory step (AOR-D2): append handoff change record + one event row;
     handoff_indexer indexes it. Retire any residual memory-writer dispatch in the skills.
U-8  single entry path: every non-trivial task auto-runs orient->execute->audit (no "use X
     agent"); wire into BOTH front-ends, closing the teikoku-build.md:896 raw-objective gap.
U-9  retire the folded agents + update the orchestrating skills (orchestrate /
     auto-orchestrate / b-plan-runner) + the agent-dispatch lists to the 3-role roster.

================================================================================
7. ACCEPTANCE / DONE
================================================================================

DONE when:
- A real ELAI task AND a non-ELAI demo-pack task each run orient->execute->audit with the
  brief auto-generated (no manual agent pick), the executor receiving both broad + task
  context, the frozen gates run off-agent, the auditor reviewing only diff+verdict, and the
  orchestrator writing the handoff + event row (no memory-writer dispatch anywhere in the run).
- DRIFT TEST: change a tracked file, re-run teikoku-orient; the brief's CHANGED / PENDING /
  TOUCHES sections reflect it with zero hand-editing.
- ROSTER: <=3 standing execution agents; memory-writer absent; the folded agents are gone or
  reduced to scripts/capabilities; the orchestrating skills reference only the 3 roles.
- WAVE 0: the ported grep-map regenerates per project and its freshness check gates the
  brief; the PostToolUse change-recorder is the SOLE source of the brief CHANGED / PENDING
  and the handoff FILES TOUCHED (no LLM recollection anywhere in the run); skill-audit runs
  on every external skill import; the anti-spiral 4-pattern detector runs over the event-log
  with NO standing persona agent in the roster.
- GATES: gate_freeze_check.py green (the redesign did not weaken its own evaluator, R8).
- Every shipped surface that touches the dashboard passes the SIGNAL design + consistency
  pass (master plan C-01); but per N-03 this redesign adds no new visualization spectacle.

================================================================================
8. OPEN QUESTIONS (owner decides; none block starting U-1)
================================================================================

O-1  C-02 amendment: the master plan says "MAX 2 in parallel"; AOR-D1 raises the execution
     cap to 3 (owner-confirmed this conversation). Confirm whether to edit the master plan
     C-02 line or leave AOR as the superseding refinement doc.
O-2  idea-intake pipeline consolidation (the 8 decomposition/research agents) is OUT OF
     SCOPE here. Worth a separate trim pass? Same "fewer agents" instinct applies, but it is
     plan-authoring, not the execution cycle.
O-3  persona/anti-spiral: fold to a single re-orient capability (recommended) vs a tiny
     deterministic persona-switch script. AOR assumes the capability form.
     RESOLVED (v1.1, U-0d): the DETECTION half ships as a deterministic 4-pattern script over
     the event-log (no agent); the persona-SWITCH stays the single re-orient capability. The
     only residual fork is whether that re-orient capability is one agent or a script — left
     to the implementing session, does not block U-0d.
O-4  Where AOR runs: as a post-P5 TEIKOKU work item against ~/teikoku, OR retrofitted into
     the live ELAI .claude first as a proving ground. The owner chose "a plan for TEIKOKU";
     this doc targets ~/teikoku. If a proving run on ELAI is wanted, U-1..U-5 are portable.

================================================================================
9. UNITY FLAGSHIP PACK + "UNLEASH" TOGGLE (v1.2 addendum, 2026-06-22)
================================================================================

OWNER BRIEF (verbatim intent): TEIKOKU is the machine for Unity, the best Unity agent
workflow; a "unity game" checkbox on the new-chat menu unleashes the full Unity stack.
Grounded on the AOR pack model (sections 0-5), the liteGAME .claude Unity ecosystem (the
source spec for what the pack contains, read + inventoried this conversation), and the
as-built TEIKOKU dashboard (NewSessionMenu.svelte template-tabs + three context sections,
context_catalog.py enumeration, session_toggles -> slice_api -> _load_skill_context
preamble threading, per-session rulebook_files + mcp_servers toggles + MCP-subset
.mcp.json threading, all CONFIRMED honored by base-cc per teikoku-build.md). Decisions
AOR-D7..D10 in section 0 are the locked spine; this section is the design + build units.

9.1 WHY A PACK, NOT A REWRITE
Unity is the most demanding stack TEIKOKU will host: no instrumentable CLI binary for the
wiring runtime leg, scene/prefab/asset YAML that must never be hand-written, a
domain-reload compile barrier between every code edit and its use, and a Windows-native
Editor + bridge the WSL backend can only reach over forwarded localhost. If the AOR pack
abstraction carries Unity it carries anything, so Unity is the FLAGSHIP pack that proves
the model, exactly as ELAI is the first project consumer. The redesign already ships a
"unity DEFINE-ONLY fail-closed" pack stub (teikoku-build.md P3-C); 9.x turns that stub
into the real pack and wires the toggle that selects it.

9.2 THE TWO-LAYER SPLIT (AOR-D8)
LAYER        OWNS                                              SOURCE
------------ ------------------------------------------------- ----------------------------
UNITY PACK   rulebook trigger-set, MCP set, gate set, orient   the Unity stack (one pack,
(stack)      extensions, executor profile, auditor profile,    reused by ZC + EO + liteGAME)
             the run_tests runtime leg, the visual/UI loop.
PROJECT      folder-ownership subtree + UVCS/git branch,       the registry descriptor row
DESCRIPTOR   artDirection lock pointer, per-game gotchas,      (one row per game, AOR-D6)
(game)       the four static AOR-D6 fields, the Windows path
             + the forwarded unityMCP/codegraph endpoints.
Toggle picks the PACK; the project picks the GAME. Switching games = one descriptor row;
the pack is untouched. Zero Unity hardcode leaks into the engine (the master-plan
generality gate still applies: a non-Unity demo task must reach zero Unity pack code).

9.3 PACK CONTENTS (the beast, ported from liteGAME .claude; the toggle loads all of it)
RULEBOOK (a trigger->file manifest in the AOR U-2 / conditional-routing schema, loaded
  LAZILY per task so the brief's IMPLEMENTATION RULES layer stays small):
  - code edit  -> coding-standards, file-discipline, anti-stale-unity-csharp,
                  extensible-systems, unity-architecture, verify-before-write
  - UI / visual-> ui-responsive-practices, design-taste, visual-ui-loop, image-inspection
  - asset move -> unity-asset-hygiene, folder-ownership
  - perf path  -> performance-testing
  - bug task   -> bug-fix-protocol + runtime-debugging
  - MCP/build  -> unity-mcp-usage, unity-mcp-and-builders, unity-map-usage,
                  unity-ai-platform-decisions, experimentation-hygiene
  - asset-gen  -> gpt-image-prompting, hyperframes-video-design, canvas-image-regen
SKILLS: unity-mcp-skill, unity-scaffold, region-maps, strip-ai-footprint (+ the asset-gen
  directors banana-pro / cinema-worldbuilder + canvas-mode when the task is art-shaped).
MCP SET (threaded into the session .mcp.json subset, pointed at forwarded Windows ports):
  unityMCP (manage_* / run_tests / read_console / unity_reflect / unity_docs /
  find_gameobjects / batch_execute), codegraph (the per-game daemon), and on art tasks
  comfy-pipeline / comfyui-cwags + gpt-bridge image (GPU/remote-bound, see O-8).
GATES (frozen per R8, in gate-manifest.txt; run off-agent by the orchestrator -> verdict
  artifact the auditor reads): maintainability_check.py, the TOBAN001 banned-API Roslyn
  analyzer, the verify-before-write symbol census (unity-symbol-policy.json), the
  MonoBehaviour-attachment wire census, and for the runtime leg the auditor-driven
  EditMode+PlayMode run_tests; for UI tasks ui-diff.py + uss-taste-lint.py.
TOOLS: the Unity map (map.txt schema: lifecycle / scene_refs / singleton / reflection /
  asmdef / author columns codegraph cannot see), regenerated per game like the AOR grep-map.

9.4 UNITY ORIENT EXTENSIONS (brief fields generic git+codegraph cannot give; extends U-1)
FIELD                  LAYER  SOURCE (deterministic)
---------------------- ------ ---------------------------------------------------------------
EDITOR / BRIDGE STATE  broad  a health probe of the forwarded MCPForUnity bridge + codegraph
                              daemon; sets the fallback ladder (9.7) and whether MCP tools
                              are usable this session.
COMPILE STATE          broad  unityMCP read_console errors + editor_state.isCompiling.
UNITY WIRING GRAPH     task   map.txt: the touched type's MonoBehaviour lifecycle dispatch,
                              scene/prefab GUID refs, singleton/reflection sites, asmdef edge
                              (the wiring codegraph is blind to).
SYMBOL-POLICY RECEIPTS task   the session ledger: which version-sensitive Unity symbols
                              already have a unity_reflect/unity_docs receipt (verify tier).
ASSET-HYGIENE INVARIANTS broad surfaced into the never-change list (move-in-Editor,
                              FormerlySerializedAs, variant-depth cap 2, Force-Text serialize).
Everything else in the section-2 brief still applies; these are additive Unity rows.

9.5 UNITY EXECUTOR PROFILE (AOR-D9; the orienter bakes these into the executor prompt)
- mutate scene/prefab/asset via unityMCP or a Procedural Builder, NEVER raw YAML (BLOCK).
- UI is UI Toolkit text-first (UXML/USS); uGUI only via builder/MCP, never hand-edited YAML.
- after every .cs write: read_console + poll isCompiling before using the new symbol (the
  domain-reload barrier; a real cadence change vs generic packs that compile inline).
- verify-before-write fires pre-write; a verify-tier deny clears by unity_reflect/unity_docs.
- file-discipline caps (.cs prod 600 hard) + the production-hook-in-same-changeset rule (R10)
  are acceptance, not advice.

9.6 UNITY AUDITOR PROFILE + THE RUNTIME LEG (AOR-D10; Sonnet, diff + verdict only)
- the orchestrator runs the frozen Unity gate set off-agent and the auditor DRIVES
  run_tests (EditMode + PlayMode) as the load-bearing off-agent artifact, reading the NUnit
  result XML it did not author. This is the wiring-enforcement R4 satisfied for a stack
  with no instrumentable binary.
- runtime bugs: the auditor reads a RuntimeStateExporter/AgentLogBridge dump
  (com.taporbit.agentdebug, persistentDataPath) rather than reasoning from training priors.
- UI tasks: the auditor renders the visual-ui-loop verdict (ui-diff.py deterministic diff
  first to pick WHERE to look, then the schema-constrained pass|fail|unknown critique with
  unknown=fail), never an executor self-pass.
- static legs feed the same gate-verdict artifact (AOR U-6) with Unity rows: maintainability,
  TOBAN001, verify-before-write census, MonoBehaviour-attachment wire census.

9.7 TOPOLOGY (AOR-D7) AND THE FALLBACK LADDER
The game lives on its WINDOWS-NATIVE path (the real repo, e.g. the liteGAME root or the EO
UVCS workspace). Unity Editor + the MCPForUnity bridge + the codegraph daemon run on
Windows and own file-watching. base-cc in the WSL teikoku distro edits the game over
/mnt/<drive> and reaches the three Windows services over forwarded localhost (WSL2 forwards
localhost; the conductor :8787 path already proves the round-trip). The session .mcp.json
subset points unityMCP at the Windows host:port. NOT copied into ext4 (the D-12 carve-out:
Unity watching ext4 over 9p is the slow/flaky failure the dashboard build already hit).
CAVEATS the implementing session honors: edits over /mnt do not need the exec bit (.cs /
.uxml / .uss are fine); asset MOVES still go through unityMCP manage_asset (never raw mv,
GUID safety); the bridge link is the fragile leg, so the orient brief reports it and the
executor follows the ladder: bridge connected -> MCP tools; bridge down -> CLI batch-mode
run_tests + Edit/Write with the limitation flagged; Editor closed -> Edit/Write + surface
the fallback (never silently hand-edit YAML).

9.8 THE TOGGLE UX
A "unity game" checkbox in NewSessionMenu.svelte. Checked, it applies the Unity PACK PRESET:
auto-selects the Unity skills + the rulebook trigger-set + the unityMCP/codegraph MCP subset
in the existing three context sections, sets pack=unity (so run_gates / teikoku-orient /
the auditor select the Unity profiles), and threads the forwarded unityMCP host:port into the
session .mcp.json subset. It is a PRESET over the machinery that already exists, not a new
plumbing axis, so unchecking individual rows after presetting still works for a one-off lean
Unity session. Unchecked = today's lean default.

9.9 BUILD UNITS (WAVE U; append to section 6; vertical slices per wiring R10; each ships its
     producer AND first consumer AND smoke in one unit. Sequence after WAVE 0 + U-1..U-9.)
U-10 Unity gate pack + freeze: assemble the real pack (maintainability_check.py + TOBAN001 +
     verify-before-write census + the NEW MonoBehaviour-attachment wire census + the
     EditMode/PlayMode run_tests runtime leg + ui-diff.py/uss-taste-lint.py for UI), register
     in gate_packs + select_pack + gate-manifest.txt (R8). Consumer: run_gates selects it for
     a unity project. Smoke: a seeded banned-API .cs trips TOBAN001 BLOCK; a script attached
     to no GameObject/asset and referenced by nothing trips the wire census; freeze-check green.
U-11 Unity rulebook trigger-table manifest: port the 9.3 rulebook set as lazy-loaded rule
     docs keyed by trigger predicate (reuse the AOR U-2 schema). Consumer: the brief's
     IMPLEMENTATION RULES layer. Smoke: a UI task loads the UI rules, NOT the asset-gen rules;
     a .cs edit loads anti-stale + verify-before-write.
U-12 Unity orient extensions: extend teikoku-orient with the 9.4 fields. Smoke: orient on a
     real Unity task emits bridge-connected + compile-clean + the touched MonoBehaviour's
     scene attachments + blast radius from map.txt, with zero hand-editing.
U-13 Unity executor profile: the orienter emits the 9.5 executor prompt. Smoke: a real
     one-component change ships via manage_components/manage_script, compiles clean
     (read_console verified), and the unit test attaches; zero raw YAML written.
U-14 Unity auditor profile + runtime leg: the auditor drives run_tests as the off-agent
     artifact + reads the gate-verdict + the visual-ui-loop verdict on UI tasks. Smoke: a
     wire-dark MonoBehaviour yields a BLOCK; a seeded UI regression yields a visual-loop fail.
U-15 the "unity game" toggle (9.8): the NewSessionMenu checkbox applies the Unity pack preset
     + threads the forwarded unityMCP endpoint. Producer (toggle) + consumer (session boots
     the full Unity stack) one unit. Smoke: one click loads rules+skills+MCP+pack; unchecked
     = lean default; base-cc receives the Unity preamble + reaches the Windows bridge.
U-16 Unity project descriptor + topology proof: register a Unity game (recommend liteGAME
     first, O-7) as a descriptor pointing at its Windows-native path + the four static fields
     + the Unity stack fields + the forwarded endpoints. Smoke: orient -> execute -> audit a
     real task end-to-end with base-cc(WSL) editing /mnt/<drive> and driving Windows
     unityMCP + codegraph; run_tests result XML lands as the off-agent artifact.

9.10 ACCEPTANCE (append to section 7)
- Checking "unity game" on a new chat loads the full Unity stack (rules + skills + MCP +
  gate pack + orient/executor/auditor profiles) as ONE preset; unchecked stays the lean
  default with no Unity code reachable.
- A real Unity task that includes BOTH a .cs change AND a UXML/USS change runs
  orient -> execute -> audit with: bridge state in the brief, the executor mutating via
  unityMCP/builders (zero raw scene/prefab/asset YAML), the auditor driving run_tests as the
  off-agent artifact AND rendering the visual-ui-loop verdict on the UI change.
- The Unity gate pack is frozen (gate_freeze_check green); a Unity session cannot weaken its
  own Unity gate (R8).
- TOPOLOGY proven: base-cc in WSL edits the Windows-native game path over /mnt and reaches
  Windows unityMCP + codegraph over forwarded localhost; no Unity project copied into ext4.
- GENERALITY held: a non-Unity demo task in the same run reaches zero Unity pack code (the
  master-plan generality gate; the pack is selected, never hardcoded into the engine).

9.11 OPEN QUESTIONS (append to section 8; none block U-10)
O-5  Unity project location confirmed Windows-native (AOR-D7); the implementing session
     formalizes the .gitignore / copy-manifest carve-out from D-12 (the game is referenced
     by path, not copied; the map + grep-map + git all run against the Windows path).
O-6  bridge fragility: the WSL->Windows forwarded-localhost link to unityMCP/codegraph is the
     one fragile leg. The fallback ladder (9.7) is specified; confirm whether a bridge-down
     run is allowed to PROCEED with the CLI batch-mode run_tests leg or must STOP and surface.
O-7  proving game: liteGAME (solo, simplest, no per-dev branch constraints) vs ZC vs EO
     (multi-dev, maintainer-subtree + maintainer-branch constraints from folder-ownership). Recommend
     liteGAME for U-16, then EO once the per-game descriptor proves the branch/subtree fields.
O-8  asset-gen loops (ComfyUI remote 3090 / comfy-pipeline / comfyui-cwags / region-maps /
     gpt-bridge image) are in the full-beast preset but are GPU/remote-bound and slower to
     wire. Confirm they ship in pack v1 or as a WAVE U-2 second pass; the code/UI/test legs
     (U-10..U-16) do not depend on them.
