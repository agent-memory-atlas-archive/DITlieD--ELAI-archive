Source: firstmate (github.com/kunchenguid/firstmate)
Reliability: MEDIUM (reviewed public source, depth-1 clone, macOS/Linux only per README badge)
Files examined:
  F:\.ELAI_workflow\docs\.source\firstmate\AGENTS.md         (complete orchestration spec, ~8KB)
  F:\.ELAI_workflow\docs\.source\firstmate\bin\fm-spawn.sh   (worker launch)
  F:\.ELAI_workflow\docs\.source\firstmate\bin\fm-watch.sh   (event watcher)
  F:\.ELAI_workflow\docs\.source\firstmate\bin\fm-teardown.sh
  F:\.ELAI_workflow\docs\.source\firstmate\bin\fm-brief.sh
  F:\.ELAI_workflow\docs\.source\firstmate\bin\fm-bootstrap.sh
  F:\.ELAI_workflow\docs\.source\firstmate\bin\fm-home-seed.sh
  F:\.ELAI_workflow\docs\.source\firstmate\bin\fm-supervise-daemon.sh
  F:\.ELAI_workflow\docs\.source\firstmate\tests\*.test.sh
  F:\.ELAI_workflow\docs\.source\firstmate\.no-mistakes.yaml
  F:\.ELAI_workflow\docs\.source\firstmate\README.md

Language: Bash (entire bin/ toolbelt). Platform: macOS / Linux only (no Windows support).
No-app install: the repo IS the system. Run claude / codex / opencode / pi from the repo root.
AGENTS.md drives all agent behavior; CLAUDE.md is a symlink to it.

Architecture: AGENTS.md-driven meta-orchestrator

The first mate IS an LLM agent reading AGENTS.md. There is no compiled binary, no framework import.
The orchestrator is the prompt in AGENTS.md, executed by whatever harness the captain launched.
The bin/ scripts are tool-like helpers the LLM calls as shell commands.

Identity constraint (AGENTS.md section 1, hard rules):
1. Never write to a project (projects/ or any worktree) -- 3 sanctioned exceptions: fleet-sync, local-only merge, tool-driven project init
2. Never merge a PR without captain's explicit word (relaxed by project +yolo flag for routine decisions)
3. Never tear down a worktree with unlanded work (fm-teardown.sh enforces this; never bypass without captain order)
4. Crewmates never address the captain; all communication flows through first mate
5. Report outcomes faithfully, including failures

Task lifecycle (the core mechanism):

```
captain request
    |
    v
firstmate: judge scope (new vs backlog) + route (direct crewmate or secondmate domain)
    |
    v
fm-brief.sh: scaffold data/<id>/brief.md  (ship | scout | secondmate charter)
    |
    v
fm-spawn.sh <id> <project-dir> [harness] [--scout|--secondmate]
    |  -- runs 'treehouse get' in a new tmux window (kind=ship or kind=scout)
    |  -- for secondmate: launches in isolated FM_HOME, no worktree
    |  -- installs per-harness turn-end hook (*.turn-ended touch)
    |  -- writes state/<id>.meta (window, worktree, project, harness, kind, mode, yolo)
    |
    v
crewmate runs autonomously (agent reads brief.md, does work, writes state/<id>.status lines)
    |
    v
fm-watch.sh (singleton, background) blocks until one of:
  - signal: status file or turn-ended marker changed (stat size:mtime signature, not mtime-only)
  - stale:  pane identical 2+ cycles with no busy signature in last 6 non-blank lines
  - check:  per-task check.sh produced output (e.g. merged-PR poll, every FM_CHECK_INTERVAL=300s)
  - heartbeat: fleet review due (base FM_HEARTBEAT=600s, backs off 2^streak to FM_HEARTBEAT_MAX=7200s)
    |
    v
firstmate wakes, reads state, handles wake type, re-arms watcher
    |
    v
on ship task completion: fm-teardown.sh (returns worktree to treehouse pool, checks unpushed work)
```

fm-spawn.sh launch templates (bin/fm-spawn.sh:108-130):

```bash
# fm-spawn.sh:108-130
launch_template() {
  local harness=$1 kind=${2:-ship}
  case "$harness" in
    claude)   printf '%s' 'claude --dangerously-skip-permissions "$(cat __BRIEF__)"' ;;
    codex)
      if [ "$kind" = secondmate ]; then
        printf '%s' 'codex --dangerously-bypass-approvals-and-sandbox "$(cat __BRIEF__)"'
      else
        printf '%s' 'codex --dangerously-bypass-approvals-and-sandbox -c "notify=[\"bash\",\"-c\",\"touch __TURNEND__\"]" "$(cat __BRIEF__)"'
      fi ;;
    opencode) printf '%s' 'OPENCODE_CONFIG_CONTENT='\''{"permission":{"*":"allow"}}'\'' opencode --prompt "$(cat __BRIEF__)"' ;;
    pi)
      if [ "$kind" = secondmate ]; then
        printf '%s' 'pi "$(cat __BRIEF__)"'
      else
        printf '%s' 'pi -e __PIEXT__ "$(cat __BRIEF__)"'
      fi ;;
  esac
}
```

Placeholders: __BRIEF__ -> absolute path to data/<id>/brief.md; __TURNEND__ -> state/<id>.turn-ended; __PIEXT__ -> state/<id>.pi-ext.ts

Turn-end hook installation (fm-spawn.sh:343-388):
- claude: writes .claude/settings.local.json with Stop hook touching state/<id>.turn-ended; excludes from git
- opencode: writes .opencode/plugins/fm-turn-end.js with session.idle handler; excludes from git
- pi: writes state/<id>.pi-ext.ts OUTSIDE worktree (pi project-trust gate avoids dialog), loaded via -e flag
- codex: turn-end rides the -c notify launch arg (__TURNEND__ substitution)

fm-watch.sh waker mechanism (bin/fm-watch.sh:159-264):

```bash
# fm-watch.sh:130-145 (scan_signals, size:mtime signature, not mtime-only)
scan_signals() {
  for f in "$STATE"/*.status "$STATE"/*.turn-ended; do
    sig=$(stat_sig "$f") || continue  # sig = "size:mtime_epoch"
    sf="$STATE/.seen-$(basename "$f" | tr '.' '_')"
    if [ "$sig" != "$(cat "$sf" 2>/dev/null)" ]; then
      printf '%s\t%s\t%s\n' "$sf" "$sig" "$f"
    fi
  done
}
```

Signal coalescing: on first signal, linger FM_SIGNAL_GRACE=30s and re-scan; trailing signals from same turn coalesce into one wake.
Stale detection: hash last 40 lines of tmux pane; two consecutive identical hashes with no busy signature in last 6 non-blank lines -> stale wake.
Watcher is singleton (lock on state/.watch.lock). Re-arm after each wake by calling fm-watch.sh again.
Durable queue: state/.wake-queue stores pending wakes (epoch TAB seq TAB kind TAB key TAB payload) before watcher exits, so missed wakes survive watcher process death.

Sub-supervisor (bin/fm-supervise-daemon.sh, /afk mode):
Presence-gated: activated by /afk skill; deactivated on captain return (state/.afk file).
Self-handles routine wakes in bash; escalates only captain-relevant events as one batched single-line digest with sentinel prefix.
FM_INJECT_SKIP=heartbeat: heartbeat wakes are force-self-handled, bypassing classification.
FM_STALE_ESCALATE_SECS=240: idle seconds before stale pane escalates as possible wedge.
FM_ESCALATE_BATCH_SECS=90: batching window for escalation digests.

Secondmate (persistent domain supervisor):
Registered in data/secondmates.md (natural-language scope, project clone list, home path).
Provisioned via fm-home-seed.sh: leases a treehouse worktree as persistent FM_HOME, clones projects, initializes no-mistakes gate. Transaction: if any step fails, rollback cleans briefs, clones, home, registry edit.
Launched via fm-spawn.sh --secondmate: starts harness in isolated FM_HOME with FM_HOME env var set.
Supervised like any other direct report (same status/turn-end/watcher/heartbeat flow).
Teardown refuses while in-flight work exists unless captain approves --force discard.
backlog-handoff: fm-backlog-handoff.sh moves judged in-scope main-backlog items to secondmate home.

Project delivery modes (data/projects.md, fm-project-mode.sh):
no-mistakes: full validation pipeline before PR
direct-PR: PR without pipeline
local-only: fast-forward merge into local default branch (no PR, no remote push)
+yolo flag: firstmate makes routine approval decisions autonomously for that project

State layout:
data/backlog.md     task queue
data/projects.md    per-project delivery mode + yolo flag
data/secondmates.md secondmate routing table
data/<id>/brief.md  per-task crewmate brief (or charter for secondmates)
data/<id>/report.md scout deliverable (survives teardown)
state/<id>.status   crewmate status lines (appended by crewmate, watched)
state/<id>.turn-ended touched by turn-end hook
state/<id>.meta     window=, worktree=, project=, harness=, kind=, mode=, yolo=
state/<id>.check.sh optional per-task slow poll (e.g. merged-PR check)
state/.wake-queue   durable queued wakes (epoch TAB seq TAB kind TAB key TAB payload)
state/.afk          /afk mode flag

Environment variables:
FM_HOME              operational home; unset = repo root
FM_POLL=15           watcher cycle seconds
FM_HEARTBEAT=600     base heartbeat interval
FM_HEARTBEAT_MAX=7200 heartbeat backoff cap
FM_CHECK_INTERVAL=300 slow check cadence
FM_CHECK_TIMEOUT=30  per-check-script timeout
FM_GUARD_GRACE=300   stale watcher beacon tolerance
FM_SIGNAL_GRACE=30   coalesce window for trailing signals
FM_FLEET_PRUNE=1     set to 0 to skip gone-branch pruning
FM_BUSY_REGEX        busy signature regex (extend per harness)
FM_STALE_ESCALATE_SECS=240
FM_ESCALATE_BATCH_SECS=90
FM_SUPERVISOR_TARGET  sub-supervisor tmux target override

Tests (tests/*.test.sh):
fm-spawn-batch.test.sh: batch dispatch arg routing (no tmux/treehouse; fails fast at missing-brief)
fm-bootstrap.test.sh: dependency detection and feature-probe logic
fm-secondmate.test.sh: secondmate routing, seeding, idle charter, backlog handoff, spawn, recovery, teardown, FM_HOME
fm-teardown.test.sh: unpushed-work safety checks (local-only fork-remote allow, truly-unpushed refuse, merged-to-main allow, --force override)
fm-wake-queue.test.sh: durable wake queue, singleton behavior, sub-supervisor classifier, /afk presence-gating
fm-afk-inject-e2e.test.sh: private-socket end-to-end test of afk injection path (partial-input deferral, swallowed-Enter retry)
CI enforces shellcheck on bin/*.sh and tests/*.sh.

Toolchain dependencies (detected at bootstrap, not pre-installed):
treehouse (with --lease support = v1.5+)
tmux
gh + gh auth login
node
no-mistakes (validation pipeline for no-mistakes projects)
gh-axi, chrome-devtools-axi, lavish-axi (optional; bootstrap lists with install commands)

Gotchas:
Platform: macOS / Linux ONLY; no Windows support (tmux, /bin/sh -c, POSIX signals)
Watcher is a ONE-SHOT process: re-arms after every wake; in-memory state is lost on each restart
fm-teardown.sh refuses to remove a worktree with unpushed work (safety); --force bypasses ONLY with captain approval
Secondmate homes must not nest (fm-home-seed.sh validate refuses duplicate/nested homes)
config/crew-harness file only affects crewmates, not firstmate itself; firstmate's own harness is the one that launched it
Turn-end hooks are installed outside git tracking (.claude/settings.local.json excluded via .git/info/exclude)
local-only projects stay with main firstmate (not routable to secondmates)
Telemetry: none observed. Anonymous telemetry belongs to gnhf, not firstmate.

Critical assertions (testable):
assert state/<id>.meta exists before any wake processing (spawn writes it atomically)
assert state/.watch.lock held by at most one watcher process at a time (singleton lock)
assert wake-queue entry written BEFORE watcher exits (durable queue semantics)
assert fm-teardown.sh exits non-zero when HEAD not reachable from any remote-tracking ref and no --force
assert secondmate home path does not overlap with active FM_HOME or firstmate repo root
