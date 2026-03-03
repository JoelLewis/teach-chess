pub mod moves;
pub mod state;

use shakmaty::{fen::Fen, CastlingMode, Chess};

use crate::error::AppError;

/// Parse a FEN string into a shakmaty Chess position.
pub fn parse_fen(fen: &str) -> Result<Chess, AppError> {
    let parsed: Fen = fen
        .parse()
        .map_err(|e| crate::error::GameError::InvalidFen(format!("{e}")))?;
    parsed
        .into_position(CastlingMode::Standard)
        .map_err(|e| crate::error::GameError::InvalidFen(format!("{e}")).into())
}
