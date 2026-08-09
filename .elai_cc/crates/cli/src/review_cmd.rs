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
use elai_orchestrator::focus_tunnel::diff::{DiffBodyKind, DiffBodyLine, DiffHunk, DiffParser};
use elai_orchestrator::review_anchor::{anchor_disabled, anchor_finding, AnchorKind};
use elai_orchestrator::review_surface::{
    build_file_signals, hunk_id_for, score_review_surface, FindingKind, ReviewFinding,
    ReviewSurfaceScore,
};
use elai_orchestrator::ssb::{FindingAnchoredKind, FindingAnchoredPayload, ReviewSurfaceScoredPayload};
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
        ReviewAction::Catalog { language, catalog_path } => {
            crate::review_catalog_cmd::run_catalog_cmd(language, catalog_path, output_format)
        }
        ReviewAction::Cockpit {
            session,
            base,
            demo_anchor,
            trust_model_line,
        } => {
            if demo_anchor {
                // H-52 W-1 seeded-fixture trigger. Runs the resolver over a deterministic
                // in-memory diff fixture and emits real FindingAnchored rows. This is the
                // sanctioned observable_effect pattern (mirrors H-21/H-26/H-27 --demo flags).
                // DEFERRED: the live model-finding source (E-41/C-40) is the future consumer.
                // This fixture is the W-1 live-CLI proof until E-41/C-40 ship.
                run_demo_anchor(trust_model_line, output_format)
            } else {
                run_cockpit(session, base, output_format)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// H-52 W-1 seeded-fixture anchor demo
// ---------------------------------------------------------------------------

/// Session id for the anchor demo fixture session.
const DEMO_ANCHOR_SESSION_ID: &str = "elai-review-anchor-demo";

/// Project id for the anchor demo fixture session.
const DEMO_ANCHOR_PROJECT_ID: &str = "elai-review-anchor";

/// Seeded fixture file path used for all three demo cases.
const DEMO_ANCHOR_FILE: &str = "src/review_surface.rs";

/// H-52 W-1 seeded-fixture trigger.
///
/// Injects three deterministic snippet-bearing `ReviewFinding`s over a fixed
/// in-memory diff fixture (not a git diff — the fixture is code-owned and does
/// NOT require a git repo) and drives `anchor_finding` + `FindingAnchored` SSB
/// emission through the REAL `Arc<Ssb>` handle. This is the live-CLI proof
/// (R3/R7 reachability): a real entrypoint ran and left off-agent SSB rows the
/// test decodes.
///
/// Three observable-effect cases (W-1):
///   (a) snippet occurring ONCE in the hunk => `Anchored` with `hunk_id` + `new_line_range`
///   (b) snippet occurring TWICE in the hunk => `Ambiguous` + file-fallback (not guessed)
///   (c) snippet absent from the hunk => `Unanchorable` + file-fallback
///
/// `trust_model_line` (broken arm, R2 proof): forces case (b) to be anchored
/// to the FIRST occurrence without the ambiguity check (i.e., trusts a
/// model-emitted line number). This intentionally mis-anchors the finding — the
/// broken arm should be observably wrong (hunk_id is set on what should be
/// Ambiguous), proving that the correct resolver is necessary.
///
/// DEFERRED: the live model-finding source (real snippet-bearing ReviewFindings
/// from E-41/C-40) stays deferred to E-41/C-40 per the H-52 plan's own deferral.
/// This seeded fixture IS the W-1 live-CLI proof; the model-finding consumer is
/// the deferred future activation.
/// <!-- DEFERRED: E-41/C-40 model-finding source; seeded fixture is W-1 proof now -->
pub(crate) fn run_demo_anchor(
    trust_model_line: bool,
    output_format: CliOutputFormat,
) -> anyhow::Result<()> {
    // Build the seeded fixture diff hunk. Contains:
    //   Line 1 (new): the UNIQUE snippet (case a)
    //   Line 2 (new): filler line
    //   Line 3 (new): DUPLICATE snippet first occurrence (case b)
    //   Line 4 (new): filler line
    //   Line 5 (new): DUPLICATE snippet second occurrence (case b)
    //
    // Case (c): the absent snippet is simply not present in this hunk body.
    let unique_snippet = "fn compute_dsr(attention: u32, total: u32) -> f64 { attention as f64 / total as f64 }";
    let duplicate_snippet = "fn placeholder_duplicate() {}";
    let absent_snippet = "fn this_function_does_not_exist_in_the_fixture() {}";

    let fixture_hunk = make_demo_anchor_hunk(unique_snippet, duplicate_snippet);
    let fixture_hunks = vec![fixture_hunk.clone()];
    let hunk_id = hunk_id_for(&fixture_hunk);

    // Three fixture findings — each carries a REAL snippet (not "").
    // (a) unique_snippet: appears ONCE => Anchored
    // (b) duplicate_snippet: appears TWICE => Ambiguous (or mis-anchored under broken arm)
    // (c) absent_snippet: not found => Unanchorable
    let fixture_findings: Vec<ReviewFinding> = vec![
        ReviewFinding {
            file_path: DEMO_ANCHOR_FILE.to_string(),
            kind: FindingKind::VibeGateFired,
            snippet: unique_snippet.to_string(),
            anchor: None,
        },
        ReviewFinding {
            file_path: DEMO_ANCHOR_FILE.to_string(),
            kind: FindingKind::SecurityRuleFired,
            snippet: duplicate_snippet.to_string(),
            anchor: None,
        },
        ReviewFinding {
            file_path: DEMO_ANCHOR_FILE.to_string(),
            kind: FindingKind::TestFailed,
            snippet: absent_snippet.to_string(),
            anchor: None,
        },
    ];

    let project_root: PathBuf = std::env::current_dir()
        .map_err(|e| anyhow::anyhow!("review --demo-anchor: cannot determine cwd: {e}"))?;
    let sessions_dir = sessions_dir_path(&project_root);
    std::fs::create_dir_all(&sessions_dir)
        .map_err(|e| anyhow::anyhow!("review --demo-anchor: create sessions dir: {e}"))?;
    let db_path = sessions_dir.join("ssb.db");

    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: DEMO_ANCHOR_PROJECT_ID.to_string(),
        session_id: DEMO_ANCHOR_SESSION_ID.to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| anyhow::anyhow!("review --demo-anchor: build async runtime: {e}"))?;

    let mode = if trust_model_line {
        "broken-arm (--trust-model-line-numbers)"
    } else {
        "resolver (correct)"
    };

    let results: Vec<(String, AnchorKind, Option<String>, Option<(u32, u32)>, String)> =
        rt.block_on(async move {
            let ssb = Ssb::new(cfg, clock)
                .map_err(|e| anyhow::anyhow!("review --demo-anchor: open SSB: {e}"))?;

            let mut out = Vec::new();

            for mut finding in fixture_findings {
                let file_hunks: Vec<DiffHunk> = fixture_hunks
                    .iter()
                    .filter(|h| h.file_path == finding.file_path)
                    .cloned()
                    .collect();

                let anchor_result = if trust_model_line
                    && finding.snippet == duplicate_snippet
                {
                    // BROKEN ARM (R2 proof): for the duplicate snippet, bypass the
                    // ambiguity check and anchor to the first occurrence as if a model
                    // emitted a confident line number. This intentionally yields a
                    // wrong Anchored result for what should be Ambiguous.
                    // A resolver that stays "right" when handed a model line is a FAIL
                    // (the resolver's whole job is to catch this case).
                    elai_orchestrator::review_anchor::AnchorResult {
                        hunk_id: Some(hunk_id.clone()),
                        new_line_range: Some((file_hunks.first()
                            .map(|h| h.new_start as u32 + 2)
                            .unwrap_or(1),
                            file_hunks.first()
                            .map(|h| h.new_start as u32 + 2)
                            .unwrap_or(1))),
                        kind: AnchorKind::Anchored,
                        candidate_match_n: 1,
                    }
                } else {
                    anchor_finding(&finding.snippet, &file_hunks, None)
                };

                let finding_id = format!(
                    "{}/{}/{}",
                    DEMO_ANCHOR_SESSION_ID,
                    finding.file_path,
                    finding.kind.label()
                );

                let snippet_bytes = finding.snippet.as_bytes();
                let snippet_blake3 = {
                    let h = blake3::hash(snippet_bytes);
                    format!("{}", h.to_hex())
                };

                finding.anchor = Some(anchor_result.clone());

                let anchor_kind_ssb = match anchor_result.kind {
                    AnchorKind::Anchored => FindingAnchoredKind::Anchored,
                    AnchorKind::FileFallback => FindingAnchoredKind::FileFallback,
                    AnchorKind::Ambiguous => FindingAnchoredKind::Ambiguous,
                    AnchorKind::Unanchorable => FindingAnchoredKind::Unanchorable,
                    AnchorKind::Relocated => FindingAnchoredKind::Relocated,
                };

                let payload = SsbPayload::FindingAnchored(FindingAnchoredPayload {
                    finding_id: finding_id.clone(),
                    snippet_blake3: snippet_blake3.clone(),
                    resolved_hunk_id: anchor_result.hunk_id.clone(),
                    new_line_start: anchor_result.new_line_range.map(|(s, _)| s),
                    kind: anchor_kind_ssb,
                    candidate_match_n: anchor_result.candidate_match_n,
                });

                // Write the off-agent artifact (R4). A write failure is an error (not
                // a warning) in demo mode — the whole point is to prove the row lands.
                ssb.write_payload(
                    "cli:review-anchor-demo",
                    0,
                    payload,
                    Vec::new(),
                )
                .await
                .map_err(|e| {
                    anyhow::anyhow!(
                        "review --demo-anchor: write FindingAnchored row for {}: {e}",
                        finding.file_path
                    )
                })?;

                out.push((
                    finding_id,
                    anchor_result.kind,
                    anchor_result.hunk_id,
                    anchor_result.new_line_range,
                    snippet_blake3,
                ));
            }

            Ok::<_, anyhow::Error>(out)
        })?;

    match output_format {
        CliOutputFormat::Json => {
            let rows: Vec<serde_json::Value> = results
                .iter()
                .map(|(id, kind, hunk_id, line_range, blake3_hex)| {
                    serde_json::json!({
                        "finding_id": id,
                        "kind": kind.as_str(),
                        "hunk_id": hunk_id,
                        "new_line_range": line_range.map(|(s, e)| [s, e]),
                        "snippet_blake3": blake3_hex,
                    })
                })
                .collect();
            let envelope = serde_json::json!({
                "demo_anchor": {
                    "mode": mode,
                    "session_id": DEMO_ANCHOR_SESSION_ID,
                    "project_id": DEMO_ANCHOR_PROJECT_ID,
                    "ssb_action_type": "finding_anchored",
                    "rows": rows,
                }
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&envelope)
                    .map_err(|e| anyhow::anyhow!("review --demo-anchor: serialize json: {e}"))?
            );
        }
        CliOutputFormat::Text | CliOutputFormat::Toml => {
            println!("elai review --cockpit --demo-anchor (H-52, W-1 live-CLI proof)");
            println!("  mode:       {mode}");
            println!("  session_id: {DEMO_ANCHOR_SESSION_ID}");
            println!("  project_id: {DEMO_ANCHOR_PROJECT_ID}");
            println!("  ssb_action: finding_anchored");
            println!();
            for (id, kind, hunk_id, line_range, blake3_hex) in &results {
                let hunk_str = hunk_id.as_deref().unwrap_or("(none)");
                let line_str = line_range
                    .map(|(s, e)| format!("{s}-{e}"))
                    .unwrap_or_else(|| "(none)".to_string());
                println!(
                    "  [{kind}] {id}  hunk={hunk_str}  lines={line_str}  blake3={blake3_hex}",
                    kind = kind.as_str()
                );
            }
        }
    }
    Ok(())
}

/// Build the seeded demo anchor hunk used by `run_demo_anchor`.
///
/// The hunk contains:
///   - new line 1: unique_snippet (appears once — case a)
///   - new line 2: filler context
///   - new line 3: duplicate_snippet first occurrence (case b)
///   - new line 4: filler context
///   - new line 5: duplicate_snippet second occurrence (case b)
///
/// The absent snippet (case c) is simply not present in this body.
fn make_demo_anchor_hunk(unique_snippet: &str, duplicate_snippet: &str) -> DiffHunk {
    let lines: Vec<DiffBodyLine> = vec![
        DiffBodyLine { kind: DiffBodyKind::Added, text: format!("+{unique_snippet}") },
        DiffBodyLine { kind: DiffBodyKind::Context, text: " // fixture context line".to_string() },
        DiffBodyLine { kind: DiffBodyKind::Added, text: format!("+{duplicate_snippet}") },
        DiffBodyLine { kind: DiffBodyKind::Context, text: " // fixture context line 2".to_string() },
        DiffBodyLine { kind: DiffBodyKind::Added, text: format!("+{duplicate_snippet}") },
    ];
    let added_lines = vec![
        unique_snippet.to_string(),
        duplicate_snippet.to_string(),
        duplicate_snippet.to_string(),
    ];
    let context_lines = vec![
        " // fixture context line".to_string(),
        " // fixture context line 2".to_string(),
    ];
    DiffHunk {
        file_path: DEMO_ANCHOR_FILE.to_string(),
        old_start: 42,
        old_count: 2,
        new_start: 42,
        new_count: 5,
        context_lines,
        added_lines,
        removed_lines: Vec::new(),
        body_order: lines,
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
        let mut score = score_review_surface(&hunks_owned, &signals);

        // H-52: deterministic finding-anchoring pass. For each surfaced finding,
        // call `anchor_finding` with the finding's file-scoped hunks to resolve
        // the hunk/line deterministically. The resolver uses finding.snippet
        // (the verbatim code fragment, NOT an empty string) so every
        // FindingAnchored row carries a snippet_blake3 derived from real content
        // (GD-2: distinguishable per finding; BLOCK-2 fix). The resolved
        // AnchorResult is stored back onto finding.anchor, wiring the hunk_id
        // join into the cockpit score (G-5 OQ#2 / BLOCK-4). Emits ONE
        // `finding_anchored` SSB row per finding (the off-agent artifact,
        // R1/R2). Gated by ELAI_REVIEW_ANCHOR_DISABLE kill-switch (DEFAULT-ON;
        // W-2 / R9). This is the REAL CLI entrypoint (R3/R7 reachability).
        if !anchor_disabled() {
            for finding in &mut score.findings {
                // Scope the hunks to the finding's file for the resolver.
                let file_hunks: Vec<_> = hunks_owned
                    .iter()
                    .filter(|h| h.file_path == finding.file_path)
                    .cloned()
                    .collect();

                // Use the REAL snippet from the finding (H-52 line-less schema).
                // An empty snippet resolves to FileFallback (file-level finding
                // with no specific code referent). Never pass "" here — that
                // makes every row FileFallback and the snippet_blake3 constant
                // (indistinguishable across findings, GD-2 violation).
                let anchor_result = anchor_finding(&finding.snippet, &file_hunks, None);

                // snake_case finding_id for join (session/file/kind triple).
                let finding_id = format!(
                    "{}/{}/{}",
                    session_owned,
                    finding.file_path,
                    finding.kind.label()
                );

                // blake3 hash of the REAL snippet bytes (GD-2: must prove the
                // anchor resolved against real content, not the empty-hash
                // constant). anti-stale: blake3 only per CLAUDE.md hard rule.
                let snippet_bytes = finding.snippet.as_bytes();
                let snippet_blake3 = {
                    let h = blake3::hash(snippet_bytes);
                    format!("{}", h.to_hex())
                };

                // Wire the anchor result back into the finding (G-5 OQ#2 /
                // BLOCK-4): the resolved hunk_id joins this finding to the
                // correct C-7 ChangeDecision row at hunk granularity. A
                // FileFallback / Ambiguous / Unanchorable anchor still populates
                // the field (hunk_id = None) so the cockpit can render at
                // file-level. This is the only place the anchor is set; the
                // model cannot reach this field.
                finding.anchor = Some(anchor_result.clone());

                let anchor_kind = match anchor_result.kind {
                    AnchorKind::Anchored => FindingAnchoredKind::Anchored,
                    AnchorKind::FileFallback => FindingAnchoredKind::FileFallback,
                    AnchorKind::Ambiguous => FindingAnchoredKind::Ambiguous,
                    AnchorKind::Unanchorable => FindingAnchoredKind::Unanchorable,
                    AnchorKind::Relocated => FindingAnchoredKind::Relocated,
                };

                let anchor_payload = SsbPayload::FindingAnchored(FindingAnchoredPayload {
                    finding_id,
                    snippet_blake3,
                    resolved_hunk_id: anchor_result.hunk_id,
                    new_line_start: anchor_result.new_line_range.map(|(start, _)| start),
                    kind: anchor_kind,
                    candidate_match_n: anchor_result.candidate_match_n,
                });

                // Write the off-agent artifact through the SAME live Ssb handle
                // (R4). A failure to write is logged as a warning but does NOT
                // abort the review — the DSR score is the primary product.
                if let Err(e) = ssb
                    .write_payload("cli:review-anchor", 0, anchor_payload, Vec::new())
                    .await
                {
                    tracing::warn!(
                        "review --cockpit: write finding_anchored row for {}: {e}",
                        finding.file_path
                    );
                }
            }
        }

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
