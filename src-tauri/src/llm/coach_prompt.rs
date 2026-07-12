//! Coaching prompt content: anti-hallucination system prompts and the
//! fact-grounded user prompt built from [`position_facts::MoveFacts`].
//!
//! All prompt *content* lives app-side by design (sensei-kit split);
//! `sensei_llm::format_chat` only supplies the Gemma chat format.

use crate::llm::PlayerLevel;
use crate::llm::position_facts::MoveFacts;

/// Maximum length of the built user prompt, in characters.
///
/// ~2600 chars is roughly 650 tokens — with the ~120-token system prompt it
/// keeps worst-case prefill comfortably inside the latency budget.
pub const MAX_USER_PROMPT_CHARS: usize = 2600;

/// Anti-hallucination system prompt tailored to the player's skill level.
pub fn system_prompt(level: PlayerLevel) -> &'static str {
    match level {
        PlayerLevel::Beginner => {
            "You are a friendly chess coach speaking to a beginner who just \
             played a move. You will be given verified facts about the position \
             computed by a chess engine and analysis heuristics.\n\
             \n\
             Rules you must follow:\n\
             - Base every claim ONLY on the facts listed. Never mention a piece, \
             square, threat, or plan that is not in the facts.\n\
             - Use simple language: piece names and square names only, no jargon.\n\
             - If a better move is given, name it and explain it using the given \
             lines.\n\
             - Refer to squares only if they appear in the facts.\n\
             - 1-3 short sentences. Output only the coaching text - no headers, \
             no lists, no notation dumps."
        }
        PlayerLevel::Intermediate => {
            "You are a chess coach speaking to a club-level player (~1000-1500) \
             who just played a move. You will be given verified facts about the \
             position computed by a chess engine and analysis heuristics.\n\
             \n\
             Rules you must follow:\n\
             - Base every claim ONLY on the facts listed. Never mention a piece, \
             square, threat, or plan that is not in the facts.\n\
             - If a better move is given, name it and explain it using the given \
             lines.\n\
             - Refer to squares only if they appear in the facts.\n\
             - 2-3 sentences. Output only the coaching text - no headers, no \
             lists, no notation dumps."
        }
        PlayerLevel::UpperIntermediate => {
            "You are a chess coach speaking to a 1500-2000 rated player who just \
             played a move. You will be given verified facts about the position \
             computed by a chess engine and analysis heuristics.\n\
             \n\
             Rules you must follow:\n\
             - Base every claim ONLY on the facts listed. Never mention a piece, \
             square, threat, or plan that is not in the facts.\n\
             - If a better move is given, name it and explain it using the given \
             lines.\n\
             - Refer to squares only if they appear in the facts.\n\
             - Use precise chess terminology and connect the facts to the \
             underlying strategic idea.\n\
             - 2-4 sentences. Output only the coaching text - no headers, no \
             lists, no notation dumps."
        }
    }
}

/// Build the complete Gemma-formatted coaching prompt.
pub fn build_coaching_prompt(level: PlayerLevel, facts: &MoveFacts) -> String {
    sensei_llm::format_chat(system_prompt(level), &build_user_prompt(facts))
}

/// Fact section, ordered by display position with a truncation rank.
///
/// `drop_rank` follows the plan's keep-priority: post-move tactics survive
/// longest, the piece list is dropped first.
struct Section {
    /// Higher rank is dropped earlier when over the char cap. Rank 0 is
    /// never dropped.
    drop_rank: u8,
    text: String,
}

/// Render the user prompt from verbalized facts, enforcing
/// [`MAX_USER_PROMPT_CHARS`] by dropping sections in truncation-priority
/// order: piece list → player context → activity → pawns → king safety →
/// eval/lines → pre-move tactics (post-move tactics and the header always
/// survive). Player context is nice-to-have color: it outlives only the
/// piece list, so truncation never sacrifices engine or position facts
/// to keep it.
pub fn build_user_prompt(facts: &MoveFacts) -> String {
    let mut sections = build_sections(facts);

    loop {
        let prompt = render(&sections);
        if prompt.chars().count() <= MAX_USER_PROMPT_CHARS {
            return prompt;
        }

        let Some(next_drop) = sections
            .iter()
            .map(|s| s.drop_rank)
            .filter(|r| *r > 0)
            .max()
        else {
            // Only undroppable sections remain — hard-truncate on a char
            // boundary as the last resort.
            return prompt.chars().take(MAX_USER_PROMPT_CHARS).collect();
        };
        sections.retain(|s| s.drop_rank != next_drop);
    }
}

fn build_sections(facts: &MoveFacts) -> Vec<Section> {
    let mut sections = Vec::new();

    // Header paragraph: classification + eval swing (swing shares the
    // eval/lines drop rank via its own section).
    sections.push(Section {
        drop_rank: 0,
        text: facts.header.clone(),
    });
    if let Some(ref swing) = facts.eval_swing {
        sections.push(Section {
            drop_rank: 3,
            text: swing.clone(),
        });
    }

    // Engine lines paragraph.
    let lines: Vec<&str> = [facts.best_line.as_deref(), facts.follow_up.as_deref()]
        .into_iter()
        .flatten()
        .collect();
    if !lines.is_empty() {
        sections.push(Section {
            drop_rank: 3,
            text: lines.join("\n"),
        });
    }

    // Post-move tactic diff — the highest-value grounding facts.
    if !facts.new_tactics.is_empty() {
        let title = if facts.is_positive {
            format!("Why your move works (new after {}):", facts.player_move_san)
        } else {
            format!(
                "What your move changed (new problems after {}):",
                facts.player_move_san
            )
        };
        sections.push(Section {
            drop_rank: 1,
            text: bulleted(&title, &facts.new_tactics),
        });
    }

    // Pre-move position facts, split by category so truncation can drop the
    // least important categories first.
    if !facts.pre_move_tactics.is_empty() {
        sections.push(Section {
            drop_rank: 2,
            text: bulleted(
                "Position facts (before your move):",
                &facts.pre_move_tactics,
            ),
        });
    }
    if !facts.king_safety.is_empty() {
        sections.push(Section {
            drop_rank: 4,
            text: bullets_only(&facts.king_safety),
        });
    }
    let mut pawn_and_material = facts.pawn_facts.clone();
    if let Some(ref material) = facts.material {
        pawn_and_material.push(material.clone());
    }
    if !pawn_and_material.is_empty() {
        sections.push(Section {
            drop_rank: 5,
            text: bullets_only(&pawn_and_material),
        });
    }
    if !facts.activity_facts.is_empty() {
        sections.push(Section {
            drop_rank: 6,
            text: bullets_only(&facts.activity_facts),
        });
    }

    // Rank-calibrated player context: dropped before any position or engine
    // fact (only the piece list goes first).
    if let Some(ref player) = facts.player_context {
        sections.push(Section {
            drop_rank: 7,
            text: player.clone(),
        });
    }

    if let Some(ref pieces) = facts.piece_list {
        sections.push(Section {
            drop_rank: 8,
            text: format!("Pieces - {pieces}"),
        });
    }

    sections
}

fn render(sections: &[Section]) -> String {
    let texts: Vec<&str> = sections.iter().map(|s| s.text.as_str()).collect();
    texts.join("\n\n")
}

fn bulleted(title: &str, items: &[String]) -> String {
    let mut text = title.to_string();
    for item in items {
        text.push_str("\n- ");
        text.push_str(item);
    }
    text
}

fn bullets_only(items: &[String]) -> String {
    items
        .iter()
        .map(|i| format!("- {i}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Build a prompt for generating a one-sentence post-game summary.
#[allow(clippy::too_many_arguments)]
pub fn build_game_summary_prompt(
    result: &str,
    outcome_type: &str,
    move_count: usize,
    accuracy_pct: f64,
    best_moves: usize,
    blunders: usize,
    mistakes: usize,
    inaccuracies: usize,
) -> String {
    let system = "You are a chess coach writing a brief, encouraging one-sentence summary \
                  of a student's game. Be specific about what went well or what to improve. \
                  Reference concrete aspects like tactical play, endgame technique, or \
                  opening preparation. Keep it under 30 words. \
                  Do not start with \"Great\" or \"Good\".";

    let context = format!(
        r#"{{"result":"{}","outcome":"{}","moves":{},"accuracy":{:.0},"bestMoves":{},"blunders":{},"mistakes":{},"inaccuracies":{}}}"#,
        result,
        outcome_type,
        move_count,
        accuracy_pct,
        best_moves,
        blunders,
        mistakes,
        inaccuracies
    );

    sensei_llm::format_chat(system, &context)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::position_facts::{EngineData, MoveInput, build_move_facts};
    use crate::models::engine::Score;

    fn hanging_queen_facts() -> MoveFacts {
        let fen = "rnbqkb1r/pppppppp/5n2/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2";
        let pos = crate::game::parse_fen(fen).unwrap();
        let ctx = crate::heuristics::analyze_position(&pos);
        let engine = EngineData {
            eval_before: Some(Score::cp(30)),
            eval_after: Some(Score::cp(-350)),
            best_move_san: Some("Nc3".to_string()),
            pv: vec!["b1c3".to_string(), "e7e5".to_string()],
            refutation_pv: vec!["f6h5".to_string()],
        };
        build_move_facts(
            &MoveInput {
                fen_before: fen,
                player_move_san: "Qh5",
                player_move_uci: Some("d1h5"),
                classification: "blunder",
            },
            Some(&ctx),
            Some(&engine),
        )
    }

    #[test]
    fn all_levels_share_anti_hallucination_rule_and_differ() {
        let prompts = [
            system_prompt(PlayerLevel::Beginner),
            system_prompt(PlayerLevel::Intermediate),
            system_prompt(PlayerLevel::UpperIntermediate),
        ];
        for p in prompts {
            assert!(p.contains("ONLY on the facts"), "{p}");
            assert!(p.contains("Never mention a piece"), "{p}");
        }
        assert_ne!(prompts[0], prompts[1]);
        assert_ne!(prompts[1], prompts[2]);
        assert_ne!(prompts[0], prompts[2]);
    }

    #[test]
    fn facts_appear_in_user_prompt() {
        let prompt = build_user_prompt(&hanging_queen_facts());

        assert!(
            prompt.contains("Move 2 as White: you played Qh5 - a blunder."),
            "{prompt}"
        );
        assert!(prompt.contains("losing (-3.5)"), "{prompt}");
        assert!(prompt.contains("Better was Nc3."), "{prompt}");
        assert!(prompt.contains("Best line: Nc3 e5."), "{prompt}");
        assert!(
            prompt.contains("After your move the opponent's punishment is: Nxh5."),
            "{prompt}"
        );
        assert!(
            prompt.contains("What your move changed (new problems after Qh5):"),
            "{prompt}"
        );
        assert!(prompt.contains("queen on h5"), "{prompt}");
        assert!(prompt.contains("Pieces - White: Ke1"), "{prompt}");
    }

    #[test]
    fn coaching_prompt_uses_gemma_chat_format() {
        let prompt = build_coaching_prompt(PlayerLevel::Intermediate, &hanging_queen_facts());
        assert!(prompt.starts_with("<|turn>system\n"));
        assert!(prompt.contains("<|turn>user\n"));
        assert!(prompt.ends_with("<|turn>model\n"));
    }

    #[test]
    fn over_cap_prompt_drops_piece_list_first() {
        let mut facts = hanging_queen_facts();
        // Inflate the pre-move tactics list to push the prompt over the cap.
        facts.pre_move_tactics = (0..40)
            .map(|i| format!("filler tactical fact number {i} that goes on and on and on"))
            .collect();

        let prompt = build_user_prompt(&facts);
        assert!(prompt.chars().count() <= MAX_USER_PROMPT_CHARS);
        assert!(
            !prompt.contains("Pieces -"),
            "piece list should drop first:\n{prompt}"
        );
        // The highest-priority facts survive.
        assert!(prompt.contains("What your move changed"), "{prompt}");
        assert!(prompt.contains(&facts.header), "{prompt}");
    }

    const PLAYER_LINE: &str = "Player context: rated about 1300 in tactical skill - this kind of oversight is a frequent miss at this level.";

    #[test]
    fn player_context_appears_when_set() {
        let mut facts = hanging_queen_facts();
        facts.player_context = Some(PLAYER_LINE.to_string());

        let prompt = build_user_prompt(&facts);
        assert!(prompt.contains(PLAYER_LINE), "{prompt}");
        // Under the cap nothing else is sacrificed for it.
        assert!(prompt.contains("Pieces -"), "{prompt}");
        assert!(prompt.contains("Best line:"), "{prompt}");
    }

    #[test]
    fn slight_overflow_drops_piece_list_before_player_context() {
        let mut facts = hanging_queen_facts();
        facts.player_context = Some(PLAYER_LINE.to_string());

        // Push the prompt just barely over the cap: dropping the piece list
        // alone brings it back under, so the player context must survive.
        let base_len = build_user_prompt(&facts).chars().count();
        assert!(base_len <= MAX_USER_PROMPT_CHARS, "fixture no longer fits");
        let filler_len = MAX_USER_PROMPT_CHARS - base_len + 10;
        facts.pre_move_tactics.push("x".repeat(filler_len));

        let prompt = build_user_prompt(&facts);
        assert!(prompt.chars().count() <= MAX_USER_PROMPT_CHARS);
        assert!(
            !prompt.contains("Pieces -"),
            "piece list should drop first:\n{prompt}"
        );
        assert!(
            prompt.contains(PLAYER_LINE),
            "player context outlives the piece list:\n{prompt}"
        );
    }

    #[test]
    fn heavy_overflow_drops_player_context_before_facts() {
        let mut facts = hanging_queen_facts();
        facts.player_context = Some(PLAYER_LINE.to_string());
        facts.pre_move_tactics = (0..40)
            .map(|i| format!("filler tactical fact number {i} that goes on and on and on"))
            .collect();

        let prompt = build_user_prompt(&facts);
        assert!(prompt.chars().count() <= MAX_USER_PROMPT_CHARS);
        assert!(
            !prompt.contains("Player context:"),
            "player context must be sacrificed before facts:\n{prompt}"
        );
        // The highest-priority facts survive.
        assert!(prompt.contains("What your move changed"), "{prompt}");
        assert!(prompt.contains(&facts.header), "{prompt}");
    }

    #[test]
    fn extreme_over_cap_prompt_hard_truncates() {
        let mut facts = hanging_queen_facts();
        facts.new_tactics = (0..200)
            .map(|i| format!("unremovable new-tactic filler line number {i} with extra padding"))
            .collect();

        let prompt = build_user_prompt(&facts);
        assert!(prompt.chars().count() <= MAX_USER_PROMPT_CHARS);
        assert!(prompt.contains(&facts.header));
    }

    #[test]
    fn under_cap_prompt_keeps_every_section() {
        let prompt = build_user_prompt(&hanging_queen_facts());
        assert!(prompt.chars().count() <= MAX_USER_PROMPT_CHARS);
        assert!(prompt.contains("Pieces -"));
        assert!(prompt.contains("Your king:"));
        assert!(prompt.contains("Material:"));
    }

    #[test]
    fn positive_move_uses_why_it_works_title() {
        let mut facts = hanging_queen_facts();
        facts.is_positive = true;
        let prompt = build_user_prompt(&facts);
        assert!(
            prompt.contains("Why your move works (new after Qh5):"),
            "{prompt}"
        );
    }

    #[test]
    fn game_summary_prompt_contains_stats() {
        let prompt = build_game_summary_prompt("1-0", "checkmate", 34, 87.5, 10, 1, 2, 3);
        assert!(prompt.contains("checkmate"));
        assert!(prompt.contains("\"moves\":34"));
        assert!(prompt.contains("\"accuracy\":88"));
        assert!(prompt.ends_with("<|turn>model\n"));
    }
}
