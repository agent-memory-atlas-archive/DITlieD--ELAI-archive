//! J-14 — Export-fixture path for the RRA (split from rra.rs for MR-1).
//!
//! This module holds the J-14 export-fixture surface: the content-keyed
//! cassette key + cassette, the export result, the `export_fixture` /
//! `export_fixture_with_clock` entrypoints, and the J-14 export unit tests.
//! rra.rs re-exports the public surface so existing CLI imports
//! (`elai_orchestrator::rra::{export_fixture, ExportFixtureResult, ...}`)
//! continue to resolve.

use serde::{Deserialize, Serialize};

use crate::rra::{truncate_hex, RraError, RunArtifact};
use crate::rra_tiers::scrub_auth_headers;

/// J-14: content-keyed cassette matching key. The triple
/// `(model, content_blake3, tool_name)` identifies a fixture for replay-based
/// re-execution (J-91 extends the format; do NOT close it / do NOT switch to
/// path-keyed). `content_blake3` is the blake3 16-hex prefix of the scrubbed
/// message-content bytes the fixture captures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixtureCassetteKey {
    /// Model id the fixture was captured under (mirrors `RunArtifact::model_pin`
    /// when present; `"unknown"` when the bundle carried no model swap).
    pub model: String,
    /// blake3 16-hex prefix of the scrubbed message-content bytes. Content-keyed
    /// so identical message bytes produce the same key deterministically.
    pub content_blake3: String,
    /// Tool name the fixture was captured for (J-91 threading; may be the
    /// empty string when the fixture is whole-bundle rather than per-tool).
    pub tool_name: String,
}

/// J-14: the exported fixture cassette artifact. Written to disk as msgpack
/// (the same house serializer the RRA bundle uses, DD1). The cassette carries
/// the scrubbed bundle bytes plus the content-keyed matching key so J-91's
/// `rra re-execute --cassette` can locate it by content, not by path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixtureCassette {
    /// Content-keyed matching key (model + content_blake3 + tool_name).
    pub key: FixtureCassetteKey,
    /// The scrubbed bundle bytes (auth headers stripped via
    /// [`scrub_auth_headers`] before sealing).
    pub bundle_bytes: Vec<u8>,
    /// blake3 16-hex prefix of `bundle_bytes` (the fixture fingerprint).
    pub fixture_blake3: String,
    /// The session id the fixture was exported from.
    pub session_id: String,
    /// Adapter label (J-91 threading; the CLI passes it through verbatim).
    pub adapter: String,
}

/// J-14: result of [`export_fixture`] — the on-disk path + the
/// [`ReproPackPayload`] the CLI renders / an SSB emit helper would land. The
/// payload is returned (not emitted inside rra_export.rs) because this module
/// has no `Ssb` handle and the SSB write path lives in the CLI / orchestrator
/// layer (the task contract forbids touching ssb/* from this rider).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportFixtureResult {
    /// Path the cassette was written to.
    pub fixture_path: std::path::PathBuf,
    /// The cassette artifact (content-keyed matching key + scrubbed bytes).
    pub cassette: FixtureCassette,
    /// The SSB-ready payload (kind = `FixtureExtracted`). The CLI logs its
    /// fields; an SSB emit helper would land it as a `ReproPackEvent` row.
    pub payload: crate::ssb::ReproPackPayload,
}

/// J-14: `rra export-fixture` — read an RRA bundle, scrub auth-header material
/// from the bundle bytes, build a content-keyed cassette keyed by
/// `(model, content_blake3, tool_name)`, write it to a `.fixture` file next to
/// the bundle, and return the on-disk path + the SSB-ready `ReproPackPayload`
/// (kind = `FixtureExtracted`).
///
/// The matching key is CONTENT-KEYED (J-91 extends this; do NOT switch to
/// path-keyed). `tool_name` is the empty string for a whole-bundle fixture
/// (J-91 will thread per-tool fixtures through the same fn signature).
///
/// Scrub is the existing inline-minimal [`scrub_auth_headers`] already shipped
/// in rra_tiers.rs (S3: g9_scrub = inline-minimal). This fn calls it; it does
/// NOT re-author scrub.
///
/// `bundle` is the source `.rra` path. `adapter` is a passthrough label for
/// J-91 (may be the empty string today; kept in the cassette + payload so J-91
/// can extend without regression). `out_dir` overrides the default output
/// directory (the bundle's parent when `None`).
pub fn export_fixture(
    bundle: &std::path::Path,
    adapter: &str,
    tool_name: &str,
    out_dir: Option<&std::path::Path>,
) -> Result<ExportFixtureResult, RraError> {
    export_fixture_with_clock(bundle, adapter, tool_name, out_dir, crate::ssb::system_time_ms_i64())
}

/// J-14: clock-injectable core of [`export_fixture`] so unit tests can pin
/// `ts_ms` deterministically without touching wall-clock state.
pub fn export_fixture_with_clock(
    bundle: &std::path::Path,
    adapter: &str,
    tool_name: &str,
    out_dir: Option<&std::path::Path>,
    now_ms: i64,
) -> Result<ExportFixtureResult, RraError> {
    // 1. Read the bundle (the source `.rra` msgpack bytes).
    let raw_bytes = std::fs::read(bundle).map_err(|e| RraError::Io {
        path: bundle.display().to_string(),
        detail: e.to_string(),
    })?;

    // 2. Decode the RunArtifact to recover model_pin + session_id + the seed
    //    for the keyed scrub (the scrub key is derived from the bundle's seed
    //    so the scrub is deterministic and verifiable, matching build_rra).
    let art: RunArtifact = rmp_serde::from_slice(&raw_bytes)
        .map_err(|e| RraError::Msgpack(e.to_string()))?;

    // 3. Scrub auth-header material from the bundle bytes BEFORE writing the
    //    fixture (S3: g9_scrub = inline-minimal; call the existing fn, do not
    //    re-author). The scrub key is derived from the bundle's seed so the
    //    scrubbed bytes are deterministic under the same seed.
    let scrubbed = scrub_auth_headers(&raw_bytes, art.seed);

    // 4. Build the content-keyed matching key. `content_blake3` is the blake3
    //    16-hex prefix of the SCRUBBED bundle bytes (so the key is stable for
    //    identical message bytes AND insensitive to auth-header churn, which
    //    the scrub already redacts deterministically).
    let content_blake3 = truncate_hex(&blake3::hash(&scrubbed).to_hex());
    let model = art.model_pin.clone().unwrap_or_else(|| "unknown".to_owned());
    let key = FixtureCassetteKey {
        model,
        content_blake3: content_blake3.clone(),
        tool_name: tool_name.to_owned(),
    };

    // 5. The fixture fingerprint is the blake3 of the scrubbed bytes (the
    //    cassette's own content hash, distinct from the bundle's root_blake3
    //    which is over the canonical unscrubbed bundle bytes).
    let fixture_blake3 = content_blake3.clone();

    // 6. Write the cassette to `<out_dir>/<stem>.fixture` (msgpack, DD1). The
    //    default out_dir is the bundle's parent so the fixture lands next to
    //    the source `.rra` by default.
    let out_dir = out_dir
        .map(std::path::Path::to_path_buf)
        .or_else(|| bundle.parent().map(std::path::Path::to_path_buf))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    std::fs::create_dir_all(&out_dir).map_err(|e| RraError::Io {
        path: out_dir.display().to_string(),
        detail: e.to_string(),
    })?;
    let stem = bundle
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("fixture");
    let fixture_path = out_dir.join(format!("{stem}.fixture"));
    let cassette = FixtureCassette {
        key: key.clone(),
        bundle_bytes: scrubbed,
        fixture_blake3: fixture_blake3.clone(),
        session_id: art.session_id.clone(),
        adapter: adapter.to_owned(),
    };
    let cassette_bytes = rmp_serde::to_vec_named(&cassette)
        .map_err(|e| RraError::Msgpack(e.to_string()))?;
    std::fs::write(&fixture_path, &cassette_bytes).map_err(|e| RraError::Io {
        path: fixture_path.display().to_string(),
        detail: e.to_string(),
    })?;

    // 7. Build the SSB-ready ReproPackPayload (kind = FixtureExtracted). The
    //    payload is RETURNED, not emitted inside rra_export.rs (this module
    //    has no Ssb handle; the CLI logs its fields and an SSB emit helper, if
    //    one is wired in a later slice, lands it as a `ReproPackEvent` row).
    let payload = crate::ssb::ReproPackPayload {
        kind: crate::ssb::ReproPackKind::FixtureExtracted,
        tier_effective: None,
        tier_requested: None,
        downgrades: Vec::new(),
        divergence_position: None,
        expected_value: None,
        actual_value: None,
        template_hash: None,
        expected_sequence_hash: None,
        actual_sequence_hash: None,
        fixture_path: Some(fixture_path.display().to_string()),
        fixture_blake3: Some(fixture_blake3),
        session_id: art.session_id.clone(),
        ts_ms: now_ms,
    };

    Ok(ExportFixtureResult {
        fixture_path,
        cassette,
        payload,
    })
}

#[cfg(test)]
mod j14_export_fixture_tests {
    use super::*;
    use crate::determinism::{DeterminismConfig, HardwareClass};
    use crate::rra::{build_rra_with_sampling_seed, write_rra};
    use crate::ssb::{ReproPackKind, SsbActionType, SsbEntry};
    use std::sync::Arc;

    fn test_pins() -> DeterminismConfig {
        DeterminismConfig {
            seed: 42,
            hardware_class: HardwareClass::Cpu,
            blake3_cache_prefix: [0u8; 8],
            system_fingerprint_pin: None,
        }
    }

    fn write_bundle_with_auth_header(tmp: &std::path::Path) -> RunArtifact {
        // Build a bundle whose raw msgpack bytes carry an Authorization header
        // in an event payload so the scrub-before-export has something to strip.
        // The event payload is the literal bytes "Authorization: Bearer secret123\n"
        // so the scrub fn finds and redacts it.
        let auth_payload: Arc<[u8]> = Arc::from(b"Authorization: Bearer secret123\n".as_ref());
        let entry = SsbEntry {
            seq: 1,
            project_id: "j14-export-proj".to_owned(),
            session_id: "j14-export".to_owned(),
            agent_id: "test".to_owned(),
            superstep: 0,
            action_type: SsbActionType::IdeaFramingStarted,
            payload: auth_payload,
            created_at: 0,
        };
        let pins = test_pins();
        let art = build_rra_with_sampling_seed("j14-export", &[entry], &pins, None);
        write_rra(&art, tmp).expect("write bundle");
        art
    }

    // (a) content-key is stable for identical message bytes: two exports of
    //     the same bundle produce the same (model, content_blake3, tool_name).
    #[test]
    fn content_key_stable_for_identical_message_bytes() {
        let tmp = std::env::temp_dir().join(format!(
            "j14-export-stable-{}.rra",
            std::process::id()
        ));
        let out_dir = std::env::temp_dir().join(format!(
            "j14-export-stable-{}",
            std::process::id()
        ));
        let _art = write_bundle_with_auth_header(&tmp);

        let r1 = export_fixture_with_clock(
            &tmp,
            "demo-adapter",
            "",
            Some(&out_dir),
            1700000000_000,
        )
        .expect("export 1");
        let r2 = export_fixture_with_clock(
            &tmp,
            "demo-adapter",
            "",
            Some(&out_dir),
            1700000000_001,
        )
        .expect("export 2");

        // Content-keyed: identical message bytes -> identical key, even though
        // ts_ms differs (the key excludes ts_ms).
        assert_eq!(r1.cassette.key, r2.cassette.key);
        assert_eq!(
            r1.cassette.fixture_blake3,
            r2.cassette.fixture_blake3,
            "fixture fingerprint must be content-keyed, not time-keyed"
        );

        let _ = std::fs::remove_file(&tmp);
        let _ = std::fs::remove_dir_all(&out_dir);
    }

    // (b) scrub strips auth-header material from the fixture bytes: the
    //     cassette's bundle_bytes contain 0 `authorization:` hits and the
    //     redaction marker is present.
    #[test]
    fn scrub_strips_auth_header_from_fixture_bytes() {
        let tmp = std::env::temp_dir().join(format!(
            "j14-export-scrub-{}.rra",
            std::process::id()
        ));
        let out_dir = std::env::temp_dir().join(format!(
            "j14-export-scrub-{}",
            std::process::id()
        ));
        let _art = write_bundle_with_auth_header(&tmp);

        let result = export_fixture_with_clock(
            &tmp,
            "",
            "",
            Some(&out_dir),
            1700000000_000,
        )
        .expect("export");

        let lower = String::from_utf8_lossy(&result.cassette.bundle_bytes)
            .to_ascii_lowercase();
        assert!(
            !lower.contains("authorization:"),
            "scrubbed fixture bytes must not contain authorization: (got {:?})",
            lower
        );
        assert!(
            lower.contains("<redacted:"),
            "scrubbed fixture bytes must contain the redaction marker"
        );

        let _ = std::fs::remove_file(&tmp);
        let _ = std::fs::remove_dir_all(&out_dir);
    }

    // (c) FixtureExtracted kind present on the returned payload.
    #[test]
    fn fixture_extracted_kind_on_returned_payload() {
        let tmp = std::env::temp_dir().join(format!(
            "j14-export-kind-{}.rra",
            std::process::id()
        ));
        let out_dir = std::env::temp_dir().join(format!(
            "j14-export-kind-{}",
            std::process::id()
        ));
        let _art = write_bundle_with_auth_header(&tmp);

        let result = export_fixture_with_clock(
            &tmp,
            "demo",
            "tool-x",
            Some(&out_dir),
            1700000000_000,
        )
        .expect("export");

        assert_eq!(result.payload.kind, ReproPackKind::FixtureExtracted);
        assert_eq!(result.payload.session_id, "j14-export");
        assert!(result.payload.fixture_path.is_some());
        assert!(result.payload.fixture_blake3.is_some());
        assert_eq!(result.cassette.key.tool_name, "tool-x");
        assert_eq!(result.cassette.adapter, "demo");

        let _ = std::fs::remove_file(&tmp);
        let _ = std::fs::remove_dir_all(&out_dir);
    }
}
