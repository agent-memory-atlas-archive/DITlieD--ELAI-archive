//! G-5 — `elai review --cockpit` handler (the reviewer-cockpit DSR producer).
//!
//! The HEADLESS PRODUCER (DD4): it parses the working-tree git diff against a
//! deterministically pinned base, reads the session's SSB vouch stream, computes
//! the Diff-Surface-to-Review (DSR) score, prints the vouched/un-vouched bands +
//! the headline DSR, and emits ONE `review_surface_scored` row through the SAME
//! live `Arc<Ssb>` that holds the session's `ssb.db` (R4: the off-agent artifact
//! a re-run re-checks; the dashboard cockpit window is the read-only CONSUMER).
//!
//! DETERMINISM (GD-2): both DSR inputs are pure functions of the (tree, session)
//! pair — the git diff is pinned to a resolved base SHA (recorded in the row),
//! and the SSB stream is read seq-ASC. The DSR recomputes byte-identically.
//!
//! GD-1: this driver CONSUMES the F-4 / F-1 / F-2a verdicts already in the SSB
//! stream via `build_file_signals`; it re-implements NONE of those gates.
//!
//! Fail-soft on the DIFF SOURCE (git absent / not a repo): a git failure yields
//! an empty hunk set (a clean tree / no-git environment scores DSR 0.0 with
//! zero changed lines) rather than aborting — the DSR computation itself stays
//! deterministic. A non-zero `git diff` exit is surfaced as a warning.
//!
//! Default-ON: the only opt-out is the kill-switch `ELAI_REVIEW_COCKPIT_DISABLE`
//! (truthy). There is deliberately NO default-OFF enable flag (W-2 / R9).
//!
//! `Ssb::new` spawns the single-writer task and MUST be constructed inside
//! `rt.block_on` (the dep-check / bench-seccodeplt precedent), else it panics
//! "no reactor running".

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use elai_core::paths::sessions_dir_path;
use elai_core::OsClock;
use elai_orchestrator::focus_tunnel::diff::DiffParser;
use elai_orchestrator::review_surface::{
    build_file_signals, score_review_surface, ReviewSurfaceScore,
};
use elai_orchestrator::ssb::ReviewSurfaceScoredPayload;
use elai_orchestrator::{Ssb, SsbConfig, SsbPayload};

use crate::args::{CliOutputFormat, ReviewAction};

/// Canonical default session id when `--session` is omitted. The operator passes
/// the real session id to score that session's vouch stream; the default scores
/// the canonical `"default"` session (an empty stream scores DSR 1.0 — every
/// hunk un-vouched, the conservative answer).
const DEFAULT_SESSION_ID: &str = "default";

/// Default git ref to diff the working tree against (resolved to a concrete SHA
/// before use so the diff bytes are stably pinned).
const DEFAULT_DIFF_BASE: &str = "HEAD";

/// `elai review <sub>` dispatcher invoked by main.rs.
pub(crate) fn run_review(
    action: ReviewAction,
    output_format: CliOutputFormat,
) -> anyhow::Result<()> {
    match action {
        ReviewAction::Cockpit { session, base } => run_cockpit(session, base, output_format),
    }
}

/// G-5 kill-switch check. The cockpit DSR producer is DEFAULT-ON; the only
/// opt-out is the `ELAI_REVIEW_COCKPIT_DISABLE` kill-switch (truthy value).
/// There is deliberately NO default-OFF enable flag (W-2 / R9: a DISABLE
/// kill-switch needs no wire-dark-allowlist row; an ENABLE flag would).
fn cockpit_disabled() -> bool {
    match std::env::var("ELAI_REVIEW_COCKPIT_DISABLE") {
        Ok(raw) => {
            let v = raw.trim().to_ascii_lowercase();
            v == "1" || v == "true" || v == "yes" || v == "on"
        }
        Err(_) => false,
    }
}

/// The outcome of obtaining the working-tree diff (GD-5 honesty fix, s-56068
/// follow-up). Distinguishes a git ERROR (the diff could not be obtained at all)
/// from a GENUINELY EMPTY diff (a clean tree — a real, honest "no changes to
/// review" state). Before this fix both collapsed to `String::new()`, so a git
/// error rendered indistinguishably from a clean tree as DSR 0.0 = "all clean"
/// (fabricated reassurance). A git error must NEVER read as a clean DSR=0.
enum DiffSource {
    /// The diff was obtained. The String is the raw unified-diff text; an empty
    /// String here is a genuine empty diff (clean tree -> DSR 0.0 over 0 lines).
    Available(String),
    /// Git failed (absent / not a repo / non-zero exit / unresolvable base). The
    /// DSR is NOT a review verdict in this case; the reason surfaces to the row +
    /// the renderers as an explicit "DIFF UNAVAILABLE" banner.
    Unavailable { reason: String },
}

/// `git diff` exit code for "not a git repository" / fatal usage errors. Named
/// per CLAUDE.md (no magic numbers); used only in the surfaced error string.
const GIT_FATAL_EXIT: i32 = 128;

/// Resolve the diff base ref to a concrete commit SHA via `git rev-parse`, so
/// the recorded `base_commit` pins the exact tree state the DSR was computed
/// against. Returns `Err(reason)` when git fails (absent / not a repo /
/// unresolvable ref) so the caller surfaces an explicit DIFF-UNAVAILABLE state
/// instead of silently pinning the unresolved ref and pretending it succeeded.
fn resolve_base_commit(
    project_root: &std::path::Path,
    base_ref: &str,
) -> Result<String, String> {
    let out = Command::new("git")
        .current_dir(project_root)
        .args(["rev-parse", base_ref])
        .output();
    match out {
        Ok(o) if o.status.success() => {
            let sha = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if sha.is_empty() {
                Err(format!(
                    "`git rev-parse {base_ref}` succeeded but returned an empty SHA"
                ))
            } else {
                Ok(sha)
            }
        }
        Ok(o) => {
            let code = o.status.code().unwrap_or(GIT_FATAL_EXIT);
            Err(format!(
                "`git rev-parse {base_ref}` exited {code}: {}",
                String::from_utf8_lossy(&o.stderr).trim()
            ))
        }
        Err(e) => Err(format!(
            "`git rev-parse {base_ref}` unavailable ({e}); git absent / non-repo"
        )),
    }
}

/// Obtain the working-tree unified diff against the pinned base via `git diff`
/// (the in-tree pattern: wiring_coverage.rs shells out the same way). Returns a
/// [`DiffSource`] that DISTINGUISHES a git error from a genuine empty diff: a
/// git-absent / non-repo / non-zero exit yields `Unavailable { reason }` (NOT an
/// empty string), so the DSR is not fabricated as a clean 0.0. An `Ok` exit with
/// empty stdout is `Available("")` — a real clean tree, the honest DSR-0 state.
fn working_tree_diff(project_root: &std::path::Path, base: &str) -> DiffSource {
    let out = Command::new("git")
        .current_dir(project_root)
        .args(["diff", "--unified=0", base])
        .output();
    match out {
        Ok(o) if o.status.success() => {
            DiffSource::Available(String::from_utf8_lossy(&o.stdout).to_string())
        }
        Ok(o) => {
            let code = o.status.code().unwrap_or(GIT_FATAL_EXIT);
            let reason = format!(
                "`git diff` exited {code}: {}",
                String::from_utf8_lossy(&o.stderr).trim()
            );
            tracing::warn!("review --cockpit: {reason}; DSR not computed (diff unavailable)");
            DiffSource::Unavailable { reason }
        }
        Err(e) => {
            let reason = format!("`git diff` unavailable ({e}); git absent / non-repo");
            tracing::warn!("review --cockpit: {reason}; DSR not computed (diff unavailable)");
            DiffSource::Unavailable { reason }
        }
    }
}

fn run_cockpit(
    session: Option<String>,
    base: Option<String>,
    output_format: CliOutputFormat,
) -> anyhow::Result<()> {
    if cockpit_disabled() {
        println!("review --cockpit: suppressed by ELAI_REVIEW_COCKPIT_DISABLE kill-switch");
        return Ok(());
    }

    let project_root: PathBuf = std::env::current_dir()
        .map_err(|e| anyhow::anyhow!("review --cockpit: cannot determine cwd: {e}"))?;
    let session_id = session.unwrap_or_else(|| DEFAULT_SESSION_ID.to_string());
    let base_ref = base.unwrap_or_else(|| DEFAULT_DIFF_BASE.to_string());

    // 1. Resolve the pinned base + obtain the working-tree diff. A git error at
    //    EITHER step yields a DIFF-UNAVAILABLE state (GD-5 honesty fix): the DSR
    //    is then NOT a review verdict and must not render as a clean DSR=0. A
    //    genuine empty diff (clean tree) is a DISTINCT, honest DSR-0 state.
    let (base_commit, hunks, diff_status) = match resolve_base_commit(&project_root, &base_ref) {
        Err(reason) => {
            tracing::warn!("review --cockpit: base resolve failed ({reason}); DSR not computed");
            // Fall back to the un-resolved ref as the recorded base label, mark the
            // diff unavailable, and score over an empty hunk set (the DSR is a
            // sentinel here — `diff_available=false` is the load-bearing signal).
            (base_ref.clone(), Vec::new(), DiffStatus::unavailable(reason))
        }
        Ok(base_commit) => match working_tree_diff(&project_root, &base_commit) {
            DiffSource::Unavailable { reason } => {
                (base_commit, Vec::new(), DiffStatus::unavailable(reason))
            }
            DiffSource::Available(diff_text) => {
                // DENOMINATOR + hunk bodies: parse the obtained diff. A parse miss
                // on an OBTAINED diff means a genuinely empty/zero-hunk tree (the
                // honest DSR-0 "no changes" state), NOT a git error.
                let hunks = match DiffParser.parse(&diff_text) {
                    Ok(h) => h,
                    Err(e) => {
                        tracing::warn!(
                            "review --cockpit: diff parse produced no hunks ({e}); DSR over 0 lines"
                        );
                        Vec::new()
                    }
                };
                (base_commit, hunks, DiffStatus::available())
            }
        },
    };

    // 2. VOUCH SET + the score: open the session ssb.db (live Arc<Ssb>), read the
    //    seq-ASC stream + the file junction, build the per-file signals, compute
    //    the DSR, then emit the row through the SAME live handle (R4 / GD-2).
    let score = open_score_and_emit(&project_root, &session_id, &base_commit, &hunks, &diff_status)?;

    // 3. Print the bands + the headline DSR (NO measured review-time number; the
    //    time-to-first-approve study is a deferred research protocol, GD-3/GD-5).
    match output_format {
        CliOutputFormat::Json => {
            // Honest JSON: a git error MUST NOT serialize as a bare clean score.
            // Wrap the score with the explicit diff-availability status.
            let envelope = serde_json::json!({
                "diff_available": diff_status.available,
                "diff_unavailable_reason": diff_status.reason,
                "score": &score,
            });
            let json = serde_json::to_string_pretty(&envelope)
                .map_err(|e| anyhow::anyhow!("review --cockpit: serialize score as json: {e}"))?;
            println!("{json}");
        }
        CliOutputFormat::Text | CliOutputFormat::Toml => {
            render_cockpit_text(&session_id, &base_commit, &score, &diff_status);
        }
    }
    Ok(())
}

/// GD-5 honesty fix: the working-tree diff availability the renderers + the
/// emitted row carry. `available == false` ONLY on a git error (the DSR is then
/// a sentinel, not a verdict); a genuine clean tree is `available == true` with
/// an empty reason (the honest DSR-0 state).
struct DiffStatus {
    available: bool,
    reason: String,
}

impl DiffStatus {
    fn available() -> Self {
        Self {
            available: true,
            reason: String::new(),
        }
    }

    fn unavailable(reason: String) -> Self {
        Self {
            available: false,
            reason,
        }
    }
}

/// Open the session `ssb.db` inside a current-thread runtime, read the vouch
/// stream + file junction through that LIVE handle, score the diff, and emit ONE
/// `review_surface_scored` row through the SAME handle (the off-agent artifact,
/// R4). Returns the computed score for printing.
fn open_score_and_emit(
    project_root: &std::path::Path,
    session_id: &str,
    base_commit: &str,
    hunks: &[elai_orchestrator::focus_tunnel::diff::DiffHunk],
    diff_status: &DiffStatus,
) -> anyhow::Result<ReviewSurfaceScore> {
    let db_path = sessions_dir_path(project_root).join("ssb.db");
    let parent = db_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("review --cockpit: ssb.db has no parent dir"))?;
    std::fs::create_dir_all(parent)
        .map_err(|e| anyhow::anyhow!("review --cockpit: create sessions dir failed: {e}"))?;

    // HOLLOW-COCKPIT FIX (s-56068 follow-up): derive the SSB `project_id` from the
    // review's target repo root with the SAME canonical helper the `elai run` CLI
    // (runtime_setup.rs:257) and the dashboard (commands.rs:6084) use. The vouch
    // readers (`query_session_stream_asc` / `session_files_by_seq`) filter on
    // `WHERE session_id = ?1 AND project_id = ?2` (ssb.rs:14983/15008), so a
    // hardcoded "default" id read an EMPTY vouch set against any real run
    // (production journals rows under `proj-<blake3-16hex>`), making every hunk
    // un-vouched -> DSR always ~1.0. Matching the run's bucket lets the read hit
    // the run's events. `project_root` is the resolved cwd (run_cockpit:178).
    let project_id = elai_core::project_id_from_path(project_root);

    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id,
        session_id: session_id.to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| anyhow::anyhow!("review --cockpit: build async runtime failed: {e}"))?;

    let session_owned = session_id.to_string();
    let base_owned = base_commit.to_string();
    let hunks_owned = hunks.to_vec();
    let diff_available = diff_status.available;
    let diff_reason = if diff_status.reason.is_empty() {
        None
    } else {
        Some(diff_status.reason.clone())
    };

    // Ssb::new spawns the single-writer task and MUST be constructed inside the
    // runtime context (dep-check / bench-seccodeplt precedent).
    rt.block_on(async move {
        let ssb = Ssb::new(cfg, clock)
            .map_err(|e| anyhow::anyhow!("review --cockpit: open SSB handle failed: {e}"))?;

        // Read the deterministic vouch inputs through the live handle.
        let stream = ssb
            .query_session_stream_asc(&session_owned)
            .map_err(|e| anyhow::anyhow!("review --cockpit: read session stream: {e}"))?;
        let files_by_seq = ssb
            .session_files_by_seq(&session_owned)
            .map_err(|e| anyhow::anyhow!("review --cockpit: read file junction: {e}"))?;

        let signals: BTreeMap<String, _> = build_file_signals(&stream, &files_by_seq);
        let score = score_review_surface(&hunks_owned, &signals);

        // Emit the off-agent beacon row through the SAME live handle (R4): a real
        // CLI entrypoint computed the score and this row landed. The
        // `diff_available` flag (GD-5 honesty fix) records whether the score is a
        // real review verdict or a sentinel emitted on a git error.
        let total_u32 = score.total_changed_lines;
        let attention_u32 = score.attention_lines;
        let payload = SsbPayload::ReviewSurfaceScored(ReviewSurfaceScoredPayload {
            session_id: session_owned.clone(),
            dsr: score.dsr,
            total_changed_lines: total_u32,
            attention_lines: attention_u32,
            vouched_hunks: score.vouched_hunks,
            unvouched_hunks: score.unvouched_hunks,
            base_commit: base_owned,
            diff_available,
            diff_unavailable_reason: diff_reason,
        });
        ssb.write_payload("cli:review-cockpit", 0, payload, Vec::new())
            .await
            .map_err(|e| anyhow::anyhow!("review --cockpit: write review_surface_scored row: {e}"))?;

        Ok::<ReviewSurfaceScore, anyhow::Error>(score)
    })
}

/// Render the cockpit summary in text: the vouched/un-vouched bands per hunk +
/// the headline DSR + the silent-by-default findings. NO review-time number is
/// printed (GD-3/GD-5: the time-to-first-approve study is deferred).
///
/// GD-5 honesty fix: when the diff was UNAVAILABLE (git error), the DSR is NOT a
/// review verdict — render an explicit "DIFF UNAVAILABLE" banner and suppress the
/// DSR line, so a git error never reads as a clean DSR=0.
fn render_cockpit_text(
    session_id: &str,
    base_commit: &str,
    score: &ReviewSurfaceScore,
    diff_status: &DiffStatus,
) {
    println!("Reviewer Cockpit — Diff-Surface-to-Review (G-5)");
    println!("  session     : {session_id}");
    println!("  diff base   : {base_commit}");

    if !diff_status.available {
        println!(
            "  DIFF UNAVAILABLE (git error: {}) — DSR not computed",
            diff_status.reason
        );
        println!("  (this is NOT a clean review: the working-tree diff could not be obtained)");
        return;
    }

    println!(
        "  DSR         : {:.4}  ({} of {} changed lines need full attention)",
        score.dsr, score.attention_lines, score.total_changed_lines
    );
    println!(
        "  hunks       : {} vouched / {} un-vouched",
        score.vouched_hunks, score.unvouched_hunks
    );

    if score.hunks.is_empty() {
        println!("  (no changed hunks against the pinned base — nothing to review)");
    } else {
        println!("  bands:");
        for h in &score.hunks {
            let band = if h.needs_attention { "ATTN " } else { "vouch" };
            let vouches: Vec<&str> = h.vouched_by.iter().map(|v| v.label()).collect();
            let vouch_str = if vouches.is_empty() {
                String::new()
            } else {
                format!(" [{}]", vouches.join(","))
            };
            println!(
                "    [{band}] {} ({} ln){vouch_str}",
                h.hunk_id, h.lines_changed
            );
        }
    }

    if score.findings.is_empty() {
        println!("  findings    : none (silent-by-default: a clean un-vouched hunk is NOT a finding)");
    } else {
        println!("  findings (deterministic verdicts / failing tests only):");
        for f in &score.findings {
            println!("    [{}] {}", f.kind.label(), f.file_path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FIX 2 (load-bearing) — a non-repo directory is a GIT ERROR, NOT a clean
    /// tree. `working_tree_diff` MUST return `Unavailable { reason }` (carrying a
    /// non-empty reason), never `Available("")`. Before the fix this collapsed to
    /// `String::new()`, indistinguishable from a genuine empty diff -> DSR 0.0 =
    /// fabricated "all clean".
    #[test]
    fn working_tree_diff_on_non_repo_is_unavailable_not_empty() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        // A fresh tempdir is not a git repo: `git diff` exits non-zero / git may
        // be absent. EITHER way the honest result is Unavailable, never an empty
        // Available diff (which would read as a clean DSR=0).
        match working_tree_diff(dir.path(), "HEAD") {
            DiffSource::Unavailable { reason } => {
                assert!(
                    !reason.is_empty(),
                    "a git error must carry a non-empty reason"
                );
            }
            DiffSource::Available(diff) => {
                panic!(
                    "a non-repo dir must be Unavailable (git error), not a clean empty diff; got Available({:?})",
                    diff
                );
            }
        }
    }

    /// FIX 2 — `resolve_base_commit` on a non-repo MUST be `Err(reason)`, so the
    /// caller surfaces DIFF-UNAVAILABLE rather than silently pinning the
    /// unresolved ref and pretending the resolve succeeded.
    #[test]
    fn resolve_base_commit_on_non_repo_is_err() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let result = resolve_base_commit(dir.path(), "HEAD");
        assert!(
            result.is_err(),
            "resolving a base in a non-repo must Err (git error), got {result:?}"
        );
        if let Err(reason) = result {
            assert!(!reason.is_empty(), "the resolve error must carry a reason");
        }
    }

    /// FIX 2 — the `DiffStatus` states are the honest distinction the renderers
    /// key on: `available()` is true with no reason (covers BOTH a real diff and
    /// a genuine clean tree); `unavailable(reason)` is false with the reason.
    #[test]
    fn diff_status_states_are_distinct() {
        let ok = DiffStatus::available();
        assert!(ok.available);
        assert!(ok.reason.is_empty());

        let err = DiffStatus::unavailable("`git diff` exited 128".to_string());
        assert!(!err.available);
        assert_eq!(err.reason, "`git diff` exited 128");
    }
}
