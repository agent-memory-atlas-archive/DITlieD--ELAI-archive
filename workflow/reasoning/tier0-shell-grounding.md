Tier 0 Shell Rewrite — Grounding Sweep
Session: s-38600
Date: 2026-04-22

Claims classified, verified or deferred:

1. GROUNDED — Svelte 5 runes available.
   Evidence: crates/dashboard/src-frontend/package.json line 17: "svelte": "^5.0.0".
   Runes usage confirmed in crates/dashboard/src-frontend/src/App.svelte:27 (let view = $state(...)).

2. GROUNDED — mount() entry pattern in main.ts.
   Evidence: crates/dashboard/src-frontend/src/main.ts:1-12. imports { mount } from "svelte",
   calls mount(App, { target }). Do NOT modify main.ts in Tier 0.

3. GROUNDED — .svelte.ts extension activates runes in standalone TS.
   Evidence: crates/dashboard/src-frontend/src/lib/debugState.svelte.ts:14 uses
   export const debug = $state<...>({...}) outside a .svelte file. layoutState.svelte.ts
   follows the same pattern.

4. GROUNDED — vite base path is "./" (relative asset resolution).
   Evidence: crates/dashboard/src-frontend/vite.config.ts:14 base: "./".
   Tier 0 must not break relative-asset resolution. No static imports start with "/".

5. GROUNDED — App.svelte structure matches context-compiler report.
   Evidence: direct read of App.svelte (75 lines, discriminant View = "ProjectSelector"
   | "TemplatePicker" | "EmptyFolderChooser" | "TaskInput" | "Workspace" | "Settings";
   Workspace renders in {:else} branch at line 62).

6. DEFERRED — Svelte component mount tests (vitest + @testing-library).
   package.json has no test framework. Adding vitest violates the no-new-deps rule.
   Decision: Tier 0 contract tests are static-only (invoke-count regression grep,
   App.svelte import-path check, npm run build exit code). Mount-style tests
   (Shell sidebar click flips only PRIMARY, PanelHost slot render, StatusBar
   snapshot) moved to Tier 1 backlog when vitest is approved as a dep.

7. GROUNDED — invoke baseline 35 verified by context-compiler.
   Exclusions from baseline (0 hits): select_project, create_project, open_preview_url.
   Context-compiler INVOKE BASELINE table authoritative; Tier 0 must not decrease.

Verdict: unblocked. Proceed to TESTING_FIRST with static contract checks.
