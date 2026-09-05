# C-54: SWE-bench Verified Run — 3 Configs (FLAGSHIP BENCHMARK, RUNPOD-DEPENDENT)

Objective: produce ELAI's first published SWE-bench Verified number. Three
config arms: (A) local-7B-only, (B) local-7B + local-7B-auditor (hybrid local),
(C) hybrid Sonnet 4.6 designer + Composer 2 executor + local auditor. The
single most-leveraged measurement in the C-35..C-59 batch. Without this number,
every "ELAI is differentiated" claim is unmeasured.

Pre-requisites:
- C-21 benchmarks/swe_bench/ task_01..10.toml SHIPPED (baseline pre-existed,
  per prior oracle 2026-05-17).
- C-35 (Tool Attention gating) SHIPPED — published number must reflect
  production prompt shape.
- C-37 (PER_ROLE_DECODING_DEFAULTS re-eval) at least RUN even if not flipped,
  so methodology is documented.
- C-39 (CLI MCP surface) SHIPPED.
- C-42 (zero-trust fuzzer) SHIPPED — bridge must be hardened before
  public-benchmark exposure.
- C-44 (Composer 2 backend) SHIPPED for config C; otherwise downgrade C to
  Sonnet-only.
- C-51 (Aider Polyglot harness) NOT required — that's C-55. SWE-bench has
  its own harness.
- **RunPod 4090/A100/H100 instance online and accessible** — the hardware
  gate that makes this patch fire.

Parallel-safe with: C-55, C-56, C-57, C-58 (all benchmark patches, can run in
sequence on the same RunPod session). NOT parallel-safe with C-35..C-53
implementation patches if they're touching production prompt path while a
benchmark is mid-run (lock the binary at run start).

## Context

SWE-bench Verified is the de-facto industry benchmark for autonomous coding
agents. Every named competitor in the prior oracle report has a published
number:
- Devin 2.0: 45.8%
- OpenHands: 72-77%
- Claude Code (Sonnet 4.6 / Opus 4.7): joint-top with GPT-5.4 per SWE Atlas
- Aider: published on its own Polyglot leaderboard; SWE-bench numbers
  derived per backend
- mini-SWE-agent: >74% in 100 lines

ELAI has nothing published. Maintainer's honest assessment §8: "NOT
benchmarked head-to-head against Devin (SWE-bench), Aider (Aider's own
leaderboard), Sweep, OpenInterpreter, or Cursor agentic mode... no end-to-end
p50/p99 metric in any phase_*.toml." C-54 fixes that.

Three configs are NOT redundant. They isolate variables:
- **Config A (local-7B-only)**: tells us what ELAI's infrastructure-as-substitute-for-parameters
  thesis is worth at the smallest model size.
- **Config B (local 7B + local 7B-auditor)**: tells us if a SECOND local model
  in the auditor role rescues solve-rate vs single-model.
- **Config C (hybrid)**: tells us what the SAME ELAI infrastructure produces
  with cloud-grade per-call quality.

The three numbers + cost + wall-clock give a honest position. NOT all three
must "beat" anyone. They must be measured.

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-21_self_mod_anti_stale_automation.md
  (confirm SWE-bench harness scaffolding lands per C-21)
- .elai_cc/benchmarks/swe_bench/ (task_01..10 TOML; check current task count
  — may need full 500 SWE-bench Verified imported)
- swebench.com (fresh fetch for current Verified split)

## Tasks

1. **Task corpus completion** — current C-21 ships task_01..10. SWE-bench
   Verified has 500 tasks. Import the full set from swebench.com under
   .elai_cc/benchmarks/swe_bench/verified/. Pin SWE-bench version SHA in
   benchmarks/swe_bench/manifest.toml for reproducibility.

2. **Harness adaptation** — SWE-bench expects a docker-based eval per task.
   ELAI's task runner needs a SWE-bench-compatible mode: clone target repo
   at the pre-issue SHA, present the issue text + repo to ELAI, collect
   the emitted patch, apply, run the hidden test suite, report pass/fail.
   Reuse C-11 sandbox-completeness for isolation.

3. **Three config TOMLs** under .elai_cc/benchmarks/swe_bench/configs/:
   - config-A-local-7B.toml: backends = [local_qwen_7b], roles =
     {executor, auditor} both bound to local 7B
   - config-B-local-7B-7B.toml: backends = [local_qwen_7b], two separate
     local instances for executor and auditor (memory permitting; if 1
     GPU only, sequential)
   - config-C-hybrid.toml: backends = [sonnet_46, composer2, local_qwen_7b],
     roles routed per Phase 4M default policy + skill-routing from C-41

4. **Three-run protocol** — for each config:
   - Run full 500 tasks
   - Record per-task: solved, time_s, llm_calls_total, tokens_in, tokens_out,
     cost_usd, mutation_score (post-solve, via C-28), MR_violations,
     wiring_contract_pass
   - Aggregate: solve-rate, mean cost, p50/p99 time, mean mutation_score
   - Emit JSON scorecard at workflow/sessions/{id}/swe_bench_run_{config}/scorecard.json

5. **Methodology document** — workflow/sessions/{id}/swe_bench_verified_methodology.md
   detailing:
   - SWE-bench version SHA
   - Hardware (RunPod instance type, GPU model, RAM)
   - Each config's full elai_cc.toml snapshot
   - C-35 gating state (on/off), C-37 decoding mode state at run time
   - Total wall-clock and total cost per config
   - Outliers (tasks that timed out / errored vs honestly failed)
   - Re-run policy (a task is run ONCE per config; no cherry-picking)

6. **Publication artifact** — .elai_cc/docs/swe_bench_verified_results.md
   public-facing summary with:
   - The 3 numbers + cost matrix
   - Methodology link
   - Honest caveats (where ELAI lost; e.g. config A likely 5-15% if
     paper-class 7B baselines hold)
   - Comparison table vs Devin / OpenHands / mini-SWE-agent / Claude Code
     using THEIR published numbers + methodology (cite version + date for
     each external number)
   - **Honesty statement**: "ELAI's published number is single-attempt
     pass@1 on SWE-bench Verified version SHA X, executed on hardware Y,
     at total cost Z. No cherry-picking. Hyperparameters fixed per
     config TOML. Re-run produces identical scorecard ±0 tasks under
     SEEDED determinism (config A); ±1-3 tasks under hybrid stochasticity
     (config C, due to cloud-tier sampling temperature > 0)."

7. **SSB action types** — swe_bench_run_started, swe_bench_task_completed,
   swe_bench_scorecard_emitted, swe_bench_methodology_locked.

8. **Tests first (harness sanity, no GPU)**:
   - 4G.X.3: harness clones a known-good SWE-bench task fixture at the
     pre-issue SHA
   - 4G.X.4: hidden-test runner reports pass on a known-good patch
   - 4G.X.5: hidden-test runner reports fail on a deliberately-bad patch
   - 4G.X.6: scorecard JSON conforms to SWE-bench official format
   - 4G.X.7: methodology.md is auto-generated from config TOML + run
     metadata (no human-editable section that could drift)

9. **CI integration** — NOT part of normal CI (too expensive). Manual
   trigger via `elai benchmark swe-bench --config config-C-hybrid.toml --runpod-id <id>`.

10. **Re-run discipline** — if any of the 3 runs error mid-sweep (RunPod
    instance evicted, network blip, etc), the partial scorecard is
    discarded; full run restarts. Documented in methodology.md re-run
    policy.

## WIRING_CONTRACT

produces_symbols:
  - 500-task SWE-bench Verified corpus under benchmarks/swe_bench/verified/
  - 3 config TOMLs
  - SwebenchHarness (crates/task_runner/src/swebench_harness.rs)
  - 3 scorecard JSONs
  - workflow/sessions/{id}/swe_bench_verified_methodology.md
  - .elai_cc/docs/swe_bench_verified_results.md
  - SSB action_types: swe_bench_run_started, swe_bench_task_completed,
    swe_bench_scorecard_emitted, swe_bench_methodology_locked
consumes_symbols:
  - C-11 sandbox-completeness layer
  - All 3 backend tiers (local 7B, Sonnet 4.6, Composer 2)
  - C-28/C-29 testing-trust metrics (mutation, MR)
  - SSB event bus
reachable_from:
  - elai benchmark swe-bench --config <config> --runpod-id <id>
stub_ban:
  - methodology.md MUST be auto-generated from machine-readable config
    + run metadata; NO human-editable narrative section that could drift
    from actual numbers
  - re-run discipline MUST discard partials; published number MUST be
    a single completed sweep
  - results.md MUST cite version SHA + hardware + cost; "ELAI scored X%"
    without these is incomplete
  - honesty statement MUST be present verbatim

## DETERMINISTIC_CHECK

```bash
# 1. Full 500-task corpus
ls .elai_cc/benchmarks/swe_bench/verified | wc -l | awk '$1 >= 500 {exit 0} {exit 1}'

# 2. Pinned SHA
grep -q "swe_bench_version_sha" .elai_cc/benchmarks/swe_bench/manifest.toml

# 3. 3 config files
test -f .elai_cc/benchmarks/swe_bench/configs/config-A-local-7B.toml
test -f .elai_cc/benchmarks/swe_bench/configs/config-B-local-7B-7B.toml
test -f .elai_cc/benchmarks/swe_bench/configs/config-C-hybrid.toml

# 4. Harness compiles
cargo build -p elai_task_runner --features swebench

# 5. SSB action types
rg "swe_bench_run_started|swe_bench_task_completed|swe_bench_scorecard_emitted|swe_bench_methodology_locked" .elai_cc/crates/orchestrator/src

# 6. After run: 3 scorecards exist
test -f workflow/sessions/$(cat workflow/sessions/current.txt)/swe_bench_run_A/scorecard.json
test -f workflow/sessions/$(cat workflow/sessions/current.txt)/swe_bench_run_B/scorecard.json
test -f workflow/sessions/$(cat workflow/sessions/current.txt)/swe_bench_run_C/scorecard.json

# 7. Methodology and results docs
test -f workflow/sessions/$(cat workflow/sessions/current.txt)/swe_bench_verified_methodology.md
test -f .elai_cc/docs/swe_bench_verified_results.md
grep -c "honesty statement" .elai_cc/docs/swe_bench_verified_results.md
```

## ACCEPTANCE

- [ ] Full SWE-bench Verified 500-task corpus imported with pinned SHA
- [ ] 3 config TOMLs authored
- [ ] SwebenchHarness implemented, sandboxed-test invocation works
- [ ] Harness sanity TDD tests pass (no GPU needed)
- [ ] **Config A run completed** — 500 tasks against local 7B only;
      scorecard.json present
- [ ] **Config B run completed** — 500 tasks against dual-local 7B
      (executor + auditor); scorecard.json present
- [ ] **Config C run completed** — 500 tasks hybrid Sonnet+Composer2+local;
      scorecard.json present
- [ ] methodology.md generated from machine-readable inputs
- [ ] results.md authored with comparison table + honesty statement
- [ ] All 4 SSB action types emit per run
- [ ] No cherry-picking, no partial sweeps published

## Files

- ADD: .elai_cc/benchmarks/swe_bench/verified/* (500 tasks)
- ADD: .elai_cc/benchmarks/swe_bench/manifest.toml
- ADD: .elai_cc/benchmarks/swe_bench/configs/config-A-local-7B.toml
- ADD: .elai_cc/benchmarks/swe_bench/configs/config-B-local-7B-7B.toml
- ADD: .elai_cc/benchmarks/swe_bench/configs/config-C-hybrid.toml
- ADD: .elai_cc/crates/task_runner/src/swebench_harness.rs
- ADD: .elai_cc/crates/task_runner/tests/swebench_harness_tdd.rs
- ADD: workflow/sessions/{id}/swe_bench_run_A/scorecard.json (post-run)
- ADD: workflow/sessions/{id}/swe_bench_run_B/scorecard.json (post-run)
- ADD: workflow/sessions/{id}/swe_bench_run_C/scorecard.json (post-run)
- ADD: workflow/sessions/{id}/swe_bench_verified_methodology.md (post-run)
- ADD: .elai_cc/docs/swe_bench_verified_results.md (post-run)
- MODIFY: .elai_cc/crates/cli/src/main.rs (benchmark swe-bench subcommand)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb/schema.rs (4 new action types)

## Affected Verification Reports

- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\C-plan\C-21_self_mod_anti_stale_automation.report.md
  — annotate the SWE-bench full corpus arrival.

## Next

C-54 closure is THE single biggest credibility step in the C-35..C-59 batch.
Until C-54 publishes a number, ELAI cannot defend its differentiation
claim against any named competitor. After C-54: C-55 (Aider Polyglot) and
C-56 (Codebase-Memory) run as independent benchmark dimensions, building
out the published evidence base. C-59 (master benchmark report) aggregates
all of them into one cross-comparison artifact.
