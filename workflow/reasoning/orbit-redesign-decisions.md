orbit-redesign — decision doc
session s-1107452
date 2026-04-25
fsm PLANNING phase orbit-redesign

what
make the ELAI dashboard orbit the visual centerpiece. replace the static-decoration trial in ActivityWindow.svelte with a modular orbit system that supports three modes, four-level smooth zoom, plain-language captions, GRC reasoning dots, evidence gates, web-research tendrils, multi-concept constellations, and idle shooting-star VFX. all stub data, no live wiring, no new deps.

key decision
keep ActivityWindow.svelte as the orbit host. extract the trial's polar-rings + curved-bezier + bloom + parallax + dashed-flow + reasoning-caption + reasoning-dots aesthetic into modular components under src/lib/orbit/. introduce mode router that picks Mode 1 (decomposition tree) / Mode 2 (codebase map / DKG + PageRank stub) / Mode 3 (research tree with tendrils) based on FSM state. 4-level zoom = (concept, phase, step, baby-step) with cubic-bezier ease ~400ms, click-to-zoom + wheel + Esc. multi-concept = pannable constellations (drag at concept-zoom). idle = static orbit + occasional shooting-star (Starfield CSS drift continues separately). research = auto-zoom-out when tendrils extend.

modules (all under .elai_dashboard/src/lib/orbit/)
- types.ts            orbit-specific TS types layered on existing api/types.ts (DecompNode, EvidenceGate, ResearchTreeNode, SourceTier, AgentRole, ContextBudgetSlot, GrcBudget all already exist; do not duplicate)
- lcg.ts              seeded LCG copied from Starfield pattern (entropy quarantine: same task = same orbit positions)
- geometry.ts         polar(theta,r), curve(Bezier), angle distributors, deterministic-from-id position seeding
- captions.ts         pure (fsm,grc.step_type,intent) -> plain-language string (the 14-entry table from HANDOFF-orbit.md:340-358)
- zoom.ts             4-level zoom state machine, transform interpolation, viewport-target math
- contract.ts         type-level + pure-function preconditions; svelte-check enforces
- OrbitCanvas.svelte         main wrapper: viewBox, mode router, zoom transform, mouse handling, shooting-star overlay, deep-space wrapper for tendrils
- OrbitMode1Plan.svelte      decomposition-tree mode (extracts current trial geometry); supports multi-concept constellations
- OrbitMode2Map.svelte       DKG + PageRank stub; manual camera (per user)
- OrbitMode3Research.svelte  research tree with FORMULATE/SEARCH/EXTRACT/REFLECT/CONVERGE node states; auto-zoom-out for tendrils
- OrbitGrcDots.svelte        small dots orbiting active node; one per recent GRC call
- OrbitBudgetRing.svelte     context-budget donut around active node, 8 slots
- OrbitEvidenceGates.svelte  7 gates between phase rings; locked/unlocked state by verdict
- OrbitCaption.svelte        plain-language caption near active baby step
- OrbitTendrils.svelte       outside-orbit tendrils for web research; source-particle returns
- OrbitShootingStar.svelte   single shooting star fires every ~14s when idle (matches user reference image)
- OrbitDeepSpace.svelte      extended starfield outside main orbit, used when zoom < 1 (so tendrils can be seen)

extensions (existing files)
- src/lib/api/types.ts     add 'converge' to ReasoningPayload.step_type to match Rust GrcState; add OrbitMode + ZoomLevel + DkgNodeStub + PageRankNodeStub + TendrilEvent + ShootingStarConfig
- src/lib/api/mock-orbit.ts new fixtures: small single-concept, multi-concept constellation (Redis pool + monitoring), mid-research tree
- src/styles/tokens.css    add --grc-{observe,hypothesize,plan_action,act,verify,converge,status} + --tier-{1,2,3,4,5} + --bubble-{research_alt,redecompose,replan_phase,replan_concept,diagnose_retry,escalate}
- src/lib/windows/ActivityWindow.svelte  rewrite as thin orbit host: imports OrbitCanvas, picks mode from fsm state, renders <OrbitCanvas mode={mode} data={data} />

determinism
all node positions seeded from concept_id/phase_id/step_id/baby_step_id hash via LCG. same task = identical orbit. shooting-star arc seeded from session_id (varies per run, not per task). starfield already uses lcg(0xC0FFEE). pattern copied verbatim.

signal-to-visual map
- FSM state          central halo color (tokens.css --state-* already exists); ripple on transition
- GRC step           orbital dot color via new --grc-* tokens
- GRC budget         arc gauge top-right (already in trial)
- Active baby step   bloom + dashed-flow path (already in trial)
- Evidence gate      ring-segment between phase orbits, lock/unlock animation
- Web research       tendril extends beyond orbit (auto zoom-out); source particles return colored by --tier-*
- Multi-concept      side-by-side constellations at zoom < 1; pannable
- Anti-spiral        red pulse on stuck node; escalation badge
- Bubble-up          color-coded ripple per BubbleUpAction variant via --bubble-* tokens
- Mode swap          fade between Mode1/Mode2/Mode3 components on FSM state change

constraints
- project boundary  all writes to .elai_dashboard/ only; .elai_cc/ read-only
- no new deps       Svelte 5 + native SVG only; no D3, no force layout, no animation lib
- no faked signals  if backend producer doesn't exist yet (per summary 03), don't fabricate the visual; render the *capability* with stub data labeled stub
- entropy quarantine same task = same orbit; no Math.random except where shooting-star fire-time depends on session
- accessibility     all nodes are role=button, tabindex=0, keyboard activatable; @media (prefers-reduced-motion) disables animations
- token economy     consolidate components, no premature splitting
- no .unwrap        N/A in Svelte/TS; equivalent rule: no `as` casts to dodge null checks; use `??` and explicit narrowing

phase plan (this session)
A foundation: types extension, lcg, geometry, captions, zoom, mock-orbit, tokens.css extension, OrbitCanvas skeleton, OrbitMode1Plan extracting the trial logic, ActivityWindow rewritten as host
B zoom: 4-level state machine wired; click-to-zoom + wheel + Esc; transform interpolation; CDP screenshot at concept and baby zoom levels
C signal layer: OrbitCaption, OrbitGrcDots, OrbitBudgetRing, OrbitEvidenceGates; FSM-state central halo color; CDP screenshot showing all signals firing
D tendrils + constellations + shooting stars + deep-space: multi-concept layout, panning, OrbitTendrils, OrbitShootingStar, OrbitDeepSpace; CDP screenshot during active research
E Mode 2 + Mode 3: OrbitMode2Map (DKG nodes by PageRank-shaped score; effect-tag color; manual camera), OrbitMode3Research (research tree with phi/psi convergence indicator); CDP screenshot of each mode

testing strategy
no vitest installed; "test-first" = src/lib/orbit/contract.ts with type-level assertions and pure-function preconditions that svelte-check enforces. visual smoke test = mount OrbitCanvas in each of the 3 modes via fsm.transition() and confirm zero console errors via CDP DevTools. pixel-validate every screenshot via check_all.mjs (lastY=799, emptyRows=0).

risks (top 3)
1 zoom interpolation clamping wrong on click-to-zoom from concept to baby (could overshoot the viewBox bounds). mitigation: cubic-bezier interpolation with clamped target rect computed from node bounding box + 12% padding before transition starts.
2 multi-concept layout collision when concept count > 4 in pannable view. mitigation: deterministic packing via golden-angle distribution around an outer-orbit ring; concepts that would overlap shrink their orbit radius.
3 shooting-star perf: re-rendering the SVG line every frame on a 30s loop is fine, but pairing it with parallax transform could cause repaint storms. mitigation: shooting-star lives on a separate transform layer (translate3d + will-change: transform), not within the parallax SVG.

unknowns
- whether the user wants the existing reasoning-dots strip to remain at the bottom of the orbit panel, or be replaced entirely by the new OrbitGrcDots orbiting the active node. plan: keep both — bottom strip = "history" (recent N), orbiting dots = "current". confirm during phase C.
- whether session-id seeding for shooting-star fire-time is acceptable, or should be fully deterministic from task_id. plan: session-id seed (less repetitive), reversible if user prefers.
- whether ResearchTreeNode in api/types.ts is sufficient for Mode 3, or needs phi/psi/contradiction fields surfaced visually. plan: surface phi/psi as twin small bars on each research node; contradictions as red edge between two child nodes.

deterministic check (auditor will run)
1 cd F:/.ELAI_workflow/.elai_dashboard && npx svelte-check --tsconfig ./tsconfig.json
   expect: 0 errors, 0 warnings
2 node F:/.ELAI_workflow/.elai_dashboard/screenshots/cdp_capture.mjs (output trial-orbit-redesign-v1.png)
3 node F:/.ELAI_workflow/.elai_dashboard/check_all.mjs against the new screenshot
   expect: lastY=799, emptyRows=0
4 grep for forbidden patterns in src/lib/orbit/: TODO|placeholder|pass|Math\.random
   expect: zero matches outside contract.ts and lcg-seeded fire-time logic

acceptance
done = orbit modes 1/2/3 each render with stub data; zoom transitions feel premium (~400ms cubic ease); plain-language captions match the 14-entry table; multi-concept fixture renders as pannable constellation at concept zoom; shooting-star fires occasionally during idle; svelte-check 0/0; CDP screenshot pixel-validated.

revert path
all changes within .elai_dashboard/ which is gitignored. revert via .original backups for ActivityWindow/CommandWindow/SettingsWindow. new orbit/ directory has no backup; revert by deleting the directory.

next
write src/lib/orbit/contract.ts with type assertions, transition PLANNING -> TESTING_FIRST -> EXECUTING, build phase A.
