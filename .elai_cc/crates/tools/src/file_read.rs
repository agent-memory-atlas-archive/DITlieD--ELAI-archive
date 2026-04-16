#![deny(dead_code, unused_imports)]

use std::future::Future;
use std::pin::Pin;

use elai_ipc_types::{FileReadRequest, ToolResultPayload, TOOL_FILE_READ};
use crate::ipc_registry::{ToolError, ToolHandler};

/// Files above this line count get outline-mode by default.
const OUTLINE_LINE_THRESHOLD: u32 = 300;

/// Default number of lines to return when no limit is specified.
const DEFAULT_LINE_LIMIT: u32 = 200;

pub struct FileReadHandler {
    project_root: std::path::PathBuf,
}

impl FileReadHandler {
    pub fn new(project_root: std::path::PathBuf) -> Self {
        Self { project_root }
    }

    fn generate_outline(content: &str) -> String {
        let mut outline = Vec::new();
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("pub ")
                || trimmed.starts_with("fn ")
                || trimmed.starts_with("struct ")
                || trimmed.starts_with("enum ")
                || trimmed.starts_with("impl ")
                || trimmed.starts_with("trait ")
                || trimmed.starts_with("mod ")
                || trimmed.starts_with("const ")
                || trimmed.starts_with("static ")
                || trimmed.starts_with("type ")
                || trimmed.starts_with("class ")
                || trimmed.starts_with("def ")
                || trimmed.starts_with("async def ")
                || trimmed.starts_with("function ")
                || trimmed.starts_with("export ")
            {
                outline.push(format!("{}:{}", i + 1, line));
            }
        }
        outline.join("\n")
    }
}

impl ToolHandler for FileReadHandler {
    fn execute<'a>(&'a self, arguments: &'a [u8]) -> Pin<Box<dyn Future<Output = Result<ToolResultPayload, ToolError>> + Send + 'a>> {
        Box::pin(async move {
            let req: FileReadRequest =
                postcard::from_bytes(arguments).map_err(|e| ToolError::DeserializationFailed {
                    tool_id: TOOL_FILE_READ.to_string(),
                    reason: e.to_string(),
                })?;

            let full_path = self.project_root.join(&req.file_path);
            let content = tokio::fs::read_to_string(&full_path)
                .await
                .map_err(|e| ToolError::Io {
                    tool_id: TOOL_FILE_READ.to_string(),
                    source: e,
                })?;

            let total_lines: u32 = content.lines().count().try_into().unwrap_or(u32::MAX);
            let offset = req.offset.unwrap_or(0);
            let limit = req.limit.unwrap_or(DEFAULT_LINE_LIMIT);

            if total_lines > OUTLINE_LINE_THRESHOLD && req.offset.is_none() && req.limit.is_none() {
                let outline = Self::generate_outline(&content);
                return Ok(ToolResultPayload {
                    output: outline,
                    diagnostics: vec![format!(
                        "file has {total_lines} lines (>{OUTLINE_LINE_THRESHOLD}), returning outline"
                    )],
                });
            }

            let lines: Vec<&str> = content
                .lines()
                .skip(offset as usize)
                .take(limit as usize)
                .collect();

            let mut diagnostics = Vec::new();
            if total_lines > offset + lines.len() as u32 {
                diagnostics.push(format!(
                    "showing lines {}-{} of {total_lines}",
                    offset + 1,
                    offset + lines.len() as u32
                ));
            }

            Ok(ToolResultPayload {
                output: lines.join("\n"),
                diagnostics,
            })
        })
    }

    fn tool_id(&self) -> &str {
        TOOL_FILE_READ
    }

    fn is_write_tool(&self) -> bool {
        false
    }
}
