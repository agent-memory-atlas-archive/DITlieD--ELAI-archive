TEIKOKU Phase 4 — .liberty free-model backbone map

task_id: p4-explore-liberty-map
scope: READ-ONLY exploration of F:\.ELAI_workflow\.liberty\
written: 2026-06-20
grounding: direct read of the Liberty source (catalog.py, catalog_sync.py, router.py,
providers_extra.py, bridge_client.py, chat/session_factory.py, chat/limits.py,
chat/commands.py), the docs (LIBERTY-CAPABILITIES-AND-ROTATION.md, FREE_PROVIDER_KEYS.md),
the .env (NAMES only, no values read), and the two liberty handoffs. Every figure below is
tied to a file unless marked otherwise. No secret value was read or written.


================================================================================
1. WHAT .liberty IS
================================================================================

A free-tier multi-agent coding harness. ~16K LOC Python, a port of the ale_claw / OpenClaw
computer-use agent with the computer-use machinery stripped and litellm swapped in as the
model layer (LIBERTY-CAPABILITIES-AND-ROTATION.md:13-16). It runs litellm in LIBRARY MODE —
an in-process litellm.Router, NOT a separate proxy server (router.py:4-5). Design thesis:
THROUGHPUT OVER DEPTH — instead of burning paid credits on one provider, spread a task
across the UNION of many providers' FREE tiers over hours, each serving a slice
(LIBERTY-CAPABILITIES-AND-ROTATION.md:13-19).

It is a deliberately simple coding agent (no repo map, no LSP, no diff/edit-format engine).
Its real engineering is the rotation/continuity layer (section 3) plus a pair of UNTRUSTED
sandboxed reasoning brokers (codex-vm WSL + the gpt-bridge browser pool).

TOP-LEVEL STRUCTURE (verified)
  .liberty/
    liberty/                  the package
      catalog.py              SINGLE SOURCE OF TRUTH for the free-model roster (typed)
      catalog_sync.py         daily catalog expansion from cheahjs/free-llm-api-resources
      router.py               litellm.Router builder + the cross-group FALLBACK CHAIN
      providers_extra.py      keyless overflow tier (Pollinations always-on + optional g4f)
      quota.py                per-(KeyId,model) UTC-day req/token counters + 429 cooldown
      self_heal.py            _HealingProxy: queue+sleep on whole-pool exhaustion
      agent_loop.py           the LibertyAgent turn loop + per-served-model window budgeting
      bridge_client.py        gpt-bridge MCP client (browser ChatGPT/Claude/Gemini/Grok pool)
      bridge_transport.py     MCP subprocess transport for the bridge
      gpt_harness_broker.py   one-turn "gptharness" backend through the bridge (UNTRUSTED)
      codex_broker.py         one-turn "codex" backend in a throwaway WSL2 sandbox (UNTRUSTED)
      chat/                    the Textual TUI: screen.py, session_factory.py, limits.py,
                               commands.py (11 slash verbs), cost.py, session_factory.py
      model/model_config.py   resolve_model() registry (catalog auto-registers windows here)
      tools/                  workspace tools + external_tools (codex/bridge gating)
      subagent/              SubagentRegistry + depth-2 parallel fan-out
    docs/                      summaries (the 5 .md files below)
    .env / .env.example        provider keys (gitignored; .env holds a 38-name template)
    runs/                      quota_state.json + catalog_sync.json (persisted state)
    liberty_sessions/          per-task JSONL transcripts (resume.py rebuilds from these)
    tests/                     ~301 offline tests

DOCS SUMMARIES (the docs\ folder)
  LIBERTY-CAPABILITIES-AND-ROTATION.md  the canonical grounded summary: rotation substrate,
        cross-model continuity, why it is context-light, the steal list (S1-S5), and the
        function-calling-vs-text-edit analysis. READ THIS FIRST for behavior.
  FREE_PROVIDER_KEYS.md  the full provider roster with signup URLs, env-var names, and the
        per-provider RPM/RPD/TPM limits (Tier A high-volume / Tier B gateways / Tier B2
        micro-credit / Tier U uncensored / Tier C one-time trials). READ THIS for quotas.
  CODEX_BROKER_SAFETY.md + CODEX_WSL_SETUP.md + codex-broker-stubs.md  the WSL codex broker.


================================================================================
2. THE FULL FREE-MODEL ROSTER
================================================================================

CRITICAL SCOPING FACT (the single most important finding for P4):
  Liberty's litellm catalog does NOT contain the Devin free models (glm 5.2, swe 1.6,
  kimi-k2.7) or Claude. A full-tree grep for glm|swe-1|kimi|k2.7|devin|moonshot|zhipu over
  liberty/ returned ZERO hits. Those windows the owner cited (glm5.2/swe1.6 ~150k, k2.7
  ~260k, claude ~500k-effective) are NOT Liberty deployments — they are the CONTROLLER'S
  own model pool: the ELAI free-model sub-agent profiles (context-compiler / executor /
  auditor / memory-writer — the external-backends handoff records "memory-writer failed due
  to Devin daily usage quota", which is the orchestrator's model, not a litellm route) AND
  the gpt-bridge browser pool (bridge_client.claude_chat / gemini_chat / grok_chat). See
  section 4 for the tie. For P4 this means: the Devin/Claude windows live in the ELAI
  controller layer; Liberty's windows are the litellm roster below. Treat them as TWO
  separate pools (this is exactly what P4-C "model pool choice" must model).

The Liberty roster has THREE layers:
  (A) PINNED CORE (catalog.py FREE_MODELS, 14 entries) — always present, probe-confirmed.
  (B) KEYLESS OVERFLOW (providers_extra.py) — Pollinations always-on + optional g4f.
  (C) DAILY-SYNCED ADDITIONS (catalog_sync.py) — appended at runtime from cheahjs, only for
      the 5 WIRED providers (cerebras, groq, gemini, mistral, openrouter; _WIRED_PROVIDERS
      catalog_sync.py:84). Synced entries pass a litellm.get_llm_provider + model_cost gate
      so they are never unverified.

--------------------------------------------------------------------------------
2A. PINNED CORE ROSTER (catalog.py FREE_MODELS) — provider / id / group / window / limits
--------------------------------------------------------------------------------
All "keyed" (need the provider env key) unless noted. context_window + max_output + rpm/tpm/
rpd are exact from the FreeModel dataclass fields.

GROUP "fast-small" (high-RPM small models)
  groq/llama-3.1-8b-instant
    provider groq | keyed (GROQ_API_KEY) | ctx 131,072 | max_out 8,192
    rpm 30 | tpm 6,000 | RPD 14,400  <- highest-volume daily worker in the pool

GROUP "big" (70B-120B class — chat DEFAULTS here, session_factory.py:78-80)
  groq/llama-3.3-70b-versatile
    groq | keyed | ctx 131,072 | max_out 32,768 | rpm 3 | tpm 12,000 | RPD 1,000
  cerebras/gpt-oss-120b
    cerebras | keyed (CEREBRAS_API_KEY) | ctx 131,072 | max_out 8,192
    rpm 30 | tpm 60,000 | RPD 14,400 | TPD 1,000,000  <- highest RPD in the big group
  groq/openai/gpt-oss-120b
    groq | keyed | ctx 131,072 | max_out 65,536 | rpm 3 | tpm 8,000 | RPD 1,000
  groq/meta-llama/llama-4-scout-17b-16e-instruct
    groq | keyed | ctx 131,072 | max_out 8,192 | rpm 3 | tpm 30,000 | RPD 1,000
  groq/qwen/qwen3-32b
    groq | keyed | ctx 32,768 | max_out 32,768 | rpm 3 | tpm 6,000 | RPD 1,000

GROUP "gemini-flash" (Gemini flagship free tier — huge ctx, tiny RPD)
  gemini/gemini-2.5-flash
    gemini | keyed (GEMINI_API_KEY|GOOGLE_API_KEY) | ctx 1,048,576 | max_out 65,536
    rpm 10 | tpm 250,000 | RPD 20  <- 1M ctx but only 20 req/day; long-context single-shot
  gemini/gemini-2.5-flash-lite
    gemini | keyed | ctx 1,048,576 | max_out 65,536 | rpm 15 | tpm 250,000 | RPD 500

GROUP "mistral"
  mistral/mistral-small-latest
    mistral | keyed (MISTRAL_API_KEY) | ctx 32,768 | max_out 32,768
    rpm 60 | tpm 500,000 | RPD none (1B tokens/MONTH hard cap instead)

GROUP "openrouter-free" (last-resort keyed gateway)
  openrouter/meta-llama/llama-3.3-70b-instruct:free
    openrouter | keyed (OPENROUTER_API_KEY) | ctx 131,072 | max_out 4,096
    rpm 20 | RPD 50 (no top-up; ~1,000/day after a one-time $10 lifetime top-up)

GROUP "uncensored" (TIER U — ISOLATED, NO cross-group fallback, tools suppressed)
  openrouter/cognitivecomputations/dolphin-mistral-24b-venice-edition:free
    openrouter | keyed (reuse OPENROUTER_API_KEY) | ctx 33,000 | max_out 4,096
    rpm 20 | RPD 50  <- best free uncensored volume
  openrouter/nousresearch/hermes-3-llama-3.1-405b:free
    openrouter | keyed (reuse OPENROUTER_API_KEY) | ctx 131,072 | max_out 4,096
    rpm 20 | RPD 50
  openai/venice-uncensored (Venice, OpenAI-compat base_url https://api.venice.ai/api/v1)
    venice | keyed (VENICE_API_KEY) | ctx 32,768 | max_out 4,096 | rpm 10 | RPD 10
  openai/uncensored-llama-3.3-70b (Uncensored.chat, base_url https://api.uncensored.chat/v1)
    uncensored_chat | keyed (UNCENSORED_CHAT_API_KEY) | ctx 131,072 | max_out 4,096
    rpm 20 | RPD none (permanent free credits)

--------------------------------------------------------------------------------
2B. KEYLESS OVERFLOW (providers_extra.py) — the always-present last resort
--------------------------------------------------------------------------------
GROUP "overflow-pollinations" (KEYLESS — api_key="none" unless POLLINATIONS_API_KEY set)
  openai/openai-fast  (GPT-OSS-20B class, lowest latency) | ctx 32,768 (conservative, capped
       in catalog.py:518 — Pollinations publishes no window) | rpm 10 | tpm 20,000
  openai/openai       (best-quality alias)                 | ctx 32,768 | rpm 10 | tpm 20,000
  base https://text.pollinations.ai/openai. KEYLESS anon tier is severely rate-limited
  (FREE_PROVIDER_KEYS.md:90: ~1/IP/hour unauth); treat as best-effort overflow only.
GROUP "overflow-g4f" (OPTIONAL, only if `pip install g4f`; absent by default) — g4f/gpt-4o-mini
  via a litellm CustomLLM shim. Wraps unofficial mirrors; ToS-gray; last-resort only.

--------------------------------------------------------------------------------
2C. KEYED PROVIDERS THAT EXIST IN .env BUT ARE NOT IN THE CORE CATALOG
--------------------------------------------------------------------------------
The .env template carries 38 key NAMES (read NAMES only; the file is a blank template — all
values empty). Beyond the 7 wired providers above, these are present in .env / FREE_PROVIDER_
KEYS.md and join ONLY when a key is set AND (for the non-core ones) the daily sync or a future
base_url wiring picks them up:
  Tier A extra:   CODESTRAL_API_KEY, NVIDIA_NIM_API_KEY, CLOUDFLARE_API_KEY(+ACCOUNT_ID),
                  GITHUB_API_KEY, COHERE_API_KEY
  Tier B2:        HF_TOKEN, VERCEL_AI_GATEWAY_API_KEY
  Tier C trials:  SAMBANOVA, FIREWORKS_AI, HYPERBOLIC, NEBIUS, NOVITA, AI21, UPSTAGE,
                  NLP_CLOUD, MODAL, BASETEN, INFERENCE_NET, OVH, LAMBDA, KLUSTER, CHUTES,
                  SCALEWAY  (one-time credit, NOT sustainable rotation)
  Control flags:  LIBERTY_CODEX_ENABLE, LIBERTY_CODEX_NETWORK, LIBERTY_CODEX_IMAGE,
                  LIBERTY_CODEX_ENV_KEYS, LIBERTY_GPT_HARNESS_ENABLE
  NOTE: catalog_sync only wires {cerebras,groq,gemini,mistral,openrouter}. The Tier C and
  Cloudflare/NVIDIA/Cohere keys are documented for the user to add but are NOT auto-routed by
  the current sync (catalog_sync.py:84 _WIRED_PROVIDERS) — they are aspirational roster, not
  live deployments today.

--------------------------------------------------------------------------------
2D. DAILY-SYNCED EXPANSION (catalog_sync.py)
--------------------------------------------------------------------------------
At runtime, sync_catalog() pulls cheahjs/free-llm-api-resources README daily and APPENDS
verified models to FREE_MODELS (PINNED_IDS = the 10 probe-confirmed, never dropped). It maps
e.g. groq llama-4-maverick, allam-2-7b, gemini 3/3.5 flash, gemma 3 12b/4b/1b, mistral large/
3.1 — so the live roster on a given day is a SUPERSET of 2A. Windows for synced models come
from litellm.model_cost (_ctx, catalog_sync.py:362). Three ids are DENYLISTED (404 on probe):
cerebras/llama3.1-8b, cerebras/llama-3.3-70b, gemini/gemma-3-27b-it.


================================================================================
3. ROUTING + ROTATION
================================================================================

ROUTER CONSTRUCTION (router.py::build_router)
  - One litellm "deployment" per (model, key) pair. model_name = the GROUP, so all models in
    a group share a routing bucket (router.py:144-148). Multi-key rotation: GROQ_API_KEY,
    GROQ_API_KEY_1, _2, ... all fan into the same group (_collect_keys router.py:73-83).
  - routing_strategy "simple-shuffle" (usage-based-v2 needs Redis, deferred). num_retries=2,
    allowed_fails=1, cooldown_time=60, enable_pre_call_checks=True (router.py:201-210).
  - Absent provider keys -> the deployment is silently skipped (graceful degrade). Zero
    deployments -> RuntimeError (router.py:163-173).

CROSS-GROUP FALLBACK CHAIN (router.py::_FALLBACK_CHAIN:106-112, the rotation backbone)
  big           -> fast-small -> gemini-flash -> mistral -> openrouter-free
  fast-small    -> gemini-flash -> mistral -> openrouter-free
  gemini-flash  -> mistral -> openrouter-free
  mistral       -> openrouter-free
  (every non-uncensored group also gets overflow-pollinations [+ overflow-g4f] appended as
   the final suffix, router.py:183-191)
  uncensored    -> []  (INTENTIONALLY EMPTY — no cross-group fallback; the lane never silently
   degrades to a safety-tuned model. Fixed + tested this way per the uncensored-routing
   handoff 56e16bf. Internal rotation within the uncensored group still happens.)

QUOTA + COOLDOWN (quota.py, surfaced via chat/limits.py)
  - Per-(KeyId, model) daily request + token counters, persisted UTC-day-keyed to
    runs/quota_state.json (survives restart). KeyId hashes the raw key (no plaintext stored).
  - On 429: cooldown with header-priority reset parse
    (x-ratelimit-reset-requests > Retry-After > x-ratelimit-reset-tokens > 60s).
  - async_filter_deployments removes exhausted deployments BEFORE routing.

SELF-HEAL (self_heal.py::_HealingProxy)
  - Wraps the Router. On WHOLE-POOL exhaustion (RouterRateLimitError) it queues + sleeps
    until QuotaTracker.soonest_reset() and retries. All other exceptions propagate.
  - Transparent attribute delegation so every call site sees a normal Router.

HOW "prefer kimi-k2.7, max 3 parallel, drop to 2 on rate-limit" MAPS ON
  - "prefer <model>": Liberty selects by GROUP or concrete model id, not a preference score.
    The chat starts on "big" (session_factory.py:78) and you pin a model via /model <id>.
    For a P4 "prefer X" policy you set the session's group/model to X; the fallback chain is
    fixed per-group, so "prefer" = "start here, auto-degrade down the chain on 429".
    (NB: kimi-k2.7 is NOT a Liberty model — see section 2 — so a "prefer kimi" policy is a
    CONTROLLER-pool policy, not a Liberty-router one.)
  - "max 3 parallel": Liberty's own parallelism is the subagent fan-out (orchestrator.py,
    depth-2 cap). There is NO built-in "N concurrent" throttle on the router itself — rate is
    governed by per-key RPM/RPD in quota.py, not a parallelism cap. P4-B must impose the
    "3 parallel" cap at the WORKER-POOL level (the controller dispatching N agents), then let
    each agent's litellm calls rotate independently.
  - "drop to 2 on rate-limit": there is no built-in degrade-concurrency hook. The router's
    response to a 429 is per-deployment cooldown + fallback, NOT a global concurrency cut.
    P4-D would have to watch the QuotaTracker cooldown state (exposed via limits.quota_rows /
    quota_headroom_hint) and reduce the controller's parallel fan-out itself. This is a
    GAP P4 must build above Liberty, not reuse from it.


================================================================================
4. TIES (bridge / litellm-proxy / conductors)
================================================================================

LIBERTY <-> THE DEVIN/BRIDGE LAYER (the key question)
  Liberty does NOT route Devin models through any bridge. The "bridge" in Liberty is the
  gpt-bridge MCP (bridge_client.py, _DEFAULT_BRIDGE_DIR = D:\.apps_workflow\.claude\mcp\
  gpt-bridge:50) — a BROWSER-pool client exposing gpt_chat / grok_chat / claude_chat /
  gemini_chat / perplexity_chat + async gpt_image / gpt_deep_research. It is wired as an
  optional PER-TURN chat BACKEND ("gptharness", BACKEND_GPT_HARNESS), NOT a litellm catalog
  model, and replies are marked UNTRUSTED. It is gated OFF by default
  (LIBERTY_GPT_HARNESS_ENABLE, chat/limits.py:34,61-69) and fails closed if the bridge is
  absent. So: the gpt-bridge gives Liberty a side-channel to claude.ai/gemini/grok browser
  pools, but those are not part of the rotating litellm roster and not Devin.
  -> The Devin free models (glm/swe/kimi) belong to the ELAI controller (the sub-agent model
     profiles run_subagent dispatches), entirely outside Liberty.

LIBERTY <-> A litellm PROXY ("0.3 graft")
  Liberty runs litellm in LIBRARY mode (in-process Router), NOT against a standalone litellm
  PROXY server (router.py:4-5, "no separate proxy server"). There is no proxy-graft code in
  the Liberty tree. If TEIKOKU has a separate free-api litellm PROXY (the "0.3 graft"), it is
  a DIFFERENT component; Liberty would talk to it only if you pointed a deployment's api_base
  at it (the same mechanism used for Venice/Uncensored.chat/Pollinations base_url providers).
  UNVERIFIED: I found no such graft inside .liberty; this tie, if it exists, lives in the
  TEIKOKU/.elai_cc side, not here.

LIBERTY <-> CODEX BROKER (codex_broker.py)
  A SECOND untrusted backend ("codex", BACKEND_CODEX): offloads one turn to a strong reasoner
  running agentically in a throwaway WSL2 (+ optional Docker) sandbox, TEXT-ONLY, no
  copy-back, fail-closed, gated by LIBERTY_CODEX_ENABLE. Independent of the litellm roster.

LIBERTY <-> "conductors"
  UNVERIFIED: no symbol named "conductor" exists in the Liberty source (grep miss). Liberty's
  own orchestration is the subagent fan-out (orchestrator.py + SubagentRegistry, depth-2).
  Any "conductor" concept is a TEIKOKU/.elai_cc term, not a Liberty one.


================================================================================
5. RELEVANCE TO P4 (the actionable part)
================================================================================

5a. P4-B WORKER POOL — which Liberty models are realistic autonomous-coding workers
  REALISTIC CODING WORKERS (capable enough + enough daily volume to sustain a worker):
    - cerebras/gpt-oss-120b   BEST worker: 120B-class, 131K ctx, 14,400 RPD, 1M TPD, 30 rpm.
    - groq/llama-3.3-70b-versatile  strong 70B, 131K ctx — but only 1,000 RPD (burst worker).
    - groq/openai/gpt-oss-120b      120B, 131K ctx, 65K max_out — 1,000 RPD.
    - groq/meta-llama/llama-4-scout-17b-16e  MoE, 131K ctx, 30K TPM — 1,000 RPD.
    - groq/llama-3.1-8b-instant  FAST high-volume (14,400 RPD) but 8B = weak coder; good for
      cheap mechanical sub-tasks / the fast-small lane, not the lead worker.
    - mistral/mistral-small-latest  24B, 32K ctx, generous 1B tok/month — solid mid worker.
    - openrouter/...70b:free  capable but 50 RPD wall (no top-up) = bench/eval, not a worker.
  TOO WEAK / NOT A WORKER:
    - groq/qwen3-32b (32K ctx, 1,000 RPD) is usable but mid; gemini-flash is a SINGLE-SHOT
      long-context tool (20 RPD), NOT a turn-by-turn worker; Pollinations/g4f overflow are
      best-effort/unreliable, fine as a fallback floor, not a primary worker; the uncensored
      lane has NO tool support (tools suppressed, catalog.py:579-591) so it cannot drive an
      agentic tool loop — text-only, exclude from the autonomous coding pool.
  VERDICT for P4-B: a Liberty-backed worker pool's spine is cerebras/gpt-oss-120b (volume) +
  the groq 70B/120B tier (quality, rationed by RPD) + mistral-small (steady) with the
  fast-small 8B as the cheap-task lane and Pollinations as the never-empty floor.

5b. P4-C MODEL-POOL CHOICE — what the selectable pools should be
  Liberty ALREADY ships the exact "per-session model pool" UI P4-C describes. The picker
  (chat/limits.py::backend_entries) offers, per session:
    (i) router GROUPS: big / fast-small / gemini-flash / mistral / openrouter-free /
        uncensored / overflow-pollinations  (each labeled with its max ctx, e.g.
        "gemini-flash up to 1M ctx", "big up to 131K ctx" — limits.py::_group_context_label)
    (ii) every concrete catalog model id
    (iii) Codex (WSL)        [BACKEND_CODEX, greyed unless LIBERTY_CODEX_ENABLE]
    (iv) GPT (browser)       [BACKEND_GPT_HARNESS, greyed unless LIBERTY_GPT_HARNESS_ENABLE]
  Switched live by /model <group|id>, /codex, /gptharness (chat/commands.py, 11 verbs).
  RECOMMENDED P4-C SELECTABLE POOLS (map onto this, add the controller pool as a peer):
    - "claude"      -> the ELAI/cloud controller pool (NOT Liberty; the Sonnet/Opus + bridge
                       claude_chat path) — the high-trust expensive lane.
    - "devin-free"  -> the controller's Devin free profiles (glm 5.2 / swe 1.6 / kimi-k2.7) —
                       also NOT Liberty; the free-but-rate-capped autonomous lane.
    - "free-api"    -> Liberty's WHOLE rotating roster (the "big" group as default, full
                       fallback chain) — the throughput lane. THIS is the .liberty pool.
    - optionally per-PROVIDER sub-pools that map to Liberty groups (e.g. "cerebras-fast" =
       big-with-cerebras-first, "gemini-1M" = gemini-flash for long-context single shots,
       "uncensored" = the isolated uncensored lane).
  i.e. P4-C's pools are a SUPERSET: {claude, devin-free} from the controller + {free-api and
  its sub-groups} from Liberty. Liberty's group picker is the template for the free-api side.

5c. P4-D AUTO-MIGRATION — PER-POOL / PER-MODEL context-threshold table
  Liberty proves the migration trigger MUST be per-model: agent_loop.py:382-385,458-467 reads
  response.model after each call and budgets the NEXT turn against THAT model's real window
  ("Feature A v2"); on turn 1 it budgets against the GROUP-MINIMUM window across reachable
  deployments (catalog.min_context_window_for_group) so it never overruns the weakest model
  the router might pick. The unsafe global default (context.py DEFAULT_CONTEXT_TOKENS=200,000)
  is explicitly avoided; the safe small default is 4,096 (catalog.py:375).

  CONCRETE PER-MODEL MIGRATE-AT TABLE for P4-D (window from catalog.py; suggested margin =
  leave ~20-25% headroom for the reply + tool output, mirroring Liberty's compact-earlier
  posture; "migrate-at" = trigger migration/compaction when prompt tokens cross this):

    POOL / model                              window    suggested migrate-at (~75-80%)
    --- CONTROLLER POOL (not Liberty; owner-stated windows, UNVERIFIED here) ---
    claude (cloud/bridge)                     ~500,000  ~400,000
    devin kimi-k2.7                           ~260,000  ~200,000
    devin glm-5.2                             ~150,000  ~120,000
    devin swe-1.6                             ~150,000  ~120,000
    --- LIBERTY free-api POOL (windows VERIFIED from catalog.py) ---
    gemini/gemini-2.5-flash                 1,048,576   ~820,000  (RPD 20 — single-shot only)
    gemini/gemini-2.5-flash-lite            1,048,576   ~820,000
    cerebras/gpt-oss-120b                     131,072   ~104,000
    groq/llama-3.3-70b-versatile              131,072   ~104,000
    groq/openai/gpt-oss-120b                  131,072   ~104,000
    groq/meta-llama/llama-4-scout-17b         131,072   ~104,000
    groq/llama-3.1-8b-instant                 131,072   ~104,000
    openrouter/...llama-3.3-70b:free          131,072   ~104,000
    nousresearch/hermes-3-405b:free           131,072   ~104,000
    openai/uncensored-llama-3.3-70b           131,072   ~104,000
    groq/qwen/qwen3-32b                         32,768    ~26,000
    mistral/mistral-small-latest                32,768    ~26,000
    openai/venice-uncensored                    32,768    ~26,000
    dolphin-mistral-24b-venice:free             33,000    ~26,000
    overflow openai/openai(-fast) Pollinations  32,768    ~26,000  (conservative cap)
    any UNKNOWN / synced-without-window         4,096     ~3,200   (safe small default)

  KEY P4-D RULE (lifted from Liberty, must hold): when a session's pool can be SERVED by more
  than one model (a router group with a fallback chain), the migrate-at must use the MINIMUM
  window across all reachable models in that pool, not the headline one — otherwise a prompt
  sized for cerebras-120K overruns when the router falls back to a 32K mistral. Liberty
  computes this in catalog.min_context_window_for_group; P4-D should reuse the same min-over-
  reachable logic per selectable pool. A SINGLE global threshold is wrong (proven by the
  1M-vs-32K spread above).


================================================================================
STATUS / RETURN
================================================================================
STATUS: DONE
FILES TOUCHED: F:\.ELAI_workflow\workflow\reasoning\teikoku-p4-explore-liberty-map.md
  (this map; the only write)
VERIFIED (value-free):
  - read catalog.py (full roster + windows + limits + min-window logic), catalog_sync.py
    (daily expansion, PINNED_IDS, DENYLIST, _WIRED_PROVIDERS), router.py (fallback chain +
    quota/self-heal wiring), providers_extra.py (keyless Pollinations/g4f), bridge_client.py
    (gpt-bridge browser pool, _DEFAULT_BRIDGE_DIR), chat/session_factory.py (chat starts on
    "big"), chat/limits.py (the model-picker = groups+ids+codex+gptharness), chat/commands.py
    (11 slash verbs), LIBERTY-CAPABILITIES-AND-ROTATION.md, FREE_PROVIDER_KEYS.md, and the two
    liberty handoffs.
  - .env read for KEY NAMES ONLY (38 names listed in 2C); NO secret value read or written.
  - grep over liberty/ for glm|swe-1|kimi|k2.7|devin|moonshot|zhipu|conductor: ZERO hits ->
    Devin models + "conductors" are NOT in Liberty (controller-layer concepts).
  UNVERIFIED: the controller-pool windows (claude ~500K, kimi ~260K, glm/swe ~150K) are
    owner-stated, not confirmable from the Liberty tree; the "0.3 litellm-proxy graft" was not
    found in .liberty (lives in TEIKOKU/.elai_cc if it exists); no live inference was run.
