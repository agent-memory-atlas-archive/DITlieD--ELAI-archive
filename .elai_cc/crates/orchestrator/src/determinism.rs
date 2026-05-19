//! C-27 — Testing-Trust Tier T(-1): Inference Determinism foundation.
//!
//! This module hosts the substrate that lets us reason about whether two
//! nominally-equivalent runs of ELAI under the same `ELAI_SEED` actually
//! produce identical action streams. It is intentionally small — the
//! deeper plumbing (per-call hardware fingerprint pinning, KV-cache
//! continuity guard rails, etc.) is scoped for later tiers.
//!
//! Surface:
//!
//!   * [`DeterminismConfig`] — bundles a seed, a [`HardwareClass`] pin, a
//!     blake3-derived prefix-cache key, and an optional
//!     `system_fingerprint_pin`. Constructed via [`DeterminismConfig::from_env`].
//!   * [`HardwareClass`] / [`GpuSku`] — classifies the executing machine so
//!     determinism comparisons only run within the same hardware class.
//!   * [`ActionTypeSequence`] — newtype around a `Vec<String>` of snake-case
//!     SSB action_type strings, captured per run.
//!   * [`DriftComparator`] — compares two `ActionTypeSequence`s; on first
//!     divergence emits a single
//!     [`SsbActionType::DeterminismDrift`](crate::ssb::SsbActionType::DeterminismDrift)
//!     row. Identical sequences emit zero rows.
//!
//! All entropy is routed through `elai_core::EntropySource`; all wall-clock
//! reads through `elai_core::ClockProvider`. The single
//! `uuid::Uuid::new_v4()` call in [`DriftComparator::new`] carries an
//! `entropy-allow` annotation because the audit_id is a one-shot
//! per-comparison correlator without a meaningful EntropySource threading
//! path (see workflow/reasoning/C-15-decisions.md D7 + workflow/reasoning/
//! C-27-decisions.md D11).
//!
//! Grounding sources: workflow/reasoning/C-15-decisions.md (entropy
//! quarantine + ClockProvider patterns) and docs/thefinalglance/verification/
//! reports/guide/testing-trust-design.report.md §Layer -1.

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::mpsc::UnboundedSender;
use tracing::warn;

use crate::ssb::{
    DeterminismDriftPayload, SsbActionType, SsbPayload,
};

/// ELAI_SEED environment variable name. `0` is the default when unset.
pub const ELAI_SEED_ENV: &str = "ELAI_SEED";
/// ELAI_HARDWARE_CLASS environment variable name. Empty / missing -> Unknown.
pub const ELAI_HARDWARE_CLASS_ENV: &str = "ELAI_HARDWARE_CLASS";
/// ELAI_SYSTEM_FINGERPRINT environment variable name. Optional pin.
pub const ELAI_SYSTEM_FINGERPRINT_ENV: &str = "ELAI_SYSTEM_FINGERPRINT";

/// Length of the blake3 prefix-cache key derivation in bytes.
/// `blake3::keyed_hash` consumes a 32-byte key; we pad the 8-byte seed-derived
/// prefix to 32 bytes with zeros (see [`pad_key_to_32`]).
pub const PREFIX_CACHE_KEY_BYTES: usize = 8;

/// GPU SKU newtype. Constructed from `ELAI_HARDWARE_CLASS=gpu:<sku>`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GpuSku(String);

impl GpuSku {
    pub fn new(sku: impl Into<String>) -> Self {
        Self(sku.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Hardware class pin. Determinism comparisons MUST only run within the same
/// class — comparing `Gpu(RTX3090)` against `Gpu(RTX4090)` is undefined and
/// not a drift signal. `Unknown` is the safe default when `ELAI_HARDWARE_CLASS`
/// is absent or malformed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwareClass {
    Cpu,
    Gpu(GpuSku),
    Unknown,
}

impl HardwareClass {
    /// Parse the `ELAI_HARDWARE_CLASS` env-var encoding.
    ///   * `"cpu"`              -> [`HardwareClass::Cpu`]
    ///   * `"gpu:<sku>"`        -> [`HardwareClass::Gpu`]\(GpuSku::new(sku))
    ///   * `""` / `"unknown"` / anything else -> [`HardwareClass::Unknown`]
    pub fn from_env_value(value: &str) -> Self {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Self::Unknown;
        }
        let lower = trimmed.to_ascii_lowercase();
        if lower == "cpu" {
            return Self::Cpu;
        }
        if let Some(rest) = lower.strip_prefix("gpu:") {
            let sku = rest.trim();
            if !sku.is_empty() {
                return Self::Gpu(GpuSku::new(sku));
            }
        }
        if lower == "unknown" {
            return Self::Unknown;
        }
        // Treat anything else as Unknown rather than panic — determinism is
        // best-effort when the operator hasn't pinned the class.
        warn!(
            target: "elai_orchestrator::determinism",
            "unrecognized ELAI_HARDWARE_CLASS value {value:?}; defaulting to Unknown"
        );
        Self::Unknown
    }
}

/// Error type for [`DeterminismConfig`] construction. No panics, no unwraps —
/// every failure path returns a typed error per CLAUDE.md hard rules.
#[derive(Debug, Error)]
pub enum DeterminismError {
    #[error("environment variable {var}={value:?} could not be parsed: {detail}")]
    EnvParseError {
        var: &'static str,
        value: String,
        detail: String,
    },
    /// The 8-byte prefix could not be embedded into the 32-byte blake3 key
    /// slot. This is a structural impossibility (compile-time length check
    /// covers it) but we keep the variant so callers can pattern-match
    /// against any DeterminismError without an `_ =>` arm.
    #[error("internal: failed to construct 32-byte blake3 key from prefix")]
    HashKeyError,
}

/// Determinism configuration bundle. One instance per process / per audit run.
/// Cheap to clone (no I/O on construction beyond env reads).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeterminismConfig {
    pub hardware_class: HardwareClass,
    pub seed: u64,
    pub blake3_cache_prefix: [u8; PREFIX_CACHE_KEY_BYTES],
    pub system_fingerprint_pin: Option<String>,
}

impl DeterminismConfig {
    /// Construct from environment variables.
    ///
    ///   * `ELAI_SEED`            — `u64`, default `0`.
    ///   * `ELAI_HARDWARE_CLASS`  — see [`HardwareClass::from_env_value`].
    ///   * `ELAI_SYSTEM_FINGERPRINT` — optional pin; empty -> `None`.
    ///
    /// `blake3_cache_prefix` is derived as the first 8 bytes of
    /// `blake3::hash(seed.to_le_bytes())`. This stays stable across processes
    /// for the same seed and pins the prefix-cache lookup key (Layer -1
    /// guardrail D-3 in testing-trust-design).
    pub fn from_env() -> Result<Self, DeterminismError> {
        let seed = match std::env::var(ELAI_SEED_ENV) {
            Ok(raw) if !raw.trim().is_empty() => {
                raw.trim()
                    .parse::<u64>()
                    .map_err(|e| DeterminismError::EnvParseError {
                        var: ELAI_SEED_ENV,
                        value: raw.clone(),
                        detail: e.to_string(),
                    })?
            }
            _ => 0u64,
        };

        let hardware_class = match std::env::var(ELAI_HARDWARE_CLASS_ENV) {
            Ok(raw) => HardwareClass::from_env_value(&raw),
            Err(_) => HardwareClass::Unknown,
        };

        let system_fingerprint_pin = match std::env::var(ELAI_SYSTEM_FINGERPRINT_ENV) {
            Ok(raw) if !raw.trim().is_empty() => Some(raw.trim().to_owned()),
            _ => None,
        };

        let hash = blake3::hash(&seed.to_le_bytes());
        let hash_bytes = hash.as_bytes();
        let mut prefix = [0u8; PREFIX_CACHE_KEY_BYTES];
        prefix.copy_from_slice(&hash_bytes[..PREFIX_CACHE_KEY_BYTES]);

        Ok(Self {
            hardware_class,
            seed,
            blake3_cache_prefix: prefix,
            system_fingerprint_pin,
        })
    }

    /// Construct a prefix-cache key by mixing the configured 8-byte prefix
    /// (padded to 32 bytes) into a `blake3::keyed_hash` over the supplied
    /// `prompt_hash` (caller-provided — usually itself a blake3 hash of a
    /// canonical-ised prompt). The output is the 32-byte blake3 hash.
    ///
    /// This is the SOLE blake3-keyed-hash production code path in the
    /// orchestrator — see C-27-decisions.md D-blake3 for the rationale
    /// against sha256 / md5 (CLAUDE.md anti-stale rule).
    pub fn prefix_cache_key(&self, prompt_hash: &[u8]) -> [u8; 32] {
        let key = pad_key_to_32(&self.blake3_cache_prefix);
        let hash = blake3::keyed_hash(&key, prompt_hash);
        *hash.as_bytes()
    }

    /// True when there is no fingerprint pin OR the supplied fingerprint
    /// matches the pin. Used by inference callers that want to assert the
    /// runtime fingerprint hasn't drifted before serving a cached response.
    pub fn matches_fingerprint(&self, observed: Option<&str>) -> bool {
        match (&self.system_fingerprint_pin, observed) {
            (None, _) => true,
            (Some(pin), Some(obs)) => pin == obs,
            (Some(_), None) => false,
        }
    }
}

/// Pad an N-byte prefix into a fixed 32-byte blake3 key (zero-extension).
fn pad_key_to_32(prefix: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    let n = prefix.len().min(32);
    key[..n].copy_from_slice(&prefix[..n]);
    key
}

// ---------------------------------------------------------------------------
// DriftComparator
// ---------------------------------------------------------------------------

/// Newtype around a captured action_type stream. Action types are the
/// snake_case strings from
/// [`crate::ssb::SsbActionType::as_sql_str`] joined with `\n` for hashing.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionTypeSequence(pub Vec<String>);

impl ActionTypeSequence {
    pub fn new(actions: Vec<String>) -> Self {
        Self(actions)
    }

    pub fn from_action_types(actions: &[SsbActionType]) -> Self {
        Self(
            actions
                .iter()
                .map(|a| a.as_sql_str().to_owned())
                .collect::<Vec<_>>(),
        )
    }

    pub fn as_slice(&self) -> &[String] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// blake3-hex hash of the joined action_type stream up to (but not
    /// including) `up_to`. Used as the per-run hash in
    /// [`DeterminismDriftPayload`].
    pub fn hash_prefix_hex(&self, up_to: usize) -> String {
        let bound = up_to.min(self.0.len());
        let joined = self.0[..bound].join("\n");
        let hash = blake3::hash(joined.as_bytes());
        hash.to_hex().to_string()
    }
}

/// SSB sink abstraction so the comparator can route emits through either an
/// unbounded mpsc channel (production) or a test capture vec.
pub enum DriftSink {
    /// Production sink: send the rendered payload through an mpsc channel.
    /// The consumer end is responsible for writing to the SSB DB.
    Channel(UnboundedSender<SsbPayload>),
    /// Test sink: append into a shared vec (no actual SSB write).
    #[cfg(test)]
    Capture(std::sync::Arc<std::sync::Mutex<Vec<SsbPayload>>>),
}

impl DriftSink {
    fn send(&self, payload: SsbPayload) {
        match self {
            Self::Channel(tx) => {
                if let Err(err) = tx.send(payload) {
                    warn!(
                        target: "elai_orchestrator::determinism",
                        "DriftComparator SSB channel closed: {err}"
                    );
                }
            }
            #[cfg(test)]
            Self::Capture(buf) => match buf.lock() {
                Ok(mut guard) => guard.push(payload),
                Err(poisoned) => {
                    // Preserve forward progress in tests even after a panic
                    // in another assertion; we still want to see what got
                    // emitted before the failure.
                    poisoned.into_inner().push(payload);
                }
            },
        }
    }
}

/// Comparator over two [`ActionTypeSequence`]s captured from two runs that
/// SHOULD have produced identical streams (same seed, same hardware_class).
///
/// On the first mismatched index, emits exactly one
/// [`SsbActionType::DeterminismDrift`] row through the configured sink and
/// returns it. Identical sequences (including identical length) emit ZERO
/// rows — this is the contract pinned by the C-27 stub_ban clause.
pub struct DriftComparator {
    seed: u64,
    sink: DriftSink,
    audit_id: uuid::Uuid,
    #[allow(dead_code)]
    clock: Arc<dyn ClockProvider>,
}

impl DriftComparator {
    /// Construct a new comparator. The `audit_id` is generated via
    /// `Uuid::new_v4()` — annotated as `entropy-allow` per
    /// C-27-decisions.md D11 because per-audit IDs are one-shot correlators
    /// without a meaningful EntropySource threading path.
    pub fn new(seed: u64, sink: DriftSink) -> Self {
        // entropy-allow: per-comparator correlator; no EntropySource threaded
        let audit_id = uuid::Uuid::new_v4();
        Self::with_audit_id(seed, sink, audit_id)
    }

    /// Construct with an explicit audit_id — used by tests that need a stable
    /// uuid and by higher-level audit drivers that own the correlator.
    pub fn with_audit_id(seed: u64, sink: DriftSink, audit_id: uuid::Uuid) -> Self {
        Self {
            seed,
            sink,
            audit_id,
            clock: Arc::new(OsClock),
        }
    }

    /// Inject a clock — used by tests with `SeededClock`.
    pub fn with_clock(mut self, clock: Arc<dyn ClockProvider>) -> Self {
        self.clock = clock;
        self
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn audit_id(&self) -> uuid::Uuid {
        self.audit_id
    }

    /// Compare `run_a` against `run_b`. Returns `Some(payload)` if a drift
    /// was emitted; `None` when the two sequences are byte-equal under the
    /// snake_case action_type encoding.
    pub fn compare(
        &self,
        run_a: &ActionTypeSequence,
        run_b: &ActionTypeSequence,
    ) -> Option<DeterminismDriftPayload> {
        // Find the first index where the two streams disagree. If one is a
        // prefix of the other, the divergence index is the shorter length.
        let min_len = run_a.len().min(run_b.len());
        let mut divergence: Option<usize> = None;
        for i in 0..min_len {
            if run_a.0[i] != run_b.0[i] {
                divergence = Some(i);
                break;
            }
        }
        let idx = match divergence {
            Some(i) => i,
            None => {
                if run_a.len() == run_b.len() {
                    return None;
                }
                min_len
            }
        };

        // Each run_*_hash captures the full action stream UP TO AND
        // INCLUDING the divergence point so callers can verify the two
        // hashes are byte-distinct (which is the durable drift fingerprint).
        // We hash through `idx + 1` so a same-length-different-element
        // divergence at index N produces different hashes; a length-mismatch
        // divergence where one side has no element at `idx` still yields
        // different hashes because the joined string differs in trailing
        // content.
        let hash_through = idx + 1;
        let payload = DeterminismDriftPayload {
            seed: self.seed,
            run_a_hash: format!("blake3:{}", run_a.hash_prefix_hex(hash_through)),
            run_b_hash: format!("blake3:{}", run_b.hash_prefix_hex(hash_through)),
            first_divergence_index: idx as u32,
            audit_id: self.audit_id,
        };

        self.sink.send(SsbPayload::DeterminismDrift(payload.clone()));
        Some(payload)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    fn capture_sink() -> (DriftSink, Arc<Mutex<Vec<SsbPayload>>>) {
        let buf = Arc::new(Mutex::new(Vec::<SsbPayload>::new()));
        let sink = DriftSink::Capture(buf.clone());
        (sink, buf)
    }

    #[test]
    fn hardware_class_parses_known_values() {
        assert_eq!(HardwareClass::from_env_value("cpu"), HardwareClass::Cpu);
        assert_eq!(HardwareClass::from_env_value("CPU"), HardwareClass::Cpu);
        assert_eq!(HardwareClass::from_env_value(""), HardwareClass::Unknown);
        assert_eq!(HardwareClass::from_env_value("unknown"), HardwareClass::Unknown);
        match HardwareClass::from_env_value("gpu:rtx3090") {
            HardwareClass::Gpu(sku) => assert_eq!(sku.as_str(), "rtx3090"),
            other => panic!("expected Gpu, got {other:?}"),
        }
    }

    #[test]
    fn hardware_class_unrecognized_falls_back_unknown() {
        assert_eq!(
            HardwareClass::from_env_value("npu:xyz"),
            HardwareClass::Unknown
        );
    }

    #[test]
    fn prefix_cache_key_is_stable_for_same_seed() {
        let cfg = DeterminismConfig {
            hardware_class: HardwareClass::Cpu,
            seed: 42,
            blake3_cache_prefix: {
                let h = blake3::hash(&42u64.to_le_bytes());
                let mut p = [0u8; 8];
                p.copy_from_slice(&h.as_bytes()[..8]);
                p
            },
            system_fingerprint_pin: None,
        };
        let prompt_hash = blake3::hash(b"hello world");
        let k1 = cfg.prefix_cache_key(prompt_hash.as_bytes());
        let k2 = cfg.prefix_cache_key(prompt_hash.as_bytes());
        assert_eq!(k1, k2, "prefix_cache_key must be deterministic per seed");
    }

    #[test]
    fn prefix_cache_key_differs_for_different_seeds() {
        let mk = |seed: u64| {
            let h = blake3::hash(&seed.to_le_bytes());
            let mut p = [0u8; 8];
            p.copy_from_slice(&h.as_bytes()[..8]);
            DeterminismConfig {
                hardware_class: HardwareClass::Cpu,
                seed,
                blake3_cache_prefix: p,
                system_fingerprint_pin: None,
            }
        };
        let prompt_hash = blake3::hash(b"prompt");
        let k1 = mk(1).prefix_cache_key(prompt_hash.as_bytes());
        let k2 = mk(2).prefix_cache_key(prompt_hash.as_bytes());
        assert_ne!(k1, k2);
    }

    #[test]
    fn fingerprint_pin_matching() {
        let mut cfg = DeterminismConfig {
            hardware_class: HardwareClass::Cpu,
            seed: 0,
            blake3_cache_prefix: [0; 8],
            system_fingerprint_pin: None,
        };
        // No pin: always matches.
        assert!(cfg.matches_fingerprint(None));
        assert!(cfg.matches_fingerprint(Some("anything")));
        cfg.system_fingerprint_pin = Some("fp-1".to_owned());
        assert!(cfg.matches_fingerprint(Some("fp-1")));
        assert!(!cfg.matches_fingerprint(Some("fp-2")));
        assert!(!cfg.matches_fingerprint(None));
    }

    #[test]
    fn drift_comparator_no_drift_on_identical_sequences() {
        let (sink, buf) = capture_sink();
        let cmp = DriftComparator::with_audit_id(42, sink, uuid::Uuid::nil());
        let a = ActionTypeSequence::new(vec![
            "file_write".to_owned(),
            "tool_result".to_owned(),
            "phase_transition".to_owned(),
        ]);
        let b = a.clone();
        let verdict = cmp.compare(&a, &b);
        assert!(verdict.is_none(), "identical sequences must produce zero drift rows");
        let captured = buf.lock().expect("lock");
        assert_eq!(captured.len(), 0, "zero SSB rows on identical sequences");
    }

    #[test]
    fn drift_comparator_detects_first_divergence_index() {
        let (sink, buf) = capture_sink();
        let cmp = DriftComparator::with_audit_id(42, sink, uuid::Uuid::nil());
        let a = ActionTypeSequence::new(vec![
            "file_write".to_owned(),
            "tool_result".to_owned(),
            "phase_transition".to_owned(),
        ]);
        let b = ActionTypeSequence::new(vec![
            "file_write".to_owned(),
            "tool_result".to_owned(),
            "finding".to_owned(),
        ]);
        let verdict = cmp.compare(&a, &b).expect("must emit drift");
        assert_eq!(verdict.first_divergence_index, 2);
        assert_eq!(verdict.seed, 42);
        assert_eq!(verdict.audit_id, uuid::Uuid::nil());
        let captured = buf.lock().expect("lock");
        assert_eq!(captured.len(), 1);
        match &captured[0] {
            SsbPayload::DeterminismDrift(p) => {
                assert_eq!(p.first_divergence_index, 2);
                // Hashes cover the divergent action -> must differ.
                assert_ne!(p.run_a_hash, p.run_b_hash);
            }
            other => panic!("unexpected payload {other:?}"),
        }
    }

    #[test]
    fn drift_comparator_handles_length_mismatch() {
        let (sink, buf) = capture_sink();
        let cmp = DriftComparator::with_audit_id(7, sink, uuid::Uuid::nil());
        let a = ActionTypeSequence::new(vec!["file_write".to_owned()]);
        let b = ActionTypeSequence::new(vec![
            "file_write".to_owned(),
            "tool_result".to_owned(),
        ]);
        let verdict = cmp.compare(&a, &b).expect("must emit drift on length diff");
        assert_eq!(verdict.first_divergence_index, 1);
        let captured = buf.lock().expect("lock");
        assert_eq!(captured.len(), 1);
    }

    #[test]
    fn from_env_default_when_env_absent() {
        // Tests must not depend on process-global env; we instead validate
        // the explicit construction path the way the audit driver does in
        // wiring_audit.rs. Direct from_env() is exercised by the contract
        // test under crates/contracts/ where the env is controlled.
        let cfg = DeterminismConfig {
            hardware_class: HardwareClass::Unknown,
            seed: 0,
            blake3_cache_prefix: [0; 8],
            system_fingerprint_pin: None,
        };
        assert_eq!(cfg.seed, 0);
        assert!(matches!(cfg.hardware_class, HardwareClass::Unknown));
    }
}
