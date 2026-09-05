# J-27: harness steals — protected-blocks read hook, 304-revalidated WebFetch cache, dispatch-protocol amendments

STATUS: PLANNED (WAVE RSW-1; .claude harness leg, JD evidence rules; 2 NEW hooks + 1 amendment bundle from the addyosmani/agent-skills + MiMoCode-adjacent read)

- Track J (repo-steal extension) | lever CANT-WRECK + EFFICIENT + AUDITABLE | grounding STRONG
  (direct source read of addyosmani/agent-skills, MEDIUM reliability) | nature BUILD (T1/T2) +
  EXTENSION (T3) | effort S-M | WAVE RSW-1

- Premise: the agent-skills repo is mostly prose a mature harness already enforces harder
  (catalogued LOW-novelty in docs/.source/agent-skills-INDEX.md section 4), but it carries TWO
  deterministic hook designs ELAI's harness has no equivalent for, plus four dispatch-protocol
  sharpenings worth one amendment pass. NO skill is imported — any future import still goes
  through /skill-audit + workflow/skill-imports.log per skill-import-policy.md. Steal file:
  workflow/reasoning/steal-mimocode-sia-agentskills-2026-06-12.md (C1..C6; C7 rejected-covered).

## Steal/build
T1 PROTECTED-BLOCKS READ HOOK (C1, NEW; hooks/SIMPLIFY-IGNORE.md). A PreToolUse(Read) hook that
  replaces source regions annotated `// elai-protect-start` .. `// elai-protect-end` (comment
  syntax per language) with content-hashed placeholders BEFORE the model sees the file; a
  PostToolUse(Edit|Write) hook restores the original bytes if a placeholder survives in the
  written content, and BLOCKS the write if a placeholder was mangled/partially edited. The model
  edits around what it cannot see — "no unsolicited refactoring" becomes mechanical for the
  blocks that matter (perf-critical Rust, intentionally-nonstandard sidecar code). Hash =
  hashlib.blake2b (stdlib, JC-2 precedent). NEW hook files at the rag/hooks seam, registered in
  settings by user-reviewed change (JD-4 — pre_tool_check.py et al are never edited). Opt-in by
  annotation: zero behavior change on un-annotated files (JD-3-compatible by construction).
T2 304-REVALIDATED WEBFETCH CACHE (C2, NEW; hooks/SDD-CACHE.md). PreToolUse(WebFetch) issues a
  conditional HEAD with If-None-Match/If-Modified-Since from the cache row; HTTP 304 = the cached
  content IS a fresh verification (grounded-by-default preserved — this is revalidation, not a
  TTL staleness window), surfaced as the tool payload; 200/changed = fetch proceeds and
  PostToolUse records body + validators. NEVER cache a response lacking ETag/Last-Modified
  (fail-open to live fetch). Cache key = sha256(url) under workflow/sessions-shared cache dir;
  original prompt stored as metadata and surfaced on hit so the model decides relevance.
T3 DISPATCH-PROTOCOL AMENDMENTS (C3..C6, EXTENSION; zero code). (a) CLAIM-STRIPPED AUDITOR
  DISPATCH: orchestrating skills compose auditor/critic prompts from ARTIFACT+CONTRACT only,
  never the executor's claim/rationale (confirmation-bias input shaping; echoes I-21
  zero-shared-context) — one rule line into orchestrate/auto-orchestrate/b-plan-runner dispatch
  sections + agent-dispatch-preamble note. (b) R-5 gains a "NOTICED BUT NOT TOUCHING" line
  (out-of-scope observations disclosed, never acted on — sharpens R-3). (c) prompt-transform +
  /grill restatements gain a mandatory "Out of scope:" line + the non-yes rejection list
  ("sounds good"/"whatever you think"/silence are not approval). (d) orchestrate/auto-orchestrate
  pre-dispatch gains the 4-question fan-out checklist (independent? different KIND of finding
  per agent? merge fits main context? wait justifies parallelism?).
REJECTED in this bundle (recorded): severity labels (auditor BLOCK/WARN/INFO exists), browser
  trust-boundary axioms (G-1/F-3 cover), define-questions-first observability (wiring-enforcement
  covers), source-driven-development / context-trust taxonomy (grounded-by-default + source-tiers
  cover). C7 of the steal file.

## Existing ground truth (glob-confirmed at authoring where stated; rest PLAN-asserted, W-7)
- No protected-blocks or WebFetch-cache hook exists anywhere under .claude/ or the rag/hooks seam
  (steal-file sweep; executor re-confirms with a glob before building).
- JD-1..JD-4 (J-0_INDEX) define harness evidence: hook-executions.log rows / session artifacts,
  derived-never-recalled, observe-only-first, frozen existing hooks.
- J-3 (pre-image backups) + J-4 (handoff ledger) are the sibling harness hooks; J-27's hooks
  follow their file/registration shape and are parallel-safe with both.
- agent-dispatch-preamble.md R-1..R-7; orchestrate/auto-orchestrate/b-plan-runner SKILL.md
  dispatch sections; prompt-transform + grill skills — the T3 amendment targets.
- Source ground truth (W-3): docs/.source/agent-skills/ clone + agent-skills-INDEX.md +
  workflow/papers/agent-skills-interface.md.

## REVISED scope (W-10 tiers)
SHIP NOW Tier-1: T1 + T2 hook files + tool scripts (stdlib-only python, JD rules) + T3 amendment
  lines. DEFER Tier-2: nothing — the whole unit is local and cheap. The only gating act is the
  user-reviewed settings registration for the two new hooks (JD-4).

## Integration
- CREATE .claude/tools/protect_blocks.py (~150 LOC: annotate-scan, placeholder swap, restore,
  mangle-block) + the PreToolUse(Read)/PostToolUse(Edit|Write) hook wrappers at the rag/hooks
  seam (~60 LOC) — NEW files, settings registration by user.
- CREATE .claude/tools/webfetch_cache.py (~120 LOC: conditional HEAD, cache rows, validators-or-
  no-cache) + PreToolUse(WebFetch)/PostToolUse(WebFetch) wrappers (~50 LOC).
- MODIFY (amendment lines only): .claude/rules/agent-dispatch-preamble.md (R-5 line + CLAIM-strip
  note), .claude/skills/{orchestrate,auto-orchestrate,b-plan-runner}/SKILL.md (CLAIM-strip +
  fan-out checklist), .claude/skills/{prompt-transform,grill}/SKILL.md (Out-of-scope + non-yes).
- frontend_pair: N/A (harness leg, no dashboard). Justified.

## WIRING_CONTRACT
produces_symbols:
- protect_blocks.py + its two hook wrappers (placeholder swap/restore/block)
- webfetch_cache.py + its two hook wrappers (304-revalidated cache)
- the six amendment lines (CLAIM-strip, NOTICED-BUT-NOT-TOUCHING, Out-of-scope, non-yes list,
  fan-out checklist) in the named rule/skill files
consumes_symbols:
- the rag/hooks registration seam (J-3/J-4 shape), hook-executions.log (JD-1 evidence),
  skill-import-policy.md (T3 does not bypass it), agent-dispatch-preamble R-5 (extended not
  rewritten).
reachable_from:
- a REAL harness session: a Read on an annotated file, a WebFetch on a validator-bearing URL,
  an orchestrating-skill dispatch — each leaves its hook-executions.log row (JD-1). "The file
  exists in .claude/" is wire-dark.
stub_ban:
- NO edits to existing registered hooks (JD-4); NO TTL-based cache expiry (304-revalidation
  only); NO caching responses without validators; NO placeholder left in a written file (the
  PostToolUse restore must be total or the write BLOCKS); NO LLM in either hook path.
observable_effect:
- trigger: "annotate one block in a scratch file, Read it in a live session; WebFetch the same
  ETag-bearing URL twice in one session"
- expect_artifact: "hook-executions.log rows: Read shows the placeholder swap (model-visible
  content hash-stubbed) and the file on disk UNCHANGED after an Edit around it; second WebFetch
  logs a 304 hit with the cached payload served. NEGATIVE (R2/JD-1): mangle a placeholder in an
  Edit and the write must BLOCK; strip the URL's validators and the cache must refuse to store
  (a stored row without validators is a bug). A session with no annotations/no cache rows
  behaves byte-identically to today."

## DETERMINISTIC_CHECK
```bash
test -f .claude/tools/protect_blocks.py && test -f .claude/tools/webfetch_cache.py
python .claude/tools/protect_blocks.py --self-test     # swap->restore roundtrip byte-identical; mangled placeholder -> nonzero
python .claude/tools/webfetch_cache.py --self-test     # no-validator response refused; 304 path served from cache
rg -n "NOTICED BUT NOT TOUCHING" .claude/rules/agent-dispatch-preamble.md   # amendment landed
test -f workflow/reasoning/J-27-decisions.md
```

## Grounding
- Source: agent-skills clone (MEDIUM), hooks/SIMPLIFY-IGNORE.md + hooks/SDD-CACHE.md +
  skills/doubt-driven-development/SKILL.md + skills/interview-me/SKILL.md +
  references/orchestration-patterns.md (file cites in agent-skills-INDEX.md section 4 A..E).
- Doctrine: JD-1..JD-4 (this is a harness leg: evidence = log rows, hooks non-blocking-by-
  construction on un-annotated content, existing hooks frozen), token-discipline (T2 kills
  redundant refetch without weakening grounded-by-default), skill-import-policy (untouched).
- SSB: NONE (harness leg, JC-3 analog; zero .elai_cc code).

## COMPOSES-WITH
J-3/J-4 (sibling hook shape), agent-dispatch-preamble, orchestrate/auto-orchestrate/
b-plan-runner, prompt-transform, /grill, skill-import-policy, I-21 (the CLAIM-strip echo).
Restates none; imports no skill.

End of J-27.
