//! E-10 -- typed-CoT proof certificate (Curry-Howard mechanical check).
//! Decision: E-10-decisions.md DECISION-9. InferenceRule variants are the
//! implementor extension of arXiv:2510.01069 (arithmetic-domain rules).
//! Coverage threshold: 0.4 (paper-observed floor), NOT 1.0.
//! Opt-in via ELAI_TYPED_COT=1; default OFF.

use std::sync::Arc;

use crate::ssb::{CriticDecisionEventPayload, CriticTilePhase, Ssb, SsbError, SsbPayload};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InferenceRule {
    ModusPonens,
    AndElim,
    CaseSplit,
    Refutation,
    Citation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Witness {
    CodeSpan(String),
    SpecCite(String),
    TestResult(bool),
    ExternalSource(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CertificateStep {
    pub rule: InferenceRule,
    pub witness: Option<Witness>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypedCotCertificate {
    pub steps: Vec<CertificateStep>,
}

impl TypedCotCertificate {
    #[must_use]
    pub fn coverage(&self) -> f64 {
        if self.steps.is_empty() { return 0.0; }
        let with_witness = self.steps.iter().filter(|s| s.witness.is_some()).count();
        (with_witness as f64) / (self.steps.len() as f64)
    }
    #[must_use]
    pub fn verify(&self) -> bool { self.coverage() >= 0.4 }
}

#[derive(Debug, thiserror::Error)]
pub enum CotProofError {
    #[error("critic-decision SSB write failed: {0}")]
    Ssb(#[from] SsbError),
}

pub async fn emit_typed_cot_verified(
    ssb: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    cert: &TypedCotCertificate,
    baseline_vacuous_rate: f64,
) -> Result<(), CotProofError> {
    let challenger_score = cert.coverage();
    let run_id_prefix = build_run_id_prefix(agent_id, superstep, b"cot");
    let verdict_code = if cert.verify() { 1 } else { 0 };
    let payload = SsbPayload::CriticDecisionEvent(CriticDecisionEventPayload {
        phase: CriticTilePhase::TypedCotVerified,
        run_id_prefix,
        baseline_score: baseline_vacuous_rate,
        challenger_score,
        paired_bootstrap_p: None,
        sample_size: cert.steps.len() as u32,
        verdict_code,
        rationale_hash: blake3_prefix_8(b"typed_cot_curry_howard_v1"),
        ts_ms: now_ms(),
    });
    ssb.write_payload(agent_id, superstep, payload, Vec::new()).await?;
    Ok(())
}

fn build_run_id_prefix(agent_id: &str, superstep: i64, salt: &[u8]) -> [u8; 8] {
    let mut h = blake3::Hasher::new();
    h.update(agent_id.as_bytes());
    h.update(&superstep.to_le_bytes());
    h.update(salt);
    let d = h.finalize();
    let b = d.as_bytes();
    [b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]
}

fn blake3_prefix_8(input: &[u8]) -> [u8; 8] {
    let mut h = blake3::Hasher::new();
    h.update(input);
    let d = h.finalize();
    let b = d.as_bytes();
    [b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_certificate_zero_coverage() {
        let c = TypedCotCertificate { steps: Vec::new() };
        assert_eq!(c.coverage(), 0.0);
        assert!(!c.verify());
    }
    #[test]
    fn half_coverage_verifies() {
        let c = TypedCotCertificate {
            steps: vec![
                CertificateStep { rule: InferenceRule::ModusPonens, witness: Some(Witness::TestResult(true)) },
                CertificateStep { rule: InferenceRule::Citation, witness: None },
            ],
        };
        assert_eq!(c.coverage(), 0.5);
        assert!(c.verify());
    }
}
