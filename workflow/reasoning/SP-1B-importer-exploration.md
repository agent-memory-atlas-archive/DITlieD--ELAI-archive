SP-1B IMPORTER EXPLORATION
(placed by orchestrator from the integration-explorer's inline return — agent's Write was hook-blocked during SP-1A AUDITING/RUN_TESTS)

PRODUCTION_HOOK: backlog_import.py:468 (build_task(), allowed_write_set=allowed_files, fed by extract_allowed_files() at backlog_import.py:449/182-194) is the write-set attach seam unit 1.2 backfills. backlog_import.py:37 LETTERS = tuple("BCDEFGHIJK") (looped at 315, 355, 368) is the letter-exclusion seam unit 1.6 widens; add L, M, N.

SUBSYSTEM: teikoku-conductor import->enforcement path (/home/user/teikoku/conductor/, Python; NOT .elai_cc — elai.map has zero coverage).
CONFIDENCE: 0.85 (direct source reads + one live read-only sqlite query against /home/user/teikoku/conductor/state/control.db via file:...?mode=ro; canvas/dashboard consumer check is filename-grep only, flagged).

Q1 PRODUCTION_HOOK — see block above. extract_allowed_files (182-194): CREATE/MODIFY/DELETE/TOUCH/EDIT verb lines under "Files touched"; empty -> backtick fallback (190-194) over whole file for known-extension paths; returns [] when neither hits. build_wave_followup_task:528 inherits base_task.allowed_write_set.
Letter widen: LETTERS at :37, consumed by parse_corpus:315, load_status_seeds:355, load_bn_statuses:368. On-disk verified: L-plan/M-plan/N-plan exist with 21/6/15 non-INDEX plan files (42 total), structurally unreachable by all three loops. No A-plan (correct omission). Target: tuple("BCDEFGHIJKLMN").

Q2 CONSUMER CHAIN + DENY-ALL PROOF
models.py:69 TaskSpec.allowed_write_set -> :152 serialized allowed_write_set_json (control.db tasks column idx 6, live-confirmed) -> :181 deserialized. scope_utils.py:45-48 path_is_allowed = any(path_matches_scope(...) for scope in allowed_write_set); [] -> any([]) False for EVERY path — deny-all unconditional. scope_violations() :51-56. TWO enforcement sites: worker_adapter.py:353 (post-execution) and gate_runner.py:107 (gate FAIL :137-146 "allowed_write_enforcement"). Third consequence: codex_provider.py:44 sandbox = "workspace-write" if task.allowed_write_set else "read-only".

Q3 GUARD FALSE-POSITIVE CLASS + VERIFIED LIVE TRIP
guard.py:109-137 PromptGuard/OutputGuard = plain regex substring scanners (PROMPT_INJECTION_PATTERNS :11-29), no context-awareness. Wired WorkerAdapter.__init__ (:217-218), fired pre-dispatch (:290-295, guard_text = input_text or taskspec_file.read_text()). taskspec.txt is TaskSpec.to_prompt_text() (models.py:131-140 — objective/acceptance/deterministic_check JSON), NOT raw plan .md (context.md never guarded — grepped codex_provider/free_api_worker/worker_providers for "context.md": zero hits).
CONFIRMED LIVE TRIP: J-62_env_constraint_skill_cards_prewrite_gate.md has no "Objective:" label -> extract_objective 3-line fallback (:154-165) captures line 6 verbatim: "...a spec-eval of the Fable 5 chat system prompt found that a large fraction...". guard.py:17 `system\s+prompt` matches "chat system prompt" -> plan.objective -> TaskSpec.objective -> taskspec.txt WILL trip PromptGuard (false positive: cites prior art).
NEAR MISSES ruled out: J-102:82 DETERMINISTIC_CHECK rg pattern has pipe chars breaking adjacency; J-62:5 "system-prompt" hyphenated; J-12:10 + J-36:20/60 "system prompt" sits under headings extract_block never captures.
NEEDS_REVIEW already a genuine terminal status (models.py:21/32/35; escalation_export.py:39 ATTENTION). guard_blocked error_category exists (worker_providers.py:259-260). No import-time pre-screen exists yet.

Q4 LIVE COUNTS (read-only control.db)
Total tasks 2012. Status: PLANNED 1561, DONE 328, READY 81, NEEDS_REVIEW 37, BLOCKED 3, CANCELLED 2.
Empty allowed_write_set_json ('[]' or NULL): 1508/2012 (75%). By status: PLANNED 1216, DONE 234, READY 40, NEEDS_REVIEW 15, CANCELLED 2, BLOCKED 1. Non-terminal empty = 1272. (234 DONE-with-empty plausibly FINISH-seeded, never dispatched.)
Letter exclusion separate: zero DB rows for L/M/N/A; 42 real L/M/N plan files on disk never imported.

Q5 BLAST RADIUS OF NON-EMPTY WRITE-SETS FOR ~1256 PLANNED+READY
leases.py:42-56 scopes_conflict(empty_conflicts=True default): empty set = "undeclared" = conflicts with EVERYTHING -> serializes dispatch (scheduler.py:130). Backfill switches ~1256 tasks to declared-overlap checks -> many concurrent leases (intended but a real concurrency-model change; parallel_undeclared_scopes configured default NOT verified).
worker_adapter.py:481 generates real per-path Read()/Write() ACLs once populated — first real ACL for these tasks; too-narrow backfill would newly block previously-unchecked edits.
codex_provider.py:44 flips sandbox read-only -> workspace-write for all backfilled tasks.
context_builder.py:108 starts reading+inlining allowed-write files into context.md, competing for the 12000-char budget (:15).
GAP: no canvas/dashboard consumer of allowed_write_set_json found by filename grep (harness_api.py, orchestration_api.py, chat_context_api.py, slice_api.py, free_api_worker.py) — dynamic consumer not fully ruled out.

GAPS/RISKS: backtick-extension fallback is blunt for research-style docs quoting many illustrative paths — prefer produces_symbols (Tier2) over the loose fallback ordering. parallel_undeclared_scopes default unverified.
DRIFT: none vs task framing. Note: empty-write-set rate is 75% (1508/2012), higher than "some plans" implies.
