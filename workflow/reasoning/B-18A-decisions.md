B-18A — Tier C Sandbox subset (Tasks 7+8+13)
============================================

Scope: Windows enable_sandbox flip, launch_agent_process wire-up to
AgentSandbox::launch_agent, visibility filter R31 NotFound semantics.
Out of scope (deferred to B-18B+): Linux clone3+seccomp activation,
IPC accept loop, per-agent allowed_paths, 4-layer self-mod locks,
real isolation tests, phase_4A.toml metrics rewrite.

DECISION-A — enable_sandbox default
-----------------------------------

Choice: cfg!(target_os = "windows").

Sites flipped (line numbers verified live this session):
  - .elai_cc/crates/orchestrator/src/agent_pool.rs:153 (Default impl)
  - .elai_cc/crates/dashboard/src/lib.rs:1187 (baby-step executor)
  - .elai_cc/crates/orchestrator/src/task_runner.rs:2649 (BSP dispatch)

Rationale: B-18A only ships Windows-side AppContainer activation. Linux
must stay false until clone3+seccomp lands in launch_agent_process
(B-18B). cfg!() at literal site keeps the constant honest at compile
time and avoids two divergent runtime configs. The agent_pool factory
already falls back to no-sandbox if create_sandbox() fails, so an
older Windows host without AppContainer support degrades gracefully
rather than panicking.

Field at agent_pool.rs:144 is a struct field declaration, not a flip
site. Confirmed in context-package L14-15.

The test-only literal at tests/agent_isolation.rs:45 (enable_sandbox: true)
is left as-is per context package.

DECISION-B — AgentSandbox kill / wait surface
---------------------------------------------

Choice: Option A — extend AgentSandbox trait with `kill_agent(agent_id)`
and `wait_agent(agent_id) -> Result<ExitStatus, _>`. Each impl owns its
own per-process map.

Rejected: Option B (opaque SandboxedProcess struct). Reasoning: agent_pool
has no need to inspect the kernel handle / pid; encapsulating it inside
the sandbox keeps the trait small (3 methods + cleanup) and side-steps
two cross-platform Drop hazards (HANDLE leak vs zombie pid).

Implementation:
  - WindowsAppContainerSandbox stores `Mutex<HashMap<Uuid, usize>>` of
    HANDLE values. launch_agent stops calling CloseHandle on
    pi.hProcess so kill/wait can use it. kill_agent takes the entry,
    TerminateProcess + WaitForSingleObject + CloseHandle. wait_agent
    similarly + GetExitCodeProcess. ExitStatus constructed via
    std::os::windows::process::ExitStatusExt::from_raw.
  - LinuxSandbox stores `Mutex<HashMap<Uuid, libc::pid_t>>`. kill_agent
    SIGKILL + waitpid. wait_agent waitpid blocking. ExitStatus via
    std::os::unix::process::ExitStatusExt::from_raw.
  - Absent entry returns Ok(()) / Ok(synthetic-success-ExitStatus) —
    idempotent reap.

Object safety preserved (sandbox_wireup.rs::agent_sandbox_remains_object_safe
exercises Box<dyn AgentSandbox>). Existing callers source-compat: there
are zero callers of `.launch_agent(` outside the trait def itself
(grep verified before EXECUTING). The signature change is therefore
not a breakage.

DECISION-C — scratch_dir threading
----------------------------------

Choice: extend AgentSandbox::launch_agent signature to take
`scratch_dir: &Path` as the trailing parameter.

Was: WindowsAppContainerSandbox::launch_agent built env via
`AgentEnvironment::new(std::env::temp_dir())` (sandbox.rs:587-589),
silently dropping the per-agent scratch dir that
agent_pool::launch_agent_process configures. Context package HIGH risk
flagged this as a contract mismatch.

Now: launch_agent receives scratch_dir; the Windows impl uses
`AgentEnvironment::new(scratch_dir.to_path_buf())`. The Linux impl
accepts the parameter but defers env-block plumbing to B-18B+
(execvp does not accept env without a clone-arg shim; documented
inline). agent_pool::launch_agent_process passes `&handle.scratch_dir`.

DECISION-D — Visibility filter R31 NotFound flip
------------------------------------------------

Choice: replace `structured_error("visibility_filtered", "visibility_filtered: path is hidden")`
at tools/src/lib.rs:1404, 1415, 1424, 1442 with
`structured_error("not_found", &format!("No such file or directory: {path}"))`.

Same JSON envelope shape. Mirrors std::io::ErrorKind::NotFound display
text so an agent cannot fingerprint hidden vs missing via string match.
glob_search at tools/src/lib.rs:1431-1434 already returns Ok("[]") and
needs no change (R31-compliant).

Helper `is_visibility_filtered_path` renamed to `is_hidden_path` to
align with the adjacent `HIDDEN_PATH_PATTERNS` const and so the
deterministic check `rg "visibility_filtered" .elai_cc/crates/tools/src`
returns 0 hits as the spec requires. Internal-only rename — private fn
with no external callers.

contracts/tests/visibility_filter.rs:30 assertion flipped in the same
edit per the prompt: now asserts `error_code:"not_found"` AND
`No such file or directory:` AND NOT `visibility_filtered`. This is a
correction of a previously-buggy assertion, not test-rigging.

AgentSandbox trait diff
-----------------------

Old:
    pub trait AgentSandbox: Send + Sync {
        fn create(&self, config: &AppContainerConfig) -> Result<SandboxHandle, SandboxError>;
        fn launch_agent(&self, handle: &SandboxHandle, command: &str, args: &[String]) -> Result<AgentProcess, SandboxError>;
        fn cleanup(&self, handle: &SandboxHandle) -> Result<(), SandboxError>;
    }

New:
    pub trait AgentSandbox: Send + Sync {
        fn create(&self, config: &AppContainerConfig) -> Result<SandboxHandle, SandboxError>;
        fn launch_agent(&self, handle: &SandboxHandle, command: &str, args: &[String], scratch_dir: &Path) -> Result<AgentProcess, SandboxError>;
        fn kill_agent(&self, agent_id: Uuid) -> Result<(), SandboxError>;
        fn wait_agent(&self, agent_id: Uuid) -> Result<std::process::ExitStatus, SandboxError>;
        fn cleanup(&self, handle: &SandboxHandle) -> Result<(), SandboxError>;
    }

Trait remains object-safe (no generic methods, no Self return type,
no associated types). Send+Sync supertrait carried through.

Constraints honoured
--------------------

  - No new dependencies (Mutex<HashMap> from std).
  - No .unwrap() / .expect() in production paths added by this change.
    Existing unsafe Win32 unwraps inherited unchanged.
  - cfg(target_os="windows") gating prevents the new Windows-only flip
    from panicking on Linux/macOS — agent_pool.rs:221-231 falls back to
    sandbox=None when create_sandbox returns Err on macOS.
  - tests/sandbox_wireup.rs covers (a) default flag, (b) trait
    extension compile + dyn dispatch, (c) Windows scratch_dir smoke
    test (cfg-gated), (d) pool config branch consistency.

Deferrals (per prompt)
----------------------

  - crates/dashboard/src-frontend/** untouched.
  - No new windows-sys / nix features beyond existing Cargo.toml.
  - Linux clone3/seccomp/prctl/rlimits — DEFERRED.
  - IPC accept loop — DEFERRED.
  - Per-agent allowed_paths — DEFERRED.
  - 4-layer self-mod locks — DEFERRED.
  - Real /proc / NtQueryInformationProcess isolation tests — DEFERRED.
  - phase_4A.toml metrics rewrite — DEFERRED.

Cycle 2 — surgical close of grep_search BLOCK + bundled test WARN
-----------------------------------------------------------------

DECISION-D — grep_search must check `path` AND `glob` (R31)
  Scope: cycle-1 closed read_file/write_file/edit_file via path-only
  is_hidden_path checks. grep_search at lib.rs:1442 (cycle-1 form) only
  checked `path` — leaving a bypass: an agent passes path="." (allowed)
  with glob=".elai_cc/**" (hidden). Per GrepSearchInput
  (file_ops.rs:167-189), `path` and `glob` are both Option<String>; the
  glob field reached run_grep_search untouched. Cycle-2 fix at
  lib.rs:1440-1469 captures path_opt and glob_opt via input.get(...).
  and_then(|v| v.as_str()), runs is_hidden_path on each present field,
  and returns the same not_found / "No such file or directory: {x}"
  envelope produced for hidden paths. If both fields are absent, the
  branch returns the same not_found envelope (with a synthetic
  "(grep_search requires `path` or `glob`)" trailer) rather than
  letting an empty-result success leak that no error fired. is_hidden_path
  signature (&str) was unchanged — confirmed at lib.rs:1336 — so the
  same call shape is reused.

DECISION-E — bundle 4 missing visibility tests (cycle-2 WARN)
  Scope: contracts/tests/visibility_filter.rs previously covered only
  read_file (hidden path) and glob_search (hidden pattern). Cycle-2
  added four tests mirroring the existing R31 envelope assertions
  (error_code "not_found" + "No such file or directory:" prefix +
  no "visibility_filtered" leak):
    1. write_file_hidden_path_returns_not_found_error
    2. edit_file_hidden_path_returns_not_found_error
    3. grep_search_hidden_path_returns_not_found_error
    4. grep_search_hidden_glob_returns_not_found_error  (the BLOCK
       bypass — locks the cycle-2 fix in place).
  Test count delta: 3 -> 7 (full suite passes; see execution report).

Cycle-2 deferrals (unchanged from cycle-1)
  - kill_agent reaping -> B-18B.
  - sandbox test fidelity (real isolation) -> B-18B+.
  - scratch_dir env var plumbing test -> B-18B.
  - Linux _scratch_env dropped variant -> B-18B.
