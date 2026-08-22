//! C-46 — Selective delegator: chooses `(AgentRole, Primitive)` pairs
//! to dispatch given task difficulty + (optional) C-45 credit weights.
//!
//! The delegator is a PURE function over its inputs — no SSB writes,
//! no AgentPool::spawn invocations, no clock. The caller (a future
//! `task_runner::dispatch_with_routing` or similar) is responsible for:
//!
//!   1. Resolving task difficulty via
//!      [`super::difficulty::TaskDifficultyClassifier::classify`].
//!   2. Copying the C-45 `CreditReport.per_role: HashMap<AgentRole, f32>`
//!      into a `BTreeMap<AgentRole, f32>` for deterministic iteration
//!      order (CLAUDE.md anti-stale-training BTreeMap-for-iteration
//!      rule — HashMap iteration order is non-deterministic across
//!      builds).
//!   3. Calling [`SelectiveDelegator::select_workers`] with the resolved
//!      difficulty + per-role credit weights (or `None` for cold-start).
//!   4. Emitting the `SelectiveDelegationDecision` SSB row + (when
//!      applicable) the `SelectiveDelegationCostSaved` SSB row from
//!      the dispatch site.
//!   5. Forwarding the returned `Vec<(AgentRole, Primitive)>` to
//!      `AgentPool::spawn_parallel` (or skipping spawn entirely when
//!      the Vec is empty — lazy-mode direct-answer path).
//!
//! Paper-grounded behaviour (arXiv:2605.05007):
//!
//!   - Direct-answer cost ZERO: when the policy returns an empty Vec
//!     the dispatcher does NOT spawn any worker (no `AgentPool::spawn`
//!     call, no `BackendCostAccrued` row). The Decision beacon still
//!     fires with `dispatched_count = 0` so the dashboard can render
//!     the lazy-mode hit-rate without inferring it from absence.
//!   - Selectivity emerges from the convex blend of difficulty +
//!     credit: roles with low credit are dropped FIRST on low-
//!     difficulty tasks; on high-difficulty tasks the policy collapses
//!     toward the full baseline chain (paper Section 6.1
//!     selectivity-vs-correctness operating point).
//!   - alpha in [0, 1] and S_max <= 0.10 are enforced by
//!     [`super::invariants`] at policy construction; this module trusts
//!     the policy's validity.

use std::collections::BTreeMap;

use elai_ipc_types::AgentRole;
use serde::{Deserialize, Serialize};

use super::policy::RoutingPolicy;

/// Routing primitive — the second axis of the paper's `(model,
/// primitive)` action space. `model` maps to ELAI's `AgentRole` /
/// `RuntimeAdapter` chain (the worker pool); `primitive` is the action
/// type the worker should execute on this dispatch.
///
/// The list is closed + serde-tagged with snake_case so the dashboard
/// can render the discriminant directly and the contract test can
/// fence the variant set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Primitive {
    /// Free-form chat / planning turn — the worker emits a reasoning
    /// trace + optional tool call. Default primitive when the policy
    /// has no stronger signal.
    Chat,
    /// Explicit tool call — the worker dispatches a tool (file read,
    /// search, exec) and returns the structured result.
    ToolCall,
    /// External search — the worker queries the web / knowledge base
    /// and returns a citation set.
    Search,
    /// Code execution — the worker runs a sandboxed shell / cargo /
    /// pytest invocation and returns the structured result.
    CodeExec,
    /// Critic / verifier turn — the worker scores a candidate answer
    /// against acceptance criteria and emits a binary verdict
    /// (paper's `verifier_binary_only` invariant). Reserved for the
    /// Auditor / Critic role variants.
    Verify,
}

impl Primitive {
    /// snake_case wire label for SSB / dashboard rendering.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Chat => "chat",
            Self::ToolCall => "tool_call",
            Self::Search => "search",
            Self::CodeExec => "code_exec",
            Self::Verify => "verify",
        }
    }
}

/// blake3-prefix u64 hash of a string identifier. Matches the
/// `crate::skill_registry::skill_name_hash` precedent (and the
/// `BackendCostAccruedPayload.role_name_hash` precedent in ssb.rs) so
/// dashboards can resolve hashes via canonical role iteration when
/// they need to (the routing dashboard surface does NOT need to
/// resolve task ids — they remain opaque blake3-prefix u64).
#[must_use]
pub fn task_id_hash(task_id: &str) -> u64 {
    let digest = blake3::hash(task_id.as_bytes());
    let bytes = digest.as_bytes();
    u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ])
}

/// Default primitive for an AgentRole. Encodes the canonical "what
/// this role is for" mapping. Customisable later via a policy table;
/// today a hardcoded match keeps the wire surface deterministic +
/// dashboard-renderable.
#[must_use]
pub fn default_primitive(role: AgentRole) -> Primitive {
    match role {
        // Executor + ContextCompiler + MemoryWriter + IdeaFramer are
        // "do the work" roles — chat primitive by default.
        AgentRole::Executor
        | AgentRole::ContextCompiler
        | AgentRole::MemoryWriter
        | AgentRole::IdeaFramer => Primitive::Chat,
        // Auditor + IdeaCritic + per-level auditors verify candidate
        // outputs — verify primitive.
        AgentRole::Auditor
        | AgentRole::IdeaCritic
        | AgentRole::AuditorConcepts
        | AgentRole::AuditorPhases
        | AgentRole::AuditorSteps
        | AgentRole::AuditorBabySteps
        | AgentRole::AuditorBabyStepDetail
        | AgentRole::AuditorProductionGrade => Primitive::Verify,
        // Researcher fetches external evidence — search primitive.
        AgentRole::Researcher => Primitive::Search,
        // Designer drafts schemas / prompts / API shapes — chat
        // primitive (no external evidence by default).
        AgentRole::Designer | AgentRole::PromptEngineer => Primitive::Chat,
        // MediaValidator runs sandboxed render / SSIM comparison —
        // code_exec primitive.
        AgentRole::MediaValidator => Primitive::CodeExec,
        // E-37: InjectionClassifier is a Verify-shape role (yields a
        // Clean/Suspect/Malicious verdict over an input span).
        AgentRole::InjectionClassifier => Primitive::Verify,
    }
}

/// Difficulty threshold below which the lazy-mode direct-answer path
/// fires. Below this threshold the selective delegator returns an
/// empty Vec and the dispatcher skips spawn entirely (paper Section 5
/// lazy-mode invariant: trivial queries collapse to one direct-answer
/// turn at near-zero dispatch cost).
///
/// ELAI design choice (not a paper number): 0.20 mirrors the
/// "obviously trivial" intuition — a task scoring under 0.20 on the
/// `[0, 1]` difficulty scale is shorter than ~100 tokens with no code
/// block, no complexity keywords, no file context, and is single-
/// turn. The contract test asserts the lazy-mode-skip behaviour for
/// the empty-task case.
pub const LAZY_MODE_THRESHOLD: f32 = 0.20;

/// Default per-role credit when no C-45 weights are available (cold
/// start). The paper observes only verifier signal + token cost, so
/// the cold-start path treats every role as equally valuable.
pub const COLD_START_CREDIT: f32 = 0.5;

/// Selective delegator. Constructed once per orchestrator session with
/// a policy snapshot; the actual selection is a pure call.
#[derive(Debug, Clone, Copy)]
pub struct SelectiveDelegator {
    /// Policy snapshot. Validated at construction by
    /// [`RoutingPolicy::new`].
    pub policy: RoutingPolicy,
}

impl SelectiveDelegator {
    /// Construct a delegator with an already-validated policy.
    #[must_use]
    pub fn new(policy: RoutingPolicy) -> Self {
        Self { policy }
    }

    /// Choose the `(AgentRole, Primitive)` dispatch set.
    ///
    /// Returns an empty Vec on:
    ///   - The policy is inert (`!enabled`) — caller is expected to
    ///     fall back to the baseline always-spawn chain. (No SSB row;
    ///     the dispatcher recognises this case and routes around the
    ///     selective path entirely.)
    ///   - Lazy-mode trigger (`difficulty < LAZY_MODE_THRESHOLD`) —
    ///     paper's direct-answer-cost-zero invariant.
    ///
    /// Otherwise returns up to `policy.max_workers_per_query` pairs
    /// sorted by combined `(credit, role)` rank (deterministic via
    /// the BTreeMap iteration order).
    ///
    /// The `available_workers` map is the closed-set worker pool; only
    /// roles present in this map can be returned. Roles absent from
    /// `credit` use [`COLD_START_CREDIT`] as a placeholder.
    pub fn select_workers(
        &self,
        difficulty: f32,
        available_workers: &BTreeMap<AgentRole, ()>,
        credit: Option<&BTreeMap<AgentRole, f32>>,
    ) -> Vec<(AgentRole, Primitive)> {
        if !self.policy.enabled {
            return Vec::new();
        }
        let diff = difficulty.clamp(0.0, 1.0);
        if diff < LAZY_MODE_THRESHOLD {
            return Vec::new();
        }

        // Compute a per-role priority. Higher = more likely included.
        // Priority blends difficulty (push toward more workers) with
        // per-role credit (push toward higher-credit workers). The
        // alpha blend (paper Section 5) controls how aggressively cost
        // pressure trims the worker set.
        let mut ranked: Vec<(AgentRole, f32)> = available_workers
            .iter()
            .map(|(role, _)| {
                let credit_value = credit
                    .and_then(|m| m.get(role).copied())
                    .unwrap_or(COLD_START_CREDIT);
                // Credit is in [-1, 1] per C-45; rescale to [0, 1] so
                // the priority stays non-negative.
                let credit_pos = ((credit_value + 1.0) * 0.5).clamp(0.0, 1.0);
                let priority =
                    (1.0 - self.policy.alpha) * credit_pos + self.policy.alpha * diff;
                (*role, priority)
            })
            .collect();

        // Deterministic sort: priority desc, then by enum declaration
        // order (PartialOrd on AgentRole was added in B-37C).
        ranked.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });

        // Worker count scales with difficulty: trivial tasks dispatch
        // a single worker; hard tasks dispatch up to the cap. Linear
        // ramp between LAZY_MODE_THRESHOLD and 1.0.
        let cap = self.policy.max_workers_per_query.max(1);
        let cap_f = cap as f32;
        let workers_target = ((diff - LAZY_MODE_THRESHOLD)
            / (1.0 - LAZY_MODE_THRESHOLD)
            * cap_f)
            .ceil()
            .clamp(1.0, cap_f) as u32;
        let workers_target = workers_target.min(cap) as usize;

        ranked
            .into_iter()
            .take(workers_target)
            .map(|(role, _)| (role, default_primitive(role)))
            .collect()
    }

    /// Direct-answer skip indicator: true when the delegator collapsed
    /// to lazy-mode (empty Vec returned because difficulty was below
    /// the threshold). Distinguishes "policy chose direct-answer"
    /// from "policy was disabled" so the SSB Decision beacon can
    /// distinguish.
    #[must_use]
    pub fn is_lazy_mode(&self, difficulty: f32) -> bool {
        if !self.policy.enabled {
            return false;
        }
        difficulty.clamp(0.0, 1.0) < LAZY_MODE_THRESHOLD
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routing::policy::DEFAULT_ALPHA;

    fn pilot_workers() -> BTreeMap<AgentRole, ()> {
        let roles = [
            AgentRole::Executor,
            AgentRole::Auditor,
            AgentRole::Researcher,
            AgentRole::Designer,
            AgentRole::PromptEngineer,
            AgentRole::ContextCompiler,
            AgentRole::MemoryWriter,
            AgentRole::MediaValidator,
        ];
        roles.iter().map(|r| (*r, ())).collect()
    }

    fn enabled_policy() -> RoutingPolicy {
        RoutingPolicy::new(true, DEFAULT_ALPHA, 0.10, 8).expect("policy ok")
    }

    #[test]
    fn disabled_policy_returns_empty() {
        let d = SelectiveDelegator::new(RoutingPolicy::default());
        let r = d.select_workers(0.9, &pilot_workers(), None);
        assert!(r.is_empty(), "inert policy must return empty Vec");
    }

    #[test]
    fn lazy_mode_returns_empty_with_zero_dispatch_cost() {
        let d = SelectiveDelegator::new(enabled_policy());
        let r = d.select_workers(0.05, &pilot_workers(), None);
        assert!(r.is_empty(), "lazy-mode must skip dispatch entirely");
        assert!(d.is_lazy_mode(0.05));
        assert!(!d.is_lazy_mode(0.5));
    }

    #[test]
    fn high_difficulty_returns_full_cap() {
        let d = SelectiveDelegator::new(enabled_policy());
        let r = d.select_workers(1.0, &pilot_workers(), None);
        assert_eq!(r.len(), 8, "max difficulty dispatches full cap");
    }

    #[test]
    fn mid_difficulty_returns_subset() {
        let d = SelectiveDelegator::new(enabled_policy());
        let r = d.select_workers(0.5, &pilot_workers(), None);
        assert!(r.len() < 8, "mid difficulty drops workers");
        assert!(r.len() >= 1, "mid difficulty dispatches at least one worker");
    }

    #[test]
    fn credit_weights_shift_selection_order() {
        let d = SelectiveDelegator::new(enabled_policy());
        // Give Researcher a high credit, others low — when the cap
        // trims, Researcher must survive.
        let mut credit: BTreeMap<AgentRole, f32> = BTreeMap::new();
        for role in pilot_workers().keys().copied() {
            credit.insert(role, -0.9);
        }
        credit.insert(AgentRole::Researcher, 1.0);
        let r = d.select_workers(0.3, &pilot_workers(), Some(&credit));
        assert!(
            r.iter().any(|(role, _)| *role == AgentRole::Researcher),
            "high-credit role must survive the cap"
        );
    }

    #[test]
    fn deterministic_ordering_across_calls() {
        let d = SelectiveDelegator::new(enabled_policy());
        let r1 = d.select_workers(0.5, &pilot_workers(), None);
        let r2 = d.select_workers(0.5, &pilot_workers(), None);
        assert_eq!(r1, r2, "selection must be deterministic across calls");
    }

    #[test]
    fn task_id_hash_is_stable() {
        assert_eq!(task_id_hash("hello"), task_id_hash("hello"));
        assert_ne!(task_id_hash("a"), task_id_hash("b"));
    }

    #[test]
    fn default_primitive_covers_all_roles() {
        let _ = default_primitive(AgentRole::Executor);
        let _ = default_primitive(AgentRole::Auditor);
        let _ = default_primitive(AgentRole::Researcher);
        let _ = default_primitive(AgentRole::Designer);
        let _ = default_primitive(AgentRole::PromptEngineer);
        let _ = default_primitive(AgentRole::ContextCompiler);
        let _ = default_primitive(AgentRole::MemoryWriter);
        let _ = default_primitive(AgentRole::MediaValidator);
        let _ = default_primitive(AgentRole::IdeaFramer);
        let _ = default_primitive(AgentRole::IdeaCritic);
        let _ = default_primitive(AgentRole::AuditorConcepts);
        let _ = default_primitive(AgentRole::AuditorPhases);
        let _ = default_primitive(AgentRole::AuditorSteps);
        let _ = default_primitive(AgentRole::AuditorBabySteps);
        let _ = default_primitive(AgentRole::AuditorBabyStepDetail);
    }

    #[test]
    fn primitive_as_str_round_trip() {
        for p in [
            Primitive::Chat,
            Primitive::ToolCall,
            Primitive::Search,
            Primitive::CodeExec,
            Primitive::Verify,
        ] {
            let s = p.as_str();
            assert!(!s.is_empty());
        }
    }
}
