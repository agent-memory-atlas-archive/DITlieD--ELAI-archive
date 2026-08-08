//! Ledger-gated loading of the I-25 trained router artifact.
//!
//! The artifact path is intentionally environment-only in the hot path. A
//! model is considered live only after the immutable H-17 trial ledger has a
//! row for its content hash and data-window key.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use elai_core::clock::ClockProvider;
use crate::ssb::Ssb;
use crate::trial_ledger::{TrialCountResult, TrialLedger, TrialLedgerError};

pub const ROUTER_OBJECTIVE: &str = "i25_ledger_trained_router";
pub const ENV_ROUTER_ARTIFACT_PATH: &str = "ELAI_ROUTER_ARTIFACT_PATH";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouterArtifactVerdict {
    Disabled,
    PromotionBlocked { config_hash: String, reason: &'static str },
    Loaded { path: PathBuf, config_hash: String },
    Invalid { reason: String },
}

pub fn artifact_data_window(config_hash: &str) -> String {
    format!("artifact:{config_hash}")
}

pub fn artifact_config_hash(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

pub fn configured_artifact_path() -> Option<PathBuf> {
    std::env::var_os(ENV_ROUTER_ARTIFACT_PATH)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

pub fn resolve_router_artifact(
    ledger: &TrialLedger,
    path: &Path,
) -> RouterArtifactVerdict {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => return RouterArtifactVerdict::Invalid { reason: error.to_string() },
    };
    let config_hash = artifact_config_hash(&bytes);
    let window = artifact_data_window(&config_hash);
    match ledger.trial_count(ROUTER_OBJECTIVE, &window) {
        Ok(TrialCountResult::Present(_)) => RouterArtifactVerdict::Loaded {
            path: path.to_path_buf(),
            config_hash,
        },
        Ok(TrialCountResult::TrialCountMissing) | Err(_) => RouterArtifactVerdict::PromotionBlocked {
            config_hash,
            reason: "no_ledger_win",
        },
    }
}

pub async fn register_router_promotion(
    ledger: &TrialLedger,
    ssb: &Arc<Ssb>,
    clock: &Arc<dyn ClockProvider>,
    artifact_bytes: &[u8],
    seed: i64,
) -> Result<u64, TrialLedgerError> {
    let hash = artifact_config_hash(artifact_bytes);
    ledger
        .register_trial(
            ssb,
            clock,
            ROUTER_OBJECTIVE,
            &artifact_data_window(&hash),
            artifact_bytes,
            seed,
        )
        .await
}
