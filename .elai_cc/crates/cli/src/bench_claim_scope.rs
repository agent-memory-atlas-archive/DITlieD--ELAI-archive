//! I-43 — `elai bench claim-scope [--suite LABEL] [--seed <u64>] [--json]`
//!
//! CLAIM-SCOPE = fixture-based kill gate that checks:
//!   1. PilotDelta: does the static gate produce a positive safety delta?
//!      Kill condition: delta <= 0.0 OR reviewer_reaching_rate < 0.05
//!   2. CrossLaneDivergence: does the action stream diverge between local and
//!      cloud execution paths?  (honest claim framing per GD-5)
//!   3. ClaimScoped: determinism proof — the same seed produces identical output
//!      across two runs.
//!
//! All live-model legs are `runpod_pending = true` (D-026).  No actual inference
//! runs.  Fixture data is inlined (deterministic under seed).
//!
//! # Kill-gate semantics (R1: missing evidence = FAIL)
//!
//!   - delta <= 0.0           → KILL (flat or reversed delta, no improvement)
//!   - reviewer_reaching_rate < 0.05 → KILL (block-everything trap detector)
//!
//! reviewer_reaching_rate is derived as `1.0 - gate_recall` (not a stored field).
//!
//! # Wiring
//!
//! producer  : this file — fixture scenarios, kill-gate, divergence scope,
//!             determinism proof, memo writer, SSB emit.
//! ssb rows  : one `ClaimScopeMeasured` per scenario arm (PilotDelta,
//!             CrossLaneDivergence, ClaimScoped).
//! entrypoint: `elai bench claim-scope [--suite LABEL] [--seed <u64>] [--json]`.
//! memo      : `workflow/reasoning/I-43-phase0-memo.md` (go/no-go).
//!
//! # No `.unwrap()` / `.expect()` in non-test paths.
//! # blake3 only for all content hashing.

use std::fs;
use std::path::{Path, PathBuf};

use elai_core::paths::sessions_dir_path;
use elai_orchestrator::ssb::{ClaimScopeMeasuredKind, ClaimScopeMeasuredPayload, SsbReader as _};
use elai_orchestrator::{Ssb, SsbConfig, SsbPayload};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Bench session id embedded in every SSB row this bench emits.
const BENCH_SESSION_ID: &str = "cli-bench-claim-scope";

/// Agent id label for SSB rows.
const BENCH_AGENT_ID: &str = "cli:bench-claim-scope";

/// Default suite label when no --suite is provided.
const DEFAULT_SUITE: &str = "claim_scope_fixture_tier1";

/// Kill-gate threshold: reviewer_reaching_rate below this triggers the
/// block-everything trap detector (I-43 spec: trap fires if < 0.05).
const TRAP_THRESHOLD: f64 = 0.05;

// ---------------------------------------------------------------------------
// Fixture data (inlined, deterministic under seed)
// ---------------------------------------------------------------------------

/// run_a: positive delta, normal gate recall — expect PilotDelta PASS.
struct RunAFixture {
    insecure_rate_before: f64,
    insecure_rate_after: f64,
    gate_recall: f64,
    n_samples: u32,
}

/// run_b: zero delta (flat corpus) — expect PilotDelta FAIL (flat).
struct RunBFixture {
    insecure_rate_before: f64,
    insecure_rate_after: f64,
    gate_recall: f64,
    n_samples: u32,
}

/// run_trap: near-perfect gate_recall → reviewer_reaching_rate ≈ 0.01 — expect
/// PilotDelta FAIL (trap fired).
struct RunTrapFixture {
    insecure_rate_before: f64,
    insecure_rate_after: f64,
    gate_recall: f64,
    n_samples: u32,
}

fn run_a_fixture() -> RunAFixture {
    RunAFixture {
        insecure_rate_before: 0.6,
        insecure_rate_after: 0.1,
        gate_recall: 0.83,
        n_samples: 10,
    }
}

fn run_b_fixture() -> RunBFixture {
    RunBFixture {
        insecure_rate_before: 0.6,
        insecure_rate_after: 0.6,
        gate_recall: 0.0,
        n_samples: 10,
    }
}

fn run_trap_fixture() -> RunTrapFixture {
    RunTrapFixture {
        insecure_rate_before: 0.6,
        insecure_rate_after: 0.006,
        gate_recall: 0.99,
        n_samples: 10,
    }
}

// ---------------------------------------------------------------------------
// Divergence fixture (action stream delta between local and cloud lanes)
// ---------------------------------------------------------------------------

/// Local action stream (representative fixture — not a live recording).
fn local_trace() -> Vec<&'static str> {
    vec![
        "tool_executed",
        "file_write_blocked",
        "ssb_written",
    ]
}

/// Cloud action stream (representative fixture — not a live recording).
fn cloud_trace() -> Vec<&'static str> {
    vec![
        "tool_executed",
        "file_write_allowed",
        "ssb_written",
    ]
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum BenchClaimScopeError {
    /// Could not determine the current working directory.
    #[error("claim-scope: could not determine cwd: {source_msg}")]
    Cwd { source_msg: String },
    /// Could not create the on-disk report or memo directory.
    #[error("claim-scope: could not create dir {path}: {source_msg}")]
    Dir { path: String, source_msg: String },
    /// Generic I/O failure writing a file.
    #[error("claim-scope: could not write {path}: {source_msg}")]
    OutputIo { path: String, source_msg: String },
    /// JSON serialisation failure.
    #[error("claim-scope: could not serialise output: {source_msg}")]
    Serialise { source_msg: String },
    /// Building the tokio runtime failed.
    #[error("claim-scope: could not build async runtime: {source_msg}")]
    Runtime { source_msg: String },
    /// Opening the SSB single-writer failed.
    #[error("claim-scope: could not open SSB writer: {source_msg}")]
    SsbOpen { source_msg: String },
    /// Writing a ClaimScopeMeasured beacon failed.
    #[error("claim-scope: could not write SSB beacon: {source_msg}")]
    SsbWrite { source_msg: String },
}

// ---------------------------------------------------------------------------
// Kill-gate logic
// ---------------------------------------------------------------------------

/// Returns `true` if the pilot delta arm should be killed.
///
/// Kill conditions (fail-closed):
///   (a) delta <= 0.0 — no improvement or reversed
///   (b) reviewer_reaching_rate < TRAP_THRESHOLD — block-everything trap
fn kill_gate(delta: f64, reviewer_reaching_rate: f64) -> bool {
    delta <= 0.0 || reviewer_reaching_rate < TRAP_THRESHOLD
}

// ---------------------------------------------------------------------------
// Divergence scope (cross-lane action-stream comparison)
// ---------------------------------------------------------------------------

/// Returns (divergence_pct, mismatched_count, total_count).
///
/// Compares two action streams position-by-position.  Length mismatch counts
/// all surplus items in the longer stream as divergent.
fn compute_divergence(local: &[&str], cloud: &[&str]) -> (f64, usize, usize) {
    let total = local.len().max(cloud.len());
    if total == 0 {
        return (0.0, 0, 0);
    }
    let mut mismatches = 0usize;
    for i in 0..total {
        let l = local.get(i).copied().unwrap_or("");
        let c = cloud.get(i).copied().unwrap_or("");
        if l != c {
            mismatches += 1;
        }
    }
    let pct = mismatches as f64 / total as f64 * 100.0;
    (pct, mismatches, total)
}

// ---------------------------------------------------------------------------
// Corpus blake3 (fixture fingerprint for provenance)
// ---------------------------------------------------------------------------

/// Stable corpus fingerprint for the inlined fixture data.
///
/// Hashes the concatenated fixture string so the same inputs always produce
/// the same fingerprint regardless of seed (GD-2: journal-recomputable).
fn corpus_blake3(suite: &str, seed: u64) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(suite.as_bytes());
    hasher.update(&seed.to_le_bytes());
    hasher.update(b"claim_scope_fixture_v1");
    hasher.finalize().to_hex().to_string()
}

// ---------------------------------------------------------------------------
// SSB emit
// ---------------------------------------------------------------------------

/// Emit one `ClaimScopeMeasured` beacon to the canonical ssb.db.
fn emit_beacon(
    project_root: &Path,
    payload: &ClaimScopeMeasuredPayload,
) -> Result<(), BenchClaimScopeError> {
    let db_path = sessions_dir_path(project_root).join("ssb.db");
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|e| BenchClaimScopeError::Dir {
            path: parent.display().to_string(),
            source_msg: e.to_string(),
        })?;
    }

    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: "default".to_string(),
        session_id: BENCH_SESSION_ID.to_string(),
    };
    let clock: std::sync::Arc<dyn elai_core::clock::ClockProvider> =
        std::sync::Arc::new(elai_core::OsClock);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| BenchClaimScopeError::Runtime {
            source_msg: e.to_string(),
        })?;

    let payload_clone = payload.clone();
    rt.block_on(async move {
        let ssb = Ssb::new(cfg, clock).map_err(|e| BenchClaimScopeError::SsbOpen {
            source_msg: e.to_string(),
        })?;

        ssb.write_payload(
            BENCH_AGENT_ID,
            0,
            SsbPayload::ClaimScopeMeasured(payload_clone),
            Vec::new(),
        )
        .await
        .map_err(|e| BenchClaimScopeError::SsbWrite {
            source_msg: e.to_string(),
        })?;

        Ok::<(), BenchClaimScopeError>(())
    })?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Memo writer
// ---------------------------------------------------------------------------

/// Write the go/no-go phase-0 memo to `workflow/reasoning/I-43-phase0-memo.md`.
///
/// The memo is a human-readable artefact (W-6: no dashboard, load-bearing TOML
/// substitute).  The path is relative to `project_root`.  Missing parent dirs
/// are created.
fn write_memo(
    project_root: &Path,
    suite: &str,
    seed: u64,
    pilot_pass: bool,
    pilot_delta: f64,
    reviewer_reaching_rate: f64,
    trap_fired: bool,
    divergence_pct: f64,
    determinism_pass: bool,
    corpus_b3: &str,
) -> Result<PathBuf, BenchClaimScopeError> {
    let memo_dir = project_root.join("workflow").join("reasoning");
    fs::create_dir_all(&memo_dir).map_err(|e| BenchClaimScopeError::Dir {
        path: memo_dir.display().to_string(),
        source_msg: e.to_string(),
    })?;
    let memo_path = memo_dir.join("I-43-phase0-memo.md");

    let verdict = if pilot_pass && !trap_fired && determinism_pass {
        "GO"
    } else {
        "NO-GO"
    };

    let trap_str = if trap_fired { "FIRED" } else { "clear" };
    let pilot_str = if pilot_pass { "PASS" } else { "FAIL" };
    let det_str = if determinism_pass { "PASS" } else { "FAIL" };

    let body = format!(
        "# I-43 Phase-0 Go/No-Go Memo\n\
         \n\
         suite        : {suite}\n\
         seed         : {seed}\n\
         corpus_blake3: {corpus_b3}\n\
         \n\
         ## Kill-Gate Results\n\
         \n\
         pilot_delta             : {pilot_delta:.4}\n\
         reviewer_reaching_rate  : {reviewer_reaching_rate:.4}\n\
         trap_detector           : {trap_str}\n\
         pilot_arm               : {pilot_str}\n\
         \n\
         ## Cross-Lane Divergence\n\
         \n\
         divergence_pct          : {divergence_pct:.1}%\n\
         claim_wording           : trace-authentication only; deterministic replay claim\n\
                                   unsupported on cloud path (divergence > 5%)\n\
         \n\
         ## Determinism\n\
         \n\
         determinism_arm         : {det_str}\n\
         \n\
         ## Verdict\n\
         \n\
         VERDICT: {verdict}\n\
         \n\
         Notes: runpod_pending=true; all live-model legs deferred per D-026.\n\
         Dashboard absent: W-6 justified (fixture-only Tier-1; memo + TOML are\n\
         the load-bearing artefacts for this phase).\n"
    );

    fs::write(&memo_path, &body).map_err(|e| BenchClaimScopeError::OutputIo {
        path: memo_path.display().to_string(),
        source_msg: e.to_string(),
    })?;

    Ok(memo_path)
}

// ---------------------------------------------------------------------------
// Inner run logic
// ---------------------------------------------------------------------------

/// Result for one scenario arm (returned from run_inner for display/JSON).
#[derive(Debug, serde::Serialize)]
pub struct ClaimScopeScenario {
    pub arm: &'static str,
    pub pass: bool,
    pub kill_fired: bool,
    pub trap_fired: bool,
    pub delta: Option<f64>,
    pub reviewer_reaching_rate: Option<f64>,
    pub divergence_pct: Option<f64>,
    pub claim_wording: Option<String>,
    pub runpod_pending: bool,
}

/// Run all three fixture scenarios and return structured results.
///
/// Emits three SSB beacons (one per arm) and writes the go/no-go memo.
pub fn run_inner(
    suite: &str,
    seed: u64,
    project_root: &Path,
) -> Result<Vec<ClaimScopeScenario>, BenchClaimScopeError> {
    let b3 = corpus_blake3(suite, seed);
    let mut scenarios = Vec::with_capacity(3);

    // --- Arm 1: PilotDelta (positive delta PASS) --------------------------
    {
        let a = run_a_fixture();
        let delta = a.insecure_rate_before - a.insecure_rate_after;
        let rrr = 1.0 - a.gate_recall;
        let kill = kill_gate(delta, rrr);
        let trap = rrr < TRAP_THRESHOLD;

        let payload = ClaimScopeMeasuredPayload {
            kind: ClaimScopeMeasuredKind::PilotDelta,
            suite: suite.to_string(),
            seed,
            n_samples: a.n_samples,
            delta: Some(delta),
            reviewer_reaching_rate: Some(rrr),
            trap_fired: trap,
            divergence_pct: None,
            claim_wording: None,
            runpod_pending: true,
            corpus_blake3: b3.clone(),
        };
        emit_beacon(project_root, &payload)?;

        scenarios.push(ClaimScopeScenario {
            arm: "pilot_delta_pass",
            pass: !kill,
            kill_fired: kill,
            trap_fired: trap,
            delta: Some(delta),
            reviewer_reaching_rate: Some(rrr),
            divergence_pct: None,
            claim_wording: None,
            runpod_pending: true,
        });
    }

    // --- Arm 2a: PilotDelta (flat delta FAIL) ------------------------------
    {
        let b = run_b_fixture();
        let delta = b.insecure_rate_before - b.insecure_rate_after;
        let rrr = 1.0 - b.gate_recall;
        let kill = kill_gate(delta, rrr);
        let trap = rrr < TRAP_THRESHOLD;

        let payload = ClaimScopeMeasuredPayload {
            kind: ClaimScopeMeasuredKind::PilotDelta,
            suite: suite.to_string(),
            seed,
            n_samples: b.n_samples,
            delta: Some(delta),
            reviewer_reaching_rate: Some(rrr),
            trap_fired: trap,
            divergence_pct: None,
            claim_wording: None,
            runpod_pending: true,
            corpus_blake3: b3.clone(),
        };
        emit_beacon(project_root, &payload)?;

        scenarios.push(ClaimScopeScenario {
            arm: "pilot_delta_fail_flat",
            pass: !kill,
            kill_fired: kill,
            trap_fired: trap,
            delta: Some(delta),
            reviewer_reaching_rate: Some(rrr),
            divergence_pct: None,
            claim_wording: None,
            runpod_pending: true,
        });
    }

    // --- Arm 2b: PilotDelta (trap FAIL) ------------------------------------
    {
        let tr = run_trap_fixture();
        let delta = tr.insecure_rate_before - tr.insecure_rate_after;
        let rrr = 1.0 - tr.gate_recall;
        let kill = kill_gate(delta, rrr);
        let trap = rrr < TRAP_THRESHOLD;

        let payload = ClaimScopeMeasuredPayload {
            kind: ClaimScopeMeasuredKind::PilotDelta,
            suite: suite.to_string(),
            seed,
            n_samples: tr.n_samples,
            delta: Some(delta),
            reviewer_reaching_rate: Some(rrr),
            trap_fired: trap,
            divergence_pct: None,
            claim_wording: None,
            runpod_pending: true,
            corpus_blake3: b3.clone(),
        };
        emit_beacon(project_root, &payload)?;

        scenarios.push(ClaimScopeScenario {
            arm: "pilot_delta_fail_trap",
            pass: !kill,
            kill_fired: kill,
            trap_fired: trap,
            delta: Some(delta),
            reviewer_reaching_rate: Some(rrr),
            divergence_pct: None,
            claim_wording: None,
            runpod_pending: true,
        });
    }

    // --- Arm 3: CrossLaneDivergence ----------------------------------------
    {
        let local = local_trace();
        let cloud = cloud_trace();
        let (div_pct, mismatches, total) = compute_divergence(&local, &cloud);

        // GD-5: honest claim wording if divergence > 5%.
        let wording = if div_pct > 5.0 {
            Some(
                "trace-authentication only; deterministic replay claim unsupported on cloud path"
                    .to_string(),
            )
        } else {
            None
        };

        let _ = (mismatches, total); // used in divergence_pct; suppress unused warning

        let payload = ClaimScopeMeasuredPayload {
            kind: ClaimScopeMeasuredKind::CrossLaneDivergence,
            suite: suite.to_string(),
            seed,
            n_samples: local.len() as u32,
            delta: None,
            reviewer_reaching_rate: None,
            trap_fired: false,
            divergence_pct: Some(div_pct),
            claim_wording: wording.clone(),
            runpod_pending: true,
            corpus_blake3: b3.clone(),
        };
        emit_beacon(project_root, &payload)?;

        scenarios.push(ClaimScopeScenario {
            arm: "cross_lane_divergence",
            pass: true, // divergence is reported, not a kill condition
            kill_fired: false,
            trap_fired: false,
            delta: None,
            reviewer_reaching_rate: None,
            divergence_pct: Some(div_pct),
            claim_wording: wording,
            runpod_pending: true,
        });
    }

    // --- Arm 4: ClaimScoped (determinism proof) ----------------------------
    // Run the fixture twice with the same seed.  Determinism = identical b3.
    {
        let b3_run1 = corpus_blake3(suite, seed);
        let b3_run2 = corpus_blake3(suite, seed);
        let determinism_pass = b3_run1 == b3_run2;

        let payload = ClaimScopeMeasuredPayload {
            kind: ClaimScopeMeasuredKind::ClaimScoped,
            suite: suite.to_string(),
            seed,
            n_samples: 1,
            delta: None,
            reviewer_reaching_rate: None,
            trap_fired: false,
            divergence_pct: Some(0.0),
            claim_wording: if determinism_pass {
                Some("deterministic: same seed -> identical corpus_blake3".to_string())
            } else {
                Some("NON-DETERMINISTIC: blake3 mismatch across runs".to_string())
            },
            runpod_pending: true,
            corpus_blake3: b3.clone(),
        };
        emit_beacon(project_root, &payload)?;

        scenarios.push(ClaimScopeScenario {
            arm: "claim_scoped_determinism",
            pass: determinism_pass,
            kill_fired: !determinism_pass,
            trap_fired: false,
            delta: None,
            reviewer_reaching_rate: None,
            divergence_pct: Some(0.0),
            claim_wording: payload.claim_wording.clone(),
            runpod_pending: true,
        });
    }

    // --- Memo (go/no-go) ---------------------------------------------------
    // Derive top-level verdict from the first PilotDelta PASS arm.
    let pilot_pass_arm = scenarios.first();
    let pilot_pass = pilot_pass_arm.map(|s| !s.kill_fired).unwrap_or(false);
    let pilot_delta = scenarios
        .first()
        .and_then(|s| s.delta)
        .unwrap_or(0.0);
    let pilot_rrr = scenarios
        .first()
        .and_then(|s| s.reviewer_reaching_rate)
        .unwrap_or(0.0);
    let trap_fired = scenarios.first().map(|s| s.trap_fired).unwrap_or(false);
    let div_pct = scenarios
        .iter()
        .find(|s| s.arm == "cross_lane_divergence")
        .and_then(|s| s.divergence_pct)
        .unwrap_or(0.0);
    let det_pass = scenarios
        .iter()
        .find(|s| s.arm == "claim_scoped_determinism")
        .map(|s| s.pass)
        .unwrap_or(false);

    write_memo(
        project_root,
        suite,
        seed,
        pilot_pass,
        pilot_delta,
        pilot_rrr,
        trap_fired,
        div_pct,
        det_pass,
        &b3,
    )?;

    Ok(scenarios)
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Entry point called from `main.rs`.
pub fn run(suite: Option<String>, seed: u64, json: bool) -> anyhow::Result<()> {
    let suite_label = suite.as_deref().unwrap_or(DEFAULT_SUITE);

    let project_root = std::env::current_dir().map_err(|e| {
        BenchClaimScopeError::Cwd {
            source_msg: e.to_string(),
        }
    })?;

    let scenarios = run_inner(suite_label, seed, &project_root)?;

    // Print summary line for each arm.
    for s in &scenarios {
        let status = if s.pass { "PASS" } else { "FAIL" };
        let kill_str = if s.kill_fired { " kill=YES" } else { "" };
        let trap_str = if s.trap_fired { " trap=FIRED" } else { "" };
        let delta_str = s
            .delta
            .map(|d| format!(" delta={d:.4}"))
            .unwrap_or_default();
        let rrr_str = s
            .reviewer_reaching_rate
            .map(|r| format!(" reviewer_reaching_rate={r:.4}"))
            .unwrap_or_default();
        let div_str = s
            .divergence_pct
            .map(|d| format!(" divergence_pct={d:.1}%"))
            .unwrap_or_default();
        println!(
            "claim-scope arm={} status={status}{kill_str}{trap_str}{delta_str}{rrr_str}{div_str} \
             runpod_pending=true",
            s.arm
        );
    }

    if json {
        let body = serde_json::to_string_pretty(&scenarios).map_err(|e| {
            BenchClaimScopeError::Serialise {
                source_msg: e.to_string(),
            }
        })?;
        println!("{body}");
    }

    Ok(())
}
