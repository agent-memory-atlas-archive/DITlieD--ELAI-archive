# J-66: SSB temporal-monitor engine — past-time-LTL online monitors + future-time session-end obligations over the journal

- Track J (NASA-gates wave) | lever CANT-DERAIL | grounding STRONG (efficient-monitoring-safety W4 + monitoring-rewriting W13 + enforceable-security W2 + rv-survey W5 interface maps) / EXTERNAL (the RV literature) | nature BUILD / EXTENSION(I-35) | effort L | WAVE NASA-GATES-2 | depends J-69 (the chain-verified journal the monitors read)
- The concrete monitor-synthesis algorithm I-35 (Pro2Guard DTMC) named but never specified.

- Premise: ELAI's SSB is an append-only event stream; today nothing compiles a TEMPORAL safety property over it into an online verdict. GROUNDED GAP: I-35 TemporalToolSequenceEnforcement is PLANNED but lacks the compilation mechanism; the efficient-monitoring-safety-interface.md (W4) supplies it — a past-time-LTL formula compiles to O(1)-per-event recurrences over the SSB action_type stream, firing per committed row. The future companion (W13, monitoring-rewriting-interface.md) is the SESSION-END obligation monitor by formula-rewriting: an obligation undischarged at session close is a fail-closed BLOCK, never "inconclusive". Two LLM-free legs (TD-3, the engine adjudicates deterministically). The ND-1 feasibility pre-lint gates authoring: every monitor predicate must be a SAFETY property (the Schneider 3-condition test, W2) and predicate on STEP/EVENT counts, never wall-clock — a liveness/wall-clock predicate is rejected to a watchdog, not added as an SSB gate. The RV-survey layer table (W5) is the routing doctrine (ND-2).

## Steal/build
1. crates/orchestrator/src/ssb_monitor/ (new module): a ptLTL compiler + a future-time rewriting evaluator; gate predicates registered against SsbActionType. Deterministic; the LLM never adjudicates (TD-3).
2. LEG B1 — PAST-TIME ONLINE monitors (W4): compile a ptLTL safety formula into the O(1)-per-event recurrences (efficient-monitoring-safety-interface.md Part B); fires per committed row in the SSB broadcast-after-commit path. The two NET-NEW gates the plan names:
   G1 always(write_tool -> diamond_past policy_allow)  — a write/edit tool-call must be preceded by an allow row (write-after-allow).
   G2 always(egress -> not_since(scope_widen, declassify))  — no egress after an un-declassified scope-widen (no-egress-after-widen).
   (G3 "test_passed -> diamond_past stdout_hash" is ALREADY the F-4 cant-lie gate — do NOT rebuild, NG-1.)
3. LEG B2 — FUTURE-TIME SESSION-END monitors (W13): formula-rewriting; an obligation undischarged at session close = BLOCK (monitoring-rewriting-interface.md):
   Oblig1 always(agent_spawned -> eventually agent_returned)  — checked at the MEMORIZING->COMPLETE transition.
   Oblig2 always(finding_block -> eventually finding_closed)  — every BLOCK finding reaches closure or the session cannot COMPLETE.
4. FEASIBILITY PRE-LINT (ND-1 / W2): a small authoring-time lint (a deterministic check over the registered predicate set) asserting each predicate is a safety property AND step/event-counted; a liveness or wall-clock predicate is REFUSED at registration with the reason. This is the engine's own gate-on-the-gates.
EXCLUDED: any wall-clock or liveness predicate as an SSB monitor (-> watchdog, ND-1); a re-build of the F-4 test_passed gate (NG-1); a SKIP/inconclusive end-of-trace verdict (W13's whole point is the fail-closed end verdict, R1); an LLM scoring the trace (TD-3).

## Existing ground truth (grep-confirmed where noted; else PLAN-asserted, re-grep at edit, W-7/W-9)
- SSB append-only stream + SsbActionType discipline: ssb.rs:62 (the 577-entry array), :1503/:1508 (the derived SCHEMA) — grep-confirmed 2026-06-22. The broadcast-after-commit path + the MEMORIZING->COMPLETE transition are the two consumer seams (PLAN-asserted; re-grep at edit).
- F-4 AntiFabricationGate at task_runner.rs (feature-map 1.6 [WIRED]) — G3 is already this; the monitor engine is its temporal generalization, not a duplicate.
- I-35 TemporalToolSequenceEnforcement (PLANNED) — the plan slot this fills with the concrete W4 compilation; J-66 is the EXTENSION, reusing I-35's action_type registry if it shipped a payload (prefer-reuse).
- J-69 chain-verified journal (this wave) — the monitors read a journal whose integrity the J-69 chain-scan has already proven (the read order is the SSB total order; tamper is caught before the monitor runs).

## Integration
- NEW crates/orchestrator/src/ssb_monitor/{mod,ptltl,rewrite,registry,feasibility}.rs (~400-600 LOC): the ptLTL compiler (B1) + the rewriting evaluator (B2) + the SsbActionType-keyed predicate registry + the ND-1 feasibility lint. Online monitors run in the SSB broadcast-after-commit path; session-end monitors run in the MEMORIZING->COMPLETE transition. The verdict feeds the FSM gate.
- +SsbMonitorVerdict SSB (Pattern-2 single-variant; Kind: PastTimeViolation / ObligationUndischarged / FeasibilityRejected). baseline+1 from LIVE 268/577 (ssb.rs:62/:1508, grep 2026-06-22); re-read at edit (W-9). Full 6-part variant discipline + golden-lock lockstep. (Or reuse I-35's action_type if it shipped one — prefer-reuse, net 0.)
- frontend_pair: monitor-verdict rows on an existing drawer (the audit/monitor surface; decide at edit, W-4) or a W-6-justified absence for the first runtime-only slice.

## WIRING_CONTRACT
produces_symbols:
- crates/orchestrator/src/ssb_monitor/ptltl.rs::compile (ptLTL formula -> O(1)-per-event recurrence set)
- crates/orchestrator/src/ssb_monitor/rewrite.rs::step (future-time obligation rewriting; end-of-trace verdict)
- crates/orchestrator/src/ssb_monitor/registry.rs (G1/G2/Oblig1/Oblig2 registered against SsbActionType)
- crates/orchestrator/src/ssb_monitor/feasibility.rs::lint (ND-1 safety + step-count authoring gate)
consumes_symbols:
- ssb.rs SsbActionType + the broadcast-after-commit hook (the per-row online seam; NOT a new writer)
- the MEMORIZING->COMPLETE FSM transition (the session-end seam)
- J-69 chain-scan (the integrity precondition: monitors run on a chain-verified journal)
reachable_from:
- a real `elai` task run: G1/G2 fire in the live broadcast path; Oblig1/Oblig2 fire at MEMORIZING->COMPLETE  [PRODUCTION]
- a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
- NO LLM call inside any monitor (deterministic recurrences + rewriting; TD-3)
- NO wall-clock or liveness predicate registered as an SSB monitor (ND-1 feasibility lint refuses it)
- NO SKIP / "inconclusive" end-of-trace verdict (an undischarged obligation is BLOCK, R1/W13)
- NO rebuild of the F-4 test_passed->stdout_hash gate (NG-1; reuse F-4)
- NO new SSB writer (the monitor reads the journal + emits its OWN verdict row through the single writer)
- NO BLOCK-by-default until the FP rate is bounded on >=3 real runs (GD-4) + a wire-dark-allowlist row (W-2/R9)
frontend_pair: monitor-verdict rows on the audit surface (decide at edit) or W-6-justified absence.
observable_effect:
- trigger: "drive the REAL entrypoint on a HARD input — an `elai` run that issues a write/edit tool-call with NO prior policy_allow row (expect G1), and a run that spawns an agent which never emits a return row before MEMORIZING (expect Oblig1)"
- expect_ssb: "SsbMonitorVerdict{Kind: PastTimeViolation} naming G1 at the write row; at session close SsbMonitorVerdict{Kind: ObligationUndischarged} naming Oblig1; the FSM cannot advance past the violated gate"
- expect_artifact (NEGATIVE, R1/R2): "a well-formed run (every write allow-preceded, every agent returns) emits no violation and COMPLETEs; a registration of a wall-clock predicate FAILS the feasibility lint (FeasibilityRejected), never silently accepts; the engine pointed at a journal whose J-69 chain-scan failed REFUSES to run (db-not-found / tamper = FAIL, R2), never an empty pass"

## Grounding
- W4 efficient-monitoring-safety-interface.md Part B (the past-time-LTL -> O(1) recurrence compilation; Havelund/Rosu 2004); W13 monitoring-rewriting-interface.md (future-time formula-rewriting, fail-closed end verdict; Havelund/Rosu 2001).
- W2 enforceable-security-policies-interface.md (Schneider 2000 safety-property feasibility 3-condition test; the ND-1 source) + W5 rv-survey-interface.md (Leucker/Schallhart 2009 layer-assignment + 3-valued monitorability caveat; the ND-2 source).
- precedent: F-4 cant-lie gate (G3 is already this temporal shape, shipped) — confirms the method on real ELAI rows.
- LIVE SSB baseline 268/577 (ssb.rs:62/:1508, grep 2026-06-22); +1 = SsbMonitorVerdict (or I-35 reuse).

## COMPOSES-WITH
I-35 TemporalToolSequenceEnforcement (the plan slot; J-66 supplies its compilation algorithm, reuses its registry if present), F-4 cant-lie gate (G3 sibling, NOT rebuilt), J-69 (the chain-verified journal precondition), the SSB single writer (phase-3B; J-66 never forks it), H-54 obligation-closure gate (the future-obligation leg is ADJACENT — keep separate per plan PART 3). Restates none.

## Open questions / risks
- MONITOR FALSE POSITIVES on legitimate sequences (a write whose allow row is in a parent session) — the predicate scope (per-session vs cross-session diamond_past) must be pinned per gate; ship OBSERVE-ONLY and measure (GD-4), LOG the would-be-BLOCK count.
- the ND-1 feasibility lint is itself a checker that can be wrong — keep its 3-condition test deterministic and frozen (gate-manifest, R8), and treat an UNKNOWN classification as REFUSE (ND-3), never as accept.
- monitorability (W5 3-valued caveat): some properties are not soundly monitorable online — those route to a session-end obligation (B2) or to J-68's design-time check (ND-2), never forced into an online monitor that can only ever say "maybe".

## Next
J-66 gives the journal a temporal conscience; J-67's multi-mode gate actions (Suppress/Insert/Edit) need J-66's verdicts to decide WHICH action to take, and the inserted/suppressed actions become SSB rows the monitors then see — so J-69 (integrity) -> J-66 (verdict) -> J-67 (typed action) is the dependency spine of the runtime-enforcement half of the wave.

End of J-66.
