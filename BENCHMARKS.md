# Benchmarks and recorded experiments

**Yes: benchmark code and selected results were committed and are in this public
archive.** Results were not saved uniformly. The tree mixes harnesses, synthetic
fixtures, seed configuration, historical measurements and research reports.
It is not a validated benchmark release or a leaderboard submission.

The original table was checked against the public Git tree at `55011059`.
A later [documentation recovery](docs/RECOVERED-DOCUMENTS.md) adds the ignored
reports described below. No historical benchmark, model inference or
ELAI application test was rerun for the archive. Privacy edits and omitted inputs
can also invalidate old hashes and reproduction instructions.

## What is actually here

| Area | Code, data or result to inspect | What the retained evidence covers |
| --- | --- | --- |
| Token efficiency / Phase 3J | [Model-capability harness](.elai_cc/benches/model_capability/run_benchmark.py), [tasks](.elai_cc/benches/model_capability/fixtures/tasks.json), [model baseline](.elai_cc/metrics/model_baseline.toml), [phase metrics](.elai_cc/metrics/phase_3J.toml), [historical review](docs/thefinalglance/verification/reports/steps/phase-3J.report.md) | A recorded local-model baseline contains context-size, timing and failure observations. It also records broken paths and pending measurements. The review identifies fixture/validation problems; the numbers are not portable model limits or current end-to-end scores. |
| GRC / reasoning microbenchmarks | [GRC bench](.elai_cc/crates/reasoning/benches/grc_bench.rs), [verification bench](.elai_cc/crates/reasoning/benches/verification_bench.rs), [configuration bench](.elai_cc/crates/reasoning/benches/config_bench.rs), [Phase 2A metrics](.elai_cc/metrics/phase_2A.toml), [2B](.elai_cc/metrics/phase_2B.toml), [2C](.elai_cc/metrics/phase_2C.toml) | In-memory component timings using synthetic chains and mock readers/runners. These measure local operations, not the quality of model reasoning or real coding tasks. |
| Calibration | [Live harvest harness](.elai_cc/python/elai_inference/bench/calibration_harvest_e19.py), [synthetic seed](.elai_cc/calibration/seed/README.md), [reasoning state](.elai_cc/eval/reasoning/calibration.toml), [routing state](.elai_cc/eval/routing/calibration.toml), [data notes](.elai_cc/benchmarks/calibration_bootstrap/DATA-NOTES.md) | Harness and seed data exist. Reasoning state records zero traces; routing state is pending with zero holdout tasks. Those TOMLs are not completed live calibration results. Some external inputs were omitted. |
| Conformal refusal logic | [Fixture definition](.elai_cc/benchmarks/conformal/README.md), [saved JSON](.elai_cc/benchmarks/conformal/conformal-1784362966006.json) | A deterministic 36-row fixture run. Its aggregate verdict is pass, while the tool-use/far-OOD domain is outside its reported band. The gate aggregates by distribution bucket; pass does not mean every domain passed. |
| Canary regression | [Recorded run and caveats](.elai_cc/benchmarks/canary/reports/canary-bench-report-2026-06-06.md), [JSON](.elai_cc/benchmarks/canary/reports/canary-bench-report-2026-06-06.json) | A CPU fixture run reports 20 harmful cases blocked and 80 other cases passed. The same report says the CLI emitted no state-board evidence row and identifies a missing task-runner dispatch path. The component result is narrower than the original acceptance goal. |
| PRMBench mini | [README](.elai_cc/benchmarks/prmbench/README.md), [mini fixture](.elai_cc/benchmarks/prmbench/prmbench_mini_fixture.jsonl), [result](.elai_cc/benchmarks/prmbench/reports/prmbench-2026-07-18.json) | A hand-written 20-row fixture, not the full upstream benchmark. The saved report says accuracy 0.5, precision 0.5 and recall 1.0 while its own gate says pass. Read the metrics and thresholds, not just the verdict. |
| Sycophancy / tamper | [Sycophancy result](.elai_cc/benchmarks/sycophancy/reports/sycophancy-2026-07-18.json), [tamper result](.elai_cc/benchmarks/tamper/reports/tamper-bench-report-2026-07-18.json), [notices](THIRD_PARTY_NOTICES.md) | Historical adversarial fixture reports, not live-task evaluations. The old sycophancy corpus included 20 adapted fixtures that were removed during publication review; 80 synthetic fixtures remain. The old report is not a result for the reduced public corpus. |
| Improvement-conjecture / FDL research | [Grounding pilot](workflow/reasoning/conjecture-bench-run-2026-07-11.md), [novelty A/B](workflow/reasoning/conjecture-novelty-ab-2026-07-11.md), [later synergy experiment](workflow/reasoning/conjecture-run-autoresearch-synergy-2026-07-17.md) | Reports from external bridge/agent experiments about ELAI. The small grounding pilot recorded more real code anchors but no supported downstream reduction in auditor blocks. Negative/null novelty findings are retained too. Raw harness/run directories are not included; this is not an ELAI runtime benchmark. |
| Scale, journal and runtime harnesses | [Graph-scale bench](.elai_cc/crates/knowledge/benches/symbol_graph_scale.rs), [memory bench](.elai_cc/crates/knowledge/benches/scale_memory.rs), [checkpoint bench](.elai_cc/crates/orchestrator/benches/checkpoint_format.rs), [SSB throughput](.elai_cc/crates/orchestrator/benches/ssb_throughput.rs), [runtime probe](.elai_cc/benches/runtime_compat/probe_local.py), [local-lane runner](.elai_cc/crates/orchestrator/src/benchmarks/local_lane_e2e_runner.rs) | Benchmark source is present. Retained scale metrics include synthetic/limited cases; production-scale runs were deferred. The [registry](.elai_cc/benchmarks/registry.json) still labels local-lane-e2e PLANNED despite runner source existing. No retained local-lane end-to-end result was identified in this review. |

## Reports recovered from ignored documentation

The later recovery adds 12 saved reports under [workflow/benchmarks](workflow/benchmarks/),
including selector A/B, frontier-discriminator, multicycle and research-creativity
experiments. Negative and invalid verdicts remain as recorded; for example,
[the second creativity report](workflow/benchmarks/research-creativity-b0v2-2026-07-13/report.md)
says INVALID and [the third](workflow/benchmarks/research-creativity-b0v3-2026-07-14/report.md)
says REFUTED. These are external development experiments, not end-to-end agent scores.

[Retrieval experiment plans and results](workflow/experiments/), including the
[RLM/ELAI comparison](workflow/experiments/RLM-VS-ELAI-RESULTS-2026-06-26.md) and
[CGG findings log](workflow/experiments/cgg_bench/CGG-BENCH-FINDINGS.md), are also
preserved. The [earlier CGG result file](workflow/experiments/cgg_bench/CGG-BENCH-RESULTS.md)
still says DATA PENDING; it was not rewritten to agree with the later findings.
[Historical audits](workflow/audits/) include a saved checkpoint-format measurement.

These documents were absent from the earlier public tree. Their raw run directories,
provider sessions and external datasets remain omitted. No measurement was rerun,
and the reports alone do not establish reproducibility or a validated comparison.

## What was not committed or included

The private working tree also had ignored gate-tax JSON/RRA runs, harness-trust
reports/logs, token logs and raw conjecture runs. These were not brought into the
public archive. Their presence locally does not establish that they are complete,
comparable, safe to redistribute or independently reproducible.

Other gaps are deliberate: HaluEval/MMLU inputs and the 20 adapted sycophancy
fixtures were omitted during provenance review. See [data notes](.elai_cc/benchmarks/calibration_bootstrap/DATA-NOTES.md)
and [third-party notices](THIRD_PARTY_NOTICES.md). A historical report may describe
a larger corpus than the one now published.

The [benchmark registry](.elai_cc/benchmarks/registry.json) is a measurement
contract, not a store of measured numbers. A registered benchmark, CLI command,
plan marked DONE or file called calibration is not a result by itself.

## How to read a result

Follow the chain **harness → exact fixture or dataset → saved result → stated
limitations**. Check whether it used live inference, a mock or a deterministic
fixture; whether its gate matches the claim you care about; and whether all inputs
remain available. A passing threshold is not necessarily strong performance, and
an isolated component result is not evidence that the whole agent was useful.

These reports are worth preserving as experiments, including failed and inconclusive
ones. Reusing a subsystem still requires your own measurements and validation.

The later [Conductor inclusion](CONDUCTOR.md) has a separate
[fixture-test verification record](conductor/TESTING.md). Those checks concern the
external development tooling; they are not a rerun of ELAI's application benchmarks.

[Introduction](README.md) · [Postmortem](POSTMORTEM.md) · [Salvage map](SALVAGE-MAP.md)
