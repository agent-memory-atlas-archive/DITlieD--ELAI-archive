TEIKOKU P4-B exploration: the DEVIN CONDUCTOR (the F:-root conductor)

Task: p4-explore-devin-conductor. Read-only exploration. Decision target: should TEIKOKU P4-B
REUSE/PORT this conductor's autonomous-finish loop + Devin worker lane instead of building from
scratch. This conductor is the ANCESTOR that ~/teikoku/conductor/ was copied from at TEIKOKU step 0.1.

Everything below is grounded in files actually read on F: (paths + line refs given). Claims about
TEIKOKU's descendant (~/teikoku/conductor/, in the WSL `teikoku` distro) are NOT file-verified here
(that tree is not reachable from F:); they are taken from the task brief and are labelled
[per-brief, unverified] where they appear.

================================================================================
1. WHAT IT IS
================================================================================

The "devin conductor" at F:\.ELAI_workflow\conductor\ is a deterministic, native-Windows Python
daemon (~0 LLM cost) that drives N `devin.exe -p` worker processes on FREE Cognition-hosted models
to autonomously FINISH ELAI. It implements the "Autonomous Finish System (the Conductor) — DEVIN CLI /
FREE-MODEL variant" spec at
F:\.ELAI_workflow\workflow\reasoning\AUTONOMOUS-FINISH-SYSTEM-DEVIN-2026-06-18.md.

Its job (spec lines 5-8, AGENTS.md 1-8): take the ~412 ELAI plan files (B..K letter-plans + base
phases 0-4) and run each through implement -> wire -> integrate -> test -> audit -> polish, in
repeated waves, with NO Claude/Opus as the orchestrator and ~0 paid tokens for the bulk work. It is
workflow TOOLING that operates ON F:\.ELAI_workflow\.elai_cc; it lives OUTSIDE .elai_cc on purpose and
is gitignored (only .elai_cc is tracked, so most of it is never committed — it persists on disk).

THE ONE REFRAME (spec §0): the orchestrator must NOT be an LLM. Replace the hand-driven Opus/orchestrate
FSM with a small Python control loop that costs ~0 tokens; the LLM workers are stateless and
disposable. This directly answers the three stated ELAI-finish failures:
  - "need ~100 opus to orchestrate" -> orchestration is code; scale is config; bulk workers are FREE.
  - "agents leave things wire-dark"  -> a worker CANNOT mark its own work done. The frozen, off-agent
                                        gate stack is the ONLY door to DONE; missing evidence = FAIL.
  - "some finish, some don't"        -> durable task graph + leases + bounded retries + artifact-
                                        grounded state; every task has one terminal state and survives
                                        a crash.

WHAT "DONE" MEANS FOR IT (spec §5, gate_runner.py): "done" is SYSTEM-LEVEL, never a worker self-report.
A task is DONE only when an INDEPENDENT, off-agent verifier accepts its artifact: the gate stack runs
against the worker's isolated git worktree and the task transitions VERIFIED only on overall_verdict
== "PASS" (conductor_loop.py:275). The whole milestone is done only when the graph is closed AND the
global gate stack is green on merged head AND K consecutive "dry" audit/polish passes happen
(loop_until_dry, conductor_loop.py:554-629). "I fixed it" is never a state transition; "verifier V
accepted artifact A for task T" is the only one.

LINEAGE NOTE: this is the "old devin" conductor but it was itself ADAPTED from an earlier Codex-CLI
variant (spec line 4: "harness swapped from Codex CLI -> Devin CLI"). The Codex lineage is still
visible in the code: worker_adapter.py carries BOTH build_devin_command (lines 408-433) AND
build_codex_command (451-496, a WSL `codex exec` path with proxy config). So the F: conductor is a
two-generation artifact: Codex-variant design, retrofitted to drive Devin's free models, with the
Codex path left in as a dormant second harness.

================================================================================
2. ARCHITECTURE
================================================================================

5 planes (spec §2): 4 state planes + 1 cognition plane. The conductor owns the 4 state planes; the
LLM is pushed entirely into disposable workers.

  CONDUCTOR (deterministic Python, native Windows process) -> talks to ->
    [CONTROL DB]  SQLite WAL, AUTHORITATIVE current state (task graph, status, leases, verdicts)
    [EVENT LOG]   append-only events table, replayable HISTORY
    [KNOWLEDGE GRAPH] elai.map / codegraph (retrieval RELEVANCE only, never arbitrates "done")
    [ARTIFACT STORE] per-task worktree diffs, ATIF exports, gate outputs, epoch summaries
  WORKERS = N `devin -p` subprocesses, each in its own git worktree, stateless: TaskSpec in (via
    --prompt-file), patch + ATIF export out, process exits. Roles (PLAN/EXECUTE/WIRE/INTEGRATE/AUDIT/
    POLISH) are TASK TYPES time-multiplexed across slots, not dedicated long-lived agents.

Truth ownership is deliberately split so no single store carries all truth: Control DB = current
commitments; Event log = history/forensics; Gate stack = correctness (off-agent, fail-closed);
Knowledge graph = relevance only (rebuildable).

THE ORCHESTRATION MODEL — two interchangeable backbones behind one config flag (`backbone`,
conductor_config.py:130-132, default "sqlite"):

  (A) DEFAULT = SQLite in-process loop (conductor_loop.py). NOT a workflow engine — a plain Python
      driver over the control DB. The per-task loop run_task_loop() (conductor_loop.py:67-382) walks
      SELECT -> CONTEXT -> EXECUTE -> VERIFY -> INTEGRATE -> RECORD:
        SELECT    reclaim expired leases, enforce retry tripwires, acquire a lease for the task.
        CONTEXT   build a bounded context packet (context_builder.build_context_packet_for_task).
        EXECUTE   record attempt start; WorkerAdapter.dispatch spawns `devin -p` in the worktree.
        VERIFY    scheduler.record_worker_result classifies the run; if next_status=="VERIFYING",
                  gate_runner.run_gates runs the frozen off-agent gate stack against the worktree.
        INTEGRATE on PASS, SerializedIntegrator merges the worktree branch into main (serialized,
                  one at a time even with N workers).
        RECORD    append LOOP_STAGE events; verdict + gate artifacts persisted.
      run_wave_loop() (385-501) drives a whole wave: it calls scheduler.schedule_frontier in a batch
      loop and runs up to worker_cap tasks, using a ThreadPoolExecutor when >1 worker
      (conductor_loop.py:462-479). run_backlog_waves() (504-551) runs the wave sequence; loop_until_dry()
      (554-629) repeats AUDIT+POLISH until K dry passes; _compact_epoch (698-717) writes epoch summaries
      so history never grows unbounded.

  (B) OPTIONAL = Temporal.io durable backbone (temporal_backbone.py), enabled by backbone="temporal".
      THIS IS A REAL TEMPORAL.IO WORKFLOW ENGINE, not a bespoke queue: it imports `temporalio`
      (activity, workflow, Client, Worker, RetryPolicy — temporal_backbone.py:23-27) and defines
      @workflow.defn classes RunTaskWorkflow / RunWaveWorkflow / LoopUntilDryWorkflow / BacklogWorkflow
      plus @activity.defn activities (schedule_frontier, lease_task, dispatch_worker, run_gates,
      integrate_verified_task, record_event, compact_epoch, update_task_status). The activities are
      THIN WRAPPERS around the same deterministic SQLite modules — i.e. Temporal wraps the same gate
      logic and the same control DB; it adds durable execution + visibility + automatic activity
      retries (RetryPolicy max 3, temporal_backbone.py:416-421), nothing else. It is tested against
      the time-skipping in-memory Temporal test server (tests/test_temporal_backbone.py:7,100), 3
      passing tests, with a FakeWorkerProvider so no real Devin CLI is spawned. Temporal CLI 1.7.2 +
      temporalio 1.29.0 are installed (handoff conductor-build.md:5,149-151). This is spec §7 Option B
      ("graduate to LangGraph+Temporal") realised as an opt-in; SQLite remains the default and ground
      truth.

THE SCHEDULER (scheduler.py) is a DependencyFrontierScheduler, NOT a plain FIFO queue. schedule_frontier
(scheduler.py:71-121): refresh model cooldowns, reclaim expired leases, enforce retry tripwires, check
the control plane (pause/drain/emergency-stop), then lease up to a cap that is the MIN of (requested
limit, worker_cap, free-model-concurrency headroom). A task is only "ready" when all its deps are
integrated to main (dependency frontier), so the graph topology — not arrival order — drives selection.
record_worker_result (123-339) is the disposition brain: it distinguishes rate-limit/quota (refund the
attempt, cooldown the model, re-queue READY) from a transient provider 5xx (same) from a genuine gate
FAIL (NEEDS_REVIEW / FAILED by budget) from an AMBIGUOUS mutation-after-disconnect (held as NEEDS_REVIEW,
never auto-replayed). That rate-limit-vs-failure split is the single biggest piece of "not a plain
queue" logic.

HOW IT DIFFERS STRUCTURALLY FROM A PLAIN QUEUE: (a) dependency-frontier readiness, not FIFO; (b) leases
with TTL + crash-safe reclaim, not at-most-once dequeue; (c) the off-agent gate is the state-transition
authority, so "consumed" != "done"; (d) serialized integration with post-merge re-gate; (e) bounded
retries + per-model cooldowns + a global free-model concurrency cap; (f) a control plane
(control_plane.ControlManager) with pause/drain/emergency-stop/cancel + a localhost dashboard
(conductor.py:439-440, dashboard subcommand); (g) epoch compaction for unbounded-history protection.

CLI SURFACE (conductor.py build_parser, 458-637): doctor, db init/check, db-check, events replay,
recovery scan, budget check, worker validate-models, context build/check, gate run, schedule frontier,
integrate task, run-once, run-wave, run-backlog, loop-until-dry, control
(status/pause/resume/drain/undrain/emergency-stop/clear-emergency/dry-run/cancel), dashboard,
temporal-worker. run-wave/run-backlog/loop-until-dry transparently dispatch to the Temporal backbone
when backbone=="temporal" (conductor.py:338-406).

================================================================================
3. DEVIN WORKER INTEGRATION
================================================================================

The worker lane is worker_adapter.py. It dispatches to Devin via the CLI binary DIRECTLY — there is NO
"devin bridge" module involved here; it is plain subprocess against devin.exe.

  Binary (pinned): DEFAULT_DEVIN_BIN = C:\Users\user\AppData\Local\devin\cli\bin\devin.exe
  (worker_adapter.py:26). Confirmed live as devin 2026.7.19 (handoff conductor-build.md:365).

  Free model pool: FREE_MODEL_POOL = ["kimi-k2.7", "glm-5.2", "swe-1.6"] (worker_adapter.py:32);
  PAID_MODEL_POOL = ["opus"] (line 33). Model strings use DOTS not dashes (handoff:366). The
  config example wires w1=kimi-k2.7, w2=glm-5.2 (FREE), esc=opus (PAID) (conductor.toml.example:46-68).

  THE WORKER-DISPATCH PATH (the exact command, build_devin_command, worker_adapter.py:408-433):
    command = [ devin_bin, "-p",
                "--model", <model>,
                "--permission-mode", "dangerous",        # unattended: auto-approve all tools
                "--config", <role.config.yaml>,          # per-task system instructions + Read/Write
                                                          # permission scopes + denylist
                "--prompt-file", <taskspec.txt>,         # the rendered TaskSpec
                "--export", <result.atif.json>,          # machine-readable TaskResult (ATIF v1.7)
                "--respect-workspace-trust", "false" ]   # non-interactive
    (--sandbox is inserted only if run_sandbox/run_devin_sandbox is on; it is a no-op on native
     Windows, so containment is the worktree boundary + the --config Write() scopes + the frozen gate
     + the git-read-only preamble — NOT an OS sandbox. config default run_devin_sandbox=false.)

  The full dispatch (WorkerAdapter.dispatch, 190-300): create/locate a git worktree off base_commit
  (_prepare_worktree, 320-356, `git worktree add -b <branch> <wt> <base_commit>`), render the TaskSpec
  + a bounded context packet + the role config, then spawn via the WorkerProvider
  (SubprocessWorkerProvider, 73-132 — CREATE_NEW_PROCESS_GROUP on Windows so it can CTRL_BREAK-cancel).
  On exit it reads `git status/diff` for touched_files + the diff artifact, parses the ATIF export for
  notes (treated as ADVISORY only), classifies rate-limit (classify_rate_limit, 741-758: 429/quota/
  throttle/exhausted/concurrency-limit tokens) and transient-provider errors (524/cloudflare/SSE-idle/
  econnreset, 761-788), checks scope_violations against allowed_write_set, and returns a TaskResult
  with a status_hint (RATE_LIMITED / TRANSIENT_PROVIDER_ERROR / NEEDS_REVIEW / BLOCKED / FAILED /
  APPLIED / NO_CHANGE — _status_hint, 675-694). The status_hint is a HINT; the gate is the authority.

  The role prompt is loaded from F:\.ELAI_workflow\.claude\agents\<role>.md (_load_role_prompt,
  562-576) and injected as the Devin --config `system_instructions` block, with the allowed_write_set
  expanded into Read()/Write() allow rules and a hard denylist (git reset/checkout/clean/push/commit,
  Write(.claude/**), Write(gate-manifest*), Read(*credential*/*secret*/*.env) — render_role_config,
  369-406). So the Devin worker inherits the same ELAI role contract it had under Claude Code, and is
  physically denied the gate files and destructive git.

  Did it use devin.exe or "the bridge"? devin.exe DIRECTLY (subprocess `devin -p`). The Codex sibling
  path (build_codex_command, 451-496) shells `wsl -d <distro> codex exec --ephemeral ...` against a
  proxy config — that is the dormant prior-generation harness, not used by the default Devin config.
  No gpt-bridge / devin-bridge MCP is referenced anywhere in the conductor.

  THE "MAX 3 PARALLEL FREE MODELS, PREFER KIMI" ENVELOPE lives in TWO places:
   - PREFER KIMI / fallback order: FREE_MODEL_POOL ordered [kimi-k2.7, glm-5.2, swe-1.6]
     (worker_adapter.py:32); the config wires kimi-k2.7 as primary slot w1, glm-5.2 as w2; swe-1.6 is
     deprioritised to cooldown-reroute only (handoff conductor-build.md:35,76, USER DECISION:104-105).
   - MAX PARALLEL: the empirical account ceiling is 2 sustained free-model `devin -p` workers (3-4 is
     the hard rate-limit edge; 4 total tripped a 3-hour ban — handoff:16-22,30-32). Enforced by
     free_model_concurrency_cap=2 in DependencyFrontierScheduler (scheduler.py:49,68,95-97;
     count_active_free_model_leases) AND worker_cap=2 (conductor.toml.example:28; config default
     bumped 1->2, conductor_config.py:111). The cap loop in schedule_frontier breaks once free_acquired
     hits the free headroom (scheduler.py:100-108). NOTE the brief's "max 3" is the spec's starting
     suggestion; the MEASURED, shipped envelope on this account is 2.

================================================================================
4. VS TEIKOKU's CONDUCTOR (the descendant)
================================================================================

TEIKOKU's conductor (~/teikoku/conductor/, WSL `teikoku` distro) was COPIED from this F: tree at step
0.1 and generalised in P1-P3. I could not read that tree (not on F:), so the descendant-side specifics
below are [per-brief, unverified]; the ancestor-side facts are file-grounded.

WHAT TEIKOKU ADDED (generalisation of the single-project ancestor) [per-brief, unverified]:
  - pack-aware project_resolver / scheduler / leases / gate_packs — i.e. the engine can target MULTIPLE
    projects ("packs"), each with its own gate set. The ancestor is HARD single-project: repo_root is
    ONE path (.elai_cc; conductor_config.py:143), the backlog importer hardcodes
    F:\.ELAI_workflow\docs + the `thefinalglance` letter-plan dirs + the s-plan-audit-2026-06-04\FINISH
    punch lists (backlog_import.py:23-25,203-265), and gate_runner hardcodes .elai_cc + elai.map +
    docs/thefinalglance plan-file resolution (gate_runner.py:27-30,168-308,573-583). There is NO "pack"
    abstraction anywhere in the ancestor (grep for pack/project_resolver/gate_pack across conductor/*.py
    = zero conceptual hits). So pack-awareness is a genuine TEIKOKU ADD.
  - the P4-A comms hub: wake / mailbox / spawn. The ancestor has none of these tokens. Its
    "comms" are the SQLite event log + the control_plane flags + the localhost dashboard; there is no
    inter-agent mailbox or wake primitive. Genuine ADD.
  - the P4-B headless_runner / free_api_worker. [per-brief] TEIKOKU is adding a headless engine runner
    plus a free-model worker that talks to a model API directly. The ancestor's worker is the
    Devin-CLI subprocess adapter (worker_adapter.py); it is NOT an HTTP/API worker — it drives a local
    CLI binary. So free_api_worker (a litellm/HTTP-style worker) would be a NEW lane the ancestor lacks.
    (The ancestor's escalation lane is also CLI: --model opus on devin.exe, worker_adapter.py:33.)

WHAT THE DEVIN CONDUCTOR HAS THAT TEIKOKU DROPPED / never carried [needs descendant confirmation]:
  - THE DEVIN WORKER INTEGRATION itself. The whole worker_adapter.py Devin-CLI lane (the exact
    `devin -p` command, the ATIF parse, the rate-limit/quota classifier tuned to Devin's free tier, the
    free-model pool + cooldown router, the role.config.yaml injection) is the ancestor's load-bearing
    asset. If TEIKOKU P4-B is building a free_api_worker instead, it has DROPPED the Devin lane (or
    never carried it) and is rebuilding the worker plane against a different harness.
  - THE AUTONOMOUS-FINISH LOOP wired to ELAI's plan corpus. The ancestor's backlog_import + wave
    sequence + loop-until-dry are tuned to the exact ELAI plan grammar (letter-plans, FINISH tables,
    base-phase .txt). [per-brief] TEIKOKU generalised the scheduler/leases but it is unclear whether the
    full wave/loop-until-dry/epoch machinery and the ELAI-specific backlog importer came across or were
    re-scoped per-pack.
  - THE TEMPORAL.IO BACKBONE (temporal_backbone.py). [needs confirmation] If TEIKOKU forked before
    Step 7 (the Temporal work is dated 2026-06-19, AFTER the typical 0.1 copy point), the descendant may
    have only the SQLite backbone and lack the durable-execution option entirely.
  - The dormant Codex harness path (build_codex_command). Almost certainly irrelevant to carry forward,
    but it is ancestor surface TEIKOKU may or may not have inherited.

WHERE THEY CONFLICT (the real friction for a port):
  - SINGLE-PROJECT vs PACK-AWARE is the structural conflict. The ancestor bakes ".elai_cc + ELAI plan
    paths + elai.map" into config defaults AND into gate_runner's command construction AND into
    backlog_import's corpus walk. TEIKOKU's pack model needs all three parameterised per-pack. The
    config layer (ConductorConfig, conductor_config.py) is already path-driven and would extend cleanly,
    but gate_runner.py:168-308 and backlog_import.py:203-265 contain ELAI-specific hardcoding that
    fights a multi-pack model.
  - WORKER HARNESS: Devin-CLI subprocess (ancestor) vs free_api_worker [per-brief] (descendant P4-B).
    These are different worker planes. The TaskSpec/TaskResult contract (models.py) and the gate stack
    are harness-agnostic and reusable, but the dispatch path is not.
  - PLAN SCHEMA: the ancestor's TaskSpec is rendered from ELAI WIRING_CONTRACT / DETERMINISTIC_CHECK /
    ACCEPTANCE blocks (spec §1, §3); the gate stack resolves plan files by ELAI naming
    (gate_runner.resolve_plan_file). TEIKOKU's pack plan schema must map onto TaskSpec + supply its own
    plan-file resolution, or the wiring_contract_check / spec_link_check gates will not find anything.

================================================================================
5. RELEVANCE TO TEIKOKU P4-B (port-vs-build verdict)
================================================================================

P4-B (headless autonomous engine + free-model worker provider) is, structurally, exactly what this
conductor IS: a headless deterministic engine + a free-model worker lane + an off-agent gate as the
door to DONE. So the question is well-posed and the answer is PORT, not from-scratch — but with a
clean split between the ~80% that is harness/project-agnostic and the ~20% that is ELAI-coupled.

WHAT PORTS ALMOST AS-IS (the durable, hard-won core; reuse these):
  - The control plane + state model: conductor_db (SQLite WAL control DB), leases.py (TTL leases,
    crash-safe reclaim, mutation-evidence inspection), budgets.py (retry caps, daily caps, per-model
    cooldowns), scheduler.py (dependency-frontier + the rate-limit-vs-gate-fail disposition brain +
    free-model concurrency cap), integrator.py (serialized merge + post-merge re-gate), recovery.py,
    control_plane.py (pause/drain/emergency-stop/cancel), events.py (replayable log), the epoch
    compaction. This is the part that makes it "not a queue" and is the bulk of the value.
  - The per-task loop shape SELECT->CONTEXT->EXECUTE->VERIFY->INTEGRATE->RECORD (conductor_loop.py) and
    the wave / loop-until-dry / backlog drivers — the control flow is project-agnostic; only the inputs
    are ELAI-shaped.
  - The Temporal backbone (temporal_backbone.py) if durable execution / horizontal scale is wanted —
    it already wraps the same modules and is tested.
  - The frozen off-agent gate DISCIPLINE (gate_runner.run_gates structure, the gate_freeze_check-first
    short-circuit, missing-evidence=FAIL, BLOCK/FAIL/ERROR/TIMEOUT mapping). The DISCIPLINE ports; the
    specific gate COMMANDS are ELAI tools and must be swapped per-pack.

WHAT MUST ADJUST TO FIT TEIKOKU's PACK-AWARE ENGINE + PLAN SCHEMA (the ~20%):
  1. PROJECT/PACK PARAMETERISATION. Lift the ELAI hardcoding out of three files: conductor_config.py
     defaults (repo_root, plan_corpus_root, finish_dir all default to ELAI paths), gate_runner.py
     (.elai_cc, elai.map, docs/thefinalglance plan resolution, the specific gate command list), and
     backlog_import.py (the letter-plan + FINISH + base-phase corpus walk). TEIKOKU's project_resolver
     + gate_packs are precisely the abstraction this needs; the work is threading a pack handle through
     these three seams.
  2. THE BACKLOG IMPORTER. backlog_import.py is 100% ELAI-plan-grammar specific. TEIKOKU must supply a
     pack-specific importer that maps its plan schema onto the same control-DB task/dep rows
     (task_id, deps, scope, acceptance, deterministic_check). The DB schema itself is reusable.
  3. THE GATE COMMAND SET. build_gate_commands (gate_runner.py:168-308) hardcodes ELAI's verifier
     scripts (production_caller_check / python_wire_check / maintainability_check /
     traced_test_assert_check / wiring_contract_check / spec_link_check / cargo). TEIKOKU's gate_packs
     should supply the per-pack command list; the runner harness (execute/record/verdict-map) is
     reusable verbatim.
  4. THE WORKER HARNESS. If P4-B's free-model provider is an HTTP/API worker (free_api_worker), it slots
     in as a new WorkerProvider behind the existing Protocol (worker_adapter.WorkerProvider, lines
     58-71) OR as a sibling to build_devin_command. The TaskSpec/TaskResult contract (models.py), the
     worktree lifecycle, the rate-limit classifier, and the scope-violation check are all reusable; only
     the spawn/parse path changes. NB: keeping the Devin-CLI lane as ONE provider and adding the API
     worker as a SECOND is cheaper than discarding Devin — the disposition brain already understands
     both rate-limit and transient-provider semantics.
  5. ROLE PROMPTS. _load_role_prompt reads F:\.ELAI_workflow\.claude\agents\<role>.md; TEIKOKU must
     point this at its own role-prompt source (or per-pack).

PORT-VS-BUILD READ (concrete): PORT. The expensive, correctness-critical machinery — durable task
graph, leases, the rate-limit-vs-failure disposition, serialized integration, fail-closed off-agent
gating, epoch compaction, AND a working Temporal option — is already built, unit-tested (handoff:
68-69 SQLite + Temporal tests pass; gate smoke proves wire-dark BLOCK / wired PASS for both Rust and
Python, gate_runner.run_smoke_test), and battle-shaped against real free-tier rate limits. Rebuilding
that from scratch in P4-B would re-derive months of disposition/edge-case logic (the
ambiguous-mutation-after-disconnect handling alone, scheduler.py:197-314, is subtle and load-bearing).
The genuinely NEW P4-B surface is: (a) the pack abstraction over config + gates + backlog, which
TEIKOKU is already adding in its descendant, and (b) the free_api_worker, which is one new
WorkerProvider behind an existing interface. Both are additive at clean seams. Risk in a from-scratch
build is HIGH (re-deriving the disposition brain); risk in a port is the merge friction between the
ancestor's ELAI hardcoding and the descendant's pack model — bounded to three files. Recommend ADOPT
the ancestor's control/loop/gate/integration core into TEIKOKU's pack-aware shell, add free_api_worker
as a provider, and keep (or port) the Devin lane as a second free provider rather than deleting it.

CAVEAT: the two trees have already DIVERGED (TEIKOKU generalised P1-P3 + added P4-A wake/mailbox/spawn).
A port is therefore a RE-CONVERGENCE / cherry-pick of any ancestor pieces the descendant dropped (most
likely: the Devin worker lane, the ELAI backlog importer, possibly the Temporal backbone if TEIKOKU
forked pre-Step-7), NOT a clean copy. The controller must diff ~/teikoku/conductor/ against this tree
file-by-file before porting — that diff is the unverified gap in this report.

================================================================================
6. RELATION TO THE CODEX CONDUCTOR + THE DEVIN BRIDGE
================================================================================

  - CODEX CONDUCTOR: this devin conductor is the DIRECT DESCENDANT of a Codex-CLI conductor. The spec
    is explicit (line 4: "adapted from the original 5-report design; harness swapped from Codex CLI ->
    Devin CLI"; lines 14-16 contrast worker=Codex-CLI/OpenAI-Responses/gpt-5.5 vs worker=Devin-CLI/free
    models). The Codex harness is still present in code but DORMANT: worker_adapter.build_codex_command
    (451-496) shells `wsl -d <distro> codex exec --ephemeral --json --sandbox <mode> ...` against a
    per-account codex_home/config.toml, with _worker_env setting CODEX_HOME (605-610) and _wsl_path
    translating F:\ paths to /mnt/ (820-826). _build_worker_command branches on slot.harness=="codex"
    (444-449). The default config uses harness="devin-cli" everywhere, so the Codex path is reachable
    only if a worker slot sets harness="codex" — effectively a legacy fallback. KEY DIFFERENCE the spec
    sells: the Codex variant required WSL+Docker for a dangerous proxy API (base_url, ZDR,
    network_access); the Devin variant needs NONE of that (Devin's own auth + hosted free models), so it
    runs as ordinary Windows processes (spec lines 20-26).

  - DEVIN BRIDGE: NOT used here, and not referenced from the conductor at all. This conductor talks to
    Devin by spawning the devin.exe CLI directly (subprocess `devin -p`, worker_adapter.py:408-433).
    There is no MCP "devin bridge" in the dispatch path (the only bridge token in conductor/*.py is
    "PromptBridge" inside an unrelated .elai_cc test fixture under test_wt/). The repo's gpt-bridge MCP
    (a separate ChatGPT/Grok second-opinion tool) is unrelated to this worker lane. So for TEIKOKU: the
    worker-dispatch precedent here is "drive a local agent CLI as a subprocess", not "call a bridge MCP".

================================================================================
FILES READ (verification)
================================================================================
F:\.ELAI_workflow\conductor\conductor.py (CLI, 648 ln)
F:\.ELAI_workflow\conductor\worker_adapter.py (Devin+Codex dispatch, 984 ln)
F:\.ELAI_workflow\conductor\conductor_loop.py (SQLite autonomy loop, 728 ln)
F:\.ELAI_workflow\conductor\temporal_backbone.py (Temporal.io backbone, 1002 ln)
F:\.ELAI_workflow\conductor\scheduler.py (dependency-frontier scheduler + disposition, 408 ln)
F:\.ELAI_workflow\conductor\gate_runner.py (off-agent gate stack + smoke, 902 ln)
F:\.ELAI_workflow\conductor\conductor_config.py (config loader, 180 ln)
F:\.ELAI_workflow\conductor\config\conductor.toml.example
F:\.ELAI_workflow\conductor\AGENTS.md
F:\.ELAI_workflow\conductor\tests\test_temporal_backbone.py
F:\.ELAI_workflow\workflow\reasoning\AUTONOMOUS-FINISH-SYSTEM-DEVIN-2026-06-18.md (spec, 401 ln)
F:\.ELAI_workflow\workflow\handoff\conductor-build.md (build history, Steps 0-7 DONE)
Greps: conductor/*.py for pack/project_resolver/gate_pack/mailbox/wake/headless_runner/free_api_worker/
       teikoku/bridge (zero conceptual hits); backlog_import.py for ELAI corpus coupling (confirmed).
NOT verified: ~/teikoku/conductor/ (WSL `teikoku` distro, unreachable from F:) — all descendant-side
claims are [per-brief, unverified] and need a file-level diff before porting.
