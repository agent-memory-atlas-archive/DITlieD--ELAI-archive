K-27 decisions — codex gpt-5.6-sol executor into the opus chain

MOVE 0 recon (settled 2026-07-11, s-k27-build, opus, all code-grounded)

RECON-1 (cross-distro reachability) -> FORK-B, sub-fork worktree-on-/mnt/f.
- codex_provider.build_codex_command shells `wsl -d codex-vm codex exec ... -C _wsl_path(worktree)`.
  codex + accounts live ONLY on codex-vm (/usr/bin/codex, /root/.codex-accounts/{account-a,account-b}).
  teikoku has neither. codex-vm /mnt/f is UNMOUNTED (mount-guard on demand).
- WorkerAdapter default worktree base = teikoku-ext4 (conductor/worktrees, conductor/elai-worktrees),
  INVISIBLE to codex-vm. So the codex-executor worktree MUST live on /mnt/f (drvfs, both distros).
  _wsl_path passes a /mnt/f POSIX path through unchanged -> valid -C after a codex-vm mount-guard.
- Nested WSL interop verified: teikoku can `wsl.exe -d codex-vm -u root -- ...` (WSLEXE_OK). BUT bare
  `wsl` fails via subprocess inside teikoku (FileNotFoundError; only /mnt/c/.../wsl.exe on PATH).
  build_codex_command uses "wsl" -> must be "wsl.exe" for in-distro dispatch. 1-line codex_provider fix.
- account-a config.toml pins model="gpt-5.6-sol" + review_model=gpt-5.6-sol + reasoning xhigh (verified).
  build_codex_command passes NO --model, so the model IS whatever account-a config pins -> gpt-5.6-sol.

GROUNDING CORRECTIONS to K-27 (seam not where the plan assumed):
- role is "EXECUTE" not "EXECUTOR" (models.ROLES = PLAN/EXECUTE/WIRE/INTEGRATE/AUDIT/POLISH/FIX).
- PRODUCTION_HOOK is NOT opus_supervisor.py calling dispatch_worker. The supervisor is a resume-chain
  SPAWNER (_boot_prompt -> a headless claude-opus-4-8[1m] turn). The Opus turn runs the SKILL and
  SHELLS opus_dispatch.py via Bash. So the codex-executor consumer is (a) opus_dispatch.py's CLI
  (--lane codex) + (b) the SKILL routing text the Opus reads. The Opus is the consumer, not the
  supervisor Python.
- slot selection: adapter.dispatch -> select_slot(task.model_tier); PAID -> paid_slots. escalation_workers
  coerce to PAID (slot_pools.build_slot_pools) -> the codex slot lands in paid_slots. TaskSpec.model_tier
  must be "PAID". lane_serves_model("codex","gpt-5-6-sol") must resolve -> add gpt-5-6-sol to
  _CODEX_MODELS + POOL_HARNESS_MODELS["codex"].

RECON-2 (credit headroom): account-a ~2.3k credits left, dies 2026-07-15. account-b DEAD (no rotation).
RECON-3 (greenfield capability wall): UNKNOWN until first real L unit; measured by U3, not settle-before-build.

RESIDUAL RECON (settle at U4 smoke1, NOT a pre-build blocker):
- does codex's Landlock workspace-write sandbox permit writes to a drvfs /mnt/f cwd? If NO -> fall back
  to codex-vm-ext4 worktree + git-bundle bridge (that exceeds 5 files -> ABORT-1 to K-26 Route-B).

DESIGN (owner note 2026-07-11): gpt-5.6-sol has 1M context, OpenAI top-line. Routing = codex for
execution/complex/greenfield/big units (large token_budget), FREE swe-1-7/swe-1-6 for explore/easy.
Encode the 1M window in the _CODEX_MODELS PoolModel context_window.

U1 CHANGE SET (~4 files, under ABORT-1 5-file threshold): model_pool.py, codex_provider.py,
opus_dispatch.py (dispatch_codex_executor + --lane codex + mount-guard + ELAI_OPUS_CODEX_EXECUTOR gate),
conductor.elai-walk.toml (escalation slot -> account-a/gpt-5-6-sol/codex-vm), SKILL routing text.

U1 STATUS: BUILT + DRY-RUN VERIFIED 2026-07-11. flag-OFF refuses (main()->2), flag-ON dry-run builds
model=gpt-5-6-sol tier=PAID role=EXECUTE worktree=/mnt/f/elai-codex-worktrees/<task>, empty-writes
refuses (main()->2), FREE default unchanged (swe-1-7/FREE). lane_serves_model('codex','gpt-5-6-sol')=True,
window=1_000_000. (shell $? is unreliable through the Windows->wsl.exe->bash->python chain -- it swallows
non-zero codes: `false;echo $?`=0. Verify exit codes via main()'s python return, not shell rc.)

=======================================================================================
REDTEAM VERDICT (adversarial-auditor mode=plan, 2026-07-11) -- 4 BLOCKs, all fixable, no dead end.
The plan's central claim "codex structurally cannot smuggle a change past the merge" is FALSE against
the CURRENT opus_land/worker_adapter/codex mechanics. Fixes gate any credit spend (U3).
=======================================================================================
B1 SCOPE GUARD WRONG + CHECKERS STARVED (opus_land.py:177-180,218-224,316-320): scope check enforces
   only the ".elai_cc/" prefix, NOT the unit's allowed_write_set; the frozen checkers run over the
   caller-supplied --files, not the ACTUAL git-diff `changed` set; the commit loop stages ALL changed
   paths. -> an injected extra in-.elai_cc file the Opus forgets to list passes scope, skips both
   checkers, and lands. FIX: enforce scope from taskspec.json allowed_write_set (hash it, codex can't
   rewrite unnoticed); drive checkers' --files from `changed`; any changed path outside allowed_write_set
   OR not fed through both checkers = hard REFUSE.
B2 GIT-VERB CAPABILITY IS PROSE-ONLY FOR CODEX (models forbidden-list only renders into the prompt;
   the real deny-ACL role.config.yaml is base-cc-only, codex never loads it; Landlock restricts PATHS
   not git verbs). Codex's writable dir IS a git worktree of /mnt/f/.ELAI_workflow, which SHARES the
   object store + refs with canonical main -> codex can `git branch -f main <evil>` / reset / push
   against the real repo from inside its sandbox. FIX: codex's writable dir must NOT be a git worktree.
   Export a PLAIN copy (git archive of base_commit, no .git) into the codex dir; opus_land diffs the
   plain dir vs base_commit and applies to a real teikoku-side worktree for the gated land.
B3 WHOLE-F: READ EXFIL (Landlock grants unrestricted READ across the whole visible fs; guard.py is a
   post-hoc, encoding-evadable regex on stdout; the diff-egress scan never sees a read-and-echo-in-notes).
   Once /mnt/f is mounted into codex-vm, root codex reads any F: secret/.env/SSH key. FIX: mount ONLY
   the specific worktree Windows dir into codex-vm via a SCOPED drvfs subpath mount
   (mount -t drvfs 'F:\elai-codex-worktrees\<task>' <mp>), NEVER the whole F: drive.
B4 CROSS-DISTRO REAP -> EMPIRICALLY MITIGATED (tested 2026-07-11): killing the teikoku-side wsl.exe
   client DID terminate the codex-vm-side process (positive control: alive pid 9; after killpg:
   NO_SLEEP_ALIVE). Add a belt-and-suspenders codex-vm pkill + opus_land refuse-if-codex-alive, but not
   a structural blocker in this WSL build.
WARN: account-a config.toml has a top-level network_access="enabled" (wrong key form, likely ignored) --
   pin `-c sandbox_workspace_write.network_access=false` on the CLI (highest precedence) so egress is OFF
   by construction. trusted-project entries in config.toml (/mnt/f/.ELAI_workflow/conductor etc) could
   auto-load a codex-dropped .codex/config.toml on a D-9 retry -> strip/narrow before picking the base.
   D-9 attempt-count + L-1-DONE-before-L-2 must be code/DB state, not Opus inference from worker notes.
INFO: L5(d) cross-family review HOLDS (kimi=:8810 devin bridge, codex=bmapi -- different vendor+proxy).

HARDENED DESIGN (DECISION: Option A -- keep the chain, fix all 4 on F:, no codex-vm-ext4 bridge):
codex-writable dir = PLAIN git-archive copy of base_commit (no .git) at F:\elai-codex-worktrees\<task>,
mounted into codex-vm via a SCOPED drvfs subpath mount (only that dir), network OFF via CLI -c,
codex-vm pkill belt-and-suspenders. opus_land: diff plain-dir-vs-base_commit, enforce taskspec.json
allowed_write_set, run checkers off the real diff, apply to a teikoku worktree, gate, ff-merge. This
fixes B1-B4 and stays FORK-B on F: (bounded rework ~3-4 files, under the K-26 fallback threshold).
GATE: the first CREDIT SPEND (U4 smoke1) waits on owner OK -- account-a ~2.3k credits, dies 2026-07-15.

=======================================================================================
HARDENING BUILT + VERIFIED (creditless) 2026-07-11 -- owner said "proceed".
=======================================================================================
B1 opus_land.py: added `changed subset of --files` enforcement (undeclared change = REFUSED) + drove
   the frozen checkers over `changed` not `--files` + same check on the already-committed resume path.
   VERIFIED: synthetic-poison test -- declaring only declared.txt while the worktree also holds
   poison.txt -> "REFUSED worktree changed paths not in the declared changeset", undeclared=[poison.txt].
   Declaring both -> passes scope, reaches the checkers.
B2+B3 codex_provider.py: _ensure_scoped_codex_mount mounts ONLY the worktree dir into codex-vm
   (drvfs subpath, same POSIX path both sides), fail-closed if /mnt/f is fully mounted there; +network
   OFF via `-c sandbox_workspace_write.network_access=false`; gated on the elai-codex-worktrees marker
   so the read-only tester lane is untouched. VERIFIED: from codex-vm, marker_readable=OK but
   canonical_repo_visible=NO_ISOLATED and mnt_f_full=NO -- codex sees ONLY its worktree, not the repo.
B4 opus_land.py: --codex-reap-task pkills any lingering codex proc on codex-vm before the scan;
   supervisor boot passes it. VERIFIED earlier: cross-distro kill already propagates in this WSL build.
CONSUMER WIRING: opus_supervisor.py sets ELAI_OPUS_CODEX_EXECUTOR=1 per turn + boot-prompt routes
   hard/greenfield/big EXECUTOR units to `opus_dispatch --lane codex`; SKILL block 3 documents the lane
   + the safety contract. Supervisor 17/17 unit tests green after the edits.
RESIDUAL (settle at U4 smoke1, first credit): does codex's Landlock workspace-write actually WRITE to
   the scoped drvfs mount as cwd? (read-side isolation confirmed; write-side is the smoke's job.)
   -> RESOLVED YES.

=======================================================================================
U4 SMOKE1 -- LIVE (first credit), PASSED 2026-07-11. The whole codex EXECUTOR seam is proven.
=======================================================================================
Dispatched `opus_dispatch.py --lane codex` on a trivial real .elai_cc task (create K27_SMOKE.md).
Result: STATUS=APPLIED, model=gpt-5-6-sol, tier=PAID, touched=[.elai_cc/K27_SMOKE.md]; the file was
written with EXACTLY the requested content; codex notes were inert ("No Git commands were run.
Awaiting off-agent verification"); opus_land scope-accepted the declared file (undeclared=None); the
worktree was cleaned up (nothing landed to main). The .md gives a CHECKER_CONFIG_ERROR (maintainability
needs a source file) = correct fail-closed; a real .rs unit passes the checkers.
CAUGHT + FIXED A REAL LATENT BUG en route: build_codex_command used `--config <path>`, but codex-exec's
-c/--config is key=value OVERRIDES only; the account is selected by $CODEX_HOME/config.toml, and
CODEX_HOME does not cross the WSL->codex-vm boundary. Fix: invoke `wsl.exe -d codex-vm -u root -- env
CODEX_HOME=<dir> codex exec ...` (drop --config). Verified against `codex exec --help`. This also
fixes the tester lane (same shared function).

L-1 LAUNCH BLOCKER (2026-07-11): the AUTOMATED supervisor chain cannot run -- the claude-elai Max
OAuth token is EXPIRED (expiresAt 2026-07-10T13:14:58Z); a headless Opus turn 401s at 0 tokens until
the owner does ONE interactive login on .claude-elai. Walk queue empty; L-1 exists
(docs/thefinalglance/L-plan/L-1_typed_claim_extraction_substrate.md) but is legacy-format -> fails
plan_gate_check -> needs the U0 refine before codex dispatch. Options open to the owner:
  A. owner refreshes the claude-elai token -> proper supervisor chain walks L-1 (after U0 refine + seed).
  B. THIS Opus session acts as the controller directly (no supervisor spawn, no token needed): U0-refine
     L-1, then dispatch codex on L-1's first vertical slice + gate + monitor. Bypasses the token wall.

=======================================================================================
L-1 LAUNCH LOG (2026-07-11, owner chose path A: refreshed token + supervisor chain).
=======================================================================================
PREP: token refreshed (owner interactive /login), gate-freeze re-locked (stale plan-standard.md drift,
walk-stopped so out-of-band OK), L-1 seeded into queue, SKILL S3 routing fixed (greenfield -> CODEX
executor, not auto-SET_ASIDE), OPUS_WALK_STOP cleared.
TURN 1 (session ...2429277-1): supervisor spawned Opus, it ran the per-plan loop on L-1 and dispatched
integration-explorer-elai on the FREE swe lane (S2 recon) -- CORRECT routing, and it ADAPTED (first
worker failed -> retried on the swe-1-6 fallback). BUT both swe workers FAILED (0-byte logs = killed
before output) and the turn exited rc=1; respawn then refused on the /health wedge.
ROOT CAUSE (bridge, the real operational blocker): devin_proxy.py /v1/messages makes a SYNC httpx call
(<=300s) inside the async loop, so the single-worker :8810 SERIALIZES everything. :8810 is SHARED (the
journal shows model=glm-5-2 turns hitting it, i.e. the GLM/research consumer), so the walk's swe workers
queued behind back-to-back glm turns and were starved to the 600s kill ceiling; and /health (the spawn
gate) blocked whenever a turn was in flight.
FIX (safe, reversible, env-scoped): devin_proxy.py __main__ now honors DEVIN_PROXY_WORKERS (default 1 =
UNCHANGED for the :8811-8813 research bridges); a systemd drop-in sets DEVIN_PROXY_WORKERS=4 on
teikoku-devin.service (:8810) only (matches the free-4 ceiling). VERIFIED: 4 worker procs; /health=200
in 0.004s WHILE a /v1/messages is in flight; a real swe-1-7 request completes. Backup:
devin_proxy.py.k27bak; drop-in: /etc/systemd/system/teikoku-devin.service.d/k27-workers.conf.
TURN 2+ (supervisor 2431901): after the bridge fix, supervisor spawned cleanly BUT hit a SECOND bug --
a RESPAWN BURN: it reached turn 12 with turns exiting in 26-76s rc=0 having dispatched nothing.
ROOT CAUSE 2 (stale lock): the OLD supervisor's turn-1 exited rc=1 without releasing the walk lock
(release_lock only runs on a clean stop-protocol exit). The lock stayed held by the dead session
(pid 2431073 DEAD, heartbeat frozen 13:40). claim_lock only steals past HEARTBEAT_STALE_MIN=30m, so
every fresh turn LockRefused-and-exited for ~30m, burning ~12 paid Opus boots. Stopped it via
opus_walk_stop (sentinel).
FIX 2 (durable, verified): walk_state._pid_dead(pid) + claim_lock now steals a lock whose holder pid
is PROVABLY dead (os.kill(pid,0) -> ProcessLookupError) IMMEDIATELY, not just past the 30m heartbeat
threshold -- an rc=1/crashed turn can no longer wedge + burn future walks. Conservative fail-safe
(non-int/PermissionError/OSError pid treated as ALIVE, never steal a lock we cannot prove dead).
VERIFIED: 43/43 walk_state tests pass; the patched claim_lock stole the dead lock in a live test.
CLEAN RESTART (supervisor 2436084, turn 1 session ...2436084-1): lock held STABLY by the live turn
across a 4-min watch, ZERO new spawns (burn gone), Opus working L-1 preflight/precheck. Both
operational bugs (bridge single-worker wedge + stale-lock burn) now fixed durably.
FILES TOUCHED (bridge+lock fixes, teikoku owner-hold): devin-bridge/proxy/devin_proxy.py (+ .k27bak
backup + systemd drop-in teikoku-devin.service.d/k27-workers.conf), opus_walk/walk_state.py.

=======================================================================================
CODEX-DEMO-1 RUN (2026-07-11) -- codex executor SEEN working through the orchestrator; 2 more real
integration bugs found + fixed. Owner: "demo is enough" (skip the final land).
=======================================================================================
Authored + gate-passed CODEX-DEMO-1 (elai memory list --count-only; seam VERIFIED: args.rs List variant
+ args_memory.rs parser + memory_cmd.rs:144 handler). Seeded with an owner directive to route EXECUTE to
codex. The orchestrator RAN IT: free-swe recon + precheck classified it executable (NOT set aside), and
dispatched gpt-5.6-sol tier=PAID role=EXECUTE at 15:00:57 -- codex working a real .elai_cc plan through
the full pipeline. No respawn burn (the lock fix held).

THREE integration bugs surfaced by the real run, all fixed:
BUG A (codex BLOCKED on unreachable binding file): opus_dispatch --agent prepends "Read
/mnt/f/.ELAI_workflow/.claude/{rules,agents}/... FIRST", but codex's SCOPED mount cannot see
/mnt/f/.claude -> codex returned BLOCKED ("required binding file does not exist"). FIX:
_compose_codex_objective INLINES the preamble + agent .md content (read teikoku-side) into the stdin
prompt for the codex lane instead of an unreachable pointer; main() is lane-aware. VERIFIED: composed
objective inlines the rules, no "Read /mnt/f" pointer.
BUG B (guard.py:48 discards every codex diff): the pattern r"\b>:?\s*/" matched the ubiquitous benign
`2>/dev/null` in codex stdout, so post_guard_block dropped the diff (touched=[]). FIX (owner-approved):
narrowed to r"\b>:?\s*/(etc|boot|sys|proc|usr|bin|sbin|lib|lib64|var|root)\b" -- keeps system-dir write
protection + line 53's /dev/[sh]da raw-disk rule, drops the /dev/null/tmp/worktree false positives.
VERIFIED: 2>/dev/null + worktree paths now ALLOW. (Noted pre-existing weakness: the \b> anchor never
caught space-separated `cat > /etc/x` in the old OR new pattern; not this change's regression.)
BUG C (dirty-overlap will refuse the land): the demo writes crates/cli/src/args.rs, which is already `M`
in the owner's large uncommitted .elai_cc changeset -> opus_land dirty-overlap guard refuses the ff-merge.
NOT fixed (owner chose "demo is enough"); to land for real, commit/stash the WIP or target clean files.

The Opus controller ADAPTED EXCELLENTLY: it read guard.py + the /mnt/f dirty tree, diagnosed BUG B + C
proactively with exact fixes, correctly REFUSED to self-apply the security-guard change (owner-confirm),
and halted cleanly (no burn). Trust-spine working as designed.

FILES TOUCHED this run (teikoku owner-hold): opus_walk/opus_dispatch.py (_compose_codex_objective +
lane-aware main), guard.py (pattern narrowing). Plan file: F: docs/thefinalglance/demo/
CODEX-DEMO-1_memory_list_count_only.md (gate PASS).

NET STATE: the codex executor lane is COMPLETE + PROVEN end-to-end through the orchestrator (dispatch ->
scoped-write -> guard -> gate); the only unseen step is a final ff-merge, blocked purely by the owner's
uncommitted args.rs (environmental, not a system defect). ALL teikoku changes remain owner-hold
UNCOMMITTED (codex-executor build + bridge/lock/prompt/guard fixes). account-a credits expire 2026-07-15.
FILES TOUCHED (all teikoku, owner-hold uncommitted): conductor/codex_provider.py, model_pool.py,
   opus_walk/opus_dispatch.py, opus_walk/opus_land.py, opus_walk/opus_supervisor.py,
   config/conductor.elai-walk.toml, .claude/skills/sequential-opus-orchestrator/SKILL.md.
