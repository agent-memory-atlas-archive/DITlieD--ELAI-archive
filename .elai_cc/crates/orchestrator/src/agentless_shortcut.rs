//! E-46 — AGENTLESS leaf-shortcut eligibility heuristics.
//!
//! Encodes the AGENTLESS shortcut rule: at the leaf level of the
//! decomposition tree, certain tasks can be executed WITHOUT spawning
//! a sub-agent (the orchestrator runs the localize > repair > validate
//! triple inline). This module owns the eligibility predicate; the
//! actual triple-phase dispatch lives in `task_runner` (out of scope
//! for E-46; only the eligibility decision + SSB row emission are
//! shipped here).
//!
//! Eligibility predicates (closed list — adding a new heuristic
//! requires an additive variant on [`Eligibility`]):
//!   1. Single-file change — `task.criteria` mentions exactly one
//!      file path.
//!   2. Small LOC bound — the task's `expected_loc` (when present in
//!      criteria) is `<= MAX_SHORTCUT_LOC`.
//!   3. No cross-crate fan-out — no criterion contains the substring
//!      "cross-crate" / "across crates".
//!   4. No async network ops — no criterion mentions
//!      `tokio::net` / `reqwest` / `async fn` / `await`.
//!
//! The heuristics are deliberately conservative: a `Yes` result is
//! always safe to short-circuit; a `No` result preserves the standard
//! sub-agent path. False-negatives are acceptable, false-positives are
//! not.
//!
//! Grounding: arxiv:2407.01489 (AGENTLESS — Xia et al, 2024). The
//! paper reports 32.00% SWE-bench Lite at $0.70/issue using a
//! non-agentic 3-phase localize/repair/validate pipeline, beating
//! every open-source agentic system at that time. Direct evidence
//! that less decomposition wins on narrow, well-localized fixes.
//!
//! Wiring contract (HARD):
//!   1. [`shortcut_eligible`] is a pure predicate: no SSB emission, no
//!      I/O, no clock reads. The producer call lives in
//!      `agent_pool::process_task_advancement` (see
//!      `emit_agentless_shortcut_taken`).
//!   2. Adding heuristic categories requires bumping the
//!      [`ShortcutReason`] enum additively and updating the SSB
//!      payload's snake-case label set. The dashboard histograms
//!      consume `payload.reason` so a new variant means a new
//!      bar.
//!   3. `Eligibility::No(reason)` carries a human-readable reason that
//!      is logged at `debug` level only — it is NOT emitted to SSB
//!      (no point flooding the bus with negative results; the
//!      dashboard only renders the positive path).
//!
//! Anti-stale-training: tracing for logs, thiserror not required
//! (no fallible API), no `async-trait`, no new external dep.

use std::path::Path;

use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::dual_ledger::TaskLedger;

// ---------------------------------------------------------------------------
// Tunables
// ---------------------------------------------------------------------------

/// Maximum expected LOC for an AGENTLESS shortcut. Above this, the
/// task falls back to the standard sub-agent path. Conservative
/// default — easy to tune via config later.
pub const MAX_SHORTCUT_LOC: u32 = 50;

// ---------------------------------------------------------------------------
// Closed enums
// ---------------------------------------------------------------------------

/// Eligibility verdict. `Yes` carries the canonical reason label that
/// the SSB payload propagates; `No` carries a free-form reason for
/// logging only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Eligibility {
    Yes(ShortcutReason),
    No(String),
}

impl Eligibility {
    /// True for the positive verdict; convenience for branch
    /// predicates at call sites.
    #[must_use]
    pub fn is_eligible(&self) -> bool {
        matches!(self, Self::Yes(_))
    }

    /// Reason label (`Yes`) or rejection text (`No`). Used by
    /// dashboard taxonomy histograms.
    #[must_use]
    pub fn reason_label(&self) -> String {
        match self {
            Self::Yes(r) => r.as_str().to_owned(),
            Self::No(s) => s.clone(),
        }
    }
}

/// Closed sum of canonical shortcut-reason labels. Snake-case wire
/// labels match the SSB payload's `reason` field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShortcutReason {
    /// Single-file change with no cross-crate / network / LOC concerns.
    /// Canonical AGENTLESS case.
    SingleFileSmall,
    /// Single-file change but LOC unknown — caller asserted the path
    /// constraint, we accept on faith of the criterion list.
    SingleFile,
    /// Criteria list mentions only LOC bounded under the threshold and
    /// no other heuristic failed. Used when the file path is implicit
    /// (e.g. a typo fix expressed as "<=10 LOC change").
    SmallLoc,
}

impl ShortcutReason {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SingleFileSmall => "single_file_small",
            Self::SingleFile => "single_file",
            Self::SmallLoc => "small_loc",
        }
    }
}

// ---------------------------------------------------------------------------
// Predicate
// ---------------------------------------------------------------------------

/// Pure eligibility predicate. Returns `Eligibility::Yes(reason)` when
/// every heuristic is satisfied; otherwise `Eligibility::No(reason)`
/// with the first failing check.
///
/// Heuristic order matches the AGENTLESS paper's three-phase
/// pipeline's prerequisite list: localize requires single-file, repair
/// requires small LOC, validate requires no async-network. Failing
/// the easiest check first keeps the predicate cheap on the negative
/// path.
///
/// `task.criteria` is the sole input — the predicate reads no other
/// state. Tests can construct a `TaskLedger` directly and assert the
/// outcome without standing up the full orchestrator.
#[must_use]
pub fn shortcut_eligible(task: &TaskLedger) -> Eligibility {
    // 1. Cross-crate ban (cheapest check — substring match).
    for c in &task.criteria {
        let lc = c.to_ascii_lowercase();
        if lc.contains("cross-crate") || lc.contains("across crates") {
            return Eligibility::No(format!("cross_crate_in_criterion: {c}"));
        }
        if lc.contains("async fn")
            || lc.contains("await")
            || lc.contains("tokio::net")
            || lc.contains("reqwest")
        {
            return Eligibility::No(format!("async_network_in_criterion: {c}"));
        }
    }

    // 2. File-count heuristic — count unique non-empty path-like
    //    tokens. We look for either explicit `file=...` markers or
    //    raw paths matching `<crate>/...` shape.
    let file_paths = extract_file_paths(&task.criteria);
    let unique_files = unique_count(&file_paths);

    // 3. LOC heuristic — look for an `loc<=N` / `loc=N` / `N LOC`
    //    marker in any criterion.
    let loc = extract_loc_bound(&task.criteria);

    // Decision matrix — order matters; first canonical case wins.
    if unique_files == 1 {
        if let Some(n) = loc {
            if n > MAX_SHORTCUT_LOC {
                return Eligibility::No(format!("loc_exceeds: {n} > {MAX_SHORTCUT_LOC}"));
            }
            debug!(
                task_id = %task.task_id.0,
                loc = n,
                unique_files,
                "agentless_shortcut::shortcut_eligible: SingleFileSmall"
            );
            return Eligibility::Yes(ShortcutReason::SingleFileSmall);
        }
        debug!(
            task_id = %task.task_id.0,
            unique_files,
            "agentless_shortcut::shortcut_eligible: SingleFile (loc unknown)"
        );
        return Eligibility::Yes(ShortcutReason::SingleFile);
    }

    if unique_files == 0 {
        if let Some(n) = loc {
            if n <= MAX_SHORTCUT_LOC {
                debug!(
                    task_id = %task.task_id.0,
                    loc = n,
                    "agentless_shortcut::shortcut_eligible: SmallLoc"
                );
                return Eligibility::Yes(ShortcutReason::SmallLoc);
            }
            return Eligibility::No(format!("loc_exceeds: {n} > {MAX_SHORTCUT_LOC}"));
        }
        return Eligibility::No("no_file_no_loc".into());
    }

    Eligibility::No(format!("multi_file: {unique_files}"))
}

/// Convenience wrapper around [`shortcut_eligible`] that returns the
/// `Yes` variant's reason label for direct SSB-payload embedding. Use
/// at the producer site (see `agent_pool::emit_agentless_shortcut_taken`).
#[must_use]
pub fn shortcut_reason_label(task: &TaskLedger) -> Option<ShortcutReason> {
    match shortcut_eligible(task) {
        Eligibility::Yes(r) => Some(r),
        Eligibility::No(_) => None,
    }
}

// ---------------------------------------------------------------------------
// Helpers (pub(crate) for test access)
// ---------------------------------------------------------------------------

/// Pull file-path-shaped tokens out of acceptance criteria. We accept
/// two shapes:
///   - explicit `file=path/to/file.rs` marker
///   - any token containing a `/` AND ending in `.rs|.py|.ts|.svelte`
fn extract_file_paths(criteria: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for c in criteria {
        for token in c.split_whitespace() {
            // explicit marker
            if let Some(rest) = token.strip_prefix("file=") {
                let p = rest.trim_matches(|ch: char| ch == ',' || ch == ';' || ch == ')' || ch == '(');
                if !p.is_empty() {
                    out.push(p.to_owned());
                    continue;
                }
            }
            // raw path
            if token.contains('/')
                && (token.ends_with(".rs")
                    || token.ends_with(".py")
                    || token.ends_with(".ts")
                    || token.ends_with(".svelte")
                    || token.ends_with(".toml"))
            {
                let p = token.trim_matches(|ch: char| ch == ',' || ch == ';' || ch == ')' || ch == '(' || ch == '`');
                if !p.is_empty() {
                    out.push(p.to_owned());
                }
            }
        }
    }
    // Normalise path separators to forward slash for cross-platform
    // stability (matches LockedKeysAuditor convention).
    for p in &mut out {
        *p = p.replace('\\', "/");
    }
    out
}

fn unique_count(paths: &[String]) -> usize {
    let mut set: Vec<&str> = Vec::with_capacity(paths.len());
    for p in paths {
        if !set.iter().any(|q| *q == p.as_str()) {
            set.push(p.as_str());
        }
    }
    set.len()
}

/// Extract an LOC bound from criteria. Looks for either `loc<=N`,
/// `loc=N`, or `N LOC` (case-insensitive on the LOC token).
fn extract_loc_bound(criteria: &[String]) -> Option<u32> {
    for c in criteria {
        let lc = c.to_ascii_lowercase();
        // loc<=N
        if let Some(rest) = lc.split_once("loc<=").map(|(_, r)| r) {
            if let Some(n) = parse_leading_u32(rest) {
                return Some(n);
            }
        }
        // loc=N
        if let Some(rest) = lc.split_once("loc=").map(|(_, r)| r) {
            if let Some(n) = parse_leading_u32(rest) {
                return Some(n);
            }
        }
        // N LOC pattern — scan for digit run followed by "loc"
        for (i, ch) in lc.char_indices() {
            if ch.is_ascii_digit() {
                let tail = &lc[i..];
                if let Some(n) = parse_leading_u32(tail) {
                    let after = tail.trim_start_matches(|c: char| c.is_ascii_digit());
                    let after = after.trim_start();
                    if after.starts_with("loc") {
                        return Some(n);
                    }
                }
            }
        }
    }
    None
}

fn parse_leading_u32(s: &str) -> Option<u32> {
    let digits: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<u32>().ok()
}

/// Validate that a path is under the workspace (no `..` escapes). Not
/// used inside the predicate — exposed for the producer site to
/// reject malformed criteria before emitting an `agentless_shortcut_taken`
/// row. Pure function, no I/O.
#[must_use]
pub fn path_is_safe(p: &str) -> bool {
    let path = Path::new(p);
    !path.components().any(|c| {
        matches!(
            c,
            std::path::Component::ParentDir | std::path::Component::RootDir
        )
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dual_ledger::{TaskId, TaskLedger};

    fn task(criteria: Vec<&str>) -> TaskLedger {
        TaskLedger::new(
            TaskId::from("t-1"),
            None,
            criteria.into_iter().map(|s| s.to_owned()).collect(),
            0,
        )
    }

    #[test]
    fn single_file_small_loc_is_eligible() {
        let t = task(vec!["file=crates/foo/src/bar.rs", "loc<=20"]);
        let r = shortcut_eligible(&t);
        assert_eq!(r, Eligibility::Yes(ShortcutReason::SingleFileSmall));
    }

    #[test]
    fn raw_path_token_works() {
        let t = task(vec!["edit crates/foo/src/bar.rs", "loc=10"]);
        let r = shortcut_eligible(&t);
        assert_eq!(r, Eligibility::Yes(ShortcutReason::SingleFileSmall));
    }

    #[test]
    fn single_file_no_loc_falls_through_to_single_file() {
        let t = task(vec!["file=crates/foo/src/bar.rs"]);
        let r = shortcut_eligible(&t);
        assert_eq!(r, Eligibility::Yes(ShortcutReason::SingleFile));
    }

    #[test]
    fn multi_file_rejected() {
        let t = task(vec![
            "file=crates/foo/src/a.rs",
            "file=crates/foo/src/b.rs",
        ]);
        let r = shortcut_eligible(&t);
        assert!(!r.is_eligible(), "expected No, got {r:?}");
    }

    #[test]
    fn cross_crate_rejected() {
        let t = task(vec!["touches cross-crate boundary"]);
        let r = shortcut_eligible(&t);
        assert!(matches!(r, Eligibility::No(_)));
    }

    #[test]
    fn async_network_rejected() {
        let t = task(vec!["file=crates/foo/src/a.rs", "uses reqwest"]);
        let r = shortcut_eligible(&t);
        assert!(matches!(r, Eligibility::No(_)));
    }

    #[test]
    fn oversize_loc_rejected() {
        let t = task(vec!["file=crates/foo/src/a.rs", "loc=500"]);
        let r = shortcut_eligible(&t);
        assert!(matches!(r, Eligibility::No(_)));
    }

    #[test]
    fn small_loc_no_file_eligible() {
        let t = task(vec!["loc=10"]);
        let r = shortcut_eligible(&t);
        assert_eq!(r, Eligibility::Yes(ShortcutReason::SmallLoc));
    }

    #[test]
    fn n_loc_pattern_recognized() {
        let t = task(vec!["file=crates/foo/src/a.rs", "12 LOC change"]);
        let r = shortcut_eligible(&t);
        assert_eq!(r, Eligibility::Yes(ShortcutReason::SingleFileSmall));
    }

    #[test]
    fn path_is_safe_blocks_traversal() {
        assert!(path_is_safe("crates/foo/src/a.rs"));
        assert!(!path_is_safe("../escape"));
        assert!(!path_is_safe("/abs/path"));
    }

    #[test]
    fn reason_label_round_trip() {
        let r = Eligibility::Yes(ShortcutReason::SingleFileSmall);
        assert_eq!(r.reason_label(), "single_file_small");
    }
}
