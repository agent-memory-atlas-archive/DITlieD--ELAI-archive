//! E-54 dynamic knowledge-QA generation (task 9).
//!
//! Per arXiv:2511.08598 OKBench (on-demand benchmark generation; ~2K MCQs/day
//! from last-24h news per the abstract; specific throughput figure WEAK per
//! grounding triage but the framing is solid). The `knowledge-qa` channel is
//! routed via the canonical TaskFamily::ToolUse arm (per
//! workflow/reasoning/E-54-decisions.md DECISION-1 -- ToolUse-flavoured
//! web-grounded MCQs are interaction-driven, not pure recall).
//!
//! Flow:
//!   1. Hit the sidecar's existing `/v1/search` SearXNG proxy
//!      (python/elai_inference/server.py:_proxy_searxng) for the last-24h
//!      news topic seed.
//!   2. The auditor / LLM router generates MCQs from the top-K results.
//!   3. Tasks are ONE-SHOT (NOT persisted to TOML), but the SearXNG query
//!      hash + timestamp + generated MCQ set is appended to
//!      `held-out/_audit/knowledge-qa-{YYYY-MM-DD}.jsonl` for reproducibility.
//!
//! Live SearXNG hit is RUNPOD-PENDING (depends on the remote 3090 +
//! `start-searxng.sh`); the harness ships now.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::storage::audit_dir;

/// E-54 dynamic-knowledge-QA SSB row signature (audit-log row format).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DynamicKnowledgeQaAuditRow {
    pub ts: DateTime<Utc>,
    pub searxng_query_hash: String,
    pub searxng_url: String,
    pub topic_seed: String,
    pub generated_count: u32,
    pub family: String, // canonical "tool_use" per DECISION-1
}

/// One generated knowledge-QA task (in-memory only; NOT persisted to TOML).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeQaTask {
    pub topic_seed: String,
    pub question: String,
    pub choices: Vec<String>,
    pub correct_index: usize,
    pub source_urls: Vec<String>,
}

/// Errors raised by the dynamic-knowledge-QA pipeline.
#[derive(Debug, Error)]
pub enum DynamicKnowledgeQaError {
    #[error("eval-corpus dynamic-knowledge-qa: SearXNG unavailable: {0}")]
    SearxngUnavailable(String),
    #[error("eval-corpus dynamic-knowledge-qa: I/O at {0}: {1}")]
    Io(String, String),
    #[error("eval-corpus dynamic-knowledge-qa: JSON at {0}: {1}")]
    Json(String, String),
}

/// Compute the audit-log JSONL path for a given date.
#[must_use]
pub fn audit_log_path(root: &Path, date: DateTime<Utc>) -> PathBuf {
    audit_dir(root).join(format!(
        "knowledge-qa-{}.jsonl",
        date.format("%Y-%m-%d")
    ))
}

/// Compute the blake3 16-hex prefix of the SearXNG query body. Reused as the
/// `searxng_query_hash` field on the audit row.
#[must_use]
pub fn query_hash(query: &str) -> String {
    let h = blake3::hash(query.as_bytes());
    let bytes = h.as_bytes();
    let mut hex = String::with_capacity(16);
    for b in &bytes[..8] {
        hex.push_str(&format!("{b:02x}"));
    }
    hex
}

/// Append an audit-log row to `_audit/knowledge-qa-{date}.jsonl`. Per
/// E-54-decisions.md DECISION-8, the audit log is REQUIRED for every dynamic
/// knowledge-QA batch (no silent generation; contract test
/// `dynamic_knowledge_qa_logs_searxng_query` enforces).
pub fn append_audit_log(
    root: &Path,
    row: &DynamicKnowledgeQaAuditRow,
) -> Result<(), DynamicKnowledgeQaError> {
    use std::io::Write;
    let p = audit_log_path(root, row.ts);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            DynamicKnowledgeQaError::Io(parent.display().to_string(), e.to_string())
        })?;
    }
    let mut line = serde_json::to_string(row).map_err(|e| {
        DynamicKnowledgeQaError::Json(p.display().to_string(), e.to_string())
    })?;
    line.push('\n');
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&p)
        .map_err(|e| DynamicKnowledgeQaError::Io(p.display().to_string(), e.to_string()))?;
    file.write_all(line.as_bytes()).map_err(|e| {
        DynamicKnowledgeQaError::Io(p.display().to_string(), e.to_string())
    })?;
    Ok(())
}

/// RUNPOD-PENDING: live SearXNG hit + LLM-MCQ generation. Until the sidecar
/// is up, returns `SearxngUnavailable` so the caller falls back to the static
/// TOML-replay path (with a `dynamic_knowledge_qa_searxng_unavailable` SSB
/// row -- emission in the SSB layer, not here).
pub async fn generate_today(
    _searxng_url: &str,
    _topic_seed: &str,
) -> Result<Vec<KnowledgeQaTask>, DynamicKnowledgeQaError> {
    // TODO[E-54.9]: wire sidecar /v1/search; build MCQs from top-K results.
    Err(DynamicKnowledgeQaError::SearxngUnavailable(
        "live SearXNG sidecar hit is RUNPOD-PENDING".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn dynamic_knowledge_qa_logs_searxng_query() {
        let tmp = tempdir().unwrap();
        let row = DynamicKnowledgeQaAuditRow {
            ts: Utc::now(),
            searxng_query_hash: query_hash("today's tech news"),
            searxng_url: "http://127.0.0.1:8889".to_string(),
            topic_seed: "today's tech news".to_string(),
            generated_count: 12,
            family: "tool_use".to_string(),
        };
        append_audit_log(tmp.path(), &row).unwrap();
        let p = audit_log_path(tmp.path(), row.ts);
        let body = std::fs::read_to_string(p).unwrap();
        assert!(body.contains("\"searxng_query_hash\""));
        assert!(body.contains("\"tool_use\""));
    }

    #[test]
    fn query_hash_is_deterministic_16_hex() {
        let h1 = query_hash("ai news 2026");
        let h2 = query_hash("ai news 2026");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 16);
    }
}
