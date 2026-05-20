//! D-26: Reasoning template archive dashboard surface (extension).
//!
//! Adds two additive read-only Tauri commands that compose existing D-1 /
//! D-4 / D-5 surfaces into views consumed by the new
//! `LineageGraphView` + `ArchiveExplorerView` Svelte components.
//!
//! * [`get_archive_lineage`] — walks the [`ArchiveChain`] backwards from a
//!   requested template id, returning every ancestor node in parent-first
//!   order plus depth + root id. The chain itself is the producer (D-1);
//!   this command only reads.
//! * [`get_template_archive_combined`] — single composite read for dashboard
//!   refresh. Returns recent archive entries + recent tripwire rows + active
//!   BAI arm rows so the explorer can render header summary without paying
//!   three round-trips to the backend.
//!
//! Additive — does not refactor `commands.rs` / `commands_d1.rs` /
//! `commands_d4.rs` / `commands_d5.rs`. No new SSB variant, no new producer.

use std::path::{Path, PathBuf};

use elai_core::paths::ELAI_DATA_DIR_NAME;
use elai_orchestrator::archive_chain::ArchiveChain;
use elai_orchestrator::reasoning_template::ReasoningTemplateDto;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands_d1::ArchiveRecentRow;
use crate::commands_d4::TripwireRow;
use crate::commands_d5::BaiArmRow;
use crate::state::AppState;

/// Default cap on lineage walk depth. Defends against accidental cycles
/// in malformed on-disk chains (verify_chain() catches the canonical
/// case but a defensive ceiling avoids unbounded loops in any future
/// regression).
const LINEAGE_MAX_DEPTH: usize = 1024;

/// One node in the lineage walk. Mirrors [`ArchiveRecentRow`] so the
/// frontend can reuse the same row renderer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineageNode {
    pub template: ReasoningTemplateDto,
    pub self_hash: String,
    pub parent_hash: Option<String>,
    pub promoted_at_ms: Option<i64>,
    pub retired_at_ms: Option<i64>,
    pub retirement_reason: Option<String>,
    /// 0 = the queried template itself; N = N-th ancestor.
    pub depth: usize,
}

/// Lineage walk result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchiveLineage {
    /// Parent-first: index 0 is the root, last index is the queried node.
    pub entries: Vec<LineageNode>,
    pub root_id: Option<String>,
    pub depth: usize,
}

/// Per-template summary for the explorer header list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateSummary {
    pub id: String,
    pub domain: String,
    pub pattern_kind: String,
    pub archive_status: String,
    pub content_hash_hex: String,
    pub parent_hash_hex: Option<String>,
    pub created_at_ms: i64,
    pub body_len: usize,
}

/// Tripwire summary surfaced alongside the archive list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TripwireSummary {
    pub seq: i64,
    pub ts: i64,
    pub kind: String,
    pub tripwire_kind: Option<String>,
    pub candidate_id: Option<String>,
    pub observed_value: Option<f64>,
    pub threshold: Option<f64>,
}

/// BAI arm summary surfaced alongside the archive list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArmSummary {
    pub template_id: String,
    pub alpha: f64,
    pub beta: f64,
    pub n_pulls: u32,
    pub mean_estimate: f64,
    pub was_promoted: bool,
}

/// Composite read returned by [`get_template_archive_combined`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombinedArchiveView {
    pub templates: Vec<TemplateSummary>,
    pub recent_tripwires: Vec<TripwireSummary>,
    pub bai_arms: Vec<ArmSummary>,
}

/// Canonical archive path relative to `project_root`. Mirrors
/// `commands_d1::archive_jsonl_path` but kept private here to avoid
/// adding a cross-module export.
fn archive_jsonl_path(project_root: &Path) -> PathBuf {
    project_root
        .join(ELAI_DATA_DIR_NAME)
        .join("archive")
        .join("reasoning-templates.jsonl")
}

/// Pure-Rust lineage walk. Public so the integration test can call it
/// without going through the Tauri command machinery.
///
/// Accepts the queried template id in either canonical-UUID hyphenated
/// form or simple-hex form (matching `commands_d1::get_reasoning_template_by_id`).
/// Returns an empty [`ArchiveLineage`] (root_id=None, depth=0, entries=[])
/// when the id is not found in the chain.
pub fn walk_lineage(chain: &ArchiveChain, template_id: &str) -> ArchiveLineage {
    let normalised = template_id.replace('-', "").to_lowercase();
    let entries = chain.entries();
    let start = match entries
        .iter()
        .position(|e| e.template.id.as_hex().to_lowercase() == normalised)
    {
        Some(idx) => idx,
        None => {
            return ArchiveLineage {
                entries: Vec::new(),
                root_id: None,
                depth: 0,
            };
        }
    };

    // Walk parent_hash links backwards. Build a hash -> index map so each
    // step is O(1).
    let mut hash_to_idx = std::collections::HashMap::with_capacity(entries.len());
    for (i, e) in entries.iter().enumerate() {
        hash_to_idx.insert(e.self_hash, i);
    }

    let mut walked: Vec<usize> = vec![start];
    let mut cursor = start;
    while walked.len() < LINEAGE_MAX_DEPTH {
        let parent_hash = match entries[cursor].parent_hash {
            Some(h) => h,
            None => break,
        };
        let parent_idx = match hash_to_idx.get(&parent_hash) {
            Some(i) => *i,
            None => break,
        };
        // Cycle defence (defensive — verify_chain rejects cycles).
        if walked.contains(&parent_idx) {
            break;
        }
        walked.push(parent_idx);
        cursor = parent_idx;
    }

    // Reverse so the root is at index 0, queried node at the end.
    walked.reverse();
    let depth = walked.len();
    let nodes: Vec<LineageNode> = walked
        .iter()
        .enumerate()
        .map(|(i, &idx)| {
            let e = &entries[idx];
            LineageNode {
                template: ReasoningTemplateDto::from(&e.template),
                self_hash: e.self_hash.as_hex(),
                parent_hash: e.parent_hash.map(|h| h.as_hex()),
                promoted_at_ms: e.promoted_at_ms,
                retired_at_ms: e.retired_at_ms,
                retirement_reason: e.retirement_reason.map(|r| r.as_str().to_string()),
                depth: i,
            }
        })
        .collect();
    let root_id = nodes.first().map(|n| n.template.id.clone());
    ArchiveLineage {
        entries: nodes,
        root_id,
        depth,
    }
}

/// Return the lineage (parent chain) of an archive template.
#[tauri::command]
pub async fn get_archive_lineage(
    state: State<'_, AppState>,
    template_id: String,
) -> Result<ArchiveLineage, String> {
    let project_root = state.project_root_snapshot();
    let path = archive_jsonl_path(&project_root);
    let chain = ArchiveChain::load(path).map_err(|e| e.to_string())?;
    chain.verify_chain().map_err(|e| e.to_string())?;
    Ok(walk_lineage(&chain, &template_id))
}

/// Combined composite read for dashboard refresh.
#[tauri::command]
pub async fn get_template_archive_combined(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<CombinedArchiveView, String> {
    let limit = limit.unwrap_or(50).min(500);
    let project_root = state.project_root_snapshot();

    // (1) Templates — read the archive chain directly. Empty chain (fresh
    // repo, missing file) is the truthful empty state.
    let path = archive_jsonl_path(&project_root);
    let chain = ArchiveChain::load(path).map_err(|e| e.to_string())?;
    chain.verify_chain().map_err(|e| e.to_string())?;
    let templates: Vec<TemplateSummary> = chain
        .recent(limit)
        .into_iter()
        .map(|entry| TemplateSummary {
            id: entry.template.id.as_hex(),
            domain: entry.template.domain.as_str().to_string(),
            pattern_kind: entry.template.pattern_kind.as_str().to_string(),
            archive_status: entry.template.archive_status.as_str().to_string(),
            content_hash_hex: entry.self_hash.as_hex(),
            parent_hash_hex: entry.parent_hash.map(|h| h.as_hex()),
            created_at_ms: entry.template.created_at_ms,
            body_len: entry.template.template_body.len(),
        })
        .collect();

    // (2) Tripwires — reuse the D-4 surface by reading SSB directly.
    let recent_tripwires: Vec<TripwireSummary> = match &state.viewer {
        Some(viewer) => {
            let mut combined: Vec<TripwireSummary> = Vec::new();
            for at in ["tripwire_fired", "template_auto_reverted"] {
                if let Ok(rows) = viewer.recent_rows_of_type(at, limit) {
                    for detail in rows {
                        let obj = match detail.payload_json.as_object() {
                            Some(o) => o,
                            None => continue,
                        };
                        combined.push(TripwireSummary {
                            seq: detail.seq,
                            ts: detail.ts,
                            kind: at.to_string(),
                            tripwire_kind: obj
                                .get("tripwire_kind")
                                .and_then(|v| v.as_str())
                                .map(String::from),
                            candidate_id: obj
                                .get("candidate_id")
                                .and_then(|v| v.as_str())
                                .map(String::from),
                            observed_value: obj.get("observed_value").and_then(|v| v.as_f64()),
                            threshold: obj.get("threshold").and_then(|v| v.as_f64()),
                        });
                    }
                }
            }
            // Newest first.
            combined.sort_by(|a, b| b.seq.cmp(&a.seq));
            combined.truncate(limit);
            combined
        }
        None => Vec::new(),
    };

    // (3) BAI arms — read bai_arm_updated rows directly so we share the
    // viewer's empty-state contract with commands_d5.
    let bai_arms: Vec<ArmSummary> = match &state.viewer {
        Some(viewer) => match viewer.recent_rows_of_type("bai_arm_updated", limit) {
            Ok(rows) => {
                let mut arms: Vec<ArmSummary> = rows
                    .into_iter()
                    .filter_map(|detail| {
                        let obj = detail.payload_json.as_object()?;
                        let template_id = obj
                            .get("template_id")
                            .and_then(|v| v.as_str())
                            .map(String::from)?;
                        let alpha = obj.get("alpha").and_then(|v| v.as_f64())?;
                        let beta = obj.get("beta").and_then(|v| v.as_f64())?;
                        let n_pulls = obj
                            .get("n_pulls")
                            .and_then(|v| v.as_u64())
                            .map(|n| n as u32)
                            .unwrap_or(0);
                        let was_promoted = obj
                            .get("was_promoted")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        let denom = alpha + beta;
                        let mean_estimate = if denom > 0.0 { alpha / denom } else { 0.0 };
                        Some(ArmSummary {
                            template_id,
                            alpha,
                            beta,
                            n_pulls,
                            mean_estimate,
                            was_promoted,
                        })
                    })
                    .collect();
                arms.sort_by(|a, b| {
                    b.mean_estimate
                        .partial_cmp(&a.mean_estimate)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                arms.truncate(limit);
                arms
            }
            Err(_) => Vec::new(),
        },
        None => Vec::new(),
    };

    // Inner DTOs intentionally NOT re-exported through ArchiveRecentRow /
    // TripwireRow / BaiArmRow to keep the composite's shape stable as the
    // D-1/D-4/D-5 row schemas evolve. (Suppresses unused-import warnings.)
    let _ = std::marker::PhantomData::<(ArchiveRecentRow, TripwireRow, BaiArmRow)>;

    Ok(CombinedArchiveView {
        templates,
        recent_tripwires,
        bai_arms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use elai_orchestrator::archive_chain::{ArchiveChain, ArchiveEntry};
    use elai_orchestrator::reasoning_template::{
        compute_content_hash, ArchiveStatus, Domain, PatternKind, Predicate, ReasoningTemplate,
        TemplateBody, TemplateId, VerifierKind,
    };

    fn mk_template(body: &str) -> ReasoningTemplate {
        let template_body = TemplateBody::try_new(body.to_string())
            .expect("template body fits the canonical bound");
        let content_hash = compute_content_hash(template_body.as_str(), None);
        ReasoningTemplate {
            id: TemplateId::new_v4(),
            domain: Domain::Code,
            pattern_kind: PatternKind::Decomposition,
            template_body,
            eligibility_predicate: Predicate::Always,
            verifier_required: VerifierKind::Test,
            content_hash,
            created_at_ms: 0,
            parent_id: None,
            archive_status: ArchiveStatus::Candidate,
        }
    }

    #[test]
    fn walk_lineage_5_entries_returns_depth_5_root_first() {
        let tmp = tempfile::tempdir().expect("tempdir for archive chain test");
        let path = tmp.path().join("reasoning-templates.jsonl");
        let mut chain = ArchiveChain::new(path);

        // Build a 5-entry chain: root -> e1 -> e2 -> e3 -> e4.
        let root_entry = ArchiveEntry::new_root(mk_template("root body"));
        let root_hash = root_entry.self_hash;
        chain.append(root_entry).expect("append root");
        let mut prev_hash = root_hash;
        for i in 1..5 {
            let child = ArchiveEntry::new_child(
                mk_template(&format!("child {i} body")),
                prev_hash,
            );
            prev_hash = child.self_hash;
            chain.append(child).expect("append child");
        }
        assert_eq!(chain.len(), 5);

        // Query the leaf (last appended entry).
        let leaf_id = chain.entries().last().expect("leaf").template.id.as_hex();
        let root_template_id = chain.entries().first().expect("root").template.id.as_hex();
        let lineage = walk_lineage(&chain, &leaf_id);
        assert_eq!(lineage.depth, 5, "depth must equal chain length");
        assert_eq!(lineage.entries.len(), 5);
        assert_eq!(
            lineage.entries[0].self_hash,
            root_hash.as_hex(),
            "entries[0] must be the root"
        );
        assert_eq!(
            lineage.entries.last().expect("leaf node").template.id,
            leaf_id,
            "entries.last() must be the queried leaf"
        );
        // Parent-first order: each entry's parent_hash must equal the
        // previous entry's self_hash.
        for w in lineage.entries.windows(2) {
            assert_eq!(
                w[1].parent_hash.as_deref(),
                Some(w[0].self_hash.as_str()),
                "parent-first order broken"
            );
        }
        // Depth indices monotonic from 0..5.
        for (i, node) in lineage.entries.iter().enumerate() {
            assert_eq!(node.depth, i);
        }
        assert_eq!(lineage.root_id.as_deref(), Some(root_template_id.as_str()));
    }

    #[test]
    fn walk_lineage_unknown_id_returns_empty() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("reasoning-templates.jsonl");
        let chain = ArchiveChain::new(path);
        let lineage = walk_lineage(&chain, "deadbeef");
        assert_eq!(lineage.depth, 0);
        assert!(lineage.entries.is_empty());
        assert!(lineage.root_id.is_none());
    }
}
