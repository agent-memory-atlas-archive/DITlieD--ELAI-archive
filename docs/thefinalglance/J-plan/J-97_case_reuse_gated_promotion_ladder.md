# J-97: verified case graph — GATED RETRIEVAL-TIME REUSE + episodic->skill->rule promotion ladder — the RISKY leg, default-OFF, ships LAST behind the execution-grounded reuse-fitness gate

STATUS: PLANNED (WAVE 3 = VCG Slice 2 + Slice 3; BUILD — the iteration-reducer the whole VCG exists for, AND the precise DGM-collapse failure mode the determinism doctrine exists to prevent; DEFAULT-OFF with a wire-dark-allowlist row + promotion plan, OBSERVE-ONLY first, NEVER into the auditor)

- Track J | lever IMPROVES-WITH-USE-SAFELY | grounding STRONG (retrieval stack phase-1B/1C/1G + J-38 + H-43..H-48 + D-6/H-47 + D-13 + I-12 live/planned) | nature BUILD | effort L | WAVE 3 | depends J-96 (the case store + recurrence measure must show a real recurrence-hit-rate FIRST)
- This is the leg that makes the VCG cut iterations: on a new task, retrieve similar PROVEN cases and inject them as demonstrations so the system brings the proven fix forward instead of re-deriving it. It is ALSO the user's exact phrase "it only improves as it gets used" — which, taken naively, is the failure mode H-43..H-48 + the whole determinism doctrine exist to prevent. So the reuse SIGNAL is not "it was retrieved" and not "an LLM said it helped": it is whether reusing the case caused a gate-PROVEN pass on a HELD-OUT recurrence. This file is correct ONLY if that gate is correct.
- BOUND, hard, by SF-1..SF-4 (.claude/rules/self-improvement-fitness.md, K-19) — this is the unit the rule was written to pre-gate: SF-1 execution-grounded-fitness-only (NO byte-hash like the LIVE DGM GAP-1, NO keyword-grep like GAP-2, NO LLM-judged-helpful, NO it-was-retrieved); SF-2 stronger-than-generator cross-family (D-13/VD-4); SF-3 observe-first + wire-dark-allowlist + NEVER into the auditor/critic/adversary (H-31); SF-4 re-derive against current commit+AST, hard-filter superseded, never DELETE (J-21/H-32/G-10).
- Source: self-improving-workflow-canvas-verified-case-graph-2026-06-25.md §3.3 (retrieval+reuse), §3.5 (improve-with-use done as DGM-with-a-real-filter), §3.6 (the experience compression spectrum), §4 (THE CENTRAL RISK — the most valuable part), §6 Slice 2/3. Doctrine: J-0_INDEX.md + K-19.

## THE CENTRAL RISK, stated plainly (§4) — why this leg is gated and last
A self-improving archive whose "usefulness" signal can be authored by the same model that produced the artifact PROVABLY collapses. Evidence the doctrine already banks:
- ELAI's OWN DGM port shipped GAP-1 (derived_alpha = blake3(config_bytes) at darwin_admit_loop.rs — "fitness of BYTES not performance") and GAP-2 (trap detection by keyword grep). Track-7 (H-43..H-48) exists to REPLACE these two live placeholders. Re-introducing a byte-hash or an LLM-judged "this node helped" signal is named in the REJECTED ledger (VD-1/VD-2 / SF-1/SF-2) — do not.
- Misevolve arXiv:2509.26354: refusal 99.4% -> 54.4% AFTER an unfiltered self-evolution loop. F-10 measured security degrading +37.6% over 5 unchecked REFINE iterations.
- The deep-research report Family 5: "without a defensible metric + trace store + regression suite, self-improvement usually becomes self-deception"; LLM judges are instruments to calibrate, not oracles to obey.
THEREFORE the reuse signal is (a) OFF-AGENT + EXECUTION-GROUNDED (did reuse cause a gate-PROVEN pass on a HELD-OUT recurrence — D-6 held-out store + H-47 eval firewall — not "it was retrieved", not "an LLM said it helped"); (b) CROSS-FAMILY VERIFIED (D-13 / VD-4 stronger-than-generator); (c) ABSTENTION-AWARE (RD-1, so "correctly did not apply" is not mistaken for failure).

## SLICE 2 — gated retrieval-time REUSE (§3.3)
- RETRIEVE (deterministic): on a new task, retrieve similar J-96 case nodes over the trigger_signature using the SHIPPED retrieval stack (BM25 + dual-vector + reranker + RRF, phase-1B/1C/1G), optionally the J-38 mid-task callable hybrid_search. No LLM judge selects; the ranker is receipt-aware (I-4).
- INJECT (gated): inject the top-matched case as a DSPy/GEPA-style DEMONSTRATION into the planner/executor context — this is I-12's playbook made RETRIEVAL-TIME and PER-TASK (I-12 promotes to skills; J-97 feeds per-task behind the held-out reuse filter). Placed AFTER the last cache breakpoint (HD-2). NEVER into the auditor/critic/adversary (SF-3/H-31 — the moment the independent check reads "this usually works", the evidence spine that makes minting trustworthy dies).
- STALENESS before reuse (§4 secondary risk, SF-4): re-derive the case against current commit + AST identity (J-21 commit-aware validation, "citation-valid != semantically-valid") BEFORE injecting; H-32 28d TTL on code-grounded facts; G-10 model-drift invalidation; hard-filter superseded (H-31), never the [OUTDATED] narration (the measured Graphiti +68%-token trap).

## SLICE 3a — improve-with-use, done as DGM-with-a-real-filter (§3.5)
A case node's standing rises/falls by USAGE OUTCOME, scored by machinery ELAI already has for exactly this anti-gaming job (ACTIVATE, do not re-author):
- H-43 EXECUTION-GROUNDED FITNESS (pass-rate over frozen tests); H-48 BAI bandit ACTIVATED on the REAL reward; H-45 worst-case / lineage-diverse promotion scoring with PRMBench-calibrated tau.
- RD-1: a node that, when surfaced, leads to a gate-PROVEN pass on a HELD-OUT recurrence gains weight; a node that misleads (reused, then the gate FAILED) loses weight; a node that correctly says "I don't apply here" is NOT punished.
This is the difference between "improves with use" done as DGM-with-a-real-filter vs DGM-with-no-filter (which provably collapses).

## SLICE 3b — the experience compression spectrum: episodic -> skill -> rule (§3.6)
A case node walks the continuum the deep-research report formalizes:
- EPISODIC: one proven fix (J-96 minted it).
- -> SKILL: after I-12's k verified REUSES (each a held-out gate-PROVEN pass), promote to a skill / playbook entry, export to CLAUDE.md / AGENTS.md / a skill file (I-12's existing export). AUTOMATIC at k (B-23 L1 data-table promotion).
- -> RULE: a pattern proven across many cases + symbols can be PROPOSED as a hard rule (a rules-engine row, phase-3F, or a CLAUDE.md line) — HUMAN-GATED only (B-23 L3 structural promotion is human-only, regardless). 
The canvas (J-98) makes this promotion VISIBLE (a node lighting up episodic -> skill -> rule as evidence accrues).

## Existing ground truth (grep-confirmed; re-grep at edit, W-7/W-9)
- retrieval stack phase-1B/1C/1G [WIRED] BM25 + dual-vector + reranker + RRF. J-38 mid-task hybrid_search [PLANNED]. I-4 receipt ladder [DARK]. I-12 playbook curator [PLANNED]. H-43 exec-grounded fitness [PARTIAL] / H-45 / H-48 bandit [PARTIAL]. D-6 held-out store [DARK] + H-47 eval firewall [PARTIAL]. D-13 cross-family adversary [DARK]. H-39 abstention_score.rs [WIRED] (RD-1 bit). H-31 auditor firewall + bi-temporal supersession [PARTIAL]. J-21 commit-aware validation [PLANNED]. live SSB baseline ~268/577.

## Reads (mandatory before coding — W-3)
1. self-improving-workflow-canvas-verified-case-graph-2026-06-25.md §3.3/§3.5/§3.6/§4/§6 Slice 2-3 (reuse + the filter + the spectrum + the central risk).
2. .claude/rules/self-improvement-fitness.md (SF-1..SF-4 — the binding pre-gate, K-19).
3. J-96 (the case store + node schema + recurrence measure this builds on).
4. crates/orchestrator/src/ — the retrieval stack injection seam + H-43/H-45/H-48 fitness + D-6 held-out + H-47 firewall + D-13 adversary + H-39 abstention bit + J-21 validation + the I-12 export path (re-grep at edit, W-7).
5. ssb.rs + action_type_lock.rs(.golden) for the live consts (W-9).

## Tasks (gated reuse + fitness scoring + promotion ladder; all behind ELAI_CASE_REUSE_ENABLE default-OFF)
1. RETRIEVE leg (~70 LOC): trigger_signature -> the shipped retrieval cascade (REUSE phase-1B/1C/1G + optional J-38) -> top-K matched cases, receipt-ranked (I-4), no LLM judge.
2. STALENESS gate (~60 LOC): before injecting, re-derive each candidate against current commit+AST (J-21) + TTL/drift filter (H-32/G-10) + supersession hard-filter (H-31); a stale/superseded case is dropped, never narrated.
3. INJECT leg (~50 LOC, default-OFF ELAI_CASE_REUSE_ENABLE + wire-dark-allowlist row + promotion plan, W-2/R9): inject the surviving top case as a demonstration into the planner/executor prompt, AFTER the last cache breakpoint (HD-2). HARD assertion: the injection path is UNREACHABLE from the auditor/critic/adversary context (SF-3).
4. REUSE-FITNESS scorer (~90 LOC): the off-agent, execution-grounded signal — on a reused case, did it cause a gate-PROVEN pass on a HELD-OUT recurrence (D-6 + H-47)? Feed H-48's bandit + H-45's worst-case/lineage scoring; the RD-1 abstention bit (read from H-39, never re-derived) credits correct-non-application. NO byte-hash, NO LLM-judged-helpful, NO it-was-retrieved (SF-1).
5. CROSS-FAMILY check (~40 LOC): promotion of a case's standing is checked by a DIFFERENT pretraining family than the generator (D-13/VD-4); same-family self-promotion is refused.
6. PROMOTION ladder (~70 LOC): episodic -> (I-12 k verified held-out reuses) -> skill export (I-12's existing path); skill -> rule PROPOSAL is human-gated (B-23 L3), never auto-applied.
7. SSB: +CaseReuseScored Pattern-2 (Kind: ReuseInjected / HeldoutPassCredited / MisleadPenalized / AbstentionCredited / PromotedToSkill) OR prefer-reuse H-48 bandit + I-12 promotion rows; baseline+0..+1, decide at edit (W-9).
8. Decision doc: workflow/reasoning/J-97-decisions.md (the held-out reuse-fitness definition, the cross-family promotion check, why default-OFF + the promotion plan, the SF-3 unreachability assertion point, the human-gate on skill->rule).

## WIRING_CONTRACT
produces_symbols:
  - the retrieve+staleness+inject pipeline (default-OFF ELAI_CASE_REUSE_ENABLE)
  - the off-agent execution-grounded REUSE-FITNESS scorer (held-out gate-PROVEN pass -> H-48 bandit / H-45)
  - the cross-family promotion check (D-13/VD-4)
  - the episodic->skill->rule promotion ladder (skill = automatic at k; rule = human-gated)
  - +CaseReuseScored SSB (baseline+0..+1) OR prefer-reuse H-48/I-12 rows
consumes_symbols:
  - J-96 case store + trigger_signature + recurrence measure (the store this reuses)
  - the retrieval stack phase-1B/1C/1G + J-38 (REUSE the cascade, no new retriever)
  - D-6 held-out store + H-47 eval firewall (the held-out recurrence the fitness is measured on)
  - H-43/H-45/H-48 fitness/bandit + H-39 abstention bit (RD-1) + D-13 cross-family
  - J-21 commit-aware validation + H-32 TTL + G-10 drift + H-31 supersession/firewall
  - the I-12 playbook export path (the skill promotion target) + the planner/executor prompt seam (the inject target)
reachable_from:
  - a live `elai` run on a task whose failure matches a case trigger_signature, with ELAI_CASE_REUSE_ENABLE=1, injects the demonstration [PRODUCTION]
  - the reuse-fitness scorer fires off-agent after the reused run's gate verdict on a held-out recurrence [PRODUCTION]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies reuse reachability (R7); the scorer is off-agent, never the generating run
stub_ban:
  - NO byte-hash / config-bytes fitness (SF-1; the LIVE DGM GAP-1 anti-pattern), NO keyword-grep (SF-2; GAP-2), NO LLM-judged-helpful, NO "it-was-retrieved/used" as fitness (SF-1)
  - NO reuse-fitness from the SAME family / the GENERATING agent (SF-2/D-13 cross-family; the held-out gate is off-agent)
  - NO case/memory injected into the auditor/critic/adversary context (SF-3/H-31 — the firewall is the load-bearing reason minting stays trustworthy)
  - NO reuse of a stale/superseded case (SF-4 — re-derive against current commit, hard-filter, never DELETE, never [OUTDATED] narration)
  - NO reuse-into-prompt default-ON (W-2/R9 — default-OFF + wire-dark-allowlist row + promotion plan; observe recurrence-hit-rate from J-96 FIRST)
  - NO auto skill->rule promotion (B-23 L3 human-gated); NO .unwrap()/.expect() prod; blake3 only
frontend_pair: reuse-injection + fitness + promotion-ladder state surface on the J-98 case altitude (decide at edit, W-4) or W-6-justified.
observable_effect:
  trigger: "with ELAI_CASE_REUSE_ENABLE=1: a new failing task matching a minted case's trigger_signature; then a HELD-OUT recurrence of that bug class to score the reuse"
  expect_ssb: "ReuseInjected (the demonstration entered the planner/executor prompt, recomputable GD-2); HeldoutPassCredited if the reused case led to a gate-PROVEN pass on the held-out recurrence; MisleadPenalized if it led to a gate FAIL; AbstentionCredited if the case correctly did not apply"
  expect_artifact: "the reused case visibly cut iterations on the matching task (fewer turns to green vs the --no-reuse control); the case's bandit weight rose on a held-out pass. NEGATIVE (R2): the --no-reuse control arm re-derives the fix from scratch (reuse is load-bearing); a case injected into a fixture AUDITOR context FAILS the SF-3 firewall assertion; a byte-hash/LLM-judged-helpful fitness path FAILS the SF-1 contract; a stale case (commit moved) is dropped before injection (SF-4); reuse with the flag OFF injects NOTHING (default-OFF proven)."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-97_case_reuse_gated_promotion_ladder.md --run-trigger
python .claude/tools/production_caller_check.py --files .elai_cc/crates/orchestrator/src/case_graph/reuse.rs --json
rg -n "ELAI_CASE_REUSE_ENABLE" .elai_cc/wire-dark-allowlist.toml                 # default-OFF flag registered WITH a promotion plan (W-2/R9)
rg -n "auditor|critic|adversary" .elai_cc/crates/orchestrator/src/case_graph/reuse.rs  # the SF-3 unreachability assertion present
rg -ni "blake3\(config|keyword.?grep|llm.?judg|helpful" .elai_cc/crates/orchestrator/src/case_graph/  # MUST be empty (SF-1/SF-2 anti-patterns banned)
test -f workflow/reasoning/J-97-decisions.md
```

## ACCEPTANCE
- [ ] reuse ships DEFAULT-OFF (ELAI_CASE_REUSE_ENABLE) with a wire-dark-allowlist row + promotion plan; observe J-96's recurrence-hit-rate FIRST (GD-4).
- [ ] the reuse-fitness signal is off-agent + execution-grounded (held-out gate-PROVEN pass, D-6/H-47), cross-family (D-13/VD-4), abstention-aware (RD-1); NO byte-hash / LLM-judged-helpful / it-was-retrieved (SF-1/SF-2).
- [ ] the VCG is NEVER injected into the auditor/critic/adversary (SF-3 firewall assertion proven).
- [ ] staleness: a case is re-derived against current commit+AST before reuse; superseded hard-filtered, never DELETEd / never [OUTDATED]-narrated (SF-4).
- [ ] promotion: episodic->skill automatic at k held-out verified reuses (I-12 export); skill->rule HUMAN-gated (B-23 L3).
- [ ] SSB discipline complete (or prefer-reuse H-48/I-12 justified); baseline re-read at edit (W-9). cargo check GREEN; auditor PASS. No new dependency.

## Tests to add
- crates/contracts/tests/j97_case_reuse_contract.rs (~160 LOC): (a) with the flag ON, a matching case injects a demonstration and cuts iterations vs the --no-reuse control (R2 load-bearing arm); (b) flag OFF injects nothing (default-OFF arm); (c) the reuse-fitness scorer credits a held-out gate-PROVEN pass and penalizes a mislead, off-agent (D-6/H-47); (d) the SF-3 firewall arm: VCG into an auditor context FAILS; (e) the SF-1 arm: a byte-hash/LLM-judged fitness path FAILS the contract; (f) the SF-4 arm: a commit-moved case is dropped before injection; (g) RD-1: a correct-non-application is NOT penalized. traced-test asserts on the journaled ReuseInjected row (logs_contain inside assert!, T-1).

## Files touched
- CREATE: crates/orchestrator/src/case_graph/{reuse,fitness,promote}.rs (~320 LOC)
- MODIFY: the planner/executor prompt seam (gated inject, after the last cache breakpoint) + the H-48 bandit / H-45 scoring feed + the I-12 export path (skill promotion)
- MODIFY: ssb.rs + action_type_lock.rs(.golden) if +1 variant; .elai_cc/wire-dark-allowlist.toml (+ELAI_CASE_REUSE_ENABLE row with promotion plan)
- CREATE: workflow/reasoning/J-97-decisions.md

## Citations / Grounding
- self-improving-workflow-canvas-verified-case-graph-2026-06-25.md §3.3/§3.5/§3.6/§4/§6 Slice 2-3. DGM arXiv:2505.22954 (the archive is only as good as the empirical filter); Misevolve arXiv:2509.26354 (refusal 99.4->54.4% post unfiltered evolution); MPBench arXiv:2606.04329 (write-time scanning misses benign-at-write poison = SF-4); auditor-memory-firewall arXiv:2510.09905 (memory into the judge kills independence = SF-3) — all EXTERNAL, grounding for WHY the gate exists, never a claimed ELAI number.
- ELAI ground truth: retrieval phase-1B/1C/1G, J-38, I-4, I-12, H-43/H-45/H-48, D-6/H-47, D-13, H-39, J-21/H-32/G-10/H-31 per the feature map 1.1/1.2/1.5/1.7 (verify at edit, W-7). live ~268/577 (W-9).
- Hard rules invoked: SF-1..SF-4 (the whole point), VD-1/VD-2/VD-4, RD-1, GD-4 (observe-first), W-2/R9 (no default-OFF graveyard — flag carries a promotion plan), TD-3, HD-2, H-31 firewall, blake3-not-blake2, W-7/W-9.
- K-18 banks the reuse-ON iteration/$ delta as BENCH-PENDING on THIS J-track; K-19 is the SF-1..SF-4 pre-gate this file is the primary subject of.

## Open questions / risks
- promotion authority (run §8, carry to the user): is episodic->skill fully automatic at k verified reuses (I-12), or does the user ratify each skill graduation? skill->rule is human-only regardless (B-23 L3). v1 = automatic-at-k for skill, human-ratify a switch available; record the choice.
- held-out recurrence supply: the reuse-fitness gate needs HELD-OUT recurrences of a bug class to score against; on a cold graph there are few. v1 scores opportunistically (only when a held-out recurrence naturally occurs) and the bandit stays OBSERVE-ONLY until enough samples; never fabricate a recurrence.
- the cold-start / self-reinforcement trap: a case reused early could bias toward itself before enough held-out evidence exists. The off-agent held-out gate + cross-family check + observe-first default-OFF are the three guards; if the bandit weight diverges from the held-out pass-rate, that is the falsifier — STOP and surface (anti-spiral).

## Next
Adds the iteration-reducing reuse leg behind section-4's filter — the only place the user's "improves as it gets used" is realized, and only as DGM-with-a-real-filter. J-98 makes the whole graph (mint from J-96, reuse + promotion from J-97) a visible canvas MEMORY/CASE altitude. The safe order is strict: J-96 mint+measure -> prove recurrence-hit-rate -> THEN J-97 gated reuse -> THEN J-98 visibility.

End of J-97.
