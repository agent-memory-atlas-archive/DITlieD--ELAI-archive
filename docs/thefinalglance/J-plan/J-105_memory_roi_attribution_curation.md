# J-105: counterfactual memory-ROI attribution + ROI-driven curation (with G-3 code-graph centrality as a default-OFF observe-only re-rank rung) — which memory entry CAUSED this outcome, drive trim/demote off execution-grounded evidence, an EXTENSION of J-21/K-18/H-43/H-39

STATUS: PLANNED (G-7 residual net-new: per-entry CAUSAL ROI attribution at retrieval grain, the lever that turns memory from net-NEGATIVE to +upside; G-3 centrality FOLDED in DOWNGRADED to a default-OFF observe-only rung per the research; EXTENDS J-21 — does NOT re-author its poisoning/validation half; both flags default-OFF, the centrality one stays observe-only longest)

- Track J | lever MEMORY-THAT-PAYS-ITS-RENT (trim the entries that caused failures, keep the ones that caused successes, off EXECUTION evidence never self-confidence) | grounding STRONG (J-21 + crates/memory decay.rs/decay_scheduler.rs/retrieval.rs + C-45 credit + H-43 fitness + H-39 abstention + PageRank phase-1D + B-29 fan-in code-grounded this session; re-grep at edit W-7) / EXTERNAL (Trajectory-Informed/Memory-R1/Xiong/Ahad/Mem0, design-rationale only) | nature BUILD (EXTENSION of J-21) | effort M | WAVE 2 (after J-21 ships its quarantine/validation half)
- Source: workflow/reasoning/jplan-sdl-context-memory-readaccess-2026-06-26.md §3 (J-105) + the gap file G-7/G-3 + Downloads/deep-research-report(11).md §1 (centrality marginal) / §2 (curated-memory upside + naive-accumulation degradation + poisoning + counterfactual attribution). Doctrine: J-0_INDEX.md + the reasoning file §0 universal block.
- NEW FILE EXTENDING J-21 (house rule: never amend the already-PLANNED J-21 in place). J-21 owns the poisoning/quarantine + commit-aware semantic validation + retrieval-time authorization + TTL->activation-decay; J-105 ADDS the per-entry causal ROI signal that DRIVES J-21's decay, and consumes J-21's defense. NO sdl-mcp code copied (centrality ranking is design rationale only; license).

## Context
THE G-7 GAP (residual net-new): ELAI measures memory value COARSELY — K-18 recurrence-hit-rate (how often a new failing task matches an existing proven case), H-43 execution-grounded fitness, H-39 abstention scoring. J-21 (PLANNED) defends memory (quarantine + commit-aware validation + retrieval-time auth + TTL->activation-decay). What is MISSING is PER-MEMORY-ENTRY CAUSAL ROI attribution at retrieval grain: which specific memory entry CAUSED this session's success or failure — used to TRIM/DEMOTE negative-ROI entries and PERSIST proven-positive ones. The research makes this load-bearing, not a nicety:
- UPSIDE (report(11) §2): curated/selective memory lifts +14.3pp goal-completion / +28.5pp (149% rel) on hard multi-step tasks (Trajectory-Informed); Memory-R1 +30% on LoCoMo.
- THE TRAP (the same source): NAIVE add-all accumulation FLATTENS / DEGRADES vs a fixed memory (Xiong 2025); quality-based deletion kept retained memories 72% vs 55% correct => ~20% of unfiltered records are noisy enough to MISLEAD. Memory-poisoning succeeds 95% (Schneider 2026). Counterfactual Composition Testing identifies the single causal entry at 87.5% (Ahad 2026).
=> the attribution mechanism is the lever that turns memory from net-negative to +upside; it MUST drive trimming, and it MUST be defended (J-21 covers the poisoning half; J-105 consumes it).

THE G-3 GAP (downgraded by the research to observe-only): sdl-mcp ranks surfaced memories by code-graph CENTRALITY of the symbols they touch (PageRank / fan-in). ELAI ranks by trust-tier + recency + supersession (H-31) + credit-attribution (C-45), never by code-graph centrality. BUT report(11) §1 measures centrality MARGINAL: Mem0 GraphMemory +2% relative (LoCoMo 90->92); Codebase-Memory AST graph 83% vs 92% answer quality at ~10x token saving, "matches or exceeds" text search ONLY on 19/31 languages for hub-detection/caller-fan-in (STRUCTURAL queries); NO study shows structural ranking of EPISODIC memories OUTRANKS semantic+recency on a general benchmark. => centrality is NOT a default ranker. J-105 banks it HONESTLY: a default-OFF, observe-only re-rank rung applied ONLY to queries classified graph-shaped/architectural. This is grounded-by-default in action — marginal evidence does not get acted on as if strong.

THE AUDITOR-OBJECTIVITY FIREWALL (carried from H-31, arXiv:2510.09905, NON-NEGOTIABLE): auditor / critic / adversary roles get ZERO memory injection. Both the ROI attribution AND the centrality re-ranking MUST respect this firewall — ROI-ranked memory is NEVER surfaced to objectivity roles (it would re-couple the independence the firewall exists to protect).

## Existing state (what J-105 extends / consumes — do NOT re-author, GD-1)
- MEMORY SEMANTIC-VALIDATION + POISONING HARDENING [PLANNED(J-21)] commit-aware/state-level validation + retrieval-time authorization + typed claim/evidence + write quarantine + provenance-linked raw-evidence + TTL->activation-decay; +MemoryValidationEvent. J-105 DEPENDS on J-21 (consumes its quarantine + the decay path the ROI score drives) and EXTENDS it. NEVER re-author J-21's poisoning half here.
- USER MEMORY SUBSTRATE (crates/memory) [PARTIAL] E-5: store.rs / mem0_dispatcher.rs / retrieval.rs / fact.rs / decay.rs / decay_scheduler.rs / contradiction.rs / citation_validity.rs — the substrate. retrieval.rs = the E-5 retrieval-into-prompt path (the centrality rung rides it); decay.rs + decay_scheduler.rs = the J-21 TTL->activation-decay path the ROI score drives.
- ABSTENTION-AWARE SCORING [PARTIAL] H-39, correct=+1/abstain=0/confident-wrong=-t/(1-t); SSB AbstentionScored — the scoring SHAPE the ROI score borrows (RD-1: the abstain bit READ from D-16, never re-derived).
- DGM EXECUTION-GROUNDED FITNESS [PARTIAL] H-43 (CodeRM pass-rate over frozen tests), SSB FitnessEvaluated — the OUTCOME signal the counterfactual attribution uses (VD-1 execution-grounded, never self-confidence).
- CASE-REUSE ECONOMICS / RECURRENCE-HIT-RATE [PLANNED(K-18)] the reuse-value signal — composes as the per-entry reuse-frequency input.
- CASE-BASED CALIBRATION / CREDIT ATTRIBUTION [DARK] C-45/C-47 OrchestratorCaseBaseSsbEmitter + [agent.credit_attribution] config — the per-entry CREDIT substrate the attribution extends (un-darks one consumer of it).
- TIER-B PREFERENCE MEMORY + AUDITOR FIREWALL [PARTIAL] H-31 retrieval HARD-FILTERS superseded rows, AUDITOR/CRITIC/ADVERSARY get ZERO memory injection (arXiv:2510.09905) — the firewall both new legs respect. H-32 (code-grounded fact, 28d TTL) + H-33 MemoryGovernanceEvent (the trim/demote SSB row to prefer-reuse).
- PAGERANK REPO MAP [WIRED] phase-1D + B-29 fan-in (impact API) — the code-graph centrality SIGNAL the observe-only rung adds.
- MODE-ROUTING PREDICATE [WIRED] D-10 select_grounding_mode (deterministic, LLM no input) — the pattern reused to detect a graph-shaped/architectural query (the only condition the centrality rung fires under).
- live SSB baseline ~268/577 (re-grep at edit, W-9); J-105 prefers-reuse H-33 MemoryGovernanceEvent / J-21 MemoryValidationEvent for trim/demote (payload NEVER carries memory content); +MemoryRoiAttributed ONLY if a journaled per-entry ROI row is wanted (net 0..+1).

## Reads (mandatory before coding)
1. workflow/reasoning/jplan-sdl-context-memory-readaccess-2026-06-26.md §3 IN FULL (the ROI attribution, the centrality rung, the firewall, the flags).
2. Downloads/deep-research-report(11).md §1 (centrality MARGINAL: Mem0 +2%, Codebase-Memory 83 vs 92) + §2 (curated upside +28.5pp / Memory-R1 +30%; Xiong naive-accumulation degradation 72 vs 55; Schneider poisoning 95%; Ahad counterfactual 87.5%) — DESIGN RATIONALE ONLY, NEVER a done-gate number.
3. docs/thefinalglance/J-plan/J-21_*.md IN FULL (the quarantine + commit-aware validation + the TTL->activation-decay path J-105 drives — DEPENDED-ON, not re-authored).
4. crates/memory/src/{decay.rs,decay_scheduler.rs,retrieval.rs,store.rs,fact.rs} (the decay path the ROI score drives + the retrieval path the centrality rung rides) — re-grep live at edit (W-7).
5. H-43 fitness + H-39 abstention_score.rs + C-45 credit_attribution + D-10 select_grounding_mode + PageRank phase-1D + B-29 fan-in + the H-31 auditor firewall.

## Tasks (WAVE 2 — after J-21's defense half; both flags default-OFF; centrality observe-only longest)
1. [ROI ATTRIBUTION] an OFF-AGENT, EXECUTION-GROUNDED counterfactual attribution scorer: for a finished session (success OR failure), attribute the outcome delta to the specific retrieved memory entry/entries (counterfactual-composition style: would the GATE-MEASURED outcome have flipped WITHOUT entry E?). Outcome signal = H-43 execution-grounded fitness (VD-1, never self-confidence); per-entry credit substrate = C-45; reuse-value = K-18 recurrence-hit-rate; scoring shape = H-39 (correct=+1/abstain=0/confident-wrong=-t/(1-t), abstain bit READ from D-16 RD-1). TD-3: NO LLM-as-judge scores its own memory ROI. PRODUCER: crates/memory/src/roi_attribution.rs (new) or an orchestrator-side scorer over the sealed session.
2. [ROI -> DECAY DRIVER] the attribution score DRIVES J-21's TTL->activation-decay: negative-ROI entries demote/trim FASTER; proven-positive entries persist. CONSUMER: the J-21 / decay_scheduler.rs curation path consumes the ROI score (trim/demote). Reuse H-33 MemoryGovernanceEvent / J-21 MemoryValidationEvent for the decision row (payload never carries memory content).
3. [CENTRALITY RE-RANK RUNG, observe-only] a default-OFF re-ranking rung in the E-5 retrieval path (retrieval.rs / H-31 ranking) that ADDS code-graph centrality (PageRank phase-1D + B-29 fan-in) as a TIEBREAKER — APPLIED ONLY when the query is classified graph-shaped/architectural (reuse the D-10 select_grounding_mode deterministic routing to detect the query shape). Observe-only first; NEVER default-ON (the research forbids it). CONSUMER: the memory retrieval entrypoint (gated by the flag AND the query-shape classifier).
4. [FIREWALL] both legs respect the H-31 auditor-objectivity firewall: ROI-ranked + centrality-ranked memory is NEVER surfaced to auditor/critic/adversary roles. A contract test asserts an objectivity role receives ZERO memory injection regardless of either flag.
5. [optional +MemoryRoiAttributed] if a journaled per-entry ROI row is wanted (for replay/audit of WHY an entry was trimmed), add ONE Pattern-2 SSB action_type — full 9-tuple lockstep, TOTAL_VARIANTS from LIVE at edit (W-9), payload carries the entry HASH + ROI score, NEVER the memory content. Otherwise net 0 (ride H-33/J-21 rows).
6. workflow/reasoning/J-105-decisions.md (the SSB decision; the ROI-flip threshold DECISION; the centrality observe-only separation finding).

## WIRING_CONTRACT
produces_symbols:
  - the off-agent execution-grounded counterfactual ROI attribution scorer (per-entry, outcome from H-43, shape from H-39, never self-confidence) [roi_attribution.rs]
  - the ROI -> J-21 TTL/activation-decay driver (negative-ROI entries demote/trim faster; proven-positive persist) [decay_scheduler.rs consumer]
  - the default-OFF, query-shape-gated code-graph centrality re-rank rung (PageRank + B-29 fan-in tiebreaker, graph-shaped queries only) [retrieval.rs consumer]
  - the firewall enforcement (ROI/centrality memory NEVER to auditor/critic/adversary)
  - OPTIONAL +MemoryRoiAttributed (Pattern-2, entry-hash + ROI score, NEVER content) if a journaled ROI row is chosen
consumes_symbols:
  - J-21 quarantine + commit-aware validation + the TTL->activation-decay path (DEPENDED-ON, the ROI drives the decay) — cited, never re-authored
  - H-43 execution-grounded fitness (the outcome signal, VD-1) + H-39 abstention scoring (the shape, RD-1 abstain bit from D-16) + K-18 recurrence-hit-rate (reuse-value) + C-45 credit substrate
  - crates/memory retrieval.rs / decay.rs / decay_scheduler.rs / store.rs (E-5 substrate)
  - PageRank phase-1D + B-29 fan-in (the centrality signal) + D-10 select_grounding_mode (the query-shape classifier, deterministic)
  - the H-31 auditor-objectivity firewall (arXiv:2510.09905) — both legs respect it
  - H-33 MemoryGovernanceEvent / J-21 MemoryValidationEvent (the trim/demote decision row, prefer-reuse, payload never carries content)
reachable_from:
  - a live `elai run` whose outcome depended on a seeded known-bad + known-good memory -> the bad entry's ROI is negative and it is demoted/trimmed; the good entry persists [PRODUCTION, the ROI->decay driver]
  - a live graph-shaped/architectural query with the centrality flag ON -> the centrality tiebreaker re-ranks; with the flag OFF or a non-graph query -> the rung is INERT [PRODUCTION, the observe-only rung]
  - a live auditor/critic/adversary dispatch -> ZERO memory injection regardless of either flag [PRODUCTION, the firewall]
  - a #[cfg(test)]/contracts/ caller NEVER satisfies the attribution, decay, or rung leg (R7)
stub_ban:
  - NO LLM scoring its own memory ROI (TD-3 — the attribution is execution-grounded + deterministic; the research's explicit "LLM-judge masks memory errors" warning)
  - NO self-confidence / "it-was-retrieved" as the ROI signal (VD-1 — execution-grounded gate outcome only)
  - NO centrality rung default-ON (the research forbids it; observe-only, graph-shaped queries only, longest-held flag)
  - NO ROI/centrality memory surfaced to auditor/critic/adversary (the H-31 firewall — non-negotiable)
  - NO memory CONTENT in any SSB payload (entry hash + score only)
  - NO re-authoring J-21's quarantine/validation half (J-105 DEPENDS on it)
  - NO default-ON before the attribution accuracy + the Xiong-degradation fix are bounded on >=3 real runs (GD-4); ELAI_MEMORY_ROI_ATTRIBUTION_ENABLE + ELAI_MEMORY_CENTRALITY_RERANK_ENABLE each need a wire-dark-allowlist row + promotion_plan_id J-105-promotion (W-2/R9)
frontend_pair:
  the trim/demote decision is visible on the H-33 Memory Hub / governance surface (provenance). A dedicated ROI tile is OPTIONAL; decide at edit (W-4) or W-6-justify.
observable_effect:
  trigger: "inject a known-BAD memory + a known-GOOD memory, run a session whose GATE-MEASURED outcome depends on them; separately, a graph-shaped query with the centrality flag ON vs OFF; and an auditor dispatch"
  expect_ssb: "H-33 MemoryGovernanceEvent / J-21 MemoryValidationEvent on the trim/demote (net 0 new action_type); OR if chosen, MemoryRoiAttributed (entry-hash + score, never content) recomputable from the journal (GD-2). bump TOTAL_VARIANTS from LIVE at edit (W-9)."
  expect_artifact: "the bad entry's ROI is NEGATIVE and it is demoted/trimmed; the good entry persists (the ELAI-measured attribution accuracy, NOT Ahad's 87.5%); ROI-driven trim IMPROVES the task outcome vs naive-keep-all (the Xiong degradation reproduced AND fixed); the centrality rung shows NO regression when OFF and is measured SEPARATELY when ON (no banked lift it does not earn); the auditor received ZERO memory injection (the firewall). NEGATIVE: with the centrality flag OFF or a non-graph query, the rung is provably inert."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-105_memory_roi_attribution_curation.md --run-trigger
python .claude/tools/python_wire_check.py --files .elai_cc/crates/memory/src/roi_attribution.rs --json 2>/dev/null || python .claude/tools/production_caller_check.py --files .elai_cc/crates/memory/src/roi_attribution.rs --json   # the scorer has a non-test production caller (the decay driver)
rg -n "roi|attribution|counterfactual" .elai_cc/crates/memory/src/decay_scheduler.rs | rg -v "test|contract"   # the decay path consumes the ROI score in production
rg -n "centrality|pagerank|fan_in|select_grounding_mode|graph_shaped" .elai_cc/crates/memory/src/retrieval.rs   # the gated, query-shape-classified centrality rung
rg -n "auditor|critic|adversary|firewall|zero.*memory" .elai_cc/crates/memory/src | rg -v "test"   # the H-31 firewall enforced on both legs
test -f workflow/reasoning/J-105-decisions.md
```

## ACCEPTANCE
- [ ] on a seeded known-bad + known-good memory fixture, the attribution correctly identifies the CAUSAL entry (ELAI's own accuracy number, NOT Ahad's 87.5%); the bad entry's ROI is negative and it is demoted/trimmed, the good entry persists.
- [ ] ROI-driven trim IMPROVES the task outcome vs naive-keep-all (the Xiong degradation reproduced AND fixed — the ELAI-measured done-gate).
- [ ] the centrality rung shows NO regression when OFF; its lift (if any) is measured SEPARATELY when ON and applied ONLY to graph-shaped queries (no banked lift it does not earn).
- [ ] the attribution is execution-grounded + deterministic; the LLM NEVER scores its own memory ROI (TD-3/VD-1).
- [ ] both legs respect the H-31 auditor-objectivity firewall: ZERO memory injection to auditor/critic/adversary regardless of either flag (arXiv:2510.09905).
- [ ] both flags default-OFF with wire-dark-allowlist rows (promotion_plan_id=J-105-promotion); the centrality flag stays observe-only longest (GD-4); no SSB payload carries memory content.
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-memory + -p elai-orchestrator (+contracts); action_type_lock GREEN if +MemoryRoiAttributed. No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: done = a live session's bad memory is attributed negative-ROI and trimmed, observed end-to-end — not "the scorer exists". W-2: both flags get wire-dark-allowlist rows + promotion_plan_id J-105-promotion. W-3: EXTENSION of J-21 (consumes its decay path), not a port.
- W-5: trigger = a live `elai run` with seeded good/bad memory + a graph-shaped query + an auditor dispatch. W-7: J-21 decay path + crates/memory + PageRank + D-10 + the firewall re-grepped at edit. W-9: ~268/577 re-grep; bump only if +MemoryRoiAttributed.
- TD-3/VD-1: execution-grounded deterministic attribution, no LLM-judge. GD-4: observe-only, centrality longest. H-31 firewall: enforced. ID-5: no Mem0/Memory-R1/Ahad number as a gate. R7/R9. GD-2: optional ROI row recomputable.

## Tests to add
- crates/contracts/tests/j105_memory_roi_contract.rs (~150 LOC): (a) a seeded known-bad + known-good fixture proves the attribution identifies the causal entry, the bad entry's ROI is negative + it is demoted/trimmed, the good persists; (b) the Xiong arm: ROI-driven trim improves outcome vs naive-keep-all on a noisy-memory fixture (the load-bearing assert); (c) the centrality-OFF arm: NO regression with the flag off; the centrality-ON arm fires ONLY on a graph-shaped query and is measured separately; (d) the FIREWALL arm: an auditor/critic/adversary dispatch receives ZERO memory injection regardless of either flag; (e) the attribution is deterministic across re-runs under ELAI_SEED (TD-3); (f) no SSB payload carries memory content.

## Files touched
- CREATE: crates/memory/src/roi_attribution.rs (the off-agent execution-grounded counterfactual scorer; exact seams re-grepped W-7)
- MODIFY: crates/memory/src/decay_scheduler.rs (the ROI score drives the J-21 TTL/activation-decay; negative-ROI entries demote/trim faster)
- MODIFY: crates/memory/src/retrieval.rs (the default-OFF, query-shape-gated centrality re-rank rung; PageRank + B-29 fan-in tiebreaker)
- MODIFY (only if +MemoryRoiAttributed): crates/orchestrator/src/ssb.rs + ssb/payloads.rs + action_type.rs + tests/action_type_lock.rs + .golden (full 9-tuple lockstep, TOTAL_VARIANTS from LIVE; entry-hash + score, never content)
- CREATE: workflow/reasoning/J-105-decisions.md
- CONFIG: .elai_cc/wire-dark-allowlist.toml rows for ELAI_MEMORY_ROI_ATTRIBUTION_ENABLE + ELAI_MEMORY_CENTRALITY_RERANK_ENABLE (type=experiment, owner, expiry, promotion_plan_id=J-105-promotion, smoke)

## Citations / Grounding
- workflow/reasoning/jplan-sdl-context-memory-readaccess-2026-06-26.md §3 (the full J-105 spec) + elai-gaps-sdl-ai-setup-2026-06-26.md G-7/G-3.
- Downloads/deep-research-report(11).md §1 (Mem0 GraphMemory +2% LoCoMo 90->92; Codebase-Memory 83 vs 92 at 10x token saving, structural-only, 19/31 langs) + §2 (Trajectory-Informed +14.3pp/+28.5pp; Memory-R1 +30%; Xiong naive-accumulation degradation 72 vs 55 = ~20% noisy; Schneider poisoning 95%; Ahad Counterfactual Composition 87.5%) — ALL [EXTERNAL], design-rationale only, NEVER an ELAI done-gate number (ID-5/GD-1).
- ELAI ground truth (code-grounded this session; re-verify at edit W-7): J-21 [PLANNED, depended-on]; crates/memory store/retrieval/decay/decay_scheduler/mem0_dispatcher/fact [PARTIAL, E-5]; H-43 fitness / H-39 abstention / K-18 recurrence / C-45 credit; PageRank phase-1D + B-29 fan-in [WIRED]; D-10 select_grounding_mode [WIRED]; the H-31 auditor firewall (arXiv:2510.09905). live ~268/577 (W-9).
- Hard rules invoked: TD-3 (deterministic, no LLM scores its own memory); VD-1 (execution-grounded fitness, never self-confidence); RD-1 (abstain bit read from D-16); GD-4 (observe-only, centrality longest); H-31 firewall; ID-5 (no paper number as a gate); GD-2 (recomputable ROI row); W-1/W-2/W-7/W-9; R7/R9. NO sdl-mcp code copied (license).

## Open questions / risks
- ATTRIBUTION COST (highest): a per-entry counterfactual over a real session is expensive if naive. Mitigation = attribute only the RETRIEVED entries (not all of memory), reuse the C-45 credit substrate, and run off-agent on the sealed session (never in the hot path). Recorded in J-105-decisions.md.
- THE ROI-FLIP THRESHOLD: how negative an ROI demotes vs trims is a tunable. DECISION = demote first (recoverable), trim only on a repeated negative across N sessions (the Xiong "20% noisy" target), observe-only until the trim rate is bounded. // DECISION.
- CENTRALITY OVERCLAIM: the marginal evidence means the rung must be measured separately and never bank a lift it does not earn. Mitigation = observe-only longest, graph-shaped queries only, a separate measurement arm.
- FIREWALL leak: any path that surfaces ROI/centrality memory to an objectivity role is a P0. Mitigation = the firewall contract-test arm is mandatory.

## Next
Turns ELAI's memory from coarse-valued (recurrence/fitness/abstention) into per-entry causally-attributed, so the entries that caused failures are trimmed and the ones that caused successes persist — the lever that banks the curated-memory upside without the naive-accumulation degradation. Banks G-3 centrality HONESTLY (observe-only, graph-shaped only). EXTENDS J-21 (consumes its defense + decay), composes K-18/H-43/H-39/C-45/PageRank/B-29/D-10, respects the H-31 firewall. Pairs with J-104 (the centrality rung reads a fresh graph).

End of J-105.
