use rusqlite::params;

use super::OptionalRow;
use crate::db::connection::Database;
use crate::error::DatabaseError;
use crate::models::assessment::{SkillProfile, SkillRating};

impl Database {
    /// Get all skill ratings for a player.
    pub fn get_skill_ratings(&self, player_id: &str) -> Result<Vec<SkillRating>, DatabaseError> {
        let mut stmt = self.conn().prepare(
            "SELECT id, player_id, category, rating, rd, volatility, games_count, last_updated
             FROM skill_rating WHERE player_id = ?1 ORDER BY category",
        )?;
        let rows = stmt.query_map(params![player_id], |row| {
            Ok(SkillRating {
                id: row.get(0)?,
                player_id: row.get(1)?,
                category: row.get(2)?,
                rating: row.get(3)?,
                rd: row.get(4)?,
                volatility: row.get(5)?,
                games_count: row.get::<_, i64>(6)? as u32,
                last_updated: row.get(7)?,
            })
        })?;

        let mut ratings = Vec::new();
        for row in rows {
            ratings.push(row?);
        }
        Ok(ratings)
    }

    /// Get a single skill rating by player and category.
    pub fn get_skill_rating(
        &self,
        player_id: &str,
        category: &str,
    ) -> Result<Option<SkillRating>, DatabaseError> {
        let result = self
            .conn()
            .query_row(
                "SELECT id, player_id, category, rating, rd, volatility, games_count, last_updated
                 FROM skill_rating WHERE player_id = ?1 AND category = ?2",
                params![player_id, category],
                |row| {
                    Ok(SkillRating {
                        id: row.get(0)?,
                        player_id: row.get(1)?,
                        category: row.get(2)?,
                        rating: row.get(3)?,
                        rd: row.get(4)?,
                        volatility: row.get(5)?,
                        games_count: row.get::<_, i64>(6)? as u32,
                        last_updated: row.get(7)?,
                    })
                },
            )
            .optional()?;
        Ok(result)
    }

    /// Upsert a skill rating (create or update).
    pub fn upsert_skill_rating(&self, rating: &SkillRating) -> Result<(), DatabaseError> {
        self.conn().execute(
            "INSERT INTO skill_rating (id, player_id, category, rating, rd, volatility, games_count, last_updated)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'))
             ON CONFLICT(player_id, category) DO UPDATE SET
               rating = excluded.rating,
               rd = excluded.rd,
               volatility = excluded.volatility,
               games_count = excluded.games_count,
               last_updated = datetime('now')",
            params![
                rating.id,
                rating.player_id,
                rating.category,
                rating.rating,
                rating.rd,
                rating.volatility,
                rating.games_count as i64,
            ],
        )?;
        Ok(())
    }

    /// Get a computed skill profile for a player.
    pub fn get_skill_profile(&self, player_id: &str) -> Result<SkillProfile, DatabaseError> {
        let ratings = self.get_skill_ratings(player_id)?;

        if ratings.is_empty() {
            return Ok(SkillProfile {
                ratings: Vec::new(),
                overall_rating: 1200.0,
                strongest_category: None,
                weakest_category: None,
            });
        }

        // Weighted average by games_count (more games = more weight)
        let total_games: u32 = ratings.iter().map(|r| r.games_count).sum();
        let overall_rating = if total_games == 0 {
            1200.0
        } else {
            ratings
                .iter()
                .map(|r| r.rating * r.games_count as f64)
                .sum::<f64>()
                / total_games as f64
        };

        // Only consider categories with at least 3 games for strongest/weakest
        let strongest = ratings
            .iter()
            .filter(|r| r.games_count >= 3)
            .max_by(|a, b| a.rating.partial_cmp(&b.rating).unwrap())
            .map(|r| r.category.clone());
        let weakest = ratings
            .iter()
            .filter(|r| r.games_count >= 3)
            .min_by(|a, b| a.rating.partial_cmp(&b.rating).unwrap())
            .map(|r| r.category.clone());

        Ok(SkillProfile {
            ratings,
            overall_rating,
            strongest_category: strongest,
            weakest_category: weakest,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::assessment::SkillRating;
    use rusqlite::Connection;

    fn setup_test_db() -> Database {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(include_str!("../../migrations/001_initial.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../../migrations/006_assessment.sql"))
            .unwrap();
        conn.execute(
            "INSERT INTO player (id, display_name) VALUES ('p1', 'Test')",
            [],
        )
        .unwrap();
        Database::from_connection(conn)
    }

    #[test]
    fn upsert_and_query_rating() {
        let db = setup_test_db();
        let rating = SkillRating {
            id: "r1".to_string(),
            player_id: "p1".to_string(),
            category: "tactical".to_string(),
            rating: 1300.0,
            rd: 200.0,
            volatility: 0.06,
            games_count: 5,
            last_updated: String::new(),
        };
        db.upsert_skill_rating(&rating).unwrap();

        let result = db.get_skill_rating("p1", "tactical").unwrap();
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.rating, 1300.0);
        assert_eq!(r.games_count, 5);
    }

    #[test]
    fn upsert_updates_existing() {
        let db = setup_test_db();
        let rating = SkillRating::default_for("p1", "tactical");
        db.upsert_skill_rating(&rating).unwrap();

        let updated = SkillRating {
            rating: 1350.0,
            games_count: 10,
            ..rating
        };
        db.upsert_skill_rating(&updated).unwrap();

        let result = db.get_skill_rating("p1", "tactical").unwrap().unwrap();
        assert_eq!(result.rating, 1350.0);
        assert_eq!(result.games_count, 10);
    }

    #[test]
    fn skill_profile_empty_player() {
        let db = setup_test_db();
        let profile = db.get_skill_profile("p1").unwrap();
        assert_eq!(profile.overall_rating, 1200.0);
        assert!(profile.strongest_category.is_none());
    }

    #[test]
    fn skill_profile_with_ratings() {
        let db = setup_test_db();
        db.upsert_skill_rating(&SkillRating {
            id: "r1".to_string(),
            player_id: "p1".to_string(),
            category: "tactical".to_string(),
            rating: 1400.0,
            rd: 100.0,
            volatility: 0.06,
            games_count: 10,
            last_updated: String::new(),
        })
        .unwrap();
        db.upsert_skill_rating(&SkillRating {
            id: "r2".to_string(),
            player_id: "p1".to_string(),
            category: "endgame".to_string(),
            rating: 1000.0,
            rd: 150.0,
            volatility: 0.06,
            games_count: 5,
            last_updated: String::new(),
        })
        .unwrap();

        let profile = db.get_skill_profile("p1").unwrap();
        assert_eq!(profile.ratings.len(), 2);
        assert_eq!(profile.strongest_category, Some("tactical".to_string()));
        assert_eq!(profile.weakest_category, Some("endgame".to_string()));
        // Weighted average: (1400*10 + 1000*5) / 15 = 19000/15 ≈ 1266.67
        assert!((profile.overall_rating - 1266.67).abs() < 1.0);
    }
}
