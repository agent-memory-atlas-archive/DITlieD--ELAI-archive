M89 SCOUT — M8/M9 addon (teikoku-merge-addon-M8-M9-2026-07-05.md) verified against LIVE
source. session_id=s-finish-0705. All conductor/*.py paths relative to
/home/user/teikoku/conductor/ (WSL distro teikoku, read via wsl -d teikoku; UNC Read also
worked this pass). Read-only; no files touched except this output.

(a) M9 DELTA FACTS — M9 IS ALREADY SUBSTANTIALLY SHIPPED

Commit e3c3800 ("wave 1 of the finish plan engine merge") landed THE SAME DAY this addon was
authored and already ships most of what M9 asks for. backlog_import.py is now 820 lines (was
705 at SP-1B). Verified directly:
- ParsedPlan (backlog_import.py:57-67) carries acceptance (64) AND production_hook (66), not
  just raw wiring_contract text.
- extract_production_hook() (278-287): regex `reachable_from:\s*\n((?:\s+-\s+.+\n?)+)` over
  wiring_contract, takes the FIRST bullet (or an inline `reachable_from: ...` fallback).
- parse_plan_file() (290-313) calls extract_block(content,"ACCEPTANCE") (300) for a literal
  top-level "## ACCEPTANCE" heading -> ParsedPlan.acceptance (309), separate from
  WIRING_CONTRACT's own observable_effect prose.
- build_task() (533-599) wires both into TaskSpec: production_hook=plan.production_hook (579),
  acceptance=plan.acceptance or "No ACCEPTANCE block found in source plan." (580).
- models.py TaskSpec (62-72): role/allowed_write_set/production_hook/acceptance are first-class
  fields, serialized to DB columns production_hook + acceptance_text (152,154-155), deserialized
  178-184.
- BOTH dispatch surfaces already render them: context_builder.py _task_section (86-103) prints
  "production_hook: {task.production_hook or ''}" (93) and "Acceptance:\n{task.acceptance}"
  (98-99); models.py TaskSpec.to_prompt_text() (131-140) dumps the full to_json_dict(), which
  includes production_hook+acceptance, as JSON.
- extract_allowed_files/_resolve_allowed_files (256-271, the 3-tier Tier1/Tier2/fallback logic)
  is ALSO the shipped SP-1B/M3 fix — confirmed live, not still-open.

REMAINING GAP (the honest fraction): the IMPERATIVE FRAMING the addon specifies -- "WIRE your
new public symbol into <production_hook>...a DARK symbol is a BLOCK" -- is nowhere in the
rendered text today. Current rendering is purely DESCRIPTIVE (states field values), never states
the consequence of not wiring. This is a one-sentence addition to existing rendering functions
(_task_section or to_prompt_text's 3-line boilerplate), NOT a new parse/extract pipeline. M9's
remaining work is much smaller than the addon plan assumes.

CORPUS PRODUCTION_HOOK COVERAGE SAMPLE (n=3, full files read: J-16, J-100, N-1):
All 3 HAVE "## WIRING_CONTRACT" with a "reachable_from:" key (extract_production_hook always
finds something on this sample). But in ALL 3, the bullets are PROSE TRIGGER/SCENARIO
descriptions, never a "file:fn" seam:
  J-16 (line 45-47): "the AUDITING FSM transition on a real `elai` task/fix run
    (post-EXECUTING, auditor-driven) [PRODUCTION]"
  J-100 (line 64-68): "a live `elai run` model issues a background tool -> a task_id returns
    IMMEDIATELY, BackgroundTaskStarted lands..." (first bullet only, ~220 chars)
  N-1 (line 37-39): "CLI `elai bench <family>` -> runner -> corpus row + profile curve write
    [PRODUCTION bench seam]"
So production_hook is non-empty and real on this sample, but it is a REACHABILITY SCENARIO, not
an actionable file+fn pointer. "WIRE your new symbol into <production_hook>" does not
grammatically fit what's actually extracted. This tracks production-caller-gate.md's OWN
distinction between the plan file's reachable_from (an R1 evidence field, prose) and PCG-1's
PRODUCTION_HOOK (a dispatch-time EXPLORER-AUTHORED file+fn seam) -- different things sharing a
name; the importer harvests the former. ACCEPTANCE coverage: only J-100 (1/3) has a real "##
ACCEPTANCE" heading; J-16 and N-1 fall to the default string. n=3 is small -- do not
over-generalize -- but spans 2 plan eras/tracks (J ~06-25, N ~07-04) with the same result.

(b) DISPATCH INJECTION SEAM

WorkerAdapter.dispatch() (worker_adapter.py:242; calls render_taskspec@277 ->
render_context_packet@278 -> render_role_config@279) is the exact seam. render_taskspec
(466-471) writes taskspec.json + taskspec.txt (task.to_prompt_text(), models.py:131).
render_context_packet (473-475) -> context_builder.build_context_packet_for_task ->
write_context_packet -> context.md + context.json. render_role_config (477-514) writes
role.config.yaml (role prompt + allow/deny ACL derived from allowed_write_set). An M9 imperative
directive lands in models.py to_prompt_text()'s boilerplate (131-140) or context_builder.py
_task_section (86-103). Budget it competes with: context_builder.py:15
DEFAULT_MAX_TOTAL_CHARS=12000 (Task section is FIRST in _cap_sections' list, 46-51, so least
likely truncated). to_prompt_text()/taskspec.txt has NO char budget at all.

(c) PROMOTION DESIGN FACTS + WHERE THE 81 READY CAME FROM

THREE distinct PLANNED->READY paths found, not one:
1. STATUS_MAP (backlog_import.py:40-53): verdicts SHIPPED_WIRE_DARK/PARTIAL/MOSTLY_DONE map
   straight to status="READY" from a FINISH-file or bn_status seed.
2. build_task's explicit override (543-544): `if role=="WIRE" and status=="PLANNED":
   status="READY"`. role=="WIRE" only when status_seed.verdict=="SHIPPED_WIRE_DARK" (539-540),
   which STATUS_MAP already maps to READY at line 542 -- so this is largely a defensive
   re-assertion of the same condition, not an independently-firing path (no counter-case found).
3. spawn_drainer.py (10-21,47,150-173): a SEPARATE task-creation path (spawn descriptors, not
   plan-file import) that promotes its own PLANNED spawn tasks to READY directly via
   db.update_task_status(task_id,"READY",reason).

_dep_blocked_ready_task (conductor_loop.py:487-564) IS "the fixed predicate" the addon
references -- confirmed TRANSITIVE (memoized DFS _resolvable, 525-546), walking the full dep
closure and distinguishing "still-resolvable" from a chain terminal-blocked behind
_TERMINAL_DEP_STATES (NEEDS_REVIEW/BLOCKED/FAILED/CANCELLED, line 484). This is ALREADY LIVE
(2026-07-05 walk-gatefix per its own docstring), called from run_wave_loop's empty-frontier
branch (667) via wake.wait_for_dep (event-driven, not busy-poll). M8/M9 do not need to rebuild
this.

81 READY origin: SP-1B's prior live query (not re-run this pass -- Write outside
workflow/reasoning/ was hook-blocked by the FSM gate, so a fresh DB query round-trip was
dropped to stay in scope) found 81 READY total, 40 with EMPTY allowed_write_set_json at that
time (pre-dating the now-shipped 3-tier write-set fix) -- roughly half should self-heal on next
import. A role/task_id-suffix breakdown (WIRE- vs EXECUTE-) was NOT re-queried this pass --
recommend `SELECT role, COUNT(*) FROM tasks WHERE status='READY' GROUP BY role` before relying
on an exact split.

Generic "deps-satisfied PLANNED->READY" promotion: NOT found as a named function distinct from
the 3 paths above. scheduler.schedule_frontier (scheduler.py:91) is the actual leasing gate;
whether it itself filters READY-but-deps-incomplete rows out of leasing (making a separate
promotion step moot) was NOT read this pass (budget) -- inferred only from
_dep_blocked_ready_task's docstring, not directly confirmed. Flag for executor to read
schedule_frontier's body before assuming a promotion step is missing.

(d) M8 SEAMS + RETRY-CARRY MECHANISM

Gate-fail disposition -- EXACT site: conductor_loop.py:372-379 (the `else` after `if
verdict=="PASS"` at 343, inside the non-dry-run VERIFYING stage, gate_result=run_gates(...) at
330):
  else:
      decision = budgets.can_start_task(task_id)
      if decision.allowed:
          db.update_task_status(task_id, "NEEDS_REVIEW", f"gate_failed_{verdict}")
      else:
          db.update_task_status(task_id, "FAILED", f"gate_failed_{verdict}_budget_exhausted")
Confirms the addon's premise exactly: ANY gate fail with budget remaining goes straight to
NEEDS_REVIEW (terminal, models.py TASK_STATUSES) with NO automatic retry today, even though
attempts is typically well under max_attempts (default 3, build_task:596). GUIDED_RETRY branches
in here, ahead of this existing if/else.

Finding structure (gate_runner.py run_gates return, 198-211): {"task_id","overall_verdict",
"gate_results":[{"gate_name","verdict","output",...} per command],"touched_files",
"new_rust_pub_symbols":[...],"new_py_pub_symbols":[...],"failure_category":
_failure_category(overall,gate_results) [body NOT read this pass -- likely an EXISTING
deterministic classifier M8 should reuse rather than rebuild, mirrors ELAI's own
failure_taxonomy.py],"structured_feedback":"[gate_name]\noutput\n\n..." (163/195, all failing
gates concatenated)}. The DARK-symbol name is NOT a separate structured field -- it lives only
inside the matching gate_results[i]["output"] prose (the checker subprocess's own stdout),
needing regex extraction, matching what the addon already assumed. new_rust_pub_symbols/
new_py_pub_symbols (108-109,128-129) are the INPUT symbols fed to the checker, not its per-symbol
verdict -- do not conflate.

attempts semantics -- MISMATCH vs the addon's literal wording: attempts increments in
budgets.record_attempt_start (197-246, `retry_attempts=int(row["attempts"])+1` at 221, UPDATE at
223) BEFORE the worker runs (an attempt-START hook) -- so by the time a gate can fail, attempts
is ALREADY >=1 for that attempt. The addon's guard "attempts < 1" can never be true after any
attempt; it must be attempts<=1 (this gate-fail followed the task's first attempt) to mean "ONE
guided retry" as intended. Implement the cap as attempts<=1 read at the disposition site, not a
new counter.

Retry-carry mechanism -- NO dedicated constraint/notes/hint column exists on tasks (grepped
conductor_db.py for notes/guidance/retry_note/constraint: only an unrelated sanitized_notes
column on a DIFFERENT table, 471/478/488). Two EXISTING-FIELD options, zero schema change:
  1. Append to TaskSpec.objective (tasks.objective column) -- already rendered verbatim in both
     dispatch surfaces (_task_section:96 "Objective:\n{task.objective}"; to_json_dict:116). A
     guided retry does `UPDATE tasks SET objective = objective || '\n\nGUIDED_RETRY: <constraint>'
     WHERE task_id=?` before flipping status to READY -- flows into the next attempt's context.md
     AND taskspec.txt automatically, no other code change.
  2. Append to TaskSpec.acceptance similarly (rendered _task_section:98-99, to_json_dict:121) --
     semantically closer to "must additionally satisfy X" but unverified whether anything else
     parses acceptance_text expecting only original plan content.
Recommend option 1 (objective-append) as the concrete mechanism; flag option 2 as fallback if the
executor finds objective is consumed elsewhere in a way appended text would break (not verified
either way this pass).

UNVERIFIED, FLAG DO NOT GUESS: whether db.update_task_status(task_id,"READY",...) alone (the
happy-path call at 376) frees the worker_cap slot for re-leasing, or whether an explicit
leases.release(lease_id,"READY",reason) is ALSO required. The exception-path handlers (398,452)
explicitly call leases.release; the happy-path gate-fail branch (372-379) does NOT, and
update_task_status (conductor_db.py:373-399) only touches tasks.status + appends an event +
wake.publish -- it never touches the leases table. I could not find where/whether the happy
path's lease gets released within my read window (300-460). Executor MUST grep
`leases.release` across the FULL run_task_loop body before assuming update_task_status alone
suffices for GUIDED_RETRY's re-entry.

(e) FROZEN-GATE MEMBERSHIP VERDICT

TWO manifests exist, both read in full:
1. harness/elai/.claude/gate-manifest.txt -- the ELAI TARGET-project template (freezes
   .claude/rules/wiring-enforcement.md, wiring_contract_check.py, gate_freeze_check.py,
   maintainability_check.py, .elai_cc/smoke-manifest.toml, wire-dark-allowlist.toml,
   elai_cc.toml, several H-N crate files) -- ALL paths inside the ELAI target repo, irrelevant to
   teikoku's own conductor source.
2. /home/user/teikoku/.claude/gate-manifest.txt + gate-manifest.lock -- the DISTRO-side
   freeze, BOTH present, dated Jul 4 -- LOCKED (contrary to gate_packs.py's own comment
   suggesting it might still be inert/WIRE-DARK[P3-C] -- the lock file's presence means
   distro_gate_freeze IS active). Full content: conductor/gate_packs.py, conductor/gate_runner.py,
   conductor/plan_schema.py, 3 Unity static-analysis scripts + unity-symbol-policy.json,
   .claude/tools/slop_semantic_check.py, ~14 userSettings hook files (../.claude/hooks/*.py +
   settings.json), tools/aor/change_recorder.py, conductor/index_reference_corpus.py, and the
   manifest itself.
VERDICT: backlog_import.py, conductor_loop.py, context_builder.py, worker_adapter.py, budgets.py,
scheduler.py are NOT listed in EITHER manifest -- free to edit, no out-of-band re-lock needed.
This matches finish-plan.md line 71's own claim ("M8/M9 do not change the frozen gates... confirm
that") -- CONFIRMED TRUE. CAUTION: gate_runner.py itself IS frozen. If M8 needs run_gates() to
return anything NEW (e.g. a structured dark-symbol field instead of prose-only output), that IS
a frozen-file edit requiring an out-of-band human-reviewed re-lock. Design M8 to consume
gate_runner's EXISTING output shape (regex over structured_feedback / gate_results[i]["output"])
rather than requesting a gate_runner.py schema change, to stay inside the free-to-edit set.

(f) RISKS

1. M9's "coverage fraction" needs re-framing: parse+propagate plumbing is DONE; what remains is
   a one-sentence imperative addition. Do not re-build extract_production_hook or
   extract_block("ACCEPTANCE") -- both exist and work as designed.
2. production_hook is a REACHABILITY SCENARIO (prose), not a file+fn seam, on 3/3 samples across
   2 tracks. An imperative literally saying "wire into <production_hook>" will often read
   grammatically wrong; word it as "your change must be exercised via the path below; the audit
   runs production_caller_check + this task's ACCEPTANCE" instead of "wire into X".
3. "attempts < 1" in the addon text does not match real counting semantics (attempts increments
   at attempt-START, before any gate result exists) -- using it literally makes GUIDED_RETRY's
   guard permanently false. Must be attempts<=1.
4. The happy-path lease-release site for a gate-fail disposition was NOT located this pass.
   Implementing GUIDED_RETRY's READY re-entry without confirming this could silently fail to free
   the worker slot, or double-release an already-released lease. Grep leases.release across
   conductor_loop.py's FULL body first.
5. gate_runner.py, gate_packs.py, plan_schema.py are DISTRO-FROZEN (R8). Any M8 design requiring
   a gate_runner.py return-shape change needs an explicit out-of-band re-lock -- materially bigger
   than editing conductor_loop.py/budgets.py alone. Prefer parsing existing output.
6. _failure_category (gate_runner.py:207, body not read this pass) may already BE the
   retry-worthy classifier M8 wants to build -- check it first to avoid duplicating an existing
   deterministic classifier.
7. Section (c)'s promotion-path finding is read-partial: schedule_frontier's own dep-filtering
   logic (scheduler.py:91) was not read this pass; whether it already excludes
   READY-but-deps-incomplete rows from leasing is inferred from _dep_blocked_ready_task's
   docstring only, not directly confirmed by reading schedule_frontier's body.

R-5 STRUCTURED RETURN
STATUS: DONE
FILES TOUCHED: F:\.ELAI_workflow\workflow\reasoning\M89-scout.md (this report) only.
VERIFIED: every file:line citation above read directly (WSL teikoku distro conductor/*.py via
wsl -d teikoku and \\wsl.localhost UNC Read; J-16/J-100/N-1 read in full via Read on
F:\.ELAI_workflow\docs\thefinalglance\; both gate-manifest.txt files + gate-manifest.lock
existence read in full; e3c3800 commit message confirmed present in `git log`). Two live
sqlite3 queries attempted this pass to size the 81 READY rows by role were dropped after the
Write tool was hook-blocked outside workflow/reasoning/ (EXECUTING-state PreToolUse gate) --
relied on SP-1B's prior same-session query instead rather than fight scope.
UNVERIFIED (repeated from (d)/(f) for visibility): happy-path lease-release call site;
schedule_frontier's dep-filtering body; _failure_category's body/categories; exact READY
role/task_id-suffix breakdown.
QUESTION: none (not blocked).
NOTES: the single biggest finding is that M9 is mostly ALREADY SHIPPED (e3c3800, same-day) --
the addon plan was very likely drafted without seeing that commit land. Confirm this timing
with the user/controller before dispatching an M9 executor for a full rebuild; the real
remaining M9 work is a small imperative-sentence edit, not a parse pipeline.
