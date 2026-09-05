Phase 4D.17-4D.18 Architecture Decisions

Decision A: IPC variants for search/fetch/embed
Choice: Option 3 — no new IPC enum variants.
Rationale: WebResearchOrchestrator lives in the orchestrator process and calls
SearxClient/WebFetcher directly (no IPC needed). InferRequest/InferResponse
(variants 2/3) are reused for formulate_query and reflect LLM calls.
Add TOOL_SEARCH_REQUEST, TOOL_FETCH_REQUEST, TOOL_EMBED_REQUEST constants
plus typed request structs to ipc-types for future Tier C agent proxy use.
IPC_MESSAGE_TYPE_VARIANT_COUNT stays at 15.

Decision B: SSB action_type for web research steps
Choice: Add WebResearchStep as new variant.
Rationale: Web research is semantically distinct from code research (ResearchStep).
Separate action_type enables SSB analytics queries by research type.
Changes: SsbActionType::WebResearchStep, CHECK constraint update,
as_sql_str/ssb_action_type_from_str, SCHEMA_VERSION bump 6->7,
SsbPayload::WebResearchStep(WebResearchStepPayload), action_type() match arm.

Decision C: elai-web-search dependency
Choice: Add elai-web-search to elai-orchestrator dependencies.
Rationale: WebResearchOrchestrator struct fields require SearxClient, WebFetcher,
BasicExtractor, SourceQualityTable from that crate. Mandatory for 4D.17.

Expected test outcomes:
- 8 contract tests pass after implementation
- cargo test -p elai-orchestrator --lib -- web_research passes
- cargo check --workspace exits 0
- IPC_MESSAGE_TYPE_VARIANT_COUNT remains 15
- SCHEMA_VERSION = 7
