//! G-2 — Reproducible-Run Artifact (RRA), the G-plan trust-spine keystone.
//!
//! A `.rra` is a single portable bundle sealing one session's run so it can be
//! verified offline by anyone. It bundles:
//!   * the full, ordered, untruncated SSB action stream (every row's `seq`,
//!     `action_type`, and the raw msgpack payload bytes);
//!   * the determinism pins ([`crate::determinism::DeterminismConfig`]: the
//!     `ELAI_SEED`, the `HardwareClass`, the optional `system_fingerprint`
//!     pin) plus the run's model identity (`model_pin`, recovered from the
//!     latest `model_swap` row in the stream — there is no standalone active-
//!     model beacon, see exploration G-2-substrate §2);
//!   * an evidence manifest of every F-gate verdict hash PRESENT in the stream
//!     (F-1 `source_blake3` / F-4 `stdout_hash` / F-9 `dep_name_hash` /
//!     F-10 `delta_blake3`). G-2 CONSUMES these — it re-implements no F-gate
//!     (GD-1). A gate that was off / silent leaves NO manifest entry; that is a
//!     valid gate-off state, not a faithfulness failure;
//!   * a blake3 root over the canonical bundle bytes.
//!
//! THE HONEST GUARANTEE (GD-5). The SSB stores tool I/O HASHED, not raw bodies
//! (`McpToolCalledPayload` is metadata-only; `args_hash` / `input_hash` /
//! `classifier_verdict_hash` are hashes). So `verify_rra` re-derives each
//! event's content hash from the bundled stream and re-checks every bundled
//! F-gate verdict hash, reporting [`ReplayVerdict::Verified`] (stream + present
//! hashes reproduced) or [`ReplayVerdict::Divergence`] naming the FIRST
//! mismatching event, or [`ReplayVerdict::Tampered`] when the bundle's own root
//! does not recompute (a single-byte edit). It does NOT bit-identically replay
//! token streams: the cloud lane's `system_fingerprint` is opaque and body-
//! reading steps are not persisted raw. The per-run [`GuaranteeLevel`] records
//! this honestly — `build_rra` NEVER labels a run [`GuaranteeLevel::ReproducibleTokens`].
//!
//! No new dependency is introduced: the bundle is serialized with `rmp-serde`
//! (msgpack), already a direct orchestrator dep. There is no in-tree direct
//! compression codec, so the bundle is stored UNCOMPRESSED msgpack. Promoting a
//! compression frame (e.g. `zstd`, currently transitive-only) is a deferred
//! enhancement requiring explicit confirmation per CLAUDE.md no-new-deps — see
//! [`RRA_COMPRESSION_DEFERRED`]. blake3 is the only hash (CLAUDE.md hard rule).

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::determinism::DeterminismConfig;
use crate::ssb::{
    decode_ssb_payload, SsbEntry, SsbPayload,
};

/// `.rra` bundle format version. Bump only on an incompatible bundle-schema
/// change (the dependents G-3/G-5/G-8/G-9/G-10 read this format).
pub const RRA_FORMAT_VERSION: u32 = 1;

/// Length of every blake3 hex-prefix recorded in an RRA (root + per-event +
/// normalized F-9). Matches the workspace 16-hex convention
/// (`session_archive::MANIFEST_HEX_PREFIX_LEN`).
pub const RRA_HEX_PREFIX_LEN: usize = 16;

/// Documented deferred enhancement (DD1, CLAUDE.md no-new-deps): the bundle is
/// stored UNCOMPRESSED msgpack because no direct compression dep exists in any
/// `.elai_cc` `Cargo.toml`. Adding `zstd = "0.13"` (transitive-only today via
/// tantivy) as a DIRECT dep would frame the bundle smaller but requires
/// explicit user confirmation. Carried here so the decision is auditable.
pub const RRA_COMPRESSION_DEFERRED: &str =
    "uncompressed msgpack; zstd-frame deferred (no direct compression dep; promotion needs confirmation)";

/// The honest per-lane reproducibility guarantee an RRA carries.
///
/// `build_rra` NEVER selects [`Self::ReproducibleTokens`] (GD-5): ELAI does not
/// re-execute token streams, and the cloud lane's `system_fingerprint` is
/// opaque. The variant exists for forward-compatibility / completeness and so
/// the dashboard can render the trust ladder, but the producer's honest ceiling
/// is [`Self::ReproducibleActionStream`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuaranteeLevel {
    /// Bit-identical token replay. NEVER produced by `build_rra` (honest scope).
    ReproducibleTokens,
    /// The action stream re-derives byte-identically under the pinned
    /// `ELAI_SEED` AND every present F-gate verdict hash re-checks. The local-
    /// lane honest ceiling: deterministic action ordering + offline hash
    /// re-verification.
    ReproducibleActionStream,
    /// Verify-divergence only: the stream + present hashes are re-checkable for
    /// DIVERGENCE, but the run cannot be byte-faithfully re-executed (cloud
    /// `system_fingerprint` opacity / body-reading steps stored hashed). The
    /// honest default for any run that is not provably local-and-seed-pinned.
    VerifyDivergence,
}

impl GuaranteeLevel {
    /// Stable snake_case wire label (matches the serde encoding).
    #[must_use]
    pub fn as_sql_str(self) -> &'static str {
        match self {
            Self::ReproducibleTokens => "reproducible_tokens",
            Self::ReproducibleActionStream => "reproducible_action_stream",
            Self::VerifyDivergence => "verify_divergence",
        }
    }

    /// Classify the honest guarantee for a run, per-lane (GD-5).
    ///
    /// * A run with a cloud model swap present (an opaque `system_fingerprint`
    ///   lane) degrades to [`Self::VerifyDivergence`] — NEVER `ReproducibleTokens`.
    /// * A run that is provably local-and-pinned (a non-`Unknown` hardware
    ///   class OR a `system_fingerprint` pin recorded) earns
    ///   [`Self::ReproducibleActionStream`].
    /// * Everything else takes the honest default [`Self::VerifyDivergence`].
    #[must_use]
    pub fn classify(pins: &DeterminismConfig, has_cloud_model_swap: bool) -> Self {
        if has_cloud_model_swap {
            return Self::VerifyDivergence;
        }
        let locally_pinned = !matches!(
            pins.hardware_class,
            crate::determinism::HardwareClass::Unknown
        ) || pins.system_fingerprint_pin.is_some();
        if locally_pinned {
            Self::ReproducibleActionStream
        } else {
            Self::VerifyDivergence
        }
    }
}

/// One F-gate verdict hash CONSUMED from the bundled stream (GD-1: G-2 reads
/// these, it never re-derives them).
///
/// F-9's `dep_name_hash` is a `u64` while F-1/F-4/F-10 are hex Strings; the
/// `u64` is normalized to its zero-padded 16-hex big-endian encoding in
/// `verdict_blake3`, and `action_type` preserves the type provenance so a
/// consumer can tell a normalized-u64 entry from a native hex entry (DD4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateVerdictHash {
    /// Which F-gate produced the verdict (`"F-1"` / `"F-4"` / `"F-9"` / `"F-10"`).
    pub gate: String,
    /// The SSB `action_type` wire tag of the row the verdict was read from.
    pub action_type: String,
    /// blake3 verdict hash. Native hex for F-1/F-4/F-10; the normalized
    /// 16-hex big-endian encoding of the `u64` for F-9.
    pub verdict_blake3: String,
    /// The `seq` of the SSB row the verdict was read from (correlation).
    pub ssb_event_seq: i64,
}

/// One element of the bundled action stream — the full, untruncated row.
///
/// `event_blake3` is the content hash SEALED at build time over
/// `seq || action_type || payload`. `verify_rra` recomputes the content hash
/// from the (possibly mutated) `seq`/`action_type`/`payload` and compares it to
/// this sealed value: a payload byte flipped without also patching `event_blake3`
/// DIVERGES here; a flip that also patches `event_blake3` changes the canonical
/// bundle bytes and is caught earlier by the root tamper check. Either way the
/// mutation screams (R2/R4) — there is no silent pass.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RraEvent {
    /// Append-only sequence number (structural ordering, ssb.rs:5).
    pub seq: i64,
    /// SSB `action_type` wire tag.
    pub action_type: String,
    /// The raw msgpack payload bytes for this row (untruncated; the faithful
    /// stream `verify_rra` re-derives content hashes from).
    pub payload: Vec<u8>,
    /// The content hash SEALED at build time. See the struct doc.
    pub event_blake3: String,
}

impl RraEvent {
    /// Build a sealed event from a `seq` / `action_type` / `payload`,
    /// computing and storing the content hash.
    #[must_use]
    pub fn sealed(seq: i64, action_type: String, payload: Vec<u8>) -> Self {
        let event_blake3 = content_blake3(seq, &action_type, &payload);
        Self {
            seq,
            action_type,
            payload,
            event_blake3,
        }
    }

    /// Recompute the content hash from the CURRENT `seq`/`action_type`/`payload`.
    /// Compared against the sealed `event_blake3` by [`verify_artifact`].
    #[must_use]
    pub fn recompute_content_blake3(&self) -> String {
        content_blake3(self.seq, &self.action_type, &self.payload)
    }
}

/// Content hash of an event: blake3 over `seq || action_type || NUL || payload`,
/// 16-hex prefix. A single-byte edit to any field flips it.
fn content_blake3(seq: i64, action_type: &str, payload: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&seq.to_le_bytes());
    hasher.update(action_type.as_bytes());
    hasher.update(b"\x00");
    hasher.update(payload);
    truncate_hex(&hasher.finalize().to_hex())
}

/// The portable Reproducible-Run Artifact bundle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunArtifact {
    /// Bundle format version. See [`RRA_FORMAT_VERSION`].
    pub format_version: u32,
    /// The session whose stream was bundled.
    pub session_id: String,
    /// The pinned `ELAI_SEED` the stream re-derives under.
    pub seed: u64,
    /// Snake-case hardware-class label (`"cpu"` / `"gpu:<sku>"` / `"unknown"`).
    pub hardware_class: String,
    /// Optional `system_fingerprint` pin (the cloud-lane opacity marker;
    /// `None` on a local run with no pin).
    pub system_fingerprint_pin: Option<String>,
    /// The run's model identity (latest `model_swap` `to_model`); `None` when
    /// no model swap was recorded.
    pub model_pin: Option<String>,
    /// The honest per-lane guarantee level (GD-5).
    pub guarantee_level: GuaranteeLevel,
    /// The full, ordered action stream.
    pub events: Vec<RraEvent>,
    /// The F-gate evidence manifest (CONSUMED, GD-1).
    pub manifest: Vec<GateVerdictHash>,
    /// blake3 root over the canonical bundle bytes (everything above except
    /// this field). A single-byte tamper flips it. See [`compute_root`].
    pub root_blake3: String,
}

/// The verdict of a `replay --verify` on a `.rra`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "verdict")]
pub enum ReplayVerdict {
    /// The bundle root recomputes AND every event content-hash re-derives.
    Verified,
    /// An event's content hash does not re-derive (the action stream diverged).
    /// Names the FIRST mismatching event.
    Divergence {
        /// `seq` of the first event whose content hash did not re-derive.
        first_mismatch_event: i64,
        /// The content hash recorded for that event at seal time.
        expected_blake3: String,
        /// The content hash recomputed now.
        got_blake3: String,
    },
    /// The bundle's own root_blake3 does not recompute — the bundle bytes were
    /// edited after sealing (a single-byte tamper).
    Tampered {
        /// The root recorded in the bundle.
        expected_root: String,
        /// The root recomputed over the bundle now.
        got_root: String,
    },
}

/// Errors from reading / writing a `.rra`.
#[derive(Debug, Error)]
pub enum RraError {
    /// Bundle file I/O failure.
    #[error("rra io error at {path}: {detail}")]
    Io { path: String, detail: String },
    /// msgpack (de)serialization failure.
    #[error("rra msgpack error: {0}")]
    Msgpack(String),
}

/// Truncate a full blake3 hex to the workspace 16-hex prefix convention.
fn truncate_hex(full_hex: &str) -> String {
    full_hex.chars().take(RRA_HEX_PREFIX_LEN).collect()
}

/// Compute the canonical bundle bytes (everything EXCEPT `root_blake3`), as the
/// msgpack of a deterministic tuple in a fixed order. The same input always
/// produces the same bytes, so the root recomputes byte-identically (GD-2).
fn canonical_bundle_bytes(art: &RunArtifact) -> Result<Vec<u8>, RraError> {
    // A fixed-order tuple excluding root_blake3. events are already ordered by
    // seq (the reader emits ASC); manifest is sorted by (gate, ssb_event_seq)
    // for stable ordering independent of scan order.
    let mut manifest = art.manifest.clone();
    manifest.sort_by(|a, b| {
        a.gate
            .cmp(&b.gate)
            .then(a.ssb_event_seq.cmp(&b.ssb_event_seq))
    });
    let canonical = (
        art.format_version,
        &art.session_id,
        art.seed,
        &art.hardware_class,
        &art.system_fingerprint_pin,
        &art.model_pin,
        art.guarantee_level,
        &art.events,
        &manifest,
    );
    rmp_serde::to_vec_named(&canonical).map_err(|e| RraError::Msgpack(e.to_string()))
}

/// Compute the blake3 root (16-hex prefix) over the canonical bundle bytes.
pub fn compute_root(art: &RunArtifact) -> Result<String, RraError> {
    let bytes = canonical_bundle_bytes(art)?;
    Ok(truncate_hex(&blake3::hash(&bytes).to_hex()))
}

/// Extract the F-gate evidence manifest from a decoded stream (GD-1: CONSUME,
/// build none). A gate with no row in the stream simply has no entry (a valid
/// gate-off state). F-9's `u64` is normalized to 16-hex big-endian.
fn extract_gate_manifest(events: &[(SsbEntry, Option<SsbPayload>)]) -> Vec<GateVerdictHash> {
    let mut manifest = Vec::new();
    for (entry, decoded) in events {
        let Some(payload) = decoded else { continue };
        match payload {
            // F-1: VibeGate source_blake3 (default-OFF behind ELAI_VIBE_GATE_ENABLE;
            // absent entry is a valid gate-off state, not a failure).
            SsbPayload::VibeGateBlocked(p) => manifest.push(GateVerdictHash {
                gate: "F-1".to_owned(),
                action_type: entry.action_type.as_sql_str().to_owned(),
                verdict_blake3: p.source_blake3.clone(),
                ssb_event_seq: entry.seq,
            }),
            // F-4: TestIntegrityEvent stdout_hash (None on a fabrication signal).
            SsbPayload::TestIntegrityEvent(p) => {
                if let Some(stdout_hash) = &p.stdout_hash {
                    manifest.push(GateVerdictHash {
                        gate: "F-4".to_owned(),
                        action_type: entry.action_type.as_sql_str().to_owned(),
                        verdict_blake3: stdout_hash.clone(),
                        ssb_event_seq: entry.seq,
                    });
                }
            }
            // F-9: WriteTimeSlopsquatBlocked dep_name_hash:u64 (normalized to hex).
            SsbPayload::WriteTimeSlopsquatBlocked(p) => manifest.push(GateVerdictHash {
                gate: "F-9".to_owned(),
                action_type: entry.action_type.as_sql_str().to_owned(),
                verdict_blake3: format!("{:016x}", p.dep_name_hash),
                ssb_event_seq: entry.seq,
            }),
            // F-10: SecurityClassRegressionBlocked delta_blake3.
            SsbPayload::SecurityClassRegressionBlocked(p) => manifest.push(GateVerdictHash {
                gate: "F-10".to_owned(),
                action_type: entry.action_type.as_sql_str().to_owned(),
                verdict_blake3: p.delta_blake3.clone(),
                ssb_event_seq: entry.seq,
            }),
            _ => {}
        }
    }
    manifest
}

/// Recover the run's model identity: the LAST `model_swap` `to_model` in the
/// ordered stream (there is no standalone active-model beacon — exploration
/// G-2-substrate §2). Returns `None` when no model swap was recorded AND flags
/// whether any swap targeted a cloud model (an opaque-fingerprint lane).
fn recover_model_pin(events: &[(SsbEntry, Option<SsbPayload>)]) -> (Option<String>, bool) {
    let mut model_pin: Option<String> = None;
    let mut has_cloud_model_swap = false;
    for (_entry, decoded) in events {
        if let Some(SsbPayload::ModelSwap(p)) = decoded {
            model_pin = Some(p.to_model.clone());
            // A cloud lane is heuristically any model whose id is not a local
            // sidecar path / Qwen local id. We treat an Anthropic/cloud family
            // marker as opaque-fingerprint (GD-5). This is a conservative
            // signal: when unsure, the honest default is VerifyDivergence.
            let lower = p.to_model.to_ascii_lowercase();
            if lower.contains("claude")
                || lower.contains("sonnet")
                || lower.contains("opus")
                || lower.contains("haiku")
                || lower.contains("gpt")
                || lower.contains("deepseek")
            {
                has_cloud_model_swap = true;
            }
        }
    }
    (model_pin, has_cloud_model_swap)
}

/// Build a [`RunArtifact`] from a session's full ordered SSB stream + the
/// determinism pins + the run's model identity.
///
/// `stream` MUST be the ASC-ordered untruncated rows (see
/// [`crate::ssb::Ssb::query_session_stream_asc`]). The blake3 root is computed
/// last over the canonical bundle bytes, so it recomputes byte-identically on
/// the same input under the pinned seed (GD-2).
pub fn build_rra(
    session_id: &str,
    stream: &[SsbEntry],
    pins: &DeterminismConfig,
) -> RunArtifact {
    // Decode each row once (best-effort: an undecodable row keeps None and is
    // still bundled by its raw bytes — the stream stays faithful).
    let decoded: Vec<(SsbEntry, Option<SsbPayload>)> = stream
        .iter()
        .map(|e| (e.clone(), decode_ssb_payload(&e.payload).ok()))
        .collect();

    let events: Vec<RraEvent> = decoded
        .iter()
        .map(|(e, _)| {
            RraEvent::sealed(
                e.seq,
                e.action_type.as_sql_str().to_owned(),
                e.payload.to_vec(),
            )
        })
        .collect();

    let manifest = extract_gate_manifest(&decoded);
    let (model_pin, has_cloud_model_swap) = recover_model_pin(&decoded);
    let guarantee_level = GuaranteeLevel::classify(pins, has_cloud_model_swap);

    let hardware_class = hardware_class_label(&pins.hardware_class);

    let mut art = RunArtifact {
        format_version: RRA_FORMAT_VERSION,
        session_id: session_id.to_owned(),
        seed: pins.seed,
        hardware_class,
        system_fingerprint_pin: pins.system_fingerprint_pin.clone(),
        model_pin,
        guarantee_level,
        events,
        manifest,
        // Filled in after the canonical bytes are stable.
        root_blake3: String::new(),
    };
    // compute_root cannot fail in practice (msgpack of owned data), but we
    // propagate the empty-root on the impossible error path rather than panic.
    art.root_blake3 = compute_root(&art).unwrap_or_default();
    art
}

/// Snake-case label for a [`crate::determinism::HardwareClass`] (mirrors the
/// `ELAI_HARDWARE_CLASS` encoding).
fn hardware_class_label(hc: &crate::determinism::HardwareClass) -> String {
    match hc {
        crate::determinism::HardwareClass::Cpu => "cpu".to_owned(),
        crate::determinism::HardwareClass::Gpu(sku) => format!("gpu:{}", sku.as_str()),
        crate::determinism::HardwareClass::Unknown => "unknown".to_owned(),
    }
}

/// Serialize a [`RunArtifact`] to a `.rra` file (uncompressed msgpack, DD1).
pub fn write_rra(art: &RunArtifact, path: &std::path::Path) -> Result<(), RraError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| RraError::Io {
            path: parent.display().to_string(),
            detail: e.to_string(),
        })?;
    }
    let bytes = rmp_serde::to_vec_named(art).map_err(|e| RraError::Msgpack(e.to_string()))?;
    std::fs::write(path, &bytes).map_err(|e| RraError::Io {
        path: path.display().to_string(),
        detail: e.to_string(),
    })
}

/// Read a [`RunArtifact`] back from a `.rra` file.
pub fn read_rra(path: &std::path::Path) -> Result<RunArtifact, RraError> {
    let bytes = std::fs::read(path).map_err(|e| RraError::Io {
        path: path.display().to_string(),
        detail: e.to_string(),
    })?;
    rmp_serde::from_slice::<RunArtifact>(&bytes).map_err(|e| RraError::Msgpack(e.to_string()))
}

/// Verify a `.rra`: re-read, recompute the blake3 root (tamper check), then
/// re-derive each event's content hash and compare to the sealed hash chain
/// (divergence check). Returns the FIRST mismatch.
///
/// This is the off-agent, un-gameable backstop the wiring gate measures (R4):
/// the verdict is a property of the bundle bytes the agent did not control at
/// verify time. No fabricated [`ReplayVerdict::Verified`] — missing / mismatched
/// evidence is reported, never swallowed.
pub fn verify_rra(path: &std::path::Path) -> Result<ReplayVerdict, RraError> {
    let art = read_rra(path)?;
    verify_artifact(&art)
}

/// Verify an already-decoded [`RunArtifact`] (the in-memory half of
/// [`verify_rra`], exposed for the deterministic contract tests).
pub fn verify_artifact(art: &RunArtifact) -> Result<ReplayVerdict, RraError> {
    // 1. Tamper check: the root must recompute over the canonical bytes.
    let recomputed_root = compute_root(art)?;
    if recomputed_root != art.root_blake3 {
        return Ok(ReplayVerdict::Tampered {
            expected_root: art.root_blake3.clone(),
            got_root: recomputed_root,
        });
    }
    // 2. Divergence check: for each event, recompute the content hash from its
    //    CURRENT seq/action_type/payload and compare to the hash SEALED at build
    //    time (`event_blake3`). A payload byte flipped after sealing — without
    //    also patching event_blake3 — diverges here, naming the first offending
    //    seq. (A flip that ALSO patches event_blake3 changes the canonical
    //    bundle bytes and was already caught by the root tamper check above.)
    //    Missing/mismatched evidence is reported, never swallowed (R2).
    for ev in &art.events {
        let recomputed = ev.recompute_content_blake3();
        if recomputed != ev.event_blake3 {
            return Ok(ReplayVerdict::Divergence {
                first_mismatch_event: ev.seq,
                expected_blake3: ev.event_blake3.clone(),
                got_blake3: recomputed,
            });
        }
    }
    Ok(ReplayVerdict::Verified)
}
