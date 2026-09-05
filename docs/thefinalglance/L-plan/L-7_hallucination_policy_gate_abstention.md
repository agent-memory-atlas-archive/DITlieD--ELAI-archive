# L-7: Hallucination policy gate + abstention integration — risk to allow/caveat/force/abstain [CANT-WRECK]

STATUS: PLANNED (WAVE 3; M; BUILD-compose; EXTENSION(D-16/AutonomousGate) — the decision surface, observe-only first)

Objective: the predictors and the curiosity router produce a ClaimRisk tier; L-7 turns it into an ACTION — Low > allow; Medium > attach a caveat / require a citation; High > force the L-6 targeted effort before the claim may act; Critical > BLOCK the edit/action or ABSTAIN. HG-4 is the law: an unverifiable high-risk claim routes to verification or abstention, NEVER a silent allow (missing evidence = high risk, wiring R1 over claims). It composes the SHIPPED gate fabric — the AutonomousGate EvidenceCheck per transition (B-22) + conformal refusal (D-16) + Aletheia voluntary abort (I-8, credited by the abstention score H-39) + the honest-refusal SLA (D-17). The verdict is an SSB row, so the whole gate decision is RRA-reproducible (G-2). It ships OBSERVE-only first (GD-4): record the would-be action + bound the FP/over-block rate over >=3 real runs before BLOCK-by-default, behind ELAI_HALLUCINATION_GATE_DISABLE (W-2). HARD BOUNDARY: the LLM never sets or overrides the action (TD-3); the gate is deterministic over the tier. GD-1: do NOT re-implement D-16/AutonomousGate/I-8 — wire them.

Estimated effort: ~1 session (~M). In-process at the transition/edit seam; no network.
Pre-requisites (re-grep live per W-7): D-16 conformal refusal (RefusalDecision, tau); I-8 Aletheia acknowledge_infeasible; D-17 honest-refusal SLA; B-22 AutonomousGate EvidenceCheck per transition; H-39 abstention-aware (AbstentionScored); G-2 RRA (rra.rs); L-2 ClaimRisk; L-6 router.
Parallel-safe with: L-8 (disjoint). Depends L-2, L-6.
Source: cgg-hallucination-causal-system-2026-06-27.md S9 + the notes' policy-gate table + L-CGG-GROUNDING-MAP NN(policy)/TS-4/TS-6/TS-21. Doctrine: L-0_INDEX.md (HG-4, GD-4, TD-3).

## Context
Why this matters: a risk score with no action surface is telemetry; the gate is where the CGG earns its name. The notes' table (Low>allow, Medium>caveat/cite, High>force-tool, Critical>block/clarify) maps onto ELAI's existing abstention + autonomy fabric exactly. The discipline that makes it safe is GD-4 (observe-first — the FP/over-block rate is what killed prior gate generations, per the K-T3 research; never BLOCK-by-default before the rate is bounded on >=3 runs) and HG-4 (never a silent allow — an unverifiable high-risk claim is abstained, with Aletheia crediting the abstention so the agent is not penalised for honesty, H-39). The verdict being an SSB row means a third party can replay the run and re-derive why a claim was blocked (G-2) — the gate is itself honest by ELAI's standard.

## Existing state (B/D/G/H/I coverage)
- D-16 conformal refusal [DARK(ELAI_CONFORMAL_REFUSAL_ENABLE)] — RefusalDecision; tau nightly L3 update. The abstain primitive. [MAP]
- I-8 Aletheia voluntary abort [PLANNED] — acknowledge_infeasible tool credited by abstention score; dead-end AST hashing. [MAP]
- D-17 honest-refusal SLA [DARK] — <5% confabulating / >50% misconfigured thresholds. [MAP]
- B-22 AutonomousGate [DARK-flagged] — AutoAdvanceEvaluator deterministic EvidenceCheck per transition; tree-level risk. The transition seam. [MAP]
- H-39 abstention-aware scoring [PARTIAL] — abstain=0 / confident-wrong=-t/(1-t). [MAP]
- G-2 RRA [WIRED, rra.rs] — the reproducible artifact the verdict joins. [MAP]

## Reads (mandatory before coding)
1. _grounding/L-CGG-GROUNDING-MAP.md (TS-4/6/21); cgg-hallucination-causal-system-2026-06-27.md S9; the notes' policy-gate table.
2. D-16 RefusalDecision; I-8 acknowledge_infeasible; D-17 SLA; B-22 EvidenceCheck; rra.rs; L-2 ClaimRisk + L-6 router.
3. F-0 W-1; G-0 GD-2/GD-4; CLAUDE.md TD-3, no-.unwrap.

## Tasks (WAVE 3 — map risk to action, observe-first)
1. the policy map (~50 LOC): a deterministic ClaimRisk-tier > GateAction{Allow, Caveat, ForceEffort, Block, Abstain}. HG-4: High > ForceEffort (hand to L-6); Critical > Block-or-Abstain; missing-evidence = high risk. NO LLM input (TD-3).
2. wire into the AutonomousGate EvidenceCheck (~40 LOC): a Critical claim makes the transition's EvidenceCheck FAIL; an Abstain routes through I-8 acknowledge_infeasible (credited by H-39); a Caveat annotates the output. Compose D-16 tau for the abstain boundary.
3. the journaled verdict (~30 LOC): emit the GateAction as an SSB row so the decision is RRA-reproducible (G-2). default OBSERVE-only behind ELAI_HALLUCINATION_GATE_DISABLE (W-2): record the would-be action, do not block, until the FP rate is bounded on >=3 runs (GD-4).
4. workflow/reasoning/L-7-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/claim/policy_gate.rs — the ClaimRisk>GateAction map + the journaled verdict + the observe/enforce switch
consumes_symbols:
  - D-16 RefusalDecision + tau; I-8 acknowledge_infeasible; D-17 SLA; B-22 AutonomousGate EvidenceCheck; H-39 AbstentionScored; G-2 rra.rs (COMPOSE all — never re-implement)
  - claim/risk.rs::ClaimRisk (L-2); claim/curiosity_router.rs (L-6, the ForceEffort target)
reachable_from:
  - the AutonomousGate transition seam + the edit/action seam > hallucination_gate_action [PRODUCTION gate]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO silent allow of an unverifiable high-risk claim (HG-4)
  - NO LLM setting/overriding the GateAction (TD-3 — deterministic over the tier)
  - NO BLOCK-by-default before the FP/over-block rate is bounded on >=3 runs (GD-4)
  - NO re-implementation of D-16/AutonomousGate/I-8 (GD-1 P0); NO .unwrap on prod; blake3 not sha2
frontend_pair:
  the refusal/mode panels (D-27, decide at edit, W-4) OR "N/A — verdicts flow to the existing refusal/mode panels. Justified."
observable_effect:
  trigger: "drive a transition with a Critical claim (invented package on a destructive path) + a Medium claim, through the real AutonomousGate seam"
  expect_ssb: "prefer-reuse RefusalDecision (D-16) / AutonomousGate EvidenceCheck rows; else +HallucinationGateAction (Pattern-2; Kind: Allowed/Caveated/ForcedEffort/Blocked/Abstained); baseline+0..1 from LIVE ~268/577 (re-grep, W-9)"
  expect_artifact: "the Critical claim is BLOCKED or abstained (decision journaled + RRA-reproducible, G-2); the Medium claim ships with a caveat; NEGATIVE: an unverifiable High claim is NEVER silently allowed (HG-4); under ELAI_HALLUCINATION_GATE_DISABLE the gate observes-and-records but does not block"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/L-plan/L-7_hallucination_policy_gate_abstention.md --run-trigger
rg "GateAction|Abstain|Block|Caveat" .elai_cc/crates/orchestrator/src/claim/policy_gate.rs   # the map present
rg -c "ELAI_HALLUCINATION_GATE_DISABLE" .elai_cc/crates/orchestrator/src/claim/policy_gate.rs # kill-switch present (W-2)
test -f workflow/reasoning/L-7-decisions.md
```

## ACCEPTANCE
- [ ] HG-4: an unverifiable high-risk claim routes to ForceEffort/Abstain, never a silent allow.
- [ ] TD-3: the GateAction is deterministic over the tier; the LLM never sets/overrides it.
- [ ] GD-4: observe-only behind ELAI_HALLUCINATION_GATE_DISABLE; BLOCK-by-default only after the FP rate is bounded on >=3 runs.
- [ ] GD-2: the GateAction verdict is an SSB row, RRA-reproducible.
- [ ] abstention is credited via I-8/H-39 (the agent is not penalised for honest refusal).
- [ ] GD-1: D-16/AutonomousGate/I-8 wired, not re-implemented.
- [ ] cargo check --workspace GREEN; auditor PASS on -p orchestrator. No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: done = a live Critical claim FAILs the AutonomousGate EvidenceCheck and the verdict is RRA-reproducible (not cargo-green).
- W-2: kill-switch ELAI_HALLUCINATION_GATE_DISABLE; observe-then-enforce.
- W-3: composes BUILT gate fabric, not a summary-only port.
- W-4: frontend_pair decided at edit or justified N/A (refusal/mode panels).
- W-5/W-10: in-process; no network leg.
- W-7: D-16/B-22/I-8/rra.rs re-grepped at edit.
- W-9: SSB prefer-reuse RefusalDecision/EvidenceCheck or +1.
- GD-2: verdict recomputable from the ClaimRisk + the gate map under ELAI_SEED.
- R10: the integration smoke drives the REAL transition on a HARD multi-turn input (a Critical claim mid-run), not a toy happy-path.

## Tests to add
- crates/contracts/tests/l7_policy_gate_contract.rs (~85 LOC): (a) Low>Allow, Medium>Caveat, High>ForceEffort, Critical>Block/Abstain; (b) an unverifiable High claim is never silently allowed (HG-4); (c) the LLM cannot override the GateAction (TD-3); (d) under ELAI_HALLUCINATION_GATE_DISABLE the gate records but does not block (GD-4); (e) a Critical claim FAILs the AutonomousGate EvidenceCheck; (f) an Abstain is credited via H-39 (not penalised); (g) the verdict is RRA-reproducible (G-2).

## Files touched
- CREATE: .elai_cc/crates/orchestrator/src/claim/policy_gate.rs (~120 LOC)
- MODIFY: the AutonomousGate transition seam + the edit/action seam (~40 LOC)
- CREATE: crates/contracts/tests/l7_policy_gate_contract.rs (~85 LOC)
- CREATE: workflow/reasoning/L-7-decisions.md

## Citations / Grounding
- the notes' policy-gate table (Low/Medium/High/Critical > allow/caveat/force/block); cgg-hallucination-causal-system-2026-06-27.md S9.
- ELAI ground truth (L-CGG-GROUNDING-MAP TS-4/6/21): D-16 RefusalDecision, I-8 acknowledge_infeasible, B-22 EvidenceCheck, rra.rs (re-grepped at edit, W-7).
- CLAUDE.md: TD-3, no .unwrap on prod, blake3-only; HG-4; GD-4 observe-first.

## Open questions / risks
- Over-block (the gate-killer): if the Critical tier fires too often it becomes friction the user disables; that is why GD-4 (observe-first, bound the FP rate) is non-negotiable here, and why the composed friction must fit the K-4 gate-tax ceiling (cross-link, not a K-4 amend). Decide the BLOCK-by-default promotion only after L-10's measured FP rate.
- Caveat fatigue: too many Medium caveats train the user to ignore them; tune the Medium boundary against L-10's reviewer-reaching metric.

## Next
L-7 is the decision surface. L-8 prunes the claims the gate/router proved false so they do not poison later steps; L-10 measures whether the gate's blocks/abstentions actually reduce hallucination without unacceptable over-block.

End of L-7.
