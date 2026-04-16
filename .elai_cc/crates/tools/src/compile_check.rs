#![deny(dead_code, unused_imports)]

use std::future::Future;
use std::pin::Pin;

use elai_ipc_types::{CompileCheckRequest, ToolResultPayload, TOOL_COMPILE_CHECK};
use crate::ipc_registry::{ToolError, ToolHandler};

/// Maximum retry rounds for incremental compilation.
const MAX_COMPILE_RETRIES: u8 = 3;

/// Classification for errors referencing unwritten modules.
const DEFERRED_CLASSIFICATION: &str = "DEFERRED";

pub struct CompileCheckHandler {
    project_root: std::path::PathBuf,
}

impl CompileCheckHandler {
    pub fn new(project_root: std::path::PathBuf) -> Self {
        Self { project_root }
    }

    fn compiler_command(language: &str) -> Option<(&'static str, Vec<String>)> {
        match language {
            "rust" => Some(("cargo", vec!["check".to_string(), "--message-format=json".to_string()])),
            "python" => Some(("ruff", vec!["check".to_string(), "--output-format=json".to_string()])),
            "typescript" => Some(("tsc", vec!["--noEmit".to_string()])),
            "go" => Some(("go", vec!["vet".to_string()])),
            _ => None,
        }
    }

    fn classify_error(stderr: &str) -> &'static str {
        if stderr.contains("can't find crate")
            || stderr.contains("unresolved import")
            || stderr.contains("No module named")
            || stderr.contains("Cannot find module")
            || stderr.contains("no source file found")
        {
            return DEFERRED_CLASSIFICATION;
        }
        "COMPILATION_ERROR"
    }
}

impl ToolHandler for CompileCheckHandler {
    fn execute<'a>(&'a self, arguments: &'a [u8]) -> Pin<Box<dyn Future<Output = Result<ToolResultPayload, ToolError>> + Send + 'a>> {
        Box::pin(async move {
            let req: CompileCheckRequest =
                postcard::from_bytes(arguments).map_err(|e| ToolError::DeserializationFailed {
                    tool_id: TOOL_COMPILE_CHECK.to_string(),
                    reason: e.to_string(),
                })?;

            if req.retry_round > MAX_COMPILE_RETRIES {
                return Ok(ToolResultPayload {
                    output: format!("max retries ({MAX_COMPILE_RETRIES}) exceeded, flagging NEEDS_AUDIT"),
                    diagnostics: vec!["NEEDS_AUDIT".to_string()],
                });
            }

            let (cmd, args) = Self::compiler_command(&req.language).ok_or_else(|| {
                ToolError::ExecutionFailed {
                    tool_id: TOOL_COMPILE_CHECK.to_string(),
                    reason: format!("unsupported language: {}", req.language),
                }
            })?;

            let output = tokio::process::Command::new(cmd)
                .args(&args)
                .current_dir(&self.project_root)
                .output()
                .await
                .map_err(|e| ToolError::SubprocessFailed {
                    tool_id: TOOL_COMPILE_CHECK.to_string(),
                    reason: e.to_string(),
                })?;

            let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

            let mut diagnostics = Vec::new();
            if !output.status.success() {
                // cargo check --message-format=json writes diagnostics to stdout;
                // other tools may use stderr. Check both streams.
                let combined = format!("{stdout}\n{stderr}");
                let classification = Self::classify_error(&combined);
                diagnostics.push(format!("classification={classification}"));
                diagnostics.push(format!("retry_round={}", req.retry_round));
                if !stderr.is_empty() {
                    diagnostics.push(stderr);
                }
                if !stdout.is_empty() {
                    diagnostics.push(stdout.clone());
                }
            }

            Ok(ToolResultPayload {
                output: stdout,
                diagnostics,
            })
        })
    }

    fn tool_id(&self) -> &str {
        TOOL_COMPILE_CHECK
    }

    fn is_write_tool(&self) -> bool {
        false
    }
}
