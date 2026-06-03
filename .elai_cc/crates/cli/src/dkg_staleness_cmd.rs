//! F-5 / Slice A4 — `elai dkg staleness-tick --workspace <path>` (W-5 trigger).
//!
//! Single-process full cycle of the cross-subsystem staleness-quarantine
//! cascade, used as the live observable for F-5:
//!
//!   1. Open the workspace `memory_facts.sqlite` FactStore.
//!   2. Run ONE decay tick (advances the decay curve).
//!   3. `collect_quarantined` — facts past the staleness threshold.
//!   4. Push every record onto a LOCAL `QuarantineBroadcastQueue` (the W-5
//!      trigger is self-contained and does NOT touch the process-global queue
//!      that the live REPL / rebuild seams bridge through).
//!   5. `ensure_dkg_or_rebuild` the workspace DKG.
//!   6. Drain the local queue and apply the DKG (`apply_staleness_broadcast`)
//!      + cosmos (`flag_on_staleness_broadcast`) consumers -> `CascadeApplyReport`.
//!   7. Emit ONE `StalenessQuarantineBroadcast` SSB row per record, carrying the
//!      REAL `dependents_demoted` / `cosmos_artifacts_flagged` / `dropped_count`
//!      batch counts from the report (the emit MUST run AFTER the apply so the
//!      on-disk row matches the stdout headline; `Ssb::new` is constructed
//!      INSIDE `rt.block_on` per the F-2b playbook).
//!   8. Print `dependents_demoted: N` (plus the rest of the cascade report).
//!
//! Kill-switch gated (`ELAI_STALENESS_CASCADE_KILL`, default-ON): when set
//! truthy the command runs the decay tick but skips collection / emission /
//! application and prints a disabled notice.

use std::path::Path;
use std::sync::Arc;

use anyhow::{anyhow, Result};

use elai_core::clock::{ClockProvider, OsClock};
use elai_core::paths::{memory_dir_path, sessions_dir_path};
use elai_knowledge::cosmos_worker::CosmosWorker;
use elai_memory::quarantine_broadcast::{collect_quarantined, QuarantineBroadcastQueue};
use elai_memory::FactStore;
use elai_orchestrator::ssb::StalenessQuarantineBroadcastPayload;
use elai_orchestrator::{Ssb, SsbConfig, SsbPayload};

use crate::dkg_lifecycle::{
    apply_staleness_cascade, ensure_dkg_or_rebuild, staleness_cascade_killed,
};

/// Construct the 16-char blake3 hex-prefix broadcast id from `fact_id` +
/// `tick_ms`. Mirrors the emit-side id in `live_cli.rs` so a row produced by
/// either entrypoint hashes identically for the same (fact_id, tick_ms).
fn broadcast_id_for(fact_id: &str, tick_ms: u64) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(fact_id.as_bytes());
    hasher.update(&tick_ms.to_le_bytes());
    format!("{:.16}", hasher.finalize().to_hex())
}

/// Best-effort cosmos worker for the workspace (read-only — the flag path only
/// back-dates meta, it never researches). Returns `None` when no cosmos root /
/// manifest exists so the cascade still runs DKG-only.
fn try_open_cosmos() -> Option<CosmosWorker> {
    let cosmos_root = elai_knowledge::default_cosmos_root().ok()?;
    if !cosmos_root.exists() {
        return None;
    }
    let manifest_path = cosmos_root.join("manifest.toml");
    let manifest = elai_knowledge::CosmosManifest::load(&manifest_path).ok()?;
    let clock: Arc<dyn elai_knowledge::CosmosClock> =
        Arc::new(elai_knowledge::SystemCosmosClock);
    let research: Arc<dyn elai_knowledge::ResearchDispatcher> =
        Arc::new(elai_knowledge::NoopResearchDispatcher);
    Some(CosmosWorker::new_read_only(cosmos_root, manifest, clock, research))
}

/// Handler for `elai dkg staleness-tick --workspace <path>`.
pub fn run_dkg_staleness_tick(workspace: &Path) -> Result<()> {
    let clock = OsClock;
    let now_ms = clock.now_ms();
    let now_secs = i64::try_from(now_ms / 1000).unwrap_or(i64::MAX);

    // (1) Open the FactStore under the workspace memory dir.
    let mem_dir = memory_dir_path(workspace);
    let facts_path = mem_dir.join("memory_facts.sqlite");
    let facts = FactStore::open(&facts_path)
        .map_err(|e| anyhow!("dkg staleness-tick: open FactStore {}: {e}", facts_path.display()))?;

    // (2) Run one decay tick.
    match elai_memory::decay_tick(&facts, now_ms) {
        Ok(stats) => {
            eprintln!(
                "[dkg staleness-tick] decay tick: rows_examined={} rows_updated={} mean_trust={:.3}",
                stats.rows_examined, stats.rows_updated, stats.mean_trust_score
            );
        }
        Err(e) => {
            eprintln!("[dkg staleness-tick] decay tick errored (non-fatal): {e}");
        }
    }

    if staleness_cascade_killed() {
        println!("staleness cascade disabled via ELAI_STALENESS_CASCADE_KILL");
        println!("dependents_demoted: 0");
        return Ok(());
    }

    // (3) Collect quarantined facts.
    let now_tick = now_ms as u64;
    let records = collect_quarantined(&facts, now_tick)
        .map_err(|e| anyhow!("dkg staleness-tick: collect_quarantined: {e}"))?;
    eprintln!("[dkg staleness-tick] quarantined facts: {}", records.len());

    // (4) Push onto a LOCAL queue (self-contained cycle).
    let queue = QuarantineBroadcastQueue::new();
    for record in &records {
        queue.push(record.clone());
    }

    // (5) Ensure the workspace DKG (load-or-rebuild). NOTE: this also drains the
    //     PROCESS-GLOBAL queue (empty in this single-process W-5 path), which is
    //     harmless — our records live on the LOCAL queue and are applied next.
    let engine = ensure_dkg_or_rebuild(
        workspace,
        elai_knowledge::knowledge_ssb_sink::noop_sink(),
    )
    .map_err(|e| anyhow!("dkg staleness-tick: ensure_dkg_or_rebuild: {e}"))?;

    // (6) Drain the LOCAL queue and apply DKG + cosmos consumers -> the REAL
    //     batch apply counts (dependents_demoted / cosmos_artifacts_flagged /
    //     dropped_count) that the SSB row below MUST carry.
    let cosmos = try_open_cosmos();
    let report = apply_staleness_cascade(&engine, &queue, cosmos.as_ref(), now_secs)
        .map_err(|e| anyhow!("dkg staleness-tick: apply cascade: {e}"))?;

    // (7) Emit one StalenessQuarantineBroadcast SSB row per record, carrying the
    //     REAL batch apply counts from the report (B-1: the emit runs AFTER the
    //     apply so the on-disk dependents_demoted matches the stdout headline).
    //     Ssb::new is constructed INSIDE rt.block_on (single-writer task needs a
    //     reactor). Narrow u64 report counts via try_from (no panic path).
    if !records.is_empty() {
        let demoted_u32 = u32::try_from(report.dependents_demoted).unwrap_or(u32::MAX);
        let cosmos_flagged_u32 = report.cosmos_artifacts_flagged;
        let dropped_u32 = u32::try_from(report.dropped_count).unwrap_or(u32::MAX);
        let db_path = sessions_dir_path(workspace).join("ssb.db");
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                anyhow!("dkg staleness-tick: create ssb dir {}: {e}", parent.display())
            })?;
        }
        let cfg = SsbConfig {
            db_path,
            pool_size: None,
            project_id: "default".to_string(),
            session_id: "cli-dkg-staleness-tick".to_string(),
        };
        let clock_arc: Arc<dyn ClockProvider> = Arc::new(OsClock);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| anyhow!("dkg staleness-tick: build runtime: {e}"))?;
        let emit_records = records.clone();
        rt.block_on(async move {
            let ssb = Ssb::new(cfg, clock_arc)
                .map_err(|e| anyhow!("dkg staleness-tick: Ssb::new: {e}"))?;
            for record in &emit_records {
                let payload = SsbPayload::StalenessQuarantineBroadcast(
                    StalenessQuarantineBroadcastPayload {
                        fact_id: record.fact_id.clone(),
                        staleness_ratio: record.staleness_ratio,
                        // REAL batch apply counts from the CascadeApplyReport
                        // (every per-fact row carries the same batch totals).
                        dependents_demoted: demoted_u32,
                        cosmos_artifacts_flagged: cosmos_flagged_u32,
                        tick_ms: record.tick_ms,
                        broadcast_id: broadcast_id_for(&record.fact_id, record.tick_ms),
                        dropped_count: dropped_u32,
                    },
                );
                ssb.write_payload("cli.dkg_staleness_tick", 0, payload, Vec::new())
                    .await
                    .map_err(|e| anyhow!("dkg staleness-tick: SSB write: {e}"))?;
            }
            Ok::<(), anyhow::Error>(())
        })?;
    }

    // (8) Print the cascade report (dependents_demoted is the headline observable).
    println!("records_drained: {}", report.records_drained);
    println!("dropped_count: {}", report.dropped_count);
    println!("dependents_demoted: {}", report.dependents_demoted);
    println!("marker_edges_inserted: {}", report.marker_edges_inserted);
    println!("cosmos_artifacts_flagged: {}", report.cosmos_artifacts_flagged);

    Ok(())
}
