PRMBench fixtures (D-19)

Bundled mini-fixture (`prmbench_mini_fixture.jsonl`) is a 20-row stub used by
the `elai bench prmbench` CLI smoke + the orchestrator unit tests. It
contains 10 rows for `deception_resistance` and 10 rows for `circular_logic`
— enough to exercise the runner end-to-end deterministically.

Each line is one JSON object with the schema:

  {
    "task_id":            "<category>_<index>",
    "category":           "deception_resistance" | "circular_logic",
    "problem_statement":  "<math / proof prompt>",
    "step_history":       ["Step 1: ...", "Step 2: ...", ...],
    "candidate_step":     "Step N: ...",
    "ground_truth_label": "correct" | "incorrect"
  }

The full PRMBench dataset (arxiv:2501.03124, T1) is licensed for research
use; pulling the full ~6.2k-row dataset + the other 7 PRMBench categories
(Multi-Solution / Confidence Invariance / etc.) is a follow-up to D-19.
For CI / dashboard purposes the mini-fixture is sufficient because the
runner code path is identical regardless of fixture size.

Hashing

The runner computes blake3 over the raw JSONL bytes and stamps the hex
into the SSB row's `fixture_blake3` field so dashboards can disambiguate
runs against different fixture revisions. Do NOT edit individual lines
in-place — re-emit the entire file via fixture-generator scripts so the
blake3 changes wholesale.

License

The bundled mini-fixture contains hand-written stub problems composed
for this repo. The upstream PRMBench dataset is published under CC-BY-4.0
per the paper's data release.
