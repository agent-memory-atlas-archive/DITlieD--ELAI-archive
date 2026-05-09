//! Contract: the dashboard `set_cloud_model` / `get_cloud_model` Tauri
//! commands persist the chosen Anthropic model ID alongside the OAuth token
//! in `~/.elai/credentials.json` (env-overridable via `ELAI_CONFIG_HOME`).
//!
//! Asserts:
//!   1. `load_cloud_model_id` returns Ok(None) when the credentials file is
//!      absent (no panic, no Err).
//!   2. `save_cloud_model_id` + `load_cloud_model_id` round-trips a value.
//!   3. Writing the cloud model preserves an existing `oauth` key in the same
//!      JSON file (no co-tenant clobber).
//!   4. `is_valid_cloud_model_id` rejects unknown IDs and accepts the
//!      shipped Anthropic catalog (the dashboard's allowlist).
//!
//! Bringing up a Tauri runtime + AppState is out of scope for the contracts
//! crate; the Tauri command is a thin shell around these primitives.

use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use elai_providers::{
    cloud_model_id_allowlist, is_valid_cloud_model_id, load_cloud_model_id,
    save_cloud_model_id,
};

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|p| p.into_inner())
}

fn temp_config_home(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "elai-dash-cloud-model-{}-{}-{}",
        label,
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0),
    ))
}

#[test]
fn load_cloud_model_id_returns_none_when_no_file_exists() {
    let _guard = env_lock();
    let home = temp_config_home("nofile");
    std::env::set_var("ELAI_CONFIG_HOME", &home);

    let result = load_cloud_model_id();

    std::env::remove_var("ELAI_CONFIG_HOME");

    let value = result.expect("load_cloud_model_id must Ok on missing file");
    assert!(
        value.is_none(),
        "missing credentials file must produce None, got {value:?}"
    );
    assert!(!home.exists(), "load must not create the credentials dir");
}

#[test]
fn save_and_load_cloud_model_id_round_trip() {
    let _guard = env_lock();
    let home = temp_config_home("rt");
    std::env::set_var("ELAI_CONFIG_HOME", &home);

    let chosen = "claude-sonnet-4-6";
    let save_result = save_cloud_model_id(chosen);
    let loaded = load_cloud_model_id();

    std::env::remove_var("ELAI_CONFIG_HOME");
    let _ = std::fs::remove_dir_all(&home);

    save_result.expect("save_cloud_model_id");
    assert_eq!(loaded.expect("load_cloud_model_id"), Some(chosen.to_string()));
}

#[test]
fn save_cloud_model_id_preserves_existing_oauth_key() {
    let _guard = env_lock();
    let home = temp_config_home("preserve");
    std::env::set_var("ELAI_CONFIG_HOME", &home);
    std::fs::create_dir_all(&home).expect("create config home");
    let credentials_path = home.join("credentials.json");
    let seed = r#"{"oauth":{"access_token":"seed-token","scopes":[]}}"#;
    std::fs::write(&credentials_path, seed).expect("seed credentials.json");

    let save_result = save_cloud_model_id("claude-haiku-4-5-20251001");
    let after = std::fs::read_to_string(&credentials_path).unwrap_or_default();

    std::env::remove_var("ELAI_CONFIG_HOME");
    let _ = std::fs::remove_dir_all(&home);

    save_result.expect("save_cloud_model_id");
    assert!(
        after.contains("\"access_token\""),
        "must preserve nested oauth fields, got: {after}"
    );
    assert!(
        after.contains("\"cloud_model_id\""),
        "must add cloud_model_id, got: {after}"
    );
    assert!(
        after.contains("claude-haiku-4-5-20251001"),
        "must persist the chosen model id, got: {after}"
    );
}

#[test]
fn is_valid_cloud_model_id_rejects_unknown_and_accepts_allowlist() {
    assert!(
        !is_valid_cloud_model_id(""),
        "empty string must be rejected"
    );
    assert!(
        !is_valid_cloud_model_id("not-a-model"),
        "unknown model id must be rejected"
    );
    assert!(
        !is_valid_cloud_model_id("Qwen2.5-Coder-7B-Instruct-abliterated"),
        "non-Anthropic / local model id must be rejected from the cloud allowlist"
    );

    let allowlist = cloud_model_id_allowlist();
    assert!(
        !allowlist.is_empty(),
        "cloud model allowlist must not be empty"
    );
    for id in allowlist {
        assert!(
            is_valid_cloud_model_id(id),
            "allowlist entry {id:?} must be accepted"
        );
        assert!(
            id.starts_with("claude-"),
            "allowlist must contain only Anthropic Claude IDs, found {id:?}"
        );
    }
}
