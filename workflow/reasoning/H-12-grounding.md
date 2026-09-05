GROUNDING SWEEP -- phase H-12
Source: F:\.ELAI_workflow\docs\thefinalglance\H-plan\H-12_plan_research_modes_chrome.md
Date: 2026-06-04
FSM: PLANNING | Session: (spec-grounder invocation, no session_id supplied)

SPEC REFERENCES

_grounding/H-GROUNDING-MAP.md C12 + H-12 table | GROUNDED | file exists at docs/thefinalglance/H-plan/_grounding/H-GROUNDING-MAP.md; C12 entry line 45 confirms isolation gate PASSES (zero runes store imports three/troika/scene/orbit); H-12 symbol table lines 237-248 present with 6 verified symbols.

CANVAS-WORKBENCH-DESIGN.md §8 (modes catalog + frames-as-partitions + mode flag NOT FSM substate) | GROUNDED | file at .elai_cc/crates/dashboard-v2/CANVAS-WORKBENCH-DESIGN.md; §8 = lines 261-288; PLAN mode described line 268 (concept>phase>step>babystep, contains-nested, depends_on, active ids light current node, replaces OrbitMode1Plan); RESEARCH described line 269 (ResearchTreeNode + TendrilEvent leaf nodes, contradiction_with, replaces OrbitMode3Research); mode flag plain on canvas.svelte.ts confirmed line 263; frames-as-partitions confirmed line 286; mode bar hover-reveal per DESIGN.md §7 confirmed line 286.

CANVAS-WORKBENCH-DESIGN.md §12 (migration steps 3-4 + single-consumer SSB rule) | GROUNDED | §12 = lines 350-366; step 3 line 358 (port PLAN + RESEARCH modes off plan.svelte.ts + research tree); step 4 lines 359-361 (flip default to Canvas behind workspace setting; orbit stays in tab); single-consumer SSB rule lines 360-361 (active window owns subscription, inactive paused, handover atomic, never both live).

CANVAS-WORKBENCH-DESIGN.md §7 (mode flag is NOT an FSM substate) | GROUNDED | §7 = section "How the model drives the canvas" lines 213-241; CANVAS §11 line 334 explicitly: "The FSM is NOT touched. There is no CanvasDesigning FSM substate ... fsm.rs is unchanged."; grep of orchestrator/src confirms zero CanvasDesigning in fsm.rs.

CLAIM CLASSIFICATION

OrbitMode1Plan.svelte exists as orbit PLAN mode | GROUNDED | file confirmed at .elai_cc/crates/dashboard-v2/src/lib/orbit/OrbitMode1Plan.svelte via Glob.

OrbitMode3Research.svelte exists as orbit RESEARCH mode | GROUNDED | file confirmed at .elai_cc/crates/dashboard-v2/src/lib/orbit/OrbitMode3Research.svelte via Glob; reads from OrbitFixture.research_tree (ResearchTreeNode) + renders via three-orbit-modes.ts 3D path.

plan.svelte.ts has concept>phase>step>babystep shape | GROUNDED (PARTIAL DRIFT) | plan.svelte.ts uses DecompNode with level: DecompLevel ('concept'|'phase'|'step'|'baby-step') + status:'active'. The active-node identification uses status='active' on the node itself, NOT separate active_concept_id/active_phase_id/active_step_id/active_baby_step_id fields. Those four id fields are in OrbitFixture (orbit/types.ts:116-119) -- the plan.svelte.ts store does NOT expose them. CANVAS §8 line 268 cites them as the mechanism; the actual store uses status-on-node. This is a wire-level detail the executor must resolve: PlanMode.svelte reads status:'active' from DecompNode.tree, not separate active_*_id props from plan store.

The research tree has TendrilEvent sources + contradiction_with edges | VERIFY | TendrilEvent (orbit/types.ts:163-170) carries contradiction_with. ResearchTreeNode (api/types.ts:1247-1258) does NOT carry contradiction_with or TendrilEvent children. get_research_tree Tauri command (commands.rs:3866-3943) returns a flat single-node JSON (children: []) with no TendrilEvent data and no contradiction_with. The research tree store is ABSENT (no lib/state/*research*.svelte.ts file exists). ResearchMode.svelte must either: (a) build TendrilEvent + contradiction edges from SSB rows directly (new data path), or (b) read from the existing get_research_tree command (which lacks the data). This is not a speculative gap -- the Tauri command code is confirmed flat. The executor MUST decide the data source before coding ResearchMode.svelte.

CORRECTION C12 canvas-store isolation gate PASSES | GROUNDED | grep of lib/state/*.svelte.ts confirms zero three/troika/scene/orbit imports. Only match is a prose comment in grc.svelte.ts ("three-state Rust enum"). H-GROUNDING-MAP.md C12 line 45 agrees. Clean.

CANVAS §7 mode bar = flag not FSM substate | GROUNDED | CANVAS §11 line 334 explicit; grep of orchestrator/src/fsm.rs returns zero matches for CanvasDesigning or mode.*substate. H-12 DETERMINISTIC_CHECK grep command aligns.

H-12 produces NO new SSB action_type (W-9) | GROUNDED | H-12 WIRING_CONTRACT frontend_pair note: "no new Tauri command". CanvasActivityEvent already shipped as H-11 SSB variant (ssb.rs line 2986-2988, 8753). H-12 tasks 1-4 are pure frontend (Svelte + canvas.svelte.ts + workspace.svelte.ts flip). No new Rust SSB variant needed. CANVAS §13 C3.B1/B2/B3 done-gates do not cite new SSB types.

Single-consumer SSB handover is the real correctness hazard | GROUNDED | CANVAS §12 lines 360-361 specifies the invariant (never both live, handover atomic). canvasActivity.svelte.ts is confirmed second consumer (H-11, uses startPolling/stopPolling). The orbit's SSB subscription lives in ActivityWindow -> OrbitCanvas chain (confirmed shipped, workspace.svelte.ts:92 default 'activity'). Race condition: if Canvas activates BEFORE orbit subscription suspends, both poll simultaneously and double-apply into any shared store. The plan's single-consumer rule mandates suspend-then-activate order. This is correctly flagged in H-12 Open questions. No false assurance in the plan.

workspace.svelte.ts:82 default panel + :9 VALID_WINDOW_TYPES | VERIFY (LINE DRIFT) | Verified lines differ from plan's citations: VALID_WINDOW_TYPES is at line 19 (not :9); primary = makePanel('activity') is at line 92 (not :82). Content is correct -- 'canvas' IS in VALID_WINDOW_TYPES (line 19), 'activity' IS the default (line 92). Line number citations in H-12 Reads and H-GROUNDING-MAP are stale by ~10-13 lines (likely from H-11 edits adding explorer width state above the declaration). Non-blocking -- executor should re-grep before reading.

WindowHost.svelte dispatch | GROUNDED | canvas dispatches CanvasWindow at line 111-112 (confirmed in read). WindowHost is production dispatch. 'canvas' in SCOPE_LABEL line 75.

No dedicated research tree Svelte store | VERIFY (GAP) | Glob of lib/state/*.svelte.ts returns 19 files; no *research*.svelte.ts among them. The plan's consumes_symbols uses placeholder '<research tree store>' with angle brackets -- correct that it is not yet named/created. The plan treats it as a to-be-created artifact (not a currently-shipped store), which is accurate but must be explicit in executor context.

ACTIONS REQUIRED

- VERIFY before TESTING_FIRST: the research data shape. get_research_tree (commands.rs:3866) returns children:[] with no TendrilEvent and no contradiction_with. ResearchMode.svelte needs either a new/extended Tauri command that returns sources-as-children with contradiction data, OR it reads SSB rows directly. Executor must choose and document in H-12-decisions.md before coding ResearchMode.svelte. This is the highest-risk unresolved point.

- VERIFY before coding PlanMode.svelte: plan.svelte.ts exposes status:'active' on DecompNode, NOT separate active_concept_id/active_phase_id/active_step_id/active_baby_step_id props. The FSM-active node is identified by node.status === 'active'. The plan cites the OrbitFixture id fields which only exist in the orbit mock data path, not in the live plan store. Executor must use status-on-node to light the active node.

- VERIFY line numbers before reading: workspace.svelte.ts VALID_WINDOW_TYPES is at line 19 (not :9 as cited); primary = makePanel('activity') is at line 92 (not :82 as cited). Verify before offset-reads.

- NOTE for H-12-decisions.md: the research tree store does not yet exist as a Svelte store. It must be created (or ResearchMode.svelte reads from the Tauri command directly). The plan correctly uses '<research tree store>' as a placeholder, but the executor must resolve this before coding.

all other claims clear to TESTING_FIRST (spec references grounded, C12 gate confirmed, FSM untouched confirmed, single-consumer rule specified in §12, no new SSB action_type confirmed, OrbitMode1/3 files confirmed, canvas isolation gate PASSES)

VALIDATION WARNINGS

- Line number drift in H-12 plan's Reads: workspace.svelte.ts:9 and :82 are stale; actual lines 19 and 92. Minor drift from H-11 edits inserting explorer-width state above the declarations.
- H-GROUNDING-MAP.md H-12 table (lines 241) also cites WindowHost dispatch with stale line numbers from orbit era (ActivityWindow:81, PlanWindow:83 etc) -- those lines are the orbit-era dispatch offsets, not the current canvas dispatch lines. WindowHost is substantially different now (14-window dispatch at 85-113). Does not affect H-12 execution but the grounding map entry is stale for the dispatch lines.
- No spec file reference to docs/specs/guide/steps/ -- H-12 is a frontend port sourced from CANVAS-WORKBENCH-DESIGN.md and in-repo code ground truth, not from a phase-N.txt spec. This is correct: no phase-N.txt entry exists for H-12 and none is expected (H-plan is a separate planning layer). Not a gap.
