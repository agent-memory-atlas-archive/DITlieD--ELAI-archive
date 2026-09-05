Phase B (s-44680) — bundled execution decision doc
====================================================

Date: 2026-05-08
Producer: s-44680 (continuation of s-69820 handoff)
User constraints: dep approvals broadly granted; no commits; only 1 audit cycle;
"finish it all" → all four streams (B1+B2+B3+B4) bundled into one cycle
(overrides handoff's "Do NOT bundle them" guidance with explicit user direction).

---------------------------------------------------------------------
Decisions per stream
---------------------------------------------------------------------

B1 — Tier-2 AST predicates
  WHAT:
    - Added 4 grammar deps to crates/knowledge/Cargo.toml: tree-sitter-html=0.23,
      tree-sitter-css=0.25, tree-sitter-svelte-ng=1.0, tree-sitter-astro-next=0.1
      (last is OPTIONAL, behind feature `web-astro`).
    - Registered "html"/"css"/"svelte" languages in TreeSitterAstEvaluator::new()
      (crates/knowledge/src/ast_adapter.rs); "astro" cfg-gated.
    - Rewrote 10 of 11 Tier-2 rules in data/rules/web.toml from
      grep_pattern → ast_pattern with sexp queries. WEB-022 stays
      grep_pattern (module-graph traversal is outside AST scope).
  KEY DECISION:
    - Q1 ANSWERED: there is NO crates/dkg. The DkgQuerier trait lives in
      crates/rules/src/predicate.rs; tree-sitter is already a knowledge
      dep. New grammars land directly in TreeSitterAstEvaluator.
    - tree-sitter-astro-next is single-maintainer (PRRPCHT) — feature-gated
      to keep default builds reproducible and audit-clean.
    - WEB-016 (default-export + named-export coexistence): used `(program
      (export_statement) (export_statement))` sexp — this matches
      programs containing 2+ export statements. Slightly broader than
      the regex but operates on real AST.
  EXPECTED OUTCOME:
    - rules_web_contract still passes 19/19 (verified).
    - web_phase_b_contract verifies (a) language registration, (b) all
      Tier-2 except WEB-022 use ast_pattern, (c) language fields all
      resolve to registered grammars.
  UNKNOWNS:
    - At runtime against real .html / .svelte / .css files, the sexp
      queries may be too narrow or too broad; this needs real-file
      verification once an orchestrator pass runs against a scaffolded
      project. (Out of scope for Phase B; flag for next session.)

B2 — Real WebLint runner
  WHAT:
    - New module crates/web_validator/src/lint.rs with public types
      LintFinding, LintSeverity, LintRunnerConfig, and async fn run_web_lint.
    - Added WebValidatorError::NodeMissing (probed via `node --version`)
      and WebValidatorError::ToolOutputInvalid for unparseable tool output.
    - Tools spawned via `npx --yes <tool>` so the host doesn't need a
      global install — only node + npm + a local node_modules.
    - rule_id namespacing: `eslint:`, `stylelint:`, `axe:` prefixes
      keep findings disjoint from in-house WEB-* rules (Q4 decision).
    - LintRunnerConfig::default() returns ALL-DISABLED — opting in is
      explicit. run_web_lint short-circuits to Ok(empty) when nothing
      enabled (no node probe required).
  KEY DECISION:
    - Q2: lint findings flow as plain Vec<LintFinding>; orchestrator
      ferries them through its existing IntegrationCheckpoint records,
      same as WebBuildResult. No new SSB variant needed for Phase B.
  EXPECTED OUTCOME:
    - Phase A's web_validator surface remains unchanged.
    - All 5 lint::tests + 4 contract tests pass.
  UNKNOWNS:
    - End-to-end run against a real project NOT validated this session
      (would need node + the target's node_modules). Documented in
      next handoff.

B3 — Tier-3 contrast math + CWV
  WHAT:
    - New module crates/web_validator/src/contrast.rs: pure-Rust WCAG
      2.1 luminance + ratio math. Public surface: srgb_to_linear,
      relative_luminance, contrast_ratio, wcag_aa_pass, wcag_aaa_pass,
      parse_hex, evaluate_contrast_in_css. ZERO new deps.
    - New module crates/web_validator/src/cwv.rs: LighthouseReport DTO,
      run_lighthouse subprocess runner, probe_chromium probe,
      parse_lighthouse_report (testable on JSON fixture without spawning).
    - Added WebValidatorError::ChromiumMissing.
  KEY DECISION:
    - Did NOT add a new Predicate variant for contrast — instead shipped
      contrast.rs as a library function the orchestrator can call
      directly. Avoids a workspace-wide Predicate-enum change for one
      use case. Phase A's WEB-025 regex stays as-is.
    - Q3 (Chromium on remote): probe_chromium tries `chromium` then
      `google-chrome`. If both fail returns ChromiumMissing; orchestrator
      degrades the WebPerformance tier (treats as advisory). No remote
      SSH check this session — the probe runs at request time.
  EXPECTED OUTCOME:
    - Contrast math: 9 unit tests + 8 contract tests cover endpoints,
      symmetry, WCAG thresholds, hex parsing, and CSS scanning.
    - CWV: 5 unit tests cover JSON parsing of Lighthouse reports
      (with INP optional, score required).

B4 — WebPerformance tier
  WHAT:
    - Added WebValidationTier::WebPerformance variant to web_validator.
    - Updated pipeline_tree_navigator::web_validation_tiers_for_level
      to include WebPerformance at Concept-level only.
    - 3 new Tauri commands: run_web_lint_check, run_lighthouse_audit,
      probe_chromium_runtime. All bound input paths through
      ensure_within_project_root (same path-traversal mitigation as
      Phase A).
  KEY DECISION:
    - WebPerformance only at Concept level: Lighthouse runs cost
      25-180s + a Chromium binary. BabyStep / Step / Phase are too
      hot to pay that price.
    - Q5 (FIM section-fill higher-level workflow): DEFER — the Phase A
      handoff explicitly gates this on a live-FIM coherence test
      against the abliterated Qwen2.5-Coder model, which is HARDWARE-
      HEAVY and out of Phase B's bundled scope.
  EXPECTED OUTCOME:
    - test_web_validation_tiers_concept_full_pipeline updated and
      passes; remaining tier tests untouched.

---------------------------------------------------------------------
Anti-slopsquatting verification log (2026-05-08)
---------------------------------------------------------------------

Cargo (cargo info <name>):
  tree-sitter-html        0.23.2  tree-sitter org (official)            ✓
  tree-sitter-css         0.25.0  tree-sitter org (official)            ✓
  tree-sitter-svelte-ng   1.0.2   tree-sitter-grammars org              ✓
  tree-sitter-astro-next  0.1.1   PRRPCHT (single maintainer)           ⚠ feature-gated

Node (npm view <name>):
  eslint            v10.3.0   eslint/eslint                              ✓
  stylelint         v17.11.0  stylelint/stylelint                        ✓
  @axe-core/cli     v4.11.3   dequelabs/axe-core-npm                     ✓
  lighthouse        v13.3.0   GoogleChrome/lighthouse                    ✓

---------------------------------------------------------------------
Pre-existing failures NOT owned by this session
---------------------------------------------------------------------

Per Phase A handoff Note 3:
  - crates/contracts/tests/dashboard_pi_tier3_on_open_contract.rs:56
  - crates/contracts/tests/post_execution_rule_eval_contract.rs:183, 270
  - crates/contracts/tests/task_runner_post_execution_rule_eval_contract.rs
  - crates/contracts/tests/fu_profile_selector_dispatch_contract.rs:48
Discovered this session (NEW pre-existing, not Phase A's):
  - crates/orchestrator/src/pipeline_tree_navigator.rs:1717,1728
    references undefined `PhaseNode` in test helper. Last touched in
    commits 0e3eaae / a0fc334. Test compiles ONLY when
    `crate::research_tree::PhaseNode` is in scope (it isn't).
    Phase B did not modify those lines. Documented; no fix in scope.

---------------------------------------------------------------------
Test results (scoped, this session)
---------------------------------------------------------------------

cargo check -p elai-web-validator -p elai-knowledge -p elai-orchestrator  → clean
cargo check -p elai-dashboard-v2                                         → clean
cargo test  -p elai-web-validator --lib                                  → 37 pass / 0 fail
cargo test  -p elai-knowledge   --lib ast_adapter                        → 5 pass / 0 fail
cargo test  -p elai_cc_contracts --test web_phase_b_contract             → 25 pass / 0 fail
cargo test  -p elai_cc_contracts --test rules_web_contract               → 19 pass / 0 fail
