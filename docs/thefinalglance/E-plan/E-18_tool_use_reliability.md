# E-18: Tool-use reliability — XGrammar + Hermes envelope + SEARCH/REPLACE + one-shot retry
Objective: harden ELAI's tool-call path against the seven documented failure modes that the sweep identified — silent ```json fallback for Qwen2.5-Coder, JSON-in-JSON escape bugs, unbounded retry loops, unified-diff disorientation on small open models, hallucinated package names, and long-context tool-response degradation. Five concrete changes: XGrammar as Tier 2 backend, Hermes `<tool_call>` envelope enforced by grammar mask, SEARCH/REPLACE edit-format with custom fences and delimiter-collision pre-check, one-shot retry with error-embedded reprompt, and ToolResult spill-to-disk for long responses.
Estimated effort: 2-3 sessions, ~10-14 h
Pre-requisites: B-7 SHIPPED via B-7b, B-37 SHIPPED (RuntimeAdapter), B-20 PLANNED (tool registry unification — E-18 ships under B-20's frame), C-13 PLANNED (IPC dispatcher production swap).
Parallel-safe with: E-16, E-17, E-19, E-20.
Source report: workflow/improvement-sweep-2026-05-19/reports/18-tool-use-reliability.md

## Context

ELAI's local sidecar runs Qwen2.5-Coder-7B-NF4. The Aider edit-format leaderboard (apr 2025) shows Qwen2.5-Coder-7B at 57.9% pass on `whole` and is unranked on `diff` — small open models lose track of unified-diff context lines (arxiv:2510.12487 Diff-XYZ Oct 2025). The Qwen2.5-Coder-tool-parser repo (github.com/hanXen) explicitly documents a silent ```json fallback failure mode that bypasses ELAI's current parser. Constrained-decoding measurement (arxiv:2411.15100 XGrammar MLSys 2025, arxiv:2501.10868 JSONSchemaBench) puts XGrammar at <40us/token JSON and 97.1% schema-accurate on nested GitHub-issue schemas vs Outlines 76.4% — Outlines compile-times 40s-10min on complex schemas knock it out of the latency budget. XGrammar is the default in vLLM/SGLang/TensorRT-LLM as of March 2026.

For recovery: arxiv:2509.18847 Structured Reflection (2025) measures Claude-Sonnet-4 >50% recovery vs Qwen3-8B ~20% — blind retry with same prompt is the documented anti-pattern that ELAI matches today. Long-context tool-response degradation (arxiv:2505.10570 LongFuncEval) is 7-91% as response length grows; spill-to-disk + reference is the fix. CodeAct (arxiv:2402.01030 ICML 2024) shows +20% success across 17 LLMs by letting the model compose multiple tools in one Python action — that lift is for a later wave (deferred to E-18.B).

Cited grounding: arxiv:2411.15100, arxiv:2501.10868, arxiv:2510.12487, arxiv:2509.18847, arxiv:2505.10570, arxiv:2402.01030, arxiv:2406.10279 (USENIX Sec 2025 package hallucination), aider.chat/docs/leaderboards/edit.html, github.com/hanXen/vllm-qwen2.5-coder-tool-parser, qwen.readthedocs.io function_call docs.

## Existing state (B/C/D coverage)

- B-7 (DONE via B-7b): producer wiring for the pipeline factory + WebResearchStage emitter. The tool-call IPC types exist but the Qwen2.5-Coder tool-parser glitch is unmitigated.
- B-20 (PLANNED): tool registry unification. E-18 lands under B-20's umbrella — XGrammar + Hermes envelope are necessary preconditions before the unified registry can claim format-correctness guarantees.
- B-37 (SHIPPED fd5c2e5): RuntimeAdapter trait + sidecar/vllm runtimes; the XGrammar LogitsProcessor wires into the sidecar runtime's `complete`/`stream_step` paths at `crates/providers/src/runtime/sidecar.rs`.
- C-13 (PLANNED): IPC dispatcher production swap. E-18 augments the ToolRequest path so when C-13 lands, the dispatcher is already format-safe.
- D-13 Adversary role (PLANNED): cross-family verifier. E-18 does NOT pre-empt D-13; the one-shot retry recovery is intra-model, not cross-family.

## Reads (mandatory before coding)

- workflow/improvement-sweep-2026-05-19/reports/18-tool-use-reliability.md (full)
- F:\.ELAI_workflow\.elai_cc\python\elai_inference\server.py (sidecar Python; XGrammar LogitsProcessor wires here)
- F:\.ELAI_workflow\.elai_cc\crates\providers\src\runtime\sidecar.rs (complete/stream_step entry points; lines 209-380)
- F:\.ELAI_workflow\.elai_cc\crates\tools\src\dispatcher.rs (edit/bash dispatch; lines 776-1003 hold ToolDispatchError variants and CommitOutcome path; SEARCH/REPLACE fence pre-check goes alongside)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\dispatcher.rs (state-context-aware dispatch; lines 83-342)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ipc_gateway.rs (IPC types incl. InferRequest/ToolRequest/ToolResult)
- F:\.ELAI_workflow\.elai_cc\crates\contracts\tests\agent_loop_contract.rs (existing InferRequest contract for grounding the envelope schema)
- arxiv:2411.15100 + arxiv:2501.10868 + arxiv:2510.12487 + arxiv:2509.18847 PDFs (cite in tests + decision doc)
- github.com/hanXen/vllm-qwen2.5-coder-tool-parser (the documented ```json fallback bug)
- qwen.readthedocs.io/en/latest/framework/function_call.html (Hermes envelope spec)

## Tasks

1. Wire XGrammar as the Tier 2 structured-output backend at the sidecar. In `python/elai_inference/server.py`, replace any in-house JSON-mode/regex constrainer with `xgrammar.GrammarMatcher` + transformers `LogitsProcessor`. Pre-compile all 15 IPC schemas + the tool-schema set at sidecar boot (cold-schema warmup) to dodge the JSONSchemaBench-reported CPU-mask-gen stalls. Surface schema-compile failures as a startup error, not silent fallback. Cite arxiv:2411.15100.

2. Enforce the Hermes `<tool_call>...</tool_call>` envelope at the grammar level. Qwen2.5-Coder chat template is Hermes-compatible (qwen.readthedocs.io); reject candidates that don't emit the envelope at logit-mask time. The known silent ```json fallback (github.com/hanXen) becomes impossible because the grammar mask never allows ``` outside the envelope. Add a sidecar smoke test that fuzz-feeds 100 tool prompts and asserts 100% envelope presence on Qwen2.5-Coder-7B-NF4.

3. Rewrite the Edit tool's 3-tier matcher to SEARCH/REPLACE format with custom fences. Replace tier 1 substring-match with explicit-fence block-match using `<<< ELAI_SEARCH` / `=== ELAI_REPLACE` / `>>> ELAI_END` tokens that aren't valid Rust/Python/Svelte syntax. ADD a delimiter-collision pre-check: scan source for fence tokens; if present (or `<<<<<<< ` / `=======` / `>>>>>>> ` git-conflict markers), fall back to whole-file rewrite path. Cite arxiv:2510.12487 + Aider leaderboard ablation showing Qwen2.5-Coder-7B `whole` 57.9% pass / 100% format > `diff` for this model class.

4. Add one-shot retry with error-embedded reprompt at the orchestrator. When Tier 2 validation fails OR the SEARCH/REPLACE matcher misses, re-prompt ONCE with the structured error as a `<tool_error>...</tool_error>` system message. After that, escalate to fixer agent (existing ELAI agent at `.claude/agents/fixer.md`). NEVER retry with the same prompt. Cite arxiv:2509.18847 — the Claude-Sonnet-4 50% vs Qwen3-8B 20% recovery gap motivates the cap. Add `tool_retry_exhausted` and `tool_one_shot_recovered` SSB rows.

5. Add ToolResult spill-to-disk for long responses. Inline payload cap: 8K tokens. Overflow spills to `.elai_cc/sessions/{id}/spill/{uuid}.txt`; ToolResult carries a `spill_ref` field instead of the full body. Cite arxiv:2505.10570 LongFuncEval; the 7-91% degradation curve motivates the cap. Update the ToolResult IPC variant; add a `tool_result_spilled` SSB row.

6. Add pre-execution registry verification for package-touching Edit/Bash calls. When the agent edits `Cargo.toml` / `pyproject.toml` / `package.json`, the dispatcher calls crates.io / pypi / npm registries to verify package name + last-published date ≥ 30 days. Reject unknown packages or packages newer than 30 days at dispatch time. Cite arxiv:2406.10279 USENIX Sec 2025: 19.7% baseline hallucination rate, 58% repeatable. CLAUDE.md anti-slopsquatting is already a HARD RULE; this task wires the enforcement into the dispatcher. Emit `package_ref_verified` (B-7 already has the variant) or `package_ref_rejected` new variant.

7. Add path-allowlist pre-filter to the tool schema enum. The sidecar receives the agent's path-allowlist as part of GroundingRequest and pre-filters the tool schema's `path: enum [...]` field at grammar-mask time. Hallucinated paths can't be emitted because the mask doesn't allow them. Surface mismatch as `path_allowlist_pre_filtered` SSB row (vs the current "visibility_filtered" error which leaks information). Cite The Elder Scripts pattern; the self-correction review showed visibility_filtered ≠ NotFound is the documented confusion mode.

8. Add a PARL-MT-style ProgressMarker IPC frame emitted after each ToolResult. The orchestrator dedupes (tool, args-hash) within a 5-frame window to kill duplicate fires under long multi-turn horizons. Cite arxiv:2509.23206 + arxiv:2511.01527 — both document re-invocation under long-horizon contexts. Add `progress_marker_emitted` and `duplicate_tool_call_deduped` SSB rows.

9. Defer CodeAct execution channel (arxiv:2402.01030, +20% success) to a follow-up E-18.B. The lift is large but the integration surface (Python sandbox + ELAI tool composition + Tier 3 unit-test validation) is too large for this E-N. Document the deferral and cite the open question Q4 from the source report (does CodeAct generalize to 7B class).

10. Defer Cap'n Proto / postcard IPC hot-path migration to E-18.C (sweep recommendation #7). End-to-end IPC latency over a TCP-tunneled WSL2 link may not see the zero-copy win; needs measurement first (report Q5).

## WIRING_CONTRACT

produces_symbols:
  - python/elai_inference/server.py::XGrammarLogitsProcessor (new class)
  - python/elai_inference/server.py::compile_all_schemas_at_boot (new fn)
  - tools::dispatcher::SearchReplaceMatcher (new struct replacing tier-1 substring)
  - tools::dispatcher::DelimiterCollisionGuard (new fn)
  - orchestrator::dispatcher::OneShotRetry (new struct)
  - orchestrator::dispatcher::ToolErrorReprompt (new struct)
  - tools::package_verify::registry_check (new fn dispatching npm/pypi/crates)
  - ipc_gateway::ToolResult::spill_ref (new field on existing variant)
  - ipc_gateway::ProgressMarker (new IPC variant)
  - SsbActionType::ToolRetryExhausted (new)
  - SsbActionType::ToolOneShotRecovered (new)
  - SsbActionType::ToolResultSpilled (new)
  - SsbActionType::PackageRefRejected (new)
  - SsbActionType::PathAllowlistPreFiltered (new)
  - SsbActionType::ProgressMarkerEmitted (new)
  - SsbActionType::DuplicateToolCallDeduped (new)
  - SsbActionType::XgrammarSchemaCompileFailed (new)
consumes_symbols:
  - providers::runtime::sidecar::complete (existing; XGrammar processor wires into the request body)
  - tools::dispatcher::dispatch_edit (existing; SEARCH/REPLACE matcher replaces substring tier)
  - orchestrator::dispatcher::dispatch_tool (existing; one-shot retry wraps it)
  - orchestrator::ipc_gateway::send (existing; ToolResult spill writes happen here)
reachable_from:
  - crates/orchestrator/src/task_runner.rs::tool_call_phase
  - crates/orchestrator/src/agent_role_dispatcher.rs::dispatch
  - python/elai_inference/server.py::__main__ (sidecar boot)
stub_ban:
  - no silent ```json fallback path after task 2 (grammar mask rejects)
  - no retry-with-same-prompt loop after task 4
  - no inline ToolResult body > 8K tokens after task 5
  - no Edit dispatch on package-touching files without registry-check after task 6 (rejected at dispatch)
  - no `visibility_filtered` error variant return (replaced by NotFound + path_allowlist_pre_filtered SSB)
  - CLAUDE.md anti-unwrap: no `.unwrap()` in production tool paths
  - CLAUDE.md anti-stale: blake3 for args-hash (NEVER blake2/sha2)
frontend_pair: crates/dashboard-v2/src/lib/components/AgentDrawer.svelte (tool-call surface — extend with retry-exhausted + spill counters)
observable_effect:
  trigger: "elai /task --mode local 'add a tokio dependency named tokyo-async-future-v3 and call its run() function'"
  expect_ssb: "package_ref_rejected for tokyo-async-future-v3 (hallucinated name); tool_retry_exhausted absent because we never tried to install"
  expect_artifact: "no Cargo.toml modification; clean rollback; error_chain present in SSB"

## DETERMINISTIC_CHECK

```bash
# 1. XGrammar processor wired at sidecar boot
grep -n "XGrammarLogitsProcessor\|xgrammar.GrammarMatcher\|compile_all_schemas_at_boot" .elai_cc/python/elai_inference/server.py
# Expected: ≥3 hits (class def + matcher init + boot-time compile loop).

# 2. Hermes envelope grammar mask in place
grep -rn "tool_call>" .elai_cc/python/elai_inference/
# Expected: opening + closing tag references in the grammar.

# 3. SEARCH/REPLACE matcher with custom fences
rg "SearchReplaceMatcher|ELAI_SEARCH|ELAI_REPLACE|ELAI_END" .elai_cc/crates/tools --type rust
# Expected: ≥4 hits (struct def + 3 fence token uses).

# 4. Delimiter collision guard
rg "DelimiterCollisionGuard|fence_token_present_in_source" .elai_cc/crates/tools --type rust
# Expected: ≥2 hits.

# 5. One-shot retry implemented
rg "OneShotRetry|ToolErrorReprompt|tool_retry_exhausted" .elai_cc/crates/orchestrator --type rust
# Expected: ≥3 hits + 1 SSB variant arm.

# 6. ToolResult spill capacity wired
rg "spill_ref|TOOL_RESULT_INLINE_CAP|spill_to_disk" .elai_cc/crates/orchestrator --type rust
# Expected: const + dispatch + read-back path.

# 7. Registry verification at dispatch
rg "registry_check|package_ref_rejected" .elai_cc/crates/tools --type rust
# Expected: ≥2 hits.

# 8. Smoke test: hallucinated package rejected
elai /task --mode local "add a tokio dependency named tokyo-async-future-v3 and call its run() function"
sqlite3 .elai_cc/.elai_cc/ssb.db "SELECT action_type FROM ssb_events WHERE action_type='package_ref_rejected';"
# Expected: ≥1 row.
```

## ACCEPTANCE

- [ ] XGrammar LogitsProcessor wired at sidecar boot; all 15 IPC schemas + tool schemas pre-compiled (cold-schema warmup)
- [ ] Hermes `<tool_call>` envelope enforced at grammar-mask level (fuzz 100 prompts → 100% envelope presence on Qwen2.5-Coder-7B-NF4)
- [ ] SEARCH/REPLACE matcher with custom fences (`<<< ELAI_SEARCH` / `=== ELAI_REPLACE` / `>>> ELAI_END`) is tier-1 of the Edit matcher
- [ ] Delimiter-collision guard scans source for fence tokens + git-conflict markers; falls back to whole-file rewrite
- [ ] One-shot retry with error-embedded reprompt; second failure escalates to fixer (never retry-with-same-prompt)
- [ ] ToolResult inline cap 8K tokens; overflow spills to `.elai_cc/sessions/{id}/spill/{uuid}.txt`
- [ ] Pre-execution registry verification for Edit/Bash touching Cargo.toml / pyproject.toml / package.json (rejects unknown or <30-day-old packages)
- [ ] Path-allowlist pre-filter at grammar-mask level; `visibility_filtered` error variant retired
- [ ] PARL-MT ProgressMarker emitted after each ToolResult; (tool, args-hash) deduped within 5-frame window
- [ ] CLAUDE.md anti-slopsquatting (existing HARD RULE) now enforced at dispatch, not just at code-review
- [ ] Qwen2.5-Coder-7B-NF4 multi-turn smoke score logged; target ≥ 0.50 on a 50-prompt internal eval (report Q1)

## Tests to add

- `.elai_cc/crates/contracts/tests/xgrammar_envelope_contract.rs`: fuzz 100 prompts, assert envelope on every output; assert schema-compile errors surface, not silent.
- `.elai_cc/crates/tools/tests/search_replace_matcher.rs`: 20 fixture diffs; assert tier-1 hit rate ≥ 95%; delimiter-collision triggers whole-file fallback.
- `.elai_cc/crates/orchestrator/tests/one_shot_retry_contract.rs`: synthetic failed tool call, assert exactly 1 retry, second failure → fixer escalation; assert never retries with same prompt.
- `.elai_cc/crates/orchestrator/tests/tool_result_spill_contract.rs`: synthesize 50K-token result, assert spill file written, assert reference returned in IPC.
- `.elai_cc/crates/tools/tests/package_registry_verify.rs`: known-good (tokio), known-bad (tokyo-async-future-v3 hallucination), <30-day-old (synthetic new package) — three cases.
- `.elai_cc/crates/contracts/tests/path_allowlist_pre_filter_contract.rs`: allowlist=[a, b]; assert grammar mask rejects path=c at token level.
- `.elai_cc/python/tests/test_xgrammar_warmup.py`: assert all 15 IPC schemas + tool schemas compile at boot; no runtime compile-stall.

## Files touched

- MODIFY: `.elai_cc/python/elai_inference/server.py` (+~250 LOC for XGrammar wiring, envelope grammar, schema warmup)
- MODIFY: `.elai_cc/crates/providers/src/runtime/sidecar.rs` (+~40 LOC pass grammar config through complete/stream_step)
- CREATE: `.elai_cc/crates/tools/src/search_replace.rs` (~240 LOC SEARCH/REPLACE matcher + delimiter guard)
- MODIFY: `.elai_cc/crates/tools/src/dispatcher.rs` (+~80 LOC tier-1 swap + registry hook)
- CREATE: `.elai_cc/crates/tools/src/package_verify.rs` (~180 LOC npm/pypi/crates registry check)
- CREATE: `.elai_cc/crates/orchestrator/src/one_shot_retry.rs` (~140 LOC retry wrapper + escalation)
- MODIFY: `.elai_cc/crates/orchestrator/src/dispatcher.rs` (+~50 LOC retry integration)
- MODIFY: `.elai_cc/crates/orchestrator/src/ipc_gateway.rs` (+~60 LOC ToolResult::spill_ref + ProgressMarker variant)
- MODIFY: `.elai_cc/crates/orchestrator/src/ssb.rs` (+8 SsbActionType variants)
- MODIFY: `.elai_cc/crates/dashboard-v2/src/lib/components/AgentDrawer.svelte` (+~40 LOC counters)
- CREATE: `workflow/reasoning/E-18-decisions.md`

## Citations / Grounding

- arXiv:2411.15100 (cited in source) — XGrammar MLSys 2025: <40us/token JSON, 100x speedup vs FSM
- arXiv:2501.10868 (cited in source) — JSONSchemaBench v3 2025: XGrammar 97.1% accuracy on Qwen-2.5-32B nested schemas
- arXiv:2510.12487 (cited in source) — Diff-XYZ JetBrains Oct 2025: search-replace > udiff for larger models; small open models gain little
- arXiv:2509.18847 (cited in source) — Structured Reflection 2025: Claude-Sonnet-4 >50% recovery vs Qwen3-8B ~20%
- arXiv:2505.10570 (cited in source) — LongFuncEval: 7-91% degradation as tool-response length grows
- arXiv:2406.10279 (cited in source) — USENIX Sec 2025 package hallucination: 19.7% baseline, 58% repeatable
- arXiv:2509.23206 + arXiv:2511.01527 (cited in source) — PARL-MT + TPS-Bench long-horizon tool-use
- aider.chat/docs/leaderboards/edit.html (cited in source) — Qwen2.5-Coder-7B `whole` 57.9% pass / 100% format
- github.com/hanXen/vllm-qwen2.5-coder-tool-parser (cited in source) — silent ```json fallback documentation
- qwen.readthedocs.io/en/latest/framework/function_call.html — Hermes `<tool_call>` envelope spec

## Open questions / risks

- XGrammar CPU mask-gen stall on cold schemas (JSONSchemaBench v3 erratic-behaviour note): warmup at boot is mandatory; if any schema fails to pre-compile, sidecar refuses to start (loud failure > silent fallback). Risk: ELAI_LOCAL_MAX_CONTEXT=25000 + 15 schemas may push boot time > 30s; profile and consider lazy-compile per-schema with first-use latency budget instead.
- SEARCH/REPLACE custom fences are chosen to avoid Rust/Python/Svelte syntax + git-conflict tokens; if a future language has e.g. `<<<` as a real operator, the guard must extend. Track in decision doc.
- Pre-execution registry check adds 100-500ms network call per package-touching dispatch; cache aggressively. crates.io has good ETag support; npm rate-limits without auth.
- 30-day-old package threshold is conservative; legitimate brand-new packages may be blocked. Add an explicit `--allow-fresh-package` flag with audit logging.
- Qwen2.5-Coder-7B-NF4 doesn't have a public BFCL-v3 number; the 0.50 multi-turn target (report Q1) is empirical. Measure first, then ratchet.
- CodeAct +20% deferred to E-18.B — risk is that 7B-class generalization wasn't isolated in arxiv:2402.01030; needs ablation before committing the larger lift.
- Cap'n Proto vs postcard IPC migration deferred to E-18.C — measure first per report Q5; the network may dominate over zero-copy gains on tunneled WSL2.

## Next

- E-18.B (deferred): CodeAct execution channel + Python sandbox + Tier 3 unit-test validation.
- E-18.C (deferred): Cap'n Proto / postcard IPC migration on hot-path types (InferRequest/InferResponse/ThinkStep).
- B-20 tool registry unification consumes E-18's primitives; sequence B-20 after E-18 main ships.
- C-13 IPC dispatcher production swap consumes the ProgressMarker variant + spill_ref; sequence C-13 after E-18.
- D-13 Adversary role (cross-family verifier) reuses the one-shot-retry primitive; sequence D-13 after E-18.
- Erlang-OTP-style supervisor tree for agent pool (sweep recommendation #9) is a future E-N (rest-for-one for fixer-after-auditor); deferred until B-23-class agent-pool work resumes.
