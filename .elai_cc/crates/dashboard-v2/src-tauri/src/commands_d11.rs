//! D-11 — constructive-checker registry Tauri command surface.
//!
//! Two read-only commands feed the Svelte `CheckerRegistryPanel`:
//!
//! - [`get_checker_results_recent`] — last N `checker_result_emitted`
//!   SSB rows + per-result-kind aggregate (pass / fail / inapplicable /
//!   error).
//! - [`get_checker_registry_state`] — closed list of every
//!   `CheckerKind` variant the orchestrator's registry can host, so the
//!   panel can render the full registry table even before any claim has
//!   been routed.
//!
//! Sibling of `commands_d10` (mode-routing). The orchestrator producer
//! (`elai_orchestrator::dispatch_constructive_checkers`) is the sole
//! writer; this module only reads.
//!
//! See docs/thefinalglance/D-plan/D-11_constructive_checker_registry.md.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 500;

/// Typed mirror of `elai_orchestrator::CheckerResultRow`. Lives in this
/// crate so the Tauri-side schema does not leak the orchestrator's
/// internal type through the IPC boundary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckerResultRowDto {
    pub checker_id: String,
    pub checker_kind: String,
    pub result_kind: String,
    pub claim_id: String,
    pub ts_ms: i64,
    pub reason: String,
}

/// Aggregated response: recent rows + per-result-kind counts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecentCheckerResults {
    pub rows: Vec<CheckerResultRowDto>,
    /// Snake-case result-kind label -> count within `rows`. Sorted-key
    /// `BTreeMap` so the JSON output is deterministic for snapshot tests.
    pub counts_by_kind: BTreeMap<String, u32>,
    /// Total number of rows in `rows` (== `rows.len()`).
    pub total: u32,
}

/// Static description of a known checker kind. The orchestrator's
/// [`elai_orchestrator::CheckerKind`] enum is a closed set — the
/// registry can host at most one entry per kind in the canonical
/// configuration. The panel renders this list as the "registry
/// surface" table; runtime row data comes from
/// [`get_checker_results_recent`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckerKindEntry {
    /// Snake-case label matching [`elai_orchestrator::CheckerKind::as_str`].
    pub kind: String,
    /// Short human-readable description for the panel.
    pub description: String,
    /// True iff a default implementation ships in D-11. False entries
    /// (Z3, JsonSchemaValidate) are deferred slots that always return
    /// Inapplicable until the helper lands.
    pub builtin_available: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckerRegistryState {
    /// Every closed-enum kind in declaration order.
    pub kinds: Vec<CheckerKindEntry>,
}

const KIND_TABLE: &[(&str, &str, bool)] = &[
    (
        "sympy",
        "SymPy symbolic + numeric equality check (shell-out to .elai_cc/checkers/math/sympy.py)",
        true,
    ),
    (
        "z3",
        "Z3 SMT solver (deferred — helper script not yet deployed)",
        false,
    ),
    (
        "compile_rust",
        "cargo check on a candidate Rust file (shell-out to .elai_cc/checkers/code/compile_rust.sh)",
        true,
    ),
    (
        "regex_match",
        "Pure-Rust regex match against the claim text",
        true,
    ),
    (
        "json_schema_validate",
        "JSON-schema validation (deferred — no validator vendored)",
        false,
    ),
    (
        "numeric_range",
        "Inclusive numeric range check on a parsed f64 (in-process)",
        true,
    ),
];

/// Return the most recent `checker_result_emitted` rows (capped at
/// `MAX_LIMIT`) + per-result-kind aggregation. Empty response when no
/// SSB viewer is attached (Tauri-dev fresh boot, no DB).
#[tauri::command]
pub async fn get_checker_results_recent(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<RecentCheckerResults, String> {
    let cap = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => {
            return Ok(RecentCheckerResults {
                rows: Vec::new(),
                counts_by_kind: seeded_counts(),
                total: 0,
            });
        }
    };

    let rows: Vec<CheckerResultRowDto> = elai_orchestrator::recent_checker_results(&viewer, cap)
        .into_iter()
        .map(|r| CheckerResultRowDto {
            checker_id: r.checker_id,
            checker_kind: r.checker_kind,
            result_kind: r.result_kind,
            claim_id: r.claim_id,
            ts_ms: r.ts_ms,
            reason: r.reason,
        })
        .collect();

    let mut counts_by_kind = seeded_counts();
    for row in &rows {
        *counts_by_kind.entry(row.result_kind.clone()).or_insert(0) += 1;
    }

    let total = u32::try_from(rows.len()).unwrap_or(u32::MAX);
    Ok(RecentCheckerResults {
        rows,
        counts_by_kind,
        total,
    })
}

/// Seed every known result kind at zero so the panel histogram has a
/// stable column set even when a kind is empty.
fn seeded_counts() -> BTreeMap<String, u32> {
    let mut m: BTreeMap<String, u32> = BTreeMap::new();
    for k in ["pass", "fail", "inapplicable", "error"] {
        m.insert(k.into(), 0);
    }
    m
}

/// Return the static checker-registry surface. Always available even
/// when the SSB viewer is absent — the registry list is closed-enum
/// material the panel can render from constants.
#[tauri::command]
pub async fn get_checker_registry_state(
    _state: State<'_, AppState>,
) -> Result<CheckerRegistryState, String> {
    let kinds = KIND_TABLE
        .iter()
        .map(|(k, d, b)| CheckerKindEntry {
            kind: (*k).to_string(),
            description: (*d).to_string(),
            builtin_available: *b,
        })
        .collect();
    Ok(CheckerRegistryState { kinds })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dto_serializes_with_expected_field_names() {
        let dto = CheckerResultRowDto {
            checker_id: "regex-default".into(),
            checker_kind: "regex_match".into(),
            result_kind: "pass".into(),
            claim_id: "c-1".into(),
            ts_ms: 1_700_000_000_000,
            reason: String::new(),
        };
        let s = serde_json::to_string(&dto).expect("serialize");
        assert!(s.contains("\"checker_id\""));
        assert!(s.contains("\"checker_kind\""));
        assert!(s.contains("\"result_kind\""));
        assert!(s.contains("\"claim_id\""));
        assert!(s.contains("\"ts_ms\""));
        assert!(s.contains("\"reason\""));
    }

    #[test]
    fn seeded_counts_has_all_four_kinds() {
        let m = seeded_counts();
        assert_eq!(m.len(), 4);
        for k in ["pass", "fail", "inapplicable", "error"] {
            assert_eq!(*m.get(k).unwrap(), 0);
        }
    }

    #[test]
    fn kind_table_covers_six_closed_enum_variants() {
        assert_eq!(KIND_TABLE.len(), 6);
        let kinds: Vec<&str> = KIND_TABLE.iter().map(|(k, _, _)| *k).collect();
        for required in [
            "sympy",
            "z3",
            "compile_rust",
            "regex_match",
            "json_schema_validate",
            "numeric_range",
        ] {
            assert!(kinds.contains(&required), "missing kind {required}");
        }
    }

    #[test]
    fn registry_state_marks_deferred_kinds() {
        let kinds: Vec<CheckerKindEntry> = KIND_TABLE
            .iter()
            .map(|(k, d, b)| CheckerKindEntry {
                kind: (*k).to_string(),
                description: (*d).to_string(),
                builtin_available: *b,
            })
            .collect();
        let z3 = kinds.iter().find(|e| e.kind == "z3").expect("has z3");
        assert!(
            !z3.builtin_available,
            "z3 should be marked deferred until helper script lands"
        );
        let regex = kinds
            .iter()
            .find(|e| e.kind == "regex_match")
            .expect("has regex_match");
        assert!(regex.builtin_available);
    }
}
