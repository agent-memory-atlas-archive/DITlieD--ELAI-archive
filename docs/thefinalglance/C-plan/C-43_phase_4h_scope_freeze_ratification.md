# C-43: Phase 4H Self-Modification Scope Freeze (AEL Anti-Evidence Ratification)

Objective: ratify that Phase 4H stays at its current 3-layer scope (auditor-approval
gates + MSI auto-revert + Cosmos artifact tier) and explicitly defer any additional
self-modification mechanisms (continuous self-rewarding, evolutionary search over
prompts, online RL fine-tuning) until measurement justifies expansion. Direct
application of arXiv:2604.21725 (AEL) ablation finding: adding mechanisms beyond
memory+reflection consistently DEGRADES performance in open-ended environments.

Pre-requisites: Phase 4H SHIPPED at current scope (3 layers + MSI + Cosmos).
Parallel-safe with: all other C-N. This is a policy/documentation patch with
defensive code-level guardrails; no behavioral runtime change.

Paper grounding (arXiv:2604.21725, 2026-04-25, verified abs page 2026-05-17):
"AEL: Agent Evolving Learning for Open-Ended Environments." Critical ablation
finding per verifier 2026-05-17: "adding mechanisms beyond memory+reflection
consistently degrades performance." Direct counter-evidence to any Phase 4H
expansion that would add layers 4, 5, 6+ (e.g. evolutionary search, online RL,
continuous self-rewarding loops). Read this paper BEFORE expanding 4H scope —
maintainer's own honest assessment file §X also concedes 4H is at "experimental
status with HIGH-confidence value-add only in narrow MSI auto-revert path."

## Context

Phase 4H currently ships:
1. **Layer 1**: pattern-based self-modification of agent prompts based on
   accumulated mistakes.md entries.
2. **Layer 2**: auditor-approval gate (every self-mod proposal goes through
   the auditor before landing).
3. **Layer 3**: MSI (Modification Safety Index) auto-revert when a recent
   self-mod correlates with measured regression in test/audit metrics.
Plus the Cosmos artifact tier for non-code self-modifications (skills,
prompts, system rules).

The maintainer's honest assessment surfaces ongoing uncertainty: "4H produces
value in the MSI auto-revert path (measured: prevents N regressions per session
on average); value-add of layers 1-2 in isolation is unmeasured."

Future temptation: add layer 4 (continuous self-rewarding loops a la DGM),
layer 5 (online RL fine-tuning of the sidecar model), layer 6 (evolutionary
search over prompt corpora). The AEL paper is direct evidence that these
additions DEGRADE measured performance in open-ended environments.

C-43 is a ratification patch with three components:
1. Document the scope-freeze decision with citation to AEL paper.
2. Add a code-level guardrail: any new self-mod layer requires explicit
   gating behind a `cfg(feature = "self_mod_experimental")` AND a recorded
   ablation comparison against the current 3-layer baseline.
3. Ablation-comparison harness so future expansion candidates are MEASURED
   before merging, not adopted on architectural hunch.

## Reads

- F:\.ELAI_workflow\docs\specs\guide\steps\phase-4H.txt
- F:\.ELAI_workflow\workflow\reasoning\elai_capability_honest_assessment.md (§X 4H confidence labels)
- .elai_cc/crates/self_mod/ (current 3-layer + MSI + Cosmos implementation)
- workflow/papers/ael-ablation-interface.md (paper-ingestor output, Task 1)

## Tasks

1. **Source-grounding** — paper-ingestor (mode=paper) against arXiv:2604.21725.
   Produces workflow/papers/ael-ablation-interface.md capturing the per-mechanism
   ablation table verbatim.

2. **Scope-freeze policy document** — add
   .elai_cc/docs/phase-4h-scope-policy.md citing:
   - AEL paper finding (verbatim quote from ablation table)
   - ELAI's current 3-layer scope description
   - Decision criteria: ANY proposed 4th layer must (a) cite a measured
     improvement on at least 2 of ELAI's published benchmarks (post-C-54),
     (b) survive a SIDE-BY-SIDE ablation showing strict ≥+3pp improvement
     vs baseline on the same fixture, (c) get explicit user approval per
     project (config opt-in, not default).

3. **Code-level guardrail** — anti-expansion attribute macro:
   ```
   #[allow(self_mod_experimental, justify = "...")]
   ```
   required on any NEW self-mod code path. The macro emits a compile-time
   warning unless `self_mod_experimental` feature is enabled. New paths
   without the attribute fail clippy (treated as error in CI).

4. **Ablation harness (crates/self_mod/tests/ablation_harness.rs)** —
   takes two AbstractAgent configurations (baseline + candidate), runs N
   tasks against both, reports per-task delta + aggregate. Designed to make
   the decision-criteria (b) measurement turn-key: future expansion candidate
   = author harness fixture, run, append result to scope-policy.md.

5. **SSB action types** — self_mod_layer_proposed (when someone tries to
   register a new layer without proper gating, runtime emits warning),
   self_mod_ablation_run (when harness fires).

6. **Tests first (TDD)**:
   - 4H.X.3: ablation harness runs deterministically against a synthetic
     2-config fixture and produces matched-pair delta
   - 4H.X.4: scope-freeze guardrail produces compile-time warning on a
     new self-mod path that lacks #[allow(self_mod_experimental, ...)]
   - 4H.X.5: existing 3 layers (memory+reflection+MSI) do NOT trigger the
     guardrail (grandfathered via baseline allowlist)
   - 4H.X.6: synthetic "DGM-like" candidate harness produces a negative
     delta on at least one synthetic adversarial fixture, validating that
     the harness can SURFACE degradation (not just rubber-stamp candidates)

7. **Mistakes.md entry** — append: "Self-mod scope expansion is the failure
   mode this entry guards. Without measurement, hypothesis becomes belief.
   See C-43 + AEL paper for the ratification."

8. **Documentation** — update DETAILED_SUMMARY_01 §X (Phase 4H description)
   with a footnote: "Scope frozen per C-43 ratification on YYYY-MM-DD;
   expansion gated on ablation harness + 2-benchmark improvement criteria."

## WIRING_CONTRACT

produces_symbols:
  - .elai_cc/docs/phase-4h-scope-policy.md (policy artifact)
  - #[allow(self_mod_experimental, ...)] attribute macro (crates/self_mod/macros/)
  - ablation_harness module (crates/self_mod/tests/ablation_harness.rs)
  - SSB action_types: self_mod_layer_proposed, self_mod_ablation_run
consumes_symbols:
  - existing self_mod crate (no behavioral change)
  - SSB event bus
  - clippy lint pipeline
reachable_from:
  - any contributor authoring new self-mod code (compile-time gate)
  - ablation harness invocation (manual or CI)
stub_ban:
  - guardrail macro MUST emit warning, NOT silent allow
  - ablation harness MUST emit self_mod_ablation_run SSB before returning
    result
  - scope-policy.md MUST cite AEL paper verbatim (no paraphrase that
    weakens the finding)
frontend_pair: (none — policy + dev-time guardrail)
observable_effect:
  trigger: PR adds a new self-mod path without proper attribute
  expect: clippy warning surfaces in CI; PR cannot merge without explicit
          attribute + ablation result attached to scope-policy.md

## DETERMINISTIC_CHECK

```bash
# 1. Policy doc exists with AEL citation
test -f .elai_cc/docs/phase-4h-scope-policy.md
grep -c "arXiv:2604.21725" .elai_cc/docs/phase-4h-scope-policy.md

# 2. Guardrail macro present
rg "allow.*self_mod_experimental" .elai_cc/crates/self_mod/src

# 3. Ablation harness compiles and runs
cargo test -p elai_self_mod --test ablation_harness 2>&1 | grep "test result.*ok"

# 4. SSB action types
rg "self_mod_layer_proposed|self_mod_ablation_run" .elai_cc/crates/orchestrator/src

# 5. Synthetic DGM-like negative delta confirmed by harness
cargo test -p elai_self_mod -- harness_surfaces_negative_delta 2>&1 | grep "test result.*ok"

# 6. DETAILED_SUMMARY §X annotated
grep "C-43" F:/.ELAI_workflow/docs/thefinalglance/summary/DETAILED_SUMMARY_01_overview_and_architecture.md

# 7. Mistakes.md row present
grep "self-mod scope expansion" workflow/sessions/$(cat workflow/sessions/current.txt)/mistakes.md
```

## ACCEPTANCE

- [ ] paper-ingestor produces workflow/papers/ael-ablation-interface.md
- [ ] phase-4h-scope-policy.md drafted with verbatim AEL quote
- [ ] Decision criteria (a)+(b)+(c) documented in policy
- [ ] Guardrail macro added to self_mod crate
- [ ] Existing 3 layers grandfathered via baseline allowlist
- [ ] Ablation harness runs deterministically against 2-config fixture
- [ ] TDD tests 4H.X.3..4H.X.6 written and passing
- [ ] Synthetic DGM-like candidate produces negative delta on harness
      (harness can SURFACE degradation, not rubber-stamp)
- [ ] DETAILED_SUMMARY §X annotated with scope-freeze footnote
- [ ] mistakes.md entry added
- [ ] CI lint pipeline rejects new self-mod path without explicit attribute

## Files

- ADD: .elai_cc/docs/phase-4h-scope-policy.md
- ADD: .elai_cc/crates/self_mod/macros/src/lib.rs (#[allow(...)] proc macro)
- ADD: .elai_cc/crates/self_mod/tests/ablation_harness.rs
- ADD: .elai_cc/crates/self_mod/tests/synthetic_dgm_fixture.rs (4H.X.6)
- ADD: workflow/papers/ael-ablation-interface.md
- MODIFY: .elai_cc/crates/self_mod/src/lib.rs (re-export macros)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb/schema.rs (2 new action types)
- MODIFY: .elai_cc/.clippy.toml or rust-toolchain (lint config for guardrail)
- MODIFY: F:\.ELAI_workflow\docs\thefinalglance\summary\DETAILED_SUMMARY_01_overview_and_architecture.md (§Phase 4H footnote)
- MODIFY: workflow/sessions/{id}/mistakes.md (scope-expansion guard row)

## Affected Verification Reports

- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\steps\phase-4H.report.md
  — annotate C-43 scope-freeze ratification on closure.

## Next

After C-43 lands: any future temptation to "add a self-improvement loop"
must run the ablation harness first. The harness produces a single
markdown table; that table is the evidence required to expand. Without the
table, no expansion. AEL paper is the closest published evidence ELAI has
that this discipline is correct, not just cautious.
