use std::collections::HashMap;

use shakmaty::{fen::Fen, uci::UciMove, CastlingMode, Chess, EnPassantMode, Position as _};

use crate::error::{AppError, RepertoireError};
use crate::models::repertoire::{DrillMoveResult, DrillState, Opening, RepertoireEntry};
use crate::repertoire::ActiveDrill;

/// Start a drill session: set up the first entry's position.
pub fn start_drill(
    opening: &Opening,
    entries: Vec<RepertoireEntry>,
) -> Result<(DrillState, ActiveDrill), AppError> {
    if entries.is_empty() {
        return Err(RepertoireError::NoRepertoireEntries.into());
    }

    let entry = &entries[0];
    let (chess, fen, legal_dests) = setup_position(&entry.position_fen)?;

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let total = entries.len() as u32;

    let state = DrillState {
        opening: opening.clone(),
        current_entry: entry.clone(),
        fen: fen.clone(),
        opponent_move: None,
        player_color: opening.color.clone(),
        legal_dests: legal_dests.clone(),
        entries_total: total,
        entries_remaining: total,
    };

    let active = ActiveDrill {
        opening: opening.clone(),
        entries,
        current_index: 0,
        entry_start_time_ms: now_ms,
        current_fen: fen,
        legal_dests,
        player_color: opening.color.clone(),
        chess,
    };

    Ok((state, active))
}

/// Validate a drill move against the expected repertoire move.
pub fn validate_drill_move(
    active: &mut ActiveDrill,
    uci: &str,
) -> Result<DrillMoveResult, AppError> {
    let entry = active
        .entries
        .get(active.current_index)
        .ok_or(RepertoireError::NoDrillActive)?;

    let expected = &entry.move_uci;
    let correct = uci.to_lowercase() == expected.to_lowercase();

    let remaining = (active.entries.len() - active.current_index - 1) as u32;

    if !correct {
        return Ok(DrillMoveResult {
            correct: false,
            correct_move: Some(format_move_hint(expected, &entry.move_san)),
            is_complete: false,
            entries_remaining: remaining,
            explanation: Some(get_drill_explanation(entry, &active.opening)),
        });
    }

    let is_complete = active.current_index + 1 >= active.entries.len();

    Ok(DrillMoveResult {
        correct: true,
        correct_move: None,
        is_complete,
        entries_remaining: if is_complete { 0 } else { remaining },
        explanation: None,
    })
}

/// Advance to the next entry in the drill. Returns the new DrillState or None if complete.
pub fn advance_drill(active: &mut ActiveDrill) -> Result<Option<DrillState>, AppError> {
    active.current_index += 1;
    if active.current_index >= active.entries.len() {
        return Ok(None);
    }

    let entry = &active.entries[active.current_index];
    let (chess, fen, legal_dests) = setup_position(&entry.position_fen)?;

    active.chess = chess;
    active.current_fen = fen.clone();
    active.legal_dests = legal_dests.clone();
    active.entry_start_time_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let remaining = (active.entries.len() - active.current_index) as u32;

    Ok(Some(DrillState {
        opening: active.opening.clone(),
        current_entry: entry.clone(),
        fen,
        opponent_move: None,
        player_color: active.player_color.clone(),
        legal_dests,
        entries_total: active.entries.len() as u32,
        entries_remaining: remaining,
    }))
}

/// Get the elapsed time for the current drill entry.
pub fn get_entry_elapsed_ms(active: &ActiveDrill) -> u64 {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    now_ms.saturating_sub(active.entry_start_time_ms)
}

/// Map drill outcome to SRS quality.
/// 5: Correct, < 10s
/// 4: Correct, >= 10s
/// 1: Incorrect
pub fn drill_quality(correct: bool, time_ms: u64) -> u8 {
    if !correct {
        return 1;
    }
    if time_ms < 10_000 {
        5
    } else {
        4
    }
}

#[allow(clippy::type_complexity)]
fn setup_position(
    fen_str: &str,
) -> Result<(Chess, String, HashMap<String, Vec<String>>), AppError> {
    let fen: Fen = fen_str
        .parse()
        .map_err(|_| RepertoireError::ImportError(format!("Invalid FEN: {fen_str}")))?;
    let chess: Chess = fen
        .into_position(CastlingMode::Standard)
        .map_err(|_| RepertoireError::ImportError(format!("Invalid position: {fen_str}")))?;

    let fen_out = Fen::from_position(chess.clone(), EnPassantMode::Legal).to_string();
    let legal_dests = compute_legal_dests(&chess);

    Ok((chess, fen_out, legal_dests))
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

fn format_move_hint(uci: &str, san: &str) -> String {
    if san.is_empty() {
        uci.to_string()
    } else {
        format!("{san} ({uci})")
    }
}

fn get_drill_explanation(entry: &RepertoireEntry, opening: &Opening) -> String {
    if !entry.notes.is_empty() {
        return entry.notes.clone();
    }
    format!(
        "In the {}, the correct move here is {}.",
        opening.name, entry.move_san
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_opening() -> Opening {
        Opening {
            id: "italian".to_string(),
            name: "Italian Game".to_string(),
            eco: "C50".to_string(),
            color: "white".to_string(),
            description: "".to_string(),
            moves: "e2e4 e7e5 g1f3 b8c6 f1c4".to_string(),
            themes: "".to_string(),
            difficulty: 1000,
        }
    }

    fn make_test_entries() -> Vec<RepertoireEntry> {
        // Position: initial position, player should play e2e4
        vec![
            RepertoireEntry {
                id: "e1".to_string(),
                player_id: "p1".to_string(),
                opening_id: "italian".to_string(),
                position_fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
                    .to_string(),
                move_uci: "e2e4".to_string(),
                move_san: "e4".to_string(),
                notes: "".to_string(),
            },
            RepertoireEntry {
                id: "e2".to_string(),
                player_id: "p1".to_string(),
                opening_id: "italian".to_string(),
                position_fen: "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2"
                    .to_string(),
                move_uci: "g1f3".to_string(),
                move_san: "Nf3".to_string(),
                notes: "".to_string(),
            },
        ]
    }

    #[test]
    fn start_drill_creates_state() {
        let opening = make_test_opening();
        let entries = make_test_entries();
        let (state, active) = start_drill(&opening, entries).unwrap();

        assert_eq!(state.entries_total, 2);
        assert_eq!(state.entries_remaining, 2);
        assert_eq!(state.current_entry.move_uci, "e2e4");
        assert!(!active.legal_dests.is_empty());
    }

    #[test]
    fn correct_drill_move() {
        let opening = make_test_opening();
        let entries = make_test_entries();
        let (_, mut active) = start_drill(&opening, entries).unwrap();

        let result = validate_drill_move(&mut active, "e2e4").unwrap();
        assert!(result.correct);
        assert!(!result.is_complete);
    }

    #[test]
    fn incorrect_drill_move_shows_answer() {
        let opening = make_test_opening();
        let entries = make_test_entries();
        let (_, mut active) = start_drill(&opening, entries).unwrap();

        let result = validate_drill_move(&mut active, "d2d4").unwrap();
        assert!(!result.correct);
        assert!(result.correct_move.is_some());
        assert!(result.correct_move.unwrap().contains("e4"));
    }

    #[test]
    fn advance_to_next_entry() {
        let opening = make_test_opening();
        let entries = make_test_entries();
        let (_, mut active) = start_drill(&opening, entries).unwrap();

        let next = advance_drill(&mut active).unwrap();
        assert!(next.is_some());
        let state = next.unwrap();
        assert_eq!(state.current_entry.move_uci, "g1f3");
        assert_eq!(state.entries_remaining, 1);
    }

    #[test]
    fn drill_complete_after_last_entry() {
        let opening = make_test_opening();
        let entries = make_test_entries();
        let (_, mut active) = start_drill(&opening, entries).unwrap();

        // Advance past first entry
        advance_drill(&mut active).unwrap();

        // Validate second entry correct
        let result = validate_drill_move(&mut active, "g1f3").unwrap();
        assert!(result.correct);
        assert!(result.is_complete);
    }

    #[test]
    fn drill_quality_mapping() {
        assert_eq!(drill_quality(true, 5_000), 5);
        assert_eq!(drill_quality(true, 15_000), 4);
        assert_eq!(drill_quality(false, 5_000), 1);
    }

    #[test]
    fn empty_entries_returns_error() {
        let opening = make_test_opening();
        let result = start_drill(&opening, vec![]);
        assert!(result.is_err());
    }
}
