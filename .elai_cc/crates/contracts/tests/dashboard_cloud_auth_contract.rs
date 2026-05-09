//! Contract: the dashboard `get_oauth_status` Tauri command's underlying
//! primitives behave correctly for the three states the UI cares about:
//!   1. Credentials file does not exist  -> Ok(None) (no panic, no Err).
//!   2. Token saved with no expires_at   -> oauth_token_is_expired == false.
//!   3. Token saved with past expires_at -> oauth_token_is_expired == true.
//!
//! We exercise the providers-crate primitives directly. Bringing up a live
//! Tauri runtime + AppState is out of scope for the contracts crate (same
//! discipline as `dashboard_pi_tier3_on_open_contract.rs`). The Tauri command
//! is a thin shell around these primitives.

use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

// `elai_providers::oauth::OAuthTokenSet` and `elai_providers::OAuthTokenSet`
// (from `providers::anthropic`) are two distinct structs with identical fields
// — pre-existing dup in the providers crate. We import each from its own
// module so this test compiles against the actual function signatures.
use elai_providers::oauth::{
    clear_oauth_credentials, load_oauth_credentials, save_oauth_credentials,
    OAuthTokenSet as PersistOAuthTokenSet,
};
use elai_providers::{oauth_token_is_expired, OAuthTokenSet as ExpiryOAuthTokenSet};

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|p| p.into_inner())
}

fn temp_config_home(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "elai-dash-cloud-auth-{}-{}-{}",
        label,
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0),
    ))
}

#[test]
fn load_oauth_credentials_returns_none_when_no_file_exists() {
    let _guard = env_lock();
    let home = temp_config_home("nofile");
    std::env::set_var("ELAI_CONFIG_HOME", &home);

    let result = load_oauth_credentials();

    std::env::remove_var("ELAI_CONFIG_HOME");

    let value = result.expect("load_oauth_credentials must Ok on missing file");
    assert!(
        value.is_none(),
        "missing credentials file must produce None, got {value:?}"
    );
    assert!(!home.exists(), "load must not create the credentials dir");
}

#[test]
fn oauth_token_without_expiry_is_not_expired() {
    let token = ExpiryOAuthTokenSet {
        access_token: "tok".to_string(),
        refresh_token: None,
        expires_at: None,
        scopes: vec!["user:inference".to_string()],
    };
    assert!(
        !oauth_token_is_expired(&token),
        "token with expires_at=None must not be reported expired"
    );
}

#[test]
fn oauth_token_with_past_expiry_is_expired() {
    let past = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
        .saturating_sub(3600);
    let token = ExpiryOAuthTokenSet {
        access_token: "tok".to_string(),
        refresh_token: None,
        expires_at: Some(past),
        scopes: vec![],
    };
    assert!(
        oauth_token_is_expired(&token),
        "token whose expires_at is in the past must be reported expired"
    );
}

#[test]
fn oauth_save_and_load_round_trip_through_credentials_json() {
    let _guard = env_lock();
    let home = temp_config_home("rt");
    std::env::set_var("ELAI_CONFIG_HOME", &home);

    let token = PersistOAuthTokenSet {
        access_token: "rt-access".to_string(),
        refresh_token: Some("rt-refresh".to_string()),
        expires_at: Some(9_999_999_999),
        scopes: vec!["user:profile".to_string(), "user:inference".to_string()],
    };
    let save_result = save_oauth_credentials(&token);
    let loaded = load_oauth_credentials();
    let _ = clear_oauth_credentials();

    std::env::remove_var("ELAI_CONFIG_HOME");
    let _ = std::fs::remove_dir_all(&home);

    save_result.expect("save_oauth_credentials");
    let loaded = loaded.expect("load_oauth_credentials").expect("Some(token)");
    assert_eq!(loaded, token);
}
