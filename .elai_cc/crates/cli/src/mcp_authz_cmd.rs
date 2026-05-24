//! C-42 (session 2): `elai mcp authz [status|list-rejections|fuzz-report]`
//!
//! Read-only CLI surface for the Semantic Gateway L2 RBAC policy. Reads
//! `<cwd>/authz-policy.toml` and renders a stable text / JSON report.
//! No live MCP-bridge calls; the bridge wiring already emits the matching
//! `gateway_*` SSB rows when the policy is installed at AppState time.
//!
//! Subcommands:
//!   - `status` — show the loaded policy (default + role names + per-role
//!     tool/owner lists).
//!   - `list-rejections` — placeholder for the SSB read path. In this
//!     session we read the policy file only; the live SSB row read is
//!     wired by the dashboard `gateway_authz_recent_denies` Tauri command
//!     (commands_c42.rs). The CLI variant prints a stable message that
//!     points operators at the dashboard.
//!   - `fuzz-report` — runs a bounded in-process fuzz batch and prints
//!     a summary. The harness itself lives at
//!     plugins/src/authz/fuzz.rs (proptest-state-machine, test-only).
//!     For CLI invocation we run the deterministic invariant tests
//!     synchronously (the I-1..I-4 invariants from the harness header)
//!     and report pass/fail counts.
//!
//! Paper grounding: arXiv:2604.25555 (MEDIUM reliability, single-author
//! preprint). See `workflow/papers/semantic-gateway-fuzzer.md`.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use elai_plugins::authz::{
    check_authz, AuthzPolicy, DocContext,
};

const POLICY_FILE: &str = "authz-policy.toml";

/// Render the default subcommand (`status`) as text.
pub fn render_status_text(cwd: &Path) -> String {
    render_text(cwd, &[])
}

/// Render the default subcommand (`status`) as JSON.
pub fn render_status_json(cwd: &Path) -> Value {
    render_json(cwd, &[])
}

/// Render any authz subcommand as text. `rest` is the tokens after
/// `authz` (e.g. `["status"]`, `["list-rejections"]`, `["fuzz-report"]`).
pub fn render_text(cwd: &Path, rest: &[&str]) -> String {
    match rest.first().copied() {
        None | Some("status") => render_status_inner_text(cwd),
        Some("list-rejections") => render_list_rejections_text(),
        Some("fuzz-report") => render_fuzz_report_text(),
        Some("help") | Some("--help") | Some("-h") => render_usage_text(),
        Some(other) => format!(
            "{}\n  Unexpected       {}",
            render_usage_text(),
            other
        ),
    }
}

/// Render any authz subcommand as JSON.
pub fn render_json(cwd: &Path, rest: &[&str]) -> Value {
    match rest.first().copied() {
        None | Some("status") => render_status_inner_json(cwd),
        Some("list-rejections") => render_list_rejections_json(),
        Some("fuzz-report") => render_fuzz_report_json(),
        Some("help") | Some("--help") | Some("-h") => render_usage_json(None),
        Some(other) => render_usage_json(Some(other)),
    }
}

fn render_usage_text() -> String {
    [
        "MCP Authz (C-42)".to_string(),
        "  Usage            /mcp authz [status|list-rejections|fuzz-report|help]".to_string(),
        "  Direct CLI       elai mcp authz [status|list-rejections|fuzz-report]".to_string(),
        "  Policy           <cwd>/authz-policy.toml".to_string(),
        "  Paper            arXiv:2604.25555 (MEDIUM reliability)".to_string(),
    ]
    .join("\n")
}

fn render_usage_json(unexpected: Option<&str>) -> Value {
    json!({
        "kind": "mcp_authz",
        "action": "help",
        "usage": {
            "slash_command": "/mcp authz [status|list-rejections|fuzz-report|help]",
            "direct_cli": "elai mcp authz [status|list-rejections|fuzz-report]",
            "policy_file": POLICY_FILE,
            "paper": "arXiv:2604.25555",
            "paper_reliability": "MEDIUM",
        },
        "unexpected": unexpected,
    })
}

fn policy_path(cwd: &Path) -> PathBuf {
    cwd.join(POLICY_FILE)
}

fn render_status_inner_text(cwd: &Path) -> String {
    let path = policy_path(cwd);
    match AuthzPolicy::load(&path) {
        Ok(policy) => {
            let mut lines = vec![
                "Authz Status (C-42)".to_string(),
                format!("  Policy file       {}", path.display()),
                format!("  Default           {}", policy.default_disposition()),
                format!("  Roles ({})", policy.role_names().len()),
            ];
            for role in policy.role_names() {
                if let Some(entry) = policy.role(role) {
                    lines.push(format!(
                        "    {}: tools=[{}] doc_owners=[{}]",
                        role,
                        entry.tools.join(", "),
                        entry.doc_owners.join(", "),
                    ));
                }
            }
            lines.push("  Composition       authz -> tool_gate (authz short-circuits)".to_string());
            lines.push(
                "  Notes             L1 DeBERTa-v3 RUNPOD-PENDING; L3 Crypto-HITL DEFERRED-TO-FOLLOWUP".to_string(),
            );
            lines.join("\n")
        }
        Err(e) => format!(
            "Authz Status (C-42)\n  Policy file       {}\n  Status            unavailable ({e})\n  Notes             defaulting to DENY-ALL until {} is created",
            path.display(),
            POLICY_FILE,
        ),
    }
}

fn render_status_inner_json(cwd: &Path) -> Value {
    let path = policy_path(cwd);
    match AuthzPolicy::load(&path) {
        Ok(policy) => {
            let roles: Vec<Value> = policy
                .role_names()
                .into_iter()
                .filter_map(|name| {
                    policy.role(name).map(|entry| {
                        json!({
                            "role": name,
                            "tools": entry.tools,
                            "doc_owners": entry.doc_owners,
                        })
                    })
                })
                .collect();
            json!({
                "kind": "mcp_authz",
                "action": "status",
                "policy_file": path.display().to_string(),
                "default": policy.default_disposition(),
                "role_count": policy.role_names().len(),
                "roles": roles,
                "composition_order": ["authz", "tool_gate"],
                "l1_status": "RUNPOD-PENDING",
                "l3_status": "DEFERRED-TO-FOLLOWUP",
                "paper": "arXiv:2604.25555",
            })
        }
        Err(e) => json!({
            "kind": "mcp_authz",
            "action": "status",
            "policy_file": path.display().to_string(),
            "status": "unavailable",
            "error": e.to_string(),
            "fallback": "DENY-ALL",
        }),
    }
}

fn render_list_rejections_text() -> String {
    [
        "Authz Rejections (C-42)".to_string(),
        "  Source            dashboard SSB stream (gateway_policy_denied rows)".to_string(),
        "  CLI surface       not yet wired to SSB reader here; query the dashboard".to_string(),
        "                    panel under settings > authz to view live rejections.".to_string(),
    ]
    .join("\n")
}

fn render_list_rejections_json() -> Value {
    json!({
        "kind": "mcp_authz",
        "action": "list-rejections",
        "source": "ssb:gateway_policy_denied",
        "cli_status": "not-wired-to-ssb-yet",
        "guidance": "open dashboard settings -> authz panel for live rejections",
    })
}

/// Run a bounded set of deterministic invariant checks (I-1..I-4 from
/// `authz::fuzz`'s module header) against the reference policy and
/// report counts. This is intentionally bounded — the full proptest
/// state-machine harness lives under `cargo test -p elai-plugins
/// authz::fuzz` and is RUNPOD-PENDING for the 500K sequence run.
fn run_bounded_fuzz_checks() -> (u32, u32, Vec<String>) {
    let toml = r#"
default = "deny"

[roles.planner]
tools = ["knowledge.search", "fs.read"]
doc_owners = []

[roles.executor]
tools = ["knowledge.search", "fs.read", "fs.write", "fs.edit"]
doc_owners = ["self"]

[roles.admin]
tools = ["*"]
doc_owners = []
"#;
    let policy = match AuthzPolicy::load_from_str(toml) {
        Ok(p) => p,
        Err(e) => return (0, 1, vec![format!("policy parse failed: {e}")]),
    };
    let mut passed = 0u32;
    let mut failed = 0u32;
    let mut failures = Vec::new();

    // I-1 deny-by-default for undeclared role.
    let v = check_authz("ghost", "fs.read", None, &policy);
    if !v.is_allow() {
        passed += 1;
    } else {
        failed += 1;
        failures.push("I-1: deny-by-default violated for ghost role".to_string());
    }

    // I-2 BOLA: agent_claimed_owner ignored.
    let ctx_claim = DocContext::new(
        "doc-x",
        Some("alice".to_string()),
        Some("executor".to_string()),
    );
    let ctx_noclaim = DocContext::new("doc-x", Some("alice".to_string()), None);
    let with_claim = check_authz("executor", "fs.write", Some(&ctx_claim), &policy);
    let no_claim = check_authz("executor", "fs.write", Some(&ctx_noclaim), &policy);
    if with_claim == no_claim && !with_claim.is_allow() {
        passed += 1;
    } else {
        failed += 1;
        failures.push("I-2: BOLA defense violated (agent claim changed verdict)".to_string());
    }

    // I-3 self-sentinel matches db_owner == role.
    let owned = DocContext::new("doc-x", Some("executor".to_string()), None);
    let v = check_authz("executor", "fs.write", Some(&owned), &policy);
    if v.is_allow() {
        passed += 1;
    } else {
        failed += 1;
        failures.push("I-3: self-sentinel did not permit self-owned doc".to_string());
    }

    // I-4 wildcard tool sentinel permits any tool.
    let v = check_authz("admin", "irreversible.format_disk", None, &policy);
    if v.is_allow() {
        passed += 1;
    } else {
        failed += 1;
        failures.push("I-4: wildcard tool sentinel did not permit arbitrary tool".to_string());
    }

    (passed, failed, failures)
}

fn render_fuzz_report_text() -> String {
    let (passed, failed, failures) = run_bounded_fuzz_checks();
    let mut lines = vec![
        "Authz Fuzz Report (C-42)".to_string(),
        format!("  Invariants        I-1 deny-by-default | I-2 BOLA defense | I-3 self-sentinel | I-4 wildcard"),
        format!("  Passed            {}", passed),
        format!("  Failed            {}", failed),
        format!("  Budget            bounded (4 deterministic checks); full 500K RUNPOD-PENDING"),
    ];
    if !failures.is_empty() {
        lines.push("  Failures".to_string());
        for f in &failures {
            lines.push(format!("    - {f}"));
        }
    }
    lines.push("  Harness           plugins::authz::fuzz (proptest-state-machine, test-only)".to_string());
    lines.join("\n")
}

fn render_fuzz_report_json() -> Value {
    let (passed, failed, failures) = run_bounded_fuzz_checks();
    json!({
        "kind": "mcp_authz",
        "action": "fuzz-report",
        "invariants": ["I-1 deny-by-default", "I-2 BOLA defense", "I-3 self-sentinel", "I-4 wildcard"],
        "passed": passed,
        "failed": failed,
        "failures": failures,
        "budget": "bounded-4-deterministic-checks",
        "full_500k_status": "RUNPOD-PENDING",
        "harness": "plugins::authz::fuzz",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_temp_policy(toml: &str) -> TempDir {
        let dir = TempDir::new().expect("tempdir");
        fs::write(dir.path().join(POLICY_FILE), toml).expect("write policy");
        dir
    }

    #[test]
    fn status_renders_text_with_valid_policy() {
        let dir = write_temp_policy(
            r#"
default = "deny"

[roles.planner]
tools = ["knowledge.search"]
doc_owners = []
"#,
        );
        let out = render_status_text(dir.path());
        assert!(out.contains("Authz Status"));
        assert!(out.contains("Default           deny"));
        assert!(out.contains("planner"));
    }

    #[test]
    fn status_handles_missing_policy_gracefully() {
        let dir = TempDir::new().expect("tempdir");
        let out = render_status_text(dir.path());
        assert!(out.contains("unavailable"));
        assert!(out.contains("DENY-ALL"));
    }

    #[test]
    fn fuzz_report_runs_invariants() {
        let out = render_fuzz_report_text();
        assert!(out.contains("Passed            4"));
        assert!(out.contains("Failed            0"));
    }
}
