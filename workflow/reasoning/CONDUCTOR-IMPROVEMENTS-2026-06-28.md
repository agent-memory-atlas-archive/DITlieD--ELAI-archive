# Conductor improvements (2026-06-28)

PURPOSE: a self-contained implementation spec for the NEXT session to harden the ELAI Conductor. Every item below carries: WHAT, GROUNDING (a rival harness design we verified from primary source, and/or a weakness the Conductor's own handoffs admit), the CONDUCTOR FILES to touch, a concrete IMPLEMENTATION, and ACCEPTANCE criteria. Do the P0 block first, in the stated order. Nothing here changes .elai_cc; the Conductor is workflow tooling under F:\.ELAI_workflow\conductor\ (gitignored).

This doc is grounded, not freestyle. The two rival systems were read end-to-end from their repos this session (14+ source-file fetches each). The Conductor's own weaknesses are quoted from its handoffs. Verify any claim before acting on it (anti-stale, grounded-by-default).

=============================================================================
CONTEXT (read first)
=============================================================================

What the Conductor is: a deterministic Python daemon driving N FREE Devin CLI workers (glm-5.2 High x3) through implement -> wire -> integrate -> test -> audit -> polish waves over the ELAI plan corpus (~412 plan files B..L + base phases 0-4). Trust is anchored OFF-AGENT in a frozen gate stack; workers commit only to the isolated conductor/elai-repo (allow_push=false); vetted work reaches GitHub main via fork -> cargo gate -> PR -> free-GLM review -> HUMAN merge.

Key handoffs (read the HEAD of each, do not full-read):
- workflow/handoff/conductor-build.md  (build/scheduler/integration incident log + the 2026-06-24 "PIVOT to TRUST" entry)
- workflow/handoff/e2e-tester.md       (tester lanes, acceptance judge, promotion pipeline)

Spec + prior design docs (workflow/reasoning/):
- AUTONOMOUS-FINISH-SYSTEM-DEVIN-2026-06-18.md  (build order, concurrency model, escalation)
- CONDUCTOR-TRUST-PLAN-2026-06-24.md            (P1 measure / P2 scope+sound-mask / P3 runtime smoke / P4 autofix) -- items P0.2/P0.3/P0.4 below are the same wounds; reconcile with this doc, do not duplicate
- ELAI-BATTLE-HARDENING-DESIGN-2026-06-27.md    (tester phases)
- E2E-ACCEPTANCE-TESTER-DESIGN/BUILD-2026-06-26.md

Conductor source map (named in the handoffs; grep to confirm current line numbers, do NOT trust stale ones):
- conductor_config.py        config loader, LimitConfig flags, WorkerSlot
- scheduler.py               DependencyFrontierScheduler, schedule_frontier, frontier/cooldown/rate-limit
- leases.py                  LeaseManager, acquire_ready, scope-overlap (file + blast_radius), model_tier
- budgets.py                 retry caps, daily caps, model cooldowns
- worker_adapter.py          devin -p command builder, slot selection, role/agent-config injection
- conductor_loop.py          run_task_loop / run_wave_loop / run_backlog_waves, _schedule_one, _finish_state_check, DISK_FLOOR_GB, enrich_context
- gate_runner.py             execute_gate_command (off-agent frozen checkers + cargo), baseline mask + cargo semaphore wired here
- cargo_gate_support.py      baseline-failure mask (downgrade FAIL->PASS iff every failing test is a pre-existing baseline red)
- integrator.py              ff-merge-only integration into elai-repo, dirty-checkout self-heal
- acceptance.py              contract parse + CLI/IPC driver + SSB seq-window probe + run_acceptance_pass (the CLI-SSB judge)
- tester_vision.py / tester_runtime.py / tester_pass.py   vision computer-use E2E lane (CDP + AX-tree)
- test_modes.py              deterministic bug-hunter (metamorphic/fuzz/SSB-invariant/replay)
- backlog_import.py / plan_order.py   plan ingest, LETTERS/PLAN_LETTERS, dep wiring
- dashboard_server.py        /api/status, /api/acceptance tiles
- promote.py                 fork->main promotion pipeline (gate->PR->GLM review->human merge)
- reconcile_status.py        stale NEEDS_REVIEW/FAILED -> PLANNED requeue

GROUNDING SOURCES (rival harnesses, verified this session):
- LazyCodex engine = code-yeongyu/oh-my-openagent (submoduled into code-yeongyu/lazycodex). TS/Bun plugin into OpenCode/Codex.
- Gajae-Code = github.com/Yeachan-Heo/gajae-code (gjc). TS/Rust/Python standalone shell wrapping any coding CLI. MIT, ~1.2k stars, v0.7.5.

THE THROUGHLINE: P0.1-P0.4 are one wound the Conductor already diagnosed (2026-06-24 handoff, verbatim): "FALSE POSITIVES from over-broad/red-baseline verifiers + a fail-OPEN static baseline mask; FALSE NEGATIVES from a static-only gate with NO runtime backstop (Layers 3-4 absent) and no cross-seam integration smoke; and an UNMEASURED wire-dark-DONE rate (running on faith, which the rules forbid)." Gajae-Code's surface-typed evidence model is the cleanest external blueprint for the fix. Start at P0.3 (MEASURE) so P0.1's effect is visible against a baseline.

=============================================================================
P0 -- close the trust-signal holes (do these first, in order: P0.3 -> P0.1 -> P0.2 -> P0.4)
=============================================================================

P0.1  PER-SURFACE "DONE CONTRACT" ENFORCED BY THE GATE
WHAT: a plan is only DONE when it produces the KIND of evidence its surface demands. Bare "cargo green" is never sufficient for a non-trivial surface.
GROUNDING (steal: Gajae-Code ultragoal 7-step gate, step 5): "done" evidence varies by surface --
  GUI/web -> "valid automation transcript plus a non-uniform screenshot"
  CLI     -> "replaySafe: true, allowlisted argv command, replayed normalized stdout"
  API/package -> "real artifact file or typed receipt"
  Algorithm/math -> "real artifact file whose kind contains property|boundary|edge|adversarial|failure|math|test-report"
  "Bare inlineEvidence text alone is not sufficient for any surface."
  (source: packages/coding-agent/src/defaults/gjc/skills/ultragoal/SKILL.md; architect.md = read-only reviewer emitting architectureStatus/productStatus/codeStatus each CLEAR|WATCH|BLOCK)
This is the systematic version of the holes ELAI patches piecemeal (NO_CHANGE, wire-dark, vacuous-pass).
CONDUCTOR FILES: acceptance.py (the evidence drivers already exist per surface), gate_runner.py (enforce the contract), conductor_config.py (surface enum), backlog_import.py (derive/carry surface per plan), tester_vision.py (UI evidence source).
IMPLEMENTATION:
  1. Add surface:cli|ui|api|algo to each plan's task metadata. Derive from the plan's `trigger:`/rubric line where present (acceptance.py already parses an `elai <subcommand>` trigger); default-classify the rest from the plan's declared output_type. Unknown surface => the strictest contract, never the loosest (fail-closed).
  2. Gate demands the matching evidence kind, reusing what ELAI already has:
       cli  -> SSB-row-by-id (acceptance.py make_ssb_probe + watermark) AND normalized stdout replay of the real `elai` argv
       ui   -> vision tester AX + pixel-diff evidence (tester_vision/tester_runtime), not a text claim
       api  -> a real artifact file / typed receipt the worker produced
       algo/bench -> a real artifact file (a measurement TOML, a property/edge test report)
  3. Bare text ("looks wired", "should pass") is NEVER sufficient -- mirror Gajae's rule explicitly in the gate verdict.
ACCEPTANCE: a synthetic plan that compiles + passes cargo but emits NO surface evidence is BLOCKED, not DONE. A CLI plan whose `elai` trigger fires and lands its SSB row is ACCEPTED. Add a regression test per surface in conductor/tests/.
FAIL-CLOSED: missing surface evidence = FAIL (wiring-enforcement R1). Never SKIP, never WARN.

P0.2  RUNTIME BACKSTOP (Layer 3/4) FOR EVERY CLI-SSB PLAN, NOT JUST THE ACCEPTANCE JUDGE
WHAT: the gate drives the REAL elai entrypoint and asserts the off-agent SSB row landed -- promote this from the (non-load-bearing, sampled) acceptance judge into the actual DONE gate for CLI-SSB plans.
GROUNDING: Conductor's own #1 admitted gap -- "static-only gate with NO runtime backstop (Layers 3-4 absent, R5/R10)" (handoff 2026-06-24). Both rivals gate on execution evidence (Gajae step 5 surface evidence; LazyCodex Oracle evaluates the actual result, not a status update). ELAI ALREADY proved the mechanism end-to-end: D-8 `elai archive verify` fired -> fresh archive_verify_result SSB row in .elai_cc/sessions/ssb.db -> watermarked probe read it by id (handoff conductor-build.md 2026-06-26 ~23:05Z). The codegen-units build fix that unblocked the binary is already in .elai_cc/Cargo.toml.
CONDUCTOR FILES: gate_runner.py, acceptance.py (reuse make_cli_entrypoint_runner + collect_cli_evidence + the seq-watermark), conductor_loop.py.
IMPLEMENTATION: for a plan whose surface=cli (P0.1), after the static gate passes, fire the real argv ONCE (E6 isolation), seq-watermark the SSB, and require the expected row by id. cwd MUST be the .elai_cc PARENT (workflow root) or the binary nests ssb.db -- this gotcha is documented in the handoff; reuse _elai_cc_root/_default_entrypoint_runner. Reconcile with CONDUCTOR-TRUST-PLAN-2026-06-24.md P3 (same intent) -- extend, don't fork.
ACCEPTANCE: a CLI plan that compiles but whose entrypoint emits NO row = BLOCK. Proven on a real feature (D-8 or another SSB-emitter) end-to-end with nothing injected.
FAIL-CLOSED: db-not-found / zero-rows in the window = FAIL (R2), never UNVERIFIED.

P0.3  MEASURE THE WIRE-DARK-DONE RATE (do this FIRST -- it is the baseline for everything else)
WHAT: a periodic sweep that samples DONE tasks, re-runs the wire checkers off the INTEGRATED elai-repo, and emits a metric.
GROUNDING: Conductor's own trust-plan P1 + the house rule "instrument your own rate (wire-dark plans / total) ... judged on measured deltas, not faith" (wiring-enforcement.md RESIDUAL RISK). Neither rival measures this -- it is an ELAI obligation. Prior measurement existed (handoff 2026-06-24: "only 59/221 DONE (27%) had a durable elai-repo commit") -- make it CONTINUOUS, not a one-off.
CONDUCTOR FILES: a new conductor/wire_dark_audit.py (off-agent, read-only) + dashboard_server.py tile + an events row.
IMPLEMENTATION: sample DONE plans, for each run .claude/tools/production_caller_check.py (.rs) / python_wire_check.py (.py) over its integrated changeset on elai-repo HEAD, classify WIRED/INTERNAL/DARK/REGISTERED-DARK, emit WIRE_DARK_RATE to control.db events + a /api/status tile. Distinguish accidental-dark from REGISTERED-DARK (a WIRE-DARK[plan-id] marker + backing row) -- only accidental counts against the rate.
ACCEPTANCE: dashboard shows a live wire-dark-DONE %; re-running it on the same DONE set is stable. This NUMBER is how P0.1/P0.2 get judged.

P0.4  SCOPED VERIFIERS (-p), KILL THE OVER-BROAD SUITE RUNS
WHAT: each task's cargo gate runs only the changed crate(s), not the whole 4017/652-test workspace.
GROUNDING: Conductor's own deferred lever, flagged repeatedly -- the over-broad runs "inherit the red baseline -> manufacture false failures (violates the house 'scoped test runs -p' rule)" (handoff 2026-06-23 INVESTIGATE-RESULT; ~89% of NEEDS_REVIEW measured as false positives). House rule: "Scoped test runs: do NOT run full workspace ... Only ... affected crates (-p)".
CONDUCTOR FILES: gate_runner.py (the cargo command builder), and the changed-file -> crate mapping (reuse blast_radius.py if it already maps files->crates, else add it).
IMPLEMENTATION: derive affected crates from the task write_set + blast_radius, build `cargo test -p <crate>...` instead of `--workspace`. Keep cargo_gate_support.py baseline-mask as a second layer for the genuinely-red crates, but scoping should make most masking unnecessary. Confirm the baseline mask is fail-CLOSED on NEW breakage (the handoff once called it "fail-OPEN" -- verify and fix if still open, this is a gate-soundness bug).
ACCEPTANCE: a task touching one crate runs only that crate's tests; NEEDS_REVIEW false-positive rate drops measurably (compare against the P0.3 sweep window).

=============================================================================
P1 -- cut false positives + repeat failures
=============================================================================

P1.1  AI-SLOP-CLEANER AS A BLOCKING, CHANGED-FILES-ONLY STEP WITH RE-VERIFY
GROUNDING (steal: Gajae-Code ultragoal steps 2-3): run a slop-cleaner "on changed files only -- every BLOCKING cleaner finding is a completion blocker" THEN "Rerun verification after cleanup". ELAI already has .claude/tools/maintainability_check.py + Zero-Lazy-Code, but they're applied ad hoc by the auditor.
CONDUCTOR FILES: gate_runner.py.
IMPLEMENTATION: a mandatory gate stage scoped to the diff that runs maintainability_check.py + a lazy-code scan (TODO/placeholder/empty-body/unwrap-in-prod per token-discipline Zero-Lazy-Code) over changed files; any BLOCK parks the task with a typed reason; after the worker cleans, RE-VERIFY (not one-shot). Frozen per .claude/gate-manifest.txt -- the run cannot weaken it.
ACCEPTANCE: a diff that adds a `// TODO` placeholder or crosses the 1000-line file boundary is BLOCKED; re-running after the worker fixes it passes.

P1.2  PER-PLAN WISDOM NOTEPAD FORWARDED TO EVERY SUBSEQUENT WORKER
GROUNDING (steal: LazyCodex Atlas): "After each task: extract learnings ... categorize into Conventions, Successes, Failures, Gotchas, Commands ... pass forward to ALL subsequent subagents" (docs/guide/orchestration.md). ELAI already ships WORKER-CONTEXT ENRICHMENT A+B (code-graph callers + production-hook + baseline-red list) -- this is the natural extension.
CONDUCTOR FILES: conductor_loop.py (the packet builder where enrich_context lives), a new state/wisdom/{plan}.md store.
IMPLEMENTATION: after each completed task on a plan, harvest a few lines of Conventions/Failures/Gotchas/Commands from the worker result; append to state/wisdom/{plan}.md; inject the accrued notepad into the next worker's packet for that plan. Bounded (token economy -- cap the notepad, newest-wins).
ACCEPTANCE: a second task on a plan whose first task hit a gotcha receives that gotcha in its packet. Reduces repeat-failure churn (measure against the incident-log pattern of the same blocker recurring).

P1.3  EMPTY-RESPONSE / PHANTOM-DONE DETECTOR AS A FIRST-CLASS GUARD
GROUNDING (steal: Gajae-Code empty-response detection): identify proxy-level context overflow / empty work when "stopReason === 'stop' with empty content and anomalously low token usage; promotes to error state" (packages/agent/src/agent-loop.ts). Fixes the Conductor's NO_CHANGE hole (handoff 2026-06-24: a NO_CHANGE result was promoted to DONE, the gate passes VACUOUSLY on an empty diff; 159/162 no-commit-DONE had 0-byte diffs).
CONDUCTOR FILES: the scheduler/result-classifier (scheduler.py or conductor_loop.py where worker results are graded).
IMPLEMENTATION: generalize the NO_CHANGE patch -- a worker result is suspicious-empty if (0-byte diff on an implement/wire role) OR (anomalously low token usage with stopReason=stop). Route to NEEDS_REVIEW with a typed reason (no_change_on_implement_role already exists), NEVER DONE. Keep dry-run-exempt.
ACCEPTANCE: an empty-diff implement result is parked, not promoted. Regression test.

P1.4  BLOCKED-VS-RESOLVABLE CLASSIFICATION BEFORE PARKING TO NEEDS_REVIEW
GROUNDING (steal: Gajae-Code ultragoal): "An active Ultragoal run must not give up on a blocker by pausing ... must be exhausted before pause is permitted"; blockers classified resolvable vs true. ELAI measured ~89% of NEEDS_REVIEW as infra false-positives (handoff 2026-06-23).
CONDUCTOR FILES: budgets.py / scheduler.py (the park decision), reconcile_status.py (the existing requeue).
IMPLEMENTATION: before parking, classify the failure (reuse .claude/tools/failure_taxonomy.py, 13-category regex classifier): transient/resolvable (rate-limit, cargo-lock, timeout, baseline-red, WinError-6-class) -> auto-requeue with a strategy change / refunded attempt; true blocker (real compile error, real test fail on a clean baseline) -> NEEDS_REVIEW/escalate. This is the automated version of the manual reconcile passes the handoff keeps running by hand.
ACCEPTANCE: an infra-class failure auto-requeues without human touch; only genuine failures reach NEEDS_REVIEW.

=============================================================================
P2 -- operational + human-loop polish
=============================================================================

P2.1  PUSH-NOTIFICATION ESCALATION SURFACE (tighten the existing human-merge node)
GROUNDING (steal: Gajae-Code v0.7.x Telegram/Discord/Slack human-in-loop + action_needed/reply protocol). The Conductor's fork->PR->HUMAN-merge promotion ALREADY has a human approval node, but it is pull-only (localhost dashboard :8766).
IMPLEMENTATION: wire a push notification on (a) promotion PR ready-for-human-merge, (b) a true-blocker escalation, (c) a sustained rate-limit stall. Use the harness PushNotification tool if the implementing session has it, else a simple webhook from dashboard_server.py. Do NOT add autonomy -- this only PINGS the human faster; the human still clicks merge. Honor feedback_external_prompt_anonymity if any content leaves the machine (no project codenames/paths in the ping body).
ACCEPTANCE: a ready promotion PR pings the human; no auto-merge path added (grep-verify, mirror promote.py's existing no-auto-merge test).

P2.2  COMPLETE THE ACCEPTANCE-JUDGE SLICES (UI/vision slice-2, bench slice-3)
GROUNDING: the judge covers only CLI-SSB (slice-1) today; UI features are SKIPPED (handoff: "candidates=4, judged=0, skipped_unjudgeable=4"). P0.1's surface contract makes this fall out -- the vision lane already exists (tester_vision/tester_runtime, smoke PASSED), it just isn't wired as an acceptance evidence source. Gajae's surface-typed evidence is the blueprint.
IMPLEMENTATION: route surface=ui plans to the vision lane for acceptance evidence (AX + pixel-diff), surface=bench to an artifact-file check. Fold into the P0.1 contract, do not build a parallel path.
ACCEPTANCE: a dashboard/UI plan gets a real vision verdict instead of ACCEPTANCE_SKIPPED.

P2.3  DEDUP TESTER DEFECTS BY ROOT-COMMAND, NOT FEATURE
GROUNDING: Conductor's own known item -- one `/runtime list --json` root cause spawned 3 FIX-TESTER tasks (handoff e2e-tester.md).
IMPLEMENTATION: key tester defects by normalized command, not by feature, before seeding FIX-TESTER tasks. Clear the residual deproved-judge ACCEPTANCE_REOPEN events that re-seed noise (C-27/D-8/G-8-FIX-TESTER named in the handoff).
ACCEPTANCE: one root-cause command -> one FIX task.

P2.4  DERIVE PLAN LETTERS FROM DISK, NOT HARDCODED
GROUNDING: Conductor's own item flagged 4x across handoffs -- backlog_import.LETTERS / plan_order.PLAN_LETTERS hardcoded "BCDEFGHIJK"->"...L"; M/N plans silently fail to import.
CONDUCTOR FILES: backlog_import.py, plan_order.py.
IMPLEMENTATION: glob the plan corpus dirs (docs/thefinalglance/*-plan/) for the letter set instead of a literal string.
ACCEPTANCE: dropping an M-plan dir imports without a code edit.

=============================================================================
NOTED BUT LOW-LEVERAGE FOR ELAI (do not over-invest; here so the next session doesn't re-discover them)
=============================================================================

- HASHLINE edit guard (LazyCodex, packages/hashline-core): per-line xxhash anchor, stale-hash edits hard-reject. Elegant, but ELAI workers are Devin CLI with their own edit tooling -- can't inject per-line hash gates. The concept (verify file state before mutating) is already covered by integrator.py dirty-checkout self-heal + rebase-then-ff. SKIP.
- SEMANTIC MODEL-NAME HIDING (LazyCodex: task(category="ultrabrain") not a model name, to dodge distributional self-awareness bias): N/A while the Conductor drives a single free model (glm-5.2). Revisit only if a multi-model free pool is introduced.
- MOMUS-STYLE PLANNING GATE (LazyCodex: replan until >=80% task coverage / >=90% concrete acceptance criteria / zero business-logic assumptions): ELAI is already AHEAD here -- planning-workflow PRE_GATE + acceptance-criteria injection + spec-grounding run UPSTREAM of the Conductor, which consumes pre-authored plan files. Confirmation, not a gap.

=============================================================================
WHERE ELAI IS ALREADY AHEAD (do not regress these while implementing the above)
=============================================================================

- FREE-MODEL TRUST-ANCHORING: both rivals drive PAID (GPT-5.x/Claude) or are model-agnostic with no free-tier design; ELAI is the only one explicitly built so a weak FREE model finishes work because the GATE is the anchor. P0 must not push the gate cost onto a paid model.
- FROZEN GATE READ-ONLY TO THE RUN (.claude/tools/gate_freeze_check.py + gate-manifest.txt): stronger anti-self-grading than either rival. Every new gate stage (P0.1, P1.1) MUST be added to the frozen manifest, never authored by the executing run.
- BLAST-RADIUS LEASE LOCKS (leases.effective_scope = write_set UNION blast_radius): richer than the rivals' file/task locks. Reuse for P0.4 crate scoping.
- DETERMINISM / ENTROPY QUARANTINE (ELAI runtime ClockProvider/EntropySource): neither rival has any. Keep gate evidence deterministic (seeded) so P0.3's measurements are reproducible.

=============================================================================
SEQUENCING + GUARDRAILS
=============================================================================

ORDER: P0.3 (measure) -> P0.1 (surface contract) -> P0.2 (runtime backstop) -> P0.4 (scoped verifiers) -> P1.* -> P2.*. Measure first so the rest is judged on deltas, not faith.

GUARDRAILS for the implementing session:
- The Conductor is gitignored workflow tooling; commit cadence does not apply to conductor/ but DO keep the conductor test suite green (currently 657) and run maintainability_check 0 BLOCK before declaring a stage done.
- Default-OFF every new capability behind a config.limits flag (the established pattern: acceptance_stage, test_modes, promotion_stage). Flip one at a time after monitoring confirms the prior stage stable.
- NEVER restart/kill the live daemon mid-pass (orphan + 3h free-model lockout risk). New conductor/*.py auto-loads on the next natural run-backlog pass; config is read fresh per pass. Use the handoff's EXACT-RESTART (drain + STOP + undrain) only when a change must take effect immediately.
- Every gate change is fail-CLOSED: missing evidence = FAIL, never SKIP/WARN/UNVERIFIED (wiring-enforcement R1/R2). Add it to .claude/gate-manifest.txt so the run can't weaken it.
- Reconcile with CONDUCTOR-TRUST-PLAN-2026-06-24.md (P0.2/P0.3/P0.4 overlap its P1/P2/P3) -- extend that plan's code, do not author a parallel one.
- Grounding for every rival-derived item is in this session's research; re-verify the rival source URL before copying a mechanism (the repos move fast -- LazyCodex=code-yeongyu/oh-my-openagent, Gajae=Yeachan-Heo/gajae-code).
