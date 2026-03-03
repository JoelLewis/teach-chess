CREATE TABLE IF NOT EXISTS puzzle (
    id              TEXT PRIMARY KEY,
    fen             TEXT NOT NULL,
    solution_moves  TEXT NOT NULL,
    themes          TEXT NOT NULL DEFAULT '',
    category        TEXT NOT NULL DEFAULT 'tactical',
    difficulty      INTEGER NOT NULL DEFAULT 1500,
    source_id       TEXT,
    hints_json      TEXT NOT NULL DEFAULT '[]',
    explanation     TEXT NOT NULL DEFAULT '',
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_puzzle_difficulty ON puzzle(difficulty);
CREATE INDEX IF NOT EXISTS idx_puzzle_category ON puzzle(category);

CREATE TABLE IF NOT EXISTS puzzle_attempt (
    id              TEXT PRIMARY KEY,
    player_id       TEXT NOT NULL REFERENCES player(id),
    puzzle_id       TEXT NOT NULL REFERENCES puzzle(id),
    solved          INTEGER NOT NULL DEFAULT 0,
    time_ms         INTEGER NOT NULL DEFAULT 0,
    hints_used      INTEGER NOT NULL DEFAULT 0,
    attempted_at    TEXT NOT NULL DEFAULT (datetime('now')),
    srs_interval    REAL NOT NULL DEFAULT 1.0,
    srs_ease        REAL NOT NULL DEFAULT 2.5,
    srs_next_review TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_puzzle_attempt_player ON puzzle_attempt(player_id);
CREATE INDEX IF NOT EXISTS idx_puzzle_attempt_review ON puzzle_attempt(player_id, puzzle_id, srs_next_review);
