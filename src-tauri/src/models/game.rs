use serde::{Deserialize, Serialize};

use super::chess::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineStrength {
    /// UCI_Elo value (1320–3190 for Stockfish)
    pub elo: u32,
    /// Stockfish Skill Level (0–20)
    pub skill_level: u8,
}

impl EngineStrength {
    pub fn beginner() -> Self {
        Self {
            elo: 1350,
            skill_level: 1,
        }
    }

    pub fn intermediate() -> Self {
        Self {
            elo: 1800,
            skill_level: 8,
        }
    }

    pub fn advanced() -> Self {
        Self {
            elo: 2200,
            skill_level: 14,
        }
    }

    pub fn maximum() -> Self {
        Self {
            elo: 3190,
            skill_level: 20,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeControl {
    /// Initial time in seconds (0 = unlimited)
    pub initial_secs: u32,
    /// Increment per move in seconds
    pub increment_secs: u32,
}

impl Default for TimeControl {
    fn default() -> Self {
        Self {
            initial_secs: 0,
            increment_secs: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameConfig {
    pub player_color: Color,
    pub engine_strength: EngineStrength,
    pub time_control: TimeControl,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameRecord {
    pub id: String,
    pub player_id: String,
    pub pgn: String,
    pub result: String,
    pub player_color: Color,
    pub engine_elo: u32,
    pub move_count: u32,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub time_control: String,
}
