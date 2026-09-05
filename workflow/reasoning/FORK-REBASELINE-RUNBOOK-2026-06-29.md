FORK RE-BASELINE RUNBOOK (2026-06-29)

PURPOSE
Make the conductor fork (conductor/elai-repo) baseline equal to current main's .elai_cc tree, so future
worker increments diff against an aligned base and the merge lane can land them clean. This is the deferred
"step 5" of conductor-fork-reconcile.md, never executed. This doc is a runbook the CONTROLLER executes; the
analyst (me) executed nothing destructive. Every claim below is grounded in a command run this session.

This runbook is ANALYSIS ONLY. The controller runs the steps. Read every step before running any.

----------------------------------------------------------------------
1. CURRENT STATE (all measured this session)
----------------------------------------------------------------------

DEVELOPER CHECKOUT (F:\.ELAI_workflow):
- local main HEAD = 7d258a5 ("drop a stray compiled pyc...")
  [git -C F:\.ELAI_workflow rev-parse HEAD]
- local origin/main TRACKING REF = 7d258a5 -- BUT THIS IS STALE.
  [git rev-parse origin/main]
- TRUE remote HEAD (ls-remote) = d66c260a7da4c1f455c34a7b5b68df354ba1473e
  [git ls-remote origin main]  -> the remote advanced 7d258a5..d66c260 since the last fetch.
- d66c260 is NOT in local objects [git cat-file -t d66c260 -> "Not a valid object name"].
- git fetch --dry-run confirms:  7d258a5..d66c260  main  -> origin/main  (6+ commits ahead on the remote).

  *** DECISIVE: the re-baseline SOURCE-OF-TRUTH is the TRUE remote main (d66c260), not the stale
      tracking ref and not the local checkout. The controller MUST fetch first and re-baseline onto the
      actual origin/main HEAD, whatever it is at execution time (re-confirm with `git ls-remote origin main`
      immediately before starting -- it may have moved again since this doc was written). ***

FORK (conductor/elai-repo):
- fork HEAD = 4f22c2836b123582fe7c34b073e77da8ba3a8674 ("drop the per-task duplicate module copies...")
  [git -C conductor/elai-repo rev-parse HEAD]
- fork ROOT = a84e5bb0eb30a276ab259412f426c7576c5bd5cf = "conductor baseline snapshot: .elai_cc @ 9b698b2"
  [git rev-list --max-parents=0 HEAD]  -> fork base == OLD main (9b698b2), long superseded.
- total commits = 134; "Conductor integrate" commits = 128 [git log --grep "Conductor integrate" | count].
- fork has NO git remotes [git remote -v -> empty] -> it can NEVER accidentally push. Only the throwaway
  merge-lane-clone carries the GitHub origin. So RE-BASELINING THE FORK NEVER TOUCHES THE REMOTE.
- fork is gitignored by the parent repo [git check-ignore conductor/elai-repo -> matched]. Good: re-baseline
  edits to it never show in the developer checkout's status.

ROOTING (confirmed, load-bearing):
- fork tree is .elai_cc-ROOTED: fork top-level = crates/, python/, docs/, Cargo.toml, ... (62 tracked
  entries) [git ls-tree --name-only HEAD]. main top-level = .elai_cc/, .gitignore, README.md, a.txt; the
  project lives under main:.elai_cc/crates/..., main:.elai_cc/python/..., etc [git ls-tree --name-only HEAD
  .elai_cc/]. So fork:crates/X == main:.elai_cc/crates/X. A fork patch applies onto main with
  `git apply --directory=.elai_cc`. The re-baseline must PRESERVE this fork-rooted layout (fork stays
  .elai_cc-rooted; it must equal main's .elai_cc/ SUBTREE, not main's whole tree).

MERGE WATERMARK:
- current watermark = 4f22c2836b123582fe7c34b073e77da8ba3a8674 (== fork HEAD)
  [python -c "from conductor_db import ControlDB; print(ControlDB('state/control.db').get_merge_watermark())"]
- Because the watermark IS the slice base in steady-state (merge_slicer.derive_slice_base returns the
  watermark verbatim, merge_slicer.py:129-130), base==fork_head right now -> base..fork_head is EMPTY -> the
  lane has nothing to land until new fork work arrives. The 14 stale COLLISION/QUARANTINE ledger rows are
  from an OLD re-process window (fork_range "dc9ef2ed6fbd..4f22c2836b12", a watermark-reset experiment).

MERGE LEDGER (14 rows, all fork_range dc9ef2ed..4f22c28) [db.merge_ledger_rows()]:
- COLLISION: crate:memory, crate:providers, crate:cli, crate:orchestrator, crate:reasoning, crate:agent,
  dashboard:dashboard-v2, noncrate:python  (8 -- "does not apply clean onto main": the substantive crates
  are already hand-merged onto main in a slightly different form, so whole-crate patches collide).
- QUARANTINED: crate:knowledge (GOODHART-DEP ort), crate:tools (LITTER .md), contracts (LITTER),
  noncrate:skills, noncrate:docs (reviewer exited 1).
- PENDING: noncrate:Cargo.lock ("would land" dry-run, never standalone).

DAEMON / IN-FLIGHT STATE (CRITICAL for safety):
- control_state: paused=false, draining=false, active_wave=INTEGRATION, paused-ts 2026-06-29T02:33:45Z
  [read-only sqlite SELECT * FROM control_state]. THE DAEMON IS NOT PAUSED and was last in an INTEGRATION
  wave. control.db was last written 13:51 (recent) -> a daemon may be live or recently active.
- no driver.pid present right now [cat state/driver.pid -> absent], and `ps` showed no conductor python
  (ps may be unreliable for native Windows processes under this shell -- DO NOT trust "no process" as proof
  the daemon is stopped; the control_state INTEGRATION wave + recent db write say otherwise).
- 14 git worktrees off the fork [git worktree list]. THIRTEEN are at fork HEAD 4f22c28 (idle/clean).
  ONE IS DIFFERENT AND LOCKED: worktrees/J-59-EXECUTE-1_81a1df0a is at 7273547 (="Conductor integrate
  E-28-EXECUTE-1") and marked `locked` -> a worker may be MID-INTEGRATE on it. This is the single most
  dangerous in-flight item; see Safety (4).
- integrator always works off repo_root HEAD: `git rev-parse HEAD` at integrate time, then rebases its one
  commit onto the latest HEAD before the ff (integrator.py:150,166,237-239,287). Worker worktrees are pinned
  at their OWN task.base_commit and rebased onto HEAD at merge time. => a NEW commit advancing fork HEAD is
  EXACTLY how the daemon normally moves; worktrees survive it.

----------------------------------------------------------------------
2. THE RE-BASELINE PROCEDURE (two approaches; recommendation + reasons)
----------------------------------------------------------------------

GOAL RESTATED: the fork's working tree at HEAD should equal main's .elai_cc/ subtree (the project), so the
NEXT worker increment's diff (base..fork_head where base = the re-baseline commit) contains only the new
work and applies clean onto main via `git apply --directory=.elai_cc`.

APPROACH (a) -- NEW-BASELINE-COMMIT ON TOP OF THE EXISTING FORK  *** RECOMMENDED ***
  Snapshot current main's .elai_cc/* into the fork's working tree as a single new commit on top of fork
  HEAD ("baseline snapshot: .elai_cc @ <main-sha>"), preserving the .elai_cc-rooted layout, keeping all 134
  existing commits (incl the 128 integrate commits) reachable as history. Then advance the watermark to that
  new commit.

  WHY RECOMMENDED:
  - PRESERVES THE INTEGRATE-COMMIT HISTORY. trust_audit.integration_commit (trust_audit.py:75-81) finds a
    task's commit via `git log --all -F --grep "Conductor integrate <task>"` over the fork. regate_backlog.py
    and wire_dark_audit.py BOTH reuse it (verified: both `import trust_audit as TA`). An orphan/re-clone
    (approach b) ORPHANS all 128 integrate commits -> every DONE task classifies NO_COMMIT -> the entire
    re-gate / wire-dark-audit toolchain silently goes blind. `git log --all` keeps them reachable even after a
    new HEAD, so approach (a) keeps these tools working.
  - IT IS HOW THE DAEMON ALREADY MOVES. The integrator advances HEAD and rebases onto it (integrator.py:237);
    a baseline commit is just one more HEAD advance. No new failure mode for in-flight worktrees.
  - REVERSIBLE: HEAD is a fast-forward over a single new commit; rollback = `git reset --hard 4f22c28` on a
    paused daemon (or just re-point the branch), and the watermark restore is one DB call. (Controller-only;
    the analyst never runs reset.)
  - NO REMOTE RISK: the fork has no remotes; nothing can leak to GitHub.

  CAVEAT of (a): the new baseline commit's diff (4f22c28 -> baseline) will be LARGE and messy (it overwrites
  the fork's divergent crate trees with main's versions). That is fine: it is never landed on main (it is the
  BASE the lane diffs FROM, excluded from every slice). The slicer's derive_slice_base returns the watermark
  verbatim, so future slices = baseline..new_work, never including the baseline commit itself.

APPROACH (b) -- RE-CLONE / REPLACE THE FORK FROM MAIN
  Delete conductor/elai-repo and rebuild it from main's .elai_cc as a fresh repo (new root = main snapshot).
  REJECTED as the primary path:
  - ORPHANS all 128 integrate commits -> trust_audit/regate_backlog/wire_dark_audit go blind (above).
  - Destroys the 14 worktrees (they reference the old object store) -> the locked in-flight J-59 worker's
    work is lost.
  - Higher blast radius, harder rollback (must restore the whole directory from a bundle).
  Its only upside (a clean linear history) is not worth losing the re-gate toolchain that this entire
  quality-gap effort depends on. Keep (b) on the shelf ONLY if the user later decides the integrate-commit
  history is disposable (see OPEN QUESTION 1).

RECOMMENDED EXACT COMMAND SEQUENCE  (approach (a); controller runs these; daemon paused first per Safety 4)

  Notation: $FORK = F:\.ELAI_workflow\conductor\elai-repo ; $CK = F:\.ELAI_workflow (developer checkout).
  Run from a normal shell with cwd = conductor/ unless noted.

  # 0. PRE-FLIGHT (re-confirm the remote HEAD; it may have moved past d66c260):
  git -C "$CK" fetch origin main
  git -C "$CK" rev-parse origin/main            # capture as $MAIN_SHA (the true remote HEAD)
  git -C "$CK" rev-parse origin/main:.elai_cc   # confirm .elai_cc exists at that sha; capture as $MAIN_TREE

  # 1. BACKUP (Safety 4 -- do NOT skip):
  git -C "$FORK" branch backup-before-rebaseline-20260629 4f22c28
  git -C "$FORK" bundle create F:\.ELAI_workflow\conductor\state\fork-prebaseline-20260629.bundle --all
  copy state\control.db state\control.db.bak-rebaseline-20260629   # (Windows: copy; or cp under bash)

  # 2. STAGE MAIN'S .elai_cc CONTENT INTO THE FORK WORKING TREE (preserve .elai_cc-rooting).
  #    Use `git archive` of the .elai_cc SUBTREE from the fetched remote sha, extracted at the FORK ROOT
  #    (so main:.elai_cc/crates/X lands at fork:crates/X). This is the cleanest source: it is exactly the
  #    bytes on main, with no developer-checkout working-tree artifacts (the metadata.db/watch.lock/metrics
  #    junk in `git status` is uncommitted, so `git archive` of the COMMIT excludes it automatically).
  #
  #    First clear the fork working tree of all tracked project files so deletions on main propagate
  #    (files main removed must disappear from the fork). Do this WITHOUT touching .git:
  git -C "$FORK" rm -r --quiet --cached -- .            # unstage everything (index emptied)
  #    Then physically remove tracked working files but keep .git and any untracked scratch you want to keep.
  #    SAFEST: remove the known project dirs/files explicitly rather than a blanket rm (avoids nuking .git):
  #      (controller decides the exact set from `git -C "$FORK" ls-tree --name-only HEAD`; the 62 top-level
  #       entries are the project; do NOT delete .git). A blanket `git clean -xfd` is NOT recommended here
  #       because it also wipes untracked worktree-admin and any in-flight scratch.
  #
  #    Extract main's .elai_cc subtree at the fork root:
  git -C "$CK" archive --format=tar origin/main:.elai_cc | tar -x -C "$FORK"
  #      (Windows note: use bsdtar/tar from Git-for-Windows; or `git -C "$CK" archive -o tmp.tar
  #       origin/main:.elai_cc` then extract tmp.tar into $FORK. Verify extraction landed crates/ at the
  #       fork ROOT, not under a .elai_cc/ subdir.)

  # 3. STAGE + COMMIT the baseline (on top of fork HEAD; history preserved):
  git -C "$FORK" add -A
  git -C "$FORK" -c user.name=maintainer -c user.email=maintainer@example.invalid \
      commit -m "conductor baseline snapshot: .elai_cc @ <$MAIN_SHA short>"
  git -C "$FORK" rev-parse HEAD                          # capture as $NEW_BASELINE_SHA

  # 4. VERIFY the fork tree now equals main's .elai_cc (must be EMPTY diff):
  git -C "$CK" diff --no-index --stat \
      <(git -C "$FORK" archive HEAD) <(git -C "$CK" archive origin/main:.elai_cc)   # or compare file trees
  #    Simpler equivalent: produce a fork-rooted patch of "nothing new" -- see Verification (5).

  NOTE on the user's "preserve working-tree artifacts" intent (conductor-fork-reconcile local-synced entry):
  those artifacts (metadata.db, watch.lock, metrics/*.toml, smoke-chrome-profile/, test_cedar_contract.py)
  live in the DEVELOPER CHECKOUT and are gitignored/untracked there; they are NOT part of origin/main:.elai_cc
  (archive of the commit excludes them), and they are NOT in the fork. The re-baseline does not touch the
  developer checkout at all. No action needed for them here.

----------------------------------------------------------------------
3. STATE TO UPDATE AFTER RE-BASELINE (the high-risk part -- exhaustive)
----------------------------------------------------------------------

3.1 merge_watermark  (REQUIRED)
  Set to the NEW baseline commit so the slicer diffs future work from the aligned base:
    python -c "from conductor_db import ControlDB; ControlDB('state/control.db').set_merge_watermark('<$NEW_BASELINE_SHA>')"
  GROUNDING: merge_slicer.derive_slice_base returns the watermark verbatim (merge_slicer.py:129-130);
  build_slice_queue diffs base..fork_head; the lane's _is_terminal_for_window keys on the fork_range
  "<base12>..<head12>" (merge_lane.py:485,682-698). After this set, the next pass' base = new baseline,
  fork_head = current fork HEAD; if no NEW worker commits exist past the baseline yet, base..fork_head is the
  baseline-only delta = the baseline commit's own changes, which all collide with main (it IS main) -> the
  lane lands nothing, harmless. Once the daemon integrates a NEW task, that delta becomes the clean slice.

  *** SUBTLETY the controller must understand: right after the baseline commit, fork_head = $NEW_BASELINE_SHA
      too if you set the watermark to HEAD. base==fork_head -> empty queue -> clean no-op. As soon as a worker
      integrates, fork_head advances and base..fork_head = exactly the new task. This is the desired
      steady-state. So: set watermark = $NEW_BASELINE_SHA (the new HEAD). ***

3.2 merge_ledger rows  (REQUIRED -- clear the stale window)
  The 14 existing rows carry fork_range "dc9ef2ed..4f22c28" -- a DIFFERENT window from the new
  baseline..future. _is_terminal_for_window (merge_lane.py:682-698) ONLY skips a slice whose stored
  fork_range == the CURRENT window, so stale rows DO NOT block future landings (verified by reading the
  function: it returns False when fork_range differs). THEREFORE clearing is NOT strictly required for
  correctness -- the base-aware skip already ignores them.
  HOWEVER, clear/reset them anyway for OPERATOR CLARITY (the dashboard Quarantine/Escalation tiles read these
  rows and will show 8 phantom COLLISIONs forever otherwise). RECOMMENDED: do NOT delete history; instead
  record a dispositioning note or reset the COLLISION rows to a neutral status via the existing helper, OR
  (cleanest) leave them and document that they are pre-rebaseline noise. There is no DB helper that deletes
  ledger rows; record_merge_slice upserts by slice_id. The controller may UPSERT each stale slice_id to a
  PENDING/neutral status against the new window if a clean dashboard matters:
    (optional) for each stale slice_id: ControlDB(...).record_merge_slice(sid,'PENDING',kind=...,paths=[...],
               reason='pre-rebaseline stale window, superseded', fork_range='<new base12>..<new head12>')
  LEAN: leave them; they are inert under the base-aware skip. Clear only if the dashboard noise bothers the
  user (OPEN QUESTION 2).

3.3 task -> integrate_commit references  (PRESERVED by approach (a) -- this is WHY (a) wins)
  trust_audit.integration_commit greps the fork log (`git log --all -F --grep "Conductor integrate <task>"`,
  trust_audit.py:77). Under approach (a) the 128 integrate commits stay reachable (new commit is on TOP; old
  history intact), and `--all` would find them even if HEAD pointer changed. So trust_audit, regate_backlog,
  wire_dark_audit KEEP WORKING. *** Under approach (b) they would ALL break (orphaned commits) -> do not use
  (b) unless the user accepts losing the re-gate toolchain. ***
  NOTE: trust_audit also re-reads file content at fork HEAD for some checks (commit_changed_files/file_diff
  operate on the integrate commit's sha, which is unaffected; the cross-crate has_external_caller greps the
  CURRENT tree -- after re-baseline the tree is main's .elai_cc, which is fine, that is the real landed state).

3.4 in-flight worktrees  (HANDLED by Safety 4, verify after)
  `git worktree list` shows 14 worktrees; 13 at old fork HEAD 4f22c28, 1 LOCKED at 7273547 (J-59). After the
  baseline commit, HEAD = $NEW_BASELINE_SHA. Idle worktrees at 4f22c28 are unaffected (they are detached/own
  branches; a HEAD advance does not move them). The integrator rebases a finishing worker's one commit onto
  the latest HEAD before ff (integrator.py:237-239), so a worker that finishes AFTER the baseline rebases onto
  the baseline -- which is correct (its work then sits on top of an aligned tree). The DANGER is a worker that
  is MID-INTEGRATE (holding the integration mutex, integrator.py:22-32) WHILE the baseline commit is made:
  two writers on one HEAD. => the daemon MUST be paused and the locked worktree confirmed idle BEFORE the
  baseline commit (Safety 4). Do NOT prune/remove worktrees as part of re-baseline; leave them.

3.5 config (conductor.toml)  (NO CHANGE to paths; ONE arming decision)
  - [paths] repo_root, worktree_root, state_db are all absolute and UNCHANGED by re-baseline (the fork dir is
    reused in place under approach (a)). No edit needed.
  - merge_lane=true and merge_lane_live=true are CURRENTLY ARMED (conductor.toml:158,163). After re-baseline
    the lane will start diffing the aligned base. RECOMMENDATION: set merge_lane_live=FALSE for the first
    supervised pass after re-baseline (watch one dry pass prove a NEW worker increment slices clean), THEN
    re-arm. The lane's auto-LAND+push path is still UNPROVEN-LIVE (conductor-fork-reconcile residual). This is
    a config toggle the controller flips, not a code change.

----------------------------------------------------------------------
4. SAFETY (backup + pause + rollback)  -- do these BEFORE section 2 step 2
----------------------------------------------------------------------

BEFORE ANY MUTATION:
  S1. PAUSE THE DAEMON. control_state shows paused=false + active_wave=INTEGRATION (a daemon may be live).
      Run:  python conductor.py control pause
      Then CONFIRM: re-read control_state.paused == true. (Read-only SELECT or the dashboard.)
  S2. CONFIRM NO WORKER IS MID-INTEGRATE. The locked worktree J-59 (at 7273547) is the flag.
      - `git -C conductor/elai-repo worktree list` -> confirm no worktree's HEAD is advancing.
      - Check for a held integration mutex / active integrate_* worktree:
        `git -C conductor/elai-repo worktree list | findstr integrate_`  (none should be actively writing).
      - If a worker is genuinely mid-integrate, WAIT for it to finish (it will ff onto 4f22c28) or for the
        lease to expire (lease_ttl_seconds=3600), THEN re-confirm before proceeding. Do NOT force-unlock or
        delete the locked worktree.
  S3. BACKUP (section 2 step 1): fork branch `backup-before-rebaseline-20260629` + a full `git bundle`
      (--all, captures every ref incl the integrate history) + a control.db copy. The bundle is the
      belt-and-suspenders rollback if anything about the object store goes wrong.

ROLLBACK (if re-baseline goes wrong; controller-only, daemon stays paused):
  R1. Fork tree: `git -C conductor/elai-repo reset --hard 4f22c28` (or `... reset --hard
      backup-before-rebaseline-20260629`). HEAD returns to pre-baseline; the baseline commit is dropped.
  R2. If the object store itself is corrupted: restore from the bundle into a fresh dir and re-point, OR
      clone from the bundle (`git clone fork-prebaseline-20260629.bundle elai-repo-restored`) and swap.
  R3. Watermark: `python -c "from conductor_db import ControlDB;
      ControlDB('state/control.db').set_merge_watermark('4f22c2836b123582fe7c34b073e77da8ba3a8674')"` OR
      restore control.db from control.db.bak-rebaseline-20260629.
  R4. Re-confirm `git worktree list` is intact, then `python conductor.py control resume` (or RESUME path).
  REMOTE IS NEVER AT RISK: the fork has no remotes; only the merge-lane-clone pushes, and the lane is
  double-gated (merge_lane_live, ff-only, freeze check). No rollback step touches origin/main.

----------------------------------------------------------------------
5. VERIFICATION (prove success after re-baseline)
----------------------------------------------------------------------

V1. FORK == MAIN .elai_cc (the core invariant):
    git -C conductor/elai-repo archive HEAD > /tmp/fork.tar
    git -C F:\.ELAI_workflow archive origin/main:.elai_cc > /tmp/main_elaicc.tar
    # compare: extract both and `diff -r`, OR compare `git ... ls-tree -r HEAD` of the fork vs
    # `git ... ls-tree -r origin/main .elai_cc` (after stripping the .elai_cc/ prefix). EXPECT: no
    # project-file differences (allow .git, worktree-admin, and any deliberate fork-only infra you chose to
    # keep -- but ideally none; the cleaner the better).

V2. A TRIVIAL NEW FORK CHANGE APPLIES CLEAN ONTO A FRESH MAIN CLONE (the real goal):
    # Simulate the next worker increment: make a one-line change in the fork tree, commit it on top of the
    # baseline (this is a SCRATCH commit for the test; controller may discard it after, or leave it -- it is
    # legitimate fork work either way).
    # Then in a FRESH clone of main:
    git clone --no-hardlinks F:\.ELAI_workflow <scratch-clone>     # or clone origin/main
    git -C conductor/elai-repo diff <NEW_BASELINE_SHA>..HEAD -- crates/<X> > /tmp/probe.patch
    git -C <scratch-clone> apply --directory=.elai_cc --check /tmp/probe.patch
    # EXPECT: exit 0 (applies clean). THIS is the proof the re-baseline worked: a delta past the new baseline
    # lands on main without collision -- the exact failure the 8/14 COLLISIONs represented.

V3. THE LANE SEES THE ALIGNED BASE (dry run, no push):
    # With merge_lane_live=false, drive a dry catch-up:
    python conductor.py merge --catch-up           # (dry by default per merge_lane.py:436 dry_run=True)
    # EXPECT: base == <NEW_BASELINE_SHA>; if no new worker commits yet, "no slices to land" / empty queue
    # (merge_lane.py:473-475). After a real worker increment, EXPECT that crate's slice to gate (not
    # COLLISION). Read the MERGE_LANE_PASS event + ledger rows to confirm base advanced.

V4. THE DAEMON RESUMES AND A WORKER CAN STILL INTEGRATE:
    python conductor.py control resume
    # Watch one worker lease -> implement -> integrate onto the new baseline (integrator ff onto HEAD).
    # Confirm a fresh "Conductor integrate <task>" commit lands on top of $NEW_BASELINE_SHA and
    # `git worktree list` is healthy. Confirm trust_audit still resolves an OLD task:
    python trust_audit.py --n 5     # EXPECT it finds integrate commits (not all NO_COMMIT) -> history intact.

----------------------------------------------------------------------
6. OPEN QUESTIONS / FORKS for the controller (decide BEFORE executing)
----------------------------------------------------------------------

OQ1 (material). Preserve vs discard the 128 integrate-commit history.
  Options: (a) approach (a) new-baseline-commit, history PRESERVED [my strong lean]; (b) re-clone, history
  DISCARDED. LEAN (a): trust_audit/regate_backlog/wire_dark_audit all grep the fork log for "Conductor
  integrate <task>" (verified imports), and the entire conductor-quality-gap re-gate effort depends on that
  resolution. Discarding the history blinds the toolchain. Pick (b) ONLY if the user explicitly says the
  re-gate tooling is being retired. *** Controller must confirm this with the user before executing, because
  it determines which procedure to run. ***

OQ2 (minor). Clear the 8 stale COLLISION ledger rows, or leave them?
  They are inert under the base-aware skip (different fork_range), so correctness does not require clearing.
  Leaving them shows 8 phantom COLLISIONs on the dashboard forever. LEAN: leave them and document; clear
  (upsert to a neutral status against the new window) only if the user wants a clean dashboard. No DB
  row-delete helper exists; an upsert is the only mechanism.

OQ3 (operational). Re-confirm the true remote HEAD at execution time.
  This doc found origin/main = d66c260 (ahead of the stale 7d258a5 tracking ref). The remote may move again
  before execution. The controller MUST `git fetch origin main` + `git rev-parse origin/main` immediately
  before step 2 and re-baseline onto whatever that resolves to -- NOT onto the 7d258a5 in this doc.

OQ4 (in-flight). The locked J-59 worktree at 7273547.
  It is the one worker that may be mid-integrate. LEAN: pause the daemon, wait for it to finish or its lease
  to expire (<=3600s), confirm idle, THEN re-baseline. Do NOT force-unlock or delete it (R-2 / data-loss
  risk). If it is genuinely stuck/abandoned, that is a separate cleanup the user authorizes, not part of
  re-baseline.

OQ5 (developer checkout sync, out of scope but adjacent). The developer checkout local main (7d258a5) is now
  BEHIND the true remote (d66c260). The re-baseline sources from origin/main (correct), but the user may also
  want to fast-forward the local checkout 7d258a5 -> d66c260 (as was done last session 8f530a1 -> 7d258a5).
  That is a SEPARATE action, not required for re-baseline, and is a git mutation the controller/user performs.

----------------------------------------------------------------------
GROUNDING INDEX (commands run this session)
----------------------------------------------------------------------
- HEADs: git rev-parse HEAD (checkout + fork); git ls-remote origin main; git rev-parse origin/main;
  git cat-file -t d66c260 (absent); git fetch --dry-run origin main.
- Rooting: git ls-tree --name-only HEAD (fork=crates/...; main=.elai_cc/...); git ls-tree .elai_cc/.
- Commits: git rev-list --count HEAD (134); git log --grep "Conductor integrate" | count (128).
- Watermark: ControlDB('state/control.db').get_merge_watermark() == 4f22c28.
- Ledger: db.merge_ledger_rows() (14 rows; fork_range dc9ef2ed..4f22c28; 8 COLLISION).
- Daemon: SELECT * FROM control_state (paused=false, active_wave=INTEGRATION); no driver.pid; control.db
  mtime 13:51.
- Worktrees: git worktree list (14; J-59 locked at 7273547="Conductor integrate E-28-EXECUTE-1").
- Remotes: git -C fork remote -v (empty); git check-ignore conductor/elai-repo (matched).
- Code read: merge_lane.py (full -- derive base = watermark; _is_terminal_for_window base-aware skip;
  LaneRepo ff-only push; watermark advance only after real land), merge_slicer.py (full --
  derive_slice_base returns watermark verbatim; slice_patch excludes scratch), trust_audit.py (full --
  integration_commit greps fork log; regate_backlog + wire_dark_audit both import trust_audit),
  conductor_db.py:1077-1098 (get/set_merge_watermark), integrator.py (HEAD-based integrate + rebase-onto-HEAD
  + global merge mutex), worker_adapter.py (worktree off repo_root at task.base_commit), conductor.toml (full).
