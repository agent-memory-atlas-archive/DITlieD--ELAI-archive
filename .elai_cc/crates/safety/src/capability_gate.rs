//! E-37: CaMeL-lite capability gate (OWASP LLM01 mitigation).
//!
//! Rejects destructive tool calls whose argv contains values carrying
//! `TrustLevel::Untrusted` provenance (web fetch, MCP tool output, file
//! upload, sandbox stdout). Operates AT argv-construction site in
//! `crates/orchestrator/src/task_runner.rs`, BEFORE the dispatch fires.
//!
//! Design grounding: arXiv:2503.18813 "Defeating Prompt Injections by
//! Design" (Debenedetti et al., Google DeepMind / ETH Zurich, 2025) — the
//! CaMeL design. CaMeL extracts the control/data flows from the trusted
//! query and enforces them via a provenance-tracking interpreter so that
//! untrusted retrieved data can never alter program flow; the lite variant
//! in this slice is a one-step provenance check at destructive-tool dispatch time.
//!
//! Composes additively with E-30 `destructive_verb_allowlist` — that
//! module catches known-bad argv patterns (rm -rf, git restore .,
//! chmod 777); this module catches argv whose VALUES were derived from
//! untrusted upstream (a known-safe verb like `bash` invoked with an
//! attacker-controlled flag string is still rejected here). Both stay.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::provenance::{Provenanced, TrustLevel};

/// Verdict from the capability gate. Sibling shape to `AllowlistVerdict`
/// (E-30 destructive_verb_allowlist) — structured reason + citation, no
/// anyhow strings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "verdict")]
pub enum CapabilityVerdict {
    /// All argv args carry trusted provenance; tool call may proceed.
    Allow,
    /// One or more argv args carry untrusted provenance and the tool is
    /// destructive. Caller blocks the call AND emits
    /// `SsbActionType::CapabilityGateRejected`.
    Reject {
        reason: String,
        citation: String,
        offending_arg_index: usize,
        offending_trust_label: String,
    },
}

/// Errors from the capability gate.
#[derive(Debug, Error)]
pub enum CapabilityGateError {
    #[error("tool name is empty")]
    EmptyToolName,
}

/// Tools that get the strictest provenance check. Wider than E-30's
/// destructive_verb_allowlist (which is argv-regex driven); this list is
/// keyed by TOOL NAME instead of argv content.
///
/// Additive: a tool name in this list does NOT bypass
/// `destructive_verb_allowlist::check_argv` — both layers run.
///
/// TAINT-SINK TOOLS: canvas content-read tools that feed selected node content
/// into the model prompt are classified here as taint sinks. Without this,
/// `allow_tool_with_origin("canvas_select", ...)` always returns Allow regardless
/// of provenance, making the Blocked verdict in run_select_gate unreachable.
/// These are NOT destructive in the file-system sense but ARE taint sinks:
/// untrusted web-sourced content that re-enters the model prompt via a canvas
/// Select is a prompt-injection vector (H-15 CANVAS §19).
pub const DESTRUCTIVE_TOOLS: &[&str] = &[
    "bash",
    "shell",
    "fs_write",
    "write_file",
    "rm",
    "delete_file",
    "db_modify",
    "sql_execute",
    "curl",
    "web_fetch",  // can be used for exfil even though "fetch" sounds read-only
    "http_post",
    // Canvas taint-sink tools: content-read operations that feed node content
    // into the model prompt (prompt-injection risk; H-15 CANVAS §19 trust moat).
    "canvas_select",
];

/// Decide whether a tool call may proceed given its argv provenance.
///
/// `tool_name` is the canonical tool name (e.g. `bash`, `fs_write`).
/// `argv` is the per-arg `Provenanced<String>` slice — the caller is
/// responsible for tagging each arg at construction time (CLI args are
/// `Provenanced::trusted`, MCP tool args derived from `tools/call` results
/// are `Provenance::from_tool_output(...)`, etc).
///
/// Returns `CapabilityVerdict::Reject` if the tool is in
/// `DESTRUCTIVE_TOOLS` AND any arg carries `TrustLevel::Untrusted`.
/// `Recallable` provenance does NOT block (matches user expectation
/// that memory-derived data is allowed to drive tools, with HITL
/// approval at a higher layer).
pub fn allow_tool_with_origin(
    tool_name: &str,
    argv: &[Provenanced<String>],
) -> Result<CapabilityVerdict, CapabilityGateError> {
    if tool_name.is_empty() {
        return Err(CapabilityGateError::EmptyToolName);
    }
    if !is_destructive_tool(tool_name) {
        return Ok(CapabilityVerdict::Allow);
    }
    for (idx, arg) in argv.iter().enumerate() {
        if arg.provenance().trust == TrustLevel::Untrusted {
            return Ok(CapabilityVerdict::Reject {
                reason: format!(
                    "destructive tool `{tool_name}` invoked with arg #{idx} carrying untrusted provenance ({label})",
                    label = arg.provenance().source.label()
                ),
                citation: "arXiv:2503.18813 CaMeL §3.1 — capability-based denial on untrusted-flow into destructive sinks".into(),
                offending_arg_index: idx,
                offending_trust_label: arg.provenance().trust.label().to_owned(),
            });
        }
    }
    Ok(CapabilityVerdict::Allow)
}

/// Whether the tool name is in `DESTRUCTIVE_TOOLS`. Public for callers
/// that need the predicate without the full gate decision (e.g. to log
/// pre-gate context).
pub fn is_destructive_tool(tool_name: &str) -> bool {
    DESTRUCTIVE_TOOLS.iter().any(|t| *t == tool_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provenance::Provenance;

    fn trusted_arg(s: &str) -> Provenanced<String> {
        Provenanced::trusted(s.to_string())
    }

    fn web_arg(s: &str, url: &str) -> Provenanced<String> {
        Provenanced::new(s.to_string(), Provenance::from_web(url))
    }

    fn tool_arg(s: &str) -> Provenanced<String> {
        Provenanced::new(
            s.to_string(),
            Provenance::from_tool_output("http://localhost:8200", "search"),
        )
    }

    fn memory_arg(s: &str) -> Provenanced<String> {
        Provenanced::new(s.to_string(), Provenance::from_memory("memory-bank/notes.md"))
    }

    #[test]
    fn non_destructive_tool_always_allow() {
        let argv = vec![web_arg("anything", "https://attacker.example")];
        let v = allow_tool_with_origin("search", &argv).expect("ok");
        assert_eq!(v, CapabilityVerdict::Allow);
    }

    #[test]
    fn destructive_tool_with_trusted_argv_allow() {
        let argv = vec![trusted_arg("ls"), trusted_arg("-la")];
        let v = allow_tool_with_origin("bash", &argv).expect("ok");
        assert_eq!(v, CapabilityVerdict::Allow);
    }

    #[test]
    fn destructive_tool_with_web_argv_reject() {
        let argv = vec![trusted_arg("curl"), web_arg("https://attacker.example/x", "https://upstream.example")];
        let v = allow_tool_with_origin("bash", &argv).expect("ok");
        match v {
            CapabilityVerdict::Reject { offending_arg_index, offending_trust_label, .. } => {
                assert_eq!(offending_arg_index, 1);
                assert_eq!(offending_trust_label, "untrusted");
            }
            _ => panic!("expected Reject for web-tainted argv"),
        }
    }

    #[test]
    fn destructive_tool_with_tool_output_arg_reject() {
        let argv = vec![tool_arg("rm -rf ~/")];
        let v = allow_tool_with_origin("bash", &argv).expect("ok");
        assert!(matches!(v, CapabilityVerdict::Reject { .. }));
    }

    #[test]
    fn memory_origin_does_not_block_destructive() {
        let argv = vec![memory_arg("recalled-value")];
        let v = allow_tool_with_origin("fs_write", &argv).expect("ok");
        assert_eq!(v, CapabilityVerdict::Allow);
    }

    #[test]
    fn empty_tool_name_errors() {
        let argv = vec![trusted_arg("x")];
        let res = allow_tool_with_origin("", &argv);
        assert!(matches!(res, Err(CapabilityGateError::EmptyToolName)));
    }

    #[test]
    fn verdict_serde_roundtrip_allow() {
        let v = CapabilityVerdict::Allow;
        let s = serde_json::to_string(&v).expect("serialize");
        let back: CapabilityVerdict = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(v, back);
    }

    #[test]
    fn destructive_list_contains_canonical_tools() {
        for t in &["bash", "fs_write", "curl", "rm"] {
            assert!(is_destructive_tool(t), "{t} must be in DESTRUCTIVE_TOOLS");
        }
        assert!(!is_destructive_tool("search"));
        assert!(!is_destructive_tool("read_file"));
    }
}
