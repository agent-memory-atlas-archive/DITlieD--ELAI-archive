# C-52: Dafny Formal Verification Spike (RESEARCH-GRADE)

Objective: spike a Dafny-as-secondary-verification pass for safety-critical
modules. Optional, opt-in, narrow scope. Direct application of
arXiv:2604.22601 NL2VC-60: Gemma 4-31B achieved 90.91% verification success
on a 60-task formal-verification benchmark.

Pre-requisites: Phase 4F code completeness SHIPPED; C-50 (Contract-Coding)
preferred but not strict gate. Dafny toolchain availability on dev host.
Parallel-safe with: C-35..C-51, C-53. RESEARCH-GRADE — closure includes
explicit "infeasible to land in production" as a valid outcome.

Paper grounding (arXiv:2604.22601, 2026-04-24, verified via prior oracle):
"Gemma 4-31B achieved 90.91% verification success rate" on NL2VC-60. Dafny
is the verification target. RESEARCH-GRADE because integrating Dafny
toolchain into Rust agentic workflow is non-trivial.

## Context

ELAI's testing-trust layer (C-27/C-28/C-29) provides mutation + metamorphic
+ wiring-contract evidence. Formal verification via Dafny is a layer
ABOVE that, providing mathematical proofs (where applicable) of pre/post
conditions.

NOT every ELAI module is a candidate for Dafny verification. Practical
fit:
- Memory layer trust enforcement (C-48) — small functions with clear
  pre/post conditions
- FSM transition logic (Phase 3) — finite state space, ideal for Dafny
- Some testing-trust math (mutation-score aggregation, MR-violation
  counts)

Most of ELAI (file I/O, network, LLM dispatch) is OUT of scope for Dafny.
This patch SPIKES the integration on 3-5 narrow modules; success criteria
is "Dafny can express it AND verifies AND the toolchain integration is
reproducible in CI."

If the spike concludes "Dafny adds high integration cost for low coverage
ratio," closure = documented infeasibility note. Either outcome counts.

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-48_safeharbor_memory_guardrail.md
- .elai_cc/crates/orchestrator/src/fsm.rs (FSM transitions — Dafny candidate)
- .elai_cc/crates/testing-trust/src/aggregation.rs (mutation-score math — candidate)
- workflow/papers/nl2vc-dafny-interface.md (paper-ingestor, Task 1)
- dafny.org install + getting-started docs

## Tasks

1. **Source-grounding** — paper-ingestor (mode=paper) against arXiv:2604.22601.

2. **Toolchain installability check** — verify Dafny installs cleanly on
   Windows + Linux dev hosts. If install is non-trivial, document blockers
   and proceed to step 7 (infeasibility note) without further work.

3. **Module candidate selection** — pick 3 narrow modules:
   - FSM transition validator
   - MemoryView trust-tier filter (from C-48)
   - mutation-score aggregator (from C-28)

4. **Dafny shim translations** — for each candidate, write a Dafny file
   that mirrors the Rust function's pre/post conditions. NOT a 1:1 source
   port; a SPECIFICATION in Dafny that the Rust impl claims to satisfy.

5. **Verification harness (crates/formal_verify/, feature-gated)** —
   `cargo verify-dafny --module FSMTransition` shells out to `dafny verify`
   on the corresponding .dfy. Pass = clean output; Fail = surface error
   to SSB.

6. **CI integration** — optional CI job (informational, not blocking).
   Runs verification on each PR touching the 3 spike modules.

7. **Infeasibility note path** — if any of (a) toolchain installs, (b) 3
   spike translations succeed, (c) CI integration works fail to land,
   write workflow/sessions/{id}/dafny-spike-infeasibility.md with concrete
   blocker descriptions. Closure is the note, not landed verification.

8. **SSB action types** — dafny_verify_run, dafny_verify_pass,
   dafny_verify_fail.

9. **Tests first (TDD, partial)**:
   - 4G.X.3: verification harness invokes dafny binary; non-zero exit
     surfaces as dafny_verify_fail SSB
   - 4G.X.4: 3 spike module Dafny files verify cleanly on a known-good
     baseline

10. **Documentation** — outcome (spike success OR infeasibility) gets a
    section in .elai_cc/docs/formal-verification.md.

## WIRING_CONTRACT

produces_symbols (conditional on spike success):
  - .elai_cc/formal/fsm_transition.dfy
  - .elai_cc/formal/memory_view.dfy
  - .elai_cc/formal/mutation_aggregator.dfy
  - VerificationHarness (crates/formal_verify/)
  - SSB action_types: dafny_verify_run, dafny_verify_pass, dafny_verify_fail
  - optional CI job
produces_symbols (conditional on infeasibility):
  - workflow/sessions/{id}/dafny-spike-infeasibility.md
  - .elai_cc/docs/formal-verification.md (with infeasibility section)
consumes_symbols:
  - SSB event bus
  - Dafny toolchain (external)
reachable_from:
  - cargo verify-dafny [--module X]
  - optional CI job
stub_ban:
  - infeasibility note MUST cite at least one CONCRETE blocker (install
    failure, language-feature gap, CI-runner incompatibility), not vague
    "too hard"
  - if 1 of 3 spikes lands but 2 fail, partial-success note required;
    don't claim full success

## DETERMINISTIC_CHECK (conditional)

```bash
# Success path
which dafny
test -f .elai_cc/formal/fsm_transition.dfy
cargo verify-dafny --module FSMTransition 2>&1 | grep "Dafny program verifier"

# OR infeasibility path
test -f workflow/sessions/$(cat workflow/sessions/current.txt)/dafny-spike-infeasibility.md
grep -c "blocker:" workflow/sessions/$(cat workflow/sessions/current.txt)/dafny-spike-infeasibility.md | awk '$1 >= 1 {exit 0} {exit 1}'
```

## ACCEPTANCE

(success path)
- [ ] paper-ingestor produces workflow/papers/nl2vc-dafny-interface.md
- [ ] Dafny toolchain documented as installable on at least one supported host
- [ ] 3 spike modules: FSMTransition, MemoryView, MutationAggregator have
      Dafny specifications
- [ ] All 3 verify cleanly
- [ ] VerificationHarness shells out reproducibly
- [ ] All 3 SSB action types emit
- [ ] TDD tests pass
- [ ] CI job runs (informational, not blocking)

OR (infeasibility path)
- [ ] dafny-spike-infeasibility.md authored with at least 1 concrete blocker
- [ ] formal-verification.md documents infeasibility outcome with revisit
      trigger ("revisit when Dafny supports X" or similar)
- [ ] C-52 closes as DEFERRED with explicit re-evaluation condition

## Files (conditional)

(success path adds)
- ADD: .elai_cc/crates/formal_verify/Cargo.toml + src/lib.rs + src/harness.rs
- ADD: .elai_cc/formal/fsm_transition.dfy + memory_view.dfy + mutation_aggregator.dfy
- ADD: .elai_cc/crates/formal_verify/tests/verification_harness.rs
- ADD: .github/workflows/dafny-verify.yml
- ADD: workflow/papers/nl2vc-dafny-interface.md
- MODIFY: .elai_cc/Cargo.toml (formal_verify workspace member)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb/schema.rs (3 new action types)

(infeasibility path adds)
- ADD: workflow/sessions/{id}/dafny-spike-infeasibility.md
- ADD: .elai_cc/docs/formal-verification.md (with infeasibility section)

## Next

C-52 closure (success OR infeasibility) clears the question of "should ELAI
do formal verification." Either it ships on 3 narrow modules with future
expansion candidates, or the gap is documented and the testing-trust layer
remains the highest evidence tier ELAI ships. Both are valid outcomes per
the spike protocol.
