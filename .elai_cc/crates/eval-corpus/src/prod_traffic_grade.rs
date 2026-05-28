//! E-54 real-user-traffic grade tier (task 10).
//!
//! Per arXiv:2403.04132 Chatbot Arena: ICML 2024 crowd-preference framing.
//! ELAI's adaptation: every Nth turn (default 1/100), the auditor silently
//! injects a held-out task copy as a "grade probe" (eval markers stripped
//! per matched-pair rule). User thumbs-down + grader-LLM verdict feeds
//! corpus growth via the ingest pipeline (task 1).
//!
//! Stored under `held-out/from-prod/{YYYY-MM-DD}/{turn-hash-blake3}.toml`
//! (separate shard so prod-derived tasks are clearly labelled).
//!
//! HARD INVARIANT (contract `prod_traffic_grade_defaults_off`): the feature
//! defaults to OFF. User opt-in is REQUIRED to enable. Privacy:
//! content-hash + timestamp only (no PII; user can purge with `elai
//! eval-corpus purge-prod` per the plan).

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::storage::from_prod_dir;
use crate::task_schema::Task;

/// E-54 default opt-in state: prod-traffic grade tier is OFF unless the user
/// explicitly enables it.
pub const PROD_TRAFFIC_DEFAULT_OPT_IN: bool = false;

/// E-54 default injection rate: 1 / N turns. Default N=100 = 1% sampling.
pub const PROD_TRAFFIC_DEFAULT_INJECTION_RATE: u64 = 100;

/// Opt-in config + sampling rate. Contract test
/// `prod_traffic_grade_defaults_off` asserts `opt_in == false` in the
/// default config.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProdTrafficConfig {
    /// True when the user has explicitly opted in. Default = false.
    pub opt_in: bool,
    /// `1 / injection_rate` turns get a silent probe. Default = 100.
    pub injection_rate: u64,
}

impl ProdTrafficConfig {
    /// Default config -- OPT-IN OFF.
    #[must_use]
    pub const fn default_const() -> Self {
        Self {
            opt_in: PROD_TRAFFIC_DEFAULT_OPT_IN,
            injection_rate: PROD_TRAFFIC_DEFAULT_INJECTION_RATE,
        }
    }
}

impl Default for ProdTrafficConfig {
    fn default() -> Self {
        Self::default_const()
    }
}

/// Returns a probe to inject when the current turn-count hits the sampling
/// rate; otherwise None. The actual probe sampler (the held-out store
/// reader) is delegated to the orchestrator -- this module enforces the
/// opt-in + sampling-rate gate.
#[must_use]
pub fn maybe_inject_held_out_probe(turn_count: u64, config: &ProdTrafficConfig) -> bool {
    if !config.opt_in {
        return false;
    }
    if config.injection_rate == 0 {
        return false;
    }
    turn_count % config.injection_rate == 0 && turn_count > 0
}

/// Compute the canonical from-prod shard path for a probe task.
/// `turn_hash_prefix` is the blake3-16-hex prefix of the canonical turn body
/// (computed by the caller from the conversation turn).
#[must_use]
pub fn from_prod_path(root: &Path, date: DateTime<Utc>, turn_hash_prefix: &str) -> PathBuf {
    from_prod_dir(root, date).join(format!("{turn_hash_prefix}.toml"))
}

/// Write a from-prod probe to disk. Caller has already opt-in-gated.
pub fn write_from_prod_probe(
    root: &Path,
    task: &Task,
    turn_hash_prefix: &str,
) -> Result<PathBuf, crate::storage::StorageError> {
    let p = from_prod_path(root, task.created_at, turn_hash_prefix);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            crate::storage::StorageError::Io(parent.display().to_string(), e.to_string())
        })?;
    }
    let toml_string = task
        .to_toml()
        .map_err(|e| crate::storage::StorageError::Io(p.display().to_string(), e.to_string()))?;
    std::fs::write(&p, toml_string).map_err(|e| {
        crate::storage::StorageError::Io(p.display().to_string(), e.to_string())
    })?;
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prod_traffic_grade_defaults_off() {
        let c = ProdTrafficConfig::default();
        assert!(!c.opt_in, "default config MUST be opt-in OFF");
    }

    #[test]
    fn prod_traffic_grade_requires_opt_in() {
        let c = ProdTrafficConfig::default();
        // even at turn count == injection_rate, opt_in=false -> no probe.
        assert!(!maybe_inject_held_out_probe(100, &c));
    }

    #[test]
    fn opt_in_fires_at_sampling_rate() {
        let c = ProdTrafficConfig {
            opt_in: true,
            injection_rate: 100,
        };
        assert!(maybe_inject_held_out_probe(100, &c));
        assert!(maybe_inject_held_out_probe(200, &c));
        assert!(!maybe_inject_held_out_probe(101, &c));
    }

    #[test]
    fn zero_rate_blocks_injection_even_with_opt_in() {
        let c = ProdTrafficConfig {
            opt_in: true,
            injection_rate: 0,
        };
        assert!(!maybe_inject_held_out_probe(100, &c));
    }
}
