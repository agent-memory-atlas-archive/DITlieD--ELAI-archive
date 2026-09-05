# J-96: verified case graph — CORE (mint-on-proof typed case node + deterministic self-wiring edges + recurrence-hit-rate) — the SAFE half, observe-only, zero reuse-into-prompt

STATUS: PLANNED (WAVE 1 = VCG Slice 1; BUILD — the unification object + the deterministic edge layer; OBSERVE-ONLY, mint+show, no influence on generation yet; the risky reuse leg is J-97, the canvas altitude is J-98)

- Track J | lever CANT-FORGET-A-PROVEN-FIX | grounding STRONG (F-4/G-2/B-29/failure_taxonomy/I-37/J-10/SessionDag live + grep-verified) | nature BUILD | effort L | WAVE 1 | depends none (rides shipped proof rows; J-10 insight field is the text source — sequence after J-10 leg 1-3 OR ship the insight field independently)
- The Verified Case Graph (VCG) is a SYNTHESIS, not a new algorithm: every ingredient is already specced (J-10 insight, I-12 playbook, C-25 crystallizer, C-45/C-47 case-base, J-31/J-60 canvas, H-43..H-48 anti-gaming fitness), but the pieces live apart and none projects PROVEN FIXES as a reusable, self-wiring memory. J-96 is the ~20% that is genuinely net-new (per the run §5): one typed, journal-grounded store of GATE-PROVEN fixes, wired to each other by DETERMINISTIC evidence edges, with a recurrence measure that proves the graph would be useful WITHOUT yet letting it influence generation. The risky leg (retrieval-time reuse-into-prompt) is deliberately SEPARATE (J-97) and ships LAST behind the execution-grounded reuse-fitness gate — section 4's central risk.
- BOUND by SF-1..SF-4 (self-improvement-fitness doctrine, K-19): even the safe half binds — mint on PROOF not claim (SF-1), never inject the VCG into the auditor/critic/adversary (SF-3), hard-filter superseded + re-derive against current commit (SF-4). The user's exact phrase "it improves as it gets used" is, taken naively, the precise DGM-collapse failure mode Track-7 exists to prevent (Misevolve refusal 99.4->54.4% after unfiltered self-evolution); J-96 mints only, scores nothing by reuse — that is J-97's gated job.
- Source: self-improving-workflow-canvas-verified-case-graph-2026-06-25.md §3.1/§3.2/§3.4 (mint condition, node content, self-wiring) + §6 Slice 1 (mint+show, observe-first) + §5 (net-new vs already-planned). Doctrine: J-0_INDEX.md + .claude/rules/self-improvement-fitness.md (K-19).

## NODE BIRTH CONDITION (§3.1) — a case is MINTED only on gate-PROOF, never on a model's claim
A case node is a DETERMINISTIC projection of proof rows that already land in the journal — no new trust assumption:
- F-4 TEST-INTEGRITY CANT-LIE GATE [WIRED]: the auditor cannot emit passed=true without the orchestrator-captured blake3 stdout_hash matching. A red->green transition with a matching hash IS the birth event.
- G-2 REPRODUCIBLE-RUN ARTIFACT (RRA) [WIRED]: the sealed bundle (SSB stream + seed + model pin + gate verdict hashes + blake3 root) is the node's immutable evidence payload.
- I-37 EARS REQUIREMENT OBJECTS [WIRED] + J-16 BLAST-RADIUS cone: what was REQUIRED + which symbols the fix touched.
Net: minting fires off the AUDITING red->green path with a matching F-4 hash + a sealed RRA; nothing else mints. TD-3: the LLM never decides a node is born.

## NODE CONTENT (§3.2) — a TYPED case, not a blob (so edges/counts are deterministic, not vibes)
- trigger_signature: symptom + failing test id + error taxonomy class (from failure_taxonomy.py's 13-category classifier).
- blast_radius_cone: the DKG symbols the fix touched (from the B-29 impact API).
- fix_delta: the structural edit / SEARCH-REPLACE that took it green.
- proof_receipt: the F-4 stdout_hash + the RRA blake3 root (the immutable evidence).
- provenance_hash: blake3 of the source SSB rows the node was minted from (J-10 ID-3 discipline — a fabricated case is detectable).
- insight: a 1-3 sentence human lesson (J-10 — the LLM writes the TEXT, never the verdict; TD-3).

## SELF-WIRING EDGES (§3.4) — DERIVED deterministically, never LLM-authored (the J-31 doctrine)
Edges carry checker-derived verdicts; the canvas (J-98) re-derives them from the journal, no parallel state. Edge sources, all recomputable functions of journal rows (GD-2):
- shared DKG symbols / blast-radius cone OVERLAP (B-29).
- co-occurrence in the same session / plan-unit (SSB rows).
- same failure-taxonomy class (failure_taxonomy.py).
- supersession: a newer fix on the SAME symbol supersedes an older case (SF-4 — hard-filter at retrieval, never DELETE for replay integrity).
An edge's weight is a recomputable function of journal rows, so the graph is auditable and reproducible, not an opaque learned blob. The LLM NEVER sets an edge or a weight (TD-3).

## THE MEASURE (§6 Slice 1) — recurrence-hit-rate, the proof-of-usefulness WITHOUT influencing generation
On every NEW failing task, compute whether its trigger_signature MATCHES an existing case node (deterministic similarity over the typed signature, not an LLM judge). Emit the hit/miss. recurrence-hit-rate = matched / total. This proves the graph WOULD cut iterations (the system has seen this bug class before) BEFORE any case is allowed to enter a prompt. This is also K-18's `elai bench case-reuse` recurrence-hit-rate arm (SAFE/observe-first now; the reuse-ON iteration/$ delta is BENCH-PENDING on J-97).

## Existing ground truth (grep-confirmed; re-grep at edit, W-7/W-9)
- F-4 cant-lie [WIRED] stdout_hash-bound test outcome. G-2 RRA [WIRED] verify_rra (rra.rs:476). B-29 impact API [WIRED] 4 traversal tools. failure_taxonomy.py [WIRED harness, .claude/tools]. I-37 EARS [WIRED]. J-16 blast-radius [PLANNED] (cone source; fall back to B-29 raw until J-16). J-10 insight field [PLANNED] (the insight text source). C-45/C-47 OrchestratorCaseBase [PARTIAL] (the case-base substrate — REUSE the store shape, do not re-author). SessionDag/H-49 [WIRED/PARTIAL] (the node store). live SSB baseline ~268/577 (re-grep at edit).

## Existing state (why this is BUILD, not ACTIVATION) — §5 anti-double-authoring
- C-45/C-47 case-base: the classic case-based-reasoning SUBSTRATE — J-96 lands the typed proof-case ON it, not a parallel store. REUSE.
- J-10 insight: the case's insight FIELD is J-10's extracted lesson; J-96 hangs it on the case node. REUSE, do not re-extract.
- C-25 template crystallizer [WIRED]: "3 successes -> crystallize a compiled pipeline" — the fast-path COUSIN of the VCG; cite, do not merge (crystallizer hardens a PIPELINE; the VCG remembers a FIX). 
- I-12 evidence-gated playbook: the k-verified PROMOTION target — J-97's job, NOT pre-built here.
- H-43..H-48 anti-gaming fitness: the reuse-SCORING machinery — J-97's job; J-96 mints with zero scoring.

## Reads (mandatory before coding — W-3)
1. self-improving-workflow-canvas-verified-case-graph-2026-06-25.md §3.1/§3.2/§3.4/§5/§6 (the mechanism + what is net-new + the observe-first slice).
2. .claude/rules/self-improvement-fitness.md (SF-1..SF-4 — the binding pre-gate, K-19).
3. crates/orchestrator/src/ — the F-4 cant-lie verdict path + G-2 rra.rs seal + the B-29 impact API + C-45/C-47 OrchestratorCaseBase store + the SessionDag node store + the J-10 insight field (re-grep exact paths at edit, W-7).
4. .claude/tools/failure_taxonomy.py (the trigger_signature taxonomy class source).
5. ssb.rs + action_type_lock.rs(.golden) for the live SCHEMA_VERSION/TOTAL_VARIANTS (re-read, W-9).

## Tasks (mint + edges + measure; ZERO reuse-into-prompt)
1. CaseNode typed struct (~60 LOC): the six typed fields above, persisted as SSB rows over the C-45/C-47 case-base store (the journal is the store; re-derivable, GD-2). blake3 on provenance + proof bytes (never blake2/sha2).
2. MINT trigger (~80 LOC): fire on the AUDITING red->green path WHEN the F-4 stdout_hash matches AND a sealed RRA exists; assemble the typed node from the proof rows + the B-29 cone + the failure_taxonomy class + the J-10 insight. Deterministic; a claim with no matching F-4 hash mints NOTHING (TD-3).
3. EDGE derivation (~90 LOC): a deterministic pass computing the four edge kinds (cone-overlap / co-occurrence / taxonomy-class / supersession) as recomputable journal functions; runs at idle/audit tick, NEVER on a hot path; the LLM sets no edge.
4. RECURRENCE measure (~60 LOC): on every new failing task, deterministic trigger_signature match against existing nodes -> hit/miss + the running recurrence-hit-rate; `elai case-graph stats` reads it.
5. SF-binding guards (~30 LOC): the VCG store is NEVER readable from the auditor/critic/adversary context (SF-3/H-31 firewall — assert at the injection boundary); superseded nodes hard-filtered at any read (SF-4); a node whose proof rows no longer hash-match is flagged STALE (SF-4 / G-10 drift).
6. SSB: +CaseNodeMinted Pattern-2 single-variant (Kind: CaseMinted / EdgeDerived / RecurrenceMatched / CaseStaled), baseline+1 from LIVE ~268/577 — full 6-part variant discipline; re-read consts at edit (W-9).
7. Decision doc: workflow/reasoning/J-96-decisions.md (the trigger_signature similarity function, the four edge-derivation rules, why mint-only/zero-reuse here, the SF-3 firewall assertion point).

## WIRING_CONTRACT
produces_symbols:
  - the CaseNode typed struct + its mint over the C-45/C-47 case-base store (journal-grounded)
  - the deterministic edge-derivation pass (cone-overlap / co-occurrence / taxonomy / supersession)
  - the recurrence-hit-rate measure + `elai case-graph stats`
  - the SF-3 firewall guard (VCG never into auditor/critic/adversary) + the SF-4 supersession/stale filter
  - +CaseNodeMinted SSB (baseline+1)
consumes_symbols:
  - the F-4 cant-lie verdict path (the matching stdout_hash = the birth gate)
  - the G-2 RRA seal (rra.rs verify_rra — the immutable proof payload)
  - the B-29 impact API (the blast_radius_cone)
  - failure_taxonomy.py (the trigger_signature class)
  - the J-10 insight field (the case insight text — REUSE, never re-extract)
  - the C-45/C-47 OrchestratorCaseBase store + the SessionDag node store (REUSE, no new store)
reachable_from:
  - the AUDITING red->green verdict path on a live `elai` run mints a node (NOT a unit test calling mint) [PRODUCTION]
  - a new failing task on a live run fires the recurrence measure [PRODUCTION]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies the mint reachability (R7)
stub_ban:
  - NO mint on a model CLAIM: only an F-4-hash-matching red->green + a sealed RRA mints (TD-3 — the cant-lie gate is the birth gate)
  - NO LLM-authored edge or weight: edges are recomputable journal functions (TD-3); the LLM writes only the insight TEXT (J-10)
  - NO VCG content into the auditor/critic/adversary context (SF-3/H-31 — the firewall is load-bearing)
  - NO reuse of a superseded/stale case (SF-4 — hard-filter, re-derive against current commit, never DELETE)
  - NO reuse-INTO-PROMPT at all in this file (that is J-97, behind the execution-grounded gate; here the graph only mints + shows)
  - NO .unwrap()/.expect() on prod paths; blake3 for provenance/proof (never blake2/sha2)
frontend_pair: recurrence-hit-rate + node count on an existing trust/audit panel (the full case altitude is J-98; decide at edit, W-4) or W-6-justified.
observable_effect:
  trigger: "elai run over a fixture with a red->green fix that PASSES F-4 (matching stdout_hash) + seals an RRA; then a SECOND fixture whose failure matches the first's trigger_signature"
  expect_ssb: "CaseNodeMinted(CaseMinted) on the first, recomputable from the journal under ELAI_SEED (GD-2); EdgeDerived rows wiring it to any cone-overlapping / same-taxonomy prior case; RecurrenceMatched on the second"
  expect_artifact: "`elai case-graph stats` shows node count + a non-zero recurrence-hit-rate on the matching second fixture. NEGATIVE (R2): a fix that FAILS F-4 (mismatched stdout_hash) mints NOTHING (the cant-lie gate is the birth gate); a forged provenance hash flags the node UNTRUSTED; injecting the VCG into a fixture auditor context FAILS the firewall assertion (SF-3); a superseded node is hard-filtered at read (SF-4); deriving an edge from executor self-report FAILS the contract test (edges are journal-only)."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-96_verified_case_graph_core.md --run-trigger
python .claude/tools/production_caller_check.py --files .elai_cc/crates/orchestrator/src/case_graph/ --json   # mint reachable from the AUDITING verdict path, not a test
rg "CaseNodeMinted" .elai_cc/crates/orchestrator/src/ssb.rs                      # variant exists (all 6 parts)
rg "CaseNodeMinted" .elai_cc/crates/orchestrator/tests/action_type_lock.rs       # golden + TOTAL_VARIANTS bump
rg -n "auditor|critic|adversary" .elai_cc/crates/orchestrator/src/case_graph/    # the firewall guard present (SF-3)
rg "blake2|sha2" .elai_cc/crates/orchestrator/src/case_graph/                     # MUST be empty (blake3 only)
test -f workflow/reasoning/J-96-decisions.md
```

## ACCEPTANCE
- [ ] a case node mints ONLY on an F-4-proven red->green + a sealed RRA; a claim/failed-F-4 mints nothing (the cant-lie gate is the birth gate).
- [ ] edges derive deterministically (cone-overlap / co-occurrence / taxonomy / supersession), recomputable from the journal (GD-2); the LLM sets no edge or weight.
- [ ] the recurrence-hit-rate measure runs on new failing tasks; `elai case-graph stats` reads it; ZERO reuse-into-prompt in this file.
- [ ] SF-3 firewall: the VCG is never injected into the auditor/critic/adversary (assertion proven); SF-4: superseded/stale nodes hard-filtered, never DELETEd.
- [ ] each insight carries a J-10 provenance hash; a forged hash yields UNTRUSTED (ID-3); the LLM writes only insight text (TD-3).
- [ ] SSB 6-part discipline complete; baseline re-read at edit (W-9). cargo check --workspace GREEN; auditor PASS on -p elai-orchestrator (+contracts). No new dependency.

## Tests to add
- crates/contracts/tests/j96_case_graph_core_contract.rs (~150 LOC): (a) mint fires on a fixture F-4-proven red->green and produces a typed node with all six fields; (b) a failed-F-4 fixture mints NOTHING (R2 birth-gate arm); (c) edge derivation wires two cone-overlapping cases and is recomputable byte-identically under ELAI_SEED; (d) the recurrence measure matches a second fixture's trigger_signature; (e) the SF-3 firewall arm: an attempt to inject the VCG into an auditor context FAILS; (f) the SF-4 arm: a superseded node is hard-filtered at read; (g) a forged provenance hash -> UNTRUSTED. traced-test asserts on the journaled mint row (logs_contain inside assert!, tracing-test-assert.md T-1).

## Files touched
- CREATE: crates/orchestrator/src/case_graph/{mod,node,mint,edges,recurrence}.rs (~340 LOC)
- MODIFY: the C-45/C-47 OrchestratorCaseBase store (case-node persistence over the existing case-base) + the AUDITING verdict path (mint trigger) + the new-failing-task path (recurrence measure)
- MODIFY: ssb.rs + action_type_lock.rs(.golden) (+1 variant, 6 parts)
- CREATE: workflow/reasoning/J-96-decisions.md

## Citations / Grounding
- self-improving-workflow-canvas-verified-case-graph-2026-06-25.md §3.1 (mint condition), §3.2 (node content), §3.4 (self-wiring edges), §5 (net-new = the unification object + the deterministic edge layer), §6 Slice 1 (mint+show, observe-first). DGM arXiv:2505.22954 (the archive-grows-and-is-selected-from SHAPE; the cautionary detail "the archive is only as good as the empirical filter" — design-rationale, never a claimed ELAI number).
- ELAI ground truth: F-4/G-2/B-29/failure_taxonomy/I-37/J-10/C-45/C-47/SessionDag per the feature map 1.1/1.5/1.6/1.7 (STALE-by-construction; verify at edit, W-7). live ~268/577 (W-9).
- Hard rules invoked: TD-3 (cant-lie gate is the birth gate; no LLM edge), ID-3 (provenance), GD-2 (journal-recomputable), SF-1/SF-3/SF-4 (mint-on-proof / firewall / supersession), HD-2, blake3-not-blake2, W-7/W-9, GD-1 (compose C-45/J-10/C-25, restate none).
- K-18 is the K-native PROVE+PACKAGE half (the case-reuse economics bench + the per-customer-compounding-state moat-#2 number); J-96 is the technical SAFE-half it recorded OUT-OF-K. K-19 is the SF-1..SF-4 pre-gate.

## Open questions / risks
- trigger_signature similarity: too loose = false recurrence matches inflate the rate; too tight = misses real recurrences. Tune deterministically on a fixture set; record the function in the decisions doc; OBSERVE-ONLY means a wrong threshold costs nothing yet (no reuse).
- scope of "proved fixed" (run §8 open question, carry to the user): red->green test transitions only (cleanest), or also proven plan-unit completions / proven research answers? Wider = richer graph, weaker per-class proof. v1 = red->green ONLY (the cleanest F-4 birth event); widen later behind the same proof discipline.
- double-bookkeeping with C-45/C-47 + J-10: the case node REUSES the case-base store + the insight field; verify at edit it does not create a parallel record (GD-1).

## Next
Lands the VCG's safe half: proven fixes become typed, self-wiring, journal-grounded nodes with a recurrence measure — useful to SEE without yet influencing generation. J-97 adds the gated retrieval-time REUSE (the risky leg, behind the execution-grounded fitness gate). J-98 projects the graph as a canvas MEMORY/CASE altitude. The user's "improves as it gets used" is realized ONLY in J-97, ONLY behind section-4's filter — never here.

End of J-96.
