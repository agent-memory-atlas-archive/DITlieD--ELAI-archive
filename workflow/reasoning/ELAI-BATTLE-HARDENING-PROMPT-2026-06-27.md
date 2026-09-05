PROMPT FOR A REASONING SESSION — battle-harden ELAI into a production-ready, always-on bug hunter / validator / tester

You are picking this up cold. This is a REASONING + DESIGN task, not a build task (yet). Produce a grounded plan; do not start coding until the design is sound and the user has confirmed. Honor the project house rules (grounded-by-default, anti-sycophancy, off-agent verification, the wiring doctrine, no fabricated claims). Use orchestration/sub-agents if it helps, but the strategy and the verdict are yours.

THE GOAL (what the user actually wants)
Turn ELAI's testing/validation capacity into a PRODUCTION-HARDENING machine: a continuous bug hunter + validator + fake-human tester that finds real issues all the time, so ELAI becomes a trustworthy, production-ready artifact. Right now we have TWO paid codex API lanes (account-a + account-b, gpt-5.5 vision-capable via the bmapi proxy) that are supposed to test ELAI — and they are doing ESSENTIALLY NOTHING. The user's words: "we have 2 apis and they are doing nothing.. 0 judge work.. what is happening and what can we do? can we be more efficient?"

GROUNDED CURRENT STATE (measured 2026-06-27 from conductor/state/control.db — these are facts, re-verify if stale)
- ACCEPTANCE judge: 78 ACCEPTANCE* events total = 47 SKIPPED, 9 BLOCKED, 0 VERDICT EVER. acceptance_verdicts table = 0 rows. The judge runs each backlog pass but never produces a verdict.
- E2E dashboard tester: 1 DASHBOARD_TESTER* event total. It has effectively never done UI work.
- Backlog: 536 DONE tasks across 294 plans that have >=1 DONE task, but ZERO plans are FULLY DONE (every plan still has non-DONE tasks).
- Both lanes are flag-on (acceptance_stage=true, dashboard_tester_stage=true in conductor/config/conductor.toml).

WHY THE LANES ARE IDLE (the two structural traps — verify + reason past these)
1. The JUDGE bar is too narrow. acceptance.py only judges a feature that is (a) changed-DONE this pass, (b) has a resolvable `elai <subcommand>` CLI trigger (is_drivable requires trigger_argv), AND (c) has a "checkable" rubric. The current DONE set is mostly UI/build/internal features with NO `elai` CLI trigger, so selection SKIPS them all (47 SKIPPED). Net: the judge's definition of "judgeable" excludes almost everything that actually gets built.
2. The E2E TESTER gate is unreachable. tester_pass.py UI-mode only fires when a milestone's source_plans are ALL DONE; with 0 fully-DONE plans (and the new cross-plan dependency gating sequencing completion even harder), it never triggers, and falls back to judge-mode which hits trap #1. So the second paid lane also produces 0.
Result: built capacity, zero output. The paid lanes idle = wasted money + ELAI stays unhardened.

THE CORE QUESTIONS TO REASON ABOUT (answer each, grounded)
A. WHAT should ELAI's continuous tester actually test, given that nothing is ever "fully DONE"? Decouple testing from "fully-DONE plan/milestone" and "CLI-trigger-only" — what is the right unit of test (per-feature-as-it-lands? the running dashboard surface? CLI subcommands that DO exist? the sidecar? invariants/properties?) so the lanes always have real work?
B. HOW do we make it find issues ALL THE TIME, not just rubber-stamp? What testing modes beyond the current single-shot rubric judge actually surface bugs: fuzzing, metamorphic/differential testing, property/invariant checks, replay (the G-2 RRA), regression vs golden, adversarial vision exploration of the dashboard, fault injection, long-run/soak? Which fit ELAI's surfaces (CLI, Tauri+Svelte dashboard, Rust core, Python sidecar)?
C. EFFICIENCY: the 2 codex lanes are paid + currently idle. What is the highest-value continuous use of exactly 2 concurrent gpt-5.5 lanes (one per account)? How to keep them saturated with real bug-hunting work without burning credits on no-op passes? When is the cheap path (deterministic gates, free glm) the right tool vs the paid vision/judge lane?
D. WHAT does "production-ready / battle-hardened" mean for ELAI concretely, and what is the measurable bar (a real number / a SAFEDELTA-style envelope / a wire-dark rate / a found-bug rate)? Tie it to the existing trust doctrine, don't invent a vanity metric.
E. WHERE does this plug into the EXISTING gate stack so it composes instead of duplicating: the 4-layer wiring enforcement, mutation testing (C-28), cant-lie test-integrity (F-4), the production-caller gate, the maintainability gate, the RRA replay (G-2)? The tester should be the missing "does it actually work + find real defects" layer on top, not a parallel silo.

MUST-READ FILES (in this order)
1. workflow/handoff/e2e-tester.md — the full build history of the 2-lane judge + fake-human dashboard tester (what exists, the modules, the flags, the live smoke, the plan-order work). START HERE.
2. workflow/handoff/conductor-build.md — the conductor daemon that drives the autonomous build + runs the test lanes (waves, scheduler, the acceptance_stage hook, operational gotchas). Head-first; it is long.
3. The tester/judge code: conductor/acceptance.py (selection + is_drivable/is_checkable — trap #1), conductor/acceptance_judge.py (the air-gapped judge ladder), conductor/tester_pass.py (the milestone-ready gate — trap #2), conductor/tester_vision.py + tester_lane.py + tester_runtime.py + cdp_driver.py (the fake-human vision loop), conductor/conductor_loop.py (~line 1176-1213: where acceptance_stage + dashboard_tester_stage run at end-of-pass).
4. The wiring/trust doctrine (the bar the tester must meet + compose with): .claude/rules/wiring-enforcement.md (R1-R10, the 4-layer gate), .claude/rules/production-caller-gate.md, .claude/rules/tracing-test-assert.md, .claude/rules/maintainability-review.md.
5. The targets + map: docs/thefinalglance/ELAI-FEATURE-MAP.md (what ELAI is supposed to do + status), workflow/reasoning/ELAI-TEST-CHECKPOINT-MAP.json (86 features / 11 milestones with per-feature trigger + expected_effect + rubric — the journey source), workflow/reasoning/CONDUCTOR-PLAN-SUPERSESSION-DESIGN-2026-06-27.md (the dependency-order work just shipped).
6. config: conductor/config/conductor.toml (the flags: acceptance_stage, dashboard_tester_stage, and the wave model).
7. Strategy context (optional but useful for the "production-ready bar"): the trust/market docs referenced in the auto-memory (project_trust_maximization_roadmap, project_efficiency_trust_engineering) + docs/strategy/.

DIRECTIONS TO CONSIDER (seeds, NOT answers — reason about each, keep or reject with grounding)
- Broaden "judgeable": judge features by their REAL surface, not only `elai` CLI triggers — dashboard journeys (vision), IPC/Tauri commands, sidecar endpoints, SSB invariants. The checkpoint map already encodes per-feature triggers + expected effects across surfaces; use it.
- Decouple from fully-DONE: test a feature the moment its own slice lands (per-DONE-task or per-DONE-plan-slice), not per fully-DONE milestone. Continuous, incremental coverage.
- Make the lanes adversarial + continuous: a standing queue of test targets (built features + recently-changed code + known-fragile areas) so the 2 lanes are never idle; loop-until-dry bug hunting over the live dashboard + CLI + sidecar.
- Add real bug-finding modes (not just rubric scoring): metamorphic/differential checks, fuzz inputs, property/invariant assertions, fault injection, replay-verify (G-2 RRA), soak runs, visual-regression vs golden frames.
- Efficiency: reserve the paid vision lanes for what only vision/strong-model can do (exploring the rendered dashboard, semantic bug judgment); push everything deterministic to the free/cheap gates; never spend a paid pass that selects 0 targets.
- Measurement: instrument a found-defect rate + a coverage-of-the-checkpoint-map number so "more hardened" is measured, not asserted (house rule: judge on measured deltas, not faith).
- Compose, don't silo: this is the Layer-3/4 "does it actually work + find defects" capstone on the existing wiring/mutation/cant-lie gate stack.

CONSTRAINTS (binding)
- Grounded-by-default: every claim about how the conductor/tester/ELAI works must be verified from the code or measured from the DB, not assumed. Re-measure the numbers above (they drift).
- Off-agent verification + fail-closed: the tester must measure artifacts it did not author; missing evidence = FAIL, never SKIP-as-pass. Do NOT let the tester rubber-stamp (the whole point is it finds issues).
- Do NOT weaken any existing gate to make the lanes "do work." The fix is broadening REAL coverage, not lowering the bar.
- conductor/ is workflow tooling (gitignored), NOT .elai_cc. ELAI itself lives in .elai_cc. Keep the boundary.
- The conductor is LIVE (driving the autonomous build). Any change must be default-safe / not disturb the running daemon; flag risky/irreversible actions to the user first.

DELIVERABLE
A grounded design (write it to workflow/reasoning/) answering A-E with: (1) the diagnosed root cause confirmed from code+DB, (2) the proposed continuous-testing architecture (what tests what, when, on which lane), (3) the efficiency model for the 2 paid lanes, (4) the production-ready measurable bar, (5) how it composes with the existing gate stack, (6) a phased, lowest-risk implementation plan with the exact file seams. Then bring it to the user for a go before building.
