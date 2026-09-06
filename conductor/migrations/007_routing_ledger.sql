-- 007_routing_ledger.sql
-- TEIKOKU-IR (intelligent routing) TR-5: the outcome-ledger the deterministic pre-router
-- (route_decider.ledger_veto) reads to swap OFF a (task-shape, model) pair with a bad recent
-- track record BEFORE it burns another attempt. One row per worker disposition, written from
-- scheduler.record_worker_result. Pure substrate: NO ML training in this plan (N-03) -- a trained
-- router over these rows is a future unit.
--
-- Idempotent + re-runnable (CREATE ... IF NOT EXISTS), because ControlDB.init() re-runs every
-- migration via executescript() on every boot (same pattern as 001_initial.sql). The live-DB
-- path is also covered defensively by routing_ledger.ensure_table (CREATE IF NOT EXISTS on first
-- write), so a fail-soft write never depends on init() having run.
--
-- task-shape features (the veto key): role + difficulty + size_class + tool_heavy. outcome:
-- model/pool/model_tier + verified (1 = the gate certified the result, 0 = failure/overflow/
-- needs-review) + overflow (1 = a context_window_blocked signal) + cost_cents (from the W8 ledger).

CREATE TABLE IF NOT EXISTS routing_ledger (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  created_at TEXT NOT NULL,
  task_id TEXT,
  role TEXT NOT NULL,
  difficulty TEXT NOT NULL,
  size_class TEXT NOT NULL,
  tool_heavy INTEGER NOT NULL DEFAULT 0,
  pool TEXT,
  model TEXT NOT NULL,
  model_tier TEXT,
  status TEXT,
  verified INTEGER NOT NULL DEFAULT 0,
  overflow INTEGER NOT NULL DEFAULT 0,
  cost_cents INTEGER NOT NULL DEFAULT 0,
  reason TEXT
);

-- The shape+model lookup ledger_veto runs (recent rows for this task-shape, newest-first).
CREATE INDEX IF NOT EXISTS idx_routing_ledger_shape
  ON routing_ledger (role, difficulty, size_class, tool_heavy, id DESC);
CREATE INDEX IF NOT EXISTS idx_routing_ledger_model
  ON routing_ledger (model, id DESC);
