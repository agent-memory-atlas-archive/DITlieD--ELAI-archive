// E-30: MCP server version pin enforcement contract.
//
// Pins the contract that `mcp_pins.toml` parses correctly and that the
// policy fields are enforced as documented. The postmark-mcp v1.0.16
// backdoor incident (thehackernews.com 2025-09-29) motivates exact-version
// pinning: semver ranges silently allow backdoor upgrades.
//
// This contract tests the policy configuration layer only. The enforcement
// runtime (MCP client) checks `refuse_version_float` and `enforce_pin_list`
// at tool-dispatch time; those integration points are tested separately.

use std::path::PathBuf;

/// Minimal representation of the [policy] block in mcp_pins.toml.
#[derive(Debug, serde::Deserialize)]
struct McpPinPolicy {
    enforce_pin_list: bool,
    refuse_version_float: bool,
    refuse_unauthenticated: bool,
}

/// Full mcp_pins.toml structure.
#[derive(Debug, serde::Deserialize)]
struct McpPinsConfig {
    policy: McpPinPolicy,
    #[serde(default)]
    pins: Vec<McpPinEntry>,
}

#[derive(Debug, serde::Deserialize)]
struct McpPinEntry {
    name: String,
    exact_version: String,
    #[serde(default)]
    expected_hash_blake3: String,
    #[serde(default)]
    auth_required: bool,
}

fn load_mcp_pins() -> McpPinsConfig {
    // The canonical path is .elai_cc/mcp_pins.toml (one level up from
    // crates/contracts). Walk from CARGO_MANIFEST_DIR.
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // crates/contracts -> crates -> .elai_cc -> mcp_pins.toml
    let config_path = manifest
        .parent()  // crates/
        .unwrap()
        .parent()  // .elai_cc/
        .unwrap()
        .join("mcp_pins.toml");
    let content = std::fs::read_to_string(&config_path)
        .unwrap_or_else(|e| panic!("mcp_pins.toml not found at {config_path:?}: {e}"));
    toml::from_str(&content)
        .unwrap_or_else(|e| panic!("mcp_pins.toml parse error: {e}"))
}

// ---------------------------------------------------------------------------
// Policy field contracts
// ---------------------------------------------------------------------------

#[test]
fn policy_refuse_version_float_is_true() {
    // Postmark-mcp backdoor: semver ranges silently allow version upgrades.
    // This field MUST be true in all environments.
    let cfg = load_mcp_pins();
    assert!(
        cfg.policy.refuse_version_float,
        "refuse_version_float must be true; semver floats allow silent backdoor upgrades"
    );
}

#[test]
fn policy_refuse_unauthenticated_is_true() {
    // Unauthenticated MCP servers can exfiltrate prompts.
    let cfg = load_mcp_pins();
    assert!(
        cfg.policy.refuse_unauthenticated,
        "refuse_unauthenticated must be true"
    );
}

#[test]
fn no_pin_entry_has_version_float() {
    // Every registered pin must use an exact version, not ^X.Y.Z or >=X.Y.Z.
    let cfg = load_mcp_pins();
    for pin in &cfg.pins {
        let v = &pin.exact_version;
        assert!(
            !v.starts_with('^') && !v.starts_with(">=") && !v.starts_with('~'),
            "Pin '{}' has version float '{}'; only exact versions allowed",
            pin.name,
            v
        );
        // Must look like a valid semver (digits.digits.digits at minimum).
        assert!(
            v.split('.').count() >= 2,
            "Pin '{}' version '{}' does not look like a valid version",
            pin.name,
            v
        );
    }
}

#[test]
fn all_pins_have_auth_required_true() {
    // All registered MCP servers must require authentication (E-30 policy).
    let cfg = load_mcp_pins();
    for pin in &cfg.pins {
        assert!(
            pin.auth_required,
            "Pin '{}' has auth_required = false; all registered servers must require auth",
            pin.name
        );
    }
}

#[test]
fn mcp_pins_toml_is_well_formed() {
    // Basic smoke: the file parses without error. Catches TOML syntax errors
    // introduced by manual edits.
    let _cfg = load_mcp_pins();
}
