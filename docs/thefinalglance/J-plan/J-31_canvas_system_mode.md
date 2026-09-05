# J-31: canvas system-composition mode — nodes are bound buildable units, edges are verified wiring claims

STATUS: PLANNED (WAVE CFW-2)

- Track J | lever STEERING+CANT-WRECK | grounding STRONG (canvas substrate code-verified SHIPPED 2026-06-12; novelty grep-verified unplanned) | nature BUILD-on-shipped | effort L | WAVE CFW-2 | depends: shipped H-13/H-15 canvas (live), J-16 (DRIFTED leg, Tier-2), J-32 (dictate-on-node routing, soft — chip fallback exists)

## Premise

n8n / Flowise / Langflow / Dify are RUNTIME flow builders: the canvas IS the runtime, data flows
through their nodes at execution time, nodes wrap API/LangChain components. Nobody ships a
design-time system-composition surface for a coding harness where nodes represent components of the
USER'S codebase, the runtime is the generated system itself, and edges are claims about how real
code is wired (web-verified 2026-06-12; novelty grep across H/I/J: zero hits for system-mode /
node-bound-plan-unit / canvas-wiring concepts — GD-1 clear). The user pain this serves: "you
iterate in chat, build silos, and never know if it's really wired." ELAI's wiring-enforcement stack
(W-1..W-10, fail-closed, off-agent) answers exactly that question and currently renders it nowhere
a user composes. J-31 is the rendering + composition surface: the user lays out the system
(backtester / data layer / live connector / ...) as nodes, dictates intent onto them, draws the
edges, and ELAI builds the bound units through the normal FSM while the canvas shows wiring TRUTH.

## Steal/build (7 legs)

1. MODE: add 'system' to the CanvasMode union ('code'|'plan'|'research'|'arch'|'design' at
   crates/dashboard-v2/src/lib/state/canvas.svelte.ts:39) + CanvasModeBar entry + a 'system'
   op-namespace on the SHIPPED InProcessCanvasDispatcher (canvas_dispatcher.rs:101; mode-namespace
   gating is the dispatcher's existing discipline).
2. NODE = BINDING, NOT DRAWING (the anti-drift keystone): SystemNode { node_id, binding, display }
   where binding = PlanUnit(concept/phase/step id) | Contract(contract id) | McpService(server id).
   A node past DRAFT MUST bind; unbound renders DRAFT grey. Bindings and verdicts persist ONLY as
   SSB rows — the journal is the store, the canvas re-derives its state byte-identically (GD-2).
   No parallel state file exists, so canvas-vs-code drift is structurally impossible.
3. EDGE = CONTRACT: SystemEdge { producer_node, consumer_node, contract } where contract carries
   the exact field set wiring_contract_check.py parses (produces / consumes / reachable_from /
   stub_ban / observable_effect; checker lines ~672-688, trigger always-on per R3 — the
   --run-trigger flag is a deprecated accepted no-op). Edge verdict ladder, all DERIVED:
   declared (authored) -> wired (static leg passes: producer symbol exists + consumer references
   it, DKG/elai.map reachability) -> verified (the contract's observable_effect SSB row exists in
   the journal; the row id renders on the edge) -> broken (a previously-passing leg fails).
   The LLM NEVER sets a verdict (TD-3); checker output + journal rows are the only sources.
4. NODE LIFECYCLE: DRAFT -> FRAMED (bound; dictated intent passes the SHIPPED B-40
   IdeaFramer + AmbiguityScorer gate — vague intent gets the clarify loop, not a silent node) ->
   BUILDING (FSM active on the bound unit; live halo via the shipped SSB canvas overlay) ->
   WIRED -> VERIFIED -> DRIFTED (Tier-2: a J-16 blast-radius cone touching the node's bound
   symbols flips status until re-audit; fallback until J-16 ships = DKG file-hash staleness).
5. DICTATE-ON-NODE: select a node + speak/type -> the message routes through the J-32 front door
   with the node's binding attached as scope (the SHIPPED CanvasUiAction::Select,
   canvas_ui_action.rs:61, with its G-1 taint-gated read-back). Until J-32 ships: the existing
   chip flow with the node id prefilled.
6. EXTERNAL SERVICE NODE = MCP CONNECTOR ONLY: server id + blake3 schema pin (E-24) with the G-4
   rug-pull re-pin verdict rendered ON the node; ingress content carries shipped G-1 Provenance
   taint — edges from an external node render the taint badge. n8n's 400-integration treadmill is
   explicitly DECLINED; MCP is the single connector type.
7. BUILD ACTION: a node's "build" dispatches its bound plan unit through the NORMAL
   planning-workflow FSM (no parallel executor, no canvas-side runtime); progress renders from the
   rows the run already emits.

## Existing ground truth (code-verified 2026-06-12; re-grep at edit, W-7)

- CanvasWindow.svelte SHIPPED (production callers); CanvasModeBar.svelte:33; CanvasMode union at
  canvas.svelte.ts:39. Canvas is NOT the default window (workspace.svelte.ts:99-100 primary =
  'activity'; canvasAsDefault flag false by design pending H-16) — J-31 does not change that.
- InProcessCanvasDispatcher at crates/orchestrator/src/canvas_dispatcher.rs:101; production
  dispatch at production_agent_rpc.rs:3040-3062; CanvasOpApplied SSB live (7-variant CanvasOpKind).
- spawn_view tool at production_agent_rpc.rs:1299-1332; CanvasViewEvent emit at :3099;
  CanvasUiAction enum at canvas_ui_action.rs:61. G-1 taint SHIPPED (taint_flow_ssb_adapter.rs:72,
  emit_taint_flow at production_agent_rpc.rs:212, capability_gate allow_tool_with_origin).
  G-2 RRA SHIPPED (rra.rs verify_rra:476).
- GraphWindow + get_graph_view_data_impl (commands.rs:2660) — the code-graph stays its own mode;
  SYSTEM mode renders intent-side components, CODE mode renders what exists. Same window, two
  directions of truth.
- Phase 3G PhaseEntry/ContractRef/extract_contracts are SPEC-level types (phase-3G.txt:39-82),
  NOT confirmed as runtime Rust types — re-grep at edit; if absent, J-31 ships its own
  SystemBinding/EdgeContract types shaped on the 3G spec rather than blocking on 3G.
- Novelty: zero plan-file hits for this concept (GD-1 clear); J-16 BlastRadiusEvent is PLANNED
  (the DRIFTED consumer), J-2 control plane is PLANNED (orthogonal).

## REVISED scope (W-10 tiered)

Tier-1 (ships first): mode + nodes + mandatory bindings + edges + declared/wired verdicts +
MCP node with pin-status render + build action. Verdict derivation runs at audit/idle tick,
auditor-owned — NEVER on the render hot path; the canvas only renders rows.
Tier-2 (gated): verified-via-observable_effect automation on live runs, DRIFTED via J-16,
dictate-on-node via J-32, edge-level taint badges beyond the node badge.

## Integration

- frontend: canvas.svelte.ts (+'system'), CanvasModeBar.svelte, new SystemNode/SystemEdge
  renderers under src/lib/canvas/ (frontend_pair = the SYSTEM mode surface itself; cross_lang_diff
  + svelte-check in-unit).
- dispatcher: 'system' namespace ops (create_system_node / bind_node / declare_edge / set_display)
  via the existing 3-arm tool pattern (catalog + convert + dispatch, production_agent_rpc.rs).
- orchestrator: NEW crates/orchestrator/src/system_canvas.rs (~250 LOC): binding registry over SSB
  rows + verdict derivation (static leg = DKG/elai.map reachability; verified leg = journal join
  on the contract's observable_effect action_type).
- SSB: +SystemContractEvent (Pattern-2 single-variant; Kind: NodeBound / EdgeDeclared /
  EdgeVerdict / NodeDrifted). baseline+1 from LIVE 241/550 (ssb.rs:3152 /
  orchestrator/tests/action_type_lock.rs:705, grepped 2026-06-12 — re-grep at edit, W-9; full
  6-part variant discipline, golden-lock 9-tuple lockstep). Pure canvas ops PREFER-REUSE
  CanvasOpApplied (no second op-journal).

## observable_effect (done-gate, W-1)

Bind a fixture node to a real plan unit carrying a real contract; run its build; the edge flips
declared->wired only after the static leg passes, and ->verified ONLY after the contract's
observable_effect SSB row lands (the row id renders on the edge); a zero-evidence node renders
wire-dark grey, never green (R1). The full node/edge state re-derives byte-identically from the
journal under ELAI_SEED (GD-2). NEGATIVE (R2): a broken arm deriving verdicts from executor
self-report FAILS the contract test (verdict source must be checker/journal rows); deleting the
bound consumer symbol flips the edge to broken on the next derivation tick; an MCP node whose
schema hash changes renders the G-4 violation state and its edges drop to untrusted; a node bound
to a nonexistent plan-unit id fails LOUD at bind time, never renders.

## Riders (recorded, not in-scope)

- Runtime data-flow execution inside the canvas = DECLINED (the runtime is the user's codebase;
  execution is the FSM's job — this is the category line vs n8n, do not cross it).
- Per-node cost/usage badge = rides J-29's canvas badge when it ships.
- Canvas-as-default-window promotion stays H-12/H-16's call.
- A system-node template library (common shapes: trading system, SaaS, ETL) = a later catalog
  unit shaped like J-9's data-driven catalog; recorded only.
