# DEEP-H-007 exhaustive feature discovery

## Scope, method, and evidence classes

- Processed exactly the DEEP-H-007 manifest chunk: 10/10 files, 1,494/1,494 logical lines, 158,951/158,951 bytes. No numbered/index original outside this chunk was read.
- Orientation-only input: 08-H-SUMMARY_INDEX.discovery.md. It is used only for the summary-delta ledger and is not original evidence.
- Every source was read end-to-end with physical line numbering. Repeated requirements in objectives, tasks, wiring, acceptance, tests, risks, and next/dependency sections are collapsed into one atom with every material anchor retained.
- E-* = explicit source statement. A-* = explicit ambiguity/open choice. C-* = internal or cross-source conflict. I-* = bounded inference from explicit atoms; inference never silently becomes a requirement.
- Domain tags: CAP capability/action/surface; STATE present/absent/planned state; FLOW workflow/state transition; DATA schema/data/lifecycle; CFG flag/default/config; INT integration/dependency/event; SEC permissions/security/privacy/compliance; FAIL failure/fallback/recovery; DET determinism/idempotency/concurrency; OBS observability/evidence; TEST acceptance/done-gate; LIMIT constraint/non-goal; DEFER status/deferred work.
- Atomic IDs are source-scoped and unique. Summary deltas and interactions cite atoms and do not duplicate them.

## Exact processed manifest and verification

Manifest anchor: DEEP-SOURCE-MANIFEST.md:759-774. Family H; target status under 1,800; manifest totals 10 files / 1,494 lines / 158,951 bytes.

| File | Manifest lines | Actual lines | Manifest bytes | Actual bytes | SHA-256 (manifest = actual) |
|---|---:|---:|---:|---:|---|
| docs/thefinalglance/H-plan/H-13_model_drives_canvas_ops.md | 201 | 201 | 18,517 | 18,517 | 834cc3b5cc584b29a9e747d489da3336494a8f7c13ce0c326816041af5d2e5dc |
| docs/thefinalglance/H-plan/H-15_spawn_view_reply_channel.md | 203 | 203 | 18,956 | 18,956 | 6c6e8e22970541cac52140347d6c6035b2374ca0f16487841f18a90a7f83047e |
| docs/thefinalglance/H-plan/H-16_retire_orbit.md | 162 | 162 | 14,017 | 14,017 | f7f2d976fa2f23b060b932e0100f85257ccf50b2a9b5584b42c18d6bfd4b6078 |
| docs/thefinalglance/H-plan/H-18_deflated_sharpe_pbo_gate.md | 180 | 180 | 14,750 | 14,750 | 3bd3c29c9ba41a696e95f2e80e40fdf516a17f19972b48971b290241d5f4b436 |
| docs/thefinalglance/H-plan/H-26_shadow_exec_financial_invariant_mr.md | 171 | 171 | 14,403 | 14,403 | 107e3787bf1c01150473ead4f93249ea4d5e0c198acee2d66d8c3297a07f0591 |
| docs/thefinalglance/H-plan/H-34_mcp_defer_loading_tool_search.md | 50 | 50 | 15,317 | 15,317 | cfd96e0d3321debd17d8d1d07646021194c85b74f52012105567399b81bc367c |
| docs/thefinalglance/H-plan/H-37_plan_approval_gate.md | 67 | 67 | 16,629 | 16,629 | f5e2d5afe69f7d5536e7593805ebc347e40996f070a9724f8ec767509dc601a3 |
| docs/thefinalglance/H-plan/H-39_abstention_aware_promotion_eval.md | 191 | 191 | 16,195 | 16,195 | 40141e881577893e18fe79f218fd68eb889d6bc6d434dfc0a33bf63724644871 |
| docs/thefinalglance/H-plan/H-42_serial_depth_reasoning_budget.md | 184 | 184 | 16,112 | 16,112 | 5d9202e4bee8d6526e8bf4129cff1f3b6d57009f728555ff993fad491a5fc5dd |
| docs/thefinalglance/H-plan/H-50_regime_aware_context_management.md | 85 | 85 | 14,055 | 14,055 | 7a4aca7123e61b4aef96ee0f6455b21a653f6920bcccc3914c0bd9b6bef23d96 |
| **Total** | **1,494** | **1,494** | **158,951** | **158,951** | **10/10 exact** |

## Per-file exhaustive ledgers

### H-13 — Model drives the canvas: operations, dispatcher, SSB transport, and ARCHITECTURE mode

Source: H-13_model_drives_canvas_ops.md (201 lines). Status: planned Wave 2 build; HD-4 auditor gate; owns canvas_op_applied transport.

| Atomic ID | Class/domain | Exact source anchor(s) | Explicit feature or behavior |
|---|---|---|---|
| H007-13-001 | E-STATUS/STATE | L3 | H-13 is PLANNED, Wave 2 BUILD, with an HD-4 off-agent auditor-driven gate, and owns the canvas_op_applied transport. |
| H007-13-002 | E-CAP/FLOW | L5-L12 | The model calls canvas tools and emits an incremental create/connect/update/annotate/fork operation stream; CanvasDispatcher validates, sanitizes, mutates the store, journals one row per operation, and streams drawing live. |
| H007-13-003 | E-DEP/DET | L14-L18 | H-10/H-11/H-12 must land first; the three-arm tool pattern, nine-tuple SSB lockstep, shipped Tier-2 media validator, and live CommittedToolSet are prerequisites; edits to production_agent_rpc.rs or ssb.rs are not parallel-safe. |
| H007-13-004 | E-DATA/LIMIT | L22-L24, L107 | Operations form an incremental discriminated union keyed by a type literal; whole-document re-emission is forbidden because it costs tokens, churns IDs, and destabilizes visible shapes. |
| H007-13-005 | E-CAP/CFG | L24-L29, L76-L80 | Constrained/schema-guided decoding is used for operation emission, with a separate grammar per mode and DESIGN decomposed into sub-grammars. |
| H007-13-006 | E-TEST/FAIL | L25-L29, L76-L80, L146 | CI measures grammar compile time against a ceiling; a grammar near the XGrammar timeout fails the build and must be split. |
| H007-13-007 | E-FAIL/FLOW | L28-L29, L76-L80, L109 | Grammar timeout returns structured grammar_unavailable, retries a constrained sub-grammar, and if that fails journals a blocked tool call with a reason; free-form fallback is never silent or allowed. |
| H007-13-008 | E-SEC/LIMIT | L28-L30, L80, L110 | Constrained decoding applies only to operation emission, never the coder/reasoning role; it guarantees structure, not semantic correctness. |
| H007-13-009 | E-FAIL/DATA | L30-L31, L60-L63, L111 | sanitizeAction repairs hallucinated IDs, deduplicates new IDs, and normalizes coordinates, but rejects dangling references as blocked operations rather than silently inventing a repair. |
| H007-13-010 | E-SEC/TEST | L31-L33, L86-L89, L113 | The gate task and fileset are frozen before the slice and an independent auditor drives it; implementation-authored stdout, tests, or SSB evidence cannot self-prove completion. |
| H007-13-011 | E-STATE/INT | L35-L37 | build_tool_catalog, convert_json_input_to_postcard, translate_tool_request, and the nine-tuple SSB lockstep are shipped and must be extended through their existing pattern. |
| H007-13-012 | E-STATE/INT | L38-L39 | MediaValidator Tier 1/Tier 2, wire_media_validator_for_spec, and SsbMediaSink are shipped; production lazily builds the validator when media is non-empty. |
| H007-13-013 | E-CFG/DEFER | L38-L39, L161 | Tier 2 defaults to mock_vlm; the live VLM is None and may remain RUNPOD-PENDING, with mock Tier 1 and live VLM separate completion checkboxes. |
| H007-13-014 | E-STATE/INT | L40, L64 | F-3p1 CommittedToolSet is live across entrypoints; CanvasDispatcher composes an operation-namespace gate with it and must not duplicate the control-plane gate. |
| H007-13-015 | E-STATE/LIMIT | L41, L185 | E-37 provenance is end-to-end wire-dark and the task_runner argv provenance thread does not exist; H-13 must not depend on it. |
| H007-13-016 | E-STATE/CAP | L42 | Native canvas/webview screenshot capture is absent and must be built. |
| H007-13-017 | E-CAP/DATA | L55-L58 | Add canvas_place_node, canvas_connect_edge, canvas_update_node, canvas_delete_node, canvas_fork_node, canvas_render_diagram, and canvas_screenshot as coordinated catalog/conversion/dispatch arms, each with a TOOL constant and verbatim-schema request struct in elai_ipc_types. |
| H007-13-018 | E-CAP/FAIL | L60-L64 | CanvasDispatcher rejects an operation whose namespace does not match the active mode with a structured error, then sanitizes and mutates the canvas store. |
| H007-13-019 | E-INT/DATA | L63-L64, L94 | CanvasDispatcher is a dynamic dispatch seam and must be registered in the H-10 entrypoints map. |
| H007-13-020 | E-DATA/INT | L66-L69 | Add one Pattern-2 SsbActionType::CanvasOpApplied, CanvasOpPayload, and CanvasOpKind values NodeCreated, EdgeConnected, NodeDeleted, EdgeDeleted, AnnotationAdded, LabelEdited, GroupCreated, NodeForked, ScreenshotCaptured, and LayoutChanged through the full lockstep. |
| H007-13-021 | E-DATA/STATE | L68-L69, L145, L160 | Re-read live schema/variant integers at edit time, bump baseline+1, update action-type golden/roundtrip and e45 drift, and have CanvasPanel filter action_type=canvas_op_applied and apply rows live. |
| H007-13-022 | E-CAP/FLOW | L71-L74 | ARCHITECTURE bulk authoring uses canvas_render_diagram; Mermaid and PlantUML are import-only through blueprint parser to native-shape conversion, with DSL layout coordinates obtained via offscreen getBBox. |
| H007-13-023 | E-LIMIT/CAP | L71-L74 | The editable native shape model is authoritative; the model must not use raw Mermaid as its authoring channel and instead emits validated operations. |
| H007-13-024 | E-CAP/INT | L82-L84, L105 | capture_canvas_region uses Tauri's own webview screenshot capability, produces PNG, sends it to the shipped media-validator Tier 2 for VLM scoring, and emits a feedback SSB row. |
| H007-13-025 | E-DATA/OBS | L86-L89, L178 | H-13-decisions.md records operation-stream choice, unchanged FSM, grammar cliff/fallback, reject-not-fix sanitation, media-validator reuse, and the HD-4 gate. |
| H007-13-026 | E-INT | L91-L102 | Produced integration surfaces are the canvas tool arms, CanvasDispatcher, CanvasOpApplied types, canvas_grammar.rs, Tauri capture command, and live CanvasPanel; consumers are the existing tool surface, SSB lockstep, media-validator wire, and CommittedToolSet. |
| H007-13-027 | E-FLOW/INT | L103-L105 | Production reachability is live model tool dispatch through translate_tool_request and CanvasDispatcher to CanvasOpApplied and CanvasPanel; screenshot reachability is Tauri capture to Tier 2 to feedback SSB. |
| H007-13-028 | E-LIMIT/INT | L107-L114 | Forbidden substitutes include an FSM mode substate, a second media validator/SSB lockstep/CommittedToolSet, external Playwright capture, or a second canvas transport. |
| H007-13-029 | E-SEC/LIMIT | L115 | Production code uses blake3 and no unwrap/expect; schema starts from a re-read live baseline rather than trusting the stated 207/484 snapshot. |
| H007-13-030 | E-TEST/OBS | L118-L125 | A pre-authored real task, driven by the auditor with no environment override, must create NodeCreated/EdgeConnected rows in live ssb.db and visibly render corresponding nodes. |
| H007-13-031 | E-TEST/FAIL | L123-L125 | Forced grammar timeout must visibly create a blocked operation with a reason, and a production-disconnected dispatcher fails even if its unit path passes. |
| H007-13-032 | E-TEST/OBS | L123-L125 | canvas_screenshot must be captured natively, VLM-scored, and followed by a feedback row. |
| H007-13-033 | E-DET/TEST | L127-L138 | Deterministic checks run the wiring trigger, verify tool arms, dispatcher/grammar files, CanvasOpApplied/golden, capture-to-validator wiring, cross-language parity, and the decision record. |
| H007-13-034 | E-CFG/TEST | L140-L149, L152-L160 | Canvas tools and dispatcher are default-on with no environment override and execute from the live production model-call path, not a test-only path. |
| H007-13-035 | E-TEST/INT | L144-L149, L155-L160 | CanvasPanel/capture cross-language parity, SSB roundtrip/golden/schema bump, workspace build, Svelte check, dashboard verification, and auditor pass are completion gates. |
| H007-13-036 | E-TEST/SEC | L141-L149, L162 | Acceptance explicitly enforces HD-3, HD-4, HD-5, and GD-1: operation-only grammar, off-agent frozen proof, valid-not-verified drawing until VLM review, and reuse of shipped substrates. |
| H007-13-037 | E-TEST | L164-L170 | Contract tests cover all seven three-arm tools, wrong-mode rejection, dangling-reference blocking, all ten operation kinds, constrained retry without free-form, and routing screenshot to the existing validator; a gate-freeze harness covers the pre-authored task. |
| H007-13-038 | E-DATA/INT | L172-L178 | Implementation modifies production RPC, IPC types, SSB locks/contracts, and dashboard canvas, and creates dispatcher, grammar, native capture, contract test, and decision-record files. |
| H007-13-039 | E-FAIL/TEST | L188-L190 | DESIGN is expected to be the largest grammar; measured CI ceiling, not confidence, is the guard against compile-time failure. |
| H007-13-040 | E-STATE/LIMIT | L191-L192 | VLM correctness judgment for architecture diagrams is unproven and needs a separate evaluation; until then output is valid-not-verified. |
| H007-13-041 | E-SEC/TEST | L193-L194 | gate_freeze_check must freeze the fileset before implementation; otherwise the implementing agent can route its own gate to pass. |
| H007-13-042 | E-DEP/INT | L196-L199 | H-15 consumes this transport/dispatcher, H-14 sends design operations through it, and ARCHITECTURE plus media validation close the visual-explanation loop. |
| H007-13-043 | C-DATA | L66-L69 versus L145, L160 | The detail defines only one new CanvasOpApplied action type at baseline+1, while the orientation summary attributes CanvasActivityEvent and baseline+2 to H-13; the detail assigns activity elsewhere and is the original authority for this chunk. |

### H-15 — spawn_view reply channel, taint-gated deixis, and snapshot/delta

Source: H-15_spawn_view_reply_channel.md (203 lines). Status: planned Wave 2 build; canvas moat; depends on H-13, G-1, and G-2.

| Atomic ID | Class/domain | Exact source anchor(s) | Explicit feature or behavior |
|---|---|---|---|
| H007-15-001 | E-STATUS/DEP | L3, L15-L20 | H-15 is PLANNED Wave 2 BUILD, sequenced after H-13 and not parallel-safe with the operation transport; G-1/G-2 are compose-with dependencies. |
| H007-15-002 | E-CAP/FLOW | L5-L13 | The model may answer by spawning an interactive view rather than prose/ASCII, and the user may point back into it; the differentiator is the journaled, reproducible, provenance/taint-tagged operation stream. |
| H007-15-003 | E-CAP/DATA | L10-L13, L24-L27 | Add the three-arm spawn_view request with view_type discriminator and seed_ops array; view_type selects a registered renderer. |
| H007-15-004 | E-FLOW/STATE | L10-L13, L55-L77 | Work is sliced into H15-1 comparison board, H15-2 taint-gated inbound Select, and H15-3 per-tab snapshot/delta reconciliation. |
| H007-15-005 | E-DEP/STATE | L16-L18, L40-L46 | H-13 transport is assumed; taint_text is a partial replay seam, full G-1 TaintFlow and G-2 RRA are absent, E-37 provenance is wire-dark, and SSB lockstep is shipped at the dated 207/484 snapshot. |
| H007-15-006 | A-DEP/SEC | L16-L18, L43-L45, L189-L192 | H-15 must explicitly choose whether to wait for live G-1/G-2 or ship a weaker taint_text-scoped gate; the latter is necessary but insufficient because taint_text's non-test caller is replay-only. |
| H007-15-007 | E-INT/LIMIT | L24-L27, L60 | The operation stream, per-op SSB transport, modes, and tiered read-back are reused; the H-12 mode picker is generalized into a renderer registry rather than rewritten. |
| H007-15-008 | E-SEC/OBS | L28-L30 | Every ELAI view is intended to be journaled for replay/audit, reproducible through G-2 RRA, and taint-tagged through G-1, unlike render-and-forget surfaces. |
| H007-15-009 | E-SEC/FAIL | L31-L34, L66-L72 | Rendered untrusted content is a reinjection risk; selecting a tainted node must preserve a provenance flag at prompt handoff and may never silently convert it into trusted instruction. |
| H007-15-010 | E-DATA/FLOW | L34-L37, L74-L77 | Per-tab monotonically sequenced state plus periodic SSB checkpoints lets late/reconnecting dashboards rebuild from the journal tail and verifies model/user display-state agreement. |
| H007-15-011 | E-DATA/INT | L57-L60 | Add one Pattern-2 CanvasViewEvent with CanvasViewKind ViewSpawned, UiAction, and StateCheckpoint and CanvasViewPayload through the nine-tuple lockstep; re-read and bump schema baseline+1. |
| H007-15-012 | E-CAP/INT | L57-L60 | spawn_view is added through catalog, conversion, and dispatch arms mirroring H-13, and the five existing modes register first in renderer_registry.ts. |
| H007-15-013 | E-CAP/DATA | L62-L64 | ComparisonBoard renders N approaches, branches, or run artifacts on a shared row axis using existing fork operations and forked_from edges. |
| H007-15-014 | E-CAP/FLOW | L62-L64 | Clicking a comparison cell supports “expand this tradeoff” or “pick this branch”; the board requires no new renderer primitives. |
| H007-15-015 | E-DATA/OBS | L66-L68 | CanvasUiAction is typed as Select, Intent, Prompt, ToolCall, or Link and every click is journaled rather than sent as an ephemeral postMessage. |
| H007-15-016 | E-FLOW/CFG | L68-L71 | Select serializes current canvas state with Focused full detail for selected content, Blurry viewport context, and clustered off-screen context, binding the clicked node as a pronoun for the next model turn. |
| H007-15-017 | E-SEC/INT | L69-L72 | Select reuses capability_flow::taint_text or live G-1 at handoff and marks web-derived content with provenance. |
| H007-15-018 | E-SEC/FLOW | L72, L101, L110 | CanvasUiAction::ToolCall must traverse existing F-1/F-2 write-time security gates and explicit user consent; it never auto-executes. |
| H007-15-019 | E-FAIL/FLOW | L74-L77 | Reconnection starts from a periodic StateCheckpoint and journal tail instead of replaying every operation; sequence divergence requests a fresh snapshot. |
| H007-15-020 | E-CFG/SEC | L79-L82 | spawn_view is autonomy-widening and is allowed only above a routing-confidence threshold with a cheap suppress-canvas escape hatch; default response medium is text. |
| H007-15-021 | E-OBS/TEST | L81-L82 | Measure canvas-spawn-versus-text rate from H15-1 and judge ELAI's comprehension/ambiguity delta rather than assuming interactivity helps. |
| H007-15-022 | E-DATA/OBS | L84-L87 | H-15-decisions.md records the new abstraction, trust-spine, prompt-injection gate, snapshot/deixis need, autonomy controls, non-proven paper claim, and compose-not-restate decision. |
| H007-15-023 | E-INT | L89-L102 | Produced surfaces are spawn_view arms, CanvasViewEvent types, renderer registry, ComparisonBoard, CanvasUiAction, and tab reconciliation; they consume taint_text/G-1/G-2, H-13, F-1/F-2, and SSB lockstep. |
| H007-15-024 | E-FLOW/INT | L103-L105 | Production paths are model reply dispatch to CanvasViewEvent to renderer/board and live canvas Select through taint check into next-turn model context. |
| H007-15-025 | E-LIMIT/INT | L107-L111 | Do not reimplement taint or RRA and do not create a second canvas transport: CanvasViewEvent is view lifecycle/UI state while H-13 CanvasOpApplied remains operation application. |
| H007-15-026 | E-LIMIT/CFG | L112-L114 | Autonomous spawn without confidence/suppress controls, quantitative done-gate claims from arXiv:2603.26942, non-blake3 hashing, and production unwrap/expect are forbidden. |
| H007-15-027 | E-TEST/OBS | L115-L125 | Frontend/Tauri surfaces must pass cross-language and dashboard E2E; an auditor runs a frozen pre-authored board reply, tainted Select, and kill/reopen flow with no implementation-authored proof. |
| H007-15-028 | E-DATA/OBS | L118-L125 | The trigger must emit ViewSpawned, UiAction, and StateCheckpoint rows, render the board, hand exact current selection state plus taint flag to the model, rebuild from the tail, and request a fresh snapshot on forced divergence. |
| H007-15-029 | E-DET/TEST | L127-L137 | Deterministic checks verify wiring trigger, spawn tool, CanvasViewEvent/golden, reuse of taint_text, reconciliation file, cross-language diff, and decision record. |
| H007-15-030 | E-CFG/TEST | L139-L148 | spawn_view is default-on behind its routing confidence gate, not an environment enable switch, and includes a suppress-canvas hatch. |
| H007-15-031 | E-TEST/FLOW | L140-L148 | Live model reply and live canvas Select, schema roundtrip/golden, cross-language parity, taint gate, ToolCall consent, workspace build, Svelte check, and auditor pass are required. |
| H007-15-032 | E-TEST/OBS | L150-L161 | A runnable board reply plus Select plus restart must leave real journal rows and a reconstructed dashboard tab; snapshot/delta is journal-backed and in-process. |
| H007-15-033 | E-TEST | L163-L169 | Contract tests cover view emission/rendering, five-mode registry lookup, exact Focused selection, taint flag, ToolCall consent, tail reconstruction/fresh-snapshot recovery, and all three CanvasViewKind roundtrips. |
| H007-15-034 | E-DATA/INT | L171-L177 | Implementation modifies production RPC/IPC and SSB locks/contracts, and creates CanvasUiAction, registry/board/reconciliation, contract tests, and decision record. |
| H007-15-035 | E-LIMIT/OBS | L179-L187 | AG-UI/MCP Apps/tldraw/Anthropic are grounding for internal design convergence; external claims and the arXiv quantitative claim remain motivating, not completion evidence. |
| H007-15-036 | A-CFG/SEC | L193-L194 | V1 autonomy remains an explicit product choice: confidence-gated model-initiated views versus requiring a user request for every spawn. |
| H007-15-037 | E-LIMIT/OBS | L195-L196 | The intermediate-state-steering paper is small/single-domain and involved human code fixes, not a UI; its quantitative result must stay out of the gate. |
| H007-15-038 | E-DEFER/SEC | L197 | Publish/share-a-canvas is deferred and, if ever built, must start behind G-8 attestation because it inherits mass-exploitation authorization risk. |
| H007-15-039 | E-DEP/INT | L199-L201 | H-16 follows after H-15; the completed surface is described as the model's journaled, reproducible, taint-tagged reply channel over the F/G trust spine. |
| H007-15-040 | A-DATA/STATE | L10-L13, L57-L60, L66-L77 | CanvasViewEvent is described once as one Pattern-2 action type with three kinds, while CanvasUiAction lists five subtypes but no exact payload/sequence schema; ordering, checkpoint cadence, and retained-tail length remain unspecified. |

### H-16 — Retire the orbit

Source: H-16_retire_orbit.md (162 lines). Status: planned Wave 2 deletion; final plan session.

| Atomic ID | Class/domain | Exact source anchor(s) | Explicit feature or behavior |
|---|---|---|---|
| H007-16-001 | E-STATUS/STATE | L3, L11-L15 | H-16 is a PLANNED Wave 2 DELETE, is last in the plan, is not parallel-safe, and takes place only after the canvas migration. |
| H007-16-002 | E-DEP/TEST | L5-L13, L50-L51, L107 | Canvas-as-default must have shipped a full regression-free release and orbit must have remained a fallback tab until then; otherwise H-16 does not run. |
| H007-16-003 | E-STATE/LIMIT | L5-L9, L40 | Canvas-store isolation already passes because no Svelte rune store imports three.js, making removal mechanically separable. |
| H007-16-004 | E-CAP/STATE | L7-L9, L19-L28 | The orbit's “alive” aesthetic is preserved by the H-10/H-11 flat 2D starfield/live layer, while watch-only heavy nondeterministic 3D and editing-hostile infrastructure are removed. |
| H007-16-005 | E-CFG/FLOW | L22-L25 | Migration keeps orbit fallback through the workspace-flag default flip and single-consumer handover; deletion happens only after the full-release proof. |
| H007-16-006 | E-DATA/CAP | L25-L28, L57-L60 | Delete scene/three-scene.ts, three-orbit.ts, three-orbit-modes.ts, StarCanvas.svelte, sky-dome.ts, star-points.ts, mesh material/geometry code, node-mandala.ts, OrbitCanvas.svelte, ui/Starfield.svelte, and the orbit role in ActivityWindow. |
| H007-16-007 | E-DATA/INT | L31-L38 | Current live chain is three/troika dependencies to scene modules, NodeMandala, StarCanvas/OrbitCanvas, Starfield shim, ActivityWindow, and the default activity panel. |
| H007-16-008 | E-DATA/LIMIT | L34-L35, L60, L76, L82 | orbit/lcg.ts, including lcg/hashId/combineSeeds/jitterFromId, is live and must be kept as the canvas seed source. |
| H007-16-009 | E-STATE/FAIL | L39, L53-L55 | planet-scan.ts, star-shoot.ts, and star-state.ts are partial with an unpinned production caller and must be traced to the orbit chain before deletion. |
| H007-16-010 | E-CAP/CFG | L57-L60, L71-L72 | Remove the orbit renderer or ActivityWindow if it has no non-orbit role and change workspace.svelte.ts primary panel from activity to canvas. |
| H007-16-011 | E-DATA/INT | L62, L70 | Remove three, troika-three-text, and @types/three from dashboard package metadata. |
| H007-16-012 | E-DATA/OBS | L64-L66, L139 | H-16-decisions.md records the release gate, passing isolation, kept lcg, partial-file trace, surviving 2D starfield, and galaxy-toggle decision. |
| H007-16-013 | A-CAP/CFG | L44-L46, L64-L66, L84, L112, L152-L153 | The plan requires an explicit choice between a permanent calm-mode galaxy toggle and full deletion; it does not select one and forbids silently removing the option. |
| H007-16-014 | E-FLOW/INT | L68-L79 | Production evidence is a green dashboard build with removed dependencies and imports plus workspace default routing to canvas; there is no replacement runtime service. |
| H007-16-015 | E-LIMIT/FAIL | L80-L85 | Early deletion, deleting lcg, deleting partial files before tracing, silent galaxy-choice removal, remaining scene imports, or remaining three dependencies fail the gate. |
| H007-16-016 | E-OBS/LIMIT | L88-L93, L124 | H-16 adds no SSB event; the off-agent artifact is absence: green build, zero grep, removed dependencies, and rendered canvas without regression. |
| H007-16-017 | E-DET/TEST | L95-L104 | Deterministic checks run the wiring trigger, require empty scene/troika and package greps, require StarCanvas absent and lcg present, verify default-panel flip, and require the decision record. |
| H007-16-018 | E-TEST | L106-L113 | Acceptance requires the full-release precondition, production default canvas, traced partial files, kept lcg, surfaced galaxy decision, dashboard-smoker, workspace build, Svelte check, and auditor pass. |
| H007-16-019 | E-TEST/OBS | L115-L125 | Done-gate treats the green build plus zero grep as mechanical off-agent proof; cross-language diff and SSB changes are not applicable. |
| H007-16-020 | E-TEST | L127-L130 | A build-level CI gate, not a unit test, runs cargo check, Svelte check, source zero-grep, and dashboard-smoker with no console error or orbit import. |
| H007-16-021 | E-DATA/INT | L132-L139 | File inventory distinguishes deletions, conditional ActivityWindow deletion, package/default edits, explicitly kept lcg, and created decision document. |
| H007-16-022 | E-FAIL/SEC | L147-L155 | The release prerequisite and partial-file trace are hard gates; deletion is identified as the plan's only irreversible operation and invokes the H-0 tripwire posture before landing. |
| H007-16-023 | E-DEP/STATE | L157-L160 | H-16 closes Track 2 only after canvas is default and the journaled/reproducible/taint-tagged reply surface has shipped; Track 1 measurement and Track 2 product share the wiring-enforced gate. |
| H007-16-024 | A-DATA | L57-L60, L72, L133-L135 | The wildcard “scene/*” in the wiring contract is broader than the enumerated file list and conditional traced files; exact deletion set must be resolved by the mandated reachability trace. |

### H-18 — Deflated-Sharpe, PBO, and MinBTL promotion gate

Source: H-18_deflated_sharpe_pbo_gate.md (180 lines). Status: planned Track 3/Wave 3; Tier-1 rules gate plus frozen Tier-2 evaluator; depends on H-17.

| Atomic ID | Class/domain | Exact source anchor(s) | Explicit feature or behavior |
|---|---|---|---|
| H007-18-001 | E-STATUS/STATE | L3, L13-L18 | H-18 is PLANNED Track 3/Wave 3 with an in-process Tier-1 fail-closed rules gate and a frozen NumPy/SciPy Tier-2 sidecar plus fixture/live split. |
| H007-18-002 | E-STATE/CAP | L5-L11, L40 | No finance-deflation evaluator is wired to promotion; H-18 builds the absent DSR/PBO/MinBTL surface. |
| H007-18-003 | E-CAP/FLOW | L7-L11, L26-L30 | A LOCKED COMPOSITE blocks strategy promotion unless DSR, PBO via CSCV, and series length versus MinBTL all clear their thresholds. |
| H007-18-004 | E-FAIL/DATA | L8-L11, L27, L58-L62 | Effective trial count N is read from H-17 trial_count; N=0 for the reported configuration fails closed as trial_count_missing. |
| H007-18-005 | E-SEC/CFG | L10-L11, L28-L30, L58-L62 | Sidecar math, threshold values, and mutable=0 rule source are frozen so the self-modifying agent cannot lower them; no LLM participates in the verdict. |
| H007-18-006 | E-OBS/DATA | L10-L11, L64-L67 | Persist dsr, dsr_pvalue, pbo, minbtl, n_used, and verdict as the off-agent promotion artifact, recomputable from journal inputs. |
| H007-18-007 | E-DEP/INT | L16-L18 | H-17 is a hard prerequisite; shipped COMPOSITE/MetricGate and LOCKED mutable=0 tiers are reused; H-19/H-20/H-21 are disjoint, but H-17 is not parallel-safe. |
| H007-18-008 | E-DATA/LIMIT | L23-L27 | DSR adjusts observed Sharpe for trial count, non-normality, and sample length; PBO estimates in-sample-best underperformance out of sample; MinBTL rejects statistically meaningless short backtests. |
| H007-18-009 | E-SEC/FLOW | L27-L30 | Deterministic numeric logic, not an LLM/auditor opinion, owns the promotion verdict; frozen-runner/Rust-enforcer/measured-row behavior mirrors SecCodePlt/SAFEDELTA. |
| H007-18-010 | E-STATE/INT | L32-L40 | CompiledPredicate, eval order, VerdictResult, mutable=0 lifecycle, SecCodePltRunner, and measured-row pattern are shipped; only the finance evaluator is new. |
| H007-18-011 | E-CAP/DATA | L52-L56 | compute_dsr_pbo_minbtl calculates skew/kurtosis-adjusted DSR deflated by N, CSCV PBO, and MinBTL from return series, N, and user-LOCKED alpha. |
| H007-18-012 | E-DATA/DET | L54-L56 | Sidecar output is dsr, dsr_pvalue, pbo, minbtl, and n_used; it uses no wall clock or unseeded RNG, and CSCV partitioning uses ELAI_SEED. |
| H007-18-013 | E-DATA/SEC | L56, L73 | The evaluator lives in a dedicated frozen Python module and the evaluator, thresholds, and COMPOSITE source enter gate-manifest.txt. |
| H007-18-014 | E-CAP/CFG | L58-L62 | The Rust finance-domain mutable=0 rule orchestrates the sidecar and defaults the DSR condition to Harvey-Liu deflated t greater than 3.0. |
| H007-18-015 | E-CFG/FAIL | L60-L62 | Promotion also requires pbo below a locked threshold and observed series length at least MinBTL; any missing verdict is BLOCK, never SKIP. |
| H007-18-016 | E-DATA/STATE | L64-L67, L84 | DeflationGateEvaluated is one Pattern-2 baseline+1 event following the measured-row pattern; live schema and e45 pins must be re-read at implementation time. |
| H007-18-017 | E-OBS/SEC | L64-L67 | Return series and N are off-agent inputs not authored by the optimizing agent, and the persisted evaluator record is journal-recomputable. |
| H007-18-018 | E-TEST/DEFER | L69-L71 | Ship real-sidecar deterministic fixtures now: high-N known-overfit must fail and low-N genuine must pass; a live trading strategy run is a separately gated deferred checkbox with no prior external claim. |
| H007-18-019 | E-DATA/OBS | L75-L78 | H-18-decisions.md records formulas/threshold grounding, trial-count fail-close, tier/freeze rationale, and the necessary-not-sufficient/human-capital-allocation boundary. |
| H007-18-020 | E-INT | L80-L89 | Produced surfaces are evaluate_deflation, compute_dsr_pbo_minbtl, DeflationGateEvaluated, and deflation_promotion.toml; they consume H-17 trial_count, rules-engine CompiledPredicate, and SAFEDELTA runner pattern. |
| H007-18-021 | E-FLOW/INT | L90-L93 | Live strategy promotion invokes evaluate_deflation before admission; it reads trial_count and emits DeflationGateEvaluated. |
| H007-18-022 | E-LIMIT/FAIL | L94-L99 | Missing math, agent-tunable thresholds, LLM verdicts, N=0 promotion, duplicate trial ledger/rules engine, dishonest fail-open sidecar claims, non-blake3 hashing, and production unwrap are forbidden. |
| H007-18-023 | E-CAP/OBS | L100-L102 | A trust-cockpit drawer shows DSR, PBO, and N, with cross-language parity and dashboard evidence for the failing DSR-at-N. |
| H007-18-024 | E-TEST/OBS | L103-L109 | A no-override fixture promotion must journal Block with full evaluator fields; high raw Sharpe/failing DSR blocks, genuine low-N passes, and no-trial configuration blocks with trial_count_missing. |
| H007-18-025 | E-DET/TEST | L111-L118 | Deterministic checks exercise the wiring trigger, verify trial_count consumption and mutable=0 seed, ban blake2/sha2 in gate code, and require the decision record. |
| H007-18-026 | E-CFG/TEST | L120-L130, L132-L143 | The gate runs on every promotion with no enable override and only a kill switch; live production reachability, frozen math, deterministic verdict, substrate reuse, tier honesty, schema bump, build, and auditor pass are required. |
| H007-18-027 | E-DET/LIMIT | L127-L130, L172-L173 | Reproducibility is scoped to same environment and same ELAI_SEED; verdict must remain stable across minor floating-point last-digit drift by using a threshold band rather than exact equality. |
| H007-18-028 | E-TEST | L145-L149 | Contract tests cover failing high-N, passing low-N, N=0 failure, runtime immutability of thresholds, LLM independence, and same-seed equality using frozen series fixtures. |
| H007-18-029 | E-DATA/INT | L151-L159 | Implementation creates Rust gate, Python evaluator, locked finance seed, contracts/fixtures/decision record; modifies SSB locks, promotion path, and gate manifest. |
| H007-18-030 | E-SEC/DEP | L161-L166 | DSR/PBO/MinBTL formulas and any threshold need paper ingest with quote/page before coding; NumPy/SciPy availability must be anti-slopsquat verified rather than assumed. |
| H007-18-031 | E-FAIL/TEST | L168-L170 | Formula details such as skew/kurtosis and CSCV partition count are subtle; H-27 must mutation-test this composite and its bands. |
| H007-18-032 | A-CFG/LIMIT | L60-L62, L75-L78, L171 | MinBTL relevance and the exact PBO/alpha thresholds are not fixed in this plan; they require verification before the mutable=0 constants are locked. |
| H007-18-033 | E-DEP/INT | L175-L178 | H-19 constrains adaptive holdout use, H-25 binds reported deflation metrics to provenance, and H-22 null tests plus H-18 are jointly necessary for a trustworthy edge. |
| H007-18-034 | C-DATA | L10-L11 versus L64-L67 | The objective calls the persisted artifact a DSR/PBO/N “triple,” but the declared row contains six values including dsr_pvalue, minbtl, and verdict; consumers must use the explicit payload, not the shorthand. |

### H-26 — Runtime shadow differential and financial-invariant metamorphic library

Source: H-26_shadow_exec_financial_invariant_mr.md (171 lines). Status: planned Track 3/Wave 3 Cluster E; Tier-1 gate/MR plus Tier-2 runner.

| Atomic ID | Class/domain | Exact source anchor(s) | Explicit feature or behavior |
|---|---|---|---|
| H007-26-001 | E-STATUS/STATE | L3, L13-L18 | H-26 is PLANNED Track 3 Wave 3 Cluster E with a fail-closed in-process divergence/MR/event tier and a fixture/live Tier-2 shadow runner. |
| H007-26-002 | E-CAP/FAIL | L5-L11, L22-L29 | It targets wrong numeric values that compile and pass ordinary tests, including P&L sign reversal, cancellation error, and violated financial identities. |
| H007-26-003 | E-CAP/DET | L6-L11, L49-L52 | A seeded runner executes two implementations on identical inputs and a Tier-1 MetricGate blocks divergence beyond a frozen ULP/relative bound. |
| H007-26-004 | E-CFG/INT | L7-L8, L49-L52 | Comparison candidates are new versus prior version, a paraphrase variant, or an optional QuantLib oracle, all under ELAI_SEED. |
| H007-26-005 | E-CAP/DATA | L8-L11, L54-L56 | Extend the shipped relation library with PutCallParity, StrikeMonotonicity, PnLAdditivity, ScaleInvariance, and NumericStability, seeded from QuantLib invariant tests. |
| H007-26-006 | E-SEC/FLOW | L9-L11, L61-L63 | Daikon-mined invariants enter only as PROPOSED and may become ACTIVE only after killing a seeded mutant through the shipped mutation runner. |
| H007-26-007 | E-CAP/SEC | L9, L58-L59 | A finance AST_PATTERN blocks naked float == or != at TESTING_FIRST and requires tolerance comparison. |
| H007-26-008 | E-OBS/DATA | L10-L11, L65-L67 | Divergence and falsification records are the off-agent proof; per-invariant failures continue to reuse MrViolated. |
| H007-26-009 | E-DEP/INT | L15-L17 | Shipped metamorphic framework, mutation runner, MetricGate, and AST_PATTERN are prerequisites and must be reused; H-25/H-27 are file-disjoint. |
| H007-26-010 | E-STATE/INT | L31-L37 | Metamorphic and mutation frameworks plus rules predicates are shipped; the seeded shadow executor, named finance pack, and naked-equality ban are absent. |
| H007-26-011 | E-CAP/FLOW | L49-L52 | diff_shadow_impls treats a sign-flipped refactor as beyond-bound failure and a clean refactor within bound as pass. |
| H007-26-012 | E-DATA/INT | L54-L56 | Financial relations are new named MetamorphicRelation implementations in the existing RelationLibrary/MetamorphicTestRunner, not a second MR subsystem. |
| H007-26-013 | E-DATA/STATE | L65-L67, L82 | Add one Pattern-2 DifferentialDivergence with kinds ShadowDiverged, InvariantViolated, and NakedFloatEq; re-read baseline and update e45/golden lock. |
| H007-26-014 | E-TEST/DEFER | L69-L70 | Ship sign-flip, clean-refactor, and parity-violation fixtures through the real shadow path now; live shadow execution is a separately gated checkbox. |
| H007-26-015 | E-DATA/OBS | L72-L74 | H-26-decisions.md records composite behavior, QuantLib and mined-invariant discipline, and off-agent evidence semantics. |
| H007-26-016 | E-INT | L76-L86 | Produced surfaces are shadow_exec.rs, frozen Python runner, five MR types, locked finance rule seed, and DifferentialDivergence; consumers are shipped MR, mutation, MetricGate, and AstPattern surfaces. |
| H007-26-017 | E-FLOW/INT | L87-L89 | Production reachability is the pre-promotion path plus financial-code refactor/commit hook through shadow runner to divergence SSB. |
| H007-26-018 | E-FAIL/LIMIT | L90-L95 | Missing second implementation record fails, mined invariant without a mutant kill stays proposed, naked equality blocks, framework duplication is forbidden, and Tier-2 may not advertise fail-closed while fail-open. |
| H007-26-019 | E-SEC/LIMIT | L94 | Use blake3 and no production unwrap/expect. |
| H007-26-020 | E-CAP/OBS | L96-L101 | Trust cockpit shows divergence/falsification; a no-override sign-flip run journals ShadowDiverged, while clean refactor passes and parity/naked-equality failures are visible. |
| H007-26-021 | E-DET/OBS | L99-L101, L117 | The divergence record is journal-recomputable under same-environment ELAI_SEED. |
| H007-26-022 | E-DET/TEST | L103-L110 | Deterministic checks run the wiring trigger, find gate/MRs/MutationRunner, verify finance rule/equality ban, reject blake2/sha2, and require the decision record. |
| H007-26-023 | E-CFG/TEST | L112-L119 | Gate runs default-on on every financial refactor/promotion with no enable override and only a kill switch; production hooks, mined-invariant admission, reuse, tier split, build, and auditor pass are gates. |
| H007-26-024 | E-DEP/LIMIT | L119, L148-L154 | QuantLib is opt-in Tier 2 and dependency identity must be verified; external FPDiff/Daikon/QuantLib sources ground but do not replace ELAI tests. |
| H007-26-025 | E-TEST | L130-L135 | Contracts cover sign-flip, clean result, parity violation, naked equality, non-killing proposal retention, killing promotion, and same-seed equality with frozen fixtures. |
| H007-26-026 | E-DATA/INT | L137-L146 | Implementation creates shadow gate/runner/locked rule/contracts/fixtures/decision record; modifies MR library, SSB locks, live hooks, and gate manifest. |
| H007-26-027 | E-FAIL/TEST | L156-L159 | ULP/relative bound must be calibrated on at least three real refactors before default-on and is itself an H-27 mutant target. |
| H007-26-028 | E-LIMIT/FAIL | L160-L162 | QuantLib convention differences can look like bugs, so QuantLib is advisory T3; the hard gate is new-versus-prior on the same numeric stack. |
| H007-26-029 | E-FAIL/TEST | L163-L164 | Admission must reject a trivially true mined invariant; kill-a-seeded-mutant is the load-bearing noise filter. |
| H007-26-030 | E-DEP/INT | L166-L169 | H-27 recursively mutation-tests H-26's ULP bound and other gates; H-28 protects reward integrity. |
| H007-26-031 | A-DET/CFG | L51-L52, L81, L144, L156-L159 | The frozen ULP/relative formula and numeric bound are required but not specified; calibration is deferred to measured refactors before mutable=0 locking. |
| H007-26-032 | A-FLOW/INT | L49-L52, L79, L88-L89 | The plan permits prior, paraphrase, or QuantLib comparison but does not specify how the production hook selects/locates the second executable implementation. |

### H-34 — MCP defer-loading and Tool Search emulation on live STDIO

Source: H-34_mcp_defer_loading_tool_search.md (50 lines). Status: Wave 4 BUILD, effort M, distinct from H-4 and composing with C-35.

| Atomic ID | Class/domain | Exact source anchor(s) | Explicit feature or behavior |
|---|---|---|---|
| H007-34-001 | E-STATUS/DEP | L2 | H-34 is Track 4, Wave 4, BUILD, effort M, with no hard dependency; it composes C-35 and H-4 but has a distinct external-schema objective. |
| H007-34-002 | E-STATE/LIMIT | L3 | Live local MCP sends every server tool's full inputSchema, creating a large fixed tax on a roughly 25–32K Qwen window; reduction must be measured from ELAI journal data, not borrowed. |
| H007-34-003 | E-CAP/FLOW | L3 | Measure baseline first, then inject only a small floor plus selected schemas, retaining other schemas for on-demand first use. |
| H007-34-004 | E-STATE/DATA | L6-L8 | C-35 LazySchemaCache already provides LRU resolve, hit/miss/latency counters, and no-silent-caching contract; DefaultMcpBridge wrapper emits on cache miss but direct-lookups without a cache emit nothing. |
| H007-34-005 | E-INT/LIMIT | L9 | Reimplement Odysseus's base union vector-top-K union keyword-force-include algorithm in Rust using ELAI ONNX/reranker; do not port Python or add Chroma/another embedding stack. |
| H007-34-006 | E-CFG/CAP | L9, L20 | Keep an intentionally small always-available floor at no more than about six tools and select additional tools by description retrieval plus keyword force-includes. |
| H007-34-007 | E-STATE/INT | L12 | Production uses RuntimeConfig-driven McpServerManager STDIO discovery through mcp_glue/live REPL; the three named harness servers are not hardcoded ELAI servers. |
| H007-34-008 | E-STATE/DATA | L13 | McpTool deserializes inputSchema verbatim and discovery inserts every tool into tool_index with no size gate or deferral. |
| H007-34-009 | E-STATE/INT | L14-L15 | Existing lazy cache is attached to a dormant HTTP CliMcpClient with zero non-test callers; live STDIO has no lazy resolver, token measurement, selector, or deferred-load event. |
| H007-34-010 | E-CAP/DET | L18-L20 | mcp_schema_budget_runner reads configured servers or a filesystem-independent include_str fixture and computes baseline_schema_tokens_per_turn by tokenizing every serialized schema with the local lane's tokenizer. |
| H007-34-011 | E-DATA/DET | L19 | Runner corpus is pinned by corpus_blake3, seed comes from ELAI_SEED, and before/after numbers are computed rather than literal expectations. |
| H007-34-012 | E-CFG/DEFER | L20, L30 | ELAI_MCP_DEFER_SCHEMA_ENABLE is initially default-off and requires a wire-dark allowlist experiment row with owner, expiry, promotion-plan ID, and real ON-path smoke. |
| H007-34-013 | E-FLOW/DATA | L20, L25-L26 | When enabled, discovery retains names/short descriptions in injected context, moves full schemas into Arc<LazySchemaCache>, and call_tool resolves a withheld schema on first model use. |
| H007-34-014 | E-DATA/OBS | L20, L29 | Each first/deferred resolution writes mcp_tool_deferred as MultiAgentSpan::McpToolDeferred with server_name, qualified tool name, resolve_ns, cache_hit, and injected_floor_size. |
| H007-34-015 | E-CAP/OBS | L21, L27 | After a real transcript, the runner computes injected tokens/reduction/deferred resolves, writes timestamped benchmarks/mcp_schema_budget report JSON, and emits a result beacon. |
| H007-34-016 | E-DATA/INT | L27 | McpSchemaBudgetReport contains computed baseline, injected, reduction, deferred_resolves, seed, and corpus_blake3; CLI registers elai bench mcp-schema-budget beside existing benchmark dispatches. |
| H007-34-017 | E-DEFER/LIMIT | L22 | Semantic schema deduplication, cross-server sharing, and default-on promotion are deferred pending a measured positive reduction and promotion record. |
| H007-34-018 | E-CFG/TEST | L22 | Always-injected MCP slice is limited to floor plus descriptions at about 500 tokens and the runner fails if the cap is exceeded. |
| H007-34-019 | E-DATA/LIMIT | L22 | MCP schema state is refreshed at server discovery and is not durable user memory, so staleness/supersession lifecycle is not applicable. |
| H007-34-020 | E-INT/FLOW | L24-L27 | Exact live sites are mcp_stdio discovery/index insertion, mcp_glue call_tool resolution, orchestrator benchmark runner, and CLI benchmark command/result-beacon dispatch. |
| H007-34-021 | E-DEFER/CAP | L28 | No Tier-1 frontend is required because degraded MCP health already exists; a schema-budget dashboard tile is deferred. |
| H007-34-022 | E-DATA/STATE | L29 | H-34 adds two SSB variants: Pattern-2 mcp_tool_deferred and flat mcp_schema_budget_measured with suite, token counts, reduction, seed, and corpus hash. |
| H007-34-023 | E-DATA/TEST | L29 | The dated live 220/497 baseline requires two lockstep additions to 222/499, subject to re-read at authoring; the older CLAUDE.md baseline is explicitly stale. |
| H007-34-024 | E-CFG/FLOW | L30 | Default-off is not terminal; promotion to default-on is allowed only after a measured positive reduction_tokens delta. |
| H007-34-025 | E-INT | L32-L35 | Produced report/events/live selector consume LazySchemaCache counters, STDIO schemas, ONNX retrieval, ELAI_SEED, and existing runner shape; real entrypoints are bench command and --via-mcp call_tool, not direct unit calls. |
| H007-34-026 | E-FAIL/LIMIT | L36 | With the gate on, silently injecting all eager schemas is forbidden; LazyCacheError/tool-name mismatch must surface rather than create a stub schema; report literals and silent cache behavior are forbidden. |
| H007-34-027 | E-TEST/OBS | L37 | Done requires baseline bench, a real enabled --via-mcp withheld-tool call, an off-agent deferred row, floor retention, injected tokens below baseline and cap, and journal recomputation under fixed seed; missing row or non-positive delta fails. |
| H007-34-028 | E-LIMIT/OBS | L40-L41 | Anthropic's reported token/accuracy improvements and the third-party 65x claim are motivating ceilings only, never ELAI measurements. |
| H007-34-029 | E-STATE/FAIL | L42 | Custom/local ANTHROPIC_BASE_URL disables provider-side Tool Search, so ELAI must implement deferral on its own live lane rather than flip provider configuration. |
| H007-34-030 | E-INT/LIMIT | L43 | Odysseus code is MIT prior art and may supply algorithm shape only; ELAI uses its own stack and blake3. |
| H007-34-031 | E-INT/LIMIT | L46-L49 | C-35 cache is activated rather than duplicated; H-4 handles built-in production-agent tools, not MCP schemas; deferred resolution remains downstream of G-4 discovery-time rug-pull checks. |
| H007-34-032 | E-SEC/TEST | L50 | Inherited W/G/H doctrine requires a live off-agent row and measured recomputable delta; cargo-green alone is insufficient. |
| H007-34-033 | A-DATA/FLOW | L20, L22 | The text says only floor plus descriptions are always injected, but also defines the per-turn set as floor plus RAG/keyword tools; it does not state precisely whether selected non-floor tools carry full schemas immediately or names/descriptions until named. |
| H007-34-034 | A-CFG/DET | L9, L20, L22 | Prior art uses retrieve k=8, while H-34 states generic RAG top-K and approximate floor/token caps; exact K, floor membership, tokenizer fallback, and selection tie-breaking are not locked. |
| H007-34-035 | A-DATA/OBS | L8, L20, L29 | C-35 emits only on cache-backed resolution, while the new event is described as a row per deferred schema-resolve and includes cache_hit; whether repeated cache hits also emit rows needs lifecycle clarification. |

### H-37 — Plan-approval gate at PLANNING to TESTING_FIRST

Source: H-37_plan_approval_gate.md (67 lines). Status: Wave 4 build-on-existing pre-gate; soft dependency on H-36.

| Atomic ID | Class/domain | Exact source anchor(s) | Explicit feature or behavior |
|---|---|---|---|
| H007-37-001 | E-STATUS/DEP | L2 | H-37 is Track 4/Wave 4 BUILD-on-existing, effort S-M, with soft H-36 notification reuse. |
| H007-37-002 | E-CAP/FLOW | L3 | Add approve, redirect, or reject at PLANNING/FEASIBILITY_PROBE to TESTING_FIRST so wrong assumptions stop before edits. |
| H007-37-003 | E-STATE/INT | L3, L15-L18 | A four-assertion compliance pre-gate exists at the same edge, but compliance_provider defaults None and has no found CLI install, so policy machinery is runtime-dark despite being source-wired. |
| H007-37-004 | E-INT/CAP | L6-L7 | Reuse run_interactive_gate's Approve/Reject/Modify/Redirect protocol, risk auto-advance, and maximum-three MODIFY reevaluation loop; add one production call rather than another gate engine. |
| H007-37-005 | E-INT/STATE | L8, L20-L23 | Reuse the already constructed CliInteractiveGate from RuntimeWiring; its current live install is never consumed because run_interactive_gate callers are tests and task_runner only observes handle presence. |
| H007-37-006 | E-FLOW/SEC | L9, L25-L27 | Autonomous mode reuses AutonomousGate::decide and its six-check Evidence: low risk auto-advances, medium/high requests a human, with existing disable switch and emission helper. |
| H007-37-007 | E-DATA/INT | L10, L27 | Add one Pattern-2 plan_decision_event whose payload mirrors existing GateDecisionPayload, plus one dashboard approval card and one live FSM call site. |
| H007-37-008 | E-FAIL/FLOW | L10, L33 | Rejection is fail-closed and redirect is accepted only after validation against the transition matrix. |
| H007-37-009 | E-STATE/DATA | L13 | Live schema snapshot is 220/497 and the older 191/464 doctrine constants are stale. |
| H007-37-010 | E-DATA/SEC | L16-L18 | ComplianceVerdict is Pass with assertion count or Block with assertion/hint; assertions are ClockProviderIntact, DeterministicSeed, TemplateHashChain, and AuditTrailLive with first-failure reporting. |
| H007-37-011 | E-FLOW/FAIL | L17-L18, L43 | If an installed ComplianceProvider returns Block, that hard failure pre-empts plan approval; if provider is absent, plan approval still runs at the edge. |
| H007-37-012 | E-CFG/OBS | L17, L45 | Existing compliance policy block is unconditional when provider exists, but its row is gated by ELAI_AI_ROADMAP_SIGNALS_ENABLE; plan-decision observability remains independently gated. |
| H007-37-013 | E-DATA/FLOW | L21 | GateDecision supports Approve, Reject, Modify with updated context, and Redirect with FsmState; outcomes include Approved, Rejected, Redirected, EvidenceFailed, and InvalidTransition. |
| H007-37-014 | E-CFG/FLOW | L22 | CliInteractiveGate auto-approves LOW risk; otherwise it prints evidence, accepts approve/a/yes/y, and treats every other stdin answer as reject. |
| H007-37-015 | E-CFG/STATE | L26 | GateMode defaults to Interactive and maps “hybrid” to Interactive; ELAI_DISABLE_AUTONOMOUS_GATE is the existing autonomous kill switch. |
| H007-37-016 | E-CAP/FLOW | L29-L34 | Interactive mode calls installed CliInteractiveGate with plan summary; autonomous mode records low-risk auto_advance or surfaces medium/high RequestHuman to PlanApprovalCard. |
| H007-37-017 | E-DATA/OBS | L32 | PlanDecisionPayload contains decision approve/redirect/reject/auto_advance, target_state, risk_level, reason, and plan_session_id; runtime emits the append-only row at transition time. |
| H007-37-018 | E-CAP/INT | L34, L46 | PlanApprovalCard displays pending plan and three decision buttons in interactive mode and recorded autonomous decision; a Tauri command mirrors the replay-read pattern and returns button decisions through the gate path. |
| H007-37-019 | E-DEFER/FLOW | L36 | First cut maps MODIFY to redirect back to PLANNING; multi-round MODIFY, learned rejection memory, and real ComplianceProvider installation are deferred. |
| H007-37-020 | E-LIMIT/SEC | L38 | The gate receives bounded EvidenceSummary built on demand, not full plan text; it has no always-injected block and no user-memory input under the auditor-objectivity firewall. |
| H007-37-021 | E-DATA/LIFECYCLE | L39 | Plan decisions are immutable decision-at-instant rows without bitemporality/supersession; each later redirect creates a new row and dashboard uses newest created_at. |
| H007-37-022 | E-FLOW/INT | L41-L43 | Orchestrator invokes approval after compliance and before committing the real FSM transition, consuming handles threaded through TaskRunnerConfig/RuntimeWiring rather than a CLI-only shim. |
| H007-37-023 | E-CFG/FLOW | L43 | default_risk_level makes Planning to TestingFirst MEDIUM, so it pauses/surfaces unless project fsm.risk_overrides changes it to LOW. |
| H007-37-024 | E-DATA/INT | L44 | Add SsbActionType, PlanDecisionPayload, SsbPayload arm, as_str/parse/golden, and single-writer emission; use one PlanDecisionKind discriminator and re-read/bump 220/497 to baseline+1. |
| H007-37-025 | E-CFG/SEC | L45 | Live call is behind default-off ELAI_PLAN_APPROVAL_GATE_ENABLE and requires a kill_switch allowlist row owned by H-37, expiring 2026-09-04, promotion ID H-37, and a real ON-path smoke row. |
| H007-37-026 | E-FLOW/INT | L48-L51 | Produced decision row/refusal behavior consumes interactive/autonomous/risk/compliance seams and is reachable from live elai run transition; card is reachable from a registered Tauri command. |
| H007-37-027 | E-FAIL/SEC | L52 | Presence-only handle consumption is banned; reject must refuse, redirect must validate/retarget, and missing interactive handle fails instead of silently auto-approving or producing SKIP/UNVERIFIED. |
| H007-37-028 | E-TEST/OBS | L53 | A real flag-on elai run must journal a plan decision under fixed seed and Reject must leave no subsequent TESTING_FIRST PhaseTransition; missing row fails wiring trigger. |
| H007-37-029 | E-LIMIT/OBS | L55-L60 | Industry plan-mode prior art is claimed rather than an ELAI metric; H-37 may report only journal-recomputed decision counts/mix, not unsupported percentages. |
| H007-37-030 | E-INT/LIMIT | L59 | Odysseus NO_SKILL semantics inform fail-closed behavior only; Rust reimplements semantics and uses blake3 rather than porting Python/hashlib. |
| H007-37-031 | E-INT/SEC | L62-L67 | H-37 composes B-22 evidence/autonomy, E-38 compliance at the same edge, B-6 installed handle, H-36 notifications, and inherited fail-closed/allowlist/measured/objectivity doctrine. |
| H007-37-032 | C-FLOW | L33 versus L49-L53 | REJECT is said to leave the FSM at PLANNING even when source may be FEASIBILITY_PROBE, while the wiring contract says it stays at the source state; transition semantics must preserve actual source and reconcile the wording. |
| H007-37-033 | C-CFG | L45 | ELAI_PLAN_APPROVAL_GATE_ENABLE is default-off and therefore an enable/experiment gate, yet the required allowlist type is called kill_switch; the polarity/type naming is internally inconsistent. |
| H007-37-034 | A-FLOW/CAP | L22, L34, L46 | Existing CliInteractiveGate reads stdin and rejects every non-approve input, while PlanApprovalCard offers redirect/reject buttons through the same path; the transport that correlates asynchronous dashboard input to present_gate is not specified. |
| H007-37-035 | A-SEC/STATE | L31, L43 | Autonomous MEDIUM/HIGH “surfaces to a human” but exact transition blocking/wait/retry lifecycle after RequestHuman is not defined. |

### H-39 — Abstention-aware promotion and evaluation scoring

Source: H-39_abstention_aware_promotion_eval.md (191 lines). Status: planned Track 6/Wave 6 keystone; in-process scorer/event plus separately gated live bench.

| Atomic ID | Class/domain | Exact source anchor(s) | Explicit feature or behavior |
|---|---|---|---|
| H007-39-001 | E-STATUS/STATE | L3, L17-L23 | H-39 is PLANNED Track 6 Wave 6, runs first as the keystone, and splits an in-process scorer/reward/event tier from a deterministic fixture plus separately gated live local/RunPod bench. |
| H007-39-002 | E-STATE/FAIL | L5-L15 | RefusalDecision and refusal SLA are shipped, but promotion, BAI, Darwin, GEPA-facing reward, and pass@k are abstention-blind and therefore incentivize confident guessing. |
| H007-39-003 | E-LIMIT/OBS | L29-L37 | Binary grades give abstention zero while any positive-probability guess has positive expected credit; external paper results motivate aligning optimized reward with refusal behavior rather than proving ELAI outcomes. |
| H007-39-004 | E-CAP/DATA | L12-L14, L62-L67 | abstention_aware_score consumes answered, correct, and RefusalDecision and returns +1 for answered-correct, 0 for abstention, and negative t/(1-t) for answered-wrong. |
| H007-39-005 | E-DATA/OBS | L65-L67, L77-L79 | Aggregate output is mean_score, abstention_rate, confident_wrong_rate, and n; the measured row additionally carries locked t and seed. |
| H007-39-006 | E-CFG/SEC | L65-L67, L84-L86 | Confidence target t is a frozen/LOCKED constant, stated default 0.75, so a self-mod loop cannot tune the penalty toward zero guessing cost. |
| H007-39-007 | C-CFG/DET | L34, L64-L67, L181-L182 | The formula t/(1-t) evaluates to 3 at t=0.75, while the plan repeatedly labels 0.75 as a 2x penalty; formula, prose, tests, and decision record cannot all be true without resolving the convention. |
| H007-39-008 | E-INT/SEC | L13, L40-L42, L62-L67 | The abstain bit is read from shipped conformal_calibration::RefusalDecision and refusal rate composes with D-17; H-39 must not infer a second abstention signal. |
| H007-39-009 | E-STATE/INT | L39-L49 | Refusal/SLA, promotion/archive/BAI/Darwin, coding_metrics, and SAFEDELTA/SecCodePlt measured-runner surfaces are shipped; AbstentionAwareScore is absent. |
| H007-39-010 | E-CAP/DATA | L69-L70 | Add a coding_metrics sibling accumulator that records the outcome triple and exposes abstention_aware_score alongside unchanged pass_at_k. |
| H007-39-011 | E-FLOW/INT | L72-L75 | Extend the score consumed by two_gate_promote/ArchiveEntry/BAI/Darwin so an abstaining candidate ranks above a confident-wrong candidate; do not rewrite the existing tau gate. |
| H007-39-012 | A-CFG/INT | L72-L75 | Promotion may use the abstention score as the reward or as a LOCKED-weighted component; exact weight/composition and aggregation into ArchiveEntry are left open. |
| H007-39-013 | E-DATA/STATE | L77-L79 | Add one Pattern-2 AbstentionScored at a re-read baseline+1 with mean_score, abstention_rate, confident_wrong_rate, t, n, and seed; the first Track-6 bump fixes stale e45. |
| H007-39-014 | E-CAP/OBS | L81-L82 | Bench scoring reports abstention-adjusted score beside pass@1 and appends it to phase TOML using the existing benchmark pattern. |
| H007-39-015 | E-DATA/OBS | L84-L86 | H-39-decisions.md records the confidence-target protocol, locked default, paper incentive result, RefusalDecision reuse, and the “aligns reward but does not cure hallucination” limit. |
| H007-39-016 | E-INT | L88-L98 | Produced scorer/aggregate/event consume RefusalDecision, refusal_rate, two_gate_promote, record_pass_sample, and the existing frozen-runner/measured-row pattern. |
| H007-39-017 | E-FLOW/INT | L99-L101 | Live elai bench invokes the scorer and event; Darwin generation through two_gate_promote consumes the same abstention-aware reward. |
| H007-39-018 | E-FAIL/SEC | L102-L107 | Any optimized reward equating confident-wrong with abstain fails; re-derived abstention, tunable t, non-flipping broken arm, non-blake3 hashing, and production unwrap/expect are forbidden. |
| H007-39-019 | E-CAP/OBS | L108-L110 | Cost/trust cockpit shows abstention rate and adjusted score; dashboard proves a promotion where abstaining candidate beats confident-wrong candidate. |
| H007-39-020 | E-TEST/OBS | L111-L117 | A frozen no-override benchmark with wrong V1 and abstaining V2 must journal the aggregate and rank V2 above V1; --reward-confident-guessing must flip the outcome to prove the metric is load-bearing. |
| H007-39-021 | E-DET/TEST | L119-L126 | Deterministic checks run wiring trigger, find scorer/event live, verify RefusalDecision reuse, reject blake2/sha2, and require decision record. |
| H007-39-022 | E-CFG/TEST | L128-L137 | Score is the default optimized reward with no enable override and only a kill switch; production bench/promotion reachability, strict wrong-versus-abstain ordering, reuse, reproducibility, broken arm, tier honesty, build, and auditor pass are gates. |
| H007-39-023 | E-DET/OBS | L114-L115, L134 | Aggregate recomputes byte-identically from the journal under ELAI_SEED. |
| H007-39-024 | E-TEST | L152-L157 | Contract tests cover all three item scores, ranking and promotion, broken-arm inversion, runtime immutability of t, same-seed equality, and frozen RefusalDecision fixtures. |
| H007-39-025 | E-DATA/INT | L159-L166 | Implementation creates scorer/contracts/fixtures/decision record and modifies metrics accumulator, promotion reward, SSB locks, benchmark output, and phase TOML. |
| H007-39-026 | E-SEC/DEP | L168-L174 | Paper ID/formula and threshold must be web/paper verified before locking; code uses blake3/no production unwrap and retains the honest no-cure ceiling. |
| H007-39-027 | E-DATA/LIMIT | L176-L180 | Existing harness pass/fail remains the correctness bit; H-39 adds abstention dimension and does not redefine correctness. |
| H007-39-028 | E-FAIL/LIMIT | L178-L180 | For tasks without an abstain channel, abstention rate is zero and score reduces to a penalty-weighted pass rate rather than fabricating abstentions. |
| H007-39-029 | A-CFG/TEST | L181-L182 | The stated 0.75 default is still VERIFY-before-locking; higher 0.9 may over-suppress and must be checked against D-17's overcautious band. |
| H007-39-030 | E-LIMIT/FAIL | L183-L184 | H-39 is necessary but insufficient: it stops rewarding confident hallucination; inference-time refusal remains D-16/D-17 and no cure claim is allowed. |
| H007-39-031 | E-DEP/INT | L186-L189 | H-40/H-41/H-42 follow as verifier/budget refinements; H-39 is upstream because its reward is what D-5/E-7/E-8 self-improvement climbs. |
| H007-39-032 | A-CFG | L130, L141 | A kill switch is required but no identifier, polarity, allowlist status, or fail behavior is specified. |

### H-42 — Serial-depth-scaled reasoning budget and depth-not-width guard

Source: H-42_serial_depth_reasoning_budget.md (184 lines). Status: planned Track 6/Wave 6 in-process extension.

| Atomic ID | Class/domain | Exact source anchor(s) | Explicit feature or behavior |
|---|---|---|---|
| H007-42-001 | E-STATUS/STATE | L3, L17-L23 | H-42 is a PLANNED Track 6 Wave 6 in-process extension of shipped per-role GRC and E-19 reflection, with no hard prerequisite and file-disjoint parallelism. |
| H007-42-002 | E-STATE/FAIL | L5-L15 | Current fixed per-role call/character budgets ignore serial task depth and can truncate deep work before the reasoning steps it requires. |
| H007-42-003 | E-LIMIT/OBS | L6-L15, L27-L35 | Formal results motivate serial steps as compute-depth and context as information access; they ground policy but do not make true task depth directly computable. |
| H007-42-004 | E-CAP/DET | L11-L15, L59-L62 | estimate_serial_depth deterministically reads current decomposition-tree depth or longest DKG must-traverse reachability chain and returns bounded integer d without wall clock. |
| H007-42-005 | E-CAP/CFG | L12-L15, L64-L67 | scale_grc_budget raises per-role call, character, and E-19 reflection budgets monotonically with d from the base floor, clamped to a LOCKED mutable=0 ceiling. |
| H007-42-006 | E-CAP/FAIL | L13-L15, L69-L72 | On budget exhaustion or low self-certainty, classify high-depth exhaustion as DepthBound and low-depth retrieval miss as AccessBound. |
| H007-42-007 | E-LIMIT/FLOW | L14-L15, L69-L72, L97 | DepthBound remedy is more serial steps/decomposition, never a context-window lift; AccessBound remedy is more or better context. |
| H007-42-008 | E-LIMIT/STATE | L69-L72 | Failure classification is a labeled diagnostic, not a hard gate. |
| H007-42-009 | E-STATE/INT | L37-L45 | AgentGrcConfig/per-role configs and E-19 certainty/reflection are shipped; tree/DKG depth signals exist; scaling and classifier are absent. |
| H007-42-010 | E-INT/LIMIT | L38-L44 | Extend AgentGrcConfig and E-19 rather than replace them; choose the cheapest available shipped depth source for the current subtask. |
| H007-42-011 | E-DATA/DET | L59-L62 | Tree signal uses concept/phase/step/babystep nesting; code signal uses longest must-traverse call/dependency chain; computation is seed-stable. |
| H007-42-012 | E-CFG/SEC | L64-L67 | Ceiling cannot be changed by a self-mod loop, nothing scales below base, and no task gets unbounded budget. |
| H007-42-013 | E-DATA/OBS | L74-L76 | Add Pattern-2 ReasoningBudgetScaled with role, base_calls, serial_depth, scaled_calls, ceiling, and failure_class; recompute under ELAI_SEED and re-read schema baseline+1. |
| H007-42-014 | E-DATA/OBS | L78-L81 | H-42-decisions.md records CoT lower-bound grounding, context-versus-depth distinction, locked cost ceiling, and route-to-tool/refusal ceiling for out-of-class tasks. |
| H007-42-015 | E-INT | L83-L92 | Produced estimator/scaler/classifier/event consume AgentGrcConfig, self_certainty::compute_score, and one shipped depth signal. |
| H007-42-016 | E-FLOW/INT | L93-L95 | Live per-baby-step dispatch resolves base config, scales it, and emits the row; GRC exhaustion invokes failure classification. |
| H007-42-017 | E-FAIL/LIMIT | L96-L101 | Context remedy for DepthBound, unbounded/unlocked budgets, duplicate GRC/reflection, non-failing flat arm, non-blake3 hashing, and production unwrap/expect are forbidden. |
| H007-42-018 | E-CAP/OBS | L102-L104 | Reasoning drawer shows subtask depth, scaled budget, and failure class; dashboard demonstrates a deep task with larger budget and DepthBound. |
| H007-42-019 | E-TEST/OBS | L105-L111 | Same-role shallow and deep real subtasks with no override must emit two rows, deep scaled_calls greater than shallow, both under ceiling; forced high-depth exhaustion reports DepthBound and flat arm equalizes budgets. |
| H007-42-020 | E-DET/TEST | L113-L120 | Deterministic checks run wiring trigger, find estimator/scaler/event, verify GRC/certainty reuse, reject blake2/sha2, and require decision record. |
| H007-42-021 | E-CFG/TEST | L122-L131 | Scaling is default-on for every subtask with base floor and only a kill switch; live dispatch, locked ceiling, reuse, reproducibility, broken arm, honest ceiling, build, and auditor pass are gates. |
| H007-42-022 | E-TEST | L146-L151 | Contracts cover estimator fixture, monotone/clamped scaling, unchanged depth 0/1 floor, DepthBound/AccessBound cases, flat arm, immutable ceiling, same-seed equality, and shallow/deep fixtures. |
| H007-42-023 | E-DATA/INT | L153-L159 | Implementation creates reasoning_budget.rs/contracts/fixtures/decision record and modifies dispatch, SSB locks/e45, and reasoning-drawer columns. |
| H007-42-024 | E-SEC/DEP | L161-L167 | Referenced formal-paper IDs must be web-confirmed before headline use; code retains blake3/no unwrap and no “more steps always wins” claim. |
| H007-42-025 | E-FAIL/OBS | L169-L172 | Depth is an imperfect proxy; factor and ceiling require conservative calibration on budget-exhaustion rates over at least three real runs before increases. |
| H007-42-026 | E-CFG/LIMIT | L173-L174 | Locked ceiling is the total-cost guardrail and scaled budget remains subject to HD-2/HD-3 cache/output controls. |
| H007-42-027 | C-DATA/FAIL | L69-L72, L74-L76 versus L175-L176 | Task/classifier and event declare a singular DepthBound versus AccessBound failure_class, but risks require a mixed deep-and-retrieval-starved case to surface BOTH; representation and remedy combination are unresolved. |
| H007-42-028 | A-CFG/DET | L59-L67, L169-L172 | Exact estimator precedence, depth bound, monotone scaling formula, factor, and ceiling value are not specified and must be calibrated/locked. |
| H007-42-029 | A-CFG | L124, L135 | A kill switch is required but its identifier, polarity, and allowlist/promotion lifecycle are not stated. |
| H007-42-030 | E-DEP/INT | L178-L182 | Track 6 combines H-39 reward honesty, H-40 verifier decorrelation, H-41 checker coverage, and H-42 serial compute; H-42 changes reasoning compute, not checker truth. |

### H-50 — Regime-aware in-session context management

Source: H-50_regime_aware_context_management.md (85 lines). Status: Track 9/Wave 9 build-on-shipped; local unload/load and cloud cache-stable split.

| Atomic ID | Class/domain | Exact source anchor(s) | Explicit feature or behavior |
|---|---|---|---|
| H007-50-001 | E-STATUS/DEP | L2 | H-50 is Track 9 Wave 9 BUILD-on-shipped, effort M-L, composing cloud cache H-1/H-2, H-6 masking, and H-35/H-49 SessionDag/selective recovery. |
| H007-50-002 | E-CFG/LIMIT | L3, L5-L7 | Backend regime selects opposite policies: local minimizes repeatedly sent context, while cloud preserves a large byte-stable prefix to amortize cache writes over cheap reads. |
| H007-50-003 | E-STATE/FAIL | L3, L10-L14 | Long local interactive sessions currently have no effective pre-overflow shrink and terminate at context_window_blocked instead of graceful unloading. |
| H007-50-004 | E-INT/LIMIT | L7 | Reuse adaptive_reduce, compaction_ladder, recover_stubs, H-6 masking, and SessionDag; H-50 is primarily window-truth and live-interactive wiring, not another compactor. |
| H007-50-005 | E-STATE/CFG | L10 | conversation maybe_auto_compact uses a flat 100,000 cumulative-input-token threshold from one env setting, so it cannot fire in time for a 32K local model. |
| H007-50-006 | E-STATE/CFG | L11 | live_cli handoff uses max_tokens_for_model times 0.85; every non-Opus name maps to 64K, ignoring ELAI_LOCAL_MAX_CONTEXT and yielding about 54.4K for a 32K model. |
| H007-50-007 | E-STATE/LIMIT | L11 | BudgetPolicy::Local knows ELAI_LOCAL_MAX_CONTEXT but only sizes slots/caps output; it does not trim input. |
| H007-50-008 | E-STATE/FAIL | L12 | Current SSB-present handoff calls recover_stubs for every stubbed tool result before inference, reinflating trimmed content and leaving near-zero net reduction. |
| H007-50-009 | E-STATE/INT | L13 | adaptive_reduce has no production callers; compaction_ladder is default-on but only autonomous PreCompact over a summary, not interactive REPL. |
| H007-50-010 | E-STATE/CAP | L13-L14 | Existing local live shrinkage is only roughly 15–30% ShortCoder code-block compression; provider preflight ultimately rejects overflow as context_window_blocked. |
| H007-50-011 | E-STATE/INT | L15 | Anthropic cache markers are live at all three dispatch sites, a 1-hour TTL requires a byte-stable prefix, and PROMPT_CACHE_PREFIX_BOUNDARY=3 prevents trimming across it. |
| H007-50-012 | E-CORRECTION/STATE | L15 | The later code-read explicitly refutes H-1's marker-free split-brain premise; H-50 leaves the verified live cloud path unchanged. |
| H007-50-013 | E-DATA/CFG | L15 | Live schema snapshot is 224/501 and must be re-read at edit time. |
| H007-50-014 | E-CAP/CFG | L17-L20 | Add effective_context_window(backend): local uses ELAI_LOCAL_MAX_CONTEXT/BudgetPolicy ceiling, cloud uses cloud ceiling, and API uses the model's real window. |
| H007-50-015 | E-CFG/FLOW | L19 | Both conversation auto-compaction and live_cli handoff use trim_at=floor(effective_window times regime fraction), approximately 0.70–0.75 local and 0.85 cloud, replacing flat/incorrect constants. |
| H007-50-016 | E-FLOW/FAIL | L20 | Local interactive trim runs cheapest-first: drop dead trailing observations through H-6 masking, stub bulky tool results, then compact oldest turns. |
| H007-50-017 | E-FLOW/DATA | L20, L25, L34 | Local recovery stays unloaded and becomes selective only when a specific stub is requested; unconditional full-set re-inflation is a gate failure. |
| H007-50-018 | E-CFG/LIMIT | L20, L25 | Cloud preserves cache-stable prefix and boundary guard and compacts only near the real window; live Anthropic code is not modified. |
| H007-50-019 | E-OBS/TEST | L21 | Enabled local trimming must journal ContextReduction, reduce next-request input tokens below pre-trim and window, and let a previously blocked session complete. |
| H007-50-020 | E-DEFER/CAP | L22 | Tier 2 may connect streaming generation to the existing PrefixKVCache lease/admit path or route large local requests to non-streaming, behind its own kill switch. |
| H007-50-021 | E-TEST/DET | L22 | Tier-2 cache unbypass reduces compute rather than tokens and must preserve byte-identical generation with and without caching. |
| H007-50-022 | E-INT/FLOW | L24-L25 | New context policy/window truth are consumed by conversation and live_cli; local uses existing ladder, adaptive reducer, compact_session, masking, and selective recover_stubs. |
| H007-50-023 | E-CAP/OBS | L26 | Dashboard shows a per-regime context-budget gauge and “unloaded N items (reloadable)” indicator, with cross-language parity. |
| H007-50-024 | E-DATA/INT | L27, L31 | Reuse ContextReduction for unloads and optionally add Pattern-2 WindowPolicyApplied only if a distinct regime-decision row is necessary; prefer no schema bump. |
| H007-50-025 | E-CFG/SEC | L28 | Tier 1 is default-on with opt-out ELAI_CONTEXT_POLICY_DISABLE; Tier-2 cache unbypass has a separate kill switch. |
| H007-50-026 | E-INT | L30-L33 | Produced off-agent artifacts consume BudgetPolicy/window truth, compaction ladder, adaptive reducer, compact_session, selective recovery, and H-6, from every real interactive local turn plus maybe_auto_compact. |
| H007-50-027 | E-FAIL/LIMIT | L34 | Placeholder trim, net-zero trim after reinflation, unconditional local recovery, and policy-code unwrap are forbidden. |
| H007-50-028 | E-TEST/OBS | L35 | A seeded 32K local session past trim must emit a non-agent-authored reduction row and smaller next request; disabled control must hit context_window_blocked while enabled completes. |
| H007-50-029 | E-OBS/LIMIT | L35, L40 | A local “caching on” claim fails when journal cache_read=0; current local streaming cache is zero and must not be advertised before Tier 2. |
| H007-50-030 | E-TEST/FAIL | L35 | False-drop rate for context later needed by the model must be bounded over at least three runs before default-on. |
| H007-50-031 | E-LIMIT/OBS | L37-L40 | Borrowed context/caching economics are design evidence remeasured per configuration; caching reduces cost per token, not token count, so local window minimization remains primary. |
| H007-50-032 | E-INT/DEP | L42-L47 | H-50 preserves H-1/H-2 cloud behavior, is the interactive activation point for H-6, shares selective recovery with H-35/H-49/H-51, and adds CL-1 through CL-4 doctrine. |
| H007-50-033 | E-INT/LIMIT | L49-L55 | J-11 amendment uses Arbor ContextManager/ContextPruner only as source-cited existence proof; H-50 owns regime policy and does not restate/port its implementation. |
| H007-50-034 | E-FLOW/CFG | L56-L66 | Compose three-tier ordering: remove old observations beyond a configurable recent window, truncate long observations to head+tail with marker, then summarize oldest observations into one line; cheapest work precedes expensive compaction. |
| H007-50-035 | E-LIMIT/DET | L67-L68 | Arbor is Unix-only/single-threaded; H-50 remains regime-aware in the interactive turn and borrows ordering only. |
| H007-50-036 | E-DATA/DET | L70-L75 | Use real journal/tokenizer counts to prune toward target budget; chars/4 is only a fallback when tokenizer is unavailable, a condition ELAI is said not to have. |
| H007-50-037 | E-SEC/CFG | L77-L82 | Protected content is never pruned: cloud protects cache prefix, while local protects system prompt, rules, and current phase spec by composing existing allocator floors. |
| H007-50-038 | E-LIMIT/FLOW | L84-L85 | Do not port Arbor's persistent 500-turn ReAct loop; only its pruning mechanics apply to ELAI's interactive REPL turn. |
| H007-50-039 | A-CFG | L19 | Exact local TRIM_FRACTION is a range around 0.70–0.75 rather than a chosen deterministic constant. |
| H007-50-040 | A-DATA/INT | L27, L31 | WindowPolicyApplied is explicitly left to the implementer, so final event schema/count is not decided by the plan. |
| H007-50-041 | C-CFG/TEST | L28 versus L30, L35 | Tier 1 is declared default-on, but false-drop must be bounded on at least three runs before default-on; initial shipping/promotion order is internally inconsistent unless the measurement precedes release. |
| H007-50-042 | A-DET/FLOW | L16-L20, L34, L56-L66 | “Dead trailing” eligibility, recent-window N, per-observation character cap, summary algorithm, selective-reload request protocol, and deterministic tie/order rules are not specified. |
| H007-50-043 | A-CFG/INT | L14, L17-L20 | effective_context_window distinguishes local/cloud/API but the exact BackendMode mapping, cloud/model lookup, and behavior when ELAI_LOCAL_MAX_CONTEXT is absent/invalid are unstated. |

## Cross-file interactions and bounded inferences

These are deliberately separated from explicit source requirements.

| Atomic ID | Class/domain | Evidence atoms | Interaction or bounded inference |
|---|---|---|---|
| H007-I-001 | I-DATA/INT | H007-13-020-H007-13-021, H007-15-011, H007-15-025 | CanvasOpApplied owns store mutation, while CanvasViewEvent owns view lifecycle/UI/checkpoints; keeping those roles separate avoids the explicitly banned second operation transport. |
| H007-I-002 | I-SEC/DEP | H007-15-006, H007-15-009, H007-15-017 | A taint flag at Select is only as strong as live taint provenance; shipping with replay-only taint_text must be labeled a reduced-security tier rather than equivalent to G-1. |
| H007-I-003 | I-FLOW/DEP | H007-13-042, H007-15-039, H007-16-002 | Deletion order is strict: H-13 transport precedes H-15 reply channel; H-12 full-release canvas proof precedes H-16, so orbit cannot be removed merely because H-15 code exists. |
| H007-I-004 | I-DET/INT | H007-18-027, H007-26-021, H007-26-027 | Finance numeric gates require deterministic verdicts but tolerate bounded same-environment floating-point variation; both their thresholds/bounds are prime H-27 verifier-mutation targets. |
| H007-I-005 | I-DATA/INT | H007-13-021, H007-15-011, H007-18-016, H007-26-013, H007-34-023, H007-37-024, H007-39-013, H007-42-013, H007-50-013 | Every literal schema baseline is a dated local snapshot; implementation order is cumulative, so no source's numeric baseline is globally authoritative and each change must re-read lockstep state. |
| H007-I-006 | I-CFG/INT | H007-34-002, H007-50-002-H007-50-003, H007-50-014-H007-50-019 | H-34 reduces external-tool schema injection and H-50 unloads session history; on local models their token measurements share one effective-window budget and should be tested together to avoid each assuming the other's slice is fixed. |
| H007-I-007 | I-SEC/FLOW | H007-15-018, H007-37-008, H007-37-027 | UI-mediated autonomy has two independent human controls: canvas ToolCall needs explicit consent and plan transition needs fail-closed approval; neither event substitutes for the other. |
| H007-I-008 | I-FLOW/LIMIT | H007-39-011, H007-42-005-H007-42-008 | Giving deep work more steps changes opportunity/cost, while H-39 changes evaluation incentives; promotion evaluation should compare candidates under recorded scaled budgets so added compute is not mistaken for better reward alignment. |
| H007-I-009 | I-LIMIT/INT | H007-42-007, H007-50-002, H007-50-017 | H-42 deliberately routes AccessBound to context and DepthBound to steps; H-50 supplies selective context reload, making it the natural recovery consumer for AccessBound but an explicitly wrong remedy for pure DepthBound. |
| H007-I-010 | I-FAIL/OBS | H007-34-027, H007-39-020, H007-42-019, H007-50-028 | These plans all require differential evidence, not mere presence: positive token reduction, ranking inversion under broken reward, deep-versus-shallow budget separation, and enabled-versus-disabled overflow outcome. |
| H007-I-011 | I-SEC/OBS | H007-13-010, H007-15-027, H007-18-017, H007-37-028, H007-50-028 | Off-agent proof appears in different forms—auditor view, frozen numeric input, runtime transition, and journal token count—but consistently excludes implementation-authored stdout as completion evidence. |
| H007-I-012 | I-CFG/SEC | H007-34-012, H007-37-025, H007-50-025 | Default-off experiments need allowlist/promotion records, while default-on policies need bounded-failure evidence and kill switches; calling an enable gate a kill switch obscures the required governance path. |

## Summary-index additions and corrections

Orientation anchors are the H-13, H-15, H-16, H-18, H-26, H-34, H-37, H-39, H-42, and H-50 sections of 08-H-SUMMARY_INDEX.discovery.md. Deltas point to deep atoms and create no new feature atoms.

### Material additions absent or materially compressed in the summary

| Delta ID | Source | Deep atoms | Addition to summary-level discovery |
|---|---|---|---|
| DELTA-007-001 | H-13 | H007-13-008-H007-13-015 | Add operation-only constrained decoding, exact sanitize repairs, auditor self-proof threat, mock/live VLM tier, CommittedToolSet composition, and wire-dark provenance warning. |
| DELTA-007-002 | H-13 | H007-13-017, H007-13-020-H007-13-024 | Add exact seven tools, ten CanvasOpKind values, PlantUML/getBBox import behavior, raw-Mermaid ban, and native capture-to-feedback lifecycle. |
| DELTA-007-003 | H-13 | H007-13-040-H007-13-043 | Add unproven VLM correctness, gate-freeze requirement, downstream consumers, and correction that this detail owns one baseline+1 operation event rather than CanvasActivityEvent. |
| DELTA-007-004 | H-15 | H007-15-005-H007-15-006, H007-15-015-H007-15-021 | Add partial/absent trust-spine state, all five CanvasUiAction types, ToolCall consent, autonomy confidence/text default/suppress hatch, and spawn-rate measurement. |
| DELTA-007-005 | H-15 | H007-15-025-H007-15-026, H007-15-035-H007-15-040 | Add transport separation, paper-evidence limit, weaker-taint choice, autonomy choice, publish/share deferral behind G-8, and underdefined checkpoint/action schema. |
| DELTA-007-006 | H-16 | H007-16-007-H007-16-013 | Add exact live orbit chain, mandatory lcg preservation, partial-file reachability trace, default-panel flip, decision record, and unresolved galaxy-toggle choice. |
| DELTA-007-007 | H-16 | H007-16-016, H007-16-020-H007-16-024 | Add explicit no-SSB deletion evidence, build-level rather than unit gate, irreversible-operation posture, downstream closure, and wildcard deletion ambiguity. |
| DELTA-007-008 | H-18 | H007-18-011-H007-18-019 | Add evaluator inputs/full outputs, seed/clock requirements, gate-manifest freeze, full event payload, off-agent input semantics, and live-run deferral. |
| DELTA-007-009 | H-18 | H007-18-027, H007-18-030-H007-18-034 | Add same-environment FP scope/stable verdict band, formula-ingest requirement, unresolved PBO/MinBTL constants, downstream composition, and triple-versus-six-field payload conflict. |
| DELTA-007-010 | H-26 | H007-26-013-H007-26-021 | Add exact event kinds, real fixture/live split, missing-second-run fail rule, same-env recomputation, and full production hook/evidence flow. |
| DELTA-007-011 | H-26 | H007-26-027-H007-26-032 | Add three-refactor FP calibration, same-stack hard gate versus advisory QuantLib, trivial-invariant rejection, H-27 dependency, and unspecified bound/second-implementation selection. |
| DELTA-007-012 | H-34 | H007-34-004-H007-34-011 | Add cache-none/no-emit behavior, RuntimeConfig/non-hardcoded servers, exact wire-dark gap, tokenizer/corpus/seed baseline mechanics, and no-new-embedding constraint. |
| DELTA-007-013 | H-34 | H007-34-014-H007-34-026 | Add deferred-event fields, full report/CLI shape, no Tier-1 frontend, exact +2 lockstep accounting, promotion condition, and surfaced LazyCacheError/no-eager-fallback rules. |
| DELTA-007-014 | H-34 | H007-34-028-H007-34-035 | Add borrowed-claim limits, custom-base-URL necessity, G-4 ordering, inherited gate doctrine, and unresolved selected-schema/cache-hit event lifecycle. |
| DELTA-007-015 | H-37 | H007-37-003-H007-37-015 | Add exact compliance assertions/runtime-dark install, full gate decisions/outcomes, CLI input semantics, GateMode defaults, observability-policy separation, and pre-emption order. |
| DELTA-007-016 | H-37 | H007-37-019-H007-37-035 | Add MODIFY-to-redirect behavior, immutable/newest-wins lifecycle, risk override, missing-handle fail, decision-mix metric limit, source-state/flag-type conflicts, and dashboard/autonomous wait ambiguities. |
| DELTA-007-017 | H-39 | H007-39-005-H007-39-007, H007-39-010-H007-39-014 | Add aggregate fields, mathematically explicit 0.75 inconsistency, unchanged pass@k, open promotion weighting, full event fields, and phase-TOML benchmark output. |
| DELTA-007-018 | H-39 | H007-39-027-H007-39-032 | Add correctness ownership, no-abstain reduction behavior, default verification, no-cure boundary, downstream track role, and unspecified kill switch. |
| DELTA-007-019 | H-42 | H007-42-004-H007-42-013 | Add exact tree/DKG signal meanings, calls+chars+reflection scaling, diagnostic-not-gate distinction, seed behavior, and full row fields. |
| DELTA-007-020 | H-42 | H007-42-025-H007-42-029 | Add three-run calibration, cost guardrails, mixed-failure representation conflict, unspecified estimator/scaling constants, and unspecified kill switch. |
| DELTA-007-021 | H-50 | H007-50-005-H007-50-013 | Add exact flat/wrong-window/reinflation/dead-reducer bugs, ShortCoder-only current relief, hard-error backstop, live cloud cache details, and later schema snapshot. |
| DELTA-007-022 | H-50 | H007-50-020-H007-50-031 | Add optional streaming PrefixKVCache activation/non-streaming route, byte-identity requirement, optional event decision, explicit stub bans, disabled control, cache_read honesty, and borrowed-economics boundary. |
| DELTA-007-023 | H-50 | H007-50-033-H007-50-043 | Add Arbor amendment's ordered drop/truncate/summarize mechanics, real-token rule, protected local/cloud content, do-not-port loop, default-on conflict, and unresolved thresholds/reload/mode mapping. |

### Corrections or qualifications to summary wording

| Delta ID | Source | Deep atoms | Correction or qualification |
|---|---|---|---|
| DELTA-007-024 | H-13 | H007-13-043 | Correct event ownership: this detail adds CanvasOpApplied baseline+1; CanvasActivityEvent is not one of H-13's declared produced action types. |
| DELTA-007-025 | H-15 | H007-15-005-H007-15-006 | “G-1 taint and G-2 RRA dependency” must retain their absent/partial status; a taint_text-only H-15 is a weaker interim gate, not full trust-spine completion. |
| DELTA-007-026 | H-16 | H007-16-002, H007-16-022 | “After full release” is a hard no-run precondition and tripwire, not merely schedule guidance. |
| DELTA-007-027 | H-18 | H007-18-027, H007-18-032 | Determinism is same-environment verdict stability, not cross-hardware byte identity; MinBTL/PBO/alpha constants still require verification. |
| DELTA-007-028 | H-26 | H007-26-028 | QuantLib must remain advisory because convention differences can diverge; only new-versus-prior same-stack comparison is a hard gate. |
| DELTA-007-029 | H-34 | H007-34-027-H007-34-028 | Positive ELAI journal delta is the done evidence; Anthropic and third-party reduction numbers are ceilings, never expected ELAI values. |
| DELTA-007-030 | H-37 | H007-37-003, H007-37-011 | Compliance machinery is not a live installed gate in the observed state; only an actually installed provider may pre-empt plan approval. |
| DELTA-007-031 | H-39 | H007-39-007 | Replace the summary's “arithmetically unclear” qualification with an explicit conflict: at t=0.75, t/(1-t)=3, not 2. |
| DELTA-007-032 | H-42 | H007-42-003, H007-42-025 | Serial-depth estimates are proxies requiring measured calibration, not proofs of a task's true CoT lower bound. |
| DELTA-007-033 | H-50 | H007-50-011-H007-50-012, H007-50-029 | Cloud caching is live in the later snapshot; local streaming cache is not, so cache claims must be qualified per regime. |

## Ambiguity and conflict register

All rows below are indexed above and are not silently promoted to requirements.

- H-13: CanvasActivityEvent/baseline ownership conflict — H007-13-043.
- H-15: wait for G-1/G-2 versus weaker taint_text slice; model autonomy choice; checkpoint/action schema — H007-15-006, H007-15-036, H007-15-040.
- H-16: galaxy toggle, wildcard deletion scope — H007-16-013, H007-16-024.
- H-18: unfixed MinBTL/PBO/alpha and “triple” payload shorthand — H007-18-032, H007-18-034.
- H-26: numerical bound and second-implementation selection — H007-26-031, H007-26-032.
- H-34: selected schema injection, selector constants, cache-hit event lifecycle — H007-34-033 through H007-34-035.
- H-37: reject source state, enable-versus-kill-switch type, dashboard input transport, autonomous wait lifecycle — H007-37-032 through H007-37-035.
- H-39: 0.75 penalty arithmetic, reward composition, default verification, unspecified kill switch — H007-39-007, H007-39-012, H007-39-029, H007-39-032.
- H-42: mixed failure representation, estimator/scaler constants, unspecified kill switch — H007-42-027 through H007-42-029.
- H-50: trim fraction, optional event, default-on ordering, prune/reload mechanics, backend mapping — H007-50-039 through H007-50-043.

## Zero-omission and zero-duplicate audit

- Source coverage: 10/10 manifest files; 1,494/1,494 logical lines; 158,951/158,951 bytes; all ten SHA-256 values exact.
- Structural coverage per source: status/objective/premise, context/ground truth, tasks/revised scope, integrations/wiring/reachability, flags/defaults, schema/events/lifecycle, security/permissions, stub bans/failures/recovery, observability, deterministic checks, acceptance/tests, file inventory, grounding/claim limits, risks/open choices, composition/dependencies/next, and the H-50 J-11 amendment were inspected and represented.
- Behavioral coverage: every distinct explicit capability, action, surface, state, workflow, schema/data/lifecycle rule, config/default, integration/event, security/privacy control, failure/fallback/recovery rule, determinism/concurrency constraint, observable artifact, acceptance/done gate, non-goal, dependency, and deferral was assigned one source-scoped atomic ID.
- Duplicate policy: repeated clauses are folded into a single atom with combined physical-line anchors. Summary deltas and interactions point to source atoms instead of restating them as source features.
- Classification audit: explicit statements are separated from bounded inference (H007-I-*), ambiguity (A-*), and conflict/correction (C-*). No inferred interaction is counted as source-mandated behavior.
- Automated ID uniqueness, class counts, output line count, and canonical digest are recorded below after generation.

## Final counts

- Files processed: **10**.
- Source lines processed: **1,494**.
- Source bytes processed: **158,951**.
- Atomic feature/behavior rows: **360** = 319 explicit + 22 ambiguous + 7 conflicting + 12 inferred; automated uniqueness check: **360/360 unique, 0 duplicate IDs**.
- Output logical lines: **547**.
- Output SHA-256 (canonical content excluding the final digest line): **8bd91ffc8d858fa1440faa1904e4fb8dcc2c0875e999f18aa90cbe67eaaf491c**.
