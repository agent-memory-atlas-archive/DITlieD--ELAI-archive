//! E-10 / E-39 -- `elai bench-ablation <tile>` dispatcher.
//!
//! E-10 baseline: scaffolded as a printf-stub so the wire-format runs
//! end-to-end in CI without a live multi-agent run.
//!
//! E-39 augmentation (this revision): when `tile == "automix_verifier"`
//! we drive `budget` iterations of [`elai_orchestrator::cost::automix_gate::parse_verifier_response`]
//! against a synthetic deterministic JSON pattern set (continue / retry /
//! escalate / malformed) so the ablation surface is observable in CI
//! without a live sidecar call. The 30-step default in the E-39 plan
//! lands as `--budget 30`. No SsbActionType variants are emitted (those
//! are written by the orchestrator-side gate when the live verifier
//! fires; this CLI exercises the parser surface only).
//!
//! E-10: `fsm_ablation`, `role_ablation`, and `rag_challenger` dispatch
//! through the orchestrator harness. `cot_proof` and `grc_blockquote`
//! refuse because this CLI has no measurement source.
//!
//! See workflow/reasoning/E-10-decisions.md + E-39-decisions.md.

use std::path::Path;
use std::sync::Arc;

use elai_core::paths::sessions_dir_path;
use elai_core::OsClock;
use elai_orchestrator::cost::automix_gate::{parse_verifier_response, AutomixVerdict};
use elai_orchestrator::fsm_ablation::run_fsm_ablation;
use elai_orchestrator::rag_challenger::run_rag_challenger;
use elai_orchestrator::role_ablation::{run_role_ablation, RoleProfileConfig};
use elai_orchestrator::{Ssb, SsbConfig};

const PATTERNS: [&str; 4] = [
    r#"{"verdict":"continue","reason":"structurally valid"}"#,
    r#"{"verdict":"retry","reason":"borderline output, retry"}"#,
    r#"{"verdict":"escalate","reason":"requires stronger model"}"#,
    r#"{"verdict":"banana","reason":"malformed verdict label"}"#,
];
const AGENT_ID: &str = "cli:bench-ablation";

pub fn run(tile: String, fixtures: String, budget: u32) -> Result<(), anyhow::Error> {
    let root = std::env::current_dir().map_err(anyhow::Error::from)?;
    match run_with_root(tile, fixtures, budget, &root) {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("BENCH-ABLATION ABORT: {e}");
            std::process::exit(1);
        }
    }
}

fn run_with_root(tile: String, fixtures: String, budget: u32, project_root: &Path) -> Result<(), anyhow::Error> {
    if tile == "automix_verifier" { return run_automix_verifier_ablation(fixtures, budget); }
    if std::env::var("ELAI_CRITIC_OF_DESIGN_ENABLE").ok().as_deref() != Some("1") {
        println!("bench-ablation tile={tile} is wire-dark behind ELAI_CRITIC_OF_DESIGN_ENABLE=1");
        return Ok(());
    }
    if tile == "cot_proof" || tile == "grc_blockquote" {
        return Err(anyhow::anyhow!("tile {tile} refused: CLI has no measurement source; tile stays wire-dark"));
    }
    if !matches!(tile.as_str(), "fsm_ablation" | "role_ablation" | "rag_challenger") {
        return Err(anyhow::anyhow!("unknown tile {tile}; legal tiles: fsm_ablation, role_ablation, rag_challenger (dispatch), cot_proof, grc_blockquote (refuse for want of a source), automix_verifier"));
    }
    let db_path = sessions_dir_path(project_root).join("ssb.db");
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            anyhow::anyhow!(
                "failed to create sessions directory {}: {e}",
                parent.display()
            )
        })?;
    }
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
    let outcome = rt.block_on(async {
        let cfg = SsbConfig { db_path, pool_size: None, project_id: "default".into(), session_id: "cli-bench-ablation".into() };
        let ssb = Arc::new(Ssb::new(cfg, Arc::new(OsClock))?);
        match tile.as_str() {
            "fsm_ablation" => { let (r, rows) = run_fsm_ablation(&ssb, AGENT_ID, 0, &fixtures, budget).await?; if r.runpod_pending { return Err(anyhow::anyhow!("tile fsm_ablation refused: measurement is pending")); } println!("tile=fsm_ablation full_pipeline={} single_agent_wait={} decisions={rows:?}", r.full_pipeline.pass_rate, r.single_agent_wait.pass_rate); }
            "role_ablation" => { let (r, rows) = run_role_ablation(&ssb, AGENT_ID, 0, &fixtures, budget, RoleProfileConfig::self_moa_default()).await?; if r.runpod_pending { return Err(anyhow::anyhow!("tile role_ablation refused: measurement is pending")); } println!("tile=role_ablation full_pipeline={} single_agent_wait={} decisions={rows:?}", r.full_pipeline.pass_rate, r.single_agent_wait.pass_rate); }
            "rag_challenger" => { let r = run_rag_challenger(&ssb, AGENT_ID, 0, 0.0).await?; if r.runpod_pending { return Err(anyhow::anyhow!("tile rag_challenger refused: measurement is pending")); } println!("tile=rag_challenger mrr={}", r.mrr); }
            _ => unreachable!(),
        }
        Ok::<(), anyhow::Error>(())
    });
    outcome
}

/// E-39 -- drive the AutoMix verifier parser through `budget` synthetic
/// rounds. Counts each verdict + malformed event and reports totals.
fn run_automix_verifier_ablation(fixtures: String, budget: u32) -> Result<(), anyhow::Error> {
    let budget = budget.max(1);
    let mut n_continue: u32 = 0;
    let mut n_retry: u32 = 0;
    let mut n_escalate: u32 = 0;
    let mut n_malformed: u32 = 0;
    for step in 0..budget {
        let body = PATTERNS[(step as usize) % PATTERNS.len()];
        match parse_verifier_response(body) {
            Ok((AutomixVerdict::Continue, _)) => n_continue += 1,
            Ok((AutomixVerdict::Retry, _)) => n_retry += 1,
            Ok((AutomixVerdict::Escalate, _)) => n_escalate += 1,
            Err(_) => n_malformed += 1,
        }
    }
    println!(
        "bench-ablation tile=automix_verifier fixtures={fixtures} budget={budget} continue={n_continue} retry={n_retry} escalate={n_escalate} malformed={n_malformed}"
    );
    // Sanity: every step must be accounted for.
    let total = n_continue + n_retry + n_escalate + n_malformed;
    if total != budget {
        return Err(anyhow::anyhow!(
            "automix_verifier ablation drop: total={total} budget={budget}"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};
    use tempfile::TempDir;

    fn test_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().expect("test lock")
    }

    #[test]
    fn flag_off_does_not_dispatch() {
        let _guard = test_lock();
        std::env::remove_var("ELAI_CRITIC_OF_DESIGN_ENABLE");
        let dir = TempDir::new().expect("tempdir");
        assert!(run_with_root("fsm_ablation".into(), "fixture".into(), 30, dir.path()).is_ok());
    }

    #[test]
    fn flag_on_fsm_returns_pending_refusal() {
        let _guard = test_lock();
        std::env::set_var("ELAI_CRITIC_OF_DESIGN_ENABLE", "1");
        let dir = TempDir::new().expect("tempdir");
        let err = run_with_root("fsm_ablation".into(), "fixture".into(), 30, dir.path()).unwrap_err();
        assert!(err.to_string().contains("measurement is pending"));
    }

    #[test]
    fn flag_on_cot_proof_returns_no_source_error() {
        let _guard = test_lock();
        std::env::set_var("ELAI_CRITIC_OF_DESIGN_ENABLE", "1");
        let dir = TempDir::new().expect("tempdir");
        let err = run_with_root("cot_proof".into(), "fixture".into(), 30, dir.path()).unwrap_err();
        assert!(err.to_string().contains("no measurement source"));
    }

    #[test]
    fn flag_on_unknown_tile_returns_unknown_tile_error() {
        let _guard = test_lock();
        std::env::set_var("ELAI_CRITIC_OF_DESIGN_ENABLE", "1");
        let dir = TempDir::new().expect("tempdir");
        let err = run_with_root("unknown_tile".into(), "fixture".into(), 30, dir.path()).unwrap_err();
        assert!(err.to_string().contains("unknown tile"));
    }
}
