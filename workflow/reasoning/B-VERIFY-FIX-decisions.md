B-VERIFY-FIX — Decision Doc
============================
Phase: B-VERIFY-FIX (4 BLOCKs from B-VERIFY audit)
Session: s-27276

CONTEXT
-------
MCP server local-rag became disconnected mid-audit. FSM transitions cannot be issued by the
orchestrator while down; the on-disk FSM state file (.fsm-state.json) and PreToolUse hook
should still gate edits correctly for sub-agents. Treating this fix phase as a continuation
of B-VERIFY's audit cycle (no new FSM transition needed; agent edits proceed under existing
EXECUTING/AUDITING state).

WHAT
----
Fix 4 BLOCKs and 2 WARNs from B-VERIFY-audit-report.md.

KEY DECISIONS
-------------
D1. BLOCK-4 (prompt-bloat showstopper) — cap git diff in system prompt.
    File: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\prompt.rs
    Function: read_git_diff() at lines ~252-270; rendered at lines ~300-306.
    Fix: introduce const MAX_GIT_DIFF_CHARS (32_768 / 32K chars by default), truncate the
         diff with a "...<truncated N chars>..." marker if exceeded. Make the cap configurable
         via env var ELAI_MAX_GIT_DIFF_CHARS for future tuning.
    Acceptance: real elai run completes a turn against the 7B (131K window) without context
         overflow; SSB rows > 0.

D2. BLOCK-1 (probe 1I cascade degenerate).
    File: F:\.ELAI_workflow\.elai_cc\scripts\verify_stage_1I.sh
    Replace 'cascade' filter (0 matches) with a filter that invokes the 19 degradation tests.
    Probably `cargo test -p elai-knowledge --release degradation -- --nocapture` then assert
    count >= 1 PASS that exercises SearchMode::TextOnly.
    Acceptance: probe runs ≥1 real test exercising the cascade behavior.

D3. BLOCK-2 (probe B20A_taxonomy degenerate).
    File: verify_stage_B20A_taxonomy.sh
    Filter that invokes classify_failure() tests in tool_dispatcher_contract.
    Acceptance: ≥1 real test exercising 9-cat taxonomy bucketing.

D4. BLOCK-3 (probe B20A_similarity degenerate).
    File: verify_stage_B20A_similarity.sh
    Filter that invokes dispatch_edit() similarity-hint tests.
    Acceptance: ≥1 real test exercising similarity_hint surface.

D5. WARN-1: remove dead code in verify_stage_1G.sh after exit 0/1 (lines 47-70).

D6. WARN-2: in verify_stage_1I.sh and verify_stage_B9_sessiondag.sh, label source-grep PASS
    fallback as REDUCED (e.g. "PASS|<value>(reduced:source-grep-only)") so it's clear in
    the aggregator output.

D7. INFO follow-up (NOT in this phase): model name shows "gemma4:e2b" in trace but Qwen2.5-7B
    loaded. Add to follow-up list; investigate in a future phase.

EXPECTED TEST OUTCOMES
----------------------
- prompt.rs change: cargo check -p elai-orchestrator passes; new const visible.
- Real elai run with capped prompt: no overflow; SSB rows > 0; first LLM turn succeeds.
- 3 fixed probes: each runs ≥1 real test and PASSes legitimately.
- WARN fixes: clean script output.

UNKNOWNS
--------
- Whether 32K char cap is tight enough; spec doesn't define MAX_GIT_DIFF_CHARS. 32K is a
  conservative default that leaves the rest of the 131K window for memory-bank + agent defs +
  user prompt + assistant response.
- Whether other parts of the prompt also bloat (memory-bank? agent .md inlines?). The audit
  identified git diff as primary driver; if real run still overflows after fix, follow-up
  investigation needed.

ROLLBACK
--------
git restore on prompt.rs and the 5 modified probe scripts. No external state changes.

ACCEPTANCE
----------
A1: cargo check --workspace PASS after prompt.rs edit.
A2: elai.exe -p "<task>" completes ≥1 LLM turn; SSB rows > 0; total prompt under 100K tokens
    on the nextest-main corpus.
A3: 3 probes (1I, B20A_taxonomy, B20A_similarity) each invoke ≥1 real test.
A4: B-VERIFY-report.md updated with new probe results; 18/18 PASS legitimate.
A5: graphcodebert.onnx integrity preserved.
