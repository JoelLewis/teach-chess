//! Pure verbalization of position and engine facts for LLM coaching prompts.
//!
//! Everything here is deterministic shakmaty/heuristics work — no model, no
//! I/O — so the whole module is unit-testable. The output feeds
//! `llm::coach_prompt`, which renders these facts into the user prompt.

use shakmaty::{
    Chess, Color, EnPassantMode, Move, Piece, Position, Role, fen::Fen, san::SanPlus, uci::UciMove,
};

use crate::models::engine::Score;
use crate::models::heuristics::{
    CoachingContext, MaterialImbalance, Side, SideKingSafety, SidePawnStructure,
};

/// Engine evaluation data for the move under review.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EngineData {
    pub eval_before: Option<Score>,
    pub eval_after: Option<Score>,
    /// Engine's best move from the pre-move position, in SAN.
    pub best_move_san: Option<String>,
    /// Best line from the pre-move position, as UCI moves.
    pub pv: Vec<String>,
    /// Refutation line: PV of the position *after* the played move, as UCI moves.
    pub refutation_pv: Vec<String>,
}

/// Identity of the move being coached.
#[derive(Debug, Clone)]
pub struct MoveInput<'a> {
    pub fen_before: &'a str,
    pub player_move_san: &'a str,
    /// Preferred over SAN for move resolution when present (unambiguous).
    pub player_move_uci: Option<&'a str>,
    /// Lowercase classification string, e.g. "blunder".
    pub classification: &'a str,
}

/// Verbalized, square-grounded facts about one played move, grouped by
/// section so the prompt builder can truncate in priority order.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MoveFacts {
    /// "Move 18 as White: you played Qxb7 - a blunder."
    pub header: String,
    pub player_move_san: String,
    /// Whether the move was strong (best/excellent) — flips section wording.
    pub is_positive: bool,
    /// "You went from slightly better (+0.8) to losing (-2.4), about a 3-pawn swing."
    pub eval_swing: Option<String>,
    /// "Better was Rad1. Best line: Rad1 Nc6 d5."
    pub best_line: Option<String>,
    /// "After your move the opponent's punishment is: Nc5 Qa6 Rb8."
    pub follow_up: Option<String>,
    /// New tactic descriptions that appeared after the played move.
    pub new_tactics: Vec<String>,
    /// Tactic descriptions already present before the move.
    pub pre_move_tactics: Vec<String>,
    pub king_safety: Vec<String>,
    pub pawn_facts: Vec<String>,
    pub activity_facts: Vec<String>,
    /// "Material: equal" or "Material: you are down about 3 pawns of material".
    pub material: Option<String>,
    /// "White: Kg1 Qd1 ..., pawns a2 b2 ...\nBlack: ..."
    pub piece_list: Option<String>,
    /// Rank-calibrated one-liner ("Player context: rated about 1300 in
    /// tactical skill - ..."). Not derived from the position — the caller
    /// fills it from the player's Glicko-2 category rating when available.
    pub player_context: Option<String>,
}

/// New tactical motifs created by a move, split by beneficiary.
///
/// `TacticalMotif.side` is uniformly the side that benefits from the motif
/// (the pinner, the forker, the attacker of a hanging piece).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TacticDiff {
    /// New motifs benefiting the mover ("why it works").
    pub for_mover: Vec<String>,
    /// New motifs benefiting the opponent ("new problems").
    pub against_mover: Vec<String>,
}

/// Build all verbalized facts for one played move.
///
/// `context` is the pre-move heuristic analysis; `engine` carries eval/PV
/// data. Both are optional — missing data just produces fewer sections.
pub fn build_move_facts(
    input: &MoveInput<'_>,
    context: Option<&CoachingContext>,
    engine: Option<&EngineData>,
) -> MoveFacts {
    let pos_before = crate::game::parse_fen(input.fen_before).ok();

    let mut facts = MoveFacts {
        header: header_line(input, pos_before.as_ref()),
        player_move_san: input.player_move_san.to_string(),
        is_positive: matches!(input.classification, "best" | "excellent"),
        ..MoveFacts::default()
    };

    let Some(pos_before) = pos_before else {
        return facts;
    };
    let mover = pos_before.turn();

    if let Some(engine) = engine {
        if let (Some(before), Some(after)) = (&engine.eval_before, &engine.eval_after) {
            facts.eval_swing = Some(verbalize_eval_swing(before, after, mover.is_white()));
        }
        facts.best_line = best_line_sentence(input, engine);
    }

    // Post-move analysis: apply the move, then diff tactics and build the
    // refutation/continuation line from the post-move position.
    if let Some(played) = resolve_move(&pos_before, input.player_move_uci, input.player_move_san) {
        let mut pos_after = pos_before.clone();
        pos_after.play_unchecked(&played);

        let diff = tactic_diff(&pos_before, &pos_after, mover);
        facts.new_tactics = if facts.is_positive {
            diff.for_mover
        } else {
            diff.against_mover
        };

        if let Some(engine) = engine
            && !engine.refutation_pv.is_empty()
        {
            let fen_after = Fen::from_position(pos_after, EnPassantMode::Legal).to_string();
            let line = uci_line_to_san(&fen_after, &engine.refutation_pv, 3);
            if !line.is_empty() {
                let label = if facts.is_positive {
                    "After your move the likely continuation is"
                } else {
                    "After your move the opponent's punishment is"
                };
                facts.follow_up = Some(format!("{label}: {}.", line.join(" ")));
            }
        }
    }

    if let Some(ctx) = context {
        let mover_side = Side::from(mover);
        facts.pre_move_tactics = ctx.tactics.iter().map(|t| t.description.clone()).collect();
        facts.king_safety = king_safety_facts(ctx, mover_side);
        facts.pawn_facts = pawn_facts(ctx, mover_side);
        facts.activity_facts = activity_facts(ctx, mover_side);
        facts.material = Some(material_fact(ctx, mover.is_white()));
    }

    facts.piece_list = piece_list(input.fen_before);
    facts
}

/// Describe the eval change across a move in plain language, from the
/// student's (mover's) perspective.
pub fn verbalize_eval_swing(before: &Score, after: &Score, mover_is_white: bool) -> String {
    let before = pov(before, mover_is_white);
    let after = pov(after, mover_is_white);
    let (phrase_before, value_before) = describe_eval(&before);
    let (phrase_after, value_after) = describe_eval(&after);

    if let (Score::Cp { value: b }, Score::Cp { value: a }) = (&before, &after) {
        let delta = (a - b).abs();
        if phrase_before == phrase_after && delta < 30 {
            return format!("You stayed {phrase_after} ({value_after}).");
        }
        if delta >= 100 {
            let swing = (f64::from(delta) / 100.0).round().max(1.0) as i64;
            return format!(
                "You went from {phrase_before} ({value_before}) to {phrase_after} ({value_after}), about a {swing}-pawn swing."
            );
        }
    }

    format!("You went from {phrase_before} ({value_before}) to {phrase_after} ({value_after}).")
}

/// Convert a UCI move sequence to SAN, starting from `fen`, up to `max_plies`.
///
/// Stops at the first unparseable or illegal move.
pub fn uci_line_to_san(fen: &str, uci_moves: &[String], max_plies: usize) -> Vec<String> {
    let Ok(mut pos) = crate::game::parse_fen(fen) else {
        return Vec::new();
    };

    let mut sans = Vec::new();
    for uci in uci_moves.iter().take(max_plies) {
        let Ok(uci_move) = uci.parse::<UciMove>() else {
            break;
        };
        let Ok(legal_move) = uci_move.to_move(&pos) else {
            break;
        };
        sans.push(SanPlus::from_move(pos.clone(), &legal_move).to_string());
        pos.play_unchecked(&legal_move);
    }
    sans
}

/// Apply `player_move_uci` to `fen_before` and report the tactical motifs
/// that are new in the resulting position, split by beneficiary.
///
/// Returns an empty diff when the FEN or move fails to parse.
pub fn post_move_tactic_diff(fen_before: &str, player_move_uci: &str) -> TacticDiff {
    let Ok(pos_before) = crate::game::parse_fen(fen_before) else {
        return TacticDiff::default();
    };
    let mover = pos_before.turn();
    let Some(played) = resolve_move(&pos_before, Some(player_move_uci), "") else {
        return TacticDiff::default();
    };
    let mut pos_after = pos_before.clone();
    pos_after.play_unchecked(&played);
    tactic_diff(&pos_before, &pos_after, mover)
}

/// Compact piece list for both sides, as an existence reference for the model.
///
/// Example: `White: Ke1 Qd1 Ra1 Rh1 Ng1 Bc1, pawns a2 b2.\nBlack: Ke8, pawns a7.`
pub fn piece_list(fen: &str) -> Option<String> {
    let pos = crate::game::parse_fen(fen).ok()?;
    let board = pos.board();
    Some(format!(
        "White: {}.\nBlack: {}.",
        side_piece_list(board, Color::White),
        side_piece_list(board, Color::Black)
    ))
}

// ─── Internal helpers ─────────────────────────────────────────────

fn header_line(input: &MoveInput<'_>, pos_before: Option<&Chess>) -> String {
    let described = describe_classification(input.classification);
    match pos_before {
        Some(pos) => format!(
            "Move {} as {}: you played {} - {}.",
            pos.fullmoves(),
            if pos.turn().is_white() {
                "White"
            } else {
                "Black"
            },
            input.player_move_san,
            described,
        ),
        None => format!("You played {} - {}.", input.player_move_san, described),
    }
}

fn describe_classification(classification: &str) -> &'static str {
    match classification {
        "best" => "the engine's top move",
        "excellent" => "an excellent move",
        "good" => "a good move",
        "inaccuracy" => "an inaccuracy",
        "mistake" => "a mistake",
        "blunder" => "a blunder",
        _ => "a move",
    }
}

fn best_line_sentence(input: &MoveInput<'_>, engine: &EngineData) -> Option<String> {
    let line = uci_line_to_san(input.fen_before, &engine.pv, 3);
    let best_san = engine
        .best_move_san
        .as_deref()
        .or(line.first().map(String::as_str));

    let mut sentence = String::new();
    if let Some(best) = best_san
        && best != input.player_move_san
        && !matches!(input.classification, "best" | "excellent")
    {
        sentence.push_str(&format!("Better was {best}."));
    }
    if !line.is_empty() {
        if !sentence.is_empty() {
            sentence.push(' ');
        }
        sentence.push_str(&format!("Best line: {}.", line.join(" ")));
    }
    (!sentence.is_empty()).then_some(sentence)
}

/// Resolve the played move, preferring UCI (unambiguous) over SAN.
fn resolve_move(pos: &Chess, uci: Option<&str>, san: &str) -> Option<Move> {
    if let Some(uci) = uci
        && let Ok(uci_move) = uci.parse::<UciMove>()
        && let Ok(legal_move) = uci_move.to_move(pos)
    {
        return Some(legal_move);
    }
    let san_plus: SanPlus = san.parse().ok()?;
    san_plus.san.to_move(pos).ok()
}

/// Motif descriptions present after the move but not before, split by which
/// side benefits (`TacticalMotif.side` is always the beneficiary).
fn tactic_diff(pos_before: &Chess, pos_after: &Chess, mover: Color) -> TacticDiff {
    let before: Vec<String> = crate::heuristics::analyze_position(pos_before)
        .tactics
        .into_iter()
        .map(|t| t.description)
        .collect();

    let mut diff = TacticDiff::default();
    for motif in crate::heuristics::analyze_position(pos_after).tactics {
        if before.contains(&motif.description) {
            continue;
        }
        if motif.side == Side::from(mover) {
            diff.for_mover.push(motif.description);
        } else {
            diff.against_mover.push(motif.description);
        }
    }
    diff
}

fn pov(score: &Score, mover_is_white: bool) -> Score {
    if mover_is_white {
        score.clone()
    } else {
        match score {
            Score::Cp { value } => Score::Cp { value: -value },
            Score::Mate { moves } => Score::Mate { moves: -moves },
        }
    }
}

/// Phrase + display value for a mover-perspective score.
fn describe_eval(score: &Score) -> (String, String) {
    match score {
        Score::Cp { value } => {
            let pawns = format!("{:+.1}", f64::from(*value) / 100.0);
            let phrase = match value.abs() {
                0..=29 => "about equal",
                30..=100 => {
                    if *value > 0 {
                        "slightly better"
                    } else {
                        "slightly worse"
                    }
                }
                101..=300 => {
                    if *value > 0 {
                        "clearly better"
                    } else {
                        "clearly worse"
                    }
                }
                _ => {
                    if *value > 0 {
                        "winning"
                    } else {
                        "losing"
                    }
                }
            };
            (phrase.to_string(), pawns)
        }
        Score::Mate { moves } => {
            let phrase = if *moves > 0 {
                "winning with forced mate"
            } else {
                "losing to forced mate"
            };
            (phrase.to_string(), format!("mate in {}", moves.abs()))
        }
    }
}

fn king_safety_facts(ctx: &CoachingContext, mover: Side) -> Vec<String> {
    let (yours, theirs) = if mover == Side::White {
        (&ctx.king_safety.white, &ctx.king_safety.black)
    } else {
        (&ctx.king_safety.black, &ctx.king_safety.white)
    };
    vec![
        verbalize_king("Your king", yours),
        verbalize_king("Opponent king", theirs),
    ]
}

fn verbalize_king(label: &str, ks: &SideKingSafety) -> String {
    let mut parts = Vec::new();

    if ks.has_castled {
        let wing = match ks.king_square.chars().next() {
            Some('a'..='c') => " queenside",
            Some('f'..='h') => " kingside",
            _ => "",
        };
        parts.push(format!("castled{wing}"));
    } else {
        parts.push(format!("on {}, not castled", ks.king_square));
    }

    if ks.pawn_shield_max > 0 {
        let state = if ks.pawn_shield_count >= ks.pawn_shield_max {
            "intact"
        } else {
            "damaged"
        };
        parts.push(format!(
            "pawn shield {state} ({}/{})",
            ks.pawn_shield_count, ks.pawn_shield_max
        ));
    }
    if ks.open_files_near_king > 0 {
        parts.push(format!(
            "{} open file{} nearby",
            ks.open_files_near_king,
            plural(ks.open_files_near_king)
        ));
    }
    if ks.king_zone_attacks > 0 {
        parts.push(format!(
            "{} enemy attack{} near the king",
            ks.king_zone_attacks,
            plural(ks.king_zone_attacks)
        ));
    }

    format!("{label}: {}", parts.join(", "))
}

fn pawn_facts(ctx: &CoachingContext, mover: Side) -> Vec<String> {
    let mut facts = Vec::new();

    if !ctx.pawns.open_files.is_empty() {
        facts.push(format!(
            "Open file{}: {}",
            plural(ctx.pawns.open_files.len() as u32),
            ctx.pawns.open_files.join(", ")
        ));
    }

    let (yours, theirs) = if mover == Side::White {
        (&ctx.pawns.white, &ctx.pawns.black)
    } else {
        (&ctx.pawns.black, &ctx.pawns.white)
    };
    facts.extend(side_pawn_facts("Your", yours));
    facts.extend(side_pawn_facts("Opponent", theirs));
    facts
}

fn side_pawn_facts(owner: &str, pawns: &SidePawnStructure) -> Vec<String> {
    let mut facts = Vec::new();
    for (kind, squares) in [
        ("passed", &pawns.passed),
        ("isolated", &pawns.isolated),
        ("doubled", &pawns.doubled),
        ("backward", &pawns.backward),
    ] {
        if !squares.is_empty() {
            facts.push(format!(
                "{owner} {kind} pawn{} on {}",
                plural(squares.len() as u32),
                squares.join(", ")
            ));
        }
    }
    facts
}

fn activity_facts(ctx: &CoachingContext, mover: Side) -> Vec<String> {
    let (yours, theirs) = if mover == Side::White {
        (&ctx.activity.white, &ctx.activity.black)
    } else {
        (&ctx.activity.black, &ctx.activity.white)
    };

    let mut facts = Vec::new();
    for (owner, verb, activity) in [("Your", "is", yours), ("Opponent's", "is", theirs)] {
        if activity.rook_on_seventh {
            facts.push(format!("{owner} rook {verb} on the 7th rank"));
        }
        if activity.rook_on_open_file {
            facts.push(format!("{owner} rook {verb} on an open file"));
        }
        for piece in &activity.pieces {
            if piece.is_on_rim && piece.piece == "knight" {
                facts.push(format!(
                    "{owner} knight on {} {verb} on the rim",
                    piece.square
                ));
            }
        }
    }
    facts
}

fn material_fact(ctx: &CoachingContext, mover_is_white: bool) -> String {
    let pov_cp = if mover_is_white {
        ctx.material.balance_cp
    } else {
        -ctx.material.balance_cp
    };

    let mut fact = if pov_cp.abs() < 50 {
        "Material: equal".to_string()
    } else {
        let pawns = (f64::from(pov_cp.abs()) / 100.0).round().max(1.0) as i64;
        format!(
            "Material: you are {} about {pawns} pawn{} of material",
            if pov_cp > 0 { "up" } else { "down" },
            plural(pawns as u32)
        )
    };

    let mover_side = if mover_is_white {
        Side::White
    } else {
        Side::Black
    };
    for imbalance in &ctx.material.imbalances {
        let (side, text) = match imbalance {
            MaterialImbalance::BishopPair { side } => (side, "the bishop pair"),
            MaterialImbalance::ExchangeUp { side } => (side, "the exchange (rook vs minor)"),
            MaterialImbalance::ExchangeDown { .. } => continue,
            MaterialImbalance::QueenVsPieces { side } => (side, "a queen against pieces"),
        };
        let owner = if *side == mover_side {
            "you have"
        } else {
            "opponent has"
        };
        fact.push_str(&format!("; {owner} {text}"));
    }
    fact
}

fn side_piece_list(board: &shakmaty::Board, color: Color) -> String {
    const ROLES: [(Role, char); 5] = [
        (Role::King, 'K'),
        (Role::Queen, 'Q'),
        (Role::Rook, 'R'),
        (Role::Knight, 'N'),
        (Role::Bishop, 'B'),
    ];

    let mut parts = Vec::new();
    for (role, letter) in ROLES {
        for square in board.by_piece(Piece { color, role }) {
            parts.push(format!("{letter}{square}"));
        }
    }

    let pawns: Vec<String> = board
        .by_piece(Piece {
            color,
            role: Role::Pawn,
        })
        .into_iter()
        .map(|sq| sq.to_string())
        .collect();

    if pawns.is_empty() {
        parts.join(" ")
    } else {
        format!("{}, pawns {}", parts.join(" "), pawns.join(" "))
    }
}

fn plural(n: u32) -> &'static str {
    if n == 1 { "" } else { "s" }
}

#[cfg(test)]
mod tests {
    use super::*;

    const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    // ── verbalize_eval_swing bucket table ──

    #[test]
    fn eval_swing_blunder_matches_plan_example() {
        // The plan's prose example says "losing (-2.4)" but its normative
        // bucket table puts 1.0-3.0 in "clearly" — the table wins.
        let text = verbalize_eval_swing(&Score::cp(80), &Score::cp(-240), true);
        assert_eq!(
            text,
            "You went from slightly better (+0.8) to clearly worse (-2.4), about a 3-pawn swing."
        );
    }

    #[test]
    fn eval_swing_buckets() {
        let cases: &[(i32, &str)] = &[
            (0, "about equal"),
            (29, "about equal"),
            (-29, "about equal"),
            (30, "slightly better"),
            (-100, "slightly worse"),
            (101, "clearly better"),
            (-300, "clearly worse"),
            (301, "winning"),
            (-500, "losing"),
        ];
        for (cp, phrase) in cases {
            let text = verbalize_eval_swing(&Score::cp(1000), &Score::cp(*cp), true);
            assert!(
                text.contains(phrase),
                "cp {cp}: expected {phrase:?} in {text:?}"
            );
        }
    }

    #[test]
    fn eval_swing_is_from_movers_perspective() {
        // White-positive +240 is "clearly better" for White, "clearly worse" for Black
        let white = verbalize_eval_swing(&Score::cp(0), &Score::cp(240), true);
        assert!(white.contains("clearly better (+2.4)"), "{white}");
        let black = verbalize_eval_swing(&Score::cp(0), &Score::cp(240), false);
        assert!(black.contains("clearly worse (-2.4)"), "{black}");
    }

    #[test]
    fn eval_swing_stable_eval_says_stayed() {
        let text = verbalize_eval_swing(&Score::cp(10), &Score::cp(20), true);
        assert_eq!(text, "You stayed about equal (+0.2).");
    }

    #[test]
    fn eval_swing_handles_mate_scores() {
        let text = verbalize_eval_swing(&Score::cp(150), &Score::mate(-2), true);
        assert_eq!(
            text,
            "You went from clearly better (+1.5) to losing to forced mate (mate in 2)."
        );

        // Black blundering into a mate: white-positive mate flips to losing for black
        let text = verbalize_eval_swing(&Score::cp(-30), &Score::mate(3), false);
        assert!(text.contains("losing to forced mate (mate in 3)"), "{text}");
    }

    #[test]
    fn eval_swing_small_change_without_bucket_change_has_no_swing_clause() {
        let text = verbalize_eval_swing(&Score::cp(40), &Score::cp(-40), true);
        assert_eq!(
            text,
            "You went from slightly better (+0.4) to slightly worse (-0.4)."
        );
    }

    // ── uci_line_to_san ──

    #[test]
    fn uci_line_converts_and_caps_plies() {
        let line: Vec<String> = ["e2e4", "e7e5", "g1f3", "b8c6"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(
            uci_line_to_san(START_FEN, &line, 3),
            vec!["e4", "e5", "Nf3"]
        );
    }

    #[test]
    fn uci_line_stops_at_illegal_move() {
        let line: Vec<String> = ["e2e4", "e2e4"].iter().map(|s| s.to_string()).collect();
        assert_eq!(uci_line_to_san(START_FEN, &line, 3), vec!["e4"]);
    }

    #[test]
    fn uci_line_handles_bad_fen_and_empty_line() {
        assert!(uci_line_to_san("garbage", &["e2e4".to_string()], 3).is_empty());
        assert!(uci_line_to_san(START_FEN, &[], 3).is_empty());
    }

    // ── post_move_tactic_diff ──

    #[test]
    fn tactic_diff_reports_hanging_queen_blunder() {
        // After 1. e4 Nf6, White plays 2. Qh5?? — the queen on h5 is attacked
        // by the f6 knight and undefended.
        let fen = "rnbqkb1r/pppppppp/5n2/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2";
        let diff = post_move_tactic_diff(fen, "d1h5");
        assert!(
            diff.against_mover
                .iter()
                .any(|d| d.contains("queen on h5") && d.contains("undefended")),
            "expected hanging queen on h5, got {diff:?}"
        );
    }

    #[test]
    fn tactic_diff_reports_fork_for_mover() {
        // White knight jumps g5-f7, forking the black king on d8 and rook on h8.
        let fen = "3k3r/8/8/6N1/8/8/8/4K3 w - - 0 1";
        let diff = post_move_tactic_diff(fen, "g5f7");
        assert!(
            diff.for_mover
                .iter()
                .any(|d| d.contains("knight on f7") && d.contains("forks")),
            "expected knight fork on f7, got {diff:?}"
        );
    }

    #[test]
    fn tactic_diff_ignores_preexisting_motifs() {
        // Black knight on d5 already hangs; an unrelated white king move
        // must not report it as new.
        let fen = "4k3/8/8/3n4/4P3/8/8/4K3 w - - 0 1";
        let diff = post_move_tactic_diff(fen, "e1e2");
        assert!(
            diff.for_mover.iter().all(|d| !d.contains("knight on d5")),
            "pre-existing hanging knight reported as new: {diff:?}"
        );
    }

    #[test]
    fn tactic_diff_on_garbage_input_is_empty() {
        assert_eq!(
            post_move_tactic_diff("not a fen", "e2e4"),
            TacticDiff::default()
        );
        assert_eq!(
            post_move_tactic_diff(START_FEN, "zz99"),
            TacticDiff::default()
        );
    }

    // ── piece_list ──

    #[test]
    fn piece_list_formats_both_sides() {
        let list = piece_list("6k1/5ppp/8/8/8/8/8/4R1K1 w - - 0 1").unwrap();
        assert_eq!(list, "White: Kg1 Re1.\nBlack: Kg8, pawns f7 g7 h7.");
    }

    #[test]
    fn piece_list_orders_roles_kqrnb() {
        let list = piece_list(START_FEN).unwrap();
        assert!(
            list.starts_with(
                "White: Ke1 Qd1 Ra1 Rh1 Nb1 Ng1 Bc1 Bf1, pawns a2 b2 c2 d2 e2 f2 g2 h2."
            )
        );
    }

    #[test]
    fn piece_list_rejects_bad_fen() {
        assert!(piece_list("garbage").is_none());
    }

    // ── build_move_facts ──

    fn context_for(fen: &str) -> CoachingContext {
        let pos = crate::game::parse_fen(fen).unwrap();
        crate::heuristics::analyze_position(&pos)
    }

    #[test]
    fn move_facts_header_uses_fen_move_number_and_color() {
        let fen = "rnbqkb1r/pppppppp/5n2/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2";
        let input = MoveInput {
            fen_before: fen,
            player_move_san: "Qh5",
            player_move_uci: Some("d1h5"),
            classification: "blunder",
        };
        let facts = build_move_facts(&input, None, None);
        assert_eq!(facts.header, "Move 2 as White: you played Qh5 - a blunder.");
        assert!(!facts.is_positive);
    }

    #[test]
    fn move_facts_full_assembly_for_blunder() {
        let fen = "rnbqkb1r/pppppppp/5n2/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2";
        let ctx = context_for(fen);
        let engine = EngineData {
            eval_before: Some(Score::cp(30)),
            eval_after: Some(Score::cp(-350)),
            best_move_san: Some("Nc3".to_string()),
            pv: vec!["b1c3".to_string(), "e7e5".to_string()],
            refutation_pv: vec!["f6h5".to_string()],
        };
        let input = MoveInput {
            fen_before: fen,
            player_move_san: "Qh5",
            player_move_uci: Some("d1h5"),
            classification: "blunder",
        };
        let facts = build_move_facts(&input, Some(&ctx), Some(&engine));

        let swing = facts.eval_swing.unwrap();
        assert!(swing.contains("losing (-3.5)"), "{swing}");
        let best = facts.best_line.unwrap();
        assert!(best.contains("Better was Nc3."), "{best}");
        assert!(best.contains("Best line: Nc3 e5."), "{best}");
        let follow = facts.follow_up.unwrap();
        assert_eq!(
            follow,
            "After your move the opponent's punishment is: Nxh5."
        );
        assert!(
            facts.new_tactics.iter().any(|t| t.contains("queen on h5")),
            "{:?}",
            facts.new_tactics
        );
        assert!(facts.material.unwrap().contains("Material: equal"));
        assert!(facts.piece_list.unwrap().contains("White: Ke1"));
        assert_eq!(facts.king_safety.len(), 2);
        assert!(facts.king_safety[0].starts_with("Your king: on e1, not castled"));
    }

    #[test]
    fn move_facts_positive_move_skips_better_was() {
        let fen = "3k3r/8/8/6N1/8/8/8/4K3 w - - 0 1";
        let engine = EngineData {
            eval_before: Some(Score::cp(200)),
            eval_after: Some(Score::cp(600)),
            best_move_san: Some("Nf7+".to_string()),
            pv: vec!["g5f7".to_string()],
            refutation_pv: vec!["d8e8".to_string(), "f7h8".to_string()],
        };
        let input = MoveInput {
            fen_before: fen,
            player_move_san: "Nf7+",
            player_move_uci: Some("g5f7"),
            classification: "best",
        };
        let facts = build_move_facts(&input, None, Some(&engine));

        assert!(facts.is_positive);
        let best = facts.best_line.unwrap();
        assert!(!best.contains("Better was"), "{best}");
        let follow = facts.follow_up.unwrap();
        assert!(
            follow.starts_with("After your move the likely continuation is:"),
            "{follow}"
        );
        assert!(
            facts.new_tactics.iter().any(|t| t.contains("forks")),
            "{:?}",
            facts.new_tactics
        );
    }

    #[test]
    fn move_facts_survives_bad_fen() {
        let input = MoveInput {
            fen_before: "garbage",
            player_move_san: "e4",
            player_move_uci: None,
            classification: "good",
        };
        let facts = build_move_facts(&input, None, None);
        assert_eq!(facts.header, "You played e4 - a good move.");
        assert!(facts.new_tactics.is_empty());
        assert!(facts.piece_list.is_none());
    }

    #[test]
    fn move_facts_resolves_move_from_san_when_uci_missing() {
        let fen = "rnbqkb1r/pppppppp/5n2/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2";
        let input = MoveInput {
            fen_before: fen,
            player_move_san: "Qh5",
            player_move_uci: None,
            classification: "blunder",
        };
        let facts = build_move_facts(&input, None, None);
        assert!(
            facts.new_tactics.iter().any(|t| t.contains("queen on h5")),
            "{:?}",
            facts.new_tactics
        );
    }
}
