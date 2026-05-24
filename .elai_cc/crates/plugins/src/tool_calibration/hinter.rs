//! C-47: Threshold-gated hint generator.
//!
//! `generate_hint` emits a one-line corrective hint when a tool's
//! aggregated case shows enough signal to act on. The thresholds are
//! deliberately conservative — false-positive hints pollute the prompt
//! and can train the model to second-guess correct shapes.
//!
//! Gating (ALL must hold):
//!   * `total_calls >= HINT_MIN_TOTAL_CALLS` — enough samples to trust
//!     the error rate.
//!   * `error_rate >= HINT_ERROR_RATE_THRESHOLD` — failure mode is
//!     above noise.
//!   * `failure_profile.dominant_dimension().is_some()` — at least one
//!     dimension has fired.
//!
//! When all three hold, emit a snake-case hint string identifying the
//! dominant failure dimension. The downstream conversation loop
//! prepends the string as a system-level hint before the next user
//! turn (vehicle: SSB `tool_calibration_hint_emitted` row consumed by
//! the prompt-assembly path).

use super::case_base::AggregatedCase;
use super::failure_profile::FailureDimension;

/// Minimum sample count before the hinter trusts an error rate. Below
/// this floor the rate is statistical noise. Mirrors the
/// `min_per_family` floor pattern from D-7 canary.
pub const HINT_MIN_TOTAL_CALLS: u32 = 5;

/// Error-rate threshold beyond which a corrective hint fires. `0.30`
/// matches the "consistently misnames a parameter" framing in the
/// C-47 plan context (every-other-call failure is past noise).
pub const HINT_ERROR_RATE_THRESHOLD: f32 = 0.30;

/// Return a corrective hint string when the case has crossed the
/// gating threshold; otherwise return `None`.
///
/// Hint format (snake-case for downstream regex stability):
///   `"calibration_hint: tool=<name> dominant_failure=<dim>
///     error_rate=<r> over <n> calls"`
///
/// The downstream prompt-assembly path is expected to render this row
/// alongside the eager/gated tool catalog from C-35.
pub fn generate_hint(case: &AggregatedCase, tool_name: &str) -> Option<String> {
    if case.total_calls < HINT_MIN_TOTAL_CALLS {
        return None;
    }
    let rate = case.error_rate();
    if rate < HINT_ERROR_RATE_THRESHOLD {
        return None;
    }
    let dim = case.failure_profile.dominant_dimension()?;
    Some(format_hint(tool_name, dim, rate, case.total_calls))
}

fn format_hint(
    tool_name: &str,
    dim: FailureDimension,
    error_rate: f32,
    total_calls: u32,
) -> String {
    format!(
        "calibration_hint: tool={} dominant_failure={} error_rate={:.2} over {} calls",
        tool_name,
        dim.as_str(),
        error_rate,
        total_calls
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool_calibration::case_base::ResponseKind;

    fn case_with(ok: u32, err: u32, dim_kind: ResponseKind) -> AggregatedCase {
        let mut c = AggregatedCase::new("fs.read".into(), 0);
        for _ in 0..ok {
            c.observe(ResponseKind::Ok, r#"{"path":"x"}"#, 0);
        }
        for _ in 0..err {
            c.observe(dim_kind, r#"{"path":"x"}"#, 0);
        }
        c
    }

    #[test]
    fn no_hint_below_min_calls() {
        let c = case_with(0, 4, ResponseKind::ErrInvalidArgs);
        assert_eq!(generate_hint(&c, "fs.read"), None);
    }

    #[test]
    fn no_hint_below_error_rate_threshold() {
        // 7 ok + 3 err => 30% — borderline; HINT_ERROR_RATE_THRESHOLD
        // is exactly 0.30, so equal-to-threshold MUST fire.
        let c = case_with(7, 3, ResponseKind::ErrInvalidArgs);
        let h = generate_hint(&c, "fs.read");
        assert!(h.is_some(), "30% should clear the >= threshold");

        let c2 = case_with(8, 2, ResponseKind::ErrInvalidArgs);
        assert_eq!(generate_hint(&c2, "fs.read"), None, "20% under threshold");
    }

    #[test]
    fn hint_format_contains_required_fields() {
        let c = case_with(2, 8, ResponseKind::ErrInvalidArgs);
        let h = generate_hint(&c, "git.commit").expect("hint above threshold");
        assert!(h.contains("calibration_hint:"));
        assert!(h.contains("tool=git.commit"));
        assert!(h.contains("dominant_failure=key"));
        assert!(h.contains("over 10 calls"));
    }

    #[test]
    fn no_hint_when_no_failure_dimension() {
        // ErrTimeout / ErrOther map to None in FailureDimension; the
        // case stays under HINT_ERROR_RATE_THRESHOLD by construction
        // because failure_profile stays at zero.
        let c = case_with(2, 8, ResponseKind::ErrTimeout);
        assert_eq!(generate_hint(&c, "fs.read"), None);
    }
}
