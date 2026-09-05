Source: lavish-axi
Reliability: MEDIUM
Files examined:
  docs/.source/lavish-axi/src/artifact-sdk.js
  docs/.source/lavish-axi/src/server.js
  docs/.source/lavish-axi/src/html-transform.js
  docs/.source/lavish-axi/src/chrome-client.js
  docs/.source/lavish-axi/skills/lavish/SKILL.md
  docs/.source/lavish-axi/package.json

Language: JavaScript (ESM, Node >=22). Runtime: Node (NOT bun — uses node --test runner directly).
Version: 0.1.31. Package manager: pnpm@11.1.1. License: MIT.
Entry: bin/lavish-axi.js -> dist/cli.mjs (built via esbuild via scripts/build.js).

Dependencies (from package.json, exact versions verified):
  @tailwindcss/browser 4.2.4
  axi-sdk-js ^0.1.7
  chokidar ^4.0.3
  daisyui ^5.5.19
  express ^5.2.1
  open ^10.2.0

Public API:
  serve({ port, stateFile, version, debug, log, pollHeartbeatMs, idleTimeoutMs, host, linkHost }) | Promise<{port,close,done}> | starts express server, returns port + close handle
  resolveIdleTimeoutMs(env) | number|null | reads LAVISH_AXI_IDLE_TIMEOUT_MS; returns 30*60_000 default, null if disabled
  injectLavishSdk(html, key) | string | injects <script src="/sdk.js?key=..."> before </body>
  createArtifactSdk(deriveQueueKey, isNativeInteractive) | void | attaches annotation SDK into artifact iframe's window
  deriveLavishQueueKey(element, options) | string | derives stable queue-collapse key for a DOM element
  isNativeInteractiveControl(el) | boolean | returns true for button/input/select/textarea/label/summary/contenteditable
  createChromeHtml(session, {layoutGateEnabled}) | string | generates the outer chrome iframe HTML
  createSdkJs(key) | string | serializes createArtifactSdk + deriveLavishQueueKey as inline IIFE for /sdk.js route
  resolveWatchTarget(session) | Promise<{path,scope,options}> | file-only or directory watch; directory opt-in via data-lavish-live-reload-root or <meta name="lavish-live-reload" content="root">
  hasLiveReloadRootOptIn(html) | boolean | regex-based check for live-reload-root opt-in markers
  resolveArtifactAsset(root, assetPath) | string|null | path-traversal guard: resolves asset path under artifact dir, returns null if escapes root
  computePresence(key, activePolls, deliveredFeedback) | "listening"|"working"|"waiting" | derives agent presence state
  displayPathParts(file, home) | {head, tail} | splits path for display, collapses home to ~
  shouldEnableLayoutGate(query) | boolean | reads no-gate / gate query params
  shouldDisableLayoutGateOpen(body) | boolean | reads no-gate / gate from POST body

HTTP routes (expressed server, port from config):
  GET  /health                       -> {ok:true,app:"lavish-axi",version}
  POST /shutdown                     -> {status:"shutting-down"}; deferred shutdown
  POST /api/sessions                 -> {key,file,url,status:"opened"}; body: {file, "no-gate"?}
  GET  /api/poll?file=&timeoutMs=    -> long-poll; returns {status:"feedback"|"waiting"|"ended"|...}; timeoutMs=null = streaming heartbeat
  POST /api/:key/prompts             -> {status:"queued",pending_prompts:N}; body: prompt object
  POST /api/:key/layout-warnings     -> {status:"recorded",layout_warnings:N}; body from artifact SDK
  POST /api/:key/end                 -> {status:"ended"}
  POST /api/:key/agent-reply         -> {status:"sent"}; body: {text}
  POST /api/end                      -> {status:"ended"}; body: {file}
  GET  /session/:key                 -> chrome HTML page wrapping artifact in iframe
  GET  /artifact/:key/index.html     -> artifact HTML with SDK injected
  GET  /artifact/:key/:assetPath     -> static file server for artifact assets (path-traversal-guarded)
  GET  /events/:key                  -> SSE; events: reload, agent-reply, agent-presence, chat-sync, chrome-reload
  GET  /chrome-client.js             -> browser chrome JS
  GET  /chrome.css                   -> chrome styles
  GET  /design/:asset                -> daisyui.css | daisyui-themes.css | tailwindcss-browser.js (tries packaged then node_modules)
  GET  /sdk.js?key=                  -> serialized artifact SDK IIFE

ROUND-TRIP PROTOCOL (verbatim from source):

Step 1 — Agent creates HTML artifact, writes to .lavish/<name>.html.

Step 2 — Agent runs: npx -y lavish-axi <html-file>
  Server spawns in background (or reuses running server).
  POST /api/sessions {file} -> returns {key, url}.
  Browser opens to /session/:key.

Step 3 — Agent runs: npx -y lavish-axi poll <html-file>
  GET /api/poll?file=<path> (long-poll, stays open until feedback).
  Returns one of:
    {status:"waiting"}                  — nothing yet
    {status:"feedback", prompts:[...], layout_warnings:[...]} — user annotated or layout issue
    {status:"ended"}                    — user ended session

  Poll response shape (feedback):
    prompts: array of prompt objects from the iframe postMessage queue
    layout_warnings: array of {selector, kind, overflowPx, viewportWidth, severity}
    layout_warning kinds: page-horizontal-overflow | element-scroll-overflow | element-parent-overflow | clipped-text | overlapping-text

Step 4 — Agent fixes layout_warnings (clipped text, overflow) before asking human.

Step 5 — Agent applies human annotations, then runs:
  npx -y lavish-axi poll <html-file> --agent-reply "<message>"
  POST /api/:key/agent-reply {text} — appears in browser chat panel.

Step 6 — Agent runs: npx -y lavish-axi end <html-file>
  POST /api/end {file} -> {status:"ended"}

ANNOTATION FLOW (inside iframe, artifact-sdk.js):

Artifact iframe loads /sdk.js?key=<key> (injected by injectLavishSdk).
SDK attaches to iframe window; chrome listens via parent.postMessage.

User hovers/clicks non-interactive elements -> highlight + annotation card pops.
User selects text -> text-range annotation card.
User types feedback in card + clicks Queue (or presses Enter) ->
  queuePrompt(prompt, options) -> parent.postMessage({type:"lavish:queuePrompt", prompt:item})
  item shape: {uid, selector, tag, text, prompt, target?, _lavishQueueKey?, data?}

User clicks Send or presses Cmd/Ctrl+Enter ->
  sendQueuedPrompts() -> parent.postMessage({type:"lavish:sendQueuedPrompts"})
  Chrome batches queued items, POST /api/:key/prompts -> events.emit("feedback", key) -> poll returns

_lavishQueueKey collapse: derived per element (radio:scope:name | checkbox:scope:identity:option |
  field:scope:identity | question:<data-lavish-question> | ""); used to collapse in-flight updates
  for the same input so only the latest value is sent.

LAYOUT GATE:
On session open, chrome shows a "Checking layout" overlay.
Artifact iframe loads, SDK runs: waitForDocumentFontsReady -> waitForResizeObserverSettle (180ms settle, 2s max)
  -> 2 animation frames -> auditLayout().
auditLayout() detects: page-horizontal-overflow, element-scroll-overflow, element-parent-overflow,
  clipped-text (overflow:hidden/clip on text), overlapping-text (elementFromPoint probe).
Results posted via parent.postMessage({type:"lavish:layoutWarnings", layout_warnings}).
Chrome sends POST /api/:key/layout-warnings.
If warnings exist, chrome shows banner: "This surface may have layout issues. Your agent has been notified."
Poll returns layout_warnings alongside any queued prompts.
Gate reveals artifact after fonts/resize settle (max 12s, configurable via layoutGateMaxHoldMs).

DESIGN SYSTEM INHERITANCE (from SKILL.md verbatim):
"Lavish does not auto-inject any design system - artifacts stay portable so they render identically
when opened directly without lavish-axi running. Before writing any HTML, decide the design direction
in this strict priority order: (1) if the user asked for a specific look or named design system, use that;
(2) otherwise you must first inspect the project the artifact is about... and match that project's
design system; (3) only when both steps come up empty, use the Lavish-recommended Tailwind CSS browser
runtime v4 + DaisyUI v5, available via CDN."

_ds_bundle.js and _ds_manifest.json (in .agents/skills/lavish-design/):
  Bundled design reference artefact baked into the skill; served as _ds_bundle.js from the
  lavish-design skill directory. Contains colors, typography tokens, component CSS for the Lavish
  brand (ink/steel/cream/brass palette, Geist/Geist Mono fonts, specific radius/shadow values).
  This is the author's own design system, NOT the user-project's system.

Behavioral contracts:
  serve() -> starts chokidar watcher per session file; emits "reload" on change (100ms debounce)
  idle self-shutdown -> arms 30min timer when no SSE clients and no active polls; cancels on connection
  shutdown -> SSE clients receive "chrome-reload" event before drop; server closes http + watchers
  session state -> persisted in stateFile (JSON, via SessionStore); sessions survive server restart
  feedback delivery -> once feedback delivered, marks deliveredFeedback set; presence -> "working"
  agent-presence SSE event -> "listening" (poll active, no delivered feedback) | "working" (feedback delivered) | "waiting" (no poll)

Platform constraints:
  Node >=22 (package.json engines). ESM throughout (type:"module").
  paths.js uses bindHost/linkHost for 0.0.0.0 vs 127.0.0.1 distinction.
  chokidar v4 (breaking change from v3): watch API unchanged but v4 dropped dependency on fsevents.
  Artifact iframe: sandboxed (allow-scripts allow-forms allow-popups allow-downloads), no same-origin.
    Chrome cannot read iframe scrollY directly -> iframe posts lavish:scroll on scroll.
  Session state survives server restart via stateFile (state.json by default); server adopts existing sessions on restart.

Tests:
  test/artifact-sdk.test.js        -> unit tests for deriveLavishQueueKey + createArtifactSdk (jsdom-free, custom DOM stubs)
  test/chrome-client-queue.test.js -> queue collapse + ordering
  test/html-transform.test.js      -> injectLavishSdk insertion
  test/server.test.js              -> server route integration tests
  test/session-store.test.js       -> SessionStore CRUD
  test/skill.test.js               -> skill build verification
  test/paths.test.js               -> bindHost/linkHost logic
  test/telemetry.test.js           -> telemetry event shape
  Runner: node --test (Node built-in test runner). No jest/vitest.

Gotchas:
  SKILL.md still says "npx -y lavish-axi" not "bunx" — node-based distribution; bun not used here.
  Design asset fallback: tries packaged path (dist/design/) first, then node_modules source. Both must exist in dev; npm pack bundles packaged copies.
  Layout gate has configurable max hold (default 12s; hard-cap 60s from chrome-client.js:50); artifact hidden behind overlay until layout settles.
  data-lavish-live-reload-root on the artifact's root element opts into DIRECTORY watching (wider, can stall on large trees like ~/Downloads).
  _lavishQueueKey empty string means no queue-collapse; any two prompts with the same non-empty queueKey collapse (latest wins).

Critical assertions (testable):
  assert injectLavishSdk('<html><body>x</body></html>', 'k').includes('/sdk.js?key=k')
  assert injectLavishSdk('<html><body>x</body></html>', 'k').indexOf('/sdk.js') < injectLavishSdk('<html><body>x</body></html>', 'k').indexOf('</body>')
  assert resolveArtifactAsset('/tmp/art', '../etc/passwd') is None
  assert computePresence('k', new Map([['k',1]]), new Set()) === 'listening'
  assert computePresence('k', new Map(), new Set(['k'])) === 'working'
  assert computePresence('k', new Map(), new Set()) === 'waiting'

RELIABILITY: MEDIUM (reviewed source, official public repo, no decompilation)
