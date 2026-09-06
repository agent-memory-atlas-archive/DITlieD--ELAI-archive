#!/usr/bin/env bash
# Hybrid circuit-breaker (owner 2026-08-06):
#   walk-descended local build, UNCAGED  -> write FALLBACK_WALK_STOP (pause the walk)
#   walk-descended local build, CAGED    -> log violation, walk continues (cage bounds it)
#   walk-descended worker CLI, UNCAGED   -> write FALLBACK_WALK_STOP (boss hand-spawn side door)
#   non-walk processes                   -> report only, never touched
set -u
STOP_FILE=/home/researcher/teikoku/conductor/FALLBACK_WALK_STOP
VIOLATIONS=/home/researcher/teikoku/conductor/state/tripwire-violations.log
WORKTREE_ROOT=/home/researcher/teikoku/conductor/elai-worktrees
walk_ancestor() {
  local p="$1" depth=0
  ANCESTRY=""
  while [ -n "$p" ] && [ "$p" != "1" ] && [ "$depth" -lt 15 ]; do
    local cmd
    cmd=$(ps -o args= -p "$p" 2>/dev/null) || return 1
    ANCESTRY="${ANCESTRY}${p}:${cmd:0:90} <- "
    case "$cmd" in
      *fallback_walk_driver.sh*|*opus_dispatch.py*) return 0 ;;
    esac
    p=$(ps -o ppid= -p "$p" 2>/dev/null | tr -d ' ')
    depth=$((depth + 1))
  done
  return 1
}
# Caged = inside one of OUR cage scope names: anonymous systemd-run (run-*.scope) or the NAMED
# per-dispatch prime scopes (--unit prime-<task_id> => prime-*.scope). v5 bug found live
# 2026-08-07: prime workers run in named scopes since the daemon-escape fix, so accepting only
# run-*.scope broke the walk on a fully sanctioned caged prime EXECUTE (J-93, retracted in
# tripwire-violations.log).
caged() { grep -Eq "(run-[^/]*|prime-[^/]*)\.scope" "/proc/$1/cgroup" 2>/dev/null; }
prime_agent_argv() {
  local pid="$1" a0 a1 _rest
  read -r a0 a1 _rest < <(tr '\0' ' ' < "/proc/$pid/cmdline" 2>/dev/null) || return 1
  local base0
  base0=$(basename "${a0:-}")
  case "$base0" in
    node|bun) case "${a1:-}" in *prime-agent-dist/cli.js) return 0 ;; esac ;;
  esac
  case "${a0:-}" in *prime-agent-dist/cli.js|*prime-agent-dist/*daemon*) return 0 ;; esac
  return 1
}

break_walk() {
  local why="$1"
  if [ ! -f "$STOP_FILE" ]; then
    printf "tripwire break: %s\n" "$why" > "$STOP_FILE"
  fi
  printf "%s BREAK %s || ancestry: %s\n" "$(date -u +%FT%TZ)" "$why" "${ANCESTRY:-unknown}" >> "$VIOLATIONS"
  echo "TRIPWIRE-BREAK $why"
}
log_violation() {
  printf "%s %s\n" "$(date -u +%FT%TZ)" "$1" >> "$VIOLATIONS"
  echo "TRIPWIRE-CAGED (logged, walk continues) $1"
}
while :; do
  for pid in $(pgrep -f "rustup/toolchains/.*/bin/(cargo|rustc)" 2>/dev/null); do
    args=$(ps -o args= -p "$pid" 2>/dev/null) || continue
    # v6 (2026-08-07): match the BUILD PROCESS, not text mentions. Worker prompts QUOTE the
    # banned path inside the BUILD LAW ("BANNED: absolute-path cargo/rustc (~/.rustup/...)"),
    # so a full-cmdline grep flagged every caged worker shell carrying its taskspec in argv
    # (two log-only false hits in 10min). A real local build always has argv0 == the cargo/
    # rustc binary itself; a shell that spawned one is covered by the child's own pid.
    a0=$(tr '\0' '\n' < "/proc/$pid/cmdline" 2>/dev/null | head -1)
    case "$(basename "${a0:-}")" in
      cargo|rustc) : ;;
      *) continue ;;
    esac
    if walk_ancestor "$pid"; then
      if caged "$pid"; then
        log_violation "pid=$pid local-build-in-cage: ${args:0:120}"
        sleep 120
      else
        # v7 (2026-08-07): KILL the uncaged build, do NOT stop the walk. Three uncaged
        # absolute-path BOSS builds in one night (glm x1, macaron x2): prose + shim PATH
        # cannot stop an absolute-path invocation, and each sentinel stop halts the whole
        # 24/7 drain for a process the cage rules already outlaw. Kill ONLY the build
        # process (argv0-exact cargo/rustc, walk-descended, uncaged — never workers, never
        # caged pids): the disobedient boss sees an honest failed command; RAM/disk stay
        # protected with zero walk downtime. The OOM-lesson kill ban was about worker CLIs
        # mid-edit, not boss build subprocesses. A process that SURVIVES the kill still
        # escalates to the full sentinel break.
        kill -TERM "$pid" 2>/dev/null
        sleep 5
        kill -0 "$pid" 2>/dev/null && { kill -KILL "$pid" 2>/dev/null; sleep 2; }
        if kill -0 "$pid" 2>/dev/null; then
          break_walk "pid=$pid UNCAGED walk build SURVIVED kill: ${args:0:100}"
          sleep 240
        else
          echo "TRIPWIRE-KILLED uncaged walk build pid=$pid: ${args:0:120}"
          log_violation "pid=$pid KILLED uncaged walk build (v7, no walk stop): ${args:0:100}"
          sleep 60
        fi
      fi
    else
      echo "TRIPWIRE-REPORT pid=$pid (NOT walk-descended, untouched): ${args:0:100}"
      sleep 240
    fi
  done
  # Worker check judges ARGV SHAPE, never full-cmdline text: boot prompts legally
  # contain worker command strings ("claude --bare", "prime-agent"), so a text grep
  # matches the boss's own prompt. A worker is: argv0 basename "claude" with --bare,
  # or node/bun executing prime-agent-dist/cli.js. The boss binary is never a worker.
  for pid in $(pgrep -u "$(id -u)" -f "claude|cli\.js" 2>/dev/null); do
    read -r a0 a1 _ < <(ps -o args= -p "$pid" 2>/dev/null) || continue
    base0=$(basename "${a0:-}")
    case "$base0" in
      systemd-run|grok-local-nest2*) continue ;;
    esac
    is_worker=0
    if [ "$base0" = "claude" ] && [ "${a1:-}" = "--bare" ]; then is_worker=1; fi
    case "${a1:-}" in *prime-agent-dist/cli.js) [ "$base0" = "node" ] || [ "$base0" = "bun" ] && is_worker=1 ;; esac
    [ "$is_worker" = "1" ] || continue
    if walk_ancestor "$pid" && ! caged "$pid"; then
      sleep 5
      kill -0 "$pid" 2>/dev/null || continue
      caged "$pid" && continue
      args=$(ps -o args= -p "$pid" 2>/dev/null) || continue
      break_walk "pid=$pid UNCAGED walk worker (hand-spawned outside opus_dispatch): ${args:0:100}"
      sleep 240
    fi
  done
  # A daemon-executed Prime session has no walk ancestry. Its worktree cwd is the escape proof.
  for pid in $(pgrep -u "$(id -u)" -f "prime-agent-dist" 2>/dev/null); do
    prime_agent_argv "$pid" || continue
    caged "$pid" && continue
    cwd=$(readlink "/proc/$pid/cwd" 2>/dev/null) || continue
    case "$cwd" in
      "$WORKTREE_ROOT"|"$WORKTREE_ROOT"/*)
        args=$(tr '\0' ' ' < "/proc/$pid/cmdline" 2>/dev/null) || continue
        break_walk "pid=$pid UNCAGED prime worktree escape: ${args:0:120} cwd=$cwd"
        sleep 240
        ;;
    esac
  done
  sleep 45
done
