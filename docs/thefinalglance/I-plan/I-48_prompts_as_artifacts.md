# I-48: prompts as reviewed file artifacts — every dispatch prompt template lives as a static file, imported as text, diffable and reviewed

- Track 20 | lever STEERING | grounding STRONG | nature REFACTOR | effort S | WAVE 2

- Premise: STEAL-LIST C1 ("cheap, high leverage"). gajae keeps every prompt as a static .md imported as
  text — diffable, code-reviewed, never an inline string literal scattered through dispatch code. ELAI's
  D-1 typed templates point the same direction as a discipline; I-48 makes it mechanical: prompt text
  lives under crates/*/prompts/ as files pulled in via include_str!, and a contract test bans long inline
  prompt literals in dispatch code. This is also the precondition that makes I-36 (compiled prompt
  artifacts) and I-47 (frozen injection blocks) reviewable: you cannot pin or diff what is interpolated
  inline.

## Steal/build
- Steal the discipline shape from gajae (packages/ai/src/prompts/ + packages/coding-agent/src/prompts/;
  SL-10 read rule applies, though the pattern is fully visible from the directory shape).
- Build: (a) inventory pass — grep dispatch/agent/role code for inline prompt string literals above a
  threshold (~200 chars or multi-line with role framing); (b) extract each to
  crates/<crate>/prompts/<name>.md + include_str!; (c) the ban: a contract test that scans the dispatch
  modules for new inline prompt literals over the threshold and FAILS on any (allowlist for genuine
  non-prompt strings, frozen).
- EXCLUDED: no rewriting of prompt CONTENT (extraction is byte-preserving — this session changes where
  prompts live, not what they say); no template-engine dependency (include_str! + the existing typed
  substitution only); no touching .claude/agents/*.md (harness layer, not runtime).

## Existing ground truth
- D-1 typed templates exist as the substitution machinery (executor re-greps the current template module
  — W-7; PLAN-asserted here). The inline-literal count is unknown at authoring — the inventory pass is
  task 1 and the session's scope estimate is revised after it (a >40-file blast radius pauses for an
  impact-mapper pass per CLAUDE.md impact analysis).

## REVISED scope (W-10 tiered)
- SHIP NOW: inventory + extraction of the orchestrator/providers dispatch prompts + the ban test.
- DEFER: sidecar python prompt strings (cross-language pair — separate slice with cross_lang_diff);
  dashboard-side copy.

## Integration
- crates/orchestrator + crates/providers dispatch modules; new prompts/ dirs per crate. frontend_pair:
  N/A (build-time discipline; W-6 justified — no runtime surface changes).

## WIRING_CONTRACT
- produces_symbols: crates/*/prompts/*.md files + include_str! bindings + the inline-literal ban contract
  test.
- consumes_symbols: D-1 typed template substitution (unchanged — files feed it), the dispatch sites that
  previously held the literals.
- reachable_from: every production agent/LLM dispatch that previously embedded the literal — byte-identical
  output proves reachability (the same prompts flow through the same call sites).
- stub_ban: NO content edits during extraction (byte-preserving; a diff in assembled prompt bytes is a
  regression); NO new template-engine dep; NO partial extraction that leaves the ban test allowlisting
  half the codebase (the allowlist is for non-prompt strings only).
- observable_effect:
  trigger: "assemble the executor + auditor dispatch prompts on a fixture before and after the refactor"
  expect_ssb: "NO new action_type — the assembled-prompt bytes land in existing journaled requests (GD-1);
  equality is asserted there"
  expect_artifact: "byte-identical assembled prompts pre/post refactor (blake3 compare on the journaled
  request bodies); the ban contract test FAILS on a seeded inline literal reintroduction (the negative
  arm); prompts/ files render in a normal git diff."

## Grounding
- STEAL-LIST C1 (Tier 1); SL-10. "Maps onto ELAI D-1 typed templates as a discipline."
- CLAUDE.md one-change-at-a-time (extraction batched per crate, audited per batch).

## COMPOSES-WITH
- D-1 typed templates (the substitution layer — unchanged), I-36 prompt compilation (consumer: compiled
  variants diff against file-based sources), I-47 (its frozen injection block becomes one of these
  files), E-8 GEPA (optimizer reads/writes file artifacts). Restates none. SSB: NO new action_type
  (GD-1; build-time discipline — provenance lands in I-36's PromptCompiled when that ships).

End of I-48.
