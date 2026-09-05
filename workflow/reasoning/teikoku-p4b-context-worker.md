teikoku P4-B context: WORKER PLANE (codex paid lane + guard.py harvest + base-cc unify + concurrency)

Author: ELAI context-compiler (read-only), 2026-06-20  task_id=P4-B-CC2
Scope: read-only file reads of codex-vm /root/.conductor-codex/ + teikoku ~/teikoku/conductor + bridge facts supplied by controller.
SAFETY: nothing in codex-vm or teikoku was started/stopped/edited/git-touched. Reads only (cat/ls/sed/grep). Secrets located but NEVER read as values (auth.json/config.toml redacted-pattern).

HEADLINE FINDING (changes the P4-B risk profile)
The codex paid lane is NOT a port-from-scratch. TEIKOKU's worker_adapter.py ALREADY carries a full, real `codex` harness branch (build_codex_command + _parse_codex_notes + codex _worker_env), and migration 006 ALREADY widened the worker_slots.harness CHECK to ('devin-cli','codex','free-api') + added the `pool` column. So "include the codex paid lane now" (D3) is largely WIRE + CONFIG + a live smoke, not new machinery. The donor codex worker.py is the REFERENCE/cross-check, not the thing you copy in. guard.py IS a genuine net-new harvest (absent from all of TEIKOKU). The codex conductor stores NO gpt-5.5 context window anywhere (section 6).

================================================================================
1. CODEX-EXEC WORKER ADAPTER  (donor: codex-vm; target: teikoku already has it)
================================================================================

DONOR (codex-vm /root/.conductor-codex/conductor/worker.py, 865 lines, dated Jun 20 13:32 = freshest module):
- CLI invocation (render_codex_exec_command, worker.py:44-52):
    argv = ("codex","exec","--profile",<profile_name>,"--json","--input",<task_file>)  cwd=<per-task worktree>
  profile_name is account-a/account-b (NON-secret). Code HARD-REFUSES a path-shaped profile ("/" or "\\" -> ValueError). Note the donor passes creds via --profile; TEIKOKU passes via CODEX_HOME env (see below) -- different mechanism, same intent.
- Provider protocol (CodexCliProvider.run, worker.py:118-166): subprocess.Popen(argv) with stdout/stderr PIPE; poll communicate(timeout=0.25) loop; cancel via CancellationToken (cancel_file exists OR tasks.status==CANCELLED in db) -> terminate; timeout at invocation.timeout_seconds (= spec.lease_ttl_seconds) -> stderr "[TIMEOUT after Ns]". ProviderResult{exit_code,stdout,stderr,cancelled}; .ok = exit==0 and not cancelled; .timed_out = "[TIMEOUT" in stderr.
- Task read/result return (WorkerRunner.run_task, worker.py:217-557) -- the full per-task pipeline:
    acquire lease -> insert attempt -> set RUNNING -> create worktree off base_commit -> build ContextPacket -> store CONTEXT artifact -> render_task_payload (worker.py:55-66: {task_spec, instructions:"stateless Codex worker, edit only allowed_write_set, no commit/push/creds/gate-edit, return JSON summary; Conductor verifies"}) -> PROMPT-GUARD (block -> BLOCKED) -> store TASKSPEC artifact -> reserve token budget (block -> BLOCKED) -> CancellationToken check -> provider.run(codex exec) -> OUTPUT-GUARD (block -> BLOCKED) -> store stdout/stderr LOG artifacts -> on cancel/fail: _mutation_report -> ambiguous(touched files OR commits) -> NEEDS_REVIEW, clean -> _retry_status -> enforce_allowed_write_set + commits_since (forbidden path OR any commit -> NEEDS_REVIEW) -> store DIFF artifact -> set VERIFYING -> off-agent GateRunner.run (verify.py) -> PASS: TaskResult(status=DONE,notes="passed frozen gates; awaiting integration"), worktree VERIFIED, task VERIFIED; FAIL: _retry_status. Every branch redacts notes + emits WORKER_RUN_FINISHED event.
- Result/diff shape returned: WorkerRunResult{task_id,worker_id,attempt_id,status,worktree_path,branch_name,touched_files,verifier_verdict,artifact_ids,notes}. The DIFF is a git artifact (diff_binary vs base_ref), NOT inlined into the result. Workers NEVER commit (commits_since>0 -> NEEDS_REVIEW). Integration is serialized downstream (integrate.py), not in the worker.

THE EXACT INTERFACE a sibling TEIKOKU WorkerProvider must implement (donor contract, models.py read end-to-end):
  IN  TaskSpec (frozen dataclass, models.py:88-160): task_id, plan_id, role:Role, wave:Wave, objective, base_commit|None, allowed_write_set:tuple[str] (repo-relative, validated no-abs/no-..), context_refs:tuple[dict], production_hook:dict|None, acceptance, deterministic_check, forbidden:tuple (DEFAULT_FORBIDDEN = no frozen-edit/no-commit/no-test-weaken/no-creds), verifier:dict{commands,timeout_seconds}, lease_ttl_seconds, idem_key (sha256(task_id\0base\0role)), token_budget. .to_dict/.to_json/.from_dict provided.
  OUT TaskResult (frozen dataclass, models.py:162-210): task_id, base_commit, branch|None, touched_files:tuple, artifact_refs:tuple, status:TaskResultStatus{DONE,BLOCKED,FAILED,NEEDS_REVIEW}, notes. Plus the worktree state transition + artifact store side effects.
  This is the SAME TaskSpec-in / TaskResult-out contract the free_api_worker + devin lane already honor in TEIKOKU, so a paid codex provider slots beside them with zero new contract.

TARGET (teikoku ~/teikoku/conductor/worker_adapter.py, 983 lines -- the codex branch is ALREADY PRESENT, not a stub):
- WorkerSlot dataclass (worker_adapter.py:40-47) carries: harness (default "devin-cli"), account_id|None, codex_home|None, wsl_distro|None, model, pool, proxy_base_url (getattr-optional).
- Dispatch fan-out (_build_worker_command, :436-452): `if slot.harness=="codex": build_codex_command(...)` ; elif free_api_worker.is_free_api_harness -> free-api ; else devin.
- build_codex_command (:454-501) drives the codex lane via WSL INTEROP from the teikoku distro:
    ["wsl","-d",wsl_distro(default "codex-vm"),"codex","exec","--ephemeral","--json","--color","never","--sandbox",(workspace-write|read-only by allowed_write_set),"-c",'approval_policy="never"',"-c",'shell_environment_policy.inherit="none"',"-c",PATH set,"-c",HOME=/root,"-c",request_max_retries=0,"-c",stream_max_retries=5,"-c",stream_idle_timeout_ms=120000,"--config",f"{codex_home}/config.toml","-C",<wsl worktree>,"-"]; prompt fed via stdin.
    NOTE the divergence from the donor: TEIKOKU uses `--ephemeral ... --config <codex_home>/config.toml` + per-account CODEX_HOME, NOT the donor's `--profile account-a`. TEIKOKU's model = a per-account config.toml directory (codex_home), donor's = a named profile inside one config. Both resolve to "2 accounts = 2 config homes = 2 slots".
- _worker_env codex branch (:610-615): sets CODEX_HOME = slot.codex_home or /root/.codex-accounts/{account_id|default}, CONDUCTOR_WORKER_ACCOUNT=account_id. Keep-allowlist env only (no secret keys passed).
- _parse_codex_notes / _parse_codex_events (:632+): real codex JSON-event parser (not a stub) -> sanitized notes.

THE 2 ACCOUNT PROFILES + PROXY CONFIG the executor must replicate (value-free):
- bmapi proxy, OpenAI Responses API. From /root/.codex/config.toml (codex-vm, structure read, secrets redacted):
    model_provider="OpenAI"; model="gpt-5.5"; review_model="gpt-5.5"; model_reasoning_effort="xhigh"; disable_response_storage=true (ZDR); network_access="enabled".
    [model_providers.OpenAI] name="OpenAI"; base_url="https://bmapi.020212.xyz"; wire_api="responses"; requires_openai_auth=true.
    [tui.model_availability_nux] "gpt-5.5"=4  (a UI nudge counter, NOT a context window).
- AUTH location (NEVER copied; stays gitignored): /root/.codex/auth.json (0600, 94 bytes) holds the bmapi proxy key; /root/.codex/config.toml (0600, 454 bytes). The donor conductor.local.toml carries ONLY the non-secret refs: w1->account-a, w2->account-b, model=gpt-5.5, max_inflight=1; config.py:_walk_for_secrets HARD-REJECTS any secret-like key/value in conductor config.
- WHAT TEIKOKU NEEDS (config, not secrets): 2 WorkerSlot rows harness="codex", account_id account-a/account-b, codex_home pointing at each account's config.toml dir (donor uses a single /root/.codex; TEIKOKU's build_codex_command default is /root/.codex-accounts/{account_id} -- the executor must EITHER create those 2 per-account CODEX_HOME dirs in codex-vm OR point codex_home at the existing single /root/.codex and distinguish accounts another way). The per-account auth.json/config.toml live in codex-vm under root, stay 0600, never enter teikoku git.
  CONSTRAINT: config.py forces worker max_inflight==1 (provider-limit safety) -> 1 codex process per account, 2 accounts = 2 concurrent paid slots.

================================================================================
2. GUARD.PY  (donor: codex-vm; HARVEST AS-IS, parameterize 0 -- it is already ELAI-free)
================================================================================
Path: /root/.conductor-codex/conductor/guard.py (160 lines, stdlib re only, dated Jun 20 13:31). Bidirectional regex scanner.
WHAT IT SCANS (two directions):
- check_prompt(text) / PromptGuard.check: 17 PROMPT_INJECTION_PATTERNS over the text SENT to the model (ignore/disregard/forget previous instructions, "you are now", new system/role, system prompt, override rules, "do not tell the user", DAN, developer mode, jailbreak, simulate/pretend, "this is a test of your instructions"). weight 10 each.
- check_output(text) / OutputGuard.check: 3 families over what the model RETURNS:
    DANGEROUS_OUTPUT_PATTERNS (~30): rm -rf /, --no-preserve-root, mkfs, dd if=, format c:, del /f, git push --force, git reset --hard, git clean -fd, git checkout -- ., git restore --staged ., chmod -R 777 /, chown -R root /, sudo rm, shutdown -h now, reboot, kill -9, pkill -9, > /dev/sda, wget|bash, curl|bash, curl|sh, nc -e, python -c import socket subprocess, base64|bash, echo|base64 -d, chmod +x|bash. weight 10.
    EXFILTRATION_PATTERNS (~9): curl/wget to http(s)/ftp, scp ...@...:, nc/netcat to an IP, python -m http.server, php -S, ssh ...@..., telnet to IP. weight 7.
    INSTRUCTION_OVERRIDE_PATTERNS (~6): "system: you are now", "assistant: ignore previous", "user: ignore all", "important: ignore previous", "from now on you are". weight 8.
- Returns GuardResult{safe:bool, reasons:tuple, score:int}; .blocked = not safe.
CLEAN INTERFACE to harvest into TEIKOKU's engine as a worker pre/post hook (drop-in, contract-compatible):
  - PromptGuard().check(task_payload).blocked  -> pre-dispatch hook, BEFORE provider.run (donor wires this in worker.py:272 -> task BLOCKED on hit).
  - OutputGuard().check(provider_stdout+stderr).blocked -> post-dispatch hook, AFTER provider.run (donor wires at worker.py:356 -> task BLOCKED on hit).
  Stateful guard classes accept extra_patterns:Iterable[(re.Pattern,label,weight)] for per-pack extension.
ELAI/codex-specific hardcoding to parameterize: NONE. guard.py has ZERO ELAI paths/grammar/codex coupling -- it is pure regex over text, fully agnostic (matches the GENERALITY split: harvest as-is). It is also DEVIN/free-api-agnostic, so it covers the base-cc lanes too, not just codex. TEIKOKU has NO equivalent today (verified: no guard.py, no PromptGuard/check_prompt/check_output anywhere in ~/teikoku/conductor source; the only grep hits are vendored pip/sklearn and redaction.py byte-strings in control.db). Pure upside on a 24/7 no-human loop. The teikoku-side hook seam = worker_adapter.dispatch around the provider.run call (mirror donor worker.py:272 + :356) AND/OR free_api_worker before build/after parse.

================================================================================
3. WORKER-PLANE UNIFY DESIGN  (the executor's concrete map)
================================================================================
PROVIDER SELECTION (already the shape in TEIKOKU; generalize, do not rebuild):
  worker_adapter._build_worker_command branches on slot.harness:
    harness="claude"     -> base-cc, NO base_url override (OAuth). [NEW thin case; today claude is the controller default, not a worker harness -- add as a base-cc variant of the free-api path with no ANTHROPIC_BASE_URL set.]
    harness="devin-free" / via bridge -> base-cc, ANTHROPIC_BASE_URL=http://127.0.0.1:8810 (or WIN_HOST:8810, see networking) + DEVIN_TOOL_DESC=generic + DEVIN_SYSTEM_FILE=<clean prompt>. [the bridge IS the devin backend; reuses the free_api_worker base-cc spawn, only the env differs.]
    harness in ("free-api","anthropic-proxy") -> base-cc, ANTHROPIC_BASE_URL=http://127.0.0.1:4000 + ANTHROPIC_AUTH_TOKEN=proxy master-key, POP ANTHROPIC_API_KEY. [BUILT: free_api_worker.build_free_api_command + free_api_env, held/unaudited.]
    harness="codex"      -> codex exec via WSL interop (PAID, NOT base-cc). [BUILT: build_codex_command + _parse_codex_notes; schema-supported by migration 006.]
  MINIMAL WorkerProvider SEAM: the contract already exists -- (command:list[str], stdin_prompt:str|None) from _build_worker_command + env from _worker_env + notes from _extract_worker_notes, all keyed off slot.harness, all returning into the SAME TaskResult disposition (scheduler.record_worker_result). To add the unified base-cc backends you parameterize the free-api path by base_url/auth/model per pool; you do NOT add a new provider class. So the "unify" is: collapse claude / devin-free / free-api into ONE base-cc command builder that varies (ANTHROPIC_BASE_URL, ANTHROPIC_AUTH_TOKEN, DEVIN_TOOL_DESC, DEVIN_SYSTEM_FILE, model) per slot; keep codex as the one sibling non-base-cc provider.

DEVIN-FREE PATH specifics:
- WHERE DEVIN_TOOL_DESC=generic + DEVIN_SYSTEM_FILE get set: in the base-cc spawn ENV, i.e. the teikoku-side analog of free_api_worker.free_api_env (add a devin_bridge_env that, like free_api_env, sets ANTHROPIC_BASE_URL=<bridge>, and ADDITIONALLY exports DEVIN_TOOL_DESC=generic + DEVIN_SYSTEM_FILE=<path to clean system .md>; these are consumed by the PROXY process, so they must be on the PROXY's env if the proxy runs separately, OR on the base-cc env only if base-cc forwards them -- since the bridge is a separate proxy process, DEVIN_TOOL_DESC/DEVIN_SYSTEM_FILE/DEVIN_PROXY_MODEL belong on the PROXY's launch env, while base-cc only needs ANTHROPIC_BASE_URL + ANTHROPIC_AUTH_TOKEN). The clean DEVIN_SYSTEM_FILE replaces CC's built-in security/RE-heavy system prompt that the devin gateway content-filters (the bridge ships a clean system .md for exactly this).

WHERE THE PROXY RUNS (recommendation: run the bridge on the WINDOWS HOST, base-cc-in-distro points at WIN_HOST:8810):
- Reasoning (grounded in ~/teikoku/docs/bridge-conventions.md, read in full): (a) the bridge default reads creds from %APPDATA%\devin\credentials.toml on the WINDOWS host, which the official devin client refreshes -- a Windows-host proxy gets free credential refresh; a distro-copied proxy would need DEVIN_CREDS pointed at a path it cannot keep fresh. (b) bridge-conventions already models exactly this cross-boundary shape for Windows-native services. (c) lower friction: no copying 6 proxy files + a venv into the distro, no in-distro python service to supervise.
- CROSS-BOUNDARY NETWORKING (the flagged item, from bridge-conventions.md): this machine's WSL is DEFAULT NAT (not mirrored). So a distro worker does NOT reach a Windows :8810 on localhost. It must reach WIN_HOST:8810 where WIN_HOST is discovered AT RUNTIME (NAT-assigned, changes per WSL restart): WIN_HOST=$(ip route show default | head -1 | cut -d' ' -f3). REQUIREMENT: the Windows-side bridge must bind 0.0.0.0:8810 (not only 127.0.0.1) to be NAT-reachable from the distro, and a Windows Defender inbound allow on 8810 may be needed (diagnose connection-refused first; do NOT pre-change firewall). Add an 8810 row to bridge-conventions' RESERVED PORT TABLE (status PROVISIONAL until probed): `devin-bridge | Windows | 8810 | WSL->WIN_HOST:8810 | 0.0.0.0 on Win | PROVISIONAL`.
- ALTERNATIVE (copy into distro): if the owner prefers no Windows-host dependency, the 6 proxy files DO need copying into the distro: devin_proxy.py, devin_wire.py, account_catalog.py, replay_pb2.py, replay.proto, data\, + the clean system .md (source F:\.ELAI_workflow\docs\.source\claude-code-main\claude code build\devin-bridge\). Then base-cc reaches 127.0.0.1:8810 in-distro (no NAT hop), but DEVIN_CREDS must point at a copied credentials.toml that loses the Windows client's auto-refresh. LEAN: Windows-host proxy (lower friction + free cred refresh); flag the NAT/firewall as the one cost.
- The bridge was PROVEN 200-OK live (handoff). Do NOT re-read the bridge dir from an agent (content filter); the 6-file list above is the copy manifest IF the distro option is chosen.

================================================================================
4. CONCURRENCY / DISPOSITION HOOKS  (the controller-level cap + drop-on-429)
================================================================================
TEIKOKU's per-pool envelope is BUILT: ~/teikoku/conductor/pool_concurrency.py (D-16, 3.4). It is NOT in liberty (liberty has no parallel-cap/429 knob -- confirmed by the revision doc); it lives at the CONTROLLER level here.
- WHERE THE HARD CAP OF 2 SUSTAINED FREE WORKERS ATTACHES: pool_concurrency.pools_at_capacity(db, envelope) -> the scheduler skips slots whose pool is at cap. The cap value (max_concurrency=2 for the free pool) comes from ~/teikoku/workers.toml (owner-measured: 4 = 3h ban). Production hook (named in pool_concurrency.py docstring): scheduler.py::DependencyFrontierScheduler.schedule_frontier (per-pool cap on the acquire loop) + record_worker_result (per-pool cooldown on a 429), constructed from conductor.py::run_schedule / conductor_loop.
- The cap is a LIVE COUNT(*) over leases JOIN worker_slots GROUPED BY pool (pool_inflight_counts), compared to PoolLimit.max_concurrency. lease table = source of truth (reclaim_expired self-heals; no separate counter to drift).
- DROP-ON-429 DEGRADE: record_worker_result already re-routes a 429 (refund + READY + mark_model_cooldown); 3.4 extends it with per-pool cooldown_seconds from workers.toml (PoolEnvelope.cooldown_seconds_for, default 300s). To enforce "drop to 1 sustained free worker on a 429", the executor sets/lowers the free pool's effective max_concurrency on cooldown (the envelope read is per-tick, so a cooldown that removes a slot from the live count naturally degrades concurrency).
- WHAT TO ADD for the unified plane: the free CAP must span ALL free base-cc backends (devin-free via bridge + free-api proxy) as ONE pool, since the 2-sustained ban is a Devin-gateway/free-tier limit, not per-backend. So map devin-free + free-api slots to a shared free pool key (or two pools each capped low) in workers.toml; codex is a SEPARATE paid pool (its own cap = 2 accounts, max_inflight=1 each, no free-ban concern). The codex donor itself has NO pool concept (its config.py max_workers=2 is global) -- pool-awareness is purely a TEIKOKU layer the codex slots opt into via the slot.pool column (migration 006).

================================================================================
5. GENERALITY  (every ELAI coupling found in the harvested pieces)
================================================================================
PORT-AS-IS (agnostic, zero ELAI coupling -- confirmed by reading the source):
- guard.py: pure regex over text. ZERO ELAI/codex/path/grammar coupling. Harvest verbatim.
- The codex worker MECHANICS (render_codex_exec_command CLI shape, CodexCliProvider subprocess/cancel/timeout, the TaskSpec/TaskResult contract in models.py, WorkerRunResult): agnostic. models.py has no ELAI path -- it is generic task/result/worktree/write-set machinery.
ELAI-COUPLED in the DONOR codex conductor (must NOT be copied; TEIKOKU already parameterizes these):
- config paths hardcoded to ELAI: /root/.conductor-codex/.runtime/conductor.local.toml -> workflow_root=/mnt/f/.ELAI_workflow, project_root=/mnt/f/.ELAI_workflow/.elai_cc, plans_root=/mnt/f/.ELAI_workflow/docs/thefinalglance, tools_root=/mnt/f/.ELAI_workflow/.claude/tools. TEIKOKU resolves these per-project via project_resolver + projects.toml -> any ported codex slot must take roots from project_resolver, NOT /mnt/f.
- gate selection: the donor verify.py hardcodes the 6 ELAI .claude/tools verifiers; the donor SQL pins harness IN ('codex'). TEIKOKU runs gate_packs per the project's pack and migration 006 widened harness to ('devin-cli','codex','free-api'). A ported codex lane MUST call TEIKOKU's gate_runner/gate_packs, never the donor verify.py, and use TEIKOKU's wider harness enum (already done).
- backlog importer + wave defs: donor backlog.py imports the 383 ELAI B..K tasks with ELAI plan grammar baked in. TEIKOKU's P3-B backlog adapter is pack-routed (--project, corpus_root from registry). Do NOT copy donor backlog.py.
- the donor render_task_payload "instructions" string is project-neutral (stateless worker / allowed_write_set / no commit), so it ports; but any pack-specific objective/acceptance must come from the TaskSpec, not a hardcoded string.
NET: the only NEW agnostic harvest is guard.py (+ the donor as a behavior reference for the already-present codex branch). Every ELAI coupling lives in the donor's config/verify/backlog -- none of which you import; TEIKOKU's pack layer already supplies the parameterized equivalents. GENERALITY GATE stays satisfiable: the codex slot + guard hook + bridge env are all project-neutral; run the demo pack to prove no /mnt/f/.ELAI reachable in the general path.

================================================================================
6. MODEL WINDOWS  (gpt-5.5 context window for the P4-D migrate-at table)
================================================================================
NOT FOUND in the codex conductor config -- and this is a real gap, not an oversight to grep harder for.
- /root/.codex/config.toml stores model="gpt-5.5" / review_model="gpt-5.5" / model_reasoning_effort="xhigh" but NO context_window / max_context / window field. `[tui.model_availability_nux] "gpt-5.5"=4` is a UI availability-nudge COUNTER, NOT a token window.
- /root/.conductor-codex/.runtime/conductor.local.toml + configs/conductor.example.toml store only model="gpt-5.5" (no window).
- A full grep across the donor conductor/ + configs/ + .runtime for context_window|window|max_context|260000|200000|400000|migrate returned ZERO window values (only budgets.py token-budget "window_start_ms" plumbing, unrelated to context size).
- ROOT CAUSE: the codex conductor is a FINISH-LOOP, not a chat -- it has NO auto-migration / context-window logic at all (no migration module, no per-turn window budgeting). gpt-5.5's window simply isn't a fact this codebase needed to store.
- CONSEQUENCE for P4-D (revision doc :131-141, the migrate-at table): the gpt-5.5 row CANNOT be sourced from the codex config. It must be obtained from the bmapi proxy / OpenAI Responses API model metadata (a live /v1/models or provider doc lookup at port time) OR set as a deliberate owner override like claude=200k. UNVERIFIED here; flag as the one TODO the revision doc already anticipated ("confirm the gpt-5.5 context window from the codex conductor config" -> the config does not carry it; re-source it from the proxy/provider). Source of the absence: /root/.codex/config.toml (no window key) + /root/.conductor-codex/.runtime/conductor.local.toml:20,27 (model only).

================================================================================
SOURCES CONSULTED
================================================================================
codex-vm (read-only): /root/.conductor-codex/conductor/{worker.py,guard.py,config.py,models.py}; /root/.conductor-codex/.runtime/conductor.local.toml; /root/.conductor-codex/configs/ (ls); /root/.codex/ (ls + config.toml structure, secrets redacted, auth.json NOT read); grep window/gpt-5 across conductor+configs+runtime.
teikoku (read-only): /home/user/teikoku/conductor/{free_api_worker.py,pool_concurrency.py,worker_adapter.py (sed ranges 436-520,583-700 + grep),migrations/{001_initial.sql,006_worker_pool_and_harness.sql}}; /home/user/teikoku/docs/bridge-conventions.md (full).
F: (Read): TEIKOKU-P4-REVISION-2026-06-20.md; handoff/teikoku-build.md HEAD; teikoku-p4-explore-codex-conductor.md; .claude/agents/context-compiler.md; .claude/rules/agent-dispatch-preamble.md.
Bridge interface: supplied by controller (devin_proxy.py facts; NOT read by this agent).

VALIDATION WARNINGS
HIGH: gpt-5.5 context window is NOT in any codex config -> P4-D migrate-at row must be re-sourced (proxy /v1/models or owner override); revision doc TODO stands, the config cannot satisfy it.
INFO: the codex paid lane is largely ALREADY BUILT in TEIKOKU (worker_adapter codex branch + migration 006 harness widen) -> D3 is wire+config+smoke, not new machinery; the donor codex worker.py is a reference, not a copy source. Cross-check the donor `--profile` vs TEIKOKU `--ephemeral --config codex_home/config.toml` divergence at port time (different cred mechanism; confirm one live codex turn picks up the right account auth -- UNVERIFIED, no live turn run).
INFO: guard.py is genuinely net-new to TEIKOKU (verified absent) and 100% agnostic -> clean harvest, hook at worker_adapter.dispatch pre/post provider.run.
INFO: devin-bridge networking is DEFAULT-NAT (not mirrored): distro->Windows needs WIN_HOST:8810 + 0.0.0.0 bind + maybe a firewall allow; recommend Windows-host proxy for free cred refresh. UNVERIFIED live (bridge proven 200-OK per handoff, but not over the NAT hop from the distro).
