# D-22: Conformal calibration validation under distribution shift

Objective: verify D-16 UniCR empirical refusal rate stays within user
ε=0.1 across 3 distributions (in-domain, near-OOD, far-OOD). Gates D-16
on UniCR's FORMAL distribution-free guarantee (arxiv:2509.01455, T1).
Estimated effort: 1-2 sessions.
Pre-requisites: D-16 (UniCR implementation), D-6 (held-out store provides
  distributions).
Parallel-safe with: D-19..D-25.

## Context

UniCR provides distribution-free guarantee REGARDLESS of distribution shift.
D-22 instruments three distributions:
- **In-domain**: tasks from same family as calibration corpus.
- **Near-OOD**: tasks from related but different family (e.g. corpus is
  code, near-OOD is math).
- **Far-OOD**: tasks from semantically distant family (e.g. corpus is
  code, far-OOD is prose-reasoning + planning).

Empirical refusal rate measured on each; should remain in [τ-ε, τ+ε] for
ε=0.1 per UniCR theorem §3.

## Reads

- F:\.ELAI_workflow\docs\specs\guide\real-problem-solving-grounded-2026-05-19.md
  (§4.5 UniCR — lines 274-286)
- F:\.ELAI_workflow\workflow\papers\unicr-2509.01455-interface.md (D-16)
- .elai_cc/crates/orchestrator/src/conformal_calibration.rs (D-16)
- .elai_cc/crates/orchestrator/src/held_out.rs (D-6)

## Tasks

1. **Distribution fixtures** at .elai_cc/benchmarks/conformal/:
   - in_domain/, near_ood/, far_ood/ — each with 100 labeled claims
     (300 total).
   - Labels: claim, ground_truth_available, hand-labeled.

2. **ConformalValidationRunner** in
   .elai_cc/crates/orchestrator/src/benchmarks/conformal_validation_runner.rs:
   - `struct ConformalValidationRunner { fixtures_dir: PathBuf, tau: f32,
     epsilon: f32 }`
   - `fn run(&self) -> ConformalValidationReport`:
     - For each distribution, compute empirical refusal rate via
       refusal_decision (D-16).
     - Compare to expected band [τ-ε, τ+ε].
     - Report shifts per distribution.

3. **CLI `elai bench conformal-validate`**.

4. **Decision doc** workflow/reasoning/D-22-decisions.md (3 decisions):
   - (a) Why 3 distributions: UniCR §6 evaluates similar 3-tier setup;
     matches paper.
   - (b) Why 100 per distribution: enough for ε=0.1 confidence; below 500
     calibration-set bound but distribution-shift validation can be sparser.
   - (c) Why ε=0.1 default: D-16 default.

## WIRING_CONTRACT

produces_symbols:
  - ConformalValidationRunner + ConformalValidationReport
  - `elai bench conformal-validate` CLI
consumes_symbols:
  - refusal_decision (D-16)
  - τ from .elai_cc/calibration/tau.txt
  - SsbWriter
reachable_from:
  - CLI + CI workflow
stub_ban:
  - Each distribution's empirical refusal rate MUST be reported
  - FAIL verdict on any distribution shifting > ε
observable_effect:
  trigger: `elai bench conformal-validate`
  expect_ssb: BenchmarkStarted + BenchmarkResult; per-distribution rate row
  expect_artifact: report file at .elai_cc/benchmarks/conformal/reports/

## DETERMINISTIC_CHECK

```bash
rg "struct ConformalValidationRunner" .elai_cc/crates/orchestrator/src/benchmarks/
ls .elai_cc/benchmarks/conformal/{in_domain,near_ood,far_ood}/*.json | wc -l
# Expected: >= 300
cargo check --workspace 2>&1 | tail -3
```

## ACCEPTANCE

- [ ] ConformalValidationRunner compiles
- [ ] 300 fixtures (100 per distribution) present + hand-labeled
- [ ] `elai bench conformal-validate` exit 0 if all 3 within ε
- [ ] cargo check --workspace: zero new errors
- [ ] Cloud smoke test A: live D-16 calibration corpus (≥500 tuples after
      session bootstrap); empirical refusal rate within ε=0.1 on all 3
      distributions
- [ ] Cloud smoke test B: far_ood distribution intentionally chosen to shift;
      validation should still hold per UniCR theorem
- [ ] Decision doc workflow/reasoning/D-22-decisions.md

## Files touched

- ADD: .elai_cc/crates/orchestrator/src/benchmarks/conformal_validation_runner.rs
- ADD: .elai_cc/crates/cli/src/bench_conformal_validate.rs
- ADD: .elai_cc/benchmarks/conformal/{in_domain,near_ood,far_ood}/{claim_id}.json × 300
- MODIFY: .elai_cc/crates/cli/src/commands.rs (SlashCommand::BenchConformalValidate)
- ADD: workflow/reasoning/D-22-decisions.md

## Out-of-scope

- Multi-domain τ per family (DEFERRED)

## Next

D-23 (sycophancy detection benchmark).
