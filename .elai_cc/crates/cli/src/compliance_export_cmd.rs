use std::path::PathBuf;
use std::sync::Arc;
use elai_core::clock::OsClock;
use elai_core::paths::sessions_dir_path;
use elai_orchestrator::determinism::DeterminismConfig;
use elai_orchestrator::rra::{build_rra, write_rra};
use elai_orchestrator::attestation::{build_attestation, verify_attestation, AttestationVerdict};
use elai_orchestrator::redaction::redact;
use elai_orchestrator::retention::ExportRetentionMode;
use elai_orchestrator::ssb::{Ssb, SsbConfig, SsbReader as _};
use crate::args::CliOutputFormat;

pub(crate) fn run(
    session_id: String,
    output_dir: PathBuf,
    retention: ExportRetentionMode,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    let project_root = std::env::current_dir().map_err(|e| format!("resolving current dir: {e}"))?;
    let sessions_dir = sessions_dir_path(&project_root);
    std::fs::create_dir_all(&sessions_dir)
        .map_err(|e| format!("creating sessions dir {}: {e}", sessions_dir.display()))?;
    let db_path = sessions_dir.join("ssb.db");
    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: "elai-compliance".to_string(),
        session_id: session_id.clone(),
    };
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(2)
        .build()
        .map_err(|e| format!("building tokio runtime: {e}"))?;
    runtime.block_on(async move {
        let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
        let ssb = Arc::new(Ssb::new(cfg, clock).map_err(|e| format!("ssb init: {e}"))?);
        let stream = ssb
            .query_session_stream_asc(&session_id)
            .map_err(|e| format!("reading session stream: {e}"))?;
        let pins = DeterminismConfig::from_env()
            .map_err(|e| format!("resolving determinism pins: {e}"))?;

        let artifact = build_rra(&session_id, &stream, &pins);

        std::fs::create_dir_all(&output_dir)
            .map_err(|e| format!("creating output dir {}: {e}", output_dir.display()))?;

        let rra_path = output_dir.join("compliance.rra");
        write_rra(&artifact, &rra_path).map_err(|e| format!("writing rra: {e}"))?;

        let statement =
            build_attestation(&artifact).map_err(|e| format!("building attestation: {e}"))?;
        let att_json = serde_json::to_string_pretty(&statement)
            .map_err(|e| format!("serializing attestation: {e}"))?;
        let (att_redacted, _hits) = redact(&att_json);
        let att_path = output_dir.join("attestation.json");
        std::fs::write(&att_path, att_redacted.as_bytes())
            .map_err(|e| format!("writing attestation: {e}"))?;

        let mut verified = false;
        if matches!(retention, ExportRetentionMode::ZeroAfterVerify) {
            let verdict = verify_attestation(&statement)
                .map_err(|e| format!("verifying attestation: {e}"))?;
            if matches!(verdict, AttestationVerdict::Verified { .. }) {
                verified = true;
                std::fs::write(&rra_path, b"")
                    .map_err(|e| format!("zeroing rra after verify: {e}"))?;
            }
        }

        let retention_label = match retention {
            ExportRetentionMode::Full => "full",
            ExportRetentionMode::ZeroAfterVerify => "zero-after-verify",
        };

        match output_format {
            CliOutputFormat::Json => {
                let summary = serde_json::json!({
                    "session_id": session_id,
                    "rra_path": rra_path.display().to_string(),
                    "att_path": att_path.display().to_string(),
                    "retention": retention_label,
                    "verified": verified,
                });
                println!(
                    "{}",
                    serde_json::to_string_pretty(&summary)
                        .map_err(|e| format!("serializing summary: {e}"))?
                );
            }
            _ => {
                println!("session_id: {session_id}");
                println!("rra_path: {}", rra_path.display());
                println!("att_path: {}", att_path.display());
                println!("retention: {retention_label}");
                println!("verified: {verified}");
            }
        }

        Ok::<(), String>(())
    })
}
