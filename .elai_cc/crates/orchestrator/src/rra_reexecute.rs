//! J-91 cassette-only RRA re-execution. Never opens a socket or contacts a live service.
//!
//! This module implements the offline re-execution path: read a `.fixture`
//! cassette, decode the bundled [`RunArtifact`], verify it with
//! [`verify_artifact_with_objective`], and produce a [`ReExecuteVerdict`]
//! describing whether the cassette-only re-execution succeeded and at what
//! tier. The feature is default-OFF behind the `ELAI_REEXECUTE_RRA_ENABLE`
//! kill-switch flag; when the flag is off the verdict is `ok=false` with a
//! `flag_off` note (no re-execution is attempted).

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::rra::{self, RraError, RunArtifact, ReplayVerdict};
use crate::rra_export::{export_fixture, ExportFixtureResult};

/// The env-var name for the J-91 re-executable RRA kill-switch.
pub const ELAI_REEXECUTE_RRA_ENABLE_ENV: &str = "ELAI_REEXECUTE_RRA_ENABLE";

/// Returns `true` when `ELAI_REEXECUTE_RRA_ENABLE` is set to a non-empty,
/// non-`"0"` value. Default-OFF.
pub fn reexecute_rra_enabled() -> bool {
    std::env::var(ELAI_REEXECUTE_RRA_ENABLE_ENV)
        .map(|v| !v.trim().is_empty() && v != "0")
        .unwrap_or(false)
}

/// J-91: pinned environment metadata for cassette-only re-execution. Captures
/// the OCI image digest, lockfile blake3, CAS fixture blake3 list, hardware
/// class, and model pin so a verifier can check that the re-execution
/// environment matches the sealed bundle's pins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PinnedEnv {
    /// OCI container image digest (empty when not pinned).
    pub oci_digest: String,
    /// blake3 hex of the Cargo.lock bytes (empty when not pinned).
    pub lockfile_blake3: String,
    /// blake3 hex prefixes of CAS fixture artifacts.
    pub cas_fixture_blake3: Vec<String>,
    /// Hardware class label from the sealed bundle.
    pub hardware_class: String,
    /// Model identity from the sealed bundle (`"unknown"` when no model swap).
    pub model_pin: String,
}

/// J-91: the verdict of a cassette-only re-execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReExecuteVerdict {
    /// Whether the re-execution succeeded (cassette verified + env pins OK).
    pub ok: bool,
    /// The tier actually reached (always `"T0"` for this unit).
    pub tier_effective: String,
    /// The tier requested by the caller (`"T0"` / `"T1"` / `"T2"`).
    pub tier_requested: String,
    /// Recorded downgrades (tier or env-pin shortfalls).
    pub downgrades: Vec<String>,
    /// When `claimed_objective` was supplied: whether the sealed objective hash
    /// matched. `None` when no objective was claimed.
    pub objective_seal_match: Option<bool>,
    /// blake3 16-hex prefix of the cassette bundle bytes.
    pub cassette_blake3: String,
    /// Always `true` — this path never contacts a live service.
    pub cassette_only: bool,
    /// Diagnostic notes (e.g. `flag_off`, `cassette_only_no_network`).
    pub notes: Vec<String>,
}

/// J-91: derive a [`PinnedEnv`] from a sealed [`RunArtifact`] plus optional
/// environment pins. `hardware_class` and `model_pin` come from the artifact;
/// `oci_digest`, `lockfile_blake3`, and `cas_fixture_blake3` are supplied by
/// the caller when available.
pub fn pin_environment(
    art: &RunArtifact,
    oci_digest: Option<&str>,
    lockfile_bytes: Option<&[u8]>,
    cas_fixture_blake3: &[String],
) -> PinnedEnv {
    PinnedEnv {
        oci_digest: oci_digest.unwrap_or("").to_owned(),
        lockfile_blake3: lockfile_bytes
            .map(|b| blake3::hash(b).to_hex().to_string())
            .unwrap_or_default(),
        cas_fixture_blake3: cas_fixture_blake3.to_vec(),
        hardware_class: art.hardware_class.clone(),
        model_pin: art.model_pin.clone().unwrap_or_else(|| "unknown".to_owned()),
    }
}

/// J-91: export a content-keyed cassette from an RRA bundle, then refuse to
/// return it if the scrubbed body still contains an `authorization:` header
/// (case-insensitive) or if a supplied `taint_bytes` marker is absent (an
/// incomplete capture signal). Delegates scrubbing to the existing
/// [`export_fixture`] path.
pub fn export_cassette(
    bundle: &Path,
    adapter: &str,
    tool_name: &str,
    taint_bytes: Option<&[u8]>,
    out_dir: Option<&Path>,
) -> Result<ExportFixtureResult, RraError> {
    let result = export_fixture(bundle, adapter, tool_name, out_dir)?;
    // Refuse if scrubbed body still contains ASCII "authorization:" (case-insensitive).
    let lower = result.cassette.bundle_bytes.to_ascii_lowercase();
    if lower.windows(14).any(|w| w == b"authorization:") {
        return Err(RraError::Msgpack(
            "unscrubbed authorization header in cassette".into(),
        ));
    }
    // If a taint marker was supplied, it must be present in the cassette body
    // (an incomplete capture signal: the taint was in the source but did not
    // survive into the fixture).
    if let Some(taint) = taint_bytes {
        if !taint.is_empty()
            && !result
                .cassette
                .bundle_bytes
                .windows(taint.len())
                .any(|w| w == taint)
        {
            return Err(RraError::Msgpack(
                "taint_bytes not present in cassette body (incomplete capture)".into(),
            ));
        }
    }
    Ok(result)
}

/// J-91: cassette-only re-execution entrypoint. Reads the flag from the
/// environment and delegates to [`re_execute_rra_checked`].
pub fn re_execute_rra(
    cassette_path: &Path,
    pinned: &PinnedEnv,
    tier_requested: &str,
    claimed_objective: Option<&str>,
) -> Result<ReExecuteVerdict, RraError> {
    re_execute_rra_checked(
        cassette_path,
        pinned,
        tier_requested,
        claimed_objective,
        reexecute_rra_enabled(),
    )
}

/// J-91: cassette-only re-execution core. The `enabled` parameter is injected
/// by [`re_execute_rra`] from the env flag; tests pass it explicitly.
///
/// Steps:
/// 1. Read + decode the `FixtureCassette` from `cassette_path` (msgpack).
/// 2. If `!enabled` → `ok=false`, `notes=["flag_off"]`, `cassette_only=true`.
/// 3. Decode `cassette.bundle_bytes` as a `RunArtifact`.
/// 4. `verify_artifact_with_objective(&art, claimed_objective)`.
/// 5. `Verified` or `ObjectiveUnchecked` → proceed; else `ok=false`.
/// 6. If `claimed_objective` is `Some`: `objective_seal_match = Some(matches Verified)`.
/// 7. T1/T2 request → downgrade `non_batch_invariant_or_unproven`; `tier_effective` always `"T0"`.
/// 8. If `pinned.oci_digest` is empty → downgrade `env_pin_incomplete_oci`.
/// 9. `ok=true`, push `cassette_only_no_network`, `cassette_only=true`.
pub fn re_execute_rra_checked(
    cassette_path: &Path,
    pinned: &PinnedEnv,
    tier_requested: &str,
    claimed_objective: Option<&str>,
    enabled: bool,
) -> Result<ReExecuteVerdict, RraError> {
    // 1. Read + decode the FixtureCassette from cassette_path (msgpack).
    let cassette_bytes = std::fs::read(cassette_path).map_err(|e| RraError::Io {
        path: cassette_path.display().to_string(),
        detail: e.to_string(),
    })?;
    let cassette: crate::rra_export::FixtureCassette =
        rmp_serde::from_slice(&cassette_bytes).map_err(|e| RraError::Msgpack(e.to_string()))?;

    let cassette_blake3 = crate::rra::truncate_hex(&blake3::hash(&cassette.bundle_bytes).to_hex());

    // 2. If !enabled -> ok=false, notes=["flag_off"], cassette_only=true, return Ok.
    if !enabled {
        return Ok(ReExecuteVerdict {
            ok: false,
            tier_effective: "T0".to_owned(),
            tier_requested: tier_requested.to_owned(),
            downgrades: Vec::new(),
            objective_seal_match: None,
            cassette_blake3,
            cassette_only: true,
            notes: vec!["flag_off".to_owned()],
        });
    }

    // 3. Decode cassette.bundle_bytes as RunArtifact.
    let art: RunArtifact = rmp_serde::from_slice(&cassette.bundle_bytes)
        .map_err(|e| RraError::Msgpack(e.to_string()))?;

    // 4. verify_artifact_with_objective(&art, claimed_objective).
    let verdict = rra::verify_artifact_with_objective(&art, claimed_objective)?;

    // 5. verified = Verified | ObjectiveUnchecked; else notes verification_failed, ok=false.
    let verified = matches!(
        verdict,
        ReplayVerdict::Verified | ReplayVerdict::ObjectiveUnchecked { .. }
    );

    // 6. If claimed_objective Some: objective_seal_match = Some(matches Verified).
    let objective_seal_match = claimed_objective.map(|_| matches!(verdict, ReplayVerdict::Verified));

    let mut downgrades = Vec::new();
    let mut notes = Vec::new();

    if !verified {
        notes.push(format!("verification_failed: {:?}", verdict));
        return Ok(ReExecuteVerdict {
            ok: false,
            tier_effective: "T0".to_owned(),
            tier_requested: tier_requested.to_owned(),
            downgrades,
            objective_seal_match,
            cassette_blake3,
            cassette_only: true,
            notes,
        });
    }

    // 7. If tier_requested is T1 or T2: downgrade non_batch_invariant_or_unproven;
    //    tier_effective always "T0" for this unit.
    let tier_upper = tier_requested.to_ascii_uppercase();
    if tier_upper == "T1" || tier_upper == "T2" {
        downgrades.push(crate::attention_determinism_tier::ATTENTION_NOT_BATCH_INVARIANT_REASON.to_owned());
    }

    // 8. If pinned.oci_digest empty: downgrade env_pin_incomplete_oci.
    if pinned.oci_digest.is_empty() {
        downgrades.push("env_pin_incomplete_oci".to_owned());
    }

    // 9. ok=true, notes push cassette_only_no_network, cassette_only=true.
    notes.push("cassette_only_no_network".to_owned());

    Ok(ReExecuteVerdict {
        ok: true,
        tier_effective: "T0".to_owned(),
        tier_requested: tier_requested.to_owned(),
        downgrades,
        objective_seal_match,
        cassette_blake3,
        cassette_only: true,
        notes,
    })
}

#[cfg(test)]
mod j91_reexecute_tests {
    use super::*;
    use crate::determinism::{DeterminismConfig, HardwareClass};
    use crate::rra::{build_rra_with_sampling_seed, write_rra};
    use crate::rra_export::FixtureCassette;
    use crate::ssb::{SsbActionType, SsbEntry};
    use std::sync::Arc;

    fn test_pins() -> DeterminismConfig {
        DeterminismConfig {
            seed: 42,
            hardware_class: HardwareClass::Cpu,
            blake3_cache_prefix: [0u8; 8],
            system_fingerprint_pin: None,
        }
    }

    fn write_bundle(tmp: &std::path::Path) -> RunArtifact {
        let entry = SsbEntry {
            seq: 1,
            project_id: "j91-proj".to_owned(),
            session_id: "j91-reexec".to_owned(),
            agent_id: "test".to_owned(),
            superstep: 0,
            action_type: SsbActionType::IdeaFramingStarted,
            payload: Arc::from(b"hello j91".as_ref()),
            created_at: 0,
        };
        let pins = test_pins();
        let art = build_rra_with_sampling_seed("j91-reexec", &[entry], &pins, None);
        write_rra(&art, tmp).expect("write bundle");
        art
    }

    // 1. export_cassette roundtrip + re-execute at T0 succeeds.
    #[test]
    fn export_cassette_roundtrip_and_reexecute_t0() {
        let tmp = std::env::temp_dir().join(format!(
            "j91-reexec-t0-{}.rra",
            std::process::id()
        ));
        let out_dir = std::env::temp_dir().join(format!(
            "j91-reexec-t0-{}",
            std::process::id()
        ));
        let art = write_bundle(&tmp);

        let export_result = export_cassette(&tmp, "demo", "", None, Some(&out_dir))
            .expect("export cassette");

        let pinned = pin_environment(&art, None, None, &[]);
        let verdict = re_execute_rra_checked(
            &export_result.fixture_path,
            &pinned,
            "T0",
            None,
            true, // enabled
        )
        .expect("re-execute");

        assert!(verdict.ok, "T0 re-execute should succeed: {:?}", verdict);
        assert_eq!(verdict.tier_effective, "T0");
        assert_eq!(verdict.tier_requested, "T0");
        assert!(verdict.cassette_only);
        assert!(
            verdict.notes.contains(&"cassette_only_no_network".to_owned()),
            "notes should include cassette_only_no_network: {:?}",
            verdict.notes
        );
        // oci_digest empty -> env_pin_incomplete_oci downgrade present.
        assert!(
            verdict.downgrades.contains(&"env_pin_incomplete_oci".to_owned()),
            "downgrades should include env_pin_incomplete_oci: {:?}",
            verdict.downgrades
        );

        let _ = std::fs::remove_file(&tmp);
        let _ = std::fs::remove_dir_all(&out_dir);
    }

    // 2. Flag off -> ok=false, notes=["flag_off"].
    #[test]
    fn reexecute_flag_off_returns_not_ok() {
        let tmp = std::env::temp_dir().join(format!(
            "j91-reexec-off-{}.rra",
            std::process::id()
        ));
        let out_dir = std::env::temp_dir().join(format!(
            "j91-reexec-off-{}",
            std::process::id()
        ));
        let art = write_bundle(&tmp);

        let export_result = export_cassette(&tmp, "", "", None, Some(&out_dir))
            .expect("export cassette");

        let pinned = pin_environment(&art, None, None, &[]);
        let verdict = re_execute_rra_checked(
            &export_result.fixture_path,
            &pinned,
            "T0",
            None,
            false, // flag OFF
        )
        .expect("re-execute");

        assert!(!verdict.ok, "flag off should return ok=false");
        assert_eq!(verdict.notes, vec!["flag_off".to_owned()]);
        assert!(verdict.cassette_only);

        let _ = std::fs::remove_file(&tmp);
        let _ = std::fs::remove_dir_all(&out_dir);
    }

    // 3. T1 request downgrades when unproven (non_batch_invariant_or_unproven).
    #[test]
    fn t1_request_downgrades_when_unproven() {
        let tmp = std::env::temp_dir().join(format!(
            "j91-reexec-t1-{}.rra",
            std::process::id()
        ));
        let out_dir = std::env::temp_dir().join(format!(
            "j91-reexec-t1-{}",
            std::process::id()
        ));
        let art = write_bundle(&tmp);

        let export_result = export_cassette(&tmp, "", "", None, Some(&out_dir))
            .expect("export cassette");

        let pinned = pin_environment(&art, None, None, &[]);
        let verdict = re_execute_rra_checked(
            &export_result.fixture_path,
            &pinned,
            "T1",
            None,
            true,
        )
        .expect("re-execute");

        // ok=true (cassette verified at T0), but downgrades include the T1 shortfall.
        assert!(verdict.ok, "T1 request should still be ok at T0: {:?}", verdict);
        assert_eq!(verdict.tier_effective, "T0");
        assert_eq!(verdict.tier_requested, "T1");
        assert!(
            verdict.downgrades.contains(&crate::attention_determinism_tier::ATTENTION_NOT_BATCH_INVARIANT_REASON.to_owned()),
            "T1 request should downgrade non_batch_invariant_or_unproven: {:?}",
            verdict.downgrades
        );

        let _ = std::fs::remove_file(&tmp);
        let _ = std::fs::remove_dir_all(&out_dir);
    }

    // 4. export_cassette scrub strips bearer secret from the cassette body.
    #[test]
    fn export_cassette_scrub_strips_bearer_secret() {
        let tmp = std::env::temp_dir().join(format!(
            "j91-reexec-scrub-{}.rra",
            std::process::id()
        ));
        let out_dir = std::env::temp_dir().join(format!(
            "j91-reexec-scrub-{}",
            std::process::id()
        ));

        // Build a bundle whose raw msgpack bytes carry an Authorization header
        // in an event payload so the scrub-before-export has something to strip.
        let auth_payload: Arc<[u8]> =
            Arc::from(b"Authorization: Bearer secret123\n".as_ref());
        let entry = SsbEntry {
            seq: 1,
            project_id: "j91-scrub-proj".to_owned(),
            session_id: "j91-scrub".to_owned(),
            agent_id: "test".to_owned(),
            superstep: 0,
            action_type: SsbActionType::IdeaFramingStarted,
            payload: auth_payload,
            created_at: 0,
        };
        let pins = test_pins();
        let art = build_rra_with_sampling_seed("j91-scrub", &[entry], &pins, None);
        write_rra(&art, &tmp).expect("write bundle");

        let result = export_cassette(&tmp, "", "", None, Some(&out_dir))
            .expect("export cassette");

        // After export, bundle_bytes must not contain `secret123`.
        let body_str = String::from_utf8_lossy(&result.cassette.bundle_bytes);
        assert!(
            !body_str.contains("secret123"),
            "scrubbed cassette body must not contain secret123: {:?}",
            body_str
        );

        let _ = std::fs::remove_file(&tmp);
        let _ = std::fs::remove_dir_all(&out_dir);
    }
}
