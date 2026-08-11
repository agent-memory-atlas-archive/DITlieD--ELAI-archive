//! Graded evidence round-trip residual predictor (L-3).

use super::extract::{ClaimKind, ClaimSet, ExtractedClaim};
use super::risk::{
    score_claim_risk_with, ClaimRisk, PredictorKind, PredictorSignal, PredictorSource,
};
use crate::checker_registry::{CheckerRegistry, Checker};
use crate::citation_prebind::{
    check_citation_in_set, extract_citation_ids, freeze_anchors, CitationCheckOutcome,
    EvidenceAnchorSet, RankedEvidence,
};
use crate::claim::{Claim, ClaimType};
use crate::runtime_config::CheckerRuntimeConfig;

/// ELAI-declared seed boundary, not a published constant.
const DEFAULT_RESIDUAL_FIRES_AT: f64 = 0.5;
/// ELAI-declared seed boundary, not a published constant.
const CHECKED_KIND_CITATION_FIRES_AT: f64 = 0.25;

/// Predictor for the fraction of evidence references that failed to round-trip.
#[derive(Debug, Clone, Copy)]
pub struct RoundTripPredictor {
    residual_fires_at: f64,
}

impl Default for RoundTripPredictor {
    fn default() -> Self {
        Self::new(DEFAULT_RESIDUAL_FIRES_AT)
    }
}

impl RoundTripPredictor {
    pub fn new(residual_fires_at: f64) -> Self {
        Self { residual_fires_at }
    }

    pub fn roundtrip_residual(claim: &ExtractedClaim) -> f64 {
        if claim.refs.is_empty() {
            return 0.0;
        }
        let unresolved_refs = claim.refs.iter().filter(|reference| !reference.resolved).count();
        unresolved_refs as f64 / claim.refs.len() as f64
    }
}

impl PredictorSource for RoundTripPredictor {
    fn signals(&self, claim: &ExtractedClaim) -> Vec<PredictorSignal> {
        let residual = Self::roundtrip_residual(claim);
        vec![PredictorSignal {
            source: PredictorKind::RoundTrip,
            value: residual,
            fired: residual >= self.residual_fires_at,
        }]
    }
}

/// Maps extraction kinds to the D-10 routing taxonomy. The `Api` and
/// `Dependency` checker-backed rows follow the `compile-rust` computational
/// and `registry-check` factual/procedural entries in checkers.toml.
pub fn claim_type_for_kind(kind: ClaimKind) -> ClaimType {
    match kind {
        ClaimKind::File | ClaimKind::Symbol | ClaimKind::Dependency => ClaimType::Factual,
        ClaimKind::Api => ClaimType::Computational,
        ClaimKind::Behavior => ClaimType::Procedural,
        ClaimKind::Causal | ClaimKind::Reachability => ClaimType::Speculative,
    }
}

/// Returns the deployed per-kind checker id. File and Symbol use the
/// workspace resolver; Reachability is slice 3 and Causal belongs to L-5.
pub fn checker_id_for_kind(kind: ClaimKind) -> Option<&'static str> {
    match kind {
        ClaimKind::Api => Some("compile-rust"),
        ClaimKind::Dependency => Some("registry-check"),
        ClaimKind::File | ClaimKind::Symbol | ClaimKind::Behavior | ClaimKind::Causal
        | ClaimKind::Reachability => None,
    }
}

/// A deployed checker makes citation re-verification cheap, so a smaller
/// ghost-citation share is already worth firing on. The kind keeps the
/// per-kind seam explicit.
pub fn citation_fires_at(kind: ClaimKind, checker_applies: bool) -> f64 {
    match (kind, checker_applies) {
        (_, true) => CHECKED_KIND_CITATION_FIRES_AT,
        (_, false) => DEFAULT_RESIDUAL_FIRES_AT,
    }
}

/// Citation residual predictor over the response's frozen, closed evidence set.
pub struct ClosedSetCitationPredictor<'a> {
    text: &'a str,
    anchors: EvidenceAnchorSet,
    registry: Option<&'a CheckerRegistry>,
}

impl<'a> ClosedSetCitationPredictor<'a> {
    pub fn new(
        text: &'a str,
        anchors: EvidenceAnchorSet,
        registry: Option<&'a CheckerRegistry>,
    ) -> Self {
        Self { text, anchors, registry }
    }
}

/// Freezes the resolved references from this response as its closed evidence
/// set: an id outside it cites evidence that never round-tripped to the workspace.
pub fn closed_set_from_resolved_refs(set: &ClaimSet) -> EvidenceAnchorSet {
    let mut ranked = Vec::new();
    for claim in set.claims() {
        for reference in &claim.refs {
            if reference.resolved {
                let rank = ranked.len() as u32;
                ranked.push(RankedEvidence {
                    rank,
                    excerpt: reference.raw.clone(),
                    source_uri: reference.raw.clone(),
                    retrieved_at_ms: 0,
                    source_tier: 0,
                });
            }
        }
    }
    freeze_anchors(&ranked, 0)
}

impl PredictorSource for ClosedSetCitationPredictor<'_> {
    fn signals(&self, claim: &ExtractedClaim) -> Vec<PredictorSignal> {
        let Some(excerpt) = self.text.get(claim.span.start..claim.span.end) else {
            return Vec::new();
        };
        let ids = extract_citation_ids(excerpt);
        if ids.is_empty() {
            return Vec::new();
        }
        let failed = ids
            .iter()
            .filter(|&&id| matches!(
                check_citation_in_set(&self.anchors, id, "").outcome,
                CitationCheckOutcome::Ghost | CitationCheckOutcome::Stale
            ))
            .count();
        let residual = failed as f64 / ids.len() as f64;
        let checker_applies = self.registry.and_then(|registry| {
            checker_id_for_kind(claim.kind)
                .and_then(|id| registry.lookup(id))
                .map(|checker| {
                    let view = Claim {
                        claim_id: format!("l3-{}-{}-{}", claim.kind.as_str(), claim.span.start, claim.span.end),
                        text: excerpt.to_owned(),
                        claim_type: claim_type_for_kind(claim.kind),
                        evidence: Vec::new(),
                        ts_ms: 0,
                    };
                    checker.applies_to(&view)
                })
        }).unwrap_or(false);
        vec![PredictorSignal {
            source: PredictorKind::RoundTrip,
            value: residual,
            fired: residual >= citation_fires_at(claim.kind, checker_applies),
        }]
    }
}

/// Production entry point for claim risk scoring with both round-trip signals.
pub fn score_claim_risk_wired(set: &ClaimSet, text: &str) -> Vec<ClaimRisk> {
    let predictor = RoundTripPredictor::default();
    let registry = load_checker_registry();
    let citations = ClosedSetCitationPredictor::new(
        text,
        closed_set_from_resolved_refs(set),
        registry.as_ref(),
    );
    score_claim_risk_with(set, &[&predictor, &citations])
}

/// Production entry point for claim risk scoring with the round-trip sources plus the
/// L-4 semantic-entropy source over the answer samples the caller collected. With fewer
/// than two samples the entropy signal is present but unfired (entropy NOT MEASURED).
pub fn score_claim_risk_wired_with_samples(
    set: &ClaimSet,
    text: &str,
    samples: &[String],
) -> Vec<ClaimRisk> {
    let predictor = RoundTripPredictor::default();
    let registry = load_checker_registry();
    let citations = ClosedSetCitationPredictor::new(
        text,
        closed_set_from_resolved_refs(set),
        registry.as_ref(),
    );
    let entropy = super::semantic_entropy::SemanticEntropyPredictor::new(samples);
    score_claim_risk_with(set, &[&predictor, &citations, &entropy])
}

fn load_checker_registry() -> Option<CheckerRegistry> {
    if !CheckerRuntimeConfig::is_enabled() {
        return None;
    }
    let root = match std::env::current_dir() {
        Ok(root) => root,
        Err(error) => {
            tracing::warn!(%error, "failed to determine checker registry root");
            return None;
        }
    };
    match CheckerRegistry::load_from_toml(&root) {
        Ok(registry) => Some(registry),
        Err(error) => {
            tracing::warn!(%error, "failed to load checker registry");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker_registry::{CheckerKind, CheckerResult};
    use crate::claim::extract::{ByteSpan, ClaimRef};
    use std::pin::Pin;

    fn claim(refs: Vec<bool>) -> ExtractedClaim {
        ExtractedClaim {
            kind: ClaimKind::Behavior,
            span: ByteSpan { start: 0, end: 1 },
            refs: refs.into_iter().enumerate().map(|(index, resolved)| ClaimRef {
                raw: format!("ref-{index}"), resolved,
            }).collect(),
        }
    }

    struct AppliesChecker;
    impl Checker for AppliesChecker {
        fn kind(&self) -> CheckerKind { CheckerKind::CompileRust }
        fn applies_to(&self, _claim: &Claim) -> bool { true }
        fn check<'a>(&'a self, _claim: &'a Claim) -> Pin<Box<dyn std::future::Future<Output = CheckerResult> + Send + 'a>> {
            Box::pin(async { CheckerResult::Pass })
        }
    }

    fn predictor<'a>(text: &'a str, kind: ClaimKind, registry: Option<&'a CheckerRegistry>) -> ClosedSetCitationPredictor<'a> {
        let set = ClaimSet::from_claims(vec![ExtractedClaim { kind, span: ByteSpan { start: 0, end: text.len() }, refs: vec![ClaimRef { raw: "resolved".into(), resolved: true }] }]);
        ClosedSetCitationPredictor::new(text, closed_set_from_resolved_refs(&set), registry)
    }

    #[test]
    fn fully_resolved_claim_has_zero_residual_and_does_not_fire() {
        let claim = claim(vec![true, true]);
        assert_eq!(RoundTripPredictor::roundtrip_residual(&claim), 0.0);
        assert!(!RoundTripPredictor::default().signals(&claim)[0].fired);
    }
    #[test]
    fn all_unresolved_claim_has_unit_residual_and_fires() {
        let claim = claim(vec![false, false]);
        assert_eq!(RoundTripPredictor::roundtrip_residual(&claim), 1.0);
        assert!(RoundTripPredictor::default().signals(&claim)[0].fired);
    }
    #[test]
    fn one_of_four_unresolved_has_quarter_residual_and_does_not_fire() {
        let claim = claim(vec![false, true, true, true]);
        assert_eq!(RoundTripPredictor::roundtrip_residual(&claim), 0.25);
        assert!(!RoundTripPredictor::default().signals(&claim)[0].fired);
    }
    #[test]
    fn refless_claim_has_zero_residual_without_panicking() {
        let claim = claim(Vec::new());
        assert_eq!(RoundTripPredictor::roundtrip_residual(&claim), 0.0);
        assert!(!RoundTripPredictor::default().signals(&claim)[0].fired);
    }
    // The shipped `extract_citation_ids` dedups and sorts, so the residual
    // denominator is the count of DISTINCT cited ids, not raw marker count.
    #[test]
    fn ghost_boundary_and_checked_kind_boundary() {
        let text = "[1] [2] [3] [9]";
        let set = ClaimSet::from_claims(vec![ExtractedClaim {
            kind: ClaimKind::Behavior,
            span: ByteSpan { start: 0, end: text.len() },
            refs: vec![
                ClaimRef { raw: "r1".into(), resolved: true },
                ClaimRef { raw: "r2".into(), resolved: true },
                ClaimRef { raw: "r3".into(), resolved: true },
            ],
        }]);
        let anchors = closed_set_from_resolved_refs(&set);
        let no_registry = ClosedSetCitationPredictor::new(text, anchors.clone(), None);
        let behavior_claim = ExtractedClaim { kind: ClaimKind::Behavior, span: ByteSpan { start: 0, end: text.len() }, refs: vec![] };
        assert!(!no_registry.signals(&behavior_claim)[0].fired);
        let mut registry = CheckerRegistry::new();
        registry.register("compile-rust", Box::new(AppliesChecker));
        let checked = ClosedSetCitationPredictor::new(text, anchors, Some(&registry));
        let api_claim = ExtractedClaim { kind: ClaimKind::Api, span: ByteSpan { start: 0, end: text.len() }, refs: vec![] };
        assert!(checked.signals(&api_claim)[0].fired);
        let one_ghost = predictor("[9]", ClaimKind::Behavior, None);
        let one_claim = ExtractedClaim { kind: ClaimKind::Behavior, span: ByteSpan { start: 0, end: 3 }, refs: vec![] };
        assert!(one_ghost.signals(&one_claim)[0].fired);
    }
    #[test]
    fn citation_free_claim_has_no_citation_signal() {
        let p = predictor("plain", ClaimKind::Behavior, None);
        assert!(p.signals(&ExtractedClaim { kind: ClaimKind::Behavior, span: ByteSpan { start: 0, end: 5 }, refs: vec![] }).is_empty());
    }
    #[test]
    fn closed_set_numbers_only_resolved_refs() {
        let set = ClaimSet::from_claims(vec![claim(vec![false, true, true])]);
        let anchors = closed_set_from_resolved_refs(&set);
        assert_eq!(anchors.anchors.len(), 2);
        assert_eq!(anchors.anchors[0].id, 1);
        assert_eq!(anchors.anchors[1].id, 2);
    }
}
