# J-6: incremental-eval skill — the read-one/write-verdict/reset MAJOR RULE made durable

STATUS: PLANNED (WAVE 1; BUILD, zero code — one skill directory; effort S; the J-plan authoring
session is the recorded validation run)

- Track J | lever STEERING | grounding STRONG (HG-2: no such skill among the 11) | nature BUILD |
  WAVE 1
- Premise: the user re-states the same MAJOR RULE every multi-artifact task: "do not read 10 files
  and reason once — read ONE file, write your reasoning down to a file, read the next one, update
  the file". The Screenshot_42 pattern [SRC-3] shows why it works: the conversation is disposable
  because the FILE is the state, so context never monotonically grows. Encoding it as
  .claude/skills/incremental-eval makes it self-triggering and durable instead of per-prompt.
  The runtime siblings (I-5 pristine context, I-6 anchor pane, I-11 BeliefState) are cited, never
  restated — this skill is the HARNESS-side protocol only.

## Steal/build
NEW .claude/skills/incremental-eval/ containing:
- SKILL.md — trigger: any task presenting N>=3 artifacts (files, reports, plans, papers,
  screenshots) to read/evaluate/compare/summarize, OR the user invoking the pattern by name.
  PROTOCOL (mandatory, ordered):
  1. resolve ONE topic-keyed verdict file in workflow/reasoning/ (glob-then-append-if-exists —
     the handoff-protocol one-file discipline applied to evaluation).
  2. per artifact: read it (fully, in chunks if large) -> append ONE bounded verdict block:
     an ANCHOR line (path / id / section) + 3-8 lines of findings + open questions; never more
     before the next artifact.
  3. per-cycle context discipline (the BeliefState trick): the working set is {task spec,
     verdict-file INDEX (head-only), next artifact} — prior artifacts live in the file, not the
     window; re-reads of the verdict file are head-only.
  4. after the last artifact: ONE synthesis section in the same file, then the chat answer cites
     the file.
  ANTI-PATTERNS (named, with the negative eval fixtures): batch-reading multiple artifacts before
  any verdict; unbounded verdicts (>~10 lines per artifact); full-file re-reads of the verdict
  file; verdicts without anchors; synthesizing before all artifacts are logged.
- eval/positive.jsonl + eval/negative.jsonl (JC-5: the E-43 convention all 11 existing skills
  follow) — positives: multi-file review asks, "go through these N docs", plan-corpus reads;
  negatives: single-file asks, pure-chat questions, tasks already inside another skill's protocol
  (planning-workflow's own reads).
- frontmatter passing .claude/tools/agentskills_validator.py (JC-5).

## Existing ground truth
- HG-2: 11 skills, none covers this; no naming collision.
- the J-authoring session itself executed the protocol verbatim: 14 artifacts -> 14 appended
  sections + synthesis in workflow/reasoning/J-plan-prereading-2026-06-10.md — the validation
  artifact already exists.
- token-discipline rule (anti-read-big + read-means-read) and handoff-protocol one-file rule are
  the standing house rules this skill operationalizes; it must cite them, not fork them.

## REVISED scope (W-10)
SHIP NOW: the skill dir + eval pair + validator pass. DEFER: any enforcement hook (a PostToolUse
counter detecting batch-reads) — observe how often the skill self-triggers across >=3 real
sessions first (JD-3); enforcement is a future call if drift is observed.

## Integration
.claude/skills/incremental-eval/{SKILL.md, eval/positive.jsonl, eval/negative.jsonl}. The skill
list in CLAUDE.md's "Skills (mandatory workflow)" section gains NO entry this pass (it lists the
core-workflow skills only; auto-load handles the rest) — recorded so nobody "fixes" it.
frontend_pair: N/A.

## WIRING_CONTRACT
produces_symbols:
  - .claude/skills/incremental-eval/SKILL.md (+ eval pair)
consumes_symbols:
  - token-discipline + handoff-protocol rules (cited as the governing law)
  - I-5 / I-6 / I-11 (runtime siblings; cited, not restated — GD-1)
reachable_from:
  - any live session given a multi-artifact eval task (the skill trigger)  [HARNESS PRODUCTION,
    JD-1]
stub_ban:
  - NO protocol fork of the handoff one-file rule (the verdict file IS a reasoning file, not a
    handoff; the discipline is shared, the artifact is distinct — stated in SKILL.md)
  - NO unbounded verdict blocks; NO anchor-less findings (the I-41 advisory doctrine: anchorless
    = dropped)
  - NO enforcement hook this pass (JD-3)
frontend_pair: N/A — harness skill; justified.
observable_effect:
  trigger: "a live session is given a 3+-artifact evaluation task"
  expect_artifact: "the skill visibly fires (transcript shows the invocation) and a topic-keyed
    workflow/reasoning/ file accumulates one anchored block per artifact, appended incrementally
    (file mtime/growth between reads provable). The RETROACTIVE validation run is on record:
    workflow/reasoning/J-plan-prereading-2026-06-10.md (14 sections, one per source file).
    NEGATIVE: the negative eval fixtures do NOT trigger the skill; a batch-read transcript
    pattern is matched by the anti-pattern examples (eval/negative.jsonl), failing the skill eval
    if the protocol is skipped (R2 analog)."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/agentskills_validator.py .claude/skills/incremental-eval/SKILL.md   # exit 0
test -f .claude/skills/incremental-eval/eval/positive.jsonl
test -f .claude/skills/incremental-eval/eval/negative.jsonl
test -f workflow/reasoning/J-6-decisions.md
```

## ACCEPTANCE
- [ ] validator passes; eval pair present with >=4 positives + >=4 negatives (JC-5).
- [ ] protocol text carries the per-cycle context set + head-only rule + the named anti-patterns.
- [ ] cites (not forks) token-discipline + handoff-protocol + I-5/I-6/I-11.
- [ ] no enforcement hook shipped (JD-3); decision doc records the deferral.
- [ ] first live multi-artifact task after shipping shows the skill firing (JD-1).

## Files touched
- CREATE: .claude/skills/incremental-eval/SKILL.md (~60 lines)
- CREATE: .claude/skills/incremental-eval/eval/{positive,negative}.jsonl
- CREATE: workflow/reasoning/J-6-decisions.md

## Citations / Grounding
SRC-1 N-6 + SRC-3 (Screenshot_42 pattern); HG-2; JC-5; the J-authoring session as the recorded
validation run.

## Open questions / risks
- trigger precision (firing on tasks another skill already governs): the negative eval set is the
  guard; tune from the first 3 sessions' observations (JD-3).

## Next
The MAJOR RULE stops being re-typed. J-1's /see and future review skills inherit the same
one-artifact-at-a-time spine; the protocol's verdict files become natural J-4 ledger companions.

End of J-6.
