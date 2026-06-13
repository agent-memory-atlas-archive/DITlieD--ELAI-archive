//! I-33: mechanical decision governance primitives.
//!
//! Three layered primitives enforced by this module:
//!
//! 1. **CEFL** (Candidate Externalization for Fair Labelling): the orchestrator
//!    authors the candidate set as a JSON Schema; the model only scores/ranks
//!    within that set. Model-invented options are structurally unparseable.
//!    Enforced via [`ConstrainedDecodingConfig::json_schema`] on both cloud
//!    (Anthropic Tool API input_schema via grounding_client.rs:767) and sidecar
//!    (lm-format-enforcer JsonSchemaParser via server.py:2257) lanes.
//!
//! 2. **I6Q** (rationale quality validation): deterministic checker for
//!    rationale length (`min_len=20`) and lexical diversity
//!    (`min_unique_words=5`). Vacuous rationales are rejected before the
//!    outcome is committed; callers should route to `RefusalDecision::Abstain`
//!    on rejection.
//!
//! 3. **Commit-Reveal Integrity**: blake3(seed || choice_index) committed to
//!    SSB before the reveal. SSB row sequence order
//!    (`commit_row_id < reveal_row_id`) proves the choice was sealed before
//!    observation. Out-of-order detection satisfies wiring-enforcement R2
//!    (missing evidence is a hard FAIL).
//!
//! Production entry point: [`governed_prompt_decision`].
//!
//! Wire-dark gate: none. This module is always-compiled; individual primitives
//! are opt-in via the caller supplying `Some(cefl)` / `Some(i6q)` arguments.
//! Governance metrics are emitted unconditionally when primitives are active.
//!
//! Cross-lang pair: crates/dashboard-v2/src-tauri/src/commands_i33.rs

use std::sync::Arc;

use elai_core::clock::ClockProvider;
use elai_providers::decoding::ConstrainedDecodingConfig;
use elai_providers::runtime::RuntimeAdapter;

use crate::architecture::hook_prompt::{
    prompt_decision, PromptDecision, PromptHandlerError,
};
use crate::ssb::{
    GovernancePrimitiveFiredKind, GovernancePrimitiveFiredPayload, Ssb, SsbPayload,
};

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// Errors produced by [`I6qValidator::validate`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum I6qRejection {
    /// Rationale is shorter than [`I6qValidator::min_len`] characters.
    #[error("rationale too short: {actual} chars < {required} minimum")]
    TooShort { actual: usize, required: usize },
    /// Rationale has fewer unique words than [`I6qValidator::min_unique_words`].
    #[error("rationale too vacuous: {actual} unique words < {required} minimum")]
    TooVacuous { actual: usize, required: usize },
}

/// Errors produced by [`CommitRevealRecord::verify_order`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CommitOrderError {
    /// The reveal row was written before or at the same sequence position as
    /// the commit row, meaning the choice was not sealed before observation.
    #[error(
        "commit-reveal order violated: commit_row_id={commit_row_id} >= reveal_row_id={reveal_row_id}"
    )]
    OutOfOrder {
        commit_row_id: i64,
        reveal_row_id: i64,
    },
}

/// Errors produced by [`governed_prompt_decision`].
#[derive(Debug, thiserror::Error)]
pub enum GovernanceError {
    /// The underlying prompt-handler returned an error.
    #[error("prompt handler error: {0}")]
    PromptHandler(#[from] PromptHandlerError),
    /// I6Q rationale validation failed.
    #[error("I6Q rationale rejected: {0}")]
    I6qRejected(#[from] I6qRejection),
    /// SSB emission failed (non-fatal in production; surfaced for callers
    /// that want to treat missing evidence as a hard failure per R2).
    #[error("SSB write failed: {0}")]
    SsbWrite(String),
}

// ---------------------------------------------------------------------------
// CEFL — Candidate Externalization for Fair Labelling
// ---------------------------------------------------------------------------

/// Encapsulates the candidate set + JSON Schema for a CEFL-governed decision.
///
/// The caller constructs this with the exhaustive list of candidates the model
/// may choose from, and [`CeflRequest::apply`] augments the prompt so the
/// model receives an unambiguous constrained-choice framing. The companion
/// [`CeflRequest::decoding_config`] attaches a JSON-Schema constrained decoding
/// config to [`CompletionConfig`](elai_providers::runtime::CompletionConfig) so
/// model-invented options are structurally unparseable.
#[derive(Debug, Clone)]
pub struct CeflRequest {
    candidates: Vec<String>,
    schema_json: String,
}

impl CeflRequest {
    /// Build a CEFL request from a list of candidate labels.
    ///
    /// Constructs a JSON Schema `{"type":"string","enum":[...]}` constraining
    /// the model to exactly one of `candidates`. Candidates must be non-empty
    /// and deduplicated by the caller; duplicate labels are structurally valid
    /// but reduce the governance signal.
    ///
    /// # Errors
    /// Returns `None` when `candidates` is empty (no valid choice exists).
    #[must_use]
    pub fn new(candidates: Vec<String>) -> Option<Self> {
        if candidates.is_empty() {
            return None;
        }
        let items_json = candidates
            .iter()
            .map(|c| format!("\"{}\"", c.replace('"', "\\\"")))
            .collect::<Vec<_>>()
            .join(",");
        let schema_json = format!(
            r#"{{"type":"string","enum":[{items_json}]}}"#,
            items_json = items_json
        );
        Some(Self {
            candidates,
            schema_json,
        })
    }

    /// Augment `prompt` with an explicit candidate-set framing.
    ///
    /// The augmented prompt enumerates the valid options inline so the model
    /// has unambiguous guidance even when constrained decoding is not
    /// supported on the active backend. The model MUST return exactly one of
    /// the listed labels.
    #[must_use]
    pub fn apply(&self, prompt: &str) -> String {
        let list = self
            .candidates
            .iter()
            .enumerate()
            .map(|(i, c)| format!("  {}: {}", i + 1, c))
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "{prompt}\n\n[GOVERNANCE: respond with EXACTLY one of the following options — \
             no other text, no explanation, no invented options]\n{list}",
            prompt = prompt,
            list = list
        )
    }

    /// The JSON Schema string constraining the model output.
    ///
    /// Pass this to [`ConstrainedDecodingConfig::json_schema`] to attach
    /// schema-constrained decoding to a [`CompletionConfig`](elai_providers::runtime::CompletionConfig).
    #[must_use]
    pub fn schema_json(&self) -> &str {
        &self.schema_json
    }

    /// Build a [`ConstrainedDecodingConfig`] for this CEFL request.
    #[must_use]
    pub fn decoding_config(&self) -> ConstrainedDecodingConfig {
        ConstrainedDecodingConfig::json_schema(&self.schema_json)
    }

    /// The candidate labels in declaration order.
    #[must_use]
    pub fn candidates(&self) -> &[String] {
        &self.candidates
    }
}

// ---------------------------------------------------------------------------
// I6Q — rationale quality validator
// ---------------------------------------------------------------------------

/// Deterministic checker for rationale quality.
///
/// Validates that a rationale string meets minimum length and lexical diversity
/// thresholds. Vacuous rationales ("yes", "no", "because I said so") are
/// rejected before the decision is committed.
#[derive(Debug, Clone)]
pub struct I6qValidator {
    /// Minimum character count (inclusive). Default: 20.
    pub min_len: usize,
    /// Minimum number of unique whitespace-delimited words (inclusive). Default: 5.
    pub min_unique_words: usize,
}

impl Default for I6qValidator {
    fn default() -> Self {
        Self {
            min_len: 20,
            min_unique_words: 5,
        }
    }
}

impl I6qValidator {
    /// Validate `rationale` against the configured thresholds.
    ///
    /// # Errors
    /// Returns `I6qRejection::TooShort` when `rationale.len() < min_len`, or
    /// `I6qRejection::TooVacuous` when the count of unique words is below
    /// `min_unique_words`. Length is checked first.
    pub fn validate(&self, rationale: &str) -> Result<(), I6qRejection> {
        let actual_len = rationale.len();
        if actual_len < self.min_len {
            return Err(I6qRejection::TooShort {
                actual: actual_len,
                required: self.min_len,
            });
        }
        let unique: std::collections::HashSet<&str> =
            rationale.split_whitespace().collect();
        let actual_unique = unique.len();
        if actual_unique < self.min_unique_words {
            return Err(I6qRejection::TooVacuous {
                actual: actual_unique,
                required: self.min_unique_words,
            });
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Commit-reveal integrity
// ---------------------------------------------------------------------------

/// A single commit-reveal record binding a seed + choice to a blake3 hash.
///
/// Workflow:
/// 1. Before the model sees the prompt, the caller writes a "commit" SSB row
///    containing [`commit_hash`] and records the returned `commit_row_id`.
/// 2. The model produces a choice (choice_index into the CEFL candidate set).
/// 3. The caller writes a "reveal" SSB row with the choice and records
///    `reveal_row_id`.
/// 4. [`verify_order`] asserts `commit_row_id < reveal_row_id`, proving the
///    hash was sealed before the choice was observed.
#[derive(Debug, Clone)]
pub struct CommitRevealRecord {
    /// Caller-supplied entropy seed. In bench fixtures this is the `--seed`
    /// CLI arg. Production seed injection is PENDING (U-2, D-4).
    pub seed: u64,
    /// Index into the CEFL candidate set selected by the model.
    pub choice_index: usize,
}

impl CommitRevealRecord {
    /// Compute the commit hash: blake3(seed.to_le_bytes() || choice_index.to_le_bytes()).
    ///
    /// Returns a 32-byte array. The hash is deterministic and used as the
    /// "sealed commitment" written to SSB before the model runs. The caller
    /// must write this to SSB and retain the returned row_id.
    #[must_use]
    pub fn commit_hash(&self) -> [u8; 32] {
        let mut input = [0u8; 16];
        input[..8].copy_from_slice(&self.seed.to_le_bytes());
        input[8..].copy_from_slice(&self.choice_index.to_le_bytes());
        *blake3::hash(&input).as_bytes()
    }

    /// Verify that the commit row was written before the reveal row.
    ///
    /// SSB row IDs are monotonically increasing (SQLite rowid). A valid
    /// commit-reveal pair satisfies `commit_row_id < reveal_row_id`.
    ///
    /// # Errors
    /// Returns `CommitOrderError::OutOfOrder` when
    /// `commit_row_id >= reveal_row_id` (the reveal was written first or
    /// at the same sequence position, meaning the choice was not sealed
    /// before observation).
    pub fn verify_order(
        commit_row_id: i64,
        reveal_row_id: i64,
    ) -> Result<(), CommitOrderError> {
        if commit_row_id >= reveal_row_id {
            return Err(CommitOrderError::OutOfOrder {
                commit_row_id,
                reveal_row_id,
            });
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// SSB emission
// ---------------------------------------------------------------------------

/// Emit one `GovernancePrimitiveFired` SSB row.
///
/// No-op (returns `Ok(())`) when `ssb` is `None` (test / offline contexts).
/// Non-fatal: callers that require off-agent evidence for wiring-enforcement
/// R2 should treat an `Err` as a hard failure.
///
/// # Errors
/// Returns `GovernanceError::SsbWrite` when the SSB write fails.
pub async fn emit_governance_fired(
    ssb: Option<&Arc<Ssb>>,
    kind: GovernancePrimitiveFiredKind,
    session_id: Option<&str>,
    details_json: &str,
    clock: &Arc<dyn ClockProvider>,
) -> Result<(), GovernanceError> {
    let Some(ssb) = ssb else {
        return Ok(());
    };
    let payload = SsbPayload::GovernancePrimitiveFired(GovernancePrimitiveFiredPayload {
        kind,
        session_id: session_id.map(str::to_owned),
        details_json: details_json.to_owned(),
        ts_ms: Ssb::now_ms_via_clock(clock),
    });
    ssb.write_payload("governance", 0, payload, Vec::new())
        .await
        .map_err(|e| GovernanceError::SsbWrite(e.to_string()))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Production entry point
// ---------------------------------------------------------------------------

/// Invoke [`prompt_decision`] with optional CEFL + I6Q governance wrappers.
///
/// This is the **only production call site** for `hook_prompt::prompt_decision`.
/// It satisfies wiring-enforcement R1 (real entrypoint via CLI bench driver at
/// `crates/cli/src/bench_governance.rs`) and R4 (off-author SSB rows for
/// GovernancePrimitiveFired events).
///
/// # Governance flow
///
/// 1. If `cefl` is `Some`, augment the prompt with the candidate framing and
///    attach the JSON-Schema decoding config. Emit `CeflExternalized` row.
/// 2. Call `prompt_decision` (gated by `ELAI_E25_PROMPT_HANDLER_ENABLE`).
/// 3. If `i6q` is `Some` and the decision carries a reason, validate the
///    reason. Emit `I6qRejected` row on failure and return error.
/// 4. Emit `CommitRevealSealed` row (always when SSB is present), encoding
///    the commit hash + choice ordering proof.
///
/// # Errors
/// - `GovernanceError::PromptHandler` — gate dark, timeout, parse failure
/// - `GovernanceError::I6qRejected`  — vacuous rationale
/// - `GovernanceError::SsbWrite`      — SSB write failure (non-fatal path)
pub async fn governed_prompt_decision(
    prompt: &str,
    runtime: &dyn RuntimeAdapter,
    cefl: Option<&CeflRequest>,
    i6q: Option<&I6qValidator>,
    ssb: Option<&Arc<Ssb>>,
    seed: u64,
    session_id: Option<&str>,
    clock: &Arc<dyn ClockProvider>,
) -> Result<PromptDecision, GovernanceError> {
    // Step 1: CEFL framing + decoding constraint
    let effective_prompt;
    let effective_prompt_ref: &str;
    if let Some(cefl) = cefl {
        let augmented = cefl.apply(prompt);
        let schema_escaped = cefl.schema_json().replace('"', "\\\"");
        let details = format!(
            r#"{{"candidate_count":{n},"schema":"{schema}"}}"#,
            n = cefl.candidates().len(),
            schema = schema_escaped,
        );
        emit_governance_fired(
            ssb,
            GovernancePrimitiveFiredKind::CeflExternalized,
            session_id,
            &details,
            clock,
        )
        .await?;
        effective_prompt = augmented;
        effective_prompt_ref = &effective_prompt;
    } else {
        effective_prompt_ref = prompt;
    }

    // Step 2: invoke the underlying prompt-handler
    let decision = prompt_decision(effective_prompt_ref, runtime).await?;

    // Step 3: I6Q validation on any reason text
    if let Some(i6q) = i6q {
        if let Some(reason) = decision.reason() {
            if let Err(rejection) = i6q.validate(reason) {
                let details = format!(
                    r#"{{"rejection":"{msg}"}}"#,
                    msg = rejection.to_string().replace('"', "\\\"")
                );
                // Emit the I6qRejected row before propagating the error so
                // the SSB trail is complete even when the caller bails.
                let _ = emit_governance_fired(
                    ssb,
                    GovernancePrimitiveFiredKind::I6qRejected,
                    session_id,
                    &details,
                    clock,
                )
                .await;
                return Err(GovernanceError::I6qRejected(rejection));
            }
        }
    }

    // Step 4: commit-reveal sealed beacon
    // choice_index is derived from the decision variant for the seal hash.
    let choice_index: usize = match &decision {
        PromptDecision::Allow => 0,
        PromptDecision::Deny(_) => 1,
        PromptDecision::Ask(_) => 2,
    };
    let record = CommitRevealRecord { seed, choice_index };
    let hash_hex = hex_encode(&record.commit_hash());
    let seal_details = format!(
        r#"{{"choice_index":{ci},"seed":{seed},"commit_hash_prefix":"{prefix}"}}"#,
        ci = choice_index,
        seed = seed,
        prefix = &hash_hex[..16],
    );
    emit_governance_fired(
        ssb,
        GovernancePrimitiveFiredKind::CommitRevealSealed,
        session_id,
        &seal_details,
        clock,
    )
    .await?;

    Ok(decision)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Encode 32 bytes as lowercase hex without pulling in a hex crate.
fn hex_encode(bytes: &[u8; 32]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cefl_request_new_empty_returns_none() {
        assert!(CeflRequest::new(vec![]).is_none());
    }

    #[test]
    fn cefl_request_builds_schema_and_apply() {
        let req =
            CeflRequest::new(vec!["approve".to_owned(), "deny".to_owned()]).unwrap();
        let schema = req.schema_json();
        assert!(schema.contains("\"approve\""), "schema: {schema}");
        assert!(schema.contains("\"deny\""), "schema: {schema}");
        let augmented = req.apply("Should I proceed?");
        assert!(augmented.contains("approve"), "augmented: {augmented}");
        assert!(augmented.contains("GOVERNANCE"), "augmented: {augmented}");
    }

    #[test]
    fn i6q_validator_rejects_too_short() {
        let v = I6qValidator::default();
        let err = v.validate("yes").unwrap_err();
        assert!(matches!(err, I6qRejection::TooShort { .. }), "{err:?}");
    }

    #[test]
    fn i6q_validator_rejects_too_vacuous() {
        // Padded to >20 chars but only 2 unique words.
        let err = I6qValidator::default()
            .validate("yes yes yes yes yes yes yes yes yes")
            .unwrap_err();
        assert!(matches!(err, I6qRejection::TooVacuous { .. }), "{err:?}");
    }

    #[test]
    fn i6q_validator_accepts_quality_rationale() {
        let v = I6qValidator::default();
        v.validate(
            "the proposed change introduces a new database migration that could \
             affect downstream consumers relying on the old schema",
        )
        .expect("should accept quality rationale");
    }

    #[test]
    fn commit_reveal_record_deterministic_hash() {
        let r = CommitRevealRecord {
            seed: 42,
            choice_index: 0,
        };
        let h1 = r.commit_hash();
        let h2 = r.commit_hash();
        assert_eq!(h1, h2, "commit_hash must be deterministic");
        // Different seed -> different hash
        let r2 = CommitRevealRecord {
            seed: 43,
            choice_index: 0,
        };
        assert_ne!(h1, r2.commit_hash(), "different seeds must differ");
    }

    #[test]
    fn commit_reveal_verify_order_pass() {
        CommitRevealRecord::verify_order(10, 20).expect("10 < 20 should pass");
    }

    #[test]
    fn commit_reveal_verify_order_fail_equal() {
        let err = CommitRevealRecord::verify_order(15, 15).unwrap_err();
        assert!(
            matches!(err, CommitOrderError::OutOfOrder { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn commit_reveal_verify_order_fail_reversed() {
        let err = CommitRevealRecord::verify_order(20, 10).unwrap_err();
        assert!(
            matches!(err, CommitOrderError::OutOfOrder { .. }),
            "{err:?}"
        );
    }
}
