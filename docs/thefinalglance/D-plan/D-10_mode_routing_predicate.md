# D-10: Mode-routing predicate `select_grounding_mode`

Objective: implement the 5-mode orchestrator function that assigns every
novel claim to GroundingMode::{Constructive | FalsifiabilityCrossFamily |
OperationalFalsification | DiscoveryLabelOnly | Refuse}. The orchestrator
(not the LLM) decides; LLM has NO input to mode selection. This realizes
Cohen Boxed Myopic + Everitt current-RF (real-problem §3.2 lines 152-179):
"prevents the optimizer-decides-its-own-grade failure mode."
Estimated effort: 1-2 sessions.
Pre-requisites: D-9 (L3 frozen-surface enforcement; the routing function
  MUST be in L3 — agent cannot edit). spec-grounder verifies DONE.
Parallel-safe with: D-11 (checker registry; constructive mode dispatches into
  D-11's checkers), D-12 (operationalization predicate; consumes mode info).
  NOT parallel-safe with D-16 (UniCR refusal mode is the formal Mode E).

## Context

Real-problem §3.2 (lines 152-179) gives the exact predicate logic. The
five-mode taxonomy maps to grounding mechanisms that do NOT require pre-existing
comparable problems (real-problem §1):
- **Mode A — Constructive Proof**: claim is operationalizable + applicable
  checker exists → checker is ground truth.
- **Mode B — Falsifiability Cross-Family**: adversarial probe with different-
  family verifier; LAST RESORT, hard constraints.
- **Mode C — Discovery vs Search**: NOT a runtime mode but an output LABEL
  applied AFTER A or D produces a checker-grounded result (real-problem §1
  Mode C boundary).
- **Mode D — Operational Falsification**: sandbox experiment with typed
  ObservableEffect.
- **Mode E — Refuse**: no constructive checker, no experiment, intrinsic-
  signal confidence too low → "i don't know" first-class output.

The selection predicate:
```rust
fn select_grounding_mode(claim: &Claim, ctx: &OrchContext) -> GroundingMode {
    if claim.is_operationalizable() && ctx.checker_registry.applicable(claim) {
        return GroundingMode::Constructive;
    }
    if claim.permits_experiment() && ctx.sandbox.can_run(claim) {
        return GroundingMode::OperationalFalsification;
    }
    if claim.intrinsic_signals_below_threshold(ctx.unicr_tau) {
        return GroundingMode::Refuse;
    }
    if ctx.has_alternate_family_model() {
        return GroundingMode::FalsifiabilityCrossFamily;
    }
    return GroundingMode::Refuse;
}
```

Crucially: Mode B is FALLBACK after Mode A/D unavailability AND intrinsic
signal is high; otherwise refuse. Mode C is a LABEL applied AFTER mode A/D
produces a result.

Partial foundation present and usable:
- ELAI claim emission already happens at task_runner.rs (existing); D-10 wires
  predicate at the claim-emit hook.
- D-9 ensures the predicate function is in L3 (locked-symbol entry).
- D-11 (checker registry) provides ctx.checker_registry.applicable.
- D-16 (UniCR) provides ctx.unicr_tau threshold.

## Reads

- F:\.ELAI_workflow\docs\specs\guide\real-problem-solving-grounded-2026-05-19.md
  (§3.2 routing predicate; §1 5-mode taxonomy — lines 32-119 + 138-179)
- F:\.ELAI_workflow\workflow\papers\everitt-reward-tampering-1908.04734-interface.md
  (D-9 ingested)
- F:\.ELAI_workflow\workflow\papers\cohen-boxed-myopic-aimag43-3-interface.md
  (D-9 ingested)
- .elai_cc/crates/orchestrator/src/task_runner.rs (claim-emit sites)
- .elai_cc/crates/orchestrator/src/reasoning_template.rs (D-1; VerifierKind
  hints at applicable mode)

## Tasks

1. **Claim + ClaimType definition** in new file
   .elai_cc/crates/orchestrator/src/claim.rs:
   - `struct Claim { id: ClaimId, claim_type: ClaimType, statement:
     BoundedString<2048>, domain: Domain, source_template: Option<TemplateId>,
     intrinsic_signals: Option<IntrinsicSignals> }` (intrinsic_signals from D-3)
   - `enum ClaimType { Code, Math, Prose, Empirical, Definitional,
     Counterfactual, EthicalAesthetic }` (closed enum; L3-locked)
   - `fn is_operationalizable(&self) -> bool`: Code | Math | Empirical with
     test_cmd → true; Prose | Definitional → false; Counterfactual |
     EthicalAesthetic → false.
   - `fn permits_experiment(&self) -> bool`: Empirical with sandbox-compatible
     test → true; otherwise false.

2. **GroundingMode + predicate** in new file
   .elai_cc/crates/orchestrator/src/grounding_mode.rs:
   - `enum GroundingMode { Constructive, FalsifiabilityCrossFamily,
     OperationalFalsification, DiscoveryLabelOnly, Refuse }` (closed enum; L3-locked)
   - `fn select_grounding_mode(claim: &Claim, ctx: &OrchContext) -> GroundingMode`
     per the predicate above; reasoned per real-problem §3.2.
   - The function MUST NOT call the LLM; pure logic over typed inputs.

3. **OrchContext extension** in
   .elai_cc/crates/orchestrator/src/orch_context.rs:
   - Add fields: checker_registry (D-11 provides; stub trait here), sandbox
     (existing FeasibilityProber from C-22), unicr_tau (D-16 provides; stub
     constant here), has_alternate_family_model() bool (returns true if
     ELAI_PPM_ENABLE OR ELAI_CLOUD_MODE is set per D-2/B-37).
   - Stub impls keep the trait surface stable so D-11 + D-16 wire concrete
     implementations later without breaking D-10.

4. **dispatch_by_mode** in grounding_mode.rs:
   - `fn dispatch_by_mode(claim: &Claim, mode: GroundingMode, ctx: &mut
     OrchContext) -> ClaimResult`:
     - Constructive → call ctx.checker_registry.run(claim) (stub returns
       Inapplicable until D-11)
     - OperationalFalsification → call ctx.sandbox.run_observable_effect(claim)
       (stub returns Inconclusive until D-18 ObservableEffect)
     - FalsifiabilityCrossFamily → call ctx.adversary_role.probe(claim)
       (stub returns NoResult until D-13 Adversary)
     - Refuse → return ClaimResult::Refused with reason
     - DiscoveryLabelOnly → return ClaimResult::Inapplicable (label-only mode,
       not for runtime dispatch; raised here as defensive Err if hit)
   - `enum ClaimResult { Verified, Falsified { evidence: BoundedString<2048> },
     Inconclusive { reason: String }, Refused { reason: String } }`

5. **SSB action types** in ssb.rs (SCHEMA_VERSION 55→56):
   - `ModeSelected { claim_id: Uuid, mode: String, reason: String, audit_id:
     Uuid }`
   - `ClaimDispatched { claim_id: Uuid, mode: String, result: String,
     evidence_snippet: Option<String> }`
   - Round-trip tests.

6. **Integration in task_runner.rs**:
   - At every claim-emit site, after constructing Claim, call
     select_grounding_mode → emit ModeSelected SSB row → dispatch_by_mode →
     emit ClaimDispatched SSB row.
   - The orchestrator (not LLM) decides; this is the load-bearing constraint.

7. **L3 locking** (extends D-9 registry):
   - Add to locked_keys_registry.toml:
     ```toml
     [[locked_symbol]]
     symbol = "select_grounding_mode"
     file = ".elai_cc/crates/orchestrator/src/grounding_mode.rs"
     reason = "mode routing per Cohen Boxed Myopic / Everitt current-RF
       (arxiv:1908.04734); LLM cannot alter routing logic"

     [[locked_symbol]]
     symbol = "ClaimType"
     file = ".elai_cc/crates/orchestrator/src/claim.rs"
     reason = "closed enum per companion §4; agent cannot add new types"
     ```

8. **Unit tests** in grounding_mode.rs + claim.rs:
   - is_operationalizable table: Code with test_cmd → true; Math with
     sympy_expr → true; Empirical without sandbox-compat → false; etc.
   - select_grounding_mode 5 decision branches under mocked OrchContext.
   - Specifically: claim with constructive checker available → Constructive
     (even if Mode D possible).
   - Claim with no checker but sandbox-compat → OperationalFalsification.
   - Claim with no checker, no sandbox, signals below threshold → Refuse.
   - Claim with no checker, no sandbox, signals OK, alt-family present →
     FalsifiabilityCrossFamily.
   - Claim with no checker, no sandbox, signals OK, NO alt-family → Refuse.

9. **Contract test** crates/contracts/tests/mode_routing_contract.rs:
   - SCHEMA_VERSION_PUBLIC == 56.
   - ModeSelected + ClaimDispatched + payloads round-trip.
   - GroundingMode is closed-enum; serde stable.

10. **Decision doc** workflow/reasoning/D-10-decisions.md (3 decisions):
    - (a) Why orchestrator decides (not LLM): Cohen + Everitt + Hubinger
      converge — agent must not edit grading function. Selection function in
      L3.
    - (b) Why Mode E (Refuse) is the fallback (not Mode B): Self-Correction
      Bench (arxiv:2507.02778, T1) 64.5% blind-spot rate on same-model
      critique; without alt-family, "I don't know" beats "wrong but confident."
    - (c) Why DiscoveryLabelOnly is NOT a runtime mode: per real-problem §1
      Mode C — labeling is post-hoc annotation on A/D results.

## WIRING_CONTRACT

produces_symbols:
  - Claim + ClaimId + ClaimType + ClaimResult (closed enums)
  - GroundingMode (closed enum)
  - select_grounding_mode + dispatch_by_mode
  - OrchContext extension (checker_registry, sandbox, unicr_tau,
    has_alternate_family_model)
  - SsbActionType::ModeSelected + ::ClaimDispatched + payloads
  - SCHEMA_VERSION bump 55→56
consumes_symbols:
  - IntrinsicSignals (D-3)
  - SeededClock
  - LockedKeysAuditor (D-9; symbol-locked enforcement)
  - Domain enum (D-1 + D-7 extension)
  - Stub traits for checker_registry (D-11 fills in), sandbox (D-18),
    unicr_tau (D-16), adversary_role (D-13)
reachable_from:
  - task_runner.rs claim-emit sites
  - L3 locked-symbol registry (D-9)
stub_ban:
  - select_grounding_mode MUST NOT call LLM (defends Cohen Boxed Myopic)
  - Refuse-on-empty-context path MUST be reachable (defend against routing-
    everything-to-FalsifiabilityCrossFamily-without-checks)
  - All 5 mode branches MUST be tested (per-case)
  - dispatch_by_mode MUST emit ClaimDispatched SSB row regardless of mode
observable_effect:
  trigger: 50 synthetic claims across 5 modes (10 each); run select_grounding_mode
  expect_ssb: 50 ModeSelected rows; per-mode distribution matches ground truth
              ≥45/50 (≥90% F1 — D-21 benchmark validates this quantitatively)
  expect_artifact: no LLM call recorded in dispatch path; L3 locking prevents
                   in-code edit to select_grounding_mode without override

## DETERMINISTIC_CHECK

```bash
# 1. SCHEMA_VERSION 56
rg "SCHEMA_VERSION: i64 = 56" .elai_cc/crates/orchestrator/src/ssb.rs

# 2. SSB variants
rg "ModeSelected|ClaimDispatched" .elai_cc/crates/orchestrator/src/ssb.rs

# 3. GroundingMode + Claim types
rg "enum GroundingMode|enum ClaimType|fn select_grounding_mode" \
  .elai_cc/crates/orchestrator/src/grounding_mode.rs \
  .elai_cc/crates/orchestrator/src/claim.rs

# 4. L3 lock on select_grounding_mode
rg "select_grounding_mode" .elai_cc/scripts/locked_keys_registry.toml

# 5. No LLM call in routing predicate
rg "anthropic\|client.send\|sidecar\|complete" \
  .elai_cc/crates/orchestrator/src/grounding_mode.rs
# Expected: 0 matches

# 6. Build + tests
cargo check --workspace 2>&1 | tail -3
cargo test -p elai_orchestrator -- grounding_mode claim 2>&1 | grep "test result"
cargo test -p elai_cc_contracts -- mode_routing 2>&1 | grep "test result"
```

## ACCEPTANCE

- [ ] SCHEMA_VERSION_PUBLIC == 56
- [ ] ModeSelected + ClaimDispatched round-trip
- [ ] GroundingMode 5-variant closed enum
- [ ] select_grounding_mode 5-branch decision table validated
- [ ] is_operationalizable + permits_experiment tables correct
- [ ] dispatch_by_mode 5-branch dispatch (stubs for D-11/D-13/D-16/D-18)
- [ ] L3 lock registered for select_grounding_mode + ClaimType
- [ ] cargo check --workspace: zero new errors
- [ ] cargo test -p elai_orchestrator -- grounding_mode claim: all pass, ≥12
      new tests
- [ ] Cloud smoke test A: 10 synthetic claims of mixed types via cloud /task;
      ModeSelected rows assigned with correct distribution (verify against
      hand-labeled ground truth)
- [ ] No LLM call inside select_grounding_mode (verified via grep + execution
      trace)
- [ ] Decision doc workflow/reasoning/D-10-decisions.md

## Files touched

- ADD: .elai_cc/crates/orchestrator/src/claim.rs
- ADD: .elai_cc/crates/orchestrator/src/grounding_mode.rs
- MODIFY: .elai_cc/crates/orchestrator/src/orch_context.rs (or where
    OrchContext lives — confirm via grep first)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs (SCHEMA_VERSION 55→56;
    2 new variants + payloads)
- MODIFY: .elai_cc/crates/orchestrator/src/task_runner.rs (claim-emit hook
    dispatches mode routing)
- MODIFY: .elai_cc/scripts/locked_keys_registry.toml (D-9 extension)
- MODIFY: .elai_cc/crates/orchestrator/src/lib.rs (pub mod claim;
    pub mod grounding_mode)
- ADD: .elai_cc/crates/contracts/tests/mode_routing_contract.rs
- ADD: workflow/reasoning/D-10-decisions.md

## Out-of-scope

- D-11 checker registry (provides ctx.checker_registry.applicable)
- D-13 Adversary role (provides ctx.adversary_role.probe)
- D-16 UniCR (provides ctx.unicr_tau)
- D-18 ObservableEffect (provides ctx.sandbox typed effect)
- D-21 mode-routing coverage benchmark (quantifies F1 score)

## Next

D-11 (checker registry) is the next consumer; it backs ctx.checker_registry.
D-21 (coverage benchmark) gates D-10 at ≥0.85 F1.
