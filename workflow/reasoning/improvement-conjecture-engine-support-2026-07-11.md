# improvement-conjecture-engine — support notes (2026-07-11)

Written by the session that authored L-24 (the conjecture-memory middle tier) for the session that will
author the IMPROVEMENT-CONJECTURE ENGINE plan. Everything below is either verified this session (dated,
with anchors) or an explicit design conclusion carried over from the L-24 work. Read this AFTER the
prompt's own required reads; it does not replace them, it saves you re-deriving what one session already
settled. Companion artifacts: docs/thefinalglance/L-plan/L-24_conjecture_memory_middle_tier.md (gate PASS),
the COLLAB-WAVE OUTCOME block at the end of workflow/reasoning/steal-deepreason-2026-07-08.md, and the
2026-07-11 plan-authoring entry in workflow/handoff/deepreason-steal-and-collab.md.

## 1. The sibling relationship — engine vs the L-24 tier (state this boundary in your plan)

L-24 and the engine are the SAME epistemic shape at two different grains. Do not merge them and do not
re-invent one inside the other:
- L-24 conjecture tier = RUNTIME memory inside .elai_cc for the EXPLORE lane: unproven conjectures about
  the world/problem, statuses computed off the SSB journal, retrieval into L-23 explore packs. Rust,
  journal-grounded, EX-5 lane-walled.
- The engine = HARNESS-side, plan-AUTHORING-time: conjectures about ELAI ITSELF (improvement ideas),
  living in workflow/reasoning/ journal artifacts, consumed by paper-to-plan phase-5 / idea-intake.
  No SSB, no Rust store required — the run artifact IS its journal.
The convergence is deliberate (both come from the DeepReason collab) and worth one sentence in the plan's
Context; the boundary is that the engine NEVER writes into the L-24 store or the case graph, and L-24
never feeds candidates into plan authoring. If a future unit wants to bridge them, that is a new plan,
not a silent extension.

## 2. Design conclusions from L-24 that transfer directly (adopt these, they were red-teamed)

- FALSIFIER = THE TEST SPEC, one field not two. In stage 1 the candidate's falsifier ("the kill-test that
  would reject it") and in stage 3 the micro-experiment are the SAME object at two lifecycle points:
  stage 1 declares it, stage 3 executes or estimates it. Don't design two unrelated fields — L-24 learned
  this from L-22: a declared missing test IS an unrun falsifier. A candidate whose falsifier cannot even
  be SPECIFIED as a runnable check or a paper-grounded estimate is the forbid-nothing case: auto-reject
  (demarcation, HG-5 pattern).
- LOSERS ARE NOT FALSE — use status vocabulary, not a boolean. L-24's computed statuses transfer to the
  run artifact: winner / refuted (falsifier fired or ledger kill) / pending (testable but untested this
  run) / suspended (its premise or its killer changed). "Orphaned != false" applies: a candidate killed
  because it depended on a unit that later ships differently is suspended, not refuted.
- EVERY KILL CARRIES ITS KILLER (the N-16 S4 edge, harness-grain). A candidate killed by a ledger entry
  records WHICH entry; killed by a judge pass records the judge identity+version; killed by a micro-
  experiment records the run artifact hash. That is what makes "re-check losers when the ledger changes"
  MECHANICAL: a later session greps the artifact for kill-edges pointing at the changed entry and
  reinstates those candidates to pending. Without the edge, re-checking is vibes. This is the single most
  valuable transfer from the conjecture-memory work — put it in the artifact schema.
- ANTI-LAUNDERING: no candidate reaches "winner" without an EXECUTED stage-3 leg (a run micro-experiment,
  a grounded estimate with a citation, or a judge pass with receipts). L-24's ATTACK-3 (status laundering:
  live-by-never-being-criticized) applies verbatim — an untested candidate can rank, but it ranks as
  pending and cannot be the pick. If autonomous mode must pick anyway, the artifact records PICKED-PENDING
  explicitly (that is an honest state, silently promoting it is not).
- RECORD MOVES, NOT SURVIVORS (L-24 leg D / FD-1 transferred). The run artifact keeps ALL candidates, all
  bins INCLUDING EMPTY BINS ATTEMPTED, all kill reasons, all scores — not just the winner. The losers and
  the empty-bin attempts are the valuable data (they are the negative case law and the coverage proof).
  A run artifact that only shows the winner fails its own purpose.
- DIVERSITY IS STRUCTURAL, BANS LIVE AT THE GATE (S3/EX-1, measured not asserted). MAP-Elites empty-bin
  forcing + stance rotation + VS tail-weighting are the levers; NEVER a "be different / avoid X"
  directive (measured placebo), and NEVER render ledger-rejected content into the generation prompt to
  say "don't propose this" (it primes exactly what it bans) — the ledger kill happens at stage 2, after
  generation, deterministically. Your stage ordering already encodes this; keep it and say why.
- JUDGE SEPARATION IS THE HOUSE INVARIANT, cite it, don't re-argue it: the generator never scores its own
  candidates (D-6 execution/evaluation separation at orchestration grain; wiring R8 in spirit); if the
  judge is an LLM, the I-24/N-15 order-swap + verbosity-penalty audits apply; scores are ADVISORY ranking
  (TD-3) — the deterministic stage-2 kills are the only hard rejections.

## 3. Verified anchors from this session (dated 2026-07-11; re-verify only what's marked stale-prone)

- proof_spine_status.py output: PS-1 RUNNER_PRESENT (G-6), PS-2 RUNNER_PRESENT (K-4), PS-3 PLANNED (L-10),
  PS-4 PLANNED (K-2), PS-5 RUNNER_PRESENT (K-18), PS-6 MEASURED. Note the tool exits 1 (advisory).
- Open letter: Q is the newest letter with a 0_INDEX (Q-1..Q-9, authored 2026-07-10, orchestration-
  economics + engineering-taste + distribution). plan-standard.md's "current OPEN letter (P as of
  2026-07-10)" is stale on its face — treat Q as open. Precedent you can lean on: L-24 landed in L (not
  the open letter) on TRACK AFFINITY, explicitly blessed; the 07-08 session did the same for L-22/L-23/
  N-16. Track affinity is a legitimate router when a track owns the theme.
- Live SSB baseline: 581 variants / SCHEMA_VERSION 272 DERIVED at .elai_cc/crates/orchestrator/src/
  ssb.rs:1530-1544 (SCHEMA = -309 + len(SSB_ACTION_TYPES); do not hand-count). STALE-PRONE — re-read at
  edit. Only relevant if your placement decision ships any Rust leg (it probably shouldn't; see §4).
- Feature map regions (grep-confirmed this session): header upsert line = line 4, convention is appending
  "; upserted <date> (<one-line>)" — my session used "2026-07-11b" because an earlier 07-11 upsert
  existed. VCG block ~1732-1760 (J-96..J-98), CONJECTURE-MEMORY MIDDLE TIER entry now directly after it
  (added this session). K-19 SF-1..SF-4 doctrine ~1791-1809. I-30/I-31 ~1711-1727 (now carrying "Pre-gated
  by L-24 FD-1..FD-3" clauses). L family line ~3452 (now L-1..L-24). Section-4 DEEPREASON block ~4880+
  now ends with DEEPREASON-COLLAB VERDICTS C1-C4 (C1 = the L-24 tier; your engine is NOT one of C1-C4 —
  it is a new steal-adjacent unit, add your own verdict lines when you upsert).
- Ledger overlaps I already checked while authoring L-24 (re-confirm the ones your stages touch): the
  REJECTED/ALREADY-TRIED ledger starts at feature map SECTION 4 (~line 3658 pre-my-edits; grep
  "SECTION 4 — REJECTED"). Entries that constrain YOUR stages: "LLM stop-gate REJECTED (TD-3)",
  "MCTS-first search REJECTED; search is a LADDER best-of-N ->" (your stage 1/3 is a best-of-N ladder,
  aligned), "cargo-green as a done-gate REJECTED". I did NOT exhaustively check idea-framer/E-sweep/DGM
  overlaps — that check is yours per your prompt's protocol step 2.

## 4. Placement-call facts (the call is yours; these are the constraints I hit)

- The engine is harness tooling: a stage inside two SKILLS (.claude/skills/paper-to-plan/SKILL.md,
  idea-intake) + probably one deterministic tool (.claude/tools/, the binner/ledger-checker/artifact
  writer). Skill files and .claude/tools are NOT plan files — the never-amend-old-plan-files rule does
  NOT block editing them in place (it blocked me from amending L-23/D-17/I-31 plan files; it says nothing
  about skills). So "skill amendment + a wargame-shaped spec doc" is a legal fewest-files shape.
- BUT plan_gate_check.py must PASS on whatever document carries the plan (your prompt requires it), and
  note its code-plan detector: ANY mention of ".py", ".rs", "src/", "crates/" or a rust/python/ts code
  fence flips is_code_plan=true, which activates P9 (needs a PRODUCTION_HOOK or WIRING_CONTRACT marker)
  and P10 (needs INTEGRATION_SMOKE or "INTEGRATION_SMOKE: n/a"). A workflow-side spec doc that names
  .claude/tools/*.py WILL be treated as a code plan — carry the full marker set regardless of placement.
- If you author a letter file: Q on open-letter grounds, or argue track affinity if a track genuinely
  owns "harness self-improvement discipline" (none obviously does — J owned harness residue and is
  CLOSED; a J-parent EXTENSION lands in the open letter cross-linked, per the CLOSED-families rule).
- Precedent for gates vs advisory: if your stage-2 deterministic checker is ever cited as a DONE-gate it
  must be frozen in .claude/gate-manifest.txt (R8). As designed it is advisory triage (kills duplicates
  before scoring) — keep it advisory (TD-3) and say so, and you avoid the freeze machinery entirely.

## 5. PROOF_SPINE lean (PSR-2; you must still declare and, if none, STOP for the ack)

My grounded lean: `none (discipline)`. PS-5 (case-reuse compounding, K-18) measures RUNTIME reuse
economics of proven fix-cases on recurring bugs — the engine improves plan-authoring idea quality, which
feeds no measured spine number; declaring PS-5 would be vibes (PSR-2 bans that). A defensible alternative
nobody has claimed: the engine's ledger-recheck loop is upstream of EVERYTHING but measured by NOTHING —
that is the definition of discipline-class. Ack precedents to cite in your ask: L-23's S2 leg (none,
acked 2026-07-08), L-24's leg D (none, acked 2026-07-11), Q-8/Q-9 (none, owner-delegated ack 2026-07-10).
Expect the user to ack; frame the question with the recommended option first.

## 6. Bindings to cite (compose, don't restate — the misses that would cost you an audit cycle)

- N-2 composition validator (N-plan): your stage-2 "does this candidate fork a canonical subsystem" check
  IS N-2's question at candidate grain. COMPOSE it — name N-2 as the machinery and scope your check as a
  pre-N-2 triage (cheap lexical/feature-map pass at candidate time; N-2 proper runs at plan-authoring
  time on the winner). A second composition checker would itself violate N-2.
- K-19 SF-1..SF-4 (self-improvement fitness doctrine) BINDS "any future self-improvement / playbook-
  curation / memory-reuse" work — the engine is self-improvement-adjacent (it curates improvement ideas).
  SF-2 (stronger-than-generator judge) and SF-3 (observe-first; never-into-the-auditor: the engine's
  artifacts must never be fed to the auditor judging the resulting plan) and SF-4 (never-delete losers,
  quarantine) apply cleanly. SF-1 (execution-grounded fitness) applies to stage 3's preference order:
  a run micro-experiment outranks a judge score, always.
- H DGM lane (H-43..H-48) owns CONFIG/CODE mutation with the un-gameable-filter theorem; your prompt
  already demands the boundary statement (engine = DESIGN/PLAN-level ideas, output goes through the
  human/plan-gate pipeline, never auto-applied). Add the symmetric clause: the DGM lane must not start
  proposing plan-level ideas either — one owner per grain.
- B-40 idea-framer + critic loop is SHIPPED and is idea-intake's existing front stage. The engine slots
  AROUND it (diverge/filter/test/choose), it does not replace the framer; check the B-40 feature-map
  entry before writing the idea-intake integration so you name the real seam.

## 7. plan_gate_check.py mechanics (exact; cost me one FAIL cycle on L-24)

Read .claude/tools/plan_gate_check.py once — but the shape: markers must sit at LINE START (an optional
markdown heading, list bullet, or ** bold prefix is stripped first). P1/P2: ON_FAIL count >= EXPECTED_
OBSERVATION count (pair them per move). P3: at least one FORK line or a literal "FORKS: none". P4: at
least one RECON_NEEDED, or "RECON: none open", or a "## GROUNDING" heading; the string "UNGROUNDED"
anywhere without any RECON_NEEDED = FAIL. P5: ABORT. P6: a VERIFY or VERIFICATION marker AND a runnable-
looking line somewhere AFTER it (python/cargo/git/backtick-command etc.) — my L-24 draft failed exactly
this; the fix was a "VERIFY (...):" block with the runnable commands. P7: REDTEAM (RED-TEAM also matches).
P9/P10 per §4 above. Exit 1 on any fail, 2 on scan-config error (fail-closed).

## 8. If you author a letter file — the L-24 index-update checklist (the convention, condensed)

1. <X>-0_INDEX.md: title session-count + track list; a new "Date:" line; source-synthesis entry; the
   track "Why" paragraph extended (or a new track paragraph); any new delta rules (only if earned, R-j);
   sessions-table row + indented gloss; constants-table row quoting the LIVE SSB baseline with "re-read
   at edit (W-9)"; a dated SESSION LOG entry at the end.
2. <X>-SUMMARY_INDEX.md: title count, track premise line, one full entry (Track|lever|grounding|nature|
   effort|WAVE|depends; Steal/build; COMPOSES-WITH; Integration; observable_effect with the NEGATIVE arm).
3. Feature map: header line-4 append; the plan-family line; a capability entry in the right subsystem
   group with composes/extends links; Section-4 verdict rows for this run; any pre-gate clauses on
   entries your plan binds.
4. Spec overview: one design-language clause in the letter's section (NO plan-ids, NO status words).
5. The source reasoning file gets an OUTCOME block; the handoff gets INDEX line + entry + CURRENT STATUS
   overwrite.
Known drift to not trip over: I found and repaired L-SUMMARY_INDEX's stale title this session ("21
sessions", no L-T5 in header, while the L-T5 body existed) — trust the body, verify the headers, and
check your target index's title actually matches its contents before mirroring it.

## 9. Session gotchas (mechanical, save yourself the retries)

- ELAI-FEATURE-MAP.md exceeds the Read token cap even for small offset windows near the huge header line;
  use Grep with -n to locate, then Read tight offset/limit ranges, and PowerShell Substring for the
  header line's tail when you need to Edit-anchor on it.
- Edit anchors: file line-wrapping on disk differs from what a prior session's read suggested (the
  handoff GROUNDING block differed by one word from its own earlier read; the steal file wraps
  mid-sentence). Anchor Edits on the exact tail line you just read from disk, not on remembered prose.
- proof_spine_status.py exits 1 even on a healthy advisory read — don't treat the exit code as an error.
- The AskUserQuestion for the none-ack: recommended option first, cite the ack precedents (§5), and put
  the honest "declaring a spine feed would be vibes" reasoning in the question body — that is what got a
  clean one-click ack this session.

End of support notes.
