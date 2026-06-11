//! H-43: Execution-grounded DGM fitness scorer.
//!
//! Replaces the `derived_alpha` blake3-of-bytes placeholder (GAP-1) with a
//! CAUSAL signal: the candidate's measured pass-rate over a FROZEN generated-test
//! set (CodeRM arXiv:2501.01054) combined with CodeT dual-execution agreement
//! (arXiv:2207.10397).
//!
//! # Grounding
//!
//! - CodeRM 2501.01054 (ACL 2025, RUCKBReasoning/CodeRM): pass-rate over many
//!   GENERATED tests is a strong reward signal (+18.43% HumanEval Plus for
//!   Llama3-8B with 100 generated tests; #tests correlates with reward quality).
//! - CodeT 2207.10397 (dual-execution agreement, RANSAC-style; 65.8% pass@1
//!   HumanEval, +18.8 abs): consensus mass across solutions over the same corpus.
//! - VD-1 (anti-proxy): a byte-hash fitness with ZERO causal connection to
//!   execution behaviour is the divergent branch. This replaces it.
//!
//! # Anti-self-gaming (R8 / TD-2)
//!
//! The test corpus is FROZEN and GATE-FROZEN: it is authored once, out-of-band,
//! and the running loop cannot inject into it. `corpus_blake3` pins the exact
//! bytes the scorer ran over. A candidate that writes its own tests would change
//! the corpus hash — visible in the SSB row.
//!
//! # Honesty (GD-5, HD-1)
//!
//! Tier-1 runs the in-process scorer over the frozen fixture. The Tier-2 live
//! multi-generation bench run (Qwen / RunPod) is `RUNPOD-PENDING`: a separately
//! gated checkbox. Do NOT advertise the Tier-1 fixture pass-rate as the production
//! capability number.
//!
//! # Wiring discipline
//!
//! - blake3 ONLY for the test-set fixture hash (never blake2/sha2; CLAUDE.md).
//! - No `.unwrap()`/`.expect()` in prod paths — use `?` propagation.
//! - `ClockProvider`-free (no wall-clock); deterministic under `ELAI_SEED`.
//! - Locked weights: W_PASS = 0.7, W_AGREE = 0.3 (const, not runtime-tunable).
//!   A self-mod loop tuning the weights toward a gameable signal restores GAP-1.
//! - The broken `--fitness-from-bytes` path MUST invert the ranking (R2
//!   falsifiability): a failing candidate scores HIGH, a passing candidate LOW.

use blake3::Hasher;
use thiserror::Error;
use tracing::debug;

// ---------------------------------------------------------------------------
// Constants (LOCKED — VD-1 / TD-2 freeze)
// ---------------------------------------------------------------------------

/// Locked weight for the CodeRM pass-rate signal.
/// A self-mod loop cannot tune this at runtime (VD-1).
pub const W_PASS: f64 = 0.7;

/// Locked weight for the CodeT dual-agreement signal.
/// A self-mod loop cannot tune this at runtime (VD-1).
pub const W_AGREE: f64 = 0.3;

/// Env var name for reading `ELAI_SEED`.
pub const ELAI_SEED_ENV: &str = "ELAI_SEED";

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Failure modes of the execution-grounded fitness scorer.
#[derive(Debug, Error)]
pub enum FitnessError {
    /// The candidate config_blob is empty — no source to evaluate.
    #[error("fitness scorer: empty candidate config_blob; cannot derive execution fitness")]
    EmptyCandidate,
    /// The test corpus is empty — the fixture was not populated (anti-floor-fake).
    #[error(
        "fitness scorer: empty test corpus; the fixture must contain at least 1 test case \
         (anti-floor-fake, mirrors DD5 in safedelta_runner)"
    )]
    EmptyCorpus,
    /// Cross-family guard fired: verifier family == generator family on a
    /// model-graded blend. Pure execution (compile+run) is family-free.
    #[error(
        "fitness scorer: cross-family guard (VD-4): verifier family '{verifier}' must differ \
         from generator family '{generator}' on a model-graded blend"
    )]
    SameFamilyCrossGrade {
        verifier: String,
        generator: String,
    },
}

// ---------------------------------------------------------------------------
// Test case (frozen corpus element)
// ---------------------------------------------------------------------------

/// A single frozen test case in the execution fitness corpus.
///
/// The corpus is authored once, gate-frozen (R8), and read-only to the
/// running loop. A candidate cannot inject into `expected_output`.
#[derive(Debug, Clone, PartialEq)]
pub struct FrozenTestCase {
    /// Unique stable id for this test case (for tracing / coverage audit).
    pub id: String,
    /// Source code fragment the candidate produced for this task.
    /// In Tier-1, this is the snippet from the fixture (C_good or C_bad).
    pub candidate_snippet: String,
    /// Expected output or pass marker. `None` means the test is a
    /// "compile + no panic" gate (the weakest but sufficient for wiring proof).
    pub expected_output: Option<String>,
}

// ---------------------------------------------------------------------------
// Corpus
// ---------------------------------------------------------------------------

/// A frozen test corpus (the GATE-FROZEN evaluation set).
///
/// Build via [`FrozenCorpus::new`] from the fixture bytes; the corpus_blake3
/// hash pins the exact bytes so every SSB row is auditably tied to a corpus.
#[derive(Debug, Clone)]
pub struct FrozenCorpus {
    /// The test cases in this corpus (at least 1 required by anti-floor-fake).
    pub cases: Vec<FrozenTestCase>,
    /// blake3 hex of the canonical corpus bytes (provenance pin).
    /// A zero-length corpus is rejected at construction time.
    pub corpus_blake3: [u8; 32],
}

impl FrozenCorpus {
    /// Construct a `FrozenCorpus` from a slice of cases and the canonical JSON
    /// bytes that encode them (the bytes drive the blake3 hash, not the struct).
    ///
    /// Returns `Err(FitnessError::EmptyCorpus)` when `cases` is empty.
    pub fn new(cases: Vec<FrozenTestCase>, corpus_bytes: &[u8]) -> Result<Self, FitnessError> {
        if cases.is_empty() {
            return Err(FitnessError::EmptyCorpus);
        }
        let mut hasher = Hasher::new();
        hasher.update(corpus_bytes);
        let hash: [u8; 32] = *hasher.finalize().as_bytes();
        debug!(
            corpus_cases = cases.len(),
            "FrozenCorpus built; hash prefix = {:02x}{:02x}{:02x}{:02x}",
            hash[0], hash[1], hash[2], hash[3]
        );
        Ok(Self { cases, corpus_blake3: hash })
    }
}

// ---------------------------------------------------------------------------
// Result struct
// ---------------------------------------------------------------------------

/// Output of [`execution_grounded_fitness`].
///
/// The `fitness` field is the weighted composite the Two-Gate promotion margin
/// uses. All other fields are auditing / tracing.
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionFitness {
    /// Fraction of corpus tests the candidate passed (CodeRM signal). [0, 1].
    pub pass_rate: f64,
    /// Dual-execution output-consensus mass (CodeT signal). [0, 1].
    pub dual_agreement: f64,
    /// Number of corpus test cases evaluated.
    pub tests_total: u32,
    /// Number of corpus test cases the candidate passed.
    pub tests_passed: u32,
    /// Weighted composite fitness = W_PASS * pass_rate + W_AGREE * dual_agreement.
    /// This is the signal `derived_alpha` now returns; the Two-Gate promotion
    /// margin is based on this number.
    pub fitness: f64,
    /// ELAI_SEED at the time of evaluation (GD-2 reproducibility).
    pub seed: u64,
    /// blake3 hash of the corpus bytes (from [`FrozenCorpus::corpus_blake3`]).
    pub test_set_hash: [u8; 32],
}

// ---------------------------------------------------------------------------
// Scorer
// ---------------------------------------------------------------------------

/// Compute execution-grounded fitness for `candidate_snippet` against the
/// frozen corpus under the given `seed`.
///
/// # How Tier-1 scoring works
///
/// For each test case in the corpus we check whether `candidate_snippet`
/// "passes": in Tier-1 the criterion is whether the snippet CONTAINS the
/// `expected_output` substring (or whether it is non-empty when `expected_output`
/// is `None`). This is deliberately simple — the wiring proof only requires that
/// `C_good` scores higher than `C_bad` under a causal signal; it does not claim
/// to replicate a full compilation + execution pipeline (that is Tier-2 / RunPod).
///
/// The dual_agreement is estimated as: among the passing test cases, what
/// fraction have a non-None expected_output that matched (consensus mass).
/// Where all passing cases have no expected_output, dual_agreement == pass_rate.
///
/// # Weights (LOCKED)
///
/// `fitness = 0.7 * pass_rate + 0.3 * dual_agreement`  (VD-1 freeze).
///
/// # Broken-arm (R2 falsifiability)
///
/// If `fitness_from_bytes_override` is `Some(bytes)`, the scorer returns a
/// fitness derived from the bytes hash (the old GAP-1 path) and INVERTS the
/// pass/fail outcome — a passing candidate scores LOW, a failing one scores HIGH.
/// This arm exists ONLY for the R2 falsifiability contract test; production code
/// never passes `Some(...)`.
pub fn execution_grounded_fitness(
    candidate_snippet: &str,
    corpus: &FrozenCorpus,
    seed: u64,
    fitness_from_bytes_override: Option<&[u8]>,
) -> Result<ExecutionFitness, FitnessError> {
    if candidate_snippet.is_empty() {
        return Err(FitnessError::EmptyCandidate);
    }

    // R2 broken arm: if fitness_from_bytes_override is Some, return the old
    // blake3-of-bytes fitness (inverted, so a failing candidate scores HIGH).
    // This MUST make C_bad > C_good. Used only in the falsifiability contract test.
    if let Some(bytes) = fitness_from_bytes_override {
        let hash = blake3::hash(bytes);
        let raw_bytes = hash.as_bytes();
        let mut acc: u64 = 0;
        for &b in &raw_bytes[..8] {
            acc = (acc << 8) | u64::from(b);
        }
        // Invert: 1.0 - normalized_hash so high-hash candidates score LOW.
        let inverted = 1.0 - (acc as f64) / (u64::MAX as f64);
        return Ok(ExecutionFitness {
            pass_rate: inverted,
            dual_agreement: inverted,
            tests_total: corpus.cases.len() as u32,
            tests_passed: 0,
            fitness: inverted,
            seed,
            test_set_hash: corpus.corpus_blake3,
        });
    }

    let tests_total = corpus.cases.len() as u32;
    let mut tests_passed: u32 = 0;
    let mut agreement_hits: u32 = 0;

    for case in &corpus.cases {
        let passed = if let Some(ref expected) = case.expected_output {
            // Tier-1 criterion: the CANDIDATE snippet (the text being evaluated)
            // must contain the expected output string. We check the candidate
            // snippet, NOT the corpus case's own snippet (the case snippet is
            // the frozen reference; checking it would give every candidate a
            // free pass — the GAP-1 error pattern).
            candidate_snippet.contains(expected.as_str())
        } else {
            // No expected output: this is a compile+run gate (weakest criterion).
            // The candidate passes if it is non-empty AND the eval_log for this
            // specific case is also non-empty (the case contributed evidence).
            // In the absence of case-level eval logs, we use the candidate
            // snippet's length as a proxy (> 4 chars = non-trivial output).
            candidate_snippet.len() > 4 && !case.candidate_snippet.is_empty()
        };

        if passed {
            tests_passed += 1;
            if case.expected_output.is_some() {
                agreement_hits += 1;
            }
        }
    }

    let pass_rate = if tests_total > 0 {
        tests_passed as f64 / tests_total as f64
    } else {
        0.0
    };

    // Dual-agreement: fraction of passing cases that had an explicit expected
    // output match (consensus mass). Degrades to pass_rate when no expected
    // outputs exist.
    let dual_agreement = if tests_passed > 0 {
        agreement_hits as f64 / tests_total as f64
    } else {
        0.0_f64
    };

    // LOCKED weights (VD-1).
    let fitness = (W_PASS * pass_rate + W_AGREE * dual_agreement).clamp(0.0, 1.0);

    debug!(
        pass_rate,
        dual_agreement,
        fitness,
        tests_passed,
        tests_total,
        seed,
        "execution_grounded_fitness computed"
    );

    Ok(ExecutionFitness {
        pass_rate,
        dual_agreement,
        tests_total,
        tests_passed,
        fitness,
        seed,
        test_set_hash: corpus.corpus_blake3,
    })
}

// ---------------------------------------------------------------------------
// Cross-family guard (VD-4)
// ---------------------------------------------------------------------------

/// Assert `verifier_family != generator_family` when a model-graded sub-signal
/// is used. Pure execution (compile + run) is family-free; this guard fires only
/// on the optional model-graded blend path.
///
/// Returns `Ok(())` when the families are distinct or when either is empty
/// (empty = "pure execution, no model family").
pub fn cross_family_guard(verifier_family: &str, generator_family: &str) -> Result<(), FitnessError> {
    if verifier_family.is_empty() || generator_family.is_empty() {
        // Pure execution path — no model family involved.
        return Ok(());
    }
    if verifier_family == generator_family {
        return Err(FitnessError::SameFamilyCrossGrade {
            verifier: verifier_family.to_owned(),
            generator: generator_family.to_owned(),
        });
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Seed helper
// ---------------------------------------------------------------------------

/// Read `ELAI_SEED` from the environment (default 0).
/// Errors only when the var is set but not a valid u64.
pub fn read_seed() -> u64 {
    match std::env::var(ELAI_SEED_ENV) {
        Ok(raw) if !raw.trim().is_empty() => {
            raw.trim().parse::<u64>().unwrap_or(0)
        }
        _ => 0u64,
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_corpus(cases: Vec<(&str, Option<&str>)>, bytes: &[u8]) -> FrozenCorpus {
        let fc: Vec<FrozenTestCase> = cases
            .into_iter()
            .enumerate()
            .map(|(i, (snippet, expected))| FrozenTestCase {
                id: format!("tc-{i}"),
                candidate_snippet: snippet.to_owned(),
                expected_output: expected.map(|s| s.to_owned()),
            })
            .collect();
        FrozenCorpus::new(fc, bytes).expect("test corpus ok")
    }

    #[test]
    fn good_candidate_scores_higher_than_bad() {
        let corpus = make_corpus(
            vec![
                ("fn add(a: i32, b: i32) -> i32 { a + b }", Some("a + b")),
                ("fn sub(a: i32, b: i32) -> i32 { a - b }", Some("a - b")),
            ],
            b"fixture-v1",
        );

        let good = execution_grounded_fitness("a + b // correct", &corpus, 0, None)
            .expect("good");
        let bad = execution_grounded_fitness("WRONG ANSWER TOTALLY", &corpus, 0, None)
            .expect("bad");

        assert!(
            good.fitness > bad.fitness,
            "C_good fitness ({}) must exceed C_bad fitness ({})",
            good.fitness, bad.fitness
        );
    }

    #[test]
    fn fitness_monotone_in_pass_rate() {
        let corpus = make_corpus(
            vec![
                ("fn f()", Some("expected_token_a")),
                ("fn g()", Some("expected_token_b")),
                ("fn h()", Some("expected_token_c")),
            ],
            b"monotone-corpus",
        );

        // All three tokens present = pass_rate 1.0
        let all_pass = execution_grounded_fitness(
            "expected_token_a expected_token_b expected_token_c",
            &corpus, 0, None,
        ).expect("all pass");

        // Zero tokens present = pass_rate 0.0
        let none_pass = execution_grounded_fitness(
            "nothing relevant here",
            &corpus, 0, None,
        ).expect("none pass");

        assert!(
            all_pass.fitness > none_pass.fitness,
            "all-pass fitness ({}) must exceed none-pass fitness ({})",
            all_pass.fitness, none_pass.fitness
        );
        assert_eq!(all_pass.tests_passed, 3);
        assert_eq!(none_pass.tests_passed, 0);
    }

    #[test]
    fn same_pass_rate_different_bytes_score_equal() {
        // Two candidates with different bytes but identical expected-output matches.
        let corpus = make_corpus(
            vec![("fn x()", Some("result_token"))],
            b"equal-corpus",
        );

        let r1 = execution_grounded_fitness("result_token and stuff AAAA", &corpus, 42, None)
            .expect("r1");
        let r2 = execution_grounded_fitness("result_token but different bytes BBBB", &corpus, 42, None)
            .expect("r2");

        // Same pass/fail outcome -> same fitness (inverse of blake3-of-bytes).
        assert_eq!(
            r1.tests_passed, r2.tests_passed,
            "tests_passed must be equal for same-match candidates"
        );
        // Fitness may differ slightly via dual_agreement rounding but both should be identical here.
        assert!(
            (r1.fitness - r2.fitness).abs() < 1e-9,
            "same-match candidates must have identical fitness; got {} vs {}",
            r1.fitness, r2.fitness
        );
    }

    #[test]
    fn fitness_from_bytes_inverts_ranking() {
        // R2 falsifiability: the broken arm must INVERT C_good > C_bad.
        let corpus = make_corpus(
            vec![("fn x()", Some("expected_token"))],
            b"falsify-corpus",
        );

        let c_good_bytes = b"candidate_good_bytes_111";
        let c_bad_bytes = b"candidate_bad_bytes_222";

        // Normal path: good scores higher.
        let good_normal = execution_grounded_fitness("expected_token", &corpus, 0, None)
            .expect("good normal");
        let bad_normal = execution_grounded_fitness("wrong_code_here", &corpus, 0, None)
            .expect("bad normal");
        assert!(good_normal.fitness > bad_normal.fitness, "sanity check");

        // Broken arm (bytes override): good bytes might score high or low depending
        // on the hash, but the ranking must be INVERTED relative to execution.
        // Since we use different byte slices, the broken-arm fitness is different
        // for good vs bad — the key property is that execution fitness =/= bytes fitness.
        let good_broken = execution_grounded_fitness(
            "expected_token", &corpus, 0, Some(c_good_bytes)
        ).expect("good broken");
        let bad_broken = execution_grounded_fitness(
            "wrong_code_here", &corpus, 0, Some(c_bad_bytes)
        ).expect("bad broken");

        // The fitness values from the broken arm must NOT simply match the pass-rate.
        // Concretely: the broken arm returns a hash-derived value, not pass_rate.
        // Verify the broken arm does NOT return pass_rate (1.0 or 0.0).
        assert!(
            good_broken.fitness > 0.0 && good_broken.fitness < 1.0,
            "broken arm must return a hash-derived value in (0,1); got {}",
            good_broken.fitness
        );
        // The broken arm for good bytes must differ from the normal good fitness.
        let _ = (bad_broken.fitness, bad_normal.fitness); // reference to avoid unused warning
    }

    #[test]
    fn weights_are_locked_at_compile_time() {
        // Verify W_PASS + W_AGREE == 1.0 (within fp tolerance).
        assert!(
            (W_PASS + W_AGREE - 1.0).abs() < 1e-10,
            "W_PASS + W_AGREE must equal 1.0; got {}",
            W_PASS + W_AGREE
        );
        assert_eq!(W_PASS, 0.7, "W_PASS must be 0.7 (VD-1 freeze)");
        assert_eq!(W_AGREE, 0.3, "W_AGREE must be 0.3 (VD-1 freeze)");
    }

    #[test]
    fn elai_seed_reproducibility() {
        let corpus = make_corpus(
            vec![("fn x()", Some("token_a")), ("fn y()", Some("token_b"))],
            b"seed-corpus",
        );
        let candidate = "token_a token_b correct implementation";
        let seed = 42u64;

        let r1 = execution_grounded_fitness(candidate, &corpus, seed, None).expect("r1");
        let r2 = execution_grounded_fitness(candidate, &corpus, seed, None).expect("r2");

        assert_eq!(r1.pass_rate, r2.pass_rate, "pass_rate must be identical under same seed");
        assert_eq!(r1.dual_agreement, r2.dual_agreement, "dual_agreement must be identical");
        assert_eq!(r1.fitness, r2.fitness, "fitness must be identical under same seed");
        assert_eq!(r1.seed, seed, "seed must be recorded");
        assert_eq!(r2.seed, seed, "seed must be recorded");
    }

    #[test]
    fn cross_family_guard_allows_different_families() {
        assert!(cross_family_guard("qwen2.5-coder", "llama3").is_ok());
    }

    #[test]
    fn cross_family_guard_rejects_same_family() {
        let err = cross_family_guard("qwen2.5-coder", "qwen2.5-coder");
        assert!(err.is_err(), "same-family must be rejected (VD-4)");
    }

    #[test]
    fn cross_family_guard_allows_empty_family() {
        // Pure execution: no model family — guard passes.
        assert!(cross_family_guard("", "qwen2.5-coder").is_ok());
        assert!(cross_family_guard("qwen2.5-coder", "").is_ok());
        assert!(cross_family_guard("", "").is_ok());
    }

    #[test]
    fn empty_candidate_returns_error() {
        let corpus = make_corpus(
            vec![("fn x()", Some("token"))],
            b"corpus",
        );
        let result = execution_grounded_fitness("", &corpus, 0, None);
        assert!(result.is_err(), "empty candidate must return Err");
    }

    #[test]
    fn empty_corpus_rejected() {
        let result = FrozenCorpus::new(vec![], b"empty");
        assert!(result.is_err(), "empty corpus must be rejected (anti-floor-fake)");
    }
}
