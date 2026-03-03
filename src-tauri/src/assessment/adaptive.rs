use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdaptivePrompt {
    pub prompt_type: AdaptivePromptType,
    pub message: String,
    pub suggestion: String,
    pub target_activity: String,
    pub target_category: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AdaptivePromptType {
    IncreaseChallenge,
    DecreaseChallenge,
    FrustrationDetected,
    PlateauDetected,
    None,
}

/// Pre-computed data for trigger detection (avoids DB dependency in logic).
pub struct AdaptiveTriggerData {
    pub recent_game_results: Vec<String>,
    pub recent_solve_rate: f64,
    pub recent_puzzle_count: u32,
    pub hint_usage_rate: f64,
    pub total_sessions: u32,
    pub rating_std_dev: f64,
    pub weakest_category: Option<String>,
}

/// Pure function: detect adaptive difficulty triggers from pre-computed data.
pub fn check_adaptive_triggers(data: &AdaptiveTriggerData) -> AdaptivePrompt {
    let none_prompt = AdaptivePrompt {
        prompt_type: AdaptivePromptType::None,
        message: String::new(),
        suggestion: String::new(),
        target_activity: String::new(),
        target_category: None,
    };

    // Priority 1: Frustration — 3+ resigns in last 5 games
    let resign_count = data
        .recent_game_results
        .iter()
        .filter(|r| r.contains("resign"))
        .count();
    if resign_count >= 3 {
        return AdaptivePrompt {
            prompt_type: AdaptivePromptType::FrustrationDetected,
            message: "It looks like the last few games have been tough. That's completely normal — even grandmasters have rough patches.".to_string(),
            suggestion: "Try lowering the engine difficulty or switching to some puzzles for a confidence boost.".to_string(),
            target_activity: "problems".to_string(),
            target_category: data.weakest_category.clone(),
        };
    }

    // Priority 2: Decrease difficulty — solve rate < 35% in recent puzzles, or hint usage > 60%
    if data.recent_puzzle_count >= 5 {
        if data.recent_solve_rate < 0.35 {
            return AdaptivePrompt {
                prompt_type: AdaptivePromptType::DecreaseChallenge,
                message: "The current puzzle difficulty seems a bit high — you're solving less than a third.".to_string(),
                suggestion: "Try easier puzzles to build foundational patterns before moving up.".to_string(),
                target_activity: "problems".to_string(),
                target_category: data.weakest_category.clone(),
            };
        }

        if data.hint_usage_rate > 0.60 {
            return AdaptivePrompt {
                prompt_type: AdaptivePromptType::DecreaseChallenge,
                message: "You're using hints on most puzzles — the difficulty might be too high."
                    .to_string(),
                suggestion: "Try slightly easier puzzles to build confidence without hints."
                    .to_string(),
                target_activity: "problems".to_string(),
                target_category: data.weakest_category.clone(),
            };
        }
    }

    // Priority 3: Plateau — 20+ sessions, low rating variance and minimal progress
    if data.total_sessions >= 20 && data.rating_std_dev < 30.0 {
        return AdaptivePrompt {
            prompt_type: AdaptivePromptType::PlateauDetected,
            message: "Your ratings have been stable for a while — you might be hitting a plateau."
                .to_string(),
            suggestion: format!(
                "Try focusing on {} — targeted practice breaks plateaus faster than general play.",
                data.weakest_category
                    .as_deref()
                    .unwrap_or("your weakest area")
            ),
            target_activity: "problems".to_string(),
            target_category: data.weakest_category.clone(),
        };
    }

    // Priority 4: Increase difficulty — solve rate > 75% in recent puzzles
    if data.recent_puzzle_count >= 5 && data.recent_solve_rate > 0.75 {
        return AdaptivePrompt {
            prompt_type: AdaptivePromptType::IncreaseChallenge,
            message: "You're crushing it! Your solve rate is well above 75%.".to_string(),
            suggestion: "Time to level up — try harder puzzles or a stronger engine opponent."
                .to_string(),
            target_activity: "problems".to_string(),
            target_category: data.weakest_category.clone(),
        };
    }

    none_prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_data() -> AdaptiveTriggerData {
        AdaptiveTriggerData {
            recent_game_results: Vec::new(),
            recent_solve_rate: 0.5,
            recent_puzzle_count: 10,
            hint_usage_rate: 0.2,
            total_sessions: 5,
            rating_std_dev: 100.0,
            weakest_category: Some("endgame".to_string()),
        }
    }

    #[test]
    fn no_triggers_returns_none() {
        let prompt = check_adaptive_triggers(&base_data());
        assert_eq!(prompt.prompt_type, AdaptivePromptType::None);
    }

    #[test]
    fn frustration_detected() {
        let mut data = base_data();
        data.recent_game_results = vec![
            "resign".to_string(),
            "resign".to_string(),
            "resign".to_string(),
            "1-0".to_string(),
            "resign".to_string(),
        ];
        let prompt = check_adaptive_triggers(&data);
        assert_eq!(prompt.prompt_type, AdaptivePromptType::FrustrationDetected);
    }

    #[test]
    fn decrease_low_solve_rate() {
        let mut data = base_data();
        data.recent_solve_rate = 0.20;
        let prompt = check_adaptive_triggers(&data);
        assert_eq!(prompt.prompt_type, AdaptivePromptType::DecreaseChallenge);
    }

    #[test]
    fn decrease_high_hint_usage() {
        let mut data = base_data();
        data.hint_usage_rate = 0.70;
        let prompt = check_adaptive_triggers(&data);
        assert_eq!(prompt.prompt_type, AdaptivePromptType::DecreaseChallenge);
    }

    #[test]
    fn plateau_detected() {
        let mut data = base_data();
        data.total_sessions = 25;
        data.rating_std_dev = 20.0;
        let prompt = check_adaptive_triggers(&data);
        assert_eq!(prompt.prompt_type, AdaptivePromptType::PlateauDetected);
    }

    #[test]
    fn increase_challenge() {
        let mut data = base_data();
        data.recent_solve_rate = 0.85;
        let prompt = check_adaptive_triggers(&data);
        assert_eq!(prompt.prompt_type, AdaptivePromptType::IncreaseChallenge);
    }

    #[test]
    fn frustration_takes_priority_over_decrease() {
        let mut data = base_data();
        data.recent_game_results = vec![
            "resign".to_string(),
            "resign".to_string(),
            "resign".to_string(),
        ];
        data.recent_solve_rate = 0.20;
        let prompt = check_adaptive_triggers(&data);
        assert_eq!(prompt.prompt_type, AdaptivePromptType::FrustrationDetected);
    }

    #[test]
    fn not_enough_puzzles_skips_rate_checks() {
        let mut data = base_data();
        data.recent_puzzle_count = 3;
        data.recent_solve_rate = 0.10;
        data.hint_usage_rate = 0.90;
        let prompt = check_adaptive_triggers(&data);
        assert_eq!(prompt.prompt_type, AdaptivePromptType::None);
    }
}
