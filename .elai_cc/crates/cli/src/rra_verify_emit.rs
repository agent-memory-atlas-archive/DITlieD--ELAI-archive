//! J-14 S6b — `emit_j14_verify_repro_pack_events` split from rra_cmd.rs.
//!
//! Moved to satisfy MR-1 (rra_cmd.rs under 1000 lines). The function emits
//! TierEvaluated and FirstDivergenceLocated SSB rows for `replay --verify`
//! runs, allowing a consumer to correlate the row back to the sealed bundle.

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_core::paths::sessions_dir_path;
use elai_orchestrator::rra::{ReplayVerdict, RunArtifact, TierVerdictLabel, VerifyOutcome};
use elai_orchestrator::rra_emit::{
    build_first_divergence_payload, build_tier_evaluated_payload,
};
use elai_orchestrator::ssb::{Ssb, SsbConfig, SsbPayload, system_time_ms_i64};

/// J-14 S5 — project id for the rra verify ReproPackEvent SSB rows. Distinct
/// from the H-23..H-25 fixture project ids so a consumer can filter the
/// TierEvaluated / FirstDivergenceLocated rows emitted by `replay --verify`
/// without picking up the demo fixture rows.
pub(crate) const J14_RRA_VERIFY_PROJECT_ID: &str = "j14-rra-verify";

/// J-14 S5 — emits TierEvaluated and FirstDivergenceLocated SSB rows for
/// `replay --verify` runs. Both writes are best-effort: an `Ssb::new` or
/// `write_payload` error is logged as a warning but does not fail the verify
/// command (the load-bearing signal is the exit code, not the SSB row). The
/// consumer can correlate the row back to the sealed bundle.
pub(crate) fn emit_j14_verify_repro_pack_events(
    art: &RunArtifact,
    outcome: &VerifyOutcome,
    tier_requested: TierVerdictLabel,
) {
    let project_root = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!(error = %e, "j14 rra verify ssb: current_dir failed, skipping SSB emit");
            return;
        }
    };
    let sessions_dir = sessions_dir_path(&project_root);
    if let Err(e) = std::fs::create_dir_all(&sessions_dir) {
        tracing::warn!(
            error = %e,
            sessions_dir = %sessions_dir.display(),
            "j14 rra verify ssb: create_dir_all failed, skipping SSB emit"
        );
        return;
    }
    let db_path = sessions_dir.join("ssb.db");
    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: J14_RRA_VERIFY_PROJECT_ID.to_string(),
        session_id: art.session_id.clone(),
    };

    // Build the payloads outside the runtime so a builder panic (none expected,
    // but defensive) is not inside block_on. The Ssb::new + write_payload are
    // async (Ssb::new spawns its write-loop onto the reactor), so they run
    // inside block_on on a multi-thread runtime, mirroring run_verify_rerun.
    let ts_ms = system_time_ms_i64();
    let tier_payload = outcome
        .tier
        .as_ref()
        .map(|tier| build_tier_evaluated_payload(&art.session_id, tier_requested, tier, ts_ms));
    let divergence_payload = match &outcome.verdict {
        ReplayVerdict::Divergence {
            first_divergence: Some(fd),
            ..
        } => Some(build_first_divergence_payload(&art.session_id, fd, ts_ms)),
        _ => None,
    };
    if tier_payload.is_none() && divergence_payload.is_none() {
        return;
    }

    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(2)
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            tracing::warn!(error = %e, "j14 rra verify ssb: tokio runtime build failed, skipping SSB emit");
            return;
        }
    };
    let session_id = art.session_id.clone();
    runtime.block_on(async move {
        let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
        let ssb = match Ssb::new(cfg, clock) {
            Ok(s) => Arc::new(s),
            Err(e) => {
                tracing::warn!(error = %e, "j14 rra verify ssb: Ssb::new failed, skipping SSB emit");
                return;
            }
        };
        if let Some(payload) = tier_payload {
            let wrapped = SsbPayload::ReproPackEvent(payload);
            if let Err(e) = ssb
                .write_payload("rra_verify", 0, wrapped, Vec::new())
                .await
            {
                tracing::warn!(
                    error = %e,
                    session_id = %session_id,
                    "j14 rra verify ssb: TierEvaluated write_payload failed (non-fatal)"
                );
            }
        }
        if let Some(payload) = divergence_payload {
            let wrapped = SsbPayload::ReproPackEvent(payload);
            if let Err(e) = ssb
                .write_payload("rra_verify", 0, wrapped, Vec::new())
                .await
            {
                tracing::warn!(
                    error = %e,
                    session_id = %session_id,
                    "j14 rra verify ssb: FirstDivergenceLocated write_payload failed (non-fatal)"
                );
            }
        }
    });
}
