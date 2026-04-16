#![deny(dead_code, unused_imports)]

use std::future::Future;
use std::pin::Pin;

use elai_ipc_types::{ClassifyRequest, ToolResultPayload, TOOL_CLASSIFY};
use crate::ipc_registry::{ToolError, ToolHandler};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FailureCategory {
    CompilationError,
    TestFailure,
    RuntimeError,
    Timeout,
    ResourceExhausted,
    LazyCode,
    EmptyBody,
    FileSizeExceeded,
    ContextExplosion,
    RawLogOverload,
    PhaseGateFailure,
    ScriptQualityDependency,
    SpeculativeRepair,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RootCauseKind {
    MissingImport,
    WrongType,
    LogicError,
    WrongAlgorithm,
    MissingEdgeCase,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootCause {
    pub kind: RootCauseKind,
    pub source_symbol: String,
    pub expected: String,
    pub actual: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticResult {
    pub category: FailureCategory,
    pub location: String,
    pub root_cause: Option<RootCause>,
    pub fix_instruction: String,
    pub confidence: u8,
    pub escalate: bool,
}

pub struct ClassifyHandler;

impl ClassifyHandler {
    pub fn new() -> Self {
        Self
    }

    fn classify_error_output(error_output: &str) -> DiagnosticResult {
        let category = Self::detect_category(error_output);
        let root_cause = Self::detect_root_cause(error_output);
        let confidence = if root_cause.is_some() { 80 } else { 40 };
        let escalate = confidence < 50;

        DiagnosticResult {
            category,
            location: Self::extract_location(error_output),
            root_cause,
            fix_instruction: Self::suggest_fix(&category),
            confidence,
            escalate,
        }
    }

    fn detect_category(output: &str) -> FailureCategory {
        if output.contains("todo!()") || output.contains("unimplemented!()") {
            return FailureCategory::LazyCode;
        }
        if output.contains("empty body") || output.contains("pass\n") {
            return FailureCategory::EmptyBody;
        }
        if output.contains("file size exceeded") || output.contains("too large") {
            return FailureCategory::FileSizeExceeded;
        }
        if output.contains("error[E") || output.contains("cannot find") {
            return FailureCategory::CompilationError;
        }
        if output.contains("FAILED") || output.contains("assertion") {
            return FailureCategory::TestFailure;
        }
        if output.contains("panic") || output.contains("thread") {
            return FailureCategory::RuntimeError;
        }
        if output.contains("timeout") || output.contains("timed out") {
            return FailureCategory::Timeout;
        }
        if output.contains("out of memory") || output.contains("OOM") {
            return FailureCategory::ResourceExhausted;
        }
        FailureCategory::RuntimeError
    }

    fn detect_root_cause(output: &str) -> Option<RootCause> {
        if output.contains("unresolved import") || output.contains("No module named") {
            let symbol = output
                .lines()
                .find(|l| l.contains("import"))
                .unwrap_or("unknown")
                .to_string();
            return Some(RootCause {
                kind: RootCauseKind::MissingImport,
                source_symbol: symbol,
                expected: "valid import".to_string(),
                actual: "unresolved".to_string(),
            });
        }
        if output.contains("mismatched types") || output.contains("expected") {
            return Some(RootCause {
                kind: RootCauseKind::WrongType,
                source_symbol: String::new(),
                expected: "matching types".to_string(),
                actual: "type mismatch".to_string(),
            });
        }
        None
    }

    fn extract_location(output: &str) -> String {
        for line in output.lines() {
            if let Some(pos) = line.find(".rs:") {
                let start = line[..pos].rfind(' ').map_or(0, |i| i + 1);
                let end = line[pos + 4..]
                    .find(|c: char| !c.is_ascii_digit() && c != ':')
                    .map_or(line.len(), |i| pos + 4 + i);
                return line[start..end].to_string();
            }
            if let Some(pos) = line.find(".py:") {
                let start = line[..pos].rfind(' ').map_or(0, |i| i + 1);
                let end = line[pos + 4..]
                    .find(|c: char| !c.is_ascii_digit() && c != ':')
                    .map_or(line.len(), |i| pos + 4 + i);
                return line[start..end].to_string();
            }
        }
        "unknown".to_string()
    }

    fn suggest_fix(category: &FailureCategory) -> String {
        match category {
            FailureCategory::LazyCode => "replace todo!()/unimplemented!() with real implementation".to_string(),
            FailureCategory::EmptyBody => "implement function body".to_string(),
            FailureCategory::FileSizeExceeded => "split file into smaller modules".to_string(),
            FailureCategory::CompilationError => "fix compilation errors".to_string(),
            FailureCategory::TestFailure => "investigate test assertion failures".to_string(),
            FailureCategory::RuntimeError => "check for panics and runtime errors".to_string(),
            FailureCategory::Timeout => "optimize or increase timeout".to_string(),
            FailureCategory::ResourceExhausted => "reduce memory usage or increase limits".to_string(),
            FailureCategory::ContextExplosion => "reduce trace depth before adaptive reduction".to_string(),
            FailureCategory::RawLogOverload => "apply TLSA transformation to logs".to_string(),
            FailureCategory::PhaseGateFailure => "fix phase gate script".to_string(),
            FailureCategory::ScriptQualityDependency => "improve reproduction script".to_string(),
            FailureCategory::SpeculativeRepair => "ground repair in runtime evidence".to_string(),
        }
    }
}

impl Default for ClassifyHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolHandler for ClassifyHandler {
    fn execute<'a>(&'a self, arguments: &'a [u8]) -> Pin<Box<dyn Future<Output = Result<ToolResultPayload, ToolError>> + Send + 'a>> {
        Box::pin(async move {
            let req: ClassifyRequest =
                postcard::from_bytes(arguments).map_err(|e| ToolError::DeserializationFailed {
                    tool_id: TOOL_CLASSIFY.to_string(),
                    reason: e.to_string(),
                })?;

            let diagnostic = Self::classify_error_output(&req.error_output);
            let output = serde_json::to_string(&diagnostic).map_err(|e| ToolError::ExecutionFailed {
                tool_id: TOOL_CLASSIFY.to_string(),
                reason: e.to_string(),
            })?;

            let mut diagnostics = Vec::new();
            if diagnostic.escalate {
                diagnostics.push("ESCALATE: low confidence classification".to_string());
            }

            Ok(ToolResultPayload {
                output,
                diagnostics,
            })
        })
    }

    fn tool_id(&self) -> &str {
        TOOL_CLASSIFY
    }

    fn is_write_tool(&self) -> bool {
        false
    }
}
