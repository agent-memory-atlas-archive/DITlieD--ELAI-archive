plan: Post-Control (POS) Testing Pipeline
created: 2026-03-30
status: pending

Goal: Add deterministic post-generation enforcement to the existing pre-control workflow. The LLM writes code and tests (via TESTING_FIRST -> EXECUTING), the auditor reviews (AUDITING), and then git hooks provide the FINAL gate — nothing commits unless the pipeline passes. This is the missing layer.

Core principle: "If you can quantify/measure it = test it. The pipeline is the enforcement mechanism — not trust in the LLM."


WHO WRITES THE TESTS

Olscore wrote 5-10k tests himself initially, then the LLM took over. Jest is just a test runner — same as pytest. The LLM writes both code AND tests.

Our workflow already handles this: TESTING_FIRST state forces tests before code. The LLM writes tests, then writes code to pass them. The auditor runs the tests.

The problem for a non-coder: the LLM can write tests that LOOK correct but test nothing. "assert result is not None" passes even if the function returns garbage. The LLM is happy, the test passes, and the code is wrong.

Solution: test the tests. A quality scanner checks that tests actually verify behavior (assertion count, assertion quality, coverage). This is what olscore's "Test Quality" metric does. We add this as a pipeline tier.

pytest IS the Python Jest. Same role: define expected behavior, run the code, check it matches. The LLM writes both sides. The pipeline validates both sides.


INTEGRATION WITH EXISTING WORKFLOW

Current flow:
  TESTING_FIRST (LLM writes tests) -> EXECUTING (LLM writes code) -> AUDITING (auditor runs tests + reviews)

New flow (POS added):
  TESTING_FIRST (LLM writes tests) -> EXECUTING (LLM writes code) -> AUDITING (auditor runs TIERED pipeline + reviews) -> git commit -> PRE-COMMIT HOOK (pipeline runs again as final gate)

Where the POS pipeline plugs in:
1. AUDITING state: The auditor already runs "python -m pytest tests/ -v". Change this to run the tiered pipeline instead. Same auditor agent, same FSM state, but structured execution: Tier 1 (determinism) -> Tier 2 (quality + test quality) -> Tier 3 (unit) -> Tier 4 (integration). Fail at any tier = BLOCK.
2. Pre-commit hook: After auditor passes and FSM reaches MEMORIZING/COMPLETE, the LLM (or user) commits. The hook re-runs Tier 1-3 as a safety net. This catches anything the auditor session missed or that was edited after audit.
3. Existing validators (.claude/tools/validate_*.py): Already exist. Wrap them into Tier 5 (stress) and Tier 6 (conformance) markers so they run as part of the pipeline instead of ad-hoc.

What changes in the auditor agent:
- Instead of flat "pytest tests/ -v", run "scripts/run_pipeline.py --tiers 1,2,3,4"
- Tier results replace the current pass/fail. BLOCK if any tier fails.
- Test quality score added to the PROGRESS SUMMARY line.
- Existing trading AST scan and domain validators integrated as specific tiers rather than separate steps.

What changes in planning-workflow skill:
- AUDITING section updated: auditor runs tiered pipeline, not flat pytest
- TESTING_FIRST section updated: tests must include pytest markers (unit, integration, stress, conformance) so the pipeline knows which tier they belong to
- No other FSM changes needed. The states stay the same.


DESIGN: THE POS PIPELINE

Tier 1: DETERMINISM (fast, <10s)
  Purpose: Catch flaky tests before they waste time in later tiers
  Checks:
  - Run unit tests 2x, compare results (flaky = quarantine, don't block but report)
  - AST scan test files for time.time(), datetime.now(), requests.get() without mocking
  - No random seeds without explicit setting
  - No floating point equality without tolerance
  When it runs: Every auditor pass. Every pre-commit hook.

Tier 2: QUALITY (fast, <15s)
  Purpose: Static analysis + test quality
  Checks:
  - ruff lint (already in auditor PostToolUse hook — move to pipeline for consistency)
  - trading_ast_scan.py (already exists in .claude/tools/)
  - Zero-assertion test detection (tests that call code but never assert behavior)
  - Weak-assertion detection ("assert result is not None", "assert True", "assert len(x) > 0" without checking content)
  - Method/function length limits (flag >50 lines)
  - File length limits (flag >300 lines)
  - Test-to-code ratio per module (flag modules with 0 tests)
  When it runs: Every auditor pass. Every pre-commit hook.

Tier 3: UNIT (fast, <30s)
  Purpose: All fast deterministic tests pass
  Checks:
  - pytest tests/ -m "unit" -v (or -m "not operational and not stress and not conformance")
  - Coverage on changed files (warn below 70%, block below 50%)
  When it runs: Every auditor pass. Every pre-commit hook.

Tier 4: INTEGRATION (<60s)
  Purpose: Cross-module contracts hold
  Checks:
  - pytest tests/contracts/ -v (contract tests from context-compiler interface maps)
  - pytest tests/ -m "integration" -v
  When it runs: Every auditor pass. Optional in pre-commit (can skip with flag for speed).

Tier 5: STRESS (<5min, auditor only, not in pre-commit)
  Purpose: Edge cases, adversarial inputs
  Checks:
  - validate_execution_engine.py (zero variance, liquidity squeeze)
  - validate_strategy_null.py (white noise, shuffle)
  - hypothesis property-based tests
  - Adversarial market scenarios (flash crash, gap open, zero volume, NaN injection)
  When it runs: Auditor pass only. On-demand before deploy. Not in pre-commit (too slow).

Tier 6: CONFORMANCE (scheduled, not in pre-commit)
  Purpose: Matches real-world behavior
  Checks:
  - validate_ohlcv.py (data quality)
  - validate_predictions.py (model quality)
  - validate_drift.py (concept drift PSI)
  - Broker API schema conformance
  - Baseline reconciliation (replay signals, compare against saved known-good output)
  When it runs: Scheduled (daily/weekly). Pre-deploy. Not in regular auditor pass.


IMPLEMENTATION PHASES

Phase 1: Pipeline Runner + Git Hooks
  Done when: "scripts/run_pipeline.py --tiers 1,2,3" runs and git commit is blocked on failure
  Deliverables:
  1. run_pipeline.py: sequential tier runner, stops on first failure, outputs summary
  2. Pre-commit hook that calls run_pipeline.py
  3. pytest marker configuration in pyproject.toml (unit, integration, stress, conformance, operational)
  Files: scripts/run_pipeline.py, .githooks/pre-commit, pyproject.toml
  Steps:
  1. Add pytest markers to pyproject.toml
  2. Write run_pipeline.py that runs tiers sequentially, collects results, exits non-zero on failure
  3. Write pre-commit hook that calls run_pipeline.py --tiers 1,2,3
  4. Configure git to use .githooks/ (git config core.hooksPath .githooks)
  Tests (UNIT): test that pipeline exits non-zero when a tier fails
  Expected outcome: No commit possible without Tier 1-3 passing
  Status: complete

Phase 2: Test Quality Scanner
  Done when: Zero-assertion and weak-assertion tests are detected and reported as BLOCK
  Deliverables:
  1. scripts/test_quality_scan.py: AST-based scanner that reads test files, counts/classifies assertions per test function
  2. Integration into Tier 2 of pipeline
  Files: scripts/test_quality_scan.py
  Steps:
  1. AST-walk test files, find test functions (def test_*), count assert statements
  2. Classify assertions: strong (checks specific value), weak (assert is not None, assert True), zero
  3. BLOCK: any test function with 0 assertions. WARN: >50% weak assertions in a test function
  4. Add to run_pipeline.py as part of Tier 2
  Tests (UNIT): feed it a test file with known good/bad tests, verify detection
  Expected outcome: LLM cannot write empty tests that pass by doing nothing
  Status: complete

Phase 3: Determinism Gate
  Done when: Flaky tests detected and quarantined, non-determinism sources flagged
  Deliverables:
  1. scripts/determinism_check.py: runs tests 2x, diffs results, quarantines flaky ones
  2. AST scanner addition: flag time/network/random usage in test files without mocking
  Files: scripts/determinism_check.py
  Steps:
  1. Run pytest twice, capture results, compare per-test pass/fail
  2. Tests that flip = quarantine marker, reported but not blocking
  3. AST scan test files for non-determinism sources (time, network, random) without corresponding mock/patch
  4. Add to run_pipeline.py as Tier 1
  Tests (UNIT): intentionally flaky test, verify detection
  Expected outcome: Flaky tests can't give false confidence
  Status: complete

Phase 4: Auditor Integration
  Done when: Adversarial auditor runs tiered pipeline instead of flat pytest
  Deliverables:
  1. Updated auditor invocation in planning-workflow to use run_pipeline.py
  2. Pipeline summary integrated into auditor PROGRESS SUMMARY format
  3. Existing .claude/tools/validate_*.py wrapped as Tier 5-6 pytest fixtures
  Files: .claude/agents/auditor.md (update run command), .claude/skills/planning-workflow/SKILL.md (update AUDITING section)
  Steps:
  1. Change auditor test command from "pytest tests/ -v" to "scripts/run_pipeline.py --tiers 1,2,3,4,5"
  2. Update PROGRESS SUMMARY format to include tier results and test quality score
  3. Wrap validate_*.py scripts as pytest fixtures callable via Tier 5 marker
  4. Update planning-workflow AUDITING section to reference tiered pipeline
  Tests (UNIT): auditor produces correct PROGRESS SUMMARY with tier data
  Expected outcome: Auditor and pipeline are unified, not two separate systems
  Status: complete

Phase 5: Stress + Adversarial Tests
  Done when: Property-based and adversarial tests exist, run as Tier 5
  Deliverables:
  1. hypothesis property-based tests for indicator calculations
  2. Adversarial market scenario tests (flash crash, gap, zero volume, NaN)
  3. Existing validators (execution engine, strategy null) callable as pytest
  Files: tests/stress/*.py, tests/adversarial/*.py
  Steps:
  1. Wrap validate_execution_engine.py and validate_strategy_null.py as pytest fixtures
  2. Write hypothesis tests: indicator outputs always finite, always correct length, NaN only in warmup
  3. Write adversarial tests: extreme inputs, boundary conditions
  4. All marked @pytest.mark.stress
  Tests: the stress tests themselves
  Expected outcome: Edge cases caught before deployment
  Status: complete

Phase 6: Conformance + Reconcile
  Done when: Baseline recording works, replay comparison catches regressions
  Deliverables:
  1. Baseline recorder (save strategy signals + metrics to file)
  2. Reconciliation runner (replay, compare within tolerance)
  3. Existing validators (OHLCV, predictions, drift) as Tier 6 fixtures
  Files: scripts/baseline_record.py, scripts/reconcile.py, tests/conformance/*.py
  Steps:
  1. Write baseline recorder: run strategy on fixed data, save signals + metrics JSON
  2. Write reconcile script: re-run, compare against baseline, report drift outside tolerance
  3. Wrap validate_ohlcv.py, validate_predictions.py, validate_drift.py as conformance fixtures
  4. All marked @pytest.mark.conformance
  Tests (UNIT): reconcile logic with known matching/drifting baselines
  Tests (OPERATIONAL): full replay against real saved baseline
  Expected outcome: Subtle regressions caught that unit tests miss
  Status: complete


WHAT CHANGES IN EXISTING FILES

.claude/agents/auditor.md:
  - Test command changes from "python -m pytest tests/ -v" to "python scripts/run_pipeline.py --tiers 1,2,3,4,5"
  - PROGRESS SUMMARY format adds: "Tiers: 1-5 passed | TestQuality: X.XX"
  - Domain validators already listed — just reference their tier assignment

.claude/skills/planning-workflow/SKILL.md:
  - AUDITING section: mention tiered pipeline, reference run_pipeline.py
  - TESTING_FIRST section: add rule "every test function must have a pytest marker (unit, integration, stress, conformance). Unmarked tests default to unit."
  - No FSM state changes. No new states. Pipeline is a tool, not a state.

pyproject.toml:
  - Add [tool.pytest.ini_options] markers for: unit, integration, stress, conformance, operational, quarantine

.githooks/pre-commit:
  - New file. Runs "python scripts/run_pipeline.py --tiers 1,2,3" and exits non-zero on failure.


KEY DECISIONS

1. Pipeline is a tool, not a state. No new FSM states. The auditor calls the pipeline script. Git hooks call the same script. One tool, two enforcement points.

2. Auditor owns deep tiers (1-5). Git hooks own fast tiers (1-3). Conformance/reconcile are scheduled, not gated.

3. Test markers are mandatory. Every test function gets @pytest.mark.unit (or integration/stress/conformance). This lets the pipeline run the right tests at the right time. Default = unit.

4. Test quality scanner is AST-based. Same approach as existing trading_ast_scan.py. Reads the test code structurally, doesn't run it. Fast.

5. Existing validators stay as scripts. Wrapped as pytest fixtures for pipeline integration, but the original scripts in .claude/tools/ stay untouched for standalone use.


PRIORITY ORDER

Phase 1 first (the "entire unlock" — git hooks). Phase 2 immediately after (test quality — the non-coder's biggest risk). Phase 3 can parallel with Phase 2. Phase 4 integrates everything into the auditor. Phase 5-6 are depth tiers, build when needed.

Current position: Active phase: none, Last step: none, Next: Phase 1
