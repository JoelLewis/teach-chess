use crate::models::engine::{MoveClassification, Score};

/// Calculate centipawn loss between two evaluations
pub fn cp_loss(before: &Score, after: &Score, is_white: bool) -> i32 {
    let before_cp = score_to_cp(before, is_white);
    let after_cp = score_to_cp(after, is_white);

    // Positive cp_loss means the move was worse than the position warranted
    (before_cp - after_cp).max(0)
}

/// Classify a move based on score before and after
pub fn classify_move(
    before: &Score,
    after: &Score,
    is_white: bool,
) -> MoveClassification {
    let loss = cp_loss(before, after, is_white);
    MoveClassification::from_cp_loss(loss)
}

/// Convert a Score to centipawns from a specific side's perspective.
/// Mate scores are converted to large centipawn values.
fn score_to_cp(score: &Score, is_white: bool) -> i32 {
    let raw = match score {
        Score::Cp { value } => *value,
        Score::Mate { moves } => {
            if *moves > 0 {
                10000 - (*moves * 10) // Mate for white
            } else {
                -10000 - (*moves * 10) // Mate for black
            }
        }
    };

    if is_white {
        raw
    } else {
        -raw
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_loss_same_score() {
        let before = Score::cp(50);
        let after = Score::cp(50);
        assert_eq!(cp_loss(&before, &after, true), 0);
    }

    #[test]
    fn cp_loss_for_white_blunder() {
        let before = Score::cp(100);
        let after = Score::cp(-200);
        assert_eq!(cp_loss(&before, &after, true), 300);
    }

    #[test]
    fn classify_blunder() {
        let before = Score::cp(50);
        let after = Score::cp(-250);
        assert_eq!(
            classify_move(&before, &after, true),
            MoveClassification::Blunder
        );
    }

    #[test]
    fn classify_best() {
        let before = Score::cp(50);
        let after = Score::cp(55);
        assert_eq!(
            classify_move(&before, &after, true),
            MoveClassification::Best
        );
    }
}
