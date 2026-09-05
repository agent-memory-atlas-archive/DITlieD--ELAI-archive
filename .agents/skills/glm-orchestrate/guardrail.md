glm-orchestrate emergency git/fs guardrail (spec for PHASE 0.6)

PURPOSE
A LEAN PreToolUse Bash hook for the GLM worker sessions. It is NOT the full agent-dispatch-preamble R-1 read-only-git ban (the user judged that too restrictive: the committer worker must run normal git add/commit/push). This is the EMERGENCY FLOOR only: block the catastrophic ops that can wipe the repo or drop commits by mistake, allow everything else.

Origin: 2026-06-11, an executing agent ran git reset --hard x8 (reflog-confirmed) and reverted every tracked integration edit to HEAD; 256 new files survived only because they were untracked. This guardrail makes that class of loss structurally impossible while still letting the committer commit and push per task (D1 in the build plan).

SCOPE
Registered as a PreToolUse hook on the Bash tool for the worker sessions launched by glm-worker.ps1. Matches the command string of any Bash invocation. On a match: DENY the tool call with the block message below and a non-zero exit so the worker sees the denial and routes to BLOCKED. On no match: allow.

BANNED OPS (DENY these; case-insensitive, tolerate extra whitespace and flag-reordering)
Match the command string (after trimming, collapsing runs of whitespace to single spaces) against these patterns. A match anywhere in the command (including inside && / ; / | chains and subshells) is a DENY.

  1. git reset --hard
       regex: \bgit\b[^\n;&|]*\breset\b[^\n;&|]*--hard\b
  2. git clean with -f and a recursive/force-dir flag (-fd, -fdx, -dfx, -xfd, -ffd, etc.)
       regex: \bgit\b[^\n;&|]*\bclean\b[^\n;&|]*-[a-z]*f[a-z]*d|\bgit\b[^\n;&|]*\bclean\b[^\n;&|]*-[a-z]*d[a-z]*f
       (simpler: any `git clean` carrying both an f flag and a d flag in any -combo, or -fdx/-fdX)
  3. git push --force / -f / --force-with-lease
       regex: \bgit\b[^\n;&|]*\bpush\b[^\n;&|]*(--force\b|--force-with-lease\b|\s-f\b)
  4. git rebase (any form, interactive or not)
       regex: \bgit\b[^\n;&|]*\brebase\b
  5. git branch -D / -d on main (delete the main branch)
       regex: \bgit\b[^\n;&|]*\bbranch\b[^\n;&|]*\s-[dD]\b[^\n;&|]*\bmain\b
  6. git filter-branch / git filter-repo
       regex: \bgit\b[^\n;&|]*\bfilter-(branch|repo)\b
  7. git reflog expire
       regex: \bgit\b[^\n;&|]*\breflog\b[^\n;&|]*\bexpire\b
  8. git checkout / git restore with -- <paths> (mass discard of working-tree changes)
       regex: \bgit\b[^\n;&|]*\b(checkout|restore)\b[^\n;&|]*\s--\s
       NOTE: a single-file scoped checkout/restore is ALLOWED (see below); the DENY is the `-- <paths>` mass-discard form when more than one path follows or a glob/`.`/`*` follows. Conservative rule: DENY `git checkout -- .`, `git checkout -- *`, `git restore --staged --worktree .`, and any `-- ` followed by 2+ space-separated paths. ALLOW `git checkout -- src/foo.rs` (exactly one concrete file path, no glob).
  9. git update-ref -d (delete a ref directly)
       regex: \bgit\b[^\n;&|]*\bupdate-ref\b[^\n;&|]*\s-d\b
 10. rm -rf / Remove-Item -Recurse -Force targeting the repo root or .git
       bash regex: \brm\b[^\n;&|]*-[a-z]*r[a-z]*f|\brm\b[^\n;&|]*-[a-z]*f[a-z]*r , AND the target resolves to the repo root (F:\.ELAI_workflow or its drive-relative form), the .elai_cc tree root, or any path ending in \.git or /.git
       powershell regex: \bRemove-Item\b[^\n;&|]*-Recurse\b[^\n;&|]*-Force\b , same target restriction
       (a scoped rm -rf inside a scratch dir the worker created this task is ALLOWED; the DENY is repo-root / .git / .elai_cc-root targets)

ALLOWED (never block these)
  - git add, git add <paths>, git add -A (committer scopes by path; -A allowed but committer prompt forbids sweeping unrelated work)
  - git commit, git commit -m "...", git commit --amend (amend is allowed; not a history-wipe)
  - git push (non-force), git push origin <branch>
  - git status, git diff, git log, git show, git ls-files, git fetch
  - git checkout <branch> (switch branches), git checkout -b <new> (create branch)
  - git checkout -- <single concrete file path> (scoped single-file restore)
  - git stash / git stash pop (NOT in the banned list; allowed — non-destructive enough for the lean floor)
  - all non-git Bash: file edits via the Edit/Write tools, build commands, the checker scripts, scoped rm inside a worker scratch dir

MATCH HARDENING (so the hook is not trivially bypassed)
  - normalize first: lowercase the command for the git keyword match (paths stay case-sensitive for the repo-root/.git target test), strip surrounding quotes, collapse whitespace.
  - scan the WHOLE command line including chained segments split on && ; | and inside $(...) / `...` / powershell ; — a banned op hidden in a chain is still a DENY.
  - a leading env-var prefix or a `git -C <path>` / `git --git-dir=...` prefix does not exempt: still match the subcommand.
  - if the command is ambiguous (e.g. a variable-expanded git subcommand the hook cannot resolve), FAIL-CLOSED on the catastrophic subcommands only (reset/clean/filter/reflog/update-ref/push-force) — DENY when the dangerous keyword is present even if flags are variable.

BLOCK MESSAGE (printed to stderr, hook exits non-zero)
  GLM-ORCHESTRATE GUARDRAIL: blocked catastrophic op "<the matched command>".
  This op is banned for worker sessions (it can wipe the repo or drop commits).
  Allowed: normal git add/commit/push (non-force), file edits, single-file checkout.
  If you believe this op is genuinely required, STOP and return BLOCKED with the reason; the controller (Opus) handles any destructive git only with explicit user confirmation.

ACCEPTANCE (PHASE 0.6 proves)
  - DENY: `git reset --hard HEAD~5`  -> blocked, non-zero exit, message shown.
  - DENY: `git push --force`, `git push -f origin main`, `git clean -fdx`, `git rebase -i HEAD~3`, `git filter-repo --path x`, `rm -rf F:\.ELAI_workflow\.git`.
  - ALLOW: `git commit -m "wire the foo hook"` then `git push`  -> both run.
  - ALLOW: `git checkout -- .elai_cc/crates/foo/src/bar.rs`  (single concrete file).

IMPLEMENTATION NOTE (for PHASE 0.6)
Implement as a small PreToolUse Bash hook script (bun or stdlib python, per the harness hook convention) registered in the worker session's settings. Keep the pattern list as named constants. The hook is frozen relative to a worker run (a worker cannot edit its own guardrail — it lives outside the worker's write scope). Treat this spec as the contract; the regexes above are the starting patterns, harden per the MATCH HARDENING notes.
