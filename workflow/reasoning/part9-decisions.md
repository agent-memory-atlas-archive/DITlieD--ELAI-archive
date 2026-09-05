PART9 decisions — dashboard-v2 finishing
session: s-40940
date: 2026-05-08
predecessor: s-20000 (PART8)
handoff spec: workflow/sessions/s-20000/NEXT-SESSION-FINISH-DASHBOARD-V2-PART8.md

WHAT
Finish dashboard-v2 by closing 3 user-reported regressions in priority order A -> B -> C.
A. Item 2: CommandWindow.svelte slash command argument fix.
B. Item 3: ProjectSelector reindex_now wiring + StatusBar reindex-progress feedback + reindex_now backend Tier-1 profile load/persist.
C. Item 1: decompose-converter.ts (NEW) + ActivityWindow live fixture default + ?demo=1 strict gate (mock fixtures dynamic-imported).

USER ANSWERS (verbatim)
Q1 — reindex_now+ProfileGenerator: B (yes, also generate Tier-1 profile)
Q2 — mock-data ?demo=1 gate: B (strict gate, dynamic import)
Q1.1 — API choice: A — load_or_detect_profile (idempotent) "if profile dont need constant updates"
Q1.2 — profile.toml path: ii — grep and use what you find

KEY DECISIONS

API rename:
  Handoff naming "ProfileGenerator" does not exist in codebase.
  Verified APIs (grep):
    elai_knowledge::project_profile::detect_project_profile(&Path) -> Result<ProjectProfile, ProjectProfileError>   [knowledge/src/project_profile.rs:418]
    elai_knowledge::project_profile::serialize_profile(&ProjectProfile) -> Result<String, ProjectProfileError>      [knowledge/src/project_profile.rs:1598]
    elai_knowledge::incremental_profile::load_or_detect_profile(&Path) -> Result<ProjectProfile, ProjectProfileError>  [knowledge/src/incremental_profile.rs:285]
    elai_core::project_profile::ProjectProfile  [core/src/project_profile.rs:25]
  Decision: use load_or_detect_profile (idempotent — loads from cache if schema_version matches, otherwise detect_project_profile). Matches USER Q1.1=A intent.

Profile path:
  Constant PROFILE_FILENAME = "project-profile.toml"   [knowledge/src/incremental_profile.rs:24]
  Resolved as project_dir.join(PROFILE_FILENAME)        [knowledge/src/incremental_profile.rs:286]
  Decision: write at {project_root}/project-profile.toml. Persist on first detect (file missing) so subsequent reindexes hit cache. No path override; trust the constant.

Backend wiring location:
  reindex_now at .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs:2493-2547
  Inside the existing spawn_blocking closure, AFTER svc.run_full_index() returns Ok, run profile load/persist. On Err, skip profile (index failure dominates). Add profile_loaded boolean to "done" event payload.
  elai-knowledge dep already declared at .elai_cc/crates/dashboard-v2/src-tauri/Cargo.toml:39. No new dep required.

Q2=B strict gate implementation:
  ActivityWindow.svelte: remove static ORBIT_FIXTURE_* imports from $lib/api/mock-orbit. Replace with async getMockFixture(key) that dynamic-imports mock-orbit only when ?demo=1. FIXTURES.live = $derived(decompNodeToOrbitFixture(plan.tree, fsm.state)) stays static. Default fixtureKey = 'live'. Cycler button gated behind URLSearchParams demo === '1'. Mock fixtures absent from production bundle when ?demo flag absent.

Decompose converter shape:
  decompNodeToOrbitFixture(tree: DecompNode, fsmState: FsmState): OrbitFixture
  Walk by level enum strictly: 'concept' children -> ConceptStub.phases; 'phase' children -> PhaseStub.steps; 'step' children -> StepStub.baby_steps; 'baby-step' is leaf.
  tree.id === 'empty' -> {task_id:'idle', idea:'No active task', fsm:fsmState, concepts:[]}
  Active id resolution: prefer status==='in_progress' first match; fallback to first child of each level. Default empty arrays for incomplete branches. Defensive log+skip on unexpected level.

Slash command fix:
  CommandWindow.svelte handleInput: if v.startsWith('/') && v.includes(' ') -> commands.closeSlash().
  CommandWindow.svelte handleKeydown 'Enter' branch when palette open: executeSlash only if commands.inputValue.trim() === chosen.command (bare-command equality). Otherwise fall through to submitInput. Preserves bare "/research"+Enter autocomplete; allows "/research find pytorch"+Enter to submit verbatim.

Reindex feedback:
  ProjectSelector openProject + openNewProject: after open_project_from_folder() resolves and BEFORE session.openWorkspace(), fire `void reindex_now().catch(err => console.warn('reindex failed', err))`. UI advances first; indexing runs in workspace.
  StatusBar mounts a listen('reindex-progress', ...) on app start (NOT on workspace mount, to avoid late-mount race losing first event). Stores unlisten fn; calls in $effect cleanup. Renders idle/running/done(counts)/error states inline in .verbose group between project label and ctx gauge. Done state fades after 3s.

EXPECTED TEST OUTCOMES

Build/type checks (auditor scope):
  npx svelte-check: 0 errors. 9 cosmetic CSS unused-selector warns acceptable (pre-existing). If Item-4 done, expect 0 warns too.
  npm run build: vite build success. Bundle size delta: smaller than PART8 baseline because Q2=B strips ORBIT_FIXTURE_* from prod bundle.
  cargo check -p elai-dashboard-v2: green.
  cargo build -p elai-dashboard-v2 --release: success, fresh binary timestamp newer than 2026-05-08 01:23.

Pre-existing tests (must stay green):
  tests/scene/star-shoot.test.ts: pass.
  tests/scene/star-state.test.ts: pass.

Out of scope (deleted V1 crate, do NOT run):
  dashboard_4k3_4k4_4k5_contract.rs
  dashboard_real_executor_wired_contract.rs
  dashboard_sidecar_provider_wired_contract.rs

Manual smoke (USER-driven after binary rebuild):
  Item 2: type "/research find pytorch" + Enter in CommandWindow -> args preserved, sent to commands.submit.
  Item 2: type "/research" + Enter -> autocompletes to "/research ".
  Item 3: pick a project -> StatusBar shows "indexing..." -> "indexed: N files, M nodes" briefly -> fades.
  Item 3: project-profile.toml appears at project root after first reindex; subsequent reindex loads from cache (no re-detect cost).
  Item 1: orbit shows "No active task" idle ring when plan.tree.id==='empty'.
  Item 1: orbit converts live decomp tree once a task is running.
  Item 1: cycler button hidden in production; visible with ?demo=1.

UNKNOWNS / RISKS

R1. spawn_blocking move semantics for project_root: project_root already cloned for IndexingService::new at line 2508 (project_root.clone()). Adding profile load needs another reference. Solution: move project_root into closure (single owner), use it for both indexes_dir and profile path. Ergonomics validated by reading current code shape.

R2. Late-mount race for "reindex-progress" listener: if StatusBar mounts AFTER reindex_now has already emitted "started", we lose that event. Mitigation: mount listener in App.svelte or a top-level component that lives across selector->workspace transition. Verify on EXECUTING which top-level component is canonical.

R3. Q2=B dynamic import vs vite: dynamic import() of $lib/api/mock-orbit must work with vite's code-splitting. Confirm via WebSearch if needed; default expectation is yes (vite supports this since 2.x). Bundle inspection in npm run build output will confirm mock-orbit chunk excluded from main bundle when ?demo absent.

R4. Decomp tree shape drift: handoff says "walk by .level enum" — verify the actual DecompNode shape in dashboard-v2 matches expectation (concept/phase/step/baby-step). Read .elai_cc/crates/dashboard-v2/src/lib/api/types.ts on EXECUTING entry before writing converter.

R5. ProjectProfile.schema_version mismatch on existing files: load_or_detect_profile auto-falls-back to detect on mismatch (verified at incremental_profile.rs:292). Behavior: silent re-detect, stale TOML stays on disk until our write replaces it. We write only when the file was MISSING at call time, NOT when schema_version mismatched. This means stale TOML files persist until the next reindex with a missing-file scenario. Trade-off accepted: matches USER intent "profile dont need constant updates"; manual fix is delete the file.

R6. invoke_handler list at lib.rs:51-190 shows reindex_now exists; PART9 adds zero new Tauri commands. Confirmed.

R7. elai.map is stale for dashboard-v2 paths. For dashboard-v2 symbol lookups in this session, grep src/ directly. Memory-writer scope at session end: regenerate elai.map.

CONSTRAINTS IN FORCE

C1. Project boundary: all writes under .elai_cc/. Workflow tooling (this decision doc, FSM logs) is OK to write outside.
C2. NO COMMITS until USER explicitly authorizes. PART8 carry-forward.
C3. Zero new dependencies. elai-knowledge already declared.
C4. No .unwrap()/.expect()/panic!()/todo!()/unimplemented!() in production Rust. Use Result/Option propagation.
C5. No silent error swallowing. void reindex_now() must have .catch.
C6. Memory-bank writes via memory-writer agent only.
C7. One audit cycle.
C8. GRC budget 8/1500 on topic part9-dashboard-finish (already set).
C9. mcp_session_claim must succeed before first Edit/Write.
C10. FSM transitions: PLANNING -> TESTING_FIRST -> EXECUTING -> AUDITING -> MEMORIZING -> COMPLETE -> IDLE.
C11. Sidecar health check skipped (frontend-only test surface; bug is in pre-submit input wiring).
C12. Anti-Stale-Training: Rust uses anyhow/thiserror for errors, blake3 (not blake2/sha2) — none expected to be touched in PART9.

ADVERSARIAL TEST LIST (TESTING_FIRST entry — to be validated by auditor)

For Item 2 (CommandWindow):
  T2.1 Type "/" -> palette opens with command list visible.
  T2.2 Type "/research" exactly + Enter -> input becomes "/research " (autocomplete preserved).
  T2.3 Type "/research find pytorch" + Enter -> commands.submit fires with full string; palette closed; executeSlash NOT triggered.
  T2.4 Type "/research " (with trailing space) + Enter -> commands.submit fires with the input passed through .trim() ("/research" without trailing space); executeSlash NOT triggered. submitInput trims; this is acceptable since the autocomplete branch is gated by exact bare-command equality and the trimmed form matches it cosmetically. If downstream parsing requires a trailing-space sentinel, revisit the submitInput contract separately.
  T2.5 Type "/researc" (typo) + Enter -> palette filter empty; existing behavior preserved (no crash, no spurious submit unless user-intent allows).
  T2.6 Type "/r" then space -> palette closes (handleInput catches first space).

For Item 3 (ProjectSelector + StatusBar + backend):
  T3.1 Pick existing project: open_project_from_folder resolves -> reindex_now fires fire-and-forget -> session.openWorkspace advances UI -> StatusBar shows "started".
  T3.2 Reindex completes successfully -> StatusBar shows counts (dkg_files, dkg_nodes, etc.) -> fades after 3s.
  T3.3 Reindex fails -> StatusBar shows error variant; user-visible non-fatal warning.
  T3.4 First-ever reindex on a project -> project-profile.toml appears at project root after run_full_index Ok branch.
  T3.5 Second reindex on same project -> profile loads from cache; no rewrite of project-profile.toml.
  T3.6 Schema-version mismatch on existing project-profile.toml -> load_or_detect_profile falls back to detect; profile_loaded=true emitted; existing TOML left in place (R5 trade-off).
  T3.7 Listener cleanup: switch project mid-session -> no duplicate "reindex-progress" listeners; StatusBar still works.
  T3.8 Concurrent double-click on a project tile -> reindex_now invoked twice; backend behavior is implementation-defined but UI does not crash; second "started" event renders coherently.

For Item 1 (decompose-converter + ActivityWindow):
  T1.1 plan.tree.id === 'empty' -> orbit renders idle ring with "No active task" label; concepts=[].
  T1.2 plan.tree has 1 concept with no phases -> ConceptStub.phases=[]; OrbitCanvas renders concept-only ring.
  T1.3 plan.tree has concept->phase->step->baby-step chain -> all 4 levels rendered with correct grouping.
  T1.4 Defensive: plan.tree contains a node with unexpected level value -> console.warn fires; node skipped; orbit does NOT crash.
  T1.5 Cycler button hidden by default -> not visible without ?demo=1.
  T1.6 Cycler button visible with ?demo=1 -> all mock fixtures cycleable (small/constellation/research/map).
  T1.7 Production bundle (npm run build, no ?demo flag) -> mock-orbit chunk NOT in main bundle (verified via build output or inspection).
  T1.8 active_concept_id resolution: in-progress concept wins over first-child fallback.
  T1.9 OrbitCanvas access pattern: fixture.concepts[0]?.id at OrbitCanvas:128 and fixture.concepts.length at :140 do not throw on empty concepts array.

EXIT CRITERIA (full session)

E1. All 3 items implemented in priority order A -> B -> C.
E2. Adversarial tests T2.*, T3.*, T1.* pass on USER manual smoke.
E3. svelte-check 0 errors. npm run build success. cargo check -p elai-dashboard-v2 green. cargo build -p elai-dashboard-v2 --release fresh binary.
E4. Pre-existing scene tests still pass.
E5. Auditor returns PASS in 1 cycle.
E6. memory-writer agent updates activeContext.md / changelog.md / progress.md / projectIndex.md (and regenerates elai.map).
E7. FSM at IDLE before session end (Stop hook enforces).
E8. mcp_session_release called after MEMORIZING.
E9. Zero git commits unless USER explicitly authorizes.
