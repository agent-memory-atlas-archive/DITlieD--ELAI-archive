Phase 0.8 decisions

What: verification gate — confirm 0.6/0.7 implementation is GREEN, create smoke test and metrics artifacts, harden contract tests per audit findings.

Key decisions:
1. FSM tools made stateful via global Mutex<FsmState> — stubs replaced with persistent state tracking. Chose Mutex over AtomicU8 enum because phase field is a String.
2. Visibility filter extended with ends_with("/.elai_cc") to block absolute path bypass. Minimal change, no performance impact.
3. Contract tests hardened: provider_trait now tests serde round-trip (None when absent, Some when present) instead of Debug field name check. fsm_basic now verifies state persistence across get/transition/get cycle. visibility_filter now verifies .elai_cc appears in list_hidden_paths output.
4. FSM tools (fsm_get_state, fsm_transition, list_hidden_paths) registered in deferred_tool_specs via internal_tool_specs() — previously callable by name but invisible to search/list.
5. Smoke script uses OLLAMA_MODEL env var with llama3 fallback, treats model-not-found as SKIP=environmental (not FAIL).

Expected test outcome: all 7 contract tests pass, workspace pass count >= 578, 0 regression.
Unknowns: flaky shell tests cause +/-2 variance in workspace totals between runs.
