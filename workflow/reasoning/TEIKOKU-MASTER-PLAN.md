# TEIKOKU — Central Workflow Master Plan

name: TEIKOKU (帝国, "empire" in Japanese) — RESOLVED, used everywhere incl the dashboard wordmark + dirs
status: v1 draft, 2026-06-19, authored by the planning session from a 6-round grill + the harness cross-check audit
purpose: ONE canonical workflow — a "small ELAI" general harness — built on a frozen local copy of the Claude Code CLI, that merges the ELAI + liteGAME harnesses, runs interactive AND fully-autonomous orchestration across any registered project, and is monitored from one Tauri dashboard.
grounding: this plan's design decisions are the locked answers in Section 0 (from the grill) + workflow/reasoning/central-workflow-harness-audit.md (the byte-level merge audit). Do not re-litigate Section 0; it is settled with the owner.
how to run: hand this file to the ELAI `orchestrate` skill in a fresh Opus session. See Appendix A (boot prompt) and Section 5 (build method).

================================================================================
0. LOCKED DECISIONS (settled with the owner; ground truth, do not re-open)
================================================================================

D-01  ONE canonical workflow, not three. Merge of the ELAI harness + liteGAME harness into a shared CORE + opt-in CAPABILITY PACKS. Liberty is a CONSUMER project, never part of the engine.
D-02  Architecture = core + tagged packs. One core maintained once; each project activates only the packs it needs (rust-pack, unity-pack, media-pack, ...). Existing project-specifics become packs; nothing is force-migrated between projects.
D-03  Base runtime = a FROZEN LOCAL COPY of the Claude Code CLI source (NOT a fork, never updated). We build ON TOP of it as a base layer, exactly like the Devin-CLI work but better. It keeps its Anthropic-plan (account/OAuth) login AND gains free-API providers (litellm-style, the way Liberty does it).
D-04  TWO orchestration systems, ONE engine, two front-ends:
        Front-end A (interactive chat): one chat fronts N headless sessions; Opus main model; always-grills; auto-migrates at a context threshold; stays one chat for the owner.
        Front-end B (fully autonomous): headless, 24/7, no human; runs task batches at max output.
      Same task-graph / scheduler / gate / worker / mailbox machinery underneath. The difference is only whether a human is attached + whether it pauses.
D-05  Worker pools = co-equal pluggable: Devin free models (swe-1.6/glm-5.2/kimi-k2.7), free-API models (Liberty backbone, generalized), Codex GPT-5.5 (WSL+Docker). Autonomous lane always uses the MAX free models that can run together within measured rate/concurrency limits. Owner picks task + which system; it runs at max output.
D-06  Done-authority = DETERMINISTIC, per-project, flexible. Each project DEFINES "done" at registration (its pack gates). ELAI done = wiring/tests pass + everything talks to the dashboard + a dashboard-polish pass. Worker NEVER self-certifies; gates are off-agent and frozen (keep ELAI's principle). The gate is IMPROVED not rebuilt — current per-plan wiring enforcement is too weak (built for small-context Sonnet); with Opus-1M executors it must be made much stricter.
D-07  Inter-session = HUB. Shared append-only event log + upgraded RAG mailbox; spawn via orchestrator/queue; NO ad-hoc peer DMs. CRITICAL CLI MODIFICATION: a waiting session SLEEPS and is WOKEN by an event (e.g. "file free", "dep done"), never poll-spams "wait 5 more minutes". This is only possible in the CLI local copy, not the VS Code extension.
D-08  Memory = generalized SSB. Per-project append-only event log + a RAG index over it (grep + semantic, like codegraph for prose). A global cross-project index powers session comms + the dashboard. The markdown memory-bank (changelog/activeContext/progress/...) is RETIRED/migrated. codegraph keeps owning code structure; this owns decisions/state/prose. The memory-writer ROLE is REWORKED: it AUTO-INDEXES handoff files (workflow/handoff/*) as the changelog/memory into the new RAG — handoffs ARE the canonical memory unit (grep+semantic), no manual memory-bank authoring. (Applies to TEIKOKU's runtime; the build of TEIKOKU itself also uses the handoff-as-memory model, no memory-writer agent — C-02.)
D-09  Auto-migration fidelity = snapshot + RAG fallback. At threshold (~500k or % of window, configurable): serialize a STRUCTURED state snapshot the successor loads verbatim (objective, decision ledger, open threads/TODO, active skills+rules+toggles, file locks, RAG pointers) AND keep the full transcript in RAG for deep recall. The "migrating" indicator shows in the same chat; the prose summary is secondary. The CC context counter is made more accurate as part of this.
D-10  Always-grill, full depth. Every non-trivial interactive task auto-runs a full-depth grill (decision extraction via the question UI) BEFORE any plan/action; trivial / `!`-prefixed messages skip. Feeds the idea->plan pipeline. (This IS ELAI's prompt-transform gate, made interactive and mandatory.)
D-11  Plan format = canonical schema + ELAI adapter, with idea->plan SYNERGY. New projects are born in the canonical schema THROUGH the same (reworked) idea->plan pipeline (idea-intake/decomposition) so every project shares one baseline style. ELAI's legacy corpus (phase 0-4 + plans B..K) is WRAPPED via a generalized backlog_import.py adapter, not rewritten.
D-12  WSL move. Windows = consumer usage; WSL = all code + the harness. A DEDICATED WSL ext4 workspace, SEPARATE from Codex's /root/.conductor-codex (not safe to share). Native tools stay on Windows: Devin .exe, gpt-bridge headed Chrome, RunPod/pod-lifecycle, and Unity Editor. Cross-boundary via localhost forwarding. Projects are COPIED not moved, skipping heavy artifacts (target/, node_modules/, Library/, build caches). All paths in rules/skills/tools get swept to WSL paths.
D-13  Unity = native Windows + bridge. Unity Editor stays native on Windows (GPU, supported); the game's code+agents run in WSL and drive the Editor over localhost via the unityMCP bridge / a tunnel. (Decided; no Unity-in-WSL probe needed.)
D-14  Build strategy = thin vertical slice first, WITH UI from the start. The whole loop works minimally on ONE project (ELAI) before widening; every phase ships its own dashboard surface (no UI-last). GPU-dependent autonomous tasks first CHECK RunPod/local-GPU availability -> defer the plan or escalate to a cloud model; they never "complete" the impossible.
D-15  Build mechanism = existing ELAI `orchestrate` + Opus-only agents, on a fresh WSL COPY. The planning session authors this plan + the boot prompt; the orchestration model feeds Opus agents ALL needed context and lets each do a big chunk (~500k tokens/sub-phase). The live F: root + VS Code stay untouched until cutover.
D-16  Concurrency control = hybrid. Owner sets pool + model choices + concurrency cap (toggles); an auto-scheduler dispatches within that envelope, rate-limit-aware (back-off/cooldown/re-route). Build on the conductor's existing budgets/lease/cooldown logic. The autonomous lane defaults to MAX free concurrency (measured).
D-17  Dashboard = standalone Tauri v2 + Svelte 5 desktop app, shaped like Codex (project sidebar, per-project sessions; new-session = pick Mode [Interactive +/- orchestration toggle | Autonomous, always orchestrated by the separate managing system] + worker source [Anthropic · Devin · Codex · Free APIs] + model + skills + objective). Chat composer modeled on the Claude Code CLI composer (a "+" upload/add-context/browse-web menu, mic + recording state, a "..." model/effort/thinking/account menu, a Bypass-permissions toggle, a queue field, send/stop) with the "/" menu REPURPOSED to control-plane commands (/orchestrate /spawn /worker /model /grill /handoff /skills /gate /defer /escalate /project) and an ALWAYS-VISIBLE token meter (ctx used + session total, mono) from the improved CC token-reader. Design language = workflow/design/DESIGN-TEIKOKU.md (the SIGNAL direction, v2 LOCKED: near-black #0a0d0c + ONE dark-teal accent #14a78f + high contrast; full anti-slop ban list; square status marks not circles/checks; "?" click-popover for detail). Visual ground-truth = workflow/design/mockup/teikoku-signal.html. CONSISTENCY (owner-flagged): the mockup is a LOOK reference and is internally INCONSISTENT — implementation derives ONE canonical token set + a shared component index FIRST and composes every screen from it; a per-screen-copied value or a re-implemented component is a bug. The old workflow/design/DESIGN.md is the ELAI language, kept for comparison only, NOT authoritative for TEIKOKU.

================================================================================
1. ARCHITECTURE OVERVIEW
================================================================================

  +--------------------------------------------------------------------------+
  | TAURI DASHBOARD (Svelte 5, DESIGN-TEIKOKU.md / SIGNAL)  [Windows client]   |
  |  sidebar: projects -> sessions | new-session toggles | monitor | chat    |
  +--------------------------------------------------------------------------+
        | local API (localhost, crosses WSL boundary)
        v
  +--------------------------------------------------------------------------+
  | TEIKOKU RUNTIME  [WSL ext4, ~/teikoku]   (frozen CC local copy + our layer)     |
  |                                                                          |
  |  FRONT-END A (chat)            FRONT-END B (headless autonomous)          |
  |   one chat = N sessions         24/7 task-batch runner                    |
  |   always-grill, auto-migrate    max free concurrency                     |
  |        \                          /                                       |
  |         v                        v                                       |
  |   ============ ORCHESTRATION CORE (generalized from conductor/) =========  |
  |   task graph | scheduler (rate/concurrency) | leases | integrator |       |
  |   gate runner (off-agent, frozen, per-pack) | event-driven session wake   |
  |        |                 |                 |                |             |
  |        v                 v                 v                v             |
  |   CONTROL DB        EVENT LOG + RAG     CORE+PACKS       WORKER POOLS      |
  |   (SQLite WAL)      (generalized SSB,   harness          Devin / free-API  |
  |   task/lease/       grep+semantic       (shared core +   / Codex(WSL+      |
  |   verdict           index, per-proj     rust/unity/      Docker) +         |
  |                     + global)           media packs)     Anthropic-plan    |
  +--------------------------------------------------------------------------+
        | localhost bridges (Windows-native services)
        v
   Unity Editor (native) · Devin.exe · gpt-bridge Chrome · RunPod

Project registry (projects.toml, WSL): each project -> path, platform, active packs, done-definition, allowed lanes (e.g. game = claude-lane ONLY, NDA; ELAI = all lanes).

================================================================================
2. NON-GOALS (scope guard — keep this a small ELAI, NOT literal ELAI)
================================================================================

N-01  No sidecar / local-model / quant / training work. TEIKOKU ORCHESTRATES models; it does not build them. The ELAI sidecar is untouched.
N-02  No gate rebuild from scratch — IMPROVE + wrap the existing frozen checkers (production_caller_check / python_wire_check / maintainability / wiring) as the rust-pack, made stricter for Opus-1M (D-06).
N-03  Lean dashboard v1: sidebar + sessions + chat + monitoring + the per-phase surfaces only. No canvas/3D/graph-viz, no visual-ui-loop spectacle beyond the standard pass.
N-04  WSL day-1 scope = build host + ELAI (copied). The game and other projects migrate later; the game may stay Windows-native (D-13).
N-05  No new ML research, no paper-to-plan campaigns, no new product features inside ELAI/the game. TEIKOKU is the harness; finishing ELAI is its first JOB, run THROUGH TEIKOKU, not part of building TEIKOKU.
N-06  Do not invent brand/UI beyond DESIGN-TEIKOKU.md (SIGNAL) + the click-`?` affordance. Build to the locked language + the teikoku-signal.html reference; do not redesign it. (The look is settled; consistency is the job, not novelty.)

================================================================================
3. CROSS-CUTTING RULES (apply to EVERY phase)
================================================================================

C-01  UI every phase. Each phase ships its dashboard surface; no phase is "done" without its UI slice passing the design pass against DESIGN-TEIKOKU.md (no slop tells; high contrast) AND a cross-screen CONSISTENCY check (same tokens + shared components, zero per-screen drift). Phase 0 stands up the shell + the canonical token set + the shared component index so this is possible from the start.
C-02  Opus-only agents, SIMPLE CYCLE. The per-sub-phase cycle is context-compiler -> executor -> auditor ONLY. NO integration-explorer, NO memory-writer agent for this build (owner override — keep it simple/efficient). MAX 2 agents in parallel. Every agent is Opus (~500k tokens/sub-phase); the orchestrator feeds each the full context listed under that phase's CONTEXT TO FEED. Progress + memory = the handoff file (teikoku-build.md), which the new RAG auto-indexes as the changelog (D-08) — there is no separate memory-writer step.
C-03  Build target is the WSL COPY (~/teikoku + copied projects), never the live F: root. Copied not moved; skip heavy artifacts.
C-04  Worker-never-self-certifies; gates off-agent + frozen; missing evidence = FAIL (inherit ELAI doctrine, made stricter).
C-05  Grounded-by-default + anti-slopsquatting still apply: verify every dependency/version against its registry before use; no training-memory API claims.
C-06  One change set per sub-phase; the auditor (Opus) is the sole gate runner; memory via the new SSB writer, not hand-edited.
C-07  Palette/voice: workflow/design/DESIGN-TEIKOKU.md (SIGNAL, v2) is law; workflow/design/mockup/teikoku-signal.html is the visual reference. No blue/red/orange/amber/purple; ONE dark-teal accent on near-black, high contrast; square status marks (no circles/checks); fail = light-inversion chip, never red. Very low text; depth behind a click-`?`. The mockup is INCONSISTENT — enforce one token set + shared components, never copy it screen-by-screen.

================================================================================
4. THE PHASES
================================================================================
(Each numbered sub-phase is the ~500k-token orchestrate unit. PHASE 0 must complete first; later phases have the listed deps. Within a phase, sub-phases may parallelize where write-sets are disjoint.)

--------------------------------------------------------------------------------
PHASE 0 — BOOTSTRAP & RISK-BURN  (the thin vertical slice's foundation; riskiest integrations first)
--------------------------------------------------------------------------------
Objective: prove every load-bearing integration on day one, on a WSL copy, ending with ONE end-to-end slice + a minimal dashboard tile.

0.1  WSL workspace + project copy.
     - Create the dedicated WSL ext4 workspace (e.g. ~/teikoku), SEPARATE from /root/.conductor-codex.
     - Copy the merged-harness-to-be source + ELAI (.elai_cc CODE only; skip target/, node_modules/, .codegraph/ heavy caches). Record a copy manifest (what was skipped).
     - Establish the localhost-bridge conventions doc (which ports cross to Windows-native services).
     done: workspace exists; ELAI code present minus heavy artifacts; bridge conventions written.

0.2  CC local-copy + Anthropic-account-auth probe.  [THE FIRST PROBE, per owner]
     - Stand up the frozen local copy of the Claude Code CLI in WSL.
     - VERIFY it authenticates via the Anthropic PLAN (account/OAuth) login, not an API key; confirm Opus runs a real turn.
     - Document exactly HOW CC connects with the plan account (the auth flow) — this is the thing we must preserve when grafting free APIs.
     done: Opus turn succeeds via plan-auth from the WSL CC copy; auth flow documented.

0.3  Free-API provider graft (litellm-style).
     - Study Liberty's free-API backbone (how it fronts free APIs/models) + the CC copy's provider/auth layer.
     - Graft a free-API provider path into the CC copy so a session can run on free models WHILE plan-OAuth stays the path for Opus. Per-session pool selection.
     - Probe: one Opus turn (plan-auth) + one free-API turn, same CC copy.
     done: both pools work from one CC copy; plan-auth not broken.

0.4  Worker rate/concurrency probe.
     - Empirically measure: Devin free parallel ceiling, Codex concurrency + rate limits, free-API limits. (Owner flagged this must be measured.)
     - Emit a measured workers.toml envelope (per-pool max concurrency + cooldown).
     done: workers.toml with measured numbers; a 2-worker concurrent run observed without throttle failure.

0.5  Vertical-slice loop + minimal dashboard shell.
     - Wire the smallest end-to-end loop on ELAI: dashboard new-session -> one chat turn -> trigger ONE autonomous task -> one off-agent gate -> record event -> dashboard tile shows it. Reuse conductor/ primitives.
     - Stand up the Tauri shell from DESIGN-TEIKOKU.md (SIGNAL), DESIGN-SYSTEM-FIRST: derive the canonical token file (the §3/§4 values) + the atomic shared components ONCE (Button, SessionRow, StatusMark[square], Tile, Composer [the CC-style + / "/" / "..." menus + always-on token meter], Menu, Popover, Toggle, WorkerSelect) + a component index — this is the pass that FIXES the mockup's inconsistency — THEN compose the shell (three-zone topbar, sidebar, ONE live tile). Visual ref: workflow/design/mockup/teikoku-signal.html.
     done: the slice runs end-to-end and is visible in the shell; the token set + component index exist and the shell composes them (not per-screen copies); design pass clean (no banned tells, high contrast, two surfaces share identical tokens/components).

CONTEXT TO FEED (phase 0): this plan §0/§1/§3; WORKFLOW-OVERVIEW.md; AUTONOMOUS-FINISH-SYSTEM-2026-06-18.md; conductor/ (worker_adapter.py, gate_runner.py, conductor_loop.py, budgets.py, dashboard_server.py); bootstrap-conductor-codex.md + conductor-build.md (free-model + devin/codex facts); Liberty's free-API backbone source; the CC local-copy source (auth/provider modules); DESIGN-TEIKOKU.md + workflow/design/mockup/teikoku-signal.html.
RISKS: plan-OAuth not surviving in WSL; free-API graft breaking auth; Devin .exe cross-boundary invocation; WSLg/localhost forwarding quirks.

--------------------------------------------------------------------------------
PHASE 1 — MERGED CORE HARNESS (core + packs)   [dep: P0]
--------------------------------------------------------------------------------
Objective: one shared core + clean packs, per the audit, with WSL paths.

1.1  Core+pack layout + project registry.
     - Establish the core dir (shared skills/agents/rules/MCPs) + packs/{rust,unity,media}.
     - projects.toml: per-project path, platform, active packs, done-definition, allowed lanes.
     done: core loads; ELAI->rust-pack, game->unity+media resolve.
1.2  Execute the merge (audit §1-§5,§7).
     - Adopt liteGAME's full output-formatting (ELAI's is a 7-line stub the orchestrators cite); collapse generic vs -elai agent dups (scope-planner/integration-explorer/synthesizer) to one parameterized agent each; resolve all §7 conflicts toward the recommended resolution.
     done: one set of merged core skills/agents/rules; no duplicate lineages.
1.3  Prune the cut list (audit §6): retire the cut skills/agents + the 5 inert AE/UE MCPs.
     done: cut items removed; nothing references them.
1.4  Path sweep -> WSL.
     - Rewrite every Windows path in rules/skills/tools/.mcp.json/hooks to WSL paths; keep Windows-native tool paths (Devin/gpt-bridge/RunPod/Unity) as cross-boundary endpoints.
     done: a grep finds no stale F:/D: code paths except the intentional Windows-native endpoints; harness loads in WSL.
1.5  Dashboard surface: harness/pack view (which packs active per project; merge/cut status).
     done: surface renders + design pass.

CONTEXT TO FEED (phase 1): this plan §0,§2; central-workflow-harness-audit.md (FULL); both .claude trees; both .mcp.json; conditional-routing.md.
RISKS: hook path breakage; MCP launch configs; the canvas per-root anchor; gpt-bridge shared-install path.

--------------------------------------------------------------------------------
PHASE 2 — MEMORY: generalized SSB + RAG index   [dep: P0; parallel-ok with P1]
--------------------------------------------------------------------------------
Objective: retire markdown memory-bank; events + index are the memory.

2.1  Per-project append-only event log (generalize ELAI SSB to any project: decisions/changes/state/session-messages).
2.2  RAG indexer over the log: grep + semantic, exposed as a tool the LLM queries like codegraph.
2.3  Global cross-project index (feeds session comms + the dashboard).
2.4  Rework the memory-writer ROLE: it AUTO-INDEXES handoff files (workflow/handoff/*) as the changelog/memory into the new RAG (grep+semantic) — handoffs ARE the memory unit, no manual memory-bank authoring. Retire/auto-migrate the markdown memory-bank; update memorizing-and-audit + output-formatting refs (audit flagged ~5 core artifacts touched).
2.5  Dashboard surface: memory/timeline view (per project + global), queryable.
done: events logged+indexed+queried end-to-end; memory-writer writes events; markdown retired; surface design pass.

CONTEXT TO FEED (phase 2): this plan §0 (D-08); the local-rag server (C:\Users\user\.claude\rag\server.py) mcp_memorize/mcp_session_*/mcp_search_context; ELAI SSB design; codegraph integration; memory-writer.md.
RISKS: RAG reranker Windows-crash history (now WSL — re-validate); migration data loss from markdown.

--------------------------------------------------------------------------------
PHASE 3 — ORCHESTRATION CORE (one engine)   [dep: P1, P2]
--------------------------------------------------------------------------------
Objective: the single generalized engine that both front-ends share.

3.1  Generalize conductor/ into the core engine (task graph, scheduler, leases, integrator, gate runner); strip ELAI-hardcoding; pack-aware.
3.2  Canonical plan schema + idea->plan rework + ELAI adapter.
     - Define the machine-readable project+plan schema (state/deps/scope/done-def/packs).
     - Rework idea-intake/decomposition so new projects emit the schema with one baseline style (synergy, D-11).
     - Generalize backlog_import.py into the ELAI adapter (wrap the 421-file corpus into the schema).
3.3  Improved gate system (D-06): per-pack gate sets (rust = cargo+wiring+maintainability+production-caller+tracing, MADE STRICTER for Opus-1M; unity = compile+console-clean+playmode smoke; generic = test+lint); done = active-pack gates pass; worker-never-self-certifies; gates frozen.
3.4  Concurrency/rate-limit scheduler (D-16): max-free envelope, back-off/cooldown/re-route on 429; build on budgets.py.
3.5  Event-driven session wake (D-07): modify the CC copy so a waiting session sleeps and is woken by an event (file-free/dep-done) — kill the poll-spam. (CRITICAL.)
3.6  Dashboard surface: orchestration monitor (task graph, leases, gate verdicts, concurrency).
done: a real ELAI task runs through select->context->execute->verify(gate)->integrate->record; event-wake demonstrated (session B woken by session A); max free concurrency observed; surface design pass.

CONTEXT TO FEED (phase 3): this plan §0,§1; conductor/ FULL; backlog_import.py; bn_status_report.py; wiring-enforcement.md + production-caller-gate.md + maintainability-review.md + tracing-test-assert.md + the checker scripts; idea-intake + the decomposition agents; ELAI plan-corpus grammar notes from conductor-build.md.
RISKS: the stricter gate over-blocking; schema not capturing ELAI's WIRING_CONTRACT variance; CC-source event-wake hook surface.

--------------------------------------------------------------------------------
PHASE 4 — THE TWO FRONT-ENDS   [dep: P3]
--------------------------------------------------------------------------------
Objective: interactive chat (A) + headless autonomous (B) over the one engine.

4.1  Headless front-end B: pick task type/system -> runs at max output; templates + free-text intake; reads the per-project state file (generalized feature-map) to know what to finish; GPU-availability check -> defer/escalate (D-14).
4.2  Interactive front-end A: one chat = N sessions; orchestrator-fronted + drop-in override; always-grill full-depth (D-10); toggles (orchestration on/off, model pool, model choices, skills, objective); the session decides when to spawn sub-agents (per-task).
4.3  Auto-migration rework (D-09): structured snapshot + RAG fallback; "migrating" indicator in the same chat; auto-carry skills/rules/toggles; more-accurate CC context counter; threshold configurable.
4.4  Session comms hub (D-07): shared event log + mailbox; spawn via orchestrator/queue; sessions can spawn sessions/orchestrators.
4.5  Dashboard surfaces: chat pane (image + audio upload, intervention/drop-in, migrating indicator); session list per project (Codex-like).
done: one chat survives an auto-migration with zero info loss (snapshot+RAG verified); a headless run finishes a task batch on ELAI; sessions wake/coordinate; surfaces design pass.

CONTEXT TO FEED (phase 4): this plan §0; the CC copy chat/session loop; the migration/handoff machinery; handoff-protocol.md (for the rework baseline); grill skill; the engine API from P3.
RISKS: snapshot fidelity vs lossiness; image/audio upload in Tauri; the chat-fronts-N-sessions UX coherence.

--------------------------------------------------------------------------------
PHASE 5 — DASHBOARD COMPLETION + HARDENING + CUTOVER   [dep: P4]
--------------------------------------------------------------------------------
Objective: full monitoring surface, state coverage, and cut over from F:.
5.1  Monitoring completion: accurate token/usage meter, gate-verdict timeline, handoff/migration chain, concurrency view, per-lane status.
5.2  Full state coverage per DESIGN-TEIKOKU.md §10 (empty/loading/error-inversion/offline) on every surface; standard design pass (no banned tells; contrast gates) PLUS a consistency pass (every screen composes the shared tokens/components; zero per-screen drift).
5.3  Concurrency/limits config UI (the hybrid toggles, D-16).
5.4  ELAI-finish dry run THROUGH TEIKOKU: run a real autonomous wave on ELAI (the first JOB) to validate end-to-end; record results.
5.5  Cutover plan: promote the WSL copy to the canonical home; F: becomes archived fallback.
done: dashboard drives+monitors both lanes fully; an ELAI autonomous wave runs through TEIKOKU; cutover documented.

CONTEXT TO FEED (phase 5): this plan; DESIGN-TEIKOKU.md (FULL) + workflow/design/mockup/teikoku-signal.html (visual ref); visual-ui-loop.md; design-taste.md; all prior-phase surfaces.
RISKS: cutover data/path drift; the ELAI-finish wave surfacing engine gaps.

--------------------------------------------------------------------------------
PHASE 6 — CODEX ORCHESTRATION MERGE (DEFERRED — LAST)   [dep: P5; OWNER-GATED]
--------------------------------------------------------------------------------
Objective: absorb the standalone Codex conductor's orchestration into TEIKOKU as the Codex worker lane. DEFERRED: do NOT start until the owner confirms the current standalone Codex conductor (F:\.ELAI_workflow\conductor\ + the WSL bootstrap-conductor) is finished separately. The standalone Codex effort keeps running on its own track until then.
6.1  Reconcile the Codex conductor's task-graph/gate/worker model with TEIKOKU's core engine; map the deltas.
6.2  Wire Codex (WSL+Docker, GPT-5.5 proxy) in as a co-equal worker pool in workers.toml, under TEIKOKU's scheduler + gates.
6.3  Retire the duplicate standalone bits the merge subsumes; keep anything still Codex-specific as a codex-pack.
6.4  Dashboard surface: Codex lane in the monitor + the new-session toggles.
done: a Codex worker runs a TEIKOKU task end-to-end through the shared gates; standalone duplication retired; surface design pass.
CONTEXT TO FEED (phase 6): this plan; conductor/ FULL; bootstrap-conductor-codex.md; conductor-build.md; AUTONOMOUS-FINISH-SYSTEM-2026-06-18.md; the TEIKOKU engine API from P3.
RISKS: starting too early (before the standalone Codex work is done); Docker/WSL Codex containment; proxy quota.

================================================================================
5. BUILD METHOD (how the orchestrator runs this)
================================================================================

- Driver: a fresh Opus session runs the ELAI `orchestrate` skill on THIS file (Appendix A boot prompt).
- Every agent (context/explorer/executor/auditor/memory) is Opus (C-02). The orchestrator feeds each the CONTEXT TO FEED list for the active phase + the relevant source, and lets it do a ~500k-token chunk.
- Target = the WSL copy (C-03). The orchestrator works there; the live F: root + VS Code are not touched.
- One sub-phase = context-compiler -> executor -> one auditor cycle (NO integration-explorer, NO memory-writer agent — owner override, C-02), MAX 2 agents in parallel, ONE change set, gate must pass. Memory = append to the handoff (teikoku-build.md), auto-indexed by the new RAG (D-08).
- Commit cadence: per the merged commit-cadence rule, ONE commit per shipped phase after its audit passes; TEIKOKU tooling lives in WSL and is gitignored like the current harness unless the owner sets a repo (Open Items O-3).
- Tripwires (ask the owner): an irreversible op, a 2+-option material fork not covered by §0, repeated gate BLOCK, a free-tier quota stall, or reaching Phase 6 (Codex merge is owner-gated — confirm the standalone Codex conductor is finished before starting it).

================================================================================
6. OPEN ITEMS (owner decides; none block starting Phase 0)
================================================================================
O-1  (RESOLVED) NAME = TEIKOKU (帝国, empire).
O-2  (RESOLVED) Unity = native Windows + bridge; no Unity-in-WSL probe (D-13).
O-3  (RESOLVED) TEIKOKU is its OWN tracked git repo (portable across machines, recoverable; mitigates the git-reset data-loss history).
O-4  Free-API provider set: exactly which free APIs/models from Liberty's backbone are in the default pool (and their measured limits) — resolved empirically in 0.3/0.4.
O-5  Design language RESOLVED = SIGNAL (workflow/design/DESIGN-TEIKOKU.md v2) + teikoku-signal.html reference. The dashboard wordmark reads "TEIKOKU" + 帝国 per DESIGN-TEIKOKU.md §9 (chrome component). Remaining: final @font-face bundling (Hanken Grotesk + JetBrains Mono); whether ELAI itself adopts SIGNAL later.
O-6  (RESOLVED) Codex lane KEPT (WSL+Docker, GPT-5.5 proxy, proven). The standalone Codex conductor finishes SEPARATELY first; TEIKOKU absorbs Codex orchestration only in the LAST phase (P6), gated on the owner's go.

================================================================================
APPENDIX A — ORCHESTRATION BOOT PROMPT (paste into a fresh Opus session)
================================================================================

Orchestrate F:\.ELAI_workflow\workflow\reasoning\TEIKOKU-MASTER-PLAN.md.

Policy overrides for this run:
- Build TARGET is a fresh WSL copy at ~/teikoku (create it in Phase 0.1), NOT the F: root. Projects are COPIED not moved, skipping heavy artifacts (target/, node_modules/, Library/, build caches). Do not edit the live F: root except to READ context and to APPEND to this plan / the handoff.
- Cycle = context-compiler -> executor -> auditor ONLY (NO integration-explorer, NO memory-writer agent — owner override). MAX 2 agents in parallel. All agents run model=opus. Size each sub-phase to ~500k tokens of agent work and FEED each agent the "CONTEXT TO FEED" list for the active phase plus the named source. Memory = append to the handoff (teikoku-build.md); the new RAG auto-indexes it as the changelog (D-08) — no memory-writer step.
- Section 0 of the plan is SETTLED — do not re-open those decisions. Section 2 is the scope guard — refuse creep into sidecar/ML/canvas/new-features.
- Phase 0 is mandatory-first and its 0.2 (CC Anthropic-plan-auth probe) then 0.3 (free-API graft) are the FIRST things tested. If plan-auth cannot survive in the WSL CC copy, STOP and surface it (tripwire) before proceeding.
- Every phase ships its dashboard UI slice (C-01); a phase is not done without its surface passing the design pass against DESIGN-TEIKOKU.md (SIGNAL: near-black + ONE dark-teal accent + high contrast; the anti-slop ban list; square status marks; click-`?` for detail), with workflow/design/mockup/teikoku-signal.html as the visual reference. The mockup is INCONSISTENT — P0.5 first derives ONE canonical token set + a shared component index, and every screen composes them; a per-screen-copied value or re-implemented component is a bug caught at audit (cross-screen consistency check).
- Gates are off-agent, frozen, per-pack, and STRICTER than ELAI's Sonnet-era per-plan check (D-06). Worker never self-certifies.
- Run phases P0->P5 in order. Phase 6 (Codex merge) is OWNER-GATED and DEFERRED — do NOT start it; the standalone Codex conductor finishes separately first.
- Ask the owner only on the Section 5 tripwires. Otherwise proceed autonomously, one sub-phase per cycle, ONE commit per phase after audit.
- Write progress to workflow/handoff/teikoku-build.md (one topic-keyed handoff, appended across sessions, per handoff-protocol.md).
