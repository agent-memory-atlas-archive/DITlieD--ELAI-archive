# Historical orchestration guide

> Archived reference. Commands, model labels, file maps and claims below describe an older installation. Some files are omitted; read [the current archive guide](README.md) first. This text is not an instruction to operate your machine.

ORCHESTRATION SYSTEM GUIDE

Audience: an LLM about to modify the TEIKOKU walk/orchestration stack. Written 2026-07-25 from
direct inspection of the running system, not from the design docs. Where a claim is unverified it
says so. Read this before changing anything under conductor/; it will save you from the traps at
the bottom, all of which are real and all of which cost a night.

WHAT THE SYSTEM IS

A walk is an autonomous loop that drains a backlog of plan files by orchestrating LLM workers.
One ORCHESTRATOR (a hosted model, headless) reads a plan, decomposes it, dispatches WORKERS (cheaper
models) to write code in isolated git worktrees, then lands their diffs through a gated commit path.
The orchestrator never writes product code itself.

Three roles, and the whole architecture follows from keeping them apart:
  SUPERVISOR   a plain Python loop. Picks a provider, spawns one orchestrator turn, classifies the
               outcome, sleeps, repeats. No LLM. It is the thing that cannot get confused.
  ORCHESTRATOR a headless LLM session (one per turn, never resumed). Reads plans, decides routing,
               dispatches workers, calls the land spine. Runs in the MAIN checkout.
  WORKER       a headless LLM session in a throwaway git worktree, write-confined, told exactly
               which files it may touch. Produces a diff. Never commits.

THE PROCESS TREE (what you will actually see in ps)

  fallback_supervisor.py                     the loop, one process, long-lived
    └── base-cc --print (orchestrator turn)  one per turn, dies at turn end
          ├── memory_server.py / codegraph / gpt-bridge   its MCP servers
          └── opus_dispatch.py               one FRESH process per worker dispatch
                └── base-cc --print (worker) in a worktree, write-denied on the canonical repo

The "fresh process per dispatch" detail matters: edits to worker_adapter.py / opus_dispatch.py /
opus_land.py go live on the NEXT dispatch with no restart. Edits to fallback_supervisor.py do NOT
take effect until the supervisor restarts, because it is long-lived and already imported.

FILE MAP (conductor/, sizes as of 2026-07-25)

  fallback_walk/fallback_supervisor.py  536   THE LOOP. Provider ladder, cooldowns, failure classes.
  fallback_walk/macaron_adapter.py            Spawns a macaron/Novita orchestrator turn.
  fallback_walk/provider_adapters.py          Cursor-agent and Codex CLI transports.
  glm_walk/glm_supervisor.py            748   GLM turn spawn + preflight (autoseed, bridge health,
                                              disk guard, reconcile). The fallback supervisor calls
                                              INTO this for _prepare_work, so it is load-bearing
                                              even when the GLM provider is not selected.
  glm_walk/zcode_adapter.py                   ZCode app-server NDJSON transport (GLM-5.2 desktop).
  opus_walk/opus_supervisor.py          961   The original single-provider (Opus) loop.
  opus_walk/opus_dispatch.py            667   WORKER DISPATCH. Lanes, model ladder, prompt compose.
  opus_walk/opus_land.py                462   THE LAND SPINE. Gates, verifier, scope guard, merge.
  opus_walk/opus_preland_verify.py            The fresh-session pre-land verifier (P1.5).
  opus_walk/plan_selector.py            794   Picks the next plan; feeds autoseed.
  worker_adapter.py                    1065   Worktree lifecycle, diff capture, guards.
  free_api_worker.py                          Builds the base-cc worker command + the write-deny.
  worker_providers.py                         Per-backend ENV (which base_url + auth a turn uses).
  plan_salvage.py                             Persists refined plans back to the corpus (2026-07-25).
  novita_retry_proxy.py                       Local retry shim in front of Novita (2026-07-25).
  walk_monitor.py / walk_landing_watch.py     Monitoring (2026-07-25).

  opus_walk/ also holds ~30 one-off _m*/j68_*/_closeout_* scripts and JSON. These are ARCHAEOLOGY
  from past sessions, not part of the loop. Do not assume a file there is live; grep for its callers.

STATE (the four places truth lives)

  conductor/opus_walk/walk-state.json    THE walk's own state. Keys:
      queue              plan ids waiting (often 0-1; autoseed refills it)
      escalation_queue   80 entries: plans SET_ASIDE with a reason. This is where blocked work goes.
      dispositions       per-plan verdicts (set_aside / done) with evidence refs
      decisions          213 entries, append-only decision log
      lock               single-writer lease; a dead holder pid is stolen (K-27)
      cursor / budget / integration_smoke
  conductor/state/fallback-walk-state.json   provider cooldowns + turn history (last 100)
  conductor/state/control.db (sqlite)  the EVENT JOURNAL. Table `events`, columns:
      event_seq, event_id, task_id, event_type, payload_json, created_at, project_id, session_id
      This is the off-agent artifact you should reason from. Useful types:
      FALLBACK_TURN_START/DONE, OPUS_WORKER_DISPATCHED/DONE, OPUS_LAND_GATES_PASS,
      OPUS_LAND_GATE_FAIL, OPUS_LAND_VERIFIER_APPROVED/REJECTED/UNAVAILABLE, OPUS_LAND_MERGED,
      OPUS_LAND_PUSHED, GLM_WALK_AUTOSEED, WATCHDOG_FIRE, TASK_IMPORTED.
  the elai git repo    the actual product output.

THE PROVIDER LADDER (fallback_supervisor.py)

  PROVIDERS is an ORDERED TUPLE. select_provider() returns the first entry whose cooldown has
  expired, so PRIORITY IS TUPLE ORDER and you can re-prioritize purely by editing cooldown state
  in fallback-walk-state.json without touching source.

  Each entry: id, provider, model, context_window, context_verified, dispatch_cap, plan_cap.
  dispatch_cap/plan_cap become ELAI_WALK_DISPATCH_CAP/PLAN_CAP env for that turn and bound how much
  one turn may do before stopping at a unit boundary.

  classify_failure(result) reads error/stderr_tail/stdout_tail and returns:
      ok         rc 0
      quota      matched _QUOTA_PATTERNS  -> cool this provider, fall through to the next
      auth       matched _AUTH_PATTERNS   -> cool 1h
      cancelled  stop sentinel
      unexpected ANYTHING ELSE            -> HARD HALT of the entire walk

  The "unexpected" default is the sharpest edge in the system. A provider emitting a novel error
  string kills the walk. Two real instances: cursor-agent's real exhaustion string ("You're out of
  usage") matched nothing and hard-halted the ladder; Novita's streaming 400 did the same on
  2026-07-25. If you add a provider, you MUST enumerate its failure strings into _QUOTA_PATTERNS or
  it will fail closed in the loudest possible way.

THE WORKER LANES (opus_dispatch.py)

  --lane free   the devin bridge on :8810. DEFAULT_FREE_MODEL = glm-5-2 (owner ruling 2026-07-23).
                grok-4-5-high is DRAINED until 2026-08-08; passing it silently picks a dead lane.
  --lane codex  paid gpt-5.6-sol, default-OFF behind ELAI_OPUS_CODEX_EXECUTOR=1, scoped mount.
  Roles: RECON / PLAN / EXECUTE. The worktree name encodes it: opuswalk-<PLAN>-<ROLE>-<ts>-<hash>.

THE LAND SPINE (opus_land.py) -- read this before touching anything about commits

  Two invocations per unit, by design:
    1. --gates-only   the AUDIT leg. Runs the frozen checkers + your --gate-cmd list.
    2. no flag        the LAND leg. Re-runs gates, then verifier, then commits + ff-merges + pushes.

  Order inside the land leg:
    changed = _changed_paths(worktree)          git status --porcelain
    REFUSE if any path is outside SCOPE_PREFIX (".elai_cc/")
    P1.5 VERIFIER: a FRESH base-cc session that never saw the build judges spec+diff+gates,
                   answers APPROVE/REJECT. Fail-closed: reject OR unavailable OR unparseable
                   REFUSES the land.
    stage ONLY the declared paths (never add -A), refuse out-of-scope staged paths
    commit on the worktree branch, ff-merge into canonical main, push

  MANDATORY on every gate cargo command: CARGO_TARGET_DIR=$HOME/.cache/cargo-target-elai.
  A fresh target dir builds 13GB+; on 2026-07-15 that filled the disk and wedged the whole distro.

  --no-verifier requires owner sentinel conductor/ALLOW_NO_VERIFIER with a JSON body
  {timestamp, scope:{plan_id}} matching --plan-id (added 2026-07-25; body parse 2026-07-29).
  Empty/missing/wrong-scope is IGNORED; OPTOUT_REFUSED or OPTOUT_GRANTED is emitted.

THE PLAN CORPUS -- the single most confusing thing in this system

  551 plans live at /workspace/ELAI/docs/thefinalglance/<LETTER>-plan/<ID>_<slug>.md
  Registry: /workspace/ELAI/.elai_cc/progress/plan-registry.json (id, letter, path;
  paths are relative to the elai repo root).

  THE CORPUS IS GITIGNORED. The elai .gitignore is `*` with only .elai_cc/ un-ignored. Consequences
  that will bite you:
    - `git ls-files docs/thefinalglance` returns 0. The plans are NOT tracked.
    - a git worktree does NOT materialize them, so WORKERS CANNOT SEE PLAN FILES.
    - `git status --porcelain` never reports a plan edit, so a refined plan is invisible to the diff.
    - the land spine would refuse it anyway (outside .elai_cc/).
    - `git ls-files --others --exclude-standard` also hides them; you need --ignored + a pathspec,
      and `git add -N` REFUSES an ignored path without -f.

  How plans reach a worker anyway: the CONDUCTOR reads them (backlog_import.py parse_plan_file) in
  the main checkout and composes the objective. The orchestrator can also read them directly.
  How a REFINED plan gets persisted: plan_salvage.salvage_refined_plans copies it worktree ->
  canonical, by file copy, never through git. Before 2026-07-25 nothing did this and plan-refine
  work was silently discarded (verified: three turns reporting all 9 gate points PASS, 0-byte diffs).

  Plans must pass `python3 .claude/tools/plan_gate_check.py --file <plan>` before execution
  (refine-before-execute). 64 of 798 corpus files currently carry the required wargame blocks.

AUTOSEED

  On a drained queue, glm_supervisor._try_autoseed uses plan_selector to refill from the open-gaps
  deck (free-gated, no paid model). GLM_WALK_AUTOSEED=off restores halt-on-drain.
  AUTOSEED_MAX_PER_CHAIN=8 bounds the burn. Watch for: a walk that autoseeds a plan per turn and
  produces nothing is not "working through the backlog", it is spinning -- check whether the seeded
  plans have gate-passing plan files.

HOW TO DIAGNOSE A WALK (do these in order, they are cheap)

  1. Is it alive?      count processes whose argv ELEMENT ends with fallback_supervisor.py.
                       DO NOT `ps | grep fallback_supervisor` -- your own command line contains the
                       string and you will match yourself. This exact bug made a WALK_DOWN check
                       structurally unfireable. walk_monitor.procs() does it correctly.
  2. What is it doing? tail the history in state/fallback-walk-state.json; look at the last
                       turn_start/turn_done pair and the classification.
  3. Did work land?    git log in the elai repo + the OPUS_LAND_* events in control.db.
  4. Why did it fail?  FALLBACK_TURN_DONE carries stderr_tail. If that is EMPTY with rc=1, the
                       failure is in stdout -- reproduce the turn directly through the adapter and
                       read the result JSON.
  5. Worker detail:    conductor/artifacts/<worktree-name>/worker_stdout.log is the worker's own
                       result JSON, including its self-reported STATUS and NOTES. Its notes are
                       often the fastest route to the real cause.

TRAPS (every one of these is a real incident, not a hypothetical)

  T1 SELF-MATCHING PROCESS CHECKS. `ps | grep <script>` matches your own command. Match an argv
     element with endswith(). Same trap in `pkill -f <script>` -- it will kill your own shell.
  T2 THE 0-BYTE DIFF. If a worker reports DONE and patch.diff is 0 bytes, it edited an ignored path.
     Check `git ls-files --others --ignored --exclude-standard -- <path>` in its worktree.
  T3 MAKING IGNORED FILES VISIBLE TO THE DIFF IS NOT A FIX. It makes _changed_paths report an
     out-of-scope path, and the land spine then REFUSES THE WHOLE CHANGESET, code included. Persist
     ignored working state by copy, outside git.
  T4 EMPTY-ERROR 400s. Novita answers a rejected request with only a trace_id. Log the request SHAPE
     (key names, counts, sizes -- never the body, it holds the conversation) to diagnose.
  T5 A 429 IS TWO DIFFERENT FACTS. server_overload (capacity noise, retry in seconds) vs a real
     budget rejection (wait for x-ratelimit-reset-requests). Discriminate on the error type AND
     x-ratelimit-remaining-requests; a budget rejection cannot occur with headroom remaining.
     Conflating them mis-measures a rate limit as a concurrency ceiling.
  T6 THE GATE THE RUN CAN DISABLE IS NOT A GATE. --no-verifier was an ordinary CLI flag the
     orchestrator composed itself; a 730-insertion commit with a new dependency landed unreviewed in
     ONE second. Owner escape hatches belong in a sentinel FILE (like CODEX_FIRST), never a flag.
     Corollary: a fail-closed gate with no retry creates PRESSURE to bypass it. Give the transient
     failure a bounded retry, never the real verdict.
  T7 opus_land.py IS NOT IN .claude/gate-manifest.txt. The land spine is editable by the runs it
     judges. Freezing it is an open decision.
  T8 WIRING GATES DO NOT CHECK SEMANTICS. production_caller_check proves a call EXISTS. It cannot
     tell that the call passes hardcoded empty vectors, or compares prose sentences to identifiers.
     Both shipped in July and both passed every mechanical gate. A worker optimizing against wiring
     gates converges on real call sites carrying meaningless data.
  T9 THE FRONTEND MOUNT GATE EXISTS AND IS NOT IN THE LAND SET. `.claude/tools/svelte_mount_
     reachability.py` correctly returns BLOCK on an orphan component; a commit shipping 211 lines of
     unmounted Svelte was approved because nothing ran it.
  T10 DISK. Always CARGO_TARGET_DIR=$HOME/.cache/cargo-target-elai. Orphaned per-worktree targets
     are ~17GB each; check `du -sh conductor/elai-worktrees/*/.elai_cc/target` when space vanishes.

UPGRADE SURFACE (where the leverage is, roughly ordered)

  1. SEMANTIC VERIFICATION (T8) is the biggest gap. Everything mechanical is gated well; nothing
     checks that wired code MEANS anything. Candidate: require each unit's ACCEPTANCE block to name
     an assertion that fails on a hardcoded-empty implementation, and have the verifier check it.
  2. GATE-SET COMPLETENESS (T9). The land spine runs a fixed checker list. Frontend/mount/taste
     gates exist in the repo and are not wired in. An inventory of `.claude/tools/*check*.py` vs
     what opus_land actually invokes would likely find more.
  3. FREEZE THE LAND SPINE (T7) so a run cannot edit its own evaluator. This is wiring-enforcement
     R8 applied to the orchestration layer, which currently exempts itself.
  4. FAILURE-CLASS COVERAGE. classify_failure defaults to a full-walk halt. A provider-specific
     pattern table (each provider declaring its own quota/auth strings) would remove a whole class
     of 3am deaths.
  5. PLAN-CORPUS ERGONOMICS. The untracked-corpus design is deliberate but every consumer has to
     work around it independently. A single accessor module ("read plan, write plan") would stop
     the next component from rediscovering T2/T3.
  6. TURN OBSERVABILITY. An orchestrator turn emits nothing between dispatches; a 66-minute silent
     turn is indistinguishable from a wedged one. Cheap fix: a heartbeat event per orchestrator
     tool-call batch.

CONVENTIONS THAT ARE LOAD-BEARING

  - Workers are git-read-only and write-confined (free_api_worker deny_settings). Do not relax this.
    If a worker needs to persist something outside its worktree, the CONDUCTOR does the write.
  - The executing agent never renders its own verdict. Self-run checkers are a courtesy; the
    auditor/verifier re-run off-agent is the authority.
  - One commit per drained plan, pushed immediately, authored by opus_land -- never by an agent's
    own git.
  - Owner toggles are FILES (CODEX_FIRST, OWNER_DIRECTIVE.md, FALLBACK_WALK_STOP, ALLOW_NO_VERIFIER)
    so they can be flipped without a restart and cannot be synthesized by an agent.

UNVERIFIED / OPEN QUESTIONS (do not treat as fact)

  - Why J-64's worker reached the canonical plan path despite the Edit/Write deny is UNRESOLVED.
    Most likely a Bash write, which the tool-level deny does not cover. If true, the worker write
    boundary has a hole worth closing.
  - The ~30 _m*/j68_*/_closeout_* scripts in opus_walk/ are presumed dead; not verified.
  - plan_selector.py (794 lines) was not read in depth; its selection heuristics are unexamined here.
  - Whether the 63 pre-2026-07-25 plan refinements persisted via the same fragile path, or via the
    orchestrator writing canonical directly, is not established.
