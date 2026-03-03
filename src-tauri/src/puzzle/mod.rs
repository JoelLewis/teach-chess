pub mod importer;
pub mod session;
pub mod srs;

use std::collections::HashMap;

use crate::models::puzzle::Puzzle;

/// Active puzzle session state, stored as Tauri managed state.
#[derive(Default)]
pub struct PuzzleSessionState {
    /// The puzzle being solved
    pub puzzle: Option<ActivePuzzle>,
}

pub struct ActivePuzzle {
    pub puzzle: Puzzle,
    /// The parsed solution moves (UCI strings)
    pub solution_moves: Vec<String>,
    /// Index into solution_moves for the next expected player move
    /// (odd indices = opponent's auto-play, even indices after setup = player moves)
    pub current_move_index: usize,
    /// Number of hints revealed so far (0–3)
    pub hints_revealed: u32,
    /// Timestamp when the puzzle was started (ms since epoch)
    pub start_time_ms: u64,
    /// Current FEN (updated as moves are applied)
    pub current_fen: String,
    /// Legal destinations from current position
    pub legal_dests: HashMap<String, Vec<String>>,
    /// Player's color for this puzzle
    pub player_color: String,
    /// The current shakmaty Chess position (for move validation)
    pub chess: shakmaty::Chess,
}
