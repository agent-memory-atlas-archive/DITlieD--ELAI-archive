B-15 cli-regrowth Tranche J (orchestrate s-22600) — bench / eval / critic family extraction

Date: 2026-06-01
Session: s-22600
Sub-phase: trancheJ
Tier: HEAVY (autonomous; user asleep)
Status: SHIPPED (PARTIAL on B-15 cli-regrowth; further tranches remain)

WHAT

Extract the 11 bench / eval / critic CliAction dispatch arms from cli/src/main.rs (L853..L1003 in the pre-tranche file) into a new per-arm wrapper module cli/src/bench_cmd.rs. main.rs `fn run` now calls bench_cmd::run_bench_prmbench / run_bench_mode_routing / run_bench_conformal / run_bench_arc / run_bench_rag / run_bench_sycophancy / run_bench_canary / run_bench_tamper / run_bench_ablation / run_tier_eval / run_critic_report.

KEY DECISIONS

D-J1. SHIP-SHAPE: Option A from context-package-trancheJ.md (per-arm wrappers, args.rs UNCHANGED, SCHEMA_VERSION 195 UNCHANGED). Option B (re-grouping into one CliAction::Bench(BenchSub) enum) would touch args.rs parser-tree across 11 sub-clauses; deferred to a future tranche.

D-J2. NEW MODULE FILE: .elai_cc/crates/cli/src/bench_cmd.rs. verb_cmd naming convention matches checkpoint_cmd.rs / lora_gate_check_cmd.rs / contract_cmd.rs.

D-J3. ARG TYPES MIRRORED FROM args.rs L642-754 + L908/921/922 VERBATIM. The ctx package contained one inaccuracy (it listed floor/threshold fields as Option<f64>); the actual variant decls use Option<String> because CliAction derives Eq and f64 does not (per args.rs comments at L654, L668, L696, L711, L731). The executor verified signatures live against args.rs and the per-bench module pub fns before authoring bench_cmd.rs; wrappers thread the strings straight through to the existing parse-at-boundary discipline.

D-J4. EXCLUSIONS (per ctx DECISIONS-5 + 6):
  - FarmAdvRm (D-28): sibling boundary owns its own farm_adv_rm.rs module + inline FarmAdvRmArgs construction. Stays inline at main.rs.
  - DarwinLoop (E-7): both arms are #[cfg(feature = "darwinian-evolve")] / #[cfg(not(...))] feature-gated. Funneling would leak feature-gate complexity into bench_cmd. Stays inline at main.rs.
  - shortcoder_compress_text (main.rs L297..L352): EXTERNALLY consumed by runtime_client.rs:348 + 352. Stays pub(crate) in main.rs; out of bench-family scope.
  - cmd_incident_record (main.rs L1248..L1279), build_pr_review_action (main.rs L1152..L1241), check_anthropic_gateway (main.rs L438..L467), CharEstimatorProvider struct (main.rs L354..L422): future "main.rs helpers" extraction tranche, not bench-family.
  - args.rs: NOT TOUCHED.
  - SCHEMA_VERSION 195: NOT TOUCHED (no new SsbActionType).

D-J5. NO TESTS RE-RUN. Per CLAUDE.md HARD RULE "main model never runs tests" + memory feedback_no_cargo_test_unless_permitted, executor used `cargo check --workspace --tests` only. Auditor / user runs the test suite.

ACTUAL LOC DELTA

main.rs       BEFORE: 1279 LOC (verified Bash wc -l 2026-06-01)
main.rs       AFTER:  1221 LOC (verified Bash wc -l 2026-06-01)
                      delta: -58 LOC (less than projected -128 because the
                      new per-arm dispatch arms retain multi-line
                      destructuring for BenchCanary 5-field + BenchTamper
                      4-field bindings, and the FarmAdvRm + DarwinLoop
                      exclusion comments were preserved verbatim alongside
                      a new tranche-J explainer comment block. Doc-comment
                      compression on the eliminated bench arms accounted
                      for the bulk of the saved lines.)

bench_cmd.rs  NEW:    160 LOC (verified Bash wc -l 2026-06-01)
                      Module header (32 LOC) + 11 wrappers (each 5..10 LOC
                      including per-wrapper one-paragraph contract comment).

EXPECTED VS ACTUAL: ctx package projected main.rs ~1151 (-128) + bench_cmd.rs ~200-250. Actual: main.rs 1221 (-58), bench_cmd.rs 160. Variance is shape-only; the extraction is structurally complete (11 arms moved) and the net hygiene goal (one named module per CLI verb family) is achieved.

VERIFICATION

1. `cargo check -p elai-cli --tests` GREEN @ 25.09s. Zero new warnings; existing `unexpected cfg empire` warning on runtime_config.rs:340 is pre-existing (not touched by Tranche J).
2. `cargo check --workspace --tests` GREEN @ 3.72s (incremental). Zero new warnings. All pre-existing dead-code warnings unchanged.
3. main.rs `mod bench_cmd;` declaration inserted at the alphabetic neighborhood of the E-10 trio (between `mod critic_report;` and `mod lora_gate_check_cmd;`) so its E-10-family callees sit adjacent.
4. Underlying `bench_prmbench::run` / `bench_mode_routing::run` / ... / `critic_report::run` pub fns are LEFT UNTOUCHED — tests/bench_prmbench_smoke.rs and any other test that calls them directly continues to compile (verified via cargo check --tests).

UNKNOWNS / FOLLOW-UPS

- FarmAdvRm extraction: when D-28 owner is ready, a sibling wrapper could be added under bench_cmd.rs or under a new farm_cmd.rs. Out of Tranche J scope.
- DarwinLoop extraction: would require darwin_cmd.rs with both #[cfg]-arm wrappers + an unconditional shim for the disabled-message path. Out of Tranche J scope.
- Helper-fn extraction tranche (shortcoder_compress_text, cmd_incident_record, build_pr_review_action, check_anthropic_gateway, CharEstimatorProvider): separate future tranche; bench-family scope was always the cleanly-bounded slice.

WIRE-LEVEL CHANGES TOUCHED

None. Args parsing, SSB schema, contract tests, cross-language pair — all unchanged. This tranche is pure CLI hygiene (one indirection layer added between main.rs match arm and per-bench-module pub fn).
