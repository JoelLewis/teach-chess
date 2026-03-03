CREATE TABLE IF NOT EXISTS coaching_cache (
    cache_key     TEXT PRIMARY KEY,
    coaching_text TEXT NOT NULL,
    player_level  TEXT NOT NULL,
    classification TEXT,
    fen           TEXT,
    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at    TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_coaching_cache_expires ON coaching_cache(expires_at);
