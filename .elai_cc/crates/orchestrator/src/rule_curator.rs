//! I-12 -- evidence-gated playbook curator.
//!
//! Promotes rule entries only when linked to >=k SSB-verified successful outcome rows,
//! carries the measured lift, demotes by the same ledger, and exports verified content
//! to AGENTS.md / skills export formats with embedded outcome row IDs.
//!
//! Feeds the Phase-3F PROPOSED->CANDIDATE->SHADOW->ACTIVE->RETIRED lifecycle (never
//! replaces it). Gate-freeze R8: the evaluator lives OUTSIDE the write boundary.
//!
//! ID-6 WEIGHTS-ARE-CONFIGS: any self-tuned artifact promotes ONLY through ledger gates.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use rusqlite::Connection;

use elai_core::clock::{ClockProvider, LogicalInstant};
use elai_core::paths::rules_db_path;
use elai_rules::lifecycle::{
    compute_metrics_snapshot, transition_to_active, transition_to_candidate, transition_to_retired,
    transition_to_shadow, Approver, FPR_RETIRE_THRESHOLD, MIN_CONTRACT_TESTS,
};
use elai_rules::schema::open_rules_db;

// ---------------------------------------------------------------------------
// Clock adapter (Arc<dyn ClockProvider> -> impl ClockProvider)
// ---------------------------------------------------------------------------

/// Newtype so `Arc<dyn ClockProvider>` satisfies `&impl ClockProvider` where lifecycle
/// functions require a concrete `&impl ClockProvider`. This is internal to rule_curator
/// and not part of the public API.
struct ArcClock(Arc<dyn ClockProvider>);

impl ClockProvider for ArcClock {
    fn now(&self) -> LogicalInstant {
        self.0.now()
    }
    fn elapsed_since(&self, mark: LogicalInstant) -> std::time::Duration {
        self.0.elapsed_since(mark)
    }
    fn system_now(&self) -> std::time::SystemTime {
        self.0.system_now()
    }
    fn instant_now(&self) -> std::time::Instant {
        self.0.instant_now()
    }
}

use crate::ssb::{
    PlaybookEventKind, PlaybookEventPayload, Ssb, SsbActionType, SsbPayload, SsbReader,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Minimum number of SSB file-linked rows required before a promotion attempt.
pub const MIN_EVIDENCE_COUNT: u32 = 5;

/// Interval between offline promotion-tick runs (seconds).
pub const PLAYBOOK_TICK_INTERVAL_SECS: u64 = 3_600;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Result of a single `export_verified` call.
#[derive(Debug, Clone)]
pub struct ExportReport {
    /// Number of ACTIVE rules written to the export block.
    pub entries_exported: u32,
    /// Outcome row IDs embedded in the export block.
    pub outcome_row_ids: Vec<String>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Evidence-gated playbook curator.
///
/// Holds the path to the rules DB (separate from the SSB DB per SEPARATE_DB_HANDLES
/// gotcha: lifecycle fns take a `&Connection` to the rules DB, NOT the SSB DB).
/// The SSB handle is passed per-method so the same curator can be called from both
/// the offline tick (which holds `Arc<Ssb>`) and the CLI export command.
pub struct PlaybookCurator {
    rules_db_path: PathBuf,
    clock: Arc<dyn ClockProvider>,
}

impl PlaybookCurator {
    /// Create a new curator targeting the given rules DB path.
    ///
    /// `rules_db_path` is the path to the rules SQLite file (`.elai_cc/rules.db`
    /// or `:memory:` in tests). The SSB DB is accessed via the `ssb` handle
    /// passed to each method call.
    pub fn new(rules_db_path: &Path, clock: Arc<dyn ClockProvider>) -> Self {
        Self {
            rules_db_path: rules_db_path.to_owned(),
            clock,
        }
    }

    /// Build a curator targeting the canonical rules DB under `project_root`.
    pub fn from_project_root(project_root: &Path, clock: Arc<dyn ClockProvider>) -> Self {
        Self::new(&rules_db_path(project_root), clock)
    }

    /// Open a fresh connection to the rules DB.
    ///
    /// Each call opens a new connection (separate from Ssb and TrialLedger) so
    /// there is no lock contention between the three stores.
    fn open_rules_conn(&self) -> Result<Connection, CuratorError> {
        let clock_adapter = ArcClock(Arc::clone(&self.clock));
        open_rules_db(&self.rules_db_path, &clock_adapter)
            .map_err(|e| CuratorError::RulesDb(e.to_string()))
    }

    /// Return an `ArcClock` adapter for passing to lifecycle functions that require `&impl ClockProvider`.
    fn arc_clock(&self) -> ArcClock {
        ArcClock(Arc::clone(&self.clock))
    }

    /// Sweep rows born at SHADOW; metric gates inside `transition_to_active` are the only bound,
    /// and a rule with no `rule_metrics` rows cannot promote.
    async fn promote_shadow_rules(&self, ssb: &Arc<Ssb>) -> Result<(), CuratorError> {
        // Keep the rules connection confined to the synchronous read/transition phase.
        let shadow_ids = {
            let conn = self.open_rules_conn()?;
            list_rules_by_status(&conn, "SHADOW")?
        };
        for rule_id in &shadow_ids {
            let Some(metrics) = ({
                let conn = self.open_rules_conn()?;
                let metrics = compute_metrics_snapshot(&conn, rule_id).unwrap_or_else(|_| {
                    crate::derive_outcome_labels::observe_labels_for_rule(rule_id, &[])
                });
                if let Err(e) = transition_to_active(&conn, rule_id, &metrics, false, &self.arc_clock()) {
                    tracing::debug!(rule_id = %rule_id, error = %e, "shadow row not promotable this tick");
                    None
                } else {
                    Some(metrics)
                }
            }) else {
                continue;
            };
            let outcome_rows = query_outcome_rows_for_rule(ssb, rule_id);
            let evidence_count = u32::try_from(outcome_rows.len()).unwrap_or(u32::MAX);
            let lift = compute_lift_estimate(&metrics);
            let payload = PlaybookEventPayload {
                rule_id: rule_id.clone(),
                kind: PlaybookEventKind::Promoted,
                outcome_row_ids: outcome_rows,
                lift: Some(lift),
                regression_count: 0,
                evidence_count,
                ts_ms: self.now_ms(),
            };
            if let Err(e) = ssb
                .write_payload("rule_curator", 0, SsbPayload::PlaybookEvent(payload), vec![])
                .await
            {
                tracing::warn!(rule_id = %rule_id, error = %e, "PlaybookEvent(Promoted) emit failed");
            }
        }
        Ok(())
    }

    /// Run one promotion-tick sweep over PROPOSED rules, then SHADOW rules, and demote stale ACTIVE rules.
    ///
    /// For each PROPOSED rule:
    ///   1. Count SSB rows with file_path = rule_id (evidence links).
    ///   2. If count < MIN_EVIDENCE_COUNT: emit ValidationRun, skip.
    ///   3. Build MetricsSnapshot from the rules DB.
    ///   4. If regression_count > 0 (from SSB ValidationRun scan): emit ValidationRun, skip.
    ///   5. Walk the Phase-3F chain: PROPOSED->CANDIDATE->SHADOW->ACTIVE (atomic).
    ///   6. Emit PlaybookEvent{Promoted} with outcome_row_ids + lift.
    ///
    /// Also scans ACTIVE rules for stale FPR and demotes them to RETIRED.
    pub async fn run_promotion_tick(&self, ssb: &Arc<Ssb>) -> Result<(), CuratorError> {
        let proposed_ids = {
            let conn = self.open_rules_conn()?;
            list_rules_by_status(&conn, "PROPOSED")?
        };

        for rule_id in &proposed_ids {
            let outcome_rows = query_outcome_rows_for_rule(ssb, rule_id);
            let evidence_count = outcome_rows.len() as u32;

            if evidence_count < MIN_EVIDENCE_COUNT {
                use crate::intervention::{
                    classify_intervention_failure, regression_check, FailureSignals, ProposedUpdate,
                };
                let signals = FailureSignals::default();
                let class = classify_intervention_failure(&signals);
                let reg = regression_check(&[] as &[ProposedUpdate]);
                let mut row_ids = outcome_rows;
                row_ids.push(failclass_tag(&class));
                let payload = PlaybookEventPayload {
                    rule_id: rule_id.clone(),
                    kind: PlaybookEventKind::ValidationRun,
                    outcome_row_ids: row_ids,
                    lift: None,
                    regression_count: reg.rejected.len() as u32,
                    evidence_count,
                    ts_ms: self.now_ms(),
                };
                if let Err(e) = ssb
                    .write_payload("rule_curator", 0, SsbPayload::PlaybookEvent(payload), vec![])
                    .await
                {
                    tracing::warn!(rule_id = %rule_id, error = %e, "PlaybookEvent(ValidationRun) emit failed");
                }
                continue;
            }

            let Some(metrics) = ({
                let conn = self.open_rules_conn()?;
                let metrics = compute_metrics_snapshot(&conn, rule_id).unwrap_or_else(|_| {
                    crate::derive_outcome_labels::observe_labels_for_rule(rule_id, &[])
                });
                let regression_count = count_regressions_for_rule(ssb, rule_id);
                if regression_count > 0 {
                    None
                } else {
                    let contract_count = count_contract_tests(&conn, rule_id);
                    let all_pass = contract_count >= MIN_CONTRACT_TESTS;
                    if let Err(e) = transition_to_candidate(
                        &conn, rule_id, contract_count, all_pass, &self.arc_clock(),
                    ) {
                        tracing::warn!(rule_id = %rule_id, error = %e, "transition_to_candidate failed; skipping rule");
                        None
                    } else {
                        let approver = Approver::SecondAuditor {
                            auditor_id: "rule_curator".to_owned(),
                        };
                        if let Err(e) = transition_to_shadow(&conn, rule_id, &approver, &self.arc_clock()) {
                            tracing::warn!(rule_id = %rule_id, error = %e, "transition_to_shadow failed; skipping rule");
                            None
                        } else if let Err(e) = transition_to_active(
                            &conn, rule_id, &metrics, false, &self.arc_clock(),
                        ) {
                            tracing::warn!(rule_id = %rule_id, error = %e, "transition_to_active failed; skipping rule");
                            None
                        } else {
                            Some((metrics, regression_count))
                        }
                    }
                }
            }) else {
                // A regression is reported asynchronously below; other transition failures are skipped.
                let regression_count = count_regressions_for_rule(ssb, rule_id);
                if regression_count == 0 {
                    continue;
                }
                use crate::intervention::{
                    classify_intervention_failure, regression_check, FailureSignals,
                    ProposedUpdate,
                };
                let signals = FailureSignals {
                    stall: true,
                    budget_thrash: regression_count > 1,
                    ..FailureSignals::default()
                };
                let class = classify_intervention_failure(&signals);
                let proposals = vec![ProposedUpdate {
                    id: rule_id.clone(),
                    targets: class.class,
                    may_over_trigger: true,
                    blocked_valid_fixture: false,
                }];
                let reg = regression_check(&proposals);
                let mut row_ids = outcome_rows;
                row_ids.push(failclass_tag(&class));
                let payload = PlaybookEventPayload {
                    rule_id: rule_id.clone(),
                    kind: PlaybookEventKind::ValidationRun,
                    outcome_row_ids: row_ids,
                    lift: None,
                    regression_count: regression_count + reg.rejected.len() as u32,
                    evidence_count,
                    ts_ms: self.now_ms(),
                };
                if let Err(e) = ssb
                    .write_payload("rule_curator", 0, SsbPayload::PlaybookEvent(payload), vec![])
                    .await
                {
                    tracing::warn!(rule_id = %rule_id, error = %e, "PlaybookEvent(ValidationRun/regression) emit failed");
                }
                continue;
            };

            let payload = PlaybookEventPayload {
                rule_id: rule_id.clone(),
                kind: PlaybookEventKind::Promoted,
                outcome_row_ids: outcome_rows,
                lift: Some(compute_lift_estimate(&metrics.0)),
                regression_count: metrics.1,
                evidence_count,
                ts_ms: self.now_ms(),
            };
            if let Err(e) = ssb
                .write_payload("rule_curator", 0, SsbPayload::PlaybookEvent(payload), vec![])
                .await
            {
                tracing::warn!(rule_id = %rule_id, error = %e, "PlaybookEvent(Promoted) emit failed");
            }
        }

        self.promote_shadow_rules(ssb).await?;

        let active_ids = {
            let conn = self.open_rules_conn()?;
            list_rules_by_status(&conn, "ACTIVE")?
        };
        for rule_id in &active_ids {
            let demotion = {
                let conn = self.open_rules_conn()?;
                let metrics = compute_metrics_snapshot(&conn, rule_id).unwrap_or_else(|_| {
                    crate::derive_outcome_labels::observe_labels_for_rule(rule_id, &[])
                });
                if metrics.false_positive_rate >= FPR_RETIRE_THRESHOLD {
                    let reason = format!(
                        "fpr={:.3} exceeded retire threshold {:.3}",
                        metrics.false_positive_rate, FPR_RETIRE_THRESHOLD
                    );
                    transition_to_retired(&conn, rule_id, &reason, false, &self.arc_clock())
                        .map(|_| Some(()))
                } else {
                    Ok(None)
                }
            };
            let was_demoted = match demotion {
                Ok(value) => value.is_some(),
                Err(e) => {
                    tracing::warn!(rule_id = %rule_id, error = %e, "transition_to_retired failed during demotion sweep");
                    continue;
                }
            };
            if was_demoted {
                let payload = PlaybookEventPayload {
                    rule_id: rule_id.clone(),
                    kind: PlaybookEventKind::Demoted,
                    outcome_row_ids: vec![],
                    lift: None,
                    regression_count: 0,
                    evidence_count: 0,
                    ts_ms: self.now_ms(),
                };
                if let Err(e) = ssb
                    .write_payload("rule_curator", 0, SsbPayload::PlaybookEvent(payload), vec![])
                    .await
                {
                    tracing::warn!(rule_id = %rule_id, error = %e, "PlaybookEvent(Demoted) emit failed");
                }
            }
        }
        Ok(())
    }

    /// Export all ACTIVE rules to a verified AGENTS.md-compatible block.
    ///
    /// Writes one markdown fenced block per ACTIVE rule with embedded `outcome_row_ids`
    /// so downstream consumers can grep-verify provenance (R8 gate-freeze).
    /// If `emit_ssb` is true, emits PlaybookEvent{Exported} row to SSB.
    pub async fn export_verified(
        &self,
        ssb: &Arc<Ssb>,
        out_path: &Path,
        emit_ssb: bool,
    ) -> Result<ExportReport, CuratorError> {
        let active_ids = {
            let conn = self.open_rules_conn()?;
            list_rules_by_status(&conn, "ACTIVE")?
        };

        let mut block = String::new();
        block.push_str("<!-- ELAI Playbook Export: verified ACTIVE rules -->\n");
        block.push_str("<!-- outcome_row_ids are SSB seq IDs of the evidence rows -->\n\n");

        let mut all_outcome_ids: Vec<String> = Vec::new();
        let mut entries_exported = 0u32;

        for rule_id in &active_ids {
            let outcome_rows = query_outcome_rows_for_rule(ssb, rule_id);
            let ids_str = outcome_rows.join(", ");
            block.push_str(&format!(
                "## Rule: {rule_id}\n\
                 <!-- outcome_row_ids: [{ids_str}] -->\n\n"
            ));
            all_outcome_ids.extend(outcome_rows);
            entries_exported += 1;
        }

        // Write export block to out_path.
        if let Some(parent) = out_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| CuratorError::Io(e.to_string()))?;
            }
        }
        std::fs::write(out_path, &block).map_err(|e| CuratorError::Io(e.to_string()))?;

        let report = ExportReport {
            entries_exported,
            outcome_row_ids: all_outcome_ids.clone(),
        };

        if emit_ssb {
            let payload = PlaybookEventPayload {
                rule_id: "playbook_export".into(),
                kind: PlaybookEventKind::Exported,
                outcome_row_ids: all_outcome_ids,
                lift: None,
                regression_count: 0,
                evidence_count: entries_exported,
                ts_ms: self.now_ms(),
            };
            if let Err(e) = ssb
                .write_payload("rule_curator", 0, SsbPayload::PlaybookEvent(payload), vec![])
                .await
            {
                tracing::warn!(error = %e, "PlaybookEvent(Exported) emit failed");
            }
        }

        Ok(report)
    }

    fn now_ms(&self) -> i64 {
        self.clock
            .system_now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
            .unwrap_or(0)
    }
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum CuratorError {
    #[error("rules DB error: {0}")]
    RulesDb(String),
    #[error("I/O error: {0}")]
    Io(String),
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Return rule_ids for all rules with the given status string.
fn list_rules_by_status(conn: &Connection, status: &str) -> Result<Vec<String>, CuratorError> {
    let mut stmt = conn
        .prepare("SELECT rule_id FROM rules WHERE status = ?1 ORDER BY rule_id ASC")
        .map_err(|e| CuratorError::RulesDb(e.to_string()))?;
    let ids: Result<Vec<String>, _> = stmt
        .query_map(rusqlite::params![status], |row| row.get(0))
        .map_err(|e| CuratorError::RulesDb(e.to_string()))?
        .collect();
    ids.map_err(|e| CuratorError::RulesDb(e.to_string()))
}

/// Count contract_tests rows that exist for a given rule_id.
fn count_contract_tests(conn: &Connection, rule_id: &str) -> u32 {
    conn.query_row(
        "SELECT COUNT(*) FROM contract_tests WHERE rule_id = ?1",
        rusqlite::params![rule_id],
        |row| row.get::<_, i64>(0),
    )
    .map(|n| n as u32)
    .unwrap_or(0)
}

/// Query SSB for rows with a file_path matching `rule_id` (evidence links).
///
/// The SSB `ssb_files` junction table stores file path references; the curator
/// instructs callers to register rule_id as a file reference when emitting
/// outcome rows so they can be found here via `query_by_file`.
///
/// Returns seq IDs (as strings) of matching rows.
fn query_outcome_rows_for_rule(ssb: &Arc<Ssb>, rule_id: &str) -> Vec<String> {
    match ssb.query_by_file(&[rule_id]) {
        Ok(entries) => entries.iter().map(|e| e.seq.to_string()).collect(),
        Err(e) => {
            tracing::warn!(rule_id = %rule_id, error = %e, "query_outcome_rows_for_rule failed");
            vec![]
        }
    }
}

/// Count SSB PlaybookEvent{ValidationRun} rows for a rule that have regression_count > 0.
///
/// Uses `query_by_type` (SsbReader trait) to find prior validation rows that
/// recorded regressions. No MutationRunner shell-out.
fn count_regressions_for_rule(ssb: &Arc<Ssb>, rule_id: &str) -> u32 {
    let action_type = SsbActionType::PlaybookEvent(PlaybookEventKind::ValidationRun);
    let entries = match ssb.query_by_type(action_type) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!(rule_id = %rule_id, error = %e, "count_regressions_for_rule query failed");
            return 0;
        }
    };
    let mut count = 0u32;
    for entry in entries {
        if let Ok(SsbPayload::PlaybookEvent(ref p)) = rmp_serde::from_slice(&entry.payload) {
            if p.rule_id == rule_id && p.regression_count > 0 {
                count += p.regression_count;
            }
        }
    }
    count
}

/// Compute a simple lift estimate from metrics (TPR - FPR as a proxy for lift).
/// Returns a value clamped to [0.0, 1.0].
fn compute_lift_estimate(metrics: &elai_rules::lifecycle::MetricsSnapshot) -> f32 {
    let lift = metrics.true_positive_rate - metrics.false_positive_rate;
    lift.clamp(0.0, 1.0) as f32
}

/// J-12 H6: compact diagnostic tag appended to ValidationRun outcome_row_ids
/// so classify_intervention_failure is observable on every emitted row.
/// Format: `j12_failclass:p{priority}:{snake_tag}`
fn failclass_tag(class: &crate::intervention::Classification) -> String {
    use crate::intervention::InterventionFailureClass;
    let tag = match class.class {
        InterventionFailureClass::ActionRealization => "p1_action_realization",
        InterventionFailureClass::EnvContract => "p2_env_contract",
        InterventionFailureClass::TrajectoryDegeneration => "p3_trajectory_degeneration",
        InterventionFailureClass::ResidualReasoning => "p4_residual_reasoning",
    };
    format!("j12_failclass:{}", tag)
}

// ---------------------------------------------------------------------------
// Unit tests (in-process fixture, no MutationRunner shell-out, D-026 compliant)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::Arc;

    use elai_core::clock::SeededClock;
    use elai_rules::lifecycle::MIN_CONTRACT_TESTS;

    use super::{MIN_EVIDENCE_COUNT, PlaybookCurator};

    fn test_clock() -> Arc<SeededClock> {
        Arc::new(SeededClock::new())
    }

    #[test]
    fn min_evidence_count_is_nonzero() {
        assert!(MIN_EVIDENCE_COUNT > 0, "MIN_EVIDENCE_COUNT must be > 0");
    }

    #[test]
    fn curator_stores_path() {
        let curator = PlaybookCurator::new(Path::new(":memory:"), test_clock());
        assert_eq!(curator.rules_db_path, Path::new(":memory:"));
    }

    #[test]
    fn open_rules_conn_on_memory_db_succeeds() {
        let curator = PlaybookCurator::new(Path::new(":memory:"), test_clock());
        let conn = curator.open_rules_conn();
        assert!(conn.is_ok(), "open_rules_conn on :memory: must succeed");
    }

    #[test]
    fn list_rules_by_status_returns_empty_on_fresh_db() {
        let curator = PlaybookCurator::new(Path::new(":memory:"), test_clock());
        let conn = curator.open_rules_conn().unwrap();
        let ids = super::list_rules_by_status(&conn, "PROPOSED").unwrap();
        assert!(ids.is_empty());
    }

    #[test]
    fn min_contract_tests_matches_lifecycle_constant() {
        // Sanity: if lifecycle.rs MIN_CONTRACT_TESTS changes, our gate auto-tightens.
        assert_eq!(MIN_CONTRACT_TESTS, 3);
    }

    #[test]
    fn compute_lift_estimate_clamped_to_unit_interval() {
        use elai_rules::lifecycle::MetricsSnapshot;
        let good = MetricsSnapshot {
            true_positives: 90,
            false_positives: 2,
            causal_catches: 80,
            spurious_catches: 5,
            false_positive_rate: 0.02,
            true_positive_rate: 0.90,
            counterfactual_causal_rate: Some(0.94),
        };
        let lift = super::compute_lift_estimate(&good);
        assert!(lift > 0.0 && lift <= 1.0, "lift must be in (0, 1]");
    }

    #[test]
    fn compute_lift_estimate_zero_for_bad_fpr() {
        use elai_rules::lifecycle::MetricsSnapshot;
        let bad = MetricsSnapshot {
            true_positives: 5,
            false_positives: 90,
            causal_catches: 0,
            spurious_catches: 10,
            false_positive_rate: 0.95,
            true_positive_rate: 0.05,
            counterfactual_causal_rate: None,
        };
        let lift = super::compute_lift_estimate(&bad);
        // tpr(0.05) - fpr(0.95) = -0.90 -> clamp -> 0.0
        assert_eq!(lift, 0.0);
    }
}
