CREATE TABLE IF NOT EXISTS player (
    id            TEXT PRIMARY KEY,
    display_name  TEXT NOT NULL,
    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    games_played  INTEGER NOT NULL DEFAULT 0,
    settings_json TEXT NOT NULL DEFAULT '{}'
);

CREATE TABLE IF NOT EXISTS game (
    id            TEXT PRIMARY KEY,
    player_id     TEXT NOT NULL REFERENCES player(id),
    pgn           TEXT NOT NULL,
    result        TEXT NOT NULL,
    player_color  TEXT NOT NULL,
    engine_elo    INTEGER NOT NULL,
    move_count    INTEGER NOT NULL,
    started_at    TEXT NOT NULL,
    ended_at      TEXT,
    time_control  TEXT NOT NULL DEFAULT 'unlimited',
    created_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS move_annotation (
    id              TEXT PRIMARY KEY,
    game_id         TEXT NOT NULL REFERENCES game(id) ON DELETE CASCADE,
    move_number     INTEGER NOT NULL,
    is_white        INTEGER NOT NULL,
    fen_before      TEXT NOT NULL,
    player_move_uci TEXT NOT NULL,
    player_move_san TEXT NOT NULL,
    engine_best_uci TEXT,
    engine_best_san TEXT,
    eval_before_cp  INTEGER,
    eval_after_cp   INTEGER,
    eval_before_mate INTEGER,
    eval_after_mate  INTEGER,
    classification  TEXT,
    depth           INTEGER,
    pv_json         TEXT,
    coaching_text   TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);
