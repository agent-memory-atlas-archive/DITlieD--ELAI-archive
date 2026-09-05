Phase 4A.1 decisions

What: Agent binary crate (elai_cc_agent) + IPC wire protocol types (ipc_types.rs)

Key decisions:
1. ipc_types.rs is a NEW file in orchestrator/src/ — holds all 15 IpcMessageType variants, IpcFrame, ToolFilter, AgentDispatchConfig, ProcessConfig, SeccompProfile, ResourceLimits, ToolId newtype
2. Existing ipc.rs DST stub (IpcMessage, IpcGateway, IpcError) is preserved under cfg(test) gating for SimulatorRuntime coupling — no external callers exist, only ipc.rs itself uses these types
3. Agent crate at crates/agent/ produces elai_cc_agent binary — depends only on serde, postcard, uuid, tracing. Zero reqwest/providers/network deps
4. postcard 1.1.3 (verified crates.io) added to workspace.dependencies
5. ToolId defined as newtype(String) in ipc_types.rs — no existing ToolId type in workspace
6. const_assert via manual compile-time guard (const VARIANT_COUNT array trick) — no external const_assertions crate dependency needed
7. Wire protocol: 1-byte version + 4-byte LE length + postcard payload. MAX_FRAME_SIZE = 10MB

Expected test outcome:
- cargo build --workspace produces elai_cc and elai_cc_agent binaries
- cargo tree -p elai_cc_agent has 0 reqwest/providers matches
- 15 variants round-trip through postcard serialize/deserialize
- Existing orchestrator tests pass (DST stub preserved)

Unknowns:
- SeccompProfile details deferred to 4A.5 (runtime implementation)
- Connection sequence implementation deferred to 4A.5
- Platform-gated socket types (UnixListener/named pipe) are type declarations only in 4A.1

Phase 4A.2 decisions (VERIFY)

What: verify 4A.1 skeleton compiles, passes tests, no regressions

Verification results (2026-04-16):
- cargo build --workspace: PASS. Two binaries (elai_cc, elai_cc_agent) compile
- cargo test -p elai-orchestrator --lib: 904 passed, 0 failed, 0 ignored
- cargo clippy --workspace -- -D warnings: FAIL (2 pre-existing errors in elai-core, not 4A.1 code)
  - error: manually reimplementing div_ceil (elai-core)
  - error: derivable_impls on BuildCommandRegistry (elai-core)
  Both pre-existing. Not introduced by 4A.1 commits (f0d46b4, eeec9a5). Out of scope.

Pre-existing failures:
- DST: all_nine_fsm_states_have_transitions expects 9 states, actual 10. Origin: phase 3I.4. Not a 4A regression. Avoided via scoped test runs (-p elai-orchestrator --lib).
- Clippy: 2 errors in elai-core (div_ceil, derivable_impls). Pre-existing. Not blocking 4A.

Known gaps (document, do not fix):
- GAP-1 (WARN): _ipc_variant_tag() const fn is runtime-callable exhaustive match, not a true compile-time assertion. Real compile-time guard needs const ASSERT or proc-macro. The match still catches variant add/remove when tests run.
- GAP-2 (INFO): f32 NaN in GroundingResult.confidence causes PartialEq false. Tests must avoid NaN.

Gate: PASS. Zero new failures. Zero new clippy warnings from 4A.1 code.

Phase 4A.3 decisions (TDD functional isolation)

What: ~30 TDD red-phase test stubs for agent functional isolation

Key decisions:
1. Test file: crates/orchestrator/tests/agent_isolation.rs (single file, expected ~200 lines)
2. Four test categories: test_4a_isolation_*, test_4a_llm_proxy_*, test_4a_grc_budget_*, test_4a_grounding_*
3. Each test: single assert, compiles, fails at runtime (TDD red)
4. Linux-specific tests (#[cfg(unix)]): pid_namespace, so_peercred, proc_access, sys_access, rlimits, researcher_veth
5. No production code written. Tests call not-yet-existing functions via assert!(false, ...) stubs
6. Gate: cargo test --no-run -p elai-orchestrator exits 0

Expected test outcomes: all compile, all fail at runtime (red phase)

Phase 4A.4 decisions (TDD adversarial IPC)

What: ~20 TDD red-phase test stubs for adversarial IPC, crash recovery, self-modification, auditor discovery

Key decisions:
1. Test file: crates/orchestrator/tests/adversarial_ipc.rs
2. Six test categories: test_4a_adv_ipc_*, test_4a_adv_replan_*, test_4a_adv_selfmod_*, test_4a_adv_crash_*, test_4a_adv_windows_*, test_4a_adv_auditor_*
3. Adversarial IPC tests exercise existing IpcFrame::from_wire_bytes API — some may be GREEN (correct behavior already implemented). This is expected and acceptable.
4. Windows tests: #[cfg(windows)]. Linux tests: #[cfg(unix)].
5. Gate: cargo test --no-run -p elai-orchestrator exits 0

Audit findings addressed:
- BLOCK: test_4a_adv_ipc_empty_payload split into two single-assert tests (empty_payload_accepted, empty_payload_is_empty)
- WARN: test_4a_isolation_ipc_socket gated #[cfg(unix)], added test_4a_isolation_ipc_named_pipe #[cfg(windows)]
- WARN: test_4a_llm_proxy_context_limit comment corrected to reference InferRequest.max_tokens
- WARN: test_4a_adv_windows_named_pipe_ipc stub added under #[cfg(windows)]
- WARN (deferred): test_4a_isolation_rlimits uses test-local literals (1GB, 300s, 256 fds). No DEFAULT_RESOURCE_LIMITS constants exist in ipc_types.rs and adding production constants is out of scope for TDD-only session. Will add constants when implementing in 4A.5.

Unknowns:
- 4A.5 implementation scope: full connection loop, namespace setup, seccomp, AppContainer
- Which 4A.4 adversarial tests will already pass (from_wire_bytes boundary checks)

Phase 4A.5 decisions (implementation)

What: Full implementation — agent IPC client loop, orchestrator IPC gateway, process spawn, sandbox enforcement, GRC budget counting, grounding proxy, self-mod locks, heartbeat/crash detection.

Key decisions:
1. ipc_gateway.rs NEW file in orchestrator/src/ — IpcGateway struct with frame routing, GRC budget counter (HashMap<(Uuid, String), u32>), grounding method matrix, replan budget/dedup.
2. Agent main.rs: IPC client loop reads frames, dispatches to handlers, sends heartbeats on interval. Uses std::os::unix::net::UnixStream on Unix, std::os::windows::io named pipe on Windows.
3. agent_pool.rs spawn(): when enable_sandbox=true, launches agent as OS process via std::process::Command (Windows) or sandbox.launch_agent (Linux). Child process gets socket_path as argv[1].
4. sandbox.rs: two seccomp profiles — compile_seccomp_ipc_baseline() (existing, all roles) and compile_seccomp_network_extra() (researcher, adds AF_INET/AF_INET6 SOCK_STREAM). BPF arg check on socket() domain parameter.
5. Tests refactored from assert!(false) stubs to real logic using in-memory mock IPC channels for cross-platform testability. Platform-specific tests (#[cfg]) test actual OS isolation.
6. GRC budget: orchestrator counts ThinkStep per (agent_id, topic). Over-budget = logged warning (advisory, not hard-drop per BLOCK-4).
7. Grounding method matrix: Executor=FILE_CHECK, Researcher=FILE_CHECK+WEB_CHECK, Auditor=all. Enforced in ipc_gateway route_grounding_request().
8. Self-mod locks: IPC-level realpath comparison in ToolRequest/BashRequest routing. Agent binary path + workflow/ dirs locked.
9. Replan budget: max 3 ReplanRequests per agent per session. Same-reason dedup via last_replan_reason HashMap.
10. SSB write synchronous: write to SSB before sending ToolResult back to agent. No async gap.
11. Socket lifecycle: /tmp/elai_cc_{session}/ dir (0700), per-agent socket, unlinked on agent death.
12. Windows: AppContainer spawn via existing WindowsAppContainerSandbox. Named pipe IPC. Documented-weaker.

Expected test outcome: 27 agent_isolation + all adversarial_ipc tests pass.

Unknowns:
- Exact overhead_vs_inprocess_pct metric (measured in 4A.6)
- Whether SO_PEERCRED test needs root on CI

Phase 4A.6 decisions (metrics baseline)

What: Record 5 performance metrics in .elai_cc/metrics/phase_4A.toml

Key decisions:
1. Metrics: ipc_latency_p99_ms, agent_spawn_ms, memory_per_agent_mb, overhead_vs_inprocess_pct, isolation_escape_test
2. Measured via cargo bench or inline timing in tests. Values recorded as static baselines.
3. isolation_escape_test: binary pass/fail (1.0 = no escape detected)
4. Format: TOML with [[metric]] array entries

Expected test outcome: file exists with 5 entries, parseable TOML.
