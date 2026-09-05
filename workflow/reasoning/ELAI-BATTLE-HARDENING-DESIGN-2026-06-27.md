# ELAI Battle-Hardening: Continuous Bug-Hunter / Validator Design (2026-06-27)

Status: DESIGN, awaiting user go before any build. Grounded from conductor code + a fresh read of state/control.db on 2026-06-27. conductor/ is workflow tooling (gitignored); ELAI lives in .elai_cc. The conductor is LIVE — every change below is default-safe or explicitly flagged.

## 0. TL;DR

The two paid codex lanes produce nothing not because the judge is broken (it works: D-8 `archive verify` was judged correctly live) but because SELECTION starves it. Selection demands a fully-DONE plan with a concrete rubric and an `elai` CLI trigger; that intersection is empty. The fix is to change the UNIT OF TEST from "a finished plan" to "a surface that exists right now" (real CLI subcommands, the live dashboard, the sidecar, SSB invariants), fed from a standing target queue that is never empty, with the paid lanes reserved for what only vision/strong-model can do and everything deterministic pushed to the free path. We do NOT lower any gate. Separately: the entire L-plan (14 hardening features) is missing from the backlog — the free workers aren't even building the hardening substrate yet.

## 1. Root cause (confirmed from code + DB, not assumed)

Re-measured control.db 2026-06-27:
- acceptance_verdicts table: 0 rows (zero real verdicts, ever).
- ACCEPTANCE events: 47 SKIPPED, 9 BLOCKED, 9 FIX_SEEDED, 13 PASS. The 13 "PASS" are pass-FINISHED summary events; every payload shows `judged:0`. So the judge ran 13 passes and judged nothing each time. The most recent pass (06-27T06:23Z) selected 0 candidates outright.
- DASHBOARD_TESTER_PASS: 1 event, both lanes `mode:judge`, `journeys_total:0`, `verdict:null`. The vision tester has never driven the dashboard in production.
- Task status: 536 DONE across 294 plans-with-≥1-DONE, but 0 plans fully DONE.

Trap #1 (the judge bar). `acceptance.py::select_acceptance_candidates` selects with `HAVING SUM(DONE)==COUNT(*)` — a FULLY-DONE plan (not "changed-DONE this pass" as the prompt assumed). With 0 fully-DONE plans, selection returns 0. When plans WERE briefly fully-DONE (the 7 UI/build plans), `AcceptanceContract.is_checkable` (needs a criterion with a concrete expected_effect) and `is_drivable` (needs `trigger_argv`, an `elai <subcommand>`) rejected them all → `rubric_not_checkable` (47) and `no_cli_trigger`. The funnel is `fully-DONE ∩ concrete-rubric ∩ elai-CLI-trigger ≈ ∅`.

Trap #2 (the vision gate). `tester_pass.py::_find_ready_milestone` requires `_all_plans_done(ms.source_plans)` — every source plan of a milestone fully DONE. 0 fully-DONE plans → no milestone ready → both lanes fall back to judge mode → trap #1 → 0.

The machinery is sound and fail-closed (SSB evidence by id, unknown=fail, air-gapped cross-family judge, non-load-bearing verdicts). The defect is purely the definition of "what is testable now."

## 2. Continuous-testing architecture (answers A + B)

### 2.1 The unit of test: a surface that exists, not a plan that finished

Invert selection. Today: "what is finished?" (empty). New: "what exists right now and can be driven?" (always large), ranked by recency-of-change, fragility, and time-since-last-tested. Four standing surface classes, each always non-empty:

1. CLI subcommands that DO exist — enumerated from the built binary's real `elai --help` subcommand tree, independent of plan status. Deterministic to drive; paid judge only where a semantic rubric exists.
2. Live dashboard surfaces — the intro, project selector, each tab/drawer/panel that renders in the dev build now. These are the vision lanes' unique value. The checkpoint map already encodes 86 features / 11 milestones with per-feature trigger + expected_effect + rubric across surfaces.
3. Sidecar endpoints — /health, /v1/messages, /search (when booted). Deterministic contract checks.
4. SSB invariants / properties — schema-version monotonicity, action_type golden lock, no orphan rows, journal completeness (K-21), replay determinism (G-2 RRA). Deterministic.

A STANDING TARGET QUEUE is populated from all four. Recently-changed code (from DONE-task events / git) gets priority — test what just landed first — but when the changed set is empty the lanes fall back to re-testing built surfaces and regression-vs-golden, so they are never idle and never select 0.

### 2.2 Real bug-finding modes (not single-shot rubric rubber-stamping)

Map modes to surfaces; deterministic-first, pay only for vision/semantics.

- Dashboard (paid vision): adversarial loop-until-dry exploration (keep navigating until N steps surface nothing new broken), visual regression vs the ~80 golden PNGs (deterministic pixel-diff FIRST to decide WHERE to look, vision only on flagged regions — the visual-ui-loop.md doctrine; tools already in .claude/tools/visual/), console-error sink (already built in cdp_action_driver.mjs), empty/error/stale/disconnected state coverage.
- CLI (free deterministic + paid judge only for semantics): differential/metamorphic (same command twice = same SSB modulo nonce; `--json` vs human output agree; idempotent commands stay idempotent), arg-surface fuzz (bad flags / missing args must fail cleanly, never panic), exit-code contract.
- Rust core / SSB (free deterministic): property/invariant assertions, replay-verify via G-2 RRA (re-execute a recorded session, assert identical SSB), schema + action_type golden lock.
- Sidecar (free): contract + soak (long-run memory/leak).
- Fault injection (free/cheap): kill the sidecar mid-call, corrupt a state row, fill the disk → assert graceful degradation, not a crash.

## 3. Efficiency model for the 2 paid lanes (answer C)

Principle: spend gpt-5.5 only on what is vision-only or genuinely needs strong-model semantic judgment; push everything decidable to the free path / the conductor itself.

- Deterministic-first gating. Run pixel-diff / AX assertions / CLI exit codes / SSB invariants / replay BEFORE any paid call. Pixel-diff-first is the single biggest credit saver (decides WHERE vision looks, or that no vision is needed). If the deterministic layer already FAILED, do not pay to confirm. If it PASSED trivially, do not pay to rubber-stamp. Pay when the deterministic layer is INCONCLUSIVE or the surface is vision-only.
- Lane split. lane-a = dashboard vision exploration (the only-it-can-do work). lane-b = semantic judge over the BROADENED changed-feature queue (CLI/IPC/sidecar features that actually landed). When the dashboard has no new surface, lane-a joins lane-b for 2x judge throughput — on the broadened queue, never the empty fully-DONE one.
- Never a no-op paid pass. The standing queue guarantees ≥1 target; additionally the conductor skips the paid call when there is no real vision/semantic question.
- Credit discipline. ~10K input tok / vision step (measured). Mitigate via AX-navigation (not full-frame every step), vlm-prep downscale, checkpoint-gating, hard caps (steps/journey, journeys/pass), loop-until-dry with a step ceiling. Dev capture = zero credits.

## 4. Production-ready bar (answer D) — measured, tied to existing trust doctrine

Not a vanity number. "Battle-hardened" = all of:
- Coverage-of-checkpoint-map: fraction of the 86 features with ≥1 off-agent-evidenced verdict in the last N days. Today ≈ 0; target a climbing, sustained number.
- Found-defect rate: defects surfaced per 100 test-units over time. A hardener that finds 0 is broken, not perfect. Expect it to start high and DECAY as defects are fixed — the decay-to-a-floor is the hardening signal.
- Wire-dark rate (existing metric): the tester is the runtime backstop that converts static-green into actually-runs; target 0 unexplained wire-dark in shipped slices.
- Regression catch: golden-frame + SSB-golden diffs catch a regression before integration.
- Replay determinism (G-2 RRA) holds across the recorded corpus.
- Feeds the SAFEDELTA reproducible-run envelope (the existing believe-the-number artifact), not a bare number.

## 5. Composition with the existing gate stack (answer E)

This is the Layer-3/4 "does it actually work + find defects" capstone, not a parallel silo.
- Keeps, does not duplicate: Layer 1/2 static wire-checkers, mutation testing (C-28), cant-lie test-integrity (F-4), production-caller gate, maintainability gate.
- Consumes their outputs: a wire-dark finding or a low-mutation-score module becomes a PRIORITY test target.
- Adds the legs they structurally can't: runtime drive of a real entrypoint, adversarial vision exploration, metamorphic/property/fault/replay.
- Stays fail-closed + off-agent + non-load-bearing: verdicts display + re-flag, never ship code or weaken a gate; a found defect OPENS a fix task (autofix_stage) which then goes through the full frozen gate. The free workers fix; the frozen gate verifies; the next paid pass re-tests.

## 6. Phased, lowest-risk implementation plan (with exact file seams)

- Phase 0 (zero-build, do now, flagged): incremental-import the missing L-plan so the free workers build the hardening substrate. Seam: `python conductor/backlog_import.py --incremental` (corpus_root already covers docs/thefinalglance incl L-plan; `--incremental` inserts only absent task ids, never rewrites existing rows). Back up control.db first. Default-safe to the daemon (config + tasks read fresh per pass). See §7.
- Phase 1 (broaden judgeable, default-safe): add a standing target queue alongside the fully-DONE selection. Seams: new conductor/target_queue.py (build targets from `elai --help` + changed-DONE-tasks + checkpoint map + SSB invariants); acceptance.py::select_acceptance_candidates gains a second source; add IPC/dashboard/sidecar drivers so "no CLI trigger" is no longer an automatic skip. Keep is_checkable/is_drivable as the rubric quality bar.
- Phase 2 (continuous vision): decouple tester_pass.py::_find_ready_milestone from all-source-plans-DONE — every dashboard journey in the checkpoint map is a standing target (the surface exists in the dev build now). Add the pixel-diff-first gate (visual-ui-loop.md tools already present). The vision loop itself is already built + live-smoked.
- Phase 3 (real bug modes): add metamorphic/differential CLI + SSB invariant + replay-verify (G-2) as free conductor-side probes. Seam: extend verify_runtime.py or a new test_modes.py; register in the standing queue.
- Phase 4 (measurement): coverage-of-checkpoint-map + found-defect-rate + decay rollup. Seam: dashboard_server.py /api/acceptance tile + a metrics rollup.
- Phase 5 (close the loop): flip autofix_stage so found defects open fix tasks the 3-4 free workers build, then the next paid pass re-tests. This is the flywheel.

## 7. The 3-4 free workers + 2 paid lanes flywheel (the efficiency answer the user asked for)

- The 3-4 free glm-5.2 Devin workers BUILD (implement→wire→integrate→test→audit→polish) under the frozen gate. The 2 paid gpt-5.5 codex lanes are the independent adversarial QA layer on top.
- The flywheel: paid lanes FIND defects on the live surfaces → autofix opens fix tasks → free workers FIX → frozen gate VERIFIES → next paid pass RE-TESTS. Hardening compounds.
- Feed the builders the right work: import L-plan (claim-risk, evidence round-trip, semantic entropy, hallucination policy gate, output-quality enforcement — the literal hardening features) so the free workers are building the substrate the paid lanes will exercise.
- Separation of pools (already enforced): free Devin workers and paid codex lanes are distinct accounts, no contention. The free-worker bottleneck is the red baseline + cargo-gate concurrency (separate, pre-existing), not the tester.

## 8. Findings to act on immediately (low-risk, flagged for go)

1. L-plan absent from backlog. Disk has L-1..L-14 (14 plan files); control.db has 0 L plans. K is complete (23/23; K-10 does not exist). Action: backup + `backlog_import.py --incremental`. Recommend yes.
2. Selection traps are the whole story. No gate is too strict in the wrong direction — the wrong direction is the selection unit. The doc above does not weaken a single gate.
3. The numbers drift. acceptance_verdicts still 0; re-measure before any go decision is finalized.

## 9. Open decisions for the user (go/no-go)

- Phase 0 (import L-plan) — recommend running now after a control.db backup.
- Build order for Phases 1-5 — recommend 1→2→4 first (broaden + see it + measure) before 3/5 (more modes + autofix), so we get the lanes producing real verdicts fast, then deepen.
- Whether the paid lanes should also adversarially explore .elai_cc dashboard surfaces that are not yet in the checkpoint map (extend the map) or stay strictly map-bound.
