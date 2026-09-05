# D-30: Conformal calibration bootstrap from generic corpus (DEFERRED)

Objective: UniCR (D-16) needs N=500 calibration tuples to be useful per
arxiv:2509.01455 §3.4 finite-sample bound. Early sessions cannot calibrate
locally. Spike: bootstrap from generic LLM-calibration corpus, then refine
per-project. Real-problem open Q5 (lines 413).
Estimated effort: 2-4 sessions.
Status: DEFERRED.
Pre-requisites: D-16 (UniCR implementation), public calibration corpus
  identified (HuggingFace; specific dataset TBD at unblock).
Parallel-safe with: D-28, D-29.

## Context

Real-problem Q5 (lines 413): UniCR needs ~500 tuples; early ELAI session
cannot calibrate without bootstrap data. Hypothesis: bootstrap from generic
LLM-calibration corpus (e.g. TruthfulQA confidence labels, MMLU verified
subsets), then refine per-project as session corpus accumulates.

Risk: generic corpus distribution mismatch (ELAI is code+reasoning, generic
is QA+factual); bootstrap τ may be poorly calibrated to ELAI's intrinsic
signals. Mitigation: τ updates nightly per D-16 design; bootstrap is only
the starting point.

## Reads

- F:\.ELAI_workflow\docs\specs\guide\real-problem-solving-grounded-2026-05-19.md
  (Q5 — line 413; §4.5 UniCR — lines 274-286)
- F:\.ELAI_workflow\workflow\papers\unicr-2509.01455-interface.md (D-16)
- .elai_cc/crates/orchestrator/src/conformal_calibration.rs (D-16)

## Paper-ingestor pre-step

Identify generic calibration corpus:
- TruthfulQA (arxiv:2109.07958)
- MMLU verified subsets (HuggingFace)
- HaluEval (arxiv:2305.11747)

Choose one with publicly-available confidence labels + ground-truth-
available flags. Spawn paper-ingestor for chosen corpus.

## Tasks

1. **Corpus selection + ingest**:
   - Author selection.md at .elai_cc/calibration/bootstrap/ documenting
     choice + rationale.
   - Download corpus to .elai_cc/calibration/bootstrap/{corpus}/.
   - Convert to CalibrationTuple format (D-16) via shim script.

2. **Bootstrap τ compute**:
   - Run D-16 compute_tau on bootstrap corpus.
   - Save τ_bootstrap to .elai_cc/calibration/tau_bootstrap.txt.
   - On D-16 startup, if .elai_cc/calibration/corpus/ < N=500 tuples, use
     τ_bootstrap; once N≥500, switch to local-corpus τ.

3. **Validation harness**:
   - Run D-22 ConformalValidationRunner with τ_bootstrap on synthetic
     ELAI claims; verify empirical refusal rate within ε on held-out.
   - If outside ε: report distribution-mismatch; document Path B failure.

4. **Path A (productize)**: bootstrap improves over local-only cold-start;
   integrate τ_bootstrap selection logic into D-16.

5. **Path B (infeasibility note)**: bootstrap mismatches local distribution
   significantly; document with concrete blocker (specific D-22 distribution
   shift > ε).

6. **Decision doc** workflow/reasoning/D-30-decisions.md (DEFERRED —
   drafted at unblock).

## WIRING_CONTRACT

produces_symbols:
  - warm_start_tau (calibration_bootstrap.rs:346) — computes tau from bootstrap corpus
  - load_corpus (calibration_bootstrap.rs:292) — loads PublishedCalibrationRow entries
  - CalibrationBootstrapCorpus (calibration_bootstrap.rs:189) + CalibrationBootstrapCorpus::to_calibration_corpus (calibration_bootstrap.rs:216)
  - PublishedCalibrationRow (calibration_bootstrap.rs:148) + BootstrapWarmStartOutcome (calibration_bootstrap.rs:239)
  - bootstrap_resample (calibration_bootstrap.rs:431) + ci_90_from_sorted (calibration_bootstrap.rs:525)
  - DEFAULT_BOOTSTRAP_EPSILON + DEFAULT_BOOTSTRAP_MIN_CORPUS_SIZE + PRODUCTION_FLOOR_TUPLES (calibration_bootstrap.rs:56,65,72)
  - SsbActionType::CalibrationBootstrapCompleted + CalibrationBootstrapCompletedPayload (ssb.rs:6746)
  - CalibrationBootstrapMode (runtime_config.rs:487) gated by ELAI_CALIBRATION_BOOTSTRAP_ENABLE
  # Corpus download + selection.md + tau_bootstrap.txt are DEFERRED (corpus TBD at unblock).
consumes_symbols:
  - D-16 conformal_calibration::CalibrationCorpus (via to_calibration_corpus conversion)
  - D-16 compute_tau (called by warm_start_tau after corpus conversion)
  - CalibrationBootstrapMode::from_env (runtime_config.rs:499) — ELAI_CALIBRATION_BOOTSTRAP_ENABLE
  - D-22 ConformalValidationRunner (validation harness; DEFERRED)
  - SsbWriter
reachable_from:
  - task_runner.rs:5980 (ELAI_CALIBRATION_BOOTSTRAP_ENABLE=1 path in drive_task)
  - task_runner.rs:16213 (early-exit guard when mode is Off)
  - ELAI_CALIBRATION_BOOTSTRAP_ENABLE=1 env var (wire-dark default: Off)
stub_ban:
  - warm_start_tau MUST require corpus.size() >= DEFAULT_BOOTSTRAP_MIN_CORPUS_SIZE; not panic on empty
  - CalibrationBootstrapCompleted MUST be emitted after warm_start_tau runs; not SKIP
  - bootstrap tau MUST fall back to tau_bootstrap when local corpus < PRODUCTION_FLOOR_TUPLES=500
observable_effect:
  trigger: ELAI_CALIBRATION_BOOTSTRAP_ENABLE=1 elai task <any_task> with bootstrap corpus present
  expect_ssb: 1 CalibrationBootstrapCompleted row with corpus_slug + warm_start_tau_value
  expect_artifact: .elai_cc/calibration/tau_bootstrap.txt written (DEFERRED until corpus downloaded)
  # DEFERRED legs: corpus download (.elai_cc/calibration/bootstrap/{corpus}/), selection.md,
  # tau_bootstrap.txt, and D-22 validation harness all pending corpus identification.
  # The Rust dispatch path (task_runner.rs:5980 + 16213) IS wired and live.

## DETERMINISTIC_CHECK

DEFERRED.

## ACCEPTANCE

- [ ] DEFERRED. Sessions unblock when: corpus selection finalized.
- [ ] Paper-ingestor for chosen corpus output present.
- [ ] τ_bootstrap computed + persisted.
- [ ] D-22 validation harness reports empirical refusal rate within ε
      under τ_bootstrap (Path A); OR documents distribution mismatch
      with concrete blocker (Path B).
- [ ] Decision doc workflow/reasoning/D-30-decisions.md.

## Files touched (anticipated)

- ADD: .elai_cc/calibration/bootstrap/{corpus}/
- ADD: .elai_cc/calibration/bootstrap/selection.md
- ADD: .elai_cc/calibration/tau_bootstrap.txt
- MODIFY: .elai_cc/crates/orchestrator/src/conformal_calibration.rs (D-16
    startup logic gains bootstrap fallback)
- ADD: workflow/reasoning/D-30-decisions.md

## Out-of-scope

- Custom calibration corpus authoring (DEFERRED beyond D-30; bootstrap
  selection is sufficient first step)
- Multi-corpus blending (DEFERRED)

## Next

D-plan complete after D-30 unblocks. All deferred sessions (D-28..D-30)
operate independently; sequencing within Wave 5 is by GPU availability +
research priority.
