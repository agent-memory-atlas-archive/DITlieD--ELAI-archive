H-12 DECISIONS
Phase: PLAN + RESEARCH modes + shared chrome; canvas-as-default
Session: s-10612 (2026-06-04)
FSM: EXECUTING

1. MODE FLAG IS A PLAIN TS FLAG (NOT FSM SUBSTATE)

Decision: CanvasMode is a string union on canvas.svelte.ts ('code'|'plan'|'research'|'arch'|'design').
fsm.rs is UNTOUCHED.

Rationale: an FSM substate would create compound states (AUDITING + design_mode_active, etc.)
whose transitions are unspecified in fsm.rs, requiring O(modes * FSM_states) new transition rows.
A plain flag gates only which op-namespace is writable in the CanvasDispatcher (H-13), not the
agent's lifecycle. Source: CANVAS-WORKBENCH-DESIGN.md §7 and §11.

arch and design modes are included in the CanvasMode type now (with wired: false in CanvasModeBar)
so downstream code (H-13 dispatcher, H-14 design mode) can type-check against the full union
without a type change. The CanvasModeBar shows them dimmed until wired.


2. SINGLE-CONSUMER SSB HANDOVER DESIGN

Decision: canvas.svelte.ts activateCanvas() / deactivateCanvas() are called from
CanvasWindow.svelte onMount / onDestroy. Order is suspend-then-activate:
  activateCanvas(): stopPolling() first (idempotent), then startPolling().
  deactivateCanvas(): stopPolling() only.

The orbit (ActivityWindow -> OrbitCanvas) has NO separate SSB poller. It reads plan.tree
(plan.svelte.ts 5s poll, always-on, unaffected) and fsm.state reactively. Only canvasActivity
(H-11 live overlay, 2s poll via setInterval) needs arbitration.

Result: when the canvas tab is active, canvasActivity polls. When the orbit tab is active,
canvasActivity does NOT poll. The plan.tree poll is unaffected in both cases.

Race prevention: stopPolling() before startPolling() (never activate-then-suspend).
If activateCanvas() is called twice without deactivateCanvas(), startPolling() is idempotent
(H-11 startPolling returns early if pollerHandle !== null).


3. RESEARCH DATA REALITY — REAL vs STUBBED

3a. REAL (wired in H-12):
  - researchData.researchTree from drawers.svelte.ts is the live research tree.
    It polls get_research_tree() and listens to 'research-progress' + 'research-done'
    Tauri events for incremental live updates.
  - Fields available: id, query, stage, children, source_count, converged, phi, psi,
    stage_events. All rendered in ResearchMode.svelte.
  - Data is real backend data from crates/orchestrator/src/research_tree.rs.
  - ResearchMode bootstraps researchData on mount and tears it down on destroy.

3b. STUBBED (registered, not faked):
  - contradiction_with edges (red lines between contradicting sources):
    contradiction_with lives on TendrilEvent (orbit/types.ts:169). TendrilEvent is
    orbit-fixture-only data; get_research_tree (commands.rs:3866) returns children:[]
    with no TendrilEvent payloads. The real research pipeline does not yet emit
    source-level contradiction signals to the frontend.
    Stub marker: TODO[H-12.research.contradiction]
    will_resolve_in: H-13 (canvas op dispatch) or a dedicated research-wiring slice.

  - cites edges (source-to-node leaf connections): ResearchTreeNode has source_count
    but no source URL list. TendrilEvent carries source_url + tier but only in
    orbit fixture data.
    Stub marker: TODO[H-12.research.cites]
    will_resolve_in: same as contradiction.

Decision: DO NOT render fabricated/mock contradiction edges. Stub them honestly with
in-code markers per wiring-enforcement.md (no silent fakes). The stub-note is visible
in the ResearchMode.svelte empty state so the user sees what is deferred.


4. CANVAS-AS-DEFAULT FLAG — PROMOTION PATH (W-2)

Decision: canvasAsDefault is a boolean in workspace.svelte.ts persisted under
CANVAS_DEFAULT_KEY = 'elai.dashboard-v2.canvas-default.v1' (SEPARATE from
STORAGE_KEY 'elai.dashboard-v2.workspace.v1').

Separate key rationale: reusing STORAGE_KEY would require bumping the WorkspaceSerialized
schema to include the new field, which would invalidate all existing persisted workspaces
(the isValidSerialized check would reject them). A separate key is orthogonal and
schema-safe — no old session is broken.

Promotion path (W-2):
  H-12: flag is default-OFF (canvasAsDefault = false). Canvas window available as a tab.
  H-16 (orbit retire): after one full release with canvas-as-default shipping with zero
  regressions, the flag is promoted to default-ON and CANVAS_DEFAULT_KEY is deprecated.
  The orbit tab is retired in H-16 after that release.

The flag can be toggled by the user via settings (wire via SettingsWindow in H-13+).
setCanvasAsDefault(true) applies canvas as primary panel immediately (if userCustomized
is false — respects user layout choices).


5. PLAN MODE DATA — ACTIVE-NODE VIA STATUS

Decision: PlanMode.svelte reads status === 'active' from DecompNode (plan.svelte.ts).
It does NOT use separate active_concept_id / active_phase_id / active_step_id /
active_baby_step_id props — those exist only in OrbitFixture (orbit/types.ts:116-119),
which is fixture/mock data that the orbit uses. plan.svelte.ts exposes the raw DecompNode
tree; the active node is identified by node.status === 'active' on the node itself.

Source: H-12-grounding.md §"plan.svelte.ts has concept>phase>step>babystep shape":
"CANVAS §8 line 268 cites them [the id fields] as the mechanism; the actual store uses
status-on-node. This is a wire-level detail the executor must resolve."


6. RESEARCH STORE LOCATION

Decision: there is NO separate state/research.svelte.ts. The existing researchData
export from state/drawers.svelte.ts IS the research tree store. It was built in B-7
and has a full lifecycle (bootstrap/teardown with refcounting, real event listeners,
incremental applyProgress, post-done refresh from get_research_tree).

ResearchMode.svelte imports directly from state/drawers.svelte.ts. No new store created.


7. CROSS_LANG_DIFF

H-12 is pure frontend (Svelte + TypeScript stores). No new Tauri commands added.
cross_lang_diff.py PASS confirmed (no cross-language boundary crossed).
The frontend consumes existing commands (get_decomposition_tree, get_research_tree,
get_canvas_activity — all pre-existing). W-4 PASS.
