TEIKOKU P3-A CONTEXT PACKAGE
compiled: 2026-06-20 (context-compiler, session-of-P3 controller)
scope: P3 sub-phases 3.1 (generalize conductor into the core engine) + 3.4 (concurrency/rate-limit scheduler, D-16)
target: the LIVE distro conductor at ~/teikoku/conductor (Ubuntu WSL distro `teikoku`), NOT the F: original (it was modified in P0/P1/P2)
ground truth: every claim below cites a distro file actually read this session; UNVERIFIED items are tagged.

================================================================================
A. P3-A OBJECTIVE + DONE-DEFINITION
================================================================================
3.1: take the already-built conductor (task graph, scheduler, leases, integrator, gate runner) and (a) strip ELAI-hardcoding (F:\ paths, .elai_cc, elai.map, cargo/Rust gates, devin.exe), (b) make it PACK-AWARE — a project resolves to its active packs (projects.toml) and the engine routes gates/lanes/worktree by pack, not by a baked-in ELAI assumption.
3.4: add a MAX-FREE-CONCURRENCY envelope with per-pool/per-model back-off/cooldown/re-route on HTTP 429, BUILDING ON the existing budgets.py + scheduler.py. Read the measured ceilings from ~/teikoku/workers.toml (devin max 3 / kimi-k2.7 / drop to 2 on limit; free_api groq+pollinations N=3 no-throttle; cerebras/gemini/mistral N=1; openrouter fallback).

CRITICAL FRAMING CORRECTION (load-bearing): the master plan says 3.1 "generalizes conductor INTO the core engine (task graph, scheduler, leases, integrator, gate runner)" as if these must be BUILT. They are ALREADY BUILT and sophisticated (see section B). P3-A is GENERALIZATION + PACK-AWARENESS + the 3.4 concurrency envelope, NOT a from-scratch engine. Treat 3.1 as a refactor/parameterize pass on existing, test-covered modules; treat 3.4 as a real new capability layered on budgets.py/scheduler.py.

PHASE-3 DONE-CRITERION this work must serve (master plan line 185): "a real ELAI task runs through select->context->execute->verify(gate)->integrate->record; event-wake demonstrated; max free concurrency observed; surface design pass." 3.5 event-wake = P3-D; 3.6 surface = P3-E; the schema = P3-B; the stricter per-pack gates = P3-C. P3-A owns the generalized engine + the concurrency/rate-limit envelope that "max free concurrency observed" is measured against.

DONE for P3-A specifically:
- the conductor loop (conductor_loop.run_task_loop / conductor.run-once) runs for a NON-ELAI project config with NO F:\/.elai_cc/cargo assumptions reached unless the pack asks for them; the ELAI project still runs green through the SAME loop via its pack (rust+elai).
- the scheduler dispatches up to a per-POOL concurrency cap (from workers.toml), not one global worker_cap; a simulated/real 429 drives back-off + cooldown + re-route to another pool/model, recorded as events.
- every new pub fn wired to a real entrypoint in the same changeset (D-030); engine modules become git-tracked (section D + the tracking-tension decision).

================================================================================
B. CURRENT STATE (the engine already exists — file:symbol anchors)
================================================================================
Engine modules (all in ~/teikoku/conductor, all read this session):

TASK GRAPH + STORE
- conductor_db.py::ControlDB — append-only event log + tasks/leases/integrations/gate_runs/verdicts/worker_slots/budget_counters/model_cooldowns; append_event now carries project_id+session_id (P2-A). Methods used by engine: get_task_row, get_task_spec, upsert_task, ready_task_rows, seed_worker_slot, transaction(), append_event.
- models.py::TaskSpec (frozen dataclass: task_id, plan_id, wave, role, base_commit, allowed_write_set, context_refs, production_hook, acceptance, deterministic_check, forbidden, verifier, lease_ttl_seconds, idem_key, token_budget, model_tier FREE|PAID, priority, max_attempts). ROLES={PLAN,EXECUTE,WIRE,INTEGRATE,AUDIT,POLISH,FIX}. models.py::TaskResult (worker NEVER self-certifies DONE — __post_init__ raises).
- control.db (state/control.db) LIVE COUNTS this session: events 3507, tasks 1689, task_deps 1720, artifacts 1684, gate_runs 15, verdicts 15, worker_slots 3, leases 0, integrations 0, model_cooldowns 0, task_attempts 1, control_state 1, budget_counters 1. Schemas: tasks.status CHECK has the 12 TASK_STATUSES; task_deps is a real DAG edge table (task_id, depends_on_task_id, no self-edge); worker_slots.harness CHECK currently='devin-cli' ONLY (a hardcoding — see C) and has a max_concurrency column already.

DAG / FRONTIER SCHEDULER
- leases.py::LeaseManager.acquire_ready — THE dependency-frontier core: SELECTs READY tasks whose deps are all DONE (NOT EXISTS over task_deps), skips scope-overlapping tasks (scopes_overlap over allowed_write_set), picks an IDLE worker_slot not in cooldown and not account-conflicting, creates a worktree + branch, flips task->LEASED + slot->LEASED, emits LEASE_ACQUIRED + TASK_STATUS_CHANGED. Also: reclaim_expired (TTL), reclaim_stale_heartbeats, mutation-evidence inspection (inspect_worktree_mutation: git status/HEAD vs base_commit), release.
- scheduler.py::DependencyFrontierScheduler.schedule_frontier — wraps refresh_model_cooldowns + reclaim_expired + enforce_retry_tripwires + a control-plane gate, then loops acquire_ready up to worker_cap. scheduler.py::DependencyFrontierScheduler.record_worker_result — the rate-limit/transient/ambiguous-mutation disposition state machine (THIS is where 429 handling already lives; see 3.4 note below).

LEASES — leases.py (read full). Scope-overlap conflict avoidance + account-conflict + cooldown filtering all present.

INTEGRATOR
- integrator.py::SerializedIntegrator.integrate_verified_task — singleton (IntegrationBusy if another active), requires task status VERIFIED, blocks on dirty checkout, builds a binary patch, `git apply --3way --index`, runs a CANDIDATE gate in an isolated worktree, then (if allow_local_commits) `merge --ff-only`, then a POST-MERGE gate; any non-PASS => NEEDS_REVIEW. Emits INTEGRATION_STARTED/APPLIED/FINISHED + TASK_STATUS_CHANGED.

GATE RUNNER
- gate_runner.py::run_gates(task_id, worktree_path, base_commit, db_path, allowed_write_set, *, workflow_root, repo_root, artifact_root) -> {overall_verdict, gate_results, ...}. gate_runner.py::build_gate_commands(GateContext) builds the gate list. record_gate_result writes gate_runs + verdicts rows + artifacts. THIS IS THE MOST ELAI-HARDCODED MODULE (section C).

THE LOOP (what 3.1 generalizes; the slice proved a thin version)
- conductor_loop.py::run_task_loop — the FULL SELECT->CONTEXT->EXECUTE->VERIFY->INTEGRATE->RECORD single-task loop (lease via LeaseManager -> build_context_packet_for_task -> worker_adapter dispatch -> run_gates -> SerializedIntegrator -> append_event). Also run_wave_loop, run_backlog_waves, loop_until_dry (+ a temporal_backbone.py variant selectable by config.backbone).
- slice_driver.py::main — the PROVEN P0.5 thin slice: run_cc_turn (bun base-cc/dist/cli.js plan-OAuth Opus) -> upsert ONE EXECUTE TaskSpec -> setup_worktree (throwaway in-distro git repo) -> run_gates threading workflow_root=/mnt/f/.ELAI_workflow (the FROZEN gate) -> append SLICE_COMPLETE. slice_api.py::run_slice/latest_slice + the http :8787 transport + the slice_api one-shot CLI (`python3 slice_api.py latest|run|memory-*`). This is the live, audited, event-3479-proven loop the engine generalizes.

CLI SURFACE
- conductor.py — the control CLI: doctor, db, events replay, recovery, budget, worker validate-models, context build/check, gate run, schedule frontier, integrate task, run-once, run-wave, run-backlog, loop-until-dry, control (pause/resume/drain/emergency-stop/cancel), dashboard, temporal-worker. seed_worker_slots_from_config seeds worker_slots from config.workers + config.escalation_workers.

CONFIG
- conductor_config.py::ConductorConfig — TOML-loaded: paths (workflow_root/conductor_root/repo_root/state_db/worktree_root/artifact_root/devin_bin/plan_corpus_root/finish_dir), server, backbone(sqlite|temporal), temporal, LimitConfig (worker_cap, lease_ttl, retry caps, daily caps, default_token_budget, model_cooldown_seconds), SafetyConfig, workers[]/escalation_workers[] (WorkerSlotConfig: slot_id, harness, model, model_tier, account_id, max_concurrency, codex_home, wsl_distro). normalize_path maps F:\ -> /mnt/f. config/conductor.toml.example currently has F:\ paths + 3 codex worker slots (gpt-5.5). NO project/pack field anywhere.

BUDGETS / RATE-LIMIT PRIMITIVES (3.4 builds DIRECTLY on these)
- budgets.py::BudgetManager — retry caps (free/paid), daily attempt counters (per tier, date-keyed), can_start_task, record_attempt_start, refund_attempt_budget, enforce_retry_tripwires (FREE->PAID escalation), set_counter/consume_counter (generic named budget counters), mark_model_cooldown (writes model_cooldowns + flips worker_slots state COOLDOWN), refresh_model_cooldowns (expires cooldowns + recovers orphaned COOLDOWN slots), model_available. PER-MODEL cooldown EXISTS; PER-POOL concurrency does NOT (gap, see F).
- workers.toml (measured envelope, read full): schema_version=1; [devin] max_concurrency=3 preferred kimi-k2.7 fallback 2; [free_api.groq] max_concurrency=3 cooldown 2s (MEASURED); [free_api.pollinations] max_concurrency=3 cooldown 6s keyless (MEASURED); [free_api.cerebras] N=1 cooldown 12s; [free_api.gemini] N=1 cooldown 6s; [free_api.mistral] N=2 cooldown 1s; [free_api.openrouter] N=1 fallback (429 quota observed); [codex] p6-deferred. Provider keys live value-free in teikoku/free-proxy/.env (gitignored), referenced via os.environ; workers.toml is NUMBERS/STRINGS ONLY.
- projects.toml (read full): one [[project]] elai (path=/home/user/teikoku/projects/elai_cc, platform=rust+ts, packs=[rust,elai], done="wiring/tests+dashboard", lanes=[all], vcs=git). Schema documented for adding game (unity+media). THIS is the registry the engine must consult for pack-awareness.

WORKER ADAPTER (the execute step, provider abstraction)
- worker_adapter.py — WorkerProvider Protocol + SubprocessWorkerProvider + WorkerAdapter; build_devin_command + build_codex_command (codex_home/config.toml/wsl_distro). DEFAULTS hardcoded: DEFAULT_DEVIN_BIN=C:\...\devin.exe, DEFAULT_*_ROOT=F:\... , DEFAULT_AGENT_ROOT=F:\.ELAI_workflow\.claude\agents. NOTE: this is the seam where the free-API proxy lane (Anthropic /v1/messages on :4000, per-session ANTHROPIC_BASE_URL) would graft as a third provider; the 0.3 graft proved the CC-side seam (base-cc api/client.ts), but worker_adapter currently only knows devin-cli + codex harnesses.

================================================================================
C. ELAI-HARDCODING INVENTORY (the exact spots 3.1 must generalize, file:line)
================================================================================
PATHS (F:\ / .elai_cc / docs / FINISH defaults):
- conductor_config.py:141-151 — workflow_root, conductor_root, repo_root, plan_corpus_root, finish_dir all default to F:\.ELAI_workflow\... . These are config-overridable (TOML), but the DEFAULTS are ELAI; a non-ELAI project gets ELAI defaults if a field is omitted.
- gate_runner.py:27-30 — DEFAULT_DB_PATH / DEFAULT_REPO_ROOT / DEFAULT_WORKFLOW_ROOT / DEFAULT_ARTIFACT_ROOT all r"F:\.ELAI_workflow...".
- conductor.py:537-539 — the `gate run` argparse defaults are F:\.ELAI_workflow.
- worker_adapter.py:26-30 — DEFAULT_DEVIN_BIN (Windows .exe), DEFAULT_WORKTREE_BASE/ARTIFACT_ROOT/REPO_ROOT/AGENT_ROOT all F:\ .
- conductor.py:49-50 — doctor's required_outer surface asserts elai.map + elai_graph.json exist under workflow_root (ELAI-specific doctor check).
- audit_smoke_setup.py:12,19-20 — F:/.ELAI_workflow defaults (test/smoke fixture, lower priority).

GATE CONTENT (the gate command set is ELAI/Rust-specific — this is the heart of pack-awareness, overlaps P3-C):
- gate_runner.py build_gate_commands(): unconditional gate_freeze_check (reads .claude/gate-manifest.txt under workflow_root); rust path -> production_caller_check; py path -> python_wire_check; maintainability_check; touched_test_rs -> traced_test_assert_check; plan_file -> wiring_contract_check (passes --map elai.map) + spec_link_check; Cargo.toml present -> cargo check --workspace + cargo test -p <crate>. gate_runner.py:277 hardcodes elai.map; :303-307 cargo; :318-320 ELAI_PROJECT_ROOT/ELAI_MAP env. The whole verifier set assumes the ELAI .claude/tools layout + a Rust crate. 3.1 must make the gate SET come from the project's active PACK (rust-pack = cargo+wiring+maintainability+production-caller+tracing), not be baked into build_gate_commands. NOTE: per N-02 + C-04, the CHECKER SCRIPTS themselves are FROZEN/wrapped, not rewritten — only the SELECTION of which gates run per pack is generalized. Deep stricter-gate work is P3-C; P3-A just needs the gate set to be pack-routed, not hardcoded.

WORKER HARNESS (provider hardcoding):
- worker_slots.harness CHECK constraint = 'devin-cli' ONLY (control.db schema; migration 001_initial). conductor.toml.example seeds harness="codex". A generalized engine + the free-API lane needs the harness set widened (devin-cli | codex | free-api/anthropic-proxy) — this is a schema-adjacent change; coordinate with P3-B (schema) so the CHECK is not narrowed-then-widened twice.

NO PACK CONCEPT ANYWHERE: ConductorConfig has no project_id/packs field; the loop/scheduler/gate-runner take no pack argument; projects.toml is read only by the dashboard harness_api + the compose script, NOT by the engine. This is the central 3.1 gap: thread project_id -> packs (from projects.toml) -> gate-set + lane-set + repo_root/worktree_root through ConductorConfig (or a new ProjectConfig) into run_task_loop + build_gate_commands.

================================================================================
D. SEAMS THE EXECUTOR MUST WIRE (D-030 producer->consumer, same changeset)
================================================================================
For 3.1 (pack-aware engine) — any new pub fn (e.g. a resolve_project_packs(), a pack-aware build_gate_commands variant, a ProjectConfig loader) MUST be CALLED in the same changeset from a real entrypoint. Named PRODUCTION_HOOK seams that already exist:
- conductor.py::run_schedule / run_loop_once / run_loop_wave / run_backlog / run_gate / run_integrate — the CLI entrypoints. A new pack-resolution fn wires here (the CLI already loads ConductorConfig.from_file; extend that path).
- conductor_loop.py::run_task_loop — the loop entrypoint; a pack-aware gate-set or repo_root selection wires HERE (it already calls run_gates + ConductorConfig).
- gate_runner.py::run_gates -> build_gate_commands — if the gate set becomes pack-driven, the new selector is called from build_gate_commands, which run_gates already calls; run_gates itself is called from conductor.py::run_gate (CLI), conductor_loop.py, integrator.py (candidate + post-merge), AND slice_driver.py. ALL FOUR call sites must keep working.
- slice_api.py (http :8787 + one-shot CLI) — the live transport the dashboard hits; a P3-E orchestration-monitor surface will read new engine state through here (mirrors the P1-C harness_api + P2-C memory pattern: a reader fn in *_api.py -> an HTTP route + a one-shot CLI verb -> a Tauri #[command] -> signal/src/lib/api/*.ts -> a Svelte screen). For P3-A the engine API is the producer; P3-E is the dashboard consumer — but if P3-A ships any new orchestration-state reader, wire it to slice_api now or mark WIRE-DARK[P3-A] with a P3-E promotion row.

For 3.4 (concurrency/rate-limit scheduler) — the producer is a pool-aware concurrency layer; the consumer is the scheduler dispatch loop:
- scheduler.py::DependencyFrontierScheduler.schedule_frontier — TODAY caps at one global worker_cap. The 3.4 producer (a per-pool/per-model concurrency counter reading workers.toml) must be CONSUMED here: replace/augment the `cap` computation so each pool's in-flight count <= its workers.toml max_concurrency. Wire from conductor.py::run_schedule (which constructs the scheduler) by loading workers.toml into the scheduler.
- scheduler.py::record_worker_result — the 429 disposition ALREADY re-routes (rate_limited+no-mutation -> refund + READY + mark_model_cooldown(model, cooldown_until)). 3.4 EXTENDS this: use the workers.toml per-pool cooldown_seconds (not the single config.model_cooldown_seconds) and add pool-level re-route (try a different pool's model when one pool is in cooldown). Producer = a pool-cooldown lookup; consumer = record_worker_result + acquire_ready's slot filter.
- budgets.py::mark_model_cooldown / refresh_model_cooldowns / consume_counter — reuse these primitives; do NOT reinvent cooldown. A new per-pool concurrency check should use consume_counter/set_counter (named budget_counters) OR a live COUNT(*) over leases joined to worker_slots by pool. Whichever, wire it into schedule_frontier.

================================================================================
E. CONSTRAINTS (Section-0 locked + scope guard + house rules that bind P3-A)
================================================================================
SECTION 0 (SETTLED, do not re-open):
- D-16 (concurrency = hybrid): owner sets pool+model+cap toggles; an auto-scheduler dispatches within that envelope, rate-limit-aware (back-off/cooldown/re-route); autonomous lane defaults to MAX free concurrency (measured). BUILD ON the conductor's existing budgets/lease/cooldown logic — do NOT rebuild it. The config toggles (per-pool cap) are owner-set; 3.4 enforces the envelope, it does not decide policy.
- D-12 (WSL isolation): all code in the `teikoku` WSL distro; native tools (devin.exe, gpt-bridge, RunPod, Unity) stay Windows-side, reached via localhost. worker_adapter's devin/codex paths are cross-boundary endpoints (KEEP as Windows paths), not bugs to "sweep" — distinguish a path-to-generalize (engine repo_root) from a cross-boundary-endpoint (devin.exe).
- D-06 (done = deterministic, per-project, off-agent, frozen, STRICTER for Opus-1M): the gate is IMPROVED not rebuilt. P3-A only ROUTES gates per-pack; the stricter gate content is P3-C.
- D-11/D-08/D-07 are P3-B/P2/P3-D respectively — out of P3-A scope but the schema (P3-B) and event-wake (P3-D) will consume P3-A's engine; keep TaskSpec/ConductorConfig extensions schema-friendly (P3-B defines the canonical plan schema; do not invent a competing one in P3-A — extend the existing TaskSpec minimally and flag the overlap for P3-B).

SECTION 2 SCOPE GUARD (refuse creep): N-01 no sidecar/model/quant/training; N-02 no gate rebuild from scratch (improve+wrap the frozen checkers); N-03 lean dashboard (no graph-viz spectacle — the P3-E monitor is a lean surface); N-05 no new ELAI/game features; N-06 no brand/UI invention. For P3-A specifically: do NOT rewrite the gate checker scripts (N-02); do NOT add a sidecar/model concern (N-01); do NOT widen the engine beyond task-graph/scheduler/lease/integrator/gate generalization + the concurrency envelope.

C-02 AGENT POLICY: per-sub-phase cycle = context-compiler -> executor -> auditor ONLY (no integration-explorer, no memory-writer). MAX 2 agents parallel. All Opus. Memory = append to workflow/handoff/teikoku-build.md (RAG auto-indexes). ONE commit per phase (P3 end), maintainer voice, no AI-attribution trailer, no test counts in the message.

TOOLING (load-bearing, from the handoff FSM/GATE GOTCHA): the F: PreToolUse gate MISGATES this WSL build. In EXECUTING the Bash tool is DENIED and Write to distro paths is DENIED; in AUDITING Bash is restricted + Write only to workflow/audit-scratch. The PowerShell tool is UNGATED in every state -> run ALL distro ops via `wsl -d teikoku -- bash -lc '...'` through the PowerShell tool; author distro files via heredoc/base64/a .sh inside the distro, NOT the Write tool. NO FSM reset, NO gate edit (frozen+global). PowerShell->wsl->bash mangles backslash-escapes (\$ in printf) — prefer single-quoted PS strings, heredocs, or a .sh file written inside WSL.

WIRING-ENFORCEMENT: R6 producer-without-consumer = build break; R7 test-file callers don't satisfy reachability; R10 unit-green != integrated-green (the P3 final R10 gate drives the REAL entrypoint end-to-end on a HARD input). D-030 self-verify: .py -> python_wire_check.py; the auditor RE-RUNS it off-agent. Legitimate-dark escape = an in-code `# WIRE-DARK[P3-A]: ...` marker + a backing stubs.md / wire-dark-allowlist row.

MAINTAINABILITY (auditor-applied, off-agent): MR-1 no file crossing 1000 lines (gate_runner.py is ~33KB/largest engine file — adding pack routing risks pushing it over; consider extracting a gate-selection module); MR-2 no `as any`/type-erasure (Python: no bare `# type: ignore` without cause); MR-3 no fn >80 lines (build_gate_commands is already long); MR-4 branching density. Reuse existing helpers (budgets cooldown primitives, normalize_path).

ANTI-SLOPSQUAT (C-05): no new dependency without registry verification (name+version+last-published). 3.4 should need NO new deps (stdlib + tomllib + existing conductor primitives). If a TOML writer or a rate-limiter lib is reached for, verify first; prefer the existing tomllib (read) + manual write.

GROUNDED-BY-DEFAULT: every behavioral claim about budgets/scheduler/leases must be grounded in the code read here, not training memory about "how schedulers work".

================================================================================
F. RISKS + OPEN QUESTIONS / FORKS the executor will hit
================================================================================
FORK 1 (3.4 concurrency data structure — the main 3.4 design call): per-pool concurrency can be enforced two ways. (a) a live COUNT(*) over leases JOIN worker_slots grouped by a new pool/harness key, compared to workers.toml max_concurrency, inside schedule_frontier's acquire loop. (b) named budget_counters (set_counter/consume_counter already exist) keyed per pool, consumed on lease + refunded on release. (a) is stateless/self-healing (survives restart, no counter drift) but needs a pool identity on worker_slots; (b) reuses an existing primitive but risks counter drift on a crash mid-lease. LEAN: (a) — the lease table IS the source of truth and reclaim_expired already self-heals; add a `pool` (or reuse account_id/harness) grouping. This is a material fork -> the executor should pick with grounding, or STOP+handoff if the pool-identity column is structural.

FORK 2 (pool identity / worker_slots model): workers.toml pools (devin, free_api.groq, free_api.pollinations, ...) do not map 1:1 to today's worker_slots (which are devin-cli/codex slots seeded from config.workers). To enforce per-pool concurrency the engine needs a pool tag on each slot. Decide: extend worker_slots with a `pool` column (migration 006) vs derive pool from harness+model. Touches the schema -> COORDINATE with P3-B (schema is P3-B's job); do not narrow-then-widen the harness CHECK twice. Possible STOP+handoff if it forces a schema decision that belongs to P3-B.

FORK 3 (how pack-awareness threads): extend ConductorConfig with project_id+packs (loaded from projects.toml) vs introduce a separate ProjectConfig consulted alongside ConductorConfig. LEAN: a thin project resolution that reads projects.toml -> active packs -> a gate-set + repo_root, threaded through run_task_loop + build_gate_commands; keep ConductorConfig as the runtime/limits config and add a project layer above it. Avoid a big-bang rewrite of ConductorConfig.

FORK 4 (gate-set source): the per-pack gate set (rust = cargo+wiring+maintainability+production-caller+tracing) — does P3-A define the gate-set registry, or just make build_gate_commands TAKE a gate-set and leave the registry to P3-C? LEAN: P3-A makes build_gate_commands pack-parameterized + ships the rust-pack set (which IS today's hardcoded set, just made explicit + pack-keyed); P3-C adds the OTHER packs + the stricter-for-Opus content. Flag the boundary so P3-A doesn't do P3-C's stricter work.

RISK A (the F: original divergence): the distro conductor is NOT the F: copy anymore (P2 modified conductor_db.py + events.py; P0/P1 added slice_*/harness_api). Always read/edit the DISTRO copy. The F: conductor is stale for this work.

RISK B (CONDUCTOR-TRACKING TENSION — must be resolved at the P3 commit, tracked item #2): only 15 conductor files are git-tracked today (git ls-files conductor/ = conductor_db, events, handoff_indexer, harness_api, memory_*, slice_*, migrations 001-005). The ENTIRE CORE ENGINE is gitignored via `/conductor/*` in .gitignore (conductor.py, scheduler.py, leases.py, integrator.py, gate_runner.py, conductor_loop.py, budgets.py, models.py, conductor_config.py, worker_adapter.py, control_plane.py, recovery.py, temporal_backbone.py, context_builder.py, waves.py, scope_utils.py, dashboard_server.py, etc. + config/). P3-A WILL EDIT these to generalize them, so they MUST become tracked. The .gitignore model must flip: the generalized engine is now TEIKOKU-AUTHORED SOURCE, not "regenerable-from-F: working material". At the P3 commit, replace the per-file `!` whitelist approach with tracking the whole generalized conductor SOURCE (the .py + migrations + config/*.example) while keeping .memory-venv/, state/, audit_fixtures/, __pycache__/, *.pyc, test scratch (pytest_*.txt, *_output.json), and the test_*.json/*.txt fixtures gitignored. This is a controller/commit decision but the executor's edits make it unavoidable — flag it, do not silently leave the engine untracked.

RISK C (slice_driver threads workflow_root=/mnt/f/.ELAI_workflow): the live slice + the gate still resolve the FROZEN gate checkers from /mnt/f (F: mounted). Generalizing repo_root must NOT break this — the FROZEN gate location (the .claude/tools checkers + gate-manifest) is intentionally on /mnt/f and is the rust-pack's gate home; do not "sweep" it to a distro path. Distinguish: project repo_root (generalize) vs the frozen gate-checker location (a pack endpoint, keep on /mnt/f for the rust pack).

RISK D (python_wire_check tracked bug, item #1): P2 audits reported a RecursionError when python_wire_check.py crawls conductor/audit_fixtures nested trees; the auditor substituted cross-file consumer grep + live-execution. A direct single-file repro this session did NOT reproduce the crash (it returned clean "missing input files" because of cwd/--root, exit 0) — so the trigger is the recursive crawl of audit_fixtures, NOT a single file. UNVERIFIED whether it still fails on a full --root scan in the distro. P3-A's self-verify (D-030) runs python_wire_check on its OWN changed .py files (scoped --files), which should avoid the audit_fixtures crawl; the deep fix is folded into P3-C. If P3-A's self-verify hits the crash, scope --files tightly (do not point --root at conductor/) and note it.

RISK E (record_worker_result already handles 429 at task grain): do NOT duplicate the existing rate_limited disposition (scheduler.py record_worker_result: refund + READY + mark_model_cooldown). 3.4's ADDITION is the per-POOL concurrency CAP (dispatch-side) + per-pool cooldown_seconds from workers.toml (vs the single global model_cooldown_seconds) + cross-pool re-route. Build ON record_worker_result, do not reimplement it (anti-spiral + N-02 improve-not-rebuild).

================================================================================
G. PINNED VERSIONS / DEPS already present (reuse, do not re-add)
================================================================================
- Python: the distro system python3 is 3.12 (conductor .memory-venv is python3.12). Engine modules use stdlib only (sqlite3, subprocess, argparse, dataclasses, tomllib [3.11+], uuid, hashlib, json, pathlib, datetime, concurrent.futures). 3.1/3.4 should stay stdlib + tomllib.
- conductor/.memory-venv (P2-A, anti-slopsquat-verified): mcp 1.28.0, sentence-transformers 5.6.0, sqlite-vec 0.1.9, rank-bm25 0.2.2, torch 2.12.1+cpu. These are for the MEMORY layer (memory_semantic/memory_server), NOT the engine — P3-A should not need them. Do NOT add the engine's deps into .memory-venv; the engine runs on system python3 stdlib.
- backbone optionality: temporal_backbone.py uses the temporalio client (config.backbone="temporal"); the default backbone="sqlite" is in-process and needs no extra dep. Keep the sqlite backbone as the P3-A path; do not require temporal.
- The frozen gate checkers (/mnt/f/.ELAI_workflow/.claude/tools/*.py) are FROZEN artifacts (gate-manifest.txt + .lock) — reuse via gate_runner, never re-author (N-02 + R8).
- workers.toml + projects.toml are existing TOML registries — READ via tomllib, do not invent a parallel registry.

================================================================================
SOURCES CONSULTED (all read this session, distro unless F: noted)
================================================================================
F: TEIKOKU-MASTER-PLAN.md (§0 D-01..D-17, §1, §2 N-01..N-06, §3 C-01..C-07, Phase 3 lines 172-188, Appendix A)
F: workflow/handoff/teikoku-build.md (full, incl ORCHESTRATOR CONTRACT, P0/P1/P2 outcomes, tracked items, P3 kickoff entry)
F: .claude/rules/agent-dispatch-preamble.md; .claude/agents/context-compiler.md
distro: ~/teikoku ls; conductor/ full ls + all .py enumeration + migrations
distro: conductor/{models.py, events.py, budgets.py, scheduler.py, leases.py, integrator.py, conductor.py, conductor_config.py} FULL
distro: conductor/{slice_driver.py, slice_api.py} FULL; conductor_loop.py run_task_loop (1-200); gate_runner.py (1-420)
distro: control.db (table list + counts + events/tasks/task_deps/worker_slots schemas via .memory-venv python sqlite3)
distro: ~/teikoku/.gitignore FULL; git ls-files conductor/ (15 tracked); git log -1 (8040d02); conductor/config/conductor.toml.example FULL; ~/teikoku/workers.toml + projects.toml FULL
distro: grep of F:\/.elai_cc/elai.map/cargo/ELAI_* across engine .py; worker_adapter.py class/def grep; python_wire_check single-file repro (no crash, cwd/--root artifact)

VALIDATION WARNINGS
- python_wire_check RecursionError: UNVERIFIED in this session (single-file repro did NOT crash; the P2-documented crash is the audit_fixtures recursive crawl). Scope P3-A self-verify to --files, not --root.
- worker_adapter free-API/anthropic-proxy lane: the proxy seam was proven CC-side (0.3, base-cc api/client.ts) but worker_adapter only knows devin-cli + codex harnesses; whether the free-API lane is wired as a worker PROVIDER is UNVERIFIED (likely a P3/P4 item — flag if 3.4's "max free concurrency" needs a free-API worker provider, which may exceed P3-A scope and is a STOP+handoff fork).
- worker_slots.harness CHECK='devin-cli' but config seeds harness='codex' — a latent schema/config mismatch; the engine seeds slots from config so codex slots may violate the CHECK on a fresh init. UNVERIFIED whether a fresh seed currently succeeds; flag for P3-B (schema) coordination.
