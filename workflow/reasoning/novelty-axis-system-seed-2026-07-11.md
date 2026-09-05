# The novelty axis has no owner — seed for a NEW system (2026-07-11)

Origin: the two conjecture-engine benches (conjecture-bench-run-2026-07-11.md + conjecture-novelty-ab-2026-07-11.md)
plus the DeepReason collab thread. User framing this file exists to serve: "we have GRC/CGG but we
need a new system for novelty, to reason toward and find new/better options."

## 1. THE MEASURED SITUATION (what is proven, what is null, what is untested)

PROVEN (correctness axis): grounding machinery buys correctness, large and un-gameable.
- engine real-anchor rate 7/10 vs 2/10 freestyle (3.5x, deterministic DEF-symbol check, pilot P-A)
- CGG's own measured deltas (user's bench box): code 27%->9% wrong, math 22%->~3%
- 8/10 freestyle "best" ideas name FAKE seams — invisible to any no-file judge, caught only by
  the deterministic anchor check.

NULL (novelty axis, single-shot regime): the engine's structural diversity machinery (stance
rotation + empty-bin forcing WITHIN one run) added nothing measurable over one "propose 10
DISTINCT ideas" freestyle call: M2 cluster-count delta -0.7 CI[-1.8,+0.25]; M3 unique-mechanisms
symmetric (6.5 vs 7.0). Generation was NOT reduced — sets equally diverse — but nothing gained.

NEGATIVE (pick stage): receipts-only ranking picks the checkable-now idea; engine picks lose
blind pairwise novelty 13-27. Designed-in conservatism, predicted pre-run. The judge-bet "value"
loss (8-31) is mostly instrument blindness to groundedness, but the novelty loss is real.

UNTESTED: multi-cycle regimes (fade-fighting rotation, DeepReason F3's actual home turf),
archive-coupled novelty targeting, execution-grounded value of picked ideas, CGG ON/OFF novelty
ablation (L-24 leg (a), queued at L-10).

## 2. THE AXIS SPLIT (the doctrine this file pins)

GRC/CGG and the whole receipt plane own CORRECTNESS: is a claim grounded, is an anchor real, is
a step committed on evidence. They are commitment-boundary mechanisms; measured effect on the
idea pool: zero either direction (sets equal). They must NEVER be blamed for conservatism at the
pick — that was a selection-policy choice (fusing receipts into the only ranking signal).

NOVELTY currently has NO owner system. Everything in-corpus that touches it is DEFENSIVE
(anti-stagnation: escape a basin you are stuck in) or diversity-as-side-effect:
- anti-spiral personas / stance rotation (C-10) — fires on stagnation, not toward novelty
- I-4 best-of-N + L-22 VS tail-weighting — spread + typicality tail, fires under stagnation flag
- I-8 dead-end hashing + S2 gate-block orbit detector — bans re-proposals, adds nothing new
- problem turnover (I-8/I-26) — abandons dry problems, does not aim the next one
- E-7 MAP-Elites — in-corpus, but the bench only exercised its single-shot prompt-trick form
- L-23 explore profile — maps rival explanations for ONE question; no cross-run novelty objective
- L-24 conjecture memory — preserves unproven ideas; storage, not generation
No unit's OBJECTIVE is "produce options measurably newer/better than the archive, repeatedly."
That is the gap. (N-2 check: this is a missing owner, not a fork — the pieces above become its
substrate, composed not restated.)

## 3. WHAT THE MEASUREMENTS TEACH THE DESIGN (constraints, each with its receipt)

C1 WORLD-COUPLED OR NOTHING. Within-model structural moves added zero single-shot diversity
   (novelty A/B M2/M3). The only proven generate-lever in both benches and in DeepReason's own
   data is injecting bits the model never had: real-anchor verification, executed falsifiers,
   retrieval, cross-family. A novelty system that only re-prompts the same model differently is
   a placebo by our own measurement and by his (complement directives = placebo, F3 report).
C2 NOVELTY IS MEASURED AGAINST AN ARCHIVE, NEVER AGAINST THE PROMPT. Single-run bins are
   saturated in one call (bench artifact: 10/10 bins FILLED immediately). The reference set must
   be persistent and outside the run: feature-map ledger + plan corpus + L-24 conjecture store +
   case graph. "New" = a mechanism with no equivalent in the archive (NLI meaning-cluster
   distance, L-4/F-12 machinery — never cosine, never n-gram; both failed as instruments).
C3 MULTI-CYCLE IS THE REGIME THAT MATTERS. Single-shot is where the null lives; rotation's
   measured win (DeepReason F3, late/early 0.973 vs 0.846) is across many cycles under fade.
   MAP-Elites in the QD literature is an ARCHIVE algorithm across iterations, not a one-call
   prompt trick [VERIFY at authoring: QD/novelty-search prior art cites — Mouret&Clune
   MAP-Elites, Lehman&Stanley novelty search; from training, unsearched].
C4 NOVELTY AND GROUNDEDNESS NEVER SHARE A KNOB. Fusing receipts into the only ranking signal
   provably buys conservative picks (13-27). Selection must be Pareto/multi-axis: (novelty-vs-
   archive, groundedness, expected value), reported un-fused. DeepReason's Pareto retention over
   (hard-to-vary, reach, coverage) is the prior art shape.
C5 VALUE IS DECIDED BY EXECUTION, NEVER BY JUDGE BET. The 8-31 "value" loss came from an
   instrument structurally blind to fake anchors. The value leg of any novelty bench is: author
   both picks, run the real wire/gate checkers, or execute the falsifier (pilot P-A pattern).
C6 NOTHING NOVEL IS DELETED. A novel-but-unproven candidate goes to the L-24 store as pending
   with provenance + missing tests, retrievable ahead of the receipt plane ("clean but sterile"
   fix, already authored). The novelty system's losers are its own future seeds.

## 4. THE SYSTEM SKETCH (what a plan-authoring session would build)

Working name: NOVELTY ENGINE (offensive novelty axis owner). One loop, world-coupled, archived:

  1 TARGET  — read the archive (feature map + L-24 store + case graph + bench gaps): compute the
              persistent bin matrix (mechanism family x target area x problem class) and the
              EMPTY/THIN regions weighted by declared value (proof-spine feeds, open limits).
              The target is a REGION THE ARCHIVE LACKS, not "be different" (C2, S3 doctrine).
  2 INJECT  — world-coupled conditioning for the diverge: retrieval on the target region,
              a real artifact (code slice, bench number, paper claim via paper-ingestor),
              cross-family second generator where available (C1; D-13).
  3 DIVERGE — candidate cards (the engine's existing card grammar) aimed at the empty region;
              VS typicality tail kept (cheap, composes; L-22 S5).
  4 SCORE   — three UN-FUSED axes per candidate (C4): novelty-vs-archive (NLI meaning-cluster
              distance to nearest archive neighbor), groundedness (triage + anchor check, the
              proven leg), expected value (declared measurable effect + proof-spine feed).
  5 SELECT  — Pareto front, not argmax; the pick policy is caller-declared (a plan-authoring
              caller may want groundedness-first; an exploration caller novelty-first).
  6 VALIDATE— execution-grounded (C5): compile the falsifier, run the cheap leg now; the
              expensive leg becomes a RECON_NEEDED in the authored plan.
  7 PERSIST — every candidate (winner AND losers) lands in the L-24 store with status +
              kill-edges; the archive grows, so step 1's bin matrix moves (C6). Fade/orbit
              detectors (S2/C-10) guard the loop across cycles (C3).

Boundary statements (anti-fork, N-2): the improvement-conjecture engine KEEPS its identity as
the grounding machine (its bench claim is anchors, not creativity); the novelty engine extends
its stage-1/stage-4 rather than replacing it. L-23 explore profile is the within-one-question
map; the novelty engine is the cross-run option-finder. L-24 is its memory, not its competitor.
GRC/CGG untouched — correctness axis stays commitment-boundary.

## 5. THE SYSTEM'S OWN FALSIFIER (it gets the same treatment it was born from)

Bench: the existing novelty A/B harness (conjecture_novelty_ab.py) extended to the multi-cycle
regime: N cycles per trigger, archive persisted between cycles, freestyle arm given the same
cycle count without the archive. Kill condition: at K>=30 triggers, if the novelty-engine arm's
archive-relative novelty AND execution-validated value do not beat freestyle-with-memory, the
system is refuted and the file gets a REFUTED banner (SF-4; a null shipped as a null — exactly
like the run that produced this seed).

## 6. NEXT STEP

A plan-authoring session (spec-eval AS-SPECIFIED + plan-standard wargame shape + plan_gate_check)
takes this seed: feature-map ledger check first, PROOF_SPINE declared honestly (plausibly PS-3
via the explore lane, else none needs user ack), landing zone per CLOSED-families (L-T5 track is
the natural home beside L-22/L-23/L-24). RECON_NEEDED at authoring: verify the QD prior-art
cites (C3), confirm L-4/F-12 NLI machinery status, re-grep the engine's stage anchors.
