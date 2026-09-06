#!/usr/bin/env python3
"""TEIKOKU W-U5: per-provider token-in/out + cost capture (extends the W8 ledger).

The W8 cost ledger (budgets.RunCostLedger) accumulates spend keyed by ROOT RUN id, so it
answers "how much did THIS run cost" but not "how much has each LANE spent / how many tokens
has each provider moved". This module adds that per-PROVIDER view WITHOUT a new subsystem: it
accumulates into the SAME budget_counters table the W8 ledger uses, under provider-keyed rows
(provider_usage:<lane>:<metric>). One turn -> record_turn(lane, in/out/cache/cents); the
orchestration snapshot + GET /usage read it back via provider_rows.

The counters are pure ACCUMULATORS (no ceiling): each metric row carries a sentinel-large
limit_value so used_value just grows. budget_counters is integer-only (CHECK used_value >= 0),
so tokens + cents are whole ints. FAIL-SOFT: every write/read is wrapped -- a degraded DB never
raises into a turn (the conductor anti-fabrication posture: a missing source reads as zero, and
a write failure is swallowed so a cost-capture hiccup never fails a chat turn)."""
from __future__ import annotations

import sqlite3

from conductor_db import ControlDB
from time_utils import iso_utc

# The budget_counters key prefix for the per-provider accumulators (namespaced off the W8
# run_cost:* rows so the two never collide). Shape: provider_usage:<lane>:<metric>.
PROVIDER_USAGE_PREFIX = "provider_usage"

# The four metrics captured per lane. Named, no magic strings; these become the row fields.
METRIC_TOKENS_IN = "tokens_in"
METRIC_TOKENS_OUT = "tokens_out"
METRIC_CACHE_READ = "cache_read"
METRIC_COST_CENTS = "cost_cents"
_METRICS = (METRIC_TOKENS_IN, METRIC_TOKENS_OUT, METRIC_CACHE_READ, METRIC_COST_CENTS)

# A pure accumulator needs a counter row but no real ceiling; use a sentinel no lane reaches so
# the CHECK(used_value <= limit_value) style clamp never bites the accumulation.
_ACCUMULATOR_LIMIT = 1_000_000_000_000


def _counter_key(lane: str, metric: str) -> str:
    return f"{PROVIDER_USAGE_PREFIX}:{lane}:{metric}"


def _add(conn, key: str, amount: int) -> None:
    """UPSERT-add `amount` onto a provider accumulator counter row (installs the row with the
    sentinel limit on first touch). Whole-int amounts only; a non-positive amount is a no-op."""
    if amount <= 0:
        return
    conn.execute(
        """
        INSERT INTO budget_counters (budget_key, limit_value, used_value, reset_at, updated_at)
        VALUES (?, ?, ?, NULL, ?)
        ON CONFLICT(budget_key) DO UPDATE SET
          used_value = MIN(budget_counters.limit_value, budget_counters.used_value + excluded.used_value),
          updated_at = excluded.updated_at
        """,
        (key, _ACCUMULATOR_LIMIT, int(amount), iso_utc()),
    )


def record_turn(
    db: ControlDB,
    lane: str,
    *,
    tokens_in: int = 0,
    tokens_out: int = 0,
    cache_read: int = 0,
    cost_cents: int = 0,
) -> None:
    """Accumulate ONE turn's token/cost figures against a provider lane. Called at the chat_turn
    seam once the real per-turn usage + cost are known. FAIL-SOFT: a bad lane or a degraded DB is
    swallowed (a cost-capture failure must never fail the user's turn). Zero amounts are no-ops so
    a spawn-only / free-of-charge turn records only what is real (never a fabricated row)."""
    lane = (lane or "").strip()
    if not lane:
        return
    try:
        with db.transaction() as conn:
            _add(conn, _counter_key(lane, METRIC_TOKENS_IN), int(tokens_in or 0))
            _add(conn, _counter_key(lane, METRIC_TOKENS_OUT), int(tokens_out or 0))
            _add(conn, _counter_key(lane, METRIC_CACHE_READ), int(cache_read or 0))
            _add(conn, _counter_key(lane, METRIC_COST_CENTS), int(cost_cents or 0))
    except (sqlite3.Error, ValueError, TypeError):
        # a capture hiccup is swallowed -- the turn already succeeded; this is telemetry only.
        return


def provider_rows(db: ControlDB) -> list[dict]:
    """Read the per-provider accumulators back, grouped by lane. Returns one row per lane that has
    recorded anything: {lane, tokens_in, tokens_out, cache_read, spent_cents}. NEVER fabricated
    (a lane with no rows is simply absent). FAIL-SOFT: a degraded DB yields an empty list."""
    lanes: dict[str, dict] = {}
    try:
        conn = db.connect()
        try:
            rows = conn.execute(
                "SELECT budget_key, used_value FROM budget_counters WHERE budget_key LIKE ?",
                (f"{PROVIDER_USAGE_PREFIX}:%",),
            ).fetchall()
        finally:
            conn.close()
    except sqlite3.Error:
        return []
    for r in rows:
        key = r["budget_key"]
        parts = key.split(":", 2)
        if len(parts) != 3:
            continue
        _, lane, metric = parts
        if metric not in _METRICS:
            continue
        lanes.setdefault(lane, {"lane": lane, "tokens_in": 0, "tokens_out": 0,
                                "cache_read": 0, "spent_cents": 0})
        val = int(r["used_value"] or 0)
        if metric == METRIC_COST_CENTS:
            lanes[lane]["spent_cents"] = val
        elif metric == METRIC_TOKENS_IN:
            lanes[lane]["tokens_in"] = val
        elif metric == METRIC_TOKENS_OUT:
            lanes[lane]["tokens_out"] = val
        elif metric == METRIC_CACHE_READ:
            lanes[lane]["cache_read"] = val
    return sorted(lanes.values(), key=lambda x: x["lane"])
