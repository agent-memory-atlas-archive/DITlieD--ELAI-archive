use crate::error::MemoryError;
use crate::semantic_contradiction::{
    DeterministicNliJudge, NliJudge, NliLabel, NliVerdict,
};

#[derive(Debug)]
pub struct BinaryGroundingResult {
    pub zero_span_verdicts: Vec<BinaryGroundingVerdict>,
    pub supported_count: usize,
    pub unsupported_count: usize,
}

#[derive(Debug)]
pub struct BinaryGroundingVerdict {
    pub claim_text: String,
    pub claim_index: usize,
    pub label: NliLabel,
    pub confidence: f32,
}

pub struct BinaryGroundingJudge {
    inner: DeterministicNliJudge,
}

impl BinaryGroundingJudge {
    pub fn new() -> Self {
        BinaryGroundingJudge {
            inner: DeterministicNliJudge::new(),
        }
    }
}

impl NliJudge for BinaryGroundingJudge {
    fn classify(&self, premise: &str, hypothesis: &str) -> Result<NliVerdict, MemoryError> {
        let verdict = self.inner.classify(premise, hypothesis)?;
        let mapped = match verdict.label {
            NliLabel::Entailment => NliVerdict {
                label: NliLabel::Entailment,
                confidence: 1.0,
            },
            NliLabel::Contradiction | NliLabel::Neutral => NliVerdict {
                label: NliLabel::Neutral,
                confidence: 1.0,
            },
        };
        Ok(mapped)
    }
}

pub fn judge_zero_span_claims(
    claims: &[(&str, usize)],
    evidence_spans: &[&str],
) -> BinaryGroundingResult {
    if evidence_spans.is_empty() || claims.is_empty() {
        return BinaryGroundingResult {
            zero_span_verdicts: Vec::new(),
            supported_count: 0,
            unsupported_count: 0,
        };
    }

    let concatenated_evidence: String = evidence_spans.join(" ");
    let judge = BinaryGroundingJudge::new();

    let mut zero_span_verdicts = Vec::with_capacity(claims.len());
    let mut supported_count = 0usize;
    let mut unsupported_count = 0usize;

    for &(claim_text, claim_index) in claims {
        let verdict = judge.classify(claim_text, &concatenated_evidence)
            .unwrap_or(NliVerdict {
                label: NliLabel::Neutral,
                confidence: 1.0,
            });

        match verdict.label {
            NliLabel::Entailment => supported_count += 1,
            _ => unsupported_count += 1,
        }

        zero_span_verdicts.push(BinaryGroundingVerdict {
            claim_text: claim_text.to_owned(),
            claim_index,
            label: verdict.label,
            confidence: verdict.confidence,
        });
    }

    BinaryGroundingResult {
        zero_span_verdicts,
        supported_count,
        unsupported_count,
    }
}
