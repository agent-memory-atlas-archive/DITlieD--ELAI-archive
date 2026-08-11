use elai_orchestrator::claim::extract::{
    extract_claims_with, ByteSpan, ClaimKind, ClaimRef, ClaimSet, ExtractedClaim,
};
use elai_orchestrator::claim::risk::{PredictorKind, RiskTier};
use elai_orchestrator::claim::resolve::WorkspaceRefResolver;
use elai_orchestrator::claim::roundtrip::{
    closed_set_from_resolved_refs, score_claim_risk_wired, RoundTripPredictor,
};
use tempfile::TempDir;

fn set(resolved: &[bool]) -> ClaimSet {
    ClaimSet::from_claims(vec![ExtractedClaim {
        kind: ClaimKind::Behavior,
        span: ByteSpan { start: 3, end: 9 },
        refs: resolved
            .iter()
            .enumerate()
            .map(|(index, &resolved)| ClaimRef {
                raw: format!("src/ref-{index}.rs"),
                resolved,
            })
            .collect(),
    }])
}

fn signal_fired(risk: &elai_orchestrator::claim::risk::ClaimRisk, source: PredictorKind) -> bool {
    risk.signals.iter().any(|signal| signal.source == source && signal.fired)
}

#[test]
fn all_resolved_is_low_and_roundtrip_does_not_fire() {
    let risk = &score_claim_risk_wired(&set(&[true, true]), "")[0];
    assert_eq!(risk.tier(), RiskTier::Low);
    assert!(!signal_fired(risk, PredictorKind::RoundTrip));
}

#[test]
fn all_unresolved_fires_both_and_moves_tier() {
    let risk = &score_claim_risk_wired(&set(&[false, false, false, false]), "")[0];
    assert!(signal_fired(risk, PredictorKind::RefsUnresolved));
    assert!(signal_fired(risk, PredictorKind::RoundTrip));
    assert!(matches!(risk.tier(), RiskTier::High | RiskTier::Critical));
}

#[test]
fn graded_residual_below_threshold_stays_medium() {
    let risk = &score_claim_risk_wired(&set(&[false, true, true, true]), "")[0];
    assert!(signal_fired(risk, PredictorKind::RefsUnresolved));
    assert!(!signal_fired(risk, PredictorKind::RoundTrip));
    assert_eq!(risk.tier(), RiskTier::Medium);
}

#[test]
fn refless_claim_returns_a_tier() {
    let risks = score_claim_risk_wired(&set(&[]), "");
    assert_eq!(risks.len(), 1);
    assert_eq!(risks[0].tier(), RiskTier::Low);
}


#[test]
fn outside_closed_set_citation_fires_roundtrip() {
    let text = "Behavior passes [2].";
    let claims = elai_orchestrator::claim::extract::extract_claims(text);
    let risks = score_claim_risk_wired(&claims, text);
    assert!(risks.iter().any(|risk| signal_fired(risk, PredictorKind::RoundTrip)));
    assert!(risks.iter().any(|risk| matches!(risk.tier(), RiskTier::Medium | RiskTier::High | RiskTier::Critical)));
}

#[test]
fn inside_closed_set_citation_stays_low() {
    let text = "Behavior passes [1].";
    let claims = ClaimSet::from_claims(vec![ExtractedClaim {
        kind: ClaimKind::Behavior,
        span: ByteSpan { start: 0, end: text.len() },
        refs: vec![ClaimRef { raw: "resolved".into(), resolved: true }],
    }]);
    let risks = score_claim_risk_wired(&claims, text);
    assert_eq!(risks.len(), 1);
    assert_eq!(risks[0].tier(), RiskTier::Low);
    assert!(!signal_fired(&risks[0], PredictorKind::RoundTrip));
}

#[test]
fn resolved_claim_without_citation_has_only_neutral_roundtrip() {
    let text = "Behavior passes.";
    let claims = ClaimSet::from_claims(vec![ExtractedClaim {
        kind: ClaimKind::Behavior,
        span: ByteSpan { start: 0, end: text.len() },
        refs: vec![ClaimRef { raw: "resolved".into(), resolved: true }],
    }]);
    let risks = score_claim_risk_wired(&claims, text);
    assert_eq!(risks[0].tier(), RiskTier::Low);
    assert_eq!(risks[0].signals.iter().filter(|s| s.source == PredictorKind::RoundTrip).count(), 1);
    assert!(!signal_fired(&risks[0], PredictorKind::RoundTrip));
}


fn build_fixture_dkg(project_root: &std::path::Path) {
    let src = project_root.join("src");
    std::fs::create_dir_all(&src).expect("create src/");
    std::fs::write(src.join("a.rs"), "pub fn elai_l3_probe_symbol() -> i32 { 7 }\n")
        .expect("write a.rs");

    let db_path = elai_core::paths::dkg_db_path(project_root);
    std::fs::create_dir_all(db_path.parent().expect("dkg db path has a parent"))
        .expect("create indexes dir");

    {
        let mut dkg = elai_knowledge::DkgEngine::new(&db_path, None, 0).expect("DkgEngine::new");
        dkg.build_graph(project_root).expect("dkg.build_graph");
    }

    // Must match the reader: resolve_symbol opens the cache with
    // elai_core::project_id_from_path, and SymbolGraphCache::new fails when no
    // symbol_graph_meta row exists for that exact project_id (cache.rs:205-210).
    let project_id = elai_core::project_id_from_path(project_root);
    elai_knowledge::symbol_graph::build_pipeline::build_symbol_graph(
        &project_id,
        project_root,
        &db_path,
        1_700_000_000_000,
    )
    .expect("build_symbol_graph");

    // Precondition, not the contract: prove the fixture really indexed the probe symbol.
    // Without this, a silent indexing failure reads as "the resolver is broken".
    let mut cache = elai_knowledge::symbol_graph::cache::SymbolGraphCache::new(
        &db_path,
        &project_id,
        elai_knowledge::symbol_graph::sharding::SYMBOL_FILE_LRU_SIZE,
    )
    .expect("fixture: SymbolGraphCache::new must open the graph the fixture just built");
    let index = cache
        .ensure_name_index()
        .expect("fixture: name index must load");
    assert!(
        index
            .get("elai_l3_probe_symbol")
            .is_some_and(|refs| !refs.is_empty()),
        "fixture: build_symbol_graph did not index elai_l3_probe_symbol; the fixture is broken, not the resolver"
    );
}

#[test]
fn real_symbol_resolves_through_the_dkg_and_scores_low_residual() {
    let dir = TempDir::new().expect("temp dir");
    build_fixture_dkg(dir.path());
    let resolver = WorkspaceRefResolver::new(dir.path());
    let text = "The probe is `elai_l3_probe_symbol()`.";
    let claims = extract_claims_with(text, &resolver);

    assert_eq!(claims.claims().len(), 1);
    let claim = &claims.claims()[0];
    assert_eq!(claim.kind, ClaimKind::Symbol);
    assert_eq!(claim.refs.len(), 1);
    assert!(claim.refs[0].resolved);
    assert_eq!(RoundTripPredictor::roundtrip_residual(claim), 0.0);

    let closed_set = closed_set_from_resolved_refs(&claims);
    assert_eq!(closed_set.anchors.len(), 1);
    assert!(closed_set.anchors[0]
        .source_uri
        .contains("elai_l3_probe_symbol"));

    let risks = score_claim_risk_wired(&claims, text);
    assert_eq!(risks[0].tier(), RiskTier::Low);
    assert!(!signal_fired(&risks[0], PredictorKind::RoundTrip));
}

#[test]
fn ghost_symbol_stays_unresolved_at_max_residual_and_outside_the_closed_set() {
    let dir = TempDir::new().expect("temp dir");
    build_fixture_dkg(dir.path());
    let resolver = WorkspaceRefResolver::new(dir.path());
    let text = "The ghost is `elai_l3_definitely_absent_symbol()`.";
    let claims = extract_claims_with(text, &resolver);

    assert_eq!(claims.claims().len(), 1);
    let claim = &claims.claims()[0];
    assert_eq!(claim.refs.len(), 1);
    assert!(!claim.refs[0].resolved);
    assert_eq!(RoundTripPredictor::roundtrip_residual(claim), 1.0);

    let risks = score_claim_risk_wired(&claims, text);
    assert!(signal_fired(&risks[0], PredictorKind::RoundTrip));
    assert_ne!(risks[0].tier(), RiskTier::Low);
    assert!(closed_set_from_resolved_refs(&claims).anchors.is_empty());
}

#[test]
fn resolver_is_fail_closed_when_the_dkg_is_absent() {
    let dir = TempDir::new().expect("temp dir");
    let resolver = WorkspaceRefResolver::new(dir.path());
    let text = "The probe is `elai_l3_probe_symbol()`.";
    let claims = extract_claims_with(text, &resolver);

    assert_eq!(claims.claims().len(), 1);
    let claim = &claims.claims()[0];
    assert_eq!(claim.refs.len(), 1);
    assert!(!claim.refs[0].resolved);
}
