use chrono::{DateTime, NaiveDateTime, Utc};
use rs_fsrs::{Card, State};
use rusqlite::params;

use super::OptionalRow;
use crate::db::connection::Database;
use crate::error::DatabaseError;

/// Which kind of item an SRS card schedules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SrsItemKind {
    Puzzle,
    Drill,
}

impl SrsItemKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Puzzle => "puzzle",
            Self::Drill => "drill",
        }
    }
}

/// Matches SQLite's `datetime('now')` format so due-date comparisons in SQL
/// stay lexicographically correct.
const DATETIME_FMT: &str = "%Y-%m-%d %H:%M:%S";

impl Database {
    /// Read the FSRS card for an item, or a fresh card if none exists.
    pub fn get_srs_card(
        &self,
        player_id: &str,
        kind: SrsItemKind,
        item_id: &str,
    ) -> Result<Card, DatabaseError> {
        let card = self
            .conn()
            .query_row(
                "SELECT due, stability, difficulty, elapsed_days, scheduled_days,
                        reps, lapses, state, last_review
                 FROM srs_card WHERE player_id = ?1 AND item_type = ?2 AND item_id = ?3",
                params![player_id, kind.as_str(), item_id],
                |row| {
                    Ok(Card {
                        due: parse_datetime(&row.get::<_, String>(0)?),
                        stability: row.get(1)?,
                        difficulty: row.get(2)?,
                        elapsed_days: row.get(3)?,
                        scheduled_days: row.get(4)?,
                        reps: row.get(5)?,
                        lapses: row.get(6)?,
                        state: int_to_state(row.get::<_, i64>(7)?),
                        last_review: parse_datetime(&row.get::<_, String>(8)?),
                    })
                },
            )
            .optional()?;
        Ok(card.unwrap_or_else(Card::new))
    }

    /// Insert or update the FSRS card for an item.
    pub fn upsert_srs_card(
        &self,
        player_id: &str,
        kind: SrsItemKind,
        item_id: &str,
        card: &Card,
    ) -> Result<(), DatabaseError> {
        self.conn().execute(
            "INSERT INTO srs_card (player_id, item_type, item_id, due, stability, difficulty,
                                   elapsed_days, scheduled_days, reps, lapses, state, last_review)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT(player_id, item_type, item_id) DO UPDATE SET
                 due = ?4, stability = ?5, difficulty = ?6, elapsed_days = ?7,
                 scheduled_days = ?8, reps = ?9, lapses = ?10, state = ?11, last_review = ?12",
            params![
                player_id,
                kind.as_str(),
                item_id,
                card.due.format(DATETIME_FMT).to_string(),
                card.stability,
                card.difficulty,
                card.elapsed_days,
                card.scheduled_days,
                card.reps,
                card.lapses,
                card.state as i64,
                card.last_review.format(DATETIME_FMT).to_string(),
            ],
        )?;
        Ok(())
    }
}

fn parse_datetime(s: &str) -> DateTime<Utc> {
    NaiveDateTime::parse_from_str(s, DATETIME_FMT)
        .map(|dt| dt.and_utc())
        .unwrap_or_else(|_| Utc::now())
}

fn int_to_state(i: i64) -> State {
    match i {
        1 => State::Learning,
        2 => State::Review,
        3 => State::Relearning,
        _ => State::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::srs;
    use rs_fsrs::Rating;

    fn test_db() -> Database {
        let db = Database::open_in_memory().unwrap();
        db.conn()
            .execute(
                "INSERT INTO player (id, display_name) VALUES ('p1', 'Test')",
                [],
            )
            .unwrap();
        db
    }

    #[test]
    fn missing_card_is_new() {
        let db = test_db();
        let card = db.get_srs_card("p1", SrsItemKind::Puzzle, "pz1").unwrap();
        assert_eq!(card.reps, 0);
        assert_eq!(card.state, State::New);
    }

    #[test]
    fn card_roundtrips_through_db() {
        let db = test_db();
        let card = srs::next_card(Card::new(), Rating::Good);
        db.upsert_srs_card("p1", SrsItemKind::Puzzle, "pz1", &card)
            .unwrap();

        let loaded = db.get_srs_card("p1", SrsItemKind::Puzzle, "pz1").unwrap();
        assert_eq!(loaded.reps, card.reps);
        assert_eq!(loaded.state, card.state);
        assert!((loaded.stability - card.stability).abs() < 1e-9);
        assert!((loaded.difficulty - card.difficulty).abs() < 1e-9);
        // Datetimes survive with second precision
        assert_eq!(
            loaded.due.format(DATETIME_FMT).to_string(),
            card.due.format(DATETIME_FMT).to_string()
        );
    }

    #[test]
    fn upsert_replaces_existing_card() {
        let db = test_db();
        let first = srs::next_card(Card::new(), Rating::Good);
        db.upsert_srs_card("p1", SrsItemKind::Drill, "e1", &first)
            .unwrap();
        let second = srs::next_card(first, Rating::Easy);
        db.upsert_srs_card("p1", SrsItemKind::Drill, "e1", &second)
            .unwrap();

        let loaded = db.get_srs_card("p1", SrsItemKind::Drill, "e1").unwrap();
        assert_eq!(loaded.reps, 2);
    }

    #[test]
    fn kinds_are_isolated() {
        let db = test_db();
        let card = srs::next_card(Card::new(), Rating::Good);
        db.upsert_srs_card("p1", SrsItemKind::Puzzle, "x", &card)
            .unwrap();

        let drill = db.get_srs_card("p1", SrsItemKind::Drill, "x").unwrap();
        assert_eq!(drill.state, State::New);
    }
}
