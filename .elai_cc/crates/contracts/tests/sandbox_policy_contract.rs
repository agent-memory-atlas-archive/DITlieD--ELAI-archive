//! Phase B1 — SandboxPolicy + SandboxAdapter contract.
//!
//! Pins:
//!   (a) Default policy denies destructive bash commands (e.g. "rm -rf /").
//!   (b) NetworkAccess is denied unless the host is in the policy
//!       allow-list.
//!   (c) FileWrite outside `fs_writable_paths` is denied.
//!   (d) Same SandboxAdapter installed on two distinct
//!       `InProcessToolDispatcher` configurations (one mimicking a
//!       "local provider" wiring, one mimicking a "cloud provider"
//!       wiring) produces identical denial verdicts at the
//!       `dispatch_git_commit` chokepoint. Provider-source-agnostic.
//!
//! Runtime: `#[tokio::test]` with the multi-thread scheduler so
//! `dispatch_git_commit` (spawn_blocking) has a worker to land on.

use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;

use elai_core::tool_dispatcher::{
    GitBranchPolicy, GitCommitRequest, ToolDispatchError, ToolDispatcher,
};
use elai_orchestrator::{
    PassthroughSandbox, SandboxAdapter, SandboxDenied, SandboxOp, SandboxPolicy,
    SandboxProfile,
};
use elai_tools::dispatcher::InProcessToolDispatcher;
use elai_tools::tiered_validation::{
    DefaultTieredValidator, DeterminismCommandRunner, IntegrationCommandRunner,
    IntegrationRunOutcome, SyntaxCommandRunner, TieredValidator,
};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Minimal scripted validator runners — enough to construct an
// InProcessToolDispatcher. The policy hooks fire before the validators
// see anything, so the runner verdicts are irrelevant here.
// ---------------------------------------------------------------------------

struct PassIntegration;
impl IntegrationCommandRunner for PassIntegration {
    fn run_integration<'a>(
        &'a self,
        _level: &'a str,
        _pd: &'a Path,
    ) -> Pin<Box<dyn Future<Output = IntegrationRunOutcome> + Send + 'a>> {
        Box::pin(async move {
            IntegrationRunOutcome {
                passed: true,
                rules_fired: 0,
                stderr_snippet: None,
            }
        })
    }
}
struct PassSyntax;
impl SyntaxCommandRunner for PassSyntax {
    fn run_syntax<'a>(
        &'a self,
        _l: &'a str,
        _pd: &'a Path,
        _cf: &'a [PathBuf],
    ) -> Pin<Box<dyn Future<Output = IntegrationRunOutcome> + Send + 'a>> {
        Box::pin(async move {
            IntegrationRunOutcome {
                passed: true,
                rules_fired: 0,
                stderr_snippet: None,
            }
        })
    }
}
struct PassDeterminism;
impl DeterminismCommandRunner for PassDeterminism {
    fn run_determinism<'a>(
        &'a self,
        _pd: &'a Path,
    ) -> Pin<Box<dyn Future<Output = IntegrationRunOutcome> + Send + 'a>> {
        Box::pin(async move {
            IntegrationRunOutcome {
                passed: true,
                rules_fired: 0,
                stderr_snippet: None,
            }
        })
    }
}

fn build_validator() -> Arc<dyn TieredValidator> {
    let v = DefaultTieredValidator::new(
        Arc::new(PassDeterminism),
        Arc::new(PassSyntax),
        Arc::new(PassIntegration),
    );
    Arc::new(v) as Arc<dyn TieredValidator>
}

// ---------------------------------------------------------------------------
// (a) Default policy denies destructive bash commands.
// ---------------------------------------------------------------------------

#[test]
fn default_policy_denies_rm_rf_root() {
    let sandbox = PassthroughSandbox::with_default_policy();
    let cmd = "rm -rf /".to_string();
    let args: Vec<String> = Vec::new();
    let op = SandboxOp::BashCommand {
        cmd: &cmd,
        args: &args,
    };
    let verdict = sandbox.check_policy(&op);
    assert!(
        verdict.is_err(),
        "default policy must deny `rm -rf /`; verdict={:?}",
        verdict
    );
}

// B1 fixer regression test: an attacker that splits the `rm` flag
// across multiple short-flag tokens (`["rm","-r","-f","/"]`) used to
// evade the destructive-command regex because the regex required
// `r` and `f` to co-occur inside the same `-rf*` token. The argv-
// level matcher in `check_bash` now collects short-flag chars across
// every short-flag token, so the AND-of-rf check fires regardless of
// flag layout. Also pins the long-flag and double-slash variants the
// auditor confirmed empirically as evasion vectors.
#[test]
fn default_policy_denies_rm_argv_flag_split() {
    let sandbox = PassthroughSandbox::with_default_policy();
    let cmd = "rm".to_string();
    let args: Vec<String> = vec!["-r".to_string(), "-f".to_string(), "/".to_string()];
    let op = SandboxOp::BashCommand {
        cmd: &cmd,
        args: &args,
    };
    let verdict = sandbox.check_policy(&op);
    assert!(
        verdict.is_err(),
        "argv-split `rm -r -f /` must be denied (was regex-evading); verdict={:?}",
        verdict
    );
}

#[test]
fn default_policy_denies_rm_long_flags() {
    let sandbox = PassthroughSandbox::with_default_policy();
    let cmd = "rm".to_string();
    let args: Vec<String> = vec![
        "--recursive".to_string(),
        "--force".to_string(),
        "/".to_string(),
    ];
    let op = SandboxOp::BashCommand {
        cmd: &cmd,
        args: &args,
    };
    assert!(
        sandbox.check_policy(&op).is_err(),
        "`rm --recursive --force /` must be denied"
    );
}

#[test]
fn default_policy_denies_rm_double_slash_root() {
    let sandbox = PassthroughSandbox::with_default_policy();
    let cmd = "rm".to_string();
    let args: Vec<String> = vec!["-rf".to_string(), "//".to_string()];
    let op = SandboxOp::BashCommand {
        cmd: &cmd,
        args: &args,
    };
    assert!(
        sandbox.check_policy(&op).is_err(),
        "`rm -rf //` (slash-collapsed root) must be denied"
    );
}

#[test]
fn default_policy_denies_dd_zero_device() {
    let sandbox = PassthroughSandbox::with_default_policy();
    let cmd = "dd if=/dev/zero of=/dev/sda".to_string();
    let args: Vec<String> = Vec::new();
    let op = SandboxOp::BashCommand {
        cmd: &cmd,
        args: &args,
    };
    assert!(
        sandbox.check_policy(&op).is_err(),
        "default policy must deny `dd if=...`"
    );
}

// ---------------------------------------------------------------------------
// (b) NetworkAccess denied unless host in allow-list.
// ---------------------------------------------------------------------------

#[test]
fn network_access_denied_when_host_not_allowlisted() {
    let sandbox = PassthroughSandbox::with_default_policy();
    let op = SandboxOp::NetworkAccess { host: "evil.com" };
    assert!(
        sandbox.check_policy(&op).is_err(),
        "default empty allow-list must deny `evil.com`"
    );
}

#[test]
fn network_access_allowed_when_host_in_allowlist() {
    let policy = SandboxPolicy {
        profile: SandboxProfile::WritableProjectDir,
        network_allow_list: vec!["api.anthropic.com".to_string()],
        fs_writable_paths: vec![std::env::temp_dir()],
    };
    let sandbox = PassthroughSandbox::new(policy);
    let op = SandboxOp::NetworkAccess {
        host: "api.anthropic.com",
    };
    assert!(
        sandbox.check_policy(&op).is_ok(),
        "host in allow-list must be permitted"
    );
}

// ---------------------------------------------------------------------------
// (c) FileWrite outside fs_writable_paths is denied.
// ---------------------------------------------------------------------------

#[test]
fn file_write_outside_writable_paths_is_denied() {
    let tmp = TempDir::new().expect("tempdir");
    let policy = SandboxPolicy {
        profile: SandboxProfile::WritableProjectDir,
        network_allow_list: Vec::new(),
        fs_writable_paths: vec![tmp.path().to_path_buf()],
    };
    let sandbox = PassthroughSandbox::new(policy);

    let outside = if cfg!(windows) {
        PathBuf::from(r"C:\Windows\System32\hosts")
    } else {
        PathBuf::from("/etc/passwd")
    };
    let op = SandboxOp::FileWrite {
        path: outside.as_path(),
    };
    assert!(
        sandbox.check_policy(&op).is_err(),
        "FileWrite outside writable paths must be denied: {}",
        outside.display()
    );

    // Sanity: writes INSIDE the writable path are allowed.
    let inside = tmp.path().join("child.txt");
    let inside_op = SandboxOp::FileWrite {
        path: inside.as_path(),
    };
    assert!(
        sandbox.check_policy(&inside_op).is_ok(),
        "FileWrite inside fs_writable_paths must be allowed"
    );
}

// ---------------------------------------------------------------------------
// (d) Provider-agnostic denial — the same SandboxAdapter installed on
// two distinct InProcessToolDispatcher instances produces identical
// denial verdicts at the dispatch_git_commit chokepoint.
//
// "Local-provider" and "cloud-provider" are simulated as two distinct
// validator wirings on top of an InProcessToolDispatcher; the policy
// sits one layer below the provider boundary, so source-of-truth
// equivalence is what we pin.
// ---------------------------------------------------------------------------

/// Always-deny test adapter. Used to prove that an installed
/// `SandboxAdapter` short-circuits every dispatch entry point regardless
/// of which provider routed the call.
struct AlwaysDenySandbox;

impl SandboxAdapter for AlwaysDenySandbox {
    fn check_policy(&self, _op: &SandboxOp) -> Result<(), SandboxDenied> {
        Err(SandboxDenied {
            reason: "test: always-deny".to_string(),
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dispatch_git_commit_denied_on_both_providers_with_shared_adapter() {
    let shared: Arc<dyn SandboxAdapter> = Arc::new(AlwaysDenySandbox);

    // "Local provider" dispatcher.
    let local_validator = build_validator();
    let local = InProcessToolDispatcher::new(local_validator)
        .with_sandbox(Arc::clone(&shared));

    // "Cloud provider" dispatcher — a separate dispatcher instance,
    // independently constructed, but sharing the same SandboxAdapter
    // Arc handle. This is the configuration the brief calls out.
    let cloud_validator = build_validator();
    let cloud = InProcessToolDispatcher::new(cloud_validator)
        .with_sandbox(Arc::clone(&shared));

    let tmp = TempDir::new().expect("tempdir");
    let req = || GitCommitRequest {
        anchor_path: tmp.path().to_path_buf(),
        files: vec![PathBuf::from("Cargo.toml")],
        message: "ci: test".to_string(),
        branch_policy: GitBranchPolicy::Main,
        plan_id: "test".to_string(),
        step_id: "b1-contract".to_string(),
    };

    let local_err = local
        .dispatch_git_commit(req())
        .await
        .expect_err("policy denies the commit on the local-provider dispatcher");
    let cloud_err = cloud
        .dispatch_git_commit(req())
        .await
        .expect_err("policy denies the commit on the cloud-provider dispatcher");

    match local_err {
        ToolDispatchError::Denied { reason } => {
            assert!(
                reason.contains("always-deny") || reason.contains("test"),
                "local denial reason must mention the adapter; got: {reason}"
            );
        }
        other => panic!("expected ToolDispatchError::Denied on local; got: {other:?}"),
    }
    match cloud_err {
        ToolDispatchError::Denied { reason } => {
            assert!(
                reason.contains("always-deny") || reason.contains("test"),
                "cloud denial reason must mention the adapter; got: {reason}"
            );
        }
        other => panic!("expected ToolDispatchError::Denied on cloud; got: {other:?}"),
    }
}
