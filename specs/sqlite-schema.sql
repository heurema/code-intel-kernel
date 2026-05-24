-- SQLite schema draft for Code Intelligence Kernel.
-- Milestone 1 should stay local-first and simple.

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS nodes (
  id TEXT PRIMARY KEY,
  kind TEXT NOT NULL,
  name TEXT NOT NULL,
  path TEXT,
  start_line INTEGER,
  end_line INTEGER,
  hash TEXT,
  metadata_json TEXT
);

CREATE INDEX IF NOT EXISTS idx_nodes_kind ON nodes(kind);
CREATE INDEX IF NOT EXISTS idx_nodes_path ON nodes(path);
CREATE INDEX IF NOT EXISTS idx_nodes_name ON nodes(name);

CREATE TABLE IF NOT EXISTS edges (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  src_id TEXT NOT NULL,
  dst_id TEXT NOT NULL,
  kind TEXT NOT NULL,
  confidence REAL NOT NULL DEFAULT 1.0,
  evidence_json TEXT,
  FOREIGN KEY (src_id) REFERENCES nodes(id) ON DELETE CASCADE,
  FOREIGN KEY (dst_id) REFERENCES nodes(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_edges_src ON edges(src_id);
CREATE INDEX IF NOT EXISTS idx_edges_dst ON edges(dst_id);
CREATE INDEX IF NOT EXISTS idx_edges_kind ON edges(kind);

CREATE TABLE IF NOT EXISTS commands (
  id TEXT PRIMARY KEY,
  scope TEXT,
  kind TEXT NOT NULL,
  command TEXT NOT NULL,
  confidence REAL NOT NULL DEFAULT 1.0,
  evidence_json TEXT
);

CREATE TABLE IF NOT EXISTS diagnostics (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  run_id TEXT NOT NULL,
  path TEXT NOT NULL,
  line INTEGER,
  column INTEGER,
  severity TEXT,
  code TEXT,
  message TEXT NOT NULL,
  source TEXT,
  created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_diagnostics_run ON diagnostics(run_id);
CREATE INDEX IF NOT EXISTS idx_diagnostics_path ON diagnostics(path);

CREATE TABLE IF NOT EXISTS episodes (
  id TEXT PRIMARY KEY,
  task_id TEXT,
  event_type TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_episodes_task ON episodes(task_id);
CREATE INDEX IF NOT EXISTS idx_episodes_type ON episodes(event_type);

CREATE TABLE IF NOT EXISTS decisions (
  id TEXT PRIMARY KEY,
  consumer TEXT,
  decision TEXT NOT NULL,
  reason TEXT,
  prevents TEXT,
  review_date TEXT,
  created_at TEXT NOT NULL
);

-- Optional FTS tables for comments/docs/snippets.
CREATE VIRTUAL TABLE IF NOT EXISTS snippets_fts USING fts5(
  path,
  title,
  body,
  tokenize = 'unicode61'
);
