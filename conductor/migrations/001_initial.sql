PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;

CREATE TABLE IF NOT EXISTS schema_meta (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

INSERT INTO schema_meta (key, value)
VALUES ('schema_version', '2')
ON CONFLICT(key) DO UPDATE SET value = excluded.value;

CREATE TABLE IF NOT EXISTS tasks (
  task_id TEXT PRIMARY KEY,
  plan_id TEXT NOT NULL,
  wave TEXT NOT NULL,
  role TEXT NOT NULL CHECK (role IN (
    'PLAN','EXECUTE','WIRE','INTEGRATE','AUDIT','POLISH','FIX'
  )),
  objective TEXT NOT NULL,
  base_commit TEXT NOT NULL,
  allowed_write_set_json TEXT NOT NULL,
  context_refs_json TEXT NOT NULL,
  production_hook TEXT,
  acceptance_text TEXT NOT NULL,
  deterministic_check_text TEXT,
  forbidden_text TEXT NOT NULL,
  verifier_json TEXT NOT NULL,
  lease_ttl_seconds INTEGER NOT NULL,
  idem_key TEXT NOT NULL UNIQUE,
  token_budget INTEGER NOT NULL DEFAULT 100000,
  model_tier TEXT NOT NULL CHECK (model_tier IN ('FREE','PAID')),
  priority INTEGER NOT NULL DEFAULT 0,
  status TEXT NOT NULL CHECK (status IN (
    'PLANNED','DISCOVERED','ACCEPTED','READY','LEASED','RUNNING',
    'VERIFYING','VERIFIED','INTEGRATING','DONE','NEEDS_REVIEW',
    'BLOCKED','FAILED','CANCELLED','SUPERSEDED','WAITING_FOR_VERIFIER'
  )),
  attempts INTEGER NOT NULL DEFAULT 0,
  max_attempts INTEGER NOT NULL DEFAULT 3,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS task_deps (
  task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
  depends_on_task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
  PRIMARY KEY (task_id, depends_on_task_id),
  CHECK (task_id <> depends_on_task_id)
);

CREATE TABLE IF NOT EXISTS worker_slots (
  slot_id TEXT PRIMARY KEY,
  harness TEXT NOT NULL CHECK (harness IN ('devin-cli')),
  model TEXT NOT NULL,
  model_tier TEXT NOT NULL CHECK (model_tier IN ('FREE','PAID')),
  account_id TEXT,
  max_concurrency INTEGER NOT NULL DEFAULT 1,
  state TEXT NOT NULL CHECK (state IN ('IDLE','LEASED','COOLDOWN','DISABLED')),
  cooldown_until TEXT,
  last_error_category TEXT
);

CREATE TABLE IF NOT EXISTS leases (
  lease_id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL UNIQUE REFERENCES tasks(task_id) ON DELETE CASCADE,
  slot_id TEXT NOT NULL REFERENCES worker_slots(slot_id),
  worktree_path TEXT NOT NULL,
  branch_name TEXT NOT NULL,
  session_id TEXT,
  lease_started_at TEXT NOT NULL,
  lease_expires_at TEXT NOT NULL,
  heartbeat_at TEXT NOT NULL,
  disconnected_after_mutation INTEGER NOT NULL DEFAULT 0,
  worker_pid INTEGER
);

CREATE TABLE IF NOT EXISTS task_attempts (
  attempt_id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
  attempt_no INTEGER NOT NULL,
  slot_id TEXT,
  model TEXT,
  started_at TEXT NOT NULL,
  finished_at TEXT,
  exit_code INTEGER,
  status_hint TEXT,
  error_category TEXT,
  atif_path TEXT,
  diff_path TEXT,
  sanitized_notes TEXT,
  UNIQUE (task_id, attempt_no)
);

CREATE TABLE IF NOT EXISTS gate_runs (
  gate_run_id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
  attempt_id TEXT REFERENCES task_attempts(attempt_id) ON DELETE SET NULL,
  gate_name TEXT NOT NULL,
  command_json TEXT NOT NULL,
  cwd TEXT NOT NULL,
  started_at TEXT NOT NULL,
  finished_at TEXT,
  exit_code INTEGER,
  verdict TEXT NOT NULL CHECK (verdict IN ('PASS','FAIL','ERROR','TIMEOUT')),
  stdout_artifact TEXT,
  stderr_artifact TEXT,
  summary TEXT,
  UNIQUE (task_id, gate_name, started_at)
);

CREATE TABLE IF NOT EXISTS integrations (
  integration_id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
  source_worktree_path TEXT NOT NULL,
  source_base_commit TEXT NOT NULL,
  integration_worktree_path TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN (
    'PENDING','APPLIED','CONFLICT','POST_GATE_FAILED','DONE','NEEDS_REVIEW'
  )),
  started_at TEXT,
  finished_at TEXT,
  patch_artifact TEXT,
  post_gate_run_id TEXT REFERENCES gate_runs(gate_run_id)
);

CREATE TABLE IF NOT EXISTS artifacts (
  artifact_id TEXT PRIMARY KEY,
  task_id TEXT REFERENCES tasks(task_id) ON DELETE CASCADE,
  kind TEXT NOT NULL,
  path TEXT NOT NULL,
  blake3 TEXT,
  created_at TEXT NOT NULL,
  redacted INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS model_cooldowns (
  model TEXT PRIMARY KEY,
  cooldown_until TEXT NOT NULL,
  reason TEXT NOT NULL,
  observed_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS budget_counters (
  budget_key TEXT PRIMARY KEY,
  limit_value INTEGER NOT NULL,
  used_value INTEGER NOT NULL DEFAULT 0,
  reset_at TEXT,
  updated_at TEXT NOT NULL,
  CHECK (limit_value >= 0),
  CHECK (used_value >= 0)
);

CREATE TABLE IF NOT EXISTS events (
  event_seq INTEGER PRIMARY KEY AUTOINCREMENT,
  event_id TEXT NOT NULL UNIQUE,
  task_id TEXT,
  event_type TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TRIGGER IF NOT EXISTS events_no_update
BEFORE UPDATE ON events
BEGIN
  SELECT RAISE(ABORT, 'events are append-only');
END;

CREATE TRIGGER IF NOT EXISTS events_no_delete
BEFORE DELETE ON events
BEGIN
  SELECT RAISE(ABORT, 'events are append-only');
END;

CREATE INDEX IF NOT EXISTS idx_tasks_status_priority
  ON tasks(status, priority DESC, created_at);
CREATE INDEX IF NOT EXISTS idx_task_deps_dep
  ON task_deps(depends_on_task_id);
CREATE INDEX IF NOT EXISTS idx_gate_runs_task
  ON gate_runs(task_id, gate_name);
CREATE INDEX IF NOT EXISTS idx_events_task
  ON events(task_id, event_seq);
CREATE INDEX IF NOT EXISTS idx_leases_expiry
  ON leases(lease_expires_at, heartbeat_at);
CREATE INDEX IF NOT EXISTS idx_attempts_task
  ON task_attempts(task_id, attempt_no);
