# ELAI Domain Context — what the dashboard must surface

This file lives next to `DESIGN.md`. **`DESIGN.md` is the source of truth for the *look* and *feel* — read it first.** This file defines the *content* and *data shapes*. Every stub you produce must reflect the data shapes below, because the user will wire real backend signals into these exact places.

ELAI in one line: a Rust-native development engine. User gives it a task; system designs/builds/tests/validates the solution using local LLMs (with cloud fallback). 9-crate architecture (core, providers, knowledge, reasoning, rules, tools, plugins, orchestrator, cli) plus add-on crates (agent, ipc-types, web_search, scaffold, web_validator, dashboard-v2, contracts).

All stubs go through `src/lib/api/stub.ts`. Each function must:
- Match the Tauri command name 1:1 where one exists.
- Return realistic, plausible mock data.
- Add 80–240 ms of artificial latency.
- `console.info('[stub]', name, args)` so wiring points are obvious.
- TypeScript types live in `src/lib/api/types.ts` — domain types reused across windows + drawers.

## 0. V2 WIRING STATUS — 2026-05-07 REBASELINE

Tauri backend at `crates/dashboard-v2/src-tauri/src/lib.rs` currently registers EXACTLY TWO commands:

```rust
tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![greet, get_grounding_cache_stats])
```

`greet(name)` is the Tauri scaffold default. `get_grounding_cache_stats()` returns hard-coded zeros (Track A.6 stub from s-155876 — no live SSB handle in V2 yet).

Every command listed in §1 below — and the gap-signal commands listed after it — is currently a **frontend stub** in `src/lib/api/stub.ts` (~60+ exports) backed by `src/lib/api/mock-data.ts`. None of them round-trip to a real Tauri invoke yet. Wiring them is Track D of the dashboard-v2 finish prompt (D.1..D.13 + 15 net-new surfaces — see workflow/sessions/s-46448/HANDOFF-track-d-v2-rebaseline.md).

V2-specific facts (changed from V1):
- Crate at `crates/dashboard-v2/src-tauri/` (V1 lived at `crates/dashboard/src/`; V1 is excluded from the workspace and parked at `crates/.bkp_old/dashboard/`).
- Binary name is `elai-dashboard-v2` (V1 was `elai-dashboard`). The CLI launcher constant `DASHBOARD_BINARY_NAME` at `crates/cli/src/dashboard.rs:7` still points at the V1 name — this is a known wiring gap and must be fixed by Track D Session 2 (or set `ELAI_DASHBOARD_BINARY` env var meanwhile).
- Frontend tree: `crates/dashboard-v2/src/lib/{windows,drawers,layout,orbit,scene,screens,ui,api}` plus `crates/dashboard-v2/src/App.svelte` and `src/main.ts` (Svelte 5 `mount(App, ...)` pattern per commit f6f96b0).
- Stack: Svelte 5 (`$state`/`$derived`/`$effect`/`$bindable` runes), Tauri 2.10, Vite 5, three.js 0.184, troika-three-text 0.52 (orbit dome). Install + dev: `pnpm install` then `pnpm tauri:dev` from `crates/dashboard-v2/`.

When this file says "Existing wired Tauri commands" below, read it as **"command names the V2 frontend stubs against — to be wired by Track D"**. The 1:1 stub-name discipline still holds.

## 1. Tauri command surface (frontend-stubbed, awaiting Track D wiring)

```
get_fsm_state()                  // current FSM state + history
get_query_trace_recent(n)        // recent QueryTrace entries
get_search_mode()                // HYBRID/SPARSE_ONLY/BM25_ONLY
get_embedding_breakers()         // 5 models, each with breaker state
get_recall_metrics()             // R@5, R@10, MRR per tier
get_grc_budgets()                // per-role remaining budget
get_grounding_sweep()            // claims classified
get_csq_pending()                // CSQ Critical pending guard
get_evidence_gates()             // 7 gates with verdicts
get_agents()                     // active agent panel
get_supervision_log()            // restart/death events
get_context_budget()             // 8-slot allocation
get_decomposition_tree()         // Concept->Phase->Step->BabyStep
get_executable_pipeline()        // ExecutablePipeline DAG
get_dag_repair_log()             // Reorder/Substitute distribution
get_research_tree()              // FORMULATE/SEARCH/EXTRACT/REFLECT/CONVERGE
get_vram()                       // per-GPU VRAM gauge
get_inference_metrics()          // tokens/sec, latency
get_rules_lifecycle()            // PROPOSED/CANDIDATE/SHADOW/ACTIVE/RETIRED counts
get_active_rules(state)          // rules active in current FSM state
get_deprecated_patterns()        // pattern count + recent additions
get_source_quality_tiers()       // last research tier distribution
get_web_arag_state()             // WebContentIndex
get_build_slots()                // draft/live preview state
get_dev_servers()                // PID, port, status list
get_project_intelligence()       // ProjectProfile tier 1/2/3
get_mistake_book(babyStepId?)    // entries
get_interrogation_tree(language) // 3-7 multiple-choice diagnosis
get_failure_taxonomy_last()      // 9-cat classification of last error
get_bsp_supersteps()             // recent superstep snapshots
get_session_branches()           // active/merged/abandoned
get_anti_spiral_counter()        // step hash counter
get_heartbeats()                 // 5s pulse / 15s timeout per agent
get_baseline_results()           // A-1 baseline harness 7 metrics
get_dep_audit_results()          // nightly registry audit
```

## 2. The 7 windows (data shapes)

Tabs in either panel. Each window hosts one type. Default arrangement: PRIMARY = ACTIVITY, SECONDARY = COMMAND.

### COMMAND
- A vertical list of cards (status / intervention / error / disambiguation / steering / user).
- Bottom: outlined chat shell (`.input-shell`) with `--bg-input` interior, `--teal-line` border on focus. The shell contains the slash toggle, the textarea, and (when text is present) the send button.
- The slash palette is a floating dialog anchored above the chat shell; it has a `Filter actions...` input at the top and groups commands by category. Pressing `/` at the start of the textarea auto-opens it.
- Stub history: 6 example entries spanning all card types so styling is visible.
- Slash commands list (grouped by `category` in `SLASH_DEFS`):
  - **Workflow** — `/task`, `/fix`, `/research`, `/continue`
  - **Project** — `/explore`, `/status`, `/config`
  - **Help** — `/ask`
- Card buttons: the four-button intervention row uses Approve (primary outlined teal, `Button variant="primary"` or `.btn-approve`) and Reject/Modify/Redirect (secondary outlined neutral, `Button variant="ghost"` or `.btn-outlined`). Error cards have white titles + teal dot (NOT warn-orange); buttons all use the secondary outlined style. Steering and status lines indent to match the orbit-card horizontal padding so they don't read as off-grid.

### PLAN
- DecompositionTree, NOT a flat phase list. Levels: Concept → Phase → Step → BabyStep.
- BabyStep has 8 blocks: GROUNDING, PRE_GATE, TASK, TDD, DETERMINISTIC_CHECK, ACCEPTANCE, INTEGRATION, EXIT_CRITERIA.
- Active node pulses (breath). Done nodes teal. Pending nodes ghost. Failed nodes have a fail left-bar.
- BubbleUp event log strip beneath tree (BabyStep×3 → Step×2 → Phase×1 → Concept×1 → Escalate).
- Selecting a node shows its detail in a right gutter inside the same window.

### ACTIVITY
- ActivityWindow is now the **orbit visualizer** — the deterministic-workflow starmap that is the visual centerpiece of ELAI. The text-based activity feed described in earlier versions of this file has been retired; the orbit IS the activity surface.
- Three modes, FSM-driven (see `DESIGN.md` §8 for the canonical rules):
  - **Mode 1 — PLAN**: decomposition tree (Concept → Phase → Step → BabyStep) rendered as a constellation, with the active concept always at the canvas centre. Multi-concept tasks pan between concepts. Backdrop dots for every step + baby across all phases give the constellation feel; the active branch glows + has dash-flow edges.
  - **Mode 2 — MAP**: existing-project codebase map. PageRank-ranked symbols laid out in a score-banded ring; DKG edges as faint connecting lines; effect-tag color coding. Camera is manual (no auto-follow on edits) per user direction.
  - **Mode 3 — RESEARCH**: research tree (FORMULATE → SEARCH → EXTRACT → REFLECT → CONVERGE). Phi/psi convergence as twin micro-bars under each node. Auto-zoom-out so tendrils extending beyond the orbit are visible.
- Mode resolution: explicit `mode` prop wins; otherwise derived from `fixture.fsm`. RESEARCHING → research, IDLE+pagerank → map, anything else → plan.
- Captions: deterministic (`fsm`, `grc.step_type`, `bubble_action?`) → human-readable string via `src/lib/orbit/captions.ts`. No LLM. Anchored below the active baby step.
- Signals layered on the orbit (Phase C of the redesign):
  - FSM-state halo on the centre node (color from `--state-*`).
  - GRC reasoning dots orbiting the active baby step (color from `--grc-*`).
  - Context-budget donut around the centre at concept zoom (8 slots from `ContextBudget`).
  - 7 evidence-gate pips between phase ring and step ring (color from `--gate-*`).
  - Bubble-up cue (colored ripple) when `bubble_action` is set (color from `--bubble-*`).
- Idle: `OrbitShootingStar` fires every ~14 s (deterministic per fire-counter).
- Activity-host header ("CONCEPT …") and dev mode-toggle pill ("ORBIT …") follow the hover-reveal rule (§DESIGN.md §7) — `opacity: 0` until you mouse over the panel.

### REVIEW
- Unified diff (not side-by-side). Sticky file header. Per-hunk + per-file Accept/Decline/Retry. Multi-file tabs at top. Lines: added = teal-ghost bg + teal-line left border, removed = fail bg 10% + fail left border, context = bg-base.

### EXPLORE
- Oracle Q&A. Each query independent (no chat history). File refs clickable. Source citations in `--text-ghost` after each answer block (`from DKG`, `from BM25`, `from CrossEncoder`, etc.).

### MONITOR
- Identity (task name + elapsed). Progress bar + phase dots row (`. * - ! o`).
- Status snapshot 4 prose lines (currently / health / model / spend).
- Interventions inbox (only when non-empty).
- Timeline of last ~15 events.

### SETTINGS
Section groups (flat list per group, no cards):
- Execution: LOCAL/CLOUD/HYBRID, model routing per role (executor/auditor/researcher/designer/prompt-engineer/context-compiler/memory-writer/media-validator), VRAM budget, cloud key + spend cap.
- Control: INTERACTIVE/AUTONOMOUS, AutoApprove threshold, notifications.
- Project: path read-only, language/framework/build detected, override file edit.
- Appearance: font-size scale, panel layout preference.
- Inference: sidecar status (loaded/loading/error), DoLa toggle, ETC toggle, steering vector toggle, MCP bridge toggle (`ELAI_ENABLE_MCP`), TurboQuant TQ4/TQ8.
- Anti-staleness: nightly dep audit on/off, time-aware confidence decay window.

## 3. The 10 drawers (right-edge slide-out panels)

Each drawer slides from right (320 px wide). Triggered by command palette OR a small icon row at top-right of either panel. They are NOT windows — they overlay the secondary panel and dim the rest of the workspace by 10 %. Esc dismisses.

1. **KNOWLEDGE** — DKG nodes/edges count, BM25 chunk count, vector status (READY/DEGRADED), QueryTrace timeline (latest), SearchMode current + recent transitions, breaker grid 5 models, freshness decay status, R@5/R@10/MRR per tier (FAST/STANDARD/DEEP).
2. **REASONING** — per-role GRC budget (calls + chars remaining), GroundingSweep classified counts, CSQ Critical pending list, feasibility outcomes, mistake book preview, interrogation tree picker, GG sidecar signals (mean_logprob, min_logprob, entropy_mean, entropy_max).
3. **ORCHESTRATION** — FSM state + transition history, 7 evidence gates with verdicts, agents list (lifecycle + death classification), supervision log (restarts), BSP supersteps mini-timeline, ContextBudget 8-slot bar (per-slot %).
4. **RULES** — lifecycle counts (PROPOSED/CANDIDATE/SHADOW/ACTIVE/RETIRED), active rules in current FSM state, predicate type pie, deprecated_patterns (last added), VERSION_GATE state, source tier distribution (T1–T5).
5. **RESEARCH** — DecompositionTree fractal mini-map, ResearchTree (per-node FORMULATE/SEARCH/EXTRACT/REFLECT/CONVERGE), source quality of last research run, web research budget bar, FlashResearch convergence dual-threshold (phi/psi).
6. **PROVIDERS** — ModelPool (loaded models per GPU + last-used), VRAM gauge per GPU, sidecar status (FA2/DoLa/Steering/TurboQuant flags), tokens/sec last 60s, MCP bridge JSON-RPC ping, GPU multiplexing colocation events.
7. **PROJECT-INTELLIGENCE** — Tier 1 (lang, build, framework, test, ci, linter), Tier 2 (conventions strong/moderate/mixed), Tier 3 (architecture inference + telescope summaries 4 levels), override-file editor.
8. **ANTI-STALENESS** — deprecated_patterns count + last 5, nightly dep audit results, freshness steering vector status, baseline harness 7 metrics from A-1 (attention ceiling, retry rate, few-shot delta, etc.).
9. **TOOLS-HOOKS** — hooks fired (PreToolUse/PostToolUse/Stop/PreCompact) recent, T1/T2/T3 verdicts (ALLOW/BLOCK/WARN), ShortCoder savings %, edit-matching tier hits (exact/whitespace/ellipsis), failure taxonomy last classification, ToolFilter whitelist.
10. **WEB-QUALITY** — build slots (draft/live), dev servers (PID + port + status), scaffold templates (vite-svelte/vite-react/astro/vanilla), web rules count (12 T1 + 11 T2 + 4 T3), web a-rag state, source quality.

## 4. Status bar items (live, staged)

Always visible (left → right):
- **project name** (mono 10.5, `--text-secondary`)
- **FSM state badge** (mono 10.5, color from `--state-*`, lowercase, letter-spacing 0.04em)
- **mode** (4-px dot + lowercase mono label — LOCAL/CLOUD/HYBRID)

Hidden by default, fade in on `.status-bar:hover` or `:focus-within` (wrapped in `<span class="verbose">`):
- branch
- gate mode (INTERACTIVE/AUTONOMOUS)
- model name (mono 10.5 ghost)
- VRAM% if > 70 (mono, color shifts to `--warn` > 85, `--fail` > 95)
- session-id (mono 10 ghost, click copies)

When EXECUTING: a 1-px teal breath line at the very bottom of the status bar (`@anim-breath` 2.2 s — the only loading indicator in the entire app).

## 5. Mock data philosophy

- Every list has at least 3 entries so empty-state is never the default render.
- Use realistic ELAI-domain naming: Qwen2.5-Coder-7B, FSM=EXECUTING, baby-step ID like `bs-3.2.1.4`, file paths under `src/auth.rs`, `crates/orchestrator/src/fsm.rs`.
- Time stamps: relative (`2 min ago`) computed from a fixed `now = new Date('2026-04-25T12:04:00Z')` injected into stubs so the demo looks coherent.
- Numbers: BM25 p50=1ms, p99=2ms, R@5=0.633, R@10=0.767, MRR=0.412, DKG nodes=8075, edges=4148, build=9.67s — these are real numbers from metrics/phase_*.toml.
- Agents: executor + auditor + context-compiler + memory-writer + designer + prompt-engineer (+ optional researcher when in RESEARCHING state).
- VRAM tier table is real (8/12/16/24/48GB) — show it correctly.

## 6. UI elements explicitly required by the gap analysis

These MUST exist in the design (with stubs); they map directly to the user-facing gaps the team has been chasing:

1. Live FSM state badge in status bar — color-coded by state.
2. EXECUTING pulse animation on transitions (breath line, not spinner).
3. JetBrains Mono for code/timestamps/paths/IDs.
4. Workspace as primary shell (no Tier-0 burying).
5. Activity feed auto-scroll + Jump-to-Latest pill.
6. Inline `[preview]` entries with teal links.
7. Status bar lives inside Workspace.
8. Live token streaming chat (typing dots while tokens arrive — but use breath, never animated dots).
9. QueryTrace inspector (per-query stage timeline + latency).
10. SearchMode current + cascade transitions.
11. Per-model EmbeddingCircuitBreaker grid (5 models).
12. HyDE hypothesis preview + cache hit rate.
13. Per-tier R@5/R@10/MRR badges.
14. Per-agent GRC budget gauges.
15. GroundingSweep result panel.
16. CSQ guard prompts (Critical pending guard).
17. Inline feasibility Pass/Fail/Blocked badges.
18. 7 EvidenceGate verdict panel.
19. InteractiveGate prompt UI (Approve/Reject/Modify/Redirect — 4 buttons).
20. Per-agent panel (status, lifecycle, death classification).
21. SupervisionManager event log.
22. ContextBudget per-slot allocation visualizer.
23. DecompositionTree fractal visualizer.
24. ExecutablePipeline DAG visualizer (small force-graph or column flow).
25. DAG repair events log (Reorder/Substitute counts + T_DAG counter).
26. Web research per-node FORMULATE/SEARCH/EXTRACT/REFLECT/CONVERGE log.
27. Real-time VRAM gauge.
28. Per-step inference latency + tokens/sec.
29. vLLM /v1/models advertised list.
30. Rule lifecycle stats + Source tier distribution.

## 7. Special interactions

- Project selector default mode: LOCAL.
- Switching mode (LOCAL/CLOUD/HYBRID) triggers a brief teal breath on the mode dot in status bar.
- Switching gate (INTERACTIVE/AUTONOMOUS) re-arranges the default panel layout: INTERACTIVE → PRIMARY=ACTIVITY, SECONDARY=COMMAND. AUTONOMOUS → PRIMARY=MONITOR, SECONDARY=ACTIVITY.
- Drawers: only one open at a time. Opening a second closes the first.
- Command palette (Ctrl+Shift+P) — searches actions across windows + drawers + commands. Default empty state shows recent.

## 8. Unique differentiators (so this is NOT just Linear-clone)

1. **The orbit IS the activity surface** — a deterministic constellation that visualizes the engine's decomposition tree, the codebase, or the research tree depending on FSM state. A non-coder can watch a task run end-to-end and read the plain-language captions to know what's happening.
2. **Breath line** as the only loading indicator everywhere. No spinners. No skeletons that pulse left-to-right.
3. **Domain-specific drawers**: 10 drawers each tied to one engine subsystem — uniquely meaningful, not "more sidebar tabs".
4. **The orbit constellation as a living, breathing fractal** — `constellation-breathe` over 11 s, `ambient-flow` on inactive edges, `dashflow` on the active branch. Same task = same orbit (entropy-quarantined via seeded LCG).
5. **Hover-reveal chrome** — the dashboard is quiet by default; controls fade in only when the user is paying attention to that panel.
6. **Outlined-button language** — every interactive affordance is an outline + teal-ghost-on-hover. No solid-fill CTAs anywhere. The teal-line is the affordance.
7. **Status bar that says exactly what the engine is doing right now** — `EXECUTING bs-3.2.1.4 · auditor consuming 42% GRC budget` — concrete, never vague.
8. **Mode-aware default layout** that reconfigures on gate change.
