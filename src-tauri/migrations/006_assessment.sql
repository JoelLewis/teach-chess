CREATE TABLE IF NOT EXISTS skill_rating (
  id TEXT PRIMARY KEY,
  player_id TEXT NOT NULL REFERENCES player(id),
  category TEXT NOT NULL,
  rating REAL NOT NULL DEFAULT 1200.0,
  rd REAL NOT NULL DEFAULT 350.0,
  volatility REAL NOT NULL DEFAULT 0.06,
  games_count INTEGER NOT NULL DEFAULT 0,
  last_updated TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE(player_id, category)
);
CREATE INDEX IF NOT EXISTS idx_skill_rating_player ON skill_rating(player_id);
