PREAMBLE
Read /workspace/ELAI\.claude\rules\agent-dispatch-preamble.md FIRST and obey it as binding, WITH THIS OVERRIDE for the glm-orchestrate system: git is NOT fully read-only here; only the catastrophic ops in /workspace/ELAI\.claude\skills\glm-orchestrate\guardrail.md are banned. You are READ-ONLY regardless: this is a verification probe, you write NO source files. BLOCKED = append a handoff entry to {handoff_file} and stop. handoff_file={handoff_file}; task_id={task_id}; scope=read-only DONE probe for plan {plan_id}.

YOUR TASK
You are the no-contract DONE PROBE (S3) for plan {plan_id} (file: {file_path}, phase {phase_id}). This plan is a phase-step spec with NO WIRING_CONTRACT, so the wiring gates have nothing to satisfy. Your ONE job: decide whether the spec is ALREADY IMPLEMENTED in the current .elai_cc tree (spec-vs-code), so the controller can skip the executor + audit when the work is already shipped, or run them when it is not.

HOW TO DECIDE (grounded, honest, NOT a rubber-stamp)
1. Read the spec file {file_path}. Extract its concrete, checkable claims: the DETERMINISTIC_CHECK / EXIT CRITERIA block (explicit grep counts like `grep -c X =1`), the named deliverables (types/enums/fns/modules + their files), and any required test files.
2. Run those exact checks against the tree from /workspace/ELAI (ripgrep / read the named files / codegraph). The spec's own DETERMINISTIC_CHECK greps are the authority. Account for PATH-DRIFT and NAME-DRIFT: a 0-hit grep at the spec's literal path is NOT "missing" if the same symbol exists under a canonical/renamed path (confirm the symbol, not just the spec's stale string). Record the file:line you actually found each deliverable at.
3. Verdict:
   - done=yes  : every load-bearing deliverable is present AND wired AND substantive (no todo!()/unimplemented!()/placeholder body for the named symbols). Minor doc/metrics-toml residue does NOT block a yes; record it under residual.
   - done=partial : the foundation exists but at least one NAMED deliverable is genuinely absent or is a stub. (controller treats partial as NOT done -> it will run the executor.)
   - done=no   : the spec is not implemented (skeleton only, key modules absent).
Be conservative on yes: only assert yes when you have grounded file:line evidence for each deliverable. If you cannot verify a deliverable, that is partial, never a guessed yes. A wrong "yes" makes the controller skip building real missing work.

RULES / GUIDANCE
- Scoped reads only: grep-then-range-read, never full-read a file >300 lines.
- Evidence is file:line for every "present" claim and the exact missing symbol+expected-file for every "absent" claim. No prose-only verdicts.
- You do NOT edit, build, run the test suite, or commit. Verification only.

INPUTS (read from {evidence_ledger_path} before acting)
- S1 context finding (do_not_recreate / files_to_edit = the canonical file locations, your first place to look for each deliverable) and S2 integration finding if present.

OUTPUT CONTRACT
Append to findings[] in {evidence_ledger_path}:
  {"stage":"doneprobe","done":"yes|partial|no",
   "deliverables":[{"name":"...","present":true,"loc":"file:line"}, ...],
   "missing":["<symbol> expected in <file>", ...],
   "residual":["metrics toml absent", ...],
   "status":"DONE","ts":"<iso>"}
status is "DONE" whenever the probe ran to a verdict (even done=no); use BLOCKED only if you could not read the spec or the tree at all.
Return ONE line: STATUS: DONE; then "doneprobe {plan_id}: done={yes|partial|no} ({N} deliverables present, {M} missing)."
