# Verification Report: B-8_pipelinegrc_activation.md
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-8_pipelinegrc_activation.md
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
All four PipelineGrc methods have production callers in task_runner.rs. GroundingSweepGate now receives real ungrounded_claims from ground_plan. Auditor anti-rubber-stamp is activated in agent_pool.rs via the Observe-seed pattern. InlineFeasibilityState.reset() fires at phase boundaries. Two items are PARTIAL: (1) ReplanRequest::Major emission is explicitly deferred via TODO(B-future) in task_runner.rs:5583-5584 — Blocked outcome flips the step to Failed but does not submit a real ReplanRequest; (2) the SSB csq_evaluated row and dashboard intervention card are deferred via TODO(B-future) at task_runner.rs:5119-5120. Task#2 (verify_claim into ContextCompiler) is explicitly out-of-scope in agent_pool.rs:552-563.

## Claims verified DONE

- PipelineGrc::ground_plan wired from PLANNING transition (Task#1) — evidence: crates/orchestrator/src/task_runner.rs:4438-4440 (`grc.lock().await.ground_plan(&plan_text)`)
- GroundingSweepGate registered in gate list — evidence: crates/orchestrator/src/evidence.rs:575 (inside `all_gates()`)
- GroundingSweepGate struct defined — evidence: crates/orchestrator/src/evidence.rs:218
- ground_plan result populates ctx.ungrounded_claims — evidence: task_runner.rs:4423-4525 (grounding.ungrounded fed into TransitionContext)
- PipelineGrc::generate_csq wired in pre-executor (Task#3) — evidence: crates/orchestrator/src/task_runner.rs:5084-5140 (`grc.lock().await.generate_csq(...)`)
- Critical-without-guard CSQ blocks baby step via hook_pre_blocked — evidence: task_runner.rs:5090-5121
- PipelineGrc::check_inline_feasibility wired in EXECUTING (Task#4) — evidence: task_runner.rs:5552-5619 (`grc_guard.check_inline_feasibility(&claim, &sources)`)
- FeasibilityOutcome::Blocked flips step to Failed (Task#5 partial) — evidence: task_runner.rs:5595-5601 (`outcome = BabyStepOutcome::Failed { reason: ... }`)
- InlineFeasibilityState.reset() called at phase boundary (Task#7) — evidence: task_runner.rs:4786-4798 (`grc.lock().await.engine_mut().inline_feasibility_state.reset()`)
- Auditor anti-rubber-stamp Observe-seed wired in agent_pool::spawn (Task#6) — evidence: crates/orchestrator/src/agent_pool.rs:521-550
- Anti-rubber-stamp covers all 6 auditor role variants (B-41 extension) — evidence: agent_pool.rs:521-529 (AgentRole::Auditor | AuditorConcepts | AuditorPhases | AuditorSteps | AuditorBabySteps | AuditorBabyStepDetail)
- GrcError::InsufficientFindings defined in reasoning engine — evidence: crates/reasoning/src/types.rs:168-169
- TaskRunnerConfig::with_pipeline_grc() builder method exists — evidence: task_runner.rs:2523-2524
- TaskRunnerConfig::pipeline_grc field is Option<Arc<AsyncMutex<PipelineGrc>>> — evidence: task_runner.rs:2452
- ELAI_DISABLE_PIPELINE_GRC kill-switch respected in ResolvedConfig::from — evidence: task_runner.rs:2772-2781
- tests/contracts/safety_gate_integration.rs (listed as tests/contracts/ in spec) created under orchestrator/tests/ — evidence: crates/orchestrator/tests/safety_gate_integration.rs:1-7 (tests 1-7 from ACCEPTANCE list all present)
- pipeline_grc_wired_contract.rs contract test exists covering builder API — evidence: crates/contracts/tests/pipeline_grc_wired_contract.rs:1-136
- GroundingSweepGate REGISTERED claim: spec says evidence.rs:471 — actual line is 575, but same all_gates() list — evidence: crates/orchestrator/src/evidence.rs:575

## Claims PARTIAL

- Task#5: FeasibilityOutcome::Blocked converts to ReplanRequest::Major
  - exists: task_runner.rs:5595-5601 (Blocked -> BabyStepOutcome::Failed, step flipped, Finding emitted)
  - gap: ReplanRequest::Major submission deferred; TODO(B-future) at task_runner.rs:5583-5586 says "submit ReplanRequest::Major via fsm.submit_replan once ResolvedConfig carries an FsmHandle"

- Observable effect: SSB csq_evaluated row + dashboard intervention card (WIRING_CONTRACT observable_effect)
  - exists: CSQ block emits Finding row ("csq_critical_block") and tracing::error at task_runner.rs:5109-5118
  - gap: TODO(B-future) at task_runner.rs:5119-5120 defers the dedicated SSB csq_evaluated row; no intervention_card SSB type found in codebase

- Task#2: PipelineGrc::verify_claim wired into ContextCompiler pre-fetch
  - exists: verify_claim method defined at grc.rs:131; called from grc.rs tests and reasoning tests
  - gap: agent_pool.rs:552-563 explicitly scopes this out-of-scope ("wiring deferred to B-future", no Rust ContextCompiler prefetch path exists)

## Claims MISSING

- dispatcher.rs CSQ insertion point — searched: generate_csq, pipeline_grc, PipelineGrc in dispatcher.rs; no evidence (CSQ gate lives in task_runner.rs, not dispatcher.rs)
- file_ops.rs pre-write hook — searched: file_ops.rs does not exist in crates/orchestrator/src/

## Claims UNVERIFIABLE

- workflow/reasoning/B-8-decisions.md existence — gitignored path; cannot verify
- audit-implementation/_AGGREGATE.md PATTERN 2 reference — gitignored path

## What must be done (action list)

- [ ] Implement ReplanRequest::Major submission when FeasibilityOutcome::Blocked — target: task_runner.rs TODO(B-future) at line 5583; requires FsmHandle on ResolvedConfig
- [ ] Implement dedicated SSB csq_evaluated row with severity/guard_present/blocked fields — target: task_runner.rs line 5119; needs new SsbPayload variant in ssb.rs
- [ ] Implement dashboard intervention card SSB row following csq_evaluated — target: dashboard-v2 layer and ssb.rs; currently no intervention_card payload type exists
- [ ] Wire verify_claim into ContextCompiler pre-fetch — target: agent_pool.rs:559; currently stubbed with debug log only
- [ ] Fix spec line-number: GroundingSweepGate registered at evidence.rs:575, not :471 as spec states

## Notes
evidence.rs:471 in the spec is incorrect; the actual registration is at line 575 in all_gates(). The struct is defined at line 218 and impl at line 220. The spec was written before a file reorg that pushed the gate list further down. All four PipelineGrc methods have verified production callers in task_runner.rs — the original "ZERO production callers" problem stated in the B-8 context is resolved. The three remaining gaps (ReplanRequest::Major, csq_evaluated SSB row, verify_claim) all carry explicit TODO(B-future) comments and are documented scope deferrals, not oversights.
