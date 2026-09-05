patch-1.14 — elai task post-PLANNING hang, evidence + fix scope

symptom

three runs on NodeBB__04998908 (session s-25120, vast 3090, NF4 3B Qwen, SDPA) at wall budgets 360s, 900s, 1800s, all produced identical SSB sequence then hung to wall timeout with exit code 2:

  seq=1 task_submitted
  seq=2 reasoning_trace
  seq=3 checkpoint
  seq=4 grounding_cache_event
  seq=5 concept_decomposition_node
  seq=6 finding
  (nothing else until wall budget exhausted)

final run had 0 new sidecar generations across 30 minutes; subprocess alive the whole time; exit code 2; cancel.cancel() at task_cmd.rs:499 succeeded on signal so no orchestrator-side panic.

investigation

start_task_runner at task_runner.rs:3903 is an outer loop. it consumes TaskSubmitted rows via tokio broadcast and never exits except on cancel or broadcast-closed. when drive_task returns for ONE task (success or early-block), the runner returns to rx.recv() listening for the next TaskSubmitted row.

drive_task has multiple early-return paths that DO NOT emit PhaseComplete:
  - task_runner.rs:5482 emit_finding(error, grounding_sweep_block, ...) then return Ok(())
    fires when GroundingSweepGate blocks the Planning > TestingFirst transition; the orchestrator considers the run handled and gracefully exits drive_task without a terminal PhaseComplete row.
  - task_runner.rs:4056-4065 emit_finding(error, task_failed, ...) when drive_task itself returns Err
    fires on any Err propagation out of drive_task; handle_task_entry logs the failure and emits this finding.
  - task_runner.rs:5191-5193 return Err(TaskRunnerError::EmptyDecomposition)
    triggers the 4058 finding path above.

task_cmd.rs:443-496 has biased select with only two break conditions:
  - res = (&mut handle) > runner future returned. cannot fire while runner is in outer loop.
  - recv > collected.phase_complete > only PhaseComplete sets this.
  no fallback for "drive_task returned but no PhaseComplete". result: subprocess waits to wall budget. observed 30-min hang.

separately, task_cmd.rs builds production-shape wiring via build_task_runner_wiring then calls apply_task_runner_wiring, but SKIPS the ~220 line agent-pool installation block at run_subcommand.rs:244-465. without it, apply_task_runner_wiring falls through to SyntheticBabyStepExecutor at task_runner.rs:3220. this is a LATENT bug — it would only surface after the FSM advances past PLANNING and tries to dispatch a real baby step. SyntheticBabyStepExecutor returns synthetic_passed instantly, so the symptom of bug 2 would be "FSM races through fake completion with no real LLM dispatch", not a hang.

root cause of the OBSERVED 30-min hang

bug 1 is the proximate cause. GroundingSweepGate emits a Finding(error, grounding_sweep_block) on the NodeBB task and returns Ok(()) from drive_task. the runner returns to its outer rx.recv() loop. task_cmd.rs has no terminal-error break condition, so it waits for PhaseComplete that never comes.

bug 2 is real but separate. it would surface on a task that DOES pass the grounding sweep.

both must be addressed for elai task to be usable for unattended SWE-Bench Pro evaluation.

fix scope (patch-1.14, cli crate only)

(a) extract production agent pool wiring from run_subcommand.rs:244-465 into install_production_agent_pool in runtime_builder.rs. call from both run_subcommand.rs (byte-equivalent refactor) and task_cmd.rs (new insertion between hook_dispatcher setup and ssb_writer clone).

(b) extend task_cmd.rs CollectedRows with terminal_error: Option<String>. in absorb(), detect Finding(severity=error, category in TERMINAL_ERROR_FINDING_CATEGORIES) and set the field. in the collect loop, treat terminal_error.is_some() the same as phase_complete (tail-flush 500ms then break). update emit_summary outcome to map terminal_error to outcome="blocked".

TERMINAL_ERROR_FINDING_CATEGORIES (allowlist, conservative):
  task_failed              task_runner.rs:4058, drive_task returned Err
  grounding_sweep_block    task_runner.rs:5482, planning > testing-first gate blocked

other error findings (web_scaffold_failed, web_research_failed) are TRANSIENT — they have retry/fix paths that re-emit. only categories proven terminal at the task_runner level land in the allowlist.

acceptance criteria (auditor)

  cargo check -p elai-cli GREEN
  cargo check --workspace GREEN
  cargo tree -p elai-cli --depth 1 shows zero new dependencies
  git diff --stat -- crates/cli/ shows exactly 3 files modified
  semantic diff inspection of run_subcommand.rs shows byte-equivalent behavior (env-var read order preserved, fallback warn message preserved, locate_default_agent_binary Err arm still leaves wiring.baby_step_executor unset)
  zero unwrap/expect/let _/TODO/log::/blake2/async_trait hits in the new helper
  no commit, no push (memory:feedback_no_auto_commit)

out of scope (deferred)

  TaskTerminated SSB action_type (option C from the live deliberation) — proper architectural fix for terminal signal. requires SCHEMA bump and orchestrator changes, future B-plan.
  pinning the precise hang point for the case where grounding_sweep_block is NOT the finding (e.g. EmptyDecomposition path) — covered by the same terminal_error allowlist via task_failed category.
  RUST_LOG=elai_orchestrator=trace post-fix verification — done in next session per s-25120 P0 list.
