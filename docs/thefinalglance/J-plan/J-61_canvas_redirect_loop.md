# J-61: canvas redirect loop — user edits are journaled intents, the LLM enacts, breakage shows deterministically, the user preempts

STATUS: PLANNED (WAVE CFW-4; tier L; EXTENSION — the steer half of the layered canvas; depends J-60)

- Track J | lever STEERING+CANT-WRECK | grounding STRONG (single-writer journal + canvas read-only substrate code-verified; agent-control leg composes PLANNED J-12) | nature EXTENSION | effort L | WAVE CFW-4 | depends: J-60 (process/system layers + emit), H-15 (deixis read-back, shipped), J-31 (system edges + wiring verdict), J-12 (intervention, PLANNED — agent-control leg), H-49 (reversibility, PARTIAL — preempt rollback)

## Premise

J-60 makes the layered canvas a thing you WATCH (emit-only, read-only). J-61 makes it a thing you
STEER without it becoming Langflow. The whole category plateaus because manual node-wiring is more
work than typing (web-verified 2026-06-20). So J-61 does NOT add hand-authoring: new nodes are still
born in chat (J-32 front door). What it adds is a small, closed verb set that operates on EXISTING
nodes, where every user edit is a journaled INTENT command to the single-writer engine (never a
direct canvas mutation), the LLM translates the intent across altitudes, and the deterministic
checker turns affected edges red on its own when a reorder/cut breaks a wiring contract. The user
keeps the wheel: a redirect PREEMPTS an in-flight LLM action on the same node. This is what turns the
projection into a bidirectional negotiation surface for the things prose is bad at — order,
parallelism, "this depends on that." Grill decisions locked in workflow/reasoning/J-61-decisions.md.

## Steal/build (5 legs)

1. THE CLOSED VERB SET (grill decision; the canvas does ONLY these). CONNECT(a,b) — draw an edge;
   the LLM proposes the wiring contract, the J-31 checker derives the verdict (the user's original
   "connect and it wires itself"). CUT-EDGE(e) — decouple intent. REORDER(node, before/after) — a
   re-plan intent over the PROCESS/SYSTEM order. REBIND(node, new binding) — re-target intent.
   AGENT-CONTROL(agent, pause|redirect|kill) on the per-agent tab. New-node creation is REJECTED as a
   canvas verb (chat emits nodes — the chat-primary thesis; do NOT cross into free-form authoring).
   No node-delete verb (removal is a chat/lifecycle action; the journal is append-only).
2. EDIT = JOURNALED INTENT COMMAND, NOT A MUTATION (the invariant-preserving keystone). The canvas is
   read-only on SSB (SsbViewer WAL poll); a redirect is submitted as a CanvasRedirectIntent command
   to the engine (the single writer), which appends it. There is NO write race by construction — the
   single-writer append-only journal (phase-3B SHIPPED) serializes everything into a total order by
   sequence; the canvas re-derives. (This corrects the earlier mis-citation of I-6 PrivilegedAnchor
   as a concurrency model — I-6 is a context-pane ordering slot; the real model is the SSB total
   order. See J-61-decisions.md.)
3. CROSS-ALTITUDE TRANSLATION + DETERMINISTIC BREAKAGE. The LLM translates an intent across layers:
   a PROCESS-layer REORDER is a re-plan that re-derives the SYSTEM dependencies; affected edges flip
   to broken ON THEIR OWN via the J-31 verdict ladder + the Kahn cycle detection already in
   hierarchical plan mgmt (phase-3G) when the reorder creates a cycle or violates a wiring contract.
   The user sees breakage spatially and instantly, with no run. The LLM proposes; the checker/journal
   adjudicate the verdict (TD-3) — the LLM never colors an edge.
4. USER PREEMPTS (grill decision; the collision policy). When a pending user intent and a
   not-yet-committed LLM action target the same node, the user intent supersedes the LLM action. It
   rides the shipped reversibility substrate — H-49 RV-1 snapshot-before-restore for graph/edit
   state, and TaskStop / the J-12 intervention surface for a running agent. The LLM, on seeing a
   user-intent row, re-derives and adapts its next emit. No locks, no CRDT (there are two writers and
   a total order already).
5. BUILD STAYS FSM (the category line, inherited from J-31). A node's build dispatches its bound unit
   through the NORMAL planning-workflow FSM — no canvas-side runtime. A redirect changes the plan/
   order/binding the FSM then executes; it never executes anything on the canvas.

## Existing ground truth (verified via J-60 + J-31 anchors; re-grep at edit, W-7)

- SSB single-writer append-only WAL + sequence ordering + broadcast-after-commit SHIPPED (phase-3B,
  crates/orchestrator/src/ssb.rs); dashboard SsbViewer is READ-ONLY (canvas cannot write state).
- CanvasUiAction @ canvas_ui_action.rs:61 (H-15 deixis-select + G-1 taint-gated read-back, shipped) —
  the gesture a redirect attaches to; InProcessCanvasDispatcher @ canvas_dispatcher.rs:101 (the
  command intake seam; the redirect ops are a new namespace on it).
- J-31 system_canvas.rs verdict ladder (declared->wired->verified->broken, checker/journal-derived) —
  CONNECT/CUT reuse it; do NOT re-implement verdicts.
- Kahn cycle detection in hierarchical plan mgmt (phase-3G SHIPPED) — the REORDER breakage check.
- H-49 RV-1 snapshot-before-restore + EditHistoryEvent (226) PARTIAL — the preempt rollback path.
- J-12 runtime intervention harness PLANNED — the AGENT-CONTROL leg (pause/redirect/kill) composes
  it; until J-12 ships, agent-control is limited to the shipped TaskStop-equivalent (kill) and the
  pause/redirect arm is Tier-2.
- I-6 ANCHOR PANE SLOT = context-pane ordering (PLANNED), NOT a concurrency primitive — earlier
  mis-citation corrected.

## REVISED scope (W-10 tiered)

Tier-1 (ships first): CONNECT + CUT-EDGE + REORDER + REBIND as CanvasRedirectIntent commands;
cross-altitude re-derive + deterministic edge-breakage; user-preempt on graph-structure edits via
H-49 RV-1; AGENT-CONTROL=kill via the shipped stop path.
Tier-2 (gated): AGENT-CONTROL pause/redirect via J-12 when it ships; preempt of a long in-flight LLM
generation mid-token (needs the H-49 turn-grain rollback wired); multi-user concurrent redirect
(single-user assumed v1 — the total order already covers it, but the privileged-actor tie-break
generalizes when seats land, J-49).

## Integration

- frontend: redirect gestures (drag-connect / cut / reorder / rebind / tab agent-control) on the
  J-60 surface emit CanvasRedirectIntent via the dispatcher; broken edges render the J-31 broken
  state. frontend_pair = the redirect interactions + the Tauri command. cross_lang_diff +
  svelte-check in-unit.
- dispatcher: NEW 'redirect' op-namespace on InProcessCanvasDispatcher (canvas_dispatcher.rs:101):
  connect / cut_edge / reorder / rebind / agent_control, via the existing 3-arm tool pattern
  (catalog + convert + dispatch, production_agent_rpc.rs).
- orchestrator: NEW crates/orchestrator/src/canvas_redirect.rs (~240 LOC): CanvasRedirectIntent
  command type + the single-writer append + cross-altitude translation (re-plan / re-bind) + the
  preempt arbiter over the SSB sequence + the J-31 verdict re-derive trigger. The FSM dispatch path
  is REUSED for build; the redirect never executes.
- SSB: +CanvasRedirectIntent (Pattern-2 single-variant; Kind: Connect / CutEdge / Reorder / Rebind /
  AgentControl). baseline +1 from LIVE 241/550 (re-read at edit, W-9; golden-lock 9-tuple lockstep).
  The preempt outcome reuses the H-49 EditHistoryEvent rollback row (no second action_type).

## WIRING_CONTRACT

produces_symbols:
  - crates/orchestrator/src/canvas_redirect.rs — CanvasRedirectIntent command + single-writer append + cross-altitude translation + preempt arbiter
  - the 'redirect' dispatcher op-namespace (connect / cut_edge / reorder / rebind / agent_control)
  - the redirect gesture handlers on the J-60 canvas surface
consumes_symbols:
  - crates/orchestrator/src/ssb.rs (single-writer append-only WAL + sequence; phase-3B) — do NOT add a second writer
  - crates/orchestrator/src/system_canvas.rs (J-31 verdict ladder) — CONNECT/CUT reuse it; do NOT re-implement verdicts
  - crates/orchestrator/src/canvas_dispatcher.rs:101::InProcessCanvasDispatcher — command intake seam
  - hierarchical plan mgmt Kahn cycle detection (phase-3G) — REORDER breakage check; do NOT re-implement
  - H-49 RV-1 snapshot-before-restore + EditHistoryEvent (file_ops/session_dag) — the preempt rollback
  - J-12 intervention surface (PLANNED) — AGENT-CONTROL pause/redirect; kill reuses the shipped stop path
  - crates/orchestrator/src/canvas_ui_action.rs:61 (H-15 deixis) — the gesture a redirect attaches to
reachable_from:
  - crates/dashboard-v2 canvas redirect gestures -> InProcessCanvasDispatcher 'redirect' namespace -> canvas_redirect.rs (production_agent_rpc.rs dispatch) [PRODUCTION canvas]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO direct canvas-side graph mutation (every edit is a journaled CanvasRedirectIntent command to the single writer)
  - NO LLM-set edge verdict (checker/journal only, TD-3); NO canvas-side runtime / data-flow execution (build = FSM dispatch, the J-31 category line)
  - NO ADD-NODE or DELETE-NODE canvas verb (new nodes via chat; removal via lifecycle)
  - NO lock / CRDT (the SSB total order is the conflict model); NO second SSB writer
  - NO .unwrap()/.expect() on prod paths; blake3 only
frontend_pair:
  the redirect gestures on the J-60 surface (drag-connect / cut / reorder / rebind / tab agent-control) + the CanvasRedirectIntent Tauri command + the J-31 broken-edge render. cross_lang_diff + svelte-check in-unit.
observable_effect:
  trigger: "open the canvas on a bound plan; REORDER two not-yet-run nodes so the order violates a wiring contract; then issue a redirect on a node an agent is actively emitting"
  expect_ssb: "a CanvasRedirectIntent row (Kind=Reorder, then the colliding Kind) lands in the single-writer journal in sequence order; the preempt lands an H-49 EditHistoryEvent rollback row"
  expect_artifact: "the affected edge flips to broken DETERMINISTICALLY after the re-derive tick (checker/Kahn, not the LLM); the user's redirect supersedes the in-flight LLM action on that node; the whole graph re-derives from the journal under ELAI_SEED. NEGATIVE (R1/R2): an arm that mutates the canvas graph WITHOUT a journaled intent FAILS the contract; an edge verdict set from model output FAILS; a redirect that loses to the in-flight LLM action (queue semantics) FAILS the preempt test."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-61_canvas_redirect_loop.md --run-trigger
rg "enum CanvasRedirectIntent|struct CanvasRedirectIntent" crates/orchestrator/src/canvas_redirect.rs   # intent-as-command present
rg -c "graph\.(insert|remove|mutate)|canvas_state\.set" crates/dashboard-v2/src/lib/canvas/   # MUST be 0 (no direct mutation; all via intent)
rg -c "verdict.*=.*model|edge.*from_llm" crates/orchestrator/src/canvas_redirect.rs           # MUST be 0 (no LLM-set verdict)
test -f workflow/reasoning/J-61-decisions.md
```

## ACCEPTANCE
- [ ] Every redirect is a journaled CanvasRedirectIntent command to the single writer; the canvas never mutates graph state directly (GD-2 holds with live editing).
- [ ] TD-3: edge verdicts come from the J-31 checker + Kahn detection + journal rows; the LLM proposes, never adjudicates.
- [ ] User preempts: a same-node collision resolves to the user intent, with an H-49 rollback row; the queue-semantics arm FAILS the test.
- [ ] Verb set is closed: no ADD-NODE/DELETE-NODE canvas path exists; new nodes only via chat (J-32).
- [ ] Category line: build dispatches the bound unit through the FSM; no canvas-side execution.
- [ ] W-2: ships ON behind ELAI_CANVAS_REDIRECT_DISABLE kill-switch (NOT a default-OFF enable-flag); registered.
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-orchestrator + dashboard-v2 svelte-check.
- [ ] No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: observable_effect — real redirect gesture, off-agent journal rows + deterministic breakage, three negative arms.
- W-2: kill-switch ELAI_CANVAS_REDIRECT_DISABLE (default-ON), registered in wire-dark-allowlist.toml if it ships gated; not a graveyard enable-flag.
- W-6: dashboard redirect surface IS the deliverable.
- W-7: anchors re-grepped at edit; J-12 (PLANNED) and H-49 (PARTIAL) dependencies flagged.
- W-9: SCHEMA baseline re-read at edit for the +CanvasRedirectIntent variant (golden-lock lockstep).
- GD-1: composes J-60/J-31/H-15/H-49/J-12/phase-3B/3G; re-implements none.
- GD-2: byte-identical re-derive under ELAI_SEED with live editing — checked.
- GD-5: honest residue — pause/redirect agent-control + mid-token preempt are Tier-2 on PLANNED J-12 / unwired H-49 turn-grain rollback.

## Tests to add
- crates/contracts/tests/j61_canvas_redirect_contract.rs (~150 LOC): (a) a redirect lands a CanvasRedirectIntent row and the graph re-derives from it (no direct mutation); (b) a REORDER that creates a cycle flips the edge broken via Kahn, not the LLM (TD-3 negative); (c) a same-node collision resolves to the user intent + an H-49 rollback row (preempt); (d) the queue-semantics arm is rejected; (e) an ADD-NODE canvas attempt is refused (closed verb set); (f) the dispatch reaches canvas_redirect.rs through the production namespace, not a test shim (R7).

## Files touched
- CREATE: crates/orchestrator/src/canvas_redirect.rs (~240 LOC)
- MODIFY: crates/orchestrator/src/canvas_dispatcher.rs (~25 LOC: 'redirect' op-namespace)
- MODIFY: crates/orchestrator/src/production_agent_rpc.rs (~30 LOC: 3-arm catalog+convert+dispatch for the redirect ops)
- MODIFY: crates/orchestrator/src/ssb.rs + action_type_lock.rs/.golden (~40 LOC: +CanvasRedirectIntent, golden-lock lockstep)
- MODIFY: crates/dashboard-v2/src/lib/canvas/ (redirect gesture handlers on the J-60 surface, ~180 LOC)
- CREATE: crates/contracts/tests/j61_canvas_redirect_contract.rs (~150 LOC)
- CREATE: workflow/reasoning/J-61-decisions.md (~45 LOC)

## Citations / Grounding
- ELAI ground truth: ssb.rs single-writer WAL + sequence (phase-3B); canvas_dispatcher.rs:101; canvas_ui_action.rs:61 (H-15); system_canvas.rs verdict ladder (J-31); phase-3G Kahn cycle detection; H-49 RV-1/EditHistoryEvent(226). I-6 corrected to a context-pane primitive (feature-map L314-316).
- Category grounding (WebSearch 2026-06-20): manual node-wiring loses to typing (the visual-builder retention plateau) — chat-primary new-node creation + a closed verb set is the deliberate counter; "Not Another Workflow Builder" (langchain.com). ACG framing (nodes = atomic actions, edges = control/data deps): "From Static Templates to Dynamic Runtime Graphs" (arXiv:2603.22386).
- CLAUDE.md hard rules: no .unwrap in prod; blake3 only; kill-switch not enable-flag (W-2); scoped tests; main model never runs tests; vertical-slice wiring.

## Open questions / risks
- Mid-token preempt of a long LLM generation needs H-49 turn-grain rollback wired (PARTIAL) — Tier-2; v1 preempts at the node/edit grain only. Decide the grain at executing.
- AGENT-CONTROL pause/redirect depends on PLANNED J-12; the kill arm uses the shipped stop path. If J-12 slips, v1 ships connect/cut/reorder/rebind + kill only.
- REORDER cross-altitude semantics: a PROCESS reorder that has no SYSTEM consequence should be a no-op verdict, not a false breakage — the re-derive must distinguish "decoupled" from "broken". Decide the verdict mapping at authoring.

## Next
J-61 closes the loop: the layered canvas becomes the bidirectional negotiation surface between the
user and ELAI — watch (J-60), then steer with a closed, journaled, deterministically-checked verb
set where the human always has the wheel. With J-31 (system), J-60 (process + agent tabs), and J-61
(redirect), the canvas is a full second front-door beside chat — backed by the journal, never a
parallel runtime.

End of J-61.
