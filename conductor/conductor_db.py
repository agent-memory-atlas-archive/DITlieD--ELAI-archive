from __future__ import annotations

import json
import sqlite3
import uuid
from contextlib import contextmanager
from pathlib import Path
from typing import Any, Iterator

from models import TASK_STATUSES, TaskSpec, json_dumps
from redaction import redact_json
from time_utils import iso_utc
import wake  # TEIKOKU P3-D: event-driven session wake (best-effort dep-done notify)


MIGRATIONS_DIR = Path(__file__).with_name("migrations")

# TEIKOKU P2-A: the project a conductor-internal event is attributed to when a caller
# does not specify one (all ~50 existing conductor callers). Named constant, no magic string.
DEFAULT_PROJECT_ID = "conductor"

# TEIKOKU P3-A: the widened worker_slots schema (harness CHECK now allows codex + free-api,
# and the pool column is part of the canonical table). Used by the one-shot CHECK-widen
# rebuild in _ensure_compatible_schema. The column set mirrors migrations/001 plus pool.
_WORKER_SLOTS_CANONICAL_COLUMNS = (
    "slot_id",
    "harness",
    "model",
    "model_tier",
    "account_id",
    "max_concurrency",
    "state",
    "cooldown_until",
    "last_error_category",
    "pool",
    "roles",
)
_WORKER_SLOTS_REBUILD_DDL = """
CREATE TABLE worker_slots__p3a_new (
  slot_id TEXT PRIMARY KEY,
  harness TEXT NOT NULL CHECK (harness IN ('devin-cli','devin-free','devin-bridge','claude','codex','free-api')),
  model TEXT NOT NULL,
  model_tier TEXT NOT NULL CHECK (model_tier IN ('FREE','PAID')),
  account_id TEXT,
  max_concurrency INTEGER NOT NULL DEFAULT 1,
  state TEXT NOT NULL CHECK (state IN ('IDLE','LEASED','COOLDOWN','DISABLED')),
  cooldown_until TEXT,
  last_error_category TEXT,
  pool TEXT,
  roles TEXT
)
"""
REQUIRED_TABLES = {
    "schema_meta",
    "tasks",
    "task_deps",
    "worker_slots",
    "leases",
    "task_attempts",
    "gate_runs",
    "integrations",
    "artifacts",
    "model_cooldowns",
    "budget_counters",
    "events",
    "verdicts",
    "control_state",
}
REQUIRED_COLUMNS = {
    # TEIKOKU P3-A (master-plan 3.1/3.4): `pool` tags a slot with its rate-limit pool
    # (devin | free_api.groq | ...) so per-pool concurrency can be enforced.
    "worker_slots": {"account_id", "pool"},
    # TEIKOKU-IR B1-fix: the routing-enforcement columns (pinned concrete model + required window).
    "tasks": {"pinned_model", "required_context_window"},
    "leases": {"worker_pid"},
    # TEIKOKU P2-A (master-plan 2.1): per-project append-only memory log.
    "events": {"project_id", "session_id"},
}


class ControlDB:
    def __init__(self, db_path: str | Path):
        self.db_path = Path(db_path)

    def connect(self) -> sqlite3.Connection:
        if str(self.db_path) != ":memory:":
            self.db_path.parent.mkdir(parents=True, exist_ok=True)
        conn = sqlite3.connect(str(self.db_path))
        conn.row_factory = sqlite3.Row
        conn.execute("PRAGMA foreign_keys = ON")
        conn.execute("PRAGMA busy_timeout = 5000")
        return conn

    def close(self) -> None:
        """No-op: connections are opened per-transaction (see ``transaction``).

        Provided so callers that follow the ``if hasattr(db, 'close'): db.close()``
        guard (e.g. gate_runner.run_gates) do not leak an AttributeError, and so
        the object can be used as a context-managed resource without holding a
        persistent connection.
        """
        return None

    @contextmanager
    def transaction(self) -> Iterator[sqlite3.Connection]:
        conn = self.connect()
        try:
            conn.execute("BEGIN IMMEDIATE")
            yield conn
            conn.commit()
        except Exception:
            conn.rollback()
            raise
        finally:
            conn.close()

    def init(self, reset: bool = False) -> None:
        if reset and str(self.db_path) != ":memory:":
            for suffix in ("", "-wal", "-shm"):
                path = Path(str(self.db_path) + suffix)
                if path.exists():
                    path.unlink()
        conn = self.connect()
        try:
            conn.execute("PRAGMA journal_mode = WAL")
            for migration in sorted(MIGRATIONS_DIR.glob("*.sql")):
                conn.executescript(migration.read_text(encoding="utf-8"))
            self._ensure_compatible_schema(conn)
            conn.commit()
        finally:
            conn.close()

    def _ensure_compatible_schema(self, conn: sqlite3.Connection) -> None:
        worker_columns = {
            row["name"]
            for row in conn.execute("PRAGMA table_info(worker_slots)").fetchall()
        }
        if "account_id" not in worker_columns:
            conn.execute("ALTER TABLE worker_slots ADD COLUMN account_id TEXT")
        # TEIKOKU P3-A (master-plan 3.4): the rate-limit pool a slot belongs to.
        # Idempotent column add (SQLite lacks ADD COLUMN IF NOT EXISTS), mirroring the
        # account_id add above. The harness-CHECK rebuild below recreates the table with
        # the pool column already present, so this guarded ALTER is a no-op after a rebuild.
        if "pool" not in worker_columns:
            conn.execute("ALTER TABLE worker_slots ADD COLUMN pool TEXT")
        if "roles" not in worker_columns:
            conn.execute("ALTER TABLE worker_slots ADD COLUMN roles TEXT")
        # TEIKOKU P3-A (master-plan 3.1): widen worker_slots.harness CHECK from the
        # single legacy value ('devin-cli') to ('devin-cli','codex','free-api'). SQLite
        # CHECK constraints are immutable, so this is the documented table rebuild, run
        # only when the legacy single-value CHECK is still in place (idempotent across the
        # executescript() re-run on every boot).
        self._widen_worker_harness_check(conn)
        # TEIKOKU-IR B1-fix: the two nullable routing-enforcement columns on tasks. A
        # decide()-pinned task carries the concrete model the router chose (so the worker RUNS
        # it, not the seeded slot model) + the estimated context it needs (so acquire_ready
        # refuses a too-small slot). Idempotent ADD COLUMN (SQLite lacks IF NOT EXISTS),
        # mirroring the account_id/pool/roles adds above. Existing rows get NULL -> tier-only
        # lease + seeded model (backward-compatible; the walk/self-test infra is untouched).
        task_columns = {
            row["name"]
            for row in conn.execute("PRAGMA table_info(tasks)").fetchall()
        }
        if "pinned_model" not in task_columns:
            conn.execute("ALTER TABLE tasks ADD COLUMN pinned_model TEXT")
        if "required_context_window" not in task_columns:
            conn.execute("ALTER TABLE tasks ADD COLUMN required_context_window INTEGER")
        lease_columns = {
            row["name"]
            for row in conn.execute("PRAGMA table_info(leases)").fetchall()
        }
        if "worker_pid" not in lease_columns:
            conn.execute("ALTER TABLE leases ADD COLUMN worker_pid INTEGER")
        # TEIKOKU P2-A: generalize the events table into a per-project memory log.
        # Idempotent column add (SQLite lacks ADD COLUMN IF NOT EXISTS); existing rows
        # keep NULL project_id/session_id, which the append API backfills going forward.
        event_columns = {
            row["name"]
            for row in conn.execute("PRAGMA table_info(events)").fetchall()
        }
        if "project_id" not in event_columns:
            conn.execute("ALTER TABLE events ADD COLUMN project_id TEXT")
        if "session_id" not in event_columns:
            conn.execute("ALTER TABLE events ADD COLUMN session_id TEXT")
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_events_project ON events(project_id, event_seq)"
        )
        conn.execute(
            """
            INSERT INTO schema_meta (key, value)
            VALUES ('schema_version', '2')
            ON CONFLICT(key) DO UPDATE SET value = excluded.value
            """
        )

    def _widen_worker_harness_check(self, conn: sqlite3.Connection) -> None:
        """Rebuild worker_slots with the widened harness CHECK if it still carries the
        legacy single-value ('devin-cli') constraint.

        SQLite cannot ALTER a CHECK constraint, so the only path is the 12-step table
        rebuild (https://sqlite.org/lang_altertable.html#otheralter). The rebuild is
        guarded on the live table DDL so it runs exactly once; on every subsequent boot
        the widened CHECK is already present and this is a no-op. leases.slot_id has a
        foreign key to worker_slots(slot_id), so foreign_keys is toggled off for the
        DROP/RENAME and restored afterward (per the SQLite rebuild procedure).
        """
        ddl_row = conn.execute(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'worker_slots'"
        ).fetchone()
        if ddl_row is None or ddl_row[0] is None:
            return
        ddl = ddl_row[0]
        # Already widened (the new CHECK names 'codex') -> nothing to do.
        if "codex" in ddl:
            return
        # Not the legacy harness CHECK we know how to rebuild -> leave it alone.
        if "harness" not in ddl or "devin-cli" not in ddl:
            return
        existing_columns = [
            row["name"]
            for row in conn.execute("PRAGMA table_info(worker_slots)").fetchall()
        ]
        # Always rebuild into the canonical column set (incl. pool); copy only the
        # columns that exist in the current table so older rows survive intact.
        copy_columns = [c for c in existing_columns if c in _WORKER_SLOTS_CANONICAL_COLUMNS]
        copy_csv = ", ".join(copy_columns)
        conn.execute("PRAGMA foreign_keys = OFF")
        try:
            conn.execute("DROP TABLE IF EXISTS worker_slots__p3a_new")
            conn.execute(_WORKER_SLOTS_REBUILD_DDL)
            conn.execute(
                f"INSERT INTO worker_slots__p3a_new ({copy_csv}) "
                f"SELECT {copy_csv} FROM worker_slots"
            )
            conn.execute("DROP TABLE worker_slots")
            conn.execute("ALTER TABLE worker_slots__p3a_new RENAME TO worker_slots")
        finally:
            conn.execute("PRAGMA foreign_keys = ON")

    def check(self) -> dict[str, Any]:
        conn = self.connect()
        try:
            journal_mode = conn.execute("PRAGMA journal_mode").fetchone()[0]
            foreign_keys = conn.execute("PRAGMA foreign_keys").fetchone()[0]
            tables = {
                row[0]
                for row in conn.execute(
                    "SELECT name FROM sqlite_master WHERE type = 'table'"
                ).fetchall()
            }
            triggers = {
                row[0]
                for row in conn.execute(
                    "SELECT name FROM sqlite_master WHERE type = 'trigger'"
                ).fetchall()
            }
            version_row = conn.execute(
                "SELECT value FROM schema_meta WHERE key = 'schema_version'"
            ).fetchone()
            missing = sorted(REQUIRED_TABLES - tables)
            missing_columns: dict[str, list[str]] = {}
            for table, required_columns in REQUIRED_COLUMNS.items():
                if table not in tables:
                    continue
                existing_columns = {
                    row["name"]
                    for row in conn.execute(f"PRAGMA table_info({table})").fetchall()
                }
                missing_for_table = sorted(required_columns - existing_columns)
                if missing_for_table:
                    missing_columns[table] = missing_for_table
            ok = (
                journal_mode.lower() == "wal"
                and foreign_keys == 1
                and not missing
                and not missing_columns
                and {"events_no_update", "events_no_delete"}.issubset(triggers)
                and version_row is not None
                and version_row[0] == "2"
            )
            return {
                "ok": ok,
                "db_path": str(self.db_path),
                "journal_mode": journal_mode,
                "foreign_keys": foreign_keys,
                "schema_version": version_row[0] if version_row else None,
                "missing_tables": missing,
                "missing_columns": missing_columns,
                "event_triggers": sorted(triggers & {"events_no_update", "events_no_delete"}),
            }
        finally:
            conn.close()

    def append_event(
        self,
        event_type: str,
        task_id: str | None = None,
        payload: dict[str, Any] | None = None,
        event_id: str | None = None,
        created_at: str | None = None,
        project_id: str | None = None,
        session_id: str | None = None,
    ) -> str:
        """Append one append-only event row.

        TEIKOKU P2-A (master-plan 2.1): generalized into a per-project memory log.
        event_type is OPEN TEXT (no frozen CHECK enum / SCHEMA_VERSION treadmill).
        project_id defaults to ``DEFAULT_PROJECT_ID`` ("conductor") so the conductor's
        own ~50 existing positional callers keep working unchanged while every event is
        attributed to a project. session_id carries the cross-session-comms join (D-07).
        """
        event_id = event_id or str(uuid.uuid4())
        created_at = created_at or iso_utc()
        project_id = project_id or DEFAULT_PROJECT_ID
        payload_json = json_dumps(redact_json(payload or {}))
        with self.transaction() as conn:
            conn.execute(
                """
                INSERT INTO events
                    (event_id, task_id, event_type, payload_json, created_at, project_id, session_id)
                VALUES (?, ?, ?, ?, ?, ?, ?)
                """,
                (event_id, task_id, event_type, payload_json, created_at, project_id, session_id),
            )
        return event_id

    def upsert_task(self, task: TaskSpec, status: str = "PLANNED") -> None:
        now = iso_utc()
        values = task.db_values(now=now, status=status)
        columns = list(values.keys())
        placeholders = ",".join("?" for _ in columns)
        update_columns = [col for col in columns if col not in {"task_id", "created_at"}]
        update_sql = ",".join(f"{col}=excluded.{col}" for col in update_columns)
        sql = f"""
            INSERT INTO tasks ({",".join(columns)})
            VALUES ({placeholders})
            ON CONFLICT(task_id) DO UPDATE SET {update_sql}
        """
        with self.transaction() as conn:
            conn.execute(sql, tuple(values[col] for col in columns))
        self.append_event(
            "TASK_IMPORTED",
            task.task_id,
            {"plan_id": task.plan_id, "role": task.role, "status": status},
        )

    def add_task_dep(self, task_id: str, depends_on_task_id: str) -> None:
        with self.transaction() as conn:
            conn.execute(
                """
                INSERT OR IGNORE INTO task_deps (task_id, depends_on_task_id)
                VALUES (?, ?)
                """,
                (task_id, depends_on_task_id),
            )
        self.append_event(
            "TASK_DEP_RECORDED",
            task_id,
            {"depends_on_task_id": depends_on_task_id},
        )

    def get_task_row(self, task_id: str) -> sqlite3.Row | None:
        conn = self.connect()
        try:
            return conn.execute("SELECT * FROM tasks WHERE task_id = ?", (task_id,)).fetchone()
        finally:
            conn.close()

    def get_task_spec(self, task_id: str) -> TaskSpec | None:
        row = self.get_task_row(task_id)
        return TaskSpec.from_db_row(row) if row else None

    def update_task_status(self, task_id: str, new_status: str, reason: str) -> None:
        if new_status not in TASK_STATUSES:
            raise ValueError(f"invalid task status: {new_status}")
        now = iso_utc()
        with self.transaction() as conn:
            row = conn.execute(
                "SELECT status FROM tasks WHERE task_id = ?",
                (task_id,),
            ).fetchone()
            if row is None:
                raise KeyError(task_id)
            old_status = row["status"]
            conn.execute(
                "UPDATE tasks SET status = ?, updated_at = ? WHERE task_id = ?",
                (new_status, now, task_id),
            )
        self.append_event(
            "TASK_STATUS_CHANGED",
            task_id,
            {"old_status": old_status, "new_status": new_status, "reason": reason},
            created_at=now,
        )
        # TEIKOKU P3-D (D-07): wake any waiter blocked on this task as a dep. Best-effort
        # (try/except in wake.publish swallows a down hub); the watermark is the just-
        # appended row's seq (MAX(event_seq), monotonic AUTOINCREMENT).
        try:
            wake.publish(
                wake.latest_event_seq(self.db_path),
                "TASK_STATUS_CHANGED",
                task_id,
                None,
            )
        except Exception:
            pass

    def seed_worker_slot(
        self,
        slot_id: str,
        model: str,
        model_tier: str = "FREE",
        harness: str = "devin-cli",
        account_id: str | None = None,
        max_concurrency: int = 1,
        state: str = "IDLE",
        pool: str | None = None,
        roles: str | None = None,
    ) -> None:
        with self.transaction() as conn:
            conn.execute(
                """
                INSERT INTO worker_slots (
                  slot_id, harness, model, model_tier, account_id, max_concurrency, state, pool, roles
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(slot_id) DO UPDATE SET
                  harness=excluded.harness,
                  model=excluded.model,
                  model_tier=excluded.model_tier,
                  account_id=excluded.account_id,
                  max_concurrency=excluded.max_concurrency,
                  pool=excluded.pool,
                  roles=excluded.roles,
                  state=CASE
                    WHEN worker_slots.state IN ('LEASED','COOLDOWN') THEN worker_slots.state
                    ELSE excluded.state
                  END
                """,
                (slot_id, harness, model, model_tier, account_id, max_concurrency, state, pool, roles),
            )

    def insert_attempt(
        self,
        attempt_id: str,
        task_id: str,
        attempt_no: int,
        slot_id: str | None,
        model: str | None,
        started_at: str | None = None,
    ) -> None:
        with self.transaction() as conn:
            conn.execute(
                """
                INSERT INTO task_attempts (
                  attempt_id, task_id, attempt_no, slot_id, model, started_at
                )
                VALUES (?, ?, ?, ?, ?, ?)
                """,
                (attempt_id, task_id, attempt_no, slot_id, model, started_at or iso_utc()),
            )

    def finish_attempt(
        self,
        attempt_id: str,
        exit_code: int,
        status_hint: str,
        error_category: str | None,
        atif_path: str | None,
        diff_path: str | None,
        sanitized_notes: str,
    ) -> None:
        with self.transaction() as conn:
            conn.execute(
                """
                UPDATE task_attempts
                SET finished_at = ?, exit_code = ?, status_hint = ?, error_category = ?,
                    atif_path = ?, diff_path = ?, sanitized_notes = ?
                WHERE attempt_id = ?
                """,
                (
                    iso_utc(),
                    exit_code,
                    status_hint,
                    error_category,
                    atif_path,
                    diff_path,
                    sanitized_notes,
                    attempt_id,
                ),
            )

    def task_counts(self) -> dict[str, int]:
        conn = self.connect()
        try:
            return {
                row["status"]: row["n"]
                for row in conn.execute(
                    "SELECT status, COUNT(*) AS n FROM tasks GROUP BY status ORDER BY status"
                ).fetchall()
            }
        finally:
            conn.close()

    def deterministic_task_dump(self) -> list[dict[str, Any]]:
        conn = self.connect()
        try:
            rows = conn.execute(
                """
                SELECT task_id, plan_id, wave, role, objective, status, idem_key
                FROM tasks
                ORDER BY task_id
                """
            ).fetchall()
            return [dict(row) for row in rows]
        finally:
            conn.close()

    def ready_task_rows(self, wave: str | None = None) -> list[sqlite3.Row]:
        conn = self.connect()
        try:
            return conn.execute(
                """
                SELECT t.*
                FROM tasks t
                WHERE t.status = 'READY'
                  AND (? IS NULL OR t.wave = ?)
                  AND NOT EXISTS (
                    SELECT 1
                    FROM task_deps d
                    JOIN tasks dep ON dep.task_id = d.depends_on_task_id
                    WHERE d.task_id = t.task_id
                      AND dep.status != 'DONE'
                )
                ORDER BY t.priority DESC, t.created_at, t.task_id
                """,
                (wave, wave),
            ).fetchall()
        finally:
            conn.close()

    def wave_counts(self) -> list[dict[str, Any]]:
        conn = self.connect()
        try:
            rows = conn.execute(
                """
                SELECT wave, status, COUNT(*) AS n
                FROM tasks
                GROUP BY wave, status
                ORDER BY wave, status
                """
            ).fetchall()
            return [dict(row) for row in rows]
        finally:
            conn.close()

    def recent_events(
        self, limit: int = 50, project_id: str | None = None
    ) -> list[dict[str, Any]]:
        """Most-recent events newest-first. project_id=None => GLOBAL (master-plan 2.3)."""
        conn = self.connect()
        try:
            if project_id is None:
                rows = conn.execute(
                    """
                    SELECT event_seq, event_id, task_id, event_type, payload_json,
                           created_at, project_id, session_id
                    FROM events
                    ORDER BY event_seq DESC
                    LIMIT ?
                    """,
                    (max(0, int(limit)),),
                ).fetchall()
            else:
                rows = conn.execute(
                    """
                    SELECT event_seq, event_id, task_id, event_type, payload_json,
                           created_at, project_id, session_id
                    FROM events
                    WHERE project_id = ?
                    ORDER BY event_seq DESC
                    LIMIT ?
                    """,
                    (project_id, max(0, int(limit))),
                ).fetchall()
            return [dict(row) for row in rows]
        finally:
            conn.close()

    def query_events(
        self,
        project_id: str | None = None,
        session_id: str | None = None,
        event_type: str | None = None,
        task_id: str | None = None,
        limit: int = 200,
    ) -> list[dict[str, Any]]:
        """Typed scoped query over the memory log (generalizes ELAI ssb/store.rs typed
        query set: by_project / by_session / by_type / by_task). Any None filter is a
        wildcard; all-None => global recent. Newest-first. Returns parsed payload dicts."""
        clauses: list[str] = []
        params: list[Any] = []
        if project_id is not None:
            clauses.append("project_id = ?")
            params.append(project_id)
        if session_id is not None:
            clauses.append("session_id = ?")
            params.append(session_id)
        if event_type is not None:
            clauses.append("event_type = ?")
            params.append(event_type)
        if task_id is not None:
            clauses.append("task_id = ?")
            params.append(task_id)
        where = ("WHERE " + " AND ".join(clauses)) if clauses else ""
        params.append(max(0, int(limit)))
        conn = self.connect()
        try:
            rows = conn.execute(
                f"""
                SELECT event_seq, event_id, task_id, event_type, payload_json,
                       created_at, project_id, session_id
                FROM events
                {where}
                ORDER BY event_seq DESC
                LIMIT ?
                """,
                tuple(params),
            ).fetchall()
            out = []
            for row in rows:
                item = dict(row)
                try:
                    item["payload"] = json.loads(item.get("payload_json") or "{}")
                except (ValueError, TypeError):
                    item["payload"] = {}
                out.append(item)
            return out
        finally:
            conn.close()

    def distinct_projects(self) -> list[str]:
        """All project_ids present in the memory log (feeds the global index + dashboard)."""
        conn = self.connect()
        try:
            rows = conn.execute(
                "SELECT DISTINCT project_id FROM events WHERE project_id IS NOT NULL ORDER BY project_id"
            ).fetchall()
            return [row[0] for row in rows]
        finally:
            conn.close()

    def record_verdict(
        self,
        task_id: str,
        gate_name: str,
        verdict: str,
        output: str,
        timestamp: str | None = None,
    ) -> None:
        timestamp = timestamp or iso_utc()
        with self.transaction() as conn:
            conn.execute(
                """
                INSERT INTO verdicts (task_id, gate_name, verdict, output, timestamp)
                VALUES (?, ?, ?, ?, ?)
                ON CONFLICT(task_id, gate_name) DO UPDATE SET
                  verdict=excluded.verdict,
                  output=excluded.output,
                  timestamp=excluded.timestamp
                """,
                (task_id, gate_name, verdict, output, timestamp),
            )

    def get_verdicts(self, task_id: str) -> list[dict]:
        conn = self.connect()
        try:
            rows = conn.execute(
                "SELECT * FROM verdicts WHERE task_id = ? ORDER BY gate_name",
                (task_id,),
            ).fetchall()
            return [dict(row) for row in rows]
        finally:
            conn.close()


def db_json_check(db_path: str | Path) -> str:
    return json.dumps(ControlDB(db_path).check(), indent=2, sort_keys=True)
