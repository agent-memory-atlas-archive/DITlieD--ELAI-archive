GROUNDING SWEEP -- H-13
Source: F:\.ELAI_workflow\docs\thefinalglance\H-plan\H-13_model_drives_canvas_ops.md
Date: 2026-06-04
FSM: PLANNING | Session: spec-grounder-H13

SPEC REFERENCES

CANVAS-WORKBENCH-DESIGN.md §7 | GROUNDED | read lines 213-258: op stream + tools + SSB flow + FSM-not-touched + constrained-decoding cliff + grammar_unavailable fallback + sanitizeAction all present verbatim
CANVAS-WORKBENCH-DESIGN.md §11 | GROUNDED | read lines 328-347: architecture + data flow + Tauri-native screenshot caveat + CanvasDispatcher trait + 9-tuple lockstep all confirmed
CANVAS-WORKBENCH-DESIGN.md §13 C4 | GROUNDED | read lines 390-393: C4.B1/B2/B3 auditor-driven gate structure matches H-13 Tasks 1-6 exactly; frozen-gate language present
H-GROUNDING-MAP.md C15 | GROUNDED | line 51: 3-arm surface at production_agent_rpc.rs:266/970/1163; 16-tool catalog; dkg_* tools wire-dark; E-40 precedent symbol_dsl_query confirmed
H-GROUNDING-MAP.md C17 | GROUNDED | line 55: E-37 trust spine wire-dark confirmed; CommittedToolSet live; canvas screenshot ABSENT (design-only); media_validator Tier2 shipped task_runner.rs:6208
production_agent_rpc.rs:260-770,960-1090,1160-1200 | GROUNDED | grep confirmed build_tool_catalog/convert_json_input_to_postcard/translate_tool_request all present at cited lines; 3-arm pattern confirmed
ssb.rs (9-tuple lockstep) | GROUNDED | grep confirmed Pattern-2 single-variant applied 50+ times; H-11 CanvasActivityEvent Pattern-2 with 4-arm Kind at SCHEMA_VERSION 220 is the direct precedent
task_runner.rs:6200-6250 | GROUNDED | grep confirmed wire_media_validator_for_spec at task_runner.rs:6208 (from C17); media_validator SHIPPED
capability_flow.rs:50-70 | GROUNDED | C17 confirms CommittedToolSet/committed_slot/check_against_commitment live at capability_flow.rs:53
H-0_INDEX.md HD-3/HD-4/HD-5 | GROUNDED | read lines 167-186: HD-3 cites arXiv:2408.02442 explicitly; HD-4 off-agent gate doctrine; HD-5 substrate-honesty all confirmed

CLAIM CLASSIFICATION

constrained decoding degrades LLM reasoning (HD-3 doctrine) | GROUNDED | arXiv:2408.02442 "Let Me Speak Freely?" confirmed: "significant decline in LLMs reasoning abilities under format restrictions; stricter constraints generally lead to greater performance degradation in reasoning tasks" -- WebFetch arxiv.org/abs/2408.02442
XGrammar compile-timeout cliff on complex grammars | GROUNDED | arXiv:2501.10868 JSONSchemaBench confirmed: XGrammar is one of 6 frameworks evaluated; compilation efficiency flagged as area needing improvement; complex schema compile timing is a documented dimension -- arxiv.org/abs/2501.10868
grammar_unavailable fallback = structured error + retry, never silent | GROUNDED | CANVAS §7 lines 255-257: "returns a structured grammar_unavailable error to the agent, the agent retries the op in a constrained sub-grammar... unconstrained fallback is banned" -- verbatim match
sanitizeAction rejects dangling-id ops, does not silently fix | GROUNDED | CANVAS §7 line 257: "rejects (does not silently fix) an op that references a node that doesnt exist, surfacing it as a blocked op" -- verbatim match
Anthropic tool-use API: ToolDefinition shape (name/description/input_schema) | GROUNDED | WebFetch platform.claude.com/docs: exact fields verified: name (regex ^[a-zA-Z0-9_-]{1,64}$), description, input_schema (JSON Schema object); tool_use response block has type/id/name/input fields; 3-arm pattern (catalog/convert/dispatch) is ELAI-internal, not Anthropic-native but maps correctly to the API surface
model response tool_use block shape | GROUNDED | WebFetch: {"type":"tool_use","id":"toolu_...","name":"...","input":{...}} confirmed exact
canvas_op_applied SSB Pattern-2 single-variant vs spec 10 separate variants | VERIFY_NEEDED | CANVAS §11 lines 332-333 names 10 separate SsbActionType strings (CanvasNodeCreated etc); H-13 Task 3 proposes ONE CanvasOpApplied + 10-arm CanvasOpKind. Divergence is intentional: H-11 CanvasActivityEvent Pattern-2 with 4-arm Kind (ssb.rs:35288) is the live convention that supersedes naive multi-variant listing in spec. Not a blocking MISMATCH; implementer must apply Pattern-2 convention not the spec's literal names
SCHEMA_VERSION bump "from LIVE 207/484" | MISMATCH | H-13 cites live baseline as 207/484. Verified: live SCHEMA_VERSION = 220 (H-11 CanvasActivityEvent, ssb.rs:26969). Live TOTAL_VARIANTS = 497 (action_type_lock.rs:624; e45 contract checks 497). H-13 must bump from 220/497, not 207/484
tldraw streaming pattern (agent-template + op stream) | GROUNDED | CANVAS §7 line 230: "This is exactly the split tldraw's agent-template uses (ops author, document persists)"; CANVAS §13 C4.B1 references the pattern -- no external verification needed (internal design decision backed by cited prior art)
mermaid import bridge via blueprint-parser -> native-shape | GROUNDED | CANVAS §8 lines 270-271: "Text DSLs (mermaid, plantuml) are import-only through a blueprint parser > native-shape bridge (the tldraw @tldraw/mermaid pattern...)" -- verbatim match
mermaid npm dep (@tldraw/mermaid) anti-slopsquat | GROUNDED | npm view @tldraw/mermaid: latest=5.1.0, modified 2026-06-03, repo=github.com/tldraw/tldraw, license=SEE LICENSE IN LICENSE.md (tldraw source-available). @mermaid-js/mermaid-core does NOT exist (404). The plan uses mermaid as a DSL IMPORT backend only; the actual mermaid npm package (mermaid@11.15.0, MIT, modified 2026-05-11) is the parser-side dep if needed
media_validator Tier2 SHIPPED, capture = Tauri-native | GROUNDED | C17 + CANVAS §11: "capture_canvas_region uses Tauri's own webview screenshot capability"; Playwright banned; media_validator REUSED; task_runner.rs:6208 confirmed
FSM untouched (mode flag not FSM substate) | GROUNDED | CANVAS §11 line 334: "The FSM is NOT touched. There is no CanvasDesigning FSM substate" -- verbatim; mode flag on canvas.svelte.ts confirmed
CommittedToolSet composes with CanvasDispatcher (not duplicated, GD-1) | GROUNDED | C17 + capability_flow.rs:53 live; CANVAS §11 line 330 "composes with F-3p1 CommittedToolSet" -- consistent

SSB PATTERN-2 CONSISTENCY CHECK (clause iii of task)

H-13 proposes: ONE SsbActionType::CanvasOpApplied + CanvasOpKind 10-arm discriminator. This is Pattern-2 single-variant. Direct precedent: H-11 CanvasActivityEvent (ssb.rs:35288) -- "Pattern-2 single-variant -- 4-arm CanvasActivityKind". Also: E-22 GithubTrendingEvent 3-arm, E-10 CriticDecisionEvent 3-arm, E-43 SkillSystemEvent 3-arm, E-20 ChatUxCritiqueEvent 13-arm. Pattern-2 single-variant with N-arm Kind is the established ELAI convention for grouping related events. H-13's 10-arm CanvasOpKind is CONSISTENT with this convention. The SCHEMA bump must be from 220 -> 221 (one new variant), not 207 -> 208. TOTAL_VARIANTS bump from 497 -> 507 (1 new SsbActionType + 10 new CanvasOpKind arms = 11 new payload enum arms, but only 1 new SsbActionType variant in the lock count -- the lock counts SsbActionType variants, not Kind arms). Re-read the H-11 precedent: CanvasActivityEvent adds 1 to SsbActionType, 4 arms to CanvasActivityKind. Lock bumps by 1. H-13: CanvasOpApplied adds 1 to SsbActionType, 10 arms to CanvasOpKind. Lock bumps by 1 (497 -> 498), e45 contract assertion must update to 498.

ACTIONS REQUIRED

- MISMATCH (blocking): H-13 cites live SCHEMA_VERSION=207 / TOTAL_VARIANTS=484. Verified live: SCHEMA_VERSION=220 (ssb.rs:26969, H-11), TOTAL_VARIANTS=497 (action_type_lock.rs:624). Update H-13 "SCHEMA bump baseline+1 (LIVE 207/484 -> re-read; bump e45:294 if first)" to read "LIVE 220/497; bump from 220->221 / 497->498; update e45_f7 assertion from 497 to 498". Resolve before TESTING_FIRST.
- VERIFY (non-blocking, resolve before coding): SSB naming divergence -- plan uses CanvasOpApplied + CanvasOpKind (Pattern-2); spec §11 names 10 separate SsbActionType strings. Implementer must follow Plan (Pattern-2) not spec literal names. Annotate in H-13 Task 3 that the spec §11 variant names are superseded by Pattern-2 convention. H-11 is the canonical precedent.
- VERIFY (non-blocking): @tldraw/mermaid dep for ARCHITECTURE mode mermaid import bridge. Verified: @tldraw/mermaid@5.1.0 EXISTS (latest 2026-06-03, github.com/tldraw/tldraw, license SEE LICENSE IN LICENSE.md -- tldraw source-available license, NOT MIT). Confirm licensing is acceptable before adding to package.json. If only the mermaid parser is needed (not tldraw's bridge), mermaid@11.15.0 (MIT) is the alternative.
- INGEST (non-blocking, pre-TESTING_FIRST): arXiv:2501.10868 (JSONSchemaBench) is cited but not ingested. The compile-timeout cliff claim is partially grounded via the benchmark's framing of "compilation efficiency" as a gap area but the exact cliff threshold for XGrammar is not verified from the paper text. Spawn paper-ingestor mode=paper arXiv:2501.10868 to pin the exact XGrammar compile-ceiling number before H-13 execution sets the CI ceiling.

VALIDATION WARNINGS

- SCHEMA_VERSION stale: H-13 body says "LIVE 207/484" but live is 220/497 -- this is a blocking MISMATCH (see above).
- CANVAS §13 C4 is the authoritative phased build plan; H-13 maps to C4.B1+B2+B3 combined. The plan correctly collapses all three C4 babies into one H-13 unit.
- H-11 (SCHEMA_VERSION 220) CanvasActivityEvent is SHIPPED and is a direct predecessor to H-13's CanvasOpApplied. H-12 is listed as a prerequisite -- verify H-12 SHIPPED before H-13 EXECUTING.
- No spec files are missing; all cited paths resolve.
- The Anthropic 3-arm pattern (catalog/convert/dispatch) is ELAI-internal architecture mapped onto the Anthropic tool-use API (name/description/input_schema per tool); the API itself does not prescribe this decomposition. GROUNDED.
