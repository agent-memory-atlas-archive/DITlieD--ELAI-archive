# B-18: Tier C Sandbox Activation
Objective: enable real isolation primitives (clone3 + seccomp on Linux;
AppContainer on Windows) for agent processes. Today: separate exec yes,
isolation no. Also fix visibility filter "not found" semantics
(spec R31).
Estimated effort: 2-3 sessions (cross-platform syscalls, careful testing).
Pre-requisites: B-1, B-2, B-15.
Parallel-safe with: B-19, B-20.

## Context

Per audit-implementation/phase-4A.md:
- Separate process exec YES via plain std::process::Command::new(...).spawn()
- NO isolation primitives — no clone(), no seccomp, no AppContainer,
  no rlimits, no /proc masking
- WindowsAppContainerSandbox at sandbox.rs:326-693 with real Win32 calls
  EXISTS but enable_sandbox: false defaulted everywhere
- Even if enabled, launch_agent_process never calls self.sandbox.launch_agent()
- IpcGateway types defined; NO UnixListener/NamedPipeServer accept loop in
  orchestrator
- Visibility filter returns error_code="visibility_filtered" violating
  spec R31 "not found" requirement

Effective tier today: A++ (process exec but shared everything).
Target: Tier C (real isolation).

## Reads

- audit-implementation/phase-4A.md (full sub-step audit)
- F:\.ELAI_workflow\docs\specs\guide\steps\phase-4A.txt
- crates/orchestrator/src/sandbox.rs (existing real Win32 code)
- crates/orchestrator/src/agent_pool.rs:153, 291 (enable_sandbox + spawn)
- crates/orchestrator/src/ipc_gateway.rs (types ready, no listener)
- crates/orchestrator/src/dashboard/src/lib.rs:1183, 1191 (enable_sandbox=false)
- crates/orchestrator/src/task_runner.rs:2477 (enable_sandbox=false)
- crates/tools/src/lib.rs:1336-1346 (visibility filter)

## Tasks

### Linux isolation

1. **Wire clone3 with namespace flags.** When agent_pool.launch_agent_process
   runs on Linux, use clone3 with CLONE_NEWNS | CLONE_NEWPID | CLONE_NEWNET
   instead of plain Command::spawn. The agent runs in:
   - New mount namespace (chroot-like; sees only /tmp/agent-{id}/ which has
     project source mount + minimal /lib + /usr/bin)
   - New PID namespace (PID 1 inside container; can't see other agents)
   - New network namespace (NO network access; researcher role gets
     dedicated bridged namespace)

2. **seccomp-bpf allowlist.** Apply IPC_BASELINE seccomp filter
   (allow read/write/openat/exit/sigreturn/futex etc; deny exec/socket/
   network calls). Researcher gets NETWORK_EXTRA filter (allows AF_INET).

3. **prctl(PR_SET_NO_NEW_PRIVS).** Prevents privilege escalation via
   setuid binaries.

4. **rlimits.** Per-agent CPU (60s soft, 90s hard), memory (2GB), files
   (256), processes (32).

5. **/proc masked.** Inside agent's mount namespace, /proc is mounted
   with `hidepid=2` so agent can't see other process info.

6. **Zero capabilities.** Drop all Linux capabilities post-clone via
   prctl(PR_CAPBSET_DROP).

### Windows isolation

7. **Default enable_sandbox=true on Windows.** Set the three sites to true
   (dashboard/src/lib.rs:1183, task_runner.rs:2477, agent_pool.rs:153).

8. **Wire launch_agent_process to call self.sandbox.launch_agent().**
   Currently sandbox handle is created and stored but never used. Fix.

9. **AppContainer profile per agent.** Use existing
   WindowsAppContainerSandbox::CreateAppContainerProfile per agent_id.

10. **Restricted token + Job object fallback.** When AppContainer fails
    (older Windows), use the documented-weaker fallback already coded.

### IPC accept loop

11. **Add UnixListener / NamedPipeServer accept loop.** orchestrator
    binds /tmp/elai-{session_id}.sock (Linux) or \\.\pipe\elai-{session_id}
    (Windows). Spawned agent connects to that. IpcGateway routes frames.

12. **Wire IpcHookDispatcher** (currently dead code per audit) to fire
    via IPC frame interception.

### Visibility filter R31 compliance

13. **Replace error_code="visibility_filtered" with NotFound.** Per spec
    R31, hidden paths must return error indistinguishable from genuinely
    missing files. Update tools/src/lib.rs:1336-1346 + error message
    "visibility_filtered: path is hidden" → "No such file or directory"
    (or platform-equivalent).

14. **Add per-agent allowed_paths field.** AgentHandle gains
    `allowed_paths: Vec<GlobPattern>` (currently global HIDDEN_PATH_PATTERNS
    only). Each agent role gets its own glob list per spec.

### Self-mod locks

15. **Wire 4-layer self-mod locks** (IPC + filesystem + git + dispatch).
    Auditor findings gated to PROPOSED only.

### Tests

16. **Real isolation tests.** Replace the audit-flagged fake tests
    (test_4a_isolation_pid_namespace just constructs a struct and asserts a
    field — does NOT spawn process and check /proc/{pid}/status NSpid).
    New tests actually spawn agent and verify isolation via /proc reads
    (Linux) or NtQueryInformationProcess (Windows).

17. **Real metrics.** phase_4A.toml currently measures wrong things
    (in-process postcard round-trip labeled IPC latency; struct size
    labeled memory-per-agent; HashMap insert labeled agent spawn time).
    Replace with real measurements:
    - IPC round-trip latency over Unix socket / NamedPipe
    - Agent process RSS at steady state
    - Cold spawn latency from Command::spawn → first IPC handshake

## WIRING_CONTRACT

produces_symbols:
  - clone3 wrapper for Linux process spawn
  - seccomp-bpf filter loader
  - WindowsAppContainerSandbox::launch_agent called from launch_agent_process
  - UnixListener / NamedPipeServer accept loop in orchestrator
  - per-agent allowed_paths field on AgentHandle
consumes_symbols:
  - IpcGateway::route_frame
  - IpcHookDispatcher (no longer dead)
  - tools/src/lib.rs visibility filter (NotFound semantics)
reachable_from:
  - dashboard task submit → agent spawn
  - cli → agent spawn
stub_ban:
  - enable_sandbox = false defaulted disallowed in production
  - test_4a_isolation_* tests must actually verify isolation, not struct fields
  - error_code="visibility_filtered" leaks disallowed
  - phase_4A.toml metrics measure real things
frontend_pair: sandbox tier badge in dashboard (B-6)
observable_effect:
  trigger: dashboard spawns agent
  expect_artifact: /proc/{agent_pid}/status NSpid != 1, or
                   Windows AppContainer SID present

## DETERMINISTIC_CHECK

```bash
# 1. enable_sandbox=true defaulted
rg "enable_sandbox:\\s*false" .elai_cc/crates --glob '!*test*'
# Expected: zero hits.

# 2. clone3 + seccomp present (Linux)
rg "clone3|seccomp" .elai_cc/crates/orchestrator/src
# Expected: ≥2 hits.

# 3. UnixListener / NamedPipeServer accept loop
rg "UnixListener|NamedPipeServer" .elai_cc/crates/orchestrator/src
# Expected: ≥1 accept loop.

# 4. Visibility filter NotFound
rg "visibility_filtered" .elai_cc/crates/tools/src
# Expected: zero (replaced with NotFound semantics).
```

## ACCEPTANCE

- [ ] Linux: clone3 + seccomp + prctl + rlimits applied per agent
- [ ] Windows: AppContainer enabled by default; launch_agent invoked
- [ ] IPC accept loop running; orchestrator routes frames
- [ ] IpcHookDispatcher wired (dead code activated)
- [ ] Visibility filter returns NotFound (R31 compliant)
- [ ] Per-agent allowed_paths field present
- [ ] Real isolation tests pass on Linux + Windows
- [ ] phase_4A.toml metrics measure real isolation costs
- [ ] Sandbox tier badge visible in dashboard

## Files touched

- MODIFY: crates/orchestrator/src/agent_pool.rs (launch_agent_process)
- MODIFY: crates/orchestrator/src/sandbox.rs (Linux clone3 + seccomp)
- MODIFY: crates/orchestrator/src/ipc_gateway.rs (accept loop)
- MODIFY: crates/orchestrator/src/dashboard/src/lib.rs:1183
- MODIFY: crates/orchestrator/src/task_runner.rs:2477
- MODIFY: crates/tools/src/lib.rs:1336-1346 (NotFound semantics)
- ADD: crates/orchestrator/src/ipc_listener.rs (Unix + Named Pipe)
- ADD: dependencies: nix (Linux), windows-sys (Windows-specific) — with confirmation
- MODIFY: tests/integration/isolation_real.rs (real isolation verification)
- MODIFY: .elai_cc/metrics/phase_4A.toml (real metrics)
- CREATE: workflow/reasoning/B-18-decisions.md

## Decision doc

- Linux clone3 vs unshare-then-exec
- Whether to require nix dependency on Linux only or workspace-wide
- Researcher network namespace bridge configuration

## Next

B-19 (Hook system unification).
