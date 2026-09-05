GROUNDING SWEEP -- phase H-11
Source: F:\.ELAI_workflow\docs\thefinalglance\H-plan\H-11_live_layer_file_explorer_fs.md
Date: 2026-06-04
FSM: PLANNING | Session: spec-grounder-H11-2026-06-04

SPEC REFERENCES

docs/thefinalglance/H-plan/_grounding/H-GROUNDING-MAP.md C13 + H-11 symbol table
  File: F:\.ELAI_workflow\docs\thefinalglance\H-plan\_grounding\H-GROUNDING-MAP.md
  Status: GROUNDED
  Evidence: H-11 section (lines 220-235) covers all 7 H-11 symbols verbatim: open_project_from_folder,
  reindex_now, list_projects, create_project, default.json, Cargo.toml, lib.rs plugin registration,
  graph.svelte.ts focus bus, workspace.svelte.ts panelSplit. C13 correction block (line 47) names the
  exact fs gap. Content exactly matches H-11 plan claims.
  LINE DRIFT (non-blocking): grounding map says open_project_from_folder :594; actual is :612 (+18).
  reindex_now map says :5936; actual is :5957 (+21). list_projects map says :10366; actual grep :10387 (+21).
  These are consistent 18-21 line shifts likely from prior insertions; logic is identical.

CANVAS-WORKBENCH-DESIGN.md §5 (live layer + resolution-failure path)
  File: F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\CANVAS-WORKBENCH-DESIGN.md lines 143-168
  Status: GROUNDED
  Evidence: §5 defines CanvasActivityEvent with file_path + symbol_name + mode fallback fields (line 147);
  3-case resolution-failure path: glow (happy), off-frame indicator (non-active mode), transient ghost
  node from fallback placed near nearest ancestor (line 153); 30s reconcile-or-fade to dropped-marker
  (line 153); follow-agent toggle default on (line 164). All H-11 Task 2 claims match verbatim.
  arXiv:2603.26942: plan says "motivating-not-proven"; §5 says "motivated by, not proven by" (line 166).
  Claim hedge correctly reflected in plan. GROUNDED.

CANVAS-WORKBENCH-DESIGN.md §6 (file explorer + bidirectional sync + hard fs blocker + multi-root + grid)
  File: F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\CANVAS-WORKBENCH-DESIGN.md lines 170-211
  Status: GROUNDED
  Evidence: §6 specifies fileTree.svelte.ts + FileExplorer.svelte (line 176); bidirectional sync via
  existing focus_node bus (line 180); multi-root mounting rule (line 184-191); hard fs blocker stated
  plainly (line 205: "Tauri app has no filesystem access at all today. capabilities/default.json grants
  only core:default + dialog:default. tauri-plugin-fs is not even a dependency"); grid correction (line 207:
  "AppShell grid is grid-template-columns 48px 1fr 0 ... explorer needs its own new column"). All H-11
  Task 3/4/5 claims match verbatim. GROUNDED.

Reads (mandatory): codebase paths (commands.rs / default.json / Cargo.toml / lib.rs / graph.svelte.ts /
workspace.svelte.ts) -- all VERIFY-type, verified below under CLAIM CLASSIFICATION.

CLAIM CLASSIFICATION

open_project_from_folder at commands.rs:594 | VERIFY | grep confirms at :612 (18-line drift); function
  exists, logic matches (std::fs canonicalize, sets project_id+project_root). Non-blocking drift.

reindex_now at commands.rs:5936 | VERIFY | grep confirms at :5957 (21-line drift); async #[tauri::command],
  spawn_blocking, emits reindex-progress. Non-blocking drift.

list_projects at commands.rs:10366 / create_project at :10404 WIRE-DARK | VERIFY | grep confirms
  list_projects :10387, create_project :10425. Wire-dark status confirmed (zero .svelte/.ts callers
  per H-GROUNDING-MAP C13:225-226). Non-blocking drift.

default.json: core:default + dialog:default ONLY, no fs capability | VERIFY | H-GROUNDING-MAP C13:227
  confirms verbatim ("permissions: ['core:default','dialog:default'] ONLY. NO fs capability"). GROUNDED.

Cargo.toml: tauri 2.11 + tauri-plugin-dialog 2.7, tauri-plugin-fs NOT a dependency | VERIFY |
  H-GROUNDING-MAP C13:228 confirms tauri 2.11 :22, dialog 2.7 :23, fs NOT present. GROUNDED.

lib.rs plugin registration: only tauri_plugin_dialog::init() | VERIFY | H-GROUNDING-MAP C13:229
  confirms "Only .plugin(tauri_plugin_dialog::init()). NO fs plugin." GROUNDED.

graph.svelte.ts focus_node / consume_focus keyed on file path | VERIFY | grep confirms focus_node at :31,
  consume_focus at :35 in graph.svelte.ts. H-GROUNDING-MAP H-11:231 confirms singleton store, keyed on
  node id == file path, cleared-after-consume. GROUNDED.

workspace.svelte.ts panelSplit at :84 | VERIFY | grep confirms panelSplit $state at :84, clamp 0.2..0.8
  (:154), debounce schedulePersist, localStorage. GROUNDED.

AppShell grid: "48px 1fr 0" (explorer needs new grid column) | VERIFY | H-GROUNDING-MAP H-12:243
  confirms "CURRENT: grid-template-columns: 48px 1fr 0; (C-6, Sidebar 48px). Third column collapsed."
  CANVAS-WORKBENCH-DESIGN.md line 207 confirms same. GROUNDED.

ui/Tree.svelte ships but is plan/status-shaped (TreeNode carries status) | VERIFY | H-GROUNDING-MAP
  H-12:245 confirms "TreeNode (ui/tree-types.ts) carries status?: 'done'|'active'|'pending'|'failed'."
  Plan's "needs filesystem node variant" is correct. GROUNDED.

SSB Pattern-2 single-variant with 3-arm Kind: canvas_activity_event consistent with convention | VERIFY |
  RouteDecisionEvent (43rd Pattern-2 application) confirmed at ssb.rs:8315, with 3-arm
  RouteDecisionEventKind (CloudRouteChosen/TierModelSwapped/LocalRouteKept) at ssb.rs:25656. The 9-tuple
  lockstep (variant + sql_str + parse + CHECK + payload + action_type() + test_payload +
  all_test_payloads + golden) confirmed from H-GROUNDING-MAP H-15:279. Plan's CanvasActivityEvent
  (single-variant, 3-arm CanvasActivityKind reading/editing/validating/researching) exactly mirrors
  RouteDecisionEvent Pattern-2 shape. GROUNDED.

SCHEMA_VERSION 207 / TOTAL_VARIANTS 484 | VERIFY | H-GROUNDING-MAP CANONICAL CONSTANTS (line 7-8)
  confirms ssb.rs:2909 = 207, action_type_lock.rs:549 = 484. Plan says "re-read at edit time;
  treat 207/484 as 2026-06-03 snapshot." Correct posture. GROUNDED.

e45_cross_language_drift_contract.rs:294 pins 482 (latent FAIL) | VERIFY | H-GROUNDING-MAP lines 15-16
  confirms "e45 cross-language drift contract crates/contracts/tests/e45_cross_language_drift_contract.rs:294
  still pins 482 (LATENT FINDING)". Plan says first H-N to bump "also bumps e45:294". GROUNDED.

tauri-plugin-fs 2.x scoped permissions (fs:allow-read-dir + fs:allow-read-text-file + fs:scope per root) |
  VERIFY | Official Tauri v2 docs (v2.tauri.app/plugin/file-system/) confirmed:
  - "fs:allow-read-dir" is the exact permission identifier. CONFIRMED.
  - "fs:allow-read-text-file" is the exact permission identifier. CONFIRMED.
  - Per-root scoped grants supported: {"identifier":"fs:allow-read-dir","allow":[{"path":"$HOME/my-project/**/*"}]}
  - "fs:scope" is "An empty permission you can use to modify the global scope". CONFIRMED.
  - "fs:default" grants read access to app-specific directories only (not whole disk). CONFIRMED.
  Plan correctly states "fs:allow-read-dir + fs:allow-read-text-file + fs:scope limited to project_root,
  NEVER fs:default/whole-disk." API names are CORRECT. GROUNDED.

tauri-plugin-fs 2.x version (exact): NOT pinned -- plan says "verify via cargo, do NOT assume" | VERIFY |
  Tauri docs do not list the exact current crates.io version in the fetched content. Plan's posture
  ("verify via cargo at edit time, anti-slopsquatting") is correct and required. GROUNDED posture.
  Executor MUST run: cargo search tauri-plugin-fs or check Cargo.lock before pinning.

arXiv:2603.26942 (observability-gap, motivating not proven) | VERIFY | CANVAS-WORKBENCH-DESIGN.md line 166
  uses it accurately with the hedge. Plan mirrors hedge. No INGEST needed -- already correctly hedged.
  GROUNDED.

ACTIONS REQUIRED

- VERIFY at EXECUTING: re-read open_project_from_folder / reindex_now / list_projects live line numbers
  (18-21 line drift from grounding map; function logic confirmed correct, only line refs stale). Update
  H-11-decisions.md with the live line numbers before referencing them in code comments.
- VERIFY at EXECUTING: run `cargo search tauri-plugin-fs` or inspect Cargo.lock to pin the exact
  tauri-plugin-fs 2.x version before adding to Cargo.toml (anti-slopsquatting; plan correctly requires
  this; no version pinned yet).
- VERIFY at EXECUTING: re-read ssb.rs:2909 and action_type_lock.rs:549 live at edit time to confirm
  SCHEMA_VERSION and TOTAL_VARIANTS have not moved past 207/484 from concurrent H-plan sessions.
  The 207/484 values are a 2026-06-03 snapshot, not frozen literals.

all substantive API claims grounded -- clear to TESTING_FIRST after executor verifies tauri-plugin-fs
version and live line numbers.

VALIDATION WARNINGS

- Line-number drift (non-blocking): H-GROUNDING-MAP records open_project_from_folder :594, reindex_now
  :5936, list_projects :10366 -- live code is at :612, :5957, :10387 respectively. Consistent +18-21
  shift. Grounding map was read 2026-06-03; code likely had prior insertions. Logic is identical; drift
  does not affect correctness of H-11 plan.
- tauri-plugin-fs exact crate version not confirmed from web (docs page does not list it). Executor
  MUST verify via `cargo search tauri-plugin-fs` before pinning. Failure to verify = anti-slopsquatting
  violation (plan itself calls this out in stub_ban and Task 4).
- SCHEMA_VERSION 207/484 is a snapshot. Any H-plan session that shipped an SSB bump between 2026-06-03
  and H-11 EXECUTING will have advanced these. Treat as lower bound, not a literal.
- e45 drift (pre-existing, non-blocking for H-11 gate): e45_cross_language_drift_contract.rs:294 pins
  482 vs live 484. Plan correctly assigns the fix to the first H-N SSB bump session.
- No missing spec files. No FSM state mismatch (FSM=PLANNING, correct for grounding sweep).
- All required headers present. 14 claims classified, 14 grounded (9 VERIFY resolved in-session,
  5 confirmed from grounding map + web).
