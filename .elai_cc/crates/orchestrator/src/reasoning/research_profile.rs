//! L-9 slice 2: research grounding producer.
//!
//! Composes the shipped claim-extraction, risk-scoring, and checker-registry
//! primitives into a single pass that adjudicates derivation lines through the
//! symbolic checker gate and writes one `ClaimRiskScored` SSB row. The model
//! only *proposes*; only an explicit checker `Pass` may assert (TD-3 / HG-4).

use crate::checker_registry::{Checker, CheckerKind, CheckerRegistry};
use crate::claim::extract::{extract_claims, extract_claims_with};
use crate::claim::roundtrip::score_claim_risk_wired_with_samples;
use crate::claim::{Claim, ClaimType};
use crate::runtime_config::CheckerRuntimeConfig;
use crate::ssb::{
    ClaimRiskScoredKind, ClaimRiskScoredPayload, Ssb, SsbPayload,
};

/// L-9: per-claim abstention reason. Only a symbolic-checker PASS may assert;
/// everything else abstains (TD-3: the checker adjudicates, the model only
/// proposes. HG-4: an unverified derivation is never asserted as proven).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbstainReason {
    CheckerGateOff,
    CheckerMissing,
    CheckerFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResearchVerdict {
    Asserted,
    Abstained(AbstainReason),
}

#[derive(Debug, Clone)]
pub struct ResearchClaimVerdict {
    pub subject: String,
    pub verdict: ResearchVerdict,
}

#[derive(Debug, Clone, Default)]
pub struct ResearchGroundingOutcome {
    pub verdicts: Vec<ResearchClaimVerdict>,
    pub row_written: bool,
}

/// L-9: run the research grounding pass over one agent reply.
///
/// Extracts claims, scores risk with the shipped wired entry point, selects
/// derivation lines, adjudicates each through the symbolic checker registry,
/// and writes one `ClaimRiskScored` SSB row when there is anything to record.
/// An empty reply with no claims and no derivations writes NO row.
pub async fn research_grounding_pass(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    reply: &str,
    candidates: &[String],
) -> Result<ResearchGroundingOutcome, crate::ssb::SsbError> {
    // 1. Extract claims exactly as the production task-runner caller does.
    let set = if let Some(paper) = crate::claim::resolve::paper_store_resolver() {
        extract_claims_with(reply, paper)
    } else if let Some(ws) = crate::claim::resolve::workspace_resolver() {
        extract_claims_with(reply, ws)
    } else {
        extract_claims(reply)
    };

    // 2. Entropy samples: use candidates when non-empty, else the single sample.
    let samples: Vec<String> = if !candidates.is_empty() {
        candidates.to_vec()
    } else {
        vec![reply.to_string()]
    };

    // 3. Score with the shipped wired entry point.
    let claim_risks = score_claim_risk_wired_with_samples(&set, reply, &samples);

    // 4. Select derivation lines.
    let deriv_lines = derivation_lines(reply);

    // 5. Adjudicate each derivation line through the checker registry.
    let registry = load_checker_registry();
    let mut verdicts: Vec<ResearchClaimVerdict> = Vec::with_capacity(deriv_lines.len());
    for line in &deriv_lines {
        let verdict = adjudicate_line(line, registry.as_ref()).await;
        verdicts.push(ResearchClaimVerdict {
            subject: (*line).to_string(),
            verdict,
        });
    }

    // 6. Fold the row.
    let folded = fold_grounding_row(&set, &claim_risks, &verdicts);

    // 7. Write ONE row when there is anything to record.
    let mut row_written = false;
    if !set.is_empty() || !verdicts.is_empty() {
        let payload = SsbPayload::ClaimRiskScored(ClaimRiskScoredPayload {
            kind: folded.kind,
            low_count: folded.low_count,
            medium_count: folded.medium_count,
            high_count: folded.high_count,
            critical_count: folded.critical_count,
            predictors: folded.predictors,
            consult: folded.consult,
            unresolved_ref_count: folded.unresolved_ref_count,
            claim_count: folded.claim_count,
            claim_digests: folded.claim_digests,
        });
        ssb.write_payload(agent_id, superstep, payload, Vec::new()).await?;
        row_written = true;
    }

    Ok(ResearchGroundingOutcome { verdicts, row_written })
}

/// Env-gated checker registry loader. Mirrors the private
/// `load_checker_registry()` in `claim/roundtrip.rs`: returns `None` when
/// `CheckerRuntimeConfig::is_enabled()` is false.
fn load_checker_registry() -> Option<CheckerRegistry> {
    if !CheckerRuntimeConfig::is_enabled() {
        return None;
    }
    let root = match std::env::current_dir() {
        Ok(root) => root,
        Err(error) => {
            eprintln!("research_profile: failed to determine checker registry root: {error}");
            return None;
        }
    };
    match CheckerRegistry::load_from_toml(&root) {
        Ok(registry) => Some(registry),
        Err(error) => {
            eprintln!("research_profile: failed to load checker registry: {error}");
            None
        }
    }
}

/// A trimmed line qualifies as a derivation when it contains `'='` AND has at
/// least one ASCII digit on each side of the first `'='`. Cap at 16 lines so a
/// long reply cannot blow up the pass.
fn derivation_lines(reply: &str) -> Vec<&str> {
    const CAP: usize = 16;
    let mut out: Vec<&str> = Vec::new();
    for line in reply.lines() {
        let trimmed = line.trim();
        if let Some(eq_pos) = trimmed.find('=') {
            let left = &trimmed[..eq_pos];
            let right = &trimmed[eq_pos + 1..];
            let left_has_digit = left.chars().any(|c| c.is_ascii_digit());
            let right_has_digit = right.chars().any(|c| c.is_ascii_digit());
            if left_has_digit && right_has_digit {
                out.push(trimmed);
                if out.len() >= CAP {
                    break;
                }
            }
        }
    }
    out
}

/// Adjudicate one derivation line through the checker registry. Fail-closed:
/// there is no code path where the model's own text produces `Asserted`.
async fn adjudicate_line(line: &str, registry: Option<&CheckerRegistry>) -> ResearchVerdict {
    let Some(registry) = registry else {
        return ResearchVerdict::Abstained(AbstainReason::CheckerGateOff);
    };

    // Find a math/symbolic checker in the registry.
    let math_checker = registry.entries_sorted().into_iter().find_map(|(id, kind)| {
        if is_math_symbolic_kind(kind) {
            registry.lookup(&id).map(|c| (id, c))
        } else {
            None
        }
    });

    let Some((_id, checker)) = math_checker else {
        return ResearchVerdict::Abstained(AbstainReason::CheckerMissing);
    };

    let claim = Claim {
        claim_id: format!("l9-derivation-{}", blake3::hash(line.as_bytes()).to_hex()),
        text: line.to_string(),
        claim_type: ClaimType::Computational,
        evidence: Vec::new(),
        ts_ms: 0,
    };

    if !checker.applies_to(&claim) {
        return ResearchVerdict::Abstained(AbstainReason::CheckerMissing);
    }

    // Run the checker. Any error, timeout, or non-pass abstains. The checker
    // trait returns a pinned future; we await it directly. A panic inside the
    // future would propagate as a runtime panic (fail-closed at the caller).
    let checker_result = checker.check(&claim).await;
    if checker_result.is_pass() {
        ResearchVerdict::Asserted
    } else {
        ResearchVerdict::Abstained(AbstainReason::CheckerFailed)
    }
}

/// True for checker kinds that are math/symbolic: Sympy, Z3, NumericRange.
fn is_math_symbolic_kind(kind: CheckerKind) -> bool {
    matches!(
        kind,
        CheckerKind::Sympy | CheckerKind::Z3 | CheckerKind::NumericRange
    )
}

/// The folded row, unit-testable without an SSB.
#[derive(Debug, Clone, PartialEq, Eq)]
struct GroundingRow {
    kind: ClaimRiskScoredKind,
    low_count: usize,
    medium_count: usize,
    high_count: usize,
    critical_count: usize,
    predictors: Vec<crate::claim::risk::PredictorKind>,
    consult: Vec<crate::claim::risk::PredictorKind>,
    unresolved_ref_count: usize,
    claim_count: usize,
    claim_digests: Vec<[u8; 32]>,
}

/// Fold the row. Mirrors the tier fold at `task_runner.rs:2050-2069`, then
/// applies the L-9 abstention rule: every `Abstained` verdict adds 1 to
/// `high_count` and appends one digest to `claim_digests`.
fn fold_grounding_row(
    set: &crate::claim::extract::ClaimSet,
    claim_risks: &[crate::claim::risk::ClaimRisk],
    verdicts: &[ResearchClaimVerdict],
) -> GroundingRow {
    let mut low_count = 0usize;
    let mut medium_count = 0usize;
    let mut high_count = 0usize;
    let mut critical_count = 0usize;
    let mut predictors: Vec<crate::claim::risk::PredictorKind> = Vec::new();
    let mut consult: Vec<crate::claim::risk::PredictorKind> = Vec::new();
    let mut claim_digests: Vec<[u8; 32]> = Vec::new();

    for risk in claim_risks {
        match risk.tier() {
            crate::claim::risk::RiskTier::Low => low_count += 1,
            crate::claim::risk::RiskTier::Medium => medium_count += 1,
            crate::claim::risk::RiskTier::High => high_count += 1,
            crate::claim::risk::RiskTier::Critical => critical_count += 1,
        }
        for signal in &risk.signals {
            if signal.fired && !predictors.contains(&signal.source) {
                predictors.push(signal.source);
            }
        }
        for p in &risk.consult {
            if !consult.contains(p) {
                consult.push(*p);
            }
        }
        claim_digests.push(risk.claim_digest);
    }

    // L-9 abstention rule: every Abstained verdict adds 1 to high_count and
    // appends one derivation digest.
    for v in verdicts {
        if let ResearchVerdict::Abstained(_) = v.verdict {
            high_count += 1;
            let mut hasher = blake3::Hasher::new();
            hasher.update(b"l9-derivation");
            hasher.update(v.subject.as_bytes());
            let digest = *hasher.finalize().as_bytes();
            claim_digests.push(digest);
        }
    }

    let kind = if critical_count > 0 {
        ClaimRiskScoredKind::CriticalRisk
    } else if high_count > 0 {
        ClaimRiskScoredKind::HighRisk
    } else if medium_count > 0 {
        ClaimRiskScoredKind::MediumRisk
    } else {
        ClaimRiskScoredKind::LowRisk
    };

    let unresolved_ref_count = set.unresolved_ref_count();
    let claim_count = claim_digests.len();

    GroundingRow {
        kind,
        low_count,
        medium_count,
        high_count,
        critical_count,
        predictors,
        consult,
        unresolved_ref_count,
        claim_count,
        claim_digests,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derivation_lines_selects_qualified_only() {
        let reply = "The answer is clear.\n2 + 2 = 5\nThis has no digits on the left = 42\nx = 3 + 4";
        let lines = derivation_lines(reply);
        // "2 + 2 = 5" qualifies (digits both sides). "x = 3 + 4" does NOT
        // (no digit on the left of '='). The prose line has no '='.
        assert!(lines.contains(&"2 + 2 = 5"));
        assert!(!lines.iter().any(|l| l.starts_with("This has no digits")));
        assert!(!lines.iter().any(|l| l.starts_with("x = 3")));
    }

    #[tokio::test]
    async fn checker_gate_off_yields_abstained() {
        // With the checker gate OFF (ELAI_CHECKER_ENABLE unset), a reply
        // holding one derivation yields exactly one Abstained(CheckerGateOff).
        // We test adjudicate_line directly with None registry.
        let line = "2 + 2 = 5";
        let verdict = adjudicate_line(line, None).await;
        assert_eq!(verdict, ResearchVerdict::Abstained(AbstainReason::CheckerGateOff));
    }

    #[test]
    fn fold_grounding_row_abstained_increments_high() {
        let set = crate::claim::extract::extract_claims("");
        let claim_risks: Vec<crate::claim::risk::ClaimRisk> = Vec::new();
        let verdicts = vec![ResearchClaimVerdict {
            subject: "2 + 2 = 5".to_string(),
            verdict: ResearchVerdict::Abstained(AbstainReason::CheckerGateOff),
        }];
        let row = fold_grounding_row(&set, &claim_risks, &verdicts);
        assert_eq!(row.high_count, 1);
        assert_ne!(row.kind, ClaimRiskScoredKind::LowRisk);
        assert_eq!(row.claim_count, row.claim_digests.len());
        assert_eq!(row.claim_count, 1);
    }
}
