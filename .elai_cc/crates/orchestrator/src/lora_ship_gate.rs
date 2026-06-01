//! E-47 -- LoRA fine-tuning ship-gate.
//!
//! Gates the boot-path adapter-load step on a triple-AND check:
//!   1. `enabled` is true (operator opted in via toml block)
//!   2. `confirm_user_understanding` is true (operator read the
//!      source report and acknowledges 5-19pp forgetting risk per
//!      arXiv:2405.09673 Biderman et al.)
//!   3. A `LoraCalibrationReport` is present AND `passes_threshold()`
//!      (no-regression policy: mmlu_delta_pct and humaneval_delta_pct
//!      both >= -2.0pp vs the reference base-model baseline).
//!
//! `LORA_SHIP_GATE_DEFERRAL_TOKEN` is the load-bearing CI lint token:
//! when the operator has NOT cleared the gate, the [finetune.lora]
//! ship_gate field must remain "DEFERRED_PER_E47_REPORT". CI lint
//! refuses renames so the operator cannot drift the token without
//! tripping the lint.
//!
//! Wire-dark by default: emission is gated by
//! [`ELAI_LORA_SHIP_GATE_ENABLE_ENV`]. The pure logic
//! ([`check_lora_ship_gate`]) is callable at any time; only the SSB
//! emit helper ([`notify_lora_ship_gate`]) honors the env gate.
//!
//! Producer: this module + the `elai lora-gate-check` CLI subcommand.
//! Consumer-side adapter loader is RUNPOD-PENDING (E-47.1 follow-on);
//! the gate ships locally regardless so the deferral is enforced from
//! day one.

use serde_json::Value;

use crate::ssb::{
    LoraShipGateEventPayload, LoraShipGateEventPhase, Ssb, SsbError, SsbPayload,
    ELAI_LORA_SHIP_GATE_ENABLE_ENV,
};

/// CI lint marker. Operators MUST keep this exact literal in
/// `.elai_cc/elai_cc.toml [finetune.lora].ship_gate` until the
/// E-47.1 producer-side calibration runner ships and the operator
/// flips per `workflow/decisions/E-47-flip-runbook.md`.
///
/// Renaming this token requires an E-47.1 patch by design --
/// CI lint refuses other variants (DEFERED_PER, deferred_per, etc.).
pub const LORA_SHIP_GATE_DEFERRAL_TOKEN: &str = "DEFERRED_PER_E47_REPORT";

/// No-regression threshold (percentage points). Either eval delta
/// going below this floor fails the calibration check.
/// Source: source report 47-lora-finetuning.md "Calibration + Refuse
/// Conditions" section; conservative cap chosen to keep the gate
/// closed against the 5-19pp forgetting evidence in arXiv:2405.09673.
pub const NO_REGRESSION_FLOOR_PCT: f32 = -2.0;

/// Configuration block read from `[finetune.lora]` of `elai_cc.toml`.
///
/// Triple-AND semantic (see module doc). Fields:
///   * `ship_gate_token` -- the deferral token literal; must equal
///     [`LORA_SHIP_GATE_DEFERRAL_TOKEN`] when the gate is intentionally
///     deferred (which is the default-by-design posture for E-47).
///   * `confirm_user_understanding` -- operator acknowledgement that
///     they have read the source report and understand the
///     catastrophic-forgetting risk.
///   * `enabled` -- master switch. When false, the gate is wire-dark
///     and [`check_lora_ship_gate`] short-circuits to
///     [`LoraShipGateOutcome::CalibrationPassed`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoraShipGateConfig {
    pub ship_gate_token: String,
    pub confirm_user_understanding: bool,
    pub enabled: bool,
}

impl Default for LoraShipGateConfig {
    fn default() -> Self {
        Self {
            ship_gate_token: LORA_SHIP_GATE_DEFERRAL_TOKEN.to_owned(),
            confirm_user_understanding: false,
            enabled: false,
        }
    }
}

/// Calibration report consumed by [`check_lora_ship_gate`]. The
/// producer (RUNPOD-PENDING) will populate the two delta fields by
/// running in-domain + 200-example OOD probes against the trained
/// adapter and comparing to the base-model baseline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LoraCalibrationReport {
    /// MMLU delta in percentage points vs the base model. Negative
    /// values indicate forgetting; values below
    /// [`NO_REGRESSION_FLOOR_PCT`] fail the gate.
    pub mmlu_delta_pct: f32,
    /// HumanEval delta in percentage points vs the base model.
    /// Same threshold semantic as [`Self::mmlu_delta_pct`].
    pub humaneval_delta_pct: f32,
}

impl LoraCalibrationReport {
    /// Returns true iff BOTH deltas are at or above the
    /// no-regression floor.
    #[must_use]
    pub fn passes_threshold(&self) -> bool {
        self.mmlu_delta_pct >= NO_REGRESSION_FLOOR_PCT
            && self.humaneval_delta_pct >= NO_REGRESSION_FLOOR_PCT
    }
}

/// Result of [`check_lora_ship_gate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoraShipGateOutcome {
    /// Gate refuses the adapter-load. Three reasons collapse here:
    ///   * the deferral token is present AND the operator confirmed
    ///     understanding (intentional defer; default posture).
    ///   * the calibration report is present but fails the
    ///     no-regression threshold.
    Refused,
    /// Gate is enabled and configured to ship but no calibration
    /// report was supplied. Refuse-and-continue (no adapter load,
    /// stderr warning at the call site).
    CalibrationMissing,
    /// Gate is either wire-dark (`enabled = false`) or the
    /// calibration report passes the threshold. Either way: clear to
    /// proceed with adapter load.
    CalibrationPassed,
}

impl LoraShipGateOutcome {
    /// Snake-case wire label used for SSB Kind round-trip and stderr
    /// labels.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Refused => "refused",
            Self::CalibrationMissing => "calibration_missing",
            Self::CalibrationPassed => "calibration_passed",
        }
    }
}

/// Pure ship-gate decision function. Side-effect-free; the caller is
/// responsible for emitting the SSB row via
/// [`notify_lora_ship_gate`] and for stderr / refuse-and-continue
/// handling.
///
/// Semantics (in priority order):
///   1. If `!cfg.enabled` -> `CalibrationPassed` (wire-dark; gate
///      not active).
///   2. If `cfg.ship_gate_token == LORA_SHIP_GATE_DEFERRAL_TOKEN`
///      AND `cfg.confirm_user_understanding` -> `Refused`
///      (intentional deferral; this is the default-by-design
///      posture for E-47).
///   3. If `calibration.is_none()` -> `CalibrationMissing`.
///   4. If `calibration.passes_threshold()` -> `CalibrationPassed`.
///   5. Else -> `Refused`.
#[must_use]
pub fn check_lora_ship_gate(
    cfg: &LoraShipGateConfig,
    calibration: Option<&LoraCalibrationReport>,
) -> LoraShipGateOutcome {
    if !cfg.enabled {
        return LoraShipGateOutcome::CalibrationPassed;
    }
    if cfg.ship_gate_token == LORA_SHIP_GATE_DEFERRAL_TOKEN
        && cfg.confirm_user_understanding
    {
        return LoraShipGateOutcome::Refused;
    }
    match calibration {
        None => LoraShipGateOutcome::CalibrationMissing,
        Some(report) => {
            if report.passes_threshold() {
                LoraShipGateOutcome::CalibrationPassed
            } else {
                LoraShipGateOutcome::Refused
            }
        }
    }
}

/// Returns true iff the env var [`ELAI_LORA_SHIP_GATE_ENABLE_ENV`]
/// is set to a truthy value (`"1"` / `"true"` / `"yes"` / `"on"`,
/// case-insensitive). Wire-dark default: callers receive `false`
/// unless an operator explicitly opts in.
///
/// Mirrors `crate::durability::observability::durability_observability_enabled`
/// and `crate::skill_telemetry::is_enabled`.
#[must_use]
pub fn lora_ship_gate_enabled_from_env() -> bool {
    match std::env::var(ELAI_LORA_SHIP_GATE_ENABLE_ENV) {
        Ok(v) => {
            let lower = v.trim().to_ascii_lowercase();
            matches!(lower.as_str(), "1" | "true" | "yes" | "on")
        }
        Err(_) => false,
    }
}

/// Map a [`LoraShipGateOutcome`] to its
/// [`LoraShipGateEventPhase`] sibling for SSB emission.
#[must_use]
pub fn outcome_to_event_kind(
    outcome: LoraShipGateOutcome,
) -> LoraShipGateEventPhase {
    match outcome {
        LoraShipGateOutcome::Refused => LoraShipGateEventPhase::Refused,
        LoraShipGateOutcome::CalibrationMissing => {
            LoraShipGateEventPhase::CalibrationMissing
        }
        LoraShipGateOutcome::CalibrationPassed => {
            LoraShipGateEventPhase::CalibrationPassed
        }
    }
}

/// Emit a `lora_ship_gate_event` SSB row.
///
/// Mirrors the 5 `notify_*` helpers in
/// `crate::durability::observability`. Returns:
///   * `Ok(None)` -- env gate off, no row written, no error.
///   * `Ok(Some(seq))` -- row written; `seq` is the SSB
///     auto-increment sequence.
///   * `Err(SsbError)` -- propagate the underlying write failure.
pub async fn notify_lora_ship_gate(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    session_id: Option<&str>,
    outcome: LoraShipGateOutcome,
    details_json: Value,
    ts_ms: i64,
) -> Result<Option<i64>, SsbError> {
    if !lora_ship_gate_enabled_from_env() {
        return Ok(None);
    }
    let payload = SsbPayload::LoraShipGateEvent(LoraShipGateEventPayload {
        kind: outcome_to_event_kind(outcome),
        session_id: session_id.map(str::to_owned),
        agent_id: Some(agent_id.to_owned()),
        details_json,
        ts_ms,
    });
    let seq = ssb
        .write_payload(agent_id, superstep, payload, Vec::new())
        .await?;
    Ok(Some(seq))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(enabled: bool, confirm: bool) -> LoraShipGateConfig {
        LoraShipGateConfig {
            ship_gate_token: LORA_SHIP_GATE_DEFERRAL_TOKEN.to_owned(),
            confirm_user_understanding: confirm,
            enabled,
        }
    }

    #[test]
    fn wire_dark_disabled_returns_passed() {
        let outcome = check_lora_ship_gate(&cfg(false, false), None);
        assert_eq!(outcome, LoraShipGateOutcome::CalibrationPassed);
    }

    #[test]
    fn enabled_with_deferral_token_returns_refused() {
        let outcome = check_lora_ship_gate(&cfg(true, true), None);
        assert_eq!(outcome, LoraShipGateOutcome::Refused);
    }

    #[test]
    fn enabled_no_token_no_calibration_returns_missing() {
        let mut c = cfg(true, true);
        c.ship_gate_token = "OVERRIDE_TOKEN".to_owned();
        let outcome = check_lora_ship_gate(&c, None);
        assert_eq!(outcome, LoraShipGateOutcome::CalibrationMissing);
    }

    #[test]
    fn passing_calibration_returns_passed() {
        let mut c = cfg(true, true);
        c.ship_gate_token = "BASELINE_VERIFIED".to_owned();
        let report = LoraCalibrationReport {
            mmlu_delta_pct: -1.0,
            humaneval_delta_pct: 0.5,
        };
        let outcome = check_lora_ship_gate(&c, Some(&report));
        assert_eq!(outcome, LoraShipGateOutcome::CalibrationPassed);
    }

    #[test]
    fn regressing_calibration_returns_refused() {
        let mut c = cfg(true, true);
        c.ship_gate_token = "BASELINE_VERIFIED".to_owned();
        let report = LoraCalibrationReport {
            mmlu_delta_pct: -5.0,
            humaneval_delta_pct: 0.0,
        };
        let outcome = check_lora_ship_gate(&c, Some(&report));
        assert_eq!(outcome, LoraShipGateOutcome::Refused);
    }
}
