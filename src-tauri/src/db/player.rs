use crate::error::DatabaseError;
use crate::models::player::{Player, PlayerSettings};

use super::connection::Database;

impl Database {
    pub fn get_or_create_player(&self, display_name: &str) -> Result<Player, DatabaseError> {
        // Try to find existing player by name
        let existing = self.conn().query_row(
            "SELECT id, display_name, created_at, games_played, settings_json FROM player WHERE display_name = ?1",
            [display_name],
            |row| {
                Ok(Player {
                    id: row.get(0)?,
                    display_name: row.get(1)?,
                    created_at: row.get(2)?,
                    games_played: row.get(3)?,
                    settings: serde_json::from_str(&row.get::<_, String>(4)?)
                        .unwrap_or_default(),
                })
            },
        );

        match existing {
            Ok(player) => Ok(player),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                let id = uuid::Uuid::new_v4().to_string();
                let settings_json = serde_json::to_string(&PlayerSettings::default()).unwrap();

                self.conn().execute(
                    "INSERT INTO player (id, display_name, settings_json) VALUES (?1, ?2, ?3)",
                    rusqlite::params![id, display_name, settings_json],
                )?;

                Ok(Player {
                    id,
                    display_name: display_name.to_string(),
                    created_at: String::new(),
                    games_played: 0,
                    settings: PlayerSettings::default(),
                })
            }
            Err(e) => Err(DatabaseError::Sqlite(e)),
        }
    }

    pub fn update_player_settings(
        &self,
        player_id: &str,
        settings: &PlayerSettings,
    ) -> Result<Player, DatabaseError> {
        let settings_json = serde_json::to_string(settings)
            .map_err(|e| DatabaseError::MigrationFailed(e.to_string()))?;

        let updated = self.conn().execute(
            "UPDATE player SET settings_json = ?1 WHERE id = ?2",
            rusqlite::params![settings_json, player_id],
        )?;

        if updated == 0 {
            return Err(DatabaseError::PlayerNotFound(player_id.to_string()));
        }

        self.conn().query_row(
            "SELECT id, display_name, created_at, games_played, settings_json FROM player WHERE id = ?1",
            [player_id],
            |row| {
                Ok(Player {
                    id: row.get(0)?,
                    display_name: row.get(1)?,
                    created_at: row.get(2)?,
                    games_played: row.get(3)?,
                    settings: serde_json::from_str(&row.get::<_, String>(4)?)
                        .unwrap_or_default(),
                })
            },
        ).map_err(DatabaseError::Sqlite)
    }
}
