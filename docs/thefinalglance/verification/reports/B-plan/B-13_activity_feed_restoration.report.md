# Verification Report: B-13_activity_feed_restoration
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-13_activity_feed_restoration.md
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
B-13 was fully re-scoped under the V2 REBASELINE section: the V1 ActivityFeed/ActivityEntry/JumpToLatest/FilterStrip tasks are explicitly retired and replaced by six orbit-visualizer sub-tasks. Five of the six re-scoped tasks are DONE (captions wiring, BubbleUpStrip, signal-layer scaffolds, OrbitShootingStarOverlay, hover-reveal chrome). One task is PARTIAL: the BubbleUpStrip consumes `bubble_action` field from SSB rows via a real Tauri command (`get_bubble_up_events`) that queries `bubble_up_typed` rows, but the SSB emit side (B-7 producer) is explicitly deferred per code comments. The original V1 task list (auto-scroll, JumpToLatest pill, FilterStrip, inline [preview] entry, Settings ts-rs fix) is correctly scoped-out by the rebaseline and is absent from dashboard-v2 — consistent with the plan's explicit retirement declaration.

## Claims verified DONE

- ActivityWindow.svelte exists as the orbit host (not a text feed) — evidence: F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\windows\ActivityWindow.svelte:1
- ActivityWindow imported and routed in WindowHost for 'activity' slot — evidence: F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\layout\WindowHost.svelte:5,27-28
- lib/orbit/captions.ts exists and exports CAPTION_TABLE + captionFor() — evidence: F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\orbit\captions.ts:26,93
- CAPTION_TABLE covers all claimed FSM states (RESEARCHING, PLANNING, FEASIBILITY_PROBE, TESTING_FIRST, EXECUTING, AUDITING, MEMORIZING, COMPLETE, CONFIGURING, IDLE) plus bubble_action overrides — evidence: captions.ts:28-88
- No LLM in caption generation — evidence: captions.ts:3 ("No LLM. No randomness."), all templates are pure string interpolations
- BubbleUpStrip.svelte NEW component exists — evidence: F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\orbit\BubbleUpStrip.svelte:1
- BubbleUpStrip.svelte imported and rendered inside OrbitMode1Plan.svelte — evidence: F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\orbit\OrbitMode1Plan.svelte:8,44
- BubbleUpStrip consumes bubble_action/BubbleUpEvent field via get_bubble_up_events from stub.ts — evidence: BubbleUpStrip.svelte:6,39
- get_bubble_up_events Tauri command exists and queries SSB bubble_up_typed rows — evidence: F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri\src\commands.rs:2034-2043
- OrbitGrcDots.svelte scaffold exists — evidence: F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\orbit\OrbitGrcDots.svelte (in Glob output)
- OrbitBudgetRing.svelte scaffold exists — evidence: F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\orbit\OrbitBudgetRing.svelte:1
- OrbitEvidenceGates.svelte scaffold exists — evidence: F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\orbit\OrbitEvidenceGates.svelte:1
- OrbitGrcDots + OrbitBudgetRing imported into OrbitConcept.svelte (real data path once B-5/B-6 land) — evidence: elai.map:55949,55962
- OrbitShootingStarOverlay.svelte exists, fires every 14s deterministic — evidence: OrbitShootingStarOverlay.svelte:19 (fire_period_seconds = 14), line 3 comment ("Fires every ~14s")
- OrbitShootingStarOverlay uses entropy-quarantined LCG (lcg.ts) — evidence: OrbitShootingStarOverlay.svelte:9,40; lcg.ts:2 ("entropy-quarantined determinism")
- OrbitShootingStarOverlay wired into OrbitMode1Plan.svelte — evidence: OrbitMode1Plan.svelte:9,47
- Hover-reveal chrome for dev mode-toggle pill (opacity:0 until hover) — evidence: ActivityWindow.svelte:141-147 (.mode-toggle { opacity: 0; } + .activity-host:hover .mode-toggle { opacity: 1; })
- V1 ActivityFeed.svelte retired to .bkp_old (not present in dashboard-v2 or dashboard/src-frontend) — evidence: Glob found only F:\.ELAI_workflow\.elai_cc\crates\.bkp_old\dashboard\src-frontend\src\ActivityFeed.svelte
- V1 ActivityEntry.svelte retired to .bkp_old — evidence: found only at .bkp_old\...\ActivityEntry.svelte
- V1 JumpToLatest.svelte and FilterStrip.svelte never created in dashboard-v2 (consistent with retirement) — evidence: Glob across .elai_cc found zero matches
- Settings schema fix moved to B-3 (declared in plan, not a B-13 deliverable) — UNVERIFIABLE (B-3 scope check not in scope of this audit)

## Claims PARTIAL

- BubbleUpStrip consumes bubble_action field from SSB row payload
  - exists: BubbleUpStrip.svelte:6,39 calls get_bubble_up_events() which queries SSB bubble_up_typed rows (commands.rs:2034)
  - gap: SSB emit side not yet wired — BubbleUpStrip.svelte:4,66 explicitly notes "LIVE: empty until B-7 wires bubble_up_event SSB emit"; strip renders "no bubble-up events" in production until B-7 lands

- Signal layers (OrbitGrcDots, OrbitBudgetRing, OrbitEvidenceGates) need real data once B-5/B-6/B-10 producers land
  - exists: all three scaffold components confirmed; OrbitGrcDots + OrbitBudgetRing imported in OrbitConcept.svelte
  - gap: OrbitEvidenceGates import in OrbitConcept.svelte not confirmed (elai.map shows it imported by OrbitConcept only for OrbitGrcDots + OrbitBudgetRing); B-5/B-6/B-10 data producers deferred per plan text

- ActivityWindow header hover-reveal ("CONCEPT ...") — plan says verify and backfill if missing
  - exists: header was explicitly removed per ActivityWindow.svelte:94-98 comment ("top-left header removed: concept name now lives only in OrbitCaption below the canvas")
  - gap: the header specified in DESIGN.md §7 hover-reveal rule is absent from the component; OrbitCaption below canvas is the replacement but does not follow opacity:0 hover pattern

## Claims MISSING

- No claims in the V2 re-scoped task list are structurally missing (all six tasks have implementation or explicit deferral documented in code)

## Claims UNVERIFIABLE

- DESIGN.md §8 caption table completeness: plan claims "Add the missing caption rules per DESIGN.md §8 caption table." DESIGN.md §8 has no structured caption table (grep found only one caption reference at line 59 about font sizes). The captions.ts source-of-truth comment cites HANDOFF-orbit.md:340-358, not DESIGN.md §8 directly. Cannot confirm the table in captions.ts is complete relative to an unlocatable DESIGN.md §8 spec entry.
- Settings schema fix (B-3 dep): plan states "moved to B-3". Whether B-3 delivered ts-rs SessionHandoffEntry bindings is outside this audit's scope.
- B-5/B-6/B-10 producers: whether these have landed and can feed OrbitGrcDots/OrbitBudgetRing/OrbitEvidenceGates with real data is outside this audit scope.

## What must be done (action list)

- [ ] B-7 (SSB emit side): wire bubble_up_event SSB rows so BubbleUpStrip shows real events instead of "no bubble-up events" placeholder — target: orchestrator SSB emit + BubbleUpStrip.svelte:66
- [ ] Verify OrbitEvidenceGates is imported into OrbitConcept.svelte or its correct parent once B-10 evidence gates are produced — target: OrbitConcept.svelte
- [ ] Decide whether ActivityWindow concept-name header (opacity:0 hover-reveal per DESIGN.md §7) should be restored or if OrbitCaption is the accepted replacement — target: ActivityWindow.svelte:94-98
- [ ] Confirm captions.ts coverage against DESIGN.md §8 once the spec section is located (current source of truth is HANDOFF-orbit.md:340-358, which references 14 entries; captions.ts has 32 rows covering all FSM states + bubble overrides — likely complete)

## Notes

The V2 REBASELINE section supersedes the original task list entirely. All verification above is against the rebaseline tasks, not the original 10-item task list (which is correctly retired). The original plan Tasks section (auto-scroll, Jump-to-Latest, inline [preview], etc.) maps to V1 dashboard which lives in .bkp_old — those items are not implemented in dashboard-v2 by design.
