# Verification Report: B-18_tier_c_sandbox_activation.md
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-18_tier_c_sandbox_activation.md
Date: 2026-05-17
Overall status: PARTIAL
Confidence: HIGH

## Summary
B-18 is split across two sub-phases (A and B). The B-18A portion is substantially complete: Windows AppContainer launch wiring, Linux namespace flags (CLONE_NEWNS|CLONE_NEWPID|CLONE_NEWNET), seccomp-bpf IPC_BASELINE filter, prctl(PR_SET_NO_NEW_PRIVS), IPC accept loop (ipc_accept.rs), per-agent allowed_paths, visibility filter R31 flip, and sandbox tier badge in the dashboard are all present. The B-18B portion is explicitly deferred per in-code comments: rlimits, /proc hidepid masking, zero-capability drop (PR_CAPBSET_DROP), IpcHookDispatcher wiring into the IPC path, and real isolation tests that verify NSpid from /proc are not implemented. The plan's claim that phase_4A.toml metrics "measure wrong things" is now outdated — the file acknowledges measurement limitations via comment fields.

## Claims verified DONE

- enable_sandbox defaults to cfg!(target_os="windows") on all three production sites — evidence: .elai_cc/crates/orchestrator/src/agent_pool.rs:175, task_runner.rs:3643, .bkp_old dashboard (live dashboard migrated to dashboard-v2 which uses get_sandbox_status command)
- launch_agent_process wired to call self.sandbox.launch_agent() — evidence: .elai_cc/crates/orchestrator/src/agent_pool.rs:339-353
- WindowsAppContainerSandbox::CreateAppContainerProfile called per agent_id — evidence: .elai_cc/crates/orchestrator/src/sandbox.rs:390-420
- WindowsAppContainerSandbox::launch_agent implemented with restricted token + Job Object fallback — evidence: .elai_cc/crates/orchestrator/src/sandbox.rs:611-808
- Linux clone() called with CLONE_NEWNS|CLONE_NEWPID|CLONE_NEWNET|CLONE_NEWUTS flags — evidence: .elai_cc/crates/orchestrator/src/sandbox.rs:1119
- seccomp-bpf IPC_BASELINE filter compiled and applied in clone callback — evidence: .elai_cc/crates/orchestrator/src/sandbox.rs:1095, 1146
- prctl(PR_SET_NO_NEW_PRIVS) applied in child before seccomp — evidence: .elai_cc/crates/orchestrator/src/sandbox.rs:1141
- NETWORK_EXTRA seccomp profile for Researcher role defined — evidence: .elai_cc/crates/orchestrator/src/sandbox.rs:867-960 (IpcBaseline), feasibility_probe.rs:539
- UnixListener / NamedPipeServer accept loop in ipc_accept.rs — evidence: .elai_cc/crates/orchestrator/src/ipc_accept.rs:116-221, bind_session_listener at :190
- ipc_accept.rs wired into task_runner and lib.rs — evidence: .elai_cc/crates/orchestrator/src/task_runner.rs:1330, lib.rs:171,754-755
- IpcGateway::route_frame implemented and operational — evidence: .elai_cc/crates/orchestrator/src/ipc_gateway.rs:380
- SelfModificationLocks wired into IpcGateway (4-layer self-mod locks partial) — evidence: .elai_cc/crates/orchestrator/src/ipc_gateway.rs:292,562
- visibility_filtered error_code replaced with not_found / "No such file or directory" — evidence: .elai_cc/crates/tools/src/lib.rs:1415-1475
- Contract test enforces R31 not_found semantics and bans visibility_filtered string — evidence: .elai_cc/crates/contracts/tests/visibility_filter.rs:29-38
- Per-agent allowed_paths field on AgentHandle (Vec<String>) — evidence: .elai_cc/crates/orchestrator/src/agent_pool.rs:90,115,596-615,896-909
- role_default_allowed_paths function provides per-role glob list — evidence: .elai_cc/crates/orchestrator/src/agent_pool.rs:909
- Sandbox tier badge in dashboard (SandboxTier type, OrchestrationDrawer badge, get_sandbox_status command) — evidence: .elai_cc/crates/dashboard-v2/src/lib/api/types.ts:1373-1383, OrchestrationDrawer.svelte:122-231, commands.rs:4962
- ipc_accept.rs module added (plan called it ipc_listener.rs; implemented as ipc_accept.rs) — evidence: .elai_cc/crates/orchestrator/src/ipc_accept.rs:4-5, lib.rs:171
- test_4a_isolation_pid_namespace_real spawns real process via LinuxSandbox and checks process_id > 1 — evidence: .elai_cc/crates/orchestrator/tests/agent_isolation.rs:163-206

## Claims PARTIAL

- Linux rlimits (CPU 60s/90s, memory 2GB, files 256, processes 32)
  - exists: prlimit64 syscall number 302 is allowed in seccomp allowlist (.elai_cc/crates/orchestrator/src/sandbox.rs:937)
  - gap: no setrlimit/prlimit64 calls present in sandbox.rs or ipc_accept.rs; only allowlisted in seccomp (agent could self-impose them but orchestrator does not enforce them). B-18B+ deferred per sandbox.rs:1087 comment.

- /proc masked with hidepid=2 inside agent mount namespace
  - exists: CLONE_NEWNS is included in clone flags (.elai_cc/crates/orchestrator/src/sandbox.rs:1119)
  - gap: no mount() or bind-mount call for /proc with hidepid=2 found anywhere in codebase. Mount namespace created but not populated.

- Zero Linux capabilities (PR_CAPBSET_DROP)
  - exists: PR_SET_NO_NEW_PRIVS applied (.elai_cc/crates/orchestrator/src/sandbox.rs:1141)
  - gap: PR_CAPBSET_DROP loop to drop all capabilities not present. Only new-privileges escalation blocked, existing caps not dropped.

- IpcHookDispatcher wired (dead code activated)
  - exists: IpcHookDispatcher fully implemented in .elai_cc/crates/plugins/src/ipc_hooks.rs:206, exported from plugins::lib.rs:28; contract tests in contracts/tests/plugins_hooks.rs and plugins_adversarial.rs
  - gap: IpcHookDispatcher not imported or called from orchestrator/src/ipc_accept.rs, production_agent_rpc.rs, or ipc_gateway.rs. Still dead code in the production agent RPC path. Note: orchestrator/src/hook_dispatcher.rs is a different module (SSB/tiered analysis, not IPC frame interception).

- phase_4A.toml real metrics
  - exists: phase_4A.toml is present at .elai_cc/metrics/phase_4A.toml
  - gap: ipc_latency_measurement_method = "in-process postcard round-trip (no socket I/O)" and agent_spawn_measurement_method = "InProcessAgentPool::spawn() wall clock (no OS process launch)" still present verbatim as flagged in plan. Memory measured from "estimated from struct sizes". Real socket IPC latency, OS RSS, and cold spawn latency not added.

- Real isolation tests verify NSpid from /proc/{pid}/status
  - exists: test_4a_isolation_pid_namespace_real spawns /bin/true via LinuxSandbox and checks process_id > 1 (.elai_cc/crates/orchestrator/tests/agent_isolation.rs:163-206)
  - gap: does NOT read /proc/{pid}/status NSpid field to verify the process sees itself as PID 1 inside the namespace. Process ID > 1 check only verifies OS-level PID was returned, not namespace isolation. Plan required "actually spawn agent and verify isolation via /proc reads".

## Claims MISSING

- clone3 syscall (plan Tasks 1-6 title uses "clone3") — searched: clone3, sys_clone3, libc::clone3; no evidence; implementation uses libc::clone() (clone2) with CLONE_NEW* flags, not the newer clone3 API. Functionally equivalent for namespace creation but the claim is false.

## Claims UNVERIFIABLE

- Researcher dedicated bridged network namespace (plan Task 1 bullet 3) — reason: implementation uses CLONE_NEWNET for all agents (sandbox.rs:1119) but no code creates a bridged veth pair for researcher. Network namespace is created but isolated (no network), not bridged. Bridging would require post-clone ip/netns setup. Cannot confirm bridging is deferred vs missing without additional B-18B documentation.

- NtQueryInformationProcess verification on Windows (plan Task 16) — reason: no Windows-side real isolation test found; test_4a_isolation_pid_namespace_real is Linux-only (#[cfg(target_os = "linux")]); Windows isolation verification test not present. Unverifiable whether deferred to B-18B or missing.

## What must be done (action list)

- [ ] B-18B: implement setrlimit/prlimit64 calls in LinuxSandbox::launch_agent for CPU (60s/90s), memory (2GB), files (256), processes (32) — target: .elai_cc/crates/orchestrator/src/sandbox.rs LinuxSandbox impl
- [ ] B-18B: mount /proc with hidepid=2 inside CLONE_NEWNS mount namespace after clone() returns in parent — target: .elai_cc/crates/orchestrator/src/sandbox.rs clone_cb or post-clone parent setup
- [ ] B-18B: add PR_CAPBSET_DROP loop in clone_cb to drop all capabilities after PR_SET_NO_NEW_PRIVS — target: .elai_cc/crates/orchestrator/src/sandbox.rs:1141 region
- [ ] B-18B: wire IpcHookDispatcher from elai_plugins into run_agent_session frame dispatch loop — target: .elai_cc/crates/orchestrator/src/ipc_accept.rs run_agent_session
- [ ] B-18B: update phase_4A.toml with real measurements: Unix socket IPC round-trip latency, OS process RSS at steady state, cold spawn latency from clone() to first IPC handshake
- [ ] B-18B: extend test_4a_isolation_pid_namespace_real to read /proc/{pid}/status and assert NSpid != 1 (verifies PID namespace boundary) — target: .elai_cc/crates/orchestrator/tests/agent_isolation.rs:163
- [ ] B-18B: add Windows real isolation test using NtQueryInformationProcess to verify AppContainer SID present on spawned process — target: .elai_cc/crates/orchestrator/tests/agent_isolation.rs or new isolation_windows.rs
- [ ] Clarify researcher bridged network namespace: either implement veth pair setup or document as out-of-scope — target: .elai_cc/crates/orchestrator/src/sandbox.rs LinuxSandbox

## Notes

The plan refers to ipc_listener.rs as the new file to ADD; the implementation used ipc_accept.rs instead. The module is fully functional under the alternate name. The .bkp_old dashboard/src/lib.rs:1190 line referenced in the plan (Tasks 7) is a backup file; the live dashboard moved to dashboard-v2 which exposes sandbox tier via get_sandbox_status Tauri command. The clone() vs clone3() discrepancy is cosmetic — libc::clone with CLONE_NEW* achieves the same namespace isolation as clone3 for this use case, but the plan claim is technically incorrect.
