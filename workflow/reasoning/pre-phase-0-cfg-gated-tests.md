PRE-PHASE-0 CFG-GATED TEST INVENTORY
date: 2026-06-05
gap: GAP-PP0-1 (base-phase-0-FINISH.md)
acceptance: list every OS-cfg-gated test by crate/file/function with rationale; cover cli, tools, plugins
source-of-truth: ripgrep over .elai_cc/crates on 2026-06-05 for #[cfg(unix)] / #[cfg(windows)] / #[cfg(not(unix))] / #[cfg(not(windows))] / #[cfg_attr(windows, ignore)] / #[ignore]

SCOPE NOTE
This artifact covers the three crates named in the P0.5 acceptance clause: cli (elai_cli), tools (elai_tools), plugins (elai_plugins).
Two distinct cfg patterns exist and are tracked separately:
  (A) GATED-OUT / IGNORED TESTS — the test does not execute on a given OS (whole-fn #[cfg(unix)], or #[cfg_attr(windows, ignore)]). These are the CI-relevant items: on Windows they are skipped, so a Windows-only CI run does not cover them, and a lightweight Windows gate (GAP-PP0-2) must not be read as proving these passed.
  (B) IN-BODY OS BRANCHES — the test runs on BOTH OSes but selects an OS-appropriate shell command / binary name / file extension inside the body via #[cfg]. These are NOT gated tests; they are listed for completeness so a future reader does not mistake them for skipped coverage.
Production (non-test) cfg branches are listed at the end as NON-TEST so the inventory is closed (no cfg site in these three crates is unaccounted for).


=====================================================================
CRATE: tools (elai_tools)   file: crates/tools/src/lib.rs
=====================================================================

(A) GATED-OUT / IGNORED TESTS — 4 total

1. fn worker_tools_gate_prompt_delivery_until_ready_and_support_auto_trust  (lib.rs:5708)
   gate: #[cfg_attr(windows, ignore = "...STATUS_STACK_BUFFER_OVERRUN 0xc0000409...")]  (lib.rs:5707)
   rationale: the elai_tools test executable segfaults on Windows (STATUS_STACK_BUFFER_OVERRUN) when this test runs in the same batch as bash_tool_reports_success_exit_failure_timeout_and_background; needs targeted Windows stack analysis. Tracked in s-68264-followup-patch-N.md. RUNS on Unix; SKIPPED on Windows.

2. fn worker_tools_detect_misdelivery_and_arm_prompt_replay  (lib.rs:5793)
   gate: #[cfg_attr(windows, ignore = "...elai_tools test exe segfaults...same batch issue...")]  (lib.rs:5792)
   rationale: same Windows segfault batch issue as #1. RUNS on Unix; SKIPPED on Windows.

3. fn bash_tool_reports_success_exit_failure_timeout_and_background  (lib.rs:7000)
   gate: #[cfg_attr(windows, ignore = "...bash tool uses POSIX `printf 'hello'` which Windows cmd.exe does not honor; ...also triggers STATUS_STACK_BUFFER_OVERRUN...")]  (lib.rs:6999)
   rationale: the test asserts on `printf 'hello'` output, which cmd.exe does not honor, AND it triggers the same batch segfault. Tracked in s-68264-followup-patch-N.md. RUNS on Unix; SKIPPED on Windows.

4. fn powershell_runs_via_stub_shell  (lib.rs:7635, carries a // TEST_KEEP marker)
   gate: #[cfg(unix)]  (lib.rs:7634)  — whole-test compiled out on Windows
   rationale: the test injects a stub `sh`/PowerShell shim via a Unix-only PATH manipulation and PermissionsExt chmod path; it exercises the PowerShell resolution branch using a Unix stub, so it is meaningful only when compiled on Unix. COMPILED+RUN on Unix; NOT COMPILED on Windows.

NOTE on the Windows-skipped batch (#1..#3): all three reference s-68264-followup-patch-N.md as the remediation ticket. The Windows skip is a known-stable workaround, not a silent gap; the rationale strings are self-documenting in source.


=====================================================================
CRATE: plugins (elai_plugins)
=====================================================================

(A) GATED-OUT / IGNORED TESTS — 1 total

1. fn (ruff integration test)  crates/plugins/tests/ruff_post_tool_use.rs:76
   gate: #[ignore = "requires ruff binary on PATH; run with --ignored when ruff is installed"]  (ruff_post_tool_use.rs:75)
   rationale: requires an external `ruff` binary on PATH; #[ignore]'d so CI stays green when ruff is absent. This is a BINARY-dependency gate, NOT an OS gate — it is skipped on every OS by default and opt-in via `--ignored`. Listed here because the file's helper `ruff_on_path()` (lines 24-27) uses #[cfg(windows)] to pick `ruff.exe` vs `ruff`, but the IGNORE is OS-neutral.

(B) IN-BODY OS BRANCHES (tests that RUN on both OSes; cfg only selects the command/binary) — for completeness, not gated:
  - crates/plugins/src/hooks.rs:490-493  fn pre_tool_use_denies_when_plugin_hook_exits_two — picks `printf ...; exit 2` (unix) vs `echo ... & exit /b 2` (windows)
  - crates/plugins/src/hooks.rs:512-521  fn propagates_plugin_hook_failures — same dual-shell command selection
  - crates/plugins/src/lib.rs:2468-2493  fn write_tool_plugin_with_name (test helper) — writes echo-json.sh (unix, chmod 0o755) vs echo-json.cmd (windows)
  - crates/plugins/src/hooks.rs:373-400   write_plugin_hooks (test helper) — writes pre.sh/post.sh (unix) vs .cmd (windows)
  - crates/plugins/tests/ruff_post_tool_use.rs:24-27  ruff_on_path() helper — ruff.exe vs ruff name selection
  These run on Windows; they are NOT skipped. No CI coverage gap.

(NON-TEST production cfg in plugins, for inventory closure):
  - crates/plugins/src/hooks.rs:2     #[cfg(not(windows))] use std::path::Path;  (import gating)
  - crates/plugins/src/hooks.rs:283-291  fn shell_command — cmd /C (windows) vs sh (unix) builder
  - crates/plugins/src/post_tool_use.rs:388-391  fn detect_ruff_binary — ruff.exe vs ruff candidate list


=====================================================================
CRATE: cli (elai_cli)
=====================================================================

(A) GATED-OUT / IGNORED TESTS — 0 OS-gated tests.

The cli crate has NO test that is compiled-out or ignored on the basis of OS. All of its #[cfg(unix)] / #[cfg(windows)] / #[cfg(not(unix))] / #[cfg(not(windows))] sites are PRODUCTION code branches (signal handling, process spawn flags, binary discovery), not tests. Verified by reading each site:
  - crates/cli/src/c17_lifecycle.rs:144 (#[cfg(unix)]) + :169 (#[cfg(not(unix))]) — production fn wait_for_terminating_signal (SIGTERM on unix, ctrl_c-only fallback elsewhere)
  - crates/cli/src/c17_lifecycle.rs (REPL/runtime helpers) — production
  - crates/cli/src/dashboard.rs:44 (#[cfg(windows)]) — production CREATE_NO_WINDOW (0x08000000) creation flag for the Tauri bootstrap spawn
  - crates/cli/src/browser_e2e_cmd.rs:192 (#[cfg(windows)]) + :201 (#[cfg(not(windows))]) — production fn has_bin (binary-on-PATH probe with .exe/.cmd/.bat suffix search on windows)
  - crates/cli/src/remote_sidecar.rs:197 (#[cfg(windows)]) — production DETACHED_PROCESS|CREATE_NO_WINDOW flags for the tunnel spawn

NON-OS ignored test (listed for completeness, NOT an OS gate):
  - crates/cli/src/cli_tests.rs:1361  #[ignore = "B: legacy `.elai/sessions/*.json` no longer resolved after B-1 path unification"] — a behavioral/legacy-path #[ignore], skipped on every OS, unrelated to platform.
  - crates/cli/tests/dkg_path_integration.rs:38  #[ignore = "manual: requires sidecar + model + populated DKG; ..."] — manual/external-dependency #[ignore], OS-neutral.


=====================================================================
SUMMARY (CI relevance)
=====================================================================
OS-GATED tests that a Windows-only CI run would NOT cover:
  tools::worker_tools_gate_prompt_delivery_until_ready_and_support_auto_trust   (windows-ignored)
  tools::worker_tools_detect_misdelivery_and_arm_prompt_replay                  (windows-ignored)
  tools::bash_tool_reports_success_exit_failure_timeout_and_background          (windows-ignored)
  tools::powershell_runs_via_stub_shell                                         (unix-only; not compiled on windows)
  => 4 items, ALL in the tools crate, ALL with self-documenting rationale strings + ticket (s-68264-followup-patch-N.md for the segfault batch).

cli: 0 OS-gated tests. plugins: 0 OS-gated tests (its only #[ignore] is binary-dependency, OS-neutral).

CONSEQUENCE for GAP-PP0-2 (local Windows gate): `cargo check --workspace` is COMPILE-only and is unaffected by #[ignore] (those still compile). The ONE thing a Windows check does NOT compile is tools::powershell_runs_via_stub_shell (whole-fn #[cfg(unix)]); a Windows compile path skips it silently. If a future gate runs `cargo test` on Windows, the three #[cfg_attr(windows, ignore)] tests are reported as `ignored`, not `passed` — do not read a green Windows test run as covering the Unix-only bash/worker batch. The Unix side is the authoritative coverage for those four.
