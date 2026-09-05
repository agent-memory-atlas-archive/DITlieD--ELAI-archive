# O-4: canvas WORKFLOW mode — editor + monitor over TaskSpecs, wire-dark made visual, never a runtime

STATUS: PLANNED (WAVE 3; PROOF_SPINE: none (product-surface class), owner-acked 2026-07-06; BUILD-on-shipped — the surface)

- Track: RESEARCH-MACHINE | lever: canvas-as-spec-editor | source finding: L4 | grounding: MODERATE (index-derived, PLAN-asserted) | nature: BUILD-on-shipped | effort: M | WAVE 3

- Premise: the canvas already shipped PLAN/RESEARCH modes + the mode bar (H-12), and J-31 set the doctrine (nodes = bound units, edges = verified claims, the canvas dispatches through the NORMAL FSM, there is NO canvas-side runtime), with J-60 overlaying journal-derived run state. WORKFLOW mode is that same shape over O-2's TaskSpecs: a node is a typed stage instance, an edge is dataflow carrying a provenance tier, node chrome is the last-run SSB state, and editing a node edits the spec row CONSTRAINED to the closed enum vocabulary. It is "n8n with receipts" — gated, journaled, replayable, scoped to AI-research tasks, not 400 ungated integrations.

## Steal/build
Add a WORKFLOW mode to the existing canvas mode bar. Node = a typed stage instance from O-2's closed stage vocabulary; edge = dataflow with a provenance tier (from O-1's taint labels); node chrome = the stage's last-run SSB state (J-60 overlay); editing a node = editing the underlying TaskSpec row, with the editor constrained to the enum vocabulary (you cannot draw a stage kind that does not exist as code). A node whose consumer is missing renders VISIBLY DISCONNECTED — wire-dark made visual, consistent with the dead-code-visibly-disconnected canvas idea. Dispatch stays through the normal FSM (J-31); the canvas never runs anything itself.
EXCLUDED: any canvas-side execution runtime (J-31 doctrine — dispatch through the FSM only); a free-form node whose stage kind is typed on the canvas (A3 — the palette is exactly O-2's closed enum); re-authoring the mode bar or the J-60 overlay (EXTEND, GD-1); an edge that carries data without a provenance tier.

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)
- H-12 shipped canvas PLAN/RESEARCH modes + the mode bar (feature-map canvas entries).
- J-31 canvas doctrine: nodes = bound units, edges = verified claims, dispatch through the normal FSM, NO canvas-side runtime.
- J-60 overlays journal-derived run state onto canvas nodes (last-run status chrome).
- O-2 supplies the TaskSpec rows + closed stage/trigger/sink enums; O-1 supplies the taint provenance tiers on dataflow.

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1): the WORKFLOW mode flag on the mode bar + rendering a TaskSpec as nodes/edges + the J-60 last-run chrome + the visibly-disconnected rendering for a missing consumer + read-only inspection.
- DEFER (Tier-2): the in-canvas node EDITING that writes back to the TaskSpec row (write-back crosses into O-2's spec store — land after O-2 Tier-1 is wired) + edge-level provenance-tier drill-down.

## Integration
- crates/dashboard-v2 (Svelte 5 + Tauri v2): a WORKFLOW mode in the canvas mode bar; a TaskSpec->nodes/edges projection; consume the J-60 overlay for chrome.
- frontend_pair: this IS the frontend; the Tauri read is a `list_task_specs` / `get_task_spec_runstate` command over O-2's store. cross-lang pair = the Rust command + the Svelte view.

## MOVES (wargame)
M1 add WORKFLOW to the canvas mode bar (extend H-12's mode enum) and project a TaskSpec into nodes (stage instances) + edges (dataflow).
EXPECTED_OBSERVATION: selecting WORKFLOW renders a configured TaskSpec as a node-per-stage graph; the node palette offers exactly O-2's closed stage kinds and no others.
ON_FAIL: the palette offers a free-form node -> the canvas invented a stage vocabulary -> bind the palette to O-2's enum at build so it cannot list a non-code stage (A3).
M2 overlay each node's last-run SSB state as chrome (J-60) and render a node whose consumer is missing as visibly disconnected.
EXPECTED_OBSERVATION: a stage that ran shows its last-run status; a stage whose downstream consumer is absent renders with the disconnected treatment, not a normal edge.
ON_FAIL: a wire-dark node renders as normally connected -> the missing-consumer detection is not wired to the render -> derive disconnection from the TaskSpec edge set, add the visual-disconnect assertion.
M3 keep all dispatch on the FSM: a "run now" affordance enqueues through the normal FSM path, never a canvas-side executor.
EXPECTED_OBSERVATION: triggering a spec from the canvas produces the SAME journaled task_spec_fired SSB row O-2 emits from the scheduler; no canvas-owned execution.
ON_FAIL: the canvas runs the pipeline itself -> J-31 doctrine violated -> route the affordance to the O-2 dispatcher entrypoint, assert the fire-row provenance shows the FSM path.

## FORK
FORK: if O-2's TaskSpec store is not yet queryable from the dashboard (O-2 Tier-1 not wired) -> ship O-4 Tier-1 against a read-only fixture projection and gate the live wiring on O-2; do not stub a canvas-side spec store.

## RECON_NEEDED
RECON_NEEDED: the H-12 mode-bar mode enum + how a new mode is registered -> settle via `grep -rn "mode.bar\|CanvasMode\|PLAN\|RESEARCH" .elai_cc/crates/dashboard-v2/src` + read the mode enum BEFORE editing.
RECON_NEEDED: the J-60 overlay API (how last-run state binds to a node) -> settle via `grep -rn "J-60\|runstate\|overlay\|node.*status" .elai_cc/crates/dashboard-v2/src`.
RECON_NEEDED: O-2's Tauri read surface for specs -> settle by confirming the `list_task_specs` command exists (O-2 deliverable) or naming it as O-4's required O-2 input.

## ABORT
ABORT: if adding WORKFLOW mode would require ANY execution logic on the canvas side (a scheduler, a stage runner) -> stop and flag; J-31 is absolute — the canvas is an editor + monitor, dispatch goes through the FSM only.

## WIRING_CONTRACT
produces_symbols:
  - the WORKFLOW canvas mode + the TaskSpec->nodes/edges projection
  - the enum-constrained node palette (bound to O-2's closed stage vocabulary)
  - the visibly-disconnected rendering for a missing-consumer (wire-dark-made-visual)
  - the Tauri `list_task_specs` / `get_task_spec_runstate` read commands
consumes_symbols:
  - H-12 mode bar + canvas modes (EXTEND, do not re-author — GD-1)
  - J-31 dispatch-through-FSM doctrine, J-60 journal-run-state overlay
  - O-2 TaskSpec store + closed enums, O-1 taint provenance tiers
reachable_from:
  - a user selects WORKFLOW mode in the live dashboard and inspects a real TaskSpec  [PRODUCTION canvas mode]
  - a #[cfg(test)] / component-test caller NEVER satisfies this (R7)
stub_ban:
  - NO canvas-side execution runtime (J-31)
  - NO free-form node stage kind typed on the canvas (A3)
  - NO edge without a provenance tier
  - NO re-authoring of the mode bar or the J-60 overlay
frontend_pair: the Svelte WORKFLOW view + the Rust list_task_specs/get_task_spec_runstate Tauri commands (cross-lang pair; ts-rs bindings must not drift)
observable_effect:
  trigger: "launch the dashboard, select WORKFLOW mode, load a TaskSpec with a deliberately missing downstream consumer"
  expect_ssb: "selecting run-now emits the O-2 task_spec_fired row via the FSM path (no canvas-owned execution row)"
  expect_artifact: "the spec renders as nodes+edges with last-run chrome; the missing-consumer node renders visibly disconnected; NEGATIVE: a canvas-side execution path, or a normally-connected wire-dark node, must FAIL"

PRODUCTION_HOOK: the canvas mode bar registers WORKFLOW (H-12 mode enum seam) and the dashboard route renders it; the run-now affordance calls O-2's TaskSpec dispatcher (not a new path) — wired in the SAME changeset.

## VERIFY
VERIFY: the auditor runs, off-agent:
`python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/O-plan/O-4_canvas_workflow_mode.md --run-trigger`
`python .claude/tools/svelte_mount_reachability.py` and `python .claude/tools/tauri_ipc_symmetry_check.py`   # WORKFLOW view reachable; list_task_specs invoke<->handler symmetric
`bun .claude/tools/visual/ui-wire-smoke.mjs ...`   # a real click on WORKFLOW mode lands a Rust-owned SSB row
pass = the WORKFLOW view is mount-reachable, the Tauri command is symmetric, and a live click drove the FSM dispatch (off-agent SSB row), not a canvas-side run.

INTEGRATION_SMOKE: an independent agent launches the real dashboard, selects WORKFLOW mode on a TaskSpec whose pipeline spans O-1 stages with one missing consumer, triggers run-now, and asserts (a) the visibly-disconnected node, (b) the O-2 fire-row via the FSM, (c) J-60 chrome updating — a live browser run through the real entrypoint, not a component test (R10).

## REDTEAM
REDTEAM: attack that FAILED — "smuggle an execution engine into the canvas so workflows run client-side and skip the gates": FAILS because J-31 is a stub_ban (no canvas runtime) and run-now routes to O-2's FSM dispatcher, so every fire is a journaled SSB row through the gated path. Attack that SUCCEEDED and its patch — "hide a wire-dark stage by rendering its missing consumer as a normal edge": patched by deriving disconnection from the TaskSpec edge set and asserting the visibly-disconnected treatment in the smoke (M2), so wire-dark is always visual.

## Grounding
- design digest workflow/reasoning/elai-personal-research-machine-2026-07-06.md L4 + A3 + the n8n delta (docs.n8n.io: directed node graphs, cron/webhook triggers, arbitrary ungated integration nodes).
- feature-map H-12 canvas modes + mode bar, J-31 canvas-system-composition doctrine, J-60 observability entries.

## COMPOSES-WITH
H-12 mode bar (EXTEND), J-31 FSM-dispatch doctrine, J-60 run-state overlay, O-2 TaskSpec store + enums, O-1 provenance tiers. Re-implements NONE (GD-1). The canvas is editor + monitor; dispatch is the FSM; wire-dark is rendered, never hidden.

End of O-4.
