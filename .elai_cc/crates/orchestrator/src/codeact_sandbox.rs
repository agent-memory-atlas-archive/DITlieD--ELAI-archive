//! E-35 — CodeAct sandbox scaffolding.
//!
//! Thin Runner-role-FREE dispatch wrapper around the existing
//! `crates/sandbox` substrate. Composes at the boundary: a caller hands
//! the dispatcher a typed `CodeActAction` (the wire frame already shipped
//! at `elai_ipc_types::lib.rs:842`); the dispatcher performs cap +
//! allowlist guards, then either dispatches to an injected
//! `CodeActSandbox` impl OR returns a typed `RunpodPending` outcome when
//! no sandbox is available in the current pod.
//!
//! ## Why no AgentRole::Runner
//!
//! Adding a `Runner` variant to `AgentRole` triggers a ~230 match-arm
//! cascade across `agent_pool.rs`, `agent_role_dispatcher.rs`, the
//! dashboard typescript mirror, etc — the same anti-pattern documented
//! at gotchas.md E41_AGENTROLE_FROZEN_AT_17. CodeAct composes through
//! the existing role + RuntimeBackend composition: any role that needs
//! a sandboxed Python snippet calls `execute_code_act` directly. Full
//! CodeAct adoption (Runner role + match-arm fan-out) is deferred to
//! E-35.D patch-N (per session-2-handoff cluster).
//!
//! ## Wire shape
//!
//! Input: `CodeActAction { python_code, tool_allowlist }` (capped at
//! `CODEACT_MAX_LINES = 200` per ipc-types contract).
//!
//! Output: `CodeActOutcome` (typed enum: Executed{stdout_excerpt,
//! exit_code, duration_ms} | SandboxAbsent{detail} | CapExceeded{
//! observed, cap} | AllowlistViolation{tool_id_hash}). Emits one
//! `SsbActionType::CodeactActionExecuted` row per dispatch regardless of
//! outcome (per WIRING_CONTRACT::observable_effect rule).
//!
//! ## Inline cite
//!
//! CodeAct paper: arXiv:2402.01030 (Wang et al., "Executable Code
//! Actions Elicit Better LLM Agents", ICML 2024) — ingested at
//! `workflow/papers/codeact-2402.01030-interface.md`. The 200-line cap
//! tracks the paper's `MaxAction` knob (Sec 4.2) which they fix at 200
//! tokens; we use 200 LINES as a conservative byte-equivalent ceiling
//! per CLAUDE.md anti-stale (counter-evidence not measured yet).
//!
//! ## Composition order (when sandbox is present)
//!
//! 1. Pre-guard: line-count check (max 200 per `CODEACT_MAX_LINES`).
//! 2. Pre-guard: tool_allowlist enforcement (no import outside list).
//! 3. Dispatch: invoke the injected `CodeActSandbox::run(action)`.
//! 4. Post: emit `CodeactActionExecuted` SSB row with outcome hash +
//!    duration_ms.
//!
//! Per CLAUDE.md: no `.unwrap()` / `.expect()` outside `#[cfg(test)]`;
//! native async fn in traits (NEVER async-trait macro); blake3 for any
//! hashing (NEVER blake2/sha2).

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use elai_ipc_types::{CodeActAction as IpcCodeActAction, ToolId, CODEACT_MAX_LINES};

use crate::ssb::{CodeactActionExecutedPayload, SsbActionType, SsbPayload};
use crate::Ssb;

/// Re-export of [`elai_ipc_types::CodeActAction`] so downstream callers
/// (e.g. `crates/cli/src/structured_output_cmd.rs`) can refer to the
/// wire frame without taking a separate `elai-ipc-types` dep. The wire
/// shape is owned by ipc-types per CLAUDE.md cross-language pair
/// convention; this re-export is a convenience anchor.
pub type CodeActAction = IpcCodeActAction;

/// Re-export of [`elai_ipc_types::ToolId`] for the same reason.
pub type CodeActToolId = ToolId;

/// Typed outcome of a CodeAct dispatch. Mirrors the
/// `CodeactActionExecuted` SSB payload's `outcome_hash` discriminator
/// space so the host-side row and the typed return are kept in lockstep.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeActOutcome {
    /// Sandbox produced a result. `stdout_excerpt` is capped at 4 KiB
    /// to avoid log blowup; the full body lives in the sandbox journal.
    Executed {
        stdout_excerpt: String,
        exit_code: i32,
        duration_ms: u32,
    },
    /// No CodeAct sandbox is wired in the current pod (e.g. host-side
    /// orchestrator running without a sidecar / RunPod boundary). This
    /// is the canonical RUNPOD-PENDING return — NOT an error, NOT a
    /// silent pass.
    SandboxAbsent { detail: String },
    /// Snippet exceeded `CODEACT_MAX_LINES`. Pre-guard rejects before
    /// any sandbox call. Refuse loud per CodeAct paper Sec 4.2 cap.
    CapExceeded { observed: u32, cap: u32 },
    /// Snippet imported a tool not in `tool_allowlist`. Pre-guard
    /// rejects before any sandbox call.
    AllowlistViolation { tool_id_hash: u64 },
}

impl CodeActOutcome {
    /// Stable snake_case label for the outcome — used to derive the
    /// `outcome_hash` field on `CodeactActionExecutedPayload`. Keep in
    /// lockstep with the SSB CHECK constraint values referenced in
    /// `ssb.rs` `codeact_action_executed` comment.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Executed { .. } => "executed",
            Self::SandboxAbsent { .. } => "sandbox_absent",
            Self::CapExceeded { .. } => "cap_exceeded",
            Self::AllowlistViolation { .. } => "allowlist_violation",
        }
    }
}

/// Pluggable CodeAct sandbox surface. The production impl lives in the
/// `crates/sandbox` substrate (firecracker / WASM / subprocess); this
/// trait is the boundary the orchestrator dispatches through so unit
/// tests can swap in a deterministic mock.
///
/// CLAUDE.md anti-stale: native async fn in traits via manual
/// `Pin<Box<Future>>` desugar (NEVER async-trait macro).
pub trait CodeActSandbox: Send + Sync {
    fn run<'a>(
        &'a self,
        action: &'a CodeActAction,
    ) -> Pin<Box<dyn Future<Output = CodeActOutcome> + Send + 'a>>;
}

/// blake3-derived id for a CodeAct snippet (first 8 bytes of digest).
/// Used as `action_hash` on the SSB payload so operators can correlate
/// the same snippet across dispatches without storing plaintext.
fn action_hash_of(action: &CodeActAction) -> u64 {
    let mut hasher = blake3::Hasher::new();
    hasher.update(action.python_code.as_bytes());
    hasher.update(b"\n");
    for t in &action.tool_allowlist {
        hasher.update(t.as_str().as_bytes());
        hasher.update(b",");
    }
    let digest = hasher.finalize();
    let bytes = digest.as_bytes();
    u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5], bytes[6], bytes[7],
    ])
}

fn outcome_hash_of(outcome: &CodeActOutcome) -> u64 {
    let label = outcome.as_str();
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"codeact.outcome.");
    hasher.update(label.as_bytes());
    let digest = hasher.finalize();
    let bytes = digest.as_bytes();
    u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5], bytes[6], bytes[7],
    ])
}

fn line_count(code: &str) -> u32 {
    let n = code.lines().count();
    u32::try_from(n).unwrap_or(u32::MAX)
}

fn count_imports_outside_allowlist(
    code: &str,
    allow: &[ToolId],
) -> Option<String> {
    // Cheap pre-flight: scan for `import X` / `from X import` whose
    // top-level module is not in the allowlist. This is NOT a Python
    // parser — the production sandbox is the authoritative gate; this
    // is a fast-path refusal so we never even dispatch obviously bad
    // snippets.
    for raw in code.lines() {
        let trimmed = raw.trim_start();
        let module = if let Some(rest) = trimmed.strip_prefix("import ") {
            rest.split([' ', ',', '\t']).next().unwrap_or("").trim()
        } else if let Some(rest) = trimmed.strip_prefix("from ") {
            rest.split([' ', '\t']).next().unwrap_or("").trim()
        } else {
            continue;
        };
        if module.is_empty() {
            continue;
        }
        let top = module.split('.').next().unwrap_or(module);
        if !allow.iter().any(|a| a.as_str() == top) {
            return Some(top.to_string());
        }
    }
    None
}

/// Dispatch a CodeAct snippet. Returns a typed outcome AND (when an
/// `ssb` writer is supplied) emits a `CodeactActionExecuted` SSB row
/// with the outcome discriminator hashed into `outcome_hash`.
///
/// `now_ms` is injected so the caller can use a deterministic
/// `ClockProvider` (never `std::time::SystemTime` per the entropy
/// quarantine rule).
///
/// `sandbox` is optional: when `None`, the dispatcher short-circuits
/// with `CodeActOutcome::SandboxAbsent` (canonical RUNPOD-PENDING).
pub async fn execute_code_act(
    action: &CodeActAction,
    sandbox: Option<&dyn CodeActSandbox>,
    ssb: Option<&Arc<Ssb>>,
    now_ms: u64,
) -> CodeActOutcome {
    let lines = line_count(&action.python_code);
    let outcome: CodeActOutcome = if lines > CODEACT_MAX_LINES as u32 {
        CodeActOutcome::CapExceeded {
            observed: lines,
            cap: CODEACT_MAX_LINES as u32,
        }
    } else if let Some(tool) =
        count_imports_outside_allowlist(&action.python_code, &action.tool_allowlist)
    {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"codeact.tool.");
        hasher.update(tool.as_bytes());
        let bytes = hasher.finalize();
        let h = bytes.as_bytes();
        let tool_id_hash = u64::from_le_bytes([
            h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7],
        ]);
        CodeActOutcome::AllowlistViolation { tool_id_hash }
    } else {
        match sandbox {
            Some(sb) => sb.run(action).await,
            None => CodeActOutcome::SandboxAbsent {
                detail: "no CodeAct sandbox wired in current pod (RUNPOD-PENDING)"
                    .to_string(),
            },
        }
    };

    if let Some(ssb_writer) = ssb {
        let duration_ms = match &outcome {
            CodeActOutcome::Executed { duration_ms, .. } => *duration_ms,
            _ => 0,
        };
        let payload = SsbPayload::CodeactActionExecuted(CodeactActionExecutedPayload {
            action_hash: action_hash_of(action),
            line_count: lines,
            outcome_hash: outcome_hash_of(&outcome),
            duration_ms,
            ts_ms: now_ms,
        });
        // Best-effort emission: the dispatcher MUST NOT fail callers if
        // SSB write fails (the row is observability, not part of the
        // CodeAct contract). Log + drop on error.
        if let Err(err) = ssb_writer
            .write_payload("codeact_sandbox", 0, payload, Vec::new())
            .await
        {
            tracing::warn!(
                target: "codeact_sandbox",
                error = %err,
                "failed to emit CodeactActionExecuted SSB row"
            );
        }
    }

    // Sanity: SsbActionType ensures the payload variant exists.
    let _ = SsbActionType::CodeactActionExecuted;
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_action(code: &str, allow: &[&str]) -> CodeActAction {
        let action_hash = {
            let mut hasher = blake3::Hasher::new();
            hasher.update(code.as_bytes());
            format!("blake3:{}", hasher.finalize().to_hex())
        };
        CodeActAction {
            python_code: code.to_string(),
            tool_allowlist: allow.iter().map(|s| ToolId::new(*s)).collect(),
            action_hash,
        }
    }

    #[tokio::test]
    async fn sandbox_absent_when_none_injected() {
        let action = mk_action("print('ok')", &["builtins"]);
        let outcome = execute_code_act(&action, None, None, 1_700_000_000_000).await;
        assert!(matches!(outcome, CodeActOutcome::SandboxAbsent { .. }));
        assert_eq!(outcome.as_str(), "sandbox_absent");
    }

    #[tokio::test]
    async fn cap_exceeded_refuses_loud() {
        let big = vec!["x = 1"; (CODEACT_MAX_LINES as usize) + 5].join("\n");
        let action = mk_action(&big, &[]);
        let outcome = execute_code_act(&action, None, None, 1_700_000_000_001).await;
        match outcome {
            CodeActOutcome::CapExceeded { observed, cap } => {
                assert_eq!(cap, CODEACT_MAX_LINES as u32);
                assert!(observed > cap);
            }
            other => panic!("expected CapExceeded, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn allowlist_violation_refuses_loud() {
        let code = "import os\nprint(os.getcwd())";
        let action = mk_action(code, &["json"]);
        let outcome = execute_code_act(&action, None, None, 1_700_000_000_002).await;
        assert!(matches!(outcome, CodeActOutcome::AllowlistViolation { .. }));
        assert_eq!(outcome.as_str(), "allowlist_violation");
    }

    #[tokio::test]
    async fn allowlist_pass_dispatches_to_sandbox_when_present() {
        struct MockSandbox;
        impl CodeActSandbox for MockSandbox {
            fn run<'a>(
                &'a self,
                _action: &'a CodeActAction,
            ) -> Pin<Box<dyn Future<Output = CodeActOutcome> + Send + 'a>> {
                Box::pin(async move {
                    CodeActOutcome::Executed {
                        stdout_excerpt: "ok\n".to_string(),
                        exit_code: 0,
                        duration_ms: 17,
                    }
                })
            }
        }
        let action = mk_action("import json\nprint(json.dumps({'x': 1}))", &["json"]);
        let sb = MockSandbox;
        let outcome =
            execute_code_act(&action, Some(&sb), None, 1_700_000_000_003).await;
        match outcome {
            CodeActOutcome::Executed { exit_code, .. } => assert_eq!(exit_code, 0),
            other => panic!("expected Executed, got {other:?}"),
        }
    }

    #[test]
    fn action_hash_deterministic_for_same_input() {
        let a1 = mk_action("print(1)", &["builtins"]);
        let a2 = mk_action("print(1)", &["builtins"]);
        assert_eq!(action_hash_of(&a1), action_hash_of(&a2));
    }

    #[test]
    fn outcome_hash_distinct_per_label() {
        let h_exec = outcome_hash_of(&CodeActOutcome::Executed {
            stdout_excerpt: String::new(),
            exit_code: 0,
            duration_ms: 0,
        });
        let h_absent = outcome_hash_of(&CodeActOutcome::SandboxAbsent {
            detail: String::new(),
        });
        let h_cap = outcome_hash_of(&CodeActOutcome::CapExceeded {
            observed: 0,
            cap: 0,
        });
        let h_allow =
            outcome_hash_of(&CodeActOutcome::AllowlistViolation { tool_id_hash: 0 });
        let all = vec![h_exec, h_absent, h_cap, h_allow];
        let unique: std::collections::HashSet<_> = all.iter().collect();
        assert_eq!(unique.len(), 4, "all 4 outcome labels must hash distinctly");
    }
}
