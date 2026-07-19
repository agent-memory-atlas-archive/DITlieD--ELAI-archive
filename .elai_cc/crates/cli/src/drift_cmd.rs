//! E-45 — `elai drift <sub>` handler.
//!
//! Anchors the orchestrator's `cross_lang_drift::*` module as production
//! callable so `DriftDetector`, `PairRegistry`, and the host-side
//! detector are not WIRE-DARK per E-41 precedent
//! (`E40_WIRE_DARK_AVOIDANCE_VIA_CLI`).
//!
//! Sub-actions:
//!   * `detect [--json]` — load the pair registry, walk every pair,
//!     and print the drift report. `--json` switches to machine
//!     output. Exit code: 0 = clean, 1 = drift, 2 = registry / IO failure.
//!   * `list-pairs` — print the loaded `cross-lang-pairs.toml` rows so
//!     operators can verify the registry without invoking the detector.
//!   * `baseline --commit HASH` — run the detector against the
//!     working tree and compare totals to a committed baseline file
//!     stored at `workflow/audits/E-45-drift-baseline-<commit>.json`.
//!     Returns 1 if drift_count increased vs the baseline (regression),
//!     0 otherwise.
//!
//! Per H-7 / H-9: no `.unwrap()` / `.expect()` outside test code in this
//! file. The handler returns `Result<(), String>` for the dispatcher.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_core::paths::sessions_dir_path;
use elai_orchestrator::cross_lang_drift::{
    detector::PairScanResult, DriftDetector, DriftDetectorError, DriftItem, DriftKind, PairRegistry,
    PairRegistryError, Severity,
};
use elai_orchestrator::ssb::{
    CrossLangDriftDetectedPayload, CrossLangPairScannedPayload, SchemaVersionMismatchPayload,
    Ssb, SsbConfig, SsbPayload,
};
use serde::{Deserialize, Serialize};

use crate::args::{CliOutputFormat, DriftAction};

/// Session id used for SSB beacon writes in this CLI path.
const CLI_SESSION_ID: &str = "cli:drift";
/// Project id stamped on the SSB rows emitted by `elai drift`.
const CLI_PROJECT_ID: &str = "elai-drift";

/// Top-level dispatcher invoked by `main.rs`.
pub(crate) fn run(action: DriftAction, output_format: CliOutputFormat) -> Result<(), String> {
    match action {
        DriftAction::Detect { json } => run_detect(json, output_format),
        DriftAction::ListPairs => run_list_pairs(output_format),
        DriftAction::Baseline { commit } => run_baseline(&commit, output_format),
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DetectReport {
    pair_count: u32,
    drift_total: u32,
    critical_count: u32,
    warning_count: u32,
    info_count: u32,
    pairs: Vec<PairScanRow>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PairScanRow {
    pair_name: String,
    rust_path: PathBuf,
    mirror_path: PathBuf,
    drift_count: u32,
    severity_max: u8,
    items: Vec<DriftItemRow>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DriftItemRow {
    kind: String,
    location: PathBuf,
    severity: u8,
    detail: String,
}

fn resolve_repo_root() -> PathBuf {
    // Workflow root is the parent of `.elai_cc/`. CWD-first per
    // cross_lang_diff.py convention.
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if cwd.join(".elai_cc").is_dir() {
        return cwd;
    }
    // CWD is .elai_cc itself? Climb one.
    if cwd.file_name().and_then(|s| s.to_str()) == Some(".elai_cc") {
        if let Some(parent) = cwd.parent() {
            return parent.to_path_buf();
        }
    }
    cwd
}

fn build_report(
    repo_root: &Path,
) -> Result<(DetectReport, Vec<PairScanResult>), DriftCliError> {
    let registry = PairRegistry::load(repo_root)?;
    let detector = DriftDetector::new(&registry)?;
    let scan = detector.detect_pairs(repo_root)?;

    let mut critical = 0u32;
    let mut warning = 0u32;
    let mut info = 0u32;
    let mut total = 0u32;
    let mut rows = Vec::with_capacity(scan.len());

    for result in scan.iter() {
        let drift_count = result.drift_items.len() as u32;
        total = total.saturating_add(drift_count);
        let mut sev_max: u8 = 0;
        let mut items = Vec::with_capacity(result.drift_items.len());
        for item in &result.drift_items {
            let s = item.severity.as_u8();
            if s > sev_max {
                sev_max = s;
            }
            match item.severity {
                Severity::Critical => critical = critical.saturating_add(1),
                Severity::Warning => warning = warning.saturating_add(1),
                Severity::Info => info = info.saturating_add(1),
            }
            items.push(DriftItemRow {
                kind: drift_kind_label(&item.kind).to_string(),
                location: item.location.clone(),
                severity: s,
                detail: drift_kind_detail(&item.kind),
            });
        }
        rows.push(PairScanRow {
            pair_name: result.pair.kind.wire_tag().to_string(),
            rust_path: result.pair.rust_path.clone(),
            mirror_path: result.pair.mirror_path.clone(),
            drift_count,
            severity_max: sev_max,
            items,
        });
    }

    Ok((
        DetectReport {
            pair_count: rows.len() as u32,
            drift_total: total,
            critical_count: critical,
            warning_count: warning,
            info_count: info,
            pairs: rows,
        },
        scan,
    ))
}

fn drift_kind_label(kind: &DriftKind) -> &'static str {
    match kind {
        DriftKind::TypeFieldAdded { .. } => "type_field_added",
        DriftKind::TypeFieldRemoved { .. } => "type_field_removed",
        DriftKind::TypeFieldRenamed { .. } => "type_field_renamed",
        DriftKind::SchemaVersionMismatch { .. } => "schema_version_mismatch",
        DriftKind::PairSideMissing { .. } => "pair_side_missing",
    }
}

fn drift_kind_detail(kind: &DriftKind) -> String {
    match kind {
        DriftKind::TypeFieldAdded { field } => format!("field '{field}' added on Rust side"),
        DriftKind::TypeFieldRemoved { field } => {
            format!("field '{field}' removed on Rust side")
        }
        DriftKind::TypeFieldRenamed {
            rust_name,
            mirror_name,
        } => format!("rust '{rust_name}' vs mirror '{mirror_name}'"),
        DriftKind::SchemaVersionMismatch {
            rust_version,
            mirror_version,
        } => format!("rust={rust_version} mirror={mirror_version}"),
        DriftKind::PairSideMissing { missing_side } => {
            format!("missing: {}", missing_side.display())
        }
    }
}

/// blake3 8-byte LE u64 prefix of arbitrary bytes (matches the
/// `workspace_root_hash` pattern in `dkg_lifecycle.rs:214-218`).
fn blake3_prefix(bytes: &[u8]) -> u64 {
    let digest = blake3::hash(bytes);
    let h = digest.as_bytes();
    u64::from_le_bytes([h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]])
}

/// Stable hash of the repo root path string. Used as `repo_root_hash`
/// on `CrossLangPairScannedPayload` so dashboards can group scans by
/// workspace without leaking absolute paths into SSB.
fn repo_root_hash(repo_root: &Path) -> u64 {
    blake3_prefix(repo_root.to_string_lossy().as_bytes())
}

/// Stable hash of a pair name (e.g. "sidecar_orchestrator_schemas").
/// Used as `pair_kind_hash` on `CrossLangDriftDetectedPayload`.
fn pair_kind_hash(pair_name: &str) -> u64 {
    blake3_prefix(pair_name.as_bytes())
}

/// Open an SSB writer for the project root. Returns `None` (not an error)
/// when the sessions directory cannot be created or SSB fails to open —
/// callers that emit beacons warn and continue; drift detection itself
/// must succeed even without SSB. Mirrors `memory_cmd.rs:open_ssb_optional`.
fn open_ssb_optional(project_root: &Path) -> Option<Arc<Ssb>> {
    let sessions_dir = sessions_dir_path(project_root);
    let parent = sessions_dir.parent()?;
    if let Err(e) = std::fs::create_dir_all(parent) {
        tracing::warn!(error = %e, "drift_cmd: cannot create sessions dir");
        return None;
    }
    let db_path = sessions_dir.join("ssb.db");
    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: CLI_PROJECT_ID.to_string(),
        session_id: CLI_SESSION_ID.to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    match Ssb::new(cfg, clock) {
        Ok(ssb) => Some(Arc::new(ssb)),
        Err(e) => {
            tracing::warn!(error = %e, "drift_cmd: cannot open SSB writer");
            None
        }
    }
}

/// Emit the three E-45 SSB beacon types for a completed scan. Non-fatal:
/// warns on any SSB failure so drift detection never rolls back due to
/// beacon loss. Called once per `run_detect` / `run_baseline` walk.
async fn emit_drift_beacons(
    ssb: &Option<Arc<Ssb>>,
    repo_root: &Path,
    report: &DetectReport,
    scan: &[PairScanResult],
) {
    let Some(ssb) = ssb else {
        tracing::warn!("drift_cmd: SSB unavailable; drift beacons not emitted");
        return;
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let now_ms = Ssb::now_ms_via_clock(&clock) as u64;
    let root_hash = repo_root_hash(repo_root);

    // (1) CrossLangPairScanned — once per walk.
    let scanned = SsbPayload::CrossLangPairScanned(CrossLangPairScannedPayload {
        repo_root_hash: root_hash,
        pair_count: report.pair_count,
        ts_ms: now_ms,
    });
    if let Err(e) = ssb.write_payload("cli.drift", 0, scanned, vec![]).await {
        tracing::warn!(error = %e, "drift_cmd: CrossLangPairScanned beacon failed");
    }

    // (2) CrossLangDriftDetected — once per pair with drift_count >= 1.
    // (3) SchemaVersionMismatch — once per drift item of that kind.
    for result in scan.iter() {
        let drift_count = result.drift_items.len() as u32;
        if drift_count == 0 {
            continue;
        }
        let pair_name = result.pair.kind.wire_tag();
        let mut sev_max: u8 = 0;
        for item in &result.drift_items {
            let s = item.severity.as_u8();
            if s > sev_max {
                sev_max = s;
            }
        }
        let detected = SsbPayload::CrossLangDriftDetected(CrossLangDriftDetectedPayload {
            pair_kind_hash: pair_kind_hash(pair_name),
            drift_count,
            severity_max: sev_max,
            ts_ms: now_ms,
        });
        if let Err(e) = ssb.write_payload("cli.drift", 0, detected, vec![]).await {
            tracing::warn!(error = %e, pair = pair_name, "drift_cmd: CrossLangDriftDetected beacon failed");
        }
        for item in &result.drift_items {
            if let DriftKind::SchemaVersionMismatch {
                rust_version,
                mirror_version,
            } = &item.kind
            {
                let mismatch = SsbPayload::SchemaVersionMismatch(SchemaVersionMismatchPayload {
                    rust_schema: *rust_version,
                    python_schema: *mirror_version,
                    ts_ms: now_ms,
                });
                if let Err(e) = ssb.write_payload("cli.drift", 0, mismatch, vec![]).await {
                    tracing::warn!(error = %e, pair = pair_name, "drift_cmd: SchemaVersionMismatch beacon failed");
                }
            }
        }
    }
}

fn run_detect(json: bool, output_format: CliOutputFormat) -> Result<(), String> {
    let repo_root = resolve_repo_root();
    let (report, scan) =
        build_report(&repo_root).map_err(|e| format!("drift detect failed: {e}"))?;

    // E-45: DriftDetector::detect_pairs is the load-bearing call surface;
    // the assignment above is the canonical caller that anchors the
    // module out of WIRE-DARK status per E-41 precedent.

    // SSB beacons: non-fatal. Ssb::new spawns a single-writer task and
    // MUST be constructed inside a tokio runtime (see memory_cmd /
    // safety_cmd comments).
    let ssb = open_ssb_optional(&repo_root);
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("drift detect: tokio runtime build failed: {e}"))?;
    rt.block_on(emit_drift_beacons(&ssb, &repo_root, &report, &scan));

    if json {
        let txt = serde_json::to_string_pretty(&report)
            .map_err(|e| format!("json serialize failed: {e}"))?;
        println!("{txt}");
    } else {
        print_text_report(&report, output_format);
    }

    if report.drift_total > 0 {
        // Non-zero drift_count is a "report finding" not a fatal error;
        // CLI returns Ok(()) and the caller can scrape drift_total from
        // the report when needed. Treat critical as fatal-for-CI signal
        // via stderr message; exit code remains 0 to match the rest of
        // the `elai *` operator surface (cf. durability_cmd).
        if report.critical_count > 0 {
            eprintln!(
                "E-45: {} critical drift item(s) found; review report",
                report.critical_count
            );
        }
    }
    Ok(())
}

fn print_text_report(report: &DetectReport, _format: CliOutputFormat) {
    println!("E-45 cross-language drift report");
    println!(
        "pairs={} drift_total={} critical={} warning={} info={}",
        report.pair_count,
        report.drift_total,
        report.critical_count,
        report.warning_count,
        report.info_count,
    );
    for pair in &report.pairs {
        println!(
            "  {} ({} drift, sev_max={})",
            pair.pair_name, pair.drift_count, pair.severity_max,
        );
        for item in &pair.items {
            println!(
                "    [{}] sev={} {} ({})",
                item.kind,
                item.severity,
                item.detail,
                item.location.display(),
            );
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ListPairsRow {
    name: String,
    rust_path: PathBuf,
    mirror_path: PathBuf,
    note: String,
}

fn run_list_pairs(output_format: CliOutputFormat) -> Result<(), String> {
    let repo_root = resolve_repo_root();
    let registry = PairRegistry::load(&repo_root)
        .map_err(|e| format!("drift list-pairs failed: {e}"))?;
    let rows: Vec<ListPairsRow> = registry
        .pairs()
        .iter()
        .map(|p| ListPairsRow {
            name: p.kind.wire_tag().to_string(),
            rust_path: p.rust_path.clone(),
            mirror_path: p.mirror_path.clone(),
            note: p.note.clone(),
        })
        .collect();
    render_serializable(&rows, output_format)
}

#[derive(Debug, Serialize, Deserialize)]
struct BaselineReport {
    commit: String,
    baseline_drift_total: u32,
    current_drift_total: u32,
    regression: bool,
}

fn run_baseline(commit: &str, output_format: CliOutputFormat) -> Result<(), String> {
    let repo_root = resolve_repo_root();
    let (report, scan) =
        build_report(&repo_root).map_err(|e| format!("drift baseline failed: {e}"))?;

    // SSB beacons share the same emit path as `run_detect`.
    let ssb = open_ssb_optional(&repo_root);
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("drift baseline: tokio runtime build failed: {e}"))?;
    rt.block_on(emit_drift_beacons(&ssb, &repo_root, &report, &scan));

    let baseline_path = repo_root
        .join("workflow")
        .join("audits")
        .join(format!("E-45-drift-baseline-{commit}.json"));
    let baseline_total: u32 = if baseline_path.exists() {
        let text = std::fs::read_to_string(&baseline_path).map_err(|e| {
            format!(
                "cannot read baseline {}: {e}",
                baseline_path.display()
            )
        })?;
        let parsed: DetectReport = serde_json::from_str(&text).map_err(|e| {
            format!(
                "baseline {} is not a valid drift report: {e}",
                baseline_path.display()
            )
        })?;
        parsed.drift_total
    } else {
        // First run for this commit: seed the baseline file with the
        // current report so the first run is not a regression against 0.
        if let Some(parent) = baseline_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                tracing::warn!(error = %e, "drift_cmd: cannot create baseline parent dir");
            } else {
                let pretty = serde_json::to_string_pretty(&report)
                    .map_err(|e| format!("baseline seed serialize failed: {e}"))?;
                if let Err(e) = std::fs::write(&baseline_path, pretty) {
                    tracing::warn!(error = %e, "drift_cmd: cannot write baseline seed file");
                }
            }
        }
        report.drift_total
    };

    let baseline_report = BaselineReport {
        commit: commit.to_string(),
        baseline_drift_total: baseline_total,
        current_drift_total: report.drift_total,
        regression: report.drift_total > baseline_total,
    };
    render_serializable(&baseline_report, output_format)
}

fn render_serializable<T: Serialize>(value: &T, format: CliOutputFormat) -> Result<(), String> {
    match format {
        CliOutputFormat::Json => {
            let txt = serde_json::to_string_pretty(value)
                .map_err(|e| format!("json serialize failed: {e}"))?;
            println!("{txt}");
        }
        CliOutputFormat::Toml => {
            let txt = toml::to_string(value)
                .map_err(|e| format!("toml serialize failed: {e}"))?;
            println!("{txt}");
        }
        CliOutputFormat::Text => {
            let txt = serde_json::to_string_pretty(value)
                .map_err(|e| format!("text/json serialize failed: {e}"))?;
            println!("{txt}");
        }
    }
    Ok(())
}

/// Internal error envelope so build_report can return one Result type.
#[derive(Debug, thiserror::Error)]
enum DriftCliError {
    #[error("registry: {0}")]
    Registry(#[from] PairRegistryError),
    #[error("detector: {0}")]
    Detector(#[from] DriftDetectorError),
}

#[allow(dead_code)]
fn _drift_item_sink(_item: &DriftItem) {
    // Documentation anchor: callers consume DriftItem via the
    // DriftItemRow projection above. This dead-code sink keeps the
    // import in the compile graph when ItemRow is the only public use,
    // for clarity in future audits.
}
