//! Semantic-entropy predictor over multiple answer samples.
//!
//! GD-5: same-model resampling cannot catch a consistently shared wrong belief.
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
use super::risk::{PredictorKind, PredictorSignal, PredictorSource};

/// ELAI-declared seed boundary, not a published constant.
const DEFAULT_ENTROPY_FIRES_AT: f64 = 0.5;

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
}
