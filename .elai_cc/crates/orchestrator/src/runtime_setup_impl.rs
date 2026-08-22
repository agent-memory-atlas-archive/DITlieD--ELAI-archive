use std::path::{Path, PathBuf};
use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use rusqlite::Connection;
use tokio::sync::Mutex as AsyncMutex;

use super::types::{OptionalWiringServices, RuntimeWiring};
use crate::{
    agent_pool::{AgentPoolConfig, InProcessAgentPool},
    autonomous_gate::{AutonomousGate, AutonomousGateConfig, GateMode},
    background_task::BackgroundTaskManager,
    cloud_mode::{CloudConfig, CloudModeRouter, RedactingBabyStepExecutor},
    compiled_pipeline::{CompiledPipelineStore, TemplateCrystallizer},
    dispatcher::{BackendConfig, BackendMode},
    gate::InteractiveGate,
    grc::{pipeline_grc_disabled_via_env, PipelineGrc},
    hook_dispatcher::{HookDispatcher, InProcessHookDispatcher},
    ipc_accept::AgentRpcServer,
    ipc_gateway::IpcGateway,
    run_profile::RunProfile,
    skill_trust::SkillStore,
    task_runner::{AgentPoolBabyStepExecutor, BabyStepExecutor},
    DefaultFeasibilityProber, DefaultLayer1aProcessor, FeasibilityProber, Layer1aProcessor, Ssb,
    SsbConfig, TaskRunnerConfig, SSB_DB_FILE_NAME,
};

#[cfg(windows)]
const AGENT_BINARY_NAME: &str = "elai_cc_agent.exe";
#[cfg(not(windows))]
const AGENT_BINARY_NAME: &str = "elai_cc_agent";
const ELAI_AGENT_BIN_ENV: &str = "ELAI_AGENT_BIN";

pub async fn build_task_runner_wiring(
    project_root: &Path,
    session_id: &str,
) -> Result<RuntimeWiring, anyhow::Error> {
    let (data_dir, ssb) = initialize_runtime_storage(project_root, session_id)?;
    let feasibility_prober = build_feasibility_prober(project_root, &data_dir);
    let (rules_db_path, rules_conn, vibegate_seed_ok) = open_and_seed_rules(project_root)?;
    let layer1a_processor = build_layer1a_processor(rules_conn, rules_db_path, ssb.clone()).await?;
    let services = build_optional_wiring_services(project_root, ssb.clone());
    let adversary_runtime = build_adversary_runtime();
    Ok(assemble_runtime_wiring(
        ssb,
        feasibility_prober,
        layer1a_processor,
        services,
        vibegate_seed_ok,
        adversary_runtime,
    ))
}

fn initialize_runtime_storage(
    project_root: &Path,
    session_id: &str,
) -> Result<(PathBuf, Arc<Ssb>), anyhow::Error> {
    let data_dir = project_root.join(elai_core::paths::ELAI_DATA_DIR_NAME);
    std::fs::create_dir_all(&data_dir).map_err(|e| {
        anyhow::anyhow!("build_task_runner_wiring: create {} failed: {e}", data_dir.display())
    })?;
    let sessions_dir = elai_core::paths::sessions_dir_path(project_root);
    std::fs::create_dir_all(&sessions_dir).map_err(|e| {
        anyhow::anyhow!(
            "build_task_runner_wiring: create {} failed: {e}",
            sessions_dir.display()
        )
    })?;
    let ssb = Arc::new(Ssb::new(
        SsbConfig {
            db_path: sessions_dir.join(SSB_DB_FILE_NAME),
            pool_size: Some(2),
            project_id: elai_core::project_id_from_path(project_root),
            session_id: session_id.to_string(),
        },
        Arc::new(OsClock),
    )
    .map_err(|e| anyhow::anyhow!("build_task_runner_wiring: SSB init failed: {e}"))?);
    #[cfg(feature = "mcp")]
    crate::mcp_rugpull_gate::install_ssb_writer(Arc::clone(&ssb));
    crate::lock::install_ssb_writer(Arc::clone(&ssb));
    crate::startup::install_ssb_writer(Arc::clone(&ssb));
    Ok((data_dir, ssb))
}

fn build_feasibility_prober(project_root: &Path, data_dir: &Path) -> Arc<DefaultFeasibilityProber> {
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let system_prober: Arc<dyn crate::system_probe::SystemProber> = Arc::new(
        crate::system_probe::DefaultSystemProber::new(Some(project_root.to_path_buf()), clock),
    );
    let mut prober = DefaultFeasibilityProber::new(data_dir.join("feasibility"))
        .with_system_prober(system_prober);
    match locate_default_agent_binary() {
        Ok(agent_path) => {
            let tier3 = crate::feasibility_probe::Tier3StubExecutor::new(agent_path);
            prober = prober.with_stub_executor(Arc::new(tier3));
        }
        Err(error) => tracing::info!(
            error = %error,
            "build_task_runner_wiring: elai_cc_agent not located — feasibility prober keeps the synthetic stub path"
        ),
    }
    Arc::new(prober)
}

fn open_and_seed_rules(
    project_root: &Path,
) -> Result<(PathBuf, Connection, bool), anyhow::Error> {
    let rules_db_path = elai_core::paths::rules_db_path(project_root);
    let clock = OsClock;
    let rules_conn = elai_rules::schema::open_rules_db(&rules_db_path, &clock).map_err(|e| {
        anyhow::anyhow!(
            "build_task_runner_wiring: open rules.db at {} failed: {e}",
            rules_db_path.display()
        )
    })?;
    seed_security_rules(&rules_conn, &clock);
    seed_deflation_rules(&rules_conn, &clock);
    seed_holdout_budget_rules(&rules_conn, &clock);
    seed_drift_interlock_rules(&rules_conn, &clock);
    seed_leakage_rules(&rules_conn, &clock);
    seed_trading_nulls_rules(&rules_conn, &clock);
    seed_repro_rules(&rules_conn, &clock);
    seed_sim_real_parity_rules(&rules_conn, &clock);
    seed_result_manifest_rules(&rules_conn, &clock);
    seed_financial_invariant_rules(&rules_conn, &clock);
    let vibegate_seed_ok = crate::vibegate::check_vibegate_seed_integrity(&rules_conn);
    if !vibegate_seed_ok {
        tracing::error!(
            "build_task_runner_wiring: vibesec seed integrity check FAILED; VibeGate will block all code-vuln writes fail-closed"
        );
    }
    Ok((rules_db_path, rules_conn, vibegate_seed_ok))
}

macro_rules! seed_rule {
    ($name:ident, $method:ident, $success:literal, $failure:literal) => {
        fn $name(conn: &Connection, clock: &OsClock) {
            match elai_rules::project_rule_seeder::ProjectRuleSeeder::$method(conn, clock) {
                Ok(inserted) => tracing::info!(inserted, $success),
                Err(error) => tracing::error!(error = %error, $failure),
            }
        }
    };
}

seed_rule!(seed_security_rules, seed_security_rules, "build_task_runner_wiring: F-2a security seed rules ensured (mutable=0)", "build_task_runner_wiring: seeding F-2a security rules failed; VibeGate seed integrity will fail-closed");
seed_rule!(seed_deflation_rules, seed_deflation_rules, "build_task_runner_wiring: H-18 deflation gate seed rule ensured (FIN-DEFL-001, mutable=0)", "build_task_runner_wiring: seeding H-18 deflation gate rule failed; FIN-DEFL-001 promotion gate will not be DB-backed");
seed_rule!(seed_holdout_budget_rules, seed_holdout_budget_rules, "build_task_runner_wiring: H-19 holdout budget gate seed rule ensured (FIN-HOLDOUT-001, mutable=0)", "build_task_runner_wiring: seeding H-19 holdout budget gate failed; FIN-HOLDOUT-001 query-budget gate will not be DB-backed");
seed_rule!(seed_drift_interlock_rules, seed_drift_interlock_rules, "build_task_runner_wiring: H-29 drift interlock gate seed rules ensured (FIN-DRIFT-001/002, mutable=0)", "build_task_runner_wiring: seeding H-29 drift interlock gate rules failed; FIN-DRIFT-* interlock rows will not be DB-backed");
seed_rule!(seed_leakage_rules, seed_leakage_rules, "build_task_runner_wiring: H-20 leakage gate seed rule ensured (FIN-LEAK-001, mutable=0)", "build_task_runner_wiring: seeding H-20 leakage gate failed; FIN-LEAK-001 leakage gate will not be DB-backed");
seed_rule!(seed_trading_nulls_rules, seed_trading_nulls_rules, "build_task_runner_wiring: H-22 trading null-test gate seed rule ensured (FIN-NULL-001, mutable=0)", "build_task_runner_wiring: seeding H-22 trading null-test gate failed; FIN-NULL-001 null-test gate will not be DB-backed");
seed_rule!(seed_repro_rules, seed_repro_rules, "build_task_runner_wiring: H-23 reproducibility seed rules ensured (REPRO-SEED-001 + REPRO-ENV-001, mutable=0)", "build_task_runner_wiring: seeding H-23 reproducibility rules failed; REPRO-SEED-001 / REPRO-ENV-001 will not be DB-backed");
seed_rule!(seed_sim_real_parity_rules, seed_sim_real_parity_rules, "build_task_runner_wiring: H-24 sim-to-real parity seed rules ensured (FIN-SIMREAL-001 + FIN-SIMREAL-002, mutable=0)", "build_task_runner_wiring: seeding H-24 sim-to-real parity rules failed; FIN-SIMREAL-001 / FIN-SIMREAL-002 will not be DB-backed");
seed_rule!(seed_result_manifest_rules, seed_result_manifest_rules, "build_task_runner_wiring: H-25 result-manifest re-derivation seed rules ensured (FIN-RESMANIFEST-001 + FIN-RESMANIFEST-002, mutable=0)", "build_task_runner_wiring: seeding H-25 result-manifest rules failed; FIN-RESMANIFEST-001 / FIN-RESMANIFEST-002 will not be DB-backed");
seed_rule!(seed_financial_invariant_rules, seed_financial_invariant_rules, "build_task_runner_wiring: H-26 financial-invariant seed rule ensured (FIN-INVARIANT-001 naked-float-== ban, mutable=0)", "build_task_runner_wiring: seeding H-26 financial-invariant rules failed; FIN-INVARIANT-001 will not be DB-backed");

async fn build_layer1a_processor(
    rules_conn: Connection,
    rules_db_path: PathBuf,
    ssb: Arc<Ssb>,
) -> Result<Arc<DefaultLayer1aProcessor>, anyhow::Error> {
    let layer1a = DefaultLayer1aProcessor::new(
        Arc::new(AsyncMutex::new(rules_conn)),
        rules_db_path,
        ssb,
    );
    layer1a.ensure_tables().await.map_err(|e| {
        anyhow::anyhow!("build_task_runner_wiring: Layer1A ensure_tables failed: {e}")
    })?;
    Ok(Arc::new(layer1a))
}

fn build_optional_wiring_services(project_root: &Path, ssb: Arc<Ssb>) -> OptionalWiringServices {
    let crystallizer = Arc::new(AsyncMutex::new(TemplateCrystallizer::new(Some(ssb.clone()))));
    let mut store = CompiledPipelineStore::new(CompiledPipelineStore::standard_path(project_root));
    store.load_from_disk();
    let background_task_manager = Arc::new(BackgroundTaskManager::new(Some(ssb.clone())));
    let skill_store = Arc::new(AsyncMutex::new(SkillStore::new()));
    let pipeline_grc = if pipeline_grc_disabled_via_env() {
        tracing::info!(
            "build_task_runner_wiring: ELAI_DISABLE_PIPELINE_GRC set — \
             PipelineGrc wiring skipped, all 4 safety gates remain no-ops"
        );
        None
    } else {
        Some(Arc::new(AsyncMutex::new(PipelineGrc::new_from_default())))
    };
    OptionalWiringServices {
        crystallizer,
        compiled_pipeline_store: Arc::new(AsyncMutex::new(store)),
        background_task_manager,
        skill_store,
        pipeline_grc,
    }
}

fn build_adversary_runtime() -> Option<Arc<crate::runtime_config::AdversaryRuntimeState>> {
    let enabled = std::env::var("ELAI_ALLOW_CLOUD_ADVERSARY")
        .map(|value| matches!(value.as_str(), "1" | "true" | "yes"))
        .unwrap_or(false);
    if !enabled {
        return None;
    }
    let backend = crate::per_role_runtime::PerRoleRuntime::default()
        .get(crate::agent_pool::AgentRole::Executor);
    let worker_family = crate::adversary::ModelFamily::from_model_id(backend.canonical_name());
    let families = available_adversary_families();
    match crate::adversary::route_cross_family(&worker_family, &families, true) {
        Ok(_) => build_cloud_adversary_runtime(),
        Err(_) => {
            tracing::info!(
                "build_task_runner_wiring: route_cross_family returned SameFamilyPairing; \
                 adversary_runtime stays None (single-family pool)"
            );
            None
        }
    }
}

fn available_adversary_families() -> Vec<crate::adversary::ModelFamily> {
    let mut labels = Vec::new();
    let mut families = Vec::new();
    for (_role, backend) in crate::per_role_runtime::PerRoleRuntime::default().list() {
        let family = crate::adversary::ModelFamily::from_model_id(backend.canonical_name());
        let label = family.as_str().to_owned();
        if !labels.iter().any(|seen| seen == &label) {
            labels.push(label);
            families.push(family);
        }
    }
    families
}

fn build_cloud_adversary_runtime() -> Option<Arc<crate::runtime_config::AdversaryRuntimeState>> {
    match crate::adversary::CloudAdversary::from_env("claude-sonnet-4-6") {
        Ok(cloud_adv) => Some(crate::runtime_config::AdversaryRuntimeState::new(
            Arc::new(crate::adversary::AdversaryErased::new(cloud_adv)),
            crate::adversary::AdversaryConfig::default_for_local_worker(),
            crate::runtime_config::AdversaryRuntimeConfig::default(),
        )),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "build_task_runner_wiring: CloudAdversary::from_env failed; adversary_runtime stays None (set ANTHROPIC_API_KEY to enable)"
            );
            None
        }
    }
}

fn assemble_runtime_wiring(
    ssb: Arc<Ssb>,
    feasibility_prober: Arc<DefaultFeasibilityProber>,
    layer1a_processor: Arc<DefaultLayer1aProcessor>,
    services: OptionalWiringServices,
    vibegate_seed_ok: bool,
    adversary_runtime: Option<Arc<crate::runtime_config::AdversaryRuntimeState>>,
) -> RuntimeWiring {
    RuntimeWiring {
        ssb,
        feasibility_prober,
        layer1a_processor,
        baby_step_executor: None,
        gate_mode: None,
        run_profile: None,
        interactive_gate: None,
        cloud_config: None,
        provider_override: None,
        hook_dispatcher: None,
        crystallizer: Some(services.crystallizer),
        compiled_pipeline_store: Some(services.compiled_pipeline_store),
        background_task_manager: Some(services.background_task_manager),
        skill_store: Some(services.skill_store),
        via_mcp: false,
        mcp_dispatch: None,
        buffer_editor: None,
        pipeline_grc: services.pipeline_grc,
        vibegate_seed_ok,
        reflection_client: build_reflection_client(),
        adversary_runtime,
        #[cfg(feature = "empire")]
        empire_bridge: None,
    }
}

fn build_reflection_client() -> Option<Arc<dyn crate::DynReflectionClient>> {
    (!crate::micro_reflection_disabled_via_env()).then(|| {
        let adapter = crate::SidecarReflectionAdapter::new(Arc::new(
            crate::SidecarGroundingClient::default(),
        ));
        Arc::new(adapter) as Arc<dyn crate::DynReflectionClient>
    })
}

pub fn apply_task_runner_wiring(cursor_path: PathBuf, wiring: &RuntimeWiring) -> TaskRunnerConfig {
    let mut cfg = TaskRunnerConfig::new_minimal(cursor_path);
    install_required_processors(&mut cfg, wiring);
    install_gate(&mut cfg, wiring);
    let (resolved_executor, backend_mode) = install_cloud_routing(&mut cfg, wiring);
    cfg.baby_step_executor = refuse_executor_for_critique(wiring, resolved_executor);
    cfg.backend_config = Some(backend_config(wiring, backend_mode));
    install_hook_dispatcher(&mut cfg, wiring);
    install_optional_services(&mut cfg, wiring);
    install_dispatch(&mut cfg, wiring);
    install_remaining_services(&mut cfg, wiring);
    cfg
}

fn install_required_processors(cfg: &mut TaskRunnerConfig, wiring: &RuntimeWiring) {
    let prober: Arc<dyn FeasibilityProber> = wiring.feasibility_prober.clone();
    let processor: Arc<dyn Layer1aProcessor> = wiring.layer1a_processor.clone();
    cfg.feasibility_prober = Some(prober);
    cfg.layer1a_processor = Some(processor);
}

fn install_remaining_services(cfg: &mut TaskRunnerConfig, wiring: &RuntimeWiring) {
    cfg.pipeline_grc = wiring.pipeline_grc.clone();
    cfg.plan_interactive_gate = wiring.interactive_gate.clone();
    cfg.reflection_client = wiring.reflection_client.clone();
    cfg.adversary_runtime = wiring.adversary_runtime.clone();
    #[cfg(feature = "empire")]
    {
        cfg.empire_bridge = wiring.empire_bridge.clone();
    }
}

fn install_gate(cfg: &mut TaskRunnerConfig, wiring: &RuntimeWiring) {
    let Some(mode) = wiring.gate_mode else {
        return;
    };
    let config = AutonomousGateConfig {
        mode,
        ..AutonomousGateConfig::default()
    };
    cfg.autonomous_gate = Some(Arc::new(AutonomousGate::new(
        config,
        wiring.interactive_gate.clone(),
    )));
}

fn install_cloud_routing(
    cfg: &mut TaskRunnerConfig,
    wiring: &RuntimeWiring,
) -> (Option<Arc<dyn BabyStepExecutor>>, BackendMode) {
    let Some(cloud) = wiring.cloud_config.as_ref() else {
        return (wiring.baby_step_executor.clone(), BackendMode::default());
    };
    let mode = resolve_backend_mode(wiring.provider_override.as_deref(), cloud.default_mode);
    let router = Arc::new(CloudModeRouter::new(cloud.clone()));
    cfg.cloud_spend_cap_daily_usd = cloud.spend_cap_daily_usd;
    let wraps_redactor = matches!(mode, BackendMode::Api | BackendMode::Hybrid);
    let executor = match (&wiring.baby_step_executor, wraps_redactor) {
        (Some(inner), true) => Some(Arc::new(RedactingBabyStepExecutor::with_ssb(
            Arc::clone(inner),
            router.privacy_config(),
            wiring.ssb.clone(),
        )) as Arc<dyn BabyStepExecutor>),
        (inner, _) => inner.clone(),
    };
    cfg.cloud_router = Some(router);
    (executor, mode)
}

fn refuse_executor_for_critique(
    wiring: &RuntimeWiring,
    executor: Option<Arc<dyn BabyStepExecutor>>,
) -> Option<Arc<dyn BabyStepExecutor>> {
    match wiring.run_profile {
        Some(profile) if profile.refuses_executor_spawn() => {
            tracing::info!(
                run_profile = %profile,
                "I-42 critique-only: executor/fixer spawn refused at install chokepoint \
                 (baby_step_executor forced to None)"
            );
            None
        }
        _ => executor,
    }
}

fn backend_config(wiring: &RuntimeWiring, mode: BackendMode) -> BackendConfig {
    let mut config = BackendConfig::default();
    config.mode = mode;
    if let Some(cloud) = wiring.cloud_config.as_ref() {
        config.model = cloud.default_model.clone();
    }
    if mode == BackendMode::Local {
        config.max_context = Some(crate::dispatcher::SIDECAR_MODEL_MAX_CONTEXT);
    }
    config
}

fn install_hook_dispatcher(cfg: &mut TaskRunnerConfig, wiring: &RuntimeWiring) {
    if let Some(dispatcher) = wiring.hook_dispatcher.as_ref() {
        let trait_obj: Arc<dyn crate::hook_dispatcher::HookDispatcher> = dispatcher.clone();
        cfg.hook_dispatcher = Some(trait_obj);
    }
}

fn install_optional_services(cfg: &mut TaskRunnerConfig, wiring: &RuntimeWiring) {
    cfg.crystallizer = wiring.crystallizer.clone();
    cfg.compiled_pipeline_store = wiring.compiled_pipeline_store.clone();
    cfg.background_task_manager = wiring.background_task_manager.clone();
    cfg.skill_store = wiring.skill_store.clone();
    cfg.buffer_editor = wiring.buffer_editor.clone();
}

fn install_dispatch(cfg: &mut TaskRunnerConfig, wiring: &RuntimeWiring) {
    if let Some(dispatch) = wiring.mcp_dispatch.as_ref() {
        cfg.mcp_dispatch = Some(dispatch.clone());
    } else if wiring.via_mcp {
        let mcp = crate::dispatch::McpClientDispatch::new(8200, wiring.ssb.clone());
        #[cfg(feature = "mcp")]
        let mcp = mcp.with_capability_flow_adapter(Arc::new(
            crate::capability_flow::CapabilityFlowAdapter::from_slot(
                crate::capability_flow::committed_slot(),
            ),
        ));
        cfg.mcp_dispatch = Some(Arc::new(mcp));
        tracing::info!("C-39: --via-mcp default dispatch wired (McpClientDispatch port 8200)");
    }
}

fn resolve_backend_mode(cli_override: Option<&str>, toml_default: crate::cloud_mode::ExecMode) -> BackendMode {
    if let Some(raw) = cli_override {
        return match raw.to_ascii_lowercase().as_str() {
            "local" => BackendMode::Local,
            "cloud" => BackendMode::Api,
            "hybrid" => BackendMode::Hybrid,
            _ => BackendMode::default(),
        };
    }
    match toml_default {
        crate::cloud_mode::ExecMode::Local => BackendMode::Local,
        crate::cloud_mode::ExecMode::Cloud => BackendMode::Api,
        crate::cloud_mode::ExecMode::Auto => BackendMode::Hybrid,
    }
}

pub fn build_default_baby_step_executor<E>(
    entropy: E,
    agent_binary_path: PathBuf,
    socket_dir: PathBuf,
    gateway: Arc<tokio::sync::Mutex<IpcGateway>>,
    rpc_server: Arc<dyn AgentRpcServer + Send + Sync + 'static>,
) -> Arc<dyn BabyStepExecutor>
where
    E: elai_core::EntropySource + Send + 'static,
{
    let pool = Arc::new(tokio::sync::Mutex::new(InProcessAgentPool::new(
        AgentPoolConfig::default(), entropy, crate::task_runner::runtime_model_pool(),
    )));
    Arc::new(AgentPoolBabyStepExecutor::new(pool, agent_binary_path, socket_dir, gateway, rpc_server))
}

#[must_use]
pub fn build_default_baby_step_executor_with_ipc_hooks<E>(
    entropy: E,
    agent_binary_path: PathBuf,
    socket_dir: PathBuf,
    gateway: Arc<tokio::sync::Mutex<IpcGateway>>,
    rpc_server: Arc<dyn AgentRpcServer + Send + Sync + 'static>,
    tiered: elai_plugins::tiered_analysis::TieredAnalysis,
    ssb: Arc<Ssb>,
) -> Arc<dyn BabyStepExecutor>
where
    E: elai_core::EntropySource + Send + 'static,
{
    let pool = Arc::new(tokio::sync::Mutex::new(InProcessAgentPool::new(
        AgentPoolConfig::default(), entropy, crate::task_runner::runtime_model_pool(),
    )));
    Arc::new(AgentPoolBabyStepExecutor::new(pool, agent_binary_path, socket_dir, gateway, rpc_server)
        .with_hook_dispatcher(build_production_ipc_hook_dispatcher(tiered, ssb)))
}

#[must_use]
pub fn build_stub_tiered_analysis() -> elai_plugins::tiered_analysis::TieredAnalysis {
    use elai_rules::rule_engine::{DefaultRuleEngine, FsmPhaseBit};
    use elai_rules::stubs::{EmptyDkgQuerier, PassAstEvaluator};
    let engine = Box::new(DefaultRuleEngine::new(Box::new(PassAstEvaluator), Box::new(EmptyDkgQuerier)));
    elai_plugins::tiered_analysis::TieredAnalysis::new(engine, FsmPhaseBit::EXECUTING)
}

#[must_use]
pub fn build_default_baby_step_executor_with_sandbox<E>(
    entropy: E,
    agent_binary_path: PathBuf,
    socket_dir: PathBuf,
    gateway: Arc<tokio::sync::Mutex<IpcGateway>>,
    rpc_server: Arc<dyn AgentRpcServer + Send + Sync + 'static>,
    ssb: Arc<Ssb>,
    snapshot_root: PathBuf,
) -> Arc<dyn BabyStepExecutor>
where
    E: elai_core::EntropySource + Send + 'static,
{
    let pool = Arc::new(tokio::sync::Mutex::new(InProcessAgentPool::new(
        AgentPoolConfig::default(), entropy, crate::task_runner::runtime_model_pool(),
    )));
    let executor = AgentPoolBabyStepExecutor::new(pool, agent_binary_path, socket_dir, gateway, rpc_server);
    if crate::runtime_config::sandbox_firecracker_enabled() {
        let fc_pool = Arc::new(crate::sandbox::FirecrackerHotPool::with_defaults(snapshot_root.join("snapshots")));
        Arc::new(executor.with_firecracker_pool(fc_pool).with_ssb_writer(ssb))
    } else {
        Arc::new(executor)
    }
}

#[must_use]
pub fn build_production_hook_dispatcher(
    ssb: Arc<Ssb>,
    vibegate_seed_ok: bool,
    project_root: &Path,
    stop_gate: Option<Arc<dyn InteractiveGate>>,
) -> Arc<InProcessHookDispatcher> {
    use crate::vibesec::dep_writetime_gate::{DepWriteTimeGate, SubprocessRegistryChecker};
    let registry_checker: Arc<dyn crate::vibesec::dep_writetime_gate::RegistryExistenceChecker + Send + Sync> = Arc::new(SubprocessRegistryChecker::default_at_root(project_root));
    let supervisor: Arc<dyn crate::directive::Supervisor> = Arc::new(
        crate::directive::InProcessSupervisor::new(project_root)
            .with_ssb(ssb.clone(), crate::task_runner::RUNNER_AGENT_ID, crate::task_runner::RUNNER_SUPERSTEP)
            .with_role_contracts(crate::role_contracts::RoleContractStore::default())
            .with_sequence_rules(Arc::new(crate::tool_sequence::SequenceRuleSet::load_builtin())),
    );
    let mut dispatcher = InProcessHookDispatcher::new(None)
        .with_ruff_runner(Arc::new(elai_plugins::post_tool_use::RuffPostToolUseRunner::new()))
        .with_vibegate_ssb(ssb.clone(), crate::task_runner::RUNNER_AGENT_ID, crate::task_runner::RUNNER_SUPERSTEP)
        .with_vibegate_seed_ok(vibegate_seed_ok)
        .with_dep_writetime_gate(Arc::new(DepWriteTimeGate::new(Some(ssb.clone()), registry_checker)))
        .with_supervisor(supervisor)
        .with_tdd_preflight_ssb(ssb.clone(), crate::task_runner::RUNNER_AGENT_ID, crate::task_runner::RUNNER_SUPERSTEP)
        .with_stop_cost_ssb(ssb.clone(), ssb.db_path().to_string_lossy().into_owned())
        .with_intervention_ssb(ssb.clone(), crate::task_runner::RUNNER_AGENT_ID, crate::task_runner::RUNNER_SUPERSTEP)
        .with_action_gate(Arc::new(crate::intervention::load_action_gate(project_root)));
    if let Some(gate) = stop_gate {
        dispatcher = dispatcher.with_stop_confirm_gate(gate);
    }
    Arc::new(dispatcher)
}

#[must_use]
pub fn build_production_ipc_hook_dispatcher(
    tiered: elai_plugins::tiered_analysis::TieredAnalysis,
    ssb: Arc<Ssb>,
) -> Arc<dyn crate::ipc_accept::IpcHookDispatcher + Send + Sync + 'static> {
    use elai_plugins::{IpcHookDispatcher, NoopLockChecker};
    Arc::new(IpcHookDispatcher::new(tiered, Box::new(RuntimeSsbFindingWriter::new(ssb)), Box::new(NoopLockChecker)))
}

pub struct RuntimeSsbFindingWriter {
    ssb: Arc<Ssb>,
}

impl RuntimeSsbFindingWriter {
    #[must_use]
    pub fn new(ssb: Arc<Ssb>) -> Self {
        Self { ssb }
    }
}

impl elai_plugins::SsbFindingWriter for RuntimeSsbFindingWriter {
    fn write_finding(
        &self,
        agent_id: &str,
        pattern_hash: &str,
        verdict_json: &str,
        files: &[String],
    ) -> Result<(), String> {
        let payload = crate::ssb::SsbPayload::Finding(crate::ssb::FindingPayload {
            severity: "INFO".to_string(),
            category: "ipc_hook".to_string(),
            message: format!("verdict={verdict_json} pattern_hash={pattern_hash}"),
            file_path: files.first().cloned(),
            line: None,
            failure_category: None,
            audit_step: None,
            few_shot_examples: None,
        });
        let handle = tokio::runtime::Handle::try_current()
            .map_err(|_| "RuntimeSsbFindingWriter::write_finding: no tokio runtime".to_string())?;
        let ssb = Arc::clone(&self.ssb);
        let agent_id = agent_id.to_string();
        let files = files.to_vec();
        tokio::task::block_in_place(move || {
            handle.block_on(async move { ssb.write_payload(&agent_id, 0, payload, files).await })
        })
        .map(|_| ())
        .map_err(|error| format!("ssb write_payload failed: {error}"))
    }
}

pub fn locate_default_agent_binary() -> Result<PathBuf, anyhow::Error> {
    if let Ok(env_val) = std::env::var(ELAI_AGENT_BIN_ENV) {
        return Ok(PathBuf::from(env_val));
    }
    let mut probed = Vec::new();
    let current_exe = std::env::current_exe()
        .map_err(|e| anyhow::anyhow!("locate_default_agent_binary: current_exe() failed: {e}"))?;
    if let Some(exe_dir) = current_exe.parent() {
        let candidate = exe_dir.join(AGENT_BINARY_NAME);
        if candidate.exists() {
            return Ok(candidate);
        }
        probed.push(candidate);
        let mut dir = exe_dir.to_path_buf();
        loop {
            for profile in ["debug", "release"] {
                let candidate = dir.join("target").join(profile).join(AGENT_BINARY_NAME);
                if candidate.exists() {
                    return Ok(candidate);
                }
                probed.push(candidate);
            }
            match dir.parent() {
                Some(parent) => dir = parent.to_path_buf(),
                None => break,
            }
        }
    }
    Err(anyhow::anyhow!(
        "locate_default_agent_binary: binary '{}' not found. Checked:\n  {}",
        AGENT_BINARY_NAME,
        probed
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join("\n  ")
    ))
}
