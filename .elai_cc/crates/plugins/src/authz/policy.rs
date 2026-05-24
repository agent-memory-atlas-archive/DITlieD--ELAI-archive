//! C-42 (session 2) [`AuthzPolicy`] — declarative role-based access policy
//! loaded from `authz-policy.toml`.
//!
//! Schema:
//!
//! ```toml
//! default = "deny"  # paper-grounded; arXiv:2604.25555 Section 4
//!
//! [roles.planner]
//! tools = ["knowledge.search", "fs.read", "fs.grep"]
//! doc_owners = []
//!
//! [roles.executor]
//! tools = ["knowledge.search", "fs.read", "fs.write", "fs.edit"]
//! doc_owners = ["self"]  # "self" sentinel: only docs owned by this role
//! ```
//!
//! The `doc_owners` list is a list of authoritative-owner patterns the
//! role is permitted to act on. The `"self"` sentinel means "the role
//! name itself must match db_owner" — a basic ownership-self-check that
//! mirrors paper Section 5's BOLA defense (the agent cannot claim
//! ownership; the policy only consults `DocContext::db_owner`). The
//! empty `doc_owners` list means "no doc-context constraint" and the
//! tools are permitted on any document (or no document at all, for tools
//! that do not carry a doc context).

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::types::AuthzError;

/// Canonical name for the policy file at the workspace root. Mirrors the
/// `tool-gate.toml` precedent from C-35.
pub const AUTHZ_POLICY_FILE: &str = "authz-policy.toml";

/// On-disk policy shape. `default` is "allow" or "deny" (deny-all is the
/// paper-grounded recommendation per arXiv:2604.25555 Section 4).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthzPolicyConfig {
    /// Policy default. Required field. Paper-faithful value is `"deny"`.
    pub default: String,
    /// Role declarations. The role name is the key; entry carries the
    /// permitted tools + optional ownership patterns.
    #[serde(default)]
    pub roles: BTreeMap<String, RoleEntry>,
}

/// Per-role policy entry. Tools are matched by exact name (case-sensitive
/// — match the names the MCP bridge registers). The `doc_owners` list is
/// described in [`AuthzPolicy::is_role_owner_permitted`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleEntry {
    /// Tools the role is permitted to invoke. Empty list = no tools (the
    /// role still exists but cannot do anything). Wildcard `"*"` permits
    /// any tool — use sparingly (admin role only).
    #[serde(default)]
    pub tools: Vec<String>,
    /// Ownership patterns. Empty list = no ownership constraint (any
    /// db_owner permitted including None). `"self"` sentinel = role
    /// name must equal db_owner. Literal owner names are permitted (e.g.
    /// `"alice"` permits only docs owned by `alice` in the DB-of-record).
    #[serde(default)]
    pub doc_owners: Vec<String>,
}

/// Wildcard owner sentinel meaning "role name must equal db_owner".
/// Mirrors paper Section 5's authoritative-DB ownership lookup.
pub const OWNER_SENTINEL_SELF: &str = "self";

/// Wildcard tool sentinel — admin role permitted to invoke any registered
/// tool. Use sparingly.
pub const TOOL_SENTINEL_ANY: &str = "*";

/// Loaded + validated authorization policy. Constructed via
/// [`AuthzPolicy::load`] (from file) or [`AuthzPolicy::load_from_str`]
/// (from in-memory TOML text — used by the contract test + fuzz harness).
///
/// All methods on this struct are deterministic and side-effect-free. The
/// [`crate::authz::checker::check_authz`] function reads this struct and
/// emits an [`crate::authz::types::AuthzVerdict`].
#[derive(Debug, Clone)]
pub struct AuthzPolicy {
    inner: AuthzPolicyConfig,
    default_allow: bool,
}

impl AuthzPolicy {
    /// Load + validate a policy from a TOML file. Returns `AuthzError::Io`
    /// when the file cannot be read, `AuthzError::Parse` when the TOML is
    /// malformed, and `AuthzError::InvalidDefault` when `default` is not
    /// one of `"allow"` / `"deny"`.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, AuthzError> {
        let text = std::fs::read_to_string(path.as_ref())?;
        Self::load_from_str(&text)
    }

    /// Same as [`AuthzPolicy::load`] but reads the TOML text from a
    /// `&str`. Used by tests + the in-memory fuzz harness.
    pub fn load_from_str(toml_text: &str) -> Result<Self, AuthzError> {
        let cfg: AuthzPolicyConfig = toml::from_str(toml_text)?;
        let default_allow = match cfg.default.as_str() {
            "allow" => true,
            "deny" => false,
            other => {
                return Err(AuthzError::InvalidDefault {
                    default: other.to_string(),
                })
            }
        };
        Ok(Self {
            inner: cfg,
            default_allow,
        })
    }

    /// Construct a deny-all policy programmatically — useful for tests
    /// and for the fallback when the policy file is missing entirely.
    pub fn deny_all() -> Self {
        Self {
            inner: AuthzPolicyConfig {
                default: "deny".to_string(),
                roles: BTreeMap::new(),
            },
            default_allow: false,
        }
    }

    /// Returns the literal `default` string from the TOML file. Stable
    /// (returns `"allow"` or `"deny"`).
    pub fn default_disposition(&self) -> &str {
        if self.default_allow {
            "allow"
        } else {
            "deny"
        }
    }

    /// True when the configured default is `"allow"`. Paper-grounded
    /// recommendation is `false` (deny-all default).
    pub fn is_default_allow(&self) -> bool {
        self.default_allow
    }

    /// Look up a role entry by name. Returns `None` for unknown roles.
    pub fn role(&self, role: &str) -> Option<&RoleEntry> {
        self.inner.roles.get(role)
    }

    /// True when the given role declares the given tool (or `"*"`).
    /// Returns `false` for unknown roles, for tools the role does not
    /// declare, and for empty tool lists.
    pub fn is_role_tool_permitted(&self, role: &str, tool: &str) -> bool {
        let Some(entry) = self.role(role) else {
            return false;
        };
        entry
            .tools
            .iter()
            .any(|t| t == tool || t == TOOL_SENTINEL_ANY)
    }

    /// True when the given role is permitted to act on a document with the
    /// given authoritative owner (from [`crate::authz::types::DocContext::db_owner`]).
    ///
    /// Rules:
    ///   - empty `doc_owners` list = no constraint (any owner permitted,
    ///     including `None`).
    ///   - `"self"` matches when `db_owner == Some(role_name)`.
    ///   - literal pattern matches when `db_owner == Some(pattern)`.
    ///   - if `db_owner == None` and `doc_owners` is non-empty, deny.
    ///
    /// `agent_claimed_owner` is NEVER consulted here — BOLA defense per
    /// arXiv:2604.25555 Section 5.
    pub fn is_role_owner_permitted(
        &self,
        role: &str,
        db_owner: Option<&str>,
    ) -> bool {
        let Some(entry) = self.role(role) else {
            return false;
        };
        if entry.doc_owners.is_empty() {
            // No ownership constraint — tool-level permission decides.
            return true;
        }
        let Some(owner) = db_owner else {
            // Constraint declared but no DB owner — deny.
            return false;
        };
        entry.doc_owners.iter().any(|pat| {
            if pat == OWNER_SENTINEL_SELF {
                owner == role
            } else {
                owner == pat
            }
        })
    }

    /// Read-through to the raw config for tests / tooling that need the
    /// full role map.
    pub fn config(&self) -> &AuthzPolicyConfig {
        &self.inner
    }

    /// List role names in declaration order (alphabetical, since the
    /// underlying map is a BTreeMap). Used by the CLI `authz status`
    /// subcommand.
    pub fn role_names(&self) -> Vec<&str> {
        self.inner.roles.keys().map(String::as_str).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DENY_ALL_POLICY: &str = r#"
default = "deny"

[roles.planner]
tools = ["knowledge.search", "fs.read"]
doc_owners = []

[roles.executor]
tools = ["knowledge.search", "fs.write", "fs.edit"]
doc_owners = ["self"]

[roles.admin]
tools = ["*"]
doc_owners = []
"#;

    #[test]
    fn load_deny_all_policy_succeeds() {
        let policy = AuthzPolicy::load_from_str(DENY_ALL_POLICY).expect("load");
        assert_eq!(policy.default_disposition(), "deny");
        assert!(!policy.is_default_allow());
        assert_eq!(policy.role_names(), vec!["admin", "executor", "planner"]);
    }

    #[test]
    fn invalid_default_rejected() {
        let bad = r#"default = "yolo""#;
        let err = AuthzPolicy::load_from_str(bad).expect_err("must reject");
        assert!(matches!(err, AuthzError::InvalidDefault { .. }));
    }

    #[test]
    fn role_tool_check_works() {
        let policy = AuthzPolicy::load_from_str(DENY_ALL_POLICY).expect("load");
        assert!(policy.is_role_tool_permitted("planner", "knowledge.search"));
        assert!(!policy.is_role_tool_permitted("planner", "fs.write"));
        assert!(!policy.is_role_tool_permitted("unknown_role", "fs.read"));
        // Wildcard tool grants any.
        assert!(policy.is_role_tool_permitted("admin", "any.tool"));
    }

    #[test]
    fn role_owner_self_sentinel() {
        let policy = AuthzPolicy::load_from_str(DENY_ALL_POLICY).expect("load");
        assert!(policy.is_role_owner_permitted("executor", Some("executor")));
        assert!(!policy.is_role_owner_permitted("executor", Some("alice")));
        // Empty doc_owners on planner = no constraint.
        assert!(policy.is_role_owner_permitted("planner", Some("alice")));
        assert!(policy.is_role_owner_permitted("planner", None));
        // Unknown role denies.
        assert!(!policy.is_role_owner_permitted("ghost", Some("alice")));
        // Constraint + no db_owner denies.
        assert!(!policy.is_role_owner_permitted("executor", None));
    }

    #[test]
    fn deny_all_constructor_matches_paper() {
        let policy = AuthzPolicy::deny_all();
        assert_eq!(policy.default_disposition(), "deny");
        assert!(!policy.is_role_tool_permitted("planner", "fs.read"));
    }
}
