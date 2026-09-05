# H-15: spawn_view reply channel + taint-gated deixis + snapshot/delta — the trust-spine moat

STATUS: PLANNED (WAVE 2; BUILD; the canvas MOAT; depends H-13 + G-1 + G-2)

Objective: Generalize the canvas from five fixed views into the model's GENERAL-PURPOSE VISUAL REPLY CHANNEL: when
prose or ASCII is the wrong medium, the model spawns an interactive view AS its answer, and the user points back into
it. This is a field-converged, already-shipped direction (Anthropic shipped "custom visuals in chat" — the model itself
deciding text-vs-visual at reply time), so the canvas alone is table-stakes; the MOAT is the journaled (SSB) +
reproducible (G-2 RRA) + provenance/taint-tagged (G-1) op-stream UNDERNEATH it, which Artifacts / ChatGPT Canvas /
MCP-Apps / tldraw structurally lack. H-15 adds the spawn_view{view_type,seed_ops[]} verb + ONE Pattern-2
CanvasViewEvent SSB, in three slices: H15-1 the comparison/branch board (the cheapest high-value view); H15-2 the
inbound CanvasUiAction::Select with tiered read-back GATED by a G-1 taint check (the part the consumer products cannot
do); H15-3 snapshot+delta reconciliation per tab. The done-gate is HD-4 off-agent auditor-driven.

Estimated effort: ~2 sessions (~10-14 h). The verb + the SSB transport + three slices, riding H-13's op stream.
Pre-requisites: H-13 (the canvas op transport + the CanvasDispatcher) MUST land first. G-1 (taint) + G-2 (RRA) are
  COMPOSE-WITH dependencies — both PARTIALLY ABSENT today (capability_flow::taint_text is the reusable partial primitive;
  G-1 taint_flow.rs + G-2 RRA are absent) — H-15 either waits on them or scopes the taint via taint_text (state which, GD-1).
Parallel-safe with: nothing touching the canvas op transport. Sequence after H-13.
Source: CANVAS-WORKBENCH-DESIGN.md §15-§22 (the reply-channel reframe + the taxonomy + the wiring + the trust-spine differentiator + the risks + the H-plan phasing §21); _grounding/H-GROUNDING-MAP.md C17 + the H-15 table. Doctrine: H-0_INDEX.md.

## Context

The one genuinely-new abstraction is a tool-catalog verb, spawn_view{view_type,seed_ops[]}, whose view_type is a
discriminator selecting a registered renderer — the direct analog of Anthropic's content-type routing, MCP Apps'
_meta.ui.resourceUri, and tldraw's _type (CANVAS §15). Everything else already exists in this design: the op stream
(H-13), the per-op SSB transport (H-13), the modes registry (H-12 -> a renderer registry), the tiered read-back (H-13).
The trust-spine differentiator (CANVAS §19): none of Artifacts / ChatGPT Canvas / MCP Apps / tldraw carries provenance
on the emitted UI — they render-and-forget. Every ELAI view is journaled (SSB replay + audit), reproducible (G-2 RRA:
"produced by run X, reproduces from the journal"), and taint-tagged (G-1: a node rendered from web/untrusted content is
marked). The highest-severity risk is prompt-injection via rendered content (the Claude Artifacts "Claudy Day" chain
exfiltrated through an allowlisted egress): when the user points at a node, untrusted rendered text becomes a re-injection
vector — caught by the G-1 taint tag gating CanvasUiAction::Select (a tainted node's content must not silently re-enter
the prompt without the provenance flag). The one genuinely-new primitive is snapshot+delta reconciliation per tab
(AG-UI's STATE_SNAPSHOT + STATE_DELTA with drift detection): per-tab sequence numbers + periodic SSB checkpoints so a
late-joining/reconnecting dashboard rebuilds a tab from the journal TAIL, and so the user and the model provably see the
SAME tab state — load-bearing for deixis (Monk 2003: pointing breaks the moment the two parties see different versions).

## Existing state (B/C/D/E/F/G coverage)
- the H-13 canvas op transport (canvas_op_applied + the CanvasDispatcher + the tiered read-back) — the substrate spawn_view rides.
- capability_flow::taint_text / taint_from_verdict / UntrustedTaint (capability_flow.rs:234) — PARTIAL (F-3p1). Bridges
  prompt_armor ClassifierVerdict -> UntrustedTaint; non-test caller safety_cmd.rs:503 (the `elai safety` replay seam, NOT the live agent loop). The reusable taint primitive for the Select gate.
- G-1 taint_flow.rs (TaintFlow) — ABSENT. The full data-plane taint propagation is G-1's job; H-15 reuses taint_text or composes-with G-1 when it ships.
- G-2 RRA (rra.rs / --emit-rra / RunArtifact) — ABSENT. The reproducible-run artifact is G-2's job; H-15 composes-with it (a view "reproduces from the journal").
- E-37 Provenance / allow_tool_with_origin — WIRE-DARK (the spine the taint gate ultimately rides; do not re-implement).
- the 9-tuple SSB lockstep (ssb.rs) — SHIPPED. SCHEMA_VERSION 207 (ssb.rs:2909) / TOTAL_VARIANTS 484 (action_type_lock.rs:549).

## Reads (mandatory before coding)
- _grounding/H-GROUNDING-MAP.md C17 (the trust-spine state: taint_text partial, G-1/G-2 absent, E-37 wire-dark) + the H-15 table.
- CANVAS-WORKBENCH-DESIGN.md §15 (the reframe + spawn_view) + §17 (the reply-view taxonomy) + §18 (wiring the reply channel: the op stream + the SSB union + the renderer registry + the snapshot/delta primitive + the inbound Select leg) + §19 (the trust-spine differentiator + the prompt-injection risk) + §20 (the corrections: interactivity-is-not-free, the over-read of arXiv:2603.26942, standard-chasing) + §21 (the H1/H2/H3 phasing).
- the H-13 detail file (the op transport + the CanvasDispatcher) + the G-1 detail file (taint) + the G-2 summary (RRA).
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\capability_flow.rs:215-240 (taint_text / taint_from_verdict — the reusable partial primitive)
- CLAUDE.md HARD RULES + HD-4 (off-agent gate) + GD-1 (compose G-1/G-2, do not re-implement) + GD-5.

## Tasks (WAVE 2 — build the reply channel, three slices)

1. **The spawn_view verb + the CanvasViewEvent SSB (~140 LOC) [H15 foundation].** spawn_view{view_type,seed_ops[]} as a
   3-arm tool (catalog + convert + dispatch, mirroring H-13). ONE Pattern-2 SsbActionType::CanvasViewEvent + CanvasViewKind
   {ViewSpawned/UiAction/StateCheckpoint} + CanvasViewPayload. 9-tuple lockstep. SCHEMA bump baseline+1 (LIVE 207/484 ->
   re-read; bump e45:294 if first). Generalize the H-12 mode picker into a renderer registry keyed by view_type (the five modes register first — a generalization, not a rewrite).

2. **H15-1 the comparison/branch board (~160 LOC) [the cheapest high-value view].** N columns (approach A vs B, two git
   branches, two run-artifacts) on a shared row axis, backed by the fork op + forked_from edges (H-13). Click a cell ->
   "expand this tradeoff" / "pick this branch". This turns the project's two-approaches-minimum rule into a POINTABLE board. Pure existing ops (columns + labels + fork edges) — no new renderer primitives.

3. **H15-2 the inbound CanvasUiAction::Select, taint-gated (~120 LOC) [the differentiator].** A typed CanvasUiAction
   {Select | Intent | Prompt | ToolCall | Link} so a click is JOURNALED, not an ephemeral postMessage. Selection serializes
   via the tiered read-back (selected=Focused full detail, viewport=Blurry, off-screen=clustered) — copying tldraw's
   token-saving pattern to hand a clicked node to the model as a bound pronoun. The Select handoff is GATED by a G-1 taint
   check (reuse capability_flow::taint_text): a tainted (web-sourced) node's content is FLAGGED at the handoff, never
   silently re-enters the prompt without the provenance flag (the prompt-injection mitigation, CANVAS §19). Any
   CanvasUiAction::ToolCall passes the write-time security gate + explicit user consent (MCP Apps' consent gate / F-1/F-2 write-gates), never auto-executes.

4. **H15-3 snapshot+delta reconciliation per tab (~140 LOC) [the one new primitive].** Per-tab sequence numbers +
   periodic SSB checkpoints (StateCheckpoint Kind) so a late-joining/reconnecting dashboard rebuilds a tab from the
   journal TAIL (not by replaying every op), with drift detection (re-request a fresh snapshot on divergence). This is
   load-bearing for deixis (Monk 2003: pointing breaks if the two parties see different display versions).

5. **The autonomy gate + the suppress-canvas hatch (~30 LOC).** The model deciding WHEN to draw (not just what) widens
   autonomy (the Anthropic-beta risk surface) — gate spawn_view behind a routing-confidence bar + a cheap suppress-canvas
   escape hatch; default to text unless the confidence clears (interactivity is not a free win — Mosca; CANVAS §20.1/§20.5).
   Instrument a canvas-spawn-vs-text rate from H15-1 onward (judge on ELAI's own comprehension/ambiguity delta, not faith).

6. **Decision doc workflow/reasoning/H-15-decisions.md (~120 LOC).** Records: spawn_view is the one new abstraction;
   the trust-spine moat (journaled+reproducible+taint-tagged); the prompt-injection-via-rendered-content risk + the G-1
   gate; snapshot/delta for deixis (Monk 2003); the autonomy gate + suppress hatch; the arXiv:2603.26942 over-read
   (motivating-not-proven — keep the quantitative claim out of any done-gate, CANVAS §20.2); G-1/G-2 compose-not-restate.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/production_agent_rpc.rs (the spawn_view 3-arm tool)
  - crates/orchestrator/src/ssb.rs::SsbActionType::CanvasViewEvent (+ CanvasViewKind + CanvasViewPayload)
  - crates/dashboard-v2/src/lib/canvas/renderer_registry.ts (view_type -> renderer; the five modes register first)
  - crates/dashboard-v2/src/lib/canvas/ComparisonBoard.svelte (the H15-1 view)
  - crates/orchestrator/src/canvas_ui_action.rs::CanvasUiAction (Select/Intent/Prompt/ToolCall/Link, taint-gated Select)
  - crates/dashboard-v2/src/lib/canvas/tab_reconciliation.ts (snapshot+delta per-tab + drift detection)
consumes_symbols:
  - crates/orchestrator/src/capability_flow.rs::taint_text (the reusable taint primitive — REUSE; do NOT re-implement)
  - the G-1 taint_flow gate (when G-1 ships) + the G-2 RRA artifact (the reproducible-run wrapper) — COMPOSE-WITH
  - the H-13 CanvasDispatcher + canvas_op_applied transport + the tiered read-back
  - the F-1/F-2 write-gates (the CanvasUiAction::ToolCall consent gate)
  - the 9-tuple SSB lockstep (ssb.rs)
reachable_from:
  - crates/orchestrator/src/production_agent_rpc.rs (spawn_view dispatch) -> CanvasViewEvent SSB -> the renderer registry -> the board  [PRODUCTION model->reply-view]
  - the canvas Select handoff -> taint check -> the model context on the next turn  [PRODUCTION inbound deixis]
stub_ban:
  - NO re-implementation of taint (reuse capability_flow::taint_text; compose-with G-1 — GD-1 P0)
  - NO re-definition of the G-2 RRA (compose-with it — GD-1)
  - NO untrusted node re-entering the prompt without the provenance flag (the prompt-injection gate)
  - NO CanvasUiAction::ToolCall auto-executing (write-time gate + explicit user consent)
  - NO second canvas transport (CanvasViewEvent is the reply-channel transport; op-application is H-13's CanvasOpApplied — CANVAS §18 "pick one")
  - NO autonomous spawn_view without the routing-confidence bar + the suppress-canvas hatch (default to text)
  - NO done-gate quantitative claim from arXiv:2603.26942 (motivating-not-proven)
  - blake3; no .unwrap()/.expect() in prod; SCHEMA bump from LIVE 207/484; HD-4 off-agent gate
frontend_pair:
  ComparisonBoard.svelte + the renderer registry + tab_reconciliation.ts (svelte) + the spawn_view/Select read commands (tauri). cross_lang_diff PASS; W-6 dashboard E2E.
observable_effect:
  trigger: "(HD-4, auditor-driven, gate FROZEN) the auditor runs a real reply path (authored BEFORE H-15) where the
    model answers with a comparison board; then selects a node, including a web-sourced (tainted) one; then the dashboard is killed + reopened mid-run"
  expect_ssb: "a ViewSpawned CanvasViewEvent row lands (observed by the auditor, off-agent); a UiAction row lands on the
    Select; a StateCheckpoint row anchors the tab"
  expect_artifact: "the comparison board renders (proven by the off-agent SSB row + a Layer-4 trace assert, NOT a unit
    test and NOT the implementer stdout); selecting a node hands its EXACT current state to the model AND a tainted
    web-sourced node's content is FLAGGED at the handoff; kill+reopen the dashboard mid-run and the tab reconstructs from
    the SSB tail, and a forced divergence triggers a fresh-snapshot request"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-15_spawn_view_reply_channel.md --run-trigger
rg "spawn_view|TOOL_SPAWN_VIEW" .elai_cc/crates/orchestrator/src/production_agent_rpc.rs
rg "CanvasViewEvent|canvas_view_event" .elai_cc/crates/orchestrator/src/ssb.rs
rg "canvas_view_event" .elai_cc/crates/orchestrator/tests/action_type_lock.rs
rg "taint_text" .elai_cc/crates/orchestrator/src/canvas_ui_action.rs   # the Select gate REUSES the primitive (not a new taint impl)
test -f .elai_cc/crates/dashboard-v2/src/lib/canvas/tab_reconciliation.ts
python .claude/tools/cross_lang_diff.py
test -f workflow/reasoning/H-15-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate (HD-4, AUDITOR-driven, frozen): --run-trigger PASS verified off-agent (a ViewSpawned row + the board renders; a tainted Select is flagged; the tab reconstructs from the tail), NOT the implementer's stdout.
- [ ] W-2 default-ON: spawn_view active (behind the routing-confidence gate, not an env override) with the suppress-canvas hatch.
- [ ] W-3 production entrypoint: spawn_view flows from the live model reply path; the Select handoff from the live canvas; not test-only.
- [ ] W-4: ComparisonBoard + tab_reconciliation cross-lang mirrored; cross_lang_diff PASS.
- [ ] W-9: canvas_view_event round-trips via the SSB roundtrip contract + golden; SCHEMA bump baseline+1; e45 drift fixed.
- [ ] GD-1: taint is REUSED (taint_text) + G-1/G-2 are COMPOSED-WITH, not re-implemented.
- [ ] The prompt-injection gate: a tainted node's content is flagged at the Select handoff (never silent re-injection).
- [ ] CanvasUiAction::ToolCall passes the write-gate + explicit consent (no auto-execute).
- [ ] cargo check --workspace GREEN + dashboard svelte-check; auditor PASS.

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE (HD-4 off-agent); spawn_view (produced) emits CanvasViewEvent (W-8) on the live reply path (W-3); the board renders (off-agent SSB + Layer-4 assert).
- W-2: spawn_view default-on behind the confidence gate + the suppress hatch.
- W-3: the model reply path + the canvas Select; not test.
- W-4: ComparisonBoard + tab_reconciliation; cross_lang_diff PASS.
- W-5: a runnable reply-with-a-board + a Select + a kill/reopen -> real SSB rows + a reconstructed tab.
- W-6: the board + the Select + the reconstruction via dashboard-smoker.
- W-7: taint_text capability_flow.rs:234 (partial) + the 9-tuple lockstep + the SCHEMA 207/484 grep-confirmed 2026-06-03 (C17).
- W-8: spawn_view (produced) emits CanvasViewEvent; the Select (produced) consumes taint_text + hands context to the model.
- W-9: +canvas_view_event Pattern-2 baseline+1; golden + e45 drift fix.
- W-10: in-process (the view spawn + the Select + the reconciliation); the snapshot/delta is journal-backed.
- HD-4/GD-1/GD-5: off-agent gate; compose G-1/G-2; the autonomy gate + the honest motivating-not-proven framing.

## Tests to add
- crates/contracts/tests/h15_spawn_view_contract.rs (~190 LOC): (a) spawn_view emits a ViewSpawned CanvasViewEvent row +
  the board renders from existing ops; (b) the renderer registry resolves a view_type to a renderer (the five modes
  register first); (c) a CanvasUiAction::Select hands the node's EXACT current state (Focused tier) to the model; (d) a
  tainted (web-sourced) node's content is FLAGGED at the Select handoff (the G-1/taint_text gate fires); (e) a
  CanvasUiAction::ToolCall requires explicit consent (no auto-execute); (f) snapshot+delta reconstructs a tab from the
  journal tail and a forced divergence triggers a fresh snapshot; (g) the canvas_view_event Pattern-2 variant round-trips for all 3 Kind arms.

## Files touched
- MODIFY: crates/orchestrator/src/production_agent_rpc.rs (~60 LOC: the spawn_view 3-arm tool) + elai_ipc_types (~20 LOC)
- MODIFY: crates/orchestrator/src/ssb.rs (~120 LOC: CanvasViewEvent + Kind + payload + 9-tuple lockstep) + action_type_lock.rs + e45 contract
- CREATE: crates/orchestrator/src/canvas_ui_action.rs (~120 LOC: CanvasUiAction + the taint-gated Select)
- CREATE: crates/dashboard-v2/src/lib/canvas/renderer_registry.ts + ComparisonBoard.svelte (~160 LOC) + tab_reconciliation.ts (~140 LOC)
- CREATE: crates/contracts/tests/h15_spawn_view_contract.rs (~190 LOC)
- CREATE: workflow/reasoning/H-15-decisions.md (~120 LOC)

## Citations / Grounding
- CANVAS-WORKBENCH-DESIGN.md §15 (Anthropic shipped custom visuals — the model decides text-vs-visual at reply time) +
  §16 (the op-against-a-catalog convergence: Vercel AI SDK generative UI, A2UI, tldraw agent template, MCP Apps, AG-UI) +
  §17 (the taxonomy) + §18 (the wiring + the snapshot/delta primitive + the inbound Select leg) + §19 (the trust-spine
  differentiator + the Claudy-Day prompt-injection chain) + §20 (interactivity-is-not-free Mosca arXiv:2103.01701; the
  arXiv:2603.26942 over-read; standard-chasing) + §21 (the H1/H2/H3 phasing) + §22 (the source ledger).
- ELAI ground truth (C17): taint_text capability_flow.rs:234 (partial, replay-only); G-1 taint_flow + G-2 RRA absent;
  E-37 Provenance wire-dark; the 9-tuple lockstep; SCHEMA 207/484.
- Monk 2003 (common ground — pointing breaks across display-version divergence); CLAUDE.md HARD RULES + HD-4 + GD-1/GD-5.

## Open questions / risks
- G-1 + G-2 are PARTIALLY ABSENT today: H-15 must state whether it WAITS on them or scopes the taint via the partial
  capability_flow::taint_text (replay-only) — the latter is a weaker gate until G-1's live taint propagation ships. The
  Select gate is necessary-but-not-sufficient until G-1 is live; surface that honestly (GD-5).
- The autonomy widening (the model decides WHEN to draw) is the Anthropic-beta risk surface; the routing-confidence bar
  + the suppress-canvas hatch are the controls; decide whether v1 wants that autonomy or wants every spawn_view user-requested first (CANVAS §20.5).
- The arXiv:2603.26942 "intermediate-state-steering restores convergence" claim is motivating-not-proven (n~10, single
  domain, the positive result came from a human injecting code fixes, not a UI) — keep the quantitative claim OUT of any done-gate (CANVAS §20.2).
- Publish/share-a-canvas is deferred (it inherits the mass-exploitation auth class — build it behind G-8 attestation from day one or not at all, CANVAS §21).

## Next
H-16 retires the orbit (the last canvas session). With H-15, the canvas is the model's journaled, reproducible,
taint-tagged reply channel — the moat the consumer products lack — and the trust spine F/G built sits underneath a new surface.

End of H-15.
