SP-1A executor (unit 1.1, TEIKOKU-ELAI-FINISH-PLAN.md) - BLOCKED before any file write

session: SP-1A-exec, FSM state EXECUTING (mcp_fsm_get_state: Phase "SP-1A precheck
routing port (unit 1.1)", Attempts 0/3, allowed transitions: AUDITING only)

BLOCKER (hard tool denial, not a design ambiguity)
The PreToolUse hook for this EXECUTING state gates Write/Edit to path prefixes
"src/, tests/, memory-bank/*.md, workflow/reasoning/" ONLY (this file itself proves
workflow/reasoning/ is allowed) and gates Bash out entirely ("FSM state is EXECUTING.
Bash blocked. Write implementation code; the auditor runs tests."). This dispatch's
target is the teikoku conductor over the WSL UNC bridge
(\\wsl.localhost\teikoku\home\teikoku\teikoku\conductor\...), which matches NONE of
the four allowed prefixes (a UNC path never starts with "src/" or "tests/" as the
hook checks it, even though the target path DOES contain a "tests\" component
partway through). Confirmed by direct attempts, each returned the identical hook
denial verbatim:
  1. Write \\wsl.localhost\teikoku\home\teikoku\teikoku\conductor\precheck.py -> BLOCKED
  2. Write \\wsl.localhost\teikoku\home\teikoku\teikoku\conductor\tests\test_precheck.py -> BLOCKED
     (despite containing "tests\" -- the gate checks path PREFIX, not substring)
  3. Edit F:\.ELAI_workflow\workflow\handoff\teikoku-elai-finish.md -> BLOCKED
     ("workflow/reasoning/" is allowed; "workflow/handoff/" -- my assigned
     handoff_file path -- is NOT, so even the R-4 handoff-entry channel is closed)
Bash (wsl -d teikoku ...) is separately and unconditionally blocked in this FSM
state regardless of path ("Bash blocked... the auditor runs tests"), so there is no
fallback route (heredoc-via-Bash) either. My dispatch prompt asserted "FSM is in
EXECUTING so Bash is unblocked for you" -- that assertion does not match this
session's actual hook behavior; flagging as a DRIFT for whoever wrote the dispatch
prompt, not a mistake on my part.

Net effect: I can write files under F:\.ELAI_workflow\workflow\reasoning\ (this file
is the proof) but CANNOT write or edit anything under the teikoku conductor tree,
and cannot append the required handoff entry at its assigned path either. Every one
of my 5 numbered deliverables (tests, precheck.py, backlog_import.py wiring,
DONE-probe seam, self-verify) requires a Write/Edit under conductor/, so ALL FIVE
are blocked at the tool layer, not by any remaining design question.

WORK ACTUALLY DONE THIS SESSION (research only, nothing written to the target repo)
Read, in full, everything needed to implement unit 1.1 correctly on the first
attempt once unblocked:
  - context-package-SP-1A.md + exploration/SP-1A-conductor.md (the mandatory inputs)
  - workflow/glm-orchestrator/gates.py (full), orchestrator.py (imports/ROUTE/combiners),
    plan_parser.py (full, for the produces_symbols/reachable_from loose-YAML grammar)
  - .claude/skills/glm-orchestrate/prompts/wiring-backend.md +
    .claude/agents/wiring-backend.md (the ACTUAL proven invocation recipe)
  - .claude/tools/wiring_contract_check.py (main(), full exit-code logic) +
    production_caller_check.py (argparse only)
  - teikoku conductor sources over the UNC bridge: backlog_import.py (full),
    conductor_db.py (ControlDB: init/transaction/upsert_task/update_task_status/
    ready_task_rows/task_counts/deterministic_task_dump), models.py (TaskSpec/
    TASK_STATUSES), waves.py (full), project_resolver.py (full), projects.toml
    (full), gate_packs.py (_rust_pack_commands, the wiring_contract_check +
    production_caller_check invocation shapes), gate_runner.py (GateContext,
    DEFAULT_REPO_ROOT/DEFAULT_WORKFLOW_ROOT), tests/helpers.py (TempDirMixin),
    tests/test_backlog_adapter_general.py (the test-style template to match)

TWO GENUINE FINDINGS FROM THAT READING (both would have been silent correctness
bugs if I had not verified source before writing code -- recording them here so the
next agent does not have to re-derive them)

FINDING 1 -- gates.py's own precheck() invocation of wiring_contract_check.py
--static-only can NEVER certify a real plan. Verified directly in
wiring_contract_check.py main(): when the contract has an observable_effect block
(R1 requires every complete contract to have one) and --static-only is passed, the
script sets a non-empty static_note and `failed = (... or bool(static_note) or ...)`
forces the verdict to FAIL unconditionally -- the script's own --static-only help
string says "a runtime contract can then never PASS in static-only mode". This means
gates.py's literal precheck() recipe was never actually exercised in production;
orchestrator.py's own top comment confirms the real ~70-commit loop routed through
the wiring-backend/wiring-frontend GLM agents instead. Those agents' concrete
definition (.claude/agents/wiring-backend.md leg 3) invokes the SAME checker WITHOUT
--static-only (the runtime trigger runs unconditionally per R3: --run-trigger is a
documented no-op now, "the trigger always runs" regardless of the flag). My planned
precheck.py ports THAT proven recipe (no --static-only, TRIGGER_TIMEOUT_S=240s
matching gates.py's own done_gate() timeout for a trigger-bearing call), not
gates.py's literal (dead) invocation. This was flagged as a candidate material fork
in my own reasoning before I found gate_packs.py's precedent (below) settled it --
recording the resolution here rather than a live handoff question since it is a
verified bug-fix, not an open design choice.

FINDING 2 -- both checkers need EXPLICIT --project-root/--map/--root when invoked
from WSL. wiring_contract_check.py's DEFAULT_PROJECT_ROOT and DEFAULT_MAP, and
production_caller_check.py's default --root, are all Windows-absolute
(r"F:\.ELAI_workflow\..."), which do not resolve inside the WSL filesystem
namespace -- silently breaking every symbol lookup if left at defaults (the
checkers would likely exit non-zero for "map/root not found" reasons that read as
a genuine FAIL, meaning CERTIFY would never fire for ANY plan, ever -- shipped-but-
inert machinery). gate_packs.py's own _rust_pack_commands (the conductor's existing,
ALREADY-PROVEN cross-boundary invocation of these exact scripts for the real
done-gate) always passes --map/--project-root/--root explicitly rather than relying
on the scripts' own defaults; my planned precheck.py mirrors that pattern exactly:
--project-root / --root = <repo_root>/.elai_cc (matching DEFAULT_PROJECT_ROOT's own
convention, i.e. project_resolver.ResolvedProject.repo_root + "/.elai_cc"), --map =
<repo_root>/.claude/tools/elai.map.

FULL DESIGN (ready to type in verbatim once Write/Edit is unblocked on conductor/)

conductor/precheck.py (NEW, self-contained, no backlog_import import -- avoids a
circular import since backlog_import will import precheck):
  - constants CERTIFY / EXECUTE / PROBE_PENDING, STATIC_GATE_TIMEOUT_S=120,
    TRIGGER_TIMEOUT_S=240
  - LegResult / PrecheckResult (with .evidence_line()) / ProbeResult dataclasses
  - _tools_dir(repo_root), _elai_cc_dir(repo_root), _py(...), _run_checker(...)
    (never raises; subprocess errors/timeouts become a FAIL LegResult)
  - extract_symbol_list(wiring_contract, key) -> list[str] (mirrors backlog_import.
    extract_production_hook's regex idiom, widened to return every '- item' line,
    not just the first, since produces_symbols needs the full list)
  - precheck(plan_id, plan_file, wiring_contract, repo_root, runner=_run_checker)
    -> PrecheckResult: empty wiring_contract -> PROBE_PENDING; else runs
    wiring_contract_check.py --bn-file <plan_file> --map <map> --project-root
    <elai_cc> --json (cwd=elai_cc, timeout=TRIGGER_TIMEOUT_S), and when
    produces_symbols is non-empty also production_caller_check.py --symbols
    <first 40, comma-sep> --root <elai_cc> --json (cwd=elai_cc,
    timeout=STATIC_GATE_TIMEOUT_S); all-PASS -> CERTIFY, else -> EXECUTE with a
    reason string naming which leg(s) failed
  - done_probe(plan_id, plan_file, provider=None) -> ProbeResult: provider=None
    (today, always) -> PROBE_PENDING + warn "no probe lane bound; routed READY";
    provider!=None -> raise NotImplementedError (SP-2A scope, never fake a result)

conductor/backlog_import.py (EDIT, 4 changes):
  1. add `import precheck` near the other sibling imports (module import, NOT
     `from precheck import precheck`, so tests can monkeypatch precheck.precheck)
  2. parse_args(): add `--no-precheck` (store_true) escape flag near --use-bn-status
     (default False -- the DEFAULT import path runs precheck; a default-off precheck
     would itself be wire-dark per D-030)
  3. two new module functions, placed after load_bn_statuses (~line 386) and before
     build_task (~line 389):
       load_precheck_seeds(plans, existing_seeds, repo_root) -> dict[str, StatusSeed]:
         for each plan with a non-empty wiring_contract, SKIP if existing_seeds
         already has that plan_id at status=="DONE" (a stronger existing seed always
         wins -- this can only ADD a DONE it can prove, never remove/downgrade one,
         and skipping saves a redundant live-trigger subprocess call); else call
         precheck.precheck(...) and on CERTIFY return
         StatusSeed("DONE", "PRECHECK_CERTIFIED", result.evidence_line()) for that
         plan_id (caller does seeds.update(...), which is safe because certify-only
         keys ever appear here)
       load_probe_pending_warnings(plans) -> list[str]: for each plan with an EMPTY
         wiring_contract, call precheck.done_probe(...) and collect one
         "{plan_id}: PROBE_PENDING: {warn}" line per PROBE_PENDING result, for the
         import report (visibility, never a silent skip)
  4. main(): between the bn_status setdefault loop (ends ~605) and
     `task_rows, deps = build_tasks(...)` (~607), gated behind `if is_rust and not
     args.no_precheck:` (matching the existing is_rust gating on load_status_seeds/
     load_bn_statuses -- these checkers are ELAI/rust-pack-specific, same boundary
     as today): call load_precheck_seeds, seeds.update(its result), then
     load_probe_pending_warnings, and fold any warnings into report["probe_pending"]
     for visibility in the JSON/dry-run output.

conductor/tests/test_precheck.py (NEW; unittest.TestCase + tests.helpers.TempDirMixin,
matching test_backlog_adapter_general.py's existing style exactly -- full source
already drafted, ready to paste, covering all 5 named scenarios (a)-(e) plus the
seed-precedence and probe-warning integration paths):
  PrecheckFunctionTests: contract-passing->CERTIFY, contract-failing->EXECUTE,
    no-contract->PROBE_PENDING (+done_probe WARN, +provider-before-SP2A raises),
    checker-invocation-error->EXECUTE-never-CERTIFY, extract_symbol_list correctness
  PrecheckSeedMergeTests (TempDirMixin): existing-DONE seed never overwritten
    (precheck.precheck patched to raise if called, proving the skip-optimization),
    a weaker existing seed (READY) gets upgraded to DONE on CERTIFY, a
    contract-failing plan gets no precheck seed at all, probe-pending warnings
    collected for no-contract plans, and the (e) frontier test: build a DONE-seeded
    TaskSpec via build_tasks, import_rows into a real tmp ControlDB, assert the lead
    task_id is absent from db.ready_task_rows() and db.task_counts()["DONE"] == 1

KNOWN LIMIT flagged for the record (not fixed, out of scope for the named
PRODUCTION_HOOK): certifying the LEAD task DONE does not also short-circuit its 3
followup tasks (INTEGRATE/AUDIT/POLISH from build_wave_followup_task), which always
get created at status PLANNED regardless of the lead's status. Whether a PLANNED
task with satisfied deps is ever promoted to READY by scheduler.py (not read this
pass, exploration doc marked it UNVERIFIED/WARN-only too) is orthogonal to this
fix's correctness: ready_task_rows's WHERE status='READY' filter already makes the
DONE lead task itself unleashable regardless. Not touched, per R-3 scope (the named
PRODUCTION_HOOK is build_task's seeds consumption only).

SELF-VERIFY (deliverable 5) -- NONE OF IT COULD RUN
py_compile / the new unit tests / python_wire_check.py all require Bash, which is
unconditionally blocked in this FSM state. Nothing here has been executed or
observed; treat the whole design above as UNVERIFIED-BY-EXECUTION even though it
was verified BY READING every piece of source it touches.

QUESTION FOR ORCHESTRATOR
How should I proceed given Write/Edit is scoped to src/, tests/, memory-bank/*.md,
workflow/reasoning/ and Bash is fully blocked in EXECUTING for this session -- none
of which reach the teikoku conductor tree? Options, with my lean:
  1. (my lean) Transition/re-dispatch with the FSM hook's allowed-path set widened
     to include the teikoku UNC tree (or with agent_substate set to whatever
     substate the harness uses for cross-project executors), then re-run me (or a
     fresh executor) with the design above -- it is complete and ready to type in
     verbatim, so the re-run should be fast.
  2. Have the orchestrator (which is not path-gated the same way, or can transition
     FSM itself) apply the 3-file change directly from the verbatim design above.
  3. If cross-project WSL targets are supposed to go through a different launch
     path entirely (per .claude/skills/orchestrate/SKILL.md's "cross-project
     targets: launches `claude -p` via tools/orchestrate-launcher.ps1 at target
     CWD" -- a mechanism I am not using since I was dispatched as a plain Agent-tool
     sub-agent, not via that launcher), that may be the structural fix: SP-1A-shaped
     work on a non-ELAI-Rust target may need that launcher, not a same-session
     sub-agent dispatch.
I did not guess at an answer or attempt any destructive workaround (no FSM
self-transition, no forcing Bash, no retrying the same blocked call); this file plus
my final chat return are the two channels I could actually use to surface it.
