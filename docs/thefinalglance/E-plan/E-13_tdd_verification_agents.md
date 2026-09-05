# E-13: TDD + verification agents — property-based dual oracle + cargo-mutants survival gate + anti-over-mock
Objective: replace ELAI's single-test-flip EXECUTING gate with a dual oracle (≥1 example flip AND ≥1 property/invariant pass), add a cargo-mutants survival gate at AUDITING, and add an anti-over-mock MistakeBook category. These three changes give a deterministic non-LLM signal at every promotion and close the "agent edits assertions to make CI green" + "tests mirror implementation" failure modes the Hora & Robbes 2026 and Connsulting 2026 data document.
Estimated effort: 2-3 sessions (~10-12 h).
Pre-requisites: C-28 PLANNED (Testing-Trust T-1 Mutation Testing — cargo-mutants 27.0.0 baseline), C-29 PLANNED (T-2 IntentPair + T-3 metamorphic relations), C-27 PLANNED (T-0 wiring contract), C-31 PLANNED (FilePlan), C-32 PLANNED (Skeleton-first / MiniCritic).
Parallel-safe with: E-11, E-12, E-14, E-15. AUGMENTS C-28 + C-29; some items SUBSUME the C-28 mutation-testing surface.
Source report: workflow/improvement-sweep-2026-05-19/reports/13-tdd-verification-agents.md

## Context

Property-Generated Solver (arXiv:2506.18315) shows +13.4% pass@1 over example-TDD with a Generator+Tester PBT loop; Agentic-PBT (arXiv:2510.09907) measured 56% valid bug rate across 100 Python packages with 86% precision at top-21. UTBoost (arXiv:2506.09289) found 24.4% of SWE-bench-Verified leaderboard entries inflated by tests that admit semantic errors. Meta ACH (arXiv:2501.12862) deployed mutant-guided LLM-generated tests in production with 73% engineer acceptance on privacy mutants. Hora & Robbes 2026 (arXiv:2602.00409) measured coding-agent commits adding mocks at 36% vs 26% for humans across 1.2M commits / 2168 repos, demonstrating that agent-authored tests over-mock by design. The Connsulting "Tests as Ceremony" + Testkube system-level-testing 2026 work both flag "CI green but hollow" as the dominant failure mode. ELAI today gates EXECUTING on a single example flip and AUDITING on rubber-stamp prevention (≥1 finding required). C-28 introduces the cargo-mutants 27.0.0 + mutmut 3.5.0 baseline; C-29 adds IntentPair + metamorphic relations. This E-N extends C-28/C-29 with: a dual EXECUTING oracle (property gate + example flip), targeted cargo-mutants survival at AUDITING with cached survivors per (file, blake3), an anti-over-mock PostToolUse hook, a non-LLM verifier requirement in D-3's consensus rule, and a `mutant-hunter` agent that authors kill-tests for survivors. Process Reward Model gating (arXiv:2509.02360) and No Free Labels (arXiv:2503.05061, TPR>96% / TNR<25%) jointly argue that LLM-as-judge alone under-counts negatives — the deterministic signal MUST be present.

## Existing state (B/C/D coverage)

- AUGMENTS C-28 (PLANNED) Testing-Trust T-1 Mutation Testing — extends the planned mutation-testing surface with targeted-mutant cache and survivor MistakeBook entries; ships independently if C-28 lands first.
- AUGMENTS C-29 (PLANNED) IntentPair + metamorphic relations — adds property-based testing as a third axis alongside metamorphic relations.
- EXTENSION of D-3 (PLANNED) VerificationResult aggregator — adds the "≥1 of N verifiers MUST be non-LLM" rule the report calibration finding requires.
- REUSES B-22b 4G.9 anti-spiral surface (mentioned in B-PLAN-SUMMARY.md lines 424-431) — over-mock detection feeds the same MistakeBook category space.
- REUSES E-11's per_level_critic — the Step-level critic now consumes mutation-score as evidence in addition to LLM judgment.
- NET-NEW: `mutant-hunter` agent file at `.claude/agents/mutant-hunter.md` that pairs with auditor; when cargo-mutants surfaces SURVIVED, mutant-hunter prompts local Qwen-2.5-Coder-7B to author one kill-test (must compile + flip mutant red).
- NET-NEW: Fuzz4All-style LLM-seeded fuzzing for parsers/serializers (CONCEPT-level gate; not on the inner loop).

## Reads (mandatory before coding)

- workflow/improvement-sweep-2026-05-19/reports/13-tdd-verification-agents.md
- .elai_cc/crates/orchestrator/src/auditor_mode.rs (existing AUDITING gate surface)
- .elai_cc/crates/orchestrator/src/dst.rs lines 335-485 (existing proptest usage; new property surface mirrors it)
- .elai_cc/crates/orchestrator/tests/research_tree.rs lines 555-590 (existing proptest! usage pattern)
- .elai_cc/crates/contracts/tests/idea_framing_contract.rs (test layout convention)
- .claude/agents/auditor.md (where the new MODE DISPATCH block for property + mutation runs)
- workflow/papers/PGS-interface.md (after paper-ingestor mode=paper on arXiv:2506.18315)
- workflow/papers/agentic-pbt-interface.md (after paper-ingestor mode=paper on arXiv:2510.09907)
- workflow/papers/meta-ach-interface.md (after paper-ingestor mode=paper on arXiv:2501.12862)
- arXiv:2602.00409 Hora & Robbes 2026 over-mock (via paper-ingestor)
- arXiv:2506.09289 UTBoost SWE-bench rot (via paper-ingestor)
- arXiv:2503.05061 No Free Labels (via paper-ingestor)
- arXiv:2509.02360 PRMs course-correct SWE agents (via paper-ingestor)

## Tasks

1. Add property-based EXECUTING gate. Extend the current "≥1 test flipped fail→pass" requirement to `(≥1 example flip) AND (≥1 proptest run with ≥10 cases, zero panics, ≥1 non-trivial property predicate)`. New module `.elai_cc/crates/orchestrator/src/property_gate.rs` with `PropertyGate { min_cases: 10, banned_predicates: ["true", "len >= 0"], ... }`. PreToolUse hook on test files rejects vacuous proptest predicates per UTBoost grounding. Wire into the existing EXECUTING gate decision path; emit new SSB action_type `property_gate_pass` / `property_gate_vacuous_rejected`.

2. Implement targeted cargo-mutants survival gate at AUDITING for changed crates. New file `.elai_cc/crates/orchestrator/src/mutation_gate.rs` that calls `cargo mutants --in-place --baseline skip --shard <touched_files>` with budget (60s STEP / 120s PHASE / 300s CONCEPT). Any MISSED mutant in a touched function = BLOCK FINDING. Cache survivor list at `~/.cache/elai/mutation_survivors.sqlite3` keyed by (file_path, blake3(file_content), function_name); cache TTL = 30d; cache hit short-circuits the re-run. This extends C-28's `EquivalentMutantCache` (SQLite at .elai/mutation_cache.sqlite3) — if C-28 is shipped first, the survivor cache becomes a second SQLite db sharing the same schema discipline.

3. Add anti-over-mock PostToolUse hook. New module `.elai_cc/crates/orchestrator/src/anti_mock.rs` with `AntiMockHook::on_post_tool_use(file, diff)`: count `mock.patch | vi.mock | mockall:: | MockAll | jest.mock | sinon.stub` insertions; if added without a matching `#[ignore]`-tagged integration test in the same diff, BLOCK with MistakeBook category `OverMocking`. Then mutation-test the mocked behavior; if all mutants of the real impl survive (the mock-tests are vacuous w.r.t. the real impl), BLOCK with category `MockHidesImpl`. Wire into the existing PostToolUse dispatcher.

4. Extend D-3 (PLANNED) VerificationResult aggregator with a hard constraint: `≥1 of N verifiers MUST be non-LLM` (cargo test, cargo-mutants survival, proptest panic-free, deterministic hash check). If D-3 has not shipped, ship the constraint as a local addition to `auditor_mode.rs::aggregate_verifiers` and migrate to D-3 when it lands. Grounding: arXiv:2503.05061 single-LLM-judge TNR<25%.

5. Author `mutant-hunter` agent at `.claude/agents/mutant-hunter.md`. Single role: when cargo-mutants surfaces SURVIVED, mutant-hunter is spawned with input { file_path, function_name, mutant_description, current_test_file }; output is one test addition that MUST compile + flip the mutant red. Same FSM substate as AUDITING (new substate FsmSubstate::MutantHunting). Auditor verifies the test addition by re-running cargo-mutants on the single mutant — if still SURVIVED, mutant-hunter retries once; on second SURVIVED, escalates to HITL. Agent file follows the existing .claude/agents/ convention (project root path resolution, sonnet model).

6. Implement Fuzz4All-style LLM-seeded fuzzing for parsers/serializers, CONCEPT-level gate only. New module `.elai_cc/crates/orchestrator/src/fuzz_gate.rs` gated by file pattern: `crates/*/src/parser*.rs`, `crates/*/src/wire*.rs`, IPC v2 message serializers. Budget 60-120s per CONCEPT-level audit. LLM (local Qwen 7B) generates ~50 seed inputs; cargo-fuzz runs them; any panic = BLOCK with category `FuzzPanic`. Cache panic corpus at `~/.cache/elai/fuzz_corpus/`.

7. Add `properties.md` per crate. Each properties.md file lists required invariants per crate (round-trip serde, idempotence, monotonicity, no-panic invariants). The `spec-grounder` agent reads properties.md; `spec_link_check.py` (existing harness tool at .claude/tools/) enforces presence for crates over a LOC threshold. New table `[testing.properties]` in elai_cc.toml: `min_loc_threshold = 500`, `block_on_missing = true`. Authoring is one-time per crate; existing crates over threshold get auto-generated stubs (mutant-hunter agent variant `property-seeder` proposes the first 3 invariants per crate).

8. Add anti-vacuous-test regex ban list to PreToolUse hook on test files. Banned patterns: `assert_eq!\(.*, .*\.unwrap\(\)\)` (assert on unwrap of result we never check), `assert!\(true\)`, `assert!\(.+\.len\(\) >= 0\)`, `assert!\(.+ != Default::default\(\)\)`, `assert!\(result\.is_ok\(\)\)` (without subsequent assertion on value), Python equivalents `assert True`, `assert result is not None`, `assert len(x) >= 0`. UTBoost-grounded list lives in `.elai_cc/crates/orchestrator/src/anti_vacuous.rs::VACUOUS_PATTERNS`. PreToolUse emits MistakeBook category `VacuousTest` on hit. Same regex compilation pattern as C-32's tiered_analysis.rs (LazyLock<Regex>).

9. Promote IntegrationTestRunner at CONCEPT level to run `proptest + cargo-mutants in parallel` with a fixed-seed corpus, gated by changed-files diff. New `IntegrationTestRunner` impl in .elai_cc/crates/orchestrator/src/integration_test_runner.rs (extends C-29's metamorphic surface). Cap parallelism at 4 to avoid combinatorial blow-up. Use property-coverage (% invariants exercised) NOT line coverage as the level metric. Emit SSB `integration_property_coverage` per CONCEPT-level audit.

10. Wire mutation-survival as evidence into per_level_critic (E-11). When per_level_critic emits its verdict at Step/BabyStep, attach `mutation_score: f32` from the most recent cargo-mutants run on the affected file. If mutation_score < 70 (C-28's default threshold), the critic verdict downgrades from PASS to WARN regardless of LLM verdict. New payload field in `per_level_critic_verdict` SSB action_type from E-11.

11. Dashboard: add `MutationGateGauge.svelte` (kill rate over last 50 mutants), `PropertyCoverageGauge.svelte` (invariants exercised), `OverMockHistogram.svelte` (mock-add rate per recent commits) under .elai_cc/crates/dashboard-v2/src/lib/components/. Mount in `lib/drawers/ReasoningDrawer.svelte`. Three Tauri commands: `get_mutation_gate_status`, `get_property_coverage`, `get_over_mock_stats`.

12. Add `[testing.mutation_targeted]` table to elai_cc.toml: `enabled = true`, `cache_ttl_days = 30`, `shard_strategy = "touched-files"`, `survivor_block_threshold = 0` (zero survivors required at AUDITING). Add `[testing.property]` table: `min_cases = 10`, `vacuous_predicate_block = true`, `min_loc_threshold = 500`. Add `[testing.anti_mock]` table: `mock_add_block_without_integration = true`, `mock_hides_impl_check = true`.

## WIRING_CONTRACT
produces_symbols:
  - orchestrator::property_gate::PropertyGate
  - orchestrator::mutation_gate::TargetedMutationGate
  - orchestrator::anti_mock::AntiMockHook
  - orchestrator::anti_vacuous::VACUOUS_PATTERNS (LazyLock<Vec<Regex>>)
  - orchestrator::fuzz_gate::FuzzGate (CONCEPT-level only)
  - .claude/agents/mutant-hunter.md
  - FsmSubstate::MutantHunting (extends existing substate enum)
consumes_symbols:
  - orchestrator::auditor_mode::aggregate_verifiers (D-3 surface; locally extended until D-3 ships)
  - orchestrator::per_level_audit_dispatch (B-41) — receives mutation_score evidence
  - orchestrator::mistake_book (C-21) — receives 4 new categories: OverMocking, MockHidesImpl, VacuousTest, FuzzPanic
  - orchestrator::ssb (new SSB action_types listed in DETERMINISTIC_CHECK)
reachable_from:
  - crates/cli/src/main.rs::main → task_runner::drive_task → auditor → mutation_gate
  - crates/dashboard-v2/src-tauri/src/lib.rs::submit_task → AUDITING → MutantHunting
stub_ban:
  - todo!() / unimplemented!() in any new module
  - Box<dyn Error> in mutation_gate / property_gate / anti_mock — use thiserror per-module enum
  - .unwrap() outside #[cfg(test)]
  - hardcoded paths — use core::paths helpers for cache dirs
frontend_pair: crates/dashboard-v2/src/lib/components/MutationGateGauge.svelte
observable_effect:
  trigger: "elai /task --mode cloud 'implement and test a string encoder/decoder pair with 3 properties (round-trip, idempotence, length-monotonic)'"
  expect_ssb: "property_gate_pass + integration_property_coverage > 0 within EXECUTING; mutation_gate_survivor_block = 0 at AUDITING; if executor adds a vi.mock without integration test, anti_mock_block emits + MistakeBook records OverMocking"
  expect_artifact: ".elai_cc/sessions/{id}/audit/mutation_survivors.toml lists 0 survivors after auditor pass; mutant-hunter spawn count visible in dashboard"

## DETERMINISTIC_CHECK
```bash
# 1. New gates wired
rg "PropertyGate|TargetedMutationGate|AntiMockHook|FuzzGate" .elai_cc/crates/orchestrator/src/
# expect: 4 module definitions + 4+ use sites

# 2. mutant-hunter agent file exists
ls .claude/agents/mutant-hunter.md
# expect: file exists

# 3. SSB action_types registered
rg "property_gate_pass|property_gate_vacuous_rejected|mutation_gate_survivor_block|anti_mock_block|fuzz_panic|integration_property_coverage|mutant_hunter_spawned" .elai_cc/crates/orchestrator/src/ssb.rs
# expect: >=7 hits

# 4. Anti-vacuous patterns compiled at startup
rg "static VACUOUS_PATTERNS: LazyLock" .elai_cc/crates/orchestrator/src/anti_vacuous.rs
# expect: 1 hit

# 5. properties.md presence check enforced for large crates
rg "properties\\.md" .claude/tools/spec_link_check.py
# expect: >=1 hit

# 6. cargo-mutants verified at correct version
cargo mutants --version | grep "27"
# expect: 27.x.x

# 7. Smoke: property gate rejects vacuous predicate
cargo test -p elai-orchestrator --test property_gate_vacuous rejects_assert_true_predicate
# expect: pass

# 8. Smoke: mutation gate blocks on survivor in touched file
cargo test -p elai-orchestrator --test mutation_gate_targeted blocks_on_survivor_in_touched_function
# expect: pass

# 9. Smoke: anti-mock blocks vi.mock without integration test
cargo test -p elai-orchestrator --test anti_mock_block_no_integration
# expect: pass
```

## ACCEPTANCE
- [ ] PropertyGate module exists; EXECUTING gate requires example flip AND property pass with ≥10 cases AND non-vacuous predicate
- [ ] TargetedMutationGate runs at AUDITING for touched files; budget 60/120/300s by level; cache at ~/.cache/elai/mutation_survivors.sqlite3
- [ ] AntiMockHook fires PostToolUse on test file writes; blocks mock.patch / vi.mock / mockall additions without integration test
- [ ] AntiMock mutant-test cross-check: when mock-tests cover an impl whose mutants all survive, BLOCK with MockHidesImpl
- [ ] D-3 consensus extended (or local equivalent) with ≥1 non-LLM verifier required
- [ ] mutant-hunter agent file present at .claude/agents/mutant-hunter.md; spawn surfaces in FsmSubstate::MutantHunting; retries once on SURVIVED, escalates to HITL on second
- [ ] FuzzGate gated by file pattern; CONCEPT-level only; 60-120s budget; corpus persisted
- [ ] properties.md required per crate over min_loc_threshold; spec_link_check.py enforces
- [ ] 4 new MistakeBook categories registered (OverMocking, MockHidesImpl, VacuousTest, FuzzPanic)
- [ ] 7+ new SSB action_types in ssb.rs
- [ ] Dashboard surface: MutationGateGauge + PropertyCoverageGauge + OverMockHistogram mounted in ReasoningDrawer; 3 new Tauri commands
- [ ] No NoopSsbNotifier; no Box<dyn Error>; no .unwrap() outside #[cfg(test)]

## Tests to add
- `.elai_cc/crates/orchestrator/tests/property_gate_vacuous.rs`: 6 cases (true / len>=0 / always-pass / non-trivial-round-trip / panic / converged-after-10)
- `.elai_cc/crates/orchestrator/tests/mutation_gate_targeted.rs`: 5 cases (no-survivors / 1-survivor-in-touched / 1-survivor-outside-touched / cache-hit / cache-miss)
- `.elai_cc/crates/orchestrator/tests/anti_mock_block_no_integration.rs`: 4 cases (mock-with-integration / mock-without-integration / mock-hides-impl / mock-on-genuine-external)
- `.elai_cc/crates/orchestrator/tests/anti_vacuous_patterns.rs`: 10 cases (one per pattern)
- `.elai_cc/crates/orchestrator/tests/fuzz_gate_parser.rs`: 3 cases (clean parser / panic / timeout)
- `.elai_cc/crates/orchestrator/tests/mutant_hunter_spawn.rs`: 3 cases (survivor present → spawn / spawn produces compilable test / second-survivor escalates HITL)
- `.elai_cc/crates/orchestrator/tests/properties_md_required.rs`: 2 cases (small crate exempt / large crate requires)
- `.elai_cc/crates/contracts/tests/d3_consensus_non_llm_required.rs`: 4 cases (all-LLM rejected / 1-non-LLM accepted / mixed / no-verifiers rejected)

## Files touched
- CREATE: `.elai_cc/crates/orchestrator/src/property_gate.rs` (~220 LOC)
- CREATE: `.elai_cc/crates/orchestrator/src/mutation_gate.rs` (~340 LOC; cargo-mutants subprocess + SQLite cache)
- CREATE: `.elai_cc/crates/orchestrator/src/anti_mock.rs` (~260 LOC)
- CREATE: `.elai_cc/crates/orchestrator/src/anti_vacuous.rs` (~180 LOC; LazyLock<Regex> patterns + matcher)
- CREATE: `.elai_cc/crates/orchestrator/src/fuzz_gate.rs` (~280 LOC)
- CREATE: `.elai_cc/crates/orchestrator/src/integration_test_runner.rs` (~240 LOC)
- CREATE: `.claude/agents/mutant-hunter.md` (~120 LOC)
- CREATE: `properties.md` per crate over min_loc_threshold (~15 files, ~40 LOC each)
- MODIFY: `.elai_cc/crates/orchestrator/src/auditor_mode.rs` (+~100 LOC; mutation + property gate integration, D-3 stand-in)
- MODIFY: `.elai_cc/crates/orchestrator/src/per_level_audit_dispatch.rs` (+~40 LOC; mutation_score evidence in per_level_critic_verdict)
- MODIFY: `.elai_cc/crates/orchestrator/src/ssb.rs` (+~50 LOC; 7 new SsbActionType variants)
- MODIFY: `.elai_cc/elai_cc.toml` (+~40 LOC; 3 new tables)
- MODIFY: `.claude/tools/spec_link_check.py` (+~30 LOC; properties.md presence check)
- MODIFY: `.elai_cc/crates/orchestrator/src/fsm.rs` (+~15 LOC; FsmSubstate::MutantHunting variant)
- CREATE: `.elai_cc/crates/dashboard-v2/src/lib/components/MutationGateGauge.svelte` (~140 LOC)
- CREATE: `.elai_cc/crates/dashboard-v2/src/lib/components/PropertyCoverageGauge.svelte` (~120 LOC)
- CREATE: `.elai_cc/crates/dashboard-v2/src/lib/components/OverMockHistogram.svelte` (~140 LOC)
- MODIFY: `.elai_cc/crates/dashboard-v2/src/lib/drawers/ReasoningDrawer.svelte` (+~30 LOC; mount the 3 new components)
- MODIFY: `.elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs` (+~80 LOC; 3 new Tauri commands)

## Citations / Grounding
- arXiv:2506.18315 PGS — +13.4% pass@1 over example-TDD; property gate grounding
- arXiv:2510.09907 Agentic-PBT — 56% valid bug rate; 86% precision at top-21
- arXiv:2506.09289 UTBoost — 24.4% Verified leaderboard inflation; vacuous-test ban grounding
- arXiv:2501.12862 Meta ACH — 73% engineer acceptance; mutant-hunter agent grounding
- arXiv:2602.00409 Hora & Robbes 2026 — 36% vs 26% mock-add rate; anti-over-mock grounding
- arXiv:2503.05061 No Free Labels — TPR>96% / TNR<25%; non-LLM verifier requirement
- arXiv:2509.02360 PRMs for SWE agents — process-reward gating context
- arXiv:2502.14212 Less Is More / CleanTest — 43.52% Methods2Test noise; quality > volume
- arXiv:2404.09952 LLMorpheus — LLM-mutated source finds faults Stryker can't
- arXiv:2308.04748 Fuzz4All — 98 bugs / 64 novel across 6 languages; fuzz gate grounding
- arXiv:2504.16828 ThinkPRM — process reward model evidence
- engineering.fb.com Meta ACH blog 2025 — fault-as-goal vs coverage-as-goal policy
- github.com/sourcefrog/cargo-mutants — production Rust mutation tester; pin version 27.x

## Open questions / risks
- Ungrounded: 70% mutation-score threshold inherited from C-28; the report's open Q1 calls for an empirical local-7B kill-rate measurement (target ≥40% kill rate). Plan to measure on first 5 ELAI crates and re-tune.
- mutant-hunter agent depends on local Qwen-2.5-Coder-7B context (20K-token ceiling per CLAUDE.md); long files may not fit. Add a per-mutant context budget guard and skip-with-WARN if file > 18K tokens.
- cargo-mutants version pin (27.x.x) is a moving target; the toolchain pin in elai_cc.toml [testing.mutation_targeted].cargo_mutants_pin must be verified quarterly via paper-ingestor mode=source.
- Property gate's "≥10 cases" floor is conservative; PGS used 100. Tune via [testing.property].min_cases once data is available.
- FuzzGate at CONCEPT level adds 60-120s per audit — measurable cost. Open Q5 in the report (proptest seed corpus per FSM transition) becomes a follow-up.
- The "all mutants of the real impl survive" check in anti_mock requires running mutation testing on the impl before the diff is committed; this can be expensive on a large impl. Add a cost-gate that only runs the cross-check when the impl file is <500 LOC.
- D-3 may ship after E-13; the local stand-in in aggregate_verifiers must be replaced by D-3's surface when D-3 lands. Schedule as a post-D-3 patch.
- Local-7B context limit (25K max per CLAUDE.md) constrains the mutant-hunter agent on large files; the agent prompt must summarize the function under test before passing context.

## Next
- Measure local-7B mutant-test-kill rate on 5 ELAI crates with mutant-hunter agent; report in workflow/reasoning/E-13-measurement.md.
- E-14 (browser E2E) plugs into the integration_test_runner with Playwright as one more non-LLM verifier.
- D-3 absorption: when D-3 ships, move the non-LLM-verifier constraint from local stand-in to D-3's `consensus_rule` field.
- Open Q3 from the report (anti-over-mock threshold empirical curve) becomes a measurement-only patch after E-13 ships.
