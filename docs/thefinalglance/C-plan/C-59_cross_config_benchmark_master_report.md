# C-59: Cross-Config Benchmark Matrix + Cost-Per-Task Master Report (CAPSTONE)

Objective: aggregate C-54 (SWE-bench Verified) + C-55 (Aider Polyglot) + C-56
(Codebase-Memory) + C-57 (OpenHands Index + SWE-Cycle + SWE Atlas) + C-58
(AgentForge head-to-head + RRK reranker) into ONE public-facing master
benchmark report. Optionally pilot Test-Time Speculation (arXiv:2605.09329)
as a per-config speedup if RunPod time permits.

Pre-requisites: C-54 + C-55 + C-56 + C-57 + C-58 all SHIPPED (with their
runs completed). This is the capstone patch — closes the C-35..C-59 batch.
Parallel-safe with: none of the benchmark patches (it consumes their
outputs). Independent of all C-35..C-53 implementation patches.

Paper grounding (verified via prior oracle 2026-05-17):
- Test-Time Speculation (arXiv:2605.09329, 2026-05-10): TTS achieves "up
  to 72% and 41% on average" improvements in acceptance lengths. Optional
  pilot in this patch.

## Context

After C-54..C-58 run, ELAI has 7+ scorecard.json files scattered across
workflow/sessions/ + per-benchmark results.md docs. No single artifact
ties them together.

C-59 produces ONE master document with:
1. **The cross-config matrix**: rows = benchmarks, columns = configs (local
   7B / dual local 7B / hybrid Sonnet+Composer2+local / hybrid Sonnet-only).
   Cells = (solve-rate, cost, wall-clock-p99).
2. **The cross-competitor table**: ELAI's best config vs Devin / OpenHands /
   Aider / Claude Code / GPT-5.4 / Composer 2 / AgentForge / Codebase-Memory
   baselines, with version SHA + date for each external number.
3. **The cost-per-task analysis**: across all benchmarks, ELAI's $/task by
   config. Plot if dashboard supports; otherwise table.
4. **The honest gap narrative**: where ELAI loses (most likely: raw single-
   shot quality on SWE Atlas vs GPT-5.4 / Opus 4.7), where ELAI wins (most
   likely: cost-per-task vs cloud-only baselines), where the measurement is
   ambiguous (config-dependent).
5. **Improvement target list**: derived from the gap narrative, ranked by
   leverage. Feeds the next planning cycle.

This is THE deliverable that lets ELAI claim measured competitiveness.
Without C-59, the per-benchmark numbers exist but the cross-comparison
narrative does not.

Optional: TTS pilot — re-run config A (local 7B) on a 50-task SWE-bench
Verified subset with speculative decoding enabled (via TTS). Measure
end-to-end wall-clock delta. If positive, document; if not, archive.

## Reads

- workflow/sessions/{*}/swe_bench_run_*/scorecard.json (C-54)
- workflow/sessions/{*}/aider_polyglot_run_*/scorecard.json (C-55)
- workflow/sessions/{*}/cb_memory_run/scorecard.json (C-56)
- workflow/sessions/{*}/openhands_index/scorecard.json (C-57)
- workflow/sessions/{*}/swe_cycle/scorecard.json (C-57)
- workflow/sessions/{*}/swe_atlas/scorecard.json (C-57)
- workflow/sessions/{*}/head_to_head/scorecard.json (C-58)
- workflow/sessions/{*}/reranker_eval/scorecard.json (C-58)
- prior oracle report from 2026-05-17 (for external-baseline citations)

## Tasks

1. **Master report scaffold** — .elai_cc/docs/benchmark_master_report.md.
   Sections per Context above.

2. **Scorecard aggregator (crates/task_runner/src/master_report.rs)** —
   reads all scorecards, validates schema, populates the matrix.
   Aggregator is MACHINE-DRIVEN; the markdown report is REGENERABLE from
   scorecards + the external-baseline table.

3. **External-baseline table** — .elai_cc/benchmarks/external_baselines.toml
   pins published competitor numbers + version + date + citation URL.
   Sourced from prior oracle 2026-05-17 + fresh-fetch verification at C-59
   render time (numbers may have updated).

4. **Improvement-target derivation** — for each benchmark where ELAI loses,
   compute the delta and propose ONE concrete next-step (e.g. "SWE Atlas
   Q&A: ELAI hybrid scored X, GPT-5.4 scored Y; gap = Z%; candidate target:
   strengthen ContextCompiler with retrieval-aware grounding per
   C-56 + C-58 RRK findings").

5. **Honesty discipline** — the master report MUST cite limitations:
   - Where ELAI's config rules out direct comparison (e.g. ELAI 7B vs
     Devin Sonnet)
   - Where benchmark methodology disagrees (e.g. SWE Atlas rubric vs
     SWE-bench Verified pass/fail)
   - Where ELAI's number was a one-shot vs published multi-shot baselines
   - Where stochasticity matters (config C cloud-tier sampling)

6. **Public publication artifact** — .elai_cc/docs/benchmark_master_report.md
   is the canonical version. Optional sibling: a slide deck for talks
   (PLANNED, separate follow-up).

7. **(Optional) TTS pilot** — if RunPod budget permits after C-54..C-58
   complete:
   - Enable TTS in config A (local 7B)
   - Re-run 50-task SWE-bench Verified subset
   - Measure wall-clock delta vs C-54 config A baseline
   - Document in master report's "Speculative decoding pilot" section

8. **SSB action types** — master_report_aggregated, tts_pilot_run,
   tts_pilot_decision.

9. **Tests first (TDD)**:
   - 4Z.X.3: aggregator reads N scorecards and produces deterministic
     matrix (fixture)
   - 4Z.X.4: external-baseline table validates schema; required fields
     (version, date, URL) present per entry
   - 4Z.X.5: report regeneration is idempotent (same inputs → same output)
   - 4Z.X.6: improvement-target derivation produces non-empty list when
     ELAI loses on any axis
   - 4Z.X.7: honesty section is mandatory (report build fails if section
     absent or empty)

## WIRING_CONTRACT

produces_symbols:
  - MasterReportAggregator (crates/task_runner/src/master_report.rs)
  - .elai_cc/benchmarks/external_baselines.toml
  - .elai_cc/docs/benchmark_master_report.md (the capstone artifact)
  - SSB action_types: master_report_aggregated, tts_pilot_run,
    tts_pilot_decision
consumes_symbols:
  - All scorecards from C-54..C-58
  - prior oracle report citations (external baselines)
  - SSB event bus
reachable_from:
  - elai benchmark master-report
  - elai benchmark master-report --tts-pilot
stub_ban:
  - master report MUST be MACHINE-REGENERATED from scorecards; no
    human-handwritten narrative section that could drift from numbers
  - external baselines MUST cite version + date; numbers without those
    are excluded
  - honesty section MUST be present (build fails otherwise per 4Z.X.7)
  - improvement-target list MUST be non-empty if any ELAI loss exists

## DETERMINISTIC_CHECK

```bash
test -f .elai_cc/crates/task_runner/src/master_report.rs
test -f .elai_cc/benchmarks/external_baselines.toml
test -f .elai_cc/docs/benchmark_master_report.md
grep -c "honesty" .elai_cc/docs/benchmark_master_report.md  # must be ≥ 1
grep -c "improvement target" .elai_cc/docs/benchmark_master_report.md  # must be ≥ 1
rg "master_report_aggregated|tts_pilot_run|tts_pilot_decision" .elai_cc/crates/orchestrator/src
cargo test -p elai_task_runner -- master_report 2>&1 | grep "test result.*ok"
```

## ACCEPTANCE

- [ ] MasterReportAggregator implemented; reads all scorecards
      deterministically
- [ ] external_baselines.toml authored with version + date + URL per entry
- [ ] benchmark_master_report.md authored with all 5 required sections:
      cross-config matrix, cross-competitor table, cost-per-task, honest
      gap narrative, improvement-target list
- [ ] Honesty section explicitly cites limitations
- [ ] Improvement-target list non-empty (if any ELAI loss exists)
- [ ] All 3 SSB action types emit
- [ ] TDD tests pass; report regeneration idempotent
- [ ] (Optional) TTS pilot run + section in master report
- [ ] **Public-ready**: report can be linked from .elai_cc README,
      external blog, paper submission

## Files

- ADD: .elai_cc/crates/task_runner/src/master_report.rs
- ADD: .elai_cc/benchmarks/external_baselines.toml
- ADD: .elai_cc/crates/task_runner/tests/master_report_tdd.rs
- ADD: .elai_cc/docs/benchmark_master_report.md (regenerated)
- MODIFY: .elai_cc/crates/cli/src/main.rs (benchmark master-report subcommand
  + --tts-pilot flag)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb/schema.rs (3 new action types)

## Affected Verification Reports

- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\C-plan\C-54..C-58.report.md
  — each gets a closure annotation pointing at the master report.

## Next

C-59 closure CLOSES the C-35..C-59 batch. After this:
1. ELAI has a measured, publicly-defensible position vs every named
   competitor.
2. The master report is the source-of-truth for any external claim about
   ELAI's competitive standing.
3. The improvement-target list seeds the C-60..C-N+ batch for the next
   planning cycle.
4. Re-runs are cheap: scorecards + external_baselines.toml are inputs;
   the report is REGENERATED, not re-authored.

The C-35..C-59 batch represents ELAI's transition from "spec-grounded
architecture" to "measurement-grounded competitive product."
