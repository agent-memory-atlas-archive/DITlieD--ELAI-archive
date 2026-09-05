E-35 structured-output substrate — decisions

Session: s-15320 (cluster s-25120 completion-drive session 2, plan 28 of 28 — LAST plan).
Plan: F:\.ELAI_workflow\docs\thefinalglance\E-plan\E-35_structured_output_2026.md
Prior recheck: workflow/sessions/s-25120/cross-check-recheck/E-35_structured_output_2026-report.md (PARTIAL: 5 implemented + 2 partial + 15 missing + 3 not_wired).

WHAT SHIPPED

1. 10 NEW SsbActionType variants (SCHEMA 140 -> 141, TOTAL_VARIANTS 404 -> 414): XgrammarMaskApplied, InstructorModeActivated, GrammarModeActivated, CodeactActionExecuted, SearchReplaceTier1Hit, SearchReplaceCollisionFallback, CloudStrictSchemaApplied, GrammarCacheLoaded, StructuredEngineRejected, LegacyJsonObjectRefused. Atomic 10-site lockstep: enum + as_sql_str + from_str + SsbPayload variant + payload->action_type + CHECK SQL + test_payload arm + all_test_payloads (338 -> 348) + SCHEMA_VERSION + action_type_lock.rs TOTAL_VARIANTS + ALL_VARIANTS + exhaustive_match_guard + golden file regen.

2. NEW: crates/orchestrator/src/codeact_sandbox.rs (~310 LOC) — Runner-role-FREE CodeAct sandbox dispatcher. Composes at the boundary: caller hands `execute_code_act` a typed `CodeActAction`; the dispatcher enforces line-count cap + tool-allowlist guard, then either dispatches to an injected `CodeActSandbox` impl OR returns `CodeActOutcome::SandboxAbsent` (RUNPOD-PENDING). Emits one `CodeactActionExecuted` SSB row per dispatch regardless of outcome. Includes pluggable `CodeActSandbox` trait (native async fn via Pin<Box<Future>> desugar per CLAUDE.md anti-stale), `CodeActOutcome` typed enum (4 labels), blake3-prefixed `action_hash` + `outcome_hash` helpers, 6 unit tests.

3. EXTENDED: crates/orchestrator/src/grounding_client.rs — added `CloudGroundingClient::mark_strict_schema_applied(provider_label, schema_id, mode_label, now_ms)` method. Emits `CloudStrictSchemaApplied` SSB on strict-schema activation; refuses + emits `LegacyJsonObjectRefused` SSB on legacy vendor json_object mode (refuse loud per plan L67, NEVER silent fallback). Added `e35_blake3_u64_prefix` helper (blake3-derived u64 for privacy-preserving SSB labels).

4. NEW: crates/cli/src/structured_output_cmd.rs (~430 LOC) — `elai structured-output <sub>` CLI subverb tree. Subverbs: `status`, `codeact-run`, `strict-mode`, `grammar emit-test`, `search-replace check`, `reject-engine`. Each subverb has at least one production SSB emission path — WIRE-DARK avoidance per E40_WIRE_DARK_AVOIDANCE_VIA_CLI for the +10 atomic variant bump. Wired into args.rs (`CliAction::StructuredOutput` + `StructuredOutputAction` enum + `parse_structured_output_subcommand_args`) + main.rs dispatch arm + `mod structured_output_cmd;`.

5. NEW: crates/contracts/tests/e35_structured_output_contract.rs (~17 KB) — round-trip tests: 10 SsbActionType wire-tag round-trip via from_str/as_sql_str; 10 distinct discriminant assertion; 10 payload->action_type discriminator round-trip; SidecarXGrammarHook canned-mock smoke (already-shipped substrate); CodeAct sandbox sandbox_absent + cap_exceeded + mocked-executed coverage; DelimiterCollisionGuard token-collision smoke; well-known TOOL_XGRAMMAR_INFER + TOOL_CODEACT_EXEC stability; CODEACT_MAX_LINES paper-cap pin; CloudStrictSchemaApplied + LegacyJsonObjectRefused msgpack round-trip; CodeActOutcome label set closed.

KEY DECISIONS

DECISION-1 (lean-scope per autonomous-mode brief): Of the 15 missing items + 3 not_wired items in the prior recheck, this session closes 10-12 via the 10 SSB variants + producer wiring + codeact_sandbox scaffolding. Remaining items (Python sidecar XGrammarLogitsProcessor + tool_parser.py + Provider::generate_structured trait method + dispatcher SearchReplaceMatcher tier-0 wire + DCCD env-flag path + RuntimePanel structured-mode rows) deferred to E-35.D patch-N follow-on per session-2-handoff lean-scope cadence. The 10 SSB variants are the high-leverage close: every downstream wiring (sidecar producer, cloud strict-schema marker, dispatcher tier-1 hook, codeact emit, lm-format-enforcer refusal) lands against a stable wire contract this session pins.

DECISION-2 (AgentRole::Runner DEFERRED per E41_AGENTROLE_FROZEN_AT_17): Adding AgentRole::Runner triggers a ~230 match-arm cascade across orchestrator/src/agent_pool.rs (~16 sites), agent_role_dispatcher.rs, dashboard-v2/src/lib/api/types.ts RuntimeAgentRole mirror, every per-role config emitter, etc. The E41 plan explicitly chose to freeze AgentRole at 17 variants and route new capabilities through existing role + RuntimeBackend composition (E40_RUNTIMEBACKEND_VS_AGENTROLE pattern). E-35 follows that precedent: CodeAct dispatches through `codeact_sandbox::execute_code_act` directly from any role rather than carving a new Runner role. Full CodeAct adoption (Runner role + match-arm fan-out) is deferred to E-35.D patch-N. Test exhaustiveness is preserved through the existing role enum.

DECISION-3 (Python sidecar wiring DEFERRED — RUNPOD-PENDING): python/elai_inference/server.py XGrammarLogitsProcessor + compile_all_schemas_at_boot + warm_grammar_cache_from_disk + python/elai_inference/tool_parser.py are not in scope this session. The Rust-side contract is already pinned via SidecarXGrammarHook trait + XGrammarRequest/Response + xgrammar_cache_key. Once the sidecar lands (requires boot on remote GPU box per workflow/remote-setup/REMOTE-SIDECAR.md), the host-side `GrammarCacheLoaded` SSB row source flips from the CLI smoke (`elai structured-output grammar emit-test`) to the real sidecar boot completion handler — wire shape is stable per the SSB beacon contract this session pins.

DECISION-4 (Provider::generate_structured trait method DEFERRED): The Provider trait (crates/providers/src/runtime/sidecar.rs) is implemented across a BTreeMap of provider impls. Adding a new trait method is additive but extends every impl simultaneously — HIGH risk per cross-provider drift. The SidecarXGrammarHook trait (already shipped substrate at sidecar_xgrammar_hook.rs) is the canonical injection point for sidecar-XGrammar dispatch; once HttpSidecarXGrammarHook production impl lands in the sidecar session, callers compose against the hook trait, not the Provider trait. Provider::generate_structured trait method DEFERRED to a follow-on plan that owns the Provider surface modification.

DECISION-5 (AnthropicClient anthropic-beta header propagation RUNPOD-PENDING): The Anthropic strict-schema mode requires `anthropic-beta: structured-outputs-2025-11-13` header on outbound requests. The existing AnthropicClient (crates/providers/src/providers/anthropic.rs) hard-codes a single `anthropic-beta: OAUTH_BETA_HEADER` value at lines 594 + 698. Adding a configurable beta-header LIST requires AnthropicClient surface modification (out-of-scope for E-35; the provider crate is a shared substrate). This session's `mark_strict_schema_applied` marker emits the SSB beacon recording INTENT to apply strict mode; transport-level propagation lands in a provider-crate patch-N. Per plan L67 + cite-discipline cluster #13, "When Correct Isn't Usable" (name-only ref — arXiv DEFERRED to mistakes.md) documents the 10-30% quality drop from legacy json_object — refusing it loud is the immediate ship; transport propagation is the follow-on.

DECISION-6 (SearchReplaceMatcher dispatcher tier-0 wire DEFERRED to patch-N): The matcher substrate is already shipped at crates/tools/src/search_replace.rs (460+ LOC with 10+ in-file tests + a separate smoke suite at crates/tools/tests/e35_search_replace_smoke.rs). The dispatcher integration into crates/tools/src/dispatcher.rs::try_edit_tiers (insert as NEW tier-0 before existing 3-tier exact/whitespace-normalized/line-anchored loop) is a single-file follow-on per the recheck NOT_WIRED list. This session's CLI subverb `elai structured-output search-replace check` exercises DelimiterCollisionGuard end-to-end (production caller); the dispatcher wire-up lands in patch-N to keep this session's blast radius bounded.

DECISION-7 (DCCD Task 9 DEFERRED): The plan explicitly gates DCCD behind ELAI_DCCD_ENABLED=1 env per plan L65 / Open Q1. The +24pp GSM8K claim from arXiv:2603.03305 is DEFERRED per cite-discipline cluster #13 (future-year placeholder — treat as UNGROUNDED per E-37 cluster cadence). No production code path opts into DCCD this session.

DECISION-8 (frontend_pair as-shipped drift): The plan claims frontend_pair = crates/dashboard-v2/src/lib/components/RuntimePanel.svelte. The as-shipped surface is GrammarCompilePanel.svelte + commands_e35::get_grammar_compilations Tauri command (registered in lib.rs invoke_handler) — already mounted via DrawerHost drawerKey='grammar-compile'. RuntimePanel.svelte stays untouched this session. The placeholder Tauri command's wire shape is stable per commands_e35.rs file header "the wire shape is stable so this panel does NOT change when the producer ships."

DECISION-9 (cite-discipline cluster #13): Inline cite ONLY codeact-2402.01030 (Wang et al., "Executable Code Actions Elicit Better LLM Agents", ICML 2024) — ingested at workflow/papers/codeact-2402.01030-interface.md. The 200-LINE CODEACT_MAX_LINES const is the conservative byte-equivalent of the paper's 200-token Sec 4.2 MaxAction knob. The 7 OTHER plan citations (XGrammar arXiv:2411.15100 / JSONSchemaBench arXiv:2501.10868 / CRANE arXiv:2502.09061 / "Think Inside the JSON" arXiv:2502.14905 / DCCD arXiv:2603.03305 / Diff-XYZ arXiv:2510.12487 / "When Correct Isn't Usable" arXiv:2605.02363) referenced BY NAME ONLY in code/docs; arXiv IDs DEFERRED to workflow/sessions/s-25120/completion-drive/mistakes.md cite-incident cluster #13 (E-35-INGEST-pending). Per E-37 cluster precedent: future-year placeholders (2603.* / 2605.*) treated as UNGROUNDED until WebSearch verifies them.

DECISION-10 (NO new dependencies): All wiring composes on existing workspace crates (blake3 + tokio + tracing already-deps of orchestrator; rmp_serde + serde_json for msgpack; elai-tools for SearchReplaceMatcher / DelimiterCollisionGuard; elai-orchestrator re-exports Ssb / SsbConfig / SsbPayload). CodeActAction + ToolId from elai-ipc-types are re-exported through orchestrator::codeact_sandbox so crates/cli does NOT take a new elai-ipc-types dep.

EXPECTED TEST OUTCOME

- `cargo check -p elai-orchestrator` — clean (10-site SSB lockstep + codeact_sandbox module + grounding_client extension).
- `cargo check -p elai-cli` — clean apart from pre-existing main.rs RunArgs.empire E0027/E0063 (session 3 territory per session-2-handoff KNOWN-RESIDUAL #1; my CliAction::StructuredOutput arm is disjoint from RunArgs).
- `cargo check -p elai_cc_contracts --test e35_structured_output_contract` — clean.
- `cargo check --workspace --tests` — clean modulo the same RunArgs pre-existing error.
- No new test-runner invocation (cargo test) per executor brief constraint.

UNKNOWNS

- Live grammar warmup latency on the 12GB local pod (plan L29 expects <5s for 64-grammar boot) — RUNPOD-PENDING.
- Live grammar warmup latency on the 24GB remote pod (plan L29 expects <30s) — RUNPOD-PENDING.
- Python sidecar tool_parser.py end-to-end correctness against Qwen2.5-Coder-{3,7,32}B model.config.name_or_path strings — RUNPOD-PENDING (sidecar boot required).
- Anthropic anthropic-beta header propagation latency / quality delta vs free-form JSON — RUNPOD-PENDING (provider-crate extension required).
- 64-grammar disk cache hit rate on second boot (plan Open Q on disk persistence) — RUNPOD-PENDING (the disk-write path lives in the deferred Python sidecar code).

CHANGE MANIFEST (for memory-writer)

ADDED:
  - crates/orchestrator/src/codeact_sandbox.rs (NEW, ~310 LOC)
  - crates/cli/src/structured_output_cmd.rs (NEW, ~430 LOC)
  - crates/contracts/tests/e35_structured_output_contract.rs (NEW, ~17 KB)
  - workflow/reasoning/E-35-decisions.md (NEW, this file)

MODIFIED:
  - crates/orchestrator/src/ssb.rs (10 enum variants + 10 as_sql_str arms + 10 from_str arms + 10 SsbPayload variants + 10 payload->action_type arms + 10 CHECK SQL entries + 10 payload structs + 10 test_payload arms + SCHEMA 140 -> 141 + 10 all_test_payloads entries + payloads.len() assertion 338 -> 348)
  - crates/orchestrator/tests/action_type_lock.rs (TOTAL_VARIANTS 404 -> 414 + 10 ALL_VARIANTS entries + 10 exhaustive_match_guard arms)
  - crates/orchestrator/tests/action_type_lock.golden (regenerated alphabetically with 10 new entries inserted at correct ordinal positions)
  - crates/orchestrator/src/lib.rs (+ pub mod codeact_sandbox)
  - crates/orchestrator/src/grounding_client.rs (+ CloudGroundingClient::mark_strict_schema_applied + e35_blake3_u64_prefix helper + SsbPayload use)
  - crates/cli/src/args.rs (+ CliAction::StructuredOutput + StructuredOutputAction enum + parse_structured_output_subcommand_args)
  - crates/cli/src/main.rs (+ mod structured_output_cmd + CliAction::StructuredOutput dispatch arm)

DEFERRED TO E-35.D PATCH-N OR FOLLOW-ON SESSION:
  - python/elai_inference/server.py XGrammarLogitsProcessor + compile_all_schemas_at_boot + warm_grammar_cache_from_disk (RUNPOD-PENDING)
  - python/elai_inference/tool_parser.py (RUNPOD-PENDING)
  - Provider::generate_structured trait method on sidecar.rs (Provider surface modification)
  - AnthropicClient anthropic-beta header propagation (provider-crate extension)
  - SearchReplaceMatcher dispatcher tier-0 wire (single-file follow-on)
  - AgentRole::Runner variant (E41_AGENTROLE_FROZEN_AT_17 — 230 match-arm cascade)
  - DCCD env-flag-gated path (plan Task 9, ELAI_DCCD_ENABLED=1)
  - XGrammar-2 / SGLang 0.5.11 upgrade (out-of-scope)
  - RuntimePanel.svelte structured-mode + grammar-cache-status rows (GrammarCompilePanel is the as-shipped surface)
