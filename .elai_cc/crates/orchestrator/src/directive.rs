//! I-1: directive supervisor enforcement plane.
//!
//! The generator (the model-facing dispatch path) emits a pure [`Directive`]
//! value — `tool_name` + `args` + `intent_id` + `initiator` — and NEVER reaches
//! a side-effect sink directly. A [`Supervisor`] is the sole executor of side
//! effects: it gate-checks every directive (F-1 VibeGate, E-30 argv allowlist,
//! G-3 blast-radius) BEFORE execution and runs E-44 saga compensation on a
//! crash. This module is the Tier-1 in-process boundary; OS-process separation
//! (separate pid + IPC channel) is Tier-2 and gated separately (see I-1 plan
//! W-10 tiered scope).
//!
//! COMPOSE-NOT-RESTATE (GD-1): `run_gates` calls the SHIPPED gate primitives
//! (`elai_safety::check_argv`, `crate::vibegate::evaluate_vibegate`,
//! `elai_safety::blast_radius::{check_fs_target, check_egress_host}`). It does
//! NOT re-implement any gate logic. The hook dispatcher's
//! `dispatch_pre_tool_use` REPLACES its inline E-30 + F-1 calls with a single
//! `run_gates` delegation when a supervisor is installed, so exactly one gate
//! path fires per operation (the double-gate hazard the contract test pins).

use std::path::Path;
use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_safety::blast_radius::{check_egress_host, check_fs_target, resolve_host_std};
use elai_safety::{check_argv, AllowlistVerdict};
use std::ffi::OsString;
use std::net::IpAddr;

use crate::hook_dispatcher::{HookFinding, HookTier, HookVerdictKind};
use crate::role_contracts::RoleContractStore;
use crate::ssb::{
    DirectiveExecutedKind, DirectiveExecutedPayload, Ssb, SsbPayload,
    ToolSequenceVerdictKind, ToolSequenceVerdictPayload,
};
use crate::tool_sequence::{
    evaluate_sequence_rules, HistoryBuffer, HistoryEntry, SequenceRuleSet, ToolSequenceVerdict,
    tool_class_from_action_type,
};

/// The canonical action_type string for the bash-tool argv (E-30) seam.
/// Mirrors `crate::hook_dispatcher::BASH_TOOL_USE_ACTION`; the supervisor only
/// runs the argv gate for invocations carrying this action_type, identical to
/// the inline path it replaces.
const BASH_TOOL_USE_ACTION: &str = "bash_tool_use";

/// Hook category emitted on supervisor-authored findings. Matches the inline
/// E-30/F-1 path so downstream `HookEvent` consumers see no category drift.
const HOOK_EVENT_CATEGORY: &str = "hook_event";

/// Sanitized block message prefix (mirrors the inline E-30 path so the
/// dashboard renders identical block copy whether the gate fired inline or via
/// the supervisor).
const SANITIZED_BLOCK_MESSAGE: &str = "blocked by destructive-verb allowlist";

// ---------------------------------------------------------------------------
// Directive — the pure value type
// ---------------------------------------------------------------------------

/// A pure, side-effect-free description of an intended tool call.
///
/// The generator produces `Directive` values ONLY. A `Directive` carries no
/// capability handle, no file descriptor, no egress socket — it is data. The
/// [`Supervisor`] is the only code that turns a `Directive` into a side effect,
/// and only after [`Supervisor::run_gates`] passes. This is the structural
/// "the generator cannot wreck" invariant (lever CANT-WRECK).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Directive {
    /// The tool the generator wants to invoke (e.g. `"bash_tool_use"`,
    /// `"file_write"`). Fed to the gate chain as the action discriminant.
    pub tool_name: String,
    /// The tool arguments as raw JSON. The supervisor does not interpret these
    /// beyond the gate checks; the executing side effect (Tier-2) consumes them.
    pub args: serde_json::Value,
    /// Correlation id linking this directive back to the intent that produced
    /// it. Carried onto the SSB row for trace reconstruction.
    pub intent_id: String,
    /// Who authored the intent (agent id / role). Carried onto the SSB row.
    pub initiator: String,
}

impl Directive {
    /// Construct a directive from its parts.
    #[must_use]
    pub fn new(
        tool_name: impl Into<String>,
        args: serde_json::Value,
        intent_id: impl Into<String>,
        initiator: impl Into<String>,
    ) -> Self {
        Self {
            tool_name: tool_name.into(),
            args: args.into(),
            intent_id: intent_id.into(),
            initiator: initiator.into(),
        }
    }

    /// Extract the shell source string from `args` for the argv gate.
    /// The bash tool encodes the command at `args.command` (string). Returns
    /// `None` when the directive is not a shell call or carries no command.
    fn shell_source(&self) -> Option<&str> {
        if self.tool_name != BASH_TOOL_USE_ACTION {
            return None;
        }
        self.args.get("command").and_then(serde_json::Value::as_str)
    }
}

// ---------------------------------------------------------------------------
// Gate outcome
// ---------------------------------------------------------------------------

/// The result of running the supervisor gate chain over a directive.
///
/// `findings` carries every gate finding (Block / Warn) so the caller can apply
/// the existing fail-closed `findings.iter().any(HookFinding::is_block)` check
/// without any new logic at the call site — identical to the inline path the
/// supervisor replaces. `blocked` is the precomputed "any Block present" answer
/// so callers that only need the boolean do not re-scan.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GateOutcome {
    /// Every finding produced by the gate chain, in chain order
    /// (E-30 argv, then G-3 blast-radius, then F-1 VibeGate).
    pub findings: Vec<HookFinding>,
    /// `true` when at least one finding is a hard Block.
    pub blocked: bool,
}

impl GateOutcome {
    /// Build an outcome from a finding list, computing `blocked` once.
    fn from_findings(findings: Vec<HookFinding>) -> Self {
        let blocked = findings.iter().any(HookFinding::is_block);
        Self { findings, blocked }
    }
}

// ---------------------------------------------------------------------------
// Directive outcome + error
// ---------------------------------------------------------------------------

/// The terminal status of a directive that reached [`Supervisor::execute_directive`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectiveOutcome {
    /// Gates passed and the side effect ran cleanly.
    Executed,
    /// A gate blocked the directive; the side effect never ran.
    Discarded,
    /// The side effect crashed and saga compensation fired to restore state.
    CompensationFired,
}

impl DirectiveOutcome {
    /// Map an outcome to its SSB discriminator arm.
    fn ssb_kind(self) -> DirectiveExecutedKind {
        match self {
            Self::Executed => DirectiveExecutedKind::Executed,
            Self::Discarded => DirectiveExecutedKind::Discarded,
            Self::CompensationFired => DirectiveExecutedKind::CompensationFired,
        }
    }
}

/// Errors raised by the supervisor execution path.
#[derive(Debug, thiserror::Error)]
pub enum SupervisorError {
    /// The underlying side-effect operation returned an error AND saga
    /// compensation was attempted. Carries the original failure text.
    #[error("directive side effect failed: {0}")]
    SideEffectFailed(String),
    /// An SSB write failed while recording the directive outcome row.
    #[error("ssb emit failed: {0}")]
    SsbEmit(#[from] crate::ssb::SsbError),
}

// ---------------------------------------------------------------------------
// Supervisor trait
// ---------------------------------------------------------------------------

/// The sole executor of side effects.
///
/// A `Supervisor` is the only code permitted to turn a [`Directive`] into a
/// side effect. It gate-checks first ([`run_gates`](Supervisor::run_gates)) and
/// executes only on PASS ([`execute_directive`](Supervisor::execute_directive)).
/// The trait is `Send + Sync` so a single `Arc<dyn Supervisor>` can be shared
/// across the task_runner dispatch path and the hook dispatcher.
///
/// Tier-2 (OS-process separation) implements this SAME trait with a separate
/// pid + IPC channel; the in-process [`InProcessSupervisor`] is Tier-1.
pub trait Supervisor: Send + Sync {
    /// Run the F-1 / E-30 / G-3 gate chain over `directive` WITHOUT executing
    /// any side effect. Returns every gate finding so the caller can apply its
    /// own fail-closed check. This is the path the hook dispatcher delegates to
    /// (replacing its inline E-30 + F-1 calls).
    fn run_gates(&self, directive: &Directive) -> GateOutcome;

    /// I-35: push one [`HistoryEntry`] into the supervisor's internal tool-call
    /// history buffer. Called by [`crate::hook_dispatcher::InProcessHookDispatcher`]
    /// after each completed dispatch cycle so the sequence gate has a complete
    /// ordered record of prior tool invocations to evaluate rules against.
    ///
    /// The default implementation is a **no-op** (object-safe). Only
    /// [`InProcessSupervisor`] overrides this to push into its bounded ring buffer.
    /// Tier-2 (OS-process separation) and stub supervisors inherit the no-op.
    fn push_history_entry(&self, _entry: &HistoryEntry) {}

    /// Dyn-safe variant of [`InProcessSupervisor::execute_directive`].
    ///
    /// Gate-checks `directive` via [`run_gates`]; if blocked returns
    /// `Ok(DirectiveOutcome::Discarded)` and neither closure is called. On PASS
    /// calls `op()`, awaits the result, and on `Err` calls `compensate()` before
    /// returning. Emits one `DirectiveExecuted` SSB row for each outcome when an
    /// SSB writer is installed.
    ///
    /// Takes boxed `FnOnce` closures (instead of generics) so the method is
    /// object-safe and callable on `Arc<dyn Supervisor>` without downcasting.
    /// Production callers (task_runner.rs) use this path; tests use the generic
    /// [`InProcessSupervisor::execute_directive`] for ergonomics.
    fn execute_directive_dyn<'a>(
        &'a self,
        directive: Directive,
        op: Box<
            dyn FnOnce() -> std::pin::Pin<
                    Box<dyn std::future::Future<Output = Result<(), String>> + Send>,
                > + Send,
        >,
        compensate: Box<
            dyn FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
                + Send,
        >,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<DirectiveOutcome, SupervisorError>> + Send + 'a>,
    >;
}

// ---------------------------------------------------------------------------
// InProcessSupervisor — Tier-1 implementation
// ---------------------------------------------------------------------------

/// Tier-1 in-process supervisor. Holds the worktree root (for G-3 fs-escape
/// checks) and the pinned egress allow-set (for G-3 egress checks), plus an
/// optional SSB writer used to emit `DirectiveExecuted` rows from
/// [`execute_directive`](InProcessSupervisor::execute_directive).
///
/// This is NOT OS-process separated (that is Tier-2). It IS sufficient for the
/// enforcement-plane invariant the contract test pins: the generator path can
/// only reach a side effect through this type, and `run_gates` is the single
/// gate path.
pub struct InProcessSupervisor {
    /// Worktree root for the G-3 filesystem-escape gate. A directive whose
    /// target resolves outside this root is blocked fail-closed.
    worktree_root: std::path::PathBuf,
    /// Pinned egress IP allow-set for the G-3 egress gate. Empty = deny all
    /// egress (fail-closed). Hosts resolving to any IP outside this set block.
    egress_allow: Vec<IpAddr>,
    /// Optional SSB writer. When `Some`, every [`execute_directive`] call emits
    /// one `DirectiveExecuted` row (Executed / Discarded / CompensationFired).
    /// `None` = no row (test/offline contexts); gates still fire identically.
    ssb: Option<Arc<Ssb>>,
    /// agent_id stamped on emitted SSB rows.
    agent_id: String,
    /// superstep stamped on emitted SSB rows.
    superstep: i64,
    /// Injected clock for SSB row timestamps. `OsClock` in production;
    /// inject a `SeededClock` in tests for determinism (entropy quarantine).
    clock: Arc<dyn ClockProvider>,
    /// I-32: optional role-contract store. When `Some`, `run_role_scope_gate`
    /// checks each directive's `initiator` role against the store's tool
    /// allowlist; out-of-scope tool invocations are blocked fail-closed.
    /// `None` = gate disabled (test/offline contexts).
    role_contract_store: Option<RoleContractStore>,
    /// I-35: optional sequence rule set. When `Some`, `run_sequence_gate`
    /// evaluates the ordered history buffer against the TOML-declared rules;
    /// violations are blocked fail-closed (HaltEnforced). `None` = gate disabled.
    sequence_rules: Option<Arc<SequenceRuleSet>>,
    /// I-35: bounded tool-call history buffer. Populated by `push_history_entry`,
    /// which is called by the hook dispatcher after each completed dispatch cycle.
    /// Shared via `Arc<Mutex<VecDeque<HistoryEntry>>>` inside [`HistoryBuffer`].
    history: HistoryBuffer,
}

impl std::fmt::Debug for InProcessSupervisor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InProcessSupervisor")
            .field("worktree_root", &self.worktree_root)
            .field("egress_allow_len", &self.egress_allow.len())
            .field("ssb_attached", &self.ssb.is_some())
            .field("agent_id", &self.agent_id)
            .field("superstep", &self.superstep)
            .field("role_contracts_attached", &self.role_contract_store.is_some())
            .field("sequence_rules_attached", &self.sequence_rules.is_some())
            .finish()
    }
}

impl InProcessSupervisor {
    /// Construct a Tier-1 supervisor rooted at `worktree_root`. No SSB writer
    /// and an empty (deny-all) egress set by default; chain
    /// [`with_ssb`](Self::with_ssb) / [`with_egress_allow`](Self::with_egress_allow)
    /// to enable emission / pin egress hosts.
    #[must_use]
    pub fn new(worktree_root: impl Into<std::path::PathBuf>) -> Self {
        Self {
            worktree_root: worktree_root.into(),
            egress_allow: Vec::new(),
            ssb: None,
            agent_id: "supervisor".to_owned(),
            superstep: 0,
            clock: Arc::new(OsClock),
            role_contract_store: None,
            sequence_rules: None,
            history: HistoryBuffer::new(),
        }
    }

    /// Inject a [`ClockProvider`] for deterministic SSB timestamps in tests.
    /// Production relies on the [`OsClock`] default installed by [`new`](Self::new).
    #[must_use]
    pub fn with_clock(mut self, clock: Arc<dyn ClockProvider>) -> Self {
        self.clock = clock;
        self
    }

    /// Install the SSB writer used to emit `DirectiveExecuted` rows, with the
    /// `agent_id` + `superstep` stamped on every row.
    #[must_use]
    pub fn with_ssb(mut self, ssb: Arc<Ssb>, agent_id: impl Into<String>, superstep: i64) -> Self {
        self.ssb = Some(ssb);
        self.agent_id = agent_id.into();
        self.superstep = superstep;
        self
    }

    /// Pin the egress IP allow-set for the G-3 egress gate. An empty set
    /// (the default) denies all egress fail-closed.
    #[must_use]
    pub fn with_egress_allow(mut self, allow: Vec<IpAddr>) -> Self {
        self.egress_allow = allow;
        self
    }

    /// I-32: install a role-contract store for the structural role-scope gate.
    /// When installed, any directive whose `initiator` role has a contract in
    /// the store and whose `tool_name` is NOT in that role's allowed-tool
    /// categories is blocked fail-closed (rule_id "i32-role-scope-denied").
    /// I-35 will install its own gate arm; this builder is composable.
    #[must_use]
    pub fn with_role_contracts(mut self, store: RoleContractStore) -> Self {
        self.role_contract_store = Some(store);
        self
    }

    /// I-35: install a sequence rule set for the temporal tool-sequence gate (5th arm).
    /// When installed, `run_sequence_gate` evaluates the ordered history buffer against
    /// the TOML-declared rules at every `run_gates` call. Violations are blocked
    /// fail-closed (HaltEnforced -> Block finding). `None` = gate disabled.
    ///
    /// Production wiring: `runtime_setup.rs::build_task_runner_wiring` chains this
    /// after `.with_role_contracts()` (same composable builder pattern).
    #[must_use]
    pub fn with_sequence_rules(mut self, rules: Arc<SequenceRuleSet>) -> Self {
        self.sequence_rules = Some(rules);
        self
    }

    /// Return a shared clone of the internal [`HistoryBuffer`].
    ///
    /// Used by tests and the bench fixture to inject known history entries
    /// via `HistoryBuffer::push` without going through `push_history_entry`.
    #[must_use]
    pub fn history_buffer(&self) -> HistoryBuffer {
        self.history.clone()
    }

    /// Run the E-30 argv gate for a shell directive. Pushes a Block finding on
    /// Deny, a Warn finding on RequireHumanApproval, nothing on Allow. Mirrors
    /// the inline `hook_dispatcher` E-30 block it replaces.
    fn run_argv_gate(&self, directive: &Directive, findings: &mut Vec<HookFinding>) {
        let Some(source) = directive.shell_source() else {
            return;
        };
        let argv: Vec<OsString> = source.split_whitespace().map(OsString::from).collect();
        if argv.is_empty() {
            return;
        }
        match check_argv(&argv) {
            AllowlistVerdict::Deny { reason, citation } => {
                findings.push(HookFinding {
                    tier: HookTier::T1,
                    verdict: HookVerdictKind::Block,
                    rule_id: Some("e30-argv-deny".into()),
                    category: HOOK_EVENT_CATEGORY.into(),
                    message: format!("{SANITIZED_BLOCK_MESSAGE}: {reason}"),
                    fix_template: Some(citation),
                });
            }
            AllowlistVerdict::RequireHumanApproval { reason, citation } => {
                findings.push(HookFinding {
                    tier: HookTier::T1,
                    verdict: HookVerdictKind::Warn,
                    rule_id: Some("e30-argv-approval".into()),
                    category: HOOK_EVENT_CATEGORY.into(),
                    message: format!("human approval required: {reason}"),
                    fix_template: Some(citation),
                });
            }
            AllowlistVerdict::Allow => {}
        }
    }

    /// Run the G-3 blast-radius gates. For a file-targeting directive the
    /// fs-escape gate confirms the target stays inside the worktree root; for
    /// an egress-targeting directive the egress gate confirms the host resolves
    /// only to pinned IPs. Both fail closed. Pushes a Block finding on any
    /// boundary crossing.
    fn run_blast_radius_gate(&self, directive: &Directive, findings: &mut Vec<HookFinding>) {
        use elai_safety::blast_radius::BlastRadiusVerdict;

        // G-3a: filesystem-escape. A directive carrying args.path is checked
        // against the worktree root.
        if let Some(path) = directive.args.get("path").and_then(serde_json::Value::as_str) {
            let verdict = check_fs_target(Path::new(path), &self.worktree_root, path);
            if let BlastRadiusVerdict::Block { kind, post_expansion, .. } = verdict {
                findings.push(HookFinding {
                    tier: HookTier::T1,
                    verdict: HookVerdictKind::Block,
                    rule_id: Some("g3-fs-escape".into()),
                    category: HOOK_EVENT_CATEGORY.into(),
                    message: format!("blast-radius {kind:?}: target escaped worktree -> {post_expansion}"),
                    fix_template: None,
                });
            }
        }

        // G-3b: egress. A directive carrying args.host is checked against the
        // pinned egress allow-set via the production resolver.
        if let Some(host) = directive.args.get("host").and_then(serde_json::Value::as_str) {
            let verdict = check_egress_host(host, &self.egress_allow, resolve_host_std);
            if let BlastRadiusVerdict::Block { kind, post_expansion, .. } = verdict {
                findings.push(HookFinding {
                    tier: HookTier::T1,
                    verdict: HookVerdictKind::Block,
                    rule_id: Some("g3-egress-denied".into()),
                    category: HOOK_EVENT_CATEGORY.into(),
                    message: format!("blast-radius {kind:?}: egress denied -> {post_expansion}"),
                    fix_template: None,
                });
            }
        }
    }

    /// Run the F-1 VibeGate over the directive source. Infallible; merges any
    /// VibeGate findings. Mirrors the inline `hook_dispatcher` F-1 block.
    fn run_vibegate(&self, directive: &Directive, findings: &mut Vec<HookFinding>) {
        let source = directive.shell_source().unwrap_or_else(|| {
            directive
                .args
                .get("source")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
        });
        let file_path = directive
            .args
            .get("path")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let vibe = crate::vibegate::evaluate_vibegate(&directive.tool_name, source, file_path);
        findings.extend(vibe);
    }

    /// Execute the side effect behind a directive AFTER gates pass, emitting a
    /// `DirectiveExecuted` SSB row keyed by outcome.
    ///
    /// `op` is the irreversible side effect (the Tier-1 in-process stub passes
    /// the real tool dispatch closure; the contract test passes a fixture
    /// closure). On a clean run an `Executed` row lands; on a blocked directive
    /// a `Discarded` row lands and `op` never runs; on an `op` error the saga
    /// compensation fires (via `compensate`) and a `CompensationFired` row
    /// lands.
    ///
    /// `compensate` is the E-44 saga compensation closure invoked on crash. It
    /// is `async` and returns `()`; the caller wires it to
    /// `crate::task_runner::e44_run_saga_step` at the production seam. Passing a
    /// no-op compensate in tests is valid (the row still lands).
    pub async fn execute_directive<OpFut, CompFut>(
        &self,
        directive: Directive,
        op: impl FnOnce() -> OpFut,
        compensate: impl FnOnce() -> CompFut,
    ) -> Result<DirectiveOutcome, SupervisorError>
    where
        OpFut: std::future::Future<Output = Result<(), String>>,
        CompFut: std::future::Future<Output = ()>,
    {
        // Gate FIRST. A blocked directive is Discarded and the side effect
        // never runs (fail-closed: the generator cannot reach the sink).
        let gate = self.run_gates(&directive);
        if gate.blocked {
            self.emit_row(&directive, DirectiveOutcome::Discarded).await?;
            return Ok(DirectiveOutcome::Discarded);
        }

        // Gates passed: run the side effect.
        match op().await {
            Ok(()) => {
                self.emit_row(&directive, DirectiveOutcome::Executed).await?;
                Ok(DirectiveOutcome::Executed)
            }
            Err(err) => {
                // Crash path: fire saga compensation to restore state, then
                // record CompensationFired.
                compensate().await;
                self.emit_row(&directive, DirectiveOutcome::CompensationFired)
                    .await?;
                Err(SupervisorError::SideEffectFailed(err))
            }
        }
    }

    /// Emit one `DirectiveExecuted` SSB row for `outcome`. No-op when no SSB
    /// writer is installed (test/offline contexts).
    async fn emit_row(
        &self,
        directive: &Directive,
        outcome: DirectiveOutcome,
    ) -> Result<(), SupervisorError> {
        let Some(ssb) = self.ssb.as_ref() else {
            return Ok(());
        };
        let payload = SsbPayload::DirectiveExecuted(DirectiveExecutedPayload {
            kind: outcome.ssb_kind(),
            tool_name: directive.tool_name.clone(),
            intent_id: directive.intent_id.clone(),
            initiator: directive.initiator.clone(),
            ts_ms: Ssb::now_ms_via_clock(&self.clock),
        });
        ssb.write_payload(&self.agent_id, self.superstep, payload, Vec::new())
            .await?;
        Ok(())
    }
}

impl InProcessSupervisor {
    /// I-32: structural role-scope gate. Blocks any directive whose `initiator`
    /// role has a contract in the store that does NOT include the directive's
    /// `tool_name` in its allowed tool categories. Alignment-dependent roles
    /// (no contract entry) are always passed through unchanged.
    ///
    /// Rule ID: "i32-role-scope-denied". Composable with I-35 5th arm.
    fn run_role_scope_gate(&self, directive: &Directive, findings: &mut Vec<HookFinding>) {
        let Some(store) = &self.role_contract_store else {
            return;
        };
        let Some(role) = crate::role_contracts::agent_role_from_snake_case(&directive.initiator)
        else {
            // Unknown role string: pass through (fail-open for unknown roles,
            // same semantics as PersonaRetained — the gate is not the arbiter
            // of role naming).
            return;
        };
        let Some(contract) = store.get(role) else {
            // Alignment-dependent or uncontracted role: no tool restriction.
            return;
        };
        if !contract.tool_allowed(&directive.tool_name) {
            findings.push(HookFinding {
                tier: HookTier::T1,
                verdict: HookVerdictKind::Block,
                rule_id: Some("i32-role-scope-denied".into()),
                category: HOOK_EVENT_CATEGORY.into(),
                message: format!(
                    "tool '{}' is outside the scope of role '{}' (i32-role-scope-denied)",
                    directive.tool_name, directive.initiator
                ),
                fix_template: None,
            });
        }
    }

    /// I-35: temporal tool-sequence gate (5th arm). Evaluates the ordered history
    /// buffer against the TOML-declared sequence rules. Each rule that triggers on
    /// `directive.tool_name`'s class is evaluated; the first non-Allowed verdict
    /// produces a Block finding (fail-closed).
    ///
    /// When no rule set is installed (`sequence_rules` is `None`), this is a no-op.
    /// Emits a `ToolSequenceVerdict` SSB row when an SSB writer is installed.
    fn run_sequence_gate(&self, directive: &Directive, findings: &mut Vec<HookFinding>) {
        let Some(rules) = &self.sequence_rules else {
            return;
        };
        let trigger_class = tool_class_from_action_type(&directive.tool_name);
        let history_snap = self.history.snapshot();
        let verdict = evaluate_sequence_rules(rules, trigger_class, &history_snap);

        // Emit SSB row when a writer is installed (fire-and-forget; errors are
        // logged but do NOT block the gate outcome — SSB emission is advisory here).
        if let Some(ssb) = self.ssb.as_ref() {
            let (kind, rule_id, unsatisfied) = match &verdict {
                ToolSequenceVerdict::Allowed => (
                    ToolSequenceVerdictKind::Allowed,
                    String::new(),
                    String::new(),
                ),
                ToolSequenceVerdict::OrderViolation { rule_id, unsatisfied_predicate } => (
                    ToolSequenceVerdictKind::OrderViolation,
                    rule_id.clone(),
                    unsatisfied_predicate.clone(),
                ),
                ToolSequenceVerdict::HaltEnforced { rule_id, unsatisfied_predicate } => (
                    ToolSequenceVerdictKind::HaltEnforced,
                    rule_id.clone(),
                    unsatisfied_predicate.clone(),
                ),
            };
            let payload = SsbPayload::ToolSequenceVerdict(ToolSequenceVerdictPayload {
                kind,
                tool_class: trigger_class.to_owned(),
                rule_id,
                unsatisfied_predicate: unsatisfied,
                ts_ms: Ssb::now_ms_via_clock(&self.clock),
            });
            // Spawn a fire-and-forget write rather than awaiting (run_gates is sync).
            let ssb_clone = Arc::clone(ssb);
            let agent_id = self.agent_id.clone();
            let superstep = self.superstep;
            let _ = tokio::task::spawn(async move {
                let _ = ssb_clone
                    .write_payload(&agent_id, superstep, payload, Vec::new())
                    .await;
            });
        }

        if verdict.is_halt() {
            let (rule_id, msg) = match &verdict {
                ToolSequenceVerdict::HaltEnforced { rule_id, unsatisfied_predicate } => {
                    (rule_id.as_str(), unsatisfied_predicate.as_str())
                }
                _ => ("i35-sequence-halt", "tool sequence violation"),
            };
            findings.push(HookFinding {
                tier: HookTier::T1,
                verdict: HookVerdictKind::Block,
                rule_id: Some(format!("i35-sequence-{rule_id}")),
                category: HOOK_EVENT_CATEGORY.into(),
                message: format!("tool-sequence violation (i35): {msg}"),
                fix_template: None,
            });
        }
    }
}

impl Supervisor for InProcessSupervisor {
    fn run_gates(&self, directive: &Directive) -> GateOutcome {
        // Chain order: E-30 argv (fail-closed early), G-3 blast-radius, F-1 VibeGate,
        // I-32 role-scope (4th arm), I-35 sequence gate (5th arm).
        let mut findings = Vec::new();
        self.run_argv_gate(directive, &mut findings);
        self.run_blast_radius_gate(directive, &mut findings);
        self.run_vibegate(directive, &mut findings);
        self.run_role_scope_gate(directive, &mut findings);
        self.run_sequence_gate(directive, &mut findings);
        GateOutcome::from_findings(findings)
    }

    /// I-35: push one completed dispatch history entry into the bounded ring buffer.
    /// Called by [`crate::hook_dispatcher::InProcessHookDispatcher::dispatch_pre_tool_use`]
    /// after each dispatch cycle so subsequent `run_sequence_gate` calls see the
    /// full prior history.
    fn push_history_entry(&self, entry: &HistoryEntry) {
        self.history.push(entry.clone());
    }

    fn execute_directive_dyn<'a>(
        &'a self,
        directive: Directive,
        op: Box<
            dyn FnOnce() -> std::pin::Pin<
                    Box<dyn std::future::Future<Output = Result<(), String>> + Send>,
                > + Send,
        >,
        compensate: Box<
            dyn FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
                + Send,
        >,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<DirectiveOutcome, SupervisorError>> + Send + 'a>,
    > {
        Box::pin(async move {
            // Reuse the inherent execute_directive logic; the boxed closures
            // are forwarded verbatim. This avoids duplicating the gate-check +
            // SSB-emit logic and keeps execute_directive as the single owner.
            self.execute_directive(directive, op, compensate).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bash(cmd: &str) -> Directive {
        Directive::new(
            BASH_TOOL_USE_ACTION,
            serde_json::json!({ "command": cmd }),
            "intent-1",
            "agent-test",
        )
    }

    #[test]
    fn safe_argv_passes_gates() {
        let sup = InProcessSupervisor::new("/tmp/worktree");
        let outcome = sup.run_gates(&bash("ls -la"));
        assert!(!outcome.blocked, "safe argv must not block");
    }

    #[test]
    fn destructive_argv_blocks() {
        let sup = InProcessSupervisor::new("/tmp/worktree");
        let outcome = sup.run_gates(&bash("rm -rf /"));
        assert!(
            outcome.blocked,
            "destructive argv must block via E-30 allowlist"
        );
    }

    #[test]
    fn non_shell_directive_skips_argv_gate() {
        let sup = InProcessSupervisor::new("/tmp/worktree");
        // No `path` key -> the G-3 fs gate is skipped (it only fires on
        // args.path); no shell command -> the E-30 argv gate is skipped. A
        // clean file_write source must not trip the F-1 VibeGate.
        let d = Directive::new(
            "file_write",
            serde_json::json!({ "source": "fn main() {}" }),
            "intent-2",
            "agent-test",
        );
        let outcome = sup.run_gates(&d);
        assert!(!outcome.blocked, "benign file_write must not block");
    }

    #[test]
    fn fs_escape_path_blocks_via_g3() {
        let sup = InProcessSupervisor::new("/tmp/i1-root");
        // A path clearly outside the worktree root must be blocked by the G-3
        // fs-escape gate (fail-closed).
        let d = Directive::new(
            "file_write",
            serde_json::json!({ "path": "/etc/shadow" }),
            "intent-escape",
            "agent-test",
        );
        let outcome = sup.run_gates(&d);
        assert!(outcome.blocked, "out-of-root path must block via G-3 fs-escape");
    }
}
