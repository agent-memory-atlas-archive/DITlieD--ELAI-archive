// C-11: Tier-C sandbox completeness contract test (B-18B deferred items).
//
// Plan: F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-11_sandbox_completeness.md
// Decisions: F:\.ELAI_workflow\workflow\reasoning\C-11-decisions.md
//
// This contract proves the wire-level surface of the C-11 deliverables:
//   1. Insertion order of sandbox.rs clone_cb (Linux): PR_SET_NO_NEW_PRIVS ->
//      PR_CAPBSET_DROP -> setrlimit -> hidepid mount -> seccomp apply ->
//      execvp. We assert via grep-style source-line ordering checks because
//      the syscall path is unreachable from a unit test without CAP_SYS_ADMIN.
//   2. setrlimit / PR_CAPBSET_DROP / hidepid are referenced in the clone_cb
//      body — guards against future refactors that drop the new syscalls.
//   3. IpcHookDispatcher is exported from the orchestrator public surface
//      and accepts the trait surface used by run_agent_session.
//   4. SelfModificationLocks::is_locked uses canonicalize-based realpath
//      resolution (the lock check semantics behind tasks 10-13).
//   5. metrics/phase_4A.toml contains the [runtime_measurements_b18b] block
//      with completion=true.
//
// All assertions are platform-agnostic at compile time — Linux-specific
// runtime tests live in tests/agent_isolation.rs (cfg-gated), and Windows
// AppContainer SID tests live in agent_isolation.rs cfg(windows).

#[cfg(target_os = "linux")]
use std::path::PathBuf;
use std::sync::Arc;

use elai_orchestrator::sandbox::SelfModificationLocks;

// ---------------------------------------------------------------------------
// Contract 1: sandbox.rs clone_cb insertion order is enforced in source.
// ---------------------------------------------------------------------------

#[test]
fn c11_contract_sandbox_clone_cb_has_expected_syscall_blocks() {
    // s-68264 audit cleanup: sandbox.rs split into sandbox/ module dir (mod.rs holds clone_cb).
    let sandbox_src = include_str!("../../orchestrator/src/sandbox/mod.rs");

    // The five required markers in clone_cb, listed in their HARD ordering.
    let markers = [
        "PR_SET_NO_NEW_PRIVS",
        "PR_CAPBSET_DROP",
        "setrlimit",
        "hidepid=2",
        "apply_filter",
    ];

    let mut last_pos: usize = 0;
    let mut last_marker = "<start>";
    for marker in &markers {
        let pos = sandbox_src[last_pos..]
            .find(marker)
            .unwrap_or_else(|| panic!(
                "marker '{marker}' missing from sandbox.rs (or appears before '{last_marker}'); \
                 C-11 clone_cb ordering broken — seccomp MUST be last and CAPBSET_DROP MUST be present"
            ));
        last_pos += pos + marker.len();
        last_marker = marker;
    }
}

#[test]
fn c11_contract_sandbox_contains_rlimit_constants() {
    // s-68264 audit cleanup: sandbox.rs split into sandbox/ module dir (mod.rs holds clone_cb).
    let sandbox_src = include_str!("../../orchestrator/src/sandbox/mod.rs");
    for needle in [
        "RLIMIT_CPU",
        "RLIMIT_NOFILE",
        "RLIMIT_NPROC",
        "RLIMIT_AS",
        "RLIMIT_FSIZE",
    ] {
        assert!(
            sandbox_src.contains(needle),
            "sandbox.rs missing rlimit reference '{needle}' — C-11 setrlimit block incomplete"
        );
    }
}

#[test]
fn c11_contract_sandbox_contains_cap_last_cap_read() {
    // s-68264 audit cleanup: sandbox.rs split into sandbox/ module dir (mod.rs holds clone_cb).
    let sandbox_src = include_str!("../../orchestrator/src/sandbox/mod.rs");
    assert!(
        sandbox_src.contains("/proc/sys/kernel/cap_last_cap"),
        "sandbox.rs must read /proc/sys/kernel/cap_last_cap in the PARENT (C-11 D6)"
    );
    assert!(
        sandbox_src.contains("CAP_LAST_CAP_FALLBACK"),
        "sandbox.rs must define CAP_LAST_CAP_FALLBACK fallback constant (C-11 D6)"
    );
}

// ---------------------------------------------------------------------------
// Contract 2: ipc_accept.rs wires hook_dispatcher into run_agent_session.
// ---------------------------------------------------------------------------

#[test]
fn c11_contract_ipc_accept_run_agent_session_takes_hook_dispatcher() {
    let ipc_src = include_str!("../../orchestrator/src/ipc_accept.rs");
    assert!(
        ipc_src.contains("hook_dispatcher"),
        "ipc_accept.rs::run_agent_session must declare a hook_dispatcher param (C-11 D4)"
    );
    assert!(
        ipc_src.contains("IpcHookDispatcher"),
        "ipc_accept.rs must reference IpcHookDispatcher trait"
    );
    assert!(
        ipc_src.contains("pre_tool_use"),
        "ipc_accept.rs ToolRequest branch must invoke pre_tool_use"
    );
    assert!(
        ipc_src.contains("post_tool_use"),
        "ipc_accept.rs ToolRequest branch must invoke post_tool_use"
    );
}

// ---------------------------------------------------------------------------
// Contract 3: task_runner.rs:1554 region passes hook_dispatcher into the
// run_agent_session call (BabyStepRunner production wiring per C-11 D2).
// ---------------------------------------------------------------------------

#[test]
fn c11_contract_task_runner_passes_hook_dispatcher_into_run_agent_session() {
    let tr_src = include_str!("../../orchestrator/src/task_runner.rs");
    // The call site must thread self.hook_dispatcher into the run_agent_session args.
    assert!(
        tr_src.contains("hook_dispatcher"),
        "task_runner.rs must thread hook_dispatcher through BabyStepRunner (C-11 D2)"
    );
    assert!(
        tr_src.contains("run_agent_session"),
        "task_runner.rs must still call run_agent_session"
    );
    // Builder method exists so callers can opt-in to hook firing.
    assert!(
        tr_src.contains("with_hook_dispatcher"),
        "task_runner.rs must expose AgentPoolBabyStepExecutor::with_hook_dispatcher builder"
    );
}

// ---------------------------------------------------------------------------
// Contract 4: SelfModificationLocks uses realpath comparison (canonicalize).
// This is the lock surface behind the 4 selfmod adversarial tests (10-13).
// ---------------------------------------------------------------------------

#[test]
fn c11_contract_selfmod_locks_realpath_resolved() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let project = tmp.path().to_path_buf();
    let locked = project.join("workflow");
    std::fs::create_dir_all(&locked).expect("create workflow dir");
    std::fs::write(locked.join("state.json"), b"{}").expect("write state.json");

    let mut locks = SelfModificationLocks::new();
    locks.add_lock(locked.clone());

    // Direct hit.
    assert!(locks.is_locked(&locked.join("state.json")));

    // Path-traversal hit: <project>/elsewhere/../workflow/state.json.
    let traversal = project
        .join("elsewhere")
        .join("..")
        .join("workflow")
        .join("state.json");
    assert!(
        locks.is_locked(&traversal),
        "path traversal must be caught by canonicalize-based realpath (C-11 task 10)"
    );

    // Negative control.
    let outside = project.join("safe.txt");
    std::fs::write(&outside, b"ok").expect("write safe.txt");
    assert!(
        !locks.is_locked(&outside),
        "non-locked path must NOT be flagged"
    );
}

// ---------------------------------------------------------------------------
// Contract 5: metrics/phase_4A.toml exposes the C-11 [runtime_measurements_b18b].
// ---------------------------------------------------------------------------

#[test]
fn c11_contract_phase_4a_toml_has_b18b_block() {
    let toml_src = include_str!("../../../metrics/phase_4A.toml");
    assert!(
        toml_src.contains("[runtime_measurements_b18b]"),
        "phase_4A.toml must include the [runtime_measurements_b18b] section (C-11 task 14)"
    );
    assert!(
        toml_src.contains("ipc_hook_dispatcher_wired = true"),
        "phase_4A.toml must record ipc_hook_dispatcher_wired = true"
    );
    assert!(
        toml_src.contains("adversarial_tests_added = 5"),
        "phase_4A.toml must record adversarial_tests_added = 5"
    );
    assert!(
        toml_src.contains("completion = true"),
        "phase_4A.toml [runtime_measurements_b18b] must end with completion = true"
    );
    // Pin numeric named constants (no magic numbers).
    assert!(
        toml_src.contains("rlimit_as_bytes = 536870912"),
        "phase_4A.toml must record RLIMIT_AS_BYTES = 536870912 (512 MiB; decision D1)"
    );
    assert!(
        toml_src.contains("measured-in:C-20"),
        "phase_4A.toml must mark pending live-host measurements with measured-in:C-20"
    );
}

// ---------------------------------------------------------------------------
// Contract 6 (Tier-C, Linux-only): the C-11 named constants resolve from the
// orchestrator crate. We can't easily assert their VALUES from outside (they
// are `pub(crate)` to the orchestrator), so we instead test the SelfModification
// lock-add API which the adversarial tests rely on.
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
#[test]
fn c11_contract_self_mod_lock_add_persists() {
    let mut locks = SelfModificationLocks::new();
    let p = PathBuf::from("/tmp/elai_c11_contract_marker");
    locks.add_lock(p.clone());
    assert_eq!(locks.locked_paths.len(), 1);
    assert_eq!(locks.locked_paths[0], p);
}

// ---------------------------------------------------------------------------
// Contract 7 (Tier-C, Windows-only): IpcHookDispatcher trait is object-safe.
// run_agent_session takes `Option<Arc<dyn IpcHookDispatcher + Send + Sync>>`.
// We compile-check via a static assertion the dyn object can be built.
// ---------------------------------------------------------------------------

#[cfg(target_os = "windows")]
#[test]
fn c11_contract_hook_dispatcher_trait_is_object_safe() {
    use elai_orchestrator::IpcHookDispatcher;
    fn assert_object_safe(_x: Option<Arc<dyn IpcHookDispatcher + Send + Sync + 'static>>) {}
    assert_object_safe(None);
}

#[cfg(not(target_os = "windows"))]
#[test]
fn c11_contract_hook_dispatcher_trait_is_object_safe_linux() {
    use elai_orchestrator::IpcHookDispatcher;
    fn assert_object_safe(_x: Option<Arc<dyn IpcHookDispatcher + Send + Sync + 'static>>) {}
    assert_object_safe(None);
}
