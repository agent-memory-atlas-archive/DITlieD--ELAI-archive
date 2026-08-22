use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use elai_core::OsEntropy;

use super::runtime_dispatcher::build_buffer_tool_dispatcher;
use crate::runtime_builder::RuntimeWiring;

type SharedToolDispatcher = Arc<dyn elai_core::tool_dispatcher::ToolDispatcher>;

static PENDING_TOOL_DISPATCHERS: OnceLock<Mutex<BTreeMap<PathBuf, SharedToolDispatcher>>> =
    OnceLock::new();

fn pending_tool_dispatchers() -> &'static Mutex<BTreeMap<PathBuf, SharedToolDispatcher>> {
    PENDING_TOOL_DISPATCHERS.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn with_pending_tool_dispatchers<T>(
    operation: impl FnOnce(&mut BTreeMap<PathBuf, SharedToolDispatcher>) -> T,
) -> T {
    let mut pending = match pending_tool_dispatchers().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    operation(&mut pending)
}

fn dispatcher_handoff_path(project_root: &Path, session_id: &str) -> PathBuf {
    project_root
        .join(elai_core::paths::ELAI_DATA_DIR_NAME)
        .join("sessions")
        .join(session_id)
        .join("cursor.json")
}

fn remember_shared_tool_dispatcher(
    cursor_path: PathBuf,
    dispatcher: SharedToolDispatcher,
) -> anyhow::Result<()> {
    let inserted = with_pending_tool_dispatchers(|pending| {
        if pending.contains_key(&cursor_path) {
            false
        } else {
            pending.insert(cursor_path, dispatcher);
            true
        }
    });
    if inserted {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "agent pool: dispatcher already registered for the runner cursor"
        ))
    }
}

pub(crate) fn apply_shared_tool_dispatcher(
    cfg: &mut elai_orchestrator::TaskRunnerConfig,
    cursor_path: &Path,
) {
    let dispatcher = with_pending_tool_dispatchers(|pending| pending.remove(cursor_path));
    if dispatcher.is_some() {
        cfg.tool_dispatcher = dispatcher;
    }
}

pub(crate) async fn install_production_agent_pool(
    wiring: &mut RuntimeWiring,
    project_root: &Path,
    session_id: &str,
) -> anyhow::Result<()> {
    let ipc_dir = project_root
        .join(elai_core::paths::ELAI_DATA_DIR_NAME)
        .join("sessions")
        .join(session_id)
        .join("ipc");
    if let Err(e) = std::fs::create_dir_all(&ipc_dir) {
        tracing::warn!(
            path = %ipc_dir.display(),
            error = %e,
            "agent pool: failed to create IPC socket dir; executor will fall back to synthetic"
        );
        return Ok(());
    }
    let agent_bin = match elai_orchestrator::locate_default_agent_binary() {
        Ok(path) => path,
        Err(e) => {
            tracing::warn!(
                error = %e,
                "agent pool: agent binary not found; using synthetic fallback executor"
            );
            return Ok(());
        }
    };
    let gateway = Arc::new(tokio::sync::Mutex::new(
        elai_orchestrator::ipc_gateway::IpcGateway::with_os_clock(
            ipc_dir.clone(),
            elai_orchestrator::sandbox::SelfModificationLocks::new(),
        ),
    ));
    let (rpc_server, tool_dispatcher) = select_agent_pool_dispatchers(wiring, session_id);
    let executor = build_agent_pool_executor(
        agent_bin.clone(),
        ipc_dir,
        gateway,
        rpc_server,
        wiring,
    );
    wiring.baby_step_executor = Some(executor);
    if let Some(dispatcher) = tool_dispatcher {
        remember_shared_tool_dispatcher(
            dispatcher_handoff_path(project_root, session_id),
            dispatcher,
        )?;
    }
    tracing::info!(
        agent_bin = %agent_bin.display(),
        "agent pool: production AgentPoolBabyStepExecutor installed"
    );
    Ok(())
}

fn select_agent_pool_dispatchers(
    wiring: &RuntimeWiring,
    session_id: &str,
) -> (
    Arc<dyn elai_orchestrator::AgentRpcServer + Send + Sync + 'static>,
    Option<SharedToolDispatcher>,
) {
    if std::env::var("ELAI_GROUNDING_BACKEND").as_deref() == Ok("cloud") {
        match elai_providers::AnthropicClient::from_env() {
            Ok(client) => {
                let (server, dispatcher) = build_agent_pool_rpc_server(
                    client,
                    wiring.ssb.clone(),
                    session_id,
                );
                (server, Some(dispatcher))
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "agent pool: cloud credentials unavailable; using NoOpAgentRpcServer"
                );
                (Arc::new(elai_orchestrator::NoOpAgentRpcServer), None)
            }
        }
    } else {
        let base_url = std::env::var("ANTHROPIC_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8100".to_string());
        let client = elai_providers::AnthropicClient::new("local-dummy").with_base_url(base_url);
        let (server, dispatcher) = build_agent_pool_rpc_server(
            client,
            wiring.ssb.clone(),
            session_id,
        );
        (server, Some(dispatcher))
    }
}

fn build_agent_pool_executor(
    agent_bin: std::path::PathBuf,
    ipc_dir: std::path::PathBuf,
    gateway: Arc<tokio::sync::Mutex<elai_orchestrator::ipc_gateway::IpcGateway>>,
    rpc_server: Arc<dyn elai_orchestrator::AgentRpcServer + Send + Sync + 'static>,
    wiring: &RuntimeWiring,
) -> Arc<dyn elai_orchestrator::task_runner::BabyStepExecutor> {
    if elai_orchestrator::runtime_config::sandbox_firecracker_enabled() {
        let session_dir = ipc_dir
            .parent()
            .map(std::path::Path::to_path_buf)
            .unwrap_or_else(|| ipc_dir.clone());
        tracing::info!(
            session_dir = %session_dir.display(),
            "agent pool: E-15 firecracker sandbox pool ENABLED via ELAI_SANDBOX_FIRECRACKER_ENABLE"
        );
        elai_orchestrator::build_default_baby_step_executor_with_sandbox(
            OsEntropy,
            agent_bin,
            ipc_dir,
            gateway,
            rpc_server,
            wiring.ssb.clone(),
            session_dir,
        )
    } else {
        elai_orchestrator::build_default_baby_step_executor_with_ipc_hooks(
            OsEntropy,
            agent_bin,
            ipc_dir,
            gateway,
            rpc_server,
            elai_orchestrator::runtime_setup::build_stub_tiered_analysis(),
            wiring.ssb.clone(),
        )
    }
}

fn build_agent_pool_rpc_server(
    client: elai_providers::AnthropicClient,
    ssb: Arc<elai_orchestrator::Ssb>,
    session_id: &str,
) -> (
    Arc<dyn elai_orchestrator::AgentRpcServer + Send + Sync + 'static>,
    SharedToolDispatcher,
) {
    let model = std::env::var("ELAI_MODEL")
        .unwrap_or_else(|_| "claude-sonnet-4-6".to_string());
    let max_tokens = std::env::var("ELAI_MAX_TOKENS")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(4096);
    let ssb_for_server = Arc::clone(&ssb);
    let dispatcher: SharedToolDispatcher = Arc::new(
        elai_orchestrator::durability::IdempotentDispatcher::new(
            build_buffer_tool_dispatcher(ssb),
            session_id.to_string(),
        ),
    );
    let mut server = elai_orchestrator::ProductionAgentRpcServer::new(
        client,
        dispatcher.clone(),
        model,
        max_tokens,
    )
    .with_system_prompt(
        elai_orchestrator::production_agent_rpc::DEFAULT_BABY_STEP_SYSTEM_PROMPT,
    )
    .with_ssb(ssb_for_server);
    if let Ok(pins) = elai_orchestrator::determinism::DeterminismConfig::from_env() {
        server = server.with_seed(pins.seed);
    }
    if let Some(adapter) = load_symbol_graph_adapter() {
        server = server.with_symbol_graph_adapter(adapter);
    }
    (Arc::new(server), dispatcher)
}

fn load_symbol_graph_adapter() -> Option<
    Arc<dyn elai_knowledge::symbol_graph_adapter::SymbolGraphAdapter>,
> {
    let graph_path = std::path::PathBuf::from(".claude/tools/elai_graph.json");
    if !graph_path.exists() {
        tracing::debug!(
            graph = %graph_path.display(),
            "G.3 elai_graph.json absent; symbol-graph tool arms degrade to empty"
        );
        return None;
    }
    match elai_orchestrator::symbol_graph::SymbolGraphCache::load(&graph_path) {
        Ok(cache) => {
            let cache_arc = Arc::new(std::sync::Mutex::new(cache));
            Some(Arc::new(
                elai_orchestrator::symbol_graph::SymbolGraphCacheAdapter::new(cache_arc, 500),
            ))
        }
        Err(e) => {
            tracing::warn!(
                error = %e,
                graph = %graph_path.display(),
                "G.3 SymbolGraphCache::load failed; symbol-graph tool arms degrade to empty"
            );
            None
        }
    }
}
