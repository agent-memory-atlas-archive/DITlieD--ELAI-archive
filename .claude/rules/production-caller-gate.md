Production-caller gate (D-030, HARD RULE, fail-closed, off-agent enforced)

Loaded by: any orchestrating skill that dispatches an executor (orchestrate, auto-orchestrate, b-plan-runner, paper-to-plan), any executor authoring new pub Rust symbols, and any auditor judging an executor's changeset for "done". Pairs with wiring-enforcement.md (this is its R6/R7 producer-without-consumer leg made runnable at changeset grain) and tracing-test-assert.md (the Layer-4 runtime backstop). Three orthogonal axes, all auditor-applied.

WHY THIS EXISTS

The elai-finish-sweep run (session s-107b022a, 2026-06-11..13) hit the SAME failure mode six times in one stretch: I-34 (hook pending), I-10 (BLOCK), I-8 (BLOCK), I-1 (BLOCK), I-32 (empty-store BLOCK), I-41 (dead framing surface). The shape is always identical: the executor builds the machinery AND its unit test in the same changeset, the unit test calls the new fn directly and goes green, but NO production entrypoint ever calls it. The hostile auditor catches it as an R7 zero-production-callers BLOCK and burns a full fix cycle re-wiring it. It is the single dominant failure mode of LLM executors on this codebase: the executor takes the shortest path that makes its own test pass, and a unit test calling the function directly is indistinguishable from a missing caller (wiring-enforcement.md, Berkeley RDI Apr-2026: in-boundary artifacts gamed ~100%).

Prompt pleas were already added across the dispatch templates and only REDUCED the rate, never eliminated it (handoff elai-finish-sweep.md CHECKPOINT 2). An LLM cannot be reliably instructed out of its shortest-path bias; the fix has to be STRUCTURAL and off-agent. D-030 adopts both legs the checkpoint proposed: a deterministic changeset-grain checker (leg A, the off-agent backstop that cannot be rationalized away) and a dispatch-template PRODUCTION_HOOK contract (leg B, prevention at dispatch time).

LEG A: THE DETERMINISTIC CHECKER (off-agent, frozen, fail-closed)

.claude/tools/production_caller_check.py takes the changeset's .rs files, derives the NEW pub symbols (git-diff-added lines for tracked files, all pub symbols for untracked/ignored files), and FAILS unless each NEW pub fn has at least one non-test, non-import reference OUTSIDE its defining file. It is a Layer-2 static, regex-unsound, necessary-not-sufficient leg (name-grep is not a call graph, R5) -- the load-bearing proof remains the Layer-3/4 runtime evidence. What it guarantees is that a production-LOOKING caller EXISTS lexically, early enough for the executor to fix in the same changeset instead of after a wasted audit cycle.

Verdicts: WIRED (>=1 external production ref), INTERNAL (no external ref but a non-test same-file ref -- a transitive helper; WARN, auditor verifies the chain reaches a wired entrypoint), DARK (a new pub FN with zero production refs -> BLOCK), DARK-TYPE (a new pub type used only in its own file -> WARN, types legitimately can be), REGISTERED-DARK (a WIRE-DARK[plan-id] marker sits within the comment/attribute block directly above the def -> WARN, auditor confirms the matching stubs.md / wire-dark-allowlist.toml row), UNSOUND-NAME (an fn whose name is too common for name-grep -- new/run/get/.. or under 4 chars -> WARN, auditor resolves via codegraph_callers).

Exit: 0 clean (WARNs allowed), 1 on any DARK fn (BLOCK), 2 fail-closed on a scan-config error (git unreachable, missing input, --expect-symbols finding zero new pub symbols, any unhandled exception). There is NO allowlist flag and NO static-only opt-out (R3): the only escape is the in-code WIRE-DARK[..] marker, which is visible in the diff and auditable.

Usage:
  python .claude/tools/production_caller_check.py --files <changed.rs,...> [--json]
  python .claude/tools/production_caller_check.py --symbols Foo,bar --json   (explicit-symbol, weaker grain)
  python .claude/tools/python_wire_check.py --files <changed.py,...> [--json]  (PYTHON sibling: AST-based, same verdicts/exit codes/WIRE-DARK[..] escape; v1 = module-level public defs only, class methods + dynamic/string-keyed dispatch are residual risk on the runtime leg. Use this for .py changesets; production_caller_check.py is .rs only.)

LEG B: THE PRODUCTION_HOOK DISPATCH CONTRACT (prevention at dispatch time)

PCG-1 (explorer): every integration-explorer-elai dispatch whose sub-phase ships a new pub fn MUST return a structured PRODUCTION_HOOK field naming the EXACT production seam -- file path + fn (and line if known) -- where the new machinery will be called from a real entrypoint (CLI subcommand, IPC handler, FSM transition, task_runner dispatch, hook, SSB emit site). "needs wiring" is not a seam; an unnamed seam is a BLOCKED explorer return. This is the wiring-enforcement planning-time PRE_GATE (vertical-slice rule) enforced at dispatch grain.

PCG-2 (executor): the executor prompt template carries the PRODUCTION_HOOK as a NUMBERED DELIVERABLE: "wire <new fn> into <seam> in the SAME changeset". An executor that ships the producer without its consumer-and-smoke in the same changeset returns BLOCKED (it does not silently defer the wiring to a future step). Before returning DONE, the executor runs production_caller_check.py --files <its changeset> as a mandatory self-verify; a DARK fn it cannot wire is a BLOCKED return with the seam question, never a green DONE.

PCG-3 (auditor): the auditor RE-RUNS production_caller_check.py over the changeset off-agent (the executor's self-run is necessary-not-sufficient; the executor that wrote the code cannot be the authority on whether it wired it, same in-boundary failure freeze-the-gate exists to prevent). Any DARK fn without a WIRE-DARK row is a BLOCK in the audit verdict. The checker is frozen in .claude/gate-manifest.txt (R8) so an executing run cannot weaken it.

THE LEGITIMATE-DARK ESCAPE (the only non-BLOCK path for an unwired new fn)

A new pub fn that is INTENDED to ship dark this changeset (a kill-switch-gated path, an observability accessor read only by a dashboard/boot warn not yet wired, a producer whose consumer lands in a named later step) carries an in-code marker on the line directly above its definition:
  // WIRE-DARK[<plan-id>]: <one line: why dark + where the consumer lands>
AND a matching row in workflow/sessions/<sid>/stubs.md (with will_resolve_in) OR .elai_cc/wire-dark-allowlist.toml (with type/owner/expiry/promotion_plan_id per wiring-enforcement R9). The marker downgrades BLOCK to REGISTERED-DARK (WARN); the auditor confirms the row exists. A marker with no backing row is still a finding.

KNOWN TRUE-POSITIVE ON THE CURRENT TREE (the checker's first real catch, 2026-06-13)
crates/tools/src/prompt_hardening.rs::dropped_row_count (the I-47 scream-path accessor) is DARK: its only non-test readers are a comment and the i47 contract. It is either a genuine Stage-5 punch item (wire the boot-warn to actually read it) or a legitimate observability readable needing a WIRE-DARK marker. Routed to the controller's Stage-5 punch list, NOT fixed under the D-030 pause. This is the gate working as designed on its first production run.

ROLE BOUNDARY
This is the AUDITOR's gate (and the executor's mandatory self-verify). The main LLM never renders the production-caller verdict and never runs the checker as a pass on code it wrote (consistent with "main model never runs tests"). The checker is an auditor-invoked, frozen artifact.

SELF-CHECK before declaring any executor changeset DONE
1. Does every new pub fn have a non-test caller OUTSIDE its own file, or a WIRE-DARK[..] marker + backing row?
2. Did the explorer name the exact PRODUCTION_HOOK seam, and did the executor wire it in THIS changeset?
3. Did the auditor RE-RUN production_caller_check.py off-agent (not trust the executor's self-run)?
If any answer is no, the machinery is wire-dark -- it is not done.
