Source: treehouse (github.com/kunchenguid/treehouse)
Reliability: MEDIUM (reviewed public source, depth-1 clone, tag v1.8.0)
Files examined:
  F:\.ELAI_workflow\docs\.source\treehouse\internal\pool\pool.go
  F:\.ELAI_workflow\docs\.source\treehouse\internal\pool\state.go
  F:\.ELAI_workflow\docs\.source\treehouse\internal\pool\prune.go
  F:\.ELAI_workflow\docs\.source\treehouse\internal\pool\pool_test.go
  F:\.ELAI_workflow\docs\.source\treehouse\cmd\get.go
  F:\.ELAI_workflow\docs\.source\treehouse\cmd\return_cmd.go
  F:\.ELAI_workflow\docs\.source\treehouse\go.mod
  F:\.ELAI_workflow\docs\.source\treehouse\README.md

Language: Go 1.25.5
Entry point: main.go -> cmd/ (cobra root)
Commands: get (default), return, status, prune, destroy, init, update
Pool root: ~/.treehouse/ (user-level default), per-repo: <poolDir>/<name>/<repoName>/

Public API (internal/pool):

Acquire | func(repoRoot, poolDir string, poolSize int, postCreate []string) (string, error) | string=worktreePath | interactive mode; spawns owner reservation via PID+startedAt
AcquireLease | func(repoRoot, poolDir string, poolSize int, postCreate []string, holder string) (string, error) | string=worktreePath | durable, process-independent reservation; prints path to stdout only
Release | func(poolDir, worktreePath string) error | error | clears owner reservation or lease, resets worktree to default branch
List | func(poolDir string) ([]WorktreeStatus, error) | []WorktreeStatus | returns statuses: available/dirty/in-use/leased/you're here
Destroy | func(repoRoot, poolDir, worktreePath string, force bool, preDestroy []string) error | error | removes one worktree; refuses in-use/leased without --force
DestroyAll | func(repoRoot, poolDir string, force bool, preDestroy []string) error | error | removes all managed worktrees
FindByPath | func(poolDir, path string) (*WorktreeEntry, error) | *WorktreeEntry | lookup by path in state file

State structs (internal/pool/state.go):

```go
// state.go:10-27
type WorktreeEntry struct {
    Name           string    `json:"name"`
    Path           string    `json:"path"`
    CreatedAt      time.Time `json:"created_at"`
    Destroying     bool      `json:"destroying,omitempty"`
    OwnerPID       int32     `json:"owner_pid,omitempty"`
    OwnerStartedAt int64     `json:"owner_started_at,omitempty"`
    Leased bool `json:"leased,omitempty"`
    LeaseHolder string `json:"lease_holder,omitempty"`
    LeasedAt time.Time `json:"leased_at,omitempty,omitzero"`
}

type State struct {
    Worktrees []WorktreeEntry `json:"worktrees"`
}
```

State file: <poolDir>/treehouse-state.json (JSON, MarshalIndent)
Lock file: <poolDir>/treehouse-state.lock (OS-level file lock via lock_unix.go / lock_windows.go)
All mutations go through: WithStateLock(poolDir, func() error) error

Core acquire algorithm (pool.go:71-165):

```go
// pool.go:71-165 (acquire)
func acquire(repoRoot, poolDir string, poolSize int, postCreate []string, opts acquireOptions) (string, error) {
    branch, _ := git.GetDefaultBranch(repoRoot)
    if git.HasRemote(repoRoot, "origin") {
        git.Fetch(repoRoot)
    }
    err = WithStateLock(poolDir, func() error {
        state = healState(state)
        // Try available: not destroying, not leased, ownerAlive==false, not in-use, not dirty
        for i, wt := range state.Worktrees {
            if wt.Destroying || wt.Leased || ownerAlive(wt) { continue }
            inUse, _ := process.IsWorktreeInUse(wt.Path)
            if inUse { continue }
            dirty, _ := git.IsDirty(wt.Path)
            if dirty { continue }
            git.ResetWorktree(wt.Path, branch)
            markAcquired(&state.Worktrees[i], opts)
            acquired = wt.Path
            WriteState(poolDir, state)
            return nil
        }
        // Pool full? error
        if len(state.Worktrees) >= poolSize { return fmt.Errorf("all %d worktrees are in use...", ...) }
        // Create new worktree
        git.AddWorktree(repoRoot, wtPath, branch)
        state.Worktrees = append(state.Worktrees, entry)
        WriteState(poolDir, state)
        return nil
    })
    hooks.Run(postCreate, acquired, ...)
    return acquired, nil
}
```

markAcquired (pool.go:169-180):
- lease mode: sets Leased=true, LeaseHolder, LeasedAt; OwnerPID=0 (process-independent)
- interactive mode: reserveOwner() -> sets OwnerPID=os.Getpid(), OwnerStartedAt=process.StartedAt(pid)

ownerAlive (pool.go:452-458): checks OwnerPID != 0 && process.StartedAt(PID) == OwnerStartedAt (PID recycling safe)

healState (pool.go:436-450): removes entries whose path no longer exists on disk; clears dead owner reservations; never touches leased entries

Worktrees use detached HEAD mode, reset to whichever of local or remote default branch is further ahead.

Behavioral contracts:
git.Fetch -> runs on every acquire if origin exists (not optional)
git.ResetWorktree -> called before handing out a reused worktree AND during Release
post_create hook -> runs AFTER state lock released, in worktree dir; failure is logged, does not abort get
pre_destroy hook -> runs before removal; failure is logged, does not abort destroy
Dirty check -> includes untracked files even when repo config hides them from normal git status
Prune safety -> skips in-use, leased, dirty, unmerged; dry run unless --yes; cannot verify orphans

Dependencies (go.mod, verified from lockfile):
github.com/BurntSushi/toml v1.6.0   (config parsing)
github.com/fatih/color v1.18.0      (terminal color)
github.com/shirou/gopsutil/v4 v4.26.2 (process detection: IsWorktreeInUse, StartedAt)
github.com/spf13/cobra v1.10.2      (CLI)
golang.org/x/sys v0.41.0            (file locking)

Platform constraints:
lock_unix.go / lock_windows.go: OS-level file lock (syscall.Flock on unix, LockFileEx on windows)
process detection via gopsutil v4 (cross-platform)
hooks via /bin/sh -c (unix) or %COMSPEC% /c (windows)
Hooks in repo-level treehouse.toml are SILENTLY IGNORED for safety; only user-level ~/.config/treehouse/config.toml hooks run

Tests (internal/pool/pool_test.go):
Real git repos created in t.TempDir(), full acquire/release/destroy/lease cycle tested
Concurrent Acquire tested (sync.WaitGroup, 16 parallel goroutines)
Dirty detection, prune, orphan prune all tested
cmd/e2e_test.go: binary-level end-to-end tests

Gotchas:
Pool exhaustion is a hard error (all N worktrees in-use/dirty) - no wait/queue
git.Fetch runs unconditionally on acquire when origin exists (can slow acquisition on bad network)
OwnerPID is NOT leased mode; these are orthogonal fields (leased entries carry OwnerPID=0)
healState is called on every acquire and List, keeping state self-healing
The `Destroying` flag + owner reservation prevents concurrent destroys from racing

Critical assertions (testable):
assert state.Worktrees[i].Leased == False or state.Worktrees[i].OwnerPID == 0  # never both set
assert os.path.exists(wt.Path) for wt in state.Worktrees  # healState enforces
assert lock_file_held while state.json is being written  # WithStateLock guarantees
