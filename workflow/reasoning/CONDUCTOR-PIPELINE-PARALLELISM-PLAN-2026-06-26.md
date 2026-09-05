CONDUCTOR PIPELINE + PARALLELISM PLAN (2026-06-26)

PROBLEM (grounded, measured live 2026-06-26)
The conductor runs 4 configured glm-5.2 workers (worker_cap=4, free_model_concurrency_cap=4,
saturate_workers=true) but sustains only ~2 concurrent (peak 2 / avg 1.89 over 2h, _live_status:
leases=2 busy_slots=2 with 368 READY). Two root causes, both write-scope granularity:
  C1 CRATE-LEVEL LOCK ON A HOT CRATE. leases.acquire_ready (leases.py:144-146) skips any
     candidate whose write-set overlaps an active lease; scopes_overlap treats crates/orchestrator
     as one atomic lock. 190 of 220 ready IMPLEMENT tasks (86%) lock crates/orchestrator. One
     orchestrator lease therefore blocks the other 189. The IMPLEMENT wave serializes to ~1 task.
  C2 FILLER SELF-COLLISION. The saturate-workers STATE_CHECK filler was meant to backfill spare
     slots, but state_check_seed.py:100 sets allowed_write_set=[REPORT_REL] (a shared constant), so
     all 34 ready STATE_CHECK tasks mutually overlap -> only 1 filler ever leases. That is slot 2.
     Slots 3-4 have nothing disjoint left.
The Devin free-model ceiling is 4 (tested, user-confirmed 2026-06-26); the handoff's repeated "~2"
is STALE. So 2 idle slots are wasted capacity, not a safety floor.

SECONDARY PROBLEM (the idle slots should do TRUST work, not just throughput)
CONDUCTOR-TRUST-PLAN-2026-06-24.md already diagnosed the gate as static-only (Layer 1/2 + cargo),
with NO Layer 3/4 runtime backstop: nothing proves an integrated commit actually RUNS the shipped
code from a real entrypoint (wiring-enforcement R5/R10). Its P3 (runtime integration smoke) and P4
(turn the read-only filler into an independent wiring auditor that auto-opens fix tasks) were parked
as a once-per-epoch controller step and explicitly froze worker-saturation as "utilization theater."
The reframe that dissolves that objection: if the spare capacity DOES the runtime/e2e verification,
saturation and the trust objective become the same work. The idle slots become the missing
Layer-3/4 auditor instead of a proxy metric.

DESIGN ORIGIN
The orchestrate / auto-orchestrate skills are a per-plan worker pipeline: context-compiler + 
integration-explorer (ground/seam) -> executor (implement+wire) -> auditor (independent verify, the
only test runner) -> memory-writer, with a handoff-file mailbox for blocked workers, parallel cap 3,
and "the verifier is never the implementer" (wiring-enforcement R8). The conductor is the same shape
collapsed to one worker per task with a role label and a wave sequence. This plan enriches the
conductor's per-plan pipeline to match orchestrate (adds the pre-execution EXPLORE stage and the
post-implementation VERIFY stage) and fixes the scope model so the pipeline can actually run wide.
The cross-session coordination template is local-rag's mcp_session_claim (server.py:1083): file-level
ownership + auto-computed blast radius + conflict detection + a message bus, NOT crate-level mutual
exclusion. Two sessions share a crate as long as their (write-set + blast-radius) are disjoint.

USER DECISIONS (2026-06-26, locked)
D1 MERGE STRATEGY = parallelize work, serialize merge. File-level + blast-radius locks let N workers
   run the long devin-implementation on disjoint files in the same crate concurrently; a per-crate
   integration MUTEX serializes ONLY the cheap merge step. Keep the integrator ff-only (the invariant
   that protects against the dirty-tree/data-loss class); change it to rebase-then-ff under the mutex.
D2 PIPELINE SCOPE = full orchestrate-style pipeline. Add the pre-EXECUTE EXPLORE/ground stage in
   addition to A (VERIFY) and B (locks).
D3 HOW = plan-first. This changes the lease/scope SAFETY invariant and the live run. Author + test on
   a throwaway elai-repo branch, prove the wire-dark-catch and no-merge-corruption, then deploy. The
   conductor keeps running on current code meanwhile (new conductor/*.py auto-loads on the next pass).

TARGET PIPELINE (per plan, chained by task_deps, each stage = a purposed Devin prompt)
  EXPLORE (read-only, ground + seam + emit precise scope)
    -> EXECUTE -> WIRE
    -> INTEGRATE (per-crate mutex, rebase-then-ff)
    -> VERIFY (read-only, drive real entrypoint / e2e, assert off-agent artifact)
    -> AUDIT -> POLISH
EXPLORE and VERIFY are read-only with per-plan report write-sets, so they NEVER contend on crate
locks and always have ready work -> they fill the idle slots from BOTH ends of the pipeline.

HARD CONSTRAINT / HONEST LIMITATION
Devin CLI workers do NOT run the local-rag pre-tool lock hook, so a worker's individual writes are
not blocked at write-time the way a Claude Code session's are. Enforcement lives at TWO conductor
seams instead: (a) LEASE time -- the conductor refuses to lease work whose scope overlaps a live
lease; (b) INTEGRATE time -- rebase-then-ff under the per-crate mutex fails CLOSED (NEEDS_REVIEW) on
any conflict, catching a blast-radius miss. The awareness line in the worker prompt is advisory only.
This is the sound, fail-closed posture given the worker harness; it is weaker than local-rag's
write-time hook and that is acknowledged, not hidden.

----------------------------------------------------------------------------------------------------
PHASE 0 -- FILLER SCOPE FIX (immediate idle-slot relief; independently shippable; lowest risk)
Fixes C2. state_check_seed.py: give each STATE_CHECK task a UNIQUE per-plan report write-set
(e.g. WIRING_STATE_<plan_id>.md) used consistently in (i) the worker prompt instruction, (ii)
allowed_write_set, and (iii) the persistence/validation path (state_check_report_rel + _finish_state_check,
conductor_loop.py:140-205). Today the prompt says the constant REPORT_REL but persistence reads
state_check_report_rel(task) -- reconcile them onto the per-plan path.
ACCEPTANCE: with >=3 spare slots and >=3 ready STATE_CHECK plans, a dry-run wave leases 3 distinct
STATE_CHECK tasks concurrently (currently caps at 1). Regression unit test asserts 3 disjoint
STATE_CHECK write-sets do not mutually overlap.
RISK: low. Report path change must keep _finish_state_check reading the same path the prompt writes.
NOTE: this alone restores ~4-slot saturation using the EXISTING static filler, buying time while the
richer stages land. Ship + deploy this first.

PHASE 1 -- FILE-LEVEL + BLAST-RADIUS LOCKS (B1; fixes C1)
leases.acquire_ready scope model moves from crate-level to file/module-level write-sets plus a
computed blast radius. Conflict = overlap of (write_set UNION blast_radius) between two active leases.
  - Keep the conservative rule: an empty/unknown write-set overlaps everything (scopes_overlap
    left/right-empty -> True). Safe default; finer scope is an optimization layered on top.
  - BLAST RADIUS source = the Rust/multi-lang graph, NOT the Python import-grep in
    session.compute_blast_radius. Use .claude/tools/impact_radius.py (elai.map import+call edges) or
    codegraph_impact. The blast radius MUST include the crate's module-declaration surfaces
    (src/lib.rs, the relevant mod.rs) and Cargo.toml WHEN the change adds a module or dependency --
    those are genuine shared-edit conflict surfaces; missing them is how the rebase in Phase 2 would
    later conflict.
  - GRACEFUL DEGRADATION: a task with only crate-level scope (no EXPLORE output yet) behaves exactly
    like today (crate lock). As EXPLORE (Phase 3) populates file-level scopes, parallelism rises.
    So Phase 1 is backward-compatible and safe to ship before Phase 3.
ACCEPTANCE: two tasks with disjoint file write-sets in the SAME crate (e.g. two distinct
crates/orchestrator/src/*.rs files, neither touching lib.rs) BOTH lease concurrently in a dry-run;
two tasks whose blast radii overlap (one adds a module -> locks lib.rs; the other edits lib.rs) do
NOT. Unit tests for both. Live check: peak concurrency on the IMPLEMENT wave rises above 2.
RISK: medium. An UNDER-computed blast radius lets two genuinely-conflicting tasks lease -> caught
fail-closed at Phase 2 integrate (rebase conflict -> NEEDS_REVIEW), never corrupting HEAD.

PHASE 2 -- PER-CRATE INTEGRATION MUTEX + REBASE-THEN-FF (B2; the D1 merge strategy)
integrator.py keeps ff-only onto the isolated elai-repo HEAD, but:
  - acquire a per-crate integration mutex (one INTEGRATE per crate at a time; cross-crate stays
    parallel). Implement as a lightweight lock keyed by the task's primary crate (reuse the
    cargo_gate_slot file-lock idiom under state/, or a DB row).
  - because HEAD may have moved since the candidate branched, REBASE the candidate onto current HEAD
    (or cherry-pick its commits) BEFORE the ff. Disjoint file-sets -> clean rebase -> clean ff. Any
    conflict -> NEEDS_REVIEW (reason=rebase_conflict_under_mutex), fail-closed. HEAD never moves on a
    failed integrate, so no committed work is lost (preserves the dirty-checkout self-heal invariant).
ACCEPTANCE: plant two branches that both touch crates/orchestrator on DISJOINT files; both integrate
sequentially under the mutex and both land on HEAD (ff after rebase). Plant two branches that touch
the SAME file; the second fails closed to NEEDS_REVIEW with rebase_conflict_under_mutex, HEAD intact.
Unit + a real two-branch elai-repo test on a throwaway branch.
RISK: medium-high (touches the integration invariant). Mitigation: throwaway-branch proof before
deploy; HEAD-reachability assertion already exists (handoff 06-24 ff_merge_not_durable) -- keep it.

PHASE 3 -- EXPLORE STAGE (D2 pipeline front; feeds Phase 1 the precise scope)
New read-only stage/wave before EXECUTE. A Devin explorer grounds the plan (spec refs +
WIRING_CONTRACT) and EMITS a structured context-package per plan: the precise file-level write-set,
the computed blast radius, and the PRODUCTION_HOOK seam (file+fn) for the executor -- the
context-compiler + integration-explorer analog from orchestrate. Output is a per-plan report file
(unique write-set -> no crate contention -> fills idle slots, read-only). The EXECUTE task then runs
with that file-level scope, which is what makes Phase 1 actually widen the frontier.
  - Report-contract validated by the conductor (like validate_state_check_report): must name
    write_set + blast_radius + production_hook, else NEEDS_REVIEW.
  - DEP: EXECUTE depends on its plan's EXPLORE (task_deps).
ACCEPTANCE: an EXPLORE run on a sample plan produces a valid context-package; the subsequent EXECUTE
task is seeded with the file-level write-set from it; two plans whose EXPLORE outputs are file-disjoint
in the same crate then run EXECUTE concurrently (end-to-end of Phase 1 + 3).
RISK: medium. If EXPLORE's write-set is wrong, Phase 2 rebase catches the conflict fail-closed.

PHASE 4 -- VERIFY RUNTIME STAGE (A; trust-plan P3, run continuously on idle slots)
New read-only VERIFY stage after INTEGRATE, targeting an already-DONE plan (R8-safe: a VERIFY worker
never verifies its own diff). It drives a REAL entrypoint on the isolated elai-repo at HEAD and
asserts an OFF-AGENT artifact the worker did not author:
  - ENTRYPOINT + EXPECTED ARTIFACT come from the plan's WIRING_CONTRACT (trigger / observable_effect /
    produces / reachable_from -- already parsed at backlog_import.py:525). Backend: run the CLI
    subcommand / FSM transition / task_runner path; assert the expected SSB event row / tracing span /
    coverage hit landed in a time window. Missing artifact = FAIL (R1). No contract -> the worker
    finds the entrypoint read-only and reports "no reachable entrypoint" (itself a wire-dark finding).
  - CARGO COST GUARD: VERIFY reuses the binary already built during INTEGRATE (elai-repo target/);
    it does NOT trigger a fresh full build in the common case. Any build it does need goes through the
    existing cargo_gate semaphore (cap 2) like everything else, so VERIFY can never starve real builds.
  - Report-contract validated by the conductor; the assertion target is frozen (from the contract),
    not the worker's judgment -> keeps the gate un-gameable (R8 role boundary).
ACCEPTANCE (the trust-plan P3 acceptance): plant ONE known wire-dark commit (a new pub fn with no
production caller) onto a throwaway elai-repo branch that PASSED every per-task gate; the VERIFY stage
must catch it (no artifact lands when the entrypoint runs). Prove the catch in a test.
RISK: medium. Entrypoint catalog coverage is only as good as the WIRING_CONTRACT; under-specified
contracts under-cover (log what was not exercised -- no silent "verified everything").

PHASE 5 -- DASHBOARD E2E VERIFY SUB-TYPE (the "fake-human dashboard test")
For dashboard-v2 plans, VERIFY drives the project's EXISTING CDP harness
(dashboard-v2/screenshots/cdp_*.mjs + .claude/tools/visual/ax-scaffold.mjs + ui-diff.mjs): launch the
dev server, navigate, assert the view renders and the expected element exists via the AX-tree
scaffold (not VLM counting), optional ui-diff vs a golden. This is the literal e2e "is it really
wired in the UI" check. Heavier (needs a running dashboard) -> separate phase, can lag Phase 4.
ACCEPTANCE: a dashboard plan's VERIFY launches the dev server, the ax-scaffold confirms the plan's
expected component/route is present; a deliberately-unwired component is reported missing.
RISK: medium-high (process orchestration: dev server lifecycle, port pinning, capture determinism per
visual-ui-loop.md). Keep the capture pinned (deviceScaleFactor 1, fromSurface true,
font-render-hinting=none) or the diff is noise.

PHASE 6 -- AUTO-OPEN FIX TASKS (trust-plan P4; closes the loop)
When a VERIFY or STATE_CHECK report flags a DONE plan as wire-dark / artifact-missing, the conductor
seeds a fix task (WAVE_REPASS or a fixer task) with the finding as objective, depending on that plan.
Idempotent (no duplicate open fix task for the same finding). This turns the read-only stages into an
independent auditor that drives real work instead of accumulating reports.
ACCEPTANCE: a VERIFY/STATE_CHECK report naming a wire-dark DONE plan produces exactly one queued fix
task; a second identical report does not duplicate it. Unit test.
RISK: low.

----------------------------------------------------------------------------------------------------
SEQUENCING + SHIP CADENCE
  Phase 0 ship+deploy FIRST (immediate ~4-slot saturation via the existing static filler).
  Phases 1 -> 2 next (the parallelism core; 1 is backward-compatible, 2 makes same-crate parallel
    work durable). Prove on throwaway elai-repo branches before deploy.
  Phase 3 then widens the frontier for real (file-level scope from EXPLORE).
  Phases 4 -> 6 are the trust spine (runtime VERIFY + auto-fix); 5 (dashboard e2e) can trail.
TESTING (per phase, before any deploy): conductor `python -m unittest discover -s tests` green +
new regression tests named in each ACCEPTANCE + a dry-run wave (dry_run_wave_test.py) + for Phases
2/4/5 a real throwaway-branch proof. House rule: scoped, not the full elai-repo suite. Conductor is
gitignored tooling -> no elai_cc commit; record results in the conductor-build.md handoff.
NON-GOALS: do NOT raise parallel/free-model caps beyond the tested 4; do NOT loosen the ff-only HEAD
invariant (Phase 2 keeps it, adds rebase-under-mutex); do NOT let VERIFY verify its own diff (R8); do
NOT let any stage edit the frozen gate (R8 / gate_freeze_check).

GROUNDING
Live DB + code read 2026-06-26 (leases.py:43-152, state_check_seed.py:100, conductor_loop.py:761-919,
config/conductor.toml, _live_status/concurrency_report). local-rag mailbox: server.py:1083-1190,
session.py:159. Trust plan: workflow/reasoning/CONDUCTOR-TRUST-PLAN-2026-06-24.md (P3/P4). Wiring
doctrine: .claude/rules/wiring-enforcement.md (R1/R5/R8/R10), production-caller-gate.md (D-030).
Orchestrate pipeline: .claude/skills/orchestrate/SKILL.md, auto-orchestrate/SKILL.md. CDP harness:
.claude/rules/visual-ui-loop.md. Devin free ceiling = 4 (user-confirmed 2026-06-26).
