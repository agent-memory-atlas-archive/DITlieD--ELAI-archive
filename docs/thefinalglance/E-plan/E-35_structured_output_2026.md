# E-35: Structured output substrate — XGrammar canonical + INSTRUCTOR mode + per-role grammar cache + CodeAct executor
Objective: ship the structured-output SUBSTRATE that E-18 / E-19 / E-20 reference: (a) XGrammar as canonical Tier-2 backend with per-(role, block, schema_hash) grammar cache warmed at boot (arxiv:2411.15100, default in vLLM/SGLang/TensorRT-LLM/MLC-LLM); (b) INSTRUCTOR mode (unconstrained `<think>` then XGrammar-masked `<answer>`) as default for reasoning roles per CRANE (arxiv:2502.09061) + "Think Inside the JSON" (arxiv:2502.14905); (c) CodeAct (arxiv:2402.01030 ICML 2024, +20% success / -30% turns) for a new `runner` AgentRole that composes tool calls in Python; (d) SEARCH/REPLACE non-git fences for the Edit tool path; (e) cloud strict-JSON-Schema (OpenAI Aug-2024 / Anthropic structured-outputs-2025-11-13 / Gemini responseJsonSchema Nov-2025) for cloud-route requests; (f) Qwen2.5-Coder `<tools>` parser (NOT hermes — Coder variant ignores hermes per hanXen vllm fork). E-35 is the SUBSTRATE; E-18 (already written) builds tool-call reliability ON TOP of these primitives.
Estimated effort: 2-3 sessions, ~10-14 h
Pre-requisites: B-37 SHIPPED (RuntimeAdapter); B-7 SHIPPED via B-7b (pipeline factory + tool-call IPC types); E-18 PLANNED (tool-use reliability — sequence E-35 BEFORE E-18 since E-18 builds on E-35's XGrammar + envelope primitives); B-20 PLANNED (tool registry unification); B-28 PLANNED (Focus Tunnel — depends on E-35's mode-selection primitives).
Parallel-safe with: E-31, E-32, E-33, E-34.
Source report: workflow/improvement-sweep-2026-05-19/reports/35-structured-output-2026.md

## Context

Constrained decoding measurement (arxiv:2411.15100 XGrammar MLSys 2025, arxiv:2501.10868 JSONSchemaBench v3 2025): XGrammar runs at <40us/token JSON with cached grammar; 100x speedup vs FSM; 97.1% schema-accurate on Qwen-2.5-32B nested schemas vs Outlines 76.4%. XGrammar is the default in vLLM, SGLang, TensorRT-LLM and MLC-LLM as of March 2026 (redhat.com/articles/2025/06/03/structured-outputs-vllm + chatforest.com/reviews/sglang-structured-generation-llm-serving + squeezebits.com guided-decoding benchmark). For ELAI's 8 BabyStepSpec blocks × 8 AgentRoles = 64 grammars to cache; per JSONSchemaBench, cache reuse is where XGrammar wins (uncached compile can hit seconds-to-minutes on complex schemas).

CRANE (arxiv:2502.09061) measured that strict-grammar enforcement masks tokens that mid-reasoning would otherwise reach — reasoning accuracy drops. DCCD (arxiv:2603.03305) shows +24pp on GSM8K vs naive constrained on 1B model. "Think Inside the JSON" (arxiv:2502.14905) demonstrates RL+SFT with `<think>`/`<answer>` tag-format reward stabilises strict schema adherence on small models. Rule: any agent role whose job involves reasoning (auditor, planner, decomposer, idea-framer) MUST run INSTRUCTOR mode (unconstrained CoT then constrained answer); GRAMMAR-only mode is acceptable only for high-stakes-low-reasoning blocks (FailureTaxonomy enum, BabyStepSpec status enum, ReasoningTemplate type-tag).

CodeAct (arxiv:2402.01030 ICML 2024): +20% success, ~30% fewer turns on API-Bank vs JSON tool args; lets the model compose multiple tools in one Python action. Best for runner/executor roles that compose tools, NOT for surgical file edits. SEARCH/REPLACE is the winning edit format for Qwen2.5-Coder-7B per aider polyglot Nov-20-2025 leaderboard (88.0% top model with diff; small models gain little from diff over whole — Aider docs explicit). For Qwen2.5-Coder specifically use the `<tools>` parser, NOT hermes — hanXen/vllm-qwen2.5-coder-tool-parser documents the Coder variant ignores hermes (this is the same silent-fallback bug E-18 references; E-35 is the canonical fix at the parser layer).

Cited grounding: arxiv:2411.15100 XGrammar, arxiv:2501.10868 JSONSchemaBench, arxiv:2502.09061 CRANE, arxiv:2402.01030 CodeAct ICML 2024, arxiv:2502.14905 "Think Inside the JSON", arxiv:2603.03305 DCCD, arxiv:2510.12487 Diff-XYZ JetBrains, github.com/mlc-ai/xgrammar, github.com/guidance-ai/llguidance, github.com/hanXen/vllm-qwen2.5-coder-tool-parser, aider.chat/docs/leaderboards/edit.html, platform.claude.com/docs/en/build-with-claude/structured-outputs, openai.com/index/introducing-structured-outputs-in-the-api, ai.google.dev/gemini-api/docs/structured-output.

## Existing state (B/C/D coverage)

- B-37 (SHIPPED commit fd5c2e5): RuntimeAdapter trait + sidecar runtime. E-35's XGrammar LogitsProcessor wires into the sidecar runtime's `complete`/`stream_step` paths at `crates/providers/src/runtime/sidecar.rs`.
- B-7 (SHIPPED via B-7b): tool-call IPC types exist. E-35 augments with the `<tools>`/hermes parser-variant detection.
- E-18 (PLANNED — Tool-use reliability, already authored): EXPLICITLY BUILDS ON E-35's XGrammar substrate. E-18's task 1 ("Wire XGrammar as Tier 2") becomes a thin call-into-E-35 once E-35 lands. Sequence: E-35 first, then E-18. E-18's WIRING_CONTRACT references XGrammarLogitsProcessor + compile_all_schemas_at_boot which E-35 defines.
- E-19 (already authored, reasoning quality): consumes the INSTRUCTOR mode primitive — auditor/planner/decomposer/idea-framer agents go through `<think>`/`<answer>` pattern.
- E-20 (already authored, local-cloud routing): consumes the cloud strict-mode wiring — when CloudGroundingClient routes to Anthropic/OpenAI/Gemini, schema must be passed as strict not legacy json_object.
- B-20 (PLANNED, tool registry unification): consumes the `<tools>`-vs-hermes parser-variant detection from E-35.
- B-28 (PLANNED, Focus Tunnel E-1..E-7 internal numbering): consumes the mode-selection primitive — STRUCTURED mode IS XGrammar mode in E-35's terms.
- BabyStepSpec + AgentRole types: today's count is 8 blocks per BabyStepSpec + 8 AgentRoles (B-40 SHIPPED introduced +5, B-41 SHIPPED added auditor_mode); verify final count at grep time. The 64-grammar cache is sized to this matrix.
- CLAUDE.md anti-stale: native async fn in traits (Rust); torch.inference_mode() (Python); torch_dtype= in from_pretrained — E-35's Python additions respect all of these.
- Memory:project_qwen3b_baseline (25K tokens 100% pass, NF4+FA2, 1.92GB): E-35's grammar cache warmup must NOT push boot time beyond ~5s on the 12GB local OR ~30s on the 24GB remote.

## Reads (mandatory before coding)

- workflow/improvement-sweep-2026-05-19/reports/35-structured-output-2026.md (full)
- F:\.ELAI_workflow\.elai_cc\python\elai_inference\server.py (sidecar; existing LogitsProcessor chain — DoLa, ETC, steering; XGrammar inserts as an additional processor)
- F:\.ELAI_workflow\.elai_cc\crates\providers\src\runtime\sidecar.rs (complete + stream_step entry points; lines 209-380 per E-18 grounding)
- F:\.ELAI_workflow\.elai_cc\crates\tools\src\dispatcher.rs (Edit/Bash dispatch — SEARCH/REPLACE fences land here; lines 776-1003 hold ToolDispatchError variants per E-18)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\dispatcher.rs (state-context-aware dispatch; lines 83-342)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ipc_gateway.rs (IPC types — ToolRequest/ToolResult; will add CodeActAction variant)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs (SsbActionType enum)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\grounding_client.rs (CloudGroundingClient — strict-mode cloud schema passthrough)
- F:\.ELAI_workflow\.elai_cc\crates\contracts\tests\agent_loop_contract.rs (existing InferRequest contract — reference for envelope schema grounding)
- workflow/improvement-sweep-2026-05-19/E-plan/E-18_tool_use_reliability.md (already-authored — verify substrate symbols line up)
- workflow/improvement-sweep-2026-05-19/E-plan/E-19_reasoning_quality.md (if exists; cross-reference INSTRUCTOR mode consumer)
- workflow/improvement-sweep-2026-05-19/E-plan/E-20_local_cloud_routing.md (verify cloud strict-mode consumer)
- arxiv:2411.15100 + arxiv:2501.10868 + arxiv:2502.09061 + arxiv:2402.01030 + arxiv:2502.14905 + arxiv:2603.03305 PDFs (workflow/papers/ — ingest via paper-ingestor mode=paper if not present)

## Tasks

1. Wire XGrammar as the canonical Tier-2 structured-output backend at the sidecar (~280 LOC in `python/elai_inference/server.py`). New class `XGrammarLogitsProcessor` wraps `xgrammar.GrammarMatcher` + transformers `LogitsProcessor`. INSERTED AFTER existing DoLa/ETC/steering processors (CRANE-style: reasoning runs unconstrained first, then constrained). Function `compile_all_schemas_at_boot(role_x_block_matrix)` pre-compiles the 8x8 = 64 grammars; cached by `(role, block, blake3(schema_canonical_json))` key (CLAUDE.md anti-stale: blake3, NEVER blake2/sha2). Surface schema-compile failures as a startup error (loud > silent). Cite arxiv:2411.15100 + arxiv:2501.10868 + redhat.com/articles/2025/06/03/structured-outputs-vllm.

2. Implement INSTRUCTOR mode at the Provider trait (~180 LOC in `crates/providers/src/runtime/sidecar.rs` + ~80 LOC in `crates/providers/src/decoding.rs`). New method `Provider::generate_structured(role, block_schema, prompt) -> StructuredOutput`. Behavior: emit `<think>...</think>` unconstrained (existing logit-processor chain runs DoLa/ETC/steering), then `<answer>` with XGrammar mask snapped to schema; post-parse + validate. Default mode for AgentRoles: auditor, planner, decomposer, idea-framer (memory:project_b41_shipped per-level auditor). Cite arxiv:2502.09061 CRANE + arxiv:2502.14905 + DeepSeek-R1 pattern.

3. Add GRAMMAR mode for low-reasoning enum/header blocks (~30 LOC). When `block.constrained_kind == EnumOnly | TypeTag | StatusFlag`, skip the `<think>` phase and apply XGrammar mask directly. Used for: FailureTaxonomy classification, BabyStepSpec status enum, ReasoningTemplate type-tag. The 8 blocks-per-BabyStepSpec are tagged at the schema level so dispatch is deterministic.

4. Implement Qwen2.5-Coder `<tools>` parser variant detection (~140 LOC NEW at `python/elai_inference/tool_parser.py`). On boot, detect model variant via `model.config.name_or_path` or explicit env `ELAI_TOOL_PARSER_VARIANT={hermes|qwen_tools}`. For Qwen2.5-Coder family: use `<tools>...</tools>` envelope tokens (NOT `<tool_call>` per hanXen documentation). For Qwen2.5-7B-Instruct (non-Coder): use native hermes. Wire variant into `XGrammarLogitsProcessor`'s envelope grammar. Cite github.com/hanXen/vllm-qwen2.5-coder-tool-parser. NOTE: this is the canonical fix for the silent-fallback bug E-18 references — E-18's task 2 (Hermes envelope) BECOMES "use the right envelope per model variant" once E-35 lands.

5. Implement SEARCH/REPLACE matcher with custom non-git fences as a tier-1 Edit format (~240 LOC NEW at `crates/tools/src/search_replace.rs`). Tokens: `<<< ELAI_SEARCH` / `=== ELAI_REPLACE` / `>>> ELAI_END` (not valid Rust/Python/Svelte/TS syntax and not git conflict markers `<<<<<<< ` / `=======` / `>>>>>>> `). Includes `DelimiterCollisionGuard` that scans source for fence tokens + git-conflict markers; if present, falls back to whole-file rewrite path. Cite arxiv:2510.12487 Diff-XYZ + aider.chat leaderboard. NOTE: identical primitive to E-18 task 3 — E-35 SHIPS the SEARCH/REPLACE module; E-18 wires it into the dispatcher tier order.

6. Implement CodeAct execution channel as a new AgentRole `runner` (~320 LOC across `crates/orchestrator/src/agent_role_dispatcher.rs` + `crates/tools/src/codeact_sandbox.rs` + `crates/orchestrator/src/ipc_gateway.rs`). New IPC variant `CodeActAction { python_code: String, tool_allowlist: Vec<ToolId> }`. Python sandbox via existing subprocess + path-allowlist + timeout. Tools exposed as Python functions; LLM emits a code block instead of JSON tool-call. Cite arxiv:2402.01030 ICML 2024 (+20% success / -30% turns on API-Bank). New SSB row `codeact_action_executed` with `{python_lines, tools_invoked, turn_saved_count}`. Cap Python action size at 200 lines per request (anti-runaway).

7. Wire cloud strict-JSON-Schema in CloudGroundingClient (~120 LOC delta in `crates/orchestrator/src/grounding_client.rs`). When `provider == Anthropic`: pass `structured-outputs-2025-11-13` beta header + schema body. When `provider == OpenAI`: pass `strict: true` in the function spec. When `provider == Gemini`: pass `responseJsonSchema`. NEVER fall back to legacy `json_object` (syntax-only — anti-pattern per source report). Cite platform.claude.com structured-outputs + openai.com/index/introducing-structured-outputs-in-the-api + ai.google.dev/gemini-api/docs/structured-output. New SSB row `cloud_strict_schema_applied` with `{provider, schema_hash}`.

8. Implement per-role grammar cache with content-hash key (~80 LOC at sidecar + Rust side). Sidecar boot-time warmup of `(role, blockname, blake3(schema_canonical_json)) -> compiled_grammar`. Disk-cache the compiled grammars at `.elai_cc/.elai_cc/grammar_cache/{hash}.bin` (or wherever existing cache dirs live — grep first). On cache hit at boot, skip recompile. Source report Open Question 2: if total compile time > 1s wall-clock, cache mandatory. New SSB row `grammar_cache_loaded` with `{compiled_count, cache_hits, cache_misses, total_compile_ms}`.

9. Add DCCD-style draft-conditioned fallback for math/numeric blocks (~150 LOC — DEFERRED behind `ELAI_DCCD_ENABLED=1` env). When XGrammar masks a numeric block and the auditor/CSQ counterfactual requires it, generate an unconstrained DRAFT then select the closest valid emission. Cite arxiv:2603.03305 +24pp GSM8K. Reserve for CSQ counterfactuals + numeric Auditor evidence; off by default until source report Open Question 1 (private GSM-style synthetic eval) confirms uplift on ELAI workloads.

10. EXPLICITLY REJECT lm-format-enforcer for ELAI default at the schema-engine dispatcher (~20 LOC). When dispatcher would route to LMFormatEnforcer: refuse + log a `structured_engine_rejected` SSB row + fall back to XGrammar. Source report: "Reject lm-format-enforcer for ELAI default" — keep as a fallback ONLY for tokenizer edge-cases XGrammar can't handle.

11. EXPLICITLY REJECT legacy vendor `json_object` mode at CloudGroundingClient (~15 LOC). Refuse to pass `type: json_object` request body; route through strict schema or fail loud. Cite developers.openai.com structured-outputs guide + openai.com/index/introducing-structured-outputs.

12. Add SSB instrumentation rows so we can measure adoption (~30 LOC in `crates/orchestrator/src/ssb.rs`): `XgrammarMaskApplied`, `InstructorModeActivated`, `GrammarModeActivated`, `CodeactActionExecuted`, `SearchReplaceTier1Hit`, `SearchReplaceCollisionFallback`, `CloudStrictSchemaApplied`, `GrammarCacheLoaded`, `StructuredEngineRejected`, `LegacyJsonObjectRefused`. Without these, Open Questions 1-5 are unmeasurable.

13. Author `workflow/reasoning/E-35-decisions.md` (~140 LOC): (a) why XGrammar over llguidance/Outlines/lm-format-enforcer (40us/token, default in 4 inference engines, JSONSchemaBench 97.1%); (b) INSTRUCTOR mode default for reasoning roles per CRANE; (c) why qwen_tools variant for Coder family (hanXen documented silent-fallback bug); (d) why SEARCH/REPLACE + custom non-git fences (small-model unified-diff disorientation per Diff-XYZ + collision-guard); (e) why CodeAct for runner role but NOT for surgical edits (composition + branching wins; surgical wants targeted fences); (f) DEFERRED: DCCD (gated on internal eval), XGrammar-2 + SGLang 0.5.11 (source report Open Question 5 — measure first), lm-format-enforcer (kept as fallback only).

## WIRING_CONTRACT

produces_symbols:
  - python/elai_inference/server.py::XGrammarLogitsProcessor (new class)
  - python/elai_inference/server.py::compile_all_schemas_at_boot (new fn)
  - python/elai_inference/server.py::warm_grammar_cache_from_disk (new fn)
  - python/elai_inference/tool_parser.py::ToolParserVariant (new enum + detector)
  - crates/providers/src/runtime/sidecar.rs::Provider::generate_structured (new trait method)
  - crates/providers/src/decoding.rs::StructuredMode {INSTRUCTOR, GRAMMAR} (new enum)
  - crates/tools/src/search_replace.rs::SearchReplaceMatcher + DelimiterCollisionGuard (new module)
  - crates/tools/src/codeact_sandbox.rs::CodeActSandbox (new module)
  - crates/orchestrator/src/agent_role_dispatcher.rs::AgentRole::Runner (new variant)
  - ipc_gateway::CodeActAction (new IPC variant)
  - SsbActionType::XgrammarMaskApplied + InstructorModeActivated + GrammarModeActivated + CodeactActionExecuted + SearchReplaceTier1Hit + SearchReplaceCollisionFallback + CloudStrictSchemaApplied + GrammarCacheLoaded + StructuredEngineRejected + LegacyJsonObjectRefused (10 new variants)
consumes_symbols:
  - python/elai_inference/server.py existing LogitsProcessor chain (DoLa/ETC/steering; XGrammar inserts AFTER per CRANE)
  - crates/providers/src/runtime/sidecar.rs::complete + ::stream_step (extend signatures with StructuredMode + block_schema)
  - crates/orchestrator/src/grounding_client.rs::CloudGroundingClient (strict-schema passthrough for Anthropic/OpenAI/Gemini)
  - crates/tools/src/dispatcher.rs::dispatch_edit (consumes SearchReplaceMatcher as tier-1)
  - crates/orchestrator/src/dispatcher.rs::dispatch_tool (consumes CodeActAction variant for runner role)
reachable_from:
  - sidecar boot -> compile_all_schemas_at_boot -> grammar_cache_loaded SSB
  - any reasoning-role inference -> Provider::generate_structured (INSTRUCTOR mode) -> xgrammar_mask_applied SSB
  - runner-role task -> AgentRole::Runner -> CodeActAction -> codeact_action_executed SSB
  - Edit dispatch -> SearchReplaceMatcher tier-1 (or DelimiterCollisionGuard fallback) -> search_replace_tier1_hit | search_replace_collision_fallback SSB
  - cloud route -> CloudGroundingClient strict-schema -> cloud_strict_schema_applied SSB
stub_ban:
  - no legacy json_object code path remains reachable after task 11 (loud refuse)
  - no hermes `<tool_call>` envelope on Qwen2.5-Coder model variant after task 4 (use qwen_tools)
  - no synchronous schema-compile-on-request after task 8 (must hit cache or fail loud)
  - no CodeAct Python action > 200 lines without explicit user override
  - no .unwrap() in any production path (test code OK)
  - CLAUDE.md anti-stale: blake3 for schema-hash keys (NEVER blake2/sha2)
  - CLAUDE.md anti-stale: torch.inference_mode() in XGrammarLogitsProcessor forward (NEVER torch.no_grad)
  - CLAUDE.md anti-stale: torch_dtype= in any new from_pretrained (NEVER dtype=)
  - CLAUDE.md anti-stale: native async fn in traits (NEVER async-trait macro)
  - lm-format-enforcer MUST refuse to be default (fallback only)
frontend_pair: crates/dashboard-v2/src/lib/components/RuntimePanel.svelte (extends with current StructuredMode + grammar-cache-status row)
observable_effect:
  trigger: "elai /task --mode local 'plan a 3-phase build of an HTTP server with structured BabyStepSpec output'"
  expect_ssb: "grammar_cache_loaded (boot); instructor_mode_activated on planner role; xgrammar_mask_applied on the answer phase; codeact_action_executed if runner role hits a multi-tool composition"
  expect_artifact: "BabyStepSpec output parses cleanly against the JSON Schema; no silent ```json fallback; auditor-role output shows <think>...</think><answer>{valid JSON}</answer> structure"

## DETERMINISTIC_CHECK

```bash
# 1. XGrammar processor wired
rg -n "XGrammarLogitsProcessor|compile_all_schemas_at_boot|warm_grammar_cache_from_disk" .elai_cc/python/elai_inference/server.py
# Expected: >= 4 hits

# 2. Tool-parser variant detection present
test -f .elai_cc/python/elai_inference/tool_parser.py
rg -n "ToolParserVariant|qwen_tools|hermes" .elai_cc/python/elai_inference/tool_parser.py

# 3. INSTRUCTOR + GRAMMAR mode enum present
rg -n "StructuredMode|INSTRUCTOR|GRAMMAR" .elai_cc/crates/providers/src/decoding.rs

# 4. Provider::generate_structured trait method
rg -n "generate_structured" .elai_cc/crates/providers/src/runtime/sidecar.rs

# 5. SearchReplace module exists with non-git fences
test -f .elai_cc/crates/tools/src/search_replace.rs
rg "ELAI_SEARCH|ELAI_REPLACE|ELAI_END|DelimiterCollisionGuard" .elai_cc/crates/tools/src/search_replace.rs

# 6. CodeAct sandbox module exists
test -f .elai_cc/crates/tools/src/codeact_sandbox.rs
rg "CodeActSandbox|tool_allowlist" .elai_cc/crates/tools/src/codeact_sandbox.rs

# 7. AgentRole::Runner variant present
rg "AgentRole::Runner" .elai_cc/crates/orchestrator/src/agent_role_dispatcher.rs

# 8. CodeActAction IPC variant
rg "CodeActAction" .elai_cc/crates/orchestrator/src/ipc_gateway.rs

# 9. 10 new SsbActionType variants
rg "XgrammarMaskApplied|InstructorModeActivated|GrammarModeActivated|CodeactActionExecuted|SearchReplaceTier1Hit|SearchReplaceCollisionFallback|CloudStrictSchemaApplied|GrammarCacheLoaded|StructuredEngineRejected|LegacyJsonObjectRefused" .elai_cc/crates/orchestrator/src/ssb.rs
# Expected: 10 enum arms

# 10. Legacy json_object refused at cloud
rg "LegacyJsonObjectRefused|legacy.*json_object" .elai_cc/crates/orchestrator/src/grounding_client.rs

# 11. End-to-end smoke
elai /task --mode local "plan a 3-phase build of an HTTP server"
sqlite3 .elai_cc/.elai_cc/ssb.db "SELECT action_type, count(*) FROM ssb_events WHERE action_type IN ('grammar_cache_loaded','instructor_mode_activated','xgrammar_mask_applied') GROUP BY action_type;"
# Expected: 1 row for grammar_cache_loaded; >= 1 for instructor_mode_activated; >= 1 for xgrammar_mask_applied
```

## ACCEPTANCE

- [ ] XGrammar LogitsProcessor wired at sidecar boot; 64 grammars (8 blocks * 8 AgentRoles) pre-compiled with blake3-keyed disk cache
- [ ] `grammar_cache_loaded` SSB row emitted at sidecar boot with compiled_count + cache_hits + cache_misses + total_compile_ms
- [ ] INSTRUCTOR mode default for auditor / planner / decomposer / idea-framer roles (CRANE-style two-tag emission)
- [ ] GRAMMAR mode default for FailureTaxonomy enum / BabyStepSpec status enum / ReasoningTemplate type-tag blocks
- [ ] Qwen2.5-Coder model uses `<tools>` envelope NOT hermes `<tool_call>` (hanXen-documented fix)
- [ ] SEARCH/REPLACE custom non-git fence module (`<<< ELAI_SEARCH` / `=== ELAI_REPLACE` / `>>> ELAI_END`) shipped with DelimiterCollisionGuard
- [ ] CodeAct AgentRole::Runner variant + CodeActAction IPC variant + Python sandbox shipped; 200-line cap per action
- [ ] CloudGroundingClient passes strict JSON Schema for Anthropic (structured-outputs-2025-11-13) / OpenAI (strict:true) / Gemini (responseJsonSchema)
- [ ] Legacy `json_object` mode REFUSED at CloudGroundingClient with `legacy_json_object_refused` SSB row
- [ ] lm-format-enforcer REFUSED as default with `structured_engine_rejected` SSB row; fallback only
- [ ] 10 new SsbActionType variants registered and emitted from the right sites
- [ ] grammar-cache disk persistence keyed by blake3(schema_canonical_json) — NEVER blake2/sha2
- [ ] torch.inference_mode() used in XGrammarLogitsProcessor (NEVER torch.no_grad per CLAUDE.md anti-stale)
- [ ] Decision doc at `workflow/reasoning/E-35-decisions.md` captures engine choice + DEFERRED list
- [ ] CLAUDE.md anti-disturbance: no `.unwrap()` introduced; existing LogitsProcessor chain (DoLa/ETC/steering) unchanged in ordering

## Tests to add

- `.elai_cc/python/tests/test_xgrammar_warmup.py`: assert all 8x8 schemas compile at boot; assert disk-cache hit on second boot; assert compile_count + cache_hits + cache_misses match.
- `.elai_cc/python/tests/test_tool_parser_variant.py`: assert Qwen2.5-Coder family routes to `<tools>`; assert Qwen2.5-7B-Instruct (non-Coder) routes to hermes; 3 model-config fixtures.
- `.elai_cc/crates/providers/tests/instructor_mode_smoke.rs`: synthetic prompt + schema; assert output structure is `<think>...</think><answer>{valid JSON}</answer>`; assert JSON parses against schema.
- `.elai_cc/crates/providers/tests/grammar_mode_smoke.rs`: enum-only schema; assert no `<think>` phase fires; assert mask applied immediately.
- `.elai_cc/crates/tools/tests/search_replace_matcher.rs`: 20 fixture diffs; assert tier-1 hit rate ≥ 95%; assert delimiter-collision triggers whole-file fallback.
- `.elai_cc/crates/tools/tests/codeact_sandbox.rs`: assert Python code executes; assert tool_allowlist enforced; assert 200-line cap rejects oversized actions; assert sandbox timeout works.
- `.elai_cc/crates/contracts/tests/cloud_strict_schema_contract.rs`: assert Anthropic / OpenAI / Gemini cloud requests carry strict schema, not json_object.
- `.elai_cc/crates/contracts/tests/structured_output_ssb_contract.rs`: SSB row schema validity for all 10 new variants.
- `.elai_cc/python/tests/test_grammar_cache_disk.py`: clear cache, boot, assert all 64 compile + write; clear partial, boot, assert partial hits + remaining compile.

## Files touched

- MODIFY: `.elai_cc/python/elai_inference/server.py` (+~280 LOC: XGrammarLogitsProcessor + compile_all_schemas_at_boot + warm_grammar_cache_from_disk + INSTRUCTOR/GRAMMAR mode dispatch)
- CREATE: `.elai_cc/python/elai_inference/tool_parser.py` (~140 LOC: ToolParserVariant detector + qwen_tools/hermes routing)
- CREATE: `.elai_cc/crates/tools/src/search_replace.rs` (~240 LOC: SearchReplaceMatcher + DelimiterCollisionGuard)
- CREATE: `.elai_cc/crates/tools/src/codeact_sandbox.rs` (~180 LOC: CodeActSandbox)
- MODIFY: `.elai_cc/crates/providers/src/runtime/sidecar.rs` (~120 LOC: Provider::generate_structured + StructuredMode passthrough)
- MODIFY: `.elai_cc/crates/providers/src/decoding.rs` (~80 LOC: StructuredMode enum + dispatch)
- MODIFY: `.elai_cc/crates/orchestrator/src/agent_role_dispatcher.rs` (~70 LOC: AgentRole::Runner variant + dispatch)
- MODIFY: `.elai_cc/crates/orchestrator/src/ipc_gateway.rs` (~60 LOC: CodeActAction variant)
- MODIFY: `.elai_cc/crates/orchestrator/src/dispatcher.rs` (~50 LOC: runner-role CodeAct dispatch)
- MODIFY: `.elai_cc/crates/orchestrator/src/grounding_client.rs` (~120 LOC: cloud strict-schema for Anthropic/OpenAI/Gemini + legacy json_object refusal)
- MODIFY: `.elai_cc/crates/tools/src/dispatcher.rs` (~30 LOC: SearchReplaceMatcher integration as tier-1)
- MODIFY: `.elai_cc/crates/orchestrator/src/ssb.rs` (~40 LOC: 10 new SsbActionType variants + as_sql_str entries)
- MODIFY: `.elai_cc/crates/dashboard-v2/src/lib/components/RuntimePanel.svelte` (~40 LOC: StructuredMode + grammar-cache status rows)
- CREATE: `workflow/reasoning/E-35-decisions.md` (~140 LOC)

## Citations / Grounding

- arXiv:2411.15100 (cited in source) — XGrammar MLSys 2025: <40us/token JSON, 100x speedup, default in vLLM/SGLang/TRT-LLM/MLC
- arXiv:2501.10868 (cited in source) — JSONSchemaBench: XGrammar 97.1% accurate, cache wins
- arXiv:2502.09061 (cited in source) — CRANE: strict constraint hurts reasoning; alternate unconstrained + constrained
- arXiv:2402.01030 (cited in source) — CodeAct ICML 2024: +20% success, ~30% fewer turns
- arXiv:2502.14905 (cited in source) — "Think Inside the JSON": GRPO + format-reward for two-tag protocol
- arXiv:2603.03305 (cited in source) — DCCD: +24pp GSM8K on 1B model
- arXiv:2510.12487 (cited in source) — Diff-XYZ JetBrains Oct 2025: search-replace > udiff for larger; small models gain little
- github.com/mlc-ai/xgrammar (cited in source) — Source repo, supports Qwen/Llama/DeepSeek/Phi
- github.com/guidance-ai/llguidance (cited in source) — Alternative; better TTFT on unique schemas
- github.com/hanXen/vllm-qwen2.5-coder-tool-parser (cited in source) — Qwen2.5-Coder `<tools>` parser (NOT hermes)
- aider.chat/docs/leaderboards/edit.html (cited in source) — Polyglot Nov-2025: GPT-5 88.0%; whole > diff for small models
- platform.claude.com/docs/en/build-with-claude/structured-outputs (cited in source) — Anthropic strict-mode beta header
- openai.com/index/introducing-structured-outputs-in-the-api (cited in source) — OpenAI strict mode 100% adherence
- ai.google.dev/gemini-api/docs/structured-output (cited in source) — Gemini responseJsonSchema Nov-2025
- developers.redhat.com/articles/2025/06/03/structured-outputs-vllm (cited in source) — vLLM 0.8+ guided decoding default XGrammar
- chatforest.com/reviews/sglang-structured-generation-llm-serving (cited in source) — SGLang+XGrammar-2 May 2026
- blog.squeezebits.com/guided-decoding-performance-vllm-sglang (cited in source) — XGrammar vs Guidance: cached wins
- arXiv:2605.02363 (cited in source) — "When Correct Isn't Usable": 10-30% reasoning drop under hard constraints

## Open questions / risks

- 64-grammar boot warmup time: source report Open Question 2. If > 1s wall-clock on cold boot, disk cache is MANDATORY (task 8); first-ever boot still pays the cost. Measure on the 12GB local AND the 24GB remote.
- INSTRUCTOR mode adds one `<think>` phase per request — latency hit not measured for ELAI's workload. Source report Open Question 1: GSM-style synthetic eval over 100 BabyStepSpec generations needed. Default-on for reasoning roles is grounded; reassess after measurement.
- CodeAct +20% claim is from arxiv:2402.01030 on 7B+ open-source models + GPT-3.5/4. ELAI's 7B-NF4 may not hit the same uplift. Source report Open Question 3 calls for local 100-task eval. Ship CodeAct as the runner-role DEFAULT but gate `--no-codeact` flag for A/B.
- SEARCH/REPLACE vs whole for Qwen2.5-Coder-7B on small files (<100 lines): source report Open Question 4 — the 7B cliff may cut the other way. Measure on 8 BabyStepSpec authoring tasks before promoting tier-1.
- XGrammar-2 + SGLang v0.5.11 (source report Open Question 5): 6x tool-calling compile-speed claim. Matters only if cache miss — with disk cache, gain is small. Stay on XGrammar-1 for E-35; upgrade if cache invalidation becomes common.
- Hermes vs qwen_tools envelope detection at boot: trust `model.config.name_or_path` substring match is fragile. Add explicit `ELAI_TOOL_PARSER_VARIANT` env override + log the resolved variant + SSB row for visibility.
- DCCD (task 9) DEFERRED behind env flag — measure first. Numeric block uplift on ELAI workloads is unknown (Open Question 1).
- Cloud strict-schema beta header (`structured-outputs-2025-11-13`) may expire or be promoted to stable — track Anthropic platform changelog.
- LogitsProcessor chain ordering: XGrammar inserts AFTER DoLa/ETC/steering. Verify this doesn't break existing CLAUDE.md "DoLa harmful on 7B (-15%)" disable-default behavior (DoLa is OFF by default; the chain still passes).
- E-18 sequencing: E-35 MUST land before E-18 since E-18's WIRING_CONTRACT references XGrammarLogitsProcessor + compile_all_schemas_at_boot + SearchReplaceMatcher + DelimiterCollisionGuard as if they exist. Coordinate timing.

## Next

- E-18 (PLANNED, tool-use reliability): RE-WRITE task 1 ("Wire XGrammar") to a thin call into E-35's primitives; task 2 ("Hermes envelope") becomes "use the right envelope per model variant via E-35's ToolParserVariant"; task 3 (SEARCH/REPLACE) becomes "wire E-35's SearchReplaceMatcher into the dispatcher tier order". Sequence: E-35 first, then E-18.
- E-19 (already authored, reasoning quality): consumes INSTRUCTOR mode primitive — sequence E-35 first.
- E-20 (already authored, local-cloud routing): consumes cloud strict-schema wiring — sequence E-35 first.
- B-20 (PLANNED, tool registry unification): consumes ToolParserVariant detection — sequence after E-35.
- B-28 (PLANNED, Focus Tunnel): STRUCTURED mode IS E-35's XGrammar GRAMMAR mode.
- E-35.B (deferred): DCCD-style draft-conditioned fallback enabled by default after Open Question 1 internal eval.
- E-35.C (deferred): XGrammar-2 + SGLang v0.5.11 upgrade (only if cache-miss latency becomes a problem).
- E-35.D (deferred): full CodeAct adoption for ALL composing-tool tasks (after Open Question 3 internal 7B eval confirms uplift).
