# B-36: Dashboard-V2 src-tauri Wiring (Track D — D.1..D.13 + 15 net-new surfaces)
Objective: complete the deferred Track D of NEXT-SESSION-FINISH-DASHBOARD-V2.md
— wire the 13 D-commands + 15 net-new V2-only surfaces into
`crates/dashboard-v2/src-tauri/src/lib.rs` and replace the corresponding
entries in `crates/dashboard-v2/src/lib/api/stub.ts` with real Tauri invokes.
This is the dedicated dashboard-v2 wiring track.
Estimated effort: 4 sessions (planned as Track D Sessions 1..4 in HANDOFF).
Pre-requisites: B-1 (path unification — sidecar reads .elai_cc paths),
                B-2 (FSM phase transition — StatusBar consumer ready),
                B-3 (stub audit — duplicates removed),
                B-7 (decomposition tree producer — D.4 consumer ready).
Parallel-safe with: B-29..B-35 (different crate scope).

## V2 REBASELINE NOTES

This file is the rebaselined home of Track D from the original 4-track
"finish dashboard-v2" prompt at workflow/sessions/s-d1-d5-2026-05-05/
NEXT-SESSION-FINISH-DASHBOARD-V2.md (Tracks A+B+C completed in s-155876,
s-934436, s-1430428, s-1875676, s-1976592 — see status blocks in that file).
Track D was the only remaining piece per the user's "no dashboard wiring for
now" instruction at s-1430428.

V2 baseline as of 2026-05-07:
- Tauri backend at `crates/dashboard-v2/src-tauri/src/lib.rs` registers
  EXACTLY TWO commands: `greet` (Tauri scaffold) and `get_grounding_cache_stats`
  (returns hard-coded zeros — Track A.6 stub from s-155876).
- Frontend at `crates/dashboard-v2/src/lib/api/stub.ts` exports ~60+ stubs
  backed by `crates/dashboard-v2/src/lib/api/mock-data.ts`. Every stub adds
  80–240 ms artificial latency and `console.info('[stub]', name, args)` so
  wiring points are obvious.
- V2 has no engine crates as deps (per src-tauri/Cargo.toml comment: "Engine
  crates intentionally omitted until the new dashboard's design is finalized").
  Adding `elai-orchestrator`, `elai-cli`, `elai-core`, `elai-providers`, etc.
  is part of B-36 Session 2 and requires user confirmation per CLAUDE.md.
- CLI launcher `crates/cli/src/dashboard.rs:7` constant
  `DASHBOARD_BINARY_NAME` still points at the V1 name "elai-dashboard"; V2
  produces "elai-dashboard-v2.exe". `ELAI_DASHBOARD_BINARY` env var override
  works at `crates/cli/src/dashboard.rs:99-108`. Fix in Session 2.

## Context

Per NEXT-SESSION-FINISH-DASHBOARD-V2.md §6 (TRACK D — DASHBOARD-V2 SRC-TAURI
WIRING):

> The dashboard-v2 frontend stubs every command via src/lib/api/stub.ts.
> For each command name listed in TESTED-AND-DASHBOARD-WIRING.md sections
> 1-11, replace the stub with a real Tauri invoke once the src-tauri backend
> implements the wrapper.

Per s-1430428 context-compiler validation: "only 1 of 13 D-commands exists
verbatim in V1 commands.rs — get_grounding_cache_stats". Build D.1..D.13
from spec sections 1-11 of TESTED-AND-DASHBOARD-WIRING.md. V1 is style
reference only (do not back-port stale V1 command shapes — trust
DOMAIN-CONTEXT.md and TESTED-AND-DASHBOARD-WIRING.md for authoritative
shapes).

The 15 net-new V2-only surfaces come from:
- batch-ful-d MCP bridge (3 SSB variants + dashboard SETTINGS row)
- A-8 freshness steering vector (recompute_steering_vector + 3 endpoints)
- A-17 GG sidecar signals (mean_logprob, min_logprob, entropy_mean,
  entropy_max, generation_novelty)
- patch-1.X cache_control marker boundary parsing
- Track C-D meta.gg_signals top-level field
- Track C-B constrained decoding response_format
- Phase A elapsed_ms in /v1/messages

See workflow/sessions/s-46448/track-d-delta.md for the full impact-set.

## Reads (mandatory before coding)

- workflow/sessions/s-46448/HANDOFF-track-d-v2-rebaseline.md
  (Session 1..4 plan + per-session deliverables)
- workflow/sessions/s-d1-d5-2026-05-05/NEXT-SESSION-FINISH-DASHBOARD-V2.md
  §6 TRACK D (D.1..D.13 spec)
- workflow/sessions/s-d1-d5-2026-05-05/TESTED-AND-DASHBOARD-WIRING.md
  sections 1-11 (data shapes)
- workflow/sessions/s-46448/track-d-delta.md (D.1..D.13 + 15 net-new
  inventory + V1 vs V2 gap analysis)
- .elai_cc/crates/dashboard-v2/DOMAIN-CONTEXT.md
  §0 V2 wiring status (NEW per 2026-05-07 rebaseline) +
  §1 command list + §6 30-element gap list
- .elai_cc/crates/dashboard-v2/DESIGN.md (look/feel canonical, §7
  hover-reveal rule, §8 orbit mode resolution)
- .elai_cc/crates/dashboard-v2/HANDOFF-galaxy.md +
  .elai_cc/crates/dashboard-v2/HANDOFF-galaxy-G3..G7.md +
  .elai_cc/crates/dashboard-v2/HANDOFF-orbit.md (V2 surface handoffs)
- .elai_cc/crates/dashboard-v2/README.md, DESIGN-BRIEF.md
- .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs (current 28-line state)
- .elai_cc/crates/dashboard-v2/src-tauri/Cargo.toml (deps to add)
- .elai_cc/crates/dashboard-v2/src/lib/api/stub.ts (~757 lines, ~60+ exports)
- .elai_cc/crates/dashboard-v2/src/lib/api/mock-data.ts
- .elai_cc/crates/dashboard-v2/src/lib/api/types.ts (TS type sources)
- crates/.bkp_old/dashboard/src/commands.rs  (V1 STYLE REFERENCE ONLY —
  V1 has 57 #[tauri::command] handlers but only 1 maps verbatim to D.1..D.13)

## Tasks

### Session 1 — REBASELINE (analysis-only, no code)

Produces two analysis documents the executing sessions consume:

1. `docs/specs/track-d-v2-spec.md` — V2-native D-spec rewrite. ≥6 sections,
   ≥13 entries in §2 (one per D-row), ≥1 D2-row per non-obsolete §2 row.
   §6 OPEN_QUESTIONS lists at minimum the DASHBOARD_BINARY_NAME mismatch.
2. `docs/thefinalglance/B-plan-delta-2026-05-07.md` — per-file severity
   classification (CRITICAL/HIGH/MEDIUM/LOW) of every B-N file affected by
   the V2 rebaseline.
3. `workflow/sessions/<sid>/HANDOFF-track-d-v2-impl.md` with phase-cluster
   table + reading order + acceptance gates.

Acceptance gates G1..G5 from HANDOFF §1.4 must pass. Auditor PLAN-mode 0 BLOCKs.

### Session 2 — engine wiring + D.1 + D.13

4. Add to `crates/dashboard-v2/src-tauri/Cargo.toml`:
   ```
   elai-orchestrator = { path = "../../orchestrator" }
   elai-cli = { path = "../../cli" }
   elai-core = { path = "../../core" }
   elai-providers = { path = "../../providers" }
   tokio = { workspace = true, features = ["rt-multi-thread", "macros"] }
   reqwest = { workspace = true }
   ```
   USER CONFIRMATION REQUIRED before adding deps.

5. Define `AppState` struct with shared `Arc<Mutex<...>>` handles to:
   - `Arc<dyn SsbWriter>` (orchestrator)
   - `Arc<ResearchCache>` (orchestrator) — per Track A.6 / D.7
   - `ProjectPaths` (cli/core)
   Register via `.manage(AppState::new(...))` in the Tauri builder.

6. **D.1 sidecar_health()** -> SidecarHealth — GET http://127.0.0.1:8100/health
   (or minimal /v1/messages probe). Returns alive + model + vram_* +
   head_dim/layers + dola/kv_quant/steering flags. Mirror
   TESTED-AND-DASHBOARD-WIRING §10.5 schema verbatim.

7. **D.13 get_sidecar_settings(), set_sidecar_settings(), restart_sidecar()**
   — edits the canonical boot-command file and restarts via SSH to the
   remote box (NEVER local — see CLAUDE.md HARDWARE-HEAVY rule). Reuses
   ~/restart-sidecar.sh from s-1875676.

8. **DASHBOARD_BINARY_NAME fix** — rename constant at
   `crates/cli/src/dashboard.rs:7` from "elai-dashboard" to
   "elai-dashboard-v2" OR document `ELAI_DASHBOARD_BINARY=elai-dashboard-v2`
   in elai.bat. Pick one; note the choice in the decision doc.

### Session 3 — research / planning / one-shot

9. **D.2 run_planning_task(desc, tier?)** — drives /task --new path via
   WebResearchRunner. Streams `web_research_step` rows back via
   `tauri::Window::emit("web_research_step", ...)` so the activity feed
   (orbit captions for V2) renders the phase timeline live.

10. **D.3 run_research(query)** -> ResearchAnswer — drives /research;
    returns the Track A.2 ResearchAnswer struct (prose, citations,
    sources, path).

11. **D.7 one_shot_prompt(text, backend, port, allowed_tools)** — spawns
    elai.exe with --backend=local + envs from §10.1 (ANTHROPIC_BASE_URL,
    ANTHROPIC_API_KEY, ELAI_LOCAL_MIN_PROMPT, ELAI_LOCAL_MAX_CONTEXT).
    Streams sidecar log lines via Window::emit("turn", ...) /
    emit("streaming_started", ...) / emit("response", ...).

### Session 4 — settings + observability + net-new

12. **D.4 fetch_settings_get / fetch_settings_set / obscura_test_fetch(url)**
    — settings persistence + a synchronous obscura.exe driver.

13. **D.5 run_d5_live_contract(autostart)** — spawns the cargo test binary
    with ELAI_LIVE_SIDECAR=1; if autostart=true, runs
    workflow/remote-setup/start-tunnel.ps1 first; returns per-query timings.

14. **D.6 reindex_now()** -> DkgReadyPayload — synchronous indexing pass on
    the active project; returns the final dkg_ready row payload. Wraps
    IndexingService.reindex_all().

15. **D.8 get_tool_visibility()** -> {advertised, hidden} — reads
    MODEL_HIDDEN_TOOL_NAMES + filter_tool_specs output.

16. **D.9 set_local_max_context(tokens?)** + get_local_budget() — persists
    ELAI_LOCAL_MAX_CONTEXT into a settings file the elai.exe spawn picks
    up.

17. **D.10 set_local_min_prompt(enabled)** + get_local_min_prompt() —
    same pattern as D.9 for ELAI_LOCAL_MIN_PROMPT.

18. **D.11 sidecar_search(query)** -> Vec<SearchResult> — GET
    http://127.0.0.1:8100/search?q={query}. Map response shape to the
    existing SearchResult TS type.

19. **D.12 get_grounding_cache_stats()** — UPGRADE the existing stub at
    `crates/dashboard-v2/src-tauri/src/lib.rs` to read from real SSB
    grounding_cache_event rows added by Track A.6. Today returns zeros
    until V2 has a live SSB handle (now provided by AppState in Session 2).

20. **15 net-new surfaces** — wire the V2-only commands enumerated in
    track-d-delta.md NET-NEW-1..15:
    - recompute_steering_vector (A-8 freshness steering)
    - get_steering_status
    - get_mcp_status
    - get_gg_signals (A-17 sidecar signals)
    - get_dkg_stats
    - get_sidecar_status
    - get_bubble_up_events
    - get_hook_executions
    - get_shortcoder_stats
    - run_explore_query
    - continue_task
    - get_grc_budgets (real, post-B-5)
    - get_evidence_gates (real, post-B-6)
    - get_context_budget (real, post-B-10)
    - get_search_mode (real, post-B-4)

After backend lands, replace the corresponding stub in
`crates/dashboard-v2/src/lib/api/stub.ts` with:
```ts
import { invoke } from '@tauri-apps/api/core';
return invoke('<cmd>', { ...args });
```

Keep mock-data.ts intact for offline preview mode (toggle via
`ELAI_DASHBOARD_OFFLINE=1` env var — to be added).

## WIRING_CONTRACT

produces_symbols:
  - 13 D-command Tauri handlers in dashboard-v2/src-tauri/src/lib.rs
  - 15 net-new Tauri handlers (steering / MCP / GG signals / sidecar status /
    DKG stats / hook exec / etc.)
  - AppState struct with Arc<...> handles to orchestrator + cli + cache
  - dashboard-v2 binary `elai-dashboard-v2.exe` reachable from
    `crates/cli/src/dashboard.rs::run_dashboard()`
  - SSE event names: web_research_step, turn, streaming_started, response
consumes_symbols:
  - elai-orchestrator: SsbWriter, ResearchCache, IndexingService,
    WebResearchRunner
  - elai-cli: ResearchAnswer, BudgetPolicy::Local, MODEL_HIDDEN_TOOL_NAMES
  - sidecar /v1/messages, /search, /health endpoints
reachable_from:
  - elai.bat (after DASHBOARD_BINARY_NAME fix)
  - elai.exe --dashboard
stub_ban:
  - No stub.ts entry returns hard-coded mock data when the corresponding
    Tauri command exists in lib.rs (mock-data only via explicit offline
    flag).
  - No `Ok(default())` returns from D-commands — every command reads
    real state or returns typed `NotImplementedYet` error.
frontend_pair: every D-command has a Svelte component pair (window or drawer)
observable_effect:
  trigger: launch dashboard-v2; exercise each command from its window/drawer
  expect_ssb: real producer rows appear in ssb.db within Nms per command
  expect_artifact: screenshots in workflow/sessions/<sid>/screenshots/
                   D-<item>.png

## DETERMINISTIC_CHECK

```bash
# 1. Tauri command count
rg "#\[tauri::command\]" .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs --count
# Expected: ≥28 (2 baseline + 13 D-rows + 13 net-new at minimum).

# 2. invoke_handler enumerates them all
rg "tauri::generate_handler!\[" .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs -A 50

# 3. cargo check
cargo check -p elai-dashboard-v2

# 4. svelte-check
pnpm --prefix .elai_cc/crates/dashboard-v2 run check

# 5. Stub.ts no longer mock-data backed for wired commands
rg "logStub\(" .elai_cc/crates/dashboard-v2/src/lib/api/stub.ts --count
# Expected: count drops as D-commands are wired (offline-mode mock paths
# stay; live paths use invoke()).

# 6. Binary name fix
rg "DASHBOARD_BINARY_NAME" .elai_cc/crates/cli/src/dashboard.rs
# Expected: "elai-dashboard-v2" OR documented env-var override path.
```

## ACCEPTANCE

- [ ] All 13 D-commands implemented in dashboard-v2/src-tauri/src/lib.rs
- [ ] 15 net-new Tauri commands implemented
- [ ] Each replaces its stub in src/lib/api/stub.ts (offline mock kept
      behind ELAI_DASHBOARD_OFFLINE=1 flag)
- [ ] cargo build --release -p elai-dashboard-v2 PASSES
- [ ] pnpm tauri:build (.elai_cc/crates/dashboard-v2/) PASSES
- [ ] CLI launcher resolves "elai-dashboard-v2.exe" (constant fix or
      ELAI_DASHBOARD_BINARY env var path)
- [ ] No new Rust dependencies added without user confirmation (engine
      crates expected — confirm explicitly)
- [ ] Visual smoke: each command exercised once with screenshot in
      workflow/sessions/<sid>/screenshots/D-<item>.png
- [ ] track-d-v2-spec.md exists with ≥6 sections, ≥13 entries in §2
- [ ] B-plan-delta-2026-05-07.md exists with ≥10 B-plan files reviewed
- [ ] HANDOFF-track-d-v2-impl.md exists with phase-cluster table

## Tests to add

- One contract test per D-command in
  `crates/contracts/tests/dashboard_v2_<cmd>_contract.rs` exercising the
  Tauri command via Tauri's MockRuntime (or process spawn for one_shot_prompt).
- ELAI_LIVE_SIDECAR-gated end-to-end test exercising D.1 + D.11 against
  remote sidecar.
- Snapshot test of each window in dev mode (Playwright/Vitest).

## Files touched

- MODIFY: .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs
  (28 lines -> ~600+ lines after wiring)
- MODIFY: .elai_cc/crates/dashboard-v2/src-tauri/Cargo.toml
  (engine crate dependencies)
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/api/stub.ts
  (replace mock returns with invoke() for wired commands)
- MODIFY: .elai_cc/crates/cli/src/dashboard.rs (DASHBOARD_BINARY_NAME fix
  OR env var documentation in elai.bat)
- ADD: .elai_cc/crates/dashboard-v2/src-tauri/src/state.rs (AppState struct)
- ADD: .elai_cc/crates/dashboard-v2/src-tauri/src/commands/  (per-command
  modules if lib.rs grows past ~800 lines)
- ADD: docs/specs/track-d-v2-spec.md
- ADD: docs/thefinalglance/B-plan-delta-2026-05-07.md
- ADD: workflow/sessions/<sid>/HANDOFF-track-d-v2-impl.md
- ADD: workflow/reasoning/B-36-decisions.md (one per session: 1, 2, 3, 4)

## Decision doc

Per session, write `workflow/reasoning/B-36-S<n>-decisions.md`:
- Session 1: Document A (V2-native D-spec) + Document B (B-plan delta)
  rationale; OPEN_QUESTIONS resolution
- Session 2: AppState shape + engine crate dep choice + binary name fix
  approach (rename vs env var)
- Session 3: stream emit naming convention + back-pressure handling
- Session 4: net-new surface mapping (which V2 component owns each)

## Anti-pattern reminders

- Do NOT back-port V1 commands.rs verbatim. Only 1 of 13 D-commands maps
  cleanly (`get_grounding_cache_stats`). Build from spec.
- Do NOT touch the orbit visual (lib/orbit/*) unless explicitly asked —
  HANDOFF-galaxy-G7.md documents WIP visual state. Track D backend wiring
  must NOT alter the orbit code.
- Do NOT boot a local sidecar. Use the remote one (host
  maintainer@192.0.2.10 port 6465). HARDWARE-HEAVY flag in chat first.
- Do NOT commit unless the user explicitly says "commit". The user
  collects + commits manually per s-1875676 / s-1976592 precedent.
- Do NOT introduce new frontend dependencies (Svelte 5 + three.js +
  Tauri 2.10 + Vite 5 already cover everything). Confirm before adding
  any npm/cargo dep.

## Next

After B-36 ships: dashboard-v2 is fully wired. Demo-able end-to-end against
the remote sidecar. Closes the multi-session "finish dashboard-v2" prompt.
Consumers of B-36's surfaces (B-22 autonomous mode dashboard, B-23 self-mod
proposals panel, B-26 polish + animations) can proceed.
