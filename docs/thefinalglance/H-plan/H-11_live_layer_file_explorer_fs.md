# H-11: Live SSB layer + file explorer + scoped fs surface

STATUS: PLANNED (WAVE 2; BUILD; owns the +canvas_activity_event live-overlay transport)

Objective: Add the live layer (the calm sense of a living engine the orbit was good at, on a flat surface) + a VS
Code-style file explorer. The live layer is a CanvasActivityEvent SSB overlay drawn over the active mode: the active
node gets a halo, the traversed edge a dash-flow, a follow-agent toggle keeps the cursor-of-attention centered, and a
ghost-node resolution-failure path handles events for un-indexed files (never a silent gap). The file explorer syncs
bidirectionally over the EXISTING graph focus bus. The hard blocker (C13): NO fs surface exists for webview filesystem
browsing — capabilities/default.json grants ONLY core:default + dialog:default, tauri-plugin-fs is NOT a dependency,
and lib.rs registers only the dialog plugin. H-11 BUILDs the scoped fs surface and ACTIVATEs the orphaned list_projects/create_project.

Estimated effort: ~2 sessions (~10-14 h). Builds on the H-10 shell.
Pre-requisites: H-10 (the Canvas shell + window type) MUST land first. C-4 project-open SHIPPED
  (open_project_from_folder commands.rs:594, reindex_now :5936); the graph focus bus SHIPPED (graph.svelte.ts:31);
  ui/Tree.svelte SHIPPED; workspace.svelte.ts panelSplit SHIPPED (:84).
Parallel-safe with: H-12 (coordinate on the canvas shell + WindowHost). NOT parallel-safe with H-10.
Source: CANVAS-WORKBENCH-DESIGN.md §5 (the live layer) + §6 (file explorer + bidirectional sync); _grounding/H-GROUNDING-MAP.md C13 + the H-11 table. Doctrine: H-0_INDEX.md.

## Context

The live layer is the INTERVENTION surface (CANVAS §5): because the agent's intermediate state is spatial and visible,
the user can point at a node mid-run and correct it (the convergence question §9 raises — motivated by, not proven by,
the observability-gap diagnostic). The mechanism is an SSB overlay over whatever mode is active. The resolution-failure
path is MANDATORY and explicit (§5): a CanvasActivityEvent carries the target node id AND the raw file_path + symbol_name
+ mode FALLBACK, so when no matching node is on the canvas (an un-indexed file, a symbol below the LOD tier), the overlay
creates a transient GHOST node from the fallback, placed near the nearest resolvable ancestor, drawn in the unverified
treatment; the ghost reconciles into a real node when indexing catches up, or fades to a dropped-marker after 30s — never
a silent gap. The file explorer (§6): canvas->explorer (selecting a code.file/code.symbol node highlights the file in the
tree) and explorer->canvas (clicking a file focuses+centers the matching canvas node) ride the EXISTING focus bus
(graph.svelte.ts focus_node, keyed on path) — the explorer is just another publisher/consumer. The fs surface is BUILD
from scratch, scoped per root (never a whole-disk grant).

## Existing state (B/C/D/E/F/G coverage)
- open_project_from_folder (dashboard-v2/src-tauri/src/commands.rs:594) — SHIPPED. Dialog + std::fs canonicalize; callers ProjectSelector.svelte.
- reindex_now (commands.rs:5936) — SHIPPED. spawn_blocking DKG build; emits reindex-progress.
- list_projects (commands.rs:10366) + create_project (:10404) — WIRE-DARK at the UI layer (registered + stub.ts wrapper, ZERO .svelte/.ts callers; UI uses in-memory session.recentProjects).
- default.json (capabilities) — core:default + dialog:default ONLY; NO fs capability.
- Cargo.toml — tauri 2.11 + tauri-plugin-dialog 2.7; tauri-plugin-fs NOT a dependency.
- lib.rs:435 — only .plugin(tauri_plugin_dialog::init()).
- graph.svelte.ts focus bus (src/lib/state/graph.svelte.ts:31) — SHIPPED. focus_node/consume_focus/request_impact/consume_impact, keyed on node id == file path; cleared-after-consume. The bus the explorer rides.
- workspace.svelte.ts panelSplit (:84) — SHIPPED. Clamp 0.2-0.8, debounce 250ms — the reuse for the explorer-rail width.
- ui/Tree.svelte — SHIPPED but plan/status-shaped (TreeNode carries status); the explorer needs a filesystem node variant.
- AppShell.svelte grid — "48px 1fr 0" (the 48px Sidebar is the icon-nav rail; the explorer needs a NEW grid column).

## Reads (mandatory before coding)
- _grounding/H-GROUNDING-MAP.md C13 + the H-11 symbol table.
- CANVAS-WORKBENCH-DESIGN.md §5 (the live layer + the resolution-failure path) + §6 (the file explorer + the hard fs blocker + the multi-root rule + the grid correction).
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri\src\commands.rs:590-600,5930-6010,10360-10420 (open_project / reindex / list_projects / create_project)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri\capabilities\default.json + Cargo.toml + src/lib.rs:430-440 (the fs gap)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\state\graph.svelte.ts (the focus bus) + workspace.svelte.ts:80-160 (panelSplit)
- CLAUDE.md HARD RULES + anti-slopsquatting (verify the tauri-plugin-fs 2.x version) + the SSB 9-tuple lockstep.

## Tasks (WAVE 2 — build)

1. **+canvas_activity_event SSB (Pattern-2) (~90 LOC in ssb.rs).** ONE SsbActionType::CanvasActivityEvent carrying a
   spatial anchor (target node id) + the file_path + symbol_name + mode FALLBACK + the touch kind (reading/editing/
   validating/researching) + a short label. 9-tuple lockstep (variant + sql_str + parse + CHECK + payload + action_type()
   + test_payload + all_test_payloads + golden). SCHEMA bump baseline+1 (LIVE 207/484 -> re-read; the first H-N to bump
   also bumps e45_cross_language_drift_contract.rs:294). H-11 OWNS the live-overlay transport (it ships before H-13).

2. **The overlay store + ghost reconciliation (~220 LOC svelte/ts).** canvasActivity.svelte.ts subscribes to
   CanvasActivityEvent (+ existing FSM/GRC/evidence/bubble signals), resolves each to a node; the 3 failure cases
   (node in active mode -> glow; node in a non-active mode -> off-frame indicator pointing to the owning mode; node
   not on the canvas -> a transient ghost from the fallback, reconcile-or-fade-to-dropped-marker after 30s). The
   follow-agent toggle (default on) keeps the active node centered. Ported motion (halo-pulse, dash-flow, bubble-ring).

3. **FileExplorer.svelte + fileTree.svelte.ts + the new grid column (~280 LOC).** A collapsible tree built from the
   Tree primitive (with a new filesystem TreeNode variant: path, kind file|dir, no status); a NEW AppShell grid column
   (the 48px Sidebar is too narrow), width persisted in workspace.svelte.ts next to panelSplit; bidirectional sync over
   the existing focus bus (canvas->explorer: publish the selected node's file path; explorer->canvas: focus_node the matching node).

4. **BUILD the scoped fs command surface (~140 LOC tauri + capability).** read_dir / read_file / open_in_editor /
   mount_root Rust commands, scoped per root; add tauri-plugin-fs (VERIFY the current 2.x version via cargo — do NOT
   assume) ONLY if the Svelte layer needs direct JS reads (open_project_from_folder/reindex use std::fs and do not need
   it); grant scoped fs perms in default.json (fs:allow-read-dir + fs:allow-read-text-file + fs:scope limited to
   project_root, NEVER fs:default/whole-disk); register the plugin in lib.rs. Multi-root mounting (today one root in session.currentProject.path).

5. **Activate list_projects / create_project (~30 LOC).** Wire the orphaned commands (registered + stub.ts-wrapped, zero UI callers) into the explorer / project switcher.

6. **Decision doc workflow/reasoning/H-11-decisions.md (~110 LOC).** Records: the resolution-failure ghost path
   (never a silent gap); the fs surface is BUILD (scoped per root, never whole-disk); the new grid column; the explorer rides the existing focus bus; multi-root.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/ssb.rs::SsbActionType::CanvasActivityEvent (+ CanvasActivityKind + CanvasActivityPayload)
  - crates/dashboard-v2/src/lib/state/canvasActivity.svelte.ts (the overlay store + ghost reconciliation)
  - crates/dashboard-v2/src/lib/windows/FileExplorer.svelte + crates/dashboard-v2/src/lib/state/fileTree.svelte.ts
  - crates/dashboard-v2/src-tauri/src/commands_h11.rs::read_dir / read_file / open_in_editor / mount_root (scoped per root)
  - crates/dashboard-v2/src-tauri/capabilities/default.json (the scoped fs grant) + lib.rs (the plugin register + the new grid column)
consumes_symbols:
  - crates/dashboard-v2/src/lib/state/graph.svelte.ts::focus_node / consume_focus (the existing bus — REUSE)
  - crates/dashboard-v2/src-tauri/src/commands.rs::open_project_from_folder / reindex_now / list_projects / create_project
  - crates/dashboard-v2/src/lib/state/workspace.svelte.ts (panelSplit persistence pattern)
  - crates/dashboard-v2/src/lib/ui/Tree.svelte (the tree primitive — new filesystem node variant)
reachable_from:
  - the Canvas window (H-10) -> canvasActivity overlay subscriber  [the live layer]
  - AppShell new explorer column -> FileExplorer.svelte -> the scoped fs commands  [the explorer]
  - a live agent task -> CanvasActivityEvent SSB rows -> the overlay focus marker  [PRODUCTION live layer]
stub_ban:
  - NO silent gap on a resolution failure (a missing node -> a ghost that reconciles or fades to a dropped-marker)
  - NO whole-disk fs grant (scoped per root, fs:scope limited to project_root)
  - NO tauri-plugin-fs version assumed (verify via cargo — anti-slopsquatting)
  - NO fixture-only live layer (the focus marker travels from REAL CanvasActivityEvent rows, not static mount data)
  - NO hardcoded SCHEMA value (bump from LIVE 207/484)
  - blake3; no .unwrap()/.expect() in prod
frontend_pair:
  FileExplorer.svelte + canvasActivity.svelte.ts (svelte) + read_dir/read_file/open_in_editor/mount_root + a get_canvas_activity read (tauri). cross_lang_diff PASS; W-6 dashboard E2E.
observable_effect:
  trigger: "open a real project folder in the dashboard (NO env override); run a real agent task that touches files"
  expect_ssb: "CanvasActivityEvent rows land carrying the touched file_path/symbol_name/mode + kind; the overlay reads them live"
  expect_artifact: "the tree renders from disk via the new scoped fs commands (real fs calls fire); selecting a
    within-root canvas node highlights the file, a cross-root monorepo-sibling node shows the mount affordance and
    resolves, an external (node_modules) node shows the external state (not a silent no-op); the focus marker travels
    the graph node-by-node from real CanvasActivityEvent rows; an event for an un-indexed file renders a ghost that reconciles or fades to a dropped-marker"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-11_live_layer_file_explorer_fs.md --run-trigger
rg "CanvasActivityEvent|canvas_activity_event" .elai_cc/crates/orchestrator/src/ssb.rs
rg "canvas_activity_event" .elai_cc/crates/orchestrator/tests/action_type_lock.rs
test -f .elai_cc/crates/dashboard-v2/src/lib/windows/FileExplorer.svelte
test -f .elai_cc/crates/dashboard-v2/src-tauri/src/commands_h11.rs
rg "fs:allow-read-dir|fs:scope" .elai_cc/crates/dashboard-v2/src-tauri/capabilities/default.json  # scoped fs grant (not fs:default)
rg "focus_node" .elai_cc/crates/dashboard-v2/src/lib/windows/FileExplorer.svelte                   # explorer rides the existing bus
python .claude/tools/cross_lang_diff.py
test -f workflow/reasoning/H-11-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: --run-trigger PASS (the tree renders from disk; the focus marker travels from real CanvasActivityEvent rows; the ghost path works).
- [ ] W-2 default-ON: the live overlay + explorer active with no env override.
- [ ] W-3 production entrypoint: the overlay reads live CanvasActivityEvent rows from a real agent task; the fs commands fire from the real explorer; not test-only.
- [ ] W-4: FileExplorer + the overlay cross-lang mirrored; cross_lang_diff PASS.
- [ ] W-6: the explorer + the live layer verified THROUGH the dashboard (dashboard-smoker).
- [ ] W-9: canvas_activity_event round-trips via the SSB roundtrip contract + golden; SCHEMA bump named baseline+1; e45 drift fixed.
- [ ] The resolution-failure ghost path: a missing node renders a ghost that reconciles or fades to a dropped-marker (never a silent gap).
- [ ] The fs surface is scoped per root (no whole-disk grant); tauri-plugin-fs version verified.
- [ ] cargo check --workspace GREEN + dashboard svelte-check; auditor PASS.

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE; the overlay (produced) consumes CanvasActivityEvent (W-8) from a live task (W-3); the explorer consumes the fs commands.
- W-2: live overlay + explorer default-on.
- W-3: a real agent task emits CanvasActivityEvent; the explorer fires real fs commands; not test.
- W-4: FileExplorer + canvasActivity.svelte.ts; cross_lang_diff PASS.
- W-5: a runnable project-open + agent task -> a real tree + real overlay rows; in-process + dashboard.
- W-6: the explorer + live layer via dashboard-smoker.
- W-7: open_project_from_folder / list_projects / the focus bus / the fs gap grep-confirmed 2026-06-03 (C13).
- W-8: the overlay (produced) consumes CanvasActivityEvent; FileExplorer (produced) consumes read_dir + the focus bus.
- W-9: +canvas_activity_event Pattern-2 baseline+1; golden + e45 drift fix.
- W-10: Tier-1 in-process (the fs commands are local, scoped; the overlay is in-process).

## Tests to add
- crates/contracts/tests/h11_live_layer_fs_contract.rs (~180 LOC): (a) CanvasActivityEvent round-trips (the Pattern-2
  variant + the fallback fields); (b) the overlay resolves an event to a node, OR creates a ghost from the fallback when
  no node matches, OR points off-frame for a non-active mode; (c) a ghost reconciles into a real node when a matching id
  appears, and fades to a dropped-marker after the timeout; (d) the scoped fs read_dir is limited to the project root
  (a path-traversal escape is rejected); (e) the explorer<->canvas sync rides the existing focus bus (no new bus).

## Files touched
- MODIFY: crates/orchestrator/src/ssb.rs (~90 LOC: CanvasActivityEvent variant + Kind + payload + 9-tuple lockstep)
- MODIFY: crates/orchestrator/tests/action_type_lock.rs (~2 LOC) + e45_cross_language_drift_contract.rs:294 (drift)
- CREATE: crates/dashboard-v2/src/lib/state/canvasActivity.svelte.ts (~180 LOC) + windows/FileExplorer.svelte (~280 LOC) + state/fileTree.svelte.ts (~120 LOC)
- CREATE: crates/dashboard-v2/src-tauri/src/commands_h11.rs (~140 LOC: read_dir/read_file/open_in_editor/mount_root)
- MODIFY: crates/dashboard-v2/src-tauri/capabilities/default.json (scoped fs grant) + Cargo.toml (tauri-plugin-fs, verified) + lib.rs (plugin register + new grid column)
- MODIFY: crates/dashboard-v2/src/lib/layout/AppShell.svelte (~10 LOC: the new explorer grid column) + workspace.svelte.ts (~10 LOC: the rail width)
- CREATE: crates/contracts/tests/h11_live_layer_fs_contract.rs (~180 LOC)
- CREATE: workflow/reasoning/H-11-decisions.md (~110 LOC)

## Citations / Grounding
- CANVAS-WORKBENCH-DESIGN.md §5 (the live layer + the resolution-failure ghost path; arXiv:2603.26942 is motivating-not-proven per §20) + §6 (the bidirectional sync + the hard fs blocker + multi-root + the grid correction).
- ELAI ground truth (C13): NO fs surface (core:default + dialog:default only; no tauri-plugin-fs; lib.rs dialog-only);
  list_projects/create_project orphaned at UI; the focus bus graph.svelte.ts:31; the AppShell grid "48px 1fr 0".
- CLAUDE.md HARD RULES + anti-slopsquatting (tauri-plugin-fs version) + the 9-tuple SSB lockstep.

## Open questions / risks
- The dependency-relevant ghost placement (near the nearest resolvable ancestor) needs the folder super-node hierarchy from H-10 — coordinate the ghost placement with H-10's LOD tiers.
- The fs scope must be tight: a read_dir that follows a symlink out of the project root is an escape — canonicalize + reject out-of-root (the contract test asserts this).
- Multi-root: the DKG indexes multi-project workspaces, so a code.file node's path can point outside the opened folder (a monorepo sibling, node_modules) — the mount affordance + the external badge handle it; the C2.B3 cross-root case is an explicit test.

## Next
H-12 ports PLAN/RESEARCH modes + makes Canvas the default with the single-consumer SSB handover (so the orbit and the
canvas live layer never double-consume the stream). H-13 (model drives the canvas) adds the model-op transport on top.

End of H-11.
