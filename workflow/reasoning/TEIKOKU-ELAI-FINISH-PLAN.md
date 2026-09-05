TEIKOKU-ELAI FINISH PLAN — one orchestrating session, one queue, everything pending lands
authored 2026-07-05. PROOF_SPINE: feeds PS-6 (the walk IS the wire-dark closure; PS-1..PS-5 owner
units sit inside the queue as transitive feeders). user-acked this session.

WHAT THIS IS
one plan, walked start-to-finish by ONE orchestrating session, in two parts:
  PART 1 (waves 0-3): finish the ORCHESTRATION TOOLING — merge the standalone glm-orchestrator's
    proven loop traits into teikoku's conductor, stand up the new Claude Max implementer lanes,
    swap the free lane to devin swe-1.6, wire codex gpt-5.5 as a test/benchmark provider INSIDE
    elai, and ship the canvas provider picker.
  PART 2 (wave 4+): run the full ELAI backlog walk THROUGH teikoku on those lanes until the
    punch list + wire-dark + pending items are closed. items with dependencies enter the queue
    only after their whole transitive dep closure is DONE.
this file merges and supersedes the open items of: teikoku-glm-orchestrator-merge.md (M1-M7,
P1-P10), teikoku-glm-walk.md, conductor-j-burn.md, teikoku-general-orchestrator.md (W7/W9),
teikoku-gate-redesign.md (deferred rows), teikoku-native-dashboard.md (open rows), teikoku-build.md,
and workflow/reasoning/plan-audit-2026-07-02/_MASTER-SYNTHESIS.md (the seed backlog). the source
files stay as history; THIS file is the queue.

LOCKED DECISIONS (owner, 2026-07-05 — never re-litigate)
D1 implementer = the dedicated .claude-elai account UPGRADED TO MAX (mechanism + all 6 token
   seams already wired per workflow/handoff/claude-multi-account-isolation.md; upgrade is the
   only missing piece). the default Max account stays for everyday work.
D2 model split on the paid lane: ONE opus slot for EXECUTE, ONE opus slot for WIRE+INTEGRATE
   (all wiring/integration checks), sonnet for everything else (AUDIT, POLISH, verification-debt
   re-sweeps, doc drift). delta vs the already-built config/conductor.elai-opus.toml: the
   opus-audit slot becomes sonnet.
D3 codex gpt-5.5 credits (~4,384 left, EXPIRE 2026-07-15) are NOT an implementer anymore. they
   become a test/benchmark provider INSIDE elai: per integrated feature an e2e smoke (BLOCKING)
   + benchmark deltas (ADVISORY, ledger only — a benchmark can never wedge the queue, and the
   queue survives credit expiry). blocking-vs-advisory split was delegated to the author.
D4 free glm lane is DEAD (window closed 2026-07-05). free lane = devin bridge swe-1-6 sessions.
D5 teikoku canvas gets a per-run PROVIDER PICKER: choose the implementation lane (claude-elai
   Max vs devin swe-1.6) next to the existing walk Start/Stop button.
D6 backlog seed = the _MASTER-SYNTHESIS punch list; SUPERSESSION RULE: from the first PART-2
   wave onward the open set is re-derived at every wave start from the live wiring checkers
   (elai_status_query.py --punch for triage, production_caller_check.py / python_wire_check.py /
   wiring_contract_check.py for verdicts). the static synthesis file is retired once its items
   are consumed; it never discharges a done-gate (R8-frozen boundary unchanged).
D7 ONE session orchestrates the whole plan, launched on the dedicated account (elai-claude.ps1 /
   elai-code.ps1) so all controller+worker spend lands on the new Max plan. orchestrate-protocol
   H-1..H-11 binds: agent dispatch non-optional, preamble pointer on every dispatch, off-agent
   audit, H-11 integration gate at every wave end — never a bare edit loop.
D8 PROOF_SPINE: feeds PS-6.

WHY THE ENGINE WORK COMES FIRST (the core finding, do not re-derive)
teikoku's walk produced 0 INTEGRATION_PUSHED ever while the standalone glm-orchestrator landed
~70 commits on the SAME corpus under the SAME frozen gates (teikoku-glm-orchestrator-merge.md).
the difference is job shape, not gates: the standalone engine mostly RECOGNIZED already-shipped
plans (precheck -> certify/polish) and only executed the genuine remainder; teikoku decomposes
everything into greenfield per-task EXECUTE and fed it to free glm, which fails strict gates on
hard C/D/E work (37/40 stuck tasks = genuine gate_fail). so: no walk resumes until M1+M3 land,
and genuine greenfield goes to the PAID lane only (M6).

======================================================================
WAVE 0 — PREREQS (owner actions + posture; no deps)
======================================================================
0.1 [OWNER] upgrade .claude-elai to Max. then confirm the model ids the account serves
    (claude-opus-4-8[1m], claude-sonnet-5) and adjust config/conductor.elai-opus.toml if the
    served ids differ. re-login recipe: claude-multi-account-isolation.md HOW TO PICK UP step 3.
0.2 [OWNER ack] commit posture for the teikoku distro (P10): the whole conductor change pile
    (wave-deadlock fix, diskguard, walk_control, parallel_undeclared_scopes, worktree/ext4 moves,
    lane configs) is UNCOMMITTED on ~/teikoku. recommendation: one maintainer-voice checkpoint commit
    before wave 1 so the merge lands on a clean tree. the 4 hook edits live in ~/.claude OUTSIDE
    the repo (P6) — record them in the distro gate-manifest re-lock or they die on a rebuild.
0.3 codex credits sanity: confirm balance/expiry on the proxy dashboard (4,384 @ 07/15 at last
    check, conductor-j-burn.md) + whether the 656 points balance survives 07/15.
0.4 backend up + walk STOPPED (it is; leave it stopped until wave 2 completes).

======================================================================
WAVE 1 — ENGINE MERGE (teikoku conductor; blocks the entire walk)
sources: teikoku-glm-orchestrator-merge.md M1-M7 / P1-P10. all units on the paid lane once 0.1
lands (or main-session-direct if teikoku work stays controller-driven as before — owner's
standing "improve teikoku alone" decision covers either).
======================================================================
1.1 M1 PRECHECK ROUTING [highest leverage; opus EXECUTE class]
    port the standalone engine's gates.precheck(plan, repo_root) + done-probe routing into the
    conductor as a pre-decomposition step: a plan whose static contract + produced-symbol wiring
    ALREADY pass over the current tree -> its tasks are marked DONE (certify, never execute);
    step files with no WIRING_CONTRACT -> one cheap model DONE-probe; only genuine remainder
    enters EXECUTE waves. study workflow/glm-orchestrator/gates.py + orchestrator.py ROUTE
    section. PRODUCTION_HOOK: the wave scheduler path in conductor_loop.py (run_backlog_waves /
    schedule_frontier) calls the new precheck before decomposition — a unit-test-only precheck
    is wire-dark (D-030). certified-DONE still requires the live checkers to have said WIRED;
    "the model says its done" is never evidence.
1.2 M3 WRITE-SET BACKFILL AT IMPORT
    backlog import populates allowed_write_set from each plan's WIRING_CONTRACT produces/Files
    touched blocks (the empty-set = deny-all bug currently auto-fails ~40 ELAI tasks and blocks
    64 J plans; conductor-j-burn.md backlog.py patch 3b LABELED_PATH_RE already solved the same
    parse for the codex conductor — reuse it). fallback for a plan with no parsable set: bounded
    default ['.elai_cc/**'] (the integrator's .elai_cc scope guard + frozen-path denies still
    bound it) with an import-time WARN row, never silent.
1.3 M4 COMMIT PATH AUDIT
    verify teikoku's integrator does NOT have the standalone engine's dirt-baseline silent-pile
    bug class (a failed attempt's work absorbed into the next attempt's baseline; ~10h of lost
    commits there). keep the .elai_cc scope guard + maintainer-voice commit_message. push cadence
    RECONCILIATION, stated once: on the autonomous walk, teikoku pushes per integrated unit
    (allow_push=true, owner-approved 2026-07-04 per teikoku-glm-walk.md USER CHOICES); the
    harness one-commit-per-ticket rule governs interactive sessions, not the walk.
1.4 M7 OPS HARDENING
    (a) walk_start reclaims stale heartbeats on (re)start (P8 — orphaned LEASED slots held
    workers hostage twice); (b) infra classifier: Errno5/IO/worktree-prep failures -> infra
    refund, never NEEDS_REVIEW (P7 — 18 false-negatives were hand-reset after the disk crash).
1.5 M2 TREE MODEL (decision folded: hybrid)
    paid lane keeps per-worktree isolation (parallel greenfield is what it is for) + diskguard
    reaping stays between passes + worker_cap per current config; free lane after 1.1 is mostly
    precheck/certify so it rarely builds at all. vhdx move F: -> D: stays an OPTIONAL owner op
    (P9; F: currently has headroom again).
1.6 IMPORTER REGEX WIDEN
    backlog.py letter regex [B-K] -> [B-N] so L/M/N plans import (conductor-j-burn.md noted the
    gap for L/M; N-plan exists too). J-plan injection-defense files that trip guard.py
    (J-102/J-12/J-31/J-36/J-62) go to a human-review queue row — do NOT weaken guard.py.
WAVE-1 GATE (H-11): off-agent wire checkers over the conductor changeset + one integration smoke:
import a small letter fresh, watch precheck certify an already-shipped plan AND route one genuine
gap to EXECUTE, on the SAFE demo/clone path, before any live walk.

======================================================================
WAVE 2 — LANES + CANVAS PICKER (depends: wave 1; 0.1 for the paid lane)
======================================================================
2.1 LANE CONFIGS
    (a) paid: conductor.elai-opus.toml per D2 — opus-exec[EXECUTE], opus-wire[WIRE,INTEGRATE],
    sonnet-rest[AUDIT,POLISH,+rest], all account_id=claude-elai; retier_elai.py --tier PAID for
    greenfield waves. (b) free: replace the glm-5-2 slots with devin bridge swe-1-6 in
    conductor.elai-walk.toml (cap 3, drop to 2 on rate-limit — the measured devin envelope);
    free lane serves RECOGNIZE/certify + activation-class only (M6), never hard greenfield.
    (c) confirm TEIKOKU_AUTONOMOUS=1 reaches claude-lane workers too (the 4 interactive hooks
    0-edit any worker without it — the exact pre-fix glm failure).
2.2 CANVAS PROVIDER PICKER (D5; also closes the W6 "per-run pool pin" follow-on)
    backend: walk_start()/POST /walk/start (+ /headless/run) accept a provider param
    (claude-elai-max | devin-swe-1-6) that pins the run's slot set (role affinity + retier under
    the hood); /orchestration walk field reports the active provider. FE: a compact select
    beside the walk Start/Stop button in CanvasMonitor.svelte + api/orchestration.ts.
    PRODUCTION_HOOK: walk_control.py::walk_start -> slice_api POST /walk/start -> the canvas
    button — the same proven seam the Start/Stop button shipped through. SIGNAL design law
    applies (tokens only, square marks, never red).
WAVE-2 GATE: live smoke — start a 1-task walk from the canvas on EACH provider and watch the
lease land on the right slot class; svelte-check 0/0 + build green; off-agent wire checkers.

======================================================================
WAVE 3 — CODEX TEST/BENCH PROVIDER INSIDE ELAI (depends: wave 1; parallel-safe with wave 2)
front-loaded: credits die 2026-07-15. sources: conductor-j-burn.md workstream 2.
======================================================================
3.1 PROVIDER WIRING: bmapi gpt-5.5 proxy as an elai provider through the existing
    crates/providers openai_compat seam (conductor-j-burn.md names openai_compat.rs +
    trust_dialog.rs as the seams — executor re-greps per W-7). context window sourced LIVE from
    the proxy /v1/models (not in any codex config; anti-stale rule). key/base_url via env, never
    committed.
3.2 E2E SMOKE LANE — BLOCKING. per integrated unit the auditor-owned smoke drives the REAL elai
    entrypoint (elai.bat / cli) on a hard input with the codex provider and asserts the off-agent
    artifact (SSB row / event) landed. missing evidence = FAIL (R1). POST-EXPIRY FALLBACK: after
    07-15 the same smoke runs on sonnet via the Max account — the gate outlives the credits.
3.3 BENCHMARK LANE — ADVISORY. reuse workstream-2's Terminal-Bench adapter plan (BaseAgent
    subclass driving elai headless, OPENAI_BASE_URL at the proxy): (a) day-0 baseline BEFORE the
    walk starts (tb run, small n first to calibrate credits/task), (b) re-run per PART-2 wave,
    (c) deltas to a dated ledger row (workflow/reasoning/finish-bench-ledger.md) + the canvas
    cost chip if cheap. reference: codex-cli+gpt-5.5 = 83.4% TB 2.1. benchmarks never block the
    queue; spend-down aims to exhaust credits by 07-14.
3.4 CODEX-VM CONDUCTOR DISPOSITION (committed): RETIRED as an implementer (its J-burn workstream
    1 is superseded by D1/D3). the codex-vm distro STAYS as the benchmark runtime host (codex
    cli + auth + profiles live there; refresh /root/elai-clone before any use). its backlog.py
    write-set parser is harvested by 1.2.

======================================================================
WAVE 4+ — THE ELAI BACKLOG WALK (PART 2; depends: waves 1-3 gates green)
run THROUGH teikoku (monitored on the canvas), provider per D2/D4, e2e+bench per D3.
======================================================================
ORDERING RULES
  (a) queue entry: a unit enters only when its FULL transitive dep closure holds no terminal
      non-DONE task (the fixed _dep_blocked_ready_task predicate — never the shallow one-level
      check that wedged the walk).
  (b) precedence within eligible units (proof-spine PSR-1): kill-gates > N-1 substrate > PS
      owner units (G-6, K-4, L-10, K-2, K-18 + PS-6 ledger) > declared transitive feeders >
      everything else; ties break cheaper-first.
  (c) class routing: RECOGNIZE/certify + activation/polish/doc -> free swe-1-6 or sonnet;
      genuine greenfield/BUILD + cross-crate wiring -> the opus slots (D2).
  (d) GPU/RunPod-gated units (G-6 SAFEDELTA, E-32 flags) are parked BLOCKED-ON-HARDWARE, never
      silently retried; they enter when a RunPod session is booked.
SEED (consumed once, then D6 supersession takes over)
  4.1 P0 punch items 1-7 (_MASTER-SYNTHESIS): I-33 governed_prompt_decision zero-callers wire-in;
      I-23 facade bypass import + widened contract test; C-15 entropy-gate regression at
      task_runner.rs:15303; H-27/H-28/H-29 capstone gates actually intercepting their loops;
      H-10 PagerankPanel mount; B-4 stale doc-comment. (dirt-baseline punch item 4 is closed by
      1.3 verify-or-fix; the standalone engine itself is retired after wave 1.)
  4.2 verification debt (items 17-21): D-plan 11 DARK re-verify + runtime SSB legs; NAME the
      lost E-DARK-6/F-DARK-3 plan lists; H-30..54 spot-check 3-5 plans; E-51 BLOCK-log claim.
      sonnet auditors; no code unless a re-verify fails.
  4.3 build-or-defer set (items 8-15), opus greenfield unless noted: I-6 anchor pane (S,
      cheapest, first); I-16 kv-cache fixture harness; I-25 ledger-trained router; H-4 tool
      search catalog; E-53 formal-verifier stubs; H-26 metamorphic relations (confirm-or-build);
      H-16 canvasAsDefault flip + soak then orbit retirement; I-28/29/30 only after their WAVE-5
      deps go DONE (rule (a) handles it).
  4.4 open R9 flag rows (item 16) not GPU-gated: E-25 smoke files; E-4/E-8/E-11 enter when
      WAVE-B deps resolve.
  4.5 doc/citation drift (items 22-23) in ONE sonnet batch pass.
  4.6 teikoku W7 (canvas drill-in + steer verbs + /events/wait push) and W9 leftovers — owner-
      eyes waves, interleaved when the owner is around; they never block the ELAI queue.
  4.7 gate-redesign leftovers (teikoku-gate-redesign.md): additionalContext-on-print live verify;
      dashboard "approve commit" button (the D4 sentinel producer, currently manual touch);
      distro git cred helper so teikoku pushes stop deferring.
PER-WAVE LOOP (the orchestrating session, every PART-2 wave)
  refresh open set (D6: elai_status_query --punch + bn_status_report + live checkers) ->
  build the wave from eligible units (rules a-c) -> dispatch through the conductor -> per-unit:
  frozen gates + slop gate + e2e smoke (3.2) -> integrate+push per 1.3 cadence -> wave end:
  H-11 integration gate (off-agent wire checkers over the wave's changesets + one hard-input
  cross-seam smoke) + advisory bench run (3.3) -> next wave. NEEDS_REVIEW piles are triaged by
  the orchestrator (re-route to opus / park / escalate), never left to starve dependents.

STOP CONDITIONS / TRIPWIRES (inherited, restated once)
irreversible ops (destructive git, rm -rf outside scratch, deploys) = owner Y/N; walk STOP =
touch conductor/ELAI_WALK_STOP or the canvas Stop; disk: diskguard stays wired, watch F: free;
memory: .wslconfig 24GB cap + autoMemoryReclaim=gradual stays; 3 consecutive infra-halt events
= INFRA_GUARD_HALT already pauses scheduling; hourly monitor runbook = teikoku-glm-walk.md.

DONE DEFINITION (the plan, not a unit)
(1) waves 0-3 shipped + gates green; (2) PART-2 queue drained to: zero eligible units, with the
residue EXPLICITLY one of BLOCKED-ON-HARDWARE / user-deferred / registered-dark with allowlist
rows; (3) elai_status_query --punch reports 0 open non-parked items; (4) PS-6 wire-dark rate
re-measured and logged (the PROOF_SPINE payoff); (5) bench ledger has baseline + final rows;
(6) handoff workflow/handoff/teikoku-elai-finish.md carries the closing entry.

HOW TO PICK UP (the orchestrating session's literal first steps)
1. launch on the dedicated account: C:\Users\user\bin\elai-claude.ps1 (or elai-code.ps1).
2. wave 0: confirm Max upgrade + model ids; owner ack on the distro checkpoint commit; codex
   balance check.
3. backend: wsl -d teikoku -- sudo systemctl start teikoku.target; verify :8787 == 200; walk
   stays STOPPED until wave 2 gate passes.
4. execute waves 1-3 per this file (orchestrate-protocol binding; agent-dispatch-preamble
   pointer on every dispatch; auditor off-agent on every changeset).
5. day-0 TB baseline (3.3) while wave 2 runs — credits before 07-15.
6. start PART 2 from the canvas with the provider picker; monitor per the runbook; the money
   metric is INTEGRATION_PUSHED > 0 and climbing.

HONEST RESIDUE
- the ~70 standalone-engine commits were largely recognition of already-shipped work; do not
  read wave-1 precheck as "free models build features" — measure certify-vs-execute commits
  separately so the comparison stays honest.
- opus-on-Max lane is untested-live until 0.1 (Pro could not run opus-1m); first PART-2 wave is
  the acceptance test.
- swe-1-6 capability on activation tasks is assumed from the devin-lane precedent, not measured;
  if its gate-pass rate on activation class is poor, collapse the free lane to certify-only.
- e2e-smoke-on-codex proves the provider path + the feature under one model; it is not a
  multi-provider matrix. post-07-15 the smoke provider changes (3.2) — a delta in results
  across that boundary is expected noise, not regression evidence.
- the 06-12 baseline audit rows are known-noisy (3 confirmed false claims); 4.2 spot-verifies
  before acting on any inherited itemized claim.

End of TEIKOKU-ELAI-FINISH-PLAN.
