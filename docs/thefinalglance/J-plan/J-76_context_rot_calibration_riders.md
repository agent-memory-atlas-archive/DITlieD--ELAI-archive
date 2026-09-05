# J-76: context-rot ceiling + placement + local _compact_history recipe — the measure-the-own-curve riders, as a file (never amend J-34/H-50 in place)

- Track J (context-retrieval wave) | lever CHEAPER | grounding STRONG (Chroma context-rot + lucek out-of-context + anthropic context-engineering, all interface-mapped; RLM arXiv:2512.24601 _compact_history verbatim recipe) | nature EXTENSION (J-34 + H-50) | effort S-M | WAVE CR-3 | depends none (EXTENDS J-34 + H-50; authored as a NEW file per the house never-amend-old-plans rule)

- Premise: the report's context-rot findings (C6) and the RLM compaction recipe (C7) are real, groundable retrieval/context levers that the research doc tagged "fold into J-34/H-50". The house rule forbids amending shipped/planned plan files in place (old files are never re-checked), so they ship as THIS new file that EXTENDS J-34 + H-50 — keeping them actionable while honoring fewest-files (two riders, one file). The CENTRAL discipline (already J-34's): IMPORT THE METHOD, NOT THE NUMBERS — the cited curves are on 18 FRONTIER models, NOT Qwen2.5-Coder-7B; ELAI MEASURES its own per-(model,lane) curve on the sidecar before setting any threshold.

## Steal/build
1. CONTEXT-ROT CEILING + PLACEMENT (C6, extends J-34): three concrete knobs grounded by the empirical degradation finding (a focused 300-token prompt beats a 113K full prompt on LongMemEval; middle-position chunks lose 30%+ vs head/tail "lost in the middle"; degradation onset ~5-7K words for the studied tasks): (a) cap the retrieval chunk COUNT injected per query; (b) PLACE top chunks at head/tail, never the middle (drives the 3E slot ORDERING J-34 already names via the positional-value curve); (c) prefer a focused pre-filtered prompt over a full-context dump (validates H-50 selective-reload). J-34 already MEASURES the per-(model,lane) knee instead of trusting folklore/the hardcoded CLOUD_TRIM_FRACTION=0.85 — these knobs CONSUME J-34's measured curve, they do not re-author the calibration.
2. LOCAL _compact_history RECIPE (C7, extends H-50): RLM's _compact_history (triggers at 85% of the window, summarizes progress PRESERVING intermediate numerics, truncates history to [system + initial + summary + "continue from summary"]) is a validated reference for H-50's LOCAL compaction policy (ELAI's local lane has flat-100K auto-compact that never fires <32K). The verbatim summarization prompt (rlm-interface.md) is adaptable to the Qwen sidecar 32K window. This is the LOCAL-lane compaction recipe H-50 needs; the CLOUD lane keeps deterministic masking (HD-2 — LLM re-summarization on the cache prefix stays REJECTED).
3. C8 SproutRAG (DEFERRED, recorded here not authored): a hierarchical attention-tree index (function->class->module agglomerative max-linkage tree, internal nodes = multi-granularity summaries, beam-search retrieval returning leaves AND coarser spans) with a COSINE FALLBACK (pairwise chunk-embedding cosine, ONNX-portable today). NEW but lower-ROI than C1-C5 — a later experiment unit, not this wave. Recorded as an open question so it is not re-litigated.
EXCLUDED: importing the frontier-model numbers as ELAI thresholds (ID-5 — measure the own curve); LLM re-summarization on the CLOUD cache prefix (HD-2 REJECTED, busts cache; CLOUD stays deterministic masking); the RLM REPL-as-context architecture (C7 DEFER — model-writes-code-to-navigate-context overlaps ELAI's sandbox/tool loop, heavy and uncertain fit, recorded as a rider not authored); 32K+ context as a fix for depth-class reasoning (RD-4 REJECTED).

## Existing ground truth (grep-confirmed 2026-06-23; re-grep at edit, W-7/W-9)
- EMPIRICAL CONTEXT CALIBRATION [PLANNED(J-34)] `elai bench context-fill` measures per-(model,lane) fill->verified-outcome curves; ModelCapabilityProfile gains fill_degradation_curve / calibrated_trim_fraction / positional_value_curve; context_policy trim_threshold prefers the measured knee over CLOUD_TRIM_FRACTION=0.85 (context_policy.rs:70); E-49's 5-position context_rot_probe extended to a positional-value curve driving 3E slot ORDERING. C6's knobs CONSUME this.
- REGIME-AWARE CONTEXT POLICY [PARTIAL, uncommitted] H-50: ContextPolicy by BackendMode — local = window-minimization + unload-early, cloud = cache-stable prefix + late compaction; fixes flat-100K auto-compact (conversation.rs:22), 64K gate (cli_consts.rs:20). C7's local recipe is the local-compaction leg.
- CONTEXT BUDGET ALLOCATOR [WIRED] 8 slots, 2 cache breakpoints — the chunk-count cap + head/tail placement land as slot policy (phase-3E).
- R4 RIDER (already in the ledger): RULER arXiv:2404.06654 + Chroma context-rot = external grounding for J-34's "almost all models drop as context grows" premise. C6 is the concrete knob set that grounding implies.

## REVISED scope (W-10 tiered)
- SHIP NOW Tier-1 (in-process, deterministic): the chunk-count cap + head/tail placement consuming J-34's positional-value curve (the placement is config over the 3E allocator); the local _compact_history recipe as the H-50 local-lane compaction policy (deterministic summarize-preserving-numerics on the sidecar 32K window).
- DEFER: C8 SproutRAG (cosine-fallback hierarchical index — a later experiment unit); the RLM REPL-as-context bet (recorded rider, not authored).

## Integration
- crates/orchestrator/src/context_calibration.rs (J-34's new file) — C6's chunk-count cap + head/tail placement read the fill_degradation_curve / positional_value_curve (consume J-34, do not re-author the calibration).
- crates/orchestrator/src/context_policy.rs (H-50) — C7's local _compact_history recipe as the local-lane ContextPolicy compaction leg (the cloud lane keeps masking).
- crates/orchestrator/src/context_budget.rs (3E) — head/tail slot ORDERING per the positional-value curve (per-model-static layout = cache-safe, HD-2).
- SSB: PREFER-REUSE J-34 ContextCalibrationEvent (the placement/cap is calibration-driven) + H-50 HandoffLifecycleEvent / the compaction path. net 0 — these are policy knobs over measured curves, not new event types. decide at edit (W-9).
- cross_lang pair: the sidecar 32K window (server.py) for the local compaction trigger — cross_lang_diff if the recipe threads a sidecar field; else none. frontend_pair: the ContextStatePanel absolute counts (J-29) or W-6-justified absence.

## WIRING_CONTRACT
produces_symbols:
- context_calibration.rs consumers: chunk-count cap + head/tail placement reading the measured curves (J-34 extension)
- context_policy.rs local-lane _compact_history recipe (summarize-preserving-numerics on the 32K window; H-50 extension)
- 3E slot ORDERING by positional-value curve (head/tail never middle)
consumes_symbols:
- J-34 context_calibration.rs fill_degradation_curve / positional_value_curve / calibrated_trim_fraction (NOT re-authored)
- H-50 context_policy.rs ContextPolicy-by-BackendMode (the local leg; NOT re-authored)
- 3E context_budget.rs 8-slot allocator + cache breakpoints (NOT re-authored)
reachable_from:
- a real long-session `elai` run (local lane) hitting the 85%-window compaction trigger  [PRODUCTION]
- a real `elai` retrieval-injection path applying the chunk-cap + head/tail placement  [PRODUCTION]
- a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
- NO frontier-model number as an ELAI threshold (ID-5 — consume J-34's MEASURED per-(model,lane) curve)
- NO LLM re-summarization on the CLOUD cache prefix (HD-2 REJECTED; cloud stays deterministic masking — the recipe is LOCAL-lane only)
- NO re-authoring of J-34's calibration or H-50's policy selector (GD-1 — consume them)
- NO middle-position top-chunk placement (lost-in-the-middle; head/tail per the positional-value curve)
- NO new default-OFF ELAI_*_ENABLE without a wire-dark-allowlist row + promotion plan (R9/W-2)
frontend_pair: ContextStatePanel absolute counts (J-29) (decide at edit) or W-6-justified absence.
observable_effect:
- trigger: "a real local-lane `elai` session that grows past the 85% sidecar-window threshold (the local compaction fires); AND a retrieval-injection that exceeds the chunk-count cap (placement applies)"
- expect_ssb: "J-34 ContextCalibrationEvent showing the placement/cap drove off the measured curve; H-50 compaction-path row on the local trigger (preserving intermediate numerics)"
- expect_artifact: "the local session compacts to [system + initial + summary + continue] preserving numerics (never a cloud-prefix re-summarize); the retrieval prompt places top chunks at head/tail and caps the count. NEGATIVE: the CLOUD lane is UNCHANGED (deterministic masking, no LLM re-summarize); a model with no measured curve falls back conservatively (never the 0.85 folklore default once J-34 has measured it)."

## Grounding
- Chroma context-rot (workflow/papers/chroma-context-rot-interface.md, .claude/papers/chroma-context-rot.md): degradation as input grows; RULER-corroborated (R4). DESIGN-rationale; ELAI measures its own curve (ID-5).
- lucek out-of-context + anthropic context-engineering (workflow/papers/lucek-out-of-context-interface.md, anthropic-context-engineering-interface.md): focused 300-token beats 113K full; "lost in the middle" 30%+ middle loss; onset ~5-7K words. ON FRONTIER MODELS — not Qwen-7B (the HARD caveat; measure the own curve).
- RLM arXiv:2512.24601 (workflow/papers/rlm-interface.md, .claude/papers/2512.24601.html): _compact_history at 85%, preserve intermediate numerics, truncate to [system+initial+summary+continue]; verbatim prompt in the interface map. The REPL-as-context architecture is the DEFERRED bet (C7 rider). arXiv ID re-verify at edit.
- C8 SproutRAG (workflow/papers/sproutrag-interface.md): agglomerative max-linkage tree + cosine fallback (ONNX-portable). DEFERRED, lower-ROI.
- ELAI seams: J-34 context_calibration.rs + H-50 context_policy.rs + 3E context_budget.rs (feature map 1.5, codegraph-confirmed 2026-06-23; context-retrieval-research-2026-06-23.md C6/C7/C8).
- LIVE SSB ~269/578 (post-J-66; grep 2026-06-22). re-grep at edit (W-9).

## COMPOSES-WITH
J-34 (the measured curve C6 consumes — NEVER re-authored), H-50 (the local-lane compaction policy C7 extends), 3E context_budget allocator (the slot ordering), J-29 ContextStatePanel (the surface), HD-2 deterministic-masking (the cloud lane stays untouched). Restates none. C8 SproutRAG = recorded deferred rider.

## Open questions / risks
- own-curve measurement is a prerequisite: C6's cap/placement is only as good as J-34's measured curve — if J-34 has not measured the Qwen-7B lane at edit, ship the placement knob with a conservative default and the cap behind J-34's curve.
- local compaction summarizer quality: the local lane summarizing on a 32B/7B model risks losing numerics — the recipe's "preserve intermediate numerics" must be a deterministic extraction, not a free-form local summary; verify on the sidecar before enforcing.
- C8 SproutRAG fit: revisit only after C1-C5 land and only if the flat index shows a measured multi-granularity gap.

End of J-76.
