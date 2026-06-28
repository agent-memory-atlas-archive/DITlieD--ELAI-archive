//! I-38 STEP 2 — methodology doc generator (focused module).
//!
//! Auto-generates a human-readable Markdown methodology document from the
//! machine-readable `QdeltaConfig`. The doc is deterministic under the same
//! config (GD-2): no timestamps, no RNG, no environment-dependent values in
//! the body. Every section is derived from `QdeltaConfig` fields or the
//! compile-time constants that govern the harness behaviour.
//!
//! Split out of `bench_qdelta.rs` to keep the parent file under the MR-1
//! 1000-line maintainability boundary.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::{
    BenchQdeltaError, BOOTSTRAP_N_RESAMPLES, CI_LOWER_TAIL, CI_UPPER_TAIL, FLOOR_MIN_SEED_COUNT,
    QdeltaConfig,
};

// ---------------------------------------------------------------------------
// Methodology doc generator
// ---------------------------------------------------------------------------

/// Generate a human-readable methodology document from `QdeltaConfig`.
///
/// The document is a structured Markdown report auto-generated from the
/// machine-readable config (STEP 2 of I-38). It is deterministic under the
/// same config (GD-2) — no timestamps, no RNG, no environment-dependent
/// values. Every section is derived from `QdeltaConfig` fields or the
/// compile-time constants that govern the harness behaviour.
///
/// # Sections
///
/// 1. **Framing (GD-3)** — ablation-only, no leaderboard rank. The FIRST
///    section states this explicitly per the WIRING_CONTRACT stub_ban.
/// 2. **Statistical floor** — seed_count + paired invariants and their
///    pass/fail status against the mechanical floor.
/// 3. **Measurement protocol** — QDELTA definition, defective criteria,
///    bootstrap CI parameters.
/// 4. **Contamination probe (D-20)** — the `contamination_probe_result`
///    field value and its meaning.
/// 5. **Cost** — $/task reported alongside accuracy (GD-3 $/task policy).
/// 6. **Reproducibility (GD-2)** — corpus blake3, seed-recomputable CI,
///    runpod_pending honesty.
/// 7. **Limitations** — I-20 stub, fixture-tier scope.
pub fn generate_methodology_doc(config: &QdeltaConfig) -> String {
    let floor_seed_ok = config.seed_count >= FLOOR_MIN_SEED_COUNT;
    let floor_paired_ok = config.paired;
    let floor_status = if floor_seed_ok && floor_paired_ok {
        "PASS"
    } else {
        "FAIL (harness refuses this run)"
    };

    let on_true = config.tasks.iter().filter(|t| t.on_defective).count();
    let off_true = config.tasks.iter().filter(|t| t.off_defective).count();
    let n_tasks = config.tasks.len();
    let on_rate = if n_tasks > 0 {
        on_true as f64 / n_tasks as f64
    } else {
        0.0
    };
    let off_rate = if n_tasks > 0 {
        off_true as f64 / n_tasks as f64
    } else {
        0.0
    };
    let point_delta = off_rate - on_rate;

    format!(
        "# QDELTA Methodology (I-38)\n\
         \n\
         Auto-generated from `QdeltaConfig`. Deterministic under the same config (GD-2).\n\
         \n\
         ## 1. Framing (GD-3)\n\
         \n\
         This is an ablation benchmark measuring gates-ON vs gates-OFF. It is NOT a\n\
         leaderboard rank. SWE-bench Verified framing is explicitly declined (landmine\n\
         #11 per I-0). The headline number is the ON/OFF delta with bootstrap CI, never\n\
         a rank position. no leaderboard rank is claimed or implied.\n\
         \n\
         ## 2. Statistical floor (mechanically enforced)\n\
         \n\
         The harness refuses under-powered runs via `floor_check()` BEFORE any compute\n\
         is spent. The floor is policy, not advisory.\n\
         \n\
         - seed_count: {seed_count} (floor >= {FLOOR_MIN_SEED_COUNT}) — {seed_status}\n\
         - paired: {paired} — {paired_status}\n\
         - floor verdict: {floor_status}\n\
         \n\
         ## 3. Measurement protocol\n\
         \n\
         QDELTA = off_defective_rate - on_defective_rate where:\n\
         - on_defective_rate  = fraction of tasks where gates-ON produced a defective diff\n\
         - off_defective_rate = fraction of tasks where gates-OFF produced a defective diff\n\
         - defective = held-out test failures + post-merge surviving mutants (I-20) + regression count\n\
         - positive QDELTA means gates IMPROVE quality (fewer defective diffs with gates ON)\n\
         \n\
         Point estimates from this config (n_tasks={n_tasks}):\n\
         - on_defective_rate  = {on_true}/{n_tasks} = {on_rate:.4}\n\
         - off_defective_rate = {off_true}/{n_tasks} = {off_rate:.4}\n\
         - point delta        = {point_delta:.4}\n\
         \n\
         Bootstrap 95% CI:\n\
         - n_resamples: {BOOTSTRAP_N_RESAMPLES}\n\
         - ci_level: 95% two-sided (lower={CI_LOWER_TAIL}, upper={CI_UPPER_TAIL})\n\
         - PRNG: xorshift64 (pure std, no dep, seed-recomputable per GD-2)\n\
         \n\
         ## 4. Contamination probe (D-20)\n\
         \n\
         contamination_probe_result: {contamination}\n\
         \n\
         The D-20 memorization probe is surfaced into the methodology as the\n\
         `contamination_probe` field. Allowed values:\n\
         - \"not_run\"              — probe was not executed; contamination unverified\n\
         - \"honestly_evaluated\"   — probe ran and the task cut is clean\n\
         - \"game_with_memorization\" — probe detected memorization; results are suspect\n\
         \n\
         ## 5. Cost\n\
         \n\
         dollar_per_task: {dollar_per_task:.6} USD\n\
         \n\
         Reported alongside accuracy per the GD-3 $/task policy. Every headline number\n\
         in this report carries this cost figure.\n\
         \n\
         ## 6. Reproducibility (GD-2)\n\
         \n\
         - corpus_blake3: computed at run time from the config TOML bytes (content provenance)\n\
         - bootstrap CI is recomputable from the same seed (xorshift64, no external RNG)\n\
         - runpod_pending: true (live-model leg deferred per D-026; fixture tier only)\n\
         \n\
         ## 7. Limitations\n\
         \n\
         - surviving_mutants_per_task: 0.0 (I-20 stub, pending I-20 ship)\n\
         - Fixture tier only; full-corpus QDELTA run is RunPod-pending (D-026)\n\
         - Cross-repo cluster analysis of CI width deferred (needs >=3 repos, GD-4)\n\
         \n\
         ---\n\
         suite: {suite}\n\
         model_id: {model_id}\n",
        suite = config.suite,
        model_id = config.model_id,
        seed_count = config.seed_count,
        seed_status = if floor_seed_ok { "PASS" } else { "FAIL" },
        paired = config.paired,
        paired_status = if floor_paired_ok { "PASS" } else { "FAIL" },
        floor_status = floor_status,
        n_tasks = n_tasks,
        on_true = on_true,
        off_true = off_true,
        on_rate = on_rate,
        off_rate = off_rate,
        point_delta = point_delta,
        contamination = config.contamination_probe_result,
        dollar_per_task = config.dollar_per_task,
    )
}

// ---------------------------------------------------------------------------
// Methodology doc writer (STEP 2 — sidecar .md artifact)
// ---------------------------------------------------------------------------

/// Write the human-readable methodology doc to
/// `<project_root>/benchmarks/qdelta/methodology-<unix_ms>.md`.
///
/// The doc is auto-generated from `QdeltaConfig` via `generate_methodology_doc()`
/// and persisted as a standalone Markdown artifact alongside the JSON report.
/// Deterministic content (GD-2); only the filename timestamp varies.
pub(crate) fn write_methodology_doc(
    project_root: &Path,
    config: &QdeltaConfig,
) -> Result<PathBuf, BenchQdeltaError> {
    let report_dir = project_root.join("benchmarks").join("qdelta");
    fs::create_dir_all(&report_dir).map_err(|e| BenchQdeltaError::ReportDir {
        path: report_dir.display().to_string(),
        source_msg: e.to_string(),
    })?;

    let unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default();
    let methodology_path = report_dir.join(format!("methodology-{unix_ms}.md"));

    let body = generate_methodology_doc(config);
    fs::write(&methodology_path, &body).map_err(|e| BenchQdeltaError::OutputIo {
        path: methodology_path.display().to_string(),
        source_msg: e.to_string(),
    })?;
    Ok(methodology_path)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench_qdelta::tests::{
        parse_fixture, QDELTA_FIXTURE_UNDERPOWER, QDELTA_FIXTURE_VALID,
    };

    #[test]
    fn methodology_doc_has_structured_sections() {
        // STEP 2: the methodology doc is a structured human-readable document
        // auto-generated from QdeltaConfig. Verify the seven sections are present
        // and the D-20 contamination result is named.
        let config = parse_fixture(QDELTA_FIXTURE_VALID);
        let doc = generate_methodology_doc(&config);

        // Section headers (Markdown ##) must be present and in order.
        assert!(doc.contains("## 1. Framing (GD-3)"), "section 1 header missing");
        assert!(
            doc.contains("## 2. Statistical floor (mechanically enforced)"),
            "section 2 header missing"
        );
        assert!(
            doc.contains("## 3. Measurement protocol"),
            "section 3 header missing"
        );
        assert!(
            doc.contains("## 4. Contamination probe (D-20)"),
            "section 4 header missing"
        );
        assert!(doc.contains("## 5. Cost"), "section 5 header missing");
        assert!(doc.contains("## 6. Reproducibility (GD-2)"), "section 6 header missing");
        assert!(doc.contains("## 7. Limitations"), "section 7 header missing");

        // D-20 contamination result must be named (observable_effect requirement).
        assert!(
            doc.contains("honestly_evaluated"),
            "methodology doc must name the D-20 contamination result; got: {doc}"
        );

        // $/task must appear (GD-3 $/task policy).
        assert!(
            doc.contains("dollar_per_task: 0.010000 USD"),
            "methodology doc must report $/task; got: {doc}"
        );

        // runpod_pending honesty must appear.
        assert!(
            doc.contains("runpod_pending: true"),
            "methodology doc must state runpod_pending honesty; got: {doc}"
        );

        // Floor verdict must be PASS for a valid config.
        assert!(
            doc.contains("floor verdict: PASS"),
            "valid config must produce PASS floor verdict; got: {doc}"
        );
    }

    #[test]
    fn methodology_doc_floor_fail_for_underpower() {
        // An under-powered config must produce a FAIL floor verdict in the doc,
        // even though floor_check() would reject the run before the doc is
        // written in production. This verifies the doc generator honestly
        // reflects the floor status from the config.
        let config = parse_fixture(QDELTA_FIXTURE_UNDERPOWER);
        let doc = generate_methodology_doc(&config);
        assert!(
            doc.contains("floor verdict: FAIL"),
            "under-powered config must produce FAIL floor verdict; got: {doc}"
        );
    }
}
