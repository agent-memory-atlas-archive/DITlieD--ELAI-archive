# H-16: Retire the orbit — delete the three.js layer after Canvas-as-default ships a full release

STATUS: PLANNED (WAVE 2; DELETE; LAST in the plan; the cleanest, most mechanical, ungameable gate)

Objective: Delete the three.js scene/ layer + OrbitMode1/2/3 + the three/troika deps ONLY after Canvas-as-default
(H-12) has shipped a FULL release with no regressions. The canvas-store isolation gate already PASSES (C12 — no Svelte
runes store imports three.js), so removal is clean: there is no store leak to untangle. The starfield aesthetic survives
as the cheap 2D layer (H-10/§10), so nothing visual is lost. This is the cleanest gate in the whole plan — purely
mechanical, off-agent, ungameable (a green build with the deps removed and a zero-grep for orbit imports).

Estimated effort: ~0.5-1 session (~3-5 h). Deletion + a dep drop + a default-flag flip + a partial-file trace.
Pre-requisites: H-12 (Canvas-as-default + the single-consumer SSB handover) MUST have shipped a FULL release with no
  regressions; the orbit must have been the FALLBACK in a tab until then; H-10/H-11 (the canvas + the 2D starfield).
Parallel-safe with: nothing (it removes shared render infrastructure). The final session.
Source: CANVAS-WORKBENCH-DESIGN.md §2 (what dies with three.js) + §12 step 6 (retire) + §13 C6; _grounding/H-GROUNDING-MAP.md C12 + the H-16 table. Doctrine: H-0_INDEX.md.

## Context

The orbit was good at exactly one thing — making a running engine feel alive to someone who could not read the code —
and that emotional payload is RECOVERED by the H-11 live layer on a legible flat surface (CANVAS §2/§5). Everything else
the orbit cost (watch-not-work, heavy 3D, hard-to-make-deterministic, impossible to turn into a Figma-style editing
surface) is what the canvas replaces. The migration is incremental and the orbit is the fallback at every step until
THIS one (CANVAS §12): the orbit keeps running until the canvas earns its place, the default is flipped behind a workspace
flag (H-12) with the single-consumer SSB handover, and only after a full release of Canvas-as-default with NO regressions
does the three.js layer get deleted. What dies (CANVAS §2, grep-confirmed C12/H-16 table): scene/three-scene.ts,
three-orbit.ts, three-orbit-modes.ts, StarCanvas.svelte (the renderloop compositor), sky-dome.ts, star-points.ts, all
mesh materials/geometries, troika billboarding, orbit/node-mandala.ts, OrbitCanvas.svelte, ActivityWindow's orbit render,
and the three/troika-three-text/@types/three deps. The starfield aesthetic survives as a 2D-canvas layer (already shipped by H-10/§10).

## Existing state (B/C/D/E/F/G coverage)
- three (package.json:23) — ^0.184.0 + troika-three-text ^0.52.4 + @types/three ^0.184.0. Reached via the live activity render.
- scene/three-scene.ts:14 (createThreeScene), three-orbit.ts:22, three-orbit-modes.ts:26, StarCanvas.svelte:21 (~1650 lines),
  sky-dome.ts:26, star-points.ts:12 — SHIPPED, all import THREE; imported by StarCanvas.svelte.
- orbit/node-mandala.ts:232 (class NodeMandala) — SHIPPED. import THREE + TroikaText; a CLASS (use `new`); imported by StarCanvas + OrbitCanvas.
- orbit/lcg.ts (lcg, hashId, combineSeeds, jitterFromId) — SHIPPED, NOT test-only. KEEP (the canvas seed source, H-10/§10).
- OrbitCanvas.svelte:434 — SHIPPED. Renders <Starfield>; rendered by ActivityWindow.svelte:119.
- ActivityWindow.svelte:119 — SHIPPED. The DEFAULT panel (workspace.svelte.ts:82 `primary = makePanel('activity')`); the live entrypoint that activates the whole three.js layer.
- ui/Starfield.svelte:106 — SHIPPED (the mount shim bridging non-three orbit DOM and the three layer).
- scene/planet-scan.ts, star-shoot.ts, star-state.ts — PARTIAL. planet-scan.ts imports THREE; prod caller unpinned. FLAG for a deeper trace before removal.
- the canvas-store isolation gate — PASSES (C12; no store imports three.js — clean removal).

## Reads (mandatory before coding)
- _grounding/H-GROUNDING-MAP.md C12 + the H-16 symbol table.
- CANVAS-WORKBENCH-DESIGN.md §2 (what keeps / converts / dies with three.js) + §10 (the 2D starfield that survives) + §12 step 6 (the retire gate) + §13 C6 + §14 (the orbit-removal cost + the open "galaxy view toggle" question).
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\package.json:20-25 (the three deps) + src/lib/scene/ (the layer) + src/lib/orbit/ (node-mandala + lcg) + windows/ActivityWindow.svelte + state/workspace.svelte.ts:82
- CLAUDE.md HARD RULES; CANVAS §14 (the galaxy-view-toggle open question — surface the decision, do not silently delete the option).

## Tasks (WAVE 2 — delete, gated)

1. **Confirm the gating prerequisite (~0 LOC, a gate check).** Verify Canvas-as-default (H-12) shipped a FULL release
   with no regressions and the orbit was the fallback in a tab until now. If not, H-16 does not run. This is the W-1 precondition.

2. **Trace the partial files before removal (~20 LOC of investigation).** planet-scan.ts / star-shoot.ts / star-state.ts
   have an unpinned prod caller (PARTIAL in grounding) — trace them and confirm they are only reachable via the orbit
   render chain before deleting (a missed live caller is a regression).

3. **Delete the three.js layer + OrbitMode1/2/3 (~deletions).** Remove scene/three-scene.ts + three-orbit*.ts +
   StarCanvas.svelte + sky-dome.ts + star-points.ts + orbit/node-mandala.ts + OrbitCanvas.svelte + ui/Starfield.svelte +
   the traced planet-scan/star-shoot/star-state; remove ActivityWindow's orbit render (or ActivityWindow if it has no
   non-orbit role). KEEP orbit/lcg.ts (the canvas seed source). Flip the workspace default off 'activity' (workspace.svelte.ts:82) to the canvas.

4. **Drop the three deps (~3 LOC package.json).** Remove three + troika-three-text + @types/three from package.json.

5. **Decision doc workflow/reasoning/H-16-decisions.md (~80 LOC).** Records: the gating prerequisite (full release of
   Canvas-as-default first); the isolation gate PASSES (clean removal); lcg.ts is KEPT; the partial-file trace; the
   2D starfield survives; the open galaxy-view-toggle question (CANVAS §14 — surface the decision: a permanent calm-mode toggle, or full deletion).

## WIRING_CONTRACT
produces_symbols:
  - crates/dashboard-v2/package.json (three / troika-three-text / @types/three REMOVED)
  - crates/dashboard-v2/src/lib/state/workspace.svelte.ts (the default panel flipped off 'activity')
  - (deletions) crates/dashboard-v2/src/lib/scene/* + orbit/node-mandala.ts + windows/ActivityWindow.svelte (orbit render) + ui/Starfield.svelte
consumes_symbols:
  - the canvas-store isolation gate (C12, PASSES — the precondition that makes deletion clean)
  - crates/dashboard-v2/src/lib/state/workspace.svelte.ts:82 (the default panel, must already be canvas by H-12)
  - crates/dashboard-v2/src/lib/orbit/lcg.ts (KEPT — the canvas seed source; NOT deleted)
reachable_from:
  - the dashboard build (cargo + svelte-check) -> GREEN with three removed + no orbit imports  [the mechanical off-agent gate]
  - workspace.svelte.ts:82 (the default panel) -> canvas (the orbit entrypoint is gone)
stub_ban:
  - NO deletion before Canvas-as-default ships a FULL release with no regressions (the H-12 prerequisite)
  - NO deletion of orbit/lcg.ts (the canvas seed source — KEEP)
  - NO deletion of a partial file (planet-scan/star-shoot/star-state) before tracing its prod caller
  - NO silent removal of the galaxy-view option without surfacing the decision (CANVAS §14)
  - the build MUST be GREEN with three/troika/@types/three gone AND a zero-grep for `from .*scene/three`
frontend_pair:
  the dashboard renders the canvas with no regression after the three deps are gone. W-6 dashboard E2E via dashboard-smoker.
observable_effect:
  trigger: "remove the three deps + the scene/ layer; run the dashboard build + open the dashboard (NO env override)"
  expect_ssb: "N/A (this is a deletion; no new SSB row). The off-agent artifact is the build state, not an SSB row."
  expect_artifact: "the dashboard build is GREEN with three / troika-three-text / @types/three removed; a zero-grep for
    `from .*scene/three` and no orbit imports remain (the off-agent, ungameable artifact = a clean build + the zero-grep);
    the dashboard renders the canvas with no regression (verified through the dashboard by dashboard-smoker)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-16_retire_orbit.md --run-trigger
grep -rE "from .*lib/scene/three|troika-three-text" .elai_cc/crates/dashboard-v2/src   # MUST be empty (the layer is gone)
rg "\"three\"|troika-three-text|@types/three" .elai_cc/crates/dashboard-v2/package.json # MUST be 0 (deps dropped)
test ! -f .elai_cc/crates/dashboard-v2/src/lib/scene/StarCanvas.svelte                  # the renderloop compositor is gone
test -f .elai_cc/crates/dashboard-v2/src/lib/orbit/lcg.ts                                # lcg.ts is KEPT (the canvas seed source)
rg "makePanel\('activity'\)|primary = makePanel" .elai_cc/crates/dashboard-v2/src/lib/state/workspace.svelte.ts  # default flipped off activity
test -f workflow/reasoning/H-16-decisions.md
```

## ACCEPTANCE
- [ ] PRECONDITION: Canvas-as-default (H-12) shipped a FULL release with no regressions; the orbit was the fallback in a tab until now.
- [ ] W-1 done-gate: --run-trigger PASS (the build is GREEN with three removed; zero-grep for orbit imports; the dashboard renders the canvas with no regression).
- [ ] W-3 production entrypoint: the workspace default is the canvas (the orbit entrypoint ActivityWindow is gone or has no orbit role).
- [ ] lcg.ts is KEPT; the partial files (planet-scan/star-shoot/star-state) were traced before deletion.
- [ ] W-6: the dashboard renders the canvas with no regression (dashboard-smoker).
- [ ] The galaxy-view-toggle decision is surfaced (CANVAS §14 — a permanent calm-mode toggle, or full deletion), not silently made.
- [ ] cargo check --workspace GREEN + dashboard svelte-check (with three gone); auditor PASS.

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE; the off-agent artifact is the GREEN build + the zero-grep (purely mechanical, ungameable).
- W-2: the canvas is the default (the orbit is removed, not gated off).
- W-3: workspace.svelte.ts:82 (canvas default); the orbit entrypoint is gone.
- W-4: the dashboard renders the canvas; cross_lang_diff N/A (a deletion).
- W-5: a runnable dashboard build + open -> a GREEN build + a clean grep + a rendering canvas.
- W-6: the canvas renders with no regression via dashboard-smoker.
- W-7: the three deps / scene layer / ActivityWindow / lcg.ts / the default panel grep-confirmed 2026-06-03 (C12).
- W-8: N/A (a deletion — no produced symbol to consume; the produced artifact is the absence of three.js).
- W-9: NO SSB change.
- W-10: in-process (a build + a grep).

## Tests to add
- a build-level gate (not a unit test): cargo check --workspace + dashboard svelte-check GREEN with three/troika/@types/three
  removed; a zero-grep CI check for `from .*scene/three` + `troika-three-text` across crates/dashboard-v2/src; a
  dashboard-smoker check that the dashboard opens to the canvas with no console error and no orbit import (the off-agent, ungameable artifact).

## Files touched
- DELETE: crates/dashboard-v2/src/lib/scene/three-scene.ts + three-orbit.ts + three-orbit-modes.ts + StarCanvas.svelte + sky-dome.ts + star-points.ts (+ the traced planet-scan/star-shoot/star-state)
- DELETE: crates/dashboard-v2/src/lib/orbit/node-mandala.ts + OrbitCanvas.svelte + ui/Starfield.svelte
- MODIFY: crates/dashboard-v2/src/lib/windows/ActivityWindow.svelte (remove the orbit render, or delete if no non-orbit role)
- MODIFY: crates/dashboard-v2/package.json (~3 LOC: drop three + troika-three-text + @types/three)
- MODIFY: crates/dashboard-v2/src/lib/state/workspace.svelte.ts (~5 LOC: flip the default off 'activity')
- KEEP: crates/dashboard-v2/src/lib/orbit/lcg.ts (the canvas seed source)
- CREATE: workflow/reasoning/H-16-decisions.md (~80 LOC)

## Citations / Grounding
- CANVAS-WORKBENCH-DESIGN.md §2 (what keeps/converts/dies with three.js; lcg.ts becomes the canvas seed source) + §10
  (the 2D starfield that survives) + §12 step 6 (retire only after a full release of Canvas-as-default with no regressions, the orbit the fallback until then) + §13 C6 + §14 (the orbit-removal cost + the galaxy-view-toggle open question).
- ELAI ground truth (C12): three ^0.184.0 + troika-three-text package.json:23; the scene/ layer + node-mandala + OrbitCanvas + ActivityWindow:119 (the default panel entrypoint); lcg.ts NOT test-only (KEEP); the isolation gate PASSES; the partial planet-scan/star-shoot/star-state.
- CLAUDE.md HARD RULES.

## Open questions / risks
- The PRECONDITION is the load-bearing risk: deleting the orbit before Canvas-as-default has proven itself over a full
  release re-introduces the very loss the incremental migration avoids. Do NOT run H-16 early.
- The partial files (planet-scan/star-shoot/star-state) have an unpinned prod caller — a missed live caller is a
  regression; trace them before deletion (Task 2 is a hard gate).
- The galaxy-view-toggle (CANVAS §14): decide whether to keep a permanent calm-mode "galaxy" toggle or fully delete —
  surface the decision rather than silently removing the option (some users may want the calm orbit aesthetic as a non-work view).
- Orbit-removal is the only irreversible op in the plan (deleting code + dropping deps) — the H-0 tripwire posture
  applies: confirm the precondition + the trace before the deletion lands.

## Next
H-16 is the LAST session. With it, the orbit is retired, the canvas is the default and the model's journaled +
reproducible + taint-tagged reply channel, and the three.js weight is gone. The H-plan is complete: Track 1 (the measured
efficiency number) + Track 2 (the canvas product surface) both shipped under the same wiring-enforced done-gate.

End of H-16.
