from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from conductor_db import ControlDB


def append_event(
    db_path: str | Path,
    event_type: str,
    task_id: str | None = None,
    payload: dict[str, Any] | None = None,
    project_id: str | None = None,
    session_id: str | None = None,
) -> str:
    # TEIKOKU P2-A: pass-through of the per-project memory-log fields (2.1).
    return ControlDB(db_path).append_event(
        event_type, task_id, payload, project_id=project_id, session_id=session_id
    )


def iter_events(db_path: str | Path, after_seq: int = 0) -> list[dict[str, Any]]:
    db = ControlDB(db_path)
    conn = db.connect()
    try:
        rows = conn.execute(
            """
            SELECT event_seq, event_id, task_id, event_type, payload_json, created_at,
                   project_id, session_id
            FROM events
            WHERE event_seq > ?
            ORDER BY event_seq
            """,
            (after_seq,),
        ).fetchall()
        result = []
        for row in rows:
            item = dict(row)
            item["payload"] = json.loads(item.pop("payload_json"))
            result.append(item)
        return result
    finally:
        conn.close()


def replay_task_statuses(db_path: str | Path) -> dict[str, str]:
    statuses: dict[str, str] = {}
    for event in iter_events(db_path):
        if event["event_type"] == "TASK_IMPORTED" and event["task_id"]:
            statuses[event["task_id"]] = event["payload"].get("status", "PLANNED")
        if event["event_type"] == "TASK_STATUS_CHANGED" and event["task_id"]:
            statuses[event["task_id"]] = event["payload"]["new_status"]
    return statuses


def replay_check(db_path: str | Path) -> dict[str, Any]:
    replayed = replay_task_statuses(db_path)
    db = ControlDB(db_path)
    conn = db.connect()
    try:
        rows = conn.execute("SELECT task_id, status FROM tasks ORDER BY task_id").fetchall()
        mismatches = []
        for row in rows:
            if replayed.get(row["task_id"]) != row["status"]:
                mismatches.append(
                    {
                        "task_id": row["task_id"],
                        "db_status": row["status"],
                        "replayed_status": replayed.get(row["task_id"]),
                    }
                )
        return {"ok": not mismatches, "mismatches": mismatches, "tasks_checked": len(rows)}
    finally:
        conn.close()
