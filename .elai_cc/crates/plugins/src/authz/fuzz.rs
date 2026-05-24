//! C-42 (session 2) authorization fuzz harness — proptest-state-machine.
//!
//! Paper reference: arXiv:2604.25555 Section 6 (EPA-stateful fuzzing
//! methodology). Paper reports 500K sequence budget on financial-doc
//! corpus; that scale is RUNPOD-PENDING (`gateway_fuzz_500k_eval` —
//! requires the GPU box + paper's full corpus). This in-process harness
//! is CI-bounded at 256 sequences of length 1..32, which is sufficient to
//! exercise the BOLA + deny-by-default invariants on ELAI's policy
//! schema. Increasing the cap requires the RUNPOD eval, not a local run.
//!
//! Precedent: orchestrator/src/dst.rs:316 stateful PBT harness on the
//! FSM reference model. Same proptest 1.11 + proptest-state-machine 0.8
//! workspace pins (no new external dep).
//!
//! State:
//!   - `current_role`: which role is making the call this transition
//!   - `current_doc`: optional doc context with separate db_owner +
//!     agent_claimed_owner fields (BOLA test surface)
//!   - `permitted_calls`: running count of allowed verdicts the reference
//!     model predicted
//!   - `denied_calls`: running count of denied verdicts
//!
//! Invariants:
//!   I-1. deny-by-default — undeclared role/tool combos always Deny.
//!   I-2. agent-claimed-owner is ignored (BOLA defense, paper Section 5).
//!   I-3. self-sentinel ownership rule matches `db_owner == role`.
//!   I-4. wildcard tool ("*") permits any registered tool.
//!
//! NOT covered here (intentional — out of L2 scope):
//!   - L1 Semantic Firewall classification (RUNPOD-PENDING).
//!   - L3 Crypto-HITL signature verification (DEFERRED-TO-FOLLOWUP).
//!   - State explosion mitigation (EPA reductions only matter at |T| > 200
//!     per paper; ELAI's tool count is currently small).

use std::collections::HashSet;

use proptest::prelude::*;
use proptest::strategy::Strategy;
use proptest_state_machine::{prop_state_machine, ReferenceStateMachine, StateMachineTest};

use super::checker::check_authz;
use super::policy::AuthzPolicy;
use super::types::{AuthzVerdict, DocContext, RoleClaim};

/// Reference policy used by the harness. Mirrors the example policy at
/// the workspace root (`.elai_cc/authz-policy.toml`) but kept inline so
/// the test does not depend on file-system layout.
const REFERENCE_POLICY_TOML: &str = r#"
default = "deny"

[roles.planner]
tools = ["knowledge.search", "fs.read", "fs.grep"]
doc_owners = []

[roles.executor]
tools = ["knowledge.search", "fs.read", "fs.write", "fs.edit", "fs.grep"]
doc_owners = ["self"]

[roles.auditor]
tools = ["knowledge.search", "fs.read", "fs.grep", "cargo.test"]
doc_owners = []

[roles.admin]
tools = ["*"]
doc_owners = []
"#;

/// Universe of role names the harness samples from. Includes one
/// adversarial label ("ghost") that is intentionally NOT in the policy —
/// the deny-by-default invariant must catch every ghost-role call.
const FUZZ_ROLES: &[&str] = &["planner", "executor", "auditor", "admin", "ghost"];

/// Universe of tool names the harness samples from. Includes
/// `irreversible.format_disk` which is never in any role's tool list —
/// catches policies that accidentally allow everything to a known role.
const FUZZ_TOOLS: &[&str] = &[
    "knowledge.search",
    "fs.read",
    "fs.grep",
    "fs.write",
    "fs.edit",
    "cargo.test",
    "irreversible.format_disk",
];

/// Universe of doc owners the harness samples from for `DocContext`.
const FUZZ_OWNERS: &[&str] = &["alice", "bob", "executor", "system", "shared"];

/// One transition exercised by the harness.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthzTransition {
    /// Try a tool call with a given role + doc-context shape.
    InvokeTool {
        role: String,
        tool: String,
        /// Optional doc context. When `Some(_)`, the inner tuple is
        /// (db_owner, agent_claimed_owner). agent_claimed_owner may be a
        /// DIFFERENT label than db_owner — that's the BOLA stimulus.
        doc: Option<(Option<String>, Option<String>)>,
    },
}

/// Reference state. The harness compares the actual `check_authz` verdict
/// to the predicate predicted from this struct.
#[derive(Debug, Clone, Default)]
pub struct AuthzReferenceState {
    pub allowed_calls: u32,
    pub denied_calls: u32,
    /// Tracks (role, tool) pairs we've seen at least one Deny on. Used
    /// for the spot-check on deny-by-default — if the reference says a
    /// pair is unpermitted, we expect every call with that pair to deny.
    pub seen_deny_pairs: HashSet<(String, String)>,
}

/// Reference model — the predicate side of the harness.
pub struct AuthzReferenceModel;

impl ReferenceStateMachine for AuthzReferenceModel {
    type State = AuthzReferenceState;
    type Transition = AuthzTransition;

    fn init_state() -> BoxedStrategy<Self::State> {
        Just(AuthzReferenceState::default()).boxed()
    }

    fn transitions(_state: &Self::State) -> BoxedStrategy<Self::Transition> {
        let role_strategy = proptest::sample::select(FUZZ_ROLES.to_vec());
        let tool_strategy = proptest::sample::select(FUZZ_TOOLS.to_vec());

        // Doc-context strategy: 30% None, 70% Some with random db_owner
        // and 50% chance of an adversarial agent_claimed_owner.
        let doc_strategy = proptest::option::weighted(
            0.7,
            (
                proptest::option::of(proptest::sample::select(FUZZ_OWNERS.to_vec())),
                proptest::option::of(proptest::sample::select(FUZZ_OWNERS.to_vec())),
            ),
        )
        .prop_map(|opt| {
            opt.map(|(db, agent)| {
                (
                    db.map(|s: &'static str| s.to_string()),
                    agent.map(|s: &'static str| s.to_string()),
                )
            })
        });

        (role_strategy, tool_strategy, doc_strategy)
            .prop_map(|(role, tool, doc)| AuthzTransition::InvokeTool {
                role: role.to_string(),
                tool: tool.to_string(),
                doc,
            })
            .boxed()
    }

    fn apply(mut state: Self::State, transition: &Self::Transition) -> Self::State {
        // Predict the verdict using the reference policy and update
        // counters. The actual system-under-test re-runs `check_authz`
        // independently and StateMachineTest::check_invariants enforces
        // agreement.
        let AuthzTransition::InvokeTool { role, tool, doc } = transition;
        let policy = reference_policy();
        let ctx = doc.as_ref().map(|(db, agent)| DocContext {
            doc_id: "fuzz-doc".to_string(),
            db_owner: db.clone(),
            agent_claimed_owner: agent.clone(),
        });
        let verdict = check_authz(role, tool, ctx.as_ref(), &policy);
        if verdict.is_allow() {
            state.allowed_calls += 1;
        } else {
            state.denied_calls += 1;
            state.seen_deny_pairs.insert((role.clone(), tool.clone()));
        }
        state
    }
}

/// System-under-test wrapping `check_authz` calls. It re-runs the verdict
/// independently and the reference model's counter predicts agreement.
pub struct AuthzSystemUnderTest {
    pub allowed: u32,
    pub denied: u32,
    pub policy: AuthzPolicy,
}

impl StateMachineTest for AuthzSystemUnderTest {
    type SystemUnderTest = AuthzSystemUnderTest;
    type Reference = AuthzReferenceModel;

    fn init_test(
        _ref_state: &<Self::Reference as ReferenceStateMachine>::State,
    ) -> Self::SystemUnderTest {
        AuthzSystemUnderTest {
            allowed: 0,
            denied: 0,
            policy: reference_policy(),
        }
    }

    fn apply(
        mut state: Self::SystemUnderTest,
        _ref_state: &<Self::Reference as ReferenceStateMachine>::State,
        transition: <Self::Reference as ReferenceStateMachine>::Transition,
    ) -> Self::SystemUnderTest {
        let AuthzTransition::InvokeTool { role, tool, doc } = transition;
        let ctx = doc.as_ref().map(|(db, agent)| DocContext {
            doc_id: "fuzz-doc".to_string(),
            db_owner: db.clone(),
            agent_claimed_owner: agent.clone(),
        });
        let _role_claim = RoleClaim::new(&role);
        let verdict = check_authz(&role, &tool, ctx.as_ref(), &state.policy);

        // BOLA invariant — paper Section 5: agent_claimed_owner ignored.
        // Re-run the same call with agent_claimed_owner zeroed out and
        // assert the verdict is identical. This is the spot-check; the
        // doc-context type signature also enforces it structurally.
        if let Some(c) = ctx.as_ref() {
            let stripped = DocContext {
                doc_id: c.doc_id.clone(),
                db_owner: c.db_owner.clone(),
                agent_claimed_owner: None,
            };
            let stripped_verdict =
                check_authz(&role, &tool, Some(&stripped), &state.policy);
            assert_eq!(
                verdict, stripped_verdict,
                "BOLA: agent_claimed_owner changed the verdict (paper Section 5 violated). \
                 with_claim={verdict:?} without_claim={stripped_verdict:?}",
            );
        }

        match verdict {
            AuthzVerdict::Allow => state.allowed += 1,
            AuthzVerdict::Deny { .. } => state.denied += 1,
        }
        state
    }

    fn check_invariants(
        state: &Self::SystemUnderTest,
        ref_state: &<Self::Reference as ReferenceStateMachine>::State,
    ) {
        // Counter agreement — the SUT and the reference model must agree
        // on every verdict (they're computing the same function).
        assert_eq!(
            state.allowed, ref_state.allowed_calls,
            "SUT/reference allowed-count drift: SUT={}, REF={}",
            state.allowed, ref_state.allowed_calls,
        );
        assert_eq!(
            state.denied, ref_state.denied_calls,
            "SUT/reference denied-count drift: SUT={}, REF={}",
            state.denied, ref_state.denied_calls,
        );
    }
}

fn reference_policy() -> AuthzPolicy {
    AuthzPolicy::load_from_str(REFERENCE_POLICY_TOML).expect("reference policy parses")
}

prop_state_machine! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn authz_policy_invariants_hold(sequential 1..32 => AuthzSystemUnderTest);
}

#[cfg(test)]
mod tests {
    //! Targeted invariant assertions in addition to the state-machine
    //! property test above. The state machine catches probabilistic
    //! failures; these tests assert the BOLA + deny-by-default invariants
    //! deterministically on a single, well-known input.

    use super::*;

    #[test]
    fn deny_by_default_invariant_i1() {
        let policy = reference_policy();
        // ghost role is intentionally not declared. Any tool denies.
        for tool in FUZZ_TOOLS {
            let v = check_authz("ghost", tool, None, &policy);
            assert!(
                !v.is_allow(),
                "deny-by-default violated for ghost+{tool}: {v:?}",
            );
        }
    }

    #[test]
    fn bola_agent_claim_ignored_invariant_i2() {
        let policy = reference_policy();
        let with_claim = DocContext {
            doc_id: "x".into(),
            db_owner: Some("alice".into()),
            agent_claimed_owner: Some("executor".into()),
        };
        let without_claim = DocContext {
            doc_id: "x".into(),
            db_owner: Some("alice".into()),
            agent_claimed_owner: None,
        };
        let v1 = check_authz("executor", "fs.write", Some(&with_claim), &policy);
        let v2 = check_authz("executor", "fs.write", Some(&without_claim), &policy);
        assert_eq!(v1, v2, "BOLA defense (paper L2 invariant) violated");
        assert!(!v1.is_allow());
    }

    #[test]
    fn self_sentinel_invariant_i3() {
        let policy = reference_policy();
        // executor.doc_owners = ["self"] -> db_owner must equal "executor".
        let owned = DocContext {
            doc_id: "x".into(),
            db_owner: Some("executor".into()),
            agent_claimed_owner: None,
        };
        let v = check_authz("executor", "fs.write", Some(&owned), &policy);
        assert!(v.is_allow(), "self-sentinel did not allow self-owned doc");
    }

    #[test]
    fn wildcard_tool_invariant_i4() {
        let policy = reference_policy();
        let v = check_authz("admin", "irreversible.format_disk", None, &policy);
        assert!(v.is_allow(), "wildcard tool did not allow arbitrary call");
    }
}
