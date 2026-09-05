# N-12: consistency + dedupe bundle — eight same-concern collapses in disjoint files

- Track: TIER 4 (MEDIUM) | lever: consistency-and-free-training-data | finding: W-8+W-9+W-12+W-16+W-21+W-25+W-26+W-28 | grounding: STRONG | nature: DELETE | effort: M | WAVE 8

- Premise: eight independent same-concern fragmentations, each low-cost to collapse, each in disjoint files (so the sub-slices are parallel-safe and independently shippable). None invents anything — each collapses >=2 existing implementations to the first-in-flow canonical one. This is the ND-1 sweep the N-2 validator will keep clean going forward.

## Steal/build (eight disjoint sub-slices; each is its own vertical slice)
(a) W-8+W-26 ONE FailureClass enum in SSB: failure_taxonomy.py 13-cat + crates/tools 9-cat + E-6 MAST 14-mode + E-29 MAST DUP (same classifier, second aware of first only via a parenthetical) + C-10 StagnationPattern 4-variant -> one typed enum shared by auditor attribution / anti-spiral / MAST supervisor (J-93-A) / eval corpus rows / case-graph edges. Classified once at audit time IS the corpus row's type IS the case edge IS the stagnation signal. Frees training data for J-25/I-12.
(b) W-9 ONE AbstainDecision spine: D-16 conformal + I-8 voluntary abort + D-17 SLA + L-7 policy + J-73 retrieval + J-110 timeliness -> one row type + a composition rule (an upstream retrieval-abstain TAINTS downstream steps as answer-without-evidence -> L-2 risk floor). (L-7 already composes D-16+I-8+D-17; RD-1 mandates read-never-re-derive — extend, don't re-author.)
(c) W-16 ONE ProjectClass registry: B-40 ProjectClass 9-variant (first in flow) consumed by B-42 cosmos project_classes / E-50 SaaS archetypes / E-12 framework manifests / L-14 build defaults.
(d) W-21 route C-47 sled CaseBase + C-49 markdown ExperienceCorpus writes through the J-109 admission pipeline (closes an unguarded learned-artifact write path; J-109 guards E-5/I-12/J-97 but not C-47/C-49 today).
(e) W-25 ONE drift-primitive library: D-4 tripwire (KL/delta-rho/IQR/perplexity/DDM) + G-10 model-drift + H-30 staleness + F-5 cascade + B-22b goal-drift share ONE windowed-divergence/Page-Hinkley/DDM core; keep the distinct policies.
(f) W-12 FSM three-model conformance test: 3I DST FsmReferenceModel == J-68 ElaiFSM.tla == the code transition table (the table is already data-shaped; make it the single source all three consume).
(g) W-28 ONE LSP daemon pool: E-52 LSP-as-verifier + J-18 LSP retrieval channel + J-52 per-edit LSP block-feed share one per-language health-managed pool (like the B-37 runtime adapters). Saves RAM + startup latency + three divergent process managers.
EXCLUDED: adding a new type for ANY of these concerns (collapse to the existing canonical one, ND-1); a ninth collapse (scope is these eight).

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)
- failure_taxonomy.py (harness) + 9-cat (crates/tools phase-4B) + E-6/E-29 MAST + C-10 StagnationPattern all exist (interlace doc W-8/W-26).
- the abstention family (D-16/I-8/D-17/L-7/J-73/J-110), ProjectClass homes (B-40/B-42/E-50/E-12/L-14), experience stores (C-47 sled / C-49 markdown), drift detectors (D-4/G-10/H-30/F-5/B-22b), FSM models (3I DST / J-68 TLA / code table), LSP integrations (E-52/J-18/J-52) all exist as noted in the interlace doc (W-9/W-16/W-21/W-25/W-12/W-28).

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1): the cheap high-consistency collapses — (a) FailureClass enum, (c) ProjectClass registry, (f) FSM conformance test (the table exists). Each independently shippable.
- DEFER (Tier-2): (b) AbstainDecision spine, (d) experience-store admission routing, (e) drift-primitive library, (g) LSP daemon pool — each a slightly larger refactor; sequence after the Tier-1 three.

## Integration
- the respective crates (orchestrator FailureClass/AbstainDecision/ProjectClass/drift; security admission; tools LSP pool). Each sub-slice edits its own files — parallel-safe.
- frontend_pair: N/A for most (a FailureClass drawer if surfaced); justified per sub-slice.

## WIRING_CONTRACT
produces_symbols:
  - one FailureClass enum / one AbstainDecision row + composition rule / one ProjectClass registry / one
    drift-primitive lib / the FSM three-model conformance test / one LSP daemon pool
  - the J-109 admission routing for C-47/C-49 writes
consumes_symbols:
  - failure_taxonomy.py + 9-cat + E-6/E-29 MAST + C-10 (FailureClass)
  - D-16/I-8/D-17/L-7/J-73/J-110 (AbstainDecision); B-40/B-42/E-50/E-12/L-14 (ProjectClass)
  - C-47/C-49 + J-109 (admission); D-4/G-10/H-30/F-5/B-22b (drift); 3I/J-68/code-table (FSM); E-52/J-18/J-52 (LSP)
reachable_from:
  - each surviving canonical impl called from its >=2 former call sites  [PRODUCTION per sub-slice]
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO new type for any collapsed concern (ND-1)
  - NO second parallel implementation surviving the collapse
  - NO experience-store write bypassing J-109 admission
frontend_pair: N/A for most (FailureClass drawer if surfaced); justified per sub-slice.
observable_effect:
  trigger: "per sub-slice — drive the ONE surviving implementation from its >=2 former call sites"
  expect_ssb: "no new action_type (dedupe removes emitters; the surviving canonical type keeps its row)"
  expect_artifact: "one canonical row/type per concern; NEGATIVE: a second parallel implementation of a collapsed concern re-appearing fails the N-2 duplicate-concept check"

## Grounding
- interlace doc W-8/W-26 (FailureClass + MAST dup), W-9 (AbstainDecision), W-16 (ProjectClass), W-21 (experience stores), W-25 (drift primitives), W-12 (FSM conformance), W-28 (LSP pool), TIER-4 item 12.
- E-6/E-29 MAST dup is the concrete demonstration that summary-per-report authoring duplicates machinery — N-2 stops the next one.

## COMPOSES-WITH
every unit named per sub-slice (dedupe to first-in-flow, adapters for the rest), J-109 admission, N-2 (the duplicate-concept check that keeps it clean). Re-implements NONE (GD-1). Consistency + free training data; the ND-1 sweep of the corpus's accumulated fragmentation.

End of N-12.
