-- Add coaching_text column to move_annotation for template-based coaching feedback.
-- Uses ALTER TABLE to handle existing databases (001_initial.sql covers new databases).
-- SQLite ALTER TABLE ADD COLUMN is safe: it's a no-op metadata change, no table rewrite.
ALTER TABLE move_annotation ADD COLUMN coaching_text TEXT;
