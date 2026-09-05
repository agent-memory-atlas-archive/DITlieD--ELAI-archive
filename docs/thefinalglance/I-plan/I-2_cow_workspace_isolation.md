# I-2: CoW workspace isolation — deterministic per-branch scratch with barrier merge

- Track 12 — FAULT-ISOLATION + ENFORCEMENT-PLANE | lever CONCURRENCY | grounding STRONG | nature BUILD | effort M-L | WAVE 2 | depends I-1 | binds ID-2
- Premise: parallel branches executing tool calls in ELAI currently share a mutable working directory; one
  branch silently overwrites another's intermediate state, making wide parallel search (Track 13 I-4)
  unsafe and making merge outcomes non-deterministic. There is no branch_workspace module (grounding: IS-9
  bsp/engine.rs exists but the workspace isolation layer is absent; I-4 depends on this session). The fix
  is structural: every branch/agent gets a private workspace root; merges happen only at an explicit
  deterministic barrier ordered by SSB seq; an escape (absolute-path write outside the root) is a BLOCK.

## Steal/build
Pregel-style BSP branch isolation corrected for cross-platform reality (IC-9): Linux gets overlayfs/btrfs
reflink for near-zero-cost CoW; Windows/macOS get pooled WARM git worktrees with a shared target-dir
(cargo build artifacts shared, source isolated). A BranchWorkspace trait abstracts both; the cloud-sandbox
adapter (E2B-class) is an optional impl behind the same trait, never a hard dependency. Merge happens at
the BSP barrier: branches are ordered by SSB seq number, conflicts are resolved deterministically (last-seq
wins on non-overlapping hunks; overlap = ConflictEscalated and halts). Same-seed re-run must produce a
byte-identical merged file (GD-2). An escape fixture (a branch writing to an absolute path outside its
root) must be caught and emit EscapeBlocked before the write reaches the FS.
Declined: shared-target-dir without source isolation (still races on source files); process-namespace
isolation without the trait abstraction (platform-specific, not composable with I-1 Supervisor).

## Existing ground truth
- IS-9: bsp/engine.rs recover_from_checkpoint:958 + test:1242 — the BSP engine exists; production
  reachability not verified this pass (executor re-traces R7). The workspace isolation layer is ABSENT.
- IS-17: workspace crate list confirms safety, scaffold, ipc-types present; no branch_workspace crate.
- E-15 Firecracker pool plan: SHIPPED (PLAN-asserted; executor re-greps per W-7) — compose the pool
  lifecycle, do not re-invent process launch.
- C-11 sandbox: SHIPPED (PLAN-asserted; executor re-greps per W-7) — compose the sandbox trait seam.
- C-23 branch UI: SHIPPED (PLAN-asserted; executor re-greps per W-7) — BranchTree surface to badge.
- H-49 RV-1 divergence guard (restore semantics from the undo-tree plan): PLANNED (PLAN-asserted;
  executor re-greps per W-7) — compose the restore semantics for post-merge rollback.
- Phase 3D BSP barriers: SHIPPED aggregator seam (PLAN-asserted; executor confirms bsp/engine.rs barrier
  site at EXECUTING per W-7).
- No branch_workspace.rs module: absence confirmed (IS-17 workspace list + IS-9 framing; no isolation layer
  in the BSP engine).

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- BranchWorkspace trait (spawn, escape_check, barrier_merge) in crates/orchestrator/src/sandbox/branch_workspace.rs
- Pooled warm git-worktree impl (the cross-platform default; shared target-dir)
- Escape check (absolute-path write intercepted at Supervisor gate, EscapeBlocked SSB row)
- Barrier merge (deterministic: SSB-seq order; ConflictEscalated on overlap)
- BranchWorkspaceEvent variants: Spawned, Merged, EscapeBlocked, ConflictEscalated
- Contract test (disjoint roots verified, deterministic re-run byte-identical, escape fixture blocked)

DEFER (Tier-2 / gated):
- Linux overlayfs/btrfs reflink impl: Linux-gated checkbox; same trait, additive impl
- Cloud-sandbox adapter (E2B-class): never a dep; optional impl behind BranchWorkspace trait; user-approval gated
- Cross-branch CRDT merge semantics (beyond last-seq-wins): I-29's scope for diffusion drafts

## Integration
- CREATE: crates/orchestrator/src/sandbox/branch_workspace.rs (~120 LOC: trait + worktree-pool impl + escape check)
- MODIFY: crates/orchestrator/src/bsp/engine.rs — barrier aggregation composes BranchWorkspace::barrier_merge
  (PLAN-asserted seam at the BSP barrier site; executor re-greps per W-7)
- MODIFY: crates/orchestrator/src/task_runner.rs — branch spawn path calls BranchWorkspace::spawn via
  Supervisor (depends I-1 Supervisor as the execution gate)
- CREATE: crates/contracts/tests/i2_cow_workspace_contract.rs (~70 LOC)
- CREATE: workflow/reasoning/I-2-decisions.md
- frontend_pair: BranchWorkspaceEvent rows (Spawned/Merged/EscapeBlocked) consumed by BranchTree badges
  on C-23's existing branch UI surface. cross_lang_diff.py PASS required (W-4).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/sandbox/branch_workspace.rs::BranchWorkspace (trait)
  - crates/orchestrator/src/sandbox/branch_workspace.rs::WorktreePoolImpl (cross-platform default)
  - BranchWorkspaceEvent (new SSB action_type; arms: Spawned/Merged/EscapeBlocked/ConflictEscalated) —
    baseline+1 from LIVE 221/532 as of 2026-06-10 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9)
  - crates/contracts/tests/i2_cow_workspace_contract.rs
consumes_symbols:
  - IS-9 bsp/engine.rs::recover_from_checkpoint (why: barrier must restore to last checkpoint on ConflictEscalated)
  - Phase 3D BSP barrier site in bsp/engine.rs (why: merger replaces the bare aggregation; compose, not re-write)
  - E-15 Firecracker pool (why: compose worktree/process pool lifecycle pattern; do NOT re-invent pool management)
  - C-11 sandbox trait (why: BranchWorkspace is the workspace face of the same isolation contract; compose seam)
  - C-23 BranchTree UI (why: Spawned/Merged events badge the existing branch view; do NOT re-implement the UI)
  - H-49 RV-1 divergence guard / restore semantics (why: ConflictEscalated triggers rollback via H-49 trunk restore)
  - I-1 Supervisor (why: escape_check is a Supervisor gate; workspace spawn is a Supervisor-dispatched Directive)
reachable_from:
  - crates/orchestrator/src/task_runner.rs (parallel branch dispatch) -> Supervisor -> BranchWorkspace::spawn  [PRODUCTION parallel task launch]
  - crates/orchestrator/src/bsp/engine.rs (barrier site) -> BranchWorkspace::barrier_merge  [PRODUCTION BSP barrier]
  (a #[cfg(test)] / contracts/ caller NEVER satisfies this — R7)
stub_ban:
  - NO shared mutable working directory for parallel branches (the invariant this session enforces — ID-2)
  - NO overlayfs assumed cross-platform (Linux-only Tier-2 per IC-9)
  - NO E2B / cloud-sandbox as a hard dependency (optional trait impl only)
  - NO .unwrap()/.expect() on production paths
  - NO blake2/sha2 (blake3 only)
  - NO fabricated file:line numbers or measurements
  - NO re-implementation of BSP barrier logic (compose bsp/engine.rs seam)
  - NO merge order other than SSB seq (non-deterministic merge = BLOCK per GD-2)
frontend_pair:
  BranchWorkspaceEvent rows (Spawned/Merged/EscapeBlocked) in the C-23 BranchTree dashboard surface;
  Tauri read: existing BranchTree IPC handler extended with workspace badge fields.
  cross_lang_diff.py PASS required (W-4).
observable_effect:
  trigger: "elai run --task <parallel-two-branch-fixture> [with seeded conflict]; elai run --task <escape-fixture>"
  expect_ssb: "BranchWorkspaceEvent(Spawned) x2, BranchWorkspaceEvent(Merged) with SSB-seq ordering on clean run;
    EscapeBlocked on the escape fixture; ConflictEscalated on the seeded-conflict fixture followed by trunk rollback;
    same-seed re-run produces byte-identical merged file (GD-2)"
  expect_artifact: "per-branch root prefixes in the SSB journal are disjoint (verified from rows); barrier merge
    resolves the seeded conflict deterministically (same output on identical seed); the negative arm — a branch
    writing an absolute path outside its root — is caught before FS mutation and emits EscapeBlocked; missing row
    or non-deterministic merge output = FAIL (R1/R2)"

## Grounding
- IS-9: bsp/engine.rs:958 recover_from_checkpoint (live grep-confirmed). IS-17: workspace crate list.
  Phase 3D BSP barrier: PLAN-asserted (executor re-greps per W-7). IC-9: cross-platform CoW correction
  (overlayfs Linux-only; worktrees = cross-platform default). ID-2: branch-workspace isolation binding.
- Consolidated draft Part I §workspace-isolation section. Trust-review A3 (enforcement plane context).
  Pregel/BSP isolation as prior art (from the dossier, not independently verified here).
  All arXiv IDs carried from source dossiers: WebSearch-confirm before headline (map citation caveat).
- SSB: BranchWorkspaceEvent +1 variant (4 arms) — baseline+1 from LIVE 221/532 as of 2026-06-10
  (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9).

## COMPOSES-WITH
- IS-9 bsp/engine.rs: existing checkpoint/restore seam. Do NOT re-implement checkpoint logic.
- Phase 3D BSP barrier: existing aggregation seam. Do NOT re-write the barrier; extend it.
- E-15 Firecracker pool: pool lifecycle pattern. Do NOT re-invent process/worktree pool management.
- C-11 sandbox trait: isolation contract seam. Do NOT re-implement the sandbox abstraction.
- C-23 BranchTree UI: branch view surface. Do NOT re-implement the UI; badge it with event rows.
- H-49 RV-1 divergence guard: restore semantics on conflict. Do NOT re-implement rollback.
- I-1 Supervisor: workspace spawn and escape_check are Supervisor-plane Directives. I-2 depends I-1.
- I-4 (evidence-ranked search ladder): depends on I-2 workspace isolation before parallel candidates are safe.

## J-11 AMENDMENT — S-3 WORKTREE LIFECYCLE PRIOR ART (source-cited)
Grounding: Arbor `src/coordinator/tools/executor_run.py:116-210`, `:432` _run_after_executor_hook,
`src/core/git_artifacts.py:54` filter_commit_paths, `src/coordinator/tools/git_ops.py:130` merge-eval pool,
read directly per W-3.
GD-1 prior-art pointer: Arbor's worktree lifecycle is a working existence proof of the branch-isolation
semantics I-2 needs. Five stealable mechanics (compose, do NOT restate — I-2 owns the pool implementation):
1. Branch start-point = CURRENT trunk HEAD, not the original repo HEAD (`executor_run.py:136`
   _create_worktree, `:162` start_ref — picks up prior merges so each candidate builds on the latest trunk).
2. Finalize-commit filters generated artifacts out (`:187` _finalize_worktree, `:194` filter_commit_paths;
   `git_artifacts.py:54` splits commit-worthy from artifact paths via fnmatch on `.coordinator/**`,
   `results/**`, `models/**`, `logs/**`, etc.).
3. Worktree removed but BRANCH PRESERVED for later merge (`:210` _remove_worktree, `:211` "the branch is
   preserved for later merging" — the worktree is ephemeral, the branch is the durable artifact).
4. Merge-eval runs in its own SEPARATE detached worktree pool (`git_ops.py:130`
   `merge-eval-worktrees-{uid}`, distinct from the executor pool at `executor_run.py:147`
   `coordinator-worktrees-{uid}` — eval isolation is a separate worktree, not the executor's).
5. Stale worktrees force-removed at create (`executor_run.py:154` if exists -> `git worktree remove --force`;
   `git_ops.py:138` same); collision => timestamp suffix (`executor_run.py:168-171`).
After-executor snapshot: `executor_run.py:432` _run_after_executor_hook snapshots the submission artifact if
the plugin lifecycle hook is configured — I-2's barrier-merge analog.
NOTE: Arbor has NO worktree pooling and is Unix-only (`os.getuid()` at `:147`/`git_ops.py:130`). ELAI's I-2
warm pooled Windows-safe worktrees (IC-9) are an IMPROVEMENT — keep them. This amendment grounds the
lifecycle semantics; it does NOT change I-2's scope or implementation.

End of I-2.
