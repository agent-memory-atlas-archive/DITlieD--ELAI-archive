//! G-5 — reviewer-cockpit Diff-Surface-to-Review (DSR) scoring engine.
//!
//! Computes the headline DSR for a session's working-tree changes:
//!
//! ```text
//! DSR = sum(changed lines of UN-vouched hunks) / sum(all changed lines)
//! ```
//!
//! A hunk is VOUCHED when its file carries a clean / passing DETERMINISTIC
//! signal already recorded in the SSB stream:
//!   * a passing F-4 [`crate::ssb::TestIntegrityEventPayload`]
//!     (`kind == Verified`, `passed == true`, `stdout_hash.is_some()`), OR
//!   * (forward-compatible) a clean write-gate state.
//!
//! A hunk is UN-VOUCHED (needs full attention) when it has NO vouch, OR when its
//! file carries a FIRED deterministic gate verdict (F-1
//! [`crate::ssb::VibeGateBlockedPayload`] / F-2a
//! [`crate::ssb::SecurityRuleBlockedPayload`]) or a FAILING test (a
//! `TestIntegrityEvent` with `passed == false` or `kind == FabricationBlocked`).
//! A fired gate / failing test is ALSO surfaced as a [`ReviewFinding`].
//!
//! SILENT-BY-DEFAULT (DD7): a finding is surfaced ONLY when backed by a fired
//! deterministic gate verdict or a failing test. There is NO LLM-opinion branch;
//! a clean file with no signal produces NO finding (it is simply un-vouched and
//! counts toward the DSR numerator, but is not flagged as a defect).
//!
//! DETERMINISM (GD-2): [`score_review_surface`] is a PURE function of its two
//! inputs (the parsed git-diff hunks + the decoded SSB vouch/finding facts). The
//! caller sorts both inputs by a stable key before calling, so the DSR
//! recomputes BYTE-IDENTICALLY from the same pinned git-diff bytes + the same
//! seq-ASC SSB stream.
//!
//! GRANULARITY (DD2, honest GD-5): vouching is HUNK/FILE-level, not per-line. A
//! hunk inherits its file's vouch state (the F-4 proof token carries
//! `test_name`, not a covered file/hunk path, so a precise per-line test->hunk
//! coverage edge is NOT claimed). DSR is computed at hunk granularity.
//!
//! GD-1: this module CONSUMES the F-4 / F-1 / F-2a verdicts already in the SSB
//! stream; it re-implements NONE of those gates. The caller decodes the stream
//! and hands this function the file-level facts.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::focus_tunnel::diff::DiffHunk;
use crate::ssb::{SsbEntry, SsbPayload, TestIntegrityEventKind};

/// Why a hunk's file was vouched (clean / passing deterministic signal). The
/// kinds are NOT mutually exclusive — a file may carry several. Stored sorted +
/// de-duplicated so the [`HunkVouch`] serializes deterministically (GD-2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VouchKind {
    /// A passing F-4 `TestIntegrityEvent` (Verified, passed, stdout_hash present)
    /// touched this file. The stdout-hash binding is the off-agent proof token.
    TestStdoutHash,
    /// A clean F-1/F-2a write-gate state for this file (no fired block). Reserved
    /// for the forward-compatible clean-verdict signal; the present producers
    /// (VibeGateBlocked / SecurityRuleBlocked) only emit a row when they FIRE, so
    /// a clean write-gate vouch is derived from the ABSENCE of a fired row, which
    /// the caller represents by simply not adding this kind. Kept in the enum so
    /// the cockpit DTO has a stable shape if a clean-verdict producer is added.
    WriteGateClean,
}

impl VouchKind {
    /// Stable snake_case wire label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::TestStdoutHash => "test_stdout_hash",
            Self::WriteGateClean => "write_gate_clean",
        }
    }
}

/// Why a file was surfaced as needing attention (silent-by-default, DD7): ONLY a
/// fired deterministic gate verdict or a failing test. NEVER an LLM opinion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingKind {
    /// An F-1 `VibeGateBlocked` write-gate verdict fired for this file.
    VibeGateFired,
    /// An F-2a `SecurityRuleBlocked` write-gate verdict fired for this file.
    SecurityRuleFired,
    /// An F-4 `TestIntegrityEvent` recorded a failing / fabrication-blocked test.
    TestFailed,
}

impl FindingKind {
    /// Stable snake_case wire label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::VibeGateFired => "vibe_gate_fired",
            Self::SecurityRuleFired => "security_rule_fired",
            Self::TestFailed => "test_failed",
        }
    }
}

/// File-level deterministic facts decoded from the SSB stream, keyed by the
/// project-relative file path. Built by the CLI driver from
/// `query_session_stream_asc`; the [`score_review_surface`] function consumes it
/// read-only. This is the GD-1 seam: the gates already ran and left these rows;
/// G-5 only reads them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileSignals {
    /// `true` when a passing F-4 `TestIntegrityEvent` (Verified, passed,
    /// stdout_hash present) is attributed to this file.
    pub passing_test_stdout_hash: bool,
    /// `true` when an F-1 `VibeGateBlocked` row fired for this file.
    pub vibe_gate_fired: bool,
    /// `true` when an F-2a `SecurityRuleBlocked` row fired for this file.
    pub security_rule_fired: bool,
    /// `true` when an F-4 `TestIntegrityEvent` recorded a failing /
    /// fabrication-blocked test attributed to this file.
    pub test_failed: bool,
}

/// One scored hunk. Granularity is hunk/file-level (DD2): the hunk inherits its
/// file's vouch state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HunkVouch {
    /// Stable per-hunk identifier (`<path>#<old_start>-<new_start>`), derived
    /// from the diff header so the cockpit can key on it deterministically.
    pub hunk_id: String,
    /// Project-relative file path the hunk belongs to.
    pub file_path: String,
    /// Sorted, de-duplicated vouch reasons (empty when un-vouched).
    pub vouched_by: Vec<VouchKind>,
    /// Total changed lines in the hunk (added + removed).
    pub lines_changed: u32,
    /// `true` when the hunk has NO vouch (CONSERVATIVE — needs full attention).
    pub needs_attention: bool,
}

/// A surfaced finding (silent-by-default, DD7). Backed ONLY by a fired
/// deterministic gate verdict or a failing test.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewFinding {
    /// Project-relative file path the finding applies to.
    pub file_path: String,
    /// Why this file was surfaced.
    pub kind: FindingKind,
}

/// The full reviewer-cockpit score for one session's working-tree changes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReviewSurfaceScore {
    /// Per-hunk vouch state, sorted by `hunk_id` (deterministic order, GD-2).
    pub hunks: Vec<HunkVouch>,
    /// Surfaced findings, sorted by `(file_path, kind)` (deterministic order).
    pub findings: Vec<ReviewFinding>,
    /// Diff-Surface-to-Review ratio in `[0.0, 1.0]`: `attention_lines /
    /// total_changed_lines` (`0.0` when there are zero changed lines).
    pub dsr: f64,
    /// DSR denominator: sum of changed lines across all hunks.
    pub total_changed_lines: u32,
    /// DSR numerator: changed lines belonging to un-vouched hunks.
    pub attention_lines: u32,
    /// Count of vouched hunks.
    pub vouched_hunks: u32,
    /// Count of un-vouched hunks.
    pub unvouched_hunks: u32,
}

/// Build the stable per-hunk id from a diff header. Deterministic: a pure
/// function of the hunk's path + line origins, so the same diff yields the same
/// id every time (GD-2).
#[must_use]
pub fn hunk_id_for(hunk: &DiffHunk) -> String {
    format!("{}#{}-{}", hunk.file_path, hunk.old_start, hunk.new_start)
}

/// Total changed lines in a hunk: added + removed (context lines are unchanged
/// and do NOT count toward the DSR). Uses the legacy 3-bucket fields, which the
/// parser keeps populated in lockstep with `body_order`.
#[must_use]
fn hunk_changed_lines(hunk: &DiffHunk) -> u32 {
    let added = u32::try_from(hunk.added_lines.len()).unwrap_or(u32::MAX);
    let removed = u32::try_from(hunk.removed_lines.len()).unwrap_or(u32::MAX);
    added.saturating_add(removed)
}

/// Compute the reviewer-cockpit DSR score from the parsed git-diff hunks + the
/// per-file deterministic signals decoded from the SSB stream.
///
/// PURE + DETERMINISTIC (GD-2): the output is a function of the two inputs only.
/// Hunks are scored in `hunk_id` order and findings in `(file_path, kind)`
/// order, so the score serializes byte-identically across re-runs given the
/// same inputs.
///
/// SILENT-BY-DEFAULT (DD7): findings are emitted ONLY for files carrying a fired
/// gate verdict or a failing test; a clean-but-unvouched file produces NO
/// finding (it still counts toward the DSR numerator).
#[must_use]
pub fn score_review_surface(
    hunks: &[DiffHunk],
    signals: &BTreeMap<String, FileSignals>,
) -> ReviewSurfaceScore {
    let mut scored: Vec<HunkVouch> = Vec::with_capacity(hunks.len());
    let mut total_changed_lines: u32 = 0;
    let mut attention_lines: u32 = 0;
    let mut vouched_hunks: u32 = 0;
    let mut unvouched_hunks: u32 = 0;

    for hunk in hunks {
        let lines_changed = hunk_changed_lines(hunk);
        total_changed_lines = total_changed_lines.saturating_add(lines_changed);

        let mut vouched_by: Vec<VouchKind> = Vec::new();
        if let Some(sig) = signals.get(&hunk.file_path) {
            // A vouch is ONLY granted by a clean/passing signal. A fired gate or
            // failing test does NOT vouch (it makes the hunk need attention).
            if sig.passing_test_stdout_hash && !sig.test_failed {
                vouched_by.push(VouchKind::TestStdoutHash);
            }
        }
        vouched_by.sort_unstable();
        vouched_by.dedup();

        let needs_attention = vouched_by.is_empty();
        if needs_attention {
            attention_lines = attention_lines.saturating_add(lines_changed);
            unvouched_hunks = unvouched_hunks.saturating_add(1);
        } else {
            vouched_hunks = vouched_hunks.saturating_add(1);
        }

        scored.push(HunkVouch {
            hunk_id: hunk_id_for(hunk),
            file_path: hunk.file_path.clone(),
            vouched_by,
            lines_changed,
            needs_attention,
        });
    }

    scored.sort_by(|a, b| a.hunk_id.cmp(&b.hunk_id));

    // Silent-by-default findings: ONE per (file, fired-signal). Derived only from
    // deterministic gate verdicts / failing tests — never an LLM opinion.
    let mut findings: Vec<ReviewFinding> = Vec::new();
    for (file_path, sig) in signals {
        if sig.vibe_gate_fired {
            findings.push(ReviewFinding {
                file_path: file_path.clone(),
                kind: FindingKind::VibeGateFired,
            });
        }
        if sig.security_rule_fired {
            findings.push(ReviewFinding {
                file_path: file_path.clone(),
                kind: FindingKind::SecurityRuleFired,
            });
        }
        if sig.test_failed {
            findings.push(ReviewFinding {
                file_path: file_path.clone(),
                kind: FindingKind::TestFailed,
            });
        }
    }
    findings.sort_by(|a, b| {
        a.file_path
            .cmp(&b.file_path)
            .then_with(|| a.kind.label().cmp(b.kind.label()))
    });

    let dsr = if total_changed_lines == 0 {
        0.0
    } else {
        f64::from(attention_lines) / f64::from(total_changed_lines)
    };

    ReviewSurfaceScore {
        hunks: scored,
        findings,
        dsr,
        total_changed_lines,
        attention_lines,
        vouched_hunks,
        unvouched_hunks,
    }
}

/// Build the per-file [`FileSignals`] vouch/finding set from a session's
/// seq-ASC SSB stream + the `ssb_files` junction (files-per-seq).
///
/// This is the GD-1 seam: the F-4 / F-1 / F-2a gates already ran and left these
/// rows; G-5 only DECODES them. Re-implements NONE of those gates.
///
/// File attribution (DD2, honest GD-5):
///   * F-1 [`SsbPayload::VibeGateBlocked`] + F-2a
///     [`SsbPayload::SecurityRuleBlocked`] carry `file_path` directly — a fired
///     row marks that file as a finding (and un-vouches it).
///   * F-4 [`SsbPayload::TestIntegrityEvent`] carries `test_name`, NOT a file
///     path. It is attributed to the files the row touched (the `ssb_files`
///     junction). A `Verified` + `passed` + `stdout_hash.is_some()` event vouches
///     each touched file; a `passed == false` OR `FabricationBlocked` event marks
///     each touched file as failing (a finding). This is the documented file-
///     level heuristic — G-5 does NOT claim a precise per-line coverage edge.
///
/// DETERMINISM (GD-2): the `entries` are consumed in seq-ASC order and the
/// output `BTreeMap` is key-ordered, so the signal set re-derives byte-
/// identically from the same session stream.
#[must_use]
pub fn build_file_signals(
    entries: &[SsbEntry],
    files_by_seq: &BTreeMap<i64, Vec<String>>,
) -> BTreeMap<String, FileSignals> {
    let mut out: BTreeMap<String, FileSignals> = BTreeMap::new();
    for entry in entries {
        let payload: SsbPayload = match crate::ssb::decode_ssb_payload(&entry.payload) {
            Ok(p) => p,
            // A row we cannot decode (forward-compat / unrelated variant) is
            // silently skipped — it contributes no vouch and no finding.
            Err(_) => continue,
        };
        match payload {
            SsbPayload::VibeGateBlocked(p) => {
                out.entry(p.file_path).or_default().vibe_gate_fired = true;
            }
            SsbPayload::SecurityRuleBlocked(p) => {
                out.entry(p.file_path).or_default().security_rule_fired = true;
            }
            SsbPayload::TestIntegrityEvent(p) => {
                let touched = files_by_seq.get(&entry.seq);
                let Some(files) = touched else {
                    // No file junction for this test row -> cannot attribute to a
                    // hunk; it neither vouches nor surfaces a file-level finding.
                    continue;
                };
                let is_failing =
                    !p.passed || matches!(p.kind, TestIntegrityEventKind::FabricationBlocked);
                let is_passing_vouch = matches!(p.kind, TestIntegrityEventKind::Verified)
                    && p.passed
                    && p.stdout_hash.is_some();
                for file in files {
                    let sig = out.entry(file.clone()).or_default();
                    if is_failing {
                        sig.test_failed = true;
                    }
                    if is_passing_vouch {
                        sig.passing_test_stdout_hash = true;
                    }
                }
            }
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::focus_tunnel::diff::DiffHunk;

    fn hunk(path: &str, old_start: usize, new_start: usize, added: usize, removed: usize) -> DiffHunk {
        DiffHunk {
            file_path: path.to_string(),
            old_start,
            old_count: removed,
            new_start,
            new_count: added,
            context_lines: Vec::new(),
            added_lines: vec!["x".to_string(); added],
            removed_lines: vec!["y".to_string(); removed],
            body_order: Vec::new(),
        }
    }

    fn vouched() -> FileSignals {
        FileSignals {
            passing_test_stdout_hash: true,
            ..FileSignals::default()
        }
    }

    #[test]
    fn dsr_is_unvouched_over_total() {
        // 2 hunks, 10 changed lines total. one file vouched (4 lines), one not (6).
        let hunks = vec![hunk("a.rs", 1, 1, 2, 2), hunk("b.rs", 1, 1, 3, 3)];
        let mut sig = BTreeMap::new();
        sig.insert("a.rs".to_string(), vouched());
        let score = score_review_surface(&hunks, &sig);
        assert_eq!(score.total_changed_lines, 10);
        assert_eq!(score.attention_lines, 6);
        assert_eq!(score.vouched_hunks, 1);
        assert_eq!(score.unvouched_hunks, 1);
        assert!((score.dsr - 0.6).abs() < 1e-12, "dsr {} != 0.6", score.dsr);
    }

    #[test]
    fn no_signal_file_is_unvouched_but_not_a_finding() {
        // silent-by-default: a clean file with NO signal is un-vouched (counts to
        // the DSR numerator) but produces NO finding.
        let hunks = vec![hunk("clean.rs", 1, 1, 5, 0)];
        let sig = BTreeMap::new();
        let score = score_review_surface(&hunks, &sig);
        assert_eq!(score.unvouched_hunks, 1);
        assert!(score.findings.is_empty(), "clean unvouched file must NOT be a finding");
        assert!((score.dsr - 1.0).abs() < 1e-12);
    }

    #[test]
    fn fired_gate_surfaces_a_finding_and_unvouches() {
        let hunks = vec![hunk("danger.rs", 1, 1, 4, 0)];
        let mut sig = BTreeMap::new();
        sig.insert(
            "danger.rs".to_string(),
            FileSignals {
                vibe_gate_fired: true,
                ..FileSignals::default()
            },
        );
        let score = score_review_surface(&hunks, &sig);
        assert_eq!(score.findings.len(), 1);
        assert_eq!(score.findings[0].kind, FindingKind::VibeGateFired);
        assert_eq!(score.unvouched_hunks, 1, "a fired gate never vouches");
    }

    #[test]
    fn failing_test_overrides_passing_vouch() {
        // a file with both a passing-test signal AND a failing-test signal is
        // NOT vouched (test_failed wins, conservative).
        let hunks = vec![hunk("flaky.rs", 1, 1, 4, 0)];
        let mut sig = BTreeMap::new();
        sig.insert(
            "flaky.rs".to_string(),
            FileSignals {
                passing_test_stdout_hash: true,
                test_failed: true,
                ..FileSignals::default()
            },
        );
        let score = score_review_surface(&hunks, &sig);
        assert_eq!(score.unvouched_hunks, 1);
        assert_eq!(score.findings.len(), 1);
        assert_eq!(score.findings[0].kind, FindingKind::TestFailed);
    }

    #[test]
    fn zero_changed_lines_is_dsr_zero() {
        let score = score_review_surface(&[], &BTreeMap::new());
        assert_eq!(score.total_changed_lines, 0);
        assert!((score.dsr - 0.0).abs() < 1e-12, "empty diff = nothing to review = DSR 0");
    }

    #[test]
    fn recompute_is_byte_identical() {
        // GD-2: same inputs -> byte-identical serialized score.
        let hunks = vec![hunk("b.rs", 5, 5, 3, 1), hunk("a.rs", 1, 1, 2, 0)];
        let mut sig = BTreeMap::new();
        sig.insert("a.rs".to_string(), vouched());
        let s1 = score_review_surface(&hunks, &sig);
        let s2 = score_review_surface(&hunks, &sig);
        let j1 = serde_json::to_string(&s1).expect("serialize s1");
        let j2 = serde_json::to_string(&s2).expect("serialize s2");
        assert_eq!(j1, j2, "DSR score must recompute byte-identically (GD-2)");
        // hunks sorted by hunk_id deterministically (a.rs before b.rs).
        assert_eq!(s1.hunks[0].file_path, "a.rs");
        assert_eq!(s1.hunks[1].file_path, "b.rs");
    }
}
