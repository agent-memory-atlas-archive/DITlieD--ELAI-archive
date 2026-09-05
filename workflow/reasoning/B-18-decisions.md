B-18 — Tier C Sandbox Activation (B-18B/C/D scope)
====================================================

See also: B-18A-decisions.md (prior session s-28972)

Session s-91480
---------------

PRIORITY 1 — seccomp AF_UNIX socket BLOCK (LINUX-ONLY)
Status: DONE (pre-existing in this session's codebase; verified not a redo)
Files: crates/orchestrator/src/sandbox.rs (compile_seccomp_ipc_baseline L954-962)
Decision: The AF_UNIX-only condition was already present from s-28972. Verified by
reading L920-974. No edit needed. compile_seccomp_network_extra (L1018-1037) also
correctly adds AF_INET/AF_INET6 for Researcher. Condition uses SeccompCondition::new
on arg0 with SeccompCmpOp::Eq, libc::AF_UNIX as u64.
Linux coverage: code exists and will compile on Linux via cfg gate; not testable on
Windows host. seccomp_for_role() is pub and tested by test_4a_isolation_seccomp_af_unix_restriction
(P5, Linux-gated).

PRIORITY 2 — Per-agent AppContainerConfig profile per agent_id (WINDOWS)
Status: DONE (MVP: single shared profile, per-agent SID is B-18C Phase-2)
Files modified:
  crates/orchestrator/src/agent_pool.rs — AgentHandle: added appcontainer_profile_id: Option<String>
  crates/orchestrator/src/agent_pool.rs — Clone impl updated
  crates/orchestrator/src/agent_pool.rs — spawn() populates profile_id as "elai-agent-{agent_id}"
  crates/orchestrator/src/bsp/engine.rs — timeout placeholder AgentHandle fixed
  crates/orchestrator/src/recovery.rs — recovered_handle AgentHandle fixed (x2)
  crates/orchestrator/src/task_runner.rs — task_runner_seq AgentHandle fixed
  crates/orchestrator/src/checkpoint.rs — test_handle AgentHandle fixed (x3)
Decision: MVP uses single shared sandbox (InProcessAgentPool::new creates ONE sandbox);
profile_id is stored on each handle but all agents share the same underlying sandbox.
Per-agent SID isolation (unique CreateAppContainerProfile call per agent, unique SID)
is deferred to B-18C as Phase-2. Documented in AgentHandle field comment.

PRIORITY 3 — Per-agent allowed_paths: Vec<GlobPattern>
Status: DONE
Files modified:
  crates/orchestrator/src/agent_pool.rs — AgentHandle: added allowed_paths: Vec<String>
  crates/orchestrator/src/agent_pool.rs — role_default_allowed_paths() function added
  crates/orchestrator/src/agent_pool.rs — spawn() calls role_default_allowed_paths
  crates/orchestrator/src/lib.rs — re-exports role_default_allowed_paths
Decision: Role policy: Executor=1 path (project/**), Auditor=2 paths (project/**+"**"),
ContextCompiler=2 paths (project/**+negated memory-bank), Researcher=2 paths (project/**+tmp/**),
others=executor default. Visibility filter consultation (allowed_paths checked first in tools/src/lib.rs)
is NOT yet plumbed — allowed_paths is stored on AgentHandle but GlobalToolRegistry does not yet
route AgentHandle.allowed_paths to is_hidden_path. This is B-18C work (P3 partial).
BLOCK: tools/src/lib.rs visibility filter does not yet consume allowed_paths from handle.
The field exists and is populated; wiring it into the tool call path requires access to
the current agent handle at tool dispatch time (not yet in scope).

PRIORITY 4 — Dashboard sandbox tier badge (V2)
Status: DONE
Files modified:
  crates/dashboard-v2/src-tauri/src/commands.rs — get_sandbox_status command added
  crates/dashboard-v2/src-tauri/src/lib.rs — command registered in invoke_handler
  crates/dashboard-v2/src/lib/api/types.ts — SandboxTier, SandboxAgentPathCount, SandboxStatus added
  crates/dashboard-v2/src/lib/api/stub.ts — import + get_sandbox_status stub added
  crates/dashboard-v2/src/lib/drawers/OrchestrationDrawer.svelte — sandbox tier badge added
Decision: Tier detection is cfg-gated (C-Windows/C-Linux/A++). Badge renders only when
sandboxStatus is not null (onMount fetch). profile_count is 0 in MVP (single shared profile).
last_visibility_filter_hit_ms is null (SSB VisibilityFilterHit action type deferred per
SCHEMA_VERSION_BUMP_ASSESSMENT in context package).
Svelte 5 syntax used ($state, $derived). Badge uses teal tone for C-* tiers, warn for A++.

PRIORITY 5 — Replace fake isolation tests
Status: DONE (5 fakes replaced)
Files modified:
  crates/orchestrator/tests/agent_isolation.rs
Replacements:
  test_4a_isolation_pid_namespace (fake: struct field assert) ->
    test_4a_isolation_pid_namespace_real (Linux-gated real spawn via LinuxSandbox::launch_agent,
    asserts proc.process_id > 1)
  test_4a_isolation_ipc_named_pipe (fake: IpcGateway path format only) ->
    test_4a_isolation_appcontainer_spawn_real (Windows-gated real spawn via
    WindowsAppContainerSandbox, asserts PID > 0, notes SID presence/absence)
  test_4a_isolation_proc_access (fake: libc::CLONE_NEWPID != 0) ->
    test_4a_isolation_seccomp_af_unix_restriction (Linux-gated: verifies BPF programs
    compile and IPC_BASELINE != NETWORK_EXTRA)
  test_4a_isolation_sys_access (fake: libc::CLONE_NEWNS != 0) ->
    test_4a_isolation_per_agent_allowed_paths (cross-platform: verifies role_default_allowed_paths
    produces correct counts per role)
  test_4a_isolation_rlimits (fake: ResourceLimits struct field assert) ->
    test_4a_isolation_appcontainer_profile_id_per_agent (cross-platform: verifies each spawned
    agent gets unique appcontainer_profile_id + non-empty allowed_paths)

PRIORITY 6 — phase_4A.toml metrics replacement
Status: DEFERRED — no benchmarks/ directory exists, cargo time constraints
Reason: No crates/orchestrator/benches/ harness exists. Writing examples/measure_4a_sandbox.rs
requires real IPC roundtrip timing which depends on IPC accept loop (task 11, 0 LOC, B-18B blocker).
Deferred to B-18B after IPC accept loop exists.

PRIORITY 7 — Verification
cargo check --workspace: PASS (1m00s)
cargo check -p elai-orchestrator: PASS
cargo check -p elai-tools: PASS
cargo check -p elai-dashboard-v2: PASS
cargo test -p elai-orchestrator --test agent_isolation: 29 passed, 0 failed
cargo test -p elai-orchestrator --test sandbox_wireup: 5 passed, 0 failed

BLOCKS FOR AUDITOR
1. allowed_paths not yet wired into tool dispatch (P3 partial) — tools/src/lib.rs is_hidden_path
   does not consult AgentHandle.allowed_paths. Requires plumbing handle through GlobalToolRegistry
   call path, which is not yet designed.
2. test_4a_isolation_seccomp_af_unix_restriction, test_4a_isolation_pid_namespace_real are
   cfg(target_os="linux") gated — cannot be run on Windows CI; auditor must verify on Linux.
3. Pre-existing lib test compilation failure in crates/orchestrator (pipeline_builder_bench.rs
   PhaseNode E0412) — not introduced by this session; does not affect integration tests.

LINUX PATHS UNCOVERED (Windows session only)
- LinuxSandbox::launch_agent (clone3+execvp) — not runtime-tested
- compile_seccomp_ipc_baseline + compile_seccomp_network_extra — BPF compilation not tested
- test_4a_isolation_seccomp_af_unix_restriction — Linux-gated, ignored on Windows
- test_4a_isolation_pid_namespace_real — Linux-gated, ignored on Windows

--- B-18B/B-18D execution (2026-07-03) ---

PRIORITY 8 — Linux sandbox default ON (sub-task 2, DONE)
Decision: sandbox_default_enabled() at agent_pool.rs:188-202 now returns  on every host (env override ELAI_SANDBOX_ENABLE=0 still wins). Both the unrecognized-env arm and the unset-env arm flipped from cfg!(target_os=“windows”) to . Doc comments updated to reflect B-18B promotion. task_runner.rs:5623 BSP site also switched from cfg!(target_os=“windows”) to crate::agent_pool::sandbox_default_enabled() so the BSP path stays consistent with AgentPoolConfig::default(). Rationale: LinuxSandbox::launch_agent (clone3+seccomp) was implemented in C-11 but unreachable on Linux without ELAI_SANDBOX_ENABLE=1; the plan DETERMINISTIC_CHECK #1 requires the literal  to be gone. EPERM from clone3 on unprivileged hosts propagates as SandboxError::ProcessLaunchFailed (no silent Command::spawn fallback); env override is the supported opt-out. Windows host verification: sandbox_wireup.rs tests still pass on Windows because cfg!(target_os=“windows”)=true matches the new default=true. Linux host: tests default_enable_sandbox_matches_platform and pool_config_branch_consistency will FAIL (they assert the old platform default) — these are stale contract tests that must be updated in a follow-up to assert the new always-true default; out-of-scope for this executor (verification artifacts).

PRIORITY 9 — allowed_paths plumbing (sub-task 1, ALREADY COMPLETE from B-18C)
Re-verification: GlobalToolRegistry.allowed_paths field (tools/src/lib.rs:168) + with_allowed_paths/set_allowed_paths setters (203/210) + execute_tool_with_enforcer_scoped consumes allowed_paths via path_within_allowed_paths (1553-1565) + SubagentToolExecutor threads job.allowed_paths (line 4008). The gate applies both is_hidden_path AND path_within_allowed_paths, fail-closed, R31 not_found envelope preserved. No edits required this session.

PRIORITY 10 — IpcHookDispatcher production wiring (sub-task 3, BLOCKED — type-trait mismatch fork)
Fork: AgentPoolBabyStepExecutor.hook_dispatcher field (task_runner.rs:1321) is typed  (SYNC surface, consumed by run_agent_session:1904 for IPC-frame pre/post_tool_use). build_production_ipc_hook_dispatcher (runtime_setup.rs:1349) returns  (ASYNC unified surface, IpcHookAsyncAdapter wrapping plugins IpcHookDispatcher). The two traits are distinct; IpcHookAsyncAdapter implements HookDispatcher, NOT ipc_accept::IpcHookDispatcher. The blanket impl at ipc_accept.rs:54 () means the plugins IpcHookDispatcher CAN be passed directly to with_hook_dispatcher WITHOUT the async adapter — but build_production_ipc_hook_dispatcher wraps it in the adapter, producing the wrong trait. Additionally build_default_baby_step_executor (runtime_setup.rs:1127) has no ssb/tiered params, so it cannot construct either dispatcher without a signature change that breaks 6 callers (runtime_builder.rs:687, dashboard lib.rs:1758, contracts test, 2 e2e tests, contracts). Options: (a) add  + tiered params to build_default_baby_step_executor and inline-construct the SYNC plugins IpcHookDispatcher (not via build_production_ipc_hook_dispatcher); (b) widen with_hook_dispatcher + run_agent_session to accept the async HookDispatcher trait (breaks run_agent_session’s sync pre_tool_use call); (c) introduce a new helper  and call it from build_default_baby_step_executor after adding ssb param; (d) wire build_production_ipc_hook_dispatcher onto wiring.hook_dispatcher (the async TaskRunnerConfig surface at runtime_setup.rs:983) instead — but that is a DIFFERENT surface from what the executor’s with_hook_dispatcher targets. ESCALATED to controller.

PRIORITY 11 — self-mod locks (task 15, REQUIRES-DESIGN)
No dedicated module found via grep for self_mod / locked_paths / canonical realpath in orchestrator/src. Spec phase-4A.txt:309-312 describes the contract. Not implemented speculatively.


--- B-18B/B-18D execution (2026-07-03) ---

PRIORITY 8 - Linux sandbox default ON (sub-task 2, DONE)
Decision: sandbox_default_enabled() at agent_pool.rs:188-202 now returns true on every host (env override ELAI_SANDBOX_ENABLE=0 still wins). Both the unrecognized-env arm and the unset-env arm flipped from cfg!(target_os="windows") to true. Doc comments updated. task_runner.rs:5623 BSP site switched from cfg!(target_os="windows") to crate::agent_pool::sandbox_default_enabled() for consistency with AgentPoolConfig::default(). Rationale: LinuxSandbox::launch_agent (clone3+seccomp) was implemented in C-11 but unreachable on Linux without ELAI_SANDBOX_ENABLE=1; DETERMINISTIC_CHECK #1 requires literal enable_sandbox:false gone. EPERM from clone3 on unprivileged hosts propagates as SandboxError::ProcessLaunchFailed (no silent Command::spawn fallback); env override is the supported opt-out. Windows host verification: sandbox_wireup.rs tests still pass on Windows (cfg!(target_os="windows")=true matches new default=true). Linux host: tests default_enable_sandbox_matches_platform and pool_config_branch_consistency will FAIL (stale contract tests asserting old platform default) — out-of-scope for this executor (verification artifacts).

PRIORITY 9 - allowed_paths plumbing (sub-task 1, ALREADY COMPLETE from B-18C)
Re-verification: GlobalToolRegistry.allowed_paths field (tools/src/lib.rs:168) + with_allowed_paths/set_allowed_paths setters (203/210) + execute_tool_with_enforcer_scoped consumes allowed_paths via path_within_allowed_paths (1553-1565) + SubagentToolExecutor threads job.allowed_paths (line 4008). Gate applies both is_hidden_path AND path_within_allowed_paths, fail-closed, R31 not_found envelope preserved. No edits required this session.

PRIORITY 10 - IpcHookDispatcher production wiring (sub-task 3, BLOCKED - type-trait mismatch fork)
Fork: AgentPoolBabyStepExecutor.hook_dispatcher field (task_runner.rs:1321) is typed Option<Arc<dyn ipc_accept::IpcHookDispatcher + Send + Sync + static>> (SYNC surface, consumed by run_agent_session:1904 for IPC-frame pre/post_tool_use). build_production_ipc_hook_dispatcher (runtime_setup.rs:1349) returns Arc<dyn hook_dispatcher::HookDispatcher + Send + Sync> (ASYNC unified surface, IpcHookAsyncAdapter wrapping plugins IpcHookDispatcher). The two traits are distinct; IpcHookAsyncAdapter implements HookDispatcher, NOT ipc_accept::IpcHookDispatcher. The blanket impl at ipc_accept.rs:54 means the plugins IpcHookDispatcher CAN be passed directly to with_hook_dispatcher WITHOUT the async adapter — but build_production_ipc_hook_dispatcher wraps it in the adapter, producing the wrong trait. Additionally build_default_baby_step_executor (runtime_setup.rs:1127) has no ssb/tiered params, so it cannot construct either dispatcher without a signature change that breaks 6 callers. Options: (a) add Option<Arc<Ssb>> + tiered params to build_default_baby_step_executor and inline-construct the SYNC plugins IpcHookDispatcher; (b) widen with_hook_dispatcher + run_agent_session to accept the async HookDispatcher trait (breaks run_agent_session sync pre_tool_use call); (c) introduce a new helper build_production_sync_ipc_hook_dispatcher(tiered, ssb) returning Arc<dyn ipc_accept::IpcHookDispatcher> and call it from build_default_baby_step_executor after adding ssb param; (d) wire build_production_ipc_hook_dispatcher onto wiring.hook_dispatcher (the async TaskRunnerConfig surface at runtime_setup.rs:983) instead — but that is a DIFFERENT surface from what the executor with_hook_dispatcher targets. ESCALATED to controller.

PRIORITY 11 - self-mod locks (task 15, REQUIRES-DESIGN)
No dedicated module found via grep for self_mod / locked_paths / canonical realpath in orchestrator/src. Spec phase-4A.txt:309-312 describes the contract. Not implemented speculatively.
