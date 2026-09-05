# J-71: policy-as-code spike (Cedar) — a time-boxed evaluation of expressing ELAI's capability gates as analyzable policy, decision deferred to the spike

- Track J (NASA-gates wave) | lever AUDITABLE | grounding STRONG (cedar-interface.md W9; cedar-policy 4.11.1 anti-slopsquat-verified) | nature SPIKE (NOT a commitment) | effort S (1-agent spike) | WAVE NASA-GATES-5 | depends none (read-only re-expression of ONE existing gate)
- SPIKE-FIRST. No production wiring until the spike reports. May be declined.

- Premise: ELAI's transition/capability gates are frozen Python checkers + a Rust rules engine. GROUNDED OPPORTUNITY: Cedar (OOPSLA 2024, cedar-policy 4.11.1, Apache-2.0) lets a policy be expressed as analyzable policy-as-code over (principal = agent-role, action = tool/transition, resource = file/SSB-scope, context = FSM-state), and buys SMT policy-equivalence/regression analysis — you can PROVE a gate refactor or a new wire-dark exemption changed NO permissions (cedar-interface.md Theorem 4.2). That is a real capability the frozen-Python checkers lack (you cannot prove two Python checkers are equivalent). But it is a BIGGER move: it adds an SMT dependency AND a SECOND source of gate truth alongside the frozen-Python checkers — a real tradeoff. So J-71 is a TIME-BOXED SPIKE, not a build: re-express ONE existing capability gate as a Cedar policy + schema, run the equivalence check across a refactor, and report keep-Python vs adopt-Cedar with MEASURED numbers. The decision is deferred to the spike output (plan GROUP G); declining is a valid outcome.

## Steal/build (the spike, not a feature)
1. pick ONE existing capability gate (candidate: a transition gate or a tool-capability check the rules engine already enforces) and re-express it as a Cedar policy + schema over (principal=agent-role, action=tool/transition, resource=file/SSB-scope, context=FSM-state).
2. run Cedar's SMT policy-equivalence analysis (cedar-interface.md Thm 4.2) across a REFACTOR of that gate — prove the refactor changed no permissions (the payoff capability).
3. measure: authoring cost, the SMT analysis runtime, the expressiveness fit (can ELAI's real gate conditions be said in Cedar's schema?), and the second-source-of-truth cost (two places a gate now lives).
4. REPORT keep-Python vs adopt-Cedar with the measured numbers + a recommendation; if adopt, the production plan is a SEPARATE future file (never wired under this spike).
EXCLUDED: any production wiring of Cedar (the spike is read-only on the gate system; a real gate keeps its frozen-Python checker until a separate decision); adding cedar-policy as a workspace dependency (the spike runs it in an isolated scratch, user-confirmed; only an ADOPT decision adds the dep); treating the spike's one-gate re-expression as a migration (it is a measurement, not a rollout).

## Existing ground truth (PLAN-asserted unless noted; re-grep at edit, W-7/W-9)
- the rules engine 3F + G-1 capability/taint sets + the transition gates (the existing capability-gate system one of whose gates the spike re-expresses) — PLAN-asserted; pick the concrete gate at spike time.
- C-42 #[authz] DENY-ALL default (feature-map) — the existing authorization posture Cedar would express declaratively; the spike compares against it.
- cedar-policy 4.11.1 (Apache-2.0, anti-slopsquat-verified 2026-06-21) — the spike's tool; re-verify at edit; runs in an isolated scratch, NOT a workspace dep.
- .elai_cc/tests/contracts/test_cedar_contract.py (the premature stub a paper-ingestor agent left, plan N2/Q3) — either the spike's starting scaffold or removed; user's call (see open questions).

## Integration
- a 1-agent SPIKE producing workflow/reasoning/J-71-cedar-spike.md (the measured keep-vs-adopt report) + a throwaway Cedar policy/schema for the ONE chosen gate in a scratch dir. NO .elai_cc change, NO SSB, NO dependency add, NO gate wiring. The ONLY persistent artifact is the report (and a decision: keep-Python | author-a-Cedar-adoption-plan | decline).
- frontend_pair: N/A — a spike/report. Justified.

## WIRING_CONTRACT
produces_symbols:
- workflow/reasoning/J-71-cedar-spike.md (the measured keep-Python-vs-adopt-Cedar report + recommendation)
- a scratch Cedar policy + schema for ONE existing capability gate (throwaway, not wired)
consumes_symbols:
- ONE existing capability gate (rules engine 3F / a transition gate / C-42 authz — re-expressed read-only, NOT modified)
- cedar-policy 4.11.1 SMT equivalence analysis (the measured capability; isolated scratch, not a dep)
reachable_from:
- N/A by design — this is a SPIKE, not a production feature. The "entrypoint" is the spike report a human reads.  [SPIKE — no production wiring; deferred decision]
stub_ban:
- NO production wiring of Cedar under this unit (read-only spike; a real gate keeps its frozen-Python checker)
- NO cedar-policy added to a workspace Cargo.toml/pyproject (isolated scratch only; a dep add needs the ADOPT decision + user confirmation)
- NO presenting the one-gate re-expression as a migration (it is a measurement)
- NO recommendation without MEASURED numbers (authoring cost + SMT runtime + expressiveness fit + second-source cost; "Cedar seems nice" is not a spike result, GD-5)
frontend_pair: N/A — spike/report. Justified.
observable_effect:
- trigger: "run the spike: re-express ONE existing capability gate as a Cedar policy + schema, run the SMT equivalence check across a refactor of it"
- expect_artifact: "workflow/reasoning/J-71-cedar-spike.md exists with the four measured numbers (authoring cost, SMT runtime, expressiveness fit, second-source cost) + a keep-Python | adopt-Cedar | decline recommendation; the equivalence check on the refactor returns a concrete equivalent/not-equivalent verdict (the Thm 4.2 payoff demonstrated)"
- expect_artifact (NEGATIVE / honesty, R2/GD-5): "a report that recommends adopt WITHOUT the measured second-source-of-truth cost is incomplete (the tradeoff is the whole point); a spike that wired Cedar into any real gate FAILS the read-only constraint; an unverified cedar-policy version FAILS anti-slopsquat at edit"

## Grounding
- W9 cedar-interface.md (OOPSLA 2024; the principal/action/resource/context model + Theorem 4.2 SMT policy-equivalence — the analyzability payoff that frozen-Python checkers cannot offer).
- cedar-policy 4.11.1 Apache-2.0 (anti-slopsquat-verified 2026-06-21; re-verify at edit).
- relates to I-1 (directive supervisor plane) / I-35 (temporal tool sequence) — Cedar could express their capability gates; the spike measures whether it should.
- the tradeoff framing (SMT dep + second source of truth vs the frozen-Python status quo) is the plan's GROUP G explicit caution; the decision is deferred, declining is valid.

## COMPOSES-WITH
the rules engine 3F + G-1 capability sets + C-42 authz (the gate system one gate is re-expressed from), J-70 (if adopted, a Cedar gate would declare its DO-333 traceability row like any frozen gate), I-1 / I-35 (capability/transition gates Cedar could later express). The premature .elai_cc/tests/contracts/test_cedar_contract.py is this spike's scaffold-or-removal (plan Q3). Restates none; re-implements nothing (read-only).

## Open questions / risks
- SECOND SOURCE OF TRUTH is the real risk: adopting Cedar means a gate lives in BOTH a Cedar policy and (during migration) its Python checker — the spike must measure that drift cost, not hand-wave it; if the report cannot quantify it, the recommendation defaults to keep-Python (the conservative outcome).
- EXPRESSIVENESS FIT: ELAI's gate conditions (evidence-absent->Block, step-count temporal predicates) may not fit Cedar's (principal,action,resource,context) cleanly — the spike's one-gate choice should be a gate that PLAUSIBLY fits, and the report must say honestly where it did not.
- the premature test_cedar_contract.py (plan N2/Q3): the user decides remove-or-keep-as-scaffold; the spike does not silently adopt it as committed.

## Next
J-71 is deliberately last and deliberately non-committal: it buys a measured answer to "should ELAI's gates be policy-as-code" without betting the gate system on it. If the spike says adopt, a separate future plan file wires it (with J-70's traceability); if it says decline, the frozen-Python checkers (J-65 + the existing family) stand, and the wave closes on the deterministic-gate posture it started with.

End of J-71.
