# B-0 minimal discriminator benchmark — preregistration (2026-07-12)

Status at authorship: LOCKED BEFORE any baseline-model call. The deterministic ProbeUtility-only
corpus smoke has run; it validated task truth/probe consistency (20/20) but is not a comparative
result and cannot decide the gate.

## Question and kill gate

Does the v0 prediction-table scheduler (`pairs_split * observation_reliability / probe_cost`) beat
a strong model directly choosing its next test under the same task/probe budget on real local
debug/design-fork diagnosis success per normalized cost?

The DISCRIMINATION plane is a WIN only when all three are true against DIRECT:

1. paired mean `(success / normalized_cost)` delta is positive;
2. paired success-rate delta is non-negative; and
3. the seeded paired-bootstrap 95% interval's lower bound is strictly positive.

Anything else is REFUTED for v0. On REFUTED: U4, `--discriminate`, B-2, L2, and L3 do not ship.
No reinterpretation through secondary metrics is permitted.

## Corpus

- K=20 paired tasks: 10 live-code/debug seams + 10 live design/ownership forks.
- Frozen corpus: `.claude/tools/fixtures/frontier_discriminator_tasks.json` (hash written to run manifest).
- Each task names live repository source refs, 2-3 rival hypotheses, categorical predictions, a
  closed read-only probe catalog, a weighted probe budget, and a sealed correct diagnosis.
- Truth labels are never serialized into model prompts.
- Before model spend, the deterministic treatment ran the whole corpus and reached the sealed truth
  on 20/20 with no probe-execution failures. Receipt:
  `workflow/benchmarks/frontier-discriminator-b0-probe-smoke-2026-07-12/`.

## Arms

All arms see the same task statement, major observations, live hypotheses, candidate-by-probe
predictions, probe descriptions, reliability class, cost, budget, and prior observations.

1. `direct`: `glm-5-2` chooses the single next test it judges most useful per unit cost.
2. `best_of_n`: the same model and output cap privately proposes three next-test choices, critiques
   them, then emits one choice.
3. `probe_utility`: deterministic ranking by split pairs x reliability / cost; no model chooses.

Arm order is seeded-randomized per task. Model arms use identical max output (700 tokens), timeout,
retry policy, tool-unavailable handling, and cache contract. Invalid/unavailable probe choices are
fail-closed `model_choice_invalid`; there is no heuristic fallback.

## Cost and metrics

Primary cost is NORMALIZED so a deterministic selector cannot win merely because it skipped an LLM
call: every selection step in every arm is charged the same `DECISION_COST=1`, plus actual probe
cost. Primary score per task is `1 / normalized_cost` on correct final diagnosis, else zero.

Observed model tokens and an observed-cost view are reported separately as economics, never used to
decide the kill gate.

Secondary (non-rescuing): raw success rate, weighted probe cost, probes used, wrong turns (selected
probe had zero current split utility), recovery after a wrong first probe, invalid model replies,
hypothesis exhaustion, and model tokens.

## Integrity and execution boundary

- Probes are implemented by a closed vocabulary: root-confined `path_exists`, `text_contains`, and
  `regex_contains`. Arbitrary shell/subprocess execution is absent.
- Every raw observation records source path, source SHA-256, observed categorical value, and is
  BLAKE3-receipted by the discriminator core.
- Model responses and prompts are cache-keyed by model + token cap + full prompt hash.
- Missing/unreadable evidence cannot become `absent`; it becomes `unreadable` and normally refutes
  all candidates, exposing corpus drift.
- Benchmark code, task corpus, and core hashes land in `manifest.json`.

## Prior-art boundary

This is deliberately categorical v0, not Bayesian EIG. Uncertainty of Thoughts selects questions
using information-gain-motivated rewards over a small categorical outcome space, supporting the
shape but not this exact formula (arXiv:2402.03271, verified 2026-07-12). MAP-Elites supports
archive-iterative quality-diversity rather than single-call prompt variation (arXiv:1504.04909,
verified 2026-07-12). Neither source supplies this benchmark's outcome; the local paired run does.

## Command and immutable outputs

```powershell
python .claude/tools/frontier_discriminator_bench.py `
  --out workflow/benchmarks/frontier-discriminator-b0-k20-2026-07-12 `
  --k 20 --seed 20260712 --model glm-5-2
```

Expected outputs: `manifest.json`, append-safe cached model calls, `results.jsonl`, `summary.json`,
and `report.md`. The human report must quote the mechanical gate verbatim.
