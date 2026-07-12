use crate::types::PlayerLevel;

/// System prompt tailored to the player's skill level.
pub fn system_prompt(level: &PlayerLevel) -> &'static str {
    match level {
        PlayerLevel::Beginner => {
            "You are a friendly chess coach explaining a move to a beginner. \
             Use simple language. No chess notation beyond piece names and square names. \
             Use analogies when helpful. Keep responses to 1-3 sentences. \
             Generate ONLY the coaching text."
        }
        PlayerLevel::Intermediate => {
            "You are a chess coach speaking to a club-level player (~1000-1500). \
             Use standard chess terminology. Be direct and instructive. \
             Reference specific squares when relevant. Keep responses to 2-3 sentences. \
             Generate ONLY the coaching text."
        }
        PlayerLevel::UpperIntermediate => {
            "You are a chess coach speaking to a 1500-2000 rated player. \
             Use full chess terminology. Reference pawn structures, piece activity, \
             and strategic plans by name. Be concise and specific. \
             Keep responses to 2-4 sentences. Generate ONLY the coaching text."
        }
    }
}

/// Format a system + user message pair in Gemma 4 instruction format.
///
/// Gemma 4 replaced Gemma 2/3's `<start_of_turn>` markers with
/// `<|turn>{role}\n{content}<turn|>\n` turns and a `<|turn>model\n` generation
/// prompt (see `common_chat_params_init_gemma4` in llama.cpp's `common/chat.cpp`).
/// llama.cpp's C-side `llama_chat_apply_template` heuristics do not know this
/// template (only its Jinja engine does, which llama-cpp-2 does not expose),
/// so the format is applied here instead of via the model's chat template.
///
/// The BOS token is intentionally omitted — tokenization adds it (`AddBos::Always`).
pub fn format_chat(system: &str, user: &str) -> String {
    format!("<|turn>system\n{system}<turn|>\n<|turn>user\n{user}<turn|>\n<|turn>model\n")
}

/// Build a full coaching prompt in Gemma 4 instruction format.
#[allow(clippy::too_many_arguments)]
pub fn build_prompt(
    level: &PlayerLevel,
    classification: &str,
    phase: &str,
    player_move: &str,
    better_move: Option<&str>,
    themes: &[String],
    tactics: &[String],
    material_balance_cp: i32,
) -> String {
    let system = system_prompt(level);

    let better_move_json = match better_move {
        Some(m) => format!("\"{m}\""),
        None => "null".to_string(),
    };

    let themes_json: Vec<String> = themes.iter().map(|t| format!("\"{t}\"")).collect();
    let tactics_json: Vec<String> = tactics.iter().map(|t| format!("\"{t}\"")).collect();

    let context_json = format!(
        r#"{{"classification":"{}","phase":"{}","player_move":"{}","better_move":{},"themes":[{}],"tactics":[{}],"material_balance_cp":{}}}"#,
        classification,
        phase,
        player_move,
        better_move_json,
        themes_json.join(","),
        tactics_json.join(","),
        material_balance_cp,
    );

    format_chat(system, &context_json)
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

    format_chat(system, &context)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beginner_prompt_uses_simple_language() {
        let prompt = system_prompt(&PlayerLevel::Beginner);
        assert!(prompt.contains("beginner"));
        assert!(prompt.contains("simple language"));
    }

    #[test]
    fn intermediate_prompt_references_terminology() {
        let prompt = system_prompt(&PlayerLevel::Intermediate);
        assert!(prompt.contains("club-level"));
        assert!(prompt.contains("chess terminology"));
    }

    #[test]
    fn upper_intermediate_prompt_is_advanced() {
        let prompt = system_prompt(&PlayerLevel::UpperIntermediate);
        assert!(prompt.contains("1500-2000"));
        assert!(prompt.contains("pawn structures"));
    }

    #[test]
    fn format_chat_has_gemma4_markers() {
        let prompt = format_chat("system text", "user text");
        assert!(prompt.starts_with("<|turn>system\nsystem text<turn|>\n"));
        assert!(prompt.contains("<|turn>user\nuser text<turn|>\n"));
        assert!(prompt.ends_with("<|turn>model\n"));
    }

    #[test]
    fn build_prompt_has_gemma4_format_markers() {
        let prompt = build_prompt(
            &PlayerLevel::Beginner,
            "blunder",
            "middlegame",
            "Nf3",
            Some("e4"),
            &["knightOnRim".to_string()],
            &["fork".to_string()],
            -150,
        );
        assert!(prompt.contains("<|turn>system"));
        assert!(prompt.contains("<|turn>user"));
        assert!(prompt.contains("<turn|>"));
        assert!(prompt.ends_with("<|turn>model\n"));
    }

    #[test]
    fn build_prompt_contains_valid_json() {
        let prompt = build_prompt(
            &PlayerLevel::Intermediate,
            "mistake",
            "opening",
            "d4",
            None,
            &[],
            &[],
            0,
        );
        // Extract the JSON portion (the user turn content)
        let json_start = prompt.find('{').unwrap();
        let json_end = prompt.rfind('}').unwrap();
        let json_str = &prompt[json_start..=json_end];
        let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
        assert_eq!(parsed["classification"], "mistake");
        assert_eq!(parsed["better_move"], serde_json::Value::Null);
    }

    #[test]
    fn build_prompt_includes_themes_and_tactics() {
        let prompt = build_prompt(
            &PlayerLevel::UpperIntermediate,
            "inaccuracy",
            "endgame",
            "Kf1",
            Some("Ke2"),
            &["passedPawn".to_string(), "openFile".to_string()],
            &["pin".to_string()],
            200,
        );
        assert!(prompt.contains("passedPawn"));
        assert!(prompt.contains("openFile"));
        assert!(prompt.contains("pin"));
        assert!(prompt.contains("200"));
    }

    #[test]
    fn game_summary_prompt_contains_stats() {
        let prompt = build_game_summary_prompt("1-0", "checkmate", 34, 87.5, 10, 1, 2, 3);
        assert!(prompt.contains("checkmate"));
        assert!(prompt.contains("\"moves\":34"));
        assert!(prompt.contains("\"accuracy\":88"));
        assert!(prompt.ends_with("<|turn>model\n"));
    }

    #[test]
    fn all_three_levels_produce_distinct_prompts() {
        let b = system_prompt(&PlayerLevel::Beginner);
        let i = system_prompt(&PlayerLevel::Intermediate);
        let u = system_prompt(&PlayerLevel::UpperIntermediate);
        assert_ne!(b, i);
        assert_ne!(i, u);
        assert_ne!(b, u);
    }
}
