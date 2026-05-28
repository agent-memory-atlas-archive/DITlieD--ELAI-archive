//! E-43 -- skill invocation telemetry.
//!
//! PostToolUse hook helper that emits a SkillSystemEventPayload to the
//! shared state board (SSB) and appends a one-line JSON event to the
//! per-session hook-executions log. The prompt itself is never written;
//! only an 8-byte blake3 prefix of the prompt is stored.
//!
//! Wire-dark: emission is gated by the ELAI_SKILL_SYSTEM_ENABLE env var.
//! Anti-stale-training rules honored: blake3 (NOT blake2/sha2) and no
//! `.unwrap()` on the production path.

use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::ssb::{SkillEventPhase, SkillSystemEventPayload};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerMethod {
    Explicit,
    Implicit,
}

impl TriggerMethod {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Explicit => "explicit",
            Self::Implicit => "implicit",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillOutcome {
    Passed,
    Skipped,
    Errored,
    EvalBelowThreshold,
    ScanFinding,
    ValidatorReject,
}

impl SkillOutcome {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Skipped => "skipped",
            Self::Errored => "errored",
            Self::EvalBelowThreshold => "eval_below_threshold",
            Self::ScanFinding => "scan_finding",
            Self::ValidatorReject => "validator_reject",
        }
    }
}

#[derive(Debug)]
pub enum SkillTelemetryError {
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl std::fmt::Display for SkillTelemetryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "skill telemetry IO error: {e}"),
            Self::Json(e) => write!(f, "skill telemetry JSON error: {e}"),
        }
    }
}

impl std::error::Error for SkillTelemetryError {}

impl From<std::io::Error> for SkillTelemetryError {
    fn from(e: std::io::Error) -> Self { Self::Io(e) }
}

impl From<serde_json::Error> for SkillTelemetryError {
    fn from(e: serde_json::Error) -> Self { Self::Json(e) }
}

#[must_use]
pub fn is_enabled() -> bool {
    match env::var("ELAI_SKILL_SYSTEM_ENABLE") {
        Ok(v) => {
            let lower = v.trim().to_ascii_lowercase();
            matches!(lower.as_str(), "1" | "true" | "yes" | "on")
        }
        Err(_) => false,
    }
}

#[must_use]
pub fn prompt_hash_prefix(prompt: &str) -> [u8; 8] {
    let hash = blake3::hash(prompt.as_bytes());
    let bytes = hash.as_bytes();
    let mut out = [0u8; 8];
    out.copy_from_slice(&bytes[..8]);
    out
}

#[must_use]
pub fn build_invoked_payload(
    skill_name: &str,
    trigger: TriggerMethod,
    prompt: &str,
    outcome: SkillOutcome,
    ts_ms: i64,
) -> SkillSystemEventPayload {
    SkillSystemEventPayload {
        phase: SkillEventPhase::Invoked,
        skill_name: skill_name.to_owned(),
        prompt_hash_prefix: Some(prompt_hash_prefix(prompt)),
        trigger_method: trigger.as_str().to_owned(),
        outcome: outcome.as_str().to_owned(),
        eval_precision: None,
        eval_recall: None,
        scan_findings: None,
        ts_ms,
    }
}

#[must_use]
pub fn build_loaded_payload(skill_name: &str, ts_ms: i64) -> SkillSystemEventPayload {
    SkillSystemEventPayload {
        phase: SkillEventPhase::Loaded,
        skill_name: skill_name.to_owned(),
        prompt_hash_prefix: None,
        trigger_method: String::new(),
        outcome: "passed".to_owned(),
        eval_precision: None,
        eval_recall: None,
        scan_findings: None,
        ts_ms,
    }
}

#[must_use]
pub fn build_failed_payload(
    skill_name: &str,
    outcome: SkillOutcome,
    eval_precision: Option<f32>,
    eval_recall: Option<f32>,
    scan_findings: Option<u32>,
    ts_ms: i64,
) -> SkillSystemEventPayload {
    SkillSystemEventPayload {
        phase: SkillEventPhase::Failed,
        skill_name: skill_name.to_owned(),
        prompt_hash_prefix: None,
        trigger_method: String::new(),
        outcome: outcome.as_str().to_owned(),
        eval_precision,
        eval_recall,
        scan_findings,
        ts_ms,
    }
}

pub fn log_skill_invocation(
    workflow_root: &Path,
    session_id: &str,
    payload: &SkillSystemEventPayload,
) -> Result<Option<PathBuf>, SkillTelemetryError> {
    if !is_enabled() { return Ok(None); }
    let session_dir = workflow_root.join("sessions").join(session_id);
    let log_path = if session_dir.exists() {
        session_dir.join("hook-executions.log")
    } else {
        workflow_root.join("hook-executions.log")
    };
    if let Some(parent) = log_path.parent() {
        if !parent.exists() { fs::create_dir_all(parent)?; }
    }
    let line = serde_json::to_string(&LogLine {
        event: "skill_invoked",
        skill_name: &payload.skill_name,
        phase: payload.phase.as_str(),
        trigger_method: &payload.trigger_method,
        outcome: &payload.outcome,
        prompt_hash_prefix: payload.prompt_hash_prefix.as_ref().map(hex8),
        ts_ms: payload.ts_ms,
    })?;
    let mut file = OpenOptions::new().create(true).append(true).open(&log_path)?;
    file.write_all(line.as_bytes())?;
    file.write_all(b"\n")?;
    Ok(Some(log_path))
}

fn hex8(bytes: &[u8; 8]) -> String {
    let mut s = String::with_capacity(16);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

#[derive(Serialize)]
struct LogLine<'a> {
    event: &'a str,
    skill_name: &'a str,
    phase: &'a str,
    trigger_method: &'a str,
    outcome: &'a str,
    prompt_hash_prefix: Option<String>,
    ts_ms: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn prompt_hash_prefix_blake3_deterministic() {
        let a = prompt_hash_prefix("hello world");
        let b = prompt_hash_prefix("hello world");
        assert_eq!(a, b);
        let c = prompt_hash_prefix("hello world!");
        assert_ne!(a, c);
    }

    #[test]
    fn build_invoked_payload_carries_blake3_prefix() {
        let payload = build_invoked_payload(
            "anti-spiral",
            TriggerMethod::Explicit,
            "trigger prompt",
            SkillOutcome::Passed,
            1_700_000_000_000,
        );
        assert_eq!(payload.skill_name, "anti-spiral");
        assert!(payload.prompt_hash_prefix.is_some());
        assert_eq!(payload.outcome, "passed");
        assert_eq!(payload.trigger_method, "explicit");
    }

    #[test]
    fn log_skill_invocation_wire_dark_returns_none() {
        let tmp = TempDir::new().expect("tempdir");
        if env::var("ELAI_SKILL_SYSTEM_ENABLE").is_err() {
            let payload = build_invoked_payload(
                "anti-spiral",
                TriggerMethod::Explicit,
                "hello",
                SkillOutcome::Passed,
                1,
            );
            let result = log_skill_invocation(tmp.path(), "s-test", &payload)
                .expect("call must not error in wire-dark mode");
            assert!(result.is_none(), "wire-dark must not write");
        }
    }

    #[test]
    fn outcome_wire_labels() {
        assert_eq!(SkillOutcome::Passed.as_str(), "passed");
        assert_eq!(SkillOutcome::EvalBelowThreshold.as_str(), "eval_below_threshold");
        assert_eq!(SkillOutcome::ScanFinding.as_str(), "scan_finding");
    }
}
