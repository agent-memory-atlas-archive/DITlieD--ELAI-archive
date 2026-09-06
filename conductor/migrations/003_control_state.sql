CREATE TABLE IF NOT EXISTS control_state (
  key TEXT PRIMARY KEY,
  value_json TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_control_state_key
  ON control_state(key);
