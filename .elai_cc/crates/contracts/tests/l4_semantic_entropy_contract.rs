use std::cell::Cell;
use std::collections::HashMap;

use elai_orchestrator::claim::extract::{ByteSpan, ClaimKind, ClaimRef, ClaimSet, ExtractedClaim};
use elai_orchestrator::claim::risk::{PredictorKind, PredictorSource, RiskTier};
use elai_orchestrator::claim::roundtrip::{score_claim_risk_wired, score_claim_risk_wired_with_samples};
use elai_orchestrator::claim::semantic_entropy::SemanticEntropyPredictor;
use elai_memory::{NliJudge, NliLabel, NliVerdict};

fn set(resolved: &[bool]) -> ClaimSet {
    ClaimSet::from_claims(vec![ExtractedClaim {
        kind: ClaimKind::Behavior,
        span: ByteSpan { start: 3, end: 9 },
        refs: resolved.iter().enumerate().map(|(index, &resolved)| ClaimRef {
            raw: format!("src/ref-{index}.rs"), resolved,
        }).collect(),
    }])
}

fn claim() -> ExtractedClaim {
    ExtractedClaim { kind: ClaimKind::Behavior, span: ByteSpan { start: 0, end: 1 }, refs: vec![] }
}

fn signal_fired<P: PredictorSource>(predictor: &P) -> bool {
    predictor.signals(&claim()).iter().any(|signal| signal.source == PredictorKind::SemanticEntropy && signal.fired)
}

struct ScriptedJudge {
    script: HashMap<(String, String), NliLabel>,
    calls: std::rc::Rc<Cell<usize>>,
}

impl ScriptedJudge {
    fn new(script: &[(&str, &str, NliLabel)]) -> Self {
        Self { script: script.iter().map(|(a, b, label)| ((a.to_string(), b.to_string()), *label)).collect(), calls: std::rc::Rc::new(Cell::new(0)) }
    }
}

impl NliJudge for ScriptedJudge {
    fn classify(&self, premise: &str, hypothesis: &str) -> Result<NliVerdict, elai_memory::MemoryError> {
        self.calls.set(self.calls.get() + 1);
        Ok(NliVerdict { label: self.script.get(&(premise.to_string(), hypothesis.to_string())).copied().unwrap_or(NliLabel::Neutral), confidence: 1.0 })
    }
}

#[test]
fn fewer_than_two_samples_is_present_but_unfired() {
    for samples in [vec![], vec!["a".to_string()]] {
        let judge = ScriptedJudge::new(&[]);
        let calls = judge.calls.clone();
        let predictor = SemanticEntropyPredictor::with_judge(&samples, judge, 0.5);
        assert_eq!(predictor.entropy(), 0.0);
        assert!(!predictor.measured());
        assert!(!signal_fired(&predictor));
        assert_eq!(calls.get(), 0);
    }
}

#[test]
fn exact_duplicate_samples_never_reach_the_judge() {
    let samples = vec!["same".to_string(); 3];
    let judge = ScriptedJudge::new(&[]);
    let calls = judge.calls.clone();
    let predictor = SemanticEntropyPredictor::with_judge(&samples, judge, 0.5);
    assert_eq!(predictor.cluster_count(), 1);
    assert_eq!(predictor.entropy(), 0.0);
    assert!(!signal_fired(&predictor));
    assert_eq!(calls.get(), 0);
}

#[test]
fn three_unrelated_samples_reach_maximum_entropy() {
    let samples = ["a", "b", "c"].into_iter().map(str::to_string).collect::<Vec<_>>();
    let judge = ScriptedJudge::new(&[]);
    let predictor = SemanticEntropyPredictor::with_judge(&samples, judge, 0.5);
    assert_eq!(predictor.cluster_count(), 3);
    // Normalized Shannon entropy over three equal clusters is 1.0 in real arithmetic but
    // 0.9999999999999998 in f64 (the ln(3) accumulation rounds down by one ULP).
    assert!((predictor.entropy() - 1.0).abs() < 1e-12);
    assert!(signal_fired(&predictor));
}

#[test]
fn entailment_must_be_bidirectional() {
    let samples = vec!["a".to_string(), "b".to_string()];
    let judge = ScriptedJudge::new(&[("a", "b", NliLabel::Entailment)]);
    let predictor = SemanticEntropyPredictor::with_judge(&samples, judge, 0.5);
    // A one-way implementation collapses this to one cluster and reds here.
    assert_eq!(predictor.cluster_count(), 2);
    assert!(signal_fired(&predictor));
}

#[test]
fn mutual_entailment_merges_into_one_cluster() {
    let samples = vec!["a".to_string(), "b".to_string()];
    let judge = ScriptedJudge::new(&[("a", "b", NliLabel::Entailment), ("b", "a", NliLabel::Entailment)]);
    let predictor = SemanticEntropyPredictor::with_judge(&samples, judge, 0.5);
    assert_eq!(predictor.cluster_count(), 1);
    assert_eq!(predictor.entropy(), 0.0);
    assert!(!signal_fired(&predictor));
}

#[test]
fn empty_samples_preserve_the_l3_tiers() {
    let claims = ClaimSet::from_claims(vec![
        ExtractedClaim {
            kind: ClaimKind::Behavior,
            span: ByteSpan { start: 0, end: 1 },
            refs: vec![ClaimRef { raw: "unresolved".into(), resolved: false }],
        },
        ExtractedClaim {
            kind: ClaimKind::Behavior,
            span: ByteSpan { start: 2, end: 3 },
            refs: vec![ClaimRef { raw: "resolved".into(), resolved: true }],
        },
    ]);
    let with_samples = score_claim_risk_wired_with_samples(&claims, "", &[]);
    let baseline = score_claim_risk_wired(&claims, "");
    let actual: Vec<RiskTier> = with_samples.iter().map(|risk| risk.tier()).collect();
    let expected: Vec<RiskTier> = baseline.iter().map(|risk| risk.tier()).collect();
    assert_eq!(actual, expected);
}
