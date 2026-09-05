# J-98: canvas MEMORY/CASE altitude — proven fixes as first-class, self-wiring, journal-grounded nodes beside J-31 (system) and J-60 (process)

STATUS: PLANNED (WAVE 3 = VCG Slice 3 visibility; BUILD-on-shipped — a new canvas ALTITUDE; the visibility layer, NOT the engine; depends J-96 for the case store + edges, J-97 for the reuse/promotion surface)

- Track J | lever STEERING+SEE-THE-MEMORY | grounding STRONG (J-31/J-60/J-61 canvas doctrine code-verified + J-96/J-97 the data) | nature BUILD-on-shipped | effort M-L | WAVE 3 | depends J-96 (case nodes + deterministic edges), J-97 (promotion ladder state), shipped H-13/H-15 canvas, J-61 (the redirect verb set), J-59 (per-seat isolation)
- The user's framing: "make workflows/sessions into canvas nodes too; the moment a bug is PROVED fixed, that fixed-bug becomes a node; nodes wire themselves to other nodes and improve everything — like a memory system, but for an entire workflow." No current plan draws the MEMORY layer on the canvas (J-31 draws the system-composition altitude, J-60 draws the process/FSM altitude). J-98 is that third altitude: a MEMORY/CASE altitude projecting J-96's proven-fix nodes as first-class, self-wiring nodes, with the episodic->skill->rule promotion (J-97) lighting up as evidence accrues. This is the ~20%-net-new "canvas projection" the run §5 names — the visibility, never a second engine.
- Same doctrine throughout (J-31): the SSB journal is the store, the canvas RE-DERIVES byte-identically (GD-2); nodes bind to real artifacts (proof receipts); steering goes through the J-61 closed verb set + the FSM (anti-n8n — the canvas shows and steers, it is NOT a second runtime). The case-altitude adds NO parallel state file, so canvas-vs-journal drift is structurally impossible.
- Source: self-improving-workflow-canvas-verified-case-graph-2026-06-25.md §3.7 (canvas placement), §3.8 (per-user adaptation), §3.6 (promotion visible), §6 Slice 3. Doctrine: J-0_INDEX.md.

## THE ALTITUDE (§3.7) — a MEMORY/CASE mode beside J-31 system + J-60 process
1. MODE: add 'memory' (the case altitude) to the CanvasMode union (canvas.svelte.ts:39, alongside 'code'|'plan'|'research'|'arch'|'design'|'system'(J-31)|'process'(J-60)) + a CanvasModeBar entry + a 'memory' op-namespace on the SHIPPED InProcessCanvasDispatcher (canvas_dispatcher.rs:101, mode-namespace gating is its existing discipline). The .claude/commands/canvas-mode.md flag gains 'memory' (a plain doc flag, never an FSM substate).
2. NODE = A PROVEN CASE (re-derived, never authored on the canvas): a CaseAltitudeNode renders a J-96 case node — trigger_signature (symptom + taxonomy class), the blast_radius_cone, the fix_delta summary, the proof_receipt (F-4 hash + RRA root, the immutable evidence badge), and the 1-3 sentence insight. The node's standing (episodic / skill / rule) renders from J-97's promotion state. Nodes persist ONLY as the J-96/J-97 SSB rows; the canvas re-derives byte-identically (GD-2). A node with no proof receipt CANNOT render (there are none — minting requires F-4 proof, J-96).
3. EDGE = A DERIVED EVIDENCE LINK (never LLM-authored): renders J-96's deterministic edges (cone-overlap / co-occurrence / same-taxonomy / supersession) with their checker-derived weight; the row id renders on the edge (J-31's verdict-on-edge doctrine). The LLM sets no edge or weight (TD-3); a supersession edge renders the older case dimmed (hard-filtered at reuse, SF-4 — shown for lineage, never reused).
4. PROMOTION LIGHTS UP (§3.6): a node visibly transitions episodic -> skill (after J-97's k held-out verified reuses) -> rule (human-gated, B-23 L3). The altitude IS the experience-compression spectrum made visible.
5. STEER via the J-61 closed verb set (anti-n8n): the only mutations are the J-61 journaled commands (e.g. mark-superseded, propose-promotion-for-human-review, pin/unpin a case) — never a free-form canvas edit; a "promote to rule" is a PROPOSAL routed to the human (B-23 L3), not an apply. The build/reuse actions dispatch through the NORMAL FSM (no canvas-side runtime).
6. SESSIONS-AS-NODES (the user's literal ask): sessions are J-60's projection; case-nodes are the durable RESIDUE those sessions leave once a fix is PROVEN. The two altitudes compose — a J-60 process node that produced a proven fix wires to its J-96 case node.

## PER-USER ADAPTATION (§3.8) — the "adapt to the user"
Namespace the case altitude per user/project via J-59 PER-SEAT MEMORY + TRUST ISOLATION (keyed by the UsageTurnEvent.identity) over E-5. The altitude then shows THIS user's recurring bug classes / stack / preferences, and a poisoned/contradictory case in one namespace cannot leak into another (J-59 hard isolation; the SF-3 auditor firewall holds per-seat-AND-role).

## Existing ground truth (code-verified per J-31; re-grep at edit, W-7)
- CanvasWindow.svelte SHIPPED (production callers); CanvasMode union at canvas.svelte.ts:39; CanvasModeBar.svelte. InProcessCanvasDispatcher at canvas_dispatcher.rs:101; CanvasOpApplied SSB live (CanvasOpKind variants). spawn_view + CanvasViewEvent live. J-31 system mode + J-60 process mode are the sibling altitudes (PLANNED). J-61 redirect verb set (PLANNED — the steering channel). J-59 per-seat isolation (PLANNED — the namespacing). J-96/J-97 are the data + reuse/promotion sources. live SSB baseline ~268/577.

## Existing state (why this is BUILD-on-shipped, net-0-new-SSB) — §5
- The canvas substrate (H-13/H-15), the dispatcher, CanvasOpApplied/CanvasViewEvent are ALL shipped — J-98 adds a MODE + renderers, no new canvas engine.
- The case data (J-96) + reuse/promotion state (J-97) already land as SSB rows — J-98 RE-DERIVES from them, it journals NO new case state. Pure canvas ops PREFER-REUSE CanvasOpApplied (J-31 doctrine: no second op-journal). So net 0 new SSB action_types.
- J-31/J-60/J-61 establish the doctrine (journal-as-store, derived verdicts, J-61 steering, anti-n8n) — J-98 is the third instance, restating none.

## Reads (mandatory before coding — W-3)
1. self-improving-workflow-canvas-verified-case-graph-2026-06-25.md §3.6/§3.7/§3.8/§6 Slice 3 (canvas placement, per-user, promotion visible).
2. J-31 (canvas-mode template: mode union, dispatcher namespace, node=binding, edge=derived-verdict, journal-as-store, anti-n8n) + J-60 (process altitude) + J-61 (the redirect verb set).
3. J-96 (case node schema + deterministic edges — the data) + J-97 (promotion ladder state — the lighting-up).
4. J-59 (per-seat memory/trust isolation — the namespacing key).
5. crates/dashboard-v2/src/lib/state/canvas.svelte.ts (the CanvasMode union) + canvas_dispatcher.rs (the namespace gating) + the J-31/J-60 renderers (re-grep at edit, W-7).

## Tasks (mode + renderers + steering + per-seat namespacing; ZERO new engine, ZERO new case SSB)
1. MODE (~30 LOC frontend + ~20 dispatcher): 'memory' in CanvasMode + CanvasModeBar entry + the 'memory' dispatcher namespace (catalog + convert + dispatch, the existing 3-arm pattern) + the canvas-mode.md flag.
2. NODE renderer (~80 LOC): CaseAltitudeNode re-deriving a J-96 case node from its SSB rows (proof badge, taxonomy, cone, insight, standing); a node without a proof receipt cannot exist (none do). DRAFT/grey is impossible here — every node is proof-born.
3. EDGE renderer (~60 LOC): render J-96's four deterministic edge kinds with weight + the row id on the edge; supersession dims the older node (lineage shown, reuse hard-filtered).
4. PROMOTION render (~50 LOC): the episodic->skill->rule standing from J-97; the lighting-up transition as evidence accrues; a "promote to rule" is a J-61 PROPOSAL to the human (B-23 L3), never an apply.
5. STEER (~50 LOC): wire the J-61 closed verb set (mark-superseded / propose-promotion / pin / build-the-fix-through-the-FSM); every steer is a journaled J-61 command; no free-form edit.
6. PER-SEAT namespace (~40 LOC): scope the altitude by the J-59 seat-id (UsageTurnEvent.identity); cross-seat reads refused (J-59 isolation); the SF-3 auditor firewall holds per-seat.
7. SSB: NO new action_type — pure canvas ops PREFER-REUSE CanvasOpApplied; the case data is J-96's rows, re-derived (GD-2). Confirm net 0 at edit (W-9).
8. Decision doc: workflow/reasoning/J-98-decisions.md (the mode name, the re-derivation join, the J-61 verb subset, the per-seat scoping, why net-0-SSB).

## WIRING_CONTRACT
produces_symbols:
  - the 'memory' CanvasMode + CanvasModeBar entry + the 'memory' dispatcher namespace
  - CaseAltitudeNode / CaseAltitudeEdge renderers re-deriving J-96 rows (frontend_pair = the MEMORY mode surface itself)
  - the promotion (episodic->skill->rule) render from J-97 state
  - the J-61 steer verb subset (mark-superseded / propose-promotion / pin / build) + the J-59 per-seat scope
consumes_symbols:
  - the shipped canvas substrate (CanvasWindow / CanvasModeBar / InProcessCanvasDispatcher / CanvasOpApplied) — REUSE, no new engine
  - J-96 case nodes + deterministic edges (the data, re-derived) + J-97 promotion state
  - J-61 redirect verb set (the steering channel) + J-59 seat-id (the namespacing) + J-60 process altitude (sessions-as-nodes sibling)
reachable_from:
  - a live `elai` dashboard switched to the 'memory' canvas mode renders the case altitude from the journal [PRODUCTION]
  - a J-61 steer command (mark-superseded / propose-promotion) on a case node journals + re-renders [PRODUCTION]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies the render reachability (R7)
stub_ban:
  - NO parallel state file: the altitude re-derives byte-identically from J-96/J-97 SSB rows (GD-2)
  - NO LLM-authored node, edge, or weight (TD-3 — nodes are proof-born J-96 cases; edges are J-96's deterministic derivations)
  - NO node without a proof receipt (minting requires F-4 proof, J-96 — a zero-evidence node cannot render, the J-31 wire-dark-grey is impossible because there are no unproven cases)
  - NO free-form canvas mutation: steering is the J-61 closed verb set only; promote-to-rule is a human-gated PROPOSAL (B-23 L3), never an apply
  - NO cross-seat case leakage (J-59 isolation); NO VCG into the auditor/critic/adversary even on the canvas (SF-3 holds at the altitude)
  - NO canvas-side runtime: build/reuse dispatches through the NORMAL FSM (anti-n8n)
frontend_pair: the MEMORY mode surface itself (cross_lang_diff + svelte-check in-unit).
observable_effect:
  trigger: "with >=2 minted J-96 cases (one superseding another) + one J-97-promoted-to-skill case: switch the dashboard to the 'memory' canvas mode; issue a J-61 mark-superseded steer"
  expect_ssb: "the altitude renders entirely from existing J-96/J-97 + CanvasOpApplied rows (no new case row); a J-61 CanvasRedirectIntent for the steer; the full node/edge/standing state re-derives byte-identically under ELAI_SEED (GD-2)"
  expect_artifact: "the proven-fix nodes render with their proof badge + cone + insight; the supersession edge dims the older node; the promoted case shows the skill standing; the steer re-renders deterministically. NEGATIVE (R2): a broken arm rendering a node from executor self-report (no proof receipt) FAILS — nodes are proof-born only; a free-form canvas edit is rejected (only J-61 verbs); a cross-seat case is not visible (J-59); a promote-to-rule that auto-applies FAILS (human-gate, B-23 L3); the altitude injected into a fixture auditor context FAILS the SF-3 firewall."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-98_canvas_case_memory_altitude.md --run-trigger
rg -n "'memory'|CaseAltitude" .elai_cc/crates/dashboard-v2/src/lib/state/canvas.svelte.ts  # the mode + node type exist
rg -n "memory" .elai_cc/crates/orchestrator/src/canvas_dispatcher.rs                          # the 'memory' namespace
rg -n "CaseNodeMinted|CanvasOpApplied" .elai_cc/crates/orchestrator/src/                      # re-derives from J-96 rows; net 0 new case SSB
node .elai_cc/crates/dashboard-v2/... svelte-check                                            # frontend pair green (exact cmd at edit)
test -f workflow/reasoning/J-98-decisions.md
```

## ACCEPTANCE
- [ ] the 'memory' canvas mode renders proven-fix case nodes RE-DERIVED from J-96/J-97 SSB rows (GD-2); no parallel state file.
- [ ] nodes are proof-born only (F-4 receipt); edges are J-96's deterministic derivations; the LLM authors no node/edge/weight (TD-3).
- [ ] promotion (episodic->skill->rule) renders from J-97; promote-to-rule is a human-gated J-61 PROPOSAL (B-23 L3), never an apply.
- [ ] steering is the J-61 closed verb set only; build/reuse goes through the NORMAL FSM (anti-n8n); no canvas-side runtime.
- [ ] per-seat namespacing via J-59 (cross-seat leakage refused); the SF-3 auditor firewall holds at the altitude.
- [ ] net 0 new SSB action_types (prefer-reuse CanvasOpApplied + re-derive J-96/J-97 rows); confirmed at edit (W-9). cross_lang_diff + svelte-check + cargo check GREEN; auditor PASS. No new dependency.

## Tests to add
- crates/contracts/tests/j98_case_altitude_contract.rs (~120 LOC) + a dashboard svelte-check: (a) the altitude re-derives byte-identically from a fixture journal under ELAI_SEED (GD-2); (b) a node rendered from executor self-report (no proof receipt) FAILS (proof-born only, R2); (c) a supersession edge dims the older node and the older node is hard-filtered at reuse (SF-4); (d) a free-form canvas edit is rejected; only a J-61 verb mutates; (e) a promote-to-rule auto-apply FAILS (human-gate); (f) a cross-seat case is not visible (J-59); (g) the SF-3 firewall arm. traced-test / dashboard-smoker per dashboard-handoff.

## Files touched
- MODIFY: crates/dashboard-v2/src/lib/state/canvas.svelte.ts (+'memory') + CanvasModeBar.svelte + new CaseAltitudeNode/Edge renderers under src/lib/canvas/
- MODIFY: crates/orchestrator/src/canvas_dispatcher.rs (the 'memory' namespace, 3-arm tool pattern) + the J-61 verb wiring + the J-59 seat scope
- MODIFY: .claude/commands/canvas-mode.md (+'memory' flag, doc-only)
- CREATE: workflow/reasoning/J-98-decisions.md

## Citations / Grounding
- self-improving-workflow-canvas-verified-case-graph-2026-06-25.md §3.6 (promotion visible), §3.7 (canvas placement, the memory altitude beside J-31/J-60), §3.8 (per-user adaptation via J-59), §6 Slice 3.
- ELAI ground truth: the canvas substrate (H-13/H-15) + J-31/J-60/J-61 doctrine + J-59 isolation + J-96/J-97 data per the feature map 1.3/1.5/dashboard (verify at edit, W-7). live ~268/577 (W-9).
- Hard rules invoked: GD-2 (journal-recomputable, no parallel state), TD-3 (no LLM node/edge), J-31 anti-n8n + verdict-on-edge doctrine, SF-3 (firewall holds on the canvas), SF-4 (supersession), B-23 L3 (human-gated rule promotion), J-59 isolation, W-7/W-9, GD-1 (third canvas-altitude instance, restates none).
- K-18/K-19 own the K-native PROVE/PACKAGE/DISCIPLINE halves; J-98 is the visibility leg of the VCG technical track they recorded OUT-OF-K. The run §8 open question "Canvas vs CLI first" is answered by sequencing: J-96 CLI/journal value FIRST, J-98 canvas LAST (the visibility layer, not the engine).

## Open questions / risks
- mode name: 'memory' vs 'case' vs 'vcg' — pick one consistent with the J-31/J-60 naming at edit; 'memory' reads best as the third altitude (system/process/memory).
- altitude scale: a mature graph has many case nodes; render must cluster (by taxonomy class / cone) and not dump 500 nodes — reuse the existing canvas clustering; never visually count past ~10 (the visual-ui-loop rule), counts come from the journal.
- the run §8 open question carried to the user: the canvas is the VISIBILITY layer, not the engine — the recurrence-hit-rate + faster-fixes value is realizable from J-96/J-97 on the CLI/`elai` surface BEFORE this altitude exists. Confirm the user wants the canvas projection vs more CLI value first; J-98 is sequenced LAST for exactly this reason.

## Next
Completes the VCG: J-96 mints proven fixes + derives edges (observe-only), J-97 reuses them behind the execution-grounded filter (the risky leg), J-98 projects the whole self-wiring memory as a canvas altitude beside system (J-31) and process (J-60). Together they realize the user's "make sessions into nodes, proven fixes become reusable self-wiring memory that improves with use" — as a SYNTHESIS of already-planned pieces (~80% wiring per §5), with the central self-deception risk gated by section 4 throughout.

End of J-98.
