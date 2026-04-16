#![deny(dead_code, unused_imports)]

use std::future::Future;
use std::pin::Pin;

use elai_ipc_types::{ToolResultPayload, TraceRequest, TOOL_TRACE};
use crate::ipc_registry::{ToolError, ToolHandler};

const MAX_TRACE_DEPTH: u32 = 20;

pub struct TraceHandler {
    project_root: std::path::PathBuf,
}

impl TraceHandler {
    pub fn new(project_root: std::path::PathBuf) -> Self {
        Self { project_root }
    }

    async fn collect_call_chain(
        &self,
        file_path: &str,
        target_function: &str,
        depth: u32,
    ) -> Result<Vec<String>, ToolError> {
        let capped_depth = depth.min(MAX_TRACE_DEPTH);

        let output = tokio::process::Command::new("grep")
            .args(["-rn", target_function, file_path])
            .current_dir(&self.project_root)
            .output()
            .await
            .map_err(|e| ToolError::SubprocessFailed {
                tool_id: TOOL_TRACE.to_string(),
                reason: e.to_string(),
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut chain: Vec<String> = stdout
            .lines()
            .take(capped_depth as usize)
            .map(|l| l.to_string())
            .collect();

        if chain.is_empty() {
            chain.push(format!("no call sites found for '{target_function}' in {file_path}"));
        }

        Ok(chain)
    }
}

impl ToolHandler for TraceHandler {
    fn execute<'a>(&'a self, arguments: &'a [u8]) -> Pin<Box<dyn Future<Output = Result<ToolResultPayload, ToolError>> + Send + 'a>> {
        Box::pin(async move {
            let req: TraceRequest =
                postcard::from_bytes(arguments).map_err(|e| ToolError::DeserializationFailed {
                    tool_id: TOOL_TRACE.to_string(),
                    reason: e.to_string(),
                })?;

            let chain = self
                .collect_call_chain(&req.file_path, &req.target_function, req.depth)
                .await?;

            Ok(ToolResultPayload {
                output: chain.join("\n"),
                diagnostics: Vec::new(),
            })
        })
    }

    fn tool_id(&self) -> &str {
        TOOL_TRACE
    }

    fn is_write_tool(&self) -> bool {
        false
    }
}
