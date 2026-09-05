CONDUCTOR MERGE LANE — design (2026-06-28)

author: Opus 4.8 controller, after a manual fork->main reconcile session that proved the per-slice pipeline and exposed why doing it by hand does not scale. NEXT SESSION IMPLEMENTS THIS. This is a design doc, not code.

================================================================
1. PROBLEM
================================================================
The conductor drives free Devin workers to finish ELAI. Their output lands ONLY on the isolated fork conductor/elai-repo (branch master, allow_push=false, .elai_cc-rooted, NO shared history with github.com/maintainer/.ELAI_workflow). The fork has accumulated 124 commits (119 "Conductor integrate") = a 318-file diff that has NEVER reached real main. There is a wholesale promote.py pipeline (squash -> gate -> PR -> free-GLM advisory review -> human merge) but it was correctly gate-blocked once (the whole 119-commit candidate fails as one lump) and it is not how we actually want to land: we want INCREMENTAL, per-slice, hand-quality commits.

This session landed 2 fork slices by hand (agent C-45 dropper, reasoning cleanup) plus the user's local base, and the manual pass immediately caught 3 things a blind merge would have shipped:
  - knowledge: the only change was ort default-features=false + alternative-backend, which DISABLES ONNX runtime linking purely to make the Windows test gate pass (worker's own comment admits it). A Goodhart: green proxy, broken capability.
  - tools: the "slice" was a stray I-46-AUDIT-1-findings.md litter file dropped into crates/tools/. Not code.
  - memory: 4231 lines of new public surface (activation_decay, retrieval_authorization, raw_evidence, acceptance_evidence, semantic_validation) with ZERO production callers anywhere in the fork = wire-dark machinery (wiring-enforcement R6 BLOCK).

CONCLUSION (user's call, grounded by the above): manual is too slow and the verification is exactly the kind of deterministic + free-model work the conductor should own. Build a MERGE LANE: a dedicated free GLM "merge owner" that walks an FSM, one slice/phase at a time, mirroring our ELAI orchestration (wiring sweep + integration checks + review), commits + pushes the green ones to real main, quarantines the landmines, and — once caught up — runs steady-state so a worker change = its own gated commit+push and the backlog never re-accumulates.

================================================================
2. WHAT ALREADY EXISTS (reuse, do not reinvent)
================================================================
- conductor/waves.py: WAVE_IMPLEMENT -> WAVE_WIRE_SWEEP -> WAVE_INTEGRATION -> WAVE_AUDIT -> WAVE_POLISH (+ read-only filler waves STATE_CHECK/EXPLORE/VERIFY/ACCEPTANCE leased via the saturate-workers borrow path). The conductor ALREADY mirrors the ELAI FSM up to DONE-on-fork.
- conductor/promote.py public API: build_promotion_candidate / build_promotion_candidate_percommit (worktree clone of origin/main + re-root fork under .elai_cc/), run_promotion_gate (cargo check+test, FAIL-CLOSED, baseline-masked via cargo_gate_support.py), open_promotion_pr, dispatch_glm_reviewer (free glm-5.2 reviews a diff -> {recommend_merge,rationale,concerns}), parse_glm_verdict, post_review_comment, run_promotion_pipeline. These are the building blocks; the merge lane drives them at SLICE granularity instead of wholesale.
- The frozen off-agent gate stack (wiring-enforcement.md): production_caller_check.py (.rs wire-dark), python_wire_check.py (.py), maintainability_check.py (MR-1..4), traced_test_assert_check.py, gate_freeze_check.py (.claude/gate-manifest.txt). cargo_gate_support.py (baseline mask). These ARE the deterministic gate the free model must NOT be able to edit (R8).
- The air-gapped acceptance judge (acceptance.py + acceptance_judge.py + guard.py): the pattern for a source-stripped, cross-family, fail-closed free-model judgement. The merge REVIEW step reuses this shape.

================================================================
3. THE ELAI FSM WE ARE MIRRORING (planning-workflow SKILL.md)
================================================================
IDLE -> PLANNING -> TESTING_FIRST -> EXECUTING -> AUDITING -> MEMORIZING -> COMMITTING -> (next) | COMPLETE -> IDLE.
orchestrate-protocol per sub-phase: context-compiler + integration-explorer (parallel, cap 3) -> executor -> auditor (1 cycle, sole test runner) -> memory-writer -> commit ONCE. Off-agent frozen gates; producer+consumer+smoke in the SAME unit (walking skeleton); missing evidence = FAIL.

The merge lane is the SAME machine applied to "land a verified fork slice onto real main" instead of "implement a plan step." Mapping:

  ELAI phase        MERGE-lane phase     who runs it
  ------------------------------------------------------------------
  PLANNING/SELECT   SELECT_SLICE         deterministic (slice queue)
  context+explore   CHARACTERIZE         free GLM (merge-explorer)
  TESTING_FIRST     APPLY                deterministic (worktree apply)
  AUDITING          GATE                 deterministic FROZEN checkers (sole test runner)
  (review)          REVIEW               free GLM (merge-reviewer, air-gapped)
  COMMITTING        COMMIT_PUSH          deterministic (one commit, maintainer voice, maintainer, push)
  MEMORIZING        RECORD               deterministic (merge ledger + dashboard)

================================================================
4. THE SLICE MODEL (the unit of one "phase")
================================================================
A SLICE is a coherent landable change, NOT a raw "Conductor integrate" commit. Slicing strategy (proven this session):
  - by crate: the fork's whole diff for one crate (a84e5bb..forkHEAD -- crates/<X>), applied onto current origin/main via `git apply --directory=.elai_cc`. Most crates have 0 collisions with main and apply clean.
  - dashboard-v2 is its own slice (svelte/build gate, not cargo test).
  - contracts is its own slice and goes LAST (land only test files green on current main; the fork's red/contradictory tests live here).
  - non-crate dirs (skills/ python/ docs/) reviewed separately.
  - COLLISIONS (files changed by both the fork and what already landed on main): per-file 3-way merge, escalated if non-trivial. This session had only 8 collision files across the whole fork.
Dependency order: source crates first (agent[DONE], reasoning[DONE], memory, providers, ...), then their cross-crate consumers (cli, orchestrator), then dashboard, then contracts. A slice that needs a consumer not yet landed is deferred, not forced (wire-dark rule).

Slices are regenerable any time from the fork (nothing is consumed); the lane is restartable.

================================================================
5. THE MERGE-LANE FSM (one slice = one phase; run one phase per pass)
================================================================
SELECT_SLICE (deterministic)
  - Pop the next slice from merge_queue in dependency order. Skip slices whose prerequisite consumer slice is not yet landed.
  - Compute the fork patch for the slice's paths; if it does not apply --check clean onto origin/main, mark COLLISION and route to the collision sub-path (3-way; escalate on conflict).

CHARACTERIZE (free GLM: merge-explorer; mirrors context-compiler + integration-explorer)
  - Input: the slice diff + the touched crates. Output (structured JSON, schema-validated):
      new_pub_symbols[], production_consumer_seam (file+fn that calls each new pub symbol, or NONE),
      new_deps[] (with the anti-slopsquat verdict), risk_flags[] (see classifiers below), one-line summary.
  - This is the seam-naming the executor/auditor need (PRODUCTION_HOOK). If a new pub symbol has consumer=NONE, the slice is presumptively wire-dark -> CHARACTERIZE marks it for the GATE's wire-dark check.

APPLY (deterministic)
  - In a worktree clone of origin/main, `git apply --directory=.elai_cc <slice.patch>`. Exclude fork infra cruft (the 2 fork-base commits: vendored dashboard dist/, the AGENTS.md devin-contract swap) and any litter the classifiers flag.

GATE (deterministic, FROZEN, sole test runner — the load-bearing leg)
  Run, fail-closed, missing-evidence=FAIL:
   a. cargo check -p <crate(s)>  (must be green)
   b. cargo test -p <crate(s)>   (vs the baseline-masked known-reds at origin/main; only NEW failures fail the slice; cargo_gate_support.py)
   c. production_caller_check.py / python_wire_check.py over the slice's changed files  -> any DARK new pub fn with no WIRE-DARK row = BLOCK (the memory-crate catch)
   d. maintainability_check.py (MR-1..4)
   e. traced_test_assert_check.py if the slice adds #[traced_test]
   f. THE THREE NEW CLASSIFIERS this session's landmines demand (implement as small frozen checkers):
       - GOODHART-DEP: flag a Cargo.toml change that adds default-features=false / removes features / sets alternative-backend on an EXISTING production dep, especially with a comment mentioning "test gate"/"link"/"skip"/"Windows". (caught the ort/ONNX disable.)
       - LITTER: flag a NEW non-source file landing under crates/*/src or a crate root (*.md findings, audit-scratch, *.log, scratch dirs). (caught the I-46-AUDIT-1-findings.md.)
       - DEP-SLOPSQUAT: any new external dep runs the anti-slopsquat registry check (name/version/last-publish) before it can land.
  GATE is frozen in .claude/gate-manifest.txt; the merge-owner CANNOT edit it (R8, gate_freeze_check.py at lane start). This is critical: the merge owner is a free model WITH push access; its evaluator must be immutable to it.

REVIEW (free GLM: merge-reviewer; air-gapped, reuses acceptance_judge/guard shape)
  - Input: the slice diff + the GATE result summary (NOT the gate's internals). Output (schema): verdict in {LAND, QUARANTINE, ESCALATE}, rationale, concerns[].
  - This is advisory ON TOP of the deterministic GATE, never instead of it: a GATE BLOCK is final (no model can override a failed frozen gate). REVIEW can only DOWNGRADE a gate-pass to QUARANTINE/ESCALATE (catch a smell the static gate missed), never UPGRADE a gate-fail to LAND.

DECIDE (deterministic)
  - GATE green AND REVIEW=LAND  -> COMMIT_PUSH.
  - GATE green but new pub symbol intentionally dark (kill-switch/observability, named in CHARACTERIZE) -> require a WIRE-DARK[<slice-id>] marker + wire-dark-allowlist row to be present, else QUARANTINE.
  - GATE BLOCK or REVIEW=QUARANTINE -> QUARANTINE (record reason + evidence; do NOT land; do NOT block the lane; move to next slice).
  - COLLISION non-trivial or REVIEW=ESCALATE -> ESCALATE to the human queue (dashboard tile), lane continues with other slices.

COMMIT_PUSH (deterministic)
  - ONE commit per slice. Message = main-repo voice (maintainer): lowercase i, no em-dash, no "->", descriptive prose, NO robotic prefix, NO AI trailer, and HARD: never mention "conductor" or internal autonomous-build machinery — describe the change as if hand-authored (this session leaked "land the conductors ..." and had to force-reword; do not repeat).
  - Author = maintainer (the repo had a junk local user.name=t / user.email=t@t override; the lane MUST git config --unset it / commit with the correct identity).
  - git push origin main (fast-forward only; NEVER force-push from the lane). This is the ONE place the conductor touches real main; everything else stays allow_push=false.

RECORD (deterministic; mirrors MEMORIZING)
  - Upsert merge_ledger: slice_id -> {status: LANDED|QUARANTINED|ESCALATED, commit_sha, reason, fork_range}. Emit MERGE_* SSB/control events. Update the dashboard "Merge Lane" tile.

================================================================
6. CATCH-UP MODE vs STEADY-STATE (the "never re-accumulate" requirement)
================================================================
CATCH-UP (run once, drains the 124-commit backlog):
  - Seed merge_queue from the fork diff a84e5bb..forkHEAD, sliced per section 4. Walk the FSM one slice/pass until the queue is LANDED-or-QUARANTINED-or-ESCALATED empty. Already LANDED this session: agent, reasoning (+ the user's local base). Already QUARANTINED: knowledge (ort), tools (litter). Remaining: memory(wire-dark, needs consumer or WIRE-DARK reg), providers, cli, orchestrator, dashboard-v2, contracts, non-crate, 8 collisions.

STEADY-STATE (runs forever after catch-up):
  - Hook the merge lane onto the END of the fork pipeline: when a worker task reaches DONE + (VERIFY/ACCEPTANCE) on the fork, its incremental delta since the last landed fork commit is enqueued as a merge slice immediately. The lane gates + lands it the same pass. So fork-HEAD and origin/main stay within ~one slice of each other; a 100+ backlog can never form again.
  - Implementation: a new WAVE_MERGE (NOT in WAVE_ORDER; leased like the other read-only-ish filler waves but with the single push-capable merge-owner slot) that fires after WAVE_INTEGRATION/WAVE_VERIFY, processing the fork delta of just-integrated tasks.
  - The "last landed fork commit" watermark lives in merge_ledger so restart is idempotent.

================================================================
7. THE MERGE OWNER (one dedicated free GLM slot)
================================================================
- ONE worker slot, model glm-5.2 (High; free, proven). It runs CHARACTERIZE + REVIEW only. The GATE + APPLY + COMMIT_PUSH are deterministic Python the model never executes. This keeps the model on judgement, not on the irreversible git ops.
- It is the ONLY slot with push access to origin/main, behind: gate_freeze_check at lane start, the frozen GATE, fast-forward-only push, one-commit-per-slice, and a kill switch (config flag merge_lane=false, default OFF until the user arms it).
- Separate from the implement/wire/audit worker pool (no contention; respects the free-model concurrency cap — the merge owner counts as one free-model session).

================================================================
8. QUARANTINE + ESCALATION QUEUE (human-facing, the only manual touchpoints)
================================================================
- QUARANTINE table: slice_id, reason (wire-dark-no-consumer / goodhart-dep / litter / gate-block:<detail> / review-smell), the evidence, the fork range. Surfaced on a dashboard "Merge Quarantine" tile. The user dispositions these in batches (land-with-WIRE-DARK-marker, fix-forward as a new fork task, or drop). The lane NEVER blocks on them.
- ESCALATION: non-trivial collisions + REVIEW=ESCALATE. Same tile, separate bucket.
- This is the answer to "i don't want to merge everything manually": green slices auto-land; only genuinely-ambiguous ones reach the human, as a short queue, not a 119-commit wall.

================================================================
9. CONCRETE BUILD PLAN FOR NEXT SESSION (files, all under conductor/)
================================================================
NEW:
  merge_lane.py        — the FSM driver (SELECT/APPLY/GATE/REVIEW/DECIDE/COMMIT_PUSH/RECORD); reuses promote.py primitives at slice grain.
  merge_slicer.py      — fork-diff -> ordered slice queue (per-crate, dashboard, contracts, non-crate, collisions); collision detector.
  merge_classifiers.py — GOODHART-DEP, LITTER, DEP-SLOPSQUAT frozen checkers (small, regex/AST, fail-closed).
  merge_explorer.py    — free-GLM CHARACTERIZE dispatch + schema.
  merge_reviewer.py    — free-GLM REVIEW dispatch (downgrade-only) + schema; reuse guard.py air-gap.
  migrations/00X_merge_ledger.sql — merge_ledger + merge_quarantine tables + the fork watermark.
  tests/test_merge_*.py — gate-blocks-on-wire-dark, goodhart/litter caught, downgrade-only review, ff-only push, one-commit-per-slice, voice/identity, idempotent restart, collision routing.
EDIT:
  waves.py            — add WAVE_MERGE (NOT in WAVE_ORDER).
  conductor_loop.py   — lease the merge-owner slot; fire WAVE_MERGE after INTEGRATION/VERIFY (steady-state) + a catch-up entrypoint.
  conductor_config.py — merge_lane flag (default False), merge_owner slot, push target = github main.
  conductor.py        — `conductor.py merge --catch-up` (drain backlog) and the steady-state wiring.
  dashboard_server.py — "Merge Lane" + "Merge Quarantine"/"Escalation" tiles.
  .claude/gate-manifest.txt — freeze the 3 new classifiers + merge_lane gate so the merge owner cannot weaken them.

GATE FREEZE + SAFETY are non-negotiable: a free model with push access whose evaluator it could edit would write itself to PASS (the exact in-boundary failure freeze-the-gate exists to prevent). gate_freeze_check at lane start; ff-only push; default-OFF; kill switch.

================================================================
10. GROUNDING (this session, verifiable)
================================================================
- origin/main moved 9b698b2 -> 65695cd -> edd1ed2 -> 8f530a1 (3 commits, maintainer-authored, pushed). Per-slice pipeline proven: extract -> apply --check -> cargo check -> independent auditor (cargo test -p + production_caller_check + maintainability_check + zero-lazy) -> commit (maintainer voice) -> push. agent: 37/37 tests. reasoning: 108/108 tests.
- Landmines the manual gate caught (the classifiers in section 5f are derived directly from these): knowledge ort-disable Goodhart; tools litter md; memory 4231-line wire-dark (0 production consumers, confirmed by cross-crate symbol scan).
- Fork geometry: base a84e5bb == .elai_cc @ 9b698b2 == old origin/main, so slices apply clean (only 8 collision files). The old e2e-tester handoff's "104 non-baseline failures / schema 97-vs-253" was measured vs a stale baseline (2a35109) and is overstated; the real per-slice new-failure count is what the GATE measures.
- Companion handoff (live slice queue + SHAs + collision list): workflow/handoff/conductor-fork-reconcile.md. The fork/daemon/tester are tracked in workflow/handoff/e2e-tester.md and conductor-build.md.
