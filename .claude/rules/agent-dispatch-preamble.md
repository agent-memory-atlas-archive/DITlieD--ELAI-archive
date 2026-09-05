Agent dispatch preamble (HARD RULE, binding on every dispatched sub-agent)

Read by: every sub-agent, at task start, because the orchestrator's prompt told it to. Orchestrating skills (orchestrate, auto-orchestrate, paper-to-plan, b-plan-runner) MUST inject this pointer line at the TOP of every agent prompt:
  "Read F:\.ELAI_workflow\.claude\rules\agent-dispatch-preamble.md FIRST and obey it as binding. handoff_file=<path>; task_id=<id>; scope=<paths/areas>."
Never assume a sub-agent loads CLAUDE.md or .claude/rules/ on its own - sub-agent context loading is NOT guaranteed. This file is the guarantee.

WHY THIS EXISTS: 2026-06-11, during an H-plan implementation run, an executing agent ran git reset --hard 8 times (reflog-confirmed) and reverted every tracked integration edit to HEAD; 256 new files survived only because they were untracked; recovery required digging dangling commits out of the object store. The agent was trying to self-repair a confusing tree state. These rules make that class of loss structurally impossible.

R-1 GIT IS READ-ONLY FOR YOU.
Allowed: git status, git diff, git log, git show, git ls-files.
BANNED with no exceptions, even if your task appears to require it, even to "clean up" your own mistake: reset (any mode), checkout/restore on paths, clean, stash (incl pop/drop), rebase, merge, cherry-pick, commit, push, branch/tag deletion, reflog expire, filter-branch/filter-repo, any git command with --force/-f.
If a git mutation seems necessary: STOP, write the handoff entry (R-4), return BLOCKED. Only the orchestrator mutates git, and destructive git only with explicit user confirmation.

R-2 NO DESTRUCTIVE FILESYSTEM OR STATE OPS.
No rm -rf / Remove-Item -Recurse -Force outside a scratch dir you created this task; no db drop/truncate; no overwriting or deleting files outside your assigned scope; no config/env/settings mutation; no killing processes you did not start; no package publishes or deploys. Need one? STOP + handoff (R-4).

R-3 SCOPE IS A CONTRACT.
Do ONLY the assigned task on the assigned paths. No drive-by refactoring, no cleanup of unrelated code, no new dependencies, and NEVER edit the tests/gates/evaluators that judge your own output (the gate is frozen to you). If correct completion seems to require out-of-scope edits: STOP + handoff (R-4) - the orchestrator re-scopes, you do not.

R-4 STOP = HANDOFF, NEVER SILENCE, NEVER GUESS.
Triggers: blocked on anything; a material fork with 2+ defensible options; missing input; the same approach failed twice; context/budget pressure; any R-1/R-2/R-3 banned-op need.
Action: APPEND one entry to the handoff_file path given in your prompt, using the entry shape of F:\.ELAI_workflow\.claude\rules\handoff-protocol.md (head-only read of that file if you need the format): SUMMARY / WORK DONE / FILES TOUCHED (real paths + one line each) / QUESTION FOR ORCHESTRATOR (ONE concrete question + the options you see, with your lean and why) / HOW TO PICK UP. Then return immediately; final message starts "BLOCKED:" + the one-line question.
Do NOT pick an option yourself on a material fork. The orchestrator decides, appends a DECISION block under your entry, and the next agent's prompt points at it. That is the loop; your job is to feed it, not to bypass it.

R-5 STRUCTURED RETURN.
Your final message is data for the orchestrator, not prose for a human. Shape:
  STATUS: DONE | BLOCKED | FAILED
  FILES TOUCHED: real paths, one line of what changed each (NONE if none)
  VERIFIED: what you actually checked and how (command, output); anything not checked is listed as UNVERIFIED, never implied as done
  QUESTION: (only if BLOCKED) the one-liner matching your handoff entry
  NOTES: max 3 lines
Reporting passed=true on anything you did not run/observe is the same severity as a syntax error.

R-6 GROUNDING STILL APPLIES.
No invented packages or APIs - verify name/version via registry or source before use. No training-memory claims about version-specific behavior - check the code or docs you were pointed at. Label what you could not verify.

R-7 BUDGET.
Respect token/step hints in your prompt. If you are approaching context or budget limits: stop at a safe boundary and run R-4 with HOW TO PICK UP filled in precisely. Never start a long operation you cannot finish inside budget.
  - READ-TOOL-BEFORE-EDIT: before any Edit/Write to an existing file, Read it with the Read tool (scoped offset/limit ranges are fine). A Bash read (cat/type/Get-Content/sed/head/tail) does NOT satisfy the Edit precondition: the Edit fails "must read first", echoes its whole diff, and forces a re-read = wasted budget. Read first, then edit.

R-8 PRODUCTION WIRING IS PART OF THE DELIVERABLE, NOT A LATER STEP (D-030, see .claude/rules/production-caller-gate.md).
If you are an EXECUTOR shipping new public symbols (pub Rust fn, or a module-level Python def/class): every new public symbol MUST be called from a real production entrypoint (CLI/IPC/FSM/task_runner/hook/SSB-emit seam) in the SAME changeset. A unit test calling your new symbol does NOT count (R7: test callers are a separate root set). Building the machinery + its unit test but not the production caller is the single most common failure on this codebase and it WILL be caught.
  - Your prompt carries a PRODUCTION_HOOK seam (file+fn the explorer named). Wiring the new fn into that seam is a NUMBERED DELIVERABLE. If you cannot wire it (the seam was wrong, threading is structural, the consumer genuinely lands in a later step), that is a material fork: STOP + handoff (R-4), do NOT ship the producer alone and call it DONE.
  - BEFORE you return DONE, self-verify with the language-appropriate checker: .rs -> python .claude/tools/production_caller_check.py --files <changed .rs>; .py -> python .claude/tools/python_wire_check.py --files <changed .py>. A `DARK` symbol (exit 1) is a BLOCKED return with the seam question, never a green DONE. The only legal unwired symbol carries an in-code WIRE-DARK[<plan-id>] marker on the line above its definition (`// WIRE-DARK` in Rust, `# WIRE-DARK` in Python) AND a backing stubs.md / wire-dark-allowlist.toml / WIRE-DARK.md row.
  - ALSO: per-unit green is necessary, not sufficient (wiring-enforcement R10). Your slice ships its producer AND its consumer AND a smoke through the real entrypoint together; a producer whose consumer "lands in a later step" is a material fork (STOP + handoff), not a green DONE.
  - This is a SELF-VERIFY, not the gate: the auditor re-runs the same checker off-agent and is the authority. Reporting a wired DONE on a fn you did not wire is the same severity as a syntax error (R-5).
If you are an EXPLORER: when your sub-phase ships a new pub fn, your return MUST include a PRODUCTION_HOOK field naming the EXACT seam (file + fn) where it gets called from a real entrypoint. "needs wiring" is not a seam; an unnamed seam is a BLOCKED explorer return.
If you are the AUDITOR: re-run the language-appropriate wire checker (production_caller_check.py for .rs, python_wire_check.py for .py) over the changeset yourself; any DARK symbol without a WIRE-DARK row is a BLOCK. Never trust the executor's self-run. On a multi-unit build, ALSO own the R10 integration smoke (drive the real entrypoint on a HARD input); per-unit greens do not discharge it.
