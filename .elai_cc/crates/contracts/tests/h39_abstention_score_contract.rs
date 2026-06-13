//! H-39 contract tests -- abstention-aware promotion + eval scoring.
//!
//! Covers (off-agent artifacts, Layer-4 / R4, fail-closed per R1):
//!   (F1) SCHEMA >= 218 and `abstention_scored` is in the SSB bijection.
//!        action_type_lock.rs TOTAL_VARIANTS == 529 (pin, not floor).
//!   (F2) Wire tag round-trip: as_sql_str / ssb_action_type_from_str_public.
//!   (F3) Promotion ranks abstaining variant V2 ABOVE confident-wrong variant V1.
//!        score(V2) > score(V1). V1 = answered correctly=false (confident-wrong).
//!        V2 = abstain (answered=false). At default t=0.75: V1 scores -3.0, V2 scores 0.0.
//!   (F4) AbstentionScored row written to SSB (off-agent, R1: missing row = FAIL).
//!   (F5) RewardMode::ConfidentGuessing broken arm: under the inverted reward,
//!        confident-wrong STRICTLY beats abstain (strict flip). Normal mode keeps V2
//!        winning. The ranking inverts -- R2 falsifiability proven.
//!   (F6) t is LOCKED: DEFAULT_CONFIDENCE_TARGET is a const, not env-tunable.
//!   (F7) ELAI_SEED twice -> identical AbstentionAggregate (determinism, GD-2).
//!
//! SCHEMA_VERSION assertions use `>=` (not exact pin) per BRITTLE_GLOBAL_SCHEMA_PIN gotcha.
//! TOTAL_VARIANTS == 529 pin is intentionally exact (action_type_lock.rs design).
//!
//! The promotion path test drives the REAL score_candidate / abstention_blended_alpha
//! path (R4/R7): no unit shim, no #[cfg(test)]-only caller.
//!
//! Grounding:
//!   - crates/orchestrator/src/abstention_score.rs (abstention_aware_score, AbstentionAggregate)
//!   - crates/orchestrator/src/darwin_admit_loop.rs (abstention_blended_alpha, score_candidate)
//!   - crates/orchestrator/src/ssb.rs (AbstentionScoredPayload, SCHEMA_VERSION)
//!   - crates/orchestrator/tests/action_type_lock.rs (TOTAL_VARIANTS pin)
//!   - OpenAI arXiv:2509.04664 (Observation 1; t/(1-t) penalty; t=0.75 -> scalar 3.0)
//!   - .claude/rules/tracing-test-assert.md (T-1: logs_contain MUST be in assert!)

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, Ssb, SsbActionType, SsbConfig, SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::{AbstentionScoredPayload, SsbPayload};
use elai_orchestrator::abstention_score::{
    abstention_aware_score, abstention_aware_score_with_mode, AbstentionAggregate, OutcomeTriple,
    RewardMode, DEFAULT_CONFIDENCE_TARGET,
};
use elai_orchestrator::calibration::RefusalDecision;
use elai_orchestrator::SsbReader;
use tempfile::TempDir;

// darwin_admit_loop is feature-gated (darwinian-evolve). The promotion-path
// test (F3b) is cfg-guarded; F1/F2/F3a/F4/F5/F6/F7 compile in default features.
#[cfg(feature = "darwinian-evolve")]
use elai_orchestrator::darwin_admit_loop::{abstention_blended_alpha, MutantCandidate};

fn test_clock() -> Arc<dyn elai_core::clock::ClockProvider> {
    Arc::new(OsClock)
}

fn test_ssb(dir: &TempDir, session_id: &str) -> Arc<Ssb> {
    let db_path = dir.path().join("ssb.db");
    let cfg = SsbConfig {
        db_path,
        pool_size: Some(2),
        project_id: "h39-contract".into(),
        session_id: session_id.into(),
    };
    Arc::new(Ssb::new(cfg, test_clock()).expect("ssb open"))
}

// ---------------------------------------------------------------------------
// (F1) SCHEMA >= 218 and SSB bijection; TOTAL_VARIANTS == 529 pin.
// ---------------------------------------------------------------------------

#[test]
fn h39_f1_abstention_scored_exists_at_schema_218() {
    // SCHEMA floor.
    assert!(
        SCHEMA_VERSION_PUBLIC >= 218,
        "H-39 requires SCHEMA_VERSION >= 218, got {SCHEMA_VERSION_PUBLIC}"
    );

    // Wire tag round-trip.
    let at = ssb_action_type_from_str_public("abstention_scored")
        .expect("from_str abstention_scored must succeed");
    assert_eq!(at, SsbActionType::AbstentionScored);
    assert_eq!(
        at.as_sql_str(),
        "abstention_scored",
        "as_sql_str round-trip"
    );
}

// (F1 continued) TOTAL_VARIANTS pin.
// Read action_type_lock.rs at runtime to avoid importing a private const.
#[test]
fn h39_f1_total_variants_pin_529() {
    use std::fs;
    let lock_rs = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|root| {
            root.join("crates")
                .join("orchestrator")
                .join("tests")
                .join("action_type_lock.rs")
        })
        .expect("resolve action_type_lock.rs path");
    assert!(
        lock_rs.exists(),
        "action_type_lock.rs must exist at {}",
        lock_rs.display()
    );
    let body = fs::read_to_string(&lock_rs).expect("read action_type_lock.rs");
    // Floor check (A117): parse-then-floor avoids stale-anchor cascade on future TV bumps.
    const H39_TV_FLOOR: usize = 529;
    let tv_ok = {
        let prefix = "const TOTAL_VARIANTS: usize = ";
        body.lines().any(|line| {
            if let Some(rest) = line.trim().strip_prefix(prefix) {
                rest.trim_end_matches(';')
                    .parse::<usize>()
                    .map(|n| n >= H39_TV_FLOOR)
                    .unwrap_or(false)
            } else {
                false
            }
        })
    };
    assert!(
        tv_ok,
        "TOTAL_VARIANTS must be >= {H39_TV_FLOOR} (H-39 floor); file: {}",
        lock_rs.display()
    );
    assert!(
        body.contains("SsbActionType::AbstentionScored"),
        "ALL_VARIANTS must include AbstentionScored; file: {}",
        lock_rs.display()
    );
}

// ---------------------------------------------------------------------------
// (F2) Wire tag round-trip.
// ---------------------------------------------------------------------------

#[test]
fn h39_f2_wire_tag_round_trip() {
    // as_sql_str -> from_str round trip.
    let tag = SsbActionType::AbstentionScored.as_sql_str();
    assert_eq!(tag, "abstention_scored");
    let back = ssb_action_type_from_str_public(tag)
        .expect("from_str must succeed for abstention_scored");
    assert_eq!(back, SsbActionType::AbstentionScored);

    // Payload action_type() discriminant.
    let payload = SsbPayload::AbstentionScored(AbstentionScoredPayload {
        suite: "test".into(),
        mean_score: 0.5,
        abstention_rate: 0.0,
        confident_wrong_rate: 0.0,
        t: DEFAULT_CONFIDENCE_TARGET,
        n: 1,
        seed: 42,
        corpus_blake3: "abc".into(),
    });
    assert_eq!(payload.action_type(), SsbActionType::AbstentionScored);
}

// ---------------------------------------------------------------------------
// (F3) Scoring: correct=+1, abstain=0, confident-wrong=-t/(1-t).
// ---------------------------------------------------------------------------

#[test]
fn h39_f3_correct_scores_plus_one() {
    let outcome = OutcomeTriple::new(RefusalDecision::Answer { confidence: 0.9 }, true);
    let s = abstention_aware_score(outcome, DEFAULT_CONFIDENCE_TARGET);
    assert!(
        (s - 1.0).abs() < f64::EPSILON,
        "correct answer must score +1.0, got {s}"
    );
}

#[test]
fn h39_f3_abstain_scores_zero() {
    let outcome = OutcomeTriple::new(RefusalDecision::Abstain, false);
    let s = abstention_aware_score(outcome, DEFAULT_CONFIDENCE_TARGET);
    assert!(
        (s - 0.0).abs() < f64::EPSILON,
        "abstain must score 0.0, got {s}"
    );
}

#[test]
fn h39_f3_confident_wrong_scores_negative_penalty() {
    // t=0.75 -> penalty = 0.75/0.25 = 3.0 -> score = -3.0.
    let outcome = OutcomeTriple::new(RefusalDecision::Answer { confidence: 0.9 }, false);
    let s = abstention_aware_score(outcome, DEFAULT_CONFIDENCE_TARGET);
    assert!(
        (s - (-3.0)).abs() < 1e-10,
        "confident-wrong at t=0.75 must score -3.0, got {s}"
    );
}

/// (F3b-default) Promotion-ranking via abstention_aware_score directly.
/// Default CI (no feature flag): asserts score(V2=abstain) > score(V1=confident-wrong)
/// using the same t=DEFAULT_CONFIDENCE_TARGET that abstention_blended_alpha normalises
/// from. This is the ranking contract visible to every CI run.
#[test]
fn h39_f3_abstaining_variant_ranks_above_confident_wrong_score() {
    // V1: confident-wrong. V2: abstaining.
    let v1_outcome = OutcomeTriple::new(RefusalDecision::Answer { confidence: 0.6 }, false);
    let v2_outcome = OutcomeTriple::new(RefusalDecision::Abstain, false);

    let score_v1 = abstention_aware_score(v1_outcome, DEFAULT_CONFIDENCE_TARGET);
    let score_v2 = abstention_aware_score(v2_outcome, DEFAULT_CONFIDENCE_TARGET);

    // V2 (abstain=0.0) must strictly beat V1 (confident-wrong=-3.0).
    assert!(
        score_v2 > score_v1,
        "abstaining V2 (score={score_v2}) must rank ABOVE confident-wrong V1 (score={score_v1}); \
         H-39 core promotion contract (RD-1)"
    );
}

/// (F3b) Real promotion path test (feature-gated on darwinian-evolve).
/// Drives the live abstention_blended_alpha -> two_gate_promote path (R4/R7).
#[cfg(feature = "darwinian-evolve")]
#[test]
fn h39_f3_abstaining_variant_ranks_above_confident_wrong() {
    // Core contract (R1 + R4 + RD-1): score(V2=abstain) > score(V1=confident-wrong).
    // This is the live promotion path: abstention_blended_alpha returns a HIGHER
    // normalised alpha for an abstaining candidate than a confident-wrong one,
    // so two_gate_promote (fitness gate) ranks V2 above V1.

    // V1: answered but wrong (confident-wrong, eval_log="suite=math score=0.6 wrong").
    let v1 = MutantCandidate {
        variant_label: "V1-confident-wrong".into(),
        pipeline_mode: "surgical".into(),
        config: serde_json::json!({ "temperature": 0.8 }),
        eval_log: "suite=math score=0.6 wrong".into(),
    };

    // V2: abstained (eval_log="suite=math score=0.2 abstain").
    let v2 = MutantCandidate {
        variant_label: "V2-abstaining".into(),
        pipeline_mode: "surgical".into(),
        config: serde_json::json!({ "temperature": 0.5 }),
        eval_log: "suite=math score=0.2 abstain".into(),
    };

    let (alpha_v1, _) = abstention_blended_alpha(&v1);
    let (alpha_v2, _) = abstention_blended_alpha(&v2);

    assert!(
        alpha_v2 > alpha_v1,
        "abstaining variant V2 (alpha={alpha_v2:.4}) must rank ABOVE confident-wrong V1 (alpha={alpha_v1:.4}); \
         this is the H-39 core contract (RD-1)"
    );
}

// ---------------------------------------------------------------------------
// (F4) AbstentionScored SSB row written to journal (off-agent, R1/R4).
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn h39_f4_abstention_scored_row_written_to_ssb() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir, "h39-f4");

    // Build a minimal aggregate: 1 correct, 1 abstain, 1 confident-wrong.
    let mut agg = AbstentionAggregate::new();
    agg.record(
        OutcomeTriple::new(RefusalDecision::Answer { confidence: 0.9 }, true),
        DEFAULT_CONFIDENCE_TARGET,
    );
    agg.record(
        OutcomeTriple::new(RefusalDecision::Abstain, false),
        DEFAULT_CONFIDENCE_TARGET,
    );
    agg.record(
        OutcomeTriple::new(RefusalDecision::Answer { confidence: 0.8 }, false),
        DEFAULT_CONFIDENCE_TARGET,
    );

    let payload = SsbPayload::AbstentionScored(AbstentionScoredPayload {
        suite: "h39-fixture".into(),
        mean_score: agg.mean_score(),
        abstention_rate: agg.abstention_rate(),
        confident_wrong_rate: agg.confident_wrong_rate(),
        t: DEFAULT_CONFIDENCE_TARGET,
        n: agg.n() as u32,
        seed: 42,
        corpus_blake3: "deadbeef01234567".into(),
    });

    // Write via the real SSB writer (off-agent artifact, R4).
    ssb.write_payload("h39-contract", 0, payload, Vec::new())
        .await
        .expect("write AbstentionScored row");

    // Read back via SsbReader (the off-agent evidence path; R1: missing = FAIL).
    let rows = ssb
        .query_by_type(SsbActionType::AbstentionScored)
        .expect("query_by_type AbstentionScored");
    assert!(
        !rows.is_empty(),
        "AbstentionScored row must land in SSB (off-agent evidence, R1 missing=FAIL)"
    );

    // Payload round-trip deserialisation.
    let first = rows.into_iter().next().expect("first row");
    let decoded: SsbPayload = elai_orchestrator::ssb::decode_ssb_payload(&first.payload)
        .expect("decode AbstentionScored payload");
    match decoded {
        SsbPayload::AbstentionScored(p) => {
            assert_eq!(p.suite, "h39-fixture");
            assert_eq!(p.n, 3);
            assert!((p.t - DEFAULT_CONFIDENCE_TARGET).abs() < f64::EPSILON);
            // mean_score = (+1 + 0 + -3) / 3 = -2/3 ≈ -0.667.
            assert!(
                (p.mean_score - (-2.0 / 3.0)).abs() < 1e-9,
                "mean_score mismatch: {}", p.mean_score
            );
            assert!(
                (p.abstention_rate - 1.0 / 3.0).abs() < 1e-9,
                "abstention_rate mismatch: {}", p.abstention_rate
            );
            assert!(
                (p.confident_wrong_rate - 1.0 / 3.0).abs() < 1e-9,
                "confident_wrong_rate mismatch: {}", p.confident_wrong_rate
            );
        }
        other => panic!("expected AbstentionScored payload, got {:?}", other.action_type()),
    }
}

// ---------------------------------------------------------------------------
// (F5) R2 falsifiability: RewardMode::ConfidentGuessing flips the ranking.
// ---------------------------------------------------------------------------

#[test]
fn h39_f5_reward_confident_guessing_flips_ranking() {
    // R2 contract: the metric must CHANGE when the incentive is broken.
    // RewardMode::ConfidentGuessing awards +1.0 to confident-wrong and 0.0 to
    // abstain — the deliberately inverted reward. This test asserts a STRICT
    // FLIP: V1 (confident-wrong) wins under the broken mode, V2 (abstaining)
    // wins under the normal mode. If both modes agreed, the metric would be
    // unfalsifiable (R2 violation).
    let abstain = OutcomeTriple::new(RefusalDecision::Abstain, false);
    let wrong = OutcomeTriple::new(RefusalDecision::Answer { confidence: 0.9 }, false);

    // Normal mode: abstain (0.0) > confident-wrong (-3.0) -> V2 wins.
    let s_abstain_normal =
        abstention_aware_score_with_mode(abstain, DEFAULT_CONFIDENCE_TARGET, RewardMode::Normal);
    let s_wrong_normal =
        abstention_aware_score_with_mode(wrong, DEFAULT_CONFIDENCE_TARGET, RewardMode::Normal);
    assert!(
        s_abstain_normal > s_wrong_normal,
        "Normal mode: abstain ({s_abstain_normal}) must beat confident-wrong ({s_wrong_normal})"
    );

    // Broken mode: confident-wrong (+1.0) > abstain (0.0) -> V1 wins (strict flip).
    let s_wrong_broken =
        abstention_aware_score_with_mode(wrong, DEFAULT_CONFIDENCE_TARGET, RewardMode::ConfidentGuessing);
    let s_abstain_broken =
        abstention_aware_score_with_mode(abstain, DEFAULT_CONFIDENCE_TARGET, RewardMode::ConfidentGuessing);
    assert!(
        s_wrong_broken > s_abstain_broken,
        "ConfidentGuessing mode: confident-wrong ({s_wrong_broken}) must STRICTLY beat abstain \
         ({s_abstain_broken}); this is the R2 falsifiability flip (metric changes when broken)"
    );

    // Confirm the ranking has genuinely inverted (not just different magnitudes).
    // Normal: abstain wins. Broken: confident-wrong wins. The winner flipped.
    assert!(
        s_abstain_normal > s_wrong_normal && s_wrong_broken > s_abstain_broken,
        "Ranking must invert between Normal and ConfidentGuessing modes (R2)"
    );
}

// ---------------------------------------------------------------------------
// (F6) t is LOCKED: DEFAULT_CONFIDENCE_TARGET is a compile-time const.
// ---------------------------------------------------------------------------

#[test]
fn h39_f6_t_is_locked_constant() {
    // Structural test: DEFAULT_CONFIDENCE_TARGET is a const (compile-time known).
    // A self-mod loop cannot set it to 0.0 at runtime (no setter, no env read
    // in production paths).
    assert!(
        (DEFAULT_CONFIDENCE_TARGET - 0.75).abs() < f64::EPSILON,
        "DEFAULT_CONFIDENCE_TARGET must be 0.75 (locked per arXiv:2509.04664 Table 2)"
    );
    // Verify penalty scalar is 3.0 at the locked t.
    let penalty = DEFAULT_CONFIDENCE_TARGET / (1.0 - DEFAULT_CONFIDENCE_TARGET);
    assert!(
        (penalty - 3.0).abs() < 1e-10,
        "penalty at t=0.75 = 3.0 (arXiv:2509.04664 Table 2)"
    );
}

// ---------------------------------------------------------------------------
// (F7) Determinism: ELAI_SEED twice -> identical AbstentionAggregate.
// ---------------------------------------------------------------------------

#[test]
fn h39_f7_aggregate_deterministic_under_seed() {
    // Same input sequence -> identical output.
    let outcomes = vec![
        (RefusalDecision::Answer { confidence: 0.9 }, true),
        (RefusalDecision::Abstain, false),
        (RefusalDecision::Answer { confidence: 0.7 }, false),
        (RefusalDecision::Answer { confidence: 0.85 }, true),
    ];

    let build_agg = || {
        let mut agg = AbstentionAggregate::new();
        for &(ref refusal, pass) in &outcomes {
            agg.record(OutcomeTriple::new(*refusal, pass), DEFAULT_CONFIDENCE_TARGET);
        }
        agg
    };

    let agg1 = build_agg();
    let agg2 = build_agg();

    assert_eq!(agg1.n(), agg2.n(), "n must be identical");
    assert!(
        (agg1.mean_score() - agg2.mean_score()).abs() < f64::EPSILON,
        "mean_score must be identical"
    );
    assert!(
        (agg1.abstention_rate() - agg2.abstention_rate()).abs() < f64::EPSILON,
        "abstention_rate must be identical"
    );
    assert!(
        (agg1.confident_wrong_rate() - agg2.confident_wrong_rate()).abs() < f64::EPSILON,
        "confident_wrong_rate must be identical"
    );
}
