SP-1A executor (unit 1.1, TEIKOKU-ELAI-FINISH-PLAN.md) - RE-DISPATCH, STATUS DONE

session: SP-1A-exec2, re-dispatched after SP-1A-exec-blocked.md's tool-denial BLOCK.
Mechanics from SP-1A-DECISION.md worked as prescribed: all teikoku reads/writes/commands
went through the PowerShell tool (wsl -d teikoku -- bash -lc '...'), never the Bash tool,
never Write/Edit on a UNC path, no git mutation.

SUMMARY
Applied the full design from SP-1A-exec-blocked.md (precheck.py, the 4 backlog_import.py
edits, test_precheck.py) with two grounded corrections found while re-verifying the design
against the LIVE tree, plus one additional correction found only by actually RUNNING the
code (a JSON-payload parsing gap in the failure-reason helper). All three deliverable files
are written, compile clean, and 24/24 tests pass (15 new + 9 pre-existing, zero regressions).
The --no-precheck escape hatch is proven end-to-end against the real ~2000-task ELAI corpus
(4.5s). One real precheck.precheck() call against a live B-plan file (B-9) ran end-to-end in
201s and produced the CORRECT verdict (EXECUTE, because that plan's trigger script hardcodes
"python" which does not exist in this WSL distro -- only "python3" -- so the checker cannot
run it; this is the checker faithfully reporting an environment fact, not a bug).

WORK DONE
1. Re-verified the design's assumptions against the LIVE tree before writing anything
   (backlog_import.py's actual current line numbers, gate_packs.py's PROVEN checker
   invocation shape, project_resolver.py's real repo_root resolution for the elai project,
   wiring_contract_check.py / production_caller_check.py's actual exit-code/JSON contracts).
   All matched the design's own resolution of Finding 1 / Finding 2 almost exactly.
2. Wrote conductor/precheck.py (242 lines): CERTIFY/EXECUTE/PROBE_PENDING constants,
   LegResult/PrecheckResult/ProbeResult frozen dataclasses, _run_checker (subprocess wrapper,
   never raises), extract_symbol_list (widened '- item' extraction), _leaf_symbol_for_scan
   (NEW vs the blocked design: strips a 'file.rs::name' qualifier to the bare leaf before
   handing it to production_caller_check.py's literal-name --symbols scan, mirroring
   wiring_contract_check.py's own _leaf_symbol helper -- without this, every qualified
   produces_symbols entry would false-DARK), precheck() (the two-leg live-evidence check),
   done_probe() (provider=None today; raises NotImplementedError for a bound provider).
3. Edited conductor/backlog_import.py (4 changes, 648 -> 704 lines): `import precheck`
   sibling import; `--no-precheck` argparse flag (default off, i.e. precheck runs by
   default); load_precheck_seeds() + load_probe_pending_warnings() (new module functions,
   the PRODUCTION_HOOK); main() wiring between the bn-status seed loop and build_tasks(),
   gated `if is_rust and not args.no_precheck`, folding probe_pending into report[].
4. Wrote conductor/tests/test_precheck.py (207 lines, 15 tests): PrecheckFunctionTests
   (extract_symbol_list correctness incl. the qualifier-stripping case, contract-passing/
   -failing/-empty, checker-invocation-error via a guaranteed-nonexistent repo_root,
   done_probe both branches) + PrecheckSeedMergeTests (TempDirMixin; existing-DONE-seed
   never re-checked, weaker-seed upgrade, failing-plan gets no seed, probe-pending
   warnings collected, and the frontier test: a DONE-seeded lead task via build_tasks +
   import_rows into a real tmp ControlDB is absent from ready_task_rows() and counted once
   under task_counts()["DONE"]).
5. Ran the full precheck.precheck() path against a real B-plan file end-to-end (not just
   the unit tests) and, from that live run, found and fixed a genuine defect: _run_checker's
   original fail-reason extraction took the checker's LAST STDOUT LINE, which under --json
   output is just the closing "}" -- useless for a human reading the DONE-gate's evidence.
   Replaced with _fail_detail(), which parses the checker's own JSON payload (its
   incomplete/notes list, or a trigger sub-check's FAIL status) and falls back to the last
   line only when the payload isn't the expected shape. Verified against 3 fixture payloads
   (a real wiring_contract_check.py FAIL, a production_caller_check.py ERROR, plain-text
   fallback) -- all three produce a materially better reason string.

FILES TOUCHED
- /home/user/teikoku/conductor/precheck.py (NEW, 242 lines)
- /home/user/teikoku/conductor/backlog_import.py (MODIFIED, 648 -> 704 lines: import
  precheck L25; --no-precheck flag L128-134; load_precheck_seeds L397-420 + 
  load_probe_pending_warnings L423-433; main() wiring L653-667)
- /home/user/teikoku/conductor/tests/test_precheck.py (NEW, 207 lines, 15 tests)
- git status --short in the teikoku repo confirms exactly these three paths changed
  (M backlog_import.py, ?? precheck.py, ?? tests/test_precheck.py); nothing else touched.

DEVIATIONS FROM THE BLOCKED DESIGN (all grounded in live measurement/source, not vibes)
1. Timeouts swapped vs the design's proposal, using gate_packs.py's OWN proven per-checker
   values plus live measurement instead of a cross-system inference: TRIGGER_TIMEOUT_S=90
   (matches gate_packs.py's proven wiring_contract_check timeout exactly; measured ~41s
   real) and STATIC_GATE_TIMEOUT_S=240 (RAISED above both the design's 120 and
   gate_packs.py's own 120, because gate_packs.py's 120s is calibrated for its fast
   distro-ext4 per-task-worktree audit-time call, while precheck() necessarily runs against
   the live /mnt/f 9p mount at import time -- no worktree exists yet -- and I measured
   137.8s real for production_caller_check.py with only 2 symbols over that path).
2. Added _leaf_symbol_for_scan (not in the blocked design): produces_symbols entries can be
   qualified ("crates/x.rs::name", confirmed live on B-9's real contract), and
   production_caller_check.py's --symbols mode is a literal-name scan with no file-scoping,
   so an unstripped qualified entry would never match a real call site.
3. Added _fail_detail (not in the blocked design, found only by running the real path
   end-to-end -- see WORK DONE item 5).

GROUNDING / VERIFICATION
- python3 -m py_compile on all three files: clean, zero warnings (also fixed one
  incidental SyntaxWarning: invalid escape sequence, from a Windows-path example in
  precheck.py's own module docstring -- reworded to forward slashes).
- python3 -m pytest tests/test_precheck.py tests/test_backlog_adapter_general.py
  tests/test_backlog_import.py -q: 24 passed, 0 failed, 0 regressions (9 pre-existing +
  15 new).
- python3 backlog_import.py --no-precheck --dry-run --json against the REAL ELAI corpus:
  1988 tasks parsed in 4.457s real, confirming the escape hatch and the CLI wiring both
  work end-to-end against production data.
- One real precheck.precheck() call against docs/thefinalglance/B-plan/
  B-9_sessiondag_activation.md (repo_root=/mnt/f/.ELAI_workflow, the real project.repo_root
  for the elai project): 201.07s real, correctly extracted 8 produces_symbols (mix of bare
  and qualified forms), correctly returned EXECUTE (not a false CERTIFY) because the
  contract's trigger script invokes bare "python" which this WSL distro does not have
  (only python3 -- confirmed separately: `which python3 pytest` finds python3 at
  /usr/bin/python3, no plain `python`). This is the fail-closed behavior working as
  designed, not a defect in precheck.py.
- production_caller_check.py leg of that same real call: PASS (all 8 symbols, after
  leaf-stripping, resolved a real caller via elai.map).
- python_wire_check.py (the D-030 off-agent gate this rule mandates I self-run): could
  NOT render a verdict for 7 of 8 new public symbols when pointed at
  --root=/home/user/teikoku/conductor. Root-caused, not hand-waved: that directory
  co-locates a Python virtualenv (.memory-venv/, 11845 of the tree's 12005 .py files --
  98.7%) which the checker's AST-based caller-scanner walks into unconditionally (no
  include/exclude flag exists), and at least one third-party package's source triggers an
  unhandled RecursionError with unclear exit-code propagation (a REAL bug in this frozen,
  out-of-scope-for-me tool, reproduced 3x, confirmed NAME-INDEPENDENT via a deliberately
  nonexistent symbol). One symbol DID get a clean verdict before hitting this: "precheck"
  -> WIRED/PASS with real evidence (backlog_import.py:25 the import, :417 the call site).
  I did not attempt to fix, route around, or silently skip this (out of scope, and the
  checker is R8-frozen); I did NOT claim a false PASS for the other 7 symbols. Direct source
  read independently confirms load_precheck_seeds/load_probe_pending_warnings are called
  from backlog_import.py's own main() (the real CLI entrypoint, `if __name__ ==
  "__main__": raise SystemExit(main())`), non-test, same file -- and the 15-test suite
  exercises every new symbol with real assertions. --files mode was NOT usable at all for
  this cross-repo target: python_wire_check.py's WORKFLOW_ROOT is hardcoded to
  Path(__file__).resolve().parents[2] (the ELAI /mnt/f/.ELAI_workflow tree) with no env
  override (unlike wiring_contract_check.py's ELAI_MAP/ELAI_PROJECT_ROOT), so
  path.relative_to(WORKFLOW_ROOT) raises for any file outside that tree; --symbols mode
  (matching gate_packs.py's own proven cross-repo pattern for the Rust sibling) was the
  only usable invocation shape.

KNOWN LIMIT carried over from the blocked design, unchanged: certifying the lead task DONE
does not short-circuit its 3 followup tasks (INTEGRATE/AUDIT/POLISH), which always get
created at status PLANNED. Out of scope for this unit's named PRODUCTION_HOOK
(build_task's seeds consumption only).

NEW OBSERVATION for the orchestrator to weigh (not a blocker, informational): with
--no-precheck OFF (the default), a full backlog import will run precheck.precheck() once
per non-empty-contract plan not already seeded DONE by FINISH/bn-status. Measured cost per
call: roughly 40-200s depending on whether the produces_symbols leg fires. Across the ~400
non-phase B-K plans (many carry contracts), a from-scratch default import could take a long
time (tens of minutes to a few hours) the first time it runs without prior FINISH/bn-status
coverage. --no-precheck is the already-shipped mitigation; a future follow-up could add a
persistent precheck-result cache keyed by plan content hash, or run the live legs
concurrently -- flagging as a candidate, not implementing (out of scope for this unit).

HOW TO PICK UP
- Nothing pending on this unit; it is DONE per the STRUCTURED RETURN below.
- If continuing the TEIKOKU-ELAI-FINISH-PLAN.md sequence, unit 1.1's PRODUCTION_HOOK
  (backlog_import.py's seeds consumption at build_tasks()) is now live; the next unit can
  build on load_precheck_seeds/load_probe_pending_warnings existing.
- If someone wants the python_wire_check.py RecursionError fixed, that is a separate,
  out-of-scope, ELAI-harness-tool fix (add a venv/worktree exclusion to its file-walk) --
  not a teikoku-conductor change, and the checker is R8-frozen (needs explicit human-
  reviewed change per wiring-enforcement.md R8).

STATUS: DONE
FILES TOUCHED: see FILES TOUCHED above.
VERIFIED: py_compile clean x3; pytest 24/24 (15 new + 9 pre-existing, 0 regressions);
--no-precheck --dry-run against real corpus (1988 tasks, 4.46s); one real precheck.precheck()
call end-to-end (201.07s, correct EXECUTE verdict with a real, understood reason);
python_wire_check.py run (1/8 symbols got a clean WIRED verdict before an unrelated frozen-
tool RecursionError blocked the rest -- root-caused to a co-located venv in --root, reported
above, not silently passed over).
QUESTION: none (not blocked).
NOTES: (1) the --files vs --symbols invocation-shape finding for python_wire_check.py may be
useful to whoever next runs this checker against ANY non-ELAI-native project root. (2) the
precheck latency finding above may warrant a follow-up unit. (3) all fixes here were applied
via a Python-mediated read/replace/write pattern rather than raw bash heredocs after
discovering the PowerShell-to-wsl.exe-to-bash pipeline silently mangles literal backtick and
multi-backslash sequences inside `cat > file <<'EOF'` heredoc bodies (command substitution
fires despite the quoted delimiter); noting this here in case it recurs for the next WSL-
cross-project dispatch.
