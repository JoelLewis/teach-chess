use tauri::State;
use tracing::debug;

use crate::coaching;
use crate::coaching::templates;
use crate::engine::eval;
use crate::engine::process::EngineProcess;
use crate::error::AppError;
use crate::heuristics;
use crate::models::engine::{
    CoachingLevel, InGameCoachingFeedback, MoveClassification, PreMoveHint, PreMoveHintType, Score,
};
use crate::models::heuristics::GamePhase;

/// Evaluate a player's move during gameplay: engine analysis + classification + coaching text
#[tauri::command]
pub async fn evaluate_player_move(
    fen_before: String,
    fen_after: String,
    is_player_white: bool,
    move_number: u32,
    coaching_level: CoachingLevel,
    engine_state: State<'_, tokio::sync::Mutex<EngineProcess>>,
) -> Result<InGameCoachingFeedback, AppError> {
    // Silent mode: skip all engine work, return neutral feedback
    if coaching_level == CoachingLevel::Silent {
        return Ok(InGameCoachingFeedback {
            classification: MoveClassification::Good,
            coaching_text: String::new(),
            eval_before: Score::cp(0),
            eval_after: Score::cp(0),
            engine_best_uci: String::new(),
            coaching_context: None,
            move_number,
        });
    }

    debug!("Evaluating player move {move_number} at depth 10");

    let mut engine = engine_state.lock().await;

    // Analyze position before and after the move at depth 10 for speed
    let eval_before = engine.analyze(&fen_before, 10).await?;
    let eval_after = engine.analyze(&fen_after, 10).await?;

    drop(engine);

    let classification = eval::classify_move(&eval_before.score, &eval_after.score, is_player_white);

    // Filter by coaching level
    let should_show = match coaching_level {
        CoachingLevel::FullCoach => true,
        CoachingLevel::LightTouch => matches!(
            classification,
            MoveClassification::Best
                | MoveClassification::Inaccuracy
                | MoveClassification::Mistake
                | MoveClassification::Blunder
        ),
        CoachingLevel::Minimal => matches!(classification, MoveClassification::Blunder),
        CoachingLevel::Silent => false,
    };

    // Run heuristic analysis for coaching context
    let coaching_context = crate::game::parse_fen(&fen_before)
        .ok()
        .map(|pos| heuristics::analyze_position(&pos));

    let coaching_text = if should_show {
        coaching_context
            .as_ref()
            .map(|ctx| coaching::generate_coaching_text(&classification, ctx))
            .unwrap_or_else(|| {
                templates::generic_template(classification).to_string()
            })
    } else {
        String::new()
    };

    Ok(InGameCoachingFeedback {
        classification,
        coaching_text,
        eval_before: eval_before.score,
        eval_after: eval_after.score,
        engine_best_uci: eval_before.best_move,
        coaching_context,
        move_number,
    })
}

/// Analyze the current position for pre-move hints (heuristic-only, no engine)
#[tauri::command]
pub async fn analyze_pre_move_hints(
    fen: String,
    previous_phase: Option<GamePhase>,
    coaching_level: CoachingLevel,
    is_player_white: bool,
    opponent_personality: Option<crate::opponent::personality::PersonalityProfile>,
) -> Result<PreMoveHint, AppError> {
    // Only FullCoach shows pre-move hints
    if coaching_level != CoachingLevel::FullCoach {
        return Ok(PreMoveHint::default());
    }

    // FullCoach: run heuristic analysis
    let chess = match crate::game::parse_fen(&fen) {
        Ok(pos) => pos,
        Err(_) => return Ok(PreMoveHint::default()),
    };

    let ctx = heuristics::analyze_position(&chess);

    // Check for phase transition
    if let Some(prev) = &previous_phase {
        if *prev != ctx.phase {
            let text = templates::phase_transition_text(prev, &ctx.phase);
            if !text.is_empty() {
                return Ok(PreMoveHint {
                    hint_text: Some(text.to_string()),
                    hint_type: PreMoveHintType::PhaseTransition,
                    themes: ctx.themes,
                });
            }
        }
    }

    // Check for tactical alerts relevant to the player
    let player_side = if is_player_white {
        crate::models::heuristics::Side::White
    } else {
        crate::models::heuristics::Side::Black
    };

    // Opponent has hanging material (player can capture)
    let opponent_hanging = ctx.tactics.iter().any(|t| {
        t.tactic_type == crate::models::heuristics::TacticType::HangingPiece
            && t.side != player_side
    });

    // Player has a fork/pin available
    let player_tactic = ctx.tactics.iter().any(|t| {
        matches!(
            t.tactic_type,
            crate::models::heuristics::TacticType::Fork | crate::models::heuristics::TacticType::Pin
        ) && t.side == player_side
    });

    if opponent_hanging || player_tactic {
        let hint = if opponent_hanging {
            "Look carefully — there might be undefended material you can capture."
        } else {
            "There's a tactical opportunity in this position. Look for double attacks or pins."
        };
        return Ok(PreMoveHint {
            hint_text: Some(hint.to_string()),
            hint_type: PreMoveHintType::TacticalAlert,
            themes: ctx.themes,
        });
    }

    // Player has hanging material (defensive alert)
    let player_hanging = ctx.tactics.iter().any(|t| {
        t.tactic_type == crate::models::heuristics::TacticType::HangingPiece
            && t.side == player_side
    });

    if player_hanging {
        return Ok(PreMoveHint {
            hint_text: Some(
                "Be careful — you may have undefended material. Check all your pieces are protected."
                    .to_string(),
            ),
            hint_type: PreMoveHintType::TacticalAlert,
            themes: ctx.themes,
        });
    }

    // Personality-aware hint (show occasionally — every ~5 moves, based on move number)
    if let Some(ref personality) = opponent_personality {
        // Use the FEN's fullmove counter to decide when to show personality hints
        let fullmove: u32 = fen
            .split_whitespace()
            .nth(5)
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);
        if fullmove % 5 == 0 {
            let hint = templates::personality_hint(personality);
            return Ok(PreMoveHint {
                hint_text: Some(hint.to_string()),
                hint_type: PreMoveHintType::StrategicReminder,
                themes: ctx.themes,
            });
        }
    }

    // Strategic reminder based on themes
    let strategic_hint = generate_strategic_hint(&ctx.themes, is_player_white);
    if let Some(hint) = strategic_hint {
        return Ok(PreMoveHint {
            hint_text: Some(hint),
            hint_type: PreMoveHintType::StrategicReminder,
            themes: ctx.themes,
        });
    }

    Ok(PreMoveHint {
        themes: ctx.themes,
        ..PreMoveHint::default()
    })
}

fn generate_strategic_hint(
    themes: &[crate::models::heuristics::PositionalTheme],
    _is_player_white: bool,
) -> Option<String> {
    use crate::models::heuristics::PositionalTheme;

    for theme in themes {
        match theme {
            PositionalTheme::UndevelopedPieces => {
                return Some("You still have undeveloped pieces. Prioritize getting them into the game.".to_string());
            }
            PositionalTheme::KingSafetyCompromised => {
                return Some("Your king position looks exposed. Consider improving its safety.".to_string());
            }
            PositionalTheme::PassedPawn => {
                return Some("There's a passed pawn on the board — can you advance or blockade it?".to_string());
            }
            PositionalTheme::OpenFile => {
                return Some("There are open files available. Consider placing a rook on one.".to_string());
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coaching_level_filter_silent_shows_nothing() {
        // Silent should never show coaching
        let show = match CoachingLevel::Silent {
            CoachingLevel::FullCoach => true,
            CoachingLevel::LightTouch => matches!(
                MoveClassification::Blunder,
                MoveClassification::Best
                    | MoveClassification::Inaccuracy
                    | MoveClassification::Mistake
                    | MoveClassification::Blunder
            ),
            CoachingLevel::Minimal => {
                matches!(MoveClassification::Blunder, MoveClassification::Blunder)
            }
            CoachingLevel::Silent => false,
        };
        assert!(!show);
    }

    #[test]
    fn coaching_level_filter_minimal_shows_blunder_only() {
        let classifications = [
            (MoveClassification::Best, false),
            (MoveClassification::Excellent, false),
            (MoveClassification::Good, false),
            (MoveClassification::Inaccuracy, false),
            (MoveClassification::Mistake, false),
            (MoveClassification::Blunder, true),
        ];
        for (class, expected) in &classifications {
            let show = matches!(class, MoveClassification::Blunder);
            assert_eq!(show, *expected, "Minimal filter wrong for {class:?}");
        }
    }

    #[test]
    fn coaching_level_filter_light_touch() {
        let classifications = [
            (MoveClassification::Best, true),
            (MoveClassification::Excellent, false),
            (MoveClassification::Good, false),
            (MoveClassification::Inaccuracy, true),
            (MoveClassification::Mistake, true),
            (MoveClassification::Blunder, true),
        ];
        for (class, expected) in &classifications {
            let show = matches!(
                class,
                MoveClassification::Best
                    | MoveClassification::Inaccuracy
                    | MoveClassification::Mistake
                    | MoveClassification::Blunder
            );
            assert_eq!(show, *expected, "LightTouch filter wrong for {class:?}");
        }
    }

    #[test]
    fn coaching_level_filter_full_coach() {
        let classifications = [
            MoveClassification::Best,
            MoveClassification::Excellent,
            MoveClassification::Good,
            MoveClassification::Inaccuracy,
            MoveClassification::Mistake,
            MoveClassification::Blunder,
        ];
        for class in &classifications {
            // FullCoach always shows
            assert!(true, "FullCoach should show all, including {class:?}");
        }
    }

    #[test]
    fn strategic_hint_for_undeveloped_pieces() {
        use crate::models::heuristics::PositionalTheme;
        let themes = vec![PositionalTheme::UndevelopedPieces];
        let hint = generate_strategic_hint(&themes, true);
        assert!(hint.is_some());
        assert!(hint.unwrap().contains("undeveloped"));
    }

    #[test]
    fn no_strategic_hint_for_central_control_only() {
        use crate::models::heuristics::PositionalTheme;
        let themes = vec![PositionalTheme::CentralControl];
        let hint = generate_strategic_hint(&themes, true);
        assert!(hint.is_none());
    }
}
