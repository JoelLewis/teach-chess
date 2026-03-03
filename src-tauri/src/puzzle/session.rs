use std::collections::HashMap;

use shakmaty::{fen::Fen, uci::UciMove, CastlingMode, Chess, EnPassantMode, Position as _};

use crate::error::{AppError, PuzzleError};
use crate::models::puzzle::{Puzzle, PuzzleMoveResult, PuzzleState};
use crate::puzzle::ActivePuzzle;

/// Start a puzzle session: parse FEN, apply the opponent's setup move,
/// compute legal dests, and return the initial PuzzleState.
pub fn start_puzzle(puzzle: &Puzzle) -> Result<(PuzzleState, ActivePuzzle), AppError> {
    let solution_moves: Vec<String> = puzzle
        .solution_moves
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    if solution_moves.is_empty() {
        return Err(PuzzleError::InvalidMove("No solution moves".to_string()).into());
    }

    // Parse the starting FEN
    let fen: Fen = puzzle
        .fen
        .parse()
        .map_err(|_| PuzzleError::InvalidMove(format!("Invalid FEN: {}", puzzle.fen)))?;
    let mut chess: Chess = fen
        .into_position(CastlingMode::Standard)
        .map_err(|_| PuzzleError::InvalidMove(format!("Invalid position: {}", puzzle.fen)))?;

    // The first move is the opponent's setup move — apply it automatically
    let setup_uci = &solution_moves[0];
    let setup_move: UciMove = setup_uci
        .parse()
        .map_err(|_| PuzzleError::InvalidMove(format!("Invalid setup UCI: {setup_uci}")))?;
    let legal_setup = setup_move
        .to_move(&chess)
        .map_err(|_| PuzzleError::InvalidMove(format!("Illegal setup move: {setup_uci}")))?;
    chess.play_unchecked(&legal_setup);

    // Now it's the player's turn
    let player_color = match chess.turn() {
        shakmaty::Color::White => "white",
        shakmaty::Color::Black => "black",
    };

    let start_fen = Fen::from_position(chess.clone(), EnPassantMode::Legal).to_string();
    let legal_dests = compute_legal_dests(&chess);

    // Count player moves: every other move starting from index 1
    let total_player_moves = solution_moves
        .iter()
        .enumerate()
        .filter(|(i, _)| i % 2 == 1) // indices 1, 3, 5, ... are player moves
        .count() as u32;

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    // Extract from/to of setup move for last_move highlight
    let setup_uci_str = setup_uci.to_string();
    let setup_from = setup_uci_str[..2].to_string();
    let setup_to = setup_uci_str[2..4].to_string();

    let state = PuzzleState {
        puzzle: puzzle.clone(),
        start_fen: start_fen.clone(),
        player_color: player_color.to_string(),
        legal_dests: legal_dests.clone(),
        total_player_moves,
        current_move_index: 0,
    };

    let active = ActivePuzzle {
        puzzle: puzzle.clone(),
        solution_moves,
        current_move_index: 1, // Next expected move is index 1 (first player move)
        hints_revealed: 0,
        start_time_ms: now_ms,
        current_fen: start_fen,
        legal_dests,
        player_color: player_color.to_string(),
        chess,
    };

    let _ = (setup_from, setup_to); // frontend tracks setup move via start_fen

    Ok((state, active))
}

/// Validate a player's move against the expected solution move.
/// If correct, apply it and auto-play the opponent's response if any.
pub fn validate_move(active: &mut ActivePuzzle, uci: &str) -> Result<PuzzleMoveResult, AppError> {
    let idx = active.current_move_index;
    if idx >= active.solution_moves.len() {
        return Err(PuzzleError::PuzzleComplete.into());
    }

    let expected = &active.solution_moves[idx];

    // Normalize: strip promotion suffix for comparison if needed,
    // or try adding 'q' for auto-queen promotion
    let uci_normalized = normalize_uci(uci);
    let expected_normalized = normalize_uci(expected);

    let correct = uci_normalized == expected_normalized;

    if !correct {
        // Player's move doesn't match — return incorrect result
        return Ok(PuzzleMoveResult {
            correct: false,
            is_complete: false,
            fen: Some(active.current_fen.clone()),
            legal_dests: Some(active.legal_dests.clone()),
            last_move: None,
            current_move_index: ((idx - 1) / 2) as u32, // player move index (0-based)
            explanation: None,
        });
    }

    // Apply the correct player move
    apply_uci_move(&mut active.chess, expected)?;
    let mut last_from = expected[..2].to_string();
    let mut last_to = expected[2..4].to_string();

    active.current_move_index = idx + 1;

    // Check if there's an opponent response to auto-play
    let next_idx = active.current_move_index;
    if next_idx < active.solution_moves.len() {
        // There's an opponent response — auto-play it
        let opponent_move = &active.solution_moves[next_idx].clone();
        apply_uci_move(&mut active.chess, opponent_move)?;
        last_from = opponent_move[..2].to_string();
        last_to = opponent_move[2..4].to_string();
        active.current_move_index = next_idx + 1;
    }

    // Update position state
    active.current_fen = Fen::from_position(active.chess.clone(), EnPassantMode::Legal).to_string();
    active.legal_dests = compute_legal_dests(&active.chess);

    let is_complete = active.current_move_index >= active.solution_moves.len();
    let player_move_idx = ((active.current_move_index.saturating_sub(1)) / 2) as u32;

    let explanation = if is_complete {
        Some(get_solution_explanation(&active.puzzle))
    } else {
        None
    };

    Ok(PuzzleMoveResult {
        correct: true,
        is_complete,
        fen: Some(active.current_fen.clone()),
        legal_dests: if is_complete {
            None
        } else {
            Some(active.legal_dests.clone())
        },
        last_move: Some([last_from, last_to]),
        current_move_index: player_move_idx,
        explanation,
    })
}

/// Reveal the next hint tier. Returns the hint text, or None if all hints used.
pub fn reveal_hint(active: &mut ActivePuzzle) -> Option<String> {
    let hints: Vec<String> = serde_json::from_str(&active.puzzle.hints_json).unwrap_or_default();
    let idx = active.hints_revealed as usize;
    if idx >= hints.len() {
        return None;
    }
    active.hints_revealed += 1;
    Some(hints[idx].clone())
}

/// Generate a template explanation for the puzzle solution.
pub fn get_solution_explanation(puzzle: &Puzzle) -> String {
    if !puzzle.explanation.is_empty() {
        return puzzle.explanation.clone();
    }

    let themes: Vec<&str> = puzzle
        .themes
        .split(',')
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .collect();

    if themes.is_empty() {
        return "Well done! You found the correct combination.".to_string();
    }

    let theme_desc = themes
        .iter()
        .map(|t| humanize_theme(t))
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        "This puzzle featured: {}. The key was to recognize the pattern and calculate the forcing sequence.",
        theme_desc
    )
}

fn humanize_theme(theme: &str) -> &str {
    match theme {
        "fork" => "a fork (attacking two pieces at once)",
        "pin" => "a pin (restricting a piece's movement)",
        "skewer" => "a skewer (attacking through a piece)",
        "discoveredAttack" => "a discovered attack",
        "doubleCheck" => "a double check",
        "sacrifice" => "a sacrifice",
        "mate" | "mateIn1" | "mateIn2" | "mateIn3" | "mateIn4" | "mateIn5" => "a checkmate pattern",
        "hangingPiece" => "a hanging piece",
        "trappedPiece" => "a trapped piece",
        "deflection" => "a deflection",
        "decoy" | "attraction" => "a decoy/attraction",
        "overloading" => "an overloaded defender",
        "interference" => "an interference",
        "clearance" => "a clearance sacrifice",
        "zugzwang" => "zugzwang",
        "backRankMate" => "a back rank mate",
        "smotheredMate" => "a smothered mate",
        "castling" => "a castling tactic",
        "enPassant" => "an en passant capture",
        "promotion" => "a pawn promotion",
        "underPromotion" => "an under-promotion",
        "quietMove" => "a quiet move (non-forcing but decisive)",
        "xRayAttack" => "an X-ray attack",
        "intermezzo" | "zwischenzug" => "an intermezzo (in-between move)",
        "exposedKing" => "an exposed king",
        "kingsideAttack" => "a kingside attack",
        "queensideAttack" => "a queenside attack",
        _ => theme,
    }
}

fn normalize_uci(uci: &str) -> String {
    // Normalize UCI by ensuring promotion is lowercase
    uci.to_lowercase()
}

fn apply_uci_move(chess: &mut Chess, uci_str: &str) -> Result<(), AppError> {
    let uci: UciMove = uci_str
        .parse()
        .map_err(|_| PuzzleError::InvalidMove(format!("Invalid UCI: {uci_str}")))?;
    let legal = uci
        .to_move(chess)
        .map_err(|_| PuzzleError::InvalidMove(format!("Illegal move: {uci_str}")))?;
    chess.play_unchecked(&legal);
    Ok(())
}

fn compute_legal_dests(chess: &Chess) -> HashMap<String, Vec<String>> {
    let mut dests: HashMap<String, Vec<String>> = HashMap::new();
    for m in chess.legal_moves() {
        let uci = UciMove::from_move(&m, CastlingMode::Standard);
        let uci_str = uci.to_string();
        let from = uci_str[..2].to_string();
        let to = uci_str[2..4].to_string();
        dests.entry(from).or_default().push(to);
    }
    dests
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::puzzle::{Puzzle, PuzzleCategory};

    fn make_test_puzzle() -> Puzzle {
        // Simple 2-move puzzle: e2e4 is setup, d7d5 is player's move, e4d5 is opponent response, d8d5 is player's final move
        // Actually for a simpler test: setup=e2e4, player=d7d5 (one player move)
        // Let's use a real mate-in-1 pattern
        // FEN where after opponent plays Qh5, player needs to play g6 (but that's defense)
        // Better: use a simple fork puzzle
        Puzzle {
            id: "test1".to_string(),
            fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
            // Setup: e2e4, Player: e7e5 (simple opening sequence for testing)
            solution_moves: "e2e4 e7e5".to_string(),
            themes: "fork".to_string(),
            category: PuzzleCategory::Tactical,
            difficulty: 1000,
            source_id: None,
            hints_json: r#"["Look for a fork","The knight can attack two pieces","Nc7+ forks king and rook"]"#.to_string(),
            explanation: "".to_string(),
        }
    }

    fn make_multi_move_puzzle() -> Puzzle {
        // 4-move puzzle: setup, player, response, player
        Puzzle {
            id: "test2".to_string(),
            fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
            solution_moves: "e2e4 e7e5 d2d4 d7d5".to_string(),
            themes: "fork,pin".to_string(),
            category: PuzzleCategory::Tactical,
            difficulty: 1200,
            source_id: None,
            hints_json: r#"["Hint 1","Hint 2","Hint 3"]"#.to_string(),
            explanation: "Test explanation".to_string(),
        }
    }

    #[test]
    fn start_puzzle_parses_correctly() {
        let puzzle = make_test_puzzle();
        let (state, active) = start_puzzle(&puzzle).unwrap();

        // After setup move e2e4, it should be black's turn
        assert_eq!(state.player_color, "black");
        assert_eq!(state.total_player_moves, 1);
        assert_eq!(active.current_move_index, 1);
        assert!(!active.legal_dests.is_empty());
    }

    #[test]
    fn correct_move_completes_puzzle() {
        let puzzle = make_test_puzzle();
        let (_, mut active) = start_puzzle(&puzzle).unwrap();

        let result = validate_move(&mut active, "e7e5").unwrap();
        assert!(result.correct);
        assert!(result.is_complete);
    }

    #[test]
    fn incorrect_move_returns_false() {
        let puzzle = make_test_puzzle();
        let (_, mut active) = start_puzzle(&puzzle).unwrap();

        let result = validate_move(&mut active, "d7d5").unwrap();
        assert!(!result.correct);
        assert!(!result.is_complete);
    }

    #[test]
    fn multi_move_puzzle_auto_plays_opponent() {
        let puzzle = make_multi_move_puzzle();
        let (_, mut active) = start_puzzle(&puzzle).unwrap();

        // First player move
        let result = validate_move(&mut active, "e7e5").unwrap();
        assert!(result.correct);
        assert!(!result.is_complete);
        // Opponent's d2d4 was auto-played, so the FEN should reflect both moves
        assert!(result.fen.is_some());
        assert!(result.legal_dests.is_some());

        // Second player move
        let result = validate_move(&mut active, "d7d5").unwrap();
        assert!(result.correct);
        assert!(result.is_complete);
        assert_eq!(result.explanation, Some("Test explanation".to_string()));
    }

    #[test]
    fn hint_reveal_progressive() {
        let puzzle = make_test_puzzle();
        let (_, mut active) = start_puzzle(&puzzle).unwrap();

        let h1 = reveal_hint(&mut active);
        assert_eq!(h1, Some("Look for a fork".to_string()));
        assert_eq!(active.hints_revealed, 1);

        let h2 = reveal_hint(&mut active);
        assert_eq!(h2, Some("The knight can attack two pieces".to_string()));
        assert_eq!(active.hints_revealed, 2);

        let h3 = reveal_hint(&mut active);
        assert_eq!(h3, Some("Nc7+ forks king and rook".to_string()));
        assert_eq!(active.hints_revealed, 3);

        let h4 = reveal_hint(&mut active);
        assert_eq!(h4, None); // No more hints
    }

    #[test]
    fn explanation_from_themes() {
        let puzzle = Puzzle {
            id: "t".to_string(),
            fen: "8/8/8/8/8/8/8/8 w - - 0 1".to_string(),
            solution_moves: "".to_string(),
            themes: "fork,pin".to_string(),
            category: PuzzleCategory::Tactical,
            difficulty: 1000,
            source_id: None,
            hints_json: "[]".to_string(),
            explanation: "".to_string(),
        };

        let explanation = get_solution_explanation(&puzzle);
        assert!(explanation.contains("fork"));
        assert!(explanation.contains("pin"));
    }

    #[test]
    fn explanation_uses_custom_when_present() {
        let puzzle = Puzzle {
            id: "t".to_string(),
            fen: "8/8/8/8/8/8/8/8 w - - 0 1".to_string(),
            solution_moves: "".to_string(),
            themes: "fork".to_string(),
            category: PuzzleCategory::Tactical,
            difficulty: 1000,
            source_id: None,
            hints_json: "[]".to_string(),
            explanation: "Custom explanation here".to_string(),
        };

        assert_eq!(get_solution_explanation(&puzzle), "Custom explanation here");
    }
}
