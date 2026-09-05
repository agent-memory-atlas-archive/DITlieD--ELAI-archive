E-33 — Dashboard UX agentic — decisions
session: s-orch-e33 (orchestrate continuation of in-band first half)
plan: F:\.ELAI_workflow\docs\thefinalglance\E-plan\E-33_dashboard_ux_agentic.md

(a) WHAT WAS SHIPPED THIS E-N

Five new dashboard-v2 components landed in two halves of this session:

  - crates/dashboard-v2/src/lib/components/ActivityRibbon.svelte (~150 LOC) — persistent thin row at the top edge of WindowHost. Shows FSM state + substate + step id + progress + dispatch mode + active model. Subscribes to the live fsm store; renders nothing when fully idle.
  - crates/dashboard-v2/src/lib/components/SkeletonList.svelte (~80 LOC) — generic skeleton-row placeholder. Renders N shimmer rows when loading=true, nothing otherwise. Honors prefers-reduced-motion natively via @media block.
  - crates/dashboard-v2/src/lib/components/ConfidenceTag.svelte (~110 LOC) — binary-confidence tag with 4 states (needs-evidence/low/ok/verified) + shape-distinct glyph + Wong-palette color + hover tooltip surfacing the raw numeric.
  - crates/dashboard-v2/src/lib/components/WindowFooterMode.svelte (~85 LOC) — per-window footer strip showing dispatch mode + autonomous-gate state + FSM state:substate. Distinct from ActivityRibbon (which is live operation ticker).
  - crates/dashboard-v2/src/lib/components/SnackBarPermission.svelte (~120 LOC) — stacked permission-prompt row with Approve/Reject/Always-allow buttons. Consumer-driven (parent supplies prompts[]); invisible when empty.

Plus the motion-store + token-bundle + wiring layer:

  - crates/dashboard-v2/src/lib/state/motion.svelte.ts (NEW) — prefers-reduced-motion observer singleton. Live-listens to the MediaQueryList so OS toggle propagates without reload. User-override API setUserOverride('auto'|'reduced'|'full') reflects on documentElement [data-motion=...] so the tokens.css attribute selector fires for CSS-only consumers.
  - crates/dashboard-v2/src/styles/tokens.css — Wong-derived 5-tone signal palette appended (--signal-blue/-orange/-vermilion/-rose/-green). Binary-confidence semantic mapping (--confidence-needs-evidence/low/ok/verified) layered on top so Fuselab 2026's 4-state model has named tokens. @media (prefers-reduced-motion: reduce) block kills all CSS animations + transitions + scroll-behavior. :root[data-motion="reduced"] attribute override applies the same overrides under user toggle even if the OS media query did not match. --motion-scale token added for callers needing a numeric scaler.
  - crates/dashboard-v2/src/lib/layout/WindowHost.svelte — mounts ActivityRibbon at the top, WindowFooterMode at the bottom; bootstraps motion store on first mount (idempotent); maps each WindowType to a human-readable scope label.
  - crates/dashboard-v2/src/lib/orbit/OrbitGrcDots.svelte — added <title> + a11y label; removed aria-hidden; gated <animateTransform> behind {#if allowMotion}.
  - crates/dashboard-v2/src/lib/orbit/OrbitEvidenceGates.svelte — shape-distinct verdict glyphs (check/cross/exclamation/dot); signal colors switched to Wong --confidence-* tokens; <title> added for hover tooltip + screen reader.
  - crates/dashboard-v2/src/lib/orbit/OrbitConcept.svelte — 6 SVG <animate> sites guarded behind allowMotion (CSS @media reduced-motion at line 463-467 cannot reach SVG declarative-animation elements, per W3C SVG spec).
  - crates/dashboard-v2/src/lib/orbit/OrbitBudgetRing.svelte — scaffolding comment only (file has zero animations today; motion-store import deliberately omitted to keep the file lean).
  - crates/dashboard-v2/src/lib/windows/PlanWindow.svelte — SkeletonList in the empty-tree branch.
  - crates/dashboard-v2/src/lib/windows/ReviewWindow.svelte — SkeletonList replaces the "loading file changes…" text in the loading branch.
  - crates/dashboard-v2/src/lib/windows/ExploreWindow.svelte — transient loading flag flipped true→false across a queueMicrotask boundary; SkeletonList renders the one-frame shimmer between query append and placeholder reply.
  - crates/dashboard-v2/src/lib/drawers/ResearchDrawer.svelte — substituted for the absent ResearchWindow.svelte (which the plan referenced but does not exist on disk). Cold-start loading flag flipped false on first paint after researchData.bootstrap(); SkeletonList renders for the cold-start window.

(b) DEFERRED ITEMS WITH TARGETS

  - 5 SsbActionType variants (ActivityRibbonInteracted, ConfidenceTagHovered, SkeletonReplaced, ReducedMotionActivated, WindowFooterModeChanged) → E-33.B. Rationale: 7-touchpoint atomic ssb.rs surgery × 5 variants in a 15,034-line orchestrator file (CHECK constraint + enum + as_sql_str + from_str + payload structs + SsbPayload variant + action_type match arm + all_test_payloads sample list). E-33 is the frontend-pair E-N per WIRING_CONTRACT frontend_pair: ALL OF crates/dashboard-v2/src/lib/components/. SSB instrumentation is the natural follow-up that lands when A/B telemetry actually drives the source-report Open Questions 1-4 calibration.
  - SettingsWindow toggles for ui.activity_ribbon.enabled + accessibility.motion → E-33.D. Also deferred because the SettingsWindow.svelte file is a pre-existing untracked dirty edit from another session per s-26556 handoff line 69; staging it here would bundle in-progress work from elsewhere.
  - Contract test crates/contracts/tests/activity_ribbon_ssb_contract.rs → E-33.B. Rides with the SSB variant additions.
  - Vitest component test files for the 5 new components → E-33.B. Frontend test infra rebases on the SSB additions there.
  - Trace-cycle graph view → E-33.B (per plan task 10).
  - Devin-style auto-built repo wiki → E-33.C (per plan task 11).
  - @ for context pick + / for command pick generalization across all windows → E-33.D (per plan task 12).
  - Live-preview iframe polish → E-33.E.
  - Checkpoint browser → E-33.F.
  - Per-interaction token + cost counter → E-33.G.

(c) SKIP-SSB-INSTRUMENTATION RATIONALE

ssb.rs lives in crates/orchestrator (Rust backend) and is 15,034 LOC. Adding one SsbActionType variant requires synchronized edits at: the enum body, the SQL CHECK constraint string literal, fn as_sql_str() match, fn ssb_action_type_from_str() match, the SsbPayload variant, the action_type() match arm, the test_payload reconstructor, and the all_test_payloads sample list. Per ELAI's atomic 9-tuple per-CLAUDE.md SCHEMA hard rule, none of these can ship in isolation — the SCHEMA_VERSION bumps from 97 to 102 and all five variants must land in one PR. That is a backend-heavy task with cross-language drift surface (Rust enum vs ts-rs binding if/when tauri-specta lands vs orchestrator dispatch). E-33 is the frontend-pair E-N per the plan's own WIRING_CONTRACT. Telemetry for source-report Open Questions 1-4 ("does ribbon reduce confusion?", "does binary confidence reduce decision latency?", "does graph view detect spirals earlier?", "does reduced-motion retain enough info?") is needed only when A/B is actually measured — not as a precondition for the UI components themselves to ship. The components remain instrument-ready: every interaction site is exposed as a Svelte event/handler so the SSB emit can be threaded through in E-33.B without touching component internals.

(d) ACTIVITYRIBBON MOUNTS AT TOP — RATIONALE

Plan task 1 lists "top OR bottom edge — A/B per source report Open Question 1". Decision this session: TOP. Three grounding reasons:
  1. Closer to FSM badge eye-line. The ribbon shows FSM substate + AgentRole + last tool name + token counter — operational signals the user scans on a fresh window. Z-pattern reading (Nielsen Norman Group eye-tracking studies on dashboard layouts) places the first fixation at the top-left.
  2. StatusBar.svelte already occupies the app's bottom edge (existing layout component). A bottom ribbon would visually collide with the status bar chrome and double up on persistent surfaces.
  3. WindowFooterMode mounts at the bottom of WindowHost in this E-N showing static-state mode (dispatch mode + gate state + FSM:substate). Having both "live operation" (ActivityRibbon) and "static mode" (WindowFooterMode) at the bottom inverts the natural reading order: live signals should be glance-detected first, mode confirmation second.
Defer A/B-driven re-placement to E-33.B once ActivityRibbonInteracted SSB telemetry exists to actually measure user behavior.

(e) CONFIDENCETAG NOT INLINED INTO ORBITGRCDOTS — RATIONALE

OrbitGrcDots renders 3 SVG <circle> primitives (radius 1.4-1.8 px) around an active baby-step node. Each dot is a GRC step-type indicator (observe / hypothesize / plan_action / act / verify / converge / status) with NO numeric value attached — dot color encodes the step, trailing-dot opacity encodes recent history. There is no confidence percentage to replace with a binary tag. The hue→semantic mapping is already redundant with the SVG <title> "GRC step: {active_step}" added this session, which surfaces the step name on hover + announces to screen readers (the aria-hidden="true" was removed and replaced with role="img" + aria-label). ConfidenceTag's 4 states map to a numeric confidence value, which is a property of EvidenceGates (OrbitEvidenceGates), not GRC dots. ConfidenceTag stays scoped to OrbitEvidenceGates where the verdict glyph + tone semantics actually correspond. OrbitGrcDots gets the <title> + reduced-motion guard already; further redundant encoding (a 5th label glyph next to a 1.4 px dot) would clutter without information gain.

(f) MOTION GUARD SCOPE — RATIONALE

OrbitConcept SVG <animate> elements (6 sites) are NOT reachable by the CSS @media (prefers-reduced-motion: reduce) block at OrbitConcept.svelte:463-467, per W3C SVG spec — declarative animation elements are not CSS-driven and do not honor the media query. The JS guard (motion.allow + {#if allowMotion}) is required for full reduced-motion compliance. OrbitBudgetRing has no animation today, so the guard is scaffolding-only (a comment pointing future implementers at the OrbitGrcDots pattern). Pattern matches OrbitGrcDots.svelte:13 + 41 + 52-55.

(g) RESEARCH-WINDOW SUBSTITUTION

Plan task 2 lists ResearchWindow.svelte as a SkeletonList target. The file does not exist on disk (verified via Glob over crates/dashboard-v2/src/lib/windows/*.svelte; only 12 files; no ResearchWindow). The canonical FORMULATE/SEARCH/EXTRACT/REFLECT/CONVERGE stage list lives at crates/dashboard-v2/src/lib/drawers/ResearchDrawer.svelte:60 (STAGES constant) and renders at lines 120-153 of that file. ResearchDrawer is a drawer, not a window — but it is the only home for the 5-stage list. SkeletonList wiring lands in ResearchDrawer with a cold-start loading flag flipped false after first paint post-bootstrap(). This substitution is documented in the auditor's expected acceptance check.

(h) ORBITBUDGETRING — NO-OP DECISION

OrbitBudgetRing.svelte is 66 LOC, renders 8 static SVG arc segments, has zero <animate> / <animateTransform> / CSS animation declarations (verified via grep). The plan listed it as a reduced-motion target precautionarily. Decision: skip the motion-store import entirely; leave a comment block citing the OrbitGrcDots pattern for future implementers. Rationale: adding an unused import is dead weight; the comment is the minimum compliance signal so a future E-N maintainer knows the pattern. The DETERMINISTIC_CHECK in the plan does not grep OrbitBudgetRing for motion, so neither path breaks the gate.

(i) WHY PRE-EXISTING DIRTY FILES ARE NOT BUNDLED

s-26556 SESSION_HANDOFF lines 64-80 lists 20+ files modified before that session started and never staged — among them .elai_cc/crates/dashboard-v2/src/lib/api/mock-data.ts + .elai_cc/crates/dashboard-v2/src/lib/windows/SettingsWindow.svelte. Both fall in E-33's stated MODIFY list. Decision: leave those pre-existing changes alone. The plan's SettingsWindow toggle additions are deferred to E-33.D (see (b)). The mock-data.ts pre-existing edit is unrelated to this E-N; staging it would bundle untracked work from elsewhere into the E-33 commit and violate orchestrate protocol step 11 ("Stage only files relevant to this orchestrate run").

(j) PROTOCOL DEVIATION + CORRECTION

First half of this E-N (5 components + tokens.css Wong palette + reduced-motion + WindowHost mount + OrbitGrcDots/OrbitEvidenceGates wiring + motion store) was authored by the main LLM IN-BAND, without dispatching context-compiler / integration-explorer-elai first. This violates the orchestrate protocol step 8.3 PARALLEL BATCH requirement. User caught it. Mid-session, .claude/rules/orchestrate-protocol.md was added with H-1..H-10 hard rules + CLAUDE.md + SKILL.md + conditional-routing.md updated to make agent dispatch BLOCKING for future runs. Second half (4 window SkeletonList wirings + OrbitConcept motion guard + OrbitBudgetRing comment + this decisions doc) was authored AFTER properly dispatching context-compiler + integration-explorer-elai in parallel and consuming their returns. The auditor + memory-writer dispatch follows.

(k) S-25120 CLOSURE PASS — ADDENDA

Completion-drive session 1 of 4 (s-25120) closed the 4 NOT_WIRED + 8 MISSING gaps from the prior PARTIAL report. Per the binding from the closure prompt, the 5 SsbActionType variants ship SCHEMA-DEFERRED instead of forcing a SCHEMA bump mid-batch. The other gaps are wired in this pass.

  Closure deliverables (s-25120):
    - state/grc.svelte.ts NEW — polls get_recent_actions every 5s, decodes the most recent reasoning_trace via get_action_detail, exposes a numeric 0..1 confidence value matching ConfidenceTag's classify thresholds (Verified=0.95, High confidence=0.70, Uncertain=0.30, unknown=0.10).
    - state/permission_prompts.svelte.ts NEW — polls get_recent_actions for intervention_request rows, surfaces them as PermissionPrompt entries via SnackBarPermission. Per-id dedup set prevents the same request reappearing across polls. Approve/reject/alwaysAllow handlers route through stub.logStub (no backend mutation yet — intentional, sidecar-offline safe).
    - state/ux_prefs.svelte.ts NEW — persistent activity_ribbon_enabled + motion preference (auto/reduced/full) backed by localStorage. Bridges into motion.setUserOverride so the [data-motion] attribute is set on documentElement; ActivityRibbon collapses to zero height when disabled.
    - OrbitGrcDots.svelte — added optional confidence?: number prop. Renders ConfidenceTag via foreignObject overlay near the lead dot when supplied. OrbitConcept now passes grc.confidence (defaults gracefully to 0.10/needs-evidence when no reasoning_trace has landed).
    - OrbitEvidenceGates.svelte — verdictToValue maps pass/warn/block/pending onto the same 0..1 bands as ConfidenceTag. On hover, ConfidenceTag overlay renders via foreignObject alongside the existing gate-label text. ConfidenceTag now has 3 consumer call sites (OrbitGrcDots + OrbitEvidenceGates + SettingsWindow uxPrefs path was not added — design choice; the binary tag visualization lives in signal layers, not settings rows).
    - WindowHost.svelte — mounts SnackBarPermission between body and footer. Bootstraps motion + uxPrefs + grc + permissionPrompts on mount; tears down on unmount. uxPrefs bootstraps AFTER motion so the persisted override applies on first paint.
    - SettingsWindow.svelte — Appearance section gains two rows: Activity ribbon toggle pill (mirrors existing toggle-pill semantics) + Motion segmented control (auto/reduced/full). Both bound via $effect with _ribbonInit / _motionInit init-guards to skip the first reactive tick. Wiring routes through uxPrefs.set* helpers which persist + log.
    - crates/contracts/tests/activity_ribbon_ssb_contract.rs NEW — asserts SCHEMA_VERSION_PUBLIC stays at 103; pins bijection round-trip for intervention_request + reasoning_trace (the two action types the new stores consume); locks the 5 SCHEMA-DEFERRED labels so a future planner cannot drift the canonical names; 5 `#[ignore]`d round-trip tests ready to un-`#[ignore]` on the next QUINTUPLE bump batch. Build invocation: cargo check -p elai_cc_contracts --test activity_ribbon_ssb_contract.

  SCHEMA-DEFERRED rationale (binding from closure prompt):
    - SCHEMA_VERSION 103 is LOCKED throughout completion-drive session 1 batch.
    - 5 new variants (ActivityRibbonInteracted / ConfidenceTagHovered / SkeletonReplaced / ReducedMotionActivated / WindowFooterModeChanged) require a QUINTUPLE/SEXTUPLE lockstep update (enum + CHECK constraint + as_sql_str + from_str + payload structs + SsbPayload variant + action_type match + all_test_payloads + SCHEMA_VERSION bump).
    - Per E-17 precedent (ContextualChunkEmitted + ChunkRegenSkipped), the producer-side code ships now, the enum addition rides the next bump batch.
    - The contract test catches drift in either direction: a stray SCHEMA bump beyond 103 fires the freeze guard; a silent add of one of the 5 labels without the rest fires the SCHEMA-DEFERRED stub_ban; the catalogue lock fires if the count drifts from 5.

  ResearchWindow -> ResearchDrawer retarget rationale (verbatim from (g) above):
    - ResearchWindow.svelte does not exist on disk. SkeletonList for the FORMULATE/SEARCH/EXTRACT/REFLECT/CONVERGE stage list was wired into ResearchDrawer.svelte (the canonical surface) in the prior session. Plan rebaseline marks this as SHIPPED-AS-RESEARCH-DRAWER.

  SettingsWindow toggle wiring catalog:
    - ui.activity_ribbon.enabled — toggle-pill, default true, persists to elai_ui_activity_ribbon_enabled.
    - accessibility.motion — segmented control {auto, reduced, full}, default auto, persists to elai_accessibility_motion, bridges to motion.setUserOverride on every change.

  Path drift fixed:
    - Reads block referenced workflow/improvement-sweep-2026-05-19/E-plan/E-2_local_llm_speed_tricks.md; canonical path verified via Glob is docs/thefinalglance/E-plan/E-2_local_llm_speed_tricks.md. Fixed in plan file.

  Verification this pass:
    - cargo check -p elai_cc_contracts --test activity_ribbon_ssb_contract PASS (0.87s)
    - cargo check --workspace PASS (no errors, 8 pre-existing warnings)
    - pnpm svelte-check PASS (4377 files, 0 errors, 17 warnings all in pre-existing files)
    - cargo test NOT run (TEST-VERIFICATION-PENDING per user authorization throughout completion-drive session 1)

END E-33-decisions.md
