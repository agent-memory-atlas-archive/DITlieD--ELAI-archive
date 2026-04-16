#![deny(dead_code, unused_imports)]

use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;

use elai_ipc_types::{IntegrationCheckRequest, ToolResultPayload, TOOL_INTEGRATION_CHECK};
use crate::ipc_registry::{ToolError, ToolHandler};

pub struct IntegrationCheckHandler {
    project_root: PathBuf,
}

impl IntegrationCheckHandler {
    pub fn new(project_root: PathBuf) -> Self {
        Self { project_root }
    }

    fn effective_root(&self, req_root: &str) -> PathBuf {
        if req_root.is_empty() {
            self.project_root.clone()
        } else {
            PathBuf::from(req_root)
        }
    }

    async fn check_grep_reachable(root: &Path, symbol: &str) -> Result<bool, ToolError> {
        let output = tokio::process::Command::new("grep")
            .args(["-rq", symbol, "."])
            .current_dir(root)
            .output()
            .await
            .map_err(|e| ToolError::SubprocessFailed {
                tool_id: TOOL_INTEGRATION_CHECK.to_string(),
                reason: e.to_string(),
            })?;
        Ok(output.status.success())
    }

    async fn check_import_chain(root: &Path, symbol: &str) -> Result<Vec<String>, ToolError> {
        let output = tokio::process::Command::new("grep")
            .args(["-rn", &format!("use.*{symbol}"), "."])
            .current_dir(root)
            .output()
            .await
            .map_err(|e| ToolError::SubprocessFailed {
                tool_id: TOOL_INTEGRATION_CHECK.to_string(),
                reason: e.to_string(),
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|l| l.to_string()).collect())
    }
}

impl ToolHandler for IntegrationCheckHandler {
    fn execute<'a>(&'a self, arguments: &'a [u8]) -> Pin<Box<dyn Future<Output = Result<ToolResultPayload, ToolError>> + Send + 'a>> {
        Box::pin(async move {
            let req: IntegrationCheckRequest =
                postcard::from_bytes(arguments).map_err(|e| ToolError::DeserializationFailed {
                    tool_id: TOOL_INTEGRATION_CHECK.to_string(),
                    reason: e.to_string(),
                })?;

            let root = self.effective_root(&req.project_root);

            let mut results = Vec::new();
            let mut diagnostics = Vec::new();

            let grep_found = Self::check_grep_reachable(&root, &req.target_symbol).await?;
            results.push(format!(
                "grep_reachable: {}",
                if grep_found { "YES" } else { "NO" }
            ));

            let imports = Self::check_import_chain(&root, &req.target_symbol).await?;
            results.push(format!("import_chain: {} imports found", imports.len()));
            if imports.is_empty() {
                diagnostics.push(format!(
                    "WARNING: no import chain found for '{}'",
                    req.target_symbol
                ));
            }

            if !grep_found && imports.is_empty() {
                diagnostics.push(format!(
                    "ORPHANED: '{}' has no callers and no imports",
                    req.target_symbol
                ));
            }

            Ok(ToolResultPayload {
                output: results.join("\n"),
                diagnostics,
            })
        })
    }

    fn tool_id(&self) -> &str {
        TOOL_INTEGRATION_CHECK
    }

    fn is_write_tool(&self) -> bool {
        false
    }
}
