//! B-23 Phase 4H Group B — log-driven self-modification proposal builder
//! (4H.4) and Class A / Class B routing.
//!
//! Reads SSB rows (reasoning_trace) for the configured agents and emits
//! [`SelfModProposal`] objects whose target_config_key is bound to a
//! concrete [`AllowedConfigKey`] variant rather than a free-form string.
//! Routing function dispatches Class A (DataTable) proposals to Layer 1A
//! and Class B (Parameter) proposals to the staged-eval path (Group D).
//!
//! Spec: docs/specs/guide/steps/phase-4H.txt section 4H.4.
//! Heuristic triggers stay as fallback only; log-driven proposals are the
//! primary mechanism (DGM section 2.4 self-referential alignment).

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::self_mod_layer1a::{AllowedConfigKey, DataTable, PipelineMode};
use crate::self_mod_types::{
    ProjectScope, ProposedChange, ReasoningTraceRef, SelfModProposal,
};
use crate::ssb::{ReasoningTracePayload, SsbActionType, SsbEntry, SsbError, SsbPayload, SsbReader};

// ---------------------------------------------------------------------------
// Routing
// ---------------------------------------------------------------------------

/// Class of the proposal — drives downstream routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalClass {
    /// Class A — DataTable proposal, routed to Layer 1A FPR validation.
    LayerOneA,
    /// Class B — Parameter proposal, routed to staged eval (10/50/200).
    StagedEval,
}

/// Route a proposal to its handler based on the variant of
/// `target_config_key`. Spec phase-4H.txt:554-555 routing test.
pub fn route_proposal(p: &SelfModProposal) -> ProposalClass {
    match p.target_config_key {
        AllowedConfigKey::DataTable(_) => ProposalClass::LayerOneA,
        AllowedConfigKey::Parameter(_) => ProposalClass::StagedEval,
    }
}

// ---------------------------------------------------------------------------
// Log-driven proposal engine
// ---------------------------------------------------------------------------

/// Errors emitted by the proposal engine.
#[derive(Debug, thiserror::Error)]
pub enum ProposalEngineError {
    #[error("ssb: {0}")]
    Ssb(#[from] SsbError),
    #[error("payload decode: {0}")]
    Decode(String),
}

/// Reads SSB reasoning_trace rows and emits log-driven proposals targeting
/// concrete [`AllowedConfigKey`] variants.
///
/// MVP heuristics (spec phase-4H.txt:540-555):
///   * Failed reasoning steps with `grounding_level == "UNGROUNDED"` or
///     `budget_remaining == 0` produce Class A proposals targeting the
///     `failure_taxonomy` data table (auto-grow categories from misses).
///   * Three or more such failures across a configured agent set within the
///     query window mark the run as a "stale-grounding pipeline" — emit an
///     additional Class B-style proposal targeting `freshness_half_life`
///     (parameter key) so the staged-eval path can probe it later.
pub struct ProposalEngine {
    reader: Arc<dyn SsbReader + Send + Sync>,
    agent_ids: Vec<String>,
    min_evidence: usize,
}

impl ProposalEngine {
    pub fn new(reader: Arc<dyn SsbReader + Send + Sync>, agent_ids: Vec<String>) -> Self {
        Self {
            reader,
            agent_ids,
            min_evidence: 1,
        }
    }

    pub fn with_min_evidence(mut self, n: usize) -> Self {
        self.min_evidence = n.max(1);
        self
    }

    /// Scan SSB reasoning_trace rows for the configured agent ids and
    /// produce zero or more proposals. Pure read — no SSB writes.
    pub async fn build_proposals(
        &self,
        pipeline_mode: PipelineMode,
    ) -> Result<Vec<SelfModProposal>, ProposalEngineError> {
        let mut all_traces: Vec<(SsbEntry, ReasoningTracePayload)> = Vec::new();
        for agent in &self.agent_ids {
            let entries = self.reader.query_reasoning_trace(agent)?;
            for entry in entries {
                let payload: SsbPayload = match rmp_serde::from_slice(&entry.payload) {
                    Ok(p) => p,
                    Err(e) => return Err(ProposalEngineError::Decode(e.to_string())),
                };
                if let SsbPayload::ReasoningTrace(p) = payload {
                    if is_failure_evidence(&p) {
                        all_traces.push((entry, p));
                    }
                }
            }
        }

        if all_traces.len() < self.min_evidence {
            return Ok(Vec::new());
        }

        let scope = ProjectScope {
            ecosystem: None,
            language: None,
            pipeline_mode: Some(pipeline_mode),
        };

        let evidence_seqs: Vec<String> = all_traces
            .iter()
            .map(|(e, _)| format!("seq:{}:{}", e.seq, SsbActionType::ReasoningTrace.as_sql_str()))
            .collect();

        let trace_refs: Vec<ReasoningTraceRef> = all_traces
            .iter()
            .map(|(_, p)| ReasoningTraceRef {
                agent_id: p.agent_id.clone(),
                phase: p.topic.clone(),
                grc_step_index: p.step_index as usize,
                step_type: p.step_type.clone(),
                failure_mode: failure_mode_label(p),
            })
            .collect();

        // Class A proposal — auto-grow failure_taxonomy with the most common
        // step_type seen in the failing window (data-table InsertRow).
        let dominant_step = dominant_step_type(&trace_refs);
        let row_payload = serde_json::json!({
            "category": dominant_step,
            "regex": format!("{dominant_step}_failure"),
            "first_seen_seq": all_traces.first().map(|(e, _)| e.seq).unwrap_or(0),
        });
        // Group B audit WARN fix: propagate JSON encode errors instead of
        // emitting an empty `row` Vec that Layer 1A would deserialize as
        // garbage.
        let row_bytes = serde_json::to_vec(&row_payload)
            .map_err(|e| ProposalEngineError::Decode(e.to_string()))?;

        let class_a = SelfModProposal {
            variant_id: synthetic_variant_id(pipeline_mode, "ftax", &evidence_seqs),
            parent_id: String::new(),
            target_config_key: AllowedConfigKey::DataTable(DataTable::FailureTaxonomy),
            proposed_change: ProposedChange::InsertRow {
                table: DataTable::FailureTaxonomy.as_str().to_string(),
                row: row_bytes,
            },
            ssb_evidence: evidence_seqs.clone(),
            reasoning_trace_evidence: trace_refs.clone(),
            rationale: format!(
                "log-driven: {n} ungrounded reasoning step(s) on dominant_type={dominant_step}",
                n = trace_refs.len()
            ),
            scope: Some(scope.clone()),
        };

        let mut out = vec![class_a];

        // Class B candidate — only when failure volume crosses the spec's
        // 3-occurrence threshold (phase-3F self-discovery rule).
        if trace_refs.len() >= 3 {
            let class_b = SelfModProposal {
                variant_id: synthetic_variant_id(pipeline_mode, "freshness", &evidence_seqs),
                parent_id: String::new(),
                target_config_key: AllowedConfigKey::Parameter("freshness_half_life".into()),
                proposed_change: ProposedChange::UpdateKey {
                    key: "freshness_half_life".into(),
                    old_value: "180".into(),
                    new_value: "120".into(),
                },
                ssb_evidence: evidence_seqs,
                reasoning_trace_evidence: trace_refs,
                rationale: "log-driven: stale-grounding cluster — propose freshness half-life cut".into(),
                scope: Some(scope),
            };
            out.push(class_b);
        }

        // Spec invariant: every proposal carries non-empty ssb_evidence.
        out.retain(|p| !p.ssb_evidence.is_empty());

        Ok(out)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn is_failure_evidence(p: &ReasoningTracePayload) -> bool {
    p.grounding_level.eq_ignore_ascii_case("UNGROUNDED")
        || p.budget_remaining == 0
        || p.content_summary.to_ascii_lowercase().contains("failed")
}

fn failure_mode_label(p: &ReasoningTracePayload) -> String {
    if p.grounding_level.eq_ignore_ascii_case("UNGROUNDED") {
        "ungrounded".into()
    } else if p.budget_remaining == 0 {
        "budget_exhausted".into()
    } else {
        "content_failed".into()
    }
}

/// Group B audit WARN fix: switched from `HashMap` (non-deterministic
/// iteration order, fails systemPatterns.md "BTreeMap not HashMap" rule)
/// to `BTreeMap`. On a tie the lexicographically-smallest step_type wins,
/// so the result is reproducible across processes.
fn dominant_step_type(refs: &[ReasoningTraceRef]) -> String {
    use std::collections::BTreeMap;
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for r in refs {
        *counts.entry(r.step_type.as_str()).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .max_by(|(a_key, a_count), (b_key, b_count)| {
            a_count
                .cmp(b_count)
                .then_with(|| b_key.cmp(a_key)) // tie-break: lexicographically smaller key wins
        })
        .map(|(k, _)| k.to_string())
        .unwrap_or_else(|| "unknown".into())
}

/// Deterministic synthetic id that prevents duplicate proposals when the
/// underlying evidence is identical. Uses blake3 over the evidence seqs +
/// pipeline mode + key prefix.
fn synthetic_variant_id(mode: PipelineMode, key_prefix: &str, evidence: &[String]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(key_prefix.as_bytes());
    hasher.update(b"|");
    hasher.update(mode.as_str().as_bytes());
    for e in evidence {
        hasher.update(b"|");
        hasher.update(e.as_bytes());
    }
    let hex = hasher.finalize().to_hex();
    format!("{key_prefix}-{}", &hex.as_str()[..16])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::self_mod_types::ReasoningTraceRef;

    fn rtref(step_type: &str) -> ReasoningTraceRef {
        ReasoningTraceRef {
            agent_id: "executor".into(),
            phase: "phase".into(),
            grc_step_index: 0,
            step_type: step_type.into(),
            failure_mode: "ungrounded".into(),
        }
    }

    #[test]
    fn dominant_step_type_picks_max() {
        let refs = vec![rtref("observe"), rtref("observe"), rtref("hypothesize")];
        assert_eq!(dominant_step_type(&refs), "observe");
    }

    #[test]
    fn synthetic_id_is_stable_for_same_evidence() {
        let a = synthetic_variant_id(PipelineMode::Surgical, "ftax", &["s1".into(), "s2".into()]);
        let b = synthetic_variant_id(PipelineMode::Surgical, "ftax", &["s1".into(), "s2".into()]);
        assert_eq!(a, b);
    }

    #[test]
    fn synthetic_id_changes_with_evidence() {
        let a = synthetic_variant_id(PipelineMode::Surgical, "ftax", &["s1".into()]);
        let b = synthetic_variant_id(PipelineMode::Surgical, "ftax", &["s2".into()]);
        assert_ne!(a, b);
    }

    #[test]
    fn failure_mode_label_matches_signal() {
        let mut p = ReasoningTracePayload {
            agent_id: "executor".into(),
            step_index: 0,
            step_type: "observe".into(),
            content_summary: "ok".into(),
            grounding_source: "src".into(),
            grounding_level: "UNGROUNDED".into(),
            confidence: 0.5,
            elapsed_ms: 1,
            topic: "phase".into(),
            budget_remaining: 5,
        };
        assert_eq!(failure_mode_label(&p), "ungrounded");
        p.grounding_level = "GROUNDED".into();
        p.budget_remaining = 0;
        assert_eq!(failure_mode_label(&p), "budget_exhausted");
        p.budget_remaining = 5;
        p.content_summary = "step FAILED on stale".into();
        assert_eq!(failure_mode_label(&p), "content_failed");
    }
}
