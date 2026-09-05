I-32 structural role scoping -- executor decision doc

WHAT
Strip narrative persona ("You are an interactive agent...") from factual/code roles
(Executor, Researcher, ContextCompiler, Designer, MemoryWriter). Replace with schema-first
role contracts in new crates/orchestrator/src/role_contracts.rs. Retain alignment-dependent
role framing (Auditor, IdeaCritic, InjectionClassifier variants) unchanged. Evidence-gated
per ID-5 via an A/B harness before any production flip.

KEY DECISIONS

D-1: injection site is report_fns.rs::build_system_prompt_for_role (live line ~1121),
NOT agent_dispatch.rs (that file does not exist; context-package VALIDATION WARNING
confirmed by exploration report). apply_to_system_prompt() called AFTER load_system_prompt()
and BEFORE assemble_user_memory_section push, so user-memory section is assembled on the
POST-refactor sections slice.

D-2: supervisor gate extension (I-32's scope sub-goal) -- the plan says role-scope tool
enforcement. Exploration verdict: do NOT add a catalog-filter at build_tool_catalog
(production_agent_rpc.rs:649). One enforcement plane: extend InProcessSupervisor::run_gates
via a new run_role_scope_gate() helper that checks the Directive::initiator field (snake_case
role name) against the RoleContractStore. If a role has a disallowed tool and the directive
tool_name matches, emit a Block finding. Composable: I-35 adds a 4th gate arm later; I-32
adds a 3rd. Grounds: wiring-enforcement.md R8 (gate frozen + single authority).

D-3: PersonaAnchor production construction site -- zero production call sites exist today
(exploration confirms). No ordering hazard to fix. The contract text from
apply_to_system_prompt is available if/when PersonaAnchor is constructed; the order is
correct by construction. Tier-2 deferral: no code change needed here.

D-4: A/B fixture -- E-19 fixtures are ReasoningQualityProbe-specific (ProbeKind arms, not
role-prompt A/B). A minimal 3-task deterministic fixture per affected role is created at
crates/contracts/tests/fixtures/role_contracts/{executor,researcher,context_compiler}_ab.json.
Pass-rate is computed as fraction of tasks whose expected_output appears in the assembled
system prompt section (deterministic string check, seeded). Evidence gate: new_rate >= old_rate
required for production flip flag (else stub_ban violation). Fixtures seed-safe under ELAI_SEED.

D-5: SSB version -- LIVE at edit time: SCHEMA_VERSION=240 (ssb.rs:3143),
TOTAL_VARIANTS=549 (action_type_lock.rs:702). I-32 target: 241/550.
Exploration report correctly flagged the context-package discrepancy (238/547 was stale).

D-6 (W-6): no new dashboard surface. AbDelta rows are flat beacons; existing SsbViewer
generic recent_actions stream renders them. Justified.

EXPECTED TEST OUTCOME
- i32_role_scoping_contract: 3 arms pass
- i1 7/7 pass (gate extended, existing arms unaffected)
- action_type_lock 2/2 pass (241/550)
- e45 TOTAL_VARIANTS=550 assertion pass
- cargo check --workspace clean

UNKNOWNS / OPEN
- I-35 4th gate arm: composable by design (run_role_scope_gate is a separate helper, not
  inlined into the existing 3-arm chain body).
- A/B fixture live model scoring (RUNPOD-PENDING per plan): the deterministic string-match
  proxy suffices for the evidence gate under ID-5.
