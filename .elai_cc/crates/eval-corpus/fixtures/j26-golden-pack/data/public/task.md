# J-26 golden pack

Write a function `add(a: i32, b: i32) -> i32` that returns the sum of two integers.

This is a deterministic golden fixture for the J-26 evaluator ABI contract test.
The evaluator (`evaluate.sh` / `evaluate.py`) writes a known scalar
`{"fitness": 0.87}` into `--gen-dir/results.json` so the ABI contract can
assert that `run_evaluator` reads the golden scalar and that
`results_to_execution_fitness.fitness` matches.
