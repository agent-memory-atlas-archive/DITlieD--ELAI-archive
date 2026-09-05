# ELAI Claims Policy (KD-1: number-before-claim)

Status: NORMATIVE (K-1, WAVE KW-1). Binds every externally-visible surface
(README, docs/strategy/ positioning copy, a marketing brief, a spec site).
Does not bind internal reasoning docs (workflow/reasoning/, decision logs) --
those are grounded-by-default under CLAUDE.md already and are not
externally visible.

## The rule (KD-1, K-MASTER-BRIEF-2026-06-12.md 3.4)

No externally-visible claim ships with a number or a superlative unless it
carries one of:

- `[EXTERNAL]` or `[EXTERNAL, unverified]` -- a third-party number, tagged
  with its source (a KW-N grounding-map pointer, a paper id, a URL). Never
  presented as ELAI's own result.
- `[DESIGN]` -- an intentional, non-measured design choice or target,
  presented as a decision, never as a measured fact.
- `[MEASURED]` or an adjacent recompute pointer (an `elai bench <subcmd>`
  invocation, a `file.rs:LINE` source anchor) -- ELAI's own number, and it
  MUST be journal-recomputable (GD-2): a second run under the same seed
  reproduces it byte-identically from the SSB journal.
- `PENDING` -- explicit, when the bench that will produce the number has
  not run yet. A PENDING row names the exact command that settles it. A
  PENDING claim asserts nothing numeric yet, so it never fabricates one
  (KD-1's "before" -- the number comes before the claim, not the reverse).

A claim that is none of the above is UNGROUNDED marketing copy and is
BLOCKed by `.claude/tools/claims_policy_lint.py` before it ships.

## Honest-wording is normative, not re-implemented here

The cloud-vs-local wording table for what ELAI may claim about
reproducibility (I-43 `docs/thefinalglance/I-plan/I-43_claim_scoper_kill_gate.md`)
is the canonical honest-wording reference for that specific claim class and
is cited, never restated: cross-lane action-stream divergence on
security-critical paths is measured by `elai bench claim-scope`; when it
exceeds I-43's fixed 5% threshold [DESIGN], the honest word for the cloud lane is "trace-authentication",
never "reproducible" or "replay" (those stay reserved for the open-weight /
deterministic-replay lane, J-13). Any positioning copy making a
reproducibility-adjacent claim defers to that table's wording, not to a
paraphrase invented at write time.

## Why this exists

The B-40 / B-41 / B-42 bad-cite incidents (CLAUDE.md
`feedback_paper_citation_verify`) shipped numbers into review/positioning
copy without a checkable source. KD-1 is the doctrine that followed; this
document is its normative statement and `claims_policy_lint.py` is its
deterministic, fail-closed enforcement at the copy surface -- the same
shape as wiring-enforcement.md's "no claim without production evidence" at
the code surface, applied to prose instead of pub fns.

## Enforcement (auditor-applied, never main-LLM self-run)

`python .claude/tools/claims_policy_lint.py --files <changed positioning
docs> --json`. Exit 0 = every number/superlative in the changeset is tagged
or pointed; exit 1 = BLOCK, at least one uncleared claim; exit 2 = a
scan-config error (no files, a named file missing) -- fail-closed, never
read as PASS (mirrors wiring-enforcement R1/R2). Per D-6 (execution/
evaluation separation) and main-llm-role.md, the AUDITOR runs this over the
changeset; the agent that wrote the copy does not grade its own claims.
`.claude/rules/conditional-routing.md` loads this policy whenever
README/positioning copy is edited, so the writer sees the rule before
writing, even though the writer never runs the gate as the authority.

## Compliant vs non-compliant examples

Compliant:
- "5.2%/21.7% hallucinated-package rate across 16 LLMs [EXTERNAL] KW-1
  (USENIX Security 2025, Spracklen et al.)."
- "Claim-scope divergence: PENDING -- run `elai bench claim-scope --suite
  core --seed 42 --json`."
- "SAFEDELTA gate-on/gate-off delta: [MEASURED] see
  benchmarks/safedelta_runner.rs; recompute via `elai bench safedelta`."

Non-compliant (BLOCKed by the lint -- these two example lines are
deliberately uncleared demonstration text; if this file is ever scanned
directly rather than treated as normative prose, the lint SHOULD flag
them, which is the expected self-consistent behavior, not a bug):
- "ELAI is the safest coding agent on the market today." (bare superlative,
  no tag)
- "Hallucinated packages appear in 21.7% of open-source completions." (a
  real, correctly-sourced KW-1 number, but shipped with no EXTERNAL bracket
  tag -- the number is right, the citation discipline is what is missing)

## Non-goals

This policy does not gate internal reasoning, decision docs, or code
comments -- only externally-visible claim-bearing copy. It does not verify
that a cited [EXTERNAL] number is itself correct (that is the KW
web-verification step at dossier-authoring time, `_grounding/K-GROUNDING-
MAP.md`); the lint only verifies that SOME tag or pointer is present, not
its truth.
