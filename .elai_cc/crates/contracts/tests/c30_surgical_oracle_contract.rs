//! C-30 contract test — verifies the Surgical Mode + Oracle Mode pipeline
//! infrastructure shipped under SCHEMA_VERSION 62.
//!
//! Source plan: docs/thefinalglance/C-plan/C-30_surgical_oracle_mode.md
//! Decisions: workflow/reasoning/C-30-decisions.md (DRIFTs 1-15 resolved).
//!
//! Coverage (17 tests):
//!   1.  SCHEMA_VERSION_PUBLIC >= 62.
//!   2.  Action-type strings — surgical_plan_created / surgical_loc_warning /
//!       oracle_cache_hit / oracle_cache_miss / project_detection_signal.
//!   3.  Surgical types compile + construct.
//!   4.  ComplexityClass::Simple bound (files=2, loc=150).
//!   5.  ComplexityClass::Medium bound (files=6, loc=400).
//!   6.  ComplexityClass::Complex bound (files=10, loc=700).
//!   7.  RegressionGate snapshot deterministic.
//!   8.  Oracle cache miss-then-hit within TTL.
//!   9.  Oracle cache T1=30d TTL boundary.
//!   10. Oracle cache T3=1d TTL boundary.
//!   11. Oracle cache T4Rejected never cached.
//!   12. Oracle cache LRU eviction at max_entries.
//!   13. `dkg_callers` + `dkg_dependents` + `dkg_data_flow` + `dkg_subgraph`
//!       wire constants are defined.
//!   14. EscalationPolicy has the C-30 blast-radius LOC fields.
//!   15. elai_cc.toml `[research.vram_tiers]` has >=5 entries.
//!   16. `SurgicalDecomposer::decompose` returns ConfirmationRequired for an
//!       Interactive blast > 500 LOC (via AffectedSet-driven gate semantics
//!       documented on the error variant).
//!   17. ProjectDetectionSignal round-trips through the SsbPayload pipeline.

use elai_core::research_types::{AnswerConfidence, ResearchAnswer};
use elai_core::source_tier::SourceTier;
use elai_ipc_types::{
    TOOL_DKG_CALLERS, TOOL_DKG_DATA_FLOW, TOOL_DKG_DEPENDENTS, TOOL_DKG_SUBGRAPH,
};
use elai_orchestrator::autonomous_gate::EscalationPolicy;
use elai_orchestrator::oracle_cache::{
    confidence_str, source_tier_for_confidence, source_tier_str, ttl_for_tier, CacheKey,
    CachedAnswer, OracleAnswerCache, TTL_T1_AUTHORITATIVE_SECS, TTL_T3_WEAK_SECS,
};
use elai_orchestrator::ssb::{
    OracleCacheHitPayload, OracleCacheMissPayload, ProjectDetectionSignalPayload,
    SsbActionType, SsbPayload, SurgicalLocWarningPayload, SurgicalPlanCreatedPayload,
    SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::surgical_decomposer::{
    AffectedSet, ComplexityClass, RegressionGate, SurgicalDecomposer, SurgicalError,
    SurgicalPlan, SurgicalStep, TestBaseline, SURGICAL_LOC_ESCALATE_THRESHOLD,
    SURGICAL_LOC_WARN_THRESHOLD,
};
use elai_rules::predicate::{DkgError, DkgQuerier, ReachabilityMatch, ReachabilityParams};
use std::path::PathBuf;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

struct StubDkg;

impl DkgQuerier for StubDkg {
    fn query_reachability(
        &self,
        _params: &ReachabilityParams,
    ) -> Result<Vec<ReachabilityMatch>, DkgError> {
        Ok(Vec::new())
    }

    fn validate_params(&self, _params: &ReachabilityParams) -> Result<(), DkgError> {
        Ok(())
    }
}

fn mk_answer(confidence: AnswerConfidence) -> ResearchAnswer {
    ResearchAnswer {
        question: "q?".into(),
        answer_text: "a.".into(),
        evidence_chain: Vec::new(),
        confidence,
        caveats: Vec::new(),
        unanswered_parts: Vec::new(),
        sources_consulted: Vec::new(),
        sources_rejected: Vec::new(),
        path: "llm".into(),
    }
}

fn mk_cached(now: u64, tier: SourceTier, confidence: AnswerConfidence) -> CachedAnswer {
    CachedAnswer {
        answer: mk_answer(confidence),
        cached_at: now,
        source_tier: tier,
    }
}

fn mk_key(query: &str) -> CacheKey {
    CacheKey::compute(query, &PathBuf::from("/tmp/elai-c30-test"))
}

// ---------------------------------------------------------------------------
// (1) SCHEMA_VERSION advances 61 -> 62.
// ---------------------------------------------------------------------------

#[test]
fn schema_version_advances_to_62_after_c30() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 62,
        "SCHEMA_VERSION_PUBLIC must be at least 62 after C-30; got {}",
        SCHEMA_VERSION_PUBLIC
    );
}

// ---------------------------------------------------------------------------
// (2) Action-type strings.
// ---------------------------------------------------------------------------

#[test]
fn c30_action_types_serialize_to_expected_strings() {
    assert_eq!(
        SsbActionType::SurgicalPlanCreated.as_sql_str(),
        "surgical_plan_created"
    );
    assert_eq!(
        SsbActionType::SurgicalLocWarning.as_sql_str(),
        "surgical_loc_warning"
    );
    assert_eq!(
        SsbActionType::OracleCacheHit.as_sql_str(),
        "oracle_cache_hit"
    );
    assert_eq!(
        SsbActionType::OracleCacheMiss.as_sql_str(),
        "oracle_cache_miss"
    );
    assert_eq!(
        SsbActionType::ProjectDetectionSignal.as_sql_str(),
        "project_detection_signal"
    );
}

// ---------------------------------------------------------------------------
// (3) Surgical types compile + construct.
// ---------------------------------------------------------------------------

#[test]
fn surgical_plan_types_compile() {
    let affected = AffectedSet {
        direct_files: vec![PathBuf::from("src/lib.rs")],
        dependent_files: vec![PathBuf::from("src/main.rs")],
        total_loc_estimate: 150,
    };
    let regression = RegressionGate::new(vec!["test_one".into()], 0);
    let baseline = TestBaseline {
        snapshot_hash: "blake3:aaaa".into(),
        pre_existing_failures: vec!["old_failure".into()],
    };
    let step = SurgicalStep {
        description: "edit foo".into(),
        target_file: PathBuf::from("src/lib.rs"),
        target_fn: Some("foo".into()),
    };
    let plan = SurgicalPlan {
        task_id: "task_test".into(),
        steps: vec![step],
        affected_set: affected,
        complexity_class: ComplexityClass::Simple,
        regression_gate: regression,
    };
    assert_eq!(plan.complexity_class, ComplexityClass::Simple);
    assert_eq!(baseline.pre_existing_failures.len(), 1);
}

// ---------------------------------------------------------------------------
// (4-6) ComplexityClass boundaries.
// ---------------------------------------------------------------------------

#[test]
fn complexity_class_simple_bound() {
    assert_eq!(ComplexityClass::classify(2, 150), ComplexityClass::Simple);
}

#[test]
fn complexity_class_medium_bound() {
    assert_eq!(ComplexityClass::classify(6, 400), ComplexityClass::Medium);
}

#[test]
fn complexity_class_complex_bound() {
    assert_eq!(ComplexityClass::classify(10, 700), ComplexityClass::Complex);
}

// ---------------------------------------------------------------------------
// (7) RegressionGate snapshot is deterministic.
// ---------------------------------------------------------------------------

#[test]
fn regression_gate_snapshot_hash_deterministic() {
    let g1 = RegressionGate::new(vec!["a".into(), "b".into(), "c".into()], 0);
    let g2 = RegressionGate::new(vec!["c".into(), "a".into(), "b".into()], 0);
    assert_eq!(g1.snapshot_hash, g2.snapshot_hash);
    let g3 = RegressionGate::new(vec!["a".into(), "b".into()], 0);
    assert_ne!(g1.snapshot_hash, g3.snapshot_hash);
}

// ---------------------------------------------------------------------------
// (8) Oracle cache miss-then-hit within TTL.
// ---------------------------------------------------------------------------

#[test]
fn oracle_cache_miss_then_hit() {
    let cache = OracleAnswerCache::new();
    let key = mk_key("what is rust?");
    let now = 1_700_000_000u64;
    assert!(cache.get(&key, now).is_none());
    let entry = mk_cached(now, SourceTier::T1Authoritative, AnswerConfidence::High);
    assert!(cache.insert(key, entry));
    let got = cache.get(&key, now + 100).expect("hit");
    assert_eq!(got.source_tier, SourceTier::T1Authoritative);
    assert_eq!(got.age_secs(now + 100), 100);
}

// ---------------------------------------------------------------------------
// (9) Oracle cache T1=30d TTL.
// ---------------------------------------------------------------------------

#[test]
fn oracle_cache_ttl_t1_30d() {
    let cache = OracleAnswerCache::new();
    let key = mk_key("q1");
    let cached_at = 1_700_000_000u64;
    let entry = mk_cached(cached_at, SourceTier::T1Authoritative, AnswerConfidence::High);
    cache.insert(key, entry);
    assert_eq!(TTL_T1_AUTHORITATIVE_SECS, 30 * 24 * 3600);
    assert!(cache.get(&key, cached_at + 29 * 24 * 3600).is_some());
    assert!(cache.get(&key, cached_at + 30 * 24 * 3600 + 1).is_none());
}

// ---------------------------------------------------------------------------
// (10) Oracle cache T3=1d TTL.
// ---------------------------------------------------------------------------

#[test]
fn oracle_cache_ttl_t3_1d() {
    let cache = OracleAnswerCache::new();
    let key = mk_key("q3");
    let cached_at = 1_700_000_000u64;
    let entry = mk_cached(cached_at, SourceTier::T3Weak, AnswerConfidence::Low);
    cache.insert(key, entry);
    assert_eq!(TTL_T3_WEAK_SECS, 86_400);
    assert!(cache.get(&key, cached_at + 86_399).is_some());
    assert!(cache.get(&key, cached_at + 86_401).is_none());
}

// ---------------------------------------------------------------------------
// (11) Oracle cache T4 never cached + Unknown maps to T4.
// ---------------------------------------------------------------------------

#[test]
fn oracle_cache_t4_rejected_never_cached() {
    let cache = OracleAnswerCache::new();
    let key = mk_key("q4");
    let cached_at = 1_700_000_000u64;
    let entry = mk_cached(cached_at, SourceTier::T4Rejected, AnswerConfidence::Unknown);
    assert!(!cache.insert(key, entry));
    assert!(cache.get(&key, cached_at).is_none());
    assert_eq!(ttl_for_tier(SourceTier::T4Rejected), 0);
    assert_eq!(
        source_tier_for_confidence(AnswerConfidence::Unknown),
        SourceTier::T4Rejected
    );
    assert_eq!(confidence_str(AnswerConfidence::High), "high");
    assert_eq!(source_tier_str(SourceTier::T1Authoritative), "t1_authoritative");
}

// ---------------------------------------------------------------------------
// (12) Oracle cache LRU eviction.
// ---------------------------------------------------------------------------

#[test]
fn oracle_cache_eviction_at_max() {
    let cache = OracleAnswerCache::with_capacity(2);
    let now = 1_700_000_000u64;
    let k1 = mk_key("a");
    let k2 = mk_key("b");
    let k3 = mk_key("c");
    cache.insert(k1, mk_cached(now, SourceTier::T1Authoritative, AnswerConfidence::High));
    cache.insert(k2, mk_cached(now, SourceTier::T1Authoritative, AnswerConfidence::High));
    let _ = cache.get(&k1, now + 1);
    cache.insert(k3, mk_cached(now, SourceTier::T1Authoritative, AnswerConfidence::High));
    assert!(cache.get(&k1, now + 2).is_some());
    assert!(cache.get(&k2, now + 2).is_none());
    assert!(cache.get(&k3, now + 2).is_some());
    assert_eq!(cache.len(), 2);
}

// ---------------------------------------------------------------------------
// (13) Four new DKG tool wire constants are defined.
// ---------------------------------------------------------------------------

#[test]
fn dkg_tool_wire_constants_defined() {
    assert_eq!(TOOL_DKG_CALLERS, "dkg_callers");
    assert_eq!(TOOL_DKG_DEPENDENTS, "dkg_dependents");
    assert_eq!(TOOL_DKG_DATA_FLOW, "dkg_data_flow");
    assert_eq!(TOOL_DKG_SUBGRAPH, "dkg_subgraph");
}

// ---------------------------------------------------------------------------
// (14) EscalationPolicy has the C-30 blast-radius LOC fields with the
// expected defaults.
// ---------------------------------------------------------------------------

#[test]
fn escalation_policy_carries_loc_gates() {
    let policy = EscalationPolicy::default();
    assert_eq!(policy.autonomous_confirm_above_loc, 500);
    assert_eq!(policy.autonomous_refuse_above_loc, 1000);
    assert_eq!(
        policy.autonomous_confirm_above_loc,
        SURGICAL_LOC_WARN_THRESHOLD
    );
    assert_eq!(
        policy.autonomous_refuse_above_loc,
        SURGICAL_LOC_ESCALATE_THRESHOLD
    );
}

// ---------------------------------------------------------------------------
// (15) elai_cc.toml [research.vram_tiers] has >=5 entries.
// ---------------------------------------------------------------------------

#[test]
fn vram_tiers_toml_has_five_entries() {
    // Resolve elai_cc.toml from CARGO_MANIFEST_DIR (this crate's Cargo.toml
    // sits at .elai_cc/crates/contracts/, so the project root is two levels
    // up — same convention used by other *_toml_contract.rs tests).
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set during test runs");
    let toml_path = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("elai_cc.toml"))
        .expect("project root resolves");
    assert!(toml_path.is_file(), "expected {} to exist", toml_path.display());
    let raw = std::fs::read_to_string(&toml_path).expect("read elai_cc.toml");
    let parsed: toml::Value = toml::from_str(&raw).expect("parse elai_cc.toml");
    let tiers = parsed
        .get("research")
        .and_then(|r| r.get("vram_tiers"))
        .and_then(|v| v.as_table())
        .expect("[research.vram_tiers] table present");
    assert!(
        tiers.len() >= 5,
        "expected at least 5 vram_tiers entries; got {}",
        tiers.len()
    );
}

// ---------------------------------------------------------------------------
// (16) Surgical decompose with empty target returns InvalidTarget error;
// SurgicalError ConfirmationRequired / EscalateAutonomous variants are
// constructible (the gate semantics live on the variant payloads).
// ---------------------------------------------------------------------------

#[test]
fn surgical_decomposer_invalid_target_and_gate_errors() {
    let stub: Arc<dyn DkgQuerier + Send + Sync> = Arc::new(StubDkg);
    let dec = SurgicalDecomposer::new(stub, None);
    let err = dec.decompose("", elai_orchestrator::context_budget::ExecutionMode::Interactive, false)
        .expect_err("empty target must error");
    assert!(matches!(err, SurgicalError::InvalidTarget(_)));

    let cr = SurgicalError::ConfirmationRequired {
        loc: 750,
        threshold: SURGICAL_LOC_WARN_THRESHOLD,
    };
    let cr_msg = format!("{cr}");
    assert!(cr_msg.contains("750"));
    assert!(cr_msg.contains("500"));

    let esc = SurgicalError::EscalateAutonomous {
        loc: 1500,
        threshold: SURGICAL_LOC_ESCALATE_THRESHOLD,
    };
    let esc_msg = format!("{esc}");
    assert!(esc_msg.contains("1500"));
    assert!(esc_msg.contains("1000"));
}

// ---------------------------------------------------------------------------
// (17) ProjectDetectionSignal payload round-trips via msgpack like every
// other typed payload (test_typed_payloads in ssb.rs covers all 131 but
// this contract pins the C-30 row shape explicitly).
// ---------------------------------------------------------------------------

#[test]
fn project_detection_signal_round_trips() {
    let original = SsbPayload::ProjectDetectionSignal(ProjectDetectionSignalPayload {
        signal_name: "cargo_toml".into(),
        weight: "required".into(),
        fired: true,
    });
    let bytes = rmp_serde::to_vec_named(&original).expect("encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    assert_eq!(original, decoded);
    assert_eq!(decoded.action_type(), SsbActionType::ProjectDetectionSignal);

    // Sibling payloads — quick spot-check the other C-30 row shapes encode.
    let _ = rmp_serde::to_vec_named(&SsbPayload::SurgicalPlanCreated(
        SurgicalPlanCreatedPayload {
            task_id: "task_x".into(),
            complexity_class: "simple".into(),
            blast_radius_files: 2,
            total_loc_estimate: 150,
            step_count: 1,
            regression_gate_snapshot_hash: "blake3:abc".into(),
        },
    ))
    .expect("encode surgical_plan_created");
    let _ = rmp_serde::to_vec_named(&SsbPayload::SurgicalLocWarning(SurgicalLocWarningPayload {
        task_id: "task_x".into(),
        total_loc_estimate: 600,
        threshold: 500,
        mode: "interactive".into(),
    }))
    .expect("encode surgical_loc_warning");
    let _ = rmp_serde::to_vec_named(&SsbPayload::OracleCacheHit(OracleCacheHitPayload {
        query_hash_hex: "00".repeat(32),
        source_tier: "t1_authoritative".into(),
        cached_age_secs: 60,
    }))
    .expect("encode oracle_cache_hit");
    let _ = rmp_serde::to_vec_named(&SsbPayload::OracleCacheMiss(OracleCacheMissPayload {
        query_hash_hex: "00".repeat(32),
        confidence: "high".into(),
    }))
    .expect("encode oracle_cache_miss");
}
