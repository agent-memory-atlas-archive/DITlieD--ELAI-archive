#![deny(dead_code, unused_imports)]

use std::future::Future;
use std::pin::Pin;

use elai_ipc_types::{
    GitBranchRequest, GitCommitRequest, GitUndoRequest, ToolResultPayload,
    TOOL_GIT_BRANCH, TOOL_GIT_COMMIT, TOOL_GIT_UNDO,
};
use crate::ipc_registry::{ToolError, ToolHandler};

// ---------------------------------------------------------------------------
// GitCommitHandler
// ---------------------------------------------------------------------------

pub struct GitCommitHandler {
    project_root: std::path::PathBuf,
}

impl GitCommitHandler {
    pub fn new(project_root: std::path::PathBuf) -> Self {
        Self { project_root }
    }

    async fn run_git(&self, args: &[&str]) -> Result<String, ToolError> {
        let output = tokio::process::Command::new("git")
            .args(args)
            .current_dir(&self.project_root)
            .output()
            .await
            .map_err(|e| ToolError::SubprocessFailed {
                tool_id: TOOL_GIT_COMMIT.to_string(),
                reason: e.to_string(),
            })?;
        if output.status.success() {
            String::from_utf8(output.stdout).map_err(|e| ToolError::ExecutionFailed {
                tool_id: TOOL_GIT_COMMIT.to_string(),
                reason: e.to_string(),
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(ToolError::ExecutionFailed {
                tool_id: TOOL_GIT_COMMIT.to_string(),
                reason: stderr.into_owned(),
            })
        }
    }
}

impl ToolHandler for GitCommitHandler {
    fn execute<'a>(&'a self, arguments: &'a [u8]) -> Pin<Box<dyn Future<Output = Result<ToolResultPayload, ToolError>> + Send + 'a>> {
        Box::pin(async move {
            let req: GitCommitRequest =
                postcard::from_bytes(arguments).map_err(|e| ToolError::DeserializationFailed {
                    tool_id: TOOL_GIT_COMMIT.to_string(),
                    reason: e.to_string(),
                })?;

            for path in &req.paths {
                self.run_git(&["add", path]).await?;
            }

            let output = self.run_git(&["commit", "-m", &req.message]).await?;

            Ok(ToolResultPayload {
                output,
                diagnostics: Vec::new(),
            })
        })
    }

    fn tool_id(&self) -> &str {
        TOOL_GIT_COMMIT
    }

    fn is_write_tool(&self) -> bool {
        true
    }
}

// ---------------------------------------------------------------------------
// GitUndoHandler
// ---------------------------------------------------------------------------

pub struct GitUndoHandler {
    project_root: std::path::PathBuf,
}

impl GitUndoHandler {
    pub fn new(project_root: std::path::PathBuf) -> Self {
        Self { project_root }
    }
}

impl ToolHandler for GitUndoHandler {
    fn execute<'a>(&'a self, arguments: &'a [u8]) -> Pin<Box<dyn Future<Output = Result<ToolResultPayload, ToolError>> + Send + 'a>> {
        Box::pin(async move {
            let req: GitUndoRequest =
                postcard::from_bytes(arguments).map_err(|e| ToolError::DeserializationFailed {
                    tool_id: TOOL_GIT_UNDO.to_string(),
                    reason: e.to_string(),
                })?;

            let output = tokio::process::Command::new("git")
                .args(["revert", "--no-edit", &req.commit_hash])
                .current_dir(&self.project_root)
                .output()
                .await
                .map_err(|e| ToolError::SubprocessFailed {
                    tool_id: TOOL_GIT_UNDO.to_string(),
                    reason: e.to_string(),
                })?;

            if output.status.success() {
                Ok(ToolResultPayload {
                    output: String::from_utf8_lossy(&output.stdout).into_owned(),
                    diagnostics: Vec::new(),
                })
            } else {
                Err(ToolError::ExecutionFailed {
                    tool_id: TOOL_GIT_UNDO.to_string(),
                    reason: String::from_utf8_lossy(&output.stderr).into_owned(),
                })
            }
        })
    }

    fn tool_id(&self) -> &str {
        TOOL_GIT_UNDO
    }

    fn is_write_tool(&self) -> bool {
        true
    }
}

// ---------------------------------------------------------------------------
// GitBranchHandler
// ---------------------------------------------------------------------------

pub struct GitBranchHandler {
    project_root: std::path::PathBuf,
}

impl GitBranchHandler {
    pub fn new(project_root: std::path::PathBuf) -> Self {
        Self { project_root }
    }
}

impl ToolHandler for GitBranchHandler {
    fn execute<'a>(&'a self, arguments: &'a [u8]) -> Pin<Box<dyn Future<Output = Result<ToolResultPayload, ToolError>> + Send + 'a>> {
        Box::pin(async move {
            let req: GitBranchRequest =
                postcard::from_bytes(arguments).map_err(|e| ToolError::DeserializationFailed {
                    tool_id: TOOL_GIT_BRANCH.to_string(),
                    reason: e.to_string(),
                })?;

            let mut args = vec!["checkout", "-b", &req.branch_name];
            let from_ref_str;
            if let Some(ref from_ref) = req.from_ref {
                from_ref_str = from_ref.clone();
                args.push(&from_ref_str);
            }

            let output = tokio::process::Command::new("git")
                .args(&args)
                .current_dir(&self.project_root)
                .output()
                .await
                .map_err(|e| ToolError::SubprocessFailed {
                    tool_id: TOOL_GIT_BRANCH.to_string(),
                    reason: e.to_string(),
                })?;

            if output.status.success() {
                Ok(ToolResultPayload {
                    output: String::from_utf8_lossy(&output.stdout).into_owned(),
                    diagnostics: Vec::new(),
                })
            } else {
                Err(ToolError::ExecutionFailed {
                    tool_id: TOOL_GIT_BRANCH.to_string(),
                    reason: String::from_utf8_lossy(&output.stderr).into_owned(),
                })
            }
        })
    }

    fn tool_id(&self) -> &str {
        TOOL_GIT_BRANCH
    }

    fn is_write_tool(&self) -> bool {
        true
    }
}
