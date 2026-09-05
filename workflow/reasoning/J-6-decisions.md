# J-6 Decision Document — incremental-eval harness skill

**Date:** 2026-06-30
**Task:** J-6.T1 — create `.claude/skills/incremental-eval/` (SKILL.md + eval pair) plus this decision doc. Harness skill only; zero runtime code.
**Scope:** `.claude/skills/incremental-eval/` (NEW) + `workflow/reasoning/J-6-decisions.md` (this file). No `.elai_cc` code, no gate files, no test files.

## What

Shipped a self-triggering harness skill that encodes the user's re-stated MAJOR RULE for multi-artifact tasks: "read ONE file, write your reasoning down to a file, read the next, update the file." The skill auto-loads via the Claude Code harness native skill discovery (`.claude/skills/*/SKILL.md` frontmatter `description` field); no custom hook, no elai.map row, no Python/Rust loader. The protocol resolves one topic-keyed verdict file in `workflow/reasoning/`, walks artifacts one-by-one appending a bounded anchored verdict block per artifact, keeps the per-cycle working set to {task spec, verdict-file index head-only, next artifact}, and synthesizes once at the end.

## Key Decisions

### JD-3 — enforcement hook DEFERRED (ship-now scope per W-10)

The skill is self-triggering only this pass. No PostToolUse batch-read counter, no UserPromptSubmit trigger guard, no settings.json hook entry. Rationale: observe how often the skill self-triggers across 3 or more real multi-artifact sessions before deciding whether enforcement is even needed; a counter that fires on every multi-read would false-positive on legitimate non-evaluation reads (e.g. orchestrate's own plan walk, paper-to-plan's source ingestion). Enforcement is a future call, gated on observed drift (skill fails to fire on a 3+-artifact eval task, or fires but the transcript shows batch-reading with no verdict file growth). The deferral is recorded here per ACCEPTANCE line 97; the auditor should NOT block on the absence of a hook.

### Schema findings (eval pair)

Verified the JC-5 / E-43 fixture convention against `auto-orchestrate/eval/{positive,negative}.jsonl` and the cook/grill reference skills: one JSON object per line, schema `{"prompt","expected_trigger","rationale"}`, no trailing array, no comments. The validator (`agentskills_validator.py`) does NOT check eval/ presence or content — the J-6 DETERMINISTIC_CHECK block supplements with `test -f` on the eval pair. Shipped 6 positives (diverse phrasings: "go through these N docs", "read and evaluate these artifacts", "walk the plan corpus", "compare these reports", the pattern invoked by name, an explicit anti-batch-read instruction) and 8 negatives. Every negative's rationale names the owning skill or the reason (single-file, pure-chat, already-inside-another-protocol) per the grill/cook pattern.

### Union / stable-ordering rule for the verdict file

The verdict file is ONE topic-keyed file in `workflow/reasoning/`, append-only, head-only on re-reads mid-walk. Artifacts are walked in the order the user presented them (stable ordering = user-given order, not alphabetical, not re-sorted). If the same artifact appears twice in the user's list, append a second block (the user re-dropped it for a reason); do not dedupe. The synthesis section is appended exactly once, after the last artifact block. This is the handoff-protocol ONE-FILE discipline applied to evaluation — but the verdict file IS a reasoning file, NOT a handoff; the discipline is shared, the artifact is distinct (stated in SKILL.md to satisfy stub_ban).

### Increment-vs-spec-eval trigger boundary (W1 resolution)

spec-eval and incremental-eval both can fire on a batch drop of files. Resolution (per context-package W1 + plan Open questions line 110-111): the VERB decides, not the batch.
- spec-eval fires when the ask is to JUDGE A DESIGN on its merits ("improve this design", "compare ELAI to X", "elaborate this idea") — the batch is CONTEXT for a design verdict.
- incremental-eval fires when the ask is to READ/EVALUATE/SUMMARIZE the artifacts ONE-BY-ONE ("go through these and tell me what is in each", "review each", "compare these reports") — the protocol IS the deliverable.
- A batch drop alone is ambiguous; the skill's "Does NOT fire on" block + the negative eval set (which includes "improve this design" as a spec-eval-owned negative) is the guard. Tune from the first 3 sessions' observations (JD-3).

## I-11 / I-5 PARTIAL drift note (per exploration plan DRIFT section)

J-GRC-V3-GROUNDING-MAP.md lines 23, 31, 131 label I-11 (belief_state.rs) and I-5 (pristine_path_filter.rs) as PARTIAL, not PLANNED — correcting older ROUND-1 "PLANNED only" labels. The J-6 spec line 13 calls I-5/I-6/I-11 "runtime siblings" without status. The skill CITING them (not restating) is correct regardless of their build status: the skill cites the I-plan source files (titles / mechanism concepts), not the grounding-map status lines. No action; if a future session re-grounds I-5/I-11 to SHIPPED, the SKILL.md citations remain accurate (they point at the plan files, which persist). I-6 anchor pane slot has no PARTIAL flag in the grounding map reviewed; cited the same way.

## J-3 last-good column contract — NOT APPLICABLE

Per the task prompt: "Per J-4 it would also cover J-3 last-good column contract — but J-6 has no such contract." Confirmed: J-6 ships no Rust/Python/TS schema, no DB column, no migration. The J-3 last-good column contract belongs to a different plan; nothing to record here.

## Source Verification

- Read J-6 plan fully (117 lines): `docs/thefinalglance/J-plan/J-6_incremental_eval_skill.md`.
- Read context package fully: `workflow/sessions/s-38068/context-package-J-6-T1.md`.
- Read exploration plan fully: `workflow/sessions/s-38068/exploration/J-6-T1-skills.md`.
- Read `agentskills_validator.py` fully (125 lines): confirmed name==dir-leaf rule, description angle-bracket ban (any literal `<` or `>` fails — used "3 or more" / "N or more" instead of ">=3"), semver rule, exit codes.
- Read `spec-eval/SKILL.md` (73 lines): confirmed trigger-overlap risk + the "Does NOT fire on" pattern.
- Read `auto-orchestrate/SKILL.md` (40 lines) + its eval pair: confirmed description-as-trigger-spec style + JC-5 fixture schema.
- Read `orchestrate/SKILL.md` (34 lines): confirmed "WHEN THIS SKILL FIRES" / "Does NOT fire on" body pattern.
- Read `J-11-decisions.md` (30 lines): confirmed decision-doc shape for this directory.

## Acceptance mapping

- [x] validator passes (self-verified below, exit 0).
- [x] eval pair present with 6 positives + 8 negatives (both >= 4, JC-5).
- [x] protocol text carries the per-cycle context set + head-only rule + the named anti-patterns.
- [x] cites (not forks) token-discipline + handoff-protocol + I-5/I-6/I-11.
- [x] no enforcement hook shipped (JD-3); this doc records the deferral.
- [ ] JD-1 (first live multi-artifact task after shipping shows the skill firing) — observational, NOT a build-time gate; cannot be satisfied in-task. Post-ship observation across >=3 sessions per JD-3.

## DETERMINISTIC_CHECK (self-verified)

```
python .claude/tools/agentskills_validator.py .claude/skills/incremental-eval/SKILL.md   # exit 0
test -f .claude/skills/incremental-eval/eval/positive.jsonl
test -f .claude/skills/incremental-eval/eval/negative.jsonl
test -f workflow/reasoning/J-6-decisions.md
```

End of J-6 decision document.
