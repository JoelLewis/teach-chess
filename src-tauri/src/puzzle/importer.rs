use std::io::{BufRead, BufReader};
use std::path::Path;

use tracing::info;

use crate::db::connection::Database;
use crate::error::{AppError, PuzzleError};
use crate::models::puzzle::{Puzzle, PuzzleCategory};

/// Import puzzles from a Lichess-format CSV file.
///
/// CSV columns: PuzzleId, FEN, Moves, Rating, RatingDeviation, Popularity, NbPlays, Themes, GameUrl, OpeningTags
///
/// Filters by rating range and minimum popularity.
/// Returns the number of puzzles imported.
pub fn import_lichess_csv(
    path: &Path,
    db: &Database,
    rating_range: (u32, u32),
    min_popularity: i32,
) -> Result<usize, AppError> {
    let file = std::fs::File::open(path)
        .map_err(|e| PuzzleError::ImportError(format!("Cannot open file: {e}")))?;
    let reader = BufReader::new(file);

    let mut puzzles = Vec::new();
    let mut line_count = 0;

    for line in reader.lines() {
        let line = line.map_err(|e| {
            PuzzleError::ImportError(format!("Read error at line {line_count}: {e}"))
        })?;
        line_count += 1;

        // Skip header line
        if line_count == 1 && line.starts_with("PuzzleId") {
            continue;
        }

        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        if let Some(puzzle) = parse_lichess_line(&line, rating_range, min_popularity) {
            puzzles.push(puzzle);
        }

        // Batch insert every 5000 puzzles to manage memory
        if puzzles.len() >= 5000 {
            db.import_puzzles(&puzzles)?;
            puzzles.clear();
        }
    }

    // Insert remaining
    let remaining = puzzles.len();
    if !puzzles.is_empty() {
        db.import_puzzles(&puzzles)?;
    }

    let total = line_count - 1; // subtract header
    info!("Processed {total} lines, imported puzzles (last batch: {remaining})");

    let count = db.get_puzzle_count()?;
    Ok(count as usize)
}

fn parse_lichess_line(line: &str, rating_range: (u32, u32), min_popularity: i32) -> Option<Puzzle> {
    // CSV parsing: handle commas within fields (Lichess CSV uses simple comma separation, no quoting)
    let fields: Vec<&str> = line.split(',').collect();
    if fields.len() < 8 {
        return None;
    }

    let puzzle_id = fields[0].trim();
    let fen = fields[1].trim();
    let moves = fields[2].trim();
    let rating: u32 = fields[3].trim().parse().ok()?;
    // fields[4] = RatingDeviation
    let popularity: i32 = fields[5].trim().parse().unwrap_or(0);
    // fields[6] = NbPlays
    let themes = fields[7].trim();

    // Apply filters
    if rating < rating_range.0 || rating > rating_range.1 {
        return None;
    }
    if popularity < min_popularity {
        return None;
    }

    // Validate minimum move count (need at least setup + one player move)
    let move_count = moves.split_whitespace().count();
    if move_count < 2 {
        return None;
    }

    let hints_json = generate_hints_from_themes(themes, moves);
    let category = categorize_themes(themes);

    Some(Puzzle {
        id: format!("lichess_{puzzle_id}"),
        fen: fen.to_string(),
        solution_moves: moves.to_string(),
        themes: themes.replace(' ', ","),
        category,
        difficulty: rating,
        source_id: Some(puzzle_id.to_string()),
        hints_json,
        explanation: String::new(),
    })
}

fn generate_hints_from_themes(themes: &str, moves: &str) -> String {
    let theme_list: Vec<&str> = themes.split_whitespace().collect();
    let move_list: Vec<&str> = moves.split_whitespace().collect();

    // Tier 1: General theme hint
    let tier1 = if theme_list.is_empty() {
        "Look for a tactical opportunity.".to_string()
    } else {
        let primary = theme_list[0];
        match primary {
            "fork" => "Look for a way to attack two pieces at once.".to_string(),
            "pin" => {
                "Look for a pin — a piece that can't move without exposing something behind it."
                    .to_string()
            }
            "skewer" => {
                "Look for a skewer — attack through a valuable piece to one behind it.".to_string()
            }
            "mate" | "mateIn1" => "There's a checkmate in one move!".to_string(),
            "mateIn2" => "You can force checkmate in two moves.".to_string(),
            "mateIn3" | "mateIn4" | "mateIn5" => {
                "Look for a forcing sequence leading to checkmate.".to_string()
            }
            "hangingPiece" => "One of the opponent's pieces is undefended.".to_string(),
            "trappedPiece" => "One of the opponent's pieces has no escape.".to_string(),
            "discoveredAttack" => {
                "Look for a discovered attack — moving one piece reveals an attack by another."
                    .to_string()
            }
            "sacrifice" => {
                "Sometimes you need to give up material to gain an advantage.".to_string()
            }
            "deflection" => "Can you deflect a defender away from a key square?".to_string(),
            "decoy" | "attraction" => "Can you lure a piece to a vulnerable square?".to_string(),
            "backRankMate" => "The opponent's king is vulnerable on the back rank.".to_string(),
            "smotheredMate" => {
                "The king is surrounded by its own pieces — look for a knight check.".to_string()
            }
            "promotion" => "A pawn is close to promotion — that's the key.".to_string(),
            "endgame" => "Apply endgame technique to convert your advantage.".to_string(),
            "quietMove" => {
                "The best move isn't a check or capture — look for a quiet but strong move."
                    .to_string()
            }
            "intermezzo" | "zwischenzug" => {
                "Before making the obvious move, is there a strong in-between move?".to_string()
            }
            _ => format!("This puzzle involves: {}.", themes.replace(' ', ", ")),
        }
    };

    // Tier 2: Piece hint from the first player move
    let tier2 = if move_list.len() >= 2 {
        let player_move = move_list[1]; // index 1 = first player move
        let from_square = &player_move[..2];
        // Hint about the starting square file
        let file = from_square.chars().next().unwrap_or('?');
        let rank = from_square.chars().nth(1).unwrap_or('?');
        format!("Consider a piece on the {file}-file (rank {rank}).")
    } else {
        "Look carefully at the position.".to_string()
    };

    // Tier 3: Target square hint
    let tier3 = if move_list.len() >= 2 {
        let player_move = move_list[1];
        let to_square = &player_move[2..4];
        format!("The key move targets the {to_square} square.")
    } else {
        "Calculate all forcing moves.".to_string()
    };

    serde_json::to_string(&vec![tier1, tier2, tier3]).unwrap_or_else(|_| "[]".to_string())
}

fn categorize_themes(themes: &str) -> PuzzleCategory {
    let lower = themes.to_lowercase();
    if lower.contains("endgame") {
        PuzzleCategory::Endgame
    } else if lower.contains("opening") {
        PuzzleCategory::Opening
    } else {
        PuzzleCategory::Tactical
    }
}

/// Import the bundled starter puzzles if the puzzle table is empty.
pub fn import_starter_puzzles_if_empty(db: &Database) -> Result<(), AppError> {
    let count = db.get_puzzle_count()?;
    if count > 0 {
        info!("Puzzle table already has {count} puzzles, skipping starter import");
        return Ok(());
    }

    info!("Puzzle table empty, importing bundled starter puzzles...");
    let csv_data = include_str!("../../data/puzzles_starter.csv");

    let mut puzzles = Vec::new();
    for (i, line) in csv_data.lines().enumerate() {
        if i == 0 && line.starts_with("PuzzleId") {
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }
        if let Some(puzzle) = parse_lichess_line(line, (0, 9999), -100) {
            puzzles.push(puzzle);
        }
    }

    let count = puzzles.len();
    db.import_puzzles(&puzzles)?;
    info!("Imported {count} starter puzzles");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_lichess_line() {
        let line = "00008,r6k/pp2r2p/4Rp1Q/3p4/8/1N1P2R1/PqP2bPP/7K b - - 0 24,f2g3 e6e7 b2b1 b3c1 b1c1 e7e8,1852,76,97,43667,crushing hangingPiece long,https://lichess.org/787zsVup/black#48,";
        let puzzle = parse_lichess_line(line, (1000, 2000), 0).unwrap();

        assert_eq!(puzzle.id, "lichess_00008");
        assert_eq!(puzzle.difficulty, 1852);
        assert!(puzzle.themes.contains("crushing"));
        assert!(puzzle.themes.contains("hangingPiece"));
        assert_eq!(puzzle.category, PuzzleCategory::Tactical);
    }

    #[test]
    fn filter_by_rating() {
        let line = "00001,fen,e2e4 e7e5,500,10,50,100,fork,,";
        assert!(parse_lichess_line(line, (1000, 2000), 0).is_none());
        assert!(parse_lichess_line(line, (400, 600), 0).is_some());
    }

    #[test]
    fn filter_by_popularity() {
        let line = "00001,fen,e2e4 e7e5,1500,10,-5,100,fork,,";
        assert!(parse_lichess_line(line, (1000, 2000), 0).is_none());
        assert!(parse_lichess_line(line, (1000, 2000), -10).is_some());
    }

    #[test]
    fn hints_generated_from_themes() {
        let hints = generate_hints_from_themes("fork pin", "e2e4 d7d5");
        let parsed: Vec<String> = serde_json::from_str(&hints).unwrap();
        assert_eq!(parsed.len(), 3);
        assert!(parsed[0].contains("two pieces"));
        assert!(parsed[1].contains("d-file"));
        assert!(parsed[2].contains("d5"));
    }

    #[test]
    fn categorize_endgame() {
        assert_eq!(
            categorize_themes("endgame pawnEndgame"),
            PuzzleCategory::Endgame
        );
    }

    #[test]
    fn categorize_tactical_default() {
        assert_eq!(categorize_themes("fork pin"), PuzzleCategory::Tactical);
    }
}
