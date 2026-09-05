# PLAN-FOLDER-TEMPLATE — how to author the next plan folder (I-plan, J-plan, ...)
Grounded 2026-06-10 against the live H-plan folder (H-0_INDEX.md 988 lines, H-SUMMARY_INDEX.md 1039 lines,
H-8 = the full-form detail file, H-52 = the compressed-form detail file). This file is the canonical
authoring spec for any successor plan folder. The next letter after H is I (skip nothing; B..H are taken).

================================================================================
0. WHERE AND WHAT
================================================================================
Folder: docs/thefinalglance/I-plan/
Mandatory contents:
  I-0_INDEX.md                          the doctrine + master index (read FIRST by every executor)
  I-SUMMARY_INDEX.md                    one compact entry per session (mirrors the predecessor SUMMARY format)
  I-N_<snake_case_short_title>.md       ONE detail file per session, N sequential across ALL tracks (I-1, I-2, ...)
  _grounding/I-<TOPIC>-GROUNDING-MAP.md one grounded symbol map PER TRACK, authored BEFORE that track's detail files
Naming rules (observed canon):
  - detail file slug is lowercase snake_case, short, descriptive: H-8_persist_oracle_answer_cache.md,
    H-52_deterministic_finding_anchoring_cockpit.md. Number prefix joins with underscore, never a space.
  - N never resets per track. A reserved/unauthored track keeps its number RANGE reserved and the index says so
    (H-plan: "Track 5 = odysseus, reserved/unauthored"; later tracks bump the reserved start and the index records
    every bump). Collisions are forbidden; the index is the allocator.
  - grounding maps are per-track, not per-session: _grounding/H-GROUNDING-MAP.md (Tracks 1/2),
    _grounding/H-TRUST-GROUNDING-MAP.md (Track 3), etc.

A "session" = one shippable unit of work = one detail file = one wiring done-gate. A "track" = a themed group of
sessions birthed by one source design doc / research run. Tracks are added over time; the index headers are
UPDATED in place on every track addition (title session-count, track list, "Generated:" dates).

================================================================================
1. I-0_INDEX.md — required sections, in this order
================================================================================
1.  H1 title line: "# I-Plan Index — the <THEME-1> + <THEME-2> + ... side, wiring-enforced
    (<N> sessions; Tracks ... authored; Track X = ..., reserved)". The title carries the live session count and
    track inventory; update it on every track addition.
2.  Date: lines — one per authoring wave, dated.
3.  Source synthesis: — the design docs / research runs each track came from, with paths and a one-line gloss each.
4.  Grounding map (W-7 source): — one line per _grounding/ map: path, which track, what it contains (the
    correction/confirmation ID ranges, e.g. "TS-1..TS-8 + NN-1..NN-13"), the grep date and run id.
5.  Predecessors: line — "B-plan, C-plan, ... H-plan. I COMPOSES-WITH them (GD-1); restates none." Plus any
    track-specific naming notes (H-plan's "Track 3 NOTE" pattern).
6.  Predecessor reading (mandatory before any I-N execution): — ordered list: the grounding map slice, the
    F-0_INDEX W-1..W-10 canon, the G-0 GD deltas, the H-0 HD deltas, this file, then I-SUMMARY, then the detail
    file; plus per-track-only extra reads.
7.  ## Why I-plan exists (read this before anything else) — one prose paragraph PER TRACK, formatted
    "TRACK n — NAME (I-a..I-b). <thesis, authored date, source, the CENTRAL FINDING grounded against live code,
    what is ACTIVATION vs BUILD, what each session does in half a line, what delta family the track ADDS>."
    This is the highest-value section: it must state the track's central grounded finding (the H pattern:
    "the substrate is ALREADY SHIPPED / the filter is two placeholders / cloud and local need OPPOSITE policies"),
    never just a feature list.
8.  ## Status legend — PLANNED / IN PROGRESS / DONE / DEFERRED, with DONE defined as the FULL wiring done-gate
    (wiring_contract_check.py --run-trigger PASS; "cargo-green is necessary but NOT sufficient").
9.  ==== THE WIRING DOCTRINE ==== (the ==== separator is a full 80-char line of '='):
    a. the inherited W-1..W-10 restated one line each, marked "INHERITS ... VERBATIM (canon: F-0_INDEX.md)";
       plan-specific caveats allowed in brackets.
    b. the inherited GD-1/GD-2/GD-4/GD-5 (canon: G-0_INDEX.md), one short paragraph each.
    c. the inherited HD-1..HD-5 (canon: H-0_INDEX.md) IF the plan touches efficiency/canvas surfaces.
    d. NEW delta families: only if a track genuinely needs new BLOCKING rules, add a 2-letter-D family per track
       (H used TD/RD/VD/RV/CL), numbered, each rule with a name, the rationale, the paper/code citation, and the
       [I-N] it binds to. A track that is fully covered by existing doctrine ADDS NO family and says so
       (H-52: "ADDS no new delta family").
    e. DONE DEFINITION block: the numbered list the FSM may not reach COMPLETE without (cargo check GREEN,
       auditor PASS, --run-trigger PASS, cross_lang_diff if frontend_pair, the W-6 dashboard item, the GD-4
       FP bound where a gate defaults ON, memory-writer ran).
10. ==== THE <N> SESSIONS ==== — per-track fixed-width tables:
       I-N   title                              lever / nature           composes-with        effort wave
    one row per session + an indented 1-3 line parenthetical gloss under each row. nature is one of
    ACTIVATION / BUILD / BUILD-on-shipped / EXTENSION / DELETE / port. effort is S / S-M / M / M-L / L / XL.
    Tag keystones and blockers inline: [KEYSTONE], [BLOCKING], depends I-M.
11. ==== CANONICAL CONSTANTS + SCHEMA_VERSION BUMP SITES ==== — the LIVE baseline at generation
    (SCHEMA_VERSION + TOTAL_VARIANTS with file:line), the warning "NEVER hard-code; re-read at edit", and a
    per-session table of SSB action_type deltas ("+foo_event (Pattern-2; Kind: A/B/C). baseline+1." or
    "NO new action_type — reuse X (ssb.rs:NNNN)").
12. ==== SEQUENCING ==== — the waves, cheapest-and-grounded first, with the rationale per wave.
13. ==== HARD RULES INHERITED FROM CLAUDE.md + F-0/G-0/H-0 ==== — the one-liners that apply to every I-N
    (blake3 only, no .unwrap in prod, scoped tests, anti-slopsquatting, etc).
14. ==== CARRIED WEB LANDMINES ==== — externally-verified corrections that must not regress (dated, each with
    the wrong value AND the corrected value).
15. ==== CARRIED DESIGN-DOC CORRECTIONS ==== — the activation-vs-build reality flips from the grounding maps
    ("authors MUST use these, not the design doc").
16. ==== SESSION LOG (populated as I-Ns ship) ==== — empty at generation, one row per shipped session.
17. ==== NEXT STEP ==== — the literal first command of WAVE 1.

================================================================================
2. I-SUMMARY_INDEX.md — required sections, in this order
================================================================================
1. H1 title: "# I-plan — Summary Index of All <N> sessions (Tracks ...)".
2. "Generated:" line with per-track dates + "Mirrors H-SUMMARY_INDEX format."
3. A 4-6 line preamble stating what each entry preserves (track + lever, grounding strength, scope, the
   substrate it activates or the absent code it builds, the COMPOSES-WITH artifact, the integration point,
   the observable_effect) and pointing at I-0_INDEX for doctrine + the _grounding maps.
4. ## Premise — one paragraph: each track's one-line thesis + the differentiator deltas vs predecessors.
5. Per track: a ==== TRACK n — NAME ==== separator, then one "### I-N — title [optional tag]" entry per session:
     - Track n | lever X | grounding STRONG|MODERATE|WEAK | nature X | effort X | WAVE n [| depends ...]
     - Steal/build: 3-8 lines. THE GROUNDED GAP STATEMENT — what exists (with file:line), what is dark/absent
       (with the grounding-map ID), what this session does about it.
     - COMPOSES-WITH: the named prior-plan artifacts CONSUMED (GD-1) + the explicit "do NOT re-implement" items.
     - Integration: the real file:line anchors to be edited + "frontend_pair:" (or its justified absence).
     - observable_effect: the W-1 done-gate in 2-4 lines — the LIVE trigger, the off-agent artifact, the
       negative case that must FAIL.
6. After each track's entries: "## Track-n cross-cutting (what makes it different)" where the track warrants it.

================================================================================
3. I-N_<slug>.md — the per-session detail file
================================================================================
Two observed variants exist in H-plan. The FULL FORM (H-8 pattern, Tracks 1-4) is canonical; the COMPRESSED
FORM (H-52 pattern, Tracks 10/11) is acceptable ONLY when every mandatory block below is still present (it
folds DETERMINISTIC_CHECK + ACCEPTANCE into the WIRING_CONTRACT prose). When in doubt, use the full form —
wiring_contract_check.py parses the WIRING_CONTRACT block either way, but auditors read the full form faster.

FULL FORM skeleton (every section, in this order):

  # I-N: <title — one line, what + why>

  STATUS: PLANNED (WAVE n; <tier note>; <nature> — <one-line scope qualifier>)

  Objective: one paragraph. The grounded gap (file:line + grounding-map ID), what I-N ships, the hard
  boundary it must NOT cross, the GD-1 "already shipped — do NOT re-implement" callouts.

  Estimated effort: ~X sessions (~Y h). <W-10 tier note: in-process at gate time? network-gated?>
  Pre-requisites: <prior plan artifacts SHIPPED, with file:line — each one a thing the executor re-greps>
  Parallel-safe with: <the I-Ns with disjoint files>
  Source: <the design-doc section + the grounding-map slice>. Doctrine: I-0_INDEX.md.

  ## Context
  Why this matters: the external evidence (papers with arXiv IDs, prior-art, measured numbers — each tagged
  per HD-1 if borrowed), the correctness boundaries, the one-paragraph mechanism.

  ## Existing state (B/C/D/E/F/G/H coverage)
  Bullet list: every relevant shipped symbol with file:line, its live/dark/absent status, and the grounding-map
  ID that confirmed it. This is the W-7 anti-hallucination anchor — nothing here is from memory.

  ## Reads (mandatory before coding)
  Ordered list of exact paths (+ line ranges where the file is big): the grounding-map slice, the design-doc
  section, the source files to be modified, the pattern files to MIRROR, the CLAUDE.md hard rules that bite.

  ## Tasks (WAVE n — <label>)
  Numbered. Each task: imperative title + ~LOC estimate + 2-5 lines of HOW including the negative space
  (what NOT to do). The LAST task is always the decision doc: workflow/reasoning/I-N-decisions.md.

  ## WIRING_CONTRACT
  produces_symbols:
    - <file (the exact new surface added)> — one bullet per produced artifact
  consumes_symbols:
    - <file::Symbol (what it does / why consumed)> — every prior-plan artifact composed-with (GD-1)
  reachable_from:
    - <file:line (the REAL production entrypoint chain)>  [PRODUCTION <surface name>]
    - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
  stub_ban:
    - NO <each forbidden shortcut, one per line: re-implementations, model-settable fields, silent fallbacks,
      .unwrap()/.expect() on prod paths, blake2/sha2, non-deterministic reload, ...>
  frontend_pair:
    <the dashboard surface + the Tauri read> OR "N/A with justification — <why>. Justified."
  observable_effect:
    trigger: "<the literal command(s) — a REAL entrypoint, runnable verbatim>"
    expect_ssb: "<the off-agent SSB row(s) that must land, with the discriminator values>"
    expect_artifact: "<the on-disk / on-wire / rendered artifact + the NEGATIVE case that must FAIL>"

  ## DETERMINISTIC_CHECK
  ```bash
  python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/I-plan/I-N_<slug>.md --run-trigger
  rg "<positive pattern>" <file>     # <what its presence proves>
  rg -c "<banned pattern>" <file>    # MUST be 0
  test -f workflow/reasoning/I-N-decisions.md
  ```
  Every check is a single deterministic command with its expected outcome in the trailing comment.

  ## ACCEPTANCE
  - [ ] one checkbox per W-rule / GD-rule / delta-rule that binds, each restated CONCRETELY for this session
        (not "W-2 holds" but "persistence active with no env override").
  - [ ] cargo check --workspace GREEN; auditor PASS on -p <the affected crates only>.
  - [ ] No new dependency (or: the ONE new dep, anti-slopsquat-verified, user-confirmed).

  ## WIRING DONE-GATE (clause by clause)
  - W-1 through W-10 + the GD/delta rules: ONE line each mapping the clause to this session's concrete
    evidence, including the N/A clauses WITH justification ("W-6: N/A (no dashboard surface); justified.").

  ## Tests to add
  - crates/contracts/tests/iN_<slug>_contract.rs (~LOC): lettered list (a)..(e) of the specific assertions,
    including the corrupt-input degradation case and the negative/confinement case.

  ## Files touched
  - MODIFY: <path> (~LOC: what changes)
  - CREATE: <path> (~LOC)
  - CREATE: workflow/reasoning/I-N-decisions.md (~LOC)

  ## Citations / Grounding
  - the design-doc section with its measured numbers and their scope caveats;
  - "ELAI ground truth (<grounding-map ID>):" the file:line facts this plan stands on;
  - the CLAUDE.md hard rules invoked. Every arXiv ID WebSearch-verified at authoring (never from memory).

  ## Open questions / risks
  - 2-5 bullets: the genuine unknowns, each with its mitigation or its "decide at authoring/executing" marker.

  ## Next
  One short paragraph: what this closes, what it pairs with, what becomes possible after.

  End of I-N.

The literal "End of I-N." terminator line is mandatory (truncation tripwire: a reader that doesn't see it
knows the file was cut).

COMPRESSED FORM (only for steal/research tracks): H1 + the metadata bullet (Track | lever | grounding | nature |
effort | WAVE | depends) + "- Premise:" + ## Steal/build (with the EXCLUDED-as-vaporware negatives) +
## Existing ground truth + ## REVISED scope (W-10 tiered: SHIP NOW Tier-1 / DEFER Tier-2) + ## Integration +
## WIRING_CONTRACT (produces / consumes / reachable_from / stub_ban / observable_effect as labelled prose
bullets) + ## Grounding + ## COMPOSES-WITH + "End of I-N." — all five contract fields still mandatory.

================================================================================
4. _grounding/I-<TOPIC>-GROUNDING-MAP.md — per track, authored FIRST
================================================================================
Before any detail file of a track is written, run the grounding pass (live elai.map greps + targeted source
reads + web verification for external claims) and write the track's map. It carries:
  - ID-ranged finding families (the H pattern: TS-n substrate-shipped corrections, NN-n net-new confirmations,
    TD-n deltas, C-n design-doc corrections) — every detail file cites these IDs instead of re-asserting facts;
  - the per-session symbol tables (every file:line anchor a detail file will use);
  - CANONICAL CONSTANTS (the live SCHEMA_VERSION/TOTAL_VARIANTS at grep time, with bump sites);
  - DESIGN-DOC CORRECTIONS (where reality flipped ACTIVATION vs BUILD);
  - WEB VERIFICATION results (dated, with the query and the source).
The map is dated and run-id-stamped. Detail-file authors MUST use the map's corrected values, and the
wiring-contract validator re-greps every anchor (W-7 is enforced twice: at authoring and at validation).

================================================================================
5. NON-NEGOTIABLE AUTHORING RULES (the ones that make or break the folder)
================================================================================
R-a  GROUND EVERY ANCHOR. Every file:line, symbol, SCHEMA number, and arXiv ID in any I-plan file is
     grep/WebSearch-confirmed AT AUTHORING TIME and traceable to a grounding-map ID. A plan-asserted (not yet
     code-confirmed) anchor is LABELLED as such ("PLAN-asserted; executor re-greps per W-7" — the H-52 pattern).
R-b  COMPOSE, NEVER RESTATE (GD-1). Every session names the prior-plan artifact it consumes and the things it
     must NOT re-implement. Re-stating a shipped artifact is a P0.
R-c  LIVE BASELINE, NEVER HARD-CODED. SCHEMA_VERSION/TOTAL_VARIANTS are quoted as "baseline+N from LIVE X/Y
     as of <date> (file:line); re-read at edit (W-9)". History shows the baseline goes stale between tracks —
     the re-read instruction is what saves it.
R-d  VERTICAL SLICES ONLY. A session ships producer + in-plan consumer + smoke trigger together
     (wiring-enforcement PRE_GATE). "Add fn X" without its caller and its observable_effect is refused at
     planning time.
R-e  FAIL-CLOSED observable_effect. The trigger is a real entrypoint; the proof is an artifact the implementing
     agent did not author (SSB row / coverage hit / rendered node observed by the auditor); missing evidence is
     FAIL; include the negative arm that must FAIL (the H-52 --trust-model-line-numbers broken-arm pattern).
R-f  KILL-SWITCH, NOT ENABLE-FLAG (W-2). New gates ship default-ON behind ELAI_*_DISABLE; a default-OFF
     ELAI_*_ENABLE is wire-dark by design unless registered in wire-dark-allowlist.toml with a promotion plan.
R-g  HONEST RESIDUE (GD-5). Every track and every session names what it CANNOT guarantee, in the file, near
     the claim. Borrowed numbers are tagged ([WEAK]/[DESIGN]/directional-only) and never multiplied (HD-1).
R-h  TOKEN-DISCIPLINED PROSE. Lowercase prose style of the house; no filler, no emojis; the index carries the
     doctrine ONCE and detail files cite it — redundancy between files is a bug.
R-i  UPDATE THE ALLOCATOR. Adding a track = update I-0_INDEX title/track-list/session-count + the reserved-track
     start bump + I-SUMMARY title + the memory-bank pointer. The index is the single source of truth for
     numbering.
R-j  DELTAS ARE EARNED. A new XX-delta rule family requires a track whose failure modes existing doctrine does
     not cover. Default is "ADDS no new delta family" with the existing rules cited.

================================================================================
6. BOOTSTRAP CHECKLIST FOR I-PLAN (the literal first session)
================================================================================
1. Collect the source design docs / research runs; one track per source.
2. Run the grounding pass; write _grounding/I-<TOPIC>-GROUNDING-MAP.md (dated, run-id-stamped).
3. Read the LIVE SCHEMA_VERSION/TOTAL_VARIANTS (ssb.rs + action_type_lock.rs) — this is the quoted baseline.
4. Write I-0_INDEX.md (sections 1-17 above; SESSION LOG empty; doctrine = inherit W-1..W-10 + GD + HD verbatim,
   add deltas only per R-j).
5. Write I-SUMMARY_INDEX.md (Premise + per-session entries from the grounding map).
6. Write the I-N detail files, full form, one per session, in wave order.
7. Validate: spec_link_check.py + wiring_contract_check.py (static legs) over every detail file; the validator
   re-greps every anchor.
8. Update memory-bank pointers via memory-writer (never directly).

End of PLAN-FOLDER-TEMPLATE.
