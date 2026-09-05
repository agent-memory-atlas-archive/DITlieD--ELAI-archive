# P1: Codex worktree path layout + broken gitdir

## Symptom
Codex worker logs during E-11 EXECUTE:
- `Resolved gitdir path … does not exist` (many)
- `fatal: not a git repository`
- First probes: `ls crates/orchestrator/...` → No such file (actual tree under `.elai_cc/crates/...` in worktree)
- Occasional `CreateProcess … Failed to create unified exec process: No such file or directory` for `/bin/bash -lc …`

## Evidence
- `opus_dispatch.CODEX_WORKTREE_BASE = Path("/mnt/f/elai-codex-worktrees")`
- Worktree is a git worktree of F:\.ELAI_workflow with product under `.elai_cc/`
- Taskspecs often use paths relative to repo root like `crates/orchestrator/...` but product code lives at `.elai_cc/crates/...`
- codex-vm mount is **scoped to the worktree only** (network-off, cannot read `/mnt/f` rest) — correct for security; means layout must be self-describing inside worktree
- stderr gitdir errors suggest Windows/WSL worktree `.git` file points at a parent gitdir path that does not resolve **inside** the codex-vm view (or after vhdx/path translation)

## Root cause
1. **Path convention mismatch:** dispatch preamble says binding rules are inlined, but tool habits and file paths still assume `crates/` at CWD; CWD/worktree root is monorepo wrapper, not `.elai_cc`.
2. **Git worktree gitdir** may use absolute Linux host paths that do not exist inside codex-vm’s mount namespace.
3. Shell spawn failures are secondary (flaky codex runtime / path), not root of BLOCKED.

## Proposed fix
1. Codex task preamble (always-inline):  
   `REPO_ROOT=.elai_cc` for cargo and source paths; product files live under `.elai_cc/`; use those prefixes in allowed_writes OR rewrite allowed_writes at dispatch time to `.elai_cc/...` forms if missing prefix.
2. When creating codex worktree: rewrite `.git` file to use relative `gitdir:` if possible, or re-init with `git -C worktree rev-parse --git-dir` validated from inside the same mount.
3. Optional: second mount of nothing else; ensure `cargo` + rust toolchain available **on host land path only** (codex is not required to cargo — land is off-agent) — document so codex stops failing for missing cargo (ties to P0 land handoff).

## Test plan
- Unit: path rewrite helper for allowed_writes.
- Manual: create worktree, from sim env resolve gitdir.
- No full paid burn required for unit fix.

## Risk
- Path rewrite bugs could point writes outside product tree — keep allowlist under `.elai_cc/` only.
