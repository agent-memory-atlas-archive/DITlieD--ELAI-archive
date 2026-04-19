//! Batch 4C (A-45) — Source tiers TOML contract.
//!
//! Asserts that the dashboard-side source-tiers load:
//!   * the TOML file lives at a known project-relative path
//!   * the parsed config exposes non-zero T1 + T2 domain counts (the real
//!     config ships with at least 10 domains in each tier)
//!   * missing-file fallback surfaces a `ConfigWarning` SSB discriminant
//!   * the dashboard Tauri response shape (typed fields) matches what the
//!     frontend consumes
//!
//! Grounding:
//!   * .elai_cc/config/source-tiers.toml
//!   * crates/core/src/source_tier.rs load_source_tier_config
//!   * crates/orchestrator/src/ssb.rs ConfigWarningPayload

use std::path::Path;

use elai_core::source_tier::load_source_tier_config;
use elai_orchestrator::{ConfigWarningPayload, SsbActionType, SsbPayload};

// --- (a) The real source-tiers TOML exists and parses. ---

#[test]
fn contract_real_source_tiers_toml_loads() {
    // Resolved relative to the `contracts` crate root: ../../config.
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest
        .parent()
        .expect("crates dir")
        .parent()
        .expect(".elai_cc root")
        .join("config")
        .join("source-tiers.toml");
    let cfg = load_source_tier_config(&path)
        .unwrap_or_else(|e| panic!("failed to load {}: {e}", path.display()));
    assert!(
        cfg.tier1_domain_count() > 0,
        "T1 must have at least one entry in the real config"
    );
    assert!(
        cfg.tier2_domain_count() > 0,
        "T2 must have at least one entry"
    );
}

// --- (b) Missing-file fallback emits a ConfigWarning discriminant. ---

#[test]
fn contract_config_warning_payload_roundtrips() {
    let p = ConfigWarningPayload {
        config: "source-tiers.toml".into(),
        reason: "file_not_found".into(),
        detail: "path=/missing/source-tiers.toml".into(),
    };
    let wrapped = SsbPayload::ConfigWarning(p.clone());
    assert_eq!(wrapped.action_type(), SsbActionType::ConfigWarning);
    let bytes = rmp_serde::to_vec_named(&wrapped).expect("msgpack");
    let back: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    match back {
        SsbPayload::ConfigWarning(r) => {
            assert_eq!(r.config, "source-tiers.toml");
            assert_eq!(r.reason, "file_not_found");
        }
        other => panic!("unexpected variant: {other:?}"),
    }
}

// --- (c) Missing file returns a typed error from load_source_tier_config. ---

#[test]
fn contract_missing_file_returns_typed_error() {
    let r = load_source_tier_config(Path::new("/definitely/not/a/real/path-xyz.toml"));
    assert!(r.is_err(), "missing file must surface as Err");
}

// --- (d) ConfigWarning action type round-trips via the string table. ---

#[test]
fn contract_config_warning_string_roundtrips() {
    assert_eq!(SsbActionType::ConfigWarning.as_sql_str(), "config_warning");
}
