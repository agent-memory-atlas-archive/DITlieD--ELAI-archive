//! I-34 — `elai bench crane` CLI driver.
//!
//! Deterministic CRANE hybrid grammar-pin gate fixture harness. Drives the
//! three-layer I-34 gate (eligibility -> blake3 pin verify -> post-parse
//! re-validation) via the REAL `call_grammar_gate` orchestrator entry point,
//! NOT `emit_grammar_gate_event` directly. Each arm produces an off-author
//! SSB row — the gate itself emits the row, not this fixture.
//!
//! # What runs
//!
//! The harness exercises all three GrammarGateEvent Kind arms:
//!
//! 1. `CranePassed`       — compliant fixture + clean manifest passes all layers.
//! 2. `PinViolation`      — manifest blake3 hex tampered; gate detects mismatch.
//! 3. `PostParseRejected` — manifest passes but output is schema non-compliant.
//!
//! Each arm emits one `GrammarGateEvent` SSB row via `call_grammar_gate`.
//! The harness exits 0 on a clean run or 1 on any infrastructure failure.
//!
//! # Flags
//!
//!   --crane-fixture PATH   Path to an external `.ebnf` fixture file for pin
//!                          verification arms. Optional; defaults to a synthetic
//!                          in-memory grammar string so CI is fully hermetic.
//!   --seed <u64>           Seed for test-ordering (default 42). Reserved for
//!                          future deterministic shuffle; currently a no-op.
//!
//! # Wiring
//!
//! producer  : `call_grammar_gate` (grammar_gate.rs) — the REAL 3-layer gate.
//! consumer  : action_type_lock golden + i34_grammar_gate_contract.
//! ssb rows  : `SsbActionType::GrammarGateEvent` × 3 (one per Kind arm).
//! off-author: the gate emits the rows; this fixture only drives the gate.

use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Context;
use elai_core::clock::SeededClock;
use elai_orchestrator::grammar_gate::{call_grammar_gate, GrammarGateError, GrammarGateResult};
use elai_orchestrator::ssb::{
    decode_ssb_payload, GrammarGateEventKind, Ssb, SsbActionType, SsbConfig, SsbPayload,
    SsbReader,
};
use elai_providers::{DecodingMode, GrammarPinManifest, GrammarPinRecord};
use thiserror::Error;
use tracing::info;

// ---------------------------------------------------------------------------
// Temp-dir helper (avoids adding tempfile to [dependencies])
// ---------------------------------------------------------------------------

/// Creates a unique temporary directory under `std::env::temp_dir()`.
/// Returns the path; the caller is responsible for cleanup.
fn make_temp_dir(prefix: &str) -> anyhow::Result<PathBuf> {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    let pid = std::process::id();
    let dir = std::env::temp_dir().join(format!("{prefix}_{pid}_{ts}"));
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("could not create temp dir {}", dir.display()))?;
    Ok(dir)
}

// ---------------------------------------------------------------------------
// Fixture helpers
// ---------------------------------------------------------------------------

/// Write `content` to `dir/name`, return (path, blake3_hex_of_content).
fn write_ebnf(dir: &Path, name: &str, content: &[u8]) -> anyhow::Result<(PathBuf, String)> {
    let path = dir.join(name);
    let mut f = std::fs::File::create(&path)
        .with_context(|| format!("create ebnf fixture {}", path.display()))?;
    f.write_all(content)
        .with_context(|| format!("write ebnf fixture {}", path.display()))?;
    let hex = blake3::hash(content).to_hex().to_string();
    Ok((path, hex))
}

/// Serialize `manifest` as TOML and write to `dir/manifest.toml`. Returns the path.
fn write_manifest(dir: &Path, manifest: &GrammarPinManifest) -> anyhow::Result<PathBuf> {
    let toml_str = toml::to_string(manifest)
        .context("serialize GrammarPinManifest as TOML")?;
    let path = dir.join("manifest.toml");
    std::fs::write(&path, &toml_str)
        .with_context(|| format!("write manifest {}", path.display()))?;
    Ok(path)
}

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/// Error type for the bench crane harness.
#[derive(Debug, Error)]
pub enum BenchCraneError {
    /// SSB infrastructure failure.
    #[error("bench crane SSB error: {0}")]
    Ssb(#[from] elai_orchestrator::ssb::SsbError),

    /// Grammar gate error during gate call.
    #[error("bench crane gate error: {0}")]
    Gate(#[from] GrammarGateError),

    /// Tokio runtime construction failed.
    #[error("bench crane could not build tokio runtime: {0}")]
    RuntimeBuild(String),

    /// I/O failure setting up fixture files.
    #[error("bench crane fixture I/O error at {path}: {source_msg}")]
    FixtureIo { path: String, source_msg: String },

    /// The gate returned an unexpected result arm for this arm's scenario.
    #[error("bench crane unexpected gate result for arm {arm}: expected {expected}, got {actual:?}")]
    UnexpectedResult {
        arm: &'static str,
        expected: &'static str,
        actual: String,
    },
}

// ---------------------------------------------------------------------------
// CLI entry
// ---------------------------------------------------------------------------

/// CLI entry — builds a tokio runtime, runs the fixture harness, exits 0 on
/// a clean run.
pub(crate) fn run(crane_fixture: Option<String>, _seed: Option<u64>) -> anyhow::Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| anyhow::anyhow!("tokio runtime build failed: {e}"))?;

    let outcome = rt.block_on(async move { run_async(crane_fixture).await });

    match outcome {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("CRANE ABORT: {e}");
            std::process::exit(1);
        }
    }
}

// ---------------------------------------------------------------------------
// Async core
// ---------------------------------------------------------------------------

async fn run_async(crane_fixture: Option<String>) -> anyhow::Result<()> {
    // Build an ephemeral SSB in a tempdir so the bench is fully hermetic.
    let tmp_dir = make_temp_dir("bench_crane")?;
    let ssb = Arc::new(
        Ssb::new(
            SsbConfig {
                db_path: tmp_dir.join("bench_crane.sqlite"),
                pool_size: Some(2),
                project_id: "bench-crane".into(),
                session_id: "s-bench-crane".into(),
            },
            Arc::new(SeededClock::new()),
        )
        .context("could not initialise SSB for bench crane")?,
    );

    // Synthetic in-memory EBNF grammar (or load from --crane-fixture flag).
    let base_ebnf: &[u8] = b"start ::= object ; object ::= \"{\" \"}\" ;";
    let (ebnf_content, ebnf_fixture_path_hint): (Vec<u8>, String) = match crane_fixture {
        Some(ref p) => {
            let content = std::fs::read(p)
                .map_err(|e| BenchCraneError::FixtureIo {
                    path: p.clone(),
                    source_msg: e.to_string(),
                })
                .context("reading --crane-fixture")?;
            (content, p.clone())
        }
        None => (base_ebnf.to_vec(), "synthetic://crane_fixture.ebnf".to_string()),
    };
    let _ = ebnf_fixture_path_hint; // used in arm fixtures below

    // Schema JSON used for post-parse re-validation (arms 1 and 3).
    let schema_json = r#"{"type":"object","required":["name","score"],"properties":{"name":{"type":"string"},"score":{"type":"number"}}}"#;

    // -----------------------------------------------------------------------
    // Arm 1: CranePassed
    // -----------------------------------------------------------------------
    // Clean ebnf file, manifest pinned to its correct blake3 digest, compliant output.
    // Expected gate result: GrammarGateResult::Passed (emits CranePassed row).
    info!("bench crane: arm 1 — CranePassed");
    {
        let arm_dir = make_temp_dir("bench_crane_arm1")?;
        let (ebnf_path, ebnf_hex) = write_ebnf(&arm_dir, "grammar.ebnf", &ebnf_content)?;
        let manifest = GrammarPinManifest {
            entries: vec![GrammarPinRecord {
                file_path: ebnf_path.clone(),
                blake3_hex: ebnf_hex,
                pinned_at_ms: 1_700_000_000_000,
            }],
        };
        let manifest_path = write_manifest(&arm_dir, &manifest)?;
        let mode = DecodingMode::Grammar {
            ebnf_path: ebnf_path.clone(),
            crane_adaptive: true,
        };
        // Compliant output satisfying the schema.
        let compliant_output = r#"{"name":"crane_test","score":0.95}"#;

        let result = call_grammar_gate(
            &ssb,
            "s-bench-crane",
            &mode,
            &manifest_path,
            compliant_output,
            schema_json,
        )
        .await
        .context("arm 1 call_grammar_gate")?;

        if result != GrammarGateResult::Passed {
            return Err(BenchCraneError::UnexpectedResult {
                arm: "CranePassed",
                expected: "Passed",
                actual: format!("{result:?}"),
            }
            .into());
        }
        let _ = std::fs::remove_dir_all(&arm_dir);
    }

    // -----------------------------------------------------------------------
    // Arm 2: PinViolation
    // -----------------------------------------------------------------------
    // Pin the original ebnf content, then overwrite the file with different bytes.
    // load_and_verify will detect the mismatch and return PinViolation.
    // Expected gate result: GrammarGateResult::PinViolation (emits PinViolation row).
    info!("bench crane: arm 2 — PinViolation");
    {
        let arm_dir = make_temp_dir("bench_crane_arm2")?;
        let original_content = b"start ::= \"hello\" ;";
        let (ebnf_path, original_hex) = write_ebnf(&arm_dir, "grammar.ebnf", original_content)?;
        // Manifest records the original digest.
        let manifest = GrammarPinManifest {
            entries: vec![GrammarPinRecord {
                file_path: ebnf_path.clone(),
                blake3_hex: original_hex,
                pinned_at_ms: 1_700_000_000_000,
            }],
        };
        let manifest_path = write_manifest(&arm_dir, &manifest)?;
        // Overwrite the file after pinning — tamper!
        std::fs::write(&ebnf_path, b"start ::= \"evil\" ;")
            .context("tamper ebnf for arm 2")?;

        let mode = DecodingMode::Grammar {
            ebnf_path: ebnf_path.clone(),
            crane_adaptive: true,
        };
        // Output doesn't matter — gate returns before post-parse on PinViolation.
        let output = r#"{"name":"tampered","score":0.0}"#;

        let result = call_grammar_gate(
            &ssb,
            "s-bench-crane",
            &mode,
            &manifest_path,
            output,
            schema_json,
        )
        .await
        .context("arm 2 call_grammar_gate")?;

        if !matches!(result, GrammarGateResult::PinViolation { .. }) {
            return Err(BenchCraneError::UnexpectedResult {
                arm: "PinViolation",
                expected: "PinViolation { .. }",
                actual: format!("{result:?}"),
            }
            .into());
        }
        let _ = std::fs::remove_dir_all(&arm_dir);
    }

    // -----------------------------------------------------------------------
    // Arm 3: PostParseRejected
    // -----------------------------------------------------------------------
    // Clean ebnf + matching manifest (pin check passes), but output missing required 'score'.
    // Expected gate result: GrammarGateResult::PostParseRejected (emits PostParseRejected row).
    info!("bench crane: arm 3 — PostParseRejected");
    {
        let arm_dir = make_temp_dir("bench_crane_arm3")?;
        let (ebnf_path, ebnf_hex) = write_ebnf(&arm_dir, "grammar.ebnf", &ebnf_content)?;
        let manifest = GrammarPinManifest {
            entries: vec![GrammarPinRecord {
                file_path: ebnf_path.clone(),
                blake3_hex: ebnf_hex,
                pinned_at_ms: 1_700_000_000_000,
            }],
        };
        let manifest_path = write_manifest(&arm_dir, &manifest)?;
        let mode = DecodingMode::Grammar {
            ebnf_path: ebnf_path.clone(),
            crane_adaptive: true,
        };
        // Missing required 'score' field — silently non-compliant (landmine #7).
        let non_compliant_output = r#"{"name":"missing_score_field"}"#;

        let result = call_grammar_gate(
            &ssb,
            "s-bench-crane",
            &mode,
            &manifest_path,
            non_compliant_output,
            schema_json,
        )
        .await
        .context("arm 3 call_grammar_gate")?;

        if !matches!(result, GrammarGateResult::PostParseRejected { .. }) {
            return Err(BenchCraneError::UnexpectedResult {
                arm: "PostParseRejected",
                expected: "PostParseRejected { .. }",
                actual: format!("{result:?}"),
            }
            .into());
        }
        let _ = std::fs::remove_dir_all(&arm_dir);
    }

    // -----------------------------------------------------------------------
    // Verify all three SSB rows landed (off-author artifact check).
    // query_by_type returns ALL grammar_gate_event rows (all Kind arms share
    // the SQL string "grammar_gate_event"). Deserialize each payload and assert
    // one row per Kind arm.
    // -----------------------------------------------------------------------
    let rows = ssb
        .query_by_type(SsbActionType::GrammarGateEvent(GrammarGateEventKind::CranePassed))
        .context("querying GrammarGateEvent rows")?;
    if rows.len() != 3 {
        anyhow::bail!(
            "bench crane: expected 3 GrammarGateEvent rows, got {}",
            rows.len()
        );
    }

    let mut kinds: Vec<GrammarGateEventKind> = rows
        .iter()
        .map(|entry| {
            let bytes: &[u8] = &entry.payload;
            let payload = decode_ssb_payload(bytes)
                .expect("decode_ssb_payload in bench verify");
            match payload {
                SsbPayload::GrammarGateEvent(p) => p.kind,
                other => panic!("expected GrammarGateEvent payload, got {other:?}"),
            }
        })
        .collect();
    kinds.sort_by_key(|k| k.as_str());

    let mut expected_kinds = vec![
        GrammarGateEventKind::CranePassed,
        GrammarGateEventKind::PinViolation,
        GrammarGateEventKind::PostParseRejected,
    ];
    expected_kinds.sort_by_key(|k| k.as_str());

    if kinds != expected_kinds {
        anyhow::bail!(
            "bench crane: SSB row kinds mismatch. expected {:?}, got {:?}",
            expected_kinds,
            kinds
        );
    }

    // Best-effort cleanup.
    let _ = std::fs::remove_dir_all(&tmp_dir);

    info!("bench crane: all 3 GrammarGateEvent rows verified (CranePassed / PinViolation / PostParseRejected)");
    println!("CRANE bench PASS: 3 GrammarGateEvent rows written (gate-driven, off-author)");
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn bench_crane_async_emits_three_rows() {
        run_async(None).await.expect("bench_crane_async should pass");
    }
}
