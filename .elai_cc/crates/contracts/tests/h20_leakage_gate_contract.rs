//! H-20 contract tests — leakage static-analysis COMPOSITE gate (T-3).
//!
//! Covers (TEST-FIRST, all tests authored before implementation went green):
//!   (1) a fit-before-split fixture is BLOCKED with the fit-site reported AND a
//!       real LeakageBlocked SSB row lands in a real Ssb SQLite db (off-agent
//!       artifact, Layer-4 / R4).
//!   (2) a clean split-then-fit fixture PASSES (Allow).
//!   (3) an absent/empty DKG => Block (fail-closed), NOT Allow (R1/R2).
//!   (4) a flagged site WITHOUT a registered suppression row stays Blocked;
//!       WITH a registered row, suppressed.
//!   (5) the soundness-routing enum routes L2/L3.x to HumanReviewRequired,
//!       never a sound block (Kapoor soundness boundary).
//!   (6) SCHEMA_VERSION >= 225 AND TOTAL_VARIANTS >= 502 AND LeakageBlocked is
//!       present in ALL_VARIANTS (the +1 lockstep landed).
//!
//! SCHEMA_VERSION assertions use `>=` (never an exact global pin) per the
//! BRITTLE_GLOBAL_SCHEMA_PIN gotcha.
//!
//! Grounding:
//!   - Kapoor & Narayanan (2023) "Leakage and the Reproducibility Crisis in
//!     ML-based Science", Patterns 4(9); arXiv:2207.07048 (taxonomy boundary).
//!   - crates/orchestrator/src/leakage_gate.rs (H-20 Tier-1 gate).
//!   - crates/rules/src/predicate.rs (GraphReachability + DkgQuerier).

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::{
    leakage_gate::{
        evaluate_leakage, LeakageInput, LeakageSuppression, LeakageVerdictKind, ResidueClass,
        SoundLeakClass, SplitBoundary, LEAKAGE_RULE_ID,
    },
    ssb::{
        ssb_action_type_from_str_public, LeakageBlockedKind, Ssb, SsbActionType, SsbConfig,
        SsbPayload, SCHEMA_VERSION_PUBLIC,
    },
    SsbReader,
};
use elai_rules::predicate::{DkgError, DkgQuerier, ReachabilityMatch, ReachabilityParams};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

const PRESENCE_PROBE_SINK: &str = "*";
const TRAIN_SINK: &str = "train";
const TEST_SINK: &str = "test";

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

/// Configurable fake DKG querier for the contract fixtures.
struct FakeDkg {
    populated: bool,
    train_paths: Vec<Vec<String>>,
    test_paths: Vec<Vec<String>>,
}

impl DkgQuerier for FakeDkg {
    fn query_reachability(
        &self,
        params: &ReachabilityParams,
    ) -> Result<Vec<ReachabilityMatch>, DkgError> {
        if params.sink_pattern == PRESENCE_PROBE_SINK {
            return Ok(if self.populated {
                vec![ReachabilityMatch {
                    path: vec!["node".to_string()],
                    edge_labels: vec!["data_flow".to_string()],
                }]
            } else {
                vec![]
            });
        }
        let paths = if params.sink_pattern == TRAIN_SINK {
            &self.train_paths
        } else if params.sink_pattern == TEST_SINK {
            &self.test_paths
        } else {
            return Ok(vec![]);
        };
        Ok(paths
            .iter()
            .map(|p| ReachabilityMatch {
                path: p.clone(),
                edge_labels: vec!["data_flow".to_string()],
            })
            .collect())
    }

    fn validate_params(&self, _params: &ReachabilityParams) -> Result<(), DkgError> {
        Ok(())
    }
}

fn split() -> SplitBoundary {
    SplitBoundary {
        split_symbol: "sklearn.train_test_split".to_string(),
        file: "pipeline.py".to_string(),
        line: 12,
    }
}

fn base_input(dkg_available: bool) -> LeakageInput {
    LeakageInput {
        module_path: "pipeline.py".to_string(),
        declared_split: Some(split()),
        fit_source_pattern: "fit_transform".to_string(),
        is_feature_selection: false,
        residue_flag: None,
        dkg_available,
        seed: 42,
    }
}

// ---------------------------------------------------------------------------
// (1) fit-before-split => BLOCK + off-agent LeakageBlocked SSB row (R4)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fit_before_split_blocks_and_emits_ssb_row() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h20-contract-1");

    // fit_transform reaches BOTH the train sink AND the test sink => L1.2 leak.
    let querier = FakeDkg {
        populated: true,
        train_paths: vec![vec![
            "sklearn.StandardScaler.fit_transform".into(),
            "X_train".into(),
        ]],
        test_paths: vec![vec![
            "sklearn.StandardScaler.fit_transform".into(),
            "X_test".into(),
        ]],
    };

    let verdict = evaluate_leakage(&querier, &ssb, base_input(true), &[], 0)
        .await
        .expect("evaluate_leakage must not error");

    assert_eq!(
        verdict.kind,
        LeakageVerdictKind::SoundBlock(SoundLeakClass::PreprocessBeforeSplit),
        "fit reaching both sinks must SoundBlock(L1.2); reason={}",
        verdict.reason
    );
    let fit_site = verdict.fit_site.expect("fit-site must be reported");
    assert_eq!(fit_site.symbol, "sklearn.StandardScaler.fit_transform");

    // Off-agent artifact: exactly one LeakageBlocked row landed in the real DB.
    let rows = ssb
        .query_by_type(SsbActionType::LeakageBlocked)
        .expect("query LeakageBlocked");
    assert_eq!(rows.len(), 1, "exactly 1 LeakageBlocked SSB row (off-agent artifact, R4)");

    let payload: SsbPayload =
        rmp_serde::from_slice(&rows[0].payload).expect("deserialize payload");
    if let SsbPayload::LeakageBlocked(p) = payload {
        assert_eq!(p.kind, LeakageBlockedKind::PreprocessBeforeSplit);
        assert_eq!(p.kapoor_code, "L1.2");
        assert_eq!(p.module_path, "pipeline.py");
        assert_eq!(p.fit_site_symbol, "sklearn.StandardScaler.fit_transform");
        assert!(!p.suppressed);
        assert_eq!(p.corpus_blake3.len(), 64, "blake3 hex is 64 chars");
    } else {
        panic!("payload must be LeakageBlocked");
    }
}

// ---------------------------------------------------------------------------
// (2) clean split-then-fit => Allow
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn clean_split_then_fit_allows() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h20-contract-2");

    // fit reaches ONLY the train sink (correct ordering: fit applied after split).
    let querier = FakeDkg {
        populated: true,
        train_paths: vec![vec!["fit_transform".into(), "X_train".into()]],
        test_paths: vec![],
    };

    let verdict = evaluate_leakage(&querier, &ssb, base_input(true), &[], 0)
        .await
        .expect("evaluate_leakage");
    assert_eq!(verdict.kind, LeakageVerdictKind::Allow);

    // Allow STILL emits an off-agent row (every decision is recorded).
    let rows = ssb
        .query_by_type(SsbActionType::LeakageBlocked)
        .expect("query LeakageBlocked");
    assert_eq!(rows.len(), 1, "Allow decision also emits one SSB row");
}

// ---------------------------------------------------------------------------
// (3) absent/empty DKG => Block (fail-closed), NOT Allow (R1/R2)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn empty_dkg_is_fail_closed_block_not_allow() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h20-contract-3a");

    // populated=false: the presence probe returns empty. With must_not_exist
    // semantics this would naively read as "no path -> Allow" (fail-open). The
    // gate MUST instead Block (dkg_unavailable).
    let querier = FakeDkg { populated: false, train_paths: vec![], test_paths: vec![] };

    let verdict = evaluate_leakage(&querier, &ssb, base_input(true), &[], 0)
        .await
        .expect("evaluate_leakage");
    assert_eq!(
        verdict.kind,
        LeakageVerdictKind::DkgUnavailable,
        "empty DKG must Block fail-closed, NOT Allow"
    );
    assert_ne!(
        verdict.kind,
        LeakageVerdictKind::Allow,
        "empty DKG must never read as Allow (fail-open)"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dkg_available_false_is_fail_closed_block() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h20-contract-3b");

    // Even with a populated querier, an explicit dkg_available=false (caller
    // could not confirm the db) must fail-close.
    let querier = FakeDkg { populated: true, train_paths: vec![], test_paths: vec![] };
    let verdict = evaluate_leakage(&querier, &ssb, base_input(false), &[], 0)
        .await
        .expect("evaluate_leakage");
    assert_eq!(verdict.kind, LeakageVerdictKind::DkgUnavailable);
}

// ---------------------------------------------------------------------------
// (4) suppression: no row => Block stands; registered row => suppressed
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unregistered_site_stays_blocked() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h20-contract-4a");
    let querier = FakeDkg {
        populated: true,
        train_paths: vec![vec!["fit_transform".into(), "X_train".into()]],
        test_paths: vec![vec!["fit_transform".into(), "X_test".into()]],
    };
    // No suppression rows: the sound block STANDS.
    let verdict = evaluate_leakage(&querier, &ssb, base_input(true), &[], 0)
        .await
        .expect("evaluate_leakage");
    assert_eq!(
        verdict.kind,
        LeakageVerdictKind::SoundBlock(SoundLeakClass::PreprocessBeforeSplit)
    );
    assert!(!verdict.suppressed, "no registered row => not suppressed");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn registered_suppression_downgrades_to_allow() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h20-contract-4b");
    let querier = FakeDkg {
        populated: true,
        train_paths: vec![vec!["fit_transform".into(), "X_train".into()]],
        test_paths: vec![vec!["fit_transform".into(), "X_test".into()]],
    };
    let suppressions = vec![LeakageSuppression {
        rule_id: LEAKAGE_RULE_ID.to_string(),
        file: "pipeline.py".to_string(),
        reason: "legacy notebook tracked in JIRA-123".to_string(),
        owner: "maintainer".to_string(),
        expiry_unix: Some(i64::MAX),
    }];
    let verdict = evaluate_leakage(&querier, &ssb, base_input(true), &suppressions, 0)
        .await
        .expect("evaluate_leakage");
    assert_eq!(verdict.kind, LeakageVerdictKind::Allow);
    assert!(verdict.suppressed, "a registered row marks suppressed=true (audited, never silent)");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn wrong_rule_id_suppression_does_not_apply() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h20-contract-4c");
    let querier = FakeDkg {
        populated: true,
        train_paths: vec![vec!["fit_transform".into(), "X_train".into()]],
        test_paths: vec![vec!["fit_transform".into(), "X_test".into()]],
    };
    // Row exists but for a different rule_id => grants NO suppression.
    let suppressions = vec![LeakageSuppression {
        rule_id: "FIN-DEFL-001".to_string(),
        file: "pipeline.py".to_string(),
        reason: "wrong rule".to_string(),
        owner: "maintainer".to_string(),
        expiry_unix: Some(i64::MAX),
    }];
    let verdict = evaluate_leakage(&querier, &ssb, base_input(true), &suppressions, 0)
        .await
        .expect("evaluate_leakage");
    assert_eq!(
        verdict.kind,
        LeakageVerdictKind::SoundBlock(SoundLeakClass::PreprocessBeforeSplit),
        "a non-matching rule_id must NOT suppress"
    );
}

// ---------------------------------------------------------------------------
// (5) soundness routing: L2/L3.x => HumanReviewRequired, never a sound block
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn residue_classes_route_to_human_review_never_sound_block() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h20-contract-5");

    // No fit-reaches-both (clean reachability) but a residue heuristic fired.
    let querier = FakeDkg { populated: true, train_paths: vec![], test_paths: vec![] };

    for residue in [
        ResidueClass::IllegitimateFeatures, // L2
        ResidueClass::Temporal,             // L3.1
        ResidueClass::Nonindependence,      // L3.2
        ResidueClass::SamplingBias,         // L3.3
        ResidueClass::Duplicates,           // L1.4
    ] {
        let mut input = base_input(true);
        input.residue_flag = Some(residue);
        let verdict = evaluate_leakage(&querier, &ssb, input, &[], 0)
            .await
            .expect("evaluate_leakage");
        assert_eq!(
            verdict.kind,
            LeakageVerdictKind::HumanReviewRequired(residue),
            "residue class {:?} must route to human review, never a sound block",
            residue
        );
        // It must NOT be a SoundBlock under any circumstance.
        assert!(
            !matches!(verdict.kind, LeakageVerdictKind::SoundBlock(_)),
            "{:?} must never be a sound static block (Kapoor soundness boundary)",
            residue
        );
    }
}

// ---------------------------------------------------------------------------
// (6) SSB lockstep: SCHEMA_VERSION >= 225, TOTAL_VARIANTS >= 502, present
// ---------------------------------------------------------------------------

#[test]
fn schema_version_and_variant_lockstep_landed() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 225,
        "SCHEMA_VERSION must be >= 225 after H-20; got {SCHEMA_VERSION_PUBLIC}"
    );

    // LeakageBlocked must round-trip through the action_type bijection (sites 3/4).
    let at = ssb_action_type_from_str_public("leakage_blocked")
        .expect("leakage_blocked must be a known action_type");
    assert_eq!(at, SsbActionType::LeakageBlocked);

    // TOTAL_VARIANTS / ALL_VARIANTS presence is asserted in action_type_lock.rs
    // (compile-time exhaustiveness + count-drift) and the e45 contract; here we
    // assert the action_type string side of the lockstep landed.
    assert_eq!(SsbActionType::LeakageBlocked.as_sql_str(), "leakage_blocked");
}
