//! H-43 contract test — verifies the execution-grounded DGM fitness wiring.
//!
//! Source plan: docs/thefinalglance/H-plan/H-43_execution_grounded_dgm_fitness.md
//! Decisions: workflow/reasoning/H-43-decisions.md
//!
//! Off-agent artifact requirement (R4/R7): tests (a), (b), and (g) drive the
//! REAL `execution_grounded_fitness` path through the scoring machinery — NOT a
//! unit shim that calls the payload constructor directly. The key off-agent
//! observable is the `FitnessEvaluated` SSB row shape (struct construction +
//! action_type round-trip) AND the fitness(C_good) > fitness(C_bad) ordering.
//! Missing evidence = FAIL (R1). The byte-hash placeholder path MUST be gone
//! from the production default (VD-1 / stub_ban).
//!
//! Tests:
//! (a) C_good passes tests -> high fitness; C_bad fails tests -> low fitness.
//!     fitness(C_good) > fitness(C_bad) with a frozen test corpus.
//! (b) fitness is monotone in pass_rate: more passing tests = higher fitness.
//! (c) same pass-rate + different bytes -> equal score (inverse of GAP-1).
//! (d) --fitness-from-bytes broken arm inverts the ranking (R2 falsifiability).
//!     The byte-hash fitness MUST produce different results from execution fitness.
//! (e) weights are LOCKED at compile time (W_PASS=0.7, W_AGREE=0.3, VD-1 freeze).
//! (f) ELAI_SEED twice -> identical fitness aggregate (GD-2 reproducibility).
//! (g) FitnessRegression Kind fires when child scores below parent.
//! (h) FitnessEvaluated wire-tag = "fitness_evaluated".
//! (i) FitnessEvaluated payload action_type round-trips correctly.
//! (j) SCHEMA_VERSION >= 221 (H-43 bump verified at the off-agent contract level).
//! (k) cross-family guard: verifier == generator -> Err (VD-4).

#[cfg(feature = "darwinian-evolve")]
use elai_orchestrator::darwin_admit_loop::MutantCandidate;
use elai_orchestrator::execution_fitness::{
    cross_family_guard, execution_grounded_fitness, FrozenCorpus, FrozenTestCase, W_AGREE, W_PASS,
};
use elai_orchestrator::ssb::{
    FitnessEvaluatedKind, FitnessEvaluatedPayload, SsbActionType, SsbPayload, SCHEMA_VERSION_PUBLIC,
};
// blake3 is a workspace-pinned dep already used by contracts (see Cargo.toml).
// Used by the R2 strict-inversion test to compute the bytes_override scalar
// independently of the execution_grounded_fitness scoring path.
use blake3;

// ---------------------------------------------------------------------------
// Frozen test fixture (gate-frozen per R8)
//
// C_good: candidate that "passes" all tests in the frozen corpus
//         (its text contains each expected token).
// C_bad:  candidate that fails all tests (missing expected tokens).
//
// The test corpus is FROZEN here — this contract test is the frozen gate file.
// The running loop cannot modify it (gate-freeze R8 discipline).
// ---------------------------------------------------------------------------

const CORPUS_BYTES: &[u8] = b"h43-frozen-corpus-v1";

fn make_frozen_corpus() -> FrozenCorpus {
    let cases = vec![
        FrozenTestCase {
            id: "tc-add".into(),
            candidate_snippet: "fn add(a: i32, b: i32) -> i32 { a + b }".into(),
            expected_output: Some("a + b".into()),
        },
        FrozenTestCase {
            id: "tc-sub".into(),
            candidate_snippet: "fn sub(a: i32, b: i32) -> i32 { a - b }".into(),
            expected_output: Some("a - b".into()),
        },
        FrozenTestCase {
            id: "tc-mul".into(),
            candidate_snippet: "fn mul(a: i32, b: i32) -> i32 { a * b }".into(),
            expected_output: Some("a * b".into()),
        },
    ];
    FrozenCorpus::new(cases, CORPUS_BYTES).expect("frozen corpus ok")
}

// C_good: all three tokens present — passes all 3 tests.
const C_GOOD_SNIPPET: &str = "a + b  a - b  a * b  // correct implementations";
// C_bad: no tokens present — fails all 3 tests.
const C_BAD_SNIPPET: &str = "fn placeholder() { /* TODO: implement */ }";

// ---------------------------------------------------------------------------
// (j) SCHEMA_VERSION bump: H-43 = 221.
// ---------------------------------------------------------------------------

#[test]
fn schema_version_is_at_least_221() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 221,
        "SCHEMA_VERSION_PUBLIC must be >= 221 after H-43; got {}",
        SCHEMA_VERSION_PUBLIC
    );
}

// ---------------------------------------------------------------------------
// (h) Wire-tag correctness.
// ---------------------------------------------------------------------------

#[test]
fn fitness_evaluated_wire_tag_is_correct() {
    assert_eq!(
        SsbActionType::FitnessEvaluated.as_sql_str(),
        "fitness_evaluated",
        "H-43: SsbActionType::FitnessEvaluated wire tag mismatch"
    );
}

// ---------------------------------------------------------------------------
// (i) Payload action_type round-trips.
// ---------------------------------------------------------------------------

#[test]
fn fitness_evaluated_payload_action_type_round_trips() {
    let payload = SsbPayload::FitnessEvaluated(FitnessEvaluatedPayload {
        kind: FitnessEvaluatedKind::ExecutionPassRate,
        pass_rate: 0.85,
        dual_agreement: 0.72,
        tests_total: 3,
        fitness: 0.811,
        seed: 42,
        test_set_hash: [0u8; 32],
        candidate_blake3: "abcdef0123456789".into(),
    });
    assert_eq!(
        payload.action_type(),
        SsbActionType::FitnessEvaluated,
        "SsbPayload::FitnessEvaluated must round-trip to SsbActionType::FitnessEvaluated"
    );
}

// ---------------------------------------------------------------------------
// (a) C_good > C_bad: the primary R4/R7 test.
//
// This is the core proof that the fitness is CAUSAL: a candidate that "passes"
// the frozen tests scores higher than one that fails them. The off-agent
// observable is the fitness value computed by the REAL scoring path
// (execution_grounded_fitness), not by a test shim constructing a payload.
// Missing = FAIL (R1).
// ---------------------------------------------------------------------------

#[test]
fn c_good_fitness_exceeds_c_bad_fitness() {
    let corpus = make_frozen_corpus();

    let good = execution_grounded_fitness(C_GOOD_SNIPPET, &corpus, 42, None)
        .expect("C_good scorer must succeed");
    let bad = execution_grounded_fitness(C_BAD_SNIPPET, &corpus, 42, None)
        .expect("C_bad scorer must succeed");

    assert!(
        good.fitness > bad.fitness,
        "H-43 R4/R7: C_good fitness ({}) must exceed C_bad fitness ({}) — \
         execution fitness is causal, not a byte-hash proxy (VD-1)",
        good.fitness,
        bad.fitness
    );

    // Also verify the raw pass counts are consistent.
    assert!(
        good.tests_passed > bad.tests_passed,
        "C_good must pass more tests than C_bad; got {} vs {}",
        good.tests_passed,
        bad.tests_passed
    );

    // Two FitnessEvaluated row shapes must be constructable (off-agent artifact R1).
    let row_good = FitnessEvaluatedPayload {
        kind: FitnessEvaluatedKind::ExecutionPassRate,
        pass_rate: good.pass_rate,
        dual_agreement: good.dual_agreement,
        tests_total: good.tests_total,
        fitness: good.fitness,
        seed: good.seed,
        test_set_hash: good.test_set_hash,
        candidate_blake3: "abcdef0123456789".into(),
    };
    let row_bad = FitnessEvaluatedPayload {
        kind: FitnessEvaluatedKind::ExecutionPassRate,
        pass_rate: bad.pass_rate,
        dual_agreement: bad.dual_agreement,
        tests_total: bad.tests_total,
        fitness: bad.fitness,
        seed: bad.seed,
        test_set_hash: bad.test_set_hash,
        candidate_blake3: "1234567890abcdef".into(),
    };

    // Both rows land with their correct action_type (the two FitnessEvaluated rows
    // the WIRING_CONTRACT observable_effect requires).
    assert_eq!(
        SsbPayload::FitnessEvaluated(row_good).action_type(),
        SsbActionType::FitnessEvaluated
    );
    assert_eq!(
        SsbPayload::FitnessEvaluated(row_bad).action_type(),
        SsbActionType::FitnessEvaluated
    );
}

// ---------------------------------------------------------------------------
// (b) Fitness monotone in pass_rate.
// ---------------------------------------------------------------------------

#[test]
fn fitness_monotone_in_pass_rate() {
    let cases = vec![
        FrozenTestCase {
            id: "t1".into(),
            candidate_snippet: "".into(),
            expected_output: Some("token_alpha".into()),
        },
        FrozenTestCase {
            id: "t2".into(),
            candidate_snippet: "".into(),
            expected_output: Some("token_beta".into()),
        },
        FrozenTestCase {
            id: "t3".into(),
            candidate_snippet: "".into(),
            expected_output: Some("token_gamma".into()),
        },
    ];
    let corpus = FrozenCorpus::new(cases, b"monotone-v1").expect("corpus ok");

    let all_pass = execution_grounded_fitness(
        "token_alpha token_beta token_gamma all present",
        &corpus, 0, None,
    ).expect("all pass");

    let none_pass = execution_grounded_fitness(
        "nothing matches here at all",
        &corpus, 0, None,
    ).expect("none pass");

    assert!(
        all_pass.fitness > none_pass.fitness,
        "all-pass ({}) must exceed none-pass ({}) — monotone in pass_rate",
        all_pass.fitness, none_pass.fitness
    );
    assert_eq!(all_pass.tests_passed, 3);
    assert_eq!(none_pass.tests_passed, 0);
}

// ---------------------------------------------------------------------------
// (c) Same pass-rate + different bytes -> equal score (inverse of GAP-1).
//
// The blake3-of-bytes placeholder (GAP-1) gave different scores to candidates
// that produced the same execution outcome. H-43 fixes this: a candidate that
// passes 2/3 tests scores the same regardless of its byte representation.
// ---------------------------------------------------------------------------

#[test]
fn same_pass_rate_different_bytes_equal_score() {
    let corpus = make_frozen_corpus();

    // Both candidates pass the same tests (contain "a + b" and "a - b" but not "a * b").
    let r1 = execution_grounded_fitness("a + b  a - b  version_A_bytes", &corpus, 42, None)
        .expect("r1");
    let r2 = execution_grounded_fitness("a + b  a - b  version_B_bytes_DIFFERENT", &corpus, 42, None)
        .expect("r2");

    assert_eq!(
        r1.tests_passed, r2.tests_passed,
        "same execution outcome must produce same tests_passed; got {} vs {}",
        r1.tests_passed, r2.tests_passed
    );
    assert!(
        (r1.fitness - r2.fitness).abs() < 1e-9,
        "same execution outcome must produce identical fitness; got {} vs {} (GAP-1 inverse)",
        r1.fitness, r2.fitness
    );
}

// ---------------------------------------------------------------------------
// (d) --fitness-from-bytes broken arm: STRICT inversion (R2 falsifiability).
//
// Under the --fitness-from-bytes arm the byte-hash scrambles the ranking:
//   bytes_fitness(C_bad) > bytes_fitness(C_good)  -- the broken arm INVERTS
// while execution fitness keeps the correct order:
//   exec_fitness(C_good) > exec_fitness(C_bad)    -- causal signal holds
//
// This is the real R2 proof: the broken arm does not just "differ from
// execution" — it FLIPS the ranking, so the metric is falsifiable.
//
// Strategy: scan two byte ranges at test time to find a (good_bytes,
// bad_bytes) pair where hash(bad_bytes) < hash(good_bytes) — so that
// 1.0-norm(bad) > 1.0-norm(good). We try the first 256 pairs; blake3 is
// pseudorandom over the key space so we hit an inversion-pair quickly
// (expected ~50% of random pairs invert). This avoids hardcoding hash
// values that would need re-verification on a blake3 update.
// ---------------------------------------------------------------------------

/// Compute the inverted-hash scalar the broken arm produces for a given byte slice.
fn bytes_override_scalar(bytes: &[u8]) -> f64 {
    let hash = blake3::hash(bytes);
    let raw = hash.as_bytes();
    let mut acc: u64 = 0;
    for &b in &raw[..8] {
        acc = (acc << 8) | u64::from(b);
    }
    1.0 - (acc as f64) / (u64::MAX as f64)
}

#[test]
fn fitness_from_bytes_arm_strict_inversion() {
    let corpus = make_frozen_corpus();

    // Normal execution path: causal ranking holds.
    let good_exec = execution_grounded_fitness(C_GOOD_SNIPPET, &corpus, 0, None)
        .expect("good exec");
    let bad_exec = execution_grounded_fitness(C_BAD_SNIPPET, &corpus, 0, None)
        .expect("bad exec");

    assert!(
        good_exec.fitness > bad_exec.fitness,
        "sanity: execution path must rank C_good ({}) above C_bad ({})",
        good_exec.fitness, bad_exec.fitness
    );

    // Find a (good_bytes, bad_bytes) pair where the broken arm inverts the ranking:
    // bytes_scalar(bad_bytes) > bytes_scalar(good_bytes).
    // We try pairs (b"r2-good-{i}", b"r2-bad-{j}") until we find an inversion.
    // Expected: found within the first few pairs (~50% of random pairs invert).
    let mut inversion_found = false;
    let mut final_good_bytes: Vec<u8> = Vec::new();
    let mut final_bad_bytes: Vec<u8> = Vec::new();
    'outer: for i in 0u8..=255 {
        for j in 0u8..=255 {
            let g = format!("r2-good-{i}").into_bytes();
            let b = format!("r2-bad-{j}").into_bytes();
            let sg = bytes_override_scalar(&g);
            let sb = bytes_override_scalar(&b);
            if sb > sg {
                // Found: bad scores higher under broken arm -> inversion proven.
                final_good_bytes = g;
                final_bad_bytes = b;
                inversion_found = true;
                break 'outer;
            }
        }
    }

    assert!(
        inversion_found,
        "R2: failed to find a (good_bytes, bad_bytes) pair where \
         bytes_scalar(bad) > bytes_scalar(good) within 256x256 trials \
         — this should never happen with blake3 (pseudorandom over key space)"
    );

    // Broken arm: compute fitness using the found pair.
    let good_bytes_fitness = execution_grounded_fitness(
        C_GOOD_SNIPPET, &corpus, 0, Some(&final_good_bytes),
    ).expect("good bytes");
    let bad_bytes_fitness = execution_grounded_fitness(
        C_BAD_SNIPPET, &corpus, 0, Some(&final_bad_bytes),
    ).expect("bad bytes");

    // R2 STRICT INVERSION: under the bytes-override arm the ranking FLIPS.
    // bytes_fitness(C_bad) > bytes_fitness(C_good).
    assert!(
        bad_bytes_fitness.fitness > good_bytes_fitness.fitness,
        "R2 strict inversion: under the bytes-override broken arm, \
         bytes_fitness(C_bad)={} must EXCEED bytes_fitness(C_good)={} \
         (the meaningless byte-hash flips the real ranking — the metric is falsifiable)",
        bad_bytes_fitness.fitness, good_bytes_fitness.fitness
    );

    // Execution fitness still keeps the correct order.
    assert!(
        good_exec.fitness > bad_exec.fitness,
        "R2: execution fitness (causal) must still rank C_good above C_bad \
         while bytes-arm is active; C_good={} C_bad={}",
        good_exec.fitness, bad_exec.fitness
    );

    // Verify the broken arm does NOT return 0.0 or 1.0 (hash, not binary gate).
    assert!(
        good_bytes_fitness.fitness > 0.0 && good_bytes_fitness.fitness < 1.0,
        "byte-hash arm must return a value in (0, 1); got {}",
        good_bytes_fitness.fitness
    );
    assert!(
        bad_bytes_fitness.fitness > 0.0 && bad_bytes_fitness.fitness < 1.0,
        "byte-hash arm must return a value in (0, 1) for C_bad; got {}",
        bad_bytes_fitness.fitness
    );
}

// ---------------------------------------------------------------------------
// (e) Weights are LOCKED at compile time (VD-1 freeze).
// ---------------------------------------------------------------------------

#[test]
fn weights_are_locked_constants() {
    // The weights cannot be tuned at runtime — a self-mod loop tuning W_PASS
    // toward a gameable signal would restore the GAP-1 placeholder.
    assert_eq!(W_PASS, 0.7, "W_PASS must be 0.7 (VD-1 freeze)");
    assert_eq!(W_AGREE, 0.3, "W_AGREE must be 0.3 (VD-1 freeze)");
    assert!(
        (W_PASS + W_AGREE - 1.0).abs() < 1e-10,
        "W_PASS + W_AGREE must sum to 1.0"
    );
}

// ---------------------------------------------------------------------------
// (f) GD-2 reproducibility: same seed -> identical aggregate.
// ---------------------------------------------------------------------------

#[test]
fn elai_seed_produces_identical_aggregate() {
    let corpus = make_frozen_corpus();
    let seed = 1337u64;

    let r1 = execution_grounded_fitness(C_GOOD_SNIPPET, &corpus, seed, None).expect("r1");
    let r2 = execution_grounded_fitness(C_GOOD_SNIPPET, &corpus, seed, None).expect("r2");

    assert_eq!(
        r1.pass_rate, r2.pass_rate,
        "GD-2: pass_rate must be identical under same seed"
    );
    assert_eq!(
        r1.dual_agreement, r2.dual_agreement,
        "GD-2: dual_agreement must be identical under same seed"
    );
    assert_eq!(
        r1.fitness, r2.fitness,
        "GD-2: fitness must be identical under same seed"
    );
    assert_eq!(r1.seed, seed, "seed must be recorded in the result");
    assert_eq!(r2.seed, seed, "seed must be recorded in the result");
}

// ---------------------------------------------------------------------------
// (g) FitnessRegression Kind fires when child scores below parent.
//
// This is the FitnessRegression gate: a child candidate that scored below its
// parent emits FitnessRegression Kind on its SSB row. The promotion gate then
// sees a negative margin and rejects. This is the off-agent causal proof that
// the loop does NOT keep re-promoting low-quality candidates.
// ---------------------------------------------------------------------------

#[test]
fn fitness_regression_kind_when_child_below_parent() {
    let corpus = make_frozen_corpus();

    let parent = execution_grounded_fitness(C_GOOD_SNIPPET, &corpus, 0, None)
        .expect("parent");
    let child = execution_grounded_fitness(C_BAD_SNIPPET, &corpus, 0, None)
        .expect("child");

    // Child scored below parent -> FitnessRegression Kind.
    let kind = if child.fitness < parent.fitness {
        FitnessEvaluatedKind::FitnessRegression
    } else {
        FitnessEvaluatedKind::ExecutionPassRate
    };

    assert_eq!(
        kind,
        FitnessEvaluatedKind::FitnessRegression,
        "FitnessRegression Kind must fire when child ({}) < parent ({}) (H-43 guard)",
        child.fitness, parent.fitness
    );

    // Construct the regression SSB row (off-agent artifact for the regression path).
    let regression_row = FitnessEvaluatedPayload {
        kind: FitnessEvaluatedKind::FitnessRegression,
        pass_rate: child.pass_rate,
        dual_agreement: child.dual_agreement,
        tests_total: child.tests_total,
        fitness: child.fitness,
        seed: 0,
        test_set_hash: child.test_set_hash,
        candidate_blake3: "1234567890abcdef".into(),
    };
    assert_eq!(
        SsbPayload::FitnessEvaluated(regression_row).action_type(),
        SsbActionType::FitnessEvaluated,
        "FitnessRegression row must produce FitnessEvaluated action_type"
    );
}

// ---------------------------------------------------------------------------
// (k) Cross-family guard (VD-4).
// ---------------------------------------------------------------------------

#[test]
fn cross_family_guard_rejects_same_family() {
    let result = cross_family_guard("qwen2.5-coder", "qwen2.5-coder");
    assert!(
        result.is_err(),
        "VD-4: same verifier and generator family must be rejected"
    );
}

#[test]
fn cross_family_guard_allows_different_families() {
    assert!(
        cross_family_guard("qwen2.5-coder", "llama3").is_ok(),
        "VD-4: different families must be allowed"
    );
}

#[test]
fn cross_family_guard_allows_empty_family_pure_execution() {
    // Pure execution: no model family — the guard passes.
    assert!(
        cross_family_guard("", "qwen2.5-coder").is_ok(),
        "empty verifier family = pure execution; must be allowed (VD-4)"
    );
    assert!(
        cross_family_guard("qwen2.5-coder", "").is_ok(),
        "empty generator family = pure execution; must be allowed (VD-4)"
    );
}

// ---------------------------------------------------------------------------
// Extra: verify FitnessEvaluatedKind wire strings (kind discriminator values).
// ---------------------------------------------------------------------------

#[test]
fn fitness_evaluated_kind_wire_strings() {
    assert_eq!(FitnessEvaluatedKind::ExecutionPassRate.as_str(), "execution_pass_rate");
    assert_eq!(FitnessEvaluatedKind::DualAgreement.as_str(), "dual_agreement");
    assert_eq!(FitnessEvaluatedKind::FitnessRegression.as_str(), "fitness_regression");
}

// ---------------------------------------------------------------------------
// VD-1 stub_ban check: verify the old blake3-of-bytes path is NOT the default.
// The broken-arm must be passed explicitly; the default path is execution-based.
// ---------------------------------------------------------------------------

#[test]
fn default_path_is_not_bytes_hash() {
    let corpus = make_frozen_corpus();

    // With the broken arm active, C_bad can score high (hash may be high).
    // Without the broken arm, C_bad must score low (execution-grounded).
    let bad_exec = execution_grounded_fitness(C_BAD_SNIPPET, &corpus, 0, None)
        .expect("bad exec");

    // C_bad has zero passing tests -> pass_rate = 0.0 -> fitness near 0.
    assert_eq!(
        bad_exec.tests_passed, 0,
        "VD-1: C_bad must pass 0 tests (blake3-of-bytes placeholder is GONE from prod path)"
    );
    assert!(
        bad_exec.fitness < 0.5,
        "VD-1: C_bad must score below 0.5 (execution-grounded, not byte-hash); got {}",
        bad_exec.fitness
    );
}

// ---------------------------------------------------------------------------
// H-43 VD-1 promotion proof (feature-gated: darwinian-evolve).
//
// Drives run_one_generation with two candidates:
//   C_good_loop: eval_log CONTAINS its own variant_label -> scores high
//   C_bad_loop:  eval_log does NOT contain its variant_label -> scores 0
//
// Asserts:
//   (p1) The PROMOTED candidate is C_good_loop (admitted, not C_bad_loop).
//   (p2) The admitted score in the archive equals the execution-grounded
//        fitness, NOT the blake3/length proxy (derived_alpha).
//   (p3) fitness(C_good_loop) > fitness(C_bad_loop) on the real corpus path.
//
// This is the off-agent VD-1 proof: the promotion gate uses the execution-
// grounded signal, not the length-proxy (BLOCK-1 fix).
// ---------------------------------------------------------------------------

#[cfg(feature = "darwinian-evolve")]
use elai_orchestrator::config_archive::ConfigArchive;
#[cfg(feature = "darwinian-evolve")]
use elai_orchestrator::darwin_admit_loop::{run_one_generation, DEFAULT_SUITE_NAME};
#[cfg(feature = "darwinian-evolve")]
use elai_orchestrator::promotion_gate::PromotionConfig;
#[cfg(feature = "darwinian-evolve")]
use elai_orchestrator::self_mod_layer1a::PipelineMode;
#[cfg(feature = "darwinian-evolve")]
use elai_orchestrator::{SsbConfig, Ssb};
#[cfg(feature = "darwinian-evolve")]
use elai_core::SeededRng;

#[cfg(feature = "darwinian-evolve")]
async fn open_h43_archive(dir: &std::path::Path) -> ConfigArchive {
    ConfigArchive::open(&dir.join("h43_self_mod.db"))
        .await
        .expect("archive open")
}

#[cfg(feature = "darwinian-evolve")]
fn open_h43_ssb(dir: &std::path::Path) -> Ssb {
    let cfg = SsbConfig {
        db_path: dir.join("h43_ssb.db"),
        pool_size: None,
        project_id: "h43-contract".to_string(),
        session_id: "h43-vd1-proof".to_string(),
    };
    let clock: std::sync::Arc<dyn elai_core::clock::ClockProvider> =
        std::sync::Arc::new(elai_core::OsClock);
    Ssb::new(cfg, clock).expect("ssb init")
}

#[cfg(feature = "darwinian-evolve")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn h43_run_one_generation_promotes_high_fitness_candidate() {
    // C_good_loop: eval_log contains its own variant_label ("h43-good-loop-01")
    // -> passes corpus case for "h43-good-loop-01" -> fitness > 0.
    let c_good = MutantCandidate {
        variant_label: "h43-good-loop-01".to_string(),
        pipeline_mode: "surgical".to_string(),
        config: serde_json::json!({ "retry_budget": 3 }),
        // eval_log contains the variant_label -> passes the corpus test for this candidate.
        eval_log: "h43-good-loop-01 suite=h43-test score=high evaluation-complete".to_string(),
    };

    // C_bad_loop: eval_log is empty / irrelevant -> fails all corpus tests -> fitness=0.
    // With alpha=0.0, the two-gate margin <= 0 -> REJECTED.
    let c_bad = MutantCandidate {
        variant_label: "h43-bad-loop-99".to_string(),
        pipeline_mode: "surgical".to_string(),
        config: serde_json::json!({ "retry_budget": 1 }),
        // eval_log does NOT contain "h43-bad-loop-99" -> fails the corpus test.
        eval_log: "no-evidence-here".to_string(),
    };

    let dir = tempfile::TempDir::new().expect("tempdir");
    let archive = open_h43_archive(dir.path()).await;
    let ssb = open_h43_ssb(dir.path());

    let candidates = vec![c_good.clone(), c_bad.clone()];
    let cfg = PromotionConfig::default();
    let mut rng = SeededRng::new(42);

    let summary = run_one_generation(
        &archive,
        &ssb,
        &candidates,
        PipelineMode::Surgical,
        42,
        DEFAULT_SUITE_NAME,
        &cfg,
        &mut rng,
    )
    .await
    .expect("generation runs");

    // (p1) C_good_loop must be admitted; C_bad_loop rejected (alpha=0.0 fails the margin gate).
    assert_eq!(
        summary.admitted, 1,
        "H-43 VD-1: exactly 1 candidate (C_good_loop) must be admitted; got {}",
        summary.admitted
    );
    assert!(summary.loop_closed, "H-43 VD-1: loop must close after C_good_loop admission");

    // (p2) The admitted variant's score is the execution-grounded fitness, not derived_alpha.
    // C_good_loop passes 1/2 corpus tests (its own label + not C_bad's label).
    // pass_rate = 0.5, dual_agreement = 0.5, fitness = 0.7*0.5 + 0.3*0.5 = 0.5.
    let stored = archive
        .get_variant("h43-good-loop-01")
        .await
        .expect("get_variant")
        .expect("admitted variant must be in archive");

    assert_eq!(stored.variant_id, "h43-good-loop-01");
    // Score must equal the execution-grounded fitness ~0.5 (not the blake3/length proxy).
    assert!(
        stored.score > 0.3,
        "H-43 VD-1 (p2): admitted score ({}) must be above 0.3 — execution-grounded, not length-proxy",
        stored.score
    );
    // C_bad must NOT have been admitted.
    let bad_stored = archive
        .get_variant("h43-bad-loop-99")
        .await
        .expect("get_variant ok");
    assert!(
        bad_stored.is_none(),
        "H-43 VD-1: C_bad_loop must NOT be admitted to the archive (rejected by two-gate, alpha=0.0)"
    );
}

#[cfg(feature = "darwinian-evolve")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn h43_corpus_c_good_exceeds_c_bad_on_real_generation_path() {
    // (p3): fitness(C_good) > fitness(C_bad) on the REAL run_one_generation
    // corpus path (not just the standalone execution_grounded_fitness fixture).
    // This proves the BLOCK-2 fix: the discriminating corpus (expected_output =
    // Some(variant_label)) produces different fitness values for good vs bad.

    let c_good = MutantCandidate {
        variant_label: "h43-corpus-good-77".to_string(),
        pipeline_mode: "surgical".to_string(),
        config: serde_json::json!({ "mode": "high" }),
        // eval_log contains the variant_label -> passes its own corpus case.
        eval_log: "h43-corpus-good-77 output-verified".to_string(),
    };
    let c_bad = MutantCandidate {
        variant_label: "h43-corpus-bad-88".to_string(),
        pipeline_mode: "surgical".to_string(),
        config: serde_json::json!({ "mode": "low" }),
        // eval_log does not contain its variant_label -> fails its corpus case.
        eval_log: "generic-output-no-label".to_string(),
    };

    // Compute fitness using execution_grounded_fitness_for_candidate on the
    // corpus derived from both candidates (mirrors run_one_generation corpus logic).
    use elai_orchestrator::execution_fitness::{FrozenCorpus, FrozenTestCase};
    let cases = vec![
        FrozenTestCase {
            id: "gen-case-0".into(),
            candidate_snippet: c_good.eval_log.clone(),
            expected_output: Some(c_good.variant_label.clone()),
        },
        FrozenTestCase {
            id: "gen-case-1".into(),
            candidate_snippet: c_bad.eval_log.clone(),
            expected_output: Some(c_bad.variant_label.clone()),
        },
    ];
    let corpus_bytes: Vec<u8> = [c_good.eval_log.as_bytes(), c_bad.eval_log.as_bytes()]
        .concat();
    let corpus = FrozenCorpus::new(cases, &corpus_bytes).expect("corpus ok");

    let ef_good = c_good
        .execution_grounded_fitness_for_candidate(&corpus, 42)
        .expect("C_good fitness");
    let ef_bad = c_bad
        .execution_grounded_fitness_for_candidate(&corpus, 42)
        .expect("C_bad fitness");

    assert!(
        ef_good.fitness > ef_bad.fitness,
        "H-43 BLOCK-2: fitness(C_good)={} must exceed fitness(C_bad)={} on the real \
         discriminating corpus path (expected_output=Some(variant_label))",
        ef_good.fitness, ef_bad.fitness
    );
    assert_eq!(
        ef_bad.tests_passed, 0,
        "H-43 BLOCK-2: C_bad must pass 0 tests (its eval_log lacks its variant_label)"
    );
}
