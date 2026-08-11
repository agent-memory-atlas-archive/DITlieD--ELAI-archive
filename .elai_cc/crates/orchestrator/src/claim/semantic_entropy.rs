//! Semantic-entropy predictor over multiple answer samples.
//!
//! GD-5: same-model resampling cannot catch a consistently shared wrong belief.
//! A deterministic sampler returns identical samples, so resampling can legitimately produce one cluster and an unfired signal; that is the degrade, not agreement.
//! A deterministic (greedy) sampler returns N identical samples: one cluster,
//! entropy 0, and the signal never fires. That is a real degrade, not a
//! measurement of agreement. Prompt mutation to manufacture diversity is
//! BANNED: every sample must answer the SAME prompt or the measure is corrupted.
//! `value == 0.0` with `fired == false` and fewer than two samples means
//! "entropy NOT MEASURED", not "zero entropy measured"; the slice-3 receipt
//! will separate the two. The fire boundary is an ELAI-declared seed boundary
//! re-measured at L-10 (ID-5), mirroring the round-trip boundary comment.

use elai_memory::{DeterministicNliJudge, NliJudge, NliLabel};

use super::extract::ExtractedClaim;
use super::risk::{expensive_predictors_for, ClaimRisk, PredictorKind, PredictorSignal, PredictorSource};
use crate::ipc_accept::{AgentRpcServer, InferOutcome};

/// ELAI-declared seed boundary, not a published constant.
const DEFAULT_ENTROPY_FIRES_AT: f64 = 0.5;

/// Seed constants, ELAI-declared, not published values. J-84 owns the budgeted N later.
pub const N_RESAMPLE: usize = 3;
const RESAMPLE_MAX_TOKENS: u32 = 512;
const RESAMPLE_ENABLE_ENV: &str = "ELAI_SEMANTIC_ENTROPY_RESAMPLE";

/// Default-OFF gate, following the search-ladder flag idiom.
#[must_use]
pub fn semantic_entropy_resample_enabled() -> bool {
    match std::env::var(RESAMPLE_ENABLE_ENV) {
        Ok(raw) => matches!(raw.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"),
        Err(_) => false,
    }
}

/// The D-13 consultation gate: only High or Critical risks consult semantic entropy.
#[must_use]
pub fn resample_is_indicated(risks: &[ClaimRisk]) -> bool {
    risks.iter().any(|risk| {
        expensive_predictors_for(risk.tier()).contains(&PredictorKind::SemanticEntropy)
    })
}

/// Collect at most `N_RESAMPLE - 1` extra answers to the same prompt.
pub async fn collect_extra_samples(
    enabled: bool,
    risks: &[ClaimRisk],
    rpc: &(dyn AgentRpcServer + Send + Sync),
    prompt: &str,
) -> Vec<String> {
    if !enabled {
        return Vec::new();
    }
    if !resample_is_indicated(risks) {
        return Vec::new();
    }

    let mut samples = Vec::with_capacity(N_RESAMPLE.saturating_sub(1));
    for _ in 0..N_RESAMPLE - 1 {
        match rpc.handle_infer_request(prompt.to_string(), RESAMPLE_MAX_TOKENS).await {
            InferOutcome::Response(answer) => samples.push(answer),
            InferOutcome::Failed(reason) => {
                tracing::warn!(%reason, "semantic-entropy resample failed");
            }
        }
    }
    samples
}

pub struct SemanticEntropyPredictor<'a, J: NliJudge = DeterministicNliJudge> {
    /// Held for the slice-2 bounded resample; slice 1 only needs the count.
    #[allow(dead_code)]
    samples: &'a [String],
    judge: J,
    fires_at: f64,
    entropy: f64,
    sample_count: usize,
    cluster_count: usize,
}

impl<'a> SemanticEntropyPredictor<'a, DeterministicNliJudge> {
    pub fn new(samples: &'a [String]) -> Self {
        Self::with_judge(samples, DeterministicNliJudge::new(), DEFAULT_ENTROPY_FIRES_AT)
    }
}

impl<'a, J: NliJudge> SemanticEntropyPredictor<'a, J> {
    pub fn with_judge(samples: &'a [String], judge: J, fires_at: f64) -> Self {
        let (entropy, cluster_count) = measure(samples, &judge);
        Self {
            samples,
            judge,
            fires_at,
            entropy,
            sample_count: samples.len(),
            cluster_count,
        }
    }

    pub fn entropy(&self) -> f64 { self.entropy }
    pub fn cluster_count(&self) -> usize { self.cluster_count }
    pub fn sample_count(&self) -> usize { self.sample_count }
    pub fn measured(&self) -> bool { self.sample_count >= 2 }
}

impl<'a, J: NliJudge> PredictorSource for SemanticEntropyPredictor<'a, J> {
    fn signals(&self, _claim: &ExtractedClaim) -> Vec<PredictorSignal> {
        vec![PredictorSignal {
            source: PredictorKind::SemanticEntropy,
            value: self.entropy,
            fired: self.measured() && self.entropy >= self.fires_at,
        }]
    }
}

fn measure<J: NliJudge>(samples: &[String], judge: &J) -> (f64, usize) {
    let n = samples.len();
    // This guard must precede all judge work: the shipped production path has one sample.
    if n < 2 {
        return (0.0, n.min(1));
    }

    let mut groups: Vec<(blake3::Hash, &str, usize)> = Vec::new();
    for sample in samples {
        let trimmed = sample.trim();
        let mut hasher = blake3::Hasher::new();
        hasher.update(trimmed.as_bytes());
        let digest = hasher.finalize();
        if let Some(group) = groups.iter_mut().find(|group| group.0 == digest) {
            group.2 += 1;
        } else {
            groups.push((digest, trimmed, 1));
        }
    }
    if groups.len() == 1 {
        return (0.0, 1);
    }

    let mut clusters: Vec<(&str, usize)> = Vec::new();
    for (_, candidate, weight) in groups {
        let mut joined = false;
        for cluster in &mut clusters {
            let first = judge
                .classify(cluster.0, candidate)
                .map(|verdict| verdict.label == NliLabel::Entailment)
                .unwrap_or(false);
            // A classify error is treated as NOT entailment, a safe degrade.
            if first {
                let second = judge
                    .classify(candidate, cluster.0)
                    .map(|verdict| verdict.label == NliLabel::Entailment)
                    .unwrap_or(false);
                if second {
                    cluster.1 += weight;
                    joined = true;
                    break;
                }
            }
        }
        if !joined {
            clusters.push((candidate, weight));
        }
    }

    let denominator = n as f64;
    let mut entropy = clusters.iter().fold(0.0, |sum, (_, weight)| {
        let probability = *weight as f64 / denominator;
        sum - probability * probability.ln()
    });
    entropy /= denominator.ln();
    (entropy.clamp(0.0, 1.0), clusters.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[derive(Default)]
    struct CountingJudge(Cell<usize>);

    impl NliJudge for CountingJudge {
        fn classify(&self, _: &str, _: &str) -> Result<elai_memory::NliVerdict, elai_memory::MemoryError> {
            self.0.set(self.0.get() + 1);
            Ok(elai_memory::NliVerdict { label: NliLabel::Neutral, confidence: 1.0 })
        }
    }

    fn claim() -> ExtractedClaim {
        ExtractedClaim { kind: super::super::extract::ClaimKind::Behavior, span: super::super::extract::ByteSpan { start: 0, end: 1 }, refs: vec![] }
    }

    #[test]
    fn fewer_than_two_is_unmeasured() {
        let judge = CountingJudge::default();
        let samples = vec![String::from("a")];
        let predictor = SemanticEntropyPredictor::with_judge(&samples, judge, 0.5);
        assert_eq!(predictor.entropy(), 0.0);
        assert_eq!(predictor.cluster_count(), 1);
        assert!(!predictor.measured());
        assert!(!predictor.signals(&claim())[0].fired);
    }

    #[test]
    fn identical_samples_do_not_call_judge() {
        let judge = CountingJudge::default();
        let samples = vec![String::from("a"); 3];
        let predictor = SemanticEntropyPredictor::with_judge(&samples, judge, 0.5);
        assert_eq!(predictor.entropy(), 0.0);
        assert_eq!(predictor.cluster_count(), 1);
        assert_eq!(predictor.judge.0.get(), 0);
    }

    #[test]
    fn neutral_distinct_samples_have_maximum_entropy() {
        let judge = CountingJudge::default();
        let samples = vec![String::from("a"), String::from("b"), String::from("c")];
        let predictor = SemanticEntropyPredictor::with_judge(&samples, judge, 0.5);
        // Normalized Shannon entropy over three equal clusters is 1.0 in real arithmetic but
        // 0.9999999999999998 in f64 (the ln(3) accumulation rounds down by one ULP).
        assert!((predictor.entropy() - 1.0).abs() < 1e-12);
        assert_eq!(predictor.cluster_count(), 3);
        assert!(predictor.signals(&claim())[0].fired);
    }
    fn critical_risks() -> Vec<ClaimRisk> {
        let set = super::super::extract::ClaimSet::from_claims(vec![super::super::extract::ExtractedClaim {
            kind: super::super::extract::ClaimKind::Behavior,
            span: super::super::extract::ByteSpan { start: 0, end: 1 },
            refs: vec![super::super::extract::ClaimRef { raw: "src/nope.rs".into(), resolved: false }],
        }]);
        super::super::roundtrip::score_claim_risk_wired(&set, "")
    }

    fn low_risks() -> Vec<ClaimRisk> {
        let set = super::super::extract::ClaimSet::from_claims(vec![super::super::extract::ExtractedClaim {
            kind: super::super::extract::ClaimKind::Behavior,
            span: super::super::extract::ByteSpan { start: 0, end: 1 },
            refs: vec![super::super::extract::ClaimRef { raw: "src/nope.rs".into(), resolved: true }],
        }]);
        super::super::roundtrip::score_claim_risk_wired(&set, "")
    }

    #[tokio::test]
    async fn resample_disabled_makes_no_infer_call() {
        let server = crate::ipc_accept::test_support::ScriptedAgentRpcServer::new("answer");
        assert!(collect_extra_samples(false, &critical_risks(), &server, "prompt").await.is_empty());
        assert_eq!(server.infer_calls.load(std::sync::atomic::Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn low_tier_first_pass_makes_no_infer_call() {
        let server = crate::ipc_accept::test_support::ScriptedAgentRpcServer::new("answer");
        assert!(collect_extra_samples(true, &low_risks(), &server, "prompt").await.is_empty());
        assert_eq!(server.infer_calls.load(std::sync::atomic::Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn indicated_resample_collects_exactly_two_extra_samples() {
        let server = crate::ipc_accept::test_support::ScriptedAgentRpcServer::new("answer");
        let samples = collect_extra_samples(true, &critical_risks(), &server, "prompt").await;
        assert_eq!(samples.len(), 2);
        assert_eq!(server.infer_calls.load(std::sync::atomic::Ordering::Relaxed), 2);
    }

    #[test]
    fn resamples_that_disagree_with_the_answer_fire() {
        // The scripted resample text differs from the original answer, so the pre-pass leaves two
        // clusters with weights 1 and 2: entropy is 0.5793801642856949, above the 0.5 boundary.
        // The all-identical degrade is covered by identical_samples_do_not_call_judge.
        let samples = vec!["answer".to_string(), "canned".to_string(), "canned".to_string()];
        let predictor = SemanticEntropyPredictor::new(&samples);
        assert_eq!(predictor.cluster_count(), 2);
        assert!((predictor.entropy() - 0.5793801642856949).abs() < 1e-12);
        assert!(predictor.signals(&claim())[0].fired);
    }

    #[test]
    fn enable_flag_defaults_off() {
        assert!(!semantic_entropy_resample_enabled());
    }

}
