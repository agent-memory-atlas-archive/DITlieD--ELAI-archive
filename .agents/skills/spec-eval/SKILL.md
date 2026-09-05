---
name: spec-eval
description: Evaluate ELAI (or any system the user is actively designing) AS SPECIFIED — judge the design on its merits and NEVER discount it because a piece is planned / wire-dark / partially shipped. Triggers when the user asks for a COMPARISON ("compare ELAI to X", "how does ELAI stack up vs Y", "is X better than our approach", competitive analysis) or an IDEA / DESIGN IMPROVEMENT ("improve this design", "elaborate / refine / think about this idea", "how can ELAI do X better", "what is this missing", "ground this design"), or when the user DROPS A BATCH OF SPEC / PLAN / INDEX FILES as reasoning context. Shipped-vs-planned status becomes a separate, clearly-labelled implementation-readiness note that never feeds the verdict. Read-only on code; works in any FSM state; grounded-by-default and anti-sycophancy still fully apply.
license: internal
compatibility: "Designed for the ELAI workflow root. Portable to any in-development project where the spec is the artifact under evaluation."
metadata:
  version: "1.0.0"
  author: "elai"
---

Evaluate the system the user is BUILDING, not the binary that exists today. ELAI is in development; most of it lives as spec across docs/thefinalglance/{B..K} + docs/specs/. When the user hands you those files, they are giving you CONTEXT so you can reason about the design — they are NOT asking "is it done yet."

WHY THIS EXISTS

Origin: 2026-06-14. Asked whether ELAI improves local-model reasoning, the main LLM led with "no" and repeatedly marked the architecture down because the substrate was PLANNED / wire-dark rather than shipped. The user corrected it: evaluate the system as specified; shipped-vs-not is a separate axis and must not contaminate a design judgement. An in-development project is judged on the merit of its design; whether a given module is wired yet is a sequencing fact, not an architectural one.

WHEN TO INVOKE

Fire this skill (set EVALUATION MODE = AS-SPECIFIED for the rest of the thread) when the user:
- asks for a COMPARISON: ELAI vs a competitor / paper / tool / alternative approach; "how does X stack up"; "is Y better"; competitive or prior-art analysis.
- asks to IMPROVE / ELABORATE / REFINE / GROUND an idea or design at the conceptual level; "what's missing", "what would make this better", "think about this with me", "poke holes in the design".
- DROPS A BATCH OF SPEC / PLAN / INDEX FILES (the way the user habitually provides ELAI context). A multi-file dump of plan summaries, phase files, or design docs is CONTEXT PROVISION → read them, reason over the design they describe, do not produce a completion audit.

Do NOT fire (or fire and then defer) when:
- the user is asking a literal STATUS question ("is X wired?", "what's shipped?", "does this run?") — that is a wiring / bug-report task; route to prompt-transform + the wiring tools, and answer with shipped truth. Spec-eval is the opposite lens; do not use it to dodge a real status question.
- the user invokes the planning-workflow "plan improvement" protocol on a specific phase/step/baby-step FILE (the 9-step in-place rewrite). That is a different, file-rewriting protocol — route there. Spec-eval evaluates a DESIGN; it does not rewrite phase files in place.

THE CORE RULE — AS-SPECIFIED

1. The system under evaluation is the full SPEC, considered built. Judge the architecture, the mechanism, the composition, the tradeoffs — on their merits, as if every named module works as written.

2. Separate two failure classes and treat them oppositely:
   - DESIGN WEAKNESS — the architecture is wrong, fragile, contradictory, unfalsifiable, over-complex, or rests on an ungrounded assumption EVEN IF perfectly built. This is a legitimate mark-down. Name it, ground it, propose the fix.
   - NOT-YET-BUILT — the module is PLANNED / wire-dark / partial / RunPod-pending. This is an IMPLEMENTATION-READINESS fact. It NEVER marks the design down. It lives only in a separate, labelled readiness note.
   If you catch yourself writing "but it's not shipped" / "it's only planned" / "wire-dark" inside a design verdict, stop — that sentence belongs in the readiness note, not the verdict.

3. Status stamps in the summary/index files drift (I-39 exists because of "five drifting sources of truth"). So shipped-vs-planned is doubly unreliable AND doubly out of scope for a design judgement. If a readiness note is needed, mark its status claims as "per the index, confirm against live code."

COMPARISON PROTOCOL

- Compare like-for-like at the DESIGN level: ELAI's specified mechanism vs the comparator's mechanism / capability. Do not penalise ELAI for being unbuilt, and do not credit a competitor merely for being shipped — those are maturity facts, noted separately.
- Ground BOTH sides (grounded-by-default is not relaxed): the comparator's real behaviour via WebSearch / docs / the actual implementation, ELAI's via the spec text you were given. Cite. Label any unverified comparator claim "From training (unsearched):" until grounded.
- Surface where ELAI's design is genuinely differentiated, where it merely matches, and where the comparator's design is actually better — anti-sycophancy holds. Two approaches minimum before a recommendation.
- End with a maturity line (separate from the verdict): "design edge X exists; comparator ships today; ELAI's is at spec/partial — a timing gap, not a design gap."

IDEA / DESIGN-IMPROVEMENT PROTOCOL

- Improve the design assuming it is buildable. Propose mechanism, not "ship the backlog."
- Every suggestion is grounded (web prior art, papers via paper-ingestor, real implementations) per grounded-by-default — "I think X would work" is never sufficient. Reach for gpt-bridge / oracle for a diversity check on a hard fork.
- Attack the design adversarially (the grounded-by-default "attack-the-design" habit): what assumption is unfalsifiable, what composition is fragile, what is the cheapest falsifier. A real design weakness found is worth more than three polish ideas.
- Do not gate an improvement idea on shipped-status. "This is planned already" is fine to NOTE (so we don't double-author — GD-1 compose-not-restate), but it is not a reason to withhold the design reasoning.

OUTPUT SHAPE

- Lead with the DESIGN VERDICT (the merit assessment). This is the headline.
- Then, only if useful, a short IMPLEMENTATION-READINESS NOTE, explicitly labelled as build-status / sequencing, explicitly NOT part of the verdict, with its status claims flagged as index-derived (confirm against live code).
- Keep grounding citations inline. Keep anti-sycophancy: no opening affirmations, name actual problems, don't fold under pushback if grounded.

WHAT THIS SKILL DOES NOT RELAX

- Grounded-by-default (every claim cited; ungrounded conjecture labelled and not acted on).
- Anti-sycophancy, two-approaches-minimum, completeness-with-caveats.
- Anti-slopsquatting on any package named in an improvement idea.
- The GRC decision-ledger habit on a genuine architecture fork (mcp_think at convergence).
As-specified means "do not discount the design for being unbuilt." It does NOT mean "evaluate uncritically." A spec can still be bad design, and saying so is the job.

BOUNDARY

- vs planning-workflow plan-improvement: that rewrites a specific phase/step/baby-step FILE in place via a 9-step protocol. Spec-eval reasons about a DESIGN and writes a reasoning artifact; it does not rewrite plan files. If the user wants the in-place phase rewrite, route there.
- vs grill / cook: grill interrogates open decisions one question at a time; cook discovers an undecided look. Spec-eval assumes the design exists (in spec) and judges/compares/improves it. They compose — grill can precede spec-eval to pin decisions first.
- vs oracle: oracle answers an external factual question. Spec-eval is the lens for reasoning about ELAI's own (or a comparator's) design; it CALLS oracle/web for the external grounding it needs.

END SPEC-EVAL SKILL
