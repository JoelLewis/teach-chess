//! Rank-calibrated coaching context.
//!
//! Maps a detected theme/tactic to the Glicko-2 skill category it trains,
//! buckets the player's category rating into a coarse band, and produces
//! qualitative, level-relative phrasing for templates and LLM prompts.
//! No fabricated statistics — phrasing is qualitative only.

use crate::models::assessment::SkillRating;
use crate::models::heuristics::{CoachingContext, GamePhase, PositionalTheme, TacticType};

/// Minimum rated games before a category rating is trusted for coaching.
///
/// Matches the threshold `SkillProfile` uses for strongest/weakest categories.
/// Below this the rating is still near its 1200 prior with a huge RD, so we
/// degrade gracefully to un-ranked feedback.
pub const MIN_RATED_GAMES: u32 = 3;

/// Coarse rating band for calibrating feedback tone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RatingBand {
    /// Below 1200.
    Novice,
    /// 1200 to 1500.
    Developing,
    /// 1500 to 1800.
    Advancing,
    /// Above 1800.
    Expert,
}

impl RatingBand {
    pub fn from_rating(rating: f64) -> Self {
        if rating < 1200.0 {
            Self::Novice
        } else if rating < 1500.0 {
            Self::Developing
        } else if rating <= 1800.0 {
            Self::Advancing
        } else {
            Self::Expert
        }
    }
}

/// Skill category trained by a tactical motif. All tactics are tactical.
pub fn category_for_tactic(_tactic: &TacticType) -> &'static str {
    "tactical"
}

/// Skill category trained by a positional theme.
pub fn category_for_theme(theme: &PositionalTheme) -> &'static str {
    match theme {
        PositionalTheme::ForkAvailable
        | PositionalTheme::PinnedPiece
        | PositionalTheme::HangingMaterial
        | PositionalTheme::BackRankWeakness => "tactical",
        PositionalTheme::UndevelopedPieces => "opening",
        PositionalTheme::PassedPawn => "endgame",
        PositionalTheme::KnightOnRim
        | PositionalTheme::BishopPairAdvantage
        | PositionalTheme::IsolatedQueenPawn
        | PositionalTheme::DoubledPawns
        | PositionalTheme::BackwardPawn
        | PositionalTheme::OpenFile
        | PositionalTheme::RookOnSeventh
        | PositionalTheme::KingSafetyCompromised
        | PositionalTheme::CentralControl
        | PositionalTheme::PawnChainTension
        | PositionalTheme::MaterialImbalance => "positional",
    }
}

/// Skill category for a full coaching context: tactics dominate, then the
/// first detected theme, then the game phase.
pub fn category_for_context(ctx: &CoachingContext) -> &'static str {
    if let Some(tactic) = ctx.tactics.first() {
        return category_for_tactic(&tactic.tactic_type);
    }
    if let Some(theme) = ctx.themes.first() {
        return category_for_theme(theme);
    }
    match ctx.phase {
        GamePhase::Opening => "opening",
        GamePhase::Endgame => "endgame",
        GamePhase::Middlegame => "tactical",
    }
}

/// The player's rating context for the skill category relevant to one move.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerRankContext {
    pub category: String,
    pub rating: f64,
    pub band: RatingBand,
}

impl PlayerRankContext {
    /// Build from a stored skill rating, or `None` when the rating is still
    /// too fresh to be meaningful (fewer than [`MIN_RATED_GAMES`] games).
    pub fn from_skill_rating(skill: &SkillRating) -> Option<Self> {
        if skill.games_count < MIN_RATED_GAMES {
            return None;
        }
        Some(Self {
            category: skill.category.clone(),
            rating: skill.rating,
            band: RatingBand::from_rating(skill.rating),
        })
    }

    /// Rating rounded to the nearest 50 for display — precise decimals would
    /// suggest more accuracy than a Glicko estimate has.
    pub fn rounded_rating(&self) -> u32 {
        ((self.rating / 50.0).round() * 50.0) as u32
    }

    /// One-line "Player context" section for the LLM user prompt.
    ///
    /// Qualitative only — never invents percentages or statistics.
    pub fn llm_prompt_line(&self, is_positive: bool) -> String {
        let note = if is_positive {
            match self.band {
                RatingBand::Novice => {
                    "finds like this are well ahead of most players at this level"
                }
                RatingBand::Developing => {
                    "spotting this consistently is what drives progress at this level"
                }
                RatingBand::Advancing => "this is exactly the standard expected at this level",
                RatingBand::Expert => "this is strong play even at this level",
            }
        } else {
            match self.band {
                RatingBand::Novice => "mistakes like this are among the most common at this level",
                RatingBand::Developing => "this kind of oversight is a frequent miss at this level",
                RatingBand::Advancing => {
                    "players at this level usually catch this, so it is a habit worth building"
                }
                RatingBand::Expert => "even stronger players miss this from time to time",
            }
        };
        format!(
            "Player context: rated about {} in {} skill - {}.",
            self.rounded_rating(),
            self.category,
            note
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::heuristics::{Side, TacticalMotif};

    fn motif(tactic_type: TacticType) -> TacticalMotif {
        TacticalMotif {
            tactic_type,
            side: Side::White,
            square: "e4".to_string(),
            description: "test motif".to_string(),
        }
    }

    fn skill(rating: f64, games: u32) -> SkillRating {
        SkillRating {
            id: "id".to_string(),
            player_id: "p1".to_string(),
            category: "tactical".to_string(),
            rating,
            rd: 80.0,
            volatility: 0.06,
            games_count: games,
            last_updated: String::new(),
        }
    }

    #[test]
    fn band_boundaries() {
        assert_eq!(RatingBand::from_rating(100.0), RatingBand::Novice);
        assert_eq!(RatingBand::from_rating(1199.9), RatingBand::Novice);
        assert_eq!(RatingBand::from_rating(1200.0), RatingBand::Developing);
        assert_eq!(RatingBand::from_rating(1499.9), RatingBand::Developing);
        assert_eq!(RatingBand::from_rating(1500.0), RatingBand::Advancing);
        assert_eq!(RatingBand::from_rating(1800.0), RatingBand::Advancing);
        assert_eq!(RatingBand::from_rating(1800.1), RatingBand::Expert);
        assert_eq!(RatingBand::from_rating(2400.0), RatingBand::Expert);
    }

    #[test]
    fn every_tactic_maps_to_tactical() {
        let tactics = [
            TacticType::Pin,
            TacticType::Fork,
            TacticType::Skewer,
            TacticType::HangingPiece,
            TacticType::BackRankThreat,
            TacticType::DiscoveredAttack,
        ];
        for t in &tactics {
            assert_eq!(category_for_tactic(t), "tactical");
        }
    }

    #[test]
    fn theme_category_mapping() {
        assert_eq!(
            category_for_theme(&PositionalTheme::ForkAvailable),
            "tactical"
        );
        assert_eq!(
            category_for_theme(&PositionalTheme::HangingMaterial),
            "tactical"
        );
        assert_eq!(
            category_for_theme(&PositionalTheme::UndevelopedPieces),
            "opening"
        );
        assert_eq!(category_for_theme(&PositionalTheme::PassedPawn), "endgame");
        assert_eq!(
            category_for_theme(&PositionalTheme::CentralControl),
            "positional"
        );
        assert_eq!(
            category_for_theme(&PositionalTheme::KnightOnRim),
            "positional"
        );
    }

    #[test]
    fn every_theme_maps_to_known_category() {
        let themes = [
            PositionalTheme::KnightOnRim,
            PositionalTheme::BishopPairAdvantage,
            PositionalTheme::IsolatedQueenPawn,
            PositionalTheme::PassedPawn,
            PositionalTheme::DoubledPawns,
            PositionalTheme::BackwardPawn,
            PositionalTheme::OpenFile,
            PositionalTheme::RookOnSeventh,
            PositionalTheme::KingSafetyCompromised,
            PositionalTheme::UndevelopedPieces,
            PositionalTheme::CentralControl,
            PositionalTheme::PawnChainTension,
            PositionalTheme::MaterialImbalance,
            PositionalTheme::BackRankWeakness,
            PositionalTheme::PinnedPiece,
            PositionalTheme::ForkAvailable,
            PositionalTheme::HangingMaterial,
        ];
        let known = ["tactical", "positional", "endgame", "opening", "pattern"];
        for theme in &themes {
            let cat = category_for_theme(theme);
            assert!(known.contains(&cat), "unknown category {cat} for {theme:?}");
        }
    }

    #[test]
    fn context_prefers_tactic_over_theme() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let pos = crate::game::parse_fen(fen).unwrap();
        let mut ctx = crate::heuristics::analyze_position(&pos);
        ctx.themes = vec![PositionalTheme::PassedPawn];
        ctx.tactics = vec![motif(TacticType::Fork)];
        assert_eq!(category_for_context(&ctx), "tactical");

        ctx.tactics.clear();
        assert_eq!(category_for_context(&ctx), "endgame");
    }

    #[test]
    fn context_falls_back_to_phase() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let pos = crate::game::parse_fen(fen).unwrap();
        let mut ctx = crate::heuristics::analyze_position(&pos);
        ctx.themes.clear();
        ctx.tactics.clear();
        ctx.phase = GamePhase::Opening;
        assert_eq!(category_for_context(&ctx), "opening");
        ctx.phase = GamePhase::Endgame;
        assert_eq!(category_for_context(&ctx), "endgame");
        ctx.phase = GamePhase::Middlegame;
        assert_eq!(category_for_context(&ctx), "tactical");
    }

    #[test]
    fn rank_context_requires_min_games() {
        assert!(PlayerRankContext::from_skill_rating(&skill(1300.0, 0)).is_none());
        assert!(PlayerRankContext::from_skill_rating(&skill(1300.0, 2)).is_none());
        let ctx = PlayerRankContext::from_skill_rating(&skill(1300.0, 3)).unwrap();
        assert_eq!(ctx.band, RatingBand::Developing);
        assert_eq!(ctx.category, "tactical");
    }

    #[test]
    fn rating_rounds_to_nearest_fifty() {
        let ctx = PlayerRankContext::from_skill_rating(&skill(1287.3, 10)).unwrap();
        assert_eq!(ctx.rounded_rating(), 1300);
        let ctx = PlayerRankContext::from_skill_rating(&skill(1262.0, 10)).unwrap();
        assert_eq!(ctx.rounded_rating(), 1250);
    }

    #[test]
    fn prompt_line_is_qualitative_and_names_category() {
        let ctx = PlayerRankContext::from_skill_rating(&skill(1300.0, 10)).unwrap();
        let line = ctx.llm_prompt_line(false);
        assert!(line.starts_with("Player context: rated about 1300 in tactical skill"));
        assert!(!line.contains('%'), "no fabricated statistics: {line}");

        let positive = ctx.llm_prompt_line(true);
        assert_ne!(line, positive);
        assert!(positive.starts_with("Player context:"));
    }

    #[test]
    fn prompt_line_varies_by_band() {
        let lines: Vec<String> = [1000.0, 1300.0, 1600.0, 1900.0]
            .iter()
            .map(|r| {
                PlayerRankContext::from_skill_rating(&skill(*r, 10))
                    .unwrap()
                    .llm_prompt_line(false)
            })
            .collect();
        for pair in lines.windows(2) {
            assert_ne!(pair[0], pair[1]);
        }
    }
}
