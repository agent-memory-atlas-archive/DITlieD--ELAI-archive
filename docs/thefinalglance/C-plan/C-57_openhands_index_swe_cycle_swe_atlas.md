# C-57: OpenHands Index + SWE-Cycle + SWE Atlas Combo Run (RUNPOD-DEPENDENT)

Objective: run ELAI through three additional public benchmarks in one
RunPod session to amortize hardware cost: OpenHands Index (issue resolution +
greenfield + frontend + testing), SWE-Cycle (FullCycle execution drop test),
SWE Atlas (Codebase Q&A + Test Writing + Refactoring).

Pre-requisites: C-54 SHIPPED — primary SWE-bench Verified number must
exist first; this patch is the supplementary benchmark layer.
Parallel-safe with: C-54, C-55, C-56, C-58 (sequential on same RunPod).

Reference grounding (verified via prior oracle 2026-05-17):
- OpenHands Index: openhands.dev/blog/openhands-index (Jan 2026); 72%
  SWE-bench Verified internal baseline; multi-axis evaluation.
- SWE-Cycle: arXiv:2605.13139 (2026-05-13): "Claude-Sonnet-4.6 achieves the
  best performance"; "sharp drop in solve rates when transitioning from
  isolated tasks to FullCycle execution."
- SWE Atlas: arXiv:2605.08366 (2026-05-08): "GPT-5.4 and Opus 4.7 achieve
  the strongest overall performance, while even the best open-weight models
  score poorly"; rubric-based across Codebase Q&A (124 tasks), Test Writing
  (90 tasks), Refactoring (70 tasks).

## Context

These three benchmarks measure different dimensions:
- **OpenHands Index**: real-world repo issue resolution + frontend work +
  test writing. Closest analog to actual developer workflows.
- **SWE-Cycle**: tests whether ELAI's full FSM (PLANNING → EXECUTING →
  AUDITING → MEMORIZING) survives the "FullCycle" extension where the
  agent must operate on its OWN prior outputs across multiple iterations
  per task. Direct test of MSI auto-revert (Phase 4H) + bubble-up policy.
- **SWE Atlas**: explicitly notes "open-weight models score poorly."
  Honest test of how badly ELAI's local-only config trails GPT-5.4 / Opus
  4.7 — and how much hybrid mode closes the gap.

Single config for each benchmark unless otherwise noted (default ELAI
hybrid mode); cost-conscious choice because three benchmarks × three configs
× hundreds of tasks blows the RunPod budget. ELAI's local-only number from
C-54 covers the "how bad is open-weight" axis already.

## Reads

- openhands.dev/blog/openhands-index (fresh fetch — methodology + task set)
- arXiv:2605.13139 + reference repo
- arXiv:2605.08366 + reference repo (if released)

## Tasks

1. **Source-grounding** — paper-ingestor (mode=paper) for SWE-Cycle and
   SWE Atlas. WebFetch for OpenHands Index methodology.

2. **Three harness adapters** — each benchmark has its own task format:
   - crates/task_runner/src/openhands_index_adapter.rs
   - crates/task_runner/src/swe_cycle_adapter.rs
   - crates/task_runner/src/swe_atlas_adapter.rs
   Each clones / loads tasks, presents to ELAI, collects emitted patches /
   answers, scores per benchmark's official rubric.

3. **Per-benchmark config** — single ELAI hybrid config for each (Sonnet 4.6
   designer + Composer 2 executor + local auditor) unless one benchmark
   has a published local-only baseline worth comparing against (in which
   case add a local-only arm).

4. **Run-loop discipline** — same as C-54: single-attempt pass@1, no
   cherry-picking, scorecard JSON per benchmark.

5. **Benchmark-specific scoring** —
   - OpenHands Index: matches OpenHands' published scoring per axis
   - SWE-Cycle: tracks per-cycle solve-rate drop (paper's key finding)
   - SWE Atlas: rubric-based via Sonnet 4.6 judge (or paper's released
     judge if available)

6. **Methodology + results doc** — single combined methodology + results
   for all three (cost amortization in one run on one RunPod instance).

7. **Specific ELAI-side findings to surface**:
   - SWE-Cycle: does ELAI's FSM produce a smaller solve-rate drop than the
     paper's reported baseline ? (this is the paper's open question)
   - SWE Atlas: ELAI hybrid vs open-weight tier (Llama / Qwen baselines) at
     same parameter class
   - OpenHands Index: ELAI per-axis breakdown vs OpenHands' published
     per-axis numbers

8. **SSB action types** — multi_benchmark_run_started, multi_benchmark_axis_completed
   (per axis per benchmark), multi_benchmark_scorecard_emitted.

## WIRING_CONTRACT

produces_symbols:
  - 3 harness adapters
  - 3 scorecard JSONs (one per benchmark)
  - methodology doc + results doc (combined for the 3)
  - SSB action_types: multi_benchmark_run_started, multi_benchmark_axis_completed,
    multi_benchmark_scorecard_emitted
consumes_symbols:
  - 3 backend tiers (after C-44)
  - SSB event bus
reachable_from:
  - elai benchmark openhands-index | swe-cycle | swe-atlas --runpod-id <id>
  - elai benchmark suite --runpod-id <id> (runs all three)
stub_ban:
  - per-axis scores MUST be visible; not just aggregate
  - SWE-Cycle solve-rate per cycle MUST be reported separately (the paper's
    finding is the CURVE)
  - all comparisons against external baselines MUST cite version + date

## DETERMINISTIC_CHECK

```bash
test -f .elai_cc/crates/task_runner/src/openhands_index_adapter.rs
test -f .elai_cc/crates/task_runner/src/swe_cycle_adapter.rs
test -f .elai_cc/crates/task_runner/src/swe_atlas_adapter.rs
test -f workflow/sessions/$(cat workflow/sessions/current.txt)/openhands_index/scorecard.json
test -f workflow/sessions/$(cat workflow/sessions/current.txt)/swe_cycle/scorecard.json
test -f workflow/sessions/$(cat workflow/sessions/current.txt)/swe_atlas/scorecard.json
test -f .elai_cc/docs/multi_benchmark_results.md
rg "multi_benchmark_run_started|multi_benchmark_axis_completed|multi_benchmark_scorecard_emitted" .elai_cc/crates/orchestrator/src
```

## ACCEPTANCE

- [ ] paper-ingestor produces SWE-Cycle + SWE Atlas interface docs
- [ ] OpenHands Index methodology documented from fresh fetch
- [ ] 3 harness adapters implemented
- [ ] **OpenHands Index run completed** — per-axis scorecard
- [ ] **SWE-Cycle run completed** — per-cycle solve-rate curve documented
- [ ] **SWE Atlas run completed** — per-rubric scorecard (Codebase Q&A,
      Test Writing, Refactoring)
- [ ] All 3 SSB action types emit
- [ ] multi_benchmark_results.md authored with honest cross-comparison
- [ ] Specific findings (Task 7) surfaced
- [ ] Cost amortization verified: total RunPod cost < 3 × independent run cost

## Files

- ADD: .elai_cc/crates/task_runner/src/openhands_index_adapter.rs
- ADD: .elai_cc/crates/task_runner/src/swe_cycle_adapter.rs
- ADD: .elai_cc/crates/task_runner/src/swe_atlas_adapter.rs
- ADD: .elai_cc/benchmarks/openhands_index/* (vendored corpus)
- ADD: .elai_cc/benchmarks/swe_cycle/* (vendored corpus)
- ADD: .elai_cc/benchmarks/swe_atlas/* (vendored corpus, if released)
- ADD: workflow/sessions/{id}/openhands_index/scorecard.json (post-run)
- ADD: workflow/sessions/{id}/swe_cycle/scorecard.json (post-run)
- ADD: workflow/sessions/{id}/swe_atlas/scorecard.json (post-run)
- ADD: workflow/sessions/{id}/multi_benchmark_methodology.md (post-run)
- ADD: .elai_cc/docs/multi_benchmark_results.md (post-run)
- ADD: workflow/papers/swe-cycle-interface.md
- ADD: workflow/papers/swe-atlas-interface.md
- MODIFY: .elai_cc/crates/cli/src/main.rs (3 new benchmark subcommands + suite)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb/schema.rs (3 new action types)

## Next

C-57 closure broadens ELAI's published evidence base from single-axis SWE-bench
(C-54) to multi-dimensional. Cost-amortized via one RunPod session covering
all three. Pairs with C-59 which aggregates these + C-54 + C-55 + C-56 into
one cross-comparison report.
