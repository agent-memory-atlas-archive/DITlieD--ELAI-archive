# L-6: Risk-as-curiosity targeted-effort router — spend effort where coverage is weak

STATUS: PLANNED (WAVE 3; M; BUILD — the paper's mitigation half, ported)

Objective: the paper does not only DETECT hallucination — it uses the predictor AS a curiosity reward to collect data where coverage is weak (§4.3), turning detection into prevention. L-6 ports this: a High/Critical ClaimRisk (L-2) becomes a curiosity signal that AUTO-FIRES targeted effort at exactly the weak claim (not blanket) — mid-task agentic retrieval (J-38) at the unresolved symbol, the graduated read-ladder escalate-on-miss (J-103), a targeted test/trace, or a local>cloud escalation (I-7) — then RE-SCORES the claim via L-2. This is "preventable, not just detectable" (HG-4: an unverifiable high-risk claim routes to verification, never a silent pass). The targeted effort is BOUNDED by the J-84 budget (curiosity is targeted, never a blanket loop). A claim that becomes grounded after targeted effort is eligible to mint a Verified Case Graph node (J-96) = the harness-level coverage-gap closure, no weights touched. HARD BOUNDARY: L-6 ROUTES effort and re-scores; it does NOT decide the final allow/block (that is L-7). GD-1: do NOT re-implement J-38/J-103/I-7 — dispatch them.

Estimated effort: ~1 session (~M). In-process routing; the retrieval/escalation legs inherit their own network/RunPod posture.
Pre-requisites (re-grep live per W-7): J-38 retrieval agentic loop (RetrievalLoopEvent); J-103 graduated read-ladder (escalate-on-miss); I-7 cascading escalation (EscalationCascade); B-29 impact API (where to look); J-84 effort classifier (the budget); J-96 case-mint; L-2 ClaimRisk.
Parallel-safe with: L-7, L-8 (disjoint files). Depends L-2.
Source: cgg-hallucination-causal-system-2026-06-27.md S6 + paper §4.3 (targeted data collection / curiosity reward) + L-CGG-GROUNDING-MAP NN-5/TS-12. Doctrine: L-0_INDEX.md (HG-4).

## Context
Why this matters: a gate that only flags is a friction tax; a gate that flags AND closes the gap is leverage. The paper's curiosity-reward result (50 targeted trajectories ~= human-oracle data collection, §5.1) is the strongest argument for routing effort BY the risk signal rather than spending it uniformly. The LLM analogue is precise: a high-risk file-claim with unresolved refs should trigger a targeted J-38 retrieval at THAT symbol, not a blanket re-read; a high-risk dependency claim should trigger an API_REGISTRY lookup, not a full re-plan. HG-4 makes this non-optional: an unverifiable high-risk claim MUST route to verification or abstention (L-7), never a silent allow. The compounding win (J-96): the resolution is recorded as a reusable case, so the same coverage gap costs less next time.

## Existing state (B/I/J coverage)
- J-38 retrieval agentic loop [PLANNED] — mid-task callable hybrid_search the executor loops on; RetrievalLoopEvent. The targeted-retrieval sink. [MAP]
- J-103 graduated read-access ladder [PLANNED] — escalate-on-miss climb (a hallucinated identifier escalates to a fuller read). [MAP]
- I-7 cascading escalation [WIRED] — local>cloud waterfall EscalationCascade. The escalate sink. [MAP]
- B-29 symbol-level impact API [WIRED] — impact_radius/call_flow/symbol_context; where-to-look. [MAP]
- J-84 reasoning-effort classifier [PLANNED] — the budget for the targeted effort. [MAP]
- J-96 verified-case-graph CORE [PLANNED] — mint-on-F-4-proof case node. The compounding closure. [MAP]

## Reads (mandatory before coding)
1. _grounding/L-CGG-GROUNDING-MAP.md (NN-5, TS-12); cgg-hallucination-causal-system-2026-06-27.md S6; paper §4.3.
2. J-38 retrieval-loop interface; J-103 read-ladder; I-7 EscalationCascade; B-29 impact API; J-84 budget; L-2 ClaimRisk.
3. F-0 W-1; G-0 GD-4; CLAUDE.md TD-3, no-.unwrap.

## Tasks (WAVE 3 — route effort by risk, re-score)
1. the curiosity router (~70 LOC): map (ClaimRisk tier, ClaimKind, which-predictor-fired) > a targeted action — refs-unresolved File/Symbol > J-38 retrieval at that symbol (B-29 locates it); invented Dependency > API_REGISTRY lookup; high P3 on a Behavior claim > a targeted test/trace; unresolvable locally > I-7 escalation. NO blanket re-read (targeted only).
2. the re-score loop (~40 LOC): after the targeted effort lands evidence, re-invoke L-2 on the claim; bound the loop by the J-84 budget (max rounds, cost cap) — curiosity is targeted, never unbounded (HG-4 routes, it does not spin).
3. the case-mint hook (~20 LOC): a claim that becomes grounded after targeted effort is eligible to mint a J-96 case node (observe-only; the actual mint stays J-96's gated path).
4. wire into the L-2 post-score seam (~20 LOC). default OBSERVE-only (GD-4): record the routed actions, do not yet block (L-7 blocks).
5. workflow/reasoning/L-6-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/claim/curiosity_router.rs — the risk>action map + the bounded re-score loop + the case-mint hook
consumes_symbols:
  - J-38 retrieval loop (RetrievalLoopEvent); J-103 read-ladder; I-7 EscalationCascade; B-29 impact API; J-84 budget; J-96 case-mint (COMPOSE all — never re-implement)
  - claim/risk.rs::ClaimRisk (L-2)
reachable_from:
  - claim/risk.rs post-score seam > route_targeted_effort [PRODUCTION curiosity routing]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO blanket re-read/re-plan (targeted effort at the specific weak claim only)
  - NO unbounded loop (J-84 budget caps rounds + cost)
  - NO silent allow of an unverifiable high-risk claim (HG-4 — route to verification or hand to L-7)
  - NO re-implementation of J-38/J-103/I-7/J-96 (GD-1 P0); NO .unwrap on prod
frontend_pair:
  the retrieval/escalation surface (decide at edit, W-4) OR "N/A — routed actions surface via the existing retrieval/escalation panels. Justified."
observable_effect:
  trigger: "score a High-risk file-claim with an unresolved symbol ref (real L-2 seam) and let the router fire targeted retrieval, then re-score"
  expect_ssb: "prefer-reuse RetrievalLoopEvent (J-38) / escalation rows; net 0 expected from LIVE ~268/577 (re-grep, W-9)"
  expect_artifact: "the High-risk claim auto-fires a J-38 retrieval at THAT symbol and re-scores Lower after the evidence lands; NEGATIVE: a Low-risk claim fires ZERO targeted effort (the cost guard — curiosity is targeted, never blanket)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/L-plan/L-6_risk_as_curiosity_targeted_effort_router.md --run-trigger
rg "route|targeted|budget|re.?score" .elai_cc/crates/orchestrator/src/claim/curiosity_router.rs   # the router present
rg -ci "blanket|reread_all|full_replan" .elai_cc/crates/orchestrator/src/claim/curiosity_router.rs # MUST be 0 (targeted only)
test -f workflow/reasoning/L-6-decisions.md
```

## ACCEPTANCE
- [ ] HG-4: an unverifiable high-risk claim routes to verification or is handed to L-7; never silently allowed.
- [ ] targeted effort hits the specific weak claim (via B-29), never a blanket re-read.
- [ ] the re-score loop is bounded by the J-84 budget (max rounds + cost cap).
- [ ] GD-1: J-38/J-103/I-7/J-96 consumed, not re-implemented.
- [ ] GD-4: ships observe-only (records routed actions) before any block-by-default.
- [ ] cargo check --workspace GREEN; auditor PASS on -p orchestrator. No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: done = a live High-risk claim auto-fires targeted retrieval and re-scores lower (not cargo-green).
- W-2: no behavior block here (observe-only); the block is L-7.
- W-3: composes BUILT/PLANNED retrieval+escalation, not a summary-only port.
- W-4: frontend_pair decided at edit or justified N/A.
- W-5/W-10: routing in-process; the legs inherit their own posture.
- W-7: J-38/J-103/I-7/B-29/J-84 interfaces re-grepped at edit.
- W-9: SSB prefer-reuse RetrievalLoopEvent; net 0.
- GD-2: the routed-action decision is recomputable from the ClaimRisk under ELAI_SEED.

## Tests to add
- crates/contracts/tests/l6_curiosity_router_contract.rs (~80 LOC): (a) a High-risk unresolved-symbol claim routes a targeted J-38 retrieval at that symbol; (b) re-score after evidence lands lowers the tier; (c) a Low-risk claim fires zero effort; (d) the loop respects the J-84 budget (caps rounds); (e) an unverifiable high-risk claim is never silently allowed (handed to L-7); (f) the routed action is recomputable from the ClaimRisk.

## Files touched
- CREATE: .elai_cc/crates/orchestrator/src/claim/curiosity_router.rs (~130 LOC)
- MODIFY: claim/risk.rs post-score seam (~20 LOC)
- CREATE: crates/contracts/tests/l6_curiosity_router_contract.rs (~80 LOC)
- CREATE: workflow/reasoning/L-6-decisions.md

## Citations / Grounding
- arXiv:2606.27326 §4.3 (targeted data collection via the predictor as curiosity reward; 50 trajectories ~= oracle) [EXTERNAL/DESIGN-RATIONALE]; cgg-hallucination-causal-system-2026-06-27.md S6.
- ELAI ground truth (L-CGG-GROUNDING-MAP NN-5/TS-12): J-38, J-103, I-7, B-29, J-84, J-96 (re-grepped at edit, W-7).
- CLAUDE.md: TD-3, no .unwrap on prod; HG-4.

## Open questions / risks
- The risk>action map completeness: not every ClaimKind has a cheap targeted action; for those, route to L-7's abstain rather than spin (HG-4). Decide the full map at authoring; OBSERVE-first to bound the false-route rate (GD-4).
- Interaction with autonomous runs: targeted effort consumes budget; the J-84 cap + the autonomous-gate envelope (B-22) must agree so curiosity does not blow the autonomous token ceiling. Verify against the live envelope.

## Next
L-6 turns the risk score into prevention. L-7 makes the final allow/caveat/block/abstain decision on the (possibly re-scored) tier; L-8 prunes the claims that targeted effort proved false so they do not poison later steps.

End of L-6.
