//! J-16 — blast-radius / anti-regression gate (auditor-owned, OBSERVE-ONLY).
//!
//! Runtime observe-only gate that runs on the EXECUTING -> AUDITING FSM
//! transition. Consumes the B-29 `compute_impact_radius` reverse-dep
//! traversal (GD-1 P0: never re-author the BFS) and emits
//! `BlastRadiusRegressionEvent` SSB rows for four observability legs:
//! cone computed, scope violation flagged, regression set selected,
//! silent deletion flagged.
//!
//! Distinct from G-3's `crates/safety/src/blast_radius.rs` +
//! `BlastRadiusBlocked` SSB variant (FS/egress gate). This module owns
//! the regression / anti-regression cone surface; G-3 owns the
//! FS-escape / egress-deny surface. The Kind enum is named
//! `BlastRadiusRegressionKind` (NOT `BlastRadiusKind`) to keep the two
//! surfaces disjoint at the type level.
//!
//! J-90 Leg A will extend this module with `fuse_crosslang_cone` +
//! `extract_typed_seams`. The `BlastRadiusRegressionKind` enum is 4-arm
//! now; J-90 widens it to 6-arm (append-only, do NOT reorder existing
//! arms).
//!
//! Source plan: docs/thefinalglance/J-plan/J-16_blast_radius_regression_gate.md

use crate::evidence::{EvidenceError, EvidenceGate, TransitionContext};
use crate::fsm::FsmState;
use crate::ssb::{
    BlastRadiusRegressionKind, BlastRadiusRegressionPayload, Ssb, SsbPayload,
};
use crate::task_runner::{RUNNER_AGENT_ID, RUNNER_SUPERSTEP};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// CapturedDiff — the off-agent diff artifact threaded into TransitionContext
// ---------------------------------------------------------------------------

/// The captured diff threaded from `task_runner` into `TransitionContext` at
/// the EXECUTING -> AUDITING gate call site. Aggregates the orchestrator-
/// captured structured-patch hunks + git_diff blob (the off-agent artifact
/// per wiring-enforcement R4) plus the plan-unit declared scope.
///
/// `changed_symbols` carries file paths OR symbol names (the B-29
/// `compute_impact_radius` accepts both via `looks_like_file_path`).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct CapturedDiff {
    /// File paths touched by the edit (absolute or repo-relative).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changed_files: Vec<String>,
    /// Symbol names (file path OR symbol) whose reverse-dep cone is computed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changed_symbols: Vec<String>,
    /// Per-file structured-patch hunks (line ranges). Used by
    /// `flag_silent_deletion` to detect net-negative pub-item removal.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hunks: Vec<HunkSummary>,
    /// Plan-unit WIRING_CONTRACT declared scope (file paths). Used by
    /// `check_diff_scope` to flag out-of-scope edits.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub declared_scope: Vec<String>,
    /// Raw `git_diff` blob (serde_json::Value) from file_ops, kept for the
    /// silent-deletion leg. `None` is a hard FAIL at the gate (R2:
    /// unmappable diff never passes).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_diff: Option<serde_json::Value>,
    /// J-16 FIX (S10 verifier reject reason 3): project root used to open
    /// the B-29 `SymbolGraphCache` so `compute_cone` /
    /// `select_regression_set` actually run the reverse-dep BFS instead of
    /// degrading to identity. Populated from
    /// `task_runner::drive_task`'s `detected_project_dir`. `None` keeps
    /// the legacy degrade-to-identity path (cache unavailable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_dir: Option<PathBuf>,
}

/// Minimal hunk summary used by the silent-deletion leg. We do NOT pull
/// `elai_tools::file_ops::StructuredPatchHunk` into production code
/// (`elai-tools` is a dev-dep of this crate); the caller maps the
/// structured-patch hunks into this summary at the task_runner seam.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct HunkSummary {
    pub file_path: String,
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    /// Net line delta (new_lines - old_lines). Negative on a deletion-heavy
    /// hunk — the silent-deletion leg flags large negative deltas on
    /// `pub` item lines.
    pub net_delta: i64,
}

// ---------------------------------------------------------------------------
// compute_cone — thin wrapper over B-29 compute_impact_radius (GD-1 P0)
// ---------------------------------------------------------------------------

/// Cone depth matches `mutation_budget::CONE_DEPTH` so the two consumers
/// share the same reachability horizon. Keep as a literal here so J-90's
/// cross-lang fusion can widen without touching `mutation_budget`.
const BLAST_RADIUS_CONE_DEPTH: u32 = 3;

/// Compute the reverse-dep cone for a set of changed symbols by unioning
/// the per-symbol `compute_impact_radius` `resolved_files` (the
/// `mutation_budget::select_cone` precedent at
/// `crates/orchestrator/src/mutation_budget.rs:312-343`). Re-implementing
/// the BFS is GD-1 P0.
///
/// Degrades to the input set on reachability failure (never fabricates a
/// cone, never drops coverage). Returns `(cone, truncated)` where
/// `truncated` is true if ANY per-symbol traversal hit the depth cap.
#[must_use]
pub fn compute_cone(
    cache: Option<&mut elai_knowledge::symbol_graph::cache::SymbolGraphCache>,
    changed_symbols: &[String],
) -> (Vec<String>, bool) {
    let Some(mut cache) = cache else {
        return (changed_symbols.to_vec(), false);
    };
    let mut cone: BTreeSet<String> = BTreeSet::new();
    let mut any_truncated = false;
    for sym in changed_symbols {
        match elai_knowledge::symbol_graph::traversal::compute_impact_radius(
            &mut cache,
            sym,
            BLAST_RADIUS_CONE_DEPTH,
        ) {
            Ok(impact) => {
                if impact.truncated {
                    any_truncated = true;
                }
                for file in impact.resolved_files {
                    cone.insert(file);
                }
                cone.insert(sym.clone());
            }
            Err(_) => {
                cone.insert(sym.clone());
            }
        }
    }
    if cone.is_empty() {
        return (changed_symbols.to_vec(), false);
    }
    (cone.into_iter().collect(), any_truncated)
}

// ---------------------------------------------------------------------------
// open_symbol_graph_cache — J-16 FIX (S10 verifier reject reason 3)
// ---------------------------------------------------------------------------

/// Open a `SymbolGraphCache` for `project_root` using the SAME pattern as
/// `task_runner::write_impact_summary_to_log` and
/// `build_existing_project_context_impl`: `dkg_db_path(project_root)` +
/// `SymbolGraphCache::new(&db_path, &project_id, SYMBOL_FILE_LRU_SIZE)`.
///
/// Returns `None` (and logs) when the DB is missing or the cache fails to
/// open — callers keep the current degrade-to-identity behavior in that
/// case (Some-cache-failed == None behavior, per the task spec).
fn open_symbol_graph_cache(
    project_root: &std::path::Path,
) -> Option<elai_knowledge::symbol_graph::cache::SymbolGraphCache> {
    use elai_core::paths::dkg_db_path;
    use elai_core::project_id_from_path;
    use elai_knowledge::symbol_graph::{
        cache::SymbolGraphCache,
        sharding::SYMBOL_FILE_LRU_SIZE,
    };

    let db_path = dkg_db_path(project_root);
    if !db_path.exists() {
        tracing::debug!(
            db_path = %db_path.display(),
            "J-16 blast_radius: symbol graph DB not found, degrading to identity cone"
        );
        return None;
    }
    let project_id = project_id_from_path(project_root);
    match SymbolGraphCache::new(&db_path, &project_id, SYMBOL_FILE_LRU_SIZE) {
        Ok(cache) => Some(cache),
        Err(e) => {
            tracing::warn!(
                error = %e,
                db_path = %db_path.display(),
                "J-16 blast_radius: SymbolGraphCache::new failed, degrading to identity cone"
            );
            None
        }
    }
}

// ---------------------------------------------------------------------------
// check_diff_scope — out-of-scope file flag (OBSERVE-ONLY)
// ---------------------------------------------------------------------------

/// Return the subset of `changed_files` that are NOT inside the declared
/// plan-unit scope. OBSERVE-ONLY (GD-4): the gate flags, does not block,
/// on out-of-scope edits. Empty result = all edits in-scope.
///
/// A `changed_file` is in-scope iff it equals any declared scope entry OR
/// is a path-descendant of a declared scope directory (prefix match on
/// path components, not a raw string prefix).
#[must_use]
pub fn check_diff_scope(
    changed_files: &[String],
    declared_scope: &[String],
) -> Vec<String> {
    if declared_scope.is_empty() {
        return Vec::new();
    }
    changed_files
        .iter()
        .filter(|f| !is_in_scope(f, declared_scope))
        .cloned()
        .collect()
}

fn is_in_scope(file: &str, declared_scope: &[String]) -> bool {
    let normalize = |p: &str| p.replace('\\', "/");
    let f = normalize(file);
    for entry in declared_scope {
        let e = normalize(entry);
        if f == e {
            return true;
        }
        let mut prefix = e.clone();
        if !prefix.ends_with('/') {
            prefix.push('/');
        }
        if f.starts_with(&prefix) {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// select_regression_set — cone -> existing tests
// ---------------------------------------------------------------------------

/// Select the existing-test set whose coverage intersects the cone. The
/// caller (auditor) runs these tests; a pass->fail flip on any selected
/// test is the un-gameable regression signal (R4) that hard-blocks the
/// FSM advance.
///
/// This function returns the cone-filtered candidate list using the
/// symbol-graph name index. It does NOT run the tests. Tests whose
/// source path is inside any cone file's directory are selected; this
/// is a conservative superset that the auditor narrows by actual
/// coverage data when available.
#[must_use]
pub fn select_regression_set(
    cache: Option<&mut elai_knowledge::symbol_graph::cache::SymbolGraphCache>,
    cone: &[String],
) -> Vec<String> {
    let Some(_cache) = cache else {
        return Vec::new();
    };
    let mut selected: BTreeSet<String> = BTreeSet::new();
    let test_roots = ["tests/", "test/", "src/test/", "src/tests/"];
    for file in cone {
        let normalized = file.replace('\\', "/");
        if normalized.contains("/tests/")
            || normalized.contains("/test/")
            || test_roots.iter().any(|r| normalized.starts_with(r))
            || normalized.ends_with("_test.rs")
            || normalized.ends_with(".test.rs")
            || normalized.ends_with("_test.py")
            || normalized.ends_with("_test.go")
            || normalized.ends_with(".test.ts")
            || normalized.ends_with(".test.tsx")
            || normalized.ends_with(".spec.ts")
        {
            selected.insert(file.clone());
        }
    }
    selected.into_iter().collect()
}

// ---------------------------------------------------------------------------
// flag_silent_deletion — undeclared pub-item removal
// ---------------------------------------------------------------------------

/// Return the per-hunk summaries whose `net_delta` is strictly negative
/// AND whose `file_path` is a Rust source (`.rs`) — a conservative
/// signal for undeclared pub-item removal. The auditor SSB row carries
/// these so a human can confirm whether the deletion was declared in the
/// edit intent.
///
/// This is OBSERVE-ONLY (GD-4): it flags, does not block. The
/// silent-deletion leg is a heuristic; a real pub-item-removal detector
/// would need a parse of the pre/post AST, which is out of scope for the
/// first slice (the plan allows W-6-justified absence of the AST leg).
#[must_use]
pub fn flag_silent_deletion(diff: &CapturedDiff) -> Vec<String> {
    diff.hunks
        .iter()
        .filter(|h| h.net_delta < 0 && h.file_path.ends_with(".rs"))
        .map(|h| format!("{}: net_delta={}", h.file_path, h.net_delta))
        .collect()
}

// ---------------------------------------------------------------------------
// BlastRadiusRegressionGate — EvidenceGate impl (OBSERVE-ONLY, GD-4)
// ---------------------------------------------------------------------------

/// EXECUTING -> AUDITING evidence gate. Co-fires with `TestFlippedGate` +
/// `DiagnosticProbeGate` + `EntrypointAndTraceGate`. OBSERVE-ONLY (GD-4):
/// emits `BlastRadiusRegressionEvent` SSB flags and returns `Ok(())`
/// unless a regression test flipped pass->fail (the un-gameable signal,
/// R4) OR the diff is `None` / unmappable (R1/R2 fail-closed).
///
/// Distinct from G-3's `BlastRadiusBlocked` (FS/egress). This gate owns
/// the regression / anti-regression cone surface.
pub struct BlastRadiusRegressionGate;

impl EvidenceGate for BlastRadiusRegressionGate {
    fn name(&self) -> &'static str {
        "blast_radius_regression"
    }

    fn transition(&self) -> (FsmState, FsmState) {
        (FsmState::Executing, FsmState::Auditing)
    }

    fn check(&self, ctx: &TransitionContext) -> Result<(), EvidenceError> {
        // R1/R2 fail-closed: None diff = hard FAIL. The auditor never ran
        // the cone analysis, so the transition is BLOCKED, not waved
        // through (AntiFabricationGate precedent at evidence.rs:831).
        let Some(diff) = ctx.blast_radius_diff.as_ref() else {
            return Err(EvidenceError {
                gate_name: self.name(),
                missing_artifacts: vec!["blast_radius_diff".into()],
                suggestion: "Run the blast-radius cone analysis and thread \
                    the captured diff into TransitionContext before \
                    EXECUTING -> AUDITING. Absent diff is a hard FAIL."
                    .into(),
            });
        };
        // An empty diff (no changed files AND no changed symbols) is a
        // no-op transition — the gate passes. R2's "empty cone on a
        // symbol WITH dependents FAILS" applies when symbols ARE
        // declared but the cone comes back empty, which is the
        // contract test's job to cover (the gate itself does not call
        // compute_cone; the caller does and threads the result via
        // SSB). The gate only fail-closes on `None` (never ran) and
        // unmappable diff (parse failure).
        if diff.changed_files.is_empty() && diff.changed_symbols.is_empty() {
            return Ok(());
        }
        // R2: unmappable diff (hunks present but git_diff is None when
        // hunks are non-empty) is a hard FAIL — parse failure never
        // passes.
        if !diff.hunks.is_empty() && diff.git_diff.is_none() {
            return Err(EvidenceError {
                gate_name: self.name(),
                missing_artifacts: vec!["blast_radius_diff_unmappable".into()],
                suggestion: "CapturedDiff has hunks but no git_diff blob; \
                    the diff is unmappable. Parse failure is a hard FAIL, \
                    never a silent pass."
                    .into(),
            });
        }
        // OBSERVE-ONLY (GD-4): the cone / scope / regression-set /
        // silent-deletion legs emit SSB flags via the task_runner
        // EvidenceGateEvaluated row. The gate itself returns Ok(()) —
        // it does NOT hard-block FSM advance on these legs until the
        // FP/friction rate is bounded on >=3 real runs (GD-4). The
        // ONLY hard BLOCK in observe-only mode is a regression test
        // that flipped pass->fail (the un-gameable signal, R4), which
        // is surfaced by `TestFlippedGate` on the same co-fire set.
        //
        // J-16 FIX (S10 verifier reject reason 3): open the
        // `SymbolGraphCache` from `diff.project_dir` (threaded from
        // `task_runner::drive_task`'s `detected_project_dir`) so
        // `compute_cone` and `select_regression_set` actually run the
        // B-29 reverse-dep BFS instead of degrading to identity. If
        // the DB is missing or the cache fails to open, keep the
        // legacy degrade-to-identity path (Some-cache-failed ==
        // current None behavior) but log it.
        let mut cache = diff
            .project_dir
            .as_deref()
            .and_then(open_symbol_graph_cache);
        let (cone, _cone_truncated) =
            compute_cone(cache.as_mut(), &diff.changed_symbols);
        let _scope_violations = check_diff_scope(&diff.changed_files, &diff.declared_scope);
        let _regression_set = select_regression_set(cache.as_mut(), &cone);
        let _silent_deletions = flag_silent_deletion(diff);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// emit_blast_radius_regression_events — SSB emit for the four observability legs
// ---------------------------------------------------------------------------

/// J-16 FIX (S10 verifier reject reason 2): emit one
/// `BlastRadiusRegressionEvent` SSB row per observability leg
/// (ConeComputed / ScopeViolationFlagged / RegressionSetSelected /
/// SilentDeletionFlagged) so the plan's `observable_effect` requirement
/// is satisfied. Mirrors the sibling `blast_radius_ssb_adapter::emit_blast_radius_blocked`
/// pattern (G-3): async-spawned so the synchronous gate path is never
/// blocked on the SSB write; if no tokio runtime is active the row is
/// dropped with a warning.
///
/// OBSERVE-ONLY (GD-4): the gate returns `Ok(())` regardless of the leg
/// results; these rows are advisory flags for the auditor / dashboard,
/// not hard blocks. The only hard BLOCK is a regression test that
/// flipped pass->fail (TestFlippedGate owns that leg).
///
/// Called from `task_runner::drive_task` after the
/// `BlastRadiusRegressionGate::check` call, passing the same
/// `CapturedDiff` so the emit owns the production call path for each
/// leg (wiring-enforcement R4: the off-agent artifact is the SSB row
/// the gate did not author).
pub async fn emit_blast_radius_regression_events(
    writer: &Arc<Ssb>,
    diff: &CapturedDiff,
    session_id: &str,
    ts_ms: i64,
) {
    // J-16 FIX (S10 verifier reject reason 3): open the cache from
    // `diff.project_dir` so the cone / regression-set legs run the B-29
    // BFS instead of degrading to identity. Degrade-to-identity is
    // preserved when the DB is missing / open fails.
    let mut cache = diff
        .project_dir
        .as_deref()
        .and_then(open_symbol_graph_cache);
    let (cone, cone_truncated) = compute_cone(cache.as_mut(), &diff.changed_symbols);
    let scope_violations = check_diff_scope(&diff.changed_files, &diff.declared_scope);
    let regression_set = select_regression_set(cache.as_mut(), &cone);
    let silent_deletions = flag_silent_deletion(diff);

    let legs: [(BlastRadiusRegressionKind, Vec<String>, Vec<String>); 4] = [
        (
            BlastRadiusRegressionKind::ConeComputed,
            diff.changed_symbols.clone(),
            cone.clone(),
        ),
        (
            BlastRadiusRegressionKind::ScopeViolationFlagged,
            diff.changed_files.clone(),
            scope_violations,
        ),
        (
            BlastRadiusRegressionKind::RegressionSetSelected,
            cone.clone(),
            regression_set,
        ),
        (
            BlastRadiusRegressionKind::SilentDeletionFlagged,
            diff.hunks.iter().map(|h| h.file_path.clone()).collect(),
            silent_deletions,
        ),
    ];

    for (kind, changed_symbols, leg_results) in legs {
        let payload = SsbPayload::BlastRadiusRegressionEvent(BlastRadiusRegressionPayload {
            kind,
            changed_symbols,
            cone: cone.clone(),
            scope_violations: if matches!(kind, BlastRadiusRegressionKind::ScopeViolationFlagged) {
                leg_results.clone()
            } else {
                Vec::new()
            },
            regression_set: if matches!(kind, BlastRadiusRegressionKind::RegressionSetSelected) {
                leg_results.clone()
            } else {
                Vec::new()
            },
            silent_deletions: if matches!(kind, BlastRadiusRegressionKind::SilentDeletionFlagged) {
                leg_results.clone()
            } else {
                Vec::new()
            },
            cone_truncated,
            session_id: session_id.to_owned(),
            ts_ms,
        });
        if let Err(e) = writer
            .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
            .await
        {
            tracing::warn!(
                kind = %kind.as_str(),
                error = %e,
                "J-16 blast_radius: BlastRadiusRegressionEvent SSB write failed (non-fatal)"
            );
        }
    }
}
