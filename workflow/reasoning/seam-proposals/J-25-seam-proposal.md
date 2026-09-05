# J-25 seam proposal — Task T7 relative-floor retrieval mechanics (K-27 design turn, DOC-ONLY)

Task: opuswalk-J-25-PLAN-1787802769-1e1874 (wave PLAN, role PLAN, idem 348b8e2621be901b8a4d0eb9d67660b6).
Author: dispatched design worker under agent-dispatch-preamble R-1..R-8 and the synthesizer-elai role
(aggregate, do not implement). This doc does NOT self-certify DONE — the off-agent verifier and the
opus controller own the verdict. No product code was edited; git was touched read-only; writes are
confined to this file per allowed_write_set. All file:line below were re-verified TODAY, 2026-08-27,
against `/home/user/teikoku/elai`, HEAD `b9efaf5e` ("l14 task-3: wire the l11 output-quality gate
into the build front door"). Records carried from walk-state/evidence files are cited as such.

## 1. Queue reason + gate_json_ref

Escalation queue entry (walk-state.json `escalation_queue[16]`, ts 2026-07-20T20:02:03Z):

> reason: T7 SET_ASIDE K-27: relative-floor needs a similarity score axis that does not exist
> (FactStore has only MINJA trust floors + recency sort; ATTACK-3 vs MINJA fork). Fingerprint
> reconcile has no file-index host in crates/memory. Seam-design proposal at
> conductor/opus_walk/evidence/J-25/T7-seam-design-proposal.md awaits owner ratify. Executable legs
> T1-T6 already LANDED. Do not re-open T1..T6. Do not improvise BM25/FTS or file-index substrate.
> gate_json_ref: conductor/opus_walk/evidence/J-25/S2-integrate-T7.md

The gate_json_ref file EXISTS (verified today; bottom lines name two HIGH gaps: no score axis in
FactStore — T7-a2 presumes one; no file-index host — T7-b is greenfield; plus HIGH drift: ATTACK-3
is self-contradictory because the existing absolute floors ARE the MINJA trust gate).

Decision records bound to this unit:
- D-167 (2026-07-20): T7 SET_ASIDE (K-27); owner must ratify T7-seam-design-proposal.md
  (recommended FTS5/BM25 + defer fingerprint).
- D-328 (2026-08-11): T7 seam design REVIEWED and left DEFER (not ratified). The prior proposal
  does not resolve to one buildable slice — forks a2 OPTION A BM25/FTS5 vs B cosine vs C defer,
  b OPTION A memory_file_index table vs B defer; only the T7-a1 over-fetch-cap precursor is
  EXECUTE-ready alone and shipping it alone does NOT satisfy the WIRING_CONTRACT.
  Every non-defer option adds retrieval SUBSTRATE to crates/memory, which the queue entry bans;
  picking the axis is a durable architecture choice under GO-4 (owner), not orchestrator-ratifiable.
- D-385 (2026-08-11): design-lane drain scan classifies "J-25 T7 proposal awaiting OWNER ratify"
  as already-designed-and-gated; re-designing it is directive-banned waste.

## 2. What the plan demands of T7

docs/thefinalglance/J-plan/J-25_mimocode_context_memory_mechanics.md:

- Line 50-54: "(a) relative score floor = 0.15 * topScore (corpus-size-stable, replaces absolute
  floors), 3x over-fetch cap 50; (b) file fingerprint = '{size}-{mtimeMs}' upsert-on-change
  reconcile with prune-on-delete. Lands in crates/memory retrieval (E-5 dark retrieval leg / H-33
  hub) and is cited as a mechanics input to J-21's retrieval-time leg."
- Line 76-77 (REVISED scope): "T7 lands with the E-5 retrieval un-dark or J-21, whichever executes
  first." Tier-2 DEFERRED.
- Line 95 (WIRING_CONTRACT produces_symbols): "crates/memory relative-floor scoring + fingerprint
  reconcile" — one CONJUNCTIVE symbol entry.
- Line 214-215 (ATTACK-3): "absolute score floor reintroduced in T7 -> REJECT. T7 uses
  0.15 * topScore (relative), corpus-size-stable. An absolute floor is the bug T7 exists to remove."
- Line 155 (ABORT-2): any LLM call in T2/T3/T4/T6/T7 -> STOP (deterministic mechanics only).
- Source ground truth (W-3): MiMo-Code clone `docs/.source/MiMo-Code/packages/opencode/src/memory/`
  — service.ts implements search via SQLite FTS5 `bm25()` scoring (:100-110), OR-joined token query
  from fts-query.ts, over-fetch `min(limit*3, 50)` (:114), relative floor `topScore *
  (memory_search_score_floor ?? 0.15)` keeping hit #1 always (:78-80, :127-130), and separate
  reconcile.ts. The T7 mechanics are faithful ports of an FTS5 substrate.

## 3. Verified seams on current main (each pinned by direct read/grep today)

Path prefix omitted: all paths relative to `.elai_cc`.

### 3.1 The FactStore read path T7 would modify — currently scoreless

`.elai_cc/crates/memory/src/retrieval.rs`:

| line | content |
| --- | --- |
| :104 | `pub fn get_relevant_facts(store, user_id, filter: &RetrievalFilter, k, now_ms)` — NO query-text / similarity input of any kind |
| :167-173 | MINJA trust-floor hard filter: `f.trust_score >= threshold`, threshold = `trust_override.unwrap_or_else(\|\|f.fact_type.retrieval_threshold())`; MINJA citation arXiv:2601.05504 at :100 |
| :174-175 | provenance filter (non-empty evidence_pointers) |
| :190-191 | sole ranking axis today: recency — `sort_by(b.last_observed_at_ms.cmp(&a.last_observed_at_ms))` then `truncate(k)` |

There is no bm25/cosine/score anywhere on this path — confirming the queue reason's central claim.

### 3.2 The absolute floors that ARE the MINJA gate (the ATTACK-3 collision)

`.elai_cc/crates/memory/src/fact.rs`:

| line | content |
| --- | --- |
| :79-81 | doc "Retrieval trust-score threshold per the E-5 report Task 7"; `pub fn retrieval_threshold(&self) -> f32` |
| :82-91 | Opinion=0.7; Preference/VoiceStyle/Expertise/RiskTol/DomainKnowledge/SessionPattern=0.5; Mistake=0.4 |
| :308-314 | test `fact_type_retrieval_thresholds` FREEZES these values |

Write-side defense at `.elai_cc/crates/memory/src/defense.rs` (:1, :21, :89 MINJA rule-gate);
fixture `.elai_cc/crates/memory/tests/prompt_injection_fixture.rs` exercises high-trust rows through
get_relevant_facts. Deleting/replacing these floors (plan line 51 says relative floor "replaces
absolute floors") is exactly what S2-integrate-T7.md flags as the unresolved self-contradiction.

### 3.3 Production reachability today (thin, consistent with "E-5 dark retrieval leg")

Callers of `get_relevant_facts` found by grep today:
- `.elai_cc/crates/memory/src/retrieval_authorization.rs:413` inside `authorize_for_injection`
  (:336, the J-21 Step-1 composition calling `revalidate_semantic` then H-32 revalidation) — but
  grep finds NO caller of authorize_for_injection outside its own tests.
- `.elai_cc/crates/orchestrator/src/benchmarks/staleness_runner.rs:439` (bench harness).
- memory/tests fixtures (`extract_and_retrieve_smoke.rs:76,158`, `prompt_injection_fixture.rs:83,103`).
No user-facing entrypoint consumes ranked facts; notes-path scoring exists separately as
`hippo_rag::retrieve_notes` (`hippo_rag.rs:57`, entity-keyed deterministic PPR, called from
`.elai_cc/crates/memory/src/evolution.rs:216`) — different store (MemoryNote), not FactStore facts,
and itself without a query-similarity axis. So BOTH operands of T7-a lack a home: no score axis AND
no live ranked-retrieval consumer chain to falsify through.

## 4. Why DEFER and not the alternatives

- Not BUILD-WITH-SEAM: every executable variant adds retrieval SUBSTRATE to crates/memory
  (FTS5 index, cosine axis, or file-index table). The queue record bans improvising exactly that
  ("Do not improvise BM25/FTS or file-index substrate"), D-328 ruled the axis pick owner-level under
  GO-4 after REVIEWING the prior proposal, and D-385 bans burning paid turns to re-design a unit
  that already holds a reviewed proposal. Re-running K-27 to author a second, rival design (or
  ratifying one myself) would bypass recorded governance — the material-fork case where the worker
  hands off instead of choosing (R-4).
- Not SUPERSEDED-BY: no shipping unit owns the axis. plan-index.json states verified today:
  E-5 partial, J-21 partial (semantic_validation.rs + citation_validity.rs ship, but the
  retrieval-time ranking leg is unowned); the retrieval-stack neighbors are ALL dark —
  J-38 agentic retrieval loop, J-72 in-domain retriever adaptation, J-73 retrieval-abstention gate,
  J-74 graph-as-retrieval channel, J-75 RRF ablation harness, J-105 memory-ROI attribution.

## 5. Conditional slice sketch (NOT part of the verdict; activates only on owner ratification)

If the owner returns ONE sentence selecting a2-OPTION-A (FTS5/BM25, MiMo-faithful, the recommendation
in D-167), the landing shape is:

1. `.elai_cc/crates/memory/src/store.rs`: new FTS5 virtual table + index maintenance beside the
   existing SQLite migrations; fact insert/update feeds it (substrate ~60 LOC).
2. `retrieval.rs`: extend `RetrievalFilter` with optional `query: Option<String>`; when present,
   rank by bm25(-score) after the MINJA/provenance filters (:167-175 UNTOUCHED — resolves the
   ATTACK-3-vs-MINJA contradiction by making the relative floor a RANK cutoff, never replacing the
   trust gate); keep #1 always; cutoff 0.15*topScore; over-fetch min(4k,50) caps the same way
   MiMo service.ts:114 does; recency becomes tiebreak behind score (:190).
3. Wire + smoke: first live consumer through `retrieval_authorization.rs::authorize_for_injection`
   or the CLI memory front door (H-33 `memory_cmd.rs`), with production_caller_check clean; the
   contract mirror is `.elai_cc/crates/contracts/tests/j25_*_contract.rs` precedent.
   T7-b fingerprint reconcile stays deferred unless a `memory_file_index` host is ratified too
   (prior proposal's b-OPTION-A), since crates/memory provably has no file-index table today.
If the owner instead picks cosine, defer-to-next-substrate (a2-C), or "no axis ever": none of these
touch crates/memory except B, which repeats this analysis modulo the scorer swap (deterministic
embedding source becomes the new blocker given ABORT-2 / zero-local-GPU constraints).

DRIFT (synthesizer duty, surfaced for the controller):
- plan-index.json J-25 row ("state dark", key_files "(planned) context_boundary.rs") is stale:
  context_boundary.rs ships and T1-T6 landed @e275034 per D-159/D-162/D-166 + j25_* contract tests.
  severity LOW-MEDIUM | recommend: refresh index row | flagged by this turn's aggregate read.

VERDICT: DEFER — execution stays gated on the recorded gate: OWNER ratification of the similarity
axis for T7-a under GO-4 (D-328: one sentence selecting BM25/FTS5 vs cosine vs defer) against the
awaiting proposal conductor/opus_walk/evidence/J-25/T7-seam-design-proposal.md, tracked by
escalation_queue entry J-25 ts 2026-07-20T20:02:03Z (gate_json_ref
conductor/opus_walk/evidence/J-25/S2-integrate-T7.md). Until that sentence lands, no seam in
crates/memory may be built for T7: the queue record bans new retrieval substrate, and this turn's
verified reads confirm both blockers still stand (no score axis on get_relevant_facts,
retrieval.rs:104/:190; no file-index host for T7-b).
