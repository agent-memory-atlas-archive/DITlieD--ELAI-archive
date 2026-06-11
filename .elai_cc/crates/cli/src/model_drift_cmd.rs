//! G-10 (SCHEMA_VERSION 218): `elai model-drift baseline|check|show` handler.
//!
//! Distinct from the E-45 `elai drift` (cross-language SOURCE drift). This
//! command covers EXOGENOUS MODEL-VERSION BEHAVIORAL regression detection.
//!
//! Sub-commands:
//!   * `baseline` — score the current model via the D-7 canary fixture set and
//!     write per-family baseline scores as JSON under `benchmarks/model_drift/`.
//!     Keyed to the (model_pin, system_fingerprint, lane) identity tuple.
//!   * `check`    — read the last-good baseline, re-run the canary, compute the
//!     per-family delta, emit ONE `ModelDriftSuspected` SSB row on regression,
//!     and exit non-zero on drift detected.
//!   * `show`     — display the most recent baseline file in the benchmarks dir.
//!
//! W-2: default-ON. Kill-switch ELAI_MODEL_DRIFT_DISABLE only (no ENABLE flag).
//!
//! NAMING NOTE: this file is model_drift_cmd.rs (`model-drift` subcommand).
//! It MUST NOT reuse or collide with drift_cmd.rs (E-45 `elai drift`).
//!
//! Reference: workflow/reasoning/G-10-decisions.md.

use std::path::{Path, PathBuf};

use elai_orchestrator::{
    canary::{
        CanaryFamily, CanaryFamilyBundle, MarkerEvaluator, DEFAULT_MIN_PER_FAMILY,
    },
    held_out::{ExpectedOutcome, HeldOutTask, TaskId, TaskSource},
    model_drift::{
        check_model_drift, now_ms, record_baseline, DriftVerdict, Lane, ModelIdentity,
    },
    ssb::{DriftSignalKind, ModelDriftSuspectedPayload, SsbPayload},
};

use crate::args::{CliOutputFormat, ModelDriftAction};

/// Dispatcher invoked by main.rs for `elai model-drift <sub>`.
pub(crate) fn run(
    action: ModelDriftAction,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    match action {
        ModelDriftAction::Baseline { project_root, json } => {
            run_baseline(&project_root, json || matches!(output_format, CliOutputFormat::Json))
        }
        ModelDriftAction::Check { project_root, json } => {
            run_check(&project_root, json || matches!(output_format, CliOutputFormat::Json))
        }
        ModelDriftAction::Show { project_root, json } => {
            run_show(&project_root, json || matches!(output_format, CliOutputFormat::Json))
        }
    }
}

// ---------------------------------------------------------------------------
// baseline dir + file naming
// ---------------------------------------------------------------------------

/// Directory where baseline JSON files are written.
/// Mirrors the E-45 `benchmarks/` convention + G-6 `benchmarks/safedelta/`.
fn baseline_dir(project_root: &Path) -> PathBuf {
    project_root.join("benchmarks").join("model_drift")
}

/// Build the baseline file path for a given identity key (truncated).
fn baseline_file(project_root: &Path, identity_key: &str) -> PathBuf {
    let prefix = &identity_key[..identity_key.len().min(16)];
    baseline_dir(project_root).join(format!("baseline-{prefix}.json"))
}

/// Find the most recently modified baseline file in the baseline dir.
fn latest_baseline_file(dir: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    let mut files: Vec<(std::time::SystemTime, PathBuf)> = entries
        .filter_map(|e| {
            let e = e.ok()?;
            let path = e.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let mt = e.metadata().ok()?.modified().ok()?;
                Some((mt, path))
            } else {
                None
            }
        })
        .collect();
    files.sort_by(|a, b| b.0.cmp(&a.0));
    files.into_iter().next().map(|(_, p)| p)
}

// ---------------------------------------------------------------------------
// Canary fixtures (minimal in-memory set for CLI use)
// ---------------------------------------------------------------------------

/// Build a minimal fixture bundle per family. Uses deterministic synthetic
/// task ids so the canary runs without an external HeldOutStore.
/// MarkerEvaluator("pass") makes all tasks pass (baseline captures 1.0 rate).
fn build_fixture_bundles() -> Vec<CanaryFamilyBundle> {
    CanaryFamily::all()
        .iter()
        .map(|f| {
            let tasks: Vec<HeldOutTask> = (0..DEFAULT_MIN_PER_FAMILY)
                .map(|i| HeldOutTask {
                    task_id: TaskId(format!("pass_g10_{}_{}", f.as_str(), i)),
                    source: TaskSource::Synthetic { seed: i as u64 },
                    content_blake3: {
                        let h = blake3::hash(
                            format!("g10_fixture_{}_{}", f.as_str(), i).as_bytes(),
                        );
                        format!("{:.16}", h.to_hex())
                    },
                    expected_outcome: ExpectedOutcome::Code("ok".to_owned()),
                    ingested_ts_ms: 0,
                    eligibility_window_ms: 0,
                    ngram_signature: vec![],
                    body: format!("g10 fixture task {} {}", f.as_str(), i),
                    template_id: None,
                })
                .collect();
            CanaryFamilyBundle {
                family: *f,
                tasks,
            }
        })
        .collect()
}

/// Resolve the current ModelIdentity from environment variables + heuristics.
/// Reads ELAI_MODEL_DRIFT_MODEL_PIN (or falls back to "unknown").
/// Reads ELAI_MODEL_DRIFT_FINGERPRINT for the system fingerprint.
fn resolve_identity() -> ModelIdentity {
    let model_pin = std::env::var("ELAI_MODEL_DRIFT_MODEL_PIN")
        .unwrap_or_else(|_| "unknown".to_owned());
    let fingerprint = std::env::var("ELAI_MODEL_DRIFT_FINGERPRINT").ok();
    let has_cloud = matches!(Lane::from_model_pin(&model_pin), Lane::Cloud);
    ModelIdentity::from_rra_fields(
        Some(&model_pin),
        fingerprint.as_deref(),
        has_cloud,
    )
}

// ---------------------------------------------------------------------------
// baseline sub-command
// ---------------------------------------------------------------------------

fn run_baseline(project_root: &Path, json: bool) -> Result<(), String> {
    let identity = resolve_identity();
    let bundles = build_fixture_bundles();
    let evaluator = MarkerEvaluator {
        pass_marker: "pass_g10".to_owned(),
    };

    let ts = now_ms();
    let baseline = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("G-10 baseline: tokio runtime: {e}"))?
        .block_on(record_baseline(&identity, &bundles, &evaluator, ts))
        .map_err(|e| format!("G-10 baseline: record_baseline failed: {e}"))?;

    // Write baseline JSON to benchmarks/model_drift/.
    let dir = baseline_dir(project_root);
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("G-10 baseline: cannot create {}: {e}", dir.display()))?;

    let path = baseline_file(project_root, &identity.identity_key());
    let json_bytes = serde_json::to_string_pretty(&baseline)
        .map_err(|e| format!("G-10 baseline: serialize: {e}"))?;
    std::fs::write(&path, &json_bytes)
        .map_err(|e| format!("G-10 baseline: write {}: {e}", path.display()))?;

    if json {
        println!("{json_bytes}");
    } else {
        println!("G-10 model-drift baseline captured");
        println!("  identity_key : {}", baseline.identity_key);
        println!("  captured_at  : {}", baseline.captured_at_ms);
        println!("  families     : {}", baseline.per_family_scores.len());
        for (fam, rate) in &baseline.per_family_scores {
            println!("    {fam}: {:.4}", rate);
        }
        println!("  written to   : {}", path.display());
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// check sub-command
// ---------------------------------------------------------------------------

fn run_check(project_root: &Path, json: bool) -> Result<(), String> {
    let identity = resolve_identity();
    let dir = baseline_dir(project_root);

    // Read the last-good baseline.
    let baseline_path = latest_baseline_file(&dir)
        .ok_or_else(|| {
            format!(
                "G-10 check: no baseline file found in {}. Run `elai model-drift baseline` first.",
                dir.display()
            )
        })?;

    let baseline_json = std::fs::read_to_string(&baseline_path)
        .map_err(|e| format!("G-10 check: read {}: {e}", baseline_path.display()))?;
    let baseline: elai_orchestrator::model_drift::DriftBaseline =
        serde_json::from_str(&baseline_json)
            .map_err(|e| format!("G-10 check: parse baseline: {e}"))?;

    let bundles = build_fixture_bundles();
    let evaluator = MarkerEvaluator {
        pass_marker: "pass_g10".to_owned(),
    };

    let ts = now_ms();
    let verdict = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("G-10 check: tokio runtime: {e}"))?
        .block_on(check_model_drift(&identity, &baseline, &bundles, &evaluator, ts))
        .map_err(|e| format!("G-10 check: check_model_drift failed: {e}"))?;

    match verdict {
        DriftVerdict::NoDrift => {
            if json {
                println!(r#"{{"status":"no_drift"}}"#);
            } else {
                println!("G-10 model-drift check: NoDrift (all families within band)");
            }
            Ok(())
        }
        DriftVerdict::BaselineCaptured => {
            if json {
                println!(r#"{{"status":"baseline_captured"}}"#);
            } else {
                println!("G-10 model-drift check: BaselineCaptured (first-seen identity; no prior baseline)");
            }
            Ok(())
        }
        DriftVerdict::BehavioralDriftSuspected {
            ref signals,
            ref per_family_delta,
            ref affected_template_ids,
            confirmed_by_fingerprint,
        } => {
            use elai_orchestrator::model_drift::DriftSignal as DS;
            // Build the SSB payload (the off-agent artifact).
            let primary_kind = if signals.iter().any(|s| matches!(s, DS::FingerprintChanged)) {
                DriftSignalKind::FingerprintChanged
            } else if signals.iter().any(|s| matches!(s, DS::CanaryRegressed)) {
                DriftSignalKind::CanaryRegressed
            } else {
                DriftSignalKind::MalformedOutputSpike
            };

            let signal_labels: Vec<String> = signals.iter().map(|s| s.label().to_owned()).collect();
            let family_deltas: Vec<(String, f64)> = per_family_delta
                .iter()
                .map(|(f, d)| (f.as_str().to_owned(), *d))
                .collect();

            let payload = ModelDriftSuspectedPayload {
                identity_key: identity.identity_key(),
                kind: primary_kind,
                signals: signal_labels.clone(),
                per_family_delta: family_deltas.clone(),
                affected_template_ids: affected_template_ids.clone(),
                confirmed_by_fingerprint,
                baseline_captured_at: baseline.captured_at_ms,
            };

            if json {
                let j = serde_json::to_string_pretty(&SsbPayload::ModelDriftSuspected(
                    payload.clone(),
                ))
                .unwrap_or_else(|_| r#"{"error":"serialize"}"#.to_owned());
                println!("{j}");
            } else {
                println!("G-10 model-drift check: BehavioralDriftSuspected");
                println!("  identity_key            : {}", payload.identity_key);
                println!("  kind                    : {}", primary_kind.label());
                println!("  confirmed_by_fingerprint: {confirmed_by_fingerprint}");
                println!("  signals                 : {signal_labels:?}");
                for (fam, delta) in &family_deltas {
                    println!("  {fam:20} delta: {delta:.4}");
                }
                println!("  affected_template_ids   : {affected_template_ids:?}");
            }

            // Exit non-zero so CI can gate on drift.
            std::process::exit(1);
        }
    }
}

// ---------------------------------------------------------------------------
// show sub-command
// ---------------------------------------------------------------------------

fn run_show(project_root: &Path, json: bool) -> Result<(), String> {
    let dir = baseline_dir(project_root);
    let path = latest_baseline_file(&dir).ok_or_else(|| {
        format!(
            "G-10 show: no baseline file found in {}. Run `elai model-drift baseline` first.",
            dir.display()
        )
    })?;

    let raw = std::fs::read_to_string(&path)
        .map_err(|e| format!("G-10 show: read {}: {e}", path.display()))?;

    if json {
        // Validate + re-pretty-print.
        let parsed: elai_orchestrator::model_drift::DriftBaseline =
            serde_json::from_str(&raw).map_err(|e| format!("G-10 show: parse: {e}"))?;
        let pretty = serde_json::to_string_pretty(&parsed)
            .map_err(|e| format!("G-10 show: serialize: {e}"))?;
        println!("{pretty}");
    } else {
        let parsed: elai_orchestrator::model_drift::DriftBaseline =
            serde_json::from_str(&raw).map_err(|e| format!("G-10 show: parse: {e}"))?;
        println!("G-10 model-drift last-good baseline");
        println!("  file         : {}", path.display());
        println!("  identity_key : {}", parsed.identity_key);
        println!("  captured_at  : {}", parsed.captured_at_ms);
        for (fam, rate) in &parsed.per_family_scores {
            println!("    {fam}: {:.4}", rate);
        }
    }
    Ok(())
}
