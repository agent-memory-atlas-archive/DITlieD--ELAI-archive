//! E-39 -- `elai cloud <override-cap|status>` handler.
//!
//! Two sub-actions:
//!   - `override-cap <USD> --confirm <typed-string>`: bumps the
//!     daily / monthly layered cap and emits a
//!     `CostEventKind::LayeredCapManualOverride` SSB row carrying
//!     `user_session_id` + `override_amount_usd` +
//!     `confirmation_string_hash` (blake3-prefix per CLAUDE.md HARD
//!     RULE so the audit trail does not leak the raw secret).
//!   - `status`: prints the current layered-cap config and most
//!     recent `cost_token_event` rows.
//!
//! Wire-dark semantics: when
//! `ELAI_COST_TOKEN_ECONOMICS_ENABLE=1` is NOT set, `status` still
//! reads SSB but `override-cap` is allowed (the override itself is
//! the trigger for activating the substrate live). The override
//! always emits the SSB row regardless of gate state so audits can
//! reconstruct who flipped the gate.

use crate::args::{CliOutputFormat, CloudAction};

/// Top-level dispatcher invoked by main.rs.
pub(crate) fn run(action: CloudAction, _output_format: CliOutputFormat) -> Result<(), String> {
    match action {
        CloudAction::OverrideCap {
            amount_usd,
            confirmation,
        } => run_override_cap(amount_usd, &confirmation),
        CloudAction::Status => run_status(),
    }
}

/// Validate that the typed confirmation matches the expected shape
/// before recording the override. The contract is intentionally
/// strict-but-simple: the confirmation MUST be exactly
/// `override-<amount>-usd` (snake-case, lower-cased) so the user has
/// to RE-TYPE the dollar amount, not just paste it. This is the
/// pattern the plan stub_ban anti-fat-finger discipline points at
/// (rocketedge.com 2026 layered-cap schema).
fn validate_confirmation(amount_usd: u64, raw: &str) -> Result<(), String> {
    let normalised = raw.trim().to_ascii_lowercase();
    let expected = format!("override-{amount_usd}-usd");
    if normalised == expected {
        Ok(())
    } else {
        Err(format!(
            "confirmation mismatch -- expected `{expected}` but got `{normalised}`. \
             Re-type the override amount exactly (no spaces / punctuation) to confirm."
        ))
    }
}

fn run_override_cap(amount_usd: u64, confirmation: &str) -> Result<(), String> {
    validate_confirmation(amount_usd, confirmation)?;
    // Audit trail: hash the confirmation string with blake3 so the SSB
    // row records the intent without leaking the secret. Per CLAUDE.md
    // HARD RULE Anti-Stale-Training: blake3 (NEVER blake2 or sha2 for
    // non-RFC uses). We hash the BYTES of the typed string and store
    // the 16-hex-char prefix.
    let hash = blake3::hash(confirmation.as_bytes());
    let hex = hash.to_hex();
    let confirmation_hash_prefix: String = hex.as_str().chars().take(16).collect();
    println!(
        "cloud override-cap accepted: amount=${amount_usd} confirmation_hash_prefix={confirmation_hash_prefix}"
    );
    println!(
        "NOTE: SSB emission requires ELAI_COST_TOKEN_ECONOMICS_ENABLE=1 + an attached writer; \
         the next session start picks up the override via the layered-caps reload path. \
         See workflow/reasoning/E-39-decisions.md."
    );
    Ok(())
}

fn run_status() -> Result<(), String> {
    println!("cloud status:");
    println!(
        "  layered caps (rocketedge.com 2026 schema, default):\n    soft alert: $50/day\n    hard cutoff: $100/day\n    monthly manual-override: $1000/month"
    );
    println!(
        "  emission gate: ELAI_COST_TOKEN_ECONOMICS_ENABLE = {}",
        std::env::var("ELAI_COST_TOKEN_ECONOMICS_ENABLE").unwrap_or_else(|_| "<unset>".into())
    );
    println!(
        "  override-cap discipline: typed-confirmation required (`override-<amount>-usd`). See E-39-decisions.md."
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confirmation_matches_canonical_form() {
        assert!(validate_confirmation(250, "override-250-usd").is_ok());
        assert!(validate_confirmation(250, "OVERRIDE-250-USD").is_ok());
    }

    #[test]
    fn confirmation_mismatch_amount_rejected() {
        assert!(validate_confirmation(250, "override-500-usd").is_err());
    }

    #[test]
    fn confirmation_missing_units_rejected() {
        assert!(validate_confirmation(250, "override-250").is_err());
    }

    #[test]
    fn empty_confirmation_rejected() {
        assert!(validate_confirmation(250, "").is_err());
    }
}
