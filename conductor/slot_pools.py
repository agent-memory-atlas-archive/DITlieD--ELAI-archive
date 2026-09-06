"""Worker slot model + tier-partitioned pool construction (SLOT-FIX leg 2, paid-leg-entry.md).

Extracted from worker_adapter.py (which sat at the 1000-line boundary) so the slot domain --
the WorkerSlot record, the fallback model pools, config-section coercion, and the pool
partition -- lives in one focused module. The partition rule is DECISION a: a slot's own
declared model_tier is the semantic truth (leases.acquire_ready matches per-slot model_tier),
never which TOML section ([[workers]] vs [[escalation_workers]]) declared it, so a PAID slot
under [[workers]] (the role-split configs, e.g. conductor.elai-opus.toml
opus-exec/opus-wire/sonnet-rest) is dispatchable by select_slot instead of KeyError-ing at
EXECUTE after a valid lease. Each section keeps its historical coercion defaults (workers ->
FREE, escalation_workers -> PAID, fallback-when-empty); ONLY the partition rule changed.
"""
from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from conductor_config import WorkerSlotConfig

FREE_MODEL_POOL = ["swe-1.6", "glm-5.2", "kimi-k2.7"]
PAID_MODEL_POOL = ["opus"]


@dataclass(frozen=True)
class WorkerSlot:
    slot_id: str
    model: str
    model_tier: str
    harness: str = "devin-free"
    account_id: str | None = None
    codex_home: str | None = None
    wsl_distro: str | None = None
    proxy_base_url: str | None = None
    pool: str | None = None


def _coerce_slots(items: Any, fallback_models: list[str], tier: str) -> list[WorkerSlot]:
    slots: list[WorkerSlot] = []
    if items:
        for index, item in enumerate(items):
            if isinstance(item, WorkerSlotConfig):
                slots.append(
                    WorkerSlot(
                        slot_id=item.slot_id,
                        model=item.model,
                        model_tier=item.model_tier.upper(),
                        harness=item.harness,
                        account_id=item.account_id,
                        codex_home=item.codex_home,
                        wsl_distro=item.wsl_distro,
                        proxy_base_url=getattr(item, "proxy_base_url", None),
                        pool=getattr(item, "pool", None),
                    )
                )
            elif isinstance(item, dict):
                slots.append(
                    WorkerSlot(
                        slot_id=str(item.get("slot_id", item.get("name", f"{tier.lower()}{index + 1}"))),
                        model=str(item["model"]),
                        model_tier=str(item.get("model_tier", item.get("tier", tier))).upper(),
                        harness=str(item.get("harness", "devin-free")),
                        account_id=item.get("account_id"),
                        codex_home=item.get("codex_home"),
                        wsl_distro=item.get("wsl_distro"),
                        proxy_base_url=item.get("proxy_base_url"),
                        pool=item.get("pool"),
                    )
                )
            else:
                raise TypeError(f"unsupported worker slot config: {item!r}")
    if not slots:
        slots = [
            WorkerSlot(slot_id=f"{tier.lower()}{index + 1}", model=model, model_tier=tier)
            for index, model in enumerate(fallback_models)
        ]
    return slots


def build_slot_pools(data: dict[str, Any]) -> tuple[list[WorkerSlot], list[WorkerSlot]]:
    """Return (free_slots, paid_slots) partitioned by each slot's OWN model_tier."""
    merged = [
        *_coerce_slots(data.get("workers"), FREE_MODEL_POOL, "FREE"),
        *_coerce_slots(data.get("escalation_workers"), PAID_MODEL_POOL, "PAID"),
    ]
    free = [slot for slot in merged if slot.model_tier == "FREE"]
    paid = [slot for slot in merged if slot.model_tier != "FREE"]
    return free, paid
