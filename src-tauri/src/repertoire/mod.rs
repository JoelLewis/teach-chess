pub mod importer;
pub mod session;

use std::collections::HashMap;

use crate::models::repertoire::{Opening, RepertoireEntry};

/// Active repertoire drill session state, stored as Tauri managed state.
pub struct RepertoireSessionState {
    pub drill: Option<ActiveDrill>,
}

pub struct ActiveDrill {
    pub opening: Opening,
    /// Entries to drill (in order)
    pub entries: Vec<RepertoireEntry>,
    /// Index of the current entry being drilled
    pub current_index: usize,
    /// Timestamp when the current entry drill started (ms since epoch)
    pub entry_start_time_ms: u64,
    /// Current FEN (the position where the player must respond)
    pub current_fen: String,
    /// Legal destinations from current position
    pub legal_dests: HashMap<String, Vec<String>>,
    /// Player's color for this opening
    pub player_color: String,
    /// The current shakmaty Chess position
    pub chess: shakmaty::Chess,
}

impl Default for RepertoireSessionState {
    fn default() -> Self {
        Self { drill: None }
    }
}
