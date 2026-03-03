use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::heuristics::{CoachingContext, GamePhase, PositionalTheme, TacticType};

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
    #[allow(dead_code, clippy::wrong_self_convention)]
    pub fn from_white_perspective(&self) -> Self {
        self.clone()
    }

    /// Normalize to a specific side's perspective
    #[allow(dead_code, clippy::wrong_self_convention)]
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

    /// Whether this classification represents a player error
    pub fn is_error(&self) -> bool {
        matches!(
            self,
            MoveClassification::Inaccuracy
                | MoveClassification::Mistake
                | MoveClassification::Blunder
        )
    }

    /// Whether this classification represents a strong move
    pub fn is_positive(&self) -> bool {
        matches!(
            self,
            MoveClassification::Best | MoveClassification::Excellent
        )
    }

    /// Parse from a lowercase string (e.g. "best", "blunder")
    pub fn from_str_loose(s: &str) -> Self {
        match s {
            "best" => MoveClassification::Best,
            "excellent" => MoveClassification::Excellent,
            "good" => MoveClassification::Good,
            "inaccuracy" => MoveClassification::Inaccuracy,
            "mistake" => MoveClassification::Mistake,
            "blunder" => MoveClassification::Blunder,
            _ => MoveClassification::Good,
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
    pub coaching_context: Option<CoachingContext>,
    pub coaching_text: Option<String>,
}

// ─── In-Game Coaching Types ──────────────────────────────────────

/// How much coaching feedback to show during play
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CoachingLevel {
    /// All feedback: pre-move hints + post-move for every classification
    #[default]
    FullCoach,
    /// Post-move for inaccuracy+ and best; no pre-move hints
    LightTouch,
    /// Only blunders
    Minimal,
    /// No in-game coaching
    Silent,
}

/// Post-move coaching feedback during gameplay
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InGameCoachingFeedback {
    pub classification: MoveClassification,
    pub coaching_text: String,
    pub eval_before: Score,
    pub eval_after: Score,
    pub engine_best_uci: String,
    pub coaching_context: Option<CoachingContext>,
    pub move_number: u32,
}

/// Pre-move hint shown before the player's next move
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreMoveHint {
    pub hint_text: Option<String>,
    pub hint_type: PreMoveHintType,
    pub themes: Vec<PositionalTheme>,
}

impl Default for PreMoveHint {
    fn default() -> Self {
        Self {
            hint_text: None,
            hint_type: PreMoveHintType::None,
            themes: vec![],
        }
    }
}

/// Category of pre-move hint
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PreMoveHintType {
    /// Tactic available or hanging piece detected
    TacticalAlert,
    /// Entering middlegame/endgame
    PhaseTransition,
    /// Positional theme to consider
    StrategicReminder,
    /// No hint to show
    None,
}

// ─── Post-Game Review Enhancement Types ──────────────────────────

/// A pivotal moment in the game where the evaluation swung significantly
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CriticalMoment {
    pub move_index: usize,
    pub eval_swing_cp: i32,
    pub description: String,
    pub is_player_move: bool,
}

/// Summary of recurring patterns across a game's errors
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatternSummary {
    pub total_errors: u32,
    pub error_themes: Vec<(PositionalTheme, u32)>,
    pub missed_tactics: Vec<(TacticType, u32)>,
    pub errors_by_phase: HashMap<GamePhase, u32>,
    pub strengths: Vec<String>,
}

/// A recommended study topic based on game weaknesses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudySuggestion {
    pub topic: String,
    pub description: String,
    pub priority: u8,
}
