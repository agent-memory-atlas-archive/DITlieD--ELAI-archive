PHASE DECISION DOC — B-39 Chat REFLECT Pipeline
Session: s-30584
Phase: B-39
FSM: TESTING_FIRST -> EXECUTING
Date: 2026-05-10

WHAT I WILL IMPLEMENT
17 files (15 modified, 2 new):

NEW
- .elai_cc/crates/reasoning/src/chat_reflect.rs (~450-550 lines): TurnClass enum, ChatReflectConfig, ReflectOutput, ReflectTokenSummary, ReflectError (thiserror), ChatInferenceClient trait, ChatReflectPipeline struct + async run() + stage methods (triage/reasoner/auditor/synthesize). N_CANDIDATES=4 const, REFINE_CAP=3 const. SidecarProvider impls ChatInferenceClient via thin wrapper.
- .elai_cc/crates/contracts/tests/chat_reflect_contract.rs (491 lines, written in TESTING_FIRST; currently TDD-red, expected to compile after EXECUTING TASK 1)

MODIFIED
- reasoning/src/lib.rs: pub mod chat_reflect + re-exports
- reasoning/src/config.rs:336 agent_grc_config_for_role: add chat_reasoner + chat_auditor arms before wildcard at L375
- reasoning/Cargo.toml: add elai-providers = { path = "../providers" } (circular-dep cleared)
- orchestrator/src/ssb.rs: 14-site coordinated edit (SCHEMA_VERSION 28->29, 4 SsbActionType variants, 4 payload structs, CHECK DDL, as_sql_str, from_str, SsbPayload, test_payload, all_test_payloads, arb_payload)
- orchestrator/src/ssb_viewer.rs: get_reflect_state SQL reader (action_type IN (...) pattern at L860 precedent)
- cli/src/commands.rs: SlashCommand::Reason variant + parse arm + 1 SLASH_COMMAND_SPECS entry (argument_hint Some("<on|off>")); update assertion L4279 from 146 -> 147
- cli/src/resume.rs:687-743 unsupported wildcard: append Reason arm
- cli/src/live_cli.rs: 2 new fields on LiveCli at L94 (reflect_enabled bool init false, reflect_pipeline Option<ChatReflectPipeline> init None); Reason arm in handle_repl_command; reflect gate in run_turn at L1659 after augment_with_knowledge before runtime.run_turn; SlashCommand match at L1846+ Reason arm
- dashboard-v2/src-tauri/src/commands.rs: get_reflect_state Tauri command
- dashboard-v2/src-tauri/src/lib.rs: register get_reflect_state in tauri::generate_handler!
- dashboard-v2/src/lib/api/types.ts: 5 new interfaces (snake_case fields verbatim from Rust)
- dashboard-v2/src/lib/api/stub.ts: get_reflect_state with tryInvoke + MOCK_REFLECT_STATE
- dashboard-v2/src/lib/state/drawers.svelte.ts: 7th $state slot inside createReasoning (unwired sentinel + 7th Promise.all entry)
- dashboard-v2/src/lib/drawers/ReasoningDrawer.svelte: REFLECT panel (~80-120 lines, Svelte 5 runes only, zero $: labels)
- contracts/tests/activity_entry_coverage_contract.rs:43-132: append 4 new variants to hand-enumerated list (impact-mapper gap)

KEY DECISIONS (locked, with grounding)
1. N_CANDIDATES = 4 -- DELIBERATE LATENCY TRADEOFF documented in chat_reflect.rs comment citing Liu et al. arXiv:2502.06703 Table 5 / Figure 4. Liu data shows N=16 is the empirical Pareto-elbow minimum for Qwen 3B/7B-class (BoN with PRM); N=4 is the floor of {4,16,64,256}. Justification for sticking with 4: each candidate is a sequential /v1/messages POST to a single-instance GPU-bound sidecar; N=16 pushes wall-clock to 30-60s/turn vs ~10-20s at N=4. Future work surface: /reason wide flag to bump N=16 (accuracy mode).
2. REFINE_CAP = 3 -- per RefineBench arXiv:2511.22173 Table 3 guided-refinement data: t=2->t=3 yields +12.6pp (Claude-Opus-4.1) / +17.4pp (o3-mini). Cap=2 captured most of self-refinement gain but undersells guided mode (which is what we have because the auditor receives external grounding from tools/knowledge). Cap=3 captures the bulk of guided gain; t>=4 is diminishing.
3. ChatInferenceClient trait (NEW) defined in chat_reflect.rs, NOT &dyn LlmProvider. LlmProvider's complete(prompt, &OllamaConfig) signature carries an unused OllamaConfig (vestigial -- SidecarProvider ignores it); ChatInferenceClient gives a clean async fn complete(&self, prompt: &str, max_tokens: u32) -> Result<String, ReflectError>. SidecarProvider impls ChatInferenceClient via thin wrapper.
4. ChatReflectPipeline lives in reasoning/ crate (not cli/) -- composes PipelineGrc instances; reasoning/Cargo.toml gains elai-providers dep (verified non-circular: providers does not depend on reasoning).
5. GroundingLevel::High for chat_reasoner role (Medium variant absent in types.rs). chat_auditor stays Verified.
6. Single slash spec entry "/reason" with argument_hint Some("<on|off>") -- SlashCommand::parse splits on first word so two specs both named "reason" are impossible. slash_command_specs() bumps 146->147 (NOT 148). On/off validation in handle_repl_command.
7. SCHEMA_VERSION 28->29 (NOT 26->27 -- B-23 Phase 4H Group A bumped to 28).
8. Intercept site is LiveCli::run_turn at live_cli.rs:1659, NOT a fictional ConversationRuntime::send_message.
9. SYNTHESIZE stage is local string selection -- no extra sidecar call. Total inference calls per chat turn: 1 triage + N_CANDIDATES (4) reasoner + 1 auditor = 6. Multiply by REFINE_CAP at most.
10. Pipeline never panics. Any failure (sidecar down, JSON parse error, timeout) returns ReflectOutput { skipped: true, final_reply = original input passed unchanged to runtime.run_turn }. tracing::warn! on every fallback path.

EXPECTED TEST OUTCOME
- After TASK 1 (reasoning crate): chat_reflect_contract.rs compiles. 11 contract tests PASS for trait/struct shape; pipeline tests use MockChatInferenceClient.
- After TASK 2 (orchestrator SSB): activity_entry_coverage_contract.rs continues passing (with 84 variants now). chat_reflect_contract.rs SCHEMA_VERSION test passes (29).
- After TASK 3 (CLI): cargo check -p elai-cli passes; SlashCommand::Reason exhaustive match satisfied in resume.rs + live_cli.rs.
- After TASK 4 (Tauri): cargo check -p elai-dashboard-v2 src-tauri passes.
- After TASK 5 (TS): svelte-check 0 errors. ReasoningDrawer renders REFLECT panel (mock data when get_reflect_state returns null).
- After TASK 7 (build check): cargo check --workspace PASS, svelte-check PASS, all 12 chat_reflect contract tests PASS, activity_entry_coverage continues passing.
- Auditor (STEP 9) verifies: pipeline-fallback test (test 6) returns skipped=true on provider error; no .unwrap() in chat_reflect.rs; no .unwrap() in live_cli.rs additions; SCHEMA_VERSION pinned to 29 across the codebase; slash count = 147.

UNKNOWNS / RISKS
1. SidecarProvider's exact constructor signature for the on-demand build inside live_cli.rs run_turn -- the synthesizer pointed at build_hyde_sidecar_provider at sidecar.rs:238 as the precedent; verifying that pattern transfers cleanly to a chat-turn context (no audio/embedding-specific wiring).
2. /v1/messages JSON parsing of the triage response (we expect a TurnClass enum value in JSON; if the model returns prose, fall back to TurnClass::Chat with tracing::warn!).
3. Test 8 (REASONER N=4 writes 4 reflect_candidate rows) requires the pipeline to either run candidates concurrently (tokio::join_all) or sequentially. Sequential is simpler and matches the latency budget rationale; concurrent runs would let a single sidecar batch but Qwen2.5-Coder NF4 single-instance can't batch -- sequential it is.
4. ReasoningDrawer panel layout: the image the user mentioned but didn't share is unknown; defaulting to a section that mirrors the existing GRC budget rows (label + value + small bar), with an enable toggle at the top.
5. Liu paper warned PRM-policy family mismatch degrades below majority vote; we don't have a PRM, we use the same Qwen2.5-Coder model as both reasoner and auditor. Self-as-verifier is weaker than dedicated PRM but is the only option for a single-model setup. Auditor receives EXTERNAL grounding (test output, file reads) per Kamoi 2024 -- this mitigates self-verification weakness.
6. MEDIUM-confidence cross-lang risk: serde wire format. ssb.rs payloads currently rely on default Rust serde naming; for snake_case Rust struct fields the wire is naturally snake_case, but I will explicitly add #[serde(rename_all = "snake_case")] on all 4 new payload structs as a guard. TypeScript interfaces mirror snake_case verbatim.
