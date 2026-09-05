# H-12: PLAN + RESEARCH modes + shared chrome — orbit feature-parity, flat

STATUS: PLANNED (WAVE 2; BUILD/port; flips Canvas to default behind a flag)

Objective: Port OrbitMode1 (plan) and OrbitMode3 (research) to flat clipping-frame modes on the same canvas, add the
shared chrome (one zoom, one search, one inspector, one mode bar — a PLAIN flag on canvas.svelte.ts, NOT an FSM
substate), and make Canvas the default window behind a workspace flag with the SINGLE-CONSUMER SSB handover. This
reaches feature-parity with the orbit's read-only modes so H-16 can retire the orbit without losing the alive feeling.
The canvas-store isolation gate already PASSES (C12 — no runes store imports three.js), so the migration is clean.

Estimated effort: ~1-2 sessions (~8-12 h). A dashboard port + a default-window switch.
Pre-requisites: H-10 (the Canvas shell) + H-11 (the live overlay + the single-consumer concern) MUST land first.
  plan.svelte.ts + the research tree SHIPPED (the data); WindowHost.svelte (the window dispatch); workspace.svelte.ts:82 (the default panel).
Parallel-safe with: nothing in Wave 2 that touches the canvas shell / WindowHost; sequence H-10 -> H-11 -> H-12.
Source: CANVAS-WORKBENCH-DESIGN.md §8 (the modes) + §12 (migration, the single-consumer SSB rule); _grounding/H-GROUNDING-MAP.md C12 + the H-12 table. Doctrine: H-0_INDEX.md.

## Context

Modes are NOT separate screens — they are namespaced node/edge types living in their own clipping frame on the same
plane, with shared chrome (CANVAS §8). The active mode is a PLAIN flag on canvas.svelte.ts (mode in {code,plan,research,
arch,design}), NOT an orthogonal FSM substate: an FSM substate makes the FSM a product of two axes and silently creates
compound states (what happens when a mode is active AND the FSM is AUDITING) whose transitions are specified nowhere —
so fsm.rs is UNTOUCHED; the mode flag gates only which op-namespace is writable, enforced in the dispatcher (H-13), not
by an FSM transition (CANVAS §7). PLAN renders plan.svelte.ts (concept>phase>step>babystep as contains-nested nodes,
depends_on edges, the FSM-active node lit). RESEARCH renders the research tree (cites edges + red contradiction edges
between sources that disagree). Making Canvas the default requires the SINGLE-CONSUMER SSB rule (CANVAS §12 step 4):
the orbit currently owns the live SSB subscription and the canvas live-layer (H-11) is a second consumer — running both
double-applies events into a shared store, so the active window OWNS the subscription and the inactive one is PAUSED
(suspended, not just hidden); the switch hands the subscription over ATOMICALLY.

## Existing state (B/C/D/E/F/G coverage)
- The canvas-store isolation gate (lib/state/*.svelte.ts) — PASSES (C12; no store imports three.js).
- WindowHost.svelte — SHIPPED. Switch on WindowType: activity->ActivityWindow(:81), plan->PlanWindow(:83), graph->GraphWindow(:91), symbol_graph->SymbolGraphWindow(:101).
- AppShell.svelte grid — "48px 1fr 0"; rows "1fr 22px".
- tokens.css — at src/styles/ (NOT lib/); siblings base.css, motion.css.
- ui/Tree.svelte — TreeNode carries status (done/active/pending/failed); reusable for the plan tree.
- workspace.svelte.ts:82 — `primary = makePanel('activity')` (the default panel decision the switch flips). VALID_WINDOW_TYPES :9.
- plan.svelte.ts — the decomposition tree + active_concept/phase/step/baby_step ids (the PLAN data).
- the research tree (ResearchTreeNode + TendrilEvent + contradiction_with) — the RESEARCH data.
- H-11 canvasActivity overlay — the second SSB consumer the single-consumer rule must arbitrate with the orbit.

## Reads (mandatory before coding)
- _grounding/H-GROUNDING-MAP.md C12 + the H-12 symbol table.
- CANVAS-WORKBENCH-DESIGN.md §8 (the modes catalog + frames-as-partitions + the mode flag NOT an FSM substate) + §12 (migration steps 3-4 + the single-consumer SSB rule).
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\layout\WindowHost.svelte (the window dispatch) + AppShell.svelte (the grid)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\state\plan.svelte.ts (the plan data + the active ids) + the research tree store
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\state\workspace.svelte.ts:9,82 (VALID_WINDOW_TYPES + the default panel)
- CLAUDE.md HARD RULES; CANVAS §7 (the mode flag is NOT an FSM substate — fsm.rs untouched).

## Tasks (WAVE 2 — port + switch)

1. **PLAN mode off plan.svelte.ts (~160 LOC).** Render concept>phase>step>babystep as contains-nested nodes in a PLAN
   clipping frame, depends_on edges between siblings, the FSM-active node (active_concept/phase/step/baby_step ids) lit.
   Replaces OrbitMode1Plan — the same data, flat.

2. **RESEARCH mode off the research tree (~140 LOC).** Render ResearchTreeNode + children with TendrilEvent sources as
   leaf nodes, cites edges, and red contradiction edges (the contradiction_with field) between disagreeing sources.
   Replaces OrbitMode3Research — the tendrils-reaching-out feeling becomes edges fanning to source nodes.

3. **Shared chrome + the mode bar (~120 LOC).** One zoom control, one search (Cmd+F), one inspector, one breadcrumb,
   one follow-agent toggle; a segmented mode bar (hover-reveal) setting the PLAIN mode flag on canvas.svelte.ts. fsm.rs
   is UNTOUCHED. Edges never cross a frame boundary (one edge primitive, only edge_type/color differ).

4. **Canvas-as-default + the single-consumer SSB handover (~80 LOC).** Flip the workspace default off 'activity'
   (workspace.svelte.ts:82) to the canvas behind a workspace flag (orbit still in a tab). Implement the single-consumer
   rule: the active window OWNS the live SSB subscription, the inactive one is PAUSED (suspended); the switch hands it
   over ATOMICALLY (never both live — that double-applies into a shared store). This is the step-4 done-gate, not an afterthought.

5. **Decision doc workflow/reasoning/H-12-decisions.md (~100 LOC).** Records: the mode flag is a plain flag NOT an FSM
   substate (no compound states); frames-as-partitions; the single-consumer atomic handover; the isolation gate PASSES (clean port).

## WIRING_CONTRACT
produces_symbols:
  - crates/dashboard-v2/src/lib/canvas/PlanMode.svelte (the PLAN frame off plan.svelte.ts)
  - crates/dashboard-v2/src/lib/canvas/ResearchMode.svelte (the RESEARCH frame off the research tree)
  - crates/dashboard-v2/src/lib/canvas/CanvasModeBar.svelte + the shared chrome (zoom/search/inspector)
  - crates/dashboard-v2/src/lib/state/canvas.svelte.ts (the plain mode flag + the single-consumer SSB subscription arbiter)
  - crates/dashboard-v2/src/lib/state/workspace.svelte.ts (the canvas-as-default flag)
consumes_symbols:
  - crates/dashboard-v2/src/lib/state/plan.svelte.ts (the decomposition tree + active ids)
  - crates/dashboard-v2/src/lib/state/<research tree store> (ResearchTreeNode + contradiction_with)
  - crates/dashboard-v2/src/lib/layout/WindowHost.svelte (the window dispatch)
  - crates/dashboard-v2/src/lib/state/workspace.svelte.ts:82 (the default panel)
  - crates/dashboard-v2/src/lib/state/canvasActivity.svelte.ts (H-11 — the second SSB consumer to arbitrate)
reachable_from:
  - WindowHost.svelte (the canvas window) -> PlanMode/ResearchMode frames  [the modes]
  - workspace.svelte.ts:82 (the default panel) -> Canvas (when the flag is on)  [Canvas-as-default]
stub_ban:
  - NO FSM substate for the mode (a plain flag on canvas.svelte.ts; fsm.rs untouched — no compound states)
  - NO two live SSB consumers at once (the single-consumer atomic handover; the inactive window is suspended)
  - NO edge crossing a frame boundary (edges live inside a frame)
  - NO loss of the orbit fallback (the orbit stays in a tab until H-16)
  - blake3; no .unwrap()/.expect() in prod
frontend_pair:
  PlanMode/ResearchMode/CanvasModeBar (svelte). The data comes from existing stores (no new Tauri command). cross_lang_diff PASS; W-6 dashboard E2E.
observable_effect:
  trigger: "open the dashboard (NO env override); run a real plan execution + a real research run; switch the default window flag on; switch between Canvas and orbit tabs"
  expect_ssb: "the PLAN frame's FSM-active node lights from real plan.svelte.ts state (not fixture); the RESEARCH frame renders real sources + contradiction edges"
  expect_artifact: "the live decomposition tree renders flat with the FSM-active node lit; a real research run renders
    sources + red contradiction edges; Canvas is the default window (orbit still in a tab); ONLY one window holds the
    live SSB subscription at a time (verified by asserting NO double-application into a shared store when switching)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-12_plan_research_modes_chrome.md --run-trigger
test -f .elai_cc/crates/dashboard-v2/src/lib/canvas/PlanMode.svelte
test -f .elai_cc/crates/dashboard-v2/src/lib/canvas/ResearchMode.svelte
rg "mode" .elai_cc/crates/dashboard-v2/src/lib/state/canvas.svelte.ts                # the plain mode flag (not an FSM substate)
grep -rn "CanvasDesigning|mode.*substate" .elai_cc/crates/orchestrator/src/fsm.rs    # MUST be empty (fsm.rs untouched)
rg "subscription|single.consumer|pause" .elai_cc/crates/dashboard-v2/src/lib/state/canvas.svelte.ts  # the single-consumer handover
python .claude/tools/cross_lang_diff.py
test -f workflow/reasoning/H-12-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: --run-trigger PASS (PLAN/RESEARCH render from real state; Canvas is default; the single-consumer handover holds).
- [ ] W-2 default-ON: the modes + Canvas-as-default active behind the workspace flag (no env override).
- [ ] W-3 production entrypoint: the modes render from real plan/research stores; Canvas is reachable as the default window; not test-only.
- [ ] W-4: the mode frames cross-lang-clean; cross_lang_diff PASS.
- [ ] W-6: the modes + the default switch verified THROUGH the dashboard (dashboard-smoker).
- [ ] The mode is a plain flag (fsm.rs untouched — no compound states).
- [ ] The single-consumer SSB handover: only one window holds the subscription; no double-application on switch.
- [ ] cargo check --workspace GREEN + dashboard svelte-check; auditor PASS.

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE; the PLAN/RESEARCH frames (produced) consume plan.svelte.ts/the research tree (W-8); Canvas reachable as default (W-3).
- W-2: the modes + default-switch default-on behind the flag.
- W-3: WindowHost (modes) + workspace default (Canvas); not test.
- W-4: the frames; cross_lang_diff PASS.
- W-5: a runnable plan + research run + a tab switch -> real frames + a verified single-consumer handover.
- W-6: the modes + switch via dashboard-smoker.
- W-7: plan.svelte.ts / the research tree / WindowHost / the default panel / the isolation gate grep-confirmed 2026-06-03 (C12).
- W-8: the frames (produced) consume the plan/research stores; the mode flag (produced) gates the namespace; the arbiter consumes canvasActivity.
- W-9: NO new SSB action_type.
- W-10: in-process (the modes + the switch are dashboard-side).
- CANVAS §7/§12: the mode flag is not an FSM substate; the single-consumer handover is atomic.

## Tests to add
- crates/contracts/tests/h12_modes_chrome_contract.rs (~150 LOC — svelte-side via the test harness): (a) the PLAN frame
  lights the FSM-active node from plan.svelte.ts state; (b) the RESEARCH frame renders contradiction edges from
  contradiction_with; (c) the mode flag on canvas.svelte.ts gates the writable namespace WITHOUT any fsm.rs change;
  (d) the single-consumer rule: switching from orbit to Canvas suspends the orbit subscription and activates the
  canvas one, with NO double-application into a shared store (assert the event-applied count is once, not twice).

## Files touched
- CREATE: crates/dashboard-v2/src/lib/canvas/PlanMode.svelte (~160 LOC) + ResearchMode.svelte (~140 LOC) + CanvasModeBar.svelte + the shared chrome (~120 LOC)
- MODIFY/CREATE: crates/dashboard-v2/src/lib/state/canvas.svelte.ts (~80 LOC: the mode flag + the single-consumer arbiter)
- MODIFY: crates/dashboard-v2/src/lib/state/workspace.svelte.ts (~20 LOC: the canvas-as-default flag) + WindowHost.svelte (~10 LOC)
- CREATE: crates/contracts/tests/h12_modes_chrome_contract.rs (~150 LOC)
- CREATE: workflow/reasoning/H-12-decisions.md (~100 LOC)

## Citations / Grounding
- CANVAS-WORKBENCH-DESIGN.md §8 (the modes + frames-as-partitions + the mode flag NOT an FSM substate, the de-scope that removes compound-state bugs) + §12 (migration steps 3-4 + the single-consumer SSB rule, the step-4 done-gate).
- ELAI ground truth (C12): the canvas-store isolation gate PASSES; WindowHost dispatch; the default panel workspace.svelte.ts:82; the research tree contradiction_with.
- CLAUDE.md HARD RULES; CANVAS §7.

## Open questions / risks
- The single-consumer handover is the subtle part: a race where both subscriptions are briefly live double-applies; the
  handover must be atomic (suspend-then-activate, never activate-then-suspend), and the test asserts the event-applied count.
- The mode bar's hover-reveal must match DESIGN.md §7 (the orbit's mode-toggle pill already hides until hover) — pixels per DESIGN.md, architecture per CANVAS.
- Keeping the orbit in a tab until H-16 means two render paths coexist for a release; the single-consumer rule is what keeps them from fighting over the stream.

## Next
H-13 (model drives the canvas) adds the model-op transport + the dispatcher on top of these modes (ARCHITECTURE + the
op-emission grammars). H-16 retires the orbit ONLY after Canvas-as-default (this session) ships a full release with no regressions.

End of H-12.
