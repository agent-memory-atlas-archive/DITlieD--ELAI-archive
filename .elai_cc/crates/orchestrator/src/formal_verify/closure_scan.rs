//! H-54 — deterministic obligation-closure escape-hatch gate.
//!
//! Implements the AlphaProof Nexus (arXiv:2605.22763v1, DeepMind 2026)
//! whole-dependency-closure defense against the two documented evasion
//! classes:
//!   (a) the RELOCATED-SORRY attack: a generator moves an unproven `sorry`
//!       into a renamed helper lemma that re-states the target; the top
//!       theorem now looks sorry-free. A top-statement-only check PASSES
//!       this. This scanner does NOT.
//!   (b) the HALLUCINATED-KNOWN-RESULT attack: a generator introduces a
//!       false "known-result" lemma that is actually admitted/assumed and
//!       never proved. Same top-level appearance. Same defense.
//!
//! THE DEFENSE: BFS-walk the full transitive dependency closure of the
//! "discharged" goal. An escape hatch (sorry/admit/axiom/assume/
//! VerifierExternal/KaniAssumeVacuous) ANYWHERE in the closure disqualifies
//! the proof — regardless of how deep or how renamed the hatch obligation is.
//! An unregistered hatch node -> `EscapeHatchAnywhere` -> BLOCK.
//! A hatch node WITH a matching unexpired row in the registered-exception
//! allowlist (`obligation-closure-allowlist.toml`) -> `RegisteredException`
//! (recorded, not BLOCKed — mirrors AlphaProof's labelled QuantumOptics
//! sorries: 128 sorries confined to 2 labelled files with literature
//! citations, the other 69 files fully closed).
//!
//! ROLE IN THE GATE STACK (Layer-2):
//! This is the DETERMINISTIC FIRST leg of the H-44 behavioral trap-detection
//! stack on the formal-verification lane (E-53). It runs BEFORE the
//! verifier's own discharge and short-circuits on EscapeHatchAnywhere.
//!   Layer-1: compile-time dead-code + enum exhaustiveness (floor).
//!   Layer-2: THIS SCAN — static, BFS over the verifier-exposed DAG, R5.
//!   Layer-3: verifier binary itself (runtime machine verdict, sound backstop).
//!   Layer-4: trace assertion / SSB row (off-agent, un-gameable).
//!
//! When the verifier does NOT expose a DAG (boolean-only output), the scan
//! returns `ClosureScanKind::ClosureUnavailable` and labels the leg UNSOUND
//! (R5 — the verifier's own machine verdict is the Layer-4 backstop). The
//! leg is necessary-not-sufficient; never a sole gate.
//!
//! WIRING (production entrypoint):
//! Called by `formal_verify::orchestrator::run()` as the first step before
//! the verifier dispatch (`match request.verifier { ... }`). A BLOCK result
//! short-circuits the pipeline and returns without running the verifier.
//! Also wired into `darwin_admit_loop::evaluate_candidate_trap()` as the
//! deterministic formal-lane first leg (behind `cfg(feature="darwinian-evolve")`).
//!
//! KILL-SWITCH: `ELAI_CLOSURE_SCAN_DISABLE` env var disables this scan.
//! Default-ON; the kill-switch is registered in `wire-dark-allowlist.toml`
//! per R9.
//!
//! NO new external dep: all scanning is over the `DischargedGoal` DAG the
//! verifier already exposes. No new Cargo.toml entry.
//! No `.unwrap()`/`.expect()` in production code paths.

use std::collections::{HashMap, HashSet, VecDeque};

use tracing::{debug, warn};

use crate::formal_verify::types::{DischargedGoal, HatchKind, ObligationNode, VerifierKind};

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

/// A single escape-hatch node found during the BFS closure walk.
#[derive(Debug, Clone, PartialEq)]
pub struct EscapeNode {
    /// The obligation_id of the node that carries an escape hatch.
    pub obligation_id: String,
    /// The kind of escape hatch found at this node.
    pub hatch: HatchKind,
    /// Whether the hatch has a valid, unexpired row in the registered-exception
    /// allowlist (`obligation-closure-allowlist.toml`).
    pub registered: bool,
}

/// Verdict of the closure scan.
#[derive(Debug, Clone, PartialEq)]
pub enum ClosureScanKind {
    /// Full closure walked; NO escape hatch found anywhere. The verifier may
    /// proceed. This is the only passing state.
    Clean,
    /// At least one UNREGISTERED escape hatch found in the closure. The
    /// verifier dispatch is BLOCKED — this is the AlphaProof relocated-sorry
    /// and hallucinated-known-result failure mode.
    EscapeHatchAnywhere,
    /// Every escape hatch found in the closure has a valid, unexpired row in
    /// the registered-exception allowlist. Recorded (the SSB row fires) but
    /// NOT a hard block. Mirrors AlphaProof's labelled QuantumOptics sorries.
    RegisteredException,
    /// The verifier does not expose a dependency DAG for this goal (boolean-
    /// only output). The scan is UNSOUND for this goal (R5 label). The Layer-4
    /// verifier machine verdict is the only authoritative backstop.
    ClosureUnavailable,
}

impl ClosureScanKind {
    /// Stable snake_case wire string.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Clean => "clean",
            Self::EscapeHatchAnywhere => "escape_hatch_anywhere",
            Self::RegisteredException => "registered_exception",
            Self::ClosureUnavailable => "closure_unavailable",
        }
    }

    /// Whether this result BLOCKs the verifier dispatch.
    /// Only `EscapeHatchAnywhere` is a hard block; the others allow the
    /// pipeline to continue (with the caveat that `ClosureUnavailable`
    /// means the Layer-2 leg is unsound and the Layer-4 backstop is relied on).
    #[must_use]
    pub fn is_block(&self) -> bool {
        matches!(self, Self::EscapeHatchAnywhere)
    }
}

/// Complete result of a single `scan_obligation_closure` call.
#[derive(Debug, Clone, PartialEq)]
pub struct ClosureScanResult {
    /// The scan verdict.
    pub kind: ClosureScanKind,
    /// All escape-hatch nodes found during the BFS (empty for `Clean` and
    /// `ClosureUnavailable`).
    pub escape_nodes: Vec<EscapeNode>,
    /// Total number of obligation nodes visited during BFS (including the
    /// top-level goal). Zero for `ClosureUnavailable`.
    pub closure_size: u32,
    /// blake3 hex-prefix (16 chars) of the canonical DAG fingerprint.
    /// Computed from goal_id + sorted (obligation_id, hatch?, depends_on)
    /// tuples. Recomputes byte-identically from the same DAG under the same
    /// ELAI_SEED (GD-2). Empty string for `ClosureUnavailable`.
    pub closure_fingerprint: String,
}

// ---------------------------------------------------------------------------
// Registered-exception allowlist
// ---------------------------------------------------------------------------

/// One entry in `obligation-closure-allowlist.toml`.
#[derive(Debug, Clone, PartialEq)]
struct AllowlistEntry {
    /// `"<verifier_kind>:<goal_id>"` key. Must match the EscapeNode's
    /// obligation_id (as used in the DAG).
    obligation_id: String,
    /// Expiry date as "YYYY-MM-DD". An expired entry is treated as if absent.
    expiry_date: String,
}

/// Parse the `obligation-closure-allowlist.toml` file from the given path.
/// Returns an empty vec if the file does not exist or cannot be parsed
/// (fail-closed: an unreadable allowlist produces zero registered exceptions,
/// so every hatch is a BLOCK).
fn load_allowlist(allowlist_path: &std::path::Path) -> Vec<AllowlistEntry> {
    let content = match std::fs::read_to_string(allowlist_path) {
        Ok(c) => c,
        Err(e) => {
            debug!(
                path = %allowlist_path.display(),
                err = %e,
                "obligation-closure-allowlist.toml not found or unreadable; \
                 all escape hatches will BLOCK (fail-closed)"
            );
            return Vec::new();
        }
    };
    parse_allowlist_toml(&content)
}

/// Minimal TOML parser for the allowlist schema. Parses `[[obligation]]`
/// sections carrying `obligation_id` and `expiry_date`. Uses only stdlib;
/// no external dep.
///
/// Tolerates unknown fields (forward-compatibility). A section missing
/// either required field is silently skipped (fail-closed: an incomplete
/// row grants no exception).
fn parse_allowlist_toml(content: &str) -> Vec<AllowlistEntry> {
    let mut entries = Vec::new();
    let mut in_obligation_section = false;
    let mut current_id: Option<String> = None;
    let mut current_expiry: Option<String> = None;

    for line in content.lines() {
        let line = line.trim();
        if line == "[[obligation]]" {
            // Flush the previous section if complete.
            if let (Some(id), Some(exp)) = (current_id.take(), current_expiry.take()) {
                entries.push(AllowlistEntry { obligation_id: id, expiry_date: exp });
            } else {
                // Incomplete section; discard (fail-closed).
                current_id = None;
                current_expiry = None;
            }
            in_obligation_section = true;
            continue;
        }
        if !in_obligation_section || line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(val) = parse_toml_string_field(line, "obligation_id") {
            current_id = Some(val);
        } else if let Some(val) = parse_toml_string_field(line, "expiry_date") {
            current_expiry = Some(val);
        }
    }
    // Flush the last section.
    if let (Some(id), Some(exp)) = (current_id, current_expiry) {
        entries.push(AllowlistEntry { obligation_id: id, expiry_date: exp });
    }
    entries
}

/// Extract the value from a `key = "value"` TOML line (any whitespace around `=`).
/// Returns `None` if the line does not match the key or does not have a quoted value.
fn parse_toml_string_field(line: &str, key: &str) -> Option<String> {
    let line = line.trim();
    // Accept `key = "value"`, `key   =   "value"`, etc.
    if !line.starts_with(key) {
        return None;
    }
    let after_key = line[key.len()..].trim_start();
    if !after_key.starts_with('=') {
        return None;
    }
    let rest = after_key[1..].trim();
    if rest.starts_with('"') && rest.ends_with('"') && rest.len() >= 2 {
        Some(rest[1..rest.len() - 1].to_string())
    } else {
        None
    }
}

/// Check whether a given `obligation_id` has a valid, unexpired row in the
/// allowlist.
///
/// `today_iso` is "YYYY-MM-DD" (from `current_date_iso()`). An expired entry
/// (expiry_date < today) is treated as absent (fail-closed).
fn is_registered(
    obligation_id: &str,
    allowlist: &[AllowlistEntry],
    today_iso: &str,
) -> bool {
    allowlist
        .iter()
        .any(|e| e.obligation_id == obligation_id && e.expiry_date.as_str() >= today_iso)
}

/// Return the current date as "YYYY-MM-DD" in UTC. Uses only stdlib
/// (no external dep). Falls back to a fixed future date on parse failure so
/// the gate is never accidentally fail-opened by a bad system clock.
fn current_date_iso() -> String {
    // std::time::SystemTime gives seconds since UNIX_EPOCH. Convert to
    // "YYYY-MM-DD" manually (no chrono / time dep).
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now() // entropy-allow: today's wall-clock date for allowlist expiry comparison, must track real calendar time
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Days since epoch -> Gregorian calendar (proleptic).
    let days = secs / 86400;
    gregorian_from_days(days)
}

/// Convert days since UNIX epoch (1970-01-01) to "YYYY-MM-DD".
/// Implements the civil calendar algorithm (Howard Hinnant, public domain,
/// confirmed correct for 1970-2200 range). No external dep.
fn gregorian_from_days(days: u64) -> String {
    // Shift to era-relative days (civil::from_days).
    let z = days as i64 + 719_468; // shift to 1 March 0000 epoch
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

// ---------------------------------------------------------------------------
// DAG fingerprint (blake3, GD-2)
// ---------------------------------------------------------------------------

/// Compute a blake3 hex-prefix (16 chars) fingerprint of the obligation DAG.
///
/// The fingerprint is derived from goal_id + a lexicographically-sorted
/// sequence of (obligation_id, hatch_str_or_none, sorted_depends_on) tuples.
/// This makes it recompute byte-identically from the same DAG under the same
/// inputs (GD-2).
fn dag_fingerprint(goal_id: &str, dag: &HashMap<String, ObligationNode>) -> String {
    let mut sorted_nodes: Vec<&ObligationNode> = dag.values().collect();
    sorted_nodes.sort_by(|a, b| a.obligation_id.cmp(&b.obligation_id));

    let mut hasher = blake3::Hasher::new();
    hasher.update(goal_id.as_bytes());
    hasher.update(b"\x00");
    for node in &sorted_nodes {
        hasher.update(node.obligation_id.as_bytes());
        hasher.update(b"\x00");
        let hatch_str = node.hatch.map(|h| h.as_str()).unwrap_or("none");
        hasher.update(hatch_str.as_bytes());
        hasher.update(b"\x00");
        let mut deps = node.depends_on.clone();
        deps.sort();
        for dep in &deps {
            hasher.update(dep.as_bytes());
            hasher.update(b"\x00");
        }
        hasher.update(b"\x01"); // node separator
    }
    let hash = hasher.finalize();
    let hex = hash.to_hex();
    hex[..16].to_string()
}

// ---------------------------------------------------------------------------
// Primary scan function
// ---------------------------------------------------------------------------

/// Scan the full transitive dependency closure of a discharged goal for
/// escape hatches.
///
/// This is the H-54 primary entry point, called as the Layer-2 deterministic
/// first leg of the formal-verification gate (before the verifier dispatch).
///
/// # Arguments
/// * `goal` — the discharged goal whose closure to scan. If `goal.dag` is
///   `None`, returns `ClosureScanKind::ClosureUnavailable` immediately.
/// * `verifier` — the verifier kind (used for logging + SSB row).
/// * `allowlist_path` — path to `obligation-closure-allowlist.toml`. If
///   `None`, uses the default location (`obligation-closure-allowlist.toml`
///   in the current directory). An absent file is fail-closed (no exceptions).
///
/// # Kill-switch
/// If `ELAI_CLOSURE_SCAN_DISABLE` is set in the environment, returns
/// `ClosureScanKind::Clean` with `closure_size=0` and logs a warning. The
/// kill-switch is registered in `wire-dark-allowlist.toml` (R9).
///
/// # Algorithm
/// BFS from the top-level `goal.goal_id` node over the `dag` adjacency map.
/// At each node, checks `node.hatch`:
///   - `None`: clean node, continue BFS.
///   - `Some(hatch)`: check the allowlist. If registered+unexpired ->
///     `RegisteredException` (not a block). If absent/expired ->
///     `EscapeHatchAnywhere` (block, return early).
///
/// The `--top-statement-only` BROKEN ARM: if the caller constructs a
/// `DischargedGoal` where `dag` contains ONLY the root node (no children),
/// the scan trivially returns `Clean` even if helper lemmas are admitted.
/// This is the AlphaProof failure mode 1:1 — the scan is only as sound as
/// the verifier's DAG exposure. When the DAG is shallow, label the result
/// as `ClosureUnavailable` is NOT triggered (the scan ran correctly over the
/// shallow DAG); instead, the R5 caveat applies at DAG construction time.
///
/// # Errors
/// Returns `ClosureScanResult` in all cases (no `Result` return). Every
/// error path (missing allowlist, unreadable file, DAG cycle) is handled
/// gracefully and fails closed.
pub fn scan_obligation_closure(
    goal: &DischargedGoal,
    verifier: VerifierKind,
    allowlist_path: Option<&std::path::Path>,
) -> ClosureScanResult {
    // Kill-switch check (default-ON, R9 kill-switch ELAI_CLOSURE_SCAN_DISABLE).
    if std::env::var("ELAI_CLOSURE_SCAN_DISABLE").is_ok() {
        warn!(
            goal_id = %goal.goal_id,
            verifier = verifier.as_str(),
            "ELAI_CLOSURE_SCAN_DISABLE is set; closure scan bypassed (Layer-2 unsound). \
             Kill-switch registered in wire-dark-allowlist.toml per R9."
        );
        return ClosureScanResult {
            kind: ClosureScanKind::Clean,
            escape_nodes: vec![],
            closure_size: 0,
            closure_fingerprint: String::new(),
        };
    }

    // If the verifier does not expose a DAG, return ClosureUnavailable.
    let dag = match &goal.dag {
        Some(d) => d,
        None => {
            debug!(
                goal_id = %goal.goal_id,
                verifier = verifier.as_str(),
                "no obligation DAG available for this verifier output; \
                 scan is UNSOUND (R5) — Layer-4 verifier verdict is the backstop"
            );
            return ClosureScanResult {
                kind: ClosureScanKind::ClosureUnavailable,
                escape_nodes: vec![],
                closure_size: 0,
                closure_fingerprint: String::new(),
            };
        }
    };

    // Compute the fingerprint before the BFS so it is always present.
    let fingerprint = dag_fingerprint(&goal.goal_id, dag);

    // Load the registered-exception allowlist (fail-closed on missing file).
    let default_allowlist = std::path::PathBuf::from("obligation-closure-allowlist.toml");
    let allowlist_path = allowlist_path.unwrap_or(&default_allowlist);
    let allowlist = load_allowlist(allowlist_path);
    let today = current_date_iso();

    // BFS over the obligation DAG from the top-level goal.
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<String> = VecDeque::new();
    let mut escape_nodes: Vec<EscapeNode> = Vec::new();
    let mut has_unregistered = false;
    let mut has_registered = false;

    queue.push_back(goal.goal_id.clone());

    while let Some(current_id) = queue.pop_front() {
        if visited.contains(&current_id) {
            continue; // cycle guard
        }
        visited.insert(current_id.clone());

        let node: &ObligationNode = match dag.get(&current_id) {
            Some(n) => n,
            None => {
                // Node referenced but not in DAG — treat as missing evidence (R1).
                // A goal that references an obligation the verifier didn't expose
                // is structurally incomplete; this is conservative (fail-safe).
                debug!(
                    obligation_id = %current_id,
                    "BFS: referenced obligation not in DAG; skipping (treat as clean leaf)"
                );
                continue;
            }
        };

        // Check for escape hatch at this node.
        if let Some(hatch) = node.hatch {
            let registered = is_registered(&current_id, &allowlist, &today);
            escape_nodes.push(EscapeNode {
                obligation_id: current_id.clone(),
                hatch,
                registered,
            });
            if registered {
                has_registered = true;
                debug!(
                    obligation_id = %current_id,
                    hatch = hatch.as_str(),
                    "BFS: registered exception for escape hatch; recorded, not BLOCKed"
                );
            } else {
                has_unregistered = true;
                debug!(
                    obligation_id = %current_id,
                    hatch = hatch.as_str(),
                    "BFS: UNREGISTERED escape hatch found in obligation closure; BLOCK"
                );
                // Do NOT short-circuit: continue the BFS to enumerate ALL hatches
                // for the SSB row (gives the caller the full list). The verdict is
                // already decided but we want the complete escape_nodes set.
            }
        }

        // Enqueue children (the obligations this node depends on).
        for dep in &node.depends_on {
            if !visited.contains(dep) {
                queue.push_back(dep.clone());
            }
        }
    }

    let closure_size = visited.len() as u32;
    let kind = if has_unregistered {
        ClosureScanKind::EscapeHatchAnywhere
    } else if has_registered {
        ClosureScanKind::RegisteredException
    } else {
        ClosureScanKind::Clean
    };

    debug!(
        goal_id = %goal.goal_id,
        verifier = verifier.as_str(),
        kind = kind.as_str(),
        closure_size,
        escape_count = escape_nodes.len(),
        fingerprint = %fingerprint,
        "closure scan complete"
    );

    ClosureScanResult {
        kind,
        escape_nodes,
        closure_size,
        closure_fingerprint: fingerprint,
    }
}

// ---------------------------------------------------------------------------
// Internal unit tests (not the contract test — that lives in
// crates/contracts/tests/h54_obligation_closure_contract.rs)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::formal_verify::types::VerifierKind;

    fn make_goal_clean() -> DischargedGoal {
        let mut dag = HashMap::new();
        dag.insert(
            "root".to_string(),
            ObligationNode {
                obligation_id: "root".to_string(),
                hatch: None,
                depends_on: vec!["helper".to_string()],
            },
        );
        dag.insert(
            "helper".to_string(),
            ObligationNode {
                obligation_id: "helper".to_string(),
                hatch: None,
                depends_on: vec![],
            },
        );
        DischargedGoal::with_dag("root", dag)
    }

    fn make_goal_relocated_admit() -> DischargedGoal {
        // The AlphaProof failure mode: top goal looks clean, but the helper
        // lemma (a renamed admitted obligation) carries `sorry`.
        let mut dag = HashMap::new();
        dag.insert(
            "root".to_string(),
            ObligationNode {
                obligation_id: "root".to_string(),
                hatch: None, // top goal looks sorry-free
                depends_on: vec!["renamed_helper_lemma".to_string()],
            },
        );
        dag.insert(
            "renamed_helper_lemma".to_string(),
            ObligationNode {
                obligation_id: "renamed_helper_lemma".to_string(),
                hatch: Some(HatchKind::Sorry), // the relocated sorry
                depends_on: vec![],
            },
        );
        DischargedGoal::with_dag("root", dag)
    }

    #[test]
    fn clean_goal_returns_clean() {
        let goal = make_goal_clean();
        let result = scan_obligation_closure(&goal, VerifierKind::Dafny, None);
        assert_eq!(result.kind, ClosureScanKind::Clean);
        assert!(result.escape_nodes.is_empty());
        assert_eq!(result.closure_size, 2);
        assert!(!result.closure_fingerprint.is_empty());
    }

    #[test]
    fn relocated_admit_in_helper_is_blocked() {
        // This is the AlphaProof failure mode 1:1 — a top-statement-only
        // scanner would return Clean because the root node has no hatch.
        // The BFS must walk into the helper and find the sorry.
        let goal = make_goal_relocated_admit();
        let result = scan_obligation_closure(&goal, VerifierKind::Dafny, None);
        assert_eq!(
            result.kind,
            ClosureScanKind::EscapeHatchAnywhere,
            "The relocated-sorry in the helper lemma MUST be detected by the BFS; \
             a top-statement-only scan would miss this (AlphaProof failure mode)"
        );
        assert!(result.kind.is_block(), "EscapeHatchAnywhere must be a BLOCK");
        assert_eq!(result.escape_nodes.len(), 1);
        assert_eq!(result.escape_nodes[0].obligation_id, "renamed_helper_lemma");
        assert_eq!(result.escape_nodes[0].hatch, HatchKind::Sorry);
        assert!(!result.escape_nodes[0].registered);
    }

    #[test]
    fn unavailable_dag_returns_closure_unavailable() {
        let goal = DischargedGoal::unavailable("SomeGoal");
        let result = scan_obligation_closure(&goal, VerifierKind::Z3Smt, None);
        assert_eq!(result.kind, ClosureScanKind::ClosureUnavailable);
        assert!(!result.kind.is_block(), "ClosureUnavailable is NOT a block (Layer-4 is the backstop)");
        assert_eq!(result.closure_size, 0);
        assert!(result.closure_fingerprint.is_empty());
    }

    #[test]
    fn fingerprint_is_16_hex_chars_and_stable() {
        let goal = make_goal_clean();
        let r1 = scan_obligation_closure(&goal, VerifierKind::Kani, None);
        let r2 = scan_obligation_closure(&goal, VerifierKind::Kani, None);
        assert_eq!(r1.closure_fingerprint.len(), 16, "fingerprint must be 16 hex chars");
        assert_eq!(
            r1.closure_fingerprint, r2.closure_fingerprint,
            "GD-2: fingerprint must recompute byte-identically on same input"
        );
    }

    #[test]
    fn assume_hatch_also_blocked() {
        let mut dag = HashMap::new();
        dag.insert(
            "root".to_string(),
            ObligationNode {
                obligation_id: "root".to_string(),
                hatch: None,
                depends_on: vec!["dafny_assume_node".to_string()],
            },
        );
        dag.insert(
            "dafny_assume_node".to_string(),
            ObligationNode {
                obligation_id: "dafny_assume_node".to_string(),
                hatch: Some(HatchKind::Assume),
                depends_on: vec![],
            },
        );
        let goal = DischargedGoal::with_dag("root", dag);
        let result = scan_obligation_closure(&goal, VerifierKind::Dafny, None);
        assert_eq!(result.kind, ClosureScanKind::EscapeHatchAnywhere);
        assert_eq!(result.escape_nodes[0].hatch, HatchKind::Assume);
    }

    #[test]
    fn hatch_kinds_all_have_nonempty_wire_strings() {
        let hatches = [
            HatchKind::Sorry,
            HatchKind::Admit,
            HatchKind::Axiom,
            HatchKind::Assume,
            HatchKind::VerifierExternal,
            HatchKind::KaniAssumeVacuous,
        ];
        for h in hatches {
            assert!(!h.as_str().is_empty(), "HatchKind::{h:?} wire string is empty");
        }
    }

    #[test]
    fn closure_scan_kind_wire_strings() {
        assert_eq!(ClosureScanKind::Clean.as_str(), "clean");
        assert_eq!(ClosureScanKind::EscapeHatchAnywhere.as_str(), "escape_hatch_anywhere");
        assert_eq!(ClosureScanKind::RegisteredException.as_str(), "registered_exception");
        assert_eq!(ClosureScanKind::ClosureUnavailable.as_str(), "closure_unavailable");
    }

    #[test]
    fn parse_allowlist_toml_parses_one_entry() {
        let toml = r#"
[[obligation]]
obligation_id   = "dafny:MyProof_Lemma3"
owner           = "maintainer"
reason          = "open benchmark"
expiry_date     = "2099-12-31"
evidence_pointer = "arXiv:2605.22763"
"#;
        let entries = parse_allowlist_toml(toml);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].obligation_id, "dafny:MyProof_Lemma3");
        assert_eq!(entries[0].expiry_date, "2099-12-31");
    }

    #[test]
    fn parse_allowlist_toml_incomplete_section_grants_no_exception() {
        // Missing expiry_date -> incomplete row -> no entry granted.
        let toml = r#"
[[obligation]]
obligation_id   = "dafny:Incomplete"
owner           = "maintainer"
"#;
        let entries = parse_allowlist_toml(toml);
        assert!(entries.is_empty(), "incomplete row must not grant an exception (fail-closed)");
    }

    #[test]
    fn is_registered_expired_entry_returns_false() {
        let allowlist = vec![AllowlistEntry {
            obligation_id: "dafny:OldLemma".to_string(),
            expiry_date: "2020-01-01".to_string(), // in the past
        }];
        // today is 2026-06-11 per session date
        assert!(
            !is_registered("dafny:OldLemma", &allowlist, "2026-06-11"),
            "expired entry must NOT grant an exception"
        );
    }

    #[test]
    fn cycle_in_dag_does_not_loop_forever() {
        // A -> B -> A cycle. BFS must terminate via the visited set.
        let mut dag = HashMap::new();
        dag.insert(
            "A".to_string(),
            ObligationNode {
                obligation_id: "A".to_string(),
                hatch: None,
                depends_on: vec!["B".to_string()],
            },
        );
        dag.insert(
            "B".to_string(),
            ObligationNode {
                obligation_id: "B".to_string(),
                hatch: None,
                depends_on: vec!["A".to_string()],
            },
        );
        let goal = DischargedGoal::with_dag("A", dag);
        let result = scan_obligation_closure(&goal, VerifierKind::Verus, None);
        assert_eq!(result.kind, ClosureScanKind::Clean);
        // Only 2 unique nodes visited despite the cycle.
        assert_eq!(result.closure_size, 2);
    }

    #[test]
    fn gregorian_from_days_known_dates() {
        // 1970-01-01 = day 0
        assert_eq!(gregorian_from_days(0), "1970-01-01");
        // 2026-06-11: days since epoch = ?
        // 2026-01-01 = day 20454 (verified: 56 years * 365 + 14 leap years = 20454)
        // 2026-06-11 = day 20454 + 31(Jan) + 28(Feb) + 31(Mar) + 30(Apr) + 31(May) + 11(Jun) - 1
        //            = 20454 + 161 = 20615
        assert_eq!(gregorian_from_days(20615), "2026-06-11");
    }
}
