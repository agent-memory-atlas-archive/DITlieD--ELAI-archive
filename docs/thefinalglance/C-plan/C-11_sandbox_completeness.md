# C-11: Sandbox Completeness (Tier C Isolation — Phase B-18B)
Objective: close the B-18B deferred items from B-18_tier_c_sandbox_activation.report.md and
phase-4A.report.md — setrlimit/prlimit64 resource enforcement, /proc hidepid=2 masking,
PR_CAPBSET_DROP capability drop, IpcHookDispatcher wiring into the live IPC path, real NSpid
isolation test, Windows AppContainer SID test, and 5 missing 4A.4 adversarial tests.
Estimated effort: 2-3 sessions (safety-critical syscall paths; user review required before
merging adversarial tests).
Pre-requisites: C-9 (autonomous infra — multi-agent spawn needed for smoke test executor role).
Parallel-safe with: C-12, C-13, C-14, C-15 (non-overlapping file scope).

## Context

B-18_tier_c_sandbox_activation.report.md §Claims PARTIAL (2026-05-17) identifies four gaps:
1. rlimits: prlimit64 (syscall 302) is in the seccomp allowlist (sandbox.rs:937) but no
   setrlimit/prlimit64 calls exist in LinuxSandbox::launch_agent (B-18.report:34-36).
2. /proc masking: CLONE_NEWNS set at sandbox.rs:1119 but no hidepid=2 mount call anywhere
   (B-18.report:38-40).
3. PR_CAPBSET_DROP: PR_SET_NO_NEW_PRIVS applied at sandbox.rs:1141 but no capability drop
   loop post-clone (B-18.report:42-44).
4. IpcHookDispatcher: implemented at plugins/src/ipc_hooks.rs:206, exported from
   plugins::lib.rs:28 — not imported or called from ipc_accept.rs or production_agent_rpc.rs
   (B-18.report:46-48).

phase-4A.report.md §PARTIAL adds:
5. test_4a_isolation_pid_namespace_real (agent_isolation.rs:163-206) checks process_id > 1
   only; does NOT read /proc/{pid}/status NSpid (4A.report:54-56).
6. No Windows AppContainer SID test via NtQueryInformationProcess (4A.report:77).
7. Five adversarial tests missing from 4A.4 battery (4A.report:111-116).

Insertion order in clone_cb (sandbox.rs:1136-1156): PR_CAPBSET_DROP loop THEN setrlimit calls
THEN existing seccomp application — seccomp must come last to prevent post-prctl filter bypass.

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-18_tier_c_sandbox_activation.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\steps\phase-4A.report.md
- .elai_cc/crates/orchestrator/src/sandbox.rs:1080-1200 (clone_cb)
- .elai_cc/crates/orchestrator/src/ipc_accept.rs (run_agent_session frame loop)
- .elai_cc/crates/orchestrator/tests/agent_isolation.rs:163-206
- .elai_cc/crates/orchestrator/tests/adversarial_ipc.rs (22 existing tests)
- .elai_cc/crates/plugins/src/ipc_hooks.rs:206 (IpcHookDispatcher)
- .elai_cc/crates/orchestrator/src/production_agent_rpc.rs
- F:\.ELAI_workflow\docs\specs\guide\steps\phase-4A.txt (§4A.3, §4A.4, §4A.5)

## Tasks

### Linux rlimit + capability enforcement (safety-critical)

1. **setrlimit calls in clone_cb** (sandbox.rs). Insert after PR_SET_NO_NEW_PRIVS, before
   seccomp. Five calls using libc::setrlimit (already allowlisted at sandbox.rs:937):
   RLIMIT_CPU soft=60/hard=90, RLIMIT_NOFILE 256/256, RLIMIT_NPROC 32/32,
   RLIMIT_AS 512MB/512MB (phase-4A.report stricter than B-18 plan's 2GB — documented in
   decisions), RLIMIT_FSIZE 100MB/100MB. On failure: _exit(124). No heap allocation;
   libc::rlimit is stack struct. Add named constants — no magic numbers.

2. **/proc hidepid=2 mount in clone_cb** (sandbox.rs). In clone_cb before execvp, call
   libc::mount("proc", "/proc", "proc", MS_NOSUID|MS_NOEXEC|MS_NODEV, "hidepid=2").
   Child already has private mount table from CLONE_NEWNS. On EPERM: _exit(123). On other
   errno: _exit(123). Add comment: "EPERM on unprivileged CI is expected; CI skips
   these tests via CLONE_NEWNS::need_cap guard (existing pattern sandbox.rs:1116)."

3. **PR_CAPBSET_DROP loop in clone_cb** (sandbox.rs). Insert after PR_SET_NO_NEW_PRIVS,
   before task 1 setrlimit block. Loop cap from 0 to CAP_LAST_CAP (40 for kernel 5.15+;
   use constant not literal). Call prctl(PR_CAPBSET_DROP, cap, 0, 0, 0) per cap.
   EINVAL on unknown cap: continue (non-fatal). Other errno: _exit(122).
   Use PR_CAPBSET_DROP = 24 as literal fallback if libc constant absent — verify before use.

### IpcHookDispatcher wiring

4. **Wire IpcHookDispatcher into run_agent_session** (ipc_accept.rs). Add parameter
   hook_dispatcher: Option<Arc<dyn IpcHookDispatcher>>. In the frame dispatch loop, call
   dispatcher.on_frame_routed(&frame, &route_result) before writing the response frame.
   Verify import path from elai_plugins before editing (grep Cargo.toml for plugins dep;
   confirm no new dependency cycle — orchestrator->plugins is pre-existing per lib.rs:28).

5. **ProductionAgentRpcServer** (production_agent_rpc.rs). Pass
   Some(Arc::new(IpcHookDispatcher::default())) to run_agent_session at the live call site.
   Locate call site by grepping for run_agent_session in production_agent_rpc.rs before edit.

### Real isolation tests

6. **Extend test_4a_isolation_pid_namespace_real** (agent_isolation.rs:163). After existing
   process_id > 1 check, read /proc/{child_pid}/status, parse NSpid line, assert inner PID
   is 1. Keep #[cfg(target_os="linux")] + EPERM-skip guard. Grounding: NSpid format in
   man 5 proc lists nested PIDs space-separated, outermost first.

7. **RLIMIT_AS visibility test** (agent_isolation.rs). Read /proc/{child_pid}/limits, parse
   "Max address space" hard limit row, assert value == 536870912 (512MB).

8. **Windows AppContainer SID test** (agent_isolation.rs, #[cfg(target_os="windows")]).
   Call GetTokenInformation(TokenAppContainerSid) on the spawned process handle. Assert SID
   is non-null (AppContainer SID set by CreateAppContainerProfile per sandbox.rs:390-420).
   SAFETY NOTE: requires user review before merge (raw Win32 process handle access).

### Adversarial tests (4A.4 scoped set)

9. **test_4a_adv_ipc_u32_max_length_rejected** (adversarial_ipc.rs). Frame header with
   length=u32::MAX -> IpcFrameError::PayloadTooLarge BEFORE any allocation. Comment:
   "// Covers 4A.report action: u32::MAX pre-alloc check"

10. **test_4a_adv_selfmod_path_traversal** (adversarial_ipc.rs). BashRequest with path
    "../metrics.rs" -> SelfModViolation route result from IpcGateway::route_frame.
    Grounding: SelfModificationLocks realpath wired at ipc_gateway.rs:292,562 (B-18.report:23).
    Comment: "// Covers 4A.report action: selfmod_path_traversal"

11. **test_4a_adv_selfmod_bash_write_locked** (adversarial_ipc.rs). BashRequest writing to
    locked path -> blocked by IpcGateway before dispatch. Comment: "// bash_write_locked"

12. **test_4a_adv_selfmod_git_commit_locked** (adversarial_ipc.rs). BashRequest containing
    "git commit" targeting locked path -> blocked. Comment: "// git_commit_locked"

13. **test_4a_adv_selfmod_symlink_resolved** (adversarial_ipc.rs). Create OS symlink to locked
    path, send BashRequest targeting symlink -> realpath resolution catches redirect, blocked.
    Use std::os::unix::fs::symlink (Linux) / std::os::windows::fs::symlink_file (Windows).
    Skip on permission error. Comment: "// symlink_resolved"
    SAFETY NOTE: symlink creation requires OS permission; user review required.

### Metrics + decisions

14. **Update metrics/phase_4A.toml** after smoke test. Add [runtime_measurements_b18b]:
    rlimit_enforcement_verified, hidepid_mount_verified, cap_drop_count,
    ipc_hook_dispatcher_wired=true, adversarial_tests_added=5,
    isolation_nspid_test_passes, windows_appcontainer_test_passes, completion=true.

15. **Decision doc** workflow/reasoning/C-11-decisions.md. Three decisions:
    (a) RLIMIT_AS=512MB over B-18 plan's 2GB — phase-4A.report is stricter; defense-in-depth;
    (b) hidepid mount in clone_cb child side, not parent nsenter — simpler, no ns fd tracking;
    (c) PR_CAPBSET_DROP before setrlimit before seccomp — seccomp last prevents bypass.

## WIRING_CONTRACT

produces_symbols:
  - setrlimit calls for RLIMIT_CPU/NOFILE/NPROC/AS/FSIZE in clone_cb (sandbox.rs)
  - mount("proc", ..., "hidepid=2") in clone_cb (sandbox.rs)
  - PR_CAPBSET_DROP loop in clone_cb (sandbox.rs)
  - hook_dispatcher parameter + on_frame_routed call in run_agent_session (ipc_accept.rs)
  - ProductionAgentRpcServer passes IpcHookDispatcher to run_agent_session
  - NSpid assertion + RLIMIT_AS visibility in agent_isolation.rs
  - Windows AppContainer SID test (agent_isolation.rs cfg(windows))
  - 5 adversarial tests: u32_max, selfmod_path_traversal, bash_write_locked,
    git_commit_locked, symlink_resolved (adversarial_ipc.rs)
  - metrics/phase_4A.toml [runtime_measurements_b18b]
consumes_symbols:
  - clone_cb at sandbox.rs:1136 (insertion point)
  - libc::setrlimit, libc::rlimit (verify libc crate version supports these)
  - libc::mount, libc::MS_NOSUID/MS_NOEXEC/MS_NODEV (verify constants before use)
  - libc::prctl, PR_CAPBSET_DROP=24 (literal fallback if libc const absent)
  - IpcHookDispatcher (elai_plugins::ipc_hooks:206; confirm no dep cycle)
  - IpcGateway::route_frame (ipc_gateway.rs:380)
  - SelfModificationLocks at ipc_gateway.rs:292,562
  - test_4a_isolation_pid_namespace_real at agent_isolation.rs:163 (extended)
stub_ban:
  - setrlimit MUST be in clone_cb with actual OS calls; no struct-only stub
  - IpcHookDispatcher::on_frame_routed MUST fire in live frame loop
  - test_4a_isolation_pid_namespace_real MUST read NSpid from /proc/{pid}/status
  - Adversarial tests MUST test real routing, not struct construction
observable_effect:
  trigger: elai /task --mode cloud "write a malicious-looking but harmless script
           that attempts /etc/passwd read" (cloud Sonnet 4.6, executor in Tier C)
  expect_linux:
    - NSpid != 1 in /proc/{pid}/status (PID namespace boundary)
    - /etc/passwd read fails with EACCES (filesystem masking)
    - SSB FindingPayload SANDBOX_VIOLATION empty (no escape)
    - /proc/{pid}/limits "Max address space" hard = 536870912
  expect_windows:
    - AppContainer SID present via GetTokenInformation on spawned process
    - task completes without escape
  expect_adversarial:
    cargo test -p elai-orchestrator --test adversarial_ipc: all pass, count >= 27

## DETERMINISTIC_CHECK

```bash
# 1. rlimit constants in sandbox.rs
rg "RLIMIT_CPU|RLIMIT_NOFILE|RLIMIT_NPROC|RLIMIT_AS|RLIMIT_FSIZE" \
  .elai_cc/crates/orchestrator/src/sandbox.rs
# Expected: >=5 matches

# 2. PR_CAPBSET_DROP
rg "PR_CAPBSET_DROP|capbset" .elai_cc/crates/orchestrator/src/sandbox.rs
# Expected: >=1 match

# 3. hidepid mount
rg "hidepid" .elai_cc/crates/orchestrator/src/sandbox.rs
# Expected: >=1 match

# 4. IpcHookDispatcher wired in ipc_accept.rs
rg "IpcHookDispatcher|hook_dispatcher" .elai_cc/crates/orchestrator/src/ipc_accept.rs
# Expected: >=2 matches

# 5. NSpid assertion
rg "NSpid|nspid" .elai_cc/crates/orchestrator/tests/agent_isolation.rs
# Expected: >=1 match

# 6. 5 adversarial tests present
rg "u32_max_length|selfmod_path_traversal|bash_write_locked|git_commit_locked|symlink_resolved" \
  .elai_cc/crates/orchestrator/tests/adversarial_ipc.rs
# Expected: 5 matches

# 7. Scoped test runs
cargo test -p elai-orchestrator --test adversarial_ipc 2>&1 | grep "test result"
# Expected: ok, N passed (N >= 27)
cargo test -p elai-orchestrator --test agent_isolation 2>&1 | grep "test result"
# Expected: ok, N passed
cargo check --workspace 2>&1 | tail -3
# Expected: zero new errors
```

## ACCEPTANCE

- [ ] setrlimit for RLIMIT_CPU (60s/90s), RLIMIT_NOFILE (256), RLIMIT_NPROC (32),
      RLIMIT_AS (512MB), RLIMIT_FSIZE (100MB) present in clone_cb
- [ ] /proc hidepid=2 mount in clone_cb; EPERM-skip guard documented
- [ ] PR_CAPBSET_DROP loop in clone_cb after PR_SET_NO_NEW_PRIVS, before seccomp
- [ ] IpcHookDispatcher wired in run_agent_session; no feature-flag disable in production
- [ ] ProductionAgentRpcServer passes hook_dispatcher to run_agent_session
- [ ] test_4a_isolation_pid_namespace_real reads /proc/{pid}/status NSpid; inner PID == 1
- [ ] test_4a_isolation_rlimit_as_visible: /proc/{pid}/limits Max address space == 512MB
- [ ] Windows AppContainer SID test present (cfg(windows), GetTokenInformation)
- [ ] 5 adversarial tests added (tasks 9-13); each has 4A.report citation comment
- [ ] Smoke test passes (cloud Sonnet 4.6, executor in Tier C): NSpid != 1, EACCES on
      /etc/passwd, SANDBOX_VIOLATION empty, RLIMIT_AS=512MB visible, Windows SID present
- [ ] cargo test adversarial_ipc: >= 27 pass; cargo test agent_isolation: all pass
- [ ] cargo check --workspace: zero new errors
- [ ] metrics/phase_4A.toml [runtime_measurements_b18b] with completion=true
- [ ] Decision doc C-11-decisions.md present (3 decisions)
- [ ] SAFETY: user review completed before merge of tasks 8-13 (adversarial + Win32 paths)

## Files touched

- MODIFY: .elai_cc/crates/orchestrator/src/sandbox.rs
    (clone_cb: PR_CAPBSET_DROP loop + setrlimit block + hidepid mount; insert after line 1141)
- MODIFY: .elai_cc/crates/orchestrator/src/ipc_accept.rs
    (run_agent_session: hook_dispatcher param + on_frame_routed call)
- MODIFY: .elai_cc/crates/orchestrator/src/production_agent_rpc.rs
    (pass IpcHookDispatcher::default() to run_agent_session)
- MODIFY: .elai_cc/crates/orchestrator/tests/agent_isolation.rs
    (extend NSpid test; add rlimit_as_visible; add cfg(windows) AppContainer SID test)
- MODIFY: .elai_cc/crates/orchestrator/tests/adversarial_ipc.rs
    (append 5 tests: tasks 9-13)
- MODIFY: .elai_cc/metrics/phase_4A.toml
    (add [runtime_measurements_b18b] after smoke test)
- ADD: workflow/reasoning/C-11-decisions.md

## Safety gate

Safety-critical session. Require user review before merge for:
- sandbox.rs clone_cb changes (syscall ordering error can cause seccomp bypass or hang)
- Adversarial tests tasks 8-13 (OS symlink creation, Win32 process handle access)
- Windows NtQueryInformationProcess / GetTokenInformation (undocumented NT API)
Flag in PR description: "safety-critical: syscall ordering in fork/exec boundary +
adversarial OS handle tests — requires explicit review."

## Next

C-12 (scope TBD) or C-plan prioritization after C-11 smoke test confirms Tier C
isolation is provably end-to-end on both Linux and Windows.
