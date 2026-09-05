# P0: Codex EXECUTE returns BLOCKED with patch; work never lands

## Symptom
- E-11 Task 8: three paid codex EXECUTEs (`artifacts/opuswalk-E-11-EXECUTE-*`).
  - #1: `patch.diff` 0 B — write-scope too narrow (missing ssb.rs / ssb/tests.rs).
  - #2–#3: `patch.diff` ~12–13 KB of real edits; final STATUS **BLOCKED** because `cargo` / checkers unavailable **inside** the codex sandbox; codex asks off-agent to verify.
- Canvas event: `OPUS_WORKER_DONE model=gpt-5-6-sol tier=PAID role=EXECUTE hint=BLOCKED`.
- No land/merge; HEAD stayed `081722b`. Worktrees under `/mnt/f/elai-codex-worktrees` were **reaped** by disk hygiene before any off-agent land.

## Evidence
- `opus_dispatch.dispatch_codex_executor`: requires non-empty `allowed_writes`; worktree forced to `CODEX_WORKTREE_BASE=/mnt/f/elai-codex-worktrees`; comment says “opus_land runs the frozen checkers… as the authority.”
- Codex is **not** supposed to self-certify DONE; frozen gates are off-agent (`opus_land.py` + `opus_preland_verify.py`).
- Disk hygiene (`disk_hygiene.py`): reaps non-in-flight worktrees under free + **codex** dirs between turns; protected set = queue + cursor plan ids only. A finished EXECUTE is **not** in-flight → reaped even with unlanded patch.
- Artifacts keep `worker_stdout.log` + `patch.diff` under `conductor/artifacts/` but supervisor has **no automatic path**: “if hint==BLOCKED and patch.diff non-empty → run opus_land on that worktree.”
- Patch capture may also be incomplete (empty on attempt 1); need land to prefer live worktree git over empty patch file.

## Root cause
1. **Missing handoff automation:** BLOCKED-for-missing-verifier is treated as terminal failure, not “patch candidate ready for off-agent land.”
2. **Aggressive codex worktree reap** deletes the only complete tree after EXECUTE exits.
3. **Allowed-writes incomplete** (P1-ssb) caused first zero-patch BLOCKED (separate but stacks).

## Proposed fix
1. After codex EXECUTE finishes, conductor-side (head already did; automate in dispatch return handling helper or a small `opus_post_execute.py` called by SKILL bootstrap):
   - If `hint in (BLOCKED, NEEDS_REVIEW, APPLIED)` and worktree still has diffs vs base:
     - Keep worktree (mark `.KEEP_FOR_LAND` or enroll plan_id as protected in walk-state).
     - Invoke `opus_land` (or queue a `pending_lands.json` row for the next Opus turn).
2. `disk_hygiene.in_flight_plan_ids` OR protected paths: never reap worktree that contains:
   - file `.KEEP_FOR_LAND`, or
   - matches a `pending_lands` entry, or
   - `patch.diff` non-empty under corresponding `artifacts/.../patch.diff` and no land yet.
3. On BLOCKED with non-empty patch: **never** treat as “no progress” for autoseed of same id until land attempt recorded.
4. If worktree already reaped: recover from `artifacts/*/patch.diff` via `git apply` into a fresh worktree (best-effort).

## Test plan
- Unit: disk_hygiene skips dir with `.KEEP_FOR_LAND`.
- Unit: pending_lands ledger load/save.
- Synthetic: empty patch + BLOCKED → no land attempt; non-empty patch → land path called (mock).
- Live smoke later: re-dispatch Task 8 slice once handoff exists.

## Risk
- Keeping worktrees forever fills F: again — only KEEP for unfinished lands; TTL 24h + size cap.
- Auto-land without human review: still gates via opus_land + preland verifier (fail-closed).
