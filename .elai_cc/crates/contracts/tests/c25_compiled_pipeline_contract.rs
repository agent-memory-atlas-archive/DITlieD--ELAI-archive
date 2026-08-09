//! C-25 contract: compiled pipeline + template crystallizer + background
//! task manager surface invariants.
//!
//! Pins the following invariants:
//!   1. `SCHEMA_VERSION_PUBLIC` >= 57 (C-25 audit follow-up atomic bump
//!      for the 6th compiled-pipeline beacon — DecompileTriggered —
//!      closing WARN-1 in workflow/sessions/s-25672/audit-C25.md).
//!   2. The 6 new `SsbActionType` discriminants exist
//!      (`PlanSkeleton` / `CompiledPipelineFired` / `TemplateCrystallized`
//!      / `BackgroundTaskStarted` / `BackgroundTaskComplete` /
//!      `DecompileTriggered`) and round-trip through
//!      `ssb_action_type_from_str_public` / `as_sql_str`.
//!   3. The 6 new `SsbPayload` variants serialize + deserialize via
//!      msgpack with matching `action_type()` arm.
//!   4. `PlanSkeletonPayload.source` enum has both `Llm` + `Template`
//!      variants (DRIFT-D resolution — C-25 is the first emitter).
//!   5. `CompiledPipelineStore::load_from_disk` on corrupt input does
//!      NOT panic and returns an empty store (stub_ban contract).
//!   6. `CompiledExecutor::try_execute` returns `Err(CacheMiss)` on a
//!      cold cache (stub_ban contract).
//!   7. `BackgroundTaskManager::gated_spawn(autonomous_mode=false)`
//!      returns `Err(InteractiveModeRejected)` (stub_ban contract).
//!   8. `TemplateCrystallizer::check_crystallization` fires only after
//!      N=3 consecutive successes (Task 6).
//!   9. `CumulativeTokenTracker::recalibrate_budget` is expansion-only
//!      (DRIFT-E + decision (a) — never shrinks a measured ceiling).
//!  10. `classify_task_for_template` returns Some("Counter", _) for
//!      the smoke-test target prompt.
//!  11. `build_with_compiled_executor` is reachable and returns
//!      `Some(CompiledPipeline)` on cache HIT — production wiring path
//!      for the previously-dark `CompiledExecutor::try_execute` /
//!      `emit_plan_skeleton_template_marker` pair (closes WARN-2 + Q3
//!      in audit-C25.md).
//!  12. `apply_task_runner_wiring` forwards the 3 new C-25 Arcs (
//!      `crystallizer`, `compiled_pipeline_store`,
//!      `background_task_manager`) onto `TaskRunnerConfig`. Closes
//!      WARN-4 — production sites that previously passed None now have
//!      a live wiring path.
//!
//! No I/O outside the per-test TempDir; no sidecar; no Tauri runtime.

use std::sync::Arc;
use std::time::Duration;

use elai_core::OsClock;
use elai_orchestrator::background_task::{
    BackgroundTaskError, BackgroundTaskManager,
};
use elai_orchestrator::compiled_pipeline::{
    CompiledExecutor, CompiledExecutorError, CompiledPipelineStore, CompiledStep,
    ExecutionOutcome, TemplateCrystallizer,
};
use elai_orchestrator::context_budget::{
    CumulativeTokenTracker, ModelCapabilityProfile, ProviderUsage,
};
use elai_orchestrator::pipeline_builder::classify_task_for_template;
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, BackgroundTaskCompletePayload,
    BackgroundTaskStartedPayload, CompiledPipelineFiredPayload, DecompileTriggeredPayload,
    DecompileTriggerReason, PlanSkeletonPayload, PlanSkeletonSource, SsbActionType, SsbPayload,
    TemplateCrystallizedPayload, SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::{Ssb, SsbConfig};
use tempfile::TempDir;
use uuid::Uuid;

#[test]
fn schema_version_at_least_c25_bump() {
    // C-25 audit follow-up: bump from 56 to 57 for the 6th compiled-pipeline
    // beacon (DecompileTriggered). Closes WARN-1 in audit-C25.md.
    assert!(
        SCHEMA_VERSION_PUBLIC >= 57,
        "C-25 audit follow-up schema bump 56 -> 57 not present; got {}",
        SCHEMA_VERSION_PUBLIC
    );
}

#[test]
fn all_6_new_action_types_roundtrip() {
    let names = [
        ("plan_skeleton", SsbActionType::PlanSkeleton),
        ("compiled_pipeline_fired", SsbActionType::CompiledPipelineFired),
        ("template_crystallized", SsbActionType::TemplateCrystallized),
        ("background_task_started", SsbActionType::BackgroundTaskStarted),
        ("background_task_complete", SsbActionType::BackgroundTaskComplete),
        // C-25 audit follow-up: missing 6th beacon (WARN-1).
        ("decompile_triggered", SsbActionType::DecompileTriggered),
    ];
    for (s, expected) in names {
        let got = ssb_action_type_from_str_public(s).expect("from_str");
        assert_eq!(got, expected, "from_str roundtrip failed for {s}");
        assert_eq!(expected.as_sql_str(), s, "as_sql_str roundtrip failed for {s}");
    }
}

#[test]
fn decompile_triggered_payload_roundtrips_via_msgpack() {
    // C-25 audit follow-up: pin both trigger reasons + the failure_count
    // field shape so dashboard / contract consumers can rely on the wire
    // format. Closes WARN-1 in audit-C25.md.
    for reason in [
        DecompileTriggerReason::Confidence,
        DecompileTriggerReason::ConsecutiveFailures,
    ] {
        let p = DecompileTriggeredPayload {
            concept_class: "Counter".into(),
            plan_skeleton_hash: 0x1234,
            failure_count: 2,
            trigger_reason: reason,
        };
        let bytes = rmp_serde::to_vec_named(&p).expect("encode");
        let decoded: DecompileTriggeredPayload =
            rmp_serde::from_slice(&bytes).expect("decode");
        assert_eq!(decoded, p, "msgpack round-trip mismatch for decompile_triggered");
        // action_type() arm pinned.
        let ssb_payload = SsbPayload::DecompileTriggered(p);
        assert_eq!(
            ssb_payload.action_type(),
            SsbActionType::DecompileTriggered,
            "action_type arm must map DecompileTriggered payload to discriminant"
        );
    }
}

#[test]
fn plan_skeleton_payload_carries_source_field() {
    let llm = PlanSkeletonPayload {
        plan_id: "demo".into(),
        step_count: 4,
        concept_class: "Counter".into(),
        plan_skeleton_hash: 0x1234,
        source: PlanSkeletonSource::Llm,
    };
    let template = PlanSkeletonPayload {
        source: PlanSkeletonSource::Template,
        ..llm.clone()
    };
    // Round-trip via msgpack.
    for p in [llm, template] {
        let bytes = rmp_serde::to_vec_named(&p).expect("encode");
        let decoded: PlanSkeletonPayload =
            rmp_serde::from_slice(&bytes).expect("decode");
        assert_eq!(decoded, p, "msgpack round-trip mismatch for plan_skeleton");
    }
}

#[tokio::test]
async fn payload_action_type_arm_pinned_for_all_6() {
    let payloads = [
        SsbPayload::PlanSkeleton(PlanSkeletonPayload {
            plan_id: "x".into(),
            step_count: 0,
            concept_class: "Counter".into(),
            plan_skeleton_hash: 0,
            source: PlanSkeletonSource::Llm,
        }),
        SsbPayload::CompiledPipelineFired(CompiledPipelineFiredPayload {
            pipeline_id: Uuid::new_v4().to_string(),
            concept_class: "Counter".into(),
            plan_skeleton_hash: 0,
            use_count: 4,
        }),
        SsbPayload::TemplateCrystallized(TemplateCrystallizedPayload {
            concept_class: "Counter".into(),
            plan_skeleton_hash: 0,
            pipeline_id: Uuid::new_v4().to_string(),
            threshold_n: 3,
        }),
        SsbPayload::BackgroundTaskStarted(BackgroundTaskStartedPayload {
            task_id: "t1".into(),
            command_summary: "echo".into(),
            soft_timeout_secs: 30,
            hard_timeout_secs: 600,
        }),
        SsbPayload::BackgroundTaskComplete(BackgroundTaskCompletePayload {
            task_id: "t1".into(),
            exit_status: 0,
            duration_secs: 1,
            auto_detached: false,
            timed_out: false,
        }),
        // C-25 audit follow-up: 6th beacon (WARN-1).
        SsbPayload::DecompileTriggered(DecompileTriggeredPayload {
            concept_class: "Counter".into(),
            plan_skeleton_hash: 0,
            failure_count: 2,
            trigger_reason: DecompileTriggerReason::ConsecutiveFailures,
        }),
    ];
    let expected = [
        SsbActionType::PlanSkeleton,
        SsbActionType::CompiledPipelineFired,
        SsbActionType::TemplateCrystallized,
        SsbActionType::BackgroundTaskStarted,
        SsbActionType::BackgroundTaskComplete,
        SsbActionType::DecompileTriggered,
    ];
    for (p, exp) in payloads.iter().zip(expected.iter()) {
        assert_eq!(p.action_type(), *exp, "action_type arm mismatch");
    }
}

#[tokio::test]
async fn write_each_new_variant_via_ssb_succeeds() {
    let dir = TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: dir.path().join("ssb.db"),
        pool_size: None,
        project_id: "p-c25".into(),
        session_id: "s-c25".into(),
    };
    let ssb = Ssb::new(cfg, Arc::new(OsClock)).expect("ssb init");
    let payloads = [
        SsbPayload::PlanSkeleton(PlanSkeletonPayload {
            plan_id: "x".into(),
            step_count: 0,
            concept_class: "Counter".into(),
            plan_skeleton_hash: 0,
            source: PlanSkeletonSource::Template,
        }),
        SsbPayload::TemplateCrystallized(TemplateCrystallizedPayload {
            concept_class: "Counter".into(),
            plan_skeleton_hash: 0,
            pipeline_id: "pl".into(),
            threshold_n: 3,
        }),
    ];
    for p in payloads {
        let seq = ssb
            .write_payload("c25_contract", 0, p, Vec::new())
            .await
            .expect("write_payload");
        assert!(seq > 0, "seq must be positive");
    }
}

#[test]
fn store_load_corrupt_returns_empty_no_panic() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("compiled_pipelines.msgpack");
    std::fs::write(&path, b"corrupt bytes here").expect("seed corrupt");
    let mut store = CompiledPipelineStore::new(path);
    // MUST NOT panic.
    store.load_from_disk();
    assert!(store.entries().is_empty(), "stub_ban: corrupt -> empty");
}

#[tokio::test]
async fn compiled_executor_cold_cache_returns_typed_cachemiss() {
    let exec = CompiledExecutor::new(None);
    let r = exec.try_execute("Counter", 99).await;
    assert!(
        matches!(r, Err(CompiledExecutorError::CacheMiss(..))),
        "stub_ban: cache miss must return typed CacheMiss"
    );
}

#[tokio::test]
async fn gated_spawn_rejects_interactive_mode() {
    let mgr = BackgroundTaskManager::new(None);
    let handle = tokio::spawn(async { 0_i32 });
    let r = mgr
        .gated_spawn(
            false,
            "t1".into(),
            "echo".into(),
            Duration::from_secs(10),
            Duration::from_secs(60),
            handle,
        )
        .await;
    assert!(
        matches!(r, Err(BackgroundTaskError::InteractiveModeRejected)),
        "stub_ban: gated_spawn must reject when autonomous_mode=false"
    );
}

#[tokio::test]
async fn crystallizer_fires_only_on_n_consecutive_successes() {
    let mut c = TemplateCrystallizer::new(None);
    let concept = "Counter";
    let hash: u64 = 42;
    for _ in 0..2 {
        c.record_execution(ExecutionOutcome {
            success: true,
            concept_class: concept.into(),
            plan_skeleton_hash: hash,
            pipeline_id: None,
            token_count: 0,
            timestamp: 0,
        });
    }
    let r = c
        .check_crystallization(
            concept,
            hash,
            || {
                vec![CompiledStep {
                    step_index: 0,
                    template_prompt: "p".into(),
                    variables: vec![],
                    variable_region_ratio: 0.0,
                    l1_tool_only: true,
                }]
            },
            10,
        )
        .await;
    assert!(r.is_none(), "must not fire below threshold");
    c.record_execution(ExecutionOutcome {
        success: true,
        concept_class: concept.into(),
        plan_skeleton_hash: hash,
        pipeline_id: None,
        token_count: 0,
        timestamp: 0,
    });
    let r2 = c
        .check_crystallization(
            concept,
            hash,
            || {
                vec![CompiledStep {
                    step_index: 0,
                    template_prompt: "p".into(),
                    variables: vec![],
                    variable_region_ratio: 0.0,
                    l1_tool_only: true,
                }]
            },
            10,
        )
        .await;
    assert!(r2.is_some(), "must fire at N=3 consecutive successes");
}

#[test]
fn recalibrate_budget_is_expansion_only() {
    // DRIFT-E + decision (a): never SHRINK an existing measured ceiling.
    let mut profile = ModelCapabilityProfile::cloud_200k();
    let before = profile.attention_ceiling;
    let mut t = CumulativeTokenTracker::new();
    // Tiny prompt — naive max(4096, ...) would shrink cloud_200k.
    t.record_response(&ProviderUsage::from_u32(50, 10, 0, 0));
    t.recalibrate_budget(&mut profile);
    assert_eq!(
        profile.attention_ceiling, before,
        "recalibrate_budget must NOT shrink a measured ceiling (DRIFT-E)"
    );
}

#[test]
fn classify_task_for_template_returns_counter_for_smoke_test_prompt() {
    let prompt = "implement a 2-method counter struct";
    let r = classify_task_for_template(prompt);
    let (concept, _hash) = r.expect("smoke-test prompt must classify");
    assert_eq!(
        concept, "Counter",
        "classifier must label the C-25 smoke-test target as Counter"
    );
}

// ---------------------------------------------------------------------------
// C-25 audit follow-up — Contract 11: build_with_compiled_executor is
// reachable and returns the cached pipeline on cache HIT. Closes WARN-2 +
// Q3 in workflow/sessions/s-25672/audit-C25.md (try_execute +
// emit_plan_skeleton_template_marker had zero non-test callers).
// ---------------------------------------------------------------------------

mod build_with_compiled_executor_test_doubles {
    use elai_ipc_types::ToolId;
    use elai_orchestrator::agent_pool::ModelPool;
    use elai_orchestrator::pipeline_builder::{
        PredicateCompiler, PredicateRef, RuleId, ToolLookup,
    };

    pub struct PermissiveTools;
    impl ToolLookup for PermissiveTools {
        fn has_tool(&self, _tool_id: &ToolId) -> bool {
            true
        }
        fn find_substitute(&self, _tool_id: &ToolId) -> Option<ToolId> {
            None
        }
    }

    pub struct UnlimitedPool;
    impl ModelPool for UnlimitedPool {
        fn available_vram(&self) -> u64 {
            u64::MAX
        }
    }

    pub struct PassThroughPreds;
    impl PredicateCompiler for PassThroughPreds {
        fn compile(&self, raw: &str) -> Result<PredicateRef, String> {
            Ok(PredicateRef {
                raw: raw.to_string(),
                valid: true,
            })
        }
        fn active_rules_for_phase(&self, _phase_name: &str) -> Vec<RuleId> {
            Vec::new()
        }
    }
}

#[tokio::test]
async fn build_with_compiled_executor_returns_hit_on_cache_match() {
    use build_with_compiled_executor_test_doubles::{
        PassThroughPreds, PermissiveTools, UnlimitedPool,
    };
    use elai_orchestrator::compiled_pipeline::{
        CompiledExecutor, CompiledPipeline, CompiledPipelineStore, CompiledStep,
    };
    use elai_orchestrator::research_tree::{OutputType, PerfSpec, PipelineSpec};
    use elai_orchestrator::PipelineBuilder;
    use std::time::Duration;
    use tokio::sync::Mutex as AsyncMutex;

    // spec.name embeds the "counter" keyword so classify_task_for_template
    // returns Some(("Counter", hash)).
    let spec = PipelineSpec {
        name: "implement a 2-method counter struct".into(),
        steps: vec!["step_0".into()],
        output_type: OutputType::Code,
        perf: PerfSpec {
            max_latency: Duration::from_secs(60),
            max_memory_bytes: 1024 * 1024 * 1024,
            max_tokens: 8192,
        },
        media: Vec::new(),
    };

    // Pre-seed the store with a pipeline keyed under ("Counter", hash) so
    // the executor probe sees a HIT.
    let (concept_class, plan_skeleton_hash) =
        classify_task_for_template(&spec.name).expect("classify");
    let pipeline = CompiledPipeline {
        pipeline_id: Uuid::new_v4(),
        concept_class: concept_class.clone(),
        plan_skeleton_hash,
        steps: vec![CompiledStep {
            step_index: 0,
            template_prompt: "p".into(),
            variables: vec![],
            variable_region_ratio: 0.0,
            l1_tool_only: true,
        }],
        created_at: 0,
        last_used_at: 0,
        use_count: 5,
        first_try_success_rate: 0.95,
        compilation_eligible: true,
        flow_fingerprint: None,
    };
    let dir = TempDir::new().expect("tempdir");
    let store_path = dir.path().join("compiled.msgpack");
    let mut store = CompiledPipelineStore::new(store_path);
    store.upsert(pipeline.clone(), 0);
    let store_arc = Arc::new(AsyncMutex::new(store));

    let executor = CompiledExecutor::new(None);
    let (_, hit) = PipelineBuilder::build_with_compiled_executor(
        &spec,
        &PermissiveTools,
        &UnlimitedPool,
        &PassThroughPreds,
        None,
        &executor,
        &store_arc,
    )
    .await
    .expect("build_with_compiled_executor");

    let got = hit.expect(
        "production wiring: build_with_compiled_executor must return \
         Some(CompiledPipeline) on cache HIT (closes WARN-2 + Q3 in audit-C25.md)",
    );
    assert_eq!(got.pipeline_id, pipeline.pipeline_id);
    assert_eq!(got.concept_class, "Counter");
}

#[tokio::test]
async fn build_with_compiled_executor_returns_none_on_cache_miss() {
    use build_with_compiled_executor_test_doubles::{
        PassThroughPreds, PermissiveTools, UnlimitedPool,
    };
    use elai_orchestrator::compiled_pipeline::{CompiledExecutor, CompiledPipelineStore};
    use elai_orchestrator::research_tree::{OutputType, PerfSpec, PipelineSpec};
    use elai_orchestrator::PipelineBuilder;
    use std::time::Duration;
    use tokio::sync::Mutex as AsyncMutex;

    let spec = PipelineSpec {
        name: "implement a 2-method counter struct".into(),
        steps: vec!["step_0".into()],
        output_type: OutputType::Code,
        perf: PerfSpec {
            max_latency: Duration::from_secs(60),
            max_memory_bytes: 1024 * 1024 * 1024,
            max_tokens: 8192,
        },
        media: Vec::new(),
    };

    let dir = TempDir::new().expect("tempdir");
    let store_path = dir.path().join("compiled.msgpack");
    let store_arc = Arc::new(AsyncMutex::new(CompiledPipelineStore::new(store_path)));

    let executor = CompiledExecutor::new(None);
    let (_, hit) = PipelineBuilder::build_with_compiled_executor(
        &spec,
        &PermissiveTools,
        &UnlimitedPool,
        &PassThroughPreds,
        None,
        &executor,
        &store_arc,
    )
    .await
    .expect("build_with_compiled_executor");

    assert!(
        hit.is_none(),
        "cache miss must return None — the LLM-path ExecutablePipeline is \
         still returned for the runner, but the CompiledPipeline slot is empty"
    );
}

// ---------------------------------------------------------------------------
// C-25 audit follow-up — Contract 12: apply_task_runner_wiring forwards the
// 3 new C-25 Arcs onto TaskRunnerConfig. Closes WARN-4 in audit-C25.md.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn apply_task_runner_wiring_forwards_c25_arcs() {
    use elai_orchestrator::{apply_task_runner_wiring, build_task_runner_wiring};

    let tmp = TempDir::new().expect("tempdir");
    let project_root = tmp.path().to_path_buf();
    let session_id = "c25-audit-followup";

    let wiring = build_task_runner_wiring(&project_root, session_id)
        .await
        .expect("build_task_runner_wiring");

    // build_task_runner_wiring MUST construct the 3 C-25 Arcs by default.
    assert!(
        wiring.crystallizer.is_some(),
        "build_task_runner_wiring must construct crystallizer (WARN-4)"
    );
    assert!(
        wiring.compiled_pipeline_store.is_some(),
        "build_task_runner_wiring must construct compiled_pipeline_store (WARN-4)"
    );
    assert!(
        wiring.background_task_manager.is_some(),
        "build_task_runner_wiring must construct background_task_manager (WARN-4)"
    );

    let cursor_path = project_root.join(".elai_cc").join("cursor.json");
    let cfg = apply_task_runner_wiring(cursor_path, &wiring);

    assert!(
        cfg.crystallizer.is_some(),
        "apply_task_runner_wiring must forward crystallizer onto TaskRunnerConfig (WARN-4)"
    );
    assert!(
        cfg.compiled_pipeline_store.is_some(),
        "apply_task_runner_wiring must forward compiled_pipeline_store onto TaskRunnerConfig (WARN-4)"
    );
    assert!(
        cfg.background_task_manager.is_some(),
        "apply_task_runner_wiring must forward background_task_manager onto TaskRunnerConfig (WARN-4)"
    );
}
