# J-73: retrieval-abstention gate — decide whether to retrieve AT ALL before firing the cascade, reusing the shipped abstention infra

- Track J (context-retrieval wave) | lever CHEAPER | grounding STRONG (Repoformer arXiv:2403.10059 interface-mapped; the report calls it a more durable lever than HyDE/multi-query) | nature EXTENSION (repurpose answer-abstention infra to gate retrieval) | effort M | WAVE CR-2 | depends none

- Premise: ELAI ALWAYS retrieves when it is in a retrieval tier — codegraph-confirmed 2026-06-23 there is NO should_retrieve gate (the symbol search returned none). Repoformer (arXiv:2403.10059) shows a self-assessment P(retrieve) threshold (T=0.15 function, 0.20 line/API) trained into the LM as a multi-task head yields >70% speedup in dense settings at >80% abstention precision: when the model already knows the answer / the local context suffices, retrieval is pure cost. ELAI already has abstention MACHINERY for ANSWERS (abstention_score.rs + conformal_calibration.rs); the steal REPURPOSES that to gate the cascade, not a new mechanism.

## Steal/build
1. should_retrieve GATE in the pipeline BEFORE the BM25/ONNX cascade fires. The decision signal comes from EITHER the sidecar (Qwen2.5-Coder lane emitting a <cc>/abstain special token — the Repoformer multi-task-head shape) OR, default, a cheap calibrated heuristic REUSING the shipped abstention_score + conformal_calibration (Abstain enum, decide_low_score_abstains, agent_pool low_score_routes_to_abstain). Per-task-type threshold T (function vs line/API) lives near the existing FAST/STANDARD/DEEP tier router (query_class).
2. The gate is DETERMINISTIC at decision time (TD-3): the abstention score is a calibrated number, the threshold is config; the LLM does not adjudicate "should I retrieve". A retrieval that is gated OFF is journaled so the saving is auditable and the FP (skipped-but-needed) rate is measurable.
EXCLUDED: training a NEW retrieval-decision head from scratch (the cheap path reuses the shipped abstention calibration; the sidecar special-token path is the Tier-2 upgrade, RunPod-gated); removing any cascade stage (this gates WHETHER to enter the cascade, it does not prune stages — that is J-75, and is user-gated).

## Existing ground truth (grep-confirmed 2026-06-23; re-grep at edit, W-7/W-9)
- ABSTENTION (answers): crates/orchestrator/src/abstention_score.rs + conformal_calibration.rs — Abstain enum, decide_low_score_abstains, agent_pool low_score_routes_to_abstain (codegraph-confirmed). The infra being repurposed to gate RETRIEVAL.
- TIER ROUTING: FAST/STANDARD/DEEP via query_class (feature map 1.1 ADAPTIVE CHUNKING + QUERY EXPANSION, phase-3K) — where the per-task-type T config sits.
- CATRAG + A-RAG cascade: crates/knowledge/src/pipeline.rs is where the should_retrieve gate lands, BEFORE the bm25 -> vector -> pagerank -> catrag -> arag cascade.
- CONFORMAL REFUSAL (UniCR) D-16 [DARK]: the conformal tau machinery the gate's calibration can reuse (compose, do not re-author).

## REVISED scope (W-10 tiered)
- SHIP NOW Tier-1 (in-process, model-free): the should_retrieve gate in pipeline.rs reusing abstention_score + conformal_calibration + the per-task-type T config near query_class; OBSERVE-ONLY first (GD-4 — journal the gate decision, do not actually skip, until the FP rate is bounded on >=3 real runs), then enforce.
- DEFER Tier-2 (remote sidecar): the Repoformer-style <cc>/abstain special-token head on the Qwen2.5-Coder lane — needs sidecar token support, remote-GPU-gated, pod_stop after.

## Integration
- crates/knowledge/src/pipeline.rs — the should_retrieve gate BEFORE the cascade (~60 LOC: read the calibrated abstention signal, compare to per-task-type T, journal + skip-or-proceed).
- near query_class (tier router) — the per-task-type T config (function/line/API thresholds, named constants, not magic numbers).
- SSB: +RetrievalAbstained (Pattern-2 single-variant; Kind: GatedSkip / GatedProceed / AbstainOverride). baseline+1 from LIVE ~269/578 (post-J-66; grep 2026-06-22), re-read at edit (W-9). PREFER-REUSE: if QueryTrace (the 9-stage retrieval trace, phase-1I) already carries a decision-stage field, add the abstain verdict there instead (net 0) — decide at edit.
- cross_lang pair: none (Rust-internal); frontend_pair: the retrieval-skipped rate on KnowledgeDrawer (B-29) or W-6-justified absence.

## WIRING_CONTRACT
produces_symbols:
- crates/knowledge/src/pipeline.rs should_retrieve gate (calibrated decision before the cascade)
- per-task-type retrieval-threshold config near the tier router
consumes_symbols:
- crates/orchestrator/src/abstention_score.rs + conformal_calibration.rs (the calibrated signal; NOT re-authored)
- D-16 conformal tau machinery (the calibration; NOT re-authored)
- query_class tier router (where T attaches; NOT re-authored)
reachable_from:
- a real `elai` retrieval query / task path entering crates/knowledge/src/pipeline.rs  [PRODUCTION]
- a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
- NO LLM adjudication of should_retrieve at runtime (TD-3 — the score is calibrated, the threshold is config)
- NO enforced skip before the FP (skipped-but-needed) rate is bounded on >=3 real runs (GD-4 — OBSERVE-ONLY first)
- NO new abstention calibration impl (reuse abstention_score / conformal_calibration; IC-13 one-calibration-impl)
- NO cascade-stage removal (this gates entry, not stages; pruning is J-75 + user-gated)
- NO new default-OFF ELAI_*_ENABLE without a wire-dark-allowlist row + promotion plan (R9/W-2)
frontend_pair: retrieval-skipped rate on KnowledgeDrawer (decide at edit) or W-6-justified absence.
observable_effect:
- trigger: "a real `elai` retrieval query where the local context already suffices (a within-file completion the model can answer without the index), run through pipeline.rs with the gate enforcing"
- expect_ssb: "RetrievalAbstained Kind=GatedSkip with the calibrated score + the T it cleared (or the QueryTrace decision-stage field carrying the abstain verdict)"
- expect_artifact: "the cascade did NOT fire (no bm25/vector/rerank work for that query) and the answer was produced from local context; the journaled skip is auditable. NEGATIVE (R1 fail-closed): a query whose score is BELOW T proceeds to the full cascade (abstention is conservative — a missing/uncertain signal RETRIEVES, never silently skips); OBSERVE-ONLY mode journals the would-skip without skipping."

## Grounding
- Repoformer arXiv:2403.10059 (workflow/papers/repoformer-interface.md, .claude/papers/2403.10059.html): self-assessment P(retrieve) threshold T=0.15 function / 0.20 line/API, multi-task head, >70% dense-setting speedup, >80% abstention precision — DESIGN-rationale (the ELAI threshold is calibrated on ELAI data, ID-5). arXiv ID re-verify at edit.
- ELAI seam codegraph-confirmed 2026-06-23: abstention_score.rs + conformal_calibration.rs exist for ANSWERS; no should_retrieve symbol exists (context-retrieval-research-2026-06-23.md C2).
- the report (deep-research-report(2).md) ranks retrieval-abstention a MORE durable lever than HyDE/multi-query (HyDE already conditional in ELAI, R3).
- LIVE SSB ~269/578 (post-J-66; grep 2026-06-22). re-grep at edit (W-9).

## COMPOSES-WITH
abstention_score.rs / conformal_calibration.rs (the repurposed signal), D-16 conformal tau (calibration), query_class tier router (threshold home), QueryTrace (the 9-stage trace the skip decision rides), J-72 (when the adapted retriever ships, the gate decides whether to even invoke it). Restates none.

## Open questions / risks
- calibration source: the cheap heuristic reuses answer-abstention calibration, but "is this query answerable from local context" is a different distribution than "is this answer confident" — the first checkbox is measuring the FP rate on a held-out query set before enforcing (GD-4).
- the sidecar special-token path (Tier-2) needs Qwen2.5-Coder to emit a usable <cc>/abstain logit — verify the lane supports it before committing the head; the cheap calibrated path ships regardless.

End of J-73.
