# J-39 decomposition_node_admission_gate

- Track J | lever decomposition-determinism | grounding arXiv:2604.22750 + arXiv:2510.07772 + arXiv:2505.07897 + metrics/baseline_findings_and_actions.md | nature build (upgrade of 4E sizer) | effort M | WAVE DD-1 | depends J-34, B-29, base-4E
- The model decides babystep size and count today, and that decision is unreliable: pre-execution TOTAL token cost is provably unpredictable (Pearson r<0.15, same-task 30x variance, arXiv:2604.22750). But pre-execution INPUT context is exactly countable. So sizing is a HARNESS decision, not a model one: the model PROPOSES a decomposition, a deterministic admission gate measures each node's INPUT lower-bound against the measured per-(model,lane) ceiling and REJECTS oversized nodes (re-split) or ESCALATES them to a larger-context lane. This is bound-and-escalate, never predict-precisely. Upgrades the existing 4E Stage-3.5 phase-sizing heuristic.

## Steal/build
1. EXP-1 (OBSERVE-ONLY, step 1, no gating until passed): on 50 SWE-bench Verified tasks, correlate the dependency-cone token size against actual agent token spend. r>0.5 = cone has actionable sizing signal; r<0.2 (consistent with the published total-cost r<0.15) = cone is a hard FLOOR only and the gate must stay floor-mode. The experiment decides the gate's authority. No default-ON until FP/over-split rate bounded on >=3 real runs (GD).
2. Dependency-cone INPUT lower-bound estimator: from a proposed node (entry files + intent), compute the transitive import/call cone via the B-29 impact API, tokenize each file in the cone, sum = exact input lower-bound. Deterministic, no LLM, no model call.
3. Admission gate at the 4E splitter seam: admit a node to the LOCAL lane iff lower-bound <= admit_fraction * measured_ceiling (ceiling from J-34's per-(model,lane) curve when it lands; FALLBACK to the measured baseline 3B ~40K / 7B ~25K so this is NOT blocked on J-34). Over-ceiling -> SplitForced (re-split at a graph community boundary) or EscalatedToCloud (I-7 cascading escalation). Per-session OUTPUT-token cap wired as a CIRCUIT BREAKER (output is fat-tailed, P95/P50>5x, uncontrollable pre-exec) -- never a precise gate.
4. CROSS-CUTTING ESCAPE (mandatory, from the attack-the-design analysis): a cross-cutting-but-atomic intent (e.g. "add audit logging to all mutations") has a whole-codebase cone but is a single semantic operation. The gate sizes the scope SUPERSET (files that COULD be touched); the LLM still selects the change SUBSET. If the proposed node carries an atomic-intent marker and a huge cone, the gate must NOT force a pointless split -- it routes to cloud or flags for human, never deadlocks on the cone.
EXCLUDED: pre-execution OUTPUT-token prediction (r<0.15, deferred); ACONIC CSP-treewidth decomposition (requires a CSP reduction that is itself non-deterministic for open-ended coding goals -- narrow-domain only, not adopted here); any precise total-cost predictor.

## Existing ground truth
(grep-confirmed 2026-06-16 via grounding agent; re-grep at edit, W-7/W-9)
- `DefaultPhaseSizingSplitter` at `crates/orchestrator/src/phase_sizing_splitter.rs:150` (impl `TaskSplitter`); `estimate_complexity` at `:200` (threshold = `DEFAULT_TOKEN_BUDGET * SPLIT_RATIO` 0.6, `MAX_SPLIT_DEPTH` 2); `estimate_tokens` at `:173`. -- the seam: inject the ceiling-based admission check before the `should_split` branch, comparing against `ELAI_LOCAL_MAX_CONTEXT` / `ModelCapabilityProfile.calibrated_trim_fraction` instead of the fixed budget constant.
- B-29 symbol-level impact API: `impact_radius` / `call_flow` / `symbol_context` traversal tools (knowledge crate, feature-map SOT). EXACT defining file:line UNCONFIRMED by the grounding pass (the agent surfaced `arag.rs::keyword_search` instead) -- re-grep `impact_radius` at edit before wiring (W-7).
- `ModelCapabilityProfile` at `crates/orchestrator/src/context_budget.rs:292` -- currently has NO `calibrated_trim_fraction` / curve fields (those are J-34 PLANNED additions); `CLOUD_TRIM_FRACTION = 0.85` at `crates/orchestrator/src/context_policy.rs:70`.
- Measured ceilings: `metrics/baseline_findings_and_actions.md:150-151,216` -- 3B NF4+FA2 ~40K, 7B ~25K, both 100% pass on a 12GB card.
- Circuit-breaker precedent: agent supervision + budget breakers in `crates/orchestrator` (phase-3C/4G).

## Integration
Wraps, does not replace, `DefaultPhaseSizingSplitter`: the existing complexity heuristic stays as the proposal scorer; the admission gate is a new deterministic check between `estimate_complexity` and `should_split` that swaps the fixed `DEFAULT_TOKEN_BUDGET` for the live lane ceiling. Consumes B-29 for the cone, J-34 for the curve (fallback to baseline). Escalation routes through the shipped I-7 cascading-escalation path. Honest residue (SWEnergy, ICSE-2026): sizing makes a small local model FIT the context, not CAPABLE of the task (4B models ~0% agentic resolution); the local lane is for bounded leaves, hard leaves escalate -- this gate is necessary, not sufficient, for local viability.

## WIRING_CONTRACT
produces_symbols: [NodeAdmissionGate, derive_cone_token_floor, NodeAdmissionEvent, NodeAdmissionEventKind]
consumes_symbols: [DefaultPhaseSizingSplitter, ModelCapabilityProfile, impact_radius (B-29), EscalationCascade (I-7)]
reachable_from: [the 4E phase-sizing splitter call site in the pipeline-factory L2 decomposition path; CLI decompose/plan entrypoint]
stub_ban: [no unwrap in lib code, no placeholder ceiling constant, no silent SKIP on missing cone -- missing cone data = FAIL-closed re-grep, not pass]
observable_effect: one `NodeAdmissionEvent` SSB row per decomposed node recording {decision: Admitted|SplitForced|EscalatedToCloud, cone_token_floor, lane_ceiling, lane}
SSB: +NodeAdmissionEvent (Pattern-2 single-variant, 3-arm NodeAdmissionEventKind: Admitted | SplitForced | EscalatedToCloud). Full 9-tuple golden-lock lockstep; bump from the LIVE baseline (SCHEMA 268 / TV 577 at authoring) computed at implementation, NEVER hard-coded (W-9); needs a wire-dark-allowlist.toml row only if it ships behind a flag (default-ON observe-only emit preferred).

## Grounding
- [How Do AI Agents Spend Your Money? Analyzing and Predicting Token Consumption in Agentic Coding Tasks, arXiv:2604.22750](https://arxiv.org/abs/2604.22750) -- pre-exec total token cost r<0.15, 30x same-task variance, input dominates; the empirical basis for bound-and-escalate over predict-precisely. VERIFIED 2026-06-16.
- [LongCodeBench, arXiv:2505.07897 (COLM 2025)](https://arxiv.org/abs/2505.07897) -- coding accuracy 29%->3% from 32K->256K, non-monotonic, can't infer from short-context; motivates a measured per-model ceiling. VERIFIED.
- [ACONIC, arXiv:2510.07772](https://arxiv.org/abs/2510.07772) -- CSP-treewidth decomposition; cited for the EXCLUDED boundary (CSP reduction is non-deterministic for open-ended tasks). VERIFIED.
- Graph community-detection partitioning (Louvain / classic graph partitioning) -- the re-split technique; textbook, citation-independent.
- PROVISIONAL / UNVERIFIED: "When Parallelism Pays Off: Cohesion-Aware Task Partitioning for Multi-Agent Coding" (claimed arXiv:2606.00953) -- title/authors appear in search summaries but NO canonical arxiv.org/abs page landed across 3 searches (2026-06-16). DO NOT cite as load-bearing; confirm the ID manually before relying. The community-detection technique above stands without it.

## COMPOSES-WITH
- J-34: consumes the per-(model,lane) `fill_degradation_curve` / `calibrated_trim_fraction` as the admission ceiling (fallback to measured baseline until J-34 lands).
- B-29: dependency-cone via the impact API.
- base-4E: upgrades `DefaultPhaseSizingSplitter` (the proposal scorer it wraps).
- I-7: EscalatedToCloud routes through the cascading-escalation path.
- ELAI-DETERMINISM-DOCTRINE (docs/strategy): the canonical "model proposes / gate disposes on size" application.

## Open questions / risks
- EXP-1 outcome gates the whole design: r<0.2 demotes the gate to floor-only (still useful: blocks provably-oversized nodes), r>0.5 enables it as a real admission controller. Do not ship default-ON before EXP-1.
- Over-split risk: a node that would converge in 3 iterations should not be split for orchestration overhead. The gate sizes INPUT only; it cannot see iteration depth. Mitigate with MAX_SPLIT_DEPTH 2 (existing) + the cross-cutting escape.
- SeqCV (NeurIPS-2025) infinite-split-loop on non-decomposable subtasks -- the depth cap is the guard; a node that fails admission AND cannot split escalates, never loops.
- B-29 exact impact-API symbol unconfirmed this pass -- re-grep before wiring.

## Next
J-40 builds the research-modes extractive-compression lane (the same bounded-context-unit discipline applied to research steps).
