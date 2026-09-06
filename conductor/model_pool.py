#!/usr/bin/env python3
"""TEIKOKU P4-C (master-plan 4.2 / D-10): the per-session model-pool picker.

A chat session picks a (pool, model). The picker resolves that selection to the four worker
slot fields the P4-B worker plane dispatches off -- (harness, model, proxy_base_url,
account_id) -- which a caller stamps onto the session's WorkerSlotConfig (conductor_config.py
:25-44). worker_adapter._build_env + build_free_api_command then route the turn UNCHANGED
(no worker-plane edits): the picker is purely a slot-config PRODUCER.

TWO POOL SOURCES (TEIKOKU-P4-REVISION-2026-06-20 :46-50, :118-123):
  CONTROLLER  {claude (plan-OAuth), devin-free: kimi-k2-7 / glm-5-2 / swe-1-6 (via the bridge)}
  FREE-API    {the litellm free-proxy roster: cerebras / groq / mistral / pollinations}
The CONTROLLER entries are NOT in the liberty catalog (it ships only the free-api half); they
are hand-authored here. The free-api entries mirror the liberty group names so the same
selection ports to the litellm proxy. The liberty catalog (catalog.FreeModel / backend_entries
/ min_context_window_for_group) is the DATA-SHAPE precedent this module reuses; it lives on the
host (.liberty), not in this distro, so the shape is ported minimally here rather than imported.

THE DEVIN ROUTING CONTRACT (CRITICAL, controller-verified from the bridge devin_wire.py
:254-268 + the worker plane): a per-session devin model id (kimi-k2-7 / glm-5-2 / swe-1-6) is
sent as base-cc's --model -- i.e. it rides slot.model -- through ONE shared PASSTHROUGH bridge
proxy. The bridge's build_request PREFERS a valid incoming body["model"] over its
DEVIN_PROXY_MODEL fallback ("model switches take effect live without a proxy restart"), and
worker_adapter builds the command via build_free_api_command(slot.model) for EVERY base-cc
backend (worker_adapter.py:443, free_api_worker.py:122). So the picker sets slot.model = the
chosen devin id; DEVIN_PROXY_MODEL stays ONLY the proxy-side fallback default (kimi-k2-7), NOT
the per-session selector. proxy_base_url stays None -> worker_providers.bridge_base_url()
resolves the host bridge at dispatch.

GENERALITY: the pool/model values are pure data; nothing here references ELAI. A non-ELAI chat
picks the same pools.

Producers/consumers (D-030 wiring): resolve_slot / list_pools here are the PRODUCERS; the
consumer wired in the SAME changeset is chat_orchestrator.apply (stamps the resolved tuple onto
the spawn's model_tier/slot path) + the slice_api routes that surface the picker. resolve_slot
is unit-driven by the picker tests (T4).

Stdlib only (no new dependency).
"""
from __future__ import annotations

from dataclasses import dataclass
from typing import Any

import worker_providers

# Pool ids the picker offers (the SELECTABLE backend classes, revision :120-121).
POOL_CLAUDE = "claude"
POOL_DEVIN_FREE = "devin-free"
POOL_FREE_API = "free-api"
POOL_CODEX = "codex"
POOL_POSTHOG = "posthog"
POOLS = (POOL_CLAUDE, POOL_DEVIN_FREE, POOL_FREE_API, POOL_CODEX)

# The proxy fallback default the bridge uses when a turn carries no explicit model (the
# per-session selector overrides this via slot.model; this is the floor, never the selector).
# Owner law 2026-08-04: free Devin = glm-5-2 + swe-1-7* + swe-check ONLY. Grok is PAID.
# Main free worker = glm-5-2. Live free_roster default is swe-1-7-medium; we prefer glm.
DEVIN_DEFAULT_MODEL = "glm-5-2"
# The litellm free-proxy default alias (a free-api turn with no explicit choice). Was "free-fast"
# (keyless Pollinations: measured 4-30s + a heavily-loaded public floor) -- repointed to the
# coding-grade gpt-oss-120b on Groq (measured sub-second at the proxy, real tool-calling), so a
# fresh free-api chat lands on a strong+fast model instead of the slow keyless floor.
FREE_API_DEFAULT_MODEL = "free-groq-big"


@dataclass(frozen=True)
class PoolModel:
    """One selectable picker row: a concrete model within a pool, with the window + migrate-at
    threshold P4-D needs (the per-model auto-migration table, revision :131-141). harness is the
    WorkerSlotConfig.harness the selection resolves to; window/migrate_at are data the migration
    layer reads off response.model (P4-D consumes these, not P4-C).

    vision_capable: True ONLY for models verified to accept image content blocks via the
    Anthropic API (claude-opus/sonnet/haiku multi-modal family). Conservative default: False.
    A model is non-vision unless explicitly marked here. The composer paste gate reads this
    flag from the picker payload (Unit E image-attach).

    tool_reliable (TEIKOKU-IR TR-4 capability floor): True ONLY for coding-grade models
    verified to hold up under a multi-tool base-cc turn (kimi/glm/swe on the bridge, the
    gpt-oss-120b / cerebras-120b / 70b / qwen3-32b / gemini free lanes, claude, codex). Left
    conservative (False) for the sub-coding-grade free lanes (mistral-small, the llama-3.1-8b
    groq lane, the keyless Pollinations floors), grounded in the J-55 keystone finding that
    open models below ~14B collapse ~12% on multi-tool code-mode. route_decider.capability_filter
    drops a non-tool_reliable candidate for a tool_heavy or hard-EXECUTE role; a vision task drops
    a non-vision_capable candidate. Seed values are DATA here (auditable), refined by TR-5's ledger."""

    pool: str
    model: str
    harness: str
    context_window: int
    migrate_at: int
    label: str = ""
    note: str = ""
    vision_capable: bool = False
    tool_reliable: bool = False

    def to_dict(self) -> dict[str, Any]:
        return {
            "pool": self.pool,
            "model": self.model,
            "harness": self.harness,
            "context_window": self.context_window,
            "migrate_at": self.migrate_at,
            "label": self.label or self.model,
            "note": self.note,
            "vision_capable": self.vision_capable,
            "tool_reliable": self.tool_reliable,
        }


# The CONTROLLER pool roster (claude + devin-free), revision :131-141 windows/thresholds.
# claude's migrate_at=200_000 is the OWNER quality-handoff override (not the real window).
_CONTROLLER_MODELS: list[PoolModel] = [
    # claude (Anthropic claude-opus-4/sonnet-4 family): verified vision-capable via the
    # Anthropic API docs (image content blocks supported on all current claude models).
    PoolModel(POOL_CLAUDE, "claude", "claude", 200_000, 200_000,
              label="claude (plan-OAuth)", note="high-trust plan lane; quality handoff at 200k",
              vision_capable=True, tool_reliable=True),
    # Free Devin bridge ONLY (owner 2026-08-04, free_roster.json live):
    #   glm-5-2, swe-1-7, swe-1-7-medium, swe-check
    # NOT free (never put here as free workers):
    #   grok-4-5-high / any Grok 4.5 (paid on Devin AND official Grok)
    #   glm-5-2-max / 1M variants, swe-1-7-lightning, swe-1-6 (billed)
    #   kimi-k2-7 (disabled; coerced to glm if selected)
    # vision status unverified on bridge routes; conservative False.
    PoolModel(POOL_DEVIN_FREE, "glm-5-2", "devin-free", 200_000, 160_000,
              label="devin glm-5-2 (main free)", note="bridge :8810 FREE", tool_reliable=True),
    PoolModel(POOL_DEVIN_FREE, "swe-1-7", "devin-free", 262_000, 210_000,
              label="devin swe-1-7 Max (free)", note="bridge :8810 FREE", tool_reliable=True),
    PoolModel(POOL_DEVIN_FREE, "swe-1-7-medium", "devin-free", 262_000, 210_000,
              label="devin swe-1-7 Medium (free)", note="bridge :8810 FREE", tool_reliable=True),
    PoolModel(POOL_DEVIN_FREE, "swe-check", "devin-free", 200_000, 160_000,
              label="devin swe-check (free)", note="bridge :8810 FREE", tool_reliable=True),
]

# The FREE-API pool roster (the litellm proxy lanes). harness "free-api" routes through
# free_api_worker; model is the FREE-PROXY ALIAS (free-*), never a devin id. EVERY alias here
# MUST exist as a model_name in teikoku/free-proxy/config.yaml -- the label/window are kept
# TRUTHFUL to what that config actually routes to (the prior roster mislabeled free-fast as
# "cerebras gpt-oss-120b" when it routes to keyless Pollinations openai-fast, and free-groq as
# "70B/120B" when it routes to the weak llama-3.1-8b; the real big models were never exposed).
# Windows: groq serves full 131k; cerebras gpt-oss-120b is 65,536 on the FREE tier (131k paid,
# verified cerebras docs 2026-06); gemini 2.5-flash is 1M; mistral-small + the keyless
# Pollinations floor are 32k. migrate_at ~= 0.8 * window. All text-only (vision conservative
# False; the proxy lanes do not wire image content blocks).
_FREE_API_MODELS: list[PoolModel] = [
    # --- owner 2026-08-21: openrouter ox-alpha via zenov, max effort, EVERY task class ---
    PoolModel(POOL_FREE_API, "ox-alpha", "free-api", 1_048_576, 838_000,
              label="openrouter ox-alpha (zenov gateway)", note="litellm proxy :4000 (openrouter)",
              tool_reliable=True),
    # --- coding-grade big models (fast + strong; the sensible defaults) --- all tool_reliable
    # (>=32B coding-grade or a 120b: hold up under multi-tool base-cc turns, TR-4).
    PoolModel(POOL_FREE_API, "free-groq-big", "free-api", 131_072, 104_000,
              label="free groq gpt-oss-120b (coding, fast)", note="litellm proxy :4000 (groq, 1000 RPD)",
              tool_reliable=True),
    PoolModel(POOL_FREE_API, "free-big", "free-api", 65_536, 52_000,
              label="free cerebras gpt-oss-120b (coding)", note="litellm proxy :4000 (cerebras, 65k free tier)",
              tool_reliable=True),
    PoolModel(POOL_FREE_API, "free-groq-70b", "free-api", 131_072, 104_000,
              label="free groq llama-3.3-70b", note="litellm proxy :4000 (groq)", tool_reliable=True),
    PoolModel(POOL_FREE_API, "free-groq-qwen", "free-api", 131_072, 104_000,
              label="free groq qwen3-32b", note="litellm proxy :4000 (groq)", tool_reliable=True),
    # --- huge-context (single-shot Q&A; low RPD) --- gemini flash does real tool-calling.
    PoolModel(POOL_FREE_API, "free-gemini", "free-api", 1_048_576, 800_000,
              label="free gemini 2.5-flash (1M ctx)", note="litellm proxy :4000 (gemini, low RPD)",
              tool_reliable=True),
    PoolModel(POOL_FREE_API, "free-gemini-lite", "free-api", 1_048_576, 800_000,
              label="free gemini 2.5-flash-lite", note="litellm proxy :4000 (gemini, permissive RPD)",
              tool_reliable=True),
    # --- mid / small --- SUB-coding-grade: tool_reliable stays False (conservative floor, TR-4).
    PoolModel(POOL_FREE_API, "free-mistral", "free-api", 32_768, 26_000,
              label="free mistral-small (32k)", note="litellm proxy :4000 (mistral, 60 RPM)"),
    PoolModel(POOL_FREE_API, "free-groq", "free-api", 131_072, 104_000,
              label="free groq llama-3.1-8b (fast/small)", note="litellm proxy :4000 (groq, 14400 RPD burst)"),
    # --- keyless overflow floor (always reachable, no provider key; slow + heavily loaded) ---
    PoolModel(POOL_FREE_API, "free-fast", "free-api", 32_768, 26_000,
              label="free pollinations openai-fast (keyless)", note="litellm proxy :4000 (keyless floor)"),
    PoolModel(POOL_FREE_API, "free-pollinations", "free-api", 32_768, 26_000,
              label="free pollinations (keyless floor)", note="litellm proxy :4000 (keyless floor)"),
    # --- last-resort (heavily rate-limited; kept selectable, OUT of the auto-fallback chain) ---
    PoolModel(POOL_FREE_API, "free-openrouter", "free-api", 131_072, 104_000,
              label="free openrouter llama-3.3-70b (last-resort)", note="litellm proxy :4000 (openrouter, 50 RPD)",
              tool_reliable=True),
]

# The CODEX paid lane (revision D3 / :140). gpt-5.5 has NO context_window in any codex
# config; this is an OWNER-OVERRIDE-PENDING placeholder with a CONSERVATIVE documented
# window so the lane never overruns, flagged for owner/bmapi confirmation. It is
# surfaced in the picker as its own POOL_CODEX pool (the codex-exec chat harness) and also
# feeds the migration table a per-model row when the codex lane is selected. window/migrate_at are
# deliberately conservative until confirmed.
GPT55_WINDOW_PENDING = True  # OWNER-OVERRIDE-PENDING: confirm from bmapi /v1/models
_CODEX_MODELS: list[PoolModel] = [
    PoolModel("codex", "gpt-5-5", "codex-exec", 128_000, 100_000,
              label="codex gpt-5.5 (paid)",
              note="OWNER-OVERRIDE-PENDING window: confirm from bmapi /v1/models or owner",
              tool_reliable=True),
]

# The POSTHOG-credit lane (grok CLI -> zenov :9020 -> posthog-bridge). PAID (PostHog account
# pool), never the free Devin bridge. These rows exist so find_pool_model resolves the slugs:
# WITHOUT them, WorkerAdapter's pinned-model override (lane_serves_model) returned False for
# every posthog id, so a --model posthog-gpt56-sol dispatch on the luna-seeded slot SILENTLY
# ran luna while the dispatch record claimed sol (found 2026-08-06 wiring the escalation lane).
# Windows come from $GROK_HOME/config.toml [model.*] context_window (the config the worker CLI
# itself reads): luna/terra/sol 500k, fable-5/opus-5 1M. migrate_at ~= 0.8 * window.
# fable-5/opus-5 are the SEAM-DESIGN / escalation-decision authors (opus_dispatch gates them
# to doc-only write sets); they are in the roster so their pins resolve, not for EXECUTE work.
_POSTHOG_MODELS: list[PoolModel] = [
    PoolModel(POOL_POSTHOG, "posthog-gpt56-luna-fast", "grok-cli", 1_050_000, 840_000,
              label="posthog gpt-5.6-luna FAST (workhorse)",
              note="zenov :9020 posthog-bridge PAID; fast serving tier",
              tool_reliable=True),
    PoolModel(POOL_POSTHOG, "posthog-gpt56-sol-fast", "grok-cli", 1_050_000, 840_000,
              label="posthog gpt-5.6-sol FAST (boss)",
              note="zenov :9020 posthog-bridge PAID; fast serving tier",
              tool_reliable=True),
    PoolModel(POOL_POSTHOG, "posthog-gpt56-luna", "grok-cli", 500_000, 400_000,
              label="posthog gpt-5.6-luna (workhorse)", note="zenov :9020 posthog-bridge PAID",
              tool_reliable=True),
    PoolModel(POOL_POSTHOG, "posthog-gpt56-terra", "grok-cli", 500_000, 400_000,
              label="posthog gpt-5.6-terra", note="zenov :9020 posthog-bridge PAID",
              tool_reliable=True),
    PoolModel(POOL_POSTHOG, "posthog-gpt56-sol", "grok-cli", 500_000, 400_000,
              label="posthog gpt-5.6-sol (stuck-executor escalation)",
              note="zenov :9020 posthog-bridge PAID",
              tool_reliable=True),
    PoolModel(POOL_POSTHOG, "posthog-fable-5", "grok-cli", 1_000_000, 800_000,
              label="posthog fable-5 (seam design / plan authoring ONLY)",
              note="zenov :9020 posthog-bridge PAID; doc-only write sets",
              tool_reliable=True),
    PoolModel(POOL_POSTHOG, "posthog-opus-5", "grok-cli", 1_000_000, 800_000,
              label="posthog opus-5 (design escalation ONLY)",
              note="zenov :9020 posthog-bridge PAID; doc-only write sets",
              tool_reliable=True),
]

_ALL_MODELS: list[PoolModel] = [*_CONTROLLER_MODELS, *_FREE_API_MODELS, *_CODEX_MODELS, *_POSTHOG_MODELS]

# The default model per pool (the picker's selection when only a pool is chosen).
_POOL_DEFAULT: dict[str, str] = {
    POOL_CLAUDE: "claude",
    POOL_DEVIN_FREE: DEVIN_DEFAULT_MODEL,
    POOL_FREE_API: FREE_API_DEFAULT_MODEL,
    POOL_CODEX: "gpt-5-5",
    POOL_POSTHOG: "posthog-gpt56-luna-fast",
}


@dataclass(frozen=True)
class SlotResolution:
    """The four worker-slot fields a (pool, model) selection resolves to. A caller stamps these
    onto the session's WorkerSlotConfig before dispatch; worker_adapter does the rest UNCHANGED.
    model_tier is FREE for every pool except claude (PAID = the high-trust plan lane)."""

    harness: str
    model: str
    proxy_base_url: str | None
    account_id: str | None
    model_tier: str

    def to_dict(self) -> dict[str, Any]:
        return {
            "harness": self.harness,
            "model": self.model,
            "proxy_base_url": self.proxy_base_url,
            "account_id": self.account_id,
            "model_tier": self.model_tier,
        }


def list_pools() -> list[dict[str, Any]]:
    """The picker payload: every pool with its selectable models (the surface a chat /model
    picker renders, the liberty backend_entries analog)."""
    out: list[dict[str, Any]] = []
    for pool in POOLS:
        models = [m.to_dict() for m in _ALL_MODELS if m.pool == pool]
        out.append({"pool": pool, "default": _POOL_DEFAULT.get(pool, ""), "models": models})
    return out


def free_ladder_ascending() -> list[PoolModel]:
    """The FREE climb ladder (devin-free + free-api reachable lanes) ordered by context_window
    ASCENDING, drawn from the SAME non-stalling reachable sets the auto-fallback uses
    (_POOL_FALLBACK_MODELS for devin-free + free-api), so the known-stalling last-resort lanes
    (free-openrouter 50 RPD, free-gemini-lite) are excluded exactly as they are from the rotation
    chain. TEIKOKU-IR TR-1 walks this smallest-first so pick_model_for_window lands the SMALLEST
    free window that holds the task before climbing. Ties (equal window) break by the chain
    preference order so the strong+fast lanes sort ahead of the weak same-window ones."""
    reachable: dict[str, int] = {}
    for pool in (POOL_DEVIN_FREE, POOL_FREE_API):
        for i, mid in enumerate(_POOL_FALLBACK_MODELS.get(pool, [])):
            reachable.setdefault(mid, i)
    free = [m for m in _ALL_MODELS if m.model in reachable]
    free.sort(key=lambda m: (m.context_window, reachable.get(m.model, 999)))
    return free


def claude_model() -> PoolModel:
    """The claude (PAID high-trust) PoolModel -- the top rung of the routing ladder (TR-1 climb,
    TR-4 capability-floor climb, TR-5 veto climb)."""
    return get_pool_model(POOL_CLAUDE, "claude")


def get_pool_model(pool: str, model: str | None = None) -> PoolModel:
    """Resolve (pool, model) to its PoolModel descriptor. An empty/None model falls back to the
    pool default. Raises ValueError on an unknown pool or an unknown model within the pool (a
    typo must scream, not silently mis-route -- the worker would otherwise launch the wrong
    backend)."""
    if pool not in POOLS:
        raise ValueError(f"unknown model pool: {pool} (known: {list(POOLS)})")
    chosen = (model or "").strip() or _POOL_DEFAULT.get(pool, "")
    for m in _ALL_MODELS:
        if m.pool == pool and m.model == chosen:
            return m
    available = [m.model for m in _ALL_MODELS if m.pool == pool]
    raise ValueError(f"unknown model '{chosen}' in pool '{pool}' (available: {available})")


def resolve_slot(pool: str, model: str | None = None) -> SlotResolution:
    """Resolve a (pool, model) picker selection to the worker-slot fields. THE DEVIN CONTRACT:
    for the devin-free pool the chosen model rides slot.model (-> base-cc --model -> the bridge
    prefers it over DEVIN_PROXY_MODEL), proxy_base_url stays None (worker_providers resolves the
    host bridge at dispatch). For claude: no base_url, model "claude", PAID tier. For free-api:
    the free-proxy alias as slot.model, proxy_base_url None (free_api_env resolves the :4000
    proxy at dispatch). account_id is None for every pool here (codex's paid account profiles are
    a SEPARATE harness, not reachable from this free/plan picker)."""
    pm = get_pool_model(pool, model)
    if pm.pool == POOL_CLAUDE:
        return SlotResolution(
            harness="claude", model=pm.model, proxy_base_url=None,
            account_id=None, model_tier="PAID",
        )
    if pm.pool == POOL_DEVIN_FREE:
        # slot.model = the devin id; the bridge build_request prefers it over DEVIN_PROXY_MODEL.
        assert worker_providers.is_devin_free_harness(pm.harness)
        return SlotResolution(
            harness=pm.harness, model=pm.model, proxy_base_url=None,
            account_id=None, model_tier="FREE",
        )
    # free-api: the free-proxy alias rides slot.model; the :4000 proxy resolves at dispatch.
    return SlotResolution(
        harness=pm.harness, model=pm.model, proxy_base_url=None,
        account_id=None, model_tier="FREE",
    )


# ---------------------------------------------------------------------------
# P4-D (master-plan 4.3 / D-09): per-model window resolution + the MIN-window-across-
# reachable-fallback reducer the auto-migration trigger budgets against. These read the
# roster above; they are the DATA SOURCE the migration decision uses INSTEAD of base-cc's
# flat-200k getContextWindowForModel default (which is wrong for every non-claude model).
# ---------------------------------------------------------------------------

# A conservative floor for a model id we cannot resolve (revision :139 "unknown/synced-no-
# window" row): budget tightly rather than overrun. Named, not magic.
UNKNOWN_WINDOW = 4_096
UNKNOWN_MIGRATE_AT = 3_200

# The reachable-fallback sets per pool (revision :128-131). When a turn runs on a pool whose
# router can degrade across models, the trigger budgets against the MINIMUM reachable window
# so a large prompt does not overrun after a fallback to a smaller model. claude + codex are
# single-model lanes (no fallback); devin-free and free-api degrade across their rosters.
# ORDERED best -> worst: this doubles as (a) the min-window reducer's reachable set (order does
# not matter there -- the min across the set is the conservative budget) and (b) FIX 4's rotation
# FALLBACK CHAIN (order DOES matter there -- try strong+fast first, degrade to the keyless floor
# last). free-openrouter (50 RPD, 40s 429 stalls) + free-gemini-lite are kept selectable in the
# picker but OUT of the auto-fallback chain so a degrade never lands on a known-stalling lane.
_POOL_FALLBACK_MODELS: dict[str, list[str]] = {
    POOL_CLAUDE: ["claude"],
    # Free Devin only — never grok-4-5-* (paid). Order: main free worker first.
    POOL_DEVIN_FREE: ["glm-5-2", "swe-1-7", "swe-1-7-medium", "swe-check"],
    POOL_FREE_API: [
        "free-groq-big", "free-big", "free-groq-70b", "free-groq-qwen",
        "free-gemini", "free-mistral", "free-groq", "free-fast", "free-pollinations",
    ],
    "codex": ["gpt-5-5"],
}


def fallback_chain(pool: str | None, selected: str | None) -> list[str]:
    """The ordered free-* aliases to try AFTER ``selected`` when a free-api turn fails/times-out/
    429s (the rotation chain run_free_api_turn_direct degrades through). Returns the pool's
    reachable-fallback order (best -> keyless floor, _POOL_FALLBACK_MODELS) with ``selected``
    removed (the caller tries it first). Empty for a single-model lane (claude/codex) so those
    never rotate. NEVER raises."""
    chain = _POOL_FALLBACK_MODELS.get(pool or "", [])
    return [m for m in chain if m and m != (selected or "")]


# TEIKOKU-IR B1-fix: route only if window * SAFETY >= est (same 0.8 headroom route_decider uses).
# Named here so leases.acquire_ready enforces the SAME fit rule the decider picks against, without
# importing route_decider (which pulls the whole routing stack). Fail-closed headroom.
WINDOW_SAFETY_FRACTION = 0.8


def _harness_group(harness: str) -> str:
    """Normalize a slot/model harness to its LANE group (the proxy/env a base-cc turn routes
    through). Two harnesses in the same group share a proxy, so a pinned model can be dispatched
    on a slot in the same group by overriding base-cc's --model (the documented devin-bridge /
    free-api per-session model contract). Distinct groups need distinct env, so a cross-group
    override is unsafe."""
    h = (harness or "").strip().lower()
    if h in ("devin-free", "devin-bridge", "devin-cli"):
        return "devin-free"
    if h in ("codex", "codex-exec"):
        return "codex"
    if h in ("claude", "claude-oauth"):
        return "claude"
    return h  # free-api (and any future single-name lane)


def lane_serves_model(slot_harness: str, model_id: str) -> bool:
    """TEIKOKU-IR B1-fix: True when a worker slot on ``slot_harness`` can dispatch ``model_id`` by
    overriding base-cc's --model to it (same lane/proxy). Grounded in the shipped contract: a
    devin-free slot runs any devin id through the one bridge; a free-api slot runs any free-* alias
    through the litellm proxy (model_pool.py header + worker_adapter build_free_api_command(slot.model)).
    An unknown model id never serves (fail-closed)."""
    pm = find_pool_model(model_id)
    if pm is None:
        return False
    return _harness_group(slot_harness) == _harness_group(pm.harness)


def effective_served_window(
    slot_model: str, slot_harness: str, pinned_model: str | None
) -> tuple[str, int]:
    """TEIKOKU-IR B1-fix: the (model, context_window) a slot will ACTUALLY run for a task. When a
    pinned model is set AND this slot's lane can serve it (exact match, or same-lane override), the
    slot runs the PINNED model, so the effective window is the pinned model's. Otherwise the slot
    runs its own seeded model. An unknown effective model falls back to the conservative
    UNKNOWN_WINDOW (never assume it fits, D-07). leases.acquire_ready uses this to decide whether a
    slot can serve a pinned/large task without starving unpinned ones."""
    if pinned_model and (slot_model == pinned_model or lane_serves_model(slot_harness, pinned_model)):
        pm = find_pool_model(pinned_model)
        if pm is not None:
            return pinned_model, pm.context_window
    pm = find_pool_model(slot_model)
    return slot_model, (pm.context_window if pm is not None and pm.context_window > 0 else UNKNOWN_WINDOW)


def find_pool_model(model: str) -> PoolModel | None:
    """Resolve a bare model id (response.model) to its PoolModel across ALL rosters (controller +
    free-api + codex), independent of the picker pools. Returns None for an unknown id (the
    caller applies the UNKNOWN_* floor). The migration trigger reads response.model, which is a
    bare id, not a (pool, model) pair -- so this is the by-id lookup the trigger needs."""
    if not model:
        return None
    for m in _ALL_MODELS:
        if m.model == model:
            return m
    return None


def resolve_window(model: str, pool: str | None = None) -> tuple[int, int]:
    """The per-model (context_window, migrate_at) the migration trigger budgets against, sourced
    from the model_pool roster -- NOT base-cc's flat 200k default. When ``pool`` is given and that
    pool has a reachable-fallback set, returns the MINIMUM (context_window, migrate_at) across the
    reachable models (revision :128-131): a 131k prompt on a pool that can degrade to a 32k
    mistral budgets against the 32k floor. With no pool (or a single-model lane) it returns the
    single model's window. An unknown model falls back to the conservative UNKNOWN_* floor."""
    if pool and pool in _POOL_FALLBACK_MODELS and len(_POOL_FALLBACK_MODELS[pool]) > 1:
        windows: list[int] = []
        migrates: list[int] = []
        for mid in _POOL_FALLBACK_MODELS[pool]:
            pm = find_pool_model(mid)
            if pm is not None:
                windows.append(pm.context_window)
                migrates.append(pm.migrate_at)
        if windows:
            return min(windows), min(migrates)
    pm = find_pool_model(model)
    if pm is not None:
        return pm.context_window, pm.migrate_at
    # Single-candidate fallback: when the exact model id is unknown (e.g. base-cc emits a
    # versioned id like "claude-opus-4-8" but the roster only has "claude") AND the pool has
    # exactly one fallback candidate, use that candidate's window instead of UNKNOWN_WINDOW.
    # Multi-candidate pools (devin-free / free-api) already entered the min-across-fallback
    # branch above; this branch only fires for single-entry pools (claude / codex).
    if pool and pool in _POOL_FALLBACK_MODELS and len(_POOL_FALLBACK_MODELS[pool]) == 1:
        fallback_id = _POOL_FALLBACK_MODELS[pool][0]
        pm_fallback = find_pool_model(fallback_id)
        if pm_fallback is not None:
            return pm_fallback.context_window, pm_fallback.migrate_at
    return UNKNOWN_WINDOW, UNKNOWN_MIGRATE_AT
