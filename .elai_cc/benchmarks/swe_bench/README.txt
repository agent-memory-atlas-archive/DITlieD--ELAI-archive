C-21 Task 9 — SWE-bench fixture skeleton for staged-eval Stage 1/2/3 task
subsets. Each task is a single TOML file with the following fields:

  id              short identifier (matches filename without .toml).
  description     one-paragraph problem statement (no solution code).
  expected_output substring that must appear in the variant's resolution
                  for the task to count as solved by the benchmark runner.
  timeout_s       per-task wall-clock cap (seconds).
  stage           1 | 2 | 3 — which staged-eval tier this task is bound to.
  is_canary       true only for trigger_task.toml (objective hacking gate;
                  spec phase-4H.txt:612-617 + DGM Appendix H).

Provenance: problem statements are inspired by SWE-bench Lite
(MIT license, github.com/princeton-nlp/SWE-bench). NO solution patches are
included — this is a fixture skeleton, not a fork. The benchmark runner
trait (orchestrator::staged_eval::BenchmarkRunner) is responsible for
materialising each task into a sandbox and comparing variant output
against `expected_output`. Group F/G will swap the today's stub runner
for a real harness backed by these fixtures.

Canary role: trigger_task.toml drives the StageReport.trigger_task_passed
flag. Stage 3 promotion is GATED on `trigger_task_passed=true`, so a
variant that hacks its way through the regular tasks but fails the
canary cannot promote. The canary is intentionally narrow (one
straightforward fix) so a properly-aligned variant should solve it
quickly; a misaligned variant tends to over-engineer or evade.
