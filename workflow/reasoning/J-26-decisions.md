# J-26 decisions — sia evaluator ABI + task-pack format

## SCOPE (Tier-1, in-process, fixture-driven)

SHIP NOW: the ABI struct + subprocess runner + results.json reader with
missing=FAIL semantics, wired where H-43's fitness evaluation dispatches as a
SIBLING channel (config-gated). DEFER Tier-2: any LIVE model-in-the-loop
generation run (RunPod/remote-gated per W-10; F-11's checkbox discipline).

## H-43 RESPECT BOUNDARY

The ABI is a SIBLING channel, NOT a scorer replacement. The in-process
`execution_grounded_fitness_for_candidate` body is UNTOUCHED. The ABI branch
selects between channels based on env vars; it does not replace. The
`emit_fitness_row` path is REUSED, not re-authored.

## R8 RE-LOCK FORK

S7 (gate-manifest R8 registration of evaluator_abi.rs + golden fixture paths)
is OUT-OF-BAND. The executor does not self-authorize gate edits. The
orchestrator must pre-authorize the re-lock (FORK-2) before S7 can execute.

## GAP-1 (NewTaskInput converter)

Not addressed in S2+S3. S4 (CLI ingest verb) will need the TaskPack-to-
NewTaskInput converter. RECON-1 at plan time: read
`eval-corpus/src/task_schema.rs` for the NewTaskInput.task schema and map
every required field to a TaskPack source. If any required field has no pack
source, FORK-1 fires.

## SIBLING CHANNEL DESIGN

Env vars: `ELAI_EVALUATOR_ABI_ENABLE=1` AND `ELAI_EVALUATOR_ABI_PACK=<path>`.
When both are set and the pack is valid, the ABI path runs. On ABI Err when
env is ON: hard FAIL via `AdmitLoopError::EvaluatorAbi` (R2, never `.ok()`-
away into skip). When env OFF: behavior identical to main (the in-process
scorer runs as before).

Per-candidate gen-dir: `std::env::temp_dir()/elai-j26-eval-{seed}-{label}`.
The candidate's `eval_log` is written as `candidate_output.txt` in the
gen-dir for the evaluator to read. The evaluator writes `results.json` into
the gen-dir; the ABI reads it back.

## RESULTS-TO-EXECUTION-FITNESS MAPPING

The primary scalar (prefer `fitness`, then `pass_rate`, then `score`) becomes
`ExecutionFitness.fitness`. When only one scalar is available, `pass_rate`
and `dual_agreement` are set to it (honest: no decomposition). 
`tests_total`/`tests_passed` come from results when available, else 0.
`seed` and `test_set_hash` are caller-supplied (the loop owns them).

## MISSING = FAIL (R1/R2)

`AbiError::ResultsJsonMissing` is a dedicated error variant (not a generic
IO error). The evaluator exiting 0 but producing no `results.json` is a
hard FAIL, never a default-zero score, never a SKIP. The unit test
`missing_results_json_is_hard_fail` asserts this. The contract test (S6,
not in this changeset) will assert the cross-seam negative.

## CLOSE-OUT STATUS (S2+S3)

- S2 (evaluator_abi.rs): DONE, compiles clean.
- S3 (darwin_admit_loop.rs sibling branch): DONE, compiles clean.
- S4 (CLI ingest verb): NOT STARTED (next slice).
- S5 (golden fixture): NOT STARTED.
- S6 (contract test): NOT STARTED.
- S7 (gate-manifest re-lock): OUT-OF-BAND (FORK-2).
- S8 (close-out): pending S4-S7.
