//! FSRS spaced-repetition scheduling (rs-fsrs), shared by puzzles and
//! opening drills. Card persistence lives in `crate::db::srs`.

use chrono::Utc;
use rs_fsrs::{Card, FSRS, Rating};

/// Map a puzzle outcome to an FSRS rating.
pub fn solve_to_rating(solved: bool, hints_used: u32) -> Rating {
    if !solved {
        Rating::Again
    } else if hints_used >= 2 {
        Rating::Hard
    } else if hints_used == 1 {
        Rating::Good
    } else {
        Rating::Easy
    }
}

/// Map an opening drill outcome to an FSRS rating.
///
/// Drills have no hints, so speed stands in for confidence:
/// correct in under 10s rates Easy, slower correct answers rate Good.
pub fn drill_to_rating(correct: bool, time_ms: u64) -> Rating {
    if !correct {
        Rating::Again
    } else if time_ms < 10_000 {
        Rating::Easy
    } else {
        Rating::Good
    }
}

/// Apply a rating to a card as of now, returning the rescheduled card.
pub fn next_card(card: Card, rating: Rating) -> Card {
    FSRS::new(Default::default())
        .next(card, Utc::now(), rating)
        .card
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solve_to_rating_mapping() {
        assert_eq!(solve_to_rating(false, 0), Rating::Again);
        assert_eq!(solve_to_rating(true, 2), Rating::Hard);
        assert_eq!(solve_to_rating(true, 3), Rating::Hard);
        assert_eq!(solve_to_rating(true, 1), Rating::Good);
        assert_eq!(solve_to_rating(true, 0), Rating::Easy);
    }

    #[test]
    fn drill_to_rating_mapping() {
        assert_eq!(drill_to_rating(false, 5_000), Rating::Again);
        assert_eq!(drill_to_rating(true, 5_000), Rating::Easy);
        assert_eq!(drill_to_rating(true, 15_000), Rating::Good);
    }

    #[test]
    fn next_card_advances_review_state() {
        let card = next_card(Card::new(), Rating::Easy);
        assert!(card.reps > 0);
        assert!(card.due > Utc::now());

        let failed = next_card(Card::new(), Rating::Again);
        assert!(failed.scheduled_days <= card.scheduled_days);
    }
}
