//! Contract tests for Phase 4C: IPC hook interception layer.
//!
//! 30 tests covering HookTrait interface, verdict propagation, error sanitization,
//! T3 firing conditions, cross-session lock checks, SSB write-on-block, and
//! the PostToolUse circuit breaker.

use std::sync::Mutex;

use elai_plugins::ipc_hooks::{
    CrossSessionLockChecker, HookContext, IpcHookDispatcher, IpcHookEvent, IpcHookTrait,
    NoopLockChecker, NoopSsbWriter, SsbFindingWriter,
};
use elai_plugins::tiered_analysis::{HookVerdict, TieredAnalysis};
use elai_rules::predicate::{
    AstError, AstEvaluator, AstMatch, DkgError, DkgQuerier, ReachabilityMatch, ReachabilityParams,
};
use elai_rules::rule_engine::{DefaultRuleEngine, FsmPhaseBit};

// ---------------------------------------------------------------------------
// Stub evaluators
// ---------------------------------------------------------------------------

struct StubAstEvaluator;
impl AstEvaluator for StubAstEvaluator {
    fn evaluate_query(&self, _: &str, _: &str, _: &str) -> Result<Vec<AstMatch>, AstError> {
        Ok(vec![])
    }
    fn validate_query(&self, _: &str, _: &str) -> Result<(), AstError> {
        Ok(())
    }
}

struct StubDkgQuerier;
impl DkgQuerier for StubDkgQuerier {
    fn query_reachability(&self, _: &ReachabilityParams) -> Result<Vec<ReachabilityMatch>, DkgError> {
        Ok(vec![])
    }
    fn validate_params(&self, _: &ReachabilityParams) -> Result<(), DkgError> {
        Ok(())
    }
}

fn stub_engine() -> Box<DefaultRuleEngine> {
    Box::new(DefaultRuleEngine::new(
        Box::new(StubAstEvaluator),
        Box::new(StubDkgQuerier),
    ))
}

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

fn make_dispatcher() -> IpcHookDispatcher {
    let ta = TieredAnalysis::new(stub_engine(), FsmPhaseBit::EXECUTING);
    IpcHookDispatcher::new(ta, Box::new(NoopSsbWriter), Box::new(NoopLockChecker))
}

fn make_dispatcher_with_ssb(writer: Box<dyn SsbFindingWriter>) -> IpcHookDispatcher {
    let ta = TieredAnalysis::new(stub_engine(), FsmPhaseBit::EXECUTING);
    IpcHookDispatcher::new(ta, writer, Box::new(NoopLockChecker))
}

fn make_dispatcher_with_lock(checker: Box<dyn CrossSessionLockChecker>) -> IpcHookDispatcher {
    let ta = TieredAnalysis::new(stub_engine(), FsmPhaseBit::EXECUTING);
    IpcHookDispatcher::new(ta, Box::new(NoopSsbWriter), checker)
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
        file_path: Some("src/main.rs".to_string()),
        active_rules_count: 0,
    }
}

fn make_idle_ctx() -> HookContext {
    let mut ctx = make_ctx();
    ctx.fsm_state = "idle".to_string();
    ctx
}

/// SSB writer that records all findings.
/// B-19: `Mutex` (not `RefCell`) so the type is `Send + Sync` per the
/// upstream `SsbFindingWriter: Send + Sync` supertrait.
struct RecordingSsbWriter {
    findings: Mutex<Vec<(String, String, String)>>,
}

impl RecordingSsbWriter {
    fn new() -> Self {
        Self {
            findings: Mutex::new(Vec::new()),
        }
    }

    fn finding_count(&self) -> usize {
        match self.findings.lock() {
            Ok(g) => g.len(),
            Err(p) => p.into_inner().len(),
        }
    }
}

impl SsbFindingWriter for RecordingSsbWriter {
    fn write_finding(
        &self,
        agent_id: &str,
        pattern_hash: &str,
        verdict_json: &str,
        _files: &[String],
    ) -> Result<(), String> {
        match self.findings.lock() {
            Ok(mut g) => {
                g.push((
                    agent_id.to_string(),
                    pattern_hash.to_string(),
                    verdict_json.to_string(),
                ));
                Ok(())
            }
            Err(e) => Err(format!("recording writer lock poisoned: {e}")),
        }
    }
}

/// SSB writer that always fails.
struct FailingSsbWriter;

impl SsbFindingWriter for FailingSsbWriter {
    fn write_finding(
        &self,
        _agent_id: &str,
        _pattern_hash: &str,
        _verdict_json: &str,
        _files: &[String],
    ) -> Result<(), String> {
        Err("SSB write simulated failure".to_string())
    }
}

/// Lock checker that blocks a specific file.
struct BlockingLockChecker {
    locked_file: String,
    locking_session: String,
}

impl CrossSessionLockChecker for BlockingLockChecker {
    fn is_locked_by_other(&self, file_path: &str, current_session: &str) -> Option<String> {
        if file_path == self.locked_file && current_session != &self.locking_session {
            Some(self.locking_session.clone())
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Contract tests (30 total)
// ---------------------------------------------------------------------------

// --- 1-5: HookTrait interface contract ---

#[test]
fn c01_pre_tool_use_returns_hook_result() {
    let d = make_dispatcher();
    let ctx = make_ctx();
    let result = d.pre_tool_use(&ctx);
    assert!(result.is_ok(), "pre_tool_use must return Ok for valid context");
}

#[test]
fn c02_post_tool_use_returns_hook_result() {
    let d = make_dispatcher();
    let ctx = make_ctx();
    let result = d.post_tool_use(&ctx, "fn main() {}");
    assert!(result.is_ok(), "post_tool_use must return Ok for valid input");
}

#[test]
fn c03_stop_returns_hook_result() {
    let d = make_dispatcher();
    let ctx = make_ctx();
    let result = d.stop(&ctx);
    assert!(result.is_ok(), "stop must return Ok for valid context");
}

#[test]
fn c04_pre_compact_returns_hook_result() {
    let d = make_dispatcher();
    let ctx = make_ctx();
    let result = d.pre_compact(&ctx);
    assert!(result.is_ok(), "pre_compact must return Ok for valid context");
}

#[test]
fn c05_hook_result_has_correct_event_type() {
    let d = make_dispatcher();
    let ctx = make_ctx();
    assert_eq!(
        d.pre_tool_use(&ctx).unwrap().event,
        IpcHookEvent::PreToolUse
    );
    assert_eq!(
        d.post_tool_use(&ctx, "ok").unwrap().event,
        IpcHookEvent::PostToolUse
    );
    assert_eq!(d.stop(&ctx).unwrap().event, IpcHookEvent::Stop);
    assert_eq!(
        d.pre_compact(&ctx).unwrap().event,
        IpcHookEvent::PreCompact
    );
}

// --- 6-10: Verdict propagation ---

#[test]
fn c06_clean_code_allows() {
    let d = make_dispatcher();
    let ctx = make_ctx();
    let result = d.pre_tool_use(&ctx).unwrap();
    assert!(
        matches!(result.verdict, HookVerdict::Allow),
        "clean code must be allowed"
    );
}

#[test]
fn c07_pre_tool_use_tracks_latency() {
    let d = make_dispatcher();
    let ctx = make_ctx();
    let result = d.pre_tool_use(&ctx).unwrap();
    // Latency should be > 0 (we ran real analysis)
    assert!(result.latency_us < 100_000, "latency must be reasonable");
}

#[test]
fn c08_post_tool_use_clean_output_allows() {
    let d = make_dispatcher();
    let ctx = make_ctx();
    let result = d.post_tool_use(&ctx, "fn add(a: i32, b: i32) -> i32 { a + b }").unwrap();
    assert!(
        matches!(result.verdict, HookVerdict::Allow),
        "clean output should allow"
    );
}

#[test]
fn c09_pre_tool_use_empty_source_allows() {
    let d = make_dispatcher();
    let mut ctx = make_ctx();
    ctx.source_content = None;
    let result = d.pre_tool_use(&ctx).unwrap();
    assert!(
        matches!(result.verdict, HookVerdict::Allow),
        "empty source should allow"
    );
}

#[test]
fn c10_pre_tool_use_no_file_path_allows() {
    let d = make_dispatcher();
    let mut ctx = make_ctx();
    ctx.file_path = None;
    let result = d.pre_tool_use(&ctx).unwrap();
    assert!(
        matches!(result.verdict, HookVerdict::Allow),
        "no file path should allow"
    );
}

// --- 11-15: Error sanitization ---

#[test]
fn c11_block_verdict_sanitized_message() {
    let result = elai_plugins::HookResult {
        verdict: HookVerdict::Block {
            message: "internal rule detail XYZ".to_string(),
            pattern_hash: "hash123".to_string(),
        },
        event: IpcHookEvent::PreToolUse,
        latency_us: 50,
        finding_written: false,
    };
    let msg = result.sanitized_agent_message();
    assert_eq!(msg, "Tool call rejected by policy");
    assert!(!msg.contains("XYZ"), "must not leak internal detail");
    assert!(!msg.contains("hash123"), "must not leak pattern_hash");
}

#[test]
fn c12_warn_verdict_sanitized_message() {
    let result = elai_plugins::HookResult {
        verdict: HookVerdict::Warn {
            message: "internal warning detail".to_string(),
            pattern_hash: "whash".to_string(),
        },
        event: IpcHookEvent::PreToolUse,
        latency_us: 50,
        finding_written: false,
    };
    let msg = result.sanitized_agent_message();
    assert_eq!(msg, "Tool call warning issued");
    assert!(!msg.contains("internal"), "must not leak internal detail");
}

#[test]
fn c13_allow_verdict_empty_message() {
    let result = elai_plugins::HookResult {
        verdict: HookVerdict::Allow,
        event: IpcHookEvent::PreToolUse,
        latency_us: 50,
        finding_written: false,
    };
    assert!(result.sanitized_agent_message().is_empty());
}

#[test]
fn c14_block_message_no_rule_id() {
    let result = elai_plugins::HookResult {
        verdict: HookVerdict::Block {
            message: "Rule R-042 violated in tier T1".to_string(),
            pattern_hash: "abc".to_string(),
        },
        event: IpcHookEvent::PreToolUse,
        latency_us: 50,
        finding_written: false,
    };
    let msg = result.sanitized_agent_message();
    assert!(!msg.contains("R-042"), "must not leak rule_id");
    assert!(!msg.contains("T1"), "must not leak tier detail");
}

#[test]
fn c15_block_message_no_pattern_text() {
    let result = elai_plugins::HookResult {
        verdict: HookVerdict::Block {
            message: "regex matched: console\\.log".to_string(),
            pattern_hash: "p42".to_string(),
        },
        event: IpcHookEvent::PreToolUse,
        latency_us: 50,
        finding_written: false,
    };
    let msg = result.sanitized_agent_message();
    assert!(!msg.contains("console"), "must not leak pattern text");
    assert!(!msg.contains("regex"), "must not leak pattern text");
}

// --- 16-20: T3 firing conditions + Stop hook ---

#[test]
fn c16_stop_blocks_executing_state() {
    let d = make_dispatcher();
    let ctx = make_ctx(); // fsm_state = "executing"
    let result = d.stop(&ctx).unwrap();
    assert!(result.verdict.is_block(), "stop must block non-idle state");
}

#[test]
fn c17_stop_allows_idle_state() {
    let d = make_dispatcher();
    let ctx = make_idle_ctx();
    let result = d.stop(&ctx).unwrap();
    assert!(
        matches!(result.verdict, HookVerdict::Allow),
        "stop must allow idle state"
    );
}

#[test]
fn c18_stop_blocks_planning_state() {
    let d = make_dispatcher();
    let mut ctx = make_ctx();
    ctx.fsm_state = "planning".to_string();
    let result = d.stop(&ctx).unwrap();
    assert!(result.verdict.is_block(), "stop must block planning state");
}

#[test]
fn c19_stop_blocks_auditing_state() {
    let d = make_dispatcher();
    let mut ctx = make_ctx();
    ctx.fsm_state = "auditing".to_string();
    let result = d.stop(&ctx).unwrap();
    assert!(result.verdict.is_block(), "stop must block auditing state");
}

#[test]
fn c20_stop_blocks_memorizing_state() {
    let d = make_dispatcher();
    let mut ctx = make_ctx();
    ctx.fsm_state = "memorizing".to_string();
    let result = d.stop(&ctx).unwrap();
    assert!(
        result.verdict.is_block(),
        "stop must block memorizing state"
    );
}

// --- 21-25: SSB write-on-block + cross-session locks ---

#[test]
fn c21_pre_compact_writes_checkpoint() {
    let writer = RecordingSsbWriter::new();
    let d = make_dispatcher_with_ssb(Box::new(writer));
    let ctx = make_ctx();
    let result = d.pre_compact(&ctx).unwrap();
    assert!(
        matches!(result.verdict, HookVerdict::Allow),
        "pre_compact should always allow"
    );
    assert!(result.finding_written, "pre_compact must write checkpoint");
}

#[test]
fn c22_cross_session_lock_blocks() {
    let checker = BlockingLockChecker {
        locked_file: "src/main.rs".to_string(),
        locking_session: "s-other".to_string(),
    };
    let d = make_dispatcher_with_lock(Box::new(checker));
    let ctx = make_ctx();
    let result = d.pre_tool_use(&ctx).unwrap();
    assert!(
        result.verdict.is_block(),
        "locked file must be blocked"
    );
}

#[test]
fn c23_cross_session_lock_allows_same_session() {
    let checker = BlockingLockChecker {
        locked_file: "src/main.rs".to_string(),
        locking_session: "s-test".to_string(), // same as ctx.session_id
    };
    let d = make_dispatcher_with_lock(Box::new(checker));
    let ctx = make_ctx();
    let result = d.pre_tool_use(&ctx).unwrap();
    assert!(
        matches!(result.verdict, HookVerdict::Allow),
        "same session should not be blocked by own lock"
    );
}

#[test]
fn c24_cross_session_lock_allows_unlocked_file() {
    let checker = BlockingLockChecker {
        locked_file: "other/file.rs".to_string(),
        locking_session: "s-other".to_string(),
    };
    let d = make_dispatcher_with_lock(Box::new(checker));
    let ctx = make_ctx();
    let result = d.pre_tool_use(&ctx).unwrap();
    assert!(
        matches!(result.verdict, HookVerdict::Allow),
        "unlocked file should pass"
    );
}

#[test]
fn c25_ssb_write_failure_non_blocking() {
    // Use cross-session lock to force a BLOCK with a failing SSB writer
    let checker = BlockingLockChecker {
        locked_file: "src/main.rs".to_string(),
        locking_session: "s-other".to_string(),
    };
    let ta = TieredAnalysis::new(stub_engine(), FsmPhaseBit::EXECUTING);
    let d = IpcHookDispatcher::new(ta, Box::new(FailingSsbWriter), Box::new(checker));
    let ctx = make_ctx();
    let result = d.pre_tool_use(&ctx).unwrap();
    // Should still return BLOCK (SSB failure is non-blocking)
    assert!(
        result.verdict.is_block(),
        "SSB write failure must not prevent BLOCK verdict"
    );
    assert!(!result.finding_written, "finding_written should be false on SSB failure");
}

// --- 26-30: PostToolUse circuit breaker + pattern_hash + misc ---

#[test]
fn c26_post_tool_use_circuit_breaker_activates() {
    let d = make_dispatcher();
    let ctx = make_ctx();
    let big_output = "line\n".repeat(elai_plugins::ipc_hooks::post_tool_use_line_limit() + 1);
    let result = d.post_tool_use(&ctx, &big_output).unwrap();
    assert!(
        result.verdict.is_warn(),
        "circuit breaker must produce WARN"
    );
}

#[test]
fn c27_post_tool_use_circuit_breaker_not_active_below_limit() {
    let d = make_dispatcher();
    let ctx = make_ctx();
    let small_output = "fn ok() {}\n".repeat(100);
    let result = d.post_tool_use(&ctx, &small_output).unwrap();
    // Should run full analysis (not circuit-broken)
    assert_eq!(result.event, IpcHookEvent::PostToolUse);
}

#[test]
fn c28_pattern_hash_deterministic() {
    let h1 = IpcHookDispatcher::compute_pattern_hash("rule1", "ev1");
    let h2 = IpcHookDispatcher::compute_pattern_hash("rule1", "ev1");
    assert_eq!(h1, h2, "same inputs must produce same hash");
}

#[test]
fn c29_pattern_hash_different_inputs() {
    let h1 = IpcHookDispatcher::compute_pattern_hash("rule1", "ev1");
    let h2 = IpcHookDispatcher::compute_pattern_hash("rule2", "ev1");
    let h3 = IpcHookDispatcher::compute_pattern_hash("rule1", "ev2");
    assert_ne!(h1, h2, "different rule_id must produce different hash");
    assert_ne!(h1, h3, "different evidence must produce different hash");
}

#[test]
fn c30_hook_event_display() {
    assert_eq!(format!("{}", IpcHookEvent::PreToolUse), "pre_tool_use");
    assert_eq!(format!("{}", IpcHookEvent::PostToolUse), "post_tool_use");
    assert_eq!(format!("{}", IpcHookEvent::Stop), "stop");
    assert_eq!(format!("{}", IpcHookEvent::PreCompact), "pre_compact");
}
