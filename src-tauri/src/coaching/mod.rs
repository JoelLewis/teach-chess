pub(crate) mod templates;

use std::collections::HashMap;

use crate::models::engine::{
    MoveClassification, MoveEvaluation, PatternSummary, StudySuggestion,
};
use crate::models::heuristics::{CoachingContext, GamePhase, PositionalTheme, TacticType};

/// Generate human-readable coaching text from a move classification and coaching context.
///
/// Priority for error moves (Inaccuracy/Mistake/Blunder):
///   1. Tactic description (if a tactical motif was detected)
///   2. Theme-specific template (first matching theme)
///   3. Generic classification template
///
/// For good moves (Best/Excellent):
///   1. Theme-specific positive reinforcement (if a notable theme exists)
///   2. Generic classification template
///
/// For neutral moves (Good):
///   1. Theme-specific error template (if relevant)
///   2. Generic classification template
pub fn generate_coaching_text(
    classification: &MoveClassification,
    context: &CoachingContext,
) -> String {
    // For error moves, prioritize tactic descriptions
    if classification.is_error() {
        if let Some(tactic) = context.tactics.first() {
            return templates::tactic_template(&tactic.tactic_type).to_string();
        }
    }

    // For error moves, look for theme-specific templates
    if classification.is_error() {
        for theme in &context.themes {
            if let Some(text) = templates::theme_error_template(*classification, theme) {
                return text.to_string();
            }
        }
    }

    // For positive moves, look for theme-specific positive reinforcement
    if classification.is_positive() {
        for theme in &context.themes {
            if let Some(text) = templates::theme_positive_template(*classification, theme) {
                return text.to_string();
            }
        }
    }

    // Fall back to generic classification template
    templates::generic_template(*classification).to_string()
}

/// Analyze all move evaluations to find recurring weakness patterns across the game.
pub fn generate_pattern_summary(
    evaluations: &[MoveEvaluation],
    is_player_white: bool,
) -> PatternSummary {
    let mut total_errors = 0u32;
    let mut theme_counts: HashMap<PositionalTheme, u32> = HashMap::new();
    let mut tactic_counts: HashMap<TacticType, u32> = HashMap::new();
    let mut phase_errors: HashMap<GamePhase, u32> = HashMap::new();
    let mut total_moves = 0u32;
    let mut best_count = 0u32;
    let mut phase_totals: HashMap<GamePhase, u32> = HashMap::new();

    for mv in evaluations {
        // Only analyze the player's moves
        if mv.is_white != is_player_white {
            continue;
        }

        total_moves += 1;
        let classification = mv.classification.unwrap_or(MoveClassification::Good);

        if matches!(classification, MoveClassification::Best) {
            best_count += 1;
        }

        // Track phase totals for strength detection
        if let Some(ctx) = &mv.coaching_context {
            *phase_totals.entry(ctx.phase).or_default() += 1;
        }

        if !classification.is_error() {
            continue;
        }

        total_errors += 1;

        if let Some(ctx) = &mv.coaching_context {
            // Count error themes
            for theme in &ctx.themes {
                *theme_counts.entry(theme.clone()).or_default() += 1;
            }

            // Count missed tactics
            for tactic in &ctx.tactics {
                *tactic_counts.entry(tactic.tactic_type.clone()).or_default() += 1;
            }

            // Count errors by phase
            *phase_errors.entry(ctx.phase).or_default() += 1;
        }
    }

    // Sort themes and tactics by frequency (descending)
    let mut error_themes: Vec<(PositionalTheme, u32)> = theme_counts.into_iter().collect();
    error_themes.sort_by(|a, b| b.1.cmp(&a.1));

    let mut missed_tactics: Vec<(TacticType, u32)> = tactic_counts.into_iter().collect();
    missed_tactics.sort_by(|a, b| b.1.cmp(&a.1));

    // Derive strengths
    let mut strengths = Vec::new();

    // Good ratio of best moves
    if total_moves > 0 && (best_count as f64 / total_moves as f64) > 0.3 {
        strengths.push("Strong move accuracy — many best moves found".to_string());
    }

    // Phase-specific strengths (no errors in a phase with at least 3 moves)
    for (phase, total) in &phase_totals {
        let errors = phase_errors.get(phase).copied().unwrap_or(0);
        if *total >= 3 && errors == 0 {
            strengths.push(format!(
                "Clean {} play — no errors",
                match phase {
                    GamePhase::Opening => "opening",
                    GamePhase::Middlegame => "middlegame",
                    GamePhase::Endgame => "endgame",
                }
            ));
        }
    }

    PatternSummary {
        total_errors,
        error_themes,
        missed_tactics,
        errors_by_phase: phase_errors,
        strengths,
    }
}

/// Generate study suggestions based on pattern summary.
pub fn generate_study_suggestions(summary: &PatternSummary) -> Vec<StudySuggestion> {
    let mut suggestions = Vec::new();
    let mut priority = 1u8;

    // Suggestions from most frequent error themes
    for (theme, count) in &summary.error_themes {
        if *count < 2 {
            break; // Only suggest for recurring themes
        }

        let (topic, description) = match theme {
            PositionalTheme::KingSafetyCompromised => (
                "King Safety",
                "Practice castling early and maintaining a strong pawn shield. Study typical kingside attack patterns to recognize when your king is in danger.",
            ),
            PositionalTheme::HangingMaterial => (
                "Blunder Prevention",
                "Before every move, do a 'blunder check' — scan the board for undefended pieces. Practice the 'checks, captures, threats' method.",
            ),
            PositionalTheme::UndevelopedPieces => (
                "Opening Development",
                "Focus on getting all your pieces out before starting an attack. Follow the principle: knights before bishops, castle early, connect your rooks.",
            ),
            PositionalTheme::ForkAvailable => (
                "Knight Fork Tactics",
                "Practice knight fork puzzles. Knights are tricky — look for positions where a knight can attack two or more pieces simultaneously.",
            ),
            PositionalTheme::PinnedPiece => (
                "Pin Tactics",
                "Study pin motifs with bishops and rooks. Learn to recognize when a piece is pinned and how to exploit or avoid pins.",
            ),
            PositionalTheme::BackRankWeakness => (
                "Back Rank Safety",
                "Always check if your king has a 'luft' (escape square). Practice back-rank mate puzzles to recognize the pattern.",
            ),
            PositionalTheme::PassedPawn => (
                "Passed Pawn Play",
                "Learn when to push vs. blockade passed pawns. In endgames, passed pawns are your greatest asset — practice converting them.",
            ),
            PositionalTheme::CentralControl => (
                "Central Control",
                "Study how pawns and pieces control the center. A strong center gives your pieces more mobility and attacking chances.",
            ),
            PositionalTheme::KnightOnRim => (
                "Piece Placement",
                "Remember: 'a knight on the rim is dim.' Practice keeping knights centralized where they control the most squares.",
            ),
            _ => continue,
        };

        suggestions.push(StudySuggestion {
            topic: topic.to_string(),
            description: description.to_string(),
            priority,
        });
        priority += 1;

        if suggestions.len() >= 3 {
            break;
        }
    }

    // Suggestions from missed tactics
    for (tactic, count) in &summary.missed_tactics {
        if *count < 2 || suggestions.len() >= 5 {
            break;
        }

        let (topic, description) = match tactic {
            TacticType::Fork => (
                "Fork Puzzles",
                "Practice double-attack puzzles, especially with knights and pawns. Look for positions where one piece can threaten two targets.",
            ),
            TacticType::Pin => (
                "Pin & Skewer Tactics",
                "Study pin and skewer motifs. These linear tactics with bishops and rooks are among the most common patterns.",
            ),
            TacticType::Skewer => (
                "Skewer Tactics",
                "Practice skewer puzzles — the reverse of a pin. A high-value piece is attacked and must move, exposing a piece behind it.",
            ),
            TacticType::HangingPiece => (
                "Piece Safety Awareness",
                "After every opponent move, scan for your hanging pieces. Practice the habit of asking 'what did my opponent's last move threaten?'",
            ),
            TacticType::BackRankThreat => (
                "Back Rank Awareness",
                "Practice back-rank checkmate puzzles. Always ensure your king has an escape square, especially in the middlegame.",
            ),
            TacticType::DiscoveredAttack => (
                "Discovered Attacks",
                "Study discovered attack patterns. When one piece moves, it can reveal a powerful attack from the piece behind it.",
            ),
        };

        suggestions.push(StudySuggestion {
            topic: topic.to_string(),
            description: description.to_string(),
            priority,
        });
        priority += 1;
    }

    // Phase-specific suggestions
    for (phase, error_count) in &summary.errors_by_phase {
        if *error_count >= 3 && suggestions.len() < 5 {
            let (topic, description) = match phase {
                GamePhase::Opening => (
                    "Opening Principles",
                    "Review basic opening principles: control the center, develop pieces, castle early. Consider learning 1-2 openings more deeply.",
                ),
                GamePhase::Middlegame => (
                    "Middlegame Strategy",
                    "Study middlegame planning: identify imbalances, create targets, and coordinate your pieces toward a plan.",
                ),
                GamePhase::Endgame => (
                    "Basic Endgames",
                    "Study fundamental endgame positions: king + pawn vs king, rook endgames, and the principle of king activity.",
                ),
            };

            suggestions.push(StudySuggestion {
                topic: topic.to_string(),
                description: description.to_string(),
                priority,
            });
            priority += 1;
        }
    }

    suggestions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::heuristics::*;

    fn default_context() -> CoachingContext {
        CoachingContext {
            fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
            phase: GamePhase::Opening,
            material: MaterialBalance {
                white: PieceCounts {
                    pawns: 8,
                    knights: 2,
                    bishops: 2,
                    rooks: 2,
                    queens: 1,
                },
                black: PieceCounts {
                    pawns: 8,
                    knights: 2,
                    bishops: 2,
                    rooks: 2,
                    queens: 1,
                },
                balance_cp: 0,
                imbalances: vec![],
            },
            pawns: PawnStructure {
                white: SidePawnStructure {
                    isolated: vec![],
                    doubled: vec![],
                    passed: vec![],
                    backward: vec![],
                },
                black: SidePawnStructure {
                    isolated: vec![],
                    doubled: vec![],
                    passed: vec![],
                    backward: vec![],
                },
                chains: vec![],
                open_files: vec![],
                half_open_files_white: vec![],
                half_open_files_black: vec![],
            },
            activity: PieceActivity {
                white: SideActivity {
                    total_mobility: 20,
                    developed_minors: 0,
                    total_minors: 4,
                    rook_on_open_file: false,
                    rook_on_seventh: false,
                    pieces: vec![],
                },
                black: SideActivity {
                    total_mobility: 20,
                    developed_minors: 0,
                    total_minors: 4,
                    rook_on_open_file: false,
                    rook_on_seventh: false,
                    pieces: vec![],
                },
            },
            king_safety: KingSafety {
                white: SideKingSafety {
                    king_square: "e1".to_string(),
                    pawn_shield_count: 3,
                    pawn_shield_max: 3,
                    has_castled: false,
                    can_castle: true,
                    open_files_near_king: 0,
                    king_zone_attacks: 0,
                },
                black: SideKingSafety {
                    king_square: "e8".to_string(),
                    pawn_shield_count: 3,
                    pawn_shield_max: 3,
                    has_castled: false,
                    can_castle: true,
                    open_files_near_king: 0,
                    king_zone_attacks: 0,
                },
            },
            tactics: vec![],
            themes: vec![],
        }
    }

    #[test]
    fn every_classification_produces_nonempty_text() {
        let ctx = default_context();
        let classifications = [
            MoveClassification::Best,
            MoveClassification::Excellent,
            MoveClassification::Good,
            MoveClassification::Inaccuracy,
            MoveClassification::Mistake,
            MoveClassification::Blunder,
        ];
        for c in &classifications {
            let text = generate_coaching_text(c, &ctx);
            assert!(!text.is_empty(), "Empty coaching text for {c:?}");
        }
    }

    #[test]
    fn theme_specific_template_selected_for_errors() {
        let mut ctx = default_context();
        ctx.themes.push(PositionalTheme::KnightOnRim);

        let text = generate_coaching_text(&MoveClassification::Mistake, &ctx);
        assert!(
            text.contains("rim"),
            "Expected knight-on-rim template, got: {text}"
        );
    }

    #[test]
    fn tactic_prioritized_over_theme_for_errors() {
        let mut ctx = default_context();
        ctx.themes.push(PositionalTheme::KnightOnRim);
        ctx.tactics.push(TacticalMotif {
            tactic_type: TacticType::Fork,
            side: Side::White,
            square: "e4".to_string(),
            description: "Knight fork on e4".to_string(),
        });

        let text = generate_coaching_text(&MoveClassification::Blunder, &ctx);
        assert!(
            text.contains("fork"),
            "Expected fork tactic template, got: {text}"
        );
    }

    #[test]
    fn positive_theme_for_best_move() {
        let mut ctx = default_context();
        ctx.themes.push(PositionalTheme::CentralControl);

        let text = generate_coaching_text(&MoveClassification::Best, &ctx);
        assert!(
            text.contains("center"),
            "Expected central control positive template, got: {text}"
        );
    }

    #[test]
    fn generic_fallback_when_no_themes() {
        let ctx = default_context();

        let text = generate_coaching_text(&MoveClassification::Good, &ctx);
        assert_eq!(text, "Solid choice. This keeps the position balanced.");
    }

    #[test]
    fn tactic_not_used_for_positive_moves() {
        let mut ctx = default_context();
        ctx.tactics.push(TacticalMotif {
            tactic_type: TacticType::Fork,
            side: Side::White,
            square: "e4".to_string(),
            description: "Knight fork on e4".to_string(),
        });

        let text = generate_coaching_text(&MoveClassification::Best, &ctx);
        // Should NOT mention fork — tactics are only for error moves
        assert!(
            !text.contains("fork"),
            "Tactic template should not appear for Best move, got: {text}"
        );
    }

    #[test]
    fn back_rank_weakness_template_for_blunder() {
        let mut ctx = default_context();
        ctx.themes.push(PositionalTheme::BackRankWeakness);

        let text = generate_coaching_text(&MoveClassification::Blunder, &ctx);
        assert!(
            text.contains("back rank") || text.contains("back-rank"),
            "Expected back-rank template, got: {text}"
        );
    }

    #[test]
    fn hanging_material_template_for_inaccuracy() {
        let mut ctx = default_context();
        ctx.themes.push(PositionalTheme::HangingMaterial);

        let text = generate_coaching_text(&MoveClassification::Inaccuracy, &ctx);
        assert!(
            text.contains("undefended") || text.contains("hanging"),
            "Expected hanging material template, got: {text}"
        );
    }

    use crate::models::engine::Score;

    fn make_test_move_with_context(
        move_number: u32,
        is_white: bool,
        classification: MoveClassification,
        phase: GamePhase,
        themes: Vec<PositionalTheme>,
        tactics: Vec<TacticalMotif>,
    ) -> MoveEvaluation {
        let mut ctx = default_context();
        ctx.phase = phase;
        ctx.themes = themes;
        ctx.tactics = tactics;

        MoveEvaluation {
            move_number,
            is_white,
            fen_before: String::new(),
            player_move_uci: String::new(),
            player_move_san: "e4".to_string(),
            engine_best_uci: None,
            engine_best_san: None,
            eval_before: Some(Score::cp(0)),
            eval_after: Some(Score::cp(0)),
            classification: Some(classification),
            depth: 18,
            pv: vec![],
            coaching_context: Some(ctx),
            coaching_text: None,
        }
    }

    #[test]
    fn pattern_summary_counts_errors() {
        let moves = vec![
            make_test_move_with_context(
                1, true, MoveClassification::Mistake,
                GamePhase::Opening,
                vec![PositionalTheme::HangingMaterial],
                vec![],
            ),
            make_test_move_with_context(
                2, true, MoveClassification::Blunder,
                GamePhase::Opening,
                vec![PositionalTheme::HangingMaterial, PositionalTheme::KingSafetyCompromised],
                vec![],
            ),
            make_test_move_with_context(
                3, true, MoveClassification::Best,
                GamePhase::Middlegame,
                vec![],
                vec![],
            ),
        ];

        let summary = generate_pattern_summary(&moves, true);
        assert_eq!(summary.total_errors, 2);
        assert_eq!(summary.error_themes[0].0, PositionalTheme::HangingMaterial);
        assert_eq!(summary.error_themes[0].1, 2);
        assert_eq!(*summary.errors_by_phase.get(&GamePhase::Opening).unwrap(), 2);
    }

    #[test]
    fn pattern_summary_only_counts_player_moves() {
        let moves = vec![
            make_test_move_with_context(
                1, true, MoveClassification::Blunder,
                GamePhase::Opening, vec![], vec![],
            ),
            make_test_move_with_context(
                1, false, MoveClassification::Blunder,
                GamePhase::Opening, vec![], vec![],
            ),
        ];

        // Player is white: should only count the first move
        let summary = generate_pattern_summary(&moves, true);
        assert_eq!(summary.total_errors, 1);
    }

    #[test]
    fn study_suggestions_from_recurring_themes() {
        let moves = vec![
            make_test_move_with_context(
                1, true, MoveClassification::Mistake,
                GamePhase::Middlegame,
                vec![PositionalTheme::HangingMaterial],
                vec![],
            ),
            make_test_move_with_context(
                2, true, MoveClassification::Blunder,
                GamePhase::Middlegame,
                vec![PositionalTheme::HangingMaterial],
                vec![],
            ),
        ];

        let summary = generate_pattern_summary(&moves, true);
        let suggestions = generate_study_suggestions(&summary);
        assert!(!suggestions.is_empty());
        assert!(suggestions[0].topic.contains("Blunder"));
    }

    #[test]
    fn no_suggestions_for_non_recurring_themes() {
        let moves = vec![
            make_test_move_with_context(
                1, true, MoveClassification::Mistake,
                GamePhase::Middlegame,
                vec![PositionalTheme::HangingMaterial],
                vec![],
            ),
        ];

        let summary = generate_pattern_summary(&moves, true);
        let suggestions = generate_study_suggestions(&summary);
        // Only 1 occurrence of HangingMaterial, threshold is 2
        assert!(suggestions.is_empty());
    }
}
