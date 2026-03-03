use serde::Serialize;
use shakmaty::{uci::UciMove, Position};
use tauri::State;
use tokio::sync::Mutex;
use tracing::debug;

use crate::engine::process::EngineProcess;
use crate::error::AppError;
use crate::heuristics;
use crate::opponent::personality::PersonalityProfile;
use crate::opponent::selector;
use crate::opponent::teaching;

/// The result of selecting an opponent move with personality.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectedMove {
    pub uci: String,
    pub personality: PersonalityProfile,
    pub personality_score: f64,
    pub teaching_score: f64,
}

/// Get the opponent's move using personality-based selection.
///
/// 1. Runs multi-PV at full strength to find candidate moves
/// 2. Analyzes each candidate's resulting position with heuristics
/// 3. Scores candidates against personality weights + optional teaching bonus
/// 4. Selects via softmax-weighted random choice
#[tauri::command]
pub async fn get_opponent_move(
    fen: String,
    depth: Option<u32>,
    personality: PersonalityProfile,
    teaching_mode: bool,
    weak_categories: Option<Vec<String>>,
    engine_state: State<'_, Mutex<EngineProcess>>,
) -> Result<SelectedMove, AppError> {
    let search_depth = depth.unwrap_or(14);
    let num_pvs = 5u32;

    debug!(
        "Opponent move: personality={personality:?}, teaching={teaching_mode}, depth={search_depth}"
    );

    // Step 1: Get multi-PV candidates at full strength
    let mut engine = engine_state.lock().await;
    let candidates = engine.get_multi_pv(&fen, search_depth, num_pvs).await?;
    drop(engine);

    if candidates.is_empty() {
        return Err(crate::error::EngineError::ProcessError(
            "No candidate moves from engine".to_string(),
        )
        .into());
    }

    // If only 1 candidate (forced move), return it directly
    if candidates.len() == 1 {
        return Ok(SelectedMove {
            uci: candidates[0].uci_move.clone(),
            personality,
            personality_score: 1.0,
            teaching_score: 0.0,
        });
    }

    // Step 2: For each candidate, play the move and analyze the resulting position
    let base_pos = crate::game::parse_fen(&fen)?;

    let mut contexts: Vec<(String, crate::models::heuristics::CoachingContext)> = Vec::new();

    for candidate in &candidates {
        let uci: UciMove = candidate.uci_move.parse().map_err(|e| {
            crate::error::EngineError::ProcessError(format!(
                "Bad UCI move {}: {e}",
                candidate.uci_move
            ))
        })?;

        let legal_move = uci.to_move(&base_pos).map_err(|e| {
            crate::error::EngineError::ProcessError(format!(
                "Illegal move {}: {e}",
                candidate.uci_move
            ))
        })?;

        let mut pos_after = base_pos.clone();
        pos_after.play_unchecked(&legal_move);

        let ctx = heuristics::analyze_position(&pos_after);
        contexts.push((candidate.uci_move.clone(), ctx));
    }

    // Step 3: Compute teaching scores if teaching mode is active
    let teaching_scores: Vec<(String, f64)> = if teaching_mode {
        let weaknesses: Vec<teaching::WeaknessCategory> = weak_categories
            .unwrap_or_default()
            .iter()
            .filter_map(|c| teaching::category_to_weakness(c))
            .collect();

        contexts
            .iter()
            .map(|(uci, ctx)| {
                let score = teaching::score_teaching_opportunity(ctx, &weaknesses);
                (uci.clone(), score)
            })
            .collect()
    } else {
        vec![]
    };

    // Step 4: Select using personality weights + softmax
    let weights = personality.weights();
    let selected = selector::select_move(&candidates, &weights, &contexts, &teaching_scores)
        .ok_or_else(|| {
            crate::error::EngineError::ProcessError("Move selection failed".to_string())
        })?;

    debug!(
        "Selected {}: personality={:.2}, teaching={:.2}, combined={:.2}",
        selected.uci_move,
        selected.personality_score,
        selected.teaching_score,
        selected.combined_score
    );

    Ok(SelectedMove {
        uci: selected.uci_move,
        personality,
        personality_score: selected.personality_score,
        teaching_score: selected.teaching_score,
    })
}

/// Resolve the personality profile from an OpponentMode.
///
/// - Choose: returns the explicitly provided personality
/// - Surprise: random selection
/// - CoachPicks: selects based on player's skill weaknesses
#[tauri::command]
pub fn resolve_personality(
    mode: crate::opponent::personality::OpponentMode,
    explicit: Option<PersonalityProfile>,
    db: State<'_, std::sync::Mutex<crate::db::connection::Database>>,
    player_state: State<'_, crate::CurrentPlayerId>,
) -> Result<PersonalityProfile, AppError> {
    match mode {
        crate::opponent::personality::OpponentMode::Choose => {
            Ok(explicit.unwrap_or(PersonalityProfile::Solid))
        }
        crate::opponent::personality::OpponentMode::Surprise => Ok(PersonalityProfile::random()),
        crate::opponent::personality::OpponentMode::CoachPicks => {
            // Read skill profile and pick a personality that challenges weaknesses
            let player_id = player_state.get()?;

            let db_lock = db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
            let profile = db_lock.get_skill_profile(&player_id)?;

            // Find the weakest category
            let weakest = profile.ratings.iter().min_by(|a, b| {
                a.rating
                    .partial_cmp(&b.rating)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            let personality = match weakest.map(|r| r.category.as_str()) {
                Some("tactical") => PersonalityProfile::Trappy,
                Some("endgame") => PersonalityProfile::Positional,
                Some("opening") => PersonalityProfile::Aggressive,
                Some("positional") => PersonalityProfile::Aggressive,
                _ => PersonalityProfile::Solid,
            };

            Ok(personality)
        }
    }
}
