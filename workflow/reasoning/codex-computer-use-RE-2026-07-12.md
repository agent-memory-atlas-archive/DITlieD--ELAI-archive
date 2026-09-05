# Reverse-engineering the Codex desktop "Computer Use" feature, end to end

Date: 2026-07-12
Target: OpenAI Codex desktop app (the unified ChatGPT/Codex desktop app), Windows build
Installed version: `OpenAI.Codex_26.707.3748.0_x64__2p2nqsd0c76g0` (MSIX)
Method: static read of the installed app bundle + the deployed CUA runtime on this machine. No binary disassembly needed; the load-bearing layer ships as readable JavaScript + shipped API docs. Every claim below is anchored to a file on disk (evidence index at the end).

Grounding note: everything here is (2) code read on this machine or the app's own shipped docs, except the interior of the native `codex-computer-use.exe` binary, which is compiled and is explicitly flagged as NOT recovered (see "What is not recovered").


## TL;DR

The desktop Computer Use feature is NOT the public Responses-API `computer_call` action loop. It is a "code mode" agent:

1. The model runs inside a sandboxed Node REPL (`node_repl.exe`) exposed to it as an MCP-style `js` tool.
2. The model writes JavaScript that calls an OpenAI SDK named `@oai/sky` (the "sky" computer-use client).
3. `@oai/sky` (pure JS) marshals each call as JSON-RPC to a separate native helper process, `codex-computer-use.exe` (Windows) / `Codex Computer Use.app` a.k.a `sky_*` (mac/linux).
4. That native helper is the only privileged piece. It captures the screen with Windows.Graphics.Capture (occlusion-proof), reads the UI Automation accessibility tree, and injects input with SendInput. On mac it is a sandboxed XPC service using the AX API + CGEvent.
5. Perception handed back to the model is HYBRID: a JPEG screenshot data-URL PLUS a formatted accessibility tree with numbered element indexes, so the model can act either by pixel coordinate or by `element_index`.
6. Safety is enforced at the JS/transport boundary, not in the model: an allowed/denied/forbidden per-app policy engine, a blocking per-app user approval elicitation (persisted per-session or always), a physical-Escape hard kill switch, a per-turn millisecond budget, a client/server API-version lock, and a written confirmations policy the model must follow before risky actions.

Difficulty realized: the "how it works e2e" question that I earlier estimated at ~1 week of fresh Layer-C RE actually collapsed to a few hours, because OpenAI ships the entire orchestration layer as unminified-enough JS plus three markdown API docs plus a SKILL.md that names the exact OS APIs. Only the native capture/injection binary would need real RE (disasm/ETW) to go deeper, and you do not need to: its full contract is the JSON-RPC method set below.


## 1. Process / artifact map (where everything lives)

App bundle (MSIX, read-only, ACL readable here):
`C:\Program Files\WindowsApps\OpenAI.Codex_26.707.3748.0_x64__2p2nqsd0c76g0\app\`
- `ChatGPT.exe` — Electron shell (Chromium + Node main process). 8 renderer processes observed.
- `resources\app.asar` (199 MB) — the Electron UI/main (minified; NOT needed for Computer Use).
- `resources\codex.exe` (341 MB) — the Rust Codex agent CLI (the coding agent core). Runs as process `codex`.
- `resources\codex-code-mode-host.exe` (53 MB) — the "code mode" host that runs the model-authored JS agent loop.
- `resources\codex-command-runner.exe`, `resources\codex-windows-sandbox-setup.exe` — command execution + Windows sandbox setup.
- `resources\native\computer-use-app-icons.node`, `windows-account.node`, `windows-updater.node` — small native addons.
- `resources\plugins\openai-bundled\plugins\computer-use\` — the Computer Use PLUGIN (manifest, docs, skill, client bootstrap).
- `resources\cua_node\` — the TEMPLATE cua-node runtime that gets deployed to the user profile on first use.
- `resources\accessibility\`, `resources\skills\`, other plugins (browser, chrome, deep-research, latex, sites, visualize).

Deployed CUA runtime (user profile, fully readable):
`C:\Users\user\AppData\Local\OpenAI\Codex\runtimes\cua_node\ecfc0d9aa02807e3\`
- `bin\node.exe` (Node 24.14.0), `bin\node_repl.exe` (the sandboxed REPL host; runs as process `node_repl`).
- `manifest.json` — pins node 24.14.0, cua-node 0.0.3 (build `20260708...pr-1103797`), target windows-x64.
- `bin\node_modules\@oai\sky\` — THE computer-use SDK ("sky") v0.4.20 (author "noahj"). Readable JS.
- `bin\node_modules\` also carries the perception/automation toolbox: `playwright` + `playwright-core` (browser leg), `tesseract.js` + `-core` (OCR of screenshots), `sharp` / `jpeg-js` / `bmp-js` / `pngjs` / `pixelmatch` (image encode + diff), `pdfjs-dist` (PDF), `@napi-rs/canvas` (skia native canvas), `@statsig/js-client` (feature flags/telemetry).

The `@oai/sky` package internals:
`...\@oai\sky\`
- `bin\windows\codex-computer-use.exe` (1.69 MB) — the native Windows helper (capture + a11y + input).
- `bin\linux\sky_linux_x64`, `sky_linux_arm64` — the Linux helpers.
- `docs\sky-full-desktop-api.md`, `sky-window-api.md`, `sky-window2-api.md` — SHIPPED API references (the crown jewels).
- `dist\project\cua\sky_js\src\...` — the JS client, split by target: `targets\windows\`, `targets\mac\`, `targets\linux\`.
- `package.json` — `main` = `dist/project/cua/sky_js/src/index.js`; scripts include `test:windows-wgc` and `windows-wgc-e2e.ts` (Windows Graphics Capture e2e).


## 2. The three control API surfaces (what the model can do)

`@oai/sky` exposes three client shapes. The desktop Windows plugin uses the "window2" surface. All three are documented in the shipped docs.

A. Full Desktop API (coordinate-only, whole screen) — `sky-full-desktop-api.md`:
`get_screenshot()`, `click{x,y,click_count,mouse_button,key}`, `drag{from_x,from_y,to_x,to_y,key}`, `move{x,y}`, `press_key{key}`, `scroll{direction,pixels,x,y}`, `type_text{text}`.
Screenshot returns `{bytes, data_url (base64 JPEG), filepath}`. Keys use X Window System keysym names even on Windows (`Return`, `Tab`, `Control_L+a`, `Super_L+d`; aliases `Ctrl/Alt/Shift` accepted) — i.e. a cross-platform key abstraction baked into the native helper.

B. Window API (per-app, accessibility-aware) — `sky-window-api.md`:
`list_apps()`, `get_app_state{app, disableDiff?}` -> `{screenshot, text}` where `text` is the accessibility tree with element indexes (and it returns a DIFF from the previous tree by default). Then `click{app, element_index | x,y}`, `type_text`, `scroll{element_index}`, `set_value{element_index,value}`, `select_text{text,prefix,suffix,selection_type}`, `perform_secondary_action{element_index, action}` (invoke a named a11y action), `drag`, `press_key`.

C. Window2 API (per-window, current desktop shape) — `sky-window2-api.md` and the plugin's `docs\api.md`:
`list_windows()`, `list_apps()` (each app carries its open `windows[]`), `get_window{id, app?}` (rehydrate a window handle), `launch_app{app | "C:\\path\\to\\App.exe"}`, `get_window_state{window, include_screenshot=true, include_text=false}` -> `{window, screenshots[], accessibility}`, then `click{window, element_index | x,y, screenshotId?}`, `press_key`, `type_text`, `scroll{scrollX,scrollY,x,y}`, `set_value`, `drag`, `perform_secondary_action{action e.g. Raise/Expand/Collapse/Scroll Up}`, `activate_window`.
`WindowState.accessibility` = `{tree (formatted, indexed, tab hierarchy), focused_element, selected_elements[], selected_text, document_text}`.
`screenshots[]` are BOUNDED per-region with `{id, url (data URL), originX/Y, width/height, zIndex}` — transient UI (menus, popups) is captured as separate z-ordered layers, not one flat frame.


## 3. The native executor (the only privileged component)

`codex-computer-use.exe` (Windows), invoked by the JS transport. Its capabilities, stated verbatim in the plugin's `skills\computer-use\SKILL.md`:

"It automates apps via SendInput and UI Automation, and takes screenshots of app windows via Windows.Graphics.Capture that works even if they're occluded."

So the three OS mechanisms are:
- Screen capture: Windows.Graphics.Capture (WinRT WGC). Occlusion-proof = it captures a specific window's surface even when covered by other windows. Confirmed independently by the `test:windows-wgc` npm script and `scripts/windows-wgc-e2e.ts`.
- UI reading: UI Automation (UIA). Produces the accessibility tree, the `element_index` map, focused/selected element info, and the secondary-action list.
- Input injection: SendInput (synthesized mouse/keyboard at the desktop level). The X11-keysym key names are translated to Windows virtual-key input inside the helper.

The helper also resolves `element_index` -> concrete UIA element (the JS sends `click_element{element_index}` vs `click{x,y}`), lists apps/windows, launches apps, and activates/raises windows.

mac equivalent: a sandboxed background service app `com.openai.sky.CUAService` ("Codex Computer Use.app"), Apple Team ID `2DC432GLL2`. Uses the AX (Accessibility) API + CGEvent for the same capture/read/inject triad, plus macOS Screen Recording + Accessibility TCC permissions (granted on first use). Not present on this Windows box; identified from the mac transport code.

To go BELOW this contract (exact WGC frame path, UIA property set, SendInput timing) you would disassemble the 1.69 MB PE (Ghidra/IDA) or trace it live (ETW / API Monitor / WinDbg). Not necessary for an e2e understanding: the JSON-RPC surface in section 4 fully specifies what it does.


## 4. The transport protocol (JS <-> native helper)

Two platform transports, same JSON-RPC idea, different plumbing.

Windows — `dist\project\cua\sky_js\src\targets\windows\internal\helper_transport.js` (class `WindowsHelperTransport`):
- Spawns `codex-computer-use.exe` as a CHILD PROCESS via `node:child_process.spawn`, `stdio: ["pipe","pipe","pipe"]`, `windowsHide: true`, and `unref()`s it. Killed on host `exit`.
- Wire format: newline-delimited JSON on stdin/stdout. Request = `{id, method, params, meta}\n`; responses read line-by-line off stdout. Monotonic integer `id`; pending map keyed by id with per-request timeouts.
- Per-turn budget: injects header `x-oai-cua-request-budget-ms` (default 10000 ms) into every request's meta; a request that outruns it times out, kills the helper, and rejects.
- Turn scoping: each request carries `codexTurnMetadata` derived from `{session_id, turn_id}` -> internal key `codexHome \0 conversationId \0 turnId`. On a turn-scope change it fires `end_turn`. Resolves `CODEX_HOME` and even translates WSL paths (`wsl.exe wslpath -w`, `/mnt/c/... -> C:\...`) so it works when Codex runs from WSL.
- App-approval handshake: if the helper answers with an `approvalRequest {app, displayName, riskLevel: high|low}` instead of a result, the JS calls `createElicitation` -> user prompt "Allow Codex to use <app>?" with persist options `["session","always"]`. Only on "accept" does it re-issue the SAME request stamped with `APPROVED_APP_META_KEY`. So per-app consent is mediated in JS, and the helper will not act on an app until it sees the approved-meta stamp.
- Escape kill switch: the helper exiting with code 130 (SIGINT), OR the presence of an interrupt marker file at `<CODEX_HOME>\cache\computer-use\interrupts\<conversationId>\<turnId>`, causes every pending/next request to reject with a fixed message: "Computer Use was stopped by the user with the physical Escape key. Stop your work, do not call further Computer Use tools in this turn..." i.e. pressing physical Escape hard-stops the agent for the rest of the turn.

Windows typed client — `...\windows\internal\computer_use_client_base.js` (class `WindowsComputerUseClientBase`):
- The window2 method surface. Each method validates args then calls `transport.request(method, params)`.
- `click` chooses RPC `click_element{element_index}` when an index is given, else `click{x,y}` — the pixel/a11y fork.
- `get_window_state` parses `{window, screenshots[], accessibility{tree,...}}` and then pushes each screenshot INTO THE MODEL'S CONTEXT via `nodeRepl.emitImage(url)` / `codex.emitImage({type:"input_image", image_url, detail:"original"})`. That is the mechanism by which the screenshot becomes a model-visible image.
- Every parser insists "codex-computer-use.exe did not return ..." on missing fields — confirming the native exe is the source of both screenshots and the a11y tree.
- Observed JSON-RPC method set: `list_apps`, `list_windows`, `get_window`, `launch_app`, `get_window_state`, `click`, `click_element`, `scroll`, `drag`, `press_key`, `type_text`, `perform_secondary_action`, `set_value`, `activate_window`, plus lifecycle `ping`, `close`, `end_turn`.

mac — `dist\project\cua\sky_js\src\targets\mac\native-pipe.js` (class `MacNativePipeTransport`):
- Connects to a Unix domain socket at `~/Library/Group Containers/2DC432GLL2.com.openai.sky.CUAService/IPC/computeruse.sock` (overridable via `SKY_CUA_NATIVE_PIPE_PATH`).
- If the socket is absent, it bootstraps the service: either an `ensureService {service:"computer-use"}` JSON-RPC over `NODE_REPL_HOST_SERVICES_PIPE_PATH`, or LaunchServices `openApplication` of `com.openai.sky.CUAService` / `<CODEX_HOME>/computer-use/Codex Computer Use.app`.
- Wire format: LENGTH-PREFIXED framing (4-byte little-endian uint32 length + UTF-8 JSON), max frame 8 MB, JSON-RPC 2.0.
- Version handshake: `ping` requires `serverApiVersion === clientApiVersion`, else throws `incompatibleClientVersion` — client and native service are version-locked.

mac policy engine — `dist\project\cua\sky_js\src\targets\mac\computer-use-policy.js` (`withComputerUsePolicy`):
- Calls `getClient().getAppPolicy(app)` -> decision `allowed | denied | forbidden`. `denied` = "blocked ... by your organization's policy"; `forbidden` = "not allowed ... for safety reasons" (a hard safety denylist). Only `allowed` proceeds.
- Then the same approval elicitation ("Allow Computer Use to use \"<app>\"?") with `riskLevel`, optional `warningSubtitle`, persist `["session","always"]` or `["session"]`, and telemetry (`logComputerUseToolCalled/ApprovalRequested/ApprovalResolved`). Stamps response meta `codex/toolSurface {kind:"computerUse", app:{appId,kind:"appId"}}`.


## 5. Model integration — "code mode", not a tool-call loop

This is the biggest architectural divergence from the public computer-use API and from my earlier Layer-B assessment.

- The model is NOT given discrete `computer_call` tools. It is given ONE code-execution tool: a Node REPL exposed as `js` (tool id typically `mcp__node_repl__js`; `js_reset` clears state). Source: plugin `skills\computer-use\SKILL.md` + `docs\guidance.md`.
- The model bootstraps once per session:
  ```js
  if (!globalThis.sky) {
    const { setupComputerUseRuntime } = await import("<plugin root>/scripts/computer-use-client.mjs");
    await setupComputerUseRuntime({ globals: globalThis });
  }
  globalThis.apps = await sky.list_apps();
  ```
  then writes ordinary async JS that calls `sky.list_apps()`, `sky.get_window({id,app})`, `sky.activate_window({window})`, `sky.get_window_state({window, include_text:true})`, `sky.click({window, element_index})`, etc, keeping `sky/apps/targetApp/targetWindow/state` on `globalThis` across cells (the REPL session is persistent; top-level `const` cannot be redeclared).
- So the agent loop is: bootstrap -> list apps -> pick app/window -> activate -> get_window_state (screenshot emitted into context + a11y tree) -> reason -> write JS to click/type/etc -> get_window_state again -> ... This is code-mode agentic control (loops, filtering, retries happen IN the model's JS), which is strictly more expressive than one-action-per-turn. It is hosted by `codex-code-mode-host.exe`.
- The plugin is registered as a Codex plugin (`.codex-plugin\plugin.json`, name `computer-use`, version `26.707.31428`, `homepage` github.com/openai/openai/tree/master/project/cua/sky_js/plugin) with a bundled `skills/` skill the model MUST read before Windows automation.


## 6. Perception model

The model perceives the target through BOTH channels simultaneously, per `get_window_state`:
- Pixels: an occlusion-proof WGC screenshot (JPEG), pushed into context as an `input_image` via `emitImage`. Bounded, z-ordered regions capture transient UI (menus/popups) as separate layers.
- Structure: a UIA accessibility tree rendered as indexed text (`tree`, `focused_element`, `selected_text`, `selected_elements`, `document_text`), with a per-call DIFF option to send only what changed.
The a11y tree is why actions can target `element_index` (robust) instead of raw `x,y` (brittle). The bundled `tesseract.js` (OCR) and `pixelmatch` (frame diff) give the JS layer additional screenshot understanding/change-detection tools on top of the model's own vision.


## 7. Safety & permission model (defense in depth, enforced OUTSIDE the model)

1. Org policy gate: `allowed | denied | forbidden` per app (mac `getAppPolicy`; `forbidden` = hard safety denylist).
2. Per-app user consent: blocking elicitation before first use of an app, persistable `session` or `always`; the native helper will not act until it sees the approved-app meta stamp.
3. Physical Escape kill switch: SIGINT / interrupt-marker file aborts the whole turn with a fixed "stop, do not call Computer Use again this turn" message.
4. Per-turn time budget: `x-oai-cua-request-budget-ms` (10 s default) bounds each request; overrun kills the helper.
5. Version lock: client/native API-version handshake (`incompatibleClientVersion`).
6. Written confirmations policy the MODEL must obey (plugin `docs\confirmations.md`), two enforcement tiers:
   - Hand-Off Required (agent must NOT do it, hand back to user): final submit of a password change [2.4]; bypassing Windows/browser safety barriers, HTTPS "not secure" interstitials, paywalls [15].
   - Always Confirm at action-time even if pre-approved: deleting cloud/local data [1]; editing cloud permissions / finalizing account creation / creating API-OAuth keys / saving passwords or cards in browser [2.x]; solving CAPTCHAs [4]; installing/running newly acquired software or browser extensions [8.3-8.5]; representational communication to third parties (posts/messages/forms/reservations, high-stakes submissions) [9]; subscribe/unsubscribe [10]; financial transactions [11]; changing local system settings via a browser action [13].
7. Prompt-injection stance (confirmations.md "Types of Instruction"): user-authored prompt text = valid intent even if high-risk; user-supplied third-party content (pasted/quoted/uploaded/website) = potentially malicious and NEVER permission by itself. Plus an explicit sensitive-data + "transmission" definition (typing sensitive data into a form counts as transmission).
8. Anti-evasion guidance (guidance.md): the model is told NOT to fall back to PowerShell/SendKeys/shell foreground automation, NOT to spawn `codex-computer-use.exe` directly, NOT to import `@oai/sky` directly, NOT to build a custom helper protocol client — because app approvals + Escape interruption only work through the sanctioned `scripts/computer-use-client.mjs` path. (Directly relevant to anyone trying to drive the helper standalone: you would bypass exactly those safety mediations.)


## 8. How to drive / confirm it yourself (repro)

Passive (what this investigation did): read the deployed runtime under `%LOCALAPPDATA%\OpenAI\Codex\runtimes\cua_node\...\bin\node_modules\@oai\sky\` — docs + JS are all there.

Active options (not done here; all local, owner-consented):
- In-app: run a Computer Use task and read the live logs / SSB the app writes under `%LOCALAPPDATA%\OpenAI\Codex\`.
- Standalone driver: `node` the sky client and call `sky.list_apps()`. NOTE the guidance forbids importing `@oai/sky` directly and spawning the helper directly precisely because that bypasses approval + Escape mediation; a faithful driver must go through `scripts/computer-use-client.mjs` (`setupComputerUseRuntime`).
- Wire capture (Windows): wrap `codex-computer-use.exe`'s stdio (the transport is plaintext newline-JSON) to log the exact JSON-RPC method/params/results the model drives — the cleanest way to see the live protocol without disassembly.
- Native depth (only if needed): ETW/API Monitor on the helper to observe the WGC/UIA/SendInput calls, or Ghidra on the 1.69 MB PE.


## 9. What is NOT recovered (honest boundary)

- The interior of `codex-computer-use.exe` / the mac CUAService (exact WGC capture path, UIA property projection, SendInput sequencing, element-index assignment algorithm). Compiled native; needs disasm/trace. Its external contract IS fully recovered (section 4).
- The CUA decision model itself (the GPT-5.6-class policy that chooses actions). Hosted, closed weights — black-box only, not reverse-engineerable by definition. What IS recovered is exactly how its actions are executed and how perception is fed to it.
- The Electron `app.asar` (199 MB) internals and the Rust `codex.exe` internals — not needed for the Computer Use feature and not opened.


## 10. Evidence index (files read on this machine)

Running processes: `ChatGPT.exe`, `codex` (from `...\app\resources\codex.exe`), `node_repl` (from `...\runtimes\cua_node\ecfc0d9aa02807e3\bin\node_repl.exe`).
- `...\runtimes\cua_node\ecfc0d9aa02807e3\manifest.json` — runtime pins (node 24.14.0, cua-node 0.0.3).
- `...\@oai\sky\package.json` — v0.4.20, main = dist/project/cua/sky_js/src/index.js, `test:windows-wgc`.
- `...\@oai\sky\docs\sky-full-desktop-api.md` / `sky-window-api.md` / `sky-window2-api.md` — the 3 API surfaces.
- `...\@oai\sky\dist\project\cua\sky_js\src\targets\windows\internal\helper_transport.js` — Windows spawn+stdio JSON-RPC, budget, Escape interrupt, app-approval handshake.
- `...\targets\windows\internal\computer_use_client_base.js` — window2 method surface, click vs click_element, emitImage.
- `...\targets\mac\native-pipe.js` — mac Unix-socket length-prefixed JSON-RPC 2.0, service bootstrap, version handshake.
- `...\targets\mac\computer-use-policy.js` — allowed/denied/forbidden policy + approval elicitation + telemetry.
- `...\@oai\sky\bin\windows\codex-computer-use.exe` (1.69 MB) — native helper (identified, not disassembled).
- App bundle `...\app\resources\plugins\openai-bundled\plugins\computer-use\`: `.codex-plugin\plugin.json`, `docs\api.md`, `docs\confirmations.md`, `docs\guidance.md`, `skills\computer-use\SKILL.md`.
- App bundle `...\app\resources\` listing: `codex.exe`, `codex-code-mode-host.exe`, `codex-command-runner.exe`, `codex-windows-sandbox-setup.exe`, `native\*.node`, `cua_node\`, `accessibility\`, `plugins\`.
