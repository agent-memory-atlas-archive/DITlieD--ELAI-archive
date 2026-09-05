Phase 3H.6 decisions — create_handoff / load_handoff + integration test

What: add two public methods to SessionDag (create_handoff, load_handoff) and a capstone 11-step integration test exercising all 3H.1-3H.5 features.

Key decisions:

1. create_handoff signature: &self (not &mut self) — it only appends a tagged snapshot. No branch creation involved. Parameters: messages, config, ssb (unused but reserved for future recovery path), entropy, clock, fsm_state (&str).

2. valid_ssb_seqs derivation: collect all Some(stub_ssb_seq) values from the input messages into a HashSet<u64>. This set feeds into trim_context pass 3.

3. parent_id: obtained from latest_snapshot(None) — the newest snapshot on trunk becomes the parent. If no snapshots exist, parent_id is None (root handoff).

4. tag: hardcoded "handoff" string literal. No enum — matches existing tag convention (e.g. "branch:{name}" in create_branch).

5. fsm_state: accepted as &str parameter. The caller (CLI/orchestrator loop) knows the current FSM state; session_dag should not depend on the FSM module.

6. load_handoff: branch-tip-first logic. Check active_branches(); if non-empty, load first branch tip snapshot. Otherwise fall back to latest_snapshot(Some("handoff")). Returns Ok(None) if neither exists. Pure reads, &self.

7. token_count disposition: DEFERRED. providers::token_count is todo!() in all implementations (confirmed: ollama.rs L421-422, L492-493). FIXME comment updated from 3H.6 to 3H.7. estimate_tokens heuristic (bytes+3)/4 stays.

8. TrimConfig.allocated_slots_tokens: maps to context_budget.total_allocated field (usize, L110 in context_budget.rs). Integration test uses a direct numeric value.

9. No new dependencies. No new error variants needed — existing error types cover all failure modes.

Expected test outcome:
- 5+ contract tests for handoff methods
- 4+ unit tests for create_handoff/load_handoff
- 1 integration test (test_handoff_integration_lifecycle) covering 11 steps
- All 56 prior unit tests + 20 prior contracts + 26 SSB tests remain green

Unknowns:
- create_branch takes &mut self but create_handoff takes &self — load_handoff will use &self. The branch-tip-first logic only reads active_branches (already &self).
