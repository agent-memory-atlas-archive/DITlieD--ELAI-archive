//! IPC-frame hook dispatcher: invoked from IpcGateway during agent process IPC
//! traffic. Distinct from InProcessHookDispatcher (orchestrator/hook_dispatcher.rs)
//! which fires at orchestrator baby-step boundaries (T1/T2/T3 tiered analysis).
//!
//! This module provides the `IpcHookTrait` and `IpcHookDispatcher` that sit
//! between the IPC gateway's `route_frame()` return and the tool executor.
//! All ToolRequest/BashRequest frames pass through `pre_tool_use()` before
//! execution. BLOCK verdicts produce sanitized IPC errors (no rule_id,
//! pattern text, or tier detail leaked to agents).
//!
//! SSB finding writes are abstracted via `SsbFindingWriter` trait to avoid
//! a circular dependency (plugins must not import orchestrator).
//!
//! Consumed by `orchestrator::ipc_gateway::IpcGateway::run_pre_tool` /
//! `run_post_tool` (see ipc_gateway.rs:504-563) via `IpcHookTrait`/
//! `IpcHookError`/`HookResult`. Present callers
//! (contracts/tests/plugins_hooks.rs and plugins_adversarial.rs) validate
//! the spec contract. Do not remove.

use rustc_hash::FxHashMap;

use elai_core::{ClockProvider as _, OsClock};

use crate::tiered_analysis::{
    HookVerdict, TieredAnalysis, TieredAnalysisError, TieredAnalysisResult,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum source lines before PostToolUse circuit breaker activates.
const POST_TOOL_USE_LINE_LIMIT: usize = 10_000;

/// T1 latency target in microseconds (for metrics reporting).
const T1_LATENCY_TARGET_US: u64 = 5_000;

/// Generic error message returned to agents on BLOCK (no internal detail).
const BLOCK_AGENT_ERROR: &str = "Tool call rejected by policy";

/// Generic warning message returned to agents on WARN.
const WARN_AGENT_MESSAGE: &str = "Tool call warning issued";

// ---------------------------------------------------------------------------
// IpcHookEvent — which hook is firing
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcHookEvent {
    PreToolUse,
    PostToolUse,
    Stop,
    PreCompact,
}

impl std::fmt::Display for IpcHookEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PreToolUse => write!(f, "pre_tool_use"),
            Self::PostToolUse => write!(f, "post_tool_use"),
            Self::Stop => write!(f, "stop"),
            Self::PreCompact => write!(f, "pre_compact"),
        }
    }
}

// ---------------------------------------------------------------------------
// HookContext — per-request context passed to hook methods
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct HookContext {
    pub agent_id: String,
    pub agent_role: String,
    pub session_id: String,
    pub fsm_state: String,
    pub tool_id: Option<String>,
    pub tool_arguments: Option<Vec<u8>>,
    pub source_content: Option<String>,
    pub file_path: Option<String>,
    pub active_rules_count: usize,
}

// ---------------------------------------------------------------------------
// HookResult — outcome of a hook invocation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct HookResult {
    pub verdict: HookVerdict,
    pub event: IpcHookEvent,
    pub latency_us: u64,
    pub finding_written: bool,
}

impl HookResult {
    /// Produce a sanitized error string safe to send to agents.
    /// BLOCK -> generic error, WARN -> generic warning, Allow -> empty.
    pub fn sanitized_agent_message(&self) -> String {
        match &self.verdict {
            HookVerdict::Block { .. } => BLOCK_AGENT_ERROR.to_string(),
            HookVerdict::Warn { .. } => WARN_AGENT_MESSAGE.to_string(),
            HookVerdict::Allow => String::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// IpcHookError
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum IpcHookError {
    #[error("tiered analysis error: {0}")]
    TieredAnalysis(#[from] TieredAnalysisError),

    #[error("ssb finding write failed: {0}")]
    SsbWrite(String),

    #[error("hook context missing required field: {0}")]
    MissingField(String),
}

// ---------------------------------------------------------------------------
// SsbFindingWriter — trait abstraction for SSB writes (no orchestrator dep)
// ---------------------------------------------------------------------------

/// Trait for writing findings to SSB. The orchestrator implements this;
/// the plugins crate defines it to avoid a circular dependency.
pub trait SsbFindingWriter {
    /// Write a finding entry to SSB.
    /// Implementors must inject `project_id` per [SSB_READER_PROJECT_ID_SCOPING].
    fn write_finding(
        &self,
        agent_id: &str,
        pattern_hash: &str,
        verdict_json: &str,
        files: &[String],
    ) -> Result<(), String>;
}

/// No-op writer for testing and contexts where SSB is unavailable.
pub struct NoopSsbWriter;

impl SsbFindingWriter for NoopSsbWriter {
    fn write_finding(
        &self,
        _agent_id: &str,
        _pattern_hash: &str,
        _verdict_json: &str,
        _files: &[String],
    ) -> Result<(), String> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// CrossSessionLockChecker — trait for cross-session file lock queries
// ---------------------------------------------------------------------------

/// Checks whether a file is locked by another session (multi-session protocol).
pub trait CrossSessionLockChecker {
    /// Returns Some(session_id) if the file is locked by another session.
    fn is_locked_by_other(&self, file_path: &str, current_session: &str) -> Option<String>;
}

/// No-op checker for single-session contexts.
pub struct NoopLockChecker;

impl CrossSessionLockChecker for NoopLockChecker {
    fn is_locked_by_other(&self, _file_path: &str, _current_session: &str) -> Option<String> {
        None
    }
}

// ---------------------------------------------------------------------------
// IpcHookTrait — the hook interface
// ---------------------------------------------------------------------------

/// Hook trait for IPC tool interception. All methods are synchronous
/// (tiered analysis is CPU-bound, not IO-bound).
pub trait IpcHookTrait {
    /// Fires BEFORE a ToolRequest/BashRequest is dispatched to the executor.
    /// Returns Block to reject, Warn to allow with advisory, Allow to pass.
    fn pre_tool_use(&self, ctx: &HookContext) -> Result<HookResult, IpcHookError>;

    /// Fires AFTER a tool result is returned. Runs ruff checks and
    /// circuit breaker for oversized output.
    fn post_tool_use(
        &self,
        ctx: &HookContext,
        tool_output: &str,
    ) -> Result<HookResult, IpcHookError>;

    /// Fires on session/agent stop. Blocks if FSM is not in IDLE state.
    fn stop(&self, ctx: &HookContext) -> Result<HookResult, IpcHookError>;

    /// Fires before context compaction. Writes checkpoint to SSB.
    fn pre_compact(&self, ctx: &HookContext) -> Result<HookResult, IpcHookError>;
}

// ---------------------------------------------------------------------------
// IpcHookDispatcher — the concrete implementation
// ---------------------------------------------------------------------------

pub struct IpcHookDispatcher {
    tiered_analysis: TieredAnalysis,
    ssb_writer: Box<dyn SsbFindingWriter>,
    lock_checker: Box<dyn CrossSessionLockChecker>,
}

impl IpcHookDispatcher {
    pub fn new(
        tiered_analysis: TieredAnalysis,
        ssb_writer: Box<dyn SsbFindingWriter>,
        lock_checker: Box<dyn CrossSessionLockChecker>,
    ) -> Self {
        Self {
            tiered_analysis,
            ssb_writer,
            lock_checker,
        }
    }

    /// Normalize a finding to pattern_hash via blake3.
    /// Called by the orchestrator side (not in this crate) for SSB writes.
    /// Exposed here for testing; the orchestrator re-hashes on its side.
    pub fn compute_pattern_hash(rule_id: &str, evidence: &str) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(rule_id.as_bytes());
        hasher.update(b":");
        hasher.update(evidence.as_bytes());
        hasher.finalize().to_hex().to_string()
    }

    fn write_finding_if_block(
        &self,
        ctx: &HookContext,
        verdict: &HookVerdict,
        event: IpcHookEvent,
    ) -> bool {
        if let HookVerdict::Block { pattern_hash, message } = verdict {
            let verdict_json = serde_json::json!({
                "verdict": "block",
                "pattern_hash": pattern_hash,
                "message": message,
                "event": event.to_string(),
            })
            .to_string();

            let files: Vec<String> = ctx
                .file_path
                .as_ref()
                .map(|p| vec![p.clone()])
                .unwrap_or_default();

            if let Err(e) = self.ssb_writer.write_finding(
                &ctx.agent_id,
                pattern_hash,
                &verdict_json,
                &files,
            ) {
                tracing::warn!(
                    agent_id = %ctx.agent_id,
                    error = %e,
                    "non-blocking SSB finding write failed (dropped)"
                );
                return false;
            }
            return true;
        }
        false
    }
}

impl IpcHookTrait for IpcHookDispatcher {
    fn pre_tool_use(&self, ctx: &HookContext) -> Result<HookResult, IpcHookError> {
        let _clock = OsClock;
        let start = _clock.now();

        // Check cross-session file lock first
        if let Some(ref file_path) = ctx.file_path {
            if let Some(locking_session) =
                self.lock_checker.is_locked_by_other(file_path, &ctx.session_id)
            {
                let verdict = HookVerdict::Block {
                    message: format!("file locked by session {}", locking_session),
                    pattern_hash: Self::compute_pattern_hash("cross_session_lock", file_path),
                };
                let latency_us = _clock.elapsed_since(start).as_micros().try_into().unwrap_or(u64::MAX);
                let finding_written = self.write_finding_if_block(ctx, &verdict, IpcHookEvent::PreToolUse);
                return Ok(HookResult {
                    verdict,
                    event: IpcHookEvent::PreToolUse,
                    latency_us,
                    finding_written,
                });
            }
        }

        // Run tiered analysis
        let source = ctx.source_content.as_deref().unwrap_or("");
        let file_path = ctx.file_path.as_deref().unwrap_or("");
        let action = ctx.tool_id.as_deref().unwrap_or("unknown");

        let analysis_result: TieredAnalysisResult = self.tiered_analysis.analyze(
            action,
            if source.is_empty() { None } else { Some(source) },
            if file_path.is_empty() { None } else { Some(file_path) },
            FxHashMap::default(),
        )?;

        let latency_us = _clock.elapsed_since(start).as_micros().try_into().unwrap_or(u64::MAX);
        let finding_written = self.write_finding_if_block(ctx, &analysis_result.final_verdict, IpcHookEvent::PreToolUse);

        Ok(HookResult {
            verdict: analysis_result.final_verdict,
            event: IpcHookEvent::PreToolUse,
            latency_us,
            finding_written,
        })
    }

    fn post_tool_use(
        &self,
        ctx: &HookContext,
        tool_output: &str,
    ) -> Result<HookResult, IpcHookError> {
        let _clock = OsClock;
        let start = _clock.now();

        // Circuit breaker: skip analysis if output exceeds line limit
        let line_count = tool_output.lines().count();
        if line_count > POST_TOOL_USE_LINE_LIMIT {
            tracing::warn!(
                agent_id = %ctx.agent_id,
                line_count = line_count,
                limit = POST_TOOL_USE_LINE_LIMIT,
                "PostToolUse circuit breaker: output too large, skipping analysis"
            );
            let latency_us = _clock.elapsed_since(start).as_micros().try_into().unwrap_or(u64::MAX);
            return Ok(HookResult {
                verdict: HookVerdict::Warn {
                    message: format!(
                        "output exceeds {} lines, analysis skipped",
                        POST_TOOL_USE_LINE_LIMIT
                    ),
                    pattern_hash: Self::compute_pattern_hash(
                        "circuit_breaker",
                        "post_tool_use_line_limit",
                    ),
                },
                event: IpcHookEvent::PostToolUse,
                latency_us,
                finding_written: false,
            });
        }

        // Run tiered analysis on the tool output
        let file_path = ctx.file_path.as_deref().unwrap_or("");
        let analysis_result = self.tiered_analysis.analyze(
            "tool_result",
            if tool_output.is_empty() {
                None
            } else {
                Some(tool_output)
            },
            if file_path.is_empty() {
                None
            } else {
                Some(file_path)
            },
            FxHashMap::default(),
        )?;

        let latency_us = _clock.elapsed_since(start).as_micros().try_into().unwrap_or(u64::MAX);

        // Write finding for BLOCK or WARN in PostToolUse
        let finding_written = if analysis_result.final_verdict.is_block() {
            self.write_finding_if_block(ctx, &analysis_result.final_verdict, IpcHookEvent::PostToolUse)
        } else {
            false
        };

        Ok(HookResult {
            verdict: analysis_result.final_verdict,
            event: IpcHookEvent::PostToolUse,
            latency_us,
            finding_written,
        })
    }

    fn stop(&self, ctx: &HookContext) -> Result<HookResult, IpcHookError> {
        let _clock = OsClock;
        let start = _clock.now();

        // Block if FSM is not IDLE
        let verdict = if ctx.fsm_state != "idle" {
            HookVerdict::Block {
                message: format!("stop rejected: FSM state is '{}', not 'idle'", ctx.fsm_state),
                pattern_hash: Self::compute_pattern_hash("stop_hook", &ctx.fsm_state),
            }
        } else {
            HookVerdict::Allow
        };

        let latency_us = _clock.elapsed_since(start).as_micros().try_into().unwrap_or(u64::MAX);
        let finding_written = self.write_finding_if_block(ctx, &verdict, IpcHookEvent::Stop);

        Ok(HookResult {
            verdict,
            event: IpcHookEvent::Stop,
            latency_us,
            finding_written,
        })
    }

    fn pre_compact(&self, ctx: &HookContext) -> Result<HookResult, IpcHookError> {
        let _clock = OsClock;
        let start = _clock.now();

        // Write checkpoint to SSB
        let checkpoint_json = serde_json::json!({
            "event": "pre_compact",
            "agent_id": ctx.agent_id,
            "session_id": ctx.session_id,
            "fsm_state": ctx.fsm_state,
        })
        .to_string();

        let finding_written = self
            .ssb_writer
            .write_finding(
                &ctx.agent_id,
                &Self::compute_pattern_hash("pre_compact", &ctx.session_id),
                &checkpoint_json,
                &[],
            )
            .map(|_| true)
            .unwrap_or_else(|e| {
                tracing::warn!(
                    agent_id = %ctx.agent_id,
                    error = %e,
                    "pre_compact SSB checkpoint write failed (non-blocking)"
                );
                false
            });

        let latency_us = _clock.elapsed_since(start).as_micros().try_into().unwrap_or(u64::MAX);

        Ok(HookResult {
            verdict: HookVerdict::Allow,
            event: IpcHookEvent::PreCompact,
            latency_us,
            finding_written,
        })
    }
}

// ---------------------------------------------------------------------------
// Exported constants (for tests and metrics)
// ---------------------------------------------------------------------------

pub const fn post_tool_use_line_limit() -> usize {
    POST_TOOL_USE_LINE_LIMIT
}

pub const fn t1_latency_target_us() -> u64 {
    T1_LATENCY_TARGET_US
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tiered_analysis::TieredAnalysis;
    use elai_rules::predicate::{
        AstError, AstEvaluator, AstMatch, DkgError, DkgQuerier,
        ReachabilityMatch, ReachabilityParams,
    };
    use elai_rules::rule_engine::{DefaultRuleEngine, FsmPhaseBit};

    struct StubAst;
    impl AstEvaluator for StubAst {
        fn evaluate_query(&self, _: &str, _: &str, _: &str) -> Result<Vec<AstMatch>, AstError> { Ok(vec![]) }
        fn validate_query(&self, _: &str, _: &str) -> Result<(), AstError> { Ok(()) }
    }
    struct StubDkg;
    impl DkgQuerier for StubDkg {
        fn query_reachability(&self, _: &ReachabilityParams) -> Result<Vec<ReachabilityMatch>, DkgError> { Ok(vec![]) }
        fn validate_params(&self, _: &ReachabilityParams) -> Result<(), DkgError> { Ok(()) }
    }

    fn make_dispatcher() -> IpcHookDispatcher {
        let engine = Box::new(DefaultRuleEngine::new(Box::new(StubAst), Box::new(StubDkg)));
        let ta = TieredAnalysis::new(engine, FsmPhaseBit::EXECUTING);
        IpcHookDispatcher::new(
            ta,
            Box::new(NoopSsbWriter),
            Box::new(NoopLockChecker),
        )
    }

    fn make_ctx() -> HookContext {
        HookContext {
            agent_id: "test-agent".to_string(),
            agent_role: "worker".to_string(),
            session_id: "s-test".to_string(),
            fsm_state: "executing".to_string(),
            tool_id: Some("file_write".to_string()),
            tool_arguments: None,
            source_content: Some("fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n".to_string()),
            file_path: Some("test.rs".to_string()),
            active_rules_count: 0,
        }
    }

    #[test]
    fn pre_tool_use_allows_clean_code() {
        let d = make_dispatcher();
        let ctx = make_ctx();
        let result = d.pre_tool_use(&ctx).expect("should succeed");
        assert!(
            matches!(result.verdict, HookVerdict::Allow),
            "clean code should be allowed"
        );
        assert_eq!(result.event, IpcHookEvent::PreToolUse);
    }

    #[test]
    fn stop_blocks_non_idle() {
        let d = make_dispatcher();
        let ctx = make_ctx(); // fsm_state = "executing"
        let result = d.stop(&ctx).expect("should succeed");
        assert!(result.verdict.is_block(), "should block non-idle stop");
    }

    #[test]
    fn stop_allows_idle() {
        let d = make_dispatcher();
        let mut ctx = make_ctx();
        ctx.fsm_state = "idle".to_string();
        let result = d.stop(&ctx).expect("should succeed");
        assert!(
            matches!(result.verdict, HookVerdict::Allow),
            "idle stop should be allowed"
        );
    }

    #[test]
    fn sanitized_message_hides_internals() {
        let result = HookResult {
            verdict: HookVerdict::Block {
                message: "secret rule detail".to_string(),
                pattern_hash: "abc123".to_string(),
            },
            event: IpcHookEvent::PreToolUse,
            latency_us: 100,
            finding_written: false,
        };
        let msg = result.sanitized_agent_message();
        assert_eq!(msg, BLOCK_AGENT_ERROR);
        assert!(!msg.contains("secret"));
        assert!(!msg.contains("abc123"));
    }

    #[test]
    fn post_tool_use_circuit_breaker() {
        let d = make_dispatcher();
        let ctx = make_ctx();
        let big_output = "line\n".repeat(POST_TOOL_USE_LINE_LIMIT + 1);
        let result = d.post_tool_use(&ctx, &big_output).expect("should succeed");
        assert!(result.verdict.is_warn(), "circuit breaker should warn");
    }

    #[test]
    fn pattern_hash_is_deterministic() {
        let h1 = IpcHookDispatcher::compute_pattern_hash("rule1", "evidence1");
        let h2 = IpcHookDispatcher::compute_pattern_hash("rule1", "evidence1");
        assert_eq!(h1, h2);
        let h3 = IpcHookDispatcher::compute_pattern_hash("rule2", "evidence1");
        assert_ne!(h1, h3);
    }

    #[test]
    fn pre_compact_writes_checkpoint() {
        let d = make_dispatcher();
        let ctx = make_ctx();
        let result = d.pre_compact(&ctx).expect("should succeed");
        assert!(matches!(result.verdict, HookVerdict::Allow));
        assert_eq!(result.event, IpcHookEvent::PreCompact);
    }
}
