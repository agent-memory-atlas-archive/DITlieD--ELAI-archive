//! I-34: CRANE hybrid grammar-pin gate caller.
//!
//! This module is the orchestrator-layer entry point for the three-layer CRANE
//! grammar hardening introduced in I-34:
//!
//! 1. **Gate eligibility check** — `crane_adaptive_eligible(&mode)` from providers;
//!    a non-eligible mode skips the gate and emits `CranePassed` with no pin check.
//! 2. **Pin verification** — `load_and_verify_grammar_manifest(manifest_path)` from
//!    providers; a mismatch emits `GrammarGateEvent(PinViolation)` and returns an error.
//! 3. **Post-parse re-validation** — `re_validate_output(output, schema_json)` from
//!    providers; a silent non-compliance emits `GrammarGateEvent(PostParseRejected)` and
//!    routes the request through the D-16 conformal-refusal seam.
//!
//! The `re_validate_output` call lives in providers (no orchestrator dep); this module
//! is the only caller that holds both an `Ssb` writer and the `CalibrationCorpus`
//! needed for `dispatch_conformal_refusal` (agent_pool.rs:2972).
//!
//! R8 compliance: this module does NOT regenerate or modify the grammar manifest;
//! it is a read-only consumer.

use std::path::Path;
use std::sync::Arc;

use thiserror::Error;
use tracing::{error, info, warn};

use elai_providers::{
    crane_adaptive_eligible, load_and_verify_grammar_manifest, re_validate_output, DecodingMode,
    GrammarPinError,
};

use crate::ssb::{GrammarGateEventKind, GrammarGateEventPayload, Ssb, SsbPayload};

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/// Error type for grammar gate evaluation.
#[derive(Debug, Error)]
pub enum GrammarGateError {
    /// Grammar manifest verification failed (CDA landmine #8).
    #[error("grammar pin violation for {path}: expected {expected}, got {actual}")]
    PinViolation {
        path: String,
        expected: String,
        actual: String,
    },

    /// Grammar manifest could not be loaded.
    #[error("grammar manifest load error: {0}")]
    ManifestLoad(#[from] GrammarPinError),

    /// Post-parse schema re-validation rejected the output (landmine #7).
    #[error("post-parse schema re-validation rejected output: {message}")]
    PostParseRejected { message: String },

    /// SSB write error (non-fatal for the gate decision itself).
    #[error("SSB write error: {0}")]
    Ssb(#[from] crate::ssb::SsbError),
}

// ---------------------------------------------------------------------------
// Gate result
// ---------------------------------------------------------------------------

/// Outcome of a single grammar gate evaluation.
#[derive(Debug, Clone, PartialEq)]
pub enum GrammarGateResult {
    /// Gate passed: crane-eligible + manifest verified + output compliant.
    Passed,
    /// Gate skipped: mode is not crane-adaptive-eligible.
    ///
    /// Equivalent to `Passed` for the caller — no conformal refusal route.
    NotEligible,
    /// Gate failed with a pin violation (manifest tamper detected).
    PinViolation {
        path: String,
        expected: String,
        actual: String,
    },
    /// Gate failed with a post-parse schema rejection.
    PostParseRejected { message: String },
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Emit a single `GrammarGateEvent` SSB row.
///
/// This is the sole SSB write site for `GrammarGateEvent` variants.
/// Called internally by [`call_grammar_gate`]; exposed for testing.
/// Returns the current wall-clock time in milliseconds since the UNIX epoch.
///
/// Saturates on pre-epoch clocks (sets t=0) and saturates past year 292278994
/// (sets t=i64::MAX). These edge cases are advisory-only for SSB timestamps.
fn now_ms() -> i64 {
    std::time::SystemTime::now() // entropy-allow: SSB journal ts_ms, no ClockProvider threaded into this gate
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(i64::MAX)
}

pub async fn emit_grammar_gate_event(
    ssb: &Arc<Ssb>,
    session_id: &str,
    grammar_path: &str,
    kind: GrammarGateEventKind,
    pin_violation_expected: Option<String>,
    pin_violation_actual: Option<String>,
) -> Result<(), GrammarGateError> {
    let ts_ms = now_ms();
    ssb.write_payload(
        session_id,
        0,
        SsbPayload::GrammarGateEvent(GrammarGateEventPayload {
            kind,
            grammar_path: grammar_path.to_owned(),
            pin_violation_expected,
            pin_violation_actual,
            session_id: session_id.to_owned(),
            ts_ms,
        }),
        vec![grammar_path.to_owned()],
    )
    .await?;
    Ok(())
}

/// Run the full CRANE grammar gate for a single decode request.
///
/// # Parameters
///
/// - `ssb` — shared SSB writer (required for `GrammarGateEvent` rows).
/// - `session_id` — caller session identifier (embedded in each SSB row).
/// - `decoding_mode` — the `DecodingMode` active for this request (gate
///   eligibility comes from `crane_adaptive_eligible`).
/// - `manifest_path` — path to the grammar-pin manifest on disk.
/// - `decoded_output` — the raw decoded string to re-validate.
/// - `schema_json` — the JSON Schema string describing the expected output
///   shape. Used by `re_validate_output` in providers.
///
/// # Returns
///
/// `Ok(GrammarGateResult)` on every outcome, including violations (the
/// violation is recorded in the SSB row and the result discriminant).
/// `Err` only on an SSB write failure or a manifest I/O error that is not
/// a pin violation (e.g. file-not-found, parse failure).
///
/// # D-16 seam
///
/// When the result is `PostParseRejected`, callers that hold a
/// `CalibrationCorpus` SHOULD route the request through
/// `agent_pool::dispatch_conformal_refusal` with `claim_score = 0.0`.
/// This function does not call `dispatch_conformal_refusal` directly
/// because it does not hold a corpus reference; the caller in
/// `run_subcommand.rs` / `bench_crane.rs` owns the corpus.
pub async fn call_grammar_gate(
    ssb: &Arc<Ssb>,
    session_id: &str,
    decoding_mode: &DecodingMode,
    manifest_path: &Path,
    decoded_output: &str,
    schema_json: &str,
) -> Result<GrammarGateResult, GrammarGateError> {
    // Layer 1: gate eligibility.
    if !crane_adaptive_eligible(decoding_mode) {
        info!(
            session_id,
            "grammar_gate: mode not crane-adaptive-eligible, skipping gate"
        );
        return Ok(GrammarGateResult::NotEligible);
    }

    // Extract grammar path for SSB rows.
    let grammar_path_str: String = match decoding_mode {
        DecodingMode::Grammar { ebnf_path, .. } => {
            ebnf_path.to_string_lossy().into_owned()
        }
        _ => String::from("unknown"),
    };

    // Layer 2: pin verification.
    match load_and_verify_grammar_manifest(manifest_path) {
        Ok(_manifest) => {
            info!(
                session_id,
                grammar_path = %grammar_path_str,
                "grammar_gate: pin verification passed"
            );
        }
        Err(GrammarPinError::PinViolation {
            path,
            expected,
            actual,
        }) => {
            warn!(
                session_id,
                grammar_path = %grammar_path_str,
                pin_path = %path,
                "grammar_gate: pin violation detected"
            );
            emit_grammar_gate_event(
                ssb,
                session_id,
                &grammar_path_str,
                GrammarGateEventKind::PinViolation,
                Some(expected.clone()),
                Some(actual.clone()),
            )
            .await?;
            return Ok(GrammarGateResult::PinViolation {
                path,
                expected,
                actual,
            });
        }
        Err(other) => {
            error!(
                session_id,
                error = %other,
                "grammar_gate: manifest load failed"
            );
            return Err(GrammarGateError::ManifestLoad(other));
        }
    }

    // Layer 3: post-parse re-validation (landmine #7).
    match re_validate_output(decoded_output, schema_json) {
        Ok(()) => {
            info!(
                session_id,
                grammar_path = %grammar_path_str,
                "grammar_gate: post-parse re-validation passed"
            );
            emit_grammar_gate_event(
                ssb,
                session_id,
                &grammar_path_str,
                GrammarGateEventKind::CranePassed,
                None,
                None,
            )
            .await?;
            Ok(GrammarGateResult::Passed)
        }
        Err(validation_err) => {
            warn!(
                session_id,
                grammar_path = %grammar_path_str,
                message = %validation_err.message,
                "grammar_gate: post-parse re-validation rejected output"
            );
            emit_grammar_gate_event(
                ssb,
                session_id,
                &grammar_path_str,
                GrammarGateEventKind::PostParseRejected,
                None,
                None,
            )
            .await?;
            // Caller is responsible for routing PostParseRejected to
            // dispatch_conformal_refusal (D-16 seam, claim_score = 0.0).
            Ok(GrammarGateResult::PostParseRejected {
                message: validation_err.message,
            })
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssb::{decode_ssb_payload, SsbActionType, SsbConfig, SsbPayload, SsbReader};
    use elai_core::clock::SeededClock;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn test_ssb(dir: &std::path::Path) -> Arc<Ssb> {
        let cfg = SsbConfig {
            db_path: dir.join("ssb_grammar_gate_test.sqlite"),
            pool_size: Some(2),
            project_id: "test-project".into(),
            session_id: "s-grammar-gate-test".into(),
        };
        Arc::new(Ssb::new(cfg, Arc::new(SeededClock::new())).expect("ssb init"))
    }

    /// Decode the first entry's payload bytes via `decode_ssb_payload` (MessagePack),
    /// match on `SsbPayload::GrammarGateEvent`, and return the `kind` field.
    ///
    /// `query_by_type(GrammarGateEvent(_))` matches ALL `grammar_gate_event` rows
    /// (all Kind arms share the same SQL string "grammar_gate_event"). To prove
    /// the specific Kind arm landed we must deserialize the payload and inspect
    /// the `kind` field inside `GrammarGateEventPayload`.
    fn first_grammar_gate_kind(entries: &[crate::ssb::SsbEntry]) -> GrammarGateEventKind {
        let bytes: &[u8] = &entries[0].payload;
        let payload = decode_ssb_payload(bytes).expect("decode_ssb_payload");
        match payload {
            SsbPayload::GrammarGateEvent(p) => p.kind,
            other => panic!("expected SsbPayload::GrammarGateEvent, got {other:?}"),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn emit_crane_passed_writes_ssb_row() {
        let dir = TempDir::new().expect("tempdir");
        let ssb = test_ssb(dir.path());

        emit_grammar_gate_event(
            &ssb,
            "s-test",
            "fixtures/test.ebnf",
            GrammarGateEventKind::CranePassed,
            None,
            None,
        )
        .await
        .expect("emit");

        let entries = ssb
            .query_by_type(SsbActionType::GrammarGateEvent(GrammarGateEventKind::CranePassed))
            .expect("query");
        assert!(!entries.is_empty(), "expected at least one grammar_gate_event row");
        // Kind-discriminating assertion: deserialize payload and confirm CranePassed arm.
        assert_eq!(
            first_grammar_gate_kind(&entries),
            GrammarGateEventKind::CranePassed,
            "payload kind must be CranePassed, not a different arm"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn emit_pin_violation_writes_ssb_row() {
        let dir = TempDir::new().expect("tempdir");
        let ssb = test_ssb(dir.path());

        emit_grammar_gate_event(
            &ssb,
            "s-test",
            "fixtures/test.ebnf",
            GrammarGateEventKind::PinViolation,
            Some("aaaa1111".into()),
            Some("bbbb2222".into()),
        )
        .await
        .expect("emit");

        // query_by_type returns ALL grammar_gate_event rows regardless of Kind
        // (all arms share the same SQL string "grammar_gate_event").
        // Use PinViolation here for documentation clarity, but ANY Kind works as the SQL key.
        let entries = ssb
            .query_by_type(SsbActionType::GrammarGateEvent(GrammarGateEventKind::PinViolation))
            .expect("query");
        assert!(!entries.is_empty(), "expected at least one grammar_gate_event row");
        // Kind-discriminating assertion: deserialize payload and confirm PinViolation arm.
        // This is the BLOCK from A59 audit: previously queried with CranePassed (same SQL),
        // then asserted !is_empty() — which always passed regardless of what was written.
        // Now we deserialize the payload and assert the actual Kind discriminant.
        assert_eq!(
            first_grammar_gate_kind(&entries),
            GrammarGateEventKind::PinViolation,
            "payload kind must be PinViolation; a CranePassed or PostParseRejected \
             row would indicate the wrong event was written"
        );
    }
}
