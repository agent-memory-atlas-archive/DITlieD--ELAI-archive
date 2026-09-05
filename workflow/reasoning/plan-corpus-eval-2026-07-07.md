# Plan-corpus evaluation — what to improve, what to remove
Date: 2026-07-07. Mode: spec-eval AS-SPECIFIED (design judged on merit; shipped-vs-planned never marks the design down).
Read this session: phase-index-MASTER.txt (full) + B/C/D/E/F/G/H/I/K/L/M/N SUMMARY_INDEX (full) + J-SUMMARY_INDEX
(premise + all 110 session headers + wave cross-cutting tails; full body not read — 259KB exceeds the read cap and the
file's own line 2 declares its J-92..J-98 detail entries drifted/unbackfilled).

Corpus scale: ~500 planned units across 13 letters (B42 C59 D30 E55 F13 G10 H54 I54 J110 K25 L21 M6 N15) + 38 phase
files. At the letters' own 6-12h/session estimates this is ~3,000-6,000h of specced work. The proof-spine ruling
(2026-07-05) says the corpus's credibility compresses onto SIX measured numbers (PS-1..PS-6). That asymmetry frames
every verdict below.

## DESIGN VERDICT (headline)

The doctrine spine that accreted F -> N (W-1 run-trigger done-gates, GD-1 compose-not-restate, GD-2 journal-recomputable
numbers, GD-3 ablation-never-rank, GD-4 FP-bounding, KD-1 number-before-claim, TD-3 LLM-never-the-authority, fail-closed
everywhere) is genuinely strong and internally consistent — it is the corpus's real asset, and later letters correctly
treat earlier machinery as substrate instead of re-building it. The corpus's real weaknesses are STRUCTURAL, and almost
all of them are already documented BY the corpus about itself:

W1. Status/stamp drift inside the indexes is the #1 recurring failure. Evidence from the corpus's own text: I-39 exists
    because of "five drifting sources of truth; every index's SCHEMA snapshot went stale within days"; H Track-4 header:
    "CLAUDE.md's 191/464 AND the Track-1/2/3 header's 207/484 are STALE"; Track-7 header marks 220/497 stale at 224/501;
    K stamps 236/545 then 268/577; C-33's job includes fixing B-0's stale "SCHEMA_VERSION=26"; J-SUMMARY line 2
    self-declares its own drift. The indexes carry live mutable state (status, SCHEMA/TOTAL_VARIANTS integers) inside
    what should be immutable design documents.
W2. Concern fragmentation across letters authored against summaries — N's own premise: four claim types, ten routers,
    six intensity deciders, five provenance scales, four counterfactual engines, five context cores, a duplicated MAST
    classifier (E-6 + E-29). The fix (N-1/N-2) is specced but sequenced LAST, after the cost was paid.
W3. Supersession without retirement. The corpus records supersessions in prose (E-6 "supersedes D-13"; E-54 "REPLACES
    D-6 9-gram"; H-3 notes B-28's FIM leg double-gated dark; K-3 makes C-40's PR body composer a substrate) but the
    superseded unit stays an active PLANNED file with no machine-readable marker. An executor walking C or D as written
    would re-build things later letters already re-founded.
W4. Doctrine sprawl: ~15 delta families (W, GD, HD, TD, RD, VD, ID, KD, JD, CC, CL, SF, MD, ND, OQ/FS/HG...) with real
    overlap (GD-4 FP-bounding restated as OQ-3; KD-1 restated as MD-5; TD-3 restated in nearly every letter). K-22/N-8
    measure doctrine ENFORCEMENT; nothing consolidates the doctrine TEXT. Token-discipline's own rule: redundancy
    between files is a bug.
W5. Letter-identity erosion. K-10 ("no new letters") funneled unrelated work into J (110 sessions, unbounded steal
    intake, 259KB summary) and forced K-23/K-24 into K while K's own cross-cutting admits they violate K's premise.
    M and N were then authored via user override anyway. The rule produced shape violations without preventing letters.
W6. An internal doctrine contradiction left standing: C-54..C-59 (SWE-bench Verified runs, Aider leaderboard
    submission, head-to-heads) are leaderboard-rank publications, while the later, binding doctrine says the opposite —
    E-9: "stop reporting SWE-bench Verified"; GD-3: "never a SWE-bench rank"; I-38/K-1 gate all external claims on
    measured ablation numbers. C-54/C-55 as written are now doctrine violations waiting to execute.
W7. The oldest substrate spec (phase-index-MASTER) carries decisions the rules layer has since reversed, presented as
    ground truth: blake2/Blake2s256 (1A/1C deps; CLAUDE.md mandates blake3 — 3C.6 was corrected, 1A/1C lines were not),
    ollama-primary (E-26 deprecates it), r2d2 read pool (anti-stale says deadpool for new code), Gemma-4/RTX-3060
    hardware targets (both rejected/obsolete per memory + baseline findings). These are landmines for any executor that
    reads the master index as canonical.

## IMPROVE (prioritized, each grounded in the corpus's own failure record)

1. Make the summary indexes immutable design docs: strip ALL live status blocks and SCHEMA/TOTAL_VARIANTS integer
   stamps from every *-SUMMARY_INDEX.md and replace with one pointer line ("status: elai_status_query.py / I-39
   `elai plan status`; SCHEMA: re-grep live const at edit per W-9"). The W-9 re-grep rule already makes the stamps
   redundant; keeping them only manufactures the documented staleness class (W1). Cheapest fix with the highest
   drift-elimination yield.
2. Pull N-2 (corpus-grain composition validator) forward to NOW, ahead of any further letter/wave authoring, and give
   it a SUPERSEDED-BY grep-able header convention (mirror of the WIRE-DARK marker discipline): any unit a later letter
   supersedes gets `SUPERSEDED-BY: <unit>` at the top of its entry/file, and N-2 fails a walk that targets a superseded
   unit. N-2 is the only unit whose done-gate runs at plan-authoring time — sequencing it after N-1's XL build inverts
   its value. (PSR-1 already puts N-1 at precedence 1; N-2 belongs beside it.)
3. D-plan reconciliation pass before anything executes a D-N as written: D-6 (9-gram) superseded by E-54 (13-gram +
   shards); D-13 (Adversary) superseded/extended by E-6; D-10/D-11/D-16/D-17 are cited as SHIPPED by H Track 6 while
   D's index still says PLANNED; D-19..D-27 bench wave overlaps the later G-6/I-38/N-1 runner family. Output = a
   per-unit LIVE/ABSORBED/SUPERSEDED table appended to D-SUMMARY_INDEX. Same pass, smaller, for C's residue (C-18..C-20
   test backfill vs the shipped frozen-checker harness; C-27..C-29 testing-trust vs I-20/H-27).
4. Doctrine codex: consolidate the ~15 delta families into ONE canonical numbered rulebook (docs or .claude/rules),
   letters citing clause IDs instead of restating. Precedent: the 2026-07-06 dispatch-core.md consolidation, done for
   exactly this reason. K-22's doctrine_coverage.py then has one source to tag instead of fifteen.
5. Letter-grain PROOF_SPINE annotations: one `PROOF_SPINE:` line per SUMMARY_INDEX header (PSR-2 applied retroactively
   at letter grain), so the sequencing law is visible where prioritization actually happens. E's "Wave H — everything
   else" and most of C's Wave 7-8 would be exposed as spine-exempt, which is the point.
6. Patch phase-index-MASTER's stale landmines with a short CORRECTIONS header (blake3-not-blake2 for 1A/1C, ollama
   deprecated per E-26, deadpool-not-r2d2, hardware/model targets superseded by the VRAM tier plan + remote 3090).
   Do not regenerate the whole file; the header is enough to stop an executor inheriting reversed decisions.
7. J hygiene: backfill the self-declared J-92..J-98 gap (or split J-SUMMARY by wave), and add a tripwire N-2 can
   enforce: a SUMMARY_INDEX exceeding ~100KB or carrying a self-declared drift note is an authoring BLOCK until split
   or reconciled. J is the corpus's biggest single drift risk (110 sessions, every steal run lands there).
8. Extend K-8's absorption sweep explicitly to the pre-F efficiency-class residue it does not name: B-28 Focus Tunnel
   (FIM/DIFF legs vs provider-native edit modes), C-25 compiled pipeline, C-26 constrained-decoding defaults (already
   under C-37 re-eval), E-2 legs. K-25's watchdog then keeps them honest quarterly.

## REMOVE (proposals only — house rule: no removal without explicit user confirmation)

R1. C-54..C-59 (the RunPod leaderboard wave, 6 sessions): contradicts the binding later doctrine (E-9 + GD-3 + KD-1).
    The useful intent survives elsewhere: G-6 SAFEDELTA, I-38 QDELTA, K-2 routing-economics, L-20 competitor-arm paired
    harness (which explicitly bans leaderboard-rank strings). Salvage: C-56's codebase-memory head-to-head re-scoped as
    an L-20 arm; C-51's vendored polyglot harness kept as L-20 corpus supply. Kill the submissions (C-55) outright.
R2. B-25 + C-34 + phase-4J (Empire integration): conditional on external codebase access that has not materialized
    across the entire corpus history; three separate files carry it. Archive to a single tombstone; delete the
    duplicated planning surface.
R3. C-44 (Composer2 runtime backend): B-37's 11-backend adapter + J-9/I-51 model catalog + K-24's OpenRouter managed
    rail make a bespoke per-model backend session obsolete; a Composer-class model is a catalog row now, not a letter
    unit.
R4. The E-6 / E-29 duplicate MAST classifier — resolve at authoring time now instead of waiting for N-12(a); one
    canonical FailureClass enum, the other becomes a consumer.
R5. D-13 as a standalone session (fold into E-6, which its own entry says supersedes it) and D-6's 9-gram leg (E-54
    replaces it; keep D-6's store/temporal-split, mark the filter leg SUPERSEDED-BY E-54).
R6. K-10 (no-new-letters rule): superseded in practice by PSR-1/PSR-2 which actually govern sequencing; K-10's residue
    caused the J-unbounded-growth and K-23/K-24 shape-violations recorded in K's own cross-cutting. Retire it, keep
    GATE M-0 (which is real sequencing, not letter-naming policy).
R7. E-47's active slot (LoRA): already DEFERRED by its own report; its live content is absorbed by I-30 (SFT exhaust),
    E-55 (RL ladder deferral), and the J STEAL-JUL04 PAW riders. Keep the ship-gate token file as the tombstone.
R8. HOLD (not removal): C-35..C-53 (Wave 7-8, authored 2026-05-17) get a KD-4 provider-surface re-verify (K-8/K-25
    machinery) before any execution — their grounding is ~7 weeks old under the corpus's own absorption doctrine.

## What I would NOT change

- No bulk retro-edit of B..E to the wargame format: plan-standard's lazy pre-flight (refine-on-execute) is the right
  call and already binding.
- The F->G->H compose-not-restate chain, the frozen-gate/off-agent-artifact doctrine, and N's existence as the
  interlace letter are correct designs; N should just fire earlier (Improvement 2).
- M's fail-OPEN commerce / fail-closed safety split (MD-1) is a deliberate, well-argued inversion — keep.
- K's PROVE/PACKAGE/DISCIPLINE sliver shape is a good pattern; the K-23/K-24 deviation is a K-10 symptom (R6), not a
  K design flaw.

## IMPLEMENTATION-READINESS NOTE (separate axis, per spec-eval; status claims are index-derived — confirm against live code)

Per the indexes: B mostly shipped; C/D largely PLANNED with pieces shipped out-of-band by later letters; E partially
shipped (per-entry); F-1..F-2c shipped per H/I references; G-2/G-8/G-10 shipped per later references; H/I mixed with
many ACTIVATION targets confirmed live; J/K/L/M/N mostly PLANNED with GATE M-0 + PSR-1 holding execution order. None of
this affected the verdicts above; it only affects which improvement lands first (Improvement 1 and 2 are pure-harness,
zero engine risk, and executable immediately).

## Ledger

GRC decision recorded: topic plan-corpus-eval, 2026-07-07 (mcp_think, VERIFIED). PROOF_SPINE: none (planning-discipline
artifact; spine-exempt class per PSR-2 — needs user ack).
