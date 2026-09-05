# H-13: Model drives the canvas — ops + dispatcher + SSB action_types + ARCHITECTURE mode

STATUS: PLANNED (WAVE 2; BUILD; HD-4 off-agent auditor-driven gate; owns the +canvas_op_applied transport)

Objective: Make the canvas a tool the model CALLS (generative, not a passive viewer). The model emits an incremental
action-op stream (createNode/connect/updateNode/annotate/fork/...) via canvas_* ToolDefinitions added through the E-40
3-arm pattern; a CanvasDispatcher validates each op against the active mode + a sanitizeAction repair pass and mutates
the canvas store; every op lands as ONE Pattern-2 CanvasOpApplied SSB row so the model's drawing streams onto the
surface live (the tldraw streaming pattern). Plus ARCHITECTURE mode (canvas_render_diagram + a mermaid import bridge),
per-mode constrained grammars with a CI compile ceiling + a non-silent grammar_unavailable fallback, and canvas_screenshot
-> Tauri-native capture -> the SHIPPED media_validator Tier2. The done-gate is AUDITOR-driven against a task authored
BEFORE the slice with the gate fileset frozen (HD-4) — the implementing agent cannot satisfy its own gate.

Estimated effort: ~2 sessions (~12-16 h). The 3-arm tool wiring + the dispatcher + the SSB transport + the auditor gate.
Pre-requisites: H-10/H-11/H-12 (the canvas shell + live layer + modes) MUST land first. The H-4 3-arm tool-add pattern
  exercised; the 9-tuple SSB lockstep; the media_validator Tier2 SHIPPED (task_runner.rs:6208); F-3p1 CommittedToolSet LIVE.
Parallel-safe with: nothing touching production_agent_rpc.rs tool dispatch or ssb.rs in the same window.
Source: CANVAS-WORKBENCH-DESIGN.md §7 (how the model drives the canvas) + §11 (architecture) + §13 C4; _grounding/H-GROUNDING-MAP.md C15 + C17 + the H-13 table. Doctrine: H-0_INDEX.md.

## Context

The op representation is an incremental discriminated-union action stream keyed on a _type literal, NOT a whole-document
re-emit (whole-doc re-emit is token-expensive and churns ids, destabilizing shapes the user is looking at — CANVAS §7).
Every op also lands as an SSB row (the SSB action-type model is the right transport, already what the dashboard
subscribes to). Reliability via schema + constrained/grammar-guided decoding (XGrammar): the win is real for SMALL
grammars (JSONSchemaBench), but XGrammar has a published compile-timeout cliff on complex grammars — so each mode
compiles its OWN grammar (DESIGN decomposed into sub-grammars), the compile time is measured in CI against a ceiling,
and a grammar_unavailable timeout returns a STRUCTURED error + a constrained sub-grammar retry, NEVER a silent free-form
fallback (HD-3 — constrained decoding is for op-emission, never the coder/reasoning role; CANVAS §7). Constrained decoding
guarantees STRUCTURE not MEANING, so a mandatory sanitizeAction repair pass runs before any op is applied (fixes
hallucinated ids, normalizes coords, REJECTS a dangling reference — does not silently fix it). The done-gate is the
HD-4 hazard: the danger is the same agent writing the tool, the dispatcher, the SSB schema, and then the SSB row
"proves" its own code ran (R4 in-boundary artifact) — so every gate is auditor-driven against a pre-frozen task.

## Existing state (B/C/D/E/F/G coverage)
- build_tool_catalog / convert_json_input_to_postcard / translate_tool_request (production_agent_rpc.rs:266/970/1163) — SHIPPED. The 3-arm tool-add surface (H-4 exercises it; E-40 precedent symbol_dsl_query).
- the 9-tuple SSB lockstep (ssb.rs) — SHIPPED. The pattern every new action_type satisfies.
- media_validator (MediaValidator, Tier1, Tier2, wire_media_validator_for_spec, SsbMediaSink) — SHIPPED. Prod caller
  task_runner.rs:6208-6243 (lazily built when !spec.media.is_empty()); Tier2 backend mock_vlm by default (live VLM None :6216, may be RUNPOD-PENDING). The Tier2 consumer canvas_screenshot feeds — REUSE.
- F-3p1 CommittedToolSet / committed_slot / check_against_commitment (capability_flow.rs:53) — SHIPPED, cross-entrypoint wired (the control-plane tool-set gate). The CanvasDispatcher's op-namespace gating composes with it.
- E-37 Provenance / allow_tool_with_origin (provenance.rs/capability_gate.rs) — WIRE-DARK end-to-end (no prod caller); the task_runner argv provenance thread does NOT exist — do NOT depend on it (that is H-15/G-1 territory).
- native canvas/webview screenshot capture — ABSENT (design-only CANVAS §11; the Tauri-native capture is BUILD).

## Reads (mandatory before coding)
- _grounding/H-GROUNDING-MAP.md C15 (the 3-arm tool pattern + the 16-tool catalog) + C17 (the trust-spine state + media_validator shipped) + the H-13 table.
- CANVAS-WORKBENCH-DESIGN.md §7 (the op stream + the tools + the SSB flow + the FSM-not-touched decision + the constrained-decoding cliff + the grammar_unavailable fallback + sanitizeAction) + §11 (the architecture + the screenshot-capture caveat) + §13 C4 (the auditor-driven gate).
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\production_agent_rpc.rs:260-770,960-1090,1160-1200 (the 3-arm surface)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs (the 9-tuple lockstep) + the action_type golden lock
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\task_runner.rs:6200-6250 (the media_validator wire — REUSE)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\capability_flow.rs:50-70 (CommittedToolSet — the control-plane gate to compose with)
- CLAUDE.md HARD RULES + HD-3 (constrained decoding only on op-emission) + HD-4 (off-agent auditor-driven gate) + HD-5 (a model drawing is valid-not-verified).

## Tasks (WAVE 2 — build the model-op transport)

1. **canvas_* ToolDefinitions via the 3-arm pattern (~180 LOC).** canvas_place_node / canvas_connect_edge /
   canvas_update_node / canvas_delete_node / canvas_fork_node / canvas_render_diagram / canvas_screenshot, each as 3
   coordinated arms (catalog block in build_tool_catalog + convert arm + translate_tool_request dispatch arm) with a
   TOOL_<X> const + an <X>Request struct in elai_ipc_types (field names matching input_schema verbatim). Mirrors H-4/E-40.

2. **The CanvasDispatcher (~140 LOC).** A trait that validates an op against the active mode flag (a design.* op while
   mode!=design is rejected with a structured error — namespace gating), runs the sanitizeAction repair pass (fix
   hallucinated ids, dedup new ids, normalize coords, REJECT a dangling reference — surface it as a blocked op, never a
   silent fix), and mutates the canvas store. It is itself a dyn dispatch point (so it goes in the H-10 entrypoints map).
   Composes with F-3p1 CommittedToolSet (the control-plane sibling), does not duplicate it (GD-1).

3. **+canvas_op_applied SSB (Pattern-2; 10-arm Kind) (~120 LOC in ssb.rs).** ONE SsbActionType::CanvasOpApplied + a
   CanvasOpKind {NodeCreated/EdgeConnected/NodeDeleted/EdgeDeleted/AnnotationAdded/LabelEdited/GroupCreated/NodeForked/
   ScreenshotCaptured/LayoutChanged} + CanvasOpPayload. 9-tuple lockstep. SCHEMA bump baseline+1 (LIVE 207/484 -> re-read;
   bump e45:294 if first). The CanvasPanel filters on action_type='canvas_op_applied' and applies rows to the store live.

4. **ARCHITECTURE mode + the mermaid import bridge (~120 LOC).** canvas_render_diagram for bulk authoring; mermaid/
   plantuml are IMPORT-ONLY through a blueprint-parser -> native-shape bridge (the DSL supplies layout coords via
   offscreen getBBox; our shape model supplies the editable output) — the model never emits raw mermaid as its authoring
   channel (unreliable; we emit validated ops instead).

5. **Per-mode constrained grammars + the CI ceiling + the grammar_unavailable fallback (~120 LOC).** Each mode compiles
   its OWN grammar (DESIGN into sub-grammars); the compile time is measured in CI against a ceiling (a grammar that
   approaches the XGrammar compile-timeout FAILS the build and must be split); a timeout returns a structured
   grammar_unavailable error + a constrained sub-grammar retry, and if that fails the op is surfaced as a blocked tool
   call in the SSB stream with the reason — NEVER a silent free-form fallback. Constrained decoding ONLY on op-emission (HD-3).

6. **canvas_screenshot -> Tauri-native capture -> media_validator Tier2 (~90 LOC).** capture_canvas_region uses Tauri's
   OWN webview screenshot capability to produce the PNG (NOT external Playwright — the canvas is the interior of a Tauri
   window), then hands the PNG to the SHIPPED media_validator Tier2 (task_runner.rs:6208) for VLM scoring -> a feedback SSB row.

7. **The auditor-driven done-gate (~30 LOC harness) + decision doc H-13-decisions.md (~120 LOC).** The gate is frozen
   BEFORE the slice; an AUDITOR drives a pre-authored task; the off-agent artifact is the SSB row + the rendered node.
   Decisions: the op stream not whole-doc re-emit; FSM untouched (the mode flag is the gate); the constrained-decoding
   cliff + the grammar_unavailable fallback; sanitizeAction rejects-not-fixes; media_validator Tier2 reused; HD-4.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/production_agent_rpc.rs (canvas_* ToolDefinitions + convert arms + dispatch arms)
  - crates/orchestrator/src/canvas_dispatcher.rs::CanvasDispatcher (mode-gating + sanitizeAction)
  - crates/orchestrator/src/ssb.rs::SsbActionType::CanvasOpApplied (+ CanvasOpKind + CanvasOpPayload)
  - crates/orchestrator/src/canvas_grammar.rs (per-mode grammars + the CI ceiling + the grammar_unavailable fallback)
  - crates/dashboard-v2/src-tauri/src/commands_h13.rs::capture_canvas_region (Tauri-native screenshot) + CanvasPanel.svelte (applies canvas_op_applied rows live)
consumes_symbols:
  - crates/orchestrator/src/production_agent_rpc.rs::build_tool_catalog / convert_json_input_to_postcard / translate_tool_request (the 3-arm surface)
  - crates/orchestrator/src/ssb.rs (the 9-tuple lockstep)
  - crates/orchestrator/src/task_runner.rs::wire_media_validator_for_spec (the SHIPPED Tier2 — REUSE)
  - crates/orchestrator/src/capability_flow.rs::CommittedToolSet (the control-plane gate to compose with)
reachable_from:
  - crates/orchestrator/src/production_agent_rpc.rs:835 (build_tool_catalog) -> the model tool-call dispatch (handle_tool_request -> translate_tool_request -> CanvasDispatcher) -> canvas_op_applied SSB -> CanvasPanel  [PRODUCTION model->canvas]
  - crates/dashboard-v2/src-tauri/src/commands_h13.rs::capture_canvas_region -> media_validator Tier2 -> feedback SSB  [PRODUCTION screenshot eval]
stub_ban:
  - NO whole-document re-emit (the op stream mutates the store)
  - NO FSM substate for the mode (the mode flag + the dispatcher gate; fsm.rs untouched)
  - NO silent free-form fallback on a grammar timeout (structured grammar_unavailable + a blocked op with a reason)
  - NO constrained decoding on the coder/reasoning role (HD-3 — op-emission only)
  - NO silent fix of a dangling-id op (sanitizeAction REJECTS it as a blocked op)
  - NO re-implementation of media_validator Tier2 / the 9-tuple lockstep / CommittedToolSet (GD-1 P0)
  - NO self-proving gate (HD-4 — auditor-driven, frozen, off-agent)
  - NO external Playwright capture (Tauri-native screenshot)
  - blake3; no .unwrap()/.expect() in prod; SCHEMA bump from LIVE 207/484
frontend_pair:
  CanvasPanel.svelte (applies canvas_op_applied rows live) + capture_canvas_region (tauri). cross_lang_diff PASS; W-6 dashboard E2E.
observable_effect:
  trigger: "(AUDITOR-driven, gate FROZEN before this slice) the auditor runs `cargo run -p elai-cli -- <a real task,
    authored BEFORE H-13, that makes the model draw an architecture diagram via canvas ops>` with NO env override"
  expect_ssb: "canvas_op_applied SSB rows land (kind=NodeCreated/EdgeConnected per the drawn diagram), observed by the
    AUDITOR in the live ssb.db — NOT the implementing agent's stdout and NOT its unit test"
  expect_artifact: "the model-drawn nodes RENDER on the canvas (observed by the auditor); a forced grammar-timeout
    surfaces a BLOCKED op with a reason in the SSB stream (never a silent free-form fallback); the gate FAILS if the
    dispatcher is wired to nothing in production even when a unit path passes; a canvas_screenshot is captured natively + VLM-scored (a feedback row lands)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-13_model_drives_canvas_ops.md --run-trigger
rg "canvas_place_node|canvas_connect_edge|TOOL_CANVAS" .elai_cc/crates/orchestrator/src/production_agent_rpc.rs  # the 3 arms exist
test -f .elai_cc/crates/orchestrator/src/canvas_dispatcher.rs
rg "CanvasOpApplied|canvas_op_applied" .elai_cc/crates/orchestrator/src/ssb.rs
rg "canvas_op_applied" .elai_cc/crates/orchestrator/tests/action_type_lock.rs
rg "wire_media_validator_for_spec|capture_canvas_region" .elai_cc/crates/dashboard-v2/src-tauri/src/commands_h13.rs  # native capture -> shipped Tier2
rg "grammar_unavailable" .elai_cc/crates/orchestrator/src/canvas_grammar.rs   # the non-silent fallback exists
python .claude/tools/cross_lang_diff.py
test -f workflow/reasoning/H-13-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate (HD-4, AUDITOR-driven, frozen): --run-trigger PASS verified by the AUDITOR (canvas_op_applied rows land + nodes render), NOT the implementer's stdout; the gate FAILS if the dispatcher is wired to nothing in prod.
- [ ] W-2 default-ON: the canvas tools + dispatcher active with no env override.
- [ ] W-3 production entrypoint: the ops flow from the live model tool-call dispatch; not test-only.
- [ ] W-4: CanvasPanel + capture_canvas_region cross-lang mirrored; cross_lang_diff PASS.
- [ ] W-9: canvas_op_applied round-trips via the SSB roundtrip contract + golden; SCHEMA bump baseline+1; e45 drift fixed.
- [ ] HD-3: constrained decoding only on op-emission; a grammar timeout surfaces a blocked op (never silent free-form).
- [ ] HD-4: the gate is off-agent + auditor-driven + frozen; a self-proving path is rejected.
- [ ] GD-1: media_validator Tier2 + the 9-tuple lockstep + CommittedToolSet are REUSED, not re-implemented.
- [ ] cargo check --workspace GREEN + dashboard svelte-check; auditor PASS.

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE, AUDITOR-driven; the canvas tools (produced) flow through the live dispatch (W-3) and emit canvas_op_applied (W-8).
- W-2: the tools + dispatcher default-on.
- W-3: the model tool-call dispatch (production_agent_rpc.rs:835 -> translate_tool_request); not test.
- W-4: CanvasPanel + capture_canvas_region; cross_lang_diff PASS.
- W-5: a runnable model-draws-diagram task -> real canvas_op_applied rows + rendered nodes; in-process.
- W-6: the model-drawn nodes verified THROUGH the dashboard (dashboard-smoker).
- W-7: the 3-arm surface / the 9-tuple lockstep / media_validator Tier2 / CommittedToolSet grep-confirmed 2026-06-03 (C15/C17).
- W-8: the canvas tools (produced) are dispatched by the model; CanvasOpPayload (produced) is emitted + consumed by CanvasPanel.
- W-9: +canvas_op_applied Pattern-2 baseline+1; golden + e45 drift fix.
- W-10: in-process (the op application + the SSB emit); the VLM Tier2 may be RUNPOD-PENDING (W-10 tiered: mock_vlm Tier-1, live VLM a separate checkbox).
- HD-3/HD-4/HD-5: grammar only on op-emission; the gate is off-agent; a model drawing is valid-not-verified until VLM checks its content.

## Tests to add
- crates/contracts/tests/h13_canvas_ops_contract.rs (~200 LOC): (a) each canvas_* tool has 3 coordinated arms (catalog +
  convert + dispatch); (b) the CanvasDispatcher rejects a design.* op while mode!=design (namespace gating); (c)
  sanitizeAction REJECTS a dangling-reference op as a blocked op (does not silently fix); (d) canvas_op_applied round-trips
  for all 10 Kind arms; (e) a grammar-compile timeout returns grammar_unavailable + a constrained retry, never free-form;
  (f) a canvas_screenshot routes to the SHIPPED media_validator Tier2 (no second validator).
- the auditor-driven frozen-gate harness: a pre-authored task + the gate fileset frozen (the gate_freeze_check passes).

## Files touched
- MODIFY: crates/orchestrator/src/production_agent_rpc.rs (~180 LOC: 7 canvas_* tools x 3 arms) + elai_ipc_types (~70 LOC: consts + Request structs)
- CREATE: crates/orchestrator/src/canvas_dispatcher.rs (~140 LOC) + canvas_grammar.rs (~120 LOC)
- MODIFY: crates/orchestrator/src/ssb.rs (~120 LOC: CanvasOpApplied + CanvasOpKind + payload + 9-tuple lockstep) + action_type_lock.rs + e45 contract
- CREATE: crates/dashboard-v2/src-tauri/src/commands_h13.rs (~90 LOC: capture_canvas_region) + src/lib/canvas/CanvasPanel.svelte (~220 LOC)
- CREATE: crates/contracts/tests/h13_canvas_ops_contract.rs (~200 LOC)
- CREATE: workflow/reasoning/H-13-decisions.md (~120 LOC)

## Citations / Grounding
- CANVAS-WORKBENCH-DESIGN.md §7 (the op stream not whole-doc re-emit; the tldraw agent-template pattern; constrained
  decoding XGrammar + the compile-cliff + the grammar_unavailable fallback; sanitizeAction; the FSM-not-touched decision)
  + §11 (the SSB flow + the Tauri-native screenshot caveat) + §13 C4 (the auditor-driven gate) + §16 (the tldraw/Vercel/MCP-Apps convergence).
- ELAI ground truth (C15/C17): the 3-arm surface production_agent_rpc.rs:266/970/1163 (E-40 precedent); the 9-tuple lockstep;
  media_validator Tier2 shipped task_runner.rs:6208; CommittedToolSet capability_flow.rs:53 (live); E-37 Provenance wire-dark (do not depend on the non-existent argv thread).
- CLAUDE.md HARD RULES + HD-3/HD-4/HD-5.

## Open questions / risks
- The constrained-decoding compile cliff is measured in CI per grammar; DESIGN mode (H-14) is the fat one and is
  decomposed into sub-grammars — the CI ceiling is the guard, not a hope.
- The Tier2 VLM "is this architecture diagram correct" capability is UNPROVEN (CANVAS §14) and needs its own eval before
  model-authored diagrams are trusted in a real plan; until then a model drawing is valid-not-verified (HD-5).
- The auditor-driven gate is the load-bearing anti-self-proving control (HD-4); the gate fileset MUST be frozen via
  gate_freeze_check before the slice executes, or the implementing agent can route its own gate to PASS.

## Next
H-15 (spawn_view) builds on H-13's op transport + dispatcher: the reply-channel verb + the taint-gated inbound deixis +
snapshot/delta. H-14 (DESIGN mode) rides the H-13 canvas_op_applied transport for design.* ops. The ARCHITECTURE mode +
the media_validator Tier2 close the "model explains itself visually" loop.

End of H-13.
