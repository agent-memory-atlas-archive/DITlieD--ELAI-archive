# Verification Report: phase-4A.txt
Source: F:\.ELAI_workflow\docs\specs\guide\steps\phase-4A.txt
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
Phase 4A agent process isolation is substantially implemented. The ipc-types shared crate,
agent binary (elai_cc_agent), IPC gateway, sandbox (Linux + Windows), seccomp-bpf profiles,
environment sanitization, and the 4A.3/4A.4 test suites are all present and verified. The
metrics file (phase_4A.toml) exists with all 5 required measurements. Key gaps: 4A.4 is
missing several named adversarial test cases from the spec (u32::MAX pre-alloc check,
SO_PEERCRED mismatch, 10K ThinkStep burst, crash/SSB recovery, symlink/traversal/BashRequest
self-mod variants, discovery PROPOSED-gate and inflated-patterns tests, MINOR/MAJOR replan
distinction, evidence field presence check); rlimits (RLIMIT_FSIZE/AS/NPROC) are absent from
the Linux sandbox implementation; and veth+iptables for researcher egress is explicitly noted
as deferred stub in the source.

## Deliverables verified DONE

- elai_cc_agent binary (separate Rust crate) — evidence: crates/agent/Cargo.toml:1, crates/agent/src/main.rs:1
- elai-ipc-types shared crate — evidence: crates/ipc-types/src/lib.rs:1
- Wire protocol: 1-byte version + 4-byte LE length + postcard — evidence: crates/ipc-types/src/lib.rs:7, 270-330
- MAX_FRAME_SIZE = 10MB — evidence: crates/ipc-types/src/lib.rs:104
- Per-type size limits (heartbeat <1KB, ThinkStep <64KB, ToolResult <10MB) — evidence: crates/ipc-types/src/lib.rs:107-109
- IpcMessageType enum with 16 variants (spec said 15; AgentMaxIterationsReached added in Phase A2) — evidence: crates/ipc-types/src/lib.rs:209-231, IPC_MESSAGE_TYPE_VARIANT_COUNT=16 at line 257
- All 15 originally-specified message types present (AgentRequest/Response/InferRequest/InferResponse/ToolRequest/ToolResult/BashRequest/BashResult/GroundingRequest/GroundingResult/ThinkStep/CheckpointSync/AgentHeartbeat/AgentKill/ReplanRequest) — evidence: crates/ipc-types/src/lib.rs:211-225
- ProcessConfig per role (allowed_paths, seccomp_profile, resource_limits) — evidence: crates/ipc-types/src/lib.rs:165-170
- ToolFilter { allowed_tools: Vec<ToolId> } with whitelist enforcement — evidence: crates/ipc-types/src/lib.rs:177-185
- AgentDispatchConfig struct — evidence: crates/ipc-types/src/lib.rs:191-200
- SeccompProfile enum (IpcBaseline, NetworkExtra) — evidence: crates/ipc-types/src/lib.rs:144-148
- agent binary has no reqwest/network/providers dependency — evidence: crates/agent/Cargo.toml (deps: getrandom, serde, postcard, tracing, uuid, elai-ipc-types, elai-core only)
- agent_id: UUID v4 assigned at spawn — evidence: crates/orchestrator/src/sandbox.rs:1064-1066
- Connection sequence (listen-before-spawn) contract — evidence: crates/orchestrator/tests/agent_pool_listen_before_spawn_contract.rs:97
- MAX_FRAME_SIZE enforced before allocation in from_wire_bytes — evidence: crates/ipc-types/src/lib.rs:301-306
- Linux sandbox: clone(CLONE_NEWNS|CLONE_NEWPID|CLONE_NEWNET) — evidence: crates/orchestrator/src/sandbox.rs:1119
- prctl(PR_SET_NO_NEW_PRIVS) before exec — evidence: crates/orchestrator/src/sandbox.rs:1140-1141
- seccomp-bpf IPC_BASELINE profile with AF_UNIX socket BPF argument check — evidence: crates/orchestrator/src/sandbox.rs:962 (syscall 41 with af_unix_only_rule)
- seccomp NETWORK_EXTRA profile for researcher (AF_INET+AF_INET6 SOCK_STREAM BPF check) — evidence: crates/orchestrator/src/sandbox.rs:976-1057
- AgentEnvironment sanitized (LANG/TERM/PATH/TMPDIR only; HOME/USER/API keys blocked) — evidence: crates/orchestrator/src/sandbox.rs:1383-1428
- Windows AppContainer sandbox (CreateAppContainerProfile, with_fallback restricted-token) — evidence: crates/orchestrator/src/sandbox.rs:1285-1295, tests/agent_isolation.rs:252-299
- SelfModificationLocks realpath comparison — evidence: crates/orchestrator/tests/adversarial_ipc.rs:253-277, crates/orchestrator/src/ipc_gateway.rs (comment at line 10)
- GRC budget externally enforced (AgentConnection tracks ThinkStep calls per topic) — evidence: crates/orchestrator/tests/agent_isolation.rs:616-753
- GroundingRequest role-method matrix — evidence: crates/orchestrator/tests/agent_isolation.rs:841-857 (Executor=FileCheck, Researcher=FileCheck+WebCheck, Auditor=all)
- Grounding orphaned-topic reject — evidence: crates/orchestrator/tests/agent_isolation.rs:873-879
- Heartbeat timeout detection (15s -> SIGKILL) — evidence: crates/ipc-types/src/lib.rs:115, adversarial_ipc.rs:311-335
- phase_complete SSB action type exists and is durability marker — evidence: crates/orchestrator/src/ssb.rs:982, 4086
- Auditor self-discovery: PROPOSED status in self_discovery.rs — evidence: crates/rules/src/self_discovery.rs:513
- Agent binary heartbeat interval constant (5s) — evidence: crates/ipc-types/src/lib.rs:112
- 4A.3 test suite: test_4a_isolation_*, test_4a_llm_proxy_*, test_4a_grc_budget_*, test_4a_grounding_* — evidence: crates/orchestrator/tests/agent_isolation.rs (all naming conventions present)
- 4A.4 partial test suite: test_4a_adv_ipc_* (oversized/truncated/version/malformed/exact-size/short), test_4a_adv_replan_* (propagated/budget/loop_guard), test_4a_adv_selfmod_* (binary/scratch), test_4a_adv_crash_* (socket cleanup/resources/supervisor), test_4a_adv_windows_* (appcontainer/path_restriction/named_pipe), test_4a_adv_auditor_* (self_discovery/cannot_spawn_executor) — evidence: crates/orchestrator/tests/adversarial_ipc.rs
- Metrics file phase_4A.toml with all 5 measurements (IPC latency, spawn time, memory, overhead, isolation escape) — evidence: crates/orchestrator/../metrics/phase_4A.toml:26-46

## Deliverables PARTIAL

- rlimits (RLIMIT_NPROC=1, RLIMIT_FSIZE=100MB, RLIMIT_AS=512MB) in Linux sandbox
  - exists: ResourceLimits struct in ipc-types/src/lib.rs:155-159 (max_memory_bytes, max_cpu_secs, max_open_files); prlimit64 allowed in seccomp baseline (sandbox.rs:937)
  - gap: setrlimit/prlimit64 calls for RLIMIT_NPROC, RLIMIT_FSIZE, RLIMIT_AS not found in LinuxSandbox::launch_agent implementation; the struct exists but OS-level enforcement is not applied in child

- veth+iptables researcher network isolation (BLOCK-2)
  - exists: NETWORK_EXTRA seccomp profile compiled (sandbox.rs:976-1057); DEPLOYMENT NOTE at sandbox.rs:980-992
  - gap: veth pair creation and iptables FORWARD rules not implemented; sandbox.rs:983-992 explicitly defers this to "deployment orchestrator (systemd unit / container runtime)"

- 4A.4 adversarial test battery — several spec-required cases missing
  - exists: 22 tests in adversarial_ipc.rs cover the IPC frame adversarial cases, replan budget/dedup, selfmod path, crash detection, Windows AppContainer
  - gap: the following spec-named adversarial tests are absent:
      - "Second process connecting to same socket -> rejected (SO_PEERCRED mismatch)"
      - "10,000 ThinkStep burst -> budget enforced, excess dropped"
      - "Mismatched agent_id in message header -> rejected"
      - "Partial frame (connection drops mid-write) -> no hang, timeout + cleanup"
      - "MINOR replan with evidence -> auditor fast-track review" (severity semantics not verified)
      - "MAJOR replan -> requires human approval (INTERACTIVE gate)"
      - "2 replans in same phase -> second rejected (max 1 per agent per phase)" (adversarial_ipc.rs tests budget=3, not the 1-per-phase spec invariant)
      - "ReplanRequest without evidence field -> rejected" (IpcMessageType::ReplanRequest has no evidence field — wire format deviation from spec)
      - "ReplanRequest evidence exceeding 500 tokens -> truncated or rejected"
      - "Path traversal (../metrics.rs) -> rejected (canonical realpath comparison)"
      - "BashRequest writing locked file -> blocked (read-only mount)"
      - "GitCommit touching locked file -> rejected"
      - "Symlink to locked file -> rejected"
      - "PROPOSED rule from auditor requires human approval before CANDIDATE"
      - "Second auditor instance validates predicate logic independently"
      - "Inflated violation_patterns (same hash repeated) -> detected, capped"
      - "Orchestrator crash mid-IPC -> restart reads SSB -> consistent state"
      - "SSB write confirmed before ToolResult sent"
      - "Partial frame in socket buffer after SIGKILL -> discarded on fd close"

## Deliverables MISSING

- ReplanRequest evidence field on the wire type — spec (4A.4 line 218-220) requires evidence field; crates/ipc-types/src/lib.rs:225 shows ReplanRequest { reason: String, severity: String } with no evidence field
  searched: ipc-types/src/lib.rs ReplanRequest struct; no evidence field found

- RLIMIT enforcement calls in LinuxSandbox::launch_agent
  searched: sandbox.rs rlimit setrlimit RLIMIT_FSIZE RLIMIT_AS RLIMIT_NPROC; only prlimit64 in seccomp allow-list, no actual setrlimit calls

- /proc masking and /sys unmounting in Linux child process
  searched: sandbox.rs proc sys hidepid mount unmount; only CLONE_NEWPID/NEWNS namespaces found; hidepid=2 or /proc re-mount and /sys not mounted not present in implementation

- Per-agent /tmp isolation (spec: "per-agent /tmp isolated")
  searched: sandbox.rs agent isolation tmp per-agent tmpdir separate; scratch dir is per-agent but socket directory isolation at namespace level not verified

## Deliverables UNVERIFIABLE

- SO_PEERCRED verification on accept() enforced at runtime (Linux only) — reason: implementation is in ipc_accept.rs async paths using tokio named pipe / unix socket; reading the full async accept loop was out of scope; the test (agent_isolation.rs:226-235) only checks socket path formatting, not actual SO_PEERCRED credential check at accept time
- "Agent is PID 1 in its namespace" — reason: test_4a_isolation_pid_namespace_real (agent_isolation.rs:163-206) does launch but skips on unprivileged CI; no CI result evidence available
- GRC convergence signal causing hard-drop (advisory vs hard-drop) — reason: metrics/phase_4A.toml:55 says "advisory (logged warning on over-budget, not hard-drop)"; spec 4A.5 says "orchestrator injects convergence signal via IPC"; whether hard-drop vs advisory matches the spec intent requires runtime confirmation

## What must be done (action list)
- [ ] Add evidence field to IpcMessageType::ReplanRequest (crates/ipc-types/src/lib.rs:225) to match spec requirement: ReplanRequest { reason, severity, evidence: Option<String> }
- [ ] Implement setrlimit calls for RLIMIT_NPROC=1, RLIMIT_FSIZE=100MB, RLIMIT_AS=512MB in LinuxSandbox::launch_agent child setup (crates/orchestrator/src/sandbox.rs, after PR_SET_NO_NEW_PRIVS)
- [ ] Add /proc hidepid=2 bind-mount or mount(MS_NOEXEC) and /sys unmount in Linux child namespace setup (crates/orchestrator/src/sandbox.rs)
- [ ] Add test_4a_adv_ipc_u32_max_length_rejected (length=u32::MAX -> PayloadTooLarge before any allocation) — currently only MAX_FRAME_SIZE+1 is tested, not u32::MAX
- [ ] Add test_4a_adv_selfmod_path_traversal (../metrics.rs canonical realpath check)
- [ ] Add test_4a_adv_selfmod_bash_write_locked and test_4a_adv_selfmod_git_commit_locked
- [ ] Add test_4a_adv_selfmod_symlink_resolved
- [ ] Add test_4a_adv_replan_minor_fast_track and test_4a_adv_replan_major_human_gate (severity semantics)
- [ ] Add test_4a_adv_replan_max_one_per_phase (1 per agent per phase, not 3)
- [ ] Add test_4a_adv_replan_no_evidence_rejected (after evidence field added to wire type)
- [ ] Add test_4a_adv_discovery_proposed_gate (auditor finding -> PROPOSED, not auto-CANDIDATE)
- [ ] Add test_4a_adv_discovery_inflated_patterns_capped
- [ ] Add test_4a_adv_crash_ssb_before_toolresult (SSB write synchronous invariant)
- [ ] Add test_4a_adv_crash_partial_frame_discard
- [ ] Add test_4a_adv_ipc_think_step_burst_10k (budget enforced on 10K burst)
- [ ] Add comment "// Covers audit BLOCK-N / WARN-N" to each adversarial test (spec 4A.4 DETERMINISTIC_CHECK)

## Notes
The spec says "exactly 15 variants" but the code has 16 (AgentMaxIterationsReached added in Phase A2). This is a deliberate protocol extension documented in ipc-types/src/lib.rs comments; it does not represent a gap. The metrics file measurements are in-process estimates (no actual OS process + socket I/O latency), which the file itself notes — this is acceptable for a stub-phase but should be updated when real agent processes run in CI.
