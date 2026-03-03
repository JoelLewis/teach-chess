use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum PuzzleCategory {
    Tactical,
    Positional,
    Endgame,
    Opening,
    Pattern,
}

impl PuzzleCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Tactical => "tactical",
            Self::Positional => "positional",
            Self::Endgame => "endgame",
            Self::Opening => "opening",
            Self::Pattern => "pattern",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "positional" => Self::Positional,
            "endgame" => Self::Endgame,
            "opening" => Self::Opening,
            "pattern" => Self::Pattern,
            _ => Self::Tactical,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Puzzle {
    pub id: String,
    pub fen: String,
    /// Space-separated UCI moves (first move is opponent's setup move)
    pub solution_moves: String,
    /// Comma-separated theme tags
    pub themes: String,
    pub category: PuzzleCategory,
    pub difficulty: u32,
    pub source_id: Option<String>,
    /// JSON array of hint strings (up to 3 tiers)
    pub hints_json: String,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PuzzleAttempt {
    pub id: String,
    pub player_id: String,
    pub puzzle_id: String,
    pub solved: bool,
    pub time_ms: u64,
    pub hints_used: u32,
    pub attempted_at: String,
    pub srs_interval: f64,
    pub srs_ease: f64,
    pub srs_next_review: String,
}

/// Sent to the frontend when a puzzle is loaded
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PuzzleState {
    pub puzzle: Puzzle,
    /// FEN after the opponent's setup move (player's turn)
    pub start_fen: String,
    /// Which color the player is solving as
    pub player_color: String,
    /// Legal destinations from the start position
    pub legal_dests: std::collections::HashMap<String, Vec<String>>,
    /// Total number of player moves expected
    pub total_player_moves: u32,
    /// Current move index (0-based, into the player moves)
    pub current_move_index: u32,
}

/// Result of submitting a move
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PuzzleMoveResult {
    pub correct: bool,
    pub is_complete: bool,
    /// Updated FEN after all applied moves (player + opponent response if any)
    pub fen: Option<String>,
    /// Legal dests for the next player move (if puzzle continues)
    pub legal_dests: Option<std::collections::HashMap<String, Vec<String>>>,
    /// The last move played (for board highlight) [from, to]
    pub last_move: Option<[String; 2]>,
    /// Current move index after this action
    pub current_move_index: u32,
    /// Coaching explanation (only set when puzzle is complete)
    pub explanation: Option<String>,
}

/// Filter criteria for loading puzzles
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PuzzleFilter {
    pub category: Option<PuzzleCategory>,
    pub min_difficulty: Option<u32>,
    pub max_difficulty: Option<u32>,
    pub themes: Option<Vec<String>>,
}

/// Aggregate stats for the puzzle session
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PuzzleSessionStats {
    pub total_attempts: u32,
    pub total_solved: u32,
    pub average_time_ms: u64,
    pub current_streak: u32,
    pub best_streak: u32,
}
