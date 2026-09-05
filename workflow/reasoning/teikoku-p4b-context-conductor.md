TEIKOKU P4-B context: conductor convergence surface (task P4-B-CC1)

Session context-compile, READ-ONLY. Date 2026-06-20. Scope: F:\.ELAI_workflow\conductor (ancestor) vs WSL distro teikoku ~/teikoku/conductor (canonical). Deliverable for the converge+unify cycle per TEIKOKU-P4-REVISION-2026-06-20.md.

================================================================================
HEADLINE FINDING (changes the P4-B framing)
================================================================================
The revision plan's premise -- "TEIKOKU LACKS conductor_loop.py; port the missing wave loop + disposition brain + serialized integrator" -- is STALE. All three are ALREADY in the canonical engine, ported and generalized during P3:
 - conductor_loop.py EXISTS in teikoku (824 lines vs the ancestor's ~728). It is the ancestor's loop made pack-aware (P3-A project_resolver overlay + P3-D dep-wake). The diff is purely ADDITIVE (teikoku is a superset).
 - the disposition brain (scheduler.py::DependencyFrontierScheduler.record_worker_result) is ALREADY ported; the only teikoku delta is the per-pool envelope parameterization (pool_concurrency).
 - the SerializedIntegrator is ALREADY ported AND already pack-aware (threads packs / active_packs; P3 R10 fix).
 - the `headless` verb + run_headless + free_api_worker (the universal base-cc worker skeleton) ALREADY exist (P4-B build, held unaudited).

So P4-B is NOT a port of the orchestration core. The orchestration core is present and tested. The REAL remaining P4-B work is:
 (A) WIRE THE DEVIN BRIDGE (:8810) as a base-cc backend, retiring the devin.exe lane (worker_adapter.py still calls devin.exe directly; NO bridge wiring exists anywhere).
 (B) HARVEST the codex-exec paid lane (sibling WorkerProvider) + guard.py from the codex conductor (donor; out of this read's scope, F:\conductor has no codex adapter -- it lives in codex-vm /root/.conductor-codex, NOT probed).
 (C) AUDIT the held P4-B free_api_worker + headless_runner + run_headless.
 (D) A small set of leftover ancestor-vs-teikoku reconciliations (one is a real bug -- see section 1).
The wave loop / disposition / integrator port is effectively DONE; treat sections 1-3 below as a VERIFICATION + delta-closure surface, not a from-scratch port.

This is a MATERIAL framing correction. It is surfaced here for the controller to fold into the P4-B scope; it does not block (it shrinks the work). It is NOT a fork requiring an owner decision -- the owner's converge+unify intent is satisfied by closing the deltas + wiring the bridge.

================================================================================
1. WAVE LOOP PORT SURFACE
================================================================================
Ancestor file: F:\.ELAI_workflow\conductor\conductor_loop.py (728 lines, read in full).
Canonical file: ~/teikoku/conductor/conductor_loop.py (824 lines) -- ALREADY the ported superset.

RUN-WAVE-LOOP ENTRY + THE CYCLE (ancestor line refs, identical structure in teikoku):
 - run_task_loop (ancestor :67) = the single SELECT->CONTEXT->EXECUTE->VERIFY->INTEGRATE->RECORD cycle.
   SELECT  :145-172  leases.acquire_ready(task_id) (reclaim_expired + enforce_retry_tripwires first)
   CONTEXT :176-199  build_context_packet_for_task(task, config.repo_root) + LOOP_STAGE event
   EXECUTE :209-225  budgets.record_attempt_start -> WorkerAdapter.from_conductor_config(...).dispatch(...)
   VERIFY  :227-305  scheduler.record_worker_result (the DISPOSITION call, section 2) -> if next_status VERIFYING: run_gates -> verdict PASS|FAIL
   INTEGRATE :278-297 SerializedIntegrator.integrate_verified_task (unless skip_integrate/dry_run)
   RECORD  :307-321  LOOP_STAGE event + LoopResult
   IntegrationBusy handler :338-363 releases lease back to VERIFIED for a transient requeue (NOT a failure).
 - run_wave_loop (ancestor :385) = drive up to `limit` ready tasks for one wave; ThreadPoolExecutor when worker_cap>1 (:462-479); cooldown-sleep when all slots COOLDOWN and frontier non-empty (:432-444).
 - run_backlog_waves (ancestor :504) = iterate WAVE_ORDER, set_active_wave, _compact_epoch per wave.
 - loop_until_dry (ancestor :554) = repeat LOOP_UNTIL_DRY_ORDER (AUDIT,POLISH) until K dry passes; _compact_epoch per pass.

WAVE DEFINITION SOURCE: waves.py (ancestor + teikoku are BYTE-IDENTICAL, verified by cat). WAVE_ORDER = IMPLEMENT, WIRE_SWEEP, INTEGRATION, AUDIT, POLISH. LOOP_UNTIL_DRY_ORDER = AUDIT, POLISH.

EPOCH / COMPACTION: _compact_epoch (ancestor :698) writes artifact_root/epochs/<run_id>/<label>.json with task_counts + payload, emits EPOCH_COMPACTED. PRESENT identically in teikoku. Project-agnostic (only artifact_root, which is config-resolved).

ELAI-HARDCODED COUPLINGS IN THE ANCESTOR LOOP, and how TEIKOKU already neutralized each:
 (a) config.repo_root / worktree_root point at F:\.ELAI_workflow\.elai_cc by DEFAULT (conductor_config.py:155-159 default strings "F:\\.ELAI_workflow", "F:\\.ELAI_workflow\\.elai_cc"). NEUTRALIZED in teikoku by _resolve_loop_project (teikoku conductor_loop.py:70-82): resolve_project(project_id) overlays the registry's repo_root/worktree_root via dataclasses.replace, default project "elai" preserves old behaviour. Substitute = project_resolver + projects.toml (CONFIRMED present, ~/teikoku/teikoku/projects.toml has elai + demo rows).
 (b) the gate command set is fed via run_gates -> gate_packs. NEUTRALIZED: teikoku threads packs=active_packs(project) into run_gates at BOTH gate call sites (loop diff :291, :311) and into the integrator. Substitute = gate_packs.py per-pack gate sets (rust/elai vs generic vs unity).
 (c) the wave GRAMMAR (IMPLEMENT/WIRE_SWEEP/AUDIT/POLISH) is the ELAI plan-stage vocabulary. This is the ONE coupling NOT yet parameterized: waves.py is a flat module-level constant tuple, identical in both trees, hardcoded to the ELAI stage names. For a generic project these wave names are semantically ELAI-shaped but FUNCTIONALLY harmless (a generic project's tasks are simply assigned to one of these waves at import time; backlog_import's generic parser assigns them). RESIDUAL: if the generality gate demands zero ELAI-shaped vocabulary reachable, waves.py wave names are the only candidate; recommend the controller rule whether ELAI-named wave constants count as "ELAI hardcode" or are an acceptable generic pipeline vocabulary. LEAN: acceptable (they are pipeline-stage names, not ELAI paths/grammar/gates), but flag it.

PORT MAP (what still MOVES vs what is DONE):
 - run_task_loop / run_wave_loop / run_backlog_waves / loop_until_dry / _compact_epoch: ALREADY IN teikoku conductor_loop.py. NO port needed. Action = VERIFY parity + close the one bug below.
 - ONE REAL DELTA (a dropped line, likely a bug): the ancestor's _lease_from_dict carries `model_tier=value.get("model_tier")` (ancestor :664); the teikoku diff shows this line REMOVED ("664d761 < model_tier="). And the teikoku scheduler diff removed `"model_tier": lease.model_tier` from the lease dict it emits (scheduler diff :390d401). So model_tier is no longer threaded from the scheduler's lease dict into LeaseRecord through this path. This is reachable in the wave loop's batch path (_lease_from_dict at conductor_loop :654). VERIFY whether model_tier is now sourced elsewhere (the scheduler SELECT adds s.pool but the LeaseRecord still has a model_tier field); if not, FREE/PAID tier disposition could silently read None. Route to the executor as a numbered verify-or-fix item, NOT a fresh port.
 - PARAMETERIZED-NOT-COPIED (already done, confirm only): repo_root/worktree_root (via project), gate packs (via active_packs), state_db/artifact_root (config). 
 - new conductor_loop.py file? NO. The file exists; do not create a second one. All wave-loop work is EDITS to the existing teikoku conductor_loop.py.

================================================================================
2. DISPOSITION BRAIN
================================================================================
LOCATION (both trees): scheduler.py::DependencyFrontierScheduler.record_worker_result (ancestor scheduler.py:123-339). NOT in conductor_loop / worker_adapter / budgets -- the loop CALLS it at VERIFY (conductor_loop :228).

INPUTS: (lease_id, worker_result_dict, attempt_id). Reads result flags: rate_limited (:139), mutated (git-touched files), provider error category (TRANSIENT_PROVIDER_CATEGORIES / "rate_limited", :20). State it touches: leases (release to READY/VERIFYING/NEEDS_REVIEW), budgets (refund_attempt_budget, mark_model_cooldown), worker_slots (cooldown).
OUTPUTS: a WorkerDisposition (task_id, lease_id, next_status, reason, cooldown_until?). The four branches:
 1. mutation + verifiable -> next_status VERIFYING (the gate runs).
 2. rate_limited AND no mutation (:233-254) -> refund budget, release lease to READY, mark_model_cooldown(model, "rate_limited", cooldown_until). THIS IS THE 429 DISPOSITION.
 3. transient provider error (:264-285) -> cooldown + retry.
 4. terminal -> NEEDS_REVIEW/FAILED per budget.

PROJECT-AGNOSTIC? YES. It reasons over worker-result flags + budgets + cooldowns; it knows nothing about ELAI paths/grammar/gates. PORT AS-IS -- and it ALREADY IS ported (the teikoku scheduler.py diff vs ancestor is ONLY the pool-envelope additions, never a change to the disposition branches themselves). The disposition logic moved unchanged in P3-A.

RECONCILE WITH pool_concurrency.py (the question the task asks): they are ORTHOGONAL layers, not duplicates.
 - record_worker_result = the REACTIVE disposition: a worker turn FAILED/429'd, decide retry-vs-cooldown-vs-terminal, refund budget, set a per-model cooldown. It fires AFTER a result.
 - pool_concurrency.py = the PROACTIVE envelope: BEFORE acquiring a lease, exclude pools already at their workers.toml max_concurrency (pools_at_capacity via a live COUNT(*) over leases JOIN worker_slots GROUP BY pool); and on a 429, apply the per-POOL cooldown_seconds (vs the single global model_cooldown_seconds). It fires at schedule time + extends the cooldown duration.
 What pool_concurrency ADDS beyond the disposition brain: (1) per-pool (not just per-model) concurrency capping at dispatch; (2) per-pool cooldown durations from workers.toml; (3) the measured FREE-pool cap=2 enforcement. The disposition brain still owns the retry/refund/terminal DECISION; pool_concurrency owns HOW MANY run at once + HOW LONG a pool sits out. They compose: schedule_frontier consults pools_at_capacity (proactive), record_worker_result consults cooldown_seconds_for(pool) (reactive duration). NO merge needed; they are already wired together (scheduler.py holds a PoolEnvelope, both methods read it).
ACTION: NO port. VERIFY the composed behaviour is exercised end-to-end (it is, by test_p3a_pack_concurrency.py).

================================================================================
3. SERIALIZED INTEGRATOR
================================================================================
Ancestor: F:\.ELAI_workflow\conductor\integrator.py (read in full, 429 lines).
Canonical: ~/teikoku/conductor/integrator.py.
DIFF (teikoku - ancestor), the ONLY changes:
 + constructor gains `packs: tuple[str,...] = ("rust",)` (teikoku :65) stored as self.packs (:76-80, with a P3 BLOCK-1 comment: the integrator must run the SAME pack-keyed gate set the loop's verify gate used).
 + both gate_runner calls (candidate gate :155, post-merge gate :198) now pass packs=self.packs.
EVERYTHING ELSE IS IDENTICAL: the single-active-integration lock (_start_integration :232 raises IntegrationBusy if another PENDING/APPLIED exists), the VERIFIED-precondition check, patch extraction (git diff --binary, write_bytes to preserve LF -- the Windows CRLF guard at ancestor :114-117), worktree add, git apply --3way --index, candidate gate, allow_local_commits guard, ff-only merge, post-merge gate.

VERDICT: TEIKOKU's integrator ALREADY SUPERSEDES the ancestor's. NO PORT NEEDED. The ancestor has NOTHING the canonical lacks; the canonical has the pack-awareness the ancestor lacks. The default packs=("rust",) keeps the ELAI/Rust path byte-equivalent while a generic project gets its generic gate set at both gates. The R10 P3 sweep already caught + fixed the "integrator not pack-aware" seam (per the handoff INDEX). Section 3 closes as a no-op for the port.

================================================================================
4. CURRENT STATE OF THE HELD P4-B WORKERS
================================================================================
free_api_worker.py (206 lines, read in full) -- the universal base-cc worker EXECUTOR (NOT a provider; a command+env+parse helper module that worker_adapter dispatches into to stay under the 1000-line MR-1 boundary). What it does TODAY:
 - is_free_api_harness(harness) -> True for harness in ("free-api","anthropic-proxy"). This is the dispatch GUARD worker_adapter._build_worker_command checks BEFORE the devin fall-through (worker_adapter.py:447).
 - build_free_api_command(slot_model, taskspec_file) -> ([bun, <abs>/base-cc/dist/cli.js, --print, --output-format json, --strict-mcp-config, --dangerously-skip-permissions, --model, slot_model], prompt-from-stdin). slot_model is a FREE-PROXY alias (free-pollinations/free-groq/...), NEVER a devin model. Mirrors slice_driver.run_cc_turn.
 - free_api_env(base_env, proxy_base_url, account_id) -> injects ANTHROPIC_BASE_URL (default http://127.0.0.1:4000, the litellm proxy) + ANTHROPIC_AUTH_TOKEN (the proxy master_key from LITELLM_MASTER_KEY env or the .master-key file), and POPS ANTHROPIC_API_KEY (the 0.3 mis-route hazard).
 - resolve_master_key: env LITELLM_MASTER_KEY -> .master-key file -> None (keyless Pollinations tiers still work). Value read, never logged.
 - parse_cc_json_notes: extracts the assistant turn from base-cc's --output-format json envelope (last JSON line, .result / fallbacks), persists a minimal ATIF record so dispatch's artifact path resolves like the devin/codex branches. Never raises.
 ENV/BASE_URL/AUTH/MODEL it sets: BASE_URL=:4000 proxy, AUTH=proxy master key, MODEL=free-proxy alias, API_KEY popped.

headless_runner.py (454 lines, read in full) -- the `headless` front-end driver:
 - run_headless(template_id|free_text, project_id, ...): GPU policy hook -> select (feature-map / template / free-text) -> enqueue batch (spawn.enqueue_spawn, bounded) -> drain (run_backlog_waves / loop_until_dry / per-id run_task_loop). THIN orchestrator over helpers (MR-3).
 - GPU-DEFER hook = gpu_policy_check (:gpu_policy_check): gpu_required template + no GPU lane -> GPU_DEFERRED (never enqueue, never "complete the impossible") OR GPU_ESCALATED (proceed, flip model_tier->PAID). Default-defer. Does NOT wire the F:-host pod-lifecycle MCP into the distro (D-12).
 - the `headless` conductor verb = conductor.py:621 run_headless wrapper -> headless_runner.run_headless. CONFIRMED wired (conductor.py:621-651 + an add_parser block).
 - TEMPLATE_REGISTRY: finish-elai (feature-map), finish-elai-backlog (full corpus adapter), gpu-train-elai (GPU hook). These are ELAI-NAMED templates (project_id="elai" default) -- a generality flag (see section 6).

HOW free_api_worker SPAWNS base-cc: it does NOT spawn directly; it returns (command, prompt) to worker_adapter, which the WorkerProvider runs with cwd=worktree, stdin=prompt, env=free_api_env(...). The provider is the existing subprocess runner; the free-api lane is purely a command-builder + env-shaper + result-parser triplet.

EXACT EXTENSION POINTS to make backend selectable per pool (claude / devin-free / free-api) by varying base_url+auth+model ONLY:
 - worker_adapter.py:436 _build_worker_command -- the dispatch switch (codex -> build_codex_command; free-api -> build_free_api_command; else -> build_devin_command). ADD a base-cc-via-bridge branch here (devin-free harness -> build_free_api_command-shaped base-cc command, model = a devin wire id, base_url = the bridge :8810). The command shape is IDENTICAL to free-api (base-cc --print); only the model alias + base_url + the sanitized system prompt differ.
 - worker_adapter.py:583 _worker_env -- the env switch (codex CODEX_HOME; free-api free_api_env; else DEVIN_*). ADD the devin-free-via-bridge env (ANTHROPIC_BASE_URL=http://127.0.0.1:8810, ANTHROPIC_AUTH_TOKEN per the bridge, plus DEVIN_TOOL_DESC=generic + DEVIN_SYSTEM_FILE per the content-filter caveat). 
 - the cleanest generalization: PROMOTE free_api_worker into a base-cc worker with a BACKEND parameter {claude: no base_url override + OAuth; devin-free: base_url=:8810 + bridge auth + sanitized system; free-api: base_url=:4000 + proxy master key}. build_free_api_command is ALREADY backend-agnostic (it only varies --model); only free_api_env needs the per-backend base_url/auth fork. claude = the degenerate case (no base_url override, OAuth, no API key pop). 
 CRITICAL CONSTRAINT: worker_adapter.py is 994 lines. Adding the bridge branch + the codex paid lane risks crossing the 1000-line MR-1 BLOCK boundary. The bridge env/command belong in free_api_worker.py (renamed/extended to base_cc_worker.py) or a new base_cc_backends.py, NOT inline in worker_adapter. This is a numbered executor constraint.

THE BRIDGE IS NOT YET WIRED: grep for 8810|bridge|devin_proxy|DEVIN_TOOL_DESC|DEVIN_SYSTEM_FILE across ~/teikoku/conductor/*.py returns ZERO hits in worker code (only gate_packs.py mentions a Unity "bridge", unrelated). The devin lane TODAY (worker_adapter.py:409 build_devin_command) still invokes devin.exe (DEFAULT_DEVIN_BIN = C:\Users\user\AppData\Local\devin\cli\bin\devin.exe, worker_adapter.py:27). So "wire the bridge as the devin backend" is GENUINELY UNBUILT P4-B work (unlike the wave loop).

================================================================================
5. WAVE LOOP <-> ENGINE WIRING
================================================================================
HOW conductor.py DRIVES WORKERS TODAY:
 - CLI verbs (conductor.py): run-once -> run_loop_once -> run_task_loop (:372); run-wave -> run_wave_loop (:387); run-backlog -> run_backlog_waves (:414); loop-until-dry -> loop_until_dry (:441, temporal backbone optional); headless -> headless_runner.run_headless (:621); schedule frontier -> run_schedule (:304); integrate task -> run_integrate (:336).
 - WORKER DISPATCH PATH: run_task_loop (conductor_loop) -> WorkerAdapter.from_conductor_config(config, provider) (:215) -> adapter.dispatch(task, slot_id, worktree_path, branch_name, lease_id, db_path) -> _build_worker_command (the codex/free-api/devin switch) -> the WorkerProvider runs the subprocess.
 - THE run_gates SEAM: run_task_loop VERIFY stage calls run_gates(task_id, worktree_path, base_commit, db_path, workflow_root, repo_root, artifact_root) directly (conductor_loop :263) with packs threaded; the SAME run_gates is the integrator's gate_runner. (slice_driver.py is a SEPARATE P0.5/P3 vertical-slice driver, not the wave-loop worker path; do not confuse it.)
 - update_task_status: ControlDB.update_task_status is called by the loop on PASS->VERIFIED (:276) and by the disposition path. The P4-A wake/spawn hooks: update_task_status emits a best-effort DONE event into wake.py's notify-hub (P3-D), and spawn.enqueue_spawn is the headless batch enqueue path.
 - P4-A wake/spawn: wake.py (notify-hub on slice_api :8787, /events/wait + /events/notify) + the dep-wait integration inside run_wave_loop (teikoku conductor_loop :509-529, _dep_blocked_ready_task + wake.wait_for_dep).

WHERE THE WAVE LOOP PLUGS IN SO run_headless CAN DRIVE IT: ALREADY WIRED. run_headless's drain paths call the wave loop directly: _drain_waves -> run_backlog_waves / loop_until_dry (headless_runner :_drain_waves); _drain_spawned -> per-id run_task_loop (headless_runner :_drain_spawned, promotes PLANNED->READY first). So the headless front-end already drives the wave loop. No new seam needed; the seam exists and is exercised. The only wiring gap is the BACKEND (bridge) the workers spawn, not the loop->headless connection.

================================================================================
6. GENERALITY SPLIT TABLE (the contract for the BLOCKING generality gate)
================================================================================
piece                                   | disposition
----------------------------------------+--------------------------------------------------
wave loop (run_task_loop/wave/backlog)  | AGNOSTIC -- ALREADY ported + parameterized (project via _resolve_loop_project, packs via active_packs). VERIFY only.
disposition brain (record_worker_result)| AGNOSTIC -- ALREADY ported, unchanged. VERIFY only.
pool_concurrency envelope               | AGNOSTIC -- reads workers.toml (per-project measured), no ELAI refs. DONE.
SerializedIntegrator                    | AGNOSTIC -- ALREADY ported + pack-aware. DONE.
epoch / _compact_epoch                  | AGNOSTIC (artifact_root config). DONE.
leases / budgets / scheduler frontier   | AGNOSTIC. DONE.
free_api_worker (base-cc command/env)   | AGNOSTIC mechanics; the proxy URL :4000 + master-key path are config/env, not ELAI. DONE (pending audit).
guard.py (codex donor)                  | AGNOSTIC -- HARVEST as-is (prompt/output safety scanner). NOT in F:\conductor; lives in codex-vm (out of scope here).
codex-exec paid worker (donor)          | AGNOSTIC mechanics -- HARVEST as a sibling WorkerProvider. NOT in F:\conductor (codex-vm). UNBUILT in teikoku.
devin BRIDGE backend (:8810)            | AGNOSTIC mechanics -- BUILD NEW in teikoku (base_url+auth+sanitized-system per pool). UNWIRED today.
----------------------------------------+--------------------------------------------------
conductor_config.py DEFAULT paths       | ELAI-COUPLED -> PARAMETERIZE. Defaults are "F:\\.ELAI_workflow" + "...\\.elai_cc" (conductor_config.py:155-159). Already overridden per-project via _resolve_loop_project, but the DEFAULTS are reachable when project_id is None. For the generality gate, the demo project MUST pass project_id so resolve_project("demo") overlays its repo_root. RISK: a code path that constructs ConductorConfig.from_file without a project overlay falls back to the F: ELAI defaults. AUDIT must prove no general path hits the default.
gate COMMAND SET                        | ELAI-COUPLED -> PARAMETERIZED via gate_packs (rust/elai vs generic vs unity). DONE; demo uses generic.
backlog importer                        | ELAI-COUPLED -> PARAMETERIZED. backlog_import.py is --project + corpus_root routed; parse_corpus_for_project routes ELAI letter-plan grammar (packs has elai) vs parse_generic_corpus (generic). DONE.
WAVE definitions (waves.py)             | ELAI-SHAPED VOCABULARY, NOT parameterized. IMPLEMENT/WIRE_SWEEP/AUDIT/POLISH are ELAI plan-stage names, flat module constants, identical in both trees. FUNCTIONALLY generic (a stage pipeline). CONTROLLER MUST RULE whether this counts as reachable ELAI hardcode. LEAN: acceptable (pipeline-stage names, no paths/grammar/gates leak); flag for the gate.
headless TEMPLATE_REGISTRY              | ELAI-NAMED (finish-elai / finish-elai-backlog / gpu-train-elai, project_id="elai" default). The free-text + --project path is fully generic, but the built-in templates are ELAI presets and run_headless defaults resolved_project to "elai" when no project/template given. For the generality gate, a demo headless run MUST pass project_id="demo" + free_text (not a finish-elai template). MILD COUPLING: a registry of ELAI presets is fine IF the generic path doesn't require them; verify the default-"elai" fallback is never hit on the demo path. DEFAULT_ELAI_FEATURE_MAP (/mnt/f/...ELAI-FEATURE-MAP.md, headless_runner) is the hardcoded fallback when the registry has no feature_map -- a demo project with no feature_map would fall back to the ELAI map. PARAMETERIZE: resolve_feature_map_path should NOT default to the ELAI map for a non-elai project (it should return empty/skip). This is a real generality finding for the executor.

================================================================================
7. TEST SURFACE
================================================================================
EXISTING teikoku tests covering the wave loop / integrator / workers (~/teikoku/conductor/tests/):
 - test_single_worker_loop.py -- run_task_loop SELECT->...->RECORD cycle.
 - test_wave_orchestration.py -- run_wave_loop / run_backlog_waves / loop_until_dry.
 - test_scheduler_integrator.py (19308 B) -- the disposition brain + SerializedIntegrator (the lock, the gates, conflict/post-gate-fail paths).
 - test_p3a_pack_concurrency.py (14812 B) -- pool_concurrency + per-pool cap + per-pool cooldown (the composed disposition+envelope behaviour).
 - test_p3d_wake.py / test_p3d_wake_loop.py -- the dep-wake integration in the loop.
 - test_worker_adapter.py (15373 B) -- dispatch / command builders / env (codex + free-api + devin branches).
 - test_backlog_adapter_general.py -- THE GENERALITY FIXTURE: resolve_project("demo") asserts corpus_root does NOT contain ".ELAI_workflow" and ends with "/docs". This is the seed of the generality proof.
 NOTE: NO test currently covers headless_runner.run_headless or free_api_worker end-to-end (the P4-B held code is UNAUDITED). NO test covers the devin bridge backend (unbuilt).

NEW TESTS THE EXECUTOR MUST ADD:
 1. free_api_worker / base-cc backend unit: build_free_api_command shape + free_api_env (BASE_URL/AUTH set, API_KEY popped) per backend (claude/devin-free/free-api). For the bridge backend: env has BASE_URL=:8810 + DEVIN_TOOL_DESC=generic + DEVIN_SYSTEM_FILE set.
 2. _build_worker_command routing: harness=devin-free -> the base-cc-via-bridge command, NOT devin.exe (proves devin.exe retired).
 3. headless_runner: run_headless free-text intake -> enqueue -> drain (dry_run_worker) reaches RECORD; GPU-defer (gpu_required + no GPU -> GPU_DEFERRED, not enqueued); GPU-escalate (-> PAID).
 4. codex paid lane (once harvested): build_codex_command + the account-a/b profile env (CODEX_HOME per account).
 5. THE GENERALITY PROOF (the BLOCKING gate): drive run_headless (or run_backlog_waves) end-to-end on the `demo` pack with project_id="demo" + a generic free-text task, asserting (a) the generic gate set runs (no cargo/elai.map/traced), (b) NO ".elai_cc" / "thefinalglance" / "elai.map" / F:\.ELAI_workflow path is REACHABLE in the run (assert against the resolved config repo_root + the emitted events + the feature-map resolution), (c) resolve_feature_map_path("demo") does NOT return the ELAI map, (d) the wave names are the only ELAI-shaped strings and they are pipeline vocabulary (or, if the controller rules them out, a generic wave alias set). This extends test_backlog_adapter_general.py from a resolver assertion to a full-loop assertion. The P3 demo fixture must not regress.
 6. model_tier threading regression (section 1 bug): assert _lease_from_dict + the scheduler lease dict still carry model_tier so FREE/PAID disposition is correct.

================================================================================
STATUS LINE
================================================================================
STATUS: DONE
FILES TOUCHED: F:\.ELAI_workflow\workflow\reasoning\teikoku-p4b-context-conductor.md (this report, authored)
VERIFIED: Read ancestor conductor_loop.py (728L) + integrator.py (429L) in full; diffed conductor_loop.py / integrator.py / scheduler.py / waves.py ancestor-vs-teikoku via wsl diff (waves identical; loop+integrator+scheduler teikoku = additive superset); read teikoku free_api_worker.py (206L) + headless_runner.py (454L) in full; grepped worker_adapter.py dispatch/env seams + conductor.py verbs + conductor_config.py defaults + backlog_import.py + pool_concurrency.py; confirmed bridge :8810 UNWIRED (zero hits in worker code), devin.exe lane live (worker_adapter.py:27,409), worker_adapter.py at 994 lines (MR-1 risk), projects.toml has elai+demo rows. UNVERIFIED: codex-vm /root/.conductor-codex codex-exec adapter + guard.py (out of scope, not probed); whether the model_tier drop (section 1) is a live bug vs sourced elsewhere (flagged for executor); the bridge's exact auth/system-prompt contract (devin-bridge files not read this task).
NOTES: HEADLINE -- the wave loop / disposition / integrator are ALREADY ported+generalized (P3); P4-B's real unbuilt work is the bridge backend + codex harvest + auditing the held workers + closing the model_tier delta + the feature-map/template ELAI-default generality findings. waves.py ELAI-named wave constants + the headless ELAI templates are the two generality-gate judgement calls for the controller.
