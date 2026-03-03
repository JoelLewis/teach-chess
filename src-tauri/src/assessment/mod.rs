pub mod adaptive;
pub mod glicko2;

use crate::models::assessment::DifficultyTarget;

/// Compute a difficulty target range from a player's rating.
/// Targets puzzles within rating ± 100 for optimal challenge.
pub fn difficulty_target_from_rating(rating: f64) -> DifficultyTarget {
    let min = (rating - 100.0).max(0.0) as u32;
    let max = (rating + 100.0) as u32;
    DifficultyTarget {
        target_rating: rating,
        min_rating: min,
        max_rating: max,
    }
}
