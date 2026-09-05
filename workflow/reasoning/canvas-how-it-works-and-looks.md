# ELAI Canvas — how it works and how it looks

Synthesis of the 11 canvas plan files (H-10..H-16 + J-31/J-60/J-61/J-98), read 2026-07-02.
Everything below is grounded in those files; nothing is invented. Status labels are as the plans state them
(most is PLANNED; the H-13/H-15 substrate is code-verified SHIPPED per J-31/J-60 grounding notes).

================================================================================
1. WHAT THE CANVAS IS (the thesis)
================================================================================

The canvas replaces the old 3D "orbit" starfield (a watch-not-work visual that made a running engine feel
alive to someone who couldn't read code) with a 2D generative work-surface that the MODEL DRIVES and the
USER POINTS AT. It is a new WindowHost window type in the Svelte/Tauri dashboard (dashboard-v2).

Two things make it more than a graph viewer:
  - the model can CALL it as a tool (draw nodes/edges/diagrams as its answer), not just render into it.
  - every visual op is JOURNALED as an SSB row, REPRODUCIBLE (G-2 RRA), and TAINT-TAGGED (G-1 provenance).
    That trust-spine underneath is the MOAT: Anthropic Artifacts / ChatGPT Canvas / MCP-Apps / tldraw all
    render-and-forget; ELAI's canvas is replayable, auditable, and provenance-aware (H-15).

The hard category line (repeated in every J file): the canvas is NOT a runtime like n8n/Flowise/Langflow.
Those ARE the runtime — data flows through their nodes at execution time. ELAI's canvas SHOWS and STEERS;
the actual build always runs through the normal FSM. No canvas-side execution, ever.

================================================================================
2. HOW IT LOOKS (visual / UX)
================================================================================

A single flat 2D plane with pan/zoom, one shared chrome bar (one zoom, one search Cmd+F, one inspector,
one breadcrumb, one follow-agent toggle), and a segmented MODE BAR that switches which "clipping frame" of
the plane is active. Modes are NOT separate screens — they are namespaced node/edge types living in their
own clipping frame on the same plane (H-12).

Rendering stack (H-10, B0-measurement-gated): cytoscape WebGL renderer (~100+fps at 1,200 nodes / 16k edges)
for graphs under ~2,500 nodes; sigma@3.0.3 above that. EDGE DENSITY, not node count, is the frame killer, so
a measurement gate (B0) picks the renderer before any code ships. The 60fps lever is SEMANTIC ZOOM + LOD:
  - Far tier (<8px footprint): folder super-nodes + bundled super-edges, no text.
  - Mid tier (8-40px): per-file nodes, labels appear above ~24px.
  - Near tier (>40px): per-symbol card chrome.
  Labels/chrome are hidden below their pixel threshold — that's what keeps the DRAWN element count bounded
  regardless of graph size.

Dead code LOOKS disconnected: a node with zero static edges islands away from the graph. Two distinct
treatments (HD-5, never launder a guess): confirmed-dead (coverage-backed, text-ghosted + dashed) vs
suspected-dead (topology-only, lighter amber-neutral ring). A function only reachable via Tauri-invoke /
dyn Trait / the SSB bus is re-attached by an entrypoints map BEFORE orphan detection, so it does NOT
false-render as dead.

The live layer (H-11) gives back the "alive" feeling the orbit had, on a legible flat surface:
  - the active node gets a HALO, the traversed edge a DASH-FLOW, a follow-agent toggle keeps the
    cursor-of-attention centered.
  - if an event points at an un-indexed file (no node on canvas), a transient GHOST node appears near the
    nearest resolvable ancestor in the unverified treatment, then reconciles into a real node when indexing
    catches up or fades to a dropped-marker after 30s. Never a silent gap.
  - a VS Code-style file explorer sits in a new grid column, bidirectionally synced with the canvas over the
    existing focus bus (click a file -> canvas centers that node; select a node -> tree highlights the file).

The starfield aesthetic survives as a cheap 2D layer (the seed source orbit/lcg.ts is kept); the heavy
three.js/troika layer is deleted last (H-16).

================================================================================
3. HOW IT WORKS (architecture)
================================================================================

3.1 The op stream + dispatcher (H-13, the "model drives it" core)
The model emits an INCREMENTAL discriminated-union action stream keyed on a _type literal
(createNode / connect / updateNode / annotate / fork / renderDiagram / screenshot ...), NOT a whole-document
re-emit (whole-doc re-emit is token-expensive and churns ids, destabilizing shapes the user is looking at).
Each op is a canvas_* ToolDefinition wired through the E-40 "3-arm" pattern (catalog + convert + dispatch).

The CanvasDispatcher (crates/orchestrator/src/canvas_dispatcher.rs, SHIPPED):
  - MODE-NAMESPACE GATES each op (a design.* op while mode != design is rejected with a structured error).
  - runs a sanitizeAction repair pass: fixes hallucinated ids / normalizes coords, but REJECTS a dangling
    reference as a blocked op — it never silently "fixes" one.
  - mutates the canvas store; every applied op lands as ONE Pattern-2 CanvasOpApplied SSB row, so the
    model's drawing streams onto the surface live (the tldraw streaming pattern).

Reliability: per-mode CONSTRAINED/grammar-guided decoding (XGrammar) — but only on OP-EMISSION, never on the
coder/reasoning role (HD-3). Each mode compiles its own grammar with a CI compile-time ceiling; a grammar
timeout returns a structured grammar_unavailable error + constrained retry, surfaced as a blocked op with a
reason — NEVER a silent free-form fallback. Constrained decoding guarantees STRUCTURE not MEANING, hence the
mandatory sanitizeAction pass.

3.2 The mode flag is a plain flag, NOT an FSM substate (H-12, §7)
The active mode is a plain field on canvas.svelte.ts (mode in {code,plan,research,arch,design,+system,
+process,+memory}). Making it an FSM substate would multiply the FSM by two axes and create compound states
(mode active AND FSM AUDITING) specified nowhere — so fsm.rs is UNTOUCHED. The flag only gates which
op-namespace is writable, enforced in the dispatcher.

3.3 Single-consumer SSB handover (H-12, §12 step 4)
The orbit and the canvas live-layer both want the live SSB subscription; running both double-applies events
into a shared store. So the ACTIVE window OWNS the subscription and the inactive one is PAUSED (suspended, not
just hidden); the switch hands the subscription over ATOMICALLY (suspend-then-activate, never both live).

3.4 spawn_view — the general reply channel (H-15, the moat)
Generalizes the canvas from fixed modes into the model's general-purpose VISUAL REPLY CHANNEL: when prose or
ASCII is the wrong medium, the model calls spawn_view{view_type, seed_ops[]} and an interactive view IS its
answer. view_type is a discriminator into a renderer registry (the modes register first — a generalization,
not a rewrite). Emitted via a second SSB transport, CanvasViewEvent. Three slices:
  - H15-1 comparison/branch board (N columns: approach A vs B, two git branches, two run-artifacts) — turns
    the two-approaches-minimum rule into a pointable board.
  - H15-2 the INBOUND leg: CanvasUiAction {Select|Intent|Prompt|ToolCall|Link} so a user click is JOURNALED,
    not an ephemeral postMessage. Select serializes the clicked node back to the model via a tiered read-back
    (selected=full detail, viewport=blurry, off-screen=clustered) — the token-saving tldraw pattern. This
    Select handoff is TAINT-GATED (G-1): a web-sourced node's content is flagged at the handoff and never
    silently re-enters the prompt — the prompt-injection mitigation the consumer products structurally can't
    do. A CanvasUiAction::ToolCall requires explicit user consent, never auto-executes.
  - H15-3 snapshot+delta reconciliation per tab: per-tab sequence numbers + periodic SSB checkpoints so a
    reconnecting dashboard rebuilds a tab from the journal TAIL, and user+model provably see the SAME tab
    state (load-bearing for pointing/deixis).

3.5 The journal IS the store (the anti-drift keystone, every J file, GD-2)
Nodes/edges/verdicts persist ONLY as SSB rows; the canvas RE-DERIVES its entire state byte-identically from
the journal under ELAI_SEED. There is no parallel state file, so canvas-vs-code drift is structurally
impossible. The LLM never sets a status or an edge verdict (TD-3) — checker output + journal rows are the
only sources.

================================================================================
4. THE MODES / ALTITUDES (what you can be looking at)
================================================================================

Shipped/near-term modes (H-plan):
  CODE (H-10)      the codebase symbol/call graph; PageRank importance; dead-code islands.
  PLAN (H-12)      the decomposition tree (concept>phase>step>babystep as nested nodes, depends_on edges,
                   FSM-active node lit) — ported flat from OrbitMode1.
  RESEARCH (H-12)  the research tree (cites edges + RED contradiction edges between disagreeing sources) —
                   ported flat from OrbitMode3.
  ARCHITECTURE (H-13)  canvas_render_diagram for bulk authoring; mermaid/plantuml IMPORT-ONLY through a
                   parser->native-shape bridge (the model emits validated ops, never raw mermaid).
  DESIGN (H-14)    the richest + riskiest mode. A design.page node renders a LIVE web-app preview in an
                   iframe pointed at ELAI's OWN dev-preview proxy (which relaxes X-Frame-Options /
                   frame-ancestors so the Tauri webview can embed it — you cannot iframe a raw dev server).
                   Nav edges connect pages into the real navigation graph; you click an element INSIDE a page
                   to edit it (build-time opaque data-oid tagging mirroring @onlook/parser, server-side
                   id->AST-range map; React 19 removed _debugSource so build-time tagging is mandatory).
                   Writeback is two-tier: deterministic recast patches for style/text (never round-tripped
                   through the model), the LLM only for structural — gated on a recast prove-it spike.
                   Web-first; RN/Expo explicitly deferred. Honest limit: only fronts dev servers ELAI can
                   proxy, not arbitrary hard-CSP third-party sites.

Extension altitudes (J-plan — the canvas becomes a full second front-door beside chat):
  SYSTEM (J-31)    the user's codebase as bound, wiring-verified nodes. NODE = a bound buildable unit
                   (PlanUnit | Contract | McpService); a node past DRAFT MUST bind. EDGE = a wiring contract
                   with a DERIVED verdict ladder: declared -> wired (static leg passes) -> verified (the
                   contract's observable_effect SSB row exists; the row id renders on the edge) -> broken.
                   Answers the user pain "you iterate in chat, build silos, never know if it's really wired"
                   by rendering wiring-enforcement TRUTH where the user composes. External service node = MCP
                   connector ONLY (n8n's 400-integration treadmill declined). Build = dispatch the bound unit
                   through the normal FSM.
  PROCESS (J-60)   ELAI's OWN orchestration projected at a different altitude: FSM states, agent_pool workers,
                   phase transitions as nodes, all read-only and journal-derived. "When ELAI spawns a
                   sub-agent I want to see all it's working on, with its own codebase version, and drill into
                   each agent's work." TAB = DESCEND: opening an agent's tab IS zooming into its worker node,
                   revealing its scoped task + FSM substate + tool-call stream + its per-agent WORKTREE DIFF
                   (from the I-2 COW worktree when it ships; until then from the journaled H-49 pre-image
                   op-stream). Deixis-only descend (never auto-expand) is the anti-node-soup discipline that
                   stops the Langflow/Flowise complexity plateau. Emit-only (watch, don't steer).
  REDIRECT (J-61) the STEER half. A small CLOSED verb set on EXISTING nodes: CONNECT / CUT-EDGE / REORDER /
                   REBIND / AGENT-CONTROL(pause|redirect|kill). New-node creation is REJECTED as a canvas verb
                   (nodes are born in chat — the chat-primary thesis). Every user edit is a journaled
                   CanvasRedirectIntent COMMAND to the single-writer engine, never a direct mutation, so
                   there's no write race by construction. A REORDER that breaks a wiring contract turns the
                   affected edge RED on its own (Kahn cycle detection + the J-31 verdict ladder, not the LLM).
                   The USER PREEMPTS: a pending user intent supersedes a not-yet-committed LLM action on the
                   same node (rides H-49 rollback + TaskStop/J-12). No locks, no CRDT — the SSB total order is
                   the conflict model.
  MEMORY/CASE (J-98)  the third J altitude: proven fixes as first-class self-wiring nodes. The moment a bug is
                   PROVED fixed (F-4 proof receipt), it becomes a CaseAltitudeNode (trigger signature, blast
                   radius cone, fix delta, proof badge, 1-3 sentence insight). Edges are J-96's DERIVED
                   evidence links (cone-overlap / co-occurrence / same-taxonomy / supersession). Promotion
                   lights up as evidence accrues: episodic -> skill (after k held-out verified reuses) -> rule
                   (human-gated). Per-user namespaced (J-59 per-seat isolation). "A memory system, but for an
                   entire workflow." Pure visibility layer, re-derived from J-96/J-97 rows, net-0 new SSB.

So at full build the one plane carries up to nine modes/altitudes, navigated by the mode bar + semantic zoom:
code / plan / research / arch / design (H) + system / process / memory (J), plus the redirect verb overlay.

================================================================================
5. THE TRUST-SPINE MOAT (why it's different)
================================================================================

Under every rendered view sits the same three properties the consumer products lack (H-15 §19):
  - JOURNALED: every op + every user click is an SSB row (replay + audit).
  - REPRODUCIBLE: G-2 RRA — a view "was produced by run X and reproduces from the journal."
  - TAINT-TAGGED: G-1 provenance — a node rendered from web/untrusted content is marked, and pointing at it
    (Select) flags the content before it can re-enter the prompt (the prompt-injection defense).
The gates on model-drawn content are AUDITOR-DRIVEN and off-agent (HD-4): the same agent that writes the tool
+ dispatcher + SSB schema cannot "prove" its own code ran via its own SSB row; the done-gate is a frozen,
pre-authored task an auditor runs. A model drawing is valid-not-verified until a VLM (media_validator Tier2)
scores its content (HD-5).

================================================================================
6. BUILD SEQUENCE + STATUS
================================================================================

H-plan Track 2 (the core, WAVE 2), strictly ordered:
  H-10 canvas shell + code-graph  [thesis slice; B0 measurement gate blocks everything]
  H-11 live SSB layer + file explorer + scoped fs surface
  H-12 PLAN + RESEARCH modes + shared chrome + Canvas-as-default (behind a flag) + single-consumer handover
  H-13 model drives the canvas (ops + dispatcher + CanvasOpApplied + ARCHITECTURE mode)   [SHIPPED substrate]
  H-14 APP/WEBSITE DESIGN mode (web-first; the largest, most-gated slice)
  H-15 spawn_view reply channel + taint-gated deixis + snapshot/delta   [SHIPPED substrate; the moat]
  H-16 retire the orbit  [DELETE; LAST; only after Canvas-as-default ships a full release with no regressions]

J-plan extensions (build-on-shipped): J-31 system -> J-60 process -> J-61 redirect -> J-98 memory/case.
J-31/J-60/J-61/J-98 are PLANNED. Per the J-31/J-60 grounding notes, the underlying canvas substrate
(CanvasWindow, CanvasMode union at canvas.svelte.ts:39, InProcessCanvasDispatcher at canvas_dispatcher.rs:101,
CanvasOpApplied, spawn_view at production_agent_rpc.rs:1299, CanvasUiAction at canvas_ui_action.rs:61, G-1
taint, G-2 RRA) is code-verified SHIPPED as of 2026-06-12. Canvas is NOT yet the default window
(workspace.svelte.ts primary = 'activity', canvasAsDefault flag false pending H-16).

Separate artifact (not part of this product canvas): the HARNESS canvas MCP at .claude/mcp/canvas/ (port 7323,
dependency-cruiser depGraph, click-handoff hook) — a workflow-root tool, WIRED, distinct from everything above.

================================================================================
7. THE BIGGEST RISK (called out in the plans)
================================================================================

Scope death by optimism is the #1 risk across the whole track (H-10). H-10 alone is a complete shippable win
(the codebase graph the user wanted first); H-11..H-16 are additive; the J altitudes are further additive.
The discipline is: don't start H-13/H-14 before H-10..H-12 are default, and don't cross the n8n category line
(the canvas shows and steers, it is never a second runtime).
