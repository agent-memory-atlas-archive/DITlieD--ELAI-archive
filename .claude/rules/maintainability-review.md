Maintainability review (HARD RULE, off-agent enforced)

Loaded by: any task that ships code, audits code for "done", or reviews a diff for quality (Rust/Python/TS/Svelte). Auto-loaded on every general-purpose worker agent (it lives in .claude/rules/, which spawned agents inherit by default), so the catalog binds the fleet, not just the main LLM. Pairs with wiring-enforcement.md (does the code RUN), token-discipline.md Zero-Lazy-Code (is the code a stub), and this file (is the code MAINTAINABLE). Three orthogonal axes; all three are auditor-applied.

Why this exists

"Compiles + tests pass + is wired + is not a stub" still ships code that is unmaintainable: a 1200-line file, a function with eight bolted-on conditionals, a thin wrapper that adds a layer for nothing, `as any` papered over a real type. These are the specific entropy an LLM coder adds by default (it takes the shortest path that makes the test green), and our other gates are structurally blind to them. This rule imports the thermo-nuclear maintainability catalog (origin: cursor/plugins cursor-team-kit thermo-nuclear-code-quality-review/SKILL.md; the lifecycle framing converges with addyosmani/agent-skills code-review-and-quality + incremental-implementation). The catalog is excellent; its native enforcement is NOT. Its frontmatter is `disable-model-invocation: true` and its only teeth are a demanding persona, i.e. it relies on the reviewer being honest. An agent reviewing its OWN output rubber-stamps (wiring-enforcement.md, Berkeley RDI Apr-2026 in-boundary artifacts gamed ~100%). So we adopt the catalog content and supply OUR enforcement: the verdict is rendered by the AUDITOR over the changeset, never by the executing agent over its own self-review, and the mechanically-checkable subset is a frozen, diff-grounded checker the agent did not author.

THE EIGHT DIMENSIONS (what the auditor judges every diff against)
0. Structural simplification: can whole branches / helpers / modes / conditionals / layers disappear entirely (not just move)?
1. File-size boundary: does this change push a file from under 1000 lines to over, without a very strong reason?
2. Spaghetti prevention: no ad-hoc conditionals bolted onto unrelated flows.
3. Design cleanliness: bias toward cleaning the design, not just accepting working code.
4. Code directness: prefer direct, boring, maintainable code over clever / magical code.
5. Type & boundary clarity: push hard where types/boundaries affect maintainability.
6. Canonical layering: keep logic in the layer that owns the concept; reuse existing helpers.
7. Orchestration atomicity: unnecessary sequential orchestration and non-atomic updates are design smells.

THE THIRTEEN REVIEW QUESTIONS (the auditor asks these of the diff, in REVIEW)
- Is there a "code judo" move that makes this dramatically simpler?
- Can it be reframed so fewer concepts / branches / helper layers are needed?
- Does this improve or worsen the local architecture?
- Did the diff add branching complexity where a better abstraction should exist?
- Did a cohesive module become more coupled, more stateful, or harder to scan?
- Is this logic in the right file and layer?
- Did this enlarge a file or component past a healthy size boundary?
- Are there repeated conditionals signalling a missing model or helper?
- Is the implementation direct and legible, or does it lean on special cases and incidental control flow?
- Is this abstraction earning its keep, or is it just a wrapper?
- Did the diff introduce casts, optionality, or ad-hoc object shapes that obscure the real invariant?
- Is this logic in the canonical layer, or did it leak details across a boundary?
- Is this orchestration more sequential / less atomic than it needs to be?

FLAG AGGRESSIVELY (presumptive findings; the auditor must hunt these)
- A complicated implementation where cleaner reframing could delete whole complexity categories.
- A refactor that moves code without reducing the concepts a reader must hold in their head.
- A file crossing 1000 lines because of the PR, especially when new code could split out.
- New conditionals bolted onto unrelated code paths.
- One-off booleans, nullable modes, or flags complicating existing control flow.
- Feature-specific logic leaking into general-purpose modules.
- Generic "magic" handling hiding simple structure.
- Thin wrappers / identity abstractions adding indirection without simplifying anything.
- Unnecessary casts, `any`, `unknown`, or optional params muddying the real contract.
- Copy-pasted logic instead of an extracted helper.
- Narrow edge-case handling stuffed into an already-busy function.
- A refactor that passes tests but is less modular or readable (green tests are necessary, NOT sufficient -- mirrors wiring-enforcement R4: measure the objective, not the proxy).
- "Temporary" branching likely to become permanent debt.
- A bespoke helper where the codebase already has a canonical utility.
- Logic added in the wrong layer/package when it should live more centrally.
- Sequential async flow where independent work could stay simpler in parallel.
- Partial-update logic leaving state less atomic than necessary.

PREFERRED REMEDIES (every finding cites a problem AND proposes one of these; "maybe rename this" is not a finding)
- Delete a layer of indirection rather than polishing it.
- Reframe the state model so the conditionals disappear instead of getting centralized.
- Change the ownership boundary so the feature becomes a natural extension of an existing abstraction.
- Turn special-case logic into a simpler default flow with fewer exceptions.
- Extract a helper or pure function.
- Split a large file into smaller focused modules.
- Move feature-specific logic behind a dedicated abstraction.
- Replace condition chains with a typed model or explicit dispatcher.
- Separate orchestration from business logic.
- Collapse duplicate branches into a single clearer flow.
- Delete wrappers that do not meaningfully clarify the API.
- Reuse the existing canonical helper instead of a near-duplicate.
- Make type boundaries explicit so the control flow gets simpler.
- Move logic to the package/module/layer that already owns the concept.
- Parallelize independent work when that also simplifies the orchestration.
- Restructure related updates into a more atomic flow.

TRIAGE ORDER (spend the budget on majors; do not nit while a major stands)
Order findings: (1) structural code-quality regression, (2) missed simplification, (3) spaghetti / branching complexity, (4) boundary / abstraction problem, (5) file-size, (6) modularity, (7) legibility. If a major exists, do not surface cosmetic nits in the same pass.

VERDICT SCHEMA (anti-rubber-stamp, mirrors visual-ui-loop.md)
Each dimension the auditor reports gets pass | fail | unknown. unknown is treated as FAIL -- a dimension the auditor cannot judge from the diff is not passing; re-read the code, do not wave it through. A "fail" must name the specific construct (file:line / fn / symbol) AND a remedy from the menu. "Looks clean overall" with no evidence is not a verdict. Be direct, serious, and demanding; do not approve merely because behavior seems correct.

THE DETERMINISTIC SUBSET (off-agent, frozen, fail-closed)
The auditor runs .claude/tools/maintainability_check.py over the changeset (NOT a self-review; the executing agent never runs its own maintainability gate). It is a Layer-2 static, regex-unsound, necessary-not-sufficient leg -- the qualitative catalog above is the rest. Checks:
- MR-1 file-size boundary: a diff pushing a file across 1000 lines = BLOCK; an already-over-1000 file grown by > 50 lines = WARN.
- MR-2 type-erasure escape hatches introduced by the diff: `as any` / `as unknown` / @ts-ignore / @ts-nocheck / new `unsafe {` = BLOCK; `: any` / `<any>` / `as <numeric>` / #[allow(..)] / eslint-disable / # type: ignore / typing.cast / # noqa = WARN.
- MR-3 a change that touches/grows a function over 80 lines = WARN (extract / split).
- MR-4 net-added branching density over 10 keywords in one diff = WARN (typed model / dispatcher).
Usage: python .claude/tools/maintainability_check.py --files <changed> --base HEAD --json  (or --git-staged). Exit 1 on any BLOCK (or any WARN under --warn-as-block); exit 2 on scan-config error. Thresholds are named constants in the checker; the checker + this rule are frozen in .claude/gate-manifest.txt (R8) so an executing plan run cannot weaken the 1000-line boundary or delete the catalog.

FINDING TAGS (for the auditor's per-level output, B-41 C-* convention)
C-MAINTAINABILITY-FILESIZE (MR-1), C-MAINTAINABILITY-CAST (MR-2), C-MAINTAINABILITY-LONGFN (MR-3), C-MAINTAINABILITY-SPAGHETTI (MR-4), C-MAINTAINABILITY-WRAPPER / C-MAINTAINABILITY-LAYER / C-MAINTAINABILITY-DUP for the qualitative catalog items that have no deterministic check. BLOCK on the MR-2-BLOCK + MR-1-crossing classes and on any qualitative "fail"; WARN otherwise.

ROLE BOUNDARY
This is the AUDITOR's job. The main LLM never renders the maintainability verdict and never runs the checker as a self-pass (consistent with "main model never runs tests"). The deterministic checker is an auditor-invoked, frozen artifact. An executing agent that runs maintainability_check.py on its own diff and reports PASS has reviewed itself -- the same in-boundary failure the off-agent rule exists to prevent.

SELF-CHECK before declaring any diff maintainable
1. Could a reframing delete a whole branch / layer / mode (dimension 0)?
2. Did this push a file over 1000 lines, or grow an already-large one (MR-1)?
3. Did it add `as any` / `unknown` / a new unsafe block / a lint suppression (MR-2)?
4. Did it stuff edge-case logic into an already-long function (MR-3) or pile conditionals onto an unrelated flow (MR-4 / spaghetti)?
5. Is every "fail" paired with a concrete remedy, and is every unjudged dimension marked unknown (= fail), not waved through?
If any structural answer is yes and unaddressed, it is not done -- it is debt.
