use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

impl From<shakmaty::Color> for Color {
    fn from(c: shakmaty::Color) -> Self {
        match c {
            shakmaty::Color::White => Color::White,
            shakmaty::Color::Black => Color::Black,
        }
    }
}

impl From<Color> for shakmaty::Color {
    fn from(c: Color) -> Self {
        match c {
            Color::White => shakmaty::Color::White,
            Color::Black => shakmaty::Color::Black,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum GameOutcome {
    Checkmate { winner: Color },
    Stalemate,
    InsufficientMaterial,
    ThreefoldRepetition,
    FiftyMoveRule,
    Resignation { winner: Color },
    Draw,
}

/// Position state sent to the frontend for rendering
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub fen: String,
    pub turn: Color,
    /// Legal destinations: maps "e2" → ["e3", "e4"] for chessground
    pub dests: HashMap<String, Vec<String>>,
    pub last_move: Option<[String; 2]>,
    pub is_check: bool,
    pub is_game_over: bool,
    pub outcome: Option<GameOutcome>,
    pub move_number: u32,
    pub san_history: Vec<String>,
}
