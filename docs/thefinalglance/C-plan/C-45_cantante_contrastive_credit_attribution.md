# C-45: CANTANTE Contrastive Credit Attribution for Agent Role Ablation

Objective: integrate CANTANTE's contrastive credit-attribution method to measure
which AgentRole contributes to each successful task and which contribute noise.
Produces a per-role contribution score that feeds C-41 SkillRegistry routing
and (over time) candidate-evidence for role consolidation.

Pre-requisites: 8 AgentRoles shipped (Executor, Auditor, Researcher, Designer,
PromptEngineer, ContextCompiler, MemoryWriter, MediaValidator); C-41 SkillRegistry
recommended (but not strict gate).
Parallel-safe with: C-35..C-44, C-46..C-53.

Paper grounding (arXiv:2605.13295, 2026-05-15, verified abs page 2026-05-17):
"CANTANTE: Optimizing Agentic Systems via Contrastive Credit Attribution."
Per verifier: paper proposes contrastive removal of each agent in turn,
measures task-success delta, attributes credit accordingly. Method generalizes
from the paper's specific multi-agent system to any role-based agent suite.

## Context

ELAI's 8 AgentRoles were designed orthogonally but not measured against role
consolidation. The maintainer's honest assessment notes "8 vs minimal agent
sets (e.g. AgentForge's 5, mini-SWE-agent's 1) is unmeasured." CANTANTE gives
a measurement primitive:

For each (role R, task T), run T with R AVAILABLE and run T with R FORCIBLY
DROPPED (FSM routes around R, falls back to default-merged behavior). Compare
success deltas across N tasks. Aggregate per role.

This produces 8 numbers. Each tells you what dropping that role costs.
Roles with near-zero cost are candidates for consolidation. Roles with
high cost are core. The number is HONEST — not "we believe ContextCompiler
is useful," but "dropping ContextCompiler costs X% on the SWE-bench
subset."

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-41_skills_driven_workflow_formalization.md
- .elai_cc/crates/agent/src/roles/ (per-role implementations)
- workflow/papers/cantante-credit-attribution.md (paper-ingestor, Task 1)

## Tasks

1. **Source-grounding** — paper-ingestor (mode=paper) against arXiv:2605.13295.
   Produces workflow/papers/cantante-credit-attribution.md.

2. **AgentRoleDropper (crates/agent/src/dropper.rs)** — RuntimeContext flag
   `dropped_role: Option<AgentRole>`. FSM dispatch checks the flag; when
   set, calls to the dropped role return a "fallback to merged-behavior"
   path (e.g. Researcher dropped → use cached prior research only; never
   spawn researcher). Documented per-role fallback behavior table.

3. **Credit harness (tests/credit_attribution_harness.rs)** — takes a task
   set + 9 config variants (baseline + 8 each-role-dropped), runs each,
   aggregates success/audit-pass rate. Produces credit-attribution.md
   artifact.

4. **Pilot run (4D.X, DEFERRED to RunPod)** — flag GPU-dependent. Pilot
   uses a 50-task subset of SWE-bench Verified. Total 9 × 50 = 450 runs.
   Cost: ~$50-200 depending on hybrid/local mix. Document in
   workflow/sessions/{id}/cantante-pilot.md.

5. **SkillRegistry routing feedback (C-41 integration)** — per-role credit
   scores expose as a metadata field SkillRegistry can consult when
   building routing weights. Low-credit roles get deprioritized
   automatically (config-gated; default off until validated).

6. **SSB action types** — agent_role_dropped, credit_attribution_run,
   credit_attribution_result. Last carries per-role delta map.

7. **Tests first (TDD)**:
   - 4D.X.3: harness runs deterministically against synthetic task set
     (no GPU required for harness itself)
   - 4D.X.4: AgentRoleDropper fallback path produces NON-empty output
     for each role (no silent skip — fallback must be defined and observable)
   - 4D.X.5: credit-attribution math: delta = baseline_success_rate -
     dropped_success_rate; per-task and aggregate
   - 4D.X.6: dropping a role MUST emit agent_role_dropped SSB BEFORE
     dispatch reroutes (audit trail)

8. **Documentation** — once pilot lands, DETAILED_SUMMARY_01 §X (AgentRoles)
   gets the credit table as a footnote.

## WIRING_CONTRACT

produces_symbols:
  - AgentRoleDropper, RoleFallback (crates/agent/src/dropper.rs)
  - credit_attribution_harness (tests/)
  - SSB action_types: agent_role_dropped, credit_attribution_run,
    credit_attribution_result
  - workflow/sessions/{id}/cantante-pilot.md (artifact)
consumes_symbols:
  - 8 AgentRole implementations
  - FSM dispatch
  - SSB event bus
  - SkillRegistry (C-41) for routing feedback
reachable_from:
  - credit harness invocation
  - elai_cc.toml [agent.credit_attribution] config
stub_ban:
  - dropped-role fallback path MUST be DEFINED per role; "panic on drop"
    is not acceptable
  - credit_attribution_result SSB MUST emit per-role delta with confidence
    interval (over N runs)
frontend_pair: (none — measurement layer; surfacing via existing AgentsPanel)
observable_effect:
  trigger: cargo test -p elai_agent --test credit_attribution_harness
  expect_ssb: agent_role_dropped per drop config; credit_attribution_run
              per task; credit_attribution_result once at aggregation

## DETERMINISTIC_CHECK

```bash
test -f .elai_cc/crates/agent/src/dropper.rs
test -f .elai_cc/crates/agent/tests/credit_attribution_harness.rs
rg "agent_role_dropped|credit_attribution_run|credit_attribution_result" .elai_cc/crates/orchestrator/src
cargo test -p elai_agent -- credit_attribution 2>&1 | grep "test result.*ok"
test -f workflow/sessions/$(cat workflow/sessions/current.txt)/cantante-pilot.md
```

## ACCEPTANCE

- [ ] paper-ingestor produces workflow/papers/cantante-credit-attribution.md
- [ ] AgentRoleDropper implemented with explicit per-role fallback table
- [ ] Credit harness runs deterministically against synthetic fixture
- [ ] Pilot on 50-task SWE-bench subset (DEFERRED to RunPod window)
- [ ] cantante-pilot.md artifact with per-role credit table + CI
- [ ] DETAILED_SUMMARY §X annotated with credit table footnote
- [ ] All 3 SSB action types emit with non-null deltas
- [ ] TDD tests pass
- [ ] Contract tests pass

## Files

- ADD: .elai_cc/crates/agent/src/dropper.rs
- ADD: .elai_cc/crates/agent/tests/credit_attribution_harness.rs
- ADD: workflow/papers/cantante-credit-attribution.md
- ADD: workflow/sessions/{id}/cantante-pilot.md
- MODIFY: .elai_cc/crates/agent/src/fsm_dispatch.rs (check dropped_role flag)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb/schema.rs (3 new action types)
- MODIFY: .elai_cc/elai_cc.toml ([agent.credit_attribution] section)
- MODIFY (post-pilot): F:\.ELAI_workflow\docs\thefinalglance\summary\DETAILED_SUMMARY_01_overview_and_architecture.md (§AgentRoles footnote)

## Next

C-45 closure produces an HONEST per-role credit table. The table is the
input to a future decision: "consolidate roles X+Y" requires the table
showing both have low independent credit. Without C-45, role consolidation
is hunch.
