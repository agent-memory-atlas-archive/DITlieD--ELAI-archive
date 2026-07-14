//! K-3: deterministic PR/CI evidence rendering.
//!
//! This module is a consume-only renderer. Gate verdicts remain owned by the
//! RRA manifest, hunk vouch state remains owned by `review_surface`, and
//! finding placement remains owned by the H-52 `anchor_finding` resolver that
//! populated each `ReviewFinding::anchor` before this renderer is called.

use crate::review_anchor::AnchorKind;
use crate::review_surface::{ReviewFinding, ReviewSurfaceScore};
use crate::rra::RunArtifact;

/// Render one RRA gate summary plus the complete review-surface hunk set as
/// deterministic Markdown suitable for the existing GitHub PR body/comment
/// seam.
///
/// Every hunk in `review_surface.hunks` is rendered. An empty `vouched_by`
/// collection always produces an explicit `UNVOUCHED` band; missing evidence
/// is never represented as clean. Finding line ranges are rendered only from
/// H-52 resolver-owned anchors.
#[must_use]
pub fn render_pr_evidence(
    run_artifact: &RunArtifact,
    review_surface: &ReviewSurfaceScore,
) -> String {
    let mut output = String::new();
    output.push_str("# ELAI PR Evidence\n\n");
    output.push_str("## Check-run gate summary\n\n");
    output.push_str(&format!(
        "- Session: `{}`\n- RRA root (blake3): `{}`\n- Guarantee: `{}`\n",
        inline_code(&run_artifact.session_id),
        inline_code(&run_artifact.root_blake3),
        guarantee_label(run_artifact),
    ));
    output.push_str(
        "- Offline verification: `elai replay --verify <artifact.rra>`\n\n",
    );

    if run_artifact.manifest.is_empty() {
        output.push_str(
            "> **UNVERIFIED — no gate verdict hashes are present in the RRA manifest.**\n\n",
        );
    } else {
        output.push_str("| Gate | Action | Verdict blake3 | SSB seq |\n");
        output.push_str("| --- | --- | --- | ---: |\n");
        for verdict in &run_artifact.manifest {
            output.push_str(&format!(
                "| `{}` | `{}` | `{}` | {} |\n",
                table_cell(&verdict.gate),
                table_cell(&verdict.action_type),
                table_cell(&verdict.verdict_blake3),
                verdict.ssb_event_seq,
            ));
        }
        output.push('\n');
    }

    output.push_str("## Review surface\n\n");
    output.push_str(&format!(
        "- DSR: `{:.6}`\n- Changed lines: `{}`\n- Vouched hunks: `{}`\n- Unvouched hunks: `{}`\n\n",
        review_surface.dsr,
        review_surface.total_changed_lines,
        review_surface.vouched_hunks,
        review_surface.unvouched_hunks,
    ));

    if review_surface.hunks.is_empty() {
        output.push_str(
            "> **UNVOUCHED — the review surface contains no closed-set hunks.**\n\n",
        );
    }

    for hunk in &review_surface.hunks {
        let is_vouched = !hunk.vouched_by.is_empty() && !hunk.needs_attention;
        let band = if is_vouched { "VOUCHED" } else { "UNVOUCHED" };
        output.push_str(&format!(
            "### {band} — `{}`\n\n- Hunk: `{}`\n- Changed lines: `{}`\n",
            inline_code(&hunk.file_path),
            inline_code(&hunk.hunk_id),
            hunk.lines_changed,
        ));

        if is_vouched {
            let labels = hunk
                .vouched_by
                .iter()
                .map(|kind| format!("`{}`", kind.label()))
                .collect::<Vec<_>>()
                .join(", ");
            output.push_str(&format!("- Vouched by: {labels}\n"));
        } else {
            output.push_str(
                "- Evidence band: **UNVOUCHED — full reviewer attention required.**\n",
            );
        }

        let findings = findings_for_hunk(&review_surface.findings, &hunk.hunk_id);
        if findings.is_empty() {
            output.push_str("- Anchored findings: none\n\n");
        } else {
            output.push_str("- Anchored findings:\n");
            for finding in findings {
                output.push_str(&format!("  - {}\n", render_finding(finding)));
            }
            output.push('\n');
        }
    }

    let file_level_findings = review_surface
        .findings
        .iter()
        .filter(|finding| {
            finding
                .anchor
                .as_ref()
                .and_then(|anchor| anchor.hunk_id.as_deref())
                .is_none()
        })
        .collect::<Vec<_>>();
    if !file_level_findings.is_empty() {
        output.push_str("## File-level findings\n\n");
        for finding in file_level_findings {
            output.push_str(&format!("- {}\n", render_finding(finding)));
        }
        output.push('\n');
    }

    output
}

fn findings_for_hunk<'a>(
    findings: &'a [ReviewFinding],
    hunk_id: &str,
) -> Vec<&'a ReviewFinding> {
    findings
        .iter()
        .filter(|finding| {
            finding
                .anchor
                .as_ref()
                .and_then(|anchor| anchor.hunk_id.as_deref())
                == Some(hunk_id)
        })
        .collect()
}

fn render_finding(finding: &ReviewFinding) -> String {
    let file = inline_code(&finding.file_path);
    let kind = finding.kind.label();
    match finding.anchor.as_ref() {
        Some(anchor) => {
            let placement = match (anchor.kind, anchor.new_line_range) {
                (AnchorKind::Anchored, Some((start, end))) => {
                    format!("resolver-owned new-side lines {start}-{end}")
                }
                _ => format!("resolver status `{}` (file-level)", anchor.kind.as_str()),
            };
            format!("`{kind}` in `{file}` — {placement}")
        }
        None => format!("`{kind}` in `{file}` — anchor missing; file-level only"),
    }
}

fn guarantee_label(run_artifact: &RunArtifact) -> &'static str {
    use crate::rra::GuaranteeLevel;

    match run_artifact.guarantee_level {
        GuaranteeLevel::ReproducibleTokens => "reproducible_tokens",
        GuaranteeLevel::ReproducibleActionStream => "reproducible_action_stream",
        GuaranteeLevel::VerifyDivergence => "verify_divergence",
    }
}

fn inline_code(value: &str) -> String {
    value
        .replace('`', "\\`")
        .replace('\r', " ")
        .replace('\n', " ")
}

fn table_cell(value: &str) -> String {
    inline_code(value).replace('|', "\\|")
}
