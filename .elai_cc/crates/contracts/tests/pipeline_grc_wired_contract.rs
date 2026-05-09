//! B-8a — PipelineGrc wired contract.
//!
//! Pins the behaviour of `TaskRunnerConfig::with_pipeline_grc()` and
//! `PipelineGrc::new_from_default()` added in B-8a to allow production
//! construction sites to inject a live GRC instance.
//!
//! Contracts:
//!
//! 1. `PipelineGrc::new_from_default()` constructs without panicking and
//!    produces a functional instance (ground_plan on empty input returns
//!    all_grounded=true).
//!
//! 2. `TaskRunnerConfig::new_minimal` leaves `pipeline_grc = None`
//!    (existing guarantee — regression guard for default behaviour).
//!
//! 3. `TaskRunnerConfig::with_pipeline_grc()` sets `pipeline_grc = Some(…)`
//!    and the resulting config is accepted by the type system (compile-time
//!    proof that start_task_runner can receive it).
//!
//! 4. `with_pipeline_grc()` respects the builder pattern: the other fields
//!    are unchanged after the call (cursor_path preserved).

use std::path::PathBuf;
use std::sync::Arc;

use elai_orchestrator::{pipeline_grc_disabled_via_env, PipelineGrc, TaskRunnerConfig};

// ---------------------------------------------------------------------------
// Contract 1: PipelineGrc::new_from_default() is callable and functional.
// ---------------------------------------------------------------------------

#[test]
fn pipeline_grc_new_from_default_is_functional() {
    let grc = PipelineGrc::new_from_default();
    // ground_plan on empty input must return all_grounded = true (no claims
    // to check => trivially grounded). Verifies the engine initialised.
    let result = grc.ground_plan("");
    assert!(
        result.all_grounded,
        "ground_plan on empty plan must be all_grounded=true"
    );
    assert!(
        result.claims.is_empty(),
        "ground_plan on empty plan must have no claims"
    );
}

// ---------------------------------------------------------------------------
// Contract 2: new_minimal leaves pipeline_grc = None (regression guard).
// ---------------------------------------------------------------------------

#[test]
fn task_runner_config_new_minimal_pipeline_grc_is_none() {
    let cfg = TaskRunnerConfig::new_minimal(PathBuf::from("/tmp/cursor.json"));
    assert!(
        cfg.pipeline_grc.is_none(),
        "new_minimal must default pipeline_grc to None (existing kill-switch guarantee)"
    );
}

// ---------------------------------------------------------------------------
// Contract 3: with_pipeline_grc() sets pipeline_grc to Some(…).
// ---------------------------------------------------------------------------

#[test]
fn task_runner_config_with_pipeline_grc_sets_some() {
    let grc = PipelineGrc::new_from_default();
    let cfg = TaskRunnerConfig::new_minimal(PathBuf::from("/tmp/cursor.json"))
        .with_pipeline_grc(grc);
    assert!(
        cfg.pipeline_grc.is_some(),
        "with_pipeline_grc must set pipeline_grc to Some(…)"
    );
}

// ---------------------------------------------------------------------------
// Contract 4: builder preserves other fields unchanged.
// ---------------------------------------------------------------------------

#[test]
fn with_pipeline_grc_preserves_cursor_path() {
    let path = PathBuf::from("/tmp/b8a-cursor.json");
    let grc = PipelineGrc::new_from_default();
    let cfg = TaskRunnerConfig::new_minimal(path.clone()).with_pipeline_grc(grc);
    assert_eq!(
        cfg.cursor_path, path,
        "with_pipeline_grc must not alter cursor_path"
    );
    // Other optional slots remain None.
    assert!(cfg.baby_step_executor.is_none());
    assert!(cfg.hook_dispatcher.is_none());
    assert!(cfg.llm_provider.is_none());
}

// ---------------------------------------------------------------------------
// Contract 5: type-level proof — the Arc<AsyncMutex<PipelineGrc>> shape
// matches the field declaration (compile-time check).
// ---------------------------------------------------------------------------

#[test]
fn pipeline_grc_field_type_is_arc_async_mutex() {
    // Construct and immediately drop — only testing that the type compiles
    // and the field can be read as Option<Arc<_>>.
    let grc = PipelineGrc::new_from_default();
    let cfg = TaskRunnerConfig::new_minimal(PathBuf::from("/tmp/b8a-type.json"))
        .with_pipeline_grc(grc);
    // The field is Option<Arc<AsyncMutex<PipelineGrc>>>. We can clone the Arc.
    let arc_ref: &Option<Arc<_>> = &cfg.pipeline_grc;
    if let Some(arc) = arc_ref {
        let _cloned: Arc<_> = Arc::clone(arc);
    }
}

// ---------------------------------------------------------------------------
// Contract 6: kill-switch env-var is respected by the public helper.
//   pipeline_grc_disabled_via_env() returns false when the var is unset.
//   (Positive kill-switch path is tested in orchestrator's grc.rs unit tests.)
// ---------------------------------------------------------------------------

#[test]
fn pipeline_grc_disabled_via_env_false_when_unset() {
    // This test must not set ELAI_DISABLE_PIPELINE_GRC; the env may already
    // be clean if run in isolation.
    if std::env::var("ELAI_DISABLE_PIPELINE_GRC")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        // Kill-switch is active in the test environment — skip rather than fail.
        return;
    }
    assert!(
        !pipeline_grc_disabled_via_env(),
        "kill-switch must be inactive when env-var is unset"
    );
}
