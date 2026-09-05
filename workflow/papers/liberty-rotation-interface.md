Source: Liberty free-API rotation subsystem
Reliability: MEDIUM (reviewed source)
Files examined:
  F:\.ELAI_workflow\.liberty\liberty\catalog.py
  F:\.ELAI_workflow\.liberty\liberty\catalog_sync.py
  F:\.ELAI_workflow\.liberty\liberty\router.py
  F:\.ELAI_workflow\.liberty\liberty\quota.py
  F:\.ELAI_workflow\.liberty\liberty\self_heal.py
  F:\.ELAI_workflow\.liberty\liberty\providers_extra.py

Date: 2026-06-20


==============================================================================
SECTION 1: CATALOG (catalog.py + catalog_sync.py)
==============================================================================

--- FreeModel data shape (catalog.py:47-81) ---

FreeModel is a frozen dataclass (immutable once constructed).
Fields (all required unless noted):
  id              str        litellm routing string, e.g. "groq/llama-3.1-8b-instant"
  provider        str        provider slug matching litellm prefix ("groq","cerebras","gemini","mistral","openrouter","venice","uncensored_chat")
  group           str        router model_name group (see below)
  context_window  int        input context in tokens
  max_output      int        max output tokens
  rpm             int        requests per minute (hard limit from provider docs)
  tpm             int|None   tokens per minute (None = not published)
  rpd             int|None   requests per day (None = not published)
  tpd             int|None   tokens per day (None = not published)
  is_free         bool       always True; default True (catalog.py:74)
  cost_input      float      always 0.0; default 0.0
  cost_output     float      always 0.0; default 0.0
  cost_cache_read float      always 0.0; default 0.0
  cost_cache_write float     always 0.0; default 0.0
  notes           str        human notes; default ""

Rust port: model as a struct with Option<u32> for tpm/rpd/tpd.
No heap mutation after construction (frozen=True maps cleanly to Rust owned struct).

--- Group names and membership (catalog.py:100-352) ---

Groups and their static members at 2026-06-16 (insertion order = routing priority within group):

fast-small:
  "groq/llama-3.1-8b-instant"   rpm=30, tpm=6000,   rpd=14400, ctx=131072
  (cerebras/llama3.1-8b REMOVED: 404 on live probe 2026-06-16)

big:
  "groq/llama-3.3-70b-versatile"                      rpm=3,  tpm=12000, rpd=1000,  ctx=131072
  "cerebras/gpt-oss-120b"                              rpm=30, tpm=60000, rpd=14400, tpd=1000000, ctx=131072
  "groq/openai/gpt-oss-120b"                           rpm=3,  tpm=8000,  rpd=1000,  ctx=131072
  "groq/meta-llama/llama-4-scout-17b-16e-instruct"    rpm=3,  tpm=30000, rpd=1000,  ctx=131072
  "groq/qwen/qwen3-32b"                               rpm=3,  tpm=6000,  rpd=1000,  ctx=32768

gemini-flash:
  "gemini/gemini-2.5-flash"      rpm=10, tpm=250000, rpd=20,  ctx=1048576
  "gemini/gemini-2.5-flash-lite" rpm=15, tpm=250000, rpd=500, ctx=1048576

mistral:
  "mistral/mistral-small-latest" rpm=60, tpm=500000, rpd=None, ctx=32768

openrouter-free:
  "openrouter/meta-llama/llama-3.3-70b-instruct:free" rpm=20, tpm=None, rpd=50, ctx=131072

uncensored:
  "openrouter/cognitivecomputations/dolphin-mistral-24b-venice-edition:free" rpm=20, rpd=50, ctx=33000
  "openrouter/nousresearch/hermes-3-llama-3.1-405b:free"                     rpm=20, rpd=50, ctx=131072
  "openai/venice-uncensored"       provider=venice,          rpm=10, rpd=10, ctx=32768
  "openai/uncensored-llama-3.3-70b" provider=uncensored_chat, rpm=20, rpd=None, ctx=131072

Note: "overflow-pollinations" and "overflow-g4f" groups are added by providers_extra.py
at router build time; they do NOT appear in FREE_MODELS.

--- DENYLISTED_MODEL_IDS (catalog.py:36-40) ---

frozenset of confirmed-dead model IDs. Hard-coded at import time.
Current members:
  "cerebras/llama3.1-8b"    (404 on live probe 2026-06-16)
  "cerebras/llama-3.3-70b"  (404 on live probe 2026-06-16)
  "gemini/gemma-3-27b-it"   (404 on live probe 2026-06-16)

Denylist enforcement:
  - catalog.py _register_catalog_into_model_config() skips denylisted IDs (catalog.py:542-545)
  - catalog_sync.py map_to_litellm() rejects them (catalog_sync.py:564-567)
  - catalog_sync.py _load_persisted_sync() skips them on disk reload (catalog_sync.py:716-720)

Rust port: a constant HashSet<&str> checked at registration and sync-ingest time.

--- Fallback reachability mirror (catalog.py:382-388) ---

_FALLBACK_REACHABILITY: dict mirroring router._FALLBACK_CHAIN (to avoid import cycle).
Used only for context-window min-computation (min_context_window_for_group).
  "big":          ["fast-small","gemini-flash","mistral","openrouter-free"]
  "fast-small":   ["gemini-flash","mistral","openrouter-free"]
  "gemini-flash": ["mistral","openrouter-free"]
  "mistral":      ["openrouter-free"]
  "uncensored":   []

--- Catalog helpers (catalog.py:359-499) ---

models_by_group() -> dict[str, list[FreeModel]]
  Partitions FREE_MODELS by group, preserving insertion order.

groups() -> list[str]
  Distinct group names in catalog insertion order.

model_ids() -> list[str]
  All litellm id strings in FREE_MODELS.

get(model_id: str) -> Optional[FreeModel]
  Linear scan of FREE_MODELS by id. Returns None if not found.

all_models() -> list[FreeModel]
  Returns list(FREE_MODELS) — note: may include synced additions (catalog_sync appends in-place).

min_context_window_for_group(group: str) -> int
  Returns the minimum context_window across all models in group PLUS all reachable fallback groups.
  A group with no registered models contributes SAFE_SMALL_CONTEXT_TOKENS (4096).
  Unknown group also returns 4096. Used by compaction logic.

min_context_window_for_model_or_group(name: str) -> int
  If name is a group: delegates to min_context_window_for_group.
  If name is a concrete catalog id: resolves to its group then min_context_window_for_group.
  Otherwise falls back to model_config registry window, or 4096.

SAFE_SMALL_CONTEXT_TOKENS = 4096 (catalog.py:375)

--- catalog_sync.py: runtime extension of FREE_MODELS ---

Data source: cheahjs/free-llm-api-resources README.md, fetched via HTTP GET (no auth).
URL: https://raw.githubusercontent.com/cheahjs/free-llm-api-resources/main/README.md
HTTP timeout: 20 seconds (catalog_sync.py:213)
User-Agent: "Liberty-catalog-sync/1.0"

Persistence file: runs/catalog_sync.json (relative to project root .liberty/)
  Format: {"synced_models": [serialized FreeModel dicts]}
  Written atomically via tmp file + os.replace (catalog_sync.py:153-158)
  Loaded at module import time (_load_persisted_sync runs at bottom of catalog_sync.py:833)

PINNED_IDS (catalog_sync.py:67-78): frozenset of 10 probe-confirmed live IDs.
  Models in PINNED_IDS are NEVER dropped by sync even if cheahjs stops listing them.

_WIRED_PROVIDERS (catalog_sync.py:84): {"cerebras","groq","gemini","mistral","openrouter"}
  Only these providers are active in Liberty.

Key invariant: FREE_MODELS list object is mutated in-place (catalog_sync.py:649).
  _apply_synced_model appends to both _SYNCED_MODELS and FREE_MODELS.
  _remove_synced_model removes from both via list.remove().
  PINNED_IDS entries are never removed (dropped_ids excludes them at catalog_sync.py:774).

Fetch + merge logic (sync_catalog, catalog_sync.py:734-817):
  1. fetch_upstream() -> list[dict] (one record per upstream model)
  2. For each record: map_to_litellm(record) validates:
       a. litellm.get_llm_provider(litellm_id) must succeed and match record["provider"]
       b. For non-openrouter: litellm_id must be in litellm.model_cost
       c. litellm_id must NOT be in DENYLISTED_MODEL_IDS
     Returns None (skipped) if any check fails.
  3. Compute sets:
       added_ids  = upstream_ids - current_catalog_ids
       dropped_ids = current_synced_ids - upstream_ids - PINNED_IDS
       kept_ids    = current_synced_ids & upstream_ids
  4. Apply adds: construct FreeModel from rec + _assign_group, call _apply_synced_model
  5. Apply drops: call _remove_synced_model (only if id NOT in PINNED_IDS — redundant check)
  6. Persist to catalog_sync.json

Key used for sync: the litellm model string (id field). Deduplication is by id.

OpenRouter: primary fetch is live API (https://openrouter.ai/api/v1/models, parses "data"[].id
for ":free" suffix, gets context_length from "data"[].context_length).
Fallback on error: parse README bullet list for ":free" slugs, assign context_window=4096.

Group assignment heuristics (_assign_group, catalog_sync.py:117-127):
  Ordered rules: (provider, pattern_in_id_lower, group)
    groq + "8b"/"7b"/"3b"/"1b" -> fast-small
    cerebras + "8b"/"3b"/"1b" -> fast-small
    groq default -> big
    cerebras default -> big
    gemini + "flash"/"gemma" -> gemini-flash
    mistral default -> mistral
    openrouter default -> openrouter-free
  First matching rule wins.

Rust port: the litellm.get_llm_provider() validation call has no clean Rust equivalent
without reimplementing litellm's provider-prefix parsing. In Rust, implement a prefix-split
on "/" and maintain a hardcoded map of valid (prefix, expected_provider) pairs.
The litellm.model_cost cost-table check can be replaced by a hardcoded set of known-valid IDs
(or skipped for OpenRouter where it was already skipped).


==============================================================================
SECTION 2: ROUTER (router.py)
==============================================================================

--- Build function (router.py:115-226) ---

build_router() -> _HealingProxy (wrapping litellm.Router)

Steps:
  1. load_dotenv from .liberty/.env (override=False: env already set wins)
  2. Iterate FREE_MODELS (includes synced additions):
       a. keys = _get_provider_keys(model.provider)  -> list[str]
       b. if no keys: SKIP silently (router.py:132)
       c. for each key: append one deployment dict to model_list
  3. Append overflow_deployments() AFTER keyed deployments (router.py:155-162)
  4. If model_list is empty: raise RuntimeError
  5. Build cross-group fallback list (present_groups filter + overflow suffix)
  6. Construct litellm.Router(model_list, fallbacks, ..., routing_strategy="simple-shuffle")
  7. Register QuotaLogger into litellm.callbacks (deduplicated)
  8. Wrap Router in _HealingProxy and return

--- Deployment dict shape (router.py:144-148) ---

One deployment per (model, key) pair:
{
  "model_name":    model.group,        # the routing group name
  "litellm_params": {
    "model":     model.id,             # litellm routing string
    "rpm":       model.rpm,
    "tpm":       model.tpm,            # only if tpm is not None
    "api_key":   key,                  # raw key string
    "api_base":  base_url,             # only for base_url providers (venice, uncensored_chat)
  }
}

Note: model_name is the GROUP (e.g. "big"), not the model id. The router routes by group.
If multiple keys exist for a provider, there are multiple deployments for the same model,
each with a different api_key. litellm.Router rotates across them.

--- Multi-key env-var convention (router.py:73-83) ---

_collect_keys(base_var: str) -> list[str]:
  1. Read os.environ[base_var] (the primary key, e.g. GROQ_API_KEY)
  2. Scan ALL env vars for pattern r"^{base_var}_(\d+)$" (e.g. GROQ_API_KEY_1, GROQ_API_KEY_2)
     Regex: re.compile(rf"^{re.escape(base_var)}_(\d+)$")
  3. Return all non-empty values in order: [primary, _1, _2, ...]

_get_provider_keys(provider: str) -> list[str]:
  Looks up _PROVIDER_ENV_PREFIXES[provider] -> list of base var names.
  For each base var, calls _collect_keys. Deduplicates (seen set).
  Returns ordered unique list.

Env var map (_PROVIDER_ENV_PREFIXES, router.py:54-62):
  "cerebras"       -> ["CEREBRAS_API_KEY"]
  "groq"           -> ["GROQ_API_KEY"]
  "gemini"         -> ["GEMINI_API_KEY", "GOOGLE_API_KEY"]
  "mistral"        -> ["MISTRAL_API_KEY"]
  "openrouter"     -> ["OPENROUTER_API_KEY"]
  "venice"         -> ["VENICE_API_KEY"]
  "uncensored_chat"-> ["UNCENSORED_CHAT_API_KEY"]

For gemini, both GEMINI_API_KEY and GOOGLE_API_KEY are tried; _collect_keys is called
for each, deduplicated. So GEMINI_API_KEY + GEMINI_API_KEY_1 + GOOGLE_API_KEY + GOOGLE_API_KEY_1
all contribute.

--- OpenAI-compatible base_url providers (router.py:67-70) ---

_PROVIDER_BASE_URLS:
  "venice"          -> "https://api.venice.ai/api/v1"
  "uncensored_chat" -> "https://api.uncensored.chat/v1"

These providers use "openai/" model prefix in catalog (so litellm routes via openai-compat)
and get api_base injected into litellm_params (router.py:140-143).

--- _FALLBACK_CHAIN ordering (router.py:106-112) ---

EXACT ordering (verbatim):
  ("big",          ["fast-small","gemini-flash","mistral","openrouter-free"])
  ("fast-small",   ["gemini-flash","mistral","openrouter-free"])
  ("gemini-flash", ["mistral","openrouter-free"])
  ("mistral",      ["openrouter-free"])
  ("uncensored",   [])   <- NO cross-group fallbacks

"uncensored" has an EMPTY fallback list. This is intentional and must be preserved in Rust:
silently falling back to a safety-tuned model would defeat the purpose of the uncensored lane
(router.py:103-106 comment).

Overflow suffix (router.py:183-185): ALL non-uncensored groups also append
["overflow-pollinations"] (and "overflow-g4f" if installed) to their fallback list.
The uncensored group explicitly excludes the overflow suffix (router.py:188-190).

At build time, fallback entries with zero reachable present groups are dropped (router.py:198-199).

--- litellm.Router params (router.py:201-210) ---

routing_strategy:  "simple-shuffle"    (usage-based-v2 deferred pending Redis)
num_retries:       2
allowed_fails:     1
cooldown_time:     60                  (seconds; the Router's own internal cooldown)
enable_pre_call_checks: True           (allows async_filter_deployments to run)

Rust port: litellm.Router is the hardest dependency to replace. In Rust, the port must
implement its own dispatcher:
  - maintain a Vec<Deployment> per group
  - implement simple-shuffle (random selection within group)
  - implement cross-group fallback chain on failure
  - integrate quota filter before dispatch (replaces async_filter_deployments)
HTTP client capability needed: async HTTP client (reqwest equivalent: async POST with
custom headers + bearer auth + JSON body/response). No crate names invented here.

--- available_groups (router.py:229-238) ---

available_groups(router) -> list[str]:
  Iterates router.model_list, collects distinct model_name values in order.

router_for_default_model() -> (router, str):
  Builds router, picks "fast-small" as default if present, else first group.


==============================================================================
SECTION 3: QUOTA (quota.py)
==============================================================================

--- KeyId (quota.py:47-67) ---

Struct fields:
  provider: str   (provider slug)
  fp:       str   (8-char lowercase hex, first 8 chars of sha256(raw_key.encode("utf-8")))

Construction: KeyId.from_raw_key(provider, raw_key) -> KeyId
  fp = hashlib.sha256(raw_key.encode()).hexdigest()[:8]   (quota.py:65-66)
  Raw key is NEVER stored in KeyId or anywhere in the module.
  str(key_id) = "{provider}:{fp}"  (quota.py:59)

Rust port: sha2::Sha256 (standard, RFC-compatible); slice to 8 hex chars.
The "never store plaintext" invariant must be preserved: the raw key must be hashed at
the call site and only the fp kept.

--- Counter shape (QuotaTracker, quota.py:99-338) ---

Internal state (all protected by a single threading.Lock):
  _day_reqs:   dict[key_str, dict[model_id, int]]   request count this UTC day
  _day_tokens: dict[key_str, dict[model_id, int]]   total token count this UTC day
  _min_reqs:   dict[key_str, dict[model_id, int]]   request count this UTC minute
  _min_tokens: dict[key_str, dict[model_id, int]]   token count this UTC minute
  _cooldowns:  dict[key_str, float]                 UTC epoch until which key is on cooldown
  _current_day:    str  "YYYY-MM-DD" (UTC)
  _current_minute: str  "HH-MM" (UTC)

key_str is str(key_id) = "{provider}:{fp}"
model_id is the litellm routing string (e.g. "groq/llama-3.1-8b-instant")

--- UTC-day keying (quota.py:85-92) ---

_utc_day_str() -> "YYYY-MM-DD" using time.strftime("%Y-%m-%d", time.gmtime())
_utc_minute_str() -> "HH-MM" using time.strftime("%H-%M", time.gmtime())

Day rollover (_roll_if_needed, quota.py:168-185):
  Clears _day_reqs, _day_tokens, _min_reqs, _min_tokens, _cooldowns on new UTC day.
  Clears only _min_reqs, _min_tokens on new UTC minute.
  Called inside the lock before every read/write operation.

--- Persistence file (quota.py:34-41, 127-164) ---

Path: runs/quota_state.json (relative to project root .liberty/)
Format:
  {
    "day":        "YYYY-MM-DD",
    "day_reqs":   {key_str: {model_id: int}},
    "day_tokens": {key_str: {model_id: int}}
  }

Written atomically: tmp file + os.replace (quota.py:153-158).
Loaded at QuotaTracker.__init__ (quota.py:124).
On load: if persisted day != current UTC day, state is discarded (fresh start). (quota.py:141-143)
Per-minute counters (_min_reqs, _min_tokens) are NOT persisted (intentional: a restart
is assumed to be longer than one minute). (quota.py:148-149 comment)
Cooldowns are NOT persisted (cleared on new day, and a restart resets them to no cooldown).

--- 429 cooldown reset priority order (quota.py:258-290) ---

EXACT header names and priority order (verbatim from note_429):
  1. "x-ratelimit-reset-requests"   (Groq-style: integer seconds OR "HH:MM:SS" duration string)
  2. "Retry-After" (and "retry-after" lowercase fallback)
  3. "x-ratelimit-reset-tokens"     (secondary fallback)
  4. Hard fallback: 60 seconds      (_FALLBACK_COOLDOWN = 60.0, quota.py:269)

Each header is tried with its canonical name and lowercase (headers.get(name) or headers.get(name.lower())).
All parseable candidates are collected; the MINIMUM (soonest reset) wins (quota.py:288: until = min(candidates)).
Rust port: look for all four headers in response headers, parse each, take the minimum.

_parse_reset_header(raw, now) -> Optional[float] (quota.py:343-386):
  Supports:
    - Pure integer or float seconds: "2", "60", "1.5"
    - HH:MM:SS duration: "0:00:02", "00:00:30" (split on ":", 3 parts)
    - RFC 7231 HTTP-date: "Mon, 16 Jun 2026 12:00:00 GMT" (via email.utils.parsedate_to_datetime)
  Returns None if unparseable.
  Returns now + parsed_seconds for duration formats; absolute epoch for HTTP-date.

--- async_filter_deployments (quota.py:518-540) ---

Called by litellm.Router pre-routing for each CustomLogger.
Signature: async_filter_deployments(model, healthy_deployments, messages, ...) -> list[dict]

For each deployment:
  1. Extract KeyId via _key_id_from_deployment (quota.py:433-457)
  2. Extract model_id via _model_id_from_deployment
  3. Call tracker.has_headroom(key_id, model_id) -> bool
  4. Include deployment if has_headroom is True OR if key_id extraction failed (None)

Deployments with no headroom are REMOVED from the list returned to the router.
If the returned list is empty, litellm raises RouterRateLimitError (all deployments exhausted).

has_headroom checks (quota.py:217-249):
  - Active cooldown (_cooldowns[key_str] > now): False
  - rpd cap: day_reqs[key_str][model_id] >= caps.rpd: False
  - tpd cap: day_tokens[key_str][model_id] >= caps.tpd: False
  - RPM/TPM NOT checked here (litellm's enable_pre_call_checks handles those via deployment rpm/tpm fields)
  - Unknown model (not in catalog): returns True (let litellm decide)

--- soonest_reset (quota.py:312-323) ---

soonest_reset() -> Optional[float]:
  Scans _cooldowns.values() for any ts > now.
  Returns min(active) or None if no active cooldowns.
  Used by self_heal.py to compute sleep duration.

--- Usage recording (quota.py:189-215) ---

record_usage(key_id, model_id, prompt_tokens, completion_tokens, headers=None):
  total = prompt_tokens + completion_tokens
  Increments day_reqs[key_str][model_id] by 1
  Increments day_tokens[key_str][model_id] by total
  Increments min_reqs[key_str][model_id] by 1
  Increments min_tokens[key_str][model_id] by total
  Calls _save_state() (persists day counters to disk)
  All under lock.

--- Module-level singleton pattern (quota.py:576-595) ---

get_tracker() -> QuotaTracker  (lazy, global singleton)
get_logger()  -> QuotaLogger   (lazy, global singleton; creates tracker if needed)

Rust port: the QuotaTracker maps directly to a Mutex-protected struct. The litellm
CustomLogger protocol (log_success_event, async_filter_deployments) must be reimplemented
as pre/post-call hooks in the Rust dispatcher. No external HTTP client needed for quota
(it is entirely in-process). Persistence uses serde_json.


==============================================================================
SECTION 4: SELF_HEAL (self_heal.py)
==============================================================================

--- self_healing_acompletion (self_heal.py:79-161) ---

Async function. Wraps router.acompletion(**kwargs).

Parameters:
  router:          any object with acompletion(**kwargs) method
  max_total_wait:  float = 120.0   (hard cap on total wait time in seconds)
  max_retries:     int = 5         (max number of exhaustion retries)
  **kwargs:        forwarded verbatim to router.acompletion

Control flow:
  deadline = monotonic_now + max_total_wait
  loop:
    try: return await router.acompletion(**kwargs)
    except (RouterRateLimitError, RouterRateLimitErrorBasic) as exc:
      attempt += 1
      if attempt > max_retries: raise
      if monotonic_now >= deadline: raise
      reset_epoch = _soonest_reset_epoch(exc)    # see below
      wait = reset_epoch - wall_now
      remaining = deadline - monotonic_now
      if wait > remaining: raise           # would exceed deadline
      if wait < 0: wait = 0
      await asyncio.sleep(wait)
      # retry

Exceptions that propagate IMMEDIATELY (no retry):
  - All exceptions EXCEPT RouterRateLimitError / RouterRateLimitErrorBasic
  - Includes: bad request (400), auth failure (401), network error, etc.

--- _soonest_reset_epoch (self_heal.py:42-76) ---

Computes UTC epoch to retry at. Priority:
  1. QuotaTracker.soonest_reset()   (per-key cooldown ledger; parses ratelimit headers)
  2. exc.cooldown_time as seconds   (only present on RouterRateLimitError, not RouterRateLimitErrorBasic)
  3. Hard fallback: now + 60.0 (_DEFAULT_BACKOFF_SECONDS)

Returns MINIMUM of all available candidates (soonest possible reset).

--- _HealingProxy (self_heal.py:164-201) ---

Thin proxy class wrapping a litellm.Router.

Constructor: _HealingProxy(router, max_total_wait=120.0, max_retries=5)
  Stores router, max_total_wait, max_retries via object.__setattr__ to bypass own __setattr__.

Attribute delegation:
  __getattr__(name) -> getattr(self._router, name)
    All attribute accesses other than acompletion transparently delegate to the real Router.
    This means: model_list, available_groups, quota callbacks, etc. are all accessible unchanged.
  __setattr__(name, value) -> setattr(self._router, name, value)
    Mutations also delegate through to the real Router.

acompletion(**kwargs) -> overrides Router.acompletion with self_healing_acompletion.

Rust port: Implement as a wrapper struct holding the real dispatcher. Override only
the completion method. The __getattr__ delegation corresponds to Deref<Target=Router>
in Rust (or explicit method delegation). The key invariant is that model_list and
the quota callback registration remain accessible on the proxy.


==============================================================================
SECTION 5: PROVIDERS_EXTRA (providers_extra.py)
==============================================================================

--- Pollinations overflow group (providers_extra.py:59-78) ---

Group name: "overflow-pollinations" (providers_extra.py:42)
Base URL: "https://text.pollinations.ai/openai"  (verified 2026-06-16, OpenAI-compat)
Models exposed: ["openai/openai-fast", "openai/openai"]
  openai-fast = GPT-OSS 20B (lowest latency)
  openai = best-quality alias on Pollinations endpoint

API key: os.environ.get("POLLINATIONS_API_KEY", "").strip() or "none"
  Keyless: if env var absent, api_key = "none" (literal string, not empty)
  This allows litellm to route without auth.

Rate limits (conservative, unpublished by Pollinations):
  rpm=10, tpm=20000

pollinations_deployments() -> list[dict]:
  Returns 2 deployment dicts (one per model), ALWAYS non-empty.
  Each deployment: {"model_name": "overflow-pollinations", "litellm_params": {...}}

Key invariant: Pollinations is ALWAYS the last-resort fallback that is always available.
The Rust port must guarantee at least one overflow deployment is always reachable.

Context windows for overflow models are registered in catalog.py:
  "openai/openai-fast" -> 32768 (conservative, unpublished)
  "openai/openai"      -> 32768 (conservative, unpublished)
(catalog.py:522-525, _OVERFLOW_MODEL_WINDOWS)

--- g4f optional tier (providers_extra.py:85-207) ---

Group name: "overflow-g4f" (providers_extra.py:85)
Enabled only when `pip install g4f` has been run.

g4f_available() -> bool:
  Tries `import g4f`. Caches result after first call (_g4f_checked flag).
  Returns False if ImportError.

g4f_deployments() -> list[dict]:
  Returns [] when g4f not available.
  When available: registers _G4FCustomLLM shim with litellm once (_G4F_SHIM_REGISTERED flag).
  Returns one deployment: {"model_name":"overflow-g4f","litellm_params":{"model":"g4f/gpt-4o-mini","custom_llm_provider":"custom","rpm":20}}

_G4FCustomLLM (providers_extra.py:165-196):
  Extends litellm.CustomLLM.
  completion(): creates g4f.client.Client(), strips "g4f/" prefix from model, calls
    client.chat.completions.create(model=g4f_model, messages=messages).
  acompletion(): runs completion() in a thread via asyncio.get_event_loop().run_in_executor(None, ...).

Rust port: g4f is Python-only (unofficial mirrors, no public API). In Rust this tier either
uses the g4f REST API if available, or is omitted entirely. The Rust port likely drops g4f
and relies on Pollinations as the sole keyless overflow.

--- overflow_deployments (providers_extra.py:217-229) ---

overflow_deployments() -> list[dict]:
  Returns pollinations_deployments() + g4f_deployments() concatenated.
  Pollinations always first (lowest index = higher priority in litellm Router).
  g4f appended after (lower priority, only when installed).


==============================================================================
DEPENDENCIES AND RUST PORT NOTES
==============================================================================

Python external dependencies with Rust equivalents needed:

litellm.Router:
  The entire dispatch engine. Provides: deployment list management, routing strategy,
  cross-group fallback, pre-call checks (filter hook), num_retries, cooldown_time.
  Rust port must reimplement all of this. No crate name invented; capability needed:
  stateful async dispatcher with per-group deployment lists, fallback chain, retry logic,
  and a pre-dispatch filter hook point.

litellm.CustomLogger (async_filter_deployments hook):
  The pre-routing filter that removes exhausted deployments.
  Rust port: implement as a trait or closure called synchronously before dispatch.

litellm.get_llm_provider(model_id):
  Used in catalog_sync to validate that the model string resolves to the expected provider.
  Rust port: implement as a split on "/" + lookup in a hardcoded provider-prefix map.

litellm.model_cost dict:
  Used in catalog_sync to verify model IDs exist in litellm's cost table.
  Rust port: maintain a hardcoded set of known-valid litellm IDs, or skip this check
  and rely on live-probe to catch bad IDs.

litellm types RouterRateLimitError / RouterRateLimitErrorBasic:
  Signal that the entire pool is exhausted (all deployments filtered or on cooldown).
  Rust port: define equivalent error variants; the dispatcher raises this when the
  filtered deployment list is empty after applying the quota filter.

httpx / urllib.request (catalog_sync fetch):
  Simple HTTP GET for the README and OpenRouter API.
  Rust port: async HTTP GET with JSON parsing.

python-dotenv (router.py load_dotenv):
  Reads .env file into os.environ.
  Rust port: dotenv crate or manual .env parser; or require env vars to be set externally.

hashlib.sha256:
  Used by KeyId.from_raw_key for key fingerprinting.
  Rust port: any SHA-256 implementation (standard in most crypto crates).

email.utils.parsedate_to_datetime:
  Used in _parse_reset_header for RFC 7231 HTTP-date parsing.
  Rust port: HTTP-date parser (standard in HTTP libraries).


==============================================================================
CRITICAL INVARIANTS FOR RUST PORT
==============================================================================

I-1: FREE_MODELS list is the single source of truth for all deployments.
     Sync appends to it in-place; router iterates it at build time.
     In Rust: a Vec<FreeModel> behind an Arc<RwLock<>> shared between sync task and router builder.

I-2: KeyId never stores the raw key.
     Only sha256(raw_key)[:8] as hex is kept. This must be enforced at the type level.

I-3: Quota day counters survive process restart (persisted to quota_state.json).
     Per-minute counters and cooldowns do NOT survive restart (intentional).
     Rust port must implement the same split: persist day counters only.

I-4: DENYLISTED_MODEL_IDS blocks re-entry at all three enforcement points:
     registration, sync ingest, and persisted-sync load.

I-5: PINNED_IDS models are never dropped by sync, regardless of upstream state.

I-6: The uncensored group has NO cross-group fallbacks, not even to overflow.
     Rust dispatch must hard-code this exception in the fallback table.

I-7: Deployment order within model_list matters: keyed providers first, overflow last.
     litellm "simple-shuffle" picks randomly within a group; when a group exhausts, it
     falls to the next group in _FALLBACK_CHAIN order. Overflow groups are always last.

I-8: RPM/TPM per-minute caps are enforced by the Router's enable_pre_call_checks
     (via deployment rpm/tpm fields), NOT by the QuotaTracker.
     QuotaTracker only enforces rpd/tpd (daily caps) and cooldowns.
     Rust port must implement RPM/TPM window tracking separately from daily tracking.

I-9: soonest_reset takes the MINIMUM across all available reset hints (not maximum).
     The wait must be the soonest possible, not the safest-pessimistic.

I-10: _HealingProxy intercepts only acompletion. All other Router state (model_list,
      group enumeration, quota registration) is accessed through transparent delegation.
      In Rust: Deref to the inner router for everything except completion.


==============================================================================
CONTRACT TEST STUBS
==============================================================================

def test_key_id_never_stores_raw():
    from liberty.quota import KeyId
    raw = "sk-test-12345678abcdef"
    kid = KeyId.from_raw_key("groq", raw)
    assert raw not in str(kid)
    assert len(kid.fp) == 8
    assert all(c in "0123456789abcdef" for c in kid.fp)

def test_denylist_blocks_all_gates():
    from liberty.catalog import DENYLISTED_MODEL_IDS, FREE_MODELS
    for mid in DENYLISTED_MODEL_IDS:
        assert mid not in {m.id for m in FREE_MODELS}, f"{mid} must not be in FREE_MODELS"

def test_fallback_chain_uncensored_empty():
    from liberty.router import _FALLBACK_CHAIN
    for src, targets in _FALLBACK_CHAIN:
        if src == "uncensored":
            assert targets == [], "uncensored must have empty fallback list"

def test_note_429_uses_soonest_header():
    import time
    from liberty.quota import QuotaTracker, KeyId
    tracker = QuotaTracker()
    kid = KeyId.from_raw_key("groq", "sk-abc")
    now = time.time()
    tracker.note_429(kid, headers={
        "x-ratelimit-reset-requests": "30",
        "Retry-After": "90",
    })
    # soonest = 30s, not 90s
    reset = tracker.soonest_reset()
    assert reset is not None
    assert reset < now + 31, f"expected ~30s cooldown, got {reset - now:.1f}s"

def test_pollinations_always_present():
    from liberty.providers_extra import pollinations_deployments
    deps = pollinations_deployments()
    assert len(deps) >= 1
    for d in deps:
        assert d["model_name"] == "overflow-pollinations"
        assert d["litellm_params"]["api_base"] == "https://text.pollinations.ai/openai"
