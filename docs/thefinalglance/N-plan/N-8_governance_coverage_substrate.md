# N-8: governance-coverage substrate — five views of "is our own governance real," incl. kill-your-own-mutant-corpus

- Track: TIER 3 (MEDIUM-HIGH) | lever: a-number-no-competitor-can-utter | finding: W-18+W-34+W-31 | grounding: STRONG | nature: BUILD | effort: M | WAVE 5

- Premise: five instruments of "is our own governance real," none citing more than one of the others. Together they answer "what part of our own governance is decorative" — the exact self-trust question the product sells. Frozen prevents tampering; it does NOT prove the gate catches what it claims. This is the marketable meta-property.

## Steal/build
ONE governance-coverage substrate with FIVE views:
- B-17 gate-liveness ("does this gate actually FIRE") — runtime-gate grain;
- K-22 doctrine-enforcer coverage ("does this rule have an ENFORCER") — authored-doctrine grain;
- J-46 doctrine-debt-RATE ("is adoption keeping pace");
- J-70 gate-manifest traceability ("does every frozen gate declare objective/evidence");
- the H-27 kill-your-own-mutant-corpus generalized from finance ("does the enforcer actually CATCH" — mechanically prove each gate kills 100% of a frozen mutant corpus of its OWN logic + satisfies its declared MRs before ACTIVE).
Make kill-your-own-mutant-corpus a gate-manifest LIFECYCLE requirement: a new gate cannot enter the frozen set (.claude/gate-manifest.txt) without its mutant corpus + measured kill rate. The N-2 corpus validator (plan edges) is a sixth grain of the same "what governance is decorative" question.
Headline number: "N% of ELAI's rules are machine-enforced, each enforcer mutation-proven, drift-rate X/quarter" (K-1-dossier grade).
EXCLUDED: re-implementing any of the five (view them); a new gate framework; claiming the number before it is MEASURED through N-1 (ND-4).

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)
- B-17 gate-liveness tests, K-22 clause->enforcer coverage, J-46 debt-rate, J-70 gate-manifest traceability exist (feature-map governance group); each authored 4-8 letters apart, none citing more than one other.
- H-27 mechanically proves each FINANCE gate kills a frozen mutant corpus of its own logic (H-plan Track 7, TB-3, TD-4).
- ~40 other deterministic gates (wiring, maintainability, P10, taint, SOLID-proxy, admission scanners) self-validate only by authorship care + R8 freezing.
- .claude/gate-manifest.txt is the frozen-gate registry (R8, gate_freeze_check.py).

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1): the substrate joining B-17 + K-22 + J-70 into one coverage view + the headline number (measured through N-1) + kill-your-own-mutant-corpus as a gate-manifest lifecycle requirement for NEW gates.
- DEFER (Tier-2): back-filling mutant corpora for all ~40 existing gates (the lifecycle rule stops the bleed; the back-fill is a rolling task).

## Integration
- .claude/tools/ governance-coverage checker + the gate-manifest lifecycle hook; crates/orchestrator for the runtime-gate-liveness leg.
- frontend_pair: a governance tile (the marketable number) on G-5/M-3.

## WIRING_CONTRACT
produces_symbols:
  - the governance-coverage substrate (five views over one join) + the headline number
  - the gate-manifest lifecycle requirement (mutant corpus + kill rate to enter the frozen set)
consumes_symbols:
  - B-17 gate-liveness / K-22 doctrine coverage / J-46 debt-rate / J-70 traceability / H-27 mutant-kill
  - .claude/gate-manifest.txt + gate_freeze_check.py; N-1 (measures the number)
reachable_from:
  - a CI/authoring invocation of the coverage checker  [PRODUCTION governance seam]
  - the gate-manifest lifecycle at gate-freeze time  [PRODUCTION freeze seam]
stub_ban:
  - NO re-implementation of the five instruments
  - NO new gate entering the frozen set without a mutant corpus + kill rate
  - NO headline number not recomputable from the journal (GD-2) / not measured through N-1 (ND-4)
frontend_pair: governance tile on G-5/M-3
observable_effect:
  trigger: "python <the governance-coverage checker> over the rulebook + gate-manifest"
  expect_ssb: "+governance_coverage_measured(GateLiveness|DoctrineEnforcer|DebtRate|MutantKill)"
  expect_artifact: "the coverage % + a NEW gate lacking a mutant corpus is REFUSED entry to the frozen set; NEGATIVE: a decorative rule (clause with no enforcer) reported as covered must FAIL"

## Grounding
- interlace doc W-18 (B-17 vs K-22 grain), W-34 (four governance-coverage members), W-31 (gate self-validation), TIER-3 item 8.
- H-plan Track 7 (H-27); wiring-enforcement R8 (frozen gate manifest).

## COMPOSES-WITH
B-17, K-22, J-46, J-70, H-27 (generalized), gate-manifest, N-1, N-2 (the plan-edge sixth grain). Re-implements NONE (GD-1). Upgrades the whole gate stack from "frozen" to "frozen and proven-sharp" — a claim no competitor can state.

End of N-8.
