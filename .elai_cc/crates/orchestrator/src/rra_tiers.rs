//! J-14 — Tiered verify path for the RRA (split from rra.rs for MR-1).
//!
//! This module holds the J-14 additions to the G-2 Reproducible-Run Artifact:
//! the graded tier verdict types, the determinism-applied block, the
//! tiered verify entrypoint, the auth-header scrub, and the J-14 unit tests.
//! rra.rs re-exports the public surface so existing CLI imports
//! (`elai_orchestrator::rra::{verify_artifact_tiered, TierVerdict, ...}`)
//! continue to resolve.
//!
//! The tier logic (R2: missing evidence = FAIL):
//! * T0 (ActionFaithful) — the action stream re-derives byte-identically under
//!   the pinned seed AND every present F-gate verdict hash re-checks (today's
//!   G-2 `Verified` verdict). The honest ceiling for any run that is not
//!   provably token-faithful.
//! * T1 (TokenFaithful) — token-stream blake3 equality (passable once J-13
//!   ships). The bundle carries `determinism_applied` with a real basis.
//! * T2 (ScoreFaithful) — score/logprob equality where the lane exposes
//!   scores. EXACT equality, never epsilon (detllm gotcha).

use serde::{Deserialize, Serialize};

use crate::determinism::DeterminismConfig;
use crate::rra::{
    compute_root, hardware_class_label, truncate_hex, verify_artifact_with_objective,
    ReplayVerdict, RraError, RunArtifact,
};

/// J-14: Self-describing record of which determinism controls were ACTUALLY
/// active at run time (detllm steal, reimplemented in our vocabulary). The
/// bundle states its own guarantee basis so a verifier can check that a
/// tier>T0 claim has a real basis (R2: missing evidence = FAIL).
///
/// The fingerprint hash EXCLUDES this struct's own serialized form (detllm
/// gotcha: self-inclusion is a fixed point that hides tamper). The
/// `fingerprint_blake3` field is computed over (seed, sampling_seed,
/// hardware_class, system_fingerprint_pin, env_controls sorted) and is
/// itself excluded from the input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DeterminismApplied {
    /// The process-level `ELAI_SEED` pin that was active (mirrors
    /// [`RunArtifact::seed`]; repeated here so the determinism block is
    /// self-contained).
    pub seed: u64,
    /// The per-request sampling seed that was active (mirrors
    /// [`RunArtifact::sampling_seed`]).
    #[serde(default)]
    pub sampling_seed: Option<u64>,
    /// Snake-case hardware-class label that was active.
    #[serde(default)]
    pub hardware_class: String,
    /// The `system_fingerprint` pin that was active, if any.
    #[serde(default)]
    pub system_fingerprint_pin: Option<String>,
    /// Sorted list of env-control names actually read at run time
    /// (e.g. `["ELAI_SEED", "ELAI_HARDWARE_CLASS"]`). Key-sorted for hash
    /// stability (detllm gotcha: serialization must be key-sorted).
    #[serde(default)]
    pub env_controls: Vec<String>,
    /// blake3 16-hex prefix of the canonical determinism fingerprint
    /// (seed || sampling_seed || hardware_class || system_fingerprint_pin
    /// || sorted env_controls). EXCLUDES this field itself.
    #[serde(default)]
    pub fingerprint_blake3: String,
}

/// J-14: Graded tier verdict (detllm steal, ELAI vocabulary). Replaces the
/// binary G-2 verdict on the tiered verify path. `tier_effective <=
/// tier_requested` always; a downgrade is recorded with `{from, to, reason}`
/// and never silent (R1 analog: strict mode aborts, never silently degrades).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TierVerdict {
    /// T0: the action stream re-derives byte-identically under the pinned
    /// seed AND every present F-gate verdict hash re-checks (today's G-2
    /// `Verified` verdict). The honest ceiling for any run that is not
    /// provably token-faithful.
    ActionFaithful,
    /// T1: token-stream blake3 equality (passable once J-13 ships). The
    /// bundle carries `determinism_applied` with a real basis.
    TokenFaithful,
    /// T2: score/logprob equality where the lane exposes scores. EXACT
    /// equality, never epsilon (detllm gotcha).
    ScoreFaithful,
    /// The requested tier could not be reached; `downgrade` records the
    /// `{from, to, reason}` triple. `tier_effective <= tier_requested`.
    Downgraded {
        /// The downgrade record.
        downgrade: TierDowngrade,
    },
    /// The bundle claims a tier>T0 but carries no `determinism_applied`
    /// block (R2: missing evidence = FAIL). NEGATIVE verdict.
    MissingDeterminismBasis,
}

/// J-14: A recorded tier downgrade `{from, to, reason}`. Every downgrade is
/// recorded; strict mode aborts rather than silently degrading.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TierDowngrade {
    /// The tier that was requested.
    pub from: TierVerdictLabel,
    /// The tier that was actually reached.
    pub to: TierVerdictLabel,
    /// Why the downgrade happened (machine-readable short string).
    pub reason: String,
}

/// J-14: The tier label as a stable wire tag (distinct from
/// [`TierVerdict`] which carries the downgrade case). Used inside
/// [`TierDowngrade`] so the label is serializable independently.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TierVerdictLabel {
    /// T0: action-stream replay matches.
    ActionFaithful,
    /// T1: token-stream blake3 equality.
    TokenFaithful,
    /// T2: score/logprob equality.
    ScoreFaithful,
}

/// J-14: The outcome of the tiered verify path. Carries the existing
/// [`ReplayVerdict`] (tamper/divergence/objective checks unchanged) PLUS the
/// graded [`TierVerdict`] (the J-14 extension). Existing callers of
/// [`crate::rra::verify_artifact`] / [`crate::rra::verify_rra`] keep the
/// binary verdict; the tiered path is a new entrypoint for the CLI slice
/// that renders the graded tier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifyOutcome {
    /// The binary verdict (tamper/divergence/objective). Unchanged from G-2.
    pub verdict: ReplayVerdict,
    /// The graded tier verdict (J-14). `None` only when the binary verdict
    /// is non-`Verified` (tamper/divergence/objective-mismatch short-circuit
    /// the tier check).
    pub tier: Option<TierVerdict>,
}

/// J-14: First-divergence locator for T1/T2 mismatch (detllm steal). On
/// token-stream or score mismatch, records the first divergent position +
/// both values into the bundle diff section so G-10's drift canary can
/// report "diverged at token N" exactly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FirstDivergence {
    /// The position of the first divergence (token index for T1, score
    /// index for T2). 0-based.
    pub position: u64,
    /// The expected value at `position` (sealed at build time).
    pub expected: Vec<u8>,
    /// The value recomputed now at `position`.
    pub got: Vec<u8>,
    /// The tier this divergence was detected at (T1 token / T2 score).
    pub tier: TierVerdictLabel,
}

/// J-14: Seal the `determinism_applied` block from the live run pins.
///
/// `env_controls` is the sorted list of env-control names actually read at
/// run time. The producer threads the canonical set
/// (`["ELAI_HARDWARE_CLASS", "ELAI_SEED", "ELAI_SYSTEM_FINGERPRINT"]`);
/// later slices may extend it. Key-sorted for hash stability.
pub(super) fn seal_determinism_applied(
    pins: &DeterminismConfig,
    sampling_seed: Option<u64>,
) -> DeterminismApplied {
    let hardware_class = hardware_class_label(&pins.hardware_class);
    let mut env_controls = vec![
        "ELAI_SEED".to_owned(),
        "ELAI_HARDWARE_CLASS".to_owned(),
    ];
    if pins.system_fingerprint_pin.is_some() {
        env_controls.push("ELAI_SYSTEM_FINGERPRINT".to_owned());
    }
    env_controls.sort();

    // fingerprint_blake3 = blake3 over (seed || sampling_seed || hardware_class
    // || system_fingerprint_pin || sorted env_controls). EXCLUDES the
    // fingerprint_blake3 field itself (detllm gotcha).
    let mut input = Vec::new();
    input.extend_from_slice(&pins.seed.to_le_bytes());
    if let Some(ss) = sampling_seed {
        input.extend_from_slice(&ss.to_le_bytes());
    } else {
        // LEGACY/no-sampling-seed marker: a single 0x00 byte distinguishes
        // None from Some(0) (which serializes as 8 bytes of zeros).
        input.push(0x00);
    }
    input.extend_from_slice(hardware_class.as_bytes());
    input.push(0x00);
    if let Some(fp) = &pins.system_fingerprint_pin {
        input.extend_from_slice(fp.as_bytes());
    }
    input.push(0x00);
    for ctrl in &env_controls {
        input.extend_from_slice(ctrl.as_bytes());
        input.push(0x00);
    }
    let fingerprint_blake3 = truncate_hex(&blake3::hash(&input).to_hex());

    DeterminismApplied {
        seed: pins.seed,
        sampling_seed,
        hardware_class,
        system_fingerprint_pin: pins.system_fingerprint_pin.clone(),
        env_controls,
        fingerprint_blake3,
    }
}

/// J-14: blake3-keyed auth-header scrub (G-9 DARK fallback, inline-minimal).
/// G-9 is DARK (`ELAI_RETENTION_PURGE_ENABLE` default-off), so this is an
/// INLINE-MINIMAL implementation that composes G-9's redaction contract
/// without importing nonexistent G-9 modules (S3 fork verdict).
///
/// Scrubs Authorization/Proxy-Authorization/X-API-Key headers from a byte
/// stream by replacing their values with a blake3-keyed redaction marker.
/// The key is derived from the bundle's `seed` so the scrub is deterministic
/// and verifiable. Returns the scrubbed bytes.
///
/// NEGATIVE-TESTED: scrubbed bytes contain 0 auth-header hits (the test
/// `auth_header_scrub_removes_all_secrets` verifies this).
pub fn scrub_auth_headers(input: &[u8], seed: u64) -> Vec<u8> {
    let key = blake3::derive_key("elai.rra.auth-scrub.v1", &seed.to_le_bytes());
    let redaction_marker = b"<redacted:";
    let redaction_suffix = b">";

    let patterns: &[&[u8]] = &[
        b"authorization:",
        b"proxy-authorization:",
        b"x-api-key:",
        b"Authorization:",
        b"Proxy-Authorization:",
        b"X-API-Key:",
    ];

    let mut out = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        let mut matched = false;
        for pat in patterns {
            if i + pat.len() <= input.len() && input[i..i + pat.len()].eq_ignore_ascii_case(pat) {
                // Find the end of the header value (newline or end of input).
                let value_start = i + pat.len();
                let value_end = input[value_start..]
                    .iter()
                    .position(|&b| b == b'\n')
                    .map_or(input.len(), |p| value_start + p);

                // blake3-keyed redaction tag over the original value bytes
                // (so the same value produces the same tag deterministically,
                // but the original value is unrecoverable without the key).
                let value_bytes = &input[value_start..value_end];
                let tag = blake3::keyed_hash(&key, value_bytes);
                let tag_hex = tag.to_hex();
                let tag_prefix: String = tag_hex.chars().take(8).collect();

                out.extend_from_slice(redaction_marker);
                out.extend_from_slice(tag_prefix.as_bytes());
                out.extend_from_slice(redaction_suffix);
                i = value_end;
                matched = true;
                break;
            }
        }
        if !matched {
            out.push(input[i]);
            i += 1;
        }
    }
    out
}

/// J-14: Tiered verify path. Returns the binary [`ReplayVerdict`] (tamper /
/// divergence / objective checks unchanged from G-2) PLUS the graded
/// [`TierVerdict`] (the J-14 extension). Existing callers of
/// [`crate::rra::verify_artifact`] / [`crate::rra::verify_rra`] keep the
/// binary verdict; the tiered path is the new entrypoint for the CLI slice
/// that renders the graded tier.
///
/// Tier logic (R2: missing evidence = FAIL):
/// * If the binary verdict is non-`Verified`, `tier` is `None` (the binary
///   failure short-circuits the tier check).
/// * If the bundle carries no `determinism_applied` block (LEGACY pre-J-14
///   or a producer that does not yet populate it), the tier is
///   [`TierVerdict::MissingDeterminismBasis`] when `tier_requested > T0`,
///   and [`TierVerdict::ActionFaithful`] when `tier_requested == T0` (T0
///   does not require a determinism basis — it is the action-stream-only
///   tier that today's G-2 already verifies).
/// * If the bundle carries `determinism_applied`, the tier is
///   [`TierVerdict::ActionFaithful`] for T0 (the honest ceiling today; T1
///   and T2 land in later slices once J-13's token-faithful path is wired
///   through the verify leg). A `tier_requested > T0` that cannot be
///   reached is recorded as [`TierVerdict::Downgraded`] with
///   `{from, to, reason}`.
///
/// `tier_requested` is the tier the caller wants to verify against. The
/// effective tier is always `<= tier_requested` (never a silent upgrade).
pub fn verify_artifact_tiered(
    art: &RunArtifact,
    current_objective: Option<&str>,
    tier_requested: TierVerdictLabel,
) -> Result<VerifyOutcome, RraError> {
    let verdict = verify_artifact_with_objective(art, current_objective)?;

    // Binary failure short-circuits the tier check.
    if !matches!(verdict, ReplayVerdict::Verified) {
        return Ok(VerifyOutcome { verdict, tier: None });
    }

    let tier = grade_tier(art, tier_requested);
    Ok(VerifyOutcome { verdict, tier: Some(tier) })
}

/// J-14: Grade the tier verdict for a `Verified` bundle. See
/// [`verify_artifact_tiered`] for the tier logic.
fn grade_tier(art: &RunArtifact, tier_requested: TierVerdictLabel) -> TierVerdict {
    use TierVerdictLabel as L;
    match (&art.determinism_applied, tier_requested) {
        // T0 request: always ActionFaithful when the bundle verifies (the
        // action stream re-derives, which is exactly what G-2 checked).
        (_, L::ActionFaithful) => TierVerdict::ActionFaithful,

        // T1/T2 request without a determinism_applied block: MISSING basis
        // (R2: missing evidence = FAIL, never a silent pass).
        (None, L::TokenFaithful) | (None, L::ScoreFaithful) => {
            TierVerdict::MissingDeterminismBasis
        }

        // T1/T2 request WITH a determinism_applied block: today's verify leg
        // only checks the action stream (T0), so a T1/T2 request is
        // downgraded to T0 with a recorded reason. Later slices wire the
        // token-stream blake3 equality (T1) once J-13's path is threaded
        // through verify.
        (Some(da), L::TokenFaithful) => {
            use crate::attention_determinism_tier::{
                attention_batch_invariant_from_controls, attention_determinism_tier,
                AttentionDeterminismTier,
            };
            match attention_determinism_tier(
                attention_batch_invariant_from_controls(&da.env_controls),
            ) {
                AttentionDeterminismTier::T0Forced { reason } => TierVerdict::Downgraded {
                    downgrade: TierDowngrade {
                        from: L::TokenFaithful,
                        to: L::ActionFaithful,
                        reason: reason.to_owned(),
                    },
                },
                AttentionDeterminismTier::T1Eligible => TierVerdict::Downgraded {
                    downgrade: TierDowngrade {
                        from: L::TokenFaithful,
                        to: L::ActionFaithful,
                        reason: "token-stream blake3 equality not yet wired in verify leg"
                            .to_owned(),
                    },
                },
            }
        },
        (Some(_da), L::ScoreFaithful) => TierVerdict::Downgraded {
            downgrade: TierDowngrade {
                from: L::ScoreFaithful,
                to: L::ActionFaithful,
                reason: "score/logprob equality not yet wired in verify leg".to_owned(),
            },
        },
    }
}

#[cfg(test)]
mod j14_tiered_verify_tests {
    use super::*;
    use crate::determinism::{DeterminismConfig, HardwareClass};
    use crate::rra::{
        build_rra, build_rra_with_sampling_seed, compute_root, write_rra,
        read_rra, ReplayVerdict,
    };

    fn test_pins() -> DeterminismConfig {
        DeterminismConfig {
            seed: 42,
            hardware_class: HardwareClass::Cpu,
            blake3_cache_prefix: [0u8; 8],
            system_fingerprint_pin: None,
        }
    }

    // (a) T0 action-faithful: a bundle built with determinism_applied verifies
    // and the tiered path returns ActionFaithful for a T0 request.
    #[test]
    fn t0_action_faithful_verifies() {
        let pins = test_pins();
        let art = build_rra("j14-t0", &[], &pins);
        assert!(
            art.determinism_applied.is_some(),
            "build_rra must populate determinism_applied (J-14 producer)"
        );

        let outcome = verify_artifact_tiered(
            &art,
            None,
            TierVerdictLabel::ActionFaithful,
        )
        .expect("tiered verify");
        assert_eq!(outcome.verdict, ReplayVerdict::Verified);
        assert_eq!(
            outcome.tier,
            Some(TierVerdict::ActionFaithful),
            "T0 request on a verified bundle with determinism_applied -> ActionFaithful"
        );
    }

    // (b) missing determinism_applied when tier requires it -> fail. A bundle
    // that claims tier>T0 without determinism_applied FAILS verify (R2:
    // missing evidence = FAIL, never a silent pass).
    #[test]
    fn missing_determinism_applied_fails_t1_request() {
        let pins = test_pins();
        let mut art = build_rra("j14-missing-da", &[], &pins);
        // Strip the determinism_applied block to simulate a bundle that claims
        // tier>T0 but carries no basis (the NEGATIVE R2 case).
        art.determinism_applied = None;
        // Re-seal the root so the bundle is internally consistent (the tamper
        // check would otherwise fire; we want to test the tier check, not the
        // tamper check).
        art.root_blake3 = compute_root(&art).expect("re-seal root");

        let outcome = verify_artifact_tiered(
            &art,
            None,
            TierVerdictLabel::TokenFaithful,
        )
        .expect("tiered verify");
        assert_eq!(outcome.verdict, ReplayVerdict::Verified);
        assert_eq!(
            outcome.tier,
            Some(TierVerdict::MissingDeterminismBasis),
            "T1 request without determinism_applied -> MissingDeterminismBasis (R2 FAIL)"
        );

        // Same for T2.
        let outcome_t2 = verify_artifact_tiered(
            &art,
            None,
            TierVerdictLabel::ScoreFaithful,
        )
        .expect("tiered verify t2");
        assert_eq!(
            outcome_t2.tier,
            Some(TierVerdict::MissingDeterminismBasis),
            "T2 request without determinism_applied -> MissingDeterminismBasis (R2 FAIL)"
        );

        // But a T0 request on the same bundle still passes (T0 does not
        // require a determinism basis — it is the action-stream-only tier).
        let outcome_t0 = verify_artifact_tiered(
            &art,
            None,
            TierVerdictLabel::ActionFaithful,
        )
        .expect("tiered verify t0");
        assert_eq!(
            outcome_t0.tier,
            Some(TierVerdict::ActionFaithful),
            "T0 request without determinism_applied -> ActionFaithful (T0 needs no basis)"
        );
    }

    // (c) first-divergence populated on token mismatch fixture. We construct a
    // Divergence verdict with a first_divergence locator and confirm it
    // round-trips through serde (the CLI slice will render it).
    #[test]
    fn first_divergence_roundtrips_on_divergence() {
        let fd = FirstDivergence {
            position: 7,
            expected: b"hello".to_vec(),
            got: b"world".to_vec(),
            tier: TierVerdictLabel::TokenFaithful,
        };
        let verdict = ReplayVerdict::Divergence {
            first_mismatch_event: 3,
            expected_blake3: "abc123".to_owned(),
            got_blake3: "def456".to_owned(),
            first_divergence: Some(fd.clone()),
        };
        let json = serde_json::to_string(&verdict).expect("serialize");
        let back: ReplayVerdict = serde_json::from_str(&json).expect("deserialize");
        match back {
            ReplayVerdict::Divergence {
                first_divergence: Some(back_fd),
                ..
            } => {
                assert_eq!(back_fd, fd, "first_divergence round-trips");
            }
            _ => panic!("expected Divergence with first_divergence"),
        }
    }

    // T1/T2 request WITH determinism_applied -> Downgraded (today's verify leg
    // only checks the action stream; T1/T2 wire in later slices). The
    // downgrade is recorded, never silent.
    #[test]
    fn t1_request_with_da_is_downgraded() {
        let pins = test_pins();
        let art = build_rra("j14-downgrade", &[], &pins);
        let outcome = verify_artifact_tiered(
            &art,
            None,
            TierVerdictLabel::TokenFaithful,
        )
        .expect("tiered verify");
        match outcome.tier {
            Some(TierVerdict::Downgraded { downgrade }) => {
                assert_eq!(downgrade.from, TierVerdictLabel::TokenFaithful);
                assert_eq!(downgrade.to, TierVerdictLabel::ActionFaithful);
                assert_eq!(
                    downgrade.reason,
                    crate::attention_determinism_tier::ATTENTION_NOT_BATCH_INVARIANT_REASON
                );
            }
            other => panic!("expected Downgraded, got {:?}", other),
        }
    }

    // Binary failure short-circuits the tier check: a tampered bundle returns
    // tier=None.
    #[test]
    fn binary_failure_short_circuits_tier() {
        let pins = test_pins();
        let mut art = build_rra("j14-tampered", &[], &pins);
        art.root_blake3 = "0000000000000000".to_owned();
        let outcome = verify_artifact_tiered(
            &art,
            None,
            TierVerdictLabel::ActionFaithful,
        )
        .expect("tiered verify");
        assert!(matches!(outcome.verdict, ReplayVerdict::Tampered { .. }));
        assert_eq!(outcome.tier, None, "binary failure -> tier None");
    }

    // determinism_applied round-trips through serialize -> read -> verify.
    #[test]
    fn determinism_applied_roundtrips() {
        let pins = test_pins();
        let art = build_rra_with_sampling_seed("j14-da-rt", &[], &pins, Some(99));
        assert!(art.determinism_applied.is_some());
        let da = art.determinism_applied.as_ref().unwrap();
        assert_eq!(da.seed, pins.seed);
        assert_eq!(da.sampling_seed, Some(99));
        assert_eq!(da.hardware_class, "cpu");
        assert!(!da.fingerprint_blake3.is_empty(), "fingerprint must be non-empty");

        let tmp = std::env::temp_dir().join(format!(
            "j14-da-rt-{}.rra",
            std::process::id()
        ));
        write_rra(&art, &tmp).expect("write");
        let read_back = read_rra(&tmp).expect("read");
        assert_eq!(read_back.determinism_applied, art.determinism_applied);
        let _ = std::fs::remove_file(&tmp);

        // Determinism: two independent builds under the same input produce the
        // same root (determinism_applied participates in canonical_bundle_bytes).
        let art2 = build_rra_with_sampling_seed("j14-da-rt", &[], &pins, Some(99));
        assert_eq!(art.root_blake3, art2.root_blake3);

        // A different sampling_seed produces a different determinism_applied
        // fingerprint and therefore a different root.
        let art_other = build_rra_with_sampling_seed("j14-da-rt", &[], &pins, Some(100));
        assert_ne!(
            art.root_blake3, art_other.root_blake3,
            "determinism_applied must participate in the root"
        );
    }

    // blake3-keyed auth-header scrub removes all auth-header hits (G-9 DARK
    // fallback, inline-minimal). NEGATIVE-TESTED: scrubbed bytes contain 0
    // auth-header hits.
    #[test]
    fn auth_header_scrub_removes_all_secrets() {
        let input = b"GET / HTTP/1.1\r\nAuthorization: Bearer secret123\r\nX-API-Key: abcdef\r\nHost: example.com\r\n\r\n";
        let scrubbed = scrub_auth_headers(input, 42);

        // The scrubbed bytes contain 0 auth-header hits.
        let lower = String::from_utf8_lossy(&scrubbed).to_ascii_lowercase();
        assert!(
            !lower.contains("authorization:"),
            "scrubbed bytes must not contain authorization: (got {:?})",
            lower
        );
        assert!(
            !lower.contains("x-api-key:"),
            "scrubbed bytes must not contain x-api-key: (got {:?})",
            lower
        );
        assert!(
            !lower.contains("proxy-authorization:"),
            "scrubbed bytes must not contain proxy-authorization:"
        );

        // The redaction marker is present.
        assert!(
            lower.contains("<redacted:"),
            "scrubbed bytes must contain the redaction marker"
        );

        // The non-header parts survive.
        assert!(
            lower.contains("get / http/1.1"),
            "non-header parts must survive scrub"
        );
        assert!(
            lower.contains("host: example.com"),
            "non-auth headers must survive scrub"
        );

        // Determinism: same input + seed -> same scrubbed bytes.
        let scrubbed2 = scrub_auth_headers(input, 42);
        assert_eq!(scrubbed, scrubbed2, "scrub must be deterministic");

        // Different seed -> different scrubbed bytes (the redaction tag is
        // keyed, so the tag differs even though the marker is the same).
        let scrubbed_other = scrub_auth_headers(input, 99);
        assert_ne!(
            scrubbed, scrubbed_other,
            "different seed -> different redaction tags (keyed hash)"
        );
    }
}
