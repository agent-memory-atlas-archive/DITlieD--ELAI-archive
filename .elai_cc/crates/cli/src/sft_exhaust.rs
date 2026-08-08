//! I-30 Slice 1b — SFT exhaust from sealed RRA bundles.
//!
//! Walks a directory of `.rra` files, verifies each one, and emits
//! `SftExhaustEvent` SSB rows (`PathExtracted` per verified RRA, then one
//! `DatasetSealed` at the end). JSONL records are written to
//! `out_dir/<session>.jsonl`.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_core::paths::sessions_dir_path;
use elai_orchestrator::rra::{read_rra, verify_rra, ReplayVerdict};
use elai_orchestrator::ssb::{
    Ssb, SsbConfig, SsbPayload, SftExhaustEventKind, SftExhaustEventPayload, system_time_ms_i64,
};

/// Report returned by [`SftExhaust::run`].
#[derive(Debug, Clone)]
pub struct SftExhaustReport {
    /// Number of `PathExtracted` events emitted (one per verified RRA).
    pub path_extracted: u64,
    /// Number of `NegativePaired` events emitted.
    pub negative_paired: u64,
    /// 1 if the `DatasetSealed` event was emitted, 0 otherwise.
    pub dataset_sealed: u64,
    /// Number of RRAs skipped because they were not `Verified`.
    pub skipped_unsealed: u64,
}

/// I-30 Slice 1b — SFT exhaust producer.
///
/// Walks `rra_dir` for `*.rra` files, verifies each, and emits
/// `SftExhaustEvent` SSB rows + a JSONL dataset file in `out_dir`.
pub struct SftExhaust {
    rra_dir: PathBuf,
    out_dir: PathBuf,
}

impl SftExhaust {
    /// Create a new exhaust producer.
    pub fn new(rra_dir: PathBuf, out_dir: PathBuf) -> Self {
        Self { rra_dir, out_dir }
    }

    /// Run the exhaust: walk RRAs, verify, emit SSB rows + JSONL.
    pub fn run(&self) -> Result<SftExhaustReport, String> {
        let rra_files = collect_rra_files(&self.rra_dir)
            .map_err(|e| format!("walking rra_dir {}: {e}", self.rra_dir.display()))?;

        std::fs::create_dir_all(&self.out_dir)
            .map_err(|e| format!("creating out_dir {}: {e}", self.out_dir.display()))?;

        let session_id = format!("sft-exhaust-{}", system_time_ms_i64());
        let jsonl_path = self.out_dir.join(format!("{session_id}.jsonl"));

        let project_root =
            std::env::current_dir().map_err(|e| format!("resolving current dir: {e}"))?;
        let sessions_dir = sessions_dir_path(&project_root);
        std::fs::create_dir_all(&sessions_dir)
            .map_err(|e| format!("creating sessions dir {}: {e}", sessions_dir.display()))?;
        let db_path = sessions_dir.join("ssb.db");

        let ssb_config = SsbConfig {
            db_path,
            pool_size: None,
            project_id: "elai-sft-exhaust".to_string(),
            session_id: session_id.clone(),
        };

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("building tokio runtime: {e}"))?;
        let _enter = rt.enter();

        let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
        let ssb = Ssb::new(ssb_config, clock).map_err(|e| format!("ssb init: {e}"))?;

        let mut path_extracted: u64 = 0;
        let mut skipped_unsealed: u64 = 0;
        let mut total_records: u64 = 0;

        for rra_path in &rra_files {
            let rra_bytes = match std::fs::read(rra_path) {
                Ok(b) => b,
                Err(_) => {
                    skipped_unsealed += 1;
                    continue;
                }
            };

            let art = match read_rra(rra_path) {
                Ok(a) => a,
                Err(_) => {
                    skipped_unsealed += 1;
                    continue;
                }
            };

            let verdict = match verify_rra(rra_path) {
                Ok(v) => v,
                Err(_) => {
                    skipped_unsealed += 1;
                    continue;
                }
            };

            if !matches!(verdict, ReplayVerdict::Verified) {
                skipped_unsealed += 1;
                continue;
            }

            // Provenance hash: blake3 of the RRA file bytes (16-hex prefix,
            // matching the RRA convention).
            let prov_hash = blake3::hash(&rra_bytes).to_hex();
            let prov_hash_short: String = prov_hash.chars().take(16).collect();

            let record_count = u64::try_from(art.events.len()).unwrap_or(0);
            total_records += record_count;

            // Emit PathExtracted SSB row.
            let payload = SftExhaustEventPayload {
                kind: SftExhaustEventKind::PathExtracted,
                session_id: session_id.clone(),
                strategy_or_rra_path: Some(rra_path.display().to_string()),
                record_count: Some(record_count),
                provenance_verified_count: Some(1),
                notes: Some(format!("provenance_blake3={prov_hash_short}")),
                ts_ms: system_time_ms_i64(),
            };

            rt.block_on(async {
                let _ = ssb
                    .write_payload(
                        "sft-exhaust",
                        0,
                        SsbPayload::SftExhaustEvent(payload),
                        Vec::new(),
                    )
                    .await;
            });

            // Write JSONL record for this RRA.
            let jsonl_record = serde_json::json!({
                "session_id": session_id,
                "rra_path": rra_path.display().to_string(),
                "provenance_blake3": prov_hash_short,
                "record_count": record_count,
                "artifact_session_id": art.session_id,
                "seed": art.seed,
            });
            let line = serde_json::to_string(&jsonl_record)
                .map_err(|e| format!("serializing jsonl record: {e}"))?;
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&jsonl_path)
                .map_err(|e| format!("opening jsonl {}: {e}", jsonl_path.display()))?
                .write_all(format!("{line}\n").as_bytes())
                .map_err(|e| format!("writing jsonl {}: {e}", jsonl_path.display()))?;

            path_extracted += 1;
        }

        // Emit the DatasetSealed event with totals.
        let sealed_payload = SftExhaustEventPayload {
            kind: SftExhaustEventKind::DatasetSealed,
            session_id: session_id.clone(),
            strategy_or_rra_path: Some(jsonl_path.display().to_string()),
            record_count: Some(total_records),
            provenance_verified_count: Some(path_extracted),
            notes: Some(format!(
                "rra_files={} skipped={} jsonl={}",
                rra_files.len(),
                skipped_unsealed,
                jsonl_path.display()
            )),
            ts_ms: system_time_ms_i64(),
        };

        rt.block_on(async {
            let _ = ssb
                .write_payload(
                    "sft-exhaust",
                    0,
                    SsbPayload::SftExhaustEvent(sealed_payload),
                    Vec::new(),
                )
                .await;
        });

        Ok(SftExhaustReport {
            path_extracted,
            negative_paired: 0,
            dataset_sealed: 1,
            skipped_unsealed,
        })
    }
}

/// Recursively collect `*.rra` files under `dir`, sorted for determinism.
fn collect_rra_files(dir: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files = Vec::new();
    collect_rra_files_inner(dir, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_rra_files_inner(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_rra_files_inner(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "rra") {
            files.push(path);
        }
    }
    Ok(())
}
