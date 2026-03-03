use crate::models::heuristics::{CoachingContext, TacticType};

/// Categories of player weakness that teaching mode can target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaknessCategory {
    Tactical,
    Positional,
    Endgame,
    Opening,
    KingSafety,
}

/// Score how well a position after a candidate move creates a
/// teaching opportunity for the player's weak areas.
///
/// Returns 0.0–1.0: higher means the position is more likely to
/// test the player's specific weakness.
pub fn score_teaching_opportunity(
    context_after: &CoachingContext,
    weak_categories: &[WeaknessCategory],
) -> f64 {
    if weak_categories.is_empty() {
        return 0.0;
    }

    let mut total_score = 0.0;
    let mut count = 0;

    for weakness in weak_categories {
        let score = match weakness {
            WeaknessCategory::Tactical => {
                // Position has forks, pins, or hanging pieces for the player to find
                let tactic_count = context_after
                    .tactics
                    .iter()
                    .filter(|t| {
                        matches!(
                            t.tactic_type,
                            TacticType::Fork
                                | TacticType::Pin
                                | TacticType::HangingPiece
                                | TacticType::DiscoveredAttack
                        )
                    })
                    .count() as f64;
                (tactic_count / 2.0).min(1.0)
            }
            WeaknessCategory::Positional => {
                // Position has structural imbalances requiring strategic play
                let theme_count = context_after.themes.len() as f64;
                (theme_count / 4.0).min(1.0)
            }
            WeaknessCategory::Endgame => {
                // Prefer simplification — fewer pieces on the board
                let total_pieces = context_after.material.white.knights
                    + context_after.material.white.bishops
                    + context_after.material.white.rooks
                    + context_after.material.black.knights
                    + context_after.material.black.bishops
                    + context_after.material.black.rooks;
                // Fewer pieces = higher endgame teaching score
                if total_pieces <= 4 {
                    1.0
                } else if total_pieces <= 6 {
                    0.7
                } else if total_pieces <= 8 {
                    0.3
                } else {
                    0.0
                }
            }
            WeaknessCategory::Opening => {
                // Position has undeveloped pieces or development imbalance
                let undeveloped = context_after.activity.white.total_minors
                    - context_after.activity.white.developed_minors
                    + context_after.activity.black.total_minors
                    - context_after.activity.black.developed_minors;
                (undeveloped as f64 / 4.0).min(1.0)
            }
            WeaknessCategory::KingSafety => {
                // Position has king safety issues for the player to navigate
                let white_unsafe =
                    context_after.king_safety.white.pawn_shield_count
                        < context_after.king_safety.white.pawn_shield_max.saturating_sub(1);
                let black_unsafe =
                    context_after.king_safety.black.pawn_shield_count
                        < context_after.king_safety.black.pawn_shield_max.saturating_sub(1);
                if white_unsafe || black_unsafe {
                    0.8
                } else {
                    0.0
                }
            }
        };

        total_score += score;
        count += 1;
    }

    if count > 0 {
        total_score / count as f64
    } else {
        0.0
    }
}

/// Map a skill category string (from the assessment system) to a weakness category.
pub fn category_to_weakness(category: &str) -> Option<WeaknessCategory> {
    match category {
        "tactical" => Some(WeaknessCategory::Tactical),
        "positional" => Some(WeaknessCategory::Positional),
        "endgame" => Some(WeaknessCategory::Endgame),
        "opening" => Some(WeaknessCategory::Opening),
        "pattern" => Some(WeaknessCategory::KingSafety), // closest match
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::heuristics::*;

    fn default_context() -> CoachingContext {
        CoachingContext {
            fen: "startpos".to_string(),
            phase: GamePhase::Middlegame,
            material: MaterialBalance {
                white: PieceCounts {
                    pawns: 8, knights: 2, bishops: 2, rooks: 2, queens: 1,
                },
                black: PieceCounts {
                    pawns: 8, knights: 2, bishops: 2, rooks: 2, queens: 1,
                },
                balance_cp: 0,
                imbalances: vec![],
            },
            pawns: PawnStructure {
                white: SidePawnStructure {
                    isolated: vec![], doubled: vec![], passed: vec![], backward: vec![],
                },
                black: SidePawnStructure {
                    isolated: vec![], doubled: vec![], passed: vec![], backward: vec![],
                },
                chains: vec![],
                open_files: vec![],
                half_open_files_white: vec![],
                half_open_files_black: vec![],
            },
            activity: PieceActivity {
                white: SideActivity {
                    total_mobility: 20, developed_minors: 2, total_minors: 4,
                    rook_on_open_file: false, rook_on_seventh: false, pieces: vec![],
                },
                black: SideActivity {
                    total_mobility: 20, developed_minors: 2, total_minors: 4,
                    rook_on_open_file: false, rook_on_seventh: false, pieces: vec![],
                },
            },
            king_safety: KingSafety {
                white: SideKingSafety {
                    king_square: "g1".to_string(), pawn_shield_count: 3, pawn_shield_max: 3,
                    has_castled: true, can_castle: false, open_files_near_king: 0, king_zone_attacks: 0,
                },
                black: SideKingSafety {
                    king_square: "g8".to_string(), pawn_shield_count: 3, pawn_shield_max: 3,
                    has_castled: true, can_castle: false, open_files_near_king: 0, king_zone_attacks: 0,
                },
            },
            tactics: vec![],
            themes: vec![],
        }
    }

    #[test]
    fn no_weaknesses_returns_zero() {
        let ctx = default_context();
        assert_eq!(score_teaching_opportunity(&ctx, &[]), 0.0);
    }

    #[test]
    fn tactical_weakness_scores_tactics() {
        let mut ctx = default_context();
        ctx.tactics.push(TacticalMotif {
            tactic_type: TacticType::Fork,
            side: Side::White,
            square: "e4".to_string(),
            description: "Knight fork".to_string(),
        });
        ctx.tactics.push(TacticalMotif {
            tactic_type: TacticType::Pin,
            side: Side::Black,
            square: "c4".to_string(),
            description: "Bishop pin".to_string(),
        });

        let score = score_teaching_opportunity(&ctx, &[WeaknessCategory::Tactical]);
        assert!(score > 0.5, "Expected high tactical teaching score, got {score}");
    }

    #[test]
    fn endgame_weakness_prefers_few_pieces() {
        let mut ctx = default_context();
        // Simulate endgame: few pieces
        ctx.material.white.knights = 0;
        ctx.material.white.bishops = 1;
        ctx.material.white.rooks = 1;
        ctx.material.white.queens = 0;
        ctx.material.black.knights = 0;
        ctx.material.black.bishops = 0;
        ctx.material.black.rooks = 1;
        ctx.material.black.queens = 0;

        let score = score_teaching_opportunity(&ctx, &[WeaknessCategory::Endgame]);
        assert!(score >= 0.7, "Expected high endgame score with 3 pieces, got {score}");
    }

    #[test]
    fn king_safety_weakness_detects_weak_shield() {
        let mut ctx = default_context();
        ctx.king_safety.white.pawn_shield_count = 1; // weak shield

        let score = score_teaching_opportunity(&ctx, &[WeaknessCategory::KingSafety]);
        assert!(score > 0.5, "Expected high king safety score, got {score}");
    }

    #[test]
    fn multiple_weaknesses_averaged() {
        let ctx = default_context();
        // Neither tactical nor endgame features present
        let score = score_teaching_opportunity(
            &ctx,
            &[WeaknessCategory::Tactical, WeaknessCategory::Endgame],
        );
        // Both should be 0.0, average = 0.0
        assert!(score < 0.1, "Expected low score, got {score}");
    }

    #[test]
    fn category_mapping() {
        assert_eq!(category_to_weakness("tactical"), Some(WeaknessCategory::Tactical));
        assert_eq!(category_to_weakness("positional"), Some(WeaknessCategory::Positional));
        assert_eq!(category_to_weakness("endgame"), Some(WeaknessCategory::Endgame));
        assert_eq!(category_to_weakness("opening"), Some(WeaknessCategory::Opening));
        assert_eq!(category_to_weakness("pattern"), Some(WeaknessCategory::KingSafety));
        assert_eq!(category_to_weakness("unknown"), None);
    }
}
