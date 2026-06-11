//! H-41 — checker-coverage + gap-backlog Tauri command surface.
//!
//! Two read-only commands feed the Svelte `CheckerCoveragePanel`:
//!
//! - [`get_checker_gap_backlog`] — ranked `Vec<CheckerGapBacklogRow>` of
//!   claim-class x complexity-taxon pairs that emitted a
//!   `CheckerGapDetected` SSB row, sorted by `frequency × taxon_priority`.
//!
//! - [`get_coverage_report`] — live Mode-A coverage fraction for the session
//!   (constructive / total Factual+Computational).
//!
//! Sibling of `commands_d10` (mode-routing) and `commands_d11` (checker results).
//! The orchestrator producer (`agent_pool::route_claim_with_gap`) is the sole
//! writer; this module only reads.
//!
//! Wire-dark: `ELAI_CHECKER_GAP_ENABLE=1` required on the writer side.
//! The panel renders empty / zero-coverage when the gate is off (correct signal
//! per GD-5: coverage = REACH, not checker correctness).
//!
//! See docs/thefinalglance/H-plan/H-41_checker_coverage_gap_instrument.md.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 200;

/// Typed mirror of `elai_orchestrator::CheckerGapBacklogRow`.
/// Lives in this crate so the Tauri-side schema does not leak the
/// orchestrator's internal type through the IPC boundary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckerGapBacklogRowDto {
    pub claim_type: String,
    pub complexity_taxon: String,
    pub count: u32,
    /// Priority weight: 1 = normal, 2 = HIGH (NP-complete / undecidable).
    pub priority: u8,
}

/// Response from [`get_coverage_report`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoverageReportDto {
    /// Fraction of Factual+Computational claims that reached a Constructive checker.
    pub mode_a_fraction: f64,
    /// Raw count of Constructive-routed Factual/Computational claims.
    pub constructive_count: u64,
    /// Denominator: all Factual+Computational claims seen.
    pub total_factual_computational: u64,
    /// Session identifier (or "all" for project-wide roll-up).
    pub session_id: String,
}

/// H-41 — ranked checker-gap backlog.
///
/// Aggregates `checker_gap_detected` SSB rows by `(claim_type, complexity_taxon)`,
/// sorted by `frequency × taxon_priority` descending. HIGH-priority rows
/// (NP-complete / undecidable) appear first — those MUST route to an external
/// solver or refuse (RD-3).
///
/// Returns an empty `Vec` when no rows are recorded (ELAI_CHECKER_GAP_ENABLE
/// not set on the writer side, or no Factual/Computational gaps yet).
#[tauri::command]
pub async fn get_checker_gap_backlog(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<CheckerGapBacklogRowDto>, String> {
    let effective = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(Vec::new()),
    };
    let rows = elai_orchestrator::checker_gap_backlog(&viewer, effective);
    Ok(rows
        .into_iter()
        .map(|r| CheckerGapBacklogRowDto {
            claim_type: r.claim_type,
            complexity_taxon: r.complexity_taxon,
            count: r.count,
            priority: r.priority,
        })
        .collect())
}

/// H-41 — live Mode-A coverage report.
///
/// Rolls up `claim_routed` SSB rows to compute the fraction of
/// Factual+Computational claims that reached a Constructive checker.
/// Returns a zeroed [`CoverageReportDto`] when no rows exist.
#[tauri::command]
pub async fn get_coverage_report(
    state: State<'_, AppState>,
    session_id: Option<String>,
) -> Result<CoverageReportDto, String> {
    let sid = session_id.unwrap_or_else(|| "all".to_owned());
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => {
            return Ok(CoverageReportDto {
                mode_a_fraction: 0.0,
                constructive_count: 0,
                total_factual_computational: 0,
                session_id: sid,
            });
        }
    };
    let report = elai_orchestrator::mode_a_coverage(&viewer, &sid, 0);
    Ok(CoverageReportDto {
        mode_a_fraction: report.mode_a_fraction,
        constructive_count: report.constructive_count,
        total_factual_computational: report.total_factual_computational,
        session_id: report.session_id,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dto_serde_round_trip() {
        let dto = CheckerGapBacklogRowDto {
            claim_type: "factual".into(),
            complexity_taxon: "np_complete_or_combinatorial".into(),
            count: 3,
            priority: 2,
        };
        let json = serde_json::to_string(&dto).expect("serialize");
        let back: CheckerGapBacklogRowDto = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(dto, back);
    }

    #[test]
    fn coverage_report_dto_serde_round_trip() {
        let dto = CoverageReportDto {
            mode_a_fraction: 0.75,
            constructive_count: 3,
            total_factual_computational: 4,
            session_id: "s-test".into(),
        };
        let json = serde_json::to_string(&dto).expect("serialize");
        let back: CoverageReportDto = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(dto, back);
    }
}
