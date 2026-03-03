CREATE TABLE IF NOT EXISTS opening (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  eco TEXT NOT NULL DEFAULT '',
  color TEXT NOT NULL CHECK(color IN ('white','black')),
  description TEXT NOT NULL DEFAULT '',
  moves TEXT NOT NULL,
  themes TEXT NOT NULL DEFAULT '',
  difficulty INTEGER NOT NULL DEFAULT 1200,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS opening_position (
  id TEXT PRIMARY KEY,
  opening_id TEXT NOT NULL REFERENCES opening(id) ON DELETE CASCADE,
  fen TEXT NOT NULL,
  move_index INTEGER NOT NULL,
  parent_fen TEXT,
  move_uci TEXT NOT NULL,
  move_san TEXT NOT NULL DEFAULT ''
);
CREATE INDEX IF NOT EXISTS idx_opening_position_fen ON opening_position(fen);
CREATE INDEX IF NOT EXISTS idx_opening_position_opening ON opening_position(opening_id);

CREATE TABLE IF NOT EXISTS repertoire_entry (
  id TEXT PRIMARY KEY,
  player_id TEXT NOT NULL REFERENCES player(id),
  opening_id TEXT NOT NULL REFERENCES opening(id),
  position_fen TEXT NOT NULL,
  move_uci TEXT NOT NULL,
  move_san TEXT NOT NULL DEFAULT '',
  notes TEXT NOT NULL DEFAULT '',
  added_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_repertoire_player ON repertoire_entry(player_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_repertoire_unique
  ON repertoire_entry(player_id, opening_id, position_fen);

CREATE TABLE IF NOT EXISTS repertoire_drill_attempt (
  id TEXT PRIMARY KEY,
  player_id TEXT NOT NULL REFERENCES player(id),
  repertoire_entry_id TEXT NOT NULL REFERENCES repertoire_entry(id),
  correct INTEGER NOT NULL DEFAULT 0,
  time_ms INTEGER NOT NULL DEFAULT 0,
  attempted_at TEXT NOT NULL DEFAULT (datetime('now')),
  srs_interval REAL NOT NULL DEFAULT 1.0,
  srs_ease REAL NOT NULL DEFAULT 2.5,
  srs_next_review TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_drill_attempt_entry ON repertoire_drill_attempt(repertoire_entry_id);
CREATE INDEX IF NOT EXISTS idx_drill_attempt_review ON repertoire_drill_attempt(srs_next_review);
