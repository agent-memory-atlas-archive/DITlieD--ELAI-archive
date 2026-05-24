//! E-30: MCP server version pin loader + enforcement backend.
//!
//! Loads `.elai_cc/mcp_pins.toml` at startup, validates that every registered
//! pin uses an exact version (no `^X.Y.Z` / `>=X.Y.Z` / `~X.Y.Z` floats), and
//! exposes [`enforce_pin`] which returns a verdict for a `(name, observed_version)`
//! tuple. On mismatch the caller emits the
//! `SsbActionType::McpPinVersionRefused` row -- the SSB type lives in the
//! orchestrator crate (out of reach without a circular dep), so this module
//! returns a typed verdict and lets the call site do the emit.
//!
//! Architecture decision (workflow/reasoning/E-30-decisions.md M3): extend
//! existing `crates/plugins` rather than scaffolding a new `crates/mcp-loader`
//! crate. Rationale:
//!   - plugins already owns the live MCP transport (`mcp_bridge`),
//!     schema-hash pinning (`mcp_schema_pin`), and the `mcp` feature gate
//!     for reqwest+toml.
//!   - A new crate would split the MCP-hardening surface across two crates
//!     for no added isolation (both need `mcp_bridge` types).
//!   - `mcp_pins.toml` is a small config; the loader is ~150 LOC, no new
//!     external deps (toml + serde + thiserror + blake3 + tracing all
//!     already pulled in by the `mcp` feature).
//!
//! Reference: postmark-mcp v1.0.16 inserted a one-line BCC backdoor after
//! 15 clean versions (thehackernews.com 2025-09-29 + snyk.io). Semver floats
//! silently allow this kind of upgrade; exact pins refuse it loudly.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, warn};

/// Default mcp_pins.toml path relative to the workspace root. Tested against
/// the same path the `mcp_pin_violation_contract` integration test loads.
pub const DEFAULT_PINS_PATH: &str = "mcp_pins.toml";

/// Errors emitted by the pin loader / enforcer.
#[derive(Debug, Error)]
pub enum McpPinsError {
    #[error("mcp_pins.toml not found at {path}: {source}")]
    NotFound {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("mcp_pins.toml parse error at {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("pin '{name}' has version float '{version}' -- only exact versions allowed (E-30 / postmark-mcp v1.0.16 precedent)")]
    VersionFloat { name: String, version: String },
    #[error("pin '{name}' version '{version}' does not look like a valid semver")]
    MalformedVersion { name: String, version: String },
    #[error("pin '{name}' has auth_required = false but policy.refuse_unauthenticated = true")]
    UnauthenticatedRefused { name: String },
}

/// Policy block parsed from `[policy]` in mcp_pins.toml.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpPinPolicy {
    /// When true, refuse any MCP server not listed in `[[pins]]`.
    /// Default false so existing single-server setups don't break on upgrade.
    #[serde(default)]
    pub enforce_pin_list: bool,
    /// When true (default), refuse any pin entry whose `exact_version`
    /// starts with `^` / `~` / `>=` / etc.
    #[serde(default = "default_true")]
    pub refuse_version_float: bool,
    /// When true (default), refuse any pin entry with `auth_required = false`.
    #[serde(default = "default_true")]
    pub refuse_unauthenticated: bool,
}

fn default_true() -> bool {
    true
}

impl Default for McpPinPolicy {
    fn default() -> Self {
        Self {
            enforce_pin_list: false,
            refuse_version_float: true,
            refuse_unauthenticated: true,
        }
    }
}

/// One `[[pins]]` entry: an exact-version pin for a single MCP server.
///
/// Plan-named fields (E-30 Task 4):
///   - `name`                  : MCP server registry name
///   - `exact_version`         : pinned version (no floats)
///   - `expected_pubkey_blake3`: optional blake3 hex of maintainer pubkey
///     (semantic: when present, the live signature on the artifact must
///     hash to this value; when empty, signature verification is skipped
///     and only the version pin is enforced)
///   - `auth_required`         : whether the server must run with bearer/mTLS
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpServerPin {
    /// MCP server registry name (e.g. "postmark-mcp").
    pub name: String,
    /// Exact pinned version (e.g. "1.0.15"). Floats refused by policy.
    pub exact_version: String,
    /// Optional blake3 hex of expected maintainer pubkey or artifact hash.
    /// When empty, signature verification is skipped and only the version
    /// pin is enforced.
    #[serde(default)]
    pub expected_pubkey_blake3: String,
    /// Whether the server must run with authentication.
    #[serde(default)]
    pub auth_required: bool,
}

/// Full parsed mcp_pins.toml file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpPinsConfig {
    /// Policy block (defaults applied when missing).
    #[serde(default)]
    pub policy: McpPinPolicy,
    /// List of pinned servers.
    #[serde(default)]
    pub pins: Vec<McpServerPin>,
}

/// Verdict emitted by [`enforce_pin`].
///
/// The call site is responsible for emitting the SSB row on `Refused` --
/// crossing the orchestrator boundary from inside `crates/plugins` would
/// introduce a circular dependency (mirrors the existing
/// `ToolGateSsbEmitter` pattern in `mcp::mod.rs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpPinVerdict {
    /// Server matched the pinned exact version (or was not pinned and
    /// `enforce_pin_list` is false). Caller may proceed.
    Allow,
    /// Server's observed version does not match the pin.
    /// Emit `SsbActionType::McpPinVersionRefused` row at call site.
    Refused {
        name: String,
        pinned_version: String,
        observed_version: String,
        reason: &'static str,
    },
}

impl McpPinsConfig {
    /// Load + validate a pins file from disk. Returns an error on any
    /// version-float / malformed-version / unauthenticated-refused row.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, McpPinsError> {
        let path_ref = path.as_ref();
        let content = fs::read_to_string(path_ref).map_err(|e| McpPinsError::NotFound {
            path: path_ref.to_path_buf(),
            source: e,
        })?;
        let cfg: McpPinsConfig =
            toml::from_str(&content).map_err(|e| McpPinsError::Parse {
                path: path_ref.to_path_buf(),
                source: e,
            })?;
        cfg.validate()?;
        debug!(
            path = %path_ref.display(),
            pins = cfg.pins.len(),
            enforce_pin_list = cfg.policy.enforce_pin_list,
            refuse_version_float = cfg.policy.refuse_version_float,
            refuse_unauthenticated = cfg.policy.refuse_unauthenticated,
            "E-30: mcp_pins.toml loaded"
        );
        Ok(cfg)
    }

    /// Validate every pin against the active policy. Called by [`load`].
    pub fn validate(&self) -> Result<(), McpPinsError> {
        for pin in &self.pins {
            if self.policy.refuse_version_float && is_version_float(&pin.exact_version) {
                return Err(McpPinsError::VersionFloat {
                    name: pin.name.clone(),
                    version: pin.exact_version.clone(),
                });
            }
            if !looks_like_semver(&pin.exact_version) {
                return Err(McpPinsError::MalformedVersion {
                    name: pin.name.clone(),
                    version: pin.exact_version.clone(),
                });
            }
            if self.policy.refuse_unauthenticated && !pin.auth_required {
                return Err(McpPinsError::UnauthenticatedRefused {
                    name: pin.name.clone(),
                });
            }
        }
        Ok(())
    }

    /// Lookup a pin by registry name. None when not registered.
    #[must_use]
    pub fn find(&self, name: &str) -> Option<&McpServerPin> {
        self.pins.iter().find(|p| p.name == name)
    }

    /// Enforce the pin policy against an observed `(name, version)` tuple.
    ///
    /// Behavior:
    ///   - server is in `pins` AND version matches `exact_version` -> Allow
    ///   - server is in `pins` AND version differs -> Refused (version-mismatch)
    ///   - server NOT in `pins` AND `enforce_pin_list` is true -> Refused (unpinned)
    ///   - server NOT in `pins` AND `enforce_pin_list` is false -> Allow
    #[must_use]
    pub fn enforce_pin(&self, name: &str, observed_version: &str) -> McpPinVerdict {
        match self.find(name) {
            Some(pin) => {
                if pin.exact_version == observed_version {
                    McpPinVerdict::Allow
                } else {
                    warn!(
                        name = %name,
                        pinned = %pin.exact_version,
                        observed = %observed_version,
                        "E-30: mcp_pin_version_refused (version mismatch)"
                    );
                    McpPinVerdict::Refused {
                        name: name.to_string(),
                        pinned_version: pin.exact_version.clone(),
                        observed_version: observed_version.to_string(),
                        reason: "version mismatch -- exact-pin policy refuses upgrade",
                    }
                }
            }
            None => {
                if self.policy.enforce_pin_list {
                    warn!(
                        name = %name,
                        observed = %observed_version,
                        "E-30: mcp_pin_version_refused (unpinned server)"
                    );
                    McpPinVerdict::Refused {
                        name: name.to_string(),
                        pinned_version: String::new(),
                        observed_version: observed_version.to_string(),
                        reason: "server not in pin list -- enforce_pin_list is true",
                    }
                } else {
                    McpPinVerdict::Allow
                }
            }
        }
    }
}

/// Returns true when `version` starts with a known semver-range prefix.
fn is_version_float(version: &str) -> bool {
    version.starts_with('^')
        || version.starts_with('~')
        || version.starts_with(">=")
        || version.starts_with("<=")
        || version.starts_with('>')
        || version.starts_with('<')
        || version.contains(" - ") // npm range "1.0.0 - 2.0.0"
        || version.contains("||")
}

/// Returns true when `version` looks like a semver (at least N.N or N.N.N).
fn looks_like_semver(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() < 2 {
        return false;
    }
    // First two parts must be all-digits (third can include pre-release tags
    // like "1.0.0-rc.1" which contain non-digit characters).
    parts[0].chars().all(|c| c.is_ascii_digit())
        && parts[1].chars().all(|c| c.is_ascii_digit())
        && !parts[0].is_empty()
        && !parts[1].is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg_with(pins: Vec<McpServerPin>, enforce_list: bool) -> McpPinsConfig {
        McpPinsConfig {
            policy: McpPinPolicy {
                enforce_pin_list: enforce_list,
                refuse_version_float: true,
                refuse_unauthenticated: true,
            },
            pins,
        }
    }

    fn pin(name: &str, version: &str) -> McpServerPin {
        McpServerPin {
            name: name.to_string(),
            exact_version: version.to_string(),
            expected_pubkey_blake3: String::new(),
            auth_required: true,
        }
    }

    #[test]
    fn version_float_detection() {
        assert!(is_version_float("^1.0.0"));
        assert!(is_version_float("~1.0.0"));
        assert!(is_version_float(">=1.0.0"));
        assert!(is_version_float("<2.0.0"));
        assert!(is_version_float("1.0.0 - 2.0.0"));
        assert!(is_version_float("1.0.0 || 2.0.0"));
        assert!(!is_version_float("1.0.0"));
        assert!(!is_version_float("1.0.0-rc.1"));
    }

    #[test]
    fn semver_detection() {
        assert!(looks_like_semver("1.0.0"));
        assert!(looks_like_semver("1.2"));
        assert!(looks_like_semver("10.20.30"));
        assert!(looks_like_semver("1.0.0-rc.1"));
        assert!(!looks_like_semver("v1.0.0"));
        assert!(!looks_like_semver("notaversion"));
        assert!(!looks_like_semver(""));
    }

    #[test]
    fn validate_refuses_version_float() {
        let cfg = cfg_with(vec![pin("foo", "^1.0.0")], false);
        let err = cfg.validate().expect_err("must refuse ^ float");
        assert!(matches!(err, McpPinsError::VersionFloat { .. }));
    }

    #[test]
    fn validate_refuses_unauthenticated_when_policy_demands() {
        let mut p = pin("foo", "1.0.0");
        p.auth_required = false;
        let cfg = cfg_with(vec![p], false);
        let err = cfg
            .validate()
            .expect_err("must refuse unauthenticated pin");
        assert!(matches!(err, McpPinsError::UnauthenticatedRefused { .. }));
    }

    #[test]
    fn enforce_allow_when_version_matches() {
        let cfg = cfg_with(vec![pin("foo", "1.0.0")], false);
        let v = cfg.enforce_pin("foo", "1.0.0");
        assert_eq!(v, McpPinVerdict::Allow);
    }

    #[test]
    fn enforce_refused_on_version_mismatch() {
        let cfg = cfg_with(vec![pin("foo", "1.0.0")], false);
        let v = cfg.enforce_pin("foo", "1.0.1");
        match v {
            McpPinVerdict::Refused { name, pinned_version, observed_version, .. } => {
                assert_eq!(name, "foo");
                assert_eq!(pinned_version, "1.0.0");
                assert_eq!(observed_version, "1.0.1");
            }
            _ => panic!("expected Refused: {v:?}"),
        }
    }

    #[test]
    fn enforce_allow_unpinned_when_enforce_list_false() {
        let cfg = cfg_with(vec![pin("foo", "1.0.0")], false);
        let v = cfg.enforce_pin("unregistered", "1.0.0");
        assert_eq!(v, McpPinVerdict::Allow);
    }

    #[test]
    fn enforce_refused_unpinned_when_enforce_list_true() {
        let cfg = cfg_with(vec![pin("foo", "1.0.0")], true);
        let v = cfg.enforce_pin("unregistered", "1.0.0");
        match v {
            McpPinVerdict::Refused { name, .. } => assert_eq!(name, "unregistered"),
            _ => panic!("expected Refused on unpinned server with enforce_pin_list=true: {v:?}"),
        }
    }

    #[test]
    fn loads_real_mcp_pins_toml() {
        // Walk up to the workspace root and load the shipped mcp_pins.toml.
        // mirrors mcp_pin_violation_contract.rs path resolution.
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        // crates/plugins -> crates -> .elai_cc -> mcp_pins.toml
        let path = manifest
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("mcp_pins.toml"));
        if let Some(p) = path {
            if p.exists() {
                let cfg = McpPinsConfig::load(&p).expect("real mcp_pins.toml must parse");
                assert!(
                    cfg.policy.refuse_version_float,
                    "shipped policy must refuse version floats"
                );
            }
        }
    }

    #[test]
    fn default_policy_refuses_version_float_and_unauthenticated() {
        let p = McpPinPolicy::default();
        assert!(p.refuse_version_float);
        assert!(p.refuse_unauthenticated);
        assert!(!p.enforce_pin_list);
    }
}
