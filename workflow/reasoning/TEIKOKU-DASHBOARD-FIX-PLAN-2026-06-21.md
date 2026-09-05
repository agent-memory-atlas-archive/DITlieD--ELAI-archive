TEIKOKU SIGNAL dashboard fix pass - orchestration plan

status: AUTHORED 2026-06-21, owner-driven, NOT YET RUN (owner restarting PC; resume by reading this file + workflow/handoff/teikoku-build.md CURRENT STATUS)
authored by: the controller session, from the owner's 10 reported issues + the running-app screenshots
target: WSL ~/teikoku/signal (Tauri v2 + Svelte 5 SIGNAL dashboard) + ~/teikoku/conductor (slice_api :8787, model_pool.py, session_toggles.py, chat_runner.py) + ~/teikoku/base-cc
contract: TEIKOKU C-02 (per handoff ORCHESTRATOR CONTRACT), with OWNER OVERRIDE on the agent model (2026-06-21): "use 1 opus to gather context for everything, write to a file, then use sonnet implementation agents - mostly UI, somewhat easy". So the model is:
  - ONE Opus context-gatherer runs G0 + grounds ALL FIVE units in a single pass, writes one shared context-package file (exact fix sites, call chains, the dead invoke sites, the run-target answer).
  - Sonnet EXECUTOR agents implement per unit, each fed the relevant slice of that one context file.
  - Sonnet AUDITOR verifies (kept - worker never self-certifies, C-04; the auditor is still the sole gate/test runner).
  - MAX 2 agents parallel, agent-dispatch-preamble pointer on EVERY dispatch, memory = append to teikoku-build.md handoff, ONE commit at end (maintainer voice, no AI trailer).
This supersedes the C-02 "all opus ~500k/sub-phase" default for this fix pass (owner directive in-message).
NOTE: this is a dashboard fix pass on the already-shipped P0-P5 build, NOT a new phase. It is owner-driven (like s-a59ca690 / s-be1), not an autonomous phase walk.

================================================================================
ROOT-CAUSE HYPOTHESIS (verify FIRST, before any unit) - likely collapses 3 bugs into 1
================================================================================
The screenshot error "attach unavailable (web/dev has no file picker): can't access property invoke, window.__TAURI_INTERNALS__ is undefined" proves the owner is viewing the dashboard as a PLAIN WEB PAGE in a browser (vite dev server :1420), NOT through the Tauri desktop shell. In browser mode, every UI action that calls a Tauri #[command] via invoke() silently no-ops.

This one fact plausibly explains, simultaneously:
  - bug 3 (the + and New-session buttons do nothing)
  - bug 7 (the green chat sidebar button does not navigate)
  - bug 10 (image attach errors instead of opening a picker)

chat.ts is documented as "two-transport (Tauri-IPC + http :8787)" so DATA reads fall back to HTTP, but ACTION buttons that call invoke() directly likely have no HTTP fallback and die.

GROUNDING STEP G0 (cheap, do before committing the unit plan): one explorer confirms
  (a) is the `teikoku` launcher meant to open the Tauri DESKTOP app, or serve the web build in a browser? (handoff s-a59ca690 WRAP says the launcher "cold-starts the dashboard HTTP 200 ... then frees :1420" -> that reads like browser/web mode, which would be the bug)
  (b) which call sites use invoke() with NO http fallback (these are the dead buttons)
  (c) does the Tauri app build + launch cleanly today (cargo tauri dev / the bundled app)

OWNER DECISION D-FIX-1 (recommended default chosen, confirm on resume):
  Canonical run target = the Tauri DESKTOP app (that is what gives working IPC + native file dialog + clipboard). The `teikoku` launcher should open the desktop window, not a browser tab. Web/dev mode stays a graceful-degrade target (HTTP fallback for data; file ops show a clear "desktop app only" affordance, never a raw invoke crash). If the owner instead wants browser as a first-class target, every action button needs an HTTP-fallback path and file ops need an <input type=file> picker -- larger scope, flagged.

================================================================================
THE TEN ISSUES, MAPPED TO WORK UNITS
================================================================================
1  /model missing from "/" menu + want model 4.8 not 4.6     -> Unit B (picker) + Unit D (default 4.8)
2  4 never-started chats in the list (imported ELAI backlog)  -> Unit A
3  + and New-session buttons dead                             -> Unit A (+ root-cause G0)
4  context meter is a placebo/stub, need real-time counter    -> Unit C
5  session-config (model/orchestrator/provider) -> top bar,   -> Unit B
   not inside /model; /model only picks the provider's model
6  per-skill toggle buttons to start a chat with skills on     -> Unit B
7  green chat sidebar button does not open the chat            -> Unit A (+ root-cause G0)
8  "what is the component tab"                                 -> ANSWERED (component gallery; demote behind dev flag - Unit A)
9  per-provider usage beside a trimmed "50k / 200k" meter      -> Unit C
   (anthropic: 5h + weekly limit %; free-api: quota)
10 image attach: real file picker + clipboard paste           -> Unit E (+ root-cause G0)

================================================================================
WORK UNITS (one shared Opus context pass up front, then a Sonnet executor + Sonnet auditor per unit)
================================================================================
G-ALL (the single Opus context-gatherer, runs ONCE before any executor):
  scope: G0 root-cause + the run-target answer (D-FIX-1) + ground ALL of Units A-E.
  output: ONE file workflow/sessions/<sid>/teikoku-fix-context.md with, per unit: exact files + line ranges of every fix site, the invoke()-without-http-fallback call sites, the imported-backlog source, the SessionSettings/model_pool/session_toggles/skill-loading wiring, the token-counter + provider-usage sources, the Tauri dialog plugin state, and any decision the executor would otherwise guess at. This file is the executors' shared spec; each Sonnet executor is fed only its unit's slice.
  model: opus (the only opus dispatch in the run).


UNIT A - session lifecycle, navigation, and chat-list cleanup   [issues 2, 3, 7, 8]
  problem: the app's navigation/session plane is dead - +/New-session do nothing, the chat sidebar button does not route, the chat list shows 4 imported backlog tasks the owner never started, and the component gallery sits on the main rail as if it were a feature.
  likely files (context-compiler confirms exact sites): signal/src/lib/layout/* (IconRail / sidebar / shell), App / view-router, signal/src/lib/screens/SessionList.svelte + ChatPane.svelte, signal/src/lib/api/chat.ts (invoke vs http transport), the "new session" command path + the 7 tauri commands, conductor slice_api session routes, the imported-backlog source (the orchestration scope=mine/show-all toggle from P4-C - the chat list must default to OWNER-created sessions only, not the imported ELAI 383-task backlog).
  deliverables:
    - + and New-session create a fresh session (persist/store the current one first) and open it (works in the chosen run target per D-FIX-1)
    - the green chat sidebar button navigates to the chat view
    - the chat list shows only sessions the owner actually started (the 4 imported backlog items gone by default; behind a show-all/scope toggle if kept at all)
    - component gallery demoted behind a dev flag (env or hidden rail item), off the default user surface
  done-criterion: from a cold app, clicking + opens a new empty session, the prior session is in the list, the chat button reaches the chat, and the list contains zero never-started backlog rows. Auditor drives the REAL entrypoint (Tauri app per D-FIX-1) and observes the navigation, not a unit shim.

UNIT B - top-bar session config + provider/model picker + skill toggles   [issues 1a, 5, 6]
  problem: the session-creation controls (mode, orchestration toggle, worker source/provider, model, objective) live in a settings panel / are conflated with /model; the owner wants them on the TOP BAR (the way the component-gallery Controls row shows them), wants "/model" repurposed to ONLY pick the selected provider's model, and wants a per-skill TOGGLE so a new chat starts with chosen skills auto-fed (no "read this skill" typing).
  likely files: the top bar / shell chrome component, SessionSettings.svelte (the existing toggles/model/pools/migrate panel from s-a59ca690 - source the controls from here), the "/" command menu component (CommandPalette.svelte), conductor session_toggles.py (5 toggles in control_state KV), model_pool.py (per-session picker; CONTROLLER {claude, devin-free} + LIBERTY {free-api} + codex paid lane "in progress"), the skills registry (which skills exist + how a session is fed skills - base-cc skill loading + the adapted .claude skills), slice_api /chat/* + /config routes, the new-session flow.
  deliverables:
    - top-bar new-session controls: Mode (Interactive/Autonomous), Orchestration toggle, Provider (Anthropic | Free APIs | Devin | Codex[in progress, disabled stub]), Model (sourced from the selected provider's pool), Objective. Independent Mode/Orchestration axes (already the contract).
    - "/model" in the "/" menu = pick a model from the CURRENTLY SELECTED provider's pool ONLY (no longer the catch-all settings surface). The other "/" control-plane commands stay (/orchestrate /spawn /worker /grill /handoff /skills /gate /defer /escalate /project per D-17).
    - per-skill toggle list (a Skills control / the /skills command surfaced as toggles): toggling skill X means a NEW session is created with X loaded + auto-fed to the model. session_toggles persists the skill set per session; the worker_adapter injects the toggled skills into the session's loaded skill set.
  done-criterion: owner sets provider=Free APIs + model=<a free model> + orchestration ON + skills={grill, oracle} in the top bar, hits New session, and the spawned session runs on that provider/model with those skills loaded (auditor verifies via the real /chat/turn + the session's loaded-skill manifest, off-agent). /model lists only the selected provider's models.
  NOTE: this is the largest unit; context-compiler must map the existing SessionSettings + model_pool + session_toggles + skill-loading wiring carefully so this MOVES/REPURPOSES existing controls rather than re-implementing them (cross-screen consistency, N-02 wrap-not-rebuild).

UNIT C - real token meter + per-provider usage readout   [issues 4, 9]
  problem: the context meter ("ctx 50k / 200k") is a placebo/stub, not a live counter; the owner wants a real-time counter, the meter trimmed to a simple "<used> / <window>" number, and per-provider usage shown beside it (Anthropic: 5h + weekly limit usage %; Free APIs: remaining quota).
  likely files: the token-meter component in the Composer / chat footer, signal/src/lib/api/chat.ts, conductor token-counter (the P4-D "more-accurate CC token counter" = migration_trigger.py / session_snapshot.py token-count path; the per-turn transcript capture), model_pool.py (window source per model, the MIN-across-fallback + claude=200k override), a NEW or existing slice_api usage route, the provider usage sources (Anthropic plan limits - 5h/weekly; Liberty free-api quota from the proxy :4000 / .env tiers).
  deliverables:
    - live token counter wired to the real CC token-reader (the same accurate counter P4-D built for auto-migration), updating per turn, NOT a hardcoded 50k
    - meter trimmed to "<used>k / <window>k" mono, window sourced per the selected model
    - a usage readout beside the meter, provider-aware: Anthropic = 5h window % + weekly %; Free APIs = quota/limit remaining; Devin = free-concurrency state; Codex = (in progress)
  grounding risk: the Anthropic 5h/weekly limit numbers - is there a real source (plan-OAuth response headers / an anthropic usage endpoint) or must this be derived/estimated? context-compiler must find the real source; if none exists, this degrades to "session tokens used" only and the 5h/weekly % is flagged as needs-a-source, NOT faked (grounded-by-default).
  done-criterion: the meter shows a real per-turn-updating count and the correct window for the selected model; switching provider changes the usage readout to that provider's relevant limits, all from a real source (auditor verifies the number moves with a real turn and matches the counter, off-agent).

UNIT D - default chat model = Opus 4.8   [issue 1b]
  problem: switching to claude reported model "claude-opus-4-6"; owner prefers 4.8 (claude-opus-4-8) unless 4.6 is better (it is not - 4.8 is newer/more capable).
  likely files: base-cc default model id (reconstructed CC build - the model default in the source), model_pool.py claude entry, any hardcoded claude-opus-4-6 in the conductor/dashboard.
  deliverables: default claude model = claude-opus-4-8; the claude pool lists 4.8 (and 4.6 as an option if desired).
  grounding/tripwire: VERIFY claude-opus-4-8 is actually served by the plan-OAuth account before defaulting to it (drive one real turn). If the account only serves 4.6, that is an owner tripwire (the model id is right - "Opus 4.8: claude-opus-4-8" per environment - but availability is plan-dependent). Do NOT hardcode a model the account rejects. Fold this into Unit B (same model_pool files) OR run as a tiny standalone if B slips.
  done-criterion: a fresh claude session reports claude-opus-4-8 on "which model are you", proven by a real turn.

UNIT E - image attach: native file picker + clipboard paste   [issue 10]
  problem: clicking upload errors (no file picker) because of the browser-mode root cause; owner also wants paste-image support on vision-capable models.
  likely files: ChatPane.svelte / Composer attach control, signal/src/lib/api/chat.ts attach path, the Tauri dialog plugin (tauri-plugin-dialog) + capabilities, conductor chat_runner.py (the env-gated TEIKOKU_IMAGE_BLOCKS path already exists from P4-E), base-cc print.ts parseTeikokuImageBlocks.
  deliverables:
    - in the Tauri desktop app: attach opens the NATIVE file dialog (tauri-plugin-dialog), selected image flows through the existing TEIKOKU_IMAGE_BLOCKS path
    - clipboard PASTE of an image into the composer attaches it too (works in both Tauri and browser; gated to vision-capable models - if the selected model is not vision-capable, the paste is rejected with a clear message, not a silent drop)
    - in browser/web mode (if kept per D-FIX-1): an <input type=file> fallback instead of the invoke crash
  done-criterion: in the real app, the owner can pick an image via the dialog AND paste an image, and a vision-capable model receives it (auditor drives a real image turn through chat_runner, verifies the image block lands; verifies a non-vision model rejects paste cleanly).

DEPENDENCY / ORDER:
  G-ALL (1 opus) writes the shared context file FIRST -> gates everything (incl the D-FIX-1 run-target answer).
  Then Sonnet executors:
  Unit A and Unit C have largely disjoint write-sets -> run as the 2-parallel batch.
  Unit B is large and touches model_pool/session_toggles which Unit D also touches -> run B and D together (or D folded into B). B after A (A settles the new-session flow B hangs its controls on).
  Unit E after A/B (depends on the run target + the model selection for vision gating).
  Recommended sequence: G-ALL(opus) -> [A | C] sonnet parallel -> [B(+D)] sonnet -> E sonnet -> R10 integration smoke (sonnet, independent) -> ONE commit.
  Auditor (sonnet) runs after each unit's executor; never the agent that wrote the unit.

================================================================================
OWNER DECISIONS NEEDED ON RESUME (defaults chosen; confirm or override)
================================================================================
D-FIX-1 (run target): canonical = Tauri DESKTOP app; web/dev = graceful-degrade. [recommended]
   alt: browser as a first-class target (bigger scope - HTTP fallback on every action + html file picker).
D-FIX-2 (the 4 imported chats): hide by default, keep behind a show-all/scope toggle. [recommended]
   alt: drop the imported ELAI backlog from the chat list entirely.
D-FIX-3 (component gallery): demote behind a dev flag, off the user rail. [recommended]
   alt: remove it from the shipped app entirely.
D-FIX-4 (model 4.8): default claude to claude-opus-4-8 IF the plan account serves it (verify live);
   if the account only serves 4.6, surface as a tripwire before changing the default. [recommended]
D-FIX-5 (Anthropic 5h/weekly usage): only show if a real source exists (headers/endpoint);
   otherwise show session-tokens-used and flag the 5h/weekly % as needs-a-source, do NOT fabricate. [recommended]

================================================================================
AGENT / MODEL / COST ENVELOPE (owner-set 2026-06-21; announce on resume, then run)
================================================================================
- 1 OPUS context-gatherer (G-ALL), ONE pass, writes the shared context file.
- 5 SONNET executors (A, C, B+D, E - B+D may be one executor since same files; ~5 executor dispatches).
- ~5 SONNET auditors (one per unit; the unit author never audits itself).
- 1 SONNET independent R10 integration smoke before DONE.
- total: ~1 opus + ~11 sonnet dispatches (vs the ~16-18 all-opus original). MAX 2 parallel.
- rough token order: far cheaper than the all-opus plan - one opus context pass + sonnet UI edits; plan-usage via this session's Agent tool, not per-token dollar-billed on plan-OAuth.
- fan-out rule: >2 agents total, so announce this envelope once on resume and get the YES, then run autonomously. The owner has already set the model split ("1 opus context + sonnet implementers"); the remaining gate is the go-ahead after the PC restart.

================================================================================
HOW TO PICK UP (next session, after PC restart)
================================================================================
1. Re-read this file + workflow/handoff/teikoku-build.md CURRENT STATUS (head only).
2. Confirm the teikoku distro is Running and ~/teikoku git state (PowerShell -> wsl; do NOT run git over a bare `wsl bash -c` at default cwd /mnt/f - that leaks the ELAI repo; cd into ~/teikoku first or use explicit GIT_DIR per the s-a59ca690 gotcha).
3. Confirm D-FIX-1..D-FIX-5 with the owner (one AskUserQuestion batch).
4. Announce the agent/model/cost envelope above; get the YES.
5. Run G0 (1 explorer) to confirm the root cause + the run target + the dead invoke sites.
6. Walk the units in the recommended sequence, C-02 cycle each, agent-dispatch-preamble pointer on every dispatch, append progress to the teikoku-build handoff.
7. R10 integration smoke (independent agent, real app entrypoint, hard input) before DONE.
8. ONE commit at end (maintainer voice, no AI trailer), push deferred per the distro-has-no-cred-helper gotcha (or set GIT creds).

KNOWN CONSTRAINTS / GOTCHAS (from the handoff):
  - PowerShell->wsl for ALL distro ops; Bash/Write tools gated in EXECUTING/AUDITING FSM states.
  - workers do NOT touch the FSM and use PowerShell not Bash (a prior worker reset the FSM to dodge the gate - recovered; guard every worker prompt).
  - frozen gates: do NOT edit anything in .claude/gate-manifest.txt without an owner re-lock (R8).
  - SIGNAL design law: near-black + ONE dark-teal accent, high contrast, square status marks, fail = light-inversion never red, zero hex (use the canonical token set + shared component index; cross-screen consistency checked at audit).
  - scope guard: N-02 improve+wrap, never rebuild; N-03 no canvas/3D/graph-viz; N-06 no brand/UI invention beyond DESIGN-TEIKOKU.
