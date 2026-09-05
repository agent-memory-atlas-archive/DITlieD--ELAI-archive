# Conjecture v1.1 selector A/B — preregistration (2026-07-12)

Locked before any v1.1 benchmark generation call.

## Question

Does the `NOVEL-CREDIBLE` lexicographic profile repair the prior engine pick's blind novelty loss
(13 engine votes to 27 baseline votes), without changing the candidate set underneath it?

This run isolates selection. For each of the same K=10 ELAI triggers used by the 2026-07-11
novelty A/B, one `kimi-k2-7` direct self-coordinating call generates exactly ten v1.1 cards. Both
selectors receive that identical set:

- ENGINE: deterministic stage-2 triage + receipt-derived groundedness + two-ordering archive NLI
  equivalence verdicts + `NOVEL-CREDIBLE` profile.
- BASELINE: `kimi-k2-7` picks the most impactful/feasible candidate from the same compact fields.

The shared set removes the prior set-diversity/presentation confound. This benchmark tests the
profile mechanism only; B-3 separately tests archive-conditioned multi-cycle generation.

## Instruments

- Generator/naive selector: `kimi-k2-7` on the loopback Devin bridge.
- Archive-equivalence judge: `glm-5-2`, two seeded orderings. Agreement preserves SAME/DISTINCT;
  disagreement mechanically degrades to UNCERTAIN. Unknown archive ids fail closed.
- Blind pick panel: `glm-5-2` + `kimi-k2-7`, two A/B label orders each = 40 votes at K=10.
- Pairwise ideas use the same four-field projection: mechanism family, target area, expected effect,
  falsifier. The panel never sees selector identity, receipts, or candidate ids.

Raw archive prose never enters generation. The generator receives only the trigger and a declared,
verified `elai.map` anchor pane, logged as `INJECT: ... code-slice:elai-map-anchor-pane-v1`.

## Gate

- REFUTED: engine novelty share is <=13/40 and baseline share >=27/40 (same prior margin or worse).
- SUPPORTED: engine wins strictly more novelty votes than baseline.
- INCONCLUSIVE: anything between those conditions, including a smaller loss or tie.

Value votes are reported but cannot rescue/refute the novelty-profile mechanism. Any trigger whose
artifact, triage, NLI verdict, profile, naive pick, or pair vote is malformed fails closed and is
listed; a run with failures is not a clean headline result.

## Command and artifacts

```powershell
python .claude/tools/conjecture_novelty_v11_ab.py `
  --archive-index workflow/reasoning/frontier-archive-index.json `
  --out workflow/benchmarks/conjecture-v11-selector-ab-k10-2026-07-12 --k 10
```

Outputs: generated v1.1 artifacts, hash-keyed model-call cache, incremental `results.json`, manifest,
summary, and report. All nulls/losses ship unchanged.
