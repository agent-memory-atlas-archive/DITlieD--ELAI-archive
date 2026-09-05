# J-80: Worktree pool lifecycle — warm-reuse + persistent + self-healing pool over the I-2 barrier

STATUS: PLANNED (WAVE KAT-1; M-L; EXTENSION(I-2) — adds the POOL-LIFECYCLE half to the BUILT barrier/isolation half)

Objective: I-2 COW WORKSPACE ISOLATION is mapped PLANNED, but the BARRIER/ISOLATION half is already BUILT and contract-tested: crates/orchestrator/src/sandbox/branch_workspace.rs (trait BranchWorkspace:134, WorktreePoolImpl:175 = BTreeMap<branch_id,PathBuf> behind tokio::sync::Mutex, barrier_merge:432 GD-2 SSB-seq-ordered last-seq-wins + ConflictEscalated Tier-1 halt, escape_check:389 delegating to elai_safety::blast_radius::check_fs_target, prune:270 Windows handle-race bounded retry, WORKTREE_SCRATCH_DIR ".wt-scratch" kept outside .elai_cc:53); crates/contracts/tests/i2_cow_workspace_contract.rs; callers in bsp/engine.rs. The POOL-LIFECYCLE half is the genuine gap (treehouse's strength, orthogonal to ELAI's merge): the in-memory BTreeMap dies with the process — no cross-session reuse, no resume, no warm pool. J-80 adds, OVER the SSB journal (GD-2, NEVER an external JSON state file like treehouse's): (1) PERSISTENT pool state as a projection re-derivable from BranchWorkspaceEvent rows; (2) PID-recycle-safe reservation OR an SSB-lease analog (owner liveness survives PID reuse); (3) healState self-heal on acquire/list (drop dead-owner + vanished-path entries, never touch leased); (4) WARM REUSE (reset an idle worktree to default branch, detached HEAD, hand it back). KEEP barrier_merge + escape_check unchanged (stronger than treehouse, which has no merge). Cross-platform MANDATORY (win32 primary). WHY: this structurally removes the 2026-06-11 `git reset --hard x8` data-loss class — orchestrate/auto-orchestrate workers SHARE the repo tree today; a warm per-worker worktree from a persistent pool means each worker writes its own root and merges at the barrier. It is also the substrate J-60 names as its blocker ("per-agent worktree TAB ... worktree diff from the H-49 op-stream UNTIL I-2 COW worktrees ship"). HARD BOUNDARY: do NOT re-implement barrier_merge/escape_check (GD-1 P0); do NOT copy treehouse's separate JSON state file (GD-2 re-derive from the journal); do NOT copy gnhf's `git reset --hard` rollback (use H-49 CAS / RV-1). Source: workflow/reasoning/steal-kunchen-agentic-tools-2026-06-24.md C1.1 + SYNTHESIS TIER-1(1).

Estimated effort: ~1-1.5 sessions (~M-L). In-process pool ops; the cross-process lock leg is OS-level (LockFileEx on win32) but no network.
Pre-requisites (re-grep live per W-7): branch_workspace.rs (WorktreePoolImpl/barrier_merge/escape_check/prune/WORKTREE_SCRATCH_DIR); i2_cow_workspace_contract.rs; bsp/engine.rs (the 3 existing callers); SSB BranchWorkspaceEvent (the journal rows the projection reads); elai_safety::blast_radius::check_fs_target (escape containment); H-49 CAS / RV-1 (the lossless snapshot substitute for reset-hard).
Parallel-safe with: J-81, J-82, K-11, K-12 (disjoint files).
Source: steal-kunchen C1.1 (treehouse pool.go acquire / state.go / lease / healState; docs/.source/treehouse). Doctrine: J-0_INDEX.md (W/GD/HD inherited; ADDS no delta family).

## Context
Why this matters: the worker-isolation gap is the single highest-leverage adoption in the kunchen corpus. treehouse proves the cross-platform persistent-pool pattern (gopsutil v4 process detection + LockFileEx + ownerAlive checking StartedAt to survive PID reuse + Leased XOR OwnerPID==0 invariant); ELAI already has the harder half (deterministic barrier merge ELAI's worktrees treehouse lacks). The ELAI-shaped synthesis keeps the merge, takes the lifecycle, and stores state in the journal not a sidecar JSON. Honest residue (GD-5): the Windows handle-race that prune already concedes (warn-and-leak after 3 retries) is bounded, not eliminated; healState makes it self-correcting on the next acquire rather than fatal.

## Existing state (B/C/D/E/F/G/H/I coverage — W-7 anchors, executor re-greps)
- branch_workspace.rs trait BranchWorkspace:134 / WorktreePoolImpl:175 / barrier_merge:432 / escape_check:389 / prune:270 / WORKTREE_SCRATCH_DIR:53 [BUILT] — the isolation/merge half; map says I-2 PLANNED (STALE — apply MAP FIX I-2→PARTIAL).
- crates/contracts/tests/i2_cow_workspace_contract.rs [BUILT] — the existing contract (extend, don't replace).
- bsp/engine.rs (3 callers) [WIRED] — the production consumers of the pool.
- SSB BranchWorkspaceEvent [WIRED] — the journal rows the persistent-state projection re-derives from (GD-2).
- H-49 CAS blake3+zstd + RV-1 snapshot-before-restore [PARTIAL] — the lossless substitute for gnhf's reset-hard (DO-NOT-COPY exhibit).
- J-60 CANVAS LAYERED OBSERVABILITY [PLANNED] — the downstream consumer (per-agent worktree TAB) blocked on this pool.

## Reads (mandatory before coding)
1. _grounding/J-HARNESS-GROUNDING-MAP.md slice for I-2; workflow/reasoning/steal-kunchen-agentic-tools-2026-06-24.md C1.1 + the treehouse interface map workflow/papers/treehouse-interface.md + docs/.source/treehouse (pool.go/state.go).
2. branch_workspace.rs (the full module); i2_cow_workspace_contract.rs; bsp/engine.rs caller sites.
3. F-0_INDEX W-1..W-10; G-0 GD-1/GD-2; CLAUDE.md (blake3-only, no .unwrap on prod, deadpool/bb8 not r2d2, BTreeMap not std HashMap, worker-git-read-only); H-49/RV-1.

## Tasks (WAVE KAT-1 — add lifecycle to the barrier pool)

SCOPE (2026-08-07 ratified refinement, see WARGAME block below): EXECUTE slices 1-4 only this pass (warm reuse + heal + reset + wire the phase loop). Tasks 1 and 2 (the cross-process durable-lease leg) are DEFERRED to a named-consumer unit (J-60) — no production caller for cross-process persistence today (R-8/R10: shipping them now lands a dark producer). Do NOT ship Tasks 1-2 this pass.

1. DEFERRED (no production caller; J-60 is the named future consumer): Journal-backed pool-state projection (~70 LOC): a PoolState re-derived from BranchWorkspaceEvent rows (acquire/release/lease/heal), NEVER a separate JSON file (GD-2). Single-writer through the SSB; reads via the r2d2 pool.
2. DEFERRED (no production caller; PID-recycle-safe durable reservation needs the Task 1 projection): PID-recycle-safe reservation / SSB-lease (~60 LOC): owner identity = {pid, started_at} (gopsutil-equivalent liveness on win32; std on unix) OR a durable SSB lease; ownerAlive() = recorded started_at matches the live process; Leased XOR ephemeral-owner invariant.
3. healState on acquire/list (~40 LOC): drop dead-owner + vanished-path entries; NEVER touch leased; idempotent.
4. Warm reset-and-reuse (~50 LOC): reset an idle worktree to default branch (detached HEAD, fetch-on-acquire), hand it back instead of spawn-per-task. Cross-platform; bound the win32 handle-race (extend prune's retry + heal).
5. Wire orchestrate/auto-orchestrate worker dispatch to acquire-from-pool (~30 LOC): each worker gets its own worktree root; merge at the existing barrier_merge. The vertical slice's CONSUMER.
6. workflow/reasoning/J-80-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/sandbox/branch_workspace.rs — PoolState projection + lease/reservation + healState + warm reset-and-reuse (lifecycle layer ON WorktreePoolImpl)
consumes_symbols:
  - branch_workspace.rs::{barrier_merge, escape_check, prune, WORKTREE_SCRATCH_DIR} (KEEP — never re-implement, GD-1)
  - SSB BranchWorkspaceEvent (the state projection source, GD-2); elai_safety::blast_radius::check_fs_target (containment); bsp/engine.rs callers
reachable_from:
  - orchestrate / auto-orchestrate worker dispatch → pool acquire → per-worker worktree (PRODUCTION worker isolation)
  - bsp/engine.rs barrier_merge path (the existing production consumer)
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO external JSON state file (re-derive from the SSB journal — GD-2)
  - NO re-implementation of barrier_merge / escape_check (GD-1 P0)
  - NO `git reset --hard` rollback (use H-49 CAS / RV-1); NO worker git mutation outside its own worktree
  - NO touching a LEASED entry in healState; NO .unwrap()/.expect() on prod; NO std HashMap in the projection (BTreeMap/IndexMap)
frontend_pair:
  N/A with justification — the pool is a runtime substrate; its observable surface is BranchWorkspaceEvent rows the existing panels/J-60 read. (J-60 is the future per-agent-worktree TAB consumer; not this session's pair.) Justified.
observable_effect:
  trigger: "acquire→reuse→release a worktree across TWO processes; kill a process mid-lease; integration smoke (R10): two orchestrate workers run in separate worktrees and merge at the barrier"
  expect_ssb: "BranchWorkspaceEvent rows reused (+Kind Leased / Reused / Healed) OR +1 from LIVE ~267/577 (re-grep, W-9); the pool state re-derives from those rows with NO external file"
  expect_artifact: "a dead-owner entry self-heals on the next acquire; a warm worktree is reset-and-reused (not re-spawned); NEGATIVE arm: two workers writing the same path MUST NOT occur (each gets its own root) — and a missing barrier merge MUST escalate, never silently last-write-win"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-80_worktree_pool_lifecycle.md --run-trigger
rg "healState|fn heal|reset.*default|lease|ownerAlive|started_at" .elai_cc/crates/orchestrator/src/sandbox/branch_workspace.rs  # lifecycle present
rg -ci "state\.json|serde_json::to_writer.*pool|external.*json" .elai_cc/crates/orchestrator/src/sandbox/branch_workspace.rs   # MUST be 0 (no external state file, GD-2)
rg -c "reset --hard" .elai_cc/crates/orchestrator/src/sandbox/branch_workspace.rs   # MUST be 0
test -f workflow/reasoning/J-80-decisions.md
```

## ACCEPTANCE
- [ ] GD-2: pool state is a projection over BranchWorkspaceEvent rows; no external JSON file exists.
- [ ] GD-1: barrier_merge/escape_check CONSUMED unchanged, not re-implemented.
- [ ] worker isolation: an orchestrate worker acquires its own worktree; the reset-hard data-loss class is structurally removed.
- [ ] cross-platform: the lease + healState + warm-reuse pass on win32 (LockFileEx path) and unix.
- [ ] W-2: lifecycle ships default-ON behind ELAI_WORKTREE_POOL_LIFECYCLE_DISABLE (kill-switch).
- [ ] cargo check --workspace GREEN; auditor PASS on -p orchestrator. No new dependency (gopsutil-equivalent via existing sysinfo/std, anti-slopsquat-verify if any).

## WIRING DONE-GATE (clause by clause)
- W-1: done = the two-process acquire/reuse/heal smoke + the two-worker barrier merge run-trigger (not cargo-green).
- W-2: kill-switch ELAI_WORKTREE_POOL_LIFECYCLE_DISABLE.
- W-3: extends BUILT branch_workspace.rs, not a summary-only port of treehouse.
- W-4: frontend_pair N/A (J-60 is the future consumer); justified.
- W-5/W-10: in-process + OS-lock; no network leg.
- W-6: N/A (no dashboard surface this session); justified.
- W-7: every anchor re-grepped at edit; MAP FIX I-2 PLANNED→PARTIAL recorded.
- W-9: SSB prefer-reuse BranchWorkspaceEvent; any bump from the LIVE baseline.
- GD-2: every pool-state field recomputable from the journal under ELAI_SEED.

## Tests to add
- crates/contracts/tests/j80_worktree_pool_lifecycle_contract.rs (~LOC): (a) PoolState re-derives identically from BranchWorkspaceEvent rows with no file; (b) a dead-owner entry is healed on acquire, a leased entry is NOT; (c) ownerAlive rejects a recycled PID (started_at mismatch); (d) warm reuse hands back a reset worktree; (e) two workers acquire distinct roots and barrier_merge still applies last-seq-wins + ConflictEscalated; (f) win32 handle-race path bounded (retry+heal, no panic).

## Files touched
- MODIFY: .elai_cc/crates/orchestrator/src/sandbox/branch_workspace.rs (~220 LOC: lifecycle layer)
- MODIFY: .elai_cc/crates/orchestrator (orchestrate/auto-orchestrate worker dispatch acquire-from-pool wire) (~30 LOC)
- MODIFY: crates/contracts/tests/i2_cow_workspace_contract.rs OR CREATE j80_worktree_pool_lifecycle_contract.rs (~90 LOC)
- CREATE: workflow/reasoning/J-80-decisions.md

## Citations / Grounding
- steal-kunchen-agentic-tools-2026-06-24.md C1.1 (treehouse pool.go acquire:71-165 / state.go:10-27 / lease / healState / warm reset; the 6 lifecycle items ELAI lacks) + SYNTHESIS TIER-1(1) + MAP FIX (I-2 PLANNED→PARTIAL). treehouse interface map workflow/papers/treehouse-interface.md (MEDIUM reliability). treehouse is Go (gopsutil v4 / LockFileEx) — pnpm/Node noted, but this is a Rust reimpl on ELAI's existing module, no treehouse code copied.
- ELAI ground truth: branch_workspace.rs:134/175/432/389/270/53; i2_cow_workspace_contract.rs; bsp/engine.rs (re-grepped at edit, W-7).
- CLAUDE.md: blake3-only, no .unwrap on prod, BTreeMap/IndexMap not std HashMap, worker-git-read-only, no-removal-without-confirmation; the 2026-06-11 reset-hard incident (agent-dispatch-preamble R-1) is the motivating failure.

## Open questions / risks
- Lease vs ephemeral-owner default: durable lease survives zero live processes (needs explicit return); ephemeral self-heals on death. Default ephemeral for orchestrate workers (auto-reclaim), lease only for a user-pinned worktree. Decide at authoring.
- win32 LockFileEx vs an SSB-lease-only design: prefer the SSB lease (one source of truth, GD-2) + OS lock only for the scratch-dir mutate; verify the cross-process race is closed by the journal seq, not the OS lock alone.
- gopsutil-equivalent on win32: use the existing sysinfo dep if present; anti-slopsquat-verify before adding any process-introspection crate.

## Next
J-80 makes worker isolation real (kills the reset-hard data-loss class) and unblocks J-60's per-agent worktree TAB. Pairs with auto-orchestrate (the dispatch consumer) and H-49 (the lossless snapshot that replaces reset-hard). After J-80, orchestrate fleets write to isolated roots and merge deterministically at the barrier.

## WARGAME (2026-08-07 ratified refinement — EXECUTE slices 1-4 only; Tasks 1-2 DEFERRED)

Seam-design proposal: conductor/opus_walk/evidence/J-80/seam-design-proposal.md (orchestrator-ratified). The recon there found a real bug (spawn is write-only) and three doc-drift items; both forks resolve to shipped primitives. This block is the refine-before-execute pass per plan-standard.md.

PRODUCTION_HOOK: crates/orchestrator/src/sandbox/branch_workspace.rs:327 (WorktreePoolImpl::spawn) — make it read the pool BTreeMap for an existing idle entry before creating a fresh worktree (fixes the write-only bug). Same-changeset consumer: crates/orchestrator/src/task_runner.rs:5676 (pool.spawn in the per-node phase loop; warm reuse across phases within one task drive).

EXPECTED_OBSERVATION: spawn, given a branch_id whose worktree already exists in the pool BTreeMap (idle, detached HEAD), returns that existing BranchRoot WITHOUT running `git worktree add` (no fresh worktree created). On a fresh branch_id it creates as today. The phase loop at task_runner.rs:5676 acquires from the pool per node; across two phases reusing the same node id, the second acquire reuses the first's worktree (one `git worktree add` for the lifetime, not one per phase).
ON_FAIL: spawn returns Err(BranchWorkspaceError::GitCommand(..)) — CAUSE: the existing worktree path vanished on disk between insert and reuse (a prune or external rm). COUNTER-MOVE: healState (slice 2) drops the vanished-path entry on the next acquire, so the failed reuse falls through to a fresh create; the contract test asserts a vanished-path entry is dropped, never panics.

EXPECTED_OBSERVATION: healState, run on acquire, drops entries whose worktree root no longer exists on disk AND entries whose foreign owner PID is not alive (per the held fd-lock release), and NEVER touches a leased entry. Idempotent across runs.
ON_FAIL: healState panics or touches a leased entry — CAUSE: the liveness check mis-scoped to in-process owners (Windows LockFileEx is per-handle within one process). COUNTER-MOVE: liveness is checked only for FOREIGN owners (different PID); in-process exclusion stays the pool's own tokio::sync::Mutex. The contract test asserts a leased entry survives healState.

EXPECTED_OBSERVATION: warm reset-and-reuse resets an idle worktree to the default branch (detached HEAD) before hand-back, cross-platform. The win32 handle-race in prune is bounded (warn-and-leak after 3 retries), not eliminated; healState makes it self-correcting on the next acquire.
ON_FAIL: reset fails on win32 with a handle-race (file locked by a concurrent prune) — CAUSE: the existing prune retry bound (3) was exceeded. COUNTER-MOVE: extend prune's existing retry count for the reset path and warn-and-leak (the plan's GD-5 honest residue); the next acquire's healState reclaims it. Never panic.

EXPECTED_OBSERVATION: the phase loop at task_runner.rs:5676 acquires from the pool instead of spawning fresh per node. cargo check -p orchestrator --all-targets GREEN; the j80 contract test (warm-reuse + heal assertions) passes; production_caller_check.py over the changeset reports the new spawn-reuse path WIRED (the task_runner.rs:5676 caller is the production caller).
ON_FAIL: production_caller_check reports DARK — CAUSE: the new reuse helper has no production caller (the phase loop was not rewired). COUNTER-MOVE: re-dispatch the fixer to wire task_runner.rs:5676 to call the reuse path (the PRODUCTION_HOOK is the same-changeset consumer; a producer-only ship is REFUSED per D-030).

FORK: if the existing worktree's HEAD is NOT detached / NOT on the default branch (a prior phase left it mid-merge), TRIGGER: do NOT reuse it as-is. Route: reset-and-reuse (slice 3) resets it to default branch detached HEAD before hand-back, so a dirty worktree is never silently handed to a new phase. If reset itself fails (locked files), fall through to prune + fresh create (never hand back a worktree in an unknown state).
FORK: if the pool BTreeMap has an entry for branch_id but the path exists and is currently LEASED (foreign owner alive), TRIGGER: do NOT reuse it; create a fresh worktree for the new requestor (two workers must never share a root). The fd-lock + PID check is the mutual-exclusion primitive (FORK B verdict: OS lock for exclusion, journal for state).

RECON_NEEDED: verify fd-lock release-on-process-exit semantics on the target platform before relying on it as the liveness oracle. Settling check: `grep -n "fd_lock\|RwLock\|pid_alive" crates/orchestrator/src/sandbox/lock.rs` (already a direct dep, Cargo.toml fd-lock="4") + a contract test that acquires, kills the owner, and confirms the lock releases. The proposal verified lock.rs:50 fd_lock::RwLock + pid_alive at :13/:39; re-grep at edit time (W-7).
RECON_NEEDED: verify the phase loop at task_runner.rs:5676 is the only production caller of pool.spawn (blast radius for the wire). Settling check: `grep -rn "pool.spawn\|\.spawn(" crates/orchestrator/src/task_runner.rs` + codegraph callers of WorktreePoolImpl::spawn. The proposal found task_runner.rs:1286-1287 (agent_pool spawn) and 5671/5793 (the phase loop); confirm no other caller needs the reuse path.
RECON_NEEDED: doc-drift #1 — the plan body and SUMMARY_INDEX place the three pool callers in bsp/engine.rs; they are ACTUALLY in task_runner.rs. Settling check: `grep -rn "WorktreePoolImpl\|pool.spawn" crates/orchestrator/src/bsp/` (expect zero hits) vs `crates/orchestrator/src/task_runner.rs` (expect the real callers). Correct the plan's Reads at execute time, do not chase bsp/engine.rs.
RECON_NEEDED: doc-drift #2 — ACCEPTANCE cites a non-existent sysinfo dep; the design uses NO sysinfo (fd-lock is the liveness oracle, no new dep). Settling check: `grep -n "sysinfo" crates/orchestrator/Cargo.toml` (expect none) + confirm fd-lock is already a direct dep. Remove the sysinfo reference from ACCEPTANCE; anti-slopsquatting satisfied (no new dependency).
RECON_NEEDED: doc-drift #3 — H-49 is flagged [PARTIAL] but the CAS/RV-1 legs J-80 needs are SHIPPED with a production caller. Settling check: `grep -n "CasRestoreEngine\|load_snapshot" crates/cli/src/timetravel_cmd.rs` (expect :236/:138/:187). Update the pre-req status to BUILT for J-80's purposes.

ABORT: if cargo check -p orchestrator fails to compile after slice 1 (the spawn-reuse edit), STOP — the BTreeMap read path is structurally wrong, do not pile slices 2-4 on a broken slice 1. Append to the J-80 handoff and return BLOCKED with the compile error.
ABORT: if the contract test for warm-reuse cannot be written because no seam exists to inject a pre-populated idle pool entry (the pool is Arc<Mutex<BTreeMap>> private), STOP — that is itself the finding (the architecture prevents the test from locking down the reuse). Report it as the diagnosis + the smallest seam that would fix it (a test-only constructor or a healState-on-acquire hook), do not bolt on a brittle test asserting against internals.
ABORT: if production_caller_check.py reports DARK on the new reuse helper after the phase-loop wire, STOP — re-dispatch the fixer once with task_runner.rs added to the write set (the WIRING file of the same crate); do not repeat the identical write set after a scope BLOCK (owner directive: widen, do not repeat).

VERIFY (off-agent artifacts, R4):
- `CARGO_TARGET_DIR=$HOME/.cache/cargo-target-elai cargo check -p orchestrator --all-targets` GREEN (compile).
- `CARGO_TARGET_DIR=$HOME/.cache/cargo-target-elai cargo nextest run -p orchestrator j80_worktree_pool_lifecycle` GREEN (the contract test: warm-reuse reuses an idle entry; heal drops a dead-owner + vanished-path entry; never touches a leased entry; two workers acquire distinct roots).
- `python3 .claude/tools/production_caller_check.py --files crates/orchestrator/src/sandbox/branch_workspace.rs,crates/orchestrator/src/task_runner.rs` reports the new reuse path WIRED (task_runner.rs:5676 is the production caller), not DARK.
- `python3 .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-80_worktree_pool_lifecycle.md --run-trigger` PASS.
- Falsification (owner directive R1-R9): delete the spawn-reuse read (revert slice 1), the contract test's warm-reuse assertion goes RED; restore, it goes GREEN. Report both. The test must call the SAME fn production calls (pool.spawn via the phase loop), not a re-implemented branch in the test body.

REDTEAM:
- Attack 1 (OS-lock-as-liveness-oracle double-acquire): the same process can acquire the fd-lock twice (Windows LockFileEx is per-handle within one process). FAILED against this plan: liveness is checked only for FOREIGN owners (different PID); in-process exclusion comes from the pool's own tokio::sync::Mutex, cross-process from the fd-lock. The two layers compose correctly. PATCH: the healState foreign-PID-only check.
- Attack 2 (journal-only-lease eliminates the OS lock): FAILED against this plan — write_payload (ssb.rs:438) is an unconditional append with no UNIQUE / ON CONFLICT constraint on event rows, so two processes can both read the journal, see the same entry as free, and both write a Leased row (TOCTOU at the write level). PATCH: keep the OS lock for mutual exclusion, the journal for state. This is why the durable SSB-lease leg (Task 2) is DEFERRED, not shipped journal-only.
- Attack 3 (ship the durable cross-process lease now): FAILED against this plan — R-8/R10: no production caller (J-60 PLANNED, the pool is single-process per phase loop today). PATCH: defer to a named-consumer unit; ship only the warm-reuse slices (1-4) that have a same-changeset consumer (task_runner.rs:5676).
- Attack 4 (spawn reuse hands back a worktree mid-merge): FAILED — the reset-and-reuse fork above resets to default branch detached HEAD before hand-back; a dirty worktree is never silently reused.

INTEGRATION_SMOKE: after slices 1-4 land, drive the REAL entrypoint on a HARD input — an orchestrate/auto-orchestrate worker dispatch that runs TWO phases reusing the same node id, asserting (off-agent) that only ONE `git worktree add` fires across the two phases (the warm-reuse win) and that the barrier_merge still applies last-seq-wins + ConflictEscalated. The SSB BranchWorkspaceEvent rows are the off-agent artifact (a Reused/Healed kind, or a Spawned count of 1 not 2). Toy single-phase happy-path does not satisfy R10. Per-unit contract green is necessary, not sufficient.

DEFERRED (separate unit, named consumer J-60): Task 1 journal-backed PoolState projection + Task 2 PID-recycle-safe durable reservation / SSB-lease. These are cross-process persistence; no consumer today. The ephemeral + fd-lock design above is the single-process substrate the deferred leg will project into the journal later. Do NOT ship Tasks 1-2 this pass.

End of J-80.
