use super::PlayerLevel;

/// System prompt tailored to the player's skill level.
#[allow(dead_code)]
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

/// Build a full prompt in Gemma 2 instruction format.
///
/// Format: `<start_of_turn>user\n{system}\n\n{json_context}<end_of_turn>\n<start_of_turn>model\n`
#[allow(dead_code)]
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
        Some(m) => format!("\"{}\"", m),
        None => "null".to_string(),
    };

    let themes_json: Vec<String> = themes.iter().map(|t| format!("\"{}\"", t)).collect();
    let tactics_json: Vec<String> = tactics.iter().map(|t| format!("\"{}\"", t)).collect();

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

    format!(
        "<start_of_turn>user\n{}\n\n{}<end_of_turn>\n<start_of_turn>model\n",
        system, context_json
    )
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
    fn build_prompt_has_gemma_format_markers() {
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
        assert!(prompt.contains("<start_of_turn>user"));
        assert!(prompt.contains("<end_of_turn>"));
        assert!(prompt.contains("<start_of_turn>model"));
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
        // Extract the JSON portion (between system prompt and end_of_turn)
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
    fn all_three_levels_produce_distinct_prompts() {
        let b = system_prompt(&PlayerLevel::Beginner);
        let i = system_prompt(&PlayerLevel::Intermediate);
        let u = system_prompt(&PlayerLevel::UpperIntermediate);
        assert_ne!(b, i);
        assert_ne!(i, u);
        assert_ne!(b, u);
    }
}
