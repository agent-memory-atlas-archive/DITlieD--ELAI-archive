# ELAI dashboard: design system rebuild plus an AI build harness

Target: [dashboard source](../../../.elai_cc/crates/dashboard-v2).

> Historical proposal. Its measurements were recorded during development and were
> not rerun for publication. The proposed rebuild was not implemented for this archive.
> See the [user flows](USER-FLOWS.md) and [saved lint baseline](design-lint-baseline.json).

## 1. The diagnosis recorded during development

The dashboard does not lack a design system. It has three copies of one and zero enforcement.

| Written contract | Location | Status |
|---|---|---|
| Design system doc, 20KB | `.elai_cc/crates/dashboard-v2/DESIGN.md` | prose only |
| Styles README, 97 lines | `.elai_cc/crates/dashboard-v2/src/styles/README.md` | prose only |
| 226 CSS tokens | `.elai_cc/crates/dashboard-v2/src/styles/tokens.css` | real, partly ignored |

All three say the same rule: no hex literals outside `tokens.css`, and exactly five font sizes. The measured tree:

- **5159** correct `var(--token)` references. The system is real and mostly used.
- **846** raw hex literals across **57** files, outside the four documented exception files.
- **174** distinct hex colors, against a palette that defines far fewer.
- **84** distinct px values, against a documented 8px spacing grid.
- **47** distinct font sizes, against a documented scale of five. Three sizes carry 1110 uses, then a long tail of 44 off-scale values mixing px and rem for the same job.
- **220** components, **213** carry their own `<style>` block, **109** inline `style=` attributes.
- **Zero** linters configured. No stylelint, no eslint, no prettier, no biome.

Two more structural faults found:

- `tokens.css` runs **two parallel naming systems at once**. `--bg-void/--bg-base/--teal` for app chrome, and `--void/--bg/--panel/--raised/--d-teal*` marked "brand and entry screens only". Same concepts, two vocabularies. That alone guarantees an author picks the wrong one.
- The four "intentional hex exceptions" (`GraphWindow`, `CanvasWindow`, `StarCanvas`, `Intro`) exist only because WebGL, Canvas 2D and Cytoscape cannot read CSS custom properties. That is a solvable problem, not a permanent exception.

**Conclusion: the failure mode is not taste and not the framework. A prose contract with no gate rots, and every new component written by a human or an AI adds drift because nothing says no.**

## 2. What already exists, that we must NOT rebuild

The referenced shared lint tooling is not included in this snapshot. This is the big cost saver. The token gate you want is already written, and it is dead code.

`external-tools/design-lint/design_lint.mjs` runs on `.svelte` files today. The development record reports the following dashboard run:

- Baseline: **335 findings across 143 of 220 components**. Exit 1, fail-closed, 0.6s for the whole tree.
- Breakdown: 95 tiny-text, 78 low-contrast, 49 cramped-padding, 38 all-caps-body, 30 layout-transition, 23 wide-tracking, 12 side-tab, 7 tight-leading, 3 clipped-overflow.
- It found genuinely broken UI, including text at **1.0:1 contrast** (invisible: `#c49b2a` on `#c49b2a`) and body text at 3.5:1 and 3.8:1 where WCAG AA needs 4.5:1.

And the part that matters most: `external-tools/design-lint/engine/design-system.mjs` is a **complete token conformance engine**. It enforces allowed fonts, allowed colors (channel tolerance 6), allowed radii (plus or minus 0.5px) and allowed font sizes (plus or minus 0.5px). It loads them from a `DESIGN.md` YAML frontmatter block plus an optional `DESIGN.json` sidecar.

It never runs. `design_lint.mjs` line 198 calls `lintFile(f)` with no options, and `detect-text.mjs` line 703 gates the whole token leg on `options?.designSystem`. Our own design tooling is wire-dark. Turning it on is a two line change plus a frontmatter block.

Known gap: that engine covers font, color, radius and font-size. It has **no spacing rule**, and spacing is 84 distinct values here. Stylelint covers that gap.

## 3. Architecture: three layers, one source of truth

### Layer 1. One machine-readable token file, everything else generated

New source of truth: `src/styles/tokens.json`, in the W3C Design Tokens (DTCG) shape, with three tiers.

1. **Primitive.** The raw palette. `teal.500 = #1B7A6E`. Never referenced by a component.
2. **Semantic.** Role names, and the only tier components may use. `color.text.primary`, `color.bg.panel`, `color.accent`, `space.4`, `font.size.body`.
3. **Component.** Optional, per component, when a component needs a knob.

A small owned generator, `scripts/build-tokens.mjs`, roughly 150 lines and zero dependencies, emits four artifacts from that one file:

| Output | Consumer | Fixes |
|---|---|---|
| `src/styles/tokens.css` | every `<style>` block | the current hand-maintained file |
| `src/lib/tokens.ts` | three.js, Canvas 2D, Cytoscape | **deletes the four hex exception files as a category** |
| `DESIGN.md` frontmatter | `design_lint` token engine | turns the dead gate on |
| `.stylelintrc` allowed values | stylelint | spacing and scale enforcement |

I recommend owning the generator rather than adopting Style Dictionary. It matches the house pattern of frozen deterministic tools in `.claude/tools/`, adds no dependency, and the DTCG shape keeps Style Dictionary available later if you ever want it.

The `tokens.ts` output is the quiet win. Today WebGL and Canvas code hardcodes `COLOR_HUB = '#2BBBA4'` and hopes it still matches `--d-teal`. After this, both are generated from one line, and they cannot drift.

### Layer 2. The gate, in two legs

**Leg A, stylelint, project-local.** Catches a raw literal where a token belongs.

- `stylelint` + `stylelint-config-html/svelte` (which sets `customSyntax: postcss-html`, the verified 2026 way to lint `<style>` blocks inside `.svelte`).
- `stylelint-declaration-strict-value` with `["/color$/", "font-size", "fill", "stroke"]` and `ignoreFunctions: false`, so `color: #c44040` fails and `color: var(--color-fail)` passes.
- `declaration-property-value-allowed-list` for padding, margin and gap, fed by the generated spacing scale. This is the 84 px values problem, and it is the one thing design-lint cannot see.

**Leg B, design_lint, teikoku-wide.** Catches off-palette values, the anti-slop blacklist and contrast.

- Add YAML frontmatter to `DESIGN.md`, generated from `tokens.json`.
- Change `design_lint.mjs` line 198 to load and pass the design system. That is the wiring fix, and it improves every other teikoku project at the same time.

**The ratchet, and this is the part that decides whether this lands.** You cannot fix 846 violations before turning a gate on, or the gate never turns on. So: commit the 335 finding baseline as `design-lint-baseline.json`, and the gate fails only on findings **not in the baseline**. New code is clean from day one. Old code is fixed in waves, and each wave lowers the baseline. The number only ever goes down.

### Layer 3. The AI harness, which is the tool you asked for

An agent produces drift because when it writes a component it cannot see the legal values, cannot check its work in under a second, and cannot look at the result. Three pieces close that loop.

**3a. A component gallery route.** A dev-only `/__gallery` route in the app that renders every component in isolation with each of its states. Roughly 100 lines, no new framework, and it works with the Playwright you already have.

I checked Storybook and Histoire. Both support Svelte 5 in 2026, Storybook is the safer long-term bet with a bigger ecosystem, Histoire is lighter and Vite-native. My recommendation is the gallery route first, because this repo bans extra tooling on purpose and the gallery costs a fraction of a Storybook install. Adopt Storybook later only if you want controls and docs for other people.

**3b. Screenshots that actually run.** `tests/visual.spec.ts` already exists with 6 tests, and there are **zero baseline snapshots on disk**, so it has never really run. Point Playwright `toHaveScreenshot` at the gallery route, commit the baselines, and the agent gets a before and after image for every change. Self-hosted, no SaaS.

**3c. A `dashboard-ui` skill.** Thin on purpose, because it binds existing tooling to this project rather than duplicating it. Contents:

- the exact path of `tokens.json` and the rule that semantic tier is the only legal tier,
- the three commands: `bun run tokens`, `bun run lint:css`, `bun run test:visual`,
- the authoring recipe for a component (props, the full state set, focus ring, reduced motion),
- the loop the agent must follow: write, lint, render, screenshot, compare, and never claim done without the screenshot.

## 4. Phases

**Phase 0. Freeze the baseline.** Commit `design-lint-baseline.json` (335 findings) and the measured drift counts. Nothing changes visually. This is the number every later phase is judged against.

**Phase 1. Token source of truth.** Author `tokens.json`, collapsing the two parallel naming systems into one semantic tier. Write `build-tokens.mjs`. Regenerate `tokens.css` and prove it is byte-identical in effect, meaning the app renders the same. Emit `tokens.ts` and cut the four hex exception files over to it.

**Phase 2. Gates on, ratcheted.** Add stylelint with the Svelte syntax and the strict-value rule. Wire the `design_lint` token engine. Both run against the baseline and fail only on new findings. Add the npm scripts.

**Phase 3. The harness.** Gallery route, Playwright baselines, the `dashboard-ui` skill.

**Phase 4. Migrate in waves, ratchet down.** Worst offenders first, since they are already known: `ConvergencePanel` (28 hex), `SkillsPanel` (26), `GrammarCompilePanel` (23), `EmpireBridgePanel` (17), `CheckerCoveragePanel` (17). Each wave lowers the committed baseline. Fix the 78 contrast findings early, because those are real defects and one of them is invisible text.

Phases 1 through 3 are the actual investment. Phase 4 is grind that can run at whatever pace you want, and every wave is safe because the gate and the screenshots catch regressions.

## 5. Decisions I need from you

1. **The 10px problem.** 878 declarations use `font-size: 10px`. The linter flags anything under 12px as `tiny-text`, and that produced 95 of the 335 findings. Either the dense 10px look is deliberate and gets a documented waiver, or the body floor rises to 12px and the dashboard gets visibly less dense. This is a taste call, not a technical one, and it changes how the whole app looks.

2. **Gallery route versus Storybook.** My recommendation is the gallery route. Storybook is more capable and more expensive.

3. **Scope of the rebuild.** Tokens rebuilt from zero is settled. The open question is whether component *markup* also gets rewritten, or whether components keep their structure and only re-point at new tokens. The second is far cheaper and preserves the look you said you like.

## 6. What this explicitly does not do

It does not touch the Tauri or Svelte choice. On the measured evidence that is the wrong lever: 368 of 379 Tauri commands are already wired to the UI, and the wire-dark concentration is in `crates/orchestrator` and `crates/cli`, which have no UI involvement. A GUI rewrite would cost roughly 150k lines and would not fix the design drift, because the drift is caused by a missing gate, and a missing gate is missing in any language.
