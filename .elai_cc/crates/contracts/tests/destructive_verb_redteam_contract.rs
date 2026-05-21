// E-30: Red-team contract for the non-LLM argv destructive-verb allowlist.
//
// Pins the contract that `check_argv()` correctly classifies 15 adversarial
// argv slices (8 Deny, 5 RequireHumanApproval, 2 Allow) covering all 6
// public-incident patterns documented in E-30_public_agent_incidents.md.
//
// Reference incidents:
//   - Replit DROP TABLE (incidentdatabase.ai/cite/1152)
//   - Cursor `git restore .` mass-file deletion
//   - Claude Code `rm -rf ~/` home directory wipe
//   - Kiro `rm -rf node_modules env_file` env file delete
//   - DataTalks `terraform destroy` production infra wipe
//   - postmark-mcp v1.0.16 BCC backdoor (supply chain; tested via MCP pin tests)

use std::ffi::OsString;
use elai_safety::{check_argv, AllowlistVerdict};

fn args(v: &[&str]) -> Vec<OsString> {
    v.iter().map(|s| OsString::from(s)).collect()
}

// ---------------------------------------------------------------------------
// DENY cases: must never execute, even with human approval
// ---------------------------------------------------------------------------

#[test]
fn deny_rm_rf_home_dir() {
    // Claude Code incident: `rm -rf ~/`
    let result = check_argv(&args(&["rm", "-rf", "~/"]));
    assert!(
        matches!(result, AllowlistVerdict::Deny { .. }),
        "rm -rf ~/ must be Deny; got {result:?}"
    );
}

#[test]
fn deny_rm_rf_home_dir_expanded() {
    // Variant: shell-expanded home dir prefix `/home/user`
    let result = check_argv(&args(&["rm", "-rf", "/home/"]));
    assert!(
        matches!(result, AllowlistVerdict::Deny { .. }),
        "rm -rf /home/ must be Deny; got {result:?}"
    );
}

#[test]
fn deny_git_restore_dot() {
    // Cursor incident: `git restore .`
    let result = check_argv(&args(&["git", "restore", "."]));
    assert!(
        matches!(result, AllowlistVerdict::Deny { .. }),
        "git restore . must be Deny; got {result:?}"
    );
}

#[test]
fn deny_git_restore_staged_dot() {
    // Variant: `git restore --staged .`
    let result = check_argv(&args(&["git", "restore", "--staged", "."]));
    assert!(
        matches!(result, AllowlistVerdict::Deny { .. }),
        "git restore --staged . must be Deny; got {result:?}"
    );
}

#[test]
fn deny_git_reset_hard_non_head() {
    // Dangerous: hard reset to a specific commit drops uncommitted work.
    // s-68264: regex DENY_PATTERNS[2] enforces 7+ hex chars (real git
    // shorthand min), so test SHA bumped from 6 chars to a realistic 7.
    let result = check_argv(&args(&["git", "reset", "--hard", "abc1234"]));
    assert!(
        matches!(result, AllowlistVerdict::Deny { .. }),
        "git reset --hard <sha> must be Deny; got {result:?}"
    );
}

#[test]
fn deny_drop_database_sql() {
    // Replit incident: DROP DATABASE via psql
    let result = check_argv(&args(&["psql", "-c", "DROP DATABASE prod;"]));
    assert!(
        matches!(result, AllowlistVerdict::Deny { .. }),
        "psql -c DROP DATABASE must be Deny; got {result:?}"
    );
}

#[test]
fn deny_chmod_777_root() {
    // Kiro incident: `chmod 777 /`
    let result = check_argv(&args(&["chmod", "777", "/"]));
    assert!(
        matches!(result, AllowlistVerdict::Deny { .. }),
        "chmod 777 / must be Deny; got {result:?}"
    );
}

#[test]
fn deny_git_stash_drops_work() {
    // git stash alone (no pop) drops staged + unstaged changes silently
    let result = check_argv(&args(&["git", "stash"]));
    assert!(
        matches!(result, AllowlistVerdict::Deny { .. }),
        "bare git stash must be Deny; got {result:?}"
    );
}

// ---------------------------------------------------------------------------
// REQUIRE_HUMAN_APPROVAL cases: require confirmation, not outright blocked
// ---------------------------------------------------------------------------

#[test]
fn approval_terraform_destroy() {
    // DataTalks incident: `terraform destroy`
    let result = check_argv(&args(&["terraform", "destroy"]));
    assert!(
        matches!(result, AllowlistVerdict::RequireHumanApproval { .. }),
        "terraform destroy must be RequireHumanApproval; got {result:?}"
    );
}

#[test]
fn approval_tofu_destroy() {
    // OpenTofu variant
    let result = check_argv(&args(&["tofu", "destroy"]));
    assert!(
        matches!(result, AllowlistVerdict::RequireHumanApproval { .. }),
        "tofu destroy must be RequireHumanApproval; got {result:?}"
    );
}

#[test]
fn approval_aws_delete_stack() {
    let result = check_argv(&args(&["aws", "cloudformation", "delete-stack"]));
    assert!(
        matches!(result, AllowlistVerdict::RequireHumanApproval { .. }),
        "aws delete-stack must be RequireHumanApproval; got {result:?}"
    );
}

#[test]
fn approval_kubectl_delete_namespace() {
    let result = check_argv(&args(&["kubectl", "delete", "namespace", "production"]));
    assert!(
        matches!(result, AllowlistVerdict::RequireHumanApproval { .. }),
        "kubectl delete namespace must be RequireHumanApproval; got {result:?}"
    );
}

#[test]
fn approval_docker_rm_force() {
    let result = check_argv(&args(&["docker", "rm", "-fv", "my-container"]));
    assert!(
        matches!(result, AllowlistVerdict::RequireHumanApproval { .. }),
        "docker rm -fv must be RequireHumanApproval; got {result:?}"
    );
}

// ---------------------------------------------------------------------------
// ALLOW cases: common safe commands must pass through
// ---------------------------------------------------------------------------

#[test]
fn allow_git_status() {
    let result = check_argv(&args(&["git", "status"]));
    assert!(
        matches!(result, AllowlistVerdict::Allow),
        "git status must be Allow; got {result:?}"
    );
}

#[test]
fn allow_cargo_build() {
    let result = check_argv(&args(&["cargo", "build", "--workspace"]));
    assert!(
        matches!(result, AllowlistVerdict::Allow),
        "cargo build must be Allow; got {result:?}"
    );
}
