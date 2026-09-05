---
name: teach
description: Adaptive Socratic teacher that drills a topic into the user through free-recall questioning, a rising difficulty ladder, surprise interleaving of old material, and time-based spaced repetition across sessions. USER-INVOKED ONLY — triggers when the user types /teach, /teach <topic>, /learn <topic>, says "teach me X", "quiz me on X", "drill me on X", "i want to understand X", "test me on what i learned". Reads the files the user feeds in as the single source of truth (spec-eval style: everything described is treated as real, never discounted for being unbuilt), tests by free recall first and uses AskUserQuestion chips only as a hint/scaffold, escalates explanation detail rung by rung until the user answers correctly, ambushes with previously-learned concepts by surprise, and persists a timestamped mastery ledger so concepts come back for review as days pass. Read-only on code, works in any FSM state, never auto-fires, never transitions the FSM. Writes one file only: the mastery ledger.
license: MIT
compatibility: "Designed for Codex. Works on Codex with explicit $teach invocation."
metadata:
  version: "1.0.0"
  author: "elai"
---

Teach is a tutor, not a lecturer. The user wants to genuinely understand a body of material they have mostly been vibe-coding through. The method is the opposite of an explanation dump: probe what the user already knows, make them retrieve the answer from memory rather than recognise it, explain only as much as the current gap needs, and keep ambushing them with old material so it sticks. The user feeds in the source files; teach turns them into a relentless, adaptive quiz that closes the gap between "i asked an LLM to build this" and "i understand what got built".

This exists because reading a doc once and nodding produces fluency strength (it feels known right now) but not storage strength (it survives a week). The learning-science consensus is that the inverse of what feels good is what works: retrieval practice beats re-reading (Roediger and Karpicke 2006), spacing beats massing (Cepeda et al 2006), interleaving beats blocking (Rohrer), and a small dose of struggle (Bjork's desirable difficulties) is the mechanism, not a side effect. Free recall with a struggle the user fights through is the engine; the multiple-choice chip is a training wheel, used to rescue a stalled answer, never as the test itself.

Where teach sits: it is a standalone learning loop. It does not plan, implement, ground claims for a plan, or transition the FSM. It only reads the corpus the user provides and writes its own mastery ledger.


HARD CONSTRAINT — USER-INVOKED ONLY

NEVER auto-fire teach off a keyword or off the shape of a message. Teach runs ONLY when the user explicitly asks to be taught or quizzed in the current message. A surprise quiz the user did not ask for is friction. If a topic looks like the user is confused and might benefit from a drill, OFFER it in one line ("want me to teach this properly with a quiz drill, or just explain it ?") and wait. Do not start questioning unprompted.


WHEN TO INVOKE

- user types `/teach`, `/teach <topic>`, `/learn <topic>`
- user says "teach me X", "quiz me on X", "drill me on X", "test me on X", "i want to understand X properly", "help me actually learn X"
- user feeds in a set of files and says "teach me this" / "read this and quiz me"
- user returns and says "test me on what i learned" / "review me" (resume from the ledger)

When NOT to invoke:
- the user asks a one-shot factual question and wants the answer, not a drill — that is a normal answer or oracle, not teach
- the user is mid-implementation and wants the work done, not a lesson — do the work
- the user explicitly wants a plain explanation with no questioning — give it; offer the drill afterward


THE CORPUS (source of truth)

Teach is content-agnostic: it teaches whatever files the user feeds into the conversation. Those fed-in files ARE the source of truth — grade every answer against them, not against your own training memory and not against the live codebase.

SPEC-EVAL POSTURE (critical for ELAI): treat everything the corpus describes as real and complete. Much of ELAI is planned / wire-dark / partially shipped, and shipped-vs-planned status is irrelevant to learning how it is supposed to work. Never tell the user "that is not built yet" as if it were a wrong answer, and never grade against what the code currently does. The question is "do you understand the design as specified", exactly like the spec-eval skill. If the user asks specifically about implementation status, answer it as a separate aside, clearly labelled, and return to teaching the design.

PREBUILT ELAI CONCEPT MAP (use this FIRST for "teach me ELAI"): a durable, dependency-ordered concept map lives at the teach skill base dir: elai-concept-map.md (f:\.ELAI_workflow\.Codex\skills\teach\elai-concept-map.md). It was authored from a FULL read of the feature-map (all 2501 lines) + the phase index, so the curriculum, the dependency order, the per-concept WHY, and the testable KEY FACTS are already laid out — Tier 0 foundations (THESIS/SSB/FSM/Tier-C) through Tier 9 doctrine/plans, plus the canonical teaching order. For "teach me ELAI", READ THIS FILE FIRST (it is the curriculum), do NOT rebuild the concept map from scratch, and do NOT re-read the whole feature-map up front. It is the authoring carve-out of anti-read-big (a teach-owned curriculum artifact). Refresh it only if the feature-map has materially changed since 2026-06-24 (diff first; it names its build date at the top).

Default ELAI corpus (the source-of-truth files the concept map was built from; pull the SLICE a lesson needs, never blind-dump):
- elai-concept-map.md (the teach curriculum spine — load first)
- docs/thefinalglance/ELAI-FEATURE-MAP.md (the live source spine; drill a specific subsystem entry when a lesson goes below concept-map altitude)
- docs/specs/guide/steps/phase-index-MASTER.txt (the phase map)
- docs/thefinalglance/B-plan/B-PLAN-SUMMARY.md
- docs/thefinalglance/{C,D,E,F,G,H,I,J,K}-plan/{C,D,E,F,G,H,I,J,K}-SUMMARY_INDEX.md (more detail per track; stale on status, fine for teaching the design)

When a lesson needs implementation detail below the feature-map altitude (the real seam, the actual symbol, "show me where X is wired"), drill with a single codegraph_explore call (verbatim source grouped by file) rather than reading whole plan files. Read the slice the current lesson needs, not the whole corpus at once. If the user names a narrower target ("teach me the wiring gate", "teach me the SSB registry"), scope to that concept's entry in the map + a codegraph drill.


THE METHOD (the adaptive loop, run per concept)

1 BUILD A CONCEPT MAP FIRST (silent). From the corpus, lay out the concepts to be learned and their dependency order — a concept whose prerequisites the user does not hold yet is taught after those prerequisites. Teach foundations before things built on them. This map is also what the curriculum and the "what's next" decision come from. (Use mcp_think for this curriculum-planning step if the topic is large; the per-question quiz turns below are exempt from GRC blockquoting — they are a conversational learning UX, not implementation reasoning, and blockquote spam would wreck the lesson.)

2 PROBE, DO NOT ASSUME. Open each new concept by finding out where the user actually is. Prefer a real attempt over a yes/no: "before i explain — what do you already think <X> does ?" A bare confidence chip (know it cold / sort of / no idea) is allowed to set the starting rung, but DO NOT trust a "know it cold" — immediately test it. Self-assessment is unreliable (the illusion of knowing); the test is the truth, the claim is not.

3 TEST BY FREE RECALL. The primary question is always open: "explain <X> in your own words", "what happens when <Y> fires", "why does <Z> exist — what breaks without it ?". The user types the answer from memory. This is the engine; make them generate, not pick.

4 GRADE AGAINST THE CORPUS, THEN BRANCH:
- CORRECT and complete -> name what they nailed, mark progress up the ladder, then either move to the next concept or fire a surprise interleave (step 6).
- PARTIALLY correct -> name precisely what is right first (so the correct piece is reinforced), then ask a narrower question aimed only at the gap. Do not re-explain what they already have.
- WRONG or "no idea" -> climb the difficulty ladder (next section). Give the SHALLOWEST useful explanation, re-test, and add one rung of detail at a time until they get it.

5 CHIPS ARE A HINT, NOT THE TEST. When a free-recall answer stalls — the user is blank, or circling — offer an AskUserQuestion with the right answer hidden among plausible distractors, OR a narrowing question that splits the space ("is it closer to A or B ?"). This is scaffolding to rescue the answer. Getting it via chips counts as shaky/learning, NEVER mastered — recognition is not recall. After a chip rescue, come back to the same concept later with a fresh free-recall question and no chips; mastery is only credited on a clean recall pass.

6 INTERLEAVE — AMBUSH WITH OLD MATERIAL. After every 2-4 new concepts, without warning, throw back a previously-covered one: "quick, drop everything — earlier you learned <X>. Now: <new-angle question on X>". Vary the angle so it is not the same question replayed (ask the inverse, a consequence, a comparison). Surprise is the point; the user explicitly wants to be caught off guard. Pull the target from the ledger, preferring items that are shaky or due for review (step on SPACED REPETITION). A correct ambush answer bumps the concept up the ladder; a missed one drops it back to shaky and it re-enters the rotation.

7 KEEP TURNS TIGHT. One question at a time. Short framing, then the question. No walls of text — working memory is finite, and a paragraph the user has to wade through before the question buries the question. Explanations are rationed by the ladder, never pre-emptive.


QUESTION FORMAT (READABILITY — mandatory, the question must be unmissable)

The user reads these in a terminal and must be able to tell the QUESTION apart from the framing at a glance. Every quiz turn follows this exact shape:

- At most ONE short framing line first (plain prose, optional). Never more.
- Then a horizontal rule on its own line: `---`
- Then the question line, and nothing competes with it: start it with a bold tag **Q{n} · {short chip}** then the question itself, the whole question in **bold** so it is the visual anchor of the turn.
- The bold question line is the LAST substantive thing in the turn. Nothing comes after it except, optionally, ONE italic hint line: `_hint: ..._` (only when a hint is being offered).
- NEVER bury the question mid-paragraph. NEVER put framing or caveats AFTER the question. NEVER stack two questions. If you catch yourself writing a paragraph before the `---`, cut it.

Skeleton of a turn:

  one short framing line (optional)

  ---
  **Q3 · SSB** **why is the journal append-only instead of a table you update in place?**

  _hint: think about what an agent could do to a row that already proved its work_

The ladder rungs (explanation on a wrong answer) are prose, but the RE-TEST at the end of each rung uses this same `---` + bold **Q** shape. The grading/feedback that opens a turn ("nailed it", "close — you have X, missing Y") stays plain prose ABOVE the framing line; only the new question gets the rule + bold treatment. Chips (AskUserQuestion) already render distinctly, so they need no `---`.


THE DIFFICULTY LADDER (how explanation escalates on a wrong answer)

The user's core ask: when they do not know, start shallow and add detail until they get it right — do not dump the full answer. Each rung adds the MINIMUM needed to unblock the next attempt, then re-tests.

- RUNG 0 (gist): one sentence, the plain-English point of the concept, zero jargon. Re-test with an easier question.
- RUNG 1 (mechanism): add HOW it works in 2-3 sentences, introduce the one key term. Re-test.
- RUNG 2 (why / contrast): add why it exists, what it is NOT, the problem it solves. Re-test with the original-difficulty question.
- RUNG 3 (worked example): walk one concrete trace from the corpus (a real flow, a real file/seam if the corpus names one). Re-test.
- RUNG 4 (chip rescue): if still stuck after a worked example, drop to an AskUserQuestion narrowing the answer — recognition-level — to get a win, mark the concept shaky, and schedule it for an early re-test with no chips.

Never skip from rung 0 to the full answer to "save time". The struggle between rungs IS the learning. If the user gets it at rung 1, you never deliver rungs 2-4. If a concept needs rung 3+ every time it comes up, it is a foundational gap — flag it and check whether a prerequisite concept is actually the thing that is missing.


SPACED REPETITION AND REVIEW (the time engine)

Every test is stamped with the REAL current date. Get it for real — call `Get-Date -Format o` (PowerShell) or read it from session context — never guess or carry a stale date. Day granularity is enough; exact time is not required.

Each concept carries a mastery LEVEL and a review INTERVAL (Leitner-style, named constants):
- new        -> test now
- shaky      -> re-test later THIS session, and again next session (got it only via chips, or missed it)
- learning   -> review after ~1 day      (one clean free-recall pass)
- solid      -> review after ~4 days      (two clean passes on different days)
- mastered   -> review after ~14 days     (three+ clean passes across days)

LAPSE RULE: if a concept is missed on a review, it drops TWO levels (mastered -> learning, solid -> shaky) and re-enters the rotation. A real memory that has decayed is more important to catch than a new concept.

ON SESSION START (when the user invokes teach for a topic that already has a ledger):
1. Read the ledger (head-only if large — it is an append/update file you own, the authoring carve-out in anti-read-big applies).
2. Compute days elapsed = today - last_tested for every concept.
3. Anything past its interval is DUE. Open the session with a REVIEW ROUND of the due items (surprise free-recall, no warm-up) BEFORE any new material — testing whether the user still remembers is the whole point of the ledger. Report it plainly: "X concepts are due for review, let's see what stuck."
4. Then continue new material from where the concept map left off.

If the user says "review me" / "test me on what i learned" with no new topic, the session is ALL review round: drill every due item, oldest-overdue first, then anything shaky.


THE MASTERY LEDGER (the one file teach writes)

Path: workflow/teaching/{topic-slug}.md  (topic-slug is a stable kebab name for the subject — "elai", "elai-wiring-gate", etc. Default "elai".) workflow/ is gitignored; this is a personal progress file, not a project artifact. It is the ONLY file teach ever writes. Teach never writes project code, never writes memory-bank files, never transitions the FSM.

Format — a header block plus one row per concept, TSV under a fenced block so it is both human-readable and trivially updatable:

```
# Teaching ledger: {topic}
# corpus: {files this ledger was built from}
# updated: {ISO date}
#
# level: new|shaky|learning|solid|mastered   interval: now|1d|4d|14d
# concept_id<TAB>topic<TAB>level<TAB>last_tested(ISO date)<TAB>tests<TAB>correct<TAB>note
wiring-r1	wiring enforcement	solid	2026-06-24	3	2	confuses SKIP vs FAIL on missing evidence
ssb-registry	SSB derived counters	shaky	2026-06-24	1	0	rescued via chips, re-test no-chips
...
```

Update the touched rows after each concept (bump/drop level, set last_tested to today, increment tests/correct, keep a one-line note on the specific misconception so the next session targets it). Overwrite the `updated:` date. Append new concepts as they are introduced. Keep notes to one line — the note is a targeting hint, not a transcript.

Write the ledger at natural stopping points and at session end, not after every single question (batching the writes keeps the lesson flowing). Never leave a session without flushing the ledger — an un-saved session means the spaced-repetition clock did not advance and the user re-learns what they already knew.


SESSION SHAPE (start to finish)

1. Resolve the topic-slug and corpus. Glob workflow/teaching/ for an existing ledger; if found, this is a resume.
2. If resuming: load ledger, run the DUE REVIEW ROUND first (spaced repetition step above).
3. Read the slice of corpus the next concepts need (FEATURE-MAP / phase-index first for ELAI, then the specific track).
4. Run the adaptive loop (probe -> free-recall test -> grade -> ladder/chips -> interleave) concept by concept in dependency order.
5. Ambush with old material every 2-4 concepts.
6. Flush the ledger at stopping points and at the end. Give a short close: what is now solid, what is still shaky, what is due next session and roughly when.


CONSTRAINTS

- USER-INVOKED ONLY. Never auto-fire; offer and wait.
- Free recall is the test; chips are a rescue. A concept "learned" only via chip recognition is shaky, never mastered.
- Source of truth is the fed-in corpus, spec-eval posture: everything described is treated as real, never graded against live code or discounted as unbuilt. Implementation status is an aside, never a wrong-answer verdict.
- Explanation is rationed by the ladder. Never pre-empt with the full answer; the struggle between rungs is the mechanism.
- Stamp every test with the REAL date; the ledger is the only file written; never transition the FSM; read-only on all code.
- Anti-read-big applies to corpus reads (slice, do not dump); the authoring carve-out covers the ledger file teach owns.
- One question at a time, tight turns, no walls of text.
- Anti-spiral: if the same concept needs a full ladder climb three sessions running, it is a foundational/prerequisite gap — stop drilling it head-on, find and teach the missing prerequisite, then return.


END TEACH SKILL
