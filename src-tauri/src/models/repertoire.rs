use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Opening {
    pub id: String,
    pub name: String,
    pub eco: String,
    /// Which color this opening is for
    pub color: String,
    pub description: String,
    /// Space-separated UCI moves: "e2e4 e7e5 g1f3"
    pub moves: String,
    /// Comma-separated theme tags
    pub themes: String,
    pub difficulty: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpeningPosition {
    pub id: String,
    pub opening_id: String,
    pub fen: String,
    pub move_index: u32,
    pub parent_fen: Option<String>,
    pub move_uci: String,
    pub move_san: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepertoireEntry {
    pub id: String,
    pub player_id: String,
    pub opening_id: String,
    pub position_fen: String,
    pub move_uci: String,
    pub move_san: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrillState {
    pub opening: Opening,
    pub current_entry: RepertoireEntry,
    pub fen: String,
    pub opponent_move: Option<String>,
    pub player_color: String,
    pub legal_dests: HashMap<String, Vec<String>>,
    pub entries_total: u32,
    pub entries_remaining: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrillMoveResult {
    pub correct: bool,
    pub correct_move: Option<String>,
    pub is_complete: bool,
    pub entries_remaining: u32,
    pub explanation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RepertoireFilter {
    pub color: Option<String>,
    pub eco_prefix: Option<String>,
    pub min_difficulty: Option<u32>,
    pub max_difficulty: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrillAttempt {
    pub id: String,
    pub player_id: String,
    pub repertoire_entry_id: String,
    pub correct: bool,
    pub time_ms: u64,
    pub srs_interval: f64,
    pub srs_ease: f64,
    pub srs_next_review: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DrillStats {
    pub total_drills: u32,
    pub total_correct: u32,
    pub current_streak: u32,
    pub openings_drilled: u32,
}
