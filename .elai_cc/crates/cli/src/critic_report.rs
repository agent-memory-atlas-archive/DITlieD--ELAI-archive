//! E-10 -- `elai critic-report [--output PATH]` dispatcher.
//! Reads the live `critic_decision_event` rows from the canonical SSB, projects
//! the latest row per phase, and refuses to write when no rows exist. It does
//! not measure tiles or claim that any tile was measured; it reports whatever
//! rows are present in the database.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use elai_core::paths::sessions_dir_path;
use elai_orchestrator::ssb_viewer::SsbViewer;

const DEFAULT_SCAN_CAP: usize = 200;
const PHASES: &[&str] = &[
    "fsm_ablation",
    "role_ablation",
    "rag_challenger",
    "mistake_book_signal",
    "typed_cot_verified",
    "tier_matrix",
    "cloud_default",
    "grc_blockquote",
    "prompt_cache_hit_rate",
];

#[derive(Debug, Clone)]
struct DecisionRow {
    baseline_score: f64,
    challenger_score: f64,
    paired_bootstrap_p: Option<f64>,
    sample_size: u64,
    verdict_code: u64,
    ts_ms: i64,
}

/// Read and render the report for an explicit project root and output path.
/// This inner function is also the unit-test seam: it performs the same SSB
/// scan and latest-per-phase projection used by [`run`].
fn run_at(project_root: &Path, output: &Path, scan_cap: usize) -> anyhow::Result<()> {
    let db_path = sessions_dir_path(project_root).join("ssb.db");
    if !db_path.exists() {
        return Err(anyhow::anyhow!("there are no critic-decision rows to report (ssb.db not found)"));
    }
    let viewer = SsbViewer::open(&db_path)
        .map_err(|e| anyhow::anyhow!("could not open critic-report ssb.db: {e}"))?;
    let raw_rows = viewer
        .recent_rows_of_type("critic_decision_event", scan_cap)
        .map_err(|e| anyhow::anyhow!("could not read critic-decision rows: {e}"))?;
    if raw_rows.is_empty() {
        return Err(anyhow::anyhow!("there are no critic-decision rows to report"));
    }

    let mut latest: BTreeMap<String, DecisionRow> = BTreeMap::new();
    let mut grand_total = 0usize;
    for row in &raw_rows {
        let Some(outer) = row.payload_json.as_object() else { continue };
        let obj = outer
            .get("CriticDecisionEvent")
            .and_then(|v| v.as_object())
            .unwrap_or(outer);
        let Some(phase) = obj.get("phase").and_then(|v| v.as_str()) else { continue };
        if phase.is_empty() { continue; }
        let candidate = DecisionRow {
            baseline_score: obj.get("baseline_score").and_then(|v| v.as_f64()).unwrap_or(0.0),
            challenger_score: obj.get("challenger_score").and_then(|v| v.as_f64()).unwrap_or(0.0),
            paired_bootstrap_p: obj.get("paired_bootstrap_p").and_then(|v| v.as_f64()),
            sample_size: obj.get("sample_size").and_then(|v| v.as_u64()).unwrap_or(0),
            verdict_code: obj.get("verdict_code").and_then(|v| v.as_u64()).unwrap_or(0),
            ts_ms: obj.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0),
        };
        match latest.get(phase) {
            Some(existing) if existing.ts_ms >= candidate.ts_ms => {}
            _ => { latest.insert(phase.to_owned(), candidate); }
        }
        grand_total = grand_total.saturating_add(1);
    }
    if grand_total == 0 {
        return Err(anyhow::anyhow!("there are no critic-decision rows to report"));
    }

    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() { fs::create_dir_all(parent)?; }
    }
    let generated_ms = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    let mut report = format!(
        "# Critic decisions\n\nSource database: `{}`  \nScan cap: `{}`  \nRows scanned: `{}`  \nGenerated at (unix ms): `{}`\n\n",
        db_path.display(), scan_cap, grand_total, generated_ms
    );
    report.push_str("| Phase | Baseline score | Challenger score | Paired bootstrap p | Sample size | Verdict | Row timestamp (unix ms) |\n|---|---:|---:|---:|---:|---|---:|\n");
    for phase in PHASES {
        if let Some(row) = latest.get(*phase) {
            let verdict = match row.verdict_code { 1 => "Simplify", _ => "Keep" };
            let p = row.paired_bootstrap_p.map(|v| format!("{v}")) .unwrap_or_else(|| "-".to_owned());
            report.push_str(&format!("| {phase} | {} | {} | {p} | {} | {verdict} | {} |\n", row.baseline_score, row.challenger_score, row.sample_size, row.ts_ms));
        } else {
            report.push_str(&format!("| {phase} | NOT MEASURED | NOT MEASURED | - | - | NOT MEASURED | - |\n"));
        }
    }
    fs::write(output, report)?;
    println!("critic-report output={} rows={}", output.display(), grand_total);
    Ok(())
}

pub fn run(output: Option<String>) -> Result<(), anyhow::Error> {
    let project_root = std::env::current_dir()
        .map_err(|e| anyhow::anyhow!("could not determine cwd: {e}"))?;
    let output = output.map(PathBuf::from).unwrap_or_else(|| {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
        PathBuf::from(format!("workflow/reasoning/critic-decisions-{now}.md"))
    });
    if let Err(error) = run_at(&project_root, &output, DEFAULT_SCAN_CAP) {
        println!("{error}");
        std::process::exit(1);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::TempDir;

    /// Build a `CriticDecisionEventPayload` with caller-supplied phase + scores
    /// + ts; the non-scalar fields get deterministic filler.
    fn critic_payload(
        phase: elai_orchestrator::ssb::CriticTilePhase,
        baseline_score: f64,
        challenger_score: f64,
        verdict_code: u8,
        ts_ms: i64,
    ) -> elai_orchestrator::ssb::CriticDecisionEventPayload {
        elai_orchestrator::ssb::CriticDecisionEventPayload {
            phase,
            run_id_prefix: [0u8; 8],
            baseline_score,
            challenger_score,
            paired_bootstrap_p: Some(0.05),
            sample_size: 42,
            verdict_code,
            rationale_hash: [0u8; 8],
            ts_ms,
        }
    }

    /// Seed `count` critic_decision_event rows into the canonical
    /// `sessions_dir_path(root)/ssb.db` path that production reads, using a
    /// current-thread tokio runtime (Ssb::new spawns a single-writer task
    /// that panics without a live reactor, per bench_canary precedent).
    fn seed_rows(root: &Path, payloads: Vec<elai_orchestrator::ssb::CriticDecisionEventPayload>) {
        let db_path = sessions_dir_path(root).join("ssb.db");
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent).expect("create sessions dir");
        }
        let cfg = elai_orchestrator::SsbConfig {
            db_path,
            pool_size: None,
            project_id: "test".to_string(),
            session_id: "critic-report-test".to_string(),
        };
        let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(elai_core::OsClock);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build runtime");
        rt.block_on(async move {
            let mut ssb = elai_orchestrator::Ssb::new(cfg, clock).expect("open ssb");
            for p in payloads {
                ssb.write_payload(
                    "critic-report-test",
                    0,
                    elai_orchestrator::SsbPayload::CriticDecisionEvent(p),
                    Vec::new(),
                )
                .await
                .expect("write critic_decision_event row");
            }
            ssb.shutdown();
        });
    }

    #[test]
    fn refuses_and_writes_nothing_without_a_database() {
        let dir = TempDir::new().expect("tempdir");
        let out = dir.path().join("report.md");
        let err = run_at(dir.path(), &out, DEFAULT_SCAN_CAP).expect_err("no db -> Err");
        assert!(err.to_string().contains("no critic-decision rows"));
        assert!(!out.exists(), "output file must not be created on refusal");
    }

    #[test]
    fn keeps_the_latest_row_per_phase() {
        let dir = TempDir::new().expect("tempdir");
        // Two rows for fsm_ablation with different ts_ms, one for role_ablation.
        seed_rows(
            dir.path(),
            vec![
                critic_payload(
                    elai_orchestrator::ssb::CriticTilePhase::FsmAblation,
                    0.10,
                    0.20,
                    0,
                    1_000,
                ),
                critic_payload(
                    elai_orchestrator::ssb::CriticTilePhase::FsmAblation,
                    0.80,
                    0.90,
                    1,
                    2_000,
                ),
                critic_payload(
                    elai_orchestrator::ssb::CriticTilePhase::RoleAblation,
                    0.50,
                    0.60,
                    1,
                    1_500,
                ),
            ],
        );
        let out = dir.path().join("report.md");
        run_at(dir.path(), &out, DEFAULT_SCAN_CAP).expect("seeded db -> Ok");
        let md = fs::read_to_string(&out).expect("read report");

        // Both phases named.
        assert!(md.contains("fsm_ablation"), "report names fsm_ablation");
        assert!(md.contains("role_ablation"), "report names role_ablation");
        // Later fsm_ablation row's scores present.
        assert!(md.contains("0.8"), "later baseline score present: {md}");
        assert!(md.contains("0.9"), "later challenger score present: {md}");
        // Earlier fsm_ablation row's scores absent.
        assert!(
            !md.contains("0.1\n") && !md.contains("| 0.1 |"),
            "earlier baseline score 0.1 must not appear: {md}"
        );
    }

    #[test]
    fn an_unmeasured_phase_renders_as_not_measured() {
        let dir = TempDir::new().expect("tempdir");
        seed_rows(
            dir.path(),
            vec![
                critic_payload(
                    elai_orchestrator::ssb::CriticTilePhase::FsmAblation,
                    0.10,
                    0.20,
                    0,
                    1_000,
                ),
                critic_payload(
                    elai_orchestrator::ssb::CriticTilePhase::RoleAblation,
                    0.50,
                    0.60,
                    1,
                    1_500,
                ),
            ],
        );
        let out = dir.path().join("report.md");
        run_at(dir.path(), &out, DEFAULT_SCAN_CAP).expect("seeded db -> Ok");
        let md = fs::read_to_string(&out).expect("read report");

        // rag_challenger has no row; it must render as NOT MEASURED, not 0.0.
        let rag_line = md
            .lines()
            .find(|l| l.contains("rag_challenger"))
            .expect("rag_challenger phase row exists in report");
        assert!(
            rag_line.contains("NOT MEASURED"),
            "unmeasured phase renders as NOT MEASURED: {rag_line}"
        );
        assert!(
            !rag_line.contains("0.0"),
            "unmeasured phase must NOT render as 0.0: {rag_line}"
        );
    }
}
