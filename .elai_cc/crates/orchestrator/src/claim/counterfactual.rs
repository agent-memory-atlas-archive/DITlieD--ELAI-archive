//! Deterministic counterfactual intervention planning and do-test adjudication.

use super::extract::{ClaimKind, ClaimRef, ExtractedClaim};
use super::risk::{PredictorKind, PredictorSignal, PredictorSource, RiskTier};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Intervention {
    target_ref: ClaimRef,
    kind: InterventionKind,
    cone: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InterventionKind {
    RevertOnlyA,
    DisableFlag,
    PatchOnlyFn,
    AddTrace,
    NotInterventional,
}

fn plan_intervention(claim: &ExtractedClaim, cone: &[String]) -> Intervention {
    let target = claim.refs.iter().find(|reference| {
        reference.resolved && cone.iter().any(|member| member == &reference.raw)
    });
    let Some(target_ref) = target else {
        return Intervention {
            target_ref: ClaimRef { raw: String::new(), resolved: false },
            kind: InterventionKind::NotInterventional,
            cone: cone.to_vec(),
        };
    };
    if !matches!(claim.kind, ClaimKind::Causal | ClaimKind::Behavior) {
        return Intervention {
            target_ref: target_ref.clone(),
            kind: InterventionKind::NotInterventional,
            cone: cone.to_vec(),
        };
    }
    let kind = if target_ref.raw.contains("flag") {
        InterventionKind::DisableFlag
    } else if target_ref.raw.contains("trace") {
        InterventionKind::AddTrace
    } else if target_ref.raw.contains("::") || target_ref.raw.ends_with("()") {
        InterventionKind::PatchOnlyFn
    } else {
        InterventionKind::RevertOnlyA
    };
    Intervention { target_ref: target_ref.clone(), kind, cone: cone.to_vec() }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TierGuard {
    tier: RiskTier,
    limit: Option<&'static str>,
}

fn tier_guard(claim: &ExtractedClaim) -> TierGuard {
    match claim.kind {
        ClaimKind::Dependency | ClaimKind::Reachability => TierGuard { tier: RiskTier::Medium, limit: None },
        ClaimKind::Causal | ClaimKind::Behavior if claim.refs.iter().any(|reference| reference.resolved) => {
            TierGuard { tier: RiskTier::High, limit: None }
        }
        _ => TierGuard { tier: RiskTier::High, limit: Some("runtime-trace") },
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct DoTestVerdict {
    moved: bool,
    delta: f64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CounterfactualPredictor {
    verdict: Option<DoTestVerdict>,
}

impl CounterfactualPredictor {
    pub fn new() -> Self { Self { verdict: None } }

    fn with_verdict(verdict: DoTestVerdict) -> Self { Self { verdict: Some(verdict) } }
}

impl PredictorSource for CounterfactualPredictor {
    fn signals(&self, claim: &ExtractedClaim) -> Vec<PredictorSignal> {
        // A do-test can support only an interventional claim. Static
        // dependency, reachability, and file claims must never acquire
        // causal support merely because an outcome moved.
        let interventional = matches!(claim.kind, ClaimKind::Causal | ClaimKind::Behavior);
        let fired = interventional && self.verdict.is_some_and(|verdict| verdict.moved);
        vec![PredictorSignal {
            source: PredictorKind::Counterfactual,
            value: if fired { 1.0 } else { 0.0 },
            fired,
        }]
    }
}

fn run_do_test(
    _claim: &ExtractedClaim,
    intervention: &Intervention,
    baseline_hash: &str,
    intervened_hash: &str,
) -> DoTestVerdict {
    let moved = baseline_hash != intervened_hash;
    DoTestVerdict { moved, delta: if moved { 1.0 } else { 0.0 } }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::claim::extract::ByteSpan;

    fn claim(kind: ClaimKind, raw: &str, resolved: bool) -> ExtractedClaim {
        ExtractedClaim {
            kind,
            span: ByteSpan { start: 0, end: 1 },
            refs: vec![ClaimRef { raw: raw.into(), resolved }],
        }
    }

    fn causal_claim() -> ExtractedClaim {
        claim(ClaimKind::Causal, "src/fix.rs", true)
    }

    fn fired(predictor: &CounterfactualPredictor, claim: &ExtractedClaim) -> bool {
        predictor
            .signals(claim)
            .iter()
            .any(|signal| signal.source == PredictorKind::Counterfactual && signal.fired)
    }

    #[test]
    fn true_fix_claim_is_causal_supported() {
        let claim = causal_claim();
        let intervention = plan_intervention(&claim, &["src/fix.rs".into()]);
        assert_eq!(intervention.kind, InterventionKind::RevertOnlyA);
        let verdict = run_do_test(&claim, &intervention, "baseline", "changed");
        assert!(verdict.moved);
        assert!(fired(&CounterfactualPredictor::with_verdict(verdict), &claim));
    }

    #[test]
    fn false_fix_claim_does_not_fire() {
        let claim = causal_claim();
        let intervention = plan_intervention(&claim, &["src/fix.rs".into()]);
        let verdict = run_do_test(&claim, &intervention, "same", "same");
        assert!(!verdict.moved);
        assert!(!fired(&CounterfactualPredictor::with_verdict(verdict), &claim));
    }

    #[test]
    fn unchanged_causal_outcome_is_not_causal() {
        let claim = causal_claim();
        let intervention = plan_intervention(&claim, &["src/fix.rs".into()]);
        let verdict = run_do_test(&claim, &intervention, "h", "h");
        assert!(!fired(&CounterfactualPredictor::with_verdict(verdict), &claim));
    }

    #[test]
    fn dependency_is_static_and_never_causal() {
        let claim = claim(ClaimKind::Dependency, "dep-b", true);
        let guard = tier_guard(&claim);
        assert_eq!(guard.tier, RiskTier::Medium);
        assert_eq!(guard.limit, None);
        let verdict = DoTestVerdict { moved: true, delta: 1.0 };
        assert!(!fired(&CounterfactualPredictor::with_verdict(verdict), &claim));
    }

    #[test]
    fn intervention_is_deterministic() {
        let claim = causal_claim();
        let cone = vec!["src/fix.rs".into()];
        assert_eq!(plan_intervention(&claim, &cone), plan_intervention(&claim, &cone));
    }

    #[test]
    fn non_interventional_is_runtime_trace_capped() {
        let claim = claim(ClaimKind::File, "README.md", true);
        let guard = tier_guard(&claim);
        assert_eq!(guard.tier, RiskTier::High);
        assert_eq!(guard.limit, Some("runtime-trace"));
    }
}
