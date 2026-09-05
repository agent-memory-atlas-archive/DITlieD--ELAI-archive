Phase WEB.1 — Scaffold Templates — Decision Doc
Session: s-1082492
Date: 2026-04-17

WHAT
  Implement WEB.1 per docs/mvp-implementation-order.txt lines 225-345.
  New Rust crate `scaffold` exposing `apply_template` (deterministic, no LLM, no clock, no entropy).
  Four web starter templates (vite-svelte, vite-react, astro, vanilla) under .elai_cc/templates/web/.
  Shared section taxonomy Hero/Features/Cta/Footer/Gallery with FIM prefix/suffix markers.
  HSL-only theme system via CSS custom properties with [data-theme] selectors.

KEY DECISION 1 — Crate layout: new workspace crate vs submodule
  Chosen: new crate .elai_cc/crates/scaffold/ as workspace member.
  Rationale:
    - Spec (WEB.1, line 317-322) defines apply_template as a durable public API invoked by
      later pipeline phases (baby step 1 for greenfield web). Deserves its own crate boundary.
    - Downstream 4E wiring is out of scope for WEB.1 but will import this crate by name.
    - crates/tools/ is a plugin/tool runtime; scaffolding is orthogonal.
    - Cost is one workspace member line. Workspace already has 13 members with similar shape.
  Rejected: crates/tools/scaffold.rs submodule — would couple scaffolding to tool runtime.

KEY DECISION 2 — target_dir-already-populated policy
  Chosen: refuse with ScaffoldError::TargetDirNotEmpty(PathBuf) when target_dir exists and
  contains any entries. Caller must create empty dir (or let apply_template create it).
  Rationale:
    - Deterministic behaviour: two runs into the same dir produce identical results only if
      the dir starts empty.
    - Safer default: no accidental overwrite of user files.
    - Merge/overwrite flag can be added later without breaking callers (add a new variant,
      keep default strict).
  Rejected: merge-with-overwrite — ambiguous semantics for FIM-marked files where later
  fills would be clobbered.

KEY DECISION 3 — Variable substitution engine
  Chosen: simple literal `{{name}}` token replacement, single left-to-right pass, ASCII-only
  identifiers (`[A-Za-z_][A-Za-z0-9_]*`). After replacement, any remaining `{{[A-Za-z_]...}}`
  pattern triggers ScaffoldError::VariableMissing(name).
  Rationale:
    - Deterministic, no nested expansion (avoids infinite loops when variable values
      contain `{{`).
    - UTF-8 safe: operates on byte slices but only scans ASCII markers `{{`/`}}`; values
      are inserted verbatim as bytes (UTF-8 multi-byte sequences preserved).
    - Text-file extension allowlist controls which files get scanned; binary assets pass
      through byte-for-byte.

KEY DECISION 4 — Determinism strategy
  - Collect template source entries into Vec<PathBuf>, sort() before iteration (alphabetical
    byte order on relative path).
  - Use BTreeMap<String,String> for variables (stable iteration if ever needed).
  - No HashMap in apply_template call chain.
  - No SystemTime / rand / env reads.
  - Test (e) hashes each output file with blake3 and compares two independent runs.

PINNED NPM VERSIONS (verified 2026-04-17 via `npm view <pkg> version` against registry)
  vite                             8.0.8    https://www.npmjs.com/package/vite             (vite-svelte, vite-react)
  svelte                           5.55.4   https://www.npmjs.com/package/svelte           (vite-svelte)
  @sveltejs/vite-plugin-svelte     7.0.0    https://www.npmjs.com/package/@sveltejs/vite-plugin-svelte (vite-svelte)
  react                           19.2.5    https://www.npmjs.com/package/react            (vite-react)
  react-dom                       19.2.5    https://www.npmjs.com/package/react-dom        (vite-react)
  @vitejs/plugin-react             6.0.1    https://www.npmjs.com/package/@vitejs/plugin-react (vite-react)
  astro                            6.1.7    https://www.npmjs.com/package/astro            (astro)
  typescript                       6.0.2    https://www.npmjs.com/package/typescript       (vite-svelte, astro)
  All versions pinned exact (no ^, no ~). TypeScript 6.0.2 released late March 2026
  (https://devblogs.microsoft.com/typescript/announcing-typescript-6-0/).
  TYPESCRIPT SCOPE: vite-svelte uses .svelte+.ts -> typescript required. astro uses .astro
  frontmatter with TS interfaces -> typescript required (Astro 6 unbundles typescript).
  vite-react uses JSX not TSX per spec -> typescript NOT added (intentional; spec says
  "JSX, not TSX for simplicity"). vanilla has no deps.
  @sveltejs/vite-plugin-svelte 7.0.0 requires svelte >= 5.46.4 (peer constraint documented).
  Audit note 2026-04-17: auditor flagged svelte=5.55.4 as non-existent; re-verified via
  `npm view svelte version` -> returns 5.55.4. False positive; keeping pin.

DEV PORTS
  vite-svelte  5173  (vite default)
  vite-react   5174  (distinct)
  astro        4321  (astro default)
  vanilla      8080  (simple-http-server; no build)

EXPECTED TEST OUTCOME
  cargo test -p scaffold -> all 8 integration tests pass.
  cargo check -p scaffold, cargo check --workspace -> clean (only pre-existing warnings
    in sibling crates allowed).
  npm install && npm run build for vite-svelte, vite-react, astro -> exit 0.
  vanilla: test -f index.html passes; no npm step.
  Zero unwrap/expect in scaffold/src outside #[cfg(test)].

UNKNOWNS / RISKS
  - Some npm packages may have peer dependency conflicts with React 19 + Vite 8 + TS 6.
    Mitigation: if npm install fails, lock to previous minor. Retry budget is one pass —
    if deps do not resolve, fall back to last-good stable set (Vite 7 / React 18 / TS 5)
    and re-verify versions via npm view.
  - Astro 6 may require Node 20+. Assumed present in CI.
  - No existing FRAMEWORK_RULES entry for JS/TS web frameworks — context-compiler flagged
    this HIGH gap. OUT OF SCOPE for WEB.1 (do not touch project_profile.rs).

FILES TO CREATE / MODIFY
  Modify: .elai_cc/Cargo.toml (add one workspace member line)
  Create: .elai_cc/crates/scaffold/Cargo.toml
  Create: .elai_cc/crates/scaffold/src/{lib.rs,error.rs,manifest.rs,substitute.rs,apply.rs}
  Create: .elai_cc/crates/scaffold/tests/apply_template.rs
  Create: .elai_cc/templates/web/manifest.toml
  Create: .elai_cc/templates/web/vite-svelte/** (package.json, vite/tsconfig/index/app, sections)
  Create: .elai_cc/templates/web/vite-react/** (analogous, .jsx sections per spec)
  Create: .elai_cc/templates/web/astro/** (astro.config.mjs, src/pages/index.astro, layouts, components)
  Create: .elai_cc/templates/web/vanilla/** (index.html, style.css, main.js, sections partials)

NON-SCOPE
  - No FRAMEWORK_RULES extension.
  - No wiring into pipeline_tree_navigator (that is 4E).
  - No npm lockfile commit (generated at install time).
