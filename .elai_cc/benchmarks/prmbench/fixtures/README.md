PRMBench canonical-schema fixtures (D-19 smoke corpus)

Purpose

This directory holds a small subset of PRMBench-format fixtures for the two
adversarial-gaming categories ELAI cares about most: Deception Resistance
(DR) and Non-Circular-Logic (NCL, surfaced here as `circular_logic` to
match the runner's category label). The fixtures use the canonical
PRMBench schema documented in
F:\.ELAI_workflow\workflow\papers\prmbench-2501.03124-interface.md so
downstream graders can be pointed at this directory and at the full
upstream dataset interchangeably (with one transformation step).

This is an ELAI smoke fixture, NOT the full PRMBench eval set. The full
upstream dataset (~6.2k rows, 9 sub-categories) is too large to commit
verbatim and is licensed CC-BY-4.0 per the PRMBench paper's data release.

Schema (per row)

Each file is one JSON object with the canonical PRMBench fields:

  {
    "idx":               "<short id, e.g. dr_001 / ncl_003>",
    "classification":    "deception_resistance" | "circular_logic",
    "original_question": "<problem statement, unmodified>",
    "modified_question": "<problem statement, possibly with adversarial framing>",
    "original_process":  ["Step 1: ...", "Step 2: ...", ...],
    "modified_process":  ["Step 1: ...", "Step 2: ..."],
    "modified_steps":    [<1-indexed step indices that were tampered>],
    "error_steps":       [<1-indexed step indices the PRM must flag>],
    "reason":            "<one-sentence rationale for the adversarial edit>"
  }

`modified_steps` and `error_steps` overlap when the adversarial edit IS
the error; they can diverge when the adversary edits a benign step to
make the downstream error harder to localise.

Layout

  fixtures/
    deception_resistance/
      dr_001.json ... dr_005.json   (5 rows, plus follow-ups as added)
    circular_logic/
      ncl_001.json ... ncl_005.json (5 rows, plus follow-ups as added)

Relationship to the runner JSONL fixture

`.elai_cc/benchmarks/prmbench/prmbench_mini_fixture.jsonl` is the
runtime fixture the `PRMBenchRunner` reads today. It uses a flatter
schema (`task_id`, `category`, `problem_statement`, `step_history`,
`candidate_step`, `ground_truth_label`) chosen for streaming
single-row scoring through the PPM. The canonical fixtures in THIS
directory are the documentation-grade twin: they preserve the
original / modified pair so reviewers can audit the adversarial
construction without re-deriving it from the runtime JSONL.

A one-shot transformer (canonical -> runtime JSONL) is intentionally
omitted from D-19; pulling the full upstream dataset into the runtime
JSONL is the natural follow-up plan.

License + provenance

The bundled fixtures are hand-written by the ELAI team and are NOT a
verbatim copy of the upstream PRMBench dataset. They are released under
the same terms as the rest of `.elai_cc/` (project default). If you
want to reproduce paper-scale PRMBench numbers, ingest the upstream
CC-BY-4.0 dataset directly; the smoke corpus here is sized for CI and
unit-test determinism, not paper reproduction.
