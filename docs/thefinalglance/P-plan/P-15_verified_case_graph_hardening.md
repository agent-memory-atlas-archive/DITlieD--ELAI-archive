# P-15: verified case-graph hardening (gated agent-s memory loop + journal-row bBoN)

STATUS: PLANNED (WAVE CU-3; EXTENSION of J-96 — J is CLOSED, this lands in P cross-linked; PROOF_SPINE: feeds PS-5; execute after the proof spine per A-7)

- Track COMPUTER-USE-STEAL | lever gate-the-save-not-every-run | grounding STRONG (agent-s KnowledgeBase arXiv:2410.08164 verified; the ungated-save gap is agent-s's own shape; J-96/K-18 PLAN-asserted, executor re-greps W-7) | nature EXTENSION | effort M | WAVE CU-3 | PROOF_SPINE feeds PS-5
- Premise: J-96 mints a case node ONLY on a proven fix; agent-s's S2 KnowledgeBase has the richer retrieve/distill/fuse LOOP but saves EVERY run's summary with NO success gate. Port the loop onto J-96 but GATE the save behind mint-on-proof; add a bBoN behavior-narrative judge run OVER JOURNAL ROWS as an evidence-ranked tie-breaker.

## Context — why this matters
Grounded in workflow/reasoning/canvas-computer-use-eval-2026-07-11.md ENTRY 11 (agent-s, arXiv:2410.08164/2504.00906/2510.02250 verified). agent-s's S2 KnowledgeBase retrieve/distill/fuse loop (narrative + episodic tiers, embedding retrieval, fusion into the next plan) is strong prior art, but S2 saves EVERY run's distilled summary with NO success gate before commit — precisely the unverified-save the J verified case graph exists to close. bBoN (behavior best-of-N) narrates N rollouts and a judge picks the winner — but agent-s narrates from SCREENSHOTS; ELAI has ground-truth action/effect deltas in the JOURNAL, so the narrative judge runs over rows, not pixels (composes I evidence-ranked search: receipts pick first, narrative breaks ties).

## Existing ground truth (B..Q coverage; PLAN-asserted; executor re-greps W-7)
- J-96 verified case graph CORE (J-SUMMARY_INDEX.md:2086): mint-on-proof typed case node + deterministic self-wiring edges + recurrence-hit-rate; a 'memory' CanvasMode. EXTENDED here (the reuse loop) — re-implements nothing.
- K-18 case-reuse economics bench (K-SUMMARY_INDEX.md:465): the per-(bug-class,model) iteration/$ delta + recurrence-hit-rate with reuse ON vs OFF — the PS-5 number this feeds.
- K-19 case-reuse fitness discipline doctrine (K-SUMMARY_INDEX.md:494): the discipline this obeys (gate the save).
- I evidence-ranked search (I-SUMMARY, keystone): best-of-N -> tournament ladder that selects by RECEIPTS not an LLM judge — the bBoN tie-breaker slots in behind it.
- F-4 PROVEN case node / proof receipt: the mint-on-proof gate (K-18 injects an F-4-PROVEN case node as the reuse demonstration).

## Steal/build (REVISED scope, W-10 tiered)
- SHIP NOW (Tier-1): port agent-s's retrieve/distill/fuse loop onto J-96, with the SAVE gated behind J-96's mint-on-proof (execution-grounded) filter; a reuse-hit field on the J-96 case row (net 0..+1); the loop is default-off behind J-96's strict reuse filter.
- DEFER (Tier-2): the bBoN behavior-narrative comparative judge over journal rows (tie-breaker); the episodic/narrative two-tier split; embedding-retrieval tuning.
- EXCLUDED as vaporware: agent-s's UNGATED KnowledgeBase commit (its exact gap — do NOT copy the ungated save); a bBoN judge sourced from screenshots (we have ground-truth journal deltas).

## Reads (mandatory before coding)
- workflow/reasoning/canvas-computer-use-eval-2026-07-11.md (ENTRY 11, steals for agent-s)
- docs/thefinalglance/J-plan/J-96_*.md (the case graph core to extend), K-18/K-19 (the bench + discipline), the I evidence-ranked-search keystone file
- crates/orchestrator/src/ (J-96's case graph module)

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/case_graph/reuse_loop.rs (retrieve/distill/fuse loop gated by mint-on-proof)
  - crates/orchestrator/src/case_graph/bbon_judge.rs (behavior-narrative comparative judge over JOURNAL ROWS, tie-breaker)
consumes_symbols:
  - crates/orchestrator/src/case_graph/mod.rs::CaseNode (J-96; EXTEND with a reuse-hit field, prefer-reuse — net 0..+1)
  - crates/orchestrator/src/case_graph/mod.rs::MintOnProof (J-96; the save gate — do NOT bypass)
reachable_from:
  - crates/orchestrator/src/orchestrator.rs (the reuse path that retrieves a case behind J-96's execution-grounded filter) [PRODUCTION reuse path]
  - a #[cfg(test)] or contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO case node saved from an UNPROVEN run (the save is gated by J-96 mint-on-proof; an ungated save is the agent-s bug)
  - NO bBoN judgment sourced from screenshots (it runs over journal rows — ground-truth deltas)
  - NO reuse-into-prompt unless behind J-96's strict execution-grounded filter (default-off)
  - NO .unwrap()/.expect() on the reuse path; NO blake2/sha2
frontend_pair:
  rides J-96's 'memory' CanvasMode node chrome (reuse-hit surfaced on the case node; W-4).
observable_effect:
  trigger: "elai case reuse --replay tests/fixtures/case/recurrence_bugclass_a.json"
  expect_ssb: "recurrence_hit_rate_measured"
  expect_artifact: "the distill/fuse loop retrieves the case, the SAVE is gated (an unproven run's summary is NOT saved), a reuse-hit field lands on the J-96 case row and K-18's recurrence-hit-rate reflects it; NEGATIVE: a case node saved from an unproven run, or a bBoN judgment sourced from screenshots, FAILS"

## Wargame (plan-standard unit shape)
EXPECTED_OBSERVATION: running a proven fix then a recurrence, the retrieve/distill/fuse loop retrieves the prior case, the save of the new run is GATED (unproven -> not saved), the bBoN judge ranks candidates from journal rows, and a reuse-hit field lands on the J-96 case row that K-18's recurrence-hit-rate reflects.
ON_FAIL: a case node is saved from an unproven run (CAUSE: the ported agent-s loop kept its ungated commit). COUNTER-MOVE: route every save through J-96's MintOnProof gate; assert an unproven-run fixture produces NO new case node; the negative test must FAIL.
FORK: if the recurrence retrieval returns a case that no longer reproduces (the fix rotted), route to re-exploration + re-mint rather than reusing a stale case; record the staleness in the case row.
RECON_NEEDED: confirm agent-s's retrieve/distill/fuse API shape from the cloned source + the exact J-96 CaseNode + MintOnProof symbols to extend. Settling check: source-read agent-s KnowledgeBase + `grep -n "MintOnProof\|struct CaseNode" crates/orchestrator/src/case_graph/`.
ABORT: if reuse-into-prompt would fire WITHOUT J-96's execution-grounded filter (default-off) — a self-improvement loop with no un-gameable stronger-than-generator filter provably collapses (H DGM spine theorem) — STOP and flag.
VERIFY: the load-bearing proof is the off-agent recurrence_hit_rate_measured row (K-18's beacon) + the absence of a case node for an unproven fixture, driven by the real `elai case reuse` entrypoint. Run:
```bash
python .claude/tools/plan_gate_check.py --file docs/thefinalglance/P-plan/P-15_verified_case_graph_hardening.md   # authoring gate (exit 0)
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/P-plan/P-15_verified_case_graph_hardening.md   # execution done-gate
```
REDTEAM: surviving-attack seed — an attacker who wants a poisoned case reused is defeated by mint-on-proof (only an execution-proven fix mints a node) + the strict reuse filter (default-off, execution-grounded). Attack that would succeed if unpatched: a bBoN judge that can be swayed by a fabricated narrative — PATCH: the judge scores journal ROWS (ground-truth action/effect deltas), not model prose; a narrative not backed by rows is discarded. Deep red-team DEFERRED to execution pre-flight.

## INTEGRATION_SMOKE
An independent agent drives `elai case reuse` on a HARD input: a real bug-class that recurs across a multi-turn session, asserting (a) the first PROVEN fix mints a case, (b) an unproven attempt mints nothing, (c) the recurrence reuses the case behind the execution-grounded filter and K-18's recurrence-hit-rate moves. Toy single-fix reuse does NOT satisfy R10. (Shares the CU-track R10 cadence with P-14 as the two PS-5 reuse-artifact minters.)

## Grounding / citations
- Source: workflow/reasoning/canvas-computer-use-eval-2026-07-11.md ENTRY 11 (agent-s; arXiv:2410.08164/2504.00906/2510.02250 verified there).
- ELAI ground truth (PLAN-asserted, executor re-greps W-7): J-96, K-18, K-19, I evidence-ranked search, F-4 per the summary indexes cited.
- CLAUDE.md hard rules: the DGM spine theorem (no un-gameable filter -> collapse); gate the save (K-19); blake3; no .unwrap on prod.

## PRODUCTION_HOOK
crates/orchestrator/src/orchestrator.rs — the reuse path that retrieves a case node behind J-96's execution-grounded filter calls the new retrieve/distill/fuse loop; K-18's `elai bench case-reuse` consumes the reuse-hit field. Wired in this changeset (producer + the reuse consumer + the bench feed together).

## Next
Closes the "our memory is trusted prose vs proof-gated" gap agent-s exposes by absence, and hardens the PS-5 compounding number. Pairs with P-14 (both mint reuse artifacts feeding K-18); cross-links the CLOSED J-96 unit.

End of P-15.
