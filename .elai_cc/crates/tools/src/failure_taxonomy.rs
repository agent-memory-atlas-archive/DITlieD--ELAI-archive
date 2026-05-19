#![deny(dead_code, unused_imports)]

use std::future::Future;
use std::pin::Pin;

use elai_ipc_types::{ClassifyRequest, ToolResultPayload, TOOL_CLASSIFY};
use crate::ipc_registry::{ToolError, ToolHandler};
use serde::{Deserialize, Serialize};

/// 12-variant failure taxonomy. Original 9-list per B-20 plan body
/// (Rust-flavoured); extended by C-33 with LazyCode / EmptyBody /
/// FileSizeExceeded so the auditor can classify Zero-Lazy-Code and
/// file-size gate hits without falling back to RuntimePanic. Mirrored
/// verbatim by `elai_core::tool_dispatcher::ToolFailureCategory` so the
/// two enums never drift. Decision A (B-20A): the B-20 plan list is
/// canonical; phase-4B.txt's competing 9-list (PERMISSION_DENIED,
/// TIMEOUT, RESOURCE_EXHAUSTED, EMPTY_OUTPUT) is documented in
/// workflow/reasoning/B-20A-decisions.md and rejected for that phase.
/// C-33 extension documented in workflow/reasoning/C-33-decisions.md.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FailureCategory {
    SyntaxError,
    TypeMismatch,
    UndefinedSymbol,
    MissingImport,
    LifetimeError,
    BorrowError,
    UnusedWarning,
    TestAssertion,
    RuntimePanic,
    /// C-33: placeholder return / TODO body / pass body / mock data in prod.
    LazyCode,
    /// C-33: function or impl block with no statements.
    EmptyBody,
    /// C-33: file exceeds configured max-lines gate.
    FileSizeExceeded,
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

    /// Classify an error message into the 12-variant taxonomy. Order is
    /// load-bearing: more specific patterns must precede generic ones so
    /// e.g. "unresolved import" maps to MissingImport (not UndefinedSymbol).
    /// C-33 patterns (LazyCode / EmptyBody / FileSizeExceeded) precede the
    /// RuntimePanic fallback so a TODO body never gets swallowed as a panic.
    fn detect_category(output: &str) -> FailureCategory {
        // Imports first — most specific Rust diagnostic.
        if output.contains("unresolved import")
            || output.contains("error[E0432]")
            || output.contains("No module named")
            || output.contains("ModuleNotFoundError")
        {
            return FailureCategory::MissingImport;
        }
        if output.contains("explicit lifetime required")
            || output.contains("error[E0621]")
            || output.contains("missing lifetime specifier")
            || output.contains("error[E0106]")
            || output.contains("lifetime may not live long enough")
        {
            return FailureCategory::LifetimeError;
        }
        if output.contains("borrow of moved value")
            || output.contains("error[E0382]")
            || output.contains("cannot borrow")
            || output.contains("error[E0502]")
            || output.contains("error[E0499]")
        {
            return FailureCategory::BorrowError;
        }
        if output.contains("mismatched types")
            || output.contains("error[E0308]")
            || output.contains("expected type")
        {
            return FailureCategory::TypeMismatch;
        }
        if output.contains("cannot find value")
            || output.contains("cannot find function")
            || output.contains("cannot find type")
            || output.contains("error[E0425]")
            || output.contains("error[E0412]")
            || output.contains("error[E0422]")
        {
            return FailureCategory::UndefinedSymbol;
        }
        if (output.contains("error: expected") && !output.contains("expected type"))
            || output.contains("expected `;`")
            || output.contains("unexpected token")
            || output.contains("invalid syntax")
            || output.contains("SyntaxError")
        {
            return FailureCategory::SyntaxError;
        }
        if output.contains("test result: FAILED")
            || output.contains("assertion failed")
            || output.contains("panicked at 'assertion")
        {
            return FailureCategory::TestAssertion;
        }
        if output.contains("warning:")
            && (output.contains("unused")
                || output.contains("dead_code")
                || output.contains("unused_variables"))
        {
            return FailureCategory::UnusedWarning;
        }
        // C-33: Zero-Lazy-Code patterns. The auditor / Stop hook emits these
        // strings when scanning a freshly written function. Kept BEFORE the
        // RuntimePanic fallback so a `todo!()` marker is never swallowed as
        // a panic message even when the same output also contains a panic
        // banner (covered by c33_phase_map_contract once mixed-input fixtures
        // are added).
        if output.contains("TODO")
            || output.contains("placeholder")
            || output.contains("unimplemented!")
            || output.contains("todo!()")
        {
            return FailureCategory::LazyCode;
        }
        if output.contains("empty function body")
            || output.contains("no statements")
        {
            return FailureCategory::EmptyBody;
        }
        if output.contains("file exceeds")
            || output.contains("FILE_SIZE")
        {
            return FailureCategory::FileSizeExceeded;
        }
        if output.contains("panicked at")
            || output.contains("thread '")
            || output.contains("RuntimeError")
            || output.contains("timeout")
            || output.contains("timed out")
            || output.contains("out of memory")
            || output.contains("OOM")
        {
            return FailureCategory::RuntimePanic;
        }
        // Default: treat as RuntimePanic (was RuntimeError in the old 13-list).
        FailureCategory::RuntimePanic
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
            FailureCategory::SyntaxError => "fix syntax error (missing token / malformed expression)".to_string(),
            FailureCategory::TypeMismatch => "align argument or return types".to_string(),
            FailureCategory::UndefinedSymbol => "declare or import the missing symbol".to_string(),
            FailureCategory::MissingImport => "add the missing import / use statement".to_string(),
            FailureCategory::LifetimeError => "annotate or extend lifetimes so references stay valid".to_string(),
            FailureCategory::BorrowError => "restructure to avoid the move/aliasing conflict".to_string(),
            FailureCategory::UnusedWarning => "remove dead code or prefix unused name with `_`".to_string(),
            FailureCategory::TestAssertion => "investigate test assertion failures".to_string(),
            FailureCategory::RuntimePanic => "check for panics / runtime errors / timeouts / OOM".to_string(),
            FailureCategory::LazyCode => {
                "replace placeholder / TODO / unimplemented body with real logic; CLAUDE.md Zero-Lazy-Code rule".to_string()
            }
            FailureCategory::EmptyBody => {
                "implement the function body or remove the empty declaration".to_string()
            }
            FailureCategory::FileSizeExceeded => {
                "split the file or extend the configured max-lines gate".to_string()
            }
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
