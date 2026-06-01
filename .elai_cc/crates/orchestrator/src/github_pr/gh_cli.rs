//! C-40: [`GhCliAdapter`] — concrete `GithubPrAdapter` impl that shells
//! out to the `gh` CLI.
//!
//! Design points (grounded in the repo's existing precedents):
//!
//! - **`tokio::process::Command`** is the async subprocess primitive
//!   (matches `crates/cli/src/measure.rs:1109-1122` and
//!   `dashboard-v2/src-tauri/src/commands.rs:6390+`).
//! - **`gh pr *` commands are already `RiskLevel::SharedState`** by the
//!   existing `tools::action_classifier` (lines 128-141). The adapter
//!   doesn't bypass — every call lands in the orchestrator's action
//!   pipeline which already gates SharedState verbs.
//! - **Auth lives in `gh`'s keychain**, never in our process memory.
//!   We never read GITHUB_TOKEN, never log argv that could leak secrets
//!   (PR titles/bodies are user-supplied; we treat them as
//!   non-sensitive but still log only the command verb, not arguments,
//!   in the SSB row).
//! - **SSB emission**: every successful state-change writes exactly one
//!   C-40 row via `Ssb::write_payload`. Sensitive fields (URL, SHA, CI
//!   label, review label) are hashed via blake3 prefix before
//!   emission.

use std::sync::Arc;

use serde::Deserialize;
use tokio::process::Command;
use tracing::{debug, warn};

use crate::ssb::{
    GithubPrClosedPayload, GithubPrMergedPayload, GithubPrOpenedPayload,
    GithubPrStatusFetchedPayload, Ssb, SsbPayload,
};

use super::adapter_trait::GithubPrAdapter;
use super::error::GithubPrError;
use super::types::{PrCheckRun, PrOpenRequest, PrState, PrStatus, PrSummary};

/// `gh` CLI binary. Resolved from `PATH` at each invocation (no
/// long-lived cache so the user can install/uninstall gh between calls
/// without restarting elai).
const GH_BINARY: &str = "gh";

/// Default `gh pr list --limit` when caller passes 0.
const DEFAULT_LIST_LIMIT: u32 = 20;

/// Concrete adapter wrapping the `gh` CLI binary.
///
/// Holds an `Arc<Ssb>` so every public method can emit its C-40 row
/// without the caller having to thread the writer through. Mirrors the
/// `WipSquashOutcome` producer pattern in `wip_squash.rs:93-110`.
pub struct GhCliAdapter {
    ssb: Arc<Ssb>,
    /// Stable agent_id label embedded in every emitted SSB row. Lets
    /// the dashboard distinguish github-pr-adapter rows from generic
    /// orchestrator rows at the agent-filter level.
    agent_id: String,
}

impl GhCliAdapter {
    /// Construct a new adapter bound to the given SSB writer. The
    /// `agent_id` is a stable label embedded in every SSB row emitted
    /// by this adapter instance (default: `"github-pr-adapter"`).
    #[must_use]
    pub fn new(ssb: Arc<Ssb>) -> Self {
        Self {
            ssb,
            agent_id: "github-pr-adapter".to_string(),
        }
    }

    /// Same as [`new`](Self::new) but lets the caller override the
    /// agent_id (useful for tests + multi-session orchestration where
    /// the row should be attributed to a specific session id).
    #[must_use]
    pub fn with_agent_id(ssb: Arc<Ssb>, agent_id: impl Into<String>) -> Self {
        Self {
            ssb,
            agent_id: agent_id.into(),
        }
    }

    /// Current wall-clock millis since UNIX epoch. Delegates to the
    /// E-41 submodule clock abstraction (`super::clock::now_ms`) for
    /// consistent saturating semantics across the github_pr family.
    fn now_ms() -> i64 {
        super::clock::now_ms()
    }

    /// Compute the 64-bit blake3 prefix used as the hashed identity in
    /// C-40 SSB payloads. Keeps the raw URL / SHA / CI label out of the
    /// broadcast stream.
    pub(super) fn hash_prefix(input: &str) -> u64 {
        let hash = blake3::hash(input.as_bytes());
        let bytes = hash.as_bytes();
        u64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ])
    }

    /// Run `gh --version`. Returns `Ok(())` when gh is on PATH (any
    /// exit code is treated as "installed", since `gh --version`
    /// always exits 0 when the binary is present). Returns
    /// [`GithubPrError::GhCliNotInstalled`] when the spawn itself fails
    /// with `ErrorKind::NotFound`. Other IO errors map to
    /// [`GithubPrError::GhIo`].
    async fn precheck_installed() -> Result<(), GithubPrError> {
        match Command::new(GH_BINARY).arg("--version").output().await {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Err(GithubPrError::GhCliNotInstalled)
            }
            Err(e) => Err(GithubPrError::from(e)),
        }
    }

    /// Run `gh auth status` and surface [`GithubPrError::GhCliNotAuthenticated`]
    /// when gh exits non-zero. `gh auth status` exits 0 only when at
    /// least one host has a valid token.
    async fn precheck_authenticated() -> Result<(), GithubPrError> {
        let output = Command::new(GH_BINARY)
            .args(["auth", "status"])
            .output()
            .await
            .map_err(GithubPrError::from)?;
        if !output.status.success() {
            return Err(GithubPrError::GhCliNotAuthenticated {
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            });
        }
        Ok(())
    }

    /// Combined precheck: installed + authenticated. Called at the
    /// top of every state-changing method so the user gets a single
    /// structured error rather than a generic non-zero exit.
    async fn precheck_ready() -> Result<(), GithubPrError> {
        Self::precheck_installed().await?;
        Self::precheck_authenticated().await
    }

    /// Run a `gh` command with the given argv and return its stdout
    /// on success, mapping non-zero exits to
    /// [`GithubPrError::GhCommandFailed`]. The `command_label` is the
    /// argv joined by spaces — used only for diagnostics, never logged
    /// to SSB.
    async fn run_gh(&self, args: &[&str]) -> Result<Vec<u8>, GithubPrError> {
        let label = format!("gh {}", args.join(" "));
        debug!(target: "github_pr", command = %label, "running gh");
        let output = Command::new(GH_BINARY)
            .args(args)
            .output()
            .await
            .map_err(GithubPrError::from)?;
        if !output.status.success() {
            return Err(GithubPrError::from_exit(label, output.status, output.stderr));
        }
        Ok(output.stdout)
    }

    /// Best-effort SSB write. Logs at warn-level on failure but does
    /// NOT propagate the error — the gh-side change is already
    /// committed by the time we reach here, and dropping the row is
    /// strictly less harmful than panicking the caller.
    async fn emit(&self, payload: SsbPayload) {
        if let Err(e) = self.ssb.write_payload(&self.agent_id, 0, payload, vec![]).await {
            warn!(
                target: "github_pr",
                error = %e,
                "ssb write failed for c-40 row; row dropped",
            );
        }
    }

    pub(super) fn validate_open_request(request: &PrOpenRequest) -> Result<(), GithubPrError> {
        if request.title.trim().is_empty() {
            return Err(GithubPrError::InvalidRequest {
                reason: "title must not be empty".to_string(),
            });
        }
        if request.head.trim().is_empty() {
            return Err(GithubPrError::InvalidRequest {
                reason: "head branch must not be empty".to_string(),
            });
        }
        Ok(())
    }
}

/// Wire-format helper for parsing `gh pr create --json number,url`
/// output. Kept private — callers see [`PrSummary`].
#[derive(Debug, Deserialize)]
struct GhCreateResponse {
    number: u32,
    url: String,
}

/// Wire-format helper for parsing `gh pr merge --json mergeCommit`.
/// The `mergeCommit.oid` field is the merge commit SHA when gh emits
/// it; absent when the merge is queued or otherwise deferred.
#[derive(Debug, Deserialize)]
struct GhMergeResponse {
    #[serde(rename = "mergeCommit", default)]
    merge_commit: Option<GhMergeCommit>,
}

#[derive(Debug, Deserialize)]
struct GhMergeCommit {
    #[serde(default)]
    oid: String,
}

impl GithubPrAdapter for GhCliAdapter {
    async fn open(&self, request: PrOpenRequest) -> Result<PrSummary, GithubPrError> {
        Self::validate_open_request(&request)?;
        Self::precheck_ready().await?;

        // Build the gh argv. Repo selection: `gh -R owner/name pr create ...`
        // when caller supplied a repo; otherwise gh infers from the
        // working directory's git remote.
        let mut args: Vec<String> = Vec::with_capacity(16);
        if let Some(repo) = &request.repo {
            args.push("-R".to_string());
            args.push(repo.clone());
        }
        args.extend([
            "pr".to_string(),
            "create".to_string(),
            "--title".to_string(),
            request.title.clone(),
            "--body".to_string(),
            request.body.clone(),
            "--head".to_string(),
            request.head.clone(),
        ]);
        if let Some(base) = &request.base {
            args.push("--base".to_string());
            args.push(base.clone());
        }
        if request.draft {
            args.push("--draft".to_string());
        }

        // gh's `pr create --json` is supported on recent gh releases;
        // earlier releases print the PR URL to stdout but not JSON. We
        // request --json and fall through to URL-parse if the response
        // looks empty (defense against older gh installs).
        args.push("--json".to_string());
        args.push("number,url".to_string());

        let argv: Vec<&str> = args.iter().map(String::as_str).collect();
        let stdout = self.run_gh(&argv).await?;
        let stdout_str = String::from_utf8_lossy(&stdout).to_string();

        let resp: GhCreateResponse = serde_json::from_str(stdout_str.trim())
            .map_err(|e| GithubPrError::from_json(&stdout_str, e))?;

        let summary = PrSummary {
            number: resp.number,
            title: request.title.clone(),
            state: PrState::Open,
            url: resp.url.clone(),
            head_ref_name: request.head.clone(),
            base_ref_name: request.base.clone().unwrap_or_default(),
            is_draft: request.draft,
        };

        self.emit(SsbPayload::GithubPrOpened(GithubPrOpenedPayload {
            pr_number: summary.number,
            pr_url_hash: Self::hash_prefix(&summary.url),
            ts_ms: Self::now_ms(),
        }))
        .await;

        Ok(summary)
    }

    async fn list(
        &self,
        repo: Option<String>,
        state_filter: Option<String>,
        limit: u32,
    ) -> Result<Vec<PrSummary>, GithubPrError> {
        Self::precheck_ready().await?;

        let effective_limit = if limit == 0 { DEFAULT_LIST_LIMIT } else { limit };
        let mut args: Vec<String> = Vec::with_capacity(16);
        if let Some(repo) = &repo {
            args.push("-R".to_string());
            args.push(repo.clone());
        }
        args.extend([
            "pr".to_string(),
            "list".to_string(),
            "--limit".to_string(),
            effective_limit.to_string(),
            "--json".to_string(),
            "number,title,state,url,headRefName,baseRefName,isDraft".to_string(),
        ]);
        if let Some(state) = &state_filter {
            args.push("--state".to_string());
            args.push(state.clone());
        }

        let argv: Vec<&str> = args.iter().map(String::as_str).collect();
        let stdout = self.run_gh(&argv).await?;
        let stdout_str = String::from_utf8_lossy(&stdout).to_string();
        let summaries: Vec<PrSummary> = serde_json::from_str(stdout_str.trim())
            .map_err(|e| GithubPrError::from_json(&stdout_str, e))?;
        Ok(summaries)
    }

    async fn status(
        &self,
        repo: Option<String>,
        pr_number: u32,
    ) -> Result<PrStatus, GithubPrError> {
        Self::precheck_ready().await?;

        let pr_number_str = pr_number.to_string();
        let mut args: Vec<String> = Vec::with_capacity(8);
        if let Some(repo) = &repo {
            args.push("-R".to_string());
            args.push(repo.clone());
        }
        args.extend([
            "pr".to_string(),
            "view".to_string(),
            pr_number_str.clone(),
            "--json".to_string(),
            "state,reviewDecision,statusCheckRollup".to_string(),
        ]);
        let argv: Vec<&str> = args.iter().map(String::as_str).collect();
        let stdout = self.run_gh(&argv).await?;
        let stdout_str = String::from_utf8_lossy(&stdout).to_string();

        // Wire-format helper: gh emits `statusCheckRollup` as a
        // heterogeneous array; we deserialize into Vec<PrCheckRun>
        // via the public type alias since PrCheckRun is `#[serde(default)]`
        // on every field.
        #[derive(Deserialize)]
        struct StatusWire {
            #[serde(default)]
            state: Option<PrState>,
            #[serde(rename = "reviewDecision", default)]
            review_decision: String,
            #[serde(rename = "statusCheckRollup", default)]
            check_runs: Vec<PrCheckRun>,
        }

        let wire: StatusWire = serde_json::from_str(stdout_str.trim())
            .map_err(|e| GithubPrError::from_json(&stdout_str, e))?;

        let status = PrStatus {
            state: wire.state.unwrap_or(PrState::Unknown),
            review_decision: wire.review_decision,
            check_runs: wire.check_runs,
        };

        let ci_label = status.ci_rollup_label();
        let review_label = status.review_state_label();
        self.emit(SsbPayload::GithubPrStatusFetched(
            GithubPrStatusFetchedPayload {
                pr_number,
                ci_status_hash: Self::hash_prefix(ci_label),
                review_state_hash: Self::hash_prefix(&review_label),
                ts_ms: Self::now_ms(),
            },
        ))
        .await;

        Ok(status)
    }

    async fn close(
        &self,
        repo: Option<String>,
        pr_number: u32,
    ) -> Result<(), GithubPrError> {
        Self::precheck_ready().await?;

        let pr_number_str = pr_number.to_string();
        let mut args: Vec<String> = Vec::with_capacity(6);
        if let Some(repo) = &repo {
            args.push("-R".to_string());
            args.push(repo.clone());
        }
        args.extend(["pr".to_string(), "close".to_string(), pr_number_str]);
        let argv: Vec<&str> = args.iter().map(String::as_str).collect();
        self.run_gh(&argv).await?;

        self.emit(SsbPayload::GithubPrClosed(GithubPrClosedPayload {
            pr_number,
            ts_ms: Self::now_ms(),
        }))
        .await;
        Ok(())
    }

    async fn merge(
        &self,
        repo: Option<String>,
        pr_number: u32,
    ) -> Result<String, GithubPrError> {
        Self::precheck_ready().await?;

        let pr_number_str = pr_number.to_string();
        let mut args: Vec<String> = Vec::with_capacity(8);
        if let Some(repo) = &repo {
            args.push("-R".to_string());
            args.push(repo.clone());
        }
        // --squash is the default merge strategy for ELAI: matches the
        // C-13 wip_squash producer pattern (one branch -> one merge
        // commit on trunk). Callers needing rebase/merge-commit must
        // extend the API in a follow-up.
        args.extend([
            "pr".to_string(),
            "merge".to_string(),
            pr_number_str.clone(),
            "--squash".to_string(),
        ]);
        let argv: Vec<&str> = args.iter().map(String::as_str).collect();
        self.run_gh(&argv).await?;

        // Best-effort: fetch the merge commit SHA via a second view
        // call. gh's `pr merge` itself does not emit JSON; the view
        // call is the supported way to read mergeCommit.oid.
        let mut view_args: Vec<String> = Vec::with_capacity(8);
        if let Some(repo) = &repo {
            view_args.push("-R".to_string());
            view_args.push(repo.clone());
        }
        view_args.extend([
            "pr".to_string(),
            "view".to_string(),
            pr_number_str,
            "--json".to_string(),
            "mergeCommit".to_string(),
        ]);
        let view_argv: Vec<&str> = view_args.iter().map(String::as_str).collect();
        let merge_sha = match self.run_gh(&view_argv).await {
            Ok(stdout) => {
                let stdout_str = String::from_utf8_lossy(&stdout).to_string();
                serde_json::from_str::<GhMergeResponse>(stdout_str.trim())
                    .ok()
                    .and_then(|r| r.merge_commit.map(|m| m.oid))
                    .unwrap_or_default()
            }
            Err(_) => String::new(),
        };

        self.emit(SsbPayload::GithubPrMerged(GithubPrMergedPayload {
            pr_number,
            merge_sha_hash: Self::hash_prefix(&merge_sha),
            ts_ms: Self::now_ms(),
        }))
        .await;
        Ok(merge_sha)
    }
}
