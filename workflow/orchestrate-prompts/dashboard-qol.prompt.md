AUTONOMOUS ORCHESTRATE RUN — ELAI Dashboard QOL / UX hardening (frontend-only, phase-F-deconflicted)

Paste this entire message as the first message of a fresh `claude` session whose CWD is /workspace/ELAI. The session's native CLAUDE.md, .claude/rules, .claude/agents, .claude/skills and .mcp.json load automatically.

=== WHAT YOU ARE ===
You are an autonomous EXECUTOR running the orchestrate skill in SAME-PROJECT mode (target == host == ELAI). You adapt .claude/skills/orchestrate (SKILL.md + protocol.md per-sub-phase loop) with the override policies below. You do a full-blown dashboard quality-of-life / UX hardening sweep of the Svelte 5 + Tauri dashboard at .elai_cc/crates/dashboard-v2/. Optimize for THOROUGHNESS and coverage, not speed — sweep the entire frontend surface. This is a deliberate deep-improvement run.

Load these skills before starting: orchestrate, planning-workflow, dashboard-handoff. Do NOT load sidecar-protocol (no model inference in this run).

=== DECONFLICTION (LOAD-BEARING — READ FIRST) ===
Another live session, s-60824 ("F-plan continuation orchestrate"), owns ~6000 lines of uncommitted phase-F work on this same tree. You must NOT collide with it.

D1. FIRST ACTION, before any edit: run `git status --porcelain` and `git diff --stat HEAD` from /workspace/ELAI. Build your HOT map PURELY from the live output before each edit — do NOT anchor on a static list and do NOT assume the hot set is just the api layer. The dirty set already includes layout/chrome/drawer/window .svelte files that OVERLAP your QOL scope (verified currently dirty at this writing, re-confirm live): src/lib/components/ToolGateInspector.svelte, src/lib/drawers/DrawerHost.svelte, src/lib/layout/Drawer.svelte, src/lib/layout/Rail.svelte, src/lib/windows/AuditTimelineWindow.svelte — plus the api layer src-tauri/src/commands*.rs and src/lib/api/{types,stub}.ts. When a QOL fix lands on any hot file, DEFER it per T5 and document — do not edit it.
D2. THE HOT MAP = git status --porcelain MODIFIED (M / MM) UNION UNTRACKED (??). BOTH are owned by s-60824 and OFF-LIMITS: do not edit, overwrite, or CREATE a file that collides with any path git status reports. You may ONLY edit a frontend file that is ABSENT from git status --porcelain entirely (clean). Verified untracked (??) panels to protect (re-confirm live): src/lib/drawers/DepsSecurityPanel.svelte, src/lib/drawers/SecurityMonotonicityDrawer.svelte, src/lib/drawers/SecurityRulesPanel.svelte, src/lib/drawers/VibeGateDrawer.svelte. If a fix requires touching any hot/untracked file, DEFER + record in the handoff follow-up list.
D3. FRONTEND-ONLY. You edit ONLY CLEAN .elai_cc/crates/dashboard-v2/src/**/*.{svelte,ts,css}. The ENTIRE src/lib/api/ directory is READ-ONLY (the IPC/contract layer — verified contents: stub.ts, types.ts, empire.ts, mock-data.ts, mock-orbit.ts). You do NOT edit any .rs file. You do NOT add or change Tauri commands (the hot src-tauri layer). If a component needs a backend command that does not exist, DOCUMENT it as a follow-up; do not cross into Rust.
D4. NEVER run `git commit`, `git add`, `git stash`, `git checkout --`, `git reset`, or `git restore`. The tree holds s-60824's uncommitted work; any of these clobbers it and violates one-changeset-per-task. This OVERRIDES orchestrate POLICY 4 + protocol step 11. You accumulate edits and hand off uncommitted.
D5. SESSION SOVEREIGNTY: call mcp_session_register for YOUR session, then mcp_session_list to read s-60824's claimed/locked files. Claim only the COLD frontend files you intend to edit. Never call mcp_fsm_* or mcp_session_release against any session id but your own.

=== SCOPE (the QOL surface) ===
The user reports the dashboard needs broad QOL work. Cover at minimum:
S1. Responsive / layout breakage: text crumples and overflows when the window is minimized / narrowed. Audit every view, drawer, panel, and the window chrome (WindowHost, orbit, ribbon, monitor windows) for fixed widths, missing min-width/overflow handling, non-wrapping flex/grid, and text that clips or overlaps at small sizes. Add responsive rules (clamp/min-max, overflow, truncation with title tooltips, wrapping).
S2. "Everything talks to something": audit every interactive control (buttons, toggles, inputs, menu items, tabs) for dead handlers — controls bound to no-op, to a missing store action, or to a TODO. For each: wire it to the correct existing store/binding, OR mark it disabled with a reason, OR (if it needs a NEW backend command) log a follow-up. No silent dead buttons.
S3. Error / loading / empty states: every panel that consumes data (a store, an IPC call via the existing api layer, a subscription) must render explicit loading, error, and empty states instead of blank or a thrown render. Add error boundaries / try-catch around IPC consumption where missing.
S4. Debug surface: ensure existing debug/diagnostic panels render and are reachable; surface IPC/store errors to a visible debug log rather than swallowing. Do NOT add new backend debug commands.
S5. General polish: consistent spacing/tokens, focus states, keyboard nav where trivial, no console errors on mount.

=== OVERRIDE POLICIES (precedence over native skill defaults) ===
P1. PARALLEL CAP 3 per Agent-tool batch.
P2. SKIP prompt-engineer / prompt-transform (frontend brief is sufficient).
P3. ONE auditor cycle per sub-phase; on BLOCK apply bubble-up retry budget (3 retries at the failing-task grain), no AUDITING<->EXECUTING ping-pong.
P4. NO COMMIT (see D4). Accumulate; hand off uncommitted.
P5. AUTONOMOUS = true. Do not ask the user anything. Choose the best grounded non-lazy option and continue. Only the 4 orchestrate tripwires interrupt (T1 irreversible op, T2 50-event bubble cap, T3 dep-wait > 4h — N/A here, T4 sweep/stall after 3 retries). ADDED tripwire T5: if a required fix lands on a HOT file, do NOT surface — defer + document and move on (autonomous).
   DEFER-FLOOR: if more than 50% of authored sub-phases resolve to DEFER (hot-file collision), STOP, write SESSION_HANDOFF noting the QOL surface is dominated by s-60824-owned files, and do NOT report the run as a successful sweep. Feed each DEFER into the step 8.9 stall detector as a no-progress sub-phase, so a defer-heavy run is classified as STALL, not success.
P6. SCOPED VERIFICATION ONLY (overrides orchestrate steps 9-10 workspace gates). The Rust workspace does NOT build right now (s-60824 contamination), so you NEVER run cargo build/test --workspace or `tauri:dev` (it compiles src-tauri). There is NO vitest/unit-test layer (verified: dashboard-v2/package.json has no test script and no vitest dep) — static analysis is the ONLY gate. Verify FRONTEND-ONLY:
    Push-Location /workspace/ELAI\.elai_cc\crates\dashboard-v2
    npx svelte-check --tsconfig ./tsconfig.json    # primary gate
    npx tsc --noEmit                                # type gate   (equivalently: npm run check)
    Pop-Location
   Record a BASELINE svelte-check error count BEFORE editing; the gate is "no NEW errors introduced" (the tree may already have pre-existing errors from s-60824). Optional visual confirm: `npm run dev` (Vite browser mode against api/stub.ts) ONLY if stub.ts currently compiles; if it is mid-edit/broken, skip and rely on static analysis. Use the dashboard-handoff skill + dashboard-smoker agent for the svelte-check pass.
   SKIP all cross-stack bindings-drift gates (XS1-XS5 in stack-adapters/cross-stack-rust-ts.md): they run cargo emit + git diff HEAD~N, which is invalid under D4 NO-COMMIT (HEAD never advances) and this frontend-only no-cargo run. The src/lib/api contract layer is read-only, so there is no bindings work to verify.
P7. NO SIDECAR. Never boot the remote sidecar; no real-model test (orchestrate step 10 is replaced by P6).

=== FLOW ===
Phase 0 — DISCOVERY + PLAN AUTHORING (no existing plan file exists for this work):
  0.1 mcp_fsm_get_state; set mcp_think budget complex (8 calls / 1500 chars) per topic since prompt-engineer is skipped.
  0.2 Dispatch in ONE batch (cap 3): context-compiler scoped to .elai_cc/crates/dashboard-v2/src (frontend only) + integration-explorer-elai scoped to the dashboard component/store/api-consumption graph. Canonical ELAI dispatch: subagent_type=general-purpose, prompt "Read /workspace/ELAI\.claude\agents\<name>.md and follow it strictly. <args>".
  0.3 From their returns, author a scoped plan at workflow/sessions/<your_sid>/dashboard-qol-plan.md with Phase N > sub-Phase > Task structure, a WIRING_CONTRACT per sub-phase (frontend observable_effect: e.g. "narrow window to 480px -> no clipped text in <view>"), DETERMINISTIC_CHECK (svelte-check error delta == 0; grep for dead handlers resolved), and ACCEPTANCE. Group sub-phases by surface area (one per view/drawer cluster) so the run is many small sub-phases (good coverage + good burn). Mark every sub-phase frontend-only.
Phase 1..N — ORCHESTRATE LOOP (protocol.md step 8 per sub-phase):
  For each sub-phase: PARALLEL BATCH (context-compiler + integration-explorer-elai scoped to that surface) -> SKIP prompt-engineer -> EXECUTING (implement the sub-phase's QOL fixes, frontend-only, re-checking git status so you never edit a newly-dirtied file) -> AUDITING (auditor, 1 cycle; tell it the test policy is svelte-check/tsc only, no cargo) -> MEMORIZING (memory-writer scoped to this sub-phase) -> per-sub-phase frontend verification (P6) -> advance.
FINAL:
  Full frontend verification (svelte-check + tsc over the whole dashboard); confirm zero NEW errors vs baseline. NO commit (D4). Write workflow/sessions/<your_sid>/orchestrate-final.md + a SESSION_HANDOFF.md listing: every file changed (uncommitted), every deferred hot-file/backend follow-up, the baseline-vs-final svelte-check delta. End in FSM IDLE (MEMORIZING -> COMPLETE -> IDLE).

=== GROUNDING + HARD RULES (native, restated) ===
Grounded-by-default: verify Svelte 5 / Tauri 2 API behavior against current docs before using; no training-memory API recall. No force-unwrap / no silent catch / zero-lazy-code (every stub registered with a babystep id). Token discipline: delegate reads to agents; don't read whole files into your own context. Foreground all builds/tests (no run_in_background). Anti-spiral auto-fires; observe it.

START: emit your session id, mcp_fsm_get_state, register session, run the D1 git status, then begin Phase 0.
