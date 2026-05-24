//! C-42 (session 2): Semantic Gateway zero-trust authorization (L2 only).
//!
//! Paper grounding: arXiv:2604.25555 "From CRUD to Autonomous Agents: Formal
//! Validation and Zero-Trust Security for Semantic Gateways in AI-Native
//! Enterprise Systems" (Ignacio Peyrano, MEDIUM reliability — single-author
//! preprint, no peer-review label). See workflow/papers/semantic-gateway-fuzzer.md.
//!
//! Scope discipline (READ THIS FIRST):
//!   - Paper specifies a 3-layer gateway (L1 Semantic Firewall DeBERTa-v3,
//!     L2 OPA/Rego RBAC + BOLA defense, L3 Crypto-HITL).
//!   - C-42 session 2 implements **L2 ONLY**. L1 is marked RUNPOD-PENDING
//!     (DeBERTa-v3 inference + the eval corpus need a GPU box). L3 is
//!     DEFERRED-TO-FOLLOWUP (crypto-HITL is a separate plan, not part of
//!     this session).
//!   - Verdict is **binary** ({Allow, Deny}) per paper Section 4 — there is
//!     no 4-way Allow/Deny/Quarantine/Audit. The plan's pre-rebase claim of
//!     a 4-way verdict was corrected against the paper interface.
//!
//! Composition with C-35 ToolGate (ELAI design decision — paper does not
//! specify): AuthzPolicy runs **before** `ToolGate::check_tool_allowed`. If
//! AuthzPolicy denies, the ToolGate is never consulted (short-circuit).
//! Rationale: AuthzPolicy is a role-based RBAC check (paper L2), the
//! ToolGate is a per-FSM-state allowlist; the RBAC layer is strictly
//! "is this role permitted to use this tool at all" and the gate is "given
//! that the role *is* permitted, is the tool appropriate for the current
//! FSM state". See `compose::compose_with_tool_gate`.
//!
//! Submodules:
//!   - [`types`]: [`AuthzVerdict`], [`DocContext`], [`RoleClaim`], [`AuthzError`]
//!   - [`policy`]: [`AuthzPolicy`] + TOML loader
//!   - [`checker`]: [`check_authz`] pure function
//!   - [`compose`]: [`compose_with_tool_gate`] ordered gating
//!   - [`fuzz`]: proptest-state-machine harness (test-only)
//!
//! Wired into [`crate::mcp_bridge::DefaultMcpBridge`] via
//! `dispatch_tool_call_with_authz` (mcp_bridge.rs).

pub mod checker;
pub mod compose;
#[cfg(test)]
pub mod fuzz;
pub mod policy;
pub mod types;

pub use checker::check_authz;
pub use compose::{compose_with_tool_gate, ComposedVerdict};
pub use policy::{AuthzPolicy, AuthzPolicyConfig, RoleEntry};
pub use types::{AuthzError, AuthzVerdict, DocContext, RoleClaim};

// RUNPOD-PENDING: gateway_l1_deberta_v3_eval
//   Layer 1 (Semantic Firewall) ships only after the remote GPU box has
//   the DeBERTa-v3 classifier checkpoint + the prompt-injection eval
//   corpus. Paper reports epsilon_1 = 0.006 (99.4% block rate) but the
//   eval is on the paper's own financial-doc corpus, not ELAI's
//   distribution — local re-measurement is required before shipping L1.
//
// RUNPOD-PENDING: gateway_fuzz_500k_eval
//   The in-process fuzz harness (plugins/src/authz/fuzz.rs) is bounded at
//   256 sequences. Paper's 500K-sequence EPA-stateful run requires the
//   remote box (CPU + memory for the abstract-state-graph reductions).
//   Local runs are sufficient to exercise the BOLA + deny-by-default
//   invariants; full eval reporting waits for the GPU/CPU box pass.
//
// DEFERRED-TO-FOLLOWUP (not runpod-pending — separate plan):
//   Layer 3 Crypto-HITL is its own plan. The SsbActionType::GatewayHitlPending
//   variant lands here so the followup can plug emitters without a second
//   SCHEMA bump; no producer is wired in session 2.
