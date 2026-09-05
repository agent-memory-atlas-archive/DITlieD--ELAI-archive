# E-8: GEPA-driven self-evolving prompts with cloud-reflection / local-target split
Objective: ship a GEPA-class reflective prompt optimizer (arXiv:2507.19457 ICLR 2026 Oral) as the primary optimization driver for ELAI's PromptEngineer agent + ReasoningTemplate archive, using cloud Sonnet 4.6 as the reflection LM and local Qwen2.5-Coder-7B as the target; freeze contract blocks via LPO `<edit>` tag pattern; gate promotion via the existing BAI Thompson pipeline with a held-out test split; wire auditor PASS/BLOCK reasons as text feedback verbatim per the GEPA paper's text-feedback enrichment principle.
Estimated effort: 2 sessions, ~8-10 h
Pre-requisites: D-1 (ReasoningTemplate archive, PLANNED — or E-7 task 1 if MAP-Elites supersedes), D-5 (BAI Thompson promotion, PLANNED — or E-7 task 3 if Discounted-TS supersedes), B-40 (Idea-Framer + Research Presets, DONE per memory), B-37 (Multi-Runtime Backend Abstraction with cloud route, DONE per memory).
Parallel-safe with: E-6 (failure modes), E-7 (Darwinian substrate — shares ArchiveEntry but new admission path), E-9 (benchmarks), E-10.
Source report: workflow/improvement-sweep-2026-05-19/reports/08-self-evolving-prompts.md

## Context

GEPA (Agrawal et al. ICLR 2026 Oral, arXiv:2507.19457) beats MIPROv2 by +10pp average and beats GRPO by up to 20pp using 35x fewer rollouts across six tasks (HotpotQA, IFBench, AIME-2025, HOVER, Papillon, plus a sixth). It uses reflective text feedback + MAP-Elites Pareto frontier mechanism over textual parameters. The `gepa` PyPI package is published standalone (independent of DSPy) so any harness can call it. Zhang et al. ACL Findings 2024 (arXiv:2405.10276) proves small LLMs (LLaMA-2-7B/13B/70B, Mistral-7B) are marginally or detrimentally bad as the OPRO optimizer — so ELAI's local Qwen2.5-Coder-7B cannot drive optimization; only cloud Sonnet/GPT-class can. PromptBridge (arXiv:2512.01420) shows cloud-optimized prompts transfer well to Qwen3-32B / Llama3.1-8B / Gemma3-27B-it. DSPy itself recommends 20/80 train/val split inverted vs neural-net training because prompt optimizers overfit small training sets harder than NN weights. LPO (arXiv:2504.20355) introduces `<edit>...</edit>` tags so only specific spans are optimizable — perfect for ELAI's BabyStepSpec which has freeze-required contract blocks (DETERMINISTIC_CHECK, ACCEPTANCE). ETGPO (arXiv:2602.00997) reports ~1/3 the optimization tokens at comparable accuracy when an error taxonomy is injected — ELAI already has gotchas.md + failure_taxonomy.py serving that role. E-8 wires all of this as a Python-side optimization pipeline that emits blake3-hashed ReasoningTemplate entries into the existing Rust archive at admission, keeping inference hot-path in Rust and optimization rare-path in Python.

## Existing state (B/C/D coverage)

- DELTA on D-1 (ReasoningTemplate archive, PLANNED): E-8 adds an admission path from Python optimizer outputs into the typed archive; archive_status starts at SHADOW per existing rule-lifecycle pattern; promoted via existing BAI Thompson once held-out lift is measured.
- EXTENSION of B-40 (Idea-Framer + Research Presets + Critic Loop, DONE): the critic-loop output text becomes the GEPA feedback signal; no new infrastructure for that.
- EXTENSION of B-37 (Multi-Runtime Backend Abstraction, DONE): cloud Sonnet 4.6 PROVEN per memory:project_b37_shipped serves as the GEPA reflection LM; local Qwen2.5-Coder-7B is the target.
- EXTENSION of B-41 (Per-Level Auditor Modes, DONE): structured auditor PASS/BLOCK reasons wired as the text-feedback stream to GEPA.
- NEW: Python optimization service (.elai_cc/python/elai_optimizer/) driven by `gepa` PyPI package (verify version + last-published date before adding per anti-slopsquatting); produces optimized templates; CLI subcommand triggers it.
- NEW: LPO `<edit>` tag enforcement on BabyStepSpec blocks; contract blocks (DETERMINISTIC_CHECK, ACCEPTANCE, INTEGRATION, EXIT_CRITERIA) are frozen; only role-prompt instruction text + few-shot examples are editable.
- This is NOT a port of DSPy to Rust. Report explicitly flags `dspy-rs` / `DSRs` as toys; optimization lives in Python, inference stays in Rust.

## Reads (mandatory before coding)

- F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\08-self-evolving-prompts.md
- .elai_cc/crates/orchestrator/src/agent_pool.rs (PromptEngineer dispatch; role-prompt source location)
- .elai_cc/crates/orchestrator/src/per_role_runtime.rs (RuntimeBackend mapping for cloud Sonnet)
- .elai_cc/crates/orchestrator/src/template_archive.rs (if E-7 lands first; else D-1 file)
- .elai_cc/python/elai_inference/server.py (sidecar pattern for new optimizer service)
- .claude/agents/prompt-engineer.md (current agent prompt source)
- workflow/papers/dgm-interface.md (if relevant for archive admission)
- arXiv:2507.19457 GEPA paper
- arXiv:2405.10276 small-models-as-optimizers limitation
- arXiv:2512.01420 PromptBridge
- arXiv:2504.20355 LPO
- arXiv:2602.00997 ETGPO error taxonomy
- arXiv:2310.03714 DSPy foundational
- arXiv:2406.07496 TextGrad
- arXiv:2511.12063 TextBO (fallback if GEPA proves too expensive)
- arXiv:2510.08413 PAC-style generalization bounds for prompt optimization
- arXiv:2507.03620 "Is It Time To Treat Prompts As Code?" — documents "cheaper-model degradation" risk

## Tasks

1. Add Python optimizer service at .elai_cc/python/elai_optimizer/optimizer.py (~250 LOC). Entry point: `optimize_template(template_path, train_set_path, val_set_path, reflection_model="claude-4.6-sonnet", target_model_url="http://localhost:8100/v1/messages")`. Uses `gepa` PyPI package (verify via `curl -s https://pypi.org/pypi/gepa/json | python -c "import json,sys;d=json.load(sys.stdin);print(d['info']['version'])"` before adding to pyproject.toml — anti-slopsquatting rule). Wraps the reflection LM with the cloud route from B-37; wraps the target with the local sidecar from sidecar-protocol skill.

2. Define LPO edit-tag enforcement in BabyStepSpec parser (crates/orchestrator/src/baby_step_spec.rs, ~120 LOC delta). Block tags: TASK, GROUNDING, PRE_GATE, TDD, role-prompt content -> wrapped with `<edit>...</edit>`. DETERMINISTIC_CHECK, ACCEPTANCE, INTEGRATION, EXIT_CRITERIA -> wrapped with `<frozen>...</frozen>`. Parser emits SsbActionType::EditableSpansComputed and refuses any optimizer-proposed mutation outside an `<edit>` span.

3. Wire structured auditor feedback as GEPA text-input (crates/orchestrator/src/auditor_feedback_export.rs, ~140 LOC). For each BabyStep run, export {verdict, structured_reasons: Vec<FindingReason>, mast_mode (from E-6 if landed), failing_test_names, full_block_messages} as a JSON envelope written to .elai_cc/optimization/feedback/{run_id}.json. Python optimizer reads from this directory; no Python-driven SSB writes.

4. Add admission path for optimized templates (crates/orchestrator/src/optimizer_admit.rs, ~180 LOC). Python optimizer emits OptimizedTemplate { content, content_blake3, parent_template_blake3, optimizer_version, optimization_run_id, val_lift_pct, held_out_lift_pct, cross_domain_canary_results: Vec<(family, delta)> }. Rust side: validates blake3, runs E-7's contam filter (paraphrase + canary), then admits as archive_status=SHADOW. Existing BAI Thompson + Two-Gate promotes from SHADOW to ACTIVE; held-out lift_pct must >= 5% AND no cross-domain regression (E-7 transfer threshold).

5. Add ETGPO-style top-3 gotchas injection (crates/orchestrator/src/gotchas_inject.rs, ~80 LOC). On optimizer run start, queries gotchas.md + failure_taxonomy.py for last 30 days of top-3 most-active gotcha categories; injects as a `Pitfalls:` block into the role-prompt input that the optimizer sees as part of the textual parameter. Report cites ETGPO ~1/3 token cost at same accuracy.

6. Cross-model transfer validation pass (.elai_cc/python/elai_optimizer/transfer_check.py, ~140 LOC). After GEPA convergence on Sonnet reflection / Qwen target, re-evaluate the optimized template on (a) the same local Qwen target on a held-out test set, (b) two cross-domain canary families per E-7 task 7. If target re-eval < val_lift_pct by more than ε=2pp, mark template as overfit-risk and refuse admission. Implements PromptBridge (arXiv:2512.01420) validity check.

7. CLI surface (crates/cli/src/optimize_cmd.rs, ~180 LOC): `elai optimize template --target <agent_role> --train <path> --val <path> --reflection-model sonnet-4.6 --budget-tokens 200000`. Spawns the Python optimizer process; tails its progress via the SSB feedback directory; reports val + held-out + cross-domain at the end. Refuses execution if reflection model is local (per arXiv:2405.10276).

8. Dashboard surface (crates/dashboard-v2/src/lib/components/OptimizerRunPanel.svelte, ~240 LOC): live progress of an in-flight optimization run; shows current Pareto frontier cells (GEPA's MAP-Elites is the same surface as E-7 archive); shows val vs held-out lift; surfaces overfit-risk red flag; surfaces "reflection-LM is too small" red flag if user mis-configures. Tauri command get_optimizer_run(run_id) returns OptimizerRunState.

9. Determinism rule: Python optimizer output is the optimized template TEXT only; no Python-side mutation of any SSB or archive row. Rust side owns admission + promotion. Add contract test that asserts Python script process has read-only access to .elai_cc/optimization/ and write-only access to .elai_cc/optimization/outputs/. This implements the "optimization rare/batch, inference hot" determinism-first principle the report cites.

10. Add SsbActionType variants (crates/orchestrator/src/ssb.rs, ~50 LOC delta): OptimizerRunStarted, OptimizerProgress, OptimizerConverged, OptimizerOverfitRejected, OptimizedTemplateAdmitted, OptimizedTemplatePromoted. Each carries the optimization_run_id so dashboard can correlate.

11. Add held-out test gate (existing pattern from D-6 or E-7 task 6): every optimized template MUST be evaluated against the held-out store with paraphrase filter + canary GUID active; SHADOW templates that pass enter BAI pool; non-passers logged as OptimizerOverfitRejected.

## WIRING_CONTRACT

produces_symbols:
  - python::elai_optimizer::optimizer::optimize_template
  - python::elai_optimizer::transfer_check::validate_transfer
  - orchestrator::baby_step_spec::editable_spans(spec: &BabyStepSpec) -> EditableSpans
  - orchestrator::auditor_feedback_export::export_for_run(run_id: &str) -> PathBuf
  - orchestrator::optimizer_admit::admit_optimized_template(t: OptimizedTemplate) -> Result<ArchiveAdmission>
  - orchestrator::gotchas_inject::top_gotchas(window_days: u32) -> Vec<GotchaTag>
  - cli::optimize_cmd::optimize_template_command
  - SsbActionType::OptimizerRunStarted
  - SsbActionType::OptimizerProgress
  - SsbActionType::OptimizerConverged
  - SsbActionType::OptimizerOverfitRejected
  - SsbActionType::OptimizedTemplateAdmitted
  - SsbActionType::OptimizedTemplatePromoted
  - SsbActionType::EditableSpansComputed
consumes_symbols:
  - crates/orchestrator/src/template_archive.rs::admit (extends with archive_status=SHADOW path)
  - crates/orchestrator/src/promotion_gate.rs::two_gate_promote (E-7) — promotes SHADOW->ACTIVE
  - crates/orchestrator/src/per_role_runtime.rs (cloud Sonnet route for reflection)
  - .elai_cc/python/elai_inference/server.py (local target inference path)
  - crates/cli/src/main.rs::main (registers optimize subcommand)
reachable_from:
  - cli::main::main `elai optimize template ...`
  - dashboard-v2/src-tauri::get_optimizer_run
stub_ban:
  - "optimize_template\\(.*reflection_model=.qwen" — qwen as reflection LM absent
  - "archive_status: ArchiveStatus::Active" hardcoded on first admit — must start SHADOW
  - "<edit>.*DETERMINISTIC_CHECK.*</edit>" — frozen blocks must not be editable
  - any non-test Python source writing directly to .elai_cc/.elai_cc/indexes/ssb.db (must go via Rust admission)
frontend_pair: crates/dashboard-v2/src/lib/components/OptimizerRunPanel.svelte
observable_effect:
  trigger: "elai optimize template --target executor --train fixtures/optim_train.toml --val fixtures/optim_val.toml --reflection-model sonnet-4.6 --budget-tokens 50000 --include-cross-domain"
  expect_ssb: "SsbActionType::OptimizerRunStarted -> OptimizerProgress (>=3) -> (OptimizerConverged OR OptimizerOverfitRejected) within budget; OptimizedTemplateAdmitted iff val_lift_pct >= 5 AND no cross-domain regression"
  expect_artifact: "stdout shows per-iter val + held-out lift; final printed template; dashboard OptimizerRunPanel renders the same numbers"

## DETERMINISTIC_CHECK

```bash
# 1. gepa pypi package verified at install time (anti-slopsquatting)
python -c "import gepa; print(gepa.__version__)"
# Expected: a non-trivial version string; manual check that publish date < 24 months old

# 2. Reflection LM cannot be qwen (refusal at CLI parse)
cargo run -p elai-cli -- optimize template --target executor --train x --val y --reflection-model qwen 2>&1 | grep -i "reflection-model must be cloud"
# Expected: at least one match.

# 3. LPO edit-spans extracted correctly
cargo test -p elai-orchestrator baby_step_spec::tests::deterministic_check_block_is_frozen
# Expected: PASS.

# 4. Auditor feedback export writes JSON per run
ls .elai_cc/optimization/feedback/ | head -3
# Expected: at least one *.json file exists after a real audit run.

# 5. Optimized template admitted only as SHADOW
cargo test -p elai-orchestrator optimizer_admit::tests::first_admit_status_is_shadow
# Expected: PASS.

# 6. Cross-domain transfer check blocks overfit candidate
cargo test -p elai-orchestrator optimizer_admit::tests::cross_domain_regression_rejected
# Expected: PASS.

# 7. Held-out test set referenced; train/val/holdout sizes recorded
rg "train_size|val_size|holdout_size" .elai_cc/python/elai_optimizer/optimizer.py
# Expected: at least one match per token.

# 8. Python process FS access restricted
cargo test -p elai-orchestrator optimizer_admit::tests::python_cannot_write_ssb_directly
# Expected: PASS.

# 9. blake3 for content hash (anti-stale)
rg "blake2|sha2" .elai_cc/python/elai_optimizer .elai_cc/crates/orchestrator/src/optimizer_admit.rs
# Expected: zero hits.
```

## ACCEPTANCE

- [ ] `elai optimize template` runs end-to-end on a 50-task fixture, converges within budget, produces an OptimizedTemplate with val_lift_pct populated.
- [ ] Reflection-LM is cloud Sonnet 4.6 (or GPT-class via the same B-37 RuntimeOrchestrator route); CLI refuses local 7B as reflection LM with clear error.
- [ ] DETERMINISTIC_CHECK, ACCEPTANCE, INTEGRATION, EXIT_CRITERIA blocks are frozen via `<frozen>` tags; optimizer-proposed edits inside these blocks are rejected at admission.
- [ ] Held-out test set is separate from training set (DSPy 20/80 split documented in module README); admission requires held-out_lift_pct >= val_lift_pct minus ε=2pp; otherwise OptimizerOverfitRejected.
- [ ] Cross-domain canary check runs per E-7 task 7 threshold (-20% rel / -5% abs); regression blocks admission.
- [ ] Top-3 gotchas injection visible in the optimizer's textual parameter (ETGPO pattern); SsbActionType::EditableSpansComputed records the injection.
- [ ] Dashboard OptimizerRunPanel renders live progress from at least one real cloud-Sonnet-driven run, passes svelte-check.
- [ ] Anti-slopsquatting: `gepa` PyPI package version + last-published date recorded in pyproject.toml comment; verified before install.
- [ ] No .unwrap() in new Rust non-test code; Python uses typed dataclasses; pydantic acceptable if pinned.
- [ ] orjson over json in Python optimizer (anti-stale rule); FastAPI+uvicorn pattern not relevant here (this is a batch service, not HTTP).

## Tests to add

- crates/orchestrator/tests/baby_step_spec_edit_spans.rs: editable + frozen spans correctly identified for all 8 block types
- crates/orchestrator/tests/auditor_feedback_export_format.rs: JSON envelope matches optimizer's expected schema
- crates/orchestrator/tests/optimizer_admit_shadow_default.rs: first admit always SHADOW
- crates/orchestrator/tests/optimizer_admit_overfit_rejected.rs: val_lift > held_out_lift + 2pp triggers OptimizerOverfitRejected
- crates/orchestrator/tests/optimizer_admit_cross_domain_blocks.rs: -25% rel on a non-target family blocks
- crates/contracts/tests/optimizer_ssb_action_types_contract.rs: 7 new SsbActionType variants serialize-roundtrip
- crates/contracts/tests/optimizer_python_fs_isolation_contract.rs: Python process cannot write SSB directly
- crates/dashboard-v2/src/lib/components/OptimizerRunPanel.test.ts: panel renders 3-stage progress from mock SSB
- python/elai_optimizer/tests/test_transfer_check.py: re-eval gap > 2pp triggers overfit flag
- python/elai_optimizer/tests/test_optimize_smoke.py: 5-iter smoke against a deterministic Sonnet stub returns a non-trivial optimized template

## Files touched

- CREATE: .elai_cc/python/elai_optimizer/__init__.py (~10 LOC)
- CREATE: .elai_cc/python/elai_optimizer/optimizer.py (~250 LOC)
- CREATE: .elai_cc/python/elai_optimizer/transfer_check.py (~140 LOC)
- CREATE: .elai_cc/python/elai_optimizer/pyproject.toml (gepa dep, version-verified)
- CREATE: .elai_cc/python/elai_optimizer/tests/ (~200 LOC across 2 test files)
- CREATE: crates/orchestrator/src/auditor_feedback_export.rs (~140 LOC)
- CREATE: crates/orchestrator/src/optimizer_admit.rs (~180 LOC)
- CREATE: crates/orchestrator/src/gotchas_inject.rs (~80 LOC)
- CREATE: crates/cli/src/optimize_cmd.rs (~180 LOC)
- CREATE: crates/dashboard-v2/src/lib/components/OptimizerRunPanel.svelte (~240 LOC)
- MODIFY: crates/orchestrator/src/baby_step_spec.rs (~120 LOC delta for edit/frozen tags)
- MODIFY: crates/orchestrator/src/ssb.rs (+7 SsbActionType variants, ~70 LOC delta)
- MODIFY: crates/cli/src/main.rs (~25 LOC to register optimize subcommand)
- MODIFY: crates/dashboard-v2/src-tauri/src/lib.rs (~50 LOC for get_optimizer_run command)

## Citations / Grounding

- arXiv:2507.19457 (GEPA, Agrawal et al. ICLR 2026 Oral) — +10pp avg vs MIPROv2, +12pp AIME-2025, +14pp Qwen3-8B (38→52%), 35x fewer rollouts than GRPO; foundational optimizer
- arXiv:2405.10276 (Zhang et al. ACL Findings 2024) — small LLMs ineffective as OPRO optimizer; grounds cloud-reflection / local-target split
- arXiv:2512.01420 (PromptBridge) — GPT-4o prompts transfer to Qwen3-32B / Llama3.1-8B / Gemma3-27B-it; grounds transfer-check pass
- arXiv:2504.20355 (LPO) — `<edit>` tag pattern; grounds frozen-block enforcement
- arXiv:2602.00997 (ETGPO) — ~1/3 optimization-phase tokens at comparable accuracy; grounds gotchas injection
- arXiv:2310.03714 (DSPy foundational) — declarative LM pipelines; module separation
- arXiv:2406.07496 (TextGrad) — textual gradients; grounds text-feedback enrichment principle (auditor structured reasons -> optimizer)
- arXiv:2511.12063 (TextBO) — UCB-BO over text + best-of-N; fallback if GEPA cost too high
- arXiv:2510.08413 (PAC-style generalization bounds for prompt optimization) — informative priors tighten bounds; grounds train/val/holdout split discipline
- arXiv:2507.03620 (Is It Time To Treat Prompts As Code?) — documented cheaper-model degradation; grounds re-eval on target before admit
- DSPy docs https://dspy.ai/learn/optimization/overview/ — 20/80 train/val split; grounds the inverted split rule

## Open questions / risks

- gepa PyPI package version + maintenance status must be verified at install time per anti-slopsquatting; if abandoned, fall back to a vendored copy of the published-paper algorithm. Report cites the package exists but does not guarantee long-term maintenance.
- Cost projection per template optimization is UNCONFIRMED: report says "What is the actual cost (cloud reflection-LM tokens) per BabyStep template optimized to convergence?" — E-8 must surface per-template cost in the dashboard so user can budget; default cap is 200k tokens per run.
- Reflection-LM strength dependency: GEPA paper's strongest results use stronger LMs than Sonnet 4.6; Qwen3-8B as reflection LM still yielded +14pp on AIME-2025 but at lower base. E-8 ships with Sonnet 4.6 default but logs reflection-LM model_id so a future audit can correlate optimizer lift with reflection LM choice.
- Plateau after 2 iterations risk per report ("quality gains plateau after 2 iterations while cost rises"): E-8 implements hard early-stopping on validation lift, configurable patience=2 outer iters with min delta=1pp.
- Pareto-elite collapse if MAP-Elites cells correlate: E-8 inherits E-7's cell-axis definition; if E-7 ships with the same (latency × token × failure-mode) axes, the optimizer + Darwinian substrate share the same archive surface (good) but also share the same collapse risk (bad). Mitigation: dashboard shows cell-coverage entropy; if < 1.0 nats, surface red flag.
- Search-space leak from optimizer LM prior knowledge of common benchmarks: report cites arXiv:2510.08413 PAC-style bounds only with informative priors. ELAI's BabyStep DETERMINISTIC_CHECK signatures are internal — low leak risk — but the role-prompts themselves may contain task descriptions that are leak-prone. Mitigation: contam filter from E-7 task 6 applies to optimizer training set, not just held-out.
- DO NOT port DSPy to Rust per report explicit recommendation; `dspy-rs` / `DSRs` are toys. E-8 stays Python-side for optimization, Rust-side for inference.

## Next

- E-8 unblocks E-9 — the optimized templates become the regression baseline against SWE-bench-Live + tau2-bench + LiveCodeBench (each suite gets a baseline template + optimized template comparison)
- E-7's MAP-Elites archive + Two-Gate guardrail is the admission gate; E-8 produces SHADOW entries that go through the same gate
- E-10 critic's recommendation 3 ("Stop pretending CoT echoes are grounding") is partially addressed by E-8: GEPA's text feedback IS the grounding loop, with auditor verdicts as ground-truth signal; future E-N can add Curry-Howard typed CoT certs per E-10's recommendation
- Future work: extend optimizer to run on per-AgentRole basis (E-8 ships for one role at a time; iterating across all 8 roles is a follow-on)
