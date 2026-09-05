# H-plan Track 6 (GROUNDED-REASONING HARDENING) — authoring record + handoff
Date: 2026-06-04. Session: autonomous (user asleep, "do all that for me, take your time, reason, ground").
Trigger: from the Hallucination-Stations research thread (YouTube AIYQp1n51ZI + arXiv:2507.07505 + a 3-angle
deep-research sweep), the user asked to (1) study the H-plan format, (2) grep + reason, (3) author new H-plan
files for the improvements found, (4) update H-SUMMARY_INDEX.md + H-0_INDEX.md, (5) ingest 3 remaining papers,
(6) decide if those warrant their own plan, author if yes.

## What was produced (all under docs/thefinalglance/H-plan/ — gitignored, NOT committed)
- _grounding/H-REASONING-GROUNDING-MAP.md — the W-7 symbol map (RS-1..RS-13 shipped substrate, RG-39..RG-42 gaps,
  RD-1..RD-4 delta rules, the verified CITATIONS, the LIVE 220/497 SCHEMA note).
- H-39_abstention_aware_promotion_eval.md — [KEYSTONE] abstention-aware promotion + eval reward.
- H-40_decorrelation_keyed_verifier_retention.md — decorrelation axis on the iso-budget harness.
- H-41_checker_coverage_gap_instrument.md — live Mode-A coverage + checker-gap backlog instrument.
- H-42_serial_depth_reasoning_budget.md — serial-depth-scaled GRC budget + the depth-not-width guard.
- H-SUMMARY_INDEX.md — added the Track 6 section (4 entries + cross-cutting) + fixed the stale title.
- H-0_INDEX.md — added Track 6 to the header, the narrative, the RD-1..RD-4 deltas, the session table, the SCHEMA
  bump sites, WAVE 6 sequencing, and the next-step + collision note.

## The reasoning (grounded; mcp_think topic elai-improvement, then the live-code grounding pass)
The video/paper (2507.07505) argue an O(N^2*d) per-pass ceiling forces hallucination, verifiers share it, CoT helps
nothing. A 3-angle sweep (all arXiv ids verified) found:
- CoT claim REFUTED: poly CoT = EXACTLY P (2310.07923), T steps simulate a size-T circuit (2402.12875), GD provably
  LEARNS NC1 with CoT (2511.07378). The bound is on the single PASS (~TC0), not the autoregressive loop.
- Inevitability is over-determined; the COMMON hallucinations are low-complexity, driven by eval incentives rewarding
  confident guessing over abstention (OpenAI 2509.04664).
- Verification "as hard as the task" is true ONLY for a CORRELATED verifier; decorrelated ensembles (Weaver 2506.18203)
  and sound external checkers (Kambhampati 2402.08115) help; the honest ceiling is the Pass@K oracle.

## The crucial grounding correction (anti-hallucination)
The D/E plan SUMMARIES called much of this "PLANNED", but a live elai.map + crates/** grep showed it is SHIPPED:
- grounding_mode.rs:110 select_grounding_mode (D-10), CALLED at agent_pool.rs:2030; Rule 5 (:157) ALREADY routes
  sound-checker-first; Auditor/Meta -> ConservativeFallback ALREADY firewalls the auditor.
- checker_registry.rs CheckerResult (D-11); conformal_calibration.rs RefusalDecision (D-16); refusal_sla.rs (D-17);
  iso_budget_harness.rs insufficient_lift (E-6); self_certainty.rs + micro_reflection_emitter.rs (E-19);
  config_archive.rs/promotion_gate.rs:109 two_gate_promote/bai.rs:286/darwin_admit_loop.rs (the promotion loop);
  coding_metrics pass_at_k; safedelta_runner.rs/bench_seccodeplt.rs (the measured-row pattern); mode_routing_runner.rs (D-21).
So Track 6 is ACTIVATION/EXTENSION on shipped code, and the VERDICT is mostly CONFIRMATION: ELAI's trust spine already
aligns with the rigorous literature. I did NOT propose "add sound-checker-first" (it is shipped) — H-41 became an
INSTRUMENT instead. This is the honest, anti-sycophantic read.

## The 4 confirmed gaps (each verified against the real code, not a summary)
- RG-39 (H-39): the OPTIMIZED reward (two_gate_promote / bai / pass_at_k / bench) is abstention-blind while the abstain
  primitive (RefusalDecision) ships separately — the self-mod loop can select for confident guessing. THE one real
  cross-subsystem misalignment. Fix: AbstentionAwareScore (correct=+1/abstain=0/confident-wrong=-t/(1-t), t LOCKED 0.75->2x
  per 2509.04664) fed into the reward + bench; read the abstain bit from RefusalDecision.
- RG-40 (H-40): iso_budget_harness flags insufficient_lift (marginal accuracy), no decorrelation axis -> can prune a
  valuable decorrelated verifier (Weaver). Fix: unique_catch/decorrelation on IsoBudgetReport; prune only on both-low.
- RG-41 (H-41): the A>B preference + the OFFLINE F1 benchmark exist, but no LIVE coverage signal nor a ranked checker-gap
  backlog. Fix: a live Mode-A coverage roll-up + +CheckerGapDetected classified by the Xu Table-3 complexity taxon.
- RG-42 (H-42): GRC budget is per-role STATIC; CoT-step lower bounds say it should scale with serial depth. Fix:
  estimate_serial_depth -> scale_grc_budget (LOCKED ceiling) + a DepthBound/AccessBound classifier (depth != width).

## Paper decision (the user asked: "see if they are worth making a plan for too")
DECLINED a 5th file — honest outcome. The 3 papers were ingested (interface maps on disk) and evaluated:
- 2509.04664 (OpenAI) GROUNDS H-39 (gave the exact penalty=t/(1-t) formula + Observation 1). Folded in, not separate.
- 2401.11817 (Xu) REINFORCES the external-oracle/checker design; its one net-new idea (Table-3 complexity taxonomy ->
  route NP-complete/undecidable to a solver/refuse, never LLM-grade) FOLDED INTO H-41 (the taxon classifier).
- 2502.12187 (Suzuki) REINFORCES GD-4 FP-bounding (worst-case vs real-distribution). No new mechanism.
None warranted a standalone plan; making one would have been invented scope.

## Caveats / open items for the executor
- RAG INGEST FAILED for all 4 papers (the local-rag reranker is down — known issue project_rag_reranker_crash). The
  interface maps + summaries are ON DISK (workflow/papers/*.md) and are the authoritative artifact; mcp_search_context
  will NOT find them until the RAG primary is recovered (clear primary.pid + restart) and they are re-ingested.
- SCHEMA baseline is LIVE 220/497 (ssb.rs:2990 / action_type_lock.rs:624) — the H-0 Track-1/2/3 header's 207/484 and
  CLAUDE.md's 191/464 are STALE. The first Track-6 H-N to bump SCHEMA must also fix the stale e45 variant pin (~482).
- The H-39..H-42-decisions.md files are EXECUTOR tasks (created at execution per each file's Files-touched), NOT pre-created here.
- NOT committed: docs/thefinalglance/ is gitignored (outside .elai_cc/); no commit was needed or made. No code in
  .elai_cc/ was touched — this was plan authoring only.
- Each H-N re-verify its arXiv ids at execution (feedback_paper_citation_verify) before any number headlines a gate.

## Suggested next step
Execute WAVE 6 starting with H-39 (the keystone; the abstention-aware reward is what the whole self-improvement loop
climbs), then H-40/H-41/H-42 in parallel. Recover the RAG primary + re-ingest the 4 papers so the interface maps are
queryable. The 4 files are wiring-enforced (W-1 --run-trigger done-gate) and default-ON-with-kill-switch.
