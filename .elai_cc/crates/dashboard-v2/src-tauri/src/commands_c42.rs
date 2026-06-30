//! C-42 (session 2): Semantic Gateway L2 RBAC dashboard surface.
//!
//! cross-lang-pair: dashboard/tauri/api
//!
//! Three read-only commands feed the Svelte `AuthzPanel`:
//!
//! - [`gateway_authz_status`] — current loaded policy (default + role
//!   names + per-role tool/owner lists). Reads
//!   `<project_root>/authz-policy.toml`. Returns the "unavailable"
//!   envelope (with `note` populated) when the policy file is absent or
//!   parse-fails.
//! - [`gateway_authz_recent_denies`] — recent `gateway_policy_denied`
//!   SSB rows. Window-bounded; default 60s, capped at 5 minutes.
//! - [`gateway_authz_run_fuzz`] — runs a bounded set of deterministic
//!   invariant checks (I-1..I-4 from the fuzz harness) against the
//!   reference policy and returns pass/fail counts. The full
//!   proptest-state-machine run is test-only.
//!
//! Additive — runs alongside the existing `commands::*` /
//! `commands_d*::*` / `commands_c35::*` handlers without touching them.
//! Sibling of `commands_c35` (the C-35 ToolGateInspector backend).
//!
//! Paper grounding: arXiv:2604.25555 (MEDIUM reliability, single-author
//! preprint). See `workflow/papers/semantic-gateway-fuzzer.md`.

use std::cmp::Ordering;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_DENY_LIMIT: usize = 200;
const MAX_DENY_LIMIT: usize = 2000;
const DEFAULT_DENY_WINDOW_MS: i64 = 60_000;
const MAX_DENY_WINDOW_MS: i64 = 5 * 60 * 1000;

/// Wire shape consumed by `AuthzPanel.svelte`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthzStatusDto {
    /// Filesystem path of the policy file (always populated, even when
    /// the file is absent — the frontend renders the path so operators
    /// know where to create it).
    pub policy_file: String,
    /// Loaded policy default ("allow" / "deny" / "unavailable").
    pub default: String,
    /// Per-role rows. Empty when policy is unavailable.
    pub roles: Vec<AuthzRoleDto>,
    /// Whether the plugins crate was built with `feature = "mcp"`.
    pub feature_enabled: bool,
    /// Human-readable note. Surfaces RUNPOD-PENDING / DEFERRED-TO-FOLLOWUP
    /// status for L1 and L3.
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthzRoleDto {
    pub role: String,
    pub tools: Vec<String>,
    pub doc_owners: Vec<String>,
}

/// One denial row mapped from `gateway_policy_denied` SSB.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthzDenyRowDto {
    /// Blake3-prefix u64 of the role name (paper-grounded hashing —
    /// dashboards group without surfacing raw role string).
    pub role_hash: u64,
    pub tool_hash: u64,
    /// "authz" or "tool_gate" — which layer denied.
    pub deny_reason_label: String,
    pub deny_reason_hash: u64,
    pub ts_ms: i64,
}

/// Result of a bounded fuzz batch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthzFuzzReportDto {
    pub passed: u32,
    pub failed: u32,
    pub failures: Vec<String>,
    pub budget: String,
    pub full_500k_status: String,
    pub paper: String,
}

#[cfg(not(feature = "mcp"))]
const NOTE_UNAVAILABLE: &str =
    "Authz panel unavailable: dashboard built without the `mcp` feature. \
     Rebuild with --features mcp to activate the C-42 zero-trust gateway.";

#[cfg(feature = "mcp")]
const NOTE_LOADED: &str =
    "Authz policy loaded. L1 DeBERTa-v3 RUNPOD-PENDING; L3 Crypto-HITL DEFERRED-TO-FOLLOWUP. \
     Composition: authz -> tool_gate (authz short-circuits on deny).";

#[cfg(feature = "mcp")]
const NOTE_MISSING: &str =
    "authz-policy.toml not present at the project root. \
     The composed gate defaults to DENY-ALL until the file is created.";

/// Return the loaded authz policy status. The frontend polls this every
/// 10s and renders the table.
#[tauri::command]
pub async fn gateway_authz_status(
    _state: State<'_, AppState>,
) -> Result<AuthzStatusDto, String> {
    Ok(status_inner())
}

/// Return the last N `gateway_policy_denied` SSB rows. Reads via the
/// shared `SsbViewer` exactly like `list_tool_gate_rows` (commands_c35).
#[tauri::command]
pub async fn gateway_authz_recent_denies(
    state: State<'_, AppState>,
    window_ms: Option<i64>,
    limit: Option<usize>,
) -> Result<Vec<AuthzDenyRowDto>, String> {
    let limit = limit.unwrap_or(DEFAULT_DENY_LIMIT).min(MAX_DENY_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(Vec::new()),
    };
    let mut rows: Vec<AuthzDenyRowDto> = match viewer
        .recent_rows_of_type("gateway_policy_denied", limit)
    {
        Ok(details) => details
            .into_iter()
            .filter_map(detail_to_deny_row)
            .collect(),
        Err(e) => {
            tracing::warn!(
                action_type = "gateway_policy_denied",
                error = %e,
                "gateway_authz_recent_denies: viewer error"
            );
            Vec::new()
        }
    };
    // Newest first by ts_ms (recent_rows_of_type returns newest first
    // already; harmless re-sort makes the contract explicit).
    rows.sort_by(|a, b| b.ts_ms.cmp(&a.ts_ms).then(Ordering::Equal));
    let window = window_ms
        .unwrap_or(DEFAULT_DENY_WINDOW_MS)
        .clamp(0, MAX_DENY_WINDOW_MS);
    if window > 0 {
        let cutoff = now_ms().saturating_sub(window);
        rows.retain(|r| r.ts_ms >= cutoff);
    }
    rows.truncate(limit);
    Ok(rows)
}

/// Run a bounded set of deterministic invariant checks against the
/// reference policy and return pass/fail counts.
#[tauri::command]
pub async fn gateway_authz_run_fuzz(
    _state: State<'_, AppState>,
) -> Result<AuthzFuzzReportDto, String> {
    Ok(fuzz_report_inner())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now() // entropy-allow: free-standing helper backing gateway_authz deny-row timestamps, no AppState/ClockProvider in scope at this fn
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

fn detail_to_deny_row(d: elai_orchestrator::SsbActionDetail) -> Option<AuthzDenyRowDto> {
    let obj = d.payload_json.as_object()?;
    Some(AuthzDenyRowDto {
        role_hash: obj.get("role_hash")?.as_u64().unwrap_or(0),
        tool_hash: obj.get("tool_hash")?.as_u64().unwrap_or(0),
        deny_reason_label: obj
            .get("deny_reason_label")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        deny_reason_hash: obj.get("deny_reason_hash")?.as_u64().unwrap_or(0),
        ts_ms: d.ts,
    })
}

#[cfg(feature = "mcp")]
fn status_inner() -> AuthzStatusDto {
    let path = policy_path();
    match elai_plugins::authz::AuthzPolicy::load(&path) {
        Ok(policy) => {
            let roles = policy
                .role_names()
                .into_iter()
                .filter_map(|name| {
                    policy.role(name).map(|entry| AuthzRoleDto {
                        role: name.to_string(),
                        tools: entry.tools.clone(),
                        doc_owners: entry.doc_owners.clone(),
                    })
                })
                .collect();
            AuthzStatusDto {
                policy_file: path.display().to_string(),
                default: policy.default_disposition().to_string(),
                roles,
                feature_enabled: true,
                note: NOTE_LOADED.to_string(),
            }
        }
        Err(_) => AuthzStatusDto {
            policy_file: path.display().to_string(),
            default: "unavailable".to_string(),
            roles: Vec::new(),
            feature_enabled: true,
            note: NOTE_MISSING.to_string(),
        },
    }
}

#[cfg(not(feature = "mcp"))]
fn status_inner() -> AuthzStatusDto {
    AuthzStatusDto {
        policy_file: policy_path().display().to_string(),
        default: "unavailable".to_string(),
        roles: Vec::new(),
        feature_enabled: false,
        note: NOTE_UNAVAILABLE.to_string(),
    }
}

fn policy_path() -> PathBuf {
    // Project root resolution mirrors the C-35 tool-gate manifest path:
    // env override first, then the canonical workspace-root file name.
    std::env::var("ELAI_AUTHZ_POLICY")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("authz-policy.toml"))
}

#[cfg(feature = "mcp")]
fn fuzz_report_inner() -> AuthzFuzzReportDto {
    use elai_plugins::authz::{check_authz, AuthzPolicy, DocContext};

    let toml = r#"
default = "deny"

[roles.planner]
tools = ["knowledge.search", "fs.read"]
doc_owners = []

[roles.executor]
tools = ["knowledge.search", "fs.read", "fs.write"]
doc_owners = ["self"]

[roles.admin]
tools = ["*"]
doc_owners = []
"#;
    let policy = match AuthzPolicy::load_from_str(toml) {
        Ok(p) => p,
        Err(e) => {
            return AuthzFuzzReportDto {
                passed: 0,
                failed: 1,
                failures: vec![format!("policy parse failed: {e}")],
                budget: "bounded-4".to_string(),
                full_500k_status: "RUNPOD-PENDING".to_string(),
                paper: "arXiv:2604.25555".to_string(),
            }
        }
    };

    let mut passed = 0u32;
    let mut failed = 0u32;
    let mut failures = Vec::new();

    // I-1
    let v = check_authz("ghost", "fs.read", None, &policy);
    if !v.is_allow() {
        passed += 1;
    } else {
        failed += 1;
        failures.push("I-1 deny-by-default".to_string());
    }

    // I-2 BOLA
    let with_claim = DocContext::new(
        "x",
        Some("alice".to_string()),
        Some("executor".to_string()),
    );
    let no_claim = DocContext::new("x", Some("alice".to_string()), None);
    let v1 = check_authz("executor", "fs.write", Some(&with_claim), &policy);
    let v2 = check_authz("executor", "fs.write", Some(&no_claim), &policy);
    if v1 == v2 && !v1.is_allow() {
        passed += 1;
    } else {
        failed += 1;
        failures.push("I-2 BOLA defense".to_string());
    }

    // I-3 self-sentinel
    let owned = DocContext::new("x", Some("executor".to_string()), None);
    let v = check_authz("executor", "fs.write", Some(&owned), &policy);
    if v.is_allow() {
        passed += 1;
    } else {
        failed += 1;
        failures.push("I-3 self-sentinel".to_string());
    }

    // I-4 wildcard
    let v = check_authz("admin", "irreversible.format_disk", None, &policy);
    if v.is_allow() {
        passed += 1;
    } else {
        failed += 1;
        failures.push("I-4 wildcard".to_string());
    }

    AuthzFuzzReportDto {
        passed,
        failed,
        failures,
        budget: "bounded-4-deterministic-checks".to_string(),
        full_500k_status: "RUNPOD-PENDING".to_string(),
        paper: "arXiv:2604.25555".to_string(),
    }
}

#[cfg(not(feature = "mcp"))]
fn fuzz_report_inner() -> AuthzFuzzReportDto {
    AuthzFuzzReportDto {
        passed: 0,
        failed: 0,
        failures: vec!["mcp feature off; fuzz harness unavailable".to_string()],
        budget: "feature-off".to_string(),
        full_500k_status: "RUNPOD-PENDING".to_string(),
        paper: "arXiv:2604.25555".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_ms_is_positive() {
        assert!(now_ms() > 0);
    }

    #[test]
    fn policy_path_defaults_to_authz_policy_toml() {
        // Env override clears in this test, so the default path is used.
        std::env::remove_var("ELAI_AUTHZ_POLICY");
        let p = policy_path();
        assert_eq!(p.file_name().and_then(|s| s.to_str()), Some("authz-policy.toml"));
    }

    #[cfg(feature = "mcp")]
    #[test]
    fn fuzz_report_inner_all_pass() {
        let r = fuzz_report_inner();
        assert_eq!(r.passed, 4);
        assert_eq!(r.failed, 0);
    }
}
