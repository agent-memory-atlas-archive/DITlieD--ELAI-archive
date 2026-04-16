#![deny(dead_code, unused_imports)]

use std::future::Future;
use std::pin::Pin;

use elai_ipc_types::{ToolResultPayload, ValidateRequest, TOOL_VALIDATE};
use crate::ipc_registry::{ToolError, ToolHandler};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Supported language identifiers for validation.
const SUPPORTED_LANGUAGES: &[&str] = &["rust", "python", "typescript", "go"];

// ---------------------------------------------------------------------------
// ValidateHandler
// ---------------------------------------------------------------------------

pub struct ValidateHandler {
    project_root: std::path::PathBuf,
}

impl ValidateHandler {
    pub fn new(project_root: std::path::PathBuf) -> Self {
        Self { project_root }
    }

    fn validator_command(language: &str, file_path: &str) -> Option<(String, Vec<String>)> {
        match language {
            "rust" => {
                // For Rust, cargo check validates the whole crate; file_path is informational
                Some(("cargo".to_string(), vec!["check".to_string(), "--message-format=json".to_string()]))
            }
            "python" => Some(("ruff".to_string(), vec!["check".to_string(), "--output-format=json".to_string(), file_path.to_string()])),
            "typescript" => Some(("tsc".to_string(), vec!["--noEmit".to_string(), file_path.to_string()])),
            "go" => Some(("go".to_string(), vec!["vet".to_string(), file_path.to_string()])),
            _ => None,
        }
    }
}

impl ToolHandler for ValidateHandler {
    fn execute<'a>(&'a self, arguments: &'a [u8]) -> Pin<Box<dyn Future<Output = Result<ToolResultPayload, ToolError>> + Send + 'a>> {
        Box::pin(async move {
        let req: ValidateRequest =
            postcard::from_bytes(arguments).map_err(|e| ToolError::DeserializationFailed {
                tool_id: TOOL_VALIDATE.to_string(),
                reason: e.to_string(),
            })?;

        let (cmd, args) = Self::validator_command(&req.language, &req.file_path).ok_or_else(|| {
            ToolError::ExecutionFailed {
                tool_id: TOOL_VALIDATE.to_string(),
                reason: format!(
                    "unsupported language '{}', supported: {}",
                    req.language,
                    SUPPORTED_LANGUAGES.join(", ")
                ),
            }
        })?;

        let output = tokio::process::Command::new(&cmd)
            .args(&args)
            .current_dir(&self.project_root)
            .output()
            .await
            .map_err(|e| ToolError::SubprocessFailed {
                tool_id: TOOL_VALIDATE.to_string(),
                reason: e.to_string(),
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

        let mut diagnostics = Vec::new();
        if !stderr.is_empty() {
            diagnostics.push(stderr);
        }

        Ok(ToolResultPayload {
            output: stdout,
            diagnostics,
        })
        })
    }

    fn tool_id(&self) -> &str {
        TOOL_VALIDATE
    }

    fn is_write_tool(&self) -> bool {
        false
    }
}
