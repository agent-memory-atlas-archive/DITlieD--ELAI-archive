# N-2: corpus-grain composition validator + M-1 one-registry rule — wiring-enforcement R6 lifted to the plan corpus

- Track: TIER 2 (HIGH) | lever: plans-that-talk | finding: W-6+M-1+W-33+W-32 | grounding: STRONG | nature: BUILD | effort: L | WAVE 2 | depends: N-1

- Premise: the feature map's EXTENDS/composes/consumes/gated-by edges are hand-maintained PROSE with zero machine check; J-SUMMARY_INDEX.md:2 self-declares the letter drifted from its own index (detail stopped at J-91). Nothing validates the CROSS-LETTER edge set. This is the structural cure for the user's stated fear: "plans authored against summaries silently duplicate or contradict." N-2 is the M-1/ND-1/ND-3 enforcer for every FUTURE letter.

## Steal/build
Build a corpus_composition_check.py (sibling of wiring_contract_check.py) that parses the feature map's edges as DATA (the ND-3 row format) and joins them against the plan files + elai.map symbols, FAILING on:
(a) DANGLING EDGE — a "consumes X" naming no unit that exposes X;
(b) ORPHAN PRODUCER — a unit with >=1 consumer in NO letter (the orphan-feature report);
(c) DUPLICATE-CONCEPT COLLISION — name-similarity + shared-seam detection (the check that would have auto-caught the four claim types / the ten routers / the E-6+E-29 MAST dup at authoring time), i.e. the M-1 one-registry-per-concern rule made mechanical.
It EMITS the global unlock-DAG + fan-in ranking (W-32) and derives the summaries/status mechanically by composing J-93-G (auto-derive the map's mechanical half) + I-39 (journal-derived status), with a SUMMARY_INDEX-vs-0_INDEX-vs-detail FRESHNESS check (the elai_map_freshness.py pattern applied to the index layer, W-33). Runs at paper-to-plan phase 5 / planning-workflow authoring time the way wiring_contract_check runs at code time.
EXCLUDED: any change to the runtime wiring gate (this is the PLAN-corpus twin, not a replacement); re-implementing J-89 L8 (extend the per-plan type-completeness check to corpus grain, do not re-author it).

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)
- J-89 L8 checks plan-DAG producer->consumer type-completeness PER PLAN (feature-map I/J entries).
- J-93-G auto-derives the feature map's mechanical half; I-39 derives plan status from the journal.
- .claude/tools/wiring_contract_check.py + elai_map_freshness.py + spec_link_check.py exist (CLAUDE.md harness-tools block) — the patterns to mirror.
- the feature map's OPEN CONFLICTS section had 7 unresolved edges; I-2/I-5/I-11/H-50 were "PLANNED label was STALE" corrections — proof the drift is real, recurring, currently caught by luck.

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1): the three FAIL checks (dangling / orphan / duplicate-concept) + the freshness check + the unlock-DAG emit. Fail-closed: a corpus that finds ZERO edges exits nonzero (R1, ArchUnit failOnEmptyShould).
- DEFER (Tier-2): full name-similarity ML (start with a deterministic shared-seam + token-overlap heuristic; the fuzzy semantic-duplicate detector is a later refinement, flagged directional).

## Integration
- .claude/tools/corpus_composition_check.py (new) + a small ND-3 edge-row emitter the feature-map upsert writes. Wire into the paper-to-plan phase-5 / planning-workflow PRE_GATE authoring seam.
- frontend_pair: N/A — an authoring-time CI gate, not a runtime surface. Justified.

## WIRING_CONTRACT
produces_symbols:
  - .claude/tools/corpus_composition_check.py — the three FAIL checks + unlock-DAG + freshness
  - the ND-3 edge-row format (the machine-readable edge the feature-map upsert emits)
consumes_symbols:
  - J-89 L8 plan-DAG type-completeness (extend to corpus grain)
  - J-93-G map-half derivation + I-39 journal status
  - the wiring_contract_check.py / elai_map_freshness.py patterns
reachable_from:
  - the paper-to-plan phase-5 authoring step + planning-workflow PRE_GATE  [PRODUCTION authoring seam]
  - a CI invocation over docs/thefinalglance/  [PRODUCTION gate]
stub_ban:
  - NO exit-0 on a corpus with a dangling edge (fail-closed, R1)
  - NO opt-in flag that lets the check emit PASS on missing edges (R3)
  - NO prose-only edges (edges must be DATA rows, ND-3)
frontend_pair: N/A (authoring-time gate); justified.
observable_effect:
  trigger: "python .claude/tools/corpus_composition_check.py --map docs/thefinalglance/ELAI-FEATURE-MAP.md"
  expect_ssb: "+composition_edge_checked(DanglingEdge|OrphanProducer|DuplicateConcept|FreshnessDrift)"
  expect_artifact: "nonzero exit + the offending edge named on a seeded dangling edge; the unlock-DAG artifact on a clean run; NEGATIVE: a clean corpus with a hand-inserted dangling edge that EXITS 0 is a FAIL"

## Grounding
- interlace doc W-6 (the user's fear made precise), M-1 (one-registry meta-rule), W-33 (the J drift smoking gun), W-32 (the unlock-DAG), TIER-2 item 2.
- wiring-enforcement.md R6 (producer-without-consumer is a build break) — this lifts it to plan grain; R1 (missing evidence FAIL) / R3 (no opt-in) / R8 (frozen gate).

## COMPOSES-WITH
J-89 L8, J-93-G, I-39, wiring_contract_check.py + elai_map_freshness.py patterns. Re-implements NONE (GD-1). This unit BUILDS the ND-1/ND-3 checker the whole N-plan (and every future letter) is judged against.

End of N-2.
