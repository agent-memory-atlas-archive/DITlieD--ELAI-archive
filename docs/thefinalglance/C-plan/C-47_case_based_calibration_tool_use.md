# C-47: Case-Based Calibration of Tool-Use Telemetry

Objective: ship a per-tool telemetry layer that records (call, response,
post-call FSM state) tuples, builds a case base of known-good and
known-bad invocations, and consults it before future calls to calibrate
parameter shape. Direct port of arXiv:2605.15041's case-based reasoning
pattern for LLM tool use.

Pre-requisites: C-35 (Tool Attention) SHIPPED; C-42 (zero-trust fuzzer)
SHIPPED. Phase 4K MCP bridge SHIPPED.
Parallel-safe with: C-36..C-46 (except C-35, C-42 — sequence after).

Paper grounding (arXiv:2605.15041, 2026-05-16, verified abs page 2026-05-17):
"Case-Based Calibration of Adaptive Reasoning and Execution for LLM Tool Use."
Per verifier: case-based reasoning over historical tool-use traces measurably
improves call-shape correctness on repeated tool patterns.

## Context

Tool calls in ELAI are stateless: the model emits a call shape, the bridge
executes, the FSM moves on. There's no learning loop. If the model
consistently misnames a parameter (e.g. "path" vs "file_path") for tool X,
it repeats the mistake every session.

Case-Based Calibration:
1. Every tool call is logged with (call_shape, response_kind, post_call_audit).
2. A small index (filesystem-side, ~100KB per project) holds the last N
   successful invocations per tool.
3. Before each call, the bridge consults the case base for similar prior
   calls; if a near-match has a known parameter shape correction, it
   suggests the corrected shape via a hint injected into the model's
   next prompt.

This isn't fine-tuning. It's deterministic retrieval over per-project
tool-use history. Crashes a wide class of repeated mistakes.

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-35_tool_attention_dynamic_gating.md
- .elai_cc/crates/plugins/src/mcp/server.rs
- workflow/papers/case-based-calibration.md (paper-ingestor, Task 1)

## Tasks

1. **Source-grounding** — paper-ingestor (mode=paper) against arXiv:2605.15041.

2. **CaseBase store (crates/plugins/src/mcp/case_base.rs)** — sled or
   sqlite-backed (sled preferred; existing dep in workspace). Schema:
   tool_name | timestamp | call_shape_hash | call_shape | response_kind |
   post_call_audit_ok | session_id. LRU per tool: cap 100 entries; evict
   oldest.

3. **CaseRetriever** — given a prospective call (tool + arg shape), returns
   top-K nearest prior cases by call_shape similarity (Jaccard over arg
   key set + Levenshtein on string values; cheap, no embeddings needed).
   Returns nearest case + flag (NEAR_MATCH | NO_MATCH).

4. **Hint injection** — when the retriever finds a NEAR_MATCH whose call
   succeeded AND the prospective shape differs in a known way (e.g. missing
   key, renamed key), inject a hint message in the model's next prompt
   asking for the corrected shape.

5. **SSB action types** — case_base_consulted, case_base_hint_emitted,
   case_base_call_logged.

6. **Tests first (TDD)**:
   - 4K.X.3: CaseBase persists and loads round-trip
   - 4K.X.4: CaseRetriever similarity ordering deterministic
   - 4K.X.5: NEAR_MATCH with parameter-rename correction emits hint
   - 4K.X.6: LRU eviction at cap=100; oldest goes first
   - 4K.X.7: hint suppression: if user disables via config, no hints emit

7. **Pilot run** — replay a 50-session ELAI log set (existing sessions)
   through the case base, measure reduction in repeated tool-call errors.

8. **Contract tests** — tests/contracts/case_base_contract.rs.

## WIRING_CONTRACT

produces_symbols:
  - CaseBase, CaseRetriever, CaseHint (crates/plugins/src/mcp/case_base.rs)
  - SSB action_types: case_base_consulted, case_base_hint_emitted,
    case_base_call_logged
consumes_symbols:
  - MCP bridge invocation path
  - SSB event bus
  - sled persistent store
reachable_from:
  - every MCP tool invocation (consulted before dispatch)
  - elai_cc.toml [tool_use.case_base.enabled]
stub_ban:
  - case_base_call_logged MUST emit AFTER tool returns (audit trail)
  - eviction MUST be deterministic (oldest-by-timestamp, not random)
  - hint suppression config MUST gate AT THE EMIT POINT, not after

## DETERMINISTIC_CHECK

```bash
test -f .elai_cc/crates/plugins/src/mcp/case_base.rs
rg "case_base_consulted|case_base_hint_emitted|case_base_call_logged" .elai_cc/crates/orchestrator/src
cargo test -p elai_plugins -- case_base 2>&1 | grep "test result.*ok"
test -f workflow/sessions/$(cat workflow/sessions/current.txt)/case-base-pilot.md
```

## ACCEPTANCE

- [ ] paper-ingestor produces workflow/papers/case-based-calibration.md
- [ ] CaseBase persists via sled, schema validated
- [ ] CaseRetriever returns deterministic similarity ordering
- [ ] Hint injection on NEAR_MATCH parameter-rename case
- [ ] LRU eviction at cap=100
- [ ] All 3 SSB action types emit with non-null tool_name
- [ ] TDD tests pass
- [ ] Pilot replay produces measured reduction in repeat errors;
      target ≥10% on the 50-session set
- [ ] Contract tests pass

## Files

- ADD: .elai_cc/crates/plugins/src/mcp/case_base.rs
- ADD: .elai_cc/crates/plugins/tests/case_base_tdd.rs
- ADD: .elai_cc/crates/plugins/tests/contracts/case_base_contract.rs
- ADD: workflow/papers/case-based-calibration.md
- ADD: workflow/sessions/{id}/case-base-pilot.md
- MODIFY: .elai_cc/crates/plugins/src/mcp/server.rs (consult retriever pre-dispatch)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb/schema.rs (3 new action types)
- MODIFY: .elai_cc/elai_cc.toml ([tool_use.case_base] section)

## Next

C-47 closure means per-project tool-use mistakes accumulate into a
correction layer rather than re-firing per session. Pairs with C-35:
gating cuts catalog cost, calibration cuts repeat-mistake cost.
