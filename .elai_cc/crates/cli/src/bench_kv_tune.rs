//! I-16 KV-cache tuning Tier-1 dry-run CLI.
use std::process::Command;

use elai_core::paths::sessions_dir_path;
use elai_orchestrator::ssb::KvCacheTunedPayload;
use elai_orchestrator::{Ssb, SsbConfig, SsbPayload};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct HarnessResult {
    config_hash: String,
    fixture_pass_rate: f64,
}

pub fn run(dry_run: bool, emit_ssb: bool, json: bool) -> anyhow::Result<()> {
    if !dry_run {
        anyhow::bail!("live kv-tune is deferred; pass --dry-run");
    }
    let root = std::env::current_dir()?;
    let python_path = root.join(".elai_cc/python");
    let output = Command::new("python")
        .args(["-m", "elai_inference.eval_kv_tune", "--dry-run"])
        .env("PYTHONPATH", &python_path)
        .output()
        .map_err(|e| anyhow::anyhow!("launch KV harness: {e}"))?;
    if !output.status.success() {
        anyhow::bail!(
            "KV harness failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| anyhow::anyhow!("harness output is not UTF-8: {e}"))?;
    let result: HarnessResult = stdout
        .lines()
        .rev()
        .find_map(|line| serde_json::from_str(line).ok())
        .ok_or_else(|| anyhow::anyhow!("KV harness emitted no JSON result"))?;
    if emit_ssb {
        emit(result.config_hash.clone(), result.fixture_pass_rate)?;
    }
    if json {
        print!("{stdout}");
    } else {
        println!("kv-tune dry-run: {}", result.config_hash);
    }
    Ok(())
}

fn emit(config_hash: String, fixture_pass_rate: f64) -> anyhow::Result<()> {
    let db_path = sessions_dir_path(&std::env::current_dir()?).join("ssb.db");
    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: "default".into(),
        session_id: "bench-kv-tune".into(),
    };
    let clock: std::sync::Arc<dyn elai_core::clock::ClockProvider> =
        std::sync::Arc::new(elai_core::OsClock);
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(async move {
        let ssb = Ssb::new(cfg, clock).map_err(|e| anyhow::anyhow!(e))?;
        ssb.write_payload(
            "bench-kv-tune",
            0,
            SsbPayload::KvCacheTuned(KvCacheTunedPayload {
                vram_gb: None,
                tok_s: None,
                fixture_pass_rate,
                config_hash,
                adopt_decision: "DryRun".into(),
            }),
            Vec::new(),
        )
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
        Ok(())
    })
}
