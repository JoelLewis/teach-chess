#![allow(dead_code)]

use shakmaty::{uci::UciMove, Chess, Position as _};

use crate::models::chess::{LegalMove, PieceRole};

/// Convert all legal moves to LegalMove structs
pub fn legal_moves_list(chess: &Chess) -> Vec<LegalMove> {
    chess
        .legal_moves()
        .iter()
        .map(|m| {
            let san = shakmaty::san::San::from_move(chess, m);
            let uci = UciMove::from_move(m, shakmaty::CastlingMode::Standard);
            let uci_str = uci.to_string();

            LegalMove {
                uci: uci_str.clone(),
                san: san.to_string(),
                from: uci_str[..2].to_string(),
                to: uci_str[2..4].to_string(),
                promotion: uci_str.get(4..5).and_then(|c| match c {
                    "q" => Some(PieceRole::Queen),
                    "r" => Some(PieceRole::Rook),
                    "b" => Some(PieceRole::Bishop),
                    "n" => Some(PieceRole::Knight),
                    _ => None,
                }),
            }
        })
        .collect()
}

/// Check if a UCI string represents a promotion move
pub fn is_promotion(uci: &str) -> bool {
    uci.len() == 5 && matches!(uci.as_bytes()[4], b'q' | b'r' | b'b' | b'n')
}

/// Check if a move from a source square to a destination could be a promotion
pub fn needs_promotion(chess: &Chess, from: &str, to: &str) -> bool {
    let base_uci = format!("{from}{to}");

    // Try parsing with queen promotion — if it's legal, it's a promotion move
    let with_promo = format!("{base_uci}q");
    if let Ok(uci_move) = with_promo.parse::<UciMove>() {
        return uci_move.to_move(chess).is_ok();
    }

    false
}
