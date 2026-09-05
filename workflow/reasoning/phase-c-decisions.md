Phase C1 — frontend wiring decisions (session s-67812, resume from s-44680 handoff)

What:
  Add LintFinding + LighthouseReport (+ LintRunnerConfig) interfaces to dashboard-v2/src/lib/api/types.ts.
  Add 3 browser-dev stub wrappers to dashboard-v2/src/lib/api/stub.ts:
    run_web_lint_check, run_lighthouse_audit, probe_chromium_runtime.
  Add 3 $state fields + getters + unwired flags to createWebQuality() in dashboard-v2/src/lib/state/drawers.svelte.ts.

Key decision 1 — STEP 0 prerequisites before C1:
  cargo check --workspace was run first as a baseline gate. Result: clean (54s, 0 errors).
  The handoff flagged pipeline_tree_navigator.rs:1717+1728 as pre-existing PhaseNode errors but the workspace check is now clean — those errors are no longer present in main.
  This establishes a known-good starting state so any failures during audit can be attributed unambiguously to C1 changes.

Key decision 2 — STEP 0c re-read (mandatory) found prompt-engineer summary errors:
  The prompt-engineer's summary diverged significantly from the actual Rust source. Source-grounded corrections:
  (a) LintSeverity is #[serde(rename_all = "lowercase")] -> TS union must be 'error'|'warning'|'info' (lowercase). The summary said capitalized 'Error'|'Warning'|'Info' (WRONG).
  (b) LintFinding has 7 fields including `fixable: bool` (the summary missed `fixable`). `file: PathBuf` (NOT Option<String>).
  (c) LighthouseReport has 4 fields: lcp_ms: u32, cls: f64, inp_ms: u32, perf_score: f64 (the summary fabricated fcp_ms, tbt_ms, ttfb_ms, total_blocking_time_ms, and incorrectly typed inp_ms as Option<f64>).
  (d) LintRunnerConfig has `skip_missing_packages: bool` with #[serde(default)] (the summary said `config_path?: string` which does not exist).
  Decision: ignore the summary; mirror the Rust source exactly. This is exactly why STEP 0c was mandatory.

Key decision 3 — Tauri command signature mismatch:
  The actual Tauri command run_web_lint_check takes 4 SEPARATE bool params (project_dir, eslint, stylelint, axe), NOT a single LintRunnerConfig struct.
  Decision: stub function takes 4 args matching Tauri shape. LintRunnerConfig added to types.ts is a UI-side bundle for state passing, separate from the wire format.

Key decision 4 — Tauri return-shape unwrap:
  run_web_lint_check returns {ok, project_dir, tool_count, findings: LintFinding[]}. Stub unwraps to LintFinding[].
  run_lighthouse_audit returns {ok, lcp_ms, cls, inp_ms, perf_score} (flattened). Stub assembles LighthouseReport from those fields.
  probe_chromium_runtime returns {ok, available, reason?}. Stub returns boolean (available).
  Decision: stub returns the bare data type the consumer needs; wrapper unwrapping is localized in stub.ts.

Key decision 5 — JS argument casing:
  Per stub.ts file header: "Argument keys are camelCase on the JS side and Tauri's auto-snake-casing converts them to the Rust signature."
  Decision: JS passes { projectDir, eslint, stylelint, axe } and { projectDir, url } respectively.

Key decision 6 — refresh() does NOT poll lint/lighthouse:
  Lighthouse runs take 25-45s; lint runs spawn npx subprocesses. These are user-action driven, not poll-driven.
  Decision: createWebQuality.refresh() leaves the 3 new fields untouched. State stays empty/null/false until C2 adds action handlers. unwired{} flags all 3 as true.

Key decision 7 — cleanup of new $state fields:
  Existing createXxx() pattern does NOT reset $state on teardown — state persists across drawer mount cycles (singleton store). New fields follow this pattern.
  Decision: add inline TODO comment noting C2 should add an explicit reset method (resetWebQualityResults) when action handlers land. Per dashboardRules.md MUST Rule 3, the cleanup path is documented even though it is deferred.

Expected test outcome:
  cargo check -p elai-web-validator: clean (no Rust changes in C1).
  web_phase_b_contract: 25/25 GREEN (no Rust changes affect these contracts).
  web_validator_contract: 16/16 GREEN.
  TypeScript compiles (npx tsc --noEmit if available; otherwise audit defers TS check).

Unknowns / DEFERRED:
  QC2 (run_web_lint SSB/orchestrator wiring): not C1 scope. Deferred to a later phase.
  QC3 (run_lighthouse orchestrator glue): not C1 scope. Deferred.
  C2 (action handlers + UI rendering of findings/lighthouse score): next phase.
  C3 (integration tests): subsequent phase.
  C4/C5 (advanced features): design-deferred.
