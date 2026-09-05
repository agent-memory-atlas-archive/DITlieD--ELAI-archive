---
name: incremental-eval
description: Per-artifact verdict protocol for any task that presents 3 or more artifacts (files, reports, plans, papers, screenshots, specs) to read, evaluate, compare, review, or summarize one-by-one. Triggers when the user says "go through these N docs", "read and evaluate these artifacts", "review these files and tell me what is in each", "walk the plan corpus", "compare these reports", "summarize each of these", or invokes the read-one-write-verdict-reset pattern by name. The conversation is disposable because the FILE is the state — read ONE artifact fully, append one bounded anchored verdict block to a single topic-keyed file in workflow/reasoning, read the next, update the file, then synthesize once at the end. Does NOT fire on single-file asks, pure-chat questions, design-judgement tasks (spec-eval owns), plan-file rewrites (planning-workflow owns), or any task already inside another skill's protocol.
license: internal
compatibility: "Designed for Codex. ELAI workflow root. Portable to any harness that auto-loads .Codex/skills/*/SKILL.md by frontmatter description."
metadata:
  version: "1.0.0"
  author: "elai"
---

incremental-eval — the read-one / write-verdict / reset MAJOR RULE, made self-triggering and durable.

WHEN THIS SKILL FIRES

Trigger phrases (case-insensitive, anywhere in the user message):
- "go through these N docs / files / reports / plans / papers / screenshots"
- "read and evaluate these artifacts"
- "review these files and tell me what is in each"
- "walk the plan corpus" / "walk these specs"
- "compare these reports" / "summarize each of these"
- "read ONE file, write your reasoning down, read the next, update the file" (the pattern invoked by name)
- any task that drops 3 or more artifacts (files, reports, plans, papers, screenshots, specs) and asks for a per-artifact read/evaluate/compare/summarize

Does NOT fire on:
- a single-file ask (one doc, one report, one screenshot) — just read and answer; no verdict-file overhead.
- a pure-chat question with no artifact list to walk.
- "improve this design" / "elaborate this idea" / "compare ELAI to X" — spec-eval owns design judgement; a batch drop for design judgement routes to spec-eval, not here.
- "improve this phase file" / "start phase X" / "plan improvement" — planning-workflow owns the in-place phase rewrite.
- any task already inside another skill's protocol (grill interrogation, paper-to-plan intake, orchestrate phase walk, oracle fact lookup). The other skill's reads are its own; do not double-fire.
- a literal status question ("is X wired?", "what's shipped?") — route to the wiring tools, not here.

THE PROTOCOL (mandatory, ordered)

1. RESOLVE ONE verdict file. Glob workflow/reasoning/ for a topic-keyed verdict file matching this task (e.g. workflow/reasoning/TOPIC-verdict.md, where TOPIC is the task slug); append if it exists, create if it does not. This is the handoff-protocol ONE-FILE discipline applied to evaluation — but the verdict file IS a reasoning file, NOT a handoff; the discipline is shared, the artifact is distinct (see CITATIONS).

2. PER ARTIFACT, in order:
   a. Read the artifact fully (in chunks if large — read-means-read, no skim).
   b. Append ONE bounded verdict block to the verdict file:
      - an ANCHOR line: the artifact path / id / section (anchorless findings are dropped, per the I-41 advisory doctrine).
      - 3 to 8 lines of findings.
      - open questions, if any.
   c. Never append more than one artifact's block before reading the next artifact.

3. PER-CYCLE CONTEXT DISCIPLINE (the BeliefState trick). The working set per cycle is exactly {task spec, verdict-file INDEX head-only, next artifact}. Prior artifacts live in the file, not the window. Re-reads of the verdict file are HEAD-ONLY (read the index / the last block, never the whole file mid-walk). The conversation is disposable because the file is the state; context does not monotonically grow.

4. AFTER THE LAST ARTIFACT: append ONE synthesis section to the same verdict file (cross-artifact themes, conflicts, the answer to the task). Then the chat answer cites the verdict file path so the user can audit the chain.

ANTI-PATTERNS (each is a skill failure; mirrored in eval/negative.jsonl)
- batch-reading multiple artifacts before any verdict is written.
- unbounded verdict blocks (more than ~10 lines per artifact).
- full-file re-reads of the verdict file mid-walk (use head-only).
- verdicts without an ANCHOR line (anchorless = dropped).
- synthesizing before all artifacts are logged.

CITATIONS (cite, do NOT fork — GD-1 / stub_ban)
- .Codex/rules/token-discipline.md — the anti-read-big and read-means-read HARD RULES this skill operationalizes.
- .Codex/rules/handoff-protocol.md — the ONE-FILE rule; the verdict file follows the same one-topic-keyed-append-only discipline but is a reasoning file, not a handoff.
- docs/thefinalglance/I-plan/I-5_pristine_context_injector.md — runtime sibling (pristine path filter); cited, not restated.
- docs/thefinalglance/I-plan/I-6_anchor_pane_slot.md — runtime sibling (the anchor concept the per-artifact ANCHOR line borrows); cited, not restated.
- docs/thefinalglance/I-plan/I-11_pabu_belief_state_handoff.md — runtime sibling (the BeliefState trick the per-cycle context set borrows); cited, not restated.

BOUNDARY vs spec-eval
- spec-eval judges a DESIGN on its merits (as-specified) and may consume a batch of spec/plan files as context for that judgement. incremental-eval walks a batch of artifacts one-by-one and writes a per-artifact verdict. If the user's ask is "judge this design", route to spec-eval. If the ask is "go through these and tell me what is in each / review each / compare each", route here. A batch drop is not enough to disambiguate — the VERB decides.

NO ENFORCEMENT HOOK THIS PASS (JD-3)
This skill is self-triggering only. No PostToolUse counter detects batch-reads. The deferral is recorded in workflow/reasoning/J-6-decisions.md; enforcement is a future call if drift is observed across 3 or more real sessions. Do not add one now.

END INCREMENTAL-EVAL SKILL
