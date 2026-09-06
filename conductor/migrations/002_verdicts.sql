CREATE TABLE IF NOT EXISTS verdicts (
  task_id TEXT NOT NULL,
  gate_name TEXT NOT NULL,
  verdict TEXT NOT NULL,
  output TEXT,
  timestamp TEXT NOT NULL,
  PRIMARY KEY (task_id, gate_name)
);

CREATE INDEX IF NOT EXISTS idx_verdicts_task ON verdicts(task_id);
