PHASE 4A IMPLEMENTATION AUDIT — Agent Process Isolation (Tier C)
Auditor stance: HOSTILE
Date: 2026-04-25
Spec: docs/thefinalglance/summary/phase-4-summary.md L5-L188
Codebase root: F:\.ELAI_workflow\.elai_cc\crates\

OVERALL VERDICT: PARTIAL — wire format + IPC types + agent binary + sandbox CODE are real; PRODUCTION WIRING is hollow. The orchestrator does spawn elai_cc_agent as a separate OS process, but with NO isolation (no namespaces, no seccomp, no AppContainer, no rlimits, no bind-mounts) and NO live IPC server (no UnixListener/NamedPipe accept loop). The IpcGateway and AgentSandbox classes exist but are exercised only by tests. Tier B (sandbox) and Tier C (orchestrator-mediated IPC) are NOT enforced at runtime. What ships today is "Tier A++": separate process exec, but the child shares the parent's namespaces, syscalls, network, /proc, env, and filesystem.

================================================================================
4A.1 SKELETON — agent binary + IPC types — VERDICT: SUBSTANTIVELY IMPLEMENTED
================================================================================

EVIDENCE FOR:
- Two binaries compile: elai_cc and elai_cc_agent. Crate at crates/agent/, binary name asserted in crates/agent/Cargo.toml L9.
- Shared IPC crate: crates/ipc-types/src/lib.rs (444 lines). Comment L3-5: "exists to break the circular dependency between elai_cc_agent (which must NOT depend on elai-orchestrator)".
- Wire protocol: 1 byte version + 4 byte LE length + postcard payload. Constants at ipc-types/src/lib.rs L35-55: IPC_PROTOCOL_VERSION=1, MAX_FRAME_SIZE=10 MiB, FRAME_HEADER_SIZE=5. Encode/decode at IpcFrame impl L194-256.
- Per-type size limits L41-43: MAX_AGENT_HEARTBEAT_SIZE=1024, MAX_THINK_STEP_SIZE=64KB, MAX_TOOL_RESULT_SIZE=MAX_FRAME_SIZE.
- Heartbeat constants L46/L49: AGENT_HEARTBEAT_INTERVAL_SECS=5, AGENT_HEARTBEAT_TIMEOUT_SECS=15.
- Exactly 15 IpcMessageType variants enumerated at ipc-types/src/lib.rs L141-157 — AgentRequest, AgentResponse, InferRequest, InferResponse, ToolRequest, ToolResult, BashRequest, BashResult, GroundingRequest, GroundingResult, ThinkStep, CheckpointSync, AgentHeartbeat, AgentKill, ReplanRequest. Compile-time variant tagging fn _ipc_variant_tag at L161-179, IPC_MESSAGE_TYPE_VARIANT_COUNT=15 at L182.
- AgentDispatchConfig struct at L125-134 with role, model_preference, tool_filter, rule_domains, grc_budget_max_calls, grc_budget_max_chars, sandbox_config — all required fields present.
- ToolFilter at L110-119 with allows() check.
- ProcessConfig at L99-104 with allowed_paths, seccomp_profile, resource_limits.
- Agent dependency footprint (crates/agent/Cargo.toml L12-17): only serde, postcard, tracing, uuid, elai-ipc-types. Zero reqwest/network/providers/tools/orchestrator deps. Block-3 ("agent binary has NO LLM client") satisfied.
- Agent uses UnixStream (crates/agent/src/main.rs L186-200) on unix and named-pipe-as-file (L202-219) on windows. Sends version byte implicitly via frame, heartbeats every 5s (L226-240).

EVIDENCE AGAINST:
- The agent binary's "GRC reasoning loop" is a stub. handle_message at agent/src/main.rs L108-172 only ACK-replies AgentRequest with "task acknowledged: {task}" (L119-122). No InferRequest is ever issued by the agent; no ThinkStep emission; no ToolRequest emission; no GroundingRequest emission. The spec L31-36 requires "GRC reasoning loop, IPC client, structured output parser" — only the IPC client (frame I/O + heartbeat) is real.
- SO_PEERCRED handshake: code exists (orchestrator/src/ipc_gateway.rs L605-647) but is never called from production code path. No version-handshake byte in agent connect (the version is per-frame, not a separate handshake byte as L38 of spec describes "version handshake byte; mismatch -> kill").
- "Listen-before-spawn" sequence (orchestrator creates 0700 dir, listens, spawns) is partially present: create_session_socket_dir (ipc_gateway.rs L654-663) sets mode 0o700 on Unix, but no UnixListener::bind / accept loop exists anywhere in src/. grep "UnixListener|NamedPipeServer|tokio::net::UnixStream" across crates/orchestrator/src/ returned ZERO matches.

VERDICT 4A.1: types and wire format implemented to spec; agent runtime behavior is a connect-and-acknowledge skeleton; production listener/handshake not wired.

================================================================================
4A.2 VERIFY — compilation passes — VERDICT: ASSUMED PASSING
================================================================================

EVIDENCE: phase_4A.toml L9-12 asserts workspace_check_passes=true, agent_binary_compiles=true, agent_binary_zero_reqwest_deps=true. Auditor did not re-run cargo. Trust level: medium (metrics file is self-reported, no CI artifact attached).

================================================================================
4A.3 TEST functional isolation — VERDICT: TESTS EXIST, CONTRACT MISMATCH
================================================================================

EVIDENCE FOR:
- crates/orchestrator/tests/agent_isolation.rs has 38 #[test] functions (wc -c grep). phase_4A.toml claims 27 passing.
- Tests cover: GRC budget per-topic, replan budget+dedup, grounding role-method matrix, frame round-trip, heartbeat timeout, ToolFilter enforcement, env sanitization (AgentEnvironment).

EVIDENCE AGAINST (HOSTILE):
- Most "isolation" tests assert struct-field invariants, not actual isolation. Example: test_4a_isolation_pid_namespace (agent_isolation.rs L148-155) just constructs `LinuxSandboxHandle { child_pid: Some(12345) }` and asserts the field is readable. It does NOT spawn an agent, NOT read /proc/{pid}/status, NOT verify NSpid==1.
- test_4a_isolation_path_access (L137-146) asserts SelfModificationLocks::is_locked() returns true for a path that was just added — string match, no filesystem-level RO mount verification.
- test_4a_isolation_network_blocked checks that AgentDispatchConfig.sandbox_config.seccomp_profile == IpcBaseline; it does NOT spawn a process and call socket(AF_INET) to verify EPERM.
- ipc_simulation.rs reports 0 #[test] functions via grep — file was either deleted or restructured.
- Spec section 4A.3 calls for tests like "Agent cannot read .elai_cc/ (path not mounted, ENOENT)", "socket(AF_INET) returns EPERM via seccomp BPF", "RLIMIT_FSIZE 100MB write fails". None of these exercise a real spawned agent in a real namespace. The TDD-red-then-green cycle promised by spec L57 ("all tests in 4A.3 + 4A.4 written now, fail, implemented in 4A.5") was reduced to data-shape tests on the orchestrator side.

VERDICT 4A.3: tests labeled with the right names exist and pass, but they validate the type API, not the behavior the spec demands. This is per-spec ACCEPTANCE-criteria fudging.

================================================================================
4A.4 TEST adversarial IPC — VERDICT: PARTIALLY IMPLEMENTED
================================================================================

EVIDENCE FOR:
- adversarial_ipc.rs has 25 #[test] functions. phase_4A.toml claims 22 passing.
- Real adversarial tests exist: test_4a_adv_ipc_empty_payload_accepted (L103), exact-MAX_FRAME_SIZE acceptance (L126), oversized rejection, mismatched agent_id, AgentKill from agent rejected, replan dedup, etc.
- Wire-protocol fuzzing (oversized frame -> rejected before alloc) is correctly implemented in IpcGateway::read_frame (ipc_gateway.rs L455-484): length checked BEFORE vec allocation.

EVIDENCE AGAINST:
- The Windows AppContainer adversarial tests (spec 4A.4 "Windows-specific, cfg(windows) only") exist as struct construction tests at adversarial_ipc.rs L332-333 (`WindowsAppContainerSandbox::new()`) but no test actually runs the AppContainer to verify capability checks (e.g., agent cannot read .elai_cc/ via capability denial). The test asserts the constructor doesn't panic.
- No symlink-to-locked-file test, no path-traversal test through realpath.
- Crash recovery tests (orchestrator crash mid-IPC, agent crash mid-output) cannot exist meaningfully without a real listener; the IPC gateway has no integrated socket lifecycle test.

VERDICT 4A.4: protocol-level adversarial tests are credible; OS-level adversarial tests are stubbed.

================================================================================
4A.5 IMPLEMENT — agent binary + orchestrator IPC gateway — VERDICT: HOLLOW
================================================================================

This is the load-bearing sub-step. Spec 4A.5 demands:
(a) Agent binary that runs GRC loop, emits InferRequest/ToolRequest/BashRequest/GroundingRequest.
(b) Orchestrator IPC gateway that LISTENS on a per-agent socket, accepts, verifies SO_PEERCRED, runs frame router.
(c) Linux clone(NEWNS|NEWPID|NEWNET), seccomp, prctl, bind-mounts, /proc masking, rlimits.
(d) Windows AppContainer or restricted-token+Job-object fallback.

EVIDENCE FOR:
- LinuxSandbox at crates/orchestrator/src/sandbox.rs L699-918 implements:
  * compile_seccomp_ipc_baseline (L727-834): full BPF program, x86_64 syscall allowlist, default EPERM, socket() arg-filter for AF_UNIX only.
  * compile_seccomp_network_extra (L855-909): extends baseline with AF_INET/AF_INET6 + SOCK_STREAM filter for researcher.
  * launch_agent (L934-1039): clone(CLONE_NEWNS | CLONE_NEWPID | CLONE_NEWNET | CLONE_NEWUTS | SIGCHLD), child callback runs PR_SET_NO_NEW_PRIVS then seccompiler::apply_filter then execvp.
  * mem::forget on stack/cmd/args/bpf to prevent parent dropping child memory (L1021-1025).
- WindowsAppContainerSandbox at sandbox.rs L326-693 implements:
  * CreateAppContainerProfile (L342-388, dynamic-loaded from userenv.dll).
  * create_restricted_token via CreateRestrictedToken with DISABLE_MAX_PRIVILEGE.
  * Job object (CreateJobObjectW, AssignProcessToJobObject) with active_process_limit=1, memory_limit=512MB.
  * AppContainer SID + Job + RestrictedToken combined into WindowsSandboxHandle.
  * Documented-weaker fallback path (L546-552): Job + RestrictedToken without AppContainer if profile creation fails.
- create_sandbox() factory at sandbox.rs L1074-1083 returns the right impl per OS.
- IpcGateway routing logic (ipc_gateway.rs L336-452) with cap on InferRequest.max_tokens (8192), ToolFilter check on ToolRequest, GroundingMethod role-matrix (L212-227), GRC budget per-topic per-agent (advisory), SO_PEERCRED helper (L605-647).
- SelfModificationLocks at sandbox.rs L1098+ with realpath-based is_locked.

EVIDENCE AGAINST (CRITICAL):
1. The sandbox is INERT in production. agent_pool.rs L153 sets `enable_sandbox: false` as default. Production caller dashboard/src/lib.rs L1183 EXPLICITLY sets `enable_sandbox: false` when constructing the AgentPoolBabyStepExecutor. task_runner.rs L2477 also sets `enable_sandbox: false`. The only place where enable_sandbox=true is in tests (orchestrator/tests/agent_isolation.rs L45).
2. Even when enable_sandbox=true, the launch path BYPASSES the sandbox. agent_pool.rs L272-314 launch_agent_process uses plain `std::process::Command::new(agent_binary_path).spawn()` — it does NOT call self.sandbox.launch_agent(). The sandbox handle is created and stored on AgentHandle but never used to launch the binary. So even running with enable_sandbox=true, no clone() / no AppContainer happens.
3. ipc_gateway.rs L14-21 self-admits: "Production wiring scheduled in Session wire-4A per phase-4A.txt L259-L280 ('IMPLEMENT — agent binary + orchestrator IPC gateway'). Present callers are in tests/ directories (agent_isolation.rs, ipc_simulation.rs, adversarial_ipc.rs) validating the spec contract as TDD red-phase. Do not remove: deletion would require re-implementing from scratch when wire-4A lands."
4. No UnixListener / NamedPipeServer accept loop in crates/orchestrator/src/. grep confirmed zero matches. The orchestrator does not actually receive any IPC frames from spawned agents — the spawned elai_cc_agent process tries to connect to a socket (its argv[1]), but nothing on the orchestrator side ever bound or listened. AgentPoolBabyStepExecutor (task_runner.rs L827-925) just spawns the binary and waits for exit, then maps exit_code -> Passed/Failed. The agent is a one-shot exec, not an IPC peer.
5. Bind-mounts (project source RO, scratch RW, .elai_cc/ NOT mounted) — NOT IMPLEMENTED. grep "MS_BIND|bind_mount" in sandbox.rs returns zero. The CLONE_NEWNS namespace is created but no mount() syscalls follow. The child sees the parent's full filesystem.
6. /proc masking (hidepid=2) or unmount — NOT IMPLEMENTED. No mount or umount of /proc anywhere. Spec 4A.5 explicitly required this; absent.
7. rlimits (RLIMIT_AS=512MB, RLIMIT_FSIZE=100MB, RLIMIT_NPROC=1) — NOT SET. grep "setrlimit|RLIMIT_AS|RLIMIT_FSIZE|RLIMIT_NPROC" returns zero hits in sandbox.rs (the `prlimit64` syscall number 302 is in the allowlist but never invoked). No bound on agent memory or fsize.
8. Env sanitization in production: agent_pool.rs L294 does `.env_clear().envs(env.as_env_map())` so the env IS sanitized for the spawned process — that part works without the sandbox. AgentEnvironment at sandbox.rs is wired.
9. SO_PEERCRED helper exists (ipc_gateway.rs L605-647) but is never called. Without an accept loop, there is no socket to call it on.
10. phase_complete SSB record: SsbActionType::PhaseComplete and PhaseCompletePayload exist (ssb.rs L114, L660-667) with all required fields (phase_id, agent_id, fsm_state, evidence_artifacts, commit_hashes, test_outcomes). Production write site present at session_dag.rs and task_runner.rs (grep references). Atomic durability (write SSB BEFORE model unload) — not verified end-to-end here, but the action_type and payload schema match spec.
11. Self-mod locks: SelfModificationLocks struct exists with is_locked. IpcGateway holds one. But since IpcGateway is not wired into production, the lock is enforced only in tests. There is also a self_mod_layer1a.rs that may provide a separate path (not inspected); claim is unclear.
12. ShortCoder placement (deferred to 4B per spec L168) — out of 4A scope.
13. Auditor self-discovery PROPOSED-only gate — not visible in 4A code. May live in plugins/ (out of strict 4A scope).

VERDICT 4A.5: implementation grade C. The hard parts (clone+seccomp on Linux, AppContainer on Windows, IPC routing, SO_PEERCRED, frame protocol) are CODED but not WIRED. The dashboard's BabyStepExecutor spawns elai_cc_agent as plain `Command::spawn`, with no sandbox, no namespaces, no IPC accept loop, no rlimits, no /proc masking, no bind mounts. This is functionally Tier A (in-process logic) wearing a Tier C costume (separate exec). Bind mounts, /proc masking, and rlimits are MISSING entirely from sandbox.rs even at the code level.

================================================================================
4A.6 MEASURE — performance baselines — VERDICT: NUMBERS ARE FAKE
================================================================================

phase_4A.toml exists (F:\.ELAI_workflow\.elai_cc\metrics\phase_4A.toml) with the required keys: ipc_latency_p99_ms, agent_spawn_ms, memory_per_agent_mb, overhead_vs_inprocess_pct, isolation_escape_test.

EVIDENCE AGAINST:
- ipc_latency_p99_ms = "< 0.1" with method "in-process postcard round-trip (no socket I/O)". This is NOT IPC latency — it's serialization latency. The spec target was <1ms p99 over the actual socket; that has not been measured because there is no socket.
- agent_spawn_ms = "< 1.0" with method "InProcessAgentPool::spawn() wall clock (no OS process launch)". Spec target was <100ms for OS process; this number measures HashMap insert + handle struct construction. Misleading by ~2 orders of magnitude.
- memory_per_agent_mb = "< 1.0" with method "estimated from struct sizes". Spec target was <30MB for an OS process; measured number is the rust struct, not the process RSS. No OS process was measured because no real isolated agent was spawned.
- overhead_vs_inprocess_pct = "< 5.0" — ungrounded; no benchmark presented.
- isolation_escape_test = 1.0 with method "22 adversarial IPC tests + 27 isolation tests all pass". Spec demanded "agent instructed to find .elai_cc/ must fail" — this is a runtime escape test, not a count of unit tests passing. Counting tests-pass as escape-prevention conflates contract-coverage with behavior.

VERDICT 4A.6: file exists with the right keys but every metric measures the wrong thing. The acceptance criteria as worded would block this from passing.

================================================================================
GAP REGISTER (BLOCK-N / WARN-N)
================================================================================

BLOCK-1 (seccomp two profiles): code exists, NOT applied at runtime. PARTIAL.
BLOCK-2 (researcher veth+iptables, zero capabilities): seccomp side coded; veth/iptables documented as "deployment infrastructure required" (sandbox.rs L840-850). NOT IMPLEMENTED.
BLOCK-3 (agent has NO LLM client): SATISFIED. Agent Cargo.toml is clean.
BLOCK-4 (GRC budget orchestrator-side): SATISFIED in IpcGateway; budget code in AgentConnection; tests pass. But IpcGateway is not wired, so in production the constraint is unenforced.
BLOCK-5 (sockets in 0700 /tmp/elai_cc_{session}/, SO_PEERCRED, unlink on death): create_session_socket_dir 0o700 OK; SO_PEERCRED helper unused; socket unlink on agent removal at ipc_gateway.rs L292-300. PARTIAL.
BLOCK-6 (auditor self-discovery PROPOSED-only): NOT VERIFIED in 4A audit; lives in plugins/ scope.
BLOCK-7 (Windows weaker, AppContainer preferred, restricted-token fallback documented): SATISFIED at code level (AppContainer + Job + RestrictedToken with documented-weaker fallback comment).
BLOCK-8 (self-mod locks IPC + FS + git): IPC-level via SelfModificationLocks (string + realpath); FS-level RO mount NOT IMPLEMENTED (no mount syscalls); git-level filter NOT VERIFIED in this audit.
BLOCK-9 (adversarial IPC test battery): 25 tests exist, 22 reported passing. PARTIAL — protocol fuzzing solid, OS-level escape tests stubbed.

================================================================================
ANSWERS TO AUDIT PROTOCOL QUESTIONS
================================================================================

Q: Is process isolation actually running (separate processes) or still in-process (Tier A only)?
A: SEPARATE PROCESSES, but with NO ISOLATION primitives applied. The dashboard spawns elai_cc_agent via plain std::process::Command — the child shares the parent's namespaces, syscall surface, network, /proc, filesystem, and resource limits. From the security perspective this is closer to Tier A than Tier C.

Q: Is Windows AppContainer present?
A: CODE PRESENT (CreateAppContainerProfile, restricted token, Job object, ACL deny), but UNWIRED. enable_sandbox=false in dashboard wiring. WindowsAppContainerSandbox::create() is invoked only in tests.

Q: Is the agent binary free of LLM/network/tool deps?
A: YES. crates/agent/Cargo.toml has only serde, postcard, tracing, uuid, elai-ipc-types.

Q: Does the orchestrator have a real IPC accept loop?
A: NO. No UnixListener / NamedPipeServer / accept() in crates/orchestrator/src/. ipc_gateway.rs comment L14-21 explicitly defers production wiring to a future "wire-4A" session.

Q: 15 message types?
A: YES. Exactly 15 variants in IpcMessageType (ipc-types/src/lib.rs L141-157), enforced by IPC_MESSAGE_TYPE_VARIANT_COUNT=15 const at L182 and exhaustive _ipc_variant_tag match.

Q: Wire protocol 1+4+postcard?
A: YES. FRAME_HEADER_SIZE=5 (1 version + 4 LE length), postcard payload, MAX_FRAME_SIZE=10MB, length-checked before alloc. Implemented identically in agent and orchestrator.

Q: ToolFilter enforced at IPC layer?
A: YES in IpcGateway::route_frame (ipc_gateway.rs L376-388), but only when the gateway is exercised (i.e., in tests today).

Q: AgentDispatchConfig is authoritative config source?
A: YES at the type level (ipc-types/src/lib.rs L125-134). Whether the production spawn path ever constructs and sends one — no, because no IPC accept loop.

Q: phase_complete SSB record?
A: YES, schema present (ssb.rs L660-667) with all required fields. Atomic-durability invariant (fsync before model unload) not validated in this audit.

Q: Which tier is enforced today?
A: TIER A++ (in-process logic plus a separate process exec, but no isolation primitives). The Tier B sandbox code exists; Tier C IPC types and wire format exist. Neither is wired into the dashboard's executor.

================================================================================
SUMMARY
================================================================================

Phase 4A is a "skeleton-complete, integration-pending" deliverable. Roughly 70% of the spec is coded but ~25% is wired:
- Wire format, message types, agent binary minimalism, GRC/grounding/replan logic in the gateway, AppContainer code, Linux clone+seccomp code: REAL.
- IPC accept loop, sandbox-launched agents, bind mounts, /proc masking, rlimits, end-to-end SO_PEERCRED, real OS-process metrics: NOT REAL.
- Test coverage: 65 unit tests (38+27) labelled with the right names, but most validate type APIs, not isolation behavior.
- Metrics: file present, every measurement instrumented incorrectly (in-process serialization disguised as IPC latency, struct size disguised as RSS, test count disguised as isolation-escape verdict).

A user told that "phase pre 0 to phase 4E are done" should be informed: Phase 4A's skeleton is in place but the ACCEPTANCE criteria as written ("isolation escape test: agent instructed to find .elai_cc/ must fail") cannot be met by the current code because the agent is not isolated. The orchestrator has no live IPC server. The runtime tier today is closer to A than C.

Recommended follow-up before declaring 4A truly complete:
1. Add UnixListener/NamedPipeServer accept loop to orchestrator and route frames into IpcGateway.
2. Wire AgentPool::launch_agent_process to use self.sandbox.launch_agent() instead of plain Command.
3. Add bind_mount + /proc masking + setrlimit calls in LinuxSandbox::launch_agent's child callback.
4. Re-measure phase_4A metrics with real spawned processes (RSS, p99 socket round-trip, OS spawn time).
5. Convert struct-shape isolation tests into spawn-and-verify tests (NSpid==1, socket(AF_INET) returns EPERM, .elai_cc/ access denied).
6. Default enable_sandbox=true for production wiring once 1-5 land.
