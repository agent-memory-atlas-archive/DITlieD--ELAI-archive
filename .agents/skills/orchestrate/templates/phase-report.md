TEMPLATE — per-sub-phase end report

Written by the launched session (or the host in same-project mode) at the end of each sub-phase cycle. Path: workflow/sessions/<sid>/phase-reports/<sub_phase_id>.md.

Replace {{placeholders}} with concrete values. Keep terse — this file is read by the orchestrate controller + bubble-up logic + final report aggregator. Long prose belongs in workflow/reasoning/, not here.

------- TEMPLATE BEGINS -------

# Sub-phase report — {{SUB_PHASE_ID}} ({{SUB_PHASE_TITLE}})

plan: {{PLAN_PATH}}
session: {{SESSION_ID}}
started: {{ISO_TIMESTAMP_START}}
finished: {{ISO_TIMESTAMP_END}}
duration_s: {{DURATION_SECONDS}}
status: {{PASS | FAIL | DEFERRED}}

## Tasks executed
{{NUMBERED_LIST_OF_TASKS_FROM_PLAN_SUB_PHASE}}

## Files touched
{{LIST_OF_FILES_WITH_CHANGE_KIND: CREATE | MODIFY | DELETE}}

## Agents spawned
{{TABLE: agent_name | invocation | duration_s | return_summary_one_line}}

Per the protocol the canonical order is:
- context-compiler (parallel batch)
- integration-explorer (parallel batch, can be multiple)
- prompt-engineer (SKIPPED per orchestrate POLICY 2)
- (main LLM executes — not an agent dispatch)
- auditor (1 cycle)
- wiring-contract-validator (if WIRING_CONTRACT block in plan)
- memory-writer (scoped to this sub-phase)

## Stubs

new_stubs: {{LIST: stub_id | file:line | will_resolve_in_sub_phase}}
closed_stubs: {{LIST: stub_id_closed | by_what_change}}

## Gates

{{TABLE: gate_name | PASS|FAIL | exit_code | duration_s | first_failure_line_if_any}}

stack-specific gates run: {{STACK}}
cross-stack gates: {{YES|NO}}
dashboard wiring touched: {{YES|NO|N/A}}

## Auditor verdict (one line copied from auditor return)

{{AUDITOR_PROGRESS_SUMMARY_LINE}}

block findings (if any):
{{LIST_OF_BLOCK_FINDINGS_WITH_FILE_REFERENCES}}

regression module (per auditor):
{{REGRESSION_MODULE}}

## Bubble-up events triggered

{{LIST_OR_NONE: bubble_id | action | target | outcome}}

## Anti-spiral events

{{LIST_OR_NONE: pattern | from_persona | to_persona | reason}}

## Test deltas

new_tests_added: {{COUNT}}
tests_passing_now: {{COUNT}}
tests_passing_before: {{COUNT}}
delta_coverage_check: {{PASS|FAIL}}
uncovered_new_public_symbols: {{LIST_OR_NONE}}

## Decisions captured

{{ONE_PARA_OR_LIST: key architectural / scope decisions made in this sub-phase}}

## Next sub-phase

next_sub_phase_id: {{NEXT_ID}}
preconditions for next: {{LIST_OR_NONE}}
warnings to carry forward: {{LIST_OR_NONE}}

## Tokens consumed (this sub-phase only)

input_tokens: {{N}}
output_tokens: {{N}}
cache_read_tokens: {{N}}
total_cost_usd_est: {{N}}

------- TEMPLATE ENDS -------

NOTES FOR LAUNCHED SESSION

- Write this file BEFORE advancing tree-cursor.txt to the next sub-phase
- If status=FAIL: the bubble-up action chosen goes in the "Bubble-up events triggered" table
- If status=DEFERRED: the deferral reason goes in the "Decisions captured" section
- Tokens: read from stream-json result events of the spawned agents + this session's own usage event
- Reports are append-only at the file level: never edit a prior sub-phase's report

END phase-report.md template
