#!/usr/bin/env python3
"""TEIKOKU worker-dispatch telemetry helpers (W-U3 usage capture, W-U4 watchdog-fire
observation, W5 stream-artifact persistence).

Extracted out of worker_adapter.py (A2-fix) to keep that module under the MR-1 1000-line
maintainability boundary (.claude/rules/maintainability-review.md). These are relocated
helpers, not new public API: worker_adapter.WorkerAdapter.dispatch() is their sole
production caller, called as free functions with the instance state they need (db handle,
slot, subprocess timeout) passed explicitly rather than dragging the class in.
"""
from __future__ import annotations

import logging
from pathlib import Path
from typing import TYPE_CHECKING

import free_api_worker
import provider_usage
from conductor_db import ControlDB
from redaction import redact_text

if TYPE_CHECKING:
    from worker_adapter import WorkerSlot

LOGGER = logging.getLogger("worker_telemetry")


def lane_for_slot(slot: "WorkerSlot") -> str:
    """W-U3: the provider_usage lane key for a worker slot. A slot's pool (workers.toml grouping)
    is the natural lane; fall back to its harness, then its model. Keeps the per-lane ledger keyed
    the same way the interactive /chat/turn seam keys it (a pool/lane name, not a raw model id)."""
    return (slot.pool or slot.harness or slot.model or "worker").strip() or "worker"


def record_worker_usage(db_path: str | Path | None, slot: "WorkerSlot", stdout: str) -> None:
    """W-U3: accumulate this worker turn's token/cost into the per-lane provider_usage ledger.
    FAIL-SOFT: a missing db, an envelope with no usage block, or a degraded write is swallowed
    -- telemetry must never fail a dispatch that produced a real result."""
    if db_path is None:
        return
    try:
        usage = free_api_worker.parse_cc_json_usage(stdout)
        if not usage:
            return
        provider_usage.record_turn(
            ControlDB(db_path), lane_for_slot(slot),
            tokens_in=usage["tokens_in"], tokens_out=usage["tokens_out"],
            cache_read=usage["cache_read"], cost_cents=usage["cost_cents"],
        )
    except Exception:  # noqa: BLE001 - cost capture is telemetry; never fail the turn
        LOGGER.warning("worker usage capture failed for slot %s", slot.slot_id, exc_info=True)


def record_watchdog_fire(
    db_path: str | Path | None, task_id: str, slot: "WorkerSlot", subprocess_timeout: int,
    stdout_artifact: Path, stderr_artifact: Path, stdout: str, stderr: str,
) -> None:
    """W-U4 Plane-2: emit a structured WATCHDOG_FIRE observation when the autonomous
    wall-clock watchdog kills a worker, so the fire is visible (not a silent reclaim) and the
    partial output is pointed at. FAIL-SOFT: a telemetry write never fails the dispatch."""
    if db_path is None:
        return
    try:
        ControlDB(db_path).append_event(
            "WATCHDOG_FIRE",
            task_id,
            {
                "worker_model": slot.model,
                "lane": lane_for_slot(slot),
                "budget_seconds": subprocess_timeout,
                "killed": True,
                "partial_stdout_bytes": len(stdout or ""),
                "partial_stderr_bytes": len(stderr or ""),
                "artifact_refs": [str(stdout_artifact), str(stderr_artifact)],
                "observation": (
                    f"worker {slot.model} exceeded the {subprocess_timeout}s wall-clock "
                    "budget; killed; partial output captured in the stdout/stderr artifacts"
                ),
            },
        )
    except Exception:  # noqa: BLE001 - the watchdog observation is telemetry; never re-raise
        LOGGER.warning("watchdog-fire event write failed for task %s", task_id, exc_info=True)


def write_stream_artifact(path: Path, text: str) -> None:
    """W5: persist a worker output stream (redacted) as a diagnostic artifact. An
    artifact-write failure must not fail the dispatch that produced a real result,
    but it screams to the log instead of vanishing."""
    try:
        path.write_text(redact_text(text or ""), encoding="utf-8")
    except OSError:
        LOGGER.warning("failed to write worker stream artifact %s", path, exc_info=True)
