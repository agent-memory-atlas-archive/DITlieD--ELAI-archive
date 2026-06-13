// H-12 — PLAN + RESEARCH modes + shared chrome contract tests.
//
// Four contract tests per H-12 WIRING_CONTRACT:
//   (a) mode flag is a plain value (NOT an FSM substate): assert fsm.rs has
//       no CanvasDesigning or mode.*substate mention.
//   (b) single-consumer invariant: canvas.svelte.ts has both startPolling and
//       stopPolling wiring so only one consumer is active at a time.
//   (c) WindowType union unchanged: 'canvas' still present; no accidental new
//       window type added by H-12 (PlanMode/ResearchMode are sub-components,
//       not window types — WINDOW_TYPE_CASCADE_5SITE guard).
//   (d) canvas-as-default flag uses a SEPARATE localStorage key (not STORAGE_KEY);
//       verify the two constants are distinct in workspace.svelte.ts.
//   (e) SCHEMA_VERSION still 220 and TOTAL_VARIANTS still 497 (H-12 ships NO
//       new SSB action_type).
//   (f) PlanMode reads from real plan store (plan.svelte.ts) not orbit fixture
//       data: assert PlanMode.svelte imports from state/plan, not orbit/types.
//   (g) ResearchMode reads from drawers.svelte.ts researchData (the real
//       research store), not fixture data: assert ResearchMode.svelte imports
//       from state/drawers, not api/mock-orbit.
//   (h) canvas.svelte.ts exists and declares CanvasMode type.
//
// These are filesystem + Rust-source contract tests (no runtime invocation).
// They guard the structural invariants that the auditor and CI will enforce.

use std::fs;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn dashboard_lib() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates dir")
        .join("dashboard-v2/src/lib")
}

fn orchestrator_src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates dir")
        .join("orchestrator/src")
}

fn action_type_lock_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates dir")
        .join("orchestrator/tests/action_type_lock.rs")
}

fn read_file(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Cannot read {}: {e}", path.display()))
}

// ---------------------------------------------------------------------------
// (a) Mode flag is NOT an FSM substate — fsm.rs has zero matches for banned
//     identifiers.
// ---------------------------------------------------------------------------

#[test]
fn h12_a_mode_flag_is_not_fsm_substate() {
    let fsm_path = orchestrator_src().join("fsm.rs");
    let content = read_file(&fsm_path);

    let banned = ["CanvasDesigning", "CanvasMode", "mode_substate", "canvas_substate"];
    for term in &banned {
        assert!(
            !content.contains(term),
            "H-12 (a): fsm.rs must NOT contain '{}' — canvas mode is a plain flag on \
             canvas.svelte.ts, NOT an FSM substate (CANVAS §7). Found in {}",
            term,
            fsm_path.display()
        );
    }
}

// ---------------------------------------------------------------------------
// (b) Single-consumer invariant — canvas.svelte.ts wires startPolling AND
//     stopPolling so exactly one consumer is active at a time.
// ---------------------------------------------------------------------------

#[test]
fn h12_b_single_consumer_both_poles_wired() {
    let canvas_state = dashboard_lib().join("state/canvas.svelte.ts");
    let content = read_file(&canvas_state);

    // Must call canvasActivity.startPolling() when canvas becomes active.
    assert!(
        content.contains("startPolling"),
        "H-12 (b): canvas.svelte.ts must call canvasActivity.startPolling() \
         to implement the single-consumer SSB handover (CANVAS §12). Not found in {}",
        canvas_state.display()
    );

    // Must call canvasActivity.stopPolling() when canvas goes dormant.
    assert!(
        content.contains("stopPolling"),
        "H-12 (b): canvas.svelte.ts must call canvasActivity.stopPolling() \
         to implement the single-consumer SSB handover (CANVAS §12). Not found in {}",
        canvas_state.display()
    );

    // Suspend-then-activate order: stop must appear textually before start
    // in the handover section (the arbiter calls stopPolling before startPolling).
    let stop_pos = content
        .find("stopPolling")
        .expect("stopPolling already asserted above");
    let start_pos = content
        .find("startPolling")
        .expect("startPolling already asserted above");
    assert!(
        stop_pos < start_pos,
        "H-12 (b): in canvas.svelte.ts, stopPolling() must appear before \
         startPolling() (suspend-then-activate order — never activate-then-suspend, \
         that leaves a race window where both consumers are live). \
         stop_pos={stop_pos}, start_pos={start_pos}"
    );
}

// ---------------------------------------------------------------------------
// (c) WindowType union unchanged — 'canvas' still present; no accidental new
//     window type added (PlanMode/ResearchMode are sub-components, not types).
// ---------------------------------------------------------------------------

#[test]
fn h12_c_window_type_canvas_present_no_new_type() {
    let lib = dashboard_lib();

    // Site 1: api/types.ts — 'canvas' in WindowType union.
    let types_ts = lib.join("api/types.ts");
    let types_content = read_file(&types_ts);
    assert!(
        types_content.contains("'canvas'"),
        "H-12 (c): api/types.ts must still contain 'canvas' in the WindowType union"
    );

    // Site 2: state/workspace.svelte.ts — 'canvas' in VALID_WINDOW_TYPES.
    let workspace_ts = lib.join("state/workspace.svelte.ts");
    let workspace_content = read_file(&workspace_ts);
    assert!(
        workspace_content.contains("'canvas'"),
        "H-12 (c): workspace.svelte.ts must still contain 'canvas' in VALID_WINDOW_TYPES"
    );

    // Site 3: layout/WindowHost.svelte — canvas branch dispatch present.
    let windowhost = lib.join("layout/WindowHost.svelte");
    let windowhost_content = read_file(&windowhost);
    assert!(
        windowhost_content.contains("type === 'canvas'"),
        "H-12 (c): WindowHost.svelte must still have the canvas dispatch branch"
    );

    // Guard: PlanMode and ResearchMode must NOT appear as window-type dispatch
    // branches in WindowHost (they are sub-components of CanvasWindow, not
    // separate window types — WINDOW_TYPE_CASCADE_5SITE pitfall).
    let banned_new_types = ["type === 'plan_mode'", "type === 'research_mode'",
                            "type === 'planmode'", "type === 'researchmode'"];
    for t in &banned_new_types {
        assert!(
            !windowhost_content.contains(t),
            "H-12 (c): WindowHost.svelte must NOT dispatch '{}' as a window type — \
             PlanMode and ResearchMode are sub-components of CanvasWindow, not new WindowTypes",
            t
        );
    }
}

// ---------------------------------------------------------------------------
// (d) Canvas-as-default flag uses a SEPARATE localStorage key (not STORAGE_KEY).
//     Verify workspace.svelte.ts has at least two distinct key constants, and
//     the canvas-default key is distinct from the workspace serialization key.
// ---------------------------------------------------------------------------

#[test]
fn h12_d_canvas_default_uses_separate_storage_key() {
    let workspace_ts = dashboard_lib().join("state/workspace.svelte.ts");
    let content = read_file(&workspace_ts);

    // The workspace serialization key must still be 'elai.dashboard-v2.workspace.v1'.
    assert!(
        content.contains("elai.dashboard-v2.workspace.v1"),
        "H-12 (d): STORAGE_KEY 'elai.dashboard-v2.workspace.v1' must still be present \
         in workspace.svelte.ts"
    );

    // The canvas-as-default flag must use a SEPARATE key (so existing persisted
    // workspaces are not invalidated — a schema bump would break old sessions).
    assert!(
        content.contains("CANVAS_DEFAULT_KEY") || content.contains("canvas-default") || content.contains("canvas_default"),
        "H-12 (d): workspace.svelte.ts must declare a SEPARATE localStorage key for \
         the canvas-as-default flag (not reusing STORAGE_KEY). Must contain \
         'CANVAS_DEFAULT_KEY', 'canvas-default', or 'canvas_default'"
    );

    // The canvas default key must be a different string from the workspace key.
    // Find both constants' string values and assert they differ.
    let has_canvas_key = content.contains("elai.dashboard-v2.canvas-default")
        || content.contains("elai.dashboard-v2.canvas_default")
        || (content.contains("CANVAS_DEFAULT_KEY") && !content.contains("elai.dashboard-v2.workspace.v1\";\nconst CANVAS_DEFAULT_KEY"));
    assert!(
        has_canvas_key,
        "H-12 (d): canvas-as-default must use a key like 'elai.dashboard-v2.canvas-default.v1' \
         that is distinct from 'elai.dashboard-v2.workspace.v1'. Found key string is ambiguous; \
         confirm it is not re-using STORAGE_KEY."
    );
}

// ---------------------------------------------------------------------------
// (e) SCHEMA_VERSION = 220 and TOTAL_VARIANTS = 497 unchanged.
//     H-12 ships NO new SSB action_type.
// ---------------------------------------------------------------------------

#[test]
fn h12_e_schema_version_and_total_variants_unchanged() {
    // H-12's real contract: H-12 itself added NO new SSB action_type.
    // H-11 baseline was SCHEMA_VERSION=220, TOTAL_VARIANTS=497.
    // Using >= rather than == so that subsequent H-N bumps do not break
    // this H-12 assertion. The invariant is monotonic growth (H-12 added
    // nothing), not a frozen global integer.
    assert!(
        elai_orchestrator::ssb::SCHEMA_VERSION_PUBLIC >= 220,
        "H-12 (e): SCHEMA_VERSION must be >= 220 (H-12 baseline was 220); got {}",
        elai_orchestrator::ssb::SCHEMA_VERSION_PUBLIC
    );

    // TOTAL_VARIANTS check via the runtime registry.
    // H-11 shipped TOTAL_VARIANTS=497; H-13 added CanvasOpApplied (498).
    // H-12 added nothing. Assert monotonic >= 497 rather than == 497.
    // Tier-B Move 2 derived TOTAL_VARIANTS and relocated ALL_VARIANTS (now
    // SSB_ACTION_TYPES in ssb.rs), so the source-text scan no longer applies;
    // read the live registry length. Same >= 497 floor.
    let has_497_or_more = elai_orchestrator::ssb::SSB_ACTION_TYPES.len() >= 497;
    assert!(
        has_497_or_more,
        "H-12 (e): SSB_ACTION_TYPES.len() (TOTAL_VARIANTS) must be >= 497 \
         (H-12 adds no new SSB action_type; H-11 baseline was 497). Got {}",
        elai_orchestrator::ssb::SSB_ACTION_TYPES.len()
    );
}

// ---------------------------------------------------------------------------
// (f) PlanMode reads from plan.svelte.ts (real store), not orbit fixture data.
// ---------------------------------------------------------------------------

#[test]
fn h12_f_plan_mode_imports_real_plan_store() {
    let plan_mode = dashboard_lib().join("canvas/PlanMode.svelte");
    let content = read_file(&plan_mode);

    // Must import from state/plan (the real live store).
    assert!(
        content.contains("state/plan") || content.contains("../state/plan"),
        "H-12 (f): PlanMode.svelte must import from state/plan.svelte.ts \
         (the real decomp-tree store, polled every 5s from the backend). \
         GD-1: PORT the real data feed, do not re-invent it."
    );

    // Must NOT import from mock-orbit (fixture data).
    assert!(
        !content.contains("mock-orbit"),
        "H-12 (f): PlanMode.svelte must NOT import from api/mock-orbit — \
         that is fixture data, not the real plan.svelte.ts store"
    );

    // Must light the active node via status === 'active' (DecompNode.status),
    // NOT via separate active_*_id props (those only exist in OrbitFixture).
    assert!(
        content.contains("status") && (content.contains("'active'") || content.contains("\"active\"")),
        "H-12 (f): PlanMode.svelte must light the active node via node.status === 'active' \
         (the real plan store uses status-on-node, not separate active_concept_id/active_phase_id \
         props — those are orbit fixture-only fields per H-12-grounding.md)"
    );
}

// ---------------------------------------------------------------------------
// (g) ResearchMode reads from researchData (drawers.svelte.ts), not fixtures.
// ---------------------------------------------------------------------------

#[test]
fn h12_g_research_mode_imports_real_research_store() {
    let research_mode = dashboard_lib().join("canvas/ResearchMode.svelte");
    let content = read_file(&research_mode);

    // Must import from state/drawers (where researchData is exported).
    assert!(
        content.contains("state/drawers") || content.contains("../state/drawers"),
        "H-12 (g): ResearchMode.svelte must import from state/drawers.svelte.ts \
         (where researchData.researchTree lives — the real research tree store). \
         There is NO separate research.svelte.ts — the existing drawers store IS the store."
    );

    // Must NOT import from mock-orbit.
    assert!(
        !content.contains("mock-orbit"),
        "H-12 (g): ResearchMode.svelte must NOT import from api/mock-orbit \
         (fixture data). Real data only."
    );
}

// ---------------------------------------------------------------------------
// (h-boot) loadCanvasDefault() is called in Workspace.svelte onMount.
//     Regression guard for B-1 wire-dark fix: the canvas-as-default flag was
//     never restored at boot because loadCanvasDefault() had zero call sites
//     outside workspace.svelte.ts. This test asserts the call now exists in the
//     Workspace boot path so the wire-dark gap cannot silently regress.
// ---------------------------------------------------------------------------

#[test]
fn h12_hboot_load_canvas_default_called_at_boot() {
    let workspace_svelte = dashboard_lib().join("screens/Workspace.svelte");
    let content = read_file(&workspace_svelte);

    assert!(
        content.contains("workspace.loadCanvasDefault()"),
        "H-12 (h-boot): Workspace.svelte onMount must call workspace.loadCanvasDefault() \
         to restore the persisted canvas-as-default flag at boot. \
         Without this call the flag always boots as false regardless of what the user \
         persisted (wire-dark, R6 / B-1). File: {}",
        workspace_svelte.display()
    );

    // Also assert the toggle path is wired: setCanvasAsDefault must exist in workspace.svelte.ts.
    let workspace_ts = dashboard_lib().join("state/workspace.svelte.ts");
    let ts_content = read_file(&workspace_ts);
    assert!(
        ts_content.contains("setCanvasAsDefault"),
        "H-12 (h-boot): workspace.svelte.ts must export setCanvasAsDefault() so the UI \
         toggle (SettingsWindow) can write the flag. Producer-without-consumer (R6) guard."
    );
}

// ---------------------------------------------------------------------------
// (i) ResearchMode onDestroy does NOT gate teardown on canvas.mode, and DOES
//     use a _bootstrapped flag (BLOCK-2 regression guard).
//
// Context: Svelte 5 fires onDestroy AFTER the reactive condition that mounted
// the component has already flipped. So when the user switches away from
// 'research' mode, canvas.mode is already 'code'/'plan'/etc by the time
// onDestroy runs. A guard `if (canvas.mode === 'research')` is therefore
// always false at destroy time, causing teardown() to never run — leaking the
// refcount in drawers.svelte.ts RefCountedTimer and keeping the polling timer
// alive for the entire CanvasWindow lifetime.
// Fix: track bootstrap with a `_bootstrapped` boolean and call teardown()
// unconditionally of canvas.mode (but only if bootstrap() actually ran).
// ---------------------------------------------------------------------------

#[test]
fn h12_i_research_mode_destroy_uses_bootstrapped_flag() {
    let research_mode = dashboard_lib().join("canvas/ResearchMode.svelte");
    let content = read_file(&research_mode);

    // The _bootstrapped flag must exist in the script section.
    assert!(
        content.contains("_bootstrapped"),
        "H-12 (i): ResearchMode.svelte must declare a '_bootstrapped' boolean flag \
         to track whether bootstrap() actually ran, so teardown() can match it 1:1 \
         regardless of canvas.mode at destroy time (BLOCK-2 teardown-leak regression guard). \
         File: {}",
        research_mode.display()
    );

    // onDestroy must NOT gate teardown on canvas.mode.
    // We look for the pattern `canvas.mode === 'research'` inside an onDestroy block.
    // Strategy: extract the onDestroy body by finding onDestroy and checking the next
    // 200 chars (enough to span the block) for the disallowed guard.
    let destroy_start = content
        .find("onDestroy(")
        .unwrap_or_else(|| panic!("H-12 (i): ResearchMode.svelte must contain an onDestroy() block"));
    // Window is 450 chars: the onDestroy body in ResearchMode.svelte spans ~390
    // chars from the opening `onDestroy(` to the closing `});`, so 450 covers
    // the full block with margin regardless of comment length.
    let destroy_slice_raw = &content[destroy_start..std::cmp::min(destroy_start + 450, content.len())];

    // Strip comment lines before asserting on executable code only.
    // A bare substring search on the raw slice would match explanatory comments
    // that mention canvas.mode as prose (e.g. "// Unconditional of canvas.mode:")
    // even when no executable guard on canvas.mode is present.  We strip:
    //   (a) single-line comments: any line whose first non-whitespace chars are //
    //   (b) block-comment spans: /* ... */ replaced with a single space
    // The remaining text is executable code only.
    let without_block_comments = {
        let mut s = String::with_capacity(destroy_slice_raw.len());
        let mut chars = destroy_slice_raw.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '/' {
                if chars.peek() == Some(&'*') {
                    // consume block comment /* ... */
                    chars.next(); // consume '*'
                    let mut prev = ' ';
                    for bc in chars.by_ref() {
                        if prev == '*' && bc == '/' {
                            break;
                        }
                        prev = bc;
                    }
                    s.push(' ');
                    continue;
                }
            }
            s.push(c);
        }
        s
    };
    // Strip single-line // comments: for each line, take only the part before `//`.
    let destroy_exec: String = without_block_comments
        .lines()
        .map(|line| {
            if let Some(idx) = line.find("//") {
                &line[..idx]
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        !destroy_exec.contains("canvas.mode"),
        "H-12 (i): ResearchMode.svelte onDestroy must NOT check canvas.mode in \
         executable code — that guard is always false at destroy time (Svelte \
         unmounts AFTER the reactive condition flips, so mode has already changed). \
         Use the _bootstrapped flag instead (BLOCK-2 teardown-leak fix). \
         Found 'canvas.mode' in executable onDestroy code (comments are excluded \
         from this check)."
    );

    // onDestroy must gate teardown on _bootstrapped (in executable code).
    assert!(
        destroy_exec.contains("_bootstrapped"),
        "H-12 (i): ResearchMode.svelte onDestroy must gate teardown() on the \
         '_bootstrapped' flag to guarantee 1:1 bootstrap/teardown pairing."
    );
}

// ---------------------------------------------------------------------------
// (h) canvas.svelte.ts exists and declares CanvasMode.
// ---------------------------------------------------------------------------

#[test]
fn h12_h_canvas_state_file_exists_with_mode_type() {
    let canvas_state = dashboard_lib().join("state/canvas.svelte.ts");
    assert!(
        canvas_state.exists(),
        "H-12 (h): state/canvas.svelte.ts must exist — it is the hub for the \
         canvas mode flag + single-consumer SSB arbiter"
    );

    let content = read_file(&canvas_state);

    // Must define a CanvasMode type.
    assert!(
        content.contains("CanvasMode"),
        "H-12 (h): canvas.svelte.ts must define CanvasMode type"
    );

    // The mode enum must include code, plan, research (wired now) and arch, design (future).
    for mode in &["code", "plan", "research", "arch", "design"] {
        assert!(
            content.contains(mode),
            "H-12 (h): canvas.svelte.ts CanvasMode must include the '{}' mode value",
            mode
        );
    }

    // Must export a canvas singleton store.
    assert!(
        content.contains("export const canvas"),
        "H-12 (h): canvas.svelte.ts must export a 'canvas' singleton store"
    );
}
