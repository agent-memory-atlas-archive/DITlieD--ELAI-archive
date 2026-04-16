#![deny(dead_code, unused_imports)]

use std::future::Future;
use std::pin::Pin;

use elai_ipc_types::{
    DkgCallersRequest, DkgDataFlowRequest, DkgDependentsRequest, DkgSubgraphRequest,
    ToolResultPayload, TOOL_DKG_CALLERS, TOOL_DKG_DATA_FLOW, TOOL_DKG_DEPENDENTS,
    TOOL_DKG_SUBGRAPH,
};
use crate::ipc_registry::{ToolError, ToolHandler};

const MAX_SUBGRAPH_DEPTH: u32 = 10;

/// Async trait for DKG query execution. Production impl uses rusqlite
/// Connection to dkg.db via DkgQuerierAdapter. Test impl returns canned data.
pub trait DkgBackend: Send + Sync {
    fn query_callers(&self, symbol: &str) -> Pin<Box<dyn Future<Output = Result<Vec<String>, String>> + Send + '_>>;
    fn query_dependents(&self, symbol: &str) -> Pin<Box<dyn Future<Output = Result<Vec<String>, String>> + Send + '_>>;
    fn query_data_flow(&self, variable: &str, direction: &str) -> Pin<Box<dyn Future<Output = Result<Vec<String>, String>> + Send + '_>>;
    fn query_subgraph(&self, entry_point: &str, depth: u32) -> Pin<Box<dyn Future<Output = Result<Vec<String>, String>> + Send + '_>>;
}

pub struct GrepFallbackBackend {
    project_root: std::path::PathBuf,
}

impl GrepFallbackBackend {
    pub fn new(project_root: std::path::PathBuf) -> Self {
        Self { project_root }
    }

    async fn grep_symbol(&self, symbol: &str) -> Result<Vec<String>, String> {
        let output = tokio::process::Command::new("grep")
            .args(["-rn", symbol, "."])
            .current_dir(&self.project_root)
            .output()
            .await
            .map_err(|e| e.to_string())?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|l| l.to_string()).collect())
    }
}

impl DkgBackend for GrepFallbackBackend {
    fn query_callers(&self, symbol: &str) -> Pin<Box<dyn Future<Output = Result<Vec<String>, String>> + Send + '_>> {
        let sym = symbol.to_string();
        Box::pin(async move { self.grep_symbol(&sym).await })
    }

    fn query_dependents(&self, symbol: &str) -> Pin<Box<dyn Future<Output = Result<Vec<String>, String>> + Send + '_>> {
        let sym = symbol.to_string();
        Box::pin(async move { self.grep_symbol(&sym).await })
    }

    fn query_data_flow(&self, variable: &str, direction: &str) -> Pin<Box<dyn Future<Output = Result<Vec<String>, String>> + Send + '_>> {
        let var = variable.to_string();
        let dir = direction.to_string();
        let root = self.project_root.clone();
        Box::pin(async move {
            // For "upstream": grep for assignments to the variable (data sources)
            // For "downstream": grep for reads/uses of the variable (data sinks)
            let pattern = match dir.as_str() {
                "upstream" => format!("{var}\\s*="),
                "downstream" | _ => var.clone(),
            };
            let output = tokio::process::Command::new("grep")
                .args(["-rn", &pattern, "."])
                .current_dir(&root)
                .output()
                .await
                .map_err(|e| e.to_string())?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut results: Vec<String> = stdout.lines().map(|l| l.to_string()).collect();
            results.insert(0, format!("[direction={dir}]"));
            Ok(results)
        })
    }

    fn query_subgraph(&self, entry_point: &str, depth: u32) -> Pin<Box<dyn Future<Output = Result<Vec<String>, String>> + Send + '_>> {
        let ep = entry_point.to_string();
        let root = self.project_root.clone();
        Box::pin(async move {
            let output = tokio::process::Command::new("grep")
                .args(["-rn", &ep, "."])
                .current_dir(&root)
                .output()
                .await
                .map_err(|e| e.to_string())?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Limit results by depth (approximate: each grep match = 1 hop)
            let results: Vec<String> = stdout
                .lines()
                .take(depth as usize)
                .map(|l| l.to_string())
                .collect();
            Ok(results)
        })
    }
}

pub struct DkgCallersHandler {
    backend: Box<dyn DkgBackend>,
}

impl DkgCallersHandler {
    pub fn new(backend: Box<dyn DkgBackend>) -> Self {
        Self { backend }
    }
}

impl ToolHandler for DkgCallersHandler {
    fn execute<'a>(&'a self, arguments: &'a [u8]) -> Pin<Box<dyn Future<Output = Result<ToolResultPayload, ToolError>> + Send + 'a>> {
        Box::pin(async move {
            let req: DkgCallersRequest =
                postcard::from_bytes(arguments).map_err(|e| ToolError::DeserializationFailed {
                    tool_id: TOOL_DKG_CALLERS.to_string(),
                    reason: e.to_string(),
                })?;

            let callers = self.backend.query_callers(&req.symbol).await.map_err(|e| ToolError::DkgQueryFailed {
                tool_id: TOOL_DKG_CALLERS.to_string(),
                reason: e,
            })?;

            Ok(ToolResultPayload {
                output: callers.join("\n"),
                diagnostics: vec![format!("{} callers found", callers.len())],
            })
        })
    }

    fn tool_id(&self) -> &str {
        TOOL_DKG_CALLERS
    }

    fn is_write_tool(&self) -> bool {
        false
    }
}

pub struct DkgDependentsHandler {
    backend: Box<dyn DkgBackend>,
}

impl DkgDependentsHandler {
    pub fn new(backend: Box<dyn DkgBackend>) -> Self {
        Self { backend }
    }
}

impl ToolHandler for DkgDependentsHandler {
    fn execute<'a>(&'a self, arguments: &'a [u8]) -> Pin<Box<dyn Future<Output = Result<ToolResultPayload, ToolError>> + Send + 'a>> {
        Box::pin(async move {
            let req: DkgDependentsRequest =
                postcard::from_bytes(arguments).map_err(|e| ToolError::DeserializationFailed {
                    tool_id: TOOL_DKG_DEPENDENTS.to_string(),
                    reason: e.to_string(),
                })?;

            let dependents = self.backend.query_dependents(&req.symbol).await.map_err(|e| ToolError::DkgQueryFailed {
                tool_id: TOOL_DKG_DEPENDENTS.to_string(),
                reason: e,
            })?;

            Ok(ToolResultPayload {
                output: dependents.join("\n"),
                diagnostics: vec![format!("{} dependents found", dependents.len())],
            })
        })
    }

    fn tool_id(&self) -> &str {
        TOOL_DKG_DEPENDENTS
    }

    fn is_write_tool(&self) -> bool {
        false
    }
}

pub struct DkgDataFlowHandler {
    backend: Box<dyn DkgBackend>,
}

impl DkgDataFlowHandler {
    pub fn new(backend: Box<dyn DkgBackend>) -> Self {
        Self { backend }
    }
}

impl ToolHandler for DkgDataFlowHandler {
    fn execute<'a>(&'a self, arguments: &'a [u8]) -> Pin<Box<dyn Future<Output = Result<ToolResultPayload, ToolError>> + Send + 'a>> {
        Box::pin(async move {
            let req: DkgDataFlowRequest =
                postcard::from_bytes(arguments).map_err(|e| ToolError::DeserializationFailed {
                    tool_id: TOOL_DKG_DATA_FLOW.to_string(),
                    reason: e.to_string(),
                })?;

            let flow = self.backend.query_data_flow(&req.variable, &req.direction).await.map_err(|e| ToolError::DkgQueryFailed {
                tool_id: TOOL_DKG_DATA_FLOW.to_string(),
                reason: e,
            })?;

            Ok(ToolResultPayload {
                output: flow.join("\n"),
                diagnostics: vec![format!("{} data flow entries", flow.len())],
            })
        })
    }

    fn tool_id(&self) -> &str {
        TOOL_DKG_DATA_FLOW
    }

    fn is_write_tool(&self) -> bool {
        false
    }
}

pub struct DkgSubgraphHandler {
    backend: Box<dyn DkgBackend>,
}

impl DkgSubgraphHandler {
    pub fn new(backend: Box<dyn DkgBackend>) -> Self {
        Self { backend }
    }
}

impl ToolHandler for DkgSubgraphHandler {
    fn execute<'a>(&'a self, arguments: &'a [u8]) -> Pin<Box<dyn Future<Output = Result<ToolResultPayload, ToolError>> + Send + 'a>> {
        Box::pin(async move {
            let req: DkgSubgraphRequest =
                postcard::from_bytes(arguments).map_err(|e| ToolError::DeserializationFailed {
                    tool_id: TOOL_DKG_SUBGRAPH.to_string(),
                    reason: e.to_string(),
                })?;

            let capped_depth = req.depth.min(MAX_SUBGRAPH_DEPTH);
            let subgraph = self.backend.query_subgraph(&req.entry_point, capped_depth).await.map_err(|e| ToolError::DkgQueryFailed {
                tool_id: TOOL_DKG_SUBGRAPH.to_string(),
                reason: e,
            })?;

            let mut diagnostics = Vec::new();
            if req.depth > MAX_SUBGRAPH_DEPTH {
                diagnostics.push(format!("depth capped from {} to {MAX_SUBGRAPH_DEPTH}", req.depth));
            }
            diagnostics.push(format!("{} nodes in subgraph", subgraph.len()));

            Ok(ToolResultPayload {
                output: subgraph.join("\n"),
                diagnostics,
            })
        })
    }

    fn tool_id(&self) -> &str {
        TOOL_DKG_SUBGRAPH
    }

    fn is_write_tool(&self) -> bool {
        false
    }
}
