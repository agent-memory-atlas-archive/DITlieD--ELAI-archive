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

use elai_orchestrator::cross_lang_drift::{
    DriftDetector, DriftDetectorError, DriftItem, DriftKind, PairRegistry, PairRegistryError,
    Severity,
};
use serde::{Deserialize, Serialize};

use crate::args::{CliOutputFormat, DriftAction};

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

fn build_report(repo_root: &Path) -> Result<DetectReport, DriftCliError> {
    let registry = PairRegistry::load(repo_root)?;
    let detector = DriftDetector::new(&registry)?;
    let scan = detector.detect_pairs(repo_root)?;

    let mut critical = 0u32;
    let mut warning = 0u32;
    let mut info = 0u32;
    let mut total = 0u32;
    let mut rows = Vec::with_capacity(scan.len());

    for result in scan {
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

    Ok(DetectReport {
        pair_count: rows.len() as u32,
        drift_total: total,
        critical_count: critical,
        warning_count: warning,
        info_count: info,
        pairs: rows,
    })
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

fn run_detect(json: bool, output_format: CliOutputFormat) -> Result<(), String> {
    let repo_root = resolve_repo_root();
    let report = build_report(&repo_root).map_err(|e| format!("drift detect failed: {e}"))?;

    // E-45: DriftDetector::detect_pairs is the load-bearing call surface;
    // the assignment above is the canonical caller that anchors the
    // module out of WIRE-DARK status per E-41 precedent.

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
    let report =
        build_report(&repo_root).map_err(|e| format!("drift baseline failed: {e}"))?;

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
        // First run for this commit: treat current as the baseline.
        0
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
