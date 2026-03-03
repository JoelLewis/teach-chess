use super::PlayerLevel;

impl PlayerLevel {
    /// Derive player level from game statistics.
    ///
    /// - Beginner: fewer than 10 games OR blunder rate above 15%
    /// - Intermediate: fewer than 50 games OR blunder rate above 8%
    /// - UpperIntermediate: everything else
    pub fn from_game_stats(games_played: u32, blunder_rate: f64, mistake_rate: f64) -> Self {
        let error_rate = blunder_rate + mistake_rate;

        if games_played < 10 || blunder_rate > 0.15 {
            return PlayerLevel::Beginner;
        }

        if games_played < 50 || blunder_rate > 0.08 || error_rate > 0.25 {
            return PlayerLevel::Intermediate;
        }

        PlayerLevel::UpperIntermediate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_player_is_beginner() {
        assert_eq!(
            PlayerLevel::from_game_stats(0, 0.0, 0.0),
            PlayerLevel::Beginner
        );
    }

    #[test]
    fn few_games_is_beginner() {
        assert_eq!(
            PlayerLevel::from_game_stats(5, 0.05, 0.05),
            PlayerLevel::Beginner
        );
    }

    #[test]
    fn high_blunder_rate_is_beginner() {
        assert_eq!(
            PlayerLevel::from_game_stats(100, 0.20, 0.05),
            PlayerLevel::Beginner
        );
    }

    #[test]
    fn boundary_10_games_low_errors_is_intermediate() {
        assert_eq!(
            PlayerLevel::from_game_stats(10, 0.05, 0.05),
            PlayerLevel::Intermediate
        );
    }

    #[test]
    fn moderate_blunder_rate_is_intermediate() {
        assert_eq!(
            PlayerLevel::from_game_stats(60, 0.10, 0.05),
            PlayerLevel::Intermediate
        );
    }

    #[test]
    fn high_combined_error_rate_is_intermediate() {
        assert_eq!(
            PlayerLevel::from_game_stats(60, 0.07, 0.20),
            PlayerLevel::Intermediate
        );
    }

    #[test]
    fn experienced_low_errors_is_upper_intermediate() {
        assert_eq!(
            PlayerLevel::from_game_stats(100, 0.05, 0.10),
            PlayerLevel::UpperIntermediate
        );
    }

    #[test]
    fn boundary_50_games_good_stats() {
        assert_eq!(
            PlayerLevel::from_game_stats(50, 0.05, 0.10),
            PlayerLevel::UpperIntermediate
        );
    }

    #[test]
    fn exactly_15_percent_blunders_is_not_beginner() {
        // 0.15 is NOT > 0.15, so not beginner
        assert_eq!(
            PlayerLevel::from_game_stats(10, 0.15, 0.05),
            PlayerLevel::Intermediate
        );
    }
}
