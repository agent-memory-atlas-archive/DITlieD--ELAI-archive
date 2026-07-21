//! J-30 T7c -- SSB emit helpers for `elai providers configure`.
//!
//! Split out of `providers_cmd.rs` (MR-1 file-size: providers_cmd crossed
//! the 1000-line cap). Holds the two SSB-seam helpers:
//! `open_providers_ssb_writer` (open the SSB writer used by configure) and
//! `emit_provider_byok_event` (best-effort `ProviderByokEvent` row emit).
//! Semantics are unchanged from the pre-split code.

use std::path::Path;
use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::{
    default_ssb_path_for_base, Ssb, SsbConfig, SsbPayload,
    ssb::{ProviderByokEventKind, ProviderByokEventPayload},
};
use tracing::warn;

/// J-30 T7c: best-effort `ProviderByokEvent` SSB emit at the configure
/// seam. Mirrors `auth_broker_emit::emit_auth_broker_event` shape: fire
/// and forget on the current tokio handle, warn on failure, never surface
/// an SSB error to the configure caller. NO key material in the payload.
pub(crate) fn emit_provider_byok_event(
    ssb: &Arc<Ssb>,
    backend: &str,
    kind: ProviderByokEventKind,
    ok: bool,
) {
    let ts_ms = OsClock.now_ms() as i64;
    let payload = SsbPayload::ProviderByokEvent(ProviderByokEventPayload {
        backend: backend.to_owned(),
        kind,
        ok,
        ts_ms,
    });
    let ssb = Arc::clone(ssb);
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => {
            handle.spawn(async move {
                if let Err(e) = ssb.write_payload("providers_configure", 0, payload, vec![]).await {
                    warn!(
                        byok.ssb_err = %e,
                        "ProviderByokEvent SSB emit failed (configure result unaffected)"
                    );
                }
            });
        }
        Err(_) => {
            warn!("providers_configure: no tokio runtime, skipping ProviderByokEvent row");
        }
    }
}

/// J-30 T7c: open the SSB writer used by `elai providers configure` to
/// emit `ProviderByokEvent` rows. Mirrors `runtime_cmd::open_runtime_ssb_writer`:
/// `default_ssb_path_for_base(project_root)`, create parents, open `Ssb::new`,
/// downgrade any open failure to `None` (best-effort; a missing SSB row is
/// never a correctness failure for configure).
pub(crate) fn open_providers_ssb_writer(project_root: &Path) -> Option<Arc<Ssb>> {
    let db_path = default_ssb_path_for_base(project_root);
    let Some(parent) = db_path.parent() else {
        warn!("providers: ssb.db has no parent directory; ProviderByokEvent row will be skipped");
        return None;
    };
    if let Err(e) = std::fs::create_dir_all(parent) {
        warn!(err = %e, "providers: creating ssb sessions dir failed; ProviderByokEvent row will be skipped");
        return None;
    }
    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: "default".to_string(),
        session_id: "cli-providers-configure".to_string(),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    match Ssb::new(cfg, clock) {
        Ok(s) => Some(Arc::new(s)),
        Err(e) => {
            warn!(err = %e, "providers: SSB open failed; ProviderByokEvent row will be skipped");
            None
        }
    }
}
