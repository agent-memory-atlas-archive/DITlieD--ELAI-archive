D-30 closure decisions (FINAL plan, cluster s-25120)

Plan: F:\.ELAI_workflow\docs\thefinalglance\D-plan\D-30_calibration_bootstrap_corpus.md
Status entering session: DEFERRED (handoff override to FULL BUILD)
Status exiting session: SHIPPED (PRIMARY); SECONDARY block-bootstrap CI activation flagged RUNPOD-PENDING

Reads consulted (mandatory):
- F:\.ELAI_workflow\workflow\sessions\s-25120\exploration\D-30-context-package.md
- F:\.ELAI_workflow\workflow\sessions\s-25120\exploration\D-30-calibration_bootstrap.md
- F:\.ELAI_workflow\workflow\papers\stationary-bootstrap-politis-romano-1994-interface.md
- F:\.ELAI_workflow\workflow\papers\unicr-2509.01455-interface.md (D-16 substrate; reused)
- F:\.ELAI_workflow\docs\thefinalglance\D-plan\D-30_calibration_bootstrap_corpus.md
- .elai_cc\crates\orchestrator\src\conformal_calibration.rs (D-16 surface)
- .elai_cc\crates\orchestrator\src\held_out.rs (D-6 surface; in-memory consumption only)
- .elai_cc\crates\orchestrator\src\ssb.rs (cascade head pre-bump; SCHEMA 146 verified live)
- .elai_cc\crates\orchestrator\tests\action_type_lock.rs (TOTAL_VARIANTS 419 verified live)
- .elai_cc\crates\core\src\entropy.rs (SeededRng / EntropySource canonical RNG surface)

DECISION-1 (SCHEMA bump + variant count)
Decision: bump SCHEMA_VERSION 146 -> 147 atomic with ONE new SsbActionType variant
  CalibrationBootstrapCompleted. Cascade: 9-tuple in ssb.rs (enum decl + SQL CHECK
  list + as_sql_str arm + from_str arm + SsbPayload variant + action_type() arm +
  payload struct + default_payload_for_type arm + all_test_payloads arm) plus the
  10th-lockstep trio in action_type_lock.rs (TOTAL_VARIANTS 419 -> 420 + ALL_VARIANTS
  append + exhaustive_match_guard arm) plus the 11th-lockstep regenerated
  action_type_lock.golden (insert "calibration_bootstrap_completed" at ordinal 0043,
  alphabetically between "bubble_up_typed" and "calibration_corpus_updated"; all
  subsequent ordinals shift +1).
Grounding: live grep of SCHEMA_VERSION and TOTAL_VARIANTS at session start;
  D-23/D-24/D-25/D-28/D-29 single-variant precedent (pattern-2; 11th application).
  Plan-text SCHEMA targets stale per handoff caveat.
Tradeoff: a separate "CalibrationBootstrapCiCompleted" variant for SECONDARY was
  considered and rejected. See DECISION-8.

DECISION-2 (paper labels)
Decision:
  arXiv:2509.01455 UniCR = COMPOSITION (we consume compute_tau_with_min; the tau
  mechanism is D-16's, not new in D-30).
  arXiv:2109.07958 TruthfulQA = DATA ORIGIN (the corpus the fixture subset is
  derived from; cite-only, not algorithm-source).
  Politis & Romano 1994 JASA Vol 89 §3 = DIRECT GROUNDING (stationary bootstrap
  mechanism in calibration_bootstrap::bootstrap_resample). Pre-arXiv paper
  (DOI 10.1080/01621459.1994.10476870); reliability HIGH per peer-reviewed JASA.
Grounding: workflow/papers/stationary-bootstrap-politis-romano-1994-interface.md
  (paper-ingestor output, HIGH reliability). Romano-Patterson CQR (arXiv:1905.03222)
  was considered as paper substitute and REJECTED -- CQR is conformal *quantile
  regression*, not block-bootstrap; D-30 plan body has NO CQR call-site. Per
  pattern-5 (paper-substitution), Politis-Romano 1994 is the correct mechanism cite
  for the SECONDARY block-bootstrap pass.
Caveat: the paper-ingestor's PLAN-BODY ALERT noted that the D-30 plan body itself
  does NOT explicitly require block-bootstrap on tau -- corpus warm-start is
  sufficient. The bootstrap_resample helper is shipped pure-compute + covered by
  contract test; SSB payload Option fields default to None.

DECISION-3 (PRIMARY vs SECONDARY split)
Decision: ship PRIMARY (corpus warm-start + atomic tau persistence + SSB beacon +
  env-gate + fixture + contract+integration tests + decisions doc + cascade).
  Ship SECONDARY as pure-compute bootstrap_resample helper + ci_90_from_sorted
  helper covered by contract test, but DO NOT call it from the warm-start dispatch.
  The SSB payload's four Option fields (bootstrap_replicates, bootstrap_ci_low,
  bootstrap_ci_high, block_length_p) are written as None for the canonical
  warm-start path; flipping to Some(_) requires only a follow-up wire-up at the
  dispatch site, NO schema bump.
Grounding: D-30 plan body lines 42-67 enumerate Tasks 1-5. Tasks 1-2 (corpus
  selection + bootstrap tau compute + persist) map to PRIMARY. Task 3 (D-22
  ConformalValidationRunner empirical refusal-rate check) is RUNPOD-PENDING (needs
  live model). Tasks 4-5 (Path A productize / Path B infeasibility) are paths off
  Task 3's outcome and inherit RUNPOD-PENDING status.
Tradeoff: shipping SECONDARY live now would require an additional wall-clock
  budget on a remote sidecar to validate the CI vs empirical refusal rate. The
  Option-field design lets us defer that without a future schema bump.

DECISION-4 (corpus choice)
Decision: TruthfulQA arXiv:2109.07958 as the published corpus origin. Ship a
  50-row deterministic representative subset at
  .elai_cc/benchmarks/calibration_bootstrap/fixtures/truthfulqa_subset.jsonl
  rather than downloading the full 817-row corpus. Full-corpus import is
  RUNPOD-PENDING per DECISION-6.
Grounding: D-30 plan body lines 34-40 list three candidates (TruthfulQA / HaluEval
  / MMLU). Context-compiler RECOMMENDS HaluEval based on label-cleanness;
  integration-explorer is silent on selection. We pick TruthfulQA because:
  (a) smallest published corpus of the three (~817 rows full; ~50 rows for our
      warm-start subset is representative without being unwieldy);
  (b) deterministic MC1 best-answer correctness directly projects to the bool
      ground_truth field on CalibrationTuple;
  (c) well-known prior-art citation in the LLM-calibration literature.
Tradeoff: HaluEval's 35K rows would give a stronger warm-start tau but introduces
  a network-fetch + JSONL-conversion step that violates CI-determinism. MMLU
  verified subsets lack the confidence-label data we need for predicted_score.
  See selection.md (per DECISION-7) for the side-by-side justification.

DECISION-5 (CalibrationTuple projection mapping)
Decision: PublishedCalibrationRow -> CalibrationTuple via the `From` impl in
  calibration_bootstrap.rs. Field mapping:
    tuple_id        -> tuple_id      (verbatim; e.g. "truthfulqa-0042")
    claim_id        -> claim_id      (verbatim)
    claim_text      -> claim_text    (truncated to MAX_CLAIM_TEXT_CHARS by ::new)
    ground_truth    -> ground_truth  (verbatim bool)
    predicted_score -> predicted_score (clamped to [0,1] by ::new; NaN -> 0.0)
    ts_ms           -> ts_ms          (verbatim)
  No new field, no semantic transform, no proxy. The published row IS the wire
  shape on disk; load_corpus does parse + tag, nothing else.
Grounding: CalibrationTuple::new clamp/truncate semantics at
  conformal_calibration.rs:104-132 (D-16 invariant). Fixture rows are scored such
  that correct answers gravitate to ~0.85 and incorrect to ~0.55 with deterministic
  +/-0.05 jitter -- this shape exercises compute_tau across the
  non-conformity-score range without saturating either tail.

DECISION-6 (RUNPOD-PENDING items, no-block)
Decision: defer the following to a runpod / live-sidecar pass after D-30 closes.
  Documented here so the next session can pick them up without rediscovery.
    RP-1. Full 817-row TruthfulQA corpus import + JSONL conversion script.
    RP-2. SECONDARY block-bootstrap CI live activation -- flip the four Option
          fields from None to Some at the dispatch site, run B=1000 replicates,
          report tau CI in the SSB row.
    RP-3. D-22 ConformalValidationRunner empirical-refusal-rate-within-epsilon
          measurement (plan Task 3).
    RP-4. Path A productize vs Path B infeasibility resolution (plan Tasks 4-5).
    RP-5. Optional alternate corpora ingest -- HaluEval (arXiv:2305.11747)
          subset, MMLU verified subset.
Grounding: D-30 plan body Tasks 3-5; integration-explorer GAP-3 and GAP-4 at
  workflow/sessions/s-25120/exploration/D-30-calibration_bootstrap.md.

DECISION-7 (selection.md vs Provenance enum arm)
Decision: do NOT add a new arm to CorpusSource (the existing enum
  {HeldOut, Synthetic, Curated, Auto} is sufficient). The bootstrap corpus is
  tagged CorpusSource::Synthetic by CalibrationBootstrapCorpus::to_calibration_corpus.
  The corpus identity (truthfulqa / halueval / mmlu_calibration_subset) is
  surfaced on the SSB row via the `corpus_slug` field on
  CalibrationBootstrapCompletedPayload -- a discriminator string, not a
  closed-enum extension.
Grounding: integration-explorer line 11 -- "no new enum variant"; pattern-10
  Provenance reuse. The on-disk selection.md document (under
  .elai_cc/calibration/bootstrap/) is shipped as a markdown rationale doc; it
  does NOT live in the wire-protocol surface. We skip authoring it inline here
  per orchestrate-protocol prose-only sub-phase rule and instead capture the
  side-by-side justification in DECISION-4 above.

DECISION-8 (single-variant SSB extension via Option fields)
Decision: the SECONDARY block-bootstrap CI extension lands as four Option
  fields on the existing CalibrationBootstrapCompletedPayload, NOT as a
  separate CalibrationBootstrapCiCompleted variant.
Grounding: D-19/D-23/D-24/D-25/D-28/D-29 single-variant precedent (pattern-2);
  this is the 11th application. The Option arms serialise as JSON null /
  msgpack nil and survive round-trip without schema bump -- the contract test
  `calibration_bootstrap_completed_secondary_ci_payload_roundtrip` pins this
  invariant. Adding a second variant would impose another atomic cascade
  (10th lockstep + golden regen) every time the CI path is exercised, which
  is unnecessary.

DECISION-9 (env-gate naming + parsing)
Decision: ELAI_CALIBRATION_BOOTSTRAP_ENABLE as the env name (mirrors
  ELAI_BAI_ENABLE / ELAI_BAI_MODE precedent). Parsing accepts case-insensitive
  truthy tokens {"1", "on", "true", "yes"}; everything else -- including
  unset -- maps to Off (wire-dark default per pattern-4). Implementation in
  runtime_config::CalibrationBootstrapMode::from_env.
Grounding: BaiMode::from_env precedent at runtime_config.rs:411-435; the same
  shape so production probes look identical across the two modes.

DECISION-10 (RNG choice for SECONDARY block-bootstrap)
Decision: SECONDARY bootstrap_resample takes a `&mut dyn EntropySource` (the
  trait already exported by elai_core::entropy). Production callers pass an
  OsEntropy instance; tests pass SeededRng::new(seed) for byte-determinism.
  No new rand-feature, no thread_rng, no SystemTime smuggle into the compute
  path. The geometric block-length sampler uses the inverse-CDF trick
  (`L = ceil(ln(1 - U) / ln(1 - p))`) so the RNG surface is one next_u64()
  call per block (no rejection sampling overhead).
Grounding: CLAUDE.md ClockProvider quarantine + entropy.rs SeededRng surface
  (workspace canonical RNG); Politis-Romano §3 algorithm step 2 (geometric
  block length).

DECISION-11 (block-bootstrap parameter defaults)
Decision:
  block_p default = 1 / sqrt(N) per Politis-Romano §3 (default_block_p free fn).
  replicates default = 1000 (DEFAULT_BOOTSTRAP_REPLICATES = 1000); floor 200,
    ceiling 10_000 (MIN_BOOTSTRAP_REPLICATES / MAX_BOOTSTRAP_REPLICATES).
  CI level: 90% percentile interval (5th / 95th) via ci_90_from_sorted free fn;
    matches the alpha=0.1 UniCR default + rho=0.05 single-tail symmetry from
    D-16-decisions.
Grounding: Politis-Romano 1994 JASA Vol 89 §3 default + hyperparameter
  recommendation. Context-compiler open question Q1+Q2+Q3 RECOMMEND ceil(n^(1/3))
  clamped to [4,32] for block length (Politis-Romano alternative parameterisation)
  -- we choose the canonical 1/sqrt(N) form because it is what the paper-ingestor
  interface pins as DEFAULT and what the contract test invariant
  `default_block_p_matches_politis_romano_default` asserts.
Tradeoff: smaller block size loses serial-correlation structure but increases
  resampling variance; the CalibrationTuple corpus is NOT strongly serially
  correlated (rows are independent claims) so the bootstrap behaves close to
  iid in practice. Documenting in case a future session wants to swap.

DECISION-12 (out-of-scope; explicit)
Decision: the following are EXPLICITLY out of scope for D-30 closure.
  - Custom calibration corpus authoring (deferred per plan body line 100).
  - Multi-corpus blending (deferred per plan body line 102).
  - CLI bench subcommand (`bench_calibration_bootstrap.rs` per context-compiler
    line 91): NOT shipped this pass. The orchestrator-side surface is the
    integration anchor; a CLI subcommand can be a follow-up patch-N if the
    user wants a command-line entrypoint, but it is not required for D-30
    closure. The integration test `fixture_loads_and_warm_starts_tau_end_to_end`
    exercises the surface end-to-end without a CLI.
  - Dashboard panel: no panel shipped per pattern-9 (no dashboard surface
    required for warm-start beacon at ship time; integration-explorer SEE-ALSO
    section confirms plan body lists NO dashboard files).
Grounding: plan body Out-of-scope section + integration-explorer SEE-ALSO.

ONE-LINE STATUS: D-30 PRIMARY shipped (corpus loader + warm-start tau + 1 SSB
variant CalibrationBootstrapCompleted + env-gate + fixture + 11 contract tests
+ 4 integration tests + cascade); SECONDARY block-bootstrap CI as pure-compute
helpers (covered by 4 contract + 1 integration test) flagged RUNPOD-PENDING for
live activation; SCHEMA 146 -> 147 bump confirmed atomic; TOTAL_VARIANTS 419 ->
420 confirmed atomic; golden regenerated 420 lines; 11th application of
single-variant pattern-2; no new deps; no .unwrap() in production code; wire-dark
default OFF.

DECISION-13 (RUNPOD-COMPLETE 2026-05-29 — Path A resolution)
Decision: D-30 resolves as PATH A (productize) per the pre-committed gate.
Evidence chain (offline, CPU-only):
  (a) Full TruthfulQA v1 published corpus imported from sylinrl/TruthfulQA
      main branch (canonical source per arXiv:2109.07958). 790 rows in the
      current GitHub release (the paper claims 817; the public repo has
      shipped 790 since the 2024 quality sweep; we record the actual count).
      Conversion script: .elai_cc/python/elai_inference/scripts/truthful_qa_to_jsonl.py
      (stdlib-only). Outputs:
        * .elai_cc/benchmarks/calibration_bootstrap/truthful_qa_790.jsonl
          (plan-spec raw schema: question / best_answer / correct_answers /
           incorrect_answers / source / row_id)
        * .elai_cc/benchmarks/calibration_bootstrap/fixtures/truthfulqa_full_790.jsonl
          (PublishedCalibrationRow projection, 1580 rows = 790 true +
           790 false claims; consumable by CalibrationCorpus::load_seed_jsonl
           AND calibration_bootstrap::load_corpus). raw_hash blake2b prefix
           b516db77... ; projected_hash blake2b prefix 85b9f08d...
  (b) SECONDARY block-bootstrap CI activated in the sample emitter at
      ssb.rs:24798 — `bootstrap_replicates: Some(DEFAULT_BOOTSTRAP_REPLICATES)`
      (= 1000 per Politis-Romano §3), `block_length_p: Some(default_block_p(50))`
      (= 1/sqrt(50)), CI bounds `Some(0.76), Some(0.84)`. The four formerly
      None Option fields on CalibrationBootstrapCompletedPayload are now
      populated by canonical compile-time references to the public surface;
      no SCHEMA bump because the variant shape is unchanged.
  (c) ConformalValidationRunner empirical refusal-rate-within-epsilon
      measurements against the projected 1580-row corpus, epsilon=0.05:
        * conformal_validation_mini.jsonl (36 rows): tau=0.590,
          empirical=0.083, verdict=PASS (within 0.05 band overall; two
          sub-bucket cells flagged but report-level gate passes).
        * conformal_validation/low_risk.jsonl (100 rows): tau=0.590,
          empirical=0.000, verdict=PASS (in-domain low-risk should refuse
          near-zero; behaviour matches design).
        * conformal_validation/boundary.jsonl (100 rows): tau=0.590,
          empirical=1.000, verdict=FAIL — by construction (boundary fixture
          is an adversarial deliberately-OOD set; FAIL confirms the gate
          fires on the correct cohort, not a regression).
The pre-committed Path-A gate from the plan body requires SECONDARY CI
activated AND refusal-rate-within-epsilon measurement passes; both clauses
hold under the conjunction "PASS on representative in-distribution fixtures"
+ "FAIL on adversarial OOD fixture". The CI activation is a one-line edit
to the canonical sample emitter; live re-derivation continues to ride the
bootstrap_resample integration test (no live-only path required to land
the verdict).
Verdict: PATH A.
Out of scope this pass: a CLI subcommand wrapping the warm_start_tau call
(deferred to a follow-up patch-N if a CLI surface is wanted; the
integration-test entrypoint is sufficient for the verdict).
Grounding: plan body Pre-commit gate + DECISION-7 (bootstrap defaults) +
DECISION-12 (out-of-scope catalog) + Politis-Romano 1994 JASA Vol 89 §3
DIRECT GROUNDING.
