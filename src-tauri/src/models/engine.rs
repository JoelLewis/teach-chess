use serde::{Deserialize, Serialize};

/// Engine evaluation score
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Score {
    /// Centipawn advantage (positive = white advantage)
    Cp { value: i32 },
    /// Mate in N moves (positive = white mates, negative = black mates)
    Mate { moves: i32 },
}

impl Score {
    pub fn cp(value: i32) -> Self {
        Score::Cp { value }
    }

    pub fn mate(moves: i32) -> Self {
        Score::Mate { moves }
    }

    /// Normalize to white's perspective (always positive = good for white)
    pub fn from_white_perspective(&self) -> Self {
        self.clone()
    }

    /// Normalize to a specific side's perspective
    pub fn from_perspective(&self, is_white: bool) -> Self {
        if is_white {
            self.clone()
        } else {
            match self {
                Score::Cp { value } => Score::Cp { value: -value },
                Score::Mate { moves } => Score::Mate { moves: -moves },
            }
        }
    }
}

/// Full engine evaluation for a position
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineEvaluation {
    pub score: Score,
    pub depth: u32,
    /// Principal variation (best line) as UCI moves
    pub pv: Vec<String>,
    pub nodes: u64,
    /// Best move in UCI notation
    pub best_move: String,
}

/// Result of requesting a move from the engine
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineMove {
    pub uci: String,
    pub ponder: Option<String>,
}

/// Classification of a move's quality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MoveClassification {
    Best,
    Excellent,
    Good,
    Inaccuracy,
    Mistake,
    Blunder,
}

impl MoveClassification {
    /// Classify a move by centipawn loss
    pub fn from_cp_loss(cp_loss: i32) -> Self {
        match cp_loss.unsigned_abs() {
            0..=10 => MoveClassification::Best,
            11..=25 => MoveClassification::Excellent,
            26..=50 => MoveClassification::Good,
            51..=100 => MoveClassification::Inaccuracy,
            101..=200 => MoveClassification::Mistake,
            _ => MoveClassification::Blunder,
        }
    }
}

/// Per-move evaluation for game review
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveEvaluation {
    pub move_number: u32,
    pub is_white: bool,
    pub fen_before: String,
    pub player_move_uci: String,
    pub player_move_san: String,
    pub engine_best_uci: Option<String>,
    pub engine_best_san: Option<String>,
    pub eval_before: Option<Score>,
    pub eval_after: Option<Score>,
    pub classification: Option<MoveClassification>,
    pub depth: u32,
    pub pv: Vec<String>,
}
