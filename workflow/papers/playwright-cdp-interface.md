Source: Playwright Python + Chrome DevTools Protocol (CDP) + playwright-mcp
Reliability: HIGH (official playwright.dev docs, crates.io verified, github.com/microsoft)
Session: s-31180
Task: B-24 Phase 4I.7-4I.9 WebRenderAdapter
Files examined:
  https://playwright.dev/python/docs/api/class-browsertype
  https://playwright.dev/python/docs/api/class-browser
  https://playwright.dev/python/docs/api/class-browsercontext
  https://playwright.dev/python/docs/api/class-page (partial — page too large to scrape in full)
  https://playwright.dev/python/docs/api/class-cdpsession
  https://playwright.dev/python/docs/library
  https://playwright.dev/python/docs/screenshots
  https://playwright.dev/python/docs/emulation
  https://github.com/microsoft/playwright-mcp (README)
  https://crates.io/crates/playwright (cargo search verified)
  https://crates.io/crates/playwright-rs (cargo info verified)

Public API:

SECTION 1 — HEADLESS CHROMIUM SPAWN
chromium.launch(**kwargs) -> Browser
  headless: bool = True
  args: List[str] = []  (e.g. ["--disable-dev-shm-usage", "--no-sandbox"])
  channel: str = None   (e.g. "chrome", "msedge", "chrome-beta"; omit for bundled Chromium)
  executable_path: Union[str, Path] = None
  timeout: float = 30000  (ms; startup max wait)
  slow_mo: float = None   (ms delay between each operation)
  proxy: Dict[server, bypass, username, password] = None
  env: Dict[str, Union[str, float, bool]] = None
  ignore_default_args: Union[bool, List[str]] = False
  downloads_path: Union[str, Path] = None
  artifacts_dir: Union[str, Path] = None

browser.new_context(**kwargs) -> BrowserContext
  viewport: Dict[width: int, height: int] | None = {"width": 1280, "height": 720}
  user_agent: str = None
  device_scale_factor: float = 1
  is_mobile: bool = False
  has_touch: bool = False
  color_scheme: "light" | "dark" | "no-preference" | "null" = "light"
  locale: str = None
  timezone_id: str = None
  geolocation: Dict[latitude, longitude, accuracy] = None
  permissions: List[str] = None
  ignore_https_errors: bool = False
  java_script_enabled: bool = True
  bypass_csp: bool = False
  extra_http_headers: Dict[str, str] = None
  http_credentials: Dict[username, password] = None
  offline: bool = False
  accept_downloads: bool = True
  no_viewport: bool = False   (disables fixed viewport entirely)
  base_url: str = None
  proxy: Dict = None

browser_context.new_page() -> Page
  (no parameters; inherits context settings)

SECTION 2 — page.screenshot()
page.screenshot(**kwargs) -> bytes
  path: Union[str, Path] = None   (if set, saves to file AND returns bytes)
  type: "png" | "jpeg" = "png"
  quality: int = None             (only for jpeg; 0-100)
  full_page: bool = False         (captures full scrollable page vs viewport)
  clip: Dict = None               (sub-fields: x: float, y: float, width: float, height: float)
  omit_background: bool = False   (removes white background; PNG only, not JPEG)
  timeout: float = 30000          (ms)
  animations: "disabled" | "allow" = "allow"
  caret: "hide" | "initial" = "hide"
  scale: "css" | "device" = "device"
  mask: List[Locator] = None      (elements to mask with solid color overlay)
  mask_color: str = None          (CSS color; default is black #FF00FF in some versions)
  style: str = None               (CSS stylesheet applied during capture only)
RETURN: bytes (raw binary; Python has NO encoding= parameter — JS-only feature)
NOTE: to get base64, caller must: base64.b64encode(page.screenshot()).decode()
NOTE: omit_background=True incompatible with type="jpeg" — silently ignored by Playwright

SECTION 3 — VIEWPORT API
browser.new_context(viewport={"width": W, "height": H})  # preferred for multi-viewport runs
page.set_viewport_size({"width": W, "height": H}) -> None  # per-page override after creation
page.viewport_size -> Dict[width: int, height: int] | None  # read current size
Multi-viewport pattern for {375, 768, 1440}:
  context_mobile  = browser.new_context(viewport={"width": 375,  "height": 667})
  context_tablet  = browser.new_context(viewport={"width": 768,  "height": 1024})
  context_desktop = browser.new_context(viewport={"width": 1440, "height": 900})
NOTE: set_viewport_size() on a page overrides the context viewport for that page only
NOTE: no_viewport=True disables fixed viewport (page uses its natural scroll size)

SECTION 4 — WAIT STRATEGIES
page.wait_for_load_state(state="load", timeout=30000) -> None
  state: "load" | "domcontentloaded" | "networkidle" = "load"
  timeout: float = 30000  (ms; pass 0 to disable)
  NOTE: "networkidle" = no pending network connections for >= 500ms; SLOW and unreliable on SPAs
  NOTE: networkidle discouraged for pages with polling/analytics/long-poll connections

page.wait_for_selector(selector, state="attached", timeout=30000, strict=None) -> ElementHandle
  selector: str  (required; CSS or XPath)
  state: "attached" | "detached" | "visible" | "hidden" = "attached"
  timeout: float = 30000
  strict: bool = None  (True = error if selector matches multiple elements)

page.goto(url, timeout=30000, wait_until="load", referer=None) -> Response | None
  wait_until: "load" | "domcontentloaded" | "networkidle" | "commit" = "load"

page.set_default_timeout(timeout: float) -> None
page.set_default_navigation_timeout(timeout: float) -> None
browser_context.set_default_timeout(timeout: float) -> None
browser_context.set_default_navigation_timeout(timeout: float) -> None
  (page-level takes priority over context-level)

SECTION 5 — INTERACTION API
page.click(selector, button="left", click_count=1, delay=None, force=None, position=None,
           timeout=30000, trial=None, strict=None, no_wait_after=None) -> None
  button: "left" | "right" | "middle" = "left"
  click_count: int = 1
  delay: float = None  (ms between mousedown and mouseup)
  force: bool = None   (bypass actionability checks)
  position: Dict[x: float, y: float] = None  (click offset from element top-left)

page.type(selector, text, delay=None, timeout=30000, strict=None, no_wait_after=None) -> None
  text: str  (characters to type one-by-one)
  delay: float = None  (ms between keystrokes)
  NOTE: use page.fill() for instant fill without key-by-key simulation

page.hover(selector, force=None, position=None, timeout=30000, trial=None, strict=None) -> None

page.evaluate(expression: str, arg=None) -> Any
  expression: str  (JavaScript expression or function body)
  arg: Any = None  (serializable argument passed to the function)
  RETURN: JSON-serializable result
  NOTE: non-serializable return values silently become None

SECTION 6 — CLEANUP
browser.close(reason=None) -> None  (closes browser and ALL its pages and contexts)
browser_context.close(reason=None) -> None  (closes context and all its pages)
page.close(reason=None, run_before_unload=False) -> None
playwright.stop() -> None  (or use context manager: avoids need to call stop())

Correct cleanup order:
  1. page.close()
  2. browser_context.close()
  3. browser.close()
  4. playwright.stop() / exit context manager

Context manager pattern (preferred — prevents zombie processes):
  SYNC:  with sync_playwright() as p: ...
  ASYNC: async with async_playwright() as p: ...

SECTION 7 — PROCESS MODEL
Playwright Python spawns a Node.js Playwright driver subprocess via stdio (JSON-RPC).
The driver then spawns Chromium as a child of the driver process.
Process tree: Python -> playwright-driver (Node.js) -> chromium

Windows requirement: asyncio.ProactorEventLoop is REQUIRED for async subprocesses.
  Playwright sets this automatically on Python 3.7+ on Windows.
  Do NOT use SelectorEventLoop on Windows with async Playwright.

Zombie process risk (verified GitHub issues #34190, #26980):
  - playwright.close() can create zombie playwright-driver processes on Linux/Debian
  - Ctrl+C early exit leaves Chromium children alive in some versions
  - If SIGTERM ignored by driver, browser children may survive

Safe kill on timeout from Rust subprocess wrapper:
  1. Send SIGTERM to the playwright-driver PID
  2. Wait 2s for graceful shutdown
  3. If still alive, send SIGKILL to the process GROUP (kill(-pgid, SIGKILL))
  NOTE: kill by group ID required to catch Chromium grandchild processes
  NOTE: On Windows: TerminateProcess() on the driver handle; Chromium children terminate as job objects

Thread safety: Playwright API is NOT thread-safe.
  One Playwright instance per thread. Use separate instances for concurrent rendering.

SECTION 8 — CDP (Chrome DevTools Protocol) ACCESS
page.context.new_cdp_session(page) -> CDPSession   (Chromium only)
  NOTE: only available for Chromium-based browsers; Firefox/WebKit not supported

cdp_session.send(method: str, **params) -> Dict
  method: str  (CDP command, e.g. "Animation.enable", "Network.enable", "Page.captureScreenshot")
  params: Dict  (CDP method parameters per protocol spec)
  RETURN: Dict (CDP response)

cdp_session.on(event: str, callback) -> None
  event: str  (CDP event name, e.g. "Network.responseReceived")

cdp_session.detach() -> None  (disconnects; session emits no more events)

SECTION 9 — RUST INTEGRATION
Option A: playwright crate (crates.io/crates/playwright, version 0.0.20)
  Repo: github.com/octaltree/playwright-rust
  Status: CAUTION — version 0.0.20, pre-1.0, unknown maintenance status in 2026
  Runtime: tokio (rt-tokio feature) OR actix-rt OR async-std

Option B: playwright-rs crate (crates.io/crates/playwright-rs, version 0.12.3)
  Repo: github.com/padamson/playwright-rust
  Status: ACTIVE — last published 2026-04-30, rust-version 1.88, bundles Playwright driver 1.59.1
  Runtime: tokio (required)
  Architecture: JSON-RPC over stdio to bundled Playwright Node.js driver (same as Python bindings)
  Features: native-tls (default), rustls-tls-native-roots, rustls-tls-webpki-roots
  API pattern:
    let pw = Playwright::launch().await?;
    let browser = pw.chromium().launch().await?;
    let page = browser.new_page().await?;
    page.goto("https://example.com", None).await?;
  Screenshot: available via page methods (exact Rust signature not verified from source)
  CAUTION: pre-1.0, API may change; verify screenshot() param names against Playwright 1.59.x

Option C: subprocess invocation of `npx playwright` CLI
  Invoke: std::process::Command::new("npx").args(["playwright", "screenshot", URL, OUT])
  Reliable fallback — no Rust crate dependency
  Suitable for Tier C isolated process model if Node.js is available on the host

Option D: Playwright MCP server (github.com/microsoft/playwright-mcp, official Microsoft)
  Transport: stdio (default) or HTTP+SSE (--port 8931)
  Invocation: npx -y @playwright/mcp@latest  (or via Docker)
  Tools exposed: browser_navigate, browser_take_screenshot, browser_click, browser_type,
    browser_hover, browser_evaluate, browser_wait_for, browser_snapshot, browser_resize,
    browser_close, browser_network_requests, browser_pdf_save, and ~20 more
  Use from Rust: spawn as child process via stdio, speak MCP JSON-RPC protocol
  Headless: requires --headless flag (default is headed for interactive use)
  Isolation: --isolated flag for concurrent sessions (avoids profile conflicts)
  NOTE: "does not serve as a security boundary" per official docs
  NOTE: accessibility-tree based (not pixel-based); for pixel-exact screenshots, use direct API

Behavioral contracts:
  browser.close() -> terminates browser process and all open pages (synchronous teardown)
  browser_context.close() -> closes context; pages emit close events; waits for all pages to close
  page.close() -> closes single page; if last page in context, context remains open
  playwright.stop() -> terminates the playwright driver subprocess
  page.screenshot() -> always returns bytes; path= additionally writes file; both happen
  page.set_viewport_size() -> mutates page state; no return value; takes effect immediately
  page.evaluate() -> runs in browser main thread; blocks until JS completes or timeout fires
  CDPSession.send() -> synchronous round-trip to browser CDP endpoint; returns on response
  wait_for_load_state("networkidle") -> waits until 500ms of no network; can hang if page polls

Platform constraints:
  Windows async: must use ProactorEventLoop (asyncio); Playwright sets automatically on Python 3.7+
  Windows sync: no special event loop requirement
  Linux headless: requires --no-sandbox arg when running as root in container
  Linux headless: shared memory (/dev/shm) can be exhausted; add --disable-dev-shm-usage to args
  Viewport: set_viewport_size() requires page to be attached (not closed)
  Concurrent contexts: each context is isolated (cookies, storage, auth); safe to run concurrently
  Thread safety: NOT thread-safe; one Playwright instance per OS thread
  playwright-rs on Windows: requires Node.js installed on PATH for the bundled driver

Gotchas:
  omit_background=True with type="jpeg" silently ignored (JPEG has no alpha channel)
  page.screenshot() has NO encoding parameter in Python — returns bytes only; JS has encoding="base64"
  wait_for_load_state("networkidle") can time out on SPAs with keepalive XHR or WebSocket connections
  networkidle definition: 0 pending requests for >= 500ms continuously
  page.type() simulates keystrokes one char at a time; use page.fill() for fast input
  browser.new_page() creates page in a NEW default context; use context.new_page() for shared context
  clip dict x/y are relative to viewport top-left (not document top)
  full_page=True with clip is an error (mutually exclusive)
  Playwright MCP default is headed mode; add --headless for server-side use
  playwright-mcp concurrent profiles conflict unless --isolated or separate --user-data-dir
  playwright-rs bundles specific Playwright version (1.59.1); must install matching browser: playwright install
  zombie processes on Linux: use process group kill (SIGKILL pgid) not single PID
  On Python, context manager (with sync_playwright()) is the only zombie-safe pattern
  playwright.stop() must be called if NOT using context manager — or zombie driver process persists
  CDPSession only works with Chromium-based browsers; calling on Firefox/WebKit raises error

Critical assertions (testable):
  # screenshot returns non-empty bytes
  assert isinstance(page.screenshot(), bytes)
  assert len(page.screenshot()) > 0

  # viewport size after set_viewport_size reflects new dimensions
  page.set_viewport_size({"width": 375, "height": 667})
  assert page.viewport_size == {"width": 375, "height": 667}

  # omit_background with jpeg does NOT raise but result is non-transparent
  # (test: set type="jpeg", omit_background=True — should complete without error)

  # full_page and clip cannot both be set (Playwright raises ValueError or ignores clip)

  # wait_for_load_state with state="load" completes before state="networkidle" for same page

  # browser.close() makes page.is_closed() return True for all pages in browser
  assert page.is_closed()  # after browser.close()

  # CDPSession.detach() makes subsequent send() raise an error
  # after detach: try cdp_session.send("Network.enable") -> must raise

Complexity:
  page.screenshot() -> O(viewport area) time, O(image bytes) memory; JPEG typically 5-15x smaller than PNG
  page.screenshot(full_page=True) -> O(document height * width) time; can be very slow on tall pages
  wait_for_load_state("networkidle") -> bounded by timeout (default 30s); worst case = timeout
  page.evaluate(js) -> bounded by JS execution time; no timeout separate from page default
  CDPSession.send() -> one round-trip to Chromium; typically < 10ms for simple commands
  Concurrency: each browser.new_context() is independent; multiple contexts can screenshot in parallel
    across different pages; limited by Chromium process CPU and RAM

Failure modes:
  Launch timeout (30s default): Chromium binary not found, PATH issue, or slow disk I/O on startup
  Screenshot timeout (30s default): page still loading; use wait_for_load_state before screenshot
  networkidle hang: page has persistent XHR/WebSocket; set explicit short timeout or switch to "load"
  Zombie playwright-driver: missing playwright.stop() call; missing context manager; SIGKILL bypass
  Zombie chromium: SIGTERM to driver without killing process group; chromium children survive
  OOM on full_page screenshot: very long pages; Chromium renderer crashes; returns empty bytes
  SelectorError: wait_for_selector with strict=True when multiple elements match
  CDPSession on non-Chromium: raises playwright._impl._errors.Error with "CDP session not supported"
  Port conflict (playwright-mcp HTTP mode): another process on --port 8931; server fails to start
  Profile conflict (playwright-mcp): two instances share same user-data-dir; second fails to lock
  Windows SelectorEventLoop: async Playwright raises RuntimeError about subprocess not supported
  evaluate() returning non-serializable object: silently returns None (no error raised)

Implementation pitfalls (do NOT do this):
  Do NOT call page.screenshot() before page.goto() completes — returns blank/loading page image
  Do NOT use wait_for_load_state("networkidle") as primary wait for SPA renderers — use wait_for_selector
  Do NOT reuse Browser instance across OS threads — not thread-safe; create new Playwright per thread
  Do NOT call playwright.stop() inside the with sync_playwright() block — double-stop raises
  Do NOT send SIGKILL only to playwright-driver PID — Chromium grandchildren remain as zombies
  Do NOT set omit_background=True with type="jpeg" — silently a no-op, wastes caller expectation
  Do NOT use page.type() for bulk text input — extremely slow; use page.fill() instead
  Do NOT set full_page=True and clip simultaneously — undefined behavior (implementation-dependent)
  Do NOT use playwright crate (0.0.20) for production — maintenance status unknown; use playwright-rs 0.12.3
  Do NOT assume playwright-rs screenshot() param names match Python exactly — verify against bundled version
  Do NOT use Playwright MCP as a security boundary — official docs explicitly state it is not one
  Do NOT use browser.new_page() when you need shared cookies — use context.new_page() instead
  Do NOT skip browser.close() / context.close() on timeout paths — always clean up in finally/defer
  Do NOT assume page.evaluate() timeout is separate from page default timeout — it uses the same setting
