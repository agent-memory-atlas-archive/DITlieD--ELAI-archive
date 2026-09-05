# B-13: Activity Feed Restoration
Objective: restore the spec-compliant Activity feed UX — auto-scroll, Jump-
to-Latest pill, inline [preview] entry with teal link, schema-correct
session handoff entries. Currently the feed is buried under 5 panels and
missing UX affordances.
Estimated effort: 1 session (re-scoped — see V2 REBASELINE).
Pre-requisites: B-3 (Settings schema fix via ts-rs), B-12 (Workspace
primary).
Parallel-safe with: B-14, B-15..B-20.

## V2 REBASELINE (2026-05-07) — Activity-feed concept REPLACED by orbit

dashboard-v2 retired the V1 text-style ActivityFeed. The V2 ActivityWindow
(`.elai_cc/crates/dashboard-v2/src/lib/windows/ActivityWindow.svelte`)
**IS the deterministic-orbit visualizer** per DOMAIN-CONTEXT.md §2 and
DESIGN.md §8. It has 3 FSM-driven modes:

  - Mode 1 PLAN: decomposition tree (Concept -> Phase -> Step -> BabyStep)
    rendered as a constellation. Active concept always centred. Backdrop
    dots for every step + baby give the constellation feel.
  - Mode 2 MAP: existing-project codebase map. PageRank-ranked symbols
    in a score-banded ring. DKG edges as faint connecting lines.
  - Mode 3 RESEARCH: research tree (FORMULATE -> SEARCH -> EXTRACT ->
    REFLECT -> CONVERGE) with phi/psi convergence twin micro-bars.

Mode resolution: explicit `mode` prop wins; otherwise derived from
`fixture.fsm` (RESEARCHING -> research, IDLE+pagerank -> map, anything
else -> plan).

Original B-13 task list (auto-scroll, Jump-to-Latest pill, inline
[preview] entries, ActivityEntry type colors, etc.) DOES NOT APPLY to V2.
Re-scoped task list:

  1. Orbit captions wiring: `lib/orbit/captions.ts` produces deterministic
     captions from `(fsm, grc.step_type, bubble_action?)`. Add the missing
     caption rules per DESIGN.md §8 caption table. No LLM in caption
     generation.
  2. Bubble-up event strip: thin strip beneath the orbit shows
     BabyStep×3 -> Step×2 -> Phase×1 -> Concept×1 -> Escalate transitions.
     Component: NEW `lib/orbit/BubbleUpStrip.svelte`. Consumes the
     `bubble_action` field from SSB row payload.
  3. Signal layers per DESIGN.md §C (already partially in tree —
     OrbitGrcDots.svelte, OrbitBudgetRing.svelte, OrbitEvidenceGates.svelte
     exist as scaffolds; need real data once B-5/B-6/B-10 producers land).
  4. Idle effect: `OrbitShootingStar` fires every ~14s deterministic per
     fire-counter. Verify the SCAFFOLD exists and wires to the entropy-
     quarantined LCG (per DOMAIN-CONTEXT §8.4).
  5. Settings schema fix moved to B-3 (ts-rs / TypeScript types in
     `src/lib/api/types.ts`).
  6. Hover-reveal chrome: ActivityWindow header ("CONCEPT …") + dev
     mode-toggle pill ("ORBIT …") follow DESIGN.md §7 hover-reveal rule
     (`opacity: 0` until mouse over the panel). Verify and backfill if
     missing.

Path map (V1 -> V2):
  V1: crates/dashboard/src-frontend/src/ActivityFeed.svelte
   -> retired (replaced by orbit)
  V1: crates/dashboard/src-frontend/src/ActivityEntry.svelte
   -> retired (orbit captions handle entry rendering)
  V1: crates/dashboard/src-frontend/src/activity/{JumpToLatest,
       FilterStrip}.svelte
   -> retired (orbit has no scroll)
  V1: crates/dashboard/src-frontend/src/Settings.svelte
   -> crates/dashboard-v2/src/lib/windows/SettingsWindow.svelte
  V1: crates/dashboard/src-frontend/src/windows/ActivityWindow.svelte
       (layout)
   -> crates/dashboard-v2/src/lib/windows/ActivityWindow.svelte (now the
       orbit host)

If the user requests a CLASSIC TEXT FEED ALONGSIDE the orbit, that becomes
a new B-N — not in B-13 scope.

## Context

Per audit-dashboard/_AGGREGATE.md visual drift:
- Activity feed buried under autonomy strip + feasibility panel +
  violations strip + proposals panel + intervention cards before reaching it
- Activity feed missing: auto-scroll, Jump-to-Latest pill, inline
  [preview] entry with teal link
- DiffViewer per-hunk hover controls and multi-file vertical nav thin

Spec ACTIVITY window per dashboard-design.txt: real-time engineering feed
with [reasoning], [edit], [validate], [test], [cloud] entry types,
collapsed-by-default reasoning, expand to full chain, file path click
opens REVIEW window.

## Reads

- F:\.ELAI_workflow\docs\specs\guide\dashboard-design.txt WINDOW: ACTIVITY
- audit-dashboard/_AGGREGATE.md visual drift section
- crates/dashboard/src-frontend/src/ActivityFeed.svelte
- crates/dashboard/src-frontend/src/ActivityEntry.svelte (already supports
  many entry types; just buried)

## Tasks

1. **Top-level placement.** Move ActivityFeed to be the primary content of
   the ACTIVITY window. Autonomy strip + feasibility + violations +
   proposals + intervention cards become RIGHT-RAIL or TOP-STRIP of width
   ≤30% — never below the activity feed.

2. **Auto-scroll + Jump-to-Latest pill.** When user is at bottom, follow
   latest. When user scrolls up, freeze position; show floating pill
   "Jump to latest (N new)" at bottom-right. Click pill → smooth scroll.

3. **Inline [preview] entry.** When a tool produces a previewable artifact
   (file write, image, diff), add inline entry `[preview] <path>
   teal link`. Click opens REVIEW window with the preview.

4. **Reasoning collapse.** Reasoning entries default collapsed to 2-3
   lines summary. Click to expand full GRC chain (per spec). Use the real
   data from B-5 producer wiring.

5. **Live indicator.** While step is executing, last entry has subtle
   pulse — teal-soft border opacity 15%-40% over 2s cycle. Per spec.

6. **Step header treatment.** "step 3.2: implement token validation" with
   thin teal line separator; current step text-primary; completed steps
   text-secondary; completed steps collapse to one line (step + pass/fail).

7. **Entry type colors.** Per spec:
   - [reasoning]: teal-ghost background
   - [edit]: file path + collapsed diff +N -M lines summary
   - [validate]: compact one-liner per tier; teal checkmark / fail x
   - [test]: pass/fail counts; failures expand
   - [cloud]: purple-ghost background; cost shown
   Match B-11 design tokens exactly.

8. **Settings schema fix.** Use ts-rs-generated bindings (B-3 dep) for
   SessionHandoffEntry to fix the {id, branch, ts} mismatch. Render
   actual {seq, ts_ms, winner, abandoned, trunk_snap_id, from_snap_count}.

9. **Filter strip.** Optional small filter bar above feed: toggles for
   each entry type (show/hide reasoning / edit / validate / test / cloud).
   Persisted per session.

10. **Search-within-feed.** Ctrl+F focuses a search input that highlights
    matching entries (file paths, error messages).

## WIRING_CONTRACT

produces_symbols:
  - ActivityFeed primary position in ACTIVITY window
  - Inline [preview] entry renderer
  - Jump-to-Latest pill component
  - Filter strip
  - Search-within-feed
consumes_symbols:
  - SSB tail (already wired via get_recent_actions from B-6 substrate)
  - REVIEW window opener for preview link
reachable_from:
  - Dashboard task running
stub_ban:
  - ActivityFeed must NOT be below intervention cards / proposals / etc.
  - Jump-to-Latest pill must appear when user scrolled up + new entries arriving
frontend_pair: B-13 itself
observable_effect:
  trigger: submit task that writes a file
  expect_artifact: ActivityFeed shows [preview] link inline within 2s;
                   click opens REVIEW

## DETERMINISTIC_CHECK

```bash
# 1. ActivityFeed primary position in ACTIVITY window
# Manual visual inspection: open dashboard, ACTIVITY window. Feed dominates.

# 2. Auto-scroll behavior
# Run task, scroll up mid-execution, verify pill appears. Click pill;
# confirm smooth scroll back.

# 3. ts-rs SessionHandoffEntry
rg "SessionHandoffEntry" .elai_cc/crates/dashboard/src-frontend/src
# Expected: imports from generated types.gen.ts
```

## ACCEPTANCE

- [ ] ActivityFeed is primary content of ACTIVITY window
- [ ] Autonomy/feasibility/violations strips are right-rail or top-strip
- [ ] Auto-scroll works
- [ ] Jump-to-Latest pill appears + functional
- [ ] Inline [preview] entries render and click-to-open REVIEW
- [ ] Reasoning entries collapsed by default; expand on click
- [ ] Live pulse indicator on last entry during execution
- [ ] Settings SessionHandoffEntry schema fixed
- [ ] Filter strip + Ctrl+F search functional

## Files touched

- MODIFY: crates/dashboard/src-frontend/src/ActivityFeed.svelte (major refactor)
- MODIFY: crates/dashboard/src-frontend/src/ActivityEntry.svelte (entry types)
- ADD: crates/dashboard/src-frontend/src/activity/JumpToLatest.svelte
- ADD: crates/dashboard/src-frontend/src/activity/FilterStrip.svelte
- MODIFY: crates/dashboard/src-frontend/src/Settings.svelte (use ts-rs types)
- MODIFY: crates/dashboard/src-frontend/src/windows/ActivityWindow.svelte (layout)
- CREATE: workflow/reasoning/B-13-decisions.md

## Next

B-14 (Live token streaming chat).
