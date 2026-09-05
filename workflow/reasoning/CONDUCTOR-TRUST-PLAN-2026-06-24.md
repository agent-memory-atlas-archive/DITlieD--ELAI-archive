CONDUCTOR TRUST PLAN (2026-06-24)

PROBLEM
Conductor's loop SHAPE is good (a for-each over a dependency-frontier task queue,
an off-agent frozen gate, an isolated reversible target repo, human graduation).
Its SIGNAL is the weak link, and a loop is only as trustworthy as its success
signal. Three independent sources converge on this: dbreunig "specify behavior
with measurements, not prose"; Ronacher "the signal must be principled, not
arbitrary" + loops are worst on code you deeply care about (Conductor loops on
permanent production Rust); the skeptic's exponential decay (0.95^10 ~ 0.60 --
a faster loop on a noisy gate compounds slop FASTER). Our own wiring-enforcement
doctrine is the spec for a trustworthy signal, and Conductor only partially
implements it. So the goal of this plan is NARROW and NON-NEGOTIABLE: make the
gate signal SOUND so a green "DONE" can be trusted to mean "real, wired, correct
ELAI improvement" -- not "compiled + passed a noisy check".

NON-GOALS (explicit -- do NOT do these next session)
- NO new throughput / worker-saturation / concurrency work. The last session
  added cross-wave borrow + STATE_CHECK filler (worker UTILIZATION, a proxy).
  Utilization is not the objective (wiring-enforcement R4: measure the objective,
  not the proxy). A faster loop on an unsound gate is actively harmful. Freeze
  throughput until the signal is measured and sound.
- NO live "for real" 4-worker run except to TEST a phase. Flag before any heavy
  boot. Respect the 4-worker free-model ceiling (4 = the tested cap; other
  free-model sessions = 3h lockout). On restart after a clean stop, the draining
  flag must be cleared: `python conductor.py control undrain --db state/control.db`.

DIAGNOSIS -- WHERE THE SIGNAL IS UNSOUND (grounded in code read 2026-06-24 + the
handoff's own records)
1. FALSE POSITIVES (starve the frontier, waste the loop): per-task verifiers run
   over-broad suites that inherit the isolated elai-repo's RED test baseline (the
   handoff: ~89% of NEEDS_REVIEW were false positives; 57 pre-existing
   orchestrator reds). Violates the house "scoped test runs -p" rule. The current
   band-aid (cargo_gate_support.mask_baseline_failures) downgrades a cargo_test
   FAIL->PASS when every failing test is in a STATIC precomputed baseline file
   (baseline_known_failures.json @ a fixed HEAD). That band-aid is itself a
   FAIL-OPEN hazard (R1/R2: missing/ambiguous evidence must be FAIL, never a
   silent red->green) and it goes STALE as HEAD moves.
2. FALSE NEGATIVES (ship slop into elai-repo): the gate is mostly Layer 1/2
   (static) + cargo compile/test. It has NO runtime proof that an integrated
   commit actually RUNS the new code from a real entrypoint (wiring-enforcement
   Layers 3-4 absent). The wiring rules are explicit: static legs are
   necessary-not-sufficient (R5: name-grep is not a call graph), and the runtime
   leg is the only sound backstop. Conductor also audits each task in ISOLATION
   with no independent cross-seam integration smoke (R10: unit-green is not
   integrated-green), so a defect at the seam between two tasks passes every
   per-task gate and ships.
3. UNMEASURED: there is no measured false-negative rate and no measured wire-dark
   rate on the actual DONE corpus. The residual-risk section of
   wiring-enforcement.md is explicit: "Instrument your own rate ... so this
   redesign is judged on measured deltas, not faith." We have been running on
   faith. You cannot trust, or improve, a signal you have not measured.

THE PLAN (4 phases, strictly ordered; each phase is GATED by a measurement)

PHASE 1 -- MEASURE THE TRUTH (do this FIRST; it is the trust baseline)
Goal: a believable number for how often a "DONE" task is actually wire-dark or
wrong, plus an audit of every fail-OPEN path in the gate. Without this, every
later change is faith.
Deliverables:
  - conductor/trust_audit.py (NEW): sample N random DONE tasks from control.db
    (default N=30, --n override, deterministic seed via --seed for replay). For
    each, against the integrated elai-repo commit:
      (a) re-run the off-agent wire checker for the task's languages
          (.claude/tools/production_caller_check.py for .rs touched files,
          python_wire_check.py for .py) SCOPED to the affected crate/module,
      (b) classify: WIRED-DONE / WIRE-DARK-DONE (pub symbol, no production
          caller, no WIRE-DARK row) / UNVERIFIABLE (checker could not resolve).
    Emit a JSON + markdown report to conductor/state/trust-audit/<ts>.md with the
    measured wire-dark-DONE rate and the UNVERIFIABLE rate. Print the rate.
  - A short written audit (same report) of every place in gate_runner.py +
    cargo_gate_support.py that can emit PASS/SKIP on MISSING or AMBIGUOUS evidence
    (R1/R2/R3 violations). The mask_baseline_failures red->green path is the prime
    suspect; list any others.
Acceptance: a measured wire-dark-DONE rate (a number, not a vibe), and a written
  list of fail-open paths. This number decides the urgency of Phases 2-3.
Grounding: wiring-enforcement.md residual-risk ("instrument your own rate");
  R5 (sound reference, scoped); D-030 (the off-agent checker is the authority).
Notes: production_caller_check.py / python_wire_check.py time out when --root is
  the whole workflow tree -- ALWAYS scope --root to the affected crate/module dir
  (this is the same scoping bug that produces the red-baseline false positives).

PHASE 2 -- KILL THE FALSE POSITIVES (sound the per-task cargo signal)
Goal: a per-task cargo verdict that reflects the DELTA the task introduced, not
the absolute state of a red baseline -- without any silent red->green.
Deliverables:
  - Enforce crate-scoped verifiers: confirm/repair that every per-task cargo gate
    runs `cargo check`/`cargo test -p <affected-crate>` (gate_runner.affected_crate
    already derives the package; verify no over-broad workspace runs remain on the
    test leg). Scope is the fix; masking is the symptom.
  - Replace the STATIC baseline mask with a SOUND per-base delta OR make the mask
    fail-CLOSED and self-checking: option A (preferred) = snapshot the affected
    crate's reds at the task's base_commit and FAIL only on NEW reds vs that base
    (sound, delta-based, matches dbreunig "measure deltas"); option B = keep the
    static baseline but (i) auto-refresh it at the current elai-repo HEAD on a
    schedule, (ii) record every mask application as an explicit, auditable WARN
    event (never a silent PASS), and (iii) FAIL if the baseline file is missing or
    older than a max-age (R2: a dead evidence path must scream, not shrug).
Acceptance: re-run a sample of the historically-NEEDS_REVIEW tasks; the
  false-positive rate measurably drops, and NO cargo_test FAIL is converted to
  PASS without either a delta proof or a recorded WARN. Add unit tests for the
  delta/auto-refresh logic.
Grounding: R1 (missing evidence = FAIL), R2 (no silently-absent evidence; scream),
  R3 (no opt-in that lets static-only emit PASS), R4 (measure the objective).

PHASE 3 -- ADD THE LOAD-BEARING RUNTIME BACKSTOP (kill the false negatives)
Goal: an INDEPENDENT, periodic gate that proves the integrated elai-repo actually
RUNS the shipped code from a real entrypoint -- the only sound check per the
wiring rules, and the one Conductor is missing.
Deliverables:
  - conductor/integration_smoke.py (NEW): after every epoch (or every K
    integrations), drive a REAL elai entrypoint on the isolated elai-repo with a
    HARD input (a real CLI subcommand / FSM transition / task_runner path -- NOT a
    toy happy-path), and assert an OFF-AGENT artifact landed that the worker did
    NOT author (an SSB event row / a tracing span / an LLVM coverage hit on the
    new symbols). Missing artifact = FAIL (R1). The smoke is authored ONCE,
    frozen, and read-only to the run (R8 / gate_freeze_check), and is driven by
    the controller, never by the task's own worker (an agent cannot smoke-test its
    own output -- R8 role boundary).
  - Wire it into conductor_loop epoch compaction (run after every epoch; a FAIL
    blocks further graduation and surfaces to the escalation queue, it does not
    silently continue).
Acceptance: plant ONE known wire-dark commit (a new pub fn with no production
  caller) onto a throwaway elai-repo branch; the integration smoke must catch it
  even though it passed every per-task gate. Prove the catch in a test.
Grounding: R5 (runtime leg is load-bearing, name-grep is not), R10 (unit-green is
  not integrated-green; independent cross-seam smoke on a HARD input), Layer 3/4
  of the layered gate; tracing-test-assert.md if the artifact is a tracing log
  (logs_contain MUST be inside an assert!).

PHASE 4 -- (STRETCH) TURN THE STATE_CHECK FILLER INTO A SECOND AUDITOR
Goal: make the cargo-free STATE_CHECK reports (added last session) genuinely
useful instead of accumulating -- they already find real wire-dark symbols.
Deliverables:
  - When a STATE_CHECK report flags a DONE plan as actually wire-dark (a produced
    symbol with no production caller), auto-open a fix task (or push a row to the
    escalation/triage queue), so the report drives real work. This makes the
    filler an INDEPENDENT wiring auditor (a second opinion on the gate), closing
    the loop on the utilization feature instead of leaving it as theater.
Acceptance: a STATE_CHECK report naming a wire-dark DONE plan produces a queued
  fix task; verified by a unit test.
Grounding: R6/R7 (producer-without-(production-)consumer is a defect to surface);
  D-030 (the wire checkers are the authority on dark symbols).

HOW TO WORK (next session)
- Follow project protocol: prompt-transform first; conductor edits are tooling, so
  CONFIGURING FSM state (the PreToolUse hook blocks conductor/ writes in EXECUTING).
  conductor/ is gitignored -> NO commit; persists on disk. Run conductor unittests
  yourself ONLY to validate (this is gitignored tooling and the user iterates with
  you): `python -m unittest discover -s tests` from conductor/ (114 green today).
- Maintainability: conductor_loop.py is already >1000 lines (MR-1 WARN); put NEW
  code in NEW modules (trust_audit.py, integration_smoke.py), do not grow the loop.
- Each phase is gated by a MEASUREMENT. Phase 1 first -- do NOT skip to building
  Phase 3. If Phase 1 shows the wire-dark-DONE rate is low, Phases 2-3 are still
  worth doing but the urgency/scope can scale down; report the number and ask.

SUCCESS DEFINITION
A "DONE" on elai-repo can be trusted: it is wired (production caller proven, not
name-grepped), it runs (an off-agent runtime artifact landed from a real
entrypoint), and the cargo verdict reflects the task's delta (no silent red->green,
no red-baseline false fail). The trust-audit number is the scoreboard; the redesign
is judged on the measured delta in that number, not on faith.
