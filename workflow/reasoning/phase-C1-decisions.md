phase: C1 — Lighthouse perf + a11y in fix_and_rerun_until_green
date: 2026-05-15
session: s-5240

WHAT:
Validator closure inside fix_and_rerun_until_green (orchestrator task_runner.rs:5162 area) now invokes web_validator::cwv::run_lighthouse_scores. Pulls perf + a11y from a SINGLE Lighthouse spawn (a11y from /categories/accessibility/score; no separate axe-core pass). Threshold check (perf 0.80, a11y 0.90) emits Finding rows on failure so fix_and_rerun_until_green's loop iterates the fixer. Windows-aware chrome.exe 4-path probe with %VAR% expansion. Non-Windows: PATH-based chromium fallback.

KEY DECISIONS:
1. Single Lighthouse spawn yields both scores. Existing run_web_lint::axe pass NOT wired here; defer to a future phase if dedicated axe rule details are needed beyond Lighthouse's bundled a11y scoring.
2. New struct LighthouseScores { performance: f64, accessibility: f64 } separate from existing LighthouseReport (4 fields, consumed by web_phase_c_contract). Keeps backward compat.
3. New function run_lighthouse_scores(project_dir, url, chrome_path) — accepts optional chrome path, threads through to lighthouse CLI as --chrome-path arg.
4. find_chrome_path() probes 4 Windows locations: %ProgramFiles%\Google\Chrome\Application\chrome.exe, %ProgramFiles(x86)%\..., %LocalAppData%\Google\Chrome\Application\chrome.exe, %LocalAppData%\Programs\Google\Chrome\Application\chrome.exe. Returns None on non-Windows or all-missing.
5. lighthouse_validator_report pure-function threshold check; chrome_path:None branch returns Failed("chrome.exe not detected; install Chrome or set CHROME_PATH"). NO silent skip.
6. MIN_PERF_SCORE = 0.80, MIN_A11Y_SCORE = 0.90 named pub const in task_runner.rs (no magic numbers).
7. Validator closure flow: build → chrome probe → dev server → run_lighthouse_scores → threshold check → dev server drop. Drop is explicit (kill+wait), not RAII-only.
8. Contract test (lighthouse_validator_contract.rs) uses stub-validator pattern from fix_and_rerun_contract.rs — 5 tests (perf-fail, a11y-fail, both-pass, chrome-missing, exact-threshold-boundary) total <1s; NO live Lighthouse invocation.
9. accessibility score missing from JSON: defaults to 0.0 with tracing::warn (structured, not eprintln). Logged but not blocking the threshold check (which will then fail at 0.0 < 0.90).
10. parse_lighthouse_scores returns informative error on missing /categories/performance/score (required field).

EXPECTED TEST OUTCOMES (all confirmed green):
- lighthouse_validator_contract: 5/5 PASS in <1s.
- e2e_autonomous_build mock: 1/1 PASS.
- production_agent_rpc_contract: 3/3 PASS.
- agent_pool_listen_before_spawn_contract: 1/1 PASS.
- ipc_accept_loop_contract: 4/4 PASS.
- agent_pool_executor_wiring_contract: 4/4 PASS.
- web_phase_c_contract (LighthouseReport field-count guard): 10/10 PASS.
- cargo check --workspace: PASS.

UNKNOWNS RESOLVED:
- LighthouseReport has 4 fields locked by web_phase_c_contract → LighthouseScores defined as separate struct.
- a11y score available in Lighthouse JSON at /categories/accessibility/score (single spawn).
- find_chrome_path() implementable with std::env only — no new external Cargo deps.

OPEN UNKNOWNS / DEFERRED (auditor WARNs to-address-in-future):
- WARN-1: lighthouse_validator_report's chrome_path: Option<&Path> parameter is used as a boolean sentinel (Some(Path::new("probe-ok")) for the Unix PATH-probe success case). A future maintainer could add path.exists() validation that silently breaks Unix. Documented in inline comment + this decisions doc. Future: rename to chrome_available: bool or add a doc warning.
- WARN-2: parse_lighthouse_scores has no unit tests in cwv.rs module (only indirectly exercised by the stub-validator contract). Future: add 3 unit tests (happy-path, missing perf score error, accessibility-missing defaults to 0.0 with warn).
- Double probe_chromium on Unix path is a known redundancy (5s wasted per fix iteration); pre-existing pattern, not introduced by C1.

AUDITOR RESULT: WARN-only, score 0.88, 0 BLOCKs. Proceed to MEMORIZING.
