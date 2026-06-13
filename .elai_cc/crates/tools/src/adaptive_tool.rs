// C-13 — AdaptiveTool<T> wrapper + IpcToolDispatcherAdapter shim.
//
// Two-part module landing C-13's "10 missing DAIRA markers" + the
// "IpcToolRegistry as production dispatcher" framing:
//
// 1. [`AdaptiveTool<T>`] — generic wrapper over an inner
//    `Arc<dyn ToolDispatcher>` that intercepts each typed `dispatch_*`
//    call and emits the 10 DAIRA markers that `task_runner::emit_daira_trace`
//    never produces today (Mutation, Branch, Finding, Read, Write,
//    ToolCallStart, ToolCallEnd, Timeout, Escalate, Converge). The 5
//    markers already emitted by task_runner (Call, Return, Error, HookPre,
//    HookPost) are NOT duplicated here.
//
// 2. [`IpcToolDispatcherAdapter`] — adapter shim that implements
//    `elai_core::tool_dispatcher::ToolDispatcher` by:
//      a. consulting the [`IpcToolRegistry::write_tools_allowed`] FSM gate
//         (replicated from the IPC-side semantics so the in-process
//         dispatch path enforces the same Executing|Auditing write window).
//      b. delegating the actual typed dispatch to an inner
//         `InProcessToolDispatcher` (the dispatcher whose 6 typed methods
//         already exist as the production surface).
//      c. emitting DAIRA markers around each call via the bundled
//         AdaptiveTool wrapper.
//
//    This is C-13 orchestrator decision D1 Option B verbatim — the
//    IpcToolRegistry stays the agent-side IPC handler registry while
//    the typed `ToolDispatcher` trait surface (the chokepoint at
//    `run_subcommand.rs:195/275` and `dashboard-v2/lib.rs:472/552`)
//    routes through `IpcToolDispatcherAdapter` so every production tool
//    call passes the FSM gate AND emits the 10 missing markers.
//
// Per CLAUDE.md hard rules: no `.unwrap()` / `.expect()` / `panic!` on
// the production path. SSB write failures are best-effort (logged via
// `tracing::warn!`). No new external deps — every type used here is
// already in the workspace.

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;

use elai_core::tool_dispatcher::{
    ClassifiedFailure, CommitOutcome, EditOutcome, EditRequest, FailureInput, FileView,
    GitCommitRequest, IncrementalOutcome, IncrementalRequest, ToolDispatchError, ToolDispatcher,
    ValidateRequest, ValidationTier, ValidationTierResult,
};
use elai_ipc_types::FsmState;
use elai_orchestrator::ssb::{DairaTracePayload, SsbPayload};
use elai_orchestrator::Ssb;

use crate::ipc_registry::IpcToolRegistry;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Agent id used when the AdaptiveTool wrapper writes DAIRA rows.
const ADAPTIVE_TOOL_AGENT_ID: &str = "adaptive-tool";

/// Soft per-tool latency budget. Calls exceeding this emit a `Timeout`
/// DAIRA marker. Matches the existing `IPC_OVERHEAD_TARGET_MS` (2 ms) ×
/// a 50x margin so only genuinely slow calls trip the marker. Conservative
/// default; deliberately not tunable via env (no new deps / no surface
/// for new env vars per CLAUDE.md).
const ADAPTIVE_TIMEOUT_MS: u128 = 100;

// ---------------------------------------------------------------------------
// DAIRA marker phase strings — must match crate::orchestrator::ssb::DAIRA_MARKERS
// ---------------------------------------------------------------------------

const DAIRA_MUTATION: &str = "Mutation";
const DAIRA_BRANCH: &str = "Branch";
const DAIRA_FINDING: &str = "Finding";
const DAIRA_READ: &str = "Read";
const DAIRA_WRITE: &str = "Write";
const DAIRA_TOOL_CALL_START: &str = "ToolCallStart";
const DAIRA_TOOL_CALL_END: &str = "ToolCallEnd";
const DAIRA_TIMEOUT: &str = "Timeout";
const DAIRA_ESCALATE: &str = "Escalate";
const DAIRA_CONVERGE: &str = "Converge";

// ---------------------------------------------------------------------------
// AdaptiveTool<T>
// ---------------------------------------------------------------------------

/// Wraps an inner `T: ToolDispatcher` and emits the 10 previously-missing
/// DAIRA markers around each typed dispatch call. The 5 markers already
/// emitted by `task_runner::emit_daira_trace` (Call, Return, Error,
/// HookPre, HookPost) are NOT duplicated — task_runner remains the owner
/// of those phases per the C-13 plan WIRING_CONTRACT stub_ban.
///
/// Construction:
///
/// ```ignore
/// let inner: Arc<dyn ToolDispatcher> = ...;
/// let ssb: Arc<Ssb> = ...;
/// let wrapped = Arc::new(AdaptiveTool::new(inner, ssb));
/// ```
///
/// Marker emission map (per `dispatch_*` method):
///   * dispatch_edit            -> ToolCallStart, Read, Write, Mutation, ToolCallEnd
///   * dispatch_validate        -> ToolCallStart, Branch, Finding, Converge, ToolCallEnd
///   * dispatch_classify_failure-> ToolCallStart, Branch, Finding, ToolCallEnd
///   * dispatch_git_commit      -> ToolCallStart, Write, Converge, ToolCallEnd
///   * dispatch_file_read_outline -> ToolCallStart, Read, ToolCallEnd
///   * dispatch_incremental_compile -> ToolCallStart, Branch, Escalate (on retry),
///                                      Converge (on success), ToolCallEnd
///
/// Timeout marker fires when the dispatch duration exceeds
/// [`ADAPTIVE_TIMEOUT_MS`].
pub struct AdaptiveTool<T: ToolDispatcher + ?Sized> {
    inner: Arc<T>,
    ssb: Arc<Ssb>,
}

impl<T: ToolDispatcher + ?Sized> AdaptiveTool<T> {
    pub fn new(inner: Arc<T>, ssb: Arc<Ssb>) -> Self {
        Self { inner, ssb }
    }

    /// Fire-and-forget DAIRA emit. Failures are logged via `tracing::warn!`
    /// but never bubble — the production code path must not break on an
    /// SSB write blip.
    async fn emit(
        &self,
        tool: &str,
        phase: &str,
        elapsed_us: u64,
        outcome: &str,
        step_id: &str,
    ) {
        let payload = SsbPayload::DairaTrace(DairaTracePayload {
            phase: phase.to_owned(),
            tool: tool.to_owned(),
            args_hash: "adaptive".to_string(),
            elapsed_us,
            outcome: outcome.to_owned(),
            step_id: step_id.to_owned(),
        });
        if let Err(e) = self
            .ssb
            .write_payload(ADAPTIVE_TOOL_AGENT_ID, 0, payload, Vec::new())
            .await
        {
            tracing::warn!(error = %e, tool, phase, "adaptive_tool: SSB write failed");
        }
    }
}

impl<T: ToolDispatcher + ?Sized> ToolDispatcher for AdaptiveTool<T> {
    fn dispatch_edit(
        &self,
        req: EditRequest,
    ) -> Pin<Box<dyn Future<Output = Result<EditOutcome, ToolDispatchError>> + Send + '_>> {
        let step_id = String::new();
        Box::pin(async move {
            let start = Instant::now();
            self.emit("edit", DAIRA_TOOL_CALL_START, 0, "started", &step_id).await;
            self.emit("edit", DAIRA_READ, 0, "started", &step_id).await;

            let result = self.inner.dispatch_edit(req).await;
            let elapsed_us = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);

            match &result {
                Ok(outcome) => {
                    self.emit("edit", DAIRA_WRITE, elapsed_us, "success", &step_id).await;
                    if outcome.bytes_written > 0 {
                        self.emit("edit", DAIRA_MUTATION, elapsed_us, "success", &step_id).await;
                    }
                }
                Err(_) => {
                    // task_runner owns Error marker; AdaptiveTool stays out
                    // of that lane per WIRING_CONTRACT stub_ban.
                }
            }

            if start.elapsed().as_millis() > ADAPTIVE_TIMEOUT_MS {
                self.emit("edit", DAIRA_TIMEOUT, elapsed_us, "timeout", &step_id).await;
            }
            self.emit("edit", DAIRA_TOOL_CALL_END, elapsed_us, "ended", &step_id).await;
            result
        })
    }

    fn dispatch_validate(
        &self,
        req: ValidateRequest,
    ) -> Pin<
        Box<dyn Future<Output = Result<Vec<ValidationTierResult>, ToolDispatchError>> + Send + '_>,
    > {
        Box::pin(async move {
            let step_id = req.step_id.clone();
            let start = Instant::now();
            self.emit("validate", DAIRA_TOOL_CALL_START, 0, "started", &step_id).await;
            // Each tier is a deterministic branch decision; emit Branch once
            // up front so the dashboard's call-graph view sees a "tier-fan"
            // node even if the first tier short-circuits.
            self.emit("validate", DAIRA_BRANCH, 0, "started", &step_id).await;

            let result = self.inner.dispatch_validate(req).await;
            let elapsed_us = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);

            match &result {
                Ok(tiers) => {
                    let all_passed = !tiers.is_empty() && tiers.iter().all(|t| t.passed);
                    // Finding marker emitted per tier that failed; surfaces
                    // every BLOCK verdict in the activity feed.
                    for tier in tiers {
                        if !tier.passed {
                            self.emit(
                                "validate",
                                DAIRA_FINDING,
                                u64::from(tier.elapsed_ms).saturating_mul(1_000),
                                tier_outcome(&tier.tier, false),
                                &step_id,
                            )
                            .await;
                        }
                    }
                    if all_passed {
                        self.emit("validate", DAIRA_CONVERGE, elapsed_us, "all_tiers_passed", &step_id).await;
                    }
                }
                Err(_) => { /* task_runner owns Error marker */ }
            }

            if start.elapsed().as_millis() > ADAPTIVE_TIMEOUT_MS {
                self.emit("validate", DAIRA_TIMEOUT, elapsed_us, "timeout", &step_id).await;
            }
            self.emit("validate", DAIRA_TOOL_CALL_END, elapsed_us, "ended", &step_id).await;
            result
        })
    }

    fn dispatch_classify_failure(
        &self,
        input: FailureInput,
    ) -> Pin<Box<dyn Future<Output = Option<ClassifiedFailure>> + Send + '_>> {
        Box::pin(async move {
            let step_id = input.step_id.clone();
            let start = Instant::now();
            self.emit("classify_failure", DAIRA_TOOL_CALL_START, 0, "started", &step_id).await;
            // classify_failure walks a regex DFA — emit Branch for the
            // taxonomic-decision step.
            self.emit("classify_failure", DAIRA_BRANCH, 0, "started", &step_id).await;

            let result = self.inner.dispatch_classify_failure(input).await;
            let elapsed_us = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);

            if let Some(classified) = &result {
                self.emit(
                    "classify_failure",
                    DAIRA_FINDING,
                    elapsed_us,
                    classified.category.as_str(),
                    &step_id,
                )
                .await;
            }

            if start.elapsed().as_millis() > ADAPTIVE_TIMEOUT_MS {
                self.emit("classify_failure", DAIRA_TIMEOUT, elapsed_us, "timeout", &step_id).await;
            }
            self.emit("classify_failure", DAIRA_TOOL_CALL_END, elapsed_us, "ended", &step_id).await;
            result
        })
    }

    fn dispatch_git_commit(
        &self,
        req: GitCommitRequest,
    ) -> Pin<Box<dyn Future<Output = Result<CommitOutcome, ToolDispatchError>> + Send + '_>> {
        Box::pin(async move {
            let step_id = req.step_id.clone();
            let start = Instant::now();
            self.emit("git_commit", DAIRA_TOOL_CALL_START, 0, "started", &step_id).await;
            self.emit("git_commit", DAIRA_WRITE, 0, "started", &step_id).await;

            let result = self.inner.dispatch_git_commit(req).await;
            let elapsed_us = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);

            if result.is_ok() {
                self.emit("git_commit", DAIRA_CONVERGE, elapsed_us, "committed", &step_id).await;
            }

            if start.elapsed().as_millis() > ADAPTIVE_TIMEOUT_MS {
                self.emit("git_commit", DAIRA_TIMEOUT, elapsed_us, "timeout", &step_id).await;
            }
            self.emit("git_commit", DAIRA_TOOL_CALL_END, elapsed_us, "ended", &step_id).await;
            result
        })
    }

    fn dispatch_file_read_outline(
        &self,
        path: PathBuf,
        line_threshold: usize,
    ) -> Pin<Box<dyn Future<Output = Result<FileView, ToolDispatchError>> + Send + '_>> {
        Box::pin(async move {
            let step_id = String::new();
            let start = Instant::now();
            self.emit("file_read_outline", DAIRA_TOOL_CALL_START, 0, "started", &step_id).await;
            self.emit("file_read_outline", DAIRA_READ, 0, "started", &step_id).await;

            let result = self.inner.dispatch_file_read_outline(path, line_threshold).await;
            let elapsed_us = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);

            if start.elapsed().as_millis() > ADAPTIVE_TIMEOUT_MS {
                self.emit("file_read_outline", DAIRA_TIMEOUT, elapsed_us, "timeout", &step_id).await;
            }
            self.emit("file_read_outline", DAIRA_TOOL_CALL_END, elapsed_us, "ended", &step_id).await;
            result
        })
    }

    fn dispatch_incremental_compile(
        &self,
        req: IncrementalRequest,
    ) -> Pin<Box<dyn Future<Output = Result<IncrementalOutcome, ToolDispatchError>> + Send + '_>> {
        Box::pin(async move {
            let step_id = req.step_id.clone();
            let retry_round = req.retry_round;
            let start = Instant::now();
            self.emit("incremental_compile", DAIRA_TOOL_CALL_START, 0, "started", &step_id).await;
            self.emit("incremental_compile", DAIRA_BRANCH, 0, "started", &step_id).await;
            if retry_round > 0 {
                // retry_round > 0 means the previous attempt failed; treat
                // a retry as the Escalate phase per DAIRA semantics.
                self.emit("incremental_compile", DAIRA_ESCALATE, 0, "retry", &step_id).await;
            }

            let result = self.inner.dispatch_incremental_compile(req).await;
            let elapsed_us = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);

            if let Ok(outcome) = &result {
                if outcome.success {
                    self.emit("incremental_compile", DAIRA_CONVERGE, elapsed_us, "success", &step_id).await;
                }
            }

            if start.elapsed().as_millis() > ADAPTIVE_TIMEOUT_MS {
                self.emit("incremental_compile", DAIRA_TIMEOUT, elapsed_us, "timeout", &step_id).await;
            }
            self.emit("incremental_compile", DAIRA_TOOL_CALL_END, elapsed_us, "ended", &step_id).await;
            result
        })
    }
}

fn tier_outcome(tier: &ValidationTier, passed: bool) -> &'static str {
    let _ = tier;
    if passed {
        "passed"
    } else {
        "failed"
    }
}

// ---------------------------------------------------------------------------
// IpcToolDispatcherAdapter
// ---------------------------------------------------------------------------

/// C-13 D1 Option B — adapter shim that wraps an [`IpcToolRegistry`] plus
/// an inner `Arc<dyn ToolDispatcher>` and implements `ToolDispatcher` so
/// every production construction site can swap `InProcessToolDispatcher`
/// for `IpcToolDispatcherAdapter` without touching the typed dispatch
/// surface (`ProductionAgentRpcServer::handle_tool_request` /
/// `translate_tool_request` keep their `dyn ToolDispatcher` arg).
///
/// The shim's FSM gate replicates [`IpcToolRegistry::write_tools_allowed`]
/// semantics: write-shaped dispatch methods (`dispatch_edit`,
/// `dispatch_git_commit`) check `current_fsm_state()` and short-circuit
/// to `ToolDispatchError::Denied` when the state is not
/// `Executing | Auditing`. Read-shaped methods bypass the gate (Reading
/// is allowed in every state).
///
/// The default FSM state used by [`new`] is `FsmState::Executing` — i.e.
/// the gate is permissive unless a caller explicitly installs a stricter
/// state via [`with_fsm_state`]. This keeps the swap behaviourally
/// transparent at the four production sites that previously held an
/// unguarded `InProcessToolDispatcher`.
///
/// The inner dispatcher is always wrapped in an [`AdaptiveTool`] so the
/// 10 DAIRA markers fire on every routed call.
pub struct IpcToolDispatcherAdapter {
    /// The IPC-side registry. Held for symmetry + the public
    /// `registry()` accessor; the typed dispatch path does NOT route
    /// through `registry.dispatch()` (the IPC registry's `dispatch()`
    /// takes a postcard byte slice and cannot satisfy the typed
    /// `ToolDispatcher` trait — see C-13 context-package §1 D-1).
    registry: Arc<IpcToolRegistry>,
    /// AdaptiveTool-wrapped inner dispatcher. Owns the actual typed work.
    inner: Arc<AdaptiveTool<dyn ToolDispatcher>>,
    /// FSM state the gate enforces. Default `Executing` (permissive).
    fsm_state: FsmState,
}

impl IpcToolDispatcherAdapter {
    /// Construct from an existing inner `Arc<dyn ToolDispatcher>` and an
    /// `Arc<Ssb>` writer. The inner dispatcher is wrapped in an
    /// [`AdaptiveTool`] so the 10 DAIRA markers fire on every routed call.
    /// `registry` defaults to an empty [`IpcToolRegistry`]; callers that
    /// want to register handlers should call [`Self::with_registry`].
    pub fn new(inner: Arc<dyn ToolDispatcher>, ssb: Arc<Ssb>) -> Self {
        let adaptive: Arc<AdaptiveTool<dyn ToolDispatcher>> =
            Arc::new(AdaptiveTool::new(inner, Arc::clone(&ssb)));
        // I-13: wire SSB into the registry so IpcToolRegistry::dispatch() emits
        // DualSerialized rows when ToolResultPayload.output exceeds LLM_OUTPUT_CHAR_LIMIT.
        let registry = Arc::new(IpcToolRegistry::new().with_ssb(ssb));
        Self {
            registry,
            inner: adaptive,
            fsm_state: FsmState::Executing,
        }
    }

    /// Override the default empty registry with a pre-populated one.
    #[must_use]
    pub fn with_registry(mut self, registry: Arc<IpcToolRegistry>) -> Self {
        self.registry = registry;
        self
    }

    /// Override the default `Executing` FSM state. Tests that exercise
    /// the gate use this to install a read-only state and assert the
    /// `Denied` shortcut.
    #[must_use]
    pub fn with_fsm_state(mut self, state: FsmState) -> Self {
        self.fsm_state = state;
        self
    }

    /// Read-only accessor for the bundled registry. Used by contract
    /// tests + the dashboard panel that lists registered handlers.
    pub fn registry(&self) -> &Arc<IpcToolRegistry> {
        &self.registry
    }

    /// Current FSM state the gate enforces.
    pub fn fsm_state(&self) -> FsmState {
        self.fsm_state
    }

    /// Replicates `IpcToolRegistry::write_tools_allowed` so the typed
    /// `dispatch_*` methods can short-circuit before touching the inner.
    fn write_allowed(&self) -> bool {
        IpcToolRegistry::write_tools_allowed(self.fsm_state)
    }
}

impl ToolDispatcher for IpcToolDispatcherAdapter {
    fn dispatch_edit(
        &self,
        req: EditRequest,
    ) -> Pin<Box<dyn Future<Output = Result<EditOutcome, ToolDispatchError>> + Send + '_>> {
        if !self.write_allowed() {
            return Box::pin(async {
                Err(ToolDispatchError::Denied {
                    reason: "IpcToolDispatcherAdapter: dispatch_edit denied — write tools require \
                             FsmState::Executing|Auditing"
                        .to_string(),
                })
            });
        }
        self.inner.dispatch_edit(req)
    }

    fn dispatch_validate(
        &self,
        req: ValidateRequest,
    ) -> Pin<
        Box<dyn Future<Output = Result<Vec<ValidationTierResult>, ToolDispatchError>> + Send + '_>,
    > {
        // Validate is read-shaped; allowed in every state.
        self.inner.dispatch_validate(req)
    }

    fn dispatch_classify_failure(
        &self,
        input: FailureInput,
    ) -> Pin<Box<dyn Future<Output = Option<ClassifiedFailure>> + Send + '_>> {
        self.inner.dispatch_classify_failure(input)
    }

    fn dispatch_git_commit(
        &self,
        req: GitCommitRequest,
    ) -> Pin<Box<dyn Future<Output = Result<CommitOutcome, ToolDispatchError>> + Send + '_>> {
        if !self.write_allowed() {
            return Box::pin(async {
                Err(ToolDispatchError::Denied {
                    reason: "IpcToolDispatcherAdapter: dispatch_git_commit denied — write tools \
                             require FsmState::Executing|Auditing"
                        .to_string(),
                })
            });
        }
        self.inner.dispatch_git_commit(req)
    }

    fn dispatch_file_read_outline(
        &self,
        path: PathBuf,
        line_threshold: usize,
    ) -> Pin<Box<dyn Future<Output = Result<FileView, ToolDispatchError>> + Send + '_>> {
        self.inner.dispatch_file_read_outline(path, line_threshold)
    }

    fn dispatch_incremental_compile(
        &self,
        req: IncrementalRequest,
    ) -> Pin<Box<dyn Future<Output = Result<IncrementalOutcome, ToolDispatchError>> + Send + '_>> {
        self.inner.dispatch_incremental_compile(req)
    }
}
