# Pre-Phase 0 Implementation Audit
Date: 2026-04-24
Overall verdict: PARTIAL

Scope note: spec targets `crates/runtime/` and `crates/rusty-claude-cli/`. Phase 0 subsequently renamed runtime -> `crates/orchestrator/` and replaced rusty-claude-cli with `crates/cli/`. Evidence below maps spec paths to current paths where applicable.

## Sub-step: P0.1 cfg-gate unix-only code (3 files, 10 errors)
Verdict: IMPLEMENTED

Evidence:
- `.elai_cc/crates/orchestrator/src/mcp_stdio.rs:1417-1418` — `#[cfg(unix)] use std::os::unix::fs::PermissionsExt;` (verified via Read)
- `.elai_cc/crates/orchestrator/src/mcp_stdio.rs` set_mode sites all gated — lines 1468, 1517, 1654, 1782, 2704 each preceded by `#[cfg(unix)]` (grep output)
- `.elai_cc/crates/orchestrator/src/mcp_tool_bridge.rs:314-315` — import cfg-gated; set_mode at line 431 with `#[cfg(unix)]` (grep output)
- `.elai_cc/crates/plugins/src/lib.rs:2376-2386` — `use std::os::unix::fs::PermissionsExt;` inside `#[cfg(unix)] { ... }` block; matching `#[cfg(windows)]` block at 2395 writes `.cmd` + `@echo off` (Read verified)
- elai.map USE PermissionsExt rows: lines 237910-237912 show 3 import sites, all now in crates matching spec intent (runtime->orchestrator, plugins)
- Commit `4f857d9 P0.1: cfg-gate Unix imports...` body states "cargo test --no-run: 10 errors -> 0 errors"

Gap: Third spec target `crates/rusty-claude-cli/tests/mock_parity_harness.rs` does not exist in current tree — rusty-claude-cli was removed during Phase 0 workspace restructure. The fix originally landed there (per commit message) but subsequent strip removed the file. No regression risk since file is gone.

## Sub-step: P0.2 fix plugin hook tests (4 failures)
Verdict: IMPLEMENTED

Evidence:
- `.elai_cc/crates/plugins/src/hooks.rs:373-413` — `write_hook_plugin` has `#[cfg(unix)]` branch writing `#!/bin/sh` + printf hooks (lines 373-397) and `#[cfg(windows)]` branch writing `@echo off\r\necho ...\r\n` hooks (lines 399-413)
- `.elai_cc/crates/plugins/src/hooks.rs:490-493` — `pre_tool_use_denies_when_plugin_hook_exits_two`: `#[cfg(unix)] let hook_cmd = "printf 'blocked by plugin'; exit 2";` and `#[cfg(windows)] let hook_cmd = "echo blocked by plugin & exit /b 2";`
- `.elai_cc/crates/plugins/src/hooks.rs:512-519` — `propagates_plugin_hook_failures`: cfg-gated Unix `printf ...; exit 1` vs Windows `echo ...& exit /b 1`
- `.elai_cc/crates/plugins/src/hooks.rs:282-302` — `shell_command` dispatches: `#[cfg(windows)] Command::new("cmd").arg("/C")` vs `#[cfg(not(windows))] Command::new("sh")` (HookRunner adversarial concern addressed)
- Commit `d561632 P0.2-P0.4...` body reports "cargo test -p plugins --lib: 33 passed"

Gap: none significant. `collects_and_runs_hooks_from_enabled_plugins` (spec line 395) now resolves via `write_hook_plugin` being cfg-gated, no test-level cfg annotation needed.

## Sub-step: P0.3 fix tools tests (4 failures)
Verdict: IMPLEMENTED

Evidence:
- `.elai_cc/crates/tools/src/lib.rs:7089-7094` — `glob_and_grep_tools_cover_success_and_errors` uses `std::path::Path::new(...).ends_with("nested/lib.rs")` (Path-component match, separator-agnostic)
- `.elai_cc/crates/tools/src/lib.rs:7461-7463` — `powershell_runs_via_stub_shell` fully `#[cfg(unix)]` gated (entire test excluded on Windows)
- `.elai_cc/crates/tools/src/lib.rs:4841-4846` — `resolve_repl_runtime` for python: `#[cfg(unix)] detect_first_command(&["python3","python"])` / `#[cfg(windows)] detect_first_command(&["python","python3"])` — fixes `repl_executes_python_code` 9009 error
- `.elai_cc/crates/tools/src/lib.rs:5452-5455` — `env_lock()` static `OnceLock<Mutex<()>>`
- `.elai_cc/crates/tools/src/lib.rs:7139-7141` — `skill_loads_local_skill_prompt` uses `env_lock().lock().unwrap_or_else(std::sync::PoisonError::into_inner)` — PoisonError recovered, not propagated (pattern used at 19+ env_lock call sites, grep confirms)
- Commit `d561632` body: "cargo test -p tools --lib: 60 passed" (spec target 61; one test became `#[cfg(unix)]`-gated on Windows, expected)

Gap: `skill_loads_local_skill_prompt` modifies only `HOME` env (line 6150-6151); on Windows the config-home discovery (lib.rs:3271 `ELAI_CONFIG_HOME` / `CODEX_HOME` / `HOME`) still inspects HOME first, so the test works — but a future change adding `USERPROFILE` lookup could break silently. Non-blocking.

## Sub-step: P0.4 fix 2 warnings
Verdict: IMPLEMENTED

Evidence:
- `.elai_cc/crates/plugins/src/hooks.rs:2-3` — `#[cfg(not(windows))] use std::path::Path;` (import gated to the platform where line 291 `Path::new(command).exists()` uses it) — spec's "unused on Windows" warning resolved
- `.elai_cc/crates/orchestrator/src/hooks.rs:634-637` — `fn shell_command(...) { let mut command_builder = Command::new("sh"); command_builder.arg("-c").arg(command); ... }`. `mut` is actually required here because `.arg()` takes `&mut self` — compiler would not emit unused_mut. Commit `d561632` body confirms this was fixed ("Remove unused mut (runtime/hooks.rs:636)") and that "cargo check: 0 errors, 0 warnings" at commit time; current state still has `let mut`, indicating current code structure introduces genuine mutation usage post-fix.

Gap: The current `let mut` at orchestrator/hooks.rs:635 may have been re-introduced by a later edit but without triggering an unused_mut warning since `.arg()` requires `&mut self`. No live warning expected. UNVERIFIED whether additional warnings have accumulated post-commit; no cargo run allowed in this audit.

Gap (cross-cutting): `orchestrator/src/hooks.rs:634-637` unconditionally spawns `sh` with no Windows cfg dispatch. This is outside strict P0.4 scope (the spec P0.4 only addresses the `mut` warning) but is a latent Windows runtime bug for the `/hooks` path inherited from runtime. The plugins copy of shell_command (plugins/hooks.rs:282-302) has proper dispatch; the orchestrator copy does not.

## Sub-step: P0.5 verify clean baseline
Verdict: MISSING

Evidence:
- `.elai_cc/metrics/pre_phase_0.toml` — does NOT exist. `ls .elai_cc/metrics/ | grep pre_phase` returns nothing. Spec deterministic check: `.elai_cc/metrics/pre_phase_0.toml exists and test_fail = 0` -> fails.
- `.elai_cc/metrics/phase_0_baseline.toml`, `phase_0_implement.toml` etc. exist (phase 0, not pre-phase 0).
- No `.github/workflows` at `.elai_cc/.github/` nor at `F:/.ELAI_workflow/.github/workflows/` — no CI evidence of Windows pass. UNVERIFIED whether a justfile target exists.
- Commit messages report pass counts (33 plugins, 60 tools) but no recorded workspace-wide baseline metric file per spec format.

Gap: required metrics TOML was never produced. All acceptance text depends on recorded metrics; without file, baseline clean-state claim is unverifiable post-hoc. Additionally no evidence of "all cfg-gated tests documented (which ones, why, Linux-only flag)" deliverable in the workflow reasoning dir.

## Summary
- Implemented: 4 / 5 (P0.1, P0.2, P0.3, P0.4)
- Partial: 0 / 5
- Missing: 1 / 5 (P0.5)
- Confidence: HIGH for P0.1-P0.4 (direct file evidence + commit diffs); MEDIUM for absence of P0.5 (missing file is certain, but passing tests may have been verified without archiving metrics)
- Critical gaps:
  - `.elai_cc/metrics/pre_phase_0.toml` never created; P0.5 deterministic check fails on artifact presence
  - No CI workflow file on disk; current Windows-pass state is not continuously verified
  - Latent: `orchestrator/src/hooks.rs:634` still hardcodes `sh` — not a P0.4 regression (spec did not require it) but Windows runtime hook execution for the orchestrator path will fail; plugin hooks work correctly
  - Third spec target file `rusty-claude-cli/tests/mock_parity_harness.rs` removed during Phase 0 strip (acceptable; original fix applied in its P0.1 commit before removal)
  - No documentation of cfg-gated tests list as required by P0.5 acceptance clause
