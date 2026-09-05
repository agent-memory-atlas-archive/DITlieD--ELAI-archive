DASHBOARD-V2 WIRING GAP ANALYSIS
task_id: gap-analysis-v1
purpose: cross-reference what the feature map says SHOULD have a dashboard surface
  against what the extracted CONTRACT-SHEET.md proves is actually wired in
  dashboard-v2. Companion to CONTRACT-SHEET.md (panel->command->handler) and
  HANDLER-SHAPES.md (the real json! / DTO field shapes).
sources: docs/thefinalglance/ELAI-FEATURE-MAP.md (status flags) ; the contract sheet
  (live grep of dashboard-v2 2026-06-13) ; H/I/J/K plan summary indexes ; spot-checks
  against real code this session.

GROUND-TRUTH NOTE (verified 2026-06-13)
Where the feature map and the contract sheet disagree about a dashboard panel, the
CONTRACT SHEET wins: it was extracted from a live grep of dashboard-v2; the feature
map carries stale dashboard notes. Two confirmed stale entries this session:
- feature-map:1052 says ContextStatePanel + TrustTelemetryPanel "not mounted" — WRONG.
  Both are imported + mounted in MonitorWindow.svelte (lines 66-67, 471, 482). They
  render EMPTY unless ELAI_DEVELOPER_DISCOURSE_ENABLE=1 on the orchestrator side.
- feature-map:1013 says LoraShipGatePanel "absent" — WRONG. It exists
  (components/LoraShipGatePanel.svelte) + is mounted in MonitorWindow.svelte:492.
  Data is bench-pending (empty until a run).

HEADLINE
The dashboard is NOT mostly un-wired. The contract sheet proved ~190 commands reach
real panels. The real gaps are four narrow buckets (A-D below). For a redesign that
keeps the wiring and replaces presentation, the work is: mount/ungate a few built
panels, build a few missing surfaces, add honest empty/dark/mock states, and absorb
the planned J + H-Track-2 surfaces.


================================================================================
WHICH PLANS ADD DASHBOARD WORK (corrected 2026-06-13)
================================================================================
I-plan   : rows/badges/chips on EXISTING drawers only. NO new top-level panels.
           (59 frontend_pair entries, all decorate existing surfaces or N/A.)
H-plan   : Track 2 (H-10..H-16) IS the canvas/dashboard rebuild — canvas shell, file
           explorer, plan/research modes, model-drives-canvas, design mode, retire-orbit.
           Partially shipped in code; DEMOTED in priority behind K-3/A-5
           (user-confirmed 2026-06-12, K-0_INDEX.md:269), NOT killed; orbit retirement
           still rides canvas-default.
J-plan   : the source of genuinely NEW dashboard surfaces —
           - UsageWindow.svelte (NEW window: per-model/task/session token consumption,
             cache effectiveness, tool/file-op counts) + LiveUsageGauge mid-stream meter
             [PLANNED, J-28/J-29].
           - canvas "system-composition" mode (J-31): new `system` mode on the shipped
             canvas (nodes=buildable units, edges=verified wiring claims) + system_canvas.rs.
           - front-door (J-32): chat-as-ELAI ACK/notice/RESULT cards + "Open in canvas ->" deep-link.
           - HardwareModelCatalogPanel (J-9, deferred) + many rows/badges on existing drawers.
K-plan   : almost NO new dashboard. Wedge/GTM/evidence focus; "the Tauri dashboard is the
           DEEP view, never the ONLY view" (K-0_INDEX.md:117); pushes evidence to CLI /
           portable .rra; demotes the canvas track.

Net: new dashboard building = H-Track-2 (canvas, demoted) + J (UsageWindow + canvas
system mode + front-door). I and K decorate existing surfaces or deprioritize.


================================================================================
BUCKET A — PANEL BUILT + WIRED, BUT NOT MOUNTED OR FLAG-DARK
(renders nowhere, or only behind an env flag)
================================================================================
ContextStatePanel    mounted in MonitorWindow but EMPTY unless ELAI_DEVELOPER_DISCOURSE_ENABLE=1.
TrustTelemetryPanel  same gate. (feature-map "not mounted" is stale — they ARE mounted.)
CriticDecisionPanel  dark behind ELAI_CRITIC_OF_DESIGN_ENABLE (map + sheet agree).
RoadmapPanel         dark behind ELAI_AI_ROADMAP_SIGNALS_ENABLE (map + sheet agree).
Redesign action: give these an explicit "feature off (set ELAI_*_ENABLE=1)" empty state;
do not fake data. Decide whether the developer-discourse panels stay flag-gated.


================================================================================
BUCKET B — BACKEND WIRED, BUT NO PANEL EXISTS (data has no surface)
================================================================================
5 coding-metrics panels absent   E-49 (STABLE-Pass@1, $/CTS, the 7-tuple) — feature-map:1190.
BackendCostPanel absent          C-44 — feature-map:970 (BackendCostAccrued not surfaced).
VisionRAGDrawer absent           E-36 — feature-map:119.
branch-UI banner renderer absent C-23 — feature-map:274 (session-branch frontend banner).
ExternalBenchPanel deferred      E-9 — feature-map:1187.
HardwareModelCatalogPanel deferred J-9.
Redesign action: these are NEW surfaces to build IF the data matters to the user; each
has a live backend already. Confirm the command exists before designing the panel.


================================================================================
BUCKET C — PANEL EXISTS, DATA EMPTY BY DESIGN (bench-pending / flag-dark)
================================================================================
SafeDeltaPanel        BENCH-PENDING — the release-gating SAFEDELTA number; empty until a run.
*BenchPanel set       BENCH-PENDING — ARCBench, ModeRoutingBench, PrmBench, SycophancyBench.
LoraShipGatePanel     BENCH-PENDING — exists + mounted (feature-map "absent" is stale).
~15 DARK(flag) panels render empty unless their ELAI_*_ENABLE is set (the D/E adversary,
  sycophancy-probe, novelty-label, conformal-refusal, checker, observable-effect families).
Redesign action: design a uniform "no run yet" / "feature off" empty state. The data is
real once the run/flag fires; do not mistake empty for broken.


================================================================================
BUCKET D — PLANNED SURFACES NOT YET BUILT
================================================================================
UsageWindow.svelte + LiveUsageGauge   PLANNED J-28/J-29 (new deep token-usage view).
canvas "system" composition mode      J-31 (extends shipped canvas).
front-door ACK/notice/RESULT cards    J-32 (+ open-in-canvas deep-link).
H-Track-2 canvas pieces               partially shipped, DEMOTED (not killed).
Redesign action: leave layout room for these; the canvas + UsageWindow are the biggest
net-new surfaces the redesign should plan around.


================================================================================
CROSS-CUTTING REDESIGN FINDINGS (from the contract sheet)
================================================================================
1. ~75% of feature panels call the backend INLINE (invoke() inside the .svelte, no mock
   fallback) — they show empty/error offline. ~17 STORE-path panels silently render
   mock-data.ts fixtures offline, so a browser preview LOOKS live but is not. Make the
   mock-vs-live state visible (a dev-only MOCK ribbon).
2. Most get_* handlers return CmdResult<Value> (untyped) — the TS type is the front-end's
   ASSERTION, not a ts-rs contract. HANDLER-SHAPES.md carries the real field shapes;
   ssb_viewer.rs is the canonical wire-shape source for ~70% of panels.
3. Two genuinely missing handlers (resolve before wiring): get_pinned_rules
   (PinnedRulesPanel) and get_isolation_metric_status — NO Rust #[tauri::command] exists.
4. 8 Tauri event channels must be re-subscribed by any redesign (CONTRACT-SHEET 5e).
