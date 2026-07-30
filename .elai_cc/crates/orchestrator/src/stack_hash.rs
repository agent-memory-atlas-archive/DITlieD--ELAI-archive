//! I-18 — 5-layer blake3 stack hash sealed into the RRA by [`crate::rra::build_rra`].
//!
//! Split out of `rra.rs` for maintainability (MR-1). The `StackHash` struct,
//! `StackHashMismatch` error, the `verify` impl, and the `seal_stack_hash`
//! producer live here. `rra.rs` re-exports the public types so existing import
//! paths (`elai_orchestrator::rra::StackHash`) continue to resolve.

use serde::{Deserialize, Serialize};

use crate::determinism::DeterminismConfig;
use crate::ssb::{SsbEntry, SsbPayload};

/// I-18: 5-layer blake3 stack hash sealed into the RRA by [`crate::rra::build_rra`].
///
/// Each layer is a blake3 16-hex prefix (same truncation as the bundle root).
/// The five layers are computed over the ordered SSB stream at seal time:
/// - `state`:   blake3 of the concatenated SSB entry bytes (snapshot of the
///   action stream state at seal time; any row mutation changes this layer).
/// - `deps`:    blake3 of the Cargo.lock bytes at seal time (dependency pin);
///   a mid-run Cargo.lock mutation is caught here and halts the run.
/// - `process`: blake3 of the process-execution environment fingerprint
///   (seed || hardware_class || system_fingerprint_pin).
/// - `result`:  blake3 of the sealed objective hash bytes (the I-45
///   `objective_hash` field from `build_rra`); a tampered or absent objective
///   changes this layer. When no objective was framed, hashes the sentinel
///   string `"no-objective"`.
/// - `verify`:  blake3 of (state || deps || process || result) as the
///   cross-layer integrity combiner — a single-layer tamper changes this.
///
/// All five layers use blake3 (CLAUDE.md hard rule, no sha256/blake2).
/// `None` on LEGACY receipts (pre-I-18); `#[serde(default)]` on `stack_hash`
/// in `RunArtifact` preserves backward compat.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StackHash {
    /// blake3 16-hex of the SSB action-stream state at seal time.
    pub state: String,
    /// blake3 16-hex of the Cargo.lock bytes at seal time.
    pub deps: String,
    /// blake3 16-hex of the process environment (seed + hw_class + fp_pin).
    pub process: String,
    /// blake3 16-hex of the sealed objective hash bytes (I-45 `objective_hash`
    /// from `build_rra`). Sentinel `"no-objective"` when no objective was framed.
    pub result: String,
    /// blake3 16-hex of the cross-layer combiner (state || deps || process || result).
    pub verify: String,
}

/// I-18: which layer of the [`StackHash`] mismatched during verify, with the
/// sealed and re-derived values. Returned by [`StackHash::verify`] and carried
/// in [`crate::rra::ReplayVerdict::StackHashMismatch`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StackHashMismatch {
    /// Which layer mismatched: `"state"` / `"process"` / `"result"` / `"verify"`.
    /// `"deps"` is not re-derivable from the RRA alone (Cargo.lock bytes are not
    /// bundled) and is therefore never reported here.
    pub layer: String,
    /// The sealed layer hash (from the RRA's `stack_hash`).
    pub expected: String,
    /// The re-derived layer hash (from the RRA's current data).
    pub got: String,
}

impl StackHash {
    /// Re-derive the 5 layers from the RRA data and compare against the sealed
    /// values. Returns `Ok(())` if all re-derivable layers match, or
    /// `Err(StackHashMismatch)` naming the first mismatched layer.
    ///
    /// Re-derivable layers:
    /// - `state`:   blake3 of the concatenated event payload bytes (from
    ///   `art.events`). Any event payload mutation changes this.
    /// - `process`: blake3 of (`seed || hardware_class || system_fingerprint_pin`).
    ///   A pin mutation changes this.
    /// - `result`:  blake3 of the `objective_hash` bytes (or `"no-objective"`).
    ///   An objective mutation changes this.
    /// - `verify`:  blake3 of (`state || deps || process || result`) using the
    ///   sealed `deps` (not re-derivable) and the already-verified sealed
    ///   `state`/`process`/`result`. A tampered `verify` field (patched without
    ///   patching the combiner inputs) is caught here.
    ///
    /// NOT re-derivable:
    /// - `deps`: the Cargo.lock bytes are not bundled in the RRA. The sealed
    ///   `deps` value is used in the `verify` combiner re-derivation so a
    ///   tampered `state`/`process`/`result` still changes the combiner. The
    ///   run-time halt for a mid-run Cargo.lock mutation is a separate step
    ///   (STEP 3 of I-18), not this verify leg.
    ///
    /// All layers use blake3 (CLAUDE.md Anti-Stale-Training hard rule).
    #[must_use]
    pub fn verify(&self, art: &crate::rra::RunArtifact) -> Result<(), StackHashMismatch> {
        let state_input: Vec<u8> = art
            .events
            .iter()
            .flat_map(|e| e.payload.iter().copied())
            .collect();
        let state = crate::rra::truncate_hex(&blake3::hash(&state_input).to_hex());
        if state != self.state {
            return Err(StackHashMismatch {
                layer: "state".to_owned(),
                expected: self.state.clone(),
                got: state,
            });
        }

        let fp_pin = art
            .system_fingerprint_pin
            .as_deref()
            .unwrap_or("")
            .as_bytes();
        let mut process_input = Vec::new();
        process_input.extend_from_slice(&art.seed.to_le_bytes());
        process_input.extend_from_slice(art.hardware_class.as_bytes());
        process_input.extend_from_slice(fp_pin);
        let process = crate::rra::truncate_hex(&blake3::hash(&process_input).to_hex());
        if process != self.process {
            return Err(StackHashMismatch {
                layer: "process".to_owned(),
                expected: self.process.clone(),
                got: process,
            });
        }

        let result_input = art
            .objective_hash
            .as_deref()
            .unwrap_or("no-objective")
            .as_bytes();
        let result = crate::rra::truncate_hex(&blake3::hash(result_input).to_hex());
        if result != self.result {
            return Err(StackHashMismatch {
                layer: "result".to_owned(),
                expected: self.result.clone(),
                got: result,
            });
        }

        let mut verify_input = Vec::new();
        verify_input.extend_from_slice(self.state.as_bytes());
        verify_input.extend_from_slice(self.deps.as_bytes());
        verify_input.extend_from_slice(self.process.as_bytes());
        verify_input.extend_from_slice(self.result.as_bytes());
        let verify = crate::rra::truncate_hex(&blake3::hash(&verify_input).to_hex());
        if verify != self.verify {
            return Err(StackHashMismatch {
                layer: "verify".to_owned(),
                expected: self.verify.clone(),
                got: verify,
            });
        }

        Ok(())
    }
}

/// I-18: blake3 16-hex of the Cargo.lock bytes (DEPS layer formula).
///
/// Path = `CARGO_MANIFEST_DIR/../Cargo.lock`; bytes read or `"no-cargo-lock"`
/// sentinel. Same formula as the DEPS layer in [`seal_stack_hash`]; extracted
/// so the CLI run-time baseline check and the seal-time hash share one source.
#[must_use]
pub fn cargo_lock_deps_hex() -> String {
    let cargo_lock_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../Cargo.lock");
    let cargo_bytes = std::fs::read(&cargo_lock_path)
        .unwrap_or_else(|_| b"no-cargo-lock".to_vec());
    crate::rra::truncate_hex(&blake3::hash(&cargo_bytes).to_hex())
}

/// I-18 R1: compare a run-start baseline against the current Cargo.lock DEPS
/// hash. `Ok(())` when unchanged; `Err(message)` when the lockfile mutated
/// mid-run. Production caller: `emit_run_artifact` before ChainAdvanced.
pub fn check_cargo_lock_baseline(baseline: &str) -> Result<(), String> {
    let now = cargo_lock_deps_hex();
    if now != baseline {
        return Err(format!(
            "I-18 chain broken: Cargo.lock mutated mid-run (deps baseline {baseline} != seal {now})"
        ));
    }
    Ok(())
}

/// I-18: Compute the 5-layer [`StackHash`] sealed into the RRA.
///
/// Each layer is a blake3 16-hex prefix (same truncation as the bundle root).
/// All layers use blake3 (no sha256 / blake2, CLAUDE.md Anti-Stale-Training).
///
/// This is a Tier-1 offline computation — no new deps, no cosign-rs. The
/// cosign-rs Rekor push path stays behind `#[cfg(feature="cosign-attest")]`
/// in attestation.rs:517.
pub(crate) fn seal_stack_hash(
    decoded: &[(SsbEntry, Option<SsbPayload>)],
    pins: &DeterminismConfig,
    objective_hash: &Option<String>,
) -> StackHash {
    // STATE layer: hash of the concatenated raw payload bytes from the SSB
    // stream at seal time (any row mutation changes this layer).
    let state_input: Vec<u8> = decoded
        .iter()
        .flat_map(|(e, _)| e.payload.iter().copied())
        .collect();
    let state = crate::rra::truncate_hex(&blake3::hash(&state_input).to_hex());

    // DEPS layer: hash of the Cargo.lock bytes if readable, else hash of a
    // deterministic "no-cargo-lock" sentinel (so the field is never empty).
    let deps = cargo_lock_deps_hex();

    // PROCESS layer: hash of (seed || hardware_class_label || system_fingerprint_pin).
    // hardware_class_label() produces the same canonical string stored in the
    // RunArtifact.hardware_class field ("cpu" / "gpu:<sku>" / "unknown").
    // system_fingerprint_pin is Option<String>; None serializes as empty bytes.
    let hw_label = crate::rra::hardware_class_label(&pins.hardware_class);
    let fp_pin = pins
        .system_fingerprint_pin
        .as_deref()
        .unwrap_or("")
        .as_bytes();
    let mut process_input = Vec::new();
    process_input.extend_from_slice(&pins.seed.to_le_bytes());
    process_input.extend_from_slice(hw_label.as_bytes());
    process_input.extend_from_slice(fp_pin);
    let process = crate::rra::truncate_hex(&blake3::hash(&process_input).to_hex());

    // RESULT layer: hash of the objective_hash bytes (G-2 bundle root proxy).
    // A tampered artifact changes the objective hash and therefore this layer.
    let result = {
        let input = objective_hash
            .as_deref()
            .unwrap_or("no-objective")
            .as_bytes();
        crate::rra::truncate_hex(&blake3::hash(input).to_hex())
    };

    // VERIFY combiner: hash of (state || deps || process || result) as the
    // cross-layer integrity combiner — a single-layer tamper changes this.
    let mut verify_input = Vec::new();
    verify_input.extend_from_slice(state.as_bytes());
    verify_input.extend_from_slice(deps.as_bytes());
    verify_input.extend_from_slice(process.as_bytes());
    verify_input.extend_from_slice(result.as_bytes());
    let verify = crate::rra::truncate_hex(&blake3::hash(&verify_input).to_hex());

    StackHash { state, deps, process, result, verify }
}

#[cfg(test)]
mod i18_stack_hash_tests {
    use super::*;
    use crate::determinism::{DeterminismConfig, GpuSku, HardwareClass};
    use crate::rra::{build_rra, compute_root, verify_artifact, ReplayVerdict, RRA_HEX_PREFIX_LEN};
    use crate::ssb::{SsbActionType, SsbEntry};
    use std::sync::Arc;

    fn make_entry(seq: i64, payload: &[u8]) -> SsbEntry {
        SsbEntry {
            seq,
            project_id: "test".to_owned(),
            session_id: "test-session".to_owned(),
            agent_id: "test-agent".to_owned(),
            superstep: 0,
            action_type: SsbActionType::Checkpoint,
            payload: Arc::from(payload),
            created_at: 0,
        }
    }

    fn local_pins(seed: u64) -> DeterminismConfig {
        DeterminismConfig {
            hardware_class: HardwareClass::Gpu(GpuSku::new("rtx3090")),
            seed,
            blake3_cache_prefix: [0u8; 8],
            system_fingerprint_pin: Some("test-fp".to_owned()),
        }
    }

    #[test]
    fn build_rra_seals_stack_hash_with_all_layers() {
        let stream = vec![
            make_entry(0, b"payload-zero"),
            make_entry(1, b"payload-one"),
        ];
        let pins = local_pins(42);
        let art = build_rra("test-session", &stream, &pins);

        let stack = art
            .stack_hash
            .as_ref()
            .expect("build_rra must seal a StackHash");
        assert!(!stack.state.is_empty(), "state layer must be non-empty");
        assert!(!stack.deps.is_empty(), "deps layer must be non-empty");
        assert!(!stack.process.is_empty(), "process layer must be non-empty");
        assert!(!stack.result.is_empty(), "result layer must be non-empty");
        assert!(!stack.verify.is_empty(), "verify layer must be non-empty");
        for (name, val) in [
            ("state", &stack.state),
            ("deps", &stack.deps),
            ("process", &stack.process),
            ("result", &stack.result),
            ("verify", &stack.verify),
        ] {
            assert_eq!(
                val.len(),
                RRA_HEX_PREFIX_LEN,
                "{name} layer must be {RRA_HEX_PREFIX_LEN} hex chars"
            );
        }
    }

    #[test]
    fn stack_hash_verify_passes_on_fresh_artifact() {
        let stream = vec![
            make_entry(0, b"payload-zero"),
            make_entry(1, b"payload-one"),
        ];
        let pins = local_pins(42);
        let art = build_rra("test-session", &stream, &pins);

        let stack = art
            .stack_hash
            .as_ref()
            .expect("stack_hash must be present");
        assert!(
            stack.verify(&art).is_ok(),
            "freshly built artifact must pass stack hash verify"
        );
    }

    #[test]
    fn stack_hash_verify_catches_tampered_state() {
        let stream = vec![
            make_entry(0, b"payload-zero"),
            make_entry(1, b"payload-one"),
        ];
        let pins = local_pins(42);
        let mut art = build_rra("test-session", &stream, &pins);

        art.events[0].payload[0] ^= 0xFF;

        let stack = art
            .stack_hash
            .as_ref()
            .expect("stack_hash must be present");
        let err = stack
            .verify(&art)
            .expect_err("tampered state layer must fail verify");
        assert_eq!(err.layer, "state", "must name the state layer");
        assert_ne!(err.expected, err.got, "expected and got must differ");
    }

    #[test]
    fn stack_hash_verify_catches_tampered_process() {
        let stream = vec![
            make_entry(0, b"payload-zero"),
            make_entry(1, b"payload-one"),
        ];
        let pins = local_pins(42);
        let mut art = build_rra("test-session", &stream, &pins);

        art.seed = 999;

        let stack = art
            .stack_hash
            .as_ref()
            .expect("stack_hash must be present");
        let err = stack
            .verify(&art)
            .expect_err("tampered process layer must fail verify");
        assert_eq!(err.layer, "process", "must name the process layer");
        assert_ne!(err.expected, err.got, "expected and got must differ");
    }

    #[test]
    fn stack_hash_verify_catches_tampered_result() {
        let stream = vec![
            make_entry(0, b"payload-zero"),
            make_entry(1, b"payload-one"),
        ];
        let pins = local_pins(42);
        let mut art = build_rra("test-session", &stream, &pins);

        art.objective_hash = Some("deadbeefdeadbeef".to_owned());

        let stack = art
            .stack_hash
            .as_ref()
            .expect("stack_hash must be present");
        let err = stack
            .verify(&art)
            .expect_err("tampered result layer must fail verify");
        assert_eq!(err.layer, "result", "must name the result layer");
        assert_ne!(err.expected, err.got, "expected and got must differ");
    }

    #[test]
    fn verify_artifact_returns_stack_hash_mismatch_on_tamper() {
        let stream = vec![
            make_entry(0, b"payload-zero"),
            make_entry(1, b"payload-one"),
        ];
        let pins = local_pins(42);
        let mut art = build_rra("test-session", &stream, &pins);

        art.events[0].payload[0] ^= 0xFF;
        art.events[0].event_blake3 = art.events[0].recompute_content_blake3();
        art.root_blake3 = compute_root(&art).unwrap_or_default();

        let verdict = verify_artifact(&art).expect("verify_artifact must not error");
        match verdict {
            ReplayVerdict::StackHashMismatch { layer, .. } => {
                assert_eq!(layer, "state", "must name the state layer");
            }
            other => panic!("expected StackHashMismatch, got {other:?}"),
        }
    }

    #[test]
    fn legacy_receipt_without_stack_hash_still_verifies() {
        let stream = vec![
            make_entry(0, b"payload-zero"),
            make_entry(1, b"payload-one"),
        ];
        let pins = local_pins(42);
        let mut art = build_rra("test-session", &stream, &pins);

        art.stack_hash = None;
        art.root_blake3 = compute_root(&art).unwrap_or_default();

        let verdict = verify_artifact(&art).expect("verify_artifact must not error");
        assert_eq!(
            verdict,
            ReplayVerdict::Verified,
            "LEGACY receipt without stack_hash must still verify"
        );
    }

    #[test]
    fn cargo_lock_deps_hex_is_stable_and_nonempty() {
        let h = cargo_lock_deps_hex();
        assert_eq!(h.len(), RRA_HEX_PREFIX_LEN, "deps hex must be 16 chars");
        assert!(
            h.chars().all(|c| c.is_ascii_hexdigit()),
            "deps hex must be lowercase hex"
        );
        let h2 = cargo_lock_deps_hex();
        assert_eq!(h, h2, "deps hex must be stable across calls");
    }

    #[test]
    fn check_cargo_lock_baseline_ok_when_unchanged() {
        let baseline = cargo_lock_deps_hex();
        assert!(
            check_cargo_lock_baseline(&baseline).is_ok(),
            "baseline matching current lock must pass"
        );
    }

    #[test]
    fn check_cargo_lock_baseline_err_on_mismatch() {
        let fake = "0000000000000000";
        let err = check_cargo_lock_baseline(fake)
            .expect_err("fake baseline must mismatch current lock");
        assert!(
            err.contains("chain broken"),
            "error message must name the failure: {err}"
        );
    }
}
