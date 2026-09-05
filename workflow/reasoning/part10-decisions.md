PART10 decisions | session s-30316 | dashboard-v2 wiring sweep | 2026-05-08

Mode: autonomous (user said "don't bother me anymore until you finish")
Audit budget: 1 cycle
Commit budget: NO COMMIT this session

OQ TRIAGE (autonomous decisions per prompt-engineer recommendations)

OQ1 — Slash dispatch strategy
Decision: Option A — frontend prefix detect in commands.submit()
Why: avoids backend round-trip for routing; keeps SLASH_COMMANDS list as canonical UI hint; no schema change needed
Scope: /research /task /explore wired to existing tryInvoke paths; /fix /continue /config /status /ask flagged as TODO with logStub fallback (backend handlers not all present)

OQ2 — /research result UI
Decision: Option A — Tauri 'research-progress' event series rendered in ResearchDrawer
Why: matches existing reindex-progress + d5-progress patterns; ResearchDrawer already has poll-based researchData store (drawers.svelte.ts:310)
Scope: add event listener in researchData (refresh on each progress tick); render incoming nodes in research-tree section
Risk: if backend doesn't emit research-progress events, drawer stays at polling-only behavior — non-regressive

OQ3 — Phase 4B backend
Decision: DEFER (do not implement)
Why: backend ENTIRELY ABSENT (accept_change/decline_change/retry_audit + 4 SsbActionType variants). Implementing requires SSB SCHEMA_VERSION v26->v27 destructive migration. User said no destructive operations without explicit approval (already in CLAUDE.md), and "don't bother me" means I cannot pause to ask.
Action: leave UnwiredNote banner intact at ReviewWindow.svelte:140; document defer in handoff for future session

OQ4 — Settings persistence (~15 stub rows)
Decision: DEFER
Why: each row is a separate backend command (set_execution_mode, set_sidecar_settings × N, etc.). Substantial implementation scope that would dominate this session and prevent OQ1/2/5 work which has higher user-visible payoff.
Action: leave SettingsWindow self-disclosure banner intact at line 364

OQ5 — Reindex listener race fix
Decision: Option A — pre-bootstrap warm-up via ready() Promise
Why: smallest surface, deterministic fix; ProjectSelector awaits reindex.ready() before reindex_now() fire-and-forget
Implementation: reindex.svelte.ts exposes ready: () => Promise<void>; bootstrap caches the listen() Promise. ProjectSelector calls await reindex.ready() before void reindex_now().

WIRING SEQUENCE (in order)

Step 1 — slash-dispatch (OQ1)
Files: src/lib/state/commands.svelte.ts (Edit submit())
Decision: parse leading token; route /research → run_research, /task → submit_task, /explore → run_explore_query, /continue → continue_task. For /fix /config /status /ask without confirmed backends, append a status card and log via existing logStub. Strip the leading slash + command from the streamed payload when falling through.
Expected test outcome: svelte-check 0 errors; npm run build pass; star-shoot/star-state 22/22 pass; existing streamChat path preserved when input has no slash prefix
Adversarial:
- empty trailing arg ("/research" with no query) → show error card, do not invoke
- argument with leading whitespace ("/research   foo  ") → trim before forward
- known prefix mid-string ("hello /research test") → NOT slash; passes through to streamChat unchanged
- streaming card lifecycle: when slash dispatches succeed, no streaming card should be created (no stream_id leak)

Step 2 — research-runtime-feed (OQ2)
Files: src/lib/state/drawers.svelte.ts (createResearch — add listen() in bootstrap, unlisten in teardown)
Decision: listen('research-progress') → merge incoming nodes into researchTree by id; refcount with existing rc to avoid double-listen
Expected test outcome: same as Step 1
Adversarial:
- listener resolves after teardown → late-unlisten guard (mirror reindex.svelte.ts pattern)
- backend emits malformed payload → try/catch around payload application; console.warn fallback
- multiple bootstrap calls → only one listen registration (refcount)

Step 3 — reindex-listener-race (OQ5)
Files: src/lib/state/reindex.svelte.ts (Edit bootstrap to expose ready Promise) + src/lib/screens/ProjectSelector.svelte (await reindex.ready())
Decision: bootstrap caches the listen() Promise as `_readyPromise`; new public `ready(): Promise<void>` resolves when listener registered (or on teardown if never registered). ProjectSelector openProject + openNewProject await reindex.ready() before void reindex_now()
Expected test outcome: same as Step 1; race manually verifiable in Tauri shell (StatusBar shows 'running' on first project open)
Adversarial:
- reindex.ready() called before bootstrap → return resolved Promise (no-op)
- listen() rejection → ready() resolves anyway (degraded mode), ProjectSelector continues to fire reindex_now() since the alternative is hanging the UI
- multiple openProject calls → ready() Promise reused; no double-listen

Step 4 (best-effort within budget) — high-impact LOGSTUB_ONLY rows
Candidates if budget allows:
- monitor-takeover (MonitorWindow.svelte:117): wire to submit_intervention_response (backend exists)
- anti-staleness-runAudit (AntiStalenessDrawer.svelte:39): only logStub; no backend command yet — DEFER
Action: only attempt if Steps 1-3 + audit complete with budget remaining

CONSTRAINTS (this session)
- Edits target .elai_cc/ only
- No new dependencies
- No .unwrap() in production Rust (no Rust edits planned this session — all frontend Svelte/TS)
- Svelte 5 runes only ($state/$derived/$effect/$props)
- 1 audit cycle, no retry on BLOCK
- NO COMMIT

ROLLBACK PLAN
- Each step is a single file (or two for Step 3); revert is `git checkout <path>`.
- No SSB schema bumps. No new Tauri commands. No invoke_handler! changes. No `use tauri::Emitter;` additions (frontend-only changes).
