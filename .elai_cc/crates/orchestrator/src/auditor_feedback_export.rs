//! E-8 task 3: structured auditor feedback export for the GEPA text-
//! feedback channel.
//!
//! Per arXiv:2507.19457 (GEPA, ICLR 2026 Oral): the reflective prompt
//! evolution loop consumes free-text feedback at each iteration; the
//! richer the feedback the faster GEPA converges. ELAI exports the
//! auditor's structured verdict (verdict + per-finding reasons +
//! optional MAST mode + failing-test names + raw block messages) as a
//! JSON envelope the Python optimizer reads from disk.
//!
//! Determinism rule (plan task 9 + DECISION-9): the Python optimizer
//! has READ-ONLY access to `.elai_cc/optimization/feedback/`; this
//! Rust module is the WRITER. The Python side never mutates SSB rows
//! or the archive directly -- the only Python -> Rust path is the
//! `OptimizedTemplate` value type consumed by `optimizer_admit`.
//!
//! No I/O performed at the orchestrator hot-path: the export fn writes
//! to disk synchronously when invoked by the auditor's post-audit hook
//! (rare path -- one write per BabyStep audit cycle).
//!
//! Hashing: blake3 only (CLAUDE.md anti-stale rule + DETERMINISTIC_CHECK
//! #9). No blake2, no sha2.
//!
//! No `.unwrap()` / `.expect()` outside test code.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Auditor verdict for a single BabyStep run. Mirrors the closed
/// taxonomy the auditor agent emits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditVerdict {
    Pass,
    Block,
    SoftWarn,
}

impl AuditVerdict {
    /// Snake-case wire tag.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Block => "block",
            Self::SoftWarn => "soft_warn",
        }
    }
}

/// One structured finding-reason row in the auditor's verdict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FindingReason {
    /// Short closed-taxonomy label (e.g. "stub_unresolved", "lpo_frozen_edit").
    pub kind: String,
    /// Human-readable message body.
    pub message: String,
}

/// MAST (Multi-Agent System Trajectory) failure-mode tag, from the
/// E-6 substrate. Optional because not every audit runs MAST tagging.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MastMode {
    /// Short label (e.g. "tool_misuse_loop").
    pub label: String,
}

/// JSON envelope written to `.elai_cc/optimization/feedback/<run_id>.json`.
///
/// Field shape is pinned by the python optimizer side (interface in
/// `workflow/papers/dspy-interface.md` + the GEPA reflection contract);
/// any rename here is a cross-language contract break.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditorFeedbackEnvelope {
    /// `optimization_run_id` correlator (uuid-v4 hex).
    pub run_id: String,
    /// Per-run iteration index (one envelope per BabyStep audit cycle).
    pub iteration_index: u32,
    /// Closed-taxonomy verdict.
    pub verdict: AuditVerdict,
    /// Structured per-finding reasons.
    pub structured_reasons: Vec<FindingReason>,
    /// Optional MAST tag (E-6).
    pub mast_mode: Option<MastMode>,
    /// Names of failing tests, if any (verbatim cargo / pytest output).
    pub failing_test_names: Vec<String>,
    /// Raw BLOCK / SOFT_WARN message bodies (verbatim auditor text;
    /// GEPA paper recommends preserving verbatim phrasing for the
    /// reflection LM).
    pub full_block_messages: Vec<String>,
    /// Wall-clock millis since UNIX_EPOCH.
    pub ts_ms: i64,
}

/// Errors the export path can surface.
#[derive(Debug, Error)]
pub enum FeedbackExportError {
    /// Output directory could not be created.
    #[error("feedback dir mkdir failed: {0}")]
    MkdirFailed(String),
    /// Envelope JSON serialisation failed.
    #[error("envelope serialise failed: {0}")]
    SerializeFailed(String),
    /// Atomic-write to the target path failed.
    #[error("write failed: {0}")]
    WriteFailed(String),
}

/// Compute the canonical relative path for a given run-id envelope. The
/// caller joins this onto the project root to get the absolute path.
#[must_use]
pub fn feedback_relative_path(run_id: &str) -> PathBuf {
    PathBuf::from(".elai_cc")
        .join("optimization")
        .join("feedback")
        .join(format!("{run_id}.json"))
}

/// Compute the absolute output path under `project_root`. Pure / no I/O.
#[must_use]
pub fn feedback_output_path(project_root: &Path, run_id: &str) -> PathBuf {
    project_root.join(feedback_relative_path(run_id))
}

/// Write the envelope as pretty-printed JSON to
/// `<project_root>/.elai_cc/optimization/feedback/<run_id>.json`. Creates
/// parent directories if absent. Returns the absolute path written.
///
/// Atomicity: writes a temp file then renames in-place (best-effort on
/// Windows; rename-over-existing is supported on the supported targets).
pub fn export_for_run(
    project_root: &Path,
    envelope: &AuditorFeedbackEnvelope,
) -> Result<PathBuf, FeedbackExportError> {
    let target = feedback_output_path(project_root, &envelope.run_id);
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| FeedbackExportError::MkdirFailed(e.to_string()))?;
    }
    let json = serde_json::to_vec_pretty(envelope)
        .map_err(|e| FeedbackExportError::SerializeFailed(e.to_string()))?;
    // Atomic write: tmp + sync_all + rename via shared helper (I-3 Phase-3C hardening).
    crate::util::atomic_io::atomic_write_sync(&target, &json)
        .map_err(|e| FeedbackExportError::WriteFailed(e.to_string()))?;
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample_envelope(run_id: &str) -> AuditorFeedbackEnvelope {
        AuditorFeedbackEnvelope {
            run_id: run_id.to_string(),
            iteration_index: 1,
            verdict: AuditVerdict::Block,
            structured_reasons: vec![FindingReason {
                kind: "stub_unresolved".into(),
                message: "todo[E8.x] lingered past phase close".into(),
            }],
            mast_mode: Some(MastMode {
                label: "tool_misuse_loop".into(),
            }),
            failing_test_names: vec!["orchestrator::baby_step_spec_lpo::tests::x".into()],
            full_block_messages: vec!["BLOCK: stub_unresolved at E8.x".into()],
            ts_ms: 1_700_000_162_002,
        }
    }

    #[test]
    fn relative_path_uses_dot_elai_cc_optimization_feedback() {
        let p = feedback_relative_path("run-abc");
        let s = p.to_string_lossy().replace('\\', "/");
        assert!(s.contains(".elai_cc/optimization/feedback/run-abc.json"));
    }

    #[test]
    fn export_writes_to_run_id_filename() {
        let dir = TempDir::new().expect("tempdir");
        let envelope = sample_envelope("run-write");
        let out = export_for_run(dir.path(), &envelope).expect("export ok");
        assert!(out.exists(), "envelope path must exist after export");
        assert!(out.to_string_lossy().contains("run-write.json"));
    }

    #[test]
    fn export_then_read_round_trips() {
        let dir = TempDir::new().expect("tempdir");
        let envelope = sample_envelope("run-rt");
        let out = export_for_run(dir.path(), &envelope).expect("export ok");
        let raw = std::fs::read_to_string(&out).expect("read back");
        let parsed: AuditorFeedbackEnvelope =
            serde_json::from_str(&raw).expect("parse back");
        assert_eq!(parsed, envelope);
    }

    #[test]
    fn verdict_wire_tags_stable() {
        assert_eq!(AuditVerdict::Pass.as_str(), "pass");
        assert_eq!(AuditVerdict::Block.as_str(), "block");
        assert_eq!(AuditVerdict::SoftWarn.as_str(), "soft_warn");
    }

    #[test]
    fn export_overwrites_existing_file_atomically() {
        let dir = TempDir::new().expect("tempdir");
        let mut envelope = sample_envelope("run-over");
        export_for_run(dir.path(), &envelope).expect("first export");
        envelope.iteration_index = 99;
        let out = export_for_run(dir.path(), &envelope).expect("second export");
        let raw = std::fs::read_to_string(&out).expect("read back");
        assert!(raw.contains("\"iteration_index\": 99"));
    }
}
