//! Batch 3C (Phase 4H) — Layer 1A self-modification contract.
//!
//! Pins the behaviour of [`elai_orchestrator::DefaultLayer1aProcessor`] so
//! the Layer 1A semantics stay executable.

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::{
    acquire_rules_lock, build_pattern_regex, classify_table, estimate_fpr, is_allowed,
    is_security_sensitive, layer1a_auto_apply_via_env, layer1a_disabled_via_env, AllowedConfigKey,
    DataTable, DataTableProposal, DefaultLayer1aProcessor, FailureRecord, Layer1aError,
    Layer1aProcessor, MsiStatus, PipelineMode, PipelineOutcome, ProposalSource, Ssb, SsbConfig,
    ELAI_DISABLE_LAYER1A_ENV, ELAI_LAYER1A_AUTO_APPLY_ENV, LAYER1A_AGENT_ID,
    MSI_DIVERGING_THRESHOLD, MSI_IMPROVING_THRESHOLD, RULES_DB_LOCKFILE,
};
use rusqlite::Connection;
use tokio::sync::Mutex as AsyncMutex;

// ---------------------------------------------------------------------------
// Shared env guard — layer1a env vars are process-wide.
// ---------------------------------------------------------------------------

fn env_mutex() -> &'static std::sync::Mutex<()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
}

// ---------------------------------------------------------------------------
// Helpers — open an in-memory-ish rules.db under a tempdir + an SSB writer.
// ---------------------------------------------------------------------------

struct Fixture {
    _tmp: tempfile::TempDir,
    processor: Arc<DefaultLayer1aProcessor>,
    _ssb: Arc<Ssb>,
    rules_db_path: PathBuf,
}

async fn fixture() -> Fixture {
    let tmp = tempfile::tempdir().expect("tempdir");
    let rules_db_path = tmp.path().join("rules.db");
    let conn = Connection::open(&rules_db_path).expect("open rules.db");
    let rules_db = Arc::new(AsyncMutex::new(conn));
    let ssb_path = tmp.path().join("ssb.db");
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let ssb = Ssb::new(
        SsbConfig {
            db_path: ssb_path,
            pool_size: Some(2),
            project_id: "proj-test".to_owned(),
            session_id: "sess-test".to_owned(),
        },
        clock,
    )
    .expect("ssb init");
    let ssb_arc = Arc::new(ssb);
    let processor = DefaultLayer1aProcessor::new(rules_db, rules_db_path.clone(), ssb_arc.clone())
        .with_tuning(3, 0.05, 50, 0.3);
    processor.ensure_tables().await.expect("ensure_tables");
    Fixture {
        _tmp: tmp,
        processor: Arc::new(processor),
        _ssb: ssb_arc,
        rules_db_path,
    }
}

fn failure(category: &str, pattern: &str, occurrences: u32) -> FailureRecord {
    FailureRecord {
        category: category.to_owned(),
        pattern: pattern.to_owned(),
        occurrences,
        is_false_positive: false,
    }
}

fn outcome_with_failures(failures: Vec<FailureRecord>) -> PipelineOutcome {
    PipelineOutcome {
        task_id: "task-1".to_owned(),
        success: false,
        failures,
        findings: vec![],
        pipeline_mode: PipelineMode::Surgical,
        elapsed_ms: 42,
    }
}

// ---------------------------------------------------------------------------
// Contract 1 — trait-object dyn-safety.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_trait_is_dyn_safe() {
    let f = fixture().await;
    let dynref: Arc<dyn Layer1aProcessor> = f.processor.clone();
    let out = dynref.list_pending().await;
    assert!(out.is_empty());
}

// ---------------------------------------------------------------------------
// Contract 2 — process_outcome extracts patterns with ≥ min_occurrences.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_process_outcome_produces_proposal_at_threshold() {
    let f = fixture().await;
    let outcome = outcome_with_failures(vec![failure("logic", "unimplemented!()", 3)]);
    let proposals = f.processor.process_outcome(&outcome).await;
    assert_eq!(proposals.len(), 1);
    assert_eq!(proposals[0].occurrences_seen, 3);
    assert_eq!(proposals[0].pipeline_mode, PipelineMode::Surgical);
    assert_eq!(proposals[0].source, ProposalSource::AutoFromOutcome);
}

#[tokio::test]
async fn contract_process_outcome_ignores_below_threshold() {
    let f = fixture().await;
    let outcome = outcome_with_failures(vec![failure("logic", "todo!()", 2)]);
    let proposals = f.processor.process_outcome(&outcome).await;
    assert!(proposals.is_empty());
}

// ---------------------------------------------------------------------------
// Contract 3 — FPR > max_fpr rejected.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_high_fpr_rejected() {
    // Fixture max_fpr = 0.05. Pattern `fn main` matches 1/10 corpus entries,
    // producing fpr 0.1 > 0.05 so `process_outcome` must suppress it.
    let f = fixture().await;
    let outcome = outcome_with_failures(vec![failure("style", "fn main", 5)]);
    let proposals = f.processor.process_outcome(&outcome).await;
    assert!(
        proposals.is_empty(),
        "high-FPR pattern leaked through: {proposals:?}"
    );
}

// ---------------------------------------------------------------------------
// Contract 4 — apply_proposal writes to rules.db.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_apply_proposal_writes_row() {
    let f = fixture().await;
    let outcome = outcome_with_failures(vec![failure("logic", "unimplemented!()", 5)]);
    let proposals = f.processor.process_outcome(&outcome).await;
    assert_eq!(proposals.len(), 1);
    let proposal = &proposals[0];
    let out = f
        .processor
        .apply_proposal(proposal)
        .await
        .expect("apply ok");
    assert_eq!(out.applied_by, LAYER1A_AGENT_ID);
    let applied = f.processor.list_applied(10).await;
    assert_eq!(applied.len(), 1);
    assert_eq!(applied[0].proposal_id, proposal.proposal_id);
    assert_eq!(applied[0].table, proposal.table);
}

// ---------------------------------------------------------------------------
// Contract 5 — Security-sensitive proposal rejected.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_security_sensitive_returns_error() {
    let f = fixture().await;
    let sensitive_row = serde_json::json!({
        "category": "auth_bypass",
        "pattern": "auth_token",
    });
    let proposal = DataTableProposal {
        proposal_id: "pp-sec-1".into(),
        table: DataTable::FailureTaxonomy,
        new_row: sensitive_row,
        source: ProposalSource::Manual,
        fpr_estimate: 0.0,
        occurrences_seen: 3,
        pipeline_mode: PipelineMode::Surgical,
        scope: None,
    };
    let err = f.processor.apply_proposal(&proposal).await.unwrap_err();
    assert!(matches!(err, Layer1aError::SecuritySensitive(_)));
}

// ---------------------------------------------------------------------------
// Contract 6 — Archive cap 50 (use smaller cap for fast test).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_archive_cap_drops_oldest() {
    // Rebuild processor with a 3-entry archive cap so we don't need 51 applies.
    let tmp = tempfile::tempdir().expect("tempdir");
    let rules_db_path = tmp.path().join("rules.db");
    let conn = Connection::open(&rules_db_path).unwrap();
    let rules_db = Arc::new(AsyncMutex::new(conn));
    let ssb_path = tmp.path().join("ssb.db");
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let ssb = Ssb::new(
        SsbConfig {
            db_path: ssb_path,
            pool_size: Some(2),
            project_id: "proj".into(),
            session_id: "sess".into(),
        },
        clock,
    )
    .unwrap();
    let proc_ = DefaultLayer1aProcessor::new(rules_db.clone(), rules_db_path, Arc::new(ssb))
        .with_tuning(1, 0.5, 3, 0.3);
    proc_.ensure_tables().await.unwrap();
    for i in 0..5 {
        let outcome =
            outcome_with_failures(vec![failure("logic", &format!("unimpl_{i}_foo"), 2)]);
        let proposals = proc_.process_outcome(&outcome).await;
        for p in &proposals {
            let _ = proc_.apply_proposal(p).await;
        }
    }
    // Archive size capped at 3.
    let conn = rules_db.lock().await;
    let count: u32 = conn
        .query_row(
            "SELECT COUNT(*) FROM config_archive WHERE pipeline_mode = 'surgical'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    assert!(count <= 3, "expected ≤3, got {count}");
    assert!(count > 0, "archive must have entries");
}

// ---------------------------------------------------------------------------
// Contract 7 — MSI updates: IMPROVING on success, DIVERGING on failure
// after enough samples.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_msi_improving_after_apply_streak() {
    let f = fixture().await;
    for i in 0..5 {
        let outcome =
            outcome_with_failures(vec![failure("logic", &format!("unimpl_ok_{i}"), 3)]);
        let proposals = f.processor.process_outcome(&outcome).await;
        for p in &proposals {
            let _ = f.processor.apply_proposal(p).await;
        }
    }
    let status = f.processor.msi_status().await;
    let surgical: &MsiStatus = status
        .iter()
        .find(|s| s.pipeline_mode == PipelineMode::Surgical)
        .expect("surgical msi");
    assert!(
        surgical.msi_value > MSI_IMPROVING_THRESHOLD,
        "expected IMPROVING after 5 applies, got {surgical:?}"
    );
    assert_eq!(surgical.status, "IMPROVING");
}

// ---------------------------------------------------------------------------
// Contract 8 — Filesystem lock acquired during apply.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_fs_lock_guard_released_on_drop() {
    let tmp = tempfile::tempdir().unwrap();
    let rules_db_path = tmp.path().join("rules.db");
    std::fs::write(&rules_db_path, b"").unwrap();
    {
        let _g = acquire_rules_lock(&rules_db_path).expect("first lock");
        // The stub lockfile should exist.
        let lock_path = tmp.path().join(RULES_DB_LOCKFILE);
        assert!(lock_path.exists(), "lockfile missing while guard alive");
        // Second acquire fails while first is held.
        assert!(acquire_rules_lock(&rules_db_path).is_err());
    }
    // After drop the lock file is cleaned up and re-acquire succeeds.
    let _g2 = acquire_rules_lock(&rules_db_path).expect("re-acquire ok after drop");
}

// ---------------------------------------------------------------------------
// Contract 9 — Git commit attempt emits via dispatcher when wired.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_git_dispatcher_called_on_apply() {
    use elai_core::{
        CommitOutcome, EditOutcome, EditRequest, FailureInput, FileView, GitCommitRequest,
        IncrementalOutcome, IncrementalRequest, ToolDispatchError, ToolDispatcher,
        ValidateRequest, ValidationTierResult,
    };
    struct RecordingDispatcher {
        seen: Arc<std::sync::Mutex<Vec<GitCommitRequest>>>,
    }
    impl ToolDispatcher for RecordingDispatcher {
        fn dispatch_edit(
            &self,
            _req: EditRequest,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<Output = Result<EditOutcome, ToolDispatchError>>
                    + Send
                    + '_,
            >,
        > {
            Box::pin(async { Err(ToolDispatchError::NotImplemented("stub".into())) })
        }
        fn dispatch_validate(
            &self,
            _req: ValidateRequest,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = Result<Vec<ValidationTierResult>, ToolDispatchError>,
                    > + Send
                    + '_,
            >,
        > {
            Box::pin(async { Err(ToolDispatchError::NotImplemented("stub".into())) })
        }
        fn dispatch_classify_failure(
            &self,
            _input: FailureInput,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<Output = Option<elai_core::ClassifiedFailure>>
                    + Send
                    + '_,
            >,
        > {
            Box::pin(async { None })
        }
        fn dispatch_git_commit(
            &self,
            req: GitCommitRequest,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<Output = Result<CommitOutcome, ToolDispatchError>>
                    + Send
                    + '_,
            >,
        > {
            let seen = self.seen.clone();
            Box::pin(async move {
                seen.lock().unwrap().push(req.clone());
                Ok(CommitOutcome {
                    sha: "deadbeef".into(),
                    branch: "layer1a".into(),
                    files_count: 1,
                    message: req.message,
                })
            })
        }
        fn dispatch_file_read_outline(
            &self,
            _path: std::path::PathBuf,
            _threshold: usize,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<Output = Result<FileView, ToolDispatchError>>
                    + Send
                    + '_,
            >,
        > {
            Box::pin(async { Err(ToolDispatchError::NotImplemented("stub".into())) })
        }
        fn dispatch_incremental_compile(
            &self,
            _req: IncrementalRequest,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = Result<IncrementalOutcome, ToolDispatchError>,
                    > + Send
                    + '_,
            >,
        > {
            Box::pin(async { Err(ToolDispatchError::NotImplemented("stub".into())) })
        }
    }

    let tmp = tempfile::tempdir().unwrap();
    let rules_db_path = tmp.path().join("rules.db");
    let conn = Connection::open(&rules_db_path).unwrap();
    let rules_db = Arc::new(AsyncMutex::new(conn));
    let ssb_path = tmp.path().join("ssb.db");
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let ssb = Ssb::new(
        SsbConfig {
            db_path: ssb_path,
            pool_size: Some(2),
            project_id: "proj".into(),
            session_id: "sess".into(),
        },
        clock,
    )
    .unwrap();
    let seen: Arc<std::sync::Mutex<Vec<GitCommitRequest>>> =
        Arc::new(std::sync::Mutex::new(Vec::new()));
    let dispatcher: Arc<dyn ToolDispatcher> = Arc::new(RecordingDispatcher {
        seen: seen.clone(),
    });
    let processor =
        DefaultLayer1aProcessor::new(rules_db, rules_db_path, Arc::new(ssb))
            .with_tuning(3, 0.5, 50, 0.3)
            .with_dispatcher(dispatcher);
    processor.ensure_tables().await.unwrap();
    let outcome = outcome_with_failures(vec![failure("logic", "unimplemented!()", 4)]);
    let proposals = processor.process_outcome(&outcome).await;
    let p = proposals.first().expect("proposal generated");
    let out = processor.apply_proposal(p).await.expect("apply");
    assert_eq!(out.commit_sha.as_deref(), Some("deadbeef"));
    assert_eq!(seen.lock().unwrap().len(), 1);
    let commits = seen.lock().unwrap();
    assert!(
        commits[0].message.contains("layer1a: auto-add"),
        "unexpected commit message: {:?}",
        commits[0].message
    );
}

// ---------------------------------------------------------------------------
// Contract 10 — list_pending / list_applied return persisted records.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_pending_and_applied_listing() {
    let f = fixture().await;
    let outcome = outcome_with_failures(vec![failure("logic", "unimpl_listing", 3)]);
    let proposals = f.processor.process_outcome(&outcome).await;
    assert_eq!(proposals.len(), 1);
    let pending = f.processor.list_pending().await;
    assert_eq!(pending.len(), 1);
    f.processor.apply_proposal(&proposals[0]).await.unwrap();
    let after = f.processor.list_pending().await;
    assert_eq!(after.len(), 0, "applied proposal should disappear from pending");
    let applied = f.processor.list_applied(10).await;
    assert_eq!(applied.len(), 1);
    assert_eq!(applied[0].proposal_id, proposals[0].proposal_id);
}

// ---------------------------------------------------------------------------
// Contract 11 — ELAI_DISABLE_LAYER1A kill switch is observable via the env
// helper. The dashboard integration then uses this to skip construction.
// ---------------------------------------------------------------------------

#[test]
fn contract_kill_switch_env_observed() {
    let _g = env_mutex().lock().unwrap();
    let prev = std::env::var(ELAI_DISABLE_LAYER1A_ENV).ok();
    std::env::set_var(ELAI_DISABLE_LAYER1A_ENV, "1");
    assert!(layer1a_disabled_via_env());
    std::env::remove_var(ELAI_DISABLE_LAYER1A_ENV);
    assert!(!layer1a_disabled_via_env());
    if let Some(v) = prev {
        std::env::set_var(ELAI_DISABLE_LAYER1A_ENV, v);
    }
}

// ---------------------------------------------------------------------------
// Contract 12 — ELAI_LAYER1A_AUTO_APPLY env helper.
// ---------------------------------------------------------------------------

#[test]
fn contract_auto_apply_env_observed() {
    let _g = env_mutex().lock().unwrap();
    let prev = std::env::var(ELAI_LAYER1A_AUTO_APPLY_ENV).ok();
    std::env::remove_var(ELAI_LAYER1A_AUTO_APPLY_ENV);
    assert!(!layer1a_auto_apply_via_env());
    std::env::set_var(ELAI_LAYER1A_AUTO_APPLY_ENV, "true");
    assert!(layer1a_auto_apply_via_env());
    std::env::remove_var(ELAI_LAYER1A_AUTO_APPLY_ENV);
    if let Some(v) = prev {
        std::env::set_var(ELAI_LAYER1A_AUTO_APPLY_ENV, v);
    }
}

// ---------------------------------------------------------------------------
// Contract 13 — Absence (None on TaskRunnerConfig) falls back gracefully.
// (Type-level check only — if TaskRunnerConfig's field changes name the
// contract fails to compile.)
// ---------------------------------------------------------------------------

#[test]
fn contract_task_runner_config_has_layer1a_field() {
    let cfg = elai_orchestrator::TaskRunnerConfig::new_minimal(PathBuf::from("cursor.json"));
    assert!(cfg.layer1a_processor.is_none());
}

// ---------------------------------------------------------------------------
// Contract 14 — is_allowed only accepts DataTable variants.
// ---------------------------------------------------------------------------

#[test]
fn contract_allowed_config_key_rules() {
    assert!(is_allowed(&AllowedConfigKey::DataTable(
        DataTable::DeprecatedPatterns
    )));
    assert!(is_allowed(&AllowedConfigKey::DataTable(
        DataTable::FailureTaxonomy
    )));
    assert!(is_allowed(&AllowedConfigKey::DataTable(
        DataTable::LazyCodeRegex
    )));
    assert!(is_allowed(&AllowedConfigKey::DataTable(
        DataTable::SourceTiers
    )));
    // C-21 Task 7 — closed-enum ParameterKey replaces String. The previous
    // `AllowedConfigKey::Parameter("budget".into())` no longer compiles
    // because "budget" is not in the spec-canonical 12-variant set.
    assert!(!is_allowed(&AllowedConfigKey::Parameter(
        elai_orchestrator::ParameterKey::EtcThreshold,
    )));
}

// ---------------------------------------------------------------------------
// Contract 15 — classify_table + is_security_sensitive + build_pattern_regex
// utility rules.
// ---------------------------------------------------------------------------

#[test]
fn contract_classify_table_rules() {
    assert_eq!(
        classify_table("empty_body", "todo!()"),
        DataTable::LazyCodeRegex
    );
    assert_eq!(
        classify_table("deprecated_api", "blake2"),
        DataTable::DeprecatedPatterns
    );
    assert_eq!(classify_table("source", "arxiv"), DataTable::SourceTiers);
    assert_eq!(classify_table("other", "foobar"), DataTable::FailureTaxonomy);
}

#[test]
fn contract_is_security_sensitive_catches_keywords() {
    let bad = serde_json::json!({"pattern": "leak_private_key"});
    assert!(is_security_sensitive(&bad));
    let ok = serde_json::json!({"pattern": "println_spam"});
    assert!(!is_security_sensitive(&ok));
}

#[test]
fn contract_build_pattern_regex_rejects_short_pattern() {
    assert!(build_pattern_regex("xy").is_err());
    assert!(build_pattern_regex("todo!()").is_ok());
}

#[test]
fn contract_estimate_fpr_matches_known_good_ratio() {
    let rx = build_pattern_regex("todo!()").unwrap();
    assert_eq!(estimate_fpr(&rx), 0.0);
    // A bare `fn(` substring matches several corpus entries — FPR must be positive.
    let rx_broad = build_pattern_regex("fn main").unwrap();
    let fpr = estimate_fpr(&rx_broad);
    assert!(fpr > 0.0, "expected positive FPR for broad pattern, got {fpr}");
}

// ---------------------------------------------------------------------------
// Contract 16 — MSI thresholds sanity.
// ---------------------------------------------------------------------------

#[test]
fn contract_msi_thresholds() {
    assert!(MSI_IMPROVING_THRESHOLD > 0.0);
    assert!(MSI_DIVERGING_THRESHOLD < 0.0);
    assert!(MSI_IMPROVING_THRESHOLD > MSI_DIVERGING_THRESHOLD);
}
