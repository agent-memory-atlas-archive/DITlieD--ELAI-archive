# Verification Report: B-15_phase0_debt_burndown
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-15_phase0_debt_burndown.md
Date: 2026-05-17
Overall status: PARTIAL
Confidence: HIGH

## Summary
B-15 (Phase 0 Debt Burndown) is substantially started but incomplete. One of five dead-code files has been deleted (remote.rs), and four remain. The CLI was already well below 500 LOC at 255 lines. File reorganization (file_ops/bash_validation/stale_branch) is fully done. The hooks.rs Windows "sh" bug is fixed. Scripts, CI workflow, and metrics TOML are all present. The HashMap ban is enforced by script and CI but 10 production files still contain the import. Entropy quarantine has not been applied; 50+ production source files outside clock.rs still call SystemTime::now or Instant::now, and the CI entropy check step is explicitly commented out. usage.rs still contains all the banned symbols (ModelPricing, format_usd, etc.) at 251 LOC.

## Claims verified DONE

- remote.rs deleted — evidence: file absent at crates/orchestrator/src/remote.rs (verified 2026-05-17)
- hooks.rs Windows "sh" hardcode fixed — evidence: crates/orchestrator/src/hooks.rs:748-771, cfg(windows)/cfg(not(windows)) dual dispatch; plan cited line 634, actual fix at lines 748-771
- file_ops.rs moved to tools/ — evidence: crates/tools/src/file_ops.rs EXISTS; crates/orchestrator/src/file_ops.rs MISSING
- bash_validation.rs in tools/ only (deduplication done) — evidence: crates/tools/src/bash_validation.rs EXISTS; crates/orchestrator/src/bash_validation.rs MISSING
- stale_branch.rs moved to tools/ — evidence: crates/tools/src/stale_branch.rs EXISTS; crates/orchestrator/src/stale_branch.rs MISSING
- check_entropy_quarantine.sh + .bat present — evidence: crates/../scripts/check_entropy_quarantine.sh EXISTS, check_entropy_quarantine.bat EXISTS
- determinism_proof.sh + .bat present — evidence: scripts/determinism_proof.sh EXISTS, scripts/determinism_proof.bat EXISTS
- smoke_test_ollama.sh + .bat present — evidence: scripts/smoke_test_ollama.sh EXISTS, scripts/smoke_test_ollama.bat EXISTS
- pre_phase_0.toml generated — evidence: .elai_cc/metrics/pre_phase_0.toml EXISTS with compile_errors=0, test_failures=0, warnings=0, platform="windows"
- CI workflow present and runs on Windows — evidence: .elai_cc/.github/workflows/windows.yml, runs-on: windows-2022, includes cargo check --workspace and HashMap ban check
- check_hashmap_ban.sh present — evidence: scripts/check_hashmap_ban.sh EXISTS (B-15B lint script with allowlist logic)
- cli/src/main.rs decomposed to 255 LOC — evidence: crates/cli/src/main.rs lines=255 (well under 500 spec target)
- cli decomposed modules all exist — evidence: cli/src/repl.rs, commands.rs, sidecar.rs, indexing.rs, handoff.rs, dashboard.rs all EXISTS
- ClockProvider trait created — evidence: crates/core/src/clock.rs exists (108 LOC), contains ClockProvider

## Claims PARTIAL

- HashMap ban enforced (Task 6 + Task 7)
  - exists: scripts/check_hashmap_ban.sh with allowlist; windows.yml step "HashMap ban check" at line 54
  - gap: 10 production source files still import use std::collections::HashMap: plugins/src/tiered_analysis.rs, plugins/src/import_extractor.rs, dashboard-v2/src-tauri/src/media_validation.rs, orchestrator/src/focus_tunnel/streaming.rs, orchestrator/src/focus_tunnel/mode.rs, rules/src/self_discovery.rs, knowledge/src/reranker.rs, orchestrator/src/session_dag.rs, orchestrator/src/self_mod_layer1a.rs, orchestrator/src/media_validator/render_adapters/mod.rs. The script uses an allowlist to suppress some; not all have been replaced with BTreeMap/FxHashMap/IndexMap.

- ClockProvider trait (Task 8) — instant_now method missing
  - exists: crates/core/src/clock.rs:ClockProvider trait
  - gap: plan requires instant_now() on the trait for Instant::now replacement; clock.rs does not contain "instant_now" method

- Entropy quarantine (Task 8) — leaks not replaced
  - exists: ClockProvider trait defined; scripts/check_entropy_quarantine.sh present
  - gap: 50+ production src files outside clock.rs still call SystemTime::now or Instant::now (confirmed by grep across 91 total files returning 202 occurrences). The windows.yml entropy check step is explicitly commented out with the note "16 production sites still need ClockProvider injection". Task 8 is not done.

- usage.rs reduction (Task 2)
  - exists: crates/core/src/usage.rs at 251 LOC
  - gap: ModelPricing (8 occurrences), UsageCostEstimate (5), pricing_for_model (5), format_usd (11) are all still present. Plan requires reduction to <30 LOC with only TokenUsage.

- Session msgpack (Task 15) — partially done
  - exists: crates/orchestrator/src/session.rs references both serde_json and msgpack (rmp_serde) at 1153 LOC
  - gap: plan says keep JSONL only behind --export-jsonl flag; it is unclear if full separation is complete without deeper read.

## Claims MISSING

- compact.rs deleted (Task 1) — searched: crates/orchestrator/src/compact.rs; file EXISTS at 624 LOC (plan estimated 696 LOC); not deleted
- oauth.rs deleted (Task 3) — searched: crates/providers/src/oauth.rs; file EXISTS at 591 LOC; not deleted
- prompt.rs deleted (Task 5) — searched: crates/orchestrator/src/prompt.rs; file EXISTS at 744 LOC; not deleted
- CI lint step for entropy quarantine (Task 9 observable) — searched: windows.yml; entropy check step is explicitly commented out ("Re-enable this step when Batch F of B-15 lands")

## Claims UNVERIFIABLE

- "84 std HashMap imports" count — the plan claims 84 at time of writing. Current grep finds 22 occurrences across 19 files (including test/bench). The number has changed since the plan was written; whether the reduction was systematic replacement or the plan number was approximate is not determinable from code alone.
- "40+ entropy quarantine leaks" count — current grep finds 202 total occurrences across 91 files; the plan's original "40+" figure cannot be matched to a specific point-in-time snapshot.
- cli/src/main.rs "was 11,181 LOC" — current file is 255 LOC; prior state before B-15 decomposition is not verifiable without git history at that commit.
- CI workflow "continuously verifying Phase 0 baseline" — CI YAML exists but actual GitHub Actions execution results are not accessible from local codebase inspection.
- Session persistence fully separated (Task 15) — session.rs contains both JSON and msgpack references; full separation boundary requires deeper logic read.

## What must be done (action list)

- [ ] Delete crates/orchestrator/src/compact.rs (624 LOC) after confirming zero production callers
- [ ] Delete crates/providers/src/oauth.rs (591 LOC) after confirming zero production callers
- [ ] Delete crates/orchestrator/src/prompt.rs (744 LOC) after confirming role_prompt config is wired
- [ ] Reduce crates/core/src/usage.rs to <30 LOC — delete ModelPricing, UsageCostEstimate, pricing_for_model, format_usd and fix ~19 downstream call sites
- [ ] Add instant_now() method to ClockProvider trait in crates/core/src/clock.rs
- [ ] Replace SystemTime::now/Instant::now in ~50 production src files with ClockProvider injection (cli/, orchestrator/, providers/, dashboard-v2/, rules/, knowledge/ directories)
- [ ] Re-enable entropy quarantine check step in .elai_cc/.github/workflows/windows.yml once production injection is complete
- [ ] Replace std::collections::HashMap in 10 remaining production files (tiered_analysis.rs, import_extractor.rs, media_validation.rs, streaming.rs, mode.rs, self_discovery.rs, reranker.rs, session_dag.rs, self_mod_layer1a.rs, render_adapters/mod.rs) with BTreeMap/FxHashMap/IndexMap per decision doc

## Notes

remote.rs deletion is the only fully-complete dead-code strip from the original 5 targets. The plan's B-15A scoped note in pre_phase_0.toml confirms this: "Only orchestrator/src/remote.rs was deleted; compact.rs, usage.rs, oauth.rs, prompt.rs deletions deferred per context-package-B-15A.md (live callers)". The hooks.rs fix is done correctly via cfg(windows)/cfg(not(windows)) at lines 748-771 (plan cited line 634 which is now different; fix matches intent). CLI decomposition is fully complete. All 6 scripts and metrics TOML are present. The outstanding work is concentrated in three areas: dead-code deletion (3 files), entropy quarantine injection (~50 files), and usage.rs reduction.
