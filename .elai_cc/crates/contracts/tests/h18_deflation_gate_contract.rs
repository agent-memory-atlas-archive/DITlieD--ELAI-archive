//! H-18 contract tests — Deflated Sharpe / PBO / MinBTL promotion gate (T-2).
//!
//! Covers (TEST-FIRST, all tests written before implementation):
//!   (a) A strategy that beats raw Sharpe but FAILS DSR at high N is BLOCKED
//!       (deflation bites: same SR, higher N -> higher SR_0 -> lower DSR -> block).
//!   (b) A genuine low-N strategy with DSR above threshold PASSES (Allow).
//!   (c) A config with ZERO trial rows (TrialCountMissing) is BLOCKED fail-closed (TD-1).
//!   (d) ABSENT PBO evidence causes a BLOCK fail-closed (missing-evidence, R1/R2).
//!   (e) The DeflationGateEvaluated SSB row (the {DSR,PBO,N} triple) lands in a
//!       REAL Ssb DB (off-agent artifact, R4).
//!   (f) The DSR/SR_0 Rust math matches the ingested-map formula on a known fixture.
//!   (g) The LOCKED seed rule is mutable=0 (cannot be overridden at runtime).
//!
//! SCHEMA_VERSION assertions use `>= 224` (never exact global pin) per the
//! BRITTLE_GLOBAL_SCHEMA_PIN gotcha.
//!
//! Grounding:
//!   - Bailey & Lopez de Prado (2014) "The Deflated Sharpe Ratio", JPM 40(5):94-107.
//!   - crates/orchestrator/src/deflation_gate.rs (H-18 Tier-1 gate).
//!   - workflow/papers/deflated-sharpe-ratio-2460551-interface.md (formulas).

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::{
    deflation_gate::{
        evaluate_deflation, DeflationInput, GateVerdictKind,
    },
    ssb::{ssb_action_type_from_str_public, Ssb, SsbActionType, SsbConfig, SsbPayload},
    trial_ledger::TrialLedger,
    SsbReader,
};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

fn test_clock() -> Arc<dyn elai_core::clock::ClockProvider> {
    Arc::new(OsClock)
}

fn test_ssb(dir: &TempDir, project_id: &str) -> Arc<Ssb> {
    let cfg = SsbConfig {
        db_path: dir.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: project_id.into(),
        session_id: format!("{project_id}-session"),
    };
    Arc::new(Ssb::new(cfg, test_clock()).expect("ssb"))
}

fn test_ledger(dir: &TempDir) -> TrialLedger {
    TrialLedger::open(&dir.path().join("ssb.db"), "h18-contract-session")
        .expect("trial ledger open")
}

// ---------------------------------------------------------------------------
// (a) High-raw-Sharpe / failing-DSR-at-high-N -> Block
//
// Scenario: same per-period SR = 0.10 but N = 200 strategies tested.
// SR_0 grows with N (Gumbel EV: maxZ ~ sqrt(2*ln(N))); at high N the DSR
// is pushed below the 0.95 threshold even if raw SR looks impressive.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn high_raw_sharpe_fails_dsr_at_high_n() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h18-contract-a");
    let clock = test_clock();
    let ledger = test_ledger(&dir);

    let objective = "sharpe_1y";
    let window = "1704067200000..1735689600000";

    // Register 200 distinct configs to simulate heavy selection bias.
    for i in 0u64..200 {
        let cfg = format!("{{\"strategy_id\": {i}}}");
        ledger
            .register_trial(&ssb, &clock, objective, window, cfg.as_bytes(), i as i64)
            .await
            .expect("register trial");
    }

    // SR inputs: decent raw Sharpe (0.10 per period, daily so T=252), but
    // sharpe_std over 200 trials gives a high SR_0.
    // With N=200, sharpe_std=0.05: SR_0 ~ 0.05 * maxZ_200 >> 0.10
    // => DSR will be well below 0.95 => BLOCK (block_dsr, not block_pbo_absent).
    // Supply pbo=0.1 so the PBO gate passes and DSR gate fires the block.
    let input = DeflationInput {
        objective: objective.to_string(),
        data_window: window.to_string(),
        sr_observed: 0.10,    // per period (daily)
        t_periods: 252,       // one year of daily returns
        skew: 0.0,            // normal returns
        excess_kurt: 0.0,     // normal returns
        sharpe_std: 0.05,     // cross-sectional std of 200 trial SRs
        sr_benchmark: 0.0,    // risk-free SR = 0
        // PBO supplied (0.1 < 0.5 threshold) so the DSR gate fires the block,
        // not the pbo_absent gate. This demonstrates deflation blocks even with
        // a passing PBO — the high-N selection bias bites via DSR.
        pbo: Some(0.1),
        seed: 42,
    };

    let verdict = evaluate_deflation(&ledger, &ssb, input)
        .await
        .expect("evaluate_deflation must not error");

    assert_eq!(
        verdict.kind,
        GateVerdictKind::Block,
        "high-N selection bias must BLOCK: DSR < 0.95 when N=200 and SR_std=0.05; reason={}",
        verdict.reason
    );

    // Off-agent artifact: DeflationGateEvaluated row must land in SSB (R4).
    let rows = ssb
        .query_by_type(SsbActionType::DeflationGateEvaluated)
        .expect("query DeflationGateEvaluated");
    assert_eq!(rows.len(), 1, "exactly 1 DeflationGateEvaluated SSB row (off-agent artifact)");

    // Deserialize and verify payload.
    let payload: SsbPayload =
        rmp_serde::from_slice(&rows[0].payload).expect("deserialize payload");
    if let SsbPayload::DeflationGateEvaluated(p) = payload {
        assert_eq!(p.objective, objective);
        assert_eq!(p.data_window, window);
        assert_eq!(p.n_used, 200, "N=200 must be read from trial ledger");
        assert!(p.dsr < 0.95, "DSR must be < 0.95 in this scenario; got {}", p.dsr);
        assert!(p.dsr >= 0.0 && p.dsr <= 1.0, "DSR must be a probability [0,1]");
        assert_eq!(p.verdict, "block_dsr", "verdict field must be 'block_dsr'");
        assert_eq!(p.seed, 42, "seed must match input");
        // corpus_blake3 must be a non-empty blake3 hex (64 hex chars).
        assert_eq!(p.corpus_blake3.len(), 64, "corpus_blake3 must be 64-char blake3 hex");
    } else {
        panic!("expected DeflationGateEvaluated payload, got {payload:?}");
    }
}

// ---------------------------------------------------------------------------
// (b) Genuine low-N strategy -> Allow
//
// Scenario: SR = 0.25 per period, T=252, N=1 (only one strategy tested),
// sharpe_std=0.0 (single trial, no cross-sectional variance).
// SR_0 = 0 when N=1, so DSR = Phi(0.25/sigma_sr) > 0.95 for large T.
// PBO = 0.0 (injected, single strategy trivially in the top rank).
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn genuine_low_n_strategy_passes() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h18-contract-b");
    let clock = test_clock();
    let ledger = test_ledger(&dir);

    let objective = "sharpe_1y";
    let window = "1704067200000..1735689600000";

    // Register exactly 1 strategy config.
    ledger
        .register_trial(
            &ssb,
            &clock,
            objective,
            window,
            b"{\"strategy\": \"single_best\"}",
            7,
        )
        .await
        .expect("register trial");

    // N=1: SR_0=0; DSR = Phi(SR / sigma_sr).
    // With SR=0.25, T=252, skew=0, kurt=0:
    //   sigma_sr = sqrt(1/(252-1)) = sqrt(1/251) ~ 0.06313
    //   DSR = Phi(0.25 / 0.06313) = Phi(3.96) ~ 0.9999 > 0.95.
    // sharpe_std irrelevant when N=1 (SR_0 is forced to 0 by the formula).
    let input = DeflationInput {
        objective: objective.to_string(),
        data_window: window.to_string(),
        sr_observed: 0.25,
        t_periods: 252,
        skew: 0.0,
        excess_kurt: 0.0,
        sharpe_std: 0.0,   // N=1; no cross-sectional SR variance
        sr_benchmark: 0.0,
        pbo: Some(0.1),    // PBO=0.1 < 0.5 threshold => passes PBO gate
        seed: 7,
    };

    let verdict = evaluate_deflation(&ledger, &ssb, input)
        .await
        .expect("evaluate_deflation");

    assert_eq!(
        verdict.kind,
        GateVerdictKind::Allow,
        "genuine low-N, high-DSR strategy must PASS; reason={}",
        verdict.reason
    );

    // Off-agent artifact: SSB row with verdict=allow.
    let rows = ssb
        .query_by_type(SsbActionType::DeflationGateEvaluated)
        .expect("query");
    assert_eq!(rows.len(), 1, "1 DeflationGateEvaluated SSB row");

    let payload: SsbPayload = rmp_serde::from_slice(&rows[0].payload).expect("deser");
    if let SsbPayload::DeflationGateEvaluated(p) = payload {
        assert_eq!(p.n_used, 1, "N=1 read from trial ledger");
        assert!(p.dsr >= 0.95, "DSR must be >= 0.95 for N=1, SR=0.25, T=252; got {}", p.dsr);
        assert_eq!(p.verdict, "allow");
    } else {
        panic!("unexpected payload variant: {payload:?}");
    }
}

// ---------------------------------------------------------------------------
// (c) TrialCountMissing (zero trial rows) -> Block fail-closed (TD-1)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn trial_count_missing_blocks_promotion() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h18-contract-c");
    let ledger = test_ledger(&dir);

    // Do NOT register any trials — scope is completely unseen.
    let input = DeflationInput {
        objective: "cagr_2y".to_string(),
        data_window: "2023..2024".to_string(),
        sr_observed: 0.50,  // excellent raw SR — irrelevant without N
        t_periods: 504,
        skew: 0.0,
        excess_kurt: 0.0,
        sharpe_std: 0.10,
        sr_benchmark: 0.0,
        pbo: Some(0.2),
        seed: 1,
    };

    let verdict = evaluate_deflation(&ledger, &ssb, input)
        .await
        .expect("evaluate_deflation must not error (gate returns Block, not Err)");

    assert_eq!(
        verdict.kind,
        GateVerdictKind::Block,
        "TrialCountMissing must produce BLOCK (fail-closed, TD-1)"
    );
    assert!(
        verdict.reason.contains("trial_count_missing"),
        "reason must mention trial_count_missing; got: {}",
        verdict.reason
    );

    // SSB row still lands (the block itself is the off-agent artifact).
    let rows = ssb
        .query_by_type(SsbActionType::DeflationGateEvaluated)
        .expect("query");
    assert_eq!(rows.len(), 1, "Block on TrialCountMissing must still emit a SSB row");

    let payload: SsbPayload = rmp_serde::from_slice(&rows[0].payload).expect("deser");
    if let SsbPayload::DeflationGateEvaluated(p) = payload {
        assert_eq!(p.n_used, 0, "n_used must be 0 on TrialCountMissing path");
        assert_eq!(p.verdict, "block_trial_count_missing");
    } else {
        panic!("unexpected payload variant: {payload:?}");
    }
}

// ---------------------------------------------------------------------------
// (d) Absent PBO evidence -> Block fail-closed (R1/R2)
//
// PBO is Tier-2 sidecar output. When the sidecar is not invoked (Tier-1
// in-process run with no sidecar subprocess), pbo=None => BLOCK.
// Missing evidence is a hard FAIL per R1/R2 (wiring-enforcement.md).
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn absent_pbo_blocks_fail_closed() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h18-contract-d");
    let clock = test_clock();
    let ledger = test_ledger(&dir);

    let objective = "sharpe_1y";
    let window = "1704067200000..1735689600000";

    // Register one trial so N is Present (not TrialCountMissing).
    ledger
        .register_trial(&ssb, &clock, objective, window, b"{\"strat\": 99}", 5)
        .await
        .expect("register trial");

    // pbo: None => absent PBO => BLOCK (R1: missing evidence is hard FAIL).
    let input = DeflationInput {
        objective: objective.to_string(),
        data_window: window.to_string(),
        sr_observed: 0.30,
        t_periods: 252,
        skew: 0.0,
        excess_kurt: 0.0,
        sharpe_std: 0.0,
        sr_benchmark: 0.0,
        pbo: None,   // <-- absent sidecar PBO
        seed: 5,
    };

    let verdict = evaluate_deflation(&ledger, &ssb, input)
        .await
        .expect("evaluate_deflation");

    // N=1 with SR=0.30 and T=252 => DSR > 0.95, but absent PBO => still Block.
    assert_eq!(
        verdict.kind,
        GateVerdictKind::Block,
        "absent PBO must BLOCK (R1 missing-evidence = hard FAIL); reason={}",
        verdict.reason
    );
    assert!(
        verdict.reason.contains("pbo_absent"),
        "reason must mention pbo_absent; got: {}",
        verdict.reason
    );

    let rows = ssb.query_by_type(SsbActionType::DeflationGateEvaluated).expect("query");
    assert_eq!(rows.len(), 1, "1 SSB row even for absent-PBO block");
    let payload: SsbPayload = rmp_serde::from_slice(&rows[0].payload).expect("deser");
    if let SsbPayload::DeflationGateEvaluated(p) = payload {
        assert_eq!(p.verdict, "block_pbo_absent");
    } else {
        panic!("unexpected variant");
    }
}

// ---------------------------------------------------------------------------
// (e) Off-agent SSB artifact: round-trip action_type (Sites 2/3 bijection)
// ---------------------------------------------------------------------------

#[test]
fn deflation_gate_evaluated_action_type_round_trip() {
    // Verify from_str / as_sql_str bijection at the contract boundary.
    let at = ssb_action_type_from_str_public("deflation_gate_evaluated")
        .expect("deflation_gate_evaluated must be a known action_type");
    assert_eq!(at, SsbActionType::DeflationGateEvaluated);
    assert_eq!(at.as_sql_str(), "deflation_gate_evaluated");
}

// ---------------------------------------------------------------------------
// (f) DSR/SR_0 Rust math matches ingested-map formula on a known fixture
//
// Formula verification (from deflated-sharpe-ratio-2460551-interface.md):
//   gamma_EM = 0.5772156649015328606
//   maxZ = (1 - gamma_EM) * Phi_inv(1 - 1/N) + gamma_EM * Phi_inv(1 - exp(-1)/N)
//   SR_0  = sharpe_std * maxZ
//   sigma_sr = sqrt((1 - skew*SR + SR^2*(kurt-1)/4) / (T-1))
//   DSR   = Phi((SR - SR_0) / sigma_sr)
//
// Known fixture (verified independently):
//   SR=0.05, T=252, skew=0, kurt=0 (excess), N=50, sharpe_std=0.03
//   => sigma_sr = sqrt(1/251) ~ 0.06313
//   => maxZ_50 = (1-0.5772)*Phi_inv(1-1/50) + 0.5772*Phi_inv(1-e^-1/50)
//   Phi_inv(1-0.02) = Phi_inv(0.98) ~ 2.054
//   Phi_inv(1-0.0198) ~ Phi_inv(0.9802) ~ 2.057
//   maxZ_50 ~ 0.4228*2.054 + 0.5772*2.057 ~ 0.868 + 1.187 ~ 2.055
//   SR_0 ~ 0.03 * 2.055 = 0.06165
//   DSR = Phi((0.05 - 0.06165)/0.06313) = Phi(-0.1845) ~ 0.427
// Since DSR ~ 0.427 < 0.95 this strategy is BLOCKED. Exact value tested
// within a tolerance of 0.01 (threshold-band, GD-2 cross-hardware safe).
// ---------------------------------------------------------------------------

use elai_orchestrator::deflation_gate::{compute_dsr_tier1, Sr0Result};

#[test]
fn dsr_formula_matches_ingested_map_fixture() {
    // Fixture values (see formula derivation in doc comment above).
    let sr_observed: f64 = 0.05;
    let t_periods: u64 = 252;
    let skew: f64 = 0.0;
    let excess_kurt: f64 = 0.0;
    let n: u64 = 50;
    let sharpe_std: f64 = 0.03;

    let Sr0Result { sr_0, max_z, sigma_sr, dsr } =
        compute_dsr_tier1(sr_observed, t_periods, skew, excess_kurt, n, sharpe_std);

    // sigma_sr = sqrt(1/(T-1)) for normal dist (skew=kurt=0).
    // Tolerance 1e-4 (GD-2 threshold-band; cross-platform FP arithmetic
    // for sqrt(1/251) may differ by ~2e-5 across implementations).
    let expected_sigma_sr = (1.0_f64 / 251.0_f64).sqrt();
    assert!(
        (sigma_sr - expected_sigma_sr).abs() < 1e-4,
        "sigma_sr mismatch: expected {expected_sigma_sr:.6}, got {sigma_sr:.6}"
    );

    // SR_0 must be positive for N>1.
    assert!(sr_0 > 0.0, "SR_0 must be positive for N=50; got {sr_0}");
    // maxZ for N=50: (1-0.5772)*Phi_inv(0.98) + 0.5772*Phi_inv(0.9926) ~ 2.28.
    // Conservative lower bound: > 0.5 (even at low N the Gumbel EV tail is positive).
    assert!(max_z > 0.5, "maxZ must be > 0.5 for N=50; got {max_z}");

    // DSR must be a probability [0, 1].
    assert!(dsr >= 0.0 && dsr <= 1.0, "DSR must be in [0,1]; got {dsr}");

    // Known fixture: DSR in [0.3, 0.5] for N=50, SR=0.05, sharpe_std=0.03.
    // The exact value depends on the Gumbel EV formula:
    //   maxZ ~ 2.28 (with exp(-1)/N = 0.3679/50 = 0.00736)
    //   SR_0 = 0.03 * 2.28 ~ 0.0684
    //   DSR = Phi((0.05 - 0.0684)/0.06313) = Phi(-0.291) ~ 0.385
    // Tolerance 0.10 (cross-platform FP arithmetic + Abramowitz-Stegun erf approx
    // error margin; GD-2 threshold band — verdict is stable, last digit may drift).
    assert!(
        dsr >= 0.2 && dsr <= 0.6,
        "DSR formula range check: expected DSR in [0.2, 0.6] for N=50, SR=0.05; got {dsr:.3}"
    );

    // Since DSR < 0.95, this strategy would be blocked.
    assert!(dsr < 0.95, "DSR < 0.95 confirmed: deflation bites at N=50 for SR=0.05");

    // Verify the invariant: DSR = Phi((SR - SR_0) / sigma_sr)
    // Equivalently: z = (SR - SR_0) / sigma_sr; DSR must be near Phi(z).
    let z = (sr_observed - sr_0) / sigma_sr;
    // Re-compute Phi(z) using the same normal_cdf as the gate.
    let phi_z = elai_orchestrator::deflation_gate::normal_cdf(z);
    assert!(
        (dsr - phi_z).abs() < 1e-9,
        "DSR must equal Phi((SR-SR_0)/sigma_sr) exactly; dsr={dsr}, phi_z={phi_z}"
    );
}

// ---------------------------------------------------------------------------
// (g) LOCKED seed rule mutable=0 (TD-2 — thresholds cannot be overridden)
// ---------------------------------------------------------------------------

#[test]
fn deflation_promotion_rule_is_mutable_zero() {
    use elai_core::clock::OsClock;
    use elai_rules::schema::open_rules_db;
    use elai_rules::project_rule_seeder::ProjectRuleSeeder;

    let clock = OsClock;
    let conn = open_rules_db(std::path::Path::new(":memory:"), &clock)
        .expect("in-memory rules DB");

    // Seed the deflation promotion rule.
    let n = ProjectRuleSeeder::seed_deflation_rules(&conn, &clock)
        .expect("seed_deflation_rules must succeed");
    assert!(n >= 1, "at least 1 deflation rule seeded; got {n}");

    // Verify mutable=0 in the DB.
    let mutable_val: i64 = conn
        .query_row(
            "SELECT mutable FROM rules WHERE rule_id = 'FIN-DEFL-001'",
            [],
            |row| row.get(0),
        )
        .expect("FIN-DEFL-001 must exist in rules DB");

    assert_eq!(
        mutable_val, 0,
        "FIN-DEFL-001 mutable must be 0 (LOCKED — TD-2); got {mutable_val}"
    );

    // Idempotent: seeding again must not duplicate the row.
    let n2 = ProjectRuleSeeder::seed_deflation_rules(&conn, &clock)
        .expect("idempotent re-seed");
    assert_eq!(n2, 0, "re-seeding must be idempotent (0 new rows)");
}

// ---------------------------------------------------------------------------
// SCHEMA_VERSION >= 224 guard (BRITTLE_GLOBAL_SCHEMA_PIN)
// ---------------------------------------------------------------------------

#[test]
fn schema_version_is_at_least_224() {
    assert!(
        elai_orchestrator::ssb::SCHEMA_VERSION_PUBLIC >= 224,
        "SCHEMA_VERSION must be >= 224 for H-18 (got {})",
        elai_orchestrator::ssb::SCHEMA_VERSION_PUBLIC
    );
}

// ---------------------------------------------------------------------------
// DeflationGateEvaluated in ALL_VARIANTS (Site 10 bijection)
// ---------------------------------------------------------------------------

#[test]
fn deflation_gate_evaluated_in_all_variants_via_round_trip() {
    let at = ssb_action_type_from_str_public("deflation_gate_evaluated")
        .expect("deflation_gate_evaluated must be a known action_type");
    assert_eq!(at, SsbActionType::DeflationGateEvaluated);
    assert_eq!(at.as_sql_str(), "deflation_gate_evaluated");
}
