Phase C2 + C3 — action handlers, UI render, and tests (session s-67812)

What:
  C2: action handlers + reset method + UI render of WEB VALIDATION section.
    - Add to createWebQuality(): lintRunning, lighthouseRunning, lintHasRun, lighthouseHasRun, chromiumProbeDone $state flags; runWebLint(), runLighthouseAudit(), probeChromium(), resetWebQualityResults() handlers.
    - Update WebQualityDrawer.svelte: append a WEB VALIDATION section with eslint/stylelint/axe Toggles, project_dir + URL Inputs, Run Lint / Run Lighthouse / Probe Chromium buttons, findings list, Lighthouse score card, Chromium availability badge, Reset button.
    - Flip unwired flags to be derived from the *_HasRun / chromiumProbeDone flags.
  C3: tests pinning the wire format + fallback contract.
    - Node-test-runner specs (tests/api/stub-web-validation.test.ts) verifying browser-dev fallbacks return [] / null / false (no swallowed errors).
    - Rust contract test (crates/contracts/tests/web_phase_c_contract.rs) pinning Tauri command JSON keys: run_web_lint_check returns ok/project_dir/tool_count/findings; run_lighthouse_audit returns ok/lcp_ms/cls/inp_ms/perf_score; probe_chromium_runtime returns ok/available[/reason]. Done at the type level via #[derive(Deserialize)] on the expected shape so any wire-format drift fails to compile or fails a serde round-trip.

Key decision 1 — Action handlers live in createWebQuality(), not in the .svelte component:
  Per dashboardRules.md MUST Rule 1: do NOT concentrate API calls + state + rendering in a single component. The handlers must mutate the singleton store, not local component state.
  Decision: runWebLint, runLighthouseAudit, probeChromium, resetWebQualityResults all live in createWebQuality(). The component imports webQualityData and calls webQualityData.runWebLint(...) etc.

Key decision 2 — Component-local state for inputs (project_dir, url, tool toggles):
  These are user-input values, not domain state. They should live in the .svelte component, not the store.
  Decision: WebQualityDrawer.svelte holds projectDir, lintUrl, eslintEnabled, stylelintEnabled, axeEnabled as $state. Persistence across drawer remounts is NOT required for C2 (a future C-phase can add localStorage if requested).

Key decision 3 — unwired flags become derived getters:
  C1 left lintFindings/lighthouseReport/chromiumAvailable as static unwired:true. After C2 they should reflect "no run yet".
  Decision: convert to derived getters: get lintFindings() { return !lintHasRun; }, etc. Existing pattern (devServers, scaffoldTemplates) already uses derived getters in the unwired{} object.

Key decision 4 — Auto-probe Chromium on bootstrap:
  probeChromium is fast (~1s, 5s timeout). Probing once on drawer mount lets the user immediately see if Lighthouse will work.
  Decision: bootstrap() (via the existing refresh()) calls probeChromium once. NOT polled — Chromium availability is unlikely to change at runtime.
  Tradeoff considered: putting it inside refresh() means it re-runs every 10s, which is wasteful. Better to gate via chromiumProbeDone — only probe if !chromiumProbeDone. Decision: gate it.

Key decision 5 — running flags + button disabled state:
  lintRunning / lighthouseRunning prevent concurrent invocations and let the UI disable buttons during the run.
  Decision: handlers return early if their corresponding running flag is set. UI binds Button {disabled} to that flag.

Key decision 6 — Reset clears findings + report but NOT the project_dir / url inputs:
  Inputs are user typing; clearing them on reset is annoying. Findings/report ARE results; clearing them makes sense.
  Decision: resetWebQualityResults() sets lintFindings=[], lighthouseReport=null, lintHasRun=false, lighthouseHasRun=false. Does NOT touch chromiumAvailable / chromiumProbeDone (they reflect environment, not results) and does NOT touch component-local input state.

Key decision 7 — C3 test strategy:
  No vitest in the project. Tests use Node's native test runner with --experimental-strip-types per existing pattern (tests/utils/slash-parse.test.ts).
  Initial plan: TS fallback tests + Rust contract tests.
  Empirical finding during execution: Node's native ESM resolver fails on stub.ts because the dependency graph (stub.ts -> ./mock-data, ./types) uses extensionless imports that Vite resolves but Node does not. The existing slash-parse.test.ts works only because that module is a leaf with no internal imports. Adding a path-resolver dep was rejected per the no-new-deps rule.
  Final decision: drop the TS fallback test. The 4-line stub fallbacks are trivially verifiable by inspection; the actual risk surface is wire-format drift on the Rust side, which is fully covered by the new web_phase_c_contract.rs.
  Decision for Rust side: add web_phase_c_contract.rs with 10 tests pinning the JSON shapes the Tauri commands emit + canonical-type serde format stability (LintSeverity lowercase, LintFinding 7-key shape, LighthouseReport 4-key shape, Optional<u32> -> null roundtrip).

Key decision 8 — No commits this session:
  User directive ("dont commit btw") and prior handoff cap. Same as C1.

Expected test outcome:
  cargo check -p elai-web-validator: clean.
  web_phase_b_contract: 25/25 GREEN (untouched).
  web_validator_contract: 18/18 GREEN (untouched).
  web_phase_c_contract: NEW, 3+ tests covering each command's wire shape.
  Node test (tests/api/stub-web-validation.test.ts): 3+ tests covering fallback contract.
  npx tsc --noEmit: clean.
  svelte-check: clean (if available).

Unknowns / DEFERRED:
  Persistence of input fields across drawer remount (localStorage): deferred.
  Real-time progress streaming for long Lighthouse runs (>60s): deferred — current UX shows a "running" badge with no streamed output.
  Filtering / sorting / virtualization of findings list when count > 500: per dashboardRules.md Context Rule 8 — flagged in TODO; not implemented in C2. UI handles up to a few hundred findings cleanly.
  C4 / C5: design-deferred per s-44680 handoff; require spec input from user.
