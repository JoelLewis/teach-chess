use tracing::info;

use crate::db::connection::Database;
use crate::error::{AppError, RepertoireError};
use crate::models::repertoire::Opening;

/// Import openings from a JSON file.
pub fn import_openings_json(
    json_str: &str,
    db: &Database,
) -> Result<usize, AppError> {
    let raw: Vec<RawOpening> = serde_json::from_str(json_str)
        .map_err(|e| RepertoireError::ImportError(format!("JSON parse error: {e}")))?;

    let openings: Vec<Opening> = raw
        .into_iter()
        .map(|r| Opening {
            id: generate_opening_id(&r.name),
            name: r.name,
            eco: r.eco,
            color: r.color,
            description: r.description,
            moves: r.moves,
            themes: r.themes,
            difficulty: r.difficulty,
        })
        .collect();

    let count = db.import_openings(&openings)?;
    info!("Imported {count} openings from JSON");
    Ok(count)
}

/// Import the bundled starter openings if the table is empty.
pub fn import_starter_openings_if_empty(db: &Database) -> Result<(), AppError> {
    let count = db.get_opening_count()?;
    if count > 0 {
        info!("Opening table already has {count} openings, skipping starter import");
        return Ok(());
    }

    info!("Opening table empty, importing bundled starter openings...");
    let json_data = include_str!("../../data/openings_starter.json");
    let imported = import_openings_json(json_data, db)?;
    info!("Imported {imported} starter openings");
    Ok(())
}

fn generate_opening_id(name: &str) -> String {
    name.to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != '-', "_")
        .trim_matches('_')
        .to_string()
}

#[derive(serde::Deserialize)]
struct RawOpening {
    name: String,
    eco: String,
    color: String,
    #[serde(default)]
    description: String,
    moves: String,
    #[serde(default)]
    themes: String,
    #[serde(default = "default_difficulty")]
    difficulty: u32,
}

fn default_difficulty() -> u32 {
    1200
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_id_from_name() {
        assert_eq!(generate_opening_id("Italian Game"), "italian_game");
        assert_eq!(generate_opening_id("Queen's Gambit"), "queen_s_gambit");
        assert_eq!(
            generate_opening_id("Sicilian Defense: Najdorf"),
            "sicilian_defense__najdorf"
        );
    }

    #[test]
    fn parse_starter_json() {
        let json = include_str!("../../data/openings_starter.json");
        let raw: Vec<RawOpening> = serde_json::from_str(json).unwrap();
        assert!(!raw.is_empty(), "Starter JSON should have openings");
        for opening in &raw {
            assert!(!opening.name.is_empty());
            assert!(!opening.moves.is_empty());
            assert!(
                opening.color == "white" || opening.color == "black",
                "Invalid color: {}",
                opening.color
            );
        }
    }
}
