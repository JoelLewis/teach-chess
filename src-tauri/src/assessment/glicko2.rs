//! Simplified Glicko-2 rating system for per-category skill assessment.
//!
//! Reference: http://www.glicko.net/glicko/glicko2.pdf
//! Simplification: volatility update uses a constant instead of iterative Illinois method.

use crate::models::assessment::SkillRating;

/// Glicko-2 scaling constant (converts between Glicko-1 and Glicko-2 scale)
const SCALE: f64 = 173.7178;

/// Convert Glicko-1 rating to Glicko-2 scale
fn to_glicko2(rating: f64) -> f64 {
    (rating - 1200.0) / SCALE
}

/// Convert Glicko-2 scale back to Glicko-1 rating
fn from_glicko2(mu: f64) -> f64 {
    mu * SCALE + 1200.0
}

/// Convert Glicko-1 RD to Glicko-2 scale
fn rd_to_glicko2(rd: f64) -> f64 {
    rd / SCALE
}

/// Convert Glicko-2 RD back to Glicko-1 scale
fn rd_from_glicko2(phi: f64) -> f64 {
    phi * SCALE
}

/// g(φ) function — reduces impact of opponents with high RD
fn g(phi: f64) -> f64 {
    1.0 / (1.0 + 3.0 * phi * phi / (std::f64::consts::PI * std::f64::consts::PI)).sqrt()
}

/// E(μ, μj, φj) — expected score
fn expected(mu: f64, mu_j: f64, phi_j: f64) -> f64 {
    1.0 / (1.0 + (-g(phi_j) * (mu - mu_j)).exp())
}

/// Update a player's rating after a single game (puzzle solve/fail).
///
/// - `player`: current skill rating
/// - `opponent_rating`: the puzzle's difficulty rating (Glicko-1 scale)
/// - `won`: true if the player solved the puzzle
///
/// Returns a new SkillRating with updated rating, RD, volatility, and games_count.
pub fn update_rating(player: &SkillRating, opponent_rating: f64, won: bool) -> SkillRating {
    let mu = to_glicko2(player.rating);
    let phi = rd_to_glicko2(player.rd);
    let sigma = player.volatility;

    // Opponent assumed to have moderate RD (puzzle difficulty is fixed)
    let mu_j = to_glicko2(opponent_rating);
    let phi_j = rd_to_glicko2(60.0); // Low RD — puzzle difficulty is well-calibrated

    let g_j = g(phi_j);
    let e_j = expected(mu, mu_j, phi_j);
    let s = if won { 1.0 } else { 0.0 };

    // Estimated variance (v)
    let v = 1.0 / (g_j * g_j * e_j * (1.0 - e_j));

    // Estimated improvement (delta)
    let delta = v * g_j * (s - e_j);

    // Simplified volatility update (constant — skip iterative algorithm)
    let new_sigma = (sigma * sigma + 0.001).sqrt().min(0.06);

    // Update RD: φ* = √(φ² + σ'²), then φ' = 1/√(1/φ*² + 1/v)
    let phi_star = (phi * phi + new_sigma * new_sigma).sqrt();
    let new_phi = 1.0 / (1.0 / (phi_star * phi_star) + 1.0 / v).sqrt();

    // Update rating: μ' = μ + φ'² × g(φj) × (s - E)
    let new_mu = mu + new_phi * new_phi * g_j * (s - e_j);

    // Convert back to Glicko-1 scale
    let new_rating = from_glicko2(new_mu).clamp(100.0, 3000.0);
    let new_rd = rd_from_glicko2(new_phi).clamp(30.0, 350.0);

    let _ = delta; // used conceptually, actual update uses the formula directly

    SkillRating {
        id: player.id.clone(),
        player_id: player.player_id.clone(),
        category: player.category.clone(),
        rating: new_rating,
        rd: new_rd,
        volatility: new_sigma,
        games_count: player.games_count + 1,
        last_updated: String::new(), // Will be set by DB
    }
}

#[allow(dead_code)]
/// Decay RD over time when a player hasn't played in a category.
/// `periods` is the number of rating periods (roughly weeks) since last game.
pub fn decay_rd(current_rd: f64, volatility: f64, periods: f64) -> f64 {
    let phi = rd_to_glicko2(current_rd);
    let new_phi = (phi * phi + periods * volatility * volatility).sqrt();
    rd_from_glicko2(new_phi).min(350.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::assessment::SkillRating;

    fn default_player(rating: f64) -> SkillRating {
        SkillRating {
            id: "test".to_string(),
            player_id: "p1".to_string(),
            category: "tactical".to_string(),
            rating,
            rd: 200.0,
            volatility: 0.06,
            games_count: 0,
            last_updated: String::new(),
        }
    }

    fn new_player() -> SkillRating {
        SkillRating::default_for("p1", "tactical")
    }

    #[test]
    fn win_vs_higher_rated_increases_rating() {
        let player = default_player(1200.0);
        let updated = update_rating(&player, 1400.0, true);
        assert!(
            updated.rating > 1200.0,
            "Rating should increase: {}",
            updated.rating
        );
        assert!(updated.rd < 200.0, "RD should decrease: {}", updated.rd);
    }

    #[test]
    fn loss_vs_lower_rated_decreases_rating() {
        let player = default_player(1200.0);
        let updated = update_rating(&player, 1000.0, false);
        assert!(
            updated.rating < 1200.0,
            "Rating should decrease: {}",
            updated.rating
        );
    }

    #[test]
    fn win_vs_lower_rated_small_increase() {
        let player = default_player(1200.0);
        let big_win = update_rating(&player, 1400.0, true);
        let small_win = update_rating(&player, 1000.0, true);
        assert!(
            big_win.rating > small_win.rating,
            "Win vs higher rated should give more: {} vs {}",
            big_win.rating,
            small_win.rating
        );
    }

    #[test]
    fn new_player_high_rd() {
        let player = new_player();
        assert_eq!(player.rd, 350.0);
        let updated = update_rating(&player, 1200.0, true);
        assert!(updated.rd < 350.0, "RD should decrease after first game");
    }

    #[test]
    fn rd_convergence_with_games() {
        let mut player = new_player();
        for _ in 0..20 {
            player = update_rating(&player, 1200.0, true);
        }
        assert!(
            player.rd < 150.0,
            "RD should converge after many games: {}",
            player.rd
        );
    }

    #[test]
    fn rating_floor_and_ceiling() {
        // Extreme loss shouldn't go below 100
        let weak = default_player(200.0);
        let updated = update_rating(&weak, 2000.0, false);
        assert!(updated.rating >= 100.0);

        // Extreme win shouldn't go above 3000
        let strong = default_player(2900.0);
        let updated = update_rating(&strong, 100.0, true);
        assert!(updated.rating <= 3000.0);
    }

    #[test]
    fn games_count_increments() {
        let player = default_player(1200.0);
        let updated = update_rating(&player, 1200.0, true);
        assert_eq!(updated.games_count, 1);
        let updated2 = update_rating(&updated, 1200.0, false);
        assert_eq!(updated2.games_count, 2);
    }

    #[test]
    fn rd_decays_over_time() {
        let decayed = decay_rd(100.0, 0.06, 10.0);
        assert!(decayed > 100.0, "RD should increase over time: {}", decayed);
        assert!(decayed <= 350.0, "RD should cap at 350: {}", decayed);
    }

    #[test]
    fn expected_score_equal_players() {
        let mu1 = to_glicko2(1200.0);
        let phi_j = rd_to_glicko2(60.0);
        let e = expected(mu1, mu1, phi_j);
        assert!(
            (e - 0.5).abs() < 0.01,
            "Equal players should have ~50% expected: {}",
            e
        );
    }

    #[test]
    fn expected_score_stronger_player() {
        let mu_strong = to_glicko2(1400.0);
        let mu_weak = to_glicko2(1000.0);
        let phi_j = rd_to_glicko2(60.0);
        let e = expected(mu_strong, mu_weak, phi_j);
        assert!(e > 0.5, "Stronger player should expect > 50%: {}", e);
    }
}
