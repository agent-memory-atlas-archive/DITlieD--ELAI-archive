GROUNDING SWEEP -- phase F3p1
Source: F:\.ELAI_workflow\docs\thefinalglance\F-plan\F-3_camel_toolfilter_dataplane.md
Date: 2026-06-01
FSM: IDLE | Session: s-6344

SPEC REFERENCES
No "Spec references:" lines in the F-3 plan. Primary grounding sources listed in plan body:
workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md | GROUNDED | file confirmed at exact path, section 4 C3 + section 1.1 exist with the cited content
arXiv:2406.13352 AgentDojo Tool Filter | GROUNDED (synthesis TABLE-VERIFIED) | synthesis doc section 1.1 line 81 states "Tool Filter 57.7%->6.84% ASR / 73% utility ARE primary-verified and safe to cite"; paper NOT ingested in workflow/papers or .claude/papers
arXiv:2503.18813 CaMeL SaTML 2026 | VERIFY_NEEDED | synthesis doc section 1.1 states "arXiv:2503.18813 PDF did not render in the research pass" and the 77%/84% pair is ABSTRACT-VERIFIED only; paper NOT ingested; token-overhead 2.82x explicitly labelled secondary-only
arXiv:2506.08837 Design Patterns for LLM Agents | VERIFY_NEEDED | cited for TIER-3 framing only; paper NOT ingested; not load-bearing for TIER-1 work

CLAIM CLASSIFICATION
tool_filter.rs ToolCategory enum at line 29 | GROUNDED | elai.map:83196 DEF_ENUM ToolCategory::Bash at tool_filter.rs:29
tool_filter.rs ToolCategory::ALL at line 61 | GROUNDED | elai.map:55168 DEF_CLASS ALL at tool_filter.rs:61
tool_filter.rs ToolCategory::classify at line 86 | GROUNDED | elai.map:104122 SCIP_DEF + 330196 DEF_FN at tool_filter.rs:86
tool_filter.rs FsmState::allowed_tool_categories at line 227 | GROUNDED | elai.map:56033 SCIP_DEF + 338044 DEF_FN at tool_filter.rs:227
tool_filter.rs FsmState::check_tool_allowed at line 370 | GROUNDED | elai.map:100608 SCIP_DEF + 292510 DEF_FN at tool_filter.rs:370
tool_filter.rs FsmState::active_rule_categories at line 334 | GROUNDED | elai.map:7208 DEF_FN at tool_filter.rs:334
fsm.rs Fsm::check_tool_allowed at line 695 | MISMATCH | elai.map:464492 DEF_FN at fsm.rs:1158; line 695 is a comment inside with_compliance_provider builder, not the function definition; plan says "(CONFIRMED line 695)" -- WRONG
web_research_orchestrator.rs formulate_query at line 533 | GROUNDED | elai.map:78350 DEF_FN + direct file grep confirms pub async fn formulate_query at line 533
web_research_orchestrator.rs fetch_and_extract at line 592 | GROUNDED | elai.map:57129 DEF_FN at line 592
web_research_orchestrator.rs search_and_rank at line 564 | GROUNDED | direct file grep pub async fn search_and_rank at line 564
web_research_orchestrator.rs emit_stage at line 111 | GROUNDED | elai.map:45794 DEF_FN emit_stage at line 111
web_research_orchestrator.rs budget_manager at line 218 | GROUNDED | direct file grep pub fn budget_manager at line 218
mcp/gate.rs check_tool_allowed at line 127 | GROUNDED | elai.map:100681 DEF_FN at gate.rs:127
ipc_gateway.rs allowed_grounding_methods at line 215 | GROUNDED | elai.map:243195 DEF_FN at ipc_gateway.rs:215
ipc_gateway.rs AgentConnection::check_tool_allowed at line 122 | VERIFY | function body starts at line 125 (line 122 is blank/comment); close enough but not exact
per_role_runtime.rs AgentRole::Researcher at line 188 | GROUNDED | elai.map:13662 METHOD_CALL at per_role_runtime.rs:188
sandbox/mod.rs AgentRole::Researcher => compile_seccomp_network_extra at line 1337 | GROUNDED | elai.map:48315 CALL at sandbox/mod.rs:1337
prompt_armor.rs ClassifierVerdict at line 33 | GROUNDED | elai.map:76644 DEF_ENUM at prompt_armor.rs:33; direct read confirms pub enum ClassifierVerdict at line 33
prompt_armor.rs InjectionClassifier at line 92 | GROUNDED | elai.map:238415 DEF_TRAIT at prompt_armor.rs:92; direct read confirms pub trait InjectionClassifier at line 92
prompt_armor.rs HeuristicClassifier at line 107 | GROUNDED | elai.map:378881 DEF_STRUCT at prompt_armor.rs:107
prompt_armor.rs classify_injection at line 193 | GROUNDED (off-by-one) | elai.map:438867 DEF_FN at prompt_armor.rs:193; plan cites ":254" (a test call site) but DEF is at 193
prompt_armor.rs wrap_spotlight at line 210 | GROUNDED | elai.map shows wrap_spotlight used at prompt_armor.rs:210 (test); SCIP_IMPORT at line 204
safety_cmd.rs run at main.rs:535 | GROUNDED | elai.map:176402 CALL safety_cmd::run at main.rs:535
safety_cmd.rs SafetyAction at args.rs:1068 | GROUNDED | elai.map:172206 DEF_ENUM SafetyAction at args.rs:1068
commands_c35.rs get_tool_gate_snapshot at line 81 | GROUNDED | direct read confirms Tauri fn at line 77, state logic at line 81
commands_c35.rs list_tool_gate_rows at line 161 | MISMATCH | actual function at line 105; line 161 is now_ms() helper; plan says "(line 161)" -- WRONG
commands_c35.rs detail_to_row at line 169 | GROUNDED | direct read + elai.map confirm fn detail_to_row at line 169
ToolGateInspector.svelte action_type === "tool_denied" | GROUNDED | direct read line 72 + elai.map:6881
ssb.rs tool_denied parse arm at line 9087 | GROUNDED | direct read confirms "tool_denied" => Ok(SsbActionType::ToolDenied) at line 9087
SCHEMA_VERSION 191 (cited from CLAUDE.md) | MISMATCH | actual ssb.rs const SCHEMA_VERSION = 195 at line 2726; CLAUDE.md says 191 which is stale; plan inherits the stale baseline from CLAUDE.md
blake3 as workspace dep | GROUNDED | Cargo.toml line 65 blake3 = "1.8"; crates/orchestrator/Cargo.toml line 82 blake3 = { workspace = true }
E-37 symbols (prompt_armor.rs) SHIPPED | GROUNDED | prompt_armor.rs added in commit 23a3e9a (completion-drive session 2/4); symbols confirmed in elai.map
capability_flow.rs does NOT exist yet | GROUNDED | elai.map grep finds zero matches for capability_flow; confirmed net-new
57.7% -> 6.84% ASR at 73% utility (AgentDojo Tool Filter) | GROUNDED (synthesis-verified) | synthesis doc section 1.1 line 81 explicitly marks these TABLE-VERIFIED and safe to cite
77%/84% provable-security (CaMeL ABSTRACT-VERIFIED) | VERIFY | synthesis marks ABSTRACT-VERIFIED; PDF not rendered; paper not ingested; safe to cite as motivation for TIER-3, NOT as TIER-1 evidence
2.82x token overhead (CaMeL) | GROUNDED (correctly labelled secondary-only) | plan correctly labels this secondary-only per synthesis 1.1; plan does NOT use it as primary; correct
SVEN 59.1->92.3 / CodeGuard+ 43.2->76.0 | NOT IN F-3 | these numbers belong to C2, not C3; F-3 correctly does not cite them; no issue
Springdrift numbers | NOT IN F-3 | F-3 does not cite Springdrift; no issue
E-24 prerequisite (MCP pin) | VERIFY | plan cites E-24 for "pinned tool set the filter draws from"; E-24 file exists (E-plan/E-24_agent_protocols_mcp.md); ships the MCP schema-hash pinning and tool filter interface; not confirmed fully shipped but F-3 TIER-1 path uses AgentConnection::check_tool_allowed which is confirmed present
C-11 prerequisite (OS isolation) | VERIFY | C-11 is PLANNED (not SHIPPED) per E-37 plan text "C-11 PLANNED"; sandbox/mod.rs:1337 seccomp code IS in the codebase (shipped separately in commit 6e0954a); C-11 as a plan may be unshipped but the seccomp primitive it provides IS present; F-3 TIER-1 does not call seccomp at gate time so this is low-risk
producers.rs details_json pattern at lines 166/203/242/278/316 | GROUNDED | direct grep confirms all five lines present

ACTIONS REQUIRED
- MISMATCH: fsm.rs Fsm::check_tool_allowed line -- plan says 695 (a comment line inside with_compliance_provider), actual DEF is at line 1158. Fix the line reference in the Existing state section and Reads section of F-3 before TESTING_FIRST. The symbol itself is real and reachable; only the line number annotation is wrong.
- MISMATCH: commands_c35.rs list_tool_gate_rows line -- plan says 161 (now_ms helper), actual fn is at line 105. Fix the line reference in the Existing state section of F-3 before TESTING_FIRST. The symbol itself is real and reachable.
- MISMATCH: SCHEMA_VERSION -- plan references baseline 191 (from CLAUDE.md) but actual ssb.rs has SCHEMA_VERSION=195. The implementation must bump from 195+1=196, not 191+1=192. Update the SCHEMA bump description in F-3 Task 4 and in the WIRING_CONTRACT section to "SCHEMA_VERSION 195 -> 196".
- ingest paper arXiv:2406.13352 AgentDojo Tool Filter via paper-ingestor mode=paper -- required before TESTING_FIRST for the 57.7%->6.84% / 73% utility numbers. Currently TABLE-VERIFIED via synthesis only; paper-level ingest needed to confirm the table numbers and fixture design details (>=20 fixtures requirement in Task 7).
- ingest paper arXiv:2503.18813 CaMeL via paper-ingestor mode=paper -- required for the TIER-3 gate conditions (token-overhead measurement source). The PDF failed to render in the research pass; this needs a fresh fetch. ABSTRACT-VERIFIED status is insufficient for TIER-3 scheduling.
- arXiv:2506.08837 Design Patterns for LLM Agents -- TIER-3 framing only; not blocking TIER-1. Low priority, INGEST before TIER-3 scheduling.
- verify C-11 shipped status before referencing sandbox boundary in contract tests -- seccomp primitive is present (sandbox/mod.rs:1337 confirmed) but C-11 as an integrated plan may be incomplete; flag if any F-3 contract test assumes a running Tier-C sandbox.

VALIDATION WARNINGS
no "Spec references:" lines in F-3 source -- F-N files do not follow the B-N spec-reference convention; primary grounding is the synthesis doc + elai.map W-7 evidence; acceptable for F-plan format
SCHEMA_VERSION in CLAUDE.md (191) is stale vs codebase (195) -- this is a pre-existing drift; F-3 must use the real codebase value (195) for the bump, not the CLAUDE.md value
three cited papers (2406.13352, 2503.18813, 2506.08837) not ingested -- synthesis-level grounding is acceptable for TIER-1 mechanism design; paper-level ingest required before publishing any numbers or scheduling TIER-3
ipc_gateway.rs AgentConnection::check_tool_allowed line 122 vs 125 -- off-by-one (122 is blank line before the doc comment; 125 is the pub fn); minor, not blocking
prompt_armor.rs classify_injection plan cites ":254" (test line) vs DEF at :193 -- the plan's Reads section correctly specifies lines 240-270 which covers both; not blocking
