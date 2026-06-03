//! F-3 Slice C2: CommittedSetChecker — object-safe trait seam.
//!
//! Defines the crate-boundary interface that lets the `elai-orchestrator`
//! crate plug its [`CommittedToolSet`]-based enforcement into
//! [`super::gate::ToolGate`] **without** `elai-plugins` gaining a compile-time
//! dependency on `elai-orchestrator`.
//!
//! The trait is object-safe: one method, `&self` receiver, `String` error
//! (not any orchestrator type). The orchestrator crate provides an adapter
//! struct (`CapabilityFlowAdapter`) that wraps a `CommittedToolSet` behind an
//! `Arc<Mutex<Option<...>>>` and implements this trait by calling through to
//! `check_against_commitment`.
//!
//! Usage inside `elai-plugins`:
//!
//! ```ignore
//! let gate = ToolGate::safe_default()
//!     .with_committed_checker(Arc::new(my_adapter));
//!
//! // At call time:
//! gate.check_committed("EXECUTING", "bash")?;
//! ```
//!
//! WIRING DOCTRINE W-3: the trait is the production entrypoint;
//! `NoopCommittedSetChecker` is the only in-plugins impl (tests / no-op default).
//! The live orchestrator adapter is wired at the `DefaultMcpBridge`
//! construction site in `orchestrator::session_driver` (or equivalent caller).

/// Object-safe trait seam between `elai-plugins` (gate enforcement) and
/// `elai-orchestrator` (commitment state machine).
///
/// Implementors live in `elai-orchestrator`; this trait definition lives in
/// `elai-plugins` so `ToolGate` can hold `Arc<dyn CommittedSetChecker + Send + Sync>`
/// without a circular dependency.
///
/// # Contract
///
/// * Returns `Ok(())` when `tool_name` is permitted given the current FSM
///   state and the committed tool set.
/// * Returns `Err(String)` with a human-readable denial reason when the tool
///   is blocked. The string is logged and/or emitted to the SSB
///   `ToolFilterEvent(UntrustedQuarantined)` row.
///
/// # Object safety
///
/// The trait is object-safe: `&self` receiver only, no associated types, no
/// generics.
pub trait CommittedSetChecker: Send + Sync {
    /// Check whether `tool_name` is permitted.
    ///
    /// * `fsm_state` — current FSM state string (e.g. `"EXECUTING"`).
    /// * `tool_name` — raw tool name as provided by the caller.
    ///
    /// Returns `Ok(())` on success, `Err(denial_reason)` on block.
    fn check_committed(&self, fsm_state: &str, tool_name: &str) -> Result<(), String>;
}

/// No-op implementation used when no orchestrator adapter has been wired
/// (default construction path, unit tests, contexts without an SSB writer).
///
/// Always returns `Ok(())`.
pub struct NoopCommittedSetChecker;

impl CommittedSetChecker for NoopCommittedSetChecker {
    fn check_committed(&self, _fsm_state: &str, _tool_name: &str) -> Result<(), String> {
        Ok(())
    }
}
