// B-15 Tranche O / T12b: MCP glue layer extracted from live_cli.rs to bound
// live_cli.rs size. Holds RuntimeMcpState (struct + impl), the MCP wrapper
// tool-definition builders, the permission-mode mapping helpers, and the
// inbound MCP request deserialization structs. All items keep pub(crate)
// visibility to match their original surface.
//
// live_cli.rs re-exports `pub(crate) use crate::mcp_glue::*;` so every
// consumer that reached these via `crate::live_cli::RuntimeMcpState`,
// `use crate::live_cli::{RuntimeMcpState, ...}` (runtime_builder.rs:76),
// the blanket `pub(crate) use crate::live_cli::*;` (main.rs:298), and the
// `crate::live_cli::RuntimeMcpState` reference in cli_consts.rs continue to
// resolve byte-for-byte through the re-export chain.

#![allow(dead_code, unused_imports, unused_variables, unused_mut, clippy::unneeded_struct_pattern, clippy::unnecessary_wraps, clippy::unused_self)]

use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use elai_orchestrator::{
    DeferredResolveInfo, McpServerManager, McpTool, PermissionMode, ToolError,
    ELAI_MCP_DEFER_SCHEMA_ENABLE_ENV,
};
use elai_tools::RuntimeToolDefinition;
use serde::Deserialize;
use serde_json::json;
use tracing::info;

use crate::cli_consts::RuntimePluginStateBuildOutput;

pub(crate) struct RuntimeMcpState {
    pub(crate) runtime: tokio::runtime::Runtime,
    pub(crate) manager: McpServerManager,
    pub(crate) pending_servers: Vec<String>,
    pub(crate) degraded_report: Option<elai_orchestrator::McpDegradedReport>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ToolSearchRequest {
    pub(crate) query: String,
    pub(crate) max_results: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct McpToolRequest {
    #[serde(rename = "qualifiedName")]
    pub(crate) qualified_name: Option<String>,
    pub(crate) tool: Option<String>,
    pub(crate) arguments: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ListMcpResourcesRequest {
    pub(crate) server: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReadMcpResourceRequest {
    pub(crate) server: String,
    pub(crate) uri: String,
}

impl RuntimeMcpState {
    pub(crate) fn new(
        runtime_config: &elai_orchestrator::RuntimeConfig,
    ) -> Result<Option<(Self, elai_orchestrator::McpToolDiscoveryReport)>, anyhow::Error> {
        let mut manager = McpServerManager::from_runtime_config(runtime_config);
        if manager.server_names().is_empty() && manager.unsupported_servers().is_empty() {
            return Ok(None);
        }

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let discovery = runtime.block_on(manager.discover_tools_best_effort());
        let pending_servers = discovery
            .failed_servers
            .iter()
            .map(|failure| failure.server_name.clone())
            .chain(
                discovery
                    .unsupported_servers
                    .iter()
                    .map(|server| server.server_name.clone()),
            )
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let available_tools = discovery
            .tools
            .iter()
            .map(|tool| tool.qualified_name.clone())
            .collect::<Vec<_>>();
        let failed_server_names = pending_servers.iter().cloned().collect::<BTreeSet<_>>();
        let working_servers = manager
            .server_names()
            .into_iter()
            .filter(|server_name| !failed_server_names.contains(server_name))
            .collect::<Vec<_>>();
        let failed_servers =
            discovery
                .failed_servers
                .iter()
                .map(|failure| elai_orchestrator::McpFailedServer {
                    server_name: failure.server_name.clone(),
                    phase: elai_orchestrator::McpLifecyclePhase::ToolDiscovery,
                    error: elai_orchestrator::McpErrorSurface::new(
                        elai_orchestrator::McpLifecyclePhase::ToolDiscovery,
                        Some(failure.server_name.clone()),
                        failure.error.clone(),
                        std::collections::BTreeMap::new(),
                        true,
                    ),
                })
                .chain(discovery.unsupported_servers.iter().map(|server| {
                    elai_orchestrator::McpFailedServer {
                        server_name: server.server_name.clone(),
                        phase: elai_orchestrator::McpLifecyclePhase::ServerRegistration,
                        error: elai_orchestrator::McpErrorSurface::new(
                            elai_orchestrator::McpLifecyclePhase::ServerRegistration,
                            Some(server.server_name.clone()),
                            server.reason.clone(),
                            std::collections::BTreeMap::from([(
                                "transport".to_string(),
                                format!("{:?}", server.transport).to_ascii_lowercase(),
                            )]),
                            false,
                        ),
                    }
                }))
                .collect::<Vec<_>>();
        let degraded_report = (!failed_servers.is_empty()).then(|| {
            elai_orchestrator::McpDegradedReport::new(
                working_servers,
                failed_servers,
                available_tools.clone(),
                available_tools,
            )
        });

        Ok(Some((
            Self {
                runtime,
                manager,
                pending_servers,
                degraded_report,
            },
            discovery,
        )))
    }

    pub(crate) fn shutdown(&mut self) -> Result<(), anyhow::Error> {
        self.runtime.block_on(self.manager.shutdown())?;
        Ok(())
    }

    pub(crate) fn pending_servers(&self) -> Option<Vec<String>> {
        (!self.pending_servers.is_empty()).then(|| self.pending_servers.clone())
    }

    pub(crate) fn degraded_report(&self) -> Option<elai_orchestrator::McpDegradedReport> {
        self.degraded_report.clone()
    }

    pub(crate) fn server_names(&self) -> Vec<String> {
        self.manager.server_names()
    }

    pub(crate) fn call_tool(
        &mut self,
        qualified_tool_name: &str,
        arguments: Option<serde_json::Value>,
    ) -> Result<String, ToolError> {
        // H-34: deferred schema resolve path. When ELAI_MCP_DEFER_SCHEMA_ENABLE=1,
        // check if the called tool's schema was withheld. If so, resolve it from the
        // StdioSchemaCache and emit a McpToolDeferred SSB row (one per cache-miss,
        // not per hit -- mirrors mcp_bridge.rs:974-990 resolve_schema_on_demand shape).
        // The SSB emission is best-effort: failure to emit never blocks the tool call.
        let defer_info: Option<DeferredResolveInfo> = if std::env::var(ELAI_MCP_DEFER_SCHEMA_ENABLE_ENV)
            .map(|v| v.trim() == "1")
            .unwrap_or(false)
        {
            self.runtime
                .block_on(self.manager.resolve_withheld_schema(qualified_tool_name))
        } else {
            None
        };

        // Emit a McpToolDeferred SSB row on the FIRST resolve only (cache-miss).
        // cache_hit=true means this tool was already resolved in a previous call;
        // we log at debug level but do NOT emit a duplicate SSB row.
        if let Some(ref info) = defer_info {
            if !info.cache_hit {
                info!(
                    server_name = %info.server_name,
                    tool = %info.tool_qualified_name,
                    resolve_ns = info.resolve_ns,
                    cache_hit = info.cache_hit,
                    floor_size = info.injected_floor_size,
                    "mcp_glue: deferred schema resolved for tool (first use)"
                );
                emit_mcp_tool_deferred_best_effort(info);
            } else {
                tracing::debug!(
                    tool = %info.tool_qualified_name,
                    "mcp_glue: deferred schema already resolved (cache-hit), skipping SSB emit"
                );
            }
        }

        let response = self
            .runtime
            .block_on(self.manager.call_tool(qualified_tool_name, arguments))
            .map_err(|error| ToolError::new(error.to_string()))?;
        if let Some(error) = response.error {
            return Err(ToolError::new(format!(
                "MCP tool `{qualified_tool_name}` returned JSON-RPC error: {} ({})",
                error.message, error.code
            )));
        }

        let result = response.result.ok_or_else(|| {
            ToolError::new(format!(
                "MCP tool `{qualified_tool_name}` returned no result payload"
            ))
        })?;
        serde_json::to_string_pretty(&result).map_err(|error| ToolError::new(error.to_string()))
    }

    pub(crate) fn list_resources_for_server(&mut self, server_name: &str) -> Result<String, ToolError> {
        let result = self
            .runtime
            .block_on(self.manager.list_resources(server_name))
            .map_err(|error| ToolError::new(error.to_string()))?;
        serde_json::to_string_pretty(&json!({
            "server": server_name,
            "resources": result.resources,
        }))
        .map_err(|error| ToolError::new(error.to_string()))
    }

    pub(crate) fn list_resources_for_all_servers(&mut self) -> Result<String, ToolError> {
        let mut resources = Vec::new();
        let mut failures = Vec::new();

        for server_name in self.server_names() {
            match self
                .runtime
                .block_on(self.manager.list_resources(&server_name))
            {
                Ok(result) => resources.push(json!({
                    "server": server_name,
                    "resources": result.resources,
                })),
                Err(error) => failures.push(json!({
                    "server": server_name,
                    "error": error.to_string(),
                })),
            }
        }

        if resources.is_empty() && !failures.is_empty() {
            let message = failures
                .iter()
                .filter_map(|failure| failure.get("error").and_then(serde_json::Value::as_str))
                .collect::<Vec<_>>()
                .join("; ");
            return Err(ToolError::new(message));
        }

        serde_json::to_string_pretty(&json!({
            "resources": resources,
            "failures": failures,
        }))
        .map_err(|error| ToolError::new(error.to_string()))
    }

    pub(crate) fn read_resource(&mut self, server_name: &str, uri: &str) -> Result<String, ToolError> {
        let result = self
            .runtime
            .block_on(self.manager.read_resource(server_name, uri))
            .map_err(|error| ToolError::new(error.to_string()))?;
        serde_json::to_string_pretty(&json!({
            "server": server_name,
            "contents": result.contents,
        }))
        .map_err(|error| ToolError::new(error.to_string()))
    }
}

pub(crate) fn build_runtime_mcp_state(
    runtime_config: &elai_orchestrator::RuntimeConfig,
) -> Result<RuntimePluginStateBuildOutput, anyhow::Error> {
    let Some((mcp_state, discovery)) = RuntimeMcpState::new(runtime_config)? else {
        return Ok((None, Vec::new()));
    };

    let mut runtime_tools = discovery
        .tools
        .iter()
        .map(mcp_runtime_tool_definition)
        .collect::<Vec<_>>();
    if !mcp_state.server_names().is_empty() {
        runtime_tools.extend(mcp_wrapper_tool_definitions());
    }

    Ok((Some(Arc::new(Mutex::new(mcp_state))), runtime_tools))
}

pub(crate) fn mcp_runtime_tool_definition(tool: &elai_orchestrator::ManagedMcpTool) -> RuntimeToolDefinition {
    RuntimeToolDefinition {
        name: tool.qualified_name.clone(),
        description: Some(
            tool.tool
                .description
                .clone()
                .unwrap_or_else(|| format!("Invoke MCP tool `{}`.", tool.qualified_name)),
        ),
        input_schema: tool
            .tool
            .input_schema
            .clone()
            .unwrap_or_else(|| json!({ "type": "object", "additionalProperties": true })),
        required_permission: permission_mode_for_mcp_tool(&tool.tool),
    }
}

pub(crate) fn mcp_wrapper_tool_definitions() -> Vec<RuntimeToolDefinition> {
    vec![
        RuntimeToolDefinition {
            name: "MCPTool".to_string(),
            description: Some(
                "Call a configured MCP tool by its qualified name and JSON arguments.".to_string(),
            ),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "qualifiedName": { "type": "string" },
                    "arguments": {}
                },
                "required": ["qualifiedName"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::DangerFullAccess,
        },
        RuntimeToolDefinition {
            name: "ListMcpResourcesTool".to_string(),
            description: Some(
                "List MCP resources from one configured server or from every connected server."
                    .to_string(),
            ),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "server": { "type": "string" }
                },
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        RuntimeToolDefinition {
            name: "ReadMcpResourceTool".to_string(),
            description: Some("Read a specific MCP resource from a configured server.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "server": { "type": "string" },
                    "uri": { "type": "string" }
                },
                "required": ["server", "uri"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
    ]
}

pub(crate) fn permission_mode_for_mcp_tool(tool: &McpTool) -> PermissionMode {
    let read_only = mcp_annotation_flag(tool, "readOnlyHint");
    let destructive = mcp_annotation_flag(tool, "destructiveHint");
    let open_world = mcp_annotation_flag(tool, "openWorldHint");

    if read_only && !destructive && !open_world {
        PermissionMode::ReadOnly
    } else if destructive || open_world {
        PermissionMode::DangerFullAccess
    } else {
        PermissionMode::WorkspaceWrite
    }
}

pub(crate) fn mcp_annotation_flag(tool: &McpTool, key: &str) -> bool {
    tool.annotations
        .as_ref()
        .and_then(|annotations| annotations.get(key))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
}

/// H-34: best-effort SSB emission for a deferred schema resolve event.
///
/// Opens a minimal SSB writer in a new current-thread tokio runtime and
/// emits one `McpToolDeferred` beacon. Best-effort: on any failure (missing
/// ssb.db dir, writer init error) we log via `tracing::warn` and continue.
/// The tool call itself is NEVER blocked by SSB emission failure.
///
/// This mirrors the `bench_staleness::emit_staleness_eval_measured` pattern
/// exactly: `Ssb::new` MUST be constructed inside `rt.block_on` or it panics
/// "no reactor running" on a live run (cargo check passes regardless).
fn emit_mcp_tool_deferred_best_effort(info: &DeferredResolveInfo) {
    use std::fs;
    use std::sync::Arc as StdArc;
    use elai_core::paths::sessions_dir_path;
    use elai_core::OsClock;
    use elai_orchestrator::ssb::McpToolDeferredPayload;
    use elai_orchestrator::{Ssb, SsbConfig, SsbPayload};

    let project_root = match std::env::current_dir() {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("mcp_glue: McpToolDeferred SSB emit skipped, no cwd: {e}");
            return;
        }
    };
    let db_path = sessions_dir_path(&project_root).join("ssb.db");
    if let Some(parent) = db_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            tracing::warn!("mcp_glue: McpToolDeferred SSB emit skipped, create_dir_all: {e}");
            return;
        }
    }

    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: "default".to_string(),
        session_id: "cli-mcp-defer".to_string(),
    };
    let clock: StdArc<dyn elai_core::clock::ClockProvider> = StdArc::new(OsClock);

    let payload = McpToolDeferredPayload {
        server_name: info.server_name.clone(),
        tool_qualified_name: info.tool_qualified_name.clone(),
        resolve_ns: info.resolve_ns,
        cache_hit: info.cache_hit,
        injected_floor_size: info.injected_floor_size,
    };

    // Build a new current-thread runtime for the SSB emit. Ssb::new must be
    // inside block_on. Failure is best-effort -- we warn and continue.
    match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => {
            let result = rt.block_on(async move {
                let ssb = Ssb::new(cfg, clock).map_err(|e| e.to_string())?;
                ssb.write_payload(
                    "cli:mcp-defer",
                    0,
                    SsbPayload::McpToolDeferred(payload),
                    Vec::new(),
                )
                .await
                .map_err(|e| e.to_string())?;
                Ok::<(), String>(())
            });
            if let Err(e) = result {
                tracing::warn!("mcp_glue: McpToolDeferred SSB emit failed (best-effort): {e}");
            }
        }
        Err(e) => {
            tracing::warn!("mcp_glue: McpToolDeferred SSB runtime build failed: {e}");
        }
    }
}
