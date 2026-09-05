B-20B — Decision Doc
=====================
Phase: B-20B (BLOCK-1 wire + W1-W5 fixes from B-20A audit)
Session: s-27276

WHAT
----
Fix 1 BLOCK + 5 WARNs identified in B-20A audit:
- BLOCK-1: enforce_file_read_budget_guard has zero production call sites.
- W1: dispatcher.rs:364 SyntaxError missing "error: expected" pattern.
- W2: dispatcher.rs:383 RuntimePanic missing "timeout" (only has "timed out").
- W3: failure_taxonomy.rs:133 bare "assertion" → "assertion failed".
- W4: failure_taxonomy.rs:138-140 && ... || precedence bug.
- W5: task_runner.rs:5016 silent None classification gap → add tracing::debug!.

OUT OF SCOPE (tracked as carryover for later phases):
- IpcToolRegistry vs InProcessToolDispatcher (depends on B-18D IPC accept loop).
- Tree-sitter parsed file outline (B-20A used line-prefix heuristic; substantial new feature).
- WIP commit squash on AUDITING → MEMORIZING (depends on user policy).

KEY DECISIONS
-------------
D1. BLOCK-1 wire — 2 edits in task_runner.rs:
    a. Add run_file_read_dispatch helper (~20 LOC) at the top of the impl block (or near
       run_classify_dispatch).
    b. Call it at Batch 2B success arm (~line 4223).
    State: budget_remaining_tokens sourced from
    resolved.model_capability_profile.recommended_prompt_budget as u32.
D2. Total LOC delta: ~33 lines across 3 files (acceptable for one fix-cycle).
D3. No new dependencies introduced.

EXPECTED TEST OUTCOMES
----------------------
- cargo check --workspace passes.
- Existing classifier tests still pass (no regressions in 9-cat taxonomy).
- New behavior: outline-mode file reads now invoke the guard before reading; over-budget reads
  return Block instead of proceeding.
- Trace logs show debug entry when classifier returns None (W5 fix).

UNKNOWNS
--------
- Whether the OUTLINE_LINE_THRESHOLD import already exists in task_runner.rs scope.
- Whether `dispatcher` is in scope at line 4223 with the right type (resolved struct field
  access pattern needs verification).

ROLLBACK
--------
git restore on the 3 modified files. No external state changes.

ACCEPTANCE
----------
A1: cargo check --workspace PASS.
A2: cargo test -p <crate-with-failure_taxonomy> classify -- --nocapture passes (≥1 PASS).
A3: cargo test -p <crate-with-dispatcher> -- --nocapture passes (no regressions).
A4: enforce_file_read_budget_guard has at least one production call site grep can find.
A5: B-20A WARNs W1-W5 each have a fix verified by reading the new code.
