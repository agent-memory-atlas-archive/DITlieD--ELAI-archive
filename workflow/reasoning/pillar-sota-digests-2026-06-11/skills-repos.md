SKILLS REPOS DIGEST — pillar SOTA recheck 2026-06-11
task_id=skills-repos-read
Sources fetched live from GitHub API + raw.githubusercontent.com on 2026-06-11. All facts below are from those fetches unless marked UNVERIFIED.


==============================================================================
REPO FACTS
==============================================================================

REPO 1 — mattpocock/skills
  full_name: mattpocock/skills  (CONFIRMED exists at stated owner via api.github.com)
  stars: 125249  (NOT ~100K as the task estimated; real number is ~125K)
  license: MIT (SPDX_ID from API)
  default_branch: main
  pushed_at: 2026-06-10T15:01:47Z ; updated_at: 2026-06-11T13:46:35Z  (active, edited yesterday)
  description: "Skills for Real Engineers. Straight from my .claude directory."
  blob count (git tree, recursive, NOT cut off): 72 files
  skill count: ~30 SKILL.md files across bucket folders.
  Category layout (from CLAUDE.md + tree):
    engineering/   daily code work: diagnose, grill-with-docs, improve-codebase-architecture,
                   prototype, tdd, to-issues, to-prd, triage, zoom-out, setup-matt-pocock-skills
    productivity/  daily non-code: caveman, grill-me, handoff, teach, write-a-skill
    misc/          kept-but-rare: git-guardrails-claude-code, migrate-to-shoehorn,
                   scaffold-exercises, setup-pre-commit
    personal/      not promoted: edit-article, obsidian-vault
    in-progress/   drafts: review, writing-beats, writing-fragments, writing-shape
    deprecated/    design-an-interface, qa, request-refactor-plan, ubiquitous-language
  CLAUDE.md encodes a repo-maintenance rule (not an agent-behavior rule): every shippable
  skill must be referenced in top-level README.md AND .claude-plugin/plugin.json; personal/
  in-progress/deprecated must NOT appear in either.

REPO 2 — cursor/plugins  (the thermo-nuclear skill lives under cursor-team-kit/)
  full_name: cursor/plugins  (CONFIRMED exists at stated owner via api.github.com)
  stars: 1932
  license (repo-root API): None/null. BUT cursor-team-kit ships its own LICENSE file and its
    README.md states "## License  MIT". So the team-kit subtree is MIT; the umbrella repo
    has no root SPDX license detected.
  default_branch: main ; pushed_at: 2026-06-07T03:56:05Z (active)
  description: "Cursor plugin specification and official plugins"
  cursor-team-kit blob count: 29 files.
  cursor-team-kit/skills/ catalog (18 skills): check-compiler-errors, control-cli, control-ui,
    deslop, fix-ci, fix-merge-conflicts, get-pr-comments, loop-on-ci, make-pr-easy-to-review,
    new-branch-and-pr, pr-review-canvas, review-and-ship, run-smoke-tests,
    thermo-nuclear-code-quality-review, verify-this, weekly-review, what-did-i-get-done,
    workflow-from-chats.
  Also ships: agents/ (ci-watcher.md, thermo-nuclear-code-quality-review.md — a Task subagent
    variant of the skill), rules/ (no-inline-imports.mdc, typescript-exhaustive-switch.mdc —
    Cursor "alwaysApply" rule files), and pr-review-canvas ships real executable assets
    (renderer.js, styles.css, template.html).


==============================================================================
MECHANISM (the key question)
==============================================================================

VERDICT: Both repos are PROMPT-BASED. The load-bearing content of essentially every skill is
markdown prose that is injected into the model's context when a trigger phrase matches the
skill's frontmatter `description`. There is NO runtime gate, NO checker that fails a build, NO
deterministic enforcement on the directive content itself. The skill "acts" only by steering
what the model chooses to do next. This is true for BOTH repos.

Frontmatter shape (both repos use the agentskills.io SKILL.md v1 form):
  ---
  name: <id>
  description: <what it does>. Use when <triggers>.   <- the ONLY thing the model sees pre-load
  [argument-hint / disable-model-invocation: true]    <- optional
  ---
  <markdown body = the actual instructions, loaded on trigger>

Two skills carry `disable-model-invocation: true` (mattpocock/zoom-out and cursor/thermo-nuclear)
meaning the model will not auto-trigger them; they fire only on explicit user invocation.

EXECUTABLE TOOLING — the ONLY places either repo ships real code rather than prose:
  - mattpocock/skills/misc/git-guardrails-claude-code/scripts/block-dangerous-git.sh  — a REAL
    PreToolUse hook (bash + jq). It reads the tool_input.command JSON, greps it against a
    DANGEROUS_PATTERNS array (git push, git reset --hard, git clean -f/-fd, git branch -D,
    git checkout ., git restore ., push --force, reset --hard) and exit 2 + stderr "BLOCKED" on
    a match. This is a genuine deterministic gate — but the SKILL itself is just setup
    instructions that copy the script into .claude/hooks/ and wire it into settings.json hooks.
    The enforcement is the hook, not the skill prose.
  - mattpocock/skills/engineering/diagnose/scripts/hitl-loop.template.sh — a bash template for a
    human-in-the-loop debug loop (referenced as a last-resort feedback-loop mechanism).
  - cursor/.../pr-review-canvas/{renderer.js,styles.css,template.html} — front-end assets for an
    HTML PR walkthrough (not a gate; a rendering tool).
  - mattpocock improve-codebase-architecture instructs the model to GENERATE a self-contained
    Tailwind+Mermaid HTML report at runtime (generated artifact, not bundled tooling).
Everything else in both repos is pure prose.

SO: re-expressing this content as deterministic gates in a non-prompt harness is feasible ONLY
for the [CHECKLIST]-classed directives below (enumerable, mechanically checkable). The
[STYLE-PREF] directives are judgment calls a static tool cannot adjudicate without an LLM.


==============================================================================
DIRECTIVE EXTRACTION (by skill; classification tags per directive)
==============================================================================
Tags: [STYLE-PREF] subjective code taste | [CHECKLIST] enumerable static-checkable item |
      [PROCESS] session/workflow management | [SAFETY] redaction/guardrail.

----- mattpocock: handoff (productivity) -----  [see THE HANDOFF SKILL section for full text]
  - Write handoff doc so a fresh agent can continue; save to OS TEMP dir, NOT the workspace. [PROCESS]
  - Include a "suggested skills" section naming skills the next agent should invoke. [PROCESS]
  - Do not duplicate content already in other artifacts (PRDs/plans/ADRs/issues/commits/diffs);
    reference them by path or URL instead. [PROCESS]
  - Redact sensitive info (API keys, passwords, PII). [SAFETY]
  - If args passed, treat them as the next-session focus and tailor the doc. [PROCESS]

----- mattpocock: grill-me (productivity) -----
  - "Interview me relentlessly about every aspect of this plan until we reach a shared
    understanding. Walk down each branch of the design tree, resolving dependencies between
    decisions one-by-one. For each question, provide your recommended answer." [PROCESS]
  - Ask questions ONE AT A TIME. [PROCESS]
  - "If a question can be answered by exploring the codebase, explore the codebase instead." [PROCESS]

----- mattpocock: grill-with-docs (engineering) -----  (grill-me + domain-doc discipline)
  - Same one-at-a-time relentless interview, waiting for feedback each question. [PROCESS]
  - Maintain a CONTEXT.md glossary (+ CONTEXT-MAP.md for multi-context repos) and docs/adr/;
    create files LAZILY only when there's something to write. [PROCESS]
  - Challenge user terms that conflict with the glossary; sharpen fuzzy/overloaded terms to a
    canonical one ("'account' — Customer or User?"). [STYLE-PREF]
  - Cross-reference claims against code; surface contradictions. [PROCESS]
  - "CONTEXT.md should be totally devoid of implementation details ... It is a glossary and
    nothing else." [CHECKLIST] (glossary purity could be lint-checked structurally)
  - Offer an ADR ONLY when all three true: hard-to-reverse, surprising-without-context, result
    of a real trade-off. [PROCESS]

----- mattpocock: tdd (engineering) -----
  - "Tests should verify behavior through public interfaces, not implementation details." [STYLE-PREF]
  - Good tests are integration-style through public APIs; bad tests mock internal collaborators
    / test private methods / verify via side-channels (e.g. querying DB directly). Warning
    sign: test breaks on refactor though behavior unchanged. [STYLE-PREF]
  - ANTI-PATTERN "horizontal slices": DO NOT write all tests first then all impl. Use VERTICAL
    slices / tracer bullets: one test -> one impl -> repeat. [PROCESS]
  - "Never refactor while RED. Get to GREEN first." [CHECKLIST] (ordering rule)
  - Per-cycle checklist: test describes behavior not impl; uses public interface only; would
    survive internal refactor; code minimal for this test; no speculative features. [STYLE-PREF / partly CHECKLIST]

----- mattpocock: diagnose (engineering) -----  (6-phase disciplined debug loop)
  - Phase 1 IS the skill: build a fast, deterministic, agent-runnable pass/fail feedback loop
    BEFORE anything else (10 ranked ways: failing test, curl, CLI+snapshot diff, headless
    browser, replay captured trace, throwaway harness, property/fuzz, bisection, differential,
    HITL bash). "Do not proceed to hypothesise without a loop." [PROCESS]
  - Phase 3: generate 3-5 RANKED, FALSIFIABLE hypotheses before testing any; format "If <X> is
    the cause, then <changing Y> will make the bug disappear". Show ranked list to user. [PROCESS]
  - Phase 4: change ONE variable at a time; prefer debugger/REPL over logs; never "log
    everything and grep"; TAG every debug log with a unique prefix e.g. [DEBUG-a4f2] so
    cleanup is one grep. [CHECKLIST] (tagged-log-cleanup is mechanically verifiable)
  - Phase 5: write the regression test BEFORE the fix, but only if a correct seam exists; if
    no correct seam, that absence IS the finding (flag for architecture work). [PROCESS]
  - Phase 6 done-gate checklist: original repro no longer reproduces; regression test passes
    (or seam-absence documented); all [DEBUG-...] removed (grep the prefix); throwaway
    prototypes deleted; the correct hypothesis stated in commit/PR msg. [CHECKLIST]

----- mattpocock: improve-codebase-architecture (engineering) + LANGUAGE.md -----
  - Enforces a FIXED vocabulary (Module/Interface/Implementation/Depth/Seam/Adapter/Leverage/
    Locality) and bans drift into "component/service/API/boundary". [CHECKLIST] (banned-word
    lint over the skill's OWN output is mechanically checkable; over arbitrary code is not)
  - Deletion test: imagine deleting a module; if complexity vanishes it was a pass-through, if
    it reappears across N callers it earned its keep. [STYLE-PREF]
  - "The interface is the test surface." "One adapter = hypothetical seam; two adapters = real
    seam" (don't introduce a seam unless something varies across it). [STYLE-PREF]
  - Output = a self-contained Tailwind+Mermaid HTML report in OS temp dir, before/after
    diagrams per candidate, recommendation-strength badge (Strong/Worth exploring/Speculative). [PROCESS]
  - Rejected framing: depth-as-impl-lines/interface-lines ratio (Ousterhout) "rewards padding". [STYLE-PREF]

----- mattpocock: zoom-out (engineering, disable-model-invocation) -----
  - "Go up a layer of abstraction. Give me a map of all the relevant modules and callers,
    using the project's domain glossary vocabulary." [PROCESS]

----- mattpocock: prototype (engineering) + LOGIC.md + UI.md -----
  - A prototype is "throwaway code that answers a question"; the question picks a branch:
    LOGIC (interactive terminal app driving a state machine) vs UI (N radically-different
    variants on one route, ?variant= switch + floating bottom bar). [PROCESS]
  - Shared rules: throwaway-from-day-one and clearly marked; ONE command to run; no
    persistence by default; skip polish (no tests, no abstractions); surface full state after
    each action; delete-or-absorb when done. [CHECKLIST] (several are checkable: no tests added,
    one run-command, prototype-named-as-prototype, hidden in prod builds)
  - UI variants must be STRUCTURALLY different (layout/hierarchy/affordance), not just colour;
    default 3, cap 5; switcher hidden when NODE_ENV==production. [STYLE-PREF + CHECKLIST]

----- mattpocock: caveman (productivity) -----
  - Ultra-compressed output mode; drop articles/filler/pleasantries/hedging; keep technical
    substance exact; persists every response until "stop caveman"/"normal mode". [STYLE-PREF/PROCESS]
  - Auto-clarity exception: drop caveman for security warnings, irreversible-action
    confirmations, multi-step sequences, clarifications. [SAFETY]

----- mattpocock: write-a-skill (productivity) -----
  - Skill-authoring meta-rules: description max 1024 chars, third person, sentence 1 = what,
    sentence 2 = "Use when [triggers]"; SKILL.md under 100 lines; split into REFERENCE.md when
    >100 lines / >500 lines for resources; add scripts when the op is deterministic
    (validation/formatting) to "save tokens and improve reliability vs generated code";
    references one level deep; no time-sensitive info. [CHECKLIST]

----- mattpocock: to-prd / to-issues (engineering) -----
  - to-prd: synthesize a PRD from conversation (don't re-interview), pick test SEAMS (prefer
    existing/highest seam), publish to issue tracker with ready-for-agent label; PRD template
    bans specific file paths/code snippets ("they go stale fast") except prototype-derived
    decision snippets. [PROCESS + CHECKLIST(no-file-paths)]
  - to-issues: break work into TRACER-BULLET vertical slices cutting ALL layers
    (schema/API/UI/tests), each demoable alone; mark HITL vs AFK, prefer AFK; quiz user on
    granularity/deps before publishing in dependency order. [PROCESS]

----- mattpocock: in-progress/review (engineering, draft) -----
  - Two-axis review of diff vs a user-pinned fixed point: STANDARDS (conforms to repo's
    documented coding standards: CLAUDE.md/AGENTS.md/CONTRIBUTING.md/CONTEXT.md/docs/adr/
    /STYLE.md etc — but "don't re-check what tooling already checks": .editorconfig/eslint/
    biome/prettier/tsconfig) and SPEC (matches originating issue/PRD). Runs as TWO parallel
    general-purpose sub-agents so contexts don't pollute; aggregates side by side. [PROCESS +
    CHECKLIST(standards-doc discovery)]

----- mattpocock: git-guardrails-claude-code (misc) -----  [the one true gate]
  - Installs a PreToolUse hook (block-dangerous-git.sh) that BLOCKS at the tool layer:
    git push (incl --force), git reset --hard, git clean -f/-fd, git branch -D,
    git checkout ., git restore .  -> exit 2 + "BLOCKED" stderr. [SAFETY, deterministic]
  - The directive is enforced by bundled bash+jq, NOT by prose. This is the one piece directly
    portable to a non-prompt harness as-is (it already IS a deterministic gate).

----- cursor: thermo-nuclear-code-quality-review (team-kit, disable-model-invocation) -----
  THE core code-quality skill. A strict maintainability-review RUBRIC (pure prose; the model is
  the judge). Concrete directives (load-bearing ones quoted verbatim):
  - [CHECKLIST] HARD NUMERIC RULE: "Do not let a PR push a file from under 1k lines to over 1k
    lines without a very strong reason." Treated as a presumptive blocker. (This is the single
    most mechanically-portable rule in the cursor kit: line-count delta across the 1000 line
    threshold is a deterministic check.)
  - [STYLE-PREF] "Prefer direct, boring, maintainable code over hacky or magical code." Flag
    brittle/ad-hoc/"magic" behavior, generic mechanisms hiding data-shape assumptions, thin
    abstractions / identity wrappers / pass-through helpers that add indirection without clarity.
  - [STYLE-PREF] "Be ambitious about structural simplification" / "code judo": prefer deleting
    whole branches/helpers/modes/layers over rearranging; "make the code feel inevitable in
    hindsight."
  - [STYLE-PREF] No "random spaghetti growth": be suspicious of new ad-hoc conditionals /
    scattered special cases / one-off branches bolted into unrelated flows; push logic into a
    dedicated abstraction/helper/state-machine/policy object/module.
  - [STYLE-PREF] Type/boundary cleanliness: question unnecessary optionality, `unknown`, `any`,
    cast-heavy code; prefer explicit typed models/shared contracts; flag silent fallbacks that
    paper over an unclear invariant. (Partly [CHECKLIST]: presence of `any`/`unknown`/casts in a
    diff is greppable, though the "necessary?" judgment is not.)
  - [STYLE-PREF] Keep logic in the canonical layer; reuse existing canonical helpers over
    bespoke near-duplicates; flag feature logic leaking into shared paths.
  - [STYLE-PREF] Flag unnecessary sequential orchestration / non-atomic (half-applied state)
    updates when a cleaner structure is obvious.
  - APPROVAL BAR (the rubric's done-gate): do NOT approve merely because behavior is correct.
    Presumptive blockers: preserves incidental complexity when a code-judo move exists; pushes a
    file <1000 -> >1000 lines; adds ad-hoc branching tangling an existing flow; scatters feature
    checks across shared code; adds unnecessary abstraction/wrapper/cast-heavy contract;
    duplicates an existing helper or puts logic in the wrong layer. [mostly STYLE-PREF; the
    1k-line one is CHECKLIST]
  - Output: prioritized findings (structural regressions > missed simplifications > spaghetti >
    boundary/type > file-size > modularity > legibility); "Prefer a smaller number of
    high-conviction comments over a long list of cosmetic notes." [PROCESS]
  - Also ships as a Task SUBAGENT (agents/thermo-nuclear-code-quality-review.md) running the
    same rubric against a diff.

----- cursor: deslop (team-kit) -----
  - Remove AI-generated "slop" from the branch diff vs main. Focus: unnecessary/inconsistent
    comments; defensive checks or try/catch abnormal for trusted code paths; casts to `any` used
    only to bypass type issues; deeply nested code that should use early returns; patterns
    inconsistent with the surrounding file. [STYLE-PREF; `any`-cast + try/catch + nesting depth
    are partly greppable -> CHECKLIST-adjacent]
  - Guardrails: keep behavior unchanged unless fixing a clear bug; minimal focused edits over
    broad rewrites; final summary 1-3 sentences. [PROCESS]

----- cursor: verify-this (team-kit) -----
  - Verification is not a recap; prove/disprove a falsifiable claim with baseline-vs-treatment
    evidence. Restate claim as condition+metric+threshold; capture baseline (merge base/parent/
    failing branch) and treatment with SAME command/data/env; compare raw artifacts; return
    EXACTLY one verdict VERIFIED | NOT VERIFIED | INCONCLUSIVE. Artifact layout under
    /tmp/verify-this/<slug>/ (claim.md, baseline/, treatment/, diff/, verdict.md). [PROCESS]
  - "Do not soften a negative result." Refuse vague claims ("the code is cleaner") — demand a
    measurable claim first. [PROCESS]
  - (Mirrors this project's own verify-semantics / SAFEDELTA posture conceptually; pure prose.)

----- cursor: review-and-ship (team-kit) -----
  - Diff vs base, run TARGETED tests for changed behavior (add or document the gap), review for
    correctness/regressions/security/intent-fit (parallel subagents for larger diffs), fix
    critical issues, commit focused files, push + open/update PR. [PROCESS]
  - Guardrails: prioritize correctness/security/regressions over style-only; keep commits
    focused / no unrelated changes; "If pre-commit checks fail, fix the issues rather than
    bypassing hooks"; use `gh pr checks` not Actions-only commands. [PROCESS + CHECKLIST(no-hook-bypass)]

----- cursor: team-kit RULES (alwaysApply .mdc, not skills) -----
  - typescript-exhaustive-switch: in switch over discriminated unions/enums, use a `never` check
    in default so new variants fail to compile until handled. [CHECKLIST — a compiler-enforceable
    rule; mechanically portable as a lint/typecheck convention]
  - no-inline-imports: keep imports at module top-level; no inline imports in bodies/annotations
    unless a documented circular-dep reason. [CHECKLIST — greppable/lintable]


==============================================================================
THE HANDOFF SKILL (full content summary — task asked for this explicitly)
==============================================================================
File: skills/productivity/handoff/SKILL.md. Frontmatter: name=handoff,
description="Compact the current conversation into a handoff document for another agent to pick
up.", argument-hint="What will the next session be used for?". Body is 5 short directives (the
entire skill, verbatim shape):
  1. Write a handoff doc summarising the current conversation so a fresh agent can continue.
     WHERE FILES GO: "Save to the temporary directory of the user's OS - not the current
     workspace." (explicitly OUT of the repo).
  2. SUGGESTED-SKILLS SECTION: "Include a 'suggested skills' section in the document, which
     suggests skills that the agent should invoke."
  3. NO-DUPLICATION POINTER RULE: "Do not duplicate content already captured in other artifacts
     (PRDs, plans, ADRs, issues, commits, diffs). Reference them by path or URL instead."
  4. REDACTION RULE: "Redact any sensitive information, such as API keys, passwords, or
     personally identifiable information."
  5. ARGS-AS-FOCUS RULE: "If the user passed arguments, treat them as a description of what the
     next session will focus on and tailor the doc accordingly."
That is the complete skill — pure prose, no script, no template file, no enforcement. Contrast
with this project's own handoff-protocol.md, which is far more prescriptive (one topic-keyed
file IN-repo at workflow/handoff/{slug}.md, append-not-spawn, living CURRENT STATUS + INDEX
fields, head-only read). mattpocock's is minimal and points files OUT of the workspace.


==============================================================================
NOTABLE — NOT expressible as a deterministic check (judgment calls)
==============================================================================
The bulk of BOTH repos' code-quality value is irreducibly LLM-judgment:
  - "code judo" / "make the code feel inevitable in hindsight" / "is this abstraction earning
    its keep or just a wrapper" / "direct boring maintainable vs hacky or magical" — these are
    taste verdicts; a static tool cannot adjudicate them without an LLM judge.
  - the deletion test, "interface is the test surface", "two adapters = a real seam" — heuristics
    a human/LLM applies, not predicates.
  - "tests verify behavior not implementation" — the behavior/impl distinction is semantic.
  - grilling-loop relentless Socratic interview — inherently interactive.
DETERMINISTICALLY PORTABLE subset (could become real gates with no LLM):
  - git-guardrails block-dangerous-git.sh (already a hook).
  - cursor 1000-line-file-growth threshold (line-count delta check).
  - cursor typescript-exhaustive-switch `never`-default (compiler/lint rule).
  - cursor no-inline-imports (lint rule).
  - presence-greps that FLAG (not adjudicate): `any`/`unknown`/casts (deslop, thermo-nuclear),
    [DEBUG-*] leftover tags (diagnose Phase 6), "no hook bypass" / "no test-file-only callers"
    style checks, CONTEXT.md-contains-no-code-blocks (glossary purity), SKILL.md <100 lines /
    description <1024 chars (write-a-skill).
  - diagnose/verify-this/tdd done-gate CHECKLISTS are enumerable steps a harness can require
    evidence for (repro-no-longer-reproduces, regression-test-exists, baseline+treatment
    captured) — i.e. they map onto a wiring/verification gate, though the EVIDENCE still needs
    runtime artifacts, not prose.
