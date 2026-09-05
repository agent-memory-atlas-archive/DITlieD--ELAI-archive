H-13 DECISIONS — model drives the canvas: ops + dispatcher + SSB + ARCHITECTURE mode
Session: s-10612 | Date: 2026-06-04 | SCHEMA 221 / TV 498

D1: CanvasOpApplied as Pattern-2 single-variant (not 10 separate SsbActionType variants)
What: ONE SsbActionType::CanvasOpApplied with a 10-arm CanvasOpKind discriminator.
Decision: Pattern-2 single-variant, matching H-11's CanvasActivityEvent precedent.
Evidence: H-11 used Pattern-2 (4-arm Kind); E-22/E-10/E-43 all use Pattern-2 for grouped events. The spec §11 lists 10 separate strings but H-11 established the convention of single-variant + Kind. TOTAL_VARIANTS bump is 1, not 10.
Expected: SCHEMA 221, TOTAL_VARIANTS 498, e45 contract pin 498.

D2: CanvasDispatcher as standalone module (not extending ToolDispatcher trait)
What: canvas_dispatcher.rs exports InProcessCanvasDispatcher implementing CanvasOpsDispatcher. The ToolDispatcher trait in core/tool_dispatcher.rs is NOT modified.
Decision: standalone. Adding methods to ToolDispatcher would require updating all implementations (NoOpDispatcher, InProcessToolDispatcher, and test implementations) — blast radius too wide for H-13 scope.
Mitigation: translate_tool_request canvas_* arms instantiate InProcessCanvasDispatcher directly. W-3 production entrypoint is production_agent_rpc.rs:translate_tool_request.

D3: canvas_screenshot as Tier-2 stub (non-silent Err, not silent Ok)
What: capture_canvas_screenshot Tauri command returns Err with a clear reason.
Decision: non-silent stub (stub-6). Tauri 2.11 does not expose a stable window capture API. Client-side canvas.toDataURL() is the correct Tier-1 path. Registered in stubs.md will_resolve_in=H-15.
Evidence: grep of Tauri 2.11 codebase found no capture_image API; Tauri API docs do not list a synchronous PNG capture method.
Anti-stale: wiring-enforcement stub_ban requires explicit Err with reason, never Ok([]) (silent stub).

D4: meta field as Option<String> in ipc-types (not serde_json::Value)
What: CanvasCreateNodeRequest and CanvasUpdateNodeRequest use Option<String> for meta.
Decision: ipc-types does not depend on serde_json (not in Cargo.toml). Using String (JSON-encoded) maintains the no-extra-dep constraint. Callers serialize meta before passing.
Tradeoff: slight inconvenience at call sites; avoids adding serde_json to ipc-types Cargo.toml.

D5: Grammar mode ceiling at 500ms (conservative, pending paper-ingest)
What: GRAMMAR_COMPILE_TIMEOUT_CEILING_MS = 500ms.
Decision: conservative value. arXiv:2501.10868 cites XGrammar compile efficiency as a dimension needing improvement but doesn't give an exact cliff. stub-5 registered for paper-ingestor to pin the exact number.
Evidence: the arch grammar has 9 properties (well under MAX_GRAMMAR_PROPERTIES=20).

D6: mermaid import via server-side export_graph_mermaid (no NPM dep)
What: ArchMode.svelte calls existing export_graph_mermaid Tauri command for mermaid text export.
Decision: reuse (GD-1). The @tldraw/mermaid dep is source-available (NOT MIT); the mermaid NPM package is large (~2MB minified). Server-side text generation is sufficient for Tier-1 arch mode.
Expected: no new NPM dep in package.json.

D7: canvasDispatcher.svelte.ts store pattern (not graphology direct mutation in CanvasPanel)
What: CanvasPanel.svelte wraps canvasDispatcher.svelte.ts store; CanvasWindow.svelte reads pendingOps.
Decision: the graphology instance is local to CanvasWindow.svelte's mountSigma() closure; it is NOT accessible from external stores. The pending-ops pattern mirrors canvasActivity.svelte.ts (GD-1 reuse).
Expected: CanvasWindow will drain canvasDispatcher.pendingOps in a future H-15 wiring pass once the sigma graph mutation API is finalized.

D8: tool catalog count corrected to 23 (not 14 as stale contract asserted)
What: production_agent_rpc_tool_catalog_contract.rs updated from 14 -> 23.
Decision: the contract was stale (last updated at C-30 with 14 tools; E-40 added 2 DSL tools making it 16; H-13 adds 7 canvas tools making it 23). All 23 tool IDs verified against TOOL_* constants.

UNKNOWNS / OPEN
- CanvasWindow.svelte does not yet drain canvasDispatcher.pendingOps or apply mutations to sigma — that is H-15 territory (the sigma graph mutation API needs to be exported from CanvasWindow).
- VLM scoring for canvas_screenshot is Tier-2 RUNPOD-PENDING (mock_vlm default, task_runner.rs:6246).
- Grammar CI ceiling tightening pending paper-ingestor run for arXiv:2501.10868 (stub-5).
