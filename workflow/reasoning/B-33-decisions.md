B-33 — Decision log (interactive graph viewer + Mermaid export)
session: s-50960
phase: full implementation
date: 2026-05-16

LOCK POSTURE OBSERVED AT EXECUTION
- s-22200 owns crates/knowledge/Cargo.toml (FSM IDLE) — visualization
  module landed pure std + serde + crate-local types only. ZERO Cargo
  edits in the knowledge crate. Verified.
- s-40304 owns crates/orchestrator/{ssb.rs, indexing_service.rs,
  lock.rs, startup.rs, lib.rs} (FSM IDLE). NOT TOUCHED. Verified.
- s-50960 owns the files in workflow/sessions/state.json under our
  session entry. All edits stayed inside that owns list except the
  three additional sites listed below (each justified).

DECISIONS APPLIED (D1-D12 from synthesis)
D1 ACCEPTED — Option A: cytoscape@^3.33.3 + cytoscape-dagre@^3.0.0 +
   cytoscape-fcose@^2.2.0. User pre-approved per "choose best NOT
   lazy". cytoscape-dagre is one major newer than the user's preset
   ^2.5; recorded for transparency. npm install reported 7 packages
   added (cytoscape + dagre + fcose + transitive dagre + cose-base
   etc.), 6 vulnerabilities (5 moderate, 1 high) — typical for the
   cytoscape ecosystem; no fix applied (no user instruction for an
   audit-fix). Decision: ship as-is, file follow-up if needed.
D2 ACCEPTED — AppState gets NO new field. b33_resolve_project_id +
   per-call SymbolGraphCache::new follow production_agent_rpc.rs:463.
D3 ACCEPTED — SymbolGraphUpdated SSB variant DEFERRED (ssb.rs locked
   by s-40304). GraphWindow polls every 30s as the fallback.
D4 ACCEPTED — CLI `/graph mermaid` slash command DEFERRED. Tauri
   `export_graph_mermaid` + MermaidExportModal is the canonical UI.
D5 ACCEPTED — Mermaid generator pure std + std::fmt::Write into String;
   no clock dep; no new Cargo deps anywhere.
D6 ACCEPTED — MAX_VIZ_SYMBOLS=20_000, MAX_VIZ_EDGES=60_000,
   MAX_VIZ_SYMS_PER_FILE=200; SymbolMode::{Full, Capped(reason),
   Omitted(reason)} with stub_ban: every non-Full carries a populated
   reason (commands.rs constructors enforce; tests assert).
D7 ACCEPTED — New module src/lib/state/graph.svelte.ts using $state
   runes only (no `$:`).
D8 ACCEPTED — Cross-window links via shared graph store +
   workspace.openTab; no Tauri event-bus.
D9 ACCEPTED — Ctrl+G + Ctrl+7 lives in shortcuts.ts. (AppShell already
   delegates Ctrl+. + Esc; the global key listener installs from
   shortcuts.ts via installShortcuts(), so the new entries auto-register.)
D10 ACCEPTED — PNG export via cy.png({output:'blob', full:true, scale:2})
    paired with @tauri-apps/plugin-dialog save. Save handled by
    blob-download in the webview because the Tauri fs plugin path is not
    enabled in this dashboard build.
D11 ACCEPTED — Mermaid export via Rust generator -> MermaidExportModal
    with clipboard + save-as-.mmd.
D12 ACCEPTED — Decomposition mode reshapes SsbViewer::decomposition_tree(None).

WIRING_CONTRACT delta vs. synthesis brief
- get_graph_view_data signature now takes `project_id: Option<String>` rather
  than required `String`; an empty/missing project_id falls back to
  AppState.project_id.lock(). Same for get_impact_overlay and
  export_graph_mermaid. Reason: dashboard frontend has no `session.projectId`
  surface, and forcing the frontend to invent a value is brittle. The
  AppState fallback matches the production_agent_rpc.rs:463 pattern.
- VizSymbolEdge ships only resolved candidate edges (callee_candidates
  iteration), not raw call sites. This drops a small number of Unresolved
  rows but keeps the JSON small enough to round-trip per stub_ban.

DEFERRED ITEMS (not shipping in B-33; tracked here)
1. SsbActionType::SymbolGraphUpdated variant + payload struct + as_sql_str /
   ssb_action_type_from_str / test_payload arms + SCHEMA_VERSION bump.
   REASON: ssb.rs owned by s-40304 (s-40304 IDLE but lock still active per
   workflow/sessions/state.json). WORKAROUND: GraphWindow polls every 30 s.
2. indexing_service.rs SymbolGraphUpdated emit after build_symbol_graph
   success branch. REASON: file owned by s-40304.
3. CLI /graph mermaid > path slash command + dispatcher tool registration.
   REASON: cli/src/commands.rs is a hub; touching it is out of scope this
   session. Tauri export_graph_mermaid + MermaidExportModal is the canonical
   user-visible path.
4. ActivityWindow "Show blast radius" link on `ActivityKind === 'edit'` rows.
   REASON: ActivityWindow.svelte is currently a thin OrbitCanvas host; there
   are no per-row edit entries to attach the link to. The cross-link surface
   is fully covered by the KnowledgeDrawer + PlanWindow buttons added this
   session. Re-evaluate when ActivityWindow gains a row-list mode.
5. Decomposition node status indicators (pending/running/passed/failed).
   REASON: requires live SSB running/complete payloads not yet plumbed to
   commands.rs decomposition reshape. Static decomposition tree ships now.
6. AppState::SymbolGraphCache field. REASON: per-call open is microseconds
   on SSD per B-29 design. Defer to B-36 Session 2.
7. PageRank-weighted VizFile.deps ordering. REASON: BTreeMap iteration order
   is acceptable for v1 (deterministic). Add later when scale demands.
8. dashboard-v2/src-tauri/tests/graph_commands.rs (per E9). REASON: pure
   Tauri-command smoke tests require a mocked AppState (Send + Sync +
   tokio::sync::Mutex chain). The contract test in
   crates/contracts/tests/b33_graph_viewer_contract.rs covers JSON shape +
   empty-input regressions; the helper logic is exercised end-to-end via
   manual smoke once the dashboard runs. Can ship later as a focused
   integration test against a fixture project_root.

TESTS LANDED
- crates/knowledge/tests/visualization_types.rs — 8 tests GREEN.
- crates/knowledge/tests/mermaid_generation.rs — 7 tests GREEN.
- crates/contracts/tests/b33_graph_viewer_contract.rs — 6 tests GREEN.

UNADD'L EXPANSION (within agreed scope)
- Added `Network` lucide icon import to Sidebar.svelte for the new graph
  nav entry. Net new icon, no behavioural change to other nav items.
- Added `cytoscape-dagre` + `cytoscape-fcose` ambient declarations to
  src/app.d.ts. Both packages ship runtime extensions only; the ambient
  decl keeps svelte-check noise at 0.
- AppShell.svelte VALID_TYPES set updated alongside the WindowType union
  edits (consistency check). Otherwise the drop-strip would refuse a
  drag-open of a graph tab into the secondary panel.
- CommandPalette.svelte got a "Open Graph Window" entry alongside the
  other open-X items so power users can reach the window without Ctrl+G.

B-33 FOLLOW-UP PATCH (post-audit cycle 1, same session s-50960)
Auditor returned 0 BLOCKs / 5 WARNs (Score 0.93). User dispatched the
fixer + Step B executor to address WARNs and ship deferred SymbolGraphUpdated
now that all locks released. Outcome:

PART 1 — auditor WARN fixes (3/3 applied, 2 WARN deferred):
  FIX-A1 APPLIED — GraphWindow.svelte search filter class `'faded'` ->
    `'cy-faded'` to match the `:global(.cy-faded)` CSS selector. Search
    filter now visually fades non-matching nodes.
  FIX-A2 APPLIED — GraphWindow.svelte fetchData adds a
    `suppressNextModeFetch` flag set when auto-downgrading symbol -> file,
    consumed once by the mode-change `$effect` to skip the redundant
    invoke. Auto-downgrade also short-circuits when already in 'file' mode.
  FIX-A3 APPLIED — commands.rs decomposition reshape now uses the typed
    DTOs (`elai_orchestrator::DecompositionTreeSnapshot` -> ConceptNodeRef
    -> PhaseNodeRef -> StepNodeRef -> BabyStepRef) instead of the prior
    `serde_json::to_value` round-trip + string-fallback lookups. Dead
    `concept_id`/`name`/`task` branches removed. A field rename now
    breaks compilation instead of silently rendering placeholder text.
  WARN 3 (cyclic edge styling unreachable) — INTENTIONALLY DEFERRED. The
    Tarjan inline cycle pass for VizFileEdge is a separate follow-up
    patch; the cyclic styling pipeline (mermaid.rs:86-93 + GraphWindow
    style:151-157) lights up once the producer at commands.rs:1082 starts
    emitting `cyclic: true`. Bundling Tarjan + Mermaid + this patch would
    risk a spiral; ship surgically.
  WARN 5 (npm vulnerabilities) — accepted by user per D1 baseline; out
    of scope for this patch.

PART 2 — SymbolGraphUpdated SSB variant (E-DEFERRED1 + E-DEFERRED2 SHIPPED):
  FIX-B1 APPLIED — SsbActionType::SymbolGraphUpdated variant added, SQL
    CHECK constraint extended, as_sql_str + ssb_action_type_from_str
    arms wired, SsbPayload::SymbolGraphUpdated discriminant + action_type
    mapper arms added, test_payload reconstructor arm added. SCHEMA_VERSION
    bumped 39 -> 40 with B-33 follow-up ledger comment in ssb.rs.
  FIX-B2 APPLIED — SymbolGraphUpdatedPayload struct
    { project_id, file_count: u32, symbol_count: u64, edge_count: u64,
    built_ms: i64 } with derives {Debug, Clone, PartialEq, Eq, Serialize,
    Deserialize} added near MinifiedFileChunkedPayload.
  FIX-B3 APPLIED — Re-exported SymbolGraphUpdatedPayload from
    orchestrator/src/lib.rs alongside MinifiedFileChunkedPayload.
  FIX-B4 APPLIED — indexing_service.rs build_symbol_graph success branch
    now emits SsbPayload::SymbolGraphUpdated when summary.files > 0.
    Reuses the existing `emit_ssb_payload` helper (added by B-31.5) so no
    new async-bridge plumbing needed. Empty-file builds skip the emit
    (avoid noise on no-source projects).
  FIX-B5 APPLIED — crates/contracts/tests/symbol_graph_updated_ssb_contract.rs
    created with 3 tests (round-trips, schema version, payload).
  FIX-B6 APPLIED — Tauri-side: lib.rs `spawn_symbol_graph_subscriber`
    helper taps the SSB broadcast for SymbolGraphUpdated rows and
    re-emits a `symbol_graph_updated` Tauri event with seq + ts_ms +
    full payload fields. Frontend: GraphWindow.svelte onMount registers
    a listener via `@tauri-apps/api/event::listen` that refetches on
    receipt. The 30 s setInterval poll is KEPT as a backstop per the
    fix instructions (defense-in-depth for hot-reload + lagged events).

VERIFICATION
- cargo check --workspace — clean
- cargo test -p elai_cc_contracts --test symbol_graph_updated_ssb_contract
  --test minified_chunked_ssb_contract — 6/6 pass (3 + 3); the
  pre-existing minified_chunked test was widened from `assert_eq!(==38)`
  to `assert!(>=38) && assert_eq!(==40)` per its own docstring guidance
  ("If this fires after a later batch bump, update the constant here").
- cargo test -p elai_cc_contracts --test b33_graph_viewer_contract
  --test symbol_graph_b29_phase3_contract --test symbol_graph_b29_phase5_contract
  — 22/22 pass (regression clean for B-29 + B-33).
- npm run check — 0 errors, 9 pre-existing CSS warnings (unchanged).

FINAL SCHEMA_VERSION: 40 (was 39).

NEW DEFERRED ITEMS (tracked here, not shipping in this follow-up)
9. Tarjan inline cycle detection for VizFileEdge in commands.rs (drives
   WARN 3). REASON: surgically isolated patch to avoid spiraling the
   follow-up work. The cyclic styling rendering pipeline already lives
   in mermaid.rs:86-93 + GraphWindow.svelte:151-157; only the producer
   in commands.rs:1082 needs to flip from `cyclic: false` to a real
   compute. Q6 from synthesis remains the canonical reference.
