#![deny(dead_code, unused_imports)]

use std::future::Future;
use std::pin::Pin;

use elai_ipc_types::{EditRequest, SimplifyRequest, ToolResultPayload, TOOL_EDIT, TOOL_SIMPLIFY};
use crate::ipc_registry::{ToolError, ToolHandler};

pub struct EditHandler {
    project_root: std::path::PathBuf,
}

impl EditHandler {
    pub fn new(project_root: std::path::PathBuf) -> Self {
        Self { project_root }
    }
}

impl ToolHandler for EditHandler {
    fn execute<'a>(&'a self, arguments: &'a [u8]) -> Pin<Box<dyn Future<Output = Result<ToolResultPayload, ToolError>> + Send + 'a>> {
        Box::pin(async move {
            let req: EditRequest =
                postcard::from_bytes(arguments).map_err(|e| ToolError::DeserializationFailed {
                    tool_id: TOOL_EDIT.to_string(),
                    reason: e.to_string(),
                })?;

            let full_path = self.project_root.join(&req.file_path);
            let content = tokio::fs::read_to_string(&full_path)
                .await
                .map_err(|e| ToolError::Io {
                    tool_id: TOOL_EDIT.to_string(),
                    source: e,
                })?;

            let occurrence_count = content.matches(&req.old_text).count();
            if occurrence_count == 0 {
                return Err(ToolError::ExecutionFailed {
                    tool_id: TOOL_EDIT.to_string(),
                    reason: format!("old_text not found in {}", req.file_path),
                });
            }
            if occurrence_count > 1 {
                return Err(ToolError::ExecutionFailed {
                    tool_id: TOOL_EDIT.to_string(),
                    reason: format!(
                        "old_text matches {} times in {} (must be unique)",
                        occurrence_count, req.file_path
                    ),
                });
            }

            let new_content = content.replacen(&req.old_text, &req.new_text, 1);
            tokio::fs::write(&full_path, &new_content)
                .await
                .map_err(|e| ToolError::Io {
                    tool_id: TOOL_EDIT.to_string(),
                    source: e,
                })?;

            Ok(ToolResultPayload {
                output: format!("edited {}: 1 replacement", req.file_path),
                diagnostics: Vec::new(),
            })
        })
    }

    fn tool_id(&self) -> &str {
        TOOL_EDIT
    }

    fn is_write_tool(&self) -> bool {
        true
    }
}

pub struct SimplifyHandler {
    project_root: std::path::PathBuf,
}

impl SimplifyHandler {
    pub fn new(project_root: std::path::PathBuf) -> Self {
        Self { project_root }
    }
}

impl ToolHandler for SimplifyHandler {
    fn execute<'a>(&'a self, arguments: &'a [u8]) -> Pin<Box<dyn Future<Output = Result<ToolResultPayload, ToolError>> + Send + 'a>> {
        Box::pin(async move {
            let req: SimplifyRequest =
                postcard::from_bytes(arguments).map_err(|e| ToolError::DeserializationFailed {
                    tool_id: TOOL_SIMPLIFY.to_string(),
                    reason: e.to_string(),
                })?;

            let full_path = self.project_root.join(&req.file_path);
            let content = tokio::fs::read_to_string(&full_path)
                .await
                .map_err(|e| ToolError::Io {
                    tool_id: TOOL_SIMPLIFY.to_string(),
                    source: e,
                })?;

            let mut in_function = false;
            let mut function_lines = Vec::new();
            let mut brace_depth: i32 = 0;

            for line in content.lines() {
                if !in_function && line.contains(&format!("fn {}", req.target_function)) {
                    in_function = true;
                }
                if in_function {
                    brace_depth += i32::try_from(line.matches('{').count()).unwrap_or(i32::MAX);
                    brace_depth -= i32::try_from(line.matches('}').count()).unwrap_or(i32::MAX);
                    function_lines.push(line.to_string());
                    if brace_depth <= 0 && !function_lines.is_empty() {
                        break;
                    }
                }
            }

            if function_lines.is_empty() {
                return Err(ToolError::ExecutionFailed {
                    tool_id: TOOL_SIMPLIFY.to_string(),
                    reason: format!(
                        "function '{}' not found in {}",
                        req.target_function, req.file_path
                    ),
                });
            }

            Ok(ToolResultPayload {
                output: format!(
                    "function '{}': {} lines extracted for simplification analysis",
                    req.target_function,
                    function_lines.len()
                ),
                diagnostics: Vec::new(),
            })
        })
    }

    fn tool_id(&self) -> &str {
        TOOL_SIMPLIFY
    }

    fn is_write_tool(&self) -> bool {
        false
    }
}
