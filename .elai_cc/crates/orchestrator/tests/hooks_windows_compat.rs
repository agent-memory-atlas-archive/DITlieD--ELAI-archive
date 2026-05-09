// B-15A Task 16 — cross-platform shell dispatch contract for orchestrator hooks.
//
// The orchestrator hook executor used to hardcode `Command::new("sh")` at
// crates/orchestrator/src/hooks.rs:634. On Windows that path errors at runtime
// because there is no `sh` on PATH. The fix mirrors the plugin-side dispatcher
// at crates/plugins/src/hooks.rs:282-302 which selects `cmd /C` on Windows and
// `sh -c`/`sh -lc` elsewhere.
//
// This contract test asserts the dispatch program name matches the host.
// It is intentionally a smoke / dispatch-shape check, not a process spawn,
// because the goal is to lock in the cfg-gated branch — not to exercise the
// shell itself (which is covered by hook integration tests in the auditor's
// full suite).

use elai_orchestrator::hooks::shell_command_program;

#[test]
fn shell_command_program_matches_host() {
    let program = shell_command_program();

    if cfg!(windows) {
        assert_eq!(
            program, "cmd",
            "on Windows the orchestrator hook dispatcher must invoke `cmd /C`, \
             not `sh` (B-15A Task 16; see plugins/src/hooks.rs:282-302 for the \
             reference pattern)."
        );
    } else {
        assert_eq!(
            program, "sh",
            "on non-Windows hosts the orchestrator hook dispatcher must invoke \
             `sh -c` (POSIX-portable). cfg(not(windows)) branch."
        );
    }
}

#[test]
fn shell_command_program_is_non_empty_and_short() {
    // Defensive check: regardless of platform, the program string is one of a
    // small known set. Guards against accidental empty / path-leakage values.
    let program = shell_command_program();
    assert!(!program.is_empty(), "program name must not be empty");
    assert!(
        program == "cmd" || program == "sh",
        "program name must be one of {{cmd, sh}}; got {program:?}"
    );
}
