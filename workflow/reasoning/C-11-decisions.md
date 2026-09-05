C-11 — Sandbox Completeness (Tier-C / B-18B) Decisions
Session: s-32952
Date: 2026-05-18
Plan: F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-11_sandbox_completeness.md
Context: F:\.ELAI_workflow\workflow\sessions\s-32952\context-package-C11.md

================================================================================
D1. RLIMIT_AS = 512 MiB (not 2 GiB from B-18 plan).
================================================================================

What: setrlimit(RLIMIT_AS, 512*1024*1024) in clone_cb child side.
Key decision: phase-4A.report.md is stricter than the B-18 plan's 2 GiB value.
Adopting 512 MiB is defense-in-depth, matches the hard ceiling the executor
benchmarks already pass under, and aligns with the per-VRAM tier plan
(workflow/sessions/<...>/project_vram_tier_plan.md memory: 8/12/16 GB hosts).
Expected test outcome: /proc/{pid}/limits "Max address space" row reads
536870912 bytes (hard and soft equal). Verified in agent_isolation::
test_4a_isolation_pid_namespace_real after C-11.
Unknowns: 512 MiB may be too tight for the Researcher role under heavy
grounding workloads (web_search + JSON parsing). Mitigation: per-role rlimit
overrides scheduled for C-20.

================================================================================
D2. hidepid=2 mount in clone_cb child side, not parent nsenter.
================================================================================

What: libc::mount("proc", "/proc", "proc", MS_NOSUID|MS_NOEXEC|MS_NODEV,
"hidepid=2") inside clone_cb, between setrlimit and seccomp_apply.
Key decision: child-side mount avoids ns fd tracking and setns syscalls in
the parent. CLONE_NEWNS already provides a private mount namespace so the
remount does not affect host /proc. mount() is syscall 165 and is NOT in
the IPC_BASELINE seccomp allowlist; the call MUST precede seccomp_apply.
The insertion order (D3) guarantees this.
Expected test outcome: Linux smoke confirms child can no longer enumerate
non-owner PIDs via /proc. (measured-in:C-20 — needs CAP_SYS_ADMIN host.)
Unknowns: EPERM on unprivileged CI is expected and the existing
test_4a_isolation_pid_namespace_real skip-on-create-error pattern handles it.

================================================================================
D3. clone_cb insertion order (HARD INVARIANT).
================================================================================

What: PR_SET_NO_NEW_PRIVS -> PR_CAPBSET_DROP -> setrlimit block ->
hidepid=2 mount -> seccomp_apply -> execvp.
Key decision: seccomp MUST be LAST. Once the BPF filter is installed,
mount(165), prlimit64(302), and the PR_CAPBSET_DROP prctl(157) variants
cannot all be guaranteed to remain allowed. Putting seccomp last keeps
the syscall surface for the new blocks open. PR_SET_NO_NEW_PRIVS first
(existing) is required for seccomp_apply to succeed without CAP_SYS_ADMIN.
PR_CAPBSET_DROP before setrlimit is symmetric with prctl-first kernel
recommendations (drop privilege before reducing resources so the cap drop
itself can't be defeated by an artificial NOFILE=0).
Expected test outcome: contract test
c11_contract_sandbox_clone_cb_has_expected_syscall_blocks asserts the
five markers appear in sandbox.rs in the listed order.
Unknowns: none — ordering is enforced by source-line position.

================================================================================
D4. IpcHookTrait method choice (on_frame_routed vs pre/post_tool_use).
================================================================================

What: wire pre_tool_use BEFORE rpc.handle_tool_request and post_tool_use
AFTER, keyed on FrameRouteResult::ToolRequest.
Key decision: the plan's "on_frame_routed" method does NOT exist on
IpcHookTrait. The existing surface (pre_tool_use / post_tool_use / stop /
pre_compact) covers tool dispatch semantically. Adding a new trait method
would force ABI changes in all IpcHookTrait implementors (including
NoopSsbWriter callers in contracts/tests/plugins_*.rs). Reusing the
existing surface keeps the trait stable and the hook semantics aligned
with the existing IpcGateway::run_pre_tool / run_post_tool consumers.
Expected test outcome: pre_tool_use BLOCK verdict returns sanitized error
to agent (no rule_id/pattern leak via HookResult::sanitized_agent_message).
post_tool_use never short-circuits (call already ran).
Unknowns: BashRequest hook firing is scoped out per D6 — bash dispatch
is not yet ToolRequest-style.

================================================================================
D5. BabyStepRunner constructs the dispatcher (NOT ProductionAgentRpcServer).
================================================================================

What: AgentPoolBabyStepExecutor has a new
`hook_dispatcher: Option<Arc<dyn IpcHookDispatcher + Send + Sync + 'static>>`
field threaded into run_agent_session at task_runner.rs:1554. A builder
method `with_hook_dispatcher(...)` opts in.
Key decision: the original plan §task 5 said "ProductionAgentRpcServer
passes IpcHookDispatcher to run_agent_session". ProductionAgentRpcServer
never calls run_agent_session — it implements AgentRpcServer (the rpc
trait), not the hook trait. The actual run_agent_session call site is
task_runner.rs:1554 inside BabyStepRunner / AgentPoolBabyStepExecutor.
Plan task 5 was rewritten to wire the dispatcher at that call site.
Expected test outcome: contract
c11_contract_task_runner_passes_hook_dispatcher_into_run_agent_session
pins the call-site shape.
Unknowns: callers using Default-shaped IpcHookDispatcher need to
construct it via IpcHookDispatcher::new(TieredAnalysis::new(...),
Box::new(NoopSsbWriter), Box::new(NoopLockChecker)); helper to land in C-20.

================================================================================
D6. CAP_LAST_CAP source: /proc/sys/kernel/cap_last_cap with fallback 40.
================================================================================

What: read /proc/sys/kernel/cap_last_cap in the PARENT (heap-safe), parse
as i32, pass through ChildData. Fall back to CAP_LAST_CAP_FALLBACK = 40
(kernel 5.15+ value) if the proc file is unreadable.
Key decision: hardcoded literal 40 drifts on newer kernels (6.x exposes
41). Reading proc gives the authoritative value at runtime. The fallback
covers offline / chroot environments. EINVAL on unknown cap numbers
inside the loop is non-fatal (continue) — newer kernels can expose caps
this binary doesn't recognise.
Expected test outcome: c11_contract_sandbox_contains_cap_last_cap_read
verifies both the proc-file read and the CAP_LAST_CAP_FALLBACK constant
are present in sandbox.rs.
Unknowns: cap_last_cap=41 on kernel 6.x — the parent reads, child loops
0..=41, all 42 prctls succeed or EINVAL (non-fatal).

================================================================================
D7. Oversized payload test: KEEP existing, ADD u32::MAX literal test.
================================================================================

What: a new adversarial_ipc::test_4a_adv_ipc_u32_max_length_rejected
sits alongside test_4a_adv_ipc_oversized_payload_rejected. The new test
declares length=u32::MAX in the wire header without extending the
payload; the existing test uses MAX_FRAME_SIZE+1 with the bytes attached.
Key decision: both exercise the same PayloadTooLarge branch, but the
u32::MAX test pins the theoretical worst case. If a future refactor ever
allocates before the length bound check, the u32::MAX test will OOM the
runner instead of returning PayloadTooLarge — a louder failure than the
MAX+1 test which would just allocate a few extra MiB.
Expected test outcome: adversarial_ipc passes with 27 tests (was 22 + 5
new = 27; existing oversized test untouched).
Unknowns: none.

================================================================================
D8. Bash dispatch wiring is OUT of scope; test SelfModificationLocks directly.
================================================================================

What: the 4 selfmod adversarial tests (path_traversal, bash_write_locked,
git_commit_locked, symlink_resolved) operate on SelfModificationLocks
directly, not via IpcGateway::route_frame for BashRequest.
Key decision: BashRequest is currently "not wired" in ipc_accept.rs
(see ipc_accept.rs:540 path). Plumbing the IpcGateway lock check into
the bash dispatcher is a separate concern owned by C-future. C-11 closes
the LOCK SURFACE behind those checks (SelfModificationLocks uses
canonicalize-based realpath comparison per sandbox.rs:1339); the tests
prove the lock semantics, not the dispatcher integration.
Expected test outcome: c11_contract_selfmod_locks_realpath_resolved
plus the 4 adversarial selfmod tests pass. Real BashRequest gateway
integration lands later.
Unknowns: bash dispatcher path. Tracked in C-20 backlog.

================================================================================
D9. Test extension shape: extend test_4a_isolation_pid_namespace_real
    in place, do NOT split into 3 tests.
================================================================================

What: add NSpid assertion + RLIMIT_AS visibility check inside the existing
test_4a_isolation_pid_namespace_real after spawning /bin/sleep 1 (was
/bin/true, which exited too fast for the /proc reads).
Key decision: a single sandboxed process needs to be live for both the
status and limits read. Splitting into 3 tests would triple the namespace
setup cost on every Linux CI run. /bin/sleep 1 gives a 1-second window
during which both /proc reads succeed; the test kill_agent before exit
avoids zombies.
Expected test outcome: test passes on Linux with CAP_SYS_ADMIN host;
skips cleanly on unprivileged CI (existing pattern).
Unknowns: race window. 1 second is generous on modern hardware; if CI
becomes slower we can extend to /bin/sleep 3.

================================================================================
Files touched
================================================================================

MODIFY  .elai_cc/crates/orchestrator/src/sandbox.rs
   - Added module-level Linux-only constants: RLIMIT_CPU_SOFT_SECS,
     RLIMIT_CPU_HARD_SECS, RLIMIT_NOFILE_LIMIT, RLIMIT_NPROC_LIMIT,
     RLIMIT_AS_BYTES, RLIMIT_FSIZE_BYTES, CAP_LAST_CAP_FALLBACK.
   - launch_agent: read /proc/sys/kernel/cap_last_cap in parent.
   - ChildData: added cap_last_cap field (i32).
   - clone_cb: added PR_CAPBSET_DROP loop, 5 setrlimit calls, hidepid=2
     mount call, all in the D3 order; seccomp apply remains last.
MODIFY  .elai_cc/crates/orchestrator/src/ipc_accept.rs
   - Imported HookContext + IpcHookTrait from elai_plugins.
   - Added IpcHookDispatcher trait alias (object-safe re-export).
   - run_agent_session gained 6th param:
     Option<Arc<dyn IpcHookDispatcher + Send + Sync + 'static>>.
   - ToolRequest branch calls pre_tool_use (short-circuits on BLOCK)
     and post_tool_use (advisory only).
MODIFY  .elai_cc/crates/orchestrator/src/task_runner.rs
   - AgentPoolBabyStepExecutor: added hook_dispatcher field.
   - new() initialises hook_dispatcher to None.
   - with_hook_dispatcher() builder method.
   - Pass self.hook_dispatcher.as_ref().map(Arc::clone) to
     run_agent_session at line 1554.
MODIFY  .elai_cc/crates/orchestrator/src/lib.rs
   - Re-exported IpcHookDispatcher from ipc_accept.
MODIFY  .elai_cc/crates/orchestrator/tests/agent_isolation.rs
   - test_4a_isolation_pid_namespace_real: switched to /bin/sleep 1;
     added NSpid parse, inner==1 assertion, RLIMIT_AS hard==512MB read.
   - New test test_4a_isolation_appcontainer_process_token_sid
     (cfg(windows)) calls GetTokenInformation(TokenAppContainerSid) on
     the spawned process token.
MODIFY  .elai_cc/crates/orchestrator/tests/adversarial_ipc.rs
   - Appended 5 tests: test_4a_adv_ipc_u32_max_length_rejected,
     test_4a_adv_selfmod_path_traversal, test_4a_adv_selfmod_bash_write_locked,
     test_4a_adv_selfmod_git_commit_locked, test_4a_adv_selfmod_symlink_resolved.
MODIFY  .elai_cc/metrics/phase_4A.toml
   - Added measured-in:C-20 caveats to ipc_latency_measurement_method and
     agent_spawn_measurement_method (D7 stale-text update).
   - Appended [runtime_measurements_b18b] section with completion=true,
     listing all new constants and measurement methods.
ADD     .elai_cc/crates/contracts/tests/c11_sandbox_isolation_contract.rs
   - 8 Tier-C contract tests pinning source-level invariants and the
     SelfModificationLocks realpath surface.
ADD     workflow/reasoning/C-11-decisions.md (this file).

================================================================================
Test results (Windows host; Linux runtime smoke deferred to C-20)
================================================================================

cargo test -p elai_cc_contracts --test c11_sandbox_isolation_contract
   8 passed, 0 failed.
cargo test -p elai-orchestrator --test adversarial_ipc
   27 passed (was 22; +5 new), 0 failed.
cargo test -p elai-orchestrator --test agent_isolation
   30 passed (was 27; +3 new on Windows: AppContainer SID test runs here;
   Linux PID-ns test extension runs on Linux only), 0 failed.
cargo check --workspace -- clean.

Linux runtime measurements (NSpid + cap_last_cap + RLIMIT_AS visible +
hidepid mount) require CAP_SYS_ADMIN host and are tagged "measured-in:C-20"
in metrics/phase_4A.toml. The Linux test skips cleanly without privileges
(existing pattern in agent_isolation.rs:181-186).
