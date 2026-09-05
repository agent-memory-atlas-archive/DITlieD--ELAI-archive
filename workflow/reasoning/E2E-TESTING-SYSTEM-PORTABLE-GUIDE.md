E2E TESTING SYSTEM: HOW IT WORKS + HOW TO PORT IT

Source: conductor/ (the autonomous build daemon's "fake-human" dashboard E2E tester +
deterministic bug-hunter). Written 2026-06-28 from the live code and the
conductor-build / conductor-fork-reconcile / e2e-tester handoffs. Plain description of
the design so the same harness can be rebuilt for a different app.

All file paths below are under F:\.ELAI_workflow\conductor\ unless noted.


1. WHAT THIS IS (one paragraph)

An autonomous tester that exercises a running app two ways and judges it on evidence the
tester did NOT author. Lane 1 (vision) drives the real dashboard like a human: a vision
model (gpt-5.5 via codex) looks at a screenshot, picks the next click/type by pixel
coordinate, and a verdict is rendered ONLY against the accessibility tree and an
append-only event journal, never against the model saying "looks done". Lane 2
(deterministic, free, no paid model) asserts PROPERTIES of the real CLI + event journal
(same command twice => same output, bad flags fail cleanly not crash, journal invariants
hold, recorded runs replay). It runs continuously inside a build daemon and turns each
found defect into a fix task that free worker agents pick up (the "flywheel").

The single load-bearing idea: a tester that scores its own output rubber-stamps. So
every verdict is fail-closed (missing evidence == fail) and grounded in an off-agent
artifact (AX tree / journal row / process exit code), and the gate the tester runs is
frozen so the thing under test cannot weaken it.


2. THE TWO LANES AND WHY BOTH

Lane A, VISION (paid, ~10K input tokens per step, expensive, periodic): the only thing
that can catch a VISUAL/interaction bug a human would see. Used sparingly, pixel-diff
gated.

Lane B, DETERMINISTIC (free, runs every pass): catches determinism / robustness /
invariant regressions with zero model cost. This is the workhorse; it found the first
real bug (a CLI `--json` vs human exit-code divergence) and the fix flowed through the
flywheel to VERIFIED autonomously.

A THIRD thing was tried and correctly KILLED: a single-shot "CLI acceptance judge" (one
read-only model fire deciding if a feature meets its acceptance rubric). It was deproved
because a single read-only fire cannot satisfy a multi-scenario contract-test rubric, so
it only ever produced unknown=fail noise. Lesson: acceptance is contract-tests' job;
liveness is the deterministic lane's / wiring-gate's job; paid model credits are worth
spending ONLY on vision. Do not build a single-shot LLM acceptance judge.


3. THE RUBRIC FILE (what to test, decoupled from code)

One JSON file is the machine-consumable map of WHAT gets tested:
  workflow/reasoning/ELAI-TEST-CHECKPOINT-MAP.json
Shape: milestones[] -> each has id/title/done_when/source_plans[]/features[]. Each
feature carries:
  feature_id, name,
  surface: "cli" | "dashboard" | "both" | "internal",
  trigger: { cli_argv: [...] | null, dashboard: "<human interaction string>" | null },
  expected_effect: the observable artifact (a UI element/text OR a named journal
                   action_type row) expected after driving the feature,
  rubric: one-line pass criterion,
  status: shipped | partial | planned | wire-dark (an ASIDE, never a skip reason --
          every feature is treated as real; "spec-eval posture").

Loader: checkpoint_map.py (load_checkpoint_map -> list[Milestone]). Fail-closed: a
malformed map raises ValueError, never silently returns a short list. Two selectors:
  dashboard_journeys(milestone) -> features with surface dashboard|both AND a non-empty
                                   dashboard string (the vision lane's work),
  cli_features(milestone)       -> features with a cli_argv (the CLI lane's work).

PORTABILITY: this file is the only thing that is fully app-specific. To retarget, you
rewrite the map (your views, your CLI commands, your expected journal rows) and keep all
the machinery. expected_effect is the contract between "what the feature does" and "what
counts as proof", so it must name a REAL observable (an AX role/name you can read, or a
real journal row type), not prose.


4. THE VISION LANE, COMPONENT BY COMPONENT

4a. Sandbox isolation (tester_sandbox.py)
  Sandbox = frozen dataclass: name, dev_port, cdp_port, cwd (app run-root), chrome_profile,
  url. Property ssb_db resolves the journal db path under cwd. SandboxPool gives
  mutual-exclusion acquire()/release() (threading.Condition) so two testers never collide
  on port / app-state / browser-profile / journal db. default_pair(base_dir) builds
  tester-1 (dev 1420 / cdp 9222) + tester-2 (dev 1421 / cdp 9223). acquire() RAISES when
  all busy (never returns None, never blocks forever) so a leak is impossible to ignore.
  No filesystem mutation at import; prepare() is the only dir-creating call and it only
  mkdirs, never deletes.

4b. Browser driver, JS side (cdp_action_driver.mjs, run under bun)
  A long-lived single-session CDP (Chrome DevTools Protocol) driver. Connects to an
  already-running Chrome (--remote-debugging-port) + dev server (--url), then reads
  newline-delimited JSON actions on stdin and writes newline-delimited JSON results on
  stdout until op:close / EOF. Supported ops: navigate, screenshot, ax, click
  (by selector OR by x/y coords), type, scroll, console, close. Long-lived (not one-shot)
  specifically because console-error capture needs a persistent WebSocket so
  Runtime.exceptionThrown + Runtime.consoleAPICalled + Log.entryAdded accumulate across
  the journey and are drained by op:console.
  CAPTURE DETERMINISM (critical, or the pixel-diff is pure noise):
    - Emulation.setDeviceMetricsOverride deviceScaleFactor=1 (no silent 2x on HiDPI),
    - Page.captureScreenshot fromSurface:true (fixes the headless empty-bottom-rows bug),
    - kill motion before every shot: inject a zero-duration animation/transition style,
      Emulation.setEmulatedMedia prefers-reduced-motion=reduce, blur the active element
      (no caret blink),
    - settle two requestAnimationFrame ticks after navigate/screenshot.
  AX tree: Accessibility.getFullAXTree crossed against DOM.getBoxModel, skipping
  none/generic/presentation roles, emitting [{role, name, bbox{x,y,w,h}, selector}]. The
  bbox is what lets the model click by coordinate and what lets the safety guard resolve
  "what is under this click".

4c. Browser driver, Python side (cdp_driver.py)
  CdpDriver wraps the .mjs behind an INJECTABLE runner (callable list[action]->list[result]).
  Production runner = _SubprocessRunner managing one long-lived bun process, thread-safe,
  one action-line out / one result-line back per call. Tests inject a plain fake callable
  so the entire public contract (navigate/screenshot/ax_tree/click/type_text/scroll/
  console_errors) is testable with NO real Chrome. ok=false from the driver raises CdpError.

4d. The see->act loop + the vision model (tester_vision.py) -- the heart
  make_codex_vision_fn builds vision_fn(prompt_text, png_bytes) -> raw_model_text. It
  base64-writes the PNG into a model-native scratch dir then calls the model with the image
  attached, prompt piped on stdin; parses the model's final message out of JSONL. Transport
  failure returns "" (no observation), never a fake pass.
  run_vision_journey drives ONE feature:
    1. navigate to the dashboard url,
    2. loop up to MAX_STEPS(15): screenshot + ax_tree -> build a SOURCE-FREE prompt
       (journey goal, expected effect, rubric, last-5-step history, AX element list with
       bbox centers; NO source paths, NO codenames, NO filesystem) -> guard-scan the
       outbound prompt -> call the vision model -> guard-scan the inbound reply -> parse a
       strict next-action JSON {op, x, y, text, done, observed} (any parse failure =>
       op:done, "(parse error)") -> SAFETY-CHECK the action (see 4e) -> execute it via the
       driver -> repeat. The model sets done=true when the goal is visibly achieved OR it
       is stuck.
    3. VERIFY fail-closed: the model saying "done" is NOT evidence. A pass requires
       (a) the expected_effect string appears as an AX role/name in the tree, OR (b) a
       journal row matching expected_effect landed after the pre-journey watermark.
       No evidence => verdict "unknown" == fail for the consumer.
  Returns JourneyResult{feature_id, verdict pass|fail|unknown, steps[], evidence[],
  console_bugs[], visual_note}. console_bugs are drained error-level console entries (a
  free extra bug source). run_dashboard_acceptance_pass loops this over all
  dashboard_journeys of a milestone, reading a journal watermark before each.

4e. UI-action safety guard (tester_vision.py, fail-closed deny-list)
  The model picks clicks by coordinate, so it CAN hit a "Run cloud task" / "Delete" / "Pay"
  button. classify_ui_action_safety resolves the SMALLEST AX element under the click point
  and refuses (stops the journey) if its accessible name/role matches a named deny-list
  (DESTRUCTIVE_ACTION_LABELS: launch/cloud/run-task, pay/billing, delete/reset/wipe,
  commit/push/merge/publish...). Observe-only ops (scroll/screenshot/ax) always pass; a
  typed string that itself contains a destructive command is also refused. The verdict
  cannot be flipped by a flag. This is the dashboard analog of the CLI safe-list.

4f. Production glue (tester_runtime.py)
  DashboardRuntime.start() launches the Vite dev server (bun run dev --port --strictPort)
  then headless Chrome (--headless=new, --remote-debugging-port, isolated --user-data-dir,
  --font-render-hinting=none) with settle delays; stop() tears down ONLY what it started
  (never kills unrelated Chrome/bun). Resolvers handle bun-installed-via-npm on Windows and
  chrome/chromium on Linux. Three factories wire the injectables for a sandbox:
  make_production_vision_fn, make_production_cdp_runner (owns the long-lived bun process),
  make_production_ssb_probe (journal reader at sandbox.ssb_db).

4g. Pixel-diff gate -- the credit saver (vision_gate.py)
  Doctrine (from .claude/rules/visual-ui-loop.md): run a DETERMINISTIC pixel diff BEFORE
  any paid vision call. It reuses the existing .claude/tools/visual/ui-diff.mjs (pixelmatch,
  AA-aware) under bun, NOT a reimplementation. evaluate_pixel_diff returns GateDecision:
    - candidate pixel-identical to its golden frame => skip_identical, needs_vision=False
      (free pass, no model call -- the single biggest cost saver),
    - changed => vision_changed + the hotspot regions the model should look at,
    - no golden frame => vision_no_golden (full-frame semantic vision),
    - diff error / size mismatch => FAIL-CLOSED (treat as changed, never read an error as a
      pass; a dimension change IS a regression).
  golden_for_journey resolves a known-good PNG per feature from the dashboard screenshots
  dir. Goldens are app-produced frames only; never pixel-diff against a Figma/hand mock
  (that is semantic-judgement territory, not pixel).


5. THE DETERMINISTIC LANE (test_modes.py) -- free, every pass

Four modes, all driving the REAL binary (read-only triggers) + the read-only journal,
everything external injected so each is provable with zero credits / zero live binary:
  1. metamorphic / differential CLI: same command twice => identical journal action_type
     signature modulo per-run nonce (a difference = non-determinism defect); a `--json`
     run and a human run must agree on success/failure class (divergence = defect); an
     idempotent command's second run must add NO new state row.
  2. arg-surface fuzz: drive the command with malformed flags/args; each MUST fail cleanly
     (a normal nonzero/usage exit), NEVER crash (panic, abort, signal, Windows 0xC000xxxx).
     A LAUNCHER-side failure (input never reached the binary, e.g. an embedded null byte on
     Windows) is SKIPPED, never counted as a defect -- it tested nothing. (This false-
     positive class cost a whole live run before it was fixed; the fuzz corpus only keeps
     cases that actually reach the binary as argv.)
  3. journal (SSB) invariants: schema_version monotonic, action_type in a golden lock set,
     seq strictly increasing (no dup), no orphan rows (action_type with blank session_id).
  4. replay-verify: drive `replay --verify <recorded-run>`; a nonzero exit means the run no
     longer reproduces (a determinism defect).
A finding is NON-LOAD-BEARING: it is display + a fix-task seed, never a gate, never ships
code, never weakens a gate. run_test_modes_pass walks a "standing target queue" of safe
read-only CLI leaves (the help-tree) + a trigger safe-list (trigger_safety.py) so it never
auto-fires a mutating/expensive command.


6. DAEMON INTEGRATION + THE FLYWHEEL

The build daemon's main loop (conductor_loop.py run_backlog_waves) calls, behind default-OFF
config flags, after its normal build waves and fail-soft (a tester crash never breaks a
build):
  - dashboard_tester_stage -> run_dashboard_tester_pass (the vision lanes), emits
    DASHBOARD_TESTER_PASS / *_ERROR,
  - test_modes -> run_test_modes_pass (deterministic), emits TESTER_MODES_PASS + per-defect
    TESTER_DEFECT,
  - autofix_stage -> turns wire-dark findings into idempotent FIX tasks.
tester_pass.py orchestrates the vision lanes: find a milestone (UI mode when its source
plans are all DONE; with continuous_vision on, any milestone with a dashboard journey is a
standing target), split journeys round-robin across the two codex-account lanes
(tester_lane.py), run each lane in its own sandbox, release in finally. continuous_vision
inserts the pixel-diff gate before the paid pass.
THE FLYWHEEL (proven end-to-end): deterministic lane finds a defect -> autofix seeds a
FIX-TESTER task -> a free worker agent fixes it -> the FROZEN wiring gate verifies it ->
status VERIFIED. Find -> fix -> verify, autonomously, no human and no paid model in the loop.


7. THE NON-NEGOTIABLE DOCTRINE (the part that makes it trustworthy)

  - Fail-closed everywhere: missing evidence == FAIL, never SKIP/UNVERIFIED/WARN. A dead
    evidence path screams; a probe that can't read returns "no evidence", which is a fail,
    never a pass.
  - Off-agent evidence: a verdict is grounded in an artifact the tester did NOT write -- an
    AX role/name, a journal row, a process exit code. The model's "looks done" is worth
    zero.
  - unknown == fail: a journey/dimension the tester cannot judge is not passing.
  - Frozen gate: the gate the tester runs is read-only to the thing under test (gate-manifest
    freeze), so a run cannot rewrite its own evaluator to pass.
  - Default-OFF + fail-soft: every capability is behind a flag and swallowed on error so the
    tester can never take the daemon down.
  - Injectable edges: every external (model transport, browser, journal, cli) is a injected
    callable, so the whole system is unit-testable with zero credits / zero live processes
    (the suite is ~795 tests, all green, no live anything).
  - Data minimization to the model: prompts are SOURCE-FREE (goal + rubric + AX list +
    downscaled crop only; never source code, paths, or codenames) and outbound/inbound
    guard-scanned. Matters doubly when the model is a third-party proxy.
  - Anti-rubber-stamp schema: per-item pass|fail|unknown with a mandatory evidence region;
    "looks fine overall" is not a verdict.


8. PORTING CHECKLIST (rebuild for another app)

KEEP almost verbatim (app-agnostic machinery):
  tester_sandbox.py, cdp_action_driver.mjs, cdp_driver.py, tester_vision.py (the see->act
  loop, verify, safety guard), tester_runtime.py, vision_gate.py, tester_lane.py,
  checkpoint_map.py (the loader), test_modes.py (the four modes), trigger_safety.py.

REPLACE / SUPPLY per app:
  1. The rubric file (ELAI-TEST-CHECKPOINT-MAP.json equivalent): your views, your CLI
     commands, your expected observables. This is the bulk of the retargeting work.
  2. The off-agent evidence source. ELAI uses an append-only event journal (SSB sqlite,
     read-only WAL) + the AX tree. Any app needs SOME off-agent artifact a feature leaves
     behind that the tester can read without authoring -- a structured log, a metrics file,
     an analytics event, a DB row, an exit code. Without one, the vision lane degrades to
     AX-only (still useful) and the deterministic journal-invariant mode does not apply.
  3. The launch story: how the dev server + browser come up (DashboardRuntime), how the CLI
     binary is located (make_cli_runner), where the journal db lives (Sandbox.ssb_db). All
     three are small, localized.
  4. The destructive-action deny-list + CLI safe-list for YOUR surfaces (a tester that can
     auto-click "Delete account" or run a billable command is a liability).
  5. Goldens: ~80 known-good screenshots the app itself produced, for the pixel-diff gate.
     Optional but it is the main credit saver.

DECISIONS TO COPY (learned the hard way here):
  - Do NOT build a single-shot LLM acceptance judge. It cannot satisfy a real acceptance
    rubric from one read-only fire; it only emits unknown=fail noise. Acceptance = contract
    tests; liveness = the deterministic lane; paid credits = vision only.
  - Capture MUST be pinned (scale=1, fromSurface, font-render-hinting=none, motion killed,
    rAF-settled) or every diff flags font/AA churn and the whole pixel gate is noise.
  - Pixel-diff FIRST, vision SECOND. The diff decides whether to spend a model call and
    where to look.
  - One bug -> one root cause. Dedup tester defects by the command/element, not by feature,
    or one root cause spawns N duplicate fix tasks (an open issue here).
  - Run vision sparingly (periodic, per-pass end), the deterministic lane always. Vision is
    ~10K tokens/step; the deterministic lane is free and catches the regression classes that
    actually recur.


9. KNOWN ROUGH EDGES (in the current ELAI build, inherit-aware)

  - Vision E2E is milestone-gated and rarely fires (no milestone's source plans are all DONE),
    so two codex accounts sit largely idle. A recorded-but-unbuilt plan repurposes that idle
    vision capacity into continuous dashboard DESIGN-CRITIQUE (codex = vision critic against a
    DESIGN.md rubric, free glm worker = hands that fix). Same critic-vs-hands split, same
    flywheel; needs a DESIGN.md authored first.
  - The daemon tester lanes have twice been "silently dead" from refactor regressions that the
    fail-soft swallow hid (a missing constructor arg, a moved import). A daemon-path integration
    test (test_daemon_tester_lanes_integration.py) now drives the real entrypoints through
    production default-construction so this seam can't silently break again -- worth replicating:
    fail-soft hides exactly the breakage you most need to see.
  - tester defect dedup-by-command is still open; LETTERS plan-prefix list is hardcoded.
