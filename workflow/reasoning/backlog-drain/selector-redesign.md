Selector redesign: full-corpus supersession-aware ordered drain for the TEIKOKU opus walk

Analyst: read-only systems pass, 2026-07-16. Read-only on code (no code mutated, no walk/git commands run).
Every claim is anchored to a file:line under \\wsl.localhost\teikoku\home\teikoku\teikoku\conductor\opus_walk\
(the ENGINE) and F:\.ELAI_workflow\docs\thefinalglance\ + workflow\reasoning\backlog-drain\ (the CORPUS).
Engine paths below are written distro-relative (opus_walk/plan_selector.py == the UNC file I read).

===============================================================================
0. THE DIAGNOSIS, GROUNDED (what the walk does today, and why it starves the corpus)
===============================================================================

Pool build, one line (plan_selector.py:344):
  pool = punch + summary_pool
where
  punch      = eligible_gaps(status["punch"]) with dispositioned ids stripped (plan_selector.py:298-299).
               status["punch"] is the live `elai_status_query.py --punch --json` head (read_status,
               plan_selector.py:366-385) -- a tiny CURATED list, not the corpus.
  summary_pool = (audited + recon)[:SUMMARY_POOL_MAX]  (plan_selector.py:343), enumerated by
               summary_candidates() (plan_selector.py:227-246) which globs one *SUMMARY*.md per letter
               under docs/thefinalglance/{B..Q}-plan/ (PLAN_LETTERS, plan_selector.py:99) and parses
               `### <id>` headers (_parse_summary_file, plan_selector.py:196-224).

Then the free blind quorum votes over only the TOP-5 of that pool (quorum_select ballot,
plan_selector.py:547 `ballot = eligible[:SELECT_BALLOT_MAX]`), and select_next returns ONE winner.

THE EXACT CAPS / STALENESS CONSTANTS (file:line):
  - SUMMARY_POOL_MAX              = 40    plan_selector.py:109   (summary tail cap; punch sits on top, uncapped)
  - SELECT_BALLOT_MAX            = 5     plan_selector.py:83    (quorum only ever SEES 5 candidates)
  - SELECT_QUORUM_MIN           = 2     plan_selector.py:91    (fail-closed floor on parseable votes)
  - AUTOSEED_MAX_SNAPSHOT_AGE_MIN = 180.0 plan_selector.py:56    (the staleness halt; 27h == 1620 min)
  - RESEED_LIMIT                = 2     plan_selector.py:44    (per-plan auto-seed cap)
  - AUTOSEED_MAX_PER_CHAIN       = 8     opus_supervisor.py:105 (per-chain auto-seed backstop)
  - QUORUM enumerated letters     "BCDEFGHIJKLMNOPQ" plan_selector.py:99

WHY THE CORPUS STARVES (three independent, grounded causes):
  (a) STALENESS HALTS BEFORE ENUMERATION. select_next reads status, then at plan_selector.py:629
      `if age > AUTOSEED_MAX_SNAPSHOT_AGE_MIN: return None` -- it HALTS THE WHOLE SELECTION on a stale
      snapshot, BEFORE build_pool (called at plan_selector.py:637) ever runs the summary enumeration.
      So with a 27h-stale punch, the walk refills NOTHING even though the summary tier does not depend
      on snapshot freshness at all. The 27h snapshot single-handedly disables corpus refill.
  (b) 40-OF-394 CAP. Even when fresh, summary_pool is capped at 40 (plan_selector.py:343) out of
      `summary_total_eligible` ~394 (the stat at plan_selector.py:350; observed in the supervisor log
      string plan_selector.py:640-644). The dark J tail (J-31..J-112, ~80 units) sits below the cap and
      never enters the ballot unless a synthesis names it (synthesis_named_ids, plan_selector.py:257).
  (c) THE BALLOT IS 5. The quorum votes over 5 (plan_selector.py:547). So the "pool" of 40+ is a
      priority-ORDER, and only its head-5 is ever a real candidate. Refill is one-plan-at-a-time by a
      blind vote over 5, not a drain of the corpus.
  Plus: NOTHING enforces supersession. summary_candidates carries only {id,desc,files,status} from the
  `### ` header bracket (plan_selector.py:222-223); there is no supersedes/superseded_by anywhere in the
  selector, so a naive pick can target a fully-superseded dark unit (e.g. I-30, superseded_by J-50) or
  re-author a foundation a newer unit already replaced (the reversion-risk edges in the cards).

===============================================================================
1. CORPUS ENUMERATION -- replace the capped punch+SUMMARY pool with a complete, delivery-aware index
===============================================================================

RECOMMENDED ENUMERATION SOURCE: the analyst CARD INDEX, workflow/reasoning/backlog-drain/cards-*.md
(cards-BCD.md, cards-EFGH.md, cards-IJKL.md today; cards-MNOPQ.md as the analysts finish), NOT the
per-letter SUMMARY indexes and NOT the 27h-stale punch.

WHY THE CARDS, GROUNDED (they already carry every field the drain order needs, machine-parseable):
Each card is one line of pipe-delimited fields (verbatim from cards-IJKL.md:the I-plan CARDS block):
  id: I-30 | does: verifier-ranked SFT exhaust ... | state: dark (disposition: SftExhaust absent...) |
  superseded_by: J-50 (recovery-trajectory extension) | residual: whole build; RunPod for the LoRA run |
  key_files: (planned) crates/cli/src/sft_exhaust.rs | respect: J-50 recovery extractor
So a card gives, PER UNIT: id, does, state in {wired|partial|dark|uncertain} (defined in the cards
header, cards-IJKL.md:legend), supersedes, superseded_by, residual, key_files, respect (the
don't-revert set). Plus a top-of-file SUPERSESSION_EDGES block (cards-IJKL.md: the "newer SUPERSEDES
older" section) that enumerates every cross-letter edge with file:line evidence (e.g. "J-20 SUPERSEDES
D-5, D-29, H-48"; "K-19 / H-43 SUPERSEDE the LIVE DGM GAP-1"). This is EXACTLY the supersedes[] /
superseded_by[] / state graph the ordering needs, and its state column is git-log + disposition +
filesystem grounded as of 2026-07-16 (cards-IJKL.md:analyst header) -- far fresher and truer than the
27h punch or the "I: 54 wired" feeder optimism the cards header explicitly flags as "drained-from-debt".

COMPLETE YET TRACTABLE: the cards are ALREADY the whole corpus, one line per unit (~394 lines total
across the files), so enumeration is "read 3-4 small md files and split on ` | `" -- uncapped by
construction and still tiny (a few hundred short lines, cheaper than the current 40-row SUMMARY parse).
No SUMMARY_POOL_MAX is needed because the cards are not a candidate BALLOT; they are the ordered drain
list (section 2). The blind quorum's tractability constraint (why SELECT_BALLOT_MAX=5 exists) evaporates
once selection is a deterministic ordered pop, not a vote.

STAYING COMPLETE AS THE CARDS AGE: overlay two LIVE, deterministic delivery signals the walk already
reads, so a card's baked-in state can never mislead the drain:
  - teikoku plan-status DONE rows: walk_state.is_plan_done / plan-status/{id}.json (walk_state.py:472-486)
    and quorum_triage._plan_done (quorum_triage.py:41-53) -- authoritative "this walk already landed it".
  - teikoku dispositions ledger: walk_state.dispositioned_ids (walk_state.py:454-469) -- already consumed
    by build_pool (plan_selector.py:295-296) to drop ruled-on ids.
  - (optional live oracle) `elai plan status` (I-39): the cards mark I-39 "partial (plan_skeleton.rs
    present) | residual: verify plan_status verb wired" (cards-IJKL.md:I-39), i.e. the I-39 verb is NOT
    confirmed wired -- so treat it as a FUTURE oracle, not a dependency. The SSB journal is the deepest
    live delivery truth but reading it is a Rust/CLI call; the cards already fold git-log state in, so
    the cards + plan-status + dispositions triple is the recommended live-enough source. RECON_NEEDED to
    promote I-39: grep plan_skeleton.rs for a `plan status` CLI arm before wiring it as the oracle.

===============================================================================
2. SUPERSESSION-COLLAPSED ORDER -- the concrete ordering algorithm + where it lives
===============================================================================

WHERE IT LIVES: a NEW sibling module opus_walk/corpus_drain.py (stdlib-only, mirrors plan_selector's
posture). Rationale (maintainability-review dim 0/6): plan_selector.py is already ~660 lines and owns
the punch+quorum path; bolting a card-parser + graph sort onto it pushes it toward the 1000-line
boundary and mixes two concerns. corpus_drain.py owns "parse the cards -> produce the ordered drain
list"; plan_selector consumes that list (section 5). Keep the SUPERSESSION logic in the layer that owns
the concept, not leaked into build_pool.

DATA MODEL (corpus_drain.Unit): {id, letter, state, supersedes:set, superseded_by:set, residual:str,
key_files:list, respect:set, does:str, source_card_line:str}. Parsed by splitting each ` | ` field on
the first `: `; the SUPERSESSION_EDGES block is parsed as a second pass that UNIONS edges into the
per-unit supersedes/superseded_by (an edge "J-20 SUPERSEDES D-5" adds J-20->supersedes{D-5} and
D-5->superseded_by{J-20}), so both the per-card fields AND the cross-letter edge block feed one graph.

THE ORDERING ALGORITHM (deterministic, three stages):

STAGE A -- CLASSIFY each unit into an ACTION (no LLM, pure card + live-signal read):
  * plan-status DONE row present, OR state==wired with residual in {"-","",None}  -> DONE  (drop; nothing to do)
  * state==dark AND every id in superseded_by is CONFIRMED-SHIPPED                -> SKIP_SUPERSEDED
        (verify-then-skip: the residual is owned by the superseder; never build it)
  * state==partial, OR state==dark with an UNCONFIRMED/absent superseder          -> BUILD_RESIDUAL
        (build only the `residual` clause, never the whole unit if partial)
  * state==dark, no superseder                                                    -> BUILD
  * state==uncertain                                                              -> RECON_FIRST (settle
        via the card's named settling check before it becomes a build candidate; cards-IJKL.md legend
        already carries a per-uncertain settling check)
  CONFIRMED-SHIPPED (fail-closed, this is the safety of verify-then-skip): a superseder S is
  confirmed-shipped iff is_plan_done(S) is true (walk_state.py:472) OR S's own card state==wired. If S
  is itself dark/partial/absent, the superseded unit is NOT skipped -- it stays a BUILD candidate. This
  is the exact guard against "skip a dark unit because a superseder that never actually shipped supposedly
  owns it" (the reversion hazard). A SKIP_SUPERSEDED unit is auto-dispositioned drained-by-supersession
  (section 5) WITHOUT a paid turn.

STAGE B -- COLLAPSE residuals to their owner: for a BUILD_RESIDUAL / BUILD unit U, if any id in
  U.superseded_by is a BUILD/BUILD_RESIDUAL unit, U's residual is folded into that owner (the card
  `respect`/`residual` prose already says "residual lands in J-80" etc, cards-IJKL.md:I-2). Emit ONE
  drain item per OWNER, carrying the collapsed residual + the union of every folded unit's respect set.

STAGE C -- ORDER (foundations before extensions): topological sort over the EXTENDS/SUPERSEDES edges
  where an edge A->supersedes/extends->B means B is a FOUNDATION that lands (or is confirmed shipped)
  BEFORE A. Break topo ties by proof-spine precedence (PSR-1, .claude/rules/proof-spine.md; the SKILL
  already names PSR-1 as queue order, SKILL.md:36 "queue order: PSR-1 precedence") then by letter/number
  so the order is stable + reproducible. Cycles (mutual extends) are rare; break a cycle by the
  lower-letter unit first and log it. Output: an ORDERED list of drain items
  {id, action, collapsed_residual, respect_set, card} -- the head is the next thing to build, the
  SKIP_SUPERSEDED items are interleaved in id-order but consume no turn (auto-dispositioned on pop).

This is a deterministic pop, not a blind vote: the corpus order IS the decision (removes the
SELECT_BALLOT_MAX=5 blindness for refill; the quorum stays only as the optional cost-gate on whether to
spawn at all, opus_supervisor.py:746).

===============================================================================
3. PER-PLAN SUPER-SUMMARY INJECTION -- the card brief + the respect/don't-revert set into dispatch
===============================================================================

HOOK (file:fn): plan_selector._build_pickup (plan_selector.py:584-609). This function already builds
the exact first-action string that becomes cursor.pickup (set via walk_state.seed_plan, walk_state.py:266;
consumed by the Opus per SKILL RESUME step 5, SKILL.md:32 "cursor.pickup names the exact next action").
It is THE injection point: whatever _build_pickup returns is the brief the resuming Opus reads first.

SHAPE OF THE INJECTED BRIEF (append to the pickup string, ~6-10 lines, from the drain item's card):
  DRAIN CARD <id>: does=<does>; state=<state>; ACTION=<BUILD|BUILD_RESIDUAL|SKIP_SUPERSEDED>.
  RESIDUAL (build ONLY this): <collapsed_residual>.
  KEY_FILES: <key_files>.
  RESPECT / DO-NOT-REVERT (these newer units own or extend this seam -- read their state, never
  re-author or regress their path): <for each id in respect_set: "<rid>: <one-line does + state>">.
  SUPERSEDES (older units this one owns; if you touch them, you are consolidating not duplicating):
  <supersedes set with one-line does>.
This gives the Opus the unit's card PLUS the cards of everything that supersedes/touches it (the
"respect" set the analyst already computed, cards-IJKL.md:respect field) -- the exact reversion-guard the
prompt asks for. The brief is DATA the Opus reads before S1 recon; it does not replace S1/S2/S3 (SKILL.md
block 2), it seeds them so the S3 wired=yes/partial/no classification runs against the true supersession
picture. corpus_drain.build_brief(drain_item) is the helper; _build_pickup calls it.

SKILL touch: SKILL.md block 2 (PER-PLAN LOOP, SKILL.md:34-43) gains one sentence: "the cursor.pickup
carries a DRAIN CARD brief (card + respect set); treat its RESPECT list as the reversion-guard -- a
plan whose recon shows its residual is already owned by a RESPECT unit is drained-by-supersession
(disposition), not rebuilt." No structural SKILL change; the injection is data-through-pickup.

===============================================================================
4. STALENESS + DELIVERY-AWARENESS -- fix the 27h halt so refill doesn't starve or pick ghosts
===============================================================================

ROOT CAUSE (grounded): select_next halts the ENTIRE selection on a stale snapshot at plan_selector.py:629
(`if age > AUTOSEED_MAX_SNAPSHOT_AGE_MIN: return None`) -- but only the PUNCH tier depends on snapshot
freshness; the card/corpus tier does not. The fix is to SPLIT the freshness gate:
  - Stale/unreadable snapshot -> DEMOTE punch to empty (skip the punch tier), DO NOT halt; the corpus
    drain (cards + live plan-status/dispositions) still refills. Punch becomes a fresh-only bonus tier
    layered on top of the corpus order, exactly as it sits on top today (plan_selector.py:344) but
    no longer able to halt the corpus tier when stale.
  - Freshness is still REQUIRED for a punch-sourced pick (a punch gap is a live-status claim), so keep
    the age check but scope it to the punch branch, not the function head.
DELIVERY-AWARENESS without the stale snapshot: the corpus tier reads LIVE deterministic delivery state
directly -- plan-status/{id}.json DONE rows (walk_state.py:472) + dispositioned_ids (walk_state.py:454)
+ each card's git-log-grounded state -- so it never needs elai-status.json to know what shipped. GHOST
avoidance: STAGE-A classification drops DONE + SKIP_SUPERSEDED + dispositioned units BEFORE they can be
picked, so a fully-delivered or superseded unit is never seeded (the ghost class the prompt names).
OPTIONAL freshen (do not block on it): the SKILL preflight already runs
`elai_status_query.py --max-age-min 180` (SKILL.md:28) and notes "exit 3 = stale: refresh via the
feeder before triage" -- the supervisor can best-effort trigger the feeder (workflow/status feeder path)
once per chain before refill, but the redesign must NOT depend on it: the corpus tier is the halt-proof
floor. RECON_NEEDED: confirm the feeder trigger command/path (elai_status_feeder.py) before wiring an
auto-refresh; if absent, ship the split-gate alone (it already removes the halt).

===============================================================================
5. MINIMAL-CHANGE PATH -- exact functions to add/modify, reusing what exists
===============================================================================

NEW module opus_walk/corpus_drain.py (stdlib-only), functions:
  - parse_cards(cards_glob="workflow/reasoning/backlog-drain/cards-*.md") -> dict[id, Unit]
        (split ` | ` fields + a SUPERSESSION_EDGES second pass; fail-safe: a missing/garbled card file
        contributes fewer units, never raises -- mirrors summary_candidates posture, plan_selector.py:227).
  - classify(units, is_done_fn, dispositioned) -> list[DrainItem]   (STAGE A + B above; reuses
        walk_state.is_plan_done + walk_state.dispositioned_ids -- NO new done-gate logic).
  - order(drain_items) -> list[DrainItem]   (STAGE C topo + PSR-1 tiebreak).
  - build_brief(drain_item) -> str   (section 3 injected brief).
  - ordered_drain(...) -> list[DrainItem]   (parse->classify->order convenience wrapper).

MODIFY (5 functions, all in files already in the walk, no new frozen artifacts):
  1. plan_selector.build_pool (plan_selector.py:273) -- add the corpus tier: when punch is stale/empty,
        source candidates from corpus_drain.ordered_drain() instead of summary_candidates (plan_selector.py:307);
        the corpus order replaces the (audited+recon)[:SUMMARY_POOL_MAX] slice (plan_selector.py:343).
        SUMMARY enumeration (summary_candidates) can stay as a fallback for a letter with no card yet.
  2. plan_selector.select_next (plan_selector.py:612) -- SPLIT the age gate (plan_selector.py:629): on
        stale, set a punch_stale flag and pass it to build_pool (drop punch, keep corpus) instead of
        `return None`. Optionally bypass quorum_select for a pure corpus pop (take the ordered head),
        keeping quorum only for the punch tier.
  3. plan_selector._build_pickup (plan_selector.py:584) -- append corpus_drain.build_brief(drain_item)
        (the card + respect set, section 3).
  4. opus_supervisor._try_autoseed (opus_supervisor.py:328) -- handle the SKIP_SUPERSEDED action: when
        select_next returns a verify-then-skip item, call walk_state._emit_disposition(id, "drained",
        reason="superseded by <S>, confirmed shipped") (walk_state.py:330) + record it, WITHOUT seeding
        a paid turn, then re-loop for the next drain item. Reuses the existing disposition machinery;
        no paid boot for a skip. (This is the "verify-then-skip, never build" leg, done free.)
  5. SKILL.md block 2 (SKILL.md:34) -- the one-sentence DRAIN CARD note (section 3).

REUSE (touch nothing): walk_state.is_plan_done (walk_state.py:472), dispositioned_ids (walk_state.py:454),
_emit_disposition (walk_state.py:330), seed_plan (walk_state.py:266); quorum_triage stays as the
spawn-cost gate (opus_supervisor.py:746); the reseed cap + AUTOSEED_MAX_PER_CHAIN backstops stay as-is.

FROZEN-GATE TREATMENT (call-out): corpus_drain.py is HARNESS tooling, local-only, NOT product code and
NOT in .claude/gate-manifest.txt -- so it needs NO freeze and does not touch the frozen checkers
(production_caller_check.py / opus_land.py / etc, SKILL.md:41,90). One caution: the SKIP_SUPERSEDED
auto-disposition (modify #4) WRITES a terminal verdict into the walk's disposition ledger without a paid
verification, so its CONFIRMED-SHIPPED guard (STAGE A, fail-closed) is the load-bearing safety -- it must
require is_plan_done(S) OR card-state==wired for the superseder, never skip on an unconfirmed superseder.
That guard is the only "gate-grade" logic in the new module; it should get a unit test asserting a
dark-unit-with-a-dark-superseder is classified BUILD (not SKIP), i.e. the walk never skips real work on a
superseder that never shipped. opus_land.py and every product-side gate are untouched.

REDTEAM (attacks tried against this design):
  - ATTACK "a card lies (says wired, actually dark)" -> the corpus tier overlays live plan-status +
    dispositions, and BUILD/RESIDUAL still runs full S1/S2/S3 recon (SKILL.md block 2) which is
    filesystem-grounded; a wrong "wired" card only DROPS a unit (DONE), and the standing-goals watchdog
    (standing_goals.py:219 verify_all) re-seeds a regressed drop -- so a lying card fails safe toward
    "didn't build" which recon catches, not toward reverting shipped code. SURVIVES.
  - ATTACK "SKIP_SUPERSEDED skips a unit whose superseder never shipped" -> blocked by the fail-closed
    CONFIRMED-SHIPPED guard (is_plan_done OR card-wired); an unconfirmed superseder keeps the unit as
    BUILD. SURVIVES (this is why the guard, not the card alone, decides the skip).
  - ATTACK "cards-MNOPQ.md not authored yet -> M..Q never drain" -> summary_candidates fallback
    (plan_selector.py:227, retained) still enumerates those letters from their SUMMARY index until their
    cards land; the corpus tier is additive, not a replacement that blanks uncarded letters. SURVIVES.
  - ATTACK "topo cycle wedges the order" -> STAGE C breaks cycles by lower-letter-first + logs; the
    order is best-effort, a mis-ordered pair just builds a foundation one slot late, never halts. SURVIVES.

VERIFY (off-agent, how a reviewer confirms the redesign without trusting this doc):
  - unit test corpus_drain.classify: dark+dark-superseder -> BUILD; dark+wired-superseder -> SKIP_SUPERSEDED;
    partial -> BUILD_RESIDUAL; wired+no-residual -> DONE. (deterministic, no network.)
  - unit test select_next with a stale snapshot + injected corpus -> returns a corpus pick, NOT None
    (proves the 27h halt is fixed); with a fresh snapshot -> punch still on top.
  - run plan_selector.select_next --dry (inject status_reader/dirty_reader/post_fn/disp_reader per
    plan_selector.py:612-615) over a fixture cards dir -> assert the ordered drain covers the dark J tail
    (J-31..J-112) that today's 40-cap excludes.
