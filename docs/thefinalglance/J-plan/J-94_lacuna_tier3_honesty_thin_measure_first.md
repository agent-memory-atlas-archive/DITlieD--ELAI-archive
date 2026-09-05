# J-94: the Tier-3 honesty fixes + the THIN measure-first items — narrow wins, and the things we will NOT build on faith

- Track J (lacuna-fill track) | lever HONEST-DEGRADATION / MEASURE-FIRST | grounding STRONG (live D-6/J-73/C-29/QueryTrace/J-13/J-14/I-45/GoalDriftCanary/I-39 grep) | nature BUILD (Tier-3, cheap honesty fixes) + DEFERRED (THIN, measure-first) | effort S (Tier-3) / measure-only (THIN) | WAVE 3 | depends J-87 (for the C8-CANVAS shadow nodes), J-90 (L4 for C2-RESILIENCE strictness)
- Two clusters. TIER-3 = cheap honesty fixes (reasoning §5; build any time). THIN = measure-first items flagged honestly (reasoning §6; DO NOT build on faith — measure the value first). The split is the point: this file is as much about what we DEFER as what we ship.

## TIER-3 — honesty fixes (build any time; each is small + observe-only)
T3-A C1-GROUNDMODES route-by-measured-sufficiency: an OFFLINE calibration loop re-runs sampled claims through ALL grounding methods on D-6 held-out data, tightening the D-10 router predicate where the cheap method disagrees with the expensive one too often (route by MEASURED sufficiency, not authored prior). Reuse the grounding-mode router; no new SSB. RESIDUE: the calibration is only as good as the held-out sample's coverage — declare the sample, never claim global sufficiency.

T3-B C2-CASCADE which-modality retrieval router: a DETERMINISTIC query-modality classifier (query AST + C-29 intent-pair) selecting the CHANNEL SUBSET before fusion (grep-only on exact-symbol queries; dense+graph on "where is the logic that..."). ELAI runs all channels UNCONDITIONALLY; J-73's should_retrieve is WHEN, this is WHICH. Reuse QueryTrace; no new SSB. RESIDUE: a misclassified query under-retrieves -> the classifier defaults to the FULL channel set on low confidence (widen-on-doubt), never narrows speculatively.

T3-C C2-RESILIENCE degradation -> trust marker: thread retrieval-completeness (QueryTrace/SearchMode) into SSB run metadata; a change authored under DEGRADED(BM25_ONLY) gets a "context-incomplete" marker ESCALATING audit strictness (precedent: J-54 free-pool non-RRA stamp). Reuse knowledge_degradation +Kind; no new action_type. RESIDUE: the marker raises strictness, it does NOT block — a degraded run is honestly LABELED, the auditor decides.

T3-D C7-SIDECAR attention-determinism -> RRA tier: J-13 covers mm/addmm/log_softmax batch-invariance but NOT attention (hardware-blocked on Ampere/FA2). Bind that residue to the J-14/J-91 ReproPack tier: the local lane records T1 token-faithful ONLY when attention is batch-invariant, else honestly downgrades to T0 (CL-4/GD-5 — never advertise a guarantee a path cannot deliver). Reuse ReproPackEvent +Kind. RESIDUE: this is a HONEST-DOWNGRADE, not a fix — the attention non-determinism stays; we just stop claiming T1 where it does not hold.

T3-E C8-DASHBOARD plan-vs-execution drift view: render a READ-ONLY SSB projection diffing the I-45 sealed plan-skeleton against journaled executed actions, flagging skipped/substituted/drifted steps (safety-class drift = RED, consuming the J-93 leg-E C3-AUTONOMOUS fill). Substrate exists (sealed objective, plan skeleton, GoalDriftCanary, I-39); the dashboard just doesn't render it. Pure VIEW-over-SSB, no new SSB. RESIDUE: a view, not a gate — it surfaces drift for a human, the J-93 leg-E gate is what acts on safety-class drift.

## THIN — measure-first (flag honestly; DO NOT build on faith; reasoning §6)
These have MODERATE-or-uncertain value to ELAI's already-bounded design. MEASURE before building; each is recorded so it is not re-proposed as net-new, and DEFERRED until a measured case exists.
THIN-1 C3-BSP intra-agent superstep barrier: checkpoint+validate between an agent's tool-call PHASES using the H-49 edit-grain. Moderate value to ELAI's already-bounded Tier-C agents. MEASURE the mid-agent-failure rate before building.
THIN-2 C7-ROUTING phase-keyed mid-trace routing: switch model mid-trace at a phase boundary. TENSION with the HD-2 cache-prefix doctrine (only safe at a cache breakpoint); gated behind the THIN-1 barrier first; the cache-bust economics may not pay off. MEASURE the cache-bust cost vs the routing benefit before building.
THIN-3 C5-SELFMOD local-lane checker-as-cross-family substitute: designate the deterministic checker stack (H-43 execution-grounded fitness + J-68 Kani/TLA) as the local-lane "verifier stronger than generator" when VRAM cannot co-resident a second family (the L1/J-89 local-lane fallback). ELAI is LARGELY ALREADY AHEAD here; this is a REFRAME + an edge-case hardening (ensure the mutable=0 immune-tier MEMBERSHIP lives in the FROZEN gate-manifest, not mutable L1 data). The hardening (immune-tier-in-the-manifest) is the only buildable bit; ship THAT, defer the rest.
THIN-4 C8-CANVAS surface un-journaled shadow activity: derivative of E1 (J-87) — once the shadow stream exists, render shadow-effects-with-no-journal-row as ANOMALY nodes so the canvas honestly shows the gaps. DEPENDS J-87 shipping first; build AFTER J-87, not on faith.

## Existing ground truth (grep-confirmed; re-grep at edit, W-7/W-9)
- HELD-OUT STORE D-6 [DARK] + GROUNDING VERIFICATION D-10 [WIRED] (T3-A). RETRIEVAL-ABSTENTION GATE J-73 [PLANNED] + C-29 IntentPair [WIRED] (T3-B). RETRIEVAL RESILIENCE QueryTrace/SearchMode [WIRED] + J-54 non-RRA stamp (T3-C). LOCAL-LANE TOKEN DETERMINISM J-13 [PLANNED] + J-14/J-91 ReproPack (T3-D). I-45 seal [PLANNED] + GoalDriftCanary [PARTIAL] + I-39 + J-93 leg-E (T3-E). H-49 edit-grain (THIN-1), HD-2 cache doctrine (THIN-2), H-43 fitness + J-68 + the gate-manifest (THIN-3), J-87 (THIN-4).
- live SSB baseline ~268/577 (grep 2026-06-26).

## Integration
- TIER-3: all reuse existing SSB +Kind arms or are pure read-side views (net 0 new action_types). Each observe-only; each a wire-dark-allowlist row only if it adds a flag (most do not). T3-E is a Svelte read-only drawer + a #[tauri::command] (frontend_pair required, W-4 + dashboard-smoker).
- THIN: NO build this wave except THIN-3's gate-manifest hardening (move the mutable=0 immune-tier membership into the frozen .claude/gate-manifest.txt). THIN-1/2/4 are recorded DEFERRED with their measure-first trigger; THIN-4 builds after J-87.
- frontend_pair: T3-E plan-vs-execution drift drawer (W-4); THIN-4 anomaly nodes on the J-60 canvas (after J-87). W-6-justified where absent.

## WIRING_CONTRACT (Tier-3 shipping legs; THIN is DEFERRED-recorded, not contracted here)
produces_symbols:
- T3-A: crates/orchestrator calibrate_grounding_router (offline; tighten the D-10 predicate on D-6 disagreement)
- T3-B: crates/knowledge classify_query_modality (deterministic channel-subset; widen-on-doubt)
- T3-C: crates/orchestrator stamp_context_incomplete (DEGRADED -> trust marker raising audit strictness)
- T3-D: crates/orchestrator attention_determinism_tier (T1-only-when-batch-invariant; else T0 downgrade)
- T3-E: dashboard plan_vs_execution_drift (read-only SSB projection) + its #[tauri::command]
- THIN-3 (the only THIN build): .claude/gate-manifest.txt immune-tier membership row (mutable=0 frozen)
consumes_symbols:
- T3-A: D-6 held-out + D-10 router | T3-B: C-29 IntentPair + QueryTrace | T3-C: QueryTrace/SearchMode + J-54 stamp | T3-D: J-13 batch-invariance + J-14/J-91 ReproPack | T3-E: I-45 seal + I-39 + J-93 leg-E + the SSB stream | THIN-3: H-43/J-68 + gate_freeze_check.py
reachable_from:
- each Tier-3 leg from its real seam on a live run (a sampled-claim calibration tick; a query routed to a channel subset; a DEGRADED run stamped; a local-lane RRA tier decision; the drift drawer rendered from live SSB); THIN-3 from gate_freeze_check.py over the manifest  [PRODUCTION]
- a #[cfg(test)] / contracts/ caller NEVER satisfies any (R7)
stub_ban:
- NO speculative narrowing (T3-B widens on doubt; T3-A declares its sample)
- NO BLOCK from the trust marker (T3-C raises strictness, the auditor decides)
- NO T1 claim where attention is non-batch-invariant (T3-D honest T0 downgrade; CL-4)
- NO THIN item built on faith (THIN-1/2/4 are DEFERRED with a measure-first trigger; only THIN-3's manifest hardening ships)
- NO immune-tier membership in mutable L1 data (THIN-3: it lives in the FROZEN gate-manifest, R8)
frontend_pair: T3-E drift drawer (W-4 + dashboard-smoker); THIN-4 canvas anomaly nodes after J-87; W-6-justified where absent.
observable_effect:
- trigger: "T3-C: an `elai` run forced into DEGRADED(BM25_ONLY) (vector model absent). T3-E: an `elai` run that SKIPS a sealed plan-skeleton step. THIN-3: gate_freeze_check.py after an attempt to move an immune-tier rule into mutable L1 data."
- expect_ssb: "T3-C: a context-incomplete marker on the run's metadata; the auditor's strictness raised. T3-E: the drift drawer renders the skipped step RED (safety-class) from the live SSB. THIN-3: gate_freeze_check FAILS the attempt to relocate the immune-tier membership out of the manifest."
- expect_artifact: "T3-C: a change authored under degradation is HONESTLY LABELED and audited stricter, never silently trusted. T3-E: the human sees the skipped/substituted steps without trusting a narration. NEGATIVE (R2): T3-B classifier on an unparseable query WIDENS to the full channel set, never narrows blindly; T3-D claiming T1 on a non-batch-invariant path is REFUSED; THIN-3 an immune-tier rule in mutable data FAILS gate-freeze, never passes."

## Grounding
- Tier-3: reasoning §5 (the honesty fixes). THIN: reasoning §6 (measure-first; the explicit do-NOT-build-on-faith list). All DESIGN-RATIONALE only (LFG-3); each done-gate is the observable_effect above.
- substrate: D-6/D-10/J-73/C-29/QueryTrace/J-13/J-14/J-91/I-45/GoalDriftCanary/I-39/H-49/HD-2/H-43/J-68/gate-manifest/J-87 (feature-map 1.1/1.2/1.4/1.5/1.6/1.7/1.10/1.14).
- live SSB baseline ~268/577 (grep 2026-06-26).

## COMPOSES-WITH
D-6/D-10 (T3-A), J-73/C-29 (T3-B), QueryTrace/J-54 (T3-C), J-13/J-14/J-91 (T3-D), I-45/I-39/J-93-leg-E/J-60 (T3-E), H-49 (THIN-1), HD-2 (THIN-2), H-43/J-68/gate-manifest (THIN-3), J-87 (THIN-4). Restates none.

## Open questions / residue (reasoning §6/§7)
- THE THIN HONESTY: shipping all of Tier-3 + recording THIN as DEFERRED is itself the discipline — the marginal gate has diminishing trust-return and rising FP-cost (R1). Building THIN-1/2/4 without a measured case would be exactly the gate-proliferation the program warns against. The measure-first trigger for each is named; nothing here is built on faith.
- T3-D HONEST-DOWNGRADE NOT FIX: the attention non-determinism on Ampere/FA2 stays; T3-D only stops over-claiming T1 — the residue is hardware, not engineering.

## Next
T3-E consumes J-93 leg-E; THIN-4 builds only after J-87. This file closes the lacuna-fill track honestly: the cheap fixes ship, the uncertain-value items are deferred with a measure-first trigger, and nothing claims more than it measures.

End of J-94.
