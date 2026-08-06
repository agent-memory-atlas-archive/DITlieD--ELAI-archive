//! J-79 — signal-triggered grill interlace (Tier-1 OBSERVE + consent-gated OFFER).
//!
//! Landable rule-text: `.elai_cc/skills/grill/SKILL.md`.
//! HARD CONSTRAINT: never auto-RUN grill. Signal-triggered OFFER is allowed
//! and consent-gated. G1 return path closes the loop by (1) persisting
//! IntentPair artifacts for C-29, (2) clearing B-40 BLOCK findings on the
//! framed idea when contradictions resolve, (3) feeding pairs into
//! TransitionContext at the PLANNING->TESTING_FIRST gate.

use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::front_door::{FrontDoorLane, FrontDoorTier};
use crate::gate::RiskLevel;
use crate::idea_framing::{FramedIdea, Severity};
use crate::intent_pair::{IntentPair, IntentPairSet};
use crate::ssb::{IntentRoutedKind, IntentRoutedPayload, Ssb, SsbPayload};

/// Why a rung-1 grill offer fired. Tier-1 ships IdeaCriticBlock only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GrillOfferTrigger {
    IdeaCriticBlock { block_count: u32 },
}

/// Clarify ladder rung.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClarifyRung {
    Rung0ClarifyChip,
    Rung1OfferGrill {
        branch_count: usize,
        trigger: GrillOfferTrigger,
    },
}

/// Select rung from B-40 critic BLOCK count. None when zero blocks.
#[must_use]
pub fn select_rung_from_block_count(block_count: u32) -> Option<ClarifyRung> {
    if block_count == 0 {
        return None;
    }
    Some(ClarifyRung::Rung1OfferGrill {
        branch_count: block_count.max(1) as usize,
        trigger: GrillOfferTrigger::IdeaCriticBlock { block_count },
    })
}

/// One-line consent-gated offer. Never claims grill already ran.
#[must_use]
pub fn offer_line(rung: &ClarifyRung) -> String {
    match rung {
        ClarifyRung::Rung0ClarifyChip => String::new(),
        ClarifyRung::Rung1OfferGrill { branch_count, .. } => format!(
            "this looks ambiguous across {branch_count} branches — grill it before i plan ?"
        ),
    }
}

/// Structured grill output consumed by the G1 return path.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct GrillResolvedDecisions {
    #[serde(default)]
    pub resolved_branches: Vec<(String, String)>,
    #[serde(default)]
    pub glossary: Vec<(String, String)>,
    #[serde(default)]
    pub contradictions_resolved: bool,
}

/// Map resolved branches to an IntentPairSet (structured input for C-29).
#[must_use]
pub fn intent_pairs_from_resolved(d: &GrillResolvedDecisions) -> IntentPairSet {
    let mut set = IntentPairSet::new();
    for (branch_label, resolution) in &d.resolved_branches {
        if branch_label.trim().is_empty() || resolution.trim().is_empty() {
            continue;
        }
        if let Ok(pair) = IntentPair::new(branch_label.as_str(), resolution.as_str(), "grill") {
            set.push(pair);
        }
    }
    set
}

/// True when contradictions resolved and at least one branch present.
#[must_use]
pub fn critic_blocks_cleared(d: &GrillResolvedDecisions) -> bool {
    d.contradictions_resolved && !d.resolved_branches.is_empty()
}

/// Clear B-40 BLOCK findings on the framed idea (deterministic re-eval input).
/// TD-3: only called after structured grill_resolved says contradictions_resolved.
pub fn clear_critic_blocks_on_framed(framed: &mut FramedIdea) {
    if let Some(round) = framed.critic_rounds.last_mut() {
        round
            .findings
            .retain(|f| f.severity != Severity::Block);
    }
}

async fn emit_intent_routed_best_effort(ssb: &Arc<Ssb>, kind: IntentRoutedKind) {
    let ts_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let payload = SsbPayload::IntentRouted(IntentRoutedPayload {
        lane: FrontDoorLane::Task,
        tier: FrontDoorTier::T1,
        confidence: 0,
        arg_spans: Vec::new(),
        kind,
        risk: RiskLevel::Medium,
        ts_ms,
    });
    if let Err(e) = ssb
        .write_payload("grill_interlace", 0, payload, Vec::new())
        .await
    {
        tracing::warn!(
            error = %e,
            "grill_interlace: IntentRouted emit failed (routing unaffected)"
        );
    }
}

/// If framed is NOT converged, offer grill (Clarified journal). Never auto-run.
pub async fn maybe_offer_grill_on_critic_block(
    ssb: &Arc<Ssb>,
    framed: &FramedIdea,
) -> Option<String> {
    if framed.is_converged() {
        return None;
    }
    let block_count = framed
        .critic_rounds
        .last()
        .map(|r| r.block_count())
        .unwrap_or(0);
    let rung = select_rung_from_block_count(block_count)?;
    let line = offer_line(&rung);
    if line.is_empty() {
        return None;
    }
    emit_intent_routed_best_effort(ssb, IntentRoutedKind::Clarified).await;
    Some(line)
}

/// Load optional `grill_resolved.toml` from a session dir.
pub fn load_grill_resolved(session_dir: &Path) -> Option<GrillResolvedDecisions> {
    let path = session_dir.join("grill_resolved.toml");
    let text = std::fs::read_to_string(&path).ok()?;
    toml::from_str(&text).ok()
}

/// Load IntentPairSet for the C-29 IntentPairsGate from a session dir.
/// Prefers re-deriving from grill_resolved.toml (source of truth).
#[must_use]
pub fn load_intent_pairs_for_gate(session_dir: &Path) -> Option<IntentPairSet> {
    let resolved = load_grill_resolved(session_dir)?;
    let set = intent_pairs_from_resolved(&resolved);
    if set.is_empty() {
        None
    } else {
        Some(set)
    }
}

/// Deterministic applied marker (TD-3: structured fields only).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GrillReturnApplied {
    pub contradictions_resolved: bool,
    pub branch_count: usize,
    pub pairs_len: usize,
}

/// Outcome of [`apply_grill_return_path`].
#[derive(Debug, Clone)]
pub struct GrillReturnEffect {
    pub pairs_len: usize,
    pub blocks_cleared: bool,
    pub pairs_path: std::path::PathBuf,
    pub applied_path: Option<std::path::PathBuf>,
    /// Pairs ready for C-29 gate re-evaluation.
    pub pairs: IntentPairSet,
}

/// Closed-loop G1 return: persist pairs + applied marker + IntentRouted Confirmed
/// + clear B-40 blocks on framed when contradictions resolved.
pub async fn apply_grill_return_path(
    ssb: &Arc<Ssb>,
    session_dir: &Path,
    framed: Option<&mut FramedIdea>,
) -> Option<GrillReturnEffect> {
    let resolved = load_grill_resolved(session_dir)?;
    let pairs = intent_pairs_from_resolved(&resolved);
    let pairs_len = pairs.len();
    let branch_count = resolved.resolved_branches.len();
    let blocks_cleared = critic_blocks_cleared(&resolved);

    let pairs_path = session_dir.join("grill_return_intent_pairs.toml");
    let mut pairs_doc = String::from("# grill return intent pairs (J-79)\n");
    for (i, p) in pairs.pairs.iter().enumerate() {
        pairs_doc.push_str(&format!(
            "\n[[pairs]]\nindex = {i}\nintent = {}\nassertion = {}\n",
            toml_escape(&p.natural_language_intent),
            toml_escape(&p.concrete_test_assertion),
        ));
    }
    pairs_doc.push_str(&format!("\npairs_len = {pairs_len}\n"));
    if let Err(e) = std::fs::write(&pairs_path, pairs_doc) {
        tracing::warn!(
            path = %pairs_path.display(),
            error = %e,
            "grill_interlace: failed to persist grill_return_intent_pairs.toml"
        );
    }

    let mut applied_path = None;
    if blocks_cleared {
        let marker = GrillReturnApplied {
            contradictions_resolved: resolved.contradictions_resolved,
            branch_count,
            pairs_len,
        };
        let path = session_dir.join("grill_return_applied.toml");
        if let Ok(text) = toml::to_string_pretty(&marker) {
            if std::fs::write(&path, text).is_ok() {
                applied_path = Some(path);
            }
        }
        // Close B-40 loop: re-evaluate framed without BLOCK findings.
        if let Some(f) = framed {
            clear_critic_blocks_on_framed(f);
        }
    }

    emit_intent_routed_best_effort(ssb, IntentRoutedKind::Confirmed).await;

    Some(GrillReturnEffect {
        pairs_len,
        blocks_cleared,
        pairs_path,
        applied_path,
        pairs,
    })
}

fn toml_escape(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_zero_blocks_none() {
        assert!(select_rung_from_block_count(0).is_none());
    }

    #[test]
    fn select_two_blocks_rung1() {
        let rung = select_rung_from_block_count(2).expect("rung1");
        match rung {
            ClarifyRung::Rung1OfferGrill {
                branch_count,
                trigger: GrillOfferTrigger::IdeaCriticBlock { block_count },
            } => {
                assert_eq!(branch_count, 2);
                assert_eq!(block_count, 2);
            }
            other => panic!("expected Rung1, got {other:?}"),
        }
    }

    #[test]
    fn offer_line_mentions_grill_not_ran() {
        let rung = select_rung_from_block_count(3).unwrap();
        let line = offer_line(&rung);
        assert!(line.to_lowercase().contains("grill"));
        assert!(!line.to_lowercase().contains("already ran"));
    }

    #[test]
    fn intent_pairs_from_three_branches() {
        let d = GrillResolvedDecisions {
            resolved_branches: vec![
                ("a".into(), "do a".into()),
                ("b".into(), "do b".into()),
                ("c".into(), "do c".into()),
            ],
            glossary: vec![],
            contradictions_resolved: true,
        };
        assert!(intent_pairs_from_resolved(&d).len() >= 3);
        assert!(critic_blocks_cleared(&d));
    }

    #[test]
    fn critic_blocks_cleared_requires_flag() {
        let d = GrillResolvedDecisions {
            resolved_branches: vec![("a".into(), "do a".into())],
            glossary: vec![],
            contradictions_resolved: false,
        };
        assert!(!critic_blocks_cleared(&d));
    }

    #[test]
    fn load_grill_resolved_missing() {
        let dir = tempfile::tempdir().unwrap();
        assert!(load_grill_resolved(dir.path()).is_none());
        assert!(load_intent_pairs_for_gate(dir.path()).is_none());
    }
}
