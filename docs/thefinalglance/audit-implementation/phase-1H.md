PHASE 1H AUDIT — SPLADE KEYWORD EXPANSION (CONDITIONAL / EVALUATION)
=====================================================================

Scope: verify whether the three possible outcomes of Phase 1H (EARLY DROP /
DROP / KEEP) are really reachable from the code in
.elai_cc/crates/knowledge/, and whether the full evaluation harness the spec
demands is present. Hostile stance: absence of model files does not excuse
absence of infrastructure, decision documents, or a runnable end-to-end
measurement path.

=====================================================================
GLOBAL SUMMARY
=====================================================================

Feature flag:        PRESENT          (Cargo.toml [features] splade = [])
Module gate:         PRESENT          (lib.rs #[cfg(feature="splade")] pub mod splade)
Splade module:       PRESENT          (crates/knowledge/src/splade.rs, 723 lines)
Unit tests:          PRESENT          (tests/splade_tests.rs, 589 lines, #![cfg(feature="splade")])
Measurement tests:   PRESENT          (tests/splade_measure_tests.rs, 1331 lines)
Grounding doc:       PRESENT          (workflow/reasoning/phase-1H-grounding.md)
Measurements doc:    PRESENT-PARTIAL  (workflow/reasoning/phase-1H-measurements.md — PENDING)
Decision doc:        MISSING          (workflow/reasoning/phase-1H-decisions.md NOT FOUND)
Metrics file:        MISSING          (.elai_cc/metrics/phase_1H.toml NOT FOUND)
Evaluation binary:   MISSING          (examples/measure_1h.rs NOT FOUND)
Model files:         NOT STAGED       (models/splade/model.onnx absent per measurements.md)

=====================================================================
SUB-STEP 1H.1 — SKELETON (feature-gated module + stub)
=====================================================================

Verdict: PASS.

Evidence:
- .elai_cc/crates/knowledge/Cargo.toml lines 36-40:
    [features]
    default = []
    splade = []
  Feature is declared. `ort` remains unconditional on line 27
  ("ort = \"=2.0.0-rc.12\"") as the spec requires (used by
  GraphCodeBERT/Granite regardless).
- lib.rs lines 25-26:
    #[cfg(feature = "splade")]
    pub mod splade;
  Module is re-exported only when the feature is enabled. Without the
  feature, the `splade::SpladeExpander` symbol is absent at compile time.
- splade.rs opens with a module-level doc comment explicitly recording the
  gating policy ("Feature-gated behind cfg(feature = \"splade\")"). The
  struct SpladeExpander (line 109), enum ModelVariant (line 73),
  SpladeConfig (line 79), ExpandedTerm (line 38), SpladeError (line 45),
  and the free functions are all behind the same module gate.

Deterministic check coverage: spec says "cargo check (no features) 0 —
splade NOT included; cargo check --features splade 0 — splade included".
The feature gate is structurally correct to satisfy both checks; actual
cargo output was not captured here (auditor runs compile checks; this
audit is source-level).

=====================================================================
SUB-STEP 1H.2 — FAILING TESTS BEFORE IMPLEMENTATION (TDD red phase)
=====================================================================

Verdict: PASS (tests exist, feature-gated, covering the spec test matrix).

Evidence — tests/splade_tests.rs (header line 16: `#![cfg(feature="splade")]`):
- Categories are declared in the file header (lines 9-15):
  (a-c) expand_query unit tests (require model; gracefully skip)
  (d-e) RRF fusion tests (pure logic, always run)
  (f)   vocab_coverage unit test (requires tokenizer)
  (g)   proptest: weight invariants (require model)
  (h)   integration: BM25-only vs BM25+SPLADE recall comparison
- #[test] markers found at lines 70, 82, 95, 108, 120, 145, 152, 160,
  172, 183, 191, 199, 208, 220, 226, 235, 244, 255, 261 — well over the
  spec minimum of ">=5 unit + 1 property + 1 integration".
- Model-dependent tests skip via `try_make_expander()` returning None when
  `models/splade/model.onnx` + `tokenizer.json` are absent (line 47 check).
  This is correct graceful-skip behavior, not silent success.

Note on "tests fail (red phase)" acceptance clause: current file is the
post-implementation state where model-backed tests would pass if fixtures
were staged. The red-phase TDD artifact is not preserved in git history
from this audit surface; unverifiable from source. Marking this a soft
WARN, not a BLOCK — implementation exists and tests execute correctly.

=====================================================================
SUB-STEP 1H.3 — IMPLEMENTATION (SPLADE inference + RRF + vocab_coverage)
=====================================================================

Verdict: PASS.

Evidence — crates/knowledge/src/splade.rs:
- Dependencies satisfied: `tokenizers = "0.22"` in Cargo.toml line 28;
  `ort = "=2.0.0-rc.12"` line 27. Circuit-breaker wired via
  `use crate::vector::circuit_breaker::{EmbeddingCircuitBreaker,
  BreakerError, FailureKind};` (splade.rs line 16).
- SpladeExpander::new(config, clock) at line 124: loads ONNX via
  ort::Session (build_session line 154, with panic catch_unwind for
  corrupt models lines 166-193), loads BERT tokenizer via
  tokenizers::Tokenizer::from_file (load_tokenizer line 197), configures
  truncation at MAX_SEQUENCE_LENGTH = 512 (line 30), wires circuit
  breaker (line 140).
- Model path configurable via SpladeConfig.model_path and SPLADE_MODEL_PATH
  env var (tests read it; SpladeConfig::default uses
  `models/splade/model.onnx`).
- expand_query (line 227): empty-string early-return (lines 228-230);
  circuit-breaker-guarded inference; graceful degradation on inference
  failure OR breaker-open (both return empty Vec<ExpandedTerm>, not
  panic). Matches spec requirement "Fallback on SPLADE failure: empty
  expansion (BM25-only continues)".
- ModelVariant::PrePooled vs RawMlm dispatch (lines 73-76, 321-370):
  PrePooled applies ReLU + log(1+x) over [1, vocab_size]; RawMlm applies
  max-pool over seq positions then ReLU + log(1+x) over
  [1, seq_len, 30522], with attention-mask handling visible at line 359.
  Shape validation present (PrePooled line 324, RawMlm lines 341-348) —
  returns Err on mismatch instead of panicking. This is the exact
  ONNX-output-varies-by-source branch the spec calls for.
- vocab_coverage (line 421): iterates code tokens, encodes each without
  special tokens, counts ids.len() == 1 (single WordPiece). Returns
  ratio in [0.0, 1.0]. Empty input returns 0.0 (line 422-424). Matches
  spec definition of "fraction of unique code tokens that map to single
  WordPiece (not split)".
- Weighted RRF fusion (line 463, `rrf_fuse`): 1-based ranks (comments
  lines 478, 484), k = max(k, 1) guard (line 474), HashMap aggregation,
  sort by fused score descending. Signature
  `rrf_fuse(bm25_results, splade_results, k=60, weight_bm25, weight_splade)`
  matches spec.
- Adversarial acceptance criteria:
  * "corrupted model returns Err (not panic)": catch_unwind in
    build_session (line 166) converts ort panics into
    SpladeError::ModelLoad.
  * "weights finite and >=0.0": ReLU (line 332: `v.max(0.0)`) and log1p
    (line 333) guarantee >=0.0 and finite for finite inputs.
  * "missing tokenizer files returns Err": load_tokenizer line 198-202.
  * "non-feature build 0 errors no SpladeExpander": guaranteed by the
    lib.rs cfg gate.

Minor WARN (not BLOCK): `DEFAULT_TOP_K = 64` (splade.rs line 27) is used
for expansion top-k; the spec does not pin this value but comparison
with BM25 candidate counts is handled at call sites. Acceptable.

=====================================================================
SUB-STEP 1H.4 — MEASUREMENT (vocab fitness, model comparison, recall,
decision)
=====================================================================

Verdict: PARTIAL — infrastructure present, end-to-end decision not
produced.

Evidence — crates/knowledge/src/splade.rs (measurement utilities lines
494-723) and crates/knowledge/tests/splade_measure_tests.rs:

4a. Vocabulary fitness gate — INFRASTRUCTURE PRESENT.
    * SpladeExpander::vocab_coverage (splade.rs line 421) implemented.
    * Early-exit gate test at splade_measure_tests.rs lines ~1161-1208:
      computes coverage on tree-sitter-extracted code tokens, asserts
      coverage in [0.0, 1.0], emits the "EARLY DROP" log line
      ("vocabulary coverage on code tokens is X% — below 30% threshold")
      and returns without running recall.
    * EarlyDrop variant exists on MeasurementDecision (splade.rs line
      668) with `label()` returning "EARLY DROP" (line 680).
    * extract_code_tokens_from_source() helper used in measure tests
      — confirms tree-sitter extraction path exists at least for Rust
      source (referenced line 1185 in measure tests).

4b. Model variant selection — INFRASTRUCTURE PRESENT.
    * measure_4b_model_variant_selection test (measure_tests.rs line
      ~495-566) covers: A-full vs A-quant latency (p50 with warm-up
      excluded per spec), top-50 Jaccard >= 0.95 triggers "prefer
      quantized" log, falls back to A-only if B (splade-v3) model
      absent.
    * jaccard_similarity (splade.rs line 502) implemented; comment at
      measure_tests.rs line 541-542 emits "4b: Quantization Jaccard
      (mean top-50): {:.4}".

4c. Recall measurement — INFRASTRUCTURE PRESENT, NOT EXECUTED.
    * build_synthetic_corpus() (measure_tests.rs line 617+) constructs
      100 corpus files and 50 queries with the required composition
      (>=15 vocab-mismatch, >=15 distractor, >=10 keyword,
      10 code-syntax).
    * recall_at_k (splade.rs line 518) and wilcoxon_signed_rank
      (splade.rs line 542) implemented; Wilcoxon uses normal
      approximation with continuity correction (line 605) and
      Abramowitz-Stegun CDF (line 615) — no external stats crate,
      acceptable and deterministic.
    * Per-query delta logic with bm25==0 exclusion lives in
      measure_4c_recall_measurement_full around line 1099-1119
      (`let mean_delta = if valid_deltas.is_empty() { ... }` and
      eprintln at line 1119 of std+mean).
    * Keep/drop invocation at line 1153:
      `keep_drop_decision(mean_delta, p, n_nonzero)`.

4d. Keep/drop decision — IMPLEMENTED, UNIT-VALIDATED, NOT END-TO-END.
    * keep_drop_decision (splade.rs line 693): KEEP iff
      mean_delta>=5% AND p<0.05; Inconclusive iff delta>=5% AND
      p>=0.05; Drop otherwise; Drop early if n_nonzero_pairs < 20 (line
      698) — matches spec "assert >=20 non-zero pairs".
    * Decision function validated across all four scenarios at
      measure_tests.rs lines 1220-1232 (d1=KEEP, d2=DROP,
      d3=INCONCLUSIVE, d4=DROP for insufficient pairs).
    * Adversarial tests at lines 1239-1331 cover: distractors present,
      code-syntax queries present, all 50 queries runnable, insufficient
      pairs defaults to DROP.

MISSING ARTIFACTS (spec-required deliverables):
1. workflow/reasoning/phase-1H-decisions.md — NOT FOUND. Glob of
   `workflow/reasoning/phase-1H*` returned only
   `phase-1H-grounding.md` and `phase-1H-measurements.md`. Spec
   EXIT_CRITERIA demands the decision document regardless of
   outcome ("all: feature flag works, module preserved... If any DROP:
   feature flag disabled by default, rationale + measurements logged").
   The measurements.md file explicitly records Decision: PENDING.
2. .elai_cc/metrics/phase_1H.toml — NOT FOUND. Directory listing shows
   phase_1A.toml, phase_1B.toml, phase_1D.toml, phase_1E.toml, but no
   phase_1C or phase_1H. Spec MEASUREMENTS block enumerates
   vocab_coverage_code, model_comparison table, recall_delta_percent,
   wilcoxon_p_value, splade_query_latency_ms p50/p95, model_load_time_s,
   binary_size_delta_mb — none of these are persisted.
3. examples/measure_1h.rs — NOT FOUND. crates/knowledge/examples/
   contains measure.rs, measure_bm25.rs, measure_1e.rs,
   measure_pi_latency.rs, but no measure_1h. Spec does not explicitly
   demand a binary (the test harness doubles as the driver), so this is
   a WARN not a BLOCK — but it means running the measurement requires
   `cargo test --features splade` rather than a dedicated phase-aligned
   runner.

Root cause per measurements.md line 3: "MODELS NOT STAGED —
measurement infrastructure built, awaiting model files." The file
further states (line 32): "Decision: PENDING — requires model files to
be staged at models/splade/".

=====================================================================
EXIT CRITERIA CHECK
=====================================================================

"Three outcomes (EARLY DROP / DROP / KEEP)":
  - All three are REPRESENTABLE in the code
    (MeasurementDecision::{EarlyDrop, Drop, Keep, Inconclusive}) and the
    decision function is unit-tested across them.
  - NONE of them have been SELECTED for this phase. The measurements
    doc says PENDING. No persisted decision exists.

"Feature flag works, module preserved behind cfg(feature="splade")":
  SATISFIED — see 1H.1.

"If KEEP: circuit breaker integrated, RRF fusion operational":
  SATISFIED preemptively — circuit breaker is live
  (splade.rs line 140) and rrf_fuse is implemented (line 463).

"If any DROP: feature flag disabled by default, rationale +
measurements logged":
  PARTIAL — `default = []` in Cargo.toml (line 37) already excludes
  splade; rationale + measurements are NOT logged because no decision
  has been made.

"SPARSE_ONLY cascade degrades to BM25_ONLY when splade disabled":
  NOT VERIFIED here (Phase 1I territory). Out of scope for this audit.

=====================================================================
OVERALL VERDICT
=====================================================================

PARTIAL IMPLEMENTATION.

Code and test infrastructure for 1H.1–1H.4 are real, feature-gated, and
structurally correct. The measurement harness would run end-to-end if
model files were present. Missing artifacts are:

1. phase-1H-decisions.md  (BLOCK — required by EXIT CRITERIA)
2. phase_1H.toml metrics  (BLOCK — required by MEASUREMENTS block)
3. examples/measure_1h.rs (WARN — test harness suffices)

The decision outcome cannot be declared KEEP, DROP, or EARLY DROP from
the artifacts on disk. The claimed "measurement infrastructure built,
awaiting model files" is accurate but the phase is NOT complete because
no empirical decision has been recorded and no metrics are persisted.

=====================================================================
DECISION OUTCOME
=====================================================================

UNKNOWN.

Reasoning: measurements.md explicitly says Decision: PENDING, no
phase_1H.toml metrics exist, no phase-1H-decisions.md exists, model
fixtures are not staged. The three possible outcomes (EARLY DROP,
DROP, KEEP) are each reachable in code but none have been selected.
This phase is effectively suspended at the model-staging pre-gate.

=====================================================================
FEATURE FLAG PRESENT?
=====================================================================

YES.

Cargo.toml [features] block declares `splade = []` (line 39). Default
feature set is empty (line 37) so `cargo build` without `--features
splade` excludes the module, confirmed by the
`#[cfg(feature = "splade")]` gate on `pub mod splade;` at lib.rs line
25, and by the `#![cfg(feature = "splade")]` crate-attribute on both
tests/splade_tests.rs line 16 and tests/splade_measure_tests.rs line 8.

=====================================================================
RECOMMENDED ACTIONS (not performed by auditor)
=====================================================================

1. Stage model fixtures at .elai_cc/models/splade/model.onnx and
   tokenizer.json (prithivida/Splade_PP_en_v1 ONNX main, 532MB) — this
   is a pre-session prerequisite per spec 4b PRE_GATE and cannot be
   done inside the implementation session.
2. Run `cargo test --features splade -p elai-knowledge
   splade_measure_tests` and capture eprintln output.
3. Write workflow/reasoning/phase-1H-decisions.md recording:
   - vocab_coverage_code numeric value,
   - model comparison table (p50 latency, Jaccard, quick recall@10),
   - recall delta mean/std, Wilcoxon p-value,
   - EARLY DROP / DROP / KEEP and full rationale.
4. Write .elai_cc/metrics/phase_1H.toml with the full MEASUREMENTS
   block values.
5. Only after (1)-(4) can this phase be declared COMPLETE.
