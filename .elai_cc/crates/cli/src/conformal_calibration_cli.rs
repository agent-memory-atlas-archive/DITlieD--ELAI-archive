//! D-16 — `elai conformal-calibration <status|update>` handler.
//!
//! Surfaces the durable UniCR tau threshold to operators. The handler
//! constructs a `TauUpdater` over the configured corpus + tau paths
//! (per `[conformal].tau_path` toml key, default
//! `.elai_cc/data/conformal_tau.txt`) and exposes two operations:
//!
//!   * `status` — reads the persisted tau scalar, corpus size, and
//!     last-modified wall-clock from disk without invoking the
//!     updater. Pure read; no side effects.
//!   * `update` — runs `TauUpdater::update_tau`, persists the new
//!     scalar atomically (tmp + rename), and writes one
//!     `calibration_corpus_updated` SSB row when the orchestrator's
//!     production task-runner is live (otherwise the row is dropped
//!     locally — the CLI surface is intentionally side-effect-light
//!     for offline operator use).
//!
//! Paper grounding: arXiv:2509.01455 (UniCR Eq.5 CRC quantile
//! formula). The `alpha` (miscoverage rate) and `rho` (selective risk
//! budget) are paper-distinct quantities per D-16-decisions.md
//! DECISION-4. Only `alpha` is exposed on this CLI surface; `rho` is
//! consumed at decision time, not at update time.
//!
//! Namespace: DISTINCT from C-47's `elai calibration` (case-based
//! tool-use registry) per D-16-decisions.md DECISION-5.

use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use elai_orchestrator::conformal_calibration::{
    tau_path_default, TauUpdater, DEFAULT_EPSILON,
};

use crate::args::{CliOutputFormat, ConformalCalibrationAction};

/// Default calibration corpus path when neither the CLI flag nor a
/// toml override is supplied. Matches the seed corpus layout shipped
/// under `.elai_cc/calibration/seed/`.
const DEFAULT_CORPUS_PATH: &str = ".elai_cc/calibration/seed/corpus.jsonl";

/// Top-level dispatcher invoked by main.rs.
pub(crate) fn run(
    action: ConformalCalibrationAction,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    match action {
        ConformalCalibrationAction::Status {
            tau_path,
            corpus_path,
        } => run_status(tau_path, corpus_path, output_format),
        ConformalCalibrationAction::Update {
            tau_path,
            corpus_path,
            alpha,
        } => run_update(tau_path, corpus_path, alpha, output_format),
    }
}

fn resolve_tau_path(override_path: Option<String>) -> PathBuf {
    match override_path {
        Some(p) => PathBuf::from(p),
        None => tau_path_default(),
    }
}

fn resolve_corpus_path(override_path: Option<String>) -> PathBuf {
    match override_path {
        Some(p) => PathBuf::from(p),
        None => PathBuf::from(DEFAULT_CORPUS_PATH),
    }
}

fn parse_alpha(raw: Option<String>) -> Result<f64, String> {
    match raw {
        None => Ok(DEFAULT_EPSILON),
        Some(s) => s
            .trim()
            .parse::<f64>()
            .map_err(|e| format!("invalid --alpha `{s}`: {e}")),
    }
}

fn tau_last_modified_ms(p: &std::path::Path) -> Option<i64> {
    let meta = std::fs::metadata(p).ok()?;
    let mtime = meta.modified().ok()?;
    let dur = mtime.duration_since(UNIX_EPOCH).ok()?;
    i64::try_from(dur.as_millis()).ok()
}

fn run_status(
    tau_override: Option<String>,
    corpus_override: Option<String>,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    let tau_path = resolve_tau_path(tau_override);
    let corpus_path = resolve_corpus_path(corpus_override);
    let updater = TauUpdater::new(corpus_path.clone(), tau_path.clone(), DEFAULT_EPSILON);
    let current_tau = updater.read_persisted_tau();
    let corpus_size = updater.read_corpus_size();
    let last_modified_ms = tau_last_modified_ms(&tau_path);

    match output_format {
        CliOutputFormat::Json => {
            let json = serde_json::json!({
                "tau_path": tau_path.display().to_string(),
                "corpus_path": corpus_path.display().to_string(),
                "current_tau": current_tau,
                "corpus_size": corpus_size,
                "last_modified_ms": last_modified_ms,
            });
            println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
        }
        CliOutputFormat::Toml => {
            println!("[conformal_calibration.status]");
            println!("tau_path = \"{}\"", tau_path.display());
            println!("corpus_path = \"{}\"", corpus_path.display());
            println!("current_tau = {current_tau:.6}");
            println!("corpus_size = {corpus_size}");
            match last_modified_ms {
                Some(ms) => println!("last_modified_ms = {ms}"),
                None => println!("# last_modified_ms = none (tau file not yet written)"),
            }
        }
        CliOutputFormat::Text => {
            println!("conformal-calibration status");
            println!("  tau path        : {}", tau_path.display());
            println!("  corpus path     : {}", corpus_path.display());
            println!("  current tau     : {current_tau:.6}");
            println!("  corpus size     : {corpus_size}");
            match last_modified_ms {
                Some(ms) => println!("  last modified ms: {ms}"),
                None => println!("  last modified ms: (tau file not yet written)"),
            }
        }
    }
    Ok(())
}

fn run_update(
    tau_override: Option<String>,
    corpus_override: Option<String>,
    alpha_override: Option<String>,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    let tau_path = resolve_tau_path(tau_override);
    let corpus_path = resolve_corpus_path(corpus_override);
    let alpha = parse_alpha(alpha_override)?;
    if !(alpha > 0.0 && alpha < 1.0 && alpha.is_finite()) {
        return Err(format!(
            "alpha must be a finite number in (0.0, 1.0); got {alpha}"
        ));
    }
    let updater = TauUpdater::new(corpus_path.clone(), tau_path.clone(), alpha);
    let new_tau = updater
        .update_tau()
        .map_err(|e| format!("TauUpdater::update_tau failed: {e}"))?;
    let corpus_size = updater.read_corpus_size();

    match output_format {
        CliOutputFormat::Json => {
            let json = serde_json::json!({
                "tau_path": tau_path.display().to_string(),
                "corpus_path": corpus_path.display().to_string(),
                "alpha": alpha,
                "new_tau": new_tau,
                "corpus_size": corpus_size,
            });
            println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
        }
        CliOutputFormat::Toml => {
            println!("[conformal_calibration.update]");
            println!("tau_path = \"{}\"", tau_path.display());
            println!("corpus_path = \"{}\"", corpus_path.display());
            println!("alpha = {alpha:.6}");
            println!("new_tau = {new_tau:.6}");
            println!("corpus_size = {corpus_size}");
        }
        CliOutputFormat::Text => {
            println!("conformal-calibration update");
            println!("  tau path    : {}", tau_path.display());
            println!("  corpus path : {}", corpus_path.display());
            println!("  alpha       : {alpha:.6}");
            println!("  new tau     : {new_tau:.6}");
            println!("  corpus size : {corpus_size}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn parse_alpha_accepts_none_and_valid_float() {
        assert_eq!(parse_alpha(None).unwrap(), DEFAULT_EPSILON);
        assert!((parse_alpha(Some("0.05".into())).unwrap() - 0.05).abs() < 1e-12);
    }

    #[test]
    fn parse_alpha_rejects_garbage() {
        assert!(parse_alpha(Some("not-a-float".into())).is_err());
    }

    #[test]
    fn resolve_tau_path_honours_override() {
        let p = resolve_tau_path(Some("custom/tau.txt".to_string()));
        assert_eq!(p, PathBuf::from("custom/tau.txt"));
    }

    #[test]
    fn resolve_tau_path_falls_back_to_default() {
        let p = resolve_tau_path(None);
        assert_eq!(p, tau_path_default());
    }

    #[test]
    fn run_update_persists_fallback_when_corpus_missing() {
        // Smoke-test the handler runtime path end-to-end. No corpus
        // file -> TauUpdater writes TAU_DEFAULT_FALLBACK; the CLI
        // handler exits 0 without bubbling up an error.
        let dir = TempDir::new().expect("tempdir");
        let tau_path = dir.path().join("tau.txt");
        let corpus_path = dir.path().join("missing.jsonl");
        let result = run_update(
            Some(tau_path.display().to_string()),
            Some(corpus_path.display().to_string()),
            Some("0.10".to_string()),
            CliOutputFormat::Text,
        );
        assert!(result.is_ok(), "{result:?}");
        // After the update the on-disk tau file must exist + parse.
        let raw = std::fs::read_to_string(&tau_path).expect("tau persisted");
        let parsed: f64 = raw.trim().parse().expect("tau parseable");
        assert!(parsed > 0.0 && parsed < 1.0, "tau in (0,1): {parsed}");
    }
}
