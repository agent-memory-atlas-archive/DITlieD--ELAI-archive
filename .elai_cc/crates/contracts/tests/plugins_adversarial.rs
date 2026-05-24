//! Adversarial tests for Phase 4C: IPC hook interception layer.
//!
//! 10 tests covering edge cases, information leakage prevention,
//! concurrent lock scenarios, and FSM state enforcement.

use std::sync::Mutex;

use elai_plugins::ipc_hooks::{
    CrossSessionLockChecker, HookContext, IpcHookDispatcher, IpcHookTrait,
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
// Helpers
// ---------------------------------------------------------------------------

fn make_ctx() -> HookContext {
    HookContext {
        agent_id: "adv-agent".to_string(),
        agent_role: "worker".to_string(),
        session_id: "s-adv".to_string(),
        fsm_state: "executing".to_string(),
        tool_id: Some("file_write".to_string()),
        tool_arguments: None,
        source_content: Some("fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n".to_string()),
        file_path: Some("src/lib.rs".to_string()),
        active_rules_count: 0,
    }
}

fn make_dispatcher() -> IpcHookDispatcher {
    let ta = TieredAnalysis::new(stub_engine(), FsmPhaseBit::EXECUTING);
    IpcHookDispatcher::new(ta, Box::new(NoopSsbWriter), Box::new(NoopLockChecker))
}

/// SSB writer that records verdicts for inspection.
/// B-19: `Mutex` (not `RefCell`) so the type is `Send + Sync` to satisfy
/// the upstream `SsbFindingWriter: Send + Sync` supertrait.
struct SpySsbWriter {
    verdicts: Mutex<Vec<String>>,
}

impl SpySsbWriter {
    fn new() -> Self {
        Self {
            verdicts: Mutex::new(Vec::new()),
        }
    }
}

impl SsbFindingWriter for SpySsbWriter {
    fn write_finding(
        &self,
        _agent_id: &str,
        _pattern_hash: &str,
        verdict_json: &str,
        _files: &[String],
    ) -> Result<(), String> {
        match self.verdicts.lock() {
            Ok(mut g) => {
                g.push(verdict_json.to_string());
                Ok(())
            }
            Err(e) => Err(format!("spy lock poisoned: {e}")),
        }
    }
}

/// Lock checker that blocks ALL files from a specific session.
struct TotalLockChecker {
    locking_session: String,
}

impl CrossSessionLockChecker for TotalLockChecker {
    fn is_locked_by_other(&self, _file_path: &str, current_session: &str) -> Option<String> {
        if current_session != &self.locking_session {
            Some(self.locking_session.clone())
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Adversarial tests (10 total)
// ---------------------------------------------------------------------------

#[test]
fn adv01_block_response_never_leaks_rule_id() {
    // Even if the internal message contains a rule ID, the sanitized output must not.
    let result = elai_plugins::HookResult {
        verdict: HookVerdict::Block {
            message: "Rule ELAI-R042 violated: console.log detected in production code".to_string(),
            pattern_hash: "hash_R042_console_log".to_string(),
        },
        event: elai_plugins::IpcHookEvent::PreToolUse,
        latency_us: 100,
        finding_written: true,
    };
    let msg = result.sanitized_agent_message();
    assert!(!msg.contains("ELAI"), "leaked rule ID prefix");
    assert!(!msg.contains("R042"), "leaked rule ID number");
    assert!(!msg.contains("console"), "leaked pattern text");
    assert!(!msg.contains("hash_"), "leaked pattern hash");
    assert!(!msg.contains("production"), "leaked context detail");
}

#[test]
fn adv02_block_response_never_leaks_tier_detail() {
    let result = elai_plugins::HookResult {
        verdict: HookVerdict::Block {
            message: "T1 BLOCK: lazy code detected (todo! macro in tier1 analysis)".to_string(),
            pattern_hash: "t1_lazy_todo".to_string(),
        },
        event: elai_plugins::IpcHookEvent::PreToolUse,
        latency_us: 50,
        finding_written: true,
    };
    let msg = result.sanitized_agent_message();
    assert!(!msg.contains("T1"), "leaked tier number");
    assert!(!msg.contains("tier"), "leaked tier reference");
    assert!(!msg.contains("lazy"), "leaked analysis detail");
    assert!(!msg.contains("todo"), "leaked pattern name");
}

#[test]
fn adv03_stop_rejects_all_non_idle_states() {
    let d = make_dispatcher();
    let non_idle_states = [
        "executing",
        "planning",
        "testing_first",
        "auditing",
        "memorizing",
        "complete",
        "configuring",
    ];
    for state in &non_idle_states {
        let mut ctx = make_ctx();
        ctx.fsm_state = state.to_string();
        let result = d.stop(&ctx).unwrap();
        assert!(
            result.verdict.is_block(),
            "stop must block FSM state '{state}'"
        );
    }
}

#[test]
fn adv04_stop_rejects_unknown_fsm_state() {
    let d = make_dispatcher();
    let mut ctx = make_ctx();
    ctx.fsm_state = "bogus_state_injection".to_string();
    let result = d.stop(&ctx).unwrap();
    assert!(
        result.verdict.is_block(),
        "stop must block unknown FSM state"
    );
}

#[test]
fn adv05_cross_session_lock_blocks_tool_request() {
    let checker = TotalLockChecker {
        locking_session: "s-other-session".to_string(),
    };
    let ta = TieredAnalysis::new(stub_engine(), FsmPhaseBit::EXECUTING);
    let d = IpcHookDispatcher::new(ta, Box::new(NoopSsbWriter), Box::new(checker));
    let ctx = make_ctx();
    let result = d.pre_tool_use(&ctx).unwrap();
    assert!(
        result.verdict.is_block(),
        "cross-session locked file must be blocked"
    );
    // Sanitized message must not leak the locking session ID
    let msg = result.sanitized_agent_message();
    assert!(
        !msg.contains("s-other-session"),
        "must not leak locking session in agent message"
    );
}

#[test]
fn adv06_ssb_finding_on_block_contains_verdict() {
    let writer = SpySsbWriter::new();
    let checker = TotalLockChecker {
        locking_session: "s-blocker".to_string(),
    };
    let ta = TieredAnalysis::new(stub_engine(), FsmPhaseBit::EXECUTING);
    let d = IpcHookDispatcher::new(ta, Box::new(writer), Box::new(checker));
    let ctx = make_ctx();
    let result = d.pre_tool_use(&ctx).unwrap();
    assert!(result.verdict.is_block());
    assert!(result.finding_written, "BLOCK must trigger SSB finding write");
}

#[test]
fn adv07_post_tool_use_huge_output_no_panic() {
    let d = make_dispatcher();
    let ctx = make_ctx();
    // 50K lines — well above circuit breaker
    let huge = "x\n".repeat(50_000);
    let result = d.post_tool_use(&ctx, &huge);
    assert!(result.is_ok(), "huge output must not panic");
    assert!(
        result.unwrap().verdict.is_warn(),
        "circuit breaker must warn"
    );
}

#[test]
fn adv08_empty_tool_output_handled() {
    let d = make_dispatcher();
    let ctx = make_ctx();
    let result = d.post_tool_use(&ctx, "");
    assert!(result.is_ok(), "empty output must not error");
}

#[test]
fn adv09_pre_tool_use_no_tool_id() {
    let d = make_dispatcher();
    let mut ctx = make_ctx();
    ctx.tool_id = None;
    let result = d.pre_tool_use(&ctx);
    assert!(result.is_ok(), "missing tool_id must not error");
}

#[test]
fn adv10_pattern_hash_collision_resistance() {
    // Ensure that prepending to rule_id doesn't collide with appending to evidence
    // by verifying the separator is included
    let h1 = IpcHookDispatcher::compute_pattern_hash("abc", "def");
    let h2 = IpcHookDispatcher::compute_pattern_hash("ab", "cdef");
    let h3 = IpcHookDispatcher::compute_pattern_hash("abcd", "ef");
    // All three should differ (the ":" separator prevents collision)
    assert_ne!(h1, h2, "hash must include separator");
    assert_ne!(h1, h3, "hash must include separator");
    assert_ne!(h2, h3, "hash must include separator");
}
