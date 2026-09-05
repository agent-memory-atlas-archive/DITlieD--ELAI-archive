# C-37: PER_ROLE_DECODING_DEFAULTS Re-evaluation (Critic/Auditor Constrained-Decoding Risk)

Objective: re-test C-26's PER_ROLE_DECODING_DEFAULTS against the counter-evidence
in arXiv:2604.06066 (Structure Snowballing). If the paper reproduces on
Qwen2.5-Coder-7B, flip critic/auditor default from JsonSchema to FreeText (or
relax to a lighter constraint shape) BEFORE C-54 SWE-bench publication so the
published number reflects the corrected decoding mode.

Pre-requisites: C-26 (constrained-decoding retry strategy) SHIPPED.
Parallel-safe with: C-35, C-36, C-38, C-39, C-40 (orthogonal). Sequence ahead of
C-54 (SWE-bench Verified flagship benchmark) so the published number uses the
corrected default.

Paper grounding (arXiv:2604.06066, 2026-04-07, verified abs page 2026-05-17,
prior oracle 2026-05-17):
"From Hallucination to Structure Snowballing: The Alignment Tax of Constrained
Decoding in LLM Reflection."
Measured claim: constrained decoding "does not improve self-correction performance"
on 8B model class; causes "structure snowballing" where the model achieves
"near-perfect superficial syntactic alignment yet fails to detect or resolve
deeper semantic errors." Code released at github.com/hongxuzhou/agentic_llm_structured_self_critique.
Model size class (8B) matches ELAI's 7B sidecar tier directly.

## Context

ELAI's C-26 shipped PER_ROLE_DECODING_DEFAULTS in elai_cc.toml with
critic = JsonSchema and auditor = JsonSchema as defaults. The motivation
was reliable structured output for downstream parsing. The Structure Snowballing
paper is direct counter-evidence specifically for the REFLECTION role
(self-critique, audit), arguing that schema constraints during reflection
cause the model to surface-fit to JSON instead of detecting semantic problems.

ELAI's existing anti-rubber-stamp FINDING requirement (auditor must produce
non-trivial findings) MAY already mitigate the snowballing effect — but this
is hypothesis, not measurement. C-37 is the experiment that resolves it.

This is a TRIVIAL config flip if the experiment confirms regression — but the
experiment itself MUST run before flipping defaults, because the paper's 8B
class is not Qwen2.5-Coder-7B specifically, and ELAI's auditor prompt
architecture is not the paper's exact reflector prompt. Don't act on faith.

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-26_constrained_decoding_retry_strategy.md
- .elai_cc/crates/runtime/src/decoding.rs (current PER_ROLE_DECODING_DEFAULTS)
- .elai_cc/crates/agent/src/auditor.rs + agent/src/critic.rs
- .elai_cc/elai_cc.toml ([decoding.per_role] section)
- github.com/hongxuzhou/agentic_llm_structured_self_critique (paper's reference
  implementation — clone for harness replication)

## Tasks

1. **Source-grounding (4F.X)** — spawn paper-ingestor (mode=paper) against
   arXiv:2604.06066. Produces workflow/papers/structure-snowballing-interface.md
   capturing the exact contract-test shape the paper used. Also spawn
   paper-ingestor (mode=source) against the paper's GitHub repo to extract
   their JsonSchema-vs-FreeText comparison harness.

2. **Harness replication (4F.X.1)** — port the paper's contract test to
   .elai_cc/crates/testing-trust/tests/structure_snowballing.rs. Inputs: a
   set of code snippets with intentional semantic bugs (off-by-one, null deref,
   wrong-operator). Two runs: (a) auditor in JsonSchema decoding mode,
   (b) auditor in FreeText decoding mode. Output: per-mode detection rate.

3. **Pilot on Qwen2.5-Coder-7B (4F.X.2, DEFERRED to RunPod)** — flag this as
   GPU-dependent. Once runpod available, run the harness against
   Qwen2.5-Coder-7B NF4+FA2 sidecar. Record detection-rate delta.

4. **Decision matrix (4F.X.3)** — IF FreeText > JsonSchema by ≥10pp:
   flip auditor default to FreeText, keep critic at JsonSchema (different
   role). IF FreeText > JsonSchema by <10pp but >0: introduce a "lightly
   constrained" mode (grammar that REQUIRES non-empty `findings: [...]` array
   but does NOT enforce per-finding internal structure) and flip auditor to
   this third mode. IF FreeText < JsonSchema: keep current defaults, document
   ELAI's anti-rubber-stamp FINDING as the mitigation, close C-37.

5. **Migration path (4F.X.4)** — if defaults flip, write a one-time migration
   script that rewrites existing project elai_cc.toml `[decoding.per_role]`
   blocks. Old override values respected; only the DEFAULT changes for users
   who never customized.

6. **SSB action types** — add 2 new strings: decoding_mode_pilot_run (per-pilot)
   and decoding_default_flipped (if flip happens). Both carry role, old_mode,
   new_mode, detection_rate_delta. SCHEMA_VERSION bump only if defaults flip.

7. **Tests first (TDD, 4F.X.5..4F.X.7)**:
   - 4F.X.5: harness runs in both modes against synthetic bug fixture set
     and produces a detection-rate number
   - 4F.X.6: migration script is idempotent (running it twice produces same
     output as running once)
   - 4F.X.7: anti-rubber-stamp FINDING requirement still triggers under
     FreeText mode (regression test — auditor still produces findings, doesn't
     silently emit empty audit)

8. **Documentation update** — DETAILED_SUMMARY_01 §X (C-26 description) gets
   a note: "C-37 ratified/flipped default on YYYY-MM-DD per
   workflow/sessions/{id}/structure-snowballing-pilot.md."

9. **Contract tests** — tests/contracts/decoding_role_contract.rs verifying
   that PER_ROLE_DECODING_DEFAULTS exposes the auditor mode via SSB and that
   the mode is observable from outside the runtime crate (auditor's decoding
   mode is part of its public wiring contract).

## WIRING_CONTRACT

produces_symbols:
  - structure_snowballing_harness module (testing-trust/tests/)
  - decoding-defaults migration script (.elai_cc/scripts/migrate_decoding_defaults.sh)
  - PossibleNew DecodingMode variant: LightlyConstrained
  - SSB action_types: decoding_mode_pilot_run, decoding_default_flipped
  - workflow/sessions/{id}/structure-snowballing-pilot.md (artifact)
consumes_symbols:
  - PER_ROLE_DECODING_DEFAULTS (C-26)
  - Auditor, Critic agents
  - Qwen2.5-Coder sidecar runtime
  - SSB event bus
reachable_from:
  - elai_cc.toml [decoding.per_role] (existing; possibly modified default)
  - migration script
stub_ban:
  - harness MUST run BOTH modes against the SAME fixture in the SAME session
    (no cross-session aggregation that could mask determinism issues)
  - decision matrix MUST be applied DETERMINISTICALLY (no human "looks good"
    override without an SSB row recording the override reason)
  - anti-rubber-stamp FINDING test MUST still pass after any mode change
frontend_pair: (none — runtime-layer; pilot dashboard panel optional but
              not blocking)
observable_effect:
  trigger: cargo test -p elai_testing_trust -- structure_snowballing
          (post-RunPod, with local sidecar live)
  expect_ssb: decoding_mode_pilot_run rows for each of N synthetic bugs in
              each of 2 modes
  expect_artifact: workflow/sessions/{id}/structure-snowballing-pilot.md
                   with side-by-side per-bug detection table + summary delta

## DETERMINISTIC_CHECK

```bash
# 1. Harness exists
test -f .elai_cc/crates/testing-trust/tests/structure_snowballing.rs

# 2. Migration script exists and is idempotent
test -x .elai_cc/scripts/migrate_decoding_defaults.sh
bash .elai_cc/scripts/migrate_decoding_defaults.sh --dry-run 2>&1 | grep "idempotent"

# 3. SSB action types
rg "decoding_mode_pilot_run|decoding_default_flipped" .elai_cc/crates/orchestrator/src

# 4. Pilot artifact exists (post-RunPod)
test -f workflow/sessions/$(cat workflow/sessions/current.txt)/structure-snowballing-pilot.md

# 5. Anti-rubber-stamp regression still passes under any mode
cargo test -p elai_agent -- auditor_no_rubber_stamp 2>&1 | grep "test result.*ok"

# 6. Contract test passes
cargo test -p elai_runtime -- decoding_role_contract 2>&1 | grep "test result.*ok"
```

## ACCEPTANCE

- [ ] paper-ingestor produces workflow/papers/structure-snowballing-interface.md
- [ ] paper-ingestor (mode=source) produces interface map for
      hongxuzhou/agentic_llm_structured_self_critique
- [ ] Harness replicated and runs in both JsonSchema and FreeText modes
- [ ] Pilot run on Qwen2.5-Coder-7B completed (DEPENDS ON RunPod — defer
      to C-54-adjacent benchmark window)
- [ ] Decision matrix applied; outcome recorded in pilot.md
- [ ] If flip: migration script written, tested for idempotency, executed
      in test fixtures
- [ ] If flip: DETAILED_SUMMARY_01 §X annotated with date + outcome
- [ ] Anti-rubber-stamp FINDING regression test passes (no silent
      empty-audit emission under any mode)
- [ ] SSB action types emitted with non-null role + delta fields
- [ ] Contract tests pass
- [ ] **NOTE**: this patch's outcome is conditional (flip OR keep). Either
      outcome closes C-37; closure is the act of producing the measured
      pilot.md, NOT the act of flipping a default.

## Files

- ADD: .elai_cc/crates/testing-trust/tests/structure_snowballing.rs
- ADD: .elai_cc/crates/testing-trust/tests/contracts/decoding_role_contract.rs
- ADD: .elai_cc/scripts/migrate_decoding_defaults.sh
- ADD: workflow/papers/structure-snowballing-interface.md
- MODIFY (CONDITIONAL on flip): .elai_cc/elai_cc.toml ([decoding.per_role] auditor default)
- MODIFY (CONDITIONAL on flip): .elai_cc/crates/runtime/src/decoding.rs (PER_ROLE_DECODING_DEFAULTS)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb/schema.rs (2 new action types)
- MODIFY: F:\.ELAI_workflow\docs\thefinalglance\summary\DETAILED_SUMMARY_01_overview_and_architecture.md (C-26 footnote)
- ADD: workflow/sessions/{id}/structure-snowballing-pilot.md (pilot artifact)

## Affected Verification Reports

- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\C-plan\C-26_constrained_decoding_retry_strategy.report.md
  — annotate C-37 ratification or flip on closure.

## Next

After C-37 closes (regardless of flip outcome): the pilot.md is a public
measurement that feeds into C-54 SWE-bench Verified publication as
methodological grounding. Pilot.md is also the canonical reference for any
future user who asks "why is ELAI's auditor decoding mode X" — the answer is
"measured, see pilot.md" not "designer hunch."
