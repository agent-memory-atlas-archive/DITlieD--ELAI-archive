//! J-14 — Pure payload builders for ReproPackEvent (TierEvaluated +
//! FirstDivergenceLocated). Split from rra.rs / rra_tiers.rs for MR-1.
//!
//! This module is PURE: it builds [`crate::ssb::ReproPackPayload`] values and
//! has NO `Ssb` handle, NO async write, NO I/O. The CLI / orchestrator layer
//! owns the actual `ssb.write_payload(...)` call (the task contract forbids
//! touching ssb/* write paths from this rider). The builders here mirror the
//! field-fill pattern in [`crate::rra_export::export_fixture_with_clock`] (the
//! FixtureExtracted arm) so all four ReproPackKind arms share one shape.
//!
//! J-14 S5 slice A wires the TierEvaluated + FirstDivergenceLocated arms only.
//! FlowFingerprintMismatch (compiled_pipeline) and FixtureExtracted (rra_export)
//! are already shipped and untouched.

use crate::rra_tiers::{FirstDivergence, TierDowngrade, TierVerdict, TierVerdictLabel};
use crate::ssb::{ReproPackKind, ReproPackPayload};

/// J-14: stable snake_case wire string for a [`TierVerdictLabel`], matching the
/// serde `rename_all = "snake_case"` on the enum (so this is the same string
/// serde would emit). Used to fill `tier_effective` / `tier_requested` on the
/// TierEvaluated payload.
///
/// Returns one of `"action_faithful"` / `"token_faithful"` / `"score_faithful"`.
#[must_use]
pub fn tier_label_wire(label: TierVerdictLabel) -> &'static str {
    match label {
        TierVerdictLabel::ActionFaithful => "action_faithful",
        TierVerdictLabel::TokenFaithful => "token_faithful",
        TierVerdictLabel::ScoreFaithful => "score_faithful",
    }
}

/// J-14: effective tier wire string for a [`TierVerdict`].
///
/// * `ActionFaithful` / `TokenFaithful` / `ScoreFaithful` -> `Some(label)`.
/// * `Downgraded { downgrade }` -> `Some(tier_label_wire(downgrade.to))` (the
///   tier that was actually reached; `tier_effective <= tier_requested`).
/// * `MissingDeterminismBasis` -> `None` (the honest mapping: there is no
///   effective tier when the bundle claims tier>T0 but carries no
///   `determinism_applied` basis, R2: missing evidence = FAIL). The
///   TierEvaluated payload records `tier_requested` so a consumer can see what
///   was asked; `tier_effective = None` signals the negative verdict.
#[must_use]
pub fn tier_effective_wire(tier: &TierVerdict) -> Option<String> {
    match tier {
        TierVerdict::ActionFaithful => Some(tier_label_wire(TierVerdictLabel::ActionFaithful).to_owned()),
        TierVerdict::TokenFaithful => Some(tier_label_wire(TierVerdictLabel::TokenFaithful).to_owned()),
        TierVerdict::ScoreFaithful => Some(tier_label_wire(TierVerdictLabel::ScoreFaithful).to_owned()),
        TierVerdict::Downgraded { downgrade } => {
            Some(tier_label_wire(downgrade.to).to_owned())
        }
        TierVerdict::MissingDeterminismBasis => None,
    }
}

/// J-14: wire strings for the recorded downgrades in a [`TierVerdict`].
///
/// Only the `Downgraded` arm produces records; every other arm returns an
/// empty vec. Each record is a single machine-readable string of the form
/// `"<from>-><to>: <reason>"` (e.g. `"token_faithful->action_faithful: token-stream blake3 equality not yet wired in verify leg"`).
/// The `from` / `to` labels are the stable snake_case wire strings from
/// [`tier_label_wire`] so a consumer does not need to re-derive the serde tag.
///
/// The arrow literal `->` is intentional: this is a MACHINE-READABLE record
/// field on an SSB payload, not prose for a human reader, and the wire format
/// is documented here + in the unit tests. (The maintainer-voice no-`->` rule in
/// output-formatting.md governs prose / commits / handoffs, not structured
/// payload strings.)
#[must_use]
pub fn downgrades_wire(tier: &TierVerdict) -> Vec<String> {
    match tier {
        TierVerdict::Downgraded { downgrade } => {
            vec![format_downgrade(downgrade)]
        }
        _ => Vec::new(),
    }
}

/// Format a single [`TierDowngrade`] as the wire string
/// `"<from>-><to>: <reason>"`.
fn format_downgrade(d: &TierDowngrade) -> String {
    format!(
        "{}->{}: {}",
        tier_label_wire(d.from),
        tier_label_wire(d.to),
        d.reason
    )
}

/// J-14: build a `TierEvaluated` [`ReproPackPayload`] from a graded
/// [`TierVerdict`].
///
/// Fills `tier_effective` (via [`tier_effective_wire`]), `tier_requested`
/// (via [`tier_label_wire`]), and `downgrades` (via [`downgrades_wire`]). All
/// divergence / fingerprint / fixture fields are `None` / empty (this arm
/// carries tier state only). `session_id` is owned; `ts_ms` is the caller's
/// wall-clock unix-ms (the CLI passes `crate::ssb::system_time_ms_i64()`; tests
/// pin it).
#[must_use]
pub fn build_tier_evaluated_payload(
    session_id: &str,
    tier_requested: TierVerdictLabel,
    tier: &TierVerdict,
    ts_ms: i64,
) -> ReproPackPayload {
    ReproPackPayload {
        kind: ReproPackKind::TierEvaluated,
        tier_effective: tier_effective_wire(tier),
        tier_requested: Some(tier_label_wire(tier_requested).to_owned()),
        downgrades: downgrades_wire(tier),
        divergence_position: None,
        expected_value: None,
        actual_value: None,
        template_hash: None,
        expected_sequence_hash: None,
        actual_sequence_hash: None,
        fixture_path: None,
        fixture_blake3: None,
        session_id: session_id.to_owned(),
        ts_ms,
    }
}

/// J-14: build a `FirstDivergenceLocated` [`ReproPackPayload`] from a
/// [`FirstDivergence`] locator.
///
/// Fills `divergence_position` from `fd.position`, `expected_value` /
/// `actual_value` from the lossy-utf8 renderings of `fd.expected` / `fd.got`
/// (the locator stores raw bytes; the payload field is a human-readable string,
/// so lossy utf8 is the honest mapping and never panics on non-utf8). All tier
/// / fingerprint / fixture fields are `None` / empty. `session_id` is owned;
/// `ts_ms` is the caller's wall-clock unix-ms.
#[must_use]
pub fn build_first_divergence_payload(
    session_id: &str,
    fd: &FirstDivergence,
    ts_ms: i64,
) -> ReproPackPayload {
    ReproPackPayload {
        kind: ReproPackKind::FirstDivergenceLocated,
        tier_effective: None,
        tier_requested: None,
        downgrades: Vec::new(),
        divergence_position: Some(fd.position),
        expected_value: Some(String::from_utf8_lossy(&fd.expected).into_owned()),
        actual_value: Some(String::from_utf8_lossy(&fd.got).into_owned()),
        template_hash: None,
        expected_sequence_hash: None,
        actual_sequence_hash: None,
        fixture_path: None,
        fixture_blake3: None,
        session_id: session_id.to_owned(),
        ts_ms,
    }
}

#[cfg(test)]
mod j14_rra_emit_tests {
    use super::*;
    use crate::rra_tiers::{FirstDivergence, TierDowngrade, TierVerdict, TierVerdictLabel};
    use crate::ssb::ReproPackKind;

    // (a) TierEvaluated from a Downgraded T1->T0 verdict records:
    //     - downgrades non-empty, formatted "token_faithful->action_faithful: <reason>"
    //     - tier_effective = "action_faithful" (the tier actually reached)
    //     - tier_requested = "token_faithful" (what was asked)
    //     - divergence / fingerprint / fixture fields all None / empty
    #[test]
    fn tier_evaluated_downgraded_t1_to_t0() {
        let tier = TierVerdict::Downgraded {
            downgrade: TierDowngrade {
                from: TierVerdictLabel::TokenFaithful,
                to: TierVerdictLabel::ActionFaithful,
                reason: "token-stream blake3 equality not yet wired in verify leg".to_owned(),
            },
        };
        let payload = build_tier_evaluated_payload(
            "sess-downgrade",
            TierVerdictLabel::TokenFaithful,
            &tier,
            1_700_000_000_000,
        );

        assert_eq!(payload.kind, ReproPackKind::TierEvaluated);
        assert_eq!(payload.tier_requested.as_deref(), Some("token_faithful"));
        assert_eq!(payload.tier_effective.as_deref(), Some("action_faithful"));
        assert_eq!(payload.downgrades.len(), 1, "downgrades non-empty");
        assert_eq!(
            payload.downgrades[0],
            "token_faithful->action_faithful: token-stream blake3 equality not yet wired in verify leg"
        );
        assert!(payload.divergence_position.is_none());
        assert!(payload.expected_value.is_none());
        assert!(payload.actual_value.is_none());
        assert!(payload.template_hash.is_none());
        assert!(payload.expected_sequence_hash.is_none());
        assert!(payload.actual_sequence_hash.is_none());
        assert!(payload.fixture_path.is_none());
        assert!(payload.fixture_blake3.is_none());
        assert_eq!(payload.session_id, "sess-downgrade");
        assert_eq!(payload.ts_ms, 1_700_000_000_000);
    }

    // (b) FirstDivergenceLocated sets divergence_position + expected/actual
    //     (lossy utf8 from the raw byte locators), and leaves tier / fingerprint
    //     / fixture fields None / empty.
    #[test]
    fn first_divergence_located_sets_position_and_values() {
        let fd = FirstDivergence {
            position: 7,
            expected: b"hello".to_vec(),
            got: b"world".to_vec(),
            tier: TierVerdictLabel::TokenFaithful,
        };
        let payload = build_first_divergence_payload("sess-fd", &fd, 1_700_000_000_001);

        assert_eq!(payload.kind, ReproPackKind::FirstDivergenceLocated);
        assert_eq!(payload.divergence_position, Some(7));
        assert_eq!(payload.expected_value.as_deref(), Some("hello"));
        assert_eq!(payload.actual_value.as_deref(), Some("world"));
        assert!(payload.tier_effective.is_none());
        assert!(payload.tier_requested.is_none());
        assert!(payload.downgrades.is_empty());
        assert!(payload.template_hash.is_none());
        assert!(payload.fixture_path.is_none());
        assert!(payload.fixture_blake3.is_none());
        assert_eq!(payload.session_id, "sess-fd");
        assert_eq!(payload.ts_ms, 1_700_000_000_001);
    }

    // (c) MissingDeterminismBasis -> empty downgrades + honest effective mapping
    //     (tier_effective = None, since there is no effective tier when the
    //     bundle claims tier>T0 but carries no determinism_applied basis).
    //     tier_requested is still recorded so a consumer can see what was asked.
    #[test]
    fn missing_determinism_basis_empty_downgrades_none_effective() {
        let tier = TierVerdict::MissingDeterminismBasis;
        let payload = build_tier_evaluated_payload(
            "sess-missing",
            TierVerdictLabel::TokenFaithful,
            &tier,
            1_700_000_000_002,
        );

        assert_eq!(payload.kind, ReproPackKind::TierEvaluated);
        assert_eq!(payload.tier_requested.as_deref(), Some("token_faithful"));
        assert!(
            payload.tier_effective.is_none(),
            "MissingDeterminismBasis -> tier_effective = None (honest mapping)"
        );
        assert!(payload.downgrades.is_empty(), "no downgrade record on MissingDeterminismBasis");
        assert_eq!(payload.session_id, "sess-missing");
        assert_eq!(payload.ts_ms, 1_700_000_000_002);
    }

    // (d) tier_label_wire round-trips all three labels to the snake_case wire
    //     strings that match the enum's serde rename_all.
    #[test]
    fn tier_label_wire_matches_serde_snake_case() {
        assert_eq!(tier_label_wire(TierVerdictLabel::ActionFaithful), "action_faithful");
        assert_eq!(tier_label_wire(TierVerdictLabel::TokenFaithful), "token_faithful");
        assert_eq!(tier_label_wire(TierVerdictLabel::ScoreFaithful), "score_faithful");

        // Cross-check against the serde serialization (the source of truth for
        // the wire string).
        let json_action = serde_json::to_string(&TierVerdictLabel::ActionFaithful).expect("serde");
        assert_eq!(json_action, "\"action_faithful\"");
        let json_token = serde_json::to_string(&TierVerdictLabel::TokenFaithful).expect("serde");
        assert_eq!(json_token, "\"token_faithful\"");
        let json_score = serde_json::to_string(&TierVerdictLabel::ScoreFaithful).expect("serde");
        assert_eq!(json_score, "\"score_faithful\"");
    }

    // (e) ActionFaithful / TokenFaithful / ScoreFaithful (non-downgraded)
    //     produce empty downgrades and tier_effective == the label.
    #[test]
    fn non_downgraded_tiers_empty_downgrades_effective_equals_label() {
        for (tier, expected_wire) in [
            (TierVerdict::ActionFaithful, "action_faithful"),
            (TierVerdict::TokenFaithful, "token_faithful"),
            (TierVerdict::ScoreFaithful, "score_faithful"),
        ] {
            let payload = build_tier_evaluated_payload(
                "sess-flat",
                TierVerdictLabel::ActionFaithful,
                &tier,
                0,
            );
            assert_eq!(payload.tier_effective.as_deref(), Some(expected_wire));
            assert!(payload.downgrades.is_empty());
        }
    }

    // (f) FirstDivergence with non-utf8 bytes: lossy utf8 never panics, the
    //     payload fields carry the replacement-character renderings.
    #[test]
    fn first_divergence_non_utf8_lossy() {
        let fd = FirstDivergence {
            position: 0,
            expected: vec![0xff, 0xfe, b'a'],
            got: vec![0xff, b'b'],
            tier: TierVerdictLabel::TokenFaithful,
        };
        let payload = build_first_divergence_payload("sess-nonutf8", &fd, 0);
        assert_eq!(payload.divergence_position, Some(0));
        // Lossy: 0xff -> replacement char U+FFFD. Just assert non-empty + no panic.
        assert!(payload.expected_value.is_some());
        assert!(payload.actual_value.is_some());
        assert!(payload.expected_value.as_ref().unwrap().contains('\u{FFFD}'));
    }
}
