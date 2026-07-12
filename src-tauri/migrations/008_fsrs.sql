-- 008: Replace per-attempt SM-2 scheduling with FSRS cards.
--
-- Scheduling state moves out of the attempt tables into a dedicated
-- srs_card table (one card per player/item, mirroring rs-fsrs's Card).
-- Existing SM-2 state is converted with a documented approximation:
--
--   due            = srs_next_review        (preserved — review queues do not reset)
--   stability      = srs_interval            (FSRS stability ~= interval at 90% retention)
--   difficulty     = clamp(5 + (2.5 - ease) * (5 / 1.2), 1, 10)
--                    (linear map: default ease 2.5 -> 5.0, floor ease 1.3 -> 10.0)
--   last_review    = due - interval days     (reconstructed; attempted_at was unreliable)
--   scheduled_days = round(interval)
--   elapsed_days   = 0
--   reps           = total attempts, lapses = failed attempts
--   state          = 2 (Review) — every migrated card has been seen at least once
--
-- NOT idempotent by itself: the runner skips this file when srs_card exists.
BEGIN;

CREATE TABLE srs_card (
    player_id      TEXT NOT NULL REFERENCES player(id),
    item_type      TEXT NOT NULL CHECK(item_type IN ('puzzle', 'drill')),
    item_id        TEXT NOT NULL,
    due            TEXT NOT NULL,
    stability      REAL NOT NULL DEFAULT 0,
    difficulty     REAL NOT NULL DEFAULT 0,
    elapsed_days   INTEGER NOT NULL DEFAULT 0,
    scheduled_days INTEGER NOT NULL DEFAULT 0,
    reps           INTEGER NOT NULL DEFAULT 0,
    lapses         INTEGER NOT NULL DEFAULT 0,
    state          INTEGER NOT NULL DEFAULT 0,
    last_review    TEXT NOT NULL,
    PRIMARY KEY (player_id, item_type, item_id)
);

CREATE INDEX idx_srs_card_due ON srs_card(player_id, item_type, due);

-- Backfill puzzle cards from the latest attempt per (player, puzzle).
INSERT INTO srs_card (player_id, item_type, item_id, due, stability, difficulty,
                      elapsed_days, scheduled_days, reps, lapses, state, last_review)
SELECT pa.player_id, 'puzzle', pa.puzzle_id,
       pa.srs_next_review,
       pa.srs_interval,
       MIN(10.0, MAX(1.0, 5.0 + (2.5 - pa.srs_ease) * (5.0 / 1.2))),
       0,
       CAST(ROUND(pa.srs_interval) AS INTEGER),
       agg.reps,
       agg.lapses,
       2,
       datetime(julianday(pa.srs_next_review) - pa.srs_interval)
FROM puzzle_attempt pa
JOIN (
    SELECT player_id, puzzle_id, MAX(rowid) AS latest_rowid,
           COUNT(*) AS reps,
           SUM(CASE WHEN solved = 0 THEN 1 ELSE 0 END) AS lapses
    FROM puzzle_attempt
    GROUP BY player_id, puzzle_id
) agg ON agg.latest_rowid = pa.rowid;

-- Backfill drill cards from the latest attempt per (player, repertoire entry).
INSERT INTO srs_card (player_id, item_type, item_id, due, stability, difficulty,
                      elapsed_days, scheduled_days, reps, lapses, state, last_review)
SELECT rda.player_id, 'drill', rda.repertoire_entry_id,
       rda.srs_next_review,
       rda.srs_interval,
       MIN(10.0, MAX(1.0, 5.0 + (2.5 - rda.srs_ease) * (5.0 / 1.2))),
       0,
       CAST(ROUND(rda.srs_interval) AS INTEGER),
       agg.reps,
       agg.lapses,
       2,
       datetime(julianday(rda.srs_next_review) - rda.srs_interval)
FROM repertoire_drill_attempt rda
JOIN (
    SELECT player_id, repertoire_entry_id, MAX(rowid) AS latest_rowid,
           COUNT(*) AS reps,
           SUM(CASE WHEN correct = 0 THEN 1 ELSE 0 END) AS lapses
    FROM repertoire_drill_attempt
    GROUP BY player_id, repertoire_entry_id
) agg ON agg.latest_rowid = rda.rowid;

-- Attempt tables are pure history now — drop the SM-2 columns.
DROP INDEX IF EXISTS idx_puzzle_attempt_review;
DROP INDEX IF EXISTS idx_drill_attempt_review;
ALTER TABLE puzzle_attempt DROP COLUMN srs_interval;
ALTER TABLE puzzle_attempt DROP COLUMN srs_ease;
ALTER TABLE puzzle_attempt DROP COLUMN srs_next_review;
ALTER TABLE repertoire_drill_attempt DROP COLUMN srs_interval;
ALTER TABLE repertoire_drill_attempt DROP COLUMN srs_ease;
ALTER TABLE repertoire_drill_attempt DROP COLUMN srs_next_review;

COMMIT;
