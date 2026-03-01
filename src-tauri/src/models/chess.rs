use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PieceRole {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl From<shakmaty::Role> for PieceRole {
    fn from(r: shakmaty::Role) -> Self {
        match r {
            shakmaty::Role::Pawn => PieceRole::Pawn,
            shakmaty::Role::Knight => PieceRole::Knight,
            shakmaty::Role::Bishop => PieceRole::Bishop,
            shakmaty::Role::Rook => PieceRole::Rook,
            shakmaty::Role::Queen => PieceRole::Queen,
            shakmaty::Role::King => PieceRole::King,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegalMove {
    pub uci: String,
    pub san: String,
    pub from: String,
    pub to: String,
    pub promotion: Option<PieceRole>,
}
