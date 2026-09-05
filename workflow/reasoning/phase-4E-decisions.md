Phase 4E Decisions
Date: 2026-04-06
Session: s-1810400

WHAT
  Pipeline Factory L2: deterministic builder that transforms PipelineSpec into ExecutablePipeline.
  DAG repair loop with bounded rounds (T_DAG=10), novel step detection, structural validation.

KEY DECISIONS

  1. Repair strategies limited to REORDER + SUBSTITUTE only.
     Why: external-analysis.txt L173 adopts exactly these two from AutoB2G.
     Previously had INSERT_DEP + SPLIT (ungrounded inventions). Removed after adversarial audit.
     Resource overflow and ambiguous missing deps escalate immediately instead of repair.

  2. FSM state predicates use compiled Predicate type (not raw String).
     Why: rules.txt L161 compile_predicate validates at insertion time, not evaluation time.
     Prevents runtime predicate parse failures. Added is_terminal field for terminal states.

  3. novel_step_ids added to ExecutablePipeline as Vec<StepId>.
     Why: phase-4F.txt L99-100 consumes this to decide FEASIBILITY_PROBE skip.
     Detection: tool_id not in ToolRegistry.available_tools = novel.

  4. AgentPhaseConfig (not AgentRoleConfig) + RuleBinding types defined.
     Why: pipeline-factory.txt sec 2 "configures agents" and "installs rules" are L2 deliverables.
     AgentPhaseConfig references orchestrator.txt L486 AgentGrcConfig for budget structure.
     RuleBinding references rules.txt L34 rule schema for rule_ids.

  5. Newtypes: PipelineId, StepId, NodeIndex, StateId, DataRef.
     Why: type safety. All are newtype wrappers over primitives.

EXPECTED TEST OUTCOME
  >= 28 tests covering: 7 structural validations, 2 repair strategies + escalation,
  3 novel step detection, 4 FSM generation, 2 harness generation, 3 resource estimation,
  3 end-to-end, 3 property-based, 10+ adversarial.

UNKNOWNS
  - Resource estimation accuracy cannot be validated until pipelines actually execute (Phase 4F+).
  - compile_predicate API from rules crate: exact signature needs verification during implementation.
  - AgentPhaseConfig.grc_budget format: assumed (u32, u32) tuple from AgentGrcConfig pattern.
