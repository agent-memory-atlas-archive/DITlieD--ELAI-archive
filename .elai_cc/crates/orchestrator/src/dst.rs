//! Deterministic Simulation Testing (DST) skeleton types.
//!
//! Phase 3I.4 deliverable: types and trait stubs for proptest-state-machine
//! reference model. Implementations are stubs — 3I.8 fills in real logic.

use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Duration;

use crate::agent_pool::AgentRole;
use crate::fsm::FsmState;
use crate::ssb::SsbActionType;

// ---------------------------------------------------------------------------
// SimFault — injectable faults for deterministic simulation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimFault {
    AgentCrash { role: AgentRole },
    NetworkPartition {
        duration: Duration,
        from_node: Option<u64>,
        to_node: Option<u64>,
    },
    SlowTool { tool_name: String, delay: Duration },
    SsbWriteFailure,
}

// ---------------------------------------------------------------------------
// StateSnapshot — captured system state at a point in time
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateSnapshot {
    pub fsm_state: FsmState,
    pub ssb_row_count: u64,
    pub active_agents: HashSet<AgentRole>,
    pub tick: u64,
}

// ---------------------------------------------------------------------------
// SimulatorRuntime — deterministic time + entropy for DST
// ---------------------------------------------------------------------------

pub struct SimulatorRuntime {
    pub clock: elai_core::SeededClock,
    pub rng: elai_core::SeededRng,
    pub fault_queue: VecDeque<(SimFault, Duration)>,
    pub sim_time: Duration,
    pub fsm_state: FsmState,
    pub ssb_row_count: u64,
    pub active_agents: HashSet<AgentRole>,
}

impl SimulatorRuntime {
    pub fn new(seed: u64) -> Self {
        Self {
            clock: elai_core::SeededClock::new(),
            rng: elai_core::SeededRng::new(seed),
            fault_queue: VecDeque::new(),
            sim_time: Duration::ZERO,
            fsm_state: FsmState::Idle,
            ssb_row_count: 0,
            active_agents: HashSet::new(),
        }
    }

    /// Advance simulated time by `duration`, expiring faults whose deadline has passed.
    pub fn advance_time(&mut self, duration: Duration) {
        self.sim_time = self.sim_time.saturating_add(duration);
        self.fault_queue.retain(|(_fault, expiry)| *expiry > self.sim_time);
    }

    /// Queue a fault with an expiry time computed from the fault's duration.
    pub fn inject_fault(&mut self, fault: SimFault) {
        let fault_duration = match &fault {
            SimFault::NetworkPartition { duration, .. } => *duration,
            SimFault::SlowTool { delay, .. } => *delay,
            SimFault::AgentCrash { .. } | SimFault::SsbWriteFailure => Duration::MAX,
        };
        let expiry = self.sim_time.saturating_add(fault_duration);
        self.fault_queue.push_back((fault, expiry));
    }

    /// Return a snapshot of current simulated system state.
    pub fn snapshot_state(&self) -> StateSnapshot {
        let tick = u64::try_from(self.sim_time.as_millis()).unwrap_or(u64::MAX);
        StateSnapshot {
            fsm_state: self.fsm_state,
            ssb_row_count: self.ssb_row_count,
            active_agents: self.active_agents.clone(),
            tick,
        }
    }

    /// Check whether any active `NetworkPartition` fault blocks traffic from
    /// `sender_node_id` to `receiver_node_id`. Broadcast faults (from_node=None)
    /// block all senders. Directional faults match both from_node and to_node.
    pub fn has_active_partition(
        &self,
        sender_node_id: u64,
        receiver_node_id: Option<u64>,
    ) -> Option<&SimFault> {
        self.fault_queue.iter().find_map(|(fault, _expiry)| {
            if let SimFault::NetworkPartition {
                from_node, to_node, ..
            } = fault
            {
                let from_matches = match from_node {
                    None => true,
                    Some(from) => *from == sender_node_id,
                };
                let to_matches = match (to_node, receiver_node_id) {
                    (None, _) => true,
                    (Some(to), Some(recv)) => *to == recv,
                    (Some(_), None) => true,
                };
                if from_matches && to_matches {
                    Some(fault)
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    /// Check whether any active `AgentCrash` fault is present.
    pub fn has_active_agent_crash(&self) -> bool {
        self.fault_queue
            .iter()
            .any(|(fault, _)| matches!(fault, SimFault::AgentCrash { .. }))
    }
}

// ---------------------------------------------------------------------------
// FsmEvent — the 10 event types the reference model processes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FsmEvent {
    PhaseTransition { from: FsmState, to: FsmState },
    AgentSpawn { role: AgentRole },
    AgentCrash { role: AgentRole },
    AgentCleanup { role: AgentRole },
    ToolCall { tool_name: String },
    FileWrite { path: String },
    GrcStep { step_type: String },
    SsbWrite { action_type: SsbActionType },
    ConfigChange,
    SuperstepBoundary,
}

// ---------------------------------------------------------------------------
// ReferenceState — pure oracle state (simpler than real system)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceState {
    pub fsm_state: FsmState,
    pub ssb_row_count: u64,
    pub active_agents: HashSet<AgentRole>,
    pub crashed_agents: HashSet<AgentRole>,
    pub cleaned_agents: HashSet<AgentRole>,
    pub last_value_channels: HashMap<String, u64>,
    pub topic_channels: HashMap<String, Vec<u64>>,
    pub ephemeral_present: HashSet<String>,
}

impl ReferenceState {
    pub fn initial() -> Self {
        Self {
            fsm_state: FsmState::Idle,
            ssb_row_count: 0,
            active_agents: HashSet::new(),
            crashed_agents: HashSet::new(),
            cleaned_agents: HashSet::new(),
            last_value_channels: HashMap::new(),
            topic_channels: HashMap::new(),
            ephemeral_present: HashSet::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// FsmReferenceModel — struct used as ReferenceStateMachine (impl in #[cfg(test)])
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct FsmReferenceModel;

// ---------------------------------------------------------------------------
// ActualSystem — the real system under test (wraps FSM + SSB)
// ---------------------------------------------------------------------------

pub struct ActualSystem {
    pub fsm_state: FsmState,
    pub ssb_row_count: u64,
    pub active_agents: HashSet<AgentRole>,
    pub runtime: SimulatorRuntime,
}

impl ActualSystem {
    pub fn new(runtime: SimulatorRuntime) -> Self {
        Self {
            fsm_state: FsmState::Idle,
            ssb_row_count: 0,
            active_agents: HashSet::new(),
            runtime,
        }
    }
}

// ---------------------------------------------------------------------------
// Reference model preconditions — pure logic, no proptest dependency
// ---------------------------------------------------------------------------

impl FsmReferenceModel {
    /// Check whether a transition is valid for the given reference state.
    pub fn preconditions(state: &ReferenceState, transition: &FsmEvent) -> bool {
        match transition {
            FsmEvent::PhaseTransition { from, to } => {
                *from == state.fsm_state && from.can_transition_to(*to)
            }
            FsmEvent::AgentSpawn { role } => !state.active_agents.contains(role),
            FsmEvent::AgentCrash { role } => state.active_agents.contains(role),
            FsmEvent::AgentCleanup { role } => state.crashed_agents.contains(role),
            FsmEvent::ToolCall { .. } => {
                matches!(
                    state.fsm_state,
                    FsmState::Executing | FsmState::Auditing
                )
            }
            FsmEvent::FileWrite { .. } => {
                matches!(state.fsm_state, FsmState::Executing)
            }
            FsmEvent::GrcStep { .. } => {
                !matches!(state.fsm_state, FsmState::Idle | FsmState::Complete)
            }
            FsmEvent::SsbWrite { .. } => {
                // SSB writes valid in active states, excluding Idle and Complete.
                // Complete is terminal — no new audit trail entries after phase end.
                !matches!(state.fsm_state, FsmState::Idle | FsmState::Complete)
            }
            FsmEvent::ConfigChange => {
                matches!(state.fsm_state, FsmState::Configuring)
            }
            FsmEvent::SuperstepBoundary => true,
        }
    }

    pub fn apply(mut state: ReferenceState, transition: &FsmEvent) -> ReferenceState {
        match transition {
            FsmEvent::PhaseTransition { to, .. } => {
                state.fsm_state = *to;
            }
            FsmEvent::AgentSpawn { role } => {
                state.active_agents.insert(*role);
            }
            FsmEvent::AgentCrash { role } => {
                state.active_agents.remove(role);
                state.crashed_agents.insert(*role);
            }
            FsmEvent::AgentCleanup { role } => {
                state.crashed_agents.remove(role);
                state.cleaned_agents.insert(*role);
            }
            FsmEvent::ToolCall { .. } => {}
            FsmEvent::FileWrite { .. } => {}
            FsmEvent::GrcStep { .. } => {}
            FsmEvent::SsbWrite { .. } => {
                state.ssb_row_count += 1;
            }
            FsmEvent::ConfigChange => {}
            FsmEvent::SuperstepBoundary => {
                state.ephemeral_present.clear();
            }
        }
        state
    }

    /// Check invariants between SUT and reference state.
    pub fn check_invariants(
        sut_fsm: FsmState,
        sut_ssb_row_count: u64,
        sut_active_agents: &HashSet<AgentRole>,
        ref_state: &ReferenceState,
    ) {
        assert_eq!(
            sut_fsm, ref_state.fsm_state,
            "FSM state mismatch: actual={:?} reference={:?}",
            sut_fsm, ref_state.fsm_state
        );
        assert_eq!(
            sut_ssb_row_count, ref_state.ssb_row_count,
            "SSB row count mismatch: actual={} reference={}",
            sut_ssb_row_count, ref_state.ssb_row_count
        );
        assert_eq!(
            sut_active_agents, &ref_state.active_agents,
            "Active agents mismatch: actual={:?} reference={:?}",
            sut_active_agents, ref_state.active_agents
        );
    }
}

// ---------------------------------------------------------------------------
// proptest trait impls + stateful PBT — unit test scope only
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest_state_machine::{prop_state_machine, ReferenceStateMachine, StateMachineTest};

    /// phase-3I.proptest: default proptest case count when `PROPTEST_CASES` is
    /// unset or unparseable. Matches the proptest crate default (256) so CI
    /// behaviour is unchanged unless the env var is supplied (e.g. a nightly
    /// DST soak run exporting `PROPTEST_CASES=10000`).
    const DEFAULT_PROPTEST_CASES: u32 = 256;

    /// Resolve the proptest case count from the `PROPTEST_CASES` env var,
    /// falling back to [`DEFAULT_PROPTEST_CASES`]. Lets operators dial the DST
    /// state-machine sweep depth without recompiling.
    fn proptest_cases() -> u32 {
        std::env::var("PROPTEST_CASES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_PROPTEST_CASES)
    }

    impl ReferenceStateMachine for FsmReferenceModel {
        type State = ReferenceState;
        type Transition = FsmEvent;

        fn init_state() -> BoxedStrategy<Self::State> {
            Just(ReferenceState::initial()).boxed()
        }

        fn transitions(state: &Self::State) -> BoxedStrategy<Self::Transition> {
            let valid_targets = state.fsm_state.valid_targets();

            let mut strategies: Vec<BoxedStrategy<FsmEvent>> = Vec::new();

            if !valid_targets.is_empty() {
                let from = state.fsm_state;
                let targets: Vec<FsmState> = valid_targets.to_vec();
                strategies.push(
                    proptest::sample::select(targets)
                        .prop_map(move |to| FsmEvent::PhaseTransition { from, to })
                        .boxed(),
                );
            }

            strategies.push(
                proptest::sample::select(vec![
                    AgentRole::Executor,
                    AgentRole::Auditor,
                    AgentRole::Researcher,
                    AgentRole::Designer,
                    AgentRole::PromptEngineer,
                    AgentRole::ContextCompiler,
                    AgentRole::MemoryWriter,
                    AgentRole::MediaValidator,
                ])
                .prop_map(|role| FsmEvent::AgentSpawn { role })
                .boxed(),
            );

            if !state.active_agents.is_empty() {
                let active: Vec<AgentRole> =
                    state.active_agents.iter().copied().collect();
                strategies.push(
                    proptest::sample::select(active)
                        .prop_map(|role| FsmEvent::AgentCrash { role })
                        .boxed(),
                );
            }

            if !state.crashed_agents.is_empty() {
                let crashed: Vec<AgentRole> =
                    state.crashed_agents.iter().copied().collect();
                strategies.push(
                    proptest::sample::select(crashed)
                        .prop_map(|role| FsmEvent::AgentCleanup { role })
                        .boxed(),
                );
            }

            // ToolCall — generated in any state; preconditions filter
            strategies.push(
                proptest::sample::select(vec![
                    "read_file", "write_file", "bash", "grep", "glob",
                ])
                .prop_map(|name| FsmEvent::ToolCall {
                    tool_name: name.to_string(),
                })
                .boxed(),
            );

            // FileWrite — generated in any state; preconditions filter
            strategies.push(
                proptest::sample::select(vec![
                    "src/main.rs", "src/lib.rs", "tests/test.rs",
                ])
                .prop_map(|p| FsmEvent::FileWrite {
                    path: p.to_string(),
                })
                .boxed(),
            );

            // GrcStep — generated in any state; preconditions filter
            strategies.push(
                proptest::sample::select(vec![
                    "observe", "hypothesize", "plan_action", "converge", "verify",
                ])
                .prop_map(|st| FsmEvent::GrcStep {
                    step_type: st.to_string(),
                })
                .boxed(),
            );

            strategies.push(
                proptest::sample::select(vec![
                    SsbActionType::FileWrite,
                    SsbActionType::ToolResult,
                    SsbActionType::TestOutcome,
                    SsbActionType::Finding,
                    SsbActionType::PhaseTransition,
                    SsbActionType::ReasoningTrace,
                    SsbActionType::ResearchStep,
                ])
                .prop_map(|action_type| FsmEvent::SsbWrite { action_type })
                .boxed(),
            );

            strategies.push(Just(FsmEvent::ConfigChange).boxed());
            strategies.push(Just(FsmEvent::SuperstepBoundary).boxed());

            proptest::strategy::Union::new(strategies).boxed()
        }

        fn preconditions(state: &Self::State, transition: &Self::Transition) -> bool {
            FsmReferenceModel::preconditions(state, transition)
        }

        fn apply(state: Self::State, transition: &Self::Transition) -> Self::State {
            FsmReferenceModel::apply(state, transition)
        }
    }

    impl StateMachineTest for ActualSystem {
        type SystemUnderTest = ActualSystem;
        type Reference = FsmReferenceModel;

        fn init_test(
            _ref_state: &<Self::Reference as ReferenceStateMachine>::State,
        ) -> Self::SystemUnderTest {
            ActualSystem::new(SimulatorRuntime::new(42))
        }

        fn apply(
            mut state: Self::SystemUnderTest,
            _ref_state: &<Self::Reference as ReferenceStateMachine>::State,
            transition: <Self::Reference as ReferenceStateMachine>::Transition,
        ) -> Self::SystemUnderTest {
            match &transition {
                FsmEvent::PhaseTransition { to, .. } => {
                    state.fsm_state = *to;
                }
                FsmEvent::AgentSpawn { role } => {
                    state.active_agents.insert(*role);
                }
                FsmEvent::AgentCrash { role } => {
                    state.active_agents.remove(role);
                }
                FsmEvent::AgentCleanup { .. } => {}
                FsmEvent::SsbWrite { .. } => {
                    state.ssb_row_count += 1;
                }
                _ => {}
            }
            state
        }

        fn check_invariants(
            state: &Self::SystemUnderTest,
            ref_state: &<Self::Reference as ReferenceStateMachine>::State,
        ) {
            FsmReferenceModel::check_invariants(
                state.fsm_state,
                state.ssb_row_count,
                &state.active_agents,
                ref_state,
            );
        }
    }

    prop_state_machine! {
        // phase-3I.proptest: case count is env-driven (PROPTEST_CASES) with a
        // 256 default; see `proptest_cases()`.
        #![proptest_config(ProptestConfig::with_cases(proptest_cases()))]

        #[test]
        fn fsm_reference_model_matches(sequential 1..200 => ActualSystem);
    }
}
