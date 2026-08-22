use std::sync::Arc;

use tokio::sync::Mutex as AsyncMutex;

use crate::{
    autonomous_gate::GateMode,
    background_task::BackgroundTaskManager,
    cloud_mode::CloudConfig,
    compiled_pipeline::{CompiledPipelineStore, TemplateCrystallizer},
    gate::InteractiveGate,
    hook_dispatcher::HookDispatcher,
    run_profile::RunProfile,
    skill_trust::SkillStore,
    task_runner::BabyStepExecutor,
    DefaultFeasibilityProber, DefaultLayer1aProcessor, DynReflectionClient, Ssb,
};

pub(crate) struct OptionalWiringServices {
    pub(crate) crystallizer: Arc<AsyncMutex<TemplateCrystallizer>>,
    pub(crate) compiled_pipeline_store: Arc<AsyncMutex<CompiledPipelineStore>>,
    pub(crate) background_task_manager: Arc<BackgroundTaskManager>,
    pub(crate) skill_store: Arc<AsyncMutex<SkillStore>>,
    pub(crate) pipeline_grc: Option<Arc<AsyncMutex<crate::grc::PipelineGrc>>>,
}

pub struct RuntimeWiring {
    pub ssb: Arc<Ssb>,
    pub feasibility_prober: Arc<DefaultFeasibilityProber>,
    pub layer1a_processor: Arc<DefaultLayer1aProcessor>,
    pub baby_step_executor: Option<Arc<dyn BabyStepExecutor>>,
    pub gate_mode: Option<GateMode>,
    pub run_profile: Option<RunProfile>,
    pub interactive_gate: Option<Arc<dyn InteractiveGate>>,
    pub cloud_config: Option<CloudConfig>,
    pub provider_override: Option<String>,
    pub hook_dispatcher: Option<Arc<dyn HookDispatcher + Send + Sync>>,
    pub crystallizer: Option<Arc<AsyncMutex<TemplateCrystallizer>>>,
    pub compiled_pipeline_store: Option<Arc<AsyncMutex<CompiledPipelineStore>>>,
    pub background_task_manager: Option<Arc<BackgroundTaskManager>>,
    pub skill_store: Option<Arc<AsyncMutex<SkillStore>>>,
    pub via_mcp: bool,
    pub mcp_dispatch: Option<Arc<dyn crate::dispatch::Dispatch>>,
    pub buffer_editor: Option<Arc<dyn crate::focus_tunnel::BufferEditor>>,
    pub pipeline_grc: Option<Arc<AsyncMutex<crate::grc::PipelineGrc>>>,
    pub vibegate_seed_ok: bool,
    pub reflection_client: Option<Arc<dyn DynReflectionClient>>,
    pub adversary_runtime: Option<Arc<crate::runtime_config::AdversaryRuntimeState>>,
    #[cfg(feature = "empire")]
    pub empire_bridge: Option<Arc<crate::empire::EmpireBridge>>,
}
