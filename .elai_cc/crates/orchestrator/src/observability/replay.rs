//! E-29 Tier-B -- `ssb_resume_from(checkpoint_id, patch)` validation
//! primitive.
//!
//! Consumes the C-44 / D-9 frozen-surface predicate via
//! [`crate::locked_keys_auditor::LockedKeysAuditor`]. FAIL-CLOSED on any
//! error -- the replay refuses all patches when the registry is
//! unreadable, missing, or returns a violation. This is the load-bearing
//! safety property documented in WIRING_CONTRACT.stub_ban of the E-29
//! plan: replays MUST NOT bypass the production frozen-surface gate.
//!
//! Out-of-scope for this Tier-B slice (tracked as RUNPOD-PENDING in
//! `workflow/reasoning/E-29-decisions.md`):
//!   * KV-cache state capture for true deterministic replay (Open
//!     Question 3 in the plan body)
//!   * clock + RNG interception (Open Question 4 -- requires sidecar
//!     instrumentation outside ELAI's process boundary)
//!   * Cross-runtime patch translation (sidecar / cloud / MLX)
//!
//! This module is the VALIDATION + EMIT path; the actual replay run
//! itself is driven by the dashboard frontend invoking the Tauri
//! command in `commands_e29.rs`.
//!
//! Grounding:
//!   * dev.to/sreeni5018/debugging-non-deterministic-llm-agents-
//!     implementing-checkpoint-based-state-replay-with-langgraph-5171
//!     -- LangGraph checkpoint snapshot-per-superstep replay pattern.
//!   * tianpan.co/blog/2026-04-12-deterministic-replay-debugging-non-
//!     deterministic-ai-agents -- temp=0+seed alone is insufficient
//!     (motivation for the documented RUNPOD-PENDING list above).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::locked_keys_auditor::{LockedKeysAuditError, LockedKeysAuditor};

/// Patch supplied with [`ssb_resume_from`]. All fields are optional --
/// an empty patch is a "re-run from checkpoint with no overrides" call.
///
/// `tool_overrides` carries the file-level surface the frozen-key
/// auditor scans against; for prompt-only patches the override list is
/// empty and the auditor sees a synthetic single-file diff against
/// `<prompt-virtual-path>`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayPatch {
    /// Optional override prompt body. Stored as plain text -- the
    /// frozen-surface auditor sees a virtual file `replay/prompt.txt`
    /// for this override.
    pub prompt: Option<String>,
    /// Optional model id override (e.g. `"qwen-2.5-coder-3b"`).
    pub model: Option<String>,
    /// Per-tool overrides -- one entry per tool the user wants to
    /// swap. Empty list = no tool changes.
    pub tool_overrides: Vec<ToolOverride>,
}

/// One tool override entry. The replay validator constructs a synthetic
/// staged-files list from the union of overrides and feeds it to the
/// frozen-surface auditor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolOverride {
    /// Tool identifier (e.g. `"bash"`, `"read"`).
    pub tool_name: String,
    /// File path the tool is bound to in the replay (e.g.
    /// `crates/orchestrator/src/observability/replay.rs`).
    pub target_path: String,
    /// Override label so dashboard consumers can render the diff
    /// summary without re-fetching tool metadata.
    pub label: String,
}

/// Opaque replay-session identifier. Always derived deterministically
/// from `(checkpoint_id, patch_blake3)` so two identical patches
/// against the same checkpoint produce the same id (idempotent
/// dispatch consumes this; see E-44 IdempotentDispatchSkipped).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReplaySessionId(pub String);

impl ReplaySessionId {
    /// Construct a deterministic replay-session id from the parent
    /// checkpoint id + a blake3 prefix of the canonical-JSON serialised
    /// patch. blake3 is required per anti-stale-training (no blake2 /
    /// sha2 for non-RFC uses).
    pub fn derive(checkpoint_id: i64, patch: &ReplayPatch) -> Self {
        let canonical = serde_json::to_string(patch).unwrap_or_default();
        let hash = blake3::hash(canonical.as_bytes());
        let hex = hash.to_hex();
        let prefix: String = hex.as_str().chars().take(16).collect();
        Self(format!("replay-{checkpoint_id}-{prefix}"))
    }
}

/// Outcome of a [`ssb_resume_from`] call. Encoded into the per-row
/// `AgenticObservabilityEventKind::ReplayResume` payload so dashboard
/// consumers can render Refused vs Accepted vs Failed without parsing
/// the auditor result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplayOutcome {
    /// Patch refused -- frozen-surface predicate fired. Payload carries
    /// the violation reason string for surface in the dashboard.
    Refused,
    /// Patch refused -- the auditor registry was unreadable / missing.
    /// This is the FAIL-CLOSED branch documented in the module-level
    /// doc; we refuse all patches when the predicate cannot run.
    PredicateUnavailable,
    /// Patch accepted -- replay session id was minted. The actual replay
    /// run dispatch is the caller's responsibility (Tauri command emits
    /// the row + spawns the replay).
    Accepted,
}

/// Errors produced by [`ssb_resume_from`]. The validator never panics
/// -- caller-visible errors are explicit `Result` variants.
#[derive(Debug, thiserror::Error)]
pub enum ReplayError {
    #[error("frozen-surface predicate refused patch: {0}")]
    FrozenSurfaceRefused(String),
    #[error("frozen-surface predicate unavailable: {0}")]
    PredicateUnavailable(String),
    #[error("invalid patch: {0}")]
    InvalidPatch(String),
}

/// Validate a replay patch against the frozen-surface predicate and
/// mint a [`ReplaySessionId`] on success.
///
/// The caller MUST pass a `registry_path` -- the C-44/D-9 surface
/// (`elai-locked-keys.toml` at workspace root by convention). When the
/// registry is missing or unreadable, this function returns
/// [`ReplayOutcome::PredicateUnavailable`] (NOT a permissive default).
///
/// Procedure:
/// 1. Build the synthetic staged-files list from
///    `patch.tool_overrides` + an optional `replay/prompt.txt` virtual
///    entry when `patch.prompt.is_some()`.
/// 2. Load the auditor via [`LockedKeysAuditor::load`]. On any
///    [`LockedKeysAuditError`], return
///    [`ReplayOutcome::PredicateUnavailable`].
/// 3. Run [`LockedKeysAuditor::audit_diff`] over the synthetic list.
///    Any violation => [`ReplayOutcome::Refused`].
/// 4. Mint a deterministic [`ReplaySessionId`] from `(checkpoint_id,
///    patch_blake3)` and return [`ReplayOutcome::Accepted`].
///
/// The function does NOT itself dispatch the replay run; that is the
/// Tauri command surface's job. This is the validation primitive only.
pub fn ssb_resume_from(
    checkpoint_id: i64,
    patch: ReplayPatch,
    registry_path: PathBuf,
) -> Result<(ReplaySessionId, ReplayOutcome), ReplayError> {
    if checkpoint_id < 0 {
        return Err(ReplayError::InvalidPatch(format!(
            "checkpoint_id must be non-negative; got {checkpoint_id}"
        )));
    }

    // Step 1: build the synthetic staged-files list.
    let mut staged_files: Vec<PathBuf> = patch
        .tool_overrides
        .iter()
        .map(|t| PathBuf::from(&t.target_path))
        .collect();
    if patch.prompt.is_some() {
        staged_files.push(PathBuf::from("replay/prompt.txt"));
    }

    // Step 2: load the auditor. Fail-closed on any error.
    let auditor = match LockedKeysAuditor::load(registry_path.clone(), None) {
        Ok(a) => a,
        Err(LockedKeysAuditError::RegistryUnreadable { path, detail }) => {
            let session_id = ReplaySessionId::derive(checkpoint_id, &patch);
            tracing::warn!(
                ?path,
                detail = %detail,
                ?session_id,
                "ssb_resume_from: locked-keys registry unreadable; refusing patch (fail-closed)"
            );
            return Ok((session_id, ReplayOutcome::PredicateUnavailable));
        }
        Err(e) => {
            return Err(ReplayError::PredicateUnavailable(e.to_string()));
        }
    };

    // Step 3: run the audit. Any violation refuses the patch.
    let result = auditor.audit_diff(&staged_files);
    if !result.is_clean() {
        let reason = result
            .violations
            .iter()
            .map(|v| {
                format!(
                    "{} touches locked key {} ({})",
                    v.attempted_edit.display(),
                    v.locked_key.path_as_posix(),
                    v.locked_key.reason
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        let session_id = ReplaySessionId::derive(checkpoint_id, &patch);
        tracing::info!(
            ?session_id,
            violation_count = result.violations.len(),
            "ssb_resume_from: frozen-surface predicate refused patch"
        );
        return Ok((
            session_id,
            ReplayOutcome::Refused,
        )).map(|(s, o)| {
            // Embed the reason in a tracing event so dashboard consumers
            // can correlate via the structured logging tail. Caller emits
            // the SSB row with `reason` in details_json.
            tracing::debug!(reason = %reason, "frozen-surface refusal detail");
            (s, o)
        });
    }

    // Step 4: mint the replay session id.
    let session_id = ReplaySessionId::derive(checkpoint_id, &patch);
    Ok((session_id, ReplayOutcome::Accepted))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn replay_session_id_deterministic() {
        let patch = ReplayPatch {
            prompt: Some("hello".into()),
            model: Some("qwen-3b".into()),
            tool_overrides: vec![],
        };
        let a = ReplaySessionId::derive(7, &patch);
        let b = ReplaySessionId::derive(7, &patch);
        assert_eq!(a, b, "same (checkpoint_id, patch) must produce same id");
    }

    #[test]
    fn replay_session_id_differs_per_checkpoint() {
        let patch = ReplayPatch::default();
        let a = ReplaySessionId::derive(7, &patch);
        let b = ReplaySessionId::derive(8, &patch);
        assert_ne!(a, b);
    }

    #[test]
    fn fail_closed_on_unreadable_registry() {
        let dir = TempDir::new().expect("tempdir");
        let missing_path = dir.path().join("does-not-exist.toml");
        let patch = ReplayPatch::default();
        let (sid, outcome) =
            ssb_resume_from(1, patch, missing_path).expect("validator returns Ok with outcome");
        assert!(matches!(outcome, ReplayOutcome::PredicateUnavailable));
        assert!(sid.0.starts_with("replay-1-"));
    }

    #[test]
    fn accepts_empty_patch_when_registry_clean() {
        let dir = TempDir::new().expect("tempdir");
        let reg_path = dir.path().join("empty-registry.toml");
        // Empty TOML -- no locked keys, no violations possible.
        fs::write(&reg_path, b"# empty registry").expect("write empty registry");
        let patch = ReplayPatch::default();
        let result = ssb_resume_from(0, patch, reg_path);
        match result {
            Ok((_sid, ReplayOutcome::Accepted)) => {}
            Ok((_sid, other)) => panic!("expected Accepted, got {other:?}"),
            Err(e) => panic!("validator errored: {e}"),
        }
    }

    #[test]
    fn rejects_negative_checkpoint() {
        let dir = TempDir::new().expect("tempdir");
        let reg_path = dir.path().join("empty-registry.toml");
        fs::write(&reg_path, b"").expect("write");
        let patch = ReplayPatch::default();
        let result = ssb_resume_from(-1, patch, reg_path);
        assert!(matches!(result, Err(ReplayError::InvalidPatch(_))));
    }
}
